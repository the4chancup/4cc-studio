//! How the save editor and the widgets draw a savefile name: its colour
//! codes applied as text colours, the in-game look. `append_name` is the
//! draw; `plain_format` is the run the codes leave plain.

use pes_savefile::model::names::name_spans;

/// The format of a plain run (no code, after a reset or a non-hex code):
/// `PLACEHOLDER`, which the widget replaces with its own text color —
/// `TextFormat::default()`'s gray is not it.
pub fn plain_format() -> egui::TextFormat {
    egui::TextFormat::simple(egui::FontId::default(), egui::Color32::PLACEHOLDER)
}

/// Appends `name`'s runs to `job`, each colored as `name_spans` gives it
/// (`None` runs the normal text color). Sections' byte ranges are
/// relative to `job`'s own text, so the spans append, never merge
/// another job's.
pub fn append_name(job: &mut egui::text::LayoutJob, name: &str) {
    for (color, text) in name_spans(name) {
        let format = match color {
            Some([red, green, blue]) => egui::TextFormat::simple(
                egui::FontId::default(),
                egui::Color32::from_rgb(red, green, blue),
            ),
            None => plain_format(),
        };
        job.append(text, 0.0, format);
    }
}
