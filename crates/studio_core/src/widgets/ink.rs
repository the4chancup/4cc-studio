//! Where a laid-out text's glyphs paint, for placing text by its ink rather
//! than its layout box.

/// The horizontal span the glyphs of `galley` paint, from the galley's left
/// edge: its ink. A glyph's ink starts its side bearing in from the layout
/// box and can end before or past the box's right edge, so text placed by
/// its box stands up to a point off where the eye reads it. Empty for a
/// galley that paints nothing (no text, or only spaces).
pub fn ink_span(galley: &egui::Galley) -> egui::Rangef {
    let mut span = egui::Rangef::NOTHING;
    for row in &galley.rows {
        for glyph in &row.row.glyphs {
            if glyph.uv_rect.is_nothing() {
                continue;
            }
            let left = row.pos.x + glyph.pos.x + glyph.uv_rect.offset.x;
            span = span.union(egui::Rangef::new(left, left + glyph.uv_rect.size.x));
        }
    }
    span
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each of `texts` laid out at the installed theme's body font, in one
    /// frame (the fonts load on the first).
    fn galleys<const N: usize>(texts: [&str; N]) -> [std::sync::Arc<egui::Galley>; N] {
        let ctx = egui::Context::default();
        crate::theme::install(&ctx);
        let mut laid = None;
        let output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Body.resolve(ui.style());
            laid = Some(texts.map(|text| {
                ui.painter()
                    .layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::WHITE)
            }));
        });
        // Nothing renders here, so the frame's texture deltas are dropped on
        // purpose.
        output.drop_without_applying_deltas();
        laid.expect("run_ui calls the closure")
    }

    #[test]
    fn the_ink_lies_inside_the_box_and_a_leading_space_shifts_it_by_its_advance() {
        let [bare, spaced, blank] = galleys(["H", " H", "  "]);
        let ink = ink_span(&bare);
        // Its left bearing puts the ink in from the box; the bitmap's
        // rounding can end it on the box's right edge.
        assert!(
            ink.min > 0.0 && ink.max <= bare.size().x + 0.5,
            "an H's ink starts inside its box: {ink:?} in {}",
            bare.size().x
        );
        let space = spaced.size().x - bare.size().x;
        let shifted = ink_span(&spaced);
        assert!((shifted.min - (ink.min + space)).abs() < 1.0, "{shifted:?}");
        assert!((shifted.span() - ink.span()).abs() < 1.0, "{shifted:?}");
        let blank = ink_span(&blank);
        assert!(blank.min > blank.max, "spaces paint nothing: {blank:?}");
    }
}
