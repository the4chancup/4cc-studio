# 4cc Studio — Core plan: Development plan

Part of the [Core plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Development Plan

Phases 1–2 build the platform skeleton and every lib crate that can be verified on its own (against
fixtures or reference outputs); the tools follow, each bringing the consumer-shaped libs it is the
first to need. Each phase's detailed specification lives in the corresponding plan document: phase
2 in [libs](../libs/README.md)/[model conversion](../model_conversion/README.md)/[savefile](../pes_savefile/README.md), phases
3–4 in [team compiler](../team_compiler/README.md), phase 5 in [savefile](../pes_savefile/README.md)/[save
editor](../save_editor.md), phase 6 in [export upgrader](../export_upgrader.md), phase 7 in [model
conversion](../model_conversion/README.md), phase 9 in [stadium compiler](../stadium_compiler.md), phase 10 in
[music player](../music_player.md)/[music export editor](../music_export_editor.md), phase 11 in [match
tracker](../match_tracker/README.md), phase 12 in [kit config editor](../kit_config_editor.md), phase 13 in [refs
arranger](../refs_arranger.md), phase 14 in [balls compiler](../balls_compiler.md), and phase 15 in
[player aesthetics editor](../player_aesthetics_editor.md).

**Releases.** The suite first ships as **0.1.0 at the end of Phase 8** ("Release 0.1.0" below):
the Team compiler with compile-time savefile writing, the Save editor and the Export upgrader, on
Windows, with the CPU texture encoder and the native model formats only. Phase 8 is the line
because it is the first point where the two main tools exist with a GUI, and because its own
verification (community feedback) needs a distributed build; every later tool replaces a legacy
tool that keeps working until then. The phases therefore run in the order 1–6, 8, Release 0.1.0,
7, 9–16: glTF (Phase 7) moves after the release, since no current export carries glTF and it
depends on the Blender `pes-models` codec outside this workspace. Phase numbers are kept, since
every pointer names them. Releases stay `0.x` until Phase 16 closes with **1.0.0**; "first
release" in the plans means 1.0.0, so a "first-release" feature may land after 0.1.0.

**Target versions.** The cup moves from PES 17 (pre-Fox) to a Fox version, not yet chosen, around
April 2027, and plays Fox versions for at least a year after that; Studio's first-class target is
therefore that Fox version, and PES 15–17 support covers what the plans already give them. A
change to the game's own behavior that Studio's output relies on (gloves on outfield players,
the referee hook, LiveCPK) is made in **FoxDen**
(`Tools_4cc/FoxDen`), the community's runtime for PES 18–21, rather than in a patched executable:
FoxDen applies the 4cc exe patches at runtime to the stock exe, so one installed file carries
them. FoxDen runs on the Fox games' Lua engine and cannot exist on pre-Fox versions; a pre-Fox
equivalent is built only if a pre-Fox version becomes the cup's game again, so a feature that
needs one is Fox-only until then.

### Phase 1: Workspace bootstrap + core skeleton

Done (2026-09-13). The guardrails above apply "from the first commit", so the first commit is the
one that makes them enforceable:

- The root `Cargo.toml` is a virtual workspace (`crates/studio`, `crates/studio_core`,
  `crates/libs/*`; `crates/tools/*` joins with the first tool crate, since cargo rejects a member
  glob that matches nothing). `[workspace.dependencies]` declares every shared crate once, and
  `[workspace.lints]` holds the policy every member inherits via `[lints] workspace = true`:
  `rust::missing_docs` (the `///`-on-every-`pub` rule) and `clippy::let_underscore_must_use =
  "deny"` (a discarded `Result` is a compile error unless handled or `#[expect]`ed with a reason,
  see the code style rule in `../../CONTRIBUTING.md`), plus `dbg_macro`, `print_stdout`,
  `print_stderr` and `todo` at `warn`, which `-D warnings` makes red; `studio`'s CLI result
  output carries the one `#[expect(clippy::print_stderr)]`. `[profile.dev.package."*"] debug =
  false` builds dependencies without debug info while workspace crates keep theirs, since
  dependency debug info is the bulk of a dev `target/` (measured on a comparable project: 24 GB
  of `debug/deps/`) and is never stepped into. `rustfmt.toml` (`style_edition = "2024"`, LF);
  `rust-toolchain.toml` pins `1.98.0` with `clippy`, `rustfmt` and the `wasm32-unknown-unknown`
  target, so rustup installs all of them on the first `cargo` call.
- The root `justfile` is the single definition of the four gates (`just gates`: fmt check, clippy
  with `-D warnings`, `cargo test --workspace`, and `scripts/wasm_check.py`, which derives the
  crate list for the `wasm32-unknown-unknown` check from `cargo metadata`: `studio_core` plus
  every crate under `crates/libs/` minus a named exclusion list, so a new lib is checked without
  anyone remembering to list it) and of `just deps-check` (guardrail 4 through
  `scripts/deps_check.py`, which walks each guarded crate's normal dependencies over every target
  and every feature; plus the `cargo deny` license allowlist in `deny.toml`). On Windows the
  recipes run under PowerShell (`set windows-shell`): Git for Windows' `sh` is not on PATH from a
  plain PowerShell, the fallback `../../CONTRIBUTING.md` named; a planted clippy warning was verified to
  turn `just gates` red there. The remaining recipes the plan names (`acceptance`, `parity`,
  `bindings`, `release`) arrive with the phases that give them something to run.
- CI (`.github/workflows/ci.yml`) runs `just gates` on Ubuntu and Windows and `just deps-check`
  on Ubuntu, installing `just` and `cargo-deny` as prebuilt binaries; the `python_bindings` build
  job is added in Phase 2 with the crate.
- `crates/libs/`, `crates/tools/`, `crates/studio/`, `crates/studio_core/` exist, so every later
  crate lands in its planned place.

The non-GUI parts of `studio_core` exist as the closed module tree above prescribes: `tool.rs`
(the `StudioTool` trait as specified under `architecture.md` "Tool plugin interface", `ToolContext`, and the
`ShellRequest` enum the context queues for the shell: `SwitchTool`, `Notify`, `SettingsChanged`),
`events.rs` (the `architecture.md` "Event system" types, with `Message` and `MessageCode { tool_id, code }` as the
Team compiler plan's "Message structure" defines them), `status.rs` (`ShellCondition`,
`ToolActivity`, `Notice` with at most one typed `NoticeAction`), `help/mod.rs` (the `HelpSection`,
`HelpTopic`, `HelpTarget` types only; search, rendering and export come with the GUI), `settings/`
(`Settings::load` / `parse` / `save` / `merge_defaults`, the `[common]` + per-tool-table layout
under `gui.md` "Settings menu", `CommonSettings` with the defaults the Team compiler plan lists and the
update-state keys; saving is atomic through a `.tmp` rename and the caller decides what a failed
save means) and `shell/launch.rs` (the three launch modes, `-v` verbosity, one clap subcommand per
registered tool named after its id, duplicate or reserved ids rejected at registration, and
`run_cli` dispatch). Two leaf libs came with it: `vtree` (`ScopePath`, `RelativeScopePath`,
`VirtualTree<T>`, see `../libs/README.md`) because `PipelineEvent` addresses by `vtree::ScopePath`, and
`pes_version` because `CommonSettings` holds a `PesVersion`. `crates/studio` is a stub binary:
it parses the launch mode, installs the CLI `env_logger` sink, and dispatches to a registry that
is still empty; the GUI modes exit with a message until Phase 8.

**Verification (done):** all four gates green on the workspace and `just deps-check` green; 38
unit tests, including settings round-trip and recursive default-merge, `ScopePath` rejection and
collision cases, and a stub tool registered through the trait and dispatched from
`4cc-studio stub ping x`. CI's first green run and one deliberately red run are recorded in the
worklog when the first push happens.

### Phase 2: Library crates

Every lib crate that can be verified on its own — against real-file fixtures, the legacy tools'
reference outputs, or documented data — exists, built before any tool: leaf crates first, then
their dependents, `python_bindings` last. Libs whose API is shaped by a consumer
(`aesthetics_export`, `pipeline`, `aatf`, `music_export`, `audio_engine`, `match_feed`) wait for
the phase of the tool that first needs them. Each crate went through a converge audit (worklog
2.20): a whole-crate mutation run with every survivor triaged, a public-surface audit, a census
of every file of its format on the maintainer's machine for the format readers, and the
cross-family reviewer loop.

**Format codecs**, with `binrw` where the layout is declarative (`fpk`, `ftex`, `fmdl`,
`pes_model`) and hand-written readers elsewhere. Blue and the stadium compiler's parser variants
(`fmdl_file.py`, `cpk.py`, `ftex.py`) were format evidence checked against fixtures, not
implementations translated:
- `wezlib`
- `cpk` (read + write, CRILAYLA inside)
- `fpk`
- `ftex` + `dds_convert` (`image`, and `block_compression` for CPU BC1/BC3/BC7 encoding and for
  decoding): the CPU reference only. The desktop GPU BC7 backend (`libs/README.md` "First-release desktop
  GPU BC7") is built in Phase 16, after 0.1.0, on top of Phase 4's texture step, not here: every
  rule the plan gives it (bounded batches against the memory budget, cancellation, writer
  progress, one encoded result shared by all consumers, the fallback report) is pipeline
  integration, so a Phase 2 version would be shaped without its caller and reshaped once. The
  cold-start and upload/readback measurements are the first thing the GPU step does, before
  relying on it. BC7 output is PES 19–21 only; PES 15–18 use BC3 or eligible opaque-color BC1 — see
  [libs](../libs/README.md).
- `fmdl` — `format/` (read/write, byte-identical round-trip), `ops/` (mesh splitting, split vertex
  encoding, anti-blur, multi-FMDL merging, path-table editing), `check.rs`
- `pes_model` — `format/` (.model + sibling .mtl), `ops/` (mesh splitting, split vertex encoding,
  merging, texture stem rewriting), `check.rs`
- `uniparam`
- `fox2` (+ CityHash64, verified against fixtures)
- `archives` (.zip/.7z via `zip` and `sevenz-rust2`)

**Data and rule leaves:**
- `fpc` (Full Player Customization as data: kit values, player presets, interference rules —
  dependency-free, so it precedes `kit_config` and `pes_savefile`; spec in [libs](../libs/README.md))
- `teams_list` (`TeamName` fold, `TeamId` range, `teams_list.txt` parse/write/reconcile with the
  embedded upstream list — dependency-free; fixture: Red's current file; spec in [libs](../libs/README.md))
- `kit_config` (binary ↔ TOML kit config codec, applies `fpc::kit` values — format documentation
  in the [Kit config editor plan](../kit_config_editor.md))
- `color_tools` — dominant kit-color extraction only; the picker widget is Phase 8
- `elevation`

**`model_convert`** (spec: [Model conversion](../model_conversion/README.md)) — native formats only; glTF
is Phase 7:
- The `CanonicalModel` IR
- Importers `fmdl_to_ir`, `model_to_ir`; exporters `ir_to_fmdl`, `ir_to_model`, calling the format
  crates' `ops/` for splitting, encoding and anti-blur rather than reimplementing them
- Hand auto-split in the IR, in-process without Blender: select weighted hand vertices, grow once,
  separate — see [Model conversion](../model_conversion/hand_split.md#hand-auto-split)
- The games' skeleton files embedded and exposed per version (`resources/skeletons/`), the render
  hierarchy and fold table beside them, and the retargeting pass
- Material conversion logic
- Cross-version player save data (`convertPlayerSaveData`) belongs to `pes_savefile` below

**`pes_savefile`** (spec: [Savefile](../pes_savefile/README.md)) — the codec and its operations, no tool
wiring:
- Crypto for all versions 15–21 (MT19937 stream cipher + PES15 LCG/MD5 chunk format, from the
  Midcupping scripts and the converters' `save*.py`; PES 18/20 master keys from pesXdecrypter
  sources; replaces `libpesXcrypter.dll`)
- The schema-driven codec: per-version field tables transcribed from 4ccEditor's
  `pes15.cpp`–`pes20.cpp`, one generic engine
- The `PlayerEntry`/`TeamEntry` model, including the tactics structures (presets, formations,
  advanced instructions); `EditFile` load/save
- `settings.toml` (`PlayerSettings`) parsing and savefile merging; FPC presets via `fpc::player`
- Cross-version player conversion (`convertPlayerSaveData` bitfield surgery + playstyle/skill maps)
- Interchange formats: Team TOML (read/write), legacy `.4ccs`/`.4cct` readers, Texport read/write
- Save-to-save operations: aesthetics transplant + aesthetics fingerprinting (Midcupping), the
  comparator (merged gameplay + aesthetics diff), FPC invisibility

**`python_bindings`** — PyO3 shim over `fmdl` and `pes_model`, built via `maturin`; proves
guardrail 4 with a real `cdylib` build and a smoke test from Blender's Python. Its Phase 2 surface
is the four codecs' entry points, enough to load the wheel from Python and run every fixture
through a byte-identical round trip; the Blender-facing accessors (vertices, faces, the `ops/`)
arrive with the `pes-models` extension's hot-path step (`model_conversion/gltf.md` "Blender
integration"), shaped by what that code calls, not guessed ahead of it.

```python
import pes_models_native as native      # the wheel; the `pes-models` extension's native module
native.fmdl.Fmdl.read(data: bytes) -> Fmdl;   Fmdl.write() -> bytes      # fmdl::FmdlFile
native.fmdl.Skl.read(data) -> Skl;            Skl.write() -> bytes       # fmdl::SklFile
native.pes_model.Model.read(data) -> Model;   Model.write() -> bytes     # pes_model::format::PreFoxModel
native.pes_model.MaterialSet.read(data) -> MaterialSet;  MaterialSet.write() -> bytes  # pes_model::format::mtl::MaterialSet
native.FormatError                             # every codec error, message = the Rust Display text
```

`write` returns what the Rust `write` returns: FMDL and SKL bytes as stored; `.model` and `.mtl`
unwrapped (the WESYS wrapping is the archive's concern, `wezlib`'s in Rust). The smoke test's
oracle is each format crate's own invariant: byte identity where the crate proves it (Konami
FMDLs, every SKL), write-idempotence (`write(read(write(read(b)))) == write(read(b))`) everywhere
else, `FormatError` on junk input, and no warning-level log line on a clean read (the wheel
installs `pyo3-log`, so the libs' `log` output lands in Blender's console).

Build rules: the crate is a workspace member (it inherits the lints and the dependency table, and
`cargo clippy --workspace` type-checks it at every PR), but its `[lib]` is a `cdylib` with `test =
false`/`doctest = false` and `pyo3/extension-module` on by default, so `cargo test --workspace`
never tries to link it against a `libpython` (a Python extension links against the interpreter
that loads it, not at build time); its one test is the wheel. `abi3-py311`: one wheel per platform
loads in every Python from 3.11 up, which covers Blender 5.0 (Python 3.11) and 5.2 (3.13) and a
developer's own interpreter; `scripts/bindings_check.py` refuses a wheel not tagged `cp311-abi3`,
because CI builds under one Python (3.13) and a wheel that lost the feature would pass there and
fail to load in Blender 5.0. `just bindings` builds the wheel with `maturin` and runs the smoke
test (`crates/libs/python_bindings/tests/smoke.py`) through `scripts/bindings_check.py`: the wheel
is unzipped onto `sys.path` of a subprocess rather than `pip install`ed, because Blender's bundled
Python has no pip and the test must run under it (`--python <interpreter>`). The wheel version is
the crate's own (`distribution.md` "One exception"). CI runs `just bindings` on both platforms.

**Verification (done):** per crate, per [libs](../libs/README.md) "Testing" and each crate's own
plan: format round-trips on real fixtures (byte-identical on the layout the game's own files use);
`color_tools` on synthetic textures painted into its region constants and on the coloured kit
template (managers' `colors.txt` proved no usable ground truth, `libs/README.md` "Testing: format parser unit tests");
`model_convert` IR round-trips, with the
19to16 converter's own output as a reference fixture; `pes_savefile` round-trips on decrypted
sections (fixed test RNG), per-version field round-trips, and transplant, fingerprint and compare
parity with the Midcupping scripts' byte rules held as literal offsets, and field values equal to
4ccEditor's own `.4ccs` export of one team per version (15–21) taken from the saves the payload
fixtures hold ([Savefile verification](../pes_savefile/verification.md)); `python_bindings` import
and round-trip from Python and inside Blender 5.0 and 5.2. The converge censuses ran every
`.model`, FMDL, SKL, FPK, FTEX and savefile on the maintainer's machine through the readers.
At the close of converge (2026-09-30): `just gates` green, 1057 tests in 41 test runs, none
failed or ignored; `just bindings` green.

### Phase 3: Team compiler skeleton (`tools/team_compiler`)

The Team compiler checks and compiles the tracer bullet's content end to end, from the command
line and from a minimal window. What `compile` accepts until Phase 4, and what it refuses with
`content_not_yet_compiled`, is the Team compiler plan's "Phase 3 scope" (in its "Acceptance");
the behavior contract is that section's TC scenarios, every automated one cited by a test. Its
entry gates (normal-team/referee `ExportIdentity`, the sanitized validated-versus-eligible
projection, roster-entry scope and disposition semantics) are in the [Aesthetics
export](../aesthetics_export/object_model.md) plan's "Validation semantics".

- **Tracer bullet first.** Before the skeleton, the smallest real export was compiled end to
  end: /egg/'s one player folder (a Fox face) and one kit, read from disk and written as a CPK
  through the Phase 2 crates (`fmdl`, `ftex`, `fpk`, `cpk`, `kit_config`). Its test,
  `team_compiler/tests/parity.rs`, compares the result against Red's output for the same source
  export by the parity tiers that apply to it (tier 1 for textures, tier 2 for the FMDL through
  `fmdl`'s decoded model, tier 3 for the containers; "Testing: parity against Red" in the Team
  compiler plan), and now runs through `compile`; it grows with Phase 4. The fixture
  (`tests/fixtures/tracer/`) is a small old-layout export migrated to the Studio layout by hand,
  since the Export upgrader (Phase 6) does not exist yet; that same pair later becomes the
  upgrader's own input/expected fixture. The tracer's scaffolding (a hardcoded walk of the one
  folder) was replaced by `aesthetics_export` and the pipeline. We did this first, not as Phase
  4's verification, because the Phase 2 crates' `pub` APIs had had no consumer: a shape wrong
  across several crates (bulk-data ownership, finding codes, how a face's FPK is assembled)
  found there is a one-crate fix, found after the skeleton is built on it is shotgun surgery
  through the tool.
- The tool's settings struct and its two CLI subcommands, `check` and `compile`, through the
  `StudioTool` trait, with the exit codes 0/1/2/3 (`team_compiler/settings.md` "CLI").
- `libs/aesthetics_export`: the source-neutral object model for the Studio export format
  (canonical `vtree::ScopePath` listing → `ParsedAestheticsExport` → sanitized
  `ValidatedAestheticsExport` → `ResolvedAestheticsExport`, with `ExportIdentity`, strong
  `ValidatedRoster`, `PlayerFolder`, `SharedModelFolder`, and `KitsFolder`) plus its folder
  conventions and the structure pass's validation rules, shared with the Export upgrader, Kit
  config editor, and Refs arranger. Tool-local `BuildManifest`, `PlanReport`, `plan_run`, and
  `process_task` stay in `team_compiler`.
- The pipeline orchestration (reader, coordinator, writer) on `rayon`, over `libs/pipeline`'s
  memory budget, thread count and `CpkStem`; the run driver's shapes are the Team compiler
  plan's "Run driver shapes (Phase 3)" (`pipeline.md`).
- CLI execution with console output (`4cc-studio team-compiler compile ...`), one line per
  finding (`core/architecture.md` "Event system").
- **Shell slice, last.** The thinnest GUI path that runs: `studio_core`'s app shell reduced to a
  window, the sidebar listing the registered tools, and the selected tool's `view()`; the
  `studio` binary launching it with `team_compiler` registered; and the Team compiler's `view/`
  reduced to its settings section, a run button and a plain log of the `PipelineEvent`s the
  Phase 3 pipeline emits (no progress grid, no live validation). Everything else in the
  "Phase 8" list (common widgets, help window, status bar, watcher, cancellation, the other
  tools' views) stays in Phase 8, which completes this shell rather than starting it. We built
  it here, not in Phase 8, because the `StudioTool` trait, the event channels and the
  render-only `view/` rule would otherwise be designed against zero real tools until five tool
  crates had been written to them; one real tool on the shell early is what shows whether the
  seam holds, and it is what Phase 10 needs to run in parallel ("once the shell exists"). The
  settings section shows the effective settings read-only (editing them is the settings menu's,
  which needs the shell to save the file); the run button runs `compile` with no arguments,
  through the same parser and preflight as the CLI, on a thread of its own, and the log ends
  with the run's exit code (Phase 3 emits no `Complete`); the binary is a console-subsystem exe,
  so a console window opens beside the GUI, and `--gui` autorun is refused: both need what Phase
  8 builds (`AttachConsole`, the initial exports check).

**Verification (done):** the tracer bullet's parity case on its one-face fixture; the
`aesthetics_export` structure pass and `team_compiler`'s `check` and `compile` against the TC
scenarios, run in-process over Studio-format fixtures (folders, a `.zip` and a solid `.7z`),
with the binary's own seams (settings location, exit codes, the console printer) run through
the built `4cc-studio`; `scripts/acceptance.py` proving every automated scenario cited by a
test; the shell slice's two manual scenarios (TC-GUI-01/02) checked by the maintainer on Windows
(Linux not yet, no machine with a display). Converge ran a whole-crate mutation run over each of
the phase's crates, every survivor triaged. At the close (2026-10-02): `just gates` green, 1402
tests in 50 test runs, none failed or ignored; acceptance 77 scenarios, 77 proven. Full
packed-output comparison belongs to Phase 4.

### Phase 4: Processing logic

**Entry gates:** finalize analyzed-output enumeration/namespace allocation, including
folder-internal deep-derived names without adding a separate run phase/type, and freeze every model
`BuildTask`'s planned model-ID assignments.

Turn the Phase 3 skeleton into a compiler that produces game-facing output for Studio-format
exports. `model_convert` (Phase 2) already exists, so cross-format conversion is in scope; savefile
writing arrives with Phase 5. The work is organized by the Team compiler's stages — see "Crate layout" and
the "Per-export pipeline walkthrough" in the [Team compiler plan](../team_compiler/pipeline.md) for the
specification of each. Red module names in parentheses locate reference *behavior*; they prescribe
nothing about the Rust code.

- **Deep validation** — content-level rules in `libs/aesthetics_export` on top of Phase 3's
  structural ones, and `fmdl`/`pes_model` `check.rs` findings, all mapped through the Team
  compiler's message catalog (`export_check`, `xml_check`, `texture_check`). Live-grid scheduling in
  `check/` waits for Phase 8.
- **`plan/`** — `BuildManifest` from a `ResolvedAestheticsExport`, `PlannedModelIds` (40-ID team
  blocks, deterministic boots/gloves assignment — assignment only; the savefile write is Phase 5),
  duplicate-refs preflight, `overrides/` precedence (`team_id_get`).
- **`processing/`** — per-task work over `rayon` with memory permits: model format selection,
  cross-format conversion via `model_convert`, multi-model merging and SKL pairing
  (`fmdl_editing`); hand auto-split (`model_convert::ops::hand_split`) on every model carrying
  `skh_*_l`/`skh_*_r` weights before categorization, the split parts becoming the player's
  `glove_l`/`glove_r` parts (`player_folders.md` "At compile time, the pipeline" step 0); model
  source selection before glTF exists: a folder whose selected representation would be glTF (no
  target-native source beside it) is refused with `model_gltf_unsupported` (folder-discarding,
  not pass-through-eligible) until Phase 7, never compiled from the opposite native format
  instead, since that output would change silently once glTF lands (`team_compiler/pipeline.md`
  "Per-export pipeline walkthrough"); texture conversion with the CPU converter, each worker
  converting one texture at a time, so the conversions in flight are bounded by the worker count
  (0.1.0 ships CPU-only; the desktop GPU BC7 backend, `libs/README.md` "First-release desktop GPU
  BC7", is built on this step in Phase 16; stopping a cancelled run between textures is Phase 8's
  cancellation, which brings the Cancel that triggers it) — plus WESYS wrapping and relocation
  to common (`textures`);
  MTL / face XML / `materials.toml` editing (`xml_editing`); kit folders: config
  generation/reconciliation/emission via `libs/kit_config` (TOML in exports, binary at compile
  time), FPC patching via `fpc`, kit colors from per-kit `colors.txt` (Team Note entry format) or
  derived from the kit texture via `libs/color_tools` when absent; portraits, logos, collars and
  Common (`portraits_move`, `name_editing`); referee processing per the Team compiler plan's
  "Referee export processing" with Red's observable output as the reference (`referee_tools`,
  referee code in `fmdl_editing`/`export_move`); `materialize.rs` as the single output-mode seam:
  relocation to game paths and Fox FPK packing (`contents_packing`, `export_move`).
- **`bins/`** — UniColor / TeamColor / UniformParameter accumulation via `uniparam`, DpFileList
  read/write and slot discovery (`bins_update`); on Fox the player appearance tables, read from
  the installed CPKs by the same walk and written whole: `BootsList.bin` and `GloveList.bin` get
  a row for every compiled player whose custom boots or gloves output committed (the planned ID),
  every other row kept, a failed output keeping the installed row; `PlayerAppearance.bin` passes
  through with no row changed. Phase 5 adds the stock and default-ID rows and the appearance rows
  from `settings.toml`: this split is the only one in which Phase 4's CPK is game-correct for what
  it compiled without reading `settings.toml`.
- **`output/`** — Phase 3 already writes one CPK through the canonical-order writer, staging and
  promotion to `output/`, with the output folder's writability preflight (TC-OUT-*, TC-CLI-06);
  Phase 4 adds `OutputSink` (CPK vs loose folder for test/sideload), the `.partial` copy into
  `download/` for deployment, multi-CPK; access-denied output paths
  fail cleanly via `libs/elevation` (the GUI's elevated-relaunch prompt is Phase 8).
  `output/savefile.rs` is Phase 5.
- `dummy_kit_replace` is not ported — `dummy_kit*` stems are reserved game-substituted names, see
  the Unified model format plan's "Kit-dependent assets".

**Verification:** Parity-style test: run old reference exports through the Export upgrader (Phase 6)
and compile; compare game-facing output (nested CPK contents, bins) against Red's reference output,
accounting for intentional differences (auto-assigned IDs). Full upgraded-export parity depends on
savefile/ID integration (Phase 5) and the Export upgrader (Phase 6), so the end-to-end parity gate
is a deferred verification completed after those phases land.

### Phase 5: Savefile integration (`tools/save_editor` logic, `libs/aatf`, Team compiler savefile writing)

`pes_savefile` exists since Phase 2; this phase wires it into its two consumers.

- Commit-conditioned savefile writing in the Team compiler (`output/savefile.rs`): `BuildManifest`
  planning freezes `PlannedModelIds` and creates conditional mutations; producer commits activate
  asset-dependent mutations; final serialization applies only activated mutations while independent
  accepted `settings.toml` changes may still apply. The allocation scheme is fixed (40-ID per-team
  blocks — see the Team compiler plan), and Phase 4 model-task processing only consumes planned
  assignments. `fpc_on`/`fpc_off` markers apply `pes_savefile`'s FPC presets here. The resolved
  settings become, on Fox, each compiled player's appearance table rows (Phase 4's `bins/`) and an
  aesthetics patch of names only; on pre-Fox, the full aesthetics patch ("Player settings in
  exports" in the Aesthetics export plan).
- `libs/aatf`: the AATF rules engine (one self-contained Rhai rules file; see the Save editor
  plan's "Configurable AATF rules").
- The Save editor tool crate's non-GUI substance: settings, CLI, and the operations wiring over
  `pes_savefile` (editing, tactics, AATF checks, comparator, transplant, FPC toggle, strip and
  seed with the restored-id check) that the Phase 8 view will render — build order in the
  [Save editor plan](../save_editor.md).

**Verification:** Team compiler parity now includes savefile output — compiled savefile compared
against 4ccEditor edits for the same assignments; AATF rule fixtures (known-violating and clean
saves); save editor operations exercised through the CLI on fixture saves. In game, on the Fox
version the cup has chosen (step 4.0's Test 1 covered PES 2021 only): a stripped and seeded save
shows every player as before; a compiled team's rows show, gloves included once FoxDen keeps the
table's value; and a file without `[appearance.unknown]` renders acceptably with the all-zero
default (otherwise the default becomes the base database row's bits).

### Phase 6: Export upgrader (`tools/export_upgrader`)

- Implement old-format export loading (the only place old-format knowledge lives)
- Implement savefile-driven ID→player mapping
- Implement player folder merging (single-user model folders) and shared folder + link file
  generation (multi-user folders)
- Implement kit restructuring (binary kit configs → `config.toml`, `colors.txt` and the `icon_<N>`
  marker from Team Note colors)
- Implement `settings.toml` generation from savefile aesthetics: every key, the save's unknown bits
  included, and names as explicit strings (a migrated player must look the same once his rows
  replace his stripped record)
- CLI: `4cc-studio export-upgrader ./old_export.zip`

**Verification:** Upgrade the full library of existing old-format exports; compile the results;
compare game-facing output against Red's output for the same exports.

### Phase 7: glTF support

Runs after Release 0.1.0 (see "Releases" above). Besides the codec, it lifts the Team compiler's
interim refusal of glTF sources (Phase 4) and enables the Export upgrader's `--convert-to-gltf`
pass (`export_upgrader.md` step 13).

- Add the `gltf` crate dependency to `model_convert` (Phase 2 built its native-format half)
- Implement `gltf_to_ir` and `ir_to_gltf` with a shared PES_bone/PES_mesh and materials.toml contract
- Coordinate with the Blender project's `pes-models` PES glTF codec (its implementation lives there)
- Test with real Blender-exported models

**Verification:** glTF → FMDL → import in Blender → re-export → compare.

### Phase 8: GUI

The shell slice from Phase 3 (window, sidebar, one tool's view, a plain event log) exists by now;
this phase completes it.

- Complete the `studio_core` app shell (sidebar, tool registry rendering, settings menu with per-tool
  collapsible sections, status bar with its three slots and static empty state, window title
  from activities and the held notice, help window with chapter tree, generated Messages topics,
  search, log-panel message links, and `4cc-studio help`), and the first-run data-location dialog
  (`distribution.md` "Data location: asked on first run")
- Build the common widgets: progress grid, log panel (auto-scroll, filters, grid cross-linking),
  run strip
- Implement each tool's `view()`: Team compiler grid (live validation), save editor
  (player/team/tactics editing, AATF checks, comparator, transplant — build order in the [Save
  editor plan](../save_editor.md); the tactics editor, card pickers and violations list are built as
  `libs/team_widgets`, which the Team creator later reuses), export upgrader
- Implement the exports folder watcher (`notify` + debouncing) and the two-tier check system with
  cache
- Wire event channels to egui rendering
- Implement cancellation (`gui.md` "Cancellation"): the Compile button's Cancel, the pipeline
  checking it between tasks and between a texture task's textures, the cancelled run's staging
  discarded
- The Team compiler's `run_pes` and sideload mode's Launch PES button as one launcher of
  `PES20{version}.exe` (`team_compiler/pipeline.md` "Post-processing", Run PES): both start the
  same exe the same way, and this phase holds its only manual check
- Test on Windows (Linux is a Phase 16 item: no Linux machine is available, and 0.1.0 is
  Windows-only)

**Verification:** Manual testing on Windows; user feedback from the modding community arrives
through Release 0.1.0, which follows.

### Release 0.1.0

Cut once Phase 8 closes. It is a milestone rather than a numbered phase: its deliverables are the
slice of Phase 16 an early release cannot do without, pulled forward.

**Ships:** the Team compiler (Phases 3–5: native FMDL/`.model` sources, CPU texture encoding,
compile-time savefile writing), the Save editor (Phase 5 logic and the full Phase 8 view), and the
Export upgrader (Phase 6, structural migration; its glTF pass waits for Phase 7), as the Windows
portable bundle. Old-format exports reach the compiler through the upgrader; Red's Pre-Studio
preview layout gets no dedicated input path, since few members are expected to use it (the
reserved-subfolder legacy support in `aesthetics_export/player_folders.md` stays as it is).

**Does not ship:** glTF (Phase 7), desktop GPU BC7, the Linux bundle, and the updater's
download/swap/rollback (all Phase 16), and the tools of Phases 9–15.

- Release bundling as the `just release <version>` recipe, Windows bundle only (`distribution.md`
  "Distribution: portable .7z bundle", "Changelog and version display"): the `.7z` with a SHA256
  checksum asset, the release body copied from the version's `CHANGELOG.md` section
- `CHANGELOG.md` started; help chapters for the shipped tools and the core chapters (Red's two
  readmes migrated into the Team compiler and core chapters); `4cc-studio help --export` wired
  into the release process
- The updater's check and notice (`distribution.md` "Self-update: in-place binary swap", step 1,
  and step 2 reduced to a notice): the release notes and a link to the release page, with *Skip
  this version* / *Remind me later*; the CLI one-line notice. Plus step 5, the teams-list
  reconciliation on the first start of a new version: it runs on a new binary's first start
  however the binary arrived, so a manual update needs it as much as the swap does
- Maintenance mode, moved here from Phase 16: the `AGENTS.md` section written as 0.1.0 is cut,
  since from this release on the released surfaces (export layout, `settings.toml`/`config.toml`
  keys, CLI flags and exit codes, finding codes) are compatibility contracts. The outline is Phase
  16's bullet. Phases 7 and 9–15 keep the phase procedure; each may ship as a `0.x` release

**Verification:** `just release 0.1.0` produces the bundle and checksum; a recorded manual check
on a clean Windows machine: the bundle extracted to a fresh folder, the first-run dialog, a
fixture export compiled through `quick_compile.bat`, and the update notice shown against a test
release with a higher tag. The release closes with converge and rewrite over its own sections.

### Phase 9: Stadium compiler (`tools/stadium_compiler`)

- Implement the fox2 parser (+ CityHash64) as `libs/fox2`
- Implement the stadium DB bin generation
- Build the stadium pipeline on the shared reader/coordinator/writer scaffolding
- Implement components, billboards, packing, ID remapping
- Implement the tool's view (reuses the progress grid: rows = stadiums, cells = components)

**Verification:** fox2 roundtrip tests (byte-identical, matching FoxTool); output comparison against
the Python stadium compiler on real stadium exports.

### Phase 10: Music tools (`tools/music_player`, `tools/music_export_editor`)

The music tools are independent of phases 2–9 — they touch none of the game-file lib crates and need
only `studio_core` and the app shell, so this phase can run in parallel with (or before) the others
once the shell exists.

- Implement .4ccm parsing/writing + the condition/instruction model as `libs/music_export`
- Build `libs/audio_engine` (kira + symphonia playback, Opus decoder, ebur128 loudness, limiter)
- Music player tool: selection logic, soundboard view, normalization, chants, match state
- Music export editor tool: editing view, condition forms, live preview, validation, audition
  playback

**Verification:** .4ccm round-trip on the community export library; selection-logic scenario tests;
loudness within tolerance of the ffmpeg reference; manual streamer/manager testing.

### Phase 11: Match tracker (`tools/match_tracker`)

Needs only the shell for the tracker itself, plus Phase 10's music player for the autopilot half.

- Transcribe the PES17 memory signatures (team data table, stats table, stat catalog) as per-version
  data; implement the Win32 memory source and the acquisition ladder behind the memory-source trait
- Build the match model: stat diffing, event derivation, clock reconstruction
- Implement `libs/match_feed` (typed pub/sub + snapshot replay) and the tracker view (stats table,
  event log, match tabs)
- Match file save/load (JSON) + autosave; `watch` CLI
- Music player autopilot: feed subscription, auto team loading, feed-driven scoring, event clips

**Verification:** replay tests on recorded table snapshots (including ET/penalty matches);
clock-reconstruction unit tests; scripted-feed autopilot tests; a live simulated stream with
autopilot on.

### Phase 12: Kit config editor (`tools/kit_config_editor`)

Small: the format logic already exists in `libs/kit_config` from Phase 2/4 — this phase adds the GUI
form, the preview, and the container-editing mode.

- Editing form with constraint gating; export-folder tabs and loose-file mode
- Schematic + texture-composite preview (via `dds_convert`)
- UniformParameter container editing (via `uniparam`); `convert`/`check` CLI

**Verification:** bit-identical roundtrips over the bundled UniformParameter contents; byte-parity
edits against Kit Manager on the same file.

### Phase 13: Refs arranger (`tools/refs_arranger`)

Small: the export knowledge already lives in `libs/aesthetics_export` — this phase adds
the slot-allocation editor and the players.txt writer. Needs Phase 3 (`aesthetics_export`)
and Phase 8 (the shell).

- Slot appearance-rate tables as per-version constants (flat rates for PES 17–21, pattern table for
  PES 15/16)
- Slot-list editor with drag-and-drop from the referee storage box; read-only pattern preview with
  hover cross-linking (15/16 mode)
- Lists mode for PES 18–21 (the Fox referee hook decides who appears): one home slot per referee,
  ordered five-position lists built by hand or randomized, the weighted fallback fill of the
  leftover slots, `ref_lists.txt` writer
- players.txt writing (overwrite confirmation) + Team compiler handoff (`ToolContext` tool switch);
  `chances` and `randomize` CLI

**Verification:** players.txt and ref_lists.txt round-trips through `aesthetics_export`; chance-math
unit tests against the measured tables; randomizer and fallback-fill invariants;
interrupted/observed atomic replacement tests (including Windows); a refs export arranged, saved,
and compiled by the Team compiler.

### Phase 14: Balls compiler (`tools/balls_compiler`)

Small: the heavy machinery (FPK/CPK packing, DDS↔FTEX, FMDL path rewriting, IR
conversion) exists as lib crates by this point. Needs Phase 2 (format libs and `model_convert`
for native conversion), Phase 7 (glTF support), and Phase 8 (the shell).

- Balls export parsing + validation (in-tool; the format has one consumer)
- `Ball.bin` writer + embedded `BallCondition.bin`/`ball.fpkd` templates
- Per-ball pipeline: model selection/conversion, texture conversion, FPK packing, thumbnails, CPK
  output
- List editor view (storage box + ordered list, thumbnails, utility buttons); `import-legacy` CLI

**Verification:** output comparison against 2.03 on the same library (accounting for the intentional
drops: no `name.txt`/`ball.fpk.xml` in the CPK); `Ball.bin` byte-parity for identical lists;
interrupted/observed atomic `balls.txt` replacement tests (including Windows); shared `CpkStem`
boundary tests; in-game menu check.

### Phase 15: Player aesthetics editor (`tools/player_aesthetics_editor`)

Small and non-essential (a convenience tool — schedule freely once its
dependencies exist): Phase 2 (format libs, `model_convert`, `pes_savefile` settings and FPC
presets), Phase 3 (`aesthetics_export`), Phase 7 (glTF support), and Phase 8 (the shell). The dual-set superset merge lands here if not already built. The Blender-side
counterpart (the `pes-models` extension's loader module) is the Blender
project's deliverable, not this workspace's; the launch manifest is the
contract between them.

- Plain-folder player browser/launcher + manifest writing + Blender launch; archives require
  explicit extraction before launch, editing, or conversion
- settings.toml editing panel (comment-preserving via `toml_edit`) + `fpc.*`/`ingame_face` marker
  toggles; missing settings stay unset until the user edits them
- In-place glTF conversion (superset merge, app-data backup + restore); `launch`/`convert` CLI
- Base-model extraction from installed CPKs + bundled glTF stand-ins

**Verification:** golden-manifest tests; superset-merge field-provenance tests; transactional
conversion publication and backup/restore; comment preservation; compiled-output comparisons on both
engine targets against the chosen geometry and preserved compatible metadata. A dual-set merge
cannot retain both original geometries when they differ; verify that deliberate choice separately
from unintended conversion loss.

### Phase 16: Polish and distribution

Closes with the **1.0.0** release. Release 0.1.0 already delivered the Windows `just release`
recipe, `CHANGELOG.md`, the first help chapters, the update check and notice with the teams-list
merge, and the maintenance-mode section; this phase completes them.

- Release bundling: the Linux bundle joins the `just release <version>` recipe (see
  `distribution.md` "Distribution and updates", `distribution.md` "Changelog and version
  display"); Linux tested on a real machine
- Self-updater, the rest of it: the *Update now* option, download/verify/swap, `4cc-studio
  update`, rollback button (see `distribution.md` "Distribution and updates")
- Desktop GPU BC7 on Phase 4's texture step (`libs/README.md` "First-release desktop GPU BC7"):
  cold pipeline creation and upload/readback measured first, then bounded batches, the Auto/CPU
  setting and the fallback report
- Documentation: every shipped tool's `help/` chapter written for members (a tool shipping in a
  `0.x` release carries its chapter from that release); the bundled readme stays a pointer (the
  window itself is Phase 8)
- Maintenance mode: the `AGENTS.md` section on how the project changes once released is written
  at Release 0.1.0 (not earlier: a procedure written far ahead of its use goes stale) and reviewed
  here for 1.0.0. Agreed outline: the plans stay the spec, edited in place, and `DECISIONS.md` stays the
  decision (ADR) log, split per year past about 4-5k lines, with no per-file ADRs or ADR
  tooling. A fix is a failing regression test first, a plan edit only if behavior changes, and
  `mutants-diff`. A feature runs the phase procedure at small scale (acceptance scenarios,
  converge, present-tense rewrite), as Phases 17-18 already do. Released surfaces become
  compatibility contracts (`settings.toml`/`config.toml` keys, CLI flags and exit codes, the
  export layout, finding codes, `.4ccm`): breaking one needs an Export upgrader migration or a
  clear refusal, never a silent reinterpretation. Converge runs per release over the commits
  since the last tag: the plan sections they touched, and `mutants-diff <last tag>`

### Phase 17: Team creator (`tools/team_creator`, post-release)

Not a first-release gate: its users are the *next* cup's new managers, and it depends on the
widest set of finished pieces — `libs/team_widgets` and `libs/aatf` with `apply_tier` (Phases 5
and 8), the compiler's kit placeholder (Phase 3, its UniColor entry Phase 4) and layout-marker rules (Phase 4), `aesthetics_export`
writing, and the Player aesthetics editor to hand off to (Phase 15). Plan: [Team
creator](../team_creator.md).

- The wizard (team, players, roles, cards, looks, kits, finish) and its help chapter written
  from the wiki's newcomer pages
- Roster file grammar; `legal_defaults` (positions from a stock formation, heights to bracket
  quota, weights, captain, set pieces) over `aatf::apply_tier`; stock formation constants
- The four embedded starter heads in Studio format (in-game head, two cardheads, boxhead)
- Export + Team TOML + README writer (temp sibling, renamed into place, never over an existing
  folder); `create` CLI; hand-offs to the four tools

**Verification:** `legal_defaults` output passes `libs/aatf` for every stock formation and both
height systems; a created export compiles on both engines with only the expected placeholders; the
Team TOML imports into a fixture save and passes the Save editor's AATF check; each starter head
verified in-game on PES 17 and PES 21; wizard and CLI produce identical files for identical inputs.

### Phase 18: Studio Web (post-release)

Not a desktop release gate. Ships once the desktop pipeline has settled, in two steps that match
the eligibility tiers in `gui.md` "Browser deployment":

1. **Cheap tier**: the eframe `wasm32` build of the shell plus the file-picking tools (Save editor,
   Kit config editor, Music export editor, Export upgrader checks). Needs the async file-access
   adapter (`rfd` file picking + a File System Access API adapter for folders), browser-storage
   settings persistence, and hiding of the desktop-only tools. This step alone is the "open a
   bookmark and edit your team" demo.
2. **Pipeline tier**: single-team compilation. Needs the `libs/pipeline` browser variants (async
   reader/writer, async `MemoryBudget`, `wasm-bindgen-rayon` workers), the small in-flight limit, and
   COOP/COEP headers on the host.

Hosting on the community server (nginx, HTTPS, two headers) is evaluated at step 1; CI has been
`cargo check`ing the lib crates for `wasm32` since Phase 1 (guardrail 6), so this phase is UI
wiring and adapters, not porting.

**Verification:** the cheap-tier tools' file round-trips in Chromium and Firefox; a single-team
compile in Chromium producing the same CPK as the desktop CPU path on the same export; embedded
resource download size measured and, if it matters, lazy-fetched behind the template accessor.

### Phase 19: DB generator (`tools/db_generator`, `libs/pesdb`)

Small, and scheduled by need rather than by number: it depends only on Phase 2 (`teams_list`,
`cpk`, `pes_savefile`) and Phase 8 (the shell), and a cup start is when it is wanted. Until then
the Python scripts work. Plan: [DB generator](../db_generator.md).

- `libs/pesdb`: the six database tables the generator writes as typed per-version records
  (`Team`, `Player`, `PlayerAppearance`, `PlayerAssignment`, `Coach`, `CompetitionEntry`), the
  blank-table and Konami-table lists per version; `Ball`/`BallCondition` and `Stadium` join when
  Phases 14 and 9 need them (the Ball.bin writer planned inside the Balls compiler moves here)
- `pes_savefile::ops::populate`: the 19+ placeholder player section written through the codec
  (the scripts' hex-editor procedure), with the base player per version as a fixture
- The tool: teams list → tables (Backup/VGL/Invitational placeholder rows classified into their
  competitions), Konami tables copied from the configured install's data CPK, output as tree or
  CPK, the populated EDIT; `generate` CLI; the view

**Verification:** byte parity with the scripts' output on a three-team list per version (their
trees committed as fixtures); `pesdb` round trips on real Konami tables; populating the PES 20
fixture's stripped player section reproduces it byte for byte; in-game start with a generated
database and EDIT (manual, per version).

---
