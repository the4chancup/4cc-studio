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

/// The `index`th "Copy/swap" menu button, left to right (both bars carry
/// one: the preset bar's first, the formation bar's second).
fn copy_swap_button(nodes: &[accesskit::Node], index: usize) -> &accesskit::Node {
    let mut menus: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::Button && node.label() == Some("Copy/swap"))
        .collect();
    menus.sort_by(|a, b| {
        let x = |node: &&accesskit::Node| node.bounds().expect("menu bounds").x0;
        x(a).partial_cmp(&x(b)).expect("ordered bounds")
    });
    menus
        .get(index)
        .copied()
        .unwrap_or_else(|| panic!("no Copy/swap button {index}"))
}

// SE-GUI-25
#[test]
fn the_tactics_bars_copy_presets_and_swap_formations_as_undoable_edits() {
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
    let mut window = Headless::sized(1100.0, 900.0);
    open_team_view(&mut window, &mut view, 701);
    let preset1_before = team_entry(&view, 701).tactics.presets[0].clone();
    let formations_before = team_entry(&view, 701).tactics.presets[0].formations.clone();
    let preset2_before = team_entry(&view, 701).tactics.presets[1].clone();
    assert_ne!(
        preset1_before, preset2_before,
        "premise: presets 1 and 2 differ"
    );

    // The preset bar's Copy/swap (the first menu button): the other two
    // presets ascending, a copy to then a swap with each.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, copy_swap_button(&nodes, 0), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        menu_items(&nodes),
        ["Copy to 2", "Copy to 3", "Swap with 2", "Swap with 3"],
        "the preset menu's entries on preset 1"
    );
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy to 2"),
        &mut view,
    );
    assert_eq!(
        team_entry(&view, 701).tactics.presets[1],
        preset1_before,
        "preset 2 equals preset 1 as it was"
    );

    // The formation bar's Copy/swap, on Kick-off: the other two
    // formations in order.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, copy_swap_button(&nodes, 1), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        menu_items(&nodes),
        [
            "Copy to In-possession",
            "Copy to Out-of-possession",
            "Swap with In-possession",
            "Swap with Out-of-possession",
        ],
        "the formation menu's entries on Kick-off"
    );
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Swap with In-possession"),
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
    let mut window = Headless::sized(1100.0, 900.0);
    open_team_view(&mut window, &mut view, 701);
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

// plans/save_editor.md "Tactics editor" (the Copy/swap bullet and the
// preset bar, the draft's `presetSel`, `fluid`, `presetMenu`): on preset
// 2 the menu lists the other two presets ascending, a copy to then a
// swap with each, and Copy to 3 makes preset 3 equal preset 2 — one
// undoable edit. The bar's Fluid formation checkbox toggles the shown
// preset's flag — one undoable edit — and draws disabled with its gate
// tooltip on a version whose schema stores none (PES 2015).
#[test]
fn the_preset_bars_menu_and_fluid_checkbox() {
    let path = fixture(PesVersion::Pes21, "tactics_preset_bar");
    // Distinct sliders per preset (the copy is a real change), preset 2's
    // fluid flag unset (the checkbox toggles it on).
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for (preset, value) in [(0, 7), (1, 5), (2, 1)] {
        file.team_mut(701).unwrap().tactics.presets[preset]
            .sliders
            .support_range = value;
    }
    file.team_mut(701).unwrap().tactics.presets[1].style.fluid = Some(false);
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 900.0);
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "2"),
        &mut view,
    );
    assert_eq!(view.tactics.preset, 1, "preset 2 is on screen");
    let preset2_before = team_entry(&view, 701).tactics.presets[1].clone();
    let preset3_before = team_entry(&view, 701).tactics.presets[2].clone();
    assert_ne!(preset2_before, preset3_before, "premise: presets differ");

    // The preset menu on preset 2: the other two ascending.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, copy_swap_button(&nodes, 0), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        menu_items(&nodes),
        ["Copy to 1", "Copy to 3", "Swap with 1", "Swap with 3"],
        "the preset menu's entries on preset 2"
    );
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Copy to 3"),
        &mut view,
    );
    assert_eq!(
        team_entry(&view, 701).tactics.presets[2],
        preset2_before,
        "preset 3 equals preset 2 as it was"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the copy was one undo entry"
    );
    // Undo it for the fluid half's count.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });

    // The bar's Fluid formation checkbox toggles the shown preset's flag:
    // one undo entry, the session marked.
    let fluid1_before = team_entry(&view, 701).tactics.presets[0].style.fluid;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let fluid = node_labeled(&nodes, Role::CheckBox, "Fluid formation");
    assert_eq!(fluid.toggled(), Some(accesskit::Toggled::False));
    click(&mut window, fluid, &mut view);
    assert_eq!(
        team_entry(&view, 701).tactics.presets[1].style.fluid,
        Some(true),
        "the shown preset's flag toggled"
    );
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0].style.fluid,
        fluid1_before,
        "preset 1 untouched"
    );
    assert!(
        view.session.as_ref().unwrap().is_dirty(),
        "the toggle marked the team changed"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the fluid toggle was one undo entry"
    );
}

