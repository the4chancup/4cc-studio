# 4cc Studio — Aesthetics export plan: settings.toml

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Player settings in exports (settings.toml)

Savefile settings that currently live only in the savefile move into the export as TOML, written to
the savefile at compile time. **Everything in the file is optional** — including the file itself: a
player folder with only models is valid, and an absent authored key means "leave that savefile
setting untouched"; model-derived ID assignments and FPC directives are separate inputs.

**The file can express every aesthetic setting the savefile holds.** Its schema is `pes_savefile`'s
`PlayerSettings`: all of the player appearance record — physique, strip style, wrist-tape and
spectacle colours, skin and iris colour, the motion block, the ingame-face parameters — except the
two compiler-owned groups (boots/gloves IDs, edit flags). That completeness is a tested invariant
in the [Savefile plan](../pes_savefile/model.md) ("Player settings model"), and it matters here because this
file is the **only** route by which a team's aesthetics reach the official save: the compiler
resolves it into an aesthetics patch, the save editor applies the patch, and the editor's own
appearance fields are read-only by default (see "Read-only aesthetics" in the [Save editor
plan](../save_editor.md)). The **template** — what the Export upgrader and the save editor generate,
and what a new player folder starts from — therefore lists every key: set ones with their value,
unset ones as commented lines whose injected comment gives the range, so a team can see what is
authorable without any other document. The example below is abridged.

**Boots/gloves IDs are not authored settings.** Export `settings.toml` neither accepts nor generates
boots/gloves ID keys. Local models and shared link files determine the compiler's player-exclusive
or shared assignments, using the existing per-target rules; users never copy numeric IDs into this
file. Pre-Fox local models embedded in face XML still need no standalone ID, as described above.

```toml
# settings.toml — inside a player folder

# Optional: the savefile player name. Absent = the savefile name is left
# untouched (the save editor keeps owning it). `name = true` derives it
# from the folder: the name part of the folder name (`15 - Snuffy` →
# "Snuffy"), or the whole folder name for players.txt-mapped folders.
# A string sets it explicitly. The number is never given here —
# numbering is owned by the folder name / players.txt mapping.
name = "Snuffy"

# The file never references models: what a player wears is fully determined
# by the folder contents (models present, link files pointing at shared
# folders), so the folder view alone tells the whole story.

[appearance]
skin_color = 2
iris_color = 1

[appearance.physique]
height = 180
neck_length = 2
shoulder_width = -1
# unspecified fields keep their savefile values

[appearance.strip]
sleeves = "short"
socks = "long"
untucked = true

[appearance.motion]
# hunching = 1            # 1–3
# arm_movement = 1        # 1–5
kick_motion = 3
# … every other key of the record, commented, with its range
```

Name-writing rules, in full — **not writing is the default**; writing is opt-in:

1. `name` absent → no name is written, ever. The savefile name stays whatever the save editor set.
   This is also what makes multi-mapped `players.txt` folders work as **generic model folders**:
   several players can share one folder's models without being renamed.
2. `name = true` → the name is derived from the folder (the name part of a numbered folder name, or
   the whole folder name under `players.txt`) and written.
3. `name = "..."` → the string is written as-is. This is also the only way to write a name carrying
   the savefile's **name colour codes** (`\x11c` + 8 hex digits, see the pes_savefile plan's "Name
   colour codes") — control characters cannot appear in a folder name, so a derived name (`true`)
   is always plain. Cup organizers colour names freely (not only medal players'), so tools that
   generate `settings.toml` from a savefile (the Export upgrader, the save editor) must emit the
   explicit string whenever the savefile name carries codes, or the next compile would strip them.
4. With `name = true` or a string in a folder mapped to multiple players via `players.txt`, the same
   name is applied to all of them, with a warning (`settings_toml_name_shared`).

Compile-time flow: resolve boots/gloves IDs from the models and link files present (a broken link is
caught as `link_target_missing` — a check impossible in the current split-tool workflow) → parse
accepted TOML → compile → resolve the settings and the IDs of content that was actually written into
the **aesthetics patch** written beside the CPK → if a local savefile is configured, apply that
patch to it through `pes_savefile` (decrypt, apply, re-encrypt, `.bak`). The patch is what a DLC
builder hands to the savefile builder, who applies it in the save editor; the compiler has no
other savefile write path (see "Post-processing" in the [Team compiler plan](../team_compiler/pipeline.md)).

The save editor view can also **generate** these TOML files from an existing savefile, giving teams
a migration path from the current workflow.

---
