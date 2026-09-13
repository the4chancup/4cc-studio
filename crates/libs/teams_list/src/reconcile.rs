//! Merging an incoming list into the working list.
//!
//! Every incoming team is applied first: a working team row with the same
//! name takes the incoming id (the row stays in place, its other cells kept),
//! a name absent from the working teams replaces a working placeholder that
//! holds the incoming id (the placeholder's other cells survive) or is
//! appended at the end. Uniqueness is then validated on the final mapping:
//! while any id is held by more than one team row, the incoming changes that
//! produced the collision are reverted and reported as `unresolved`.
//! Placeholders in `incoming` are ignored.

use std::collections::{BTreeMap, BTreeSet};

use crate::file::{Row, TeamsList};
use crate::id::TeamId;
use crate::name::TeamName;

/// One applied incoming change, so it can be reverted if its id collides.
struct Change {
    /// Index into the merged row vector.
    row: usize,
    /// The id the incoming row assigned.
    id: TeamId,
    /// The incoming row's folded name.
    name: TeamName,
    /// How to undo the change: `Some` restores the row that stood there,
    /// `None` removes a row that was appended.
    before: Option<Row>,
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
/// layout: copied verbatim when the headers match, blank except the id and
/// name cells when they do not.
fn appended_cells(working: &TeamsList, incoming: &TeamsList, cells: &[String]) -> Vec<String> {
    let width = working.header().len();
    if incoming.header() == working.header() {
        let mut cells = cells.to_vec();
        cells.resize(width.max(cells.len()), String::new());
        return cells;
    }
    let mut out = vec![String::new(); width];
    if working.id_column() < width {
        out[working.id_column()] = cells.get(incoming.id_column()).cloned().unwrap_or_default();
    }
    if working.name_column() < width {
        out[working.name_column()] = cells
            .get(incoming.name_column())
            .cloned()
            .unwrap_or_default();
    }
    out
}

/// Merges `incoming` into `working`, returning the merged list and a summary
/// for the caller to show before writing.
pub fn reconcile(working: &TeamsList, incoming: &TeamsList) -> (TeamsList, MergeSummary) {
    let mut rows = working.rows().to_vec();
    let working_team_count = working.teams().count();
    let id_column = working.id_column();
    let name_column = working.name_column();
    let mut changes: Vec<Change> = Vec::new();

    for incoming_row in incoming.rows() {
        let Row::Team {
            cells: incoming_cells,
            id: incoming_id,
            name: incoming_name,
        } = incoming_row
        else {
            continue;
        };
        let incoming_raw_name = incoming_cells
            .get(incoming.name_column())
            .cloned()
            .unwrap_or_default();

        let same_name = rows
            .iter()
            .position(|row| matches!(row, Row::Team { name, .. } if name == incoming_name));
        if let Some(position) = same_name {
            let Row::Team { id: old_id, .. } = &rows[position] else {
                continue;
            };
            if *old_id == *incoming_id {
                continue;
            }
            let mut updated = rows[position].clone();
            if let Row::Team { id, cells, .. } = &mut updated {
                *id = *incoming_id;
                cells[id_column] = incoming_id.get().to_string();
            }
            changes.push(Change {
                row: position,
                id: *incoming_id,
                name: incoming_name.clone(),
                before: Some(std::mem::replace(&mut rows[position], updated)),
            });
            continue;
        }

        let holding_placeholder = rows.iter().position(|row| {
            matches!(row, Row::Placeholder { .. })
                && claimed_id(row, id_column) == Some(*incoming_id)
        });
        match holding_placeholder {
            Some(position) => {
                let Row::Placeholder { cells: before } = &rows[position] else {
                    continue;
                };
                let mut slot_cells = before.clone();
                slot_cells.resize(
                    slot_cells.len().max(id_column + 1).max(name_column + 1),
                    String::new(),
                );
                slot_cells[id_column] = incoming_id.get().to_string();
                slot_cells[name_column] = incoming_raw_name;
                changes.push(Change {
                    row: position,
                    id: *incoming_id,
                    name: incoming_name.clone(),
                    before: Some(std::mem::replace(
                        &mut rows[position],
                        Row::Team {
                            cells: slot_cells,
                            id: *incoming_id,
                            name: incoming_name.clone(),
                        },
                    )),
                });
            }
            None => {
                changes.push(Change {
                    row: rows.len(),
                    id: *incoming_id,
                    name: incoming_name.clone(),
                    before: None,
                });
                rows.push(Row::Team {
                    cells: appended_cells(working, incoming, incoming_cells),
                    id: *incoming_id,
                    name: incoming_name.clone(),
                });
            }
        }
    }

    // While an id is held by more than one team row, revert the incoming
    // changes that produced the collision. Each pass removes at least one
    // change, so the loop terminates.
    let mut unresolved = Vec::new();
    loop {
        let mut seen = BTreeSet::new();
        let mut duplicated = BTreeSet::new();
        for row in &rows {
            if let Row::Team { id, .. } = row
                && !seen.insert(*id)
            {
                duplicated.insert(*id);
            }
        }
        if duplicated.is_empty() {
            break;
        }
        let mut reverted = false;
        for change in &changes {
            if duplicated.contains(&change.id) {
                unresolved.push((change.id, change.name.clone()));
                if let Some(before) = &change.before {
                    rows[change.row] = before.clone();
                }
                reverted = true;
            }
        }
        // Remove appended rows whose change was reverted, then fix the row
        // indices of the surviving changes.
        let removed: BTreeSet<usize> = changes
            .iter()
            .filter(|change| duplicated.contains(&change.id) && change.before.is_none())
            .map(|change| change.row)
            .collect();
        changes.retain(|change| !duplicated.contains(&change.id));
        if !removed.is_empty() {
            let mut kept_rows = Vec::with_capacity(rows.len() - removed.len());
            let mut remap: BTreeMap<usize, usize> = BTreeMap::new();
            for (index, row) in rows.into_iter().enumerate() {
                if removed.contains(&index) {
                    continue;
                }
                remap.insert(index, kept_rows.len());
                kept_rows.push(row);
            }
            rows = kept_rows;
            for change in &mut changes {
                change.row = remap[&change.row];
            }
        }
        if !reverted {
            break;
        }
    }

    let mut added = Vec::new();
    let mut overridden = Vec::new();
    let mut surviving_overrides = 0usize;
    for change in &changes {
        match &change.before {
            Some(Row::Team { id: old_id, .. }) => {
                overridden.push((change.row, (change.name.clone(), *old_id, change.id)))
            }
            Some(Row::Placeholder { .. }) | None => {
                added.push((change.id, change.name.clone()));
            }
        }
    }
    // `overridden` is reported in working row order; `added` in the order the
    // rows were appended.
    overridden.sort_by_key(|(row, _)| *row);
    for change in &changes {
        if matches!(change.before, Some(Row::Team { .. })) {
            surviving_overrides += 1;
        }
    }
    let summary = MergeSummary {
        added,
        kept: working_team_count - surviving_overrides,
        overridden: overridden.into_iter().map(|(_, entry)| entry).collect(),
        unresolved,
    };

    let merged = TeamsList::from_parts(
        working.header().to_vec(),
        working.id_column(),
        working.name_column(),
        rows,
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
}
