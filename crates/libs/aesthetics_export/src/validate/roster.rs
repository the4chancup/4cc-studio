//! The roster's structural rules: assignment validity, numbering without a
//! roster file, and the sanitized slot→folder map ("Validation semantics" →
//! "Roster entries", `player_folders.md` "Player numbering").

use std::collections::BTreeMap;

use crate::conventions::split_folder_name;
use crate::parse::{AestheticsExportDraft, ExportKind, RawRoster};
use crate::slots::{PlayerSlot, RefSlot};
use crate::validate::{Disposition, IssueScope, ValidationIssue, issue};

/// An index into `ValidatedAestheticsExport.players`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerIndex(pub usize);

/// The normalized roster: strong slots to indices into the final `players`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedRoster {
    /// A normal team's assignments.
    Team(BTreeMap<PlayerSlot, PlayerIndex>),
    /// A referee export's assignments; several slots may share a folder.
    Referees(BTreeMap<RefSlot, PlayerIndex>),
}

/// The surviving slot→folder assignments, as indices into the draft's
/// `players`, with whichever slot type the export's kind uses.
pub(crate) enum SlotMap {
    /// Team slots.
    Team(BTreeMap<PlayerSlot, usize>),
    /// Referee slots.
    Referees(BTreeMap<RefSlot, usize>),
}

impl SlotMap {
    /// The draft `players` indices a surviving assignment maps.
    pub(crate) fn mapped(&self) -> Vec<usize> {
        match self {
            SlotMap::Team(map) => map.values().copied().collect(),
            SlotMap::Referees(map) => map.values().copied().collect(),
        }
    }
}

/// A folder name's fold key for lookup.
fn folder_key(name: &str) -> String {
    vtree::fold_name(name)
}

/// The raw slot's strong type by the export kind; `None` when out of range
/// (`u8` range included: a `u16` above 255 is out).
fn slot_in_range(slot: u16, referees: bool) -> Option<u8> {
    let slot = u8::try_from(slot).ok()?;
    let in_range = if referees {
        RefSlot::new(slot).is_some()
    } else {
        PlayerSlot::new(slot).is_some()
    };
    in_range.then_some(slot)
}

/// Validates the roster against the draft's player folders and returns the
/// surviving assignments. With a roster file the lines are authoritative;
/// without one a team's folder names carry the numbers.
pub(crate) fn check(
    draft: &AestheticsExportDraft,
    raw_roster: Option<&RawRoster>,
    parsed_issues: &[ValidationIssue],
    issues: &mut Vec<ValidationIssue>,
) -> SlotMap {
    match raw_roster {
        Some(roster) => with_roster(draft, roster, parsed_issues, issues),
        None => without_roster(draft, issues),
    }
}

