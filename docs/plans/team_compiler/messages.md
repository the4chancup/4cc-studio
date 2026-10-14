# 4cc Studio — Team compiler plan: Message catalog

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Message catalog

### Message structure

What exists of this is in `spec/team_compiler/messages.md` "Message structure".

- **Logs**: `issues.log` (Warning+) and `suggestions.log` (Info) are kept as disk artifacts,
  rendered from the same events. Display layers may group repeated (code, scope) pairs; the logs
  stay complete.
- **No blocking console prompts**: unknown team ID becomes the grid's inline-editable ID cell
  (including its reassignment confirmation — see the GUI section).

### Catalog

What exists of this is in `spec/team_compiler/messages.md` "Catalog".

**Export level**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `teams_list_read_only` | W | a teams-list write (ID cell, updater merge) failed because the data directory is not writable | write dropped; the in-memory list is unchanged (no elevation — see `pipeline.md` "Resolved decisions", "Teams list") |
| `export_disabled` | I | as the spec's row | omitted from the grid |
| `export_tag_missing` | E | as the spec's row | GUI: the row offers the two tags as buttons |
| `team_name_unknown` | E | as the spec's row | GUI: ID cell becomes editable for inline assignment |

**Player folders and shared model folders**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `players_txt_missing` | E | as the spec's row | `ParsedAestheticsExport` remains available to the Refs arranger for repair |
| `materials_toml_invalid` | E | a `materials.toml` or `*.materials.toml` file fails to parse | folder discarded |
| `material_file_missing` | E | a glTF file has no matched material file (no name-matched `*.materials.toml`, no catch-all `materials.toml`, no `.common` link to either) | folder discarded |
| `material_undefined` | E | a material name referenced by a glTF file has no definition in any matched material toml | folder discarded |
| `material_key_unknown` | W | a material toml entry has a key outside the schema (typo or newer-Studio field) | key ignored |
| `material_value_invalid` | E | a material toml value has the wrong type, is out of range, names an unknown family/role/state, or uses a native sampler name reserved by a canonical role | folder discarded |
| `material_texture_not_found` | E | a mesh-used glTF material's `base`, `""`-marked, or explicit texture stem cannot be resolved | folder discarded |
| `material_parameter_unused` | I | a shader parameter the target engine's shader does not define | parameter dropped for that engine |
| `material_entry_unused` | I | a material toml entry matches no material in the folder's glTF models | none |
| `materials_file_ambiguous` | I | multiple name-matched `*.materials.toml` files match one glTF; definitions layer least → most specific (intentional layered setups are ordinary usage) | none |
| `material_link_layered` | I | a `.common` material link and a local material file share a stem; the Common file layers below the local one | none |
| `gltf_embedded_image_ignored` | I | a glTF embeds texture bytes (stock Blender's default); textures are only ever read from the folder's files | embedded images ignored (once per model) |
| `gltf_bone_unknown` | E | a glTF skin joint has neither `PES_bone` metadata nor a name in the template skeleton | folder discarded |
| `settings_toml_name_shared` | W | `name` or `shirt_name` given in an autopilot export's folder mapped to multiple players | name applied to all of them |
| `fpc_strip_conflict` | W | `settings.toml` sets a key the folder's FPC preset also sets, to another value (for `skin_color`, a custom skin, 7, where the preset sets a preset skin; boots and gloves IDs aside: `settings_toml.md` resolves them) | FPC preset wins; keys ignored |
| `settings_value_out_of_range` | W | a `settings.toml` value inside its key's widest range but outside the compile target's (`settings_toml.md`, the template's ranges; context: the key, the value, the target's range) | the key's default written (`settings_toml.md`: a face type 0, a motion the template's value; a custom skin 7 as 1) |
| `settings_name_invalid` | W | a `name` or `shirt_name` string that does not fit the target version's field or character set (context: the key) | not written; the save's name stands |
| `fpc_inners_break_hiding` | W | `libs/fpc`'s interference rule on a player whose folder carries `fpc_on` (`libs/fpc.md`; context: the player); the lib's Error, a Warning here because a Studio Error would drop the folder | written as authored |
| `fpc_undershorts_break_hiding` | W | likewise, the lib's Error too | written as authored |
| `fpc_taping_shows_pieces` | I | likewise | written as authored |
| `fpc_gloves_checkbox_winter_gloves` | W | `libs/fpc`'s rule on any player: the gloves checkbox on with gloves ID 0, whatever the FPC marker (context: the player) | written as authored |
| `settings_model_id_conflict` | W | `settings.toml` sets `boots_id` or `gloves_id` for a category whose ID the compiler assigns, from the folder's standalone models or a link file (pre-Fox face-XML models assign none; context: the category) | the compiler-assigned ID wins; key ignored |
| `model_source_ambiguous` | E | duplicate model sources within the selected representation (target-native > glTF > convertible opposite); its input is two glTF files of one stem (`boots.glb` beside `boots.gltf`), so it is first reachable in Phase 7 | folder discarded (`DropFolder`) |

