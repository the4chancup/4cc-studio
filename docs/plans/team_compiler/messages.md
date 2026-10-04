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
- **Message text lives in one table** keyed by ID (template + remediation hint; until the help
  window and the GUI log need the text, in Phase 8, `messages.rs` holds each code's severity
  only, and the CLI prints the code with its context fields), not scattered
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
- **What makes a finding an Error.** A file that cannot be produced, or whose compiled result is
  unusable or visibly compromised in the game (a mesh that does not render, geometry that lags
  the match, a texture the model names and nobody supplies), is an Error and drops its folder. A
  file that compiles to something usable but may hide an issue is a Warning, and the folder
  compiles. The format crates' `check` severities follow the same rule, so the deep pass maps a
  format finding by its severity: Error is `DropFolder`, Warning and Info are `Keep`. A format
  Error that a census shows on models the game renders fine is a wrong severity in the format
  crate, fixed there, not an exception here.
- **`pass_through`** overrides only eligible content-level `DropFile`/`DropFolder` dispositions to
  keep-with-flag, as in Red, while retaining the original severity. Not eligible: findings whose
  content cannot exist (failed conversion/packing, unproducible logos), unused content
  (`shared_folder_orphaned`), contradictory directives that leave no value to keep
  (`fpc_conflict`, `kit_layout_conflict`, `shared_link_duplicate`), kit identities that cannot be kept (`kit_slot_duplicate`,
  `kit_folder_invalid`), ambiguous model-source or texture identity/conflicts, foundational
  unsafe-path/ambiguous-roster/required-metadata failures, geometry far from the origin
  (`vertex_too_far_from_origin`: the lag it causes persists for the whole matchday),
  `DropSlot`, `DropExport`, `AbortRun`, and environment-level dispositions. Written errored content
  receives a distinct `DoneWithErrors` outcome rather than an ordinary red error state.
