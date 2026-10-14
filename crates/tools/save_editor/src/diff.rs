//! The comparator's report (`plans/save_editor.md` "Comparator (gameplay + aesthetics)"):
//! two saves' players paired by id, each changed field a row, a player at -1 marked stripped
//! beside his rows (the appearance id itself is no row), then the players only in one save.

use pes_savefile::EditFile;
use pes_savefile::ops::compare::{CompareError, DiffScope, compare};
use serde::Serialize;

use crate::events::Report;
use crate::messages::{Code, Side, diff_fields, message, player_diff};
use crate::session::is_stripped;

/// The report as data, rendered either as finding lines or as one JSON document (`--json`),
/// so the two forms cannot drift.
#[derive(Serialize)]
struct DiffReport {
    /// `first`'s players that `second` also holds, with a row in scope or a stripped mark, in
    /// `first`'s record order (a player only in `first` is in `only_in_first` alone).
    players: Vec<PlayerReport>,
    /// Ids `first` alone holds, in its record order.
    only_in_first: Vec<OnlyInPlayer>,
    /// Ids `second` alone holds, in its record order.
    only_in_second: Vec<OnlyInPlayer>,
}

/// One player of the report: his stripped mark (`"first"`, `"second"` or `"both"`; null when
/// his appearance id is set in both saves) and his changed fields.
#[derive(Serialize)]
struct PlayerReport {
    id: u32,
    stripped: Option<&'static str>,
    rows: Vec<DiffRow>,
}

/// A player one save alone holds, with his stripped mark.
#[derive(Serialize)]
struct OnlyInPlayer {
    id: u32,
    stripped: Option<&'static str>,
}

/// One changed field: `diff_fields`'s schema name and old/new texts, the same strings a
/// `player_diff` finding carries.
#[derive(Serialize)]
struct DiffRow {
    field: String,
    old: String,
    new: String,
}

/// Reports the differences between `first` and `second` as findings: in `first`'s record
/// order, each player's stripped mark (for a -1 in either save) and his rows of `scope`
/// (every row when `None`); then the players only in one save, each with its mark. Saves of
/// two versions are the lib's `VersionMismatch`.
pub(crate) fn report(
    first: &EditFile,
    second: &EditFile,
    scope: Option<DiffScope>,
    report: &mut Report,
) -> Result<(), CompareError> {
    let data = build(first, second, scope)?;
    for player in &data.players {
        if let Some(side) = player.stripped {
            report.message(message(
                Code::PlayerStripped,
                vec![("player", player.id.to_string()), ("save", side.to_owned())],
            ));
        }
        for row in &player.rows {
            report.message(player_diff(player.id, &row.field, &row.old, &row.new));
        }
    }
    for (entries, side) in [
        (&data.only_in_first, Side::First),
        (&data.only_in_second, Side::Second),
    ] {
        for entry in entries {
            report.message(message(
                Code::PlayerOnlyIn,
                vec![
                    ("player", entry.id.to_string()),
                    ("save", side.as_str().to_owned()),
                ],
            ));
            if let Some(stripped) = entry.stripped {
                report.message(message(
                    Code::PlayerStripped,
                    vec![
                        ("player", entry.id.to_string()),
                        ("save", stripped.to_owned()),
                    ],
                ));
            }
        }
    }
    Ok(())
}

/// `report`'s data as one pretty-printed JSON document, sent as the run's `Output` event:
/// the only event `--json` emits on success, so stdout parses for tooling.
pub(crate) fn report_json(
    first: &EditFile,
    second: &EditFile,
    scope: Option<DiffScope>,
    report: &mut Report,
) -> Result<(), CompareError> {
    let data = build(first, second, scope)?;
    let text = serde_json::to_string_pretty(&data).expect("a report of ids and strings serializes");
    report.output(text);
    Ok(())
}

/// Both saves' report as data. Saves of two versions are the lib's `VersionMismatch`.
fn build(
    first: &EditFile,
    second: &EditFile,
    scope: Option<DiffScope>,
) -> Result<DiffReport, CompareError> {
    let diff = compare(first, second)?;
    let mut data = DiffReport {
        players: Vec::new(),
        only_in_first: Vec::new(),
        only_in_second: Vec::new(),
    };
    for player in first.players() {
        if diff.only_in_a.contains(&player.id) {
            // An unmatched player is reported once, in `only_in_first`, with his mark.
            continue;
        }
        let stripped = stripped_side(first, second, player.id);
        let rows: Vec<DiffRow> = diff
            .players
            .iter()
            .find(|(id, _)| *id == player.id)
            .map(|(_, rows)| {
                rows.iter()
                    .filter(|row| scope.is_none_or(|scope| row.scope() == scope))
                    .map(|row| {
                        let (field, old, new) = diff_fields(row);
                        DiffRow { field, old, new }
                    })
                    .collect()
            })
            .unwrap_or_default();
        if stripped.is_some() || !rows.is_empty() {
            data.players.push(PlayerReport {
                id: player.id,
                stripped: stripped.map(Side::as_str),
                rows,
            });
        }
    }
    for (ids, entries) in [
        (&diff.only_in_a, &mut data.only_in_first),
        (&diff.only_in_b, &mut data.only_in_second),
    ] {
        for &id in ids {
            entries.push(OnlyInPlayer {
                id,
                stripped: stripped_side(first, second, id).map(Side::as_str),
            });
        }
    }
    Ok(data)
}

/// Which of the two saves `id` is stripped in (at -1), if either.
fn stripped_side(first: &EditFile, second: &EditFile, id: u32) -> Option<Side> {
    match (is_stripped(first, id), is_stripped(second, id)) {
        (true, true) => Some(Side::Both),
        (true, false) => Some(Side::First),
        (false, true) => Some(Side::Second),
        (false, false) => None,
    }
}