/// `players.txt` (or the `refs.txt` alias) rules. Malformed lines keep their
/// own findings; duplicates are found before any folder is looked up.
fn with_roster(
    draft: &AestheticsExportDraft,
    roster: &RawRoster,
    parsed_issues: &[ValidationIssue],
    issues: &mut Vec<ValidationIssue>,
) -> SlotMap {
    let referees = draft.kind() == ExportKind::Referees;
    // The parse already dropped the file: nothing to add (parse emits these
    // two codes only for the roster file).
    if parsed_issues
        .iter()
        .any(|issue| matches!(issue.code, "players_txt_invalid" | "source_read_failed"))
    {
        return empty_map(referees);
    }
    let file = &roster.file;
    let scope = |entry: &crate::parse::RawRosterEntry| IssueScope::RosterEntry {
        file: file.clone(),
        line: entry.line,
        slot: entry.slot,
    };

    // A complete line carries a slot and a folder name; the fold-keyed last
    // segment is the lookup, and naming a folder counts even when the line
    // drops (the folder is "listed").
    let complete: Vec<&crate::parse::RawRosterEntry> = roster
        .entries
        .iter()
        .filter(|entry| entry.slot.is_some() && entry.folder_name.is_some())
        .collect();

    // Duplicate slots are detected first, over in-range complete lines, before
    // any target is looked up; the findings land in line order below.
    let mut claimed: BTreeMap<u16, usize> = BTreeMap::new();
    let mut duplicated = Vec::new();
    for entry in &complete {
        let Some(slot) = entry.slot else { continue };
        if slot_in_range(slot, referees).is_none() {
            continue;
        }
        if claimed.insert(slot, entry.line).is_some() {
            duplicated.push(entry.line);
        }
    }

    // Then each line's own finding, in file order; a fold-keyed name is the
    // folder lookup (vtree already refuses same-fold different-spelling
    // folders, so a fold match is unambiguous).
    let folders: BTreeMap<String, usize> = draft
        .players
        .iter()
        .enumerate()
        .map(|(index, folder)| (folder_key(folder.path.name()), index))
        .collect();
    let mut assignments = BTreeMap::new();
    for entry in &roster.entries {
        let (Some(slot), Some(folder_name)) = (entry.slot, entry.folder_name.as_deref()) else {
            issues.push(issue(
                "players_txt_line_invalid",
                scope(entry),
                vec![],
                Disposition::DropSlot,
            ));
            continue;
        };
        if duplicated.contains(&entry.line) {
            issues.push(issue(
                "players_txt_slot_duplicate",
                scope(entry),
                vec![],
                Disposition::DropExport,
            ));
            continue;
        }
        let Some(slot) = slot_in_range(slot, referees) else {
            issues.push(issue(
                "players_txt_slot_invalid",
                scope(entry),
                vec![],
                Disposition::DropSlot,
            ));
            continue;
        };
        let name_key = vtree::fold_name(folder_name);
        let Some(&index) = folders.get(&name_key) else {
            issues.push(issue(
                "players_txt_target_missing",
                scope(entry),
                vec![("folder", folder_name.to_owned())],
                Disposition::DropSlot,
            ));
            continue;
        };
        assignments.insert(slot, index);
    }

    // A folder no complete line names is unlisted.
    let named: std::collections::BTreeSet<String> = complete
        .iter()
        .filter_map(|entry| entry.folder_name.as_deref())
        .map(vtree::fold_name)
        .collect();
    for folder in &draft.players {
        if !named.contains(&folder_key(folder.path.name())) {
            issues.push(issue(
                "player_unlisted",
                IssueScope::Folder(folder.path.clone()),
                vec![],
                Disposition::DropFolder,
            ));
        }
    }

    // A referee roster needs at least one surviving assignment; an empty team
    // roster is valid (a kit-only export).
    if referees && assignments.is_empty() {
        issues.push(issue(
            "players_txt_invalid",
            IssueScope::File(file.clone()),
            vec![],
            Disposition::DropExport,
        ));
    }
    into_map(referees, assignments)
}

/// No roster file: a referee export is dropped; a team's folder names carry
/// `NN - label` numbers.
fn without_roster(draft: &AestheticsExportDraft, issues: &mut Vec<ValidationIssue>) -> SlotMap {
    if draft.kind() == ExportKind::Referees {
        issues.push(issue(
            "players_txt_missing",
            IssueScope::Export,
            vec![],
            Disposition::DropExport,
        ));
        return SlotMap::Referees(BTreeMap::new());
    }
    // Every folder name's head claims a slot; a claim two folders share drops
    // both, an invalid head drops its own.
    let mut claims: BTreeMap<PlayerSlot, Vec<usize>> = BTreeMap::new();
    for (index, folder) in draft.players.iter().enumerate() {
        let (head, _) = split_folder_name(folder.path.name());
        match head
            .bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| head.parse::<u8>().ok())
            .flatten()
            .and_then(PlayerSlot::new)
        {
            Some(slot) => claims.entry(slot).or_default().push(index),
            None => issues.push(issue(
                "player_folder_number_invalid",
                IssueScope::Folder(folder.path.clone()),
                vec![],
                Disposition::DropFolder,
            )),
        }
    }
    let mut assignments = BTreeMap::new();
    for (slot, indices) in claims {
        if indices.len() > 1 {
            for index in indices {
                issues.push(issue(
                    "player_number_duplicate",
                    IssueScope::Folder(draft.players[index].path.clone()),
                    vec![],
                    Disposition::DropFolder,
                ));
            }
            continue;
        }
        assignments.insert(slot, indices[0]);
    }
    SlotMap::Team(assignments)
}

