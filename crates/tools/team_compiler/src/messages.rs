//! The Team compiler's message catalog (`team_compiler/messages.md` "Catalog"): each code's
//! severity, and the mapping of the structure pass's issues and the tool's own findings to
//! `Message`s. Until the help window needs them (Phase 8) a row holds no text: the console prints
//! the code with its context fields.

use std::borrow::Cow;

use aesthetics_export::{IssueScope, ValidationIssue};
use studio_core::{Disposition, ExportId, Message, MessageCode, Scope, Severity};

/// The tool's id: its CLI subcommand, its settings section and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "team-compiler";

/// A code's severity as the catalog's "Sev" column gives it; two codes depend on the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CatalogSeverity {
    Info,
    Warning,
    Error,
    /// `E/I`: an Error when `strict_file_type_check` is on, else an Info.
    ErrorOrInfo,
    /// `E/F`: Fatal when the disposition aborts the run, else an Error.
    ErrorOrFatal,
}

/// Every code this tool emits in Phase 3, with its catalog severity: the structure pass's
/// (`aesthetics_export::ISSUE_CODES`) and the tool's own.
const CATALOG: &[(&str, CatalogSeverity)] = &[
    // The tool's own export-level codes.
    ("export_extract_failed", CatalogSeverity::Error),
    ("export_disabled", CatalogSeverity::Info),
    ("export_identified", CatalogSeverity::Info),
    ("export_balls_skipped", CatalogSeverity::Info),
    ("multiple_ref_exports", CatalogSeverity::Error),
    // The structure pass's codes, in `ISSUE_CODES` order.
    ("nested_folders_fixed", CatalogSeverity::Warning),
    ("nested_root_ambiguous", CatalogSeverity::Error),
    ("nested_root_conflict", CatalogSeverity::Error),
    ("players_txt_invalid", CatalogSeverity::Error),
    ("refs_txt_ignored", CatalogSeverity::Warning),
    ("source_read_failed", CatalogSeverity::ErrorOrFatal),
    ("export_empty", CatalogSeverity::Error),
    ("team_name_unknown", CatalogSeverity::Error),
    ("players_txt_missing", CatalogSeverity::Error),
    ("players_txt_line_invalid", CatalogSeverity::Error),
    ("players_txt_slot_invalid", CatalogSeverity::Error),
    ("players_txt_slot_duplicate", CatalogSeverity::Error),
    ("players_txt_target_missing", CatalogSeverity::Error),
    ("player_unlisted", CatalogSeverity::Warning),
    ("player_folder_number_invalid", CatalogSeverity::Error),
    ("player_number_duplicate", CatalogSeverity::Error),
    ("file_type_disallowed", CatalogSeverity::ErrorOrInfo),
    ("common_file_disallowed", CatalogSeverity::ErrorOrInfo),
    ("shared_link_duplicate", CatalogSeverity::Error),
    ("link_target_missing", CatalogSeverity::Error),
    ("common_link_missing", CatalogSeverity::Error),
    ("link_target_dropped", CatalogSeverity::Error),
    ("shared_folder_orphaned", CatalogSeverity::Warning),
    ("fpc_conflict", CatalogSeverity::Error),
    ("ingame_face_explicit_face_model", CatalogSeverity::Error),
    ("texture_stem_conflict", CatalogSeverity::Error),
    ("fmdl_name_invalid", CatalogSeverity::Error),
    ("kit_folder_invalid", CatalogSeverity::Error),
    ("kit_slot_duplicate", CatalogSeverity::Error),
    ("kit_texture_name_invalid", CatalogSeverity::Error),
    ("kit_layout_conflict", CatalogSeverity::Error),
    ("kit_icon_invalid", CatalogSeverity::Warning),
    ("kit_all_file_ignored", CatalogSeverity::Warning),
    ("kit_all_unused", CatalogSeverity::Warning),
    ("kit_textures_inherited", CatalogSeverity::Info),
    ("portrait_name_invalid", CatalogSeverity::Error),
    ("logo_file_invalid", CatalogSeverity::Error),
    ("logo_role_duplicate", CatalogSeverity::Error),
    ("logo_small_without_main", CatalogSeverity::Error),
    ("root_file_unexpected", CatalogSeverity::Warning),
    ("notes_found", CatalogSeverity::Info),
    ("notes_encoding_invalid", CatalogSeverity::Error),
];

