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
| Save EDIT file (encrypted) | Kept; plus **Save as** and an automatic `.bak` of the file each save replaces (one level) |
| Import Texport file (per-version submenu) | Kept, auto-detected version; plus **Texport export** (new — PES itself can import the result; see [Savefile plan](pes_savefile/README.md)) |

### Player operations

| 4ccEditor | Fate |
|-----------|------|
| Set all ability stats (dialog) | Kept |
| Bump all ability stats ±N (dialog) | Kept |
| Copy player stats (source PID → dest PID) | Kept |
| Swap player stats (two PIDs) | Kept |
| Toggle FPC settings (Ctrl+I) | Kept (see "Full Player Customization" below) |
| Quick actions: Make Gold / Make Silver / Make Bronze / Make Regular / Set stats | Kept, **driven by the AATF rules file** instead of hardcoded rates — the buttons and the checker can never disagree |

### Team operations

What exists of this is in `spec/save_editor.md` "Team operations".

| 4ccEditor | Fate |
|-----------|------|
| Clear visual settings flags (all players on team) | Kept |
| Set edited/created flag (all players on team) | Kept |
| Set / remove FPC invisibility (whole team) | Kept |
| Set boot/glove IDs for everyone (unchanged / incremental / same-for-all) | Kept |
| Set player names to positions | Kept |
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
| Output rosters to TSV | Kept |
| Fix database (clear all visual flags, reset kit slots, PES17 kit-ID repair) | Kept as a maintenance action behind a confirmation dialog |

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
  holds the nearest color it can; a palette
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
  replaces the up/down spinner. A dot marks players with unsaved changes. Names
  render with their savefile colour codes applied as text colours (the in-game
  look; 4ccEditor shows the raw control characters), and the name edit field
  keeps the raw string so codes survive a roundtrip untouched.
- **General info strip** and **quick actions** always visible above the tabs.
- **Tabs** (contents 1:1 with 4ccEditor):
  - **Abilities & Skills** — 13 playable positions with A/B/C ratings, 7 COM
    styles, up to 41 skill checkboxes (version-gated), ~25 ability stats with
    spinners, playstyle / registered position / form / injury / weak foot,
    version-gated extras (star 19+, playing attitude & stronger hand 20+).
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
- **Undo/redo** across all edits (the model is plain data; an undo stack of
  field-level changes is cheap) — the single biggest safety upgrade over
  4ccEditor.
- Keyboard shortcuts preserved where they earn their keep (Ctrl+S save, Ctrl+F
  clear visual flags, Ctrl+I FPC toggle).
- Closing, or opening another save, with unsaved changes prompts.
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
  tier from the AATF rules file (`card_limits`).
- `violations` — the AATF results list grouped per player, emitting a "selected player" event for
  the host's jump-to.

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
  when the file's version differs.
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
           THEN a prompt offers to save, discard or cancel: cancel leaves the session as it
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
           THEN the list shows the new order, and after Save the file's roster has them swapped
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
           THEN his ability values are the operation's result, as one undoable edit, and the
                saved file holds them
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
                tab-separated row per player (number, name, registered position, rating,
                captain mark, height, playing style, skills), as 4ccEditor's output does; WHEN
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
```

