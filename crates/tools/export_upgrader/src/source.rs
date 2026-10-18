//! The input (`plans/export_upgrader.md` step 1): an old export — a folder, a
//! `.zip` or a `.7z` — listed to its root, then read file by file. The root is
//! the input's own, or its one wrapper folder's when the top level holds
//! nothing an old export holds; OS artifacts are left out of the listing and
//! never reported.

use std::fmt::Display;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use aesthetics_export::is_os_artifact;
use archives::Archive;
use vtree::ScopePath;

use crate::note;

/// Why a source could not be listed or read (`export_extract_failed`): the
/// path that failed, as the file system spells it, and the reason.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceFailure {
    /// The file or folder that could not be read.
    pub(crate) path: String,
    /// The reason, from the operating system or the archive reader.
    pub(crate) error: String,
}

/// An old export opened for reading (step 1): its name, and its files and
/// folders below its root.
pub(crate) struct OldExport {
    /// The folder's name, or the archive file's stem.
    pub(crate) name: String,
    /// Every file below the root, OS artifacts left out: `/`-joined paths
    /// relative to the root, sorted.
    pub(crate) files: Vec<String>,
    /// Every folder below the root, the same way.
    pub(crate) folders: Vec<String>,
    /// The root's prefix in the source: `""` or `co Spring/`.
    prefix: String,
    reader: Reader,
}

/// What `read` reads through: the source folder's path, or the archive opened
/// by `Archive::open`.
enum Reader {
    Folder(PathBuf),
    Archive(Archive<File>),
}

impl OldExport {
    /// Opens the export at `path`, a folder or an archive (the caller has
    /// already checked it is one), and lists it to its root. A folder's name
    /// that is not UTF-8, a path that cannot name a scope path, and an archive
    /// that does not open refuse the source.
    pub(crate) fn open(path: &Path) -> Result<OldExport, SourceFailure> {
        let name = source_name(path);
        let (files, folders, reader) = if path.is_dir() {
            let (files, folders) = list_folder(path)?;
            (
                paths_only(files),
                folders,
                Reader::Folder(path.to_path_buf()),
            )
        } else {
            let archive = Archive::open(path).map_err(|error| failure(path, &error))?;
            let files: Vec<String> = archive
                .entries()
                .iter()
                .map(|entry| entry.path.clone())
                .collect();
            let mut folders: Vec<String> = archive.folders().to_vec();
            // A writer may list no directory entries: a file's folders still
            // exist.
            for file in &files {
                let mut ancestor = file.as_str();
                while let Some((parent, _)) = ancestor.rsplit_once('/') {
                    folders.push(parent.to_owned());
                    ancestor = parent;
                }
            }
            (files, folders, Reader::Archive(archive))
        };
        let (files, folders) = below_root(files, folders);
        let prefix = root_prefix(&files, &folders);
        let (files, folders) = strip_prefix(files, folders, &prefix);
        check_paths(&files, &folders)?;
        Ok(OldExport {
            name,
            files,
            folders,
            prefix,
            reader,
        })
    }

    /// The bytes of the file at `path`, a path of `files` below the root.
    pub(crate) fn read(&mut self, path: &str) -> Result<Vec<u8>, SourceFailure> {
        match &mut self.reader {
            Reader::Folder(source) => {
                let file = source.join(&self.prefix).join(path);
                fs::read(&file).map_err(|error| failure(&file, &error))
            }
            Reader::Archive(archive) => {
                let entry = format!("{}{path}", self.prefix);
                archive.read(&entry).map_err(|error| SourceFailure {
                    path: entry,
                    error: error.to_string(),
                })
            }
        }
    }
}

/// Whether `path` names a `.zip` or a `.7z`, by its extension compared
/// ASCII-case-insensitively, as Windows treats file names.
pub(crate) fn is_archive(path: &Path) -> bool {
    let extension = path.extension().and_then(|ext| ext.to_str());
    extension.is_some_and(|ext| ext.eq_ignore_ascii_case("zip") || ext.eq_ignore_ascii_case("7z"))
}

/// The export's name as the path gives it (step 1): the folder's name, or the
/// archive file's stem.
pub(crate) fn source_name(path: &Path) -> String {
    if path.is_dir() {
        file_name(path)
    } else {
        path.file_stem().map_or_else(
            || file_name(path),
            |stem| stem.to_string_lossy().into_owned(),
        )
    }
}

