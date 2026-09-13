//! Merging an incoming list into the working list.
//!
//! Rows only in `incoming` are appended as team rows (`rest` copied); rows
//! only in `working` are kept; a name present in both with a different id
//! takes the incoming id (the row stays in place, `rest` kept). Uniqueness is
//! then enforced: an incoming row whose id or name would collide with a
//! different existing row is left out and reported. Placeholders in `working`
//! are kept untouched; placeholders in `incoming` are ignored.

use crate::file::{Row, TeamsList};
use crate::id::TeamId;
use crate::name::TeamName;

/// Merges `incoming` into `working`, returning the merged list and a summary
/// for the caller to show before writing.
pub fn reconcile(working: &TeamsList, incoming: &TeamsList) -> (TeamsList, MergeSummary) {
    let mut rows = working.rows().to_vec();
    let mut summary = MergeSummary::default();

    // Folded name -> the incoming row's verbatim name spelling and rest
    // columns, so appended rows keep them.
    let incoming_details: std::collections::BTreeMap<&TeamName, (&str, &[String])> = incoming
        .rows()
        .iter()
        .filter_map(|row| match row {
            Row::Team {
                name,
                raw_name,
                rest,
                ..
            } => Some((name, (raw_name.as_str(), rest.as_slice()))),
            Row::Placeholder { .. } => None,
        })
        .collect();

    for (incoming_id, incoming_name) in incoming.teams() {
        let name_position = rows.iter().position(|row| match row {
            Row::Team { name, .. } => name == incoming_name,
            Row::Placeholder { .. } => false,
        });
        match name_position {
            Some(position) => {
                let Row::Team { id, .. } = &rows[position] else {
                    continue;
                };
                if *id == incoming_id {
                    // Same name, same id: the working row is already correct.
                    summary.kept += 1;
                } else if rows.iter().enumerate().any(|(other, row)| {
                    other != position && matches!(row, Row::Team { id, .. } if *id == incoming_id)
                }) {
                    // The incoming id belongs to a different row: applying it
                    // would collide, so the row is left out.
                    summary
                        .unresolved
                        .push((incoming_id, incoming_name.clone()));
                } else {
                    summary
                        .overridden
                        .push((incoming_name.clone(), *id, incoming_id));
                    if let Row::Team { id, .. } = &mut rows[position] {
                        *id = incoming_id;
                    }
                }
            }
            None => {
                if rows
                    .iter()
                    .any(|row| matches!(row, Row::Team { id, .. } if *id == incoming_id))
                {
                    summary
                        .unresolved
                        .push((incoming_id, incoming_name.clone()));
                } else {
                    let (raw_name, rest) = incoming_details
                        .get(incoming_name)
                        .map(|(raw, rest)| ((*raw).to_owned(), (*rest).to_vec()))
                        .unwrap_or_else(|| (incoming_name.as_str().to_owned(), Vec::new()));
                    rows.push(Row::Team {
                        id: incoming_id,
                        name: incoming_name.clone(),
                        raw_name,
                        rest,
                    });
                    summary.added.push((incoming_id, incoming_name.clone()));
                }
            }
        }
    }

    // Working team rows that were not overridden or matched stay as they are.
    let overridden = summary.overridden.len();
    let matched = summary.kept + overridden;
    summary.kept += working.teams().count().saturating_sub(matched);

    let merged = TeamsList::from_parts(working.header().to_vec(), rows);
    (merged, summary)
}

/// What a merge did, for the caller to report before writing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MergeSummary {
    /// Incoming rows appended at the end of the working list.
    pub added: Vec<(TeamId, TeamName)>,
    /// Working team rows kept as they were (working-only rows and rows
    /// identical in both lists).
    pub kept: usize,
    /// Names present in both whose id changed to the incoming one.
    pub overridden: Vec<(TeamName, TeamId, TeamId)>,
    /// Incoming rows left out because their id or name would collide with a
    /// different existing row.
    pub unresolved: Vec<(TeamId, TeamName)>,
}
