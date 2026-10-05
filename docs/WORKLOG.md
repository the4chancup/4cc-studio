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
cross-family review (a) is queued). Next: 4.33 (`name.y` in PES 21 units), 4.34 (the
`Full`/`Midcup` tag), then 4.21 (bins from the installed CPKs; 4.14 waits on 4.31's
pre-Fox export); 4.30,
4.5 to 4.8, 4.9a and 4.10 to 4.13 are done (4.6c moved to Phase 8's cancellation), 4.9b
(collars) waits on the maintainer. 2.5b (GPU BC7) is step 16.x (decision entries
2026-09-21 and 2026-09-28). Release target (2026-09-28): 0.1.0 after Phase 8; phase order 1–6,
8, 0.1.0, 7, 9–16 (`core/development_plan.md` "Releases"); first-class target the Fox version
the cup moves to around April 2027 ("Target versions").
**Blocked on:** nothing.

**Handover (2026-10-01).** Until the maintainer says otherwise, the session runs as a single
Claude agent with no sidekick and no reviewer of another model family. While that holds:

- **Since 2026-10-01 (maintainer):** the lead runs in Claude Code and delegates implementation
  to a same-family subagent (Opus from 3.8b, Fable from 3.9d on; 3.8a used Sonnet) under
  the lead/sidekick rules, to keep the lead's context small. That subagent is the sidekick, not
  a reviewer: the queue below still holds every cross-family checkpoint. From 2026-10-03, after
  step 4.6: the lead runs on Fable 5.1 and the sidekick on Opus 5.5 (the two have separate
  usage pools); which model suits which role is under evaluation, so the lead keeps a per-slice
  tally of the sidekick's deliveries in its project memory.
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
  4.3-4.4 (b): `team_compiler` from `a949664` to `ed5e6a3` plus `aesthetics_export`'s
  `SharedModelFolder.path` (two crates, new `pub(crate)` shapes: `ModelFolder`, `TaskGroup`,
  `TextureHome`, `TaskFailure`), against `pipeline.md` steps 3, 6 and 7, `player_folders.md` "ID
  allocation", "Shared models" to "Merge constraint", TC-MOD-01..09, TC-PLN-01/02, and the
  decision entries of 2026-10-03 from "a player folder's tasks commit as one group" on.
  4.5c-4.6b (b): `team_compiler` from `624825f` to the 4.6b commit plus `aesthetics_export`'s
  `common_link_name` and `dds_convert`'s `encode_dds` (three crates, new `pub` items), against
  `player_folders.md` "Common model links and model merging", "Portraits", `model_format.md`
  "Link files", `pipeline.md` steps 5-6 and its "Common textures are one task" bullet,
  `messages.md` "Textures", `libs/dds_convert.md` "`dds_convert` API", TC-MOD-10/11,
  TC-TEX-01/02/04/06, TC-PRT-01, TC-KIT-10, and the decision entries of 2026-10-03 from "Fox's
  Common textures are one task" on.
  4.7 (a) and (b), the step now done: TC-CHK-06 and TC-CHK-07
  (`team_compiler/README.md` "Deep checks"); `aesthetics_export`'s `ContentFinding` and
  `ValidationReport::with_content_findings`, `team_compiler/src/deep/`, `validation.rs` and
  `reader/` (`ContentSource`, `route`), `fmdl`'s and `pes_model`'s `check::CODES`,
  `dds_convert::probe`, from `f937e3d` to the 4.7f commit (`perf(team_compiler)`, 2026-10-04),
  against `libs/pipeline.md` "What a solid `.7z` is charged", `pipeline.md` "2. Per-export serial steps" (deep format pass),
  `object_model.md` "Validation semantics" (content findings), `messages.md` "Model checks"
  and "Textures", and the decision entries "the deep pass: the compiler checks, the export
  crate drops" and "texture checks: from the header, in the deep pass".
  4.8 (b): `aesthetics_export`'s `colors_txt` (`read_colors_txt`, `ColorsTxt`,
  `RefusedColorLine`, `ColorLineRefusal`; new `pub` items) and `validate_with`'s two
  filtered fields, `team_compiler`'s `bins/`, `deep/documents.rs` (`colors_findings`),
  `validation::team_colors`, the kit task's colors and the writer's two bins, from
  `7d3d94a` to the 4.8c commit (`feat(team_compiler)`, 2026-10-04), against `pipeline.md`
  "4. Per-export non-model steps" (Bins accumulation) and "Resolved decisions" (Kit colors
  fallback), `player_folders.md` "Root files" (Colors), `libs/color_tools.md` "Dominant
  kit-color extraction", `resources/bins/README.md`, TC-KIT-11..14, TC-ROOT-10,
  TC-BIN-01..03, and the decision entries "kit colors merge into a team's UniColor record
  by kit number" and "kit colors derive from the effective main texture; the missing
  pair's bytes".
  4.9a (b): `fpc` (`kit_values()`), `kit_config` (`apply_fpc`, `matches_fpc`: changed
  `pub` signatures) and `team_compiler` (`plan::EffectiveTeamKitFpc`, the kit task's
  reconciliation, `deep/documents.rs` `kit_config_findings`), the 4.9a commit
  (`feat(team_compiler)`, 2026-10-04), against `fpc_toggle.md` "Team kit-FPC status and kit
  configs", `libs/fpc.md`, `pipeline.md` "4. Per-export non-model steps" (Kits),
  `messages.md` (`kit_config_version_clamped`, `kit_config_fpc_adjusted`), TC-KIT-15..17
  and the decision entry "4.9a: the FPC kit values take no version; the template carries
  them".
  4.10 (b): `dds_convert` (`resize`, a new `pub`) and `team_compiler`
  (`processing/kit_layout.rs`, the kit task's re-layout), the two 4.10 commits
  (`docs(team_compiler)` and `feat(team_compiler)`, 2026-10-04), against `pipeline.md` "4.
  Per-export non-model steps" (Kits, "Layout conversion"), `player_folders.md` "Kit layout
  marker", `libs/dds_convert.md` "`dds_convert` API", `messages.md`
  (`kit_layout_converted`), TC-KIT-18, TC-KIT-20, `tests/fixtures/kit_layout/README.md`,
  and the decision entries "the kit layout table covers the socks alone, in two bands" and
  "the suite's one resampler is `dds_convert::resize`". The measurement scripts
  (`scripts/provenance/kit_uv/`) are part of the surface: the table is only as good as they
  are. 4.11 (b), one review over the step's slices as they land: 4.11a, `dds_convert`
  (`encode_png`, a new `pub`) and `team_compiler` (`processing/team_assets.rs`,
  `TaskKind::Logo`, `paths::logo`), the `feat(team_compiler)` commit of 2026-10-04 for
  4.11a, against `pipeline.md` "4. Per-export non-model steps" (Logo), `player_folders.md`
  "Logo", `messages.md` (`logo_fit_applied`, `logo_upscaled`), TC-ROOT-06 to TC-ROOT-08 and
  the decision entry "the logo's geometry, its findings and its PNG encoder"; 4.11b,
  `team_compiler` (`output/teamnotes.rs`, the note's path from `validation.rs` through
  `plan_run` to `compile::run`), its `feat(team_compiler)` commit, against `pipeline.md`
  "2. Per-export serial steps" (5, Notes collection), `messages.md`
  (`teamnotes_write_failed`), TC-ROOT-09 and the decision entry "`teamnotes.txt`: its
  layout, when it is written, and its failure"; 4.11c, `team_compiler` (`kit_variants.rs`,
  `complete_kit_variants`, `point_texture`, `PlayerFile::LeftOutKitVariant`), its
  `feat(team_compiler)` commit, against `model_format.md` "Kit-dependent assets (`kitN`)",
  `pipeline.md` "4. Per-export non-model steps" (Kit-dependent assets), `messages.md`
  (`kit_variant_missing`, `kit_variant_model_fox`), TC-CMN-04, TC-CMN-05 and the decision
  entry "kit variant sets are found from the files, and completed by the textures task".
  4.11d, `team_compiler` (`PlayerFile::CommonTexture`, `point_texture`'s places), its
  `feat(team_compiler)` commit, against `model_format.md` "Link files (`.common`)",
  `pipeline.md` "3. Per-model-folder parallel steps" step 6, TC-TEX-09 and the two "Issues"
  entries the slice opened.
  The acceptance section changed with these slices (TC-TEX-09 and TC-CMN-07 added,
  TC-CMN-05 narrowed to its Fox half): they join the queued (a) review of the section.
  4.12 (b): `team_compiler` (`plan/subset.rs` `model_role` under the marker,
  `PlayerFile::UnusedFaceFile`, `FolderModels::of_player_files`; `plan/mod.rs`
  `ModelFolder::ingame_face`, `folder_tasks`' blank face; `validation.rs`
  `face_file_not_used`), its `feat(team_compiler)` commit of 2026-10-04, against
  `player_folders.md` "`ingame_face` marker" and "`ingame_face` with shared links",
  `pipeline.md` "2. Per-export serial steps" step 4, `messages.md` (`face_file_not_used`),
  TC-MOD-16 to TC-MOD-19 and TC-MOD-32, and the two decision entries of 2026-10-04 on the
  face file with no face model and on TC-MOD-09 (reworded; TC-MOD-32 is new: both join the
  queued (a) review of the section).
  Coverage tag and `name.y` (a): the scenarios written on 2026-10-04 (TC-ID-05 to 07,
  TC-KIT-24, TC-BIN-14 to 17 new; TC-ID-01, TC-KIT-17, TC-BIN-05, 06 and 15, TC-PLN-03
  reworded) with `aesthetics_export/object_model.md` "Coverage tag", `pipeline.md` "Bins
  accumulation" and the two decision entries of that date join the queued (a) review.
  4.13 (b), one review over the step's slices as they land: 4.13a, `team_compiler`
  (`validation.rs` `refuse_duplicate_teams`), its `feat(team_compiler)` commit of
  2026-10-04, against `pipeline.md` "3. Per-model-folder parallel steps" (the run-level
  paragraph), `messages.md` (`duplicate_aesthetics_export`), TC-PLN-03, 05 and 07 and the
  decision entry "`duplicate_aesthetics_export` is the validation pass's"; 4.13b,
  `team_compiler` (`plan/overrides.rs`, `output/writer.rs`, `compile.rs`), its
  `feat(team_compiler)` commit of 2026-10-04, against `pipeline.md` "5. Writer" step 1,
  `settings.md` "Path resolution" (`overrides/`), `messages.md` (`duplicate_path`,
  `overrides_active`), TC-PLN-04 and the decision entry "the `overrides/` tree is the
  writer's; no path preflight over the tasks"; 4.13c, `team_compiler` (`reader/source.rs`
  `SourceRevision`, `compile.rs` `plan`, `build`, `coordinate`), its `feat(team_compiler)`
  commit of 2026-10-04, against `pipeline.md` "Resolved decisions" (Source snapshot),
  `messages.md` (`source_changed_during_run`), TC-PLN-06 and the decision entry "a
  source's revision is the listing's, checked after each task's read".
- For the lead, on return: the review process on trial (3.1) opens with a full sidekick review
  loop, then runs GPT's loop with a full sidekick loop after each GPT round, calling GPT again
  only once that sidekick loop has ended and GPT's own loop has not; not yet in `AGENTS.md`
  (3.6: GPT 4 of 7 accepted, then sidekick S1 3 of 7, so both loops ended after one round
  each). Prior rulings: `.tmp/review_rulings_3_6.md`. From 2026-10-04 the Devin lead runs the
  whole queue below on the code as it now is (a diff a later step superseded is not reviewed
  on its own), in queue order; its progress is `.tmp/review_queue.md`.
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
in git history up to commit `2f158a9`. CI proof: green run on `5e6ffc6`, deliberately red run on
`eccf0f3` (both `gates` jobs failed at `just gates`, `deps-check` unaffected), reverted in
`2f158a9`.

### Phase 2 — Library crates

Done 2026-09-30 (spec now describes what exists: `docs/plans/core/development_plan.md`
"Phase 2", `libs/README.md`, `model_conversion/README.md`, `pes_savefile/README.md`). Step
detail (2.1-2.21: each crate's build, review rounds A-C, the 2.20a-k converge with its
mutation runs, censuses and reviewer rulings) in git history up to commit `7de61f7`. Carried
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
  203); `mutants-diff a949664`: 26, 21 caught, 5 unviable, 0 missed. Lead fix: the archive
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
  81 of 203); `mutants-diff cd2e979`: 80 caught, 15 unviable, 1 timeout (an infinite loop), 1
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
  TC-PLN-02 cited. Gates green (84 of 203); `mutants-diff ccd9173`: 40, 34 caught, 6 unviable,
  0 missed. (b) split into b1 (boots/gloves) and b2 (faces, texture byte conflicts). (b1) done
  2026-10-03 (Fable, first time; contradictions accepted: a stem two *combined* folders hold
  collides too, refused for any two of the player's sources; a plainly linked shared folder's
  parts merge as well. Lead fix: stems compared case-folded): `ModelFolder.combined`/`sources()`,
  `subset::link_combines` shared by gate, IDs and planning, `processing::TaskFailure` carrying
  `merge_material_conflict`/`skl_merge_conflict`, `FolderModels.boots_stems`. TC-MOD-07/09 cited.
  Gates green (86 of 203); `mutants-diff b744ff3`: 43, 36 caught, 7 unviable, 0 missed. Not
  verified: the game loading a merged FMDL (`Model::to_file` re-lays it out). (b2) done
  2026-10-03 (Fable, first time, no contradiction; decision entry "texture conflicts by source
  package"): face links combine, face parts merge (`fcl_hair_sim.skl` by the boots rule),
  `ModelFolder::roles()` with the player's own `face_diff.bin`/`.fclo` winning,
  `processing/texture.rs` resolving a stem per source package (`shared_texture_conflict` drops the
  lower package via `TaskBatch.skipped`, `merged_texture_conflict` fails the textures task).
  TC-MOD-08 cited; TC-MOD-04's halves tested uncited (its GIVEN needs 4.5's subfolders). Gates
  green (87 of 203); `mutants-diff 6bd79c9`: 34, 28 caught, 6 unviable, 0 missed. Texture-name
  case questions in "Issues"

