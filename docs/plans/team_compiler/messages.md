# 4cc Studio — Team compiler plan: Message catalog

Part of the [Team compiler plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Message catalog

The warning/error messaging is overhauled, not ported (see `README.md` "Relationship to the older compilers").
Red's ~150 messages (audited across `export_check.py`, `texture_check.py`, `xml_check.py`,
`team_id_get.py`, `portraits_move.py`, `bins_update.py`, `referee_tools.py`, `cpk_tools.py` and the
stage scripts) reduce to a structured catalog: every reportable condition gets a stable ID, and the
pipeline emits it as **data** — the GUI, CLI, and log files decide presentation.

### Message structure

```rust
use std::borrow::Cow;

pub struct MessageCode {
    pub tool_id: &'static str,             // "team-compiler"
    pub code: Cow<'static, str>,            // "player_number_duplicate"
}

pub struct Message {
    pub code: MessageCode,                  // cross-tool stable identity
    pub severity: Severity,                 // presentation and filtering
    pub disposition: Disposition,           // processing consequence, independent of severity
    pub scope: Scope,                       // what the message is about — drives grid cell mapping
    pub context: Vec<(String, String)>,     // structured fields: file name, expected/found values, ...
}

pub enum Severity {
    Fatal,
    Error,
    Warning,
    Info,
}
// Ordering: Info < Warning < Error < Fatal. "Errors" filter includes Fatal.
// Cell status uses the highest relevant severity after disposition policy.

pub enum Disposition {
    Keep,
    DropFile,
    DropSlot,                             // discard one normalized players.txt assignment
    DropFolder,
    DropExport,
    AbortRun,
}

pub enum Scope {
    Run,                                  // not tied to an export (output stage, settings)
    Export { export_id: ExportId },
    Folder { export_id: ExportId, path: vtree::ScopePath }, // player/shared/kit folder
    File   { export_id: ExportId, path: vtree::ScopePath },
    RosterEntry { export_id: ExportId, file: vtree::ScopePath, line: usize, slot: Option<u16> },
}
```

`ExportId` and canonical `vtree::ScopePath` values route events and logs; human-readable
export/folder/file names stay in message context. `ScopePath` is the platform-neutral canonical path
type: `aesthetics_export` uses/wraps it but does not own it, keeping `studio_core` independent of
team-format knowledge. Stable IDs avoid ambiguity when a folder export and an archive have the same
display stem.

The `MessageCode`/`Message`/`Severity`/`Disposition`/`Scope` types live in `studio_core` alongside
`PipelineEvent` (all tools emit them). Each tool owns its code strings and text catalog under its
stable tool ID; the catalog below belongs to `team-compiler`, while the stadium compiler and export
upgrader define their own codes on the same structure.

Principles:

- **One condition, one ID.** Red repeats near-identical messages per folder type ("Bad face folders"
  / "Bad boots folders" / "Bad gloves folders"); here the folder identity is in the `scope`, so one
  `fmdl_name_invalid` ID covers all model folder kinds.
- **Message text lives in one table** keyed by ID (template + remediation hint), not scattered
  through the pipeline. Red's remediation-hint style is kept ("Resize it so that both sizes are
  powers of 2").
- **Consequences are explicit**: `Disposition` records whether the finding keeps content, drops a
  file, normalized roster slot, folder, or export, or aborts the run; severity remains a
  presentation/filtering concern. File-scoped consequences still roll up according to Red's rules
  (model-file failures drop the folder; standalone texture/portrait failures may drop only the
  file). A line-local `RosterEntry` finding with `DropSlot` removes only that normalized assignment;
  file-wide encoding failure or roster ambiguity that prevents trustworthy normalization uses
  `DropExport` instead. Runtime failures derive their effective disposition from the owning scope:
  optional root file → `DropFile`, folder/task → `DropFolder`, required export metadata or unusable
  source → `DropExport`, output-writer/global invariant → `AbortRun`.
