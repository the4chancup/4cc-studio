# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 3 (Team compiler skeleton) closed 2026-10-02, its cross-family reviews queued (see
"Handover"). Phases 1 and 2 done (Phase 2 closed 2026-09-30).
**Next:** Phase 4 is itemized and its Acceptance section written (step 4.1, 2026-10-03; its
cross-family review (a) is queued). Next: 4.7 (deep validation pass); 4.6 and 4.5a-c are done (4.6c moved to Phase 8's
cancellation), 4.5d waits on the base64 question. 2.5b (GPU BC7) is step 16.x (decision entries
2026-09-21 and 2026-09-28). Release target (2026-09-28): 0.1.0 after Phase 8; phase order 1–6,
8, 0.1.0, 7, 9–16 (`core/development_plan.md` "Releases"); first-class target the Fox version
the cup moves to around April 2027 ("Target versions").
**Blocked on:** nothing.

**Handover (2026-10-01).** Until the maintainer says otherwise, the session runs as a single
Claude agent with no sidekick and no reviewer of another model family. While that holds:

- **Since 2026-10-01 (maintainer):** the lead runs in Claude Code and delegates implementation
  to a same-family subagent (Opus from 3.8b, Fable from 3.9d on; 3.8a used Sonnet) under
  the lead/sidekick rules, to keep the lead's context small. That subagent is the sidekick, not
  a reviewer: the queue below still holds every cross-family checkpoint.
- **Implement directly**, but keep everything else `AGENTS.md` asks of a sidekick's work: read the
  plan section from the file first, paste the plan's code blocks, red-first tests (record each
  test's failing assertion in the commit's log line, since no report carries it), the honesty and
  design sweeps on your own diff, the gates, `mutants-diff` with every survivor triaged, one
  commit per reviewable slice (about 500 lines).
