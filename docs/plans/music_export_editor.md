# 4cc Studio — Music export editor plan

The music export editor (`crates/tools/music_export_editor`) is the successor to
**RigDJ**, the GUI editor for `.4ccm` music exports. It is named for what it edits —
music *exports*, not music files (plain "music editor" would wrongly suggest audio
editing). The community name stays in the user-facing labels: the sidebar shows
**"Music export editor (RigDJ)"** (the collapsed rail shows the tool icon — the
existing `rigdj.ico` artwork can be reused); the crate (`music_export_editor`) and
tool id (`music-export-editor`) use the descriptive name.

The editor is a thin tool: nearly all of its substance lives in the shared lib crates
specified in the [Music player plan](music_player.md) — `music_export` (format,
condition/instruction model, serializer) and `audio_engine` (audition playback).
Platform context is in the [core plan](core.md).

---

## Background

RigDJ (`rigdj.py` + `conditioneditor.py` in `Tools_4cc/rigdio`) lets managers build
`.4ccm` files graphically: define players, attach prioritized song lists to each slot,
edit conditions/instructions through typed dialog forms, and preview the generated
file text live. It shares its codebase (and its duplicated condition classes) with
Rigdio.

As with the player, RigDJ defines *what* the editor must do, not *how*. Editor-specific
messes the rewrite discards:

- **String-mediated editing**: RigDJ edits `ConditionList` objects through the `[...]`
  bracket-stripping/re-adding hack (see the [Music player plan](music_player.md)), whose
  re-add loop only handles `special` and `mostgoals` — any other spaced value breaks
  silently.
- **Rename implemented as delete + re-add**, shuffling selection state and losing
  list position.
- **Live bugs**: `print(...).format(...)` crash when saving a file containing an empty
  player list (`rigdj.py:966`); `updateSongs` calling `dict.delete()` (not a method);
  the victory-anthem fallback duplicating anthem entries into the VA list on every
  save pass.
- **tkinter canvas-in-frame scrolling machinery** (~100 lines) that egui gets for free.

## What the editor does

Feature parity with RigDJ, restated on the shared model:

- **Open / New / Save / Save As** for `.4ccm` files. Loading uses `music_export`'s
  parser (no audio loading); saving uses the canonical serializer. What you see in the
  preview is exactly what is written.
- **Team fields**: team name; the `sync` and `normalize` opt-out checkboxes with
  explanatory tooltips (defaults on; only written to the file when opted out).
- **Slot list**: the four reserved slots (Anthem, Victory Anthem, Goalhorn, chant) plus
  arbitrary player slots — add, rename (in place, keeping selection and order), delete;
  players sorted alphabetically in the list.
- **Song rows** per slot: file picker (mp3/ogg/opus/flac/m4a/wav filter), editable
  path, priority reorder (up/down; drag-and-drop where egui makes it cheap), delete.
- **Condition editing**: each condition/instruction renders as a chip on its row;
  clicking opens a typed form (comparison dropdown + integer entry for `goals`/
  `teamgoals`/`lead`, plain integer for `every`, team-list entry for `opponent`,
  match-type checkbox grid with a Toggle Knockouts button, subcondition editor for
  `not`, validated entries for the `start` offset and `speed` rate, plain add/delete
  for flag instructions). Each type shows its description text, as RigDJ's dialog
  does. The `time` condition (comparison + minute) and the `event` instruction
  (event-type dropdown: red/yellow/owngoal/sub) get working forms too — RigDJ had no
  `time` editor and `event` was dead code; the new parser and the
  [Match tracker](match_tracker.md) revive both (see the
  [Music player plan](music_player.md)).
- **Guard rails carried over**: `special` and `unrandom` are hidden from the general
  condition menu; rows with `randomise`/`special` don't offer Add Condition; `not`
  refuses instructions and double negation.
- **Randomise Horns** checkbox per slot: applies/removes `randomise` on all of the
  slot's entries (skipping `special`/warcry entries), reflecting mixed state. This
  matches the player's activation check, which counts every entry that can play in
  the slot's rotation — including `advance` entries.
- **Chant rows** show only the "Exclude from Random" (`unrandom`) checkbox instead of
  condition chips.
- **Special victory anthems**: a dedicated section under the VA slot listing
  `special`-marked entries with their labels; add-special creates the entry with the
  condition preattached.
