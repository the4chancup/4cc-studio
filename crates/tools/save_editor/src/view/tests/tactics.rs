use pes_savefile::model::TeamEntry;
use pes_savefile::model::tactics::FormationSlot;

use super::*;

/// The team's entry as the session holds it.
fn team_entry(view: &SaveView, team_id: u32) -> TeamEntry {
    view.session
        .as_ref()
        .unwrap()
        .file()
        .team(team_id)
        .unwrap()
        .clone()
}

/// The node of `role` labeled `label`.
fn node_labeled<'a>(nodes: &'a [accesskit::Node], role: Role, label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {role:?} labeled {label:?}"))
}

/// The lineup rows' name labels in draw order (the 11 starting-eleven
/// rows, then the bench rows): each row's last label is its name, found
/// through the ≡ handles since the fixture's players share names.
fn lineup_names(nodes: &[accesskit::Node]) -> Vec<&accesskit::Node> {
    let mut handles: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::Label && node.value() == Some("≡"))
        .collect();
    handles.sort_by(|a, b| {
        let key = |node: &&accesskit::Node| {
            let bounds = node.bounds().expect("handle bounds");
            (bounds.y0, bounds.x0)
        };
        key(a).partial_cmp(&key(b)).expect("ordered bounds")
    });
    handles
        .iter()
        .map(|handle| {
            let bounds = handle.bounds().expect("handle bounds");
            nodes
                .iter()
                .filter(|node| {
                    node.role() == Role::Label
                        && node
                            .bounds()
                            .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 > bounds.x0)
                })
                .max_by(|a, b| {
                    let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
                    x(a).partial_cmp(&x(b)).expect("ordered bounds")
                })
                .unwrap_or_else(|| panic!("no name on a lineup row"))
        })
        .collect()
}

/// The open menu's items (labels starting "Copy " or "Swap "), in draw
/// order.
fn menu_items(nodes: &[accesskit::Node]) -> Vec<String> {
    let mut items: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Copy ") || label.starts_with("Swap "))
        })
        .collect();
    items.sort_by(|a, b| {
        let key = |node: &&accesskit::Node| {
            let bounds = node.bounds().expect("item bounds");
            (bounds.y0, bounds.x0)
        };
        key(a).partial_cmp(&key(b)).expect("ordered bounds")
    });
    items
        .iter()
        .filter_map(|node| node.label().map(str::to_owned))
        .collect()
}

