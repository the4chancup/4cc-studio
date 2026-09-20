# 4cc Studio — Core plan

This is the core architectural plan for 4cc Studio: the goals, the platform (crate
structure, tool plugin system, GUI shell, parallelism), and the cross-cutting
decisions. The tools and the major library crates have their own plan documents, listed in the
[plan index](../README.md).

---

The plan is split into parts; section headings are unchanged from the
single-file plan.

| Part | Covers |
|---|---|
| [Architecture](architecture.md) | Architecture |
| [GUI design](gui.md) | GUI Design |
| [Parallelism](parallelism.md) | Parallelism |
| [Distribution and updates](distribution.md) | Distribution and updates |
| [Development plan](development_plan.md) | Development Plan |

## Overview

4cc Studio is a Rust-based **suite of all the tools used by the 4cc community**, built around two
tightly integrated main tools:

1. **The Team compiler** (successor to the AET Compiler) — compiles aesthetics exports into CPK archives,
   with compile-time model format conversion and compile-time savefile writing
2. **The save editor** (successor to 4ccEditor, absorbing the Midcupping scripts) — edits PES save
   files (`EDIT00000000`): player stats, appearance, teams, tactics, AATF rule enforcement, save
   diffing, aesthetics transplant

The tight integration between the two is a primary design driver: aesthetic settings that currently
live only in the savefile (boots IDs, gloves IDs, physique, strip style, etc.) move into the
aesthetic exports as text files (TOML) and are **written to the savefile at compile time**. This
eliminates the manual coordination between the save editor and the compiler, and enables automatic
boots/gloves ID assignment.

