//! The Export upgrader's message catalog (`plans/export_upgrader.md` "Message
//! catalog"): each code the command reports, its severity, and the `Message` it
//! is sent as. The report names two exports: the input (export 1) for what is
//! found in it, the output (export 2) for the shared validation's findings on
//! what was written. Until the help window needs them a row holds no text: the
//! console prints the code with its context fields.

use std::borrow::Cow;

use aesthetics_export::{IssueScope, ValidationIssue};
use studio_core::{Disposition, ExportId, Message, MessageCode, Scope, Severity};

use crate::source::SourceFailure;

/// The tool's id: its CLI subcommand and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "export-upgrader";

/// The input export's id in this run's events.
pub(crate) const INPUT: ExportId = ExportId(1);
/// The output export's.
pub(crate) const OUTPUT: ExportId = ExportId(2);

/// The codes the tool reports itself. `team_name_unknown` is shared with the
/// structure pass's `ISSUE_CODES` (the plan's catalog takes the compiler's
/// code), so it lives in `CATALOG`'s issue rows and is emitted by string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    /// The input holds nothing an old export holds; nothing is written.
    NoOldExport,
    /// A `refs` export without a `refs.txt` holds no roster to migrate:
    /// nothing is written (step 9).
    RefsTxtMissing,
    /// The archive does not open or its listing is refused (the compiler's
    /// code); nothing is written (`path`, `error`).
    ExportExtractFailed,
    /// An entry inside the old format's folders that no step migrates
    /// (step 10), entry by entry; a leftover outside them is
    /// `leftover_dropped`.
    UnmigratedContent,
    /// The last line: nothing above Info, nothing dropped but a leftover.
    MigrationClean,
    /// The last line otherwise: the output is a draft for manual repair
    /// ("Verification").
    MigrationNotClean,
    /// Content that never reached a player in game (step 10): outside the
    /// old format's layout — a top-level folder the AET does not define, a
    /// root file other than the format's own, a file under `Other/` — or
    /// inside it — a stray file, an item folder no save record names, a
    /// file directly in `Faces/`, `Boots/` or `Gloves/`. Dropped, and the
    /// run stays clean since the Studio export needs none of it.
    LeftoverDropped,
    /// An old `.mtl` held text between its elements: dropped as the file is
    /// written back, since the compiler's reader refuses it (step 7).
    MtlStrayTextDropped,
    /// A face folder whose slot the save's roster leaves empty: it still
    /// becomes its player folder, with no `settings.toml` (`slot`).
    RosterSlotEmpty,
    /// A path a player folder and a merging item folder share with different
    /// bytes: the item folder is kept shared with a link instead (`path`, the
    /// first clashing path relative to the player folder).
    MergeKeptShared,
    /// Loose mode's assumed wearer of an item folder (step 11): the ID's
    /// position in its kind's old block of 25, reported so the pairing can be
    /// checked (`id`, the folder's four-digit ID; `slot`, the player's).
    WearerAssumed,
    /// A kit slot a Note kit entry gives and no `Kit Configs` config does:
    /// the kit still forms, a placeholder the compiler fills (`slot`).
    KitWithoutConfig,
    /// A kit slot a `Kit Configs` config gives and no Note kit entry does:
    /// its `config.toml` is written all the same, the kit offered as the old
    /// export never offered it (`slot`).
    ConfigWithoutKitEntry,
    /// A kit's config field named another texture: the named texture was
    /// copied into the kit under the field's generic name (`kit`, `stem` the
    /// field's, `from` the source's slot and stem).
    KitTextureCopied,
    /// A texture every kit folder held byte-identically: it moved to
    /// `Kits/all/` (`stem`, the first kit's spelling).
    KitTextureHoisted,
    /// A `Portraits/` file and its slot's face-folder portrait differ in
    /// bytes: the compiler's deep pass would refuse the export (`slot`).
    PortraitConflict,
    /// The old `_r` small logo is a different drawing, kept as
    /// `logo_small.png` (`share`, the differing pixels' percentage).
    LogoSmallCarried,
    /// The old `_r` small logo is a downscale of the large one, dropped:
    /// the compiler regenerates it (`share`).
    LogoSmallDropped,
    /// No usable `_r_ll` member: the largest usable one became `logo.png`.
    LogoFallback,
    /// An `.mtl`/`.fmdl` reference into the team's `Common/` kept as written
    /// but its `???`, respelled the team's ID: the file is not in the
    /// input's `Common/`, a link would clash (the folder already holds a
    /// texture of that stem, which is the one the model will wear — the
    /// compiler looks a game path's file name up in the folder first), or a
    /// subfolder `.mtl` names a Common subfolder (`reference`, the path as
    /// written).
    CommonReferenceKept,
    /// A rostered player's save record wears a custom boots or gloves ID
    /// (above 100) no migrated folder claims: the content is missing (step
    /// 8; `slot`, `name`, `key` the ID's own, `id`). Checked for every
    /// rostered player of a Full output but in a Midcup one only for a
    /// player the output gives a folder.
    CustomModelMissing,
    /// A stored value no `settings.toml` key can hold, left out of the
    /// player's file (`slot`, `name`, `key` the dotted key, `value` the
    /// stored value).
    SettingsValueRefused,
    /// A `settings.toml` generated from a stripped record: the record still
    /// is his look until the team's first Studio compile, which keeps its
    /// own file instead (`slot`, `name`).
    SettingsFromStrippedRecord,
    /// A player's `settings.toml` could not be generated at all (`slot`,
    /// `name`, `error` the failure's text).
    SettingsNotGenerated,
    /// Model-folder mode: where an output folder goes once dropped into a
    /// Studio export (`into`: `Players`, `Boots`, `Gloves`, or `root` for
    /// `Common/`).
    ModelFolderDestination,
}

