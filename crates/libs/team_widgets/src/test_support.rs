//! The headless egui harness the crate's widget tests share: a context with
//! AccessKit on, run a frame at a time over an 800 by 600 screen, plus the
//! click and painted-text helpers.

use egui::accesskit;

/// A headless window: one egui context with AccessKit on, run a frame at a
/// time over an 800 by 600 screen, for drawing a bare `Ui`'s widgets.
pub(crate) struct Headless {
    /// The context: the painted-color assertions read the theme's visuals
    /// off it.
    pub(crate) ctx: egui::Context,
}

impl Headless {
    /// A window with AccessKit enabled.
    pub(crate) fn new() -> Headless {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        Headless { ctx }
    }

    /// Runs one frame with `events` as its input; returns the frame's
    /// AccessKit nodes and painted shapes.
    pub(crate) fn frame(
        &mut self,
        events: Vec<egui::Event>,
        contents: impl FnMut(&mut egui::Ui),
    ) -> (Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
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
        (
            update.nodes.into_iter().map(|(_, node)| node).collect(),
            shapes,
        )
    }
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
