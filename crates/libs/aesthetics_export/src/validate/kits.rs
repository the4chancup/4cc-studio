//! The kit folders: the `<slot>[ - <label>]` grammar (`all/` aside), the
//! allowlist, texture names, layout markers and the icon marker — the own
//! findings — then the surviving `KitsFolder` with `all/` inheritance.

use std::collections::{BTreeMap, BTreeSet};

use kit_config::KitSlot;

use vtree::ScopePath;

use super::folders::{directly_in, fold, relative, stem, stem_conflicts};
use crate::FileKind;
use crate::conventions::{Marker, MetadataFile, icon_number, split_folder_name};
use crate::listing::ValidationContext;
use crate::parse::{AestheticsExportDraft, FileDescriptor, FolderDraft};
use crate::validate::{
    Disposition, IssueScope, KitFolder, KitLayout, KitTexture, KitTextureSource, KitsFolder,
    ValidationIssue, dropped_scopes, issue_in, strict_disposition,
};

/// What a kit folder's head means: `all/`, or one of the ten kit slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum KitKind {
    /// The `all/` shared-textures folder.
    All,
    /// A kit slot (`p1`–`p9`, `g1`).
    Slot(KitSlot),
}

/// What a file directly in a kit folder may be (allowlist row "directly in a
/// kit folder"): textures, `config.toml`, `colors.txt`, the `pre-fox`, `fox`
/// and `icon_<N>` markers.
fn kit_direct_allowed(kind: FileKind) -> bool {
    matches!(
        kind,
        FileKind::Texture
            | FileKind::Metadata(MetadataFile::ConfigToml | MetadataFile::ColorsTxt)
            | FileKind::Marker(Marker::PreFox | Marker::Fox | Marker::Icon)
    )
}

/// `stem` is `kit` or starts with `kit_`, ASCII-case-insensitively.
fn is_kit_texture_name(stem: &str) -> bool {
    stem.eq_ignore_ascii_case("kit")
        || stem
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("kit_"))
}

