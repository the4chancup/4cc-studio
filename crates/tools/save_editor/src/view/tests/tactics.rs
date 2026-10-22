use pes_savefile::model::TeamEntry;

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

// SE-GUI-12's bench half (the scenario's citation waits for the pitch:
// its drag, the role palette and the join-attack mode), and
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
