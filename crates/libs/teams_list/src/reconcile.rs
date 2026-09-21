//! Merging an incoming list into the working list.
//!
//! Every incoming team produces one change: a working team row with the same
//! name takes the incoming id (the row stays in place, its other cells kept),
//! a name absent from the working teams replaces a working placeholder that
//! holds the incoming id (the placeholder's other cells survive) or is
//! appended at the end. An incoming placeholder is appended too when no
//! working row claims its numeric id and no working placeholder shows the
//! same raw `Name` cell; it loses to any other claimant of its id.
//! Uniqueness is then validated on the merged mapping: while any id is
//! claimed by more than one row — team or placeholder — the accepted
//! changes behind the collision are moved to `unresolved` (placeholder
//! appends are simply dropped) and the rows recomputed.

use std::collections::BTreeSet;

use crate::file::{Row, TeamsList};
use crate::id::TeamId;
use crate::name::TeamName;

/// One incoming change, computed once against the working list.
struct Change {
    /// Where the change lands and the cells it writes.
    kind: ChangeKind,
    /// Working rows the change frees (a placeholder that claimed the
    /// incoming id and is neither the rewritten row nor an append).
    removed: Vec<usize>,
    /// The id the incoming row assigns.
    id: TeamId,
    /// The incoming row's folded name.
    name: TeamName,
}

enum ChangeKind {
    /// Rewrites working row `row`: an override of a team row or a taken
    /// placeholder slot.
    Rewrite { row: usize, cells: Vec<String> },
    /// Appended after the working rows.
    Append { cells: Vec<String> },
}

/// An incoming placeholder carried over verbatim, appended after the team
/// appends.
struct PlaceholderAppend {
    /// Its cells mapped to the working column layout.
    cells: Vec<String>,
    /// Its raw `Name` cell, for `placeholders_added` reporting.
    raw_name: String,
    /// Its numeric claim, when the `ID` cell parses in range.
    id: Option<TeamId>,
}

/// The id a row claims: a team's validated id, or a placeholder's numeric
/// `ID` cell in the valid range.
fn claimed_id(row: &Row, id_column: usize) -> Option<TeamId> {
    match row {
        Row::Team { id, .. } => Some(*id),
        Row::Placeholder { cells } => cells
            .get(id_column)?
            .parse::<u16>()
            .ok()
            .and_then(|value| TeamId::new(value).ok()),
    }
}

/// The cells of an appended incoming row in the working list's column
/// layout: copied verbatim when the headers match, mapped by header label
/// when they differ (a label the incoming header does not have stays
/// blank).
fn appended_cells(working: &TeamsList, incoming: &TeamsList, cells: &[String]) -> Vec<String> {
    let width = working.header().len();
    if incoming.header() == working.header() {
        let mut cells = cells.to_vec();
        cells.resize(width.max(cells.len()), String::new());
        return cells;
    }
    working
        .header()
        .iter()
        .map(|label| {
            incoming
                .header()
                .iter()
                .position(|other| other == label)
                .and_then(|index| cells.get(index))
                .cloned()
                .unwrap_or_default()
        })
        .collect()
}

/// The merged rows: the working list with every accepted change applied.
fn merged_rows(
    working: &TeamsList,
    changes: &[Change],
    placeholder_appends: &[PlaceholderAppend],
) -> Vec<Row> {
    let mut rows = working.rows().to_vec();
    let mut removed = BTreeSet::new();
    let mut appended = Vec::new();
    for change in changes {
        removed.extend(change.removed.iter().copied());
        match &change.kind {
            ChangeKind::Rewrite { row, cells } => {
                rows[*row] = Row::Team {
                    cells: cells.clone(),
                    id: change.id,
                    name: change.name.clone(),
                };
            }
            ChangeKind::Append { cells } => appended.push(Row::Team {
                cells: cells.clone(),
                id: change.id,
                name: change.name.clone(),
            }),
        }
    }
    // Removed rows index into the working prefix, before any append.
    let mut kept: Vec<Row> = rows
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !removed.contains(index))
        .map(|(_, row)| row)
        .collect();
    kept.extend(appended);
    kept.extend(placeholder_appends.iter().map(|append| Row::Placeholder {
        cells: append.cells.clone(),
    }));
    kept
}

