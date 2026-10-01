//! The roster file: `players.txt`, or a referee export's `refs.txt` alias
//! when it has none — read from the supplied `SmallMetadata` by the raw
//! listing path.

use vtree::ScopePath;

use super::draft::{ExportKind, RawRoster, RawRosterEntry};
use super::{Canon, issue};
use crate::listing::SmallMetadata;
use crate::validate::{Disposition, IssueScope};

/// The roster bytes for the path the draft names: `Ok(bytes)` is read;
/// `Err(reason)` or no map entry is a `source_read_failed` (a roster is
/// required metadata → `DropExport`) and an empty roster.
fn read_bytes<'a>(
    canon: &'a Canon,
    path: &ScopePath,
    metadata: &'a SmallMetadata,
    issues: &mut Vec<crate::validate::ValidationIssue>,
) -> Option<&'a [u8]> {
    let file = canon.files.get(path)?;
    match metadata.files.get(&file.raw) {
        Some(Ok(bytes)) => Some(bytes.as_slice()),
        other => {
            let reason = match other {
                Some(Err(reason)) => reason.clone(),
                _ => "not read".to_owned(),
            };
            issues.push(issue(
                "source_read_failed",
                IssueScope::File(path.clone()),
                vec![("reason", reason)],
                Disposition::DropExport,
            ));
            None
        }
    }
}

/// One roster line: head is the slot when it is all ASCII digits and fits a
/// `u16`; the trimmed rest is the folder name when non-empty.
fn parse_line(line: usize, text: &str) -> RawRosterEntry {
    let trimmed = text.trim();
    let (head, rest) = trimmed
        .split_once(|c: char| c.is_whitespace())
        .unwrap_or((trimmed, ""));
    let slot = if !head.is_empty() && head.bytes().all(|b| b.is_ascii_digit()) {
        head.parse::<u16>().ok()
    } else {
        None
    };
    let folder_name = (!rest.trim().is_empty()).then(|| rest.trim().to_owned());
    RawRosterEntry {
        line,
        slot,
        folder_name,
    }
}

/// The roster text after optional-BOM stripping: strict UTF-8, LF and CRLF
/// accepted, blank lines ignored (they still count for line numbers).
fn parse_roster(
    path: &ScopePath,
    bytes: &[u8],
    issues: &mut Vec<crate::validate::ValidationIssue>,
) -> RawRoster {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let Ok(text) = str::from_utf8(bytes) else {
        issues.push(issue(
            "players_txt_invalid",
            IssueScope::File(path.clone()),
            vec![],
            Disposition::DropExport,
        ));
        return RawRoster {
            file: path.clone(),
            entries: Vec::new(),
        };
    };
    let entries = text
        .split('\n')
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_line(index + 1, line.trim_end_matches('\r')))
        .collect();
    RawRoster {
        file: path.clone(),
        entries,
    }
}

/// A root-level file of the given name, looked up fold-insensitively.
fn root_file(canon: &Canon, name: &str) -> Option<ScopePath> {
    canon
        .files
        .iter()
        .find(|(path, _)| {
            path.segments().count() == 1 && super::file_name(path).eq_ignore_ascii_case(name)
        })
        .map(|(path, _)| path.clone())
}

