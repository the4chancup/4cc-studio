//! The headless-window tests of the view's scenarios (`plans/save_editor.md`
//! "Acceptance", the Phase 8 scope paragraph): one egui context with AccessKit
//! on, run a frame at a time; what a frame draws is its AccessKit nodes and
//! painted shapes.

use std::path::PathBuf;

use egui::accesskit::{self, Role};
use pes_savefile::model::ingame_face::IngameFace;
use pes_savefile::model::names::name_spans;
use pes_savefile::schema::fields::PlayerField;
use pes_savefile::{EditFile, fixture_save};
use pes_version::PesVersion;

use super::*;

/// A headless window: one egui context with AccessKit on and animations off,
/// run a frame at a time over an 800 by 600 screen (wider when a test asks,
/// the Abilities tab's rightmost controls sit past 800).
struct Headless {
    ctx: egui::Context,
    /// The screen size each frame reports.
    screen: egui::Vec2,
}

impl Headless {
    fn new() -> Headless {
        Headless::sized(800.0, 600.0)
    }

    /// A window `width` by `height` instead of the default 800 by 600.
    fn sized(width: f32, height: f32) -> Headless {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.global_style_mut(|style| style.animation_time = 0.0);
        ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
        Headless {
            ctx,
            screen: egui::vec2(width, height),
        }
    }

    /// Runs one frame of `contents`; returns what it returned, the frame's
    /// AccessKit nodes and its painted shapes.
    fn frame<R>(
        &mut self,
        events: Vec<egui::Event>,
        contents: impl FnMut(&mut egui::Ui) -> R,
    ) -> (R, Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        let (result, nodes, shapes) = self.frame_ids(events, contents);
        (
            result,
            nodes.into_iter().map(|(_, node)| node).collect(),
            shapes,
        )
    }

    /// `frame` keeping the nodes' AccessKit ids (for `AccessKitActionRequest`
    /// events).
    fn frame_ids<R>(
        &mut self,
        events: Vec<egui::Event>,
        mut contents: impl FnMut(&mut egui::Ui) -> R,
    ) -> (
        R,
        Vec<(accesskit::NodeId, accesskit::Node)>,
        Vec<egui::epaint::ClippedShape>,
    ) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.screen)),
            events,
            ..Default::default()
        };
        let mut result = None;
        let mut output = self.ctx.run_ui(input, |ui| result = Some(contents(ui)));
        let update = output
            .platform_output
            .accesskit_update
            .take()
            .expect("AccessKit is enabled on the context");
        let shapes = std::mem::take(&mut output.shapes);
        // Nothing renders here, so the frame's texture deltas are dropped on
        // purpose.
        output.drop_without_applying_deltas();
        (
            result.expect("run_ui calls the closure"),
            update.nodes,
            shapes,
        )
    }
}

