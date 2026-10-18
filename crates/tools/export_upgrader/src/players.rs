//! Steps 2-4 for the face folders and the boots and gloves folders
//! (`plans/export_upgrader.md` steps 2-4): each face folder becomes its player
//! folder under `Players/`, named by the plan's order (the face folder's name
//! part, the worn boots folder's, the worn gloves folder's, the save's name,
//! the number alone), its portrait renamed `portrait.dds` and its
//! `ingame_face` marker normalized. A boots or gloves folder one player wears
//! merges into his folder on a Fox export; one worn by two or more, or any
//! worn one on a pre-Fox export, stays a named shared folder with a link file.
//! Each slot's output folder lands in `PlayerFolders::folders` for step 8's
//! `settings.toml` (`settings.rs`); a referee export's folders are
//! `referees.rs`'s, and loose mode builds the player folders without the
//! save's names and leaves the item folders unmigrated (6.7's).

use std::collections::{BTreeMap, BTreeSet};

use pes_version::Engine;
use studio_core::{Disposition, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, INPUT, tool_message};
use crate::roster::Roster;

/// What steps 2-4 make of the export's face and item folders.
#[derive(Debug, Default)]
pub(crate) struct PlayerFolders {
    /// Files to copy: input path → output path (both `/`-joined,
    /// root-relative).
    pub(crate) copies: Vec<(String, String)>,
    /// Files copied at their input paths below `Players/`, which step 7's name
    /// respells, renames and reference rewrites skip: a referee folder step 12
    /// leaves as it was (empty for a team export).
    pub(crate) untouched: Vec<(String, String)>,
    /// Empty marker files to write, output paths (`Players/15 -
    /// Snuffy/ingame_face`, `Players/15 - Snuffy/Crocs.boots`).
    pub(crate) markers: Vec<String>,
    /// Every input file the plan accounts for (copied, refused as part of a
    /// refused folder, or a marker it normalized): the unmigrated-content
    /// loop skips them.
    pub(crate) consumed: BTreeSet<String>,
    /// The findings on the input, in input-path order.
    pub(crate) findings: Vec<Message>,
    /// Each slot's face-folder portrait: slot (1-23) → its input path, for
    /// step 7's comparison with `Portraits/`.
    pub(crate) portraits: BTreeMap<u8, String>,
    /// Each slot's output player folder (1-23), for step 8.
    pub(crate) folders: BTreeMap<u8, OutputFolder>,
}

/// One slot's output player folder as steps 3-4 leave it.
#[derive(Debug)]
pub(crate) struct OutputFolder {
    /// The folder's output path (`Players/15 - Snuffy`).
    pub(crate) path: String,
    /// A boots folder of his worn ID was merged into it or is linked from it.
    pub(crate) boots_folder: bool,
    /// The same for a gloves folder.
    pub(crate) gloves_folder: bool,
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

/// One slot's player folder as the plan builds it: the face folder's files
/// and every folder merged into it later, keyed by relative path for the
/// byte clash check.
struct PlayerFolder {
    /// The face folder's name part, leading spaces and hyphens off; `""`
    /// when the slot has no kept face folder.
    face_name_part: String,
    /// The worn boots folder's name part, `""` when none.
    boots_name_part: String,
    /// The worn gloves folder's name part, `""` when none.
    gloves_name_part: String,
    /// A boots folder of his worn ID was merged into the folder or is linked
    /// from it (step 8's "the folder decides the category").
    boots_folder: bool,
    /// The same for a gloves folder.
    gloves_folder: bool,
    /// A kept face folder exists for the slot.
    has_face: bool,
    /// The bare `ingame_face` marker to write.
    marker: bool,
    /// The folder's files: relative path → input path.
    files: BTreeMap<String, String>,
    /// The shared-folder link file names (`Crocs.boots`, `Mitts.gloves`).
    links: BTreeSet<String>,
}

impl PlayerFolder {
    fn new() -> PlayerFolder {
        PlayerFolder {
            face_name_part: String::new(),
            boots_name_part: String::new(),
            gloves_name_part: String::new(),
            boots_folder: false,
            gloves_folder: false,
            has_face: false,
            marker: false,
            files: BTreeMap::new(),
            links: BTreeSet::new(),
        }
    }
}

/// A path of the listing as a `ScopePath`: every listed path was checked
/// when the source opened.
pub(crate) fn path_scope(path: &str) -> ScopePath {
    ScopePath::new(path).expect("every listed path was checked when the source opened")
}

/// One kind of item folder (the boots folders' `k`, the gloves' `g`, lower
/// case as the old format requires).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemKind {
    Boots,
    Gloves,
}

impl ItemKind {
    /// Its top folder, matched in any letter case.
    fn top(self) -> &'static str {
        match self {
            ItemKind::Boots => "boots",
            ItemKind::Gloves => "gloves",
        }
    }

    /// Its name's first character.
    fn letter(self) -> char {
        match self {
            ItemKind::Boots => 'k',
            ItemKind::Gloves => 'g',
        }
    }

    /// Its shared output folder.
    fn shared_top(self) -> &'static str {
        match self {
            ItemKind::Boots => "Boots",
            ItemKind::Gloves => "Gloves",
        }
    }

    /// Its link file's extension.
    fn link_ext(self) -> &'static str {
        match self {
            ItemKind::Boots => "boots",
            ItemKind::Gloves => "gloves",
        }
    }

    /// The worn ID a rostered player carries for this kind.
    fn worn_id(self, player: &crate::roster::RosteredPlayer) -> u32 {
        match self {
            ItemKind::Boots => player.boots_id,
            ItemKind::Gloves => player.gloves_id,
        }
    }
}

