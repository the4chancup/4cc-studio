//! The embedded rulesets the Ruleset editor's start page offers: the
//! latest two of the 4cc, VGL and VTL, Autumn 26 first, then DBC7.

use crate::file::file_sections;
use crate::ruleset::Ruleset;

/// One embedded ruleset offered on the Ruleset editor's start page.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    /// The template's ruleset.
    pub ruleset: Ruleset,
    /// Its CUSTOM CHECKS section as the embedded file carries it, `None`
    /// when it has none (VTL11 and VTL10 alone carry checks).
    pub custom_checks: Option<String>,
}

/// The embedded blocks, in the start page's order (Autumn 26 is
/// `rules.rs`'s embedded official block).
const BLOCKS: &[&str] = &[
    crate::rules::AUTUMN26,
    include_str!("../rulesets/spring26.rhai"),
    include_str!("../rulesets/vgl27.rhai"),
    include_str!("../rulesets/vgl26.rhai"),
    include_str!("../rulesets/vtl11.rhai"),
    include_str!("../rulesets/vtl10.rhai"),
    include_str!("../rulesets/dbc7.rhai"),
];

/// The start page's templates, in its order: Autumn 26, Spring 26, VGL27,
/// VGL26, VTL11, VTL10, DBC7.
///
/// Panics only if an embedded file does not load, which a test rules out.
pub fn templates() -> Vec<Template> {
    BLOCKS
        .iter()
        .map(|block| Template {
            ruleset: Ruleset::from_data_block(block).expect("an embedded ruleset loads"),
            custom_checks: file_sections(block).custom_checks,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// All seven templates load, in the start page's order; only the VTL
    /// ones carry custom checks.
    #[test]
    fn the_templates_load_in_the_start_pages_order() {
        let templates = templates();
        let names: Vec<&str> = templates
            .iter()
            .map(|template| template.ruleset.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "Autumn 26",
                "Spring 26",
                "VGL27",
                "VGL26",
                "VTL11",
                "VTL10",
                "DBC7"
            ]
        );
        let checks: Vec<bool> = templates
            .iter()
            .map(|template| template.custom_checks.is_some())
            .collect();
        assert_eq!(checks, [false, false, false, false, true, true, false]);
    }
}