// SE-GUI-25
#[test]
fn the_tactics_menus_copy_presets_and_swap_formations_as_undoable_edits() {
    let path = fixture(PesVersion::Pes21, "tactics_copy_swap");
    // Distinct sliders per preset and coordinates per formation, so the
    // copy and the swap are real changes (the fixture's are placeholders).
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for (preset, value) in [(0, 7), (1, 5), (2, 1)] {
        file.team_mut(701).unwrap().tactics.presets[preset]
            .sliders
            .support_range = value;
    }
    for (formation, x) in [(0, 10), (1, 20), (2, 30)] {
        file.team_mut(701).unwrap().tactics.presets[0].formations[formation].players[0].x = x;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1100.0, 900.0);
    let preset1_before = team_entry(&view, 701).tactics.presets[0].clone();
    let formations_before = team_entry(&view, 701).tactics.presets[0].formations.clone();
    let preset2_before = team_entry(&view, 701).tactics.presets[1].clone();
    assert_ne!(
        preset1_before, preset2_before,
        "premise: presets 1 and 2 differ"
    );

    // Copy preset 1 to 2, from the tab's Copy/swap preset menu.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy/swap preset"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy preset 1 to 2"),
        &mut view,
    );
    assert_eq!(
        team_entry(&view, 701).tactics.presets[1],
        preset1_before,
        "preset 2 equals preset 1 as it was"
    );

    // Swap Kick-off and In-possession, from the Copy/swap formation menu
    // (preset 1 is on screen).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy/swap formation"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Swap Kick-off and In-possession"),
        &mut view,
    );
    let formations = &team_entry(&view, 701).tactics.presets[0].formations;
    assert_eq!(
        formations[0], formations_before[1],
        "in-possession took kick-off's place"
    );
    assert_eq!(
        formations[1], formations_before[0],
        "kick-off took in-possession's"
    );
    assert_eq!(
        formations[2], formations_before[2],
        "out-of-possession untouched"
    );
    // Preset 2 still holds preset 1 as it was before the swap.
    assert_eq!(team_entry(&view, 701).tactics.presets[1], preset1_before);

    // Two undo entries: the first restores the formations, the second
    // preset 2.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0].formations,
        formations_before,
        "one undo restores the formations"
    );
    assert_eq!(
        team_entry(&view, 701).tactics.presets[1],
        preset1_before,
        "preset 2 is still the copy after one undo"
    );
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).tactics.presets[1],
        preset2_before,
        "a second undo restores preset 2"
    );
    assert!(
        !view.session.as_ref().unwrap().can_undo(),
        "exactly two entries"
    );

    // Redo both, save, and the reopened file holds the result.
    window.frame(
        vec![key(
            egui::Key::Z,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            true,
        )],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(
        vec![key(
            egui::Key::Z,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            true,
        )],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    let tactics = &reopened.team(701).unwrap().tactics;
    assert_eq!(tactics.presets[1], preset1_before, "preset 2 saved");
    assert_eq!(
        tactics.presets[0].formations[0], formations_before[1],
        "the swap saved"
    );
    assert_eq!(tactics.presets[0].formations[1], formations_before[0]);
}

// plans/save_editor.md "Editing UI", the Undo/redo bullet (one entry per
// gesture) on this tab: a slider drag lands as one undoable edit and the
// team is marked changed.
#[test]
fn a_slider_drag_on_the_tab_is_one_undo_entry_and_marks_the_team() {
    let path = fixture(PesVersion::Pes21, "tactics_slider_drag");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1100.0, 900.0);
    let before = team_entry(&view, 701).tactics.presets[0].sliders.clone();

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let slider = control_right_of(&nodes, "Support range");
    let bounds = slider.bounds().expect("slider bounds");
    let from = centre(slider);
    let to = egui::pos2((bounds.x1 - 4.0) as f32, from.y);
    drag(&mut window, from, to, &mut view);

    assert_ne!(
        team_entry(&view, 701).tactics.presets[0].sliders,
        before,
        "the drag changed preset 1's sliders"
    );
    assert!(
        view.session.as_ref().unwrap().is_dirty(),
        "the team is marked changed"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the whole drag was one undo entry"
    );
}

// The preset selector writes `TacticsView`, not the record: switching
// presets leaves the session clean (no undo entry, no unsaved dot).
#[test]
fn the_preset_selector_writes_nothing() {
    let path = fixture(PesVersion::Pes21, "tactics_preset_select");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1100.0, 900.0);
    assert_eq!(view.tactics.preset, 0, "preset 1 is on screen first");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "2"),
        &mut view,
    );
    assert_eq!(view.tactics.preset, 1, "preset 2 is on screen now");
    let session = view.session.as_ref().unwrap();
    assert!(!session.is_dirty(), "the selector wrote nothing");
    assert!(!session.can_undo(), "and pushed no undo entry");

    // And back to preset 1: still nothing.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "1"),
        &mut view,
    );
    assert_eq!(view.tactics.preset, 0);
    let session = view.session.as_ref().unwrap();
    assert!(!session.is_dirty());
    assert!(!session.can_undo());
}