/// The severity `code` is shown at, given what was done about it and the strict setting.
fn severity(code: &str, disposition: Disposition, strict_file_type_check: bool) -> Severity {
    let catalog = CATALOG
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, severity)| *severity)
        .expect("every emitted code has a catalog row: the catalog test covers every code");
    match catalog {
        CatalogSeverity::Info => Severity::Info,
        CatalogSeverity::Warning => Severity::Warning,
        CatalogSeverity::Error => Severity::Error,
        // Keyed by the setting, not the disposition: `pass_through` keeps a disallowed file
        // (`Keep`) but its finding stays an Error.
        CatalogSeverity::ErrorOrInfo if strict_file_type_check => Severity::Error,
        CatalogSeverity::ErrorOrInfo => Severity::Info,
        CatalogSeverity::ErrorOrFatal if disposition == Disposition::AbortRun => Severity::Fatal,
        CatalogSeverity::ErrorOrFatal => Severity::Error,
    }
}

/// The message for one structure-pass issue of the export `export_id`: the issue's scope placed
/// in that export, its effective disposition, and its severity from the catalog.
pub(crate) fn issue_message(
    issue: &ValidationIssue,
    export_id: ExportId,
    strict_file_type_check: bool,
) -> Message {
    let scope = match &issue.scope {
        IssueScope::Export => Scope::Export { export_id },
        IssueScope::Folder(path) => Scope::Folder {
            export_id,
            path: path.clone(),
        },
        IssueScope::File(path) => Scope::File {
            export_id,
            path: path.clone(),
        },
        IssueScope::RosterEntry { file, line, slot } => Scope::RosterEntry {
            export_id,
            file: file.clone(),
            line: *line,
            slot: *slot,
        },
    };
    let disposition = match issue.disposition {
        aesthetics_export::Disposition::Keep => Disposition::Keep,
        aesthetics_export::Disposition::DropFile => Disposition::DropFile,
        aesthetics_export::Disposition::DropSlot => Disposition::DropSlot,
        aesthetics_export::Disposition::DropFolder => Disposition::DropFolder,
        aesthetics_export::Disposition::DropExport => Disposition::DropExport,
    };
    let context = issue
        .context
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect();
    message(
        issue.code,
        scope,
        disposition,
        context,
        strict_file_type_check,
    )
}

/// A message for one of the tool's own codes (discovery, routing, identity), whose severity the
/// strict setting never changes.
pub(crate) fn tool_message(
    code: &'static str,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(&'static str, String)>,
) -> Message {
    let context = context
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    message(code, scope, disposition, context, false)
}

fn message(
    code: &'static str,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(String, String)>,
    strict_file_type_check: bool,
) -> Message {
    Message {
        code: MessageCode {
            tool_id: TOOL_ID,
            code: Cow::Borrowed(code),
        },
        severity: severity(code, disposition, strict_file_type_check),
        disposition,
        scope,
        context,
    }
}

#[cfg(test)]
mod tests {
    use aesthetics_export::ISSUE_CODES;
    use vtree::ScopePath;

    use super::*;

    fn issue(
        code: &'static str,
        scope: IssueScope,
        disposition: aesthetics_export::Disposition,
    ) -> ValidationIssue {
        ValidationIssue {
            code,
            scope,
            context: vec![("file", "readme.txt".to_owned()), ("line", "x3".to_owned())],
            disposition,
            passed_through: false,
        }
    }

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    #[test]
    fn every_issue_code_has_exactly_one_row_and_no_row_is_doubled() {
        for code in ISSUE_CODES {
            let rows = CATALOG.iter().filter(|(known, _)| known == code).count();
            assert_eq!(rows, 1, "{code}");
        }
        for (index, (code, _)) in CATALOG.iter().enumerate() {
            assert!(
                CATALOG[index + 1..].iter().all(|(other, _)| other != code),
                "{code} appears twice"
            );
        }
        // The rows beyond the structure pass's are exactly the tool's own Phase 3 codes.
        let own: Vec<&str> = CATALOG
            .iter()
            .map(|(code, _)| *code)
            .filter(|code| !ISSUE_CODES.contains(code))
            .collect();
        assert_eq!(
            own,
            [
                "export_extract_failed",
                "export_disabled",
                "export_identified",
                "export_balls_skipped",
                "multiple_ref_exports"
            ]
        );
    }

    #[test]
    fn a_disallowed_file_is_an_error_only_under_the_strict_check() {
        for code in ["file_type_disallowed", "common_file_disallowed"] {
            assert_eq!(
                severity(code, Disposition::DropFolder, true),
                Severity::Error
            );
            // pass_through's Keep does not lower it.
            assert_eq!(severity(code, Disposition::Keep, true), Severity::Error);
            assert_eq!(severity(code, Disposition::Keep, false), Severity::Info);
        }
    }