/// An item folder's name: `k` (boots) or `g` (gloves), lower case, then four
/// ASCII digits, then its name part (leading spaces and hyphens trimmed, as
/// a face folder's); a gloves folder named `g` and exactly three digits and
/// nothing else reads as four (`g567` → ID 567). `None` for any other
/// spelling (`K0430`, `k430 - X`, `boots`).
fn parse_item_name(kind: ItemKind, name: &str) -> Option<(u32, String)> {
    let name = name.strip_prefix(kind.letter())?;
    if kind == ItemKind::Gloves && name.len() == 3 && name.bytes().all(|b| b.is_ascii_digit()) {
        return Some((name.parse().ok()?, String::new()));
    }
    let digits = name.get(..4)?;
    if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some((
        digits.parse().ok()?,
        name[4..].trim_start_matches([' ', '-']).to_owned(),
    ))
}

/// The grouped files by folder under `top` (any letter case): every file
/// below a direct subfolder, mapped `top/folder` → its files. A file
/// directly in `top` belongs to none.
fn folders_below<'a>(files: &'a [String], top: &str) -> BTreeMap<String, Vec<&'a String>> {
    let mut folders: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for file in files {
        let mut parts = file.splitn(3, '/');
        let (Some(head), Some(folder), Some(_)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        if head.eq_ignore_ascii_case(top) {
            folders
                .entry(format!("{head}/{folder}"))
                .or_default()
                .push(file);
        }
    }
    folders
}

/// What steps 2-3 make of `files`' face folders for a team export (a referee
/// export's folders are `referees.rs`'s: `run` does not call this for one) with
/// `roster` for the names and empty slots (`None` in loose mode), and steps
/// 3-4 make of its boots and gloves folders for `engine` and `same_bytes`.
pub(crate) fn player_folders(
    files: &[String],
    roster: Option<&Roster>,
    engine: Engine,
    mut same_bytes: impl FnMut(&str, &str) -> bool,
) -> PlayerFolders {
    let mut plan = PlayerFolders::default();
    let mut players: BTreeMap<u8, PlayerFolder> = BTreeMap::new();
    // Step 3's face folders.
    face_folders(files, roster, &mut players, &mut plan);
    // Steps 3-4's item folders: who wears each is the save's, so a roster is
    // needed — loose mode leaves them unmigrated until its wearer inference
    // lands (6.7).
    if let Some(roster) = roster {
        for kind in [ItemKind::Boots, ItemKind::Gloves] {
            item_folders(
                files,
                kind,
                roster,
                engine,
                &mut same_bytes,
                &mut players,
                &mut plan,
            );
        }
    }
    // The player folder names and copies, every slot in order: the name is
    // the first non-empty candidate, cleaned — the face folder's name part,
    // then the worn boots folder's, then the worn gloves folder's, then the
    // save's name, then `NN` alone.
    for (nn, player) in &mut players {
        let candidates = [
            player.face_name_part.as_str(),
            player.boots_name_part.as_str(),
            player.gloves_name_part.as_str(),
            roster
                .and_then(|roster| roster.get(*nn))
                .map_or("", |p| p.name.as_str()),
        ];
        let name = candidates
            .iter()
            .find_map(|candidate| clean_folder_name(candidate));
        let folder = match name {
            Some(name) => format!("Players/{nn:02} - {name}"),
            None => format!("Players/{nn:02}"),
        };
        for (relative, source) in &player.files {
            plan.copies
                .push((source.clone(), format!("{folder}/{relative}")));
        }
        if player.marker {
            plan.markers.push(format!("{folder}/ingame_face"));
        }
        for link in &player.links {
            plan.markers.push(format!("{folder}/{link}"));
        }
        plan.folders.insert(
            *nn,
            OutputFolder {
                path: folder,
                boots_folder: player.boots_folder,
                gloves_folder: player.gloves_folder,
            },
        );
    }
    plan
}

