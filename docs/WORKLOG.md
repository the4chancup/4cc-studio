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
**In progress:** 2.16 `model_convert` (2.16a–f done; 2.16g hand split next), then `pes_savefile`, `python_bindings`
**Blocked on:** nothing

---

## How to resume

| Thing | Where |
|---|---|
| Plan index | `docs/plans/README.md` → `docs/plans/core.md` |
| Development phases | `docs/plans/core.md` "Development Plan" |
| Legacy tools (format evidence) | `docs/plans/core.md` "Project context" (paths are per-machine) |
| Skeleton data | `resources/skeletons/` (see its README) |
| FPC source text (for the `fpc` crate) | `resources/FPC.wikitext` |
| `.model` format as measured (for the plugin author; the crate's notes are in `libs.md`) | `resources/prefox_model_format.md` |
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
  PES 2017 install (`libs.md` "`pes_model::format`: the `.model` container and its sections"):
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
- [ ] 2.16g `model_convert::ops::hand_split` → verify: synthetic mesh with an `skh_` strip splits
  with the one-ring growth; a model without hand weights passes through
- [ ] 2.16h `model_convert` routing + `loss.rs` → verify: FMDL→.model of a Konami fixture equals
  the legacy converter's output semantically (lead-produced reference)
- [ ] 2.16i checkpoint (b) review of the new `pub` surface
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
- **2026-09-13** - 2.5 `dds_convert` (CPU), 2.6 `fmdl` and 2.7 `pes_model` (all but
  `ops::merge`) done; reviews A and B closed (2.5c, 2.12b, 2.13b); 262 tests workspace-wide. Two
  censuses recorded in `libs.md` (2610 `.model`, 945 `.mtl`). Safeguards for scripts and sidekick
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
