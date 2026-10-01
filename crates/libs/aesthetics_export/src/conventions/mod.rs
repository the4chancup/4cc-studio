//! The export format as data, no logic: name conventions and kind tables every
//! consumer shares, so a new allowed name is a table entry, not a behavior.

mod file_types;

pub use file_types::{FileKind, Marker, MetadataFile, ModelFormat, SharedKind, classify};
pub(crate) use file_types::{is_logo_texture, shared_link_name};

/// One of the eight content folders at the export root; the draft groups each
/// by kind. `all/` lives inside `Kits/` and is not a kind of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ContentFolder {
    /// `Players/`, the player folders.
    Players,
    /// `Faces/`, shared face folders.
    Faces,
    /// `Boots/`, shared boots folders.
    Boots,
    /// `Gloves/`, shared gloves folders.
    Gloves,
    /// `Kits/`, the kit folders (`all/` included).
    Kits,
    /// `Portraits/`.
    Portraits,
    /// `Collars/`.
    Collars,
    /// `Common/`, the team's shared content.
    Common,
}

/// The eight content folder names at the export root (matched
/// ASCII-case-insensitively): everything else at the root is an unrecognized
/// folder.
pub(crate) const CONTENT_FOLDERS: [(&str, ContentFolder); 8] = [
    ("Players", ContentFolder::Players),
    ("Faces", ContentFolder::Faces),
    ("Boots", ContentFolder::Boots),
    ("Gloves", ContentFolder::Gloves),
    ("Kits", ContentFolder::Kits),
    ("Portraits", ContentFolder::Portraits),
    ("Common", ContentFolder::Common),
    ("Collars", ContentFolder::Collars),
];

/// The small file names the structure pass reads eagerly: `players.txt`,
/// `refs.txt`, `notes.txt`, `icon.txt`, by name (ASCII-case-insensitively) at
/// any depth, so the consumer needs no root normalization of its own.
pub fn is_small_metadata(path: &str) -> bool {
    const NAMES: [&str; 4] = ["players.txt", "refs.txt", "notes.txt", "icon.txt"];
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    NAMES.iter().any(|known| name.eq_ignore_ascii_case(known))
}

/// File names a file browser writes, never an author: `parse_listing` leaves
/// them out of the draft and reports nothing ("Validation semantics" → "OS
/// artifacts").
pub(crate) const OS_ARTIFACTS: [&str; 3] = ["thumbs.db", "desktop.ini", ".ds_store"];

/// Whether `name` (a path's last segment) is an OS artifact, ASCII-case-
/// insensitively.
pub(crate) fn is_os_artifact(name: &str) -> bool {
    OS_ARTIFACTS
        .iter()
        .any(|known| name.eq_ignore_ascii_case(known))
}

/// A player or kit folder's `<head>[ - <label>]` split ("Validation
/// semantics" → "Folder names"): the name before the first `-`, trimmed, and
/// the trimmed remainder when it is non-empty.
pub(crate) fn split_folder_name(name: &str) -> (&str, Option<&str>) {
    match name.split_once('-') {
        Some((head, label)) => {
            let label = label.trim();
            (head.trim(), (!label.is_empty()).then_some(label))
        }
        None => (name.trim(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_small_metadata_recognizes_the_four_names_at_any_depth() {
        for path in [
            "players.txt",
            "wrapper/players.txt",
            "Kits/p1/ICON.TXT",
            "notes.txt",
            "refs.txt",
            "a\\b\\players.txt",
        ] {
            assert!(is_small_metadata(path), "{path}");
        }
        for path in [
            "colors.txt",
            "Players/03 - A/settings.toml",
            "config.toml",
            "players.txt.bak",
        ] {
            assert!(!is_small_metadata(path), "{path}");
        }
    }
}
