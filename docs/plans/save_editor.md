# 4cc Studio — Save editor plan

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

4ccEditor (`Tools_4cc/4ccEditor-1`) is a raw Win32 C++ application that edits PES save files
(`EDIT00000000`) for PES 2015–2021. Its codebase breaks down as:

| Component | Files | Size | Fate in the Rust port |
|-----------|-------|------|----------------------|
| Win32 UI | `main.cpp`, `window.cpp`, `resource.rc`, `menu_lists.cpp` | ~320KB | UI replaced by egui; retain the relevant domain tables in `menu_lists.cpp` as codec/conversion references |
| Per-version save codecs | `pes15.cpp` … `pes20.cpp` | ~190KB | **Collapsed** into schema tables + one codec engine (`pes_savefile`) |
| Data model | `editor.h` (`player_entry` ~100 fields, `team_entry` incl. tactics) | ~30KB | Ported to `pes_savefile`; derives replace manual boilerplate |
| AATF rules tool | `aatf.cpp` | ~28KB | Ported as configurable rules (see below) |
| Comparator | `comparator.cpp` | ~22KB | Ported, merged with Midcupping's aesthetics diff |
| Misc tools | `fpc.cpp`, `data_util.cpp` | ~12KB | Ported (FPC invisibility; the bit engine goes into `pes_savefile`) |
| Crypto interface | `crypt.h/cpp` + `libpesXcrypter.dll` | external | **Replaced** by native Rust crypto in `pes_savefile` |

Portability is good: half the C++ is Win32 UI that gets replaced rather than
ported, and portable crypto reference implementations already exist. The Rust
`pes_savefile` implementation is planned, not built — see the [Savefile plan](pes_savefile/README.md).

## Background: Midcupping

Midcupping (`Tools_4cc/Midcupping`) is a set of six standalone Python scripts
(PES 15/16/17 variants of two tools) used mid-cup, when re-running full exports
is too heavy:

- **`transplant-aesthetics-*.py`** — copies player appearance data (boots/gloves
  IDs, physique, strip style, ingame face) from a donor save into a base save,
  by player ID, ID pair, team, or team range; optionally in-place.
- **`compare-saves-*.py`** — diffs two saves aesthetically: per player, decodes
  boots/gloves/face IDs, taping, glasses, sleeves, inners, socks, undershorts,
  shirttail, winter gloves, skin color, plus a normalized hash of the ingame-face
  block, and prints what changed.

Both are absorbed into the save editor as first-class panels (the underlying
operations live in `pes_savefile` — see "Save-to-save operations" there).

---

## Feature inventory (the parity contract)

Everything 4ccEditor can do, with its fate in the port. This is the checklist the
implementation is measured against.

### File operations

| 4ccEditor | Fate |
|-----------|------|
| Load 15/16/17/18/19/20/21 EDIT file (7 menu items) | One **Open savefile** action; auto-detect the version by master keys / container shape. The loaded file's version governs its codec and editing widgets; warn on a global-selector mismatch, never reinterpret the open file. The Open menu lists the saves `pes_savefile`'s discovery finds under `Documents\KONAMI` for every version (labelled by version and, for 18+, account folder) as one-click entries above the ordinary file picker, and the picker opens in the selected version's save folder when it exists — see "Savefile discovery" in the Savefile plan |
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

| 4ccEditor | Fate |
|-----------|------|
| Clear visual settings flags (all players on team) | Kept |
| Set edited/created flag (all players on team) | Kept |
| Set / remove FPC invisibility (whole team) | Kept |
| Set boot/glove IDs for everyone (unchanged / incremental / same-for-all) | Kept |
| Set player names to positions | Kept |
| Save squad (`.4ccs`) | Replaced by **Team TOML export** (full-fidelity, human-readable — see the [Savefile plan](pes_savefile/README.md)) |
| Load squad (`.4ccs`, with stats/aesthetics/tactics checkboxes) | Kept as read-only legacy import, same section checkboxes; Team TOML import is the primary path. Cross-version import applies the playstyle/skill conversion maps |

### Tactics operations

