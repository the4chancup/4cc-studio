//! Undo and redo over General edits (RE-GUI-14): two edits in order,
//! undone and redone by the keys and by the Edit menu's buttons.

use super::*;

/// The document's `stats.at_most`.
fn at_most(view: &RulesetView) -> Vec<String> {
    view.document
        .as_ref()
        .map(|document| document.ruleset.stats.at_most.clone())
        .unwrap_or_default()
}

/// Whether the document's `stats.at_most` holds `name`.
fn has_at_most(view: &RulesetView, name: &str) -> bool {
    at_most(view).iter().any(|target| target == name)
}

/// The document's `stats.rating` as labels.
fn rating(view: &RulesetView) -> Vec<String> {
    view.document
        .as_ref()
        .map(|document| {
            document
                .ruleset
                .stats
                .rating
                .iter()
                .map(|stat| stat.label().to_string())
                .collect()
        })
        .unwrap_or_default()
}

// RE-GUI-14
#[test]
fn two_edits_undo_and_redo_by_key_and_button() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    // Blank's at_most holds attacking_prowess, its rating lacks
    // tight_possession (premises the clicks reverse).
    let frame = window.draw(&mut view, &ctx);
    assert!(has_at_most(&view, "attacking_prowess"));
    assert!(!rating(&view).iter().any(|name| name == "tight_possession"));

    // Edit 1: uncheck "Attacking Prowess" in the at-most list (the
    // second same-named checkbox, the rating list's being above it).
    let prowess = nodes_labeled(&frame.nodes, "Attacking Prowess");
    assert_eq!(prowess.len(), 2, "one checkbox per checklist");
    click(&mut window, prowess[1], &mut view, &ctx);
    // Edit 2: check "Tight Possession" in the rating list (the first).
    click_drawn_nth(&mut window, &mut view, &ctx, "Tight Possession", 0);
    assert!(!has_at_most(&view, "attacking_prowess"));
    assert!(rating(&view).iter().any(|name| name == "tight_possession"));

    // Ctrl+Z undoes the second edit only; Ctrl+Y redoes it.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert!(
        !rating(&view).iter().any(|name| name == "tight_possession"),
        "the second edit undone"
    );
    assert!(!has_at_most(&view, "attacking_prowess"), "the first kept");
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert!(
        rating(&view).iter().any(|name| name == "tight_possession"),
        "the second edit redone"
    );

    // Two more edits, undone and redone by the Edit menu's buttons:
    // "Attacking Prowess" checked in the rating list (the first) and
    // "Defensive Prowess" unchecked in the at-most list (the second).
    click_drawn_nth(&mut window, &mut view, &ctx, "Attacking Prowess", 0);
    click_drawn_nth(&mut window, &mut view, &ctx, "Defensive Prowess", 1);
    assert!(rating(&view).iter().any(|name| name == "attacking_prowess"));
    assert!(!has_at_most(&view, "defensive_prowess"));

    menu_click(&mut window, &mut view, &ctx, "Edit", "Undo");
    assert!(
        has_at_most(&view, "defensive_prowess"),
        "the second edit undone by the button"
    );
    assert!(
        rating(&view).iter().any(|name| name == "attacking_prowess"),
        "the first kept"
    );
    menu_click(&mut window, &mut view, &ctx, "Edit", "Redo");
    assert!(
        !has_at_most(&view, "defensive_prowess"),
        "the second edit redone by the button"
    );
}

/// The Name field's node (a single-line TextEdit).
fn name_field<'a>(nodes: &'a [accesskit::Node], value: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == Role::TextInput && node.value() == Some(value))
        .expect("the Name field")
}

/// The Name field focused (a click on it).
fn focus_name(window: &mut Headless, view: &mut RulesetView, ctx: &ToolContext, from: &str) {
    let frame = window.draw(view, ctx);
    click(window, name_field(&frame.nodes, from), view, ctx);
}

/// `text` typed into the focused Name field (the field keeps focus
/// between frames, so several calls are one typing gesture).
fn type_text(window: &mut Headless, view: &mut RulesetView, ctx: &ToolContext, text: &str) {
    window.frame(vec![egui::Event::Text(text.to_string())], |ui| {
        show(ui, view, ctx)
    });
}

/// A drag on a DragValue's node over `steps` frames, `dx` pixels a frame;
/// the pointer released at the end position.
fn drag_value(
    window: &mut Headless,
    node: &accesskit::Node,
    steps: u8,
    dx: f32,
    view: &mut RulesetView,
    ctx: &ToolContext,
) {
    let at = centre(node);
    window.frame(pointer_at(at, true), |ui| show(ui, view, ctx));
    for step in 1..=steps {
        window.frame(
            vec![egui::Event::PointerMoved(egui::pos2(
                at.x + dx * f32::from(step),
                at.y,
            ))],
            |ui| show(ui, view, ctx),
        );
    }
    window.frame(
        pointer_at(egui::pos2(at.x + dx * f32::from(steps), at.y), false),
        |ui| show(ui, view, ctx),
    );
}