// plans/save_editor.md "Tactics editor", "A starter swapped with a bench
// player": the roles follow the swap, one undo entry, saved.
#[test]
fn a_bench_player_dragged_onto_a_starter_swaps_them_as_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "tactics_lineup_swap");
    // The starter the drag replaces is the penalty taker, so the role has
    // to follow the swap.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    let starter_slot = file.team(701).unwrap().tactics.starting_eleven[8];
    let bench_slot = file.team(701).unwrap().tactics.bench_order[0];
    assert_ne!(starter_slot, bench_slot, "premise: two different slots");
    file.team_mut(701).unwrap().tactics.set_pieces.penalty = starter_slot;
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1100.0, 1400.0);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let names = lineup_names(&nodes);
    assert_eq!(names.len(), 32, "the eleven and the bench rows");
    drag(&mut window, centre(names[11]), centre(names[8]), &mut view);

    let tactics = &team_entry(&view, 701).tactics;
    assert_eq!(
        tactics.starting_eleven[8], bench_slot,
        "the bench player starts"
    );
    assert_eq!(tactics.bench_order[0], starter_slot, "the starter sits");
    assert_eq!(
        tactics.set_pieces.penalty, bench_slot,
        "the taker followed the swap"
    );

    // One undo entry restores both.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let tactics = &team_entry(&view, 701).tactics;
    assert_eq!(tactics.starting_eleven[8], starter_slot, "the eleven back");
    assert_eq!(tactics.bench_order[0], bench_slot, "the bench back");
    assert_eq!(tactics.set_pieces.penalty, starter_slot, "the taker back");
    assert!(
        !view.session.as_ref().unwrap().can_undo(),
        "the whole drop was one undo entry"
    );

    // Redo and save: the reopened file holds the swap and the moved taker.
    window.frame(
        vec![key(
            egui::Key::Z,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            true,
        )],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    let tactics = &reopened.team(701).unwrap().tactics;
    assert_eq!(tactics.starting_eleven[8], bench_slot, "the swap saved");
    assert_eq!(tactics.bench_order[0], starter_slot, "the bench saved");
    assert_eq!(tactics.set_pieces.penalty, bench_slot, "the taker saved");
}

// plans/save_editor.md "Tactics editor" (the copy/swap menus, the draft's
// `formationMenu` and `presetMenu`): each menu's items are the six copies
// then three swaps, in its order; a copy of the shown formation is one
// undoable edit.
#[test]
fn the_copy_swap_menus_items_and_the_formation_copy() {
    let path = fixture(PesVersion::Pes21, "tactics_menu_items");
    // Distinct coordinates per formation, so the copy is a real change.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for (formation, x) in [(0, 10), (1, 20), (2, 30)] {
        file.team_mut(701).unwrap().tactics.presets[0].formations[formation].players[0].x = x;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1100.0, 900.0);
    let formations_before = team_entry(&view, 701).tactics.presets[0].formations.clone();
    assert_ne!(
        formations_before[0], formations_before[2],
        "premise: kick-off and out-of-possession differ"
    );

    // The Copy/swap formation menu: six copies, then three swaps.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy/swap formation"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        menu_items(&nodes),
        [
            "Copy Kick-off to In-possession",
            "Copy Kick-off to Out-of-possession",
            "Copy In-possession to Kick-off",
            "Copy In-possession to Out-of-possession",
            "Copy Out-of-possession to Kick-off",
            "Copy Out-of-possession to In-possession",
            "Swap Kick-off and In-possession",
            "Swap Kick-off and Out-of-possession",
            "Swap In-possession and Out-of-possession",
        ],
        "the formation menu's items in the draft's order"
    );

    // Copy Out-of-possession to Kick-off: preset 1's kick-off becomes its
    // out-of-possession, in one undo entry.
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy Out-of-possession to Kick-off"),
        &mut view,
    );
    let formations = &team_entry(&view, 701).tactics.presets[0].formations;
    assert_eq!(
        formations[0], formations_before[2],
        "kick-off is now out-of-possession's"
    );
    assert_eq!(
        formations[1], formations_before[1],
        "in-possession untouched"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the copy was one undo entry"
    );

    // The Copy/swap preset menu: six copies, then three swaps.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy/swap preset"),
        &mut view,
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        menu_items(&nodes),
        [
            "Copy preset 1 to 2",
            "Copy preset 1 to 3",
            "Copy preset 2 to 1",
            "Copy preset 2 to 3",
            "Copy preset 3 to 1",
            "Copy preset 3 to 2",
            "Swap presets 1 and 2",
            "Swap presets 1 and 3",
            "Swap presets 2 and 3",
        ],
        "the preset menu's items in the draft's order"
    );
}

