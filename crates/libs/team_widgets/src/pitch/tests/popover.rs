//! The slot popover's tests and the armed assign modes': the open
//! and close cases, the fields' writes, the role chips and join.

use egui::accesskit::Role;
use pes_savefile::model::instruction::Instruction;
use pes_savefile::model::tactics::SetPieceTakers;
use pes_savefile::schema::fields::InstructionSide;
use pes_savefile::schema::instruction;
use pes_version::PesVersion;

use super::*;
use crate::test_support::{
    Headless, centre, chip_under, chips_below, combo_row, controls_right_of, entry, entry_id,
    fixture_rows, fixture_tactics, roster_rows,
};

// The draft's role palette: arm a chip and the note asks for its pitch
// player; the click writes his roster slot to the taker and disarms.
#[test]
fn a_role_chip_then_a_marker_click_writes_the_taker_and_disarms() {
    // The window is wide enough to hold the whole chips row (the
    // settings pane's content is wider than the pitch column).
    let mut window = Headless::sized(1400.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let index = 7;
    // A non-identity eleven: an arm writing the marker's index for the
    // roster slot would look right on the fixture's [0, 1, ...].
    tactics.starting_eleven.swap(2, index);
    let starter = tactics.starting_eleven[index];
    assert_ne!(starter, index as u8, "premise: slot and index differ");
    assert_ne!(
        tactics.set_pieces.penalty, starter,
        "premise: the marker's player is not already the taker"
    );

    // Arm the Penalty kick chip (the first chip under its label).
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chips_below(&nodes, "Penalty kick")
        .into_iter()
        .next()
        .expect("the penalty chip");
    let (_, changed) =
        window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert!(!changed, "arming writes no record");
    assert_eq!(view.assign, Some(Assign::Role(5)), "the mode is armed");

    // The marker click writes the taker; no note is drawn while armed
    // (the plan's "No note explains the armed mode").
    let (nodes, _) = window.pitch(version, &mut tactics, &mut view, &roster, vec![]);
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button && matches!(node.label(), Some("Cancel") | Some("Done"))
        }),
        "no Cancel or Done while armed"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Label
                && node
                    .value()
                    .is_some_and(|value| value.starts_with("Click the pitch player"))
        }),
        "no armed note's text"
    );
    let formation = tactics.presets[0].formations[0].clone();
    let at = marker_point(&nodes, &formation.players, index);
    let (_, changed) = window.click_pitch(at, version, &mut tactics, &mut view, &roster);
    assert!(changed, "the assignment writes the record");
    assert_eq!(
        tactics.set_pieces.penalty, starter,
        "the marker's player takes it"
    );
    assert_eq!(view.assign, None, "and the mode disarms");
}

