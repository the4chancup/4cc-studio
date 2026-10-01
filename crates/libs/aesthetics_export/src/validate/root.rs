//! The root-level checks: `Portraits/` slots, the `logo*` grammar, the
//! unexpected-file allowlist, and the small root artifacts (`notes.txt`,
//! `colors.txt`, `ref_marker.dds`).

use std::collections::BTreeMap;

use super::folders::{fold, stem};
use crate::FileKind;
use crate::conventions::{MetadataFile, strip_prefix_ci};
use crate::listing::{SmallMetadata, ValidationContext};
use crate::parse::{AestheticsExportDraft, FileDescriptor};
use crate::slots::PlayerSlot;
use crate::validate::{
    Disposition, IssueScope, LogoFile, LogoFiles, LogoFit, RootArtifacts, ValidationIssue, issue_in,
};

/// `Portraits/player_NN.*`: a direct texture whose stem is `player_` plus
/// exactly two digits forming a `PlayerSlot`.
pub(crate) fn check_portraits(
    draft: &AestheticsExportDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) -> BTreeMap<PlayerSlot, FileDescriptor> {
    let mut portraits = BTreeMap::new();
    let mut slotted: Vec<(PlayerSlot, &FileDescriptor)> = Vec::new();
    for file in &draft.portraits {
        let slot = portrait_slot(stem(file.path.name()));
        if file.kind == FileKind::Texture
            && file.path.segments().count() == 2
            && let Some(slot) = slot
        {
            slotted.push((slot, file));
        } else {
            issues.push(issue_in(
                context,
                "portrait_name_invalid",
                IssueScope::File(file.path.clone()),
                vec![],
                Disposition::DropFile,
            ));
        }
    }
    // Two files claiming one stem (`player_NN`) conflict, each dropped;
    // `file_stem_conflicts` is `Common/`'s same namespace rule.
    super::folders::file_stem_conflicts(context, slotted.iter().map(|(_, file)| *file), issues);
    let mut stems: BTreeMap<String, usize> = BTreeMap::new();
    for (_, file) in &slotted {
        *stems.entry(fold(stem(file.path.name()))).or_default() += 1;
    }
    for (slot, file) in slotted {
        if stems[&fold(stem(file.path.name()))] == 1 {
            portraits.insert(slot, file.clone());
        }
    }
    portraits
}

/// `player_` plus exactly two digits forming a `PlayerSlot`.
fn portrait_slot(stem: &str) -> Option<PlayerSlot> {
    let rest = strip_prefix_ci(stem, "player_")?;
    if rest.len() == 2 && rest.bytes().all(|byte| byte.is_ascii_digit()) {
        rest.parse::<u8>().ok().and_then(PlayerSlot::new)
    } else {
        None
    }
}

/// A `logo*` stem's grammar: `logo[_small][_<fit>]`, tags in that order,
/// ASCII-case-insensitive. `(small, fit)` or `None` on a bad tag.
fn logo_parts(stem: &str) -> Option<(bool, Option<LogoFit>)> {
    let rest = strip_prefix_ci(stem, "logo")?;
    let (small, rest) = match strip_prefix_ci(rest, "_small") {
        Some(rest) => (true, rest),
        None => (false, rest),
    };
    let fit = if rest.is_empty() {
        None
    } else if rest.eq_ignore_ascii_case("_crop") {
        Some(LogoFit::Crop)
    } else if rest.eq_ignore_ascii_case("_stretch") {
        Some(LogoFit::Stretch)
    } else if rest.eq_ignore_ascii_case("_fit") {
        Some(LogoFit::Fit)
    } else {
        return None;
    };
    Some((small, fit))
}