/// The roster, when the export has one: `players.txt` first, a referee
/// export's `refs.txt` alias second. A normal team's `refs.txt` is not a
/// roster.
pub fn read_roster(
    draft: &super::AestheticsExportDraft,
    canon: &Canon,
    metadata: &SmallMetadata,
    issues: &mut Vec<crate::validate::ValidationIssue>,
) -> Option<RawRoster> {
    let players = root_file(canon, "players.txt");
    let refs = root_file(canon, "refs.txt");
    let path = match (draft.kind(), players, refs) {
        (_, Some(path), alias) => {
            if draft.kind() == ExportKind::Referees
                && let Some(alias) = alias
            {
                issues.push(issue(
                    "refs_txt_ignored",
                    IssueScope::File(alias),
                    vec![],
                    Disposition::Keep,
                ));
            }
            path
        }
        (ExportKind::Referees, None, Some(path)) => path,
        _ => return None,
    };
    Some(match read_bytes(canon, &path, metadata, issues) {
        Some(bytes) => parse_roster(&path, bytes, issues),
        // A roster that could not be read leaves an empty roster.
        None => RawRoster {
            file: path,
            entries: Vec::new(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{listing, metadata};

    fn parsed(
        name: &str,
        files: &[(&str, u64)],
        metadata_items: &[(&str, Result<&[u8], &str>)],
    ) -> crate::ParsedAestheticsExport {
        crate::parse_listing(listing(name, files, &[]), metadata(metadata_items)).unwrap()
    }

    // TC-ROS-08
    #[test]
    fn bom_and_crlf_read_normally_and_non_utf8_is_invalid() {
        let good = parsed(
            "egg",
            &[("players.txt", 20)],
            &[("players.txt", Ok(b"\xef\xbb\xbf03 A\r\n05 B\r\n"))],
        );
        let roster = good.raw_roster.unwrap();
        assert_eq!(roster.entries.len(), 2);
        assert_eq!(roster.entries[0].slot, Some(3));
        assert_eq!(roster.entries[0].folder_name.as_deref(), Some("A"));

        let bad = parsed(
            "egg",
            &[("players.txt", 4)],
            &[("players.txt", Ok(&[0xff, 0xfe, 0xfd, 0x00][..]))],
        );
        let invalid = bad
            .issues
            .iter()
            .find(|issue| issue.code == "players_txt_invalid")
            .unwrap();
        assert_eq!(invalid.disposition, Disposition::DropExport);
        assert!(bad.raw_roster.unwrap().entries.is_empty());
    }

    #[test]
    fn line_shapes_parse_as_specified() {
        let text = b"x3 A\n24 B\n\n03\n07   C  \n99999 D\n+5 E";
        let parsed = parsed(
            "egg",
            &[("players.txt", 40)],
            &[("players.txt", Ok(&text[..]))],
        );
        let entries = parsed.raw_roster.unwrap().entries;
        assert_eq!(
            entries
                .iter()
                .map(|entry| (entry.line, entry.slot, entry.folder_name.as_deref()))
                .collect::<Vec<_>>(),
            vec![
                (1, None, Some("A")),
                (2, Some(24), Some("B")),
                (4, Some(3), None),
                (5, Some(7), Some("C")),
                (6, None, Some("D")),
                (7, None, Some("E")),
            ]
        );
    }

    // TC-ROS-11
    #[test]
    fn a_refs_export_reads_refs_txt_or_players_txt() {
        let only_refs = parsed(
            "refs Cup",
            &[("refs.txt", 10)],
            &[("refs.txt", Ok(b"01 Keeper"))],
        );
        let roster = only_refs.raw_roster.unwrap();
        assert_eq!(roster.file.as_str(), "refs.txt");
        assert_eq!(roster.entries[0].folder_name.as_deref(), Some("Keeper"));

        let both = parsed(
            "refs Cup",
            &[("refs.txt", 10), ("players.txt", 12)],
            &[("refs.txt", Ok(b"01 A")), ("players.txt", Ok(b"02 B"))],
        );
        let ignored = both
            .issues
            .iter()
            .find(|issue| issue.code == "refs_txt_ignored")
            .unwrap();
        assert_eq!(ignored.disposition, Disposition::Keep);
        assert_eq!(
            ignored.scope,
            IssueScope::File(ScopePath::new("refs.txt").unwrap())
        );
        let roster = both.raw_roster.unwrap();
        assert_eq!(roster.file.as_str(), "players.txt");
        assert_eq!(roster.entries[0].slot, Some(2));
    }

    #[test]
    fn a_normal_team_ignores_refs_txt() {
        let parsed = parsed("egg", &[("refs.txt", 10)], &[("refs.txt", Ok(b"01 A"))]);
        assert!(parsed.raw_roster.is_none());
    }

    #[test]
    fn a_failed_or_absent_roster_read_is_a_source_error() {
        let denied = parsed(
            "egg",
            &[("players.txt", 10)],
            &[("players.txt", Err("denied"))],
        );
        let failed = denied
            .issues
            .iter()
            .find(|issue| issue.code == "source_read_failed")
            .unwrap();
        assert_eq!(failed.disposition, Disposition::DropExport);
        assert_eq!(failed.context, vec![("reason", "denied".to_owned())]);

        let absent = parsed("egg", &[("players.txt", 10)], &[]);
        let failed = absent
            .issues
            .iter()
            .find(|issue| issue.code == "source_read_failed")
            .unwrap();
        assert_eq!(failed.context, vec![("reason", "not read".to_owned())]);
        assert!(absent.raw_roster.unwrap().entries.is_empty());
    }
}