/// The pitch canvas' rect (the widget's "pitch" node).
fn canvas_of(nodes: &[accesskit::Node]) -> accesskit::Rect {
    nodes
        .iter()
        .find(|node| node.label() == Some("pitch"))
        .and_then(accesskit::Node::bounds)
        .expect("the pitch canvas")
}

/// The centre of `slot`'s marker on `canvas` (the widget's marker math:
/// the stored grid is 104 by 48 at 3 px and 7 px a unit inside a 4 px / 2
/// px margin, high y on top; the 42 by 26 chip centres on the point and
/// clamps 2 px inside the canvas).
fn marker_centre(canvas: accesskit::Rect, slot: FormationSlot) -> egui::Pos2 {
    let centre = egui::pos2(
        canvas.x0 as f32 + 4.0 + f32::from(slot.x) * 3.0,
        canvas.y0 as f32 + 2.0 + (48.0 - f32::from(slot.y)) * 7.0,
    );
    egui::pos2(
        (centre.x - 21.0).clamp(canvas.x0 as f32 + 2.0, canvas.x1 as f32 - 44.0) + 21.0,
        (centre.y - 13.0).clamp(canvas.y0 as f32 + 2.0, canvas.y1 as f32 - 30.0) + 13.0,
    )
}

/// A point on `canvas` covered by no marker of `players` (a grass drop:
/// the first free of the grid scan).
fn grass_point(canvas: accesskit::Rect, players: &[FormationSlot; 11]) -> egui::Pos2 {
    let covered = |x: f32, y: f32| {
        players.iter().any(|slot| {
            let centre = marker_centre(canvas, *slot);
            (centre.x - x).abs() <= 21.0 && (centre.y - y).abs() <= 13.0
        })
    };
    for x in 0..=104_u8 {
        for y in 0..=48_u8 {
            let (px, py) = (
                canvas.x0 as f32 + 4.0 + f32::from(x) * 3.0,
                canvas.y0 as f32 + 2.0 + (48.0 - f32::from(y)) * 7.0,
            );
            if !covered(px, py) {
                return egui::pos2(px, py);
            }
        }
    }
    panic!("no free point on the canvas")
}

/// `pos` inside `canvas` snapped to the stored grid (the widget's snap:
/// the nearest unit, clamped).
fn snap(canvas: accesskit::Rect, pos: egui::Pos2) -> (u8, u8) {
    let x = ((pos.x - canvas.x0 as f32 - 4.0) / 3.0)
        .round()
        .clamp(0.0, 104.0) as u8;
    let y = (48.0 - ((pos.y - canvas.y0 as f32 - 2.0) / 7.0).round()).clamp(0.0, 48.0) as u8;
    (x, y)
}

/// A click at `pos` (press on one frame, release on the next): the
/// release frame's nodes.
fn click_at(window: &mut Headless, pos: egui::Pos2, view: &mut SaveView) -> Vec<accesskit::Node> {
    window.frame(pointer_at(pos, true), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(pointer_at(pos, false), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    nodes
}

/// The chip of the "Player assignments" palette labeled with `prefix`.
fn chip<'a>(nodes: &'a [accesskit::Node], prefix: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| {
            node.role() == Role::Button
                && node.label().is_some_and(|label| label.starts_with(prefix))
        })
        .unwrap_or_else(|| panic!("no chip {prefix:?}"))
}