// Every chip writes its own field: arm it, click a starter's marker —
// the chip's field alone takes the clicked starter's roster slot (the
// fields' own names, never `SET_PIECES`'s table).
#[test]
fn each_assignment_chip_writes_its_own_field() {
    let mut window = Headless::sized(1400.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    // A non-identity eleven: no index equals its roster slot.
    tactics.starting_eleven.rotate_left(1);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    /// One chip case: its label prefix, its field's read and the expected
    /// write — the field's own name, never `SET_PIECES`'s table.
    struct Chip {
        /// The chip's label prefix.
        label: &'static str,
        /// The field's stored read.
        get: fn(&SetPieceTakers) -> u8,
        /// The expected write, on a `TeamTactics` clone.
        set: fn(&mut SetPieceTakers, u8),
    }
    let cases: [Chip; 7] = [
        Chip {
            label: "Long free kick",
            get: |s| s.free_kick_long,
            set: |s, v| s.free_kick_long = v,
        },
        Chip {
            label: "Short free kick",
            get: |s| s.free_kick_short,
            set: |s, v| s.free_kick_short = v,
        },
        Chip {
            label: "Second free kick",
            get: |s| s.free_kick_second,
            set: |s, v| s.free_kick_second = v,
        },
        Chip {
            label: "Left corner",
            get: |s| s.corner_left,
            set: |s, v| s.corner_left = v,
        },
        Chip {
            label: "Right corner",
            get: |s| s.corner_right,
            set: |s, v| s.corner_right = v,
        },
        Chip {
            label: "Penalty kick",
            get: |s| s.penalty,
            set: |s, v| s.penalty = v,
        },
        Chip {
            label: "Captain",
            get: |s| s.captain,
            set: |s, v| s.captain = v,
        },
    ];

    // Markers a click can reach at their box's centre: no marker drawn
    // after them covers it (egui resolves to the topmost).
    let (nodes, _) = window.pitch(version, &mut tactics, &mut view, &roster, vec![]);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &formation.players);
    let order = draw_order(&centres);
    let uncovered: Vec<usize> = (1..11)
        .filter(|&index| {
            let after = order
                .iter()
                .position(|&i| i == index)
                .expect("every marker is in the order");
            let at = marker_rect(centres[index], canvas).center();
            order[after + 1..]
                .iter()
                .all(|&other| !marker_rect(centres[other], canvas).contains(at))
        })
        .collect();
    assert!(
        uncovered.len() >= cases.len(),
        "premise: enough open markers"
    );

    for (index, case) in cases.iter().enumerate() {
        let chip_label = case.label;
        // Arm the chip under its label.
        let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chips_below(&nodes, chip_label)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no chip under {chip_label:?}"));
        let (_, changed) =
            window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
        assert!(!changed, "{chip_label}: arming writes no record");
        assert_eq!(view.assign, Some(Assign::Role(index)), "{chip_label} armed");

        let marker_index = uncovered[index];
        let starter = tactics.starting_eleven[marker_index];
        assert_ne!(
            starter, marker_index as u8,
            "{chip_label}: slot and index differ"
        );
        assert_ne!(
            (case.get)(&tactics.set_pieces),
            starter,
            "{chip_label}: the marker's player is not already the holder"
        );
        let mut expected = tactics.clone();
        (case.set)(&mut expected.set_pieces, starter);

        let (nodes, _) = window.pitch(version, &mut tactics, &mut view, &roster, vec![]);
        let at = marker_point(&nodes, &formation.players, marker_index);
        let (_, changed) = window.click_pitch(at, version, &mut tactics, &mut view, &roster);
        assert!(changed, "{chip_label}: the click wrote the field");
        assert_eq!(tactics, expected, "{chip_label}: only its field written");
        assert_eq!(view.assign, None, "{chip_label}: disarmed");
    }
}

// The join pick toggles members until the chip's click disarms: three
// clicks fill the three entries, a fourth is ignored, and clicking a
// member removes him, packing the list (the plan's "Players to join the
// attack stay packed").
#[test]
fn join_clicks_fill_three_then_toggle_members_packed() {
    let mut window = Headless::sized(900.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    tactics.players_to_join_attack = [u8::MAX; 3];
    // A non-identity eleven: an arm writing the marker's index for the
    // roster slot would look right on the fixture's [0, 1, ...].
    tactics.starting_eleven.swap(4, 9);
    tactics.starting_eleven.swap(5, 10);
    tactics.starting_eleven.swap(6, 8);
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);

    // Arm join through a join chip.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chips_below(&nodes, "Join the attack")
        .into_iter()
        .next()
        .expect("a join chip");
    window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert_eq!(view.assign, Some(Assign::Join), "the mode is armed");

    // Three marker clicks fill the list, packed.
    let starters = tactics.starting_eleven;
    assert!(
        starters[4..=6]
            .iter()
            .enumerate()
            .all(|(i, s)| *s != (i + 4) as u8),
        "premise: the clicked positions' slots differ from their indices"
    );
    let click_marker =
        |window: &mut Headless, tactics: &mut TeamTactics, view: &mut TacticsView, index: usize| {
            let (nodes, _) = window.pitch(version, tactics, view, &roster, vec![]);
            let at = marker_point(&nodes, &tactics.presets[0].formations[0].players, index);
            window.click_pitch(at, version, tactics, view, &roster)
        };
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 4);
    assert!(changed, "the first click wrote");
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 5);
    assert!(changed, "the second click wrote");
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 6);
    assert!(changed, "the third click wrote");
    assert_eq!(
        tactics.players_to_join_attack,
        [starters[4], starters[5], starters[6]],
        "three clicks fill three"
    );
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 7);
    assert!(!changed, "a fourth is ignored");
    assert_eq!(
        tactics.players_to_join_attack,
        [starters[4], starters[5], starters[6]],
        "and the list is as it was"
    );
    // Clicking the first member removes him; the list packs.
    let (_, changed) = click_marker(&mut window, &mut tactics, &mut view, 4);
    assert!(changed, "the removal wrote");
    assert_eq!(
        tactics.players_to_join_attack,
        [starters[5], starters[6], u8::MAX],
        "the first member left and the list packed"
    );
    assert_eq!(view.assign, Some(Assign::Join), "join stays armed");

    // A click on an armed join chip disarms (the plan's "No note
    // explains the armed mode": the highlighted chips and the
    // candidates' corner brackets say it).
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chips_below(&nodes, "Join the attack")
        .into_iter()
        .next()
        .expect("a join chip");
    window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert_eq!(view.assign, None, "the armed chip's click disarms");
}

