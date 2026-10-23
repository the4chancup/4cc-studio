//! The unsaved-changes prompt (RE-GUI-15): each of the four ways out —
//! a window close, an Open, New from template, a template pick — meets
//! the prompt on a dirty document, and each of its three answers does
//! what it says.

use studio_core::{ShellRequest, StudioTool};

use super::*;

/// `test`'s scratch folder, a context and a headless window.
fn prompt_setup(test: &str) -> (PathBuf, ToolContext, Headless) {
    let dir = crate::test_support::scratch(test);
    let (ctx, _events) = crate::test_support::context();
    (dir, ctx, Headless::new())
}

/// A document with unsaved changes over `dir`'s `edited.rhai` (the file
/// exists for the Save answers): the name edited to "Edited".
fn dirty_view(ctx: &ToolContext, dir: &Path) -> RulesetView {
    let file = write_file(dir, "edited.rhai", &official_text());
    let mut view = RulesetView::new();
    open_path(&mut view, ctx, &file);
    edit_ruleset(&mut view, |ruleset| ruleset.name = "Edited".to_string());
    assert!(
        view.document.as_ref().expect("open").is_dirty(),
        "premise: unsaved changes"
    );
    view
}

/// Whether the prompt is up.
fn prompt_drawn(nodes: &[accesskit::Node]) -> bool {
    drawn(nodes, "Save changes to Edited?")
}

/// File's item clicked through its menu (`menu` the button, `item` a
/// label prefix); the prompt is asserted up.
fn menu_pick(
    window: &mut Headless,
    view: &mut RulesetView,
    ctx: &ToolContext,
    menu: &str,
    item: &str,
) {
    let frame = window.draw(view, ctx);
    click_labeled(window, &frame.nodes, menu, view, ctx);
    let frame = window.draw(view, ctx);
    click_prefix(window, &frame.nodes, item, view, ctx);
    let frame = window.draw(view, ctx);
    assert!(prompt_drawn(&frame.nodes), "the prompt is up");
}

/// The prompt's `answer` clicked.
fn answer(window: &mut Headless, view: &mut RulesetView, ctx: &ToolContext, answer: &str) {
    let frame = window.draw(view, ctx);
    assert!(prompt_drawn(&frame.nodes), "the prompt is up");
    click_labeled(window, &frame.nodes, answer, view, ctx);
}

/// `pick` on a fresh dirty document, then `answer`: the view after.
fn answered(
    window: &mut Headless,
    ctx: &ToolContext,
    dir: &Path,
    pick: impl Fn(&mut Headless, &mut RulesetView, &ToolContext),
    button: &str,
) -> RulesetView {
    let mut view = dirty_view(ctx, dir);
    pick(window, &mut view, ctx);
    answer(window, &mut view, ctx, button);
    view
}

/// `label` picked on the modal start page (opened over a fresh dirty
/// document), then `button`: the view after.
fn answered_start(
    window: &mut Headless,
    ctx: &ToolContext,
    dir: &Path,
    label: &str,
    button: &str,
) -> RulesetView {
    let mut view = dirty_view(ctx, dir);
    open_start_modal(window, &mut view, ctx);
    let frame = window.draw(&mut view, ctx);
    click_prefix(window, &frame.nodes, label, &mut view, ctx);
    let frame = window.draw(&mut view, ctx);
    assert!(prompt_drawn(&frame.nodes), "the prompt is up");
    answer(window, &mut view, ctx, button);
    view
}

/// `label` picked on the modal start page prompts; each of its three
/// answers does what it says, `name` the document after Discard.
fn pick_answers(window: &mut Headless, ctx: &ToolContext, dir: &Path, label: &str, name: &str) {
    let view = answered_start(window, ctx, dir, label, "Cancel");
    assert_name(&view, "Edited");
    assert!(view.document.as_ref().unwrap().is_dirty());
    let view = answered_start(window, ctx, dir, label, "Discard");
    assert_name(&view, name);
    let view = answered_start(window, ctx, dir, label, "Save");
    assert_saved(dir, "edited.rhai", "Edited");
    assert_name(&view, name);
}

/// A click on the node `nodes` holds labeled `label`, on `tool`'s view
/// (the `hold_close` leg drives `Tool::view` as the shell does).
fn click_labeled_tool(
    window: &mut Headless,
    nodes: &[accesskit::Node],
    label: &str,
    tool: &mut crate::Tool,
    ctx: &ToolContext,
) {
    let at = centre(node(nodes, label));
    window.frame(pointer_at(at, true), |ui| tool.view(ui, ctx));
    window.frame(pointer_at(at, false), |ui| tool.view(ui, ctx));
}