// The same bar on PES 2015, the version whose schema stores no fluid
// flag: the checkbox stays in place, disabled with its `PES NN+`
// tooltip.
#[test]
fn the_fluid_checkbox_is_gated_on_pes15() {
    let path = fixture(PesVersion::Pes15, "tactics_fluid_gated");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 900.0);
    window
        .ctx
        .all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let fluid = node_labeled(&nodes, Role::CheckBox, "Fluid formation");
    assert!(fluid.is_disabled(), "the flag is disabled on PES 15");
    assert_eq!(
        tooltip_after_hover(&mut window, &mut view, centre(fluid)),
        Some("PES 16+".to_owned()),
        "the gate tooltip names the first version with the flag"
    );
}

// SE-GUI-12
#[test]
fn the_pitch_moves_assigns_and_swaps_on_the_tactics_editor() {
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
    let mut window = Headless::sized(1600.0, 1400.0);
    open_team_view(&mut window, &mut view, 701);
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
    click(&mut window, chip(&nodes, "Penalty kick"), &mut view);
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
    click(&mut window, chip(&nodes, "Join the attack"), &mut view);
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

/// The stroked rects the frame drew (the panels' `Frame::group` boxes
/// and the pitch's border — transparent fill, a visible stroke), in
/// (y, x) order, `min_height` tall or more (badges, checkboxes and
/// buttons never reach it).
fn stroked_rects(shapes: &[egui::epaint::ClippedShape], min_height: f32) -> Vec<egui::Rect> {
    let mut rects: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape)
                if shape.fill == egui::Color32::TRANSPARENT
                    && !shape.stroke.is_empty()
                    && shape.rect.height() >= min_height =>
            {
                Some(shape.rect)
            }
            _ => None,
        })
        .collect();
    rects.sort_by(|a, b| {
        (a.min.y, a.min.x)
            .partial_cmp(&(b.min.y, b.min.x))
            .expect("ordered bounds")
    });
    rects
}

/// The stroked rect (a panel's box or the pitch's border) whose left
/// edge is `left` within `tol`.
fn box_at(rects: &[egui::Rect], left: f64, tol: f64) -> egui::Rect {
    *rects
        .iter()
        .find(|rect| (f64::from(rect.min.x) - left).abs() <= tol)
        .unwrap_or_else(|| panic!("no stroked rect at left {left} (have {rects:?})"))
}