impl Code {
    /// The code as the console prints it and the help window lists it.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Code::NoOldExport => "no_old_export",
            Code::RefsTxtMissing => "refs_txt_missing",
            Code::ExportExtractFailed => "export_extract_failed",
            Code::UnmigratedContent => "unmigrated_content",
            Code::MigrationClean => "migration_clean",
            Code::MigrationNotClean => "migration_not_clean",
            Code::LeftoverDropped => "leftover_dropped",
            Code::MtlStrayTextDropped => "mtl_stray_text_dropped",
            Code::RosterSlotEmpty => "roster_slot_empty",
            Code::MergeKeptShared => "merge_kept_shared",
            Code::WearerAssumed => "wearer_assumed",
            Code::KitWithoutConfig => "kit_without_config",
            Code::ConfigWithoutKitEntry => "config_without_kit_entry",
            Code::KitTextureCopied => "kit_texture_copied",
            Code::KitTextureHoisted => "kit_texture_hoisted",
            Code::PortraitConflict => "portrait_conflict",
            Code::LogoSmallCarried => "logo_small_carried",
            Code::LogoSmallDropped => "logo_small_dropped",
            Code::LogoFallback => "logo_fallback",
            Code::CommonReferenceKept => "common_reference_kept",
            Code::CustomModelMissing => "custom_model_missing",
            Code::SettingsValueRefused => "settings_value_refused",
            Code::SettingsFromStrippedRecord => "settings_from_stripped_record",
            Code::SettingsNotGenerated => "settings_not_generated",
            Code::ModelFolderDestination => "model_folder_destination",
        }
    }

    /// Every own code, for the catalog test.
    #[cfg(test)]
    const ALL: &'static [Code] = &[
        Code::NoOldExport,
        Code::RefsTxtMissing,
        Code::ExportExtractFailed,
        Code::UnmigratedContent,
        Code::MigrationClean,
        Code::MigrationNotClean,
        Code::LeftoverDropped,
        Code::MtlStrayTextDropped,
        Code::RosterSlotEmpty,
        Code::MergeKeptShared,
        Code::WearerAssumed,
        Code::KitWithoutConfig,
        Code::ConfigWithoutKitEntry,
        Code::KitTextureCopied,
        Code::KitTextureHoisted,
        Code::PortraitConflict,
        Code::LogoSmallCarried,
        Code::LogoSmallDropped,
        Code::LogoFallback,
        Code::CommonReferenceKept,
        Code::CustomModelMissing,
        Code::SettingsValueRefused,
        Code::SettingsFromStrippedRecord,
        Code::SettingsNotGenerated,
        Code::ModelFolderDestination,
    ];
}