fn empty_map(referees: bool) -> SlotMap {
    if referees {
        SlotMap::Referees(BTreeMap::new())
    } else {
        SlotMap::Team(BTreeMap::new())
    }
}

/// Fold the `u8` keys into their strong slot type (a surviving assignment's
/// slot is always in range).
fn into_map(referees: bool, assignments: BTreeMap<u8, usize>) -> SlotMap {
    if referees {
        SlotMap::Referees(
            assignments
                .into_iter()
                .map(|(slot, index)| {
                    (
                        RefSlot::new(slot).expect("a surviving assignment's slot is in range"),
                        index,
                    )
                })
                .collect(),
        )
    } else {
        SlotMap::Team(
            assignments
                .into_iter()
                .map(|(slot, index)| {
                    (
                        PlayerSlot::new(slot).expect("a surviving assignment's slot is in range"),
                        index,
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vtree::ScopePath;

    use crate::Disposition;
    use crate::testing::report;
    use crate::validate::{IssueScope, ValidationReport};

    fn scope_folder(path: &str) -> IssueScope {
        IssueScope::Folder(ScopePath::new(path).unwrap())
    }

    fn roster_scope(file: &str, line: usize, slot: Option<u16>) -> IssueScope {
        IssueScope::RosterEntry {
            file: ScopePath::new(file).unwrap(),
            line,
            slot,
        }
    }

    fn issues_of(report: &ValidationReport) -> Vec<(&'static str, IssueScope, Disposition)> {
        report
            .issues
            .iter()
            .map(|issue| (issue.code, issue.scope.clone(), issue.disposition))
            .collect()
    }

    #[test]
    fn numbered_folder_names_map_the_roster() {
        let report = report(
            "egg",
            &[
                ("Players/03 - A/face_high.fmdl", 10),
                ("Players/15 - B/face_high.fmdl", 10),
            ],
            &[],
            &[],
        );
        assert_eq!(issues_of(&report), vec![]);
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 2);
        let ValidatedRoster::Team(map) = &validated.roster else {
            panic!("a team export");
        };
        assert_eq!(
            map.keys().copied().map(PlayerSlot::get).collect::<Vec<_>>(),
            vec![3, 15]
        );
    }

    // TC-ROS-02
    #[test]
    fn an_unnumbered_or_out_of_range_folder_name_is_dropped() {
        let report = report("egg", &[], &["Players/24 - C", "Players/Snuffy"], &[]);
        assert_eq!(
            issues_of(&report),
            vec![
                (
                    "player_folder_number_invalid",
                    scope_folder("Players/24 - C"),
                    Disposition::DropFolder,
                ),
                (
                    "player_folder_number_invalid",
                    scope_folder("Players/Snuffy"),
                    Disposition::DropFolder,
                ),
            ]
        );
        let validated = report.validated.unwrap();
        assert!(validated.players.is_empty());
    }

    // TC-ROS-03
    #[test]
    fn two_folders_claiming_one_slot_are_both_dropped() {
        let report = report("egg", &[], &["Players/05 - A", "Players/05 - B"], &[]);
        assert_eq!(
            issues_of(&report),
            vec![
                (
                    "player_number_duplicate",
                    scope_folder("Players/05 - A"),
                    Disposition::DropFolder,
                ),
                (
                    "player_number_duplicate",
                    scope_folder("Players/05 - B"),
                    Disposition::DropFolder,
                ),
            ]
        );
        let validated = report.validated.unwrap();
        assert!(validated.players.is_empty());
    }

    #[test]
    fn a_roster_lists_folders_and_unlisted_ones_drop() {
        let report = report(
            "egg",
            &[("players.txt", 10)],
            &["Players/Snuffy", "Players/15 - B"],
            &[("players.txt", Ok(b"03 snuffy"))],
        );
        assert_eq!(
            issues_of(&report),
            vec![(
                "player_unlisted",
                scope_folder("Players/15 - B"),
                Disposition::DropFolder,
            )]
        );
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        assert_eq!(validated.players[0].player_name, "Snuffy");
        let ValidatedRoster::Team(map) = &validated.roster else {
            panic!("a team export");
        };
        assert_eq!(map[&PlayerSlot::new(3).unwrap()], PlayerIndex(0));
    }

    #[test]
    fn one_folder_can_fill_several_slots() {
        let report = report(
            "egg",
            &[("players.txt", 20)],
            &["Players/Keeper"],
            &[("players.txt", Ok(b"03 Keeper\n07 Keeper"))],
        );
        assert_eq!(issues_of(&report), vec![]);
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        let ValidatedRoster::Team(map) = &validated.roster else {
            panic!("a team export");
        };
        assert_eq!(
            map.keys().copied().map(PlayerSlot::get).collect::<Vec<_>>(),
            vec![3, 7]
        );
        assert_eq!(map[&PlayerSlot::new(3).unwrap()], PlayerIndex(0));
        assert_eq!(map[&PlayerSlot::new(7).unwrap()], PlayerIndex(0));
    }

    #[test]
    fn malformed_lines_drop_but_the_good_lines_stand() {
        let report = report(
            "egg",
            &[("players.txt", 40)],
            &["Players/A", "Players/B", "Players/C"],
            &[("players.txt", Ok(b"x3 A\n24 B\n05 Nobody\n07 C"))],
        );
        assert_eq!(
            issues_of(&report),
            vec![
                (
                    "players_txt_line_invalid",
                    roster_scope("players.txt", 1, None),
                    Disposition::DropSlot,
                ),
                (
                    "players_txt_slot_invalid",
                    roster_scope("players.txt", 2, Some(24)),
                    Disposition::DropSlot,
                ),
                (
                    "players_txt_target_missing",
                    roster_scope("players.txt", 3, Some(5)),
                    Disposition::DropSlot,
                ),
                (
                    "player_unlisted",
                    scope_folder("Players/A"),
                    Disposition::DropFolder,
                ),
            ]
        );
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        let ValidatedRoster::Team(map) = &validated.roster else {
            panic!("a team export");
        };
        assert_eq!(
            map.keys().copied().map(PlayerSlot::get).collect::<Vec<_>>(),
            vec![7]
        );
    }

    // TC-ROS-07
    #[test]
    fn a_duplicate_slot_reports_dup_and_the_first_lines_missing_target() {
        let report = report(
            "egg",
            &[("players.txt", 30)],
            &["Players/NewName"],
            &[("players.txt", Ok(b"03 OldName\n03 NewName"))],
        );
        assert_eq!(
            issues_of(&report),
            vec![
                (
                    "players_txt_target_missing",
                    roster_scope("players.txt", 1, Some(3)),
                    Disposition::DropSlot,
                ),
                (
                    "players_txt_slot_duplicate",
                    roster_scope("players.txt", 2, Some(3)),
                    Disposition::DropExport,
                ),
            ]
        );
        assert!(report.validated.is_none());
    }

    #[test]
    fn an_empty_team_roster_is_valid_and_every_folder_is_unlisted() {
        let report = report(
            "egg",
            &[("players.txt", 0), ("Kits/p1/kit.dds", 9)],
            &["Players/03 - A"],
            &[("players.txt", Ok(b""))],
        );
        assert_eq!(
            issues_of(&report),
            vec![(
                "player_unlisted",
                scope_folder("Players/03 - A"),
                Disposition::DropFolder,
            )]
        );
        assert!(report.validated.is_some());
    }

    // TC-ROS-10
    #[test]
    fn a_refs_export_without_a_roster_is_dropped() {
        let report = report("refs Cup", &[], &["Players/03 - A"], &[]);
        assert_eq!(
            issues_of(&report),
            vec![(
                "players_txt_missing",
                IssueScope::Export,
                Disposition::DropExport,
            )]
        );
        assert!(report.validated.is_none());
    }

    // TC-ROS-12
    #[test]
    fn a_refs_roster_maps_one_folder_to_several_ref_slots() {
        let report = report(
            "refs Cup",
            &[("players.txt", 40)],
            &["Players/Keeper"],
            &[("players.txt", Ok(b"01 Keeper\n20 Keeper\n35 Keeper"))],
        );
        assert_eq!(issues_of(&report), vec![]);
        let validated = report.validated.unwrap();
        assert_eq!(validated.players.len(), 1);
        let ValidatedRoster::Referees(map) = &validated.roster else {
            panic!("a refs export");
        };
        assert_eq!(
            map.keys().copied().map(RefSlot::get).collect::<Vec<_>>(),
            vec![1, 20, 35]
        );
        assert_eq!(map[&RefSlot::new(1).unwrap()], PlayerIndex(0));
        assert_eq!(map[&RefSlot::new(35).unwrap()], PlayerIndex(0));
    }

    // TC-ROS-12
    #[test]
    fn a_refs_roster_left_empty_is_dropped() {
        let report = report(
            "refs Cup",
            &[("players.txt", 10)],
            &["Players/Keeper"],
            &[("players.txt", Ok(b"x3 Keeper\n36 Keeper"))],
        );
        // Line 2 is complete and names Keeper, dropped slot or not, so the
        // folder is listed; both assignments drop and the refs roster is
        // left empty.
        assert_eq!(
            issues_of(&report),
            vec![
                (
                    "players_txt_line_invalid",
                    roster_scope("players.txt", 1, None),
                    Disposition::DropSlot,
                ),
                (
                    "players_txt_slot_invalid",
                    roster_scope("players.txt", 2, Some(36)),
                    Disposition::DropSlot,
                ),
                (
                    "players_txt_invalid",
                    IssueScope::File(ScopePath::new("players.txt").unwrap()),
                    Disposition::DropExport,
                ),
            ]
        );
        assert!(report.validated.is_none());
    }

    #[test]
    fn an_unreadable_roster_reports_only_the_parse_issue() {
        // Parse's `players_txt_invalid` already drops the export; validate
        // adds nothing on top (no `player_unlisted` for the folder).
        let report = report(
            "egg",
            &[("players.txt", 7)],
            &["Players/A"],
            &[("players.txt", Ok(&b"\xff\xfe03 A"[..]))],
        );
        assert_eq!(
            issues_of(&report),
            vec![(
                "players_txt_invalid",
                IssueScope::File(ScopePath::new("players.txt").unwrap()),
                Disposition::DropExport,
            )]
        );
        assert!(report.validated.is_none());
    }

    #[test]
    fn player_name_takes_the_label_or_the_whole_name() {
        let without_roster = report(
            "egg",
            &[("Players/03 - Jean-Pierre/face_high.fmdl", 10)],
            &[],
            &[],
        );
        assert_eq!(
            without_roster.validated.unwrap().players[0].player_name,
            "Jean-Pierre"
        );
        // With a roster file the whole folder name is the player name.
        let with_roster = report(
            "egg",
            &[("players.txt", 10)],
            &["Players/Snuffy"],
            &[("players.txt", Ok(b"03 Snuffy"))],
        );
        assert_eq!(
            with_roster.validated.unwrap().players[0].player_name,
            "Snuffy"
        );
    }
}
