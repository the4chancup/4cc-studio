use pes_savefile::model::instruction::Instruction;

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
    let from = marker_point(canvas, &before.presets[0].formations[0].players, index)
        .expect("a point on the marker a press can reach");
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
    let onto = marker_point(
        canvas,
        &team_entry(&view, 701).tactics.presets[0].formations[0].players,
        index,
    )
    .expect("a point on the marker a drop can reach");
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
    let at = marker_point(
        canvas,
        &team_entry(&view, 701).tactics.presets[0].formations[0].players,
        assignee,
    )
    .expect("a point on the marker a click can reach");
    click_at(&mut window, at, &mut view);
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
        let at = marker_point(
            canvas,
            &team_entry(&view, 701).tactics.presets[0].formations[0].players,
            member,
        )
        .expect("a point on the marker a click can reach");
        click_at(&mut window, at, &mut view);
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

// plans/save_editor.md "Tactics editor" (the "pitch is the editor"
// bullet's three columns): at the default pane (1224 x 776) the settings
// column sits beside the lists — the Formation settings heading right of
// the lists column's 200, its top beside the canvas — the 456-point
// column holds the preset bar on one line and the formation grid one
// pair a row (labels and controls each sharing one x0), every control
// inside the pane.
#[test]
fn the_tactics_tab_is_three_columns_at_the_default_pane() {
    let path = fixture(PesVersion::Pes21, "tactics_three_columns");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let settings = label_bounds(&nodes, "Formation settings");
    let bench = label_bounds(&nodes, "Bench");
    let canvas = canvas_of(&nodes);
    assert!(
        settings.x0 > bench.x0 + 200.0,
        "the settings column right of the lists' 200 (settings {settings:?}, bench {bench:?})"
    );
    assert!(
        settings.y0 < canvas.y1,
        "its top beside the canvas (settings {settings:?}, canvas {canvas:?})"
    );

    // The formation grid at the column's width: one pair a row — the
    // twelve labels share one x0 and each pair's control one x0.
    let labels_x0 = label_bounds(&nodes, "Support range").x0;
    let controls_x0 = control_right_of(&nodes, "Support range")
        .bounds()
        .expect("bounds")
        .x0;
    for label in [
        "Defensive line",
        "Compactness",
        "Numbers in attack",
        "Numbers in defence",
        "Attacking style",
        "Attacking zone",
        "Buildup",
        "Positioning",
        "Defensive style",
        "Containment area",
        "Pressure",
    ] {
        let label_bounds = label_bounds(&nodes, label).x0;
        assert_eq!(label_bounds, labels_x0, "{label} shares the labels' x0");
        assert_eq!(
            control_right_of(&nodes, label).bounds().expect("bounds").x0,
            controls_x0,
            "{label}'s control shares the controls' x0"
        );
    }

    // The preset bar on one line at this pane (it wraps only at the
    // narrowest three-column pane, the sibling test's): the six bar
    // widgets share one row's top.
    let bar_y = label_bounds(&nodes, "Preset").y0;
    for label in ["1", "2", "3", "Copy/swap preset", "Copy/swap formation"] {
        let button = node_labeled(&nodes, Role::Button, label);
        assert!(
            (button.bounds().expect("bounds").y0 - bar_y).abs() < 0.5,
            "{label} shares the bar's one line ({:?})",
            button.bounds()
        );
    }

    // The tab's controls (right of the player list's edge: the list's
    // own scrolled-off rows are no tab's business) end inside the pane's
    // content on both axes — its right and its bottom, each less the two
    // `CentralPanel` margins: the 456-point settings column fits, the
    // deepest control inside. The deepest is named, so the check is the
    // column's own bottom (the Automatic settings' last checkbox).
    let deepest = node_labeled(&nodes, Role::CheckBox, "Attack/defence levels");
    assert!(
        deepest.bounds().expect("bounds").y1 <= f64::from(DEFAULT_PANE.y - 2.0 * PANEL_MARGIN),
        "the deepest settings control inside the pane's bottom content edge ({:?})",
        deepest.bounds()
    );
    for role in [Role::Slider, Role::Button, Role::ComboBox, Role::CheckBox] {
        for node in nodes.iter().filter(|node| node.role() == role) {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("{role:?} has no bounds"));
            assert!(
                bounds.x0 < canvas.x0 - 1.0
                    || bounds.x1 <= f64::from(DEFAULT_PANE.x - 2.0 * PANEL_MARGIN),
                "{role:?} {} ends outside the pane's right content edge ({bounds:?})",
                node.label().or(node.value()).unwrap_or("?")
            );
            assert!(
                bounds.x0 < canvas.x0 - 1.0
                    || bounds.y1 <= f64::from(DEFAULT_PANE.y - 2.0 * PANEL_MARGIN),
                "{role:?} {} ends outside the pane's bottom content edge ({bounds:?})",
                node.label().or(node.value()).unwrap_or("?")
            );
        }
    }
}