// plans/save_editor.md "Tactics editor" (the grid of the "pitch is the
// editor" bullet, the draft's `.tacgrid`): at the default pane (1224 x
// 776) the editor is one grid of three columns and three rows — the
// settings panels' boxes at the column's 414 width, the pitch's border
// 10 points past them, the Player assignments box 10 past the pitch's
// right and out to the pane's edge; the preset bar and the formation bar
// share one line's centre, the formation bar's first button on the
// pitch's left; row 3's panels top 6 under row 2's bottom, the two
// lists halves of columns 2-3 less a 6-point gap.
#[test]
fn the_tactics_editor_is_one_grid_at_the_default_pane() {
    let path = fixture(PesVersion::Pes21, "tactics_grid");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let rects = stroked_rects(&shapes, 60.0);
    // The settings column's left and width: the Formation settings box
    // holds the heading's label at its inner edge (the group margin and
    // stroke in).
    let heading = label_bounds(&nodes, "Formation settings");
    let settings_left = heading.x0 - 7.0;
    let panels: Vec<egui::Rect> = rects
        .iter()
        .copied()
        .filter(|rect| (f64::from(rect.min.x) - settings_left).abs() < 1.0)
        .collect();
    assert_eq!(
        panels.len(),
        3,
        "Formation settings, Advanced instructions and Automatic settings on column 1 ({panels:?})"
    );
    for box_ in &panels {
        assert!(
            (f64::from(box_.width()) - 414.0).abs() <= 1.0,
            "a settings panel 414 wide ({box_:?})"
        );
    }
    let pitch_left = settings_left + 414.0 + 10.0;
    let canvas = canvas_of(&nodes);
    assert!(
        (canvas.x0 - 1.0 - pitch_left).abs() <= 1.0,
        "the pitch's box left 10 past the settings column (canvas {canvas:?})"
    );
    let assigns_left = pitch_left + f64::from(team_widgets::pitch::PITCH_BOX.x) + 10.0;
    let pane_right = f64::from(DEFAULT_PANE.x - 2.0 * PANEL_MARGIN);
    let assigns = box_at(&rects, assigns_left, 1.0);
    assert!(
        (f64::from(assigns.max.x) - pane_right).abs() <= 1.0,
        "Player assignments' right at the pane's ({assigns:?}, pane right {pane_right})"
    );

    // The bars on one line: preset 2's and Kick-off's centres within a
    // point; Kick-off's left at the pitch's.
    let two = node_labeled(&nodes, Role::Button, "2");
    let kick_off = node_labeled(&nodes, Role::Button, "Kick-off");
    let centre_y = |node: &accesskit::Node| {
        let b = node.bounds().expect("bounds");
        (b.y0 + b.y1) / 2.0
    };
    assert!(
        (centre_y(two) - centre_y(kick_off)).abs() <= 0.5,
        "the two bars' centres on one line ({:?} vs {:?})",
        two.bounds(),
        kick_off.bounds()
    );
    assert!(
        (kick_off.bounds().expect("bounds").x0 - pitch_left).abs() <= 1.0,
        "the formation bar starts at the pitch's left ({:?})",
        kick_off.bounds()
    );
    // And the bar fits its span at the design width: the Scroll names
    // checkbox, its last item, ends inside the pane's right edge.
    let scroll_names = node_labeled(&nodes, Role::CheckBox, "Scroll names");
    assert!(
        scroll_names.bounds().expect("bounds").x1 <= pane_right,
        "Scroll names ends inside the pane's right edge ({:?})",
        scroll_names.bounds()
    );

    // Row 3's tops 6 under row 2's bottom: row 2's bottom is the deepest
    // of the settings box, the pitch's and the assignments'.
    let settings_box = box_at(&rects, settings_left, 1.0);
    let pitch_box = box_at(&rects, pitch_left, 1.0);
    let row2_bottom = f64::from(settings_box.max.y.max(pitch_box.max.y).max(assigns.max.y));
    let row3_top = row2_bottom + 6.0;
    // The two lists over columns 2-3, halves less the 6-point gap.
    let span = pane_right - pitch_left;
    let half = (span - 6.0) / 2.0;
    let mut lists: Vec<egui::Rect> = rects
        .iter()
        .copied()
        .filter(|rect| (f64::from(rect.min.y) - row3_top).abs() <= 1.0)
        .collect();
    lists.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
    assert_eq!(
        lists.len(),
        3,
        "the stack's and the two lists' boxes in row 3 ({lists:?})"
    );
    let eleven = lists[1];
    let bench = lists[2];
    assert!(
        (f64::from(eleven.min.x) - pitch_left).abs() <= 1.0,
        "the eleven's left at column 2 ({eleven:?})"
    );
    assert!(
        (f64::from(eleven.width()) - half).abs() <= 1.0,
        "the eleven half of the span less 6 ({eleven:?}, half {half})"
    );
    assert!(
        (f64::from(bench.min.x) - (pitch_left + half + 6.0)).abs() <= 1.0,
        "the bench's left a half and a gap later ({bench:?})"
    );
    assert!(
        (f64::from(bench.max.x) - pane_right).abs() <= 1.0,
        "the bench's right at the pane's ({bench:?})"
    );

    // No "Starting eleven" or "Bench" headings (the lists' labels are
    // gone): the labels are nowhere in the tree.
    for heading in ["Starting eleven", "Bench"] {
        assert!(
            !nodes
                .iter()
                .any(|node| node.role() == Role::Label && node.value() == Some(heading)),
            "no {heading:?} heading"
        );
    }
}

