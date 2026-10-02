# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 3 (Team compiler skeleton). Phases 1 and 2 done (Phase 2 closed 2026-09-30).
**In progress:** 3.9 (`compile` through the pipeline): 3.9a done, 3.9b next. 3.7 (`libs/pipeline`) and 3.8
(`check` end to end, slices a-d) are implemented and mutation-clean; only their cross-family
reviews are left, queued for the lead (see "Handover" below), and each is marked done once its
review's rulings land. 3.6 is done (`aesthetics_export`, 35 of 75
scenarios proven; the rest need `check`/`compile`). 2.5b (GPU BC7) is step 16.x (decision entries 2026-09-21 and 2026-09-28). Release
target (2026-09-28): 0.1.0 after Phase 8; phase order 1–6, 8, 0.1.0, 7, 9–16
(`core/development_plan.md` "Releases").
**Blocked on:** nothing yet. **Hard gate at the end of Phase 3:** step 4.0 (the maintainer's
in-game appearance-fallback test) must be done before any agent itemizes Phase 4 or writes
anything for Phase 4, 5 or 6. Everything up to and including Phase 3's close may proceed.

**Handover (2026-10-01).** Until the maintainer says otherwise, the session runs as a single
Claude agent with no sidekick and no reviewer of another model family. While that holds:

- **Since 2026-10-01 (maintainer):** the lead runs in Claude Code and delegates implementation
  to a same-family subagent (Opus, medium effort, from 3.8b on; 3.8a used Sonnet) under
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
  past a queued review, but no phase closes with one outstanding.
- **Queue:** 3.7 (b): `crates/libs/pipeline` from its first commit, against `libs/pipeline.md`
  and `core/parallelism.md` "Memory budget"; the prior 3.6 rulings are in the log, and
  `.tmp/review_brief_3_6.md` is a template for the brief. 3.8 (b): `studio_core`'s `CliError`/
  `AppPaths`/location and the `team_compiler` CLI surface, from 3.8a's commit, against
  `core/architecture.md` "Tool plugin interface", `core/distribution.md` "Data location" and
  `team_compiler/settings.md` "CLI" (one review once 3.8 is done).
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
| 3 | Team compiler skeleton | `team_compiler`, `aesthetics_export`, `pipeline` | in progress |
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

Spec: `docs/plans/core/development_plan.md` "Phase 3", `docs/plans/team_compiler/README.md`, `docs/plans/aesthetics_export/README.md`.
Itemized 2026-09-30 at Phase 2's close. Order: the tracer first (one real export end to end), then
`check` end to end over the structure pass, then `compile` through the pipeline, the shell last.

- [x] 3.1 Acceptance — done: `team_compiler/README.md` "Acceptance" (TC-SRC/STR/ROS/ID/KIT/ROOT/
  DSP/CLI/OUT/GUI) with the Phase 3 scope, including the temporary compile subset and its
  Phase-3-only `content_not_yet_compiled`; the entry gates in `aesthetics_export/object_model.md`
  "Validation semantics"; CLI exit codes 0/1/2/3 (`settings.md` "CLI"); catalog additions
  (`link_target_dropped`, wider `texture_stem_conflict` and `root_file_unexpected`, pass-through
  exclusions); usable-root definition (`pipeline.md`). Reviewer: GPT 8 rounds, all ruled, with
  sidekick loops between rounds 5-8 (rulings `.tmp/review_rulings_3_1.md`)
- [x] 3.2 Converge check script — done: `scripts/acceptance.py` + `acceptance_test.py` (15
  tests; 19/19 hand mutants caught, `.tmp/acceptance_mutants.py`), `just acceptance
  [report|strict]` as gate 5; rules in `CONTRIBUTING.md` "Testing and verification", collapse
  keeps manual proofs (`AGENTS.md`); decision entry. Real repo: 75 scenarios (2 manual), no
  problems; a planted orphan citation fails `report`, a planted valid one counts as proven. The
  scanner rides along with the next (b) review (3.6 slice b)