// plans/save_editor.md "Tactics editor" ("wrapping inside it"): at the
// narrowest three-column pane the 360-point settings column wraps the
// preset bar to a second line rather than clipping it — the last menu
// button opens the second line, and every bar widget ends inside the
// column's right edge (the pane's right less the two `CentralPanel`
// margins).
#[test]
fn the_preset_bar_wraps_inside_the_settings_column() {
    let path = fixture(PesVersion::Pes21, "preset_bar_wrap");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let pane = three_columns_width() + TAB_AREA_OFFSET;
    let mut window = Headless::sized(pane, DEFAULT_PANE.y);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let settings_left = label_bounds(&nodes, "Formation settings").x0;
    let preset = label_bounds(&nodes, "Preset");
    assert!(
        (preset.x0 - settings_left).abs() < 1.0,
        "the bar starts the settings column"
    );
    // The wrap: the "Copy/swap formation" menu opens the second line,
    // below the "Preset" label's.
    let formation_menu = node_labeled(&nodes, Role::Button, "Copy/swap formation");
    assert!(
        formation_menu.bounds().expect("bounds").y0 > preset.y1 - 1.0,
        "the bar wraps to a second line ({:?} vs {preset:?})",
        formation_menu.bounds()
    );
    for label in ["1", "2", "3", "Copy/swap preset", "Copy/swap formation"] {
        let button = node_labeled(&nodes, Role::Button, label);
        assert!(
            button.bounds().expect("bounds").x1 <= f64::from(pane - 2.0 * PANEL_MARGIN),
            "{label} ends inside the settings column's right edge ({:?})",
            button.bounds()
        );
    }
}

// The draft's `#instr-table select` cap: the target combo is bounded
// and truncates, so a real player's long name cannot push the row past
// the settings column (the draft's 120-point cap or the column's edge,
// whichever the row leaves); the full name still shows in the popup and
// on the closed combo's hover.
#[test]
fn the_target_combo_stays_inside_the_settings_column() {
    let path = fixture(PesVersion::Pes21, "target_combo_long_name");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    let slot = file.team(701).unwrap().tactics.starting_eleven[2];
    let player_id = file.team(701).unwrap().roster[usize::from(slot)].player_id;
    file.player_mut(player_id).unwrap().name = "3DSMAX HAS STOPPED WORKING".to_owned();
    {
        let defence = file.team_mut(701).unwrap().tactics.presets[0]
            .defence_instructions
            .as_mut()
            .expect("PES 21 stores instructions");
        defence[0].instruction = pes_savefile::schema::instruction::encode(
            PesVersion::Pes21,
            Instruction::CounterTarget,
        )
        .expect("PES 21 stores Counter Target");
        defence[0].player = slot;
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The named player's target combo shows the full text truncated —
    // `value()` keeps the whole string.
    let target = right_of_label(&nodes, "Defence 1", Role::ComboBox)
        .into_iter()
        .nth(1)
        .expect("the Defence 1 target combo");
    assert_eq!(
        target.value(),
        Some("3 3DSMAX HAS STOPPED WORKING"),
        "premise: the target names the player"
    );
    assert!(
        target.bounds().expect("bounds").x1 <= f64::from(DEFAULT_PANE.x - 2.0 * PANEL_MARGIN),
        "the capped combo ends inside the settings column ({:?})",
        target.bounds()
    );
}

// Below the three-column width (a pane tighter than the smallest
// default's 981 x 624, 877 wide — taller here so the settings' nodes
// below the fold are still drawn): the settings column draws under the
// pitch and the lists, its heading below the canvas' bottom and the
// last bench row, every control inside the pane's width.
#[test]
fn the_tactics_tab_stacks_the_settings_below_at_the_smallest_pane() {
    let path = fixture(PesVersion::Pes21, "tactics_stacked");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(877.0, 1600.0);
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    // A second frame: the scroll area's claimed region lands a frame late
    // — an over-wide claim would widen the content's own rect on this
    // one, giving the columns the three-column width back.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let settings = label_bounds(&nodes, "Formation settings");
    let canvas = canvas_of(&nodes);
    assert!(
        settings.y0 > canvas.y1,
        "the settings column below the canvas (settings {settings:?}, canvas {canvas:?})"
    );
    // And below the bench list's bottom edge as drawn: the bench scroll's
    // bar (the lists' right side — the starting eleven's 11 rows never
    // scroll, the tab's own bar would sit at the pane's right edge). The
    // rows' nodes run to the scrolled-off end of the list, so the bar's
    // track bottom is the measure, not the last row's node.
    let bench_bottom = nodes
        .iter()
        .filter(|node| node.role() == Role::ScrollBar)
        .filter_map(|node| node.bounds())
        .filter(|bounds| bounds.x0 > canvas.x1 && bounds.x0 < canvas.x1 + 260.0)
        .map(|bounds| bounds.y1)
        .fold(0.0, f64::max);
    assert!(bench_bottom > 0.0, "the bench list scrolls (premise)");
    assert!(
        settings.y0 > bench_bottom,
        "and below the last bench row in view (settings {settings:?}, bench list bottom {bench_bottom})"
    );

    for role in [Role::Slider, Role::Button, Role::ComboBox, Role::CheckBox] {
        for node in nodes.iter().filter(|node| node.role() == role) {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("{role:?} has no bounds"));
            assert!(
                bounds.x1 <= 877.0,
                "{role:?} {} clips the pane's right edge ({bounds:?})",
                node.label().or(node.value()).unwrap_or("?")
            );
        }
    }
}

