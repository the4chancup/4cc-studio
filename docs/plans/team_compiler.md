# 4cc Studio — Team compiler plan

The Team compiler (`crates/tools/team_compiler`) is the flagship tool of the suite:
the successor to the AET Compiler (Red/Blue). It compiles aesthetics exports into CPK
archives, with compile-time model format conversion (see the
[Model conversion plan](model_conversion.md)) and compile-time savefile writing
(see the [Savefile plan](pes_savefile.md)). Platform context (crate structure, tool
plugin trait, GUI shell, parallelism) is in the [core plan](core.md).

### Relationship to the older compilers

The older compilers are stopgaps, not templates. The original batch-file compiler
was rudimentary; **Red** is a direct translation of it into Python — feature-complete
and battle-tested, but slow at its core (it works by moving files around on
storage); **Blue** is an LLM-made prototype demonstrating that an in-memory,
parallel pipeline is possible. None of them is followed to the letter:

- **The contract is output parity with Red** (still the golden standard): given the
  same inputs, the game-facing CPK contents must match Red's — exact bytes for
  unaffected artifacts and normalized semantic parity for intentional ID/path/layout
  changes (see "Testing: parity against Red"). *How* that output
  is produced is free to change.
- **The implementation is built around the object model** (typed structs
  representing each element of an export — see the [Aesthetics export plan](aesthetics_export.md)), not around Red's
  disk-shuffling stages or Blue's path-string processing functions. References to
  Red/Blue modules in this document identify *where the behavior to reproduce is
  specified*, not code to translate.
- **Warning/error messaging is overhauled**, not ported: the pipeline emits
  structured `PipelineEvent`s (see the core plan) carrying export/folder identity,
  and the GUI/CLI decide presentation. Red's console-text messages serve only as a
  checklist of conditions worth reporting.

---

## Data flow

```
User exports (folders / .zip / .7z)
        │
        ▼
┌──────────────────────────────────────────────┐
│  team_compiler: Reader                       │
│  ─ Scan exports/                             │
│  ─ Index each export's structure into        │
│    ParsedAestheticsExport descriptors              │
│  ─ Classify file kinds (.fmdl / .model / glTF)│
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  team_compiler: Coordinator                  │
│  ─ Structural validation                    │
│  ─ Team ID resolution                       │
│  ┐ Run-level manifest planning:              │
│  │  ─ Player split + dependency resolution   │
│  └  ─ IDs, targets, collisions, ordering     │
│  ┐ Per model folder (rayon parallel):        │
│  │  ─ Materialize content under budget       │
│  │  ─ model_convert via IR (if needed)       │
│  │  ─ XML/FMDL/model/name editing            │
│  │  ─ Texture conversion and relocation      │
│  │  ─ contents_packing (CPK/FPK)             │
│  └ ─ Emit packed entries to writer           │
│  ─ bins_update (team colors, uniparam)       │
│  ┐ Referee processing (spec: Red)            │
│  └ ─ Slot-mapped packing (players.txt)       │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  team_compiler: Writer (single-threaded)     │
│  ─ Incremental CPK writing                   │
│  ─ Per-task atomicity: staged entries of     │
│    failed tasks are discarded                │
│  ─ Sideload folder injection                 │
│  ─ Duplicate invariant check                 │
│  ─ Deploy CPKs to PES folder (or output/)    │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  pes_savefile: Aesthetics patch + savefile   │
│  ─ Resolve settings.toml + activated IDs     │
│  ─ Write aesthetics_patch.toml beside the CPK│
│  ─ Local save configured: decrypt, apply the │
│    patch, re-encrypt, save (.bak backup)     │
└──────────────────────────────────────────────┘
```

---

## Crate layout

One crate, organized by **pipeline stage**, so a reader who knows the walkthrough below knows where
the code is. The layout is fixed here because the crate is large (15–25k lines) and largely
LLM-written: a predefined tree is what keeps generated code from scattering into `utils.rs` piles
and re-inventing placement per session.

```
crates/tools/team_compiler/
├── Cargo.toml
├── src/
│   ├── lib.rs            # `Tool`: the StudioTool impl — wiring only, no logic
│   ├── settings.rs       # TeamCompilerSettings, defaults, settings_view
│   ├── cli.rs            # clap Command; cli_run / gui_run dispatch (compile, check, upgrade-dpfl)
│   ├── messages.rs       # the message catalog: IDs, severity, disposition, templates, hints
│   ├── paths.rs          # per-version game-paths table (phf) and CpkStem/slot helpers
│   ├── templates.rs      # embedded resources (include_dir!) + templates/ override accessor
│   ├── reader/           # stage 1: where exports come from
│   │   ├── mod.rs        #   discovery: exports scan, NO_USE, team_name, /refs/ and /balls/ routing
│   │   └── source.rs     #   ExportSource: folder / .zip / .7z (via archives), eager structure,
│   │                     #   lazy per-folder content into VirtualTree, memory permit acquisition
│   ├── check/            # live validation orchestration (the lib validates; this schedules)
│   │   ├── mod.rs        #   two-tier check (shallow archive / deep folder), results cache
│   │   └── watcher.rs    #   notify + debounce → rescan via reader, recheck requests (desktop only)
│   ├── plan/             # stage 2, the coordinator: ResolvedAestheticsExport → BuildManifest
│   │   ├── mod.rs        #   plan_run → PlanReport; scoped drops; AbortRun
│   │   ├── manifest.rs   #   BuildManifest, BuildTask kinds, output-path collision checks
│   │   ├── ids.rs        #   PlannedModelIds: 40-ID team blocks, boots/gloves assignment
│   │   ├── refs.rs       #   duplicate-refs preflight, referee slot planning
│   │   └── sideload.rs   #   sideload/ tree precedence
│   ├── processing/       # stages 3–4: parallel per-task work, process_task(BuildTask) → TaskBatch
│   │   ├── mod.rs        #   dispatch by task kind; memory permits; message collection
│   │   ├── model.rs      #   model folders: format selection, conversion, merging, SKL pairing
│   │   ├── texture.rs    #   DDS/FTEX conversion, WESYS zlib, relocation to common
│   │   ├── material.rs   #   MTL / face XML / materials.toml editing and checks
│   │   ├── kit.rs        #   kit folders: config generation, colors, textures, FPC patching
│   │   ├── team_assets.rs#   portraits, logos, collars, Common folder
│   │   ├── referee.rs    #   referee-specific processing (markers, per-referee common layout)
│   │   └── materialize.rs#   THE seam: relocation to game paths + Fox FPK packing (skipped in test mode)
│   ├── bins/             # UniColor / TeamColor / UniformParameter accumulation
│   │   ├── mod.rs        #   working-bin lookup via the DpFileList walk, mutation commit
│   │   └── dpfl.rs       #   DpFileList.bin read/write, slot discovery, upgrade (override)
│   ├── output/           # stages 5–6: the writer and everything after it
│   │   ├── writer.rs     #   canonical-order incremental writing, teams parts, placeholders
│   │   ├── sink.rs       #   OutputSink: CPK (normal) vs loose folder (test/sider)
│   │   ├── deploy.rs     #   staging, .partial + rename, promotion, writability preflight
│   │   └── savefile.rs   #   builds the aesthetics patch from commit outcomes; applies it to the local save (via pes_savefile)
│   └── view/             # egui — renders state, owns no pipeline logic
│       ├── mod.rs        #   tool view layout (grid / toolbar / log composition)
│       ├── grid.rs       #   team grid on studio_core's grid widget; ID cell editing
│       └── toolbar.rs    #   split Compile button, writability badge, run strip
└── tests/
    ├── parity.rs         # upgrade + compile + compare against Red's reference (hash manifest)
    ├── pipeline.rs       # reader/plan/process/writer scenario tests on fixtures
    └── fixtures/         # small Studio-format exports; large trees regenerated, not committed
```

Placement rules:

- **If it validates an export, it is not here** — it belongs in `libs/aesthetics_export`. `check/` only
  schedules and caches those validations for the live grid.
- **If two tools need it, it is not here** — it moves to a lib (`pipeline` for scaffolding such as
  `MemoryBudget`, staging/promotion, `CpkStem`; `aesthetics_export` for format knowledge). The same rule
  in reverse: `bins/dpfl.rs` stays here until a second consumer appears (the Balls compiler only
  *checks* the DPFL, via a read helper that can move to `pipeline` when needed).
- **`processing/materialize.rs` is the single output-mode seam.** No other module asks which output
  mode is active; `output/sink.rs` is the only other place that differs by mode, and it differs by
  *where bytes go*, not by *what is produced*.
- **`reader/` is the only module that touches export sources.** `check/` asks it for the current
  export set; `plan/` and `processing/` receive its `ExportSource`s; nothing else opens a folder or
  archive.
- **`view/` is render-only.** It reads tool state built from `PipelineEvent`s and emits user intents
  (compile, cancel, mode change, ID edit); it never calls into `processing/` or `output/` directly.
- **Module names follow the walkthrough's stage names** (`reader`, `check`, `plan`, `processing`,
  `bins`, `output`) so the plan and the tree use one vocabulary. `pipeline` is deliberately *not*
  used for a tool-local module: it is the shared lib's name, and `pipeline::MemoryBudget` next to a
  `crate::pipeline::…` would be a permanent source of confusion.
- **Files, not folders, until a module exceeds roughly a thousand lines** — then it becomes a
  folder with a `mod.rs` and the same name, so paths in this plan stay valid.
- No `utils.rs`, `helpers.rs`, or `misc.rs`. A helper belongs to the stage that uses it; one used by
  several stages belongs to `pipeline`.

---

## Per-export pipeline walkthrough

This section specifies, step by step, what happens to a single export. The steps describe *behavior
to reproduce*, not code to translate (see "Relationship to the older compilers"): the implementation
operates on the object model, and only the game-facing output must match Red's. Red module names
(`Engines/python/`) mark where each behavior is authoritatively specified; Blue module names
(`lib/python/pipeline/` for orchestration and `lib/python/processing/` for processing steps) mark
where the pipeline shape it maps onto was prototyped. The structure is adapted throughout to the new
export format.

### 1. Reader

1. **Discovery** — the exports folder (`exports/`) is scanned for directories and
   `.zip`/`.7z` archives. Exports containing a `NO_USE` or `NO_USE.txt` marker file are skipped with
   `export_disabled`; they do not participate in duplicate-ref detection or run planning, are
   omitted from the grid, and produce only an informational log entry. In the GUI this scan is
   continuous (folder watcher, see "Live validation"); in the CLI it runs once. (Blue:
   `reader.exports_find`)
2. **Export display name and team name** — `export_display_name` is the complete archive/folder stem
   and is used only for source identity and presentation. The first non-empty token of that name
   (split on `.`, whitespace, `-`, `+`, `_`; a name with no token is rejected) is lowercased and
   wrapped in slashes to form the canonical `team_name` (`"/" + lowercase(first token) + "/"`, for
   example `co - Spring 2026.zip` → `/co/`). Lowercasing is Unicode `str::to_lowercase`, applied
   identically to the teams list's Name column on load — the same fold on both sides, which is
   what Blue's `str.lower()` comparison did (Python's `lower()` is Unicode-aware too); the
   distinction is theoretical for board names, which are ASCII, but the rule is written so nobody
   re-derives it. `team_name`,
   **not** the complete export name, is the lookup key in `teams_list.txt` and the `/xx/` label
   shown by the compiler. If the key is `/refs/`, the export is routed to referee processing;
   referee exports use the game's fixed numeric ID 999 internally, but 999 is not a valid normal
   `TeamId`. At most one non-disabled refs export may participate in a run: the Reader already knows
   every source's `team_name`, so it detects duplicate refs exports at discovery time — each
   conflicting export is skipped with `multiple_ref_exports` and never enters validation or
   planning. If the key is `/balls/`, the export is not team content at all — it is skipped with an
   info message (`export_balls_skipped`) pointing at the [Balls compiler](balls_compiler.md), which
   owns balls exports; `/refs/` and `/balls/` are both reserved keys.
3. **Memory acquire** — the memory budget works at **task granularity** (a model folder, a kit, the
   logo, …), improving on Blue's per-export all-or-nothing accounting: a task's budget share is
   acquired when its contents load and released as soon as the writer has written its packed
   entries. Content shared by a player's face/boots/gloves tasks stays charged until all three have
   been written. Peak memory is bounded by the tasks in flight, not the exports in flight.
   Granularity is also limited by the container: plain-folder exports (the main DLC workflow) and
   `.zip` archives support lazy per-folder loading (direct file reads / per-entry random access);
   solid `.7z` archives must be decompressed in one pass, so a 7z export is resident as a whole
   until its folders are written, and the budget accounts for its full size. An oversized
   acquisition (`size > cap`) must wait specifically for `in_flight == 0` and then proceed alone; it
   must not wait for ordinary capacity, which can never satisfy it. (See the core plan's Parallelism
   section; its semaphore pseudocode must preserve this oversized-request branch.)
4. **Load** — the export's **structure** is always loaded eagerly (folder tree, file names/sizes,
   roster, link files — what validation, the grid, and ID assignment need; kilobytes); file
   **contents** load lazily per folder into the `VirtualTree` as their processing tasks start
   (native `sevenz-rust`/zip parsing; `.db` and `.ini` files ignored). No mutable extraction or
   patch-staging tree is used: export transformation happens in memory between source reads and CPK
   output. Final CPK output may use temporary files for atomic write-and-rename. **Source exports
   are read-only**: the compiler never writes into an export folder or archive — every auto-fix
   (nested-folder flattening, generated kit configs, derived kit colors, …) exists only in the
   in-memory tree and is reported as a message, so the user's submission stays exactly as they made
   it.

### 2. Per-export serial steps (coordinator)

1. **Root normalization** — if the expected content isn't at the export root, nested child folders
   are scanned for a usable export root; exactly one candidate is flattened in memory
   (`nested_folders_fixed`) with loose root files preserved. **Intentional deviation from Red**
   (whose `team_id_get.py` takes the first matching child): several usable candidates are ambiguous
   and reject the export (`nested_root_ambiguous`), and a loose root file colliding with a flattened
   file at the same virtual path rejects it too (`nested_root_conflict`) — consistent with the
   plan-wide rule that collisions are rejected, never silently resolved.
2. **Validation, in two passes** — validation is read-only and parallel like compilation: folders
   within an export are checked with rayon (`par_iter` over player folders, shared folders, and
   kits) and produce independent per-scope issues/messages, so collection order does not matter.
   - **Structure/roster/path pass (`aesthetics_export`)**: operates only on the eager descriptors and
     small required metadata. It validates virtual-path safety, folder and file naming, root shape,
     roster syntax and identity, link targets, source-format pairing, extension-based allowlists,
     and **texture stem conflicts** (two image files with the same stem but different extensions in
     one model folder — `texture_stem_conflict`), without loading model, texture, or archive
     payloads. This is the pass used by shallow archive checks.
   - **Deep format pass (format crates)**: materializes each scope under the memory budget and
     delegates FMDL, `.model`/MTL/XML, glTF, texture, and kit-config parsing to their owning crates.
     Plain-folder full checks run both passes; packed archives run the structure pass during live
     checking and this deferred deep pass at compile start before eligibility is decided.

   Together the two passes cover these categories, adapted from Red's `export_check.py` to the new