// plans/save_editor.md "Tactics editor" (the click-to-assign bullet's
// last sentence): an instruction's target chip arms the same pitch
// pick as the roles' — click the chip, then a starter's marker writes
// his roster slot as the target and disarms; a click on the player
// already the target writes nothing; and an instruction changed while
// armed disarms the mode without a write.
#[test]
fn a_target_chip_then_a_marker_click_writes_the_target_and_disarms() {
    // The window is wide enough to hold the whole chips row.
    let mut window = Headless::sized(1400.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    // Defence 1 Counter Target, its pick unset; a non-identity eleven
    // (an index write would look right on the fixture's [0, 1, ...]).
    tactics.presets[0]
        .defence_instructions
        .as_mut()
        .expect("PES 21 has instructions")[0]
        .instruction =
        instruction::encode(version, Instruction::CounterTarget).expect("PES 21 stores it");
    tactics.presets[0]
        .defence_instructions
        .as_mut()
        .expect("instructions")[0]
        .player = u8::MAX;
    tactics.starting_eleven.swap(3, 9);
    let starter = tactics.starting_eleven[3];
    assert_ne!(starter, 3, "premise: slot and index differ");
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);
    // The slot popover open: arming closes it.
    view.popover = Some(OpenSlot {
        scope: egui::Id::unique("the open slot's"),
        preset: 0,
        formation: 0,
        slot: 0,
    });

    // The chip under defence 1's combo: enabled; its click arms
    // `Assign::Target` and closes the popover.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chip_under(&nodes, combo_row(&nodes, 0)[1]);
    assert!(!chip.is_disabled(), "Counter Target's chip is enabled");
    let (_, changed) =
        window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert!(!changed, "arming writes no record");
    assert_eq!(
        view.assign,
        Some(Assign::Target {
            side: InstructionSide::Defence,
            index: 0
        }),
        "the mode is armed"
    );
    assert_eq!(view.popover, None, "the slot popover closed");
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chip_under(&nodes, combo_row(&nodes, 0)[1]);
    assert_eq!(
        chip.toggled(),
        Some(accesskit::Toggled::True),
        "the armed chip is selected"
    );

    // The marker click writes the starter's roster slot and disarms.
    let formation = tactics.presets[0].formations[0].clone();
    let (nodes, _) = window.pitch(version, &mut tactics, &mut view, &roster, vec![]);
    let at = marker_point(&nodes, &formation.players, 3);
    let (_, changed) = window.click_pitch(at, version, &mut tactics, &mut view, &roster);
    assert!(changed, "the assignment writes the record");
    assert_eq!(
        tactics.presets[0]
            .defence_instructions
            .as_ref()
            .expect("instructions")[0]
            .player,
        starter,
        "the starter's roster slot is the target"
    );
    assert_eq!(view.assign, None, "and the mode disarms");

    // Re-arm; a click on the player already the target writes nothing
    // (the mode still disarms).
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chip_under(&nodes, combo_row(&nodes, 0)[1]);
    let (_, _) = window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert!(view.assign.is_some(), "re-armed");
    let before = tactics.clone();
    let (nodes, _) = window.pitch(version, &mut tactics, &mut view, &roster, vec![]);
    let at = marker_point(&nodes, &formation.players, 3);
    let (_, changed) = window.click_pitch(at, version, &mut tactics, &mut view, &roster);
    assert!(!changed, "the same player writes nothing");
    assert_eq!(tactics, before, "the record is as it was");
    assert_eq!(view.assign, None, "still disarmed");

    // Armed again, the instruction switched to Deep Defensive Line: it
    // takes no target — the marker click writes nothing and disarms.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = chip_under(&nodes, combo_row(&nodes, 0)[1]);
    let (_, _) = window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert!(view.assign.is_some(), "re-armed");
    // Pick Deep Defensive Line on defence 1's combo — the popup's item
    // goes through AccessKit, as the instruction picks' do.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let combo = combo_row(&nodes, 0)[1];
    let (_, _) = window.click_settings(centre(combo), version, &mut tactics, &mut view, &roster);
    // The popup's first frame is a sizing pass; its items join the
    // tree on the next.
    let (_, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let (ids, _) = window.frame_at_ids(None, vec![], |ui| {
        crate::test_support::tactics_settings(
            ui,
            "tactics",
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
    });
    let item = entry_id(&ids, Role::Button, "Deep Defensive Line");
    let (_, _) = window.settings(
        version,
        &mut tactics,
        &mut view,
        &roster,
        vec![egui::Event::AccessKitActionRequest(
            accesskit::ActionRequest {
                action: accesskit::Action::Click,
                target_tree: accesskit::TreeId::ROOT,
                target_node: item,
                data: None,
            },
        )],
    );
    assert_eq!(
        tactics.presets[0]
            .defence_instructions
            .as_ref()
            .expect("instructions")[0]
            .instruction,
        instruction::encode(version, Instruction::DeepDefensiveLine).expect("PES 21 stores it"),
        "the pick changed the instruction while armed"
    );
    assert!(view.assign.is_some(), "the mode stays armed");
    // The marker click writes nothing (the instruction takes no
    // target) and disarms.
    let (nodes, _) = window.pitch(version, &mut tactics, &mut view, &roster, vec![]);
    let at = marker_point(&nodes, &formation.players, 3);
    let (_, changed) = window.click_pitch(at, version, &mut tactics, &mut view, &roster);
    assert!(!changed, "a target-less instruction writes nothing");
    assert_eq!(
        tactics.presets[0]
            .defence_instructions
            .as_ref()
            .expect("instructions")[0]
            .player,
        u8::MAX,
        "the player byte is untouched"
    );
    assert_eq!(view.assign, None, "and the mode disarms");
}

// The popover's slot is of the preset it opened on: a preset switch
// closes it (a formation switch is
// `the_popover_closes_when_the_formation_changes`'s case).
#[test]
fn a_preset_switch_closes_the_popover() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let formation = tactics.presets[0].formations[0].clone();
    let index = 3;

    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let at = marker_point(&nodes, &formation.players, index);
    window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    entry(&nodes, Role::Button, "Close");

    view.preset = 1;
    window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    assert_eq!(view.popover, None, "a preset switch closes the popover");
}

// The popover's Position combo writes the picked position byte to
// the shown formation's slot alone.
#[test]
fn the_popovers_position_pick_writes_the_slot() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let before = tactics.clone();
    let formation = before.presets[0].formations[0].clone();
    let index = 3;
    let cf = POSITIONS
        .iter()
        .position(|label| *label == "CF")
        .expect("CF is a position") as u8;
    assert_ne!(
        formation.players[index].position, cf,
        "premise: a real change"
    );

    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let at = marker_point(&nodes, &formation.players, index);
    window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let combo = controls_right_of(&nodes, "Position", Role::ComboBox)
        .into_iter()
        .next()
        .expect("the position combo");
    window.click_pitch(
        centre(combo),
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    // The item pick goes through AccessKit, as the instruction
    // combos' do: a pointer click does not reach a combo's popup.
    let ids = window.pitch_ids(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let item = entry_id(&ids, Role::Button, "CF");
    let (_, changed) =
        window.action_click_pitch(item, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    assert!(changed, "the pick changed the record");
    assert_eq!(
        tactics.presets[0].formations[0].players[index].position, cf,
        "the pick wrote CF"
    );
    assert_eq!(
        tactics.presets[0].formations[1..],
        before.presets[0].formations[1..],
        "the other two formations untouched"
    );
}
