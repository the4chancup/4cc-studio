//! The "Starting eleven" and "Bench" lists (`plans/save_editor.md`
//! "Tactics editor", the lineup bullet; the draft's `lineupList` and
//! `benchList`): each row a drag handle, a position and a name; drops
//! move, exchange or `swap_in`.

use pes_savefile::model::tactics::{SetPieceTakers, TeamTactics};

use super::{LineupDrag, position_label};
use crate::names::append_name;
use crate::tactics::{RosterRow, TacticsView, raw_name_at};

/// The draft's `tacticsTab` beside the pitch: the "Starting eleven" and
/// "Bench" lists, each row a drag handle, a position and a name (the
/// positions are `view.formation`'s; the selector heads `pitch`, as the
/// draft heads the pitch with it). Drops: a starter on a starter
/// exchanges the two, a bench row on a starter (or a starter on a filled
/// bench row) swaps them in `swap_in`'s sense, a bench row on a bench row
/// moves the dragged entry to the dropped row's place. `roster` gives each
/// roster slot's row, indexed by slot; an empty bench slot is no drag
/// source and takes no starter, so the eleven never gains an empty slot.
/// Answers `true` only when `tactics` changed this frame
/// (`tactics_settings`' contract).
pub fn lineup(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) -> bool {
    ui.push_id(id_salt, |ui| {
        // The answer is the deep compare, as `tactics_settings`' is.
        let before = tactics.clone();
        let positions = tactics.presets[view.preset.min(2)].formations[view.formation.min(2)]
            .players
            .map(|slot| slot.position);
        ui.strong("Starting eleven");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("starting eleven")
                .max_height(300.0)
                .show(ui, |ui| {
                    for (index, position) in positions.iter().enumerate() {
                        starter_row(ui, tactics, roster, *position, index);
                    }
                });
        });
        ui.strong("Bench");
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("bench")
                .max_height(300.0)
                .show(ui, |ui| {
                    for index in 0..tactics.bench_order.len() {
                        bench_row(ui, tactics, roster, index);
                    }
                });
        });
        *tactics != before
    })
    .inner
}

