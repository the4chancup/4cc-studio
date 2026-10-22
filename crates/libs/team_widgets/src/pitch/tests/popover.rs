//! The slot popover's tests and the armed assign modes': the open
//! and close cases, the fields' writes, the role chips and join.

use egui::accesskit::Role;
use pes_version::PesVersion;

use super::*;
use crate::test_support::{
    Headless, centre, controls_right_of, entry, entry_id, fixture_rows, fixture_tactics,
    roster_rows,
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
    let starter = tactics.starting_eleven[index];
    assert_ne!(
        tactics.set_pieces.penalty, starter,
        "premise: the marker's player is not already the taker"
    );

    // Arm the Penalty kick chip.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Penalty kick:"))
        })
        .expect("the penalty chip");
    let (_, changed) =
        window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert!(!changed, "arming writes no record");
    assert_eq!(view.assign, Some(Assign::Role(5)), "the mode is armed");

    // The pitch draws the note; the marker click writes the taker.
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Click the pitch player for penalty kick.")),
        "the armed note is drawn"
    );
    entry(&nodes, Role::Button, "Cancel");
    let canvas = canvas_rect(&nodes);
    let formation = tactics.presets[0].formations[0].clone();
    let at = marker_rect(
        pitch_point(
            canvas.min,
            formation.players[index].x,
            formation.players[index].y,
        ),
        canvas,
    )
    .center();
    let (_, changed) = window.click_pitch(at, &mut tactics, &mut view, &roster);
    assert!(changed, "the assignment writes the record");
    assert_eq!(
        tactics.set_pieces.penalty, starter,
        "the marker's player takes it"
    );
    assert_eq!(view.assign, None, "and the mode disarms");
}

// The join pick toggles members until Done: three clicks fill the
// three entries, a fourth is ignored, and clicking a member removes
// him, packing the list (the plan's "Players to join the attack stay
// packed").
#[test]
fn join_clicks_fill_three_then_toggle_members_packed() {
    let mut window = Headless::sized(900.0, 1100.0);
    let version = PesVersion::Pes21;
    let mut tactics = fixture_tactics(version);
    tactics.players_to_join_attack = [u8::MAX; 3];
    let mut view = TacticsView::default();
    let rows = fixture_rows(version);
    let roster = roster_rows(&rows);

    // Arm the join chip.
    let (nodes, _) = window.settings(version, &mut tactics, &mut view, &roster, vec![]);
    let chip = nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Join attack:"))
        })
        .expect("the join chip");
    window.click_settings(centre(chip), version, &mut tactics, &mut view, &roster);
    assert_eq!(view.assign, Some(Assign::Join), "the mode is armed");

    // Three marker clicks fill the list, packed.
    let starters = tactics.starting_eleven;
    let click_marker =
        |window: &mut Headless, tactics: &mut TeamTactics, view: &mut TacticsView, index: usize| {
            let (nodes, _) = window.pitch(tactics, view, &roster, vec![]);
            let canvas = canvas_rect(&nodes);
            let slot = tactics.presets[0].formations[0].players[index];
            let at = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas).center();
            window.click_pitch(at, tactics, view, &roster)
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

    // Done disarms.
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let done = entry(&nodes, Role::Button, "Done");
    window.click_pitch(centre(done), &mut tactics, &mut view, &roster);
    assert_eq!(view.assign, None, "Done disarms the mode");
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

    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let at = marker_centre(&nodes, formation.players[index]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    entry(&nodes, Role::Button, "Close");

    view.preset = 1;
    window.pitch(&mut tactics, &mut view, &roster, vec![]);
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

    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let at = marker_centre(&nodes, formation.players[index]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let combo = controls_right_of(&nodes, "Position", Role::ComboBox)
        .into_iter()
        .next()
        .expect("the position combo");
    window.click_pitch(centre(combo), &mut tactics, &mut view, &roster);
    // The item pick goes through AccessKit, as the instruction
    // combos' do: a pointer click does not reach a combo's popup.
    let ids = window.pitch_ids(&mut tactics, &mut view, &roster);
    let item = entry_id(&ids, Role::Button, "CF");
    let (_, changed) = window.action_click_pitch(item, &mut tactics, &mut view, &roster);
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