/// Merges `incoming` into `working`, returning the merged list and a summary
/// for the caller to show before writing.
pub fn reconcile(working: &TeamsList, incoming: &TeamsList) -> (TeamsList, MergeSummary) {
    let working_team_count = working.teams().count();
    let id_column = working.id_column();
    let name_column = working.name_column();
    let mut changes: Vec<Change> = Vec::new();
    let mut placeholder_appends: Vec<PlaceholderAppend> = Vec::new();

    // The match never depends on other changes: names are unique within each
    // list and a placeholder claim is read off the working rows.
    for incoming_row in incoming.rows() {
        if let Row::Placeholder { cells } = incoming_row {
            // Carried over when its numeric claim is free in the working
            // list and no placeholder already shows the same raw Name.
            let claim = claimed_id(incoming_row, incoming.id_column());
            let raw_name = cells
                .get(incoming.name_column())
                .cloned()
                .unwrap_or_default();
            let claimed = claim.is_some_and(|id| {
                working
                    .rows()
                    .iter()
                    .any(|row| claimed_id(row, id_column) == Some(id))
            });
            let named = working.rows().iter().any(|row| {
                matches!(row, Row::Placeholder { cells: other }
                    if other.get(name_column) == Some(&raw_name))
            }) || placeholder_appends
                .iter()
                .any(|append| append.cells.get(name_column) == Some(&raw_name));
            if !claimed && !named {
                placeholder_appends.push(PlaceholderAppend {
                    cells: appended_cells(working, incoming, cells),
                    raw_name,
                    id: claim,
                });
            }
            continue;
        }

        let Row::Team {
            cells: incoming_cells,
            id: incoming_id,
            name: incoming_name,
        } = incoming_row
        else {
            continue;
        };

        if let Some(position) = working
            .rows()
            .iter()
            .position(|row| matches!(row, Row::Team { name, .. } if name == incoming_name))
        {
            let Row::Team {
                cells, id: old_id, ..
            } = &working.rows()[position]
            else {
                continue;
            };
            if old_id == incoming_id {
                continue;
            }
            let mut cells = cells.clone();
            cells[id_column] = incoming_id.get().to_string();
            // A working placeholder may already claim the new id; taking it
            // over frees the placeholder's row. If the change is later
            // reverted, the removal goes with it.
            let removed = working
                .rows()
                .iter()
                .position(|row| {
                    matches!(row, Row::Placeholder { .. })
                        && claimed_id(row, id_column) == Some(*incoming_id)
                })
                .into_iter()
                .collect();
            changes.push(Change {
                kind: ChangeKind::Rewrite {
                    row: position,
                    cells,
                },
                removed,
                id: *incoming_id,
                name: incoming_name.clone(),
            });
            continue;
        }

        let holding_placeholder = working.rows().iter().position(|row| {
            matches!(row, Row::Placeholder { .. })
                && claimed_id(row, id_column) == Some(*incoming_id)
        });
        match holding_placeholder {
            Some(position) => {
                let Row::Placeholder { cells: before } = &working.rows()[position] else {
                    continue;
                };
                let incoming_raw_name = incoming_cells
                    .get(incoming.name_column())
                    .cloned()
                    .unwrap_or_default();
                let mut cells = before.clone();
                // `cells` holds at least `id_column + 1` cells: the placeholder
                // claimed this id, which its `ID` cell must carry.
                cells.resize(cells.len().max(name_column + 1), String::new());
                cells[id_column] = incoming_id.get().to_string();
                cells[name_column] = incoming_raw_name;
                changes.push(Change {
                    kind: ChangeKind::Rewrite {
                        row: position,
                        cells,
                    },
                    removed: Vec::new(),
                    id: *incoming_id,
                    name: incoming_name.clone(),
                });
            }
            None => changes.push(Change {
                kind: ChangeKind::Append {
                    cells: appended_cells(working, incoming, incoming_cells),
                },
                removed: Vec::new(),
                id: *incoming_id,
                name: incoming_name.clone(),
            }),
        }
    }

    // While an id is claimed by more than one row — team or placeholder —
    // drop the accepted changes behind the collision. A placeholder append
    // loses first: its id was unclaimed in the working list, so a duplicate
    // always comes from an incoming team, which wins like a slot take.
    // Every remaining duplicate involves a change (the working list's own
    // claims are unique), so each pass moves at least one and the loop
    // terminates.
    let mut unresolved = Vec::new();
    loop {
        let mut seen = BTreeSet::new();
        let mut duplicated = BTreeSet::new();
        for row in merged_rows(working, &changes, &placeholder_appends) {
            if let Some(id) = claimed_id(&row, id_column)
                && !seen.insert(id)
            {
                duplicated.insert(id);
            }
        }
        if duplicated.is_empty() {
            break;
        }
        let append_claims: BTreeSet<TeamId> = placeholder_appends
            .iter()
            .filter_map(|append| append.id)
            .collect();
        let mut dropped = false;
        placeholder_appends.retain(|append| match append.id {
            Some(id) if duplicated.contains(&id) => {
                dropped = true;
                false
            }
            _ => true,
        });
        changes.retain(|change| {
            if duplicated.contains(&change.id) && !append_claims.contains(&change.id) {
                unresolved.push((change.id, change.name.clone()));
                dropped = true;
                false
            } else {
                true
            }
        });
        if !dropped {
            break;
        }
    }

    let mut added = Vec::new();
    let mut overridden = Vec::new();
    for change in &changes {
        match &change.kind {
            ChangeKind::Rewrite { row, .. } => match &working.rows()[*row] {
                Row::Team { id: old_id, .. } => {
                    overridden.push((*row, (change.name.clone(), *old_id, change.id)))
                }
                Row::Placeholder { .. } => added.push((change.id, change.name.clone())),
            },
            ChangeKind::Append { .. } => added.push((change.id, change.name.clone())),
        }
    }
    // `added` is in the order the changes were computed; `overridden` is
    // reported in working row order.
    overridden.sort_by_key(|(row, _)| *row);
    let surviving_overrides = overridden.len();
    let summary = MergeSummary {
        added,
        kept: working_team_count - surviving_overrides,
        overridden: overridden.into_iter().map(|(_, entry)| entry).collect(),
        unresolved,
        placeholders_added: placeholder_appends
            .iter()
            .map(|append| append.raw_name.clone())
            .collect(),
    };

    let merged = TeamsList::from_parts(
        working.header().to_vec(),
        working.id_column(),
        working.name_column(),
        merged_rows(working, &changes, &placeholder_appends),
    );
    (merged, summary)
}

/// What a merge did, for the caller to report before writing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MergeSummary {
    /// Incoming rows appended at the end of the working list or put into a
    /// placeholder's slot.
    pub added: Vec<(TeamId, TeamName)>,
    /// Working team rows that ended unchanged (working-only rows and rows
    /// identical in both lists).
    pub kept: usize,
    /// Names present in both whose id changed to the incoming one.
    pub overridden: Vec<(TeamName, TeamId, TeamId)>,
    /// Incoming rows left out because their id collided with a different
    /// existing row after every change was applied.
    pub unresolved: Vec<(TeamId, TeamName)>,
    /// Raw `Name` cells of incoming placeholders carried over verbatim, in
    /// the order they were appended.
    pub placeholders_added: Vec<String>,
}
