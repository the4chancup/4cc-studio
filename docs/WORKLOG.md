# 4cc Studio — Worklog

Where the implementation is and what comes next. The plan (`docs/plans/`) holds the *why*;
`DECISIONS.md` holds choices made where the plan was silent; this file holds *where we are*.
Never duplicate rationale here — link to the plan section instead. Procedure for using this file
is in `AGENTS.md` ("Working documents").

---

## Current status

**Phase:** 2 (Library crates). Done: 2.1 `wezlib`, 2.2 `cpk`, 2.3 `fpk`, 2.4 `ftex`, 2.5 `dds_convert` (CPU),
2.6 `fmdl` (format, model, ops, check), 2.7 `pes_model` (format, mtl, model, ops, check), 2.8 `uniparam`, 2.9 `fox2`, 2.10 `archives`, 2.11 `fpc`, 2.12 `teams_list`, 2.13 `kit_config`, 2.14 `color_tools`, 2.15 `elevation`. Review
rounds A and B (2026-09-13) closed: 2.5c, 2.12b, 2.13b done.
**In progress:** 2.17 `pes_savefile`, 2.18 `python_bindings`, 2.19 Phase verification done; 2.20
Converge in progress (a-h done, 2.17i done; 2.20i `pes_savefile` started: census triaged,
text-field rules decided, fix and whole-crate mutation run next). Review round C (2026-09-19) closed as
2.19a; its leftovers are listed under 2.20. 2.5b (GPU BC7) deferred to Phase 4 (user decision
2026-09-21). Release target (2026-09-28): 0.1.0 after Phase 8; phase order 1–6, 8, 0.1.0, 7,
9–16 (`core/development_plan.md` "Releases").
**Blocked on:** nothing yet. **Hard gate at the end of Phase 3:** step 4.0 (the maintainer's
in-game appearance-fallback test) must be done before any agent itemizes Phase 4 or writes
anything for Phase 4, 5 or 6. Everything up to and including Phase 3's close may proceed.

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
- `just mutants-diff` reuses `mutants.out/`: it wipes a whole-crate run's results. Copy
  `mutants.out/` aside (for example to `.tmp/mutants_<step>_whole/`) before the first diff run.
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
below to this one row; the step-level detail stays in git history. Tool phases (3–6, 8–15, 17, 19) also
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

Spec: `docs/plans/core/development_plan.md` "Phase 2", `docs/plans/libs/README.md`, `model_conversion/README.md`,
`pes_savefile/README.md`. Leaf crates first, dependents after; `python_bindings` last.

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
- [x] 2.5b `dds_convert` GPU BC7 — deferred to Phase 4 by user decision 2026-09-21 (decision
  entry; `core/development_plan.md` "Phase 2" and "Phase 4"): built with the texture step that
  consumes it. The step text lives under Phase 4 below
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
- [x] 2.6c-3a `fmdl::ops::antiblur` — done (sidekick): encode/decode over `Model`, fuzzblock and
  uvscroll materials, idempotent, header round trip through `to_file`. 6 tests
- [x] 2.6c-3b `fmdl::ops::paths` — done (sidekick): `texture_paths` / `rewrite_texture_paths` on
  `FmdlFile`: only the string table is rebuilt (new strings appended and de-duplicated, extension
  tail kept), a no-op edit leaves Konami bytes identical. 6 tests
- [x] 2.6c-3c `fmdl::ops::vertex_enc` — done (sidekick): owner maps from the ordering convention,
  encode reorders/collapses loops with byte keys built through the codec. 6 tests
- [x] 2.6c-3d `fmdl::ops::split` — done (sidekick): encode over bone subtrees with principal-axis
  fragments, decode by stored encoding and Nth occurrence; synthetic 40-bone and 90k-vertex grids
  split into 2 and 7 components and round-trip; header round trip through `to_file`. Deviations
  from the legacy add-on recorded in the decision entry. 10 tests, 3.9 s
- [x] 2.6c-3e `fmdl::ops::merge` — done (sidekick): bones unioned by name with position/parent
  conflict detection, materials by name with conflict detection, meshes/groups concatenated,
  bone matrices unioned when every boned part carries them. 7 tests
- [x] 2.6d `fmdl::check` — done (sidekick): nine findings over `Model` (limits, face indices, bone
  slots, weights, empty/unassigned meshes, unused materials, duplicate bone names); every fixture
  clean. 7 tests. `fmdl` totals 76 tests; converge at 2.20 revisits `from_file` length and the
  two extension-header plan gaps
- [x] 2.7 census `pes_model` — done (lead): `.model` layout measured on all 2610 Konami files of a
  PES 2017 install (`libs/format_crates.md` "`pes_model::format`: the `.model` container and its sections"):
  LOD tables, annotation types 1/2/7/10 with their section-3 records, the version-17 layout, the
  empty-array offset-0 quirk, sections 8/9/10 empty everywhere. Six more fixtures, one per
  variant (`tests/fixtures/README.md`), the LOD one at 148 KB
- [x] 2.7a-1 `pes_model::format` container and record-array reader — done (sidekick):
  `ModelContainer` (header words, eleven sections in file order) byte-identical on all twelve
  `.model` fixtures, wrapped ones compared unwrapped; constant header words validated;
  `RecordArray` reader with checked arithmetic. 8 tests
- [x] 2.7a-2 `pes_model::format` typed layer — done (sidekick, two rework rounds: a `macro_rules!`,
  duplicated pointer checks and unchecked offset sums): `PreFoxModel` over the container (bones,
  groups, materials, annotation strings and records, geometries with raw field data, faces and
  LOD ranges, meshes with resolved indices and editor data, bounds, LOD record) and its `write`
  in the add-on layout; `read(write(m)) == m` on all twelve fixtures, version 17 included; every
  census expectation literal. Editor-data item decode is unverified against a real file (none
  exists). 15 tests. Reviewer checkpoint (b) for the new `pub` surface is batched with 2.7b
- [x] 2.7b `pes_model::format::vertex` — done (sidekick): `MeshVertices` (float weights two to
  four wide, width remembered), in-place decode/encode byte-identical on every geometry of the
  twelve fixtures, `to_fields` in Konami's field order (equal to every fixture's own order),
  `FaceStream::faces`/`level`/`from_faces` over the LOD table. 19 tests
- [x] 2.7 review of `pes_model::format` (checkpoint b, `gpt-astra-high`) — done: seven concerns,
  all accepted and fixed (sidekick): checked arithmetic and no allocation before validation,
  first section pinned at 80 and `ModelContainer::new` enforcing one section per kind,
  `to_fields` rejecting weights beyond the stored width, the writer always emitting version 19
  (decision logged), no partial edit on a failed encode, editor item kind/value agreement, LOD
  ranges required to partition the stream. 23 tests
- [x] 2.7c `pes_model::model` — done (sidekick): `Model`/`Mesh` with decoded vertices, level-0
  faces plus lower LODs, per-mesh bone groups, mesh name / extension headers / Konami tags split
  by annotation kind, model-level headers; `from_file(to_file(m)) == m` and the full byte trip on
  all twelve fixtures. Loose-vertex repair (reference importer) deliberately not reproduced;
  converge question. 28 tests