/// The `logo*` files: candidate = a root file whose stem starts with `logo`.
/// A non-texture or bad grammar is `logo_file_invalid`; two of one role
/// `logo_role_duplicate` on the later; a small with no main
/// `logo_small_without_main`. Any finding means no `logo` at all.
pub(crate) fn check_logo(
    draft: &AestheticsExportDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) -> Option<LogoFiles> {
    let mut mains = Vec::new();
    let mut smalls = Vec::new();
    let start = issues.len();
    for file in &draft.root_files {
        if !is_logo_candidate(file) {
            continue;
        }
        match logo_parts(stem(file.path.name())) {
            Some((small, fit)) if file.kind == FileKind::Texture => {
                if small {
                    smalls.push(LogoFile {
                        file: file.clone(),
                        fit,
                    });
                } else {
                    mains.push(LogoFile {
                        file: file.clone(),
                        fit,
                    });
                }
            }
            _ => issues.push(issue_in(
                context,
                "logo_file_invalid",
                IssueScope::File(file.path.clone()),
                vec![],
                Disposition::DropFile,
            )),
        }
    }
    for group in [&mains, &smalls] {
        if let [first, rest @ ..] = group.as_slice() {
            let other = first.file.path.name().to_owned();
            for logo in rest {
                issues.push(issue_in(
                    context,
                    "logo_role_duplicate",
                    IssueScope::File(logo.file.path.clone()),
                    vec![("other", other.clone())],
                    Disposition::DropFile,
                ));
            }
        }
    }
    if mains.is_empty() {
        for logo in &smalls {
            issues.push(issue_in(
                context,
                "logo_small_without_main",
                IssueScope::File(logo.file.path.clone()),
                vec![],
                Disposition::DropFile,
            ));
        }
    }
    if issues.len() > start {
        None
    } else {
        mains.into_iter().next().map(|main| LogoFiles {
            main,
            small: smalls.into_iter().next(),
        })
    }
}

/// Whether `file` is a logo candidate: a root file whose stem starts with
/// `logo` (ASCII-case-insensitively). §4 owns these — never
/// `root_file_unexpected`.
fn is_logo_candidate(file: &FileDescriptor) -> bool {
    strip_prefix_ci(stem(file.path.name()), "logo").is_some()
}

/// The root-file allowlist plus the small root artifacts: `notes.txt`,
/// `colors.txt`, `ref_marker.dds` (referee only), the unexpected files,
/// folders and strays.
pub(crate) fn check_root(
    draft: &AestheticsExportDraft,
    metadata: &SmallMetadata,
    referees: bool,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) -> RootArtifacts {
    let mut artifacts = RootArtifacts {
        team_colors: None,
        notes: None,
        referee_marker: None,
    };
    for file in &draft.root_files {
        if is_logo_candidate(file) {
            continue;
        }
        let name = file.path.name();
        match file.kind {
            // The roster file and metadata are admitted (`refs.*` and the
            // referee marker only on a referee export).
            FileKind::Metadata(MetadataFile::PlayersTxt) => {}
            FileKind::Metadata(MetadataFile::NotesTxt) => {
                check_notes(file, metadata, context, &mut artifacts, issues);
            }
            FileKind::Metadata(MetadataFile::RefsTxt | MetadataFile::RefLists) if referees => {}
            FileKind::Metadata(MetadataFile::ColorsTxt) => {
                artifacts.team_colors = Some(file.clone());
            }
            FileKind::Metadata(MetadataFile::Readme) => {}
            FileKind::Texture if referees && name.eq_ignore_ascii_case("ref_marker.dds") => {
                artifacts.referee_marker = Some(file.clone());
            }
            _ => issues.push(issue_in(
                context,
                "root_file_unexpected",
                IssueScope::File(file.path.clone()),
                vec![],
                Disposition::DropFile,
            )),
        }
    }
    for folder in &draft.root_folders {
        issues.push(issue_in(
            context,
            "root_file_unexpected",
            IssueScope::Folder(folder.clone()),
            vec![],
            Disposition::DropFolder,
        ));
    }
    for file in &draft.stray_files {
        issues.push(issue_in(
            context,
            "root_file_unexpected",
            IssueScope::File(file.path.clone()),
            vec![],
            Disposition::DropFile,
        ));
    }
    artifacts
}

/// `notes.txt`: read, BOM, strict UTF-8; non-whitespace content is kept as
/// `root.notes` with `notes_found`.
fn check_notes(
    file: &FileDescriptor,
    metadata: &SmallMetadata,
    context: &ValidationContext,
    artifacts: &mut RootArtifacts,
    issues: &mut Vec<ValidationIssue>,
) {
    let scope = IssueScope::File(file.path.clone());
    let bytes = match metadata.bytes_for(file) {
        Ok(bytes) => bytes,
        Err(reason) => {
            issues.push(issue_in(
                context,
                "source_read_failed",
                scope,
                vec![("reason", reason)],
                Disposition::DropFile,
            ));
            return;
        }
    };
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let Ok(text) = str::from_utf8(bytes) else {
        issues.push(issue_in(
            context,
            "notes_encoding_invalid",
            scope,
            vec![],
            Disposition::DropFile,
        ));
        return;
    };
    if text.trim().is_empty() {
        return;
    }
    issues.push(issue_in(
        context,
        "notes_found",
        scope,
        vec![],
        Disposition::Keep,
    ));
    artifacts.notes = Some(file.clone());
}
