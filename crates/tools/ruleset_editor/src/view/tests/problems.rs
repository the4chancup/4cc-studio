//! The `check()` problems shown: the bottom bar's count (singular,
//! plural, the hover for all) and the rail's marks.

use super::*;

#[test]
fn the_problem_count_singularizes_one_and_pluralizes_more() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    // One problem: an empty name is an empty-string rule break.
    edit_ruleset(&mut view, |ruleset| ruleset.name = String::new());
    let frame = window.draw(&mut view, &ctx);
    assert!(contains_label(&frame.nodes, "✖ 1 problem:"));
    assert!(
        !contains_label(&frame.nodes, "(hover for all)"),
        "one problem carries no hover hint"
    );

    // Two problems: the name and an unknown at-most target.
    edit_ruleset(&mut view, |ruleset| {
        ruleset.stats.at_most.push("bogus".to_string());
    });
    let frame = window.draw(&mut view, &ctx);
    assert!(contains_label(&frame.nodes, "✖ 2 problems:"));
    assert!(contains_label(&frame.nodes, "(hover for all)"));
}

#[test]
fn a_problem_marks_only_its_section_in_the_rail() {
    let (ctx, _events, mut window, mut view) = fresh_blank();
    // A tier's empty id is a `tiers[0].id` problem: Tiers alone marked.
    edit_ruleset(&mut view, |ruleset| {
        ruleset.tiers[0].id = String::new();
    });
    let frame = window.draw(&mut view, &ctx);
    let rail = rail_labels(&frame.nodes);
    assert_eq!(rail[0], "General", "no mark on General");
    assert_eq!(rail[1], "Tiers ⚠", "the faulty section marked");
    assert!(
        rail[2..].iter().all(|label| !label.contains('⚠')),
        "no other entry marked: {rail:?}"
    );
}
