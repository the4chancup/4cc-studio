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
    let (ctx, _events, mut window, mut view) = fresh();
    open_document(&mut view, crate::document::blank());
    // Blank's at_most holds attacking_prowess, its rating lacks
    // tight_possession (premises the clicks reverse).
    let frame = window.draw(&mut view, &ctx);
    assert!(
        at_most(&view)
            .iter()
            .any(|name| name == "attacking_prowess")
    );
    assert!(!rating(&view).iter().any(|name| name == "tight_possession"));

    // Edit 1: uncheck "Attacking Prowess" in the at-most list (the
    // second same-named checkbox, the rating list's being above it).
    let prowess = nodes_labeled(&frame.nodes, "Attacking Prowess");
    assert_eq!(prowess.len(), 2, "one checkbox per checklist");
    click(&mut window, prowess[1], &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    // Edit 2: check "Tight Possession" in the rating list (the first).
    let possession = nodes_labeled(&frame.nodes, "Tight Possession");
    assert_eq!(possession.len(), 2);
    click(&mut window, possession[0], &mut view, &ctx);
    assert!(
        !at_most(&view)
            .iter()
            .any(|name| name == "attacking_prowess")
    );
    assert!(rating(&view).iter().any(|name| name == "tight_possession"));

    // Ctrl+Z undoes the second edit only; Ctrl+Y redoes it.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL)], |ui| {
        show(ui, &mut view, &ctx)
    });
    assert!(
        !rating(&view).iter().any(|name| name == "tight_possession"),
        "the second edit undone"
    );
    assert!(
        !at_most(&view)
            .iter()
            .any(|name| name == "attacking_prowess"),
        "the first kept"
    );
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
    let frame = window.draw(&mut view, &ctx);
    click(
        &mut window,
        nodes_labeled(&frame.nodes, "Attacking Prowess")[0],
        &mut view,
        &ctx,
    );
    let frame = window.draw(&mut view, &ctx);
    click(
        &mut window,
        nodes_labeled(&frame.nodes, "Defensive Prowess")[1],
        &mut view,
        &ctx,
    );
    assert!(rating(&view).iter().any(|name| name == "attacking_prowess"));
    assert!(
        !at_most(&view)
            .iter()
            .any(|name| name == "defensive_prowess")
    );

    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Edit", &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    click_prefix(&mut window, &frame.nodes, "Undo", &mut view, &ctx);
    assert!(
        at_most(&view)
            .iter()
            .any(|name| name == "defensive_prowess"),
        "the second edit undone by the button"
    );
    assert!(
        rating(&view).iter().any(|name| name == "attacking_prowess"),
        "the first kept"
    );
    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Edit", &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    click_prefix(&mut window, &frame.nodes, "Redo", &mut view, &ctx);
    assert!(
        !at_most(&view)
            .iter()
            .any(|name| name == "defensive_prowess"),
        "the second edit redone by the button"
    );
}