**XML/MTL content checks** (pre-Fox, plus `face_diff.xml` in Fox)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `xml_texture_path_missing` | E | sampler with no texture path and not auto-fillable (sampler not in the suffix mapping table) | folder discarded |

The three `mtl_state_*` checks other than `mtl_state_missing` also run on a glTF material's
`[prefox.states]` table when the target is pre-Fox (the format plan's schema mirrors the `.mtl`
states one-to-one; a glTF material with states absent just gets its family's state set, so
`mtl_state_missing` does not apply).

**Output stage and savefile** (Run scope)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `savefile_autodetected` | I | `savefile_path = auto` resolved a savefile under Documents\KONAMI (names the path; noted especially when several account folders existed and the newest was chosen) | none |
| `patch_written` | I | the aesthetics patch was written to the output folder (names the path and the teams it covers) | none |
| `patch_write_failed` | F | the aesthetics patch could not be written (context: the path, the error) | as `output_commit_failed`: run aborted; prior published outputs, the patch included, remain untouched (`AbortRun`) |
| `savefile_missing` | W | the patch holds entries (Fox: motions and resolved names; pre-Fox: aesthetics) but no savefile is configured/found; the patch is the run's only savefile output | savefile step skipped; the message names the patch and the save editor's apply action |
| `savefile_skipped_pes_running` | W | savefile changes pending but the compile target's PES is running (a PES of another version does not count; any output mode; the motivating case is sideload-mode iteration) | savefile step skipped; applied by the next compile with PES closed |
| `savefile_write_failed` | E | savefile could not be updated | deployment transaction fails and rolls back; compile artifacts remain available |
| `savefile_invalid` | E | the configured or found savefile cannot be read, or its version is not the compile target's (context: the path, the version) | the save is not modified; as `savefile_write_failed` |
| `savefile_team_missing` | E | a team of the patch is not in the savefile (context: the team) | that team's entries are not applied; the rest is |
| `savefile_player_not_stripped` | W | Fox: a compiled player's record in the local save has its appearance id set (a save never stripped, or an id the game's Edit mode restored), so the game shows his record, not the compiled rows (context: the players) | reported; the save editor strips them (`strip-and-seed` for a save never stripped, `check-stripped --strip-again` otherwise) |
| `pes_folder_not_found` | E | as the spec's row | savefile step skipped |
| `dpfilelist_missing` | E/W | as the spec's row | E: savefile step skipped |
| `dpfilelist_not_official` | W | as the spec's row | the GUI offers **Upgrade DpFileList**, which asks before replacing the list |
| `dpfilelist_outdated` | E | as the spec's row | savefile step skipped; GUI offers Upgrade DpFileList |
| `cpk_name_unlisted` | E | as the spec's row | savefile step skipped |
| `old_cpk_locked` | E | as the spec's row | savefile step skipped; GUI offers Retry (deployment only) and Open output folder |
| `deploy_target_unwritable` | E | as the spec's row | savefile step skipped; GUI offers Relaunch as administrator. Normally pre-empted by the live writability preflight (see `pipeline.md` "6. Post-processing") |
| `deploy_skipped_by_flag` | I | as the spec's row | savefile step skipped |

A missing savefile
is the explicitly permitted warning-only case. A present savefile that cannot be read or whose
version differs from the compile target is an error, not `savefile_missing`; do not modify it or
publish a partially deployed run.
