# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** none started — planning complete, implementation not begun.
**In progress:** —
**Blocked on:** —

---

## How to resume

| Thing | Where |
|---|---|
| Plan index | `docs/plans/README.md` → `docs/plans/core.md` |
| Development phases | `docs/plans/core.md` "Development Plan" |
| Legacy tools (format evidence) | `docs/plans/core.md` "Project context" (paths are per-machine) |
| Skeleton data | `resources/skeletons/` (see its README) |
| FPC source text (for the `fpc` crate) | `resources/FPC.wikitext` |
| Coding rules and verification gates | `docs/CONTRIBUTING.md` |
| Domain terms | `docs/GLOSSARY.md` |

### Current-state gotchas

Things that are true *right now* and would cost the next session time to rediscover (a broken
fixture, a flaky test, a crate that doesn't build on one platform). Remove an entry when it stops
being true. Permanent toolchain facts go in `docs/CONTRIBUTING.md` "Toolchain notes" instead.

- (none yet)

---

## Phases

Status: `todo` · `in progress` · `done` · `blocked`. A phase is done only after its last two
steps, which every phase has: **converge** (cross-family reviewer audit first, then the agent's own,
against the phase's plan sections and acceptance IDs; each gap becomes a new step above it, and the
phase waits for them) and **rewrite**
(the phase's plan sections rewritten in the present tense, in place). Then collapse its step list
below to this one row; the step-level detail stays in git history. Tool phases (3–6, 8–15) also
open with an **acceptance** step: the tool plan's "Acceptance" section for that phase, written
before any code (GUI scenarios that no automated test can prove are marked `manual` and proven by
a recorded check at converge — `CONTRIBUTING.md` "Testing"). Procedure: `AGENTS.md` "Working
documents".

| Phase | Scope | Crates | Status |
|---|---|---|---|
| 1 | Workspace bootstrap + core skeleton | workspace, CI, non-GUI `studio_core`, `vtree` | todo |
| 2 | Library crates (standalone-verifiable) | `weszlib` `cpk` `fpk` `ftex` `dds_convert` `fmdl` `pes_model` `uniparam` `fox2` `archives` `fpc` `teams_list` `kit_config` `color_tools` `elevation` `model_convert` (native) `pes_savefile` `python_bindings` | todo |
| 3 | Team compiler skeleton | `team_compiler`, `aesthetics_export`, `pipeline` | todo |
| 4 | Processing logic | `team_compiler` (`plan/` `processing/` `bins/` `output/`), `aesthetics_export` deep validation | todo |
| 5 | Savefile integration | `save_editor` logic, `aatf`, `team_compiler` `output/savefile.rs` | todo |
| 6 | Export upgrader | `export_upgrader` | todo |
| 7 | glTF support | `model_convert` (glTF half) | todo |
| 8 | GUI | `studio_core` shell, `studio`, tool views, `color_tools` widget | todo |
| 9 | Stadium compiler | `stadium_compiler` | todo |
| 10 | Music tools | `music_player`, `music_export_editor`, `music_export`, `audio_engine` | todo |
| 11 | Match tracker | `match_tracker`, `match_feed` | todo |
| 12 | Kit config editor | `kit_config_editor` | todo |
| 13 | Refs arranger | `refs_arranger` | todo |
| 14 | Balls compiler | `balls_compiler` | todo |
| 15 | Player aesthetics editor | `player_aesthetics_editor` | todo |
| 16 | Polish and distribution | — | todo |
| 17 | Studio Web (post-release) | — | todo |

---

## Steps

`[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked / needs a decision

A step is done when its own `→ verify:` check has actually been run and passed, on top of the
gates. Every step carries one when itemized; if a step is listed without one, the agent writes it
before starting the step (a specific check, not "test it") and puts it in the step text. Mark a
done step with a one-line summary and the files or crates touched. One `[~]` per agent at a time.

### Phase 1 — Workspace bootstrap + core skeleton

Spec: `docs/plans/core.md` "Phase 1".

- [ ] 1.1 Workspace bootstrap — git repo, virtual workspace with `[workspace.dependencies]` and
  `[workspace.lints.clippy]` (incl. `let_underscore_must_use = "deny"`), `rustfmt.toml`,
  `rust-toolchain.toml` (exact version + `wasm32-unknown-unknown`), crate directories, root
  `justfile` with the `gates` recipe (lead-authored: a wrong gate list passes instead of failing)
  → verify: `just gates` passes on the empty workspace from PowerShell and confirms which shell
  `just` picked up (`sh` from Git for Windows, else set `windows-shell`); `rustup show` in the
  repo reports the pinned version and the `wasm32` target
- [ ] 1.2 CI — `just gates`, `just deps-check` (guardrail 4), `python_bindings` build job
  → verify: one CI run green on the bootstrap commit; one deliberately red from a pushed clippy
  warning (proves the gate bites), then reverted
- [ ] 1.3 `vtree` → verify: unit tests for `ScopePath`/`RelativeScopePath` normalization,
  traversal/absolute/empty rejection, case-fold and Unicode collision detection, join-to-root;
  `wasm32` check green
- [ ] 1.4 `studio_core` non-GUI parts: `StudioTool` trait + `ToolContext`, settings framework,
  `PipelineEvent` types → verify: settings load/merge-defaults/save round-trip tests; a stub tool
  registered through the trait and dispatched from a CLI argument; `wasm32` check green
- [ ] 1.5 Phase verification: gates green across the workspace; every step's own check re-run
- [ ] 1.6 Converge: reviewer subagent then own audit against `core.md` "Phase 1", "Tool plugin
  interface", "Workspace guardrails", and the parts of "Event system" Phase 1 delivers (the type
  definitions — the section's own notes defer status-strip metrics and structured outcomes to
  later phases; those are not gaps); gaps become steps. Record as a log line: the reviewer's
  wall-clock time and whether its concerns found anything the gates had not; and for the sidekick,
  how many of the phase's briefs needed a rework round, and which steps the lead took over and why
- [ ] 1.7 Rewrite those `core.md` sections in the present tense

### Phase 2 — Library crates

Spec: `docs/plans/core.md` "Phase 2", `docs/plans/libs.md`, `model_conversion.md`,
`pes_savefile.md`. Leaf crates first, dependents after; `python_bindings` last.

- [ ] 2.1 `weszlib`
- [ ] 2.2 `cpk` (read + write, CRILAYLA inside, roundtrip test)
- [ ] 2.3 `fpk`
- [ ] 2.4 `ftex`
- [ ] 2.5 `dds_convert` (CPU reference first; GPU BC7 proof per plan)
- [ ] 2.6 `fmdl` (`format/` + `ops/` + `check.rs`)
- [ ] 2.7 `pes_model` (`format/` + `ops/` + `check.rs`)
- [ ] 2.8 `uniparam`
- [ ] 2.9 `fox2`
- [ ] 2.10 `archives`
- [ ] 2.11 `fpc` (leaf; before `kit_config` and `pes_savefile`)
- [ ] 2.12 `teams_list` (leaf; `TeamName` fold, `TeamId`, parse/write/reconcile) → verify:
  Red's current `teams_list.txt` loads as a fixture with `/umaJP/` resolving case-folded and the
  `Backup N` rows inert; reconcile tests cover added / kept / overridden / unresolved
- [ ] 2.13 `kit_config`
- [ ] 2.14 `color_tools` (extraction only)
- [ ] 2.15 `elevation`
- [ ] 2.16 `model_convert` — IR, native importers/exporters, hand auto-split, skeleton constants,
  material conversion (glTF is Phase 7)
- [ ] 2.17 `pes_savefile` — crypto, schema codec, model, `EditFile`, `PlayerSettings`, conversion,
  interchange formats, transplant/fingerprint, comparator, FPC invisibility
- [ ] 2.18 `python_bindings` (maturin build + Python smoke test)
- [ ] 2.19 Phase verification: every crate's tests per `libs.md` "Testing" green; `wasm32` check
  green on every lib
- [ ] 2.20 Converge: reviewer subagent then own audit of each crate against its plan section
  (`libs.md`, `model_conversion.md`, `pes_savefile.md`, `core.md` "Phase 2"); gaps become steps
- [ ] 2.21 Rewrite those sections in the present tense

### Phase 3 — Team compiler skeleton

Spec: `docs/plans/core.md` "Phase 3", `docs/plans/team_compiler.md`, `docs/plans/aesthetics_export.md`.
Steps are itemized when Phase 2 closes; the first is fixed:

- [ ] 3.1 Acceptance: write `team_compiler.md` "Acceptance" for the Phase 3 scope (format:
  `CONTRIBUTING.md` "Testing")
- [ ] 3.2 Converge check script: extract every acceptance ID from the plans' "Acceptance" sections
  (skipping `withdrawn:` ones) and every `// XX-YYY-NN` citation in any `.rs` file under `crates/`
  (inline `#[cfg(test)]` modules included, not just `tests/`); report orphan citations (an ID no
  scenario defines) and unproven scenarios. Two modes: **report** — what CI runs on every change;
  it fails only on orphan citations, since scenarios are written before the code that proves them
  and unproven IDs are the normal state of an open phase; **strict** — run at converge for the
  closing phase's IDs; unproven or `manual` scenarios without a recorded check fail it. Python or a
  tiny Rust bin — decide when written (needs a decision entry either way, since it adds a gate)

---

## Issues

Bugs, unexpected behavior, things to revisit. `open` / `resolved (date)`. Resolved issues are
pruned when their phase closes; they stay in git history.

- (none yet)

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
  `TeamName` crate implemented cold from `CONTRIBUTING.md` + `team_compiler.md` §"Export display
  name and team name" in 3.5 min wall-clock, red-first, all gates green, style rules followed
  literally, two real plan gaps reported rather than decided (leading separators; Unicode vs.
  ASCII lowercasing — the latter is a genuine open point for `teams_list.tsv` matching, carried to
  Phase 3). Second-opinion checkpoints redrawn for three roles (`AGENTS.md`). Phase 1 starts in
  Fusion from step 1.1, with the lead hand-writing the gate infrastructure only.
- **2026-09-12** - Plans for the Refs arranger (Fox hook lists), aesthetics patch, Team creator
  (+ `libs/team_widgets`) written; license settled (`MIT OR Apache-2.0`, Konami data excluded).
  Repository started: first commit replaces the 2023 placeholder history on `the4chancup/4cc-studio`.
