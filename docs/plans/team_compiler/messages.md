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
| `model_source_ambiguous` | E | duplicate model sources within the selected representation (target-native > glTF > convertible opposite); its input is two glTF files of one stem (`boots.glb` beside `boots.gltf`), so it is first reachable in Phase 7 | folder discarded (`DropFolder`) |

**XML/MTL content checks** (pre-Fox, plus `face_diff.xml` in Fox)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `xml_texture_path_missing` | E | sampler with no texture path and not auto-fillable (sampler not in the suffix mapping table) | folder discarded |
| `mtl_common_path_invalid` | E | a `.mtl` sampler path below `model/character/uniform/common/` with a subfolder that is not three characters long (`common/o/env.dds`): `xml_common_path_invalid`'s rule for a `.mtl`, since the team's Common is written under its three-digit ID and such a path reaches nothing (6.12c: 16 old /o/ and VG exports name their team that way; one rule for both files, which today send the folder `face.xml` refuses on to a texture the game never finds) | folder discarded |

The three `mtl_state_*` checks other than `mtl_state_missing` also run on a glTF material's
`[prefox.states]` table when the target is pre-Fox (the format plan's schema mirrors the `.mtl`
states one-to-one; a glTF material with states absent just gets its family's state set, so
`mtl_state_missing` does not apply).

**Output stage and savefile** (Run scope)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `dpfilelist_not_official` | W | as the spec's row | the GUI offers **Upgrade DpFileList**, which asks before replacing the list |
| `dpfilelist_outdated` | E | as the spec's row | GUI offers Upgrade DpFileList |
| `old_cpk_locked` | E | as the spec's row | GUI offers Retry (deployment only) and Open output folder |
| `deploy_target_unwritable` | E | as the spec's row | GUI offers Relaunch as administrator. Normally pre-empted by the live writability preflight (see `pipeline.md` "6. Post-processing") |
