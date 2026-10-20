//! The Export upgrader's message catalog (`spec/export_upgrader.md` "Message
//! catalog"): each code the command reports, its severity, and the `Message` it
//! is sent as. The report names two exports: the input (export 1) for what is
//! found in it, the output (export 2) for the shared validation's findings on
//! what was written. Each row also carries the member-facing `when`/`effect`
//! the help's generated Messages topic shows; the console prints the code with
//! its context fields.

use std::borrow::Cow;

use aesthetics_export::{IssueScope, ValidationIssue};
use studio_core::{CatalogRow, Disposition, ExportId, Message, MessageCode, Scope, Severity};

use crate::source::SourceFailure;

/// The tool's id: its CLI subcommand and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "export-upgrader";

/// The input export's id in this run's events.
pub(crate) const INPUT: ExportId = ExportId(1);
/// The output export's.
pub(crate) const OUTPUT: ExportId = ExportId(2);

/// The codes the tool reports itself. `team_name_unknown` and
/// `source_read_failed` are shared with the structure pass's `ISSUE_CODES`
/// (the plan's catalog takes the compiler's codes), so they live in
/// `CATALOG`'s issue rows and are emitted by string.
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
    /// The last line otherwise; nothing is written ("Verification").
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
/// Error (`spec/export_upgrader.md` "Message catalog").
const CATALOG: &[(&str, Severity, &str, &str)] = &[
    // The tool's own codes.
    (
        "no_old_export",
        Severity::Error,
        r#"step 1: the input holds nothing an old export holds"#,
        r#"nothing is written"#,
    ),
    (
        "refs_txt_missing",
        Severity::Error,
        r#"a `refs` export without a `refs.txt` holds no roster to migrate (step 9)"#,
        r#"nothing is written"#,
    ),
    (
        "export_extract_failed",
        Severity::Error,
        r#"the archive does not open or its listing is refused (the compiler's code)"#,
        r#"nothing is written"#,
    ),
    (
        "unmigrated_content",
        Severity::Error,
        r#"an entry inside the old format's folders that reached the game and that no step migrates, or that step 10's boundary keeps an Error (two candidates for one place, a misnamed whole folder), entry by entry; a Note entry whose colors do not read is one, its line in the context, and so is a kit config that does not decode (step 5), its error in the context"#,
        r#"the entry is not migrated; the run is not clean, so nothing is written"#,
    ),
    (
        "migration_clean",
        Severity::Info,
        r#"last line: no Error finding, and no finding drops anything but a leftover ("Verification"'s clean migrated export)"#,
        r#"exit 0"#,
    ),
    (
        "migration_not_clean",
        Severity::Info,
        r#"last line otherwise"#,
        r#"nothing is written ("Verification"); exit 1 after any finding that drops anything but a leftover"#,
    ),
    (
        "leftover_dropped",
        Severity::Warning,
        r#"step 10: content that never reached a player in game, outside the old format's layout or inside it (a refused portrait, a stray file, a second goalkeeper kit, ...); a Note entry's dropped part names its line"#,
        r#"not written; the run stays clean"#,
    ),
    (
        "mtl_stray_text_dropped",
        Severity::Info,
        r#"step 7: an old `.mtl` held text between its elements"#,
        r#"dropped as the file is written back"#,
    ),
    (
        "roster_slot_empty",
        Severity::Warning,
        r#"a face folder whose slot the save's roster leaves empty (`slot`)"#,
        r#"it still becomes its player folder, with no `settings.toml`"#,
    ),
    (
        "merge_kept_shared",
        Severity::Info,
        r#"a path a player folder and a merging item folder share with different bytes (`path`, the first clashing path relative to the player folder)"#,
        r#"the item folder is kept shared with a link instead"#,
    ),
    (
        "wearer_assumed",
        Severity::Info,
        r#"loose mode's assumed wearer of an item folder (step 11): the ID's position in its kind's old block of 25 (`id`, the folder's four-digit ID; `slot`, the player's)"#,
        r#"reported so the pairing can be checked"#,
    ),
    (
        "kit_without_config",
        Severity::Warning,
        r#"a kit slot a Note kit entry gives and no `Kit Configs` config does (`slot`)"#,
        r#"the kit still forms, a placeholder the compiler fills"#,
    ),
    (
        "config_without_kit_entry",
        Severity::Warning,
        r#"a kit slot a `Kit Configs` config gives and no Note kit entry does (`slot`)"#,
        r#"its `config.toml` is written all the same, the kit offered as the old export never offered it"#,
    ),
    (
        "kit_texture_copied",
        Severity::Info,
        r#"a kit's config field named another texture (`kit`, `stem` the field's, `from` the source's slot and stem)"#,
        r#"the named texture was copied into the kit under the field's generic name"#,
    ),
    (
        "kit_texture_hoisted",
        Severity::Info,
        r#"a texture every kit folder held byte-identically (`stem`, the first kit's spelling)"#,
        r#"it moved to `Kits/all/`"#,
    ),
    (
        "portrait_conflict",
        Severity::Error,
        r#"a `Portraits/` file and its slot's face-folder portrait differ in bytes, which the compiler's deep pass would refuse (`slot`)"#,
        r#"the run is not clean, so nothing is written"#,
    ),
    (
        "logo_small_carried",
        Severity::Info,
        r#"the old `_r` small logo is a different drawing (`share`, the differing pixels' percentage)"#,
        r#"kept as `logo_small.png`"#,
    ),
    (
        "logo_small_dropped",
        Severity::Info,
        r#"the old `_r` small logo is a downscale of the large one (`share`)"#,
        r#"dropped: the compiler regenerates it"#,
    ),
    (
        "logo_fallback",
        Severity::Warning,
        r#"no usable `_r_ll` member"#,
        r#"the largest usable one became `logo.png`"#,
    ),
    (
        "common_reference_kept",
        Severity::Warning,
        r#"an `.mtl`/`.fmdl` reference into the team's `Common/` whose file is not in the input's `Common/`, where a link would clash (the folder already holds a texture of that stem, which is the one the model will wear: the compiler looks a game path's file name up in the folder first), or from a subfolder `.mtl` naming a Common subfolder (`reference`, the path as written)"#,
        r#"the reference is kept as written, its `???` respelled the team's ID"#,
    ),
    (
        "custom_model_missing",
        Severity::Error,
        r#"a rostered player's save record wears a custom boots or gloves ID (above 100) no migrated folder claims (step 8; `slot`, `name`, `key` the ID's own, `id`); checked for every rostered player of a Full output but in a Midcup one only for a player the output gives a folder"#,
        r#"the content is missing: the run is not clean, so nothing is written"#,
    ),
    (
        "settings_value_refused",
        Severity::Warning,
        r#"a stored value no `settings.toml` key can hold (`slot`, `name`, `key` the dotted key, `value` the stored value)"#,
        r#"left out of the player's file"#,
    ),
    (
        "settings_from_stripped_record",
        Severity::Info,
        r#"a `settings.toml` generated from a stripped record (`slot`, `name`)"#,
        r#"the record still is his look until the team's first Studio compile, which keeps its own file instead"#,
    ),
    (
        "settings_not_generated",
        Severity::Error,
        r#"a player's `settings.toml` could not be generated at all (`slot`, `name`, `error` the failure's text)"#,
        r#"the run is not clean, so nothing is written"#,
    ),
    (
        "model_folder_destination",
        Severity::Info,
        r#"model-folder mode: where an output folder goes once dropped into a Studio export (`into`: `Players`, `Boots`, `Gloves`, or `root` for `Common/`)"#,
        r#"informational; goes on"#,
    ),
    // The structure pass's codes, in `ISSUE_CODES` order.
    (
        "nested_folders_fixed",
        Severity::Warning,
        r#"content found nested one level down (exactly one usable root)"#,
        r#"auto-fixed"#,
    ),
    (
        "nested_root_ambiguous",
        Severity::Error,
        r#"several nested child folders are usable export roots"#,
        r#"export skipped (`DropExport`)"#,
    ),
    (
        "nested_root_conflict",
        Severity::Error,
        r#"loose root file collides with a flattened nested file at the same virtual path"#,
        r#"export skipped (`DropExport`)"#,
    ),
    (
        "players_txt_invalid",
        Severity::Error,
        r#"file-level encoding failure or roster-wide ambiguity prevents trustworthy normalization, or a referee roster retains zero valid assignments"#,
        r#"export skipped (`DropExport`; foundational). An empty normal-team roster remains valid for a kit-only export"#,
    ),
    (
        "refs_txt_ignored",
        Severity::Warning,
        r#"referee export has both `players.txt` and the legacy `refs.txt` alias"#,
        r#"`players.txt` used, alias ignored"#,
    ),
    (
        "source_read_failed",
        Severity::Error,
        r#"a file of the input does not read after its listing did (the compiler's code; `path`, `error`)"#,
        r#"it is not migrated and the run reports on and ends not clean, so nothing is written; an unreadable Note (it decides the team) or a referee export's `refs.txt` (it decides the roster) stops the run at once"#,
    ),
    (
        "export_empty",
        Severity::Error,
        r#"no usable content found at root"#,
        r#"export skipped"#,
    ),
    (
        "team_name_unknown",
        Severity::Error,
        r#"the team resolves to no row of the list (the compiler's code), the `Team:` line's file or the export named (`name`)"#,
        r#"nothing is written"#,
    ),
    (
        "export_tag_missing",
        Severity::Error,
        r#"a team export's name has neither `Full` nor `Midcup` as its second word (context `name`: the export's name)"#,
        r#"export skipped; the hint says to rename it `<team> Full …` or `<team> Midcup …`"#,
    ),
    (
        "export_layout_old",
        Severity::Error,
        r#"the export is in the old layout, which the compiler does not read: a root folder named `Kit Configs`, `Kit Textures` or `Other`, names the Studio format never has (its kits live in `Kits/`); context `folder`: the first such folder. Reported alone: the export's other structure findings are not computed, since every one of them would describe the old layout again"#,
        r#"export skipped; the hint says to run the export through the Export upgrader once"#,
    ),
    (
        "players_txt_missing",
        Severity::Error,
        r#"referee export has no required root `players.txt` (nor a legacy `refs.txt` alias)"#,
        r#"export skipped (`DropExport`)"#,
    ),
    (
        "players_txt_line_invalid",
        Severity::Error,
        r#"one nonblank `players.txt` line cannot be parsed into decimal slot + folder name"#,
        r#"that `RosterEntry` skipped (`DropSlot`)"#,
    ),
    (
        "players_txt_slot_invalid",
        Severity::Error,
        r#"a `players.txt` line's decimal slot is outside 01–23 (teams) / 01–35 (refs)"#,
        r#"that `RosterEntry` assignment skipped (`DropSlot`)"#,
    ),
    (
        "players_txt_slot_duplicate",
        Severity::Error,
        r#"authoritative `players.txt` assigns the same slot more than once, making roster identity ambiguous"#,
        r#"export skipped (`DropExport`)"#,
    ),
    (
        "players_txt_target_missing",
        Severity::Error,
        r#"`players.txt` entry names a nonexistent folder"#,
        r#"`RosterEntry` scoped assignment skipped (`DropSlot`)"#,
    ),
    (
        "player_unlisted",
        Severity::Warning,
        r#"authoritative `players.txt` is present but does not list this player folder, regardless of any number in its name"#,
        r#"folder ignored"#,
    ),
    (
        "player_folder_number_invalid",
        Severity::Error,
        r#"without `players.txt`, a team player folder's number is not in 01–23"#,
        r#"folder discarded (`DropFolder`)"#,
    ),
    (
        "player_number_duplicate",
        Severity::Error,
        r#"without `players.txt`, two team folder names claim the same player slot"#,
        r#"both conflicting folders discarded (`DropFolder` on each folder)"#,
    ),
    (
        "file_type_disallowed",
        Severity::Error,
        r#"a file in a player or shared folder's tree, in a kit folder or in `Collars/` that its position's allowlist row does not admit (`spec/aesthetics_export/object_model.md` "File-type allowlist": a disallowed extension, or a marker, `settings.toml` or per-player singleton below a subfolder; E if `strict_file_type_check`, else I)"#,
        r#"folder discarded / kept"#,
    ),
    (
        "common_file_disallowed",
        Severity::Error,
        r#"as `file_type_disallowed`, Common scope"#,
        r#"files discarded / kept"#,
    ),
    (
        "player_layout_proto",
        Severity::Error,
        r#"a referee export's player folder has a direct `face/`, `boots/`, `gloves/` or `common/` subfolder: the AET referee layout, Red's prototype of the player folder, whose subfolders type their files by the subfolder's name where the Studio rule types a subfolder's files by their names (`spec/aesthetics_export/player_folders.md` "Subfolders"); in a team export those names are plain subfolders (context `folder`: the first such subfolder)"#,
        r#"folder discarded (`DropFolder`); the hint says to run the export through the Export upgrader once"#,
    ),
    (
        "shared_link_duplicate",
        Severity::Error,
        r#"player folder has more than one shared link for any category (face, boots, or gloves), counted over its whole tree, two links naming one shared folder counting once"#,
        r#"folder discarded (`DropFolder`)"#,
    ),
    (
        "link_target_missing",
        Severity::Error,
        r#"link file references a nonexistent shared folder"#,
        r#"folder discarded"#,
    ),
    (
        "common_link_missing",
        Severity::Error,
        r#"a `.common` link (model, material file or texture) names a file missing from the Common folder (a `kitN` stem or directory in it standing for its set, `spec/model_format.md` "Kit-dependent assets") (context: the link file, whose name shows its kind, and the Common path looked for); on PES 18-21 a texture link is satisfied by an installed CPK holding the texture at the team's Common path (`pipeline.md` "Resolved decisions", "A texture a model names must exist")"#,
        r#"folder discarded"#,
    ),
    (
        "link_target_dropped",
        Severity::Error,
        r#"a shared-folder or `.common` link names a target the structure pass or the deep pass dropped, by its effective disposition (context: the link, the target and its finding)"#,
        r#"folder discarded (`DropFolder`); not reported when `pass_through` keeps the target; never itself pass-through-eligible"#,
    ),
    (
        "shared_folder_orphaned",
        Severity::Warning,
        r#"shared folder referenced by no player"#,
        r#"folder skipped"#,
    ),
    (
        "fpc_conflict",
        Severity::Error,
        r#"both `fpc_on` and `fpc_off` present in a player folder"#,
        r#"folder discarded"#,
    ),
    (
        "ingame_face_explicit_face_model",
        Severity::Error,
        r#"`ingame_face` combined with explicit face models (`face_high`/`hair_high`/`oral`, in any supported source format) or a shared face link, which require the suppressed face folder"#,
        r#"player folder discarded (`DropFolder`)"#,
    ),
    (
        "texture_stem_conflict",
        Severity::Error,
        r#"two image files with the same stem in one lookup namespace, whatever their extensions: one folder of a model folder's tree, the root or one subfolder (`hair.dds` beside `hair.png`, or a texture link `hair.png.common`, which counts as a file of its linked name; `hair.dds` beside `jessie/hair.dds` is no conflict, a model's texture name resolving nearest first, `spec/aesthetics_export/player_folders.md` "Subfolders"), `Common/` (`hair.dds` beside `hair.png`; each directory of `Common/` a namespace of its own), a kit folder, or `Kits/all/` (`kit.png` beside `kit.dds`; a kit's own file overriding an `all/` file of its stem is not a conflict), or `Portraits/` (`05.dds` beside `05 - Bob.png`, two files of one slot)"#,
        r#"folder discarded (the model folder, or the kit; for `all/`, `all/` itself, so no kit inherits from it); in `Portraits/` and `Common/`, both files (a player linking a dropped Common file follows `link_target_dropped`)"#,
    ),
    (
        "fmdl_name_invalid",
        Severity::Error,
        r#"Fox: FMDL anywhere in a boots/gloves shared folder not resolving to that category's allowed names"#,
        r#"folder discarded"#,
    ),
    (
        "model_name_invalid",
        Severity::Error,
        r#"pre-Fox: a model anywhere in a shared boots or gloves folder without its category's suffix, the pre-Fox twin of `fmdl_name_invalid`"#,
        r#"folder discarded"#,
    ),
    (
        "edithair_unsupported",
        Severity::Error,
        r#"pre-Fox: `face_edithair.xml` / `hair.xml` present (anywhere in a player or shared folder; reported by the structure pass, so `check` reports it)"#,
        r#"folder discarded"#,
    ),
    (
        "kit_folder_invalid",
        Severity::Error,
        r#"subfolder name doesn't parse as `<slot>[ - <label>]` with the slot in `p1`–`p9`, `g1`, `all`"#,
        r#"kit discarded"#,
    ),
    (
        "kit_slot_duplicate",
        Severity::Error,
        r#"two subfolders resolve to the same slot (`p1/` and `p1 - Lakers/`)"#,
        r#"both discarded, never a pick"#,
    ),
    (
        "kit_texture_name_invalid",
        Severity::Error,
        r#"texture doesn't carry the `kit` prefix (`kit.dds`, `kit_mask.dds`, …)"#,
        r#"file discarded"#,
    ),
    (
        "kit_layout_conflict",
        Severity::Error,
        r#"both `pre-fox` and `fox` markers in one kit folder"#,
        r#"kit discarded"#,
    ),
    (
        "kit_icon_invalid",
        Severity::Warning,
        r#"a kit's `icon_<N>` marker is numbered above 23, or the kit holds two or more (each is reported)"#,
        r#"default icon (3) used"#,
    ),
    (
        "kit_all_file_ignored",
        Severity::Warning,
        r#"`all/` holds something other than kit textures (`config.toml`, `colors.txt`, an `icon_<N>` marker, anything else)"#,
        r#"file ignored"#,
    ),
    (
        "kit_all_unused",
        Severity::Warning,
        r#"`all/` present but no kit folder to inherit from it"#,
        r#"none"#,
    ),
    (
        "kit_textures_inherited",
        Severity::Info,
        r#"a kit lacks stems that `all/` provides; lists them (`p3: back, leg, name from all/`)"#,
        r#"textures inherited"#,
    ),
    (
        "portrait_name_invalid",
        Severity::Error,
        r#"a `Portraits/` file that is not a texture named `NN` with NN in 01–23 (alone or followed by ` - ` and any text, as a player folder is numbered)"#,
        r#"file discarded"#,
    ),
    (
        "logo_file_invalid",
        Severity::Error,
        r#"a root `logo*` file has an unknown tag in its stem, a disallowed format, or fails to decode in the deep pass"#,
        r#"no logo emitted for the team (both files dropped as one unit, the game needs all three sizes)"#,
    ),
    (
        "logo_role_duplicate",
        Severity::Error,
        r#"two files claim the same role (two `logo*`, or two `logo_small*`)"#,
        r#"no logo emitted for the team"#,
    ),
    (
        "logo_small_without_main",
        Severity::Error,
        r#"`logo_small*` present with no main `logo*`"#,
        r#"no logo emitted (the small image is not a source for the large sizes)"#,
    ),
    (
        "root_file_unexpected",
        Severity::Warning,
        r#"unknown file or folder at the export root (a folder other than the content folders, a stale `wrapper/` beside a usable root included), or a file directly inside `Players`, `Kits`, `Faces`, `Boots` or `Gloves`, which hold only folders (`Players/players.txt`)"#,
        r#"file or folder ignored"#,
    ),
    (
        "notes_found",
        Severity::Info,
        r#"non-empty valid root `notes.txt` present"#,
        r#"collected into teamnotes.txt by `compile`"#,
    ),
    (
        "notes_encoding_invalid",
        Severity::Error,
        r#"root `notes.txt` is not valid UTF-8 after optional BOM handling"#,
        r#"note dropped; export otherwise continues (`DropFile`, not pass-through-eligible)"#,
    ),
];

