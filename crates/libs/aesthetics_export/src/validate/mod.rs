//! The structure pass's back half: `ParsedAestheticsExport` → a report of
//! every issue plus the sanitized export when nothing drops it. `validated`
//! holds exactly the eligible content; `parsed` keeps everything for
//! rendering and repair.

mod folders;
mod issues;
mod kits;
mod links;
mod root;
mod roster;

use std::collections::BTreeMap;

use crate::FileKind;
use crate::conventions::{SharedKind, is_logo_texture};
use crate::listing::ValidationContext;
use crate::parse::{ExportKind, FileDescriptor, ParsedAestheticsExport};
use crate::slots::PlayerSlot;

pub use folders::{
    FpcDirective, KitFolder, KitLayout, KitTexture, KitTextureSource, KitsFolder, PlayerFolder,
    SharedLink, SharedModelFolder,
};
pub use issues::{Disposition, ISSUE_CODES, IssueScope, ValidationIssue};
pub(crate) use issues::{dropped_scopes, issue, issue_in, strict_disposition};
pub use roster::{PlayerIndex, ValidatedRoster};

/// `ParsedAestheticsExport::validate`'s report: the parse retained, the
/// sanitized export when no issue drops it, and every issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    /// The parse output, its own issues included: a dropped folder stays
    /// renderable and a broken roster repairable.
    pub parsed: ParsedAestheticsExport,
    /// The sanitized export; `None` exactly when some issue's effective
    /// disposition is `DropExport`.
    pub validated: Option<ValidatedAestheticsExport>,
    /// Every issue: `parsed.issues`, then validation's.
    pub issues: Vec<ValidationIssue>,
}

/// The root `logo*` pair, each with its fit mode ("Root files").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogoFiles {
    /// The main logo.
    pub main: LogoFile,
    /// The small logo, when present.
    pub small: Option<LogoFile>,
}

/// One logo file with its fit tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogoFile {
    /// The image's descriptor.
    pub file: FileDescriptor,
    /// The stem's tag; `None`: untagged (`fit` if not square).
    pub fit: Option<LogoFit>,
}

/// How a non-square logo becomes square.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LogoFit {
    /// `_crop`.
    Crop,
    /// `_stretch`.
    Stretch,
    /// `_fit`.
    Fit,
}

/// Sanitized root colors/notes/referee marker. Invalid optional root
/// artifacts are absent here but remain in `ValidationReport`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootArtifacts {
    /// The team's `colors.txt`.
    pub team_colors: Option<FileDescriptor>,
    /// The read `notes.txt`.
    pub notes: Option<FileDescriptor>,
    /// A referee export's `ref_marker.dds`.
    pub referee_marker: Option<FileDescriptor>,
}

/// The sanitized export: exactly the content no issue dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedAestheticsExport {
    /// Complete archive/folder stem; presentation/source identity.
    pub export_display_name: String,
    /// The canonical `/xx/` name derived from the first word.
    pub team_name: teams_list::TeamName,
    /// Sanitized main reference point for each eligible player.
    pub players: Vec<PlayerFolder>,
    /// Normalized strong slots → player indices.
    pub roster: ValidatedRoster,
    /// Shared folders, referenced by name from player folders (no IDs — IDs
    /// are assigned by run planning).
    pub faces: Vec<SharedModelFolder>,
    /// Shared boots folders.
    pub boots: Vec<SharedModelFolder>,
    /// Shared gloves folders.
    pub gloves: Vec<SharedModelFolder>,
    /// Per-kit subfolders (config.toml + colors.txt + textures).
    pub kits: KitsFolder,
    /// `Portraits/player_NN.*`.
    pub portraits: BTreeMap<PlayerSlot, FileDescriptor>,
    /// Root `logo*` (+ optional `logo_small*`), each with its fit mode.
    pub logo: Option<LogoFiles>,
    /// `Collars/`, passed through.
    pub collars: Vec<FileDescriptor>,
    /// `Common/`, the targets of `.common` links.
    pub common: Vec<FileDescriptor>,
    /// Sanitized root colors/notes/referee marker.
    pub root: RootArtifacts,
}

impl ParsedAestheticsExport {
    /// Infallible: every structural failure is a `ValidationIssue`, a
    /// foundational one included (`DropExport`, `validated: None`); what
    /// cannot be parsed at all is `parse_listing`'s `SourceError`. The
    /// context is unused by this slice's checks.
    pub fn validate(self, context: &ValidationContext) -> ValidationReport {
        let mut issues = self.issues.clone();
        let draft = &self.draft;

        // Export findings.
        if draft.team_name.is_none() {
            issues.push(issue(
                "team_name_unknown",
                IssueScope::Export,
                vec![("team_name", String::new())],
                Disposition::DropExport,
            ));
        }
        // `export_empty` is not reported beside an ambiguous or conflicting
        // nested root, which leaves the root undecided.
        let root_decided = !self
            .issues
            .iter()
            .any(|issue| matches!(issue.code, "nested_root_ambiguous" | "nested_root_conflict"));
        let has_content = !(draft.players.is_empty()
            && draft.faces.is_empty()
            && draft.boots.is_empty()
            && draft.gloves.is_empty()
            && draft.kits.is_empty()
            && draft.portraits.is_empty()
            && draft.collars.is_empty()
            && draft.common.is_empty());
        let has_logo = draft
            .root_files
            .iter()
            .any(|file| is_logo_texture(file.path.name()));
        if root_decided && !has_content && !has_logo {
            issues.push(issue(
                "export_empty",
                IssueScope::Export,
                vec![],
                Disposition::DropExport,
            ));
        }

        // The roster: authoritative lines, or the folder names' numbers.
        let slot_map = roster::check(draft, self.raw_roster.as_ref(), &self.issues, &mut issues);

        // Own findings, in drop order: player folders, shared folders,
        // `Common/`, then the cascade (dropped targets, orphaned shares).
        for folder in &draft.players {
            folders::check_player(draft, folder, context, &mut issues);
        }
        for kind in [SharedKind::Face, SharedKind::Boots, SharedKind::Gloves] {
            for folder in folders::shared_folders(draft, kind) {
                folders::check_shared(folder, kind, context, &mut issues);
            }
        }
        folders::check_common(draft, context, &mut issues);
        links::cascade(draft, &slot_map, context, &mut issues);

        // Kits, portraits, logo and the root files: same order as the draft's
        // content folders end (`Kits/`, then the loose root groups).
        let kits = kits::check(draft, context, &self.metadata, &mut issues);
        let portraits = root::check_portraits(draft, context, &mut issues);
        let logo = root::check_logo(draft, context, &mut issues);
        // No root findings on an undecided root: `nested_root_ambiguous` and
        // `nested_root_conflict` leave it unresolved, as for `export_empty`.
        let root = if root_decided {
            root::check_root(
                draft,
                &self.metadata,
                draft.kind() == ExportKind::Referees,
                context,
                &mut issues,
            )
        } else {
            RootArtifacts {
                team_colors: None,
                notes: None,
                referee_marker: None,
            }
        };

        // Sanitized `players`: the draft folders a surviving assignment maps
        // and no `DropFolder` issue drops, in draft order.
        let (dropped_folders, dropped_files) = dropped_scopes(&issues);
        let is_dropped =
            |index: usize| dropped_folders.contains(&draft.players[index].path.fold_key());
        let mapped = slot_map.mapped();
        let mut kept = Vec::new();
        let mut index_of = BTreeMap::new();
        for (index, folder) in draft.players.iter().enumerate() {
            if mapped.contains(&index) && !is_dropped(index) {
                index_of.insert(index, kept.len());
                let mut player = folders::player_folder(folder, self.raw_roster.is_some());
                // A kept player loses references to what is absent: a missing
                // shared target (pass_through) comes off `links`, a missing
                // `Common` target off `files`. A resolved link carries the
                // target folder's own spelling.
                let resolved = links::player_links(folder, draft);
                player.links = resolved
                    .iter()
                    .filter_map(|link| match (&link.kind, &link.target) {
                        (links::ResolvedLinkKind::Shared(kind), Some(target)) => Some(SharedLink {
                            kind: *kind,
                            name: target.name().to_owned(),
                        }),
                        _ => None,
                    })
                    .collect();
                player.files.retain(|file| {
                    file.kind != FileKind::CommonLink
                        || resolved.iter().any(|link| {
                            matches!(&link.kind, links::ResolvedLinkKind::Common(_))
                                && link.target.is_some()
                                && link.link_file == file.path
                        })
                });
                kept.push(player);
            }
        }

        let validated = if issues
            .iter()
            .any(|issue| issue.disposition == Disposition::DropExport)
        {
            None
        } else {
            let roster = match slot_map {
                roster::SlotMap::Team(assignments) => {
                    ValidatedRoster::Team(renumber(&assignments, &index_of))
                }
                roster::SlotMap::Referees(assignments) => {
                    ValidatedRoster::Referees(renumber(&assignments, &index_of))
                }
            };
            let kept_folders = |drafts: &[crate::parse::FolderDraft]| {
                drafts
                    .iter()
                    .filter(|folder| !dropped_folders.contains(&folder.path.fold_key()))
                    .map(folders::shared_model_folder)
                    .collect()
            };
            Some(ValidatedAestheticsExport {
                export_display_name: draft.export_display_name.clone(),
                team_name: draft
                    .team_name
                    .clone()
                    .expect("a surviving export has a team name"),
                players: kept,
                roster,
                faces: kept_folders(&draft.faces),
                boots: kept_folders(&draft.boots),
                gloves: kept_folders(&draft.gloves),
                kits,
                portraits,
                logo,
                collars: draft.collars.clone(),
                common: draft
                    .common
                    .iter()
                    .filter(|file| !dropped_files.contains(&file.path.fold_key()))
                    .cloned()
                    .collect(),
                root,
            })
        };

        ValidationReport {
            parsed: self,
            validated,
            issues,
        }
    }
}

