# 4cc Studio — Team compiler plan: Testing

Part of the [Team compiler plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Testing: parity against Red (adapted from Blue)

Blue has `test_parity.py` that runs the full pipeline, compares ordinary leaf files byte-for-byte,
and compares nested CPKs by entry equivalence so container timestamp differences are accepted. Since
the Rust compiler only accepts the Studio export format, the parity test becomes a two-step flow:
**upgrade the old reference exports with the Export upgrader, compile the result, and compare the
game-facing output** (nested CPK contents, bins, packed structures) against Red's reference output.
The Rust test expands Blue's comparison into the four tiers below.

```text
cargo test -p team_compiler --test parity
```

The parity harness supplies the reference-fixture location; it is not a custom `--reference` flag
passed through to Rust's standard test harness, which does not accept that option.

The reference tree is the **extracted contents** of Red's output CPK (Blue's `Singlecpk_files`
convention) — Red itself emits `patches_output/{cpk_name}.cpk`, so the fixture is produced by a
one-time extraction step. The extracted trees are several GB of binary data and are **not committed
to git**; instead, a **hash manifest** (content hash per file, keyed by normalized relative path)
is committed as the test fixture. The test regenerates the reference tree locally by running Red +
extraction when the manifest is stale or the local tree is missing, and verifies the regenerated
tree against the committed manifest before using it. This keeps the repo small while making the
test reproducible without a pre-shared binary blob.

Intentional differences must be accounted for:

- **Auto-assigned boots/gloves IDs** may differ from the old embedded IDs — the comparison
  normalizes ID-dependent paths and embedded references.
- **Texture locations**: every player's textures are relocated to per-player common subfolders (see
  "Texture relocation to common" in the walkthrough), where Red packs team textures inside each
  model folder's structure; textures resolved from `Common/` stay in the team's Common output and
  are referenced in place. The comparison matches textures by content across locations and verifies
  that rewritten references resolve to the relocated (or in-place Common) files, and that a Common
  texture appears once in the archive however many players reference it.
- **Automatic collar reconciliation**: unlike Red's pass-through behavior, the compiler derives the
  custom collar ID from the `collar_[ID]` filename and rewrites the team's kit configs; normalized
  parity permits and verifies those expected config-field changes.
- **Dropped features** (Other folder) are excluded.
- **Kit-dependent path magic**: Red emits the legacy `u0XXXp0`/`u0XXXp1`… spelling (with the `XXX`
  placeholder or a baked-in team ID, as the author wrote it), the compiler emits `kitN`/`kit1`…
  (see "Kit-dependent assets" in the [Unified model format plan](../model_format.md)). The comparison
  normalizes Red's `u0???p<d>` to `kit<d>` (`p0` → `kitN`) in file names and in decoded FMDL/MTL/XML
  paths before diffing. The compiler gets no switch to emit the legacy spelling: parity is a
  file-tree comparison, not an in-game test, and the exes can be modded to `kitN` at any time, so a
  switch would be a second output format maintained for nothing.

Comparison uses four tiers:

1. **Exact leaf bytes** for unaffected files such as textures and raw bin outputs.
2. **Decoded and normalized comparison** for FMDL, MTL, XML, kit configs, and other ID/path-bearing
   content; generated IDs and relocated paths are normalized before remaining fields are compared.
3. **Archive-entry equivalence** for CPK/FPK containers, whose container bytes may differ because of
   ordering metadata or timestamps while their normalized entries agree.
4. **An explicit intentional-difference allowlist** reviewed with the fixture, rather than broad
   exclusions that can hide regressions.

Maintain a versioned test matrix for PES 15–21 covering team and referee exports; folders, ZIP, and
7z; native and converted pre-Fox/Fox models; local, shared, and combined models; normal, multi-CPK,
test, and sider modes; savefile success/failure; and cancellation. Key behavior areas:

- **Dispositions and GUI**: pass-through and `DoneWithErrors`; an Error-level `DropFile` rendering
  red at check time and finishing `DoneWithErrors`; `DropFolder` outcomes; an effective `DropExport`
  blocking every row cell; an invalid `ParsedAestheticsExport` still producing stable GUI rows and scoped
  messages; stale validation event rejection; disabled exports producing a log entry but no row; a
  failed kit showing base-bin colors; catalog completeness (every emitted code has a template,
  context placeholders are satisfied, code+scope grouping is stable, CLI and GUI render the same
  underlying message).
- **Rosters and identity**: an export stem such as `co - Spring.zip` resolving through `/co/`; a
  folder and archive with identical display stems staying distinct (including in test-mode output
  directories); referee identity never constructing a normal `TeamId`; duplicate folder-name slots
  vs duplicate authoritative `players.txt` slots and their distinct dispositions; line-local
  malformed and out-of-range roster entries producing `DropSlot`; an empty/all-invalid referee
  roster blocked while an empty normal-team roster supports a kit-only export; a refs export without
  `players.txt` blocked by the compiler yet repairable in the Refs arranger; multiple refs exports
  (valid or not) all blocked at discovery; team-ID resolution failure; duplicate team IDs;
  multi-mapped folders (team and referee) preparing once and instantiating per slot
  atomically, with textures emitted once into the name-keyed common subfolder shared by all mapped
  slots.
- **Models and textures**: target-native plus convertible model variants; each model source owned
  exactly once; a team player with boots/settings but no face; multiple same-category shared links
  rejected; both `ingame_face` spellings normalizing identically with a local pre-Fox boots
  composite still emitted; FMDL→pre-Fox conversion emitting an atomic `.model`+`.mtl` pair; a glTF
  image shared by two model parts loaded and emitted exactly once; shared-texture dedup and
  deterministic conflict resolution under reversed rayon completion; logo production (a 1000×600
  source under each fit mode yields the three square sizes; `logo_small` replaces only the 128²;
  an undecodable `logo_small` drops all three; a 300² source is upscaled and reported); kit
  folders (`p1 - Lakers` resolving to `p1` with the label kept; `p1/` beside `p1 - Lakers/`
  discarding both; `all/kit_back.dds` emitted under every kit's ID with each config's `back` field
  set, while a kit's own `kit_back.dds` wins for that kit only; the emitted files identical to a
  compile of the same export with the shared files copied into each kit folder; `all/config.toml`
  ignored with a warning; an empty `p3/` yielding the checkerboard texture, the mask template, the
  template config and a UniColor entry carrying the magenta/black pair, with `kit_placeholder`
  and `kit_colors_missing`; the same `p3/` with a `colors.txt` carrying its colors and no
  `kit_colors_missing`; a `p4/` holding only `kit_back.dds` yielding the same plus `_back` and a
  config whose `back` field is set; a kit with a real texture and no colors deriving from the
  texture, never reaching the loud pair); kit layout (a `pre-fox` kit compiled for PES 21: every
  texel outside the sock and shorts islands byte-identical to the no-marker compile, the islands
  matching the golden produced from the hand-adjusted fixture pair, `kit_layout_converted`
  reported; the same kit compiled for PES 17 identical to the no-marker compile; a `fox` kit for
  PES 17 taking the inverse table, and pre-Fox→Fox→pre-Fox on a synthetic texture whose bands are
  flat colors round-tripping exactly; `_back`/`_leg`/`_name` untouched either way; both markers
  discarding the kit; a marker in `all/` warned and ignored; a placeholder kit with a marker
  compiling as without one); mask/srm (a kit with `kit_mask.dds` compiled for PES 21 emitting no
  `_mask` and no `_srm`, with `kit_texture_not_used`; the same kit for PES 17 emitting the mask
  as given; a kit with `kit_srm.dds` for PES 17 emitting the mask *template* and no srm; a kit
  with both emitting exactly the target's one, silently); user `face.xml` (a hand-written xml
  compiled for PES 17 emitted with the team ID substituted and its unknown `type` and extra
  attribute kept verbatim, each with its warning, and `level="1"` kept with `xml_level_lod` as
  info; the same folder with the xml removed
  producing the generated xml, proving filename typing is off when one is present; a `<model>`
  without `path` and a `./` reference to an absent file each discarding the folder; a
  `model/character/face/common/…` path passing with `xml_path_unchecked`; a folder with a `<dif>`
  in the xml and a `face_diff.xml` beside it discarded; an xml lacking `face_neck` gaining the dummy
  entry; an unlisted `.model` reported and absent from the output; the same folder compiled for
  PES 21 ignoring the xml and converting the models); collar validation (`collar_[ID]` filename parsing, a cross-team
  `collar_id_conflict` resolving by canonical export order, and a custom collar overriding the FPC
  collar value in every config).
