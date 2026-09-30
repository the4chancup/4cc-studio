//! The export format as data, no logic: name conventions and kind tables every
//! consumer shares, so a new allowed name is a table entry, not a behavior.

mod file_types;

pub use file_types::{FileKind, Marker, MetadataFile, ModelFormat, SharedKind, classify};

/// The small file names the structure pass reads eagerly: `players.txt`,
/// `refs.txt`, `notes.txt`, `icon.txt`, by name (ASCII-case-insensitively) at
/// any depth, so the consumer needs no root normalization of its own.
pub fn is_small_metadata(path: &str) -> bool {
    const NAMES: [&str; 4] = ["players.txt", "refs.txt", "notes.txt", "icon.txt"];
    let name = path.rsplit(['/', '\\']).next().unwrap_or(path);
    NAMES.iter().any(|known| name.eq_ignore_ascii_case(known))
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
