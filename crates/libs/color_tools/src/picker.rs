//! The suite's color field: a color button opening the picker popup of
//! `plans/libs/color_tools.md` "Color-picker widget (Phase 8)", drafted in
//! `docs/prototypes/save_editor.html` (the `PICKER` section). The button shows the color
//! (or the live candidate while a swatch is hovered or a drag runs); the popup edits it
//! through a saturation-value square and a hue strip, a hex field, R/G/B fields, the
//! host's suggestion swatches and the session's recent colors. `color` moves only on a
//! finished gesture, so the host's undo takes one step per gesture.

use egui::ecolor::Hsva;
use egui::emath::{lerp, remap_clamp};
use egui::{
    Align, Color32, DragValue, Key, Layout, Mesh, Popup, PopupCloseBehavior, Rect, Response, Sense,
    Shape, Stroke, StrokeKind, TextEdit, TextStyle, Ui, WidgetInfo, WidgetType, pos2, vec2,
};

/// The most recent colors the session keeps: newest first, no duplicates.
const MAX_RECENTS: usize = 8;

/// What one frame of a color field reports to its host.
pub struct ColorFieldOutput {
    /// The color button's response.
    pub response: egui::Response,
    /// A finished gesture changed the color this frame: a released drag on the square, the
    /// hue strip or an R/G/B field, a committed hex text or R/G/B value, a swatch click. True
    /// once per gesture, so the host's undo takes one step per gesture.
    pub changed: bool,
    /// The color a hovered swatch or a running drag would set, for the host's live preview;
    /// `None` when nothing is previewed.
    pub candidate: Option<[u8; 3]>,
}

/// A color button that opens the suite's picker popup (`plans/libs/color_tools.md`
/// "Color-picker widget (Phase 8)"). `color` changes only on a finished gesture.
pub fn color_field(
    ui: &mut Ui,
    id_salt: impl egui::AsIdSalt,
    color: &mut [u8; 3],
    suggestions: &[[u8; 3]],
) -> ColorFieldOutput {
    ui.push_id(id_salt, |ui| field(ui, color, suggestions))
        .inner
}

/// The field over the pushed `id_salt` scope: the button, the popup, the open bookkeeping
/// and the recent-colors rule on close.
fn field(ui: &mut Ui, color: &mut [u8; 3], suggestions: &[[u8; 3]]) -> ColorFieldOutput {
    let popup_id = ui.make_persistent_id("picker popup");
    let state_id = ui.make_persistent_id("picker state");
    let was_open = Popup::is_id_open(ui.ctx(), popup_id);
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<PickerState>(state_id))
        .unwrap_or_else(|| PickerState::new(*color));
    if !was_open {
        // egui keeps a single popup per viewport: a sibling field's popup opening, or a
        // frame where this field was not drawn, closed this one between frames, and the
        // close rule applies as if it had closed here.
        if state.open && *color != state.opened_on {
            push_recent(ui.ctx(), *color);
        }
        state = PickerState::new(*color);
    }

    let mut response = color_button(ui, state.preview.unwrap_or(*color), was_open);
    let mut changed = false;
    Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .layout(Layout::top_down_justified(Align::Min))
        .show(|ui| popup_body(ui, &mut state, color, &mut changed, suggestions));
    if changed {
        response.mark_changed();
    }

    let is_open = Popup::is_id_open(ui.ctx(), popup_id);
    if was_open && !is_open && *color != state.opened_on {
        push_recent(ui.ctx(), *color);
    }
    state.open = is_open;
    let candidate = state.preview;
    ui.ctx().data_mut(|data| data.insert_temp(state_id, state));
    ColorFieldOutput {
        response,
        changed,
        candidate,
    }
}

/// The field's face: a button filled with `rgb` (`open` draws egui's open-widget style
/// while its popup is up). Its label carries the shown hex so a screen reader and the
/// tests can tell fields apart.
fn color_button(ui: &mut Ui, rgb: [u8; 3], open: bool) -> Response {
    let size = ui.spacing().interact_size;
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(
            WidgetType::ColorButton,
            ui.is_enabled(),
            format!("color field {}", format_hex(rgb)),
        )
    });
    if ui.is_rect_visible(rect) {
        let visuals = if open {
            &ui.visuals().widgets.open
        } else {
            ui.style().interact(&response)
        };
        let rect = rect.expand(visuals.expansion);
        ui.painter().rect_filled(
            rect.shrink(1.0),
            visuals.corner_radius.at_most(2),
            Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
        );
        ui.painter().rect_stroke(
            rect,
            visuals.corner_radius.at_most(2),
            (1.0, visuals.bg_fill),
            StrokeKind::Inside,
        );
    }
    response
}