/// Writes `version`'s fixture save under `test`'s scratch folder (fresh each
/// call, so a `.bak` an older run left cannot shadow the assertions).
fn fixture(version: PesVersion, test: &str) -> PathBuf {
    let dir = crate::test_support::scratch(test);
    let path = dir.join("EDIT00000000");
    std::fs::write(&path, fixture_save(version)).unwrap();
    path
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

/// A press on `from`'s centre, the pointer dragged to `onto`'s and released:
/// a drag of `from`'s widget onto `onto`'s.
fn drag(window: &mut Headless, from: egui::Pos2, onto: egui::Pos2, view: &mut SaveView) {
    window.frame(pointer_at(from, true), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    window.frame(vec![egui::Event::PointerMoved(onto)], |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    window.frame(pointer_at(onto, false), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
}

/// `entry` without what a roster slot owns (`id`, `appearance_id`,
/// `appearance.base_copy_id`, and the in-game face run apart from the
/// exported fields): the record a drag reorder carries.
fn record(entry: &PlayerEntry) -> PlayerEntry {
    let mut record = entry.clone();
    record.id = 0;
    record.appearance_id = None;
    record.appearance.base_copy_id = 0;
    record.appearance.ingame_face = IngameFace::default();
    record
}

/// The list's player rows' labels in screen order (the AccessKit update is
/// unordered).
fn row_labels(nodes: &[accesskit::Node]) -> Vec<String> {
    let mut rows: Vec<&accesskit::Node> = nodes.iter().filter(|node| player_row(node)).collect();
    rows.sort_by(|a, b| {
        let y = |node: &&accesskit::Node| node.bounds().map(|r| r.y0).unwrap_or(0.0);
        y(a).partial_cmp(&y(b)).expect("ordered bounds")
    });
    rows.iter()
        .filter_map(|node| node.label().map(str::to_owned))
        .collect()
}

/// A click on `node`: press on one frame, release on the next.
fn click(window: &mut Headless, node: &accesskit::Node, view: &mut SaveView) {
    let at = centre(node);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    window.frame(pointer_at(at, false), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
}

/// An AccessKit `Click` action on `id`: a click delivered as the
/// accessibility event, for widgets a pointer click does not reach (a
/// combo's popup items).
fn action_click(window: &mut Headless, view: &mut SaveView, id: accesskit::NodeId) {
    window.frame(
        vec![egui::Event::AccessKitActionRequest(
            accesskit::ActionRequest {
                action: accesskit::Action::Click,
                target_tree: accesskit::TreeId::ROOT,
                target_node: id,
                data: None,
            },
        )],
        |ui| show(ui, view, &|| Vec::new(), None),
    );
}

/// An AccessKit `Focus` action on `id`: keyboard focus without a click.
fn action_focus(window: &mut Headless, view: &mut SaveView, id: accesskit::NodeId) {
    window.frame(
        vec![egui::Event::AccessKitActionRequest(
            accesskit::ActionRequest {
                action: accesskit::Action::Focus,
                target_tree: accesskit::TreeId::ROOT,
                target_node: id,
                data: None,
            },
        )],
        |ui| show(ui, view, &|| Vec::new(), None),
    );
}

/// A key pressed (or released) with `modifiers` held.
fn key(key: egui::Key, modifiers: egui::Modifiers, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        modifiers,
        repeat: false,
    }
}

/// The AccessKit node of the row labeled `label`.
fn row<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no row {label:?}"))
}

/// Whether `node` is a player row of the list (`{number} {position} …`): a
/// Button whose label starts with the shirt number's digit.
fn player_row(node: &accesskit::Node) -> bool {
    node.role() == Role::Button
        && node
            .label()
            .and_then(|label| label.chars().next())
            .is_some_and(|c| c.is_ascii_digit())
}

/// The painted text shape whose galley text starts with `starts_with`.
fn text_shape<'a>(
    shapes: &'a [egui::epaint::ClippedShape],
    starts_with: &str,
) -> &'a egui::epaint::TextShape {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text.starts_with(starts_with) => {
                Some(shape)
            }
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("no painted text starting {starts_with:?}"))
}

/// The fill under `bounds`' centre: the smallest `Shape::Rect` covering
/// it (the field's own frame; the panel's background would win an
/// `intersects` match instead).
fn fill_at(
    shapes: &[egui::epaint::ClippedShape],
    bounds: accesskit::Rect,
) -> Option<egui::Color32> {
    let centre = egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if shape.rect.contains(centre) => {
                Some((shape.fill, shape.rect.area()))
            }
            _ => None,
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).expect("areas"))
        .map(|(fill, _)| fill)
}

/// The Shirt field's `TextInput` bounds (the rightmost enabled text edit
/// holding `value`).
fn shirt_field(nodes: &[accesskit::Node], value: &str) -> accesskit::Rect {
    nodes
        .iter()
        .filter(|node| {
            node.role() == Role::TextInput && !node.is_disabled() && node.value() == Some(value)
        })
        .map(|node| node.bounds().expect("bounds"))
        .max_by(|a, b| a.x0.partial_cmp(&b.x0).expect("ordered bounds"))
        .unwrap_or_else(|| panic!("no Shirt field {value:?}"))
}

