# 4cc Studio — Team compiler plan

The Team compiler (`crates/tools/team_compiler`) is the flagship tool of the suite:
the successor to the AET Compiler (Red/Blue). It compiles aesthetics exports into CPK
archives, with compile-time model format conversion (see the
[Model conversion plan](../model_conversion/README.md)) and compile-time savefile writing
(see the [Savefile plan](../pes_savefile/README.md)). Platform context (crate structure, tool
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

## Object model and export format

The export object model, its validation progression (`ParsedAestheticsExport` → `ValidatedAestheticsExport` →
`ResolvedAestheticsExport`), the `aesthetics_export` crate layout, the player-folder format, `settings.toml` in
exports, and the FPC marker files are specified in the [Aesthetics export plan](../aesthetics_export/README.md). The
compiler consumes them; only the compile-time behavior — the loaded processing types
(`FaceModelFolder` and friends, with their conversion and packing methods), the `BuildManifest`,
planning, processing, packing, bins, GUI, and CLI — lives in this crate and this plan.

---

## Development phases

**Phase 3 prerequisites:** the `ExportIdentity` boundary, sanitized validated-versus-eligible
projection, and roster-entry scope/disposition semantics, confirmed 2026-09-30 in the
[Aesthetics export plan](../aesthetics_export/object_model.md), "Validation semantics".

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

## Acceptance

Format and rules: `CONTRIBUTING.md` "Testing". IDs are never reused; later phases append.

**Phase 3 scope.** Discovery and export sources, `aesthetics_export`'s structure pass with its
dispositions ("Validation semantics" in the Aesthetics export plan), export identity, the settings
and CLI surface, the pipeline scaffolding compiling the tracer bullet's content (Fox player folders
with face models, and kits) into a CPK, and the shell slice. Not in Phase 3, so not here: deep
format validation, run planning (IDs, collisions, `duplicate_aesthetics_export`), other content,
bins beyond the kits' `UniformParameter.bin`, deployment, the savefile, the test and sider modes, and live validation. In Phase 3 every
compile writes its CPK as `--no-deploy` does, deployment arriving in Phase 4, and `--mode
test|sider`, like `multicpk_mode` on, is refused by `compile` as an invalid invocation or
configuration (exit code 2) until Phase 4 implements them, as is `upgrade-dpfl` (`check` takes no `--mode` and ignores
`multicpk_mode`).

Likewise until Phase 4, `compile` compiles only what the tracer path does, for a normal team
targeting a Fox version (18–21): roster-mapped player folders holding one or more Fox face
models, at most one per face name (`face_high`, `hair_high`, `oral`, `fcl_hair`), with their
textures (DDS converted, FTEX as it is; the other image formats are Phase 4's) and the files a Fox face folder carries beside them (`fcl_hair.skl` beside `fcl_hair.fmdl`, `face_diff.bin`,
`fcl_hair_sim.fclo`); and kits needing no layout conversion, their configs written into
a `UniformParameter.bin` built on the bundled base, which the CPK carries only when it has kits
(installed-bin lookup is Phase 4's). The test is over what would be emitted: content validation
drops or leaves unmapped, an unused `all/`, files a lenient setting keeps despite a finding, and
the root and kit metadata files do not count (`notes.txt` and `icon.txt` are validated and emit
nothing yet; `colors.txt` waits for its grammar, an open question for Phase 4; `README.txt` is
ignored). An export holding anything else, targeting PES
15–17, or a refs export is skipped with the Error `content_not_yet_compiled` naming the first
such item, the target or `refs`, rather than writing an incomplete CPK. That includes a face
folder missing a file Phase 4 would inject (`face_diff.bin`; with an `fcl_hair.fmdl`, also
`fcl_hair_sim.fclo` and the `fcl_hair.skl` pairing it: "Fox mode fixups" in `pipeline.md`), so
no compiled face lacks what Red's would hold. An empty kit folder, or one whose effective
textures lack `kit`, compiles as the placeholder kit (the checkerboard and the template config;
its UniColor entry waits for Phase 4's bins), which TC-SRC-01 needs. Two exports resolving to
one team, which `duplicate_aesthetics_export` reports from Phase 4's run planning, meet the
writer's duplicate-path invariant meanwhile: `cpk_write_failed` aborts the run naming the path,
and no CPK is written. `check` runs the
structure pass on every export, including content `compile` refuses (deep format checks are
Phase 4's). The code is withdrawn when Phase 4 compiles everything.
Findings are observed
in `check`'s and `compile`'s console output, one line per finding naming its code and scope; the
compiled CPK's content is the parity test's (`testing.md`), not a scenario's.

**Sources**

```
TC-SRC-01  GIVEN an exports root holding the same export, with a players.txt and an empty kit
           folder p2/, as a folder, a .zip and a solid .7z
           WHEN the root is checked, then each source is compiled alone
           THEN three exports are reported, each with the same findings, the .7z's roster read;
                the three compiled CPKs hold the same entries with the same contents, each with
                p2's placeholder texture and config
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
TC-STR-12  GIVEN a player folder holding both fpc.on and fpc.off
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
TC-ID-01   GIVEN "co - Spring 2026.zip" and a teams list with the row 701 /co/
           WHEN it is compiled
           THEN the export is reported as /co/ (701)
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
TC-KIT-08  GIVEN a kit icon.txt holding 25
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
           THEN both players are compiled, link_target_missing is still reported as an Error, the
                export finishes as done with errors, and the exit code is 1
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
TC-OUT-01  GIVEN the tracer bullet's export
           WHEN compile --no-deploy runs
           THEN <output_folder_path>/<cpk_name>.cpk is written, the PES folder and the savefile
                are untouched, and the exit code is 0
TC-OUT-02  GIVEN a root whose every export is skipped
           WHEN it is compiled
           THEN no CPK is written and a previous <cpk_name>.cpk is left as it was
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
TC-OUT-06  GIVEN an export whose players.txt lists a folder holding only boots.fmdl, beside the
           tracer bullet's export, and a third export holding a kit, an empty players.txt and a
           player folder holding only boots.fmdl
           WHEN the root is compiled, then checked
           THEN compile reports content_not_yet_compiled naming boots for the first, skips it,
                compiles the other two, and exits with 1; check reports no content_not_yet_compiled
```

**Shell slice**

```
TC-GUI-01  manual  GIVEN the studio binary
                   WHEN it is launched with no arguments
                   THEN a window opens whose sidebar lists the Team compiler, and selecting it shows
                        its settings, a Compile button and an event log
TC-GUI-02  manual  GIVEN the tracer bullet's export in the exports folder
                   WHEN Compile is clicked
                   THEN the CPK is written as in TC-OUT-01 and the log shows the run's events: the
                        export starting, its findings, and the run completing
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