/// `#RRGGBB` upper-case, matching the export text formats: how the hex field shows the
/// color and what a swatch's hover text and label are.
fn format_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// `#RRGGBB` parsed back: the `#` optional, either case, surrounding space ignored; any
/// other text is `None` (the field is put back to the current color).
fn parse_hex(text: &str) -> Option<[u8; 3]> {
    let text = text.trim();
    let digits = text.strip_prefix('#').unwrap_or(text);
    if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut rgb = [0; 3];
    for (index, channel) in rgb.iter_mut().enumerate() {
        *channel = u8::from_str_radix(&digits[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(rgb)
}

/// The recent-colors rule on the recents vector itself: `color` moves to the front (no
/// duplicates) and the row never grows past `MAX_RECENTS`.
fn push_recent_color(recents: &mut Vec<[u8; 3]>, color: [u8; 3]) {
    recents.retain(|&existing| existing != color);
    recents.insert(0, color);
    recents.truncate(MAX_RECENTS);
}

/// The egui id the session's recent colors live under: temp memory (never saved), shared
/// by every field of the program.
const RECENTS_ID: &str = "4cc studio color_tools recent colors";

/// The recents row as it stands in the context's temp memory.
fn recent_colors(ctx: &egui::Context) -> Vec<[u8; 3]> {
    ctx.data(|data| data.get_temp(egui::Id::new(RECENTS_ID)))
        .unwrap_or_default()
}

/// `color` joins the session's recents (called when a popup closes on a color other than
/// the one it opened on).
fn push_recent(ctx: &egui::Context, color: [u8; 3]) {
    ctx.data_mut(|data| {
        let recents = data.get_temp_mut_or_default::<Vec<[u8; 3]>>(egui::Id::new(RECENTS_ID));
        push_recent_color(recents, color);
    });
}

/// `[u8; 3]` sRGB as egui's `Hsva` (alpha pinned opaque: no export or save stores one).
fn hsva_of(rgb: [u8; 3]) -> Hsva {
    Hsva::from(Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
}

/// The working `Hsva` as `[u8; 3]` sRGB.
fn rgb_of(hsva: Hsva) -> [u8; 3] {
    let color = Color32::from(hsva);
    [color.r(), color.g(), color.b()]
}

/// `rgb` as `Color32`.
fn color32(rgb: [u8; 3]) -> Color32 {
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

/// The open popup's working state, kept in egui temp memory under the field's id so
/// typing, drags and the close rule survive the frames they span.
#[derive(Clone)]
struct PickerState {
    /// The color the popup opened on: the recent-colors rule compares the close color to
    /// it.
    opened_on: [u8; 3],
    /// The working color: what the square, the strip and the fields edit until a gesture
    /// commits it.
    hsva: Hsva,
    /// The hex field's text, kept while it is being typed.
    hex: String,
    /// The preview the button face and the host show right now (a hovered swatch or a
    /// running drag); `None` when nothing is previewed.
    preview: Option<[u8; 3]>,
    /// A press on the square, the strip or an R/G/B field is still held: the working color
    /// is only a candidate until it releases.
    dragging: bool,
    /// Whether the popup was open at the end of the last frame drawn: egui keeps one popup
    /// per viewport, so a sibling field's popup opening, or a frame where this field is not
    /// drawn, closes this one between frames.
    open: bool,
}

impl PickerState {
    /// The state of a popup opening on `color`.
    fn new(color: [u8; 3]) -> PickerState {
        PickerState {
            opened_on: color,
            hsva: hsva_of(color),
            hex: format_hex(color),
            preview: None,
            dragging: false,
            open: false,
        }
    }
}

/// The square's width; the draft's `PK_W` (`docs/prototypes/save_editor.html`).
const POPUP_W: f32 = 168.0;
/// The square's height (the draft's `PK_SV`).
const SQUARE_H: f32 = 120.0;
/// The hue strip's height (the draft's `PK_HUE`).
const STRIP_H: f32 = 14.0;

/// The saturation-value square (egui's `color_slider_2d`, drawn the same way: a
/// vertex-colored mesh): `x` is saturation, `y` is value, `interact_pointer_pos` edits
/// `hsva` while a press or a drag is held.
fn sv_square(ui: &mut Ui, hsva: &mut Hsva) -> Response {
    let (rect, response) = ui.allocate_at_least(vec2(POPUP_W, SQUARE_H), Sense::click_and_drag());
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::Slider, ui.is_enabled(), "saturation and value")
    });
    if let Some(pos) = response.interact_pointer_pos() {
        hsva.s = remap_clamp(pos.x, rect.left()..=rect.right(), 0.0..=1.0);
        hsva.v = remap_clamp(pos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }
    if ui.is_rect_visible(rect) {
        // Each RGB channel is bilinear in (s, v) at a fixed hue, so a coarse grid already
        // renders the gradient exactly (egui's own square samples it densely because its
        // gamma-space variant is not).
        const SUBDIVISIONS: u32 = 8;
        let color_at = |s: f32, v: f32| -> Color32 {
            Hsva {
                h: hsva.h,
                s,
                v,
                a: 1.0,
            }
            .into()
        };
        let mut mesh = Mesh::default();
        for xi in 0..=SUBDIVISIONS {
            for yi in 0..=SUBDIVISIONS {
                let (x, y) = (
                    xi as f32 / SUBDIVISIONS as f32,
                    yi as f32 / SUBDIVISIONS as f32,
                );
                mesh.colored_vertex(
                    pos2(
                        lerp(rect.left()..=rect.right(), x),
                        lerp(rect.bottom()..=rect.top(), y),
                    ),
                    color_at(x, y),
                );
            }
        }
        for xi in 0..SUBDIVISIONS {
            for yi in 0..SUBDIVISIONS {
                let tl = yi * (SUBDIVISIONS + 1) + xi;
                mesh.add_triangle(tl, tl + 1, tl + SUBDIVISIONS + 1);
                mesh.add_triangle(tl + 1, tl + SUBDIVISIONS + 2, tl + SUBDIVISIONS + 1);
            }
        }
        ui.painter().add(Shape::mesh(mesh));
        ui.painter().rect_stroke(
            rect,
            0.0,
            ui.style().interact(&response).bg_stroke,
            StrokeKind::Inside,
        );
        // The ring marks the working color's point, as the draft draws it.
        let center = pos2(
            lerp(rect.left()..=rect.right(), hsva.s),
            lerp(rect.bottom()..=rect.top(), hsva.v),
        );
        ui.painter()
            .circle_stroke(center, 4.0, Stroke::new(3.0, Color32::BLACK));
        ui.painter()
            .circle_stroke(center, 4.0, Stroke::new(1.0, Color32::WHITE));
    }
    response
}

/// The hue strip (egui's `color_slider_1d` for hue): `x` is hue, `interact_pointer_pos`
/// edits `hsva.h` while a press or a drag is held.
fn hue_strip(ui: &mut Ui, hsva: &mut Hsva) -> Response {
    let (rect, response) = ui.allocate_at_least(vec2(POPUP_W, STRIP_H), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, ui.is_enabled(), "hue"));
    if let Some(pos) = response.interact_pointer_pos() {
        hsva.h = remap_clamp(pos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }
    if ui.is_rect_visible(rect) {
        // The hue wheel's six sectors are linear in RGB within each, so seven vertex
        // columns render the strip exactly.
        let mut mesh = Mesh::default();
        for sector in 0..=6_u32 {
            let h = sector as f32 / 6.0;
            let color: Color32 = Hsva {
                h,
                s: 1.0,
                v: 1.0,
                a: 1.0,
            }
            .into();
            let x = lerp(rect.left()..=rect.right(), h);
            mesh.colored_vertex(pos2(x, rect.top()), color);
            mesh.colored_vertex(pos2(x, rect.bottom()), color);
        }
        for sector in 0..6_u32 {
            let left = 2 * sector;
            mesh.add_triangle(left, left + 1, left + 2);
            mesh.add_triangle(left + 1, left + 3, left + 2);
        }
        ui.painter().add(Shape::mesh(mesh));
        ui.painter().rect_stroke(
            rect,
            0.0,
            ui.style().interact(&response).bg_stroke,
            StrokeKind::Inside,
        );
        // The draft's 3px white bar with a black edge marks the hue's x.
        let x = lerp(rect.left()..=rect.right(), hsva.h);
        let marker =
            Rect::from_center_size(pos2(x, rect.center().y), vec2(3.0, rect.height() + 2.0));
        ui.painter().rect_filled(marker, 0.0, Color32::WHITE);
        ui.painter()
            .rect_stroke(marker, 0.0, (1.0, Color32::BLACK), StrokeKind::Inside);
    }
    response
}

/// One swatch in a suggestion or recents row: a small square of `rgb` that previews on
/// hover and commits on click; its label is its hex.
fn color_swatch(ui: &mut Ui, rgb: [u8; 3]) -> Response {
    let size = vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_at_least(size, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::ColorButton, ui.is_enabled(), format_hex(rgb))
    });
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let rect = rect.expand(visuals.expansion);
        ui.painter().rect_filled(
            rect.shrink(1.0),
            visuals.corner_radius.at_most(2),
            color32(rgb),
        );
        ui.painter().rect_stroke(
            rect,
            visuals.corner_radius.at_most(2),
            (1.0, visuals.bg_fill),
            StrokeKind::Inside,
        );
    }
    response
}

