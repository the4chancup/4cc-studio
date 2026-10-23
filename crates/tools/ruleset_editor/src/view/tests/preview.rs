//! The data-block preview: **Show preview** draws the block read-only,
//! and the last recorded edit's changed lines are the view's `changed`
//! set (the plan's preview paragraph; the highlight's own scenario is a
//! later build's).

use super::*;

#[test]
fn the_preview_draws_the_block_and_the_edit_marks_its_changed_lines() {
    let (ctx, _events, mut window, mut view) = fresh();
    open_document(&mut view, crate::document::blank());

    // Off at every start: no preview lines.
    let frame = window.draw(&mut view, &ctx);
    assert!(
        !drawn(&frame.nodes, "Rules file: data block"),
        "the preview is off at start"
    );

    // Show preview on, through the toolbar toggle.
    click_labeled(&mut window, &frame.nodes, "Show preview", &mut view, &ctx);
    assert!(view.show_preview);
    let frame = window.draw(&mut view, &ctx);
    assert!(drawn(&frame.nodes, "Rules file: data block"));
    assert!(
        labels(&frame.nodes)
            .iter()
            .any(|label| label.contains("const RULESET")),
        "the block's lines are drawn"
    );

    // A checkbox edit in the at-most list (the second same-named
    // checkbox): one gesture, one recorded snapshot, and `changed` is
    // the block's lines the edit rewrote.
    let before = view.document.as_ref().unwrap().ruleset.clone();
    let prowess = nodes_labeled(&frame.nodes, "Attacking Prowess");
    assert_eq!(prowess.len(), 2, "one checkbox per checklist");
    click(&mut window, prowess[1], &mut view, &ctx);
    assert_ne!(
        view.document.as_ref().unwrap().ruleset,
        before,
        "premise: the click changed the ruleset"
    );
    let before = before.to_data_block();
    let after = view.document.as_ref().unwrap().ruleset.to_data_block();
    let expected: std::collections::BTreeSet<String> = after
        .lines()
        .filter(|line| !before.lines().any(|old| old == *line))
        .map(str::to_owned)
        .collect();
    assert!(!expected.is_empty(), "premise: the block changed");
    assert_eq!(
        view.changed, expected,
        "the changed lines are the new block's lines the old one lacked"
    );
    assert!(
        !view.changed.iter().any(|line| line.contains("name:")),
        "an untouched line is not marked: {:?}",
        view.changed
    );
}
