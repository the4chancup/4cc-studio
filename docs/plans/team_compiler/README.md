# 4cc Studio — Team compiler plan

The Team compiler (`crates/tools/team_compiler`) is the flagship tool of the suite:
the successor to the AET Compiler (Red/Blue). It compiles aesthetics exports into CPK
archives, with compile-time model format conversion (see the
[Model conversion plan](../model_conversion/README.md)) and compile-time aesthetics (on Fox the
output CPK's database tables, on pre-Fox savefile writing;
see the [Savefile plan](../pes_savefile/README.md)). Platform context (crate structure, tool
plugin trait, GUI shell, parallelism) is in the [core plan](../core/README.md).

### Relationship to the older compilers

The older compilers are stopgaps, not templates. The original batch-file compiler
was rudimentary; **Red** is a direct translation of it into Python — feature-complete
and battle-tested, but slow at its core (it works by moving files around on
storage); **Blue** is an LLM-made prototype demonstrating that an in-memory,
parallel pipeline is possible. None of them is followed to the letter:

- **The contract is output parity with Red** (still the golden standard): given the
  same inputs, the game-facing CPK contents must match Red's — exact bytes for
  unaffected artifacts and normalized semantic parity for intentional ID/path/layout
  changes (see `testing.md` "Testing: parity against Red"). *How* that output
  is produced is free to change.
- **The implementation is built around the object model** (typed structs
  representing each element of an export — see the [Aesthetics export plan](../aesthetics_export/README.md)), not around Red's
  disk-shuffling stages or Blue's path-string processing functions. References to
  Red/Blue modules in this document identify *where the behavior to reproduce is
  specified*, not code to translate.
- **Warning/error messaging is overhauled**, not ported: the pipeline emits
  structured `PipelineEvent`s (see the core plan) carrying export/folder identity,
  and the GUI/CLI decide presentation. Red's console-text messages serve only as a
  checklist of conditions worth reporting.

---

The plan is split into parts; section headings are unchanged from the
single-file plan.

| Part | Covers |
|---|---|
| [Pipeline](pipeline.md) | Per-export pipeline walkthrough |
| [Message catalog](messages.md) | Message catalog |
| [Settings](settings.md) | Settings |
| [Blue features to port](blue_port.md) | Unimplemented Blue Features to Port |
| [GUI grid](gui.md) | GUI: the live-validating team grid; Live validation (no Check or Refresh buttons); Cell states; Toolbar |
| [Testing](testing.md) | Testing: parity against Red (adapted from Blue) |

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
│  ─ bins_update (team colors, uniparam; Fox:  │
│    player rows from resolved settings.toml)  │
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
│  ─ Overrides folder injection                │
│  ─ Duplicate invariant check                 │
│  ─ Deploy CPKs to PES folder (or output/)    │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  pes_savefile: Aesthetics patch + savefile   │
│  ─ Resolve the patch from committed outputs  │
│  ─ Write aesthetics_patch.toml beside the CPK│
│    (Fox: names only; appearance is in bins/) │
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
│   │   └── overrides.rs  #   overrides/ tree precedence
│   ├── processing/       # stages 3–4: parallel per-task work, process_task(BuildTask) → TaskBatch
│   │   ├── mod.rs        #   dispatch by task kind; memory permits; message collection
│   │   ├── model.rs      #   model folders: format selection, conversion, merging, SKL pairing
│   │   ├── texture.rs    #   DDS/FTEX conversion, WESYS zlib, relocation to common
│   │   ├── material.rs   #   MTL / face XML / materials.toml editing and checks
│   │   ├── kit.rs        #   kit folders: config generation, colors, textures, FPC patching
│   │   ├── team_assets.rs#   portraits, logos, collars, Common folder
│   │   ├── referee.rs    #   referee-specific processing (markers, per-referee common layout)
│   │   └── materialize.rs#   THE seam: relocation to game paths + Fox FPK packing (skipped in test mode)
│   ├── bins/             # UniColor / TeamColor / UniformParameter accumulation; on Fox also PlayerAppearance / BootsList / GloveList
│   │   ├── mod.rs        #   working-bin lookup via the DpFileList walk, mutation commit
│   │   └── dpfl.rs       #   DpFileList.bin read/write, slot discovery, upgrade (override)
│   ├── output/           # stages 5–6: the writer and everything after it
│   │   ├── writer.rs     #   canonical-order incremental writing, teams parts, placeholders
│   │   ├── sink.rs       #   OutputSink: CPK (normal) vs loose folder (test/sideload)
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

## Object model and export format

The export object model, its validation progression (`ParsedAestheticsExport` → `ValidatedAestheticsExport` →
`ResolvedAestheticsExport`), the `aesthetics_export` crate layout, the player-folder format, `settings.toml` in
exports, and the FPC marker files are specified in the [Aesthetics export plan](../aesthetics_export/README.md). The
compiler consumes them; only the compile-time behavior — the loaded processing types
(`FaceModelFolder` and friends, with their conversion and packing methods), the `BuildManifest`,
planning, processing, packing, bins, GUI, and CLI — lives in this crate and this plan.

---

## Development phases

- **Phase 3 — skeleton and shared export model (done 2026-10-02):** `libs/aesthetics_export`'s
  structure parsing and format-level validation (on the `ExportIdentity` boundary, the sanitized
  validated-versus-eligible projection and the roster-entry semantics of "Validation semantics"
  in the [Aesthetics export plan](../aesthetics_export/object_model.md)), the Team compiler
  settings/CLI surface, the reader/coordinator/writer over `libs/pipeline` compiling the subset
  in "Phase 3 scope" below, and the shell slice's view.
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

## Acceptance

Format and rules: `CONTRIBUTING.md` "Testing". IDs are never reused; later phases append.

**Phase 3 scope.** Discovery and export sources, `aesthetics_export`'s structure pass with its
dispositions ("Validation semantics" in the Aesthetics export plan), export identity, the settings
and CLI surface, the pipeline scaffolding compiling the tracer bullet's content (Fox player folders
with face models, and kits) into a CPK, and the shell slice. Not in Phase 3, so not here: deep
format validation, run planning (IDs, collisions, `duplicate_aesthetics_export`), other content,
bins beyond the kits' `UniformParameter.bin`, deployment, the savefile, the test and sideload modes, and live validation. In Phase 3 every
compile writes its CPK as `--no-deploy` does, deployment arriving in Phase 4, and `--mode
test|sideload`, like `multicpk_mode` on, is refused by `compile` as an invalid invocation or
configuration (exit code 2) until Phase 4 implements them, as is `upgrade-dpfl` (`check` takes no `--mode` and ignores
`multicpk_mode`).

