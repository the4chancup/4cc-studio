//! One export source's eager structure (`team_compiler/pipeline.md` "1. Reader", step 4): its
//! canonical listing and its small metadata files, the inputs of the structure pass. Folder
//! sources only in this step; `.zip` and `.7z` sources arrive in step 3.8d.

use std::fs;
use std::io;
use std::path::Path;

use aesthetics_export::{
    CanonicalListing, ListedEntry, ListedKind, SmallMetadata, is_small_metadata,
};

use super::{ExportSource, SourceKind};

/// Why a source could not be listed (`export_extract_failed`): the path that failed, as the
/// file system spells it, and the reason.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceFailure {
    /// The file or folder that could not be read.
    pub(crate) path: String,
    /// The reason, from the operating system or the reader.
    pub(crate) error: String,
}

/// What an archive source reports until step 3.8d reads archives, so a run says it skipped one
/// rather than reporting it empty.
const ARCHIVES_NOT_YET: &str = "archive sources arrive in step 3.8d";

/// Every file (with its size) and every folder in the source, paths relative to its root joined
/// with `/`.
pub(super) fn read_listing(source: &ExportSource) -> Result<CanonicalListing, SourceFailure> {
    let mut entries = Vec::new();
    match source.kind {
        SourceKind::Folder => walk(&source.path, "", &mut entries)?,
        SourceKind::Archive => {
            return Err(SourceFailure {
                path: source.path.display().to_string(),
                error: ARCHIVES_NOT_YET.to_owned(),
            });
        }
    }
    Ok(CanonicalListing {
        display_name: source.display_name.clone(),
        entries,
    })
}

/// Lists `folder` (at `prefix` within the source) and everything below it into `entries`.
fn walk(folder: &Path, prefix: &str, entries: &mut Vec<ListedEntry>) -> Result<(), SourceFailure> {
    for entry in fs::read_dir(folder).map_err(|error| failure(folder, &error))? {
        let entry = entry.map_err(|error| failure(folder, &error))?;
        let path = entry.path();
        let name = entry.file_name();
        // Export paths are UTF-8 strings from here on (`ScopePath`); a name that is not cannot
        // be represented, so the source is refused rather than misnamed.
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
            entries.push(ListedEntry {
                path: relative.clone(),
                kind: ListedKind::Folder,
            });
            walk(&path, &relative, entries)?;
        } else {
            entries.push(ListedEntry {
                path: relative,
                kind: ListedKind::File {
                    size: metadata.len(),
                },
            });
        }
    }
    Ok(())
}

fn failure(path: &Path, error: &io::Error) -> SourceFailure {
    SourceFailure {
        path: path.display().to_string(),
        error: error.to_string(),
    }
}

/// The small metadata files of `listing` (every listed file `is_small_metadata` accepts), read
/// from the source; a file that cannot be read carries its reason, which the structure pass
/// reports as `source_read_failed`.
pub(crate) fn read_metadata(source: &ExportSource, listing: &CanonicalListing) -> SmallMetadata {
    let files = listing
        .entries
        .iter()
        .filter(|entry| {
            matches!(entry.kind, ListedKind::File { .. }) && is_small_metadata(&entry.path)
        })
        .map(|entry| (entry.path.clone(), read_file(source, &entry.path)))
        .collect();
    SmallMetadata { files }
}

