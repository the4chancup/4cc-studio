# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 2 (Library crates). Done: 2.1 `wezlib`, 2.2 `cpk`, 2.3 `fpk`, 2.4 `ftex`, 2.5 `dds_convert` (CPU),
2.8 `uniparam`, 2.11 `fpc`, 2.12 `teams_list`, 2.13 `kit_config`. Next: 2.6c-3 `fmdl::ops`, one op per handoff. Review
rounds A and B (2026-09-13) closed: 2.5c, 2.12b, 2.13b done.
**In progress:** none
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
- `dds_convert`'s CPU BC1 core (`block_compression`, PCA plus one refinement) trails DirectXTex
  by about 2.5 mean error on an 8x4 grey-ramp mip while matching it within 1.0 at the top mip;
  the encode tests compare the chain-wide mean for that reason. Revisit with representative kit
  and face textures when the Team compiler can produce them (plan: "tune against representative
  textures").
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
below to this one row; the step-level detail stays in git history. Tool phases (3–6, 8–15) also
open with an **acceptance** step: the tool plan's "Acceptance" section for that phase, written
before any code (GUI scenarios that no automated test can prove are marked `manual` and proven by
a recorded check at converge — `CONTRIBUTING.md` "Testing"). Procedure: `AGENTS.md` "Working
documents".

| Phase | Scope | Crates | Status |
|---|---|---|---|
| 1 | Workspace bootstrap + core skeleton | workspace, CI, non-GUI `studio_core`, `vtree`, `pes_version` | done |
| 2 | Library crates (standalone-verifiable) | `wezlib` `cpk` `fpk` `ftex` `dds_convert` `fmdl` `pes_model` `uniparam` `fox2` `archives` `fpc` `teams_list` `kit_config` `color_tools` `elevation` `model_convert` (native) `pes_savefile` `python_bindings` | todo |
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

Done 2026-09-13 (spec now describes what exists: `docs/plans/core.md` "Phase 1"). Step detail
in git history up to commit `794ce61`. CI proof: green run on `a4be936`, deliberately red run on
`38c3e68` (both `gates` jobs failed at `just gates`, `deps-check` unaffected), reverted in
`794ce61`.

### Phase 2 — Library crates

Spec: `docs/plans/core.md` "Phase 2", `docs/plans/libs.md`, `model_conversion.md`,
`pes_savefile.md`. Leaf crates first, dependents after; `python_bindings` last.

- [x] 2.1 `wezlib` — done (sidekick): WESYS wrap/unwrap over flate2; fixture `RefereeColor.bin`
  (PES17); 4 tests. Parity for WESYS files is payload-level (deflate bytes differ from Python's)
- [x] 2.2 `cpk` — done (sidekick): @UTF read/write, CRILAYLA decoder, `CpkArchive`, `CpkWriter`
  byte-identical to pes-file-tools on a Red-written face CPK; Konami PES17/PES21 and CPKMC 1.36
  fixtures; 13 tests. Fixtures are `-text` in `.gitattributes` (a CRLF-bearing fixture was
  normalized by the first commit and caught)
- [x] 2.3 `fpk` — done (sidekick): read/write, MD5 names; writer byte-identical to the reference
  writer on synthetic goldens and the empty template, and Konami files rewrite identically; 10 tests
- [x] 2.4 `ftex` — done (sidekick): `ftex_to_dds` byte-identical on six Konami fixtures (BC1, BC3,
  chunked BC7, A8R8G8B8, cube map, 1x1); `dds_to_ftex` round-trips; 6 tests
- [x] 2.5 `dds_convert` — CPU path — done (sidekick, lead finished the tests): `decode` exact
  against texconv's decodes on all 7 fixtures and every mip (BC3/BC4/BC5 alpha ramps re-derived
  with rounding, `block_compression` truncates); passthrough BC7/BC3/BC5 byte-identical; encode
  judged per mip against the reference encoder's own error on the same image (a fixed tolerance
  was wrong: the small mips are pathological for any BC1 line); DXT5nm per engine; cache. `ftex`
  gained `pub mod dds` (`read_layout`, `header_bytes`) and a fix for a zlib chunk that is exactly
  its piece's size (read as raw by every reader). 10 + 7 tests
- [ ] 2.5b `dds_convert` GPU BC7 (wgpu backend of `block_compression`): Vulkan/Metal device, CPU
  fallback, cold-start and throughput measured → verify: GPU and CPU outputs decode within the
  same tolerance; fallback path exercised by forcing no adapter