    #[test]
    fn a_failed_read_is_fatal_only_when_it_aborts_the_run() {
        for disposition in [
            Disposition::Keep,
            Disposition::DropFile,
            Disposition::DropSlot,
            Disposition::DropFolder,
            Disposition::DropExport,
        ] {
            let message = tool_message("source_read_failed", Scope::Run, disposition, vec![]);
            assert_eq!(message.severity, Severity::Error, "{disposition:?}");
        }
        let message = tool_message(
            "source_read_failed",
            Scope::Run,
            Disposition::AbortRun,
            vec![],
        );
        assert_eq!(message.severity, Severity::Fatal);
    }

    #[test]
    fn plain_rows_keep_their_severity_whatever_the_settings() {
        for strict in [true, false] {
            assert_eq!(
                severity("player_unlisted", Disposition::DropFolder, strict),
                Severity::Warning
            );
            assert_eq!(
                severity("notes_found", Disposition::Keep, strict),
                Severity::Info
            );
            assert_eq!(
                severity("fpc_conflict", Disposition::DropFolder, strict),
                Severity::Error
            );
        }
    }

    #[test]
    #[should_panic(expected = "the catalog test covers every code")]
    fn a_code_with_no_row_is_a_programming_error() {
        severity("no_such_code", Disposition::Keep, true);
    }

    #[test]
    fn each_issue_scope_lands_in_the_export() {
        let export_id = ExportId(4);
        let cases = [
            (IssueScope::Export, Scope::Export { export_id }),
            (
                IssueScope::Folder(path("Players/03 - A")),
                Scope::Folder {
                    export_id,
                    path: path("Players/03 - A"),
                },
            ),
            (
                IssueScope::File(path("notes.txt")),
                Scope::File {
                    export_id,
                    path: path("notes.txt"),
                },
            ),
            (
                IssueScope::RosterEntry {
                    file: path("players.txt"),
                    line: 2,
                    slot: Some(3),
                },
                Scope::RosterEntry {
                    export_id,
                    file: path("players.txt"),
                    line: 2,
                    slot: Some(3),
                },
            ),
        ];
        for (issue_scope, scope) in cases {
            let issue = issue(
                "player_unlisted",
                issue_scope,
                aesthetics_export::Disposition::Keep,
            );
            assert_eq!(issue_message(&issue, export_id, true).scope, scope);
        }
    }

    #[test]
    fn each_disposition_maps_to_its_counterpart_with_the_code_and_context() {
        let cases = [
            (aesthetics_export::Disposition::Keep, Disposition::Keep),
            (
                aesthetics_export::Disposition::DropFile,
                Disposition::DropFile,
            ),
            (
                aesthetics_export::Disposition::DropSlot,
                Disposition::DropSlot,
            ),
            (
                aesthetics_export::Disposition::DropFolder,
                Disposition::DropFolder,
            ),
            (
                aesthetics_export::Disposition::DropExport,
                Disposition::DropExport,
            ),
        ];
        for (issue_disposition, disposition) in cases {
            let issue = issue("fpc_conflict", IssueScope::Export, issue_disposition);
            let message = issue_message(&issue, ExportId(0), true);
            assert_eq!(message.disposition, disposition);
            assert_eq!(
                message.code,
                MessageCode {
                    tool_id: "team-compiler",
                    code: Cow::Borrowed("fpc_conflict")
                }
            );
            assert_eq!(message.severity, Severity::Error);
            assert_eq!(
                message.context,
                [
                    ("file".to_owned(), "readme.txt".to_owned()),
                    ("line".to_owned(), "x3".to_owned())
                ]
            );
        }
    }

    #[test]
    fn an_issue_message_reads_the_strict_setting() {
        let issue = issue(
            "file_type_disallowed",
            IssueScope::Folder(path("Players/03 - A")),
            aesthetics_export::Disposition::Keep,
        );
        assert_eq!(
            issue_message(&issue, ExportId(0), true).severity,
            Severity::Error
        );
        assert_eq!(
            issue_message(&issue, ExportId(0), false).severity,
            Severity::Info
        );
    }

    #[test]
    fn a_tool_message_carries_its_scope_disposition_and_context() {
        let message = tool_message(
            "multiple_ref_exports",
            Scope::Run,
            Disposition::DropExport,
            vec![("exports", "refs a, refs b".to_owned())],
        );
        assert_eq!(message.code.tool_id, "team-compiler");
        assert_eq!(message.code.code, "multiple_ref_exports");
        assert_eq!(message.severity, Severity::Error);
        assert_eq!(message.scope, Scope::Run);
        assert_eq!(message.disposition, Disposition::DropExport);
        assert_eq!(
            message.context,
            [("exports".to_owned(), "refs a, refs b".to_owned())]
        );
    }
}