format:
   - **Nested folder fix**: `Players/Players/` and similar are auto-flattened with a warning.
   - **Player folders**: normal teams are numbered via the folder name (`NN - Name`, 01–23) or
     authoritative root `players.txt` mapping (01–23); referee exports require `players.txt`
     mappings in 01–35 and may map the same folder to several slots (see "Player numbering").
     Duplicate slots are rejected; unnumbered-and-unlisted folders are rejected; model files are
     categorized (face/boots/gloves) and validated per mode — Fox: allowed FMDL names (`face_high`,
     `hair_high`, `oral`, `fcl_hair`, `boots`, `glove_l`, `glove_r`); pre-Fox: `.model`/`.mtl`/XML
     structure checks (`face_edithair.xml` and `hair.xml` rejected).
   - **Shared folders** (`Faces/`, `Boots/`, `Gloves/`): same model checks, including glTF-only
     material files (`materials.toml` catch-all and `*.materials.toml` name-matched; see the
     [Unified model format plan](model_format.md)'s "Material files"); every folder must be referenced by at least one
     player link file (warning if orphaned; error if a link references a missing folder).
   - **Root files**: `colors.txt` and `notes.txt` are optional. `players.txt` is optional for normal
     teams but required for referee exports; when present it must satisfy the shared roster grammar
     and mapping rules. A present `colors.txt` must parse, and a non-empty `notes.txt` must be valid
     UTF-8 (an optional UTF-8 BOM is stripped). Unexpected root files are warned about; a root
     `README.txt` (case-insensitive) is known and ignored — the Team creator writes one, and human
     readme files in exports predate it. Referee exports additionally permit `ref_marker.dds` and `ref_lists.txt` (the Refs arranger's file
     for the Fox referee hook; not read, not packed). A missing referee roster reports
     `players_txt_missing`; the invalid `ParsedAestheticsExport` remains available to the Refs arranger
     for repair.
   - **Kits**: kit folder names must parse as `<slot>[ - <label>]` with the slot in `p1`–`p9`,
     `g1`, `all` (`kit_folder_invalid`), one folder per slot (`kit_slot_duplicate` discards both);
     `all/` textures are merged into every kit lacking the stem (`kit_textures_inherited`), and
     `all/` itself may hold nothing else (`kit_all_file_ignored`) and must have kits to serve
     (`kit_all_unused`) — grammar and merge in "Kits" in the [Aesthetics export
     plan](aesthetics_export.md); a missing `config.toml`
     is auto-generated from the template (`kit_config_generated`); a missing `colors.txt` has its
     two menu colors derived from the main kit texture (`kit_colors_derived` — dominant-color
     extraction via `libs/color_tools`, see the [library crates plan](libs.md)), and a present file
     that yields fewer than two valid colors falls back to the same derivation after its
     `color_entry_invalid` warnings; a kit with no derivable colors gets the loud magenta/black
     "no colors chosen" pair (`kit_colors_missing`, W); a kit whose effective textures lack
     `kit.dds` — an empty folder included — is a **placeholder kit** (`kit_placeholder`, I): it
     gets the bundled checkerboard texture and is otherwise a normal kit (see "Kits" in the [Aesthetics export
     plan](aesthetics_export.md)); a present `icon.txt` must hold a number in 0–23 (`kit_icon_invalid` warns and
     falls back to the default); at most one layout marker (`pre-fox` / `fox`, `.txt` tolerated;
     both → `kit_layout_conflict`, kit discarded); textures validated as DDS and named with the `kit` prefix
     (`kit.dds`, `kit_mask.dds`, `kit_chest.dds`, …); supplied configs' FPC fields are checked
     against the team's FPC status (reconciled at compile time — see "FPC toggle" in the [Aesthetics export plan](aesthetics_export.md)).
   - **Portraits/Logo/Common/Collars**: portrait naming (`player_NN.dds`, NN in 01–23), root logo
     files (stem grammar `logo[_small][_<fit>]`, one file per role, main present whenever
     `_small` is, format in the raster allowlist — see "Root files" in the [Aesthetics export
     plan](aesthetics_export.md); decodability is checked in the deep pass), texture integrity
     (`texture_check`), file-type allowlists per mode. Collar findings map to the team-name cell (collars have no cell
     of their own).
   - Processing consequences follow each message's effective disposition: `DropFile` removes only
     that file, `DropSlot` removes only the invalid normalized roster assignment, `DropFolder` skips
     that folder or equivalent atomic owning task scope, `DropExport` blocks the whole export, and
     `AbortRun` terminates the run. `pass_through` overrides only eligible content-level
     `DropFile`/`DropFolder` dispositions; foundational path, roster-identity, and required-metadata
     failures are never overridden. See "Cell states" for GUI outcomes.
3. **Export identity resolution** — after validation, a normal export's canonical `team_name` from
   the Reader is looked up in `teams_list.txt` — one row per team, tab-separated `ID` and `Name`
   columns, Name lowercased on load with the Reader's fold (see step 2 above), further columns
   ignored (Red's `MinBootsID`/`MaxBootsID` are dead: Studio derives the 40-ID block from the team
   ID, see the export plan's "ID allocation"); rows whose Name is not slash-wrapped (Red's `Backup 1`, `VGL 55`
   placeholders) load but can never match. (The GUI also performs an early lookup after structural validation so
   the live grid can show or edit the ID immediately.) The complete `export_display_name` is never
   used as the lookup key. Valid normal `TeamId` values are 701–920. In Red, an unresolvable team
   prompts interactively; in 4cc Studio the grid's **team ID cell** turns red and editable — the
   user types an ID, confirms with Enter, and it is written to the teams list (see "GUI: the
   live-validating team grid"). In the CLI an unresolvable normal team is a hard error. A successful
   lookup produces `ExportIdentity::Team { id, key }`; a referee export produces
   `ExportIdentity::Referees` without entering the normal teams list. The referee pipeline renders
   the game's fixed numeric ID 999 only at game-format boundaries. Either identity is wrapped in
   `ResolvedAestheticsExport` before run planning. (Red/Blue: `team_id_get.py`)
4. **Portrait extraction** — for `ExportIdentity::Team { id, .. }`, a `portrait.dds` inside a player
   folder is renamed (`player_{id}{NN}.dds` for PES ≤18, `{id}{NN}.dds` for 19+; in Red
   `portraits_move.py` stages face-folder portraits under the `player_` name and `export_move.py`'s
   Portraits pass applies the version-specific final name) and staged with the portraits; an
   `ingame_face` marker removes face-classified `ModelPart`s and face-specific ancillary files, and
   **no face folder is emitted at all** — on either engine, the presence of any face folder (even
   one containing only boots/gloves models) overrides the ingame-customized face, and an empty face
   folder makes the face blank. Explicitly-named face models (`face_high`, `hair_high`, `oral`)
   combined with `ingame_face` drop the whole player folder (`ingame_face_explicit_face_model`);
   arbitrarily-named models (which would route to `fcl_hair`) reroute to the boots folder as `boots`
   (renamed or merged), carrying their paired SKL. The remaining non-face parts are
   relocated out of the face folder: gloves parts go to **player-specific gloves folders**
   (`glove_l`, `glove_r`), and the rerouted arbitrary-named models plus any explicit boots models
   are **merged into one boots model** (on Fox via `fmdl`'s mesh merging; on pre-Fox via the IR —
   import each `.model` to the IR, `merge_ir_parts`, export back, a lossless same-format
   round-trip) and moved
   to a **player-specific boots folder** — the one case where pre-Fox produces
   player-exclusive boots/gloves folders with IDs from the per-team block scheme (see "ingame_face
   marker"). Without `ingame_face`, a player with boots/gloves but no face models still gets a
   (blank) face folder: the absence of the default PES face is the final element of FPC, so the
   folder must exist (Red's current behavior, kept). Conflicting portraits for the same number fail
   the export. Referee identity uses its separate slot-derived face paths and does not construct a
   normal `TeamId`. (Red: `portraits_move.py`, `export_move.py`)
5. **Notes collection** — a root `notes.txt` is strict UTF-8; an optional UTF-8 BOM is stripped and
   CRLF/CR newlines are normalized to LF. Invalid encoding reports the file-scoped
   `notes_encoding_invalid` and drops only the note. Empty or whitespace-only notes produce no
   entry. Valid non-empty content is stored as a manifest-staged non-model artifact under a team
   header, replacing Red's "Other Notes" extraction from the Team Note txt. `teamnotes.txt` is
   rendered only after final export outcomes are known: it is a deterministic UTF-8/LF run artifact,
   atomically replaced, ordered by canonical export order, and contains only accepted exports with
   non-empty valid notes. (Blue: `note_txt_append`)
6. **Player folder categorization** — each player folder is categorized into optional
   face/boots/gloves inputs for run planning; any category may be absent (for team players and
   referees alike — the unified player folder format has no separate face-folder concept, so a
   player folder with only boots/gloves is structurally valid). Run planning turns these inputs
   into model tasks. Normal teams receive
   boots/gloves IDs from the **hardcoded per-team block scheme**; referee tasks receive their fixed
   slot-derived `k99XX`/`g99XX` IDs instead (see "Player folders" in the [Aesthetics export plan](aesthetics_export.md)). Assigned IDs
   are written into the savefile, so a compiled team CPK and its savefile must be deployed together;
   the aesthetics patch written beside the CPK is how they travel together when the savefile is on
   someone else's machine (see "Post-processing"). A missing local savefile stays a **warning** that
   skips only the savefile step: the patch is still written, and because allocation is
   deterministic (fixed per-team blocks, exclusive IDs by player number, shared IDs in alphabetical
   order), recompiling the same exports later with a savefile configured produces identical IDs, so
   a CPK-now/savefile-later split remains consistent. The deploy-together rule is distribution
   guidance, not a compile blocker.

### 3. Per-model-folder parallel steps (rayon)

After validation and export-identity resolution, a serial **run-level** planning phase resolves
`duplicate_aesthetics_export`, shared/Common dependencies, deterministic IDs, cross-export and known-path
collisions, sideload precedence, canonical export order, and staged non-model artifacts into an
immutable **build manifest**. Its tasks cover model folders and the non-model work (kits, logo,
Common, collars, portraits, referee marker) alike; each task is an atomic unit — it commits whole or
not at all. Canonical export order uses normalized source-relative path plus source kind as its
stable key, never the potentially duplicated `export_display_name`. The manifest allocates every
task an output namespace that cannot conflict with another task's. A cross-export collision drops
the losing task or export rather than an arbitrary entry that could leave its model/XML/package
incomplete; a sideload replacement stays path-granular and intentionally supersedes the export
entry. Folder-internal names that depend on deep parsing — glTF material-role image names, model
fallback/merge products — are resolved deterministically inside that task and checked for collisions
within its allocated namespace. All identity, allocation, roster, portrait-conflict, and run-global
collision checks finish before tasks are dispatched. Deep task failures may be at most `DropFolder`
unless the writer can roll back the whole export. Each planned face/boots/gloves model folder is
then processed as an independent parallel task (Blue: `coordinator._model_folder_task`, Red:
`export_move.py`):

1. **Format conversion** — when several source representations coexist (e.g. `boots.fmdl` +
   `boots.model` + `boots.glb`), selection is **target-native first, then glTF, then conversion
   from the opposite native format**; duplicate sources within the selected representation are
   ambiguous and drop the folder (`model_source_ambiguous`). If the selected model's format doesn't
   match the target PES version, convert via the IR (see the [Model conversion
   plan](model_conversion.md)).
2. **ID replacement** — dummy team IDs are replaced in file contents (FMDL texture path tables via
   `fmdl_id_change`; `.mtl` texture IDs pre-Fox) and in file names.
3. **Fox mode fixups** — FMDL files renamed by stripping prefixes to the allowed names (arbitrary
   names route to the `fcl_hair.fmdl` merge — Red's fallback generalized), and all models resolving
   to the same allowed name — including `.common` links to Common models, which Fox cannot load at
   runtime — **merged into one FMDL** (see "Common model links and model merging" in the [Team
   export plan](aesthetics_export.md)); missing template files injected (`face_diff.bin`, `fcl_hair_sim.fclo` when
   `fcl_hair.fmdl` exists, and the default `boots.skl`/`fcl_hair_sim.skl` skeletons from template
   `body.skl` when no part brings a custom SKL for that slot — see "SKL pairing" there); `face_diff.xml`
   converted to `face_diff.bin`.
4. **Pre-Fox fixups** — `face.xml` created, with every model of the player folder listed under
   its category's type attribute (boots and gloves models included — the typed XML is why pre-Fox
   needs no per-player boots/gloves folders), or a user-supplied one checked and re-serialized
   (see "User-supplied `face.xml`" under the XML/MTL checks); `glove.xml` for shared gloves folders;
   `.common` links resolve to Common paths in the generated XML (Red's common-link behavior). (Red's
   3-digit glove ID padding, `g567` → `g0567`, has no equivalent: the Studio format has no ID-named
   folders — IDs are auto-assigned. The Export upgrader normalizes old IDs when parsing them.)
5. **Texture conversion** — all image formats are interchangeable as texture sources (see the
   [library crates plan](libs.md)): DDS, FTEX, PNG, JPEG, BMP, WebP, TGA, TIFF. Material
   references (FMDL path tables, `.mtl`, `.materials.toml`) name textures by **stem** (filename
   without extension); the compiler resolves each stem to the actual file in the model folder
   and converts according to the selected PES version, not just the Fox/pre-Fox split. BC7 is
   supported on PES 19–21 only: PES 15–18 transcode BC7 and encode raster sources to BC3, with
   BC1 for eligible fully opaque color textures under the library plan's alpha/role rules.
   PES 19–21 retain supported BC7 inputs and use BC7 for newly encoded ordinary raster textures.
   Role-specific normal/data handling still applies; already-compatible blocks avoid re-encoding.
   Raster sources decode via `image`, missing mipmaps are generated, and Fox output uses FTEX with
   target-appropriate headers. Two image files with the same stem
   but different extensions is a conflict (`texture_stem_conflict`). Optional zlib DDS
   compression (PES ≤17). An in-memory conversion cache eliminates repeat conversion on the
   edit→compile→test loop; see the [library crates plan](libs.md) for design and engagement
   rules (≤2-team limit, no disk persistence).
6. **Texture relocation to common** (player folders only) — **all** textures of a player folder's
   own models (and a referee's), whether used by one of its models or several, are moved to a
   **per-player common subfolder** in the packed output, and the texture paths inside the FMDL/MTL
   files are rewritten to match. (The destination is fixed at player-folder split time — the
   player's textures are set aside for the common subfolder and emitted once; each parallel model
   task only rewrites its own model's paths. A folder mapped to several team or referee slots still
   emits its textures **once**, into a single common subfolder keyed by the source folder's name —
   every mapped slot's instantiated model references that one location. This is Red's
   in-game-verified referee layout: its preprocessing runs once per distinct referee folder,
   rewriting texture paths to the name-keyed per-referee common subfolder — MTL/XML to
   `model/character/uniform/common/{team ID}/{folder name}/` (packed under `common/character1/`),
   Fox FMDL to `/Assets/pes16/model/character/common/{team ID}/{folder name}/sourceimages/` — and
   copying the folder's common content once, before the per-slot instantiation pass. Referees are
   the team with ID 999 here, nothing more: the same path shape with `999` where a team's ID goes —
   Red's source spells the placeholder `XXX`/`000` before substitution. Textures referenced in place
   from the export's `Common/` folder sit one level up, `…/common/{team ID}/` (pre-Fox) and
   `…/common/{team ID}/sourceimages/` (Fox), which is Red's Common location on each engine.) The
   shared textures commit
   once, after all of the player's face/boots/gloves tasks report: identical destination bytes
   deduplicate, while the same destination with different bytes reports `shared_texture_conflict`
   and drops the losing task, chosen by canonical order (`face` > `boots` > `gloves`), never rayon
   completion order. Rationale: a single, predictable location for every player's textures — no
   per-user-count special cases — and textures shared between the player's face/boots/gloves are
   packed once for free. **Shared model folders normally keep their textures with their own ID-based
   shared-model output.** When Fox combination instead merges a shared model into a player-exclusive
   FMDL, the textures used by that merged part are copied to the player's common subfolder and the
   merged FMDL's paths are rewritten; a shared output that is still used plainly keeps its own
   texture copy. This is compiler-internal (the export format has no common concept; the models' own
   texture references are the ground truth) and generalizes Red's referee-only common preprocessing
   (`referee_tools.py`) to every player. **Textures that resolve into the export's `Common/` folder
   are the exception: they are never relocated.** Whether reached through a texture link
   (`hair.png.common`), a stem set in a Common material file, or a Common-linked model's own
   textures (merged into the player's FMDL on Fox or not), such a texture is packed once in the
   team's Common output (the Common row of the Game paths table) and the model's path is rewritten
   to point there — Red's `common/XXX/` handling with the team ID substituted, on both engines.
   Relocating them per player would defeat the reason a texture is in Common (one file for many
   players), and Common is a location the game already reads on both engines, so nothing is gained
   by moving it. The rule in one line: *resolved in the player folder → the player's; resolved in
   Common → the team's* (format side: "Link files" in the [Unified model format
   plan](model_format.md)). **Intentional deviation from Red's packed layout for teams** (verified
   extensively in-game for players as well as referees, across versions) — see the parity test
   section.
7. **Packing** — pre-Fox faces are packed into a per-model nested CPK; pre-Fox boots/gloves emit
   loose files; Fox mode packs allowed types (`.bin`, `.fmdl`, `.skl`, `.fclo`) into an FPK plus a
   template `.fpkd` (player-owned and merge-copied textures having been relocated to the per-player
   common subfolder; plain shared-output textures remain in that model's own texture location).
   Packed entries are emitted to the writer queue. (Blue: `contents_packing.py`)

### 4. Per-export non-model steps

These are logical per-export steps (Red: `export_move.py`, plus `bins_update.py` for its namesake
step; Blue: `export_move_non_model`). They are ordinary
manifest tasks and may run alongside model tasks once their dependencies are satisfied; this section
describes behavior, not a serial scheduling requirement:

- **Kits** — per kit (each `KitFolder` of the validated export; `all/` is not one): missing kit
  configs (`config.toml`) are generated from the
  template, retaining the template's five `[colors]` defaults and applying the FPC kit values when
  the team's FPC status is on; the kit's two-color `colors.txt` feeds only `UniColor.bin`, not the
  config's distinct five RGB fields. Supplied configs' FPC fields are reconciled with the team's
  status (see "FPC toggle" in the [Aesthetics export plan](aesthetics_export.md)); the kit's
  *effective* textures — own files plus the `all/` files it inherits, already merged by
  `aesthetics_export` — are renamed by replacing the `kit` prefix with the kit's
  ID — `u0{team_id}{slot}` (e.g. `kit_chest.dds` in team 701's `p2/` becomes `u0701p2_chest.dds`;
  an inherited `all/kit_back.dds` becomes `u0701p1_back.dds`, `u0701p2_back.dds`, … one per kit,
  since the game has no shared kit textures) — and converted. Each kit task reads a shared file
  itself: kit tasks stay independent, and the game-facing output is byte-for-byte what a copy in
  the kit folder would give. Two textures the game requires are **filled from bundled templates**
  when the effective set lacks them: `kit.dds` from the placeholder kit texture — a placeholder
  kit, reported once (`kit_placeholder`) — and, for pre-Fox targets only, `kit_mask.dds` from the
  mask template, silently, as Red's `kit_masks_check` did (most kits ship no mask: 70 of 641
  surveyed). **Mask and srm are engine-specific, not two names for one map**: pre-Fox reads
  `{kit}_mask`, Fox reads `{kit}_srm`, each by name convention only — the config's five texture
  fields (main, back, chest, leg, name) never name either, in any game (all 788 PES 17 and 1359
  PES 21 stock configs checked; the community converters' "rewrite `_srm` to `_mask` in the
  config" touches the `chest` slot and is a no-op). Their channels mean different things (stock
  PES 17 masks average (148,133,13), stock PES 21 srms (75,141,0): Fox packs
  specular/roughness/metallic), so **neither is ever converted into the other** — a channel
  formula would be invented, not taken from anywhere. Rule: `kit_mask` is emitted for 15–17
  targets and `kit_srm` for 18–21; the one the target's engine does not read is **not emitted**
  and reported once (`kit_texture_not_used`, I); a pre-Fox target lacking a mask gets the template
  as above; a Fox target lacking an srm gets nothing — Red never made one, the game falls back on
  its own, and 0 of 365 kits in the maintainer's library ship an `_srm` at all (14 ship a mask).
  This is what the community converters do for Fox→pre-Fox (drop the srm, write a flat
  (150,130,0) mask — Red's template is flat (151,130,0), the same default) and the mirror of it
  for the direction none of them handles. Dropping a `_mask` on Fox targets is an **intentional
  deviation from Red**, which converts it to FTEX and ships a file the game never opens. The placeholder is the
  **magenta/black checkerboard** — the Source-engine "missing texture" pattern every 4cc viewer
  recognizes, so a placeholder that does get rendered (a stray non-full-body player, the GK) is
  read as exactly what it is, not as a black kit someone chose. 64² DXT1 with a full mip chain
  like Red's mask template (~3 KB, so a full-body team's nine placeholder kits cost nothing), with
  8-pixel checks so the two-color 4×4 DXT1 blocks encode it losslessly; it passes the kit-texture
  checks by construction and is never a source for color derivation. **Layout conversion**: when
  the kit's layout marker names the other engine than the target (`pre-fox` compiled for 18–21,
  `fox` for 15–17 — see "Kit layout marker" in the [Aesthetics export plan](aesthetics_export.md)),
  the effective `kit` texture and its `kit_mask` / `kit_srm` are decoded, **re-laid out** and
  re-encoded before the rename/convert step, reported once per kit (`kit_layout_converted`, I).
  The re-layout is a fixed table of axis-aligned rectangle moves, `KIT_LAYOUT_REMAP`, one entry per
  band of the four affected islands (left/right sock, left/right shorts): source rectangle in the
  declared layout → destination rectangle in the target's; a band whose width changes is resampled
  (Lanczos3, the compiler's one resampler); every texel outside the listed source rectangles is
  copied unchanged, so the shirt, sleeves and collar strip — identical in both engines — are
  byte-identical to a no-marker compile. The Fox→pre-Fox table is the inverse of the pre-Fox→Fox
  one, so the two are one const read in either direction. **The table's numbers are not in this
  plan yet**: what is measured (from both games' uniform models, `.tmp/kit_uv_diff.py`) is the
  island outlines — socks u 8–440 → 8–372 (left; mirrored on the right), v 640–1152 unchanged;
  shorts outline u 16–644 / 1404–2032, v 1164–1948 in both — and that the mapping *inside* each
  island is piecewise (two bands per island), not one scale. The exact band edges are settled in
  the Phase 4 kits step from two sources that must agree: texel correspondence through the models'
  3D positions (u only; the Fox body's different proportions make v matching unreliable, as a
  shirt control run showed) and a hand-adjusted fixture — one 4cc kit an author shipped for both a
  16/17-era and a 21-era cup. The table is lead-authored (it is a measurement) and lives with the
  kit step (`kits/layout.rs`). Placeholder textures are engine-neutral and are never re-laid out;
  `_chest`, `_back`, `_name` and `_leg` have their own UV spaces and are not touched — whether
  `_leg`'s space also changed is unchecked and belongs to the same Phase 4 step. The TOML config is compiled to the game's 120-byte binary via `libs/kit_config`
  (texture-name fields derived from the effective texture set, version-specific bit packing applied —
  including the PES 15 shirt-pattern clamp; see the [Kit config editor plan](kit_config_editor.md))
  and emitted under the game's kit-config name for its slot (old `XXX_DEF_1st_realUni.bin` pattern
  with the team ID applied).
- **Logo** — the game's three PNGs are *produced*, not passed through: the main `logo*` file is
  decoded (`image`, via `dds_convert`'s decoders), made square per its fit tag (`crop` /
  `stretch` / `fit`, default `fit`), resampled with Lanczos3 to 512² and 256², and encoded as
  PNG; the 128² one comes from `logo_small*` the same way when present, otherwise from the main
  image. Alpha is preserved (formats without it yield opaque logos). The three sizes are the same
  in every supported version (checked against PES 21's own files, not only ≤19-era exports). The
  three are named `emblem_0{team_id}_r_ll/_r_l/_r.png` (PES ≤19) or `e_000{team_id}…` (PES 20+) —
  Red's destination names unchanged, so the CPK layout is parity-identical. A source smaller than its
  largest target is upscaled and reported (`logo_upscaled`); a non-square source made square is
  reported with the mode applied (`logo_fit_applied`). The three files are one atomic producer:
  if any input fails to decode, no logo is emitted for the team.
- **Collars** — the model files are passed through unmodified. These are custom collar models that
  replace one of PES's many stock collar models (the game's `nocloth` set); the compiler derives the
  replaced model's ID and sets it as the collar in all of the team's kit configs, which puts the
  custom model on every player at once — a quick alternative to per-player models. This automatic ID
  extraction and kit-config rewriting is an **intentional deviation from Red**, which passes collar
  files through without changing the configs. The replaced ID comes from the filename, which must be
  `collar_[ID]`; a name that doesn't parse, an ID outside the supported stock-collar range, or the
  reserved ID 105 (the FPC collar — replacing it would break FPC teams everywhere) reports
  `collar_id_invalid`. Two teams replacing the same stock collar ID is an error: planning is serial,
  so a run-wide list of the IDs claimed so far catches duplicates without any advance cross-checking
  — the later claimant in canonical export order reports `collar_id_conflict` and its collar is
  discarded. Custom collars are **compatible with team FPC**: collar rewriting runs after FPC
  reconciliation, so the custom ID deliberately overrides the FPC collar value in the configs.
- **Common** — pre-Fox: `.mtl` texture IDs and relative→absolute path fixes; both modes: texture
  conversion, dummy ID replacement, `oral_`/`_win32` model-name prefixes, face XML references to
  renamed common models updated.
- **Kit-dependent assets** — `kitN`/`kit1`…`kit9` tokens in file names and written paths (FMDL
  path tables, `.mtl` sampler paths, `face.xml` model paths) pass through **verbatim**; the modded
  exes match that spelling directly, and the historical `u0XXXp0` magic is legacy that only the
  Export upgrader understands (the compiler has no finding for it — a leftover reference just fails
  the ordinary texture-existence checks). Variant sets are completed against the
  export's kit numbers (`kit_variant_missing`, lowest variant copied) and per-kit *model* sets
  collapse to one `face.xml` entry on pre-Fox or to the lowest variant on Fox
  (`kit_variant_model_fox`). Rules and rationale: "Kit-dependent assets" in the [Unified model
  format plan](model_format.md). The legacy
  `dummy_kit*` stems keep working as **reserved, game-substituted names**: the texture-existence
  checks (`mtl_texture_not_found`, `material_texture_not_found`, FMDL path checks) skip them and the
  path is emitted verbatim. Red's `dummy_kit_replace.py` — copying the team's kit 1 textures over
  `dummy_kit*` in Common as a fallback for the substitution — is **not ported**: that fallback dates
  from when the substitution lived in Sider scripts; it is in the exes now and stable.
- **Bins accumulation** — team colors (from the root `colors.txt`, when present) and per-kit colors
  (from each kit folder's `colors.txt`, or derived from the main kit texture when it's missing —
  shirt-region dominant color plus either the shirt's second tone or the shorts-region dominant;
  `libs/color_tools` — with the menu icon number from the kit's optional `icon.txt`, default 3) are
  staged for in-memory copies of `TeamColor.bin` and `UniColor.bin` (fixed per-team byte offsets;
  Red: `bins_update.py`), and kit configs are staged for `UniformParameter.bin` compilation (Fox
  only; Red's `UniformParameter{18,19}.bin` are only its bundled per-version fallback bases); when
  the team's kit-FPC status is On, kit slots absent from the export are FPC-patched from the
  installed cup content (see "FPC toggle" in the [Aesthetics export plan](aesthetics_export.md)). A kit's UniColor/UniformParameter entry is applied only
  if that kit's task actually commits — a failed kit never mutates the global bins. Final bins are
  built after all task outcomes are known. The working bins are fetched from the user's installed
  CPKs, so recompiles update the current cup state: the download folder's `DpFileList.bin` (the
  binary list PES loads CPKs from) is walked from the bottom up — the bottom has the highest PES
  loading priority (Red approximates the same precedence by sorting the DPFL's CPK names in reverse
  alphabetical order) — and, as in Red, the walk starts just below the compiler's own output CPK,
  skipping it and anything with higher priority: this keeps each compilation idempotent even after
  newer midcup CPKs are added further down the list. Each bin is taken from the first CPK the walk
  finds it in, with the supplying CPK reported; the bundled bases only serve from-scratch compiles.
  The same walk locates pre-Fox kit-config bins for FPC patching. Bins updating is unconditional
  (Red's `bins_updating` setting is dropped — see "Resolved decisions").

### 5. Writer (single-threaded)

1. **Sideload priority** — manifest preflight gives the `sideload/` tree precedence over export
   entries at the same output path, and the writer emits accepted sideload entries before export
   content in canonical order. Sideload targets the normal team CPK(s); there is no refs-specific
   sideload tree unless one is introduced later.
2. **Incremental CPK writing** — the writer accepts completed task batches as they arrive, but
   final payload layout follows canonical manifest order, not arrival order. Per-task atomicity
   discards every staged entry from a failed task. Ordered draining and memory admission must be
   coordinated so waiting later batches cannot prevent the next batch from completing. Written
   entries release their bytes; shared/cache-owned bytes remain charged while retained.
3. **Duplicate invariant check** — all collision winners are decided deterministically by the
   manifest (or, for deep-derived names, inside the owning task) — never by rayon completion or
   writer arrival order. The writer treats any path that still arrives twice as an internal
   invariant violation; canonical manifest order and normalized CPK timestamps make otherwise
   identical builds reproducible.
4. **Bins last** — after all exports complete, the accumulated bins (committed mutations only) are
   written.
5. **Output modes and targets** — normal (CPK), test (unpacked tree per export in `test_output/`),
   sider (unpacked PES-folder structure in `sider_output/`). Test-mode directory names derive from
   the canonical source key, not from potentially duplicated display names. Multi-CPK mode (the cup
   DLC mode) routes team content into **size-split `teams` parts** plus the bins CPK — see
   "Multi-CPK mode: teams parts" below. Referee content always routes to its own CPK named by
   `refs_cpk_name`, separate from those team targets; `refs_cpk_name` affects only normal CPK mode
   (test/sider retain their existing per-export layouts). Every emitted CPK name (team, teams part,
   refs) is a validated `CpkStem` (shared `pipeline` type): filename stem only, 1–28 characters from
   ASCII alphanumeric, `_`, `-`, and `.`; no separators, control characters, trailing dot, Windows
   reserved-device names, or user-supplied `.cpk` suffix; uniqueness is checked case-insensitively.
   Deployment/DpFileList/locked-file/marker-summary logic runs per generated CPK, including the refs
   CPK and every teams part.

6. **Multi-CPK mode: teams parts.** Red's multi-CPK split by *content type* (`4cc_40_faces` /
   `4cc_45_uniform` / `4cc_08_bins`) is replaced by a split by *size*: all team content — faces,
   portraits, boots, gloves, common, kits, kit configs, logos — goes into a sequence of **`teams`
   CPKs**, plus the separate bins CPK. The reason is a hard external limit: the cup DLC is versioned
   in Git, and Git for Windows cannot handle objects of 4 GiB or more (`unsigned long` is 32-bit
   under MinGW; the `size_t` conversion is still ongoing upstream in 2026). A **32-team**
   `4cc_40_faces.cpk` already sits at 3.77 GB, so a 48-team cup lands near 5.7 GB — well past the
   limit — while the uniform CPK was barely used. The faces/uniform distinction stops paying for
   itself, and one uniformly filled sequence of parts replaces it.

   - **The slots come from the DpFileList, midcup style.** The official DPFL reserves a run of
     numbered names with the same stem — `4cc_40_teams`, `4cc_41_teams`, … — exactly like the
     existing `4cc_60_midcup` … `4cc_79_midcup` run (and `4cc_30_stadiums0`…). The compiler takes as
     its part slots every DPFL entry matching `{prefix}_{NN}_{teams_cpk_stem}` (the `teams_cpk_name`
     setting names the stem, default `teams`), ordered by number. The stem match is **exact** — the
     text after `_NN_` must equal the stem, so `teams` never claims `teams2` slots — which is what
     makes a **second slot run usable for a side event**: with `teams_cpk_name = teams2` and the
     DPFL reserving `4cc_50_teams2` … in place of the old `4cc_50_other_faces`/`4cc_55_other_uniform`,
     the same compile fills that run instead, sharing only the bins CPK (there was never an
     `other_bins`). There is no parts-count setting:
     capacity is a cup-level decision made once in the artifact every user already receives, and
     the compiler can never emit a part the users' game will not load. How many slots and which
     numbers is the DPFL author's call; the planned layout is **five slots**, which at the default
     cap gives 15 GB — roughly 2.5× a 48-team cup.
   - **Filling is deterministic first-fit by canonical team order, one team per decision, made by
     the writer on exact sizes.** Teams are never split across parts: the writer holds a team's task
     batches until the team is complete, then places the whole team — into the current part if it
     fits under `cpk_part_max_size` (TOC included), otherwise it closes the part, opens the next slot,
     and places it there. This costs nothing measurable: the writer already lays out entries in
     canonical order, so a team's batches were waiting for their earlier siblings anyway; holding
     them until the team's *last* batch lands adds a few seconds of retention for one team's compiled
     output (on the order of 100–250 MB, charged to the memory budget like any pending batch) while
     the workers keep processing the next teams. Writing is disk-bound and writes the same bytes
     either way. No planning-time estimate, no sidecar state, and no hard-limit check is needed
     because the decision is made on bytes actually produced. A single team larger than the cap is
     impossible in practice at 3 GB and is a fatal `cpk_team_exceeds_cap` rather than a silent split.
     Whether teams *could* straddle parts was considered (PES merges every CPK into one virtual
     filesystem, so the game would not care) and rejected for tidiness: a team living in one CPK is
     what maintainers expect when they open one. Minimizing Git churn is not a goal here: a cup DLC
     is compiled **once**; midcup changes go to the designated midcup CPKs (one per day, compiled in
     single-CPK mode with `cpk_name` set to that day's slot), so parts are never rewritten and
     cascading cannot occur.
   - **Every slot is always written.** Slots the run does not fill receive the **empty placeholder
     CPK** — the same 6,272-byte zero-entry CPK the official DLC already ships for unused
     `midcup`/`test`/`stadiums` slots (`4cc_68_midcup.cpk` is one). This satisfies the DPFL without
     relying on PES tolerating listed-but-missing files, and it makes superseded content trivial: a
     recompile that needs fewer parts overwrites the stale higher slots with placeholders. The
     placeholder's bytes are emitted by the `cpk` crate and verified byte-identical to the shipped one
     in tests. Running out of slots (content exceeds `slots × cap`) is fatal: `cpk_slots_exhausted`
     names the shortfall so the DPFL author can add slots.
   - **`cpk_part_max_size`** defaults to 3 GB — comfortably under Git for Windows' 4 GiB object
     ceiling, and configurable. It applies to teams parts only; a single-CPK run that
     exceeds it (a midcup day gone wild) is not split — single-CPK names have no slot run — but gets
     `cpk_size_over_limit` (W), since the CPK is fine for PES and only the repository will object.
   - **No legacy DPFL support in multi-CPK mode.** Multi-CPK is run only by the cup maintainers
     assembling a whole DLC from the managers' exports; they are always on the latest tooling. The
     compiler ships the current official per-version DPFL as an embedded template (see "Resolved
     decisions") and, when the installed one lacks the required slots, refuses the run with
     `dpfilelist_outdated` and offers the **DpFileList upgrade** described under "Post-processing".
   `test_output/`, `sider_output/`, and `teamnotes.txt` are resolved beneath `output_folder_path`; a
   CLI positional exports-root overrides `exports_folder_path` for that invocation only.

   **Test mode is Red's step 1.** Red's `1_exports_to_extracted.bat` stopped after pre-processing,
   leaving the checked and edited export in `extracted/` with its original item-folder layout, before
   the files were rearranged into PES paths and packed — the one staged run that stayed useful during
   development, because it shows what the compiler *did* to an export before that is obscured by
   relocation and packing. Blue kept it as `--mode test` (processed model folders emitted under
   `{export}/{itemfolder}/{model}/`, no FPK packing, no game paths). The Studio keeps it under the
   same name for the same reason; a regression suite does not replace it. The suite says *that* final
   output differs from a reference; test mode is for the moments without a reference — developing a
   new pipeline step (glTF conversion, a new version's game paths) or a cup maintainer asking why a
   texture was renamed. Sider mode is the complementary view: the same content *after* relocation,
   unpacked — what the CPK would contain. In Red's terms it is **steps 1+2** (`extracted/` →
   `patches_contents/`, the PES folder structure that step 3 packed), and its purpose is
   prototyping: Sider's `livecpk` mechanism sideloads loose files from that structure without a CPK,
   without touching the PES install, and without restarting the game between iterations, so a
   modeler can compile → alt-tab → see the change. Output goes to `sider_output_path` (default
   `output/sider_output/`), which the user either points at Sider's livecpk root or lists in
   `sider.ini`'s `cpk.root` (see the settings table).

   The cost Blue paid — a second code path in the coordinator — is contained by deciding the
   **output target once, at manifest planning**, and consuming it at exactly one seam: each task's
   final *materialize* step (relocation to game paths, Fox FPK packing). In test mode that step is
   skipped and the task emits its processed entries with export-relative paths; in normal and sider
   mode it runs. Everything upstream (checks, model conversion, texture work, name editing) and
   everything downstream (the writer, memory permits, events) is identical. Bins and `teamnotes.txt`
   are emitted in every mode. The loose-folder sink is shared between test and sider modes and is also
   the harness the unit and parity tests drive the pipeline into — trees are diffed directly, with no
   CPK parsing in the middle — so the sink is test infrastructure that the GUI happens to expose,
   not a feature carried for one dropdown entry.

### 6. Post-processing

- **Staging** — every generated CPK is written to a run-scoped staging folder,
  `output/.staging/{run_id}/`, beneath the `output/` folder next to the exe (created on demand). No
  CPK is ever written directly at its final path, so a failed or cancelled run leaves nothing
  half-written where PES or the user could pick it up.
- **Deploy CPKs** — deployment is **always attempted**; there is no `move_cpks` switch (see "Why no
  `move_cpks`" below). Each staged CPK is copied to `{pes_folder_path}/download/{name}.cpk.partial`
  (the download folder is normally on another volume, so this is a copy, not a rename) and then
  renamed over the old CPK; the rename is the only step that needs the old CPK unlocked, and it is
  atomic on the same volume. A marker file lists what was deployed. After the deployment transaction
  commits, the staging folder is deleted — like Red, nothing is left in `output/`.
- **Degraded run: promotion to `output/`** — if deployment cannot happen (`pes_folder_not_found`,
  `old_cpk_locked`, `deploy_target_unwritable`, `dpfilelist_missing`, ...), the staged CPKs are
  **promoted** to `output/{name}.cpk` (same volume, atomic rename, replacing any previous one)
  instead of discarded; the message says so, the savefile step is skipped (see below), and the GUI
  offers **Retry** (for the locked case — the user closes PES and retries the deployment alone, no
  recompile) and **Open output folder** (a status-bar notice action after the run, and a permanent entry in the
  Compile dropdown). The CLI reports the error and the promoted path. `output/` is thus Red's
  `patches_output/` in its new place, reached by degradation or by `--no-deploy` rather than by a
  setting. A machine without PES installed compiles this way automatically: the CPK lands in
  `output/`, the bins are built from the bundled fallback bases, and the run says why.
- **`--no-deploy` (CLI only)** — skips the deployment *and* the savefile step and promotes the
  staged CPKs to `output/` as a clean run, no error. This is the production flag for cup
  maintainers building DLC on a machine whose install must not be disturbed, and for build boxes; it
  is a flag, not a persisted setting, because the GUI's job is installing into the user's game.
- **Why no `move_cpks`.** Red's setting existed because Red's bins came from its bundled
  `Engines/bins/`, so a compile was meaningful without a PES install. The Studio's bins accumulation
  reads the installed CPKs, the savefile step reads the install's save, and the version selector
  checks the install's exe — the pipeline is built around a present install, and a "don't install"
  switch on top of it was carrying real weight: dual-severity messages (`cpk_name_unlisted` W/E),
  conditional checks (`pes_version_mismatch`, `dpfilelist_missing`), a marker message
  (`ref_marker_requires_move`), `run_pes` depending on it, two preflight target sets, and — worst —
  a coherence hole, since the savefile was still written while the CPK it referred to was not
  installed. The promotion path already had to exist for failed deployments, so the setting's only
  unique behavior was reachable at zero cost by degradation; the flag covers the one deliberate case.
- **DpFileList upgrade** — the compiler never edits `DpFileList.bin` as a side effect of a compile
  (the Balls compiler's "check, never write" rule holds for every tool). It does offer one explicit,
  consent-based action. The current official per-version DPFL ships as an embedded template
  (`templates/` override applies, so maintainers can hot-swap it between releases). At deployment
  preflight the installed DPFL's entry set is compared with the bundled one:
  - identical, or a superset that still contains every target of this run → nothing to do;
  - missing any target of this run (an old official DPFL — typically the pre-`teams` layout) →
    `dpfilelist_outdated` (E; the run degrades like any deployment failure) and the GUI shows
    **Upgrade DpFileList**; the CLI prints the equivalent subcommand,
    `studio team-compiler upgrade-dpfl`, and never upgrades on its own.
  - The upgrade is an **override, not a merge**: the bundled official DPFL replaces the installed
    file byte for byte, the old one kept as `DpFileList.bin.bak`. The aesthetics community gives
    zero support for custom-edited DPFLs — they have caused a long tail of problems — so preserving
    a user's own entries would be preserving exactly the state the upgrade exists to end. The dialog
    is honest about it: it lists every installed entry that the official list does not contain
    (user additions *and* retired official names such as `4cc_40_faces`, `4cc_45_uniform`) as
    "will no longer be loaded", and for each whose `.cpk` still sits in `download/` shows the size
    and offers deletion (renaming a 3.7 GB file as a backup is pointless; the user confirms per file,
    and nothing is deleted without that confirmation). No retired-names list is needed in the
    template: retired is simply "installed but not official".
  - The DPFL binary format is small and version-stable enough to own here: a 16-byte header (`u32 0`,
    `u32 entry_count`, 8 zero bytes) followed by fixed-width name records — verify the record width
    per PES version against installed files before implementing; the reader already exists for the
    bins walk, the upgrade adds the writer.
- **Destination writability preflight** — the two failure modes users actually hit (the download
  folder needs elevation because PES sits under `Program Files`; the old CPK is locked because PES
  is still running) are detectable long before Compile, so they are checked **live**, like the
  version selector's exe check, not discovered at the end of a multi-minute run. For every planned
  destination (the single CPK, or every teams slot plus the bins CPK in multi-CPK mode, plus the refs CPK, under
  `download/`): if the file exists, open it for
  writing without truncating; if not, create and remove a probe file in its folder. The failure
  kind is read from the OS error — `ERROR_ACCESS_DENIED` → elevation needed, `ERROR_SHARING_VIOLATION`
  → in use (cross-checked against the core plan's running-PES poll, which names the culprit). The
  check runs at startup, on window focus, whenever a relevant setting changes (`pes_folder_path`,
  `pes_version`, CPK names, `multicpk_mode`), and whenever the PES process poll changes
  state. Results show on the Compile button as a warning badge with the reason in the tooltip and a
  line in the run strip: *"PES 2019 is running — close it before compiling"* / *"The download
  folder needs administrator rights"* with a **Relaunch as administrator** action (the `elevation`
  lib). The button stays enabled — the user may close PES during the compile — except for the
  elevation case, where compiling first would waste the run (relaunching loses it), so the click
  prompts for elevation up front. The deployment stage re-checks regardless; the preflight makes
  its failures rare, not impossible.
- **Aesthetics patch** — written on **every** compile, beside the output CPK, as
  `aesthetics_patch.toml`: the resolved savefile writes for every compiled player (format and rules
  in "Aesthetics patch" in the [Savefile plan](pes_savefile.md)) — settings.toml settings with
  `name = true` and FPC markers resolved to concrete values, and the auto-assigned boots/gloves IDs
  **only for content that was actually packed**: if a boots/gloves task failed, the affected players
  keep their existing savefile IDs rather than pointing at absent CPK content, while their
  independent valid settings still apply. The patch travels with the CPK: it describes the CPK it
  sits beside, whether or not this machine deployed it. This is what separates the **DLC builder**
  from the **savefile builder**: the former compiles every export and hands over CPK + patch, the
  latter applies the patch in the save editor to the official save, which the former never touches
  (it holds the teams' custom tactics, which only the savefile builder receives). Reported as
  `patch_written` (I) with the path.
- **Savefile update** — the local savefile, when one is configured, is updated by **applying the
  patch just written** through `pes_savefile`'s `apply_patch` — the compiler has no second write
  path, so the local result and the savefile builder's later result are the same bytes by
  construction. Saving uses a `.bak` backup, the same convention as the save editor's. A missing
  savefile stays a warning (`savefile_missing`, naming the patch): even the DLC builder is expected
  to compile against a template save, to test the DLC in PES. **The savefile is never written while
  PES is running**, in any output mode: the game holds its edit data in memory and writes it back on
  exit, so a write made now would not go live without a restart and would be overwritten by whatever
  the player changed in-game before closing. The step is skipped with `savefile_skipped_pes_running`
  (W) telling the user the settings will be applied by the next compile made with PES closed; the
  rest of the run is unaffected. This matters mostly for **Sider mode**, whose whole point is
  compiling while PES stays open — model iterations land immediately via livecpk, and the savefile
  half of the export (boots/gloves IDs, player settings) catches up on the first compile after PES
  is closed. In normal mode a running PES already fails deployment on the locked CPK, so the rule
  adds nothing there. The check uses the same running-PES detection as the core plan's
  version-selector poll; the CLI performs it once, at the savefile stage. The savefile is likewise
  **skipped whenever the CPKs were not deployed** — a degraded run promoted to `output/`, or
  `--no-deploy` — because a savefile pointing at boots/gloves IDs whose content is not installed is
  exactly the incoherence the "only for content actually written" rule exists to prevent.
- **Run PES** — optional launch of `PES20{version}.exe`, only after the complete deployment
  transaction succeeds.

### Game paths reference

| Content | Pre-Fox (PES 15–17) | Fox (PES 18+) |
|---|---|---|
| Faces | `common/character0/model/character/face/real/{id}.cpk` | `Asset/model/character/face/real/{id}/#Win/` |
| Boots | `common/character0/model/character/boots/{id}/` | `Asset/model/character/boots/{id}/#Win/` |
| Gloves | `common/character0/model/character/glove/{id}/` | `Asset/model/character/glove/{id}/#Win/` |
| Kit configs | `common/character0/model/character/uniform/team/{team_id}/` | same |
| Kit textures | `common/character0/model/character/uniform/texture/` | `Asset/model/character/uniform/texture/#windx11/` |
| Collars | `common/character0/model/character/uniform/nocloth/` | `Asset/model/character/uniform/nocloth/#Win/` |
| Common | `common/character1/model/character/uniform/common/{team_id}/` | `Asset/model/character/common/{team_id}/` |
| Portraits | `common/render/symbol/player/` | same |
| Logos | `common/render/symbol/flag/` | same |
| TeamColor.bin | `common/etc/TeamColor.bin` | same |
| UniColor.bin | `common/character0/model/character/uniform/team/UniColor.bin` | same |
| UniformParameter | — | `common/character0/model/character/uniform/team/UniformParameter.bin` |

### Resolved decisions and open questions

Resolved decisions:

- The team info file is dismissed in favor of root `colors.txt`/`notes.txt`; the canonical
  first-word `team_name` (not the complete `export_display_name`) is looked up in `teams_list.txt`.
- `ExportIdentity` keeps normal 701–920 `TeamId` values separate from referees; validation preserves
  all issues while producing a sanitized eligible export; normal-team numbering comes from folder
  names or root `players.txt`, while referee exports require the unified player-folder format's root
  `players.txt` slot mapping.
- Player-owned textures relocate to per-player common while plain ID-based shared outputs retain
  their own textures; a multi-mapped folder (one source folder listed under several `players.txt`
  slots — rarity-repeated referees, or one model backing several team players) is instantiated per
  slot but emits its textures **once**, into a single common subfolder keyed by the source folder's
  name that every mapped slot's model references (Red's in-game-verified referee layout — no texture
  duplication across slots).
- Cross-export duplicate precedence is resolved by run-level manifest preflight; multiple normal
  exports resolving to the same team ID are rejected with `duplicate_aesthetics_export`.
- **ID allocation**: 40-ID per-team blocks (23 player-exclusive + 17 shared), applied independently
  in the disjoint boots and gloves namespaces; `allocation_scheme_version = 1` recorded in build
  metadata. Permanent per scheme version but upgradeable: a future scheme (e.g. 5-digit IDs matching
  player IDs after an exe patch) bumps the version and ships with a from-scratch savefile remake
  (see "Assigns IDs automatically").
- **Root normalization**: exactly one usable nested root is flattened; multiple usable roots
  (`nested_root_ambiguous`) and loose-root/nested collisions (`nested_root_conflict`) reject the
  export — an intentional deviation from Red's first-match rule.
- **Model source selection**: target-native first, then glTF, then conversion from the opposite
  native format; in-representation duplicates are rejected (`model_source_ambiguous`).
- **Kit colors fallback**, in order: the kit's `colors.txt`; when missing or yielding fewer than
  two valid colors, derivation from the kit's own main texture (`kit_colors_derived`) — never from
  the placeholder checkerboard; when that is impossible too (no decodable own texture, or a
  placeholder kit), the fixed **magenta/black "no colors chosen" pair is written**
  (`kit_colors_missing`, W). Loud on purpose: the menu dots and scoreboard strip then show a pair
  nobody would pick, so the omission is seen in the first menu instead of being papered over. The
  alternatives were both quiet failures — skipping the entry leaves whatever a previous cup's bin
  held, and a stand-in such as the team's root colors looks right while being unchosen. The
  pair matches the placeholder texture, so a fully placeholder kit is consistently "unfinished"
  everywhere it appears.
- **FPC markers are player-level presets** (`fpc.on` = hide, `fpc.off` = un-hide, for that player's
  savefile settings only); non-FPC players are valid on FPC teams, so mixed-marker teams are
  ordinary supported usage. Team **kit**-FPC status is two-state (`On`/`Unknown`): any `fpc.on`
  writes the FPC kit values into every config; otherwise supplied configs are left untouched — the
  compiler never auto-reverts FPC values. Kit slots absent from the export are FPC-patched from the
  installed cup content (`kit_config_fpc_adjusted`), so a midcup export can add an FPC player
  without resending untouched kits (see "FPC toggle" in the [Aesthetics export plan](aesthetics_export.md)).
- **The aesthetics patch is the compiler's only savefile write path** (user): every compile writes
  `aesthetics_patch.toml` beside the CPK; a configured local savefile is updated by applying that
  patch, never by a separate write. Reason: the DLC builder and the savefile builder are different
  roles — the official save carries the teams' custom tactics, which the DLC builder must not have —
  so the compile's savefile half has to be a file that can be carried to another machine and applied
  there; making the local write use the same file removes the possibility of the two drifting. A
  missing local savefile remains a warning, not a supported quiet mode, because the DLC builder
  still compiles against a template save to test in PES.
- **Bins updating is unconditional** — Red's `bins_updating` setting is dropped: it only existed
  before "fetch the latest installed bins and update those" was a thing, and skipping bins updates
  is never useful now that recompiles patch the current cup state.
- **Working-bin lookup follows `DpFileList.bin`**: the download folder's DpFileList is walked from
  the bottom up (the bottom has the highest PES loading priority; Red approximates this by sorting
  the DPFL's CPK names in reverse alphabetical order), starting just below the compiler's own output
  CPK — skipping it and anything with higher priority keeps recompiles idempotent even after newer
  midcup CPKs are added — and each bin comes from the first CPK the walk finds it in, with the
  supplying CPK reported; the bundled bases only serve from-scratch compiles (see "Bins
  accumulation").
- **Templates and fallback bins are embedded in the exe, with an override directory**: Red's loose
  `Engines/templates/` and `Engines/bins/` files (refscpk trees, the per-version player skeletons
  from `resources/skeletons/` — Red's single `body.skl` generalized, see the Model conversion plan's
  "Skeleton data" — `face_diff.bin`, the template environment cubemap that the material schema's
  `metal` family falls back to on pre-Fox (see the Unified model format plan's "Textures"),
  `kit_mask.dds` and the checkerboard `kit.dds` for placeholder kits, dummy model/MTL, `generic.fpkd`,
  `fcl_hair_sim.fclo`, the generic kit config, the
  `TeamColor`/`UniColor`/`UniformParameter` fallback bases, and the current official per-version
  `DpFileList.bin` (for slot discovery and the DpFileList upgrade — see "Multi-CPK mode: teams
  parts" and "Post-processing") — ~6 MB total) ship inside the binary
  (`include_dir!`), version-locked to the compiler logic that consumes them; Red's
  `file_critical_check` class of missing/mismatched-template errors disappears. A `templates/`
  folder in the data directory shadows embedded resources per file, so cup maintainers can hot-swap
  fallback bins or referee template content between releases; each active override is reported
  (`template_override_active`), and an override file that exists but cannot be read reports
  `template_override_unreadable`.
- **Teams list: embedded upstream, one working copy in the data directory.** The current cup's
  `teams_list.txt` ships *inside* the binary like the templates (upstream), and the only copy on
  disk is `data/teams_list.txt` (working), created from the embedded one on first run and edited
  by the grid's ID cell and by the updater's merge (see the core plan's "Self-update"). We do this,
  not Red's loose file beside the exe, because the grid writes into the list, which makes it user
  data: it must follow the settings file into the user config directory when that mode is chosen,
  or a replaced program folder loses the user's ID assignments; and a single on-disk file leaves
  nothing for a user to edit by mistake. The file keeps Red's name and `.txt` extension: it is the
  file cup organizers already pass around, and `.txt` opens in a text editor on double-click where
  `.tsv` opens nothing, or a spreadsheet that rewrites cells. **Writes are best-effort, never
  elevated**: when the data directory is not writable (the typical case is a portable install
  unpacked under Program Files), the ID cell stays read-only with the reason in its tooltip and
  the updater's merge is reported as skipped (`teams_list_read_only`) — Studio is a portable app,
  and elevating for a one-line edit is handling a corner case nobody should be in. Detection is by
  attempting the write, not by inspecting the path: there is no list of protected directories to
  keep right. **The savefile is a second source for the same merge.** In-game team names in a 4cc
  save are the `/xx/` names, so the savefile's `TeamEntry { id, name }` rows for IDs 701–920 are
  a teams list — and the authoritative one for the cup the user is actually playing, ahead of
  whatever the last release embedded. The [Save editor](save_editor.md)'s **Export teams list**
  action (it owns the open savefile; the compiler only consumes the list) runs the *same*
  reconciliation as the updater with the savefile as the incoming list (added / kept / overridden,
  reviewed before writing; savefile value wins a conflict), never a blind overwrite. It is
  on-demand, not automatic: the list must keep working with no savefile present
  (`savefile_missing` is a supported mode), and rewriting user data as a side effect of a compile
  would be a surprise. Placeholder rows (`Backup N`, `Invitational N`) come along and are inert.
- **Collar contract**: the replaced stock collar ID comes from the `collar_[ID]` filename
  (`collar_id_invalid` otherwise); ID 105 is reserved for FPC and cannot be replaced; two teams
  claiming the same ID is an error caught by a run-wide claimed-ID list during serial planning
  (`collar_id_conflict`, later claimant in canonical order loses). Custom collars are compatible
  with team FPC — collar rewriting runs after FPC reconciliation and deliberately overrides the FPC
  collar value (see "Collars").
- **Missing savefile stays a warning**: deterministic ID allocation keeps a later savefile-inclusive
  recompile of the same exports consistent, so the CPK step may proceed.
- **Pre-Fox local boots/gloves never touch the savefile**: models embedded as typed face-XML entries
  are ordinary face-XML models with no ID concept, so a local-only category leaves the existing
  savefile boots/gloves ID untouched. Only shared pre-Fox boots/gloves folders (ID-named outputs)
  get savefile ID writes; a player combining local parts with a shared link gets the shared ID
  written while the local parts ride in the face XML. **Exception — `ingame_face`**: with no face
  folder emitted, gloves and boots parts are relocated to player-specific folders with IDs from the
  per-team block scheme, and those IDs are written to the savefile on pre-Fox too; a shared link
  combined with local parts of the same category is merged into that player-exclusive folder (the
  one pre-Fox `link_combined` case), so the player never has two candidate IDs for one savefile slot
  (see "ingame_face marker" in the [Aesthetics export plan](aesthetics_export.md)).
- **Check-cache identity**: recursive manifest fingerprint combined with teams-list and
  validation-settings revisions; the watcher only triggers re-fingerprinting (see "Live
  validation").
- **Shallow archive checks never decompress solid data**: `.zip` per-entry access may read small
  metadata; solid-`.7z` content checks defer to the compile-start deep check, with the roster
  recorded as unread.
- **Virtual-path safety** is specified on the shared `vtree::ScopePath`/`RelativeScopePath` types
  (see the [library crates plan](libs.md)): traversal/absolute paths rejected, separators
  normalized, case/Unicode collisions detected before extraction or packing; `aesthetics_export` uses the
  shared type.
- **Source snapshot**: export revisions are pinned at planning and verified around each folder
  materialization; a change aborts the run via `source_changed_during_run`, and archive handles stay
  bound to the opened archive revision.
- **Merge-copy collisions within one child** use the dedicated `merged_texture_conflict`: when two
  parts merged into the same output FMDL (Fox baking of local, shared, and `.common`-linked models
  into one allowed name) copy the same texture destination with different bytes, no canonical winner
  exists — the parts are equal-standing inputs of one model, so the child drops rather than silently
  corrupting one part's look. Group-wide collisions across a player's face/boots/gloves tasks keep
  the `shared_texture_conflict` rule, which does have a canonical winner (face > boots > gloves).
- **Texture references are stem-based (input)**: all material references (FMDL path tables,
  `.mtl`, `.materials.toml`) name textures by stem (filename without extension) in their
  source/authoring form. Any image format (DDS, FTEX, PNG, JPEG, BMP, WebP, TGA, TIFF) may be
  used as a source for any model format — they are fully interchangeable. The compiler resolves
  each stem to the actual file in the folder and converts to the target format. Two image files
  with the same stem but different extensions is `texture_stem_conflict` (checked in the
  structure pass). FMDL path tables were already stem-based; Studio-format `.mtl` and
  `.materials.toml` write stems; the Export upgrader strips extensions from old `.mtl` texture
  paths when migrating. **Output** `.mtl` files (written at compile time) keep `.dds` extensions
  on texture paths, as the game expects; FMDL output path tables remain stem-based. Omitted or
  blank texture roles are auto-named from the material name + role suffix (fixed table; see
  "Textures" in the [Unified model format plan](model_format.md)); this happens during deep
  parsing, not the structure pass. A stem resolves in the folder of the material file that set it,
  so material files linked from Common (`*.materials.toml.common`, `*.mtl.common`) name Common's
  textures while local overrides name the player folder's; a texture link (`hair.png.common`)
  makes the Common file resolve under that stem in the player folder. Where a texture resolved
  decides where it is packed (step 6): player folder → relocated to the player's common subfolder;
  Common → referenced in place, once per team.
- **Conversion cache engages only for ≤2 teams**: the in-memory texture conversion cache is
  bypassed for compiles of 3+ teams; see the [library crates plan](libs.md) for rationale.

Open questions — **implementation/spec work** (no user preference involved; resolved during the
phase that owns them):

- **Shared/Common dependency graph.** Define dependency ordering and cache ownership when a player
  task needs shared or Common models for a Fox merge while those folders also have independent
  output work.
- **Task-batch commit protocol.** The required guarantee is fixed: incremental CPK writing must
  never expose entries from a failed atomic task. Define the exact stage/commit boundaries between
  workers, the per-player shared-texture aggregation, and the writer.
- **Cancellation and fatal-output cleanup.** Partial CPKs must be discarded and prior
  published/installed outputs must remain untouched on failure or cancellation, as required by the
  runtime catalog. The mechanism is fixed in "Post-processing" (run-scoped `output/.staging/`,
  `.partial` copy + atomic rename in `download/`, promotion to `output/` on deployment failure);
  what remains is the backup/rollback protocol across multiple CPKs, the savefile, and
  `dt00_x64.cpk`, and cleanup of stale `.staging/` folders left by a crash.
- **Complete memory accounting.** Source sizes do not cover decoded textures, converted models,
  merged meshes, or packed entries; evaluate an RAII budget permit that grows and shrinks with
  actual allocations. Solid 7z archives charge their full decompressed buffer until all dependent
  tasks drain.
- **`teams_list.txt` contract.** Fixed already: tab-separated (not whitespace — Blue's
  `split()` turned `Backup 1` into `Backup`), `ID` and `Name` columns with further columns ignored,
  Name lowercased on load, CRLF and no BOM as Red writes it (Red's current file: 220 rows, all
  ASCII). Still to specify: whether the header row is required or detected, BOM tolerance on read,
  blank lines, duplicate names/IDs, atomic writes, and concurrency between the grid's ID-cell
  write, the updater merge and the savefile import.
- **`colors.txt` grammar and TeamColor capacity.** Define the grammar (UTF-8/BOM, decimal versus
  hex, comments, blank lines) and the exact team-color count `TeamColor.bin`'s fixed-size records
  support; the kit-side fallback policy is resolved above.
- **Run-result semantics.** Define separate compile and deployment outcome types so scoped errors,
  `pass_through`, failed moves, and failed savefile writes map predictably to GUI state and CLI exit
  codes. Deployment is transactional across generated CPKs, the savefile, and `dt00_x64.cpk`, with
  backups and rollback; PES launches only after the complete transaction succeeds.
- **`dt00_x64.cpk` transaction.** Define backup and rollback behavior if referee-marker conversion
  or system-CPK replacement fails or is cancelled.
- **Superseded installed outputs.** Within the teams slot run this is solved: every slot is written
  each multi-CPK run (content or the empty placeholder), so stale parts cannot survive. What remains
  is the single-CPK-versus-multi-CPK crossover (a `4cc_90_test` left from a test compile is not
  touched by a DLC compile, and vice versa — decide whether that needs a message) and the rule that
  an unrecognized user file is never deleted; retired official names are handled by the DpFileList
  upgrade's confirmed per-file deletion.
- **Output-mode artifact routing.** Define routing for every artifact — bins, sideload, referee
  content, and `teamnotes.txt` — across normal single-CPK, multi-CPK, refs, test, and sider modes.
- **Per-player common path templates.** Add Red's referee common layout (name-keyed per-referee
  subfolders; exact MTL/XML and Fox FMDL templates in "Texture relocation to common") to the Game
  paths table as the per-player common subfolder patterns (Phase 4 specification).

**Pre-Phase-3 gates:** the normal-team versus referee `ExportIdentity` boundary, sanitized
validated-versus-eligible projection, and roster-entry scope/disposition semantics must be confirmed
before Phase 3 implementation begins.

**Pre-Phase-4 gates:** finalize analyzed-output enumeration and namespace allocation (including
folder-internal deep-derived names, without adding a separate `AnalyzedRun` type) and freeze every
model task's planned model-ID assignments before processing implementation begins.

---

## Message catalog

The warning/error messaging is overhauled, not ported (see "Relationship to the older compilers").
Red's ~150 messages (audited across `export_check.py`, `texture_check.py`, `xml_check.py`,
`team_id_get.py`, `portraits_move.py`, `bins_update.py`, `referee_tools.py`, `cpk_tools.py` and the
stage scripts) reduce to a structured catalog: every reportable condition gets a stable ID, and the
pipeline emits it as **data** — the GUI, CLI, and log files decide presentation.

### Message structure

```rust
use std::borrow::Cow;

pub struct MessageCode {
    pub tool_id: &'static str,             // "team-compiler"
    pub code: Cow<'static, str>,            // "player_number_duplicate"
}

pub struct Message {
    pub code: MessageCode,                  // cross-tool stable identity
    pub severity: Severity,                 // presentation and filtering
    pub disposition: Disposition,           // processing consequence, independent of severity
    pub scope: Scope,                       // what the message is about — drives grid cell mapping
    pub context: Vec<(String, String)>,     // structured fields: file name, expected/found values, ...
}

pub enum Severity {
    Fatal,
    Error,
    Warning,
    Info,
}
// Ordering: Info < Warning < Error < Fatal. "Errors" filter includes Fatal.
// Cell status uses the highest relevant severity after disposition policy.

pub enum Disposition {
    Keep,
    DropFile,
    DropSlot,                             // discard one normalized players.txt assignment
    DropFolder,
    DropExport,
    AbortRun,
}

pub enum Scope {
    Run,                                  // not tied to an export (output stage, settings)
    Export { export_id: ExportId },
    Folder { export_id: ExportId, path: vtree::ScopePath }, // player/shared/kit folder
    File   { export_id: ExportId, path: vtree::ScopePath },
    RosterEntry { export_id: ExportId, file: vtree::ScopePath, line: usize, slot: Option<u16> },
}
```

`ExportId` and canonical `vtree::ScopePath` values route events and logs; human-readable
export/folder/file names stay in message context. `ScopePath` is the platform-neutral canonical path
type: `aesthetics_export` uses/wraps it but does not own it, keeping `studio_core` independent of
team-format knowledge. Stable IDs avoid ambiguity when a folder export and an archive have the same
display stem.

The `MessageCode`/`Message`/`Severity`/`Disposition`/`Scope` types live in `studio_core` alongside
`PipelineEvent` (all tools emit them). Each tool owns its code strings and text catalog under its
stable tool ID; the catalog below belongs to `team-compiler`, while the stadium compiler and export
upgrader define their own codes on the same structure.

Principles:

- **One condition, one ID.** Red repeats near-identical messages per folder type ("Bad face folders"
  / "Bad boots folders" / "Bad gloves folders"); here the folder identity is in the `scope`, so one
  `fmdl_name_invalid` ID covers all model folder kinds.
- **Message text lives in one table** keyed by ID (template + remediation hint), not scattered
  through the pipeline. Red's remediation-hint style is kept ("Resize it so that both sizes are
  powers of 2").
- **Consequences are explicit**: `Disposition` records whether the finding keeps content, drops a
  file, normalized roster slot, folder, or export, or aborts the run; severity remains a
  presentation/filtering concern. File-scoped consequences still roll up according to Red's rules
  (model-file failures drop the folder; standalone texture/portrait failures may drop only the
  file). A line-local `RosterEntry` finding with `DropSlot` removes only that normalized assignment;
  file-wide encoding failure or roster ambiguity that prevents trustworthy normalization uses
  `DropExport` instead. Runtime failures derive their effective disposition from the owning scope:
  optional root file → `DropFile`, folder/task → `DropFolder`, required export metadata or unusable
  source → `DropExport`, output-writer/global invariant → `AbortRun`.
- **`pass_through`** overrides only eligible content-level `DropFile`/`DropFolder` dispositions to
  keep-with-flag, as in Red, while retaining the original severity. Not eligible: findings whose
  content cannot exist (failed conversion/packing, unproducible logos), unused content
  (`shared_folder_orphaned`), ambiguous model-source or texture identity/conflicts, foundational
  unsafe-path/ambiguous-roster/required-metadata failures, `DropSlot`, `DropExport`, `AbortRun`, and
  environment-level dispositions. Written errored content
  receives a distinct `DoneWithErrors` outcome rather than an ordinary red error state.
- **No blocking console prompts.** Red's prompts are replaced, not kept: unknown team ID becomes the
  grid's inline-editable ID cell (including its reassignment confirmation — see the GUI section),
  standing consents become settings (`dt00_overwrite_allow`), and retry-on-locked-file becomes a
  plain Error. The CLI never prompts: all of these are hard errors there.
- **Progress chatter is not a message.** Red's "- Packing the face folders..." prints map to
  `Progress` events, not catalog entries.
- **Logs**: `issues.log` (Warning+) and `suggestions.log` (Info) are kept as disk artifacts,
  rendered from the same events. Display layers may group repeated (code, scope) pairs; the logs
  stay complete.

### Catalog

Adapted to the Studio export format: Red's Note-file, Kit Configs/Kit Textures-folder,
dependency-check, and self-healing messages are gone; player-folder, link-file, settings.toml, and
savefile messages are new.

**Export level**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `export_extract_failed` | E | archive cannot be extracted/parsed | export skipped (`DropExport`) |
| `export_disabled` | I | root `NO_USE` / `NO_USE.txt` marker disables this source | export skipped; omitted from the grid and from duplicate-ref detection |
| `source_read_failed` | E/F | a pinned source entry cannot be read | optional root file: `DropFile`; folder/task producer: `DropFolder`; required export metadata or unusable source: `DropExport`; output/global source invariant: `AbortRun` |
| `source_changed_during_run` | F | this export's pinned revision changes during planning/materialization | run aborted and partial output discarded (`AbortRun`) |
| `template_override_unreadable` | E/F | a template override file exists but cannot be read (embedded defaults cannot be missing) | folder-local injected template: `DropFolder`; referee-export template: `DropExport`; global CPK/bin template: `AbortRun` |
| `template_override_active` | I | a `templates/` override file is shadowing an embedded template/fallback-bin resource | override used; reported per file per run |
| `export_balls_skipped` | I | export name's first word is `balls` (balls exports belong to the Balls compiler) | export skipped (`DropExport`) |
| `multiple_ref_exports` | E | more than one non-disabled `refs`-prefixed export found at discovery | every conflicting export skipped (`DropExport` per row, plus a Run-scoped summary) |
| `duplicate_aesthetics_export` | E | multiple exports resolve to the same team ID | all conflicting exports skipped (`DropExport`) |
| `boots_id_pool_exhausted` | E | planned player-exclusive/shared boots outputs exceed the team's permanent ID block | export skipped (`DropExport`) |
| `gloves_id_pool_exhausted` | E | planned player-exclusive/shared gloves outputs exceed the team's permanent ID block | export skipped (`DropExport`) |
| `export_empty` | E | no usable content found at root | export skipped |
| `nested_folders_fixed` | W | content found nested one level down (exactly one usable root) | auto-fixed |
| `nested_root_ambiguous` | E | several nested child folders are usable export roots | export skipped (`DropExport`) |
| `nested_root_conflict` | E | loose root file collides with a flattened nested file at the same virtual path | export skipped (`DropExport`) |
| `team_name_unknown` | E | canonical `team_name` not in `teams_list.txt` | export skipped (GUI: ID cell becomes editable for inline assignment) |
| `team_id_out_of_range` | E | resolved ID outside 701–920 | export skipped |
| `teams_list_read_only` | W | a teams-list write (ID cell, updater merge) failed because the data directory is not writable | write dropped; the in-memory list is unchanged (no elevation — see "Resolved decisions", "Teams list") |
| `team_colors_missing` | I | no root `colors.txt` (it is optional) | TeamColor.bin entry left untouched |
| `color_entry_invalid` | W | unparsable RGB line in a `colors.txt` | entry skipped |
| `root_file_unexpected` | W | unknown file at export root | file ignored |
| `portrait_conflict` | E | same player number with differing portraits in player folder and `Portraits/` | export skipped |
| `notes_found` | I | non-empty valid root `notes.txt` present | collected into teamnotes.txt |
| `notes_encoding_invalid` | E | root `notes.txt` is not valid UTF-8 after optional BOM handling | note dropped; export otherwise continues (`DropFile`, not pass-through-eligible) |

**Player folders and shared model folders**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `player_folder_number_invalid` | E | without `players.txt`, a team player folder's number is not in 01–23 | folder discarded (`DropFolder`) |
| `players_txt_slot_invalid` | E | a `players.txt` line's decimal slot is outside 01–23 (teams) / 01–35 (refs) | that `RosterEntry` assignment skipped (`DropSlot`) |
| `player_number_duplicate` | E | without `players.txt`, two team folder names claim the same player slot | both conflicting folders discarded (`DropFolder` on each folder) |
| `players_txt_missing` | E | referee export has no required root `players.txt` (nor a legacy `refs.txt` alias) | export skipped (`DropExport`); `ParsedAestheticsExport` remains available to the Refs arranger for repair |
| `refs_txt_ignored` | W | referee export has both `players.txt` and the legacy `refs.txt` alias | `players.txt` used, alias ignored |
| `players_txt_slot_duplicate` | E | authoritative `players.txt` assigns the same slot more than once, making roster identity ambiguous | export skipped (`DropExport`) |
| `player_unlisted` | W | authoritative `players.txt` is present but does not list this player folder, regardless of any number in its name | folder ignored |
| `players_txt_line_invalid` | E | one nonblank `players.txt` line cannot be parsed into decimal slot + folder name | that `RosterEntry` skipped (`DropSlot`) |
| `players_txt_invalid` | E | file-level encoding failure or roster-wide ambiguity prevents trustworthy normalization, or a referee roster retains zero valid assignments | export skipped (`DropExport`; foundational). An empty normal-team roster remains valid for a kit-only export |
| `players_txt_target_missing` | E | `players.txt` entry names a nonexistent folder | `RosterEntry` scoped assignment skipped (`DropSlot`) |
| `link_target_missing` | E | link file references a nonexistent shared folder | folder discarded |
| `shared_link_duplicate` | E | player folder has more than one shared link for any category (face, boots, or gloves) | folder discarded (`DropFolder`) |
| `link_combined` | I | link file plus local models for the same category; the shared models become parts of the player's own set (Fox: mesh-merged, own ID) | none |
| `shared_folder_orphaned` | W | shared folder referenced by no player | folder skipped |
| `settings_toml_invalid` | E | optional `settings.toml` fails to parse | file ignored and existing savefile values preserved (`DropFile`, not pass-through-eligible); models continue |
| `materials_toml_invalid` | E | a `materials.toml` or `*.materials.toml` file fails to parse | folder discarded |
| `material_file_missing` | E | a glTF file has no matched material file (no name-matched `*.materials.toml`, no catch-all `materials.toml`, no `.common` link to either) | folder discarded |
| `material_undefined` | E | a material name referenced by a glTF file has no definition in any matched material toml | folder discarded |
| `material_key_unknown` | W | a material toml entry has a key outside the schema (typo or newer-Studio field) | key ignored |
| `material_value_invalid` | E | a material toml value has the wrong type, is out of range, names an unknown family/role/state, or uses a native sampler name reserved by a canonical role | folder discarded |
| `material_texture_not_found` | E | a mesh-used glTF material's `base`, `""`-marked, or explicit texture stem cannot be resolved | folder discarded |
| `material_texture_unused` | I | a texture role the target engine has no sampler for (e.g. `metalness` on pre-Fox) | role dropped for that engine |
| `material_parameter_unused` | I | a shader parameter the target engine's shader does not define | parameter dropped for that engine |
| `material_entry_unused` | I | a material toml entry matches no material in the folder's glTF models | none |
| `materials_file_ambiguous` | I | multiple name-matched `*.materials.toml` files match one glTF; definitions layer least → most specific (intentional layered setups are ordinary usage) | none |
| `material_link_layered` | I | a `.common` material link and a local material file share a stem; the Common file layers below the local one | none |
| `gltf_embedded_image_ignored` | I | a glTF embeds texture bytes (stock Blender's default); textures are only ever read from the folder's files | embedded images ignored (once per model) |
| `gltf_bone_unknown` | E | a glTF skin joint has neither `PES_bone` metadata nor a name in the template skeleton | folder discarded |
| `bone_folded_for_version` | I | a used bone the target PES version's skeleton lacks had its weights transferred to the fold table's (or nearest) bone — see "Skeleton retargeting" in the Model conversion plan | none |
| `skeleton_retargeted` | I | the model's bind pose was re-bound from its source version's skeleton to the target's (bones moved more than tolerance) | none |
| `kit_variant_missing` | W | a `kitN` reference has no variant for a kit number the export defines | lowest existing variant copied into the gap |
| `kit_variant_model_fox` | W | per-kit model variants (`*_kit1`, `*_kit2`, …) on a Fox target, which has no model-path indirection | lowest variant used, others ignored |
| `common_link_missing` | E | a `.common` link — model, material file or texture — names a file missing from the Common folder (context: link kind and the Common path looked for) | folder discarded |
| `settings_toml_name_shared` | W | `name` given in a folder mapped to multiple players | name applied to all of them |
| `fpc_conflict` | E | both `fpc.on` and `fpc.off` present in a player folder | folder discarded |
| `fpc_strip_conflict` | W | `settings.toml` strip keys conflict with the folder's FPC marker | FPC preset wins; keys ignored |
| `fmdl_name_invalid` | E | Fox: FMDL in a boots/gloves shared folder not resolving to that category's allowed names | folder discarded |
| `fmdl_fcl_hair_fallback` | I | Fox: arbitrary-named FMDL treated as a face model, routed into the `fcl_hair.fmdl` merge (Red's single-file fallback, generalized) | none |
| `fmdl_merged` | I | Fox: multiple models resolve to the same allowed name; meshes merged into one FMDL (alphabetical source order) | none |
| `merge_material_conflict` | E | merged parts define the same material name differently (Fox merge or pre-Fox `ingame_face` merge) | folder discarded |
| `skl_merge_conflict` | E | merged parts have incompatible skeletons (Fox compares effective SKL content; pre-Fox IR merge compares bone transforms) | folder discarded (`DropFolder`) |
| `skl_no_slot` | W | Fox: custom `.skl` paired with a model resolving to `face_high`/`hair_high`/`oral` (no skeleton slot exists for those) | SKL ignored (no-op file) |
| `ingame_face_explicit_face_model` | E | `ingame_face` combined with explicit face models (`face_high`/`hair_high`/`oral`, in any supported source format) or a shared face link, which require the suppressed face folder | player folder discarded (`DropFolder`) |
| `shared_texture_conflict` | E | a player's tasks produce the same common-texture destination with different bytes | losing task discarded (`DropFolder`; canonical winner face > boots > gloves); identical bytes deduplicate |
| `merged_texture_conflict` | E | two merge-copied parts within one output model produce the same texture destination with different bytes (no canonical winner exists inside one model) | folder discarded (`DropFolder`); identical bytes deduplicate |
| `model_source_ambiguous` | E | duplicate model sources within the selected representation (target-native > glTF > convertible opposite) | folder discarded (`DropFolder`) |
| `model_conversion_failed` | E | selected model cannot be converted to the target format | folder discarded (`DropFolder`) |
| `folder_pack_failed` | E/F | a task cannot build its packed batch | folder/task: `DropFolder`; output-writer/global: `AbortRun` |
| `model_name_invalid` | E | pre-Fox: `.model` not in the allowed names for its category | folder discarded |
| `xml_broken` | E | XML fails to parse (with line/column) | folder discarded |
| `mtl_broken` | E | MTL fails to parse (with line/column) | folder discarded |
| `edithair_unsupported` | E | `face_edithair.xml` / `hair.xml` present | folder discarded |
| `file_type_disallowed` | E/I | extension not in the mode's allowlist (E if `strict_file_type_check`, else I) | folder discarded / kept |
| `fmdl_no_texture_ids` | W | no ID-bearing texture paths found in FMDL | none (double-check hint) |

**Textures** (file-scoped; in model folders the folder fails, elsewhere the file is normally
dropped; the logo sources are the exception — main and `_small` fail as one atomic producer of the
game's three sizes)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `texture_too_small` | E | dimension < 4 | discarded |
| `texture_not_pow2` | E | portrait not power-of-2 / Fox mipmapped without pow2 side | discarded |
| `texture_not_div4` | E | pre-Fox compressed texture not divisible by 4 | discarded |
| `kit_texture_too_big` | E | main kit texture > 2048² or not pow2 | discarded |
| `kit_texture_uncompressed` | E | main kit texture in uncompressed format | discarded |
| `texture_type_mismatch` | E | header doesn't match extension (renamed, not resaved) | discarded |
| `texture_codec_unsupported` | E | codec not convertible in-process | discarded |
| `texture_stem_conflict` | E | two image files with the same stem but different extensions in one model folder | folder discarded |

**XML/MTL content checks** (pre-Fox, plus `face_diff.xml` in Fox)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `xml_texture_path_missing` | E | sampler with no texture path and not auto-fillable (sampler not in the suffix mapping table) | folder discarded |
| `mtl_texture_not_found` | E | texture (by stem) used by one of the model's meshes missing from export | folder discarded |
| `mtl_texture_unused_missing` | I | texture referenced only by materials no mesh uses | none |
| `xml_root_tag_invalid` | E | root tag is not `<config>` | folder discarded |
| `xml_model_type_missing` | E | `<model>` without `type` | folder discarded |
| `xml_model_path_missing` | E | `<model>` without `path` | folder discarded |
| `xml_model_not_found` | E | `path` (or `material`) is a `./` or Common reference whose file is missing from the export | folder discarded |
| `xml_common_path_invalid` | E | Common reference without 3-char subfolder | folder discarded |
| `xml_oral_prefix_missing` | E | PES16 target: model file name starting with none of `face_high_`/`hair_high_`/`oral_` | folder discarded |
| `xml_dif_conflict` | E | the xml carries a `<dif>` and the folder also has `face_diff.xml` (two sources for one datum) | folder discarded |
| `xml_element_unknown` | W | a child of `<config>` other than `<model>`/`<dif>` | kept verbatim |
| `xml_attribute_unknown` | W | a `<model>` attribute other than `level`/`type`/`path`/`material`/`ratio` | kept verbatim |
| `xml_type_unknown` | W | `type` not in the generated vocabulary (the type table plus `uniform_sub`) | kept verbatim |
| `xml_level_lod` | I | `level` other than `0` — the model's LoD level; higher levels are uncommon but known to work | kept verbatim |
| `xml_ratio_invalid` | W | `ratio` that is not a number | kept verbatim |
| `xml_path_unchecked` | W | `path`/`material` in a form the compiler cannot resolve (`model/character/face/common/…` or any other game path): not verified to exist | kept verbatim |
| `xml_face_neck_multiple` | W | more than one `face_neck` entry | kept verbatim |
| `xml_model_unlisted` | W | a model file in the folder that the xml does not reference | file not emitted |
| `xml_face_neck_added` | I | no `face_neck` entry; the dummy entry was appended (Red's rule) | dummy model + mtl emitted |
| `xml_uniform_pes15` | I | PES15 target: `type="uniform"` rewritten to `uniform_sub` (Red's rule) | rewritten |
| `xml_ignored_fox` | I | a user `face.xml` in a folder compiled for a Fox target | xml ignored; models compile by the normal route |
| `face_diff_invalid` | E | `face_diff.xml` structure/base64/binary validation failed | folder discarded |
| `mtl_material_duplicate` | E | material listed twice | folder discarded |
| `mtl_state_invalid` | E | `ztest` ≠ 1 / `blendmode` ∉ {0,1} / `alphablend` ∉ {0,1} | folder discarded |
| `mtl_blendmode_nonzero` | W | `blendmode` = 1 (works, not recommended) | none |
| `mtl_state_missing` | I | state names absent, defaults used | none |
| `mtl_state_nonrecommended` | I | `alphablend`/`zwrite` combination not recommended | none |

The three `mtl_state_*` checks other than `mtl_state_missing` also run on a glTF material's
`[prefox.states]` table when the target is pre-Fox (the format plan's schema mirrors the `.mtl`
states one-to-one; a glTF material with states absent just gets its family's state set, so
`mtl_state_missing` does not apply).

Red's "no models from Common on PES16" check is intentionally **not** ported: the PES16 exe will be
patched to allow loading models from Common, so the compiler must not reject such references.

**User-supplied `face.xml`.** A pre-Fox model folder may ship its own `face.xml` instead of having
the compiler generate one. This is **second-class, accepted on purpose**: the generated xml covers
everything the format knows how to express, and a hand-written one exists to let a user try what the
pre-Fox engines can do that nobody has mapped yet (a `type` no table lists, a `level` other than 0,
an element the generator never writes). Because a malformed xml can crash the game, the compiler
checks it carefully, and the severity line is drawn by evidence, not by taste:

- **Error** — what is known not to work, or cannot work: unparsable XML, a root other than
  `<config>`, a `<model>` without `type` or `path`, a `./` or Common reference to a file the export
  does not contain (the one thing the compiler can verify outright), a Common path without its
  3-character subfolder, the PES16 `oral_`/`face_high_`/`hair_high_` name rule, and two sources for
  the face diff. These are Red's checks (`xml_check.py`), each added after real breakage, plus the
  two structural impossibilities Red happened not to test (a missing `path`, a doubled diff).
- **Warning** — everything outside the vocabulary the compiler's own generator emits, because
  that vocabulary is the in-game-verified set and nothing outside it is: an unknown element or
  attribute, a `type` not in the type table, a non-numeric `ratio`, a path in a form the
  compiler cannot resolve (`model/character/face/common/…` and any other game path — Red waved
  these through silently; the compiler says it did not check). The value is **kept verbatim**: a
  warning is the compiler saying "I can't vouch for this", never "I changed this". A model file in
  the folder that the xml does not list is also a warning — it will not be in the game, which is
  almost always a mistake and occasionally the point.
- **Info** — the compiler's own rewrites, which it performs on a user xml exactly as it would on a
  generated one and reports so the user knows the emitted file differs from the source: the
  `face_neck` dummy appended when no entry has that type; `uniform` → `uniform_sub` on PES15; the
  team ID substituted into Common paths and `kitN` tokens passed through; and the `<dif>` block
  inserted from `face_diff.xml` when the xml has none. On a Fox target the xml is ignored with an
  info line and the folder's models compile by the normal route (Fox has no `face.xml`). Also info,
  though not a rewrite: a `level` other than 0 — it is the model's LoD level, and higher levels are
  merely uncommon, not unverified (a value that has already made the move from warning to known).

Rules that follow from "the xml is the authority in its folder": filename typing is off — the
suffix table is for generating an xml, and this folder has one; `.common` model links are not needed
(the xml names Common paths itself) and an unlisted one is just an unlisted file; only what the xml
references is emitted, plus the MTLs and textures those models bind, which go through the ordinary
MTL and texture checks unchanged. The compiler always re-serializes the xml it emits (it has to, for
the ID substitution and the appended entries), so comments, whitespace and the encoding declaration
of the source never reach the game — the checks above are about content, not formatting. New
knowledge moves a row: when an experiment shows a value works, it joins the generated vocabulary and
stops warning; when one shows a value crashes, it becomes an error with the crash as its citation.

Texture existence is checked **deep**, not shallow: `pes_model` parses the `.model` files and knows
which MTL material each mesh actually binds, so a missing texture on a mesh-used material is a hard
error, while a miss on a material no mesh uses can never break the model and is only reported as
info. Red only reads the MTL and cannot tell used materials from unused ones, so it has to report
every miss as a warning.

**Kits**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `kit_folder_invalid` | E | subfolder name doesn't parse as `<slot>[ - <label>]` with the slot in `p1`–`p9`, `g1`, `all` | kit discarded |
| `kit_slot_duplicate` | E | two subfolders resolve to the same slot (`p1/` and `p1 - Lakers/`) | both discarded — never a pick |
| `kit_textures_inherited` | I | a kit lacks stems that `all/` provides; lists them (`p3: back, leg, name from all/`) | textures inherited |
| `kit_all_file_ignored` | W | `all/` holds something other than kit textures (`config.toml`, `colors.txt`, `icon.txt`, anything else) | file ignored |
| `kit_all_unused` | W | `all/` present but no kit folder to inherit from it | — |
| `kit_config_generated` | I | no `config.toml`; generated from template (with FPC values if team FPC is on) | auto-fixed |
| `kit_config_invalid` | E | `config.toml` fails to parse or validate (ranges, cross-field constraints) | kit discarded |
| `kit_config_version_clamped` | W | a field doesn't fit the target PES version's encoding (e.g. Name Y > 16 before PES 21) | value clamped |
| `kit_config_fpc_adjusted` | I | team kit-FPC status is On but a config lacks the FPC values — supplied configs and unexported slots' base entries alike, GK kit included | auto-fixed (values written; FPC values are never auto-reverted) |
| `kit_config_fpc_unpatched` | W | team kit-FPC status is On but an unexported kit slot has no base entry or config to patch | slot left alone; the team needs a kit export |
| `kit_placeholder` | I | the kit's effective textures lack `kit.dds` (an empty folder included); the bundled checkerboard stands in | placeholder kit emitted: checkerboard texture, template config unless supplied, UniColor entry per the colors fallback |
| `kit_colors_derived` | I | kit `colors.txt` missing, or present but yielding fewer than two valid colors; menu colors extracted from the kit's own main texture | auto-fixed |
| `kit_colors_missing` | W | no usable `colors.txt` and no own main texture to derive from (placeholder kits without a `colors.txt` always) | the magenta/black "no colors chosen" pair is written, so the gap shows in the game menus |
| `kit_icon_invalid` | W | `icon.txt` doesn't parse to a number in 0–23 | default icon (3) used |
| `kit_texture_name_invalid` | E | texture doesn't carry the `kit` prefix (`kit.dds`, `kit_mask.dds`, …) | file discarded |
| `kit_texture_not_used` | I | the kit's effective set has a `kit_mask` on a Fox target or a `kit_srm` on a pre-Fox target — the other engine's map | file not emitted (never converted into the other map) |
| `kit_layout_conflict` | E | both `pre-fox` and `fox` markers in one kit folder | kit discarded |
| `kit_layout_converted` | I | the kit's layout marker names the other engine than the target; names the direction (`p1: pre-fox → fox`) | `kit` and its mask/srm re-laid out per `KIT_LAYOUT_REMAP`; other textures untouched |

**Portraits / Logo / Collars / Common**

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `portrait_name_invalid` | E | not `player_NN.dds` with NN in 01–23 | file discarded |
| `logo_file_invalid` | E | a root `logo*` file has an unknown tag in its stem, a disallowed format, or fails to decode in the deep pass | no logo emitted for the team (both files dropped as one unit — the game needs all three sizes) |
| `logo_role_duplicate` | E | two files claim the same role (two `logo*`, or two `logo_small*`) | no logo emitted for the team |
| `logo_small_without_main` | E | `logo_small*` present with no main `logo*` | no logo emitted (the small image is not a source for the large sizes) |
| `logo_fit_applied` | I | a non-square source was made square; names the mode (`fit` by default, or the file's tag) | — |
| `logo_upscaled` | W | a source is smaller than its largest target (512² for main, 128² for small) | emitted upscaled |
| `collar_id_invalid` | E | collar filename doesn't parse as `collar_[ID]`, the ID is outside the supported stock-collar range, or it names the reserved FPC collar 105 | collar file discarded |
| `collar_id_conflict` | E | another export already claimed this stock collar ID in this run (canonical export order) | later team's collar discarded; its configs not rewritten |
| `common_file_disallowed` | E/I | as `file_type_disallowed`, Common scope | files discarded / kept |

**Referees** — referee exports use the same player-folder format (see "Referee export processing"),
so all player-folder, texture, and XML/MTL messages above apply as-is. Referee-specific additions:

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `ref_marker_needs_consent` | E | Fox `ref_marker.dds` present but dt00 overwrite not enabled (setting) | marker skipped |
| `dt00_write_failed` | E | `dt00_x64.cpk` cannot be updated with the converted Fox referee marker | deployment transaction fails and rolls back; compile artifacts remain available |

Referee-marker deployment findings are environment-level dispositions: `pass_through` never
overrides them, even when the referee CPK itself remains compilable.

**Output stage and savefile** (Run scope)

| ID | Sev | Condition | Consequence |
|---|---|---|---|
| `duplicate_path` | W/E | manifest preflight finds colliding output paths | folder collision: losing folder blocked (`DropFolder`, E); whole-export collision: losing export blocked (`DropExport`, E); sideload override: sideload wins (`Keep`, W) |
| `cpk_write_failed` | F | incremental CPK writing fails | run aborted and partial CPK discarded (`AbortRun`) |
| `output_commit_failed` | F | a completed CPK/tree cannot be atomically committed to its final output path | run aborted; prior published outputs remain untouched (`AbortRun`) |
| `uniparam_compile_failed` | F | UniformParameter compilation failed (output would crash PES) | run aborted |
| `pes_folder_not_found` | E | `pes_folder_path` invalid at deployment time (includes the no-PES-install machine) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `pes_version_mismatch` | W | `pes_folder_path` exists but holds no `PES20{pes_version}.exe` — a different-version exe suggests a wrong `pes_version` setting (Red's suppressible console notice becomes a plain per-run warning, dropping `state/ver_mismatch_warned.txt`; the GUI also surfaces this live via the version selector's red/yellow state — see the core plan's Sidebar) | none; deployment proceeds |
| `dpfilelist_missing` | E | no `DpFileList.bin` in download folder | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `dpfilelist_outdated` | E | installed DpFileList lacks a target of this run that the bundled official DPFL lists (old layout, e.g. no `teams` slots) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Upgrade DpFileList, CLI names `upgrade-dpfl` |
| `cpk_name_unlisted` | E | CPK name in neither the installed nor the bundled DpFileList (a genuinely unknown name) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped |
| `cpk_slots_exhausted` | F | multi-CPK content does not fit the available slots at `cpk_part_max_size` (names the shortfall) | run aborted; the DPFL author adds slots or the cap is raised |
| `cpk_team_exceeds_cap` | F | a single team's compiled content is larger than `cpk_part_max_size` (impossible in practice at 3 GB) | run aborted; teams are never split across parts |
| `cpk_size_over_limit` | W | a single-CPK output exceeds `cpk_part_max_size` (valid for PES; the DLC repository will reject it) | none |
| `old_cpk_locked` | E | old CPK cannot be replaced (PES running) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Retry (deployment only) and Open output folder |
| `deploy_target_unwritable` | E | destination folder denies writes (typically elevation needed under `Program Files`) | deployment skipped; staged CPKs promoted to `output/`; savefile step skipped; GUI offers Relaunch as administrator. Normally pre-empted by the live writability preflight (see "Post-processing") |
| `deploy_skipped_by_flag` | I | `--no-deploy` given (CLI) | staged CPKs promoted to `output/`; savefile step skipped; run is clean |
| `sideload_active` | I | sideload folder present and injected | none |
| `savefile_autodetected` | I | `savefile_path = auto` resolved a savefile under Documents\KONAMI (names the path; noted especially when several account folders existed and the newest was chosen) | none |
| `patch_written` | I | the aesthetics patch was written beside the output CPK (names the path and the teams it covers) | none |
| `savefile_missing` | W | aesthetics present but no savefile configured/found; the patch is the run's only savefile output | savefile step skipped; the message names the patch and the save editor's apply action |
| `savefile_skipped_pes_running` | W | savefile changes pending but PES is running (any output mode; the motivating case is Sider-mode iteration) | savefile step skipped; applied by the next compile with PES closed |
| `savefile_write_failed` | E | savefile could not be updated | deployment transaction fails and rolls back; compile artifacts remain available |

Deployment-stage errors above mean no partial installation: preflight failures leave installed
outputs untouched; failures after replacement starts roll back the deployment as a whole. A missing savefile
is the explicitly permitted warning-only case. A present savefile that cannot be read or whose
version differs from the compile target is an error, not `savefile_missing`; do not modify it or
publish a partially deployed run.

The `cpk_write_failed` and `output_commit_failed` consequences are required guarantees: partial
generated output is discarded and prior published output remains untouched. The corresponding open
questions choose the `.partial`/rename/rollback implementation, not whether these guarantees apply.

Fatal settings/environment problems (invalid PES version, unwritable output folder) are surfaced by
the settings UI and CLI argument validation before a run starts, not as pipeline messages.

---

## Settings

Inherited from Red/Blue (`settings.ini`, `settings_default.ini`), reorganized into the suite's
two-level settings model: **common settings** live in `studio_core` (shared by all tools), **tool
settings** live in the Team compiler's own settings section.

### Common settings (studio_core)

| Setting | Old default | Notes |
|---|---|---|
| `pes_version` | 15 | Sidebar selector; 15–21. 4cc Studio defaults to 19 as the version most relevant to the current community; derives `fox_mode` (≥18), per-version file extensions, game paths, bin names |
| `pes_folder_path` | `C:\Program Files (x86)\Pro Evolution Soccer 20**` | `**` replaced with the version; derives `download/` path and `PES20{version}.exe` path |
| `exports_folder_path` | `exports/` | Watched by the GUI and scanned once by the CLI; belongs in `studio_core` so the Team, Refs, and Balls tools share one export source. Relative paths resolve **beside the executable** in both data-location modes (see "Path resolution"), so the folder sits next to `quick_compile.bat` like Red's `exports_to_add/` |
| `thread_count` | 0 (auto) | Was a Blue CLI arg; auto = logical cores − 1 |
| `memory_cap_percent` | 80.0 | Was a Blue CLI arg; memory budget cap |
| `check_for_updates` | 1 | Was per-compiler (`updates_check` in Red's ini); becomes a suite-wide update check |

### Team compiler settings

"Old default" is Red's value (— for settings Red did not have); "New default" is the Studio's. A
differing new default is a deliberate decision, explained in the Controls column.

| Setting | Old default | New default | Controls |
|---|---|---|---|
| `cpk_name` | `4cc_90_test` | same | Single-CPK output `CpkStem` (shared validation contract in the Writer section) |
| `output_folder_path` | `patches_output/` (hardcoded) | `output/` | Root for promoted CPKs (degraded runs and `--no-deploy`), the `.staging/` folder, `test_output/`, `sider_output/`, and `teamnotes.txt`. Relative paths resolve **beside the executable** (Red's `patches_output/` in its new place; `%APPDATA%` is no place to look for a CPK) |
| `sider_output_path` | — | `sider_output/` | Where Sider-mode runs write the unpacked PES folder structure; relative paths resolve beneath `output_folder_path`. For the prototyping loop the user either points this at Sider's livecpk root directly or adds the default folder to `sider.ini`'s `cpk.root` list — the settings UI shows the resolved absolute path for copy-pasting into `sider.ini` |
| `run_pes` | 0 | same | Launch PES after compiling (only after a successful deployment; a degraded run never launches) |
| `multicpk_mode` | 0 | same | Cup DLC mode: team content into size-split `teams` parts + the bins CPK (see "Multi-CPK mode: teams parts"); replaces Red's faces/uniform/bins content split |
| `teams_cpk_name` | `4cc_40_faces` + `4cc_45_uniform` | `teams` | Stem of the teams part slots; the slots themselves are every DpFileList entry matching `{prefix}_{NN}_{stem}` (`4cc_40_teams`, `4cc_41_teams`, …), ordered by number. Replaces `faces_cpk_name` and `uniform_cpk_name` |
| `cpk_part_max_size` | — | `3 GB` | Cap per teams part (TOC included); the writer places whole teams, rolling to the next slot when the next team would not fit — teams are never split across parts. Chosen comfortably under Git for Windows' 4 GiB object ceiling; five slots give 15 GB. Single-CPK runs are not split, only warned (`cpk_size_over_limit`) |
| `bins_cpk_name` | `4cc_08_bins` | same | Multi-CPK bins name |
| `refs_cpk_name` | `4cc_35_referees` | same | Referee CPK name |
| `dds_compression` | 0 | **auto** | WESYS-zlib every emitted DDS. Tri-state `auto` / `1` / `0`; `auto` **follows `multicpk_mode`**, since multi-CPK is the cup DLC workflow and containing the full-cup DLC's size is the whole point of the setting — a manager compiling one team gets no compression and no cost, a cup maintainer gets it without remembering a second switch. **Pre-Fox only (PES ≤17)**: on Fox versions the setting is ignored whatever its value, because FTEX conversion already provides the size reduction there and the game does not expect zlibbed FTEX. Cheap in-process either way (see "DDS compression cost" below) |
| `strict_file_type_check` | 1 | same | Disallowed file types are errors vs info notes (`file_type_disallowed`) |
| `pass_through` | 0 | same | Keep folders with errors instead of discarding them (cup DLC workflow) |
| `savefile_path` | — | auto | `EDIT00000000` to update. `auto` runs `pes_savefile`'s discovery under the user's **Documents\KONAMI** folder for the selected version (the savefile is never inside the PES install, so `pes_folder_path` plays no part — see "Savefile discovery" in the Savefile plan); newest account wins when several exist (`savefile_autodetected`); nothing found → `savefile_missing` (Red never touches the savefile) |
| `teams_list_path` | `teams_list.txt` (hardcoded, beside the exe) | `teams_list.txt` | Location of the working teams list; relative paths resolve in the data directory (see "Path resolution"). Created from the embedded list on first run |
| `dt00_overwrite_allow` | — | 0 | Allow `ref_marker.dds` injection into the `dt00_x64.cpk` system file (replaces Red's interactive prompt) |
| `quick_compile_close_on_success` | — | 0 | After a compile started by GUI autorun (`quick_compile.bat` → `studio --gui team-compiler compile`), close the window when the run completes with no Error-level findings and nothing skipped for errors; stay open on errors, failed deployment, or cancellation so the grid/log can be reviewed. Warnings alone still close (they are in the logs, as Red's `pause_allow = 0` reasoned). Never applies to a manual Compile click |

Output mode (normal / test / sider) is not a persisted setting: it is the Compile button's dropdown
in the GUI and a flag on the CLI subcommand, as in Blue's `--mode` argument.

**DDS compression cost.** Red's `dds_compression` is a separate pass at the end: walk
`patches_contents/`, read each DDS, Python `zlib.compress` at level 6, wrap in the 16-byte WESYS
header, write it back — I/O-bound and serial. In the Studio it is one more step in the texture task,
on bytes already in memory, running on rayon workers like everything else. Rough budget for a
200 MB pre-Fox export that is mostly DDS: `flate2` deflate at level 6 runs at roughly 30–60 MB/s
per core on block-compressed texture data (DXT blocks are high-entropy, so it is neither fast nor
very effective — expect 10–30 % size reduction), i.e. ~4–7 s of CPU spread over ~7 workers, well
under a second of wall time per export and far below the CPU BC3 encode of any raster-source
texture in the same task. Two implementation notes: prefer the `zlib-rs` backend of `flate2`
(faster than `miniz_oxide`, pure Rust) and measure whether a lower level (1–3) loses meaningfully
on DXT data — it usually does not, and halves the cost; and the Red parity harness must compare
WESYS-wrapped entries **decompressed**, since deflate output is not byte-stable across
implementations and levels. Textures already WESYS-wrapped in the export are passed through, not
re-compressed (Red's `zlib_file` skip). Measure in Phase 4. With the `auto` default the cost is only
ever paid on cup-DLC compiles, where a few seconds against a 48-team run is irrelevant — the
measurement decides the compression level, not whether the feature is on.

### Path resolution

| Artifact/input | Base location |
|---|---|
| Logs and persistent state | Selected data directory |
| Relative `exports_folder_path` | Executable directory in both data-location modes, so `exports/` sits beside `quick_compile.bat`; created on first run |
| Relative `output_folder_path` | Executable directory in both data-location modes (`output/` beside the exe, created on demand); holds `.staging/{run_id}/` during a run |
| `sideload/` input | Selected data directory (an explicit sideload setting may be added later) |
| Relative `teams_list_path` | Selected data directory — the file is user data once the grid writes ID assignments into it, so it follows the settings file (see "Resolved decisions", "Teams list"); nothing is bundled beside the binary |
| `templates/` override directory | Selected data directory; each file shadows the matching embedded template/fallback-bin resource (see "Resolved decisions") |
| `savefile_path` | Absolute when explicitly set; `auto` resolves under the shell's Documents folder → `KONAMI\{game folder}[\{account id}]\save\EDIT00000000` per the Savefile plan's discovery table — independent of `pes_folder_path` |
| `pes_folder_path` | Absolute PES installation path (after expanding its documented `**` version placeholder) |
| Generated CPKs, `test_output/`, `sider_output/`, `teamnotes.txt` | Resolved `output_folder_path` |
| Installed CPKs | `{pes_folder_path}/download/` |

The GUI and CLI use these same bases; a CLI exports-root override changes only that invocation's
resolved export source.

### Dropped settings

| Setting | Why it disappears |
|---|---|
| `cache_clear` | No disk cache: the pipeline is in-memory (VirtualTree), so there is no equivalent of Red's `patches_contents/` staging folder for the setting to clear |
| `move_cpks` | Deployment is always attempted; a run that cannot deploy degrades to `output/` by itself, and the deliberate "build but don't install" case is the CLI-only `--no-deploy` flag. See "Why no `move_cpks`" under "Post-processing" |
| `pause_allow` | The `pause()` pattern is gone; errors/warnings are events rendered by the GUI/CLI. Its unattended-run half (`pause_allow = 0`: finish without stopping) survives as `quick_compile_close_on_success` for launcher-started runs |
| `admin_mode` | Manifest-based UAC / GUI prompt (see core plan) |

### CLI

```text
studio team-compiler compile [exports-root] [--mode normal|test|sider] [--export <path>]... [--no-deploy]
studio team-compiler check [exports-root] [--export <path>]...
studio team-compiler upgrade-dpfl [--yes]      # replace the installed DpFileList with the bundled official one
studio --gui team-compiler compile [exports-root] [--mode normal|test|sider]   # GUI autorun
```

(Refreshing `teams_list.txt` from a savefile is the Save editor's `export-teams-list`; see the
[Save editor plan](save_editor.md) "CLI".)

`upgrade-dpfl` prints what the override changes (entries that will no longer be loaded, with the
sizes of any matching `.cpk` files found in `download/`) and stops unless `--yes` is given; it never
deletes CPK files non-interactively — those are listed for the user to remove. See "DpFileList
upgrade" under "Post-processing".

The optional positional argument is an exports-root override for that invocation; when omitted, both
commands use the common `exports_folder_path`. It is not a single-export path and is never persisted
back to settings.

`--no-deploy` builds the CPKs into `output/` without touching the PES install or the savefile (see
"Post-processing"). It is CLI-only and has no GUI or settings counterpart: it is the cup maintainer's
DLC-production flag and the build-box flag, not something a regular user should find and leave on.
It is rejected together with `--mode test|sider`, which have no deployment to skip.

`--export <path>` (repeatable) restricts the run to the named exports — a path to an export folder
or archive, anywhere on disk, not necessarily under the exports root. This is the contract for
**external callers driving a Sider prototyping loop**, first of all the `pes-models` Blender
extension's "export model and compile for Sider" button: it saves the model into the player folder
it was loaded from (it knows the export from the launch manifest — see the Player aesthetics editor
plan), then runs

```text
studio team-compiler compile --mode sider --export "D:\exports\aaa_export"
```

and PES, with Sider running, shows the result on the next model load. Without `--export` the plugin
would recompile every export in the folder on each iteration. Console output follows the ordinary
`-` prefixed format, and the exit code distinguishes at least clean / errors in some scope / aborted
(the exact mapping is fixed by the "Run-result semantics" open question) so a caller can show a
one-line verdict and point at the log for detail. The CLI is a separate process with its own run
and does not require the GUI to be closed. A Sider-mode run with PES open skips the savefile step
(`savefile_skipped_pes_running`, see "Post-processing") — the model iteration lands via livecpk and
the savefile catches up on the first compile after PES is closed; bins routing in Sider mode is part
of the "Output-mode artifact routing" open question.

The `--gui` form (the core plan's "Launch modes") is what `quick_compile.bat` runs. The tool's
`gui_run` implementation accepts `compile` only — `check` is meaningless in the GUI, where checking
is live — and queues the compile until the initial exports check has completed, then triggers the
same code path as the Compile button with the parsed `--mode`. `quick_compile_close_on_success`
governs what happens when that run ends.

---

## Object model and export format

The export object model, its validation progression (`ParsedAestheticsExport` → `ValidatedAestheticsExport` →
`ResolvedAestheticsExport`), the `aesthetics_export` crate layout, the player-folder format, `settings.toml` in
exports, and the FPC marker files are specified in the [Aesthetics export plan](aesthetics_export.md). The
compiler consumes them; only the compile-time behavior — the loaded processing types
(`FaceModelFolder` and friends, with their conversion and packing methods), the `BuildManifest`,
planning, processing, packing, bins, GUI, and CLI — lives in this crate and this plan.

---

## Unimplemented Blue Features to Port

Blue is a prototype missing features that Red has. The Rust rewrite must implement these.

### Referee export processing (Red is the behavioral spec)

Blue explicitly skips referee exports (`coordinator.py:218`); Red's
`Engines/python/lib/referee_tools.py`, together with its referee-related path editing in
`fmdl_editing.py` and export routing in `export_move.py`, is the authoritative behavioral
specification.

**The referee export format changes only in name.** Red's current referee layout — `refs.txt` for
number→name mappings plus per-referee face/boots/gloves/common subfolders — was the prototype of the
player-folder format, tested only on referees. Referee exports adopt the **unified player-folder
format**: a `refs` export contains `Players/` folders exactly like a team's aesthetics export, with shared models
in named shared folders + link files. Since the per-referee subfolders are the format's reserved
subfolders (see "Reserved subfolders" in the [Aesthetics export plan](aesthetics_export.md)) and `refs.txt` is accepted as a
legacy alias of `players.txt` (see "Player numbering"), a current-day referee export **compiles as it
is**. The Export upgrader still upgrades it to the fully unified layout — flat player folders and
`players.txt` — like every other legacy export, since the flat layout lists a player's contents more
explicitly; the legacy acceptance exists because it costs little, not as a second dialect to author
in.

**How a referee CPK is composed.** Every PES version has **35 referee slots**
(`referee001`–`referee035`), used randomly or pseudorandomly in every match. Filling all 35 with
distinct models is overkill, so in practice **about 8 models are repeated across the slots**, with
each model's repetition count determining how rare or frequent that referee is. Red drives this
through `refs.txt`, which may list the same folder name under multiple slot numbers. The unified
format keeps the mechanism as the required root **`players.txt`** slot mapping (see "Player
numbering" in the export format section): up to 35 slot entries, each naming a player folder, with
the same folder free to appear under any number of slots. The general multi-mapped player rule
performs shared preparation once per distinct source folder and slot instantiation once per listed
referee slot, rendering that slot's `refereeXXX`/`k99XX`/`g99XX` IDs and paths. Referee player
folders therefore carry no numbers in their names — the mapping lives in `players.txt`.

**Preparing the slot mapping is a separate tool's job.** The slots are not drawn uniformly — each
PES version has measured slot appearance rates (flat per-slot chances on 17–21, a pattern table on
15/16 that demands matchday rotation) — and on PES 18–21 the community's Fox referee hook overrides
the draw from per-match lists. The [Refs arranger](refs_arranger.md) edits `players.txt` inside the
refs export (against those tables, or as one slot per referee plus a weighted fallback fill when
the hook decides who appears) and, on Fox, the hook's `ref_lists.txt` beside it, then switches the
user to this tool for the compile — the arranger never compiles, and this tool's referee behavior
is unaffected by it: a saved `players.txt` arrives like any other file edit via the folder
watcher, and `ref_lists.txt` is not this tool's concern.

What carries over from Red as compiler-internal behavior (invisible in the format):
- `ExportIdentity::Referees`, rendered as fixed numeric ID 999 only inside game paths/templates (999
  is not a normal `TeamId`); per-referee IDs generated as `refereeXXX` / `k99XX` / `g99XX`
- Per-referee common structures and XML/MTL/FMDL path updates — no longer referee-specific: the
  standard texture relocation step moves **every** player's textures to a per-player common
  subfolder (see "Per-model-folder parallel steps"), which is Red's referee-only preprocessing
  generalized to all players
- Referee base template content (`refscpk` templates)
- `ref_marker.dds` handling: pre-Fox template injection, Fox `dt00_x64.cpk` injection (behind an
  explicit consent setting instead of Red's interactive prompt)

### Features that disappear in a compiled GUI app

These Red features are consequences of being a distributed Python script and are eliminated by the
Rust + GUI architecture:

| Feature | Red location | Why it disappears |
|---------|-------------|-------------------|
| Auto-update system | `updating.py` (22KB) | GUI "check for updates" dialog; Rust HTTP via `reqwest` |
| Self-healing / dependency check | `dependency_check.py`, `file_management.py` | Single binary with embedded templates/fallback bins (see "Resolved decisions") — no missing files or packages |
| Admin privilege elevation | `admin_tools.py` | The shared `elevation` lib (manifest execution level + elevated relaunch) |
| First-run wizard | `settings_management.py:67-128` | GUI settings dialog |
| Settings transfer between versions | `settings_management.py:130-211` | Settings versioning in the GUI |
| Interactive settings editing | `settings_management.py:23-66` | GUI form fields |
| Comment-preserving INI parser | `commentedconfigparser` dependency | Not needed; GUI manages settings |
| Step-by-step run modes | `compiler_main.py:72-93` | CLI subcommands for automation |
| Log username cleaner | `log_username_clean.py` | Trivial, optional |
| `pes_uniparam_edit` standalone tool | `pes_uniparam_edit.py` | Covered by `uniparam` format parser |

### Size deltas in shared modules

Red's shared modules are larger than Blue's due to referee-related edge cases and additional
validation:

| Module | Red | Blue | Delta |
|--------|-----|------|-------|
| `export_move.py` | 31KB | 23KB | 8KB (referee path logic) |
| `bins_update.py` | 18KB | 12KB | 6KB (more bin-packing logic) |
| `fmdl_editing.py` | 19KB | 12KB | 7KB (`fmdl_texture_paths_change` for referees) |
| `xml_editing.py` | 22.2KB | 14.8KB | 7.4KB (more template manipulation) |
| `export_check.py` | 44KB | 41KB | 3KB (minor) |

These deltas represent logic that must be accounted for in the Rust port. Use Red as the reference
for completeness.

---

## GUI: the live-validating team grid

The primary view is a grid where **each aesthetics export is a row**, keyed internally by stable
`ExportId` rather than display name, with the team name label on the left, the team ID cell next to
it, one cell per player at a fixed position by number, plus one cell per kit:

```
/aa/   [714]   [01][02][03][04][  ][  ][  ][  ]   [GK][K1][K2]
/bb/   [???]   [01][02][03][04][05][06][07][08]   [GK][K1]
/cc/   [809]   [01][  ][03][  ][05][  ][  ][  ]   [GK]
```

- **Team name label**: fixed-width left column with the canonical `/xx/` `team_name`; the tooltip
  shows the complete `export_display_name` and resolved identity. The label doubles as the **cell
  for export-scoped and non-slot task messages**: shared model folders, Logo, Common, Collars,
  RefMarker, and root-artifact scopes have no cells of their own, so their status and messages roll
  up here. For example, an orphaned shared folder (`shared_folder_orphaned`) turns the label yellow,
  with the details in its tooltip.
- **Team ID cell**: between the team name and the first player column, showing the ID resolved from
  `teams_list.txt`. When the team is not on the teams list (`team_name_unknown`), the cell turns
  **red and editable**: the user types an ID and confirms with Enter, and the ID is **written to the
  teams list** — the successor to Red's interactive prompt. The entry is validated on confirm:
  numeric and in the 701–920 range (violations keep the cell red with the reason in the tooltip). An
  ID **already assigned to another team** asks for confirmation on the cell ("in use by {team};
  reassign?") before overwriting the old entry — the same overwrite flow Red's prompt offered, kept
  because its purpose is correcting a stale teams list. The teams-list write is picked up like any
  file change: every export re-resolves and revalidates, so a reassignment's effect on other rows
  (the team that lost the ID turning red) is immediately visible. If the write fails because the
  data directory is not writable, the cell reverts and stays red with `teams_list_read_only` in
  its tooltip; there is no elevation path (see "Resolved decisions", "Teams list").
- **Player cells**: one per player, labeled with the player number (`01`–`23`, from the folder name
  or the root `players.txt`). The columns are **fixed positions 01–23**: a player without a folder
  in the export leaves its column completely empty (no cell, no number), so columns always align
  across rows by number — missing players are visible at a glance.
- **Referee rows**: the fixed positions extend to `01`–`35` (one per referee slot); each slot listed
  in `players.txt` gets a cell, and slots backed by the same model folder share its status (the
  tooltip names the folder, making the repetition visible).
- **Kit cells**: one per kit — the `Kits/` folder has one subfolder per kit (containing
  `config.toml`, `colors.txt`, and that kit's textures), which gives the per-kit cell granularity
  naturally; `all/` is not a kit and gets no cell — its effect shows in the kit cells' tooltips,
  which name the folder (`p1 - Lakers`) and the stems inherited from `all/`. Kit cells use a
  separate fixed-slot loop (`GK`, then `K1`–`K9`) following the same
  reserve-every-position rule as player cells; they use the same square shape and are separated from
  the player group by a small gap. Color carries status; the label carries identity. Once a kit's
  menu colors are known — its `colors.txt` parses at check time, or the compile derives them from
  the texture — the cell additionally shows its **menu icon's pattern** (the `icon.txt` pattern
  rendered by `color_tools` in pattern-only mode — a flat two-color swatch, no shirt silhouette,
  which wouldn't read at cell size) drawn centered under the label on the status background. A
  failed kit keeps its error state and shows the existing base-`UniColor.bin` colors rather than the
  unwritten pair. This is the kit colors' proof-of-life: a row of identical all-black miniatures
  exposes a lazy set of `colors.txt` files at a glance, and a derived-color miniature shows exactly
  what the extraction fallback picked before anything ships.
- **Task-to-cell mapping**: player model and portrait tasks update every mapped player/referee slot
  cell; kit tasks update their kit cell; Logo/Common/Collar/RefMarker/root-artifact work updates the
  team/ref row label. Run/output failures appear in the run strip and log, with run-abort state
  propagated to pending cells as described below.
- **Hover**: tooltip with folder name + warning/error text. **Click**: opens a detail panel (and
  later, the 3D preview from Future Features).
- One row per team bounds the row count (~50), so no virtualization is needed; direct egui rendering
  suffices.

---

## Live validation (no Check or Refresh buttons)

The exports folder is monitored continuously; checking requires no user input:

- **Folder watching** via the `notify` crate (`ReadDirectoryChangesW` on Windows, inotify on Linux),
  with `notify-debouncer-full` for event debouncing. New exports trigger checks after a ~1–2s quiet
  period plus a file-size stability check (so half-copied archives aren't parsed). Deletions and
  renames update the grid immediately.
- **Parallel checking**: each triggered check is an independent task on the rayon pool — never on
  the UI thread — and multiple exports check concurrently (e.g. on first launch over a full exports
  folder, or when several exports are dropped in at once). Within an export, per-scope checks fan
  out in parallel too. Results arrive as `Message`/`FolderStatus` events mapped to each scope's
  slot, kit, or row-label cell; cells flip from `checking` as each scope finishes, not per export.
  Events carry the run and export revision they were computed from, and the receiver rejects results
  older than the row's current revision, so a stale check can never overwrite newer state.
- **Deep check (plain-folder exports)**: full parse + validation of every file. Rust makes this
  near-instant for folders. When compiling a full DLC, users normally submit plain folders, so the
  main workflow gets full validation automatically.
- **Shallow check (packed archives)**: archive extraction (LZMA) is the real bottleneck, not CPU —
  so `.zip`/`.7z` exports get a shallow check only: the archive's directory listing is read (file
  names, sizes, structure) without extraction, catching naming/nesting/file-type issues in
  milliseconds. Shallow checks never decompress solid data: a `.zip`'s per-entry random access lets
  the structure pass also read small root metadata (`players.txt`, kit `config.toml`) cheaply, while
  a solid `.7z` yields only its entry table — its content-dependent checks (roster syntax, config
  parsing) defer to the compile-start deep check, and the parse result records the roster as
  *unread* rather than missing. **Shallow-passed cells are blue instead of purple**, indicating the
  export may still contain issues that only a deep check would find.
- **Compile-start deep check for archives**: compilation must materialize archive contents, so at
  Compile their deferred deep check runs before the process-or-skip decision. ZIP entries are
  materialized lazily as needed; solid 7z data is decompressed into an archive-owned buffer. Neither
  format is extracted into a mutable filesystem tree. Errors missed by the shallow check surface
  here with complete messages, so one compile round reports everything wrong with an archive instead
  of leaking them across successive attempts.
- **Check cache**: unchanged exports are not re-checked, so the grid can be warm on launch. A root
  `(path, size, mtime)` tuple is insufficient (it misses child-file edits); the cache identity is a
  **recursive manifest fingerprint** — a hash over the export's sorted relative paths, sizes, and
  mtimes (an archive hashes its file's identity) — combined with the validation-context revisions:
  the teams-list revision and the validation-affecting-settings revision, since both change results
  without changing export files. A watcher-driven revision counter was rejected as the identity
  because it cannot validate cold state across restarts — warm-on-launch is the point of the cache;
  the watcher only triggers re-fingerprinting.
- **Discard-and-re-read**: checked exports are not kept in memory; the compile re-reads from disk
  (predictable memory use, consistent with the memory budget). The check cache makes re-validation
  free. Deep checks acquire from the same memory budget as compilation, so a first launch over a
  large exports folder cannot OOM any more than a compile can.

---

## Cell states

```
        (folder watcher)
unchecked (background) → checking (inverse background) → partial-ok (blue)/full-ok (purple)/warning (faint yellow)/partial-error (faint red)/error (red)
        (Compile pressed — eligible cells are pending; effective DropFolder/DropExport scopes are blocked)
processing (bold number, color kept) → done (green)/done with warning (yellow)/done with errors (orange-red, pass-through or DropFile only)
```

Processing eligibility follows the effective disposition, not severity alone. `DropFile` discards
only the affected file and processes the remaining folder, which ends as `DoneWithErrors`;
`DropSlot` removes only the affected roster assignment (and therefore its cell) while other slots
remain eligible; `DropFolder` skips only that folder while sibling tasks continue; `DropExport`
skips everything belonging to that export; `AbortRun` terminates the run. An effective `DropExport`
gives every player and kit cell in that row a derived blocked state—not pending or processing—and
each tooltip references the export-level cause. A skipped scope otherwise keeps its error status
when Compile is pressed.

Planning messages are applied before manifest tasks become pending: rows dropped by planning become
derived blocked rows, valid tasks proceed, and a run-fatal planning failure turns all
otherwise-pending cells into a run-aborted outcome. A plan with no eligible tasks is a no-op
compile: it writes no artifacts and never replaces installed CPKs.

The debugging invariant **a red final compile outcome means nothing from that scope was written**
applies to final outcomes, not check-phase colors. During checking, a deep Error with `DropFile`
disposition is red because the offending file will be dropped; after compilation, the folder is
`DoneWithErrors` because its remaining content was written. Under `pass_through`, eligible
content-level `DropFile`/`DropFolder` findings are overridden to keep, so written errored content
likewise ends as `DoneWithErrors`; `studio_core::FolderStatus` includes that distinct state, and the
completion summary reports it separately from skipped folders.

Packed archives receive their deferred deep check at compile start (see "Live validation"), so by
the time the process-or-skip decision is made their error list is complete rather than
first-error-only.

egui renders cells with `painter.rect_filled` — direct GPU drawing, no DOM:

```rust
// Check-phase and compile-outcome colors. Processing reuses the retained
// check-phase status; the label turning bold is its only extra presentation.
fn cell_color(
    status: FolderStatus,
    check_status: FolderStatus,
    bg: Color32,
    fg: Color32,
) -> Color32 {
    let displayed = if status == FolderStatus::Processing {
        check_status
    } else {
        status
    };
    match displayed {
        FolderStatus::Unchecked       => bg,
        FolderStatus::Checking        => fg,
        FolderStatus::PartialOk       => Color32::from_rgb(0x32, 0x78, 0xDC),
        FolderStatus::FullOk          => Color32::from_rgb(0x9B, 0x59, 0xB6),
        FolderStatus::Warning         => Color32::from_rgb(0x8F, 0x82, 0x21),
        FolderStatus::PartialError    => Color32::from_rgb(0x8C, 0x2A, 0x2A),
        FolderStatus::Error           => Color32::from_rgb(0xDC, 0x3C, 0x3C),
        FolderStatus::Done            => Color32::from_rgb(0x32, 0xB4, 0x50),
        FolderStatus::DoneWithWarning => Color32::from_rgb(0xDC, 0xC8, 0x32),
        FolderStatus::DoneWithErrors  => Color32::from_rgb(0xB4, 0x64, 0x32),
        FolderStatus::Processing      => unreachable!("check status is never Processing"),
    }
}

let bg = ui.visuals().extreme_bg_color;
let fg = ui.visuals().extreme_fg_color;
let slot_count = if row.is_referee { 35 } else { 23 };

// Allocate every slot position even when it is empty, so 01-23/35 remain
// fixed columns across rows. Empty slots reserve space but paint no cell.
for slot in 1..=slot_count {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(26.0, 26.0),
        egui::Sense::click(),
    );
    let Some(cell) = row.cell_for_slot(slot) else { continue };

    let actual_color = cell_color(cell.status, cell.check_status, bg, fg);
    ui.painter().rect_filled(rect, 3.0, actual_color);

    let font = if cell.status == FolderStatus::Processing {
        egui::FontId::new(11.0, egui::FontFamily::Name("bold_mono".into()))
    } else {
        egui::FontId::monospace(11.0)
    };
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        format!("{slot:02}"),
        font,
        label_color(actual_color), // contrast derives from the painted color
    );

    if response.clicked() {
        row.open_detail_for_slot(slot);
    }
    response.on_hover_ui(|ui| {
        ui.label(&cell.folder_name);
        // Messages are pre-sorted by severity (Fatal > Error > Warning > Info),
        // then by code for stable ordering.
        for msg in &cell.messages {
            ui.colored_label(actual_color, msg);
        }
    });
}
```

**Label contrast is derived from the cell, not the theme.** The number/kit label color is chosen by
the luminance of the cell's background color at all times: dark backgrounds get a light-grey label,
light backgrounds a dark-grey label — soft extremes rather than pure white/black, to avoid excessive
contrast. This is independent of the dark/light theme, which matters most for `Checking` cells
(inverse background flips with the theme) and keeps labels readable on every status color.

Color transitions (blue→green fade) are not built-in as with CSS; if desired, interpolate between
the old and new color over a few frames in the update loop (~10 lines).

---

## Toolbar

A single primary action:

- **Compile** — the only button in the default view. During a run it becomes **Cancel**. It is a
  **split button**: the main part runs the compile in the *current output mode*, the arrow part opens
  the mode list — **Compile** (normal, CPK), **Compile for Sider** (unpacked PES folder structure,
  Red's steps 1+2) and **Test output** (processed exports written unpacked in their original layout,
  Red's step 1) — see "Output modes and targets" in the Writer section. Picking a mode **sticks for
  the session** and relabels the main part, so the prototyping loop the Sider mode exists for
  (compile → alt-tab to PES → adjust → compile again) is one click per iteration after the first.
  The mode resets to normal on the next launch (output mode is deliberately not a persisted
  setting), and a `--gui` autorun always uses its own `--mode`, so `quick_compile.bat` cannot be
  hijacked by a leftover session mode. No separate buttons: the three are the same compile with a
  different sink, so they belong to the same control. Red's staged operations as separate runs
  ("extracted → contents", "contents → CPK") are superseded by the unified pipeline and are not
  exposed. GUI autorun (`quick_compile.bat`, see
  "CLI") presses this button programmatically once the initial check completes; a run started that
  way is indistinguishable in the grid and log from a manual click, except that
  `quick_compile_close_on_success` may close the window when it ends. The button carries the
  **destination writability badge** (PES running / elevation needed — see "Post-processing") with the
  reason and remedy in its tooltip, and its dropdown always contains **Open output folder**.
- **Status strip** — a one-line bar next to the button: current stage, elapsed time, files written,
  memory in use. The core plan's current `Progress { processed, total }` payload is insufficient;
  extend it with these metrics or add separate `StageChanged`/`MetricsUpdated` events before
  implementing this view.

---

## Testing: parity against Red (adapted from Blue)

Blue has `test_parity.py` that runs the full pipeline, compares ordinary leaf files byte-for-byte,
and compares nested CPKs by entry equivalence so container timestamp differences are accepted. Since
the Rust compiler only accepts the Studio export format, the parity test becomes a two-step flow:
**upgrade the old reference exports with the Export upgrader, compile the result, and compare the
game-facing output** (nested CPK contents, bins, packed structures) against Red's reference output.
The Rust test expands Blue's comparison into the four tiers below.

```text
cargo test -p team_compiler --test parity
```

The parity harness supplies the reference-fixture location; it is not a custom `--reference` flag
passed through to Rust's standard test harness, which does not accept that option.

The reference tree is the **extracted contents** of Red's output CPK (Blue's `Singlecpk_files`
convention) — Red itself emits `patches_output/{cpk_name}.cpk`, so the fixture is produced by a
one-time extraction step. The extracted trees are several GB of binary data and are **not committed
to git**; instead, a **hash manifest** (content hash per file, keyed by normalized relative path)
is committed as the test fixture. The test regenerates the reference tree locally by running Red +
extraction when the manifest is stale or the local tree is missing, and verifies the regenerated
tree against the committed manifest before using it. This keeps the repo small while making the
test reproducible without a pre-shared binary blob.

Intentional differences must be accounted for:

- **Auto-assigned boots/gloves IDs** may differ from the old embedded IDs — the comparison
  normalizes ID-dependent paths and embedded references.
- **Texture locations**: every player's textures are relocated to per-player common subfolders (see
  "Texture relocation to common" in the walkthrough), where Red packs team textures inside each
  model folder's structure; textures resolved from `Common/` stay in the team's Common output and
  are referenced in place. The comparison matches textures by content across locations and verifies
  that rewritten references resolve to the relocated (or in-place Common) files, and that a Common
  texture appears once in the archive however many players reference it.
- **Automatic collar reconciliation**: unlike Red's pass-through behavior, the compiler derives the
  custom collar ID from the `collar_[ID]` filename and rewrites the team's kit configs; normalized
  parity permits and verifies those expected config-field changes.
- **Dropped features** (Other folder) are excluded.
- **Kit-dependent path magic**: Red emits the legacy `u0XXXp0`/`u0XXXp1`… spelling (with the `XXX`
  placeholder or a baked-in team ID, as the author wrote it), the compiler emits `kitN`/`kit1`…
  (see "Kit-dependent assets" in the [Unified model format plan](model_format.md)). The comparison
  normalizes Red's `u0???p<d>` to `kit<d>` (`p0` → `kitN`) in file names and in decoded FMDL/MTL/XML
  paths before diffing. The compiler gets no switch to emit the legacy spelling: parity is a
  file-tree comparison, not an in-game test, and the exes can be modded to `kitN` at any time, so a
  switch would be a second output format maintained for nothing.

Comparison uses four tiers:

1. **Exact leaf bytes** for unaffected files such as textures and raw bin outputs.
2. **Decoded and normalized comparison** for FMDL, MTL, XML, kit configs, and other ID/path-bearing
   content; generated IDs and relocated paths are normalized before remaining fields are compared.
3. **Archive-entry equivalence** for CPK/FPK containers, whose container bytes may differ because of
   ordering metadata or timestamps while their normalized entries agree.
4. **An explicit intentional-difference allowlist** reviewed with the fixture, rather than broad
   exclusions that can hide regressions.

Maintain a versioned test matrix for PES 15–21 covering team and referee exports; folders, ZIP, and
7z; native and converted pre-Fox/Fox models; local, shared, and combined models; normal, multi-CPK,
test, and sider modes; savefile success/failure; and cancellation. Key behavior areas:

- **Dispositions and GUI**: pass-through and `DoneWithErrors`; an Error-level `DropFile` rendering
  red at check time and finishing `DoneWithErrors`; `DropFolder` outcomes; an effective `DropExport`
  blocking every row cell; an invalid `ParsedAestheticsExport` still producing stable GUI rows and scoped
  messages; stale validation event rejection; disabled exports producing a log entry but no row; a
  failed kit showing base-bin colors; catalog completeness (every emitted code has a template,
  context placeholders are satisfied, code+scope grouping is stable, CLI and GUI render the same
  underlying message).
- **Rosters and identity**: an export stem such as `co - Spring.zip` resolving through `/co/`; a
  folder and archive with identical display stems staying distinct (including in test-mode output
  directories); referee identity never constructing a normal `TeamId`; duplicate folder-name slots
  vs duplicate authoritative `players.txt` slots and their distinct dispositions; line-local
  malformed and out-of-range roster entries producing `DropSlot`; an empty/all-invalid referee
  roster blocked while an empty normal-team roster supports a kit-only export; a refs export without
  `players.txt` blocked by the compiler yet repairable in the Refs arranger; multiple refs exports
  (valid or not) all blocked at discovery; team-ID resolution failure; duplicate team IDs;
  multi-mapped folders (team and referee) preparing once and instantiating per slot
  atomically, with textures emitted once into the name-keyed common subfolder shared by all mapped
  slots.
- **Models and textures**: target-native plus convertible model variants; each model source owned
  exactly once; a team player with boots/settings but no face; multiple same-category shared links
  rejected; both `ingame_face` spellings normalizing identically with a local pre-Fox boots
  composite still emitted; FMDL→pre-Fox conversion emitting an atomic `.model`+`.mtl` pair; a glTF
  image shared by two model parts loaded and emitted exactly once; shared-texture dedup and
  deterministic conflict resolution under reversed rayon completion; logo production (a 1000×600
  source under each fit mode yields the three square sizes; `logo_small` replaces only the 128²;
  an undecodable `logo_small` drops all three; a 300² source is upscaled and reported); kit
  folders (`p1 - Lakers` resolving to `p1` with the label kept; `p1/` beside `p1 - Lakers/`
  discarding both; `all/kit_back.dds` emitted under every kit's ID with each config's `back` field
  set, while a kit's own `kit_back.dds` wins for that kit only; the emitted files identical to a
  compile of the same export with the shared files copied into each kit folder; `all/config.toml`
  ignored with a warning; an empty `p3/` yielding the checkerboard texture, the mask template, the
  template config and a UniColor entry carrying the magenta/black pair, with `kit_placeholder`
  and `kit_colors_missing`; the same `p3/` with a `colors.txt` carrying its colors and no
  `kit_colors_missing`; a `p4/` holding only `kit_back.dds` yielding the same plus `_back` and a
  config whose `back` field is set; a kit with a real texture and no colors deriving from the
  texture, never reaching the loud pair); kit layout (a `pre-fox` kit compiled for PES 21: every
  texel outside the sock and shorts islands byte-identical to the no-marker compile, the islands
  matching the golden produced from the hand-adjusted fixture pair, `kit_layout_converted`
  reported; the same kit compiled for PES 17 identical to the no-marker compile; a `fox` kit for
  PES 17 taking the inverse table, and pre-Fox→Fox→pre-Fox on a synthetic texture whose bands are
  flat colors round-tripping exactly; `_back`/`_leg`/`_name` untouched either way; both markers
  discarding the kit; a marker in `all/` warned and ignored; a placeholder kit with a marker
  compiling as without one); mask/srm (a kit with `kit_mask.dds` compiled for PES 21 emitting no
  `_mask` and no `_srm`, with `kit_texture_not_used`; the same kit for PES 17 emitting the mask
  as given; a kit with `kit_srm.dds` for PES 17 emitting the mask *template* and no srm; a kit
  with both emitting exactly the target's one, silently); user `face.xml` (a hand-written xml
  compiled for PES 17 emitted with the team ID substituted and its unknown `type` and extra
  attribute kept verbatim, each with its warning, and `level="1"` kept with `xml_level_lod` as
  info; the same folder with the xml removed
  producing the generated xml, proving filename typing is off when one is present; a `<model>`
  without `path` and a `./` reference to an absent file each discarding the folder; a
  `model/character/face/common/…` path passing with `xml_path_unchecked`; a folder with a `<dif>`
  in the xml and a `face_diff.xml` beside it discarded; an xml lacking `face_neck` gaining the dummy
  entry; an unlisted `.model` reported and absent from the output; the same folder compiled for
  PES 21 ignoring the xml and converting the models); collar validation (`collar_[ID]` filename parsing, a cross-team
  `collar_id_conflict` resolving by canonical export order, and a custom collar overriding the FPC
  collar value in every config).
- **Planning and writer**: cross-export duplicate paths producing the same manifest under reversed
  completion order; sideload-versus-export collision resolution; nested-root path collisions;
  partial planning continuing valid teams after refs or duplicate-team drops; a plan with no
  eligible tasks producing no output or deployment; a structural `DropFolder` still yielding a
  compilable export; a late model-task failure leaving no entries from that task while successful
  sibling tasks may still commit; a failed kit never mutating UniColor/UniformParameter; notes from a rejected export excluded from
  `teamnotes.txt`; root colors/notes/referee-marker descriptors reaching planning; scope-dependent
  `source_read_failed` and `template_override_unreadable` outcomes; pass-through refusing
  conversion/packing/logo/texture-conflict findings and
  unsafe-path/ambiguous-roster/required-metadata findings.
- **Deployment and environment**: savefile IDs preserved for players whose boots/gloves task failed
  while independent settings still apply; with either FPC marker, successful standalone assets
  overriding preset IDs, failed/dropped requested assets preserving existing IDs, and absent
  standalone assets using preset IDs (including pre-Fox face-XML-local parts); source modification after planning producing
  `source_changed_during_run` (no final output published, no installed CPK/savefile/dt00 changed);
  separate refs CPK installation; `dt00_x64.cpk` write failure; multi-CPK deployment failure; no FPC
  markers with an already-FPC savefile (configs untouched); a mixed `fpc.on`/`fpc.off` team applying
  per-player presets while every kit config gains the FPC values; an FPC team's unexported kit slots
  patched from the installed cup entries (Fox and pre-Fox), including the no-existing-entry
  `kit_config_fpc_unpatched` warning; logs/sideload/output path resolution in portable and
  user-config modes; invalid-UTF-8 and empty `notes.txt`; invalid `settings.toml` preserving models
  and savefile values; atomic `players.txt` replacement never exposing a half-written roster to the
  watcher.
- **Infrastructure**: oversized memory requests, MemoryBudget wake-up stress with multiple oversized
  waiters, and automatic worker counts on one- and two-core hosts; solid-7z reads not copying the
  archive buffer per entry; a solid 7z with unnumbered players; `CpkStem` boundary cases (empty, 29
  characters, spaces, separators, reserved device names, explicit `.cpk`, case collisions);
  deterministic converted output across fresh processes with randomized hash seeds; standalone
  kit-config binary → TOML → binary preservation of source texture-name fields.

Separate **Red parity** (normalized archive-entry equivalence is acceptable) from **Rust
reproducibility**. Explicit CPU mode is the deterministic reference: identical inputs and encoder
configuration must produce byte-identical CPK, test, and sider artifacts, including canonical
ordering and normalized timestamps. GPU mode may produce different valid encoded texture bytes
across devices/drivers, and containers containing those textures may consequently differ too.
That exception does not relax model/bin output, asset identities, collision decisions, or ordering;
GPU checks compare decoded texture semantics and quality rather than demanding CPU byte equality.
Savefiles use fresh randomized encryption salts, so compare them after decryption and semantic
normalization, or inject a deterministic RNG in tests.

---

## Development phases

**Phase 3 prerequisites:** confirm the `ExportIdentity` boundary, sanitized
validated-versus-eligible projection, and roster-entry scope/disposition semantics before
implementation begins.

- **Phase 3 — skeleton and shared export model:** implement `libs/aesthetics_export` structure parsing and
  format-level validation, the Team compiler settings/CLI surface, and reader/coordinator/writer
  scaffolding over `libs/pipeline`.
- **Phase 4 prerequisites:** finalize output enumeration/namespace allocation, including
  folder-internal deep-derived names, and freeze every model task's planned ID assignments in the
  manifest.
- **Phase 4 — processing logic:** implement deep validation, model/non-model processing (including
  cross-format conversion via the Phase 2 `model_convert`), packing, bins, and referee behavior on
  synthetic Studio-format fixtures. This phase can test its components and outputs, but its full Red
  parity gate depends on Phase 6's Export upgrader (and savefile/ID integration from Phase 5), so
  end-to-end upgraded-export parity runs only after those phases land.

This documents the dependency without changing the phase order in the core plan.

---

## First-release GPU BC7

Desktop GPU BC7 for PES 19–21 uses `block_compression`'s library backend in the first release.
Auto is the default, with CPU fallback and an explicit CPU mode for reproducible builds. The
[library crates plan](libs.md) owns backend selection, startup, batching, memory, and fallback requirements; the
reproducibility section above records the accepted texture-byte exception. Integrate this into
Phase 4's processing pipeline after the early Phase 2 GPU proof, rather than waiting for 3D preview.

## Future Features

Custom GPU kernels and browser GPU acceleration remain deferred until justified by measurements;
the shared library backend does not have to be replaced merely because it shipped first.

### 3D model preview viewport

A future enhancement to the GUI: when the user clicks a player on the progress grid, a 3D viewport
shows a preview of the model.

Scope note: this stays a quick *glance* ("is this the right model") — for actually inspecting a
player folder there is the [Player aesthetics editor](player_aesthetics_editor.md), which launches
Blender instead of rendering in-app. When this feature is built, the viewport widget goes into a
shared **`libs/model_viewport`** crate (created then, not before — until this feature lands, no
viewport code exists anywhere): the Player aesthetics editor embeds the same widget as a quick
approximate preview beside its launcher, labeled as a preview with Blender as the accurate view,
which gives the crate its second consumer per the core plan's workspace guardrails.

**Feasibility:** High. egui renders via `wgpu`, which supports custom rendering alongside the
immediate-mode UI. The selected folder's structure descriptor can lazily load and parse its model
under the normal memory budget; the resulting vertex/face data is uploaded as GPU buffers and drawn
in the egui painter's render pass.

**Architecture:**

```
User clicks a player on the grid
        │
        ▼
GUI: submit the folder descriptor + source provider to a cancellable background load
  ─ Acquire a memory permit and materialize the selected FmdlFile/ModelFile off the UI thread
  ─ Read vertices (positions, normals, UVs) from the materialized model
  ─ Read faces (index buffer)
  ─ Read bone data (for optional skeleton overlay)
  ─ Convert texture (DDS/FTEX → raw RGBA via texture2ddecoder)
        │
        ▼
GUI: render with wgpu alongside egui
  ─ Upload vertex/index buffers to GPU
  ─ Create texture from raw RGBA
  ─ Draw mesh in an egui custom painter callback
  ─ Camera: orbit/zoom/pan via egui input
  ─ Optional: skeleton wireframe overlay
```

**Implementation:**

```rust
pub struct ModelPreview {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub bones: Vec<BonePreview>,
    pub texture: Option<TexturePreview>,  // raw RGBA
}
// Loaded on a cancellable background worker under a normal memory permit,
// never on the UI thread; the result is tagged with the export revision so
// a stale preview is discarded.

// Conceptual egui_wgpu pseudocode: the exact constructor/API shape depends on
// the egui version pinned when this future feature is implemented.
let (rect, response) = ui.allocate_exact_size(
    egui::vec2(400.0, 300.0),
    egui::Sense::drag(),
);
let callback_state: Arc<dyn egui_wgpu::CallbackTrait> = Arc::new(ModelPreviewCallback {
    gpu_resources,
    camera,
});
// Wrap callback_state in the pinned version's PaintCallback helper and add it
// to the painter; CallbackTrait owns prepare/finish/render-pass integration.
ui.painter().add(make_paint_callback(rect, callback_state));
```

**Performance:** A 20K-vertex model is ~640KB of vertex data — uploaded to GPU once, rendered at
negligible cost. `wgpu` handles millions of vertices trivially. Texture conversion (one 2K DDS → raw
RGBA via `texture2ddecoder`) takes ~10ms. Only one model is previewed at a time, so memory is not a
concern.

**Limitations:**
- **Bind pose only:** The model is shown in its T-pose. PES applies bone transforms at runtime via
  the skeleton and `face_diff.bin`. Showing a posed model would require implementing the PES
  skinning pipeline. Bind pose is sufficient for "is this the right model, does it look right."
- **Not a perfect PES render:** `wgpu` rendering uses standard shading. PES uses custom shaders
  (fox3ddf_blin, fox3dfw_constant, etc.) with specific alpha/blending behavior. The preview shows
  geometry and base texture accurately, but advanced material effects (anti-blur, UV scrolling,
  transparency) won't match the in-game appearance exactly.
- **One texture at a time:** A face model may have multiple materials (face, hair, oral). The
  preview shows the base texture of the selected material, or cycles through them.
- **WASM compatibility:** `wgpu` works in browsers via WebGPU. The 3D preview is available in both
  the desktop and web builds.

**When to implement:** After the GUI is functional (post-Phase 8). The core architecture doesn't
need to change; it's a `wgpu` rendering pass alongside egui. The object model already has all the
data needed.
