//! Link files resolved against shared folders and `Common/`, and the
//! cascade ("Validation semantics" → "Dropped link targets", "Drop order"):
//! after every folder's own findings, a dropped shared folder or `Common`
//! file takes its linking players down, then a shared folder no surviving
//! player links is orphaned.

use std::collections::BTreeSet;

use vtree::ScopePath;

use super::folders::{Position, fold, is_direct_common_file, position, shared_folders, stem};
use super::roster::SlotMap;
use crate::FileKind;
use crate::conventions::{SharedKind, classify, common_link_name, shared_link_name};
use crate::listing::ValidationContext;
use crate::parse::{AestheticsExportDraft, FolderDraft};
use crate::validate::{Disposition, IssueScope, ValidationIssue, dropped_scopes, issue_in};

/// The kind of link a file is, and what it names.
pub(crate) enum ResolvedLinkKind {
    /// A shared-folder link (`Crocs.boots`).
    Shared(SharedKind),
    /// A `.common` link; the name it looked for directly under `Common/`.
    Common(String),
}

/// One link file's resolution: its name, kind, and the target when it exists.
pub(crate) struct ResolvedLink {
    /// The link file's canonical path.
    pub link_file: ScopePath,
    /// The link file's name (`Crocs.boots.txt`).
    pub link_name: String,
    /// What the link names.
    pub kind: ResolvedLinkKind,
    /// The target's canonical path, when it exists.
    pub target: Option<ScopePath>,
}

impl ResolvedLink {
    /// Whether this is a `.common` link nothing satisfies: its target is not in `Common/`, and
    /// it is not a texture link naming one of `context`'s installed Common textures. That is
    /// `common_link_missing`'s condition, and what takes the link off a kept player's files.
    pub(crate) fn common_target_missing(&self, context: &ValidationContext) -> bool {
        match &self.kind {
            ResolvedLinkKind::Shared(_) => false,
            ResolvedLinkKind::Common(name) => {
                self.target.is_none() && !is_installed_texture(name, context)
            }
        }
    }
}

/// Whether `name`, a `.common` link's target name, is a texture an installed CPK holds in the
/// team's Common output. A model or material file never is: a Fox merge needs the source
/// model, which no CPK holds.
fn is_installed_texture(name: &str, context: &ValidationContext) -> bool {
    classify(name) == FileKind::Texture
        && context
            .installed_common_textures
            .contains(&fold(stem(name)))
}

/// Every player folder's link files resolved to their targets: its direct
/// `SharedLink` and `CommonLink` files. A link below a subfolder is no link
/// (the allowlist names it out of place, `player_folders.md` "Subfolders").
pub(crate) fn player_links(
    folder: &FolderDraft,
    draft: &AestheticsExportDraft,
) -> Vec<ResolvedLink> {
    let mut links = Vec::new();
    for file in &folder.files {
        let file_position = position(&file.path, &folder.path);
        match file.kind {
            FileKind::SharedLink(kind) if file_position == Position::Direct => {
                let (_, name) = shared_link_name(file.path.name())
                    .expect("a SharedLink kind implies a non-empty link stem");
                let target = shared_folders(draft, kind)
                    .iter()
                    .find(|target| fold(target.path.name()) == fold(&name))
                    .map(|target| target.path.clone());
                links.push(ResolvedLink {
                    link_file: file.path.clone(),
                    link_name: file.path.name().to_owned(),
                    kind: ResolvedLinkKind::Shared(kind),
                    target,
                });
            }
            FileKind::CommonLink if file_position == Position::Direct => {
                let name = common_link_name(file.path.name())
                    .expect("a CommonLink kind implies a non-empty link stem");
                let target = draft
                    .common
                    .iter()
                    .find(|candidate| {
                        is_direct_common_file(&candidate.path)
                            && fold(candidate.path.name()) == fold(&name)
                    })
                    .map(|candidate| candidate.path.clone());
                links.push(ResolvedLink {
                    link_file: file.path.clone(),
                    link_name: file.path.name().to_owned(),
                    kind: ResolvedLinkKind::Common(name),
                    target,
                });
            }
            _ => {}
        }
    }
    links
}

