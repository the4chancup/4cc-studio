//! The Summary section's GUI half (RE-SUM-01): the drawn text is the
//! summary and Copy puts the same text on the clipboard.

use super::*;

// RE-SUM-01
#[test]
fn the_summary_draws_the_rules_text_and_copy_puts_it_on_the_clipboard() {
    let autumn = crate::view::start::templates()[0].ruleset.clone();
    let expected = crate::summary::summary(&autumn);
    let (ctx, _events, mut window, mut view) = fresh();
    open_document(&mut view, autumn);

    let frame = window.draw(&mut view, &ctx);
    click_labeled(&mut window, &frame.nodes, "Summary", &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    assert!(
        frame
            .nodes
            .iter()
            .any(|node| node.value().is_some_and(|text| text == expected)),
        "the drawn text is the summary"
    );
    // The CopyText command lands in the click's own frame.
    let at = centre(node(&frame.nodes, "Copy"));
    window.frame(pointer_at(at, true), |ui| show(ui, &mut view, &ctx));
    let frame = window.frame(pointer_at(at, false), |ui| show(ui, &mut view, &ctx));
    assert!(
        frame
            .platform
            .commands
            .contains(&egui::OutputCommand::CopyText(expected)),
        "Copy's CopyText is the same text"
    );
}