- **`pass_through`** overrides only eligible content-level `DropFile`/`DropFolder` dispositions to
  keep-with-flag, as in Red, while retaining the original severity. Not eligible: findings whose
  content cannot exist (failed conversion/packing, unproducible logos), unused content
  (`shared_folder_orphaned`), ambiguous model-source or texture identity/conflicts, foundational
  unsafe-path/ambiguous-roster/required-metadata failures, `DropSlot`, `DropExport`, `AbortRun`, and
  environment-level dispositions. Written errored content
  receives a distinct `DoneWithErrors` outcome rather than an ordinary red error state.
- **No blocking console prompts.** Red's prompts are replaced, not kept: unknown team ID becomes the
  grid's inline-editable ID cell (including its reassignment confirmation — see the GUI section),
  standing consents become settings (`dt00_overwrite_allow`), and retry-on-locked-file becomes a
  plain Error. The CLI never prompts: all of these are hard errors there.
- **Progress chatter is not a message.** Red's "- Packing the face folders..." prints map to
  `Progress` events, not catalog entries.
- **Logs**: `issues.log` (Warning+) and `suggestions.log` (Info) are kept as disk artifacts,
  rendered from the same events. Display layers may group repeated (code, scope) pairs; the logs
  stay complete.

### Catalog

Adapted to the Studio export format: Red's Note-file, Kit Configs/Kit Textures-folder,
dependency-check, and self-healing messages are gone; player-folder, link-file, settings.toml, and
savefile messages are new.

**Export level**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `export_extract_failed` | E | archive cannot be extracted/parsed | export skipped (`DropExport`) |
| `export_disabled` | I | root `NO_USE` / `NO_USE.txt` marker disables this source | export skipped; omitted from the grid and from duplicate-ref detection |
| `source_read_failed` | E/F | a pinned source entry cannot be read | optional root file: `DropFile`; folder/task producer: `DropFolder`; required export metadata or unusable source: `DropExport`; output/global source invariant: `AbortRun` |
| `source_changed_during_run` | F | this export's pinned revision changes during planning/materialization | run aborted and partial output discarded (`AbortRun`) |
| `template_override_unreadable` | E/F | a template override file exists but cannot be read (embedded defaults cannot be missing) | folder-local injected template: `DropFolder`; referee-export template: `DropExport`; global CPK/bin template: `AbortRun` |
| `template_override_active` | I | a `templates/` override file is shadowing an embedded template/fallback-bin resource | override used; reported per file per run |
| `export_balls_skipped` | I | export name's first word is `balls` (balls exports belong to the Balls compiler) | export skipped (`DropExport`) |
| `multiple_ref_exports` | E | more than one non-disabled `refs`-prefixed export found at discovery | every conflicting export skipped (`DropExport` per row, plus a Run-scoped summary) |
| `duplicate_aesthetics_export` | E | multiple exports resolve to the same team ID | all conflicting exports skipped (`DropExport`) |
| `boots_id_pool_exhausted` | E | planned player-exclusive/shared boots outputs exceed the team's permanent ID block | export skipped (`DropExport`) |
| `gloves_id_pool_exhausted` | E | planned player-exclusive/shared gloves outputs exceed the team's permanent ID block | export skipped (`DropExport`) |
| `export_empty` | E | no usable content found at root | export skipped |
| `nested_folders_fixed` | W | content found nested one level down (exactly one usable root) | auto-fixed |
| `nested_root_ambiguous` | E | several nested child folders are usable export roots | export skipped (`DropExport`) |
| `nested_root_conflict` | E | loose root file collides with a flattened nested file at the same virtual path | export skipped (`DropExport`) |
| `team_name_unknown` | E | canonical `team_name` not in `teams_list.txt` | export skipped (GUI: ID cell becomes editable for inline assignment) |
| `team_id_out_of_range` | E | resolved ID outside 701–920 | export skipped |
| `teams_list_read_only` | W | a teams-list write (ID cell, updater merge) failed because the data directory is not writable | write dropped; the in-memory list is unchanged (no elevation — see `pipeline.md` "Resolved decisions", "Teams list") |
| `team_colors_missing` | I | no root `colors.txt` (it is optional) | TeamColor.bin entry left untouched |
| `color_entry_invalid` | W | unparsable RGB line in a `colors.txt` | entry skipped |
| `root_file_unexpected` | W | unknown file at export root | file ignored |
| `portrait_conflict` | E | same player number with differing portraits in player folder and `Portraits/` | export skipped |
| `notes_found` | I | non-empty valid root `notes.txt` present | collected into teamnotes.txt |
| `notes_encoding_invalid` | E | root `notes.txt` is not valid UTF-8 after optional BOM handling | note dropped; export otherwise continues (`DropFile`, not pass-through-eligible) |