/// The catalog rows the help's generated Messages topic is rendered from, in `CATALOG`
/// order (`lib.rs` `help`).
pub(crate) fn catalog() -> Vec<CatalogRow> {
    CATALOG
        .iter()
        .map(|row| CatalogRow {
            code: row.0,
            severity: row.1.name(),
            when: row.2,
            effect: row.3,
        })
        .collect()
}

/// The severity `code` is shown at.
fn severity(code: &str) -> Severity {
    CATALOG
        .iter()
        .find(|row| row.0 == code)
        .map(|row| row.1)
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
            let rows = CATALOG.iter().filter(|(known, ..)| known == code).count();
            assert_eq!(rows, 1, "{code}");
        }
        for code in Code::ALL {
            let rows = CATALOG
                .iter()
                .filter(|(known, ..)| *known == code.as_str())
                .count();
            assert_eq!(rows, 1, "{}", code.as_str());
        }
        for (index, (code, ..)) in CATALOG.iter().enumerate() {
            assert!(
                CATALOG[index + 1..].iter().all(|(other, ..)| other != code),
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

    #[test]
    fn every_catalog_row_carries_its_member_facing_text() {
        for (code, _, when, effect) in CATALOG {
            assert!(!when.is_empty() && !effect.is_empty(), "{code}");
        }
    }

    #[test]
    fn the_help_chapter_lists_the_whole_catalog_in_order() {
        use studio_core::StudioTool;

        let help = crate::Tool::new().help();
        let codes: Vec<&str> = help.messages.iter().map(|row| row.code).collect();
        let catalog: Vec<&str> = CATALOG.iter().map(|row| row.0).collect();
        assert_eq!(codes, catalog);
    }

    #[test]
    fn a_rows_text_is_its_spec_cells() {
        let row = CATALOG.iter().find(|row| row.0 == "no_old_export").unwrap();
        assert_eq!(row.2, "step 1: the input holds nothing an old export holds");
        assert_eq!(row.3, "nothing is written");
        // A structure-pass row takes the Team compiler's spec cells for the code, unless this
        // tool's own table describes it (`team_name_unknown`, `source_read_failed`).
        let row = CATALOG
            .iter()
            .find(|row| row.0 == "nested_folders_fixed")
            .unwrap();
        assert_eq!(
            row.2,
            "content found nested one level down (exactly one usable root)"
        );
        assert_eq!(row.3, "auto-fixed");
        let row = CATALOG
            .iter()
            .find(|row| row.0 == "team_name_unknown")
            .unwrap();
        assert_eq!(row.3, "nothing is written");
    }
}
