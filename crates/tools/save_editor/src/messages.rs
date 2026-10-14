//! The Save editor's message catalog: each code the commands report, its severity, and the
//! `Message` it is sent as. Every finding is run-wide (`Scope::Run`) and keeps what it is
//! about (`Disposition::Keep`): a command works on one save, not on exports, and a finding
//! that drops something says so with its own code. Until the help window needs them a row
//! holds no text: the console prints the code with its context fields.

use std::borrow::Cow;

use pes_savefile::interchange::aesthetics_patch::PatchSkip;
use pes_savefile::interchange::team_toml::ImportNote;
use pes_savefile::ops::compare::PlayerDiff;
use studio_core::{Disposition, Message, MessageCode, Scope, Severity};

use crate::session::Stripped;

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
    /// `transplant`: the target's appearance id is -1, so the game does not read the record
    /// the donor's fields went into; the tables decide his look (`player`).
    TransplantTargetStripped,
    /// `transplant`: the donor's appearance id is -1, so the fields copied are his record's,
    /// not the look the game shows for him (`player`).
    TransplantDonorStripped,
    /// `diff`: one field of a player in both saves, old to new (`player`, `field`, `old`,
    /// `new`).
    PlayerDiff,
    /// `diff`: a player in one save alone (`player`, `save`: `first` or `second`).
    PlayerOnlyIn,
    /// `diff`: the player's appearance id is -1 in `save` (`first`, `second` or `both`); his
    /// record is compared, not the look the game shows (`player`, `save`).
    PlayerStripped,
    /// `export-teams-list`: a team of the save the list lacked, appended or put into a
    /// placeholder's slot (`team`, `name`).
    TeamsListAdded,
    /// `export-teams-list`: a name in both whose id becomes the save's (`name`, `old`, `new`).
    TeamsListOverridden,
    /// `export-teams-list`: a change dropped because its id collided with another row of the
    /// merged list (`team`, `name`).
    TeamsListUnresolved,
    /// `export-teams-list`: a team of the save left out because an earlier team folds to the
    /// same name, which the list can hold once (`team`, `name`, `kept`).
    TeamsListNameDuplicate,
    /// `export-teams-list`: the working rows that ended unchanged (`count`).
    TeamsListKept,
    /// `export-teams-list`: the merged list was written (`path`).
    TeamsListWritten,
    /// `export-teams-list` without `--yes`: the summary only, the file untouched (`path`).
    TeamsListNotWritten,
    /// `export-teams-list --yes`: the data directory cannot take the file; nothing is
    /// written (`path`, `error`).
    TeamsListReadOnly,
}

/// Which of `diff`'s two saves a finding is about, as the console names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Side {
    /// The first save on the command line.
    First,
    /// The second.
    Second,
    /// Both saves.
    Both,
}

