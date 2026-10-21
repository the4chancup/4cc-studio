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
    let mut changed = false;
    let mut state = ui
        .ctx()
        .data(|data| data.get_temp::<PickerState>(state_id))
        .unwrap_or_else(|| PickerState::new(*color));
    if !was_open {
        // egui keeps a single popup per viewport: a sibling field's popup opening, or a
        // frame where this field was not drawn, closed this one between frames, and the
        // close rules apply as if it had closed here.
        if state.open {
            commit_pending_text(&mut state, color, &mut changed);
            if *color != state.opened_on {
                push_recent(ui.ctx(), *color);
            }
        }
        state = PickerState::new(*color);
    }
    if *color != state.shown {
        // The host moved the color under the popup. When the last drawn frame committed,
        // this is the host rounding that pick back (the Save editor's 6 bits): feedback,
        // so `opened_on` stays and the pick still joins the recents on close. Otherwise
        // it is a host edit (an undo, a palette apply): the working state reseeds and
        // the change is not a pick, so `opened_on` moves and it cannot join the recents.
        // The hex text follows `hsva` in the body unless the member is typing in it.
        state.hsva = hsva_of(*color);
        if !state.committed {
            state.opened_on = *color;
        }
    }

    let mut response = color_button(ui, state.preview.unwrap_or(*color), was_open);
    Popup::from_toggle_button_response(&response)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .layout(Layout::top_down_justified(Align::Min))
        .show(|ui| popup_body(ui, &mut state, color, &mut changed, suggestions));

    let is_open = Popup::is_id_open(ui.ctx(), popup_id);
    if was_open && !is_open {
        commit_pending_text(&mut state, color, &mut changed);
        if *color != state.opened_on {
            push_recent(ui.ctx(), *color);
        }
    }
    if changed {
        response.mark_changed();
    }
    state.open = is_open;
    state.shown = *color;
    state.committed = changed;
    let candidate = state.preview;
    ui.ctx().data_mut(|data| data.insert_temp(state_id, state));
    ColorFieldOutput {
        response,
        changed,
        candidate,
    }
}

