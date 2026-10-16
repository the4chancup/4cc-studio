# 4cc Studio — Aesthetics export plan: settings.toml

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Player settings in exports (settings.toml)

What exists of this is in `spec/aesthetics_export/settings_toml.md` "Player settings in exports (settings.toml)".

The spec's engine split has a second half:

- **Pre-Fox (PES 15–17): in the savefile**, through the aesthetics patch the compiler writes and
  the save editor applies, as before. The database route is untested there.

In the spec's compile-time flow, which ends on the per-engine split, the pre-Fox half:

- **Pre-Fox:** everything resolved goes into the aesthetics patch.

The save editor view can also **generate** these TOML files from an existing savefile, giving teams
a migration path from the current workflow. Generation (`PlayerSettings::from_player`, shared with
the Export upgrader) writes every key the save's version holds,
so a generated file reproduces the player as he is, whatever the defaults; it emits
`boots_id`/`gloves_id` only for a stored ID from 1 to 100: 0 (the default boots, normal hands)
stays unset, which is its default, and an ID above 100 is custom content, which the upgrader
migrates into a folder or link (the old per-team blocks start at 101 too) and the save editor
leaves out. It writes `name` per the Export upgrader's rule (`true` when the folder's name part
equals the save's name, the explicit string otherwise; the save editor, generating without a
folder, writes the string), `shirt_name` as the save's string, and no `height` or `weight`.
On a stripped Fox save a record whose appearance id is -1 is not the player's look (the tables
are, see "Stripped save" in the [Save editor plan](../save_editor.md)), so generation skips
that player with a note; the face-editor route restores his id before generating.

---