The remaining tools are listed in the plan table above; each plan carries its own scope and
rationale. Structural points worth noting here: the **Stadium compiler**'s codebase contributes the
in-process DDS conversion that eliminates the texconv.exe dependency; the **Export upgrader** exists
because the Team compiler only supports the Studio player-folder format; the three stream-side tools
(Music player, Music export editor, Match tracker) operate on audio and game memory rather than game
files, sharing only the platform and their own lib crates — the tracker feeds live match events to
the Music player, letting it run the match soundboard autonomously; the **Refs arranger** prepares
referee exports' `players.txt` and hands the compile off to the Team compiler, which owns referee
compilation; the **balls export** is the suite's third export kind, compiled by the Balls compiler
into its own CPK; and the **Player aesthetics editor** hands model viewing to Blender (launched with
a manifest consumed by the community's all-in-one `pes-models` extension) instead of embedding a 3D
preview — it only gains a labeled approximate in-app preview if the Team compiler's future
3D-preview feature creates the shared `model_viewport` lib; and the **Team creator** is a wizard
that takes a name and a list of players to a legal Team TOML plus a compilable aesthetics export
built from embedded starter heads — it owns no format and keeps no state, writing only files the
other tools already read, and shares its tactics and card widgets with the Save editor through
`libs/team_widgets`. Additional tools (CPK toolkit/decompiler; other utilities) will build on the
same lib crates and be specified later.

Model format conversion (pre-Fox ↔ Fox) is **not** offered as a standalone tool — it is integrated
into the Team compiler, which converts models to the target PES version's format at compile time.

The plan documents form a standalone architectural plan derived from a feasibility analysis of the
existing codebases.

**Relationship to legacy tools:** 4cc Studio is an independent reimplementation of their useful
ideas and required observable results, not a translation of their source. Its implementation,
internal data layout, module organization, and UI are designed anew around this plan. Legacy file
and function names identify behavioral/format evidence, not code or application structure to copy.
Required PES binary formats and game-facing output conventions remain compatible; a new internal
layout does not mean changing formats the game must read. Normal third-party library reuse remains
appropriate, with its own dependency/license checks.

### Project context

The 4cc AET Compiler is a modding tool for the Pro Evolution Soccer (PES) game series (PES 15–21).
It takes user-submitted "exports" (folders, `.zip`, or `.7z` archives containing team assets —
faces, kits, boots, gloves, portraits, logos) and compiles them into CPK archive files that the game
can load.

Three prior compiler implementations exist:

| Version | Language | Status | Location |
|---------|----------|--------|----------|
| Original | Batch file | Obsolete, rudimentary | — |
| Red | Python | Mature, feature-complete; direct batch translation, works by moving files on storage | `4cc-aet-compiler-red` |
| Blue | Python (free-threaded 3.14t) | LLM-made prototype of an in-memory parallel pipeline; missing referee support | `4cc-aet-compiler-blue` |

All three are stopgaps towards 4cc Studio: they define *what* the compiler must produce (Red's CPK
output is the golden standard for parity), not *how* the new implementation works (see "Relationship
to the older compilers" in the [Team compiler plan](../team_compiler/README.md)).

The save editor exists as a Win32 C++ application, accompanied by a set of standalone savefile
scripts:

| Tool | Language | Status | Location |
|------|----------|--------|----------|
| 4ccEditor | C++ (raw Win32) | Mature; PES 2015–2021 | `Tools_4cc/4ccEditor-1` |
| Midcupping | Python | Aesthetics transplant + aesthetics diff scripts (PES 15/16/17); reference container-crypto implementations | `Tools_4cc/Midcupping` |

Some standalone format converters also exist:

| Converter | Direction | Location |
|-----------|-----------|----------|
| 4cc-aet-converter-19to16 | FMDL (PES 19) → .model (PES 16) | `Tools_4cc/4cc-aet-converter-19to16` |
| aes_converter_16to21 | .model (PES 16) → FMDL (PES 21) | `Tools_4cc/aes_converter_16to21` |
| 4cc-model-simplifier-15 | .model (PES 16/17) → .model (PES 15): folds weights of bones PES15 lacks or poses differently (absorbed by the Team compiler's per-version skeleton retargeting) | `Tools_4cc/4cc-model-simplifier-15` |

A standalone stadium compiler exists (Python, three-stage pipeline like Red):

| Tool | Role | Location |
|------|------|----------|
| PES Stadium Compiler | Compiles stadium exports into CPKs (fox2, stadium DB bins, in-process DDS conversion) | `Tools_4cc/pes-stadium-compiler-v1.0.0` |

A stream-side music player/editor pair exists (Python/tkinter, playback via libmpv, loudness
analysis via a bundled ffmpeg.exe):

| Tool | Role | Location |
|------|------|----------|
| Rigdio | Match-day goalhorn/anthem/chant player for streamers (.4ccm music exports) | `Tools_4cc/rigdio` |
| RigDJ | GUI editor for .4ccm files (same codebase) | `Tools_4cc/rigdio` |

A stream-side match stat extractor exists (C++/Qt, dormant but the only map of the game's in-memory
match structures):

| Tool | Role | Location |
|------|------|----------|
| SEN:P-AI | Live PES17 match stats/events via process-memory reading; fed Rigdio's event clips until Rigdio dropped the link in v1.11 | `Tools_4cc/SEN_P-AI` |

Two Blender addons provide import/export for the community:

| Addon | Format | Location |
|-------|--------|----------|
| pes-fmdl | FMDL | `Blender/2.79/scripts/addons/pes-fmdl` |
| pes-model | .model | `Blender/2.79/scripts/addons/pes-model` |

4cc Studio consolidates all projects into a single Rust application with a GUI.

### Language choice: Rust

Rust was chosen over C# and Python after evaluating:

- **Binary format parsing** (the bulk of the codebase): Rust's `binrw` derive macros provide typed,
  declarative readers and writers. The compiler checks their types, not whether offsets, widths,
  or endianness match PES files; fixtures and independent field checks establish that.
- **Performance**: The mesh splitting algorithm in the converters is O(V×F) graph partitioning with
  set operations in the inner loop. Python takes 1–5 seconds per complex model; Rust estimates
  10–50ms. This is essential for compile-time format conversion.
- **Single binary distribution**: eliminates the embedded Python runtime and separate conversion
  executables. Final binary size and platform system-library requirements are measured at release;
  optional Blender viewing still requires Blender and its extension.
- **LLM-assisted development**: The compiler strictness + LLM iteration loop is maximally productive
  for binary format parsing, which is the project's core workload. The object model friction (parent
  references, ownership) is a one-time cost that LLM assistance reduces.
- **True parallelism**: No GIL, no free-threaded Python workarounds. `rayon` work-stealing replaces
  `ThreadPoolExecutor` + `console_lock`.
- **Linux support**: Native.

### GUI framework choice: egui

egui was chosen over Tauri after evaluation. egui is a pure-Rust immediate-mode GUI that renders
directly to a GPU surface via `wgpu` (pinned over the alternative `glow`/OpenGL backend). The suite
also uses wgpu for first-release desktop GPU BC7 and the future 3D preview's `egui_wgpu` render
callback — see the [Team compiler plan](../team_compiler/README.md). No webview, no HTML/CSS/JS.

Key reasons:
- **Proportionate to the UI scope**: the application is a utility tool (grid of colored squares, log
  viewer, settings forms, sidebar). A browser engine for that is overkill.
- **Zero external dependencies**: no WebView2 runtime on Windows, no WebKitGTK on Linux. One binary,
  works everywhere, instant startup.
- **Single language**: the entire codebase is Rust. No second technology stack (web frontend) to
  maintain alongside Rust.
- **Resource goal**: avoid a separate webview process alongside PES. Measure actual idle and
  processing memory on the intended machines; egui/wgpu does not guarantee a particular RAM budget.
- **Cross-platform consistency**: identical GPU rendering on all platforms. No webview rendering
  differences between Windows and Linux.
- **Native HiDPI support**: `egui-winit` applies the OS scale factor automatically (live updates
  when dragged between monitors of different pixel densities; `devicePixelRatio` on the web), with a
  persisted user zoom (Cmd/Ctrl + +/-/0) layered on top. No custom DPI code is needed.
- **Browser deployment path**: the same Rust codebase compiles to a browser-served WASM build,
  Studio Web (see `gui.md` "Browser deployment: Studio Web"). Tauri cannot serve to browsers — its web
  frontend is coupled to the Tauri runtime.

Trade-offs accepted:
- **No CSS transitions**: cell color changes are instant rather than smoothly fading. egui's
  built-in animation helpers (`ctx.animate_value_with_time` / `animate_bool`) drive the fade in
  the update loop; `egui_animation` adds easing helpers if the built-ins are not enough.
- **Styling is code, not CSS**: egui's default look is plain, so the suite ships a curated theme
  instead of defaults. One shared `Visuals` preset in `studio_core` (accent color, `CornerRadius`,
  `Shadow`, per-widget style overrides) applied by every tool, icon glyphs from the `egui-phosphor`
  icon font for sidebar/toolbar/status icons, custom fonts via `FontDefinitions`, and rounded,
  shadowed `Frame` chrome for panels and cards. A 2026 re-evaluation of the Rust GUI landscape
  against this suite's requirements (pure-Rust desktop rendering, same-codebase WASM, embedded
  wgpu viewport, ecosystem maturity for LLM-assisted coding) reconfirmed egui; the visual gap is
  closed with this theme work, not a framework change.
- **3D preview requires `wgpu`**: the future model preview viewport (see Future Features in the
  [Team compiler plan](../team_compiler/README.md)) would use `wgpu` rendering alongside egui rather than
  Three.js in a webview. More work, but it's a post-MVP feature.
- **Log viewer performance**: egui's text rendering can be slow with thousands of lines. Mitigated
  by capping displayed lines to the last few hundred (full logs go to disk regardless).

---

## External Dependencies (Rust crates)

| Crate | Purpose | Maturity |
|-------|---------|----------|
| `binrw` | Binary format parsing with derive macros | Production-ready |
| `rayon` | Parallel iteration (work-stealing thread pool) | Production-ready |
| `crossbeam-channel` | Write queue between coordinator and writer | Production-ready |
| `flate2` | zlib compression (WESYS format) | Production-ready |
| `sevenz-rust2` | Native .7z reading (replaces 7z.exe); the maintained fork of `sevenz-rust`, which RUSTSEC-2026-0246 marks unmaintained | 0.22.2 in use, no default features (decode only: LZMA, LZMA2, BCJ) |
| `zip` | Native .zip reading | 8.6.0 in use, no default features plus `deflate` |
| `gltf` | glTF parsing | Production-ready |
| `serde` + `serde_json` | Serialization (settings, IR, events) | Production-ready |
| `clap` | CLI argument parsing | Production-ready |
| `thiserror` | Ergonomic error enums for the lib crates' error types | Production-ready |
| `anyhow` | Error propagation in tool crates and the binary (`StudioTool::cli_run` returns `anyhow::Result`) | Production-ready |
| `log` | Diagnostic logging facade in every lib and tool crate (see `architecture.md` "Diagnostic logging") | Production-ready |
| `env_logger` | CLI-mode log sink in `studio` (`default-features = false`) | Production-ready |
| `pyo3` | The Python extension module of `python_bindings` (`abi3-py311`, `extension-module`); built into a wheel by `maturin`, a developer tool installed with pip like `just` is with cargo | Production-ready |
| `pyo3-log` | Forwards `log` lines to Python's `logging` in `python_bindings` | Production-ready |
| `eframe` (egui) | GUI framework (pure Rust, immediate-mode, native + WASM) | Production-ready |
| `egui-phosphor` | Icon font glyphs for the shell's sidebar, toolbar, and status icons | Release not yet reviewed; pinned to the workspace egui version once selected |
| `egui_commonmark` | Markdown rendering for the help window (CommonMark subset: headings, lists, tables, code blocks, links) | Release not yet reviewed; pinned to the workspace egui version once selected. Chosen over a hand-rolled renderer (see `gui.md` "Help window") |
| `egui_animation` | Easing helpers for cell-color fades and panel transitions (on top of egui's built-in `animate_*`) | Release not yet reviewed; dropped if the built-in helpers suffice |
| `md-5` | FPK checksums, PES15 save integrity hashes | Production-ready |
| `phf` | Compile-time static maps (skeleton data) | Production-ready |
| `unicode-normalization` | NFC normalization for `vtree`'s collision detection (two spellings of `é` are one file name on disk) | Production-ready (0.1.25 in use) |
| `roxmltree` | Read-only XML tree for `.mtl` material sets, `face.xml`, `face_diff.xml` (`pes_model`, the Team compiler); writing those small fixed shapes is done by hand so Konami's formatting is reproducible | Production-ready (0.21.1 in use) |
| `nalgebra` | Matrix operations (bone transforms) | Production-ready |
| `rfd` | Native file/folder dialogs | Production-ready |
| `ureq` (3.x; features `rustls` [default], `platform-verifier`, `json`, `win-system-proxy`) | HTTP for the desktop updater: one Releases API GET and one streamed asset download per release | Production-ready (3.4.1 verified 2026-09-10). Chosen over `reqwest` because `reqwest` starts a Tokio runtime internally even in blocking mode, which breaks the "no async runtime" rule; the updater needs nothing `ureq` lacks. `platform-verifier` trusts the OS certificate store; `win-system-proxy` honors Windows proxy settings, env vars cover Linux |
| `sha2` | SHA256 verification of downloaded updates | Production-ready |
| `directories` | User config dir resolution (data-location choice) | Production-ready |
| `toml` | Export text formats (`settings.toml`, `config.toml`, `materials.toml`), app settings | Production-ready |
| `toml_edit` | Comment/formatting-preserving edits to all auto-generated tomls (`settings.toml`, `materials.toml`, `config.toml`); comments are app-injected (predefined per-field documentation) and preserved across edits | Production-ready (the `toml` crate's own foundation) |
| `rhai` | AATF rules file (parameters and check logic; sandboxed, see the Save editor plan's AATF section) | Production-ready |
| `image` | Raster source decoding (PNG, JPEG, BMP, WebP, TGA, TIFF) for texture conversion — all formats interchangeable as sources for any model format | Production-ready (pure-Rust default formats, rayon-enabled) |
| `block_compression` | CPU BC1/BC3/BC7 encoding and BC1–BC7 decoding (one crate for both directions); first-release desktop GPU BC7 | 0.10.0 in use (features `bc15`, `bc7`; `wgpu` feature only for the GPU step); decode verified exact against texconv; GPU cold-start cost, throughput and fallback still to be verified |
| (none: `fox2` ports CityHash64 1.0.3 directly) | fox2 string hashes must match the C# tool's embedded 1.0.3 variant exactly; no crate pins that variant, and the port is 150 lines verified by a reference golden and every fixture's string table | Not a dependency |
| `notify` + `notify-debouncer-full` | Exports folder watching for live validation | Production-ready |
| `kira` | Audio playback (mixer, tweens/fades, loop regions, effects; native + WASM) | Production-ready |
| `symphonia` | Audio decoding (MP3, Vorbis, FLAC, AAC/M4A, WAV) | Production-ready |
| pure-Rust Opus decoder (e.g. `opus-decoder`) | Opus decoding, registered into symphonia (its own Opus support is unfinished) | Verify maturity (fallback: statically linked `audiopus`) |
| `ebur128` | EBU R128 loudness measurement (replaces ffmpeg volumedetect) | Production-ready |
| `windows` | Win32 APIs: `elevation` (token query, `ShellExecuteExW`), later the match tracker's memory reading | 0.62.2 in use, `Win32_Foundation`, `Win32_Security`, `Win32_System_Registry` (a `SHELLEXECUTEINFOW` field), `Win32_System_Threading`, `Win32_UI_Shell`, `Win32_UI_WindowsAndMessaging`; Windows targets only |
| `libc` | `geteuid` for `elevation` on POSIX | 0.2.189 in use; Unix targets only |
| `qrcode` | QR timestamp widget (match tracker stream-sync aid) | Production-ready |

No frontend dependencies — the entire GUI is Rust, compiled to both native and WASM from the same
codebase.

The Rust code will be more verbose per line than the Python/C++ it replaces across the source
projects, but dramatically less duplicated — especially with the IR eliminating the N×N converter
pattern and the schema-driven save codec collapsing 4ccEditor's six per-version files.

---

## Key Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Binary parsing ergonomics, performance for mesh splitting, single binary, LLM + compiler strictness loop |
| GUI framework | egui (`eframe`) | Pure Rust, zero dependencies, compiles to native + WASM; proportionate to a utility tool's UI scope |
| Object model | Typed structs with enums | Compile-time exhaustiveness, no string-extension dispatch |
| Parent references | Pass resolved identity/context, not parent references | Avoid ownership coupling; validated identity is not an untyped team-ID string |
| Model conversion | All cross-format through IR | One decode/encode per format, not N×N; lossless roundtrip tests |
| Same-format paths | Skip IR unless an explicitly planned IR operation is required | Hand auto-split and pre-Fox `ingame_face` merging are exceptions; ordinary native processing stays IR-free |
| Universal model source | glTF (`.glb`, single file; `.gltf`+`.bin` accepted) with *optional* PES extensions (PES_bone/PES_mesh) + materials.toml — own plan: [Unified model format](../model_format.md) | Blender exports natively, mature Rust parser, extensible; material properties in comment-friendly TOML, not uncommented JSON. Design goal: one authored model converts to `.model` and FMDL with no loss of customizability in the simplest possible way, or users keep the native formats. Stock-Blender exports must compile (extensions have fallbacks, PBR material data is ignored), which is why a custom `.pes` extension was rejected — it would make the plugin mandatory |
| glTF material files | `materials.toml` (catch-all) + `*.materials.toml` (name-matched) + `.common` links to Common's files, mirroring `.model`/`.mtl` and Red's `.mtl.common` | Materials never embedded in glTF JSON — glTF files need textures anyway, JSON has no comments, and tomls-as-siblings make sharing unambiguous; name-matching (startsWith/endsWith) ports Red's `.mtl` logic for the same modular copy-paste workflow; Common-linked models resolve materials in Common first with local overrides layered on top (Red's cascade); missing material = hard error (no safe defaults without texture paths) |
| Material schema | Shader `family` + engine-neutral keys (booleans, canonical texture roles, parameters) + optional `[fox]`/`[prefox]` native tables | `[face]` alone is a valid material (family `shaded`, textures auto-detected by name); families are named as laymen know them (`shaded`/`shadeless`, not `blin`/`constant`) and encode the converters' shader mapping tables so both engines get a working material from one word; native tables keep same-format round-trips lossless and expose every knob; unknown keys warn (forward-compatible), invalid values error |
| Blender integration | Two complementary paths, packaged as one all-in-one extension (`pes-models`) | Path A: vendored `pes-fmdl`/`pes-model` 5.2 port codecs, hot paths accelerated by the `python_bindings` PyO3 wheel (lossless same-format round-trips, pure-Python fallback). Path B: PES glTF codec (native glTF I/O + `materials.toml` read/write + `PES_bone`/`PES_mesh` extension handling) for new authoring targeting both engines. Plus the Player aesthetics editor's loader module; replaces the standalone addons at release (plan lives with the Blender project) |
| Mesh splitting | CPU only (Rust) | GPU unsuitable for branch-heavy graph partitioning; Rust is fast enough |
| Texture conversion | In-process (`texture2ddecoder` + `image` + `block_compression` CPU/GPU backends + bounded cache) | BC7 supported only on PES 19–21, with first-release desktop GPU acceleration in Auto mode and CPU fallback/explicit CPU mode. BC7/raster sources for 15–18 use CPU BC3 or eligible opaque-color BC1. Codec, alpha, cache, and fallback policy live in [libs](../libs/README.md) |
| Texture references | Stem-based (input); `.dds` extensions on output `.mtl` | All image formats (DDS, FTEX, PNG, JPEG, BMP, WebP, TGA, TIFF) are interchangeable as input sources for any model format; FMDL path tables were already stem-based; Studio-format `.mtl` and `.materials.toml` write stems; output `.mtl` (game format) keeps `.dds` extensions; two image files with the same stem but different extensions is `texture_stem_conflict` |
| Stadium compiler | `tools/stadium_compiler` crate | Ports the Python stadium compiler; fox2 parser goes in `libs/fox2` |
| Model conversion exposure | Shared `libs/model_convert`, used by compilers and glTF migration actions | No separate conversion-only tool; consumers include Team/Balls compilers, Export upgrader, and Player aesthetics editor |
| GUI validation | Live, no Check/Refresh buttons | Folder watcher + deep checks for extracted folders, shallow (blue) for archives |
| GUI actions | Single Compile button (becomes Cancel) | Staged operations superseded by the unified pipeline |
| Old export format | Not supported by the compiler | Migrated once via the Export upgrader tool; keeps the compiler simple |
| Export upgrader | Dedicated tool crate | Merges single-user model folders into player folders; shared folders + link files; savefile-driven ID mapping |
| Shared model folders | Name-based, no embedded IDs | Player folders are the reference point; link files (`Name.boots`); IDs auto-assigned at compile time |
| Kits format | `Kits/` folder, one subfolder per kit (`<slot>[ - <label>]`) plus an optional `all/` of textures every kit inherits per stem; an empty kit folder is a placeholder kit (bundled magenta/black checkerboard, template config) for full-body-model teams; an optional `pre-fox` / `fox` marker file declares which engine's kit layout the main texture is drawn for, and the compiler re-lays out the sock and shorts islands when the target is the other engine | `config.toml` + optional `colors.txt` (Team Note color format; derived from the kit texture when absent, else the loud magenta/black pair) + optional `icon.txt` + textures per kit; drives per-kit grid cells |
| 7z extraction | Native `sevenz-rust` crate | Eliminates subprocess and temp-folder dance |
| Compression | Archive-level only | Exports already submitted as .zip/.7z; no format-level compression needed |
| Testing | Upgrader+compile parity + IR roundtrips; CPU reproducibility + GPU quality checks | CPU mode is the byte-reproducible reference. GPU texture bytes and containing CPKs may differ across devices/drivers; decoded quality and all non-texture identities/ordering remain constrained |
| Suite scope | Team compiler + save editor first | Shared `libs/pes_savefile` crate; other tools build on the same libs later |
| Crate architecture | Minimal core + tool crates + lib crates | `studio_core` = shell/trait/settings only; every tool its own crate; all shared functionality in lib crates |
| Tool crate granularity | One crate per tool, no companion libs | Even the Team compiler (15–25k lines) stays one crate with internal modules; only genuinely shared parts move down — hence `libs/aesthetics_export` (compiler + upgrader + kit config editor + refs arranger + future tools), not a single-consumer `team_compiler_lib` |
| Refs tooling | Arranger + handoff, no engine split | The Refs arranger edits the refs export's `players.txt` (via `aesthetics_export`) and switches to the Team compiler for the compile — a compile-engine lib crate would have one thin extra consumer |
| Balls tooling | Standalone compiler + balls export | Ball content shares nothing with aesthetics exports (own paths, bins, CPK), so the Balls compiler owns its compile on the shared libs; balls become the third export kind in `exports/` (first word `balls`, skipped by the Team compiler) |
| Tool modularity | Tools own their GUI, settings, CLI, and help | `StudioTool` trait: view, settings section (injected into settings menu), clap subcommand, help chapter (injected into the help window, Messages topic generated from the catalog) |
| CLI shape | `studio <tool-id> <command>` subcommands | Standard clap subcommands; better help/completion than flag-style dispatch |
| Distribution | Portable `.7z` (binary + `quick_compile.bat` + exports/ + teams list + readme), no installer | Extract anywhere and run; no elevation, no registry. Extract-and-run with no external runtime is the requirement, not file count — Red's many-file bundle causes users no trouble |
| License | `MIT OR Apache-2.0`; game-derived data excluded and named as Konami's | The suite is libraries first, for a community that writes tools in every language; copyleft would confine the libs to GPL consumers for a protection this community has never needed. See `distribution.md` "License" |
| Quick-compile launcher | `quick_compile.bat` → `studio --gui team-compiler compile`; GUI-subsystem exe + `AttachConsole` for CLI output; `exports/` beside the exe in both data modes | Red's `0_all_in_one.bat` is the one script users actually run; the GUI autorun keeps that double-click habit while showing the grid/log instead of a console. `gui_run` trait hook reuses the tool's CLI grammar, so no second command surface |
| Studio Web | Post-release lite browser edition, not a first-release gate; lib crates `wasm32`-checked in CI from Phase 1 | Convenience for compiling/editing one's own team from another PC via a bookmark, never full-cup compilation (4 GB `wasm32` ceiling); the cheap file-picking tier ships first, the pipeline tier needs adapters + COOP/COEP hosting on the community server |
| Data location | First-run choice: `data/` next to the exe (fully portable) or the user config dir; presence-based resolution, no marker files | EGG-Translator's wizard mechanism; portable users get delete/move-the-folder semantics, config-dir users survive program-folder replacement |
| Self-update | In-place binary swap (move-running-exe pattern) + teams-list merge; previous binary kept in `old/` with a rollback button | Inverts Red's sibling-folder update: the folder is a few files, user data stays put; settings need no transfer (default-merge handles new keys); the `old/` subfolder keeps the previous binary out of misclick range |
| Save codec | Schema tables + one engine | Collapses 4ccEditor's six per-version files; mechanically verifiable |
| Save crypto | Native Rust (MT19937 stream cipher; PES15 LCG + MD5) | Not AES (earlier analysis was wrong); replaces libpesXcrypter.dll; proven portable via the Midcupping scripts and save*.py |
| Save editor scope | Full 4ccEditor parity + direct-manipulation tactics UI | Every 4ccEditor feature carried over; drag-and-drop replaces spinner/dropdown friction |
| Midcupping tools | Absorbed into the save editor | Aesthetics transplant + aesthetics diff as panels; operations live in `pes_savefile` |
| Team interchange | New full-fidelity Team TOML; `.4ccs`/`.4cct` read-only; Texport read+write | Text format editable without the Studio; player settings tables shared with settings.toml; Texport is PES's own format so both directions matter |
| Player folders | The primary export unit | Face+boots+gloves in one folder; primary motivation for the project |
| Boots/gloves IDs | Automatic deterministic assignment | Player-exclusive: from (team_id, player_number); shared: per-team pool; written to savefile at compile time |
| Player settings | TOML in exports (`settings.toml`), merged at compile time; all fields optional; schema covers every savefile aesthetic field (tested) | Replaces manual savefile editing; editor can generate TOML from saves; name writing is opt-in (`name = true`/string — absent leaves the savefile name untouched), so shared folders act as generic model folders |
| Aesthetics patch | `aesthetics_patch.toml` written by every compile beside the CPK; the compiler's only savefile write path (the local save is updated by applying it); applied to other saves by the save editor, whose own aesthetics fields are read-only by default (session unlock) | Separates the DLC builder from the savefile builder — the official save holds the teams' custom tactics, which the DLC builder must not receive; one write path means no drift between the two machines |
| Auto-generated toml comments | App-injected (predefined per-field documentation), preserved by `toml_edit` | Users never write comments from scratch; the comments are the simplified documentation, present in every auto-generated `settings.toml`/`config.toml`/`materials.toml` |
| FPC toggle | Empty `fpc.on`/`fpc.off` marker files in the player folder | Folder-level like link files; apply `pes_savefile`'s version-aware enable/disable presets (same as the editor's toggle); absent = savefile untouched |
| AATF rules | One self-contained Rhai file: parameter map at the top, check functions below | Editable without recompiling; one file is one ruleset, so an invitational's variant is a copy with the top edited and cannot skew against its logic; TOML+CEL and a two-file split rejected (Save editor plan) |
| Music tools | Two tool crates (`music_player`, `music_export_editor`) + `music_export`/`audio_engine` libs | Rigdio/RigDJ successors; descriptive crate names, old names kept in the user-facing labels ("Music player (Rigdio)") and old icons in the collapsed rail |
| .4ccm format | Canonical, unchanged, read + write | Interop with legacy Rigdio during transition; no new format, no migration for managers |
| Audio backend | Pure Rust: `kira` + `symphonia` (+ Rust Opus decoder) + `ebur128` | Drops libmpv-2.dll and ffmpeg.exe; single binary; in-process loudness; WASM-compatible |
| Match tracker | `tools/match_tracker` crate; per-version memory signatures (PES17 initially); Windows-only backend behind a memory-source trait | SEN:P-AI successor; offsets are data, not code; Win32 is the only backend for now |
| Match events interface | In-process `libs/match_feed` (typed pub/sub + snapshot replay); no pipe/socket, `watch` CLI for external scripts | Tracker and music player share one binary; the old pipe protocol has no surviving consumers |
| Match records | New JSON format (`.match.json`); binary `.sen` dropped | Readable and schema-stable; old archives keep old SEN:P-AI |
| Kit configs | `config.toml` in exports, game binary emitted at compile time; `libs/kit_config` shared by compiler/upgrader/editor | Format recovered by disassembling the abandoned closed-source Kit Manager (documented in the Kit config editor plan); text in exports matches the settings.toml philosophy |
| Music autopilot | Tracker-driven team loading/scoring/goalhorns/event clips; Full, Co-pilot (highlights + automatic buttonless clips), Assist (highlight-only), or Off (suppresses tracker-triggered playback/highlights but does not transfer score authority); manual override always live | The suite's end goal: the music player runs the match unattended, with Co-pilot/Assist/Off as trust-building middle steps |
| Player aesthetics viewing | Blender launch via a versioned JSON manifest consumed by the `pes-models` extension's loader; the in-app quick preview waits for `libs/model_viewport`, created only with the Team compiler's future 3D-preview feature | Blender's viewport/Outliner beat any bespoke wgpu pass and stay the accurate view (the future embedded preview is labeled approximate); export-format knowledge stays in Rust (the loader is manifest-driven); glTF conversion stays Studio-side in `model_convert` (`ir_to_gltf` + superset merge) |
