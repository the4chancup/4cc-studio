//! The Save editor's message catalog: each code the commands report, its severity, and the
//! `Message` it is sent as. Every finding is run-wide (`Scope::Run`) and keeps what it is
//! about (`Disposition::Keep`): a command works on one save, not on exports, and a finding
//! that drops something says so with its own code. Each `CATALOG` row also carries the
//! member-facing `when`/`effect` the help's generated Messages topic shows (`spec/save_editor.md`
//! "Messages"); the console prints the code with its context fields.

use std::borrow::Cow;

use aatf::{Violation, ViolationKind};
use pes_savefile::interchange::aesthetics_patch::PatchSkip;
use pes_savefile::interchange::team_toml::ImportNote;
use pes_savefile::model::PlayerEntry;
use pes_savefile::model::names::display_name;
use pes_savefile::model::roster_order_player_id;
use pes_savefile::ops::compare::PlayerDiff;
use studio_core::{CatalogRow, Disposition, Message, MessageCode, Scope, Severity};

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
    /// `import-toml`: a name or shirt name longer than the save's field; cropped to it
    /// (`path`, `cropped`).
    ImportTextCropped,
    /// `apply-patch`: the save holds the team and at least one of its patch players was
    /// written (`team`). A team whose players the save all lacks is not listed.
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
    /// `check-aatf`: one rule the team breaks under the ruleset (`team`; `player` for a
    /// player's violation; `rule`, then the rule's own fields). The severity is the
    /// violation's, so `aatf_violation` sets it on the message it builds.
    AatfViolation,
    /// `check-aatf`: the ruleset's check itself failed on the team, which the other teams'
    /// results do not cover (`team`, `error`).
    AatfCheckFailed,
    /// `check-stripped`: the player's appearance id is set again while another player's is
    /// stripped, so the game shows his save record, not the database rows; the member runs
    /// `check-stripped --strip-again` to strip him again (`player`, `name`). Error while the
    /// id stands, Info once `--strip-again` has stripped him.
    AppearanceIdRestored,
    /// `strip-and-seed`: the player's record copied another player's face, which the seed
    /// rows cannot carry, so once stripped he shows the face his row describes; his export
    /// describing his face settles it (`player`, `name`, `copied`).
    StripBaseCopiedFace,
    /// The Appearance tab's FPC group: the player wears inners under a hidden body, which
    /// breaks the hiding.
    FpcInnersBreakHiding,
    /// The Appearance tab's FPC group: the player wears undershorts under a hidden body,
    /// which breaks the hiding.
    FpcUndershortsBreakHiding,
    /// The Appearance tab's FPC group: the player wears wrist or ankle taping under a
    /// hidden body, so the taped pieces still show.
    FpcTapingShowsPieces,
    /// The Appearance tab's FPC group: the gloves checkbox is on with gloves ID 0, so the
    /// player wears winter gloves in winter conditions.
    FpcGlovesCheckboxWinterGloves,
    /// The Appearance tab's FPC group: a custom skin on a player whose body is not hidden
    /// hides the body but not the jersey.
    FpcCustomSkinOnNonFpc,
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
            Code::ImportTextCropped => "import_text_cropped",
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
            Code::AatfViolation => "aatf_violation",
            Code::AatfCheckFailed => "aatf_check_failed",
            Code::AppearanceIdRestored => "appearance_id_restored",
            Code::StripBaseCopiedFace => "strip_base_copied_face",
            Code::FpcInnersBreakHiding => "fpc_inners_break_hiding",
            Code::FpcUndershortsBreakHiding => "fpc_undershorts_break_hiding",
            Code::FpcTapingShowsPieces => "fpc_taping_shows_pieces",
            Code::FpcGlovesCheckboxWinterGloves => "fpc_gloves_checkbox_winter_gloves",
            Code::FpcCustomSkinOnNonFpc => "fpc_custom_skin_on_non_fpc",
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
            | Code::TeamsListNotWritten
            | Code::FpcTapingShowsPieces => Severity::Info,
            // A value the save did not take as written, or a record the game does not read:
            // worth knowing, not a failure (the plan's cross-version import converts "with a
            // warning list").
            Code::ImportKeyNotInVersion
            | Code::ImportValueNotEncodable
            | Code::ImportValueCapped
            | Code::ImportSlotEmpty
            | Code::ImportTextCropped
            | Code::TransplantTargetStripped
            | Code::TransplantDonorStripped
            | Code::StripBaseCopiedFace
            | Code::FpcGlovesCheckboxWinterGloves
            | Code::FpcCustomSkinOnNonFpc => Severity::Warning,
            // A skipped team or player is an error and the rest is written (the plan's
            // `apply-patch`: exit 1); so is a dropped teams-list change and a list that
            // cannot be written (`export-teams-list`: exit 1).
            Code::PatchTeamNotInSave
            | Code::PatchPlayerNotInSave
            | Code::TeamsListUnresolved
            | Code::TeamsListNameDuplicate
            | Code::TeamsListReadOnly
            | Code::FpcInnersBreakHiding
            | Code::FpcUndershortsBreakHiding => Severity::Error,
            // `check-aatf`: a failed check is an error; a violation's own severity decides,
            // and `aatf_violation` overwrites this default with it. `check-stripped`: a
            // listed player is an error (`check-stripped`: exit 1);
            // `appearance_id_restored` overwrites this to Info once `--strip-again` has
            // stripped him.
            Code::AatfViolation | Code::AatfCheckFailed | Code::AppearanceIdRestored => {
                Severity::Error
            }
        }
    }

    /// Every code, for the catalog test, in the enum's (and `CATALOG`'s) order.
    #[cfg(test)]
    const ALL: &'static [Code] = &[
        Code::TeamExported,
        Code::ImportKeyNotInVersion,
        Code::ImportValueNotEncodable,
        Code::ImportValueCapped,
        Code::ImportSlotEmpty,
        Code::ImportTextCropped,
        Code::PatchTeamApplied,
        Code::PatchTeamNotInSave,
        Code::PatchPlayerNotInSave,
        Code::TransplantTargetStripped,
        Code::TransplantDonorStripped,
        Code::PlayerDiff,
        Code::PlayerOnlyIn,
        Code::PlayerStripped,
        Code::TeamsListAdded,
        Code::TeamsListOverridden,
        Code::TeamsListUnresolved,
        Code::TeamsListNameDuplicate,
        Code::TeamsListKept,
        Code::TeamsListWritten,
        Code::TeamsListNotWritten,
        Code::TeamsListReadOnly,
        Code::AatfViolation,
        Code::AatfCheckFailed,
        Code::AppearanceIdRestored,
        Code::StripBaseCopiedFace,
        Code::FpcInnersBreakHiding,
        Code::FpcUndershortsBreakHiding,
        Code::FpcTapingShowsPieces,
        Code::FpcGlovesCheckboxWinterGloves,
        Code::FpcCustomSkinOnNonFpc,
    ];
}