// plans/save_editor.md "Tactics editor" (the "Portraits" sub-bullet):
// the pitch draws a starter's portrait read off the UI thread from the
// selected game's `download/` CPKs; the markers without one paint the
// bundled placeholder (the turf's is the only other image).
#[test]
fn the_pitch_draws_a_starter_s_portrait_from_the_selected_game() {
    let dir = crate::test_support::scratch("tactics_portrait");
    let path = dir.join("EDIT00000000");
    std::fs::write(&path, fixture_save(PesVersion::Pes21)).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    // A portrait for the starter of position index 4, at his player id's
    // CPK path, in the one listed CPK.
    let starter = usize::from(team_entry(&view, 701).tactics.starting_eleven[4]);
    let player_id = team_entry(&view, 701).roster[starter].player_id;
    let entry = pes_savefile::player_tables::portrait_path(PesVersion::Pes21, player_id);
    let bytes: &[u8] = include_bytes!("../../../tests/fixtures/portraits/75823_faces.dds");
    let folder = crate::test_support::pes_folder(
        "tactics_portrait_pes",
        &[("4cc_40_faces.cpk", &[(entry.as_str(), bytes)])],
    );
    let mut window = Headless::sized(1100.0, 900.0);
    view.select_portraits(&window.ctx, &folder, PesVersion::Pes21);

    // Bounded: the worker answers off the UI thread, so frames poll with a
    // real nap between; a portrait never arriving fails rather than hangs.
    // The portrait rect is matched by size and centre (a pixel's tolerance
    // for the AccessKit bounds' rounding), not by exact rect equality.
    let canvas_min = |canvas: accesskit::Rect| egui::pos2(canvas.x0 as f32, canvas.y0 as f32);
    let canvas_max = |canvas: accesskit::Rect| egui::pos2(canvas.x1 as f32, canvas.y1 as f32);
    let mut found = false;
    for _ in 0..200 {
        let (_, nodes, shapes) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let canvas = canvas_of(&nodes);
        let canvas_rect = egui::Rect::from_min_max(canvas_min(canvas), canvas_max(canvas));
        let centre = clamped_centre(
            canvas,
            team_entry(&view, 701).tactics.presets[0].formations[0].players[4],
        );
        let expected = egui::pos2(centre.x, centre.y - 32.0);
        let portraits: Vec<(egui::Rect, egui::TextureId)> = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Mesh(mesh) => {
                    Some((clipped.shape.visual_bounding_rect(), mesh.texture_id))
                }
                _ => None,
            })
            .filter(|(rect, _)| {
                canvas_rect.contains_rect(*rect)
                    && (rect.size() - egui::vec2(50.0, 50.0)).length() < 0.5
            })
            .collect();
        // Until the worker answers, every marker paints the placeholder's one
        // texture; the starter's arriving shows as a texture id no other
        // portrait shares, at his spot.
        let his: Vec<egui::TextureId> = portraits
            .iter()
            .filter(|(rect, _)| (rect.center() - expected).length() < 1.0)
            .map(|(_, texture)| *texture)
            .collect();
        if his.len() == 1
            && portraits
                .iter()
                .filter(|(_, texture)| *texture == his[0])
                .count()
                == 1
        {
            found = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        found,
        "the starter's marker paints his portrait and no other marker's after 200 frames"
    );
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