Likewise until Phase 4, `compile` compiles only what the tracer path does, for a normal team
targeting a Fox version (18–21): roster-mapped player folders holding one or more Fox face
models, at most one per face name (`face_high`, `hair_high`, `oral`, `fcl_hair`), with their
textures (DDS converted, FTEX as it is; the other image formats are Phase 4's) and the files a Fox face folder carries beside them (`fcl_hair.skl` beside `fcl_hair.fmdl`, `face_diff.bin`,
`fcl_hair_sim.fclo`); and kits needing no layout conversion, their configs written into
a `UniformParameter.bin` built on the bundled base, which the CPK carries only when it has kits
(installed-bin lookup is Phase 4's). The test is over what would be emitted: content validation
drops or leaves unmapped, an unused `all/`, a link whose missing target `pass_through` keeps, a
`kit_mask` on a Fox target (Fox has no mask slot), a player's `settings.toml` and `fpc_off`
(settings-side only), and the root and kit metadata files and the kit icon marker do not count
(`notes.txt` and `icon_<N>` are validated and emit nothing yet; `colors.txt` waits for step 4.8, which implements its grammar
("Root files", "Colors" in `player_folders.md`); `README.txt` is ignored). Everything else the
validated export holds counts: a file a lenient setting keeps despite its finding, a portrait, an
`ingame_face` or `fpc_on` marker
(the second changes every kit config), any shared folder, logo, collar or Common file. The
validated export does not mark what a lenient setting kept, and a refusal never writes a CPK
that differs from Phase 4's. An export holding anything else, targeting PES
15–17, or a refs export is skipped with the Error `content_not_yet_compiled` naming the first
such item (context `what`: its path, the target or `refs`; for a missing injected file,
context `missing`: its path), rather than writing an incomplete CPK. The first item is found
in a fixed order: the target, `refs`, then the players in the export's order (each folder's
files before its missing files), the shared folders, the kits by slot, then the rest. That includes a face
folder missing a file Phase 4 would inject (`face_diff.bin`; with an `fcl_hair.fmdl`, also
`fcl_hair_sim.fclo` and the `fcl_hair.skl` pairing it: "Fox mode fixups" in `pipeline.md`), so
no compiled face lacks what Red's would hold. An empty kit folder, or one whose effective
textures lack `kit`, compiles as the placeholder kit (the checkerboard and the template config;
its UniColor entry waits for Phase 4's bins), which TC-SRC-01 needs. Two exports resolving to
one team are each skipped with `duplicate_aesthetics_export` (step 4.13a; until then they met
the writer's duplicate-path invariant, which stays as the backstop: `cpk_write_failed` aborts
the run naming the path, and no CPK is written). `check` runs the
structure pass on every export, including content `compile` refuses (deep format checks are
Phase 4's). The code is withdrawn when Phase 4 compiles everything.
Findings are observed
in `check`'s and `compile`'s console output, one line per finding naming its code and scope; the
compiled CPK's content is the parity test's (`testing.md`), not a scenario's. Of the events,
Phase 3 emits `ExportStarted`, `Message` and `ExportProcessed` only: `FolderStatus`, `Progress`
and `Complete` wait for their consumers, the progress grid and the run strip (Phase 8), which
is also when `Complete`'s placeholder fields are restructured ("Event system" in
`core/architecture.md`), so a cell's outcome (`DoneWithErrors`, "Cell states" in `gui.md`) is
not one of these scenarios. A team export in a scenario is a `Full` export, its name carrying
the tag, unless the scenario says `Midcup` (the Aesthetics export plan's "Coverage tag").

**Sources**

```
TC-SRC-01  GIVEN an exports root holding the same export, with a players.txt and an empty kit
           folder p2/, as a folder, a .zip and a solid .7z
           WHEN the root is checked, then each source is compiled alone
           THEN three exports are reported, each with the same findings, the .7z's roster read;
                the three compiled CPKs hold the same entries with the same contents, each with
                p2's placeholder texture and config, and the same UniColor entry for p2 (TC-BIN-02)
TC-SRC-02  GIVEN an export folder "co - Spring" and an archive "co - Spring.zip"
           WHEN the root is checked
           THEN they are two exports, and every finding names the source it is about
TC-SRC-03  GIVEN an export whose root holds NO_USE or NO_USE.txt
           WHEN the root is checked or compiled
           THEN export_disabled is reported and nothing else about that export, and it is not compiled
TC-SRC-04  GIVEN an export whose name's first word is "balls"
           WHEN the root is checked or compiled
           THEN export_balls_skipped is reported and the export is neither validated nor compiled
TC-SRC-05  GIVEN two non-disabled exports whose first word is "refs", and a third one disabled
           WHEN the root is checked
           THEN multiple_ref_exports is reported for each of the two, neither is validated, and
                the disabled one reports only export_disabled
TC-SRC-06  GIVEN a corrupt .zip beside a valid export
           WHEN the root is compiled
           THEN export_extract_failed is reported for the .zip and the valid export is compiled
TC-SRC-07  GIVEN a .zip holding two entries whose names differ only in case, or an entry "../x"
           WHEN the root is checked
           THEN export_extract_failed names the path, and the export is skipped
TC-SRC-08  GIVEN --export given twice, naming an export outside the exports root and one inside
           it, beside a third export in the root
           WHEN check runs, then compile
           THEN each reports only the two named exports, and compile compiles only those two
TC-SRC-09  GIVEN any export, one of them with its content nested one folder down
           WHEN it is checked and then compiled
           THEN every file of every source is byte-identical to before, and no file was added
```

**Structure**

```
TC-STR-01  GIVEN an export whose root holds only notes.txt and a folder wrapper/ holding Players/
           WHEN it is compiled
           THEN nested_folders_fixed is reported, it compiles as if wrapper/'s content were at the
                root, and notes.txt is kept as the export's root notes (notes_found); an export
                holding Players/Players/03 - A/ likewise reports nested_folders_fixed and compiles
                A as player 03
TC-STR-02  GIVEN an export with two nested child folders that each hold usable content
           WHEN it is checked
           THEN nested_root_ambiguous is reported and the export is skipped
TC-STR-03  GIVEN a loose root file at the same path as a file of the one nested root
           WHEN it is checked
           THEN nested_root_conflict is reported and the export is skipped
TC-STR-04  GIVEN an export with no usable content
           WHEN it is checked
           THEN export_empty is reported and the export is skipped
TC-STR-05  GIVEN a player folder with the link file Crocs.boots, or Crocs.boots.txt, and Boots/Crocs/
           WHEN it is checked
           THEN the link resolves and no finding is reported for it
TC-STR-06  GIVEN a player folder linking a shared folder that does not exist
           WHEN it is checked
           THEN link_target_missing is reported and that player folder is dropped
TC-STR-07  GIVEN a player folder with two .boots links
           WHEN it is checked
           THEN shared_link_duplicate is reported and that player folder is dropped
TC-STR-08  GIVEN a shared folder no link file names, and one linked only by a player folder that
           players.txt does not list
           WHEN it is checked
           THEN shared_folder_orphaned is reported for both, and neither is compiled
TC-STR-09  GIVEN a player folder holding a file outside the allowlist
           WHEN it is checked with strict_file_type_check on, then off
           THEN file_type_disallowed is an Error dropping the folder, then an Info keeping it
TC-STR-10  GIVEN two player folders marked with ingame_face and with ingame_face.txt
           WHEN they are checked
           THEN both markers are recognized alike and neither is reported as a disallowed file
TC-STR-11  GIVEN an ingame_face player folder that also holds face_high.fmdl, or a .face link
           WHEN it is checked
           THEN ingame_face_explicit_face_model is reported and that player folder is dropped
TC-STR-12  GIVEN a player folder holding both fpc_on and fpc_off
           WHEN it is checked
           THEN fpc_conflict is reported and that player folder is dropped
TC-STR-13  GIVEN a player folder holding hair.dds and hair.png, and another holding hair.dds and
           common/hair.dds
           WHEN they are checked
           THEN texture_stem_conflict is reported for each and both player folders are dropped
TC-STR-14  GIVEN ingame_face player folders holding, in turn, boots/hair_high.fmdl with
           gloves/glove_l.fmdl and common/skin.dds; face/hair_high.fmdl; and a model in common/
           with a file in a subfolder extra/
           WHEN they are checked
           THEN the first reports no finding (its hair_high is a boots part, by its subfolder);
                the second reports ingame_face_explicit_face_model and is dropped; the third
                reports file_type_disallowed for both files
TC-STR-15  GIVEN player folders linking Common/torso.fmdl as torso.fmdl.common and as
           torso.fmdl.common.txt, and a third linking missing.fmdl.common
           WHEN the export is checked
           THEN the first two links resolve with no finding; the third reports common_link_missing
                naming the Common path, and only that player folder is dropped
TC-STR-16  GIVEN strict_file_type_check on, Faces/Base/ holding a disallowed file, and a player
           folder linking it with Base.face
           WHEN the export is checked, then with pass_through on
           THEN file_type_disallowed drops Base and link_target_dropped drops the player folder;
                with pass_through both are kept and only file_type_disallowed is reported; with
                pass_through and Base holding hair.dds and hair.png instead, texture_stem_conflict
                and link_target_dropped still drop both
TC-STR-17  GIVEN a Fox target, Boots/Crocs/ holding torso.fmdl and Boots/Mud/ holding kit_boots.fmdl,
           each linked by a player, and a player folder holding torso.fmdl
           WHEN the export is checked
           THEN fmdl_name_invalid is reported for Crocs, which is dropped with its linking player
                (link_target_dropped); Mud, its player and the player holding torso.fmdl (face
                content by the name table) report no Warning or Error
```

**Roster**

```
TC-ROS-01  GIVEN no players.txt and player folders "03 - A" and "15 - B"
           WHEN the export is compiled
           THEN A is compiled as player 03 and B as player 15
TC-ROS-02  GIVEN no players.txt and player folders "24 - C" and "Snuffy"
           WHEN the export is checked
           THEN player_folder_number_invalid is reported for each and both are dropped
TC-ROS-03  GIVEN no players.txt and player folders "05 - A" and "05 - B"
           WHEN the export is checked
           THEN player_number_duplicate is reported for each and both are dropped
TC-ROS-04  GIVEN players.txt "03 snuffy", a folder "Snuffy" and a folder "15 - B"
           WHEN the export is compiled
           THEN Snuffy is player 03, and B reports player_unlisted and is not compiled
TC-ROS-05  GIVEN players.txt listing one folder under slots 03 and 07
           WHEN the export is compiled
           THEN that folder is compiled for both slots
TC-ROS-06  GIVEN players.txt lines "x3 A", "24 B", "05 Nobody" and "07 C", and folders A, B and C
           WHEN the export is compiled
           THEN the first three lines report players_txt_line_invalid, players_txt_slot_invalid
                and players_txt_target_missing, each naming its line; A also reports
                player_unlisted, B does not; C alone is compiled, as player 07
TC-ROS-07  GIVEN players.txt lines "03 OldName" and "03 NewName", with only a folder NewName
           WHEN the export is checked
           THEN players_txt_slot_duplicate is reported on the second line and
                players_txt_target_missing on the first, and the export is skipped
TC-ROS-08  GIVEN players.txt that is not UTF-8, and another with a BOM and CRLF line ends
           WHEN each export is checked
           THEN the first reports players_txt_invalid and is skipped; the second reads normally
TC-ROS-09  GIVEN a normal team with an empty players.txt, a player folder and a kit
           WHEN the export is compiled
           THEN the kit is compiled, and the player folder reports player_unlisted
TC-ROS-10  GIVEN a refs export with no players.txt and no refs.txt, its folders numbered
           WHEN it is checked
           THEN players_txt_missing is reported and the export is skipped
TC-ROS-11  GIVEN a refs export with refs.txt only, and another with refs.txt and players.txt
           WHEN each is checked
           THEN the first reads refs.txt as its roster; the second reports refs_txt_ignored and
                reads players.txt
TC-ROS-12  GIVEN a refs export whose players.txt maps one folder to slots 01, 20 and 35, and one
           whose every line is dropped
           WHEN each is checked
           THEN the first is valid with three slots; the second reports players_txt_invalid and
                is skipped
```

**Identity**

```
TC-ID-01   GIVEN "co Full Spring 2026.zip" and a teams list with the row 714 /co/
           WHEN it is compiled
           THEN the export is reported as /co/ (714)
TC-ID-02   GIVEN an export whose first word is not in the teams list, beside a valid export
           WHEN the root is compiled
           THEN team_name_unknown is reported, that export is skipped, the other is compiled,
                and the exit code is 1
TC-ID-03   GIVEN a refs export and a teams list with no /refs/ row
           WHEN it is checked
           THEN no team_name_unknown is reported and the export is reported as referees
TC-ID-04   GIVEN an export named "--- .zip"
           WHEN it is checked
           THEN team_name_unknown is reported with an empty name and the export is skipped
TC-ID-05   GIVEN exports "co Spring 2026" and "a Full Spring 2026"
           WHEN the root is checked, then compiled
           THEN both commands report export_tag_missing for the first, naming it; it is skipped,
                the other is compiled, and the exit code is 1
TC-ID-06   GIVEN exports "co midcup day 5", "a FULL v2" and "b Spring Full"
           WHEN the root is checked
           THEN the first two report no export_tag_missing (letter case does not matter) and
                the third does (only the second word counts)
TC-ID-07   GIVEN a refs export named "refs Spring 2026"
           WHEN it is checked
           THEN no export_tag_missing is reported
```

**Kits and root files**

```
TC-KIT-01  GIVEN kit folders "p1 - Lakers" and "g1"
           WHEN the export is compiled
           THEN they are compiled as kits p1 and g1
TC-KIT-02  GIVEN kit folders "p1" and "p1 - Lakers"
           WHEN the export is checked
           THEN kit_slot_duplicate is reported and both are dropped
TC-KIT-03  GIVEN kit folders "p10", "x1" and "home"
           WHEN the export is checked
           THEN kit_folder_invalid is reported for each and each is dropped
TC-KIT-04  GIVEN all/ holding kit_back.dds and kit_name.dds, and p2/ holding its own kit_name.dds
           WHEN the export is checked
           THEN kit_textures_inherited lists only back for p2
TC-KIT-05  GIVEN all/ holding config.toml, and in another export all/ with no kit folder
           WHEN each is checked
           THEN the first reports kit_all_file_ignored, the second kit_all_unused
TC-KIT-06  GIVEN a kit folder holding both the pre-fox and fox markers
           WHEN the export is checked
           THEN kit_layout_conflict is reported and the kit is dropped
TC-KIT-07  GIVEN a kit folder holding back.dds
           WHEN the export is checked
           THEN kit_texture_name_invalid is reported and only that file is dropped
TC-KIT-08  GIVEN a kit folder holding an icon_25 marker
           WHEN the export is checked
           THEN kit_icon_invalid is reported and the kit is kept
TC-KIT-09  GIVEN p1/ holding kit.png and kit.dds; then all/ holding kit_back.png and kit_back.dds
           beside p2/; then all/kit_name.dds and p3/kit_name.png
           WHEN each export is checked
           THEN the first reports texture_stem_conflict and drops p1; the second reports it and
                drops all/, p2 inheriting nothing; the third reports no finding at all (no
                texture_stem_conflict, no kit_textures_inherited for p3)
TC-ROOT-01 GIVEN root files extra.bin and readme.TXT, a root folder wrapper/ beside Players/, a
           file Players/players.txt, and a refs export's ref_lists.txt
           WHEN each export is checked
           THEN root_file_unexpected is reported for extra.bin, wrapper/ and Players/players.txt
                only, each ignored
TC-ROOT-02 GIVEN a root notes.txt that is not UTF-8, and pass_through on
           WHEN the export is compiled
           THEN notes_encoding_invalid is reported, the note is dropped, and the export compiles
TC-ROOT-03 GIVEN root logo_zoom.png; then logo.png and logo.dds; then logo_small.png alone; then
           logo.png and logo_small.png
           WHEN each export is checked
           THEN the first three report logo_file_invalid, logo_role_duplicate and
                logo_small_without_main, each export otherwise kept; the fourth reports none
TC-ROOT-04 GIVEN Portraits/player_24.dds
           WHEN the export is checked
           THEN portrait_name_invalid is reported and only that file is dropped
TC-ROOT-05 GIVEN the tracer bullet's export with a root notes.txt that cannot be read
           WHEN it is compiled
           THEN source_read_failed is reported for notes.txt only, the rest compiles, and the
                exit code is 1
```

**Dispositions**

```
TC-DSP-01  GIVEN an export with one valid player folder and one dropped by link_target_missing
           WHEN it is compiled
           THEN the CPK holds the valid player's content only, and the exit code is 1
TC-DSP-02  GIVEN the same export with pass_through on
           WHEN it is compiled
           THEN both players are compiled, link_target_missing is still reported as an Error,
                and the exit code is 1
TC-DSP-03  GIVEN pass_through on and exports with players_txt_slot_duplicate, players_txt_slot_invalid,
           shared_folder_orphaned, fpc_conflict, kit_layout_conflict, shared_link_duplicate,
           kit_slot_duplicate or kit_folder_invalid
           WHEN they are compiled
           THEN each finding keeps its drop: the export, the assignment, the shared folder, the
                player folder, the kit (both kits, for kit_slot_duplicate)
TC-DSP-04  GIVEN an export skipped by players_txt_slot_duplicate beside a valid export
           WHEN the root is compiled
           THEN the valid export is compiled as it is when compiled alone
```

**CLI and output**

```
TC-CLI-01  GIVEN an exports root, an existing settings file and teams list
           WHEN check runs
           THEN nothing is written to the output folder or into any export, and the settings
                file and teams list are byte-identical to before (logs may be written)
TC-CLI-02  GIVEN exports with, in turn, only Info and Warning findings, an Error finding, and none
           WHEN each is checked
           THEN the exit codes are 0, 1 and 0
TC-CLI-03  GIVEN compile --no-deploy --mode test
           WHEN it runs
           THEN it is refused as an invalid invocation naming --no-deploy and --mode as
                incompatible, with exit code 2, and nothing runs
TC-CLI-04  GIVEN compile with a positional exports root
           WHEN it runs
           THEN that root is compiled and the settings file's exports_folder_path is unchanged
TC-CLI-05  GIVEN a cpk_name that is not a valid CpkStem (empty, "con", "a/b")
           WHEN compile starts
           THEN it is refused before any export is read, naming the setting, with exit code 2
TC-CLI-06  GIVEN an output_folder_path that cannot be created or written
           WHEN compile starts
           THEN it is refused before any export is read, naming the path, with exit code 3
TC-CLI-07  GIVEN --export naming a path that does not exist
           WHEN check or compile runs
           THEN it is refused as an invalid invocation naming the path, with exit code 2
TC-CLI-08  GIVEN an absolute exports_folder_path naming a folder that does not exist
           WHEN check or compile runs
           THEN it is refused before any export is read with the message messages.md gives,
                naming the path and the settings file, no OS error text, and exit code 2
TC-CLI-09  GIVEN the default relative exports_folder_path and no exports folder beside the
           executable
           WHEN compile runs
           THEN the folder is created, no_exports_found is reported naming it, nothing is
                written to the output folder, and the exit code is 0
TC-OUT-01  GIVEN the tracer bullet's export
           WHEN compile --no-deploy runs
           THEN <output_folder_path>/<cpk_name>.cpk is written, the PES folder and the savefile
                are untouched, and the exit code is 0
TC-OUT-02  GIVEN a root whose every export is skipped
           WHEN it is compiled
           THEN no CPK is written, deployed or promoted, and a previous <cpk_name>.cpk in output/
                or in download/ is left as it was
TC-OUT-03  GIVEN a previous <cpk_name>.cpk and a run whose CPK writing fails
           WHEN it is compiled
           THEN cpk_write_failed is reported, the exit code is 3, no partial CPK remains, and the
                previous CPK is byte-identical to before
TC-OUT-04  GIVEN a previous <cpk_name>.cpk and one export whose only player folder reports
           link_target_missing, pass_through off
           WHEN it is compiled
           THEN no CPK is written, the previous CPK is byte-identical to before, and the exit
                code is 1
TC-OUT-05  GIVEN a previous <cpk_name>.cpk that another process holds open without delete sharing
           WHEN a run writes its new CPK completely and then replaces the previous one
           THEN output_commit_failed is reported, the exit code is 3, the previous CPK is
                byte-identical to before, and no partial output is left beside it
TC-OUT-06  GIVEN an export whose players.txt lists a folder holding a model in gloves/ whose name
           gives no hand, beside the tracer bullet's export, and a third export holding a kit, an
           empty players.txt and a player folder holding only such a gloves model
           WHEN the root is compiled, then checked
           THEN compile reports content_not_yet_compiled naming that gloves model for the first,
                skips it, compiles the other two, and exits with 1; check reports no
                content_not_yet_compiled
```

**Shell slice**

```
TC-GUI-01  manual  GIVEN the studio binary
                   WHEN it is launched with no arguments
                   THEN a window opens whose sidebar lists the Team compiler, and selecting it shows
                        its settings, a Compile button and an event log
TC-GUI-02  manual  GIVEN the tracer bullet's export in the exports folder
                   WHEN Compile is clicked
                   THEN the CPK is deployed as in TC-DEP-01 when the configured PES install holds
                        a DpFileList listing it, else promoted to output/ with the degrading
                        finding; the log shows the run's findings as the console prints them (the
                        export's export_identified line among them), then the run's exit code
```

**Phase 4 scope.** `compile` builds everything a validated export holds, for every target version
(PES 15-21) and for referee exports, into the normal CPK, the test tree or the sideload tree, and
deploys the CPKs into the PES install, degrading to `output/` when it cannot; `check` runs the
structure pass and the deep pass on every source kind. The run plans IDs (the 40-ID blocks;
`duplicate_aesthetics_export` is the validation pass's, `duplicate_path` the writer's), reads the
working bins from the installed CPKs and writes `UniColor.bin` and `TeamColor.bin` and, on Fox,
`UniformParameter.bin` and the `BootsList.bin` and `GloveList.bin` rows for the compiled players whose custom boots or gloves
committed, a failed output keeping the installed row, a compiled player of a `Full`
export whose folder holds no boots or no gloves losing that installed row, every other row kept,
`PlayerAppearance.bin` passing through unchanged until Phase 5
builds its rows from `settings.toml`. `content_not_yet_compiled` is withdrawn (step 4.20). What
`compile` still refuses: a model folder whose selected representation is glTF, dropped with
`model_gltf_unsupported` until Phase 7, never compiled from a native source beside it. Not in
Phase 4, so not here: the savefile step and the aesthetics patch (Phase 5; `compile` emits no
`savefile_*` or `patch_written` finding and a configured savefile is left byte-identical),
`run_pes`, `FolderStatus`/`Progress`/`Complete`, live validation and the GUI's degraded-run
actions (Phase 8), the in-game effect of a `GloveList.bin` row (FoxDen's gloves patch, worklog
"Issues") and of a sideload tree (FoxDen's LiveCPK gaps). The areas `PRT`, `MOD`, `TEX`, `CHK`,
`XML`, `CMN`, `BIN`, `PLN`, `REF` and `DEP` are new; `KIT`, `ROOT` and `OUT` continue their
numbering. Scenarios observe the console findings, the exit code, the files of the output tree or
of the CPK (read back with the `cpk` crate); byte parity with Red is the parity test's. Test
data: `/co/` is team 714, so its 40-ID block is 621-660, slot 05 is player 71405 with exclusive
boots/gloves ID 625 and the first shared ID is 644; `/egg/` is 792 (the tracer fixture); `/a/` is
702 and `/b/` 707.

**Portraits**

```
TC-PRT-01  GIVEN a /co/ export whose slot 05 folder holds portrait.dds, and Portraits/player_07.png
           WHEN it is compiled for PES 21, then for PES 18
           THEN the CPK holds common/render/symbol/player/71405.dds and 71407.dds, then
                player_71405.dds and player_71407.dds, each a DDS
TC-PRT-02  GIVEN slot 05's folder holding portrait.dds and Portraits/player_05.dds with different
           contents, and another export where the two files are byte-identical
           WHEN each is compiled
           THEN the first reports portrait_conflict and is skipped; the second compiles with one
                71405.dds and no finding
TC-PRT-03  GIVEN Portraits/player_05.dds whose side is 300 pixels
           WHEN the export is checked
           THEN texture_not_pow2 is reported and only that file is dropped
```

**Models**

```
TC-MOD-01  GIVEN a /co/ export whose slot 05 folder holds kit_boots.fmdl, glove_l.fmdl and
           glove_r.fmdl beside its face models and textures
           WHEN it is compiled for PES 21
           THEN the CPK holds Asset/model/character/boots/k0625/#Win/boots.fpk and .fpkd and
                glove/g0625/#Win/glove.fpk, the boots package holding boots.fmdl and boots.skl, the
                gloves package glove_l.fmdl and glove_r.fmdl, and every texture of the folder once
                under common/714/05 - <name>/sourceimages/
TC-MOD-02  GIVEN slot 05 holding kit_boots.fmdl and kit_boots.skl
           WHEN the export is compiled
           THEN boots.skl in the boots package is byte-identical to kit_boots.skl; without the
                .skl it is byte-identical to the bundled PES 21 body.skl
TC-MOD-03  GIVEN a players.txt mapping one folder with boots.fmdl to slots 03 and 07
           WHEN the export is compiled
           THEN k0623 and k0627 both hold the boots package, and the folder's textures appear once,
                under the folder's name
TC-MOD-04  GIVEN slot 05 holding a face model, skin.dds and kit_boots.fmdl, and linking Crocs.boots,
           whose folder also holds skin.dds: once with the same bytes, once with different ones
           WHEN each is compiled
           THEN the first packs skin once with no texture finding; the second reports
                shared_texture_conflict, keeps the face and the player's skin, and k0625 is absent
TC-MOD-05  GIVEN Boots/Crocs/boots.fmdl and Boots/Mud/boots.fmdl, Crocs linked by slots 03 and 07,
           Mud by slot 11
           WHEN the export is compiled for PES 21
           THEN k0644 holds Crocs and k0645 Mud (alphabetical), each once, and no k06NN folder
                exists for slots 03, 07 or 11
TC-MOD-06  GIVEN eighteen shared boots folders each linked by a player
           WHEN the export is checked
           THEN boots_id_pool_exhausted is reported and the export is skipped
TC-MOD-07  GIVEN Boots/Crocs/ linked by slot 05, which also holds kit_boots.fmdl
           WHEN the export is compiled for PES 21
           THEN link_combined and fmdl_merged are reported for slot 05, k0625 holds one boots.fmdl
                whose mesh count is Crocs's plus the local model's, and Crocs's own output exists
                only if another player links it plainly
TC-MOD-08  GIVEN Faces/Longhair/ holding hair_high.fmdl, linked by slot 05, which holds face_high.fmdl
           WHEN the export is compiled for PES 21
           THEN slot 05's face package holds face_high.fmdl and hair_high.fmdl, and no output
                exists for Longhair on its own
TC-MOD-09  GIVEN two parts merged into one boots.fmdl, one paired with a custom .skl and the other
           with none; and another folder whose two parts define material "skin" differently
           WHEN each export is compiled
           THEN the first reports skl_merge_conflict and the second merge_material_conflict, and
                each folder's boots are left out of the CPK, its blank face and its textures
                still packed
TC-MOD-10  GIVEN slot 05 holding torso.fmdl, legs.fmdl.common and Common/legs.fmdl
           WHEN the export is compiled for PES 21
           THEN fmdl_fcl_hair_fallback is reported for torso.fmdl, fmdl_merged for the folder, the
                face package holds one fcl_hair.fmdl whose mesh count is the sum, and no
                legs.fmdl.common or Common model path is in the CPK
TC-MOD-11  GIVEN Common/legs.fmdl referencing cloth.dds, which sits in Common/
           WHEN TC-MOD-10's export is compiled
           THEN cloth.ftex sits once at Asset/model/character/common/714/sourceimages/#windx11/
                and the merged FMDL's path table points there, not at the player's subfolder
TC-MOD-12  GIVEN slot 05 holding fcl_hair.fmdl only (no face_diff.bin, no fcl_hair_sim.fclo, no .skl)
           WHEN the export is compiled for PES 21
           THEN the face package holds fcl_hair.fmdl, face_diff.bin, fcl_hair_sim.fclo and
                fcl_hair_sim.skl, the last three byte-identical to the bundled templates
TC-MOD-13  GIVEN slot 05 holding face_high.fmdl and face_high.skl
           WHEN the export is checked
           THEN skl_no_slot is reported and the folder is kept, the .skl not emitted
TC-MOD-14  GIVEN slot 05 holding boots/hair_high.fmdl and common/skin.dds
           WHEN the export is compiled for PES 21
           THEN hair_high.fmdl is a boots part (k0625 holds it as boots.fmdl) and skin.ftex sits in
                the player's common subfolder
TC-MOD-15  GIVEN slot 05 holding face_diff.xml, and another holding a face_diff.xml whose base64
           payload is corrupt
           WHEN each export is compiled for PES 21
           THEN the first's face package holds a face_diff.bin decoded from the xml; the second
                reports face_diff_invalid and the folder is dropped
TC-MOD-16  GIVEN slot 05 holding ingame_face, torso.fmdl, torso.skl and glove_l.fmdl
           WHEN the export is compiled for PES 21
           THEN the CPK holds no face/real/71405/ path, k0625's package holds boots.fmdl and a
                boots.skl byte-identical to torso.skl, and g0625's holds glove_l.fmdl
TC-MOD-17  GIVEN slot 05 holding ingame_face, kit_boots.fmdl and Crocs.boots
           WHEN the export is compiled for PES 21
           THEN link_combined is reported, k0625 holds the merged boots, and Crocs keeps its own
                shared output only for players linking it plainly
TC-MOD-18  GIVEN slot 05 holding ingame_face and an empty face/ subfolder, with boots.fmdl
           WHEN the export is compiled
           THEN no finding is reported for face/, no face package is emitted, and k0625 holds the
                boots
TC-MOD-19  GIVEN slot 05 holding boots.fmdl only, without ingame_face; and slot 07 holding an
           empty face/ and nothing else
           WHEN the export is compiled for PES 21
           THEN a face package is emitted for 71405 and for 71407 (the blank face folder) and k0625
                holds the boots
TC-MOD-20  GIVEN a /co/ slot 05 folder holding face_high.model, face_high.mtl and skin.dds
           WHEN it is compiled for PES 17
           THEN the CPK holds common/character0/model/character/face/real/71405.cpk, whose face.xml
                lists one face_neck entry naming the emitted model, whose .mtl names skin.dds under
                model/character/uniform/common/714/05 - <name>/, and no Asset/ path exists
TC-MOD-21  GIVEN slot 05 holding face_high.model, kit_boots.model, hat_parts.model and
           arm_gloveL.model
           WHEN it is compiled for PES 17
           THEN face.xml lists four entries typed face_neck, parts, parts and gloveL, in the face
                CPK alone: no boots/ or glove/ folder is written for the player
TC-MOD-22  GIVEN slot 05 holding face_high.model, face_high.mtl and Crocs.boots, Boots/Crocs/ holding
           boots.model and boots.mtl
           WHEN it is compiled for PES 17
           THEN common/character0/model/character/boots/0644/ holds Crocs's files as loose entries
                and slot 05's face.xml does not list them
TC-MOD-23  GIVEN slot 05 holding body_uniform.model, compiled for PES 15, then PES 16, then PES 17
           WHEN each is compiled
           THEN the entry's type is uniform_sub with xml_uniform_pes15 on 15, uniform on 16 and 17;
                on 16 the emitted model name carries the oral_ prefix, and a folder whose own
                face.xml names ./body_uniform.model reports xml_oral_prefix_missing
TC-MOD-24  GIVEN slot 05 holding legs.model.common and Common/legs.model with Common/legs.mtl
           WHEN it is compiled for PES 17
           THEN face.xml points the entry at the Common path with 714 substituted, Common/ output
                holds legs.model and legs.mtl once, and the face CPK does not hold them
TC-MOD-25  GIVEN slot 05 holding face_edithair.xml, and another folder holding hat.model in
           Boots/Mud/ (a shared boots folder)
           WHEN each export is checked for PES 17
           THEN edithair_unsupported drops the first, model_name_invalid drops Mud and its
                linking player (link_target_dropped)
TC-MOD-26  GIVEN slot 05 holding boots.fmdl and boots.model
           WHEN it is compiled for PES 21, then for PES 17
           THEN the FMDL is used on 21 and the .model on 17, with no conversion finding either way
TC-MOD-27  GIVEN the tracer export (a Fox face: fcl_hair.fmdl)
           WHEN it is compiled for PES 17
           THEN the face CPK holds a .model and .mtl pair the pes_model reader accepts with the
                FMDL's mesh count, textures as DDS, and skeleton_retargeted or
                bone_folded_for_version is reported when the model's bones require it
TC-MOD-28  GIVEN slot 05 holding boots.glb and skin.png, and another folder holding boots.glb
           beside boots.fmdl
           WHEN each export is compiled for PES 21
           THEN the first reports model_gltf_unsupported naming boots.glb and the folder is
                dropped; the second compiles the FMDL with no finding; check reports nothing for
                either
TC-MOD-29  GIVEN slot 05 holding a .model whose conversion to FMDL fails (an unsupported shader
           the IR cannot carry)
           WHEN the export is compiled for PES 21
           THEN model_conversion_failed is reported naming the file and the folder is dropped;
                with pass_through on it is still dropped
TC-MOD-30  GIVEN slot 05 holding a .model with a vertex 6000 units from the origin
           WHEN the export is compiled for PES 21
           THEN vertex_too_far_from_origin is reported naming the .model and the folder dropped
TC-MOD-31  GIVEN slot 05 holding body.fmdl whose vertices carry skh_*_l and skh_*_r weights
           WHEN the export is compiled for PES 21
           THEN g0625 holds glove_l.fmdl and glove_r.fmdl, the face's merged FMDL holds the rest,
                and the three together hold exactly the source's vertex count
TC-MOD-32  GIVEN slot 05 holding boots.fmdl and a face_diff.bin that is not the bundled one,
           without ingame_face; and slot 07 holding ingame_face, boots.fmdl and face_diff.bin
           WHEN the export is compiled for PES 21
           THEN face_file_not_used names face_diff.bin on each folder, 71405's face package
                holds the bundled face_diff.bin alone, and no face package is emitted for 71407
TC-MOD-33  GIVEN slot 05 holding face_high.fmdl and skin.dds, and linking Faces/Round, whose
           folder holds hair_high.fmdl and a skin.dds of other bytes
           WHEN the export is compiled for PES 21
           THEN merged_texture_conflict is reported and slot 05's folder is dropped: no 71405
                face package and nothing at slot 05's texture folder
TC-MOD-34  GIVEN slot 05 holding boots.model and boots.mtl naming skin, and skin.dds
           WHEN the export is compiled for PES 21
           THEN k0625 holds a boots.fmdl the fmdl reader accepts with the source's mesh count,
                and skin.ftex sits in the player's common subfolder
```

**Textures**

```
TC-TEX-01  GIVEN slot 05 holding skin.png (1024x1024 with alpha) referenced by its face model
           WHEN the export is compiled for PES 21, then PES 18
           THEN the emitted skin.ftex decodes as BC7 with 11 mip levels, then as BC3 with 11
TC-TEX-02  GIVEN slot 05 holding skin.dds already BC7 with a full mip chain
           WHEN the export is compiled for PES 21, then PES 18
           THEN on 21 the FTEX's mip data converts back to the source DDS unchanged; on 18 it is BC3
TC-TEX-03  GIVEN three player folders: skin.png of 3x3 pixels, skin.png of 300x300 pixels on a
           Fox face, and skin.dds whose header is a PNG's
           WHEN the export is checked for PES 21
           THEN texture_too_small, texture_not_pow2 and texture_type_mismatch are reported, each
                dropping its folder
TC-TEX-04  GIVEN a .dds in a codec dds_convert cannot decode
           WHEN the export is compiled
           THEN texture_codec_unsupported is reported naming the file and the folder is dropped
TC-TEX-05  GIVEN slot 05's face_high.fmdl naming hair in the team's Common output, an export with
           no Common/hair.*, and a PES folder whose DpFileList lists 4cc_61_midcup.cpk before
           the CPK compiled (cpk_name 4cc_62_midcup)
           WHEN the export is compiled for PES 21 with 4cc_61_midcup.cpk holding
           Asset/model/character/common/714/sourceimages/#windx11/hair.ftex, then without it, then
           with no PES folder
           THEN the first compiles with no texture finding; the second reports fmdl_texture_not_found as
                an Error and drops the folder; the third reports it as a Warning and keeps it
TC-TEX-06  GIVEN p1/kit.png, p1/kit_back.tga and Portraits/player_05.webp
           WHEN the export is compiled for PES 21
           THEN u0714p1.ftex and u0714p1_back.ftex are emitted and 71405.dds is a DDS
TC-TEX-07  GIVEN slot 05 holding skin.png of 1002x1002 pixels
           WHEN the export is compiled for PES 17
           THEN texture_not_div4 is reported and the folder is dropped
TC-TEX-08  GIVEN dds_compression = true, then false, then auto with multicpk_mode off
           WHEN a /co/ export is compiled for PES 17 each time
           THEN every .dds entry of the first run is WESYS-wrapped and decompresses to the second
                run's bytes; the third run equals the second; a source already WESYS-wrapped is
                emitted as it is in every run; compiled for PES 21 the setting changes nothing
TC-TEX-09  GIVEN slot 05 holding a face model naming the textures hair and skin, skin.dds,
           hair.dds.common, and Common/hair.dds
           WHEN the export is compiled for PES 21
           THEN the packed model's hair path points at
                Asset/model/character/common/714/sourceimages/#windx11/, where hair.ftex sits
                once, its skin path at the player's own texture folder, and no hair.ftex and no
                link file is in the player's folder
TC-TEX-10  GIVEN slot 05's face model naming skin and skin_nrm, skin.png (1024x1024, fully
           opaque) and skin_nrm.png
           WHEN the export is compiled for PES 18
           THEN skin.ftex decodes as BC1, and skin_nrm.ftex as BC3 holding X in alpha and Y in
                green
TC-TEX-11  GIVEN slot 06 holding a face model naming hair and hair.dds.common, no Common/hair.*,
           and TC-TEX-05's PES folder
           WHEN the export is compiled for PES 21 with 4cc_61_midcup.cpk holding the texture at
           the team's Common path, then without it
           THEN the first reports no texture or link finding and the model's hair path names
                that Common path; the second reports common_link_missing and drops the folder
```

**Deep checks**

```
TC-CHK-01  GIVEN slot 05's boots.fmdl holding a vertex 6000 units from the origin
           WHEN the export is checked, then compiled with pass_through on
           THEN check reports vertex_too_far_from_origin naming the file and exits 1; compile
                leaves the folder out of the CPK and reports it again
TC-CHK-02  GIVEN a solid .7z export whose model has the same vertex
           WHEN it is checked
           THEN vertex_too_far_from_origin is reported, as for the folder export
TC-CHK-03  GIVEN slot 05 holding a settings.toml with a key of the wrong type
           WHEN the export is compiled
           THEN settings_toml_invalid is reported, the folder's models compile, and the exit code
                is 1
TC-CHK-04  GIVEN p1/kit.dds of 4096x4096 and p2/kit.dds uncompressed (RGBA8)
           WHEN the export is checked, then compiled for PES 21
           THEN kit_texture_too_big is reported for p1 and the kit dropped; p2 gets no finding
                and its u0714p2.ftex is BC7
TC-CHK-05  GIVEN a root logo.png whose bytes are not a decodable image
           WHEN the export is checked
           THEN logo_file_invalid is reported and no logo is emitted when compiled
TC-CHK-06  GIVEN slot 05's boots.fmdl holding a mesh of 21846 faces, one over the hard limit
           WHEN the export is checked and compiled, then again with pass_through on
           THEN both report fmdl_mesh_over_face_limit as an Error naming the file; without
                pass_through the folder is left out of the CPK, with it the folder is compiled
TC-CHK-07  GIVEN slot 05's boots.fmdl holding bytes that are not a model
           WHEN the export is checked, then compiled with pass_through on
           THEN model_broken is reported naming the file and the reader's error, and the folder
                is left out of the CPK
```

**Pre-Fox XML and MTL checks**

```
TC-XML-01  GIVEN slot 05 holding face_high.model, hat_parts.model and its own face.xml listing
           both, one with type "cape", a <model> attribute "glow" and level="1", and a third
           entry naming Common/legs.model, which Common/ holds
           WHEN the export is compiled for PES 17
           THEN the emitted xml keeps cape, glow and level="1" verbatim, with xml_type_unknown,
                xml_attribute_unknown (Warnings) and xml_level_lod (Info), and the Common path has
                714 substituted
TC-XML-02  GIVEN the same folder with the face.xml removed
           WHEN it is compiled for PES 17
           THEN the generated face.xml lists face_high.model as face_neck and hat_parts.model as
                parts
TC-XML-03  GIVEN a face.xml with a <model> lacking path, and another whose ./hat.model names an
           absent file
           WHEN each export is checked for PES 17
           THEN xml_model_path_missing and xml_model_not_found each drop their folder
TC-XML-04  GIVEN a face.xml carrying a <dif> beside a face_diff.xml, and another with no face_neck
           entry
           WHEN each is compiled for PES 17
           THEN the first reports xml_dif_conflict and is dropped; the second gains the dummy
                face_neck entry with xml_face_neck_added, its dummy model and mtl emitted
TC-XML-05  GIVEN a face.xml not listing hat.model, which sits in the folder
           WHEN the export is compiled for PES 17
           THEN xml_model_unlisted is reported and hat.model is not in the face CPK
TC-XML-06  GIVEN a user face.xml in a folder compiled for PES 21
           WHEN it is compiled
           THEN xml_ignored_fox is reported and the models compile by the normal route
TC-XML-07  GIVEN face.mtl whose mesh-used material names a missing texture, and another whose
           missing texture is on a material no mesh uses
           WHEN each export is checked for PES 17
           THEN mtl_texture_not_found drops the first; mtl_texture_unused_missing is an Info on
                the second and the folder is kept
```

**Kits**

```
TC-KIT-10  GIVEN p1/ holding kit.dds and kit_mask.dds, compiled for PES 21
           WHEN it is compiled
           THEN kit_texture_not_used is reported once for p1 and no u0714p1_mask or _srm entry
                exists
TC-KIT-11  GIVEN p1/colors.txt holding two valid entries, p2/ holding kit.dds and no colors.txt,
           and an empty p3/
           WHEN the export is compiled for PES 21
           THEN UniColor.bin carries p1's two colors at team 714's p1 entry, p2's entry holds the
                pair the dominant-color extraction gives for kit.dds with kit_colors_derived reported, and p3's
                the magenta/black pair with kit_colors_missing
TC-KIT-12  GIVEN p1/colors.txt holding one valid entry and one unparsable line
           WHEN the export is compiled
           THEN color_entry_invalid is reported for the line, and the kit's colors are derived from
                its texture (kit_colors_derived)
TC-KIT-13  GIVEN a p1/icon_7 marker, and p2 without an icon marker
           WHEN the export is compiled
           THEN p1's UniColor entry carries icon 7 and p2's icon 3
TC-KIT-14  GIVEN a kit whose task fails (its kit.dds holds bytes no decoder reads)
           WHEN the export is compiled
           THEN that kit has no UniColor entry change and no UniformParameter entry; the other kits'
                entries are written
TC-KIT-15  GIVEN fpc_on in slot 05's folder, p1/config.toml without the FPC values and p2/ without
           a config
           WHEN the export is compiled for PES 21
           THEN both emitted configs carry the FPC values (shirt model, shorts model, collar,
                winter collar), kit_config_fpc_adjusted is reported for p1 and
                kit_config_generated for p2, and the export is compiled (Phase 3 refused fpc_on)
TC-KIT-16  GIVEN no fpc_on anywhere and p1/config.toml carrying the FPC values
           WHEN the export is compiled
           THEN the config is emitted unchanged and no kit_config_fpc_adjusted is reported
TC-KIT-17  GIVEN p1/config.toml whose name.y is 36
           WHEN the export is checked, then compiled, for PES 18
           THEN kit_config_version_clamped is reported both times with the maximum 33, and the
                emitted config holds 33
TC-KIT-18  GIVEN p1/ holding kit.dds drawn for the pre-Fox layout and the marker pre-fox
           WHEN the export is compiled for PES 21
           THEN kit_layout_converted is reported for p1 naming pre-fox to fox, the decoded texture
                equals the no-marker compile's outside the sock rectangles, and each sock stripe's
                centre lands within 6 px (of 2048) of where the games' uniform models put it
TC-KIT-19  GIVEN the same kit compiled for PES 17, and a kit marked fox compiled for PES 17
           WHEN each is compiled
           THEN the first equals the no-marker compile; the second is re-laid with the inverse
                table and reports kit_layout_converted naming fox to pre-fox
TC-KIT-20  GIVEN an empty p3/ marked pre-fox, and all/ holding a file named fox
           WHEN the export is compiled for PES 21
           THEN p3 compiles as without a marker (the placeholder is never re-laid), and
                kit_all_file_ignored is reported for all/fox
TC-KIT-21  GIVEN p1/ holding kit.dds without kit_mask.dds
           WHEN the export is compiled for PES 17
           THEN u0714p1_mask.dds is emitted byte-identical to the bundled mask template with no
                finding
TC-KIT-22  GIVEN p1/ holding kit.dds, kit_mask.dds and kit_srm.dds
           WHEN the export is compiled for PES 17, then PES 21
           THEN on 17 the mask is emitted as given and the srm is not, on 21 the srm is emitted as
                given and the mask is not, each time with kit_texture_not_used naming the map left
                out
TC-KIT-23  GIVEN p1/ and g1/ with configs
           WHEN the export is compiled for PES 17
           THEN the CPK holds the two 120-byte configs as loose files under
                common/character0/model/character/uniform/team/714/ and no UniformParameter.bin
TC-KIT-24  GIVEN p1/config.toml whose name.y is 30
           WHEN the export is compiled for PES 18, then for PES 21
           THEN neither run reports kit_config_version_clamped and the two emitted configs hold
                the same bytes at 0x1C and 0x1D
TC-KIT-25  GIVEN Kits/all/kit.dds, and p2/ holding a config and neither kit.dds nor colors.txt
           WHEN the export is compiled for PES 21
           THEN p2's UniColor entry holds the pair the dominant-color extraction gives for
                all/kit.dds, with kit_colors_derived reported for p2
```

**Root files, Common and collars**

```
TC-ROOT-06 GIVEN a root logo.png of 1000x600 pixels
           WHEN the export is compiled for PES 21
           THEN the CPK holds e_000714_r_ll.png, e_000714_r_l.png and e_000714_r.png under
                common/render/symbol/flag/, square at 512, 256 and 128 pixels, letterboxed with
                transparent borders, and logo_fit_applied names fit
TC-ROOT-07 GIVEN logo_crop.png of 1000x600, logo_stretch.png of 1000x600 in another export, and a
           300x300 logo.png in a third
           WHEN each is compiled
           THEN the first is center-cropped and the second resampled to square, each with
                logo_fit_applied naming its mode; the third reports logo_upscaled
TC-ROOT-08 GIVEN logo.png beside logo_small.png, and an export whose logo_small.png is undecodable
           WHEN each is compiled
           THEN the first's 128 PNG comes from logo_small and the larger two from logo; the second
                emits no logo at all and reports logo_file_invalid
TC-ROOT-09 GIVEN exports /co/ with notes.txt "Hello", /a/ with notes.txt, and /b/ skipped by
           players_txt_slot_duplicate with a notes.txt
           WHEN the root is compiled
           THEN output/teamnotes.txt holds /a/'s then /co/'s notes under team headers (canonical
                order), LF line ends, and nothing of /b/; a later run with no notes removes it
TC-ROOT-10 GIVEN a root colors.txt with valid entries, and another export without one
           WHEN each is compiled
           THEN TeamColor.bin carries the first's colors at team 714's entry; the second reports
                team_colors_missing (Info) and its entry is the installed or base value
TC-ROOT-11 GIVEN /co/ with notes.txt, and output/teamnotes.txt a folder
           WHEN the root is compiled
           THEN teamnotes_write_failed is reported, the run's CPK is in place and the folder is
                left as it was
TC-CMN-01  GIVEN Collars/collar_12.fmdl and p1/config.toml
           WHEN the export is compiled for PES 21
           THEN Asset/model/character/uniform/nocloth/#Win/collar_012.fmdl is in the CPK and every
                emitted kit config's collar and winter collar read 12
TC-CMN-02  GIVEN Collars/collar_105.fmdl, Collars/neck.fmdl and Collars/collar_9999.fmdl
           WHEN the export is checked
           THEN collar_id_invalid is reported for neck.fmdl and collar_9999.fmdl,
                collar_id_conflict for collar_105.fmdl (reserved for FPC), and each file is dropped
TC-CMN-03  GIVEN exports /a/ and /co/ each holding Collars/collar_12.fmdl
           WHEN the root is compiled
           THEN /a/ keeps its collar (canonical order) and /co/ reports collar_id_conflict, its
                collar dropped and its configs not rewritten
TC-CMN-04  GIVEN slot 05's face model referencing pants_kitN, with pants_kit1.dds and pants_kit3.dds
           in the folder, and Kits/ holding p1, p2 and p3
           WHEN the export is compiled for PES 21
           THEN kit_variant_missing is reported for kit 2, pants_kit2.ftex is a copy of
                pants_kit1's conversion, and the path table keeps the literal pants_kitN
TC-CMN-05  GIVEN slot 05 holding pants_kit1.fmdl and pants_kit2.fmdl
           WHEN the export is compiled for PES 21
           THEN kit_variant_model_fox is reported and only pants_kit1 is compiled
TC-CMN-06  GIVEN slot 05's face model referencing dummy_kit and dummy_kit_srm, neither present
           WHEN the export is checked
           THEN no texture-existence finding is reported and the paths are emitted verbatim
TC-CMN-07  GIVEN slot 05 holding pants_kit1.model and pants_kit2.model with their .mtl files
           WHEN the export is compiled for PES 17
           THEN the face.xml holds one entry naming pants_kitN, with both variant files beside it
```

**Bins**

```
TC-BIN-01  GIVEN a /co/ export with kits p1 and p2, compiled with no PES install configured
           WHEN it is compiled for PES 21
           THEN the CPK holds UniColor.bin, TeamColor.bin and UniformParameter.bin built on the
                bundled bases, with only team 714's entries changed
TC-BIN-02  GIVEN TC-SRC-01's export (empty kit p2/)
           WHEN it is compiled
           THEN UniColor.bin carries p2's entry with the magenta/black pair and icon 3
TC-BIN-03  GIVEN a kit whose task fails beside one that commits
           WHEN the export is compiled
           THEN UniColor.bin and UniformParameter.bin carry the committing kit's entry only
TC-BIN-04  GIVEN a /co/ export with p1 and a root colors.txt
           WHEN it is compiled for PES 17
           THEN UniColor.bin and TeamColor.bin follow the pre-Fox layouts and no
                UniformParameter.bin is written
TC-BIN-05  GIVEN a PES folder whose download/DpFileList.bin lists 4cc_08_bins, 4cc_61_midcup and
           4cc_99_test, 4cc_08_bins.cpk holding a UniColor.bin with team 714's p1 entry set to A and
           4cc_61_midcup.cpk one with it set to B
           WHEN a Midcup /co/ export with only p2/ is compiled with cpk_name 4cc_99_test, then with cpk_name
           4cc_61_midcup
           THEN the first run's UniColor.bin carries B at p1 (the highest-priority CPK below the
                output's) and the second run's carries A (the midcup CPK and everything above it
                skipped), both with p2 set; bin_source names the supplying CPK for each bin
TC-BIN-06  GIVEN an installed UniformParameter.bin holding team 714's p1 entry, and a Midcup
           export with fpc_on and only p2/
           WHEN it is compiled for PES 21
           THEN the emitted UniformParameter.bin's p1 entry carries the FPC values with
                kit_config_fpc_adjusted reported for slot p1, and p3, which has no entry, reports
                kit_config_fpc_unpatched
TC-BIN-07  GIVEN a templates/UniColor.bin in the data directory, and a templates/TeamColor.bin
           that cannot be read
           WHEN a from-scratch compile runs
           THEN template_override_active names UniColor.bin and the bin is built on it;
                template_override_unreadable names TeamColor.bin and the run aborts with exit 3
TC-BIN-08  GIVEN an installed CPK whose UniColor.bin is WESYS-wrapped
           WHEN a compile reads it
           THEN the emitted UniColor.bin is plain and carries the installed entries
TC-BIN-09  GIVEN a PES folder with no DpFileList.bin
           WHEN a compile runs
           THEN the bins are built on the bundled bases, dpfilelist_missing is reported, and the
                CPK is promoted to output/
TC-BIN-10  GIVEN an installed BootsList.bin of ten pairs and a /co/ export whose slot 05 holds
           boots.fmdl
           WHEN it is compiled for PES 21
           THEN the CPK's BootsList.bin holds eleven pairs sorted by player id with (71405, 625)
                among them and the ten unchanged; GloveList.bin and PlayerAppearance.bin are
                byte-identical to the installed ones
TC-BIN-11  GIVEN slot 05 holding boots.fmdl whose task fails, and an installed pair (71405, 7)
           WHEN the export is compiled for PES 21
           THEN BootsList.bin keeps (71405, 7)
TC-BIN-12  GIVEN slot 05 holding glove_l.fmdl and glove_r.fmdl, and slot 07 linking Gloves/Keeper/
           WHEN the export is compiled for PES 21
           THEN GloveList.bin holds (71405, 625) and (71407, 644)
TC-BIN-13  GIVEN an installed TeamColor.bin whose record for team 799 starts with color bytes in
           place of its team ID and color count
           WHEN a /co/ export is compiled
           THEN the emitted TeamColor.bin's record for team 799 starts with team ID 799 and
                count 4, every other installed record is unchanged, and bin_header_repaired
                is reported once, naming TeamColor.bin and team 799
TC-BIN-14  GIVEN a Full /co/ export with kits p1 and p2, and no PES install configured (the
           bundled UniColor.bin holds a past cup's kits for team 714 under a larger count)
           WHEN it is compiled for PES 21
           THEN team 714's UniColor.bin record counts 2 and holds p1's and p2's entries, every
                other entry unused
TC-BIN-15  GIVEN a Midcup /co/ export with kit p1 and a kit number the bundled base's record for
           team 714 does not hold, and no PES install configured
           WHEN it is compiled for PES 21
           THEN team 714's record keeps the base's other entries, p1's is replaced, the new
                kit's is added, and the count is one more than the base's
TC-BIN-16  GIVEN an installed UniformParameter.bin holding team 714's p1, p2 and p3 configs and
           team 702's p1, and a Full /co/ export with p1 only
           WHEN it is compiled for PES 21
           THEN the emitted UniformParameter.bin holds team 714's p1 config and neither its p2
                nor its p3, and team 702's p1 unchanged
TC-BIN-17  GIVEN an installed BootsList.bin holding (71405, 7) and (71406, 9), and a /co/ export
           whose slot 05 holds boots.fmdl and whose slot 06 holds no boots
           WHEN it is compiled for PES 21 as a Full export, then as a Midcup export
           THEN the Full run's BootsList.bin holds (71405, 625) and no row for 71406, and the
                Midcup run's holds (71405, 625) and (71406, 9)
```

**Planning**

```
TC-PLN-01  GIVEN /co/ with slot 05 holding boots.fmdl and slot 23 holding glove_l.fmdl
           WHEN the export is compiled twice
           THEN both runs write k0625 and g0643 and the two CPKs are byte-identical
TC-PLN-02  GIVEN Boots/Zebra/ and Boots/Apple/, each linked by a player
           WHEN the export is compiled, then with a third folder Boots/Mango/ added
           THEN Apple is k0644 and Zebra k0645 the first time; the second time Apple k0644, Mango
                k0645, Zebra k0646
TC-PLN-03  GIVEN exports "co Full A" (a folder) and "co Full B.zip", both resolving to 714, beside a
           valid /a/ export
           WHEN the root is compiled
           THEN duplicate_aesthetics_export is reported for each of the two, neither is compiled,
                /a/ is, and the exit code is 1
TC-PLN-04  GIVEN overrides/common/etc/TeamColor.bin in the data directory, and
           overrides/Asset/model/character/boots/k0625/#Win/boots.fpk
           WHEN a /co/ export whose slot 05 has boots is compiled
           THEN overrides_active is reported, the CPK's two entries hold the override bytes,
                duplicate_path is a Warning naming each path, and the export's own boots task is
                not dropped
TC-PLN-05  GIVEN exports /a/ and /co/
           WHEN the root is compiled with thread_count 1, then with thread_count 8
           THEN the two CPKs are byte-identical
TC-PLN-06  GIVEN an export whose boots.fmdl is replaced after planning and before its task reads it
           WHEN it is compiled
           THEN source_changed_during_run is reported, the run aborts with exit 3, no CPK is
                written and a previous one is byte-identical
TC-PLN-07  GIVEN a root with /co/ skipped by duplicate_aesthetics_export and /a/ valid
           WHEN it is compiled
           THEN /a/'s CPK content equals a compile of /a/ alone
```

**Referees**

```
TC-REF-01  GIVEN a refs export whose players.txt maps "Ref A" (face_high.fmdl, boots.fmdl, skin.dds)
           to slots 01, 20 and 35
           WHEN the root is compiled for PES 21 with refs_cpk_name 4cc_18_referees
           THEN 4cc_18_referees.cpk holds the referee face package of slots 01, 20 and 35, boots
                folders k9901, k9920 and k9935, one skin.ftex under common/999/Ref A/sourceimages/,
                and the team CPK holds no 999 path
TC-REF-02  GIVEN the same refs export and a /co/ export in one root
           WHEN the root is compiled
           THEN two CPKs are written, 4cc_99_test.cpk without referee content and
                4cc_18_referees.cpk without /co/'s, and both are deployed or promoted together
TC-REF-03  GIVEN a refs export with Faces/Base/ linked by Ref A and Ref B
           WHEN it is compiled for PES 21
           THEN each referee's face package holds the merged face and Base has no output of its
                own
TC-REF-04  GIVEN a refs export with ref_marker.dds
           WHEN it is compiled for PES 17
           THEN the referee CPK holds the marker model as collar_077.model with its .mtl, the
                marker texture in the referees' Common output, and referee kit configs whose
                collar is 77
TC-REF-05  GIVEN a refs export compiled with --mode test
           WHEN it runs
           THEN output/test_output/<refs source key>/Players/Ref A/ holds the processed files once,
                not per slot
TC-REF-06  GIVEN a refs export with ref_marker.dds, and a PES folder with dt00_x64.cpk
           WHEN it is compiled for PES 21
           THEN the referee CPK holds the marker model as collar_077.fmdl, the marker's FTEX
                under common/999/sourceimages/, and referee kit configs whose collar is 77;
                dt00_x64.cpk is byte-identical and the exit code is 0
TC-REF-07  GIVEN a /co/ export holding Collars/collar_77.fmdl, p1/config.toml with collar 77 and
           p2/config.toml with winter collar 77
           WHEN it is compiled for PES 21
           THEN collar_id_conflict is reported and the file dropped, and kit_collar_reserved is
                reported for p1 and for p2, each kit dropped
TC-REF-08  GIVEN a refs export with no ref_marker.dds
           WHEN it is compiled for PES 21
           THEN the referee CPK holds no collar_077.fmdl and no marker texture, and its kit
                configs keep the template's collar
```

**Output modes, deployment, multi-CPK**

```
TC-OUT-07  GIVEN the tracer export
           WHEN compile --mode test runs
           THEN output/test_output/egg Tracer/Players/05 - The Chad Stormworks Player/ holds
                fcl_hair.fmdl with its texture path rewritten, shirt.ftex, face_diff.bin,
                fcl_hair_sim.fclo and fcl_hair_sim.skl, no .fpk exists under test_output/, the bins
                are written under test_output/_bins/ at their game-relative paths, and the PES
                folder is untouched
TC-OUT-08  GIVEN a folder "co - Spring" and an archive "co - Spring.zip" of different content
           WHEN compile --mode test runs
           THEN two test_output directories exist, named from each source's key, neither
                overwriting the other
TC-OUT-09  GIVEN a PES folder with livecpk/old.txt and the tracer export
           WHEN compile --mode sideload runs for PES 21
           THEN livecpk/ holds exactly the files a normal-mode CPK of the export holds, at the
                same relative paths with the same bytes, old.txt is gone, download/ is untouched,
                and the exit code is 0
TC-OUT-10  GIVEN pes_version 15, then 16, then 17
           WHEN compile --mode sideload runs
           THEN 15 and 16 are refused as an invalid configuration naming the version with exit
                code 2 and nothing runs; 17 writes livecpk/
TC-OUT-11  GIVEN compile --mode test with a kit and a root notes.txt
           WHEN it runs
           THEN output/teamnotes.txt is written and the kit's UniColor entry is in the UniColor.bin
                under test_output/_bins/
TC-OUT-12  GIVEN multicpk_mode on, exports /a/, /b/ and /co/, cpk_part_max_size just above the
           compiled size of /a/ and /b/ together, and a DPFL reserving 4cc_41_teams to 4cc_45_teams
           WHEN the root is compiled
           THEN 4cc_41_teams.cpk holds /a/ and /b/ whole, 4cc_42_teams.cpk holds /co/,
                4cc_43_teams.cpk to 4cc_45_teams.cpk are each the 6,272-byte placeholder, and
                4cc_08_bins.cpk holds the bins only
TC-OUT-13  GIVEN the same with cpk_part_max_size below one team's size
           WHEN the root is compiled
           THEN cpk_team_exceeds_cap is reported, the run aborts with exit 3 and no part is written
TC-OUT-14  GIVEN the same with a DPFL reserving one teams slot
           WHEN the root is compiled
           THEN cpk_slots_exhausted names the shortfall, the run aborts with exit 3 and no part is
                written
TC-OUT-15  GIVEN multicpk_mode off and cpk_part_max_size set below the single CPK's size
           WHEN the root is compiled
           THEN cpk_size_over_limit is a Warning, the CPK is written whole and the exit code is 0
TC-OUT-16  GIVEN teams_cpk_name teams2 and a DPFL reserving 4cc_51_teams2 and 4cc_41_teams
           WHEN a multi-CPK compile runs
           THEN only 4cc_51_teams2.cpk is written and 4cc_41_teams is not touched
TC-OUT-17  GIVEN overrides/common/etc/TeamColor.bin and the tracer export
           WHEN compile --mode test runs
           THEN test_output/_bins/common/etc/TeamColor.bin is the compiled one, not the
                override's bytes
TC-DEP-01  GIVEN a PES folder with PES2021.exe, download/DpFileList.bin listing 4cc_99_test and an
           old download/4cc_99_test.cpk
           WHEN the tracer is compiled for PES 21
           THEN download/4cc_99_test.cpk holds the run's CPK, no .cpk.partial remains, the marker
                file lists it, output/ holds no CPK and no .staging/, and the exit code is 0
TC-DEP-02  GIVEN the same with the old CPK held open without delete sharing
           WHEN it is compiled
           THEN old_cpk_locked is reported, output/4cc_99_test.cpk holds the run's CPK, the old
                download CPK is byte-identical, and the exit code is 1
TC-DEP-03  GIVEN pes_folder_path naming a folder that does not exist
           WHEN it is compiled
           THEN pes_folder_not_found is reported, the CPK is promoted to output/, the bins come from
                the bundled bases, and the exit code is 1
TC-DEP-04  GIVEN a PES folder whose download/ denies writes
           WHEN compile starts
           THEN deploy_target_unwritable is reported before any export is read, the run still
                compiles and promotes to output/, and the exit code is 1
TC-DEP-05  GIVEN a DPFL that lists neither cpk_name nor any bundled name for it
           WHEN it is compiled
           THEN cpk_name_unlisted is reported and the CPK is promoted
TC-DEP-06  GIVEN a PES folder holding PES2019.exe while pes_version is 21
           WHEN it is compiled
           THEN pes_version_mismatch is a Warning and deployment proceeds
TC-DEP-07  GIVEN output/.staging/<a dead run's folder> left from a killed process
           WHEN compile starts
           THEN that folder is removed and the run's own staging is created and removed as usual
TC-DEP-08  GIVEN an installed DPFL without 4cc_41_teams and multicpk_mode on
           WHEN the root is compiled
           THEN dpfilelist_outdated is reported naming upgrade-dpfl, every CPK is promoted to
                output/, and the exit code is 1
TC-DEP-09  GIVEN the same install with a 1 KiB download/4cc_40_faces.cpk
           WHEN upgrade-dpfl runs without --yes
           THEN 4cc_40_faces is printed as renamed to 4cc_41_teams, DpFileList.bin is
                byte-identical, no file is renamed, and the exit code is 0
TC-DEP-10  GIVEN the same
           WHEN upgrade-dpfl --yes runs
           THEN DpFileList.bin equals the bundled PES 21 list, DpFileList.bin.bak equals the old
                file, and 4cc_40_faces.cpk is now 4cc_41_teams.cpk with its bytes unchanged
TC-DEP-11  GIVEN multicpk_mode on and a refs export beside two teams
           WHEN the root is compiled with an old 4cc_42_teams.cpk locked
           THEN old_cpk_locked is reported, every CPK of the run (parts, placeholders, bins, refs)
                is promoted to output/ and none replaces an installed one
TC-DEP-12  GIVEN an installed DPFL holding the official list's entries plus 4cc_80_mine, and
           another holding exactly the official list's
           WHEN /co/ is compiled on each with the default cpk_name
           THEN the first reports dpfilelist_not_official naming 4cc_80_mine and upgrade-dpfl,
                download/4cc_99_test.cpk holds the run's CPK and the exit code is 0; the second
                reports neither dpfilelist_not_official nor dpfilelist_outdated
TC-DEP-13  GIVEN an installed DPFL listing 4cc_38_balls, 4cc_40_faces, 4cc_45_uniform,
           4cc_60_midcup, 4cc_61_midcup and 4cc_86_mine, each a 1 KiB file in download/
           WHEN upgrade-dpfl --yes runs
           THEN the files are named 4cc_16_balls, 4cc_41_teams, 4cc_42_teams, 4cc_61_midcup and
                4cc_62_midcup with their bytes unchanged, 4cc_86_mine.cpk is still present and
                reported as no longer loaded, and every other official entry is the 6,272-byte
                placeholder
TC-DEP-14  GIVEN an installed DPFL listing 4cc_62_midcup with no such file in download/
           WHEN /co/ is compiled with the default cpk_name
           THEN dpfilelist_cpk_missing is reported naming 4cc_62_midcup.cpk, the run's CPK is
                deployed, and the exit code is 0
```

---

## First-release GPU BC7

Desktop GPU BC7 for PES 19–21 uses `block_compression`'s library backend in the first release.
Auto is the default, with CPU fallback and an explicit CPU mode for reproducible builds. The
[library crates plan](../libs/README.md) owns backend selection, startup, batching, memory, and fallback requirements; the
the reproducibility paragraph in `testing.md` "Testing: parity against Red (adapted from Blue)" records the accepted texture-byte exception. Phase 4
builds the texture step CPU-only and Release 0.1.0 ships that way; the GPU backend is integrated
into it in Phase 16, rather than waiting for 3D preview.

## Future Features

Custom GPU kernels and browser GPU acceleration remain deferred until justified by measurements;
the shared library backend does not have to be replaced merely because it shipped first.

### 3D model preview viewport

A future enhancement to the GUI: when the user clicks a player on the progress grid, a 3D viewport
shows a preview of the model.

Scope note: this stays a quick *glance* ("is this the right model") — for actually inspecting a
player folder there is the [Player aesthetics editor](../player_aesthetics_editor.md), which launches
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
  ─ Convert texture (DDS/FTEX → raw RGBA via dds_convert)
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
RGBA via `dds_convert`) takes ~10ms. Only one model is previewed at a time, so memory is not a
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
