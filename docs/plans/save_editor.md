# 4cc Studio — Save editor plan

The save editor (`crates/tools/save_editor`) is the successor to **4ccEditor**
and absorbs the **Midcupping** scripts: it edits PES save files (player stats,
appearance, teams, tactics), enforces AATF rules, diffs saves (gameplay and
aesthetics), transplants aesthetics between saves, handles team import/export
(new Team TOML format, legacy `.4ccs`/`.4cct` import, Texport read/write) and
FPC invisibility. It can also **generate `settings.toml` files from an existing
savefile** (via `pes_savefile`'s `PlayerSettings` model), giving teams a migration path
from the manual-editing workflow to the [Team compiler](team_compiler.md)'s
compile-time savefile writing. The savefile codec, the data model, and all
interchange formats live in the `pes_savefile` lib crate — see the
[Savefile plan](pes_savefile.md). Platform context is in the [core plan](core.md).

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
`pes_savefile` implementation is planned, not built — see the [Savefile plan](pes_savefile.md).

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
| Save EDIT file (encrypted) | Kept; plus **Save as** and automatic `.bak` of the original on first save |
| Import Texport file (per-version submenu) | Kept, auto-detected version; plus **Texport export** (new — PES itself can import the result; see [Savefile plan](pes_savefile.md)) |

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
| Save squad (`.4ccs`) | Replaced by **Team TOML export** (full-fidelity, human-readable — see the [Savefile plan](pes_savefile.md)) |
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
| — | **Export teams list** (new): the open save's team names and IDs for 701–920 (in-game names are the `/xx/` names) merged into `data/teams_list.txt` through the same reconciliation the updater uses — added / kept / overridden shown for review, savefile wins conflicts, unresolved conflicts leave the file unchanged, never a blind overwrite (see the [Team compiler plan](team_compiler.md), "Resolved decisions", "Teams list"). Lives here because this tool owns the open savefile; the compiler only consumes the list |
| Fix database (clear all visual flags, reset kit slots, PES17 kit-ID repair) | Kept as a maintenance action behind a confirmation dialog |

### New (from Midcupping)

| Feature | Notes |
|---------|-------|
| Aesthetics transplant | Donor save → current save; see below |
| Aesthetics diff | Part of the merged comparator |

### New (from the Team compiler)

| Feature | Notes |
|---------|-------|
| Apply aesthetics patch | The compiler's `aesthetics_patch.toml` → current save; the default route by which teams' aesthetics reach the official save. See "Read-only aesthetics" |

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
├── cli.rs              # export-toml / import-toml / check-aatf / compare / transplant subcommands
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
    style (boots ID, GK gloves ID, taping, spectacles, sleeves, inners, socks,
    undershorts, shirttail, gloves), motion (hunching, arm movement, kick
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
[Team compiler](team_compiler.md) handles that side), and the player's savefile
settings hide the rest — boots ID 55 and GK gloves ID 11 (conventional
**nonexistent IDs**, so nothing renders) plus strip settings that suppress the
remaining default geometry. Together they make the default player model
invisible, leaving only the FBM visible.

The editor's side of this, ported from `fpc.cpp`: a per-player toggle and a
team-wide on/off that apply `pes_savefile`'s version-aware FPC enable/disable
presets — the nonexistent
boots/gloves IDs, tucked shirt, long sleeves, short socks, custom skin (pre-18),
and cleared taping/inners/undershorts/gloves — and restore the visible defaults
when disabled (IDs 0, untucked, short sleeves, standard socks, light skin
pre-18). The ID constants become a suite-common setting consumed by
the presets rather than being hardcoded (any nonexistent ID works;
55/11 are the 4cc convention). The same presets serve the
[Team compiler's](team_compiler.md) per-player-folder `fpc.on`/`fpc.off` marker
files, so the editor's toggle and the compiler's markers cannot drift apart.

---

## Read-only aesthetics

A player's aesthetics are owned by the team's export: `settings.toml` and the models decide them,
the Team compiler resolves them into an **aesthetics patch** (format in the [Savefile
plan](pes_savefile.md)), and the save editor applies the patch. Hand-editing the same fields here
would create the two-sources problem — the next patch silently overwrites the hand edit, or the
hand edit silently diverges from the DLC — so the editor **does not edit aesthetics by default**:

- The Appearance tab renders its values disabled, with one line of explanation and a link to the
  apply action ("Owned by the team's export — edit its `settings.toml`, recompile, apply the patch").
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
  tool. Turning it on re-enables all the paths above for the session, with the note that a later
  patch overwrites whatever is edited by hand. It exists for the cases the rule does not cover — a
  team with no export, a cup-night fix when recompiling is not an option — and it is a session
  switch rather than a setting so that the default cannot quietly become "unlocked" on the
  savefile builder's machine.

Gameplay data (stats, skills, positions), team data and tactics are unaffected: they were never
the compiler's, and the savefile builder edits them as before.

---

## Comparator (gameplay + aesthetics)

One panel merging 4ccEditor's `comparator.cpp` with Midcupping's
`compare-saves-*.py`:

- Load a second save (same version); the panel lists per-team, per-player
  differences.
- **Gameplay scope** (from 4ccEditor): names, IDs, basics (age/height/weight),
  every ability stat, playstyle, positions and ratings, COM styles, all skills —
  with old → new values.
- **Aesthetics scope** (from Midcupping): boots/gloves/face IDs, taping, glasses,
  sleeves, inners, socks, undershorts, shirttail, winter gloves, skin color, and
  the normalized ingame-face fingerprint (catches "the face was edited" without
  decoding every facial parameter).
- Filter toggles: All / Gameplay / Aesthetics; a team filter; export the diff as
  text.
- Clicking a diff row jumps to that player in the editor.

---

## Aesthetics transplant

Midcupping's transplant as a guided panel:

- Pick a donor save (validated to be the same PES version).
- Build the selection the same four ways the scripts support: player IDs,
  `target:source` pairs, whole teams, team ranges — plus by clicking players in
  a two-pane team browser.
- **Preview before applying**: the affected players are listed with their
  aesthetics diff (reusing the comparator's fingerprint), so a wrong team ID is
  visible before it does damage.
- Applies through the normal edit pipeline: undoable, and saved only when the
  user saves.

---

## Interchange formats in the UI

The formats themselves are specified in the [Savefile plan](pes_savefile.md);
the editor exposes them as:

- **Team TOML export/import** per team (the `.4ccs`/`.4cct` successor: complete
  team + tactics + players including full aesthetics and ingame-face data).
  Import offers the same section choices as 4ccEditor's squad load (gameplay /
  aesthetics / tactics) and applies cross-version conversion with a warning list
  when the file's version differs.
- **Legacy import**: `.4ccs` and `.4cct` open through the same import dialog,
  read-only.
- **Texport import/export** (export is new; PES itself can import the file).
- **Aesthetics patch apply** — see "Read-only aesthetics"; the patch is written by the Team
  compiler, never by the editor.
- **`settings.toml` generation** — per player folder, for migrating a team to the
  [Team compiler](team_compiler.md)'s compile-time savefile writing. Uses the
  shared authorable `PlayerSettings` subset, omitting boots/gloves IDs even when the source
  save contains them; those are compiler-assigned from models/links, not export settings.
  Full Team TOML remains a full-fidelity save interchange. Name handling follows the
  Export upgrader's rule: `name = true` only when the folder's name part
  equals the savefile name, the explicit string otherwise — in particular a
  name carrying colour codes is always emitted as the explicit string, so the
  next compile preserves the codes.

---

## Configurable AATF rules

4ccEditor's AATF tool (auto-attribute enforcement of 4cc player rules) hardcodes both its
parameters and its logic in C++; a ruleset change is a recompile. The Rust version reads the whole
ruleset from **one self-contained [Rhai](https://rhai.rs) file**: the numbers that change between
cups at the top, the check logic below. One file is one ruleset: an invitational's variant is a
copy of the official file with the parameter block edited, and any file can be shared, diffed
against the official one and run by any Studio build.

Rhai is a small scripting language written in pure Rust for embedding. It is the safe answer to
Python's `eval()`: scripts can only access what the host exposes, cannot touch the filesystem or
network, and have configurable operation/recursion limits so a broken script cannot hang the tool.

### The rules file

Two sections, separated by a banner comment. The split is a convention the validator cannot
enforce, and does not need to: the point is that a member editing a number never has to read
past the banner, and a committee member fixing a rule never has to touch a Rust build.

**Parameters** are one top-level constant map. Rhai's `#{ … }` map literal reads like a config
file; the host reads `CFG` out of the script's global scope, and the functions below reach it as
`global::CFG`. The defaults are `aatf.cpp`'s constants from the Autumn 26 ruleset
(`Autumn_2026_AATF` branch). Tiers are map keys, not a fixed set: Autumn 26 added a fourth
(bronze) to a ruleset that had had three for years, and a fifth must cost a line in this block
and nothing in Rust.

```rhai
// ============================================================================
// PARAMETERS. Everything a cup changes lives here; edit numbers, keep the keys.
// ============================================================================

const CFG = #{
    name: "Autumn 26",
    // Ordered: the quick-action buttons and the card-picker badges follow this list.
    tiers: ["gold", "silver", "bronze", "regular"],

    rates:  #{ gold: 99, silver: 92, bronze: 86, regular: 77, goalkeeper: 77 },
    medals: #{ gold: 1,  silver: 2,  bronze: 2 },            // exact counts; the rest are regular
    form:   #{ gold: 8,  silver: 8,  bronze: 8,  regular: 4 },
    injury_resistance: #{ gold: 3, silver: 3, bronze: 3, regular: 1 },

    weak_foot: #{
        usage:    #{ gold: 4, silver: 4, bronze: 4, regular: 2, manlet: 4 },
        accuracy: #{ gold: 4, silver: 4, bronze: 4, regular: 2, manlet: 4 },
    },

    cards: #{
        skill: #{ goalkeeper: 2, regular: 3, bronze: 4, silver: 5, gold: 6 },
        trick: #{ goalkeeper: 0, regular: 2, bronze: 3, silver: 3, gold: 3 },
        com:   #{ regular: 0, bronze: 1, silver: 1, gold: 2 },
        pes_skill_card_max: 10,                              // the game's own limit
    },

    heights: #{ giga: 199, giant: 194, tall: 185, tall_gk: 189, mid: 180, manlet: 175 },
    brackets: #{                       // required counts per height system
        green: #{ giga: 0, giant: 6, tall: 6,  mid: 5, manlet: 6 },
        red:   #{ giga: 0, giant: 0, tall: 10, mid: 7, manlet: 6 },
    },

    bonuses: #{
        manlet: #{ gold: 0, silver: 2, bronze: 3, regular: 5 },   // stat bonus for manlets (Red only)
        giant_penalty: #{                                   // per height system
            red:   #{ gold: 0, silver: 0, bronze: 0 },
            green: #{ gold: 0, silver: 3, bronze: 0 },
        },
        manlet_card_bonus: 1,
        manlet_pos_bonus: 1,
    },
};

// ============================================================================
// CHECK LOGIC. Rules committee only below this line.
// ============================================================================
```

**Check logic** is plain Rhai functions. The host requires three and calls nothing else:

- `check_team(team) -> [violation]`: the whole check for one team; the script owns the loop
  over players and the aggregates (medal counts, bracket quotas, captain, GK), the way
  `aatf_single` does. A violation is `#{ slot: <roster slot or ()>, message: "…" }`; `()` marks a
  team-level finding.
- `tier_values(player, tier, team) -> map`: the stat, form, injury-resistance and weak-foot values
  a player of `tier` must carry, height bonuses included. Behind `apply_tier` (below), so the
  quick actions and the checker read the same arithmetic from the same file.
- `card_limits(player, tier, team) -> map`: skill/trick/COM limits with the player's free cards
  applied; behind the card pickers' remaining-count badges.

Anything else in the file (`tier_of`, `height_bonus`, `using_red`, …) is the script's own
business. Sketch of the shape, not the shipped rules:

```rhai
fn check_team(team) {
    let out = [];
    let red = using_red(team);
    for p in team.players {
        let tier = tier_of(p, red);
        let target = global::CFG.rates[tier] + height_bonus(p, tier, red);
        for skill in OUTFIELD_SKILLS {
            if p.stat(skill) != target {
                out.push(#{ slot: p.slot, message: `${skill} is ${p.stat(skill)}, should be ${target}` });
            }
        }
        if p.playable_at(p.registered_position) != "A" {
            out.push(#{ slot: p.slot, message: "Not rated A in the registered position" });
        }
    }
    for tier in global::CFG.medals.keys() {
        let count = global::CFG.medals[tier];
        let n = team.players.filter(|p| tier_of(p, red) == tier).len();
        if n != count { out.push(#{ slot: (), message: `${n} ${tier} players, should be ${count}` }); }
    }
    out
}

fn height_bonus(p, tier, red) {
    if !red || p.height > global::CFG.heights.manlet { return 0; }
    global::CFG.bonuses.manlet[tier]
}
```

**The rule set to express**, transcribed from `aatf_single` in `aatf.cpp`; this is the
behavioral spec for the shipped file:

- The registered position must be rated A; a GK rating cannot be the second A.
- No B ratings anywhere (A or C only).
- Age within 15–50; weight within `max(30, height−129)…(height−81)`.
- Registered position and playstyle within the version's valid ranges.
- Exactly one captain; at least one registered GK.
- Medal counts exact (per `CFG.medals`; Autumn 26: 1 gold, 2 silver, 2 bronze, rest regular).
- Ability stats must equal the tier's target rate plus height bonuses (attack and defense may be
  lower; stamina has its own target).
- Skill/trick/COM card counts within tier limits, with free cards (Malicia is free; the
  captaincy card is free for the captain; manlet card bonus) and the game's 10-skill-card cap.
- Weak-foot usage/accuracy within tier limits (manlet exceptions).
- Height systems: **Green** if any player ≥ the giant threshold, else **Red**; the team's
  height-bracket counts must match the system's quotas exactly.
- GK height: exactly `tall_gk` in Green; below giant in both systems.
- Gold players below the giant threshold; medal players cannot be GKs.

Autumn 26 specials, the conditional rules that decided the engine choice below:

- **Medals get a free A position *or* a free COM style**: a medal player with more COM styles
  than their free allowance takes +1 to that allowance; only if they take no COM bonus may they
  claim one free extra A position (raised card limit instead). Ordered, stateful logic inside
  one player's check.
- **Red non-medal CBs may reach 189cm** (`tall + 4`): only players registered at CB *and* played
  at CB in every formation the game can field: across all three presets, and across all three
  formations of a fluid preset (a non-fluid preset only fields its kick-off formation). The first
  rule that reads **tactics data**: the host exposes the team's presets, formations, fluid flags
  and starting eleven to the script, not just player fields.
- **Green silver giants capped at silver−3**: a silver medal player at exactly the giant
  threshold on a Green team may not exceed rate−3.
- **Silver giant penalty is bracket-conditional**: 0 on Red, 3 on Green (hence
  `bonuses.giant_penalty` is keyed by height system above).

Upstream errata, do not transcribe (the `Autumn_2026_AATF` branch, unfixed as of `cf61542`): its
bronze-count check reads the silver counter (`numSilver != reqNumBronze`, masked today because
both quotas are 2); and `aatf_check_player_in_pos` matches player IDs as `team_id*1000` (the rest
of the editor uses `*100`), so its starting-eleven lookup never succeeds and the preset check
silently degrades to `reg_pos`, plus its fluid branch counts formations backwards (checks 1 when
fluid, 3 when not; the correct semantics are the opposite).

### The host

`libs/aatf` compiles the file once per load, reads `CFG`, and runs the three functions on
demand. What the script sees is registered by the host and nothing else:

```rust
let mut engine = rhai::Engine::new();
engine.register_type::<PlayerEntry>()
      .register_get("slot", |p: &mut PlayerEntry| p.slot as i64)
      .register_get("height", |p: &mut PlayerEntry| p.height as i64)
      .register_fn("stat", |p: &mut PlayerEntry, name: &str| p.stat_by_name(name))
      .register_fn("playable_at", |p: &mut PlayerEntry, pos: &str| p.playable_at(pos));
engine.register_type::<TeamContext>()          // players, captain, version, tactics presets
      .register_get("players", |t: &mut TeamContext| t.players.clone())
      .register_get("presets", |t: &mut TeamContext| t.presets.clone());

engine.set_max_operations(1_000_000);
engine.set_max_call_levels(32);
```

Closed sets cross the boundary as strings the script compares (`"GK"`, `"A"`, tier names) because
that is what the file's author writes; the Rust side keeps its enums and converts at the
registration functions, nowhere else. The registered accessors are the script API and are listed
in `libs/aatf`'s crate doc; adding one is a plan edit to this section, never a silent addition.

`apply_tier(player, tier, rules)` is the lib's one *writing* operation: it calls `tier_values`
and writes the result into the player. It is the function behind this editor's Make
Gold/Silver/Bronze/Regular quick actions (one button per entry of `CFG.tiers`) and behind the
Team creator's default stats: one function, so neither tool can produce a player the checker then
rejects. AATF itself reports violations and never auto-fixes (matching 4ccEditor). Results render
in a panel grouped per player, and clicking a violation jumps to the player; team selection
matches 4ccEditor: current team or a multi-select list.

**Loading and validation.** The official file ships embedded in `libs/aatf` and is the default;
the editor's settings hold the path of an alternative file (an invitational's copy), and a
"save a copy" action writes the embedded file out for editing. A file is validated on load and
by the "validate rules" button: it must parse; `CFG` must be a map with `name` (string), `tiers`
(non-empty array of strings), `rates`, `medals`, `form`, `injury_resistance`, `weak_foot`,
`cards`, `heights`, `brackets`, `bonuses`; every tier in `tiers` must have an entry in `rates`,
`form` and `injury_resistance`; the three required functions must exist with the right arity;
and `check_team` must run to completion on a synthetic legal team without a script error.
Failures are `Message`s with the file's line and column, before any real check runs. The host
does not validate what the functions compute; the corpus test below does.

**Why one Rhai file, not TOML parameters plus a scripted or CEL logic file.** The natural design
is two layers: a TOML of numbers anyone can edit, and logic in a scripting language, with
Google's CEL (the `cel-interpreter` crate, non-Turing-complete, one expression per rule) as the
simpler candidate for the logic. Autumn 26 killed both halves of that design. *CEL*: the three
specials need sequencing (`comMod` is a running allowance consumed later in the same check), a
nested preset×formation traversal with a fluid-conditional range, and a starting-eleven lookup;
in CEL each of those is a new host-precomputed field or helper, so a ruleset change would still
need a Rust release, which is the one thing the configurable layer exists to prevent. *Two
files*: a parameter file and a logic file version-skew (an invitational's TOML without `bronze`
keys against a script that reads them fails or silently defaults), tiers as serde fields
resist the extensibility the fourth tier demanded, and the official-to-invitational workflow is
"copy one file, edit the top" only when there is one file. The costs accepted: map-literal
syntax is denser than TOML (the validator reports line and column either way); nothing but the
banner stops a member from editing logic (which is also what lets a committee member hot-fix a
`numSilver`-style bug without a build); and the parameters are readable only through Rhai,
which nothing outside the workspace needs to do.

---

## CLI

The batch surface for the operations that came from scripts — Midcupping existed as
Python scripts precisely because GUIs can't be looped:

```
studio save-editor export-toml ./EDIT00000000 --team 701 -o ./aaa.toml   # single-team full-fidelity Team TOML dump
studio save-editor export-toml ./EDIT00000000 --all-teams --out-dir ./teams
studio save-editor import-toml ./aaa.toml --base ./EDIT00000000 --out ./EDIT_new   # patch the base save; .bak of an existing target first
studio save-editor apply-patch ./aesthetics_patch.toml --base ./EDIT00000000 --out ./EDIT_new   # the compiler's patch; same base/out rules as import-toml
studio save-editor transplant ./src_save ./dst_save [--fields boots,gloves,physique,...] [--players 3,7,10-14]
studio save-editor diff ./a_save ./b_save [-a]                  # merged gameplay+aesthetics report; -a = aesthetics only
studio save-editor export-teams-list ./EDIT00000000 [--yes]     # merge the save's team names/IDs into data/teams_list.txt
```

- `export-toml` requires an explicit source EDIT and either `--team <id>` with `-o`/`--out <file>`,
  or `--all-teams` with `--out-dir <directory>` for one file per team. These forms are mutually
  exclusive; there is no implicit current-team selection.
- `import-toml` requires the Team TOML, `--base <EDIT>`, and `--out <EDIT>`. It patches the base
  using the document's team identity and the same import logic as the GUI, preserving unmodeled
  save data. A different output path leaves the base untouched; an existing target gets the usual
  backup before replacement. There is no implicit base save or creation of an entire EDIT from
  one team's TOML.
- `apply-patch` takes the patch, `--base <EDIT>` and `--out <EDIT>` exactly like `import-toml`;
  it refuses a version or allocation-scheme mismatch before touching anything, prints the per-team
  summary (applied / skipped: not in save), and is the batch form of the GUI action — the savefile
  builder can apply a night's patches in one script. The read-only lock is a GUI guard; the CLI
  commands that write aesthetics (`transplant`, `import-toml` with aesthetics) are explicit by
  nature and are not gated.
- `transplant` mirrors the transplant panel's selection modes (field groups,
  player IDs/pairs/ranges, whole teams); both saves must be closed in PES — the
  command refuses files locked by the running game.
- `diff` prints the comparator's merged report to stdout (human-readable by
  default, `--json` for tooling), replacing the Midcupping compare scripts.
- `export-teams-list` prints the merge summary (added / kept / overridden) and stops unless
  `--yes` is given; unresolved conflicts leave the list unchanged, as for the updater merge. An
  unwritable data directory reports `teams_list_read_only` and writes nothing.

---

## Build order and verification

`pes_savefile` (codec, model, conversions, transplant/fingerprint ops, interchange
formats, comparator, FPC invisibility) is built in Phase 2 as a standalone lib; the
editor's non-UI substance — the tool crate's settings, CLI and operations wiring,
plus the `aatf` rules engine — lands in Phase 5; the view lands in Phase 8 (see the
[core plan](core.md#development-plan)). Within Phase 8, the view builds up as:

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
- AATF default rules must reproduce `aatf.cpp`'s violations on a corpus of real
  cup saves.
- Comparator/transplant output parity against the Midcupping scripts.
- Texport export verified by importing into the game (manual, per version).