/// The document's ruleset's name, at-most list and squad size.
fn basics(view: &RulesetView) -> (String, Vec<String>, u8) {
    let ruleset = &view.document.as_ref().unwrap().ruleset;
    (
        ruleset.name.clone(),
        ruleset.stats.at_most.clone(),
        ruleset.squad_size,
    )
}

#[test]
fn ctrl_z_during_a_drag_restores_the_value_before_it() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    let frame = window.draw(&mut view, &ctx);
    let squad = frame
        .nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("23"))
        .expect("the Squad size control");
    // A drag over two frames, the pointer still down.
    let at = centre(squad);
    window.frame(pointer_at(at, true), |ui| show(ui, &mut view, &ctx));
    for step in 1..=2 {
        window.frame(
            vec![egui::Event::PointerMoved(egui::pos2(
                at.x + 30.0 * step as f32,
                at.y,
            ))],
            |ui| show(ui, &mut view, &ctx),
        );
    }
    let dragged = view.document.as_ref().unwrap().ruleset.squad_size;
    assert_ne!(dragged, 23, "premise: the drag changed the squad size");
    // Ctrl+Z while the pointer is still down: the drag seals first, so
    // the undo lands before it.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert_eq!(
        view.document.as_ref().unwrap().ruleset.squad_size,
        23,
        "back to before the drag"
    );
}

#[test]
fn a_text_field_and_a_click_are_two_undo_entries() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    focus_name(&mut window, &mut view, &ctx, "New ruleset");
    type_text(&mut window, &mut view, &ctx, "x");
    assert_eq!(basics(&view).0, "New rulesetx");
    // A checkbox click is its own gesture (focus moved Some→Some).
    click_drawn_nth(&mut window, &mut view, &ctx, "Attacking Prowess", 1);
    assert!(!has_at_most(&view, "attacking_prowess"));

    // One undo: the checkbox is back, the name kept.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert_eq!(basics(&view).0, "New rulesetx", "the name kept");
    assert!(has_at_most(&view, "attacking_prowess"), "the checkbox back");
    // A second undo takes the name's whole gesture.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert_eq!(basics(&view).0, "New ruleset");
}

#[test]
fn typing_in_name_is_one_entry_and_focus_move_seals_it() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    // Two characters over two frames: one gesture.
    focus_name(&mut window, &mut view, &ctx, "New ruleset");
    type_text(&mut window, &mut view, &ctx, "a");
    type_text(&mut window, &mut view, &ctx, "b");
    // A click on dead space ends the gesture.
    let away = egui::pos2(5.0, 5.0);
    window.frame(pointer_at(away, true), |ui| show(ui, &mut view, &ctx));
    window.frame(pointer_at(away, false), |ui| show(ui, &mut view, &ctx));
    assert_eq!(basics(&view).0, "New rulesetab");
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert_eq!(basics(&view).0, "New ruleset", "the whole gesture undone");

    // Focus Name, type, then move focus to Squad size: the name's
    // gesture is sealed before the squad edit.
    focus_name(&mut window, &mut view, &ctx, "New ruleset");
    type_text(&mut window, &mut view, &ctx, "x");
    let frame = window.draw(&mut view, &ctx);
    let squad = frame
        .nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("23"))
        .expect("the Squad size control");
    drag_value(&mut window, squad, 2, 30.0, &mut view, &ctx);
    let squad_size = basics(&view).2;
    assert_ne!(squad_size, 23, "premise: the squad edit happened");
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert_eq!(basics(&view).2, 23, "the squad edit undone");
    assert_eq!(basics(&view).0, "New rulesetx", "the name kept");
}

#[test]
fn the_checklists_draw_held_stats_checked_and_a_toggle_keeps_the_rest() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    let frame = window.draw(&mut view, &ctx);
    // Blank's rating holds "Dribbling" but not "Tight Possession" (the
    // checklist's drawn state, `Toggled` on the checkbox nodes).
    let toggled = |nodes: &[accesskit::Node], label: &str| {
        nodes_labeled(nodes, label)
            .into_iter()
            .map(|node| node.toggled())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        toggled(&frame.nodes, "Dribbling"),
        [
            Some(accesskit::Toggled::True),
            Some(accesskit::Toggled::False)
        ],
        "held in rating, not in at-most"
    );
    assert_eq!(
        toggled(&frame.nodes, "Tight Possession"),
        [
            Some(accesskit::Toggled::False),
            Some(accesskit::Toggled::False)
        ],
        "held in neither"
    );

    // Uncheck "Dribbling" in the rating list (the first): the others stay.
    click(
        &mut window,
        nodes_labeled(&frame.nodes, "Dribbling")[0],
        &mut view,
        &ctx,
    );
    let rating = rating(&view);
    assert!(!rating.iter().any(|name| name == "dribbling"), "unticked");
    for held in ["ball_control", "speed", "stamina"] {
        assert!(
            rating.iter().any(|name| name == held),
            "{held} is still rated"
        );
    }
    let frame = window.draw(&mut view, &ctx);
    assert_eq!(
        toggled(&frame.nodes, "Ball Control")[0],
        Some(accesskit::Toggled::True),
        "the neighbours stay checked"
    );
}