// RE-GUI-15
#[test]
fn a_window_close_prompts_and_each_answer_does_what_it_says() {
    let dir = crate::test_support::scratch("prompt_close");
    let (ctx, _events, requests) = crate::test_support::context_requests();
    let mut window = Headless::new();

    // Cancel keeps everything: the document as it was, no close asked.
    let mut tool = crate::Tool::new();
    open_document(&mut tool.view, crate::document::blank());
    edit_ruleset(&mut tool.view, |ruleset| {
        ruleset.name = "Edited".to_string()
    });
    assert!(StudioTool::hold_close(&mut tool), "the close is held");
    let frame = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    click_labeled_tool(&mut window, &frame.nodes, "Cancel", &mut tool, &ctx);
    assert!(tool.view.pending_action.is_none());
    assert!(
        tool.view.document.as_ref().expect("kept").is_dirty(),
        "cancel keeps the document as it was"
    );
    assert!(
        requests.try_recv().is_err(),
        "no close is asked after Cancel"
    );

    // Discard goes on: the shell is asked to close, and the dropped
    // document lets the re-asked hold_close through.
    assert!(
        StudioTool::hold_close(&mut tool),
        "a later close asks again"
    );
    let frame = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    click_labeled_tool(&mut window, &frame.nodes, "Discard", &mut tool, &ctx);
    assert_eq!(
        requests.try_recv().expect("the close request"),
        ShellRequest::Close
    );
    assert!(
        !StudioTool::hold_close(&mut tool),
        "the re-asked close is not held"
    );

    // Save writes it first (Save as for a never-saved document) and
    // goes on.
    let mut tool = crate::Tool::new();
    open_document(&mut tool.view, crate::document::blank());
    edit_ruleset(&mut tool.view, |ruleset| {
        ruleset.name = "Edited".to_string()
    });
    let saved = dir.join("kept.rhai");
    tool.view.save_as_dialog = Box::new(move |_| Some(saved.clone()));
    assert!(StudioTool::hold_close(&mut tool));
    let frame = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    click_labeled_tool(&mut window, &frame.nodes, "Save", &mut tool, &ctx);
    assert_saved(&dir, "kept.rhai", "Edited");
    let sent: Vec<ShellRequest> = requests.try_iter().collect();
    assert!(
        sent.contains(&ShellRequest::Close),
        "the close is asked after the saved write (the settings write precedes it): {sent:?}"
    );
}

// RE-GUI-15
#[test]
fn an_open_prompts_and_each_answer_does_what_it_says() {
    let (dir, ctx, mut window) = prompt_setup("prompt_open");
    let other = write_file(&dir, "other.rhai", &official_text());

    // Cancel keeps the document as it was.
    let view = answered(
        &mut window,
        &ctx,
        &dir,
        |w, v, c| menu_pick(w, v, c, "File", "Open…"),
        "Cancel",
    );
    assert_eq!(
        view.document.as_ref().expect("kept").ruleset.name,
        "Edited",
        "cancel keeps the document as it was"
    );
    assert!(view.document.as_ref().unwrap().is_dirty());

    // Discard goes on: the file dialog follows and the file opens.
    let view = answered(
        &mut window,
        &ctx,
        &dir,
        |w, v, c| {
            let picked = other.clone();
            v.open_dialog = Box::new(move |_| Some(picked.clone()));
            menu_pick(w, v, c, "File", "Open…");
        },
        "Discard",
    );
    assert_eq!(
        view.document.as_ref().expect("opened").ruleset.name,
        "Autumn 26",
        "the picked file opened"
    );

    // Save writes it first, then goes on.
    let view = answered(
        &mut window,
        &ctx,
        &dir,
        |w, v, c| {
            let picked = other.clone();
            v.open_dialog = Box::new(move |_| Some(picked.clone()));
            menu_pick(w, v, c, "File", "Open…");
        },
        "Save",
    );
    assert_saved(&dir, "edited.rhai", "Edited");
    assert_eq!(
        view.document.as_ref().expect("opened").ruleset.name,
        "Autumn 26"
    );
}

/// File's "New from template…" opens the start page over the document
/// at once — nothing is lost there, so no prompt.
fn open_start_modal(window: &mut Headless, view: &mut RulesetView, ctx: &ToolContext) {
    let frame = window.draw(view, ctx);
    click_labeled(window, &frame.nodes, "File", view, ctx);
    let frame = window.draw(view, ctx);
    click_prefix(window, &frame.nodes, "New from template…", view, ctx);
    window.draw(view, ctx);
    assert!(view.start_open, "the start page opened");
    assert!(
        view.pending_action.is_none(),
        "no prompt for the page itself"
    );
}

// RE-GUI-15
#[test]
fn new_from_template_is_unguarded_but_a_pick_in_it_prompts() {
    let (dir, ctx, mut window) = prompt_setup("prompt_new");

    // On a dirty document File > New from template draws the start page
    // and no prompt (Close returns to the document as it was).
    let mut view = dirty_view(&ctx, &dir);
    open_start_modal(&mut window, &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    assert!(drawn(&frame.nodes, "New from template"));
    assert!(
        !prompt_drawn(&frame.nodes),
        "the page itself is not a way out"
    );

    // Picking a template there meets the prompt; Discard opens it.
    click_prefix(&mut window, &frame.nodes, "Spring 26", &mut view, &ctx);
    let frame = window.draw(&mut view, &ctx);
    assert!(prompt_drawn(&frame.nodes), "the prompt is up");
    click_labeled(&mut window, &frame.nodes, "Discard", &mut view, &ctx);
    assert_eq!(
        view.document.as_ref().expect("opened").ruleset.name,
        "Spring 26"
    );
}

// RE-GUI-15
#[test]
fn a_recent_file_pick_prompts_and_each_answer_does_what_it_says() {
    let (dir, ctx, mut window) = prompt_setup("prompt_recent");
    // The recent file the list offers, seeded first; each leg's open of
    // `edited.rhai` then adds its own entry above it.
    let other = write_file(&dir, "other.rhai", &official_text());
    ctx.set_tool_settings(
        crate::messages::TOOL_ID,
        crate::settings::RulesetEditorSettings {
            last_folder: String::new(),
            recent_files: vec![other.to_string_lossy().into_owned()],
        }
        .to_table(),
    );

    pick_answers(&mut window, &ctx, &dir, "other.rhai", "Autumn 26");
}

// RE-GUI-15
#[test]
fn a_template_pick_prompts_and_each_answer_does_what_it_says() {
    let (dir, ctx, mut window) = prompt_setup("prompt_template");

    pick_answers(&mut window, &ctx, &dir, "Spring 26", "Spring 26");
}
