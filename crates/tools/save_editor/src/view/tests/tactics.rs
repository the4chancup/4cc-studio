use super::*;

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

// The same wrap on the Automatic settings row and the armed mode's note:
// the Substitution pair, the three checkboxes and the note's button all
// land inside the width (the row fits 800 either way; the wrap itself is
// `the_automatic_settings_and_the_note_wrap_at_a_narrow_pane`'s case,
// inside the widget's own pane where a narrower width reaches it).
#[test]
fn the_automatic_settings_and_the_note_wrap_inside_a_narrow_window() {
    let path = fixture(PesVersion::Pes21, "tactics_wrap2");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(800.0, 1400.0);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let substitution = control_right_of(&nodes, "Substitution");
    assert!(
        substitution.bounds().expect("field bounds").x1 <= 800.0,
        "the Substitution field inside the window's width"
    );
    for label in ["Offside trap", "Preset change", "Attack/defence levels"] {
        let checkbox = node_labeled(&nodes, Role::CheckBox, label);
        assert!(
            checkbox.bounds().expect("checkbox bounds").x1 <= 800.0,
            "{label} inside the window's width"
        );
    }

    // Arm join: the note draws its longer line; its button lands inside.
    click(&mut window, chip(&nodes, "Join attack:"), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let done = node_labeled(&nodes, Role::Button, "Done");
    assert!(
        done.bounds().expect("button bounds").x1 <= 800.0,
        "the note's button inside the window's width"
    );
}
