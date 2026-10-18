//! Steps 2-3 for the face folders (`plans/export_upgrader.md` steps 2-3):
//! each becomes its player folder under `Players/`, named by the plan's
//! order (the face folder's name part, the save's display name, the number
//! alone), its portrait renamed `portrait.dds` and its `ingame_face` marker
//! normalized. Boots, gloves, merges, shared folders and links are 6.2b's;
//! `settings.toml` is 6.5's; a referee export's face folders stay unmigrated
//! content here (6.6's), and loose mode builds the folders without the
//! save's names and empty slots (6.7's).

use std::collections::{BTreeMap, BTreeSet};

use studio_core::{Disposition, Message, Scope};
use teams_list::TeamId;
use vtree::ScopePath;

use crate::messages::{Code, INPUT, tool_message};
use crate::roster::Roster;

/// What steps 2-3 make of the export's face folders.
#[derive(Debug, Default)]
pub(crate) struct PlayerFolders {
    /// Files to copy: input path → output path (both `/`-joined,
    /// root-relative).
    pub(crate) copies: Vec<(String, String)>,
    /// Empty marker files to write, output paths (`Players/15 -
    /// Snuffy/ingame_face`).
    pub(crate) markers: Vec<String>,
    /// Every input file the plan accounts for (copied, refused as part of a
    /// refused folder, or a marker it normalized): the unmigrated-content
    /// loop skips them.
    pub(crate) consumed: BTreeSet<String>,
    /// The findings on the input, in input-path order.
    pub(crate) findings: Vec<Message>,
}

/// The slot a face folder's name gives: characters 4-5, after any three
/// characters (`XXX` or a baked-in team ID, `71416`), two ASCII digits making
/// 01-23.
fn slot_of(name: &str) -> Option<u8> {
    let digits = name.get(3..5)?;
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u8>().ok().filter(|nn| (1..=23).contains(nn))
}

