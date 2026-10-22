//! The headless egui harness the crate's widget tests share: a context with
//! AccessKit on, run a frame at a time over an 800 by 600 screen, plus the
//! click and painted-text helpers.

use egui::accesskit::{self, Role};
use pes_savefile::EditFile;
use pes_savefile::model::names::display_name;
use pes_savefile::model::tactics::TeamTactics;
use pes_version::PesVersion;

use crate::tactics::RosterRow;

/// A headless window: one egui context with AccessKit on, run a frame at a
/// time over an 800 by 600 screen, for drawing a bare `Ui`'s widgets.
pub(crate) struct Headless {
    /// The context: the painted-color assertions read the theme's visuals
    /// off it.
    pub(crate) ctx: egui::Context,
    /// The screen size each frame reports.
    screen: egui::Vec2,
}

impl Headless {
    /// A window with AccessKit enabled.
    pub(crate) fn new() -> Headless {
        Headless::sized(800.0, 600.0)
    }

    /// `new` over a `width` by `height` screen.
    pub(crate) fn sized(width: f32, height: f32) -> Headless {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        Headless {
            ctx,
            screen: egui::vec2(width, height),
        }
    }

    /// Runs one frame with `events` as its input; returns the frame's
    /// AccessKit nodes and painted shapes.
    pub(crate) fn frame(
        &mut self,
        events: Vec<egui::Event>,
        contents: impl FnMut(&mut egui::Ui),
    ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        self.frame_at(None, events, contents)
    }

    /// `frame` with `time` as the frame's timestamp (a tooltip's delay counts
    /// in seconds of it).
    pub(crate) fn frame_at(
        &mut self,
        time: Option<f64>,
        events: Vec<egui::Event>,
        contents: impl FnMut(&mut egui::Ui),
    ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        let (nodes, shapes) = self.frame_at_ids(time, events, contents);
        (nodes.into_iter().map(|(_, node)| node).collect(), shapes)
    }

    /// `frame_at` keeping the nodes' AccessKit ids (for
    /// `AccessKitActionRequest` events, which target a node by id).
    pub(crate) fn frame_at_ids(
        &mut self,
        time: Option<f64>,
        events: Vec<egui::Event>,
        contents: impl FnMut(&mut egui::Ui),
    ) -> (
        Vec<(accesskit::NodeId, accesskit::Node)>,
        Vec<egui::epaint::ClippedShape>,
    ) {
        let input = egui::RawInput {
            time,
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
        let shapes = std::mem::take(&mut output.shapes);
        // Nothing renders here, so the frame's texture deltas are dropped on
        // purpose.
        output.drop_without_applying_deltas();
        (update.nodes, shapes)
    }
}

/// Team 701's `TeamTactics` of `version`'s fixture save.
pub(crate) fn fixture_tactics(version: PesVersion) -> TeamTactics {
    EditFile::from_bytes(&pes_savefile::fixture_save(version))
        .expect("fixture save")
        .team(701)
        .expect("team 701 is in the fixture")
        .tactics
        .clone()
}

/// Team 701's roster as the widgets' `roster` sees it, indexed by slot:
/// `(display name, raw name, registered position)`, `("", "", None)` an
/// empty slot, as the hosts build it.
pub(crate) fn fixture_rows(version: PesVersion) -> Vec<(String, String, Option<u8>)> {
    let file = EditFile::from_bytes(&pes_savefile::fixture_save(version)).expect("fixture save");
    let team = file.team(701).expect("team 701 is in the fixture");
    team.roster
        .iter()
        .map(|slot| {
            file.player(slot.player_id).map_or_else(
                || (String::new(), String::new(), None),
                |player| {
                    (
                        display_name(&player.name),
                        player.name.clone(),
                        Some(player.positions.registered),
                    )
                },
            )
        })
        .collect()
}

/// `rows` as the `&[RosterRow]` the widgets take.
pub(crate) fn roster_rows(rows: &[(String, String, Option<u8>)]) -> Vec<RosterRow<'_>> {
    rows.iter()
        .map(|(name, raw_name, position)| RosterRow {
            name: name.as_str(),
            raw_name: raw_name.as_str(),
            registered_position: *position,
        })
        .collect()
}

/// The widget node of `role` labeled `label`.
pub(crate) fn entry<'a>(
    nodes: &'a [accesskit::Node],
    role: Role,
    label: &str,
) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == role && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no {role:?} labeled {label:?}"))
}

/// The nodes of `role` right of `label`'s, on its row, in x order.
pub(crate) fn controls_right_of<'a>(
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

/// The first node of `role` right of `label`'s, on its row.
pub(crate) fn control_right_of<'a>(
    nodes: &'a [accesskit::Node],
    label: &str,
    role: Role,
) -> &'a accesskit::Node {
    controls_right_of(nodes, label, role)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no {role:?} right of {label:?}"))
}

/// `nodes`' buttons' labels in draw order (top to bottom, left to
/// right), kept only when `keep` holds.
pub(crate) fn button_labels(nodes: &[accesskit::Node], keep: impl Fn(&str) -> bool) -> Vec<String> {
    let mut buttons: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| node.role() == Role::Button && node.label().is_some_and(&keep))
        .collect();
    buttons.sort_by(|a, b| {
        let key = |node: &&accesskit::Node| {
            let bounds = node.bounds().expect("drawn bounds");
            (bounds.y0, bounds.x0)
        };
        key(a).partial_cmp(&key(b)).expect("ordered bounds")
    });
    buttons
        .iter()
        .filter_map(|node| node.label().map(str::to_owned))
        .collect()
}

/// The AccessKit id of the widget node of `role` labeled `label`.
pub(crate) fn entry_id(
    nodes: &[(accesskit::NodeId, accesskit::Node)],
    role: Role,
    label: &str,
) -> accesskit::NodeId {
    nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no {role:?} labeled {label:?}"))
}

/// The centre of `node`'s bounds, where a click lands.
pub(crate) fn centre(node: &accesskit::Node) -> egui::Pos2 {
    let bounds = node.bounds().expect("a widget has bounds");
    egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

/// The primary button pressed (or released) at `pos`, the pointer moved
/// there first.
pub(crate) fn pointer_at(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
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

/// The color `shape`'s text paints in: a section's own color when one was
/// set, else the shape's fallback.
pub(crate) fn painted_color(shape: &egui::epaint::TextShape) -> egui::Color32 {
    if let Some(overridden) = shape.override_text_color {
        return overridden;
    }
    shape
        .galley
        .job
        .sections
        .iter()
        .map(|section| section.format.color)
        .find(|&color| color != egui::Color32::PLACEHOLDER)
        .unwrap_or(shape.fallback_color)
}

/// The painted text shape whose galley text starts with `starts_with`.
pub(crate) fn text_shape<'a>(
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

/// The color the text `text` was painted in this frame.
pub(crate) fn painted_text_color(
    shapes: &[egui::epaint::ClippedShape],
    text: &str,
) -> egui::Color32 {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == text => Some(painted_color(shape)),
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("no painted text {text:?}"))
}