/// The undo stack's depth, counted by Ctrl+Z until it is empty (bounded, so
/// a stuck stack fails rather than hangs).
fn undo_entries(window: &mut Headless, view: &mut SaveView) -> usize {
    for count in 0..64 {
        if !view.session.as_ref().is_some_and(|s| s.can_undo()) {
            return count;
        }
        window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
            show(ui, view, &|| Vec::new(), None)
        });
    }
    panic!("Ctrl+Z did not empty the undo stack in 64 presses")
}

/// The widget nodes of `role` on `label`'s row to its right, in x order.
fn right_of_label<'a>(
    nodes: &'a [accesskit::Node],
    label: &str,
    role: Role,
) -> Vec<&'a accesskit::Node> {
    let label_node = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some(label))
        .unwrap_or_else(|| panic!("no label {label:?}"));
    let bounds = label_node.bounds().expect("the label's bounds");
    let mut found: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == role
                && node
                    .bounds()
                    .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
        })
        .collect();
    found.sort_by(|a, b| {
        let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
        x(a).partial_cmp(&x(b)).expect("ordered bounds")
    });
    found
}

/// The `C` `B` `A` buttons of `position`'s cell, in row order.
fn rating_buttons<'a>(nodes: &'a [accesskit::Node], position: &str) -> Vec<&'a accesskit::Node> {
    let ratings: Vec<&accesskit::Node> = right_of_label(nodes, position, Role::Button)
        .into_iter()
        .filter(|node| matches!(node.label(), Some("C" | "B" | "A")))
        .take(3)
        .collect();
    assert_eq!(ratings.len(), 3, "{position}'s three ratings");
    ratings
}

/// The first interactive node (any role but Label) on `label`'s row to its
/// right: the control the label names.
fn control_right_of<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    let label_node = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some(label))
        .unwrap_or_else(|| panic!("no label {label:?}"));
    let bounds = label_node.bounds().expect("the label's bounds");
    nodes
        .iter()
        .filter(|node| {
            node.role() != Role::Label
                && node
                    .bounds()
                    .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
        })
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .unwrap_or_else(|| panic!("no control right of {label:?}"))
}

/// The disabled `-` button right of `label` (a gated number control).
fn disabled_dash<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    right_of_label(nodes, label, Role::Button)
        .into_iter()
        .find(|node| node.is_disabled() && node.label() == Some("-"))
        .unwrap_or_else(|| panic!("no disabled '-' after {label:?}"))
}

/// The tooltip text after the pointer rests at `pos` for a while (bounded:
/// a missing tooltip fails rather than hangs). The pointer first moves away
/// for a frame: egui refuses a tooltip while the last click is more recent
/// than the last movement plus 0.1 s, and both clocks advance together, so
/// a `PointerMoved` that only repeats the clicked position never clears it.
fn tooltip_after_hover(
    window: &mut Headless,
    view: &mut SaveView,
    pos: egui::Pos2,
) -> Option<String> {
    tooltip_containing(window, view, pos, "PES ")
}

/// `tooltip_after_hover` filtered to the tooltip node holding `needle`.
/// The hover must add a node holding `needle` to the frame the pointer
/// left from: a label the view always draws with the same text (the
/// ownership line, an error line) is counted there already, so it cannot
/// pass for the tooltip.
fn tooltip_containing(
    window: &mut Headless,
    view: &mut SaveView,
    pos: egui::Pos2,
    needle: &str,
) -> Option<String> {
    let holding = |nodes: &[accesskit::Node]| -> Vec<String> {
        nodes
            .iter()
            .filter_map(|node| node.value())
            .filter(|value| value.contains(needle))
            .map(str::to_owned)
            .collect()
    };
    let (_, resting, _) = window.frame(vec![egui::Event::PointerMoved(egui::Pos2::ZERO)], |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    let resting = holding(&resting);
    window.frame(vec![egui::Event::PointerMoved(pos)], |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    let count = |values: &[String], value: &String| values.iter().filter(|v| *v == value).count();
    for _ in 0..90 {
        let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
        let values = holding(&nodes);
        if let Some(added) = values
            .iter()
            .find(|value| count(&values, value) > count(&resting, value))
        {
            return Some(added.clone());
        }
    }
    None
}

mod abilities;
mod appearance;
mod list;
mod player;
mod savefile;
mod shell;
mod strip;
mod tactics;
mod team;
