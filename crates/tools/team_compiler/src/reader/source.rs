//! One export source's eager structure (`team_compiler/pipeline.md` "1. Reader", step 4): its
//! canonical listing and its small metadata files, the inputs of the structure pass, from a
//! folder, a `.zip` or a `.7z`.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aesthetics_export::{
    CanonicalListing, ListedEntry, ListedKind, SmallMetadata, is_small_metadata,
};
use archives::Archive;
use pipeline::MemoryBudget;

use super::{ExportSource, SourceKind};

/// Why a source could not be listed (`export_extract_failed`): the path that failed, as the
/// file system spells it, and the reason.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SourceFailure {
    /// The file or folder that could not be read.
    pub(crate) path: String,
    /// The reason, from the operating system or the archive reader.
    pub(crate) error: String,
}

/// A source opened for its structure. An archive stays open from its listing to its metadata
/// reads, so its header is read once, not again for the metadata.
pub(super) enum OpenSource {
    /// A folder: its files are read one by one from its path.
    Folder(PathBuf),
    /// A `.zip` or `.7z` archive.
    Archive {
        /// The open archive, its entry list already read.
        archive: Archive<File>,
        /// A `.7z`: its first read decompresses the whole archive, so the read is charged to
        /// the run's memory budget. A `.zip` inflates one entry at a time and is not charged.
        charged: bool,
    },
}

impl OpenSource {
    /// Opens `source` and lists every file (with its size) and every folder in it, paths
    /// relative to its root joined with `/` (an archive's as `archives` normalized them). An
    /// archive that cannot be opened (damaged, encrypted, an entry named outside the root, two
    /// entries naming one path) is a failure naming the archive.
    pub(super) fn open(
        source: &ExportSource,
    ) -> Result<(OpenSource, CanonicalListing), SourceFailure> {
        let mut entries = Vec::new();
        let open = match source.kind {
            SourceKind::Folder => {
                walk(&source.path, "", &mut entries)?;
                OpenSource::Folder(source.path.clone())
            }
            SourceKind::Archive => {
                let archive =
                    Archive::open(&source.path).map_err(|error| failure(&source.path, &error))?;
                for entry in archive.entries() {
                    entries.push(ListedEntry {
                        path: entry.path.clone(),
                        kind: ListedKind::File { size: entry.size },
                    });
                }
                // An empty folder exists in an archive only as a directory entry, and an empty
                // kit folder is a placeholder kit: leaving these out would lose the kit.
                for folder in archive.folders() {
                    entries.push(ListedEntry {
                        path: folder.clone(),
                        kind: ListedKind::Folder,
                    });
                }
                // The same extension test `Archive::open` dispatched on.
                let charged = source
                    .path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("7z"));
                OpenSource::Archive { archive, charged }
            }
        };
        let listing = CanonicalListing {
            display_name: source.display_name.clone(),
            entries,
        };
        Ok((open, listing))
    }

    /// The small metadata files of `listing` (every listed file `is_small_metadata` accepts),
    /// read from the source, which is closed afterwards. A file that cannot be read carries its
    /// reason, which the structure pass reports as `source_read_failed`.
    pub(super) fn read_metadata(
        self,
        listing: &CanonicalListing,
        budget: &Arc<MemoryBudget>,
    ) -> SmallMetadata {
        let paths: Vec<&str> = listing
            .entries
            .iter()
            .filter(|entry| {
                matches!(entry.kind, ListedKind::File { .. }) && is_small_metadata(&entry.path)
            })
            .map(|entry| entry.path.as_str())
            .collect();
        let files = match self {
            OpenSource::Folder(root) => paths
                .into_iter()
                .map(|path| {
                    let bytes = fs::read(root.join(path)).map_err(|error| error.to_string());
                    (path.to_owned(), bytes)
                })
                .collect(),
            OpenSource::Archive { archive, charged } => {
                read_archive_files(archive, charged, &paths, budget)
            }
        };
        SmallMetadata { files }
    }
}