// plans/save_editor.md "Tactics editor" (row 2 of "the pitch is the
// editor", the draft's `.tacgrid` stretching its cells): the Formation
// settings' and Player assignments' frames are exactly as tall as the
// pitch with its border — tops and bottoms level at the 1280 x 800
// default pane.
#[test]
fn the_two_panels_frames_are_level_with_the_pitchs() {
    let path = fixture(PesVersion::Pes21, "tactics_panel_frames");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let rects = stroked_rects(&shapes, 60.0);
    let settings_left = label_bounds(&nodes, "Formation settings").x0 - 7.0;
    let pitch_left = settings_left + 414.0 + 10.0;
    let assigns_left = pitch_left + f64::from(team_widgets::pitch::PITCH_BOX.x) + 10.0;
    let settings = box_at(&rects, settings_left, 1.0);
    let pitch_box = box_at(&rects, pitch_left, 1.0);
    let assigns = box_at(&rects, assigns_left, 1.0);
    for (name, frame) in [
        ("Formation settings", settings),
        ("Player assignments", assigns),
    ] {
        assert!(
            (f64::from(frame.min.y) - f64::from(pitch_box.min.y)).abs() <= 0.5,
            "{name}'s top level with the pitch's ({frame:?} vs {pitch_box:?})"
        );
        assert!(
            (f64::from(frame.max.y) - f64::from(pitch_box.max.y)).abs() <= 0.5,
            "{name}'s bottom level with the pitch's ({frame:?} vs {pitch_box:?})"
        );
    }
}

// plans/save_editor.md "Tactics editor" (row 3 of "the pitch is the
// editor", the draft's `#tac-auto { flex: 1 }` and `.tacauto`'s
// `align-content: center`): at the 1280 x 800 default pane the
// Automatic settings' frame reaches the lineup lists' bottom, its two
// rows centred in it.
#[test]
fn the_automatic_settings_frame_reaches_the_lists_bottom() {
    let path = fixture(PesVersion::Pes21, "tactics_auto_frame");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let rects = stroked_rects(&shapes, 60.0);
    let settings_left = label_bounds(&nodes, "Formation settings").x0 - 7.0;
    // Column 1's three boxes, top to bottom: Formation settings,
    // Advanced instructions, Automatic settings.
    let panels: Vec<egui::Rect> = rects
        .iter()
        .copied()
        .filter(|rect| (f64::from(rect.min.x) - settings_left).abs() < 1.0)
        .collect();
    assert_eq!(panels.len(), 3, "the settings column's boxes");
    let automatic = panels[2];
    // The lists' boxes share row 3's top — the first box right of the
    // settings column whose top is below the pitch's is theirs (the
    // stack's Advanced instructions is at settings_left).
    let pitch_left = settings_left + 414.0 + 10.0;
    let pitch_box = box_at(&rects, pitch_left, 1.0);
    let row3_top = f64::from(pitch_box.max.y) + 6.0;
    let lists_bottom = rects
        .iter()
        .filter(|rect| {
            f64::from(rect.min.x) > settings_left + 1.0
                && (f64::from(rect.min.y) - row3_top).abs() <= 1.0
        })
        .map(|rect| rect.max.y)
        .reduce(f32::max)
        .expect("the lists' boxes in row 3");
    assert!(
        (f64::from(automatic.max.y) - f64::from(lists_bottom)).abs() <= 0.5,
        "the frame's bottom is the lists' ({automatic:?}, lists' bottom {lists_bottom})"
    );

    // The rows centred in `.tacauto`, the region under the heading:
    // the four controls' vertical centre is the centre of the space
    // between the heading's bottom (plus its 3-point gap) and the
    // frame's bottom inner edge (the 7-point margin and stroke).
    let heading_bottom = label_bounds(&nodes, "Automatic settings").y1 + 3.0;
    let region_bottom = f64::from(automatic.max.y) - 7.0;
    let controls = [
        right_of_label(&nodes, "Substitution", Role::ComboBox)
            .into_iter()
            .next()
            .expect("the Substitution combo"),
        node_labeled(&nodes, Role::CheckBox, "Preset change"),
        node_labeled(&nodes, Role::CheckBox, "Offside trap"),
        node_labeled(&nodes, Role::CheckBox, "Attack/defence levels"),
    ];
    let centre = controls
        .iter()
        .map(|node| {
            let bounds = node.bounds().expect("control bounds");
            (bounds.y0 + bounds.y1) / 2.0
        })
        .sum::<f64>()
        / controls.len() as f64;
    let region_centre = (heading_bottom + region_bottom) / 2.0;
    assert!(
        (centre - region_centre).abs() <= 1.0,
        "the rows centred under the heading ({centre} vs {region_centre})"
    );
}