// SE-GUI-12
#[test]
fn the_pitch_moves_assigns_and_swaps_on_the_tactics_tab() {
    let path = fixture(PesVersion::Pes21, "tactics_pitch");
    // The join-attack list starts empty so the scenario's clicks alone
    // fill it, and the eleven is shuffled so a pick's roster slot never
    // equals its position index (an index-writing arm would look right
    // on the fixture's [0, 1, ...]).
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.team_mut(701).unwrap().tactics.players_to_join_attack = [u8::MAX; 3];
    file.team_mut(701)
        .unwrap()
        .tactics
        .starting_eleven
        .swap(5, 7);
    file.team_mut(701)
        .unwrap()
        .tactics
        .starting_eleven
        .swap(6, 10);
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1600.0, 1400.0);
    let index = 4;
    let before = team_entry(&view, 701).tactics;
    let slot_before = before.presets[0].formations[0].players[index];

    // A pitch player dragged to another spot: the slot's x and y are the
    // snapped drop point, undoable.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let canvas = canvas_of(&nodes);
    let from = marker_centre(canvas, slot_before);
    let onto = grass_point(canvas, &before.presets[0].formations[0].players);
    let (x, y) = snap(canvas, onto);
    assert_ne!(
        (slot_before.x, slot_before.y),
        (x, y),
        "premise: a real move"
    );
    drag(&mut window, from, onto, &mut view);
    let written = team_entry(&view, 701).tactics.presets[0].formations[0].players[index];
    assert_eq!((written.x, written.y), (x, y), "the snapped drop point");

    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let undone = team_entry(&view, 701).tactics.presets[0].formations[0].players[index];
    assert_eq!(
        (undone.x, undone.y),
        (slot_before.x, slot_before.y),
        "undo restored"
    );
    window.frame(
        vec![key(
            egui::Key::Z,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            true,
        )],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let redone = team_entry(&view, 701).tactics.presets[0].formations[0].players[index];
    assert_eq!((redone.x, redone.y), (x, y), "redo moved it back");

    // Saved and reopened: the file holds the new spot.
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    let saved = reopened.team(701).unwrap().tactics.presets[0].formations[0].players[index];
    assert_eq!((saved.x, saved.y), (x, y), "the file holds the drop point");

    // A bench player dragged onto a pitch player: the two swap.
    let out = team_entry(&view, 701).tactics.starting_eleven[index];
    let inn = team_entry(&view, 701).tactics.bench_order[0];
    assert_ne!(out, inn, "premise: two different slots");
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let canvas = canvas_of(&nodes);
    let moved = team_entry(&view, 701).tactics.presets[0].formations[0].players[index];
    let onto = marker_centre(canvas, moved);
    let names = lineup_names(&nodes);
    drag(&mut window, centre(names[11]), onto, &mut view);
    let tactics = team_entry(&view, 701).tactics;
    assert_eq!(
        tactics.starting_eleven[index], inn,
        "the bench player starts"
    );
    assert_eq!(tactics.bench_order[0], out, "the starter sits");

    // A set-piece role assigned from the palette: arm Penalty kick, click
    // a pitch player; the taker field holds him.
    let assignee = 7;
    let starter = team_entry(&view, 701).tactics.starting_eleven[assignee];
    assert_ne!(
        starter, assignee as u8,
        "premise: the picked position's slot differs from its index"
    );
    assert_ne!(
        tactics.set_pieces.penalty, starter,
        "premise: the marker's player is not already the taker"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, chip(&nodes, "Penalty kick:"), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let canvas = canvas_of(&nodes);
    let slot = team_entry(&view, 701).tactics.presets[0].formations[0].players[assignee];
    click_at(&mut window, marker_centre(canvas, slot), &mut view);
    assert_eq!(
        team_entry(&view, 701).tactics.set_pieces.penalty,
        starter,
        "the taker field holds the clicked player"
    );

    // Join mode with two pitch players clicked: they are the setting's
    // players, the third entry 255.
    let members = [5_usize, 6_usize].map(|i| team_entry(&view, 701).tactics.starting_eleven[i]);
    assert!(
        members.iter().enumerate().all(|(i, s)| *s != (i + 5) as u8),
        "premise: the clicked positions' slots differ from their indices"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, chip(&nodes, "Join attack:"), &mut view);
    for member in [5_usize, 6_usize] {
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let canvas = canvas_of(&nodes);
        let slot = team_entry(&view, 701).tactics.presets[0].formations[0].players[member];
        click_at(&mut window, marker_centre(canvas, slot), &mut view);
    }
    assert_eq!(
        team_entry(&view, 701).tactics.players_to_join_attack,
        [members[0], members[1], u8::MAX],
        "the two clicked players are the setting's players"
    );
}

// plans/save_editor.md "Editing UI", the Undo/redo bullet ("changes from
// one widget merge into the open entry"): a popover X typed into, then a
// style pair clicked in the same unsealed span, is two undo steps — the
// pitch's and the settings group's own.
#[test]
fn a_pitch_edit_and_a_settings_click_undo_as_two_entries() {
    let path = fixture(PesVersion::Pes21, "tactics_undo_keys");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1600.0, 1400.0);
    let before = team_entry(&view, 701).tactics;
    let players = before.presets[0].formations[0].players;
    assert!(
        !before.presets[0].style.attacking_style,
        "premise: a real change"
    );

    // The popover on an outfield marker no other marker covers (a covered
    // centre would open a later marker's), the X field focused, "55"
    // typed (no Enter: the field keeps the keyboard, the span unsealed).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let canvas = canvas_of(&nodes);
    let covered = |centre: egui::Pos2, slot: &FormationSlot| {
        let other = marker_centre(canvas, *slot);
        (other.x - centre.x).abs() <= 21.0 && (other.y - centre.y).abs() <= 13.0
    };
    let index = (0..11)
        .find(|&index| {
            players[index].position != 0
                && players.iter().enumerate().all(|(other, slot)| {
                    other == index || !covered(marker_centre(canvas, players[index]), slot)
                })
        })
        .expect("a marker no other marker covers");
    let slot_before = players[index];
    click_at(&mut window, marker_centre(canvas, slot_before), &mut view);
    let (_, ids, _) = window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let x_label = ids
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.value() == Some("X"))
        .and_then(|(_, node)| node.bounds())
        .expect("the X label");
    let mut fields: Vec<accesskit::NodeId> = ids
        .iter()
        .filter(|(_, node)| {
            node.role() == Role::SpinButton
                && node.bounds().is_some_and(|bounds| {
                    (bounds.y0 - x_label.y0).abs() < 4.0 && bounds.x0 >= x_label.x1 - 1.0
                })
        })
        .map(|(id, _)| *id)
        .collect();
    fields.sort_by(|a, b| {
        let bounds = |id: &accesskit::NodeId| {
            ids.iter()
                .find(|(other, _)| other == id)
                .and_then(|(_, node)| node.bounds())
                .expect("bounds")
                .x0
        };
        bounds(a).partial_cmp(&bounds(b)).expect("ordered bounds")
    });
    let field_id = *fields.first().expect("the X field's id");
    action_focus(&mut window, &mut view, field_id);
    window.frame(vec![egui::Event::Text("55".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0].formations[0].players[index].x,
        55,
        "the typed x is written"
    );

    // A style pair click in the same span.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Possession"),
        &mut view,
    );
    assert!(
        team_entry(&view, 701).tactics.presets[0]
            .style
            .attacking_style,
        "the click wrote"
    );

    // One undo each: the settings click first, then the pitch's X.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let tactics = &team_entry(&view, 701).tactics;
    assert!(
        !tactics.presets[0].style.attacking_style,
        "the style click reverted"
    );
    assert_eq!(
        tactics.presets[0].formations[0].players[index].x, 55,
        "the pitch edit still applied"
    );
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0].formations[0].players[index].x,
        slot_before.x,
        "the X edit reverted"
    );
}