- [x] 2.7d-1 `pes_model::ops::vertex_enc` — done (sidekick): the `fmdl` convention on `.model`
  (per-mesh `vertex-loop-preservation` header, key with the stored weight width, bitangent in
  the encoding, every LOD level remapped); the card heads carry the marker with no loops. 7 tests
- [x] 2.7e `pes_model::format::mtl` — done (lead census over 945 Konami files, sidekick code):
  typed `MaterialSet` over `roxmltree`, hand writer under a detected per-file style; the writer
  spec measured at 909/945 byte-identical before coding; parity tested on the four regular
  fixtures, semantic round trip on all seven, unknown elements/attributes are errors. 20 tests
- [x] 2.7d-2 `pes_model::ops::paths` — done (sidekick): `texture_paths` / `rewrite_texture_paths`
  over the `.mtl` samplers, paths split at the last `/`; a no-op edit leaves the parity fixtures
  byte-identical. 4 tests
- [x] 2.7d-3 `pes_model::ops::split` — done (sidekick): the `fmdl` port with `.model` limits,
  caller-supplied parents, `Split-Mesh: N` headers; synthetic 70-bone grid → 11 components,
  90k-vertex grid → 7; round trips, file round trip, two-source numbering. Departures logged.
  12 tests, suite 4.2 s
- [x] 2.7f `pes_model::check` — done (sidekick): ten `model_*` rules over `Model` (limits, indices,
  slots, weights within 1e-3, degenerate faces, empty meshes, unused materials, duplicate bones,
  LOD record), six `mtl_*` rules over `MaterialSet`, `check_bundle` adds `model_material_undefined`;
  every `.model` fixture clean, `.mtl` fixtures Info-only (states missing on sampler-only Konami
  materials, `alphablend 1 + zwrite 1` on hair and glasses). 10 tests
- [x] 2.7d-4 `pes_model::ops::merge` — done (home decided by the user: native over `Model` +
  `MaterialSet`, the IR route removed from six plan passages; sidekick code): bones by name
  within a measured `1e-4` (the exact rule could not merge cap with collar; census of 2606 files
  in the decision entry), `.mtl` materials by name with equal definitions, meshes/headers
  concatenated and deduplicated, bounds and LOD record rebuilt; identity on nine fixture
  bundles. 10 tests; `pes_model` totals 91, workspace 272
- [x] 2.8 `uniparam` — done (sidekick): WESYS-unwrapping read, sorted writer; Konami PES21
  container (2174 entries) and a reference-writer golden; 4 tests