/// The kit folders' findings and the sanitized `KitsFolder`: name grammar
/// and duplicates, each folder's own findings, `all/`'s, then building with
/// `all/` inheritance.
pub(crate) fn check(
    draft: &AestheticsExportDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) -> KitsFolder {
    // The head's kind per folder; an unparsable head is `kit_folder_invalid`.
    let mut claims: Vec<Option<KitKind>> = Vec::new();
    for folder in &draft.kits {
        let (head, _) = split_folder_name(folder.path.name());
        let kind = if head.eq_ignore_ascii_case("all") {
            Some(KitKind::All)
        } else {
            KitSlot::parse(head).map(KitKind::Slot)
        };
        if kind.is_none() {
            issues.push(issue_in(
                context,
                "kit_folder_invalid",
                IssueScope::Folder(folder.path.clone()),
                vec![],
                Disposition::DropFolder,
            ));
        }
        claims.push(kind);
    }

    // Two folders resolving to one slot (`all` included) drop each other.
    let mut by_kind: BTreeMap<KitKind, Vec<usize>> = BTreeMap::new();
    for (index, kind) in claims.iter().enumerate() {
        if let Some(kind) = kind {
            by_kind.entry(*kind).or_default().push(index);
        }
    }
    for (_, indices) in by_kind {
        if indices.len() > 1 {
            for index in indices {
                issues.push(issue_in(
                    context,
                    "kit_slot_duplicate",
                    IssueScope::Folder(draft.kits[index].path.clone()),
                    vec![],
                    Disposition::DropFolder,
                ));
            }
        }
    }

    // Each folder's own findings; `all/` gets the ignored-everything rule,
    // an invalid head none at all (it may be a misspelled `all`).
    for (index, folder) in draft.kits.iter().enumerate() {
        match claims[index] {
            Some(KitKind::All) => check_all(folder, context, issues),
            Some(KitKind::Slot(_)) => check_kit(folder, context, issues),
            None => {}
        }
    }

    // The drops so far name which folders survive (a slot-invalid or
    // duplicated folder claims nothing).
    let (dropped_folders, _) = dropped_scopes(issues);
    let surviving: Vec<usize> = (0..draft.kits.len())
        .filter(|index| {
            claims[*index].is_some()
                && !dropped_folders.contains(&draft.kits[*index].path.fold_key())
        })
        .collect();
    let all_index = surviving
        .iter()
        .copied()
        .find(|index| claims[*index] == Some(KitKind::All));
    let kit_indices: Vec<usize> = surviving
        .iter()
        .copied()
        .filter(|index| matches!(claims[*index], Some(KitKind::Slot(_))))
        .collect();
    if let Some(index) = all_index
        && kit_indices.is_empty()
    {
        issues.push(issue_in(
            context,
            "kit_all_unused",
            IssueScope::Folder(draft.kits[index].path.clone()),
            vec![],
            Disposition::Keep,
        ));
    }

    // `all/`'s surviving textures (empty when it is absent or dropped).
    let shared: Vec<FileDescriptor> = match all_index {
        Some(index) => kit_textures(&draft.kits[index]).cloned().collect(),
        None => Vec::new(),
    };

    // Each surviving kit: own surviving textures, then the `all/` stems it
    // lacks, sorted by stem; an inherited stem is reported.
    let mut kits = BTreeMap::new();
    for index in kit_indices {
        let folder = &draft.kits[index];
        let Some(KitKind::Slot(slot)) = claims[index] else {
            continue;
        };
        let own: BTreeSet<String> = kit_textures(folder)
            .map(|file| fold(stem(file.path.name())))
            .collect();
        let mut textures: Vec<KitTexture> = kit_textures(folder)
            .map(|file| KitTexture {
                stem: stem(file.path.name()).to_lowercase(),
                file: file.clone(),
                source: KitTextureSource::Own,
            })
            .collect();
        let mut inherited = Vec::new();
        for file in &shared {
            let key = fold(stem(file.path.name()));
            if !own.contains(&key) {
                inherited.push(stem(file.path.name()).to_lowercase());
                textures.push(KitTexture {
                    stem: stem(file.path.name()).to_lowercase(),
                    file: file.clone(),
                    source: KitTextureSource::Shared,
                });
            }
        }
        textures.sort_by(|a, b| a.stem.cmp(&b.stem));
        if !inherited.is_empty() {
            inherited.sort();
            let stems = inherited
                .iter()
                .map(|stem| {
                    stem.strip_prefix("kit_")
                        .unwrap_or(stem.as_str())
                        .to_owned()
                })
                .collect::<Vec<_>>()
                .join(", ");
            issues.push(issue_in(
                context,
                "kit_textures_inherited",
                IssueScope::Folder(folder.path.clone()),
                vec![("stems", stems)],
                Disposition::Keep,
            ));
        }
        let (_, label) = split_folder_name(folder.path.name());
        kits.insert(
            slot,
            KitFolder {
                path: folder.path.clone(),
                label: label.map(str::to_owned),
                config: direct_metadata(folder, MetadataFile::ConfigToml),
                colors: direct_metadata(folder, MetadataFile::ColorsTxt),
                icon: kit_icon(folder),
                layout: layout_of(folder),
                textures,
            },
        );
    }
    KitsFolder { kits, shared }
}

/// `folder`'s direct files of the given metadata kind.
fn direct_metadata(folder: &FolderDraft, kind: MetadataFile) -> Option<FileDescriptor> {
    folder
        .files
        .iter()
        .find(|file| file.kind == FileKind::Metadata(kind) && directly_in(&file.path, &folder.path))
        .cloned()
}

/// `folder`'s markers: `pre-fox`/`fox`, or `None` (`kit_layout_conflict`
/// covered the both case).
fn layout_of(folder: &FolderDraft) -> Option<KitLayout> {
    if has_direct_marker(folder, Marker::PreFox) {
        Some(KitLayout::PreFox)
    } else if has_direct_marker(folder, Marker::Fox) {
        Some(KitLayout::Fox)
    } else {
        None
    }
}

/// The kit's menu icon: its one icon marker's number, or `None` when it has
/// no icon marker, several, or one out of range (`kit_icon_invalid` covers
/// the last two).
fn kit_icon(folder: &FolderDraft) -> Option<u8> {
    match icon_markers(folder).collect::<Vec<_>>()[..] {
        [only] => valid_icon(only),
        _ => None,
    }
}

/// `folder`'s direct icon markers (`icon_7`).
fn icon_markers(folder: &FolderDraft) -> impl Iterator<Item = &FileDescriptor> {
    folder.files.iter().filter(|file| {
        file.kind == FileKind::Marker(Marker::Icon) && directly_in(&file.path, &folder.path)
    })
}