/// Slot → draft player index becomes slot → `PlayerIndex` in `players`.
fn renumber<Slot: Copy + Ord>(
    assignments: &BTreeMap<Slot, usize>,
    index_of: &BTreeMap<usize, usize>,
) -> BTreeMap<Slot, PlayerIndex> {
    assignments
        .iter()
        .filter_map(|(slot, index)| {
            index_of
                .get(index)
                .map(|index| (*slot, PlayerIndex(*index)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{context_with, report, report_with};
    use crate::validate::IssueScope;
    use crate::{FileKind, ModelFormat, SharedKind};
    use vtree::ScopePath;

    fn issue_codes(report: &ValidationReport) -> Vec<(&'static str, Disposition)> {
        report
            .issues
            .iter()
            .map(|issue| (issue.code, issue.disposition))
            .collect()
    }

    // TC-STR-04
    #[test]
    fn an_export_with_no_usable_content_is_dropped_empty() {
        let report = report("egg", &[("readme.txt", 10)], &[], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![("export_empty", Disposition::DropExport)]
        );
        assert_eq!(report.issues[0].scope, IssueScope::Export);
        assert!(report.validated.is_none());
    }

    #[test]
    fn an_ambiguous_or_conflicting_root_reports_no_export_empty() {
        for files in [
            &[("a/Players/x/face_high.fmdl", 10), ("b/Kits/p1/kit.dds", 9)][..],
            &[
                ("notes.txt", 5),
                ("wrapper/notes.txt", 5),
                ("wrapper/Players/03 - A/face_high.fmdl", 10),
            ][..],
        ] {
            let report = report("egg", files, &[], &[]);
            assert!(
                report
                    .issues
                    .iter()
                    .all(|issue| issue.code != "export_empty"),
                "{files:?}"
            );
        }
    }

    // TC-ID-04
    #[test]
    fn a_stem_with_no_team_token_is_dropped_unknown() {
        let report = report("--- ", &[("Players/03 - A/face_high.fmdl", 10)], &[], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![("team_name_unknown", Disposition::DropExport)]
        );
        assert_eq!(report.issues[0].scope, IssueScope::Export);
        assert_eq!(report.issues[0].context, vec![("team_name", String::new())]);
        assert!(report.validated.is_none());
    }

    #[test]
    fn a_player_folder_builds_with_its_links_and_markers() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/gloves/glove_l.fmdl", 10),
                ("Players/03 - A/common/skin.dds", 9),
                ("Players/03 - A/Crocs.boots.txt", 0),
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/fpc.on", 0),
                ("Players/03 - A/portrait.dds", 9),
                ("Players/03 - A/settings.toml", 20),
                ("Boots/Crocs/boots.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let folder = &report.validated.unwrap().players[0];
        assert_eq!(folder.player_name, "A");
        assert_eq!(
            folder.links,
            vec![SharedLink {
                kind: SharedKind::Boots,
                name: "Crocs".to_owned(),
            }]
        );
        assert!(folder.ingame_face);
        assert_eq!(folder.fpc, Some(FpcDirective::On));
        assert_eq!(
            folder.portrait.as_ref().map(|f| f.path.as_str()),
            Some("Players/03 - A/portrait.dds")
        );
        assert_eq!(
            folder.settings.as_ref().map(|f| f.path.as_str()),
            Some("Players/03 - A/settings.toml")
        );
        // Pipeline content: loose files and reserved-subfolder files alike.
        assert_eq!(
            folder
                .files
                .iter()
                .map(|f| f.path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Players/03 - A/common/skin.dds",
                "Players/03 - A/gloves/glove_l.fmdl",
                "Players/03 - A/hair.dds",
            ]
        );
        assert_eq!(folder.files[1].kind, FileKind::Model(ModelFormat::Fmdl));
    }

    #[test]
    fn only_the_portrait_stem_is_the_portrait() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/portrait.dds", 9),
            ],
            &[],
            &[],
        );
        let folder = &report.validated.unwrap().players[0];
        assert_eq!(
            folder.portrait.as_ref().map(|file| file.path.as_str()),
            Some("Players/03 - A/portrait.dds")
        );
        assert_eq!(
            folder
                .files
                .iter()
                .map(|file| file.path.as_str())
                .collect::<Vec<_>>(),
            vec!["Players/03 - A/hair.dds"]
        );
    }

    // TC-STR-13
    #[test]
    fn two_textures_sharing_a_stem_drop_the_folder() {
        for files in [
            &[
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/hair.png", 9),
            ][..],
            &[
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/common/hair.dds", 9),
            ][..],
        ] {
            let report = report("egg", files, &[], &[]);
            let issues: Vec<(&'static str, Disposition)> = report
                .issues
                .iter()
                .map(|issue| (issue.code, issue.disposition))
                .collect();
            assert_eq!(
                issues,
                vec![("texture_stem_conflict", Disposition::DropFolder)]
            );
            assert_eq!(
                report.issues[0].scope,
                IssueScope::Folder(ScopePath::new("Players/03 - A").unwrap())
            );
            assert!(report.validated.unwrap().players.is_empty(), "{files:?}");
        }
    }

    #[test]
    fn fpc_off_alone_is_an_off_directive() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/face_high.fmdl", 10),
                ("Players/03 - A/fpc.off", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            report.validated.unwrap().players[0].fpc,
            Some(FpcDirective::Off)
        );
    }

    // TC-STR-12
    #[test]
    fn both_fpc_markers_drop_the_folder() {
        // A roster-mapped player dropped by its own finding.
        let report = report(
            "egg",
            &[
                ("players.txt", 5),
                ("Players/A/hair.dds", 9),
                ("Players/A/fpc.on", 0),
                ("Players/A/fpc.off", 0),
            ],
            &[],
            &[("players.txt", Ok(b"03 A"))],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("fpc_conflict", Disposition::DropFolder)]
        );
        let validated = report.validated.unwrap();
        assert!(validated.players.is_empty());
        let ValidatedRoster::Team(map) = &validated.roster else {
            panic!("a team export");
        };
        assert!(map.is_empty());
    }

    #[test]
    fn shared_folders_and_common_files_build_unchecked() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/Longhair.face", 0),
                ("Players/03 - A/Crocs.boots", 0),
                ("Players/03 - A/Keeper gloves.gloves", 0),
                ("Players/03 - A/hair.dds", 9),
                ("Faces/Longhair/face_high.fmdl", 10),
                ("Boots/Crocs/boots.fmdl", 10),
                ("Gloves/Keeper gloves/glove_l.fmdl", 10),
                ("Common/hair.dds", 9),
                ("Collars/collar_101.dds", 9),
            ],
            &[],
            &[],
        );
        let validated = report.validated.unwrap();
        assert_eq!(validated.faces[0].folder_name, "Longhair");
        assert_eq!(
            validated.faces[0].files[0].path.as_str(),
            "Faces/Longhair/face_high.fmdl"
        );
        assert_eq!(validated.boots[0].folder_name, "Crocs");
        assert_eq!(validated.gloves[0].folder_name, "Keeper gloves");
        assert_eq!(validated.common[0].path.as_str(), "Common/hair.dds");
        assert_eq!(validated.common[0].kind, FileKind::Texture);
        assert_eq!(validated.collars[0].path.as_str(), "Collars/collar_101.dds");
    }

    fn folder(path: &str) -> IssueScope {
        IssueScope::Folder(ScopePath::new(path).unwrap())
    }

    // TC-STR-05
    #[test]
    fn a_shared_link_resolves_with_or_without_the_txt_tail() {
        for link_file in ["Crocs.boots", "Crocs.boots.txt"] {
            let path = format!("Players/03 - A/{link_file}");
            let report = report(
                "egg",
                &[
                    ("Players/03 - A/hair.dds", 9),
                    (path.as_str(), 0),
                    ("Boots/Crocs/boots.fmdl", 10),
                ],
                &[],
                &[],
            );
            assert_eq!(issue_codes(&report), vec![], "{link_file}");
            let player = &report.validated.unwrap().players[0];
            assert_eq!(
                player.links,
                vec![SharedLink {
                    kind: SharedKind::Boots,
                    name: "Crocs".to_owned(),
                }],
                "{link_file}"
            );
        }
    }

    // TC-STR-06
    #[test]
    fn a_link_to_a_missing_shared_folder_drops_the_player() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/Nowhere.boots", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("link_target_missing", Disposition::DropFolder)]
        );
        assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
        assert_eq!(
            report.issues[0].context,
            vec![("link", "Nowhere.boots".to_owned())]
        );
        assert!(report.validated.unwrap().players.is_empty());
    }

    // TC-STR-07
    #[test]
    fn two_links_of_one_kind_drop_the_player() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/Crocs.boots", 0),
                ("Players/03 - A/Mud.boots", 0),
            ],
            &[],
            &[],
        );
        // Every finding is reported: the duplicate, and each missing target.
        assert_eq!(
            issue_codes(&report),
            vec![
                ("shared_link_duplicate", Disposition::DropFolder),
                ("link_target_missing", Disposition::DropFolder),
                ("link_target_missing", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].context, vec![("kind", "boots".to_owned())]);
        assert!(report.validated.unwrap().players.is_empty());
    }

    // TC-STR-08
    #[test]
    fn a_shared_folder_no_surviving_player_links_is_orphaned() {
        let report = report(
            "egg",
            &[
                ("players.txt", 5),
                ("Boots/Solo/boots.fmdl", 10),
                ("Boots/Duo/boots.fmdl", 10),
                ("Players/Droppable/Duo.boots", 0),
            ],
            &["Players/A", "Players/Droppable"],
            &[("players.txt", Ok(b"03 A"))],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("player_unlisted", Disposition::DropFolder),
                ("shared_folder_orphaned", Disposition::DropFolder),
                ("shared_folder_orphaned", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Players/Droppable"));
        assert_eq!(report.issues[1].scope, folder("Boots/Duo"));
        assert_eq!(report.issues[2].scope, folder("Boots/Solo"));
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        assert!(validated.boots.is_empty());
    }

    #[test]
    fn a_disallowed_file_drops_strict_and_keeps_lenient() {
        let files = &[
            ("Players/03 - A/hair.dds", 9u64),
            ("Players/03 - A/readme.txt", 20),
        ];
        let strict = report("egg", files, &[], &[]);
        assert_eq!(
            issue_codes(&strict),
            vec![("file_type_disallowed", Disposition::DropFolder)]
        );
        assert!(strict.validated.unwrap().players.is_empty());

        let lenient = report_with(&context_with(false, false), "egg", files, &[], &[]);
        assert_eq!(
            issue_codes(&lenient),
            vec![("file_type_disallowed", Disposition::Keep)]
        );
        let player = &lenient.validated.unwrap().players[0];
        assert_eq!(
            player
                .files
                .iter()
                .map(|file| file.path.name().to_owned())
                .collect::<Vec<_>>(),
            vec!["hair.dds", "readme.txt"]
        );
    }

    // TC-STR-10
    #[test]
    fn both_ingame_face_spellings_mark_the_folder() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/hair.dds", 9),
                ("Players/04 - B/ingame_face.txt", 0),
                ("Players/04 - B/hair.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let validated = report.validated.unwrap();
        assert!(validated.players[0].ingame_face);
        assert!(validated.players[1].ingame_face);
    }

    // TC-STR-11
    #[test]
    fn ingame_face_with_explicit_face_content_drops_the_folder() {
        let local = report(
            "egg",
            &[
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/face_high.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&local),
            vec![("ingame_face_explicit_face_model", Disposition::DropFolder)]
        );
        assert!(local.validated.unwrap().players.is_empty());

        let linked = report(
            "egg",
            &[
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/Long.face", 0),
                ("Faces/Long/face_high.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&linked),
            vec![
                ("ingame_face_explicit_face_model", Disposition::DropFolder),
                ("shared_folder_orphaned", Disposition::DropFolder),
            ]
        );
        let validated = linked.validated.unwrap();
        assert!(validated.players.is_empty());
        assert!(validated.faces.is_empty());
    }

    // TC-STR-14
    #[test]
    fn ingame_face_acts_by_subfolder_category() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/boots/hair_high.fmdl", 10),
                ("Players/03 - A/gloves/glove_l.fmdl", 10),
                ("Players/03 - A/common/skin.dds", 9),
                ("Players/04 - B/ingame_face", 0),
                ("Players/04 - B/face/hair_high.fmdl", 10),
                ("Players/05 - C/ingame_face", 0),
                ("Players/05 - C/common/x.fmdl", 10),
                ("Players/05 - C/extra/x.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("ingame_face_explicit_face_model", Disposition::DropFolder),
                ("file_type_disallowed", Disposition::DropFolder),
                ("file_type_disallowed", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Players/04 - B"));
        assert_eq!(report.issues[1].scope, folder("Players/05 - C"));
        assert_eq!(
            report.issues[1].context,
            vec![("file", "common/x.fmdl".to_owned())]
        );
        assert_eq!(report.issues[2].scope, folder("Players/05 - C"));
        assert_eq!(
            report.issues[2].context,
            vec![("file", "extra/x.dds".to_owned())]
        );
        // The boots/gloves/common files are those categories' parts: no
        // finding for A.
        let validated = report.validated.unwrap();
        assert_eq!(
            validated
                .players
                .iter()
                .map(|player| player.path.as_str())
                .collect::<Vec<_>>(),
            vec!["Players/03 - A"]
        );
    }

    // TC-STR-15
    #[test]
    fn a_common_link_resolves_or_drops_the_player() {
        let report = report(
            "egg",
            &[
                ("Common/torso.fmdl", 10),
                ("Players/01 - A/torso.fmdl.common", 0),
                ("Players/02 - B/torso.fmdl.common.txt", 0),
                ("Players/03 - C/missing.fmdl.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("common_link_missing", Disposition::DropFolder)]
        );
        assert_eq!(report.issues[0].scope, folder("Players/03 - C"));
        assert_eq!(
            report.issues[0].context,
            vec![
                ("link", "missing.fmdl.common".to_owned()),
                ("path", "Common/missing.fmdl".to_owned()),
            ]
        );
        let validated = report.validated.unwrap();
        assert_eq!(
            validated
                .players
                .iter()
                .map(|player| player.path.as_str())
                .collect::<Vec<_>>(),
            vec!["Players/01 - A", "Players/02 - B"]
        );
        // Resolved `.common` links stay as link-kind files.
        assert_eq!(validated.players[0].files[0].kind, FileKind::CommonLink);
    }

    // TC-STR-16
    #[test]
    fn a_dropped_link_target_drops_its_players_and_pass_through_keeps() {
        let files = &[
            ("Faces/Base/x.exe", 4u64),
            ("Players/03 - A/Base.face", 0),
            ("Players/03 - A/hair.dds", 9),
        ];
        let strict = report("egg", files, &[], &[]);
        assert_eq!(
            issue_codes(&strict),
            vec![
                ("file_type_disallowed", Disposition::DropFolder),
                ("link_target_dropped", Disposition::DropFolder),
            ]
        );
        assert_eq!(strict.issues[0].scope, folder("Faces/Base"));
        assert_eq!(strict.issues[1].scope, folder("Players/03 - A"));
        assert_eq!(
            strict.issues[1].context,
            vec![
                ("link", "Base.face".to_owned()),
                ("target", "Faces/Base".to_owned()),
                ("finding", "file_type_disallowed".to_owned()),
            ]
        );
        let validated = strict.validated.unwrap();
        assert!(validated.players.is_empty());
        assert!(validated.faces.is_empty());

        // With pass_through the target stays, no link_target_dropped.
        let kept = report_with(&context_with(true, true), "egg", files, &[], &[]);
        assert_eq!(
            issue_codes(&kept),
            vec![("file_type_disallowed", Disposition::Keep)]
        );
        assert!(kept.issues[0].passed_through);
        let validated = kept.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        assert_eq!(validated.faces.len(), 1);
        assert_eq!(
            validated.players[0].links,
            vec![SharedLink {
                kind: SharedKind::Face,
                name: "Base".to_owned(),
            }]
        );

        // A conflict is never pass-through-eligible: both drop.
        let conflicted = report_with(
            &context_with(true, true),
            "egg",
            &[
                ("Faces/Base/hair.dds", 9),
                ("Faces/Base/hair.png", 9),
                ("Players/03 - A/Base.face", 0),
                ("Players/03 - A/hair.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&conflicted),
            vec![
                ("texture_stem_conflict", Disposition::DropFolder),
                ("link_target_dropped", Disposition::DropFolder),
            ]
        );
        assert!(conflicted.issues.iter().all(|issue| !issue.passed_through));
        let validated = conflicted.validated.unwrap();
        assert!(validated.players.is_empty());
        assert!(validated.faces.is_empty());
    }

    // TC-STR-17
    #[test]
    fn a_wrongly_named_boots_model_drops_folder_and_linker() {
        let report = report(
            "egg",
            &[
                ("Boots/Crocs/torso.fmdl", 10),
                ("Boots/Mud/kit_boots.fmdl", 10),
                ("Players/01 - A/Crocs.boots", 0),
                ("Players/01 - A/hair.dds", 9),
                ("Players/02 - B/Mud.boots", 0),
                ("Players/02 - B/hair.dds", 9),
                ("Players/03 - C/torso.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("fmdl_name_invalid", Disposition::DropFolder),
                ("link_target_dropped", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Boots/Crocs"));
        assert_eq!(
            report.issues[0].context,
            vec![("file", "torso.fmdl".to_owned())]
        );
        assert_eq!(report.issues[1].scope, folder("Players/01 - A"));
        let validated = report.validated.unwrap();
        assert_eq!(
            validated
                .players
                .iter()
                .map(|player| player.path.as_str())
                .collect::<Vec<_>>(),
            vec!["Players/02 - B", "Players/03 - C"]
        );
        assert_eq!(
            validated
                .boots
                .iter()
                .map(|folder| folder.folder_name.as_str())
                .collect::<Vec<_>>(),
            vec!["Mud"]
        );
    }

    #[test]
    fn pass_through_keeps_a_dangling_link_player_minus_the_link() {
        let report = report_with(
            &context_with(true, true),
            "egg",
            &[
                ("Players/03 - A/Nowhere.boots", 0),
                ("Players/03 - A/hair.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("link_target_missing", Disposition::Keep)]
        );
        assert!(report.issues[0].passed_through);
        let player = &report.validated.unwrap().players[0];
        // The player is kept; the missing reference is not.
        assert!(player.links.is_empty());
    }

    #[test]
    fn a_common_file_off_the_allowlist_drops_or_keeps() {
        let strict = report("egg", &[("Common/x.exe", 4)], &[], &[]);
        assert_eq!(
            issue_codes(&strict),
            vec![("common_file_disallowed", Disposition::DropFile)]
        );
        assert_eq!(
            strict.issues[0].scope,
            IssueScope::File(ScopePath::new("Common/x.exe").unwrap())
        );
        assert!(strict.validated.unwrap().common.is_empty());

        let lenient = report_with(
            &context_with(false, false),
            "egg",
            &[("Common/x.exe", 4)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&lenient),
            vec![("common_file_disallowed", Disposition::Keep)]
        );
        assert_eq!(lenient.validated.unwrap().common.len(), 1);
    }

    #[test]
    fn pass_through_keeps_the_resolved_common_link_and_prunes_the_other() {
        let report = report_with(
            &context_with(true, true),
            "egg",
            &[
                ("players.txt", 5),
                ("Common/torso.fmdl", 10),
                ("Players/A/torso.fmdl.common", 0),
                ("Players/A/missing.fmdl.common", 0),
                ("Players/A/hair.dds", 9),
            ],
            &["Players/A"],
            &[("players.txt", Ok(b"03 A"))],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("common_link_missing", Disposition::Keep)]
        );
        assert!(report.issues[0].passed_through);
        let player = &report.validated.unwrap().players[0];
        assert_eq!(
            player
                .files
                .iter()
                .map(|file| file.path.name().to_owned())
                .collect::<Vec<_>>(),
            vec!["hair.dds", "torso.fmdl.common"]
        );
    }

    #[test]
    fn boots_and_fcl_hair_models_are_no_explicit_face_content() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/kit_boots.fmdl", 10),
                ("Players/03 - A/x_fcl_hair.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        assert_eq!(report.validated.unwrap().players.len(), 1);
    }

    #[test]
    fn wrong_suffixed_models_drop_their_shared_folders_and_linkers() {
        let report = report(
            "egg",
            &[
                ("Boots/X/glove_l.fmdl", 10),
                ("Gloves/Y/kit_boots.fmdl", 10),
                ("Players/01 - A/X.boots", 0),
                ("Players/01 - A/hair.dds", 9),
                ("Players/02 - B/Y.gloves", 0),
                ("Players/02 - B/hair.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("fmdl_name_invalid", Disposition::DropFolder),
                ("fmdl_name_invalid", Disposition::DropFolder),
                ("link_target_dropped", Disposition::DropFolder),
                ("link_target_dropped", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Boots/X"));
        assert_eq!(report.issues[1].scope, folder("Gloves/Y"));
        let validated = report.validated.unwrap();
        assert!(validated.players.is_empty());
        assert!(validated.boots.is_empty());
        assert!(validated.gloves.is_empty());
    }

    #[test]
    fn a_nested_common_file_is_disallowed_and_no_link_target() {
        let report = report(
            "egg",
            &[
                ("Common/sub/torso.fmdl", 10),
                ("Players/03 - A/torso.fmdl.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("common_link_missing", Disposition::DropFolder),
                ("common_file_disallowed", Disposition::DropFile),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
        assert_eq!(
            report.issues[1].scope,
            IssueScope::File(ScopePath::new("Common/sub/torso.fmdl").unwrap())
        );
    }

    #[test]
    fn reserved_folders_admit_only_model_content_and_common_links() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/face/settings.toml", 20),
                ("Players/04 - B/boots/torso.fmdl.common", 0),
                ("Common/torso.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("file_type_disallowed", Disposition::DropFolder)]
        );
        assert_eq!(report.issues[0].scope, folder("Players/03 - A"));
        assert_eq!(
            report.issues[0].context,
            vec![("file", "face/settings.toml".to_owned())]
        );
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
    }

    #[test]
    fn ingame_face_and_common_links_act_by_target_and_position() {
        let report = report(
            "egg",
            &[
                ("Common/face_high.fmdl", 10),
                ("Common/torso.fmdl", 10),
                ("Common/face_high.dds", 9),
                ("Players/01 - A/ingame_face", 0),
                ("Players/01 - A/face_high.fmdl.common", 0),
                ("Players/02 - B/ingame_face", 0),
                ("Players/02 - B/torso.fmdl.common", 0),
                ("Players/03 - C/ingame_face", 0),
                ("Players/03 - C/face_high.dds.common", 0),
                ("Players/04 - D/ingame_face", 0),
                ("Players/04 - D/boots/face_high.fmdl.common", 0),
            ],
            &[],
            &[],
        );
        // Only A's explicit-face-model link contradicts the marker.
        assert_eq!(
            issue_codes(&report),
            vec![("ingame_face_explicit_face_model", Disposition::DropFolder)]
        );
        assert_eq!(report.issues[0].scope, folder("Players/01 - A"));
        let validated = report.validated.unwrap();
        assert_eq!(
            validated
                .players
                .iter()
                .map(|player| player.path.as_str())
                .collect::<Vec<_>>(),
            vec!["Players/02 - B", "Players/03 - C", "Players/04 - D"]
        );
    }

    #[test]
    fn a_shared_folder_with_different_kinds_of_one_stem_is_fine() {
        let report = report(
            "egg",
            &[
                ("Faces/Base/hair.dds", 9),
                ("Faces/Base/hair.xml", 10),
                ("Players/03 - A/Base.face", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        assert_eq!(validated.faces.len(), 1);
    }

    #[test]
    fn a_link_file_below_an_unreserved_subfolder_is_no_link() {
        let report = report_with(
            &context_with(false, false),
            "egg",
            &[
                ("Players/03 - A/extra/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("file_type_disallowed", Disposition::Keep),
                ("shared_folder_orphaned", Disposition::DropFolder),
            ]
        );
        let player = &report.validated.unwrap().players[0];
        assert!(player.links.is_empty());
    }

    #[test]
    fn a_common_link_in_common_is_no_link_either() {
        let report = report_with(
            &context_with(false, false),
            "egg",
            &[("Players/03 - A/common/x.fmdl.common", 0)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("file_type_disallowed", Disposition::Keep)]
        );
    }

    #[test]
    fn a_dropped_target_names_its_own_first_finding() {
        let report = report(
            "egg",
            &[
                ("Players/01 - P/fpc.on", 0),
                ("Players/01 - P/fpc.off", 0),
                ("Players/01 - P/hair.dds", 9),
                ("Players/02 - Q/Base.face", 0),
                ("Players/02 - Q/hair.dds", 9),
                ("Faces/Base/notes.txt", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("fpc_conflict", Disposition::DropFolder),
                ("file_type_disallowed", Disposition::DropFolder),
                ("link_target_dropped", Disposition::DropFolder),
            ]
        );
        assert_eq!(
            report.issues[2].context,
            vec![
                ("link", "Base.face".to_owned()),
                ("target", "Faces/Base".to_owned()),
                ("finding", "file_type_disallowed".to_owned()),
            ]
        );
    }

    #[test]
    fn a_player_dropped_by_its_own_finding_reports_no_dropped_target() {
        let report = report(
            "egg",
            &[
                ("players.txt", 5),
                ("Players/A/fpc.on", 0),
                ("Players/A/fpc.off", 0),
                ("Players/A/Base.face", 0),
                ("Players/A/hair.dds", 9),
                ("Faces/Base/x.exe", 4),
            ],
            &[],
            &[("players.txt", Ok(b"03 A"))],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("fpc_conflict", Disposition::DropFolder),
                ("file_type_disallowed", Disposition::DropFolder),
            ]
        );
    }

    fn file_scope(path: &str) -> IssueScope {
        IssueScope::File(ScopePath::new(path).unwrap())
    }

    #[test]
    fn kit_slots_and_labels_parse() {
        let report = report(
            "egg",
            &[("Kits/p1 - Lakers/kit.dds", 9), ("Kits/g1/kit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let kits = &report.validated.unwrap().kits;
        let lakers = &kits.kits[&kit_config::KitSlot::P1];
        assert_eq!(lakers.folder_name, "p1 - Lakers");
        assert_eq!(lakers.label.as_deref(), Some("Lakers"));
        let goalie = &kits.kits[&kit_config::KitSlot::G1];
        assert_eq!(goalie.folder_name, "g1");
        assert_eq!(goalie.label, None);
    }

    // TC-KIT-02
    #[test]
    fn two_folders_of_one_slot_are_both_dropped() {
        let report = report(
            "egg",
            &[("Kits/p1/kit.dds", 9), ("Kits/p1 - Lakers/kit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("kit_slot_duplicate", Disposition::DropFolder),
                ("kit_slot_duplicate", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Kits/p1"));
        assert_eq!(report.issues[1].scope, folder("Kits/p1 - Lakers"));
        assert!(report.validated.unwrap().kits.kits.is_empty());
    }

    // TC-KIT-03
    #[test]
    fn a_folder_name_with_no_kit_slot_is_invalid() {
        let report = report("egg", &[], &["Kits/p10", "Kits/x1", "Kits/home"], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![
                ("kit_folder_invalid", Disposition::DropFolder),
                ("kit_folder_invalid", Disposition::DropFolder),
                ("kit_folder_invalid", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, folder("Kits/home"));
        assert_eq!(report.issues[1].scope, folder("Kits/p10"));
        assert_eq!(report.issues[2].scope, folder("Kits/x1"));
        assert!(report.validated.unwrap().kits.kits.is_empty());
    }

    #[test]
    fn an_invalid_kit_head_gets_no_own_findings() {
        let report = report("egg", &[("Kits/x1/back.dds", 9)], &[], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![("kit_folder_invalid", Disposition::DropFolder)]
        );
    }

    // TC-KIT-04
    #[test]
    fn a_kit_inherits_only_the_stems_it_lacks() {
        let report = report(
            "egg",
            &[
                ("Kits/all/kit_back.dds", 9),
                ("Kits/all/kit_name.dds", 9),
                ("Kits/p2/kit_name.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("kit_textures_inherited", Disposition::Keep)]
        );
        assert_eq!(report.issues[0].scope, folder("Kits/p2"));
        assert_eq!(report.issues[0].context, vec![("stems", "back".to_owned())]);
        let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P2];
        assert_eq!(
            kit.textures
                .iter()
                .map(|texture| (texture.stem.as_str(), texture.source))
                .collect::<Vec<_>>(),
            vec![
                ("kit_back", KitTextureSource::Shared),
                ("kit_name", KitTextureSource::Own),
            ]
        );
    }

    // TC-KIT-05
    #[test]
    fn all_files_and_an_unused_all_folder() {
        let ignored = report("egg", &[("Kits/all/config.toml", 10)], &[], &[]);
        assert_eq!(
            issue_codes(&ignored),
            vec![
                ("kit_all_file_ignored", Disposition::DropFile),
                ("kit_all_unused", Disposition::Keep),
            ]
        );
        assert_eq!(ignored.issues[0].scope, file_scope("Kits/all/config.toml"));

        let unused = report("egg", &[("Kits/all/kit_back.dds", 9)], &[], &[]);
        assert_eq!(
            issue_codes(&unused),
            vec![("kit_all_unused", Disposition::Keep)]
        );
        assert_eq!(unused.issues[0].scope, folder("Kits/all"));
        assert_eq!(unused.validated.unwrap().kits.shared.len(), 1);
    }

    // TC-KIT-06
    #[test]
    fn both_layout_markers_drop_the_kit() {
        let report = report(
            "egg",
            &[
                ("Kits/p1/kit.dds", 9),
                ("Kits/p1/pre-fox", 0),
                ("Kits/p1/fox", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("kit_layout_conflict", Disposition::DropFolder)]
        );
        assert!(report.validated.unwrap().kits.kits.is_empty());
    }

    // TC-KIT-07
    #[test]
    fn a_texture_without_the_kit_prefix_drops_only_itself() {
        let report = report(
            "egg",
            &[("Kits/p1/back.dds", 9), ("Kits/p1/kit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("kit_texture_name_invalid", Disposition::DropFile)]
        );
        assert_eq!(report.issues[0].scope, file_scope("Kits/p1/back.dds"));
        let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1];
        assert_eq!(kit.textures.len(), 1);
        assert_eq!(kit.textures[0].stem, "kit");
    }

    // TC-KIT-08
    #[test]
    fn an_out_of_range_icon_is_invalid_and_the_kit_kept() {
        let report = report(
            "egg",
            &[("Kits/p1/icon.txt", 3), ("Kits/p1/kit.dds", 9)],
            &[],
            &[("Kits/p1/icon.txt", Ok(b"25"))],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("kit_icon_invalid", Disposition::DropFile)]
        );
        assert_eq!(report.issues[0].scope, file_scope("Kits/p1/icon.txt"));
        let kit = &report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1];
        assert_eq!(kit.icon, None);
    }

    // TC-KIT-09
    #[test]
    fn stem_conflicts_drop_the_folder_they_are_in() {
        let own = report(
            "egg",
            &[("Kits/p1/kit.png", 9), ("Kits/p1/kit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&own),
            vec![("texture_stem_conflict", Disposition::DropFolder)]
        );
        assert!(own.validated.unwrap().kits.kits.is_empty());

        let shared = report(
            "egg",
            &[
                ("Kits/all/kit_back.png", 9),
                ("Kits/all/kit_back.dds", 9),
                ("Kits/p2/kit.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&shared),
            vec![("texture_stem_conflict", Disposition::DropFolder)]
        );
        assert_eq!(shared.issues[0].scope, folder("Kits/all"));
        let kits = &shared.validated.unwrap().kits;
        assert!(kits.shared.is_empty());
        assert_eq!(kits.kits[&kit_config::KitSlot::P2].textures.len(), 1);

        let overridden = report(
            "egg",
            &[("Kits/all/kit_name.dds", 9), ("Kits/p3/kit_name.png", 9)],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&overridden), vec![]);
    }

    // TC-ROOT-01
    #[test]
    fn unexpected_root_files_and_folders_are_dropped() {
        let base = report(
            "egg",
            &[
                ("extra.bin", 4),
                ("readme.TXT", 20),
                ("Players/players.txt", 10),
                ("Players/03 - A/hair.dds", 9),
            ],
            &["wrapper"],
            &[],
        );
        assert_eq!(
            issue_codes(&base),
            vec![
                ("root_file_unexpected", Disposition::DropFile),
                ("root_file_unexpected", Disposition::DropFolder),
                ("root_file_unexpected", Disposition::DropFile),
            ]
        );
        assert_eq!(base.issues[0].scope, file_scope("extra.bin"));
        assert_eq!(base.issues[1].scope, folder("wrapper"));
        assert_eq!(base.issues[2].scope, file_scope("Players/players.txt"));

        // A referee export's ref_lists.txt is admitted.
        let refs = report(
            "refs Cup",
            &[("players.txt", 10), ("ref_lists.txt", 10)],
            &["Players/Keeper"],
            &[
                ("players.txt", Ok(b"01 Keeper")),
                ("ref_lists.txt", Ok(&b"R1"[..])),
            ],
        );
        assert!(
            refs.issues
                .iter()
                .all(|issue| issue.code != "root_file_unexpected")
        );
    }

    // TC-ROOT-03
    #[test]
    fn logo_files_get_one_of_each_role() {
        let invalid = report("egg", &[("logo_zoom.png", 9)], &[], &[]);
        assert_eq!(
            issue_codes(&invalid),
            vec![("logo_file_invalid", Disposition::DropFile)]
        );
        assert_eq!(invalid.validated.unwrap().logo, None);

        let duplicate = report("egg", &[("logo.png", 9), ("logo.dds", 9)], &[], &[]);
        assert_eq!(
            issue_codes(&duplicate),
            vec![("logo_role_duplicate", Disposition::DropFile)]
        );
        assert_eq!(duplicate.issues[0].scope, file_scope("logo.png"));
        assert_eq!(
            duplicate.issues[0].context,
            vec![("other", "logo.dds".to_owned())]
        );
        assert_eq!(duplicate.validated.unwrap().logo, None);

        let small_only = report("egg", &[("logo_small.png", 9)], &[], &[]);
        assert_eq!(
            issue_codes(&small_only),
            vec![("logo_small_without_main", Disposition::DropFile)]
        );
        assert_eq!(small_only.validated.unwrap().logo, None);

        let pair = report("egg", &[("logo.png", 9), ("logo_small.png", 9)], &[], &[]);
        assert_eq!(issue_codes(&pair), vec![]);
        let logo = pair.validated.unwrap().logo.unwrap();
        assert_eq!(logo.main.file.path.as_str(), "logo.png");
        assert_eq!(logo.small.unwrap().file.path.as_str(), "logo_small.png");
    }

    // TC-ROOT-04
    #[test]
    fn a_bad_portrait_name_drops_only_that_file() {
        let report = report(
            "egg",
            &[
                ("Portraits/player_24.dds", 9),
                ("Portraits/player_03.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("portrait_name_invalid", Disposition::DropFile)]
        );
        assert_eq!(
            report.issues[0].scope,
            file_scope("Portraits/player_24.dds")
        );
        assert_eq!(
            report
                .validated
                .unwrap()
                .portraits
                .keys()
                .map(|slot| slot.get())
                .collect::<Vec<_>>(),
            vec![3]
        );
    }

    #[test]
    fn notes_are_read_or_dropped() {
        // A non-empty note is kept and reported.
        let note = report(
            "egg",
            &[("notes.txt", 12), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[("notes.txt", Ok(b"good cup"))],
        );
        assert_eq!(issue_codes(&note), vec![("notes_found", Disposition::Keep)]);
        assert_eq!(
            note.validated.unwrap().root.notes.unwrap().path.as_str(),
            "notes.txt"
        );

        // Not UTF-8: the note is dropped, the export still valid.
        let bad = report(
            "egg",
            &[("notes.txt", 4), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[("notes.txt", Ok(&b"\xff\xfe"[..]))],
        );
        assert_eq!(
            issue_codes(&bad),
            vec![("notes_encoding_invalid", Disposition::DropFile)]
        );
        let validated = bad.validated.unwrap();
        assert_eq!(validated.root.notes, None);
        assert_eq!(validated.players.len(), 1);

        // Unreadable: source_read_failed, still validated.
        let denied = report(
            "egg",
            &[("notes.txt", 5), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[("notes.txt", Err("denied"))],
        );
        assert_eq!(
            issue_codes(&denied),
            vec![("source_read_failed", Disposition::DropFile)]
        );
        assert_eq!(
            denied.issues[0].context,
            vec![("reason", "denied".to_owned())]
        );
        assert!(denied.validated.is_some());

        // Whitespace only: nothing.
        let empty = report(
            "egg",
            &[("notes.txt", 3), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[("notes.txt", Ok(b"  \n"))],
        );
        assert_eq!(issue_codes(&empty), vec![]);
        assert_eq!(empty.validated.unwrap().root.notes, None);
    }

    #[test]
    fn an_all_folder_beside_only_invalid_kits_is_unused() {
        let report = report("egg", &[("Kits/all/kit.dds", 9)], &["Kits/x1"], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![
                ("kit_folder_invalid", Disposition::DropFolder),
                ("kit_all_unused", Disposition::Keep),
            ]
        );
        assert_eq!(report.issues[1].scope, folder("Kits/all"));
        assert_eq!(report.validated.unwrap().kits.shared.len(), 1);
    }

    #[test]
    fn two_portraits_of_one_slot_conflict_each() {
        let report = report(
            "egg",
            &[
                ("Portraits/player_03.dds", 9),
                ("Portraits/player_03.png", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("texture_stem_conflict", Disposition::DropFile),
                ("texture_stem_conflict", Disposition::DropFile),
            ]
        );
        assert_eq!(
            report.issues[0].scope,
            file_scope("Portraits/player_03.dds")
        );
        assert_eq!(
            report.issues[1].scope,
            file_scope("Portraits/player_03.png")
        );
        assert!(report.validated.unwrap().portraits.is_empty());
    }

    #[test]
    fn a_logo_pair_reads_its_fit_tags() {
        let report = report(
            "egg",
            &[("logo_crop.png", 9), ("logo_small_fit.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let logo = report.validated.unwrap().logo.unwrap();
        assert_eq!(logo.main.fit, Some(LogoFit::Crop));
        assert_eq!(logo.small.unwrap().fit, Some(LogoFit::Fit));
    }

    #[test]
    fn a_non_texture_in_a_kit_drops_the_kit_strict() {
        let report = report(
            "egg",
            &[("Kits/p1/kit.dds", 9), ("Kits/p1/hair.fmdl", 10)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("file_type_disallowed", Disposition::DropFolder)]
        );
        assert_eq!(report.issues[0].scope, folder("Kits/p1"));
        assert!(report.validated.unwrap().kits.kits.is_empty());
    }

    #[test]
    fn a_kit_file_below_a_subfolder_offends() {
        let report = report(
            "egg",
            &[("Kits/p1/kit.dds", 9), ("Kits/p1/extra/kit_back.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("file_type_disallowed", Disposition::DropFolder)]
        );
        assert_eq!(
            report.issues[0].context,
            vec![("file", "extra/kit_back.dds".to_owned())]
        );
    }

    #[test]
    fn layout_markers_pick_the_kit_layout() {
        let report = report(
            "egg",
            &[
                ("Kits/p1/kit.dds", 9),
                ("Kits/p1/fox", 0),
                ("Kits/p2/kit.dds", 9),
                ("Kits/p2/pre-fox", 0),
                ("Kits/p3/kit.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let kits = &report.validated.unwrap().kits;
        assert_eq!(
            kits.kits[&kit_config::KitSlot::P1].layout,
            Some(KitLayout::Fox)
        );
        assert_eq!(
            kits.kits[&kit_config::KitSlot::P2].layout,
            Some(KitLayout::PreFox)
        );
        assert_eq!(kits.kits[&kit_config::KitSlot::P3].layout, None);
    }

    #[test]
    fn a_valid_icon_with_bom_and_crlf_reads() {
        let report = report(
            "egg",
            &[("Kits/p1/icon.txt", 5), ("Kits/p1/kit.dds", 9)],
            &[],
            &[("Kits/p1/icon.txt", Ok(&b"\xef\xbb\xbf 7\r\n"[..]))],
        );
        assert_eq!(issue_codes(&report), vec![]);
        assert_eq!(
            report.validated.unwrap().kits.kits[&kit_config::KitSlot::P1].icon,
            Some(7)
        );
    }

    #[test]
    fn a_portrait_number_must_be_exactly_two_digits() {
        let report = report("egg", &[("Portraits/player_003.dds", 9)], &[], &[]);
        assert_eq!(
            issue_codes(&report),
            vec![("portrait_name_invalid", Disposition::DropFile)]
        );
        assert!(report.validated.unwrap().portraits.is_empty());
    }

    #[test]
    fn a_logo_candidate_that_is_no_texture_is_invalid() {
        let report = report(
            "egg",
            &[("logo.txt", 9), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("logo_file_invalid", Disposition::DropFile)]
        );
        assert_eq!(report.validated.unwrap().logo, None);
    }

    #[test]
    fn a_root_colors_file_is_the_team_colors() {
        let report = report(
            "egg",
            &[("colors.txt", 10), ("Players/03 - A/hair.dds", 9)],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        assert_eq!(
            report
                .validated
                .unwrap()
                .root
                .team_colors
                .unwrap()
                .path
                .as_str(),
            "colors.txt"
        );
    }

    #[test]
    fn referee_files_are_unexpected_on_a_team_export() {
        let report = report(
            "egg",
            &[
                ("refs.txt", 10),
                ("ref_lists.txt", 10),
                ("ref_marker.dds", 9),
                ("Players/03 - A/hair.dds", 9),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("root_file_unexpected", Disposition::DropFile),
                ("root_file_unexpected", Disposition::DropFile),
                ("root_file_unexpected", Disposition::DropFile),
            ]
        );
        assert_eq!(report.issues[0].scope, file_scope("ref_lists.txt"));
        assert_eq!(report.issues[1].scope, file_scope("ref_marker.dds"));
        assert_eq!(report.issues[2].scope, file_scope("refs.txt"));
        assert_eq!(report.validated.unwrap().root.referee_marker, None);
    }

    #[test]
    fn a_ref_marker_is_admitted_on_a_refs_export() {
        let report = report(
            "refs Cup",
            &[("players.txt", 10), ("ref_marker.dds", 9)],
            &["Players/Keeper"],
            &[("players.txt", Ok(b"01 Keeper"))],
        );
        assert!(
            report
                .issues
                .iter()
                .all(|issue| issue.code != "root_file_unexpected")
        );
        assert_eq!(
            report
                .validated
                .unwrap()
                .root
                .referee_marker
                .unwrap()
                .path
                .as_str(),
            "ref_marker.dds"
        );
    }

    #[test]
    fn a_stem_ending_in_space_folds_without_panicking() {
        // `skin .png`'s stem is `skin ` — no valid path segment, still a name.
        let report = report(
            "egg",
            &[
                ("Players/03 - A/skin .png", 9),
                ("Players/03 - A/face_high.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        assert_eq!(report.validated.unwrap().players.len(), 1);
    }

    #[test]
    fn a_link_name_ending_in_space_misses_its_target() {
        // `Crocs .boots` names `Crocs ` — `Boots/Crocs` exists but does not
        // match; the name folds rather than panicking.
        let report = report(
            "egg",
            &[
                ("Players/03 - A/Crocs .boots", 0),
                ("Boots/Crocs/boots.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("link_target_missing", Disposition::DropFolder),
                ("shared_folder_orphaned", Disposition::DropFolder),
            ]
        );
    }

    #[test]
    fn a_flattened_folder_spelling_collision_is_a_conflict() {
        // `Docs/b.txt` flattens onto `docs/`'s spelling — rejected, no panic.
        let report = report(
            "egg",
            &[
                ("docs/a.txt", 4),
                ("wrapper/Docs/b.txt", 4),
                ("wrapper/Players/03 - A/face_high.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("nested_root_conflict", Disposition::DropExport)]
        );
        assert!(report.validated.is_none());
    }

    #[test]
    fn a_texture_common_link_joins_the_stem_namespace() {
        let linked = report(
            "egg",
            &[
                ("Common/hair.png", 9),
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/hair.png.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&linked),
            vec![("texture_stem_conflict", Disposition::DropFolder)]
        );
        assert_eq!(linked.issues[0].scope, folder("Players/03 - A"));
        assert!(linked.validated.unwrap().players.is_empty());

        // A model link is not a texture: same stem, no conflict.
        let model_link = report(
            "egg",
            &[
                ("Common/torso.fmdl", 10),
                ("Players/03 - A/torso.dds", 9),
                ("Players/03 - A/torso.fmdl.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&model_link), vec![]);
    }

    #[test]
    fn common_textures_conflict_each_other_and_drop_the_linker() {
        let report = report(
            "egg",
            &[
                ("Common/hair.dds", 9),
                ("Common/hair.png", 9),
                ("Players/03 - A/hair.png.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![
                ("texture_stem_conflict", Disposition::DropFile),
                ("texture_stem_conflict", Disposition::DropFile),
                ("link_target_dropped", Disposition::DropFolder),
            ]
        );
        assert_eq!(report.issues[0].scope, file_scope("Common/hair.dds"));
        assert_eq!(report.issues[1].scope, file_scope("Common/hair.png"));
        assert_eq!(report.issues[2].scope, folder("Players/03 - A"));
        let validated = report.validated.unwrap();
        assert!(validated.common.is_empty());
        assert!(validated.players.is_empty());
    }

    #[test]
    fn two_collisions_on_one_loose_path_report_once() {
        // Both flattened names claim `docs/`'s spelling: one finding.
        let report = report(
            "egg",
            &[
                ("docs/a.txt", 4),
                ("wrapper/Docs/b.txt", 4),
                ("wrapper/Docs/c.txt", 4),
                ("wrapper/Players/03 - A/face_high.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("nested_root_conflict", Disposition::DropExport)]
        );
        assert!(report.validated.is_none());
    }

    #[test]
    fn a_common_link_below_common_is_no_texture() {
        // `common/` is not a link position: the file is disallowed there but
        // never joins the stem namespace.
        let report = report(
            "egg",
            &[
                ("Common/hair.png", 9),
                ("Players/03 - A/hair.dds", 9),
                ("Players/03 - A/common/hair.png.common", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(
            issue_codes(&report),
            vec![("file_type_disallowed", Disposition::DropFolder)]
        );
    }

    #[test]
    fn fmdl_name_invalid_does_not_apply_on_pre_fox() {
        let report = report_with(
            &ValidationContext {
                version: pes_version::PesVersion::Pes16,
                strict_file_type_check: true,
                pass_through: false,
            },
            "egg",
            &[
                ("Boots/Crocs/torso.fmdl", 10),
                ("Players/03 - A/Crocs.boots", 0),
            ],
            &[],
            &[],
        );
        assert_eq!(issue_codes(&report), vec![]);
        let validated = report.validated.unwrap();
        assert_eq!(validated.boots.len(), 1);
    }
}