| 4ccEditor | Fate |
|-----------|------|
| Export / import tactics (`.4cct` "nightly" files) | Import kept (read-only legacy); export replaced by Team TOML (tactics are a section of it) |
| Copy / swap presets (1↔2↔3) | Kept |
| Copy / swap formations (kick-off / in-possession / out-of-possession) | Kept |
| Full tactics editing (see "Tactics editor" below) | Kept at full parity + interaction improvements |

### Database operations

| 4ccEditor | Fate |
|-----------|------|
| AATF: current team / select teams | Kept (rules become configurable — see below) |
| Compare EDITs | Kept, merged with Midcupping's aesthetics diff (see "Comparator") |
| Output rosters to TSV | Kept |
| — | **Export teams list** (new): the open save's team names and IDs for 701–920 (in-game names are the `/xx/` names) merged into `data/teams_list.txt` through the same reconciliation the updater uses — added / kept / overridden shown for review, savefile wins conflicts, a change behind an unresolved conflict is dropped and the rest applied, never a blind overwrite (see the [Team compiler plan](team_compiler/pipeline.md), "Resolved decisions", "Teams list"). Lives here because this tool owns the open savefile; the compiler only consumes the list |
| Fix database (clear all visual flags, reset kit slots, PES17 kit-ID repair) | Kept as a maintenance action behind a confirmation dialog |
| — | **Strip and seed** (new, Fox saves): every not-yet-stripped player's appearance into `4cc_08_bins.cpk`'s `PlayerAppearance.bin`/`BootsList.bin`/`GloveList.bin` rows, then every appearance id to -1; plus the restored-id check. See "Stripped save" |

### New (from Midcupping)

| Feature | Notes |
|---------|-------|
| Aesthetics transplant | Donor save → current save; see below |
| Aesthetics diff | Part of the merged comparator |

### New (from the Team compiler)

| Feature | Notes |
|---------|-------|
| Apply aesthetics patch | The compiler's `aesthetics_patch.toml` → current save: on pre-Fox the default route by which teams' aesthetics reach the official save, on Fox the compiled players' motions and the resolved `name`/`shirt_name` entries (autopilot exports set them; managed exports leave them out). See "Read-only aesthetics" |

---

## Crate layout

The Save editor follows the standard tool-crate skeleton (core plan, "Tool crates") and elaborates
below it. The organizing principle: **`pes_savefile` owns every byte and every operation on the
model; this crate owns the session (open file, undo stack, selection) and the widgets.** If a
function here could be called without an egui context and without a `Session`, it belongs in the
lib.

```
crates/tools/save_editor/src/
├── lib.rs              # Tool (StudioTool impl) — wiring only
├── settings.rs         # editor settings (last folder, AATF rules path, comparator defaults)
├── cli.rs              # the subcommands of "CLI" below
├── messages.rs         # message catalog (AATF violations, interchange import findings, …)
├── session/            # everything about "the file that is open"
│   ├── mod.rs          #   Session: EditFile + version + dirty tracking + selection
│   ├── undo.rs         #   field-level undo/redo stack over model edits
│   ├── selection.rs    #   team/player selection, cross-team filter query
│   └── recent.rs       #   discovered saves (pes_savefile::discovery) + recent files list
├── aatf.rs             # runs libs/aatf rules over the session's teams, maps results to messages
└── view/
    ├── mod.rs          #   master-detail layout, open/save actions, unsaved-changes prompt
    ├── player_list.rs  #   list with drag-to-reorder, dirty dots, colour-coded names
    ├── info_strip.rs   #   general info + quick actions (Gold/Silver/Regular, set stats)
    ├── tabs/
    │   ├── abilities.rs    # ability spinners, positions; hosts team_widgets' card pickers
    │   ├── appearance.rs   # edit flags, physique, colors, strip style, motion
    │   ├── team.rs         # team identity, colors, manager/stadium, kit slots
    │   └── tactics.rs      # hosts team_widgets' tactics editor over the session's team;
    │                       #   copy/swap preset menus, undo wiring
    ├── panels/
    │   ├── aatf.rs         # hosts team_widgets' violations list; jump-to-player
    │   ├── comparator.rs   # gameplay + aesthetics diff view
    │   ├── transplant.rs   # aesthetics transplant flow
    │   └── interchange.rs  # Team TOML / .4ccs / .4cct / Texport import-export dialogs
    └── widgets.rs      #   editor-local reusable widgets (version-gated control, RGB field, ID picker)
```

