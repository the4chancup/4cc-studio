//! `FolderDraft` → `PlayerFolder` / `SharedModelFolder`, and the pass-through
//! file lists — the sanitized export's parts, before this slice's checks run
//! (content checks are the deep pass's).

use crate::FileKind;
use crate::conventions::{Marker, MetadataFile, SharedKind, shared_link_name, split_folder_name};
use crate::parse::{FileDescriptor, FolderDraft};

/// A player folder: the sanitized main reference point for one player.
/// Numbering is a roster property, not folder content; the roster maps the
/// slots. Typed fields hold the savefile-stage inputs and folder-level
/// references; `files` holds the model-pipeline content (models, textures,
/// material files, skeletons), however it reaches the output — files inside a
/// reserved subfolder (`face/`, `boots/`, `gloves/`, `common/`) are ordinary
/// `files` entries, the path keeping the subfolder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerFolder {
    /// The folder's path (`Players/03 - A`): the scope its issues name.
    pub path: vtree::ScopePath,
    /// The folder name without its `NN - ` number; with a roster file the
    /// whole folder name.
    pub player_name: String,
    /// Names, sizes, kinds — contents load later, per task.
    pub files: Vec<FileDescriptor>,
    /// Shared-folder link files (`Crocs.boots` → shared `Boots/Crocs/`);
    /// `.common` links are `files` entries of link kind, resolved against
    /// `Common/` by the pipeline.
    pub links: Vec<SharedLink>,
    /// A recognized `ingame_face` / `ingame_face.txt` marker.
    pub ingame_face: bool,
    /// The normalized `fpc.on`/`fpc.off` directive; `None` when absent or when
    /// both markers are present.
    pub fpc: Option<FpcDirective>,
    /// A `portrait.*` texture directly in the folder (the first, in path
    /// order, when several).
    pub portrait: Option<FileDescriptor>,
    /// `settings.toml`.
    pub settings: Option<FileDescriptor>,
}

/// A shared-folder link's target: `Crocs.boots` names `Boots/Crocs/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedLink {
    /// Which shared folder kind the link targets.
    pub kind: SharedKind,
    /// The shared folder's name (`Crocs`).
    pub name: String,
}

/// The normalized `fpc.on`/`fpc.off` marker directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FpcDirective {
    /// `fpc.on`: apply the FPC preset.
    On,
    /// `fpc.off`: un-apply the FPC preset.
    Off,
}

/// A shared model folder (`Faces/`/`Boots/`/`Gloves/` child), referenced by
/// name from player folders; IDs are assigned by run planning, not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedModelFolder {
    /// The folder's name (its path's last segment) — the link lookup key.
    pub folder_name: String,
    /// Every file below it.
    pub files: Vec<FileDescriptor>,
}

/// `FolderDraft` → `PlayerFolder`, no checks (that is the next slice's). The
/// folder's name is the player name, whole under a roster file, else the
/// `<head>[ - <label>]` split's label when present.
pub(crate) fn player_folder(draft: &FolderDraft, roster_file: bool) -> PlayerFolder {
    let name = draft.path.name();
    let player_name = if roster_file {
        name.to_owned()
    } else {
        split_folder_name(name).1.unwrap_or(name).to_owned()
    };
    let depth = draft.path.segments().count() + 1;
    let mut files = Vec::new();
    let mut links = Vec::new();
    let mut ingame_face = false;
    let mut fpc_on = false;
    let mut fpc_off = false;
    let mut portrait = None;
    let mut settings = None;
    for file in &draft.files {
        // Only a file directly in the folder can be a link, marker, settings
        // or portrait; everything below a subfolder is pipeline content.
        if file.path.segments().count() != depth {
            files.push(file.clone());
            continue;
        }
        let name = file.path.name();
        match file.kind {
            FileKind::SharedLink(_) => {
                // `shared_link_name` and `classify` share the link table, so a
                // SharedLink kind implies a non-empty stem.
                let (kind, name) = shared_link_name(name)
                    .expect("a SharedLink kind implies a non-empty link stem");
                links.push(SharedLink { kind, name });
            }
            FileKind::Marker(Marker::IngameFace) => ingame_face = true,
            FileKind::Marker(Marker::FpcOn) => fpc_on = true,
            FileKind::Marker(Marker::FpcOff) => fpc_off = true,
            FileKind::Metadata(MetadataFile::SettingsToml) => settings = Some(file.clone()),
            FileKind::Texture
                if name
                    .rsplit_once('.')
                    .is_some_and(|(stem, _)| stem.eq_ignore_ascii_case("portrait"))
                    && portrait.is_none() =>
            {
                portrait = Some(file.clone());
            }
            FileKind::Model(_)
            | FileKind::Texture
            | FileKind::Skl
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::Mtl
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::CommonLink
            | FileKind::Marker(Marker::PreFox | Marker::Fox)
            | FileKind::Metadata(
                MetadataFile::PlayersTxt
                | MetadataFile::RefsTxt
                | MetadataFile::RefLists
                | MetadataFile::NotesTxt
                | MetadataFile::ColorsTxt
                | MetadataFile::IconTxt
                | MetadataFile::ConfigToml
                | MetadataFile::Readme,
            )
            | FileKind::Other => files.push(file.clone()),
        }
    }
    // Both fpc markers present is a conflict, not a directive; the check that
    // drops such a folder is the next slice's.
    let fpc = match (fpc_on, fpc_off) {
        (true, false) => Some(FpcDirective::On),
        (false, true) => Some(FpcDirective::Off),
        _ => None,
    };
    PlayerFolder {
        path: draft.path.clone(),
        player_name,
        files,
        links,
        ingame_face,
        fpc,
        portrait,
        settings,
    }
}

/// `FolderDraft` → `SharedModelFolder`, unchecked this slice.
pub(crate) fn shared_model_folder(draft: &FolderDraft) -> SharedModelFolder {
    SharedModelFolder {
        folder_name: draft.path.name().to_owned(),
        files: draft.files.clone(),
    }
}