/// Step 3's face folders into `players`: each becomes its player folder,
/// the portrait renamed and the `ingame_face` spellings normalized.
fn face_folders(
    files: &[String],
    roster: Option<&Roster>,
    players: &mut BTreeMap<u8, PlayerFolder>,
    plan: &mut PlayerFolders,
) {
    let mut taken = BTreeSet::new();
    for (folder, folder_files) in folders_below(files, "faces") {
        let folder_name = folder.rsplit('/').next().unwrap_or(folder.as_str());
        let refused = |plan: &mut PlayerFolders| {
            plan.findings.push(tool_message(
                Code::UnmigratedContent,
                Scope::Folder {
                    export_id: INPUT,
                    path: path_scope(&folder),
                },
                Disposition::DropFolder,
                vec![],
            ));
            plan.consumed
                .extend(folder_files.iter().map(|file| (*file).clone()));
        };
        // A slot that is not two digits making 01 to 23 is unmigrated content as
        // a whole folder, as is the second face folder of a slot (the first
        // by name is kept).
        let Some(nn) = slot_of(folder_name) else {
            refused(&mut *plan);
            continue;
        };
        if !taken.insert(nn) {
            refused(&mut *plan);
            continue;
        }
        // The name part is what follows the five characters, its leading
        // spaces and hyphens trimmed.
        let name_part = folder_name
            .get(5..)
            .unwrap_or("")
            .trim_start_matches([' ', '-']);
        if roster.is_some_and(|roster| roster.get(nn).is_none()) {
            // An empty slot still becomes its player folder, with no
            // `settings.toml`; the compiler's validation rules on it as for
            // any export.
            plan.findings.push(tool_message(
                Code::RosterSlotEmpty,
                Scope::Folder {
                    export_id: INPUT,
                    path: path_scope(&folder),
                },
                Disposition::Keep,
                vec![("slot", format!("{nn:02}"))],
            ));
        }
        let player = players.entry(nn).or_insert_with(PlayerFolder::new);
        player.face_name_part = name_part.to_owned();
        player.has_face = true;
        // Every file lands under the player folder at its relative path,
        // except the portrait (`*portrait.dds`, renamed when alone) and the
        // `ingame_face` spellings (normalized to the bare marker).
        let mut portraits = Vec::new();
        for file in &folder_files {
            plan.consumed.insert((*file).clone());
            let relative = &file[folder.len() + 1..];
            if relative.contains('/') {
                player.files.insert(relative.to_owned(), (*file).clone());
                continue;
            }
            let lowered = relative.to_ascii_lowercase();
            if lowered == "ingame_face" || lowered == "ingame_face.txt" {
                player.marker = true;
            } else if lowered.ends_with("portrait.dds") {
                portraits.push((*file, relative));
            } else {
                player.files.insert(relative.to_owned(), (*file).clone());
            }
        }
        match portraits.as_slice() {
            [] => {}
            [(file, _)] => {
                player
                    .files
                    .insert("portrait.dds".to_owned(), (*file).clone());
                plan.portraits.insert(nn, (*file).clone());
            }
            many => {
                // One too many: each is reported and left in the folder as
                // named.
                for &(file, relative) in many {
                    plan.findings.push(tool_message(
                        Code::UnmigratedContent,
                        Scope::File {
                            export_id: INPUT,
                            path: path_scope(file),
                        },
                        Disposition::Keep,
                        vec![],
                    ));
                    player.files.insert(relative.to_owned(), (*file).clone());
                }
            }
        }
    }
}