- **Cross-family reviews are queued, not skipped and not substituted.** A same-family subagent
  shares the blind spots the review exists to catch, so it is no replacement. Each checkpoint
  that `AGENTS.md` "Second opinion" would trigger goes on the queue below with the commit range
  and the plan sections it covers; the lead runs the queue when it returns. Work may continue
  past a queued review, but no phase closes with one outstanding (one exception: Phase 3
  closed with its reviews queued, by the maintainer's directive of 2026-10-02).
- **Queue:** 3.7 (b): `crates/libs/pipeline` from its first commit, against `libs/pipeline.md`
  and `core/parallelism.md` "Memory budget"; the prior 3.6 rulings are in the log, and
  `.tmp/review_brief_3_6.md` is a template for the brief. 3.8 (b): `studio_core`'s `CliError`/
  `AppPaths`/location and the `team_compiler` CLI surface, from 3.8a's commit, against
  `core/architecture.md` "Tool plugin interface", `core/distribution.md` "Data location" and
  `team_compiler/settings.md` "CLI" (one review once 3.8 is done). 3.z (b): the shell slice,
  from the 3.z commit (new `pub` `StudioApp`, `run_gui`, `ToolContext::with_events`,
  `EventLines`; three crates), against `core/gui.md` "Shell layout" and "Event wiring",
  `core/architecture.md` "Tool plugin interface" and "Event system", `core/development_plan.md`
  "Phase 3" (the shell slice bullet) and the decision entry "3.z: the shell slice's edges".
  3.9f (a): TC-CLI-08 and TC-CLI-09 (`team_compiler/README.md` "CLI and output"), against
  `team_compiler/settings.md` "Path resolution", `team_compiler/messages.md` (`no_exports_found`
  and the settings/environment paragraph) and the decision entry "a missing exports folder is
  created or refused in plain words"; the 3.9f code rides along with it. 3.y (c): the converge
  reviewer loop, one surface per crate (`aesthetics_export`, `pipeline`, `team_compiler`, the
  Phase 3 parts of `studio_core` and `studio`), against the Phase 3 plan sections and the TC
  IDs, the lead's converge record in the 3.y row and its log line; after the loop ends, the
  second whole-crate `just mutants` per crate (`AGENTS.md` "Closing a phase").
  4.0 (a)-style: the plan rewrite of the decision entries "aesthetics travel in the database
  tables" and "Fox first; game-behavior changes go through FoxDen" (2026-10-03, its commit), against
  the step 4.0 row and `.tmp/apptest/results.txt` (the in-game evidence); the surfaces are the
  plan files those entries list, `settings_toml.md` and `save_editor.md` "Stripped save" first.
  4.1 (a): the Phase 4 Acceptance section (`team_compiler/README.md`, the Phase 4 scope paragraph
  and TC-PRT/MOD/TEX/CHK/XML/KIT-10../ROOT-06../CMN/BIN/PLN/REF/OUT-07../DEP) and the decision
  entry "Phase 4 itemization rulings", against `development_plan.md` "Phase 4" and the
  `pipeline.md` walkthrough, from the 4.1 commit.
  4.3-4.4 (b): `team_compiler` from `69be25b` to `a86ade1` plus `aesthetics_export`'s
  `SharedModelFolder.path` (two crates, new `pub(crate)` shapes: `ModelFolder`, `TaskGroup`,
  `TextureHome`, `TaskFailure`), against `pipeline.md` steps 3, 6 and 7, `player_folders.md` "ID
  allocation", "Shared models" to "Merge constraint", TC-MOD-01..09, TC-PLN-01/02, and the
  decision entries of 2026-10-03 from "a player folder's tasks commit as one group" on.
  4.5c-4.6b (b): `team_compiler` from `c7086b5` to the 4.6b commit plus `aesthetics_export`'s
  `common_link_name` and `dds_convert`'s `encode_dds` (three crates, new `pub` items), against
  `player_folders.md` "Common model links and model merging", "Portraits", `model_format.md`
  "Link files", `pipeline.md` steps 5-6 and its "Common textures are one task" bullet,
  `messages.md` "Textures", `libs/dds_convert.md` "`dds_convert` API", TC-MOD-10/11,
  TC-TEX-01/02/04/06, TC-PRT-01, TC-KIT-10, and the decision entries of 2026-10-03 from "Fox's
  Common textures are one task" on.
- For the lead, on return: the review process on trial (maintainer, 3.1) runs a full sidekick
  review loop after each GPT round and calls GPT again only once that loop has ended and GPT's
  own loop has not; not yet in `AGENTS.md` (3.6: GPT 4 of 7 accepted, then sidekick S1 3 of 7,
  so both loops ended after one round each). The sidekick's context was near its limit at the
  handover, so its next brief must stand alone. Prior rulings: `.tmp/review_rulings_3_6.md`.
- Decisions the maintainer must make (new dependencies, `unsafe` outside the listed sites, game-
  or format-facing behavior the plan does not settle) are still asked, not decided.

---

## How to resume

| Thing | Where |
|---|---|
| Plan index | `docs/plans/README.md` → `docs/plans/core/README.md` |
| Development phases | `docs/plans/core/development_plan.md` "Development Plan" |
| Legacy tools (format evidence) | `docs/plans/core/README.md` "Project context" (paths are per-machine) |
| Skeleton data | `resources/skeletons/` (see its README) |
| FPC source text (for the `fpc` crate) | `resources/FPC.wikitext` |
| `.model` format as measured (for the plugin author; the crate's notes are in `libs/README.md`) | `resources/prefox_model_format.md` |
| Coding rules and verification gates | `docs/CONTRIBUTING.md` |
| Domain terms | `docs/GLOSSARY.md` |

### Current-state gotchas

Things that are true *right now* and would cost the next session time to rediscover (a broken
fixture, a flaky test, a crate that doesn't build on one platform). Remove an entry when it stops
being true. Permanent toolchain facts go in `docs/CONTRIBUTING.md` "Toolchain notes" instead.

- `just` needs PowerShell as its Windows shell (`set windows-shell` in the justfile): Git for
  Windows' `sh` is not on PATH from a plain PowerShell. Recipes still stop at the first failure.
- `cargo install just cargo-deny` is required once per machine (both take a few minutes to build);
  the pinned toolchain and the wasm32 target install themselves on the first `cargo` call.
- The lockfile holds egui 0.36.1 on purpose (0.36.2 was under a week old at bootstrap); a bare
  `cargo update` would move it. Bump deliberately.
- `dds_convert`'s CPU BC1 core (`block_compression`, PCA plus one refinement) trails DirectXTex
  by about 2.5 mean error on an 8x4 grey-ramp mip while matching it within 1.0 at the top mip;
  the encode tests compare the chain-wide mean for that reason. Revisit with representative kit
  and face textures when the Team compiler can produce them (plan: "tune against representative
  textures").
- `just mutants-diff` reuses `mutants.out/`: it wipes a whole-crate run's results. Keep the
  whole-crate run's summary aside (to `.tmp/mutants_<step>_whole/`, only `*.txt` and
  `outcomes.json`: `AGENTS.md` "An archived run keeps its summary") before the first diff run.
- `block_compression` 0.10's BC3/BC4/BC5 decoder truncates the alpha-ramp interpolation where
  DirectX rounds; `dds_convert::dds::fix_interpolated_channels` re-derives those channels. Drop
  it if a later release rounds (the exact-match test will say).

---

## Phases

Status: `todo` · `in progress` · `done` · `blocked`. A phase is done only after its last two
steps, which every phase has: **converge** (the lead's own audit first, then the cross-family
reviewer's, both against the phase's plan sections and acceptance IDs; each gap becomes a new
step above it, and the phase waits for them) and **rewrite**
(the phase's plan sections rewritten in the present tense, in place). Then collapse its step list
below to this one row, keeping its `manual: checked` lines; the step-level detail stays in git history. Tool phases (3–6, 8–15, 17, 19) also
open with an **acceptance** step: the tool plan's "Acceptance" section for that phase, written
before any code (GUI scenarios that no automated test can prove are marked `manual` and proven by
a recorded check at converge — `CONTRIBUTING.md` "Testing"). Procedure: `AGENTS.md` "Working
documents".

| Phase | Scope | Crates | Status |
|---|---|---|---|
| 1 | Workspace bootstrap + core skeleton | workspace, CI, non-GUI `studio_core`, `vtree`, `pes_version` | done |
| 2 | Library crates (standalone-verifiable) | `wezlib` `cpk` `fpk` `ftex` `dds_convert` `fmdl` `pes_model` `uniparam` `fox2` `archives` `fpc` `teams_list` `kit_config` `color_tools` `elevation` `model_convert` (native) `pes_savefile` `python_bindings` | done |
| 3 | Team compiler skeleton | `team_compiler`, `aesthetics_export`, `pipeline` | done (reviews queued) |
| 4 | Processing logic | `team_compiler` (`plan/` `processing/` `bins/` `output/`), `aesthetics_export` deep validation | todo |
| 5 | Savefile integration | `save_editor` logic, `aatf`, `team_compiler` `output/savefile.rs` | todo |
| 6 | Export upgrader | `export_upgrader` | todo |
| 7 | glTF support (runs after Release 0.1.0) | `model_convert` (glTF half) | todo |
| 8 | GUI | `studio_core` shell, `studio`, tool views, `color_tools` widget | todo |
| 0.1.0 | Release 0.1.0, after Phase 8 (`core/development_plan.md` "Release 0.1.0") | `just release`, `CHANGELOG.md`, help chapters, update check + notice, teams-list merge, maintenance mode | todo |
| 9 | Stadium compiler | `stadium_compiler` | todo |
| 10 | Music tools | `music_player`, `music_export_editor`, `music_export`, `audio_engine` | todo |
| 11 | Match tracker | `match_tracker`, `match_feed` | todo |
| 12 | Kit config editor | `kit_config_editor` | todo |
| 13 | Refs arranger | `refs_arranger` | todo |
| 14 | Balls compiler | `balls_compiler` | todo |
| 15 | Player aesthetics editor | `player_aesthetics_editor` | todo |
| 16 | Polish and distribution | — | todo |
| 17 | Team creator (post-release) | `team_creator` | todo |
| 18 | Studio Web (post-release) | — | todo |
| 19 | DB generator (scheduled by need, after 2 and 8) | `db_generator`, `pesdb`, `pes_savefile` `ops/populate.rs` | todo |

---

## Steps

`[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked / needs a decision

A step is done when its own `→ verify:` check has actually been run and passed, on top of the
gates. Every step carries one when itemized; if a step is listed without one, the agent writes it
before starting the step (a specific check, not "test it") and puts it in the step text. Mark a
done step with a one-line summary and the files or crates touched. One `[~]` per agent at a time.

### Phase 1 — Workspace bootstrap + core skeleton

Done 2026-09-13 (spec now describes what exists: `docs/plans/core/development_plan.md` "Phase 1"). Step detail
in git history up to commit `794ce61`. CI proof: green run on `a4be936`, deliberately red run on
`38c3e68` (both `gates` jobs failed at `just gates`, `deps-check` unaffected), reverted in
`794ce61`.

### Phase 2 — Library crates

Done 2026-09-30 (spec now describes what exists: `docs/plans/core/development_plan.md`
"Phase 2", `libs/README.md`, `model_conversion/README.md`, `pes_savefile/README.md`). Step
detail (2.1-2.21: each crate's build, review rounds A-C, the 2.20a-k converge with its
mutation runs, censuses and reviewer rulings) in git history up to commit `8561f0a`. Carried
forward: 2.5b GPU BC7 is step 16.x, the `dds_convert` cache bound is step 4.y, and the
phase's open questions are under "Issues".

### Phase 3 — Team compiler skeleton

Done 2026-10-02, its cross-family reviews still queued (the maintainer's directive; "Handover"
lists them, 3.7's and 3.8's among them; what they find becomes steps under this heading). Spec
now describes what exists: `docs/plans/core/development_plan.md` "Phase 3",
`team_compiler/README.md` ("Acceptance": 77 scenarios, 75 cited by tests, 2 manual),
`team_compiler/pipeline.md` "Run driver shapes (Phase 3)", `aesthetics_export/object_model.md`,
`libs/pipeline.md`. Step detail (3.1-3.9f, the 3.w `.cargo/mutants.toml` audit, the 3.y
converge with its whole-crate mutation runs, the 3.z shell slice, 3.x this rewrite) in git
history up to commit `32261dd`. Carried forward: the phase's open issues under "Issues".

TC-GUI-01 manual: checked 2026-10-02 by the maintainer on Windows, the window, the sidebar's
Team compiler entry, its Settings, Compile button and log.
TC-GUI-02 manual: checked 2026-10-02 by the maintainer on Windows, the /egg/ tracer export
compiled from the window, the log showing `egg Tracer: Info export_identified (team=/egg/,
id=792)` then `Run finished: exit code 0`. Linux: not yet checked, no machine with a display.

### Phase 4 — Processing logic

Itemized 2026-10-03 (step 4.1). Each step is one reviewable slice (about 500 lines of new
non-generated code; fixtures, tables and tests do not count); "Plan" names the section(s) the step
implements; `ae` is `aesthetics_export`, `tc` is `team_compiler`. A `(lead first: ...)` note is
plan text the lead writes before briefing the step; `(waits on the maintainer: ...)` names an input
only the maintainer has (listed again under "Phase 4 open questions"). Test data: `/co/` is team
714 (`teams_list.txt`), so its 40-ID block is 621-660, slot 05 is player 71405 with exclusive
boots/gloves ID 625 and the first shared ID is 644; `/egg/` is 792 (the tracer fixture, slot 05 =
79205 / 3745); `/a/` is 702, `/b/` is 707:

- [x] 4.0a PES12 re-study (maintainer, 2026-10-01): the stashed stub (`git stash` entry `pes12`,
  read with `git show 'stash@{0}^3:docs/plans/pes12.md'`, never popped or applied) checked against
  the code and `Tools_4cc/pes12tools` (`058398f`) and `PES-Tools`. Done 2026-10-02: the maintainer
  kept the road open for a third engine (`Engine` matched exhaustively, step 4.0c; the output sink
  over output-relative paths) and nothing PES 12-specific enters the plans (decision entry "the
  road stays open for a third engine"). Corrections for when the stub is restored: pes12tools
  reads Red-format exports (`pes12_import_team.py` walks `Faces/`, `Kit Textures/`), not Studio's;
  its main kit path is a draw-time UV remap (`runtime/kitforce.h`), the 1024x512 repaint the
  fallback; licensing is moot (its author allows free use, and the suite only reproduces results);
  its runtime and ID scheme may change as PES 12 modding progresses (CPK-like files, custom team
  IDs), so the stub's runtime, ID and bins sections are to be re-checked then
- [x] 4.0c `Engine` matched exhaustively (decision entry "the road stays open for a third
  engine"; `CONTRIBUTING.md` "Closed sets are enums"): `PesVersion::engine()` a `match`, and the
  `==`/`!=` comparisons on `Engine` in `aesthetics_export` (`validate/folders.rs`),
  `team_compiler` (`plan/mod.rs`, `plan/subset.rs`), `model_convert` (`convert.rs`) and
  `dds_convert` (`encode.rs`) rewritten as `match`es; no behavior change → verify: `just gates`,
  `mutants-diff` over the step, and a grep finding no `==`/`!=` on `Engine` left. Done: the six
  sites and `engine()` are `match`es, `engine_of_every_version` tests all seven; gates green, 77
  of 77; `mutants-diff f678d4f`: 13, 11 caught, 2 unviable; the grep finds none
- [x] 4.0b Sider feasibility study (maintainer, 2026-10-02): `c:/Data/4cc/0Tools/sider`. Done
  2026-10-02: the Team compiler's `--mode sider` becomes `--mode sideload` and Red's `sideload/`
  folder `overrides/`, with a plain **Launch PES** button beside Compile in sideload mode (decision
  entry "sideload mode and the overrides folder"). The from-scratch LiveCPK replacement the
  maintainer asked for is superseded by 4.0e: FoxDen already serves it
- [x] 4.0e Sideloading through FoxDen (maintainer, 2026-10-03; study notes outside git,
  `.tmp/sider_info/`): no Studio sideloader; sideload mode writes `{pes_folder_path}\livecpk\`,
  served by FoxDen on 18–21 and the user's Sider 3 on 17, refused on 15/16; `sideload_output_path`
  dropped; the untracked `docs/plans/sideloader.md` stub deleted; `fox_hook` → FoxDen in the
  Refs arranger plan; the PES 2020 history recorded (decision entry "sideloading through FoxDen")
  → verify: no `sideload_output`, `Sider's LiveCPK` or `fox_hook` (outside the "formerly" note)
  left in `docs/plans`, `GLOSSARY.md` or `crates/`; `cargo test -p team_compiler`. Done: the grep
  finds only the "formerly" note; `cli.rs` `Mode::Sideload` doc comment; 171 tests, fmt, clippy
  clean; acceptance 77 of 77. FoxDen gaps forwarded to its maintainer: under "Issues"
- [x] 4.0d The 4.0b renames in the code: `team_compiler` `Mode::Sider` → `Mode::Sideload`
  (`--mode sideload`), its help topic and tests → verify: `just gates`; the refusal test asserts
  `--mode sideload`; no `sider` left in `crates/tools/team_compiler`. Done: `cli.rs` `Mode::Sideload`,
  `tests/cli/preflight.rs`, `help/01_command_line.md`; gates green, 77 of 77; `mutants-diff
  b8e7393`: no mutants (a rename)
- [x] 4.0 **GATE, maintainer only: in-game appearance-fallback test.** No agent itemizes Phase 4,
  writes a Phase 4/5/6 Acceptance section, or starts Phase 4/5/6 work until the maintainer has
  run the test and reported the result. The idea under test: a savefile player whose appearance
  record's player ID is -1 (`0xFFFFFFFF`) takes his appearance from the database table
  `PlayerAppearance.bin` instead, and his boots and gloves from `BootsList.bin`/`GloveList.bin`
  (player ID, item ID pairs next to the boots/glove models). If that holds, the compiler writes
  those three tables into its CPK the way it writes `UniColor.bin`/`TeamColor.bin`, and the
  savefile patch shrinks to names and the stats half. What the result decides: Phase 4 `plan/`
  (boots/gloves assignment output) and `bins/` (which tables are accumulated; `libs/pesdb`
  possibly moving up from Phase 19), Phase 5 savefile writing and the patch format, what an
  absent `settings.toml` key means, the Save editor's appearance, transplant and diff features,
  and Phase 6's "`settings.toml` from savefile aesthetics". Test 1 (PES 2021, team `/a/`) files
  are prepared, outside git, in `.tmp/apptest/out/` with install, undo and reading instructions
  in its `manifest.txt` and a step-by-step for the maintainer in `GUIDE.txt`. Follow-ups (field meaning in `Player.bin`, whether an in-game edit
  writes the record back, other PES versions: 2017 tables in `E:\PES2017\Data\dt10_win_files\
  common\etc\pesdb`) are planned from Test 1's result → done when: the result is recorded in the
  log and in a decision entry, and the plan changes it implies are written. Done 2026-10-03, seven
  runs on PES 2021 (the 2023 Winter Cup save and DLC; run records, scripts and every finding in
  `.tmp/apptest/results.txt`, outside git): with the save's appearance id at -1 the game takes
  the player's appearance from `PlayerAppearance.bin` (per player: Look A and Look B both showed),
  his boots from `BootsList.bin`, and with the stock exe his gloves from `GloveList.bin`, without
  needing a goalkeeper position in `Player.bin`; a kept id wins (the save's look and boots). The
  4cc gloves patch (patched exe and FoxDen) overwrites the table's gloves with the save record's
  field, so gloves need a FoxDen change ("Issues"). Any Edit-mode change to a stripped player
  restores his id: a look edit freezes the shown database look into the save (boots 0), a rename
  brings back his old save look; untouched players stay at -1. Plan changes: decision entry
  "aesthetics travel in the database tables"

- [x] 4.1 **Acceptance step** (lead): the Phase 4 scope paragraph and 126 scenarios written into
  `team_compiler/README.md` "Acceptance" (new areas `PRT`, `MOD`, `TEX`, `CHK`, `XML`, `CMN`,
  `BIN`, `PLN`, `REF`, `DEP`; `KIT`, `ROOT`, `OUT` continued), with the Phase 3 edits of TC-OUT-02,
  TC-GUI-02 and TC-SRC-01; the catalog rows `model_gltf_unsupported` and `bin_source`
  (`messages.md`); the itemization rulings written into the plans (decision entry "Phase 4
  itemization rulings"); `GLOSSARY.md` lines for teams part, placeholder CPK and output sink.
  Plan: `AGENTS.md` "Working documents" (before a tool phase). Crates: none → verify: `just
  acceptance` (report mode) lists every new ID as unproven and reports no definition error;
  `rg -c "^TC-" docs/plans/team_compiler/README.md` equals 203 (77 plus 126). Done 2026-10-03:
  `just acceptance` 203 scenarios, 77 proven, the 126 new ones unproven, no definition error;
  drafted and written by Fable from the lead's rulings (`.tmp/rulings_4_1.md`)

- [x] 4.2 **Portraits (DDS, Fox targets)**: `TaskKind::Portrait`; a player folder's
  `portrait.dds` (every slot it is mapped to) and `Portraits/player_NN.dds` emitted byte for byte
  as `common/render/symbol/player/{id}{NN}.dds` (PES 19-21) or `player_{id}{NN}.dds` (PES 18);
  still refused by the Phase 3 gate: a non-DDS portrait (encoding is 4.6's: BC3 with a full mip
  chain, `player_folders.md` "Portraits") and a slot with both sources (`portrait_conflict` is
  4.7's deep pass). The tracer's Studio fixture gains its `portrait.dds`. Plan: `pipeline.md` "2.
  Per-export serial steps" step 4, "Game paths reference" (Portraits). IDs: none cited yet
  (TC-PRT-01 and 03 complete at 4.6, TC-PRT-02 at 4.7). Crates: tc (`plan/`, `processing/`,
  `paths.rs`) → verify: the tracer parity test's `79205.dds` row is `Exact` and passes; compile
  tests show `71405.dds`/`71407.dds` on PES 21 and `player_71405.dds` on PES 18 byte-identical to
  their sources, a two-slot mapping emitting both, and the two refusals. Done 2026-10-03 (Fable,
  landed first time, 0 contradictions): parity's `79205.dds` row `Exact` and passing; four compile
  tests and four unit tests, red runs in the report (old gate `content_not_yet_compiled
  (what=Players/05 - A/portrait.dds)`; perturbations of `is_dds`, `folder_holds_portrait`, the
  sort, the version and the parity bytes each failed); gates green (50 runs, acceptance 77 of
  203); `mutants-diff 69be25b`: 26, 21 caught, 5 unviable, 0 missed. Lead fix: the archive
  fixtures' README (they keep the eight pre-portrait files)

- [x] 4.3 **Planned IDs and player-exclusive boots/gloves (Fox)**: `plan/ids.rs` `PlannedModelIds`
  (block start `101 + (team_id - 701) * 40`, exclusive `block_start + NN - 1`, boots and gloves
  namespaces independent; frozen in the manifest, processing never allocates: the Phase 4 entry
  gate); `TaskKind::Boots`/`Gloves` for a folder's `*_boots`, `*_glove_l`/`gloveL`, `*_glove_r`/
  `gloveR`, `handL`/`handR` models (the suffix table), prefix stripped (`kit_boots.fmdl` →
  `boots.fmdl`); `boots.skl` injected from the PES 21 `body.skl` unless a same-basename `.skl`
  pairs the model; packed into `Asset/model/character/boots/k{id}/#Win/boots.fpk` + `.fpkd`,
  `glove/g{id}/#Win/glove.fpk`; the folder's textures relocated once to its common subfolder and
  committed after its face/boots/gloves tasks (`shared_texture_conflict`, canonical winner face >
  boots > gloves). The tracer's boots and gloves join the Studio fixture; the parity test's ID
  normalization (k2180 → k3745) lands. Plan: `pipeline.md` "3. Per-model-folder parallel steps"
  steps 2, 3, 6, 7; `player_folders.md` "Assigns IDs automatically", "Model names: a free part plus
  a suffix", "SKL pairing"; `development_plan.md` "Phase 4" entry gates. IDs: TC-MOD-01..04,
  TC-PLN-01. Crates: tc (`plan/ids.rs`, `plan/manifest.rs`, `processing/model.rs`) → verify: the
  tracer parity test passes over all 17 `red/` files (IDs normalized, textures matched by
  content); a `plan/ids.rs` unit test gives `/co/` slot 05 boots 625 and gloves 625, slot 23 643.
  Done 2026-10-03 (Fable; two rework rounds, two sidekick contradictions accepted): one
  `TaskKind::Models` per package plus a folder's `Textures` task, the writer committing a player
  folder as one group sharing one memory permit (decision entry "a player folder's tasks commit as
  one group"); `ModelSuffix`/`model_suffix` public in `aesthetics_export`, one matcher for every
  model; parity passes over the 17 `red/` files. TC-MOD-01/02/03, TC-PLN-01 cited; TC-MOD-04's
  conflict half waits for 4.5's reserved subfolders. Red runs in the reports (old gate:
  `content_not_yet_compiled (what=.../boots.fmdl)`; group commit: `nothing of slot 05 is in the
  CPK`; shared permit: `Arc::ptr_eq` failed with the guard at `true`). Gates green (acceptance
  81 of 203); `mutants-diff 99b7059`: 80 caught, 15 unviable, 1 timeout (an infinite loop), 1
  missed (`compile.rs:226` group guard), now killed by the shared-permit test

- [x] 4.4 **Shared folders and links (Fox)**: shared `Boots/`/`Gloves/` folders take the 17 shared
  IDs (644 upward for `/co/`) in alphabetical folder-name order, `boots_id_pool_exhausted`/
  `gloves_id_pool_exhausted`; a link alone makes the player wear the shared ID (no per-player
  output); a link plus local parts merges the shared model into the player-exclusive folder
  (`link_combined`, `fmdl::ops::merge`, alphabetical source order, `fmdl_merged`), the shared
  folder untouched; a shared folder every linker combines emits nothing and takes no ID; a shared
  face link is the player's face FMDL (merge of one); `merge_material_conflict`,
  `skl_merge_conflict` (content hash). Plan: `player_folders.md` "Shared models", "A link plus
  local models combines", "Merge constraint"; `pipeline.md` step 3. IDs: TC-MOD-05..09, TC-PLN-02.
  Crates: tc (`plan/ids.rs`, `processing/model.rs`) → verify: a `/co/` export with `Boots/Crocs/`
  linked plainly by slots 03 and 07 and combined by slot 05 (own `kit_boots.fmdl`): the CPK holds
  `boots/k0644/` once, `boots/k0625/` whose FMDL's mesh count equals Crocs's plus the local
  model's, and no `k0623`/`k0627`. Sliced: (a) shared IDs, plain-linked shared output with its
  textures in its own `k{id}/#windx11/` (measured: Red's boots FMDL paths
  `/Assets/pes16/model/character/boots/k{id}/`; 306 of 400 VGL26 gloves FMDLs name
  `…/glove/g{id}/`), pool exhaustion in the structure pass (TC-MOD-05/06, TC-PLN-02);
  (b) combining, merges and their conflicts, shared face links (TC-MOD-07/08/09).
  (a) done 2026-10-03 (Fable, first time; contradictions accepted: the `Faces/` file refusal is
  unreachable, validation dropping every shared folder no mapped player links; lead fix:
  `SharedModelFolder.path`): `PlannedModelIds::shared`, `ids::shared_folders_taking_ids`
  (case-folded order), `paths::TextureHome`, `plan::ModelFolder` for player and shared folders
  alike, `structure::pool_messages`; decision entry "shared boots/gloves outputs". TC-MOD-05/06,
  TC-PLN-02 cited. Gates green (84 of 203); `mutants-diff 0ce26fd`: 40, 34 caught, 6 unviable,
  0 missed. (b) split into b1 (boots/gloves) and b2 (faces, texture byte conflicts). (b1) done
  2026-10-03 (Fable, first time; contradictions accepted: a stem two *combined* folders hold
  collides too, refused for any two of the player's sources; a plainly linked shared folder's
  parts merge as well. Lead fix: stems compared case-folded): `ModelFolder.combined`/`sources()`,
  `subset::link_combines` shared by gate, IDs and planning, `processing::TaskFailure` carrying
  `merge_material_conflict`/`skl_merge_conflict`, `FolderModels.boots_stems`. TC-MOD-07/09 cited.
  Gates green (86 of 203); `mutants-diff 033eb1c`: 43, 36 caught, 7 unviable, 0 missed. Not
  verified: the game loading a merged FMDL (`Model::to_file` re-lays it out). (b2) done
  2026-10-03 (Fable, first time, no contradiction; decision entry "texture conflicts by source
  package"): face links combine, face parts merge (`fcl_hair_sim.skl` by the boots rule),
  `ModelFolder::roles()` with the player's own `face_diff.bin`/`.fclo` winning,
  `processing/texture.rs` resolving a stem per source package (`shared_texture_conflict` drops the
  lower package via `TaskBatch.skipped`, `merged_texture_conflict` fails the textures task).
  TC-MOD-08 cited; TC-MOD-04's halves tested uncited (its GIVEN needs 4.5's subfolders). Gates
  green (87 of 203); `mutants-diff 304a0b2`: 34, 28 caught, 6 unviable, 0 missed. Texture-name
  case questions in "Issues"

- [ ] 4.5 **Face assembly (Fox)**: unsuffixed models routed to the `fcl_hair` merge (`torso.fmdl` +
  `legs.fmdl.common` → `fcl_hair.fmdl`; `fmdl_fcl_hair_fallback` per routed file; same-name
  merges themselves landed at 4.4b, `processing/model.rs`), `.common` model
  links baked in with their textures referenced in place in the team's Common output
  (`Asset/model/character/common/{team_id}/sourceimages/#windx11/`), `merged_texture_conflict`;
  reserved subfolders `face/`/`boots/`/`gloves/`/`common/` force the category; a custom `.skl`
  paired with the `fcl_hair` part replaces the injected `fcl_hair_sim.skl`, `skl_no_slot` for
  `face_high`/`hair_high`/`oral`; `face_diff.bin` and `fcl_hair_sim.fclo` injected from templates
  (lead first: the two templates as lead-authored fixtures with a provenance README,
  `resources/bins/README.md` the model; the tracer's copies are byte-identical to Red's), so a
  face folder lacking them compiles, which Phase 3 refused; `face_diff.xml` → `face_diff.bin`
  (`face_diff_invalid`). Plan: `pipeline.md` step 3 "Fox mode fixups", step 6;
  `player_folders.md` "Common model links and model merging", "Reserved subfolders", "What the
  injected files are"; `model_format.md` "Link files". IDs: TC-MOD-10..15. Crates: tc
  (`processing/model.rs`, `templates.rs`), resources → verify: a `/co/` player with `torso.fmdl`,
  `legs.fmdl.common` and `Common/legs.fmdl` referencing `Common/cloth.dds`: the face package holds
  one `fcl_hair.fmdl` whose mesh count is the sum, `cloth.ftex` sits once under
  `common/714/sourceimages/#windx11/` and is absent from the player's common subfolder, and the
  console shows `fmdl_fcl_hair_fallback` then `fmdl_merged`. Sliced: (a) templates, fallback,
  `skl_no_slot`; (b) reserved subfolders; (c) `.common` model links; (d) `face_diff.xml` (waits on
  the base64 question under "Phase 4 open questions"). (a) done
  2026-10-03 (Fable, first time; lead first: `resources/templates/` from Red with a provenance
  README): `templates::{FACE_DIFF, FCL_HAIR_SIM_FCLO, BODY_SKELETON}` injected per Face package,
  unsuffixed models are `fcl_hair` parts, `PlayerFile::SlotlessSkeleton`,
  `structure::model_name_messages` (findings on the folder holding the file, a shared face's
  included); the gate's `missing` checks gone. TC-MOD-12/13 cited. Gates green (89 of 203);
  `mutants-diff b8e6aac`: 38, 33 caught, 5 unviable, 0 missed. Found: the tracer's
  `fcl_hair.skl` is PES 21's `body.skl`, so two older processing tests' "custom" skeleton equals
  the template (fix in 4.5b's brief). (b) done 2026-10-03 (Fable, first time; no contradiction
  of substance): `subset::position`/`model_role` force a reserved subfolder's category, skeletons
  pair by path stem (same directory), the two weak skeleton tests now use PES 19's `body.skl`.
  TC-MOD-14 cited, TC-MOD-04 re-scoped and cited (decision entry). Gates green (91 of 203);
  `mutants-diff ffd4734`: 40, 34 caught, 6 unviable, 0 missed. For converge: the four reserved
  names are matched in both `aesthetics_export` (`validate::folders::position`, `pub(crate)`) and
  `team_compiler` (`subset::position`); one exported matcher would remove the copy. (c) done
  2026-10-03 (Fable; one contradiction accepted: with paths rewritten per part, TC-MOD-09's two
  `shirt` materials differing only by source directory became one, so its fixture now differs
  by shader): `PlayerFile::CommonModel`, `plan::CommonModel` resolved at planning, the Common
  `.skl` travelling with a slotted link, texture paths rewritten per part before the merge, the
  Common textures task (`TaskKind::CommonTextures`, decision entry); lead first: `aesthetics_export`
  exports `common_link_name`. TC-MOD-10/11 cited. Gates green (93 of 203); `mutants-diff
  c7086b5`: 86, 72 caught, 13 unviable, 1 missed (`model.rs` `CommonModel` package guard), given a
  test in one rework round (red shown). For converge: `subset::is_direct_common_file` copies
  validation's private check

- [x] 4.6 **Textures, all formats (Fox)**: every accepted image format (`dds_convert::decode`/
  `convert`) for player, shared, Common, kit and portrait textures; BC7 kept and encoded on PES
  19-21, BC7 transcoded to BC3 and raster encoded to BC3/BC1 on PES 18; missing mips generated;
  FTEX headers per version; `texture_too_small`, `texture_not_pow2`, `texture_type_mismatch`,
  `texture_codec_unsupported`, `fmdl_no_texture_ids`; `kit_texture_not_used` for a `kit_mask` on a
  Fox target (Phase 3's silent drop was a stopgap, not a decision); the `Converter` cache engaged
  for runs of at most two teams; conversion in bounded batches on a cancel token
  (`MemoryBudget::cancel`). Plan: `pipeline.md` step 5; `libs/dds_convert.md` "In-memory
  conversion cache" (engagement); `messages.md` "Textures"; `development_plan.md` "Phase 4"
  (CPU-only, GPU is 16.x). IDs: TC-TEX-01, TC-TEX-02, TC-TEX-04, TC-TEX-06, TC-KIT-10,
  TC-PRT-01 (non-DDS portraits encoded here); TC-TEX-03, TC-TEX-05 and TC-PRT-03 say "checked",
  so they are 4.7's: the deep pass parses textures and FMDLs (`pipeline.md` "2. Per-export serial
  steps" step 2), while this step raises the texture codes at conversion, failing the task.
  Sliced: (a) conversion core: `dds_convert` for every model-folder, Common and kit texture, the
  codec by version, the role by stem (`_nrm`, decision entry), the run's `Converter` and its
  team-count policy, the gate opened to every accepted format (TC-TEX-01, TC-TEX-02, the kit half
  of TC-TEX-06); (b) the conversion findings as task failures (`texture_too_small`,
  `texture_not_pow2`, `texture_type_mismatch`, `texture_codec_unsupported`; TC-TEX-04),
  `kit_texture_not_used` (TC-KIT-10), portraits encoded to BC3 DDS (TC-PRT-01, TC-TEX-06's
  portrait half; `dds_convert` has no DDS output for a Fox target yet, lead first); (c) bounded
  batches on the cancel token. (a) done 2026-10-03 (Fable; contradictions applied: the role
  computed inside `texture::convert`, `texture_format` kept returning `SourceFormat`): every
  model-folder, Common and kit texture through the run's `Converter` (`CompileContext::new`,
  `cache_policy`), the gate open to every accepted format but portraits; the BC7 encoder builds
  optimized in dev (decision entry: TC-TEX-01 took 56 s unoptimized, 9 s now). TC-TEX-01/02
  cited. Gates green (95 of 203); `mutants-diff 3b9cc36`: 23, 15 caught, 8 unviable, 0 missed.
  For converge: `Converter::convert` returns `Arc<[u8]>` even under `Bypass` and `Entry` owns a
  `Vec`, so each texture is copied once (`Use`) or twice (`Bypass`); an `.ftex` source is now
  rebuilt rather than passed through. (b) done 2026-10-03 (Fable; contradictions accepted: a
  failed portrait task reports `DropFile`; the size checks read the written FTEX's header, so
  no second decode; `kit_texture_not_used` is reported before the gate, noisy until 4.20; lead
  first: the catalog's drop scopes and signature rule, decision entry, and the fixtures):
  `dds_convert::encode_dds` (sharing `convert`'s steps), `texture::TextureError`, the four
  findings, Common's `DropFile`, portraits in any format. TC-TEX-04/06, TC-PRT-01, TC-KIT-10
  cited. Gates green (99 of 203); `mutants-diff 8ae37d9`: 75, 68 caught, 7 unviable, 1 missed
  (`needs_pow2`'s `mipmaps > 1`), given a test with the `single_level.dds` fixture in one rework
  round (red shown); the remote half peaked at 7.90 GiB of 8G (maintainer asked). Found:
  `texture_codec_unsupported` also covers `dds_convert`'s other `Unsupported` refusals (a 16-bit
  TGA, the 4 GiB limit), which the help names. (c) moved to Phase 8's cancellation 2026-10-03
  (decision entry): the bound already holds, one texture at a time per worker; nothing in Phase 4
  cancels a run, so the between-textures check is built with the Cancel that triggers it.
  Crates: tc (`processing/texture.rs`,
  `processing/kit.rs`) → verify: a `/co/` player with `skin.png` (1024x1024, alpha) compiled for
  PES 21 then PES 18: `ftex::info` of the emitted texture reports BC7 then BC3, each with an
  11-level mip chain; a 3x3 `skin.png` reports `Error texture_too_small [DropFolder]` and the
  folder is absent from the CPK

- [ ] 4.7 **Deep validation pass**: `ae` orchestrates the deep pass over the sanitized export with
  the same scope rule and pass-through eligibility; the format crates' `check` findings mapped
  (`fmdl_vertex_far_from_origin`/`model_vertex_far_from_origin` → `vertex_too_far_from_origin`,
  not pass-through-eligible), `portrait_conflict`, `logo_file_invalid` (decode),
  `kit_config_invalid`, `kit_texture_too_big`, `kit_texture_uncompressed`, `settings_toml_invalid`
  (parse only, through `pes_savefile`'s `PlayerSettings` TOML reader; the other `settings_*` and
  `fpc_strip_conflict` codes describe resolved values and are Phase 5's); CLI `check` runs both
  passes on every source kind, archives included; a test that every `ISSUE_CODES` entry and every
  format finding code has a catalog row. Plan: `pipeline.md` "2. Per-export serial steps" step 2
  (deep format pass); `object_model.md` "Validation semantics" (sanitized versus eligible: "the
  deep pass applies the same rule"); `messages.md` (`vertex_too_far_from_origin`). IDs:
  TC-CHK-01..05, TC-PRT-02, and from 4.6 TC-TEX-03, TC-TEX-05, TC-PRT-03 (texture findings and
  `fmdl_no_texture_ids` reported by `check`). Crates: ae (`validate/deep.rs`, new), tc (`check.rs`, `compile.rs`,
  `messages.rs`) → verify: `check` on a `/co/` export whose `boots.fmdl` holds a vertex 6000 units
  out prints `Error vertex_too_far_from_origin [DropFolder]` naming the file and exits 1;
  `compile` with `pass_through` on still leaves the folder out of the CPK

- [ ] 4.8 **Kit colors, UniColor and TeamColor (the `bins/` module)**: kit `colors.txt` grammar
  (`player_folders.md` "Root files", "Colors"; waits on the maintainer: its confirmation, an
  export text format), `color_entry_invalid`, derivation via
  `color_tools::kit::extract_kit_colors` on the decoded main texture (`kit_colors_derived`), the
  magenta/black pair (`kit_colors_missing`), `icon.txt` (default 3); `UniColor.bin` built on the
  bundled base (lead first: the `UniColor.bin` and `TeamColor.bin` bases as lead-authored
  fixtures with a provenance README) at Red's per-team offsets, an entry applied only when its
  kit's task commits (placeholder kits included: TC-SRC-01's p2 gains its entry); root
  `colors.txt` → `TeamColor.bin`, `team_colors_missing` (I). Plan: `pipeline.md` "4. Per-export
  non-model steps" (Bins accumulation), "Resolved decisions" (Kit colors fallback);
  `libs/color_tools.md` "Dominant kit-color extraction". IDs: TC-KIT-11..14, TC-ROOT-10,
  TC-BIN-01..03. Crates: tc (`bins/mod.rs`, `processing/kit.rs`), resources (`resources/bins/`) →
  verify: a `/co/` export with `p1/colors.txt`, `p2/kit.dds` without colors and an empty `p3/`:
  the CPK's `UniColor.bin` carries at team 714's p1 offset the file's two colors, at p2's the
  pair `extract_kit_colors` returns for the decoded texture, at p3's the magenta/black pair; the
  tracer parity test compares `TeamColor.bin` byte-identical and `UniColor.bin` with g1's entry
  excluded (Red does not derive)

- [ ] 4.9 **Kit configs, FPC reconciliation and collars** (the brief settles what a collar in
  the other engine's format does: converted, or refused; `Collars/` admits any model format):
  team kit-FPC status (`fpc.on` in any
  player folder → On), generated configs with FPC values (`kit_config::fpc::apply_fpc`), supplied
  configs reconciled upward (`kit_config_fpc_adjusted`, GK included),
  `kit_config_version_clamped`; `fpc.on`/`fpc.off` no longer refuse the export (their savefile
  half is Phase 5); `Collars/` gets its allowlist row in `ae` (model files named `collar_<ID>`,
  any model format; the per-version stock sets are in `messages.md` `collar_id_invalid`; waits on
  the maintainer: its confirmation, game-facing), `collar_<ID>` parsed with or without zero
  padding, `collar_id_invalid` (not a stock collar of the target version, 105),
  `collar_id_conflict` in canonical export order, every kit config's collar fields rewritten
  after FPC, collar files passed through to `uniform/nocloth/#Win/` under the game's three-digit
  name. Plan: `fpc_toggle.md` "Team
  kit-FPC status and kit configs"; `pipeline.md` "4. Per-export non-model steps" (Kits, Collars),
  "Resolved decisions" (Collar contract); `object_model.md` "File-type allowlist". IDs:
  TC-KIT-15..17, TC-CMN-01..03. Crates: ae (`conventions/file_types.rs`, `validate/folders.rs`),
  tc (`processing/kit.rs`, `processing/team_assets.rs`, `plan/`) → verify: a `/co/` export with
  `fpc.on` in slot 05 and a supplied `p1/config.toml` without FPC values: the emitted 120-byte
  config decodes with `kit_config::fpc::matches_fpc` true and `kit_config_fpc_adjusted` is
  reported for p1; with `Collars/collar_12.fmdl` added, every emitted config's collar fields read
  12 and the file sits at `Asset/model/character/uniform/nocloth/#Win/collar_012.fmdl`

- [ ] 4.10 **Kit layout conversion**: `KIT_LAYOUT_REMAP` (lead-authored measurement,
  `kits/layout.rs`; band edges from texel correspondence plus the hand-adjusted fixture pair),
  `kit_layout_converted`, Lanczos3 for bands whose width changes, every texel outside the islands
  copied, `_chest`/`_back`/`_name`/`_leg` untouched, placeholder never re-laid, the inverse table
  for `fox` kits on pre-Fox (exercised fully in 4.16). Plan: `pipeline.md` "4. Per-export
  non-model steps" (Kits, "Layout conversion"); `player_folders.md` "Kit layout marker". (waits
  on the maintainer: the fixture pair, one kit an author shipped for a 16/17-era and a 21-era
  cup, from the maintainer's library; the step waits for it.) IDs: TC-KIT-18..20. Crates: tc
  (`processing/kits/layout.rs`) → verify: a `pre-fox` kit compiled for PES 21 decodes to a
  texture whose every texel outside the sock and shorts islands equals the no-marker compile's
  and whose islands match the golden from the fixture pair; a synthetic flat-band texture
  round-trips pre-Fox → Fox → pre-Fox exactly

- [ ] 4.11 **Team root artifacts and Common**: logo (main decoded, made square per tag, Lanczos3 to
  512 and 256, `logo_small*` or main to 128, PNG with alpha, `logo_fit_applied`, `logo_upscaled`,
  one atomic producer, names `emblem_0{id}_r_ll/_r_l/_r.png` on PES 15-19 and `e_000{id}...` on
  20-21, `_r_ll` 512, `_r_l` 256, `_r` 128); `notes.txt` → `output/teamnotes.txt` (UTF-8/LF,
  canonical order, accepted exports only, `notes_found`); `kitN` variants completed
  (`kit_variant_missing` copies
  the lowest, `kit_variant_model_fox`), `dummy_kit*` stems skipped by the existence checks;
  `Common/` on Fox: textures converted into `common/{team_id}/sourceimages/`, models only
  reachable through `.common` links (4.5; 4.5c already emits `Common/`'s `.dds`/`.ftex` as one
  task); texture `.common` links (`hair.dds.common`: the player's stem resolves into Common and
  its path names the team's Common output; found at 4.5c with no step and no acceptance ID, so
  this step writes the scenario first). Plan: `pipeline.md` "4. Per-export non-model steps"
  (Logo, Common, Kit-dependent assets), "2. Per-export serial steps" step 5; `model_format.md`
  "Kit-dependent assets". IDs: TC-ROOT-06..09, TC-CMN-04..06. Crates: tc
  (`processing/team_assets.rs`) → verify: a 1000x600 `logo.png` yields three square PNGs decoding
  to 512, 256 and 128 pixels with transparent side borders and `logo_fit_applied` (`fit`);
  `logo_small_crop.png` beside it changes only the 128 one; the notes of two accepted exports
  appear in `output/teamnotes.txt` in canonical order and a skipped export's do not

- [ ] 4.12 **`ingame_face` processing (Fox)**: no face package emitted; arbitrary-named models
  rerouted to the player-exclusive boots folder (merged, the paired `.skl` becoming `boots.skl`);
  gloves parts to the player's gloves folders; a boots/gloves link combined with local parts
  under the marker; an empty `face/` ignored; a player without face models and without the marker
  gets the blank face folder (its contents per engine: `pipeline.md` step 4). Plan:
  `player_folders.md` "`ingame_face` marker", "`ingame_face` with shared links"; `pipeline.md`
  "2. Per-export serial steps" step 4. IDs:
  TC-MOD-16..19. Crates: tc (`processing/model.rs`, `plan/`) → verify: a `/co/` slot 05 folder
  with `ingame_face`, `torso.fmdl`, `torso.skl` and `glove_l.fmdl`: the CPK has no
  `face/real/71405/`, has `boots/k0625/#Win/boots.fpk` holding `boots.fmdl` and a `boots.skl`
  byte-equal to `torso.skl`, and `glove/g0625/#Win/glove.fpk`

- [ ] 4.13 **Run planning**: `duplicate_aesthetics_export`; canonical export order by normalized
  source-relative path plus source kind; every task's output namespace allocated in planning,
  `duplicate_path` (folder E, export E, override W); the `overrides/` tree of the data directory
  injected first (`overrides_active`); the writer's duplicate invariant kept; export revision
  pinning at planning, `source_changed_during_run` (a folder's revision is the listing's (path,
  size, modified time) set, an archive's its size and modified time, pinned at planning and
  rechecked before each task's read). Plan: `pipeline.md` "3. Per-model-folder parallel steps"
  (the run-level planning paragraph), "5. Writer" steps 1-3, "Resolved decisions" (Cross-export
  duplicate precedence, Source snapshot); `settings.md` "Path resolution" (`overrides/`). IDs:
  TC-PLN-03..07. Crates: tc (`plan/mod.rs`, `plan/overrides.rs`, `reader/`) → verify: exports
  `co - A/` and `co - B.zip` in one root both report `duplicate_aesthetics_export` and no CPK is
  written; with `overrides/common/etc/TeamColor.bin` in the data directory the CPK's entry has
  the override's bytes and the console shows `overrides_active` then `Warning duplicate_path
  [Keep]`

- [ ] 4.14 **Pre-Fox faces (PES 15-17, native `.model` + `.mtl`)**: `face.xml` generated from the
  suffix table (`face_neck` for `face_high`, `parts`, `gloveL`/`gloveR`, `handL`/`handR`,
  `model_type_<x>`, `_ratio_<n>`, `uniform` → `uniform_sub` on PES 15), `oral_`/`_win32` affixes,
  `xml_face_neck_added` with the dummy model and MTL (lead first: both as lead-authored fixtures
  with a provenance README); `.mtl` texture paths rewritten to
  `model/character/uniform/common/{team_id}/{folder}/` with `.dds` extensions; textures as DDS
  (BC7 transcoded to BC3, `texture_not_div4`); each face packed into
  `common/character0/model/character/face/real/{id}.cpk`; portraits `player_{id}{NN}.dds`; a
  shared face folder copied per linking player with local files on top; boots/gloves links keep
  the shared folder's ID, the shared folders emitted as loose files under `boots/{id}/`; local
  boots/gloves parts ride in the face XML (no per-player folders); `model_name_invalid`,
  `edithair_unsupported`, `xml_oral_prefix_missing` (PES 16), `.model.common` as an XML Common
  path, `xml_common_path_invalid`. Plan: `pipeline.md` step 3 "Pre-Fox fixups", steps 5-7, "Game
  paths reference"; `player_folders.md` "A link plus local models combines" (pre-Fox), "Model
  names". (waits on the maintainer: a Red run on PES 17 over a hand-migrated Studio twin of a
  small pre-Fox export, committed under `tests/fixtures/tracer_prefox/` if under 1 MB, the
  pre-Fox parity reference for 4.14-4.17; until it exists the step's checks are the ones below.)
  IDs: TC-MOD-20..25, TC-TEX-07. Crates: tc (`processing/model.rs`, `processing/material.rs`,
  `processing/texture.rs`, `paths.rs`) → verify: a `/co/` slot 05 folder with the smallest
  `pes_model` fixture pair as `face.model` + `face.mtl` and `skin.dds`, compiled for PES 17: the
  CPK holds `common/character0/model/character/face/real/71405.cpk` whose `face.xml` lists one
  `face_neck` entry, whose MTL names `skin.dds` under `.../common/714/05 - .../`, and no `Asset/`
  entry

- [ ] 4.15 **Pre-Fox XML and MTL checks, user `face.xml`**: every `xml_*` and `mtl_*` row of the
  catalog, the Error/Warning/Info line of "User-supplied `face.xml`", `xml_ignored_fox`,
  `mtl_texture_not_found` deep (mesh-used materials) against `mtl_texture_unused_missing`, the
  states checks also on a converted model's `[prefox.states]`. Plan: `messages.md` "XML/MTL
  content checks", "User-supplied `face.xml`". IDs: TC-XML-01..07. Crates: tc
  (`processing/material.rs`) → verify: the hand-written xml of `testing.md` ("user `face.xml`")
  compiled for PES 17 is emitted with 714 substituted into its Common path, its unknown `type`
  and extra attribute kept with `xml_type_unknown` and `xml_attribute_unknown`, `level="1"` kept
  with `xml_level_lod`; a `<model>` without `path` drops the folder with `xml_model_path_missing`

- [ ] 4.16 **Pre-Fox kits, bins and DDS compression**: `kit_mask` injected from the mask template
  (lead first: the template as a lead-authored fixture with a provenance README) when absent,
  `kit_srm` dropped with `kit_texture_not_used`, kit configs emitted as loose per-team bins under
  `uniform/team/{team_id}/` (no `UniformParameter.bin` before PES 18), the pre-Fox
  `UniColor`/`TeamColor` layouts, `fox` kits re-laid with the inverse table (4.10);
  `dds_compression` (`auto` follows `multicpk_mode`, `true`, `false`) wrapping every emitted DDS
  with `wezlib::compress` on PES 15-17 only, already-wrapped sources passed through, the level
  chosen by measuring levels 1, 3 and 6 on the tracer's DDS set and recorded in a decision entry.
  Plan: `pipeline.md` "4. Per-export non-model steps" (Kits: mask and srm); `settings.md`
  (`dds_compression`, "DDS compression cost"). IDs: TC-KIT-21..23, TC-TEX-08, TC-BIN-04. Crates:
  tc (`processing/kit.rs`, `processing/texture.rs`, `bins/`, `settings.rs`) → verify: PES 17
  compile of `/co/` p1 without a mask emits `u0714p1_mask.dds` byte-identical to the template;
  with `dds_compression = true` every `.dds` entry satisfies `wezlib::is_wrapped` and
  decompresses to the `false` run's bytes; a `kit_srm.dds` reports `kit_texture_not_used` and no
  `_srm` entry exists

- [ ] 4.17 **Cross-format conversion and source selection**: target-native first, then glTF, then
  the opposite native format converted through `model_convert::convert` (FMDL → `.model` + `.mtl`
  as one bundle, `.model` → FMDL), `model_conversion_failed`; `model_source_ambiguous` keeps its
  catalog row but its only input, two glTF files of one stem, is Phase 7's, so no scenario here; a
  selected glTF representation refused with `model_gltf_unsupported` (folder dropped, not
  pass-through-eligible); `bone_folded_for_version`, `skeleton_retargeted`, an SKL generated from
  the IR for a converted model with bones outside the target's tables,
  `vertex_too_far_from_origin` on the converted form, the `metal` family's environment-cubemap
  sampler and template on pre-Fox (lead first: the cubemap as a lead-authored fixture with a
  provenance README). Plan: `pipeline.md` step 3 "Format conversion", "Resolved decisions" (Model
  source selection); `development_plan.md` "Phase 4" `processing/` (glTF refusal);
  `model_conversion/README.md`. IDs: TC-MOD-26..30. Crates: tc (`processing/model.rs`) → verify:
  the tracer's `fcl_hair.fmdl` compiled for PES 17 yields a `.model` + `.mtl` pair in the face
  CPK that `pes_model` reads back with the FMDL's mesh count; `boots.fmdl` beside `boots.model`
  on PES 21 compiles the FMDL with no conversion finding; `boots.glb` alone on PES 21 reports
  `model_gltf_unsupported` and drops the folder

- [ ] 4.18 **Hand auto-split**: `model_convert::ops::hand_split::split_by_skeleton_group` on every
  model with `skh_*_l`/`skh_*_r` weights before categorization, the split parts as virtual
  `glove_l`/`glove_r` parts, models without such weights untouched. Plan: `player_folders.md` "At
  compile time, the pipeline" step 0; `development_plan.md` "Phase 4" `processing/`;
  `model_conversion/README.md` (`ops/hand_split.rs`). IDs: TC-MOD-31. Crates: tc
  (`processing/model.rs`) → verify: a `/co/` slot 05 folder holding `model_convert`'s hand-split
  fixture as `body.fmdl`: the CPK holds `glove/g0625/#Win/glove.fpk` with `glove_l.fmdl` and
  `glove_r.fmdl`, and the face's merged FMDL plus the two gloves hold exactly the source's vertex
  count

- [ ] 4.19 **Referees**: a `/refs/` export compiled into `refs_cpk_name`'s CPK; slots 01-35 mapped
  by `players.txt`, a folder mapped to several slots prepared once and instantiated per slot; IDs
  `k99XX`/`g99XX` and the referee face paths (the referee rows of "Game paths reference"); the
  per-referee common subfolder keyed by folder name under team 999; the refscpk template content
  (lead first: as lead-authored fixtures with a provenance README); pre-Fox `ref_marker.dds` template
  injection; the team CPK untouched by referee content. Plan: `blue_port.md` "Referee export
  processing"; `pipeline.md` step 6 (the referee layout), "5. Writer" step 5 (refs CPK);
  `player_folders.md` "Multi-mapped processing". IDs: TC-REF-01..05. Crates: tc
  (`processing/referee.rs`, `plan/refs.rs`, `paths.rs`), resources → verify: a refs export
  mapping `Ref A` to 01, 20 and 35 on PES 21 writes `4cc_35_referees.cpk` holding three face
  packages, one `common/999/Ref A/sourceimages/` texture set, and `k9901`, `k9920`, `k9935`
  boots folders when the folder has boots; the team CPK of the same run holds no `999` path.
  Open first (found at 4.4a): the plan gives a shared `Boots/`/`Gloves/` folder in a refs export
  no ID (refs have no block; the structure pass's pool check skips refs); rule it in the plan

- [ ] 4.20 **Withdraw the Phase 3 subset gate**: `plan/subset.rs` and `content_not_yet_compiled`
  removed (the catalog row reads withdrawn), TC-OUT-06 withdrawn, every content kind and both
  engines reach processing; the "Phase 3 scope" paragraph's refusals are gone except the glTF
  one (4.17). Plan: `team_compiler/README.md` "Acceptance" (Phase 3 scope: "The code is withdrawn
  when Phase 4 compiles everything"); `messages.md` (`content_not_yet_compiled`). IDs: TC-OUT-06
  withdrawal. Crates: tc → verify: `rg content_not_yet_compiled crates/` finds nothing; `rg
  "subset" crates/tools/team_compiler/src` finds nothing; `just acceptance` reports TC-OUT-06
  withdrawn and no test citing it. The paragraph's `missing` context and its injected-file
  refusals already went at 4.5a (templates injected). Open first (found at 4.5b): a model in a
  player's `gloves/` subfolder whose suffix gives no side is refused only by the gate; a shared
  `Gloves/` folder's is `fmdl_name_invalid` (validation, Fox), the likely finding for both.
  Likewise a `face_diff.bin`/`fcl_hair_sim.fclo` in `boots/` or `gloves/` (validation allows it,
  no package has a slot for it). Open first (found at 4.5c): a `.common` link to a texture or a
  material file is refused only by the gate (4.11 and Phase 7 build them); a `Common/legs.skl`
  beside a glove link's model is accepted and never read, while a local glove's `.skl` is refused
  by the gate, so the two need one rule. The
  gate's role helpers (`FolderModels`, `player_file`,
  `holds_model`, `package_of`) move to their users, not out with it. Open first (found at 4.4a):
  a plainly linked shared folder holding no model (empty, or textures only) passes validation and
  is refused only by the gate; without it, it would take a shared ID and emit nothing. The plan is
  silent: decide whether it is a validation finding

- [ ] 4.21 **Bins from the installed CPKs**: `bins/dpfl.rs` `DpFileList.bin` reader (16-byte
  header, 48-byte records, an all-zero tail of any length ignored; measured on the PES 17 and 21
  files, the PES 15/16/18/19/20 files when an install exists); the walk from
  the entry of next-lower priority than the output CPK upward, each bin from the first CPK that
  holds it (`cpk::CpkArchive`, `wezlib::decompress_if_wrapped`), the supplying CPK reported with
  `bin_source` (I; `bundled` when the embedded base was used); the bundled bases when no install
  or DPFL is found; the `templates/` override directory (`template_override_active`,
  `template_override_unreadable`); FPC patching of kit slots absent from the export from the
  installed `UniformParameter` or pre-Fox kit bins (`kit_config_fpc_adjusted`,
  `kit_config_fpc_unpatched`). Plan: `pipeline.md` "4. Per-export non-model steps" (Bins
  accumulation), "Resolved decisions" (Working-bin lookup, Templates and fallback bins);
  `fpc_toggle.md` "Kit slots absent from the export". IDs: TC-BIN-05..09. Crates: tc
  (`bins/dpfl.rs`, `bins/mod.rs`, `templates.rs`) → verify: a sandbox install whose DPFL lists
  `4cc_08_bins`, `4cc_60_midcup` and `4cc_90_test` with a `4cc_08_bins.cpk` holding a
  `UniColor.bin` in which team 714's p1 entry is set and a `4cc_60_midcup.cpk` holding one in
  which it differs: compiling `/co/` with only `p2/` leaves p1's bytes equal to the
  higher-priority CPK's and p2's set; with `cpk_name = 4cc_60_midcup` the p1 bytes come from
  `4cc_08_bins.cpk`

- [ ] 4.22 **Fox player tables**: `BootsList.bin` and `GloveList.bin` read from the installed (or
  seed) CPK by the same walk, the (player id, item id) pair of every compiled player whose
  boots/gloves output committed replaced with the planned ID, a failed output keeping its row,
  every other row kept, written whole, plain, sorted by id; `PlayerAppearance.bin` read and
  written whole with no row changed (the rows are Phase 5's, with the stock and default-ID
  boots/gloves rows). **External:** the in-game effect of a gloves row waits on FoxDen's gloves
  patch (worklog "Issues"); the table content does not. Plan: `pipeline.md` "Bins accumulation"
  (player appearance tables), "Game paths reference" (the three rows); `settings_toml.md` "Player
  settings in exports" (Fox); `development_plan.md` "Phase 4" `bins/`. IDs: TC-BIN-10..12.
  Crates: tc (`bins/mod.rs`, `paths.rs`) → verify: PES 21 compile of `/co/` with slot 05's
  `boots.fmdl` over an installed `BootsList.bin` of ten pairs: the CPK's table holds eleven pairs
  sorted by id with (71405, 625) among them and the ten unchanged; `GloveList.bin` is
  byte-identical to the installed one; `PlayerAppearance.bin` is byte-identical to the installed
  one

- [ ] 4.23 **Output sink and modes**: `output/sink.rs` `OutputSink` (CPK, loose folder) fed by
  output-relative paths; `processing/materialize.rs` the one seam (relocation and FPK packing,
  skipped in test mode); `--mode test` writing `output/test_output/<canonical source key>/` with
  export-relative processed entries and no FPK; `--mode sideload` replacing the whole contents of
  `{pes_folder_path}/livecpk/` with the run's game-path tree, refused on PES 15/16 (exit 2);
  artifact routing per mode (`teamnotes.txt` under `output_folder_path` in every mode; sideload:
  bins, overrides and referee content at their game paths in `livecpk/`; test: bins under
  `test_output/_bins/` at game-relative paths, referee content per export like a team's,
  overrides not applied); the loose sink is the harness the parity test drives. **External:**
  FoxDen's LiveCPK gaps (worklog "Issues": 2019/2021 sites, PES 2020, path length, in-match
  loads) decide the in-game effect only; the written tree is what the scenarios test. Plan:
  `pipeline.md` "5. Writer" step 5 and the output-modes paragraphs, "Resolved decisions"
  (Output-mode artifact routing); `settings.md` "CLI", "Path resolution"; decision entry
  "sideloading through FoxDen (4.0e)". IDs: TC-OUT-07..11. Crates: tc (`output/sink.rs`,
  `processing/materialize.rs`, `cli.rs`) → verify: `--mode test` on the tracer writes
  `output/test_output/egg Tracer/Players/05 - The Chad Stormworks Player/fcl_hair.fmdl` with its
  texture path rewritten and no `.fpk` anywhere under `test_output/`; `--mode sideload` with a
  stale `livecpk/old.txt` removes it and writes files whose relative paths and bytes equal the
  entries of a normal-mode CPK of the same export

- [ ] 4.24 **Deployment**: each staged CPK copied to `download/{name}.cpk.partial` and renamed over
  the old one, the marker file, the staging folder removed; the preflight before any export is
  read (`download/` probe; `pes_version_mismatch` W); degradation to `output/` with
  `pes_folder_not_found`, `dpfilelist_missing`, `cpk_name_unlisted`, `old_cpk_locked`,
  `deploy_target_unwritable` (`elevation::is_access_denied`), `dpfilelist_outdated` (the
  installed list lacks a target the bundled one has, 4.25 supplies the template); `--no-deploy`
  unchanged; stale `.staging/` folders of dead runs removed at start; deployment adds no exit
  code (a degraded run exits 1, an aborted one 3, as the mapping already says). Plan:
  `pipeline.md` "6. Post-processing" (Staging, Deploy CPKs, Degraded run, `--no-deploy`,
  Destination writability preflight); `messages.md` "Output stage and savefile"; `settings.md`
  "CLI" (exit codes). IDs: TC-DEP-01..07. Crates: tc (`output/deploy.rs`, `cli.rs`) → verify: a
  sandbox PES folder with `PES2021.exe`, a DPFL listing `4cc_90_test` and an old
  `download/4cc_90_test.cpk`: `compile` leaves `download/4cc_90_test.cpk` equal to the staged
  bytes, no `.partial`, the marker beside it, nothing in `output/`, exit 0; with the old CPK held
  open by the test: `old_cpk_locked`, `output/4cc_90_test.cpk` holds the run's CPK, the old one
  is byte-identical, exit 1

- [ ] 4.25 **DpFileList upgrade**: the official per-version `DpFileList.bin` embedded (lead first:
  as lead-authored fixtures with a provenance README, from the installed PES 17 and 21 files);
  `upgrade-dpfl [--yes]`: the entries the official list lacks printed with the size of each
  matching `download/*.cpk`, nothing written without `--yes`, the installed file replaced byte
  for byte with `DpFileList.bin.bak` kept, no CPK ever deleted by the command;
  `dpfilelist_outdated` names the subcommand. Plan: `pipeline.md` "6. Post-processing"
  (DpFileList upgrade); `settings.md` "CLI" (`upgrade-dpfl`). IDs: TC-DEP-08..10. Crates: tc
  (`bins/dpfl.rs`, `cli.rs`), resources → verify: an installed DPFL lacking `4cc_40_teams` beside
  a 1 KiB `download/4cc_40_faces.cpk`: `upgrade-dpfl` prints `4cc_40_faces` with `1 KiB` and
  exits without writing; `--yes` makes `DpFileList.bin` equal to the embedded PES 21 list and
  `DpFileList.bin.bak` equal to the old file, `4cc_40_faces.cpk` still present

- [ ] 4.26 **Multi-CPK mode**: `multicpk_mode` honored (the Phase 3 refusal removed); slots from
  the DPFL entries matching `{prefix}_{NN}_{teams_cpk_name}` exactly, ordered by number; whole
  teams placed first-fit by exact size under `cpk_part_max_size`; every unfilled slot written as
  the empty placeholder CPK, byte-identical to the shipped one (the lead commits
  `E:/PES2021/download/4cc_68_midcup.cpk`, 6,272 bytes, as the placeholder fixture);
  `cpk_slots_exhausted`, `cpk_team_exceeds_cap`, `cpk_size_over_limit` (single-CPK); the bins CPK
  (`bins_cpk_name`), the
  refs CPK beside them; `dds_compression = auto` follows the mode; deployment per generated CPK.
  Plan: `pipeline.md` "5. Writer" step 6 ("Multi-CPK mode: teams parts"); `settings.md`
  (`multicpk_mode`, `teams_cpk_name`, `cpk_part_max_size`, `bins_cpk_name`). IDs: TC-OUT-12..16,
  TC-DEP-11. Crates: tc (`output/writer.rs`, `output/deploy.rs`, `settings.rs`), cpk (placeholder
  writer, if not already byte-identical) → verify: `/co/`, `/a/` and `/b/` exports with
  `cpk_part_max_size` set just above the first two teams' compiled size and a DPFL reserving
  `4cc_40_teams`..`4cc_44_teams`: `4cc_40_teams.cpk` holds the first two teams whole,
  `4cc_41_teams.cpk` the third, `4cc_42`..`44` are each 6,272 bytes equal to the placeholder
  fixture, `4cc_08_bins.cpk` holds the bins and nothing else

- [ ] 4.27 **Fox referee marker into `dt00_x64.cpk`** (waits on the maintainer: the transaction
  protocol, and whether FoxDen's LiveCPK could serve the Fox marker instead, which would retire
  the setting): `dt00_overwrite_allow` read; `ref_marker_needs_consent` when off; when on, the
  converted marker written into the system CPK as part of the deployment transaction with
  `dt00_x64.cpk.bak` kept and the old file restored on `dt00_write_failed`. Plan: `blue_port.md`
  "Referee export processing" (`ref_marker.dds`); `messages.md` "Referees"; `pipeline.md`
  "Resolved decisions and open questions" (`dt00_x64.cpk` transaction). IDs: TC-REF-06..07.
  Crates: tc (`output/deploy.rs`, `processing/referee.rs`) → verify: a refs export with
  `ref_marker.dds` and the setting off leaves a sandbox `dt00_x64.cpk` byte-identical and
  reports `ref_marker_needs_consent`; on, the CPK's marker entry is the FTEX of the file and
  `dt00_x64.cpk.bak` equals the old CPK

- [ ] 4.28 **Complete memory accounting**: the permit grows with decoded textures, converted
  models, merged meshes and packed entries (the open question "Complete memory accounting"); a
  team's held batches in multi-CPK mode charged; shared and cache-owned bytes charged while
  retained. Plan: `pipeline.md` "Resolved decisions and open questions" (Complete memory
  accounting); `libs/pipeline.md` "Memory budget" (last bullet). IDs: none (not user-observable;
  the measurement is the proof). Crates: pipeline (`Permit::grow`, or the shape the step
  settles), tc → verify: a recording `MemoryBudget` wrapped around the tracer compile reports a
  peak equal to the sum of the decoded and packed sizes the tasks allocated, not the source
  sizes; with a cap below one decoded texture the task waits for an empty pipeline and completes
  (the oversized rule)

- [ ] 4.y `dds_convert` cache retention bound (found at 2.20d converge; spec `libs/dds_convert.md`
  "In-memory conversion cache", "Retention is separately bounded and budgeted"): the
  `Converter` holds every distinct conversion until `clear`; the pipeline's memory budget
  charges `retained_bytes` and evicts under pressure, and repeated edits do not keep every
  superseded conversion → verify: a test compiling the same export with one texture edited N
  times retains one conversion of it, and a budget smaller than the cache evicts rather than
  blocking a task. (Placed after 4.28: eviction is the budget's policy, which 4.28 gives the
  budget the accounting for.)

- [ ] 4.y-conv **Converge** (`AGENTS.md` "Closing a phase" (1)): the lead's audit of
  `team_compiler`, `aesthetics_export`, `pipeline` and the Phase 4 edits of the lib crates
  against `development_plan.md` "Phase 4", the `pipeline.md` walkthrough, `messages.md`,
  `settings.md` and every TC ID; `just acceptance strict`; the design-health pass (design-tell
  sweep, `just mutants` per crate with every survivor triaged, `pub` census); the census over
  every export on the maintainer's machine (Everything index) run through `check` and `compile
  --mode test`, tallied by outcome; then the cross-family reviewer loop, one surface per crate
  (queued per the handover if no reviewer is available); each gap a new step above this one →
  verify: `just gates` green, `just acceptance strict` green, the census tally recorded in the
  worklog with every failing class diagnosed

- [ ] 4.z-rewrite **Rewrite** (`AGENTS.md` "Closing a phase" (2) and (3)): `development_plan.md`
  "Phase 4", `pipeline.md` (the walkthrough's Phase 4 parts and the "Run driver shapes" block,
  which becomes the Phase 4 shapes), `messages.md`, `settings.md`, `object_model.md` "Validation
  semantics" (deep pass), `team_compiler/README.md` "Development phases" and the "Phase 3 scope"
  paragraph (its refusal list replaced by what Phase 3 built) in the present tense; resolved open
  questions moved to "Resolved decisions"; the worklog's step list collapsed to the phase row →
  verify: `rg -n "Phase 4" docs/plans` shows no future-tense Phase 4 sentence outside deferred
  items; the worklog's Phase 4 heading is one row plus any `manual: checked` lines

Out of Phase 4 (named so the steps above do not absorb them): the savefile step, the aesthetics
patch and every `savefile_*`/`patch_written` code (Phase 5, `output/savefile.rs`);
`PlayerAppearance.bin` rows and the stock/default-ID boots and gloves rows from `settings.toml`
(Phase 5); the other `settings_*` codes and `fpc_strip_conflict` (Phase 5, resolved values; Phase
4 reports only `settings_toml_invalid`); `run_pes` and the Launch PES button as one launcher
(Phase 8); `FolderStatus`, `Progress` and `Complete` events, live validation (`check/`,
`watcher.rs`) and every GUI action on a degraded run (Retry, Open output folder, Relaunch as
administrator, Upgrade DpFileList dialog): Phase 8; glTF sources: Phase 7; the GPU BC7 backend:
16.x; the parity run over upgraded reference exports: after Phases 5 and 6
(`development_plan.md` "Phase 4" Verification).

Phase 4 open questions (maintainer):

- `colors.txt` grammar: confirm the grammar now in `player_folders.md` "Root files", "Colors"
  (one grammar for the root and the kit files, four team colors), since it is an export text
  format. 4.8 waits on the confirmation.
- `dt00_x64.cpk` transaction: the backup and rollback protocol for the Fox referee marker, and
  whether FoxDen's LiveCPK could serve the marker from `livecpk/` instead of editing a system CPK,
  which would retire `dt00_overwrite_allow`. 4.27 waits on it.
- Kit layout fixture pair: one 4cc kit an author shipped for both a 16/17-era and a 21-era cup,
  from the maintainer's library (Everything index). 4.10 waits on it; 4.16's inverse-table check
  uses it.
- The default `cpk_name` on PES 21: the cup's official PES 2021 `DpFileList.bin` has no
  `4cc_90_test` entry (PES 2017's has), so a default-settings compile on a PES 21 install would
  report `cpk_name_unlisted` and skip deployment. A per-version default, a test slot in the
  official list, or intended? 4.24 waits on it.
- `DpFileList.bin` per PES version: the PES 15/16/18/19/20 files, when an install exists (17 and
  21 are measured). 4.25 embeds each as it arrives.
- Pre-Fox parity reference: a Red run on PES 17 over a hand-migrated Studio twin of a small
  pre-Fox export (as the tracer was made), committed under `tests/fixtures/tracer_prefox/` if
  under 1 MB. Without it every pre-Fox byte of 4.14-4.17 is unchecked until Phase 6.
- A base64 decoder for `face_diff.xml` (4.5d): no crate in the workspace decodes base64, and the
  plan's dependency table names none. Add the `base64` crate (a new dependency), or hand-write
  the standard-alphabet decoder (about 30 lines, in `team_compiler`)? `roxmltree` and `wezlib`
  cover the XML and zlib parts. 4.5d waits on it.

### Phase 5 — Savefile integration

Steps are itemized when Phase 4 closes; one is fixed already:

- [ ] 5.0 Check the latest commits made to the unofficial 4ccEditor fork (`Tools_4cc/4ccEditor-1`,
  origin `AnonymousClouds/4ccEditor`; `pes_savefile/README.md` names it the reference): it is
  under constant development. Fetch, then read every commit after `4a95b7c` (2026-06-19, "Add
  Tactics tab for 16-21. Add ability to import Texports from 15-21"), the local clone's head at
  Phase 2's close → done when: each savefile-relevant
  change (schema fields, write behavior, `.4ccs`/`.4cct`/Texport formats) is reflected in
  `pes_savefile` and its plans or recorded as not applying, before any Phase 5 writing code, and
  the commit reached is in the log

### Phase 16 — Polish and distribution

Steps are itemized when Phase 15 closes; one is fixed already:

- [ ] 16.x `dds_convert` GPU BC7 (deferred from 2.5b to Phase 4, decision entry 2026-09-21, then
  past Release 0.1.0, decision entry 2026-09-28; spec `libs/README.md` "First-release desktop GPU
  BC7"): `block_compression`'s wgpu backend on a Vulkan/Metal device, CPU fallback with the
  fallback reason reported, cold pipeline creation and upload/readback measured first, then the
  texture step's bounded batches → verify: GPU and CPU outputs decode within the same tolerance
  on the `dds_convert` fixtures; the fallback path exercised by forcing no adapter

---

## Issues

Bugs, unexpected behavior, things to revisit. `open` / `resolved (date)`. Resolved issues are
pruned when their phase closes; they stay in git history.

- open — the headless egui test harness (a frame with AccessKit on, a node by label, a click as
  press and release frames) exists twice, in `studio_core/src/shell/mod.rs` and
  `team_compiler/src/view/mod.rs` (3.y design sweep). The next tool view in Phase 8 would be
  the third copy: extract it then into `studio_core` behind a test-support feature, rather
  than a third copy.
- resolved (2026-10-02) — the VPS mutation half's memory peak reached 7.07 GiB of the 8 GiB cap
  on `team_compiler` (3.y whole-crate run, 2 build jobs); no build was killed, and a killed one
  now fails the run. The maintainer keeps 8 GiB. (The 37 OOM kills of 2026-10-02 04:00 were the
  3.z run under the old 6 GiB cap, the reason it was raised; none since.)

- open — u16 face indices cap a reassembled split mesh (found at 2.20f review): `fmdl::Mesh`
  and the IR (`ir.md` "IR struct") store faces as `[u16; 3]`, so `fmdl::ops::split::decode`
  refuses (loud `VertexMismatch`) an add-on file whose components together reference more
  than 65536 vertices, and vertex-limit splitting of an IR mesh can only move loose vertices.
  The 2.20h conversion census found 114 real files over it. Since 2.20h such a group stays
  split rather than erroring (`conversion.md` split rules), so nothing refuses them now. The
  face type is still Phase 7's decision (glTF brings u32 indices).
- open — texture names and case on Fox (found at 4.4b2): stems now compare case-folded when a
  player's sources share one, but `processing/model.rs` still points a model's texture path at the
  player's folder only when the referenced name matches a stem exactly (since 4.3), so a model
  naming `Shirt.dds` beside `shirt.dds` keeps its original directory; the written `.ftex` keeps
  one source's spelling while each model keeps its own. Whether the game's CPK lookup is
  case-insensitive is unmeasured; measure it (or fold both sides) before 4.20 opens every export.
  Also: per-stem conflict decisions are independent, so with two conflicts in one folder a
  package can be dropped over a stem whose winner is a package already dropped for another stem
  (rare; needs two differing stems across three packages).
- open — `model_convert` converge questions (2.20): (1) Fox decal shaders (`translucent`,
  `3ddc`, `eyeocclusion`) infer `Shaded` *exact* through the `3ddf` rule, so the highneck
  fixture converts to an opaque `Basic_C`, where the 19to16 converter wrote `Overlay` with alpha
  blending; the format plan deliberately gives decals no family — decide whether the rule should
  at least mark them approximate. (2) Hand split: a face inside both hands' selections goes to
  both gloves; unused materials/textures are not pruned from the split parts. (3) `retarget`
  returns `ConvertError::Validation` after mutating the IR (no rollback). (4) `.model` tangent `w`
  is 1.0 on import (plan bullet); the bitangent-derived handedness is untested in game.
  (5) A `Metal` material without an `Environment` texture exports `Basic_CNSR` with no
  `EnvironmentMap` sampler; the format plan assigns the template-cubemap fallback to the
  compiler's texture step (Phase 4), which must add the sampler, not just the file. (6) Konami
  FMDLs carry a 64-byte-per-bone `bone_matrices` block the IR does not; the add-on writes it
  empty and its files work in game, so the export drops it with a finding; what the block holds
  and whether it is derivable from `Bone.matrix` is unmeasured.
- open — `pes_savefile` PES 21 team record: (1) the reference's colour bits read zero for 203 of
  220 teams of the 4cc save, and the PES 17 save's colours for the same teams appear at no 6-bit
  offset anywhere in the 21 records, so the save most likely carries none (kit configs supply
  colours on 19+); the reference's 21 colour positions are unconfirmed, not disproven. Check what
  4ccEditor displays for that save before the Save editor shows colours on 21. (2) The 21 record
  holds a kit-slot block at +48 (`00 40 af 00 | 01 40 af 00 | 80 40 af 00`: numbers 0, 1, 0x80
  with team 701 x 0x40) that the reference reads only on PES 17 (at +28); the 18-21 tables have no
  `KitSlot*` rows. Measure 18/19 and add the rows when the Save editor needs kit bindings.
- open — `pes_savefile` unmodeled player-record bits that real saves set (2.20i,
  `.tmp/save_census/src/bin/gaps.rs` over every census save; retained on write, so nothing is
  lost): PES 18 bits 252-254 (3 bits, set in about a third of records; most likely `Star`,
  3 bits on 19/20, absent from the 18 schema); PES 16 bits 222 and 362 (437 records each).
  Model them when the Save editor needs them.
- open — `pes_savefile` player id versus appearance id (2.20i reviewer): the appearance
  block's player id (+116 on 17-19, +240 on 20/21) is unmodeled, so changing
  `PlayerEntry.id` leaves it stale. Nothing changes an id today. Step 4.0 settled what the field
  does (-1 = the player's appearance comes from the database tables), so the model gains it with
  the Phase 5 work that strips and preserves it (decision entry "aesthetics travel in the
  database tables").
- open — FoxDen's PES 2021 gloves trampoline discards `GloveList.bin` (step 4.0, runs 4-7):
  `build_botch_21` in `Tools_4cc/FoxDen/scripts/01_patches.lua` (and the patched exe it
  reproduces) loads the gloves id from `[rsi+0x2C]` for outfield players and goalkeepers,
  overwriting the value the original code chose in `[rsp+0x70]`, which for a player whose save
  appearance id is -1 is his `GloveList.bin` row (the stock exe showed it; the patched exe showed
  bare hands). Request for FoxDen's maintainer: keep the original value when the player has no
  save appearance record, for outfield players too (p1 routes them through the goalkeeper path),
  then an in-game retest with a `GloveList.bin` row for an outfield and a goalkeeper player.
  The Team compiler's gloves output on Fox depends on it.
- open — `pes_savefile::ops::fpc::is_fpc_player` finds no FPC player in a real cup save (step
  4.0 survey of the 2023 Winter Cup save): FPC teams there give each player a blank boots and
  gloves id of his own (`/a/`: 126/126 ... 148/148) rather than 55/11, which its doc comment
  already calls indistinguishable. The Save editor cannot use it to tell FPC teams apart; a hide
  strip (long sleeves, tucked, short socks) plus boots no other player wears marked them in the
  survey (`.tmp/apptest`, `fpc` subcommand).
- **Texport write is unverified in-game** (2.17h): `Texport::new` synthesizes 18-21 files from
  measured templates and `to_bytes` rewrites read files; both round-trip byte-identical, but no
  generated file has been imported by the game yet (`verification.md` "Texport write": manual,
  per version; a `new` file and an edited round-tripped one). PES 15/16/20 texports have no
  fixture at all (offsets/key index are the reference's; PES 20's size is derived).
- open — FoxDen's LiveCPK gaps that sideload mode depends on (forwarded by the maintainer to
  FoxDen's maintainer, 2026-10-03; source `Tools_4cc/FoxDen/src/livecpk.c`, `patterns.c`):
  (1) the call sites are found on PES 2018 only, 2019 and 2021 still to locate; (2) PES 2020 is
  not recognised (proxy only); (3) a served path is limited to about 99 characters (redirect plus
  name in the read request's 0x80-byte field; a face texture's folder alone is 59), longer names
  silently fall back to the CPK; (4) only boot-time shader loads are verified in game, model,
  texture and kit-bin loads under `livecpk\` are not (Sider's buffered-copy and CPK-size hooks
  were not ported). Phase 4's sideload acceptance waits on them; recheck FoxDen when itemizing.
- open — a run that does not end cleanly leaves its staging (3.z): closing the window while
  the GUI's compile runs ends the process mid-run, and the coordinator's `Cancelled` path does
  not discard the staging either, so `output/.staging/<run>/` can be left beside the previous
  CPK (which stays intact). Fixed with cancellation (`gui.md` "Cancellation", Phase 8): the
  window stops the run and the cancelled path calls `deploy::discard`. The same step adds the
  check between a texture task's textures (4.6's slice (c)); a test cancels a real run partway.

---

## Log

Dated, newest last, three lines at most: what changed and what it means for the next session.
No rationale (→ plan), no decisions (→ `DECISIONS.md`).

- **2026-09-07** — Worklog created alongside `AGENTS.md` and `DECISIONS.md`. Planning phase
  near completion; no code yet, no git repository yet.
- **2026-09-10** — Methodology pass: code style reframed and extended, logging and lint decided,
  toolchain pin, `GLOSSARY.md` added, acceptance/converge/rewrite steps added to every phase. Still
  no code and no repository; next is Phase 1.
- **2026-09-10** — First duck review (`gpt-astra-high`) on the methodology docs: 3.5 min
  wall-clock, 7 concerns, 6 accepted and fixed (step-local verification, converge scope vs.
  deferred plan items, Phase 8 acceptance, converge-script modes and scan scope, reactive-review
  exemption); 1 is a real plan contradiction awaiting a user decision (`reqwest` vs. "no async
  runtime" — see `DECISIONS.md`). Nothing it raised had been caught by any prior pass.
- **2026-09-11** — Sessions move to Fusion (Claude lead + SWE-2 sidekick). Sidekick probe: a
  `TeamName` crate implemented cold from `CONTRIBUTING.md` + `team_compiler/README.md` §"Export display
  name and team name" in 3.5 min wall-clock, red-first, all gates green, style rules followed
  literally, two real plan gaps reported rather than decided (leading separators; Unicode vs.
  ASCII lowercasing — the latter is a genuine open point for `teams_list.tsv` matching, carried to
  Phase 3). Second-opinion checkpoints redrawn for three roles (`AGENTS.md`). Phase 1 starts in
  Fusion from step 1.1, with the lead hand-writing the gate infrastructure only.
- **2026-09-12** - Plans for the Refs arranger (Fox hook lists), aesthetics patch, Team creator
  (+ `libs/team_widgets`) written; license settled (`MIT OR Apache-2.0`, Konami data excluded).
  Repository started: first commit replaces the 2023 placeholder history on `the4chancup/4cc-studio`.
- **2026-09-13** - Phase 1 steps 1.1–1.5: workspace, gates, CI file, `vtree` (sidekick),
  `pes_version` and non-GUI `studio_core` (lead), 30 tests, all gates green locally. Two plan
  gaps decided by the user (`PesVersion` home, `unicode-normalization`); font licenses allowed.
  Parallel lead/sidekick work in one tree tripped once on a half-declared module; rule added to
  `AGENTS.md`.
- **2026-09-13** - Phase 1 converge and rewrite done (7 reviewer concerns, all fixed; 35 tests,
  gates green). Converge order flipped to own-audit-first in `AGENTS.md`. Phase 1 closes once the
  first push produces the two CI runs step 1.2 asks for; then Phase 2 starts at 2.1 `wezlib`.
- **2026-09-13** - Second reviewer pass on Phase 1 (5 concerns, all fixed; 38 tests). Commit
  subjects are Conventional Commits from here on; `just` stays on PowerShell for Windows.
- **2026-09-13** - Phase 1 closed: CI green on the first push, red on the planted warning,
  reverted. Phase 2 starts at 2.1 `wezlib`.
- **2026-09-13** - 2.1 `wezlib`, 2.2 `cpk` done (writer parity with pes-file-tools proven);
  fixtures for `fpk`/`ftex` extracted with provenance READMEs; `weszlib` renamed `wezlib`.
- **2026-09-13** - 2.3 `fpk`, 2.4 `ftex`, 2.8 `uniparam`, 2.11 `fpc`, 2.12 `teams_list`, 2.13
  `kit_config` done; 97 tests workspace-wide. The `resources/*.wikitext` pages, committed empty
  in the first commit, were restored from the IDE's local history. Two methodology additions in
  `AGENTS.md`: no legacy tool names in code; the sidekick's report must list every error it hit
  and its fix. `dds_convert` (CPU) briefed.
- **2026-09-13** - 2.5 `dds_convert` (CPU), 2.6 `fmdl` and 2.7 `pes_model` (all but
  `ops::merge`) done; reviews A and B closed (2.5c, 2.12b, 2.13b); 262 tests workspace-wide. Two
  censuses recorded in `libs/README.md` (2610 `.model`, 945 `.mtl`). Safeguards for scripts and sidekick
  trees added to `AGENTS.md` after a truncated source file.
- **2026-09-13** - `resources/prefox_model_format.md` written for the plugin author from the
  census. Merge home decided by the user: native `pes_model::ops::merge`; six plan passages and
  `AGENTS.md` updated, decision logged. Next: implement 2.7d-4, then 2.9 `fox2`.
- **2026-09-13** - 2.7d-4 `pes_model::ops::merge` (bone tolerance measured over 2606 files),
  2.9 `fox2` (87-file census, four fixtures, reviewer pass), 2.10 `archives` (`sevenz-rust2`,
  `zip`), 2.14 `color_tools` extraction (134-kit harness), 2.15 `elevation` done; 329 tests
  workspace-wide, 18 crates on the wasm32 gate. Next: 2.16 `model_convert`.
- **2026-09-14** - 2.16c `model_convert::formats::fmdl` + `loss.rs` done (one rework: the
  vertex-loop encoder reorders Konami vertices, so round trips compare against the decoded input
  re-encoded); `to_fox`/`to_prefox` family defaults only without a native table; dummy maps only
  for family-derived materials. Bone-order census over 2606 `.model` files for 2.16d: 44 list a
  child before its present render parent. Next: 2.16d `formats/pes_model.rs`.
- **2026-09-14** - 2.16d `model_convert::formats::pes_model` done; two censuses behind it
  (`.model` bone order over 2606 files, `.mtl` entry order over 941 files) and the `.model`
  matrix layout checked against PES17 `body.skl` on the fixtures. Retargeting plan section
  sharpened (standard/custom rule, fold mechanics, no-op guarantee); fold literals measured for
  its tests. Next: 2.16f `skeletons/retarget.rs`.
- **2026-09-14** - 2.16 `model_convert` complete: 2.16f retargeting, 2.16g hand split (Blender
  5.2.1 reference fixture), 2.16h routing with the legacy 19to16 reference, 2.16i cross-family
  review (six of seven concerns fixed). 93 crate tests, 422 workspace-wide, wasm32 gate on 19
  crates. Converge questions in "Issues". Next: 2.17 `pes_savefile`.
- **2026-09-14** - 2.17a `pes_savefile::container` done: nine real saves decrypted in the census
  (15/16/17/18/19/21; no 20), slice fixtures plus zlib payloads committed, discovery table
  corrected (PES 18 is flat). Field tables for 2.17b derived by symbolic interpretation of the
  reference read walks and checked against every payload (method in the plan; the
  script survives as `scripts/derive_savefile_schema.py`). Next: 2.17b schema + codec + `PlayerEntry`.
- **2026-09-14** - 2.17b–d done: generated schema tables (`scripts/derive_savefile_schema.py`),
  codec, player/team models, `EditFile`, `display_name`, discovery; 37 crate tests; all nine real
  saves lossless through `EditFile`. Stopped before 2.17e for two user decisions: the
  `settings.toml` key table (export text format) and the ingame-face run, which no legacy tool
  decodes beyond eleven feature types (converters) and a few colour bits, so `PlayerSettings`
  cannot cover "every appearance field" without an opaque-bytes model of that run.
- **2026-09-15** - Method review against "Why Software Factories Fail" (humanlayer). `AGENTS.md`: briefs sized for one review (about 500 lines, slices otherwise); red-run evidence per new test in the brief's closing section; the review sweep split into an honesty sweep and a design sweep; converge gains a design-health pass. Phase 3 tracer-bullet question recorded as step 3.x.
- **2026-09-15** - User decided: Phase 3 opens with a tracer bullet (step 3.3; `core/development_plan.md` "Phase 3" first bullet; decision entry). Early `studio` shell stays an open question (3.y).
- **2026-09-15** - User decided: Phase 3 also closes with a minimal `studio` shell (step 3.z; `core/development_plan.md` "Phase 3" last bullet, "Phase 8" note; decision entry).
- **2026-09-19** - Reviewed 4ccEditor's `Tactics` (`4a95b7c`) and `Autumn_2026_AATF` (`cf61542`)
  branches against the plans. Schemas 15-20 match the new tactics decoders offset for offset;
  `pes_savefile/README.md` gained the Texport `.ted` 18-21 crypto and per-version offsets (for 2.17h),
  `.4cct` layout and the canonical advanced-instruction mapping. `save_editor.md` AATF section
  moved to the Autumn 26 ruleset (bronze tier, specials, two upstream errata recorded for the
  user to report upstream) and, by user decision, to one self-contained Rhai rules file (decision entry). No code.
- **2026-09-19** - Mutation testing adopted after a probe on three closed crates (decision
  entry): `just mutants <crate>` at converge, `just mutants-diff` at every diff review. Review
  round C (2.19a): the probe's survivors plus a workspace read-through fixed in six commits;
  the whole-crate runs on cpk/ftex/dds_convert left 138 survivors for 2.20. One brief premise
  was wrong and the gates caught it: "SKL parents precede children" is false for the game's
  `body.skl`, so the new parent check compares against the bone count. `AGENTS.md`: briefs name
  the `karpathy-guidelines` skill and the consumer crates' tests; `drop(` joins the honesty
  sweep. Next: 2.17e.
- **2026-09-19** - Housekeeping: the session scratch heap is gone and the scripts it held that
  the plans and fixture READMEs cite live in `scripts/provenance/`; the seven plans over a
  thousand lines are folders (`plans/README.md` "How these documents evolve", decision entry);
  the spec stays in `docs/plans/` rather than moving into crates. Pointer-only change, checked
  over every tracked Markdown file. Next: 2.17e.
- **2026-09-19** - User decided the two 2.17e questions: the `settings.toml` key table (one key
  per line, range comment on the same line) and option A for the ingame-face run (opaque bytes
  with typed accessors, "every decoded field"); ranges verified against the reference editor's
  lists and the converters' caps. 2.17e split into three slices; e-1 briefed next.
- **2026-09-19** - 2.17e done in four commits (`58c37af`, `51dea0e`, `eef7e6d`, `8b2d864` plus
  the review rework): the ingame-face run, `PlayerSettings` with the key table and the TOML
  half, `ops::fpc`; checkpoint (b) review closed. Next: 2.17f `convert.rs` (the 46/50-byte run
  padding rule is its first decision).
- **2026-09-20** - 2.17f done in three commits (`b9d5535` plan, `da6e60b` playstyle lists +
  caps, `17d011c` `convert.rs` plus the review rework): canonical `PlayStyle` with the reference
  editor's twelve arrays as golden data, `face_type_cap`, `convert_player` as a rewrite into a
  target-version template (prefix copy of the ingame-face run, `min(len)` bytes). Checkpoint (b)
  closed, one plan sharpening (what is and is not a `ConvertNote`). Next: 2.17g
  `ops/{transplant,fingerprint,compare}`.
- **2026-09-20** - 2.17g opened: the reference scripts read (transplant = block bytes 4..68,
  compare = fifteen fields plus a masked SHA-256 prefix; the reference editor's comparator =
  roster-slot pairing over the gameplay fields); the run's tail bytes measured zero on every
  16+ fixture player, so the transplant copies the block whole (decision entry). Plan section
  written with the API blocks, two golden tests written, 2.17g split into two slices.
- **2026-09-20** - 2.17g done (`806adcd` plan + goldens, `74136aa` transplant/fingerprint,
  `7a7ab4a` compare, plus the review rework): the transplant copies the appearance block whole
  (tail bytes measured zero), the comparator walks the schema's stored fields and tags rows by
  scope, `FaceRun` fires only for undecoded bits. Checkpoint (b) closed. Next: 2.17h
  `interchange/{team_toml,legacy,texport}`.
- **2026-09-20** - PES 20 verified on two real saves (a 4cc invitational's day-0 save and the
  game-generated one): key, header, discovery layout, section offsets and the 20 tables all hold;
  both rewrite byte-identical with their own salt; `compare` between them finds exactly the 414
  named players. Day-0 save is now the `pes20` fixture, in `FIXTURES` (every fixture-wide test
  runs on it; two non-vacuity floors carry its measured thinness). PES 20 open issue closed.
- **2026-09-20** - pes-db-generator planned into the suite: `db_generator.md` (tool, Phase 19,
  scheduled by need), `libs/pesdb` (the Konami table layouts; Ball/Stadium bins move there),
  `pes_savefile::ops::populate` (the 19+ player-section fill). Decision entry. Next: 2.17h.
- **2026-09-21** - 2.17h done (`6122b09` plan, `3575b19` fixtures + goldens, `8385437` texport,
  `2d9365c` instruction + Team TOML team half, `ceb9b6f` player half, `1ae6e16` legacy,
  `28e2eab` review rework): the interchange formats measured on the real files (texport 18-21
  is the save's records concatenated; PES 20/21 store a 17th instruction), Team TOML as the one
  import path with schema-gated notes and stored ranges, `.4ccs`/`.4cct` readers. Crate 113 → 181
  tests; four decision entries. A flaky `file` test fixed on the way: PES 15's one-byte seed made
  `assert_ne!(saved, original)` fail once in 256 saves; the test now saves a PES 16 fixture.
  Next: 2.18 `python_bindings`.
- **2026-09-21** - 2.18 done: the `pes_models_native` wheel (codec `read`/`write` only, the Blender
  accessors wait for the extension's hot-path step), `just bindings`, the CI job deferred from 1.2.
  Mutation runs capped like the builds (`.cargo/mutants.toml`: two mutant processes at 7 build
  jobs / 7 test threads each). Next: 2.19.
- **2026-09-21** - Review rounds are now bounded by accept rate (`AGENTS.md`), and the rule applied
  retroactively to 2.17h: a second round found seven more real concerns (2.17h-6), the largest a
  15-17 texport that wrote a reordered squad and reopened short; a third found three (2.17h-7),
  all the same class (a value `apply` accepted and the codec refused at write), and stopped the
  loop. Next: 2.19, then 2.20 with the bounded loop per crate.
- **2026-09-21** - 2.19 done: gates, deps-check and bindings green locally at `43b4ccb` and in CI
  on both platforms (658 tests over 40 crate binaries). User decisions: 2.5b GPU BC7 deferred to
  Phase 4 (decision entry); 2.20 runs per crate with the bounded reviewer loop, the tiny leaves
  batched into one reviewer round, `pes_savefile::interchange`'s reviewer half counted as done by
  2.17h's three rounds. Next: 2.20, tiny leaves first.
- **2026-09-23** - 2.20c `cpk` done. Method finding: the reviewer's first round returned five,
  and its reasoning trace (read by the user) showed it aiming for "3-5 strong concerns" because
  the brief said "do not pad"; a second round run as an experiment returned three verified
  concerns, all accepted (row-count allocation with a zero row length, contradicting a lead
  ruling; a `Tvers` long enough to overwrite the first file; NUL paths colliding in the string
  pool). `AGENTS.md` "Second opinion" now continues on the accept count alone (five or more
  accepted, whatever was returned) and bans "do not pad" from the brief. Next: 2.20d `ftex` +
  `dds_convert`.
- **2026-09-26** - 2.20d `ftex` + `dds_convert` done after six reviewer rounds (7/7/7/7/5/4
  accepted); the loop closed on the accept-rate rule with no ceiling. Method finding: a reviewer
  resumed from the previous round found a strict subset (4 of 7) of what a fresh one found on
  the same surface, so reviewers stay fresh (`AGENTS.md` "Second opinion"). Next: 2.20e `fox2`.
- **2026-09-27** - 2.20e `fox2` done after three reviewer rounds (5/5/2 accepted). The
  maintainer ruled that inputs come from a trusted environment, so concerns that need a crafted
  file are rejected (`AGENTS.md` "Fundamental concepts"); two of round 1's were. Finding: the
  reference is not always the oracle: its 33-64 byte hash leaves a sum unmasked, so for
  non-ASCII text we follow CityHash (the game's hash) and pin it with goldens that assert the
  disagreement. Next: 2.20f `fmdl`.
- **2026-09-27** - 2.20f `fmdl` done after four reviewer rounds (7/5/7/4 accepted). The
  maintainer asked that a lead recommendation be applied and logged rather than asked
  (`AGENTS.md` "When the plan has gaps"). Finding: every round found real defects in paths no
  fixture exercises (split output that did not reload, a hang, panics); `pes_model` is the
  `fmdl` port, so its audit starts from this list. Next: 2.20g `pes_model`.
- **2026-09-27** - Method review after 2.20f (`AGENTS.md`): whole-crate mutation runs at the
  start and close of a crate's converge only, `mutants-diff` for the rework rounds between;
  reviewer briefs rank concerns by reachability; briefs tell the sidekick to insert tests
  inside the test module. Maintainer decided the executable is `4cc-studio` (decision entry).
  `fmdl`: the combine reorder gate's `>`/`>=` survivor was a missing test, now written.
- **2026-09-27** - Maintainer decisions: `archives` refuses password-protected archives at open
  (a zip with any encrypted entry, a 7z with an encrypted header or an AES-coded block), so the
  live check reports them at once; maintenance mode is a Phase 16 deliverable with its outline in
  `core/development_plan.md`, not per-file ADRs (decision entries).
- **2026-09-27** - `just mutants <crate>` splits the run with the maintainer's VPS when
  `STUDIO_MUTANTS_REMOTE` is set (`scripts/mutants.py`; `AGENTS.md` "Environment" has the
  benchmark). Verified on `archives`: the split's totals equal a local run's (24 caught, 6
  unviable). Its first whole-crate use is 2.20g's `pes_model` run.
- **2026-09-27** - Maintainer decisions: `fmdl`/`pes_model` `check` flag vertices more than 5000
  units from the origin (Team compiler: `vertex_too_far_from_origin`, folder dropped); code no
  longer names or alludes to the legacy tools (`CONTRIBUTING.md` rule broadened, design sweep
  entry in `AGENTS.md`), swept across 44 files; parity tests keep their tool names.
- **2026-09-28** - Release target set (maintainer): 0.1.0 after Phase 8 (Windows, CPU textures,
  native models, updater check + notice); Phase 7 and GPU BC7 (now 16.x) move after it; no
  Pre-Studio input path. Plan-only change (decision entry). Next: another plan change, then 2.20g.
- **2026-09-28** - `settings.toml` gains top-level stock `boots_id`/`gloves_id` (0 to 100, `""` =
  default: FPC marker or savefile decides); plan-only change across ten documents (decision
  entry), code step 2.17i added. Test 1 files for the appearance-fallback idea are in
  `.tmp/apptest/out/`, awaiting the maintainer's in-game run.
- **2026-09-28** - Hard gate set (maintainer): step 4.0, the appearance-fallback test, stands at
  the end of Phase 3; no agent goes past it until the maintainer reports the result. Phase 2's
  remaining converge, 2.17i and Phase 3 proceed. Next: 2.20g `pes_model`.
- **2026-09-28** - 2.20g `pes_model` done (one reviewer round, 4/4 accepted). Method finding:
  the Konami-measured reader refused about 1900 of the machine's 6037 community `.model`
  files, cup exports among them, and no fixture, mutation run or reviewer would have found
  it; a census of every real file of the format now belongs in each format crate's converge
  (`.tmp/model_census/` as the template). Next: 2.20g-fmdl census, then 2.20h.
- **2026-09-28** - 2.20g-fmdl census and 2.20h `model_convert` done. The fmdl census found only
  SKL tails, and every other failure was a correct refusal. The conversion census found about
  one failure in six (weights, split groups, bone order, flags, our own empty containers),
  each traced by measurement before a rule was written. Reviewer rounds 7/3. Next: 2.17i, then
  2.20i `pes_savefile`, opening with a savefile census.
- **2026-09-28** - 2.17i done. 2.20i started: the savefile census found three text-field
  classes that read but do not round-trip (full-length names with no NUL on PES 16/17, and a
  CP1252 byte in a PES 17 name). The session was stopped on the maintainer's request during
  the whole-crate mutation run (partial results in the 2.20i step). Next: decide the text
  rules, fix, rerun `just mutants pes_savefile`.
- **2026-09-28** - Maintainer rulings: the CP1252 save stays refused (4ccEditor cannot load it),
  the 21845-face limit stays. Text fields hold their full length (decision entry). Test 1's
  step-by-step guide is `.tmp/apptest/out/GUIDE.txt`. Next: the 2.20i text fix.
- **2026-09-28** - 2.20i text fix landed (`40a07be`); census clean. Whole-crate run done
  (24 survivors, `.tmp/mutants_2_20i_whole/missed.txt`), the crate's last (maintainer). Lead
  audit in `.tmp/audit_2_20i.md` (F1-F7). Next: slice A = survivors + F1-F4, F6, F7; slice B =
  F5's pure moves; each checked with `mutants-diff`; then the reviewer.
- **2026-09-28** - `just mutants` fixed: the remote half runs detached and survives a dropped
  link or a killed controller (`just mutants-collect`), and shells without
  `STUDIO_MUTANTS_REMOTE` read it from the registry.
- **2026-09-28** - Maintainer check of the full-field shirt name (`EDIT00000000 prespoon`,
  PES 16): 4ccEditor shows `MEAT ON THE BON`, and the save crashes PES 16 after the start
  screen, cause unknown (the maintainer doubts the shirt name). `shirt_name_from` keeps its
  free byte. Next: 2.20i slice A (`.tmp/brief_2_20i_a.md`).
- **2026-09-29** - 2.20i slice A done (survivor tests, F1-F4, F6, F7; `mutants-diff`: 0
  missed). Correction: the `40a07be` whole-crate run measured 1130 of 1611 mutants, not all
  of them (its VPS half died partway); the maintainer approved one run of the 486 unmeasured.
- **2026-09-29** - 2.20i remainder run: 502 mutants, 4 missed (3 equivalent as written,
  rewritten; 1 test added), `mutants-diff`: 0 missed. Next: slice B.
- **2026-09-29** - 2.20i slice B done: pure moves split `team_toml/team.rs` (tactics, items)
  and `settings_toml/mod.rs` (document), checked by `.tmp/pure_move_check.py`. Next: the
  reviewer.
- **2026-09-29** - 2.20i done. The reviewer's round (3 of 4 accepted) found PES 18's
  dribbling-arm motion stored in 3 bits where the schema read 2; a census of set-but-unmodeled
  bits found no other such field. Overfull rosters and malformed PES 15 descriptions are
  now refused at write. Next: 2.20j `python_bindings`.
- **2026-09-29** - Memory caps after a mutant (`display_name`, `i *= 1`) OOMed the VPS: the
  remote half is a capped, CPU-idle systemd unit, and Windows test binaries run under a 6 GB
  per-process cap (`scripts/test_runner.py`). Both caught the mutant when re-run under the cap.
  A census of a bit-packed format now also tallies unmodeled set bits (`AGENTS.md`). Next:
  2.20j `python_bindings`.
- **2026-09-29** - 2.20j done: the wheel's smoke test, stale since 2.20g-h's fixtures, passes
  again and runs inside Blender 5.0/5.2; mutants measured by hand (11 of 12 caught); reviewer
  5/1 accepted. The VPS is off limits for mutation runs until the maintainer says otherwise
  (gotchas). Next: 2.21, the Phase 2 plan rewrite.
- **2026-09-29** - 2.21 in progress: Phase 2's development-plan section, `model.md`'s model
  blocks and stale library names rewritten. It found a gap, now 2.20k: savefile field values
  were never compared with 4ccEditor's. Next: 2.20k.
- **2026-09-30** - 2.20k done: 4ccEditor's own `.4ccs` exports from the fixture saves equal the
  codec's read on every carried value, PES 15-21. It found `.4ccs` records ordered by save
  record, not roster; `read_squad` now maps by the target's record order. VPS mutation runs
  allowed again (maintainer). Next: the rest of 2.21.
- **2026-09-30** - `just mutants-diff` splits with the VPS when its estimated local time is at
  least 4 minutes, estimated from each crate's measured seconds per mutant
  (`target/mutants-cost.json`); the 2.20k diff's slower half took 502 s against 717 s local-only.
- **2026-09-30** - Phase 2 closed: 2.21 done. An as-built audit of the Phase 2 plans against
  the code (`.tmp/audit_2_21_as_built.md`) fixed stale crate trees, deps rows, two code blocks
  and unmarked Phase 8/19 parts. Step list collapsed. Next: 3.1, Phase 3's Acceptance section.
- **2026-09-30** - 3.1 done: Phase 3's Acceptance section, the entry gates ("Validation
  semantics") and the CLI exit codes, after 8 GPT review rounds and 4 sidekick rounds. Reviewers
  are now uncapped (decision entry). Next: 3.2, the acceptance-ID scanner.
- **2026-09-30** - 3.3 done: the tracer compiles /egg/'s Stormworks player face and kit g1 for
  PES 21 and matches Red's output on every row Phase 3 produces (FTEX by decoded content,
  decision entry). Six Phase 2 frictions listed as 3.4. Next: 3.4.
- **2026-09-30** - 3.4 done: `kit_config::KitSlot`, FTEX headers in the parity comparison, the
  plan's stale `fmdl_id_change` and the CPK tool version settled (decision entry). Next: 3.5.
- **2026-09-30** - 3.5 done: the `aesthetics_export` structure pass shapes written into
  `object_model.md` (decision entry). Next: 3.6 slice (a).
- **2026-10-01** - 3.6 (b) rules settled in `object_model.md` "Validation semantics" (allowlist
  per position, OS artifacts, folder-name split, issue scope, eligibility, drop order, kits/
  portraits/logo details; decision entry, census of 225 OS artifacts in real model folders).
  Slice b1 landed (`mutants-diff`: 81, 59 caught, 15 unviable, 7 missed: 4 got tests, 2 went with a
  redundant roster guard, 2 are the dropped-player filter b1 cannot reach and b2's tests cover). Next: b2.
- **2026-10-01** - 3.6 slice b2 landed: allowlist, links, markers, reserved subfolders, stems,
  Fox `fmdl_name_invalid`, the cascade and pass-through. Review rework: one link resolver, shared
  helpers for drops/strictness/mapped slots. `mutants-diff 0d7d489` (b1+b2): 221, 18 missed, all
  given tests; rerun 198 caught, 23 unviable, 0 missed. Next: b3.
- **2026-10-01** - 3.6 slice b3 landed: kit grammar, duplicates, `all/` inheritance, kit
  allowlist/names/markers/icon, portraits, logo grammar and roles, root allowlist and notes
  (`kit_config` dependency). Review rework: one stem-conflict helper (b2's two copies included),
  shared icon/marker/name checks, an invalid kit head gets no other finding (decision entry).
  `mutants-diff 54e44f0`: 114, 15 missed (14 given tests, 1 equivalent removed by restructuring);
  rerun 111, 97 caught, 14 unviable, 0 missed. Next: the reviewer on all of (b).
- **2026-10-01** - 3.6 cross-family review, GPT round 1: 7 concerns, 4 accepted (fold panics on
  real names → `vtree::fold_name`; flattening panic → insertion-built tree, no root findings on an
  undecided root; texture `.common` links and `Common/` join the stem check; TC-STR-09 uncited
  until 3.8), 3 rejected; the GPT loop ends (rulings `.tmp/review_rulings_3_6.md`; plan + decision
  entry). `mutants-diff 26775b0`: 31, 2 missed, both given tests; rerun 26 caught, 5 unviable, 0 missed. Next: the sidekick
  review loop.
- **2026-10-01** - 3.6 sidekick review S1: 7 concerns, 3 accepted (a kept disallowed `.common`
  file stays in the player's files; no logo findings on an undecided root; allowlist wording for
  `all/` and `Portraits/`), 4 rejected; the loop ends, and with it the 3.6 review.
  `mutants-diff 9030603`: 5, 4 caught, 1 unviable. 3.6 done. Next: 3.7, plan section first.
- **2026-10-01** - 3.7 started: `libs/pipeline.md` written (permit, cancellation, oversized
  priority, the solid-7z charge, thread count, `CpkStem`; decision entry), then the crate's
  budget, thread count and `CpkStem` (11 tests, each group red against a stub; the oversized-
  priority test red against a budget without `oversized_waiting`). The memory cap is a share of
  the memory available at run start, read per OS (`windows` feature approved by the maintainer;
  decision entry); its `memory.rs` is not written yet. Not yet run: `mutants-diff`, the
  cross-family review (queued, see "Handover").
- **2026-10-01** - 3.7 implemented: `pipeline::memory_cap` (Windows `GlobalMemoryStatusEx`,
  Linux `/proc/meminfo` through a `mem_available` parse tested on every platform). `mutants-diff
  ab28300`: 12 missed, 7 given tests (exact-cap request is ordinary, OS read and cap above a
  floor, the 4 GiB fallback, the parse); the 5 left are the Linux and other-platform
  `available_memory` arms, compiled out on Windows (the Linux arm is checked by
  `the_os_reports_available_memory` on Linux CI; the other arm is a constant `None`). 17 tests.
  Next: 3.8; the 3.7 review is queued.
- **2026-10-01** - 3.8 shapes settled (decision entry; plan edits in `core/architecture.md`,
  `distribution.md`, `team_compiler/settings.md`, `messages.md`, `pipeline.md`) and sliced
  a-d. 3.8a landed: `cli_run` returns `Result<u8, CliError>`, `AppPaths`, CLI data location
  (never writes), the binary's console printer. Red: the unknown-tool test failed `left: 1
  right: 2` against a code-1 refusal; the rest is new code. Implemented by a Sonnet subagent
  (handover note), reviewed by the lead. Next: 3.8b.
- **2026-10-01** - 3.8a re-checked at the maintainer's request: `load_settings` moved from a
  `String` error and `current_exe().ok()` to `anyhow` with context; missing why-comments added
  (`BaseDirs`, the printer fallback, the printer-failure exit code, now 3). 3.8b landed: the
  tool skeleton and preflight; red: all 10 binary tests failed `unexpected argument
  'team-compiler' found` before the subcommand existed. Subagents now Opus (maintainer). Next: 3.8c.
- **2026-10-01** - 3.8c landed: `check` over folder sources, the catalog, discovery and
  routing, findings as events. Red: 15 of 22 in-process tests failed against 3.8b's stub (e.g.
  TC-SRC-05 `left: []`, TC-CLI-01 `refused with 3: check is not built yet`). Rulings on the
  subagent's open points: an unreadable root aborts (3); sources are listed before routing,
  so an unreadable one never counts as a refs duplicate; `export_extract_failed` carries
  `path` and `error`; the refs summary precedes the exports. Found: `mutants-diff` skips
  untracked files (fixed in the detour below); cargo-mutants copies the gitignored `.tmp/` into its build
  copies, which filled the disk (3.7's two diff runs died of it). Next: the mutants/`.tmp`
  detour (maintainer), then 3.8d.
- **2026-10-01** - Detour: neither 3.7 diff run had finished (the second reported 11 of 67
  mutants before the disk filled), so its numbers above had no complete run behind them.
  Re-run over `ab28300..3b6741e` (`pipeline` unchanged since): 67, 51 caught, 5 unviable, 6
  timeouts (mutated `MemoryBudget::acquire` wait conditions that block forever), 5 missed, the
  compiled-out Linux and other-platform `available_memory` arms the entry names; run on the
  VPS (Linux, 4c7c424), the Linux arm's 3 are caught, leaving only the other-platform 2,
  which no tested OS compiles. Every other
  Phase 3 run had completed (3.3's and 3.4's survivors triaged in their steps). Fixed:
  `mutants.toml` sets `gitignore = true` (the copies leave out `.tmp/`, checked in the run's
  `debug.log`); `mutants_diff.py` diffs against the working tree written as a tree object
  (`mutants.working_tree`, shared with the remote snapshot), so untracked files are measured
  without staging. `AGENTS.md`: archived runs keep only their summaries.
- **2026-10-01** - Detour done. Cleanup (maintainer-approved): the 10 stale `%TEMP%` copies,
  archived runs cut to their summaries, `bench/`, census and probe build outputs; 63 GB freed,
  `.tmp/` 17 GB to 0.8 GB. `teams_list.txt` marked `-text` in `.gitattributes`: CRLF by its
  crate's contract (a test compares the shipped list with the crate's own write), so the
  remote snapshot's fresh `git add` had stored it LF. Next: 3.8d.
- **2026-10-01** - 3.8d landed: archive sources in `check` (`archives::folders()`, one open per
  source, the 7z read charged per `libs/pipeline.md`). Red: TC-SRC-01/02/06/07/08/09's check
  tests all reported `archive sources arrive in step 3.8d` before the reader change. Lead's
  review: a directory entry naming the root (`./`) skipped instead of refusing the archive
  (decision entry); the brief's "cite every scenario" overrode 3.8's rule, so the tags of
  the four compile-half scenarios were removed (44 proven). `mutants-diff 4c7c424`: 28, 21
  caught, 7 unviable, 0 missed. `team_compiler` now depends on `archives` (planned, in-tree).
  `mutants.toml` notes the `available_memory` platform arms; 3.w (mutants.toml audit) and
  4.0a (PES12 re-study, stash `pes12`) added at the maintainer's request. Next: 3.9.
- 2026-10-02: 3.9a landed after one rework round. The lead's review of the first diff found
  game-facing loosenings (any `.bin`/`.fclo` packed into `face.fpk`, the `fcl_hair.skl` pairing
  check dropped, any image read as DDS), unreadable files reported as `folder_pack_failed`, kit
  scopes rebuilt as `Kits/<name>`, an empty `.staging/` left behind, two struct lifetimes and a
  three-noun `check` module; all fixed (decision entry "3.9a review"). The task's files are now
  read by the coordinator, which 3.9c's pool needs. The slice ran ~600 lines with 14 audit items
  folded in, over budget: later slices carry no unrelated fixes. `mutants-diff 6c254c3`: 146,
  109 caught, 36 unviable, 1 missed: the `.7z` permit choice (3.9c's fixture).
- 2026-10-02: 3.9b landed first time. A design pass (Plan subagent) before the brief found
  the scope's "files a lenient setting keeps do not count" unimplementable (the validated
  export does not mark them); the plan now lists what counts (decision entry "3.9b"). The brief
  settled every rule, and the diff followed it; its one reported gap (a `pass_through`-kept
  `.common` link) is already handled by the lib, which takes such a link off `files`. `mutants-diff 2c1f31e`: 68, 63 caught,
  5 unviable, 0 missed.
- 2026-10-02: 3.9b2 landed first time, tests only: all 17 scenarios passed on the code as built.
  The brief's one wrong expectation was the lead's (a kit with no `config.toml` also reports
  `kit_config_generated`). With `pass_through`, a folder that `shared_link_duplicate` drops still
  shows its `link_target_missing` lines as `[Keep]`: each finding carries its own disposition,
  and the grid's cell (Phase 8) derives the folder's outcome from all of them. No mutation run
  (no `src/` change).
- 2026-10-02: 3.9c landed first time. The sidekick found two brief gaps, both accepted:
  `crossbeam-channel` was only a dev-dependency of `team_compiler`, and draining the channel
  after a write error still left the writer's waiting batches holding permits an oversized
  `acquire` could wait on forever (fix: `submit` clears them; red-first test). A write error
  drops the messages of batches not yet reported; the run aborts on that error anyway.
  `mutants-diff 1306cad`: 16, 10 caught, 5 unviable, 1 timeout (the `.7z` permit choice
  `==` → `!=` deadlocks under the over-cap test, as it should: 3.9a's survivor is gone).
- 2026-10-02: 3.9d landed first time, the first slice by a Fable sidekick (maintainer's trial).
  Its three reported brief errors were all right (no catalog `F` kind existed; TC-OUT-03's
  chain names the face's texture, the first duplicate the writer meets, not `face.fpk`;
  `promote` reuses `discard` whole), and it caught one the brief missed: the writer's
  `CpkWriter` would still hold the staged file open when a failure removed the folder.
  `mutants-diff cb6f0cf`: 12, 10 caught, 2 unviable, 0 missed.
- 2026-10-02: 3.9e landed first time (Fable), closing 3.9 at 73 of 75. A kit folder holding only
  `config.toml` is a placeholder kit too, so one older planning test gained a `kit_placeholder`
  line. Lead fixes: the help paragraph's "It skips" read as the kit folder after the new
  sentence; a second CPK-reading helper merged into `cpk_entries`. `mutants-diff 9072f79`: 11,
  10 caught, 1 unviable, 0 missed.
- 2026-10-02: 3.z landed after one tests-only rework round (Fable): its first diff passed review
  as written, and `mutants-diff` left 11 survivors in the window code. Eight are now proven
  headless (`egui::Context::run_ui`, AccessKit bounds for clicks, `eframe::Frame::_new_kittest`);
  `run_gui`/`run_gui_mode` open a native window and are excluded (`mutants.toml`, TC-GUI-01
  proves them); the lead added the test that a poll never blocks on a run in flight. Final
  local `mutants-diff 44c284f`: 38, 27 caught, 7 unviable, 4 timeouts (each a mutant that
  never ends the run), 0 missed. `egui` raised to 0.36.2 with `eframe`. Manual TC-GUI-01/02
  pending the maintainer.
- 2026-10-02: mutation tooling (maintainer's report and idea): `mutants-diff`'s estimate now
  adds the measured baseline and averages every outcome (it said 80 s for a 270 s run; studio
  and studio_core had been counted as one crate, `src`); local runs start at below-normal
  priority with half the CPUs per job; a build killed for memory is listed and fails the run
  instead of passing as unviable (12 remote mutants of 3.z, eframe's tree past the 6 GiB cap).
- **2026-10-02** - 3.9f: the maintainer's TC-GUI-01/02 run passed; with no exports folder it
  printed a raw OS error, so the relative exports folder is now created before a run, a missing
  one named any other way is refused in a sentence naming the path and the setting (exit 2),
  and an empty one reports `no_exports_found`. Red runs: TC-CLI-08 exit `3` against `2`;
  TC-CLI-09 and its `check` half "the folder was created"; the `cli.rs` unit tests against an
  unconditional `Ok(root)`. Fable: landed first time.
- **2026-10-02** - 3.w: `.cargo/mutants.toml` audited entry by entry (row 3.w); five entries
  were wrong or too broad, the config now removes 132 mutants instead of 150, and three
  `model_convert` tests kill the overlapping ORs the old entry hid. Red runs (mutation applied
  by hand): `to_fox.rs:148` `left: 0 right: 32`; `pes_model/export.rs:129` `left: [] right:
  ["no_shadow_cast"]`, `:134` `left: [] right: ["invisible"]`. Fable: landed first time and
  disproved the lead's `quantize_weights` counterexample with a 6-million-input probe.
- **2026-10-02** - 3.y converge, the reviewer loop aside (row 3.y). Rejected sweep findings: the
  headless egui harness's second copy (the third, in Phase 8, extracts it: Issues), the two
  `SmallMetadata` lookups (the parse stage has raw keys, validation only canonical ones), the
  two `Disposition` enums (libs do not depend on `studio_core`; `messages.rs` is the seam),
  `PLACEHOLDER_KIT.to_vec()` and the kit config copies (owned input, small), "legacy" for
  `refs.txt` (the format's history, as `messages.md` says it), `ctx` (the plan's trait
  signature), the plan-named or plan-deferred `pub` items, `Tool`'s `Default` beside `new`
  (clippy's `new_without_default`). Red runs (mutation by hand): `Settings::load` guard
  `matches!(…Io(error) if error.kind() != NotFound)` failed; `gui_run` default `unwrap_err()
  on Ok`; `version_change_blocker` `left: Some("xyzzy") right: None`; `ToolContext::common`
  `left: 0 right: 4`; `exe_dir` exit `Some(0)` against `Some(2)`; `install_cli_logger` `left:
  Off`; `log_level` arm `left: Trace right: Info`. `mutants-diff b6d898c`: 58, 49 caught, 9
  unviable. Fable: landed first time; a name it could not use as briefed (`position` shadows
  the function) renamed `file_position` and reported.
- **2026-10-02** — 3.x: Phase 3 closed, its cross-family reviews queued. The plan's Phase 3
  sections describe what exists (`development_plan.md` "Phase 3" with its done Verification:
  1402 tests, 77 of 77 scenarios); the step list collapsed. Next: 4.0a, 4.0b, then stop.
- **2026-10-02** — 4.0a and 4.0b reported. PES 12: pes12tools reads the old export layout, has no
  license and reshapes daily; keep the stub stashed. Sider: LiveCPK matches the planned Sider mode;
  a launcher over the user's own Sider is recommended. Both await the maintainer's decisions.
- **2026-10-02** — 4.0c: `Engine` is matched exhaustively everywhere (Fable, landed first time,
  0 lead fixes). Sideloader decisions recorded in the lead's state; study part 2 running.
- **2026-10-02** — 4.0b closed: sideload mode and the `overrides/` folder renamed in the plans,
  a plain Launch PES button planned; 4.0d renames the code. Test 1 (step 4.0) installed for the
  maintainer's in-game check, backups beside the save and `DpFileList.bin`.
- **2026-10-03** — 4.0e: sideloading goes through FoxDen's LiveCPK into the game folder's
  `livecpk\`; no Studio sideloader, the untracked stub deleted, PES 15/16 refused. FoxDen's gaps
  (2019/2021 sites, 2020, path length, in-match loads) are under "Issues". Still blocked on 4.0.
- **2026-10-03** — CI's `deps-check` red since 3.z (`a691220`): `eframe` → `arboard` brought
  `clipboard-win` and `error-code` under `BSL-1.0`, and Phase 3's close never ran the recipe.
  `BSL-1.0` allowed; compatible licenses join without asking, and `deps-check` runs right after
  any `Cargo.lock` addition (decision entry). `just deps-check` green locally.
- **2026-10-03** — `(maintainer, <date>)` notes removed from the plans and `AGENTS.md`; the four
  with no decision entry got one, and `AGENTS.md` keeps them out (decision entry).
- **2026-10-03** — Step 4.0 done: seven in-game runs on PES 2021 (the first, on the June save,
  failed on a corrupted `SYSTEM00000000`; the maintainer moved to the 2023 Winter Cup save and
  DLC). Database tables are read for players whose save appearance id is -1; gloves need a
  FoxDen change ("Issues"). "Target versions" added to `development_plan.md` (Fox first,
  game-behavior changes through FoxDen) and the aesthetics design recorded (decision entries).
  Next: the plan rewrite those entries imply, then Phase 4's itemization.
- **2026-10-03** — 4.1: Phase 4 itemized (4.2-4.28 plus 4.y, converge, rewrite) and its
  Acceptance section written (126 scenarios); 27 plan gaps found, the lead-decidable ones ruled
  (decision entry), the rest under "Phase 4 open questions". A VGL26 corpus of 53 old-layout
  exports (`C:/Data/4cc/Lab/Gud`, 15 GB) is the converge census's input once Phase 6 upgrades it.
- **2026-10-03** — Phase 4 pre-step plan text (decision entry "Phase 4 pre-step measurements"):
  portraits pass DDS through, the stock collar sets per version, logo sizes, the `colors.txt`
  grammar, referee paths, the blank face folder and the DpFileList layout, measured on the PES
  2021/2017 installs and Red. The sidekick's three contradictions answered (trailing icon kept,
  `4cc_90_test` on PES 21 asked, cross-engine collars at 4.9).
- **2026-10-03** — 4.2: DDS portraits compile on Fox targets, passed through byte for byte
  (`player_` prefix on PES 18); non-DDS and two-source slots stay refused until 4.6/4.7. Fable
  landed it first time under the new brief shape (Devin's 3.3 model, contradictions invited).
- **2026-10-03** — 4.3: per-team ID blocks and each player's own boots/gloves compile on Fox;
  the tracer now matches all 17 of Red's files. The textures-last shape the lead briefed was
  wrong (a failed texture left dangling packages); the sidekick's folder-group commit replaced it.
- **2026-10-03** — 4.4a: shared Boots/Gloves folders linked plainly compile once under the team's
  shared IDs (644 up for `/co/`), with their textures beside their own package; more than 17 is
  `boots_id_pool_exhausted`/`gloves_id_pool_exhausted`, reported by `check` too. The remote
  mutation half peaked at 7.00 GiB of its 8G cap (40 mutants).
- **2026-10-03** — 4.4b1: boots/gloves parts resolving to one name merge on Fox, and a boots or
  gloves link beside a local model of its kind combines the shared folder into the player's
  exclusive package (`link_combined`, `fmdl_merged`, `skl_merge_conflict`,
  `merge_material_conflict`). The tracer's own files supply real conflicts: its `glove_l.fmdl`
  and `boots.fmdl` disagree on bone `sk_forearm_l`, its `fcl_hair.fmdl` on material `shirt`.
- **2026-10-03** — 4.4b2, closing 4.4: a face link makes the shared face folder part of the
  player's face, face parts merge like boots parts, and a texture stem two of a player's sources
  hold is one entry when the bytes agree, else `shared_texture_conflict` (lower package dropped)
  or `merged_texture_conflict` (folder dropped). Remote mutation half peaked at 7.14 GiB of 8G.
- **2026-10-03** — 4.5a: a Fox face folder no longer needs `face_diff.bin`, `fcl_hair_sim.fclo`
  or a hair skeleton (Red's templates and PES 21's `body.skl` injected), an unsuffixed model is a
  `fcl_hair` part (`fmdl_fcl_hair_fallback`), and a skeleton beside a slotless face model is
  `skl_no_slot`; `check` reports both findings.
- **2026-10-03** — 4.5b: a player folder's `face/`, `boots/`, `gloves/` and `common/` subfolders
  compile on Fox, each forcing its category (Red's referee layout, kept as legacy support).
  TC-MOD-04 re-scoped: two `skin.dds` in one player's subfolders are `texture_stem_conflict`, so
  only a combined shared folder can reach `shared_texture_conflict`.
- **2026-10-03** — 4.5c: a player's `.common` link to an FMDL bakes the Common model into its Fox
  package as a part (its Common skeleton with it), and `Common/`'s `.dds`/`.ftex` are converted
  once per export into the team's Common output, which a Common part's paths name. Each part's
  texture paths are now rewritten before the merge. Texture `.common` links found with no step;
  filed under 4.11.
- **2026-10-03** — 4.6a: every player, shared, Common and kit texture in any accepted image
  format compiles on Fox through `dds_convert` (BC7 on PES 19-21, BC3/BC1 on PES 18, missing
  mips generated, `_nrm` stems as normal maps), with a per-run conversion cache for runs of at
  most two exports. The BC7 encoder now builds optimized in dev (56 s to 9 s for one test).
- **2026-10-03** — 4.6b: a texture that cannot convert names its finding (`texture_too_small`,
  `texture_not_pow2`, `texture_type_mismatch`, `texture_codec_unsupported`) and drops its folder,
  its kit, or only itself in `Common/` and for a portrait; a Fox `kit_mask` is reported
  (`kit_texture_not_used`); portraits come in any format, encoded to BC3 by
  `dds_convert::encode_dds`.
- **2026-10-03** — 4.6 closed: slice (c), the cancellation check between a texture task's
  textures, moved to Phase 8's cancellation, which brings the Cancel that triggers it; the
  conversions in flight are already bounded, one texture at a time per worker.