Placement rules:

- **No savefile bytes, offsets, or version `match`es in this crate.** Version gating in widgets asks
  the schema (`schema.has(PlayerField::Star)`), never `version >= 19`.
- **`session/` is egui-free.** It is the unit-testable heart of the tool (undo, selection, dirty
  tracking) and is what the CLI subcommands drive too, so the CLI and GUI cannot diverge.
- **`view/tabs/` map 1:1 to 4ccEditor's tabs**, which is the parity contract; a new tab is a new
  file, not a new section of an existing one.
- **`view/panels/` are the Midcupping-derived and new features**; each is self-contained and can be
  developed in the build order below without touching the tabs.
- The AATF *engine* (rules-file loading, Rhai evaluation) lives in `libs/aatf`; `aatf.rs` here only
  feeds it the session's data and renders results.
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
- Closing with unsaved changes prompts.

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

Version quirks (advanced-instruction ranges differ 17 vs 18+; tactics import is
16–18-only for `.4cct`) are enforced by the `pes_savefile` model, not the UI.

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

FPC is the 4cc system for fielding **FBMs (Full Body Models)** — custom models
that don't just replace the head and neck but the player's entire body. It works
in two halves: the cup DLC replaces some default kit model pieces with blank
models, selected via kit config values (shirt/shorts/collar model fields — the
[Team compiler](team_compiler/README.md) handles that side), and the player's appearance
settings hide the rest — boots ID 55 and gloves ID 11 (conventional
**nonexistent IDs**, so nothing renders) plus strip settings that suppress the
remaining default geometry. Together they make the default player model
invisible, leaving only the FBM visible.

The editor's side of this, ported from `fpc.cpp`: a per-player toggle and a
team-wide on/off that apply `libs/fpc`'s FPC enable/disable
presets (their fields are listed in the [libs/fpc plan](libs/fpc.md)); the strip settings
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

On Fox the game takes a player's appearance, boots and gloves from the database tables
(`PlayerAppearance.bin`, `BootsList.bin`, `GloveList.bin`) when his appearance record's player id
is -1, and from the record when the id is set (in-game Test 1, PES 2021). The cup's official save
is therefore **stripped**: every appearance id at -1, so the tables the Team compiler writes decide
every player's look. It is prepared once per game version and distributed as the blank save
managers receive at every version change; no compile and no manager writes appearance into it
afterwards.

- **Strip and seed** is one operation (Database operations, Fox saves only): it writes every
  player's current appearance (the record's appearance bytes, undecoded bits included, and his
  boots and gloves IDs) into rows of the three tables inside `4cc_08_bins.cpk`, the official
  DpFileList entry that holds the cup's team-related bins (`TeamColor.bin`, `UniColor.bin`,
  `UniformParameter.bin`), which the DLC builder installs at lower priority than the
  compiler's output, then sets every appearance id to -1. A player already at -1 is left out,
  his record not being his look: his rows stand (a table created whole holds none for him). The seed is what a
  player no export describes looks like afterwards, so stripping changes nobody's look; compiled
  players' rows replace their seed rows. Appearance holds no tactics, so the seed can come from
  any save with the cup's appearances, keeping the DLC builder and the savefile builder apart.
- **Nothing in the editor sets an id back.** Imports (Team TOML, `.4ccs`/`.4cct`, Texport) and
  name writes, the aesthetics patch included, leave the appearance block's id as it is; the aesthetics
  transplant and the unlocked Appearance tab write the record's fields but not its id, and on a
  stripped save they say that the game does not read them.