// The draft's `.row` (flex-wrap) on the Formation settings cells: the
// host scrolls only vertically, so at a narrow window every slider and
// every style choice must land inside the width, the cells wrapping.
#[test]
fn the_formation_settings_wrap_inside_a_narrow_window() {
    let path = fixture(PesVersion::Pes21, "tactics_wrap");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(800.0, 1400.0);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let sliders: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::Slider)
        .collect();
    assert_eq!(sliders.len(), 5, "the five sliders are drawn");
    for slider in sliders {
        assert!(
            slider.bounds().expect("slider bounds").x1 <= 800.0,
            "a slider inside the window's width"
        );
    }
    for label in [
        "Counter Attack",
        "Possession",
        "Centre",
        "Long Pass",
        "Short Pass",
        "Maintain",
        "Flexible",
        "Frontline Pressure",
        "All Out Defence",
        "Middle",
        "Aggressive",
        "Conservative",
    ] {
        let choice = node_labeled(&nodes, Role::Button, label);
        assert!(
            choice.bounds().expect("choice bounds").x1 <= 800.0,
            "{label} inside the window's width"
        );
    }
    let wides: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::Button && node.label() == Some("Wide"))
        .collect();
    assert_eq!(
        wides.len(),
        2,
        "the attacking zone's and the containment area's Wide"
    );
    for wide in wides {
        assert!(
            wide.bounds().expect("choice bounds").x1 <= 800.0,
            "Wide inside the window's width"
        );
    }
}