// The draft's `.rate` toggles write their field, one undo entry: a
// click on Defensive style's "All Out Def." stores the flag.
#[test]
fn a_style_toggle_click_is_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "tactics_toggle_undo");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1600.0, 1400.0);
    open_team_view(&mut window, &mut view, 701);
    let before = team_entry(&view, 701).tactics.presets[0]
        .style
        .defensive_style;

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let choice = node_labeled(&nodes, Role::Button, "All Out Def.");
    click(&mut window, choice, &mut view);
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0]
            .style
            .defensive_style,
        !before,
        "the click wrote the field"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the click was one undo entry"
    );
}

// The draft's `.rate.tri` toggles write their field, one undo entry:
// a click on Numbers in attack's "Many" stores 3.
#[test]
fn a_numbers_choice_click_is_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "tactics_tri_undo");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.team_mut(701).unwrap().tactics.presets[0]
        .sliders
        .numbers_in_attack = 1;
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1600.0, 1400.0);
    open_team_view(&mut window, &mut view, 701);

    // "Numbers in attack"'s "Many" is the choice under its label.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let many = choice_below(&nodes, "Numbers in attack", "Many");
    click(&mut window, many, &mut view);
    assert_eq!(
        team_entry(&view, 701).tactics.presets[0]
            .sliders
            .numbers_in_attack,
        3,
        "Many stored"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the click was one undo entry"
    );
}

// plans/save_editor.md "Tactics editor" (the Substitution sentence):
// the combo's pick writes the stored byte — Flexible stores 2 — as one
// undo entry.
#[test]
fn a_substitution_pick_is_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "tactics_subs_undo");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.team_mut(701).unwrap().tactics.auto.substitution = Some(0);
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1600.0, 1400.0);
    open_team_view(&mut window, &mut view, 701);

    // The combo right of the Substitution label, opened by an
    // AccessKit click; its items join the tree the frame after.
    let (_, ids, _) = window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combo = node_id_right_of(&ids, "Substitution", Role::ComboBox);
    action_click(&mut window, &mut view, combo);
    let (_, ids, _) = window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combo_bounds = ids
        .iter()
        .find(|(id, _)| *id == combo)
        .and_then(|(_, node)| node.bounds())
        .expect("the combo's bounds");
    // The popup's "Flexible", below the combo — Positioning's own
    // "Flexible" choice is up in Formation settings.
    let flexible = ids
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button
                && node.label() == Some("Flexible")
                && node.bounds().is_some_and(|b| b.y0 > combo_bounds.y0)
        })
        .map(|(id, _)| *id)
        .expect("the popup's Flexible item");
    action_click(&mut window, &mut view, flexible);
    assert_eq!(
        team_entry(&view, 701).tactics.auto.substitution,
        Some(2),
        "Flexible stored as 2"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the pick was one undo entry"
    );
}