// The settings column's widgets keep their ids across `three_columns`
// (the same class as the width thresholds' ids: the column draws in one
// salted child `Ui` at the same call position in both arrangements, so a
// field's AccessKit id is equal at the default pane and at the smallest).
#[test]
fn the_settings_widgets_keep_their_ids_across_three_columns() {
    let path = fixture(PesVersion::Pes21, "settings_ids");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));

    let mut wide = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    let (_, ids, _) = wide.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let support_range = node_id_right_of(&ids, "Support range", Role::Slider);

    let mut narrow = Headless::sized(877.0, 624.0);
    let (_, ids, _) = narrow.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        node_id_right_of(&ids, "Support range", Role::Slider),
        support_range,
        "the slider's id across the three-column break"
    );
}

/// The Tactics tab's three-column break width: `PITCH.x` plus
/// `LISTS_WIDTH` plus `SETTINGS_MIN` plus two item gaps — `tactics.rs`
/// computes the same from the same constants; the gap is read off a
/// `Headless` frame.
fn three_columns_width() -> f32 {
    let mut gap = 0.0;
    Headless::new().frame(Vec::new(), |ui| gap = ui.spacing().item_spacing.x);
    team_widgets::pitch::PITCH.x
        + crate::view::tactics::LISTS_WIDTH
        + crate::view::tactics::SETTINGS_MIN
        + 2.0 * gap
}

// `three_columns`' edge: the pane whose tab area is exactly
// `PITCH.x + LISTS_WIDTH + SETTINGS_MIN + 2 gaps` (the constant plus
// `TAB_AREA_OFFSET`) is three columns, and one point narrower stacks
// the settings below.
#[test]
fn the_tactics_tab_switches_arrangement_at_three_columns() {
    let three_columns = three_columns_width();
    for (width, three) in [
        (three_columns + TAB_AREA_OFFSET, true),
        (three_columns + TAB_AREA_OFFSET - 1.0, false),
    ] {
        let path = fixture(PesVersion::Pes21, "three_columns_edge");
        let mut view = SaveView::new();
        view.open(&path);
        view.tab = Tab::Tactics;
        view.selected = Some((701, 0));
        let mut window = Headless::sized(width, 1400.0);
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let settings = label_bounds(&nodes, "Formation settings");
        let canvas = canvas_of(&nodes);
        if three {
            assert!(
                settings.y0 < canvas.y1,
                "width {width}: the settings beside the canvas ({settings:?} vs {canvas:?})"
            );
        } else {
            assert!(
                settings.y0 > canvas.y1,
                "width {width}: the settings below the canvas ({settings:?} vs {canvas:?})"
            );
        }
    }
}

// The lists column's left edge is the canvas' right edge plus one
// `item_spacing.x` — the tab draws the lists hard at 200 wide past it.
#[test]
fn the_lists_start_one_gap_right_of_the_canvas() {
    let path = fixture(PesVersion::Pes21, "lists_left_edge");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    let (gap, nodes, _) = window.frame(Vec::new(), |ui| {
        let gap = ui.spacing().item_spacing.x;
        show(ui, &mut view, &|| Vec::new(), None);
        gap
    });
    let canvas = canvas_of(&nodes);
    assert_eq!(
        label_bounds(&nodes, "Starting eleven").x0 - canvas.x1,
        f64::from(gap),
        "the lists' left edge is the canvas' right edge plus one item gap"
    );
}

// Below the columns' width the settings' first widget starts one
// `item_spacing.y` below the pitch-and-lists block's bottom — the bench
// list's frame ends it (the scroll's track bottom plus its group
// frame's margin).
#[test]
fn the_stacked_settings_start_one_gap_below_the_columns() {
    let path = fixture(PesVersion::Pes21, "stacked_top_gap");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(877.0, 624.0);
    let ((margin, spacing_y), nodes, _) = window.frame(Vec::new(), |ui| {
        let margin = egui::Frame::group(ui.style()).total_margin().bottom;
        let spacing_y = ui.spacing().item_spacing.y;
        show(ui, &mut view, &|| Vec::new(), None);
        (margin, spacing_y)
    });

    let canvas = canvas_of(&nodes);
    // The bench list's scroll bar (the lists' inner scroll — the tab's
    // own bar would sit at the pane's right edge, past the column).
    let bench_bottom = nodes
        .iter()
        .filter(|node| node.role() == Role::ScrollBar)
        .filter_map(|node| node.bounds())
        .filter(|bounds| bounds.x0 > canvas.x1 && bounds.x0 < canvas.x1 + 260.0)
        .map(|bounds| bounds.y1)
        .fold(0.0, f64::max);
    assert!(bench_bottom > 0.0, "the bench list scrolls (premise)");
    let preset = label_bounds(&nodes, "Preset");
    // The group frame's bottom total margin (its inner margin plus
    // stroke) under the viewport's bottom and the one item gap the
    // settings sit below the block's bottom, measured 10 at this pane.
    let gap = f64::from(margin) + f64::from(spacing_y);
    assert!(
        (preset.y0 - bench_bottom - gap).abs() < 0.5,
        "the settings' top is one frame margin and one item gap below the lists' \
         (Preset {preset:?}, bench viewport bottom {bench_bottom}, margin {margin}, gap {spacing_y})"
    );
}

