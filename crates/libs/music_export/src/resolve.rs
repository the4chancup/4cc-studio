//! Resolving an export's song paths against the `.4ccm`'s folder before a
//! match: each `SongEntry::file` looked up under the case-insensitive and
//! `_normalized` companion rules, and every file that resolves nowhere
//! collected into one list. Native only: the wasm build has no filesystem.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::model::MusicExport;

/// The songs of an export located on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    /// Each resolved `SongEntry::file`, with the path it resolved to.
    pub found: Vec<(String, PathBuf)>,
    /// The files that resolved nowhere, each once, in file order.
    pub missing: Vec<String>,
}

/// Where `file` (a `SongEntry::file`, relative to `folder`) is on disk, or
/// `None`.
///
/// Two candidates are tried in `folder` (and in `file`'s subfolder when it
/// names one): with `normalization_applies` unset, `<stem>_normalized.<ext>`
/// first, then `file`; with it set, the other way round. Each candidate is
/// its own lookup in the folder's listing, its file name matched
/// case-insensitively (the subfolder components are used as written). The
/// path returned carries the name as it is on disk, so two spellings of one
/// file resolve to one path. An unreadable folder is "not found" for that
/// candidate.
pub fn resolve_song(folder: &Path, file: &str, normalization_applies: bool) -> Option<PathBuf> {
    let (parent, name) = match file.rfind(['/', '\\']) {
        Some(index) => (&file[..index], &file[index + 1..]),
        None => ("", file),
    };
    let normalized = match name.rfind('.') {
        Some(index) if index > 0 => format!("{}_normalized{}", &name[..index], &name[index..]),
        _ => format!("{name}_normalized"),
    };
    let dir = parent
        .split(['/', '\\'])
        .filter(|component| !component.is_empty())
        .fold(folder.to_path_buf(), |dir, component| dir.join(component));
    let candidates: [&str; 2] = if normalization_applies {
        [name, normalized.as_str()]
    } else {
        [normalized.as_str(), name]
    };
    candidates.into_iter().find_map(|name| find_in(&dir, name))
}

/// `dir`'s file named `name`, by its name on disk: the listing's exact match,
/// else its first case-insensitive one. Not `dir.join(name).exists()`: on a
/// case-insensitive filesystem that answers with `name`'s own spelling. An
/// unreadable folder, or an unreadable entry, finds nothing.
fn find_in(dir: &Path, name: &str) -> Option<PathBuf> {
    let wanted = name.to_lowercase();
    let mut folded = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let on_disk = entry.file_name();
        let on_disk = on_disk.to_string_lossy();
        if on_disk == name {
            return Some(entry.path());
        }
        if folded.is_none() && on_disk.to_lowercase() == wanted {
            folded = Some(entry.path());
        }
    }
    folded
}

/// Every distinct song file of `export` (slot entries and event clips, in
/// file order), resolved; the files that resolve nowhere listed once each,
/// in file order.
pub fn resolve_export(
    export: &MusicExport,
    folder: &Path,
    normalization_applies: bool,
) -> Resolution {
    let files = export
        .slots
        .values()
        .flatten()
        .map(|entry| entry.file.as_str())
        .chain(export.events.iter().map(|clip| clip.entry.file.as_str()));
    let mut seen = HashSet::new();
    let (mut found, mut missing) = (Vec::new(), Vec::new());
    for file in files {
        if !seen.insert(file) {
            continue;
        }
        match resolve_song(folder, file, normalization_applies) {
            Some(path) => found.push((file.to_string(), path)),
            None => missing.push(file.to_string()),
        }
    }
    Resolution { found, missing }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    /// A unique temp folder holding `files` (each `<name>=<bytes>`, a `/`
    /// naming a subfolder), removed by hand at the test's end.
    fn temp_dir(name: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "music_export_resolve_{}_{}",
            name,
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&dir));
        for (file, bytes) in files {
            let path = dir.join(file);
            std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
            std::fs::write(path, bytes).expect("seed");
        }
        dir
    }

    /// The path `resolve_song` returns, relative to `dir`, with `/`
    /// separators: the name as it is on disk.
    fn resolved(dir: &Path, file: &str, normalization_applies: bool) -> Option<String> {
        resolve_song(dir, file, normalization_applies).map(|path| {
            path.strip_prefix(dir)
                .expect("under the folder")
                .to_string_lossy()
                .replace('\\', "/")
        })
    }

    fn export(text: &str) -> MusicExport {
        parse(format!("name;jp\n{text}").as_bytes(), "jp")
            .expect("parses")
            .export
    }

    #[test]
    fn a_song_resolves_its_file_then_the_normalized_companion() {
        let dir = temp_dir(
            "songs",
            &[
                ("Song.mp3", b"song"),
                ("song_normalized.mp3", b"normalized"),
                ("sub/Other.MP3", b"other"),
            ],
        );
        assert_eq!(
            resolved(&dir, "song.mp3", true).as_deref(),
            Some("Song.mp3"),
            "the plain file first while normalization applies, by its name on disk"
        );
        assert_eq!(
            resolved(&dir, "song.mp3", false).as_deref(),
            Some("song_normalized.mp3"),
            "the normalized companion first when it does not"
        );
        assert_eq!(
            resolved(&dir, "sub/other.mp3", true).as_deref(),
            Some("sub/Other.MP3"),
            "the name matches case-insensitively inside its own subfolder"
        );
        assert_eq!(resolved(&dir, "Gone.mp3", true), None);
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn a_missing_plain_file_falls_back_to_the_normalized_companion() {
        let dir = temp_dir("gone", &[("gone_normalized.mp3", b"gone")]);
        assert_eq!(
            resolved(&dir, "gone.mp3", true).as_deref(),
            Some("gone_normalized.mp3"),
            "the plain file missing: the normalized file is used regardless"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn an_export_resolves_each_distinct_file_and_reports_the_missing_once() {
        let dir = temp_dir("export", &[("Song.mp3", b"song")]);
        let export = export(
            "goal;Song.mp3\nBilly;Song.mp3\nBilly;a.mp3\n\
             Billy;b.mp3;event red\nBilly;a.mp3;event yellow\n",
        );
        let resolution = resolve_export(&export, &dir, true);
        assert_eq!(
            resolution.found,
            [("Song.mp3".to_string(), dir.join("Song.mp3"))],
            "Song.mp3 named twice resolves once"
        );
        assert_eq!(
            resolution.missing,
            ["a.mp3".to_string(), "b.mp3".to_string()],
            "the slot and the event clip's missing files, in file order"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