/// `paths` read from `archive`, which is dropped before returning. A charged archive (a `.7z`)
/// is read under a permit for the sum of its entries' sizes, what its first read decompresses
/// (`libs/pipeline.md` "What a solid `.7z` is charged"); a sum over the cap waits for the
/// budget to empty, then runs alone.
fn read_archive_files(
    mut archive: Archive<File>,
    charged: bool,
    paths: &[&str],
    budget: &Arc<MemoryBudget>,
) -> BTreeMap<String, Result<Vec<u8>, String>> {
    let permit = if charged {
        let total: u64 = archive.entries().iter().map(|entry| entry.size).sum();
        // A sum past `usize` (a 32-bit host only) is over any cap, and so is the saturated
        // value: the request is the same oversized one.
        let size = usize::try_from(total).unwrap_or(usize::MAX);
        match budget.acquire(size) {
            Ok(permit) => Some(permit),
            Err(cancelled) => {
                return paths
                    .iter()
                    .map(|path| ((*path).to_owned(), Err(cancelled.to_string())))
                    .collect();
            }
        }
    } else {
        None
    };
    let files = paths
        .iter()
        .map(|path| {
            let bytes = archive.read(path).map_err(|error| error.to_string());
            ((*path).to_owned(), bytes)
        })
        .collect();
    // The permit stands for the buffer the archive now holds, so the archive goes first. Only
    // the metadata bytes outlive the structure pass: keeping the buffer until the export's
    // tasks run would hold every `.7z` export at once while the run is planned.
    drop(archive);
    drop(permit);
    files
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

fn failure(path: &Path, error: &impl Display) -> SourceFailure {
    SourceFailure {
        path: path.display().to_string(),
        error: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{RecvTimeoutError, channel};
    use std::thread;
    use std::time::Duration;

    use studio_core::ExportId;

    use super::*;

    /// Long enough that a scheduling hiccup never outlives it; short enough that a permit that
    /// never comes fails the test rather than hanging it.
    const GUARD: Duration = Duration::from_secs(5);
    /// How long "still waiting" takes to observe.
    const BLOCKED: Duration = Duration::from_millis(100);

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

    /// The archive fixture `tests/fixtures/sources/<name>` as a source, read in place.
    fn archive_source(name: &str) -> ExportSource {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sources")
            .join(name);
        ExportSource {
            kind: SourceKind::Archive,
            file_name: name.to_owned(),
            ..folder_source(path)
        }
    }

    fn sorted_entries(listing: &CanonicalListing) -> Vec<(&str, ListedKind)> {
        let mut entries: Vec<(&str, ListedKind)> = listing
            .entries
            .iter()
            .map(|entry| (entry.path.as_str(), entry.kind))
            .collect();
        entries.sort();
        entries
    }

    /// Whether `budget` admits `size` bytes within `GUARD`, asked from another thread so a
    /// budget that never frees fails the test instead of hanging it.
    fn admits(budget: &Arc<MemoryBudget>, size: usize) -> bool {
        let (admitted_tx, admitted) = channel();
        let budget = Arc::clone(budget);
        thread::spawn(move || {
            let permit = budget.acquire(size);
            admitted_tx.send(permit.is_ok()).unwrap();
        });
        admitted.recv_timeout(GUARD) == Ok(true)
    }

    /// Opens and reads the archive fixture `name` on another thread, under `budget`; the
    /// receiver gets its metadata once the read is done.
    fn read_in_background(
        name: &'static str,
        budget: &Arc<MemoryBudget>,
    ) -> std::sync::mpsc::Receiver<SmallMetadata> {
        let (done_tx, done) = channel();
        let budget = Arc::clone(budget);
        thread::spawn(move || {
            let (open, listing) = OpenSource::open(&archive_source(name)).unwrap();
            done_tx.send(open.read_metadata(&listing, &budget)).unwrap();
        });
        done
    }

    #[test]
    fn a_folder_lists_every_file_with_its_size_and_every_folder() {
        let root = scratch("listing");
        fs::create_dir_all(root.join("Players/03 - A")).unwrap();
        fs::create_dir_all(root.join("Kits/p2")).unwrap();
        fs::write(root.join("Players/03 - A/face_high.fmdl"), "12345").unwrap();
        fs::write(root.join("players.txt"), "03 A").unwrap();

        let (_, listing) = OpenSource::open(&folder_source(root.clone())).unwrap();

        assert_eq!(listing.display_name, "co - Spring");
        assert_eq!(
            sorted_entries(&listing),
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
        let Err(failure) = OpenSource::open(&folder_source(missing.clone())) else {
            panic!("listed a missing folder");
        };
        assert_eq!(failure.path, missing.display().to_string());
        assert!(!failure.error.is_empty());
    }

    #[test]
    fn an_archive_lists_its_files_and_its_directory_entries() {
        for name in ["co - Spring.zip", "co - Spring.7z"] {
            let (_, listing) = OpenSource::open(&archive_source(name)).unwrap();
            assert_eq!(listing.display_name, "co - Spring");
            assert_eq!(
                sorted_entries(&listing),
                [
                    ("Kits", ListedKind::Folder),
                    ("Kits/p2", ListedKind::Folder),
                    ("notes.txt", ListedKind::File { size: 24 }),
                    ("players.txt", ListedKind::File { size: 10 }),
                ],
                "{name}"
            );
        }
    }

    #[test]
    fn an_archive_that_cannot_be_opened_is_a_failure_naming_it() {
        let source = archive_source("co - Escape.zip");
        let Err(failure) = OpenSource::open(&source) else {
            panic!("listed an entry outside the root");
        };
        assert_eq!(
            failure,
            SourceFailure {
                path: source.path.display().to_string(),
                error: "invalid entry name \"../x\"".to_owned(),
            }
        );
    }

    #[test]
    fn a_7z_is_read_under_a_permit_for_its_whole_size_and_releases_it() {
        // The fixture's entries sum to 34 bytes: over this cap, an oversized request.
        let budget = MemoryBudget::new(1);
        let held = budget.acquire(1).unwrap();

        let done = read_in_background("co - Spring.7z", &budget);

        // An oversized request waits for the budget to empty: the read is charged.
        assert!(matches!(
            done.recv_timeout(BLOCKED),
            Err(RecvTimeoutError::Timeout)
        ));
        drop(held);
        let metadata = done.recv_timeout(GUARD).expect("the read runs alone");
        assert_eq!(metadata.files["players.txt"], Ok(b"01 Keeper\n".to_vec()));
        assert_eq!(
            metadata.files["notes.txt"],
            Ok(b"Spring kit placeholder.\n".to_vec())
        );
        // The permit went with the archive: the whole cap is free again.
        assert!(admits(&budget, 1), "the 7z's permit was not released");
    }

    #[test]
    fn a_zip_is_read_without_a_charge() {
        let budget = MemoryBudget::new(1);
        let _held = budget.acquire(1).unwrap();

        let done = read_in_background("co - Spring.zip", &budget);

        let metadata = done
            .recv_timeout(GUARD)
            .expect("a zip read does not wait for the budget");
        assert_eq!(metadata.files["players.txt"], Ok(b"01 Keeper\n".to_vec()));
    }

    #[test]
    fn a_7z_read_in_a_cancelled_run_fails_each_file_with_the_reason() {
        let budget = MemoryBudget::new(1 << 20);
        budget.cancel();
        let (open, listing) = OpenSource::open(&archive_source("co - Spring.7z")).unwrap();

        let metadata = open.read_metadata(&listing, &budget);

        let keys: Vec<&str> = metadata.files.keys().map(String::as_str).collect();
        assert_eq!(keys, ["notes.txt", "players.txt"]);
        for result in metadata.files.values() {
            assert_eq!(result, &Err("the run was cancelled".to_owned()));
        }
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

        let metadata =
            OpenSource::Folder(root.clone()).read_metadata(&listing, &MemoryBudget::new(1 << 20));

        let keys: Vec<&str> = metadata.files.keys().map(String::as_str).collect();
        assert_eq!(keys, ["Kits/p1/icon.txt", "notes.txt", "players.txt"]);
        assert_eq!(metadata.files["players.txt"], Ok(b"03 A".to_vec()));
        assert_eq!(metadata.files["Kits/p1/icon.txt"], Ok(b"3".to_vec()));
        // Listed but missing on disk: the read fails with the system's reason.
        assert!(metadata.files["notes.txt"].is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