// The scroll's content size is the advance rect the tab claims: at the
// smallest pane the tab still scrolls to the deepest settings control
// (its bottom inside the viewport) — the settings column's claimed
// height and width both feed the scroll extent.
#[test]
fn the_stacked_tab_scrolls_to_its_last_control() {
    let path = fixture(PesVersion::Pes21, "stacked_scroll_bottom");
    let mut view = SaveView::new();
    view.open(&path);
    view.tab = Tab::Tactics;
    view.selected = Some((701, 0));
    let mut window = Headless::sized(877.0, 624.0);

    let at = egui::pos2(300.0, 350.0);
    for _ in 0..40 {
        window.frame(
            vec![
                egui::Event::PointerMoved(at),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -120.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Wheel,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
    }
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let last = node_labeled(&nodes, Role::CheckBox, "Attack/defence levels");
    assert!(
        last.bounds().expect("bounds").y1 <= 620.0,
        "the deepest settings control reached the viewport ({:?})",
        last.bounds()
    );
}

// plans/save_editor.md "Editing UI" with `docs/plans/core/gui.md` "Shell
// layout" (the window's 960 × 600 minimum, ST-WIN-01's last clause): the
// pane the minimum window leaves — (960 - 56) wide, the sidebar's rail
// off, and (600 - 24) high, the status bar's — holds every tab whole:
// every widget and label ends inside the pane's right edge, no row
// clipping in place of a wrap.
#[test]
fn every_tab_fits_the_pane_the_minimum_window_leaves() {
    /// The widget and label roles, plus `Unknown` (the pitch canvas'
    /// role).
    const NODE_ROLES: [Role; 12] = [
        Role::Button,
        Role::CheckBox,
        Role::ComboBox,
        Role::RadioButton,
        Role::SpinButton,
        Role::Slider,
        Role::TextInput,
        Role::ColorWell,
        Role::ScrollBar,
        Role::Link,
        Role::Label,
        Role::Unknown,
    ];
    const TABS: [(Tab, &str); 4] = [
        (Tab::Abilities, "Abilities"),
        (Tab::Appearance, "Appearance"),
        (Tab::Tactics, "Tactics"),
        (Tab::TeamSettings, "Team settings"),
    ];
    // The pane the minimum window leaves.
    let pane = egui::vec2(960.0 - 56.0, 600.0 - 24.0);
    let edge = f64::from(pane.x - PANEL_MARGIN);
    for version in [PesVersion::Pes21, PesVersion::Pes17] {
        let path = fixture(version, "min_pane_fit");
        for (tab, name) in TABS {
            let mut view = SaveView::new();
            view.open(&path);
            view.tab = tab;
            // A Fox save with some players stripped and some restored opens
            // the Strip again window (SE-GUI-20); closed, the tab's own
            // nodes are the only ones measured.
            view.strip_again = None;
            let mut window = Headless::sized(pane.x, pane.y);
            // Three frames: the scroll's claimed content size lands a
            // frame late.
            let mut nodes = Vec::new();
            for _ in 0..3 {
                let (_, n, _) =
                    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
                nodes = n;
            }
            for node in nodes
                .iter()
                .filter(|node| NODE_ROLES.contains(&node.role()))
            {
                let bounds = node
                    .bounds()
                    .unwrap_or_else(|| panic!("{:?} has no bounds", node.role()));
                assert!(
                    bounds.x1 <= edge,
                    "{version:?} {name}: {:?} {:?} ends at {:.0}, past the pane's right edge \
                     {edge:.0} ({bounds:?})",
                    node.role(),
                    node.label().or(node.value()).unwrap_or(""),
                    bounds.x1
                );
            }
        }
    }
}
