//! The structure pass's back half: `ParsedAestheticsExport` → a report of
//! every issue plus the sanitized export when nothing drops it. `validated`
//! holds exactly the eligible content; `parsed` keeps everything for
//! rendering and repair. The consumer's content findings (the deep pass)
//! join through `ValidationReport::with_content_findings`, which runs the
//! same body with them.

mod folders;
mod issues;
mod kits;
mod links;
mod root;
mod roster;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use crate::conventions::{SharedKind, is_logo_texture};
use crate::listing::ValidationContext;
use crate::parse::{ExportKind, FileDescriptor, ParsedAestheticsExport};
use crate::slots::PlayerSlot;

pub use folders::{
    FpcDirective, KitFolder, KitLayout, KitTexture, KitTextureSource, KitsFolder, PlayerFolder,
    SharedLink, SharedModelFolder,
};
pub use issues::{ContentFinding, Disposition, ISSUE_CODES, IssueScope, ValidationIssue};
pub(crate) use issues::{content_issue, dropped_scopes, issue, issue_in, strict_disposition};
pub use roster::{PlayerIndex, ValidatedRoster};

/// `ParsedAestheticsExport::validate`'s report (or
/// `with_content_findings`'s): the parse retained, the sanitized export when
/// no issue drops it, and every issue.
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
    /// context decides what `pass_through` keeps, whether a disallowed file
    /// type drops its item or is only noted (`strict_file_type_check`), and
    /// which model names the target version allows.
    pub fn validate(self, context: &ValidationContext) -> ValidationReport {
        self.validate_with(context, Vec::new())
    }

    /// `validate`'s body, with the consumer's content findings as issues of
    /// its own: after every folder's own findings and `Common/`'s, before the
    /// cascade, so a dropped target takes its linking players down.
    fn validate_with(
        self,
        context: &ValidationContext,
        content_findings: Vec<ContentFinding>,
    ) -> ValidationReport {
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
        // `Common/`, the content findings, then the cascade (dropped targets,
        // orphaned shares).
        for folder in &draft.players {
            folders::check_player(draft, folder, context, &mut issues);
        }
        for kind in [SharedKind::Face, SharedKind::Boots, SharedKind::Gloves] {
            for folder in folders::shared_folders(draft, kind) {
                folders::check_shared(folder, kind, context, &mut issues);
            }
        }
        folders::check_common(draft, context, &mut issues);
        issues.extend(
            content_findings
                .into_iter()
                .map(|finding| content_issue(finding, context)),
        );
        links::cascade(draft, &slot_map, context, &mut issues);

        // Kits, portraits, logo and the root files: same order as the draft's
        // content folders end (`Kits/`, then the loose root groups).
        let mut kits = kits::check(draft, context, &mut issues);
        let mut portraits = root::check_portraits(draft, context, &mut issues);
        // No root-level finding on an undecided root: `nested_root_ambiguous`
        // and `nested_root_conflict` leave it unresolved, as for
        // `export_empty` and `root_file_unexpected`.
        let logo = if root_decided {
            root::check_logo(draft, context, &mut issues)
        } else {
            None
        };
        let mut root = if root_decided {
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
        let file_kept = |file: &FileDescriptor| !dropped_files.contains(&file.path.fold_key());
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
                        (links::ResolvedLinkKind::Shared(_), None)
                        | (links::ResolvedLinkKind::Common(_), _) => None,
                    })
                    .collect();
                player.files.retain(|file| {
                    // Only a resolved `.common` link whose target is absent
                    // (pass_through) comes off; a `.common` file at a
                    // position that is never a link keeps its own finding.
                    !resolved.iter().any(|link| {
                        matches!(&link.kind, links::ResolvedLinkKind::Common(_))
                            && link.target.is_none()
                            && link.link_file == file.path
                    })
                });
                // A dropped `settings.toml` or portrait leaves the rest of
                // the folder standing.
                player.settings = player.settings.filter(file_kept);
                player.portrait = player.portrait.filter(file_kept);
                kept.push(player);
            }
        }

        // The check functions already leave out what the structure pass
        // drops; a content finding names its item only through `issues`, so
        // every drop is applied here once more.
        kits.kits
            .retain(|_, kit| !dropped_folders.contains(&kit.path.fold_key()));
        // A dropped `colors.txt` leaves its kit, or the export, without
        // colors, as a dropped `settings.toml` leaves its player folder.
        for kit in kits.kits.values_mut() {
            kit.colors = kit.colors.take().filter(file_kept);
        }
        root.team_colors = root.team_colors.filter(file_kept);
        portraits.retain(|_, file| file_kept(file));
        let logo = logo.filter(|logo| {
            file_kept(&logo.main.file)
                && logo
                    .small
                    .as_ref()
                    .is_none_or(|small| file_kept(&small.file))
        });

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
                collars: draft
                    .collars
                    .iter()
                    .filter(|file| file_kept(file))
                    .cloned()
                    .collect(),
                common: draft
                    .common
                    .iter()
                    .filter(|file| file_kept(file))
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

impl ValidationReport {
    /// The report validation would have made had `findings`, the consumer's
    /// content checks over this report's `validated` export, been its own
    /// ("Validation semantics" → "Content findings"): each becomes an issue
    /// standing after every folder's own findings and before the cascade, an
    /// eligible `DropFile`/`DropFolder` is kept under `pass_through`, and the
    /// sanitized export is derived again without what they drop. `context`
    /// must be the one this report was made with.
    pub fn with_content_findings(
        self,
        findings: Vec<ContentFinding>,
        context: &ValidationContext,
    ) -> ValidationReport {
        self.parsed.validate_with(context, findings)
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