/// The folder or file name at the end of `path`; the caller's resolved path
/// always has one (`resolve_input` in `upgrade.rs`).
fn file_name(path: &Path) -> String {
    path.file_name()
        .expect("the resolved path always has a file name")
        .to_string_lossy()
        .into_owned()
}

/// Whether `files`' and `folders`' direct entries — the paths holding one
/// segment — are something an old export's root holds (steps 1 and 9, names in
/// any letter case): a Note, or a `Faces`, `Boots`, `Gloves`, `Kit Configs`,
/// `Kit Textures`, `Common`, `Portraits`, `Collars`, `Logo`, `Other` or
/// `Players` folder, or a `refs.txt` file.
pub(crate) fn holds_old_export(files: &[String], folders: &[String]) -> bool {
    const OLD_FOLDERS: [&str; 11] = [
        "faces",
        "boots",
        "gloves",
        "kit configs",
        "kit textures",
        "common",
        "portraits",
        "collars",
        "logo",
        "other",
        "players",
    ];
    files
        .iter()
        .filter(|path| !path.contains('/'))
        .any(|name| note::is_note_name(name) || name.eq_ignore_ascii_case("refs.txt"))
        || folders
            .iter()
            .filter(|path| !path.contains('/'))
            .any(|name| {
                OLD_FOLDERS
                    .iter()
                    .any(|known| name.eq_ignore_ascii_case(known))
            })
}

/// The listing's entries below its root: OS artifacts left out, sorted.
fn below_root(files: Vec<String>, folders: Vec<String>) -> (Vec<String>, Vec<String>) {
    let mut files: Vec<String> = files
        .into_iter()
        .filter(|path| !is_os_artifact(path.rsplit('/').next().unwrap_or(path.as_str())))
        .collect();
    let mut folders = folders;
    files.sort();
    folders.sort();
    folders.dedup();
    (files, folders)
}

/// The root's prefix in the listing: `""`, or the one top-level folder's name
/// with a slash when the top level holds nothing an old export holds and no
/// file (OS artifacts aside) beside it. Applied once: the AET page packs an
/// export inside one main folder.
fn root_prefix(files: &[String], folders: &[String]) -> String {
    if holds_old_export(files, folders) {
        return String::new();
    }
    let top_files: Vec<&String> = files.iter().filter(|path| !path.contains('/')).collect();
    let top_folders: Vec<&String> = folders.iter().filter(|path| !path.contains('/')).collect();
    match (top_folders.as_slice(), top_files.as_slice()) {
        ([wrapper], []) => format!("{wrapper}/"),
        _ => String::new(),
    }
}

/// The listing with `prefix` stripped from every entry below it; the wrapper's
/// own folder entry goes with it.
fn strip_prefix(
    files: Vec<String>,
    folders: Vec<String>,
    prefix: &str,
) -> (Vec<String>, Vec<String>) {
    if prefix.is_empty() {
        return (files, folders);
    }
    let strip = |path: &String| path.strip_prefix(prefix).map(str::to_owned);
    let files = files.iter().filter_map(strip).collect();
    let folders = folders.iter().filter_map(strip).collect();
    (files, folders)
}

/// Every listed path must name a scope path, or the source is refused: a file
/// that cannot be scoped could not be reported or migrated.
fn check_paths(files: &[String], folders: &[String]) -> Result<(), SourceFailure> {
    for path in files.iter().chain(folders) {
        ScopePath::new(path).map_err(|error| SourceFailure {
            path: path.clone(),
            error: error.to_string(),
        })?;
    }
    Ok(())
}

/// A folder's listing: every file (its `/`-joined relative path and its size)
/// and every folder below it.
pub(crate) type FolderListing = (Vec<(String, u64)>, Vec<String>);

/// The listing of `folder` below its root, in walk order. Shared by the
/// input's folder sources and the output's validation listing.
pub(crate) fn list_folder(folder: &Path) -> Result<FolderListing, SourceFailure> {
    let mut files = Vec::new();
    let mut folders = Vec::new();
    walk(folder, "", &mut files, &mut folders)?;
    Ok((files, folders))
}

