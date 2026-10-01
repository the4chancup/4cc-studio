//! Stage 1, where exports come from (`team_compiler/pipeline.md` "1. Reader", steps 1 and 2):
//! discovery of the export sources, their team names, and the routing that sets aside the
//! disabled, balls and conflicting referee exports before any validation. The only module that
//! touches export sources.

mod source;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use aesthetics_export::{CanonicalListing, ListedKind, SmallMetadata};
use anyhow::Context;
use pipeline::MemoryBudget;
use studio_core::ExportId;
use teams_list::TeamName;

use source::OpenSource;
pub(crate) use source::{ContentSource, SourceFailure};

/// One export source found by discovery: a folder or an archive.
#[derive(Debug)]
pub(crate) struct ExportSource {
    /// The source's id for this run, sequential in discovery order.
    pub(crate) export_id: ExportId,
    /// Where the source is on disk.
    pub(crate) path: PathBuf,
    /// What the source is, which decides how it is read.
    pub(crate) kind: SourceKind,
    /// The folder or archive name, extension included: what every console line names, so a
    /// folder and an archive sharing a stem stay apart.
    pub(crate) file_name: String,
    /// The stem: the folder name, or the archive name without extension.
    pub(crate) display_name: String,
    /// The canonical team name from the stem; `None` when the stem has no token, or when its
    /// first token does not fold to a valid team name.
    pub(crate) team_name: Option<TeamName>,
}

/// What an export source is, which decides how it is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceKind {
    /// A plain folder.
    Folder,
    /// A `.zip`: read one entry at a time.
    Zip,
    /// A `.7z`: its first read decompresses the whole archive.
    SevenZ,
}

/// What the reader decided for one source before validation (pipeline.md steps 1 and 2).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Route {
    /// The source could not be listed: `export_extract_failed`.
    Unreadable(SourceFailure),
    /// A `NO_USE` marker at its root: `export_disabled`, and nothing else.
    Disabled,
    /// A `/balls/` export, the Balls compiler's: `export_balls_skipped`.
    Balls,
    /// One of several non-disabled `/refs/` exports: `multiple_ref_exports`.
    ConflictingRefs,
    /// To be validated, from this listing and the small metadata files read with it.
    Validate {
        /// Every file and folder in the source.
        listing: CanonicalListing,
        /// The listing's small metadata files, read from the source.
        metadata: SmallMetadata,
    },
}

/// Whether `path` names a `.zip` or `.7z` archive.
pub(crate) fn is_archive(path: &Path) -> bool {
    archive_kind(path).is_some()
}

/// The archive kind `path`'s extension names, compared ASCII-case-insensitively, as Windows
/// treats file names; `None` for any other extension.
fn archive_kind(path: &Path) -> Option<SourceKind> {
    let extension = path.extension()?.to_str()?;
    if extension.eq_ignore_ascii_case("zip") {
        Some(SourceKind::Zip)
    } else if extension.eq_ignore_ascii_case("7z") {
        Some(SourceKind::SevenZ)
    } else {
        None
    }
}

/// What `path` is as an export source: a folder, or an archive by its extension.
fn source_kind(path: &Path) -> Option<SourceKind> {
    if path.is_dir() {
        Some(SourceKind::Folder)
    } else {
        archive_kind(path)
    }
}

/// The export sources of a run: exactly the `exports` paths when any are given (the root is then
/// not scanned), else every folder and `.zip`/`.7z` file directly in `exports_root`, other files
/// ignored. Sorted by folded file name, then by the name itself, so the order is the same on
/// every file system, names differing only in case included, and numbered in that order. An
/// exports root that cannot be listed is an error naming it, and so is a named path that is no
/// longer a folder or an archive.
pub(crate) fn discover(
    exports_root: &Path,
    exports: &[PathBuf],
) -> anyhow::Result<Vec<ExportSource>> {
    let mut paths = if exports.is_empty() {
        scan_root(exports_root)?
    } else {
        named_paths(exports)?
    };
    paths.sort_by_cached_key(|(path, _)| {
        let name = file_name(path);
        (vtree::fold_name(&name), name)
    });
    Ok(paths
        .into_iter()
        .zip(0..)
        .map(|((path, kind), id)| source(path, kind, ExportId(id)))
        .collect())
}

