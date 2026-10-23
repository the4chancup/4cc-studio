//! The headless-window tests of the view's scenarios
//! (`plans/ruleset_editor.md` "Acceptance", the Phase 20 scope
//! paragraph): one egui context with AccessKit on, run a frame at a
//! time; what a frame draws is its AccessKit nodes, what the clipboard
//! holds is its platform output, and a mark's colour is read back from
//! the view state the frame paints from.

mod custom_checks;
mod file;
mod general;
mod preview;
mod prompt;
mod start;
mod summary;

use std::path::{Path, PathBuf};

use crossbeam_channel::Receiver;
use egui::accesskit::{self, Role};
use studio_core::{PipelineEventEnvelope, ToolContext};

use super::*;

/// A headless window: one egui context with AccessKit on and animations
/// off, run a frame at a time over an 800 by 600 screen (the Save
/// editor's `Headless`).
struct Headless {
    ctx: egui::Context,
    screen: egui::Vec2,
}

/// What a frame drew: its AccessKit nodes and its platform output (the
/// clipboard's `CopyText` command).
struct Frame {
    nodes: Vec<accesskit::Node>,
    platform: egui::PlatformOutput,
}

impl Headless {
    fn new() -> Headless {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.global_style_mut(|style| style.animation_time = 0.0);
        ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
        Headless {
            ctx,
            screen: egui::vec2(800.0, 600.0),
        }
    }

    /// Runs one frame of `contents`; returns what it drew.
    fn frame(&mut self, events: Vec<egui::Event>, contents: impl FnMut(&mut egui::Ui)) -> Frame {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.screen)),
            events,
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(input, contents);
        let update = output
            .platform_output
            .accesskit_update
            .take()
            .expect("AccessKit is enabled on the context");
        let platform = std::mem::take(&mut output.platform_output);
        // Nothing renders here, so the frame's texture deltas are
        // dropped on purpose.
        output.drop_without_applying_deltas();
        Frame {
            nodes: update.nodes.into_iter().map(|(_, node)| node).collect(),
            platform,
        }
    }

    /// One quiet frame of the view.
    fn draw(&mut self, view: &mut RulesetView, ctx: &ToolContext) -> Frame {
        self.frame(Vec::new(), |ui| show(ui, view, ctx))
    }
}

/// The centre of a node's bounds, where a click lands.
fn centre(node: &accesskit::Node) -> egui::Pos2 {
    let bounds = node.bounds().expect("a widget has bounds");
    egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

/// The primary button pressed (or released) at `pos`, the pointer moved
/// there first.
fn pointer_at(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        },
    ]
}

/// A click on `node`: press on one frame, release on the next.
fn click(window: &mut Headless, node: &accesskit::Node, view: &mut RulesetView, ctx: &ToolContext) {
    let at = centre(node);
    window.frame(pointer_at(at, true), |ui| show(ui, view, ctx));
    window.frame(pointer_at(at, false), |ui| show(ui, view, ctx));
}

/// A click on the node `nodes` holds labeled exactly `label`.
fn click_labeled(
    window: &mut Headless,
    nodes: &[accesskit::Node],
    label: &str,
    view: &mut RulesetView,
    ctx: &ToolContext,
) {
    click(window, node(nodes, label), view, ctx);
}

/// A click on the node whose label starts with `prefix` (menu items
/// carry their shortcut in the label).
fn click_prefix(
    window: &mut Headless,
    nodes: &[accesskit::Node],
    prefix: &str,
    view: &mut RulesetView,
    ctx: &ToolContext,
) {
    let target = nodes
        .iter()
        .find(|node| node.label().is_some_and(|label| label.starts_with(prefix)))
        .unwrap_or_else(|| panic!("no node labeled {prefix:?}…"));
    click(window, target, view, ctx);
}

/// A key pressed with `modifiers` held.
fn key(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        modifiers,
        repeat: false,
    }
}

/// The node labeled (or valued) `label`.
fn node<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.label() == Some(label) || node.value() == Some(label))
        .unwrap_or_else(|| panic!("no node labeled {label:?}"))
}