- **Victory fallback on save**: if the VA slot has no non-special entries, the anthem
  entries are written as the victory anthems (once — the duplication bug dies).
  Empty slots are skipped when writing.
- **Live preview pane**: the serialized `.4ccm` text, read-only, updated on every
  model change, copyable.

## New capabilities

Cheap wins from sitting inside the suite:

- **Audition playback**: a play/stop button on every song row (via `audio_engine`),
  honoring the row's `start` and `speed` instructions — managers currently have to
  load the export into Rigdio to hear anything.
- **Validation panel**: file-existence checks (with the player's case-insensitive +
  `_normalized` resolution), unknown-condition warnings, empty-slot notices, duplicate
  player names, missing anthem — shown inline and summarized before save. Flag known legacy
  compatibility exceptions such as `time` (unregistered in affected Rigdio parsers; see the
  Music player plan). RigDJ only discovers missing files when Rigdio errors on load.
- **Unsaved-changes tracking** (dirty flag + confirm-on-close), which RigDJ lacks
  entirely.
- The editor and player being in one binary means a manager can flip to the player
  tool to test the full export without leaving the app.

## GUI view

```
┌──────────────┬──────────────────────────────────────────────┬─────────────────┐
│ [New] [Open] │ Team name [________]  □ sync  □ normalize    │ Preview         │
│ [Save] [As…] │                                              │ ─────────────── │
│──────────────│ Songs for: Player One                        │ # team          │
│ Anthem       │ [✖][▼1▲][Open][path/to/song.mp3   ][▶]       │ name;aaa        │
│ Victory A.   │   (goals == 2) (start 0:15) (+ Add)          │                 │
│ Goalhorn     │ [✖][▼2▲][Open][path/to/other.ogg  ][▶]       │ # reserved      │
│ chant        │   (+ Add)                                    │ anthem;...      │
│ ────────     │ [Add Song]         □ Randomise Horns         │ goal;...        │
│ Player One   │                                              │ ...             │
│ Player Two   │ ── Validation ──                             │                 │
│ [Add player] │ ⚠ other.ogg not found next to the .4ccm      │                 │
│ [Rename]     │                                              │                 │
│ [Delete]     │                                              │                 │
└──────────────┴──────────────────────────────────────────────┴─────────────────┘
```

Condition forms open as small egui modals (like RigDJ's `ConditionDialog`), with
OK/Delete/Cancel. The preview pane is collapsible for small screens.

## CLI

```
studio music-export-editor check ./team.4ccm    # parse + validate: missing files,
                                                # unknown conditions, empty slots;
                                                # exit code for CI
studio music-export-editor format ./team.4ccm   # rewrite in canonical serializer layout
```

`check` is the validation entry point for the whole music toolchain (the player's CLI
only pre-warms the loudness cache).

## Settings (tool section)

Almost nothing — the editor inherits the suite theme and file dialogs. One setting:
the default folder for the file picker. RigDJ's dark-mode button, settings window,
and `config.yml` handling all disappear into the platform.

## Testing

- **Round-trip**: RigDJ-written files from the community library load and re-save
  byte-identically (golden files); hand-written files re-save semantically equal
  (model-level comparison), including bracketed spaced values on *all* condition
  types — the regression class RigDJ's bracket hack papered over.
- **Editing operations** as model-level unit tests: rename keeps order and songs,
  randomise-horns respects the skip list, VA fallback writes once, empty slots are
  skipped, flags only written when opted out.
- **Validation**: fixture exports with missing/case-mismatched/`_normalized` files.
- Manual pass by a manager building a real export end to end (with audition).

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Naming | `music_export_editor` crate; label "Music export editor (RigDJ)"; existing RigDJ icon in the collapsed rail | "Music editor" would wrongly suggest audio editing; the RigDJ name and icon stay user-visible |
| Substance location | Model + serializer in `music_export`, playback in `audio_engine` | The editor is UI over shared libs; no editor-only format logic |
| Editing | Direct model editing, serialize on save/preview | Kills the string-round-trip bracket hack class of bugs |
| Preview | Always the real serializer output | What you see is what is saved |
| Audition | Per-row playback via `audio_engine` | Biggest manager-side quality-of-life win, nearly free inside the suite |
| Validation | In-editor panel + `check` CLI | Catches missing files before match day instead of on the streamer's machine |
