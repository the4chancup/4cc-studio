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
Converge next (per crate, bounded review loop). Review round C (2026-09-19) closed as 2.19a; its
leftovers are listed under 2.20. 2.5b (GPU BC7) deferred to Phase 4 (user decision 2026-09-21).
**Blocked on:** nothing

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
- [ ] 2.20 Converge: own audit then reviewer subagent, each crate against its plan section
  (`libs/README.md`, `model_conversion/README.md`, `pes_savefile/README.md`, `core/development_plan.md` "Phase 2"); gaps become steps.
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

Steps are itemized when Phase 3 closes; one is fixed already:

- [ ] 4.x `dds_convert` GPU BC7 (deferred from 2.5b, decision entry 2026-09-21; spec
  `libs/README.md` "First-release desktop GPU BC7"): `block_compression`'s wgpu backend on a
  Vulkan/Metal device, CPU fallback with the fallback reason reported, cold pipeline creation
  and upload/readback measured first, then the texture step's bounded batches → verify: GPU and
  CPU outputs decode within the same tolerance on the `dds_convert` fixtures; the fallback path
  exercised by forcing no adapter

---

## Issues

Bugs, unexpected behavior, things to revisit. `open` / `resolved (date)`. Resolved issues are
pruned when their phase closes; they stay in git history.

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