/// Every node labeled `label` top to bottom (the two stat checklists
/// share names).
fn nodes_labeled<'a>(nodes: &'a [accesskit::Node], label: &str) -> Vec<&'a accesskit::Node> {
    let mut found: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.label() == Some(label) || node.value() == Some(label))
        .collect();
    found.sort_by(|a, b| {
        let y = |node: &&accesskit::Node| node.bounds().map(|rect| rect.y0).unwrap_or(0.0);
        y(a).partial_cmp(&y(b)).expect("ordered bounds")
    });
    found
}

/// Every node's label (or value), top to bottom then left to right
/// (the AccessKit update is unordered; a node without bounds sorts
/// last).
fn labels(nodes: &[accesskit::Node]) -> Vec<String> {
    let mut labelled: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.label().is_some() || node.value().is_some())
        .collect();
    labelled.sort_by(|a, b| {
        let at = |node: &&accesskit::Node| {
            node.bounds()
                .map(|rect| (rect.y0, rect.x0))
                .unwrap_or((f64::MAX, f64::MAX))
        };
        at(a).partial_cmp(&at(b)).expect("ordered bounds")
    });
    labelled
        .iter()
        .filter_map(|node| node.label().or_else(|| node.value()).map(str::to_owned))
        .collect()
}

/// Whether a node labeled `label` was drawn.
fn drawn(nodes: &[accesskit::Node], label: &str) -> bool {
    nodes
        .iter()
        .any(|node| node.label() == Some(label) || node.value() == Some(label))
}

/// Whether a label (or value) containing `fragment` was drawn.
fn contains_label(nodes: &[accesskit::Node], fragment: &str) -> bool {
    labels(nodes).iter().any(|label| label.contains(fragment))
}

/// `text` written to `dir/name`; the path.
fn write_file(dir: &Path, name: &str, text: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    path
}

/// `test`'s scratch folder with `official_text()` written to `name`
/// inside; folder and file.
fn scratch_file(test: &str, name: &str) -> (PathBuf, PathBuf) {
    let dir = crate::test_support::scratch(test);
    let file = write_file(&dir, name, &official_text());
    (dir, file)
}

/// The embedded official ruleset's text as an assembled file.
fn official_text() -> String {
    aatf::assemble_file(aatf::Rules::official().ruleset(), None, None)
}

/// A view with `ruleset` opened as a new, unsaved document (a template
/// or Blank pick's result without the pick).
fn open_document(view: &mut RulesetView, ruleset: aatf::Ruleset) {
    view.set_document(Document::new(ruleset));
}

/// `view`'s document's ruleset edited in place: the tests' shortcut for
/// a change that marks the document dirty (the undo gesture itself is
/// exercised by the checkbox clicks).
fn edit_ruleset(view: &mut RulesetView, edit: impl FnOnce(&mut aatf::Ruleset)) {
    edit(&mut view.document.as_mut().expect("a document").ruleset);
}

/// A fresh context, headless window and view.
fn fresh() -> (
    ToolContext,
    Receiver<PipelineEventEnvelope>,
    Headless,
    RulesetView,
) {
    let (ctx, events) = crate::test_support::context();
    (ctx, events, Headless::new(), RulesetView::new())
}

/// `view` with `path` opened.
fn opened_view(ctx: &ToolContext, path: &Path) -> RulesetView {
    let mut view = RulesetView::new();
    open_path(&mut view, ctx, path);
    view
}

/// `test`'s scratch `rules.rhai` opened in a fresh view: context, file,
/// view.
fn official_opened(
    test: &str,
) -> (
    ToolContext,
    Receiver<PipelineEventEnvelope>,
    PathBuf,
    RulesetView,
) {
    let (_dir, path) = scratch_file(test, "rules.rhai");
    let (ctx, events) = crate::test_support::context();
    let view = opened_view(&ctx, &path);
    (ctx, events, path, view)
}

/// The view's document's ruleset is named `name`.
fn assert_name(view: &RulesetView, name: &str) {
    assert_eq!(
        view.document.as_ref().expect("a document").ruleset.name,
        name
    );
}

/// `dir`'s `file` was written with `name` as the ruleset's name.
fn assert_saved(dir: &Path, file: &str, name: &str) {
    let text = std::fs::read_to_string(dir.join(file)).unwrap();
    assert!(
        text.contains(&format!("name: \"{name}\"")),
        "the file was written first"
    );
}