- [x] 4.5 **Face assembly (Fox)**: unsuffixed models routed to the `fcl_hair` merge (`torso.fmdl` +
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
  `skl_no_slot`; (b) reserved subfolders; (c) `.common` model links; (d) `face_diff.xml`, decoded with
  the `base64` crate (decision entry 2026-10-03). (a) done
  2026-10-03 (Fable, first time; lead first: `resources/templates/` from Red with a provenance
  README): `templates::{FACE_DIFF, FCL_HAIR_SIM_FCLO, BODY_SKELETON}` injected per Face package,
  unsuffixed models are `fcl_hair` parts, `PlayerFile::SlotlessSkeleton`,
  `structure::model_name_messages` (findings on the folder holding the file, a shared face's
  included); the gate's `missing` checks gone. TC-MOD-12/13 cited. Gates green (89 of 203);
  `mutants-diff 5eeb04c`: 38, 33 caught, 5 unviable, 0 missed. Found: the tracer's
  `fcl_hair.skl` is PES 21's `body.skl`, so two older processing tests' "custom" skeleton equals
  the template (fix in 4.5b's brief). (b) done 2026-10-03 (Fable, first time; no contradiction
  of substance): `subset::position`/`model_role` force a reserved subfolder's category, skeletons
  pair by path stem (same directory), the two weak skeleton tests now use PES 19's `body.skl`.
  TC-MOD-14 cited, TC-MOD-04 re-scoped and cited (decision entry). Gates green (91 of 203);
  `mutants-diff 428ef6c`: 40, 34 caught, 6 unviable, 0 missed. For converge: the four reserved
  names are matched in both `aesthetics_export` (`validate::folders::position`, `pub(crate)`) and
  `team_compiler` (`subset::position`); one exported matcher would remove the copy. (c) done
  2026-10-03 (Fable; one contradiction accepted: with paths rewritten per part, TC-MOD-09's two
  `shirt` materials differing only by source directory became one, so its fixture now differs
  by shader): `PlayerFile::CommonModel`, `plan::CommonModel` resolved at planning, the Common
  `.skl` travelling with a slotted link, texture paths rewritten per part before the merge, the
  Common textures task (`TaskKind::CommonTextures`, decision entry); lead first: `aesthetics_export`
  exports `common_link_name`. TC-MOD-10/11 cited. Gates green (93 of 203); `mutants-diff
  624825f`: 86, 72 caught, 13 unviable, 1 missed (`model.rs` `CommonModel` package guard), given a
  test in one rework round (red shown). For converge: `subset::is_direct_common_file` copies
  validation's private check. (d) done 2026-10-03 (Opus 5.5, first time; two contradictions
  accepted: the catalog holds no text templates yet, and `base64` is taken without its default
  SIMD feature; lead first: the plan's `face_diff.xml` paragraph, the fixtures under
  `tests/fixtures/face_diff/` from two real bins): `processing/face_diff.rs` (`check`,
  `from_xml`), `PlayerFile::FaceDiffXml` routed wherever `face_diff.bin` is, the xml standing
  for the bin across sources and both kept within one, `face_diff_invalid` and
  `xml_dif_conflict` as failures of the Face task (the folder's boots, gloves and textures
  still commit), a supplied `face_diff.bin` checked too. A face diff passes when it is at
  least as long as its header gives: Red's exact-length check would refuse 326 of the
  machine's 2,695 loose bins (decision entry). TC-MOD-15 cited. Gates green (100 of 203);
  `mutants-diff a75e3a9`: 57, 51 caught, 6 unviable, 0 missed. For 4.7: both findings are
  reported only by `compile` until the deep pass runs the same check at `check`

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
  cited. Gates green (95 of 203); `mutants-diff bd3bb97`: 23, 15 caught, 8 unviable, 0 missed.
  For converge: `Converter::convert` returns `Arc<[u8]>` even under `Bypass` and `Entry` owns a
  `Vec`, so each texture is copied once (`Use`) or twice (`Bypass`); an `.ftex` source is now
  rebuilt rather than passed through. (b) done 2026-10-03 (Fable; contradictions accepted: a
  failed portrait task reports `DropFile`; the size checks read the written FTEX's header, so
  no second decode; `kit_texture_not_used` is reported before the gate, noisy until 4.20; lead
  first: the catalog's drop scopes and signature rule, decision entry, and the fixtures):
  `dds_convert::encode_dds` (sharing `convert`'s steps), `texture::TextureError`, the four
  findings, Common's `DropFile`, portraits in any format. TC-TEX-04/06, TC-PRT-01, TC-KIT-10
  cited. Gates green (99 of 203); `mutants-diff 0d26507`: 75, 68 caught, 7 unviable, 1 missed
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

- [x] 4.7 **Deep validation pass**, in slices (a) to (f), done 2026-10-04. The shape (decision entry "the deep pass:
  the compiler checks, the export crate drops"): tc reads the files of the sanitized export
  and runs the checks (`deep.rs`, new); `ae` takes what they find
  (`ValidationReport::with_content_findings`) and derives the sanitized export again, so the
  drops, the link cascade and `pass_through` are the structure pass's own; the pass runs
  before identity and planning, in `check` and in `compile`, on every source kind, archives
  included. A format finding maps by severity (`messages.md` "What makes a finding an Error":
  Error drops the folder, Warning and Info keep it); the lead's `fmdl::check` census over the
  VGL26 corpus (`C:\Data\4cc\Lab\Gud`) found one Error class, the far vertex, which stands.
  Plan: `pipeline.md` "2. Per-export serial steps" step 2 (deep format pass);
  `object_model.md` "Validation semantics" (content findings); `messages.md`
  (`vertex_too_far_from_origin`). IDs: TC-CHK-01..05, TC-PRT-02, and from 4.6 TC-TEX-03 and
  TC-PRT-03 (texture findings reported by `check`; TC-TEX-05 is 4.29's). Crates: ae
  (`validate/`), tc (`deep.rs`, `structure.rs`, `check.rs`, `messages.rs`, `processing/`),
  fmdl, pes_model, dds_convert (the code lists and the header probe)
  → verify: `check` on a `/co/` export whose `boots.fmdl` holds a vertex 6000 units out prints
  `Error vertex_too_far_from_origin [DropFolder]` naming the file and exits 1; `compile` with
  `pass_through` on still leaves the folder out of the CPK
  - (a) the thin path end to end: `ContentFinding` and `with_content_findings` in `ae`;
    `deep.rs` reading every `.fmdl` of the player folders, the shared folders and `Common/`
    and reporting `vertex_too_far_from_origin` (`DropFolder`, not pass-through-eligible) for
    `fmdl_vertex_far_from_origin`; TC-CHK-01..02 (lead first: the far-vertex model and the
    solid `.7z` holding it, `tests/fixtures/deep/`). Done 2026-10-03 (Opus 5.5, first
    time): `ContentFinding`, `ValidationReport::with_content_findings` (`validate`'s one
    body, the findings standing before the cascade; kits, portraits, collars, the logo and
    a player's `settings.toml`/portrait now read the dropped scopes too); tc `deep.rs`,
    `structure.rs` renamed `validation.rs` (`validation_pass`), the help topic. Gates green
    (102 of 203); `mutants-diff f937e3d`: 29, 16 caught, 12 unviable, 1 missed
    (`is_fox_model -> true`: a file that does not parse has no finding yet, so reading
    every file as a model changes nothing; (b)'s `model_broken` makes it observable and
    its brief asks for the test). Not covered, for (c)/(d) if they need it: a drop naming
    `Kits/all`, a kit's `config.toml` or `colors.txt`, or the root `colors.txt`
  - (b) every format finding by its severity. Done 2026-10-03 (Opus 5.5, one rework
    round): the deep pass reads every `.fmdl`, `.model` and `.mtl` and reports each check
    code of `fmdl` and `pes_model` once per file with its summed count (`deep.rs`); an Error
    drops the folder and is pass-through-eligible, the far vertex and a file that does not
    parse (`model_broken`, `mtl_broken`) are not; `CODES` in both crates' `check.rs` is the
    one place a severity is written, and the catalog test holds every format code's row to
    it; a rule about a mesh, material or bone itself now counts 1, not 0. TC-CHK-06..07
    (new). The tracer's models carry `fmdl_weights_not_normalized` (Info), so most CLI
    tests gained those lines; placeholder models in tests are now a real clean one. Gates
    green (104 of 205); `mutants-diff eb5fb67`: 96, 83 caught, 13 unviable, 0 missed. `check_bundle`
    (`model_material_undefined`) is the pre-Fox face steps'
  - (c1) textures of model folders, `Common/` and kits from their headers
    (`dds_convert::probe`, no decode): `texture_type_mismatch`, `texture_too_small`,
    `texture_not_pow2`, `kit_texture_too_big`; processing's own checks on those sources
    retired, since the deep pass has run them and `pass_through` must be able to keep the
    file (`texture_codec_unsupported` stays conversion's). Plan: `messages.md` "Textures",
    `libs/dds_convert.md` "`dds_convert` API", decision entry "texture checks: from the
    header, in the deep pass". TC-TEX-03, TC-CHK-04. Done 2026-10-03 (Opus 5.5, one
    tests-only rework round): `dds_convert::probe` (`Probe`; a raster source's level count
    from `mips::chain_len`, the one place the chain's length is written); in `deep.rs`,
    `Checked::Texture(SourceFormat, SizeRule)`, `texture_finding` (the first rule that
    fires: mismatch, too small, then the kit's main texture or the Fox mip rule) and the
    signature table, moved from processing; a kit's findings on the kit's folder, an
    `all/` texture read once and named by its export path on each inheriting kit;
    `texture::convert` checks nothing itself any more (a portrait's checks stay in
    processing until (c2)); an unreadable texture is `source_read_failed` like a model.
    Under `pass_through` a 3x3 texture converts and is packed. Gates green (106 of 206);
    `mutants-diff e6e9f21`: 123, 73 caught, 1 timeout (an infinite loop), 42 unviable, 7
    missed, all the size rules asked about square textures only; after the rework's table
    test: 123, 80 caught, 1 timeout, 42 unviable, 0 missed. `deep.rs` is 1,258 lines with its tests: (c2) splits it into `deep/`
  - (c2) portraits from their headers (the file dropped; a power-of-two side always),
    `portrait_conflict` (both sources, different bytes: the export dropped; equal bytes:
    one portrait, and planning's refusal of a slot with both goes), the logo's decode
    (`logo_file_invalid`), processing's portrait checks retired. TC-PRT-02..03, TC-CHK-05
    (its "no logo is emitted" half is trivially true until 4.11; the test asserts the
    CPK's exact entries, so it keeps proving it afterwards). Done 2026-10-03 (Opus 5.5,
    first time): `deep.rs` split into `deep/mod.rs` (the walk), `deep/model.rs` and
    `deep/texture.rs`, a pure move; a portrait checked on its own `File` scope
    (`SizeRule::Portrait`), a player's after its folder's files, the `Portraits/` files
    after `Common/`; `portrait_conflict` (`Export`, `DropExport`) compares the bytes of a
    slot's two files whatever their own findings (each is read a second time for it);
    planning keeps the folder's file for a player id with both, and the gate's refusal is
    gone; the logo's files decoded last (`Checked::Logo`); `portrait` in processing still
    decodes every source, so a broken one fails its task, and checks nothing else. Only
    team rosters pair a folder's portrait with a `Portraits/` file. Not tested in the
    deep pass: a folder two slots map, paired with each slot's file (planning's side is).
    Gates green (109 of 206); `mutants-diff 8eb7931`: 65, 52 caught, 13 unviable, 0 missed
  - (d) `face_diff_invalid` and `xml_dif_conflict` reported by the deep pass, where they drop
    the folder (`player_folders.md` "`face_diff.xml`"); `kit_config_invalid`,
    `settings_toml_invalid` (parse only, through `pes_savefile`'s `PlayerSettings` TOML
    reader; the other `settings_*` and `fpc_strip_conflict` codes describe resolved values
    and are Phase 5's). TC-MOD-15, TC-CHK-03. Done 2026-10-03 (Opus 5.5, first time; three
    lead fixes): `deep/documents.rs` (the face diff of each player folder and shared face
    folder, over the files planning's `player_file` gives a face diff's role; a kit's
    `config.toml` before its textures; a player's `settings.toml` after its portrait, on
    its own `File` scope); the portrait checks moved to `deep/portrait.rs`, a pure move;
    `face_diff.rs` now at the crate root; `pes_savefile` is a dependency of tc. Processing
    lost its conflict check and its `face_diff.bin` check, and a `face_diff.xml` that
    fails to decode there is an ordinary `folder_pack_failed` (unreachable from the CLI,
    so untested); planning's `roles` no longer keeps both forms of one source (lead fix,
    with the duplicated test helper and a doc comment). A face diff finding now names its
    file below the folder (`face_diff.xml`, not the export path). Not covered: a folder
    whose only face models are `.model` files gives its face diff no role, so it is
    unchecked until the pre-Fox face steps. Gates green (110 of 209);
    `mutants-diff 90e5bfb`: 38, 25 caught, 13 unviable, 0 missed
  - The timing, done 2026-10-03 (lead; release build of `78fc7c2`, 16 logical CPUs, warm
    file cache, median of 3; `.tmp/timing_build.py` lays an old-layout VGL26 export out as
    `Players/NN - Name/`, `.tmp/timing_run.py` times it). FNG, the corpus's largest: 991 MB,
    245 files, 880 MB of DDS (nine 8192x8192 textures of 67 to 90 MB), 111 MB of models;
    its solid `.7z` is 78 MiB. DBG: 604 MB, 214 files, 355 MB of models, 249 MB of DDS; its
    `.7z` is 111 MiB.

    | seconds | `check` folder | `check` `.7z` | `compile` folder | `compile` `.7z` |
    |---|---|---|---|---|
    | FNG | 0.55 | 2.59 | 3.08 | 7.19 |
    | DBG | 0.82 | 2.76 | 1.74 | 5.78 |

    `check` on a folder is nearly all deep pass, so the serial pass is 18% (FNG) to 47%
    (DBG) of a folder compile. Reading every file whole takes 0.41 to 0.50 s on FNG and
    0.19 s on DBG (`.tmp/timing_read.py`): FNG's pass is whole-file reads of textures for
    their headers, DBG's is model parsing. Each `.7z` decompression costs about 2 s, and a
    compile of these exports does two, about 4 s of its 5.8 to 7.2 s (corrected at (f): first
    written as three of 1.0 to 1.4 s, the count assumed; the timing exports hold no metadata
    file, so routing never decompressed them). Not measured: a cold cache, a run of
    many exports. Found: every face folder of both exports holds only a `face_diff.bin` and
    a portrait (an in-game face with a diff), which `compile` refuses today as
    `content_not_yet_compiled` (4.12's); the timing exports leave those files out
  - (e) the pass on the worker pool (decision entry "the deep pass runs on the worker pool,
    within one export"; `pipeline.md` "Deep format pass"): an export's model folders and
    `Common/` files checked in parallel, a folder's files too, findings in file order;
    `ContentSource::read` on `&self`, an archive behind a lock; the sources routed in
    parallel; folder and `.zip` exports checked in parallel with each other, `.7z` ones in
    turn (decision entry "folder and zip exports are checked in parallel");
    `check` and `compile` share one pool. Done 2026-10-03 (Opus 5.5, first time; resumed
    once after an IDE restart; one lead fix, a test comment): `validation::run_pool`,
    built at the start of both commands; `ContentSource::read(&self)`, the archive and its
    permit in one `Mutex` held for one read; `par_iter` with an indexed `collect` over
    the player folders, the shared folders, a folder's files and `Common/`'s files, the
    portraits, kits and logo in order; `check_sources` (folders and zips in parallel,
    then the `.7z` ones, merged by discovery index); `reader::route` in parallel. No
    existing test changed. The brief's premise that one worker thread would expose a
    lock held across parallel work was wrong (the sidekick showed it does not hang): the
    guard is the read's shape, owned bytes with the lock released. Gates green (110 of
    209); `mutants-diff 828242e`: 21, 12 caught, 9 unviable, 0 missed. Measured again
    (same setup, median of 3, before → after):

    | seconds | `check` folder | `check` `.7z` | `compile` folder | `compile` `.7z` |
    |---|---|---|---|---|
    | FNG | 0.55 → 0.32 | 2.59 → 2.45 | 3.08 → 2.87 | 7.19 → 7.01 |
    | DBG | 0.82 → 0.34 | 2.76 → 2.35 | 1.74 → 1.30 | 5.78 → 5.32 |
    | both in one run | 1.31 → 0.55 | 5.05 → 4.72 | 4.69 → 3.95 | 12.86 → 12.03 |

    What is left of a folder compile is planning, the coordinator's reads, the tasks and
    the CPK's writing (FNG: about 2.5 s of 2.87); of a `.7z` compile, mostly its
    decompressions (two for these exports, three for one holding a metadata file: (f))
    → verify: the timing table above measured again, every existing test unchanged
  - (f) a `.7z` decompressed once for the structure pass and the deep pass (decision entry
    "a source's metadata is read by its check, not by routing"; `libs/pipeline.md` "What a
    solid `.7z` is charged"). Done 2026-10-04 (Opus 5.5, first time, no lead fix): routing
    only lists (`reader::source::list`; `route` takes no budget, `Route::Validate` holds the
    listing alone); `ContentSource::read_metadata`, called by `check_source` on the handle
    the deep pass then reads from (`deep::content_findings` takes the `ContentSource`),
    dropped when the deep pass ends; `OpenSource` and its archive-only metadata read are
    gone, one read path for the three source kinds. A disabled, balls or conflicting-refs
    `.7z` is never decompressed. No existing test's expectation changed; one new test (the
    metadata read and the later reads share one permit, held until the source is dropped).
    Gates green (110 of 209); `mutants-diff 3ea4018`: 11, 4 caught, 7 unviable, 0 missed.
    Measured (release build, same setup, median of 3, the 4.7e binary → this one). The
    timing exports hold no metadata file, so routing never decompressed them and they
    measure the same as before (FNG `.7z`: `check` 2.17 → 2.28, `compile` 6.36 → 6.62,
    within the runs' spread). With a root `notes.txt` added to each
    (`.tmp/timing_notes.py`):

    | seconds, `.7z` with `notes.txt` | `check` | `compile` |
    |---|---|---|
    | FNG | 4.39 → 2.36 | 8.82 → 6.79 |
    | DBG | 4.23 → 2.31 | 7.19 → 5.21 |
    | both in one run | 6.95 → 4.77 | 14.02 → 11.76 |

    A decompression costs about 2 s per export (both in one run save 2.2 s, not 4: routing
    decompressed the two in parallel). The folder runs are unchanged (FNG `check` 0.32 →
    0.29, `compile` 2.74 → 2.73). Left: the tasks' decompression, about 2 s of a `.7z`
    compile per export ("Issues")

- [x] 4.8 **Kit colors, UniColor and TeamColor (the `bins/` module)**: kit `colors.txt` grammar
  (`player_folders.md` "Root files", "Colors": one color per line in both files, the
  maintainer's ruling of 2026-10-03), `color_entry_invalid`, derivation via
  `color_tools::kit::extract_kit_colors` on the decoded main texture (`kit_colors_derived`), the
  magenta/black pair (`kit_colors_missing`), the icon marker (`icon_<N>`, step 4.30; default
  3); `UniColor.bin` built on the
  bundled base (lead first, done 2026-10-03: `resources/bins/TeamColor.bin` and
  `UniColor.bin`, Red's, with the measured record layout in the folder's README and
  `scripts/provenance/fixtures/color_bins_compare.py`) at Red's per-team offsets, an entry applied only when its
  kit's task commits (placeholder kits included: TC-SRC-01's p2 gains its entry); root
  `colors.txt` → `TeamColor.bin`, `team_colors_missing` (I); both bins written whole with
  every record's header set from its position, `bin_header_repaired` (W) when one was wrong
  (TC-BIN-13; installed bins carry headerless `TeamColor.bin` records). Plan: `pipeline.md` "4. Per-export
  non-model steps" (Bins accumulation), "Resolved decisions" (Kit colors fallback);
  `libs/color_tools.md` "Dominant kit-color extraction". IDs: TC-KIT-11..14, TC-ROOT-10,
  TC-BIN-01..03, TC-BIN-13. Crates: tc (`bins/mod.rs`, `processing/kit.rs`), resources (`resources/bins/`) →
  verify: a `/co/` export with `p1/colors.txt`, `p2/kit.dds` without colors and an empty `p3/`:
  the CPK's `UniColor.bin` carries at team 714's p1 offset the file's two colors, at p2's the
  pair `extract_kit_colors` returns for the decoded texture, at p3's the magenta/black pair; the
  tracer parity test compares `TeamColor.bin` and `UniColor.bin` byte-identical (the tracer's
  Studio fixture gains its root and `g1` `colors.txt`, the note's colors; lead first). In
  slices (decision entry "kit colors merge into a team's UniColor record by kit number"):
  - (a) the `colors.txt` reader (`aesthetics_export::colors_txt`) and `color_entry_invalid`
    from the deep pass, for the kit files and the root file, so `check` reports it. Done
    2026-10-04 (Opus 5.5, first time, no lead fix): `read_colors_txt(bytes, capacity)`
    giving the valid colors and the refused lines (`NotOneColor`, `PastCapacity`), each
    line decoded alone; `deep/documents.rs` `colors_findings`, one Warning per refused line
    on the file, kept, with `line` and `reason`, a kit's after its config and the root
    file's last; the help paragraph. The reader also settles: separators are spaces and
    commas only (a tab is refused), a leading `+` is refused, leading zeros are read.
    Open for (b): a `colors.txt` that cannot be read is reported `DropFile` but stays in
    the validated export. Gates green (110 of 209); `mutants-diff a9d9a51`: 37, 34 caught,
    3 unviable, 0 missed
  - (b) `TeamColor.bin`: `bins/` (the bin as records, every header set from its position,
    `bin_header_repaired`, the bundled base as the run's working bin); the root `colors.txt`
    read while the export's source is open and applied for each export planning keeps;
    `team_colors_missing`; the writer adds the bin to every CPK it writes; a `colors.txt`
    the deep pass drops leaves the validated export. TC-ROOT-10; the tracer parity row
    `TeamColor.bin` exact. TC-BIN-13's repair is tested at the writer on a working bin
    given to it; its CLI proof needs an installed bin, so the ID's citing test lands with
    4.21. Done 2026-10-04 (Opus 5.5, first time; two lead fixes: a root file whose second
    read fails gives no color and is not reported missing, and one test for a mutation
    survivor): `bins/mod.rs` (`TeamColorBin`: `read`, `repair_headers`, `set_colors`;
    `WorkingBins::bundled`, the seam 4.21 fills from the installed CPKs; `Rgb`, the two
    capacities); `validation::team_colors` reads the root file after the deep pass, a
    referee export's never; `plan_run` takes each export's colors and lists them in
    `BuildManifest::team_colors` for the exports the gate keeps, reporting
    `team_colors_missing` (compile only, after the gate's findings);
    `CpkOutput::finish(version, bins, team_colors)` adds `common/etc/TeamColor.bin` after
    `UniformParameter.bin` whenever a CPK is written and returns `bin_header_repaired`
    for `compile::run` to report; `validate_with` filters `KitFolder::colors` and
    `team_colors` by `file_kept`. The tracer's `TeamColor.bin` equals Red's byte for byte.
    69 CLI tests and one of `studio` gained the bin's entry or the `team_colors_missing`
    line, nothing else. Not covered: a `.7z` export with a root `colors.txt` (no fixture);
    `bin_header_repaired` from the CLI (4.21). Gates green (111 of 209);
    `mutants-diff b685288`: 37, 27 caught, 9 unviable, 1 missed (`read`'s record-count
    limit), closed by a test seen failing on the mutant
  - (c) `UniColor.bin`: the kit task's colors (its `colors.txt`, else derived, else the
    magenta/black pair) and icon, merged into the team's record by kit number; TC-KIT-11..14,
    TC-BIN-01..03; the tracer parity row `UniColor.bin` exact. Done 2026-10-04 (Opus 5.5,
    first time; two lead fixes, a help sentence and a test local's name): `bins/mod.rs`
    (`Records`, the plumbing both bins share: length check, a team's record, the header
    loop; `UniColorBin`: `read`, `repair_headers`, `set_kit`; `KitColorEntry`,
    `kit_number`; `WorkingBins::uni_color`); the kit task reads `colors.txt` and builds
    the entry (`processing/kit.rs`: `listed_colors`, `derived_colors`, `MISSING_COLORS`
    magenta then black, `DEFAULT_ICON` 3), reporting `kit_colors_derived` (I) or
    `kit_colors_missing` (W) on the kit folder; `TaskBatch::uni_color` travels beside
    `uniparam` and the writer applies it only for a batch that commits;
    `CpkOutput::finish` adds the bin after `TeamColor.bin` on every CPK and returns a
    second `bin_header_repaired` for it. Colors derive from the `kit` texture of the
    kit's effective set, one inherited from `all/` included (decision entry). The
    tracer's `UniColor.bin` equals Red's byte for byte. TC-KIT-14's GIVEN changed (a
    `kit.dds` no decoder reads: the deep pass refuses a non-UTF-8 `config.toml` before
    any task runs). 42 existing tests gained the bin's entry or a `kit_colors_*` line,
    nothing else. The derivation decodes the main texture once more than its conversion
    does (`dds_convert` shares no decode): measured on three real 2048 kits (release
    build, median of 5), a compile takes 0.167 s without `colors.txt` files against
    0.140 s with them (DXT1), 0.259 against 0.216 s (DXT5). Not covered: a main texture
    whose shirt region is fully transparent (magenta and black, by reading); the colors
    a raster (`.png`) main texture gives. Gates green (118 of 209); `mutants-diff
    ccbd7c4`: 77, 59 caught, 18 unviable, 0 missed
  The tracer's Studio fixture holds both `colors.txt` files since 2026-10-04 (lead)

- [ ] 4.9 **Kit configs, FPC reconciliation and collars** (the brief settles what a collar in
  the other engine's format does: converted, or refused; `Collars/` admits any model format):
  team kit-FPC status (`fpc_on` in any
  player folder → On), `fpc::kit_values` returning the four FPC values on every version,
  PES 15 and 18 included, so without its `Option` (both are `None` today, from a wiki page
  silent on them; PES 15 measured on the install, PES 18 confirmed by the maintainer;
  `libs/fpc.md` has the counts), with `kit_config`'s `apply_fpc`/`matches_fpc` simplified to
  match,
  generated configs with FPC values (`kit_config::fpc::apply_fpc`), supplied
  configs reconciled upward (`kit_config_fpc_adjusted`, GK included),
  `kit_config_version_clamped`; `fpc_on`/`fpc_off` no longer refuse the export (their savefile
  half is Phase 5); `Collars/` gets its allowlist row in `ae` (model files named `collar_<ID>`,
  any model format; the per-version stock sets are in `messages.md` `collar_id_invalid`,
  measured on every install, PES 20's included since 2026-10-03; waits on
  the maintainer: its confirmation, game-facing), `collar_<ID>` parsed with or without zero
  padding, `collar_id_invalid` (not a stock collar of the target version),
  `kit_collar_reserved` for a regular team's kit
  whose effective collar or winter collar is 77 (kit dropped; `blue_port.md` "Referee export
  processing", TC-REF-07),
  `collar_id_conflict` in canonical export order, the claimed-ID list starting with the
  reserved 105 (FPC) and 77 (the referees' marker), every kit config's collar fields rewritten
  after FPC, collar files passed through to `uniform/nocloth/#Win/` under the game's three-digit
  name. Plan: `fpc_toggle.md` "Team
  kit-FPC status and kit configs"; `pipeline.md` "4. Per-export non-model steps" (Kits, Collars),
  "Resolved decisions" (Collar contract); `object_model.md` "File-type allowlist". IDs:
  TC-KIT-15..17, TC-CMN-01..03. Crates: ae (`conventions/file_types.rs`, `validate/folders.rs`),
  tc (`processing/kit.rs`, `processing/team_assets.rs`, `plan/`) → verify: a `/co/` export with
  `fpc_on` in slot 05 and a supplied `p1/config.toml` without FPC values: the emitted 120-byte
  config decodes with `kit_config::fpc::matches_fpc` true and `kit_config_fpc_adjusted` is
  reported for p1; with `Collars/collar_12.fmdl` added, every emitted config's collar fields read
  12 and the file sits at `Asset/model/character/uniform/nocloth/#Win/collar_012.fmdl`.
  In slices (decision entry "4.9a: the FPC kit values take no version; the template carries
  them"):
  - (a) FPC reconciliation and the clamp warning: `fpc::kit_values()` with no version and
    `u8` values, `kit_config`'s `apply_fpc`/`matches_fpc` to match; the team's kit-FPC status
    from the validated export's player folders; a supplied config lacking the FPC values
    gets them when the status is On (`kit_config_fpc_adjusted`); `fpc_on` no longer refuses
    the export; `kit_config_version_clamped` from the deep pass. TC-KIT-15..17. Done
    2026-10-04 (Opus 5.5, first time, no lead code fix): `fpc::kit_values()`;
    `kit_config::apply_fpc(&mut config)` and `matches_fpc(&config)`;
    `plan::EffectiveTeamKitFpc::of(&export)`, carried by `TaskKind::Kit { fpc }`; the kit
    task (`processing/kit.rs`) applies the values to a config lacking them when the
    status is On and reports `kit_config_fpc_adjusted` (I, kit folder, kept), and never
    reverts; `plan/subset.rs` no longer names `fpc_on` as content not yet compiled;
    `deep/documents.rs` `kit_config_findings` runs `kit_config::validate` on a config
    that parses and reports `kit_value_out_of_range` and `kit_pattern_unsupported_pes15`
    as `kit_config_version_clamped` (W, on the file, kept; `field`, `value`, `max`), so
    `check` prints it too (`toml_finding` became `parsed_toml`, returning the parsed
    value); two help paragraphs. TC-OUT-06's and TC-OUT-02's tests now use an
    `ingame_face` export as their not-yet-compiled example. Found: the template's
    `name.y` is 30, over PES 15 to 20's 16 ("Phase 4 open questions"). Not covered: the
    PES 15 pattern clamp from the CLI (the deep pass's unit test has it; `compile`
    refuses PES 15); a goalkeeper config adjusted at the CLI (the task's unit tests use
    `g1`). Gates green (121 of 209); `mutants-diff f31075e`: 34, 24 caught, 10 unviable,
    0 missed
  - (b) collars (`Collars/`'s allowlist row, `collar_id_invalid`, `collar_id_conflict`,
    `kit_collar_reserved`, the configs' collar fields rewritten after FPC, the files passed
    through). TC-CMN-01..03. Waits on the maintainer's confirmation of the stock collar sets

- [x] 4.10 **Kit layout conversion**: `KIT_LAYOUT_REMAP` (the plan's four sock bands; the
  shorts are not re-laid) in `processing/kit_layout.rs`, `kit_layout_converted`, each band
  resampled along u from its own rectangle (Lanczos3), every mip level the source carries, the
  blocks no destination rectangle touches kept, `_chest`/`_back`/`_name`/`_leg` untouched,
  placeholder never re-laid, the Fox → pre-Fox direction tested at the function (its CLI test,
  TC-KIT-19, comes with pre-Fox targets at 4.16). Plan: `pipeline.md` "4. Per-export
  non-model steps" (Kits, "Layout conversion"); `player_folders.md` "Kit layout marker". IDs:
  TC-KIT-18, TC-KIT-20. Crates: tc → verify: a `pre-fox` kit compiled for PES 21 decodes to a
  texture whose every texel outside the sock rectangles equals the no-marker compile's and
  whose sock stripes' centres sit within 6 px of the golden; a synthetic flat-band texture
  round-trips pre-Fox → Fox → pre-Fox exactly
  - lead first, done 2026-10-04: the measurement (`scripts/provenance/kit_uv/`:
    `kit_uv_sock_angle.py`, `kit_uv_pants_height.py`, `kit_leg_atlas.py`), the plan's numbers
    and the golden (`tests/fixtures/kit_layout/`, written by `kit_layout_fixture.py`).
    Finding: the shorts are laid out alike in both engines; the earlier 36 to 60 px "shift"
    was the Fox body's height (decision entry)
  - done 2026-10-04: `dds_convert::resize` (`resample.rs`; Lanczos3 through `image`, the
    suite's one resampler); `processing/kit_layout.rs` (`KIT_LAYOUT_REMAP`, `relaid`); the
    kit task re-lays a marked kit's main texture and converts it outside the converter's
    cache, `kit_layout_converted` (from, to); the gate no longer refuses a `pre-fox` kit;
    help paragraph; `tests/cli/kit_layout.rs`. TC-KIT-18 and TC-KIT-20 proven (stripe
    centres within 4.5 px of the golden, 6 allowed). Not covered: a marked kit whose main
    texture is inherited from `all/`, raster and BC3 sources from the CLI, the lower mip
    levels in TC-KIT-18 (blocks straddling a rectangle's edge are encoded afresh there), a
    BC7 source keeps no blocks (encoded whole, as an unmarked raster is), and the mask and
    srm, which are re-laid when their steps emit them (4.16 for the mask). Gates green (123
    of 209); `mutants-diff 482c92d`: 150, 132 caught, 14 unviable, 4 missed (a block touching
    a rectangle's edge counted as reached), closed by a test of `reached_blocks`

- [x] 4.11 **Team root artifacts and Common**: logo (main decoded, made square per tag, Lanczos3 to
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
  appear in `output/teamnotes.txt` in canonical order and a skipped export's do not.
  Worked as four slices, each its own commit (decision entries of 2026-10-04: the logo's
  geometry; `teamnotes.txt`; kit variant sets):
  - [x] 4.11a the logo (TC-ROOT-06..08): done 2026-10-04 (Opus 5.5, first time, no lead
    fix). `dds_convert::encode_png` (`raster.rs`); `TaskKind::Logo`, one task per export
    after its kits, scoped on the main file, `DropFile` when it fails;
    `processing/team_assets.rs` (`logo`, `squared`, `centre_square`, `letterboxed`);
    `paths::logo`; `logo_fit_applied` (I) and `logo_upscaled` (W); the gate no longer names
    a logo; help. Red runs and five perturbations in the sidekick's report (padding before
    resampling shows as `[5, 0, 0, 5]` where `[0, 0, 0, 0]` is expected at (0, 100) of 512).
    Not covered from the CLI: sources other than PNG (a DDS at the function), the `emblem_0`
    names (PES 18 and 19; at the function), a tall `stretch` source. Gates green (126 of
    209); `mutants-diff 0e3739b`: 103, 94 caught, 7 unviable, 2 missed, both equivalent
    (`width > height` against `>=` in `centre_square` and `letterboxed`, which are called
    for a non-square image only)
  - [x] 4.11b the notes (TC-ROOT-09): done 2026-10-04 (Opus 5.5, first time, no lead code
    fix). `CheckedSource::notes` (the text read while the source is open, BOM removed);
    `plan_run` takes `ExportToPlan` tuples and fills `BuildManifest::notes` for the exports
    it keeps; `output/teamnotes.rs` (`render`, `write` through `.teamnotes-<pid>.tmp`);
    `compile::run` writes or removes the file once the CPK is promoted;
    `teamnotes_write_failed` (E, on the run); the test sandbox's teams list gains `/a/` and
    `/b/`; help. One brief premise was wrong (a `/b/` skipped by
    `players_txt_slot_duplicate` is dropped by validation, so no collection point could
    show its note; the planning test uses an export the gate skips instead). Known shape:
    `ExportToPlan` is a four-field tuple alias, kept because 27 test call sites build it
    inline; a struct is the next step if it grows. Not covered from the CLI: the removal
    failing, the temporary file failing to be written, a `--no-deploy` run with notes. Gates
    green (127 of 209); `mutants-diff f0f9002`: 19, 16 caught, 3 unviable, 0 missed
  - [x] 4.11c kit variants on Fox (TC-CMN-04, TC-CMN-05): done 2026-10-04 (Opus 5.5, first
    time; one lead fix, a test comment). `kit_variants.rs` (`kit_token`, `variant_stem`,
    `kit_number`, `model_variant_sets`); `TaskKind::Textures` and `CommonTextures` carry the
    export's kit numbers and `texture.rs` `complete_kit_variants` fills each set's gaps with
    the lowest variant's bytes (`kit_variant_missing`, W); `point_texture` points a `kitN`
    path at the part's textures when a variant of its set is among them;
    `PlayerFile::LeftOutKitVariant` for a higher per-kit model, which no task reads, and
    planning's `kit_variant_model_fox` (W) once per set and folder; help. TC-CMN-05 was split:
    it is the PES 21 half, and the PES 17 half is the new TC-CMN-07 (4.14), so no scenario
    counts as proven by half. Left as they are, each rare: a per-kit `.common` model link is
    not left out (the links merge like any parts); a skeleton named after a left-out variant
    has no model to pair with, so the gate names it; `check` does not print
    `kit_variant_model_fox` (a planning finding, as `link_combined` is); a reference is
    matched as spelled (`Pants_kitN` beside `pants_kit1.dds` is not). Not covered from the
    CLI: per-kit models in a reserved subfolder or a shared folder, a Common set completed
    (at the function), a set across a player's folder and a combined shared folder.
    Gates green (129 of 211); `mutants-diff 2249bf1`: 61, 48 caught, 13 unviable, 0
    missed. TC-CMN-06 (`dummy_kit*`) is
    4.29's, which builds the checks it is about
  - [x] 4.11d texture `.common` links (TC-TEX-09): done 2026-10-04 (Opus 5.5, first time, no
    lead code fix). `PlayerFile::CommonTexture(stem)` for a link to a texture, anywhere a
    link resolves (not in the `common/` subfolder, where validation checks no link);
    `processing/model.rs` `point_texture` takes the places a part's textures may be, in
    order: the folder's own stems at its texture home, then the linked stems at the team's
    Common output, a kit reference resolving in each; the gate names only material links
    now; help. Two contradictions: the `common/` exclusion (applied by the sidekick,
    accepted), and the brief's premise that validation refuses every stem held two ways
    (wrong for a combined shared folder: "Issues"). Not covered from the CLI: per-kit
    textures linked one by one (at the function). Gates green (130 of 211);
    `mutants-diff 2b0a388`: 26, 22 caught, 4 unviable, 0 missed

- [x] 4.12 **`ingame_face` processing (Fox)**: no face package emitted; arbitrary-named models
  rerouted to the player-exclusive boots folder (merged, the paired `.skl` becoming `boots.skl`);
  gloves parts to the player's gloves folders; a boots/gloves link combined with local parts
  under the marker; an empty `face/` ignored; a player without face models and without the marker
  gets the blank face folder (its contents per engine: `pipeline.md` step 4), always with the
  bundled face diff; a face file left in a folder with no face model is `face_file_not_used`
  (Info; 467 face folders of the VGL26 exports hold one, `.tmp/face_diff_blank_census.py`).
  Plan: `player_folders.md` "`ingame_face` marker", "`ingame_face` with shared links";
  `pipeline.md` "2. Per-export serial steps" step 4. IDs: TC-MOD-16..19, TC-MOD-32 (new).
  Done 2026-10-04 (sidekick, one brief, landed with two lead fixes to tests): the reroute is
  one branch of `plan/subset.rs` `model_role`, the marker carried by `FolderModels`
  (`of_player_files`) and `ModelFolder::ingame_face`, so the gate, `link_combines`, the ids,
  the findings and the tasks agree; `PlayerFile::UnusedFaceFile`, reported by
  `validation.rs` (`check` too); `plan/mod.rs` `folder_tasks` plans the face of every
  unmarked player folder (`blank_face`), and `processing/model.rs` `package` builds the
  blank one unchanged; the gate no longer names the marker or a model-less folder; help;
  `tests/cli/face_folders.rs`. The pre-Fox blank folder comes with 4.14. TC-MOD-09 is
  reworded (decision entry): a failed package is left out alone, the blank face and the
  textures still packed ("Phase 4 open questions"). Known, not fixed: under the marker a
  texture of the player's own that differs from a combined `Boots/` folder's is
  `shared_texture_conflict` (dropped=boots) where both feed the boots, by the rule that a
  player's own textures stand for its face (the folder's models are left out either way); a
  marked folder holding only textures emits nothing and reports nothing; a `Faces/` folder
  with no face model also gets `face_file_not_used` from `check` (`compile` refuses the
  folder). Not covered from the CLI: those three, and a marked folder whose rerouted models
  bring different skeletons (`skl_merge_conflict`, seen by the sidekick's probe). Gates
  green (135 of 212); `mutants-diff 03d4fae`: 50, 43 caught, 7 unviable, 0 missed

- [x] 4.13 **Run planning**: `duplicate_aesthetics_export`; canonical export order by normalized
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
  [Keep]`. Worked as three slices (decision entry of 2026-10-04):
  - [x] 4.13a duplicate exports and the export order (TC-PLN-03, 05, 07), done 2026-10-04
    (sidekick, landed first time): `validation.rs` `refuse_duplicate_teams`, after every
    source is checked, so `check` reports it too: each export of a team several resolve to
    gets `duplicate_aesthetics_export` (context `id`, `exports`) and is not planned; the
    export order is discovery's, unchanged. Two exports of one team with disjoint content
    used to compile into one CPK with no finding, and colliding ones aborted the run at the
    writer. TC-OUT-03's test now blocks the staging folder with a file to get its failed
    CPK write (it used two exports of one team); no CLI test reaches the writer's duplicate
    invariant any more (its unit test stays). Not covered: an export dropped by an exhausted
    ID pool beside another of its team (no duplicate finding, by the rule that only a
    resolved export counts); a folder beside a `.7z` through `compile`. The sidekick looked
    for two tasks emitting one CPK path and found none (paths are keyed by team, player,
    boots or gloves id, kit name or folder name, and the in-export cases are refused or
    merged before tasks). Gates green (138 of 212); `mutants-diff d053ac9`: 7, 6 caught, 1 unviable, 0 missed
  - [x] 4.13b the `overrides/` tree and `duplicate_path` (TC-PLN-04), done 2026-10-04
    (sidekick, landed first time; one lead fix, a help line rewrapped): `plan/overrides.rs`
    `list` (every file below `<data dir>/overrides/` by CPK path, and the `overrides_active`
    note), listed by `compile` before any export is read; `output/writer.rs` `CpkOutput`
    adds them when it creates the CPK, in path order, each read as it is added, and creates
    the CPK at `finish` when nothing else was committed; `add` leaves out an entry or a bin
    at an override's path and reports `duplicate_path` (Warning, on the run) among that
    task's messages or `finish`'s. A run with overrides and no export compiled now writes
    and promotes a CPK holding the overrides and the bins. Known, not fixed: a bin an
    override replaces is still built, so `bin_header_repaired` is still reported for it and
    a failed kit config insert still fails the run (neither reachable on the bundled bins;
    to settle with 4.21's installed bins). Paths compare exactly, by the maintainer's
    ruling (decision entry of 2026-10-04: no case folding). Not covered from the CLI: an `overrides/` tree that cannot be listed or a
    name that is not UTF-8 (exit 3), an unreadable override (unit-tested at the writer), an
    override at `UniformParameter.bin` or `UniColor.bin`. Gates green (139 of 212);
    `mutants-diff 62d8e66`: 28, 16 caught, 12 unviable, 0 missed
  - [x] 4.13c source pinning, `source_changed_during_run` (TC-PLN-06), done 2026-10-04
    (sidekick, landed first time, no lead fix): `reader/source.rs` `list` also returns the
    source's `SourceRevision` (a folder: each file's size and modified time, from the
    metadata the walk already asks for; an archive: the archive file's), carried by
    `Route::Validate` and `CheckedSource`; `compile.rs` `coordinate` asks
    `SourceRevision::changed` after each task's read (a `.7z` once, after its one read) and
    stops at the first change; `run` is `plan` then `build`, and `build` discards the
    staging and reports the Fatal on the export, naming the file. A file gone before its
    read is a change, not a `source_read_failed`. TC-PLN-06's test replaces `boots.fmdl`
    (nothing staged yet) and, in a second pass, a kit texture (a staged CPK exists and must
    go): with the boots alone the staging assertion could not fail. Not decided: an aborted
    run sends no `ExportProcessed` for the exports it did not finish (as after
    `cpk_write_failed`; the console prints nothing for it, Phase 8's grid may want it);
    events still carry no export revision (Phase 8). Not covered from the CLI: a `.7z` or
    `.zip` changed during a whole compile (covered at `coordinate`), a change in a second
    export after the first one's batches were committed. Gates green (140 of 212);
    `mutants-diff 527423c`: 26, 15 caught, 11 unviable, 0 missed

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
  names". (The pre-Fox parity reference for 4.14-4.17 is step 4.31's, the lead's; until it
  exists the step's checks are the ones below.)
  Also the pre-Fox half of the kit variants (4.11c did Fox): a model variant set as one
  `face.xml` entry naming `…kitN…` with the variant files beside it, and the texture sets
  completed as on Fox.
  Also the pre-Fox blank face folder and the pre-Fox half of `ingame_face` (4.12 did Fox):
  `pipeline.md` "2. Per-export serial steps" step 4 gives the blank folder's contents, and
  `face_file_not_used` is Fox-only until then.
  Open first (found at 4.11c): a model's type is read from its stem's last part, so a
  per-kit model named `boots_kit1` is typed as face content; typing should probably skip
  the kit token.
  IDs: TC-MOD-20..25, TC-TEX-07, TC-CMN-07 (split from TC-CMN-05 at 4.11c). Crates: tc (`processing/model.rs`, `processing/material.rs`,
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

- [ ] 4.16 **Pre-Fox kits, bins and DDS compression**: a kit marked `fox` has its main texture
  and its own mask re-laid to the pre-Fox layout (`kit_layout::relaid`, TC-KIT-19), `kit_mask` injected from the mask template
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
  no package has a slot for it). Open first (found at 4.5c): a `.common` link to a
  material file is refused only by the gate (Phase 7 builds them; texture links compile
  since 4.11d, except in a player's `common/` subfolder or a shared folder, where the gate
  names them); a `Common/legs.skl`
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
  `4cc_08_bins`, `4cc_61_midcup` and `4cc_99_test` with a `4cc_08_bins.cpk` holding a
  `UniColor.bin` in which team 714's p1 entry is set and a `4cc_61_midcup.cpk` holding one in
  which it differs: compiling `/co/` with only `p2/` leaves p1's bytes equal to the
  higher-priority CPK's and p2's set; with `cpk_name = 4cc_61_midcup` the p1 bytes come from
  `4cc_08_bins.cpk`

- [ ] 4.22 **Fox player tables**: `BootsList.bin` and `GloveList.bin` read from the installed CPKs
  by the same walk (the seed rows ride in `4cc_08_bins.cpk`), the (player id, item id) pair of
  every compiled player whose
  boots/gloves output committed replaced with the planned ID, a failed output keeping its row,
  a compiled player the `Full` export gives no boots or no gloves losing that installed row, every
  other row kept, written whole, plain, sorted by id; `PlayerAppearance.bin` read and
  written whole with no row changed (the rows are Phase 5's, with the stock and default-ID
  boots/gloves rows). **External:** the in-game effect of a gloves row waits on FoxDen's gloves
  patch (worklog "Issues"); the table content does not. Plan: `pipeline.md` "Bins accumulation"
  (player appearance tables), "Game paths reference" (the three rows); `settings_toml.md` "Player
  settings in exports" (Fox); `development_plan.md` "Phase 4" `bins/`. IDs: TC-BIN-10..12,
  TC-BIN-17.
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
  sandbox PES folder with `PES2021.exe`, a DPFL listing `4cc_99_test` and an old
  `download/4cc_99_test.cpk`: `compile` leaves `download/4cc_99_test.cpk` equal to the staged
  bytes, no `.partial`, the marker beside it, nothing in `output/`, exit 0; with the old CPK held
  open by the test: `old_cpk_locked`, `output/4cc_99_test.cpk` holds the run's CPK, the old one
  is byte-identical, exit 1

- [x] 4.25a **The default `cpk_name` is `4cc_99_test`** (lead; mechanical): `4cc_90_test`
  renamed in the code and its tests (81 mentions in 17 files of `crates/`), the plan having
  it already. Done 2026-10-03, by script; gates green. The tracer fixture's README keeps
  `4cc_90_tracer`, the name Red's golden run used → verify: no `4cc_90_test` left under
  `crates/`, `just gates` green

- [ ] 4.25 **DpFileList upgrade and the official-list check** (the entry list is fixed:
  `resources/templates/DpFileList.txt`, 53 entries; before this step, 4.25a renames the
  default `cpk_name` in the code): the one official `DpFileList.bin` embedded; on every
  compile with a PES folder the installed list's entries compared with it in order,
  `dpfilelist_not_official` (W) for any difference that still lists the run's targets, the
  run compiling and deploying all the same (TC-DEP-12) (lead first: the
  file written from the maintainer's entry list in the layout measured at 4.21, as a
  lead-authored fixture with a provenance README; no installed list has the slot run, PES 17's
  still lists `4cc_40_faces` and `4cc_45_uniform`; the brief confirms with the reader of 4.21
  that one file serves every PES version, and reports if a version needs its own bytes);
  `upgrade-dpfl [--yes]`: the entries the official list lacks printed with the size of each
  matching `download/*.cpk`, nothing written without `--yes`, the installed file replaced byte
  for byte with `DpFileList.bin.bak` kept, an old DLC's CPKs renamed by stem to the official
  names (TC-DEP-13), the empty placeholder CPK written for every official entry with no file
  in `download/` (an existing file is never overwritten), no CPK ever deleted by the command;
  `dpfilelist_cpk_missing` (W) at every compile for a listed CPK with no file (TC-DEP-14);
  `dpfilelist_outdated` names the subcommand. Plan: `pipeline.md` "6. Post-processing"
  (DpFileList upgrade); `settings.md` "CLI" (`upgrade-dpfl`). IDs: TC-DEP-08..10, TC-DEP-12..14. Crates: tc
  (`bins/dpfl.rs`, `cli.rs`), resources → verify: an installed DPFL lacking `4cc_41_teams` beside
  a 1 KiB `download/4cc_40_faces.cpk`: `upgrade-dpfl` prints `4cc_40_faces` with `1 KiB` and
  exits without writing; `--yes` makes `DpFileList.bin` equal to the embedded list and
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
  `4cc_41_teams`..`4cc_45_teams`: `4cc_41_teams.cpk` holds the first two teams whole,
  `4cc_42_teams.cpk` the third, `4cc_43`..`45` are each 6,272 bytes equal to the placeholder
  fixture, `4cc_08_bins.cpk` holds the bins and nothing else

- [ ] 4.27 **Referee marker as reserved collar 77, both engines** (lead first: the two
  marker models bundled as referee templates with a provenance README, from the sources
  named under "Phase 4 open questions", each checked to be painted to `static`): the marker model bundled as a
  referee template and emitted in the refs CPK as collar 77, its texture path naming
  `ref_marker.dds` converted into the referees' Common output, the referee template kit configs
  naming collar 77; on a regular team, `collar_id_conflict` for `Collars/collar_77.*` and
  `kit_collar_reserved` (kit dropped) for a kit whose effective collar or winter collar is 77
  (the kit half lands with 4.9 if that step comes first); nothing written outside the refs CPK
  (no `dt00_x64.cpk` write, no setting). Plan: `blue_port.md` "Referee export processing";
  `messages.md` "Referees", `collar_id_conflict`; `pipeline.md` "Collars". IDs: TC-REF-04,
  TC-REF-06..07. Crates: tc (`processing/referee.rs`, `processing/team_assets.rs`,
  `processing/kit.rs`), resources → verify: a refs export with `ref_marker.dds` compiled for
  PES 21 beside a sandbox `dt00_x64.cpk`: the refs CPK holds `nocloth/#Win/collar_077.fmdl`
  whose texture path resolves to the marker's FTEX under `common/999/sourceimages/`, each
  referee kit config decodes with collar 77, and `dt00_x64.cpk` is byte-identical; a `/co/`
  export with `p1/config.toml` naming collar 77 reports `kit_collar_reserved` and p1 is absent
  from the CPK

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

- [ ] 4.29 **A texture a model names must exist** (after 4.21, whose walk of the installed CPKs
  it reuses): `fmdl_texture_not_found` on Fox for a mesh's texture supplied by nobody: stem not
  in the folder, path naming the team's Common output, and neither the export's `Common/` nor an
  installed CPK's table of contents holding it, the CPKs searched being those the installed
  list names before the CPK being compiled, nearest first, never a later one; an Error dropping
  the folder, a Warning keeping it when the lookup cannot be made; a texture `.common` link with no target in the export
  satisfied by an installed CPK the same way; paths naming anything else kept and not looked
  up. The pre-Fox half (`mtl_texture_not_found`) lands with 4.15 under the same rule. Plan:
  `pipeline.md` "Resolved decisions" ("A texture a model names must exist"); `messages.md`
  (`fmdl_texture_not_found`, `mtl_texture_not_found`). IDs: TC-TEX-05, TC-CMN-06 (the
  `dummy_kit*` stems the checks skip; moved here from 4.11). Crates: tc (`check.rs`,
  `processing/model.rs`, `bins/`) → verify: TC-TEX-05's three runs (the CPK holding the
  texture, lacking it, no PES folder): no finding, Error with the folder out of the CPK, Warning
  with the folder in it

- [x] 4.30 **Marker names**: done 2026-10-03 (Opus 5.5, first time; one contradiction accepted:
  it also renamed the markers in `pes_savefile`'s injected `settings.toml` comments, outside its
  crates). The FPC markers are `fpc_on`/`fpc_off`, a bare `fpc` reads as on, and a kit's menu
  icon is the empty marker `icon_<N>` (`Marker::Icon`; 0-23, zero padding optional; two
  markers, or a number above 23, are `kit_icon_invalid` and the default icon applies), so
  `icon.txt` left the small metadata the readers load; every marker tolerates `.txt`; the old
  spellings are plain disallowed files. Renamed in every plan section, the glossary and the
  fixtures (the tracer's `Kits/g1/icon_11`, the three source archives regenerated by the
  lead). TC-STR-12 and TC-KIT-08 reworded. Files: ae `conventions/file_types.rs`,
  `conventions/mod.rs`, `listing.rs`, `validate/{kits,folders,mod,tests}.rs`; ps
  `settings_toml/{keys,tests}.rs`; tc `plan/subset.rs`, `reader/source.rs`,
  `tests/cli/compile.rs`. Gates green (99 of 203); `mutants-diff a89f0b6`: 41, 35 caught, 6
  unviable, 0 missed. `rg "fpc\.on|fpc\.off|icon\.txt" crates` leaves only the tests that
  assert the old names are refused. Left for Phase 5: `keys.rs`' boots/gloves comments still
  say "no marker: untouched", where `settings_toml.md` has said "otherwise 0" since step 4.0

- [ ] 4.31 **Pre-Fox parity reference** (lead, before 4.14): a small pre-Fox export cut from one
  in the maintainer's library (one player with face, boots and gloves, one kit, as the Fox
  tracer was cut), its Studio-layout twin migrated by hand, and Red's CPK for the old one
  compiled for PES 17 with the tracer's settings (`tests/fixtures/tracer/README.md` "`red/`":
  no PES folder, so every bin is built on Red's fallback base), committed under
  `tests/fixtures/tracer_prefox/` with a provenance README if under 1 MB, as a hash manifest
  otherwise. Without it every pre-Fox byte of 4.14-4.17 is unchecked until Phase 6 → verify:
  the README's command reproduces `red/` byte for byte from `old/`

- [ ] 4.32 **Number atlases re-arranged across engines** (lead first: the measurement):
  neither engine reads the other's `_back`, `_chest` and `_leg` arrangement (ten digits in
  a column on PES 15 to 17, 128×2048 or 64×1024; in a row on PES 18 to 21, 2048×256 or
  1024×128), so an atlas in the other engine's arrangement is re-arranged for the target,
  told by its shape alone, `_name` untouched. The cells are not whole pixels (2048 over
  ten), and a column cell and a row cell differ in proportion, so the lead first measures
  where each game takes each digit from (the stock atlases and how the number meshes map
  them) and records it as a table with a provenance script, as `KIT_LAYOUT_REMAP` was;
  then the re-arrangement is briefed. Plan: `pipeline.md` "4. Per-export non-model steps"
  (the glyph atlases). Crates: tc (`processing/kit.rs`, `processing/kit_layout.rs`) →
  verify: a column atlas whose ten cells are ten flat colors, compiled for PES 21, comes
  out as a row atlas with the ten colors in digit order, and the reverse for PES 17; an
  atlas already in the target's arrangement is byte-identical to today's output

- [ ] 4.33 **`name.y` is PES 21's value on every version** (decision entry of 2026-10-04):
  `kit_config` decodes and encodes Name Y the PES 21 way on every version (6 bits from 0x1C
  bit 3 and 0x1D bit 0; the version branch in `binary.rs` goes), the maximum being 33 on
  PES 15 to 20 and 39 on PES 21 (`model.rs`'s limits table), so the template's 30 is no
  longer clamped and `kit_config_version_clamped` fires only over 33. Plan:
  `kit_config_editor.md` (layout rows 0x1C and 0x1D, "Version differences"). IDs: TC-KIT-17
  (reworded: 36 clamps to 33), TC-KIT-24. Crates: `kit_config`, tc (tests and help only) →
  verify: the template encodes to its own bytes for PES 17 and for PES 21; a PES 17 binary
  with 0x1C bit 3 set decodes and encodes back byte-identical with no `unknown` entry for
  that bit; `just bindings` if the bindings expose the crate

- [ ] 4.34 **Coverage tag: `Full` or `Midcup` in a team export's name** (decision entry of
  2026-10-04; before 4.21, whose TC-BIN-05 and 06 are Midcup cases). Plan:
  `aesthetics_export/object_model.md` "Validation semantics" (Coverage tag, `ExportCoverage`,
  the two `coverage` fields); `team_compiler/pipeline.md` "Bins accumulation";
  `messages.md` `export_tag_missing`. Slices:
  - [ ] 4.34a the tag is read and required: `aesthetics_export` gives the coverage of a name
    (second word, any letter case; a referee export is Full), the draft and the validated
    export carry it, a team export with neither word is `export_tag_missing` (E, skipped;
    `check` and `compile`), and every test export, fixture archive, golden and help example
    is renamed with its tag (TC-ID-01 and TC-PLN-03 already name tagged exports; their
    tests still use untagged names until this slice). IDs: TC-ID-05, 06, 07 → verify: the
    three scenarios; the parity test's CPK is unchanged by the renames.
  - [ ] 4.34b `Full` resets the team's `UniColor.bin` record (its committed kits, the count
    theirs, every other entry unused) and `Midcup` keeps today's merge, a new kit raising
    the count. IDs: TC-BIN-14, 15 → verify: the two scenarios on the bundled base.
  - The rest lands with the steps that own the data: the team's stale kit configs removed
    from `UniformParameter.bin` and no FPC patching for a Full export, with 4.21
    (TC-BIN-16); a Full export's compiled players without boots or without gloves losing the
    matching installed row, with 4.22 (TC-BIN-17); the savefile fields, Phase 5; the upgrader's tag
    (`midcup` or `additions` in the old name), Phase 6; the two buttons on an untagged row,
    Phase 8.

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

- The sock table's look in-game (4.10): `KIT_LAYOUT_REMAP` approximates, in two bands, a map
  the models give to within about 10 px of 2048 (`pipeline.md` "Layout conversion"). To
  settle: a pre-Fox kit with a design on its socks (hoops do not show it; a vertical stripe
  or a logo does), compiled for a Fox game with the `pre-fox` marker and looked at in-game.
- Collars beyond the stock set (4.9, 4.27; the maintainer's idea, 2026-10-03): PES 15 loads
  `collar_105`, which its stock game lacks, so the games probably accept collar IDs they do
  not ship. If so, the FPC collar and the referees' marker could move to IDs no stock collar
  uses (still reserved), and a team could replace or add any ID nobody else claims, which
  would retire `collar_id_invalid`'s stock-set rule. What is known: a kit config stores each
  collar in one byte (`kit_config` offsets 0x14 and 0x15), so a kit can name 1-255 at most,
  whatever the file name allows; the only non-stock collar in any install is that PES 15
  one, an empty 852-byte model, so nothing shows yet that a Fox game (18-21) loads one, or
  that a visible model at a non-stock ID renders. To settle it: an in-game test per engine,
  a kit naming an ID above the version's stock set (200, say) with a visible collar model
  under that name. Until then 4.9 keeps the stock-set rule and the reserved 105 and 77.

Answered 2026-10-03 (decision entries of that date; each is in the plan): the official
DpFileList's entries (`resources/templates/DpFileList.txt`, the maintainer's own edit of the
lead's draft: 53 entries, the test CPK `4cc_99_test`) and the check of the installed list on
every compile (`dpfilelist_not_official`); the `colors.txt`
grammar (one color per line, in both files); the kit icon as a marker file and the marker
spelling (4.30); the Fox referee marker as a reserved collar instead of a `dt00_x64.cpk` write
(4.27); the default `cpk_name` on PES 21 (intended: that install's list is obsolete, and the
upgrade to the one standard list is the path); one official `DpFileList.bin` for every version
(4.25), whose 53 entries the game loads whole (the maintainer's in-game tests on PES 2015 and
PES 2021: each install's own list extended to 53 entries with placeholder CPKs, the test CPK
last, `.tmp/dpfl_pes15_53.py` and `.tmp/dpfl_pes21_53.py`; the versions between are taken to
hold it too); `face_diff.xml` decoded with the `base64` crate (4.5d); a model's missing texture is an
Error, with the installed CPKs searched for a partial export's Common textures (4.29), and
`fmdl_no_texture_ids` is dropped; a format finding drops its folder by severity (4.7). The
pre-Fox parity reference was never a question: it is the lead's step 4.31. The kit layout
referee marker (4.27) waits on nothing: its models are the 4cc's `referee_prop.fmdl`
(`C:\Data\4cc\Tools_Mine\4cc refs compiler\referee_prop.fmdl`, texture `cup_logo.dds`
beside it) and Red's pre-Fox referee template's `referee_prop.model` with its `.mtl`
(`Engines/templates/refscpk_prefox/common/character1/model/character/parts/referee/`), and
the square stays on the ground as a collar by static painting (`blue_port.md` "Referee
export processing"). The far-vertex rule stands as it is (4.7): the lead's census of the
VGL26 corpus (2,938 FMDL, 21 `.model`; `.tmp/fmdl_census/corpus_census.txt`, `far.txt`)
found one Error class, `fmdl_vertex_far_from_origin`, in 89 `face_high`/`hair_high` files of
7 exports, every vertex of each parked 14,660 units or more from the origin, and those
parked placeholder faces and hair are the main cause of the matchday lag the rule exists
for, so 4.7 drops their folders. The kit layout
table (4.10) takes its numbers from the games' uniform models alone: the pair made with PES
Master's two kit creators (scripts, renders and `FINDINGS.md` in `.tmp/kit_creator/`) agrees
with them on the socks and not on the shorts, and the models win (`pipeline.md` "Layout
conversion").

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

- open — the compiler reports only two of `kit_config::validate`'s findings (the two a
  version's encoding clamps, as `kit_config_version_clamped`, step 4.9a). The others have no
  row in `messages.md`: `kit_collar_zero` (the lib's one Error; `messages.md` says
  `kit_config_invalid` covers a config that "fails to validate", which the deep pass does
  not do), `kit_shirt_model_unknown`, the three "requires model 144 or 160" warnings and
  `kit_unknown_sleeve_value`. Before a kit is dropped for a zero collar, count how many real
  configs carry one (the old-format exports' `Kit Configs/` and the installs' CPKs).

- open — the headless egui test harness (a frame with AccessKit on, a node by label, a click as
  press and release frames) exists twice, in `studio_core/src/shell/mod.rs` and
  `team_compiler/src/view/mod.rs` (3.y design sweep). The next tool view in Phase 8 would be
  the third copy: extract it then into `studio_core` behind a test-support feature, rather
  than a third copy.
- open — for Phase 8's settings menu, the first code that saves `settings.toml`: the `[common]`
  table is read into `CommonSettings`, so a key it does not know (a newer version's) is lost on
  save, while a tool's table keeps unknown keys (`core/gui.md`). Settle whether `[common]` keeps
  them too before that save path lands (review S2.1).
- open, needs the maintainer — the boots/gloves rows a `Full` export leaves (`pipeline.md`
  "Bins accumulation", TC-BIN-17, decision "a team export's name carries `Full` or `Midcup`").
  Phase 4 (4.22) drops the installed row of a compiled player the export gives no boots or
  gloves, so he has no row until Phase 5 writes the default-ID rows; what a player with no
  `BootsList`/`GloveList` row wears on a stripped save is untested in game. Also unsettled: a
  team player the `Full` export does not compile at all (no folder), whose installed row may be
  a seed row with a stock ID or name the team's own block: kept, dropped, or written 0? Settle
  before 4.22 (review S5.1 item 9, S5.2 items 1 and 7).
- open, needs the maintainer — `messages.md` (around line 333, "the PES16 exe will be patched
  to allow loading models from Common") and `player_folders.md` (around line 222, "on PES16 via
  the patched exe") rely on a PES 16 exe patch. Does it exist already (then "will be" is stale
  tense), or is it a planned exe change, which the "Fox first" decision defers for pre-Fox
  until a pre-Fox version is the cup's game again (review S5.4 item 4)? The same question for
  `kitN`: `model_format.md` "Kit-dependent assets" (the exes "are being modded to look for"
  `kitN`) and `testing.md` "Kit-dependent path magic" describe exe modding on both engines; on
  Fox the decision routes such a change through FoxDen (review S5.7 item 2).
- open, needs the maintainer — generated `settings.toml` files opt in to names: the Save
  editor's and the Export upgrader's generation write `name` and `shirt_name` for every player
  they generate, and the Team creator writes `name = true` in every player folder it
  creates (`team_creator.md`, review S5.A3 item 2), so a
  managed team that migrates through them holds an export whose aesthetics patch (applied
  whole, decision 2026-10-05) writes the generated names over any rename its manager makes
  later. Intended (the author deletes the keys), a note at generation, or generation leaves
  names out for managed teams (review S5.7 item 8)?
- open, needs the maintainer — the seed rows and multi-CPK mode: strip-and-seed puts its rows
  into `4cc_08_bins.cpk` (decision 2026-10-05), but in multi-CPK mode that CPK is the
  compiler's own bins output (`settings.md` `bins_cpk_name`), and the bins walk starts below
  the output CPK (`pipeline.md` "Bins accumulation"). So the DLC builder's next multi-CPK
  compile reads the three player tables from below the seed and replaces `4cc_08_bins.cpk`
  without the seed rows of every player no export compiles. Options: the walk reads the three
  player tables from the output CPK too (compiled players' rows are rebuilt every run, so the
  walk stays idempotent for them), or the seed goes into a CPK of its own listed below
  `4cc_08_bins` (review S5.A1 item 1).
- open, needs the maintainer — motions on Fox: `settings.toml`'s `[appearance.motion]`
  (hunching, arm movement, kick motions, celebrations, dribbling) are player-record fields
  outside the appearance block (PES 20/21 bits 96 to 332), so neither the `PlayerAppearance.bin`
  row (the block's bytes) nor the Fox patch (names only) carries them: on Fox a compiled
  export's motions reach nobody. Does the Fox patch carry the motion keys too (whether the game
  reads them from a stripped record is untested), or do motions become the manager's, edited
  in the save editor like gameplay (review S5.A1 item 2)?
- open — the unknown bits across versions: `settings_toml.md` names them by record bit
  position per version and never converts them, but the ingame-face run's layout is the same
  on every version (`model.md` "The ingame-face run"), Team TOML prefix-copies it between
  versions today, and a PES 16 export may be compiled for PES 21, so per-version tables drop
  hair and sliders a conversion could keep. Settle before Phase 5 removes the hex: tables
  named by position in the run rather than the record, or converted by the run's offset
  (review S5.A1 item 8).
- open — the base-copy ID on pre-Fox: `model.md` makes it compiler-owned ("derived from what
  was written"), but no plan says what the compiler derives, and the aesthetics patch, the
  only savefile write path, has no key for it (Team TOML has `base_copy_id`). Settle with
  Phase 5's patch: the rule and its key (review S5.A1 item 10).
- open — the three Fox player tables with no installed copy: a compile with no PES folder or
  DpFileList builds the bins on bundled bases, but there are bases only for `TeamColor`,
  `UniColor` and `UniformParameter` (`pipeline.md` "Templates and fallback bins"). For
  `PlayerAppearance`, `BootsList` and `GloveList`: a bundled base, the compiled rows alone, or
  no table? Settle before 4.22 (review S5.A1 item 12).
- open — shared IDs and retained rows: a team's shared boots/gloves IDs are assigned in
  alphabetical folder order from the export being compiled (`player_folders.md` "Assigns IDs
  automatically"), while a player the export does not compile keeps his installed row
  (`pipeline.md` "Bins accumulation"). A `Midcup` export holding shared folder B but not A
  gives B the ID A had in the full CPK, so A's wearers, their rows untouched, now wear B.
  The same holds for pre-Fox save IDs. Settle before 4.22: a midcup keeps the full export's
  assignment (from where?), or must carry every shared folder its team uses (review S5.A2
  item 1).
- open — generation from a stripped save: `settings_toml.md` has generation skip a player
  at -1, so an old-format export migrated against the cup's stripped Fox save gets no
  `settings.toml` for him and his next compile writes all-default rows over the look the
  tables hold. Reading his installed table rows (the bins walk) instead would keep it.
  Settle with Phase 5's generation (review S5.10 item 3).
- open — where the aesthetics patch goes: "beside every CPK a compile publishes" names no
  place in multi-CPK mode (teams parts, the bins CPK, the refs CPK, placeholder parts) or
  in sideload mode (no CPK, yet the savefile step applies "the patch just written"), and
  `patch_written` names one path. The fixed name breaks the pairing in single-CPK mode too:
  two `--no-deploy` runs with different `cpk_name`s keep both CPKs in `output/` but only the
  second patch (review S5.A3 item 1). And the patch is written after the CPK is promoted,
  outside the deployment transaction, which names only the CPKs and the savefile
  (`pipeline.md` "Run-result semantics"), with no finding for a failed patch write, which
  leaves a new CPK beside the old patch (S5.A3 item 4). Settle with Phase 5's patch: one file
  per run or per CPK, its name, its place, and publication with the CPK (review S5.10 item 5).
- open — the Export upgrader's `fpc_on` detection (`export_upgrader.md` step 8, "settings
  match the enable preset"): the cup's FPC players mostly ride their bodies in per-player
  boots IDs, so their boots field holds that ID, not 55, and Test 1's save had every /a/
  player FPC while `is_fpc_player` found none (`.tmp/apptest/results.txt`, run 2). Matching
  the strip fields alone would also mark a dressed player with long sleeves, tucked shirt and
  short socks. Settle the fields compared before Phase 6 (review S5.11 item 2).
- open — strip-and-seed and a base-copied face: the `PlayerAppearance.bin` row carries no
  base-copy id (`model.md`, "The Fox database row"), which holds a face-transplant
  relationship (`operations.md` "Save-to-save operations"), so a stripped player whose
  record copied another player's face may change face, against "stripping changes nobody's
  look". Untested (Test 1 did not cover it). Measure before strip-and-seed is built; then
  carry the relationship or report the affected players (review S5.A3 item 3).
- resolved (2026-10-02) — the VPS mutation half's memory peak reached 7.07 GiB of the 8 GiB cap
  on `team_compiler` (3.y whole-crate run, 2 build jobs); no build was killed, and a killed one
  now fails the run. The maintainer keeps 8 GiB. (The 37 OOM kills of 2026-10-02 04:00 were the
  3.z run under the old 6 GiB cap, the reason it was raised; none since.)

- open — the deep pass reads everything `compile` reads again (4.7): each checked file is
  read once by the pass and once by its task, and a solid `.7z` is decompressed twice (its
  check, then its tasks; three times before 4.7f for an export holding a metadata file).
  For a folder the second read comes from the system's file cache; for a `.7z` it is a whole
  extra decompression per export, about 2 s for a 0.6 to 1 GB export (4.7f's timing): about
  2 s of a 5.2 to 6.8 s compile. The buffer is not kept from the check to the tasks because
  planning needs every export validated first, and holding every `.7z` until then is the
  residency the budget forbids (`libs/pipeline.md` "What a solid `.7z` is charged"). Not
  designed: keeping the buffers that fit the budget and letting go of the rest. Cup exports
  are nearly always compiled from folders, so this waits for the maintainer's word.
- open — a face diff is engine-specific (maintainer, 2026-10-03): how the game uses the diff to
  shape the face skeleton differs between pre-Fox and Fox, so a `face_diff.bin` (or the
  `face_diff.xml` and `<dif>` text forms of it) authored for one engine misplaces the face on
  the other, the same problem the kit layout has. Today the compiler passes a diff through
  for whatever target it compiles and nothing records which engine a diff was made for. To
  investigate, after Phase 4's kits work: what differs (the format, the bones it moves, or
  the rest pose it is relative to), whether one engine's diff converts into the other's
  automatically, and, if it does, how a folder says which engine its diff is for (the kit
  layout's `pre-fox`/`fox` markers are the model). Until then a diff is the author's
  responsibility, as with Red.
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
- open — a texture link beside a combined shared folder's texture of the same stem (4.11d):
  a player holding `hair.dds.common` and a link to a shared face whose folder holds
  `hair.dds` compiles with no finding; the shared folder's copy wins, the link is ignored,
  and `hair` is packed twice (the player's home and the team's Common output).
  `texture_stem_conflict` covers the player folder's own files alone. The plan is silent;
  the likely rule is a conflict finding, since collisions are rejected and never silently
  resolved, and it belongs with the stem namespace in `aesthetics_export` or with the
  textures task's conflicts. Rare: it needs a shared folder combined into a player who
  also links one of its stems to `Common/`.
- open — texture stems are matched as spelled when a model's path is pointed at its texture
  (4.5, seen again at 4.11c and 4.11d), while validation folds case: a model naming
  `hair.dds` beside `Hair.dds` or `Hair.dds.common` keeps a game path with no finding.
  4.29's existence check is where it would surface; decide there whether the match folds
  case.

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
- **2026-09-19** - 2.17e done in four commits (`912237c`, `a43d1ad`, `97102ea`, `4084541` plus
  the review rework): the ingame-face run, `PlayerSettings` with the key table and the TOML
  half, `ops::fpc`; checkpoint (b) review closed. Next: 2.17f `convert.rs` (the 46/50-byte run
  padding rule is its first decision).
- **2026-09-20** - 2.17f done in three commits (`f3ad268` plan, `1001ba9` playstyle lists +
  caps, `bd2ec4c` `convert.rs` plus the review rework): canonical `PlayStyle` with the reference
  editor's twelve arrays as golden data, `face_type_cap`, `convert_player` as a rewrite into a
  target-version template (prefix copy of the ingame-face run, `min(len)` bytes). Checkpoint (b)
  closed, one plan sharpening (what is and is not a `ConvertNote`). Next: 2.17g
  `ops/{transplant,fingerprint,compare}`.
- **2026-09-20** - 2.17g opened: the reference scripts read (transplant = block bytes 4..68,
  compare = fifteen fields plus a masked SHA-256 prefix; the reference editor's comparator =
  roster-slot pairing over the gameplay fields); the run's tail bytes measured zero on every
  16+ fixture player, so the transplant copies the block whole (decision entry). Plan section
  written with the API blocks, two golden tests written, 2.17g split into two slices.
- **2026-09-20** - 2.17g done (`03459eb` plan + goldens, `6226a6e` transplant/fingerprint,
  `74285bd` compare, plus the review rework): the transplant copies the appearance block whole
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
- **2026-09-21** - 2.19 done: gates, deps-check and bindings green locally at `36848c9` and in CI
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
- **2026-09-28** - 2.20i text fix landed (`cd3a1d1`); census clean. Whole-crate run done
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
  missed). Correction: the `cd3a1d1` whole-crate run measured 1130 of 1611 mutants, not all
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
  helpers for drops/strictness/mapped slots. `mutants-diff 8b17f4c` (b1+b2): 221, 18 missed, all
  given tests; rerun 198 caught, 23 unviable, 0 missed. Next: b3.
- **2026-10-01** - 3.6 slice b3 landed: kit grammar, duplicates, `all/` inheritance, kit
  allowlist/names/markers/icon, portraits, logo grammar and roles, root allowlist and notes
  (`kit_config` dependency). Review rework: one stem-conflict helper (b2's two copies included),
  shared icon/marker/name checks, an invalid kit head gets no other finding (decision entry).
  `mutants-diff 3f838a4`: 114, 15 missed (14 given tests, 1 equivalent removed by restructuring);
  rerun 111, 97 caught, 14 unviable, 0 missed. Next: the reviewer on all of (b).
- **2026-10-01** - 3.6 cross-family review, GPT round 1: 7 concerns, 4 accepted (fold panics on
  real names → `vtree::fold_name`; flattening panic → insertion-built tree, no root findings on an
  undecided root; texture `.common` links and `Common/` join the stem check; TC-STR-09 uncited
  until 3.8), 3 rejected; the GPT loop ends (rulings `.tmp/review_rulings_3_6.md`; plan + decision
  entry). `mutants-diff c936e0e`: 31, 2 missed, both given tests; rerun 26 caught, 5 unviable, 0 missed. Next: the sidekick
  review loop.
- **2026-10-01** - 3.6 sidekick review S1: 7 concerns, 3 accepted (a kept disallowed `.common`
  file stays in the player's files; no logo findings on an undecided root; allowlist wording for
  `all/` and `Portraits/`), 4 rejected; the loop ends, and with it the 3.6 review.
  `mutants-diff 78a3190`: 5, 4 caught, 1 unviable. 3.6 done. Next: 3.7, plan section first.
- **2026-10-01** - 3.7 started: `libs/pipeline.md` written (permit, cancellation, oversized
  priority, the solid-7z charge, thread count, `CpkStem`; decision entry), then the crate's
  budget, thread count and `CpkStem` (11 tests, each group red against a stub; the oversized-
  priority test red against a budget without `oversized_waiting`). The memory cap is a share of
  the memory available at run start, read per OS (`windows` feature approved by the maintainer;
  decision entry); its `memory.rs` is not written yet. Not yet run: `mutants-diff`, the
  cross-family review (queued, see "Handover").
- **2026-10-01** - 3.7 implemented: `pipeline::memory_cap` (Windows `GlobalMemoryStatusEx`,
  Linux `/proc/meminfo` through a `mem_available` parse tested on every platform). `mutants-diff
  af9da18`: 12 missed, 7 given tests (exact-cap request is ordinary, OS read and cap above a
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
  Re-run over `af9da18..5618240` (`pipeline` unchanged since): 67, 51 caught, 5 unviable, 6
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
- **2026-10-03** — maintainer rulings recorded in the plans (eight decision entries): format
  findings drop by severity; a missing texture is an Error with the installed CPKs searched
  (4.29); `base64` for `face_diff.xml`; `colors.txt` one color per line; marker names (4.30);
  the Fox referee marker as a reserved collar (4.27 rewritten); one official DpFileList. Step
  4.31 (pre-Fox parity reference) is the lead's. Lead on Fable 5.1, sidekick on Opus 5.5.
- **2026-10-03** — more rulings recorded: collar 77 reserved for the referee marker on both
  engines, `kit_collar_reserved` for a team kit using it (survey: 4,635 kit configs, 77 unused);
  the texture lookup searches only the CPKs before the one compiled; the official DpFileList's
  slot run is still to be authored (open question).
- **2026-10-03** — 4.30: marker names (`fpc_on`, `fpc_off`, `icon_<N>`), the first slice by the
  Opus 5.5 sidekick, landed first time. Also recorded: a `Collars/` file named for a reserved
  collar (105, 77) is `collar_id_conflict`, not `collar_id_invalid` (decision entry); the kit
  layout pair made with PES Master's two kit creators settles the socks and leaves the shorts
  open ("Phase 4 open questions"); `fpc_toggle.md`'s precedence table no longer says a
  default boots/gloves key with no marker preserves the installed ID (it is 0, as
  `settings_toml.md` has said since 4.0).
- **2026-10-03** — 4.5d: a Fox face folder's `face_diff.xml` is decoded into `face_diff.bin`,
  and every face diff is checked (`face_diff_invalid`, `xml_dif_conflict`); 4.5 is closed.
  Maintainer's answers recorded: the far-vertex rule stands (the census's 89 parked placeholder
  models are what it exists for); the slot stem is `teams`, and the official DpFileList is
  drafted in `resources/templates/DpFileList.txt`; the referee marker's models are named and
  it stays on the ground by static painting, so 4.27 waits on nothing. Measured while the
  external drive was connected: every version's stock collar set but PES 20's (decision
  entry). New issue: a face diff is engine-specific.
- **2026-10-03** — A claim checked on the installs at the maintainer's request: collar 105 is
  missing from the stock PES 15 game, but the cup's PES 15 has FPC all the same (its own FPC
  CPK supplies the collar; 301 of 360 kit configs use the four FPC values). `libs/fpc.md` and a
  correcting decision entry say so; the code follows at 4.9. Two open questions added: PES 18's
  FPC values, and whether the games load collars beyond their stock set.
- **2026-10-03** — 4.7a: the deep pass exists end to end. `check` and `compile` read every
  `.fmdl` of the sanitized export, a solid `.7z`'s included, and a vertex over 5000 units out
  drops its folder (`vertex_too_far_from_origin`), with `pass_through` on too; a dropped shared
  folder or `Common/` model takes the players linking it. The export crate re-derives the
  sanitized export from the findings, so no drop or cascade code exists twice.
- **2026-10-03** — 4.7b: `check` and `compile` now report every model check of the two model
  formats and of `.mtl` files, one line per file and problem with a count. An Error (a mesh
  over a hard limit, a face naming a missing vertex) drops the folder unless `pass_through`
  keeps it; a file that is not a model at all is `model_broken` and always dropped. On the
  VGL26 corpus (2,938 `.fmdl`): 89 files with a far vertex (dropped, as decided at 4.7),
  640 with unnormalized weights (Info), 1 with an empty mesh (Warning), no other Error.
  One rework round: counts of item rules (0 became 1), the catalog test's order check, two
  acceptance IDs. Lead fix: `crates/studio`'s CLI test wrote an empty model, now a real
  one. Also this session: the plan for the texture checks (4.7c, split in two) with its
  decision entry, and 4.8's two bin bases bundled with their measured layout.
- **2026-10-03** — PES 2020 is installed (`C:\Data\Games\PES2020`, and on the external
  drive), with the cup's base DLC. Measured on it: the stock collars are 1-127 and 901-913 (the
  plan had assumed 1-124, what PES 19 and 21 share); its FPC CPK supplies the empty
  `collar_105` and `pants_016` like every other version's; it holds no team kit config and no
  color bins (`4cc_08_bins.cpk` is a placeholder).
- **2026-10-03** — Two maintainer rulings. Color bins (4.8): the compiler writes both whole
  on every run and sets every record's header, which repairs the headerless `TeamColor.bin`
  records installed cups carry (decision entry; `pipeline.md` "Bins accumulation";
  TC-BIN-13). Mutation runs: the VPS memory cap cannot be raised; when a build is killed at
  it the run fails and names `REMOTE_BUILD_JOBS` as the setting to lower (`AGENTS.md`,
  `scripts/mutants.py`). Also: PES 21's `download` folder now holds the last VGL's DLC, and
  its kit-config tally in `libs/fpc.md` is redone on it (300 of 314 with the FPC values).
- **2026-10-03** — The repaired bin headers are reported: `bin_header_repaired` (W), once per
  bin, naming the teams (maintainer; decision entry; `messages.md`, TC-BIN-13).
- **2026-10-03** — 4.7c1: the deep pass checks textures from their headers
  (`dds_convert::probe`), so `check` reports `texture_type_mismatch`, `texture_too_small`,
  `texture_not_pow2` and the new `kit_texture_too_big`, each dropping its folder, kit or
  `Common/` file before planning, and `pass_through` keeps the three size findings' files.
  One tests-only rework round (the size rules' one-sided cases, from 7 mutation survivors).
- **2026-10-03** — 4.7c2: `check` reports a portrait's texture findings and drops that file
  alone (`pass_through` keeps a size finding's file); two differing portraits for one slot
  are `portrait_conflict` and skip the export, identical ones compile as one; a logo that
  does not decode is `logo_file_invalid`. `deep.rs` is now the `deep/` module.
- **2026-10-03** — The official DpFileList is fixed by the maintainer
  (`resources/templates/DpFileList.txt`, 53 entries: stadiums 20-35 with the base CPK at 20,
  `teams` 41-45, `teams2` 51-55, midcups 61-75, `4cc_99_test`), and every compile is to compare
  the installed list with it: `dpfilelist_not_official` (W) when it differs and still lists
  the run's targets, the upgrade offered and never forced. The plan and the steps use the new
  names; the code's default `cpk_name` follows at 4.25a. The list file's edit went into commit
  `90e5bfb` with slice 4.7c2 (the lead's `git add -A`), not into a commit of its own.
- **2026-10-03** — The installs' DpFileLists compared with the official one
  (`.tmp/dpfl_compare.py`): 29 to 45 entries each, every listed CPK present in `download/`,
  15 to 24 names per install that the official list retires. So the upgrade also writes a
  placeholder for each official entry with no file (decision entry), and two open questions
  are added: a 53-entry list in the game, and an old DLC under the new list. The plan's
  default `refs_cpk_name` is `4cc_18_referees`. PES 21's `download/` on the external drive
  has no `DpFileList.bin` since the VGL DLC was copied in (the old one is in
  `download_old/`).
- **2026-10-03** — The maintainer's answers on the DpFileList: the upgrade renames an old
  DLC's CPKs by stem (best effort; decision entry, TC-DEP-13); a listed CPK with no file
  makes the game reject the whole download folder and run vanilla, so every compile reports
  one (`dpfilelist_cpk_missing`, TC-DEP-14) and the placeholders are confirmed; a list
  lacking the run's CPK stays `dpfilelist_outdated`; the 53-entry list is still to be
  re-tested in the game (about 60 held, years ago).
- **2026-10-03** — 4.7d: `check` reports a broken face diff, a folder giving its face diff
  in both forms, a kit's `config.toml` that does not parse (`kit_config_invalid`, the kit
  dropped) and a player's `settings.toml` that does not (`settings_toml_invalid`, the file
  alone dropped); none is kept by `pass_through`. The deep pass's checks are complete; step
  4.7's timing is left.
- **2026-10-03** — 4.25a: the default `cpk_name` is `4cc_99_test` in the code and its tests.
- **2026-10-03** — 4.7 timed on FNG (991 MB) and DBG (604 MB): `check` 0.55 and 0.82 s as
  a folder, 2.6 and 2.8 s as a solid `.7z`; `compile` 3.1 and 1.7 s, 7.2 and 5.8 s. The
  serial deep pass is up to 47% of a folder compile, so it goes on the worker pool within
  each export (4.7e, decision entry; the maintainer asked for it), and the `.7z`'s three
  decompressions get a step of their own (4.7f).
- **2026-10-03** — The maintainer's answers: cups are compiled from plain folders, so
  folder and `.zip` exports are checked in parallel with each other (4.7e; only `.7z`
  exports in turn, decision entry); a model-less face always takes the bundled face diff,
  the two other files in the VGL26 exports being outdated FPC-dedicated diffs (decision
  entry; 4.12 settles how the leftover file is reported).
- **2026-10-03** — 4.7e: the deep pass runs on the worker pool (folders, their files and
  `Common/` in parallel; folder and `.zip` exports in parallel with each other, `.7z` ones
  in turn; sources routed in parallel). `check` on a folder export went from 0.55 to
  0.32 s (FNG) and 0.82 to 0.34 s (DBG), both in one run from 1.31 to 0.55 s; no finding or
  order changed.
- **2026-10-03** — The 53-entry DpFileList passed the maintainer's in-game test on PES 2015
  and on PES 2021 (one per engine), so the official list's length is taken to hold on every
  version and the open question is closed (decision entry).
- **2026-10-04** — 4.7f, and step 4.7 done: routing only lists each source, and a source's
  check reads its metadata through the handle its deep pass reads from, so a `.7z` holding a
  metadata file is decompressed once for both passes, not twice. `check` on such an export
  went from 4.39 to 2.36 s (FNG) and 4.23 to 2.31 s (DBG); an export with no metadata file
  measures the same as before. This corrects 4.7's timing note: a decompression costs about
  2 s per export, and the timing exports, which hold no metadata file, were decompressed
  twice per compile, not three times. The tasks' decompression stays ("Issues").
- **2026-10-04** — 4.8a: `check` and `compile` read each kit's and the root `colors.txt` and
  warn about every line that does not give one color (`color_entry_invalid`); nothing is
  written to a bin yet. Step 4.8 is in three slices, and a kit's colors will merge into its
  team's `UniColor.bin` record by kit number, where Red rewrites the whole record (decision
  entry; open with the maintainer: whether a kit a team no longer has should keep its entry).
- **2026-10-04** — 4.8b: every CPK `compile` writes carries a whole `TeamColor.bin`, the
  bundled base with each compiled team's root `colors.txt` colors set; a team export without
  the file reports `team_colors_missing` and keeps its colors. The tracer's bin equals Red's
  byte for byte. Kit colors (`UniColor.bin`) are 4.8c.
- **2026-10-04** — 4.8c, and step 4.8 done: every CPK `compile` writes carries a whole
  `UniColor.bin`, the bundled base with each committed kit's two menu colors and icon merged
  into its team's record. A kit without two valid colors in its `colors.txt` gets them from
  its main texture (`kit_colors_derived`), a kit with no main texture gets magenta and black
  (`kit_colors_missing`). The tracer's bin equals Red's byte for byte. Open with the
  maintainer: kits a team no longer has keep their entries ("Phase 4 open questions").
- **2026-10-04** — 4.9a: an export with an `fpc_on` marker compiles. When any player
  folder carries the marker, every supplied kit config lacking the four FPC values gets
  them (`kit_config_fpc_adjusted`); without it configs are emitted as supplied. `check` and
  `compile` warn about a config value the target version clamps
  (`kit_config_version_clamped`). 4.9b (collars) waits on the maintainer.
- **2026-10-04** — 4.10, lead first: the kit layout table is measured and in the plan. Only
  the socks differ between the layouts; they convert in two bands per sock (pre-Fox u 8–168
  ↔ Fox 8–128, 168–448 ↔ 128–376). The shorts are laid out alike: the "shift" the plan
  quoted was the Fox body standing 33 to 44 mm taller, read as a layout change by a
  nearest-point match (decision entry). Golden: `tests/fixtures/kit_layout/`. Found on the
  way: the stock number atlases are a column of digits pre-Fox and a row in Fox ("Phase 4
  open questions").
- **2026-10-04** — 4.10: a kit marked `pre-fox` compiles for a Fox game with its socks
  re-laid to the Fox layout (`kit_layout_converted`); everything else in the texture stays
  as it was, and a DXT kit keeps the blocks the move does not touch. The other direction
  is built and tested at the function; its CLI test waits for pre-Fox targets (4.16).
- **2026-10-04** — 4.11a: `compile` makes the game's three logo PNGs (512, 256, 128) from
  the export's `logo*` image, the smallest from `logo_small*` when there is one; a
  non-square image is fitted, cropped or stretched by its tag. Step 4.11 is split into four
  slices; the plan text and decisions for the next two (notes, kit variants) are in.
  TC-CMN-06 moved to 4.29.
- **2026-10-04** — 4.11b: `compile` gathers the compiled exports' `notes.txt` into
  `output/teamnotes.txt`, one `--- /co/ ---` section per team, once the CPK is in place; a
  compile with no notes removes an older file. TC-TEX-09 (a texture `.common` link) is
  written for 4.11d.
- **2026-10-04** — 4.11c: kit variants on Fox. A texture set (`pants_kit1`, `pants_kit3`)
  gets a copy of its lowest variant for each kit number the team has and the set lacks
  (`kit_variant_missing`); a model's `pants_kitN` path is pointed at where the variants
  go; of per-kit model files only the lowest is compiled (`kit_variant_model_fox`).
  TC-CMN-05 split: its PES 17 half is TC-CMN-07, with 4.14.
- **2026-10-04** — 4.11d: a texture `.common` link (`hair.dds.common`) compiles: the
  player's model names the texture in the team's Common output, where it is packed once.
  Step 4.11 is done. Two open issues recorded: a link beside a combined shared folder's
  texture of the same stem, and stems matched as spelled.
- **2026-10-04** — 4.12: `ingame_face` compiles on Fox (no face folder, the models the face
  would take become the player's boots), every other player folder gets a face folder, blank
  without a face model, and a face file with no face model is `face_file_not_used`.
  TC-MOD-09 reworded: a failed package is left out alone.
- **2026-10-04** — 4.13a: two exports of one team are each skipped with
  `duplicate_aesthetics_export`, by `check` and `compile`, and the other teams compile;
  the CPK is proven independent of the worker count and of skipped exports. 4.13 goes on
  as 4.13b (`overrides/`) and 4.13c (source pinning).
- **2026-10-04** — 4.13b: the files of the data directory's `overrides/` folder go into the
  CPK first and win over an export's entry or a bin at their path (`overrides_active`,
  `duplicate_path`); they are written even when no export compiles.
- **2026-10-04** — 4.13c: an export file that changes while `compile` runs (saved over,
  removed, the archive replaced) aborts the run with `source_changed_during_run`; the
  previous CPK stays. Step 4.13 is done.
- **2026-10-04** — the maintainer answered five Phase 4 open questions: override paths compare
  exactly; a failed package is left out alone as long as the Error is visible (help sentence
  added); number atlases are re-arranged across engines (new step 4.32); one kit config
  template for every version (the census found the template's bytes already the common
  ones, the clamp coming from how they are read); a full export resets a team's kit
  record and a midcup one does not (`Full`/`Midcup` in the export's name, under discussion).
- **2026-10-04** — settled with the maintainer: the kit config template stays and `name.y`
  is PES 21's value on every version (step 4.33); a team export's name carries `Full` or
  `Midcup` as its second word (step 4.34, plan and eight scenarios written: TC-ID-05 to 07,
  TC-KIT-24, TC-BIN-14 to 17; four reworded).
- **2026-10-04** — Review queue, S1 `pipeline` (3.7 (b), 3.y (c)): sidekick S1.1 1 of 5
  accepted; GPT A1 3 of 3 (a finished player group's permit held by the coordinator hung the
  run when the next acquire needed it; the writer released a permit before its bytes;
  `CpkStem`'s rule order untested). Rulings `.tmp/review_rulings_S1.md`.
- **2026-10-04** — Review queue, S2 `studio_core` + `studio` (3.8 (b), 3.z (b), 3.y (c)):
  sidekick S2.1 0 of 4 (one recorded under "Issues" for Phase 8); GPT A1 4 of 6 (an unwritable
  data folder stopped `compile` at the teams list; a TOML error's snippet broke the one-line
  console format; shared temp folders in settings tests; a `pub` with no consumer). Rulings
  `.tmp/review_rulings_S2.md`.
- **2026-10-04** — Review queue, S3 `aesthetics_export` (3.y (c), with its Phase 4 parts):
  sidekick S3.1 1 of 9 (an empty doubled layer folder kept as a phantom folder); GPT A1 4 of
  4 (two spellings of one Common model link panicked a compile; skeletons paired case-exactly
  where planning folds; roster findings on an undecided root; a flattened wrapper of empty
  folders kept); sidekick S3.2 2 of 3 (face file names and texture stems matched
  case-exactly). Rulings `.tmp/review_rulings_S3.md`.
- **2026-10-05** — Review queue, S4 the Team compiler's Phase 3 skeleton (3.8 (b) CLI half,
  3.9f (a), 3.y (c) team_compiler): sidekick S4.1 1 of 4 (the command-line root's refusal said
  no action); GPT A1 5 of 5 (a failed CPK write kept the whole cup processing; a teams list
  written straight onto its path left a truncated file later runs read; `--export ..` named
  `..`; the parity test normalized relocated references unchecked; TC-OUT-06's text stale);
  sidekick S4.2 0; GPT A2 1 of 1 (a `.7z` kept running after the cancel); sidekick S4.3 1 of 1
  (the cancel checked only between sources). From S4.3 the sidekick's role is a `swe-2-high`
  subagent (AGENTS.md "Environment"). Rulings `.tmp/review_rulings_S4.md`.
- **2026-10-05** — Review queue, S5 the step 4.0 plan rewrite (docs only): sidekick loop S5.1
  to S5.8 (21, 9, 9, 5, 7, 5, 8, 3 accepted); GPT A1 14 of 14 (the Fox rows' stage in the
  diagram, pre-Fox face-XML IDs through the settings order, the patch written per published
  CPK, the unknown-bits example on decoded bits, model.md's copy of the precedence rule);
  sidekick S5.9 4; GPT A2 6 of 6 (generation from a stripped record, the full-length shirt
  name, the face-editor recipe's recompile); sidekick S5.10 to S5.13 (10, 5, 6, 2: one rule
  for every reader of a stripped record, the presets' fields listed only in `libs/fpc.md`);
  GPT A3 4 of 4 (all worklog issues); sidekick S5.14 0 of 2. Thirteen open "Issues" came out
  of it, five for the maintainer. Rulings `.tmp/review_rulings_S5.md`.