- [x] 2.9 `fox2` — done (lead: census of 87 files inside 108 FPKDs, four fixtures with reference
  goldens, plan section; sidekick code in three handoffs): `hash` (CityHash64 1.0.3 port,
  `hash_string`, `Dictionary`), `text` (C# round-trip float text, 36-case golden), `file`
  (`Fox2File` read/write byte-identical on the four Konami and four compiled fixtures, every
  constant word checked, the reference writer's slack tolerated on read and dropped),
  `values` (24 data types, the nine without a fixture tested synthetically), `xml` (decompile
  equal to the reference's XML on all four, compile equal to its binaries, fixed point).
  Checkpoint (b) review (`gpt-astra-high`): seven concerns, all accepted and fixed in one
  handoff (integer digits trimmed by the float text, a 48-byte guard that rejected empty
  properties, padding never validated, unresolved names lost through XML, whitespace not
  escaped, lenient bools, `0x` keys). Padding measured zero on all 87 files. 24 tests;
  workspace 310
- [x] 2.10 `archives` — done (lead: plan section, `sevenz-rust2` + `zip` without default
  features, six fixtures from one sample tree; sidekick code): `Archive<R: Read + Seek>` with
  `zip`/`seven_z`/`entries`/`read` and a native `open`; the same six entries and bytes from
  7-Zip `.7z`, 7-Zip `.zip` (OEM-code-page names), stored `.zip` and PowerShell `.zip`;
  encrypted archives refused (an encrypted 7z header surfaces as the missing AES codec, mapped
  to `Encrypted`); names normalized and zip-slip names rejected. 6 tests; workspace 316
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
- [x] 2.14 `color_tools` (extraction only) — done (lead: regions measured on the template
  sheet, thresholds and a trim fallback from a 134-kit harness, decision logged; sidekick
  code): `kit::extract_kit_colors` and `dominant_colors` over RGBA pixels, no dependency.
  9 synthetic tests; workspace 325. Widget and icon drawing wait for Phase 8
- [x] 2.15 `elevation` — done (lead: surface pinned in the plan, `windows` + `libc` target-gated;
  sidekick code): `is_elevated` (token query / euid), `relaunch_elevated` (`runas`, UAC
  decline mapped), `is_access_denied`, argument quoting verified against `CommandLineToArgvW`
  itself; wasm32 compiles to "not elevated". The relaunch is a manual check at the first tool
  phase that needs it. 4 tests; workspace 329
- [x] 2.16a `model_convert` skeletons — done (lead: plan rewritten after a census of the six
  `body.skl` and the legacy tables, `render_parents.rs` and `fold.rs` data authored, four
  decisions; sidekick code): `affine.rs` (3x4 transform, f64 inverse), `skeletons/` (embedded
  `.skl` parsed once, `PesBone`/`Skeleton`/`VersionSkeletons`, `render_parent`, `fold_target`);
  every fold chain lands on every version. Plan error fixed: hand bones are `skh_`, not `skf_`.
  12 tests
- [x] 2.16b `model_convert::ir` + `materials` types — done (sidekick): the plan's two code
  blocks pasted with docs, `validate` with thirteen invariants (weighted bone slots only: Konami
  files leave stale indices in unweighted slots), one failing case each. 26 tests
- [x] 2.16c `model_convert::formats::fmdl` + `loss.rs` — done (sidekick, one rework after the
  brief's "vertex-loop encode is a no-op" premise failed: it reorders 198/204 highneck vertices;
  decision logged): `fmdl_to_ir` (split/anti-blur decoded, bind pose from the SKL else PES21's
  tables, per-mesh flags lifted to split materials, weights `/255`, out-of-group slots dropped
  with a finding) and `ir_to_fmdl` (Fox resolution, total-preserving weight quantization, derived
  positions/boxes, encoders in the legacy order anti-blur → vertex loops → split, SKL only for a
  bone PES21 lacks: the audience fixture's `sk_root_hip`). Round trip equals the decoded input
  with the same encoders applied on all three fixtures, no findings; IR is a fixed point after
  one encode. Lead review: dummy maps only for a material without a `fox` table; family defaults
  only without a native table in both `to_fox`/`to_prefox`. 10 tests; crate 51
- [x] 2.16d `model_convert::formats::pes_model` — done (sidekick; three fixture contradictions
  ruled by the lead and written into the plan: indices-only meshes get `[1,0,0,0]` weights,
  `.mtl` entries regroup to Konami's majority order (census: 1217/1265), `transparent` owns
  `alphablend` alone over a stored table): `model_to_ir` (split decoded, parent-first bone
  order from the render hierarchy with a 44/2606 census behind it, matrices inverted, `.mtl`
  verbatim into `prefox`, LODs/tags/order/flags reported as `native_field_dropped`) and
  `ir_to_model` (pre-Fox resolution, matrices re-inverted, bounds recomputed, vertex loops then
  split). Round trip on all four pairs: matrices within 1e-5, everything else `==`; Fox→pre-Fox
  smoke on highneck. 11 tests; crate 62
- [x] 2.16e `model_convert::materials` logic — done (sidekick, one rework: native sampler
  settings verbatim, plain branches): `family` (Fox substring rules, pre-Fox exact names),
  `to_fox` (family defaults table, role <-> sampler, `resolve` with the booleans owning their
  bits), `to_prefox` (`Basic_*` ladder, state sets in the fixed order, role <-> sampler with
  attributes, `resolve`). Census of 1983 Konami FMDLs: `fox3ddf_blin` meshes carry alpha
  128/160/32/0 in near-equal shares, `constant_srgb_ndr_solid` mostly (16, 4) where the plan's
  default is (16, 5); the plan's table kept, see the open question. 41 tests
- [x] 2.16f `model_convert::skeletons::retarget` — done (sidekick, clean first pass): standard
  bones the target lacks fold through the table chain (nearest body bone by position as the
  reported fallback), the simplifier's group/slot remap with weight merge, then every surviving
  standard bone re-binds through `B_target · B_source⁻¹` above 1e-3 (positions blended,
  normals/tangents/bitangents rotated and renormalized; unmoved bones and vertices untouched
  bit for bit). PES17→PES15 folds exactly the legacy six and moves 18 bones; PES19→PES16 folds
  46, none by position; PES19→PES21 the ten `dsk_pos_*`; PES21→PES21 is `==`. 9 tests; crate 71
- [x] 2.16g `model_convert::ops::hand_split` — done (sidekick, clean first pass): `hand_of`,
  `has_hand_weights` (weights, not names), `split_by_skeleton_group` (select → grow once →
  separate per hand; parts re-index vertices and copy every column; groups survive on a listed
  mesh or as an ancestor; unweighted bones pruned through `ir::{rebuild_bone_list,
  remap_bone_group}`, shared with `retarget`). Matches Blender 5.2.1's own result on the
  connected-wrist fixture (10 vertices / 9 faces each side, face sets equal). 6 tests; crate 77
- [x] 2.16h `model_convert` routing — done (sidekick; one ruling: the pre-Fox output keeps the
  normal/specular maps the legacy dropped for want of texture files): `convert(bundle,
  PesVersion)` with `needs_conversion` as the native pre-check (bone names and poses against the
  target's tables, no geometry), import → `retarget` → export; the Fox `static` bone for
  unskinned meshes. Fox oral → PES16 equals the 19to16 converter's output on bones, geometry,
  states and the diffuse sampler; the shader/sampler set difference is explained in the fixture
  README. 6 tests; crate 83
- [x] 2.16i checkpoint (b) review of the new `pub` surface — done: seven concerns, six
  accepted and fixed in one rework (exporters recover vertex-loop owners from the IR order
  with the unflagged per-mesh decode; `remap_bone_group` never indexes with an unweighted
  slot; unique names for flag-split materials; `native_field_dropped` for `bone_matrices`, a
  disagreeing SKL parent, non-unit normal/tangent `w`; weights validated finite and in 0..=1;
  hand split selects over topological vertices, with a seam test that discriminates), one
  rejected (the `Metal` environment fallback is the compiler's texture step; worklog issue 5).
  10 tests; crate 93; workspace 422
- [x] 2.17a `pes_savefile::container` — done (lead: container census over nine real saves of
  15/16/17/18/19/21, per-version slice fixtures plus zlib payloads, plan corrections; sidekick
  code): `SaveContainer`/`Scheme`/`MasterKey`, MT19937 with the published known answers, the
  keyed and PES 15 schemes; every head fixture opens with exactly its own key, `to_bytes`
  reproduces the real prefixes byte for byte, round trips under all seven keys. PES 20 has no
  save on the machine: key and header size untested. 14 tests
- [x] 2.17b `pes_savefile::schema` + `codec` + `model::player` — done (lead: tables generated by
  `scripts/derive_savefile_schema.py` from the reference read walks and checked on every real
  payload, plan's "18 shares 17's layout" withdrawn; sidekick code, one rework: `runs()` iterator,
  `Missing` instead of a silent 0 for an unset gated field): schema types, `schema_for`, the bit
  engine, `read_player`/`read_player_into`/`write_player` with the text rules, `PlayerEntry`.
  Every player and appearance record of six saves round-trips byte-identical; census literals
  (counts, ages with PES 18's nine 13/14-year-olds, abilities 40..99 bar one PES 16 place kick of
  100, the two non-UTF-8 shirt names). 12 tests; crate 26
- [x] 2.17c `pes_savefile::model::{team,tactics}` + team/roster/tactics codec — done (sidekick;
  one rework: `tactics_runs` as plain loops, `codec/` split into `mod`/`player`/`team`/`tests`):
  `TeamEntry` merging the three records, `TeamTactics` with presets/formations/instructions,
  byte booleans checked (`NotBoolean`), roster numbers `u16` (PES 19+ store values to 999). Every
  team, roster and tactics record of six saves round-trips byte-identical; the three sections list
  the same ids; team 701/100 and tactics literals. 5 tests; crate 31
- [x] 2.17d `file.rs` `EditFile` + `model::names` + `discovery.rs` — done (lead: colour-code rule
  measured over 346 decorated names, API blocks in the plan; sidekick code, one rework:
  `side_section` helper instead of a string dispatch): `from_bytes`/`to_bytes(salt)` pure,
  `load`/`save` (`.bak`, sha2 salt) native, players/teams joined by id with strict layout checks;
  `display_name` (eight bytes after `\x11c`, `\x11d` reset); `save_layout`/`discover_savefiles_in`.
  Every payload fixture lossless through `EditFile`; an edit changes only its records; 60/103/183
  decorated names on 16/19/21. Lead probe (not committed): all nine real saves on the machine,
  a second PES 16 save (4431 players, 273 teams) included, load and write back byte-identical
  with their own salt, logo and serial included. 6 tests; crate 37
- [x] 2.17e-1 the ingame-face run (`pes_savefile/model.md` "Player settings model", the run and
  `IngameFaceField`; `codec.md` `RecordSchema.ingame_face`): `schema/ingame_face.rs`,
  `model/ingame_face.rs`, the four fields removed from the tables and `PlayerField` by the
  generator, `read_player_into`/`write_player` carrying the run; `fpc::custom_skin_available`
  → verify: every player and appearance record of the six payload fixtures still round-trips
  byte-identical; the skin/iris/gloves census literals of 2.17b hold through the accessors; the
  run of a PES 16 record is 50 bytes and of a PES 15 record 46 — done (sidekick, one rework: `get`/`set` return
  `NoIngameFaceRun` instead of panicking on an unread run; `ALL` + exhaustive-row test): `ByteRun`,
  `schema/ingame_face.rs`, `model/ingame_face.rs`, generator emits the run and drops the four
  fields; 45 crate tests, census literals unchanged; `mutants-diff` 33 caught / 0 survived
- [x] 2.17e-2 `settings_toml.rs`: `SettingKey` table, `PlayerSettings::{parse, from_player, apply,
  to_toml, update_toml}`, the ownership classification and completeness test → verify:
  `to_toml` of `from_player(p)` parses back to an equal `PlayerSettings` for a player of each
  fixture; `to_toml(&Default)` equals the plan block's key set, order and comments (every key
  commented); parse rejects an unknown key, a compiler-owned key, an out-of-range value and an
  unknown label, each naming the key; `apply` then `write_player` changes only the run bits and
  fields the settings named (byte diff against the untouched record)
- [x] 2.17e-3 `ops/fpc.rs` (`apply`, `strip_style`, `is_fpc_player` per the plan block) → verify:
  the hide preset applied to a fixture player yields the FPC.wikitext values through the model;
  `Custom` skin on PES 19 is `CustomSkinUnavailable`; `strip_style` of a player with inners
  trips `fpc_inners_break_hiding` through `fpc::check` — done (sidekick, two review reworks: no panic
  on a non-table TOML position or an unrepresentable stored value, key-table-driven unknown-key
  sweep; five mutation survivors killed). 2.17e-2 landed as 2a (model half, `51dea0e`) and 2b
  (parse/template/update, `eef7e6d`); crate 71 tests, `mutants-diff` 48/48 caught
- [x] 2.17e-4 checkpoint (b) review of the 2.17e surface (`gpt-astra-high`) — done: six concerns,
  five accepted and fixed in one rework (behavioral completeness test over real writes,
  all-or-nothing `apply` in both modules, `set`/`to_toml`/`update_toml` return `OutOfRange`
  instead of panicking, NUL in a name refused, the copy test now proves the undecoded bits are
  carried not copied), one accepted as a doc/plan sharpening (`is_fpc_player` classifies what
  the save shows; decision entry). Crate 75 tests; workspace gates green
- [x] 2.17f `convert.rs` cross-version conversion (`pes_savefile/operations.md` "Cross-version
  player conversion", "What the Rust module is"), in two slices — done (`da6e60b`, `17d011c` plus
  the review rework); checkpoint (b) review (`gpt-astra-high`): five concerns, four accepted and
  fixed (shirt name cut by chars, not UTF-8 bytes, since it is single-byte text on disk; the skill
  test asserts literal drop lists for PES 15 and the PES 16 array path instead of re-asserting
  `has`; the caps table has a literal golden test; `copy_from` is `pub(crate)`), one resolved in
  the plan (a version-wide non-carry, a gated field the target lacks or the tail a 16+ template
  keeps, is not a note; decision entry). Crate 93 tests; `mutants-diff` 33 + 2 + 6 caught, 0
  survived:
  - [x] 2.17f-1 `model/playstyle.rs` + `schema/playstyle.rs` (`decode`/`encode`, the four lists,
    `CodecError::UnknownPlayingStyle`), `schema/limits.rs` (`face_type_cap`), `settings_toml`
    face-key widest ranges derived from it → verify: the lead's golden test reproduces the
    reference editor's twelve conversion arrays through `encode(to, decode(from, i))`; every
    player of every fixture decodes; ≥ 95 % of registered goalkeepers with a style decode to a
    goalkeeper style on every fixture; the `settings.toml` template is byte-identical to before
  - [x] 2.17f-2 `convert.rs` (`convert_player`, `ConvertNote`, `ConvertError`),
    `IngameFace::copy_from`, `RecordSchema::has` → verify: 19 → 16 and 16 → 21 of fixture players
    match the converters' appearance writes field by field (copy-through, caps, skin) bar their
    compile-policy rewrites; 16 → 15 drops the run's last four bytes and 15 → 16 keeps the
    target's; a rejected conversion leaves the target unchanged; the converted target
    `write_player`s into the target schema without error
- [x] 2.17g `ops/{transplant,fingerprint,compare}` (`pes_savefile/operations.md` "Save-to-save
  operations") — done in two slices (`74136aa`, `7a7ab4a` plus the review rework): the lead's
  golden tests hold the scripts' byte rules as literal offsets and pass on every fixture player
  (transplant slice rule, the compare script's fifteen fields and hash, its diff outcome on a
  transplanted twin); `transplant_player`/`transplant`/`parse_selection`, `FaceHash`/`face_hash`,
  `compare_players`/`PlayerDiff`/`DiffScope`/`compare`. Checkpoint (b) review (`gpt-astra-high`):
  six concerns, all verified and fixed (the by-value `appearance` schema change reverted, it
  contradicted the codec plan block and the generator; duplicate ids resolve first-wins as
  `EditFile::player` does; the 15/16 appearance record's `Id` is not a second row; three vacuous
  tests strengthened, `scope()` pinned mechanically to what `transplant_player` moves). Crate 113
  tests; `mutants-diff` 31 caught + 3 boundary survivors killed, then 21/21
- [x] 2.17h `interchange/{team_toml,legacy,texport}` (`pes_savefile/operations.md` "Interchange
  formats", rewritten from measurement on the real files: three 17 / four 18 / eleven 19 / five 21
  texports, two `.4ccs`; decision entry 2026-09-20). Texport write is a manual game check.
  Slices, vertical first (real input in, real output out):
  - [x] 2.17h-0 lead: fixtures (`pes17_texport_*` slices, `pes18/19/21_texport.ted`,
    `pes19_squad.4ccs`, synthesized `pes19_tactics.4cct`) via
    `scripts/provenance/fixtures/interchange_fixtures.py`; golden tests holding the `.4ccs`
    ctypes offsets and the texport layout literals (`3575b19`)
  - [x] 2.17h-1 `schema/texport.rs` + `interchange/texport.rs` (`8385437`): round trip
    byte-identical on the 17/18/19/21 fixtures, `new` reproduces the fixtures' header/coach/tail
    bytes, a synthesized PES 16 texport pins the player+appearance stride; PES 20's size derived
    from its own records (decision). `mutants-diff` 137 caught, 2 equivalent (documented in the
    2.17h-1 rework brief: integer division absorbs the tail, a slice length `record_id` ignores)
  - [x] 2.17h-2 `model/instruction.rs` + `schema/instruction.rs` + `team_toml/{mod,labels,team}.rs`
    (`2d9365c`): the 20/21 fixtures store a 17th instruction value (0x10, `Anchoring`; plan and
    golden updated); team/tactics parse/emit/`from_team`/`apply` with schema-gated notes,
    multi-line formations. `mutants-diff` 99 caught, 10 survivors: 9 missing tests added in
    2.17h-3 (`TacticsSchema::has`/`has_preset`, `shirt_name_from`), 1 equivalent (the two colour
    gates co-vary). `team.rs` is ~1600 non-test lines of per-key three-pass code; accepted,
    the player half was made table-driven instead (design-health input for converge)
  - [x] 2.17h-3 `team_toml/{player,player_keys}.rs` (`ceb9b6f`): `PlayerKey` table drives
    parse/emit/apply; `settings_toml` parser/emitter shared through `pub(crate)` seams; stored
    ranges, not editor ranges (decision: real saves hold face types past the caps); fixture
    round trips exhaustive at the model level, TOML text sampled (first/last/densest team per
    version) after a 104 s test was cut to 9 s and `from_team` sped up ~9x (`player_field_set`)
  - [x] 2.17h-4 `interchange/legacy.rs` (`1ae6e16`): both goldens green; `from_team` pool rule
    (empty = team-only document, partial = `PlayerMissing`)
  - [x] 2.17h-5 checkpoint (b) review (`gpt-astra-high`, seven concerns, all verified and fixed,
    `28e2eab`): the 15-17 texport import path (implied roster, `to_team_toml`), source-version
    skill gating on up-conversion, face types reset to 0 as 2.17f, `TextTooLong` before write,
    legacy tactics gated by the exporting version, stored-range mode bounded by bit width with
    label-less values as integers, two hostile-input panics, a test oracle built from the
    post-apply state. Plus the 16 2.17h-3 mutation survivors, all missing tests. Crate 181
    tests. Manual game check of a written texport (`new` and edited) still open
  - [x] 2.17h-6 second review round (the first returned seven, all accepted, so the cap was
    binding; `AGENTS.md` "Second opinion" now bounds rounds by accept rate; decision entry):
    seven more, all verified and fixed: 15-17 texport writer refuses ids the reader cannot
    read back, NUL in Team TOML text refused at parse, shirt numbers checked against the
    target's field width at `apply`, checked arithmetic on user integers, legacy playable
    ratings validated at ingestion, unknown members of inline records rejected, the `.4ccs`
    golden extended to all 23 records and every mapped field (lead). Crate 194 tests
  - [x] 2.17h-7 third round (three concerns, under the cap: loop ends): `apply` checks every
    stored value against the target's field width (`schema::bit_width`; `check_width`), one
    `check_text` (NUL, single-byte encodability, capacity) at all four text sites; the texport
    golden compares whole tactics and every player with the schema codec's reads at the literal
    offsets on all four fixtures (lead). Crate 202 tests
- [x] 2.17i — done (the commit after `faa35e0`; 207 tests, mutants-diff 50/0 missed) `settings.toml` stock boots/gloves IDs (decision entry 2026-09-28; spec
  `aesthetics_export/settings_toml.md` "Player settings in exports", `pes_savefile/model.md`
  "Player settings model"): top-level `boots_id`/`gloves_id` in `PlayerSettings` (0 to 100 or
  `""` = default), `BootsId`/`GlovesId` reclassified from compiler-owned to settings, `from_player`
  emitting only 1 to 100, `to_toml`/`update_toml` writing an unset ID as `""`; Team TOML's
  player-level full-range IDs and its `[appearance]` embedding unchanged → verify: `to_toml` of
  `Default` equals the updated plan block literally (`PLAN_BLOCK`); parse accepts 0, 100 and `""`
  and rejects 101 and a `boots_id` inside `[appearance]` or `[appearance.strip]`, each naming the
  key; `from_player` on fixture players gives `None` for 0 and custom IDs and `Some` for 1 to 100;
  `apply` writes an authored ID and leaves the field alone for `None`; the completeness test and
  every Team TOML test pass unchanged. Also: the "Goalkeeper gloves model id" doc comments in
  `schema/fields.rs` and `model/player.rs` become "Gloves model id (any player; 0 = normal
  hands)". Before the `pes_savefile` converge
- [x] 2.18 `python_bindings` (`core/development_plan.md` "Phase 2" `python_bindings`; decision
  entry 2026-09-21): `pes_models_native` wheel (`Fmdl`/`Skl`/`Model`/`MaterialSet` `read`/`write`,
  `FormatError`, `pyo3-log`), `abi3-py311`, a `cdylib` member with `test = false`; `just bindings
  [interpreter]` → `scripts/bindings_check.py` (maturin build, wheel unzipped onto `sys.path`,
  `tests/smoke.py`: 27 fixtures, byte identity where the crates prove it, idempotence elsewhere,
  junk → `FormatError`, no warning on a clean read); green under the dev Python 3.14, Blender
  5.2's 3.13 and Blender 5.0's 3.11. CI `bindings` job (both platforms) added; `pyo3` joins the
  deps table. The Linux wheel built and passed the smoke test in CI run 18 (2.19). Not verified:
  loading from inside a running Blender (only its interpreter was used)
- [x] 2.19 Phase verification — done 2026-09-21 at `43b4ccb`: `just gates` green (41 test
  binaries, 658 tests, 0 failed/ignored; clippy clean; wasm32 check on the 19 libs + `studio_core`,
  `python_bindings` the one documented exclusion), `just deps-check` (`licenses ok` with the
  LLVM-exception allowance), `just bindings` (27 fixtures under Python 3.14); CI run 18 on the
  same commit green on all five jobs, the Linux `bindings` job included (closes 2.18's open item).
  Per-crate counts in the commit message
- [x] 2.19a Review round C (2026-09-19): the first mutation runs and a workspace read-through,
  fixed across six commits (`0fe5e33`..`77b6362`): `cargo-mutants` adopted (decision entry);
  kit_config/fpk/vtree/cpk/ftex/dds_convert test gaps the runs found; fmdl and model_convert
  return `BadReference` where they indexed hostile face, parent and bone-group indices (fmdl
  `SklFile::read` rejects a parent past the bone table; the game's `body.skl` parents bones 1-2
  under 18, so order is not checked); checked arithmetic on file-declared lengths in cpk, ftex,
  fmdl, pes_savefile; `TableOverflow`/`HeaderFieldOverflow`/`SectionTooLarge` instead of
  wrapping casts on write; `hand_split`'s `u16::MAX` sentinel; `sampler_settings_defaulted`
  finding; `formats/{fmdl,pes_model}` split into import/export halves; `.tmp` cleanup on a
  failed save; fmdl read-side and dds_convert mip copies removed; unused `log`/`serde` deps
- [~] 2.20 Converge: own audit then reviewer subagent, each crate against its plan section
  (`libs/README.md`, `model_conversion/README.md`, `pes_savefile/README.md`, `core/development_plan.md` "Phase 2"); gaps become steps.
  Per crate: whole-crate mutation run (sidekick), lead audit (plan blocks, constants, `pub`
  inventory, design sweep, survivor triage), one rework brief, the bounded reviewer loop, one
  commit. Batches: A the seven small leaves in one reviewer round; then `teams_list` +
  `kit_config`; `cpk`; `ftex` + `dds_convert`; `fox2`; `fmdl`; `pes_model`; `model_convert`;
  then read the user-found fork `https://github.com/jasonjk192/pesXdecrypter` for savefile
  facts worth adding to `pes_savefile` before its audit; `pes_savefile` (lead audit whole,
  reviewer on the non-interchange modules: 2.17h's three rounds are the interchange half);
  `python_bindings`.
  - [x] 2.20a batch A (`wezlib` `uniparam` `elevation` `archives` `fpc` `color_tools` `fpk`) —
    done (`efcc580` + the third-round commit): 315 mutants, 54 survivors triaged (36 missing
    tests written with independent oracles where one exists: shell32 `IsUserAnAdmin`, `getuid`;
    13 `relaunch_elevated` documented untestable in `mutants.toml`; 6 bin-key mutants
    equivalent under the distance merge). Reviewer rounds: 7 → 6 accepted, then 4 → 4
    accepted (under the cap, loop ends): `DuplicateName` collisions, drive-relative prefixes,
    lossless UTF-16 argument quoting, color_tools' exact-length contract, 7z skip predicate
    shared with the cache and declared-vs-decoded size check, uniparam NUL names refused at
    `insert` (now `Result`); plan corrections for the zip open cost, uniparam's canonical
    layout (Konami's measured), the stale `libs/README.md` color_tools testing paragraph.
    Fixtures: three archives codec/encryption archives, the colored kit template at 128x128
    as the region constants' independent oracle. Rejected once: fallible writers for 4 GiB
    in-memory containers. fpc's PES 19+ confirmation moved to Phase 4 (decision entry).
    Untested by construction: fpk's u64→usize checks (64-bit host), the 7z size check (header
    CRC), the POSIX `is_elevated` oracle (runs in CI only)
  - [x] 2.20b `teams_list` + `kit_config` — done (`7c4dcc9` + the second-round commit):
    teams_list 71 mutants / 14 survivors (five missing tests, dead guards, one equivalent
    term removed), the revert loop replaced by compute-once-then-recompute; kit_config 310
    mutants / 0 survivors. Reviewer rounds: 7 → 7 accepted, then 4 → 4 (loop ends): only
    slash-wrapped Name cells are teams (a bare `Backup` loaded as team `/backup/`); reconcile
    validates over every claimed id, consumes a placeholder whose id an existing name takes,
    maps appended cells by header label and carries incoming placeholders
    (`MergeSummary::placeholders_added`), every merged list reparses (upstream: 125 teams, 95
    placeholders); kit_config `update_toml` edits `[unknown]` (standard or inline) and badge
    tables in place and returns `Result` instead of panicking on a non-table section, short
    sleeves kept/reported/clamped, `[unknown]` validated at parse, `source_texture_names` is
    `None` without its table. Lead goldens: the referee fixture hand-decoded from the plan
    table field by field, five distinct colors at the table's offsets. Fixtures: Blue's PES
    18/19 UniformParameter containers (4424 configs bit-identical at their versions; 17 GK
    configs per season carry a shirt model outside 144/160/176). Decision entry
  - [x] 2.20c `cpk` — done: 28 survivors triaged (23 missing tests written; 5 sat in code
    removed as dead or redundant: the required-column loop, the `read_table` pre-check the EOF
    helper made redundant, a dead store); after the rework 249 mutants, 0 missed (2 killed by
    timeout: the zero-row-length loop). Public
    surface shrunk to `CpkArchive::{open, entries, read}`, `CpkEntry`, `CpkTimestamp`,
    `CpkWriter::{new, add, finish}`, `CpkError` (no consumer for the @UTF types, `header()`,
    `into_inner`, `crilayla`). Reader: one `read_exact` helper for EOF→`Truncated`, checked
    `FileOffset + base`, row area and row count bounded before any allocation, pool strings
    decoded strictly (`InvalidUtf8`, the reference raises), U32 `EtocOffset` accepted, CRILAYLA
    payload size checked against `ExtractSize` before allocating. Writer: `HashSet` duplicate
    check, NUL paths refused (`InvalidPath`), a header past 0x800 refused (`HeaderTooLarge`).
    Lead goldens: three hand-built CRILAYLA streams (literals only at an exact byte boundary,
    one back-reference, one run through every length-chunk width), decoded by Blue's reference
    decoder before committing. Reviewer rounds: 5 → 5 accepted, then (as an experiment, see the
    2026-09-23 log line) 3 → 3 accepted; loop ended. Decision entry. Untested by construction:
    the wasm32 `checked_add`s in `utf.rs` (64-bit host)
  - [x] 2.20d `ftex` + `dds_convert` — 110 survivors triaged (tests, redundant checks removed,
    one mip-count rule, dead arms gone); `ftex`'s unconsumed `pub` items crate-private; lead
    fixtures `ftex_fixtures.py`, `fixtures_dds_converge{,_r2}.py`, `fixtures_raster_formats.py`
    (texconv 2024.1.1.1 and Pillow as oracles), census `.tmp/ftex_census*.txt` (`c621896`).
    Reviewer loop, six rounds, 7/7/7/7/5/4 accepted (`6157fd7`, `0703e20`, `76d6a08`,
    `9a4cce1`, `7af2de3`, this commit; rulings `.tmp/review_rulings_2_20d.md`): bounded reads
    (FTEX frames, WESYS inflate), checked sizes and allocations (4 GiB decoder/validate caps,
    `isize::MAX` on wasm32), texconv parity (one-channel grey, mip count from the field, NVTT
    L8, `BC4U`, bit-count and alpha-flag matching, legacy `DDSD_DEPTH` volumes; TGA:
    premultiplied, all-zero alpha, 16-bit and interleaved refused), straight alpha (TIFF at
    source precision, premultiplied/paletted/bump/signed refused), encoder borrows source mips,
    JPEG/BMP/WebP/opaque-BC7 tests. Round 4 ran a resumed and a fresh reviewer: resumed found
    4, all within fresh's 7; reviewers stay fresh. Decision entries per round. Deferred: cache
    retention (4.y). Retained gaps (decision entries): hostile FTEX padding in `ftex_to_dds`,
    legacy A8L8 decode, DXGI 2 for RGBA32F. Untested by construction: `dds_to_ftex`'s frame
    offset checks (need a >4 GiB frame area), the wasm32 `alloc_len` path (64-bit host)
  - [x] 2.20e `fox2` — 502 mutants / 47 survivors triaged (missing tests, constants re-read in
    their error branches, dead guards and branches), 0 missed after every round. One byte
    reader, write refusals (`TooLarge`, `KeyCount`, `ZeroTableHash`), explicit `Values` arms,
    unused `binrw` dropped, `text`/`Container` helpers `pub(crate)`. Reviewer loop, three
    rounds, 5/5/2 accepted (`871e7b2`, `3d41122`, this commit; rulings
    `.tmp/review_rulings_2_20e.md`, audit `.tmp/audit_2_20e.md`): XML value text across
    comments, floats read through a double and refused past `f32::MAX`, double text as `repr`
    (ties to even, power-of-two boundary), table dedup on hash and text, blank numbers the
    reference refuses, `<entities />`; the reference's unmasked 33-64 byte hash sum is not
    followed (we match CityHash and the game). Two concerns rejected under the user's "inputs
    are trusted" rule (now `AGENTS.md`). Lead goldens: `double_golden.tsv`, long and wrapping
    rows in `hash_golden.tsv`, raw `city_hash64` values, a real 48-bit collision. Plan
    corrections: double text, `resolve`/`write` rules, `from_xml` defaults, trailer slack,
    `stadium_compiler.md`'s stale `cityhash`/`binrw` line. Decision entries per round
  - [x] 2.20f `fmdl` — 858 mutants / 207 survivors triaged (missing tests; redundant
    pre-checks and capacity hints removed; `select_base_bone`'s dead parent hand-over), three
    rework slices (`cb040b0` format/, `b727747` model/, `b8ce894` ops/): `Model::validate` as the
    one invariant list (`from_file` ends with it, `to_file` and every op start with it;
    `antiblur::encode/decode`, `vertex_enc::decode` fallible, round C's item (b)),
    `Custom-Bounding-Box-Meshes` as the add-on writes it, from/to_file split per table, codec
    internals `pub(crate)`, `MergeError` on thiserror; lead golden `f16_golden.tsv` (numpy as
    the f32->half oracle). Reviewer loop, four rounds, 7/5/7/4 accepted (`25130bb`, `62b124a`,
    `2865a96`, `c2a8f60`; rulings `.tmp/review_rulings_2_20f.md`, audit `.tmp/audit_2_20f.md`):
    split output reloads (repeated assignments with one box id), aliased uv maps, cyclic split
    hierarchies, merge (shared bone boxes, duplicate names, mixed anti-blur), split key and loop
    key on weighted lanes only, true principal axis, decode-before-mutate, -0.0 uvs, zero-vertex
    layout, group boxes over child groups. Final mutation run 873 / 13 missed, triaged in the
    closing commit. Plan corrections: byte identity on Konami's layout only, no per-mesh box.
    Deferred: u16 faces cap a reassembled split mesh (issue below). Decision entries per round
  - [x] 2.20g `pes_model` — 915 mutants / 203 survivors triaged (tests; five equivalences in
    `mutants.toml` with proofs; dead `ModelContainer::sections` and `select_base_bone`
    hand-over removed); lead audit `.tmp/audit_2_20g.md`. New converge method: a **census**
    of every file of the format on the maintainer's machine (`.tmp/model_census/`, 6575
    distinct `.model` files) held the reader to community files, not only the Konami set:
    4145 -> 5999 readable of 6037 PES-format files, 0 regressions per round (plan
    "Community files break two of those regularities", decision entry). Slices `a238c0b`
    (offsets past their section, template annotations), `c796f4b` (zero-length reads,
    repeated field types, loose-vertex repair), `727a92b` (`Model::validate`, empty-array
    header, format/check survivors, format pub surface crate-private), `b862183` (ops start
    with validate; four panics or wraps become errors; split pub surface), `f9c8888` (fmdl
    round 2-4 ports: split and loop keys on positive-weight lanes, principal axis from
    every seed, deterministic points, combine reorders only past u16, encode fails before
    it mutates). Reviewer loop, one round, 4/4 accepted (rulings
    `.tmp/review_rulings_2_20g.md`; `edf68e6`: merge renumbers split groups, decode rolls
    back, v19 extras always; repeated `.mtl` states deferred to 2.20h, issue below). Closing
    run 933 mutants / 10 missed, all killed by tests in the closing commit (filtered re-run
    46 / 0 missed); timeouts are hang-class. Census leftovers, all refused loudly: 19 PES 16
    cloth pants (second vertex set), 7 files with several face descriptors (balls, the PES 16
    shadow), 3 X360, 4 genuinely truncated or corrupt, 5 singles
  - [x] 2.20g-fmdl census (maintainer request 2026-09-28) — done: `.tmp/fmdl_census/` over
    8805 paths (every `.fmdl`, `.skl`, `.fpk` and the models inside each `.fpk`), 4135
    distinct. Readers held up far better than `pes_model`'s: 4088 of 4101 FMDLs read. (a) SKL
    trailing bytes: 13 SKLs (Konami's own `dt00` among them) did not rewrite byte-identically,
    only the tail differing; `SklFile::trailing` carries it, `SklFile::new` pads fresh files
    to 4 (plan "SKL binary format"); now every SKL but one bad-magic file rewrites exactly.
    (b) the 13 FMDL failures are all correct refusals, and the community add-on refuses every
    one the same way: VAT flags with skin fields but no bone table, 0xFFFF bone and string ids
    in test faces, a stadium part with one box for three groups, hand-scrambled experimental
    balls, a tombstoned duplicate block, the `fpk` crate's synthetic fixtures, and O's
    `Winter 20 Additions/Boots/k0977/boots.fmdl`, genuinely truncated (1.39 MB of a declared
    4.28 MB: broken in that export). (c) The 64 refused `.fpk` are another game's archives
    (`chr_FUC`, `stg_fuc`), not PES files
  - [x] 2.20h `model_convert` — done. It opened with a **conversion census**
    (`.tmp/convert_census/`: every readable `.model` bundle and FMDL on the machine converted to
    the other engine and back, written and read back) and a `.mtl` census (`.tmp/mtl_census/`).
    Before, about one conversion in six failed. Now every file converts except 4 with NUL
    material names, which are refused on purpose (`MtlName`). The decisions are in `ir.md`
    "What real files carry", the `conversion.md` split rules, and two decision entries.
    Slices:
    - `dc19c62`: split groups that cannot be whole stay split, no empty containers, FMDL
      over-flagged groups, `.mtl` `maxfilter` and trailing content.
    - `8d30428`: unnormalized weights, Fox bones parent-first, `MtlName`, vertexless meshes.
    - `1bf6a0f`: open questions (1)-(3) settled (decal families approximate, hand split
      left-first with material pruning, `retarget` by value); incompatible split groups stay
      split.
    - `3f3f7d5`: 568 mutants / 74 survivors triaged; fmdl's combine no longer welds components
      with different materials; helpers crate-private.

    Reviewer loop, two rounds, 7/3 accepted (`e335a6e`: poses travel in an SKL, the retarget
    blend normalizes, clamp-then-total quantization, hand-table conformance for gloves'
    shared bones, native texture and parameter loss reported, one `static` bone, `maxfilter`
    through the IR; `fe33060`: linear base colour space, stale bone boxes, Fox flag loss, and
    default-valued parameters silent). Rulings: `.tmp/review_rulings_2_20h.md`. Closing run:
    616 mutants / 7 missed, all known equivalences whose line patterns moved. Questions (4)-(6)
    stay open (issue below)
  - [x] pesXdecrypter fork read (`jasonjk192/pesXdecrypter`): it merges `libpes15crypter`'s
    PES 15 routine into pesXdecrypter. The seed-byte LCG `(c * 21 + 7) % 32768`, key `c % 255`,
    restarted per chunk, with the 4-byte chunk lengths left clear, is what
    `pes_savefile::container::pes15` already does. Nothing to add
  - [~] 2.20i `pes_savefile`. The **savefile census** (`.tmp/save_census/`: every
    `EDIT` + 8 digits file on the machine, 228 distinct) decrypts, decodes, re-encodes and
    compares each decrypted payload byte for byte. 186 files round-trip exactly (PES 16-21).
    32 archives and shortcuts are correctly refused. There are no PES 15 saves on the machine.
    The triage traced three failing classes (not yet fixed):
    - (1) 6 PES 16 saves, the maintainer's own among them: a `ShirtName` fills all 16 bytes
      with no NUL (`MEAT ON THE BONE`). Read accepts it; write refuses it (`len < field`).
    - (2) 3 PES 17 saves: a UTF-8 `Name` exactly 46 bytes long, the same asymmetry.
    - (3) 1 PES 17 save: a `Name` with a CP1252 byte `0xA3` ('£'), refused on read as
      not UTF-8. 4ccEditor truncates (1-2) and mangles (3).
    Decided 2026-09-28 (`codec.md` "Text fields hold `len` bytes", decision entry): (1)-(2)
    a text field holds `len` bytes, no NUL when full (codec write, `text_max`, `convert`;
    `shirt_name_from` keeps its free byte); (3) stays a refusal. → verify: census re-run,
    the 9 saves of (1)-(2) round-trip byte-identical, 0 regressions, (3) still refused; codec
    and Team TOML tests for a full field and one byte over, red first. Done in the text-fix
    commit: re-run (`.tmp/save_census/census_after_moved.txt`, `compare.py`; the Discord
    backup's 61 paths repointed to `Tools_Mine/bonfire/discord-backup` by `repoint.py`) has
    all 9 round-tripping, no other outcome changed, and the '£' save still refused (now also
    at `Pro Evolution Soccer 2017/save/EDIT00000000`, a copy the maintainer made to test it).
    `mutants-diff 6aeeecf`: 16 mutants, 0 missed. The whole-crate mutation run was stopped at about 87
    minutes on the maintainer's request. The partial local shard (385 of 805 mutants) left 20
    survivors, all reading as missing tests (discovery's account-folder filter,
    `section_records` bounds, the `*_mut` accessors, `fresh_salt`, `codec/team.rs` index
    math, container boundary checks). Rerun it whole next session. Round C input (c) is ruled
    no change: discovery returning an empty list for both "no Documents" and "no saves" feeds
    only the Open menu's shortcut entries, where both mean "no entries". Whole-crate run at
    `40a07be` (`.tmp/mutants_2_20i_whole/`): 806 mutants, 716 caught, 24 missed, 2 timeouts,
    64 unviable, all on this PC in 3 h 23 min (see the sharding issue below); the VPS half
    is discarded. This run is the crate's last (maintainer, 2026-09-28, decision entry): the closing
    measurement is `just mutants-diff 40a07be` over every rework commit, not a second
    whole-crate run. Next: rework from the lead audit (`.tmp/audit_2_20i.md`) and the run's
    survivors, reviewer on the non-interchange modules.
  - Remaining 2.20 order after this:
    `python_bindings`; then 2.21.
  Known inputs from round C: (a) whole-crate mutation runs left survivors to triage in cpk (28),
  ftex (80), dds_convert (30): table-variant arms, boundary comparisons, `write_cell` and
  `Writer::finish` padding math; the other thirteen crates have not been run; (b)
  `fmdl::ops::antiblur::decode` and `model_convert::ir::remap_bone_group` return `()` and still
  index caller-supplied indices raw (both callers return `Result`; threading one through is a
  signature decision); (c) `pes_savefile::discovery` returns an empty candidate list both for
  "no Documents folder" and "no saves"; (d) `pes_savefile/README.md`'s `TeamEntry`/`TacticsPreset`/
  `PlayerEntry` blocks are rewritten from the code in 2.21 (decision entry 2026-09-19)
- [ ] 2.21 Rewrite those sections in the present tense

### Phase 3 — Team compiler skeleton

Spec: `docs/plans/core/development_plan.md` "Phase 3", `docs/plans/team_compiler/README.md`, `docs/plans/aesthetics_export/README.md`.
Steps are itemized when Phase 2 closes; the first is fixed:

- [ ] 3.1 Acceptance: write `team_compiler/README.md` "Acceptance" for the Phase 3 scope (format:
  `CONTRIBUTING.md` "Testing")
- [ ] 3.2 Converge check script: extract every acceptance ID from the plans' "Acceptance" sections
  (skipping `withdrawn:` ones) and every `// XX-YYY-NN` citation in any `.rs` file under `crates/`
  (inline `#[cfg(test)]` modules included, not just `tests/`); report orphan citations (an ID no
  scenario defines) and unproven scenarios. Two modes: **report** — what CI runs on every change;
  it fails only on orphan citations, since scenarios are written before the code that proves them
  and unproven IDs are the normal state of an open phase; **strict** — run at converge for the
  closing phase's IDs; unproven or `manual` scenarios without a recorded check fail it. Python or a
  tiny Rust bin — decide when written (needs a decision entry either way, since it adds a gate)
- [ ] 3.3 Tracer bullet (`core/development_plan.md` "Phase 3", first bullet; decided 2026-09-15): fixture pair
  (one old-layout face export, its hand-migrated Studio-layout twin, the hash manifest of Red's
  output for it); the thin compile path through `fmdl`/`ftex`/`fpk`/`cpk`/`kit_config`; the
  first `tests/parity` case. Lead writes the fixtures and the manifest (correctness-critical);
  the compile path and comparison are briefed. → verify: `cargo test -p team_compiler --test
  parity` green on the fixture, and every Phase 2 API friction met on the way listed in the
  brief's report (each is a lib-crate fix or a decision entry, made before 3.4)
- [ ] 3.z Shell slice, last code step of the phase (`core/development_plan.md` "Phase 3", last bullet; decided
  2026-09-15): minimal `studio_core` shell (window, sidebar, selected tool's `view()`), `studio`
  binary registering `team_compiler`, Team compiler `view/` with settings, run button and a plain
  `PipelineEvent` log. → verify: manual, recorded in the converge step: the 3.3 fixture compiled
  from the GUI with its events visible, on Windows; Linux when a machine is available

### Phase 4 — Processing logic

Steps are itemized when Phase 3 closes, and only after 4.0 is done; one more is fixed already
(the GPU BC7 step moved to Phase 16, decision entry 2026-09-28):

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

- resolved (2026-09-28, maintainer: the limit stays, decision entry) — `.model` face limit
  21845 contradicted by cup files (found at the 2.20g census):
  193 community `.model` files carry a mesh over `FACE_LIMIT_HARD` (more than 65535 face
  indices; the face count field is `u32`), among them cup exports (`Teams_Main/O/Exports/O
  Aesthetic Export from Autumn 17/Faces/73618 - Bedford Rascal/oral_rascal_win32.model`), and no
  Konami file does. The limit comes from the old add-on's own constant ("the maximum that can
  be saved"). If PES loads such meshes, `check`'s `model_mesh_over_face_limit` (Error, "the game
  cannot load it") is wrong. `split::encode` still takes them apart harmlessly. Needs the
  maintainer: is one of those models known to render in game? Decide before Phase 3 maps the
  code in the Team compiler's catalog.
- resolved (2026-09-28) — `just mutants` sharding (found at 2.20i): at `40a07be` all 806
  mutants ran locally while a VPS half also ran and then died when ssh dropped. Cause of the
  local half: agent shells do not inherit the user variable `STUDIO_MUTANTS_REMOTE` (process
  empty, user registry `bonfire`), so `just mutants` from one took the local-only path. Fixed
  in `scripts/mutants.py`: the variable is read from the registry too, the mode is printed,
  and the remote half runs detached with polling and `just mutants-collect` (`AGENTS.md`
  "Environment"). Checked on `fpc`, `color_tools` and `cpk`: halves of 8/8, 46, 125/124; the
  remote half survived its controller being killed and was collected; a died half is
  refused as a new run and collected partially; an unreachable host retries then gives up.
- open — u16 face indices cap a reassembled split mesh (found at 2.20f review): `fmdl::Mesh`
  and the IR (`ir.md` "IR struct") store faces as `[u16; 3]`, so `fmdl::ops::split::decode`
  refuses (loud `VertexMismatch`) an add-on file whose components together reference more
  than 65536 vertices, and vertex-limit splitting of an IR mesh can only move loose vertices.
  The 2.20h conversion census found 114 real files over it. Since 2.20h such a group stays
  split rather than erroring (`conversion.md` split rules), so nothing refuses them now. The
  face type is still Phase 7's decision (glTF brings u32 indices).
- resolved (2026-09-28) — repeated `.mtl` states: the `.mtl` census found no file on the
  machine that repeats a state; every reader now takes the first value.
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
  link or a killed controller (`just mutants-collect`), and the split no longer silently
  becomes a full local run in shells without `STUDIO_MUTANTS_REMOTE`.