- [x] 2.6a-1 `fmdl::format` container (raw records per block, raw section-1 blocks) and SKL codec
  — done (sidekick): byte-identical on the three Konami FMDLs and three SKLs; add-on FMDLs
  round-trip semantically (`oral` pads blocks to 16, ours does not); unknown block ids survive as
  one span. Quirks kept: section-1 block 3 always reads to file end; a section-1 length past the
  file end is clamped. The denied-dependency check was exercised earlier (`egui` planted on
  `fmdl` turned `deps_check.py` red, reverted). 8 tests
- [x] 2.6a-2 `fmdl::format` typed records — done (sidekick): `FmdlFile` over the container, 19
  record structs with every byte a named field, byte-identical on the Konami fixtures, bone names
  of all four boned fixtures resolve to the reference parser's lists. 14 tests
- [x] 2.6b `fmdl::format` vertex and face codec — done (sidekick): attributes resolved through
  mesh-format assignments and buffer offsets, decode/encode in place, hand-written half floats
  (exhaustive round trip); decode then re-encode of every mesh of every fixture leaves the buffer
  byte-identical; every highneck weight quad sums to 255. 19 tests
- [x] 2.6c-1 `fmdl::model` `Model::from_file` — done (sidekick): every fixture loads; bones,
  materials (shader, technique, textures, parameters), groups, meshes and bone groups equal the
  reference parser's output as literal expectations; extension headers parsed (`Extensions.other`
  keeps unknown flags). Open: `Custom-Bounding-Box-Meshes` has nothing to populate (no per-mesh
  box in the format); revisit at `to_file`. 29 tests
- [x] 2.6c-2 `fmdl::model` `Model::to_file` — done (sidekick): the add-on writer's layout with
  Konami's missing group boxes computed or omitted; `from_file(to_file(m)) == m` on all five
  fixtures, Konami ones included; `model/` split into `mod`/`from_file`/`to_file`/`tests`. 34 tests.
  Open (plan gaps, revisit at converge): `Custom-Bounding-Box-Meshes` and any unknown per-object
  extension header are parsed but have no `Model` field, so a rewrite drops them
- [ ] 2.6c-3 `fmdl::ops` antiblur, split, vertex_enc, merge, paths per `model_conversion.md`
  "Extension algorithms" and "multi-FMDL mesh merging", one handoff each
- [ ] 2.6d `fmdl::check` findings
- [ ] 2.7 `pes_model` (`format/` + `ops/` + `check.rs`)
- [x] 2.8 `uniparam` — done (sidekick): WESYS-unwrapping read, sorted writer; Konami PES21
  container (2174 entries) and a reference-writer golden; 4 tests
- [ ] 2.9 `fox2`
- [ ] 2.10 `archives`
- [x] 2.11 `fpc` — done (sidekick): kit values per version, three presets, five interference
  findings, each citing its `FPC.wikitext` line; 4 tests
- [x] 2.12 `teams_list` — done (sidekick): fold, id range, parse/write byte-identical on the
  shipped list (`data/teams_list.txt`, `/umaJP/` case-folded, `Backup N` inert), reconcile with
  all four summary buckets; 10 tests
- [x] 2.13 `kit_config` — done (sidekick, two rework rounds on validation): bit-identical on all
  1372 PES 2021 stock configs, TOML form with comments, texture names, FPC apply/matches; the
  plan's ranges and 144-only sleeve rule were the old editor's UI limits (plan corrected); 8 tests
- [x] 2.5c `dds_convert` review fixes — done (sidekick): `Decoded.authored_mips`, normal role
  passes through only BC3 (and BC5 on 19-21), declared row pitch honoured with the DWORD rule on
  lower mips, BC1 output and every generated mip proved by decode against the reference; `ftex`
  refuses DX10 arrays, the DX10 cube flag and signed BC4/BC5, reads B8G8R8X8 as opaque. 13 + 8
  tests (review A, all seven accepted)
- [x] 2.12b `teams_list` review fixes — done (sidekick): rows keep every cell, `ID`/`Name`
  located anywhere in the header; placeholder ids (all 95 in the shipped list are numeric) join
  the duplicate check and an incoming team takes a placeholder's slot; reconcile applies every
  incoming change then reverts the ones whose final id collides. 14 tests (review B, findings
  1-3). Converge note: the revert loop in `reconcile.rs` is heavier than the plan's sentence;
  candidate for simplification at 2.20
- [x] 2.13b `kit_config` review fixes — done (sidekick): one `field_limits` table drives both
  `kit_value_out_of_range` (with field/value/max context) and the emission clamps (Name Y 16 on
  PES <= 20, 39 on 21); `kit_pattern_unsupported_pes15`; wrong-typed TOML tables are errors;
  non-ASCII hex is an error, not a panic. 11 tests, the 1372-config mass test unchanged (review B,
  findings 4-7)
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
- [ ] 2.20 Converge: own audit then reviewer subagent, each crate against its plan section
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