fn scan_root(exports_root: &Path) -> anyhow::Result<Vec<(PathBuf, SourceKind)>> {
    let cannot_read = || format!("{}: cannot read the exports folder", exports_root.display());
    let mut paths = Vec::new();
    for entry in fs::read_dir(exports_root).with_context(cannot_read)? {
        let path = entry.with_context(cannot_read)?.path();
        if let Some(kind) = source_kind(&path) {
            paths.push((path, kind));
        }
    }
    Ok(paths)
}

/// The `--export` paths with their kinds. The preflight refused any other path; one that
/// changed since is an error naming it.
fn named_paths(exports: &[PathBuf]) -> anyhow::Result<Vec<(PathBuf, SourceKind)>> {
    exports
        .iter()
        .map(|path| match source_kind(path) {
            Some(kind) => Ok((path.clone(), kind)),
            None => anyhow::bail!("--export {}: not a folder, .zip or .7z", path.display()),
        })
        .collect()
}

/// The folder or archive name at the end of `path`. `--export .` has none of its own; the path
/// as given stands in for it.
fn file_name(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// The source at `path`, of `kind`.
fn source(path: PathBuf, kind: SourceKind, export_id: ExportId) -> ExportSource {
    let file_name = file_name(&path);
    let display_name = match kind {
        SourceKind::Folder => file_name.clone(),
        SourceKind::Zip | SourceKind::SevenZ => path.file_stem().map_or_else(
            || file_name.clone(),
            |stem| stem.to_string_lossy().into_owned(),
        ),
    };
    let team_name = aesthetics_export::team_name(&display_name);
    ExportSource {
        export_id,
        path,
        kind,
        file_name,
        display_name,
        team_name,
    }
}

/// Each source's route, in the order given, its `.7z` reads charged to `budget`. The
/// duplicate-refs rule counts only the `/refs/` exports still headed for validation, so a
/// disabled or unreadable one never conflicts.
pub(crate) fn route(sources: &[ExportSource], budget: &Arc<MemoryBudget>) -> Vec<Route> {
    let mut routes: Vec<Route> = sources
        .iter()
        .map(|source| route_source(source, budget))
        .collect();
    let refs: Vec<usize> = sources
        .iter()
        .zip(&routes)
        .enumerate()
        .filter(|(_, (source, route))| {
            matches!(route, Route::Validate { .. })
                && source.team_name.as_ref().is_some_and(TeamName::is_referees)
        })
        .map(|(index, _)| index)
        .collect();
    if refs.len() > 1 {
        for index in refs {
            routes[index] = Route::ConflictingRefs;
        }
    }
    routes
}

/// One source's route before the duplicate-refs rule. The listing comes first, since a disabled
/// export is recognized by its root's files; only an export headed for validation has its
/// metadata read, from the source its listing opened, so a disabled or balls `.7z` is never
/// decompressed.
fn route_source(source: &ExportSource, budget: &Arc<MemoryBudget>) -> Route {
    let (open, listing) = match OpenSource::open(source) {
        Ok(opened) => opened,
        Err(failure) => return Route::Unreadable(failure),
    };
    if is_disabled(&listing) {
        return Route::Disabled;
    }
    if source.team_name.as_ref().is_some_and(TeamName::is_balls) {
        return Route::Balls;
    }
    let metadata = open.read_metadata(&listing, budget);
    Route::Validate { listing, metadata }
}

/// A `NO_USE` or `NO_USE.txt` file directly in the source's own root. Compared folded, as
/// Windows does; a marker below a wrapper folder is the nested root's content, not the
/// source's, and is reported as an unexpected root file instead.
fn is_disabled(listing: &CanonicalListing) -> bool {
    let markers = [vtree::fold_name("NO_USE"), vtree::fold_name("NO_USE.txt")];
    listing.entries.iter().any(|entry| {
        matches!(entry.kind, ListedKind::File { .. })
            && !entry.path.contains('/')
            && markers.contains(&vtree::fold_name(&entry.path))
    })
}

#[cfg(test)]
mod tests {
    use aesthetics_export::ListedEntry;

    use super::*;
    use crate::testing::scratch;

    fn listing(files: &[&str]) -> CanonicalListing {
        CanonicalListing {
            display_name: "co".to_owned(),
            entries: files
                .iter()
                .map(|path| ListedEntry {
                    path: (*path).to_owned(),
                    kind: ListedKind::File { size: 0 },
                })
                .collect(),
        }
    }

    #[test]
    fn archive_extensions_are_case_insensitive() {
        assert_eq!(archive_kind(Path::new("a.zip")), Some(SourceKind::Zip));
        assert_eq!(archive_kind(Path::new("a.ZIP")), Some(SourceKind::Zip));
        assert_eq!(archive_kind(Path::new("a.7z")), Some(SourceKind::SevenZ));
        assert_eq!(archive_kind(Path::new("a.7Z")), Some(SourceKind::SevenZ));
        assert_eq!(archive_kind(Path::new("a.rar")), None);
        assert_eq!(archive_kind(Path::new("zip")), None);
        assert!(is_archive(Path::new("a.7z")));
        assert!(!is_archive(Path::new("a.rar")));
    }

    #[test]
    fn discovery_takes_folders_and_archives_in_folded_name_order() {
        let root = scratch("reader_discovery");
        for folder in ["b - Two", "A - One"] {
            fs::create_dir(root.join(folder)).unwrap();
        }
        for file in ["c - Three.ZIP", "D - Four.7z", "e.rar", "f.txt"] {
            fs::write(root.join(file), "").unwrap();
        }

        let sources = discover(&root, &[]).unwrap();

        let summary: Vec<(u64, &str, &str, SourceKind, Option<&str>)> = sources
            .iter()
            .map(|source| {
                (
                    source.export_id.0,
                    source.file_name.as_str(),
                    source.display_name.as_str(),
                    source.kind,
                    source.team_name.as_ref().map(TeamName::as_str),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (0, "A - One", "A - One", SourceKind::Folder, Some("/a/")),
                (1, "b - Two", "b - Two", SourceKind::Folder, Some("/b/")),
                (
                    2,
                    "c - Three.ZIP",
                    "c - Three",
                    SourceKind::Zip,
                    Some("/c/")
                ),
                (
                    3,
                    "D - Four.7z",
                    "D - Four",
                    SourceKind::SevenZ,
                    Some("/d/")
                ),
            ]
        );
        assert_eq!(sources[0].path, root.join("A - One"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn named_exports_replace_the_scan() {
        let root = scratch("reader_named");
        fs::create_dir(root.join("co - In root")).unwrap();
        fs::create_dir(root.join("zz - Named")).unwrap();
        fs::write(root.join("aa - Named.zip"), "").unwrap();

        let named = [root.join("zz - Named"), root.join("aa - Named.zip")];
        let sources = discover(Path::new("no such root"), &named).unwrap();

        let names: Vec<&str> = sources
            .iter()
            .map(|source| source.file_name.as_str())
            .collect();
        assert_eq!(names, ["aa - Named.zip", "zz - Named"]);
        assert_eq!(sources[1].export_id, ExportId(1));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn names_differing_only_in_case_take_the_same_order_whatever_order_they_come_in() {
        let root = scratch("reader_case_order");
        for folder in ["one/co - a", "two/co - A", "three/CO - A"] {
            fs::create_dir_all(root.join(folder)).unwrap();
        }
        let given = [
            root.join("one/co - a"),
            root.join("three/CO - A"),
            root.join("two/co - A"),
        ];
        let mut reversed = given.clone();
        reversed.reverse();

        for paths in [given, reversed] {
            let sources = discover(Path::new("no such root"), &paths).unwrap();
            let names: Vec<&str> = sources
                .iter()
                .map(|source| source.file_name.as_str())
                .collect();
            assert_eq!(names, ["CO - A", "co - A", "co - a"]);
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_named_path_that_is_no_longer_an_export_is_an_error_naming_it() {
        let root = scratch("reader_named_gone");
        let error = discover(Path::new("no such root"), &[root.join("gone.txt")]).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!(
                "--export {}: not a folder, .zip or .7z",
                root.join("gone.txt").display()
            )
        );
    }

    #[test]
    fn an_exports_root_that_cannot_be_read_is_an_error_naming_it() {
        let root = scratch("reader_missing").join("exports");
        let error = discover(&root, &[]).unwrap_err();
        assert!(
            format!("{error:#}").contains(&root.display().to_string()),
            "{error:#}"
        );
    }

    #[test]
    fn the_no_use_marker_counts_only_at_the_root_and_in_any_case() {
        for marker in ["NO_USE", "no_use", "NO_USE.txt", "No_Use.TXT"] {
            assert!(
                is_disabled(&listing(&[marker, "Players/03/face_high.fmdl"])),
                "{marker}"
            );
        }
        for not_marker in ["wrapper/NO_USE", "NO_USE.bak", "NOUSE"] {
            assert!(!is_disabled(&listing(&[not_marker])), "{not_marker}");
        }
        let mut folder = listing(&[]);
        folder.entries.push(ListedEntry {
            path: "NO_USE".to_owned(),
            kind: ListedKind::Folder,
        });
        assert!(!is_disabled(&folder), "a folder is not a marker");
    }

    #[test]
    fn routing_sets_aside_disabled_balls_and_conflicting_refs_exports() {
        let root = scratch("reader_routing");
        let write = |relative: &str| {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        };
        write("balls Off/NO_USE");
        write("balls On/ball.dds");
        write("co - Team/notes.txt");
        write("refs a/players.txt");
        write("refs b/players.txt");
        write("refs c/NO_USE.txt");
        write("refs d.zip");

        let sources = discover(&root, &[]).unwrap();
        let routes: Vec<String> = route(&sources, &MemoryBudget::new(1 << 20))
            .into_iter()
            .map(|route| match route {
                Route::Unreadable(failure) => format!("unreadable: {}", failure.error),
                Route::Disabled => "disabled".to_owned(),
                Route::Balls => "balls".to_owned(),
                Route::ConflictingRefs => "conflicting refs".to_owned(),
                Route::Validate { listing, .. } => format!("validate {}", listing.display_name),
            })
            .collect();

        assert_eq!(
            routes,
            [
                // Disabled comes before the balls rule.
                "disabled",
                "balls",
                "validate co - Team",
                "conflicting refs",
                "conflicting refs",
                // A disabled or unreadable refs export does not conflict.
                "disabled",
                "unreadable: zip: invalid Zip archive: Could not find EOCD",
            ]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_single_refs_export_is_validated() {
        let root = scratch("reader_single_refs");
        fs::create_dir_all(root.join("refs a")).unwrap();
        fs::write(root.join("refs a/players.txt"), "").unwrap();
        fs::create_dir_all(root.join("refs b")).unwrap();
        fs::write(root.join("refs b/NO_USE"), "").unwrap();
        let sources = discover(&root, &[]).unwrap();
        let routes = route(&sources, &MemoryBudget::new(1 << 20));
        assert!(matches!(routes[0], Route::Validate { .. }), "{routes:?}");
        assert_eq!(routes[1], Route::Disabled);
        fs::remove_dir_all(&root).unwrap();
    }
}