- [x] 3.3 Tracer bullet — done: fixture `crates/tools/team_compiler/tests/fixtures/tracer/`
  (/egg/'s Stormworks player + kit g1: old layout, Studio twin with face and kit, Red's extracted
  CPK) and `resources/bins/` (`2938b4c`); crate `team_compiler` with `src/tracer.rs`
  (`compile_tracer`, scaffolding) and `tests/parity.rs` (17-row table: 6 produced rows green, face
  FMDL decoded-equal, UniformParameter/kit config/FPKD byte-equal, FTEX by decoded DDS; 11 Phase 4
  rows). Decision entries: fixture and bases; FTEX parity by decoded content. `mutants-diff`: 43,
  34 caught, 2 unviable, 7 missed, all refusal guards of `tracer.rs` (root/folder kind guards,
  the `.fclo`/`face_diff` name guards, the `.ftex` pass-through arm, the layout-marker and `.dds`
  guards), left untested because 3.9 deletes `tracer.rs`; the pipeline's own tests replace them
- [x] 3.4 The tracer's Phase 2 frictions — done: `kit_config::KitSlot` (parse, `as_str`,
  `config_name`) replaces the tracer's `kit_name_suffix` and `texture_names`' `&str` slot; the team
  id stays `u16`; `pipeline.md` step 2 names the real texture-path rewrite, "5. Writer" the tool
  version; the parity test compares FTEX headers too (a kit written as sRGB now fails it, checked);
  `FtexInfo` keeps its raw type; the player ID's type goes to 3.5. Decision entry. `mutants-diff`:
  19, 17 caught, 1 unviable, 1 missed (the tracer's layout-marker guard, one of 3.3's seven)
- [x] 3.5 `aesthetics_export` shapes — done: `object_model.md` "Structure pass types" (listing,
  metadata, context, errors, draft, raw roster, issues and `ISSUE_CODES`, file kinds, slots with
  `PlayerSlot::player_id` → `u32`), "Core types" adjusted (infallible `validate`, plain portrait/
  collar/common collections, parsed kit icon, `FileDescriptor.source`), crate layout updated;
  `players_txt.rs`/`kit_config_toml.rs` wait for their consumers. Decision entry
- [x] 3.6 `libs/aesthetics_export`, two slices, each its own commit and review: (a) `listing`,
  `conventions/`, `parse/`; (b) `validate/` with the sanitized-scope rule and `resolve.rs`, in
  three slices: b1 the report, roster rules, player/shared folders built, export findings,
  `resolve_identity`, OS-artifact skip; b2 player/shared/Common checks (allowlist, links,
  markers, reserved subfolders, stems, `fmdl_name_invalid`, cascade, orphans) with pass-through;
  b3 kits, portraits, logo, root files (adds those `ValidatedAestheticsExport` fields). The
  allowlist and the other rules (b) needed are settled in `object_model.md` "Validation
  semantics" (decision entry 2026-10-01).
  `players_txt.rs` and `kit_config_toml.rs` wait for their consumers (Refs arranger, Phase 4 kit
  step). → verify: lib tests citing the TC-STR/ROS/KIT/ROOT/ID/DSP scenarios the structure pass
  decides; `wasm_check.py` includes the crate; `mutants-diff` per slice; reviewer (b) on the new
  `pub` surface. Done 2026-10-01 (commits `63c9599`..this one): 35 scenarios proven, every
  slice's `mutants-diff` at 0 missed; review GPT round 1 (4 of 7 accepted) and sidekick S1 (3 of
  7), rulings in the log. Files: `crates/libs/aesthetics_export/`, `vtree::fold_name`,
  `object_model.md`, `messages.md`
