//! The Team compiler's message catalog (`team_compiler/messages.md` "Catalog"): each code's
//! severity, and the mapping of the structure pass's issues and the tool's own findings to
//! `Message`s. Until the help window needs them (Phase 8) a row holds no text: the console prints
//! the code with its context fields.

use std::borrow::Cow;

use aesthetics_export::{IssueScope, ValidationIssue};
use studio_core::{Disposition, ExportId, Message, MessageCode, Scope, Severity};

/// The tool's id: its CLI subcommand, its settings section and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "team-compiler";

/// The codes the tool reports itself, beside the structure pass's issues (whose codes are the
/// lib's strings): discovery, routing, identity, planning, processing and output findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    /// An archive cannot be opened, or its listing is refused.
    ExportExtractFailed,
    /// A `NO_USE` marker disables the export.
    ExportDisabled,
    /// The export's team, or the referees, as identified.
    ExportIdentified,
    /// A balls export, which the Balls compiler owns.
    ExportBallsSkipped,
    /// More than one non-disabled refs export in the run.
    MultipleRefExports,
    /// The export's first word names no teams-list row.
    TeamNameUnknown,
    /// A kit without `config.toml` gets the template config.
    KitConfigGenerated,
    /// A kit whose effective textures lack `kit.dds` (an empty folder included): the bundled
    /// checkerboard stands in as its main texture.
    KitPlaceholder,
    /// Phase 3 only: the export holds content `compile` cannot build yet; it is skipped.
    ContentNotYetCompiled,
    /// A file a task reads cannot be read from its export; its folder is left out.
    SourceReadFailed,
    /// A task could not build its entries; its folder is left out.
    FolderPackFailed,
    /// The CPK could not be written; the run's staging is discarded.
    CpkWriteFailed,
    /// The written CPK could not replace the previous one; the run's staging is discarded.
    OutputCommitFailed,
    /// `--no-deploy`: the CPK was promoted to the output folder instead of installed.
    DeploySkippedByFlag,
}

impl Code {
    /// Every code, for the catalog test: a variant missing here would make its first message
    /// panic in `severity`, so a new variant is added to this list too.
    #[cfg(test)]
    const ALL: [Code; 14] = [
        Code::ExportExtractFailed,
        Code::ExportDisabled,
        Code::ExportIdentified,
        Code::ExportBallsSkipped,
        Code::MultipleRefExports,
        Code::TeamNameUnknown,
        Code::KitConfigGenerated,
        Code::KitPlaceholder,
        Code::ContentNotYetCompiled,
        Code::SourceReadFailed,
        Code::FolderPackFailed,
        Code::CpkWriteFailed,
        Code::OutputCommitFailed,
        Code::DeploySkippedByFlag,
    ];

    /// The code as the catalog spells it, the stable id a message carries.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Code::ExportExtractFailed => "export_extract_failed",
            Code::ExportDisabled => "export_disabled",
            Code::ExportIdentified => "export_identified",
            Code::ExportBallsSkipped => "export_balls_skipped",
            Code::MultipleRefExports => "multiple_ref_exports",
            Code::TeamNameUnknown => "team_name_unknown",
            Code::KitConfigGenerated => "kit_config_generated",
            Code::KitPlaceholder => "kit_placeholder",
            Code::ContentNotYetCompiled => "content_not_yet_compiled",
            Code::SourceReadFailed => "source_read_failed",
            Code::FolderPackFailed => "folder_pack_failed",
            Code::CpkWriteFailed => "cpk_write_failed",
            Code::OutputCommitFailed => "output_commit_failed",
            Code::DeploySkippedByFlag => "deploy_skipped_by_flag",
        }
    }
}

/// A code's severity as the catalog's "Sev" column gives it; two codes depend on the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CatalogSeverity {
    Info,
    Warning,
    Error,
    Fatal,
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
    ("kit_config_generated", CatalogSeverity::Info),
    ("kit_placeholder", CatalogSeverity::Info),
    ("content_not_yet_compiled", CatalogSeverity::Error),
    ("folder_pack_failed", CatalogSeverity::ErrorOrFatal),
    ("cpk_write_failed", CatalogSeverity::Fatal),
    ("output_commit_failed", CatalogSeverity::Fatal),
    ("deploy_skipped_by_flag", CatalogSeverity::Info),
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
        .expect(
            "every code has a catalog row: the catalog test checks `Code::ALL` and `ISSUE_CODES`",
        );
    match catalog {
        CatalogSeverity::Info => Severity::Info,
        CatalogSeverity::Warning => Severity::Warning,
        CatalogSeverity::Error => Severity::Error,
        CatalogSeverity::Fatal => Severity::Fatal,
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

/// A message for one of the tool's own codes, whose severity the strict setting never changes.
pub(crate) fn tool_message(
    code: Code,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(&'static str, String)>,
) -> Message {
    let context = context
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    message(code.as_str(), scope, disposition, context, false)
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
        for code in Code::ALL {
            let rows = CATALOG
                .iter()
                .filter(|(known, _)| *known == code.as_str())
                .count();
            assert_eq!(rows, 1, "{code:?}");
        }
        // The rows beyond the structure pass's are exactly the tool's own codes.
        let own: Vec<&str> = Code::ALL
            .iter()
            .map(|code| code.as_str())
            .filter(|code| !ISSUE_CODES.contains(code))
            .collect();
        let extra: Vec<&str> = CATALOG
            .iter()
            .map(|(code, _)| *code)
            .filter(|code| !ISSUE_CODES.contains(code))
            .collect();
        assert_eq!(extra, own);
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
    fn a_failed_pack_is_fatal_only_when_it_aborts_the_run() {
        for disposition in [
            Disposition::Keep,
            Disposition::DropFile,
            Disposition::DropSlot,
            Disposition::DropFolder,
            Disposition::DropExport,
        ] {
            let message = tool_message(Code::FolderPackFailed, Scope::Run, disposition, vec![]);
            assert_eq!(message.severity, Severity::Error, "{disposition:?}");
        }
        let message = tool_message(
            Code::FolderPackFailed,
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
    #[should_panic(expected = "the catalog test checks")]
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
            Code::MultipleRefExports,
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
