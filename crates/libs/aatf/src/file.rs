//! Assembling a rules file from a data block, the current logic and the
//! empty custom-checks hook.

/// The current logic section, banner included.
pub(crate) const LOGIC: &str = include_str!("../logic/logic.rhai");

/// The CUSTOM CHECKS banner line, under which a file's own checks go.
pub(crate) const CUSTOM_CHECKS: &str = "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====";

/// The whole rules file for `data_block`: the block, the current logic
/// and the empty custom-checks stub.
pub(crate) fn assemble(data_block: &str) -> String {
    format!(
        "{}\n\n{}\n\n{}\nfn custom_checks(team) {{ [] }}\n",
        data_block.trim_end(),
        LOGIC.trim_end(),
        CUSTOM_CHECKS
    )
}