/// A candidate name as a folder name can hold it: the characters `vtree`'s
/// `validate_segments` refuses — control characters and `< > : " / \ | ? *` —
/// dropped, then trailing `.` and spaces trimmed (a name cannot end in them
/// either: `T.O.M.` → `T.O.M`); an empty remainder counts as absent.
fn clean_folder_name(name: &str) -> Option<String> {
    let cleaned: String = name
        .chars()
        .filter(|c| {
            !c.is_control() && !matches!(c, '<' | '>' | ':' | '"' | '|' | '?' | '*' | '/' | '\\')
        })
        .collect();
    let trimmed = cleaned.trim_end_matches(['.', ' ']);
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// What steps 2-3 make of `files`' face folders, for the resolved `team` (a
/// referee export's `None`: nothing is planned) with `roster` for the names
/// and empty slots (`None` in loose mode).
pub(crate) fn face_folders(
    files: &[String],
    team: Option<TeamId>,
    roster: Option<&Roster>,
) -> PlayerFolders {
    let mut plan = PlayerFolders::default();
    if team.is_none() {
        return plan;
    }
    // The files by face folder: `Faces/<folder>/...`, the top folder in any
    // letter case; a file directly in `Faces/` belongs to none.
    let mut folders: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for file in files {
        let mut parts = file.splitn(3, '/');
        let (Some(top), Some(folder), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if top.eq_ignore_ascii_case("faces") {
            folders
                .entry(format!("{top}/{folder}"))
                .or_default()
                .push(file);
        }
    }
    let scope = |path: &str| {
        ScopePath::new(path).expect("every listed path was checked when the source opened")
    };
    let mut taken = BTreeSet::new();
    for (folder, folder_files) in folders {
        let folder_name = folder.rsplit('/').next().unwrap_or(folder.as_str());
        let refused = |plan: &mut PlayerFolders| {
            plan.findings.push(tool_message(
                Code::UnmigratedContent,
                Scope::Folder {
                    export_id: INPUT,
                    path: scope(&folder),
                },
                Disposition::DropFolder,
                vec![],
            ));
            plan.consumed
                .extend(folder_files.iter().map(|file| (*file).clone()));
        };
        // A slot that is not two digits making 01-23 is unmigrated content as
        // a whole folder, as is the second face folder of a slot (the first
        // by name is kept).
        let Some(nn) = slot_of(folder_name) else {
            refused(&mut plan);
            continue;
        };
        if !taken.insert(nn) {
            refused(&mut plan);
            continue;
        }
        // The name part is what follows the five characters, its leading
        // spaces and hyphens trimmed.
        let name_part = folder_name
            .get(5..)
            .unwrap_or("")
            .trim_start_matches([' ', '-']);
        let name = clean_folder_name(name_part).or_else(|| {
            roster
                .and_then(|roster| roster.get(nn))
                .and_then(|player| clean_folder_name(&player.name))
        });
        if roster.is_some_and(|roster| roster.get(nn).is_none()) {
            // An empty slot still becomes its player folder, with no
            // `settings.toml`; the compiler's validation rules on it as for
            // any export.
            plan.findings.push(tool_message(
                Code::RosterSlotEmpty,
                Scope::Folder {
                    export_id: INPUT,
                    path: scope(&folder),
                },
                Disposition::Keep,
                vec![("slot", format!("{nn:02}"))],
            ));
        }
        let player_folder = match name {
            Some(name) => format!("Players/{nn:02} - {name}"),
            None => format!("Players/{nn:02}"),
        };
        // Every file copies below the player folder at its relative path,
        // except the portrait (`*portrait.dds`, renamed when alone) and the
        // `ingame_face` spellings (normalized to the bare marker).
        let mut marker = false;
        let mut portraits = Vec::new();
        for file in &folder_files {
            plan.consumed.insert((*file).clone());
            let relative = &file[folder.len() + 1..];
            if relative.contains('/') {
                plan.copies
                    .push(((*file).clone(), format!("{player_folder}/{relative}")));
                continue;
            }
            let lowered = relative.to_ascii_lowercase();
            if lowered == "ingame_face" || lowered == "ingame_face.txt" {
                marker = true;
            } else if lowered.ends_with("portrait.dds") {
                portraits.push((*file, relative));
            } else {
                plan.copies
                    .push(((*file).clone(), format!("{player_folder}/{relative}")));
            }
        }
        match portraits.as_slice() {
            [] => {}
            [(file, _)] => plan
                .copies
                .push(((*file).clone(), format!("{player_folder}/portrait.dds"))),
            many => {
                // One too many: each is reported and left in the folder as
                // named.
                for &(file, relative) in many {
                    plan.findings.push(tool_message(
                        Code::UnmigratedContent,
                        Scope::File {
                            export_id: INPUT,
                            path: scope(file),
                        },
                        Disposition::Keep,
                        vec![],
                    ));
                    plan.copies
                        .push(((*file).clone(), format!("{player_folder}/{relative}")));
                }
            }
        }
        if marker {
            plan.markers.push(format!("{player_folder}/ingame_face"));
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use studio_core::Severity;
    use teams_list::TeamId;

    use super::*;
    use crate::roster::RosteredPlayer;

    fn team() -> TeamId {
        TeamId::new(714).unwrap()
    }

    /// A roster of (slot → name) pairs; boots and gloves are 6.2b's.
    fn roster(entries: &[(u8, &str)]) -> Roster {
        Roster(
            entries
                .iter()
                .map(|(slot, name)| {
                    (
                        *slot,
                        RosteredPlayer {
                            name: (*name).to_owned(),
                            boots_id: 0,
                            gloves_id: 0,
                        },
                    )
                })
                .collect::<BTreeMap<u8, RosteredPlayer>>(),
        )
    }

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|path| (*path).to_owned()).collect()
    }

    #[test]
    fn the_slot_is_two_digits_making_01_to_23() {
        for (name, accepted) in [
            ("XXX01", true),
            ("XXX15", true),
            ("71416", true),
            ("XXX23", true),
            ("XXX00", false),
            ("XXX24", false),
            ("XXXab", false),
            ("XX15", false),
            ("ab15x", false),
        ] {
            let plan = face_folders(
                &files(&[&format!("Faces/{name}/face.fmdl")]),
                Some(team()),
                None,
            );
            assert_eq!(
                plan.copies.is_empty(),
                !accepted,
                "{name}: {:?}",
                plan.findings
            );
            if !accepted {
                assert!(plan.consumed.contains(&format!("Faces/{name}/face.fmdl")));
                assert_eq!(
                    plan.findings[0].scope,
                    Scope::Folder {
                        export_id: INPUT,
                        path: ScopePath::new(&format!("Faces/{name}")).unwrap()
                    },
                    "{name}"
                );
            }
        }
    }

    #[test]
    fn the_first_folder_of_a_slot_by_name_is_kept() {
        let plan = face_folders(
            &files(&[
                "Faces/XXX15 - Alpha/face.fmdl",
                "Faces/XXX15 - Beta/face.fmdl",
            ]),
            Some(team()),
            None,
        );
        let refused: Vec<&Message> = plan
            .findings
            .iter()
            .filter(|m| m.code.code == "unmigrated_content")
            .collect();
        assert_eq!(refused.len(), 1);
        assert_eq!(
            refused[0].scope,
            Scope::Folder {
                export_id: INPUT,
                path: ScopePath::new("Faces/XXX15 - Beta").unwrap()
            }
        );
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to.starts_with("Players/15 - Alpha/"))
        );
        assert!(!plan.copies.iter().any(|(_, to)| to.contains("Beta")));
    }

    #[test]
    fn the_name_comes_from_the_face_folder_then_the_save_then_the_number() {
        // The face folder's own name part wins over the save's.
        let list = files(&["Faces/XXX09 - FaceName/face.fmdl"]);
        let plan = face_folders(&list, Some(team()), Some(&roster(&[(9, "SaveName")])));
        assert_eq!(plan.copies[0].1, "Players/09 - FaceName/face.fmdl");
        // Without a name part the save's name is taken.
        let list = files(&["Faces/XXX09/face.fmdl"]);
        let plan = face_folders(&list, Some(team()), Some(&roster(&[(9, "SaveName")])));
        assert_eq!(plan.copies[0].1, "Players/09 - SaveName/face.fmdl");
        // With neither, the number alone names the folder.
        let plan = face_folders(&list, Some(team()), None);
        assert_eq!(plan.copies[0].1, "Players/09/face.fmdl");
        // A name part that cleans to nothing is absent too.
        let list = files(&["Faces/XXX09 -   ./face.fmdl"]);
        let plan = face_folders(&list, Some(team()), None);
        assert_eq!(plan.copies[0].1, "Players/09/face.fmdl");
    }

    #[test]
    fn a_cleaned_name_is_always_a_valid_segment() {
        for (input, expected) in [
            ("Snuffy", Some("Snuffy")),
            ("T.O.M.", Some("T.O.M")),
            ("a:b?c", Some("abc")),
            ("a/b\\c", Some("abc")),
            ("a b ", Some("a b")),
            ("x  .  ", Some("x")),
            ("...", None),
            ("   ", None),
        ] {
            let cleaned = clean_folder_name(input);
            assert_eq!(cleaned.as_deref(), expected, "{input:?}");
            if let Some(cleaned) = &cleaned {
                assert!(ScopePath::new(cleaned).is_ok(), "{cleaned:?}");
            }
        }
    }

    #[test]
    fn either_marker_spelling_becomes_one_bare_marker() {
        for names in [
            vec!["ingame_face"],
            vec!["ingame_face.txt"],
            vec!["ingame_face", "ingame_face.txt"],
        ] {
            let files: Vec<String> = names
                .iter()
                .map(|name| format!("Faces/XXX05 - A/{name}"))
                .chain(std::iter::once("Faces/XXX05 - A/face.fmdl".to_owned()))
                .collect();
            let plan = face_folders(&files, Some(team()), None);
            assert_eq!(
                plan.markers,
                ["Players/05 - A/ingame_face".to_owned()],
                "{names:?}"
            );
            assert!(!plan.copies.iter().any(|(_, to)| to.contains("ingame_face")));
            assert!(
                plan.consumed
                    .contains(&format!("Faces/XXX05 - A/{}", names[0]))
            );
        }
    }

    #[test]
    fn one_portrait_is_renamed_and_two_are_reported_as_named() {
        let list = files(&[
            "Faces/XXX15 - A/x_portrait.dds",
            "Faces/XXX15 - A/face.fmdl",
        ]);
        let plan = face_folders(&list, Some(team()), None);
        assert!(plan.copies.contains(&(
            "Faces/XXX15 - A/x_portrait.dds".to_owned(),
            "Players/15 - A/portrait.dds".to_owned()
        )));
        let list = files(&[
            "Faces/XXX16 - A/portrait.dds",
            "Faces/XXX16 - A/alt_portrait.dds",
        ]);
        let plan = face_folders(&list, Some(team()), None);
        let reported: Vec<&Message> = plan
            .findings
            .iter()
            .filter(|m| m.code.code == "unmigrated_content")
            .collect();
        assert_eq!(reported.len(), 2);
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Players/16 - A/portrait.dds")
        );
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Players/16 - A/alt_portrait.dds")
        );
    }

    #[test]
    fn an_empty_roster_slot_is_a_warning_and_the_folder_still_forms() {
        let files = files(&["Faces/XXX23 - Pinto/face.fmdl"]);
        let plan = face_folders(&files, Some(team()), Some(&roster(&[])));
        let warning = plan
            .findings
            .iter()
            .find(|m| m.code.code == "roster_slot_empty")
            .expect("roster_slot_empty expected");
        assert_eq!(warning.severity, Severity::Warning);
        assert_eq!(warning.disposition, Disposition::Keep);
        assert_eq!(warning.context, vec![("slot".to_owned(), "23".to_owned())]);
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to.starts_with("Players/23 - Pinto/"))
        );
    }

    #[test]
    fn a_file_below_a_subfolder_of_another_top_folder_is_not_planned() {
        // A file below `Other/` or `Boots/` is not a face folder's.
        let plan = face_folders(
            &files(&[
                "Faces/XXX15 - A/face.fmdl",
                "Other/docs/readme.txt",
                "Boots/k0430 - Crocs/boots.fmdl",
            ]),
            Some(team()),
            None,
        );
        let scopes: Vec<String> = plan
            .findings
            .iter()
            .map(|m| match &m.scope {
                Scope::Folder { path, .. } | Scope::File { path, .. } => path.as_str().to_owned(),
                Scope::Run | Scope::Export { .. } | Scope::RosterEntry { .. } => String::new(),
            })
            .collect();
        for path in ["Other/docs/readme.txt", "Boots/k0430 - Crocs/boots.fmdl"] {
            assert!(!plan.consumed.contains(path), "{path}");
            assert!(
                !plan.copies.iter().any(|(_, to)| to.contains(path)),
                "{path}"
            );
            assert!(scopes.iter().all(|s| s != path), "{path}: {scopes:?}");
        }
        assert_eq!(plan.copies.len(), 1);
    }

    #[test]
    fn a_file_directly_in_faces_and_a_referee_export_belong_to_no_plan() {
        let files = files(&["Faces/loose.fmdl", "Faces/XXX15 - A/face.fmdl"]);
        let plan = face_folders(&files, Some(team()), None);
        assert!(!plan.consumed.contains("Faces/loose.fmdl"));
        let plan = face_folders(&files, None, None);
        assert!(plan.copies.is_empty() && plan.consumed.is_empty());
    }
}
