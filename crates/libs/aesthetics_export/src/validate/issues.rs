//! What the structure pass reports. The crate decides each issue's
//! disposition; the consumer maps each issue to its own message (severity,
//! text, hint), testing that every code in `ISSUE_CODES` has a catalog row.

use std::collections::BTreeSet;

use vtree::ScopePath;

use crate::listing::ValidationContext;

/// One finding of the structure pass, with what it is about and what becomes
/// of it. `code` is stable and always one of `ISSUE_CODES`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// The stable code (always one of `ISSUE_CODES`).
    pub code: &'static str,
    /// What the issue is about.
    pub scope: IssueScope,
    /// Structured fields, in the message template's order.
    pub context: Vec<(&'static str, String)>,
    /// The effective disposition: `Keep` when `pass_through` kept the item.
    pub disposition: Disposition,
    /// `pass_through` turned a `DropFile`/`DropFolder` into `Keep`.
    pub passed_through: bool,
}

/// What a `ValidationIssue` is about, within the one export this crate sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueScope {
    /// The export itself.
    Export,
    /// One folder (player, shared or kit).
    Folder(ScopePath),
    /// One file.
    File(ScopePath),
    /// One roster-file line: the file, the 1-based line number, and the slot
    /// when it parsed.
    RosterEntry {
        /// The roster file's path.
        file: ScopePath,
        /// The 1-based line number.
        line: usize,
        /// The parsed slot, when the line carries one.
        slot: Option<u16>,
    },
}

/// What becomes of the item an issue's scope names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Disposition {
    /// Kept unchanged.
    Keep,
    /// One file dropped.
    DropFile,
    /// One normalized roster assignment dropped.
    DropSlot,
    /// One folder dropped.
    DropFolder,
    /// The whole export dropped.
    DropExport,
}

/// The shared issue constructor: every code the crate emits must be listed in
/// `ISSUE_CODES`.
pub(crate) fn issue(
    code: &'static str,
    scope: IssueScope,
    context: Vec<(&'static str, String)>,
    disposition: Disposition,
) -> ValidationIssue {
    debug_assert!(ISSUE_CODES.contains(&code), "{code} not in ISSUE_CODES");
    ValidationIssue {
        code,
        scope,
        context,
        disposition,
        passed_through: false,
    }
}

/// The structure-pass codes `pass_through` can keep ("Validation semantics" →
/// "Pass-through eligibility"): the eligible `DropFile`/`DropFolder`
/// dispositions only.
pub(crate) const PASS_THROUGH_ELIGIBLE: [&str; 4] = [
    "link_target_missing",
    "common_link_missing",
    "file_type_disallowed",
    "common_file_disallowed",
];

/// The context-aware issue constructor: `pass_through` turns an eligible
/// `DropFile`/`DropFolder` into `Keep` and marks it (`passed_through`).
/// Every later step reads the effective disposition only.
pub(crate) fn issue_in(
    context: &crate::listing::ValidationContext,
    code: &'static str,
    scope: IssueScope,
    issue_context: Vec<(&'static str, String)>,
    disposition: Disposition,
) -> ValidationIssue {
    let mut issue = issue(code, scope, issue_context, disposition);
    if context.pass_through
        && PASS_THROUGH_ELIGIBLE.contains(&code)
        && matches!(
            issue.disposition,
            Disposition::DropFile | Disposition::DropFolder
        )
    {
        issue.disposition = Disposition::Keep;
        issue.passed_through = true;
    }
    issue
}

/// `disposition` when `strict_file_type_check` is on, `Keep` when off — the
/// strict/lenient split shared by the allowlist checks.
pub(crate) fn strict_disposition(
    context: &ValidationContext,
    disposition: Disposition,
) -> Disposition {
    if context.strict_file_type_check {
        disposition
    } else {
        Disposition::Keep
    }
}

/// The dropped folder and file paths, by fold key, from the issues'
/// effective dispositions (`(folders, files)`).
pub(crate) fn dropped_scopes(issues: &[ValidationIssue]) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut folders = BTreeSet::new();
    let mut files = BTreeSet::new();
    for issue in issues {
        match (&issue.scope, issue.disposition) {
            (IssueScope::Folder(path), Disposition::DropFolder) => {
                folders.insert(path.fold_key());
            }
            (IssueScope::File(path), Disposition::DropFile) => {
                files.insert(path.fold_key());
            }
            _ => {}
        }
    }
    (folders, files)
}

/// Every code the crate emits, listed here so consumers can test their
/// catalog against it.
pub const ISSUE_CODES: &[&str] = &[
    "nested_folders_fixed",
    "nested_root_ambiguous",
    "nested_root_conflict",
    "players_txt_invalid",
    "refs_txt_ignored",
    "source_read_failed",
    "export_empty",
    "team_name_unknown",
    "players_txt_missing",
    "players_txt_line_invalid",
    "players_txt_slot_invalid",
    "players_txt_slot_duplicate",
    "players_txt_target_missing",
    "player_unlisted",
    "player_folder_number_invalid",
    "player_number_duplicate",
    "file_type_disallowed",
    "common_file_disallowed",
    "shared_link_duplicate",
    "link_target_missing",
    "common_link_missing",
    "link_target_dropped",
    "shared_folder_orphaned",
    "fpc_conflict",
    "ingame_face_explicit_face_model",
    "texture_stem_conflict",
    "fmdl_name_invalid",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pass_through_eligible_code_exists() {
        for code in PASS_THROUGH_ELIGIBLE {
            assert!(ISSUE_CODES.contains(&code), "{code}");
        }
    }
}