/// Every code this tool emits, in the enum's (and the spec table's) order, with the
/// member-facing `when`/`effect` the help's generated Messages topic shows.
const CATALOG: &[(Code, &str, &str)] = &[
    (
        Code::TeamExported,
        r#"`export-toml` wrote one team's file."#,
        r#"Nothing to do."#,
    ),
    (
        Code::ImportKeyNotInVersion,
        r#"`import-toml`: a key the document's version stores and the save's does not."#,
        r#"The key is dropped from the import; the rest is written."#,
    ),
    (
        Code::ImportValueNotEncodable,
        r#"`import-toml`: a value the save's version cannot store (an instruction it lacks)."#,
        r#"Written as off; the rest is written."#,
    ),
    (
        Code::ImportValueCapped,
        r#"`import-toml`: a face type above the save's version's cap, or a custom skin the version has no slot for."#,
        r#"Written at the lower value (the face type as 0, the skin as 1); the rest is written."#,
    ),
    (
        Code::ImportSlotEmpty,
        r#"`import-toml`: a `[players.NN]` whose roster slot is empty."#,
        r#"The player is skipped; the rest is written."#,
    ),
    (
        Code::ImportTextCropped,
        r#"`import-toml`: a name or shirt name longer than the save's field; the field holds fewer letters there."#,
        r#"Cut to the text the line shows; the rest is written."#,
    ),
    (
        Code::PatchTeamApplied,
        r#"`apply-patch`: the save holds the team and at least one of its patch players was written; a team whose players the save all lacks is not listed."#,
        r#"Nothing to do."#,
    ),
    (
        Code::PatchTeamNotInSave,
        r#"`apply-patch`: the save has no team with this id; its players were not looked at."#,
        r#"The rest is written; exit 1."#,
    ),
    (
        Code::PatchPlayerNotInSave,
        r#"`apply-patch`: the save has no player with this id."#,
        r#"The rest is written; exit 1."#,
    ),
    (
        Code::TransplantTargetStripped,
        r#"`transplant`: the target's appearance id is -1, so the game does not read the record the donor's fields went into; the seeded tables decide his look."#,
        r#"The write stands; check him in the game."#,
    ),
    (
        Code::TransplantDonorStripped,
        r#"`transplant`: the donor's appearance id is -1, so the fields copied are his record's, not the look the game shows for him."#,
        r#"The write stands; the targets get the record, not the look."#,
    ),
    (
        Code::PlayerDiff,
        r#"`diff`: one field of a player in both saves, old to new."#,
        r#"A report row; differences are the output, exit 0."#,
    ),
    (
        Code::PlayerOnlyIn,
        r#"`diff`: a player in one save alone (the line says which)."#,
        r#"A report row; nothing to do."#,
    ),
    (
        Code::PlayerStripped,
        r#"`diff`: the player's appearance id is -1 in the save the line names; his record is compared, not the look the game shows."#,
        r#"Marked beside his rows; exit 0."#,
    ),
    (
        Code::TeamsListAdded,
        r#"`export-teams-list`: a team of the save the list lacked, appended or put into a placeholder's slot."#,
        r#"Merged; written only with `--yes`."#,
    ),
    (
        Code::TeamsListOverridden,
        r#"`export-teams-list`: a name in both whose id becomes the save's."#,
        r#"Merged; written only with `--yes`."#,
    ),
    (
        Code::TeamsListUnresolved,
        r#"`export-teams-list`: a change dropped because its id collided with another row of the merged list."#,
        r#"The change is dropped, the other changes written with `--yes`; exit 1."#,
    ),
    (
        Code::TeamsListNameDuplicate,
        r#"`export-teams-list`: a team of the save left out because an earlier team folds to the same name, which the list can hold once."#,
        r#"The team is left out; exit 1. Rename one of them in the save to keep both."#,
    ),
    (
        Code::TeamsListKept,
        r#"`export-teams-list`: the rows that ended unchanged (the line gives the count)."#,
        r#"Nothing to do."#,
    ),
    (
        Code::TeamsListWritten,
        r#"`export-teams-list`: the merged list was written."#,
        r#"Nothing to do."#,
    ),
    (
        Code::TeamsListNotWritten,
        r#"`export-teams-list` without `--yes`: the summary only, the file untouched."#,
        r#"Nothing is written; run it again with `--yes` to write it."#,
    ),
    (
        Code::TeamsListReadOnly,
        r#"`export-teams-list --yes`: the data directory cannot take the file."#,
        r#"Nothing is written; exit 1."#,
    ),
    (
        Code::AatfViolation,
        r#"`check-aatf`: one rule the team breaks under the ruleset (the line names the team, the player for a player's violation, the rule and its values)."#,
        r#"An error-severity violation makes the exit code 1; a warning alone leaves it 0."#,
    ),
    (
        Code::AatfCheckFailed,
        r#"`check-aatf`: the ruleset's check itself failed on the team, which the other teams' results do not cover."#,
        r#"The other teams are still checked; exit 1."#,
    ),
    (
        Code::AppearanceIdRestored,
        r#"`check-stripped`: the player's appearance id is set again while another player's is stripped, so the game shows his save record, not the database rows."#,
        r#"An Error while he stands listed (exit 1); reported at Info and exit 0 once `--strip-again` has stripped him."#,
    ),
    (
        Code::StripBaseCopiedFace,
        r#"`strip-and-seed`: the player's record copied another player's face, which the seed rows cannot carry, so once stripped he shows the face his row describes."#,
        r#"Nothing to do; compiling his export's face settles it."#,
    ),
    (
        Code::FpcInnersBreakHiding,
        r#"The player wears inners under a hidden body, which breaks the hiding."#,
        r#"Nothing is changed for you; take the inners off, or the body will not hide."#,
    ),
    (
        Code::FpcUndershortsBreakHiding,
        r#"The player wears undershorts under a hidden body, which breaks the hiding."#,
        r#"Nothing is changed for you; take the undershorts off, or the body will not hide."#,
    ),
    (
        Code::FpcTapingShowsPieces,
        r#"The player wears wrist or ankle taping under a hidden body: the taped pieces still show."#,
        r#"Deliberate in custom setups; nothing to do."#,
    ),
    (
        Code::FpcGlovesCheckboxWinterGloves,
        r#"The gloves checkbox is on with gloves ID 0."#,
        r#"Nothing is changed for you; the player wears winter gloves in winter conditions. Untick the checkbox or give him a gloves ID if that is not wanted."#,
    ),
    (
        Code::FpcCustomSkinOnNonFpc,
        r#"A custom skin on a player whose body is not hidden by the FPC hide boots ID from Settings hides the body but not the jersey."#,
        r#"Nothing is changed for you; hide the body too, or pick a preset skin."#,
    ),
];