fn read_file(source: &ExportSource, relative: &str) -> Result<Vec<u8>, String> {
    match source.kind {
        SourceKind::Folder => {
            fs::read(source.path.join(relative)).map_err(|error| error.to_string())
        }
        SourceKind::Archive => Err(ARCHIVES_NOT_YET.to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use studio_core::ExportId;

    use super::*;

    /// A fresh `<temp>/team_compiler_source_<name>_<pid>` folder.
    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "team_compiler_source_{name}_{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        root
    }

    fn folder_source(path: PathBuf) -> ExportSource {
        ExportSource {
            export_id: ExportId(0),
            path,
            kind: SourceKind::Folder,
            file_name: "co - Spring".to_owned(),
            display_name: "co - Spring".to_owned(),
            team_name: None,
        }
    }

    #[test]
    fn a_folder_lists_every_file_with_its_size_and_every_folder() {
        let root = scratch("listing");
        fs::create_dir_all(root.join("Players/03 - A")).unwrap();
        fs::create_dir_all(root.join("Kits/p2")).unwrap();
        fs::write(root.join("Players/03 - A/face_high.fmdl"), "12345").unwrap();
        fs::write(root.join("players.txt"), "03 A").unwrap();

        let mut listing = read_listing(&folder_source(root.clone())).unwrap();

        listing.entries.sort_by(|a, b| a.path.cmp(&b.path));
        assert_eq!(listing.display_name, "co - Spring");
        let entries: Vec<(&str, ListedKind)> = listing
            .entries
            .iter()
            .map(|entry| (entry.path.as_str(), entry.kind))
            .collect();
        assert_eq!(
            entries,
            [
                ("Kits", ListedKind::Folder),
                ("Kits/p2", ListedKind::Folder),
                ("Players", ListedKind::Folder),
                ("Players/03 - A", ListedKind::Folder),
                (
                    "Players/03 - A/face_high.fmdl",
                    ListedKind::File { size: 5 }
                ),
                ("players.txt", ListedKind::File { size: 4 }),
            ]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_folder_that_cannot_be_listed_is_a_failure_naming_it() {
        let missing = scratch("unlistable").join("gone");
        let failure = read_listing(&folder_source(missing.clone())).unwrap_err();
        assert_eq!(failure.path, missing.display().to_string());
        assert!(!failure.error.is_empty());
    }

    #[test]
    fn an_archive_is_not_read_yet() {
        let mut source = folder_source(PathBuf::from("co - Spring.zip"));
        source.kind = SourceKind::Archive;
        let failure = read_listing(&source).unwrap_err();
        assert_eq!(
            failure,
            SourceFailure {
                path: "co - Spring.zip".to_owned(),
                error: ARCHIVES_NOT_YET.to_owned()
            }
        );
        let listing = CanonicalListing {
            display_name: "co - Spring".to_owned(),
            entries: vec![ListedEntry {
                path: "players.txt".to_owned(),
                kind: ListedKind::File { size: 4 },
            }],
        };
        assert_eq!(
            read_metadata(&source, &listing).files["players.txt"],
            Err(ARCHIVES_NOT_YET.to_owned())
        );
    }

    #[test]
    fn only_the_small_metadata_files_are_read_and_a_failed_read_keeps_its_reason() {
        let root = scratch("metadata");
        fs::create_dir_all(root.join("Kits/p1")).unwrap();
        fs::write(root.join("players.txt"), "03 A").unwrap();
        fs::write(root.join("Kits/p1/icon.txt"), "3").unwrap();
        fs::write(root.join("colors.txt"), "not metadata").unwrap();
        let file = |path: &str| ListedEntry {
            path: path.to_owned(),
            kind: ListedKind::File { size: 1 },
        };
        let listing = CanonicalListing {
            display_name: "co - Spring".to_owned(),
            entries: vec![
                file("players.txt"),
                file("Kits/p1/icon.txt"),
                file("colors.txt"),
                file("notes.txt"),
                ListedEntry {
                    path: "refs.txt".to_owned(),
                    kind: ListedKind::Folder,
                },
            ],
        };

        let metadata = read_metadata(&folder_source(root.clone()), &listing);

        let keys: Vec<&str> = metadata.files.keys().map(String::as_str).collect();
        assert_eq!(keys, ["Kits/p1/icon.txt", "notes.txt", "players.txt"]);
        assert_eq!(metadata.files["players.txt"], Ok(b"03 A".to_vec()));
        assert_eq!(metadata.files["Kits/p1/icon.txt"], Ok(b"3".to_vec()));
        // Listed but missing on disk: the read fails with the system's reason.
        assert!(metadata.files["notes.txt"].is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