/// A typed text still in the hex field or a channel field when the popup closes without
/// the field losing focus first (a toggle through the field's own button, or a close
/// that lands between frames): any close but Escape commits it, like focus loss would;
/// the channel's typed value is already in the working color.
fn commit_pending_text(state: &mut PickerState, color: &mut [u8; 3], changed: &mut bool) {
    if state.hex_focused
        && let Some(parsed) = parse_hex(&state.hex)
    {
        commit_color(state, color, changed, parsed);
    }
    if state.rgb_focused {
        commit_color(state, color, changed, rgb_of(state.hsva));
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
            color32(rgb),
        );
        ui.painter().rect_stroke(
            rect,
            visuals.corner_radius.at_most(2),
            visuals.bg_stroke,
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

/// `[u8; 3]` sRGB as the working hsv: the formula is the same in every color space, h,
/// s and v are parameters of it, not colors. `hsv_from_rgb` on the sRGB fractions gives
/// the triple the fields and the gradients all share.
fn hsva_of(rgb: [u8; 3]) -> Hsva {
    let (h, s, v) = egui::ecolor::hsv_from_rgb([
        rgb[0] as f32 / 255.0,
        rgb[1] as f32 / 255.0,
        rgb[2] as f32 / 255.0,
    ]);
    Hsva { h, s, v, a: 1.0 }
}

/// The working hsv as `[u8; 3]` sRGB, in the plan's "HSV over sRGB values" (the draft's
/// `hsvToRgb`): the formula's output used as the sRGB value, not re-encoded, so a pick
/// equals the color the gradients show, which a renderer interpolates as sRGB.
fn rgb_of(hsva: Hsva) -> [u8; 3] {
    let [r, g, b] = egui::ecolor::rgb_from_hsv((hsva.h, hsva.s, hsva.v));
    [channel(r), channel(g), channel(b)]
}

/// `fraction` of 1.0 as an sRGB byte.
fn channel(fraction: f32) -> u8 {
    (fraction.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// The working hsv as `Color32` for vertex colors and swatch fills.
fn srgb_of(hsva: Hsva) -> Color32 {
    let [r, g, b] = rgb_of(hsva);
    Color32::from_rgb(r, g, b)
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
    /// The hex field had keyboard focus the last frame the popup's body drew: a typed
    /// text then commits when the popup closes another way (the field's own button, a
    /// sibling popup), as losing focus would have committed it.
    hex_focused: bool,
    /// An R, G or B field had keyboard focus the last frame the popup's body drew: its
    /// typed value commits on the same closes as the hex text.
    rgb_focused: bool,
    /// The color the field showed at the end of the last frame drawn: anything different
    /// now is the host moving the color (an undo, a palette apply), not a pick.
    shown: [u8; 3],
    /// Whether the last frame drawn committed a pick: the host's rounding of that pick,
    /// arriving as a color change on the next frame, is feedback and not an undo, so it
    /// must not move `opened_on`.
    committed: bool,
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
            hex_focused: false,
            rgb_focused: false,
            shown: color,
            committed: false,
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
        // In sRGB terms each channel is v·(a + b·s), bilinear in (s, v), so a small grid
        // already matches the gradient a click picks; egui's own square samples its
        // gamma-space variant more densely, which is not bilinear.
        const SUBDIVISIONS: u32 = 8;
        let color_at = |s: f32, v: f32| -> Color32 {
            srgb_of(Hsva {
                h: hsva.h,
                s,
                v,
                a: 1.0,
            })
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
            let color: Color32 = srgb_of(Hsva {
                h,
                s: 1.0,
                v: 1.0,
                a: 1.0,
            });
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
            visuals.bg_stroke,
            StrokeKind::Inside,
        );
    }
    response
}

/// A finished gesture lands: `color` moves once (not at all when the gesture ended on the
/// color it started from), and the working hsv is rebuilt from the committed RGB, the
/// right reset for a typed text, a swatch or a host edit.
fn commit_color(state: &mut PickerState, color: &mut [u8; 3], changed: &mut bool, c: [u8; 3]) {
    if c != *color {
        *color = c;
        *changed = true;
    }
    state.hsva = hsva_of(c);
}

/// A gesture release on the square, the strip or an R/G/B field lands: one commit for the
/// whole gesture. The working hsv is *kept* rather than rebuilt from the committed RGB:
/// white, gray and black hold no hue, so rebuilding would drop a hue the member just
/// chose on the strip before moving into the square.
fn commit_working(state: &mut PickerState, color: &mut [u8; 3], changed: &mut bool) {
    let working = rgb_of(state.hsva);
    if working != *color {
        *color = working;
        *changed = true;
    }
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
    state.hex_focused = ui.memory(|memory| memory.has_focus(hex_id));

    // R/G/B fields: typing updates the working color live (`update_while_editing`'s
    // default, the draft's live preview) and commits on Enter or focus loss (or through
    // `commit_pending_text` when the popup closes first); a drag only previews until its
    // release lands in the gesture below.
    let mut channels = rgb_of(state.hsva);
    let mut channel_changed = false;
    let mut channel_focused = false;
    let mut channel_lost_focus = false;
    ui.horizontal(|ui| {
        for (index, prefix) in ["R ", "G ", "B "].iter().enumerate() {
            let mut channel = channels[index];
            let response = ui.add(DragValue::new(&mut channel).speed(0.5).prefix(*prefix));
            channels[index] = channel;
            channel_changed |= response.changed();
            channel_focused |= response.has_focus();
            channel_lost_focus |= response.lost_focus();
            down |= response.is_pointer_button_down_on();
        }
    });
    state.rgb_focused = channel_focused;
    if channel_changed {
        state.hsva = hsva_of(channels);
    }
    if channel_lost_focus && !ui.input(|input| input.key_pressed(Key::Escape)) {
        commit_color(state, color, changed, channels);
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

    // A press still held previews; a release, a one-frame click and a drag's stop all
    // commit the working color once (`commit_working` keeps the working hsv, so a hue
    // chosen on a hueless color survives).
    if down {
        state.preview = Some(rgb_of(state.hsva));
    } else {
        state.preview = hovered;
        if state.dragging
            || sv.clicked()
            || hue.clicked()
            || sv.drag_stopped()
            || hue.drag_stopped()
        {
            commit_working(state, color, changed);
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
        /// The last frame's paint shapes (`FullOutput::shapes`), kept for assertions.
        shapes: Vec<egui::epaint::ClippedShape>,
    }

    impl Headless {
        fn new() -> Headless {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            Headless {
                ctx,
                shapes: Vec::new(),
            }
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
            self.shapes = std::mem::take(&mut output.shapes);
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

        /// One frame over two fields, returning the nodes and each field's output. The
        /// spacer keeps `b`'s button below `a`'s popup, so it stays clickable while `a`'s
        /// is open.
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
                ui.add_space(400.0);
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

    /// The hex field's node: the popup's text input showing a `#RRGGBB` value.
    fn hex_field(nodes: &[accesskit::Node]) -> &accesskit::Node {
        nodes
            .iter()
            .find(|node| {
                node.role() == Role::TextInput
                    && node.value().is_some_and(|value| value.starts_with('#'))
            })
            .expect("no hex field drawn")
    }

    /// The R, G or B field's node: a spin button whose value text carries its prefix
    /// (`"R 128"`-shaped).
    fn spin_field<'a>(nodes: &'a [accesskit::Node], prefix: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| {
                node.role() == Role::SpinButton
                    && node.value().is_some_and(|value| value.starts_with(prefix))
            })
            .unwrap_or_else(|| panic!("no field with prefix {prefix:?}"))
    }

    /// `node`'s AccessKit bounds as an egui `Rect`.
    fn egui_rect(node: &accesskit::Node) -> egui::Rect {
        let bounds = node.bounds().expect("a drawn node has bounds");
        egui::Rect::from_min_max(
            egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
            egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
        )
    }

    /// The frame's mesh covering `rect` (within `tolerance` points of error) that also
    /// satisfies `has` (bounds alone cannot tell a fill mesh from its outline stroke).
    fn mesh_at(
        shapes: &[egui::epaint::ClippedShape],
        rect: egui::Rect,
        tolerance: f32,
        has: impl Fn(&egui::Mesh) -> bool,
    ) -> &egui::Mesh {
        shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Mesh(mesh)
                    if has(mesh)
                        && (mesh.calc_bounds().min - rect.min).length() < tolerance
                        && (mesh.calc_bounds().max - rect.max).length() < tolerance =>
                {
                    Some(mesh.as_ref())
                }
                _ => None,
            })
            .expect("no mesh drawn at the expected bounds")
    }

    /// The color of `mesh`'s vertex nearest to `pos`.
    fn color_at(mesh: &egui::Mesh, pos: egui::Pos2) -> Color32 {
        mesh.vertices
            .iter()
            .min_by(|a, b| {
                a.pos
                    .distance_sq(pos)
                    .partial_cmp(&b.pos.distance_sq(pos))
                    .expect("no NaN positions")
            })
            .expect("a mesh has vertices")
            .color
    }

    /// `a` lerped to `b` by `t`, per channel, as the renderer blends vertex colors.
    fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
        let channel = |a: u8, b: u8| -> u8 { (a as f32 + (b as f32 - a as f32) * t).round() as u8 };
        Color32::from_rgba_unmultiplied(
            channel(a.r(), b.r()),
            channel(a.g(), b.g()),
            channel(a.b(), b.b()),
            channel(a.a(), b.a()),
        )
    }

    /// The color the strip's mesh shows at `x`: its vertex columns (one per sextant
    /// boundary) lerped between the two bracketing `x`.
    fn interpolated_at_x(mesh: &egui::Mesh, x: f32) -> Color32 {
        let mut columns: Vec<(f32, Color32)> = mesh
            .vertices
            .iter()
            .map(|vertex| (vertex.pos.x, vertex.color))
            .collect();
        columns.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("ordered xs"));
        columns.dedup_by_key(|column| column.0.to_bits());
        let right = columns
            .iter()
            .position(|&(col_x, _)| col_x > x)
            .unwrap_or(columns.len() - 1);
        let (x0, c0) = columns[right - 1];
        let (x1, c1) = columns[right];
        lerp_color(c0, c1, (x - x0) / (x1 - x0))
    }

    /// The color the square's mesh shows at `pos`: the four vertex colors of the grid
    /// cell holding `pos`, bilinearly interpolated.
    fn interpolated_in_cell(mesh: &egui::Mesh, pos: egui::Pos2) -> Color32 {
        let mut xs: Vec<f32> = mesh.vertices.iter().map(|v| v.pos.x).collect();
        let mut ys: Vec<f32> = mesh.vertices.iter().map(|v| v.pos.y).collect();
        xs.sort_by(|a, b| a.partial_cmp(b).expect("ordered xs"));
        xs.dedup_by_key(|x| x.to_bits());
        ys.sort_by(|a, b| a.partial_cmp(b).expect("ordered ys"));
        ys.dedup_by_key(|y| y.to_bits());
        let ix = xs
            .iter()
            .position(|&x| x > pos.x)
            .unwrap_or(xs.len() - 1)
            .max(1);
        let iy = ys
            .iter()
            .position(|&y| y > pos.y)
            .unwrap_or(ys.len() - 1)
            .max(1);
        let (x0, x1) = (xs[ix - 1], xs[ix]);
        let (y0, y1) = (ys[iy - 1], ys[iy]);
        let color_of = |x: f32, y: f32| color_at(mesh, egui::pos2(x, y));
        let t = (pos.x - x0) / (x1 - x0);
        lerp_color(
            lerp_color(color_of(x0, y0), color_of(x1, y0), t),
            lerp_color(color_of(x0, y1), color_of(x1, y1), t),
            (pos.y - y0) / (y1 - y0),
        )
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

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a parseable text commits
    // whenever the field loses focus, the popup closing included; the field's own
    // button closing it is not an exception.
    #[test]
    fn a_typed_hex_commits_when_the_button_closes_the_popup() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
        window.field(&mut color, &[], type_text("#112233"));

        let (_, out) = window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        assert!(out.changed, "the typed text commits on the button close");
        assert_eq!(color, [17, 34, 51], "the typed color committed");
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "the commit fires once");

        // The close also joins the recents, since the color is not the one it opened on.
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert!(
            swatch_labels(&nodes).contains(&"#112233".to_owned()),
            "the close joined the recents"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a committed R/G/B value
    // is one change; the keystrokes before it change nothing.
    #[test]
    fn a_typed_rgb_value_commits_on_enter() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(spin_field(&nodes, "R ")), &mut color, &[]);

        // The keystrokes themselves are not commits: one each frame, untouched.
        for text in ["1", "0"] {
            let (_, out) = window.field(&mut color, &[], vec![egui::Event::Text(text.to_owned())]);
            assert!(!out.changed, "a keystroke is not a commit");
            assert_eq!(color, [0, 0, 0], "the color waits for the Enter");
        }
        let (_, out) = window.field(&mut color, &[], key(egui::Key::Enter));
        assert!(out.changed, "the Enter commits once");
        assert_eq!(color, [10, 0, 0], "the typed channel committed");
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "changed fires on the commit frame only");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a drag on an R/G/B
    // field is a candidate while it lasts and one change when released.
    #[test]
    fn a_drag_on_an_rgb_field_is_one_change() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let green = centre(spin_field(&nodes, "G "));

        let mut last_candidate = None;
        for (index, dx) in [10.0, 20.0, 30.0].iter().enumerate() {
            let pos = egui::pos2(green.x + dx, green.y);
            let (_, out) = window.field(
                &mut color,
                &[],
                if index == 0 {
                    pointer_at(pos, true)
                } else {
                    vec![egui::Event::PointerMoved(pos)]
                },
            );
            assert!(out.candidate.is_some(), "the dragged value previews");
            assert!(!out.changed, "no commit while the drag runs");
            assert_eq!(color, [255, 128, 0], "the color waits for the release");
            last_candidate = out.candidate;
        }

        let (_, out) = window.field(
            &mut color,
            &[],
            pointer_at(egui::pos2(green.x + 30.0, green.y), false),
        );
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

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": any close but Escape
    // commits a parseable typed text, hex or R/G/B; the field's own button closing it
    // is not an exception.
    #[test]
    fn a_typed_channel_commits_when_the_button_closes_the_popup() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(spin_field(&nodes, "R ")), &mut color, &[]);
        window.field(&mut color, &[], type_text("10"));

        let (_, out) = window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        assert!(out.changed, "the typed value commits on the button close");
        assert_eq!(color, [10, 0, 0], "the typed channel committed");
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "the commit fires once");
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

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a color joins the
    // recents only when the popup closes on a color other than the one it opened on.
    #[test]
    fn closing_unchanged_adds_no_recent() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];

        // Open and close with no pick is not a pick, by Escape.
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        window.field(&mut color, &[], Vec::new());
        window.field(&mut color, &[], key(egui::Key::Escape));
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let labels = swatch_labels(&nodes);
        assert!(
            labels.is_empty(),
            "an unchanged Escape close adds no recent: {labels:?}"
        );
        window.field(&mut color, &[], key(egui::Key::Escape));

        // Nor is a close between frames, when the field is not drawn for a frame (the
        // host switched tabs) and egui reaps the abandoned popup.
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        window.field(&mut color, &[], Vec::new());
        window.frame(Vec::new(), |_| {});
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let labels = swatch_labels(&nodes);
        assert!(
            labels.is_empty(),
            "an unchanged between-frame close adds no recent: {labels:?}"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a color joins the
    // recents when the popup closes, not when the gesture commits.
    #[test]
    fn a_commit_joins_the_recents_only_on_close() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        // Seed one recent so the row is drawn, then commit a second color and check the
        // row while the popup is still open.
        for hex in ["#556677", "#112233"] {
            let (nodes, _) = window.field(&mut color, &[], Vec::new());
            window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
            let (nodes, _) = window.field(&mut color, &[], Vec::new());
            window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
            let mut events = type_text(hex);
            events.extend(key(egui::Key::Enter));
            window.field(&mut color, &[], events);
            if hex == "#112233" {
                let (nodes, _) = window.field(&mut color, &[], Vec::new());
                assert_eq!(
                    swatch_labels(&nodes),
                    vec!["#556677".to_owned()],
                    "the commit is not a recent while the popup is open"
                );
            }
            window.field(&mut color, &[], key(egui::Key::Escape));
        }

        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert_eq!(
            swatch_labels(&nodes),
            vec!["#112233".to_owned(), "#556677".to_owned()],
            "on close the committed color fronts the recents"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the recents are the
    // session's, shared by every picker in the program.
    #[test]
    fn a_sibling_field_close_pushes_the_color_to_the_shared_recents() {
        let mut window = Headless::new();
        let mut color_a = [0, 0, 0];
        let mut color_b = [64, 64, 64];

        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(field_button(&nodes, 0)), &mut color_a, &mut color_b);
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(hex_field(&nodes)), &mut color_a, &mut color_b);
        let mut events = type_text("#112233");
        events.extend(key(egui::Key::Enter));
        window.two_fields(&mut color_a, &mut color_b, events);

        // Clicking field b's button opens its popup: egui keeps one popup per viewport,
        // so a's closes and its changed color joins the shared recents.
        window.click_two(centre(field_button(&nodes, 1)), &mut color_a, &mut color_b);
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        let labels = swatch_labels(&nodes);
        assert!(
            labels.contains(&"#112233".to_owned()),
            "field b's recents hold field a's color: {labels:?}"
        );
        window.two_fields(&mut color_a, &mut color_b, key(egui::Key::Escape));

        // The same rule applies when the popup closes between frames (the field is not
        // drawn for a frame and egui reaps the abandoned popup).
        color_b = [32, 32, 32];
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(field_button(&nodes, 1)), &mut color_a, &mut color_b);
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(hex_field(&nodes)), &mut color_a, &mut color_b);
        let mut events = type_text("#778899");
        events.extend(key(egui::Key::Enter));
        window.two_fields(&mut color_a, &mut color_b, events);
        window.frame(Vec::new(), |ui| {
            color_field(ui, "a", &mut color_a, &[]);
        });
        let (_, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        window.click_two(centre(field_button(&nodes, 1)), &mut color_a, &mut color_b);
        let (nodes, _, _) = window.two_fields(&mut color_a, &mut color_b, Vec::new());
        let labels = swatch_labels(&nodes);
        assert!(
            labels.contains(&"#778899".to_owned()),
            "the between-frame close joined the recents: {labels:?}"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the host owns the
    // color; its own edits (an undo) are not picks, so the popup follows and nothing
    // joins the recents.
    #[test]
    fn a_host_change_reseeds_the_open_popup_and_is_not_a_pick() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        window.field(&mut color, &[], Vec::new());

        // The host moves the color between frames (an undo): the popup reseeds to it.
        color = [50, 60, 70];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert_eq!(
            hex_field(&nodes).value(),
            Some("#323C46"),
            "the hex field shows the host's color"
        );

        // Closing after the host's move is not a pick: nothing joins the recents.
        window.field(&mut color, &[], key(egui::Key::Escape));
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        assert!(
            swatch_labels(&nodes).is_empty(),
            "a host change does not join the recents"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the host's rounding of
    // the widget's own commit (the Save editor's 6 bits), arriving the next frame, is
    // that commit, which still joins the recents on close.
    #[test]
    fn the_hosts_rounding_of_a_commit_still_joins_the_recents() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(hex_field(&nodes)), &mut color, &[]);
        let mut events = type_text("#FF8000");
        events.extend(key(egui::Key::Enter));
        window.field(&mut color, &[], events);
        assert_eq!(color, [255, 128, 0], "the pick committed");

        // The host stores the pick rounded to 6 bits a channel and shows it widened
        // back: 0x80 stores as 32, shown as 130; the widget must read that as the same
        // commit, not as an undo.
        color = [255, 130, 0];
        window.field(&mut color, &[], Vec::new());
        window.field(&mut color, &[], key(egui::Key::Escape));
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let labels = swatch_labels(&nodes);
        assert_eq!(
            labels,
            vec!["#FF8200".to_owned()],
            "the rounded commit joined the recents: {labels:?}"
        );
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

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a drag on the hue strip
    // is a candidate while it lasts and one change when released.
    #[test]
    fn a_drag_on_the_hue_strip_is_one_change() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let strip = entry(&nodes, "hue");
        let rect = strip.bounds().expect("the strip is drawn");
        let point = |x: f64| {
            egui::pos2(
                (rect.x0 + x * (rect.x1 - rect.x0)) as f32,
                ((rect.y0 + rect.y1) / 2.0) as f32,
            )
        };

        let mut last_candidate = None;
        for (index, x) in [0.3, 0.5, 0.8].iter().enumerate() {
            let (_, out) = window.field(
                &mut color,
                &[],
                if index == 0 {
                    pointer_at(point(*x), true)
                } else {
                    vec![egui::Event::PointerMoved(point(*x))]
                },
            );
            assert!(out.candidate.is_some(), "the dragged hue previews");
            assert!(!out.changed, "no commit while the drag runs");
            assert_eq!(color, [255, 128, 0], "the color waits for the release");
            last_candidate = out.candidate;
        }

        let (_, out) = window.field(&mut color, &[], pointer_at(point(0.8), false));
        assert!(out.changed, "the release commits once");
        assert_eq!(
            color,
            last_candidate.expect("the drag previewed"),
            "the committed color is the last candidate"
        );
        assert_ne!(
            hsva_of(color).h,
            hsva_of([255, 128, 0]).h,
            "the committed hue differs from the start's"
        );
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "no repeat on the idle frame");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": a click on the square
    // or the slider is a gesture too, however short.
    #[test]
    fn a_one_frame_click_on_the_square_commits() {
        let mut window = Headless::new();
        let mut color = [0, 0, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let square = entry(&nodes, "saturation and value");
        let rect = square.bounds().expect("the square is drawn");
        let pos = egui::pos2(
            (rect.x0 + rect.x1) as f32 / 2.0,
            (rect.y0 + rect.y1) as f32 / 2.0,
        );

        // A press and its release in one input frame (a touchpad tap) is a click:
        // one change, to the colour under the point.
        let mut events = pointer_at(pos, true);
        events.extend(pointer_at(pos, false));
        let (_, out) = window.field(&mut color, &[], events);
        assert!(out.changed, "the one-frame click commits once");
        assert_ne!(
            color,
            [0, 0, 0],
            "the click point's colour committed: {color:?}"
        );
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert!(!out.changed, "no repeat on the idle frame");
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the working hue
    // survives a gesture on white, gray or black, whose RGB holds no hue.
    #[test]
    fn a_hue_chosen_on_a_hueless_color_survives_for_the_square() {
        let mut window = Headless::new();
        let mut color = [255, 255, 255];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let strip_rect = egui_rect(entry(&nodes, "hue"));
        let square_rect = egui_rect(entry(&nodes, "saturation and value"));

        // Drag the strip to blue's x (hue 2/3) and release: white has no hue, so the
        // release commits nothing, but the marker stays at blue's x.
        let blue_x = strip_rect.left() + strip_rect.width() * (2.0 / 3.0);
        let strip_y = (strip_rect.top() + strip_rect.bottom()) / 2.0;
        let (_, out) = window.field(
            &mut color,
            &[],
            pointer_at(egui::pos2(blue_x, strip_y), true),
        );
        assert!(out.candidate.is_some() && !out.changed);
        let (_, out) = window.field(
            &mut color,
            &[],
            pointer_at(egui::pos2(blue_x, strip_y), false),
        );
        assert!(!out.changed, "white commits nothing on release");
        assert_eq!(color, [255, 255, 255]);
        let (_, out) = window.field(&mut color, &[], Vec::new());
        assert_eq!(out.candidate, None, "the gesture ended cleanly");
        // The marker's fill rect (its stroke sibling's fill is transparent; the Area
        // fade keeps its white premultiplied until the fade is over, so match on
        // non-transparent, not white).
        let marker = window
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape)
                    if shape.fill != Color32::TRANSPARENT
                        && (shape.rect.width() - 3.0).abs() < 0.5
                        && (shape.rect.height() - (STRIP_H + 2.0)).abs() < 0.5 =>
                {
                    Some(shape)
                }
                _ => None,
            })
            .expect("the hue marker is drawn");
        assert!(
            (marker.rect.center().x - blue_x).abs() < 1.0,
            "the marker stays at the chosen hue: {} vs {}",
            marker.rect.center().x,
            blue_x
        );

        // A click at the square's top-right (s=1, v=1) then commits the chosen hue:
        // a saturated blue.
        let (_, out) = window.click_at(
            egui::pos2(square_rect.right(), square_rect.top()),
            &mut color,
            &[],
        );
        assert!(out.changed, "the square click commits");
        assert_eq!(color[2], 255, "blue's channel is full: {color:?}");
        assert!(
            color[0] < 20 && color[1] < 20,
            "a blue, not a warm gray: {color:?}"
        );
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

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the popup holds egui's
    // saturation-value square and hue strip, opaque, and the swatch rows.
    #[test]
    fn the_painted_shapes_match_the_working_color() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let suggestions = [[200, 16, 46]];
        let (nodes, _) = window.field(&mut color, &suggestions, Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &suggestions);
        let (nodes, _) = window.field(&mut color, &suggestions, Vec::new());

        let square_rect = egui_rect(entry(&nodes, "saturation and value"));
        let strip_rect = egui_rect(entry(&nodes, "hue"));
        let swatch_rect = egui_rect(entry(&nodes, "#C8102E"));

        // egui's Areas fade in over about a third of a second; the paint assertions
        // need the fade done, which takes some twenty frames at the harness's dt.
        for _ in 0..25 {
            window.field(&mut color, &suggestions, Vec::new());
        }

        // The square runs white to the hue color left to right, and darkens top to
        // bottom (the draft's canvas).
        let square = mesh_at(&window.shapes, square_rect, 0.5, |mesh| {
            mesh.vertices.len() > 20
        });
        let pure: Color32 = srgb_of(Hsva {
            h: hsva_of(color).h,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        });
        assert_eq!(
            color_at(square, square_rect.left_top()),
            Color32::WHITE,
            "the top-left is white"
        );
        assert_eq!(
            color_at(square, square_rect.right_top()),
            pure,
            "the top-right is the pure hue"
        );
        for corner in [square_rect.left_bottom(), square_rect.right_bottom()] {
            assert_eq!(
                color_at(square, corner),
                Color32::BLACK,
                "the bottom edge is black"
            );
        }

        // The strip runs the hues: red at both edges.
        let strip = mesh_at(&window.shapes, strip_rect, 0.5, |mesh| {
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color == Color32::RED)
        });
        for vertex in &strip.vertices {
            if vertex.pos.x == strip_rect.left() || vertex.pos.x == strip_rect.right() {
                assert_eq!(vertex.color, Color32::RED, "the hue edges are red");
            }
        }

        // The hue marker is the draft's 3px bar, two points taller than the strip and
        // centred on the hue's x.
        let marker = window
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape)
                    if shape.fill == Color32::WHITE
                        && (shape.rect.width() - 3.0).abs() < 0.5
                        && (shape.rect.height() - (STRIP_H + 2.0)).abs() < 0.5 =>
                {
                    Some(shape)
                }
                _ => None,
            })
            .expect("a 3px marker over the strip");
        let hue_x = lerp(strip_rect.left()..=strip_rect.right(), hsva_of(color).h);
        assert!(
            (marker.rect.center().x - hue_x).abs() < 0.5,
            "the marker centres on the hue's x: {} vs {}",
            marker.rect.center().x,
            hue_x
        );

        // A swatch's fill is its color.
        let fill = window
            .shapes
            .iter()
            .find_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape)
                    if shape.fill == color32([200, 16, 46])
                        && (shape.rect.center() - swatch_rect.center()).length() < 0.5 =>
                {
                    Some(shape)
                }
                _ => None,
            })
            .expect("a filled swatch at its bounds");
        assert_eq!(
            fill.fill,
            color32([200, 16, 46]),
            "the swatch fills with its color"
        );
    }

    // plans/libs/color_tools.md "Color-picker widget (Phase 8)": the gradients show the
    // color a click there picks (HSV over sRGB).
    #[test]
    fn a_click_inside_the_gradients_picks_the_shown_color() {
        let mut window = Headless::new();
        let mut color = [255, 128, 0];
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        window.click_at(centre(field_button(&nodes, 0)), &mut color, &[]);
        let (nodes, _) = window.field(&mut color, &[], Vec::new());
        let strip_rect = egui_rect(entry(&nodes, "hue"));
        let square_rect = egui_rect(entry(&nodes, "saturation and value"));
        // The Area fade-in is over before the paints are read.
        for _ in 0..25 {
            window.field(&mut color, &[], Vec::new());
        }

        // A click at hue 1/12, halfway between red's and yellow's columns, commits the
        // color the strip shows there.
        let strip = mesh_at(&window.shapes, strip_rect, 0.5, |mesh| {
            mesh.vertices
                .iter()
                .any(|vertex| vertex.color == Color32::RED)
        });
        let hue_x = strip_rect.left() + strip_rect.width() / 12.0;
        let shown = interpolated_at_x(strip, hue_x);
        let pos = egui::pos2(hue_x, (strip_rect.top() + strip_rect.bottom()) / 2.0);
        window.click_at(pos, &mut color, &[]);
        let channels = |a: [u8; 3], b: Color32| {
            [a[0], a[1], a[2]]
                .iter()
                .zip([b.r(), b.g(), b.b()])
                .map(|(&picked, shown)| picked.abs_diff(shown))
                .max()
                .unwrap_or(0)
        };
        assert!(
            channels(color, shown) <= 1,
            "the pick matches the shown color: {color:?} vs {shown:?}"
        );

        // A click at the square's centre commits the color its mesh interpolates there.
        let square = mesh_at(&window.shapes, square_rect, 0.5, |mesh| {
            mesh.vertices.len() > 20
        });
        let centre = square_rect.center();
        let shown = interpolated_in_cell(square, centre);
        window.click_at(centre, &mut color, &[]);
        assert!(
            channels(color, shown) <= 1,
            "the pick matches the shown color: {color:?} vs {shown:?}"
        );
    }
}