/// A finished gesture lands: `color` moves once (not at all when the gesture ended on the
/// color it started from), and the popup's working state follows.
fn commit_color(state: &mut PickerState, color: &mut [u8; 3], changed: &mut bool, c: [u8; 3]) {
    if c != *color {
        *color = c;
        *changed = true;
    }
    state.hsva = hsva_of(c);
}

/// One row of swatches (suggestions or recents): each previews into `hovered` while the
/// pointer is on it and commits its color on click.
fn swatch_row(
    ui: &mut Ui,
    swatches: &[[u8; 3]],
    hovered: &mut Option<[u8; 3]>,
    state: &mut PickerState,
    color: &mut [u8; 3],
    changed: &mut bool,
) {
    for &swatch in swatches {
        let response = color_swatch(ui, swatch).on_hover_text(format_hex(swatch));
        if response.hovered() {
            *hovered = Some(swatch);
        }
        if response.clicked() {
            commit_color(state, color, changed, swatch);
        }
    }
}

/// The popup's contents, top to bottom as the draft lays them out: the square and the
/// strip, the hex field, the R/G/B fields, the host's suggestions, the session's recents.
/// The gesture rule (`plans/libs/color_tools.md` "Color-picker widget (Phase 8)"): a press
/// on the square, the strip or an R/G/B field previews while held and commits once on
/// release; a typed value, a committed hex text and a swatch click are one change each.
fn popup_body(
    ui: &mut Ui,
    state: &mut PickerState,
    color: &mut [u8; 3],
    changed: &mut bool,
    suggestions: &[[u8; 3]],
) {
    let sv = sv_square(ui, &mut state.hsva);
    let hue = hue_strip(ui, &mut state.hsva);
    let mut down = sv.is_pointer_button_down_on() || hue.is_pointer_button_down_on();

    // The hex field shows the working color while it is not being typed in (a drag
    // mirrors into it); on Enter or focus loss its text commits, and a text that does
    // not read as `#RRGGBB` is put back. Escape cancels the edit instead (the popup is
    // closing anyway, as in the draft where the field is already gone on blur).
    let hex_id = ui.make_persistent_id("hex");
    if !ui.memory(|memory| memory.has_focus(hex_id)) {
        state.hex = format_hex(rgb_of(state.hsva));
    }
    let hex_response = ui.add(
        TextEdit::singleline(&mut state.hex)
            .id(hex_id)
            .font(TextStyle::Monospace)
            .desired_width(f32::INFINITY),
    );
    if hex_response.lost_focus() && !ui.input(|input| input.key_pressed(Key::Escape)) {
        if let Some(parsed) = parse_hex(&state.hex) {
            commit_color(state, color, changed, parsed);
        }
        state.hex = format_hex(rgb_of(state.hsva));
    }

    // R/G/B fields: a typed value commits once (`update_while_editing(false)` keeps the
    // keystrokes inside the field until Enter or focus loss); a drag only previews until
    // its release lands in the gesture below.
    let mut channels = rgb_of(state.hsva);
    let mut channel_changed = false;
    ui.horizontal(|ui| {
        for (index, prefix) in ["R ", "G ", "B "].iter().enumerate() {
            let mut channel = channels[index];
            let response = ui.add(
                DragValue::new(&mut channel)
                    .speed(0.5)
                    .update_while_editing(false)
                    .prefix(*prefix),
            );
            channels[index] = channel;
            channel_changed |= response.changed();
            down |= response.is_pointer_button_down_on();
        }
    });
    if channel_changed {
        state.hsva = hsva_of(channels);
        if !down {
            commit_color(state, color, changed, channels);
        }
    }

    let mut hovered = None;
    if !suggestions.is_empty() {
        ui.horizontal(|ui| {
            swatch_row(ui, suggestions, &mut hovered, state, color, changed);
        });
    }
    let recents = recent_colors(ui.ctx());
    if !recents.is_empty() {
        ui.horizontal(|ui| {
            swatch_row(ui, &recents, &mut hovered, state, color, changed);
        });
    }

    if down {
        state.preview = Some(rgb_of(state.hsva));
    } else {
        state.preview = hovered;
        if state.dragging {
            // The release ends the gesture: one commit for the whole drag, or none when
            // it ended back on the color it started from (`commit_color` checks).
            commit_color(state, color, changed, rgb_of(state.hsva));
        }
    }
    state.dragging = down;
}

