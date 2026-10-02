//! What the crate's unit tests share.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use aesthetics_export::{
    CanonicalListing, ListedEntry, ListedKind, ResolvedAestheticsExport, SmallMetadata,
    ValidationContext, parse_listing,
};
use pes_version::PesVersion;
use teams_list::TeamsList;

/// A fresh, empty `<temp>/team_compiler_<name>_<pid>` folder; `name` is unique per test.
pub(crate) fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("team_compiler_{name}_{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap();
    }
    fs::create_dir_all(&root).unwrap();
    root
}

/// The export `name` with these files (path, size), folders and `players.txt`, validated
/// for PES 21 and resolved against a teams list holding `701 /co/` and `702 /da/`; the export
/// must validate with no issue.
pub(crate) fn resolved(
    name: &str,
    files: &[(&str, u64)],
    folders: &[&str],
    players_txt: Option<&[u8]>,
) -> ResolvedAestheticsExport {
    let (resolved, issues) = resolved_with_issues(name, files, folders, players_txt);
    assert_eq!(issues, Vec::<&str>::new(), "a clean export");
    resolved
}

/// `resolved`'s export, with the codes of the issues validation reported on it.
pub(crate) fn resolved_with_issues(
    name: &str,
    files: &[(&str, u64)],
    folders: &[&str],
    players_txt: Option<&[u8]>,
) -> (ResolvedAestheticsExport, Vec<&'static str>) {
    let roster = players_txt.map(|bytes| ("players.txt", bytes.len() as u64));
    let files = files
        .iter()
        .copied()
        .chain(roster)
        .map(|(path, size)| (path.to_owned(), ListedKind::File { size }));
    let folders = folders
        .iter()
        .map(|path| ((*path).to_owned(), ListedKind::Folder));
    let listing = CanonicalListing {
        display_name: name.to_owned(),
        entries: files
            .chain(folders)
            .map(|(path, kind)| ListedEntry { path, kind })
            .collect(),
    };
    let metadata = SmallMetadata {
        files: players_txt
            .map(|bytes| ("players.txt".to_owned(), Ok(bytes.to_vec())))
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
    };
    let report = parse_listing(listing, metadata)
        .unwrap()
        .validate(&ValidationContext {
            version: PesVersion::Pes21,
            strict_file_type_check: true,
            pass_through: false,
        });
    let issues = report.issues.iter().map(|issue| issue.code).collect();
    let teams = TeamsList::parse("ID\tName\n701\t/co/\n702\t/da/\n").unwrap();
    let resolved = report.validated.unwrap().resolve_identity(&teams).unwrap();
    (resolved, issues)
}
