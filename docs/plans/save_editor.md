# 4cc Studio — Save editor plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

The save editor (`crates/tools/save_editor`) is the successor to **4ccEditor**
and absorbs the **Midcupping** scripts: it edits PES save files (player stats,
appearance, teams, tactics), enforces AATF rules, diffs saves (gameplay and
aesthetics), transplants aesthetics between saves, handles team import/export
(new Team TOML format, legacy `.4ccs`/`.4cct` import, Texport read/write) and
FPC invisibility. It can also **generate `settings.toml` files from an existing
savefile** (via `pes_savefile`'s `PlayerSettings` model), giving teams a migration path
from the manual-editing workflow to the [Team compiler](team_compiler/README.md)'s
compile-time aesthetics (on Fox the CPK's database tables, on pre-Fox savefile writing). The
savefile codec, the data model, and all
interchange formats live in the `pes_savefile` lib crate — see the
[Savefile plan](pes_savefile/README.md). Platform context is in the [core plan](core/README.md).

The porting stance: **full feature parity with 4ccEditor, improved wherever the
Win32 UI imposed friction** — but only with improvements that are actually
useful, never decoration. Concrete improvements are listed per section.

---

## Background: 4ccEditor

What exists of this is in `spec/save_editor.md` "Background: 4ccEditor".

| Component | Files | Size | Fate in the Rust port |
|-----------|-------|------|----------------------|
| Win32 UI | `main.cpp`, `window.cpp`, `resource.rc`, `menu_lists.cpp` | ~320KB | UI replaced by egui; retain the relevant domain tables in `menu_lists.cpp` as codec/conversion references |

---

## Feature inventory (the parity contract)

What exists of this is in `spec/save_editor.md` "Feature inventory (the parity contract)".

Everything 4ccEditor can do, with its fate in the port. This is the checklist the
implementation is measured against.

### File operations

| 4ccEditor | Fate |
|-----------|------|
| Load 15/16/17/18/19/20/21 EDIT file (7 menu items) | One **Open savefile** action; auto-detect the version by master keys / container shape. The loaded file's version governs its codec and editing widgets; warn on a global-selector mismatch, never reinterpret the open file. The Open menu lists the saves `pes_savefile`'s discovery finds under `Documents\KONAMI` for every version (labelled by version and, for 19+, account folder) as one-click entries, with the files opened earlier in the session, above the ordinary file picker, and the picker opens in the selected version's save folder when it exists — see "Savefile discovery" in the Savefile plan |
| Save EDIT file (encrypted; always asks for a path; the first save after a load keeps the file it replaces as `<file>.BACKUP`) | Kept; plus **Save as**, and the file each save replaces kept as `.bak` (one level), at every save rather than the first after a load only |
| Import Texport file (per-version submenu) | Kept, auto-detected version; plus **Texport export** (new — PES itself can import the result; see [Savefile plan](pes_savefile/README.md)) |

### Player operations

| 4ccEditor | Fate |
|-----------|------|
| Set all ability stats (dialog: every ability stat to N, 40 to 99, prefilled with the player's first; weak foot, form and injury untouched) | Kept, same fields and range |
| Bump all ability stats ±N (dialog, Ctrl+B: N from -10 to 10, remembered; each ability stat clamped to 40 to 99; weak foot, form and injury untouched) | Kept, same; N is read whole (4ccEditor reads two characters, so -10 bumps by -1) |
| Copy player stats (source PID → dest PID: height, weight, every ability, form, injury, weak foot, registered and playable positions, playing style, skills, COM styles, stronger foot and hand, star, playing attitude, the free-kick motion only, and the edited-player and gameplay edit flags) | Kept, same field set |
| Swap player stats (two PIDs; the same field set as Copy) | Kept |
| Move a player up or down the list (the spinner beside it: one step per click, within one team; the two players' records trade places between their ids, `base_copy_id` traded back to match each slot, shirt numbers and the captain left with the slots) | Kept as drag reorder, with the same effect ("Editing UI", the Player list bullet) |
| Toggle FPC settings (Ctrl+I) | Kept (see "Full Player Customization" below) |
| Quick actions: Make Gold / Make Silver / Make Bronze (from the Autumn 26 edition) / Make Regular / Set stats | Kept, **driven by the AATF rules file** instead of hardcoded rates — the buttons and the checker can never disagree |
| Hidden shortcuts: PgUp/PgDn (previous/next player from any field) | Kept, the list's selection moving |
| Hidden shortcuts: Ctrl+Shift+P (a joke preset), Ctrl+Shift+B (a sound), Ctrl+Shift+H (show the dribbling-motion and hand controls), Ctrl+O and Ctrl+5 to Ctrl+1 (open as a given version) | Not kept: Open detects the version and every control is always drawn |
| Hidden shortcut: Ctrl+Shift+T on the team selector (empties the team's slots past 23; its team index looks one off) | Dropped: cups no longer trim a roster to 23 this way |
| Is a copy? (Appearance; ticking it also sets the ability, COM, motion, playable-position, registered-position, skill and playing-style edit flags) | Kept, the same flags set |

### Team operations

What exists of this is in `spec/save_editor.md` "Team operations".

| 4ccEditor | Fate |
|-----------|------|
| Clear visual settings flags (all players on team) | Kept |
| Set edited/created flag (all players on team) | Kept |
| Set / remove FPC invisibility (whole team) | Kept |
| Set boot/glove IDs for everyone (boots: unchanged, incremental or same for all; gloves: unchanged, like the boots, or same for all; incremental gives slot `k` the start ID + `k` − 1, slots 1 to 23) | Kept, every filled slot of the roster numbered by its slot (4ccEditor stops at the first id missing from the team, a bug not kept) |
| Set player names to positions (each name becomes its registered position's full name, "Goalkeeper", and the edited-player flag is set) | Kept, same |
| Load squad (`.4ccs`, with stats/aesthetics/tactics checkboxes) | Kept as read-only legacy import, same section checkboxes; Team TOML import is the primary path. Cross-version import applies the playstyle/skill conversion maps |

### Tactics operations

| 4ccEditor | Fate |
|-----------|------|
| Export / import tactics (`.4cct` "nightly" files) | Import kept (read-only legacy); export replaced by Team TOML (tactics are a section of it) |
| Copy / swap presets (1↔2↔3) | Kept |
| Copy / swap formations (kick-off / in-possession / out-of-possession) | Kept |
| Full tactics editing (see "Tactics editor" below) | Kept at full parity + interaction improvements |

### Database operations

What exists of this is in `spec/save_editor.md` "Database operations".

| 4ccEditor | Fate |
|-----------|------|
| Output rosters to TSV (`EDIT.tsv` in the working folder, no dialog, UTF-8; every team, hidden ones included, in roster order; per player: number, name, registered position, Finishing as the "rating", captain mark, height, playing style, skills, COM styles) | Kept, the same columns; written where a save dialog says, `EDIT.tsv` its default name |
| Fix database (clear all visual flags, reset kit slots, PES17 kit-ID repair; a player's base-copy id cleared when it is neither his own nor 0 and his base-copy flag is off, the flag cleared when the id is his own or 0; every team's strip-edited flag cleared) | Kept as a maintenance action behind a confirmation dialog, all of it |

---

## Crate layout

What exists of this is in `spec/save_editor.md` "Crate layout".

Planned files that do not exist yet:

- `src/session/undo.rs` — field-level undo/redo stack over model edits
- `src/session/selection.rs` — team/player selection, cross-team filter query
- `src/session/recent.rs` — discovered saves (`pes_savefile::discovery`) + recent files list
- `src/view/` — the view, in the pre-move tree's shape: `mod.rs` (master-detail layout,
  open/save actions, unsaved-changes prompt), `player_list.rs` (drag-to-reorder, dirty dots,
  colour-coded names), `info_strip.rs` (general info + quick actions: Gold/Silver/Regular, set
  stats), `tabs/` (`abilities.rs`, `appearance.rs`, `team.rs`, `tactics.rs`), `panels/`
  (`aatf.rs`, `comparator.rs`, `transplant.rs`, `interchange.rs` — Team TOML / `.4ccs` / `.4cct`
  / Texport import-export dialogs) and `widgets.rs` (editor-local reusable widgets: version-gated
  control, ID picker; a color is drawn by how the save stores it: an RGB one (the team colors,
  the only RGB fields a tab draws) is `color_tools`' picker, `libs/color_tools.md`, not a
  widget of this crate, the host converting between the picker's 8 bits and the save's 6 per
  channel: a picked channel `c` is stored as `c * 63 / 255` rounded, a stored `v` shown as
  `v * 255 / 63` rounded (the Team compiler's widening of 5:6:5 colors), so a save always
  holds the nearest color it can; 4ccEditor stores `ceil(c / 4.05)` and shows `floor(4.05 v)`,
  so a value can read one higher here (`v` 32: 130, where 4ccEditor shows 129), a difference
  kept on purpose, since the stored value is what reaches the game; a palette
  index (skin, iris, wrist tape, spectacles, gloves, hair accessory, as `settings.toml`'s
  `skin_color` 0-7) is a choice of its palette's entries, never a picker whose free color no
  record can hold)

Placement rules for the planned parts:

- **`view/tabs/` map 1:1 to 4ccEditor's tabs**, which is the parity contract; a new tab is a new
  file, not a new section of an existing one.
- **`view/panels/` are the Midcupping-derived and new features**; each is self-contained and can be
  developed in the build order below without touching the tabs.
- **The tactics editor, the card pickers and the violations list are `libs/team_widgets`** (see
  below); the tab and panel files here are hosts: they hand the widget the session's team, apply
  the edits it reports to the undo stack, and add what only this tool has (preset copy/swap,
  jump-to-player).

---

## Editing UI

The tool view (the panel right of the suite sidebar) is a master-detail layout,
mirroring 4ccEditor's structure without its dialog sprawl:

```
┌────────────────────┬──────────────────────────────────────────────┐
│ [team selector ▾]  │  Name [........] ID 70103   Shirt [......]   │
│ [player filter 🔍] │  Height/Weight/Age/Nation/Number/Captain     │
│                    │  [Gold] [Silver] [Bronze] [Regular] [Set stats: __]   │
│  01 GK Snuffy      ├──────────────────────────────────────────────┤
│  02 CB Anon    ●   │  Abilities & Skills │ Appearance │ Team │    │
│  03 CB Mod         │                       Tactics               │
│  ⋮  (drag to       │                                              │
│      reorder)      │  (active tab content)                        │
└────────────────────┴──────────────────────────────────────────────┘
```

- **Team selector** with an "ALL" entry (as in 4ccEditor), plus a **filter box
  that searches across all teams** (name, shirt name, ID) — finding a player no
  longer requires knowing their team first. Matching uses the colour-code-free
  `display_name()` (see the pes_savefile plan's "Name colour codes").
- **Player list**: shows number, position, name; **drag-and-drop reordering**
  replaces the up/down spinner, and moves what the spinner moves: the players' records trade
  places between the ids, each id staying in its roster slot (`base_copy_id` traded so each
  matches its new slot, and the in-game face run's face structure and hairstyle staying too:
  the spinner's record, the `.4ccs` one, holds of that run only the skin, iris and player
  gloves), and the shirt numbers, held by the roster's slots, stay where they
  are, as does the captain mark. The roster's own order is never changed: by 4cc rule slot `NN`
  holds player `team id × 100 + NN` (`plans/aatf_rules.md` "The host"), and reordering the
  roster itself would break it. A drop several rows away is the spinner pressed that many
  times, so the players in between shift one slot along; drag works in one team's list and is
  off in ALL and while a filter is typed, where neighbouring rows are not one team's
  neighbouring slots. A dot marks players with unsaved changes. Names
  render with their savefile colour codes applied as text colours (the in-game
  look; 4ccEditor shows the raw control characters), and the name edit field
  keeps the raw string so codes survive a roundtrip untouched. A code's first six bytes are
  the RGB hex; its alpha is not drawn, and a code whose six are not hex leaves the text the
  normal color (the game does not check them: `spec/pes_savefile/codec.md` "Name colour
  codes, as measured").
- **General info strip** and **quick actions** always visible above the tabs. The save is where
  a member writes the names a tactical export carries to a cup of any version, so its Name and
  Shirt fields follow `pes_savefile`'s portable limits ("Team TOML" in its operations plan):
  - **Name**: the open save's own field (61 bytes on PES 20/21, 46 before). A name past 46
    bytes, which only a PES 20/21 save holds, draws its field with an amber tint and a hover
    "In PES 2019 and older this name will be cut to: «…»", the name as `crop_player_text`
    would cut it. A tint of the theme's warning colour, not a pale yellow fill: the theme is
    dark and its text light, which a pale fill would wash out.
  - **Shirt**: 18 letters on every version but PES 16 (its field holds 16). A shirt name the
    save already holds past 18 (PES 20/21 allow more) is kept, drawn with the same tint and a
    hover "PES 2019 and older show 18 letters of a shirt name (PES 2016: 16)"; an edit may shorten it but not lengthen
    it, so it can be fixed a keystroke at a time.
  - Nothing more: a warning, never a block on Save, so editing a save stays frictionless.
  - What 4ccEditor's fields do besides storing, kept: a name or shirt name edit sets the
    player's edited-player flag; a shirt name is upper-cased as typed; Captain is a radio
    that makes this player's slot the team's captain and is not unticked by a second click
    (the captain belongs to the slot).
- **Tabs** (contents 1:1 with 4ccEditor, but for star, playing attitude and stronger hand,
  which 4ccEditor draws on its Appearance tab and the approved draft on Abilities & Skills,
  beside the gameplay values they are):
  - **Abilities & Skills** — 13 playable positions with A/B/C ratings, 7 COM
    styles, up to 41 skill checkboxes (version-gated), ~25 ability stats with
    spinners, playstyle / registered position / form / injury / weak foot,
    version-gated extras (star 19+, playing attitude & stronger hand 20+).
    Its numbers are the game's, in 4ccEditor's ranges: abilities 40 to 99, form 1 to 8,
    injury resistance 1 to 3, weak foot usage and accuracy 1 to 4 (these four stored one
    lower), star 0 to 7, playing attitude 0 to 3, each inside the field's bit width on every
    version. A value the save holds outside its range shows as stored and changes only when
    the member edits it: drawing a player never edits him (an egui number control clamps the
    value it draws unless told not to, which would mark him unsaved on sight). The skills'
    **Cards left** badges are the active ruleset's, the one `check-aatf` runs (the
    `aatf_rules` setting's file, else the official one), one badge per tier; a file that does
    not load shows its error in the badges' place.
  - **Appearance** — edit flags (face/hair/physique/strip, base copy + copy-from
    ID), physique sliders (14 values), colors (wrist tape L/R, spectacles), strip
    style (boots ID, gloves ID — any player's, 0 = normal hands — taping, spectacles,
    sleeves, inners, socks, undershorts, shirttail, the player-gloves checkbox), motion (hunching, arm movement, kick
    motions, gc1/gc2, randomize; dribbling motion 20+), skin/iris color.
    **Read-only by default** — see "Read-only aesthetics" below.
  - **Team** — team name/short name, colors 1–2 (RGB), manager ID, stadium ID,
    kit slot assignments.
  - **Tactics** — see below.
- Version-gated widgets render disabled with a tooltip ("PES 19+") rather than
  disappearing, so the UI stays spatially stable across versions.
- **Undo/redo** across all edits — the single biggest safety upgrade over
  4ccEditor. An undo entry holds the before and after of each model entry it changed (a
  `PlayerEntry`, a `TeamEntry`): the model is plain data, an entry is a few hundred bytes, and
  one mechanism serves every widget, since a widget edits a copy and reports only that it
  changed (`libs/team_widgets`' contract), not which field. One entry per gesture: changes
  from one widget merge into the open entry while a pointer button is held or a widget has
  keyboard focus, so a drag or a typed value is one undo step, not one per frame.
- **Unsaved changes** are the difference from the file as opened or last saved, not the undo
  stack's length: undoing back to the saved state clears the dot. A player's dot covers his
  entry and his roster slot's shirt number.
- Keyboard shortcuts preserved where they earn their keep (Ctrl+S save, Ctrl+F
  clear visual flags, Ctrl+I FPC toggle).
- Closing, or opening another save, with unsaved changes prompts: save, discard or cancel,
  over a table of what would be saved or lost, columns Item, Old value and New value (the
  table spans the prompt's width, the two value columns sharing its right part), one row
  per changed field against the file as opened or last saved (the dots' baseline, so an edit
  undone back is not listed), the players' rows first, then the teams' own fields. A row is
  worded as the Comparator panel's are, in the tabs' terms: the item is the player's display
  name (or the team's name) and the field's tab label, the values as that tab shows them.
  The table is computed when the prompt opens, not per frame; players through
  `pes_savefile`'s `compare_players`, teams through a team twin of it over the team, roster
  and tactics schemas.
- Save is refused while the save's own PES runs, as the CLI's writing commands are (the game
  writes its save back when it closes, over the one just written); another version's game does
  not block it.

Improvements policy: each of the above exists to remove a real friction point
(finding players, reordering, fear of misclicks). No animations-for-the-sake-of-it.

---

## Tactics editor

Full parity with 4ccEditor's Tactics tab: preset selector (1/2/3) with fluid
checkbox and the five sliders (support range, defensive line, compactness,
numbers in attack/defense), formation settings toggles (attacking style, zone,
buildup, positioning, defensive style, containment, pressure), advanced
instructions (2 attack + 2 defense, with target player where applicable),
set-piece takers (FK long/short/2, CK L/R, PK), captain, auto flags (substitution,
offside trap, atk/def levels, preset change), the three formations per preset,
starting eleven and bench order.

The interaction layer is rebuilt around direct manipulation (this is where the
Win32 original hurt the most):

- **The pitch is the editor**: an egui painter canvas draws the formation;
  players are **dragged** to reposition (x/y), with position labels and snapping.
  4ccEditor's per-slot dropdown + coordinate fields remain as a detail popover on
  click, not the primary interface.
- **Lineup by drag-and-drop**: the starting XI and bench are reorderable lists;
  **dragging a bench player onto a pitch player swaps them** (and vice versa).
  The arrow-button ordering UI is gone.
- **Click-to-assign roles**: "players to join attack" (and other
  pick-N-players settings) are set by clicking players on the pitch in an
  assignment mode — the three dropdowns are gone. Set-piece takers get the same
  treatment with a role palette.
- Copy/swap preset and copy/swap formation become two small toolbar menus on the
  tactics tab (parity with the 4ccEditor menu tree, minus the 18-item menu).
- Formation edits participate in undo/redo like everything else.

Version quirks (advanced-instruction ranges differ 17 vs 18+) are enforced by the
`pes_savefile` model, not the UI; a `.4cct` imports into any version `TeamToml::apply`'s
version rule admits (`spec/pes_savefile/operations.md` "Team TOML"), not only into 16-18 as
4ccEditor's did.

## `libs/team_widgets`

The tactics editor above, the skill/COM/playstyle pickers of the Abilities tab, and the AATF
violations list are not this tool's alone: the [Team creator](team_creator.md) shows the same
formation, cards and instructions to a new manager over a team that does not exist in any save
yet. Tool crates never depend on tool crates (core plan, guardrail 1), so the widgets live in a
lib — `libs/team_widgets`, an egui widget crate in the class the core plan already allows for
`color_tools` and `model_viewport`: egui-dependent, `wasm32`-clean, and nothing else.

Contract, the same one the color picker has: a widget takes `&mut` model (`TeamEntry`,
`TacticsPreset`, `PlayerEntry` — `pes_savefile`'s types), the version's schema for gating, and the
AATF rules where it shows limits; it draws, and it returns what changed. It never sees a session,
an undo stack, a file, or a savefile byte. The host decides what an edit means: this editor pushes
it onto the undo stack, the creator writes it into its draft.

Contents:

- `pitch` — the drag-and-drop formation canvas, position labels and snapping, the per-slot detail
  popover, lineup and bench lists with swap-by-drag.
- `tactics` — preset selector, sliders, style toggles, advanced instructions (with target-player
  pick), set-piece role palette, auto flags, click-to-assign modes.
- `cards` — skill, COM-style and playstyle pickers, version-gated, with remaining-count badges per
  tier from the AATF rules file (`card_limits`). The playing styles are listed in the order the
  version stores them, which is the game's own Edit-mode list, not the canonical order the
  Team TOML names them in: a member compares the combo against the game's list.
- `violations` — the AATF results list grouped per player, emitting a "selected player" event for
  the host's jump-to. The groups keep `check_team`'s order (the players by slot, then the team's
  own rows), each group's errors before its warnings; a team header heads each team's rows;
  suggestions are drawn only when the host asks, in their own group after the violations. A row
  is the rule's name as a rules file writes it and a sentence saying what is wrong; the
  sentences are the widget's, one per `ViolationKind`, so both hosts word a violation alike
  (the CLI prints the name and the fields only, `aatf`'s `ViolationKind::rule`, which both use).

Stock-formation constants stay in the Team creator (its only consumer) until this editor wants
an "apply stock formation" action, at which point they move here.

---

## Full Player Customization (FPC)

FPC is the 4cc system for fielding **FBMs (Full Body Models)**, custom models that replace
the player's entire body; the system and its presets are specified with `libs/fpc`
([spec](../spec/libs/fpc.md)).

The editor's side of this, ported from `fpc.cpp`: a per-player toggle and a
team-wide on/off that apply `libs/fpc`'s FPC enable/disable
presets (their fields are listed in the [libs/fpc spec](../spec/libs/fpc.md)); the strip settings
that break or bend hiding (inners, undershorts, taping, the gloves checkbox) are reported by
its interference rules on the Appearance tab, not cleared. The ID constants become a suite-common setting consumed by
the presets rather than being hardcoded (any nonexistent ID works;
55/11 are the 4cc convention). The same presets serve the
[Team compiler's](team_compiler/README.md) per-player-folder `fpc_on`/`fpc_off` marker
files, so the editor's toggle and the compiler's markers cannot drift apart.

---

## Read-only aesthetics

A player's aesthetics are owned by the team's export: `settings.toml` and the models decide them.
On Fox the Team compiler writes them into the database tables of its CPK and the savefile's
appearance records are stripped (see "Stripped save" below); on pre-Fox it resolves them into an
**aesthetics patch** (format in "Aesthetics patch" of the
[Savefile plan](pes_savefile/operations.md)), and the save editor
applies the patch. Hand-editing the same fields here
would create the two-sources problem — the next patch silently overwrites the hand edit, or the
hand edit silently diverges from the DLC — so the editor **does not edit aesthetics by default**:

- The Appearance tab renders its values disabled, with one line of explanation ("Owned by the
  team's export — edit its `settings.toml` and recompile", plus on pre-Fox "then apply the
  patch" and a link to the apply action).
- The same lock covers every other aesthetics write path: the team operations that write
  appearance fields (set boots/gloves IDs for everyone, set/remove FPC for the team, clear visual
  flags, the FPC toggle), the aesthetics transplant, and the aesthetics section of Team TOML and
  legacy squad imports (the section checkbox is disabled, gameplay and tactics import as before).
- **Apply aesthetics patch** is the one aesthetics write open by default: open a patch; the
  version must match the save and the allocation scheme the suite's (refused otherwise, save
  untouched); the comparator's preview lists every player the patch will change with old → new;
  applied through the undo pipeline; saved only when the user saves. A team the save lacks is
  reported and skipped; the rest applies. Patches apply cleanly in sequence — a midcup patch
  carries only the teams it recompiled and only the fields their compile resolved.
- **Unlock: "Edit aesthetics anyway"**, a switch on the Appearance tab, off at every start of the
  tool. Turning it on re-enables all the paths above for the session, with the note that on
  pre-Fox a later patch overwrites whatever is edited by hand, and on a stripped Fox save the game
  does not read it. It exists for the cases the rule does not cover — a
  team with no export, a cup-night fix when recompiling is not an option — and it is a session
  switch rather than a setting so that the default cannot quietly become "unlocked" on the
  savefile builder's machine.

Gameplay data (stats, skills, positions), team data and tactics are unaffected: they were never
the compiler's, and the savefile builder edits them as before.

---

## Stripped save

What exists of this is in `spec/save_editor.md` "Stripped save".

- The GUI parts of its rules: `.4ccs`/`.4cct` and Texport imports leave the appearance block's
  id as it is, as every import does; the unlocked Appearance tab writes the record's fields but
  not its id, and on a stripped save it says that the game does not read it; every reader of a
  record's appearance in the GUI (the Appearance tab, the FPC classifier and its interference
  findings) marks a player at -1 as stripped beside what it shows; `settings.toml` generation
  skips him ("Player settings in exports" in the
  [Aesthetics export plan](aesthetics_export/settings_toml.md)); opening a Fox save lists the
  players whose appearance id is set while others are stripped, with a one-click **Strip
  again** (their record is ignored from then on; the tables decide).

---

## Comparator (gameplay + aesthetics)

What exists of this is in `spec/save_editor.md` "Comparator (gameplay + aesthetics)".

The panel over the spec's `diff` report:

- Load a second save (same version); the panel lists per-team, per-player
  differences.
- Filter toggles: All / Gameplay / Aesthetics; a team filter; export the diff as
  text.
- Clicking a diff row jumps to that player in the editor.
- A row is drawn in the tabs' terms: the field under its tab's label, its value as that tab
  shows it (a free-kick motion 15, where the save stores 14), a field no tab shows keeping the
  report's name and value. Export writes the CLI's report unchanged, stored values included.
  Not the report's values on screen, because a member reads a row against the tab it jumps to,
  and a motion one lower than the tab's reads as a wrong row; not tab values in the report,
  because the report is what the compare scripts it replaces print.

---

## Aesthetics transplant

Midcupping's transplant as a guided panel:

- Pick a donor save (validated to be the same PES version).
- Build the selection the same four ways the scripts support: player IDs,
  `target:donor` pairs, whole teams, team ranges — plus by clicking players in
  a two-pane team browser.
- **Preview before applying**: the affected players are listed with their
  aesthetics diff (reusing the comparator's fingerprint), so a wrong team ID is
  visible before it does damage.
- Applies through the normal edit pipeline: undoable, and saved only when the
  user saves.

---

## Interchange formats in the UI

The formats themselves are specified in the [Savefile plan](pes_savefile/README.md);
the editor exposes them as:

- **Team TOML export/import** per team (the `.4ccs`/`.4cct` successor: complete
  team + tactics + players including full aesthetics and ingame-face data).
  Import goes onto the team selected in the editor and offers the same section choices as
  4ccEditor's squad load (gameplay / aesthetics / tactics), and applies cross-version conversion with a warning list
  when the file's version differs. The sections split the file as the squad load does: names,
  shirt names and shirt numbers go with aesthetics (its `PlayerImport` copies them under
  `importAes`), not with gameplay as the comparator's scopes have them, and so do nationality,
  age, every motion but the free kick's, the dribbling motion and the base-copy flag and id;
  the free-kick motion, height, weight, star, playing attitude and stronger hand go with
  gameplay (`editor.h` `PlayerImport`, read 2026-10-21), so a file that loads
  with 4ccEditor's choices loads the same here. The file's `[team]` keys, which the squad file
  never carried, go with gameplay.
- **Legacy import**: `.4ccs` and `.4cct` open through the same import dialog,
  read-only.
- **Texport import/export** (export is new; PES itself can import the file). Export writes
  PES 18-21 files from `Texport::new`'s templates; on a PES 15-17 save it is disabled, its
  tooltip saying those versions have no template (`spec/pes_savefile/operations.md`
  "Texport (read + write)").
- **Aesthetics patch apply** — see "Read-only aesthetics"; the patch is written by the Team
  compiler, never by the editor.
- **`settings.toml` generation** — per player folder, for migrating a team to the
  [Team compiler](team_compiler/README.md)'s compile-time aesthetics (on Fox the CPK's database
  tables, on pre-Fox savefile writing). Uses the
  shared authorable `PlayerSettings` subset; boots/gloves IDs are emitted only when they name a
  stock model (1 to 100), since custom IDs are compiler-assigned from models/links, not export
  settings.
  Full Team TOML remains a full-fidelity save interchange. Name handling: the editor
  generates without a folder, so it writes the save's strings (`name = true` is the
  Export upgrader's rule) — in particular a
  name carrying colour codes is always emitted as the explicit string, so the
  next compile preserves the codes.

---

## Configurable AATF rules

What exists of this is in `spec/save_editor.md` "Configurable AATF rules".

What this editor adds on top of the lib:

- **Quick actions.** One button per tier of the active ruleset, in its order (Make Gold / Silver /
  Bronze / Goalkeeper / Regular under the official one), each `aatf::apply_tier`.
- **The AATF panel.** Results grouped per player; clicking a violation jumps to the player; team
  selection matches 4ccEditor (current team or a multi-select list). A **Show suggestions**
  toggle, off by default, lists unused allowances apart from errors and warnings (see
  "Suggestions" in the AATF rules plan). The panel is `team_widgets`' violations list.
- **Live AATF.** A toolbar toggle, off by default and remembered in the tool's settings
  (`live_aatf`), for a member who has built the team and is now making it legal. On, a panel
  docked at the bottom of the view lists the open team's violations, errors first, grouped per
  player as the AATF panel does, under the header `N issues · <ruleset>`, or `No issues:
  <team> passes <ruleset>` once the list is empty; clicking a row selects that player. The
  team is re-checked after every edit to it, undo and redo included, and when another team is
  opened, so the list shrinks as the member fixes it. One team's check takes about 50 ms in a
  release build (2026-10-21, the official ruleset on a PES 2017 save: 220 teams in 10.7 s),
  more than a frame, so it runs on a worker once the edit settles (a drag released, a field
  committed) and the list keeps the previous result, marked as checking, until the new one
  arrives. The panel is the log panel widget's frame (its chevron collapses it to the header,
  its divider resizes it). At the bottom, not at a side: the player list holds the left and the
  tabs need their width (the Abilities grid), and a bottom strip is where the Team compiler's
  log already sits. The floating AATF panel stays for several teams at once, suggestions,
  Validate rules and Save a copy; its whole-save check (10.7 s above) runs on a worker too.
- **Validate rules.** A button running the lib's load-time validation on the chosen file, its
  findings shown with line or key path.
- A **"save a copy"** action writes the embedded official ruleset out.

---

## Build order and verification

`pes_savefile` (codec, model, conversions, transplant/fingerprint ops, interchange
formats, comparator, FPC invisibility) is built in Phase 2 as a standalone lib; the
editor's non-UI substance a command drives — the tool crate's settings, CLI and the `session/`
operations its commands run, plus the `aatf` rules engine — lands in Phase 5; the view lands
in Phase 8 with the `session/` wiring only it drives (undo, selection, recent files, the FPC
toggle, AATF quick actions), each built in the slice that uses it (see the
[core plan](core/development_plan.md#development-plan)). Within Phase 8, the draft comes first:
`docs/prototypes/save_editor.html` (`core/gui.md` "Prototypes before implementation") shows the
master-detail layout, every tab and every panel of the five slices below with sample data, and
is refined with the maintainer before slice 1 is coded; a slice that changes what the draft
shows changes the draft first. Then the view builds up as:

1. Open/save + player editing tabs (Abilities & Skills, Appearance) — the card pickers as the
   first `team_widgets` content
2. Team tab + batch operations + FPC
3. Tactics editor in `team_widgets` (pitch canvas last — everything else works without it)
4. Comparator, transplant, AATF panels
5. Interchange format dialogs (Team TOML, legacy import, Texport, settings.toml)

Verification:

- Field-level parity: the same save opened in 4ccEditor and in the new editor
  must display identical values for every field, all versions (spot-checked per
  tab; automated for the codec layer in `pes_savefile`'s roundtrip tests).
- Batch operations and Fix database compared against 4ccEditor's output on the
  same input save (byte-diff of the decrypted payload).
- AATF: the official ruleset's parity with `aatf.cpp` and the rest of the AATF rules plan's
  "Development phase and verification".
- Comparator/transplant output parity against the Midcupping scripts.
- Texport export verified by importing into the game (manual, per version).

---

## Acceptance

What exists of this is in `spec/save_editor.md` "Acceptance", whose rules (format, the fixture
saves, a team or player named by its id in them) these scenarios follow; IDs continue the
spec's numbering.

**Phase 8 scope.** Phase 8 delivers the view and the `session/` wiring only it drives (undo,
selection, recent files, the FPC toggle, AATF quick actions), in the five slices of "Build
order and verification". What a scenario observes is what a frame draws, through the headless
harness (the frame's AccessKit nodes, a click or a drag as pointer frames, keyboard shortcuts
as key events; a color, which no AccessKit node carries, is read back from the view state the
frame paints from), the file written by Save, and the session's model; `manual` marks only
what needs a display, a native dialog or a running game. "The save" is the fixture save of
the version named, PES 21 unless the scenario says otherwise, copied under the test's own
folder; save discovery is pointed at a Documents folder under that folder, never the
developer's. An operation's outcome in the file is its own specification's where one exists
(a transplant's, `SE-TRN`; a patch's, `SE-PAT`; a tier's values, `spec/aatf_rules.md`
`apply_tier`; a Team TOML's, the CLI's export); these scenarios observe that the view reaches
the same operation and shows its result. The operations with no command, which the spec's
Phase 5 scope left to the view (the quick actions, the player and team operations, the
tactics copies, the Texport and legacy imports, generation), get theirs here.

```
SE-GUI-01  GIVEN the save
           WHEN it is opened from the view (Open, or its entry in the discovered-saves list)
           THEN the team selector lists its teams and ALL, the player list shows each player's
                number, position and name with the name's color codes rendered as colors, and
                the info strip shows the selected player's general fields
SE-GUI-02  GIVEN the save open on team 701
           WHEN a name, shirt name or id of a player of another team is typed in the filter box
           THEN the list shows the matching players across all teams, matched on the
                color-code-free display name, and selecting one shows him
SE-GUI-03  GIVEN the save open
           WHEN a player's ability value is edited in the Abilities & Skills tab
           THEN his list entry carries the unsaved-changes dot, the file is unchanged until Save
                (Ctrl+S or the button), and after Save the reopened file holds the value with
                the dot gone
SE-GUI-04  GIVEN two edits made in order
           WHEN Undo is invoked, then Redo
           THEN the second edit's field returns to its earlier value and the first's stays,
                then the second is reapplied; the file changes only on Save
SE-GUI-05  GIVEN the save open with unsaved changes
           WHEN the window is closed, or another save opened
           THEN a prompt offers to save, discard or cancel over a table of the changes, one
                row per changed field (GIVEN a player's Height and Speed edited and a third
                field edited then undone THEN two rows, each naming him and the field with its
                old and new value as the tab shows them); cancel leaves the session as it
                was; save writes the edits to the open file, then closes or opens the other
                save; discard closes or opens it and the open file is unchanged; GIVEN the
                Team compiler's view active when the window is closed THEN the close waits,
                the Save editor becomes the active view and shows the same prompt
SE-GUI-06  GIVEN a PES 17 save
           WHEN the Abilities & Skills tab is drawn
           THEN the star, playing attitude and stronger hand controls are disabled with a tooltip
                naming the first version that has them, in the same place as on a PES 21 save,
                where they are enabled
SE-GUI-07  GIVEN the save open
           WHEN the Appearance tab is drawn
           THEN its values are disabled with the line saying the team's export owns them (and
                on pre-Fox, the patch step); WHEN "Edit aesthetics anyway" is switched on THEN
                they are editable, the switch is off again at the next start of the tool, and on
                a stripped player the tab says the game does not read the record
SE-GUI-08  GIVEN the save open on a team
           WHEN player 02 is dragged above player 01 in the list
           THEN the list shows the new order, and after Save slot 01 still holds player
                `team id × 100 + 01`, whose record (name, stats, appearance but the face and
                hair) is the one player 02 had, and the other way round; the roster's ids and shirt numbers are as they
                were, and the AATF check reports no `RosterOrder` error for the team
SE-GUI-09  GIVEN the official ruleset active
           WHEN Make Gold is clicked on a player
           THEN his fields hold the Gold tier's values as `aatf::apply_tier` sets them
                (`spec/aatf_rules.md`), as one undoable edit, and check-aatf then reports no
                violation of the Gold tier's targets for him; GIVEN the VTL11 ruleset active
                (bundled, `aatf/rulesets/vtl11.rhai`) THEN the quick actions are Make Gold,
                Silver, Regular and Buffed in that order, no Bronze or Goalkeeper, and Make
                Silver sets VTL11's Silver values (rate 89, not the official 92)
SE-GUI-10  GIVEN the save open on a team
           WHEN its name, a color or the manager id is edited in the Team tab and saved
           THEN the reopened file holds the values; GIVEN #FF8000 picked as color 1 THEN the
                reopened team's color 1 is (63, 32, 0) and the Team tab shows it as #FF8200
SE-GUI-11  GIVEN a player without FPC and "Edit aesthetics anyway" switched on
           WHEN Ctrl+I, or the FPC toggle, is used
           THEN his record carries the FPC enable preset's values and the toggle reads on; the
                team-wide switch applies it to every player of the team; a strip setting that
                breaks hiding is listed on the Appearance tab as an interference finding
SE-GUI-12  GIVEN the Tactics tab on preset 1
           WHEN a pitch player is dragged to another spot
           THEN the slot's position is the drop point after snapping, undoable, and the saved
                file holds it; WHEN a bench player is dragged onto a pitch player THEN the two
                swap; WHEN a set-piece role is assigned from the palette THEN the taker field
                holds that player; WHEN "players to join attack" is in its assignment mode and
                two pitch players are clicked THEN those two are the setting's players
SE-GUI-13  GIVEN the save open and a second save of the same version loaded in the comparator
           WHEN the panel is drawn
           THEN it lists the per-team, per-player differences the diff command reports, the
                All / Gameplay / Aesthetics toggles and the team filter narrow them, Export
                writes the same text as the CLI's report, and clicking a row selects that player
SE-GUI-14  GIVEN "Edit aesthetics anyway" switched on and the transplant panel with a donor
                 save of another version
           WHEN it is picked
           THEN it is refused naming both versions; GIVEN one of the same version and a
                selection built in the panel THEN the preview lists the affected players with
                their aesthetics diff before Apply, and Apply is one undoable edit written only
                on Save
SE-GUI-15  GIVEN a ruleset with violations in the open team
           WHEN the AATF panel is drawn
           THEN the violations are grouped per player, clicking one selects that player, Show
                suggestions is off and reveals the unused allowances when on, Validate rules
                lists a broken rules file's findings with their line or key path, and Save a
                copy writes the embedded official ruleset out byte for byte
SE-GUI-16  GIVEN the save open on a team
           WHEN Team TOML export, then import on another team with only the gameplay section
                chosen, runs from the interchange dialog
           THEN the file written equals the CLI's export, and the import changes only the
                gameplay fields, with a warning list when the file's version differs
SE-GUI-17  GIVEN a `.4ccs` or `.4cct` file, or a Texport
           WHEN it is opened through the import dialog
           THEN its sections import as the Team TOML import does, read-only for the legacy
                formats; WHEN Texport export runs on a team of a PES 18-21 save THEN the file
                written is a Texport of the save's version that the import dialog reads back to
                the same team values; GIVEN a PES 15-17 save THEN the export is disabled, its
                tooltip naming the missing template
SE-GUI-18  GIVEN an aesthetics patch of the save's version
           WHEN Apply aesthetics patch is used
           THEN the preview lists every player it changes with old and new values, Apply is
                one undoable edit, and a patch of another version is refused with the save
                untouched
SE-GUI-19  GIVEN a player with name color codes
           WHEN settings.toml generation runs on his folder from the view
           THEN the file holds the explicit name string with its codes (never `name = true`,
                the Export upgrader's folder-matching rule)
SE-GUI-20  GIVEN a Fox save where some players' appearance ids are set and the rest are -1
           WHEN it is opened
           THEN the players with ids set are listed with a Strip again action, which sets their
                ids to -1 as one undoable edit
SE-GUI-21  manual  GIVEN the save's game running
                   WHEN Save is clicked
                   THEN it is refused naming the game, as the CLI's writing commands are
                        (SE-CLI-02), and the file is unchanged
SE-GUI-22  GIVEN the save open with "Edit aesthetics anyway" off (every start of the tool)
           WHEN the Appearance tab, the team operations menu, the transplant panel and the
                import dialog are drawn
           THEN the FPC toggle and the team-wide FPC switch, the boots and gloves assignment,
                Clear visual flags, the transplant's Apply and the imports' aesthetics section
                are disabled, each with the ownership line as its tooltip, while Apply
                aesthetics patch is enabled
SE-GUI-23  GIVEN the save open on a team and "Edit aesthetics anyway" on
           WHEN a team operation runs from its menu (Set edited flag, Set boots and gloves for
                everyone in incremental mode, Set player names to positions, Clear visual flags,
                the last also by Ctrl+F)
           THEN every player of the team carries the operation's values, as one undoable edit,
                and the saved file holds them
SE-GUI-24  GIVEN the save open on a player
           WHEN Set all stats to N, Bump all stats by N, Copy stats from another player or Swap
                stats with him runs from the player operations
           THEN his ability values are the operation's result (Set and Bump clamped to 40 to
                99, weak foot, form and injury untouched; Copy and Swap over the parity row's
                field set, positions, skills and COM styles included), as one undoable edit,
                and the saved file holds them
SE-GUI-25  GIVEN the Tactics tab on preset 1
           WHEN Copy preset to 2, then Swap formation kick-off with in-possession, run from the
                tab's menus
           THEN preset 2 equals preset 1 before the swap, the two formations are exchanged, each
                as one undoable edit
SE-GUI-26  GIVEN two PES 21 saves and one PES 19 save under the test's Documents folder, the
                 selector at PES 21, and one of the three opened earlier in the session
           WHEN the Open menu is drawn
           THEN it lists the three discovered saves, each with its version (and account folder
                for PES 19 and later), and the recent file above the file picker; WHEN Save as writes a new
                path THEN the file replaced, if any, is kept once as its .bak
SE-GUI-27  GIVEN the save open
           WHEN Output rosters to TSV runs
           THEN the file written holds, for every team, its name on a line of its own, then one
                tab-separated row per player (number, name, registered position, Finishing as
                the rating, captain mark, height, playing style, skills, COM styles), as
                4ccEditor's output does; WHEN
                Fix database is chosen THEN a
                confirmation dialog precedes it and cancelling changes nothing
SE-GUI-28  GIVEN the selector at PES 19
           WHEN a PES 21 save is opened
           THEN the view warns, naming both versions, the save keeps its own version (its codec
                and its widgets' gating are PES 21's), and the selector's setting is unchanged
SE-GUI-29  manual  GIVEN a Texport written by SE-GUI-17's export from a PES 18, 19, 20 and 21
           save
           WHEN it is imported through the game's own team import, in that version
           THEN the game accepts it and the imported team shows the exported squad and names
SE-GUI-30  GIVEN the save open on a team with violations under the active ruleset
           WHEN Live AATF is switched on
           THEN the docked panel lists the team's violations under `N issues · <ruleset>`;
                WHEN an edit fixes one of them THEN, once the check returns, its row is gone
                and N is one less, and an undo brings it back; WHEN the last one is fixed THEN
                the header reads `No issues: <team> passes <ruleset>`; WHEN the tool is
                reopened THEN Live AATF is still on
```