- **Planning and writer**: cross-export duplicate paths producing the same manifest under reversed
  completion order; sideload-versus-export collision resolution; nested-root path collisions;
  partial planning continuing valid teams after refs or duplicate-team drops; a plan with no
  eligible tasks producing no output or deployment; a structural `DropFolder` still yielding a
  compilable export; a late model-task failure leaving no entries from that task while successful
  sibling tasks may still commit; a failed kit never mutating UniColor/UniformParameter; notes from a rejected export excluded from
  `teamnotes.txt`; root colors/notes/referee-marker descriptors reaching planning; scope-dependent
  `source_read_failed` and `template_override_unreadable` outcomes; pass-through refusing
  conversion/packing/logo/texture-conflict findings and
  unsafe-path/ambiguous-roster/required-metadata findings.
- **Deployment and environment**: savefile IDs preserved for players whose boots/gloves task failed
  while independent settings still apply; with either FPC marker, successful standalone assets
  overriding preset IDs, failed/dropped requested assets preserving existing IDs, and absent
  standalone assets using preset IDs (including pre-Fox face-XML-local parts); source modification after planning producing
  `source_changed_during_run` (no final output published, no installed CPK/savefile/dt00 changed);
  separate refs CPK installation; `dt00_x64.cpk` write failure; multi-CPK deployment failure; no FPC
  markers with an already-FPC savefile (configs untouched); a mixed `fpc.on`/`fpc.off` team applying
  per-player presets while every kit config gains the FPC values; an FPC team's unexported kit slots
  patched from the installed cup entries (Fox and pre-Fox), including the no-existing-entry
  `kit_config_fpc_unpatched` warning; logs/sideload/output path resolution in portable and
  user-config modes; invalid-UTF-8 and empty `notes.txt`; invalid `settings.toml` preserving models
  and savefile values; atomic `players.txt` replacement never exposing a half-written roster to the
  watcher.
- **Infrastructure**: oversized memory requests, MemoryBudget wake-up stress with multiple oversized
  waiters, and automatic worker counts on one- and two-core hosts; solid-7z reads not copying the
  archive buffer per entry; a solid 7z with unnumbered players; `CpkStem` boundary cases (empty, 29
  characters, spaces, separators, reserved device names, explicit `.cpk`, case collisions);
  deterministic converted output across fresh processes with randomized hash seeds; standalone
  kit-config binary → TOML → binary preservation of source texture-name fields.

Separate **Red parity** (normalized archive-entry equivalence is acceptable) from **Rust
reproducibility**. Explicit CPU mode is the deterministic reference: identical inputs and encoder
configuration must produce byte-identical CPK, test, and sider artifacts, including canonical
ordering and normalized timestamps. GPU mode may produce different valid encoded texture bytes
across devices/drivers, and containers containing those textures may consequently differ too.
That exception does not relax model/bin output, asset identities, collision decisions, or ordering;
GPU checks compare decoded texture semantics and quality rather than demanding CPU byte equality.
Savefiles use fresh randomized encryption salts, so compare them after decryption and semantic
normalization, or inject a deterministic RNG in tests.

---