// plans/save_editor.md "Tactics editor" (the click-to-assign bullet)
// on the fixtures' real team: each taker and the captain show their
// holder's name on a chip under the label, "Join the attack" shows
// its members' names or "Unassigned", and a join chip's click arms
// the one mode, all three chips selected, in which a pitch click adds
// a starter; an armed click disarms.
#[test]
fn the_assignments_rows_name_holders_and_arm_join() {
    let path = fixture(PesVersion::Pes17, "assignments_rows_734");
    // Two join members and a non-identity eleven (an arm writing the
    // index for the slot would look right on a [0, 1, ...] eleven).
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).unwrap();
    {
        let tactics = &mut file.team_mut(734).unwrap().tactics;
        tactics.starting_eleven.swap(4, 9);
        tactics.players_to_join_attack = [
            tactics.starting_eleven[3],
            tactics.starting_eleven[5],
            u8::MAX,
        ];
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1600.0, 1400.0);
    // Team 734's own team view (its row is below the list's fold).
    view.selected = Some(Selection::Team(734));
    let name_of = |view: &SaveView, slot: u8| {
        let team = team_entry(view, 734);
        let player_id = team.roster[usize::from(slot)].player_id;
        display_name(
            &view
                .session
                .as_ref()
                .unwrap()
                .file()
                .player(player_id)
                .unwrap()
                .name,
        )
    };

    // The captain's chip reads his name; the join list's chips read
    // the two names and one "Unassigned".
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let captain = team_entry(&view, 734).tactics.set_pieces.captain;
    assert_ne!(captain, u8::MAX, "premise: a captain is assigned");
    let captain_chip = chip(&nodes, "Captain");
    assert_eq!(
        captain_chip.label(),
        Some(name_of(&view, captain).as_str()),
        "the captain chip names the captain"
    );
    let starters = team_entry(&view, 734).tactics.starting_eleven;
    let expected: Vec<String> = [
        name_of(&view, starters[3]),
        name_of(&view, starters[5]),
        "Unassigned".to_owned(),
    ]
    .into_iter()
    .collect();
    let join_chips = chips_below(&nodes, "Join the attack");
    assert_eq!(join_chips.len(), 3, "three join chips");
    for (chip, expected) in join_chips.iter().zip(&expected) {
        assert_eq!(chip.label(), Some(expected.as_str()));
    }

    // Clicking the third join chip arms the one mode, all three
    // selected; a pitch click on a starter adds him; clicking it again
    // disarms.
    let third = join_chips[2];
    click(&mut window, third, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    for chip in chips_below(&nodes, "Join the attack") {
        assert_eq!(
            chip.toggled(),
            Some(accesskit::Toggled::True),
            "all three chips show armed"
        );
    }
    let added = team_entry(&view, 734).tactics.starting_eleven[7];
    let canvas = canvas_of(&nodes);
    let at = marker_point(
        canvas,
        &team_entry(&view, 734).tactics.presets[0].formations[0].players,
        7,
    )
    .expect("a point on the marker a click can reach");
    click_at(&mut window, at, &mut view);
    assert_eq!(
        team_entry(&view, 734).tactics.players_to_join_attack[2],
        added,
        "the pitch click added him"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let third = chips_below(&nodes, "Join the attack")[2];
    click(&mut window, third, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    for chip in chips_below(&nodes, "Join the attack") {
        assert_eq!(
            chip.toggled(),
            Some(accesskit::Toggled::False),
            "the armed click disarms all three"
        );
    }
}

// plans/save_editor.md "Tactics editor" (the click-to-assign bullet's
// last sentence) on the fixtures' real team: an instruction's target
// chip arms the same pitch pick as the roles' — click it, then a
// starter's marker writes his roster slot as the target, one undo
// entry, and the mode is off after. Under a target-less instruction
// the chip is disabled ("No target"), under a held Tight Marking it
// reads "An opponent".
#[test]
fn the_instruction_target_is_picked_on_the_pitch() {
    let path = fixture(PesVersion::Pes17, "instruction_target_734");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).unwrap();
    {
        let encode = |inst| {
            pes_savefile::schema::instruction::encode(PesVersion::Pes17, inst)
                .expect("PES 17 stores it")
        };
        let tactics = &mut file.team_mut(734).unwrap().tactics;
        let defence = tactics.presets[0]
            .defence_instructions
            .as_mut()
            .expect("PES 17 stores instructions");
        // Defence 1 Counter Target, its pick unset; defence 2 a
        // target-less attacking one held (the combo keeps the wrong
        // side's stored value); attack 1 a held Tight Marking.
        defence[0].instruction = encode(Instruction::CounterTarget);
        defence[0].player = u8::MAX;
        defence[1].instruction = encode(Instruction::HugTheTouchline);
        let attack = tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("instructions");
        attack[0].instruction = encode(Instruction::TightMarking);
        attack[0].player = 4;
        // A non-identity eleven: an arm writing the position index for
        // the roster slot would look right on a [0, 1, ...] eleven.
        tactics.starting_eleven.swap(4, 9);
    }
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1600.0, 1400.0);
    view.selected = Some(Selection::Team(734));

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combos = instruction_combos(&nodes);
    // The disabled chips first: the target-less one reads "No target",
    // the held Tight Marking "An opponent".
    let no_target = chip_under(&nodes, combos[1][1]);
    assert!(no_target.is_disabled(), "the target-less chip is disabled");
    assert_eq!(no_target.label(), Some("No target"));
    let opponent = chip_under(&nodes, combos[0][0]);
    assert!(opponent.is_disabled(), "the Tight Marking chip is disabled");
    assert_eq!(opponent.label(), Some("An opponent"));

    // Defence 1's chip arms the pick: it toggles on, then a starter's
    // marker writes his roster slot, one undo entry, the chip back off.
    let chip = chip_under(&nodes, combos[0][1]);
    assert!(!chip.is_disabled(), "Counter Target's chip is enabled");
    click(&mut window, chip, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let chip = chip_under(&nodes, instruction_combos(&nodes)[0][1]);
    assert_eq!(
        chip.toggled(),
        Some(accesskit::Toggled::True),
        "the armed chip is selected"
    );
    let index = 4;
    let starter = team_entry(&view, 734).tactics.starting_eleven[index];
    assert_ne!(
        starter, index as u8,
        "premise: the picked position's slot differs from its index"
    );
    let canvas = canvas_of(&nodes);
    let at = marker_point(
        canvas,
        &team_entry(&view, 734).tactics.presets[0].formations[0].players,
        index,
    )
    .expect("a point on the marker a click can reach");
    click_at(&mut window, at, &mut view);
    assert_eq!(
        team_entry(&view, 734).tactics.presets[0]
            .defence_instructions
            .as_ref()
            .expect("instructions")[0]
            .player,
        starter,
        "the clicked starter's roster slot is the target"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let chip = chip_under(&nodes, instruction_combos(&nodes)[0][1]);
    assert_eq!(
        chip.toggled(),
        Some(accesskit::Toggled::False),
        "the write disarmed the mode"
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the pick was one undo entry"
    );
}

// The draft's `.rolechip` ellipsis: the target chip is its column's
// width and truncates, so a real player's long name cannot push the
// chip past the settings column; the chip's label keeps the whole
// string (its hover does).
#[test]
fn the_target_chip_stays_inside_the_settings_column() {
    let path = fixture(PesVersion::Pes21, "target_chip_long_name");
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
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The target chip under defence 1's combo — its label keeps the
    // whole name though the text truncates.
    let chip = chip_under(&nodes, instruction_combos(&nodes)[0][1]);
    assert_eq!(
        chip.label(),
        Some("3 3DSMAX HAS STOPPED WORKING"),
        "premise: the target names the player"
    );
    // The settings column's right edge: the column's left and 414.
    let column_right = label_bounds(&nodes, "Formation settings").x0 - 7.0 + 414.0;
    assert!(
        chip.bounds().expect("bounds").x1 <= column_right,
        "the truncating chip ends inside the settings column ({:?})",
        chip.bounds()
    );
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
    open_team_view(&mut window, &mut view, 701);
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

// The Automatic settings row wraps inside the settings column (the
// draft's `.row`): the Substitution pair and the three checkboxes each
// end inside the column's 414 width. And the armed mode draws no note
// (the plan's "No note explains the armed mode"): no Cancel, no Done,
// no "Click the pitch player" line while a mode is armed — the armed
// chip and the candidates' corner brackets say it.
#[test]
fn the_automatic_settings_wrap_inside_the_column_and_armed_draws_no_note() {
    let path = fixture(PesVersion::Pes21, "tactics_wrap2");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    open_team_view(&mut window, &mut view, 701);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let column_right = label_bounds(&nodes, "Formation settings").x0 - 7.0 + 414.0;
    let substitution = control_right_of(&nodes, "Substitution");
    assert!(
        substitution.bounds().expect("field bounds").x1 <= column_right,
        "the Substitution field inside the settings column"
    );
    for label in ["Offside trap", "Preset change", "Attack/defence levels"] {
        let checkbox = node_labeled(&nodes, Role::CheckBox, label);
        assert!(
            checkbox.bounds().expect("checkbox bounds").x1 <= column_right,
            "{label} inside the settings column"
        );
    }

    // Arm join: no note and no Cancel or Done anywhere in the tree.
    click(&mut window, chip(&nodes, "Join the attack"), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    for label in ["Cancel", "Done"] {
        assert!(
            !nodes
                .iter()
                .any(|node| node.role() == Role::Button && node.label() == Some(label)),
            "no {label} button while armed"
        );
    }
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Label
                && node
                    .value()
                    .is_some_and(|value| value.starts_with("Click the pitch player"))
        }),
        "no note's text while armed"
    );
}

// The scroll's content size is the advance rect the view claims: the
// view still scrolls to the grid's deepest control (its bottom inside
// the viewport) — the grid's claimed height feeds the scroll extent.
#[test]
fn the_view_scrolls_to_the_grids_last_control() {
    let path = fixture(PesVersion::Pes21, "stacked_scroll_bottom");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(877.0, 624.0);
    open_team_view(&mut window, &mut view, 701);

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
// off, and (600 - 24) high, the status bar's — holds every view whole:
// every widget and label ends inside the pane's right edge, no row
// clipping in place of a wrap.
#[test]
fn every_view_fits_the_pane_the_minimum_window_leaves() {
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
    // The pane the minimum window leaves.
    let pane = egui::vec2(960.0 - 56.0, 600.0 - 24.0);
    let edge = f64::from(pane.x - PANEL_MARGIN);
    // Each selection kind: the player view (the Appearance section closed
    // as it opens, then open). The team view's grid lays out at the
    // design width — below it the window's width zoom is what fits it
    // (the core plan, "Scaling with the width"), so the raw-pane check is
    // the player view's alone.
    for version in [PesVersion::Pes21, PesVersion::Pes17] {
        let path = fixture(version, "min_pane_fit");
        for name in ["player", "appearance"] {
            let mut view = SaveView::new();
            view.open(&path);
            // A Fox save with some players stripped and some restored opens
            // the Strip again window (SE-GUI-20); closed, the view's own
            // nodes are the only ones measured.
            view.strip_again = None;
            let mut window = Headless::sized(pane.x, pane.y);
            match name {
                "appearance" => open_appearance(&mut window, &mut view),
                "team" => open_team_view(&mut window, &mut view, 701),
                _ => {}
            }
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

// The formation bar's Scroll names checkbox (the plan's marker bullet,
// `TacticsView::scroll_names`): a view setting, not an edit — on by
// default, its toggle marks no unsaved change and pushes no undo entry,
// it writes `scroll_names` to the tool's settings, and a fresh Tool on
// the same settings reads it back.
#[test]
fn scroll_names_is_a_remembered_view_setting() {
    use studio_core::StudioTool;

    let save_path = fixture(PesVersion::Pes21, "scroll_names_setting");
    let dir = save_path.parent().expect("the scratch dir").to_path_buf();
    let mut tool = crate::Tool::new();
    tool.view.open(&save_path);
    let mut settings = studio_core::Settings::default();
    let mut table = tool.default_settings();
    table.insert("unread_key".to_owned(), toml::Value::Integer(3));
    settings.set_tool("save-editor", table);
    let (ctx, _events_rx, _requests_rx) = tool_context(&dir, settings);
    // The design pane: the grid is laid out at 1280, and a narrower
    // window scales (the core plan, "Scaling with the width"), so the
    // checkbox sits inside the pane here (below it, the bar's content
    // runs past the clip).
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    open_team_view(&mut window, &mut tool.view, 701);

    // On by default; the toggle writes the setting false, the tool's
    // other keys untouched, and leaves the session clean (a view
    // setting, not an edit).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    let checkbox = node_labeled(&nodes, Role::CheckBox, "Scroll names");
    assert_eq!(
        checkbox.toggled(),
        Some(accesskit::Toggled::True),
        "on by default"
    );
    press_release(&mut window, centre(checkbox), |ui| tool.view(ui, &ctx));
    assert!(
        !tool.view.tactics.scroll_names,
        "the pointer toggle flipped the checkbox"
    );
    let table = ctx.tool_settings("save-editor");
    assert_eq!(
        table["scroll_names"].as_bool(),
        Some(false),
        "the toggle wrote the setting off"
    );
    assert_eq!(
        table["unread_key"].as_integer(),
        Some(3),
        "the tool's other keys survive"
    );
    let session = tool.view.session.as_ref().unwrap();
    assert!(!session.is_dirty(), "a view setting marks nothing");
    assert!(!session.can_undo(), "and pushes no undo entry");

    // A fresh tool on the same settings: off on the first frame.
    let mut tool2 = crate::Tool::new();
    tool2.view.open(&save_path);
    open_team_view(&mut window, &mut tool2.view, 701);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool2.view(ui, &ctx));
    let checkbox = node_labeled(&nodes, Role::CheckBox, "Scroll names");
    assert_eq!(
        checkbox.toggled(),
        Some(accesskit::Toggled::False),
        "the restart reads it back"
    );
}