/// One starting-eleven row (the draft's XI `lrow`): the formation slot's
/// position and the player's name, always a drag source and a drop
/// target — the goalkeeper a starter like the others. Drops: a starter
/// exchanges the two slots' players (both stay on the pitch, so no role
/// moves); a bench player swaps in (`swap_in`).
fn starter_row(
    ui: &mut egui::Ui,
    tactics: &mut TeamTactics,
    roster: &[RosterRow<'_>],
    position: u8,
    index: usize,
) {
    let slot = tactics.starting_eleven[index];
    let response = lineup_row(
        ui,
        ("starting eleven", index),
        Some(LineupDrag::Starter(index)),
        position_label(position),
        raw_name_at(roster, slot),
        false,
    );
    if let Some(dropped) = response.dnd_release_payload::<LineupDrag>() {
        match *dropped {
            LineupDrag::Starter(from) | LineupDrag::Marker(from) => {
                tactics.starting_eleven.swap(from, index);
            }
            LineupDrag::Bench(from) => swap_in(tactics, index, from),
        }
    }
}

/// One bench row (the draft's bench `lrow`): the player's registered
/// position and name, "--" and "(empty)" on an empty slot, dimmed. An
/// empty row is no drag source and a starter dropped on it does nothing
/// (the eleven never gets an empty slot); a bench row dropped on any
/// bench row moves the dragged entry to the dropped row's place, the rows
/// between shifting by one (the draft's `moveBench`, a move, not an
/// exchange).
fn bench_row(ui: &mut egui::Ui, tactics: &mut TeamTactics, roster: &[RosterRow<'_>], index: usize) {
    let slot = tactics.bench_order[index];
    let row = roster.get(usize::from(slot));
    let empty = row.is_none_or(|row| row.registered_position.is_none());
    let position = row
        .and_then(|row| row.registered_position)
        .map_or("--", position_label);
    let response = lineup_row(
        ui,
        ("bench", index),
        (!empty).then_some(LineupDrag::Bench(index)),
        position,
        raw_name_at(roster, slot),
        empty,
    );
    if let Some(dropped) = response.dnd_release_payload::<LineupDrag>() {
        match *dropped {
            LineupDrag::Starter(starter) | LineupDrag::Marker(starter) => {
                if !empty {
                    swap_in(tactics, starter, index);
                }
            }
            LineupDrag::Bench(from) => {
                if from < index {
                    tactics.bench_order[from..=index].rotate_left(1);
                } else if from > index {
                    tactics.bench_order[index..=from].rotate_right(1);
                }
            }
        }
    }
}

/// One lineup row, the draft's `.lrow`: the ≡ handle, the position
/// column and the raw name drawn with its colour codes; a drag source
/// when `payload` is some, a drop target always. `empty` dims the row.
fn lineup_row(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    payload: Option<LineupDrag>,
    position: &str,
    raw_name: &str,
    empty: bool,
) -> egui::Response {
    let contents = |ui: &mut egui::Ui| {
        ui.horizontal(|ui| {
            ui.weak("≡");
            ui.add_sized(
                egui::vec2(30.0, ui.spacing().interact_size.y),
                egui::Label::new(egui::RichText::new(position).weak()),
            );
            if empty {
                ui.weak(raw_name);
            } else {
                let mut job = egui::text::LayoutJob::default();
                append_name(&mut job, raw_name);
                ui.label(job);
            }
        });
    };
    match payload {
        Some(payload) => {
            ui.dnd_drag_source(ui.id().with(id_salt), payload, contents)
                .response
        }
        None => ui.scope(contents).response,
    }
}

/// A bench player swapped onto the pitch: `starting_eleven[starter]` and
/// `bench_order[bench]` exchange, then every set-piece taker, every
/// join-attack entry and every preset's instruction target that held the
/// outgoing starter's roster slot now holds the incoming one's, so no
/// role points at a player who left the pitch (`plans/save_editor.md`
/// "Tactics editor", "A starter swapped with a bench player"). The
/// captain stays who he was, benched or not.
pub(super) fn swap_in(tactics: &mut TeamTactics, starter: usize, bench: usize) {
    let out = tactics.starting_eleven[starter];
    let inn = tactics.bench_order[bench];
    tactics.starting_eleven[starter] = inn;
    tactics.bench_order[bench] = out;
    let SetPieceTakers {
        free_kick_long,
        free_kick_short,
        free_kick_second,
        corner_left,
        corner_right,
        penalty,
        // Named, not `..`: a taker field added later must join the list
        // above or fail to compile here, never be skipped like the captain.
        captain: _,
    } = &mut tactics.set_pieces;
    for taker in [
        free_kick_long,
        free_kick_short,
        free_kick_second,
        corner_left,
        corner_right,
        penalty,
    ] {
        if *taker == out {
            *taker = inn;
        }
    }
    for join in &mut tactics.players_to_join_attack {
        if *join == out {
            *join = inn;
        }
    }
    for preset in &mut tactics.presets {
        for instructions in [
            &mut preset.attack_instructions,
            &mut preset.defence_instructions,
        ]
        .into_iter()
        .flatten()
        {
            for entry in instructions {
                if entry.player == out {
                    entry.player = inn;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::accesskit::{self, Role};
    use pes_version::PesVersion;

    use super::super::{marker_rect, pitch_point};
    use super::*;
    use crate::pitch::tests::{canvas_rect, lineup_rows, position_of_row, row_centre};
    use crate::test_support::{
        Headless, centre, entry, fixture_rows, fixture_tactics, pointer_at, roster_rows,
    };

    // plans/save_editor.md "Tactics editor", "A starter swapped with a
    // bench player": every set-piece role, the joining of the attack and
    // every preset's instruction targets the starter held repoint to the
    // incoming player; the captain stays.
    #[test]
    fn a_swap_hands_the_starters_roles_to_the_bench_player_but_not_the_captaincy() {
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let out = tactics.starting_eleven[4];
        let inn = tactics.bench_order[2];
        assert_ne!(out, inn, "premise: two different slots");
        let other = tactics.starting_eleven[6];
        assert!(other != out && other != inn, "premise: a third slot");
        let pieces = &mut tactics.set_pieces;
        pieces.free_kick_long = out;
        pieces.free_kick_short = out;
        pieces.free_kick_second = out;
        pieces.corner_left = out;
        pieces.corner_right = out;
        pieces.penalty = out;
        pieces.captain = out;
        tactics.players_to_join_attack = [other, out, u8::MAX];
        tactics.presets[1]
            .attack_instructions
            .as_mut()
            .expect("PES 21 stores instructions")[0]
            .player = out;
        tactics.presets[2]
            .defence_instructions
            .as_mut()
            .expect("PES 21 stores instructions")[1]
            .player = out;
        tactics.presets[0]
            .defence_instructions
            .as_mut()
            .expect("PES 21 stores instructions")[0]
            .player = other;
        swap_in(&mut tactics, 4, 2);
        assert_eq!(tactics.starting_eleven[4], inn, "the bench player starts");
        assert_eq!(tactics.bench_order[2], out, "the starter sits");
        let pieces = &tactics.set_pieces;
        for taker in [
            pieces.free_kick_long,
            pieces.free_kick_short,
            pieces.free_kick_second,
            pieces.corner_left,
            pieces.corner_right,
            pieces.penalty,
        ] {
            assert_eq!(taker, inn, "every taker follows the swap");
        }
        assert_eq!(pieces.captain, out, "the captain stays who he was");
        assert_eq!(
            tactics.players_to_join_attack,
            [other, inn, u8::MAX],
            "the join entry follows, the other starter's stays"
        );
        assert_eq!(
            tactics.presets[1]
                .attack_instructions
                .as_ref()
                .expect("instructions")[0]
                .player,
            inn,
            "preset 2's attack target follows"
        );
        assert_eq!(
            tactics.presets[2]
                .defence_instructions
                .as_ref()
                .expect("instructions")[1]
                .player,
            inn,
            "preset 3's defence target follows"
        );
        assert_eq!(
            tactics.presets[0]
                .defence_instructions
                .as_ref()
                .expect("instructions")[0]
                .player,
            other,
            "the other starter's target stays"
        );
    }

    // The same swap on a version with no instruction table moves the
    // takers and the join entries only.
    #[test]
    fn a_swap_on_pes16_moves_the_takers_and_join_entries() {
        let mut tactics = fixture_tactics(PesVersion::Pes16);
        assert!(
            tactics.presets.iter().all(|preset| {
                preset.attack_instructions.is_none() && preset.defence_instructions.is_none()
            }),
            "premise: PES 16 stores no instructions"
        );
        let out = tactics.starting_eleven[0];
        let inn = tactics.bench_order[0];
        assert_ne!(out, inn, "premise: two different slots");
        let pieces = &mut tactics.set_pieces;
        pieces.penalty = out;
        pieces.captain = out;
        tactics.players_to_join_attack = [out, u8::MAX, u8::MAX];
        swap_in(&mut tactics, 0, 0);
        assert_eq!(tactics.starting_eleven[0], inn);
        assert_eq!(tactics.bench_order[0], out);
        assert_eq!(tactics.set_pieces.penalty, inn, "the taker follows");
        assert_eq!(tactics.set_pieces.captain, out, "the captain stays");
        assert_eq!(
            tactics.players_to_join_attack,
            [inn, u8::MAX, u8::MAX],
            "the join entry follows"
        );
    }

    // The draft's `rowDrop` over `lineupList`: a bench row dropped on a
    // starter runs the swap through the widget.
    #[test]
    fn a_bench_row_dropped_on_a_starter_swaps_them_and_moves_the_roles() {
        let mut window = Headless::sized(900.0, 1100.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let out = tactics.starting_eleven[3];
        let inn = tactics.bench_order[0];
        assert_ne!(out, inn, "premise: two different slots");
        tactics.set_pieces.penalty = out;
        let (nodes, _) = window.lineup(&mut tactics, &mut view, &roster, vec![]);
        let changed = window.drag_lineup(
            row_centre(&nodes, 11),
            row_centre(&nodes, 3),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(changed, "the drop changed the record");
        assert_eq!(tactics.starting_eleven[3], inn, "the bench player starts");
        assert_eq!(tactics.bench_order[0], out, "the starter sits");
        assert_eq!(
            tactics.set_pieces.penalty, inn,
            "the taker followed the swap"
        );
    }

    // The draft's `swapStarters`: a starter on a starter exchanges the two
    // slots' players; both stay on the pitch so no role moves.
    #[test]
    fn a_starter_dropped_on_a_starter_exchanges_them_and_moves_no_role() {
        let mut window = Headless::sized(900.0, 1100.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let first = tactics.starting_eleven[1];
        let second = tactics.starting_eleven[2];
        assert_ne!(first, second, "premise: two different slots");
        tactics.set_pieces.penalty = first;
        let (nodes, _) = window.lineup(&mut tactics, &mut view, &roster, vec![]);
        let changed = window.drag_lineup(
            row_centre(&nodes, 1),
            row_centre(&nodes, 2),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(changed, "the drop changed the record");
        assert_eq!(tactics.starting_eleven[1], second);
        assert_eq!(tactics.starting_eleven[2], first);
        assert_eq!(
            tactics.set_pieces.penalty, first,
            "the role stays on the player"
        );
    }

    // The draft's `moveBench`: the dragged entry lands on the dropped
    // row's place, the rows between shifting by one toward it.
    #[test]
    fn a_bench_row_dropped_on_a_bench_row_lands_in_its_place() {
        let mut window = Headless::sized(900.0, 1100.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let before = tactics.bench_order;
        assert_ne!(before[4], before[1], "premise: two different slots");

        // Row 4 dropped on row 1: it takes 1's place, the rows between
        // shifting down.
        let (nodes, _) = window.lineup(&mut tactics, &mut view, &roster, vec![]);
        let changed = window.drag_lineup(
            row_centre(&nodes, 11 + 4),
            row_centre(&nodes, 11 + 1),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(changed, "the drop changed the record");
        let mut expect = before;
        expect[1] = before[4];
        expect[2] = before[1];
        expect[3] = before[2];
        expect[4] = before[3];
        assert_eq!(tactics.bench_order, expect, "row 4 landed at 1");

        // Row 1 dropped on row 4: it takes 4's place, the rows between
        // shifting up.
        tactics = fixture_tactics(PesVersion::Pes21);
        let before = tactics.bench_order;
        let (nodes, _) = window.lineup(&mut tactics, &mut view, &roster, vec![]);
        let changed = window.drag_lineup(
            row_centre(&nodes, 11 + 1),
            row_centre(&nodes, 11 + 4),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(changed, "the drop changed the record");
        let mut expect = before;
        expect[1] = before[2];
        expect[2] = before[3];
        expect[3] = before[4];
        expect[4] = before[1];
        assert_eq!(tactics.bench_order, expect, "row 1 landed at 4");
    }

    // An empty bench slot is no drag source and a starter dropped on it
    // does nothing: the eleven never gets an empty slot (the fixture's
    // bench rows 12 to 20 hold roster slots 23 to 31, past team 701's 23
    // players).
    #[test]
    fn the_empty_bench_rows_neither_drag_nor_take_a_starter() {
        let mut window = Headless::sized(900.0, 1100.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        assert!(
            tactics.bench_order.iter().any(|slot| {
                roster
                    .get(usize::from(*slot))
                    .is_none_or(|row| row.registered_position.is_none())
            }),
            "premise: the fixture's bench has an empty row"
        );
        let before = tactics.clone();
        // The first bench row naming nobody, inside the list's viewport.
        let empty_row = |nodes: &[accesskit::Node]| {
            lineup_rows(nodes)
                .iter()
                .enumerate()
                .skip(11)
                .find(|(_, row)| row.get(2).and_then(|node| node.value()) == Some("(empty)"))
                .map(|(index, _)| index)
                .expect("an empty bench row is drawn")
        };
        let (nodes, _) = window.lineup(&mut tactics, &mut view, &roster, vec![]);
        let empty = empty_row(&nodes);

        // Dragging an empty row onto a starter changes nothing.
        let changed = window.drag_lineup(
            row_centre(&nodes, empty),
            row_centre(&nodes, 5),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "an empty row is no drag source");
        assert_eq!(tactics, before, "and nothing moved");

        // A starter dropped on an empty row changes nothing.
        let (nodes, _) = window.lineup(&mut tactics, &mut view, &roster, vec![]);
        let empty = empty_row(&nodes);
        let changed = window.drag_lineup(
            row_centre(&nodes, 5),
            row_centre(&nodes, empty),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "an empty row takes no starter");
        assert_eq!(tactics, before, "the eleven keeps its player");
    }

    // The XI rows' position labels read the preset's formation on screen:
    // the selector (the pitch's header) writes `view.formation`, never the
    // record.
    #[test]
    fn the_position_labels_follow_the_formation_on_screen() {
        let mut window = Headless::sized(900.0, 1100.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        tactics.presets[0].formations[0].players[5].position = 4; // DMF
        tactics.presets[0].formations[1].players[5].position = 8; // AMF
        let before = tactics.clone();
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let (nodes, _) = window.pitch_and_lineup(&mut tactics, &mut view, &roster, vec![]);
        assert_eq!(position_of_row(&nodes, 5), "DMF", "kick-off's slot 5");

        let at = centre(entry(&nodes, Role::Button, "In-possession"));
        window.pitch_and_lineup(&mut tactics, &mut view, &roster, pointer_at(at, true));
        let (_, changed) =
            window.pitch_and_lineup(&mut tactics, &mut view, &roster, pointer_at(at, false));
        assert!(!changed, "the selector writes the view, not the record");
        assert_eq!(view.formation, 1, "in-possession is on screen");
        assert_eq!(tactics, before, "and the record is untouched");

        let (nodes, _) = window.pitch_and_lineup(&mut tactics, &mut view, &roster, vec![]);
        assert_eq!(position_of_row(&nodes, 5), "AMF", "in-possession's slot 5");
    }

    // And the reverse direction: a marker dragged onto a bench row swaps
    // him out through `swap_in`, as a marker's `LineupDrag::Starter` is
    // the row's payload.
    #[test]
    fn a_marker_dropped_on_a_bench_row_swaps_him_out() {
        let mut window = Headless::sized(600.0, 1100.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let index = 4;
        let out = tactics.starting_eleven[index];
        let inn = tactics.bench_order[0];
        assert_ne!(out, inn, "premise: two different slots");
        tactics.set_pieces.penalty = out;
        let (nodes, _) = window.pitch_and_lineup(&mut tactics, &mut view, &roster, vec![]);
        let canvas = canvas_rect(&nodes);
        let formation = tactics.presets[0].formations[0].clone();
        let from = marker_rect(
            pitch_point(
                canvas.min,
                formation.players[index].x,
                formation.players[index].y,
            ),
            canvas,
        )
        .center();

        let changed = window.drag_pitch_and_lineup(
            from,
            row_centre(&nodes, 11),
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(changed, "the drop changed the record");
        assert_eq!(
            tactics.starting_eleven[index], inn,
            "the bench player starts"
        );
        assert_eq!(tactics.bench_order[0], out, "the starter sits");
        assert_eq!(
            tactics.set_pieces.penalty, inn,
            "the taker followed the swap"
        );
    }
}
