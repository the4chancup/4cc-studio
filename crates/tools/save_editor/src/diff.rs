//! The comparator's report (`plans/save_editor.md` "Comparator (gameplay + aesthetics)"):
//! two saves' players paired by id, each changed field a row, a player at -1 marked stripped
//! beside his rows (the appearance id itself is no row), then the players only in one save.

use pes_savefile::EditFile;
use pes_savefile::ops::compare::{CompareError, DiffScope, compare};

use crate::events::Report;
use crate::messages::{Code, Side, message, player_diff};
use crate::session::is_stripped;

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
    let diff = compare(first, second)?;
    for player in first.players() {
        mark_stripped(first, second, player.id, report);
        let Some((_, rows)) = diff.players.iter().find(|(id, _)| *id == player.id) else {
            continue;
        };
        for row in rows
            .iter()
            .filter(|row| scope.is_none_or(|scope| row.scope() == scope))
        {
            report.message(player_diff(player.id, row));
        }
    }
    for (ids, side) in [
        (&diff.only_in_a, Side::First),
        (&diff.only_in_b, Side::Second),
    ] {
        for &id in ids {
            report.message(message(
                Code::PlayerOnlyIn,
                vec![
                    ("player", id.to_string()),
                    ("save", side.as_str().to_owned()),
                ],
            ));
            mark_stripped(first, second, id, report);
        }
    }
    Ok(())
}

/// One `player_stripped` line when `id` is at -1 in either save, saying which.
fn mark_stripped(first: &EditFile, second: &EditFile, id: u32, report: &mut Report) {
    let side = match (is_stripped(first, id), is_stripped(second, id)) {
        (true, true) => Side::Both,
        (true, false) => Side::First,
        (false, true) => Side::Second,
        (false, false) => return,
    };
    report.message(message(
        Code::PlayerStripped,
        vec![
            ("player", id.to_string()),
            ("save", side.as_str().to_owned()),
        ],
    ));
}
