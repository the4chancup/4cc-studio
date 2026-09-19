# 4cc Studio — Aesthetics export plan: FPC toggle

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## FPC toggle (`fpc.on` / `fpc.off` marker files)

FPC (Full Player Customization — see the [Save editor plan](../save_editor.md)) makes the default
player model invisible via a specific mix of savefile settings (blank-model boots/gloves IDs plus
strip settings, varying slightly per PES version), so that an FBM (Full Body Model) replaces the
player entirely. Setting it through individual `settings.toml` keys would be error-prone and
version-dependent; instead it gets a folder-level toggle, consistent with the principle that the
folder view tells the whole story about how the player's models render:

- **`fpc.on`** — an empty marker file in a player folder. When present, the compile-time savefile
  write applies the version-appropriate **FPC enable preset** from `pes_savefile` (the same preset
  the save editor's FPC toggle uses — one implementation, so the two tools can never drift).
- **`fpc.off`** — applies the disable preset (visible defaults) instead, for un-FPC'ing a previously
  FPC'd player at compile time without a trip to the save editor.
- **Absent = no FPC preset applied**: existing FPC settings are not reset merely because the marker
  is missing. Independent authored settings and model-derived ID assignments still apply.
- Tolerances, in the same spirit as link files: a stray `.txt` suffix (`fpc.on.txt`) is accepted
  silently, and a bare `fpc` file is accepted as `fpc.on` (presence reads as "on").
- Both markers in the same folder raise `fpc_conflict` (error; folder discarded).
- Either preset **overrides** any conflicting `[appearance.strip]` keys in the folder's
  `settings.toml`; explicitly-set keys that get overridden raise `fpc_strip_conflict`.
- In a `players.txt` multi-mapped folder, the preset applies to all mapped players (like the rest of
  the folder's settings).

**Boots/gloves ID precedence**, independently per category, for both `fpc.on` and `fpc.off`:

| Standalone asset outcome | Savefile ID |
|---|---|
| Requested local/shared output committed | Compiler-assigned ID wins over the preset's hide/default ID |
| Requested output failed or was dropped | Existing savefile ID is preserved; failure is not treated as absence |
| No standalone output requested | Apply the marker's preset ID, or preserve the existing ID if there is no marker |

Other preset fields still apply normally. Pre-Fox local models embedded in face XML do not request
a standalone boots/gloves output, so they follow the third row.

**Team kit-FPC status and kit configs.** FPC also requires settings on **every one of the team's kit
configs, including the goalkeeper kit** (modern system, per the [wiki's PES17 FPC
guide](https://implyingrigged.info/wiki/Pro_Evolution_Soccer_2017/Full_Player_Customization): shirt
model 176, shorts model 16, collar 105, winter collar 105 — stored as per-version constants
alongside the presets, since the retro pre-2024 system differed). These kit values are a **team-wide
prerequisite that enables per-player FPC**, not a per-player switch: with them in place, each
player's own savefile settings decide whether that player's body is hidden, and non-FPC (head-only)
players render normally on the same team. The markers are therefore strictly **player-level** —
`fpc.off` says "this player needs its body", not "this team's kits must not be FPC" — and mixed
teams (some `fpc.on` folders, some `fpc.off` or unmarked) are ordinary, supported usage;
`fpc_conflict` only rejects both markers inside *one* folder.

An export's **team kit-FPC status** is two-state — `EffectiveTeamKitFpc::{On, Unknown}`:

- **On** when at least one player folder carries `fpc.on` — the configs must then carry the FPC
  values for that player's hiding to work; **Unknown** otherwise — the absence of `fpc.on` markers
  makes no claim about the team (its FPC players may live only in the savefile, set through the save
  editor), and `fpc.off` markers contribute nothing here because they are per-player statements.
- **Generated kit configs** (the `kit_config_generated` path, when a kit folder has no
  `config.toml`) are created with the FPC values when On, and with the plain defaults when Unknown.
- **Supplied kit configs are reconciled only upward** (`kit_config_fpc_adjusted`): On → the FPC
  values are written into every config, which also protects against the classic user error of
  copying a kit config from another team. Unknown → supplied configs are left untouched. The
  compiler **never automatically reverts** FPC values found in supplied configs — no export state
  proves the team stopped using FPC, so de-FPC'ing a team's kits is a deliberate manual config edit
  (a future explicit team-level mechanism could revisit this).
- **Kit slots absent from the export are patched in place, not demanded back.** A midcup (partial)
  export can add an FPC player to a team whose kits need no other changes; requiring the untouched
  kit exports to be resent just so the compiler can see them would be easy to forget. When the
  status is On, the compiler edits the team's existing kit entries itself. Fox: the team's entries
  in the working `UniformParameter` get the FPC values — the working bins come from the installed
  cup CPKs (the bundled bases only serve from-scratch compiles), so a midcup compile patches the
  current cup state, and the `uniparam` container format is fully parsed (see the [library crates
  plan](../libs/README.md)), so locating a team's entries is routine. Pre-Fox: the team's current kit-config
  bins are located in the same installed CPKs, patched, and re-emitted. Patched slots report
  `kit_config_fpc_adjusted` like supplied configs; a slot with no existing entry or config to patch
  reports `kit_config_fpc_unpatched` (warning) — that team genuinely needs a kit export.
- **A custom collar wins over the FPC collar value**: collar rewriting (see "Collars" in the [Team compiler plan](../team_compiler/README.md)) runs after
  FPC reconciliation, so an FPC team with a custom collar keeps its replacement collar ID while the
  other FPC kit values stand.

The kit-config field logic this needs (reading/writing the shirt/shorts/collar model fields, plus
the compile-time binary emission) lives in **`libs/kit_config`**, shared with the [Kit config
editor](../kit_config_editor.md) tool; the FPC values themselves, the player presets, and the
interference rules come from the leaf crate **`libs/fpc`** (see [libs](../libs/README.md)), which `kit_config`
and `pes_savefile` both consume — the FPC system has exactly one description, so the compiler, the
kit config editor, and the save editor can never drift.
