# 4cc Studio — Library crates plan: elevation

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## `libs/elevation`

What exists of this is in `spec/libs/elevation.md` "`libs/elevation`".

- **Manifest execution level**: the `studio` binary ships `asInvoker`; a
  `requireAdministrator` cargo feature flips the manifest for always-elevated
  setups.

The relaunch itself is a manual check at the first tool phase that needs it.
