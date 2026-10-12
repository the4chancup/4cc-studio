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
