//! The position a starter takes at a spot of the pitch: the rule the
//! game's own formation editor applies at every step of a drag and in
//! its Select Position menu, read from PES 2016's executable (the last
//! version left unpacked) and checked against PES 2017's and PES 2021's
//! editors. It is no fixed map of pitch areas to positions: a point
//! offers up to two positions and the dragged player's own per-position
//! overalls pick between them, so the same spot gives different players
//! different positions.

use pes_savefile::model::tactics::Formation;

/// A position's pitch area in stored units, bounds inclusive (x 0 to
/// 104 across, y 0 to 48 up).
#[derive(Clone, Copy)]
pub(super) struct Rect {
    pub(super) y_min: u8,
    pub(super) y_max: u8,
    pub(super) x_min: u8,
    pub(super) x_max: u8,
}

impl Rect {
    /// The point is inside the area, bounds included.
    fn contains(self, x: u8, y: u8) -> bool {
        self.y_min <= y && y <= self.y_max && self.x_min <= x && x <= self.x_max
    }
}

/// One table row, `(y_min, y_max, x_min, x_max)` as the tables store it.
const fn rect(y_min: u8, y_max: u8, x_min: u8, x_max: u8) -> Rect {
    Rect {
        y_min,
        y_max,
        x_min,
        x_max,
    }
}

/// The two area tables the game holds per position, indexed by the
/// position byte (GK 0 to CF 12); a position's area holds a point when
/// either row does. The second table widens the first's left and
/// central rows out to the pitch's edges and its top rows to y 48 —
/// the right-side rows it leaves at x 99 — and both are kept as the
/// game has them. The first table's rows are also the areas a held
/// starter lights.
pub(super) const AREAS_A: [Rect; 13] = [
    rect(0, 3, 52, 56),   // GK
    rect(4, 15, 5, 99),   // CB
    rect(4, 15, 5, 20),   // LB
    rect(4, 15, 84, 99),  // RB
    rect(16, 25, 21, 83), // DMF
    rect(16, 34, 5, 99),  // CMF
    rect(16, 34, 5, 20),  // LMF
    rect(16, 34, 84, 99), // RMF
    rect(26, 34, 21, 83), // AMF
    rect(35, 47, 5, 20),  // LWF
    rect(35, 47, 84, 99), // RWF
    rect(35, 47, 5, 99),  // SS
    rect(40, 47, 21, 83), // CF
];
/// The second table; `AREAS_A`'s doc explains it.
const AREAS_B: [Rect; 13] = [
    rect(0, 3, 52, 56),   // GK
    rect(4, 15, 0, 104),  // CB
    rect(4, 15, 0, 20),   // LB
    rect(4, 15, 84, 99),  // RB
    rect(16, 25, 21, 83), // DMF
    rect(16, 34, 0, 104), // CMF
    rect(16, 34, 0, 20),  // LMF
    rect(16, 34, 84, 99), // RMF
    rect(26, 34, 21, 83), // AMF
    rect(35, 48, 0, 20),  // LWF
    rect(35, 48, 84, 99), // RWF
    rect(35, 48, 0, 104), // SS
    rect(40, 48, 21, 83), // CF
];

/// The full-width bands' defaults, in the order the game tries them:
/// CB, CMF, SS.
const BAND_DEFAULTS: [u8; 3] = [1, 5, 11];
/// The positions tried for a candidate, in the game's order: LB, RB,
/// DMF, LMF, RMF, AMF, LWF, RWF, CF, then CB and SS.
const CANDIDATES: [u8; 11] = [2, 3, 4, 6, 7, 8, 9, 10, 12, 1, 11];
/// The wide positions (LB, RB, LMF, RMF, LWF, RWF): one is offered
/// only when no other slot holds it; the rest are always free, however
/// many slots share them.
const WIDE: [u8; 6] = [2, 3, 6, 7, 9, 10];

/// The positions the snapped point `(x, y)` in stored units offers
/// `slot` of `formation`, as the game's drag and its Select Position
/// menu offer them: the band's default (the first of `BAND_DEFAULTS`
/// whose area holds the point), then the first candidate of
/// `CANDIDATES` whose area holds it and which is free, each only when
/// found and the candidate left out when it is the default. At most
/// two; none in the keeper's box, GK being neither.
pub(super) fn spot_choices(formation: &Formation, slot: usize, x: u8, y: u8) -> Vec<u8> {
    let default = BAND_DEFAULTS
        .into_iter()
        .find(|&position| holds(position, x, y));
    let candidate = CANDIDATES
        .into_iter()
        .find(|&position| holds(position, x, y) && free(formation, slot, position))
        .filter(|&candidate| Some(candidate) != default);
    default.into_iter().chain(candidate).collect()
}

/// The position `slot` of `formation` takes at the snapped point `(x,
/// y)` in stored units: his stored position where the point offers it or
/// offers nothing (`spot_choices`); else the one position offered, or of
/// two the one `overalls` (his thirteen per-position ratings, indexed by
/// the position byte) rates higher, the band's default on a tie.
/// `overalls` `None` (an empty slot or a host that reads none) keeps his
/// position: no overall, no pick.
///
/// A drag rules every step from the stored position, which nothing
/// writes until the release, never from the position shown a step
/// earlier, as the game does: a CF dragged down to AMF and back up is a
/// CF again, where ruling step to step would leave him the SS he passed
/// through on the way back.
pub(super) fn dropped_position(
    formation: &Formation,
    slot: usize,
    x: u8,
    y: u8,
    overalls: Option<&[u8; 13]>,
) -> u8 {
    let current = formation.players[slot].position;
    let choices = spot_choices(formation, slot, x, y);
    if choices.contains(&current) {
        return current;
    }
    let Some(overalls) = overalls else {
        return current;
    };
    match *choices.as_slice() {
        [] => current,
        [only] => only,
        [default, candidate, ..] => {
            if overalls[usize::from(candidate)] > overalls[usize::from(default)] {
                candidate
            } else {
                default
            }
        }
    }
}

/// `position`'s area holds the point: either of its two table rows
/// does (bounds inclusive).
fn holds(position: u8, x: u8, y: u8) -> bool {
    let index = usize::from(position);
    AREAS_A[index].contains(x, y) || AREAS_B[index].contains(x, y)
}

/// `position` may be taken at the drop: a central one always, however
/// many slots share it; a wide one only when no slot of `formation`
/// but `slot` himself holds it.
fn free(formation: &Formation, slot: usize, position: u8) -> bool {
    !WIDE.contains(&position)
        || formation
            .players
            .iter()
            .enumerate()
            .all(|(index, other)| index == slot || other.position != position)
}
