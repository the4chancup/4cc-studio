# 4cc Studio — Savefile plan: Data model

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Player settings model (settings.toml)

What exists of this is in `spec/pes_savefile/model.md` "Player settings model (settings.toml)".

**Planned for Phase 5**, landed in the crate except its consumers (decision entry "aesthetics
travel in the database tables"), which stay open:

- The Team compiler resolves `NameSetting::FromFolder` (`name = true` in a player's file) to
  `Explicit` by its folder rules before `apply`; this crate applies whatever the resolved
  `PlayerSettings` carries.
- **The Fox database row.** The compile writes the rows `appearance_row` produces for a
  stripped player into its `PlayerAppearance.bin`; boots and gloves go to the
  `BootsList.bin`/`GloveList.bin` pairs, not to the row.
- Generation's callers skip a record whose appearance id is -1 (`settings_toml.md`, the end
  of "Player settings in exports"); `from_player` itself reads whatever record it is given.
- Only the Save editor's strip operation writes the appearance id's -1 (worklog issue
  "player id versus appearance id").

**Decoding the rest of the ingame-face run (before 1.0.0).** The spec models the run as opaque
bytes with fifteen named fields because no legacy tool decodes more (`spec/pes_savefile/model.md`
"The ingame-face run"); nothing published does either (searched 2026-10-13). Every remaining
bit gets a name by measurement in the game itself, so that by 1.0.0 `[appearance.unknown]`
carries nothing and every face and hair setting is an authorable `settings.toml` key.

- **Method: one setting per player, many players per save.** In Edit mode (Edit Player >
  Appearance), each player of one team gets exactly one setting changed, then one Save; the
  diff of the run per player attributes each changed bit run to its setting. A slider is
  measured at both ends (two players, minimum and maximum) for its width and its stored
  coding; a list setting at its first, second and last entries. Driven by the in-game harness
  (`scripts/ingame/`, pad and OBS), the live save backed up first and restored by hash after,
  every measured save kept under the step's scratch folder.
- **PES 21 first, then the others by spot check.** The fifteen named fields sit at the same bits
  on every version, so the full map is measured on PES 21, then a handful of its new fields per
  version (17, 18, 19, 20, and 15/16 with their 46/50-byte runs) confirm or refute the shared
  layout; a version that differs is measured whole.
- **What lands.** Each measured field becomes an `IngameFaceField` row (one place, the table
  in `schema/ingame_face.rs`) and a `settings.toml` key with the game's own label and range;
  the completeness test's `unknown` remainder shrinks with each, and a bit no setting moves
  is recorded as such (measured unused), not left unknown.