/// An icon marker's number when it is one of the game's 24 menu icons, 0–23.
fn valid_icon(marker: &FileDescriptor) -> Option<u8> {
    icon_number(marker.path.name()).filter(|number| *number <= 23)
}

/// `folder`'s direct textures that passed the name check.
fn kit_textures(folder: &FolderDraft) -> impl Iterator<Item = &FileDescriptor> {
    folder.files.iter().filter(|file| {
        file.kind == FileKind::Texture
            && directly_in(&file.path, &folder.path)
            && is_kit_texture_name(stem(file.path.name()))
    })
}

/// A kit folder's own findings: allowlist, texture names, markers, stems,
/// the icon marker.
fn check_kit(folder: &FolderDraft, context: &ValidationContext, issues: &mut Vec<ValidationIssue>) {
    let scope = IssueScope::Folder(folder.path.clone());
    let direct = |path: &ScopePath| directly_in(path, &folder.path);

    // The allowlist row, per file; everything below a subfolder offends.
    for file in &folder.files {
        if !(direct(&file.path) && kit_direct_allowed(file.kind)) {
            issues.push(issue_in(
                context,
                "file_type_disallowed",
                scope.clone(),
                vec![("file", relative(&file.path, &folder.path))],
                strict_disposition(context, Disposition::DropFolder),
            ));
        }
    }

    // Texture names: `kit` or `kit_*` — a file that fails drops only itself.
    check_texture_names(folder, context, issues);

    // Both layout markers contradict.
    if has_direct_marker(folder, Marker::PreFox) && has_direct_marker(folder, Marker::Fox) {
        issues.push(issue_in(
            context,
            "kit_layout_conflict",
            scope.clone(),
            vec![],
            Disposition::DropFolder,
        ));
    }

    // Stems collide over the textures that kept their name.
    stem_conflicts(
        context,
        scope.clone(),
        kit_textures(folder)
            .map(|file| (fold(stem(file.path.name())), file.path.name().to_owned())),
        issues,
    );

    // The icon markers: one, numbered 0–23. With several, none says which
    // icon was meant, so each is invalid whatever its number.
    let icons: Vec<&FileDescriptor> = icon_markers(folder).collect();
    for icon in &icons {
        if icons.len() > 1 || valid_icon(icon).is_none() {
            issues.push(issue_in(
                context,
                "kit_icon_invalid",
                IssueScope::File(icon.path.clone()),
                vec![],
                Disposition::DropFile,
            ));
        }
    }
}

/// Whether `folder` holds `marker` directly.
fn has_direct_marker(folder: &FolderDraft, marker: Marker) -> bool {
    folder
        .files
        .iter()
        .any(|file| file.kind == FileKind::Marker(marker) && directly_in(&file.path, &folder.path))
}

/// `kit_texture_name_invalid` for each direct texture whose stem is not
/// `kit`/`kit_*`.
fn check_texture_names(
    folder: &FolderDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    for file in &folder.files {
        if file.kind == FileKind::Texture
            && directly_in(&file.path, &folder.path)
            && !is_kit_texture_name(stem(file.path.name()))
        {
            issues.push(issue_in(
                context,
                "kit_texture_name_invalid",
                IssueScope::File(file.path.clone()),
                vec![],
                Disposition::DropFile,
            ));
        }
    }
}

/// `all/`'s own findings: anything that is not a direct texture is ignored;
/// the textures get the same name and stem rules as a kit's.
fn check_all(folder: &FolderDraft, context: &ValidationContext, issues: &mut Vec<ValidationIssue>) {
    let direct = |path: &ScopePath| directly_in(path, &folder.path);
    for file in &folder.files {
        if !(direct(&file.path) && file.kind == FileKind::Texture) {
            issues.push(issue_in(
                context,
                "kit_all_file_ignored",
                IssueScope::File(file.path.clone()),
                vec![],
                Disposition::DropFile,
            ));
        }
    }
    check_texture_names(folder, context, issues);
    stem_conflicts(
        context,
        IssueScope::Folder(folder.path.clone()),
        kit_textures(folder)
            .map(|file| (fold(stem(file.path.name())), file.path.name().to_owned())),
        issues,
    );
}