/// Lists `folder` (at `prefix` within the source) and everything below it into
/// `files` and `folders`.
fn walk(
    folder: &Path,
    prefix: &str,
    files: &mut Vec<(String, u64)>,
    folders: &mut Vec<String>,
) -> Result<(), SourceFailure> {
    for entry in fs::read_dir(folder).map_err(|error| failure(folder, &error))? {
        let entry = entry.map_err(|error| failure(folder, &error))?;
        let path = entry.path();
        let name = entry.file_name();
        // Export paths are UTF-8 strings from here on (`ScopePath`); a name
        // that is not cannot be represented, so the source is refused rather
        // than misnamed.
        let Some(name) = name.to_str() else {
            return Err(SourceFailure {
                path: path.display().to_string(),
                error: "the name is not valid UTF-8".to_owned(),
            });
        };
        let relative = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        let metadata = fs::metadata(&path).map_err(|error| failure(&path, &error))?;
        if metadata.is_dir() {
            folders.push(relative.clone());
            walk(&path, &relative, files, folders)?;
        } else {
            files.push((relative, metadata.len()));
        }
    }
    Ok(())
}

fn paths_only(files: Vec<(String, u64)>) -> Vec<String> {
    files.into_iter().map(|(path, _)| path).collect()
}

fn failure(path: &Path, error: &impl Display) -> SourceFailure {
    SourceFailure {
        path: path.display().to_string(),
        error: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `files` and `folders` as `below_root`'s caller builds them (unsorted is
    /// fine: it sorts).
    fn listing(files: &[&str], folders: &[&str]) -> (Vec<String>, Vec<String>) {
        below_root(
            files.iter().map(|path| (*path).to_owned()).collect(),
            folders.iter().map(|path| (*path).to_owned()).collect(),
        )
    }

    #[test]
    fn a_wrapper_folder_becomes_the_root() {
        let (files, folders) = listing(
            &["co Spring/co Note.txt", "co Spring/Other/readme.txt"],
            &["co Spring", "co Spring/Other"],
        );
        let prefix = root_prefix(&files, &folders);
        assert_eq!(prefix, "co Spring/");
        assert_eq!(
            strip_prefix(files, folders, &prefix),
            (
                vec!["Other/readme.txt".to_owned(), "co Note.txt".to_owned()],
                vec!["Other".to_owned()]
            )
        );
    }

    #[test]
    fn a_wrapper_beside_another_file_is_not_unwrapped() {
        let (files, folders) = listing(&["co Spring/co Note.txt", "readme.pdf"], &["co Spring"]);
        assert_eq!(root_prefix(&files, &folders), "");
        // And a wrapper beside a second folder is not either.
        let (files, folders) = listing(
            &["co Spring/co Note.txt", "extra/x.txt"],
            &["co Spring", "extra"],
        );
        assert_eq!(root_prefix(&files, &folders), "");
    }

    #[test]
    fn a_top_level_os_artifact_is_aside_for_the_root_rule() {
        let (files, folders) = listing(&["co Spring/co Note.txt", "Thumbs.db"], &["co Spring"]);
        assert_eq!(root_prefix(&files, &folders), "co Spring/");
    }

    #[test]
    fn a_path_that_cannot_name_a_scope_path_refuses_the_source() {
        // A `.zip` can hold a folder whose name ends in a space; `ScopePath`
        // refuses the segment, so the whole listing is refused.
        let Err(failure) = check_paths(&["Other /readme.txt".to_owned()], &[]) else {
            panic!("accepted a trailing-space segment");
        };
        assert_eq!(failure.path, "Other /readme.txt");
        assert!(!failure.error.is_empty());
        // And a clean listing passes.
        assert!(check_paths(&["Other/readme.txt".to_owned()], &["Other".to_owned()]).is_ok());
    }

    #[test]
    fn an_old_exports_own_root_is_not_unwrapped() {
        let (files, folders) = listing(&["co Note.txt", "Other/readme.txt"], &["Other"]);
        assert_eq!(root_prefix(&files, &folders), "");
        // A referee export's `refs.txt` and `Players/` count the same way.
        let (files, folders) = listing(&["refs.txt"], &["Players"]);
        assert_eq!(root_prefix(&files, &folders), "");
    }
}
