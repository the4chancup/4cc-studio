//! The structure pass's back half: `ParsedAestheticsExport` → a report of
//! every issue plus the sanitized export when nothing drops it. `validated`
//! holds exactly the eligible content; `parsed` keeps everything for
//! rendering and repair.

mod folders;
mod issues;
mod roster;

use std::collections::BTreeMap;

use crate::conventions::is_logo_texture;
use crate::listing::ValidationContext;
use crate::parse::{FileDescriptor, ParsedAestheticsExport};

pub use folders::{FpcDirective, PlayerFolder, SharedLink, SharedModelFolder};
pub(crate) use issues::issue;
pub use issues::{Disposition, ISSUE_CODES, IssueScope, ValidationIssue};
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

/// The sanitized export: exactly the content no issue dropped. `kits`,
/// `portraits`, `logo` and `root` join when their checks land.
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
    /// `Collars/`, passed through.
    pub collars: Vec<FileDescriptor>,
    /// `Common/`, the targets of `.common` links.
    pub common: Vec<FileDescriptor>,
}

impl ParsedAestheticsExport {
    /// Infallible: every structural failure is a `ValidationIssue`, a
    /// foundational one included (`DropExport`, `validated: None`); what
    /// cannot be parsed at all is `parse_listing`'s `SourceError`. The
    /// context is unused by this slice's checks.
    pub fn validate(self, _context: &ValidationContext) -> ValidationReport {
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

        // Sanitized `players`: the draft folders a surviving assignment maps
        // and no `DropFolder` issue drops, in draft order.
        let dropped: Vec<vtree::ScopePath> = issues
            .iter()
            .filter(|issue| issue.disposition == Disposition::DropFolder)
            .filter_map(|issue| match &issue.scope {
                IssueScope::Folder(path) => Some(path.clone()),
                _ => None,
            })
            .collect();
        let is_dropped = |index: usize| {
            dropped
                .iter()
                .any(|path| path.fold_key() == draft.players[index].path.fold_key())
        };
        let mapped: Vec<usize> = match &slot_map {
            roster::SlotMap::Team(map) => map.values().copied().collect(),
            roster::SlotMap::Referees(map) => map.values().copied().collect(),
        };
        let mut kept = Vec::new();
        let mut index_of = BTreeMap::new();
        for (index, folder) in draft.players.iter().enumerate() {
            if mapped.contains(&index) && !is_dropped(index) {
                index_of.insert(index, kept.len());
                kept.push(folders::player_folder(folder, self.raw_roster.is_some()));
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
            Some(ValidatedAestheticsExport {
                export_display_name: draft.export_display_name.clone(),
                team_name: draft
                    .team_name
                    .clone()
                    .expect("a surviving export has a team name"),
                players: kept,
                roster,
                faces: draft
                    .faces
                    .iter()
                    .map(folders::shared_model_folder)
                    .collect(),
                boots: draft
                    .boots
                    .iter()
                    .map(folders::shared_model_folder)
                    .collect(),
                gloves: draft
                    .gloves
                    .iter()
                    .map(folders::shared_model_folder)
                    .collect(),
                collars: draft.collars.clone(),
                common: draft.common.clone(),
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
    use crate::testing::report;
    use crate::validate::IssueScope;
    use crate::{FileKind, ModelFormat, SharedKind};

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
                ("Players/03 - A/face_high.fmdl", 10),
                ("Players/03 - A/Crocs.boots.txt", 0),
                ("Players/03 - A/ingame_face", 0),
                ("Players/03 - A/fpc.on", 0),
                ("Players/03 - A/portrait.dds", 9),
                ("Players/03 - A/settings.toml", 20),
                ("Players/03 - A/extra/more.dds", 9),
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
        // Pipeline content: the model and the file below `extra/` alike.
        assert_eq!(
            folder
                .files
                .iter()
                .map(|f| f.path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "Players/03 - A/extra/more.dds",
                "Players/03 - A/face_high.fmdl"
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

    #[test]
    fn the_first_portrait_in_path_order_wins() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/portrait.dds", 9),
                ("Players/03 - A/portrait.png", 9),
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
            vec!["Players/03 - A/portrait.png"]
        );
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

    #[test]
    fn both_fpc_markers_leave_no_directive() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/face_high.fmdl", 10),
                ("Players/03 - A/fpc.on", 0),
                ("Players/03 - A/fpc.off", 0),
            ],
            &[],
            &[],
        );
        let folder = &report.validated.unwrap().players[0];
        assert_eq!(folder.fpc, None);
    }

    #[test]
    fn shared_folders_and_common_files_build_unchecked() {
        let report = report(
            "egg",
            &[
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
}
