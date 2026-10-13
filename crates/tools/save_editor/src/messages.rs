//! The Save editor's message catalog: each code the commands report, its severity, and the
//! `Message` it is sent as. Every finding is run-wide (`Scope::Run`) and keeps what it is
//! about (`Disposition::Keep`): a command works on one save, not on exports, and a finding
//! that drops something says so with its own code. Until the help window needs them a row
//! holds no text: the console prints the code with its context fields.

use std::borrow::Cow;

use pes_savefile::interchange::aesthetics_patch::PatchSkip;
use pes_savefile::interchange::team_toml::ImportNote;
use studio_core::{Disposition, Message, MessageCode, Scope, Severity};

/// The tool's id: its CLI subcommand, its settings section and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "save-editor";

/// The codes the commands report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    /// `export-toml` wrote one team's file (`team`, `path`).
    TeamExported,
    /// `import-toml`: a key the document's version stores and the save's does not; dropped
    /// (`path`).
    ImportKeyNotInVersion,
    /// `import-toml`: a value the save's version cannot store (an instruction it lacks);
    /// written as off (`path`, `label`).
    ImportValueNotEncodable,
    /// `import-toml`: a face type above the save's version's cap, or a skin the version has
    /// no custom slot for; written as `to` (`path`, `from`, `to`).
    ImportValueCapped,
    /// `import-toml`: a `[players.NN]` whose roster slot is empty; skipped (`slot`).
    ImportSlotEmpty,
    /// `apply-patch`: the team's players were written (`team`).
    PatchTeamApplied,
    /// `apply-patch`: the save has no team with this id; its players were not looked at
    /// (`team`).
    PatchTeamNotInSave,
    /// `apply-patch`: the save has no player with this id (`player`).
    PatchPlayerNotInSave,
}

impl Code {
    /// The code as the console prints it and the help window lists it.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Code::TeamExported => "team_exported",
            Code::ImportKeyNotInVersion => "import_key_not_in_version",
            Code::ImportValueNotEncodable => "import_value_not_encodable",
            Code::ImportValueCapped => "import_value_capped",
            Code::ImportSlotEmpty => "import_slot_empty",
            Code::PatchTeamApplied => "patch_team_applied",
            Code::PatchTeamNotInSave => "patch_team_not_in_save",
            Code::PatchPlayerNotInSave => "patch_player_not_in_save",
        }
    }

    /// How serious the finding is: an `Error` makes the command's exit code 1.
    fn severity(self) -> Severity {
        match self {
            Code::TeamExported | Code::PatchTeamApplied => Severity::Info,
            // A value the save did not take as written: worth knowing, not a failure (the
            // plan's cross-version import converts "with a warning list").
            Code::ImportKeyNotInVersion
            | Code::ImportValueNotEncodable
            | Code::ImportValueCapped
            | Code::ImportSlotEmpty => Severity::Warning,
            // A skipped team or player is an error and the rest is written (the plan's
            // `apply-patch`: exit 1).
            Code::PatchTeamNotInSave | Code::PatchPlayerNotInSave => Severity::Error,
        }
    }
}

/// The message for one import note: what the import did not apply as written, and why.
pub(crate) fn import_note(note: &ImportNote) -> Message {
    match note {
        ImportNote::NotInThisVersion { path } => {
            message(Code::ImportKeyNotInVersion, vec![("path", path.clone())])
        }
        ImportNote::NotEncodable { path, label } => message(
            Code::ImportValueNotEncodable,
            vec![("path", path.clone()), ("label", label.clone())],
        ),
        ImportNote::Capped { path, from, to } => message(
            Code::ImportValueCapped,
            vec![
                ("path", path.clone()),
                ("from", from.to_string()),
                ("to", to.to_string()),
            ],
        ),
        ImportNote::EmptySlot { slot } => {
            message(Code::ImportSlotEmpty, vec![("slot", format!("{slot:02}"))])
        }
    }
}

/// The message for one patch item the save had no target for.
pub(crate) fn patch_skip(skip: PatchSkip) -> Message {
    match skip {
        PatchSkip::TeamNotInSave { team } => {
            message(Code::PatchTeamNotInSave, vec![("team", team.to_string())])
        }
        PatchSkip::PlayerNotInSave { player } => message(
            Code::PatchPlayerNotInSave,
            vec![("player", player.to_string())],
        ),
    }
}

/// The message for `code` with its context fields, in template order.
pub(crate) fn message(code: Code, context: Vec<(&'static str, String)>) -> Message {
    Message {
        code: MessageCode {
            tool_id: TOOL_ID,
            code: Cow::Borrowed(code.as_str()),
        },
        severity: code.severity(),
        disposition: Disposition::Keep,
        scope: Scope::Run,
        context: context
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_carries_its_code_severity_and_context_run_wide() {
        let message = message(
            Code::TeamExported,
            vec![("team", "714".to_owned()), ("path", "714.toml".to_owned())],
        );
        assert_eq!(message.code.tool_id, "save-editor");
        assert_eq!(message.code.code, "team_exported");
        assert_eq!(message.severity, Severity::Info);
        assert_eq!(message.disposition, Disposition::Keep);
        assert_eq!(message.scope, Scope::Run);
        assert_eq!(
            message.context,
            vec![
                ("team".to_owned(), "714".to_owned()),
                ("path".to_owned(), "714.toml".to_owned())
            ]
        );
    }

    /// Severity, code and context of a message as one line, the way the tests read them.
    fn line(message: &Message) -> String {
        let context: Vec<String> = message
            .context
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        format!(
            "{:?} {} ({})",
            message.severity,
            message.code.code,
            context.join(", ")
        )
    }

    #[test]
    fn each_import_note_is_a_warning_with_its_fields() {
        let path = || "players.03.stats.star".to_owned();
        assert_eq!(
            line(&import_note(&ImportNote::NotInThisVersion { path: path() })),
            "Warning import_key_not_in_version (path=players.03.stats.star)"
        );
        assert_eq!(
            line(&import_note(&ImportNote::NotEncodable {
                path: path(),
                label: "anchoring".to_owned()
            })),
            "Warning import_value_not_encodable (path=players.03.stats.star, label=anchoring)"
        );
        assert_eq!(
            line(&import_note(&ImportNote::Capped {
                path: path(),
                from: 19,
                to: 0
            })),
            "Warning import_value_capped (path=players.03.stats.star, from=19, to=0)"
        );
        assert_eq!(
            line(&import_note(&ImportNote::EmptySlot { slot: 7 })),
            "Warning import_slot_empty (slot=07)"
        );
    }

    #[test]
    fn each_patch_skip_is_an_error_naming_its_item() {
        assert_eq!(
            line(&patch_skip(PatchSkip::TeamNotInSave { team: 999 })),
            "Error patch_team_not_in_save (team=999)"
        );
        assert_eq!(
            line(&patch_skip(PatchSkip::PlayerNotInSave { player: 71499 })),
            "Error patch_player_not_in_save (player=71499)"
        );
    }
}
