# Log

What happened, dated, newest last, three lines at most per entry: what changed and what it
means for the next session. Append-only. No rationale (→ plan), no decisions (→
`DECISIONS.md`), no status (→ `WORKLOG.md`, which holds where we are; this file holds how
we got there). Until 2026-10-11 this was the worklog's last section, moved out when the
worklog passed 7,000 lines.

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
  `STUDIO_13 caught, 0 missed, 3 unviable; the remote half peaked at the 9G cap with no build killed_REMOTE` is set (`scripts/mutants.py`; `AGENTS.md` "Environment" has the
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
  `.tmp/4_0/apptest/out/`, awaiting the maintainer's in-game run.
- **2026-09-28** - Hard gate set (maintainer): step 4.0, the appearance-fallback test, stands at
  the end of Phase 3; no agent goes past it until the maintainer reports the result. Phase 2's
  remaining converge, 2.17i and Phase 3 proceed. Next: 2.20g `pes_model`.
- **2026-09-28** - 2.20g `pes_model` done (one reviewer round, 4/4 accepted). Method finding:
  the Konami-measured reader refused about 1900 of the machine's 6037 community `.model`
  files, cup exports among them, and no fixture, mutation run or reviewer would have found
  it; a census of every real file of the format now belongs in each format crate's converge
  (`.tmp/2_20/model_census/` as the template). Next: 2.20g-fmdl census, then 2.20h.
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
  step-by-step guide is `.tmp/4_0/apptest/out/GUIDE.txt`. Next: the 2.20i text fix.
- **2026-09-28** - 2.20i text fix landed (`cd3a1d1`); census clean. Whole-crate run done
  (24 survivors, `.tmp/2_20/mutants_2_20i_whole/missed.txt`), the crate's last (maintainer). Lead
  audit in `.tmp/2_20/audit_2_20i.md` (F1-F7). Next: slice A = survivors + F1-F4, F6, F7; slice B =
  F5's pure moves; each checked with `mutants-diff`; then the reviewer.
- **2026-09-28** - `just mutants` fixed: the remote half runs detached and survives a dropped
  link or a killed controller (`just mutants-collect`), and shells without
  `STUDIO_13 caught, 0 missed, 3 unviable; the remote half peaked at the 9G cap with no build killed_REMOTE` read it from the registry.
- **2026-09-28** - Maintainer check of the full-field shirt name (`EDIT00000000 prespoon`,
  PES 16): 4ccEditor shows `MEAT ON THE BON`, and the save crashes PES 16 after the start
  screen, cause unknown (the maintainer doubts the shirt name). `shirt_name_from` keeps its
  free byte. Next: 2.20i slice A (`.tmp/2_20/brief_2_20i_a.md`).
- **2026-09-29** - 2.20i slice A done (survivor tests, F1-F4, F6, F7; `mutants-diff`: 0
  missed). Correction: the `cd3a1d1` whole-crate run measured 1130 of 1611 mutants, not all
  of them (its VPS half died partway); the maintainer approved one run of the 486 unmeasured.
- **2026-09-29** - 2.20i remainder run: 502 mutants, 4 missed (3 equivalent as written,
  rewritten; 1 test added), `mutants-diff`: 0 missed. Next: slice B.
- **2026-09-29** - 2.20i slice B done: pure moves split `team_toml/team.rs` (tactics, items)
  and `settings_toml/mod.rs` (document), checked by `.tmp/2_20/pure_move_check.py`. Next: the
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
  the code (`.tmp/2_21/audit_2_21_as_built.md`) fixed stale crate trees, deps rows, two code blocks
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
  until 3.8), 3 rejected; the GPT loop ends (rulings `.tmp/3_6/review_rulings_3_6.md`; plan + decision
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
  (`.tmp/4_7/dpfl_compare.py`): 29 to 45 entries each, every listed CPK present in `download/`,
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
  `CpkStem`'s rule order untested). Rulings `.tmp/review_queue_S/review_rulings_S1.md`.
- **2026-10-04** — Review queue, S2 `studio_core` + `studio` (3.8 (b), 3.z (b), 3.y (c)):
  sidekick S2.1 0 of 4 (one recorded under "Issues" for Phase 8); GPT A1 4 of 6 (an unwritable
  data folder stopped `compile` at the teams list; a TOML error's snippet broke the one-line
  console format; shared temp folders in settings tests; a `pub` with no consumer). Rulings
  `.tmp/review_queue_S/review_rulings_S2.md`.
- **2026-10-04** — Review queue, S3 `aesthetics_export` (3.y (c), with its Phase 4 parts):
  sidekick S3.1 1 of 9 (an empty doubled layer folder kept as a phantom folder); GPT A1 4 of
  4 (two spellings of one Common model link panicked a compile; skeletons paired case-exactly
  where planning folds; roster findings on an undecided root; a flattened wrapper of empty
  folders kept); sidekick S3.2 2 of 3 (face file names and texture stems matched
  case-exactly). Rulings `.tmp/review_queue_S/review_rulings_S3.md`.
- **2026-10-05** — Review queue, S4 the Team compiler's Phase 3 skeleton (3.8 (b) CLI half,
  3.9f (a), 3.y (c) team_compiler): sidekick S4.1 1 of 4 (the command-line root's refusal said
  no action); GPT A1 5 of 5 (a failed CPK write kept the whole cup processing; a teams list
  written straight onto its path left a truncated file later runs read; `--export ..` named
  `..`; the parity test normalized relocated references unchecked; TC-OUT-06's text stale);
  sidekick S4.2 0; GPT A2 1 of 1 (a `.7z` kept running after the cancel); sidekick S4.3 1 of 1
  (the cancel checked only between sources). From S4.3 the sidekick's role is a `swe-2-high`
  subagent (AGENTS.md "Environment"). Rulings `.tmp/review_queue_S/review_rulings_S4.md`.
- **2026-10-05** — Review queue, S5 the step 4.0 plan rewrite (docs only): sidekick loop S5.1
  to S5.8 (21, 9, 9, 5, 7, 5, 8, 3 accepted); GPT A1 14 of 14 (the Fox rows' stage in the
  diagram, pre-Fox face-XML IDs through the settings order, the patch written per published
  CPK, the unknown-bits example on decoded bits, model.md's copy of the precedence rule);
  sidekick S5.9 4; GPT A2 6 of 6 (generation from a stripped record, the full-length shirt
  name, the face-editor recipe's recompile); sidekick S5.10 to S5.13 (10, 5, 6, 2: one rule
  for every reader of a stripped record, the presets' fields listed only in `libs/fpc.md`);
  GPT A3 4 of 4 (all worklog issues); sidekick S5.14 0 of 2. Thirteen open "Issues" came out
  of it, five for the maintainer. Rulings `.tmp/review_queue_S/review_rulings_S5.md`.
- **2026-10-05** — Review queue, S6 the Team compiler's Phase 4 Acceptance section (docs, plus
  test citations and two test fixes): sidekick loop S6.1 to S6.3 (30, 7, 4 accepted); GPT A1
  22 of 24 (unreachable scenarios withdrawn or rewritten, four new); sidekick S6.4, S6.5 (5,
  1); GPT A2 16 of 16 (pre-Fox scenarios: shared gloves `glove.xml`, Common MTL links, referee
  faces; TC-TEX-12 proven); sidekick S6.6 to S6.12 (14, 10, 9, 6, 8, 11, 4: WHENs and GIVENs
  name what their THEN observes, pre-Fox collars and merges, TC-MOD-42 and TC-ROOT-13 proven,
  referee kit configs pinned as loose files); GPT A3 6 of 8 (TC-TEX-01/10's tests decode the
  payload, not the header); sidekick S6.13 2; GPT A4 4 of 4 (the earlier-only texture lookup);
  sidekick S6.14 3 of 3. Three conventions now head "Acceptance" ("Format and rules"): `Full`
  unless `Midcup`, PES 21 unless a version is named, and a GIVEN's implied least content.
  Five plan questions went to "open first" on their steps (4.9a(b) collar format, 4.14 MTL
  cascade, 4.19 pre-Fox referee boots) or "Issues" (DpFileList outside a deploying compile).
  Acceptance 246 scenarios (2 withdrawn), 148 proven. Rulings `.tmp/review_queue_S/review_rulings_S6.md`.
- **2026-10-05** — the maintainer answered four questions (decision entry): a collar in the
  other engine's format is converted (TC-CMN-09); sideload and test mode report
  `dpfilelist_missing` as a Warning (TC-OUT-18); a `Full` export keeps the rows of team
  players it does not compile; a referee folder is a player folder, so pre-Fox local boots
  ride in its `face.xml` (TC-REF-09). Test 2, a stripped player with no `BootsList`/
  `GloveList` row, is ready for the maintainer (`.tmp/4_0/apptest/out_test2/GUIDE.txt`).
- **2026-10-05** — more answers (decision entry): the PES 16 Common patch is treated as
  existing; `kitN` is FoxDen's on Fox; the stock collar sets need no confirmation (they were
  counted), so 4.9b waits only on where a collar's textures land; a refs export may hold
  shared boots and gloves, each linking referee's `k99XX`/`g99XX` (TC-REF-10). Test 3
  (`.tmp/4_0/apptest/out_test3/`) checks the converted socks and collar 200 in game.
- **2026-10-05** — third set (decision entry): collars are models only, drawn with the kit
  texture (TC-CMN-08..10); multi-CPK keeps the seed rows; the `autopilot` root marker alone
  opts names in, the Fox patch carries motions, and a compile with nothing to patch backs the
  old patch up as `.bak`. Found: pre-Fox player tables, and referee collars named
  `referee_collar_<ID>` (both in "Issues").
- **2026-10-06** — AATF redesigned with the maintainer (decision entry): a ruleset is data
  (`RULESET`) run by a generic Rhai interpreter carried in each rules file, plus an optional
  `custom_checks` hook; the official ruleset uses the same schema; VGL26 (4ccEditor-VGL) is the
  second embedded ruleset and fixture; VGL's suggestions become a third severity. New plans
  `aatf_rules.md` (the format, moved out of `save_editor.md`) and `ruleset_editor.md` (new tool,
  Phase 20). Checked on the way: Rhai is maintained (1.26.0, 2026-08; only its LSP is abandoned);
  upstream fixed the three AATF errata the old section listed (`f5e7b3e`). No code.
- **2026-10-06** — Ruleset editor: Randomize added to the plan (Sensible / Weird / Crazy, seeded,
  every result backed by a legal witness team; decision entry). VGL26 parity now has its save:
  `C:\Data\4cc\Saves\EDIT00000000_VGL26`. No code.
- **2026-10-06** — VTL11 (VTLEditor `ab5f0e2`) checked against the AATF schema with the
  maintainer (decision entry): small general additions, `team_settings` for PES 16 tactics
  checks, the 188 cm positional rule as a custom check; VTL11 is a third test ruleset with parity
  on `C:\Data\4cc\Saves\EDIT00000000_VTL11`. No code.
- **2026-10-07** — 4ccEditor-VGL `vgl27` (`45ce31d`, tag `VGL27.A`) reviewed: `8f86bd3` is
  upstream's Tactics tab and Texport import merged in (covered by step 5.0); `4447996` is the
  VGL27 ruleset, a fourth AATF fixture, adding `registered_position_fielded` (decision entry).
  Its upstream defects were written up for the maintainer to report. No code.
- **2026-10-07** — 2026's official rulesets checked against the schema: Autumn 25 (`7e01541`)
  and Spring 26 (`c92d535`) fit unchanged and join as test rulesets, official parity now running
  each cup save under its own ruleset (decision entry). No code.
- **2026-10-07** — VGL24 (`d4d7b15`) and VGL25 (`2c2d26a`) checked: the optional manlet bonus
  becomes a one-option `choice` with a `choices` suggestion; VGL25 joins as a test ruleset
  (decision entry). No code.
- **2026-10-07** — VGL26 and VGL27 checked against their rules pages: both follow the pages
  (`fielded_at_a` replaces `registered_position_fielded`; `forbidden_shapes` for VGL26's 3-4-3
  ban; exact injury resistance; tactics settings), VGL27's differences added to the bug report
  (decision entry). No code.
- **2026-10-06** — the Clef scan joins the review (maintainer; decision entry):
  `scripts/clef_scan.py`, `just clef-diff [base]` and `just clef <crate>`, rulings in
  `scripts/clef_rulings.md`, a queue for days the free tier's neurons run out. Checked offline
  with a fake client (windowing, an injected bug flagged and located, cache, rulings, the queue
  across a date change); the live check waits for the quota reset. Next for it: one
  `just clef all` pass over the current code, its flags ruled.
- **2026-10-06** — Clef scan: windows measured positional vs function-sized (positional kept,
  decision entry); the report merges overlapping flagged windows; production code now ends at
  the inline test module, not the first `#[cfg(test)]` (5,007 lines were skipped). The
  scan reads several tokens in order with a per-day fallback (`cf96357`).
- **2026-10-06** — in-game Tests 2, 4 and 5 run by the maintainer, results in "Issues":
  - **Test 2:** a stripped Fox player with no `BootsList` row wears boots 0, so 4.22's drop is
    safe for boots; the gloves default is still open.
  - **Test 4:** PES 17 reads `PlayerAppearance.bin` and `BootsList.bin` for -1 records.
  - **Test 5:** PES 21 referees load `collar_<ID>`, and its referee configs are read from
    `UniformParameter.bin`; PES 17 inconclusive.
  - **Test 3:** postponed; Test 3b added for kit 2 (edit mode shows only kit 1). No code.
- **2026-10-07** — implementation resumed (the Devin lead's review rounds paused; the
  maintainer asks for autonomous work, decisions only they can take logged for later):
  - **4.33:** `name.y` is PES 21's value on every version.
  - **Clef full pass (4.c-pass):** 54 flags, 53 rejected and 1 accepted (step 4.c-fix1). The
    scan now leaves out test modules in files of their own. The maintainer took Workers Paid
    on the second account: the first token stays on the free tier and is used first on
    purpose (`CONTRIBUTING.md` "Clef scan").
  - **4.34a:** the coverage tag is read and required.
  - **Maintainer's rulings:** every team needs one player kit and one goalkeeper kit, so a
    `Full` export without them compiles an empty `p1/` or `g1/` (TC-BIN-19, 20; TC-BIN-14
    given a `g1`).
  - **4.21a:** the working bins come from the installed CPKs (the `DpFileList.bin` layout
    measured on four versions' lists); `pes_folder_path`'s `**` is now expanded; the placeholder
    CPK's one `placeholder` file corrected in the plan and glossary.
  - **4.21b:** the `templates/` override folder; an unreadable override now aborts the run
    whatever the resource (decision entry).
  - **4.21c:** a midcup `fpc_on` export patches the FPC values into its team's installed kit
    configs of the kits the game offers that it does not hold; a full export removes its
    team's other configs; a working bin that does not parse stops the run before any export
    is read. Step 4.21 is done.
  - **4.22:** the Fox boots and gloves lists and the player appearance table come from the
    installed CPKs and carry the compiled players' rows; a list no installed CPK holds is not
    written (`player_table_missing`, decision entry).
  - **4.29a:** a Fox model naming a texture in the team's Common output that neither the
    export nor an earlier installed CPK holds is `fmdl_texture_not_found` (Error, package left
    out; Warning when the CPKs cannot be searched). 4.29b next (the texture link).
  - **4.29b:** a texture `.common` link whose target an earlier installed CPK holds is
    satisfied, in `check` and `compile` alike. Step 4.29 is done.
  - **4.23a:** `compile --mode sideload` writes the CPK's entries as loose files that replace
    `livecpk/` once written whole; refused on PES 15/16 and without a PES folder. 4.23b next.
  - **4.23b:** `compile --mode test` writes each export's processed files, models unpacked,
    under `output/test_output/<source>/<folder>/`, the bins under `_bins/`, overrides not
    applied; `test_output/` replaced once written whole. Step 4.23 is done.
  - **4.24:** `compile` installs the CPK into the game's `download/` (a `.partial` copy renamed
    over the old one); when it cannot (no PES folder, the name unlisted, `download/` unwritable,
    the old CPK in use) it says why and leaves the CPK in `output/`, exit 1. A killed run's
    staging is swept by the next run. Every test now runs against a sandbox PES folder.
  - **4.25b:** the official `DpFileList.bin` is a `templates/` resource; a deploying compile
    compares the installed list with it: `dpfilelist_outdated` (E) for an older list lacking
    the run's CPK, `dpfilelist_not_official` and `dpfilelist_cpk_missing` (W), each naming
    `upgrade-dpfl`. Test installs are now upgraded ones (official list, placeholders).
  - **4.25c:** `upgrade-dpfl [--yes]` lists, then makes, the renames of an old DLC's CPKs by
    stem, the placeholders of empty official slots and the official list (old one as `.bak`),
    the list last; it never deletes or overwrites a CPK. Step 4.25 is done.
  - **4.26a:** the `cpk` writer tells the length its file would have with more entries
    (`len_with`), laid out by the code `finish` writes with. 4.26b next.
  - **4.26b:** multi-CPK mode with `--no-deploy`: whole teams first-fit into the official
    list's `teams` slots under `cpk_part_max_size` (a byte count, 3 GiB), placeholders in
    the slots left, overrides and bins in `4cc_08_bins`; `cpk_team_exceeds_cap`,
    `cpk_slots_exhausted` (Fatal) and `cpk_size_over_limit` (single CPK, Warning). A
    deploying multi-CPK run is still refused; 4.26c installs them all or none.
  - **4.26c:** a multi-CPK compile deploys: every CPK of the run judged by the preflight,
    then installed all or none (old CPKs moved aside first, put back on any failure, every
    CPK then promoted to `output/`). Step 4.26 is done; 4.19 (referees) next.
  - **4.19a:** a referee export compiles on PES 18 to 21: each referee folder built once and
    written for every slot `players.txt` gives it (`referee0NN`, `k99NN`, `g99NN`), its
    textures once under `common/999/<folder>/`, a link to `Boots`/`Gloves` made each slot's
    own; a referee export's kit, logo or portrait is still skipped by name. It lands in the
    run's one CPK until 4.19b writes the refs CPK.
  - **4.19b:** the referees go into their own CPK, `refs_cpk_name` (`4cc_18_referees`),
    installed or promoted with the team CPKs, all or none; a compile whose only output is
    the referees writes that CPK alone and leaves the team CPK installed as it was. A
    `refs_cpk_name` naming a team-side CPK is refused.
  - **4.19c:** the referees' CPK also holds the referee kits and `RefereeAppearance.bin` the
    game needs (Red's `refscpk_fox` tree, embedded); a file at `templates/referees_fox/<game
    path>` in the data directory replaces that file. Sideload writes them too. Step 4.19
    (Fox referees) is done.
  - **4.27a:** a `ref_marker.dds` at a refs export's root goes into the referees' CPK on
    Fox with a flat square model written as collar 77, and the referee kits are set to
    wear collar 77; a team kit whose collar or winter collar is 77 is left out with
    `kit_collar_reserved`. Whether PES 21 reads the loose referee kits that carry the
    collar is still to be tested in game (Issues, referee collars).
  - **4.9b1:** `check` reports a `Collars/` file that is no model, a name that gives no
    stock collar of the version a kit can wear (`collar_id_invalid`, the 9xx ones
    included), a claim on 105 or 77 (`collar_id_conflict`), and a collar model's check
    findings as any model's. `compile` still refuses an export that keeps a collar.
  - **4.9b2:** a team's `Collars/collar_<ID>.fmdl` compiles on Fox, and every kit of the
    team wears it, after FPC, kits a midcup does not resend included; the later of two
    exports claiming one collar, or an export's second collar, is `collar_id_conflict`.
    Step 4.9 is done.
  - **4.28:** a running task's decoded textures, parsed models and finished entries are
    charged to the run's memory budget, without waiting, so later tasks wait for them to
    be freed. No member-visible change. 4.y (bounding the conversion cache) moved to
    Phase 8, where the GUI first keeps a cache across runs.
  - **4.32:** a kit's number textures made for the other games (a column of digits for
    PES 2015 to 2017, a row for 2018 to 2021) are re-arranged for the game compiled for,
    each digit moved into the places Konami's own textures keep it, measured on the stock
    textures of both games. The name texture is left as it is.
  - **4.18:** on PES 2018 to 2021, a face model whose vertices are weighted to the hand
    bones has its hands cut off at the wrist into the player's gloves, with their own
    gloves ID and row, and the note `model_hand_split`; a model named as boots or gloves
    is never cut.
  - **4.14a:** a per-kit model's name is read without its kit token, on every game:
    `boots_kit1.fmdl` is boots, not a part of the face's hair, and a shared `Boots/`
    folder may hold per-kit boots, of which kit 1's is compiled on PES 2018 to 2021.
  - **4.14b:** PES 2015 to 2017 compile a player folder's own `.model` files with their
    `.mtl` files, textures and face diff into the player's face CPK, with a generated
    `face.xml` typing each model by its name (boots and gloves models included), and
    the blank face of a player with no model; a model with no `.mtl` is
    `model_material_undefined`. Links, `Common/`, shared folders, kits, collars,
    `ingame_face`, `.fmdl` files and referee exports are still refused on those games.
  - **4.14c1:** on PES 2015 to 2017 a linked `Faces/` folder is copied into the player's
    face (his own file of the same name wins), and a linked `Boots/` or `Gloves/` folder is
    written once under its shared ID as loose files, the boots as `boots.model` and
    `boots.mtl`, the gloves under their own names with a generated `glove.xml`. `Common/`,
    kits, collars, `ingame_face` and referee exports are still refused there.
  - **4.14c2:** on PES 2015 to 2017 the export's `Common/` folder compiles: its models,
    `.mtl` files and textures are written once into the team's Common folder, a
    `.model.common` link puts that model in the player's `face.xml` by its Common path, a
    `.mtl.common` link or a Common `.mtl` found for a linked model is named there too, and a
    texture link points the player's `.mtl` at the Common texture. Kits, collars,
    `ingame_face` and referee exports are still refused there.
  - **4.14d:** on PES 2015 to 2017 `check` and `compile` refuse a folder holding
    `face_edithair.xml` or `hair.xml`, a shared boots or gloves folder's model without its
    suffix (`model_name_invalid`), a texture whose width or height is not a multiple of 4
    (`texture_not_div4`, even with `pass_through`), and a model naming a material its `.mtl`
    lacks (`model_material_undefined`, which `pass_through` may keep); on PES 2015 a
    `uniform` model is written `uniform_sub` (`xml_uniform_pes15`).
  - **4.31:** a pre-Fox parity test compiles a real PES 2017 player (face, boots, gloves)
    and compares the CPK with what the legacy compiler built from the same export. Nothing
    a member would see changes.
  - **4.14e1:** on PES 2015 to 2017 a player folder holding `ingame_face` compiles: every
    model but a glove becomes part of his own boots, merged into one `boots.model` with a
    boots folder his link combines (`model_merged`, `link_combined`; conflicting parts leave
    the boots out with `merge_material_conflict`, `skl_merge_conflict` or
    `model_merge_flags_conflict`), and a shared `Boots/` folder holding several boots models
    is merged into its `boots.model`. Gloves beside the marker are still refused there.
  - **4.14e2:** on PES 2015 to 2017 an `ingame_face` player's gloves compile into his own
    gloves folder, unmerged, each model with its `.mtl` and a `glove.xml`, a gloves folder his
    link combines adding its models (his own file replacing one of the same name); a shared
    gloves folder no longer writes a `.mtl` none of its models uses.
  - **4.14e3:** on PES 2015 to 2017 a face model whose vertices weigh on the hand bones is
    split at the wrists at compile time (`model_hand_split`): the body keeps its `face.xml`
    entry and its hands become `oral_<stem>_glove_l_win32.model` and `_glove_r_` entries
    typed `gloveL`/`gloveR` naming the same `.mtl`; a model named as boots or gloves, one a
    `.common` link brings in, and one under `ingame_face` are not split; a split model whose
    `.mtl` is a Common file is refused (decision entry).
  - **In-game tests run by the lead** (maintainer's authorization of 2026-10-08: PES 2017
    and 2021 on `E:` windowed, saves and game files backed up) through a Python harness,
    `.tmp/4_0/ingame/game.py` (Sider then the game, not elevated; screenshots of the window's
    client area with `mss`; DirectInput keys with `pydirectinput`; `burst.py` tiles frames),
    every game-side change made and undone by a hashed install/revert script: Test 5b/5c
    settled the pre-Fox referee collar names (Issues, referee collars) and Test 4b the
    pre-Fox boots rule (Issues, pre-Fox player tables); gloves stay open there. Every file
    checked back to its baseline by hash after the runs (save, SYSTEM, the exe, the test CPK
    slot, the livecpk root).
  - **4.14e4 done** (TC-CMN-07): on PES 15-17 a per-kit `.model` set is packed whole and
    listed once in the generated `face.xml`, through its lowest variant's entries with the kit
    token spelled `kitN` in `path` and `material` (decision entry: the exe to come respells
    the whole entry); the set is found over the face's packed names, a linked shared face's
    files and the player's own together; a variant whose `.mtl` is not the one the entry
    implies is `kit_variant_mtl_differs`; a `.mtl` texture path naming `pants_kitN` is pointed
    at the texture home. Still refused by name: a per-kit model under `ingame_face`, in a
    shared boots or gloves folder, or behind a link. Next: 4.14e5.
  - **4.14e5 done** (TC-MOD-44, TC-MOD-45), the last slice of 4.14: under `ingame_face` on
    PES 15-17 a `.model.common` link's Common model is copied in as one more part of the
    player's boots merge or one more model of his gloves, with the `.mtl` its search finds,
    and a `.mtl.common` link one of his parts uses is copied the same way; a copied Common
    `.mtl` names the Common textures in the team's Common output (decision entry). 4.14 has
    no refusal left for a pre-Fox player folder but a member's own `face.xml` (4.15). Next:
    4.15.
  - **4.15 recon**: the `mtl_material_duplicate` and `mtl_state_*` rows were already reported
    by the deep pass from `pes_model::check` (4.14d), so 4.15 is the member's own `face.xml`
    and `mtl_texture_not_found`; three items deferred with reasons in the step. Red keeps a
    member's xml and its file names as written (the pre-Fox tracer's Fumos folder: Red's output
    names `./face_high_*.model` beside `face_high_win32.model`), so a `./` reference packs its
    file under the referenced name and a Common reference is written as the Common output packs
    the model (decision entry; plan paragraph "How a `path` or `material` is resolved"). The
    step is sliced (a1) read/check, (a2) emit, (b) textures.
  - **4.15a1 done** (TC-XML-03, TC-XML-04 first half, TC-XML-06, TC-XML-09): a member's own
    `face.xml` is parsed, its references resolved and every content check of the table
    reported by the deep pass, which compares a listed model's materials against the xml's
    `.mtl` rather than the search's; the folder has a face; on Fox the xml is ignored with
    `xml_ignored_fox` and the gate lets it through; the pre-Fox gate still names it (a2 emits
    it). Next: 4.15a2.
  - **4.15a2 done** (TC-XML-01, TC-XML-02, TC-XML-04, TC-XML-05): on PES 15-17 a member's own
    `face.xml` compiles: written back in the generated shape with its attributes and unknown
    elements verbatim, a `./` file packed under the referenced name, a Common model written as
    the Common output packs it, the dummy and the PES 2015 rewrite as for a generated xml, its
    own `<dif>` else the folder's; only the files it names are packed; no hand split for such a
    folder; the pre-Fox tracer's Fumos xml compiles verbatim as a test. TC-XML-07 is 4.15b's.
    Next: 4.15b.
  - **4.15b done** (TC-XML-07; 4.15 closed): on PES 15-17 the deep pass looks every `.mtl`'s
    texture paths up by the face task's stem rule, `check` and `compile` alike: a miss on a
    material a paired model's mesh binds is `mtl_texture_not_found`, a Warning keeping the
    folder (the plan's Error refused the cup-played Fumos tracer; Red warned on the same file),
    a miss only unbound materials name `mtl_texture_unused_missing`; a Common-form path is
    looked for in `Common/` and the installed CPKs listed before the run's. Deferred: a pre-Fox
    texture link satisfied by an installed CPK; the `XXX` segment of a supplied Common-form
    `.mtl` path left as written. Next: 4.16.
  - **4.16a done** (TC-KIT-19, 21, 22, 23, 26, 28, 29, TC-BIN-04): PES 15-17 compile kits: the
    gate names only a kit texture no engine builds (a stem outside the seven, shared with the
    Fox walk), the textures go out as DDS under `common/character0/.../uniform/texture/`, the
    mask the kit's own or the bundled template byte for byte, the srm dropped with
    `kit_texture_not_used` (the mask on Fox), the main texture and the target's map re-laid by
    the marker and reported once per kit, the loose config the kit's only one (no
    `UniformParameter.bin`), the two color bins as on Fox and none of the Fox player bins.
    Red's mask template is bundled with its provenance (`resources/kits/README.md`). Next: the
    pre-Fox tracer's kit parity rows (lead), then 4.16b.
  - **4.16a parity** (lead): the pre-Fox tracer's twin gains `Kits/g1/`, and the parity test's
    four kit rows compare: the mask byte for byte, the texture by pixels, the config outside
    the four number-texture names Red keeps for textures the export does not ship (a question
    for the maintainer, in the 4.16 step), `UniColor.bin` as the bundled base with Red's `g1`
    entry set (Red's fallback base held an empty record for the team).
  - **4.16b done** (TC-TEX-08): `dds_compression` (`auto` follows `multicpk_mode`, `true`,
    `false`) WESYS-zlibs every `.dds` a PES 15-17 run emits, on the worker that made it; a
    wrapped source the conversion would keep is emitted as it is; Fox ignores the setting.
    Next: 4.16c (collars for pre-Fox), 4.16d (installed loose kit configs patched).
  - **4.16c done** (TC-CMN-08): a team's `Collars/collar_<ID>.model` compiles for PES 15-17 as
    its FMDL does on Fox, written unchanged in place of the stock collar, the claim and the
    conflict rules the same; a `.fmdl` collar is named there until 4.17 converts it. Next:
    4.16d.
  - **4.16d done; 4.16 closed** (TC-BIN-18): on PES 15-17 a `Midcup` export's absent kit slots
    have their installed loose configs gathered by the working-bin walk, FPC-patched and
    collar-worn as the `UniformParameter.bin` entries are on Fox, and the changed ones go out
    through the bins path (DECISIONS 2026-10-08). Next: 4.17 (cross-format conversion and
    source selection).
  - **4.17-lead** (recon `.tmp/4_17/recon_4_17.md`, Opus, read-only): the step's text widened
    (TC-MOD-26's PES 17 half, same-engine retargeting, the loss-code mapping, shared folders
    and links as slice f), the source-selection rulings logged (DECISIONS 2026-10-08), the
    template environment cubemap bundled with its provenance. Next: 4.17a (a Fox face
    converted for PES 17 through the face task).
  - **4.17a done** (TC-MOD-27): on PES 15-17 a player's `.fmdl` with no `.model` twin is
    converted through `model_convert` and packed by the face as a `.model` and `<stem>.mtl`,
    its `.skl` the bind pose; the Fox tracer compiles whole for PES 17 and PES 15. The game's
    dummy textures cross engines by rule (DECISIONS 2026-10-08): the FMDL import gives
    `dummy_nrm`/`dummy_srm` no role, `dummy_kit*` is pointed at the team's Common directory.
    Next: 4.17b (pre-Fox to Fox in the Models task, the selection rule on Fox).
  - **4.17b done** (TC-MOD-26, 29, 30, 34): on PES 18-21 a player's `.model` with no `.fmdl`
    twin is converted in the Models task with the `.mtl` its search finds, the skeleton the
    conversion writes taking a member's `.skl` path; every converted model is checked in its
    target form, a failure leaving its package out as every task failure does (DECISIONS
    2026-10-08); a beaten model is read by nothing, and a selected `.model`'s `.mtl` gets the
    texture check on Fox. Next: 4.17c (every loss code mapped, Warnings of the converted form,
    the same-engine pre-check, `model_gltf_unsupported`).
  - **4.17c1 done**: every `model_convert` loss code is reported at a catalog severity
    (Warning for a change in how the model looks or moves, Info for a drop the game cannot
    show), the converted form's own Warnings and Infos are not (DECISIONS 2026-10-08). The
    same-engine pre-check was wired as the plan said and measured: `needs_conversion`'s
    reference poses disagree with the pre-Fox parity tracer on Red's own target and fire on
    Fox only for boots shipping the game's `boots.skl`, so it is deferred to slice (g), a
    two-crate step reconciling the poses first (DECISIONS 2026-10-08). Next: 4.17c2 (a
    selected glTF refused at planning, TC-MOD-28).
  - **4.17c2 done** (TC-MOD-28): a `.glb`/`.gltf` with no model of the target's own format
    of its stem beside it is its stem's selection, beating the other engine's model; planning
    drops its player folder with `model_gltf_unsupported` instead of the gate refusing the
    export, never compiling the beaten model in its place; a glTF the target's format beats
    is ignored. A shared folder's glTF stays with the gate until slice f. Next: 4.17d (the
    template environment cubemap for a converted metal material, TC-MOD-36).
  - **4.17d done** (TC-MOD-36): a Fox metal material converted for PES 15-17 gets an
    `EnvironmentMap` sampler naming `env.dds` in the player's texture home, where the
    textures task emits the template cubemap unless the folder holds an `env` texture of
    its own; the deep pass records the metal FMDLs with the converter's own family rule and
    planning flags the folder. Decision logged. Mutants: 27, 21 caught, 6 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17e
    (collars across engines, TC-CMN-09; plan edits and the rulings first).
  - **4.17e done** (TC-CMN-09): an FMDL collar compiled for PES 15-17 is converted with the
    stock collars' material names and no `.mtl`; a glTF collar is dropped at planning with
    `model_gltf_unsupported`, the file alone; a `.model` collar for PES 18-21 stays with the
    gate. Two decisions logged. Mutants: 24, 19 caught, 5 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f (shared folders,
    `Common/` models and `.common` links in the other format, the shared folder's glTF, the
    `ingame_face` FMDL parts, the remaining 4.17 open items; sliced after an exploration).
  - **4.17f1 done**: a shared folder's `.model` on PES 18-21 and a shared face's FMDL on
    PES 15-17 convert as a player's own, the gates no longer refusing them; a shared folder
    whose selected model is a glTF is removed at planning and every player linking it is
    dropped with `model_gltf_unsupported` naming the shared file. Decision logged. Mutants:
    27, 27 caught, 0 unviable, 0 missed. Next: 4.17f2 (a shared boots or gloves folder's FMDL and an `ingame_face`
    player's FMDL parts converted by the pre-Fox boots and gloves writer; the two plan gaps
    from f1).
  - **4.17f2 done**: on PES 15-17 a shared boots or gloves folder's FMDL and an `ingame_face`
    player's FMDL parts convert in the boots and gloves writer as the face converts one, the
    gates no longer refusing them; a conversion finding names a shared source by its export
    path; role-less Fox files in a shared folder ignored as a player's. Decision logged.
    Mutants: 60, 57 caught, 3 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f3 (`Common/` models and `.common` links in the other
    format, per-kit `.model` variants on Fox, the pre-Fox Common `.mtl` path on Fox,
    `kit_variant_model_fox` on a shared folder linked only by a dropped folder, the `env`
    link ruling, the remaining `model` contexts, the env-sampler asymmetry).
  - **4.17f3a done**: the four leftovers of 4.17f: planning's glTF drop removes the shared
    folders it orphans; a player's `env.dds.common` link is the environment map his
    converted metal materials name, no template emitted and no empty textures task planned
    (one planned lost his face CPK silently: the writer takes an empty batch for a failed
    task's, now an open converge item); every `model` context of a folder's task named
    through `source_name`; the boots and gloves writer adds the environment sampler as the
    face does; `kit_variant_model_fox` silent on pre-Fox, where the set is packed whole
    (lead fix from the sidekick's probe). Mutants: 19, 15 caught, 4 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f3b (an FMDL
    in `Common/` converted once in the Common models task on PES 15-17, the `x.fmdl.common`
    link listing its `.mtl`, the marker case).
  - **4.17f3b done**: on PES 15-17 an FMDL directly in `Common/` converts once, in the
    Common models task, into the team's Common output with its material set as
    `<stem>.mtl`, its `.skl` the bind pose and the environment map emitted there for a metal
    one; an `x.fmdl.common` link lists it at the Common path with that `.mtl`, and under
    `ingame_face` converts into the part; a link to a Common FMDL a Common `.model` of its
    stem beats loads the `.model`. Decision logged. Mutants: 52, 43 caught, 9 unviable (`Default::default()` on types without one), 0 missed. Next: 4.17f3c
    (PES 18-21: a Common `.model` + `.mtl` converted in each linking player's Models task;
    per-kit `.model` sets; the pre-Fox Common texture path pointed at the Fox Common
    directory).
- **2026-10-08** — `.tmp/` sorted (maintainer's request): one folder per subphase (`4_14a`
  and `4_14b` under `4_14/`), `lead/` for the lead's live notes, `review_queue_S/` for the
  S1-S6 review rounds, `reference/` for cup material; files with no step in their name went
  by the subphase the log mentions most on their mtime day (about 550 of 2063 entries, so
  some sit a step off). Every `.tmp/<name>` reference in the docs, `AGENTS.md`, the fixture
  READMEs and `scripts/provenance/` repointed (85 + 7). Deleted as regenerable: every cargo
  `target/` under `.tmp/` but the apptest tool's, and 4.7's laid-out timing exports and
  outputs (`timing_build.py` rebuilds them): 5.9 GB of 7.0. What else can go is checked at
  the end of the implementation work, not now. Scripts and the move list in `.tmp/sort_tmp/`.
- **2026-10-08** — Test 4c (in-game, PES 17; Issues "pre-Fox player tables", the Test 4b
  paragraph): no. 1's PlayerGloves bit set on his DB appearance row changed nothing, bare
  hands in Edit mode (lead) and in a match (maintainer). A GloveList.bin row never applied
  on PES 17; pre-Fox gloves stay in the savefile. The maintainer settled that Edit mode shows
  a player as a match does (only outfielders' 2nd+ kits need a match), so in-game look tests
  stop at Edit mode.
- **2026-10-08** — `docs/QUESTIONS.md` created (maintainer's request): the one list of the
  questions only he can answer, fourteen moved out of the worklog's steps and issues (each
  of those now points there); `AGENTS.md` "Working documents" gained the row.
  - **4.17f3c done**: on PES 18-21 a `.model` directly in `Common/` with its `.mtl`, linked
    by `x.model.common`, converts in each linking player's Models task as his own `.model`
    does, an FMDL of its stem beating it, its Common `.mtl` paired and looked up in the deep
    pass on the player's folder; a per-kit model set's variants are the folder's `.fmdl`
    and `.model` files, a number in both formats one variant with its FMDL selected; a
    converted path naming the pre-Fox Common texture directory is pointed at the Fox Common
    directory when `Common/` holds the stem. Two contradictions accepted (the finding names
    the Common model's export path; the fixture). Mutants: 76, 62 caught, 13 unviable
    (`Default::default()` on types without one, `||` in conditions the types refuse), 1
    missed (`model_variant_sets`'s dedup rule for two files of one number and format),
    caught by the lead's added assertion (rerun on the function: 5 caught, 1 unviable).
    Next: 4.17g (the same-engine pre-check of a selected native model).
  - **4.17g1 done** (measurement, no code): which pose each real model is bound to, on the
    games' own files (stock PES 2017 faces, boots and gloves out of `dt33_win`/`dt36_win`,
    stock PES 2021 boots and gloves FPKs out of `dt33_g4`) and the tracers
    (`.tmp/4_17/sk_4_17g1_report.md`). Faces carry a per-face `skf_*` pose (Konami's up to
    0.34 off `face.skl`); boots sit on one boots pose 7° off the body on every version, both
    feet in one mesh; the pre-Fox tracer gloves sit on PES 15's arm pose with every bone
    sharing one delta; a stock glove is 0.0025 off the hand table, the smallest version
    difference 0.0042. Ruling (DECISIONS 2026-10-09): the pre-check tests whether a vertex
    blends bones with differing deltas, `skf_*` take no part, tolerance 3e-3; no boots table.
    Fixtures copied by the lead into `model_convert/tests/fixtures/` (stock PES 17 boots and
    glove, stock PES 21 boots with its six-bone `.skl`, the tracer's three models). The
    2026-10-08 ruling's three comparisons are withdrawn. Next: 4.17g2 (`model_convert`).
  - **4.17g2 done**: `model_convert::needs_conversion` is the plan's blended-delta test (a
    used bone the target lacks, or a vertex blending bones whose deltas to the target's table
    differ beyond 3e-3; `skf_*` and custom bones take no part), shared by both engines through
    one pass over the weight rows; `retarget` keeps a face's `skf_*` matrices; the tolerance
    3e-3. On their own version the stock PES 17 boots and glove, the stock PES 21 boots with
    its `.skl`, the tracer's boots, gloves and face and the one-bone card are all `false`
    (every one was `true` before). One contradiction accepted: a pre-Fox weight below half a
    Fox step (0.5/255) is no weight, since the tracer's boots carry 2^-24 of noise on the
    other leg's bone (plan item 3, DECISIONS 2026-10-09). Side effect pinned by a lead test:
    a converted face now writes an SKL carrying its own `skf_*` pose (`needs_skl`), where the
    re-bind used to hide it. Mutants: 23 (19 caught, 3 unviable, 1 missed: `slot + 1` to `slot * 1` in the pair loop of `blends_differing_deltas`, equivalent, since a slot compared with itself never differs). Next: 4.17g3a (`team_compiler`, Fox).
  - **4.17g3a done**: the Fox Models task runs `needs_conversion` on every selected FMDL (a
    player's own, a shared folder's, a `.common` link's) with the `.skl` of its stem as the
    bind pose, through one `convert_part` for both formats (`conversion::fmdl_for_fox`;
    `fox_written` shared with `model_for_fox`); a flagged FMDL is moved onto the version's
    skeleton (`skeleton_retargeted`), its `.skl` dropped for the conversion's; skeletons are
    read before the models; the gloves task's files include a hand-split part's `.skl`
    (`ModelFolder::hand_split_skeleton`), so its hands are moved as the face's body is. The
    skeleton a conversion writes for a slotless role is dropped with no finding, `skl_no_slot`
    staying a member's-file warning (every converted face writes one since g2: TC-MOD-48's red
    run showed the warning at HEAD). Collars are not pre-checked. Lead measurements
    (`.tmp/4_17/lead_g3a_probe/`): the Fox tracer's four FMDLs with their own, PES 18's or
    PES 19's skeleton, the hand-split body and every `fmdl` fixture are `false` on PES 18-21,
    so only a moved blended bone flags a Fox model. TC-MOD-46, 47, 48 added and proven. One
    test's input replaced (the linked Common model's export-path naming, now through a renamed
    bone's `skeleton_retargeted`). Mutants: 29 (21 caught, 8 unviable: type-driven defaults and an `&&` inside a let chain, none missed). Next: 4.17g3b (`team_compiler`,
    pre-Fox).
  - **4.17g3b done**: `conversion::model_for_pre_fox` (the `.model` taken, the bytes to pack
    returned: the source as it is, or the model moved onto the version's skeleton), its
    writing half `pre_fox_written` shared with `fmdl_for_pre_fox`; called by the face
    (`FaceSource::Member`), the shared boots and gloves writer (`prefox_shared::package`, both
    arms) and the Common models task; the face task's files include the Common `.mtl` each
    `.mtl.common` link names (`plan::linked_common_materials`), so a model with a Common
    `.mtl` is checked and converted too, and the hand split of such a model is no longer
    refused (DECISIONS 2026-10-09); a `Common/` `.model` with no `.mtl` in `Common/` is packed
    as it is (sidekick's finding: the `expect` fired on a linking player's own `.mtl` and on an
    unlinked model). The cap folds two bones on PES 15 and re-binds none, so the tests pin the
    fold lines, not `skeleton_retargeted` (TC-MOD-49 to 52 reworded). Two rework rounds: R1 the `model_for_pre_fox` shape (the bytes to pack returned) and the hand split over a Common `.mtl` allowed, both from the sidekick's own report; R2 one test for the mutation run's survivor.
    Mutants: 26 (21 caught, 5 unviable: type-driven defaults; the first run's one survivor, the dedup in `linked_common_materials`, caught by R2's two-links test). Next: 4.19d.
- **2026-10-09** — Reviews run by the lead from now on: the maintainer wired the `duck` skill
  (Astra, SWE-2 rounds) for Claude Code, so cross-family reviews are no longer queued except on
  an Astra quota stop (five-hour quota, five or six reviews; retried hourly). The S7-S16
  backlog and the four queued 4.14/4.17 reviews run as the first half of converge, after the
  remaining slices (maintainer: finish the crate first). "Handover" rule and `AGENTS.md`
  "Second opinion" reworded. 4.17g started as g1, a measurement brief (`.tmp/4_17/brief_4_17g1.md`):
  which game table each real model is bound to (the tracers, stock PES 17 and PES 21 models from
  the installs, the converter's fixtures, the body-table version deltas), so g2 changes the
  converter from numbers, not from a reading of its code. The maintainer also noted that the
  in-game harness on PES 17 and 21 can answer `docs/QUESTIONS.md` entries without them.
- **2026-10-09** — 4.17g1 (measurement): the sidekick's probe (`.tmp/4_17/sk_4_17g1_probe/`)
  measured every real model's bind pose against the tables; the 2026-10-08 ruling (face table,
  `boots.skl`, hand pose) is withdrawn and the pre-check becomes a blended-delta test
  (`conversion.md` "When re-binding changes anything", DECISIONS 2026-10-09); fixtures added.
  Test 3 on PES 17 run through the harness as far as Edit mode allows: team /a/'s outfielders
  are all billboard or custom-body models, so kit 1's socks and collar 200 cannot be seen there
  (only a match would); the keeper's collar 201 (no model anywhere) drew a plain neckline, no
  crash. The PES 21 half and Test 3b are left for the maintainer; the apptest scripts'
  absolute `.tmp/apptest` paths were repointed to `.tmp/4_0/apptest` (missed by the sort).
- **2026-10-09** — 4.17g2 landed (sidekick, first time; one contradiction accepted, the
  pre-Fox weight floor): `needs_conversion` as the blended-delta test, `skf_*` out of the check
  and the re-bind, tolerance 3e-3, seven real fixtures `false` on their own version. Lead test
  added (no red run; it pins the `needs_skl` side effect: a converted face's SKL carries its
  own `skf_*` pose). Gates 255 proven. Mutants: 23 (19 caught, 3 unviable, 1 missed: `slot + 1` to `slot * 1` in the pair loop of `blends_differing_deltas`, equivalent, since a slot compared with itself never differs). The FMDL importer's
  `bone_matrices` reading filed for converge.
- **2026-10-09** — 4.17g3a landed (sidekick, first time; one test input replaced, reported):
  the Fox pre-check call per selected FMDL with its `.skl`, the gloves task reading a
  hand-split part's `.skl`, the slotless conversion skeleton dropped silently (the g2 side
  effect would have warned `skl_no_slot` on every converted face: proven by TC-MOD-48's red
  run), collars not pre-checked; two decision entries, plan edits first; TC-MOD-46 to 48.
  Gates 258 proven. Mutants: 29 (21 caught, 8 unviable: type-driven defaults and an `&&` inside a let chain, none missed). The `bone_matrices` converge item extended
  (exporter writes the block, `needs_skl` then skips `skf_*`).
- **2026-10-09** — 4.17g3b landed (sidekick; two rework rounds: the `model_for_pre_fox` shape
  and the hand split over a Common `.mtl` allowed, both from its own report; then one test for
  the mutation run's survivor, two `.mtl.common` links to two Common files): the pre-Fox
  pre-check call for every packed `.model`, the member's `.mtl` kept, the Common `.mtl` a link
  names read by the face task; three contradictions accepted (the cap folds and re-binds
  nothing, the IR drops `.model` mesh tags, the hand-split refusal lost its reason), one plan
  gap accepted (a `Common/` `.model` with no `.mtl`); three decision entries. Gates 262 proven.
  Mutants: 26 (21 caught, 5 unviable: type-driven defaults; the first run's one survivor, the dedup in `linked_common_materials`, caught by R2's two-links test). 4.17's slices are done; converge items filed (`.model` collars,
  mesh tags, `bone_matrices`).
- **2026-10-09** — 4.20 ruled and 4.20a landed. Lead: a reconnaissance (Opus agent,
  `.tmp/4_20/recon_4_20.md`) of what validation, planning and processing do with each class
  the gate named, eight decision entries, the plan edits, TC-OUT-06 withdrawn and eleven IDs
  added; the pre-Fox referee template tree bundled for 4.19d. Sidekick: the removal and
  `file_not_used`; one tests-only rework round (the mutation run's two survivors) and one lead
  fix (the pre-Fox list of Common files a face reads made direct-only, which R1's test showed
  nested); eight contradictions, one applied (the deep
  pass's search resolving a shared folder's `.mtl` link, a panic), one deferred to b1 with a
  decision (a Common-set part's `./skin.dds`), six accepted (material count, TC-OUT-02's inputs,
  a line's form, the glTF rule's reading, which becomes b1's, face files in boots folders, the
  help). Gates 268 proven. Mutants: 78 (62 caught, 16 unviable: type-driven defaults and an && inside a let chain; the first run's two survivors, the Fox Common .mtl filter's && and admitted, caught by R1's tests).
- **2026-10-09** — 4.19d lead-first: the pre-Fox referee template tree bundled
  (`resources/templates/referees_prefox/`, 51 files from Red `e12aa01`, byte-identical to
  Blue's), its provenance in the templates README, the tree marked `-text` in `.gitattributes`
  (three CRLF `.mtl` files). Nothing reads it yet: 4.19d chooses the tree by engine.
- **2026-10-09** — 4.19d lead-first (2): `resources/templates/collar_empty.model` bundled (FPC's
  empty collar, from `4cc_04_fpc.cpk`, provenance in the templates README); the marker paragraph
  of `blue_port.md` names the `referee_prop` pair and the empty collar; decision entry; 4.27's
  pre-Fox half folded into 4.19d; brief drafted.
- **2026-10-09** — 4.20b1 landed (Opus sidekick, one round, no lead fix): unused kit stems,
  `shared_folder_no_model`, a Common-set part's textures, the Common glTF order. One
  contradiction applied by the sidekick and accepted with a decision entry (a player whose link
  names a beaten Common model is dropped: the brief's shape panicked in every link reader), four
  accepted (a selected glTF counts as a model, roster order is slot order, the readers of the
  removal, TC-MOD-54's THEN), three observations sent to b2's brief (the help's gate passage,
  `shared_folder_no_model` in the help, a refs kit reported twice). Gates 270 proven. Mutants:
  51 (38 caught, 12 unviable: type-driven defaults; one survivor, the `||` of `linked_common_gltf`'s link filter, caught by a lead input added to the Common glTF planning test, a `.mtl.common` link whose stem a selected glTF holds, which must not drop its player; the kill checked by a hand perturbation).
- **2026-10-09** — 4.20b2 landed (Opus sidekick, one round, no lead fix); 4.20 closed: per-kit
  model sets left out on PES 15-17 where no `face.xml` names them (the finding renamed
  `kit_variant_model_left_out`, the set's selected representation by the target's format), a
  shared folder's `face.xml` ignored with `xml_ignored_shared`, the help's gate passage gone, a
  refs export's kits left to validation. Seven contradictions, none applied against the brief:
  one widened the rule's plan text (any shared folder's xml), one recorded in the plan (a lone
  variant under the marker is a part), one brief error (ten callers, not nine), four accepted
  as written. Gates 272 proven. Mutants: 61 (47 caught, 14 unviable: type-driven defaults and an && inside a let chain, 0 missed).
- **2026-10-09** — 4.19d landed (Opus sidekick, one round, no lead fix); 4.27's pre-Fox half
  with it: a PES 15-17 refs CPK carries the pre-Fox template tree (51 files, overridable from
  `templates/referees_prefox/`), and `ref_marker.dds` goes out as `referee_collar_077.model`
  (the tree's prop model), its `.mtl` holding `judge_incom` alone pointed at the marker
  texture, and the bundled empty `collar_077.model`. Four contradictions accepted (TC-REF-09's
  wording twice, the pre-Fox prop unskinned rather than painted to `static`, the pre-Fox
  configs' collar 26); the TC-REF-10 probe opened 4.19e (a pre-Fox referee's shared boots
  written twice). Gates 274 proven. Mutants: 55 (30 caught, 25 unviable: type-driven defaults and leaked-static replacements of the tree accessors, 0 missed).
- **2026-10-09** — 4.27b landed (Opus sidekick, one rework round, the multi-CPK refs-only test the sidekick itself flagged as uncovered; no lead fix); 4.27 closes: on
  PES 18-21 every referee kit config written to the refs CPK is also the entry of its file
  name in the run's `UniformParameter.bin`, with the bytes written (collar 77 under the
  marker, a `templates/referees_fox/` replacement's bytes), so a run whose only committing
  export is the refs export writes the team side for the bin (multi-CPK: the bins CPK and
  the placeholder parts); PES 15-17 unchanged. One contradiction accepted: the bundled
  bases hold no `referee_1..5` entries (a byte grep had read kit texture names inside the
  configs as entry names); the `LB` and `SDA` entries are new, inserted by name. Gates 275
  proven. Mutants: 10 (8 caught, 2 unviable: type-driven defaults of finish_team's return, 0 missed).
- **2026-10-09** — 4.19f landed (Opus sidekick, one round, no rework; lead fix: a test name
  shortened): on PES 15-17 `ref_marker.dds` goes out converted as the template tree's
  `parts/referee/incom_bsm.dds`, the texture of the prop the game draws under the referee,
  in place of the tree's file (skipped in `finish_referees` as an override's is); no collar
  pair, no `collar_077.model`, the configs the template's; `collar_empty.model` and its
  resource removed, one marker path per engine (`paths::referee_marker`). Six contradictions
  accepted (an `Engine` match, not `==`; `referee_tree_file` removed with its last caller;
  a help sentence and a README sentence the change made false; the tree's own
  `incom_bsm.dds` is WESYS-compressed where the compiler writes a plain DDS, as Red did,
  noted in the plan and the open question; an over-long test name). Gates 275 proven.
  Mutants: 15 (13 caught, 2 unviable: type-driven defaults of the Referees constructors, 0 missed).
- **2026-10-09** — 4.19e landed (Opus sidekick, one round, no rework; lead fix: the help's
  referee paragraph): on PES 15-17 a referee's link to a shared `Boots/` or `Gloves/` folder
  is written as his slot's `k99NN`/`g99NN` folder alone, no longer copied into his face as a
  `parts` entry (the pre-Fox face reads a combined boots or gloves source's texture stems
  alone; planning's file list likewise); a referee folder with no face model gets no face
  folder on either engine, the game's referee head staying (decision entry: a referee has no
  FPC body to bring a head). Two of the brief's rulings corrected by the sidekick on
  evidence, both accepted. TC-REF-13 proven, TC-REF-10 extended; the Phase 4 steps are all
  done but converge. Gates 276 proven. Mutants: 9 (7 caught, 2 unviable: type-driven defaults of TaskKind::files and plan_run, 0 missed).
- **2026-10-09** — Converge (4.y-conv) opened: the 4.14 and 4.17 parent bullets closed with
  their verify criteria re-run (TC-MOD-20, 26, 27, 28 tests green). The lead's audit runs
  first (`.tmp/lead/converge_4_plan.md`), the `duck` reviews after it.
- **2026-10-09** — 4.y-conv, the lead's audit done: 64 verify criteria re-run (55 proven, 3
  without a proving test, 1 stale), TC-OUT-18 proven by a lead test, two catalog rows
  withdrawn, 6 Clef flags ruled (1 accepted), the export census over 71 roots (55 old-layout
  exports, 16 referee exports) diagnosed into eight classes. Three gap steps added above
  4.y-conv (small fixes; `export_layout_old` and the dual-engine face diff, two decision
  entries; cube-map textures), four issues and two maintainer questions logged.
- **2026-10-09** — 4.y-fix1 done (sidekick, landed first time): the eight converge fixes;
  three issues settled by tests, one of them red (a Fox `ingame_face` player's own texture
  counts for his boots, `merged_texture_conflict`). Whole-crate mutation runs of
  `aesthetics_export` (548 mutants, 2 survivors) and `pipeline` (72, 6 survivors, all one
  platform read) done and triaged into 4.y-fix2 (the `pipeline` six turned out to be the
  platform arms `.cargo/mutants.toml` already lists as accepted, each half running the other
  platform's body: 4.y-fix2's report); `team_compiler`'s run next, over this tree.
  `team_compiler`'s run done 13:34 (2,222 mutants over `23f68ae`, split: 1,111 a half, 2 h 25
  local and 2 h 39 remote, the remote peak 9.00 GiB of 9G with no build killed; 1,866 caught,
  347 unviable, 3 timeouts, 6 survivors; `.tmp/4_y/mutants_team_compiler/`): the timeouts
  are an atlas eight times too long (`kit_layout.rs:331` `/`→`*`), an empty range that
  never advances (`writer.rs:213` `+`→`*`) and an inverted poll (`gui_run.rs:52`), all
  explained; the survivors: `team_assets.rs` `centre_square`/`letterboxed` `>`→`>=`
  equivalent (4.11a's ruling, now in `.cargo/mutants.toml`), `deploy.rs` three error-kind
  guards (`sweep_run` 180, `probe_download` 406 and 409) missing tests → 4.y-fix2 (e), and
  `lib.rs:75` `settings_view`, a placeholder label no test reads until Phase 8 builds the
  settings view (left as a missing test for that step).
- **2026-10-09** — 4.31's verify criterion run at converge: Red rebuilt from the fixture's
  `old/`, every entry byte-identical to `red/` (the nested CPK's timestamp aside). 4.y-fix3's
  recon done and ruled: a cube-map DDS goes out as Red ships it on each engine (decision
  entry; a maintainer question on the Fox form). The `team_compiler` whole-crate mutation run
  started over `23f68ae`.
- **2026-10-09** — The `duck` reviews started: S7 and S8 (briefs `.tmp/duck_brief.md`,
  `.tmp/duck_brief_S8.md`, pointers only, the current code as the surface) launched in
  parallel at 11:11 and 11:15, both stopped at 11:16 by Astra's five-hour quota
  mid-review with no report; retried from 14:50 one at a time, the queue as in
  "Handover". 4.c-threshold done meanwhile: 0.7 kept (decision entry).
- **2026-10-09** — 4.y-fix2 done (sidekick, landed first time, two contradictions accepted:
  the `pipeline` survivors were the documented platform arms; two of the three `deploy.rs`
  guards are killed on Windows already). An old-layout export is `export_layout_old` alone;
  a `face.xml` `<dif>` beside a `face_diff.bin` is each engine's own. TC-ROOT-14 and
  TC-XML-11 proven. The census re-run follows the commit.
- **2026-10-09** — 4.y-fix3 done (sidekick, one rework round from its own contradiction: the
  FTEX writer and the compiler's cube test now read cube-ness by one predicate). A cube-map
  DDS is never decoded: as it is on PES 15-17, `dds_to_ftex`'s type-0xD cube map on PES
  18-21. TC-TEX-13 proven; acceptance 280 of 280. The verify re-run follows the commit.
- **2026-10-09** — The `duck` review S7 ran at 14:52 (`198711a`, six minutes): four concerns,
  three accepted (a panic when a linked face's `.model` boots convert without their
  `.mtl`; a boots or gloves link never combining with a linked face's parts or the split
  hands; the plan's tolerance rule for IR-derived skeletons, narrowed by a decision entry
  rather than built), one rejected (one Common model linked twice from one player).
  Rulings `.tmp/4_y/duck_rulings.md` S7.A1. S8's run at 15:00 stopped on the five-hour
  quota again ("try again at 4:44 PM"): the reset bought one review; retried hourly.
- **2026-10-09** — 4.y-fix4 done (sidekick, landed first time): S7's two code concerns.
  TC-MOD-58 to 60 proven; acceptance 283 of 283. One open issue filed (the deep pass's
  link answer without the hand-weighted set).
- **2026-10-09** — The `duck` review S8 ran at 16:45 (`f65c07c`, five minutes): four concerns,
  three accepted (a Common model converted with the player's local `.mtl` pointing its
  textures at Common; a signed-format DDS in `Common/` failing the whole Common textures
  task; an FTEX portrait keeping its single level), one rejected (S7.A1-4 again). Rulings
  `.tmp/4_y/duck_rulings.md` S8.A1. S9's run at 16:52 stopped on the quota ("try again at
  9:45 PM"): one review per reset so far.
- **2026-10-09** — 4.y-fix5 done (sidekick, landed first time): S8's three concerns.
  TC-MOD-61, TC-TEX-14 and TC-TEX-15 proven; acceptance 286 of 286.
- **2026-10-09** — The Astra reviews moved to the Devin CLI (`duck.py --reviewer astra-devin`,
  read-only, the user's three Devin logins rotated by `devin_account.py`; no Codex call until
  the user says otherwise). S9's three runs that evening all died on the accounts' daily
  usage (one review of this size exceeds a Pro plan's day; no export is written when a run
  dies mid-turn), so S9 stays queued. Step 4.y-fix6 opened for the VGL stream's PES 19
  crash on a DX10-header portrait (maintainer's report).
- **2026-10-09** — 4.y-fix6 done (sidekick, one rework round): a DX10-header DDS portrait
  re-headered, `portrait_header_rewritten`. TC-PRT-04 proven; acceptance 287 of 287. The
  maintainer traced the header to paint.net's "BC3 (sRGB, DX 10+)" save option. The reviews
  wait for the maintainer's instructions (every Devin daily quota spent on S9's attempts).
- **2026-10-09** — S9's Astra round ran on the user's own ChatGPT Plus through Codex (one
  whole-module review took 78% of the five-hour window and 12% of the weekly one): six
  concerns, five accepted. The duck skill's SWE-2 loop after each Astra round, skipped for
  S7 to S9 until the user asked, ran on S9 before the rework (two halves, one round each:
  13 concerns, 5 accepted, both loops ended); S7's and S8's are queued. Step 4.y-fix7
  opened.
- **2026-10-09** — 4.y-fix7 slice (a) done (sidekick, landed first time): the deep pass
  checks what `compile` reads. Eight scenarios proven; acceptance 295 of 295. S10 and S11
  ran on the brother's Plus through Codex (2 and 3 concerns, all accepted: step 4.y-fix8
  to open); Astra's windows then ran out, and by the maintainer's directive Phase 4
  closes on SWE-2 reviews alone, the rest of Astra queued as a re-audit (Handover).
- **2026-10-09** — S10's and S11's SWE-2 loops ran from the review worktree (`15270a5`): 3
  and 3 items, 1 and 2 accepted, both loops ended. Their reports, which `duck.py` overwrote
  with the reviewer's final reply, were recovered by resuming the sessions. Step 4.y-fix8
  opened for the two surfaces' eight accepted concerns.
- **2026-10-09** — Two maintainer answers landed: a shared face folder's `face.xml` rules
  its linking faces (step 4.y-fix9 opened), and on Fox each kit number gets a face package
  of its own once FoxDen loads one (an open issue; no phase does FoxDen work).
- **2026-10-09** — The maintainer's answer on pre-Fox per-kit model sets: completed against
  the export's kit numbers as texture sets are (the game skips an entry whose respelled
  model is missing); step 4.y-fix10 opened. `AGENTS.md`: an answer is open to discussion.
- **2026-10-09** — 4.y-fix8 done (sidekick, landed first time): S10's and S11's eight
  accepted concerns. TC-BIN-23, TC-BIN-24, TC-CMN-16 and TC-REF-14 proven. S8's SWE-2 loop
  ran (3 concerns, all accepted, the loop ended; step 4.y-fix11 to open with S7's). Step 5.0c
  queued: a Red behavior inventory before Phase 5 (maintainer).
- **2026-10-09** — The maintainer's answers on converted collars and `dummy_kit`: the
  templates ship PES 17's `uniform.mtl` (step 4.y-fix12), and the two in-game checks are
  the lead's, through the harness moved into `scripts/ingame/` (steps 4.y-harness,
  4.y-ingame).
- **2026-10-09** — The maintainer's answer on hidden Fox meshes: left out of a `.model`
  export (step 4.y-fix13). S7's SWE-2 loop ran (6 concerns, 5 accepted: a second round after
  the rework, step 4.y-fix11 with S8's three).
- **2026-10-09** — The maintainer's answers on `Common/refkit/`: every `Common/` subfolder is
  packed on PES 15-17 (Red's rule), a refs export's Common lays over the referee template's,
  and a referee's `fpc_off` appends the refkit body (Fox body later, an open issue); step
  4.y-fix14. The lead found PES 15's and 16's `uniform.mtl` without `uni_collar` (their stock
  collars use `uni_shirts` alone): a collar converted for them names `uni_shirts` alone, added
  to step 4.y-fix12. S13's SWE-2 loop ran (ruled separately).
- **2026-10-09** — 4.y-fix9 done (sidekick, landed first time): a shared face folder's
  `face.xml` is its linking faces' xml, a player's own beside the link
  `xml_shared_face_conflict`. TC-XML-14 and TC-XML-15 proven; acceptance 300 of 302.
- **2026-10-09** — S12's and S13's SWE-2 loops ran (6 and 5 items, 1 accepted each, both
  loops ended). Step 4.y-fix11 opened with S7's five and S8's three accepted concerns:
  a kit variant set spans the files a part is built from, and a shared boots or gloves
  folder's face file is not used (decisions).
- **2026-10-10** — S14's SWE-2 loop ran (7 items, 5 accepted: a second round after the
  rework, step 4.y-fix15). S15's first run ended at SWE-2's output-token limit before
  its report; rerun.
- **2026-10-10** — 4.y-ingame done by the lead on PES 17, the harness now driven by a
  virtual pad and OBS (no keyboard focus taken): the converted collar draws as the
  stock ones do; `dummy_kit` draws kit 2's texture in Edit mode and a file at its path
  is not read (plan corrected, decision). The maintainer's ATF bot joins the plan as an
  unscheduled tool.
- **2026-10-10** — The maintainer's answer on solid `.7z` exports: one at most an eighth of
  the budget stays decompressed from its check to its tasks while 1 GiB stays free (step
  4.y-7z; the maintainer's 512 MiB raised once the lead found 8192x8192 textures in a
  VGL26 export, whose decode charges up to 0.7 GB).
- **2026-10-10** — 4.y-fix10 done (sidekick): a pre-Fox per-kit model set is completed
  against the export's kit numbers, a member's xml's set too; TC-CMN-17 proven, the
  fix9 survivor caught. A lone variant stays an ordinary model (plan).
- **2026-10-10** — Astra's rounds on S12 and S13 ran on the maintainer's own ChatGPT
  Plus (one concern each, both accepted: fix11 items (11) and (12); both loops ended).
  Step 4.y-ingame2 started: the harness knows PES 19's and 21's test slots and the
  pad's triggers; on PES 21 a collar ID beyond the stock set loads when its model
  exists (201, no model, draws a plain neckline), and sock design A's visible part
  sits as drawn. The portrait header variants hovered one at a time in Kick Off's
  team sheet: the DX10 header crashes PES 21's BC3 portrait reader with either
  colour-space id, the legacy header and full-chain BC7 under DX10 draw; the cup's BC7
  portraits all have D's shape, so the QUESTIONS entry is closed.
- **2026-10-10** — S9's second Astra round ran on the brother's ChatGPT Plus (five
  concerns, all accepted after an evidence pass over the worktree: step 4.y-fix16; a
  third round after the rework).
- **2026-10-10** — The maintainer's subfolder rule landed in the plans (step 4.y-sub opened):
  any subfolder of a player folder is a player folder of its own merged into the root's,
  paths kept, a texture name resolving nearest first, only the root's markers and links
  counting; the reserved subfolders are gone and a refs export's AET-layout folder is
  refused with `player_layout_proto`, naming the upgrader (two decision entries;
  TC-MOD-65 to 68, TC-REF-18). The glossary gains AET and ATF (AATF, Aesthetics ATF)
  from the maintainer's definitions.
- **2026-10-10** — The maintainer's answers to the QUESTIONS entries landed: Clef's paid
  plan is on the second token, the free first one used first on purpose
  (`CONTRIBUTING.md` "Clef scan"); the 2026-10-05 retags get a review step (4.y-retag);
  `just parity` runs the two parity tests and `just release` arrives with 0.1.0; the
  shorts keep their layout (the 2026-10-04 decision confirmed); `kit_config`'s three
  model-gated warnings become one `kit_config_option_ignored` (step 4.y-kitwarn,
  decision), its two Infos wait for the in-game checks of 4.y-ingame2 (e) and
  `kit_collar_zero` for a collar census.
- **2026-10-10** — 4.y-fix11 done (sidekick): the fourteen review items landed, a kit
  variant set spanning every file a part is built from where the folder leaves variants
  out, a shared boots or gloves folder giving no face, a `kitN` reference looked up in
  the installed CPKs as a set, a WESYS-wrapped portrait unwrapped on Fox, TC-KIT-19
  proving the inverse table's stripe centres. Two contradictions accepted; four
  follow-ups to fix16; the right sock's table margin an issue.
- **2026-10-10** — 4.y-ingame2, the PES 17 half: the sock table's reference view on
  `/out/`'s standard bodies (every feature of design A where it was drawn, the lower
  sock included), the stock collar IDs drawing (the template's, 116, 131), shirt
  model 150 a sleeveless shirt on both engines and an undocumented sleeve value drawn
  without sleeves, no crash: `kit_config`'s two Infos stay unreported (decision). The
  `/a/` save hides standard bodies, which cost three runs before `/out/` was used.
- **2026-10-10** — 4.y-fix16 done (sidekick, one run): the deep pass reads a player's
  roles as planning does (`part_source_models`, shared by both), drops a beaten `Common/`
  model with its winner (`common_model_beaten_dropped`, decision), leaves an xml's
  unnamed models and `.mtl` files and Fox's unsearched Common `.mtl` files unread, skips a
  refs export's kits, logo, portraits and collars; `admitted` through `role_position`, the
  pre-Fox search sites' Common files, the `part_sources` survivor's test. One follow-up
  (`held_stems` under the marker) to the next rework brief. In game (4.y-ingame2 (b)): the
  pre-Fox marker's scene still unknown; the maintainer's PES 17 install draws no referee
  figure at all, and no marker, with the cup's refs CPK or the test slot's prop pair.
- **2026-10-10** — 4.y-fix12 done (sidekick, one run): a `.model` collar converts with the
  templates' `uniform.mtl` on both engines (`Templates::uniform_mtl()`,
  `ConvertedMaterials`, the Fox FMDL's samplers dropped), the pre-check moving a PES 15-17
  one onto the version's skeleton; `uni_shirts` alone on PES 15-16. TC-CMN-11 rewritten,
  TC-CMN-19 proven. The brief's fixture was not a collar (its material `modD_phone`): a
  real stock collar fixture goes to the next rework brief, with the converted Fox collar's
  shader and samplers to see on PES 21.
- **2026-10-10** — 4.y-ingame2 (c) done on PES 17: a 192x512 ten-level texture compiles
  with no finding and draws (the power-of-two rule stays Fox's), and the captain armband's
  `DCaptainmark` mesh tag changes nothing the game draws on a non-captain, in Edit mode or a
  match, so the IR keeps dropping tags (decision; the QUESTIONS entry removed).
- **2026-10-10** — 4.y-fix17 started (sidekick): `held_stems` under the marker, a texture
  the decoder refuses as the deep pass's finding (`texture_unreadable`, decision), the real
  stock collar fixture (the lead's, from PES 17's `dt35_win.cpk`). In game on PES 21: a
  `.model` collar converted by 4.y-fix12 draws nothing (the plain neckline of a collar with no
  model, for two stock PES 17 collars), so step 4.y-fix18 gives the FMDL the stock collars'
  material set (decision).
- **2026-10-10** — 4.y-fix17 done (sidekick, one run): the `.mtl` lookup counts a marked
  player's combined boots and gloves folders; a texture the probe refuses is
  `texture_unreadable` or `texture_codec_unsupported` on the file (a cube map skips the
  probe, the sidekick's catch); the collar tests convert the real stock collar. The catalog's
  "file alone" wording corrected (the place's disposition); a same-engine collar's
  `uni_collar` on PES 15-16 is a new issue.
- **2026-10-10** — 4.y-fix18 done (sidekick, one run): a `.model` collar converted for
  PES 18-21 takes the stock collars' Fox material set by name, every mesh two-sided (the
  sidekick's catch); in game on PES 21 the converted collar draws its fold where the
  converter's material set drew nothing (4.y-ingame2's manual line).
- **2026-10-10** — 4.y-fix13 done (sidekick, one run): the `.model` export skips a hidden
  Fox mesh with no finding; a model whose every mesh is hidden is
  `ConvertError::EveryMeshHidden`, which the compiler's four pre-Fox callers turn into
  nothing written with the Info `model_hidden_dropped` (the step respecified in the
  morning from the lead's FMDL scan: 61 hidden meshes, 38 all-hidden files, deliberate
  placeholders). Three all-hidden edge cases no file reaches filed as one issue.
- **2026-10-10** — S9's third Astra round ran on the user's own ChatGPT Plus (a fresh
  five-hour window, 85% of it): three concerns, two accepted (a link's search may fall back
  to a Common `.mtl` the pass never read; a non-model file in `Collars/` under the lenient
  check is a false `collar_id_invalid`), one rejected (a shared folder holding only a higher
  kit variant: a hand-built shape). Under five accepted, so the S9 Astra loop ends with the
  rework, step 4.y-fix19, briefed with 4.y-fix15's one open item ((1) and (3) landed with
  4.y-fix17 and 4.y-fix16).
- **2026-10-10** — S15's first Astra round ran on the brother's ChatGPT Plus (a fresh
  five-hour window, 48% of it): no concerns over run planning, the overrides and source
  pinning, so the S15 Astra loop ends at its first round; its SWE-2 round is still owed.
- **2026-10-10** — 4.y-fix15 and 4.y-fix19 done (sidekick, one run): a `.face` link counts
  as a face only when the linked folder gives one; the deep pass reads every direct Common
  `.mtl` once a link's search lands in `Common/`; a non-model file in `Collars/` gets no
  collar finding. S14's second SWE-2 round and its first Astra round run on the rework.
- **2026-10-10** — S14's first Astra round ran on the brother's ChatGPT Plus (41% of the
  window) on the 4.y-fix15 rework: one concern, accepted as a plan correction with no code
  (the blank face's `.fpkd` is generated empty, not the template `generic.fpkd` the plan
  named: Red's file is that empty package, the face parity row proves the bytes), so the
  S14 Astra loop ends at its first round; its second SWE-2 round is still owed.
- **2026-10-10** — 4.y-fix14 (a1) done (sidekick, one run and one rework round): every
  directory of `Common/` is a Common folder of its own on PES 15-17, packed at its own
  path, a member's `face.xml` naming a subfolder's file; the allowlist admits model
  content anywhere in `Common/`; a Fox subfolder's file is `file_not_used`. The step's
  (a) was split at briefing: the refs overlay and the template tree as an xml source are
  (a2), `fpc_off` (b).
- **2026-10-10** — 4.y-fix14 (a2) done (sidekick, one run, no rework): a refs export's
  file at a referee template tree path replaces the tree's in the refs CPK, and a
  referee's `face.xml` names a tree file the export does not hold (the template tree is
  an xml source in a refs export). The refs census re-run: the six exports that failed
  the allowlist now compile but for a real material defect of theirs.
- **2026-10-10** — 4.y-fix14 (b) done (sidekick, one run, no rework), the step closed: on
  PES 15-17 a referee folder holding `fpc_off` lists the referee template's refkit body
  after his own models, but for the boots or hands he has of his own.
- **2026-10-10** — 4.y-kitwarn done (sidekick, one run and one rework round): a sleeve
  or fit option the kit's shirt model ignores is one Warning, `kit_config_option_ignored`,
  on the config as emitted: the sidekick found that the FPC values set the shirt model to
  176, so a config written for 144 or 160 lost its option with no finding (decision).
- **2026-10-10** — 4.y-fix7 slice (b) done (sidekick, landed first time), the step closed:
  the deep pass's pairing code moved into `deep/pairings.rs`, no line changed.
- **2026-10-10** — 4.y-7z done (sidekick, two rework rounds): a solid `.7z` export under an
  eighth of the cap stays decompressed from its check to its tasks; the sidekick found
  that the floor alone lets a kept permit stall a later whole-archive acquire, so the
  rule also reserves the run's largest other archive (decision).
- **2026-10-10** — 4.y-sub slice (a) done (sidekick, one rework round): a player folder's
  subfolder files take their roles from their names at any depth, the reserved `face`,
  `boots`, `gloves` and `common` subfolders are gone, and a refs export still laid out that
  way is refused per folder with `player_layout_proto`; the sidekick found that a kept
  subfolder link would reach a role nobody resolved, so a `.common` link counts directly
  in the folder alone.
- **2026-10-10** — S17's Astra round 1 ruled (four concerns, all accepted): a Fox model
  whose every mesh is hidden is dropped by the deep pass instead of left out by each task,
  since the face listed an all-hidden Common model the Common task never wrote and a shared
  boots merge of none failed (decision); a package's names collide case-folded; the hand
  split's test compares faces, not counts. Step 4.y-fix20 opened.
- **2026-10-10** — 4.y-fix20 done (sidekick, one rework round): a Fox model whose every
  mesh is hidden is dropped by the deep pass on PES 15-17, with the `.common` link naming
  it; the sidekick found that `aesthetics_export` never took a player or shared folder's
  own file off on `DropFile` and that the cascade followed a dropped link (decision);
  a package's names collide case-folded; the hand split's tests compare faces.
- **2026-10-10** — S18's first Astra round (4.y-sub (a) and 4.y-fix20 together, the
  brother's ChatGPT Plus, 54% of a fresh five-hour window; the slot is off until its
  weekly reset, Friday): five concerns, five accepted (a nested `.mtl.common` link
  panics a lenient compile; the singletons below a subfolder, already slice (b1)'s;
  TC-MOD-64's two identical meshes; TC-MOD-67 and TC-MOD-68 claiming a strict run's
  survival, corrected; a dropped shared link still counted), so a second Astra round
  follows the rework, step 4.y-fix21 opened. The maintainer: SWE-2 runs free on Devin
  `work` whatever its usage, so the queued SWE-2 rounds run now: S7's and S14's second
  rounds and S17's first launched at 13:55 from the review worktree (`36f646f`); S12's
  and S13's post-Astra rounds, S15's two halves and S18's first follow.
- **2026-10-10** — 4.y-sub (b1) done (sidekick, two rework rounds, one survivor each): a subfolder's textures at their
  paths under the texture home on both engines, a texture name resolving nearest first
  (`texture_lookup`), each folder its own stem namespace, the singletons root-only in
  roles too; three contradictions accepted (the Fox platform folder sits in the
  texture's own directory, decision and in-game item (f); the shared writer resolves
  nearest first too; a nested singleton gets no role); TC-MOD-65 reworded to the rule
  and proven.
- **2026-10-10** — S7's second SWE-2 round and S17's first ran on Devin `work` (the
  maintainer: SWE-2 runs free there whatever its usage) from the review worktree
  (`36f646f`): S7 five concerns, four accepted (two already closed by 4.y-sub (b1), a
  textures-only folder's textures discarded with no finding (decision), stale doc text;
  the cross-folder kit-set edge case rejected), its loop ends; S17 three concerns, three
  accepted (the `.model` collar's conversion cannot skip material losses, a comment, the
  `prefox_face.rs` size for the design-health pass), its loop ends and S17 is closed.
  Step 4.y-fix21 takes items (4) to (7). S14's second SWE-2 round ran beside them: no
  defect (the subfolder singletons seen again, slice (b1)'s), its loop ends and S14 is
  closed; its note that the 4.12 step's "known, not fixed" list is stale (item 1 fixed
  by 4.y-fix1 (h), item 3's parenthetical wrong since 4.y-fix15) is for the collapse.
- **2026-10-10** — S12's and S13's post-Astra SWE-2 rounds and S15's two halves ran on
  Devin `work` from the review worktree (`36f646f`): S12 five concerns, two accepted (plan
  text: a codec the suite cannot encode is encoded whole; `resize`'s consumers), its loop
  ends and S12 is closed; S13 six concerns, five accepted (a left-out kit number's second
  file merged, validation's one-folder variant sets, a Common `.mtl`'s stems looked for in
  the folder first, a slice parameter, plan text), so a third round follows the rework;
  S15 half a three of three (a glTF-dropped folder's `fpc_on` counted, an order test, a
  visibility) and half b two of three (a file named `overrides` read as none, its test),
  both loops end. Step 4.y-fix22 opened. The retag (4.y-retag) is ready but its history
  rewrite was refused by the session's permission classifier: the maintainer runs it.
- **2026-10-10** — S18's first SWE-2 round ran on Devin `work` (`36f646f`): five concerns,
  four accepted (two stale comments and a finding's order into 4.y-fix22; a lenient
  check's nested shared-face file resolved by a member's xml, into slice (b2)); a third
  panic site of S18.A1-1's class rejected as the same defect (slice (b3)'s fix at the
  search covers it). Its loop ends; the second Astra round follows 4.y-fix21.
- **2026-10-10** — The maintainer withdrew the mirrored-path link rule of the same day: a
  link must mean the same wherever it sits (a player folder dropped into another keeps its
  links), and /hdg/'s per-kit `Common/kit1` to `kit5` folders, named by every model of that
  export, need a link from the root. A link's content naming the folder was rejected
  (Explorer would hide where links point); the maintainer's naming stands:
  `kitN.armor_bsm.dds.common`, a dotted prefix naming folders below `Common/`. Kit-token
  folders become variant sets, shared links count at any depth. Plan and decision updated;
  slice (b3) rewritten, (b4) added.
- **2026-10-10** — One Fox layout for a subfolder's texture (the maintainer: align if both
  work): before `sourceimages`, as Red packs a `Common/` subfolder and /hdg/'s models name;
  (b1)'s `sourceimages/<sub>/#windx11/` moves there in slice (b3). The `file_not_used` row
  names a no-package folder's textures (4.y-fix21 (4)).
- **2026-10-10** — 4.y-fix21 done (sidekick, landed first time, one contradiction
  accepted): TC-MOD-64 tells its meshes apart, a dropped shared link is no link, a folder
  planning no package plans no textures task and reports them `file_not_used`, a `.model`
  collar skips material losses and keeps its names; mutants-diff 0 missed.
- **2026-10-10** — 4.y-retag done: no commit needed a new type; the 76 subjects over 120
  characters cut, each old subject kept in its body; history from `b197139` rewritten and
  force-pushed (`bd5bc82`), gates green.
- **2026-10-10** — 4.y-fix22 done (sidekick, landed first time): a left-out kit number's
  files stay out wherever they sit, validation reads variant sets as compile does, a
  Common `.mtl`'s stems resolve in Common, a glTF-dropped folder's `fpc_on` does not count,
  an `overrides` file is an error, a no-package folder's texture links are not used, two
  comments and a finding's order. Its residuals are step 4.y-fix23.
- **2026-10-10** — The maintainer set the priority: members sort a player's textures into
  subfolders (/haha/), and /hdg/'s per-kit Common textures take one `kitN/` folder of links;
  composition is a low-priority goal. So links mirror `Common/`'s tree again (no dotted
  prefix), a model names a subfolder's texture by its path (`./kitN/`, an FMDL directory not
  starting with `/`), kit-token folders form sets in any folder, and a shared folder's tree
  follows the player folder's rules. Plan, decision and slices (b3)/(b4) revised.