/// Every code this tool emits, with its severity: the tool's own, then the
/// structure pass's (`aesthetics_export::ISSUE_CODES`) in `ISSUE_CODES` order.
/// The issue severities are the Team compiler's catalog's, each run-dependent
/// row taken as for a compile that deploys with pass-through and the strict
/// file-type check on (the output's check runs with it), and an E/F row as an
/// Error (`plans/export_upgrader.md` "Message catalog").
const CATALOG: &[(&str, Severity)] = &[
    // The tool's own codes.
    ("no_old_export", Severity::Error),
    ("refs_txt_missing", Severity::Error),
    ("export_extract_failed", Severity::Error),
    ("unmigrated_content", Severity::Error),
    ("migration_clean", Severity::Info),
    ("migration_not_clean", Severity::Info),
    ("leftover_dropped", Severity::Warning),
    ("mtl_stray_text_dropped", Severity::Info),
    ("roster_slot_empty", Severity::Warning),
    ("merge_kept_shared", Severity::Info),
    ("wearer_assumed", Severity::Info),
    ("kit_without_config", Severity::Warning),
    ("config_without_kit_entry", Severity::Warning),
    ("kit_texture_copied", Severity::Info),
    ("kit_texture_hoisted", Severity::Info),
    ("portrait_conflict", Severity::Error),
    ("logo_small_carried", Severity::Info),
    ("logo_small_dropped", Severity::Info),
    ("logo_fallback", Severity::Warning),
    ("common_reference_kept", Severity::Warning),
    ("custom_model_missing", Severity::Error),
    ("settings_value_refused", Severity::Warning),
    ("settings_from_stripped_record", Severity::Info),
    ("settings_not_generated", Severity::Error),
    ("model_folder_destination", Severity::Info),
    // The structure pass's codes, in `ISSUE_CODES` order.
    ("nested_folders_fixed", Severity::Warning),
    ("nested_root_ambiguous", Severity::Error),
    ("nested_root_conflict", Severity::Error),
    ("players_txt_invalid", Severity::Error),
    ("refs_txt_ignored", Severity::Warning),
    ("source_read_failed", Severity::Error),
    ("export_empty", Severity::Error),
    ("team_name_unknown", Severity::Error),
    ("export_tag_missing", Severity::Error),
    ("export_layout_old", Severity::Error),
    ("players_txt_missing", Severity::Error),
    ("players_txt_line_invalid", Severity::Error),
    ("players_txt_slot_invalid", Severity::Error),
    ("players_txt_slot_duplicate", Severity::Error),
    ("players_txt_target_missing", Severity::Error),
    ("player_unlisted", Severity::Warning),
    ("player_folder_number_invalid", Severity::Error),
    ("player_number_duplicate", Severity::Error),
    ("file_type_disallowed", Severity::Error),
    ("common_file_disallowed", Severity::Error),
    ("player_layout_proto", Severity::Error),
    ("shared_link_duplicate", Severity::Error),
    ("link_target_missing", Severity::Error),
    ("common_link_missing", Severity::Error),
    ("link_target_dropped", Severity::Error),
    ("shared_folder_orphaned", Severity::Warning),
    ("fpc_conflict", Severity::Error),
    ("ingame_face_explicit_face_model", Severity::Error),
    ("texture_stem_conflict", Severity::Error),
    ("fmdl_name_invalid", Severity::Error),
    ("model_name_invalid", Severity::Error),
    ("edithair_unsupported", Severity::Error),
    ("kit_folder_invalid", Severity::Error),
    ("kit_slot_duplicate", Severity::Error),
    ("kit_texture_name_invalid", Severity::Error),
    ("kit_layout_conflict", Severity::Error),
    ("kit_icon_invalid", Severity::Warning),
    ("kit_all_file_ignored", Severity::Warning),
    ("kit_all_unused", Severity::Warning),
    ("kit_textures_inherited", Severity::Info),
    ("portrait_name_invalid", Severity::Error),
    ("logo_file_invalid", Severity::Error),
    ("logo_role_duplicate", Severity::Error),
    ("logo_small_without_main", Severity::Error),
    ("root_file_unexpected", Severity::Warning),
    ("notes_found", Severity::Info),
    ("notes_encoding_invalid", Severity::Error),
];

/// The severity `code` is shown at.
fn severity(code: &str) -> Severity {
    CATALOG
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, severity)| *severity)
        .expect(
            "every code has a catalog row: the catalog test checks `Code::ALL` and `ISSUE_CODES`",
        )
}

/// The message for `code` with its scope, disposition and context fields.
fn message(
    code: &'static str,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(&'static str, String)>,
) -> Message {
    Message {
        code: MessageCode {
            tool_id: TOOL_ID,
            code: Cow::Borrowed(code),
        },
        severity: severity(code),
        disposition,
        scope,
        context: context
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    }
}