- **No blocking console prompts.** Red's prompts are replaced, not kept: unknown team ID becomes the
  grid's inline-editable ID cell (including its reassignment confirmation — see the GUI section),
  the one standing consent (overwriting `dt00_x64.cpk` with the Fox referee marker) is gone with
  the write it guarded, and retry-on-locked-file becomes a plain Error. The CLI never prompts: all of these are hard errors there.
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
| `export_extract_failed` | E | archive cannot be extracted/parsed, or its listing is refused (a path escaping the root, two names that fold to one; context: the path) | export skipped (`DropExport`) |
| `no_exports_found` | W | discovery finds no export folder, `.zip` or `.7z` in the exports folder (Run scope; context: the folder) — what a fresh install's created `exports/` reports until the user fills it | none; nothing is checked or compiled |
| `export_disabled` | I | root `NO_USE` / `NO_USE.txt` marker disables this source | export skipped; omitted from the grid and from duplicate-ref detection |
| `export_identified` | I | the export's identity resolved (context: `team` and `id`, or `team` = `referees`) | none; how the CLI reports which team an export is |
| `source_read_failed` | E/F | a pinned source entry cannot be read | optional root file: `DropFile`; folder/task producer: `DropFolder`; required export metadata or unusable source: `DropExport`; output/global source invariant: `AbortRun` |
| `source_changed_during_run` | F | a file a task read is gone, or its size or modified time is no longer what the export's listing gave (an archive: the archive file's); on the export, context `path` | run aborted, the staging output discarded, nothing deployed (`AbortRun`) |
| `template_override_unreadable` | E/F | a template override file exists but cannot be read (embedded defaults cannot be missing) | folder-local injected template: `DropFolder`; referee-export template: `DropExport`; global CPK/bin template: `AbortRun` |
| `template_override_active` | I | a `templates/` override file is shadowing an embedded template/fallback-bin resource | override used; reported per file per run |
| `export_balls_skipped` | I | export name's first word is `balls` (balls exports belong to the Balls compiler) | export skipped (`DropExport`) |
| `multiple_ref_exports` | E | more than one non-disabled `refs`-prefixed export found at discovery | every conflicting export skipped (`DropExport` per row, plus a Run-scoped summary) |
| `duplicate_aesthetics_export` | E | multiple exports resolve to the same team ID; found once every export's identity is resolved, so `check` reports it too. An export validation dropped does not count, whether before its identity resolved or after (an exhausted ID pool): it is not compiled anyway (context: `id`, the team ID; `exports`, the conflicting sources' file names in export order) | all conflicting exports skipped (`DropExport`, on each) |
| `boots_id_pool_exhausted` | E | planned player-exclusive/shared boots outputs exceed the team's permanent ID block | export skipped (`DropExport`) |
| `gloves_id_pool_exhausted` | E | planned player-exclusive/shared gloves outputs exceed the team's permanent ID block | export skipped (`DropExport`) |
| `export_empty` | E | no usable content found at root | export skipped |
| `content_not_yet_compiled` | E | Phase 3 only (withdrawn in Phase 4): what an export would output holds content `compile` cannot compile yet (`team_compiler/README.md` "Acceptance", Phase 3 scope; context: what) | export skipped by `compile` (`DropExport`); `check` unaffected |
| `nested_folders_fixed` | W | content found nested one level down (exactly one usable root) | auto-fixed |
| `nested_root_ambiguous` | E | several nested child folders are usable export roots | export skipped (`DropExport`) |
| `nested_root_conflict` | E | loose root file collides with a flattened nested file at the same virtual path | export skipped (`DropExport`) |
| `export_tag_missing` | E | a team export's name has neither `Full` nor `Midcup` as its second word (context `name`: the export's name) | export skipped; the hint says to rename it `<team> Full …` or `<team> Midcup …` (GUI: the row offers the two tags as buttons) |
| `team_name_unknown` | E | canonical `team_name` not in `teams_list.txt` | export skipped (GUI: ID cell becomes editable for inline assignment) |
| `team_id_out_of_range` | E | resolved ID outside 701–920 | export skipped |
| `teams_list_read_only` | W | a teams-list write (ID cell, updater merge) failed because the data directory is not writable | write dropped; the in-memory list is unchanged (no elevation — see `pipeline.md` "Resolved decisions", "Teams list") |
| `team_colors_missing` | I | no root `colors.txt` (it is optional) | the team's colors in `TeamColor.bin` are left as they are |
| `color_entry_invalid` | W | a `colors.txt` line that does not parse as exactly one color (two colors on one line, the old Team Note kit entry, included), or a valid line past the file's color count (two for a kit, four for the team). Grammar: "Root files" (Colors) in `aesthetics_export/player_folders.md` | the line skipped |
| `root_file_unexpected` | W | unknown file or folder at the export root (a folder other than the content folders — a stale `wrapper/` beside a usable root included), or a file directly inside `Players`, `Kits`, `Faces`, `Boots` or `Gloves`, which hold only folders (`Players/players.txt`) | file or folder ignored |
| `portrait_conflict` | E | a slot with a portrait in its player folder and one in `Portraits/` whose bytes differ (the deep pass compares the two files; byte-identical files are one portrait and no finding) | export skipped: the compiler cannot tell which one the manager means |
| `notes_found` | I | non-empty valid root `notes.txt` present | collected into teamnotes.txt by `compile` |
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
| `link_target_dropped` | E | a shared-folder or `.common` link names a target the structure pass dropped, by its effective disposition (context: the link, the target and its finding) | folder discarded (`DropFolder`); not reported when `pass_through` keeps the target; never itself pass-through-eligible |
| `shared_link_duplicate` | E | player folder has more than one shared link for any category (face, boots, or gloves) | folder discarded (`DropFolder`) |
| `link_combined` | I | link file plus local models for the same category; the shared models become parts of the player's own set (Fox: mesh-merged, own ID) | none |
| `shared_folder_orphaned` | W | shared folder referenced by no player | folder skipped |
| `settings_toml_invalid` | E | optional `settings.toml` is not UTF-8 text or fails to parse, found by the deep pass (context: the file, the error) | file ignored and existing savefile values preserved (`DropFile`, not pass-through-eligible); models continue |
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
| `kit_variant_missing` | W | a folder's texture variant set (`pants_kit1`, `pants_kit3`) has no variant for a kit number the export defines; on the folder, once per set and number (context: `texture`, the set's reference as `pants_kitN`; `kit`, the number; `copied`, the lowest variant's stem) | lowest existing variant copied into the gap |
| `kit_variant_model_fox` | W | per-kit model variants (`*_kit1`, `*_kit2`, …) on a Fox target, which has no model-path indirection; on the folder, once per set (context: `model`, the set's reference as `pants_kitN.fmdl`; `used`, the lowest variant's file name) | lowest variant used, others ignored |
| `common_link_missing` | E | a `.common` link — model, material file or texture — names a file missing from the Common folder (context: the link file, whose name shows its kind, and the Common path looked for) | folder discarded |
| `settings_toml_name_shared` | W | `name` given in a folder mapped to multiple players | name applied to all of them |
| `fpc_conflict` | E | both `fpc_on` and `fpc_off` present in a player folder | folder discarded |
| `fpc_strip_conflict` | W | `settings.toml` strip keys conflict with the folder's FPC marker | FPC preset wins; keys ignored |
| `settings_unknown_other_version` | I | `settings.toml` holds an `[appearance.unknown.pesNN]` table for a version other than the compile's target (context: the table's version) | those bits stay at their default (0) for this compile; the table is kept in the file |
| `settings_model_id_conflict` | W | `settings.toml` sets `boots_id` or `gloves_id` for a category the folder's own models or a link file already provide (context: the category) | the compiler-assigned ID wins; key ignored |
| `fmdl_name_invalid` | E | Fox: FMDL in a boots/gloves shared folder not resolving to that category's allowed names | folder discarded |
| `fmdl_fcl_hair_fallback` | I | Fox: arbitrary-named FMDL treated as a face model, routed into the `fcl_hair.fmdl` merge (Red's single-file fallback, generalized) | none |
| `fmdl_merged` | I | Fox: multiple models resolve to the same allowed name; meshes merged into one FMDL (alphabetical source order) | none |
| `merge_material_conflict` | E | merged parts define the same material name differently (Fox merge or pre-Fox `ingame_face` merge) | that package (face, boots or gloves) left out (`DropFolder`); the folder's other packages and its textures are still packed, so the player is built incomplete (beside a blank face, no head over the stock body): the Error on the player's folder is what tells the member the folder is not usable until the conflict is fixed, and the help says so |
| `skl_merge_conflict` | E | merged parts have incompatible skeletons (Fox compares effective SKL content; pre-Fox IR merge compares bone transforms) | that package left out (`DropFolder`), as for `merge_material_conflict` |
| `skl_no_slot` | W | Fox: custom `.skl` paired with a model resolving to `face_high`/`hair_high`/`oral` (no skeleton slot exists for those) | SKL ignored (no-op file) |
| `face_file_not_used` | I | Fox: a face file (`face_diff.bin`, `face_diff.xml`, `fcl_hair_sim.fclo`) in a player folder with no face model, under `ingame_face` or not; a blank face folder always takes the bundled face diff (context: the file) | file not read |
| `ingame_face_explicit_face_model` | E | `ingame_face` combined with explicit face models (`face_high`/`hair_high`/`oral`, in any supported source format) or a shared face link, which require the suppressed face folder | player folder discarded (`DropFolder`) |
| `shared_texture_conflict` | E | a player's tasks produce the same common-texture destination with different bytes | losing task discarded (`DropFolder`; canonical winner face > boots > gloves); identical bytes deduplicate |
| `merged_texture_conflict` | E | two merge-copied parts within one output model produce the same texture destination with different bytes (no canonical winner exists inside one model) | folder discarded (`DropFolder`); identical bytes deduplicate |
| `model_source_ambiguous` | E | duplicate model sources within the selected representation (target-native > glTF > convertible opposite); its input is two glTF files of one stem (`boots.glb` beside `boots.gltf`), so it is first reachable in Phase 7 | folder discarded (`DropFolder`) |
| `model_gltf_unsupported` | E | the folder's selected representation is glTF (no target-native source beside it), which the compiler does not read until Phase 7; never compiled from the opposite native format instead (context: the file) | folder discarded (`DropFolder`); not pass-through-eligible |
| `model_conversion_failed` | E | selected model cannot be converted to the target format | folder discarded (`DropFolder`) |
| `vertex_too_far_from_origin` | E | a model has a vertex more than 5000 units from the origin (the format crate's `fmdl_vertex_far_from_origin` / `model_vertex_far_from_origin`, run on every model the compiler processes: native sources as loaded, converted and glTF sources in their target-format form, before any merge; context: model file, offending vertex count). Text: "Vertex too far away, it will cause persistent lag for the whole matchday" | folder discarded (`DropFolder`); not pass-through-eligible |
| `folder_pack_failed` | E/F | a task cannot build its packed batch | folder/task: `DropFolder`; output-writer/global: `AbortRun` |
| `model_name_invalid` | E | pre-Fox: `.model` not in the allowed names for its category | folder discarded |
| `xml_broken` | E | XML fails to parse (with line/column) | folder discarded |
| `mtl_broken` | E | MTL fails to parse (with line/column) | folder discarded |
| `edithair_unsupported` | E | `face_edithair.xml` / `hair.xml` present | folder discarded |
| `file_type_disallowed` | E/I | extension not in the mode's allowlist (E if `strict_file_type_check`, else I) | folder discarded / kept |
| `fmdl_texture_not_found` | E/W | Fox: a texture one of the model's meshes uses is supplied by nobody: its stem resolves to no file of the folder, and its path names the team's Common output, where neither the export's `Common/` nor an installed CPK holds it (`pipeline.md` "Resolved decisions", "A texture a model names must exist"; context: the model file, the texture path). W when no install could be read to look, since the texture may be there | folder discarded (`DropFolder`); kept when W |

**Model checks** (the format crates' `check`, run by the deep pass on every native model as it
is loaded, whatever the target, and from the conversion steps on, on each converted model in its
target form. A finding keeps the format crate's code and its severity, which "What makes a
finding an Error" sets; the one renamed is the far vertex, which both formats report as
`vertex_too_far_from_origin`, above. An Error discards the folder and is
pass-through-eligible, since the file can still be packed as it is; a Warning or an Info changes
nothing. One finding per file and code, however many meshes trip the rule. Context: the model
file, and the number of items that tripped it over the whole file: faces, vertices or bone
slots for a rule about a mesh's contents, and the meshes, materials or bones themselves for a
rule about one of them (two empty meshes count 2). A finding on a
`Common/` model acts on that file, and the players linking it follow `link_target_dropped`)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `model_broken` | E | a model file (`.fmdl` or `.model`) does not parse (context: the file, the reader's error) | folder discarded; not pass-through-eligible (it cannot be processed) |
| `fmdl_mesh_over_bone_limit`, `fmdl_mesh_over_vertex_limit`, `fmdl_mesh_over_face_limit` | E | Fox: a mesh exceeds a hard FMDL limit; the game cannot load it | folder discarded |
| `fmdl_face_index_out_of_range` | E | Fox: a face names a vertex that does not exist | folder discarded |
| `fmdl_mesh_unassigned` | E | Fox: no mesh group lists the mesh, so the written file would not contain it | folder discarded |
| `fmdl_bone_slot_out_of_range` | W | Fox: a weighted bone index points past the mesh's bone group; the game ignores that weight (Konami's own models do this) | none |
| `fmdl_mesh_empty` | W | Fox: a mesh with no faces | none |
| `fmdl_duplicate_bone_name` | W | Fox: two bones of one name; merges and `.skl` matching get ambiguous | none |
| `fmdl_weights_not_normalized` | I | Fox: a vertex's weights sum to neither 255 nor 0 | none |
| `fmdl_material_unused` | I | Fox: a material instance no mesh uses | none |
| `model_mesh_over_bone_limit`, `model_mesh_over_vertex_limit`, `model_mesh_over_face_limit` | E | pre-Fox: a mesh exceeds a hard `.model` limit; the game cannot load it | folder discarded |
| `model_face_index_out_of_range` | E | pre-Fox: a face of any LOD level names a vertex that does not exist | folder discarded |
| `model_bone_slot_out_of_range` | W | pre-Fox: a weighted bone index points past the mesh's bone group | none |
| `model_mesh_empty` | W | pre-Fox: a mesh with no faces | none |
| `model_duplicate_bone_name` | W | pre-Fox: two bones of one name | none |
| `model_lod_record_mismatch` | W | pre-Fox: the LOD record's level count disagrees with the meshes' LOD levels | none |
| `model_weights_not_normalized` | I | pre-Fox: a vertex's weights do not sum to 1 | none |
| `model_degenerate_face` | I | pre-Fox: a face uses one vertex twice | none |
| `model_material_unused` | I | pre-Fox: a material name no mesh uses | none |

The `.mtl` checks (`mtl_material_duplicate`, `mtl_state_*`) are in "XML/MTL content checks"
below; the deep pass runs them on every `.mtl`. `model_material_undefined` needs to know which
`.mtl` a model is paired with, which the pre-Fox face steps decide, so it is theirs.

**Textures** (file-scoped; in model folders the folder fails, and so does a kit, whose config
names its textures; elsewhere (`Common/`, portraits) the file is dropped; the logo sources are the
exception — main and `_small` fail as one atomic producer of the game's three sizes. A file is
`texture_type_mismatch` when its bytes open with the signature of another accepted format than
its extension's; TGA has no signature, so a `.tga` is a mismatch only when it opens with
another's. An uncompressed texture is no finding, a kit's included: it is encoded like any
raster source (`libs/dds_convert.md` "Passthrough"), which is what PES 15–17's crash on
uncompressed kits needed)

The deep pass reports `texture_type_mismatch`, `texture_too_small`, `texture_not_pow2` and
`kit_texture_too_big` from each texture's header (`probe` in `libs/dds_convert.md`), without
decoding it, so `check` reports them and the folder is dropped before any ID is planned for
it. A mismatched file gets that one finding: its header is another format's, so its size is
not read. `texture_type_mismatch` is not pass-through-eligible, because the file cannot be
converted as the format its name declares; the three size findings are eligible: the texture
converts, and what the game makes of it is the member's risk. A texture whose header cannot
be read gets no finding here; converting it fails its task (`folder_pack_failed`), as does
one whose pixel data is cut short, which no header shows. `texture_codec_unsupported` is
conversion's own finding, reported when the file is compiled, not by `check`.

A portrait (a `Portraits/` file or a player folder's `portrait.*`) is checked the same way and
dropped alone, its player folder keeping everything else. Its sides must be powers of two
whatever its level count and the target. A logo source is the one texture the deep pass
decodes in full (an export has at most two, and nothing else can tell that the game's three
sizes can be made from it): one that does not decode is `logo_file_invalid`, not
pass-through-eligible, and the logo goes as one unit.

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `texture_too_small` | E | a side under 4 pixels, one block | discarded |
| `texture_not_pow2` | E | a side that is not a power of two, on a portrait (any target), or on a Fox target on a texture that carries a mip chain: a raster source always (its chain is generated), a DDS or FTEX source with more than one level (a single-level one passes at any size). Not reported on a kit's main texture, where `kit_texture_too_big` covers it | discarded |
| `texture_not_div4` | E | pre-Fox compressed texture not divisible by 4 | discarded |
| `kit_texture_too_big` | E | a kit's main texture (`kit`, its own or the one inherited from `all/`) wider or taller than 2048 pixels, or with a side that is not a power of two; any target | discarded |
| `texture_type_mismatch` | E | header doesn't match extension (renamed, not resaved) | discarded |
| `texture_codec_unsupported` | E | codec not convertible in-process | discarded |
| `texture_stem_conflict` | E | two image files with the same stem in one lookup namespace, whatever their extensions: a model folder with its reserved subfolders (`hair.dds` beside `common/hair.dds`, or `hair.png`, or a texture link `hair.png.common`, which counts as a file of its linked name), `Common/` (`hair.dds` beside `hair.png`), a kit folder, or `Kits/all/` (`kit.png` beside `kit.dds`; a kit's own file overriding an `all/` file of its stem is not a conflict), or `Portraits/` (`player_03.dds` beside `player_03.png`) | folder discarded (the kit; for `all/`, `all/` itself, so no kit inherits from it); in `Portraits/` and `Common/`, both files (a player linking a dropped Common file follows `link_target_dropped`) |

**XML/MTL content checks** (pre-Fox, plus `face_diff.xml` in Fox)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `xml_texture_path_missing` | E | sampler with no texture path and not auto-fillable (sampler not in the suffix mapping table) | folder discarded |
| `mtl_texture_not_found` | E/W | texture (by stem) used by one of the model's meshes supplied by nobody: not in the folder, and for a Common path neither in the export's `Common/` nor in an installed CPK (the rule of `fmdl_texture_not_found`, W included) | folder discarded; kept when W |
| `mtl_texture_unused_missing` | I | texture referenced only by materials no mesh uses | none |
| `xml_root_tag_invalid` | E | root tag is not `<config>` | folder discarded |
| `xml_model_type_missing` | E | `<model>` without `type` | folder discarded |
| `xml_model_path_missing` | E | `<model>` without `path` | folder discarded |
| `xml_model_not_found` | E | `path` (or `material`) is a `./` or Common reference whose file is missing from the export | folder discarded |
| `xml_common_path_invalid` | E | Common reference without 3-char subfolder | folder discarded |
| `xml_oral_prefix_missing` | E | PES16 target: model file name starting with none of `face_high_`/`hair_high_`/`oral_` | folder discarded |
| `xml_dif_conflict` | E | two sources for one datum in one folder: the xml carries a `<dif>` and the folder also has `face_diff.xml`, or (Fox) the folder has both `face_diff.bin` and `face_diff.xml` | folder discarded |
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
| `face_diff_invalid` | E | `face_diff.xml` is not base64 text or a `<dif>` holding it, or its decoded bytes, or a `face_diff.bin`, are not a face diff (`player_folders.md` "`face_diff.xml`"; context: the file, the reason) | folder discarded |
| `mtl_material_duplicate` | E | material listed twice | folder discarded |
| `mtl_state_invalid` | E | `ztest` ≠ 1 / `blendmode` ∉ {0,1} / `alphablend` ∉ {0,1} | folder discarded |
| `mtl_blendmode_nonzero` | W | `blendmode` = 1 (works, not recommended) | none |
| `mtl_state_missing` | I | state names absent, defaults used | none |
| `mtl_state_nonrecommended` | I | `alphablend`/`zwrite` combination not recommended | none |
| `mtl_state_unknown` | I | a state name outside the seven the schema knows (`shadowcaster` in 18 of Konami's files) | kept verbatim |
| `model_material_undefined` | E | a material name a `.model` uses has no entry in the `.mtl` it is paired with; the mesh would render with the game's fallback | folder discarded |

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
| `kit_all_file_ignored` | W | `all/` holds something other than kit textures (`config.toml`, `colors.txt`, an `icon_<N>` marker, anything else) | file ignored |
| `kit_all_unused` | W | `all/` present but no kit folder to inherit from it | — |
| `kit_config_generated` | I | no `config.toml`; generated from the template, which carries the FPC values whatever the team's FPC status | auto-fixed |
| `kit_config_invalid` | E | `config.toml` is not UTF-8 text or fails to parse or validate (ranges, cross-field constraints), found by the deep pass (context: the file, the error) | kit discarded (`DropFolder`, not pass-through-eligible) |
| `kit_config_version_clamped` | W | a supplied config's field doesn't fit the target PES version's encoding (e.g. Name Y > 16 before PES 21), found by the deep pass (context: the field, the value, the version's maximum) | value clamped when the config is emitted |
| `kit_config_fpc_adjusted` | I | team kit-FPC status is On but a config lacks the FPC values — supplied configs and unexported slots' base entries alike, GK kit included | auto-fixed (values written; FPC values are never auto-reverted) |
| `kit_config_fpc_unpatched` | W | team kit-FPC status is On but an unexported kit slot has no base entry or config to patch | slot left alone; the team needs a kit export |
| `kit_placeholder` | I | the kit's effective textures lack `kit.dds` (an empty folder included); the bundled checkerboard stands in | placeholder kit emitted: checkerboard texture, template config unless supplied, UniColor entry per the colors fallback |
| `kit_colors_derived` | I | kit `colors.txt` missing, or present but yielding fewer than two valid colors; menu colors extracted from the kit's own main texture | auto-fixed |
| `kit_colors_missing` | W | no usable `colors.txt` and no own main texture to derive from (placeholder kits without a `colors.txt` always) | the magenta/black "no colors chosen" pair is written, so the gap shows in the game menus |
| `kit_icon_invalid` | W | a kit's `icon_<N>` marker is numbered above 23, or the kit holds two or more (each is reported) | default icon (3) used |
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
| `logo_fit_applied` | I | a non-square source was made square; names the file and the mode (`fit` by default, or the file's tag) (context: `file`, `mode`) | — |
| `logo_upscaled` | W | a source is smaller than its largest target (512² for main, 128² for small): its long side for `fit`, its short side for `crop` and `stretch` (context: `file`, `size` as `300x300`, `target`) | emitted upscaled |
| `collar_id_invalid` | E | collar filename doesn't parse as `collar_<ID>` (zero padding optional), the ID is not a stock collar of the target version (each version's set counted in its install's base data CPK, `dt35`'s `nocloth` collars: PES 15: 1-101 and 901-904; PES 16: 1-105 and 901-904; PES 17 and 18: 1-116 and 901-916; PES 19: 1-124 and 901-916; PES 20: 1-127 and 901-913; PES 21: 1-131 and 901-913) | collar file discarded |
| `collar_id_conflict` | E | the stock collar ID is already claimed: by another export earlier in this run (canonical export order), or by the suite itself, which reserves 105 (FPC) and 77 (the referees' marker, `blue_port.md` "Referee export processing") (context: the claimant, an export's name or `FPC` / `referees`) | the collar is discarded and its team's configs are not rewritten |
| `common_file_disallowed` | E/I | as `file_type_disallowed`, Common scope | files discarded / kept |

**Referees** — referee exports use the same player-folder format (see `blue_port.md` "Referee export processing"),
so all player-folder, texture, and XML/MTL messages above apply as-is. The referee marker needs
no code of its own on a refs export: on both engines it is a model on the referees' reserved
collar and a texture in their Common output, both inside the refs CPK (`blue_port.md` "Referee
export processing"), so its failures are the texture codes above, on `ref_marker.dds`. Red's
injection into the system `dt00_x64.cpk`, and with it `ref_marker_needs_consent` and
`dt00_write_failed`, is gone. The one addition guards the reserved collar on regular teams:

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `kit_collar_reserved` | E | a regular team's kit whose effective collar or winter collar, after FPC reconciliation and custom-collar rewriting, is 77, the referees' reserved collar (context: the kit, the field) | kit discarded: its players would wear the referee marker |

**Output stage and savefile** (Run scope)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `duplicate_path` | W/E | a task's entry, or a bin, at the path of an `overrides/` file (Run scope; context: the path). The folder and export forms (two tasks claiming one path) have no trigger today and are not built (`pipeline.md` "3. Per-model-folder parallel steps") | override: the `overrides/` file wins, the entry is left out (`Keep`, W); folder collision: losing folder blocked (`DropFolder`, E); whole-export collision: losing export blocked (`DropExport`, E) |
| `cpk_write_failed` | F | incremental CPK writing fails | run aborted and partial CPK discarded (`AbortRun`) |
| `output_commit_failed` | F | a completed CPK/tree cannot be atomically committed to its final output path | run aborted; prior published outputs remain untouched (`AbortRun`) |
| `teamnotes_write_failed` | E | `teamnotes.txt` cannot be written, or a previous one cannot be removed, in the output folder (context: `path`, `error`) | the run's CPK stays in place; the notes file is left as it was |
| `uniparam_compile_failed` | F | UniformParameter compilation failed (output would crash PES) | run aborted |
| `pes_folder_not_found` | E | `pes_folder_path` invalid at deployment time (includes the no-PES-install machine) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `pes_version_mismatch` | W | `pes_folder_path` exists but holds no `PES20{pes_version}.exe` — a different-version exe suggests a wrong `pes_version` setting (Red's suppressible console notice becomes a plain per-run warning, dropping `state/ver_mismatch_warned.txt`; the GUI also surfaces this live via the version selector's red/yellow state — see the core plan's Sidebar) | none; deployment proceeds |
| `dpfilelist_missing` | E | no `DpFileList.bin` in download folder | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `dpfilelist_not_official` | W | the installed DpFileList's entries differ from the bundled official list's (missing entries, entries the official list lacks, or their order), while every target of this run is listed (context: the entries missing, the entries not official, whether the order differs) | none: the run compiles and deploys; the GUI offers **Upgrade DpFileList** and the CLI names `upgrade-dpfl`, which ask before replacing the list |
| `dpfilelist_cpk_missing` | W | the installed DpFileList names CPKs with no file in the download folder, other than this run's own target (context: the files). The game then rejects the whole download folder and runs vanilla | none: the run compiles and deploys; `upgrade-dpfl` writes a placeholder for each |
| `dpfilelist_outdated` | E | installed DpFileList lacks a target of this run that the bundled official DPFL lists (old layout, e.g. no `teams` slots) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Upgrade DpFileList, CLI names `upgrade-dpfl` |
| `cpk_name_unlisted` | E | CPK name in neither the installed nor the bundled DpFileList (a genuinely unknown name) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `cpk_slots_exhausted` | F | multi-CPK content does not fit the available slots at `cpk_part_max_size` (names the shortfall) | run aborted; the DPFL author adds slots or the cap is raised |
| `cpk_team_exceeds_cap` | F | a single team's compiled content is larger than `cpk_part_max_size` (impossible in practice at 3 GB) | run aborted; teams are never split across parts |
| `cpk_size_over_limit` | W | a single-CPK output exceeds `cpk_part_max_size` (valid for PES; the DLC repository will reject it) | none |
| `old_cpk_locked` | E | old CPK cannot be replaced (PES running) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Retry (deployment only) and Open output folder |
| `deploy_target_unwritable` | E | destination folder denies writes (typically elevation needed under `Program Files`) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Relaunch as administrator. Normally pre-empted by the live writability preflight (see `pipeline.md` "Post-processing") |
| `deploy_skipped_by_flag` | I | `--no-deploy` given (CLI) | staged CPKs promoted to `output/`; savefile step skipped; run is clean |
| `overrides_active` | I | the data directory's `overrides/` folder holds at least one file, each injected into the CPK at its relative path (`compile` only, Run scope; context: the folder, the file count) | none |
| `bin_source` | I | which installed CPK supplied a working bin in the DpFileList walk, or `bundled` when the embedded base was used (context: the bin, the CPK or `bundled`) | none |
| `bin_header_repaired` | W | a working `TeamColor.bin` or `UniColor.bin` held records whose header was not their position's: the team ID, or in `TeamColor.bin` the color count. One finding per bin (context: the bin, the teams). It marks a corrupt installed bin, whatever wrote it, so the cause can be looked for | the headers are rewritten; each record's other bytes are kept, so those teams' colors may be wrong until their exports are compiled again |
| `savefile_autodetected` | I | `savefile_path = auto` resolved a savefile under Documents\KONAMI (names the path; noted especially when several account folders existed and the newest was chosen) | none |
| `patch_written` | I | the aesthetics patch was written beside the output CPK (names the path and the teams it covers) | none |
| `savefile_missing` | W | aesthetics present but no savefile configured/found; the patch is the run's only savefile output | savefile step skipped; the message names the patch and the save editor's apply action |
| `savefile_skipped_pes_running` | W | savefile changes pending but PES is running (any output mode; the motivating case is sideload-mode iteration) | savefile step skipped; applied by the next compile with PES closed |
| `savefile_write_failed` | E | savefile could not be updated | deployment transaction fails and rolls back; compile artifacts remain available |

Deployment-stage errors above mean no partial installation: preflight failures leave installed
outputs untouched; failures after replacement starts roll back the deployment as a whole. A missing savefile
is the explicitly permitted warning-only case. A present savefile that cannot be read or whose
version differs from the compile target is an error, not `savefile_missing`; do not modify it or
publish a partially deployed run.

The `cpk_write_failed` and `output_commit_failed` consequences are required guarantees: partial
generated output is discarded and prior published output remains untouched. The corresponding open
questions choose the `.partial`/rename/rollback implementation, not whether these guarantees apply.

Fatal settings/environment problems (invalid PES version, unwritable output folder, a missing
exports folder) are surfaced by the settings UI and CLI argument validation before a run starts,
not as pipeline messages. Their text is a sentence written for the user, naming the path and what
to do, never a bare OS error (`The system cannot find the path specified. (os error 3)` is what a
missing exports folder printed in Phase 3's shell slice). A missing exports folder reads
`the exports folder <path> does not exist: create it and put your exports inside, or set
exports_folder_path in <settings file> to the folder that holds them`; when the path came from the
command line, `the exports folder <path> given on the command line does not exist`. It is a
configuration error (exit code 2). The relative default is created instead (`settings.md` "Path
resolution"); when it cannot be, the run is refused naming the path, as an unwritable output folder
is (exit code 3).

---