- **A record at -1 is not the player's look.** Everything that reads a record's appearance, in
  the GUI or the CLI (the Appearance tab, the FPC classifier and its interference findings,
  the comparator and `diff`, a transplant's donor, a Team TOML export), marks a player at -1
  as stripped beside what it shows or writes; generation skips
  him ("Player settings in exports" in the
  [Aesthetics export plan](aesthetics_export/settings_toml.md)).
- **The game's own Edit mode does.** Test 1 found that any change to a stripped player in Edit
  mode restores his id: a rename brings back his pre-strip look, a look edit freezes the database
  look shown into the save (with boots 0). Managers therefore rename players in this editor and
  change looks only through the export (the Player aesthetics editor, then a recompile).
- **A check reports restored ids**: opening a Fox save lists the players whose appearance id is
  set while others are stripped, with a one-click **Strip again** (their record is ignored from
  then on; the tables decide).

Pre-Fox saves are not stripped: the database route is untested there, and the aesthetics patch
stays their route.

---

## Comparator (gameplay + aesthetics)

One panel merging 4ccEditor's `comparator.cpp` with Midcupping's
`compare-saves-*.py`:

- Load a second save (same version); the panel lists per-team, per-player
  differences.
- The two scopes are `PlayerDiff::scope()`'s (`DiffScope`, "Save-to-save operations" in the
  [Savefile plan](pes_savefile/operations.md)); the lists below are the two references' rows,
  each a subset of ours.
- **Gameplay scope** (from 4ccEditor): names, IDs, basics (age/height/weight),
  every ability stat, playstyle, positions and ratings, COM styles, all skills —
  with old → new values.
- **Aesthetics scope** (from Midcupping): boots/gloves/face IDs, taping, glasses,
  sleeves, inners, socks, undershorts, shirttail, winter gloves, skin color, and
  the normalized ingame-face fingerprint (catches "the face was edited" without
  decoding every facial parameter). It compares the records, so a player at -1 is
  marked ("Stripped save").
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
- **Texport import/export** (export is new; PES itself can import the file).
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

4ccEditor's AATF tool hardcodes both its parameters and its logic in C++; a ruleset change is a
recompile. This editor checks and applies a **rules file** instead: a ruleset as data plus a
generic interpreter, specified in the [AATF rules plan](aatf_rules.md) (`libs/aatf`), and built
without code in the [Ruleset editor](ruleset_editor.md). The official ruleset ships embedded and
is the default; the editor's settings hold the path of an alternative file (an invitational's),
and a "save a copy" action writes the embedded file out.

What this editor adds on top of the lib:

- **Quick actions.** One button per tier of the active ruleset, in its order (Make Gold / Silver /
  Bronze / Goalkeeper / Regular under the official one), each `aatf::apply_tier`.
- **The AATF panel.** Results grouped per player; clicking a violation jumps to the player; team
  selection matches 4ccEditor (current team or a multi-select list). A **Show suggestions**
  toggle, off by default, lists unused allowances apart from errors and warnings (see
  "Suggestions" in the AATF rules plan). The panel is `team_widgets`' violations list.
- **Validate rules.** A button running the lib's load-time validation on the chosen file, its
  findings shown with line or key path.
- AATF reports and never fixes, as in 4ccEditor.

---

## CLI

The batch surface for the operations that came from scripts — Midcupping existed as
Python scripts precisely because GUIs can't be looped:

```
4cc-studio save-editor export-toml ./EDIT00000000 --team 701 -o ./aaa.toml   # single-team full-fidelity Team TOML dump
4cc-studio save-editor export-toml ./EDIT00000000 --all-teams --out-dir ./teams
4cc-studio save-editor import-toml ./aaa.toml --base ./EDIT00000000 --out ./EDIT_new   # patch the base save
4cc-studio save-editor apply-patch ./aesthetics_patch.toml --base ./EDIT00000000 --out ./EDIT_new   # the compiler's patch
4cc-studio save-editor transplant ./donor_save --base ./EDIT00000000 --out ./EDIT_new --select 70105,70107:70207,703-705
4cc-studio save-editor diff ./a_save ./b_save [--aesthetics | --gameplay] [--json]
4cc-studio save-editor check-aatf ./EDIT00000000 [--teams 701,705-710] [--rules ./vgl27.rhai]
4cc-studio save-editor strip-and-seed --base ./EDIT00000000 --bins ./4cc_08_bins.cpk --out ./EDIT_new --bins-out ./4cc_08_bins_new.cpk
4cc-studio save-editor check-stripped ./EDIT00000000 [--strip-again --out ./EDIT_new]
4cc-studio save-editor export-teams-list ./EDIT00000000 [--yes]     # merge the save's team names/IDs into data/teams_list.txt
```