/// The catalog rows the help's generated Messages topic is rendered from, in `CATALOG`
/// order (`lib.rs` `help`).
pub(crate) fn catalog() -> Vec<CatalogRow> {
    CATALOG
        .iter()
        .map(|(code, when, effect)| CatalogRow {
            code: code.as_str(),
            severity: catalog_severity(*code),
            when,
            effect,
        })
        .collect()
}

/// The severity the Messages topic spells out: the code's own, the two codes whose
/// `Message` overwrites it spelled as their spec rows do.
fn catalog_severity(code: Code) -> &'static str {
    match code {
        Code::AatfViolation => "Error or Warning",
        Code::AppearanceIdRestored => "Error or Info",
        _ => code.severity().name(),
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
        ImportNote::TextCropped { path, cropped } => message(
            Code::ImportTextCropped,
            // As the game draws it: a colour code's bytes are no text a member reads.
            vec![("path", path.clone()), ("cropped", display_name(cropped))],
        ),
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

/// The message for one `check-aatf` violation of `team`: `player` is the rostered player's
/// id when the violation names a slot, `severity` the violation's (a suggestion is filtered
/// by the caller, which prints nothing for it), and `rule` with its fields is
/// `aatf::ViolationKind::rule`'s. A `RosterOrder` names the held id in its own `player`
/// field (the context's would repeat it) and adds `expected`, the id the rule puts in the
/// slot.
pub(crate) fn aatf_violation(
    team: u32,
    player: Option<u32>,
    violation: &Violation,
    severity: Severity,
) -> Message {
    let (rule, fields) = violation.kind.rule();
    let mut context = vec![("team", team.to_string())];
    if let Some(player) = player
        && !matches!(violation.kind, ViolationKind::RosterOrder { .. })
    {
        context.push(("player", player.to_string()));
    }
    context.push(("rule", rule.to_owned()));
    context.extend(fields);
    if let (ViolationKind::RosterOrder { .. }, Some(slot)) = (&violation.kind, violation.slot) {
        context.push(("expected", roster_order_player_id(team, slot).to_string()));
    }
    let mut message = message(Code::AatfViolation, context);
    message.severity = severity;
    message
}

/// The message for a team whose `check-aatf` run failed (a bug in the logic or in a custom
/// check): `error` is the `TeamViewError`'s or `CheckError`'s text.
pub(crate) fn aatf_check_failed(team: u32, error: String) -> Message {
    message(
        Code::AatfCheckFailed,
        vec![("team", team.to_string()), ("error", error)],
    )
}

/// The message for one player `check-stripped` lists: his appearance id came back while
/// other players stay stripped, so the game shows his save record, not the database rows.
/// `severity` is Error while he stands listed (exit 1), Info once `--strip-again` has
/// stripped him again.
pub(crate) fn appearance_id_restored(player: &PlayerEntry, severity: Severity) -> Message {
    let mut message = message(
        Code::AppearanceIdRestored,
        vec![
            ("player", player.id.to_string()),
            ("name", player.name.clone()),
        ],
    );
    message.severity = severity;
    message
}

/// The message for one player `strip-and-seed` stripped whose record copied player
/// `copied`'s face and hair: the seed rows hold no base-copy id, so he loses the copied
/// face and shows the one his row describes. His export describing his face settles it:
/// the compiled rows replace his seed.
pub(crate) fn strip_base_copied_face(player: &PlayerEntry, copied: u32) -> Message {
    message(
        Code::StripBaseCopiedFace,
        vec![
            ("player", player.id.to_string()),
            ("name", player.name.clone()),
            ("copied", copied.to_string()),
        ],
    )
}

/// The message for one `fpc::check` finding of the Appearance tab's FPC group: the code
/// it maps to and the finding's own severity. A code `fpc::check` does not emit is a
/// programming error and panics naming it.
pub(crate) fn fpc_finding(finding: &fpc::Finding) -> Message {
    let code = match finding.code {
        "fpc_inners_break_hiding" => Code::FpcInnersBreakHiding,
        "fpc_undershorts_break_hiding" => Code::FpcUndershortsBreakHiding,
        "fpc_taping_shows_pieces" => Code::FpcTapingShowsPieces,
        "fpc_gloves_checkbox_winter_gloves" => Code::FpcGlovesCheckboxWinterGloves,
        "fpc_custom_skin_on_non_fpc" => Code::FpcCustomSkinOnNonFpc,
        other => panic!("fpc::check emits its five codes, got {other}"),
    };
    let mut message = message(code, vec![]);
    message.severity = match finding.severity {
        fpc::Severity::Info => Severity::Info,
        fpc::Severity::Warning => Severity::Warning,
        fpc::Severity::Error => Severity::Error,
    };
    message
}

/// The `when` text `CATALOG` spells for `message`'s code: the sentence a view draws
/// for the finding (the Appearance tab's FPC findings).
pub(crate) fn when_text(message: &Message) -> &'static str {
    CATALOG
        .iter()
        .find(|(code, ..)| code.as_str() == message.code.code)
        .map(|(_, when, _)| *when)
        .expect("every emitted code has a catalog row")
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
    use aatf::ViolationKind;

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
        assert_eq!(
            line(&import_note(&ImportNote::TextCropped {
                path: "players.02.shirt_name".to_owned(),
                cropped: "ABCDEFGHIJKLMNOP".to_owned()
            })),
            "Warning import_text_cropped (path=players.02.shirt_name, cropped=ABCDEFGHIJKLMNOP)"
        );
        // A coloured name is shown as the game draws it, its codes left out.
        assert_eq!(
            line(&import_note(&ImportNote::TextCropped {
                path: "players.03.name".to_owned(),
                cropped: "\x11cFF0000FFAKI\x11dRA".to_owned()
            })),
            "Warning import_text_cropped (path=players.03.name, cropped=AKIRA)"
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

    /// One variant of `ViolationKind` each, and the `rule=` line's fields as the rules
    /// file spells them: the map keys `logic.rhai` emits for the kind, in emit order.
    #[test]
    fn every_violation_kind_prints_its_rules_file_fields() {
        let cases: &[(ViolationKind, &str)] = &[
            (ViolationKind::RegisteredNotA, "rule=RegisteredNotA"),
            (
                ViolationKind::FieldedNotA {
                    preset: 1,
                    position: "GK".to_owned().try_into().unwrap(),
                },
                "rule=FieldedNotA, preset=1, position=GK",
            ),
            (ViolationKind::BRating, "rule=BRating"),
            (ViolationKind::GkSecondA, "rule=GkSecondA"),
            (
                ViolationKind::Age {
                    age: 15,
                    min: 16,
                    max: 40,
                },
                "rule=Age, age=15, min=16, max=40",
            ),
            (
                ViolationKind::Weight {
                    weight: 80,
                    min: 30,
                    max: 50,
                },
                "rule=Weight, weight=80, min=30, max=50",
            ),
            (ViolationKind::PositionRange, "rule=PositionRange"),
            (ViolationKind::PlayingStyleRange, "rule=PlayingStyleRange"),
            (ViolationKind::NoTier, "rule=NoTier"),
            (
                ViolationKind::Stat {
                    stat: "dribbling".to_owned().try_into().unwrap(),
                    value: 80,
                    target: 92,
                    at_most: true,
                },
                "rule=Stat, stat=dribbling, value=80, target=92, at_most=true",
            ),
            (
                ViolationKind::StatTotal {
                    total: 1772,
                    limit: 1771,
                },
                "rule=StatTotal, total=1772, limit=1771",
            ),
            (
                ViolationKind::Form {
                    value: 2,
                    target: 4,
                },
                "rule=Form, value=2, target=4",
            ),
            (
                ViolationKind::InjuryResistance { value: 0, max: 2 },
                "rule=InjuryResistance, value=0, max=2",
            ),
            (
                ViolationKind::WeakFootUsage { value: 2, max: 4 },
                "rule=WeakFootUsage, value=2, max=4",
            ),
            (
                ViolationKind::WeakFootAccuracy { value: 1, max: 3 },
                "rule=WeakFootAccuracy, value=1, max=3",
            ),
            (
                ViolationKind::NameColor {
                    color: Some("c0c0c0ff".to_owned()),
                    expected: Some("cc9900ff".to_owned()),
                },
                "rule=NameColor, color=c0c0c0ff, expected=cc9900ff",
            ),
            (
                ViolationKind::Cards { paid: 10, limit: 8 },
                "rule=Cards, paid=10, limit=8",
            ),
            (
                ViolationKind::MinCards { paid: 3, min: 5 },
                "rule=MinCards, paid=3, min=5",
            ),
            (
                ViolationKind::SkillCardCap { count: 11, max: 10 },
                "rule=SkillCardCap, count=11, max=10",
            ),
            (
                ViolationKind::ComStyleCap { count: 8, max: 7 },
                "rule=ComStyleCap, count=8, max=7",
            ),
            (
                ViolationKind::RequiredCard {
                    card: "malicia".to_owned().try_into().unwrap(),
                },
                "rule=RequiredCard, card=malicia",
            ),
            (
                ViolationKind::APositions { count: 3, max: 2 },
                "rule=APositions, count=3, max=2",
            ),
            (
                ViolationKind::Height { height: 191 },
                "rule=Height, height=191",
            ),
            (
                ViolationKind::TierCount {
                    tier: "gold".to_owned(),
                    count: 0,
                    expected: 1,
                },
                "rule=TierCount, tier=gold, count=0, expected=1",
            ),
            (
                ViolationKind::BracketQuota {
                    system: "red".to_owned(),
                    bracket: "tall".to_owned(),
                    count: 3,
                    quota: 6,
                },
                "rule=BracketQuota, system=red, bracket=tall, count=3, quota=6",
            ),
            (
                ViolationKind::MedalSystem {
                    systems: vec!["GGG".to_owned(), "GGSS".to_owned()],
                },
                "rule=MedalSystem, systems=[GGG, GGSS]",
            ),
            (
                ViolationKind::ExtraHeight {
                    used: 51,
                    total: 50,
                },
                "rule=ExtraHeight, used=51, total=50",
            ),
            (ViolationKind::Captain, "rule=Captain"),
            (
                ViolationKind::Goalkeepers { count: 0, min: 1 },
                "rule=Goalkeepers, count=0, min=1",
            ),
            (
                ViolationKind::TeamSetting {
                    label: "no offside trap".to_owned(),
                    setting: "offside_trap".to_owned(),
                },
                "rule=TeamSetting, label=no offside trap, setting=offside_trap",
            ),
            (
                ViolationKind::FormationShape {
                    preset: 2,
                    formation: 3,
                    shape: [3, 4, 3],
                },
                "rule=FormationShape, preset=2, formation=3, shape=[3, 4, 3]",
            ),
            (
                ViolationKind::Special {
                    label: "GKs below giant".to_owned(),
                    text: "GK heights cannot be 194cm or more".to_owned(),
                },
                "rule=Special, label=GKs below giant, text=GK heights cannot be 194cm or more",
            ),
            (
                ViolationKind::Custom {
                    text: "a custom check's text".to_owned(),
                },
                "rule=Custom, text=a custom check's text",
            ),
            (
                ViolationKind::Unused {
                    allowance: "cards".to_owned(),
                    used: 3,
                    allowed: 8,
                },
                "rule=Unused, allowance=cards, used=3, allowed=8",
            ),
            (
                ViolationKind::UnusedChoice {
                    special: "manlet bonus".to_owned(),
                },
                "rule=UnusedChoice, special=manlet bonus",
            ),
        ];
        for (kind, expected) in cases {
            let violation = Violation {
                slot: Some(0),
                severity: aatf::Severity::Error,
                kind: kind.clone(),
            };
            assert_eq!(
                line(&aatf_violation(
                    718,
                    Some(71802),
                    &violation,
                    Severity::Error
                )),
                format!("Error aatf_violation (team=718, player=71802, {expected})"),
                "{kind:?}"
            );
        }
        // `RosterOrder` prints its held id as `player` (the context's own
        // would repeat it) and `expected`, the id the rule puts in the slot
        // (slot 0 of team 718: 71801).
        let roster_order = Violation {
            slot: Some(0),
            severity: aatf::Severity::Error,
            kind: ViolationKind::RosterOrder { player: 71802 },
        };
        assert_eq!(
            line(&aatf_violation(
                718,
                Some(71802),
                &roster_order,
                Severity::Error
            )),
            "Error aatf_violation (team=718, rule=RosterOrder, player=71802, expected=71801)"
        );
        // A team-level violation names no player, an `Option` field none when `None`, and
        // the severity is the violation's.
        let name_color = Violation {
            slot: None,
            severity: aatf::Severity::Warning,
            kind: ViolationKind::NameColor {
                color: None,
                expected: Some("cc9900ff".to_owned()),
            },
        };
        assert_eq!(
            line(&aatf_violation(718, None, &name_color, Severity::Warning)),
            "Warning aatf_violation (team=718, rule=NameColor, expected=cc9900ff)"
        );
        assert_eq!(
            line(&aatf_check_failed(
                701,
                "the check's script failed: x".to_owned()
            )),
            "Error aatf_check_failed (team=701, error=the check's script failed: x)"
        );
    }

    #[test]
    fn every_code_has_exactly_one_catalog_row_and_every_row_its_text() {
        for code in Code::ALL {
            let rows = CATALOG.iter().filter(|(known, ..)| known == code).count();
            assert_eq!(rows, 1, "{}", code.as_str());
        }
        for (index, (code, ..)) in CATALOG.iter().enumerate() {
            assert!(
                CATALOG[index + 1..].iter().all(|(other, ..)| other != code),
                "{} appears twice",
                code.as_str()
            );
        }
        for (code, when, effect) in CATALOG {
            assert!(!when.is_empty() && !effect.is_empty(), "{}", code.as_str());
        }
    }

    #[test]
    fn the_help_chapter_lists_the_whole_catalog_in_order() {
        use studio_core::StudioTool;

        let help = crate::Tool::new().help();
        let codes: Vec<&str> = help.messages.iter().map(|row| row.code).collect();
        let catalog: Vec<&str> = CATALOG.iter().map(|(code, ..)| code.as_str()).collect();
        assert_eq!(codes, catalog);
    }

    #[test]
    fn a_rows_text_is_written_for_a_member() {
        let row = CATALOG
            .iter()
            .find(|(code, ..)| *code == Code::TeamExported)
            .unwrap();
        assert_eq!(row.1, "`export-toml` wrote one team's file.");
        assert_eq!(row.2, "Nothing to do.");
        // The two codes whose `Message` overwrites the severity, spelled out as the
        // Messages topic shows them.
        assert_eq!(catalog_severity(Code::AatfViolation), "Error or Warning");
        assert_eq!(
            catalog_severity(Code::AppearanceIdRestored),
            "Error or Info"
        );
    }
}