#[cfg(test)]
mod tests {
    use egui::accesskit::{self, Role};

    use super::*;

    /// A headless window: one egui context with AccessKit on, run a frame at a time
    /// over an 800 by 600 screen (the `studio_core` log panel's harness, for a bare
    /// `Ui`).
    struct Headless {
        ctx: egui::Context,
    }

    impl Headless {
        fn new() -> Headless {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            Headless { ctx }
        }

        /// Runs one frame with `events` as its input; returns the frame's AccessKit
        /// nodes.
        fn frame(
            &mut self,
            events: Vec<egui::Event>,
            contents: impl FnMut(&mut egui::Ui),
        ) -> Vec<accesskit::Node> {
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
            // Nothing renders here, so the frame's texture deltas are dropped on
            // purpose.
            output.drop_without_applying_deltas();
            update.nodes.into_iter().map(|(_, node)| node).collect()
        }

        /// One frame of a single field over `color` with `suggestions`, returning the
        /// nodes and the widget's output.
        fn field(
            &mut self,
            color: &mut [u8; 3],
            suggestions: &[[u8; 3]],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, ColorFieldOutput) {
            let mut output = None;
            let nodes = self.frame(events, |ui| {
                output = Some(color_field(ui, "field", color, suggestions));
            });
            (nodes, output.expect("the field ran"))
        }

