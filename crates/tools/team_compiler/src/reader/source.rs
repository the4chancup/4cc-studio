//! One export source read (`team_compiler/pipeline.md` "1. Reader", step 4): its eager
//! structure (the canonical listing and the small metadata files the structure pass needs), and
//! the file contents its tasks load later, from a folder, a `.zip` or a `.7z`.

use std::collections::BTreeMap;
use std::fmt::Display;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aesthetics_export::{
    CanonicalListing, ListedEntry, ListedKind, SmallMetadata, is_small_metadata,
};
use archives::Archive;
use pipeline::{MemoryBudget, Permit};

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
            SourceKind::Zip | SourceKind::SevenZ => {
                let archive = open_archive(&source.path, source.kind == SourceKind::SevenZ)?;
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
                OpenSource::Archive {
                    archive,
                    charged: source.kind == SourceKind::SevenZ,
                }
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
/// is read under a permit for what its first read decompresses (`libs/pipeline.md` "What a
/// solid `.7z` is charged"); a sum over the cap waits for the budget to empty, then runs alone.
/// An archive with no metadata file to read is not decompressed, so not charged.
fn read_archive_files(
    mut archive: Archive<File>,
    charged: bool,
    paths: &[&str],
    budget: &Arc<MemoryBudget>,
) -> BTreeMap<String, Result<Vec<u8>, String>> {
    if paths.is_empty() {
        return BTreeMap::new();
    }
    let permit = if charged {
        match budget.acquire(decompressed_size(&archive)) {
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

/// The file contents of one export, read for its tasks (`team_compiler/pipeline.md` "1. Reader",
/// step 4, "Load"). A folder reads each file from disk. An archive is opened on the first read
/// and stays open for the export's later reads, so its header is read once per export; a `.7z`
/// is held under a permit for its whole decompressed size until the export's tasks are done.
pub(crate) struct ContentSource {
    path: PathBuf,
    kind: SourceKind,
    budget: Arc<MemoryBudget>,
    // Declared before `permit`, so the decompressed buffer is freed before the bytes it was
    // charged are released.
    archive: Option<Archive<File>>,
    permit: Option<Permit>,
}

impl ContentSource {
    /// `source`'s contents, nothing opened yet; a `.7z`'s read is charged to `budget`.
    pub(crate) fn new(source: &ExportSource, budget: &Arc<MemoryBudget>) -> ContentSource {
        ContentSource {
            path: source.path.clone(),
            kind: source.kind,
            budget: Arc::clone(budget),
            archive: None,
            permit: None,
        }
    }

    /// The bytes of the file at `path`, a path of the source as a `FileDescriptor.source` gives
    /// it. A failure names the file or archive that could not be read.
    pub(crate) fn read(&mut self, path: &str) -> Result<Vec<u8>, SourceFailure> {
        if self.kind == SourceKind::Folder {
            let file = self.path.join(path);
            return fs::read(&file).map_err(|error| failure(&file, &error));
        }
        let seven_z = self.kind == SourceKind::SevenZ;
        let archive = match self.archive.take() {
            Some(archive) => archive,
            None => {
                let archive = open_archive(&self.path, seven_z)?;
                if seven_z {
                    let permit = self
                        .budget
                        .acquire(decompressed_size(&archive))
                        .map_err(|cancelled| failure(&self.path, &cancelled))?;
                    self.permit = Some(permit);
                }
                archive
            }
        };
        self.archive
            .insert(archive)
            .read(path)
            .map_err(|error| SourceFailure {
                path: path.to_owned(),
                error: error.to_string(),
            })
    }
}

/// Opens the archive at `path`, a `.7z` when `seven_z`, else a `.zip`: the dispatch the
/// extension decided once, at discovery.
fn open_archive(path: &Path, seven_z: bool) -> Result<Archive<File>, SourceFailure> {
    let file = File::open(path).map_err(|error| failure(path, &error))?;
    let archive = if seven_z {
        Archive::seven_z(file)
    } else {
        Archive::zip(file)
    };
    archive.map_err(|error| failure(path, &error))
}

/// What a `.7z`'s first read decompresses: the sum of its entries' sizes. A sum past `usize` (a
/// 32-bit host only) is over any cap, and so is the saturated value: the request is the same
/// oversized one.
fn decompressed_size(archive: &Archive<File>) -> usize {
    let total: u64 = archive.entries().iter().map(|entry| entry.size).sum();
    usize::try_from(total).unwrap_or(usize::MAX)
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
    use crate::testing::scratch;

    /// Long enough that a scheduling hiccup never outlives it; short enough that a permit that
    /// never comes fails the test rather than hanging it.
    const GUARD: Duration = Duration::from_secs(5);
    /// How long "still waiting" takes to observe.
    const BLOCKED: Duration = Duration::from_millis(100);

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
        let kind = if name.ends_with(".7z") {
            SourceKind::SevenZ
        } else {
            SourceKind::Zip
        };
        ExportSource {
            kind,
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
        let root = scratch("source_listing");
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
        let missing = scratch("source_unlistable").join("gone");
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
    fn a_7z_with_no_metadata_file_to_read_is_not_charged() {
        let budget = MemoryBudget::new(1);
        let _held = budget.acquire(1).unwrap();
        let (done_tx, done) = channel();
        let unheld = Arc::clone(&budget);
        thread::spawn(move || {
            let (open, mut listing) = OpenSource::open(&archive_source("co - Spring.7z")).unwrap();
            listing
                .entries
                .retain(|entry| matches!(entry.kind, ListedKind::Folder));
            done_tx.send(open.read_metadata(&listing, &unheld)).unwrap();
        });

        let metadata = done
            .recv_timeout(GUARD)
            .expect("nothing to read does not wait for the budget");
        assert!(metadata.files.is_empty());
    }

    #[test]
    fn content_is_read_from_a_folder_a_zip_and_a_7z() {
        let root = scratch("source_content");
        fs::create_dir_all(root.join("Kits")).unwrap();
        fs::write(root.join("Kits/notes.txt"), "a note").unwrap();
        let budget = MemoryBudget::new(1 << 20);

        let mut folder = ContentSource::new(&folder_source(root.clone()), &budget);
        assert_eq!(folder.read("Kits/notes.txt").unwrap(), b"a note");
        for name in ["co - Spring.zip", "co - Spring.7z"] {
            let mut archive = ContentSource::new(&archive_source(name), &budget);
            assert_eq!(archive.read("players.txt").unwrap(), b"01 Keeper\n");
            assert_eq!(
                archive.read("notes.txt").unwrap(),
                b"Spring kit placeholder.\n",
                "{name}: a second read from the open archive"
            );
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_7z_holds_its_charge_until_its_content_source_is_dropped() {
        // The fixture's entries sum to 34 bytes, the whole cap: a second byte must wait.
        let budget = MemoryBudget::new(34);
        let mut content = ContentSource::new(&archive_source("co - Spring.7z"), &budget);
        assert!(
            admits(&budget, 34),
            "nothing is charged before the first read"
        );

        content.read("players.txt").unwrap();

        let (admitted_tx, admitted) = channel();
        let waiting = Arc::clone(&budget);
        thread::spawn(move || {
            admitted_tx.send(waiting.acquire(1).is_ok()).unwrap();
        });
        assert_eq!(
            admitted.recv_timeout(BLOCKED),
            Err(RecvTimeoutError::Timeout),
            "the read is charged"
        );
        drop(content);
        assert_eq!(
            admitted.recv_timeout(GUARD),
            Ok(true),
            "and released with the source"
        );
    }

    #[test]
    fn a_zip_read_is_not_charged() {
        let budget = MemoryBudget::new(1);
        let _held = budget.acquire(1).unwrap();
        let (done_tx, done) = channel();
        let unheld = Arc::clone(&budget);
        thread::spawn(move || {
            let mut content = ContentSource::new(&archive_source("co - Spring.zip"), &unheld);
            done_tx.send(content.read("players.txt").is_ok()).unwrap();
        });
        assert_eq!(done.recv_timeout(GUARD), Ok(true));
    }

    #[test]
    fn content_that_cannot_be_read_names_the_file_or_the_archive() {
        let budget = MemoryBudget::new(1 << 20);
        let root = scratch("source_unreadable");
        let mut folder = ContentSource::new(&folder_source(root.clone()), &budget);
        let failure = folder.read("Kits/gone.dds").unwrap_err();
        assert_eq!(
            failure.path,
            root.join("Kits/gone.dds").display().to_string()
        );

        let mut archive = ContentSource::new(&archive_source("co - Spring.zip"), &budget);
        let failure = archive.read("Kits/gone.dds").unwrap_err();
        assert_eq!(
            failure,
            SourceFailure {
                path: "Kits/gone.dds".to_owned(),
                error: "no such entry \"Kits/gone.dds\"".to_owned(),
            }
        );

        let source = archive_source("co - Escape.zip");
        let mut refused = ContentSource::new(&source, &budget);
        let failure = refused.read("x").unwrap_err();
        assert_eq!(failure.path, source.path.display().to_string());
        fs::remove_dir_all(&root).unwrap();
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
        let root = scratch("source_metadata");
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