// The Undo/redo bullet's "one entry per gesture" where the same widget
// key covers two gestures: a typed popover X then a Make GK click (the
// click's press keeps the span open, and both writes are the pitch's)
// are two steps — a gesture ends when its widget loses the keyboard.
#[test]
fn a_typed_popover_edit_then_make_gk_undo_as_two_entries() {
    let path = fixture(PesVersion::Pes21, "tactics_undo_focus");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(1600.0, 1400.0);
    let before = team_entry(&view, 701).tactics;
    let players = before.presets[0].formations[0].players;
    let keeper = players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");

    // The popover on an outfield marker no other marker covers (a covered
    // centre would open a later marker's), the X field focused, "55"
    // typed (no Enter: the field keeps the keyboard, the span unsealed).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let canvas = canvas_of(&nodes);
    let covered = |centre: egui::Pos2, slot: &FormationSlot| {
        let other = marker_centre(canvas, *slot);
        (other.x - centre.x).abs() <= 21.0 && (other.y - centre.y).abs() <= 13.0
    };
    let index = (0..11)
        .find(|&index| {
            players[index].position != 0
                && players.iter().enumerate().all(|(other, slot)| {
                    other == index || !covered(marker_centre(canvas, players[index]), slot)
                })
        })
        .expect("a marker no other marker covers");
    assert_ne!(index, keeper, "premise: not the keeper's slot");
    let slot_before = players[index];
    click_at(&mut window, marker_centre(canvas, slot_before), &mut view);
    let (_, ids, _) = window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let x_label = ids
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.value() == Some("X"))
        .and_then(|(_, node)| node.bounds())
        .expect("the X label");
    let mut fields: Vec<accesskit::NodeId> = ids
        .iter()
        .filter(|(_, node)| {
            node.role() == Role::SpinButton
                && node.bounds().is_some_and(|bounds| {
                    (bounds.y0 - x_label.y0).abs() < 4.0 && bounds.x0 >= x_label.x1 - 1.0
                })
        })
        .map(|(id, _)| *id)
        .collect();
    fields.sort_by(|a, b| {
        let bounds = |id: &accesskit::NodeId| {
            ids.iter()
                .find(|(other, _)| other == id)
                .and_then(|(_, node)| node.bounds())
                .expect("bounds")
                .x0
        };
        bounds(a).partial_cmp(&bounds(b)).expect("ordered bounds")
    });
    let field_id = *fields.first().expect("the X field's id");
    action_focus(&mut window, &mut view, field_id);
    window.frame(vec![egui::Event::Text("55".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0].formations[0].players[index].x,
        55,
        "the typed x is written"
    );

    // Make GK in the same popover: the keeper swap is its own gesture.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Make GK"),
        &mut view,
    );
    let tactics = team_entry(&view, 701).tactics;
    assert_eq!(
        tactics.starting_eleven[index], before.starting_eleven[keeper],
        "he takes the goal"
    );
    assert_eq!(
        tactics.starting_eleven[keeper], before.starting_eleven[index],
        "the keeper takes his slot"
    );

    // One Ctrl+Z reverts the keeper swap alone; the X still applied.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let tactics = team_entry(&view, 701).tactics;
    assert_eq!(
        tactics.starting_eleven[index], before.starting_eleven[index],
        "one undo reverts the keeper swap"
    );
    assert_eq!(
        tactics.starting_eleven[keeper], before.starting_eleven[keeper],
        "and the keeper's slot"
    );
    assert_eq!(
        tactics.presets[0].formations[0].players[index].x, 55,
        "the X edit still applied"
    );
    // A second reverts the typed X.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0].formations[0].players[index].x,
        slot_before.x,
        "a second undo reverts the X edit"
    );
    assert!(
        !view.session.as_ref().unwrap().can_undo(),
        "exactly two entries"
    );
}
