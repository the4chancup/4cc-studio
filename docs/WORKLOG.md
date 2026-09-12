# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 1 (Workspace bootstrap + core skeleton), steps 1.1–1.5 done; 1.6 converge next.
**In progress:** 1.6 converge
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

- `just` needs PowerShell as its Windows shell (`set windows-shell` in the justfile): Git for
  Windows' `sh` is not on PATH from a plain PowerShell. Recipes still stop at the first failure.
- `cargo install just cargo-deny` is required once per machine (both take a few minutes to build);
  the pinned toolchain and the wasm32 target install themselves on the first `cargo` call.
- The lockfile holds egui 0.36.1 on purpose (0.36.2 was under a week old at bootstrap); a bare
  `cargo update` would move it. Bump deliberately.

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
| 1 | Workspace bootstrap + core skeleton | workspace, CI, non-GUI `studio_core`, `vtree`, `pes_version` | in progress |
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

- [x] 1.1 Workspace bootstrap — done: `Cargo.toml` (workspace deps, lints, dev profile),
  `rust-toolchain.toml` (1.98.0 + wasm32), `rustfmt.toml`, `justfile` (`gates`, `deps-check`;
  Windows shell is PowerShell, `sh` was not on PATH), `deny.toml`, `scripts/wasm_check.py`,
  `scripts/deps_check.py`, `crates/{studio,studio_core,libs,tools}`. Verified: `just gates` green
  from PowerShell; a planted clippy warning made it exit 1 (reverted); `just deps-check` green
  after the font-license decision
- [x] 1.2 CI — done: `.github/workflows/ci.yml` (gates on ubuntu + windows, deps-check on
  ubuntu). The `python_bindings` job is deferred to step 2.18, when the crate exists. Not yet
  verified: no CI run has happened (first push after this commit); the deliberately red run is
  still owed → verify: one CI run green on the bootstrap commit; one deliberately red from a
  pushed clippy warning, then reverted
- [x] 1.3 `vtree` — done (sidekick, one brief, no rework): `ScopePath`, `RelativeScopePath`,
  `PathError`, `VirtualTree<T>`, `InsertError`, `Entry`; 13 tests covering the listed cases;
  wasm32 check green. `crates/libs/vtree/src/lib.rs`
- [x] 1.4 `studio_core` non-GUI parts — done (lead): `tool.rs` (`StudioTool`, `ToolContext`,
  `ShellRequest`), `events.rs`, `status.rs`, `help/mod.rs` (types only), `settings/` (framework +
  `CommonSettings`), `shell/launch.rs` (three launch modes, CLI dispatch); `pes_version` leaf
  crate; `studio` stub binary (CLI path only). 14 + 3 tests: settings round-trip, defaults,
  recursive merge, stub tool dispatched from `studio stub ping x`; wasm32 check green
- [x] 1.5 Phase verification — done: `just gates` green (30 tests across `pes_version`,
  `studio_core`, `vtree`), `just deps-check` green
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
- [ ] 2.18 `python_bindings` (maturin build + Python smoke test; add the `just bindings` recipe
  and the CI job deferred from step 1.2)
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
- **2026-09-13** - Phase 1 steps 1.1–1.5: workspace, gates, CI file, `vtree` (sidekick),
  `pes_version` and non-GUI `studio_core` (lead), 30 tests, all gates green locally. Two plan
  gaps decided by the user (`PesVersion` home, `unicode-normalization`); font licenses allowed.
  Parallel lead/sidekick work in one tree tripped once on a half-declared module; rule added to
  `AGENTS.md`. Next: 1.6 converge (reviewer first), 1.7 rewrite, first push and CI run.
