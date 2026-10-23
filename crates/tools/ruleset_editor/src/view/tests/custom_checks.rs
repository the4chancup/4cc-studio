//! The Custom checks section (RE-GUI-13): the file's checks read-only
//! with the kept-as-is line.

use super::*;

// RE-GUI-13
#[test]
fn the_custom_checks_section_shows_the_code_read_only() {
    let template = &crate::view::start::templates()[4];
    assert_eq!(template.ruleset.name, "VTL11", "premise: index 4 is VTL11");
    let checks = template
        .custom_checks
        .as_ref()
        .expect("VTL11 carries checks");
    let mut document = Document::new(template.ruleset.clone());
    document.custom_checks = Some(checks.clone());
    let (ctx, _events, mut window, mut view) = fresh();
    view.set_document(document);

    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Custom checks", &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    assert!(drawn(
        &frame.nodes,
        "This code runs after the ruleset's own checks, and is kept as it is when you save."
    ));
    let code = frame
        .nodes
        .iter()
        .find(|node| node.value() == Some(checks.as_str()))
        .expect("the file's custom-checks code is drawn");
    assert_eq!(
        code.role(),
        Role::Label,
        "a label, not an editor: read-only"
    );
}