        /// A click at `pos`: the press frame, then the release frame whose nodes and
        /// output are returned (`clicked()` lands on the release).
        fn click_at(
            &mut self,
            pos: egui::Pos2,
            color: &mut [u8; 3],
            suggestions: &[[u8; 3]],
        ) -> (Vec<accesskit::Node>, ColorFieldOutput) {
            self.field(color, suggestions, pointer_at(pos, true));
            self.field(color, suggestions, pointer_at(pos, false))
        }

        /// One frame over two fields, returning the nodes and each field's output.
        fn two_fields(
            &mut self,
            a: &mut [u8; 3],
            b: &mut [u8; 3],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, ColorFieldOutput, ColorFieldOutput) {
            let mut out_a = None;
            let mut out_b = None;
            let nodes = self.frame(events, |ui| {
                out_a = Some(color_field(ui, "a", a, &[]));
                out_b = Some(color_field(ui, "b", b, &[]));
            });
            (
                nodes,
                out_a.expect("field a ran"),
                out_b.expect("field b ran"),
            )
        }

        /// A click at `pos` over the two fields; returns the release frame.
        fn click_two(
            &mut self,
            pos: egui::Pos2,
            a: &mut [u8; 3],
            b: &mut [u8; 3],
        ) -> (Vec<accesskit::Node>, ColorFieldOutput, ColorFieldOutput) {
            self.two_fields(a, b, pointer_at(pos, true));
            self.two_fields(a, b, pointer_at(pos, false))
        }
    }