/// Steps 3-4's item folders of `kind`: name, ID and wearers resolve, then
/// each is merged into its one wearer's folder (Fox only), or kept shared
/// with a link.
fn item_folders(
    files: &[String],
    kind: ItemKind,
    roster: &Roster,
    engine: Engine,
    same_bytes: &mut impl FnMut(&str, &str) -> bool,
    players: &mut BTreeMap<u8, PlayerFolder>,
    plan: &mut PlayerFolders,
) {
    /// A shared folder as the inputs give it: name part or the ID spelling,
    /// colliding names resolved below.
    struct Shared {
        id: u32,
        name_part: String,
        files: Vec<(String, String)>,
        wearers: Vec<u8>,
    }
    /// Whether `other` holds the same files as `first`: the same relative
    /// paths, letter case aside, each byte for byte.
    fn same_files(
        first: &Shared,
        other: &Shared,
        same_bytes: &mut impl FnMut(&str, &str) -> bool,
    ) -> bool {
        fn by_folded_path(item: &Shared) -> BTreeMap<String, &String> {
            item.files
                .iter()
                .map(|(input, relative)| (vtree::fold_name(relative), input))
                .collect()
        }
        let (first_files, other_files) = (by_folded_path(first), by_folded_path(other));
        first_files.len() == other_files.len()
            && other_files.iter().all(|(path, input)| {
                first_files
                    .get(path)
                    .is_some_and(|first_input| same_bytes(first_input, input))
            })
    }
    let mut taken = BTreeSet::new();
    let mut shared: Vec<Shared> = Vec::new();
    for (folder, folder_files) in folders_below(files, kind.top()) {
        let folder_name = folder.rsplit('/').next().unwrap_or(folder.as_str());
        let refused = |plan: &mut PlayerFolders| {
            plan.findings.push(tool_message(
                Code::UnmigratedContent,
                Scope::Folder {
                    export_id: INPUT,
                    path: path_scope(&folder),
                },
                Disposition::DropFolder,
                vec![],
            ));
            plan.consumed
                .extend(folder_files.iter().map(|file| (*file).clone()));
        };
        // Any other spelling is unmigrated content as a whole folder, as is
        // the second folder of one kind with one ID (the first by name) and
        // a folder no rostered player wears.
        let Some((id, name_part)) = parse_item_name(kind, folder_name) else {
            refused(&mut *plan);
            continue;
        };
        if !taken.insert(id) {
            refused(&mut *plan);
            continue;
        }
        let wearers: Vec<u8> = (1..=23)
            .filter(|nn| {
                roster
                    .get(*nn)
                    .is_some_and(|player| kind.worn_id(player) == id)
            })
            .collect();
        if wearers.is_empty() {
            refused(&mut *plan);
            continue;
        }
        for &nn in &wearers {
            let player = players.entry(nn).or_insert_with(PlayerFolder::new);
            // A wearer with no kept face folder still gets the bare
            // `ingame_face` marker.
            player.marker = player.marker || !player.has_face;
            match kind {
                ItemKind::Boots => {
                    player.boots_name_part = name_part.clone();
                    player.boots_folder = true;
                }
                ItemKind::Gloves => {
                    player.gloves_name_part = name_part.clone();
                    player.gloves_folder = true;
                }
            }
        }
        // On a Fox export a folder used by exactly one player merges into his
        // folder; pre-Fox never merges, and a path two folders share, folded
        // (`Skin.dds` and `skin.dds` are one on Windows), with different bytes
        // keeps the item folder shared instead (`merge_kept_shared`, the
        // first clashing path in its context).
        let player = &players[&wearers[0]];
        let share = engine == Engine::PreFox
            || wearers.len() > 1
            || folder_files
                .iter()
                .find_map(|file| {
                    let relative = &file[folder.len() + 1..];
                    let folded = vtree::fold_name(relative);
                    player
                        .files
                        .iter()
                        .find(|(path, _)| vtree::fold_name(path) == folded)
                        .and_then(|(_, existing)| {
                            (!same_bytes(existing, file)).then(|| relative.to_owned())
                        })
                })
                .is_some_and(|path| {
                    plan.findings.push(tool_message(
                        Code::MergeKeptShared,
                        Scope::Folder {
                            export_id: INPUT,
                            path: path_scope(&folder),
                        },
                        Disposition::Keep,
                        vec![("path", path)],
                    ));
                    true
                });
        plan.consumed
            .extend(folder_files.iter().map(|file| (*file).clone()));
        if share {
            shared.push(Shared {
                id,
                name_part,
                files: folder_files
                    .iter()
                    .map(|file| ((*file).clone(), file[folder.len() + 1..].to_owned()))
                    .collect(),
                wearers,
            });
        } else {
            let player = players
                .get_mut(&wearers[0])
                .expect("the wearer's folder was ensured");
            for file in &folder_files {
                let relative = &file[folder.len() + 1..];
                let folded = vtree::fold_name(relative);
                // A folded-equal path the merge already holds (byte-equal by
                // the clash check above) keeps the player folder's spelling.
                if player
                    .files
                    .keys()
                    .any(|path| vtree::fold_name(path) == folded)
                {
                    continue;
                }
                player.files.insert(relative.to_owned(), (*file).clone());
            }
        }
    }
    // Two shared folders of one kind whose names match, letter case aside,
    // compared each with the lowest-ID one of its name. One holding the same
    // files (the same paths folded, each byte for byte) is the same model
    // under another ID: it merges — its files are not copied and its wearers
    // link the lowest ID's folder name. One that differs takes its ID
    // spelling, the links following, so no wearer loads another's model.
    let mut by_name: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut names: Vec<String> = shared
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let name = clean_folder_name(&item.name_part)
                .unwrap_or_else(|| format!("{}{:04}", kind.letter(), item.id));
            by_name
                .entry(vtree::fold_name(&name))
                .or_default()
                .push(index);
            name
        })
        .collect();
    let mut merged = vec![false; shared.len()];
    for indexes in by_name.values_mut() {
        indexes.sort_by_key(|&index| shared[index].id);
        let first = indexes[0];
        for &index in &indexes[1..] {
            if same_files(&shared[first], &shared[index], same_bytes) {
                names[index] = names[first].clone();
                merged[index] = true;
            } else {
                names[index] = format!("{}{:04}", kind.letter(), shared[index].id);
            }
        }
    }
    for (index, (item, name)) in shared.iter().zip(&names).enumerate() {
        if !merged[index] {
            for (file, relative) in &item.files {
                plan.copies.push((
                    file.clone(),
                    format!("{}/{name}/{relative}", kind.shared_top()),
                ));
            }
        }
        for &nn in &item.wearers {
            players
                .get_mut(&nn)
                .expect("a wearer's folder was ensured")
                .links
                .insert(format!("{name}.{}", kind.link_ext()));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use studio_core::Severity;

    use super::*;
    use crate::roster::RosteredPlayer;

    /// A roster of (slot → name) pairs; boots and gloves are 0.
    fn roster(entries: &[(u8, &str)]) -> Roster {
        Roster(
            entries
                .iter()
                .map(|(slot, name)| {
                    (
                        *slot,
                        RosteredPlayer {
                            id: 71400 + u32::from(*slot),
                            name: (*name).to_owned(),
                            boots_id: 0,
                            gloves_id: 0,
                        },
                    )
                })
                .collect::<BTreeMap<u8, RosteredPlayer>>(),
        )
    }

    /// A roster of (slot → (name, boots_id, gloves_id)) triples.
    fn wearing(entries: &[(u8, (&str, u32, u32))]) -> Roster {
        Roster(
            entries
                .iter()
                .map(|(slot, (name, boots, gloves))| {
                    (
                        *slot,
                        RosteredPlayer {
                            id: 71400 + u32::from(*slot),
                            name: (*name).to_owned(),
                            boots_id: *boots,
                            gloves_id: *gloves,
                        },
                    )
                })
                .collect::<BTreeMap<u8, RosteredPlayer>>(),
        )
    }

    fn files(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|path| (*path).to_owned()).collect()
    }

    /// `player_folders` with `bytes` as the byte-comparison's answers: each
    /// input path to its bytes; a path not listed answers `false` (as a read
    /// failure does).
    fn plan_with_bytes(
        files: &[String],
        roster: Option<&Roster>,
        engine: Engine,
        bytes: &[(&str, &[u8])],
    ) -> PlayerFolders {
        let bytes_of = |path: &str| {
            bytes
                .iter()
                .find(|(listed, _)| *listed == path)
                .map(|(_, bytes)| *bytes)
        };
        player_folders(files, roster, engine, |a, b| {
            match (bytes_of(a), bytes_of(b)) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            }
        })
    }

    /// `player_folders` with no roster, Fox, and a byte check that answers
    /// false to every pair (nothing is ever byte-equal).
    fn plan_folders(files: &[String]) -> PlayerFolders {
        plan_with_bytes(files, None, Engine::Fox, &[])
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
            let plan = plan_with_bytes(
                &files(&[&format!("Faces/{name}/face.fmdl")]),
                None,
                Engine::Fox,
                &[],
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
        let plan = plan_folders(&files(&[
            "Faces/XXX15 - Alpha/face.fmdl",
            "Faces/XXX15 - Beta/face.fmdl",
        ]));
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
        let plan = plan_with_bytes(&list, Some(&roster(&[(9, "SaveName")])), Engine::Fox, &[]);
        assert_eq!(plan.copies[0].1, "Players/09 - FaceName/face.fmdl");
        // Without a name part the save's name is taken.
        let list = files(&["Faces/XXX09/face.fmdl"]);
        let plan = plan_with_bytes(&list, Some(&roster(&[(9, "SaveName")])), Engine::Fox, &[]);
        assert_eq!(plan.copies[0].1, "Players/09 - SaveName/face.fmdl");
        // With neither, the number alone names the folder.
        let plan = plan_folders(&list);
        assert_eq!(plan.copies[0].1, "Players/09/face.fmdl");
        // A name part that cleans to nothing is absent too.
        let list = files(&["Faces/XXX09 -   ./face.fmdl"]);
        let plan = plan_folders(&list);
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
            let plan = plan_folders(&files);
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
        let plan = plan_folders(&list);
        assert!(plan.copies.contains(&(
            "Faces/XXX15 - A/x_portrait.dds".to_owned(),
            "Players/15 - A/portrait.dds".to_owned()
        )));
        // Step 7's comparison map: the renamed portrait alone gives an
        // entry; two portraits as named give none.
        assert_eq!(
            plan.portraits.get(&15),
            Some(&"Faces/XXX15 - A/x_portrait.dds".to_owned())
        );
        let list = files(&[
            "Faces/XXX16 - A/portrait.dds",
            "Faces/XXX16 - A/alt_portrait.dds",
        ]);
        let plan = plan_folders(&list);
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
        assert!(!plan.portraits.contains_key(&16));
    }

    #[test]
    fn an_empty_roster_slot_is_a_warning_and_the_folder_still_forms() {
        let files = files(&["Faces/XXX23 - Pinto/face.fmdl"]);
        let plan = plan_with_bytes(&files, Some(&roster(&[])), Engine::Fox, &[]);
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
        let plan = plan_folders(&files(&[
            "Faces/XXX15 - A/face.fmdl",
            "Other/docs/readme.txt",
            "Boots/k0430 - Crocs/boots.fmdl",
        ]));
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
    fn the_item_folder_grammar() {
        // `k`/`g` lower case, four digits, name part trimmed; `g` and exactly
        // three digits alone reads as four with the `gNNNN` fallback name.
        for (kind, name, expected) in [
            (
                ItemKind::Boots,
                "k0430 - Crocs",
                Some((430, "Crocs".to_owned())),
            ),
            (ItemKind::Boots, "k0430", Some((430, String::new()))),
            (ItemKind::Gloves, "g567", Some((567, String::new()))),
            (
                ItemKind::Gloves,
                "g0432 Mitts",
                Some((432, "Mitts".to_owned())),
            ),
            (ItemKind::Gloves, "g567 - X", None),
            (ItemKind::Boots, "K0430", None),
            (ItemKind::Boots, "k430 - X", None),
            // Three digits alone read as four for gloves only.
            (ItemKind::Boots, "k430", None),
            (ItemKind::Boots, "boots", None),
        ] {
            assert_eq!(parse_item_name(kind, name), expected, "{name}");
        }
        // `g567`'s shared-name fallback is `g0567`, as the folder's ID spells.
        let files = files(&["Gloves/g567/glove_l.fmdl"]);
        let roster = wearing(&[(1, ("A", 0, 567)), (2, ("B", 0, 567))]);
        let plan = plan_with_bytes(&files, Some(&roster), Engine::Fox, &[]);
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Gloves/g0567/glove_l.fmdl")
        );
        assert!(
            plan.markers
                .contains(&"Players/01 - A/g0567.gloves".to_owned())
        );
        assert!(
            plan.markers
                .contains(&"Players/02 - B/g0567.gloves".to_owned())
        );
    }

    #[test]
    fn two_shared_folders_of_one_kind_sharing_a_name_resolve_by_lowest_id() {
        // `k0430 - Crocs` and `k0431 - crocs`, each worn by two: the lowest ID
        // keeps the name, the other takes its ID spelling.
        let files = files(&[
            "Boots/k0430 - Crocs/boots.fmdl",
            "Boots/k0431 - crocs/boots.fmdl",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
        ]);
        let plan = plan_with_bytes(&files, Some(&roster), Engine::Fox, &[]);
        assert!(
            plan.copies
                .iter()
                .any(|(from, to)| from == "Boots/k0430 - Crocs/boots.fmdl"
                    && to == "Boots/Crocs/boots.fmdl")
        );
        assert!(
            plan.copies
                .iter()
                .any(|(from, to)| from == "Boots/k0431 - crocs/boots.fmdl"
                    && to == "Boots/k0431/boots.fmdl")
        );
        // Their player folders take the worn folders' name parts (`Crocs`,
        // `crocs`), and each link follows its shared folder's name.
        for marker in [
            "Players/01 - Crocs/Crocs.boots",
            "Players/02 - Crocs/Crocs.boots",
            "Players/03 - crocs/k0431.boots",
            "Players/04 - crocs/k0431.boots",
        ] {
            assert!(plan.markers.contains(&marker.to_owned()), "{marker}");
        }
    }

    #[test]
    fn a_wearer_without_a_face_folder_is_named_by_the_item_then_roster() {
        // The worn boots folder's name part first, then the gloves folder's,
        // then the save's, then `NN` alone.
        let roster = wearing(&[(5, ("Save", 430, 432))]);
        for (paths, expected) in [
            (
                vec![
                    "Boots/k0430 - Crocs/boots.fmdl",
                    "Gloves/g0432 - Mitts/glove_l.fmdl",
                ],
                "Players/05 - Crocs/ingame_face",
            ),
            (
                vec![
                    "Boots/k0430/boots.fmdl",
                    "Gloves/g0432 - Mitts/glove_l.fmdl",
                ],
                "Players/05 - Mitts/ingame_face",
            ),
            (
                vec!["Boots/k0430/boots.fmdl", "Gloves/g0432/glove_l.fmdl"],
                "Players/05 - Save/ingame_face",
            ),
        ] {
            let plan = plan_with_bytes(&files(&paths), Some(&roster), Engine::Fox, &[]);
            assert!(plan.markers.contains(&expected.to_owned()), "{paths:?}");
        }
        // And `NN` alone when nothing names it.
        let plan = plan_with_bytes(
            &files(&["Boots/k0430/boots.fmdl"]),
            Some(&wearing(&[(5, ("", 430, 0))])),
            Engine::Fox,
            &[],
        );
        assert!(plan.markers.contains(&"Players/05/ingame_face".to_owned()));
    }

    #[test]
    fn a_gloves_folder_merges_into_a_player_folder_that_merged_boots() {
        // Boots are planned before gloves: both kinds land in the one
        // wearer's folder.
        let files = files(&[
            "Faces/XXX05 - A/face.fmdl",
            "Boots/k0430 - Crocs/boots.fmdl",
            "Gloves/g0432 - Mitts/glove_l.fmdl",
            "Gloves/g0432 - Mitts/glove_r.fmdl",
        ]);
        let roster = wearing(&[(5, ("A", 430, 432))]);
        let plan = plan_with_bytes(&files, Some(&roster), Engine::Fox, &[]);
        for rel in ["face.fmdl", "boots.fmdl", "glove_l.fmdl", "glove_r.fmdl"] {
            assert!(
                plan.copies
                    .iter()
                    .any(|(_, to)| *to == format!("Players/05 - A/{rel}")),
                "{rel}"
            );
        }
        assert!(
            !plan
                .copies
                .iter()
                .any(|(_, to)| to.contains("Crocs/") || to.contains("Mitts/"))
        );
    }

    #[test]
    fn the_pre_fox_engine_never_merges() {
        // Even one wearer stays shared on pre-Fox, with the link file.
        let files = files(&[
            "Faces/XXX05 - A/face.fmdl",
            "Boots/k0430 - Crocs/oral_boots_win32.model",
        ]);
        let roster = wearing(&[(5, ("A", 430, 0))]);
        let plan = plan_with_bytes(&files, Some(&roster), Engine::PreFox, &[]);
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Boots/Crocs/oral_boots_win32.model")
        );
        assert!(
            plan.markers
                .contains(&"Players/05 - A/Crocs.boots".to_owned())
        );
        assert!(
            !plan
                .copies
                .iter()
                .any(|(_, to)| to.starts_with("Players/05 - A/oral"))
        );
    }

    #[test]
    fn a_byte_clash_keeps_the_item_folder_shared() {
        // Equal bytes merge silently; different bytes keep the folder shared
        // with `merge_kept_shared` naming it and its first clashing path.
        let files = files(&[
            "Faces/XXX05 - A/skin.dds",
            "Boots/k0430 - Crocs/skin.dds",
            "Boots/k0430 - Crocs/boots.fmdl",
        ]);
        let roster = wearing(&[(5, ("A", 430, 0))]);
        let equal = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Faces/XXX05 - A/skin.dds", b"same"),
                ("Boots/k0430 - Crocs/skin.dds", b"same"),
            ],
        );
        assert!(
            equal
                .copies
                .iter()
                .any(|(_, to)| to == "Players/05 - A/boots.fmdl")
        );
        assert_eq!(
            equal
                .copies
                .iter()
                .filter(|(_, to)| to == "Players/05 - A/skin.dds")
                .count(),
            1,
            "the equal path is kept once"
        );
        let differing = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Faces/XXX05 - A/skin.dds", b"mine"),
                ("Boots/k0430 - Crocs/skin.dds", b"theirs"),
            ],
        );
        let kept = differing
            .findings
            .iter()
            .find(|m| m.code.code == "merge_kept_shared")
            .expect("merge_kept_shared expected");
        assert_eq!(kept.severity, Severity::Info);
        assert_eq!(
            kept.context,
            vec![("path".to_owned(), "skin.dds".to_owned())]
        );
        assert_eq!(
            kept.scope,
            Scope::Folder {
                export_id: INPUT,
                path: ScopePath::new("Boots/k0430 - Crocs").unwrap()
            }
        );
        assert!(
            differing
                .copies
                .iter()
                .any(|(_, to)| to == "Boots/Crocs/skin.dds")
        );
        assert!(
            differing
                .markers
                .contains(&"Players/05 - A/Crocs.boots".to_owned())
        );
    }

    #[test]
    fn a_folder_nobody_wears_and_a_second_folder_of_one_id_are_refused() {
        // EU-PLY-06's shape: no wearer → the folder is unmigrated as a whole;
        // a second folder of the same ID follows the first by name.
        let files = files(&[
            "Boots/k0430 - A/boots.fmdl",
            "Boots/k0430 - B/boots.fmdl",
            "Gloves/g0432 - Mitts/glove_l.fmdl",
        ]);
        let roster = wearing(&[(5, ("A", 430, 0))]);
        let plan = plan_with_bytes(&files, Some(&roster), Engine::Fox, &[]);
        let folders: Vec<String> = plan
            .findings
            .iter()
            .filter(|m| m.code.code == "unmigrated_content")
            .filter_map(|m| match &m.scope {
                Scope::Folder { path, .. } => Some(path.as_str().to_owned()),
                _ => None,
            })
            .collect();
        assert!(
            folders.contains(&"Boots/k0430 - B".to_owned()),
            "{folders:?}"
        );
        assert!(
            folders.contains(&"Gloves/g0432 - Mitts".to_owned()),
            "{folders:?}"
        );
        assert!(
            !plan
                .copies
                .iter()
                .any(|(_, to)| to.contains("g0432") || to.contains("Mitts/"))
        );
    }

    #[test]
    fn a_file_directly_in_faces_belongs_to_no_plan() {
        // A referee export's folders are `referees.rs`'s: `run` does not
        // call `player_folders` for one.
        let files = files(&["Faces/loose.fmdl", "Faces/XXX15 - A/face.fmdl"]);
        let plan = plan_folders(&files);
        assert!(!plan.consumed.contains("Faces/loose.fmdl"));
    }

    #[test]
    fn a_case_folded_clash_keeps_the_item_folder_shared() {
        // `Skin.dds` in the face folder and `skin.dds` in the boots folder:
        // paths compare folded, so of different bytes they clash.
        let files = files(&[
            "Faces/XXX05 - A/Skin.dds",
            "Boots/k0430 - Crocs/skin.dds",
            "Boots/k0430 - Crocs/boots.fmdl",
        ]);
        let roster = wearing(&[(5, ("A", 430, 0))]);
        let differing = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Faces/XXX05 - A/Skin.dds", b"face"),
                ("Boots/k0430 - Crocs/skin.dds", b"boots"),
            ],
        );
        let kept = differing
            .findings
            .iter()
            .find(|m| m.code.code == "merge_kept_shared")
            .expect("merge_kept_shared expected");
        // The item folder's spelling of the clashing path.
        assert_eq!(
            kept.context,
            vec![("path".to_owned(), "skin.dds".to_owned())]
        );
        assert!(
            differing
                .copies
                .iter()
                .any(|(_, to)| to == "Boots/Crocs/skin.dds")
        );
        // Equal bytes merge once, the player folder's own spelling kept.
        let equal = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Faces/XXX05 - A/Skin.dds", b"same"),
                ("Boots/k0430 - Crocs/skin.dds", b"same"),
            ],
        );
        assert!(
            equal
                .copies
                .iter()
                .any(|(_, to)| to == "Players/05 - A/Skin.dds")
        );
        assert!(
            equal
                .copies
                .iter()
                .any(|(_, to)| to == "Players/05 - A/boots.fmdl")
        );
        assert!(
            !equal
                .copies
                .iter()
                .any(|(_, to)| to == "Players/05 - A/skin.dds")
        );
    }

    #[test]
    fn two_shared_folders_matching_only_by_fold_resolve_by_lowest_id() {
        // `Éclair` and `éclair` fold to one name where `to_ascii_lowercase`
        // kept them apart.
        let files = files(&[
            "Boots/k0430 - Éclair/boots.fmdl",
            "Boots/k0431 - éclair/boots.fmdl",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
        ]);
        let plan = plan_with_bytes(&files, Some(&roster), Engine::Fox, &[]);
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Boots/Éclair/boots.fmdl")
        );
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Boots/k0431/boots.fmdl")
        );
    }

    #[test]
    fn identical_shared_folders_merge_into_the_lowest_ids() {
        // `k0430 - Crocs` and `k0431 - Crocs`, the same files byte for byte:
        // one `Boots/Crocs/`, every wearer of either linking it.
        let files = files(&[
            "Boots/k0430 - Crocs/boots.fmdl",
            "Boots/k0430 - Crocs/boots.skl",
            "Boots/k0431 - Crocs/boots.fmdl",
            "Boots/k0431 - Crocs/boots.skl",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
        ]);
        let plan = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Boots/k0430 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0431 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0430 - Crocs/boots.skl", b"skel"),
                ("Boots/k0431 - Crocs/boots.skl", b"skel"),
            ],
        );
        // Two files, from `k0430 - Crocs` alone.
        assert_eq!(
            plan.copies,
            [
                (
                    "Boots/k0430 - Crocs/boots.fmdl".to_owned(),
                    "Boots/Crocs/boots.fmdl".to_owned()
                ),
                (
                    "Boots/k0430 - Crocs/boots.skl".to_owned(),
                    "Boots/Crocs/boots.skl".to_owned()
                ),
            ]
        );
        for player in ["01 - Crocs", "02 - Crocs", "03 - Crocs", "04 - Crocs"] {
            assert!(
                plan.markers
                    .contains(&format!("Players/{player}/Crocs.boots")),
                "{player}"
            );
        }
        // Its files are not copied but stay consumed — nothing is lost and
        // nothing is reported.
        for file in ["boots.fmdl", "boots.skl"] {
            assert!(
                plan.consumed
                    .contains(&format!("Boots/k0431 - Crocs/{file}")),
                "{file}"
            );
        }
        assert!(
            plan.findings
                .iter()
                .all(|m| m.code.code.as_ref() != "unmigrated_content")
        );
    }

    #[test]
    fn a_difference_in_one_file_keeps_the_id_spelling() {
        // `boots.skl` differing in one byte: the 6.2b rule, `Boots/k0431/`.
        let files = files(&[
            "Boots/k0430 - Crocs/boots.fmdl",
            "Boots/k0430 - Crocs/boots.skl",
            "Boots/k0431 - Crocs/boots.fmdl",
            "Boots/k0431 - Crocs/boots.skl",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
        ]);
        let plan = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Boots/k0430 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0431 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0430 - Crocs/boots.skl", b"one"),
                ("Boots/k0431 - Crocs/boots.skl", b"two"),
            ],
        );
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Boots/k0431/boots.skl")
        );
        assert!(
            plan.markers
                .contains(&"Players/03 - Crocs/k0431.boots".to_owned())
        );
        assert!(
            plan.markers
                .contains(&"Players/04 - Crocs/k0431.boots".to_owned())
        );
    }

    #[test]
    fn an_extra_file_means_no_merge() {
        // `k0431 - Crocs` holding one more file than `k0430 - Crocs`.
        let files = files(&[
            "Boots/k0430 - Crocs/boots.fmdl",
            "Boots/k0430 - Crocs/boots.skl",
            "Boots/k0431 - Crocs/boots.fmdl",
            "Boots/k0431 - Crocs/boots.skl",
            "Boots/k0431 - Crocs/extra.dds",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
        ]);
        let plan = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Boots/k0430 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0431 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0430 - Crocs/boots.skl", b"skel"),
                ("Boots/k0431 - Crocs/boots.skl", b"skel"),
                ("Boots/k0431 - Crocs/extra.dds", b"extra"),
            ],
        );
        assert!(
            plan.copies
                .iter()
                .any(|(_, to)| to == "Boots/k0431/extra.dds")
        );
        assert!(
            plan.markers
                .contains(&"Players/03 - Crocs/k0431.boots".to_owned())
        );
    }

    #[test]
    fn a_case_folded_file_set_merges_too() {
        // `Boots.fmdl` in `k0431 - crocs` is `boots.fmdl` in `k0430 - Crocs`,
        // paths folded: equal bytes merge them.
        let files = files(&[
            "Boots/k0430 - Crocs/boots.fmdl",
            "Boots/k0431 - crocs/Boots.fmdl",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
        ]);
        let plan = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Boots/k0430 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0431 - crocs/Boots.fmdl", b"model"),
            ],
        );
        assert_eq!(
            plan.copies,
            [(
                "Boots/k0430 - Crocs/boots.fmdl".to_owned(),
                "Boots/Crocs/boots.fmdl".to_owned()
            )]
        );
        assert!(
            plan.markers
                .contains(&"Players/03 - crocs/Crocs.boots".to_owned())
        );
    }

    #[test]
    fn among_three_folders_only_the_identical_pair_merges() {
        // `k0431 - Crocs` is `k0430 - Crocs` byte for byte, `k0432 - Crocs`
        // is not: the middle merges, the third keeps its ID spelling.
        let files = files(&[
            "Boots/k0430 - Crocs/boots.fmdl",
            "Boots/k0431 - Crocs/boots.fmdl",
            "Boots/k0432 - Crocs/boots.fmdl",
        ]);
        let roster = wearing(&[
            (1, ("A", 430, 0)),
            (2, ("B", 430, 0)),
            (3, ("C", 431, 0)),
            (4, ("D", 431, 0)),
            (5, ("E", 432, 0)),
            (6, ("F", 432, 0)),
        ]);
        let plan = plan_with_bytes(
            &files,
            Some(&roster),
            Engine::Fox,
            &[
                ("Boots/k0430 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0431 - Crocs/boots.fmdl", b"model"),
                ("Boots/k0432 - Crocs/boots.fmdl", b"other"),
            ],
        );
        assert_eq!(
            plan.copies,
            [
                (
                    "Boots/k0430 - Crocs/boots.fmdl".to_owned(),
                    "Boots/Crocs/boots.fmdl".to_owned()
                ),
                (
                    "Boots/k0432 - Crocs/boots.fmdl".to_owned(),
                    "Boots/k0432/boots.fmdl".to_owned()
                ),
            ]
        );
        for marker in [
            "Players/01 - Crocs/Crocs.boots",
            "Players/03 - Crocs/Crocs.boots",
            "Players/05 - Crocs/k0432.boots",
            "Players/06 - Crocs/k0432.boots",
        ] {
            assert!(plan.markers.contains(&marker.to_owned()), "{marker}");
        }
    }
}
