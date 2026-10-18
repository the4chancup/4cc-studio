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
    /// The files that resolved nowhere, each once, in the model's order
    /// (slot by slot, then the event clips).
    pub missing: Vec<String>,
}

/// Where `file` (a `SongEntry::file`, relative to `folder`) is on disk, or
/// `None`.
///
/// Two candidates are tried in the song's folder (`folder`, or its subfolder
/// when `file` names one, a `\` separator read as `/`, so a path written on
/// Windows resolves on Linux; a path anchored by a leading separator or a
/// drive is used as written, root-level ones included, as a join does): the
/// plain file, and its `_normalized` companion, a file whose stem is
/// `<stem>_normalized` whatever its extension (real exports pair `X.mp3` with
/// `X_normalized.opus`), the one with the song's own extension first when
/// there are several. With `normalization_applies` unset the companion comes
/// first, with it set the plain file. Each candidate is its own lookup in the
/// folder's listing, names matched case-insensitively (the subfolder
/// components are used as written). The path returned carries the name as it
/// is on disk, so two spellings of one file resolve to one path. An
/// unreadable folder is "not found" for that candidate.
pub fn resolve_song(folder: &Path, file: &str, normalization_applies: bool) -> Option<PathBuf> {
    let path = folder.join(file.replace('\\', "/"));
    let dir = path.parent()?;
    let name = path.file_name()?.to_str()?;
    let (stem, extension) = split_extension(name);
    let (folded_name, companion_stem) = (name.to_lowercase(), format!("{stem}_normalized"));
    let plain = || {
        find_in(dir, |on_disk| {
            if on_disk == name {
                Some(Rank::Preferred)
            } else if on_disk.to_lowercase() == folded_name {
                Some(Rank::Acceptable)
            } else {
                None
            }
        })
    };
    let companion = || {
        find_in(dir, |on_disk| {
            let (on_disk_stem, on_disk_extension) = split_extension(on_disk);
            if !on_disk_stem.eq_ignore_ascii_case(&companion_stem) {
                return None;
            }
            Some(if on_disk_extension.eq_ignore_ascii_case(extension) {
                Rank::Preferred
            } else {
                Rank::Acceptable
            })
        })
    };
    if normalization_applies {
        plain().or_else(companion)
    } else {
        companion().or_else(plain)
    }
}

/// A file name's stem and extension (the extension with its dot); a name whose
/// only dot is its first byte (`.intro`) has no extension.
fn split_extension(name: &str) -> (&str, &str) {
    match name.rfind('.') {
        Some(index) if index > 0 => (&name[..index], &name[index..]),
        _ => (name, ""),
    }
}

/// How well a listed file fits a lookup.
enum Rank {
    /// The exact name, or the companion with the song's own extension.
    Preferred,
    /// A case-folded name, or a companion of another extension.
    Acceptable,
}

/// `dir`'s file that `rank` likes best, by its name on disk: the first it
/// ranks `Preferred`, else the `Acceptable` one with the smallest name (the
/// listing's order is the filesystem's, so it decides nothing). A listing
/// scan, not `dir.join(name).exists()`: on a case-insensitive filesystem that
/// answers with the written spelling. An unreadable folder, or an unreadable
/// entry, finds nothing.
fn find_in(dir: &Path, rank: impl Fn(&str) -> Option<Rank>) -> Option<PathBuf> {
    let mut acceptable = Vec::new();
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let on_disk = entry.file_name();
        match rank(&on_disk.to_string_lossy()) {
            Some(Rank::Preferred) => return Some(entry.path()),
            Some(Rank::Acceptable) => acceptable.push((on_disk, entry.path())),
            None => {}
        }
    }
    acceptable.into_iter().min().map(|(_, path)| path)
}