/// A message for one of the tool's own codes.
pub(crate) fn tool_message(
    code: Code,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(&'static str, String)>,
) -> Message {
    message(code.as_str(), scope, disposition, context)
}

/// `export_extract_failed` on `export_id` for `failure`: the archive does not
/// open or its listing is refused, nothing is written (`path`, `error`).
pub(crate) fn extract_failed(export_id: ExportId, failure: SourceFailure) -> Message {
    tool_message(
        Code::ExportExtractFailed,
        Scope::Export { export_id },
        Disposition::DropExport,
        vec![("path", failure.path), ("error", failure.error)],
    )
}

/// `source_read_failed`: the file at `failure.path` did not read after the
/// listing did, `disposition` what becomes of it (`path`, `error`). A shared
/// code — one of `ISSUE_CODES` — so it has no `Code` variant and takes the
/// catalog's row.
pub(crate) fn read_failed(
    export_id: ExportId,
    failure: SourceFailure,
    disposition: Disposition,
) -> Message {
    let scope = Scope::File {
        export_id,
        path: vtree::ScopePath::new(&failure.path)
            .expect("a source path was checked when the export opened"),
    };
    message(
        "source_read_failed",
        scope,
        disposition,
        vec![("path", failure.path), ("error", failure.error)],
    )
}

/// `team_name_unknown`: the team resolves to no row of the list, the `Team:`
/// line's file or the export named (`name`, the value as written). A shared
/// code — one of `ISSUE_CODES` — so it has no `Code` variant and takes the
/// catalog's row.
pub(crate) fn team_name_unknown(scope: Scope, name: String) -> Message {
    message(
        "team_name_unknown",
        scope,
        Disposition::DropExport,
        vec![("name", name)],
    )
}

/// The message for one shared-validation issue of the output export
/// `export_id`: the issue's scope placed in that export, its effective
/// disposition, and its severity from the catalog.
pub(crate) fn issue_message(issue: &ValidationIssue, export_id: ExportId) -> Message {
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
    let context: Vec<(&'static str, String)> = issue
        .context
        .iter()
        .map(|(key, value)| (*key, value.clone()))
        .collect();
    message(issue.code, scope, disposition, context)
}

#[cfg(test)]
mod tests {
    use aesthetics_export::ISSUE_CODES;

    use super::*;

    #[test]
    fn every_issue_code_and_every_own_code_has_exactly_one_catalog_row() {
        for code in ISSUE_CODES {
            let rows = CATALOG.iter().filter(|(known, _)| known == code).count();
            assert_eq!(rows, 1, "{code}");
        }
        for code in Code::ALL {
            let rows = CATALOG
                .iter()
                .filter(|(known, _)| *known == code.as_str())
                .count();
            assert_eq!(rows, 1, "{}", code.as_str());
        }
        for (index, (code, _)) in CATALOG.iter().enumerate() {
            assert!(
                CATALOG[index + 1..].iter().all(|(other, _)| other != code),
                "duplicate row: {code}"
            );
        }
    }

    #[test]
    fn each_code_has_the_catalogs_severity() {
        for (code, expected) in [
            ("migration_clean", Severity::Info),
            ("migration_not_clean", Severity::Info),
            ("leftover_dropped", Severity::Warning),
            ("mtl_stray_text_dropped", Severity::Info),
            ("unmigrated_content", Severity::Error),
            ("no_old_export", Severity::Error),
            ("export_extract_failed", Severity::Error),
            ("root_file_unexpected", Severity::Warning),
            ("file_type_disallowed", Severity::Error),
            ("common_file_disallowed", Severity::Error),
            ("export_empty", Severity::Error),
            ("team_name_unknown", Severity::Error),
            ("nested_folders_fixed", Severity::Warning),
            ("kit_textures_inherited", Severity::Info),
        ] {
            assert_eq!(severity(code), expected, "{code}");
        }
    }

    #[test]
    fn no_own_code_is_also_an_issue_code() {
        for code in Code::ALL {
            assert!(!ISSUE_CODES.contains(&code.as_str()), "{}", code.as_str());
        }
    }

    #[test]
    fn read_failed_is_a_file_scoped_error_naming_the_path_and_reason() {
        let failure = crate::source::SourceFailure {
            path: "co Note.txt".to_owned(),
            error: "denied".to_owned(),
        };
        let message = read_failed(ExportId(1), failure, Disposition::DropExport);
        assert_eq!(message.code.tool_id, "export-upgrader");
        assert_eq!(message.code.code, "source_read_failed");
        assert_eq!(message.severity, Severity::Error);
        assert_eq!(message.disposition, Disposition::DropExport);
        assert_eq!(
            message.scope,
            Scope::File {
                export_id: ExportId(1),
                path: vtree::ScopePath::new("co Note.txt").unwrap()
            }
        );
        assert_eq!(
            message.context,
            vec![
                ("path".to_owned(), "co Note.txt".to_owned()),
                ("error".to_owned(), "denied".to_owned())
            ]
        );
    }

    #[test]
    fn a_message_carries_its_code_severity_scope_and_context() {
        let message = tool_message(
            Code::UnmigratedContent,
            Scope::File {
                export_id: ExportId(1),
                path: vtree::ScopePath::new("Other/readme.txt").unwrap(),
            },
            Disposition::DropFile,
            vec![],
        );
        assert_eq!(message.code.tool_id, "export-upgrader");
        assert_eq!(message.code.code, "unmigrated_content");
        assert_eq!(message.severity, Severity::Error);
    }
}