- **2026-10-10** — In-game check (f) on PES 21: a player's subfolder texture draws both at
  (b1)'s `sourceimages/jessie/#windx11/` and at the chosen `jessie/sourceimages/#windx11/`;
  the QUESTIONS entry removed. Phase 16 gains a web guide before 1.0.0 (`565c802`).
- **2026-10-10** — A texture named by its path is one whose directory starts with `./`, FMDL
  and `.mtl` alike: a converted `.model` carries its `.mtl`'s game paths into the FMDL without a
  leading `/`. A census of the 6,897 FMDLs on the maintainer's machine (outside the studio's
  tree) found no relative texture directory but a bare `./` (49 files), which stays a name.
- **2026-10-10** — In-game item (b) answered by the maintainer: the pre-Fox referee marker
  shows under every official in every scene; the lead's runs missed the small referees.
- **2026-10-10** — S13's third SWE-2 round (after 4.y-fix22): three concerns, all accepted
  into 4.y-fix23 (a per-kit set across two shared folders unreported, a standalone shared
  folder's off-package model unreported, `combined_folders` reused); S13's loop ends.
- **2026-10-10** — The referee marker's routes confirmed by the maintainer: pre-Fox keeps the
  prop (a collar with the kit's material has no alpha test, so no transparent logo), Fox keeps
  collar 77 (`dt00_x64.cpk` loads over every CPK, so the Fox prop is reached only by writing
  it); `blue_port.md`'s rationale says so.
- **2026-10-10** — In-game item (d) answered by the maintainer: no Fox material samples a
  cube map (`resources/Materials.wikitext`, added), so a Fox `env.ftex` is a leftover the
  compiler keeps writing as Red does; the QUESTIONS entry removed.
- **2026-10-10** — Step 4.y-mattable opened (the maintainer): the material table and the
  converter's materials checked against `resources/Materials.wikitext`.
- **2026-10-10** — 4.y-sub (b2) landed: the pre-Fox face packs and lists a subfolder's files
  at their paths; a pre-Fox per-kit model set is one directory's (decision).
- **2026-10-10** — 4.y-sub (b3a) landed: a texture named by its path below its file's folder
  resolves there; a Fox player's subfolder texture sits before `sourceimages`.
- **2026-10-10** — 4.y-sub (b3b1) landed: links at any depth of a player folder; a `.common`
  model or material link stands for the `Common/` file at its own path.
- **2026-10-10** — Every CLI test of a tool cites a scenario (the maintainer's request; unit
  tests exempt): rule in `CONTRIBUTING.md`, step 4.y-scen.
- **2026-10-10** — 4.y-sub (b3b2) landed (SWE-2's first slice): a texture link below a
  player's subfolder stands for the `Common/` texture at its path.
- **2026-10-10** — 4.y-sub (b3b3) landed: a player's own texture or texture link wins over
  a combined shared folder's, as the Info `shared_texture_overridden`.
- **2026-10-10** — 4.y-sub (b3b4a) landed: on PES 15-17 a `Common/` `.mtl` resolves its
  textures from its own `Common/` directory, a player's texture shadowing nothing.
- **2026-10-10** — 4.y-sub (b3b4b) landed: on Fox a linked `Common/` subfolder model's
  textures travel with it and resolve beside it; S18.2's accepted findings with it.
- **2026-10-11** — 4.y-sub (b3b4c) landed: S18.A3's three fixes to the Fox Common parts;
  the step's slice plan restructured as a nested list.
- **2026-10-11** — S19.1 (SWE-2 on the subfolder and Common-texture plan text): 12 of 16
  accepted; plan fixes landed, (b3b4d) and (b3b4e) added to 4.y-sub.
- **2026-10-11** — S19.2 (SWE-2 round 2 on the plan text): 10 of 13 accepted; plan fixes
  landed, (b3b4d) widened to `kitN/` Common paths.
- **2026-10-11** — S19.3 (SWE-2 round 3 on the plan text): 11 of 13 accepted, all wording
  and scope fixes; landed.
- **2026-10-11** — S19.4 (SWE-2 round 4 on the plan text): 7 of 7 accepted, scope and
  wording; landed.
- **2026-10-11** — 4.y-sub (b3c1) landed: a shared folder's subfolders follow the player
  folder's rules (TC-MOD-68, TC-MOD-73, TC-MOD-74).
- **2026-10-11** — S19.5 (SWE-2 round 5 on the plan text): 5 of 8 accepted, wording and
  one scope word; landed. S20.5 (round 5 on the mutants scripts): 4 of 8 accepted, the
  loop ends; landed.
- **2026-10-11** — S19.6 to S19.8 (SWE-2 rounds 6-8 on the plan text, the `kitF` text
  included): 10 of 11, 5 of 6 and 4 of 4 accepted, the loop ends; landed. Astra S20.A1
  (the mutants scripts): 7 of 8 accepted, `scripts/mutants_test.py` added; S20.A2: 2 of 2,
  the loop ends; landed.
- **2026-10-11** — 4.y-sub (b3b4d) landed: a texture path into a `Common/` subfolder is
  checked on both engines; its `kitN/` half went to (b4).
- **2026-10-11** — 4.y-sub (b3c2a) landed: a shared folder's `.common` links are admitted
  and checked; on Fox its texture links count (TC-MOD-75 to TC-MOD-78, TC-TEX-17,
  TC-TEX-18); on PES 15-17 they are `file_not_used` (TC-MOD-77).
- **2026-10-11** — 4.y-sub (b3c2b) landed: on Fox a shared folder's model and `.mtl` links
  count, in its own output and combined into a player's (TC-MOD-79 to TC-MOD-82).
- **2026-10-11** — 4.y-sub (b4a) landed: a kit set's `kitF` fallback, written once as
  `kitN` on PES 18-21 and copied into each gap on PES 15-17, never under its own name
  (TC-CMN-04, TC-CMN-26 to TC-CMN-28).
- **2026-10-11** — 4.y-sub (b4b1) landed: a `.common` texture link naming a kit set, by a
  `kitN` stem or folder, finds the set's members in `Common/` (TC-MOD-72).
- **2026-10-11** — 4.y-sub (b4b2a) landed: a kit-token folder set (`kit1/armor_bsm`,
  `kit2/armor_bsm`) is completed as a stem set is, `Common/kit1/` and `Common/kit2/` as one
  (TC-CMN-21); test mode keeps a texture's subfolders.
- **2026-10-11** — 4.y-sub (b4b2b1) landed: a model's texture path into a `kitN/` folder,
  or a Common path into one, finds the kit set's members; a path naming `kitF` finds nothing
  (TC-CMN-29, TC-CMN-30).
- **2026-10-11** — 4.y-sub (b4b2b2) landed: a `.common` link to a kit set is dropped only
  when every member it reaches is, a dropped member a gap the completion fills (TC-CMN-31).
- **2026-10-11** — 4.y-sub done: (c) and (d) found done by earlier slices; a new-layout
  copy of the HAHA census export compiles for PES 17 with 22 of 23 players kept, the 23rd
  dropped by its own empty-named material.
- **2026-10-11** — 4.y-fix23 done: on PES 18-21 a shared boots or gloves folder's model link
  is named by the folder's suffix rule (TC-MOD-83); its pre-Fox item is an open issue.
- **2026-10-11** — 4.y-mattable done: the material tables checked row by row against the cup
  wiki and a census of every FMDL and `.mtl` on the machine; no code change, three family
  defaults left for Phase 7, one pre-Fox issue.
- **2026-10-11** — 4.y-simp done: the converge's simplification pass; a
  `shared_texture_conflict` leaves the player folder out, a Common texture only conversion
  refuses fails its directory's task, an undecided root reports the parse's findings alone,
  `multiple_ref_exports` lost its run summary; net about -270 lines.
- **2026-10-11** — 4.y-scen done: every `team_compiler` CLI test cites the scenario it proves;
  363 scenarios (20 new, about 60 widened), all proven; six Astra rounds on the text, 56
  concerns, all accepted.
- **2026-10-11** — 4.z-rewrite, first part: Phase 4's plan sections in the present tense, the run
  driver shapes copied from the code, this log moved out of the worklog. 5.0 done: no savefile
  change in the 4ccEditor fork since `4a95b7c`; reached `Autumn_2026_AATF` `f66e8b4`; VGL27 at `VGL27.B`.
- **2026-10-11** — 5.0c done: Red's behavior inventory at `e12aa01`, 442 rows, 25 gaps. The
  Fox compiler's four became step 5.0e (BC7 portraits for PES 15-18, a texture-size census, kit
  texture paths, `.txt` notes); 14 pre-Fox gaps deferred as one issue; nine plan passages on Red corrected.
- **2026-10-11** — 5.0e (2): texture census of the machine's exports (12,857 distinct textures):
  15 mipped one-side-non-power-of-two textures and 11 kit textures over 2048 in Fox exports, so
  Studio's stricter rules wait on an in-game check (`QUESTIONS.md`) before any relaxing.
- **2026-10-11** — Match tracker: measured PES 2019 and PES 2021 layouts from an independent SEN:P-AI
  rebuild recorded in the plan; on PES 2017, the 15-minute buckets and the heatmap grid confirmed live
  and the unofficial fork's pointer chain refuted; version scope asked in `QUESTIONS.md`.