/// Every distinct song file of `export`, resolved, in the model's order: slot
/// by slot in order of first appearance, file order within each, then the
/// event clips; the files that resolve nowhere listed once each, in that order.
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
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("a stale folder is removed");
        }
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
    fn a_subfolder_written_with_backslashes_resolves_on_any_os() {
        let dir = temp_dir("backslash", &[("sub/deep/Other.MP3", b"other")]);
        assert_eq!(
            resolved(&dir, "sub\\deep\\other.mp3", true).as_deref(),
            Some("sub/deep/Other.MP3"),
            "a path written on Windows, its name matched case-insensitively in its own subfolder"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    /// A leading separator or a drive anchors the path, as a join does: it is
    /// not looked up under the export's folder.
    #[test]
    fn an_anchored_path_is_not_looked_up_under_the_folder() {
        // The subfolder's name exists under the folder alone, nowhere at a
        // root, so the anchored spellings find nothing.
        let sub = format!("music_export_resolve_anchor_{}", std::process::id());
        let relative = format!("{sub}/x.mp3");
        let dir = temp_dir("anchored", &[(relative.as_str(), b"x")]);
        assert_eq!(resolved(&dir, &format!("/{relative}"), true), None);
        assert_eq!(resolved(&dir, &format!("\\{sub}\\x.mp3"), true), None);
        assert_eq!(
            resolved(&dir, &relative, true).as_deref(),
            Some(relative.as_str()),
            "the same file under a relative path is found"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn a_name_written_in_upper_case_matches_its_lower_case_file() {
        let dir = temp_dir("upper", &[("song.mp3", b"song")]);
        assert_eq!(
            resolved(&dir, "SONG.MP3", true).as_deref(),
            Some("song.mp3")
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    /// A name whose only dot is its first byte (`.intro`) has no
    /// extension: its companion is `.intro_normalized`, not
    /// `_normalized.intro`.
    #[test]
    fn a_leading_dot_name_is_no_extension() {
        let dir = temp_dir("dotfile", &[(".intro_normalized", b"normalized")]);
        assert_eq!(
            resolved(&dir, ".intro", true).as_deref(),
            Some(".intro_normalized")
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    /// Real exports pair `X.mp3` with `X_normalized.opus` (IZAKAYA's, 17
    /// songs): the companion is matched by stem, whatever its extension.
    #[test]
    fn a_normalized_companion_of_another_extension_is_found() {
        let dir = temp_dir(
            "companion",
            &[
                ("ANTHEM.mp3", b"song"),
                ("ANTHEM_normalized.opus", b"normalized"),
                ("gone_normalized.opus", b"gone"),
            ],
        );
        assert_eq!(
            resolved(&dir, "ANTHEM.mp3", false).as_deref(),
            Some("ANTHEM_normalized.opus"),
            "the companion first when normalization does not apply"
        );
        assert_eq!(
            resolved(&dir, "ANTHEM.mp3", true).as_deref(),
            Some("ANTHEM.mp3")
        );
        assert_eq!(
            resolved(&dir, "gone.mp3", true).as_deref(),
            Some("gone_normalized.opus"),
            "the plain file missing: the companion of another extension serves"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn among_several_companions_the_song_s_own_extension_wins_case_insensitively() {
        let dir = temp_dir(
            "companions",
            &[
                ("x_normalized.opus", b"opus"),
                ("x_normalized.MP3", b"mp3"),
                ("song_NORMALIZED.ogg", b"ogg"),
                ("y_normalized.opus", b"opus"),
                ("y_normalized.ogg", b"ogg"),
            ],
        );
        assert_eq!(
            resolved(&dir, "x.mp3", false).as_deref(),
            Some("x_normalized.MP3"),
            "the song's own extension, in any case, over another"
        );
        assert_eq!(
            resolved(&dir, "y.mp3", false).as_deref(),
            Some("y_normalized.ogg"),
            "neither has the song's extension: the smaller name, whatever the listing's order"
        );
        assert_eq!(
            resolved(&dir, "Song.mp3", false).as_deref(),
            Some("song_NORMALIZED.ogg"),
            "the companion's stem matches case-insensitively"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    /// A root-level anchored path (`/x.mp3`, `C:/x.mp3`) is as anchored as one
    /// with a subfolder: the folder's own file of that name is not it.
    #[test]
    fn a_root_level_anchored_path_is_not_the_folder_s_file() {
        let name = format!("music_export_resolve_root_{}.mp3", std::process::id());
        let dir = temp_dir("root", &[(name.as_str(), b"x")]);
        assert_eq!(resolved(&dir, &format!("/{name}"), true), None);
        assert_eq!(resolved(&dir, &format!("C:/{name}"), true), None);
        assert_eq!(
            resolved(&dir, &name, true).as_deref(),
            Some(name.as_str()),
            "the relative spelling is found"
        );
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