**Player folders and shared model folders**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `player_folder_number_invalid` | E | without `players.txt`, a team player folder's number is not in 01–23 | folder discarded (`DropFolder`) |
| `players_txt_slot_invalid` | E | a `players.txt` line's decimal slot is outside 01–23 (teams) / 01–35 (refs) | that `RosterEntry` assignment skipped (`DropSlot`) |
| `player_number_duplicate` | E | without `players.txt`, two team folder names claim the same player slot | both conflicting folders discarded (`DropFolder` on each folder) |
| `players_txt_missing` | E | referee export has no required root `players.txt` (nor a legacy `refs.txt` alias) | export skipped (`DropExport`); `ParsedAestheticsExport` remains available to the Refs arranger for repair |
| `refs_txt_ignored` | W | referee export has both `players.txt` and the legacy `refs.txt` alias | `players.txt` used, alias ignored |
| `players_txt_slot_duplicate` | E | authoritative `players.txt` assigns the same slot more than once, making roster identity ambiguous | export skipped (`DropExport`) |
| `player_unlisted` | W | authoritative `players.txt` is present but does not list this player folder, regardless of any number in its name | folder ignored |
| `players_txt_line_invalid` | E | one nonblank `players.txt` line cannot be parsed into decimal slot + folder name | that `RosterEntry` skipped (`DropSlot`) |
| `players_txt_invalid` | E | file-level encoding failure or roster-wide ambiguity prevents trustworthy normalization, or a referee roster retains zero valid assignments | export skipped (`DropExport`; foundational). An empty normal-team roster remains valid for a kit-only export |
| `players_txt_target_missing` | E | `players.txt` entry names a nonexistent folder | `RosterEntry` scoped assignment skipped (`DropSlot`) |
| `link_target_missing` | E | link file references a nonexistent shared folder | folder discarded |
| `shared_link_duplicate` | E | player folder has more than one shared link for any category (face, boots, or gloves) | folder discarded (`DropFolder`) |
| `link_combined` | I | link file plus local models for the same category; the shared models become parts of the player's own set (Fox: mesh-merged, own ID) | none |
| `shared_folder_orphaned` | W | shared folder referenced by no player | folder skipped |
| `settings_toml_invalid` | E | optional `settings.toml` fails to parse | file ignored and existing savefile values preserved (`DropFile`, not pass-through-eligible); models continue |
| `materials_toml_invalid` | E | a `materials.toml` or `*.materials.toml` file fails to parse | folder discarded |
| `material_file_missing` | E | a glTF file has no matched material file (no name-matched `*.materials.toml`, no catch-all `materials.toml`, no `.common` link to either) | folder discarded |
| `material_undefined` | E | a material name referenced by a glTF file has no definition in any matched material toml | folder discarded |
| `material_key_unknown` | W | a material toml entry has a key outside the schema (typo or newer-Studio field) | key ignored |
| `material_value_invalid` | E | a material toml value has the wrong type, is out of range, names an unknown family/role/state, or uses a native sampler name reserved by a canonical role | folder discarded |
| `material_texture_not_found` | E | a mesh-used glTF material's `base`, `""`-marked, or explicit texture stem cannot be resolved | folder discarded |
| `material_texture_unused` | I | a texture role the target engine has no sampler for (e.g. `metalness` on pre-Fox) | role dropped for that engine |
| `material_parameter_unused` | I | a shader parameter the target engine's shader does not define | parameter dropped for that engine |
| `material_entry_unused` | I | a material toml entry matches no material in the folder's glTF models | none |
| `materials_file_ambiguous` | I | multiple name-matched `*.materials.toml` files match one glTF; definitions layer least → most specific (intentional layered setups are ordinary usage) | none |
| `material_link_layered` | I | a `.common` material link and a local material file share a stem; the Common file layers below the local one | none |
| `gltf_embedded_image_ignored` | I | a glTF embeds texture bytes (stock Blender's default); textures are only ever read from the folder's files | embedded images ignored (once per model) |
| `gltf_bone_unknown` | E | a glTF skin joint has neither `PES_bone` metadata nor a name in the template skeleton | folder discarded |
| `bone_folded_for_version` | I | a used bone the target PES version's skeleton lacks had its weights transferred to the fold table's (or nearest) bone — see "Skeleton retargeting" in the Model conversion plan | none |
| `skeleton_retargeted` | I | the model's bind pose was re-bound from its source version's skeleton to the target's (bones moved more than tolerance) | none |
| `kit_variant_missing` | W | a `kitN` reference has no variant for a kit number the export defines | lowest existing variant copied into the gap |
| `kit_variant_model_fox` | W | per-kit model variants (`*_kit1`, `*_kit2`, …) on a Fox target, which has no model-path indirection | lowest variant used, others ignored |
| `common_link_missing` | E | a `.common` link — model, material file or texture — names a file missing from the Common folder (context: link kind and the Common path looked for) | folder discarded |
| `settings_toml_name_shared` | W | `name` given in a folder mapped to multiple players | name applied to all of them |
| `fpc_conflict` | E | both `fpc.on` and `fpc.off` present in a player folder | folder discarded |
| `fpc_strip_conflict` | W | `settings.toml` strip keys conflict with the folder's FPC marker | FPC preset wins; keys ignored |
| `fmdl_name_invalid` | E | Fox: FMDL in a boots/gloves shared folder not resolving to that category's allowed names | folder discarded |
| `fmdl_fcl_hair_fallback` | I | Fox: arbitrary-named FMDL treated as a face model, routed into the `fcl_hair.fmdl` merge (Red's single-file fallback, generalized) | none |
| `fmdl_merged` | I | Fox: multiple models resolve to the same allowed name; meshes merged into one FMDL (alphabetical source order) | none |
| `merge_material_conflict` | E | merged parts define the same material name differently (Fox merge or pre-Fox `ingame_face` merge) | folder discarded |
| `skl_merge_conflict` | E | merged parts have incompatible skeletons (Fox compares effective SKL content; pre-Fox IR merge compares bone transforms) | folder discarded (`DropFolder`) |
| `skl_no_slot` | W | Fox: custom `.skl` paired with a model resolving to `face_high`/`hair_high`/`oral` (no skeleton slot exists for those) | SKL ignored (no-op file) |
| `ingame_face_explicit_face_model` | E | `ingame_face` combined with explicit face models (`face_high`/`hair_high`/`oral`, in any supported source format) or a shared face link, which require the suppressed face folder | player folder discarded (`DropFolder`) |
| `shared_texture_conflict` | E | a player's tasks produce the same common-texture destination with different bytes | losing task discarded (`DropFolder`; canonical winner face > boots > gloves); identical bytes deduplicate |
| `merged_texture_conflict` | E | two merge-copied parts within one output model produce the same texture destination with different bytes (no canonical winner exists inside one model) | folder discarded (`DropFolder`); identical bytes deduplicate |
| `model_source_ambiguous` | E | duplicate model sources within the selected representation (target-native > glTF > convertible opposite) | folder discarded (`DropFolder`) |
| `model_conversion_failed` | E | selected model cannot be converted to the target format | folder discarded (`DropFolder`) |
| `folder_pack_failed` | E/F | a task cannot build its packed batch | folder/task: `DropFolder`; output-writer/global: `AbortRun` |
| `model_name_invalid` | E | pre-Fox: `.model` not in the allowed names for its category | folder discarded |
| `xml_broken` | E | XML fails to parse (with line/column) | folder discarded |
| `mtl_broken` | E | MTL fails to parse (with line/column) | folder discarded |
| `edithair_unsupported` | E | `face_edithair.xml` / `hair.xml` present | folder discarded |
| `file_type_disallowed` | E/I | extension not in the mode's allowlist (E if `strict_file_type_check`, else I) | folder discarded / kept |
| `fmdl_no_texture_ids` | W | no ID-bearing texture paths found in FMDL | none (double-check hint) |

**Textures** (file-scoped; in model folders the folder fails, elsewhere the file is normally
dropped; the logo sources are the exception — main and `_small` fail as one atomic producer of the
game's three sizes)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `texture_too_small` | E | dimension < 4 | discarded |
| `texture_not_pow2` | E | portrait not power-of-2 / Fox mipmapped without pow2 side | discarded |
| `texture_not_div4` | E | pre-Fox compressed texture not divisible by 4 | discarded |
| `kit_texture_too_big` | E | main kit texture > 2048² or not pow2 | discarded |
| `kit_texture_uncompressed` | E | main kit texture in uncompressed format | discarded |
| `texture_type_mismatch` | E | header doesn't match extension (renamed, not resaved) | discarded |
| `texture_codec_unsupported` | E | codec not convertible in-process | discarded |
| `texture_stem_conflict` | E | two image files with the same stem but different extensions in one model folder | folder discarded |

**XML/MTL content checks** (pre-Fox, plus `face_diff.xml` in Fox)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `xml_texture_path_missing` | E | sampler with no texture path and not auto-fillable (sampler not in the suffix mapping table) | folder discarded |
| `mtl_texture_not_found` | E | texture (by stem) used by one of the model's meshes missing from export | folder discarded |
| `mtl_texture_unused_missing` | I | texture referenced only by materials no mesh uses | none |
| `xml_root_tag_invalid` | E | root tag is not `<config>` | folder discarded |
| `xml_model_type_missing` | E | `<model>` without `type` | folder discarded |
| `xml_model_path_missing` | E | `<model>` without `path` | folder discarded |
| `xml_model_not_found` | E | `path` (or `material`) is a `./` or Common reference whose file is missing from the export | folder discarded |
| `xml_common_path_invalid` | E | Common reference without 3-char subfolder | folder discarded |
| `xml_oral_prefix_missing` | E | PES16 target: model file name starting with none of `face_high_`/`hair_high_`/`oral_` | folder discarded |
| `xml_dif_conflict` | E | the xml carries a `<dif>` and the folder also has `face_diff.xml` (two sources for one datum) | folder discarded |
| `xml_element_unknown` | W | a child of `<config>` other than `<model>`/`<dif>` | kept verbatim |
| `xml_attribute_unknown` | W | a `<model>` attribute other than `level`/`type`/`path`/`material`/`ratio` | kept verbatim |
| `xml_type_unknown` | W | `type` not in the generated vocabulary (the type table plus `uniform_sub`) | kept verbatim |
| `xml_level_lod` | I | `level` other than `0` — the model's LoD level; higher levels are uncommon but known to work | kept verbatim |
| `xml_ratio_invalid` | W | `ratio` that is not a number | kept verbatim |
| `xml_path_unchecked` | W | `path`/`material` in a form the compiler cannot resolve (`model/character/face/common/…` or any other game path): not verified to exist | kept verbatim |
| `xml_face_neck_multiple` | W | more than one `face_neck` entry | kept verbatim |
| `xml_model_unlisted` | W | a model file in the folder that the xml does not reference | file not emitted |
| `xml_face_neck_added` | I | no `face_neck` entry; the dummy entry was appended (Red's rule) | dummy model + mtl emitted |
| `xml_uniform_pes15` | I | PES15 target: `type="uniform"` rewritten to `uniform_sub` (Red's rule) | rewritten |
| `xml_ignored_fox` | I | a user `face.xml` in a folder compiled for a Fox target | xml ignored; models compile by the normal route |
| `face_diff_invalid` | E | `face_diff.xml` structure/base64/binary validation failed | folder discarded |
| `mtl_material_duplicate` | E | material listed twice | folder discarded |
| `mtl_state_invalid` | E | `ztest` ≠ 1 / `blendmode` ∉ {0,1} / `alphablend` ∉ {0,1} | folder discarded |
| `mtl_blendmode_nonzero` | W | `blendmode` = 1 (works, not recommended) | none |
| `mtl_state_missing` | I | state names absent, defaults used | none |
| `mtl_state_nonrecommended` | I | `alphablend`/`zwrite` combination not recommended | none |

The three `mtl_state_*` checks other than `mtl_state_missing` also run on a glTF material's
`[prefox.states]` table when the target is pre-Fox (the format plan's schema mirrors the `.mtl`
states one-to-one; a glTF material with states absent just gets its family's state set, so
`mtl_state_missing` does not apply).

Red's "no models from Common on PES16" check is intentionally **not** ported: the PES16 exe will be
patched to allow loading models from Common, so the compiler must not reject such references.

**User-supplied `face.xml`.** A pre-Fox model folder may ship its own `face.xml` instead of having
the compiler generate one. This is **second-class, accepted on purpose**: the generated xml covers
everything the format knows how to express, and a hand-written one exists to let a user try what the
pre-Fox engines can do that nobody has mapped yet (a `type` no table lists, a `level` other than 0,
an element the generator never writes). Because a malformed xml can crash the game, the compiler
checks it carefully, and the severity line is drawn by evidence, not by taste:

- **Error** — what is known not to work, or cannot work: unparsable XML, a root other than
  `<config>`, a `<model>` without `type` or `path`, a `./` or Common reference to a file the export
  does not contain (the one thing the compiler can verify outright), a Common path without its
  3-character subfolder, the PES16 `oral_`/`face_high_`/`hair_high_` name rule, and two sources for
  the face diff. These are Red's checks (`xml_check.py`), each added after real breakage, plus the
  two structural impossibilities Red happened not to test (a missing `path`, a doubled diff).
- **Warning** — everything outside the vocabulary the compiler's own generator emits, because
  that vocabulary is the in-game-verified set and nothing outside it is: an unknown element or
  attribute, a `type` not in the type table, a non-numeric `ratio`, a path in a form the
  compiler cannot resolve (`model/character/face/common/…` and any other game path — Red waved
  these through silently; the compiler says it did not check). The value is **kept verbatim**: a
  warning is the compiler saying "I can't vouch for this", never "I changed this". A model file in
  the folder that the xml does not list is also a warning — it will not be in the game, which is
  almost always a mistake and occasionally the point.
- **Info** — the compiler's own rewrites, which it performs on a user xml exactly as it would on a
  generated one and reports so the user knows the emitted file differs from the source: the
  `face_neck` dummy appended when no entry has that type; `uniform` → `uniform_sub` on PES15; the
  team ID substituted into Common paths and `kitN` tokens passed through; and the `<dif>` block
  inserted from `face_diff.xml` when the xml has none. On a Fox target the xml is ignored with an
  info line and the folder's models compile by the normal route (Fox has no `face.xml`). Also info,
  though not a rewrite: a `level` other than 0 — it is the model's LoD level, and higher levels are
  merely uncommon, not unverified (a value that has already made the move from warning to known).

Rules that follow from "the xml is the authority in its folder": filename typing is off — the
suffix table is for generating an xml, and this folder has one; `.common` model links are not needed
(the xml names Common paths itself) and an unlisted one is just an unlisted file; only what the xml
references is emitted, plus the MTLs and textures those models bind, which go through the ordinary
MTL and texture checks unchanged. The compiler always re-serializes the xml it emits (it has to, for
the ID substitution and the appended entries), so comments, whitespace and the encoding declaration
of the source never reach the game — the checks above are about content, not formatting. New
knowledge moves a row: when an experiment shows a value works, it joins the generated vocabulary and
stops warning; when one shows a value crashes, it becomes an error with the crash as its citation.

Texture existence is checked **deep**, not shallow: `pes_model` parses the `.model` files and knows
which MTL material each mesh actually binds, so a missing texture on a mesh-used material is a hard
error, while a miss on a material no mesh uses can never break the model and is only reported as
info. Red only reads the MTL and cannot tell used materials from unused ones, so it has to report
every miss as a warning.

**Kits**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `kit_folder_invalid` | E | subfolder name doesn't parse as `<slot>[ - <label>]` with the slot in `p1`–`p9`, `g1`, `all` | kit discarded |
| `kit_slot_duplicate` | E | two subfolders resolve to the same slot (`p1/` and `p1 - Lakers/`) | both discarded — never a pick |
| `kit_textures_inherited` | I | a kit lacks stems that `all/` provides; lists them (`p3: back, leg, name from all/`) | textures inherited |
| `kit_all_file_ignored` | W | `all/` holds something other than kit textures (`config.toml`, `colors.txt`, `icon.txt`, anything else) | file ignored |
| `kit_all_unused` | W | `all/` present but no kit folder to inherit from it | — |
| `kit_config_generated` | I | no `config.toml`; generated from template (with FPC values if team FPC is on) | auto-fixed |
| `kit_config_invalid` | E | `config.toml` fails to parse or validate (ranges, cross-field constraints) | kit discarded |
| `kit_config_version_clamped` | W | a field doesn't fit the target PES version's encoding (e.g. Name Y > 16 before PES 21) | value clamped |
| `kit_config_fpc_adjusted` | I | team kit-FPC status is On but a config lacks the FPC values — supplied configs and unexported slots' base entries alike, GK kit included | auto-fixed (values written; FPC values are never auto-reverted) |
| `kit_config_fpc_unpatched` | W | team kit-FPC status is On but an unexported kit slot has no base entry or config to patch | slot left alone; the team needs a kit export |
| `kit_placeholder` | I | the kit's effective textures lack `kit.dds` (an empty folder included); the bundled checkerboard stands in | placeholder kit emitted: checkerboard texture, template config unless supplied, UniColor entry per the colors fallback |
| `kit_colors_derived` | I | kit `colors.txt` missing, or present but yielding fewer than two valid colors; menu colors extracted from the kit's own main texture | auto-fixed |
| `kit_colors_missing` | W | no usable `colors.txt` and no own main texture to derive from (placeholder kits without a `colors.txt` always) | the magenta/black "no colors chosen" pair is written, so the gap shows in the game menus |
| `kit_icon_invalid` | W | `icon.txt` doesn't parse to a number in 0–23 | default icon (3) used |
| `kit_texture_name_invalid` | E | texture doesn't carry the `kit` prefix (`kit.dds`, `kit_mask.dds`, …) | file discarded |
| `kit_texture_not_used` | I | the kit's effective set has a `kit_mask` on a Fox target or a `kit_srm` on a pre-Fox target — the other engine's map | file not emitted (never converted into the other map) |
| `kit_layout_conflict` | E | both `pre-fox` and `fox` markers in one kit folder | kit discarded |
| `kit_layout_converted` | I | the kit's layout marker names the other engine than the target; names the direction (`p1: pre-fox → fox`) | `kit` and its mask/srm re-laid out per `KIT_LAYOUT_REMAP`; other textures untouched |

**Portraits / Logo / Collars / Common**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `portrait_name_invalid` | E | not `player_NN.dds` with NN in 01–23 | file discarded |
| `logo_file_invalid` | E | a root `logo*` file has an unknown tag in its stem, a disallowed format, or fails to decode in the deep pass | no logo emitted for the team (both files dropped as one unit — the game needs all three sizes) |
| `logo_role_duplicate` | E | two files claim the same role (two `logo*`, or two `logo_small*`) | no logo emitted for the team |
| `logo_small_without_main` | E | `logo_small*` present with no main `logo*` | no logo emitted (the small image is not a source for the large sizes) |
| `logo_fit_applied` | I | a non-square source was made square; names the mode (`fit` by default, or the file's tag) | — |
| `logo_upscaled` | W | a source is smaller than its largest target (512² for main, 128² for small) | emitted upscaled |
| `collar_id_invalid` | E | collar filename doesn't parse as `collar_[ID]`, the ID is outside the supported stock-collar range, or it names the reserved FPC collar 105 | collar file discarded |
| `collar_id_conflict` | E | another export already claimed this stock collar ID in this run (canonical export order) | later team's collar discarded; its configs not rewritten |
| `common_file_disallowed` | E/I | as `file_type_disallowed`, Common scope | files discarded / kept |

**Referees** — referee exports use the same player-folder format (see `blue_port.md` "Referee export processing"),
so all player-folder, texture, and XML/MTL messages above apply as-is. Referee-specific additions:

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `ref_marker_needs_consent` | E | Fox `ref_marker.dds` present but dt00 overwrite not enabled (setting) | marker skipped |
| `dt00_write_failed` | E | `dt00_x64.cpk` cannot be updated with the converted Fox referee marker | deployment transaction fails and rolls back; compile artifacts remain available |

Referee-marker deployment findings are environment-level dispositions: `pass_through` never
overrides them, even when the referee CPK itself remains compilable.

**Output stage and savefile** (Run scope)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `duplicate_path` | W/E | manifest preflight finds colliding output paths | folder collision: losing folder blocked (`DropFolder`, E); whole-export collision: losing export blocked (`DropExport`, E); sideload override: sideload wins (`Keep`, W) |
| `cpk_write_failed` | F | incremental CPK writing fails | run aborted and partial CPK discarded (`AbortRun`) |
| `output_commit_failed` | F | a completed CPK/tree cannot be atomically committed to its final output path | run aborted; prior published outputs remain untouched (`AbortRun`) |
| `uniparam_compile_failed` | F | UniformParameter compilation failed (output would crash PES) | run aborted |
| `pes_folder_not_found` | E | `pes_folder_path` invalid at deployment time (includes the no-PES-install machine) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `pes_version_mismatch` | W | `pes_folder_path` exists but holds no `PES20{pes_version}.exe` — a different-version exe suggests a wrong `pes_version` setting (Red's suppressible console notice becomes a plain per-run warning, dropping `state/ver_mismatch_warned.txt`; the GUI also surfaces this live via the version selector's red/yellow state — see the core plan's Sidebar) | none; deployment proceeds |
| `dpfilelist_missing` | E | no `DpFileList.bin` in download folder | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `dpfilelist_outdated` | E | installed DpFileList lacks a target of this run that the bundled official DPFL lists (old layout, e.g. no `teams` slots) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Upgrade DpFileList, CLI names `upgrade-dpfl` |
| `cpk_name_unlisted` | E | CPK name in neither the installed nor the bundled DpFileList (a genuinely unknown name) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `cpk_slots_exhausted` | F | multi-CPK content does not fit the available slots at `cpk_part_max_size` (names the shortfall) | run aborted; the DPFL author adds slots or the cap is raised |
| `cpk_team_exceeds_cap` | F | a single team's compiled content is larger than `cpk_part_max_size` (impossible in practice at 3 GB) | run aborted; teams are never split across parts |
| `cpk_size_over_limit` | W | a single-CPK output exceeds `cpk_part_max_size` (valid for PES; the DLC repository will reject it) | none |
| `old_cpk_locked` | E | old CPK cannot be replaced (PES running) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Retry (deployment only) and Open output folder |
| `deploy_target_unwritable` | E | destination folder denies writes (typically elevation needed under `Program Files`) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Relaunch as administrator. Normally pre-empted by the live writability preflight (see `pipeline.md` "Post-processing") |
| `deploy_skipped_by_flag` | I | `--no-deploy` given (CLI) | staged CPKs promoted to `output/`; savefile step skipped; run is clean |
| `sideload_active` | I | sideload folder present and injected | none |
| `savefile_autodetected` | I | `savefile_path = auto` resolved a savefile under Documents\KONAMI (names the path; noted especially when several account folders existed and the newest was chosen) | none |
| `patch_written` | I | the aesthetics patch was written beside the output CPK (names the path and the teams it covers) | none |
| `savefile_missing` | W | aesthetics present but no savefile configured/found; the patch is the run's only savefile output | savefile step skipped; the message names the patch and the save editor's apply action |
| `savefile_skipped_pes_running` | W | savefile changes pending but PES is running (any output mode; the motivating case is Sider-mode iteration) | savefile step skipped; applied by the next compile with PES closed |
| `savefile_write_failed` | E | savefile could not be updated | deployment transaction fails and rolls back; compile artifacts remain available |

Deployment-stage errors above mean no partial installation: preflight failures leave installed
outputs untouched; failures after replacement starts roll back the deployment as a whole. A missing savefile
is the explicitly permitted warning-only case. A present savefile that cannot be read or whose
version differs from the compile target is an error, not `savefile_missing`; do not modify it or
publish a partially deployed run.

The `cpk_write_failed` and `output_commit_failed` consequences are required guarantees: partial
generated output is discarded and prior published output remains untouched. The corresponding open
questions choose the `.partial`/rename/rollback implementation, not whether these guarantees apply.

Fatal settings/environment problems (invalid PES version, unwritable output folder) are surfaced by
the settings UI and CLI argument validation before a run starts, not as pipeline messages.

---