impl Side {
    /// The word the finding's `save` field carries.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Side::First => "first",
            Side::Second => "second",
            Side::Both => "both",
        }
    }
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
            Code::TransplantTargetStripped => "transplant_target_stripped",
            Code::TransplantDonorStripped => "transplant_donor_stripped",
            Code::PlayerDiff => "player_diff",
            Code::PlayerOnlyIn => "player_only_in",
            Code::PlayerStripped => "player_stripped",
            Code::TeamsListAdded => "teams_list_added",
            Code::TeamsListOverridden => "teams_list_overridden",
            Code::TeamsListUnresolved => "teams_list_unresolved",
            Code::TeamsListNameDuplicate => "teams_list_name_duplicate",
            Code::TeamsListKept => "teams_list_kept",
            Code::TeamsListWritten => "teams_list_written",
            Code::TeamsListNotWritten => "teams_list_not_written",
            Code::TeamsListReadOnly => "teams_list_read_only",
        }
    }

    /// How serious the finding is: an `Error` makes the command's exit code 1.
    fn severity(self) -> Severity {
        match self {
            // Differences and marks are `diff`'s output, not errors (the plan: exit 0).
            Code::TeamExported
            | Code::PatchTeamApplied
            | Code::PlayerDiff
            | Code::PlayerOnlyIn
            | Code::PlayerStripped
            | Code::TeamsListAdded
            | Code::TeamsListOverridden
            | Code::TeamsListKept
            | Code::TeamsListWritten
            | Code::TeamsListNotWritten => Severity::Info,
            // A value the save did not take as written, or a record the game does not read:
            // worth knowing, not a failure (the plan's cross-version import converts "with a
            // warning list").
            Code::ImportKeyNotInVersion
            | Code::ImportValueNotEncodable
            | Code::ImportValueCapped
            | Code::ImportSlotEmpty
            | Code::TransplantTargetStripped
            | Code::TransplantDonorStripped => Severity::Warning,
            // A skipped team or player is an error and the rest is written (the plan's
            // `apply-patch`: exit 1); so is a dropped teams-list change and a list that
            // cannot be written (`export-teams-list`: exit 1).
            Code::PatchTeamNotInSave
            | Code::PatchPlayerNotInSave
            | Code::TeamsListUnresolved
            | Code::TeamsListNameDuplicate
            | Code::TeamsListReadOnly => Severity::Error,
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

/// The message for one transplant pair's player at -1.
pub(crate) fn stripped_mark(mark: Stripped) -> Message {
    let (code, player) = match mark {
        Stripped::Target(player) => (Code::TransplantTargetStripped, player),
        Stripped::Donor(player) => (Code::TransplantDonorStripped, player),
    };
    message(code, vec![("player", player.to_string())])
}

/// One `diff` row's texts: the field by its schema name (`BootsId`, `FacialHairType`,
/// `Name`; `face_run` for the run's bits no field names, whose values are the two runs'
/// fingerprints) and the old and new values. The finding and the JSON report carry the
/// same three strings.
pub(crate) fn diff_fields(row: &PlayerDiff) -> (String, String, String) {
    match row {
        PlayerDiff::Field { field, old, new } => {
            (format!("{field:?}"), old.to_string(), new.to_string())
        }
        PlayerDiff::Text { text, old, new } => (format!("{text:?}"), old.clone(), new.clone()),
        PlayerDiff::Face { field, old, new } => {
            (format!("{field:?}"), old.to_string(), new.to_string())
        }
        PlayerDiff::FaceRun { old, new } => {
            ("face_run".to_owned(), old.to_string(), new.to_string())
        }
    }
}

/// The message for one `diff` row of `player`: `field`, `old` and `new` are `diff_fields`'s.
pub(crate) fn player_diff(player: u32, field: &str, old: &str, new: &str) -> Message {
    message(
        Code::PlayerDiff,
        vec![
            ("player", player.to_string()),
            ("field", field.to_owned()),
            ("old", old.to_owned()),
            ("new", new.to_owned()),
        ],
    )
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
    fn each_diff_row_kind_is_an_info_line_with_the_field_old_and_new() {
        use pes_savefile::ops::fingerprint::FaceHash;
        use pes_savefile::schema::fields::{PlayerField, PlayerText};
        use pes_savefile::schema::ingame_face::IngameFaceField;
        let line_of = |row: &PlayerDiff| {
            let (field, old, new) = diff_fields(row);
            line(&player_diff(70105, &field, &old, &new))
        };
        assert_eq!(
            line_of(&PlayerDiff::Field {
                field: PlayerField::BootsId,
                old: 3601,
                new: 12
            }),
            "Info player_diff (player=70105, field=BootsId, old=3601, new=12)"
        );
        assert_eq!(
            line_of(&PlayerDiff::Text {
                text: PlayerText::ShirtName,
                old: "A".to_owned(),
                new: "B".to_owned()
            }),
            "Info player_diff (player=70105, field=ShirtName, old=A, new=B)"
        );
        assert_eq!(
            line_of(&PlayerDiff::Face {
                field: IngameFaceField::NoseType,
                old: 1,
                new: 2
            }),
            "Info player_diff (player=70105, field=NoseType, old=1, new=2)"
        );
        assert_eq!(
            line_of(&PlayerDiff::FaceRun {
                old: FaceHash([0, 1, 0xab, 0xff]),
                new: FaceHash([0; 4])
            }),
            "Info player_diff (player=70105, field=face_run, old=0001abff, new=00000000)"
        );
    }

    #[test]
    fn each_transplant_mark_is_a_warning_naming_the_player() {
        assert_eq!(
            line(&stripped_mark(Stripped::Target(70105))),
            "Warning transplant_target_stripped (player=70105)"
        );
        assert_eq!(
            line(&stripped_mark(Stripped::Donor(70205))),
            "Warning transplant_donor_stripped (player=70205)"
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