    /// The node carrying `label` (a `WidgetInfo::labeled` widget's label).
    fn entry<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.label() == Some(label))
            .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
    }

    /// Every drawn swatch label: suggestion and recents swatches are labeled by their
    /// `#RRGGBB` hex.
    fn swatch_labels(nodes: &[accesskit::Node]) -> Vec<String> {
        nodes
            .iter()
            .filter_map(|node| node.label())
            .filter(|label| label.starts_with('#'))
            .map(str::to_owned)
            .collect()
    }

    /// The `index`th field button (labeled `color field …`, role `ColorWell`), ordered
    /// top to bottom: `nodes` come back in `IdMap` order, not layout order.
    fn field_button(nodes: &[accesskit::Node], index: usize) -> &accesskit::Node {
        let mut buttons: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| {
                node.label()
                    .is_some_and(|label| label.starts_with("color field"))
            })
            .collect();
        buttons.sort_by(|a, b| {
            let (ay, by) = (a.bounds().map(|r| r.y0), b.bounds().map(|r| r.y0));
            ay.partial_cmp(&by).expect("drawn bounds")
        });
        buttons.get(index).expect("not enough color fields drawn")
    }

    /// The hex field's node: the popup's one text input.
    fn hex_field(nodes: &[accesskit::Node]) -> &accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == Role::TextInput)
            .expect("no hex field drawn")
    }

    /// The centre of `node`'s bounds, where a click lands.
    fn centre(node: &accesskit::Node) -> egui::Pos2 {
        let bounds = node.bounds().expect("a widget has bounds");
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    /// The primary button pressed (or released) at `pos`, the pointer moved there first.
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

    /// `key` pressed and released.
    fn key(key: egui::Key) -> Vec<egui::Event> {
        let event = |pressed: bool| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        vec![event(true), event(false)]
    }

    /// Replaces the focused text field's contents with `text`: End moves the caret past
    /// the text, the backspaces clear it, then the characters land.
    fn type_text(text: &str) -> Vec<egui::Event> {
        let mut events = key(egui::Key::End);
        for _ in 0..16 {
            events.push(egui::Event::Key {
                key: egui::Key::Backspace,
                physical_key: Some(egui::Key::Backspace),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        events.push(egui::Event::Text(text.to_owned()));
        events
    }

    #[test]
    fn parse_hex_reads_rrggbb_permissively() {
        for text in ["#ff8000", "FF8000", " #Ff8000 "] {
            assert_eq!(
                parse_hex(text),
                Some([255, 128, 0]),
                "{text:?} should parse"
            );
        }
        for text in ["#zz0000", "#fff", "ff80001"] {
            assert_eq!(parse_hex(text), None, "{text:?} should not parse");
        }
    }

    #[test]
    fn format_hex_is_upper_case_rrggbb() {
        assert_eq!(format_hex([255, 130, 0]), "#FF8200");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a committed hex text is
    // one change.
    #[test]
    fn hex_text_committed_with_enter_is_one_change() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        // The popup's first frame is a sizing pass: its layer is hit-testable only from
        // the next frame on.
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(hex_field(&nodes)), &mut color, &[]);

        let mut events = type_text("#ff8000");
        events.extend(key(egui::Key::Enter));
        let (_, out) = window.field(&mut color, &[], events);
        assert!(out.changed, "the Enter commit is one change");
        assert_eq!(color, [255, 128, 0], "the committed color");
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "changed fires on the commit frame only");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the `#` is optional and
    // the field commits on focus loss.
    #[test]
    fn hex_text_without_hash_commits_on_focus_loss() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
        window.field(&mut color, &[], type_text("ff3300"));

        let (_, out) = window.field(&mut color, &[], key(egui::Key::Tab));
        assert!(out.changed, "moving focus away commits the typed text");
        assert_eq!(color, [255, 51, 0], "the committed color");
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "changed fires on the commit frame only");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a text that does not
    // read as `#RRGGBB` is put back when the field loses focus.
    #[test]
    fn invalid_hex_text_is_put_back_on_focus_loss() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
        window.field(&mut color, &[], type_text("#zz0000"));

        let (_, out) = window.field(&mut color, &[], key(egui::Key::Tab));
        assert!(!out.changed, "an invalid text commits nothing");
        assert_eq!(color, [255, 128, 0], "the color is untouched");
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert_eq!(
            hex_field(&nodes).value(),
            Some("#FF8000"),
            "the field is put back to the color's hex"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": closing on another color
    // joins the session's recent colors, shared by every picker; a swatch click is one
    // change.
    #[test]
    fn a_second_field_shares_the_firsts_recent_colors() {
        let mut window = Headless::new();
        let mut color_a = [0, 0, 0];
        let mut color_b = [64, 64, 64];

        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(field_button(&nodes, 0)), &mut color_a, &mut color_b);
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(hex_field(&nodes)), &mut color_a, &mut color_b);
        let mut events = type_text("#112233");
        events.extend(key(egui::Key::Enter));
        let (_, out_a, _) = window.two_fields(&mut color_a, &mut color_b, events);
        assert!(
            out_a.changed && color_a == [17, 34, 51],
            "field a committed"
        );

        window.two_fields(&mut color_a, &mut color_b, key(egui::Key::Escape));
        // A's popup still drew on the frame it closed in; the next frame's hit test
        // uses it, so B's button is clickable only from the frame after.
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());

        let (_, _, _) =
            window.click_two(centre(field_button(&nodes, 1)), &mut color_a, &mut color_b);
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        assert!(
            swatch_labels(&nodes).contains(&"#112233".to_owned()),
            "the second field's recents hold the first's color"
        );

        let (_, _, out_b) =
            window.click_two(centre(entry(&nodes, "#112233")), &mut color_a, &mut color_b);
        assert!(out_b.changed, "a swatch click is one change");
        assert_eq!(color_b, [17, 34, 51], "the swatch's color committed");
        let (_, _, out_b) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        assert!(!out_b.changed, "changed fires on the click frame only");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a hovered swatch is a
    // hover candidate for the host's live preview, not a change.
    #[test]
    fn hovering_a_swatch_previews_it_without_committing() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        // Two commits leave two recents; hovering the older one previews a color that is
        // not the current one.
        for hex in ["#112233", "#445566"] {
            let (nodes, _) = window.field(&mut color, &[], Vec::new());
            window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
            let (nodes, _) = window.field(&mut color, &[], Vec::new());
            window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
            let mut events = type_text(hex);
            events.extend(key(egui::Key::Enter));
            window.field(&mut color, &[], events);
            window.field(&mut color, &[], key(egui::Key::Escape));
        }
        assert_eq!(color, [68, 85, 102], "the second commit landed");

        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let swatch = entry(&nodes, "#112233");
        let (_, out) = window.field(
            &mut color,
            &[],
            vec![egui::Event::PointerMoved(centre(swatch))],
        );
        assert_eq!(
            out.candidate,
            Some([17, 34, 51]),
            "a hovered swatch is the live candidate"
        );
        assert!(!out.changed, "a hover commits nothing");
        assert_eq!(color, [68, 85, 102], "the color is untouched by a hover");

        let (_, out) = window.field(
            &mut color,
            &[],
            vec![egui::Event::PointerMoved(egui::pos2(400.0, 400.0))],
        );
        assert_eq!(out.candidate, None, "moving off clears the candidate");
        assert!(!out.changed && color == [68, 85, 102]);
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a drag in the square is
    // a candidate while it lasts and one change when released.
    #[test]
    fn a_drag_on_the_square_is_one_change() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        // The popup's first frame is a sizing pass: its layer is hit-testable only from
        // the next frame on, and its position is final then too.
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let square = entry(&nodes, "saturation and value");
        let rect = square.bounds().expect("the square is drawn");
        let point = |x: f64, y: f64| egui::pos2((rect.x0 + x) as f32, (rect.y0 + y) as f32);

        let mut last_candidate = None;
        for pos in [point(30.0, 30.0), point(60.0, 50.0), point(90.0, 70.0)] {
            let pressed = last_candidate.is_none();
            let (_, out) = window.field(
                &mut color,
                &[],
                if pressed {
                    pointer_at(pos, true)
                } else {
                    vec![egui::Event::PointerMoved(pos)]
                },
            );
            assert!(
                out.candidate.is_some(),
                "the dragged color is the candidate while held"
            );
            assert!(!out.changed, "no commit while the drag runs");
            assert_eq!(color, [255, 128, 0], "the color waits for the release");
            last_candidate = out.candidate;
        }

        let (_, out) = window.field(&mut color, &[], pointer_at(point(90.0, 70.0), false));
        assert!(out.changed, "the release commits once");
        assert_eq!(
            color,
            last_candidate.expect("the drag previewed"),
            "the committed color is the last candidate"
        );
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "no repeat on the idle frame");
        assert_eq!(out.candidate, None);
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the recent colors row
    // keeps at most eight.
    #[test]
    fn nine_commits_leave_eight_recent_swatches() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        for i in 0..9 {
            let (nodes, _) = window.field(&mut color, &[], Vec::new());
            window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
            let (nodes, _) = window.field(&mut color, &[], Vec::new());
            window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
            let mut events = type_text(&format!("#{:02x}0000", 0x20 + i * 0x10));
            events.extend(key(egui::Key::Enter));
            window.field(&mut color, &[], events);
            window.field(&mut color, &[], key(egui::Key::Escape));
        }

        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert_eq!(
            swatch_labels(&nodes).len(),
            8,
            "the recents row holds the eight newest"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": Escape or a click
    // outside closes the popup.
    #[test]
    fn escape_and_an_outside_click_each_close_the_popup() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let (nodes, _) = window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        assert!(
            nodes.iter().any(|n| n.role() == Role::TextInput),
            "the popup opened"
        );

        // The popup still draws on the frame it closes in (the close is decided after
        // its contents run); the next frame shows it gone.
        window.field(&mut color, &[], key(egui::Key::Escape));
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert!(
            !nodes.iter().any(|n| n.role() == Role::TextInput),
            "Escape closed the popup"
        );

        let (nodes, _) = window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        assert!(
            nodes.iter().any(|n| n.role() == Role::TextInput),
            "the button reopened the popup"
        );
        window.click_at(egui::pos2(760.0, 560.0), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert!(
            !nodes.iter().any(|n| n.role() == Role::TextInput),
            "a click outside closed the popup"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the host's suggestion
    // swatches, shown only when it passes some.
    #[test]
    fn the_suggestion_row_shows_only_when_the_host_passes_some() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let (nodes, _) = window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        assert!(
            swatch_labels(&nodes).is_empty(),
            "no suggestion row without host swatches"
        );
        window.field(&mut color, &[], key(egui::Key::Escape));

        let suggestions = [[200, 16, 46], [255, 255, 255], [29, 66, 138]];
        window.field(&mut color, &suggestions, Vec::new());
        let (nodes, _) = window.click_at(centre(field_button(&nodes, 0)), &mut color, &suggestions);
        assert_eq!(
            swatch_labels(&nodes).len(),
            3,
            "three suggestions draw three swatches"
        );
    }

    #[test]
    fn recents_keep_the_eight_newest_without_duplicates() {
        let mut recents = Vec::new();
        for n in 0..9 {
            push_recent_color(&mut recents, [n, 0, 0]);
        }
        assert_eq!(
            recents.len(),
            MAX_RECENTS,
            "the ninth push drops the oldest"
        );
        assert_eq!(recents[0], [8, 0, 0], "newest first");
        assert!(recents.contains(&[1, 0, 0]) && !recents.contains(&[0, 0, 0]));

        push_recent_color(&mut recents, [3, 0, 0]);
        assert_eq!(recents[0], [3, 0, 0], "a repeat moves to the front");
        assert_eq!(
            recents.iter().filter(|&&c| c == [3, 0, 0]).count(),
            1,
            "no duplicate"
        );
    }
}
