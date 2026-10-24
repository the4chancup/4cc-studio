//! The cells the section forms repeat (the draft's `drag`, `text` and
//! weak "—" helpers): a number with a range, a text field of a given
//! width and the dash a cell draws where the row has no value.

/// A `DragValue` over `value` clamped to `range` (the draft's `drag`).
pub(crate) fn number(
    ui: &mut egui::Ui,
    value: &mut u8,
    range: std::ops::RangeInclusive<u8>,
) -> egui::Response {
    ui.add(egui::DragValue::new(value).range(range))
}

/// A single-line text field `width` points wide (the draft's `text`).
pub(crate) fn text(ui: &mut egui::Ui, value: &mut String, width: f32) -> egui::Response {
    ui.add(egui::TextEdit::singleline(value).desired_width(width))
}

/// The weak "—" a cell draws where the row has no value.
pub(crate) fn dash(ui: &mut egui::Ui) {
    ui.weak("—");
}
