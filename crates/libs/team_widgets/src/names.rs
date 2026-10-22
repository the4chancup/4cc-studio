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

#[cfg(test)]
mod tests {
    use super::*;

    // A coded run takes its code's color; a plain run, before a code or
    // after a reset, takes `PLACEHOLDER`, so the widget's own text color
    // replaces it.
    #[test]
    fn a_names_runs_carry_their_code_or_the_placeholder() {
        let mut job = egui::text::LayoutJob::default();
        append_name(&mut job, "WHEN THE \x11cb7bec5ffWORK RESULT\x11d WAS GOOD");
        let runs: Vec<(&str, egui::Color32)> = job
            .sections
            .iter()
            .map(|section| {
                let range = section.byte_range.start.0..section.byte_range.end.0;
                (&job.text[range], section.format.color)
            })
            .collect();
        assert_eq!(
            runs,
            [
                ("WHEN THE ", egui::Color32::PLACEHOLDER),
                ("WORK RESULT", egui::Color32::from_rgb(0xb7, 0xbe, 0xc5)),
                (" WAS GOOD", egui::Color32::PLACEHOLDER),
            ]
        );
    }
}