/// `player_links` less the links a finding dropped (`dropped_files`, as `dropped_scopes`
/// reads them): a dropped link is no link, so it names no target for the cascade, the
/// orphan pass or the kept player's `links` ("Dropped link targets").
pub(crate) fn standing_links(
    folder: &FolderDraft,
    draft: &AestheticsExportDraft,
    dropped_files: &BTreeSet<String>,
) -> Vec<ResolvedLink> {
    player_links(folder, draft)
        .into_iter()
        .filter(|link| !dropped_files.contains(&link.link_file.fold_key()))
        .collect()
}

/// The first dropping issue code for a path, in issue order.
fn first_drop(issues: &[ValidationIssue], key: &str) -> &'static str {
    issues
        .iter()
        .find(|issue| {
            matches!(
                issue.disposition,
                Disposition::DropFile | Disposition::DropFolder
            ) && match &issue.scope {
                IssueScope::Folder(path) | IssueScope::File(path) => path.fold_key() == *key,
                IssueScope::Export | IssueScope::RosterEntry { .. } => false,
            }
        })
        .map(|issue| issue.code)
        .expect("a dropped target has a dropping issue")
}

/// The cascade: `link_target_dropped` for each surviving player's link to a
/// dropped target, then `shared_folder_orphaned` for shared folders no
/// surviving player links.
pub(crate) fn cascade(
    draft: &AestheticsExportDraft,
    slot_map: &SlotMap,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    let (dropped_folders, dropped_files) = dropped_scopes(issues);
    let mapped = slot_map.mapped();
    let player_dropped =
        |index: usize| dropped_folders.contains(&draft.players[index].path.fold_key());

    // link_target_dropped: every player not yet dropped whose link names a
    // dropped target. Never pass-through-eligible itself. A link a finding
    // drops itself (a content finding's, with its target) is no link.
    for (index, folder) in draft.players.iter().enumerate() {
        if !mapped.contains(&index) || player_dropped(index) {
            continue;
        }
        for link in standing_links(folder, draft, &dropped_files) {
            let Some(target) = &link.target else { continue };
            let key = target.fold_key();
            let dropped = match link.kind {
                ResolvedLinkKind::Shared(_) => dropped_folders.contains(&key),
                ResolvedLinkKind::Common(_) => dropped_files.contains(&key),
            };
            if dropped {
                issues.push(issue_in(
                    context,
                    "link_target_dropped",
                    IssueScope::Folder(folder.path.clone()),
                    vec![
                        ("link", link.link_name.clone()),
                        ("target", target.as_str().to_owned()),
                        ("finding", first_drop(issues, &key).to_owned()),
                    ],
                    Disposition::DropFolder,
                ));
            }
        }
    }

    // Refresh the drops after the cascade: a shared folder no surviving
    // player links is orphaned (a dropped one gets no finding).
    let (dropped_folders, dropped_files) = dropped_scopes(issues);
    let player_dropped =
        |index: usize| dropped_folders.contains(&draft.players[index].path.fold_key());
    let mut linked: BTreeSet<String> = BTreeSet::new();
    for index in &mapped {
        if player_dropped(*index) {
            continue;
        }
        for link in standing_links(&draft.players[*index], draft, &dropped_files) {
            if let Some(target) = link.target {
                linked.insert(target.fold_key());
            }
        }
    }
    for kind in [SharedKind::Face, SharedKind::Boots, SharedKind::Gloves] {
        for folder in shared_folders(draft, kind) {
            if !dropped_folders.contains(&folder.path.fold_key())
                && !linked.contains(&folder.path.fold_key())
            {
                issues.push(issue_in(
                    context,
                    "shared_folder_orphaned",
                    IssueScope::Folder(folder.path.clone()),
                    vec![],
                    Disposition::DropFolder,
                ));
            }
        }
    }
}