- [ ] 3.7 `libs/pipeline`: plan section first (lead; `core/parallelism.md` "Memory budget",
  `CpkStem`'s contract in `pipeline.md` "Writer"), then the crate: `MemoryBudget` with the
  oversized branch, thread count, `CpkStem`. The section also says how the structure pass's
  solid-7z metadata read is charged (3.1 review, round S4). → verify: the `testing.md` "Infrastructure" cases
  for these three; TC-CLI-05's stems
- [ ] 3.8 `check` end to end: `team_compiler` settings struct and defaults, the clap surface of
  `settings.md` "CLI", `reader/` (discovery, folder/.zip/.7z sources via `archives`, which must
  start reporting directory entries so an archived empty kit folder survives (3.1 review), NO_USE,
  balls, duplicate refs), the structure pass and identity through `aesthetics_export`, console
  output, exit codes. → verify: TC-SRC-*, TC-CLI-01..05 and the check-observed TC-STR/ROS/KIT/
  ROOT/ID scenarios, run through the binary. `memory_cap_percent` goes through
  `pipeline::memory_cap` (3.7); TC-STR-09 is cited only here, since its THEN names
  severities, which `messages.rs` maps (the lib test asserts dispositions; 3.6 review).
  Shapes settled 2026-10-01 (decision entry "3.8: exit codes, console lines, …"). A scenario
  whose WHEN includes compiling (TC-SRC-01/03/04/06/08/09, TC-CLI-04) is cited at 3.9, where
  its compile half lands; 3.8 tests their check half uncited. TC-CLI-06 is 3.9's (output
  preflight). Slices, each its own brief, review and commit:
  - [x] 3.8a `studio_core`/`studio` plumbing: `cli_run` → `Result<u8, CliError>`, `AppPaths`
    and `ToolContext::paths`, `settings/location.rs` (CLI resolution, never writes), the
    binary's event printer thread and exit code. → verify: `studio_core` and `studio` tests
    (printer line format per scope kind; location: portable wins, config dir, neither → no
    data dir and nothing created); stub-tool exit codes through `run_cli`
    Done: `CliError`/`AppPaths`/`paths()`, `settings/location.rs`, `studio/src/console.rs`,
    `main.rs` (`load_settings`, `run_cli_mode`). `mutants-diff`: 23, 4 missed (1 fixed: the
    config-dir test was vacuous; `run_cli_mode`, `main`, `print_line` wait for 3.8b/c's binary
    tests)
  - [x] 3.8b `team_compiler` tool skeleton: `Tool` registered in `studio`, `settings.rs`
    (Phase 3 subset), `cli.rs` (the `settings.md` "CLI" surface with the Phase 3 refusals and
    the preflight: arguments, settings, teams list), a help topic, both commands stopping
    after their preflight until 3.8c/3.9. → verify: binary tests TC-CLI-03, TC-CLI-05,
    TC-CLI-07
    Done: `settings.rs`, `cli.rs` (`RunInputs`), `lib.rs` `Tool`, `help/01_command_line.md`,
    `studio` registers it, `crates/studio/tests/team_compiler_cli.rs`. `mutants-diff`: 52, 18
    missed, all in `team_compiler` code proven only by `studio`'s binary tests, which
    cargo-mutants does not run for a `team_compiler` mutant: the scenario tests move in-process
    in 3.8c
  - [x] 3.8c `check` over folder sources: `messages.rs` (severity per code, every
    `ISSUE_CODES` code covered, issue → `Message`), the scenario tests in-process in
    `team_compiler/tests/` (3.8b's moved there; the binary keeps its own seams), `reader/` discovery (folders, NO_USE,
    balls, refs), structure pass, identity, findings as events, exit code. → verify: binary
    tests TC-SRC-05, TC-STR-09, TC-CLI-01, TC-CLI-02; check halves of TC-SRC-03/04,
    TC-ID-01/02
    Done: `messages.rs` (47 rows, checked against `messages.md` by `.tmp/catalog_compare.py`),
    `reader/{mod,source}.rs`, `check/mod.rs`, `team_compiler/tests/cli.rs` (22 in-process
    tests; the binary file keeps its own seams), `aesthetics_export::team_name` re-exported.
    `mutants-diff 008a356`: 57, 45 caught, 12 unviable, 0 missed (3.8b's survivors gone but
    the `view`/`settings_view` placeholders, untestable until 3.z)
  - [x] 3.8d archive sources — done: `archives::Archive::folders()` (directory entries apart
    from `entries()`; a root entry `./` skipped), `reader/source.rs` `OpenSource` (an archive
    opened once for its listing and metadata; a `.7z` read under a `MemoryBudget` permit for
    its entries' sum, dropped with the archive; a zip uncharged), the budget made per run in
    `check::run`, routing reading metadata only for exports headed for validation; fixtures
    `tests/fixtures/sources/` (lead). TC-SRC-02 and TC-SRC-07 cited; the check halves of
    TC-SRC-01/06/08/09 tested uncited.
- [ ] 3.9 `compile` through the pipeline: coordinator and writer on rayon over `libs/pipeline`,
  the tracer's scaffolding replaced, dispositions and `pass_through` applied, the CPK written
  atomically. → verify: TC-OUT-*, TC-DSP-*, the compile-observed TC-ROS/KIT/ID scenarios,
  TC-SRC-01/03/04/06/08/09 and TC-CLI-04/06 (their compile halves), TC-STR-01, TC-ROOT-02/05
  (compile-observed, found uncited at slicing), and 3.3's parity case still green. Shapes:
  `pipeline.md` "Run driver shapes (Phase 3)"; scope edges: `README.md` "Phase 3 scope"
  (decision entry "3.9: Phase 3 compile's edges"). Slices, each its own brief, review, commit:
  - [x] 3.9a thinnest path: `compile` runs the structure pass shared with `check`, plans face and
    kit tasks from the validated export, processes them serially, writes the CPK through the
    canonical-order writer to staging and promotes it; `tracer.rs` deleted, `parity.rs` driven
    through `compile`. Until 3.9b, `compile` has no subset gate. → verify: parity green with an
    unchanged table, TC-OUT-01, a run emitting nothing writes no CPK and leaves no staging
    folder. Done: `team_compiler` `compile.rs`, `structure.rs`, `events.rs`, `check.rs`,
    `plan/`, `processing/`, `output/`, `paths.rs`, `templates.rs`, `testing.rs`,
    `tests/common/`; `aesthetics_export` `KitFolder.path`. Interim until 3.9b: a PES 15-17
    target ends the run with exit 3 and a refs export plans no task, both silently outside the
    gate 3.9b adds (both now in it). 3.9c owes the `.7z` permit-choice test (`compile.rs` `==` survivor).
  - [x] 3.9b the subset gate (`content_not_yet_compiled`, the missing-template rule; the
    pre-Fox target and refs exports move into it from 3.9a's interim), the teams list created
    by `compile` (`settings.md`), the help topic's gate limits (CONTRIBUTING: a behavior change
    lands with its topic). Design: decision entry "3.9b: what the Phase 3 subset gate counts".
    → verify: TC-OUT-02/04/06, TC-ID-02. Done: `plan/subset.rs` (the gate and the one
    classification `processing/` now reads), `plan_run(exports, version)`, `cli.rs`
    `create_teams_list`, help; 49 of 75 proven. Every later compile scenario with players needs
    a complete Phase 3 face folder (a face model plus `face_diff.bin`; `Sandbox::copy_tracer`).
  - [ ] 3.9b2 events (`FolderStatus`, `ExportProcessed`, `Complete`) and the compile-observed
    scenarios over behavior already built (split from 3.9b at 3.9a's review, to keep each
    slice one review). → verify: TC-DSP-*, TC-ROS-01/04/05/06/09, TC-KIT-01, TC-STR-01,
    TC-ROOT-02, TC-SRC-03/04/06/08, TC-CLI-04
  - [ ] 3.9c the rayon pool and admission (`pipeline.md` "Admission"), archive content read
    once per export with the `.7z` charge shared by its tasks. → verify: same CPK bytes under
    another completion order, a `.7z` over the cap compiles, TC-SRC-09, TC-ID-01, TC-ROOT-05
  - [ ] 3.9d output safety: the writability preflight, `cpk_write_failed` and
    `output_commit_failed` with nothing partial left. → verify: TC-CLI-06, TC-OUT-03, TC-OUT-05
  - [ ] 3.9e the placeholder kit (the checkerboard DDS is a lead-made resource) and
    `kit_placeholder`, with its help topic line. → verify: TC-SRC-01, every 3.9 ID
    proven in `acceptance.py report`
- [ ] 3.z Shell slice, last code step of the phase (`core/development_plan.md` "Phase 3", last bullet; decided
  2026-09-15): minimal `studio_core` shell (window, sidebar, selected tool's `view()`), `studio`
  binary registering `team_compiler`, Team compiler `view/` with settings, run button and a plain
  `PipelineEvent` log. → verify: manual, recorded in the converge step: the 3.3 fixture compiled
  from the GUI with its events visible, on Windows; Linux when a machine is available
  (TC-GUI-01/02)
- [ ] 3.w `.cargo/mutants.toml` audit, before 3.y's whole-crate runs (they measure only what it
  leaves in; maintainer, 2026-10-01: most entries came from Phase 2 sidekick work and none was
  re-verified). The lead, not a sidekick, takes every `exclude_re`/`exclude_globs` entry:
  (1) what it matches today (`cargo mutants --list` with and without it): a line-numbered
  pattern (`check\.rs:210:39`, `hand_split\.rs:210:25`, the `retarget\.rs` ones) may have
  drifted onto another expression or onto nothing; (2) the comment's equivalence argument
  re-checked against the current code; (3) each matched mutant run with the entry removed,
  confirming it survives. An entry that matches nothing, or whose mutant a test can kill, is
  removed (a killable one becomes a test in the rework brief); a broad pattern is narrowed to
  what it argues for. → verify: every surviving entry has its argument and a run showing its
  mutants survive, listed in the converge record
- [ ] 3.y Converge (`AGENTS.md` "Closing a phase"): the lead's audit against the Phase 3 plan
  sections and every TC ID (3.2's strict mode), design-health pass, `just mutants` per new
  crate, then the reviewer loop; gaps become steps above this one
- [ ] 3.x Rewrite the Phase 3 plan sections in the present tense, collapse this list

### Phase 4 — Processing logic

Steps are itemized when Phase 3 closes, and only after 4.0 is done; one more is fixed already
(the GPU BC7 step moved to Phase 16, decision entry 2026-09-28):

- [ ] 4.0a PES12 re-study, right after Phase 3 closes and before Phase 4 is itemized
  (maintainer, 2026-10-01). A quick study of supporting PES12 (the tools in
  `c:/Data/4cc/Tools_4cc/pes12tools` and `c:/Data/4cc/Tools_4cc/PES-Tools`) sits in the git
  stash named `pes12` (`docs/plans/pes12.md` with glossary, plans-index and worklog lines);
  read it with `git show 'stash@{0}^3:docs/plans/pes12.md'` (the file was untracked), never
  `git stash pop`/`apply` over uncommitted work. Re-check it against the code as Phase 3 left
  it and the two tool folders, then report to the maintainer where the codebase already
  diverges from what PES12 support would need and which Phase 4+ choices would close or widen
  that gap. → done when: the maintainer has the report and has decided what, if anything, enters
  the plans
- [ ] 4.0 **GATE, maintainer only: in-game appearance-fallback test.** No agent itemizes Phase 4,
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
  log and in a decision entry, and the plan changes it implies are written

- [ ] 4.y `dds_convert` cache retention bound (found at 2.20d converge; spec `libs/dds_convert.md`
  "In-memory conversion cache", "Retention is separately bounded and budgeted"): the
  `Converter` holds every distinct conversion until `clear`; the pipeline's memory budget
  charges `retained_bytes` and evicts under pressure, and repeated edits do not keep every
  superseded conversion → verify: a test compiling the same export with one texture edited N
  times retains one conversion of it, and a budget smaller than the cache evicts rather than
  blocking a task

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

- open — u16 face indices cap a reassembled split mesh (found at 2.20f review): `fmdl::Mesh`
  and the IR (`ir.md` "IR struct") store faces as `[u16; 3]`, so `fmdl::ops::split::decode`
  refuses (loud `VertexMismatch`) an add-on file whose components together reference more
  than 65536 vertices, and vertex-limit splitting of an IR mesh can only move loose vertices.
  The 2.20h conversion census found 114 real files over it. Since 2.20h such a group stays
  split rather than erroring (`conversion.md` split rules), so nothing refuses them now. The
  face type is still Phase 7's decision (glTF brings u32 indices).
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
  `PlayerEntry.id` leaves it stale. Nothing changes an id today. How the writer sets it is
  decided with Phase 4, since step 4.0 tests exactly that field (-1 = fall back to
  `PlayerAppearance.bin`).
- **Texport write is unverified in-game** (2.17h): `Texport::new` synthesizes 18-21 files from
  measured templates and `to_bytes` rewrites read files; both round-trip byte-identical, but no
  generated file has been imported by the game yet (`verification.md` "Texport write": manual,
  per version; a `new` file and an edited round-tripped one). PES 15/16/20 texports have no
  fixture at all (offsets/key index are the reference's; PES 20's size is derived).

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