Rules every command shares:

- **Exit codes**, the Team compiler's meanings: 0 done; 1 done with an error reported (a patch's
  team the save lacks, an AATF error, a restored id listed); 2 refused before anything was
  written (a bad argument, a version mismatch, an unreadable file, a team or player the save
  lacks); 3 a write failed (what was already written stands).
- **Writing a save.** A command that writes a save reads its source save (`--base`, or
  `check-stripped`'s one save) and writes `--out`, which may be the same path; an existing
  output save or CPK (`--out`, `--bins-out`) is first copied to `<file>.bak` (one level, as every
  save does). There is no implicit base save and no command creates a whole EDIT. A command that
  writes a save refuses while PES is running, as the compiler's savefile step skips then
  (`team_compiler/pipeline.md`, `savefile_skipped_pes_running`): the game writes its own copy
  back on exit, over ours.
- **A stripped player is marked.** On a Fox save, `diff` marks a player at -1 as stripped
  ("Stripped save") beside his rows, `export-toml` with `stripped = true` in his table
  (informational; import ignores it), and `transplant` for a stripped donor or target.

The commands:

- `export-toml` requires an explicit source EDIT and either `--team <id>` with `-o`/`--out <file>`,
  or `--all-teams` with `--out-dir <directory>` for one file per team of the save, named
  `<id>.toml`. These forms are mutually exclusive; there is no implicit current-team selection.
  An existing TOML is replaced, with no backup; a `--team` the save lacks is refused.
- `import-toml` requires the Team TOML, `--base <EDIT>`, and `--out <EDIT>`. It imports onto
  the team the document's `id` names, with the same import logic as the GUI, preserving unmodeled
  save data; it prints each import note (a converted or dropped value) on its own line. The
  file's sections are its selection: a section left out of the file is untouched, so the CLI has
  no section flags. A document with no `id`, or one the save lacks, is refused.
- `apply-patch` takes the patch, `--base <EDIT>` and `--out <EDIT>` exactly like `import-toml`;
  it refuses a version or allocation-scheme mismatch before touching anything, prints the per-team
  summary (applied / skipped: not in save), and is the batch form of the GUI action — the savefile
  builder can apply a night's patches in one script. A skipped team or player is an error (exit
  1) and the rest is written. A pre-Fox patch is refused until the compiler writes one (deferred
  with pre-Fox's other gaps). The read-only lock is a GUI guard; the CLI commands that write aesthetics
  (`transplant`, an `import-toml` whose file holds aesthetics) are explicit by nature and are not
  gated.
- `transplant` copies the donor's appearance onto `--base`'s players in the scripts' selection
  forms (`--select`, required, a comma-separated list of player IDs, `target:donor` pairs, teams
  and team ranges; "Save-to-save
  operations" in the Savefile plan), the whole appearance block but its id, as the scripts do, so
  there is no field selection. The saves must be the same version, every target in the base and
  every donor in the donor save, or nothing is written.
- `diff` prints the comparator's report to stdout, players paired by id, each changed field as
  old → new (the appearance id is no row: a stripped player is marked instead), then the
  players only in one save (human-readable by default, `--json` for
  tooling), replacing the Midcupping compare scripts; `--aesthetics` and `--gameplay` keep one
  scope. Saves of two versions are refused. Differences are its output, not an error: exit 0.
- `check-aatf` runs the active ruleset (`--rules`, else the setting's file, else the embedded
  official ruleset) over the cup teams the
  save holds (701-920, or `--teams`) and prints each violation with its team and, for a player's,
  the player, as the AATF panel
  lists them; an error-severity violation makes the exit code 1, a warning alone leaves it 0. A rules file that fails load-time validation is
  refused with its findings (line or key path), so `check-aatf` doubles as the CLI's rules
  validation.
- `strip-and-seed` is "Stripped save"'s operation on a Fox save: it reads `--bins` (the cup's
  `4cc_08_bins.cpk`) and writes the seeded tables into `--bins-out`, then the stripped save into
  `--out`: when `--bins-out` cannot be written, the save is not (exit 3); a `--bins-out` written
  beside a save left unstripped changes no look, the records still deciding. A table `--bins`
  lacks is created from the seed rows. A pre-Fox save is refused.
- `check-stripped` lists the players whose appearance id is set while others are stripped (exit 1
  when it lists any); `--strip-again` sets their ids to -1, leaving the tables as they are, and
  writes `--out`, which it requires (exit 0). A pre-Fox save is refused.
- `export-teams-list` prints the merge summary (added / kept / overridden / unresolved) and stops
  unless `--yes` is given; a change behind an unresolved conflict is dropped, as for the updater
  merge (`libs/teams_list.md`). An unwritable data directory reports `teams_list_read_only` and
  writes nothing (exit 1).

AATF quick actions (`apply_tier`), the FPC toggle, `settings.toml` generation, Texport and the
legacy imports have no command: they are GUI actions (Phase 8), and generation's batch form is
the Export upgrader's (Phase 6).

---

## Build order and verification

`pes_savefile` (codec, model, conversions, transplant/fingerprint ops, interchange
formats, comparator, FPC invisibility) is built in Phase 2 as a standalone lib; the
editor's non-UI substance — the tool crate's settings, CLI and operations wiring,
plus the `aatf` rules engine — lands in Phase 5; the view lands in Phase 8 (see the
[core plan](core/development_plan.md#development-plan)). Within Phase 8, the view builds up as:

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

Format and rules: `CONTRIBUTING.md` "Testing". IDs are never reused; later phases append.

**Phase 5 scope.** Phase 5 delivers the tool crate's settings, CLI and operations wiring, and
`libs/aatf`; the view is Phase 8's, so these scenarios are what the CLI shows: stdout, the exit
code, and the files written (a save's decrypted payload, a Team TOML, a CPK's tables read back
with the `cpk` crate). The operations with no command (editing, tactics, the FPC toggle, AATF
quick actions, generation, Texport, the legacy imports) get their scenarios with the view.
`libs/aatf` is a lib: its rulesets' fixtures are its spec (AATF rules plan, "Development phase
and verification"), and `check-aatf` is its one scenario surface here. Test data: the fixture
saves are `pes_savefile`'s (`tests/fixtures/`, one real save per version, rebuilt from the
slices held there), a team or player
named by its id in that save; a stripped save is a fixture save after `strip-and-seed`. The
cup saves with known violations and the rules files are `libs/aatf`'s fixtures. The stripped-save
scenarios run on PES 21; PES 18 and 19 join them within Phase 5, once a step has measured those
versions' appearance offsets.

**CLI**

```
SE-CLI-01  GIVEN the PES 21 fixture save, a Team TOML of one of its teams, and an existing file
           at --out
           WHEN import-toml writes --out, then again with --out equal to --base
           THEN each time <out>.bak holds the file the write replaced, and the exit code is 0
SE-CLI-02  manual  GIVEN a running PES
                   WHEN import-toml, apply-patch, transplant, strip-and-seed or check-stripped
                        --strip-again is run
                   THEN it is refused naming the running game, with exit code 2, and nothing is
                        written
```

**Team TOML**

```
SE-TOML-01 GIVEN the PES 21 fixture save
           WHEN a team is exported with export-toml --team and the file imported back with
                import-toml into a new --out
           THEN the --out save's decrypted payload equals the fixture's
SE-TOML-02 GIVEN the PES 21 fixture save
           WHEN export-toml --all-teams --out-dir is run into a folder holding an older <id>.toml
                of a team the save holds
           THEN the folder holds one <id>.toml per team of the save, the older one replaced, each
                parsing as a Team TOML of its team
SE-TOML-03 GIVEN the PES 21 fixture save
           WHEN export-toml names a --team the save lacks, or both --team and --all-teams
           THEN it is refused with exit code 2 and no file is written
SE-TOML-04 GIVEN a Team TOML whose only content is [team] id, [players.03] name and one key of
                [players.03.appearance.physique], for a team whose slot 3 is rostered
           WHEN it is imported into the PES 21 fixture save
           THEN slot 3 of that team carries the name and the key's value, and every other byte
                of the payload, the player's other appearance fields included, is unchanged
SE-TOML-05 GIVEN a Team TOML, exported from the PES 21 fixture save, of a team both fixture
                saves hold, one of whose players has a face type above PES 18's cap
           WHEN it is imported into the PES 18 fixture save
           THEN the --out save holds the team as converted for PES 18 (that face type written
                0) with every other byte unchanged, each converted or dropped
                value is printed as a note line, and the exit code is 0
SE-TOML-06 GIVEN a Team TOML with a player name longer than the target version's name field
           WHEN it is imported
           THEN it is refused naming the player, with exit code 2, and --out is not written
SE-TOML-07 GIVEN a stripped PES 21 save and the unstripped PES 21 fixture save
           WHEN a team whose slot 3 is rostered is exported from the stripped one with
                export-toml, given an appearance table for its slot 3, and imported into each
           THEN every player of the exported file carries stripped = true; after each import
                slot 3's appearance fields are the file's, his appearance id still -1 in the
                stripped save and still set in the other
SE-TOML-08 GIVEN a Team TOML whose [team] id the PES 21 fixture save lacks
           WHEN it is imported
           THEN it is refused naming the team, with exit code 2, and --out is not written
```

**Aesthetics patch**

```
SE-PAT-01  GIVEN the PES 21 fixture save and a PES 21 aesthetics patch with motions and names
           for one of its teams
           WHEN apply-patch is run
           THEN that team's players carry the patch's motions and names, every other
                byte of the payload is unchanged, the summary lists the team as applied, and the
                exit code is 0
SE-PAT-02  GIVEN a PES 21 patch for two teams, the second absent from the save
           WHEN apply-patch is run
           THEN the first team is applied, the second listed as skipped: not in save, --out is
                written, and the exit code is 1
SE-PAT-03  GIVEN a patch whose pes_version differs from the save's, and one whose
                allocation_scheme_version differs from the suite's
           WHEN apply-patch is run with each
           THEN each is refused naming the mismatch, with exit code 2, and --out is not written
SE-PAT-04  GIVEN a stripped PES 21 save and a PES 21 patch with names and motions for one team
           WHEN apply-patch is run
           THEN the team's players carry the patch's names and motions and their appearance ids
                are still -1
```

**Transplant**

```
SE-TRN-01  GIVEN two PES 21 saves differing in their teams' appearances
           WHEN transplant --select is run with a player, a target:donor pair, a team and a team
                range
           THEN each selected target's appearance fields (the appearance id aside) and face,
                hair, physique and strip edit flags equal its donor's, and every other byte of
                the payload, the appearance ids included, is unchanged
SE-TRN-02  GIVEN a PES 21 base and a PES 19 donor, and a PES 21 base and donor with selections
                naming a donor player the donor save lacks and a target the base lacks
           WHEN transplant is run with each
           THEN each is refused naming the version pair or the player, with exit code 2, and
                --out is not written
SE-TRN-03  GIVEN a stripped PES 21 base, and a stripped PES 21 donor
           WHEN a player is transplanted onto the first, and from the second
           THEN onto the stripped base his appearance fields are the donor's, his appearance id
                is still -1, and the output says the game does not read his record; from the
                stripped donor the output marks the donor stripped
```

**Diff**

```
SE-DIF-01  GIVEN two PES 21 saves where one player's boots ID and one ability stat differ, and a
                player is only in the second
           WHEN diff is run, then with --aesthetics, then with --gameplay
           THEN the first lists both fields as old → new, the second only the boots ID, the
                third only the stat, each run also listing the player only in the second;
                every exit code is 0
SE-DIF-02  GIVEN the same saves
           WHEN diff --json is run
           THEN stdout parses as JSON carrying the same rows and the player only in one save
SE-DIF-03  GIVEN a PES 21 and a PES 19 save
           WHEN diff is run
           THEN it is refused naming both versions, with exit code 2
SE-DIF-04  GIVEN a stripped PES 21 save and the fixture it was stripped from
           WHEN diff is run
           THEN each player at -1 is marked stripped, with no row for his appearance id
```

**AATF**

```
SE-AATF-01 GIVEN a cup save with known error-severity violations, a team-level one (a tier
                count) among them, one whose only violations are warning-severity, and a clean
                one, and no --rules and no rules file configured (the official ruleset)
           WHEN check-aatf is run on each
           THEN the first prints each violation with its team and, for a player's, the
                player, and exits 1; the second prints its warnings and exits 0; the third
                prints none and exits 0
SE-AATF-02 GIVEN the save with error-severity violations in two teams
           WHEN check-aatf --teams names one of them
           THEN only that team's violations are printed, and the exit code is 1
SE-AATF-03 GIVEN a rules file that fails load-time validation, and the VGL27 ruleset with a save
                whose team breaks a VGL27 rule the official ruleset does not have
           WHEN check-aatf --rules is run with each, and with no --rules and the VGL27 file
                named in the settings
           THEN the first is refused with its validation findings (line or key path) and exit
                code 2; the second and the third print that violation, which a run with neither
                does not
```

**Stripped save**

```
SE-STR-01  GIVEN the PES 21 fixture save with one player's appearance id at -1, and a
                4cc_08_bins.cpk whose three player tables hold rows for him
           WHEN strip-and-seed is run
           THEN --bins-out's PlayerAppearance.bin holds a row per other player equal to his
                record's appearance bytes, and BootsList.bin and GloveList.bin his boots and
                gloves IDs; the stripped player's three rows are unchanged; the CPK's other files are
                unchanged; in --out every appearance id is -1 and every other byte is unchanged
SE-STR-02  GIVEN a PES 17 save
           WHEN strip-and-seed is run, and check-stripped
           THEN each is refused as a pre-Fox save, with exit code 2, and nothing is written
SE-STR-03  GIVEN a stripped PES 21 save in which two players' appearance ids are set again
           WHEN check-stripped is run, then with --strip-again
           THEN the first lists the two players and exits 1; the second sets both ids to -1 in
                --out, writes no table, and exits 0
SE-STR-04  GIVEN the PES 21 fixture save, and a fully stripped one
           WHEN check-stripped is run on each
           THEN neither lists a player, and both exit 0
SE-STR-05  GIVEN a --bins-out that cannot be written
           WHEN strip-and-seed is run
           THEN it fails naming --bins-out, with exit code 3, and --out is not written
SE-STR-06  GIVEN the PES 21 fixture save with one player's appearance id at -1, and a
                4cc_08_bins.cpk holding none of the three player tables
           WHEN strip-and-seed is run
           THEN --bins-out holds PlayerAppearance.bin, BootsList.bin and GloveList.bin with a
                row per other player built from his record, as in SE-STR-01, and none for the
                stripped one; the CPK's other files are unchanged
```

**Teams list**

```
SE-TL-01   GIVEN a data/teams_list.txt and the PES 21 fixture save with a team name missing from
                the list and one listed under another ID
           WHEN export-teams-list is run, then with --yes
           THEN the first prints the added and overridden rows and leaves the file unchanged;
                the second writes them, the save's ID winning; both exit 0
SE-TL-02   GIVEN a save whose team names would give two teams one ID in the merged list
           WHEN export-teams-list --yes is run
           THEN the colliding change is listed as unresolved and not written, the other
                changes are, and the exit code is 1
SE-TL-03   GIVEN an unwritable data directory
           WHEN export-teams-list --yes is run
           THEN teams_list_read_only is reported, nothing is written, and the exit code is 1
```
