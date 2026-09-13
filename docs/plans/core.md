# 4cc Studio — Core plan

This is the core architectural plan for 4cc Studio: the goals, the platform (crate
structure, tool plugin system, GUI shell, parallelism), and the cross-cutting
decisions. The tools and the major library crates have their own plan documents, listed in the
[plan index](README.md).

---

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
to the older compilers" in the [Team compiler plan](team_compiler.md)).

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
callback — see the [Team compiler plan](team_compiler.md). No webview, no HTML/CSS/JS.

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
  Studio Web (see "Browser deployment: Studio Web" below). Tauri cannot serve to browsers — its web
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
  [Team compiler plan](team_compiler.md)) would use `wgpu` rendering alongside egui rather than
  Three.js in a webview. More work, but it's a post-MVP feature.
- **Log viewer performance**: egui's text rendering can be slow with thousands of lines. Mitigated
  by capping displayed lines to the last few hundred (full logs go to disk regardless).

---

## Architecture

### Crate structure

The suite follows a **platform + plugins** model with strict modularity rules:

1. **`studio_core` is minimal**: only what is needed to run the program with zero tools installed —
   the app shell (sidebar, settings menu, common widgets), the tool plugin trait, the settings
   framework, and the common settings (PES version, paths, threads, memory cap).
2. **Every tool is a separate crate**, no matter how small.
3. **Every shared set of functionality is a separate library crate.** Example: savefile
   loading/saving is a lib crate used by both the save editor tool and the Team compiler tool. A
   future "CPK explorer" tool and a "CPK packer" tool would both import the shared `cpk` lib crate.
4. **Each tool crate owns its own GUI view, its own settings section, and its own CLI commands**
   (see "Tool plugin interface" below).

(Rust has no enforced directory convention for workspace members; grouping crates under
`crates/tools/` and `crates/libs/` is a common, readable pattern and what we use here.)

```
4cc-studio/
├── Cargo.toml                        (workspace)
├── crates/
│   ├── studio/                       # The binary: registers tools, launches GUI or dispatches CLI
│   ├── studio_core/                  # Platform: app shell, tool trait, settings framework,
│   │                                 #   common widgets (grid, log panel), PipelineEvent types
│   ├── tools/
│   │   ├── team_compiler/            # Team compiler (pipeline, loaded processing model, packing, GUI)
│   │   ├── save_editor/              # Save editor (player/team editing, AATF checks, comparator)
│   │   ├── stadium_compiler/         # Stadium compiler (stadium pipeline, billboards)
│   │   ├── export_upgrader/          # Converts old-format exports to the Studio format
│   │   ├── music_player/             # Match-day soundboard (Rigdio successor)
│   │   ├── music_export_editor/      # .4ccm music export editor (RigDJ successor)
│   │   ├── match_tracker/            # Live match stat/event tracker (SEN:P-AI successor)
│   │   ├── kit_config_editor/        # Kit config editor (Kit Manager successor)
│   │   ├── refs_arranger/            # Referee slot allocation → players.txt (compiles via the
│   │   │                             #   Team compiler, which owns referee processing)
│   │   ├── balls_compiler/           # Balls export → ball selection CPK (own bins, own paths;
│   │   │                             #   the format knowledge lives here, single consumer)
│   │   ├── player_aesthetics_editor/ # Player-folder browser → Blender launch (manifest for the
│   │   │                             #   pes-models extension's loader), settings.toml editing,
│   │   │                             #   in-place glTF conversion, base-model extraction
│   │   ├── team_creator/             # New-team wizard: roster file → legal Team TOML + export with
│   │   │                             #   embedded starter heads; owns no format, hands off
│   │   └── ...                       # Future tools (CPK tools, ...)
│   └── libs/
│       ├── pes_savefile/             # EDIT file codec: crypto, schema tables, player/team/
│       │                             #   tactics model, interchange formats (Team TOML, Texport)
│       ├── cpk/                      # CPK read/write (includes CRILAYLA, used only here)
│       ├── fpk/                      # FPK/FPKD read/write
│       ├── fmdl/                     # FMDL (Fox model) read/write + FMDL mesh splitting,
│       │                             #   split vertex encoding, anti-blur decode/encode,
│       │                             #   multi-FMDL mesh merging
│       ├── pes_model/                # .model (PreFox model) + sibling .mtl XML read/write
│       │                             #   + .model mesh splitting, split vertex encoding decode/encode
│       ├── ftex/                     # FTEX read/write
│       ├── dds_convert/              # In-process DDS conversion (texture2ddecoder + image crate +
│       │                             #   DXT5 encoder + in-memory conversion cache, ≤2-team limit)
│       ├── color_tools/              # Dominant kit-color extraction + shared color-picker widget
│       ├── team_widgets/             # Shared egui widgets over the pes_savefile team model: tactics
│       │                             #   pitch/instructions, card pickers, AATF violations list
│       │                             #   (Save editor + Team creator)
│       ├── fox2/                     # Fox Engine entity files (+ CityHash64)
│       ├── uniparam/                 # UniformParameter container
│       ├── wezlib/                   # WESYS zlib wrapper (WE = Winning Eleven, PES's Japanese name)
│       ├── vtree/                    # VirtualTree + canonical ScopePath/RelativeScopePath shared by events/exports
│       ├── pes_version/              # PesVersion (15–21) + Engine (pre-Fox/Fox): the one closed set every
│       │                             #   version-aware crate shares (settings, savefile schemas, skeletons)
│       ├── archives/                 # .zip/.7z loading
│       ├── aesthetics_export/              # The aesthetics export format: object model, folder conventions,
│       │                             #   validation (shared by compiler, upgrader, kit config
│       │                             #   editor, refs arranger, team creator)
│       ├── pipeline/                 # Shared pipeline scaffolding: reader/writer patterns,
│       │                             #   memory budget, folder watcher, check cache, CpkStem
│       ├── elevation/                 # Admin elevation: manifest execution level, detect +
│       │                             #   elevated relaunch (UAC), CLI guidance
│       ├── model_convert/            # IR + importers/exporters + skeleton data; glTF+PES_bone/PES_mesh
│       │                             #   extension import/export + materials.toml read/write lives here,
│       │                             #   via the external `gltf` crate (calls fmdl/pes_model crates for native mesh ops)
│       ├── model_viewport/           # (future) shared wgpu model-preview widget — created with the
│       │                             #   Team compiler's 3D-preview feature; consumers: team_compiler
│       │                             #   and player_aesthetics_editor (labeled approximate preview)
│       ├── python_bindings/          # PyO3 bindings exposing fmdl + pes_model crates to Blender's
│       │                             #   Python; built separately via maturin, not in default build
│       ├── aatf/                     # AATF rules engine (TOML parameters + scripted checks)
│       ├── music_export/             # .4ccm music export format: parse/write + condition model
│       ├── audio_engine/             # Audio playback (kira/symphonia) + loudness analysis (ebur128)
│       ├── match_feed/               # Live match event feed (match tracker → music player)
│       ├── kit_config/               # Kit config codec: binary ↔ TOML, validation (applies fpc values)
│       └── fpc/                      # Full Player Customization as data: kit values, player presets,
│                                     #   interference rules — leaf crate shared by kit_config,
│                                     #   pes_savefile, and the tools that surface FPC
```

**Naming convention.** Directory names equal package names, exactly. Package names use underscores
(Bevy style: `pes_savefile`, not `pes-savefile`), so the directory name, the `Cargo.toml` entry, and
every `use` statement in Rust code share one identical spelling — no hyphen↔underscore mapping to
remember or grep around. Names are chosen by these rules (the workspace is not published to
crates.io, so no blanket prefix is needed):

- **Specific format names stay plain**: `cpk`, `fpk`, `fmdl`, `ftex`, `fox2`, `uniparam`, `wezlib`.
  These are the formats' actual names — unambiguous as-is (`wezlib`: the WESYS header's "WE" is
  Winning Eleven, PES's Japanese name, so the crate is WE + zlib and cannot be mistaken for a
  general zlib crate).
- **Generic names get a disambiguating prefix**: `pes_savefile` (not `savefile`), `pes_model` (not
  `model` — even FMDL files are "models"; the name also matches the existing `pes-model` Blender
  addon, which the community already knows the format by).
- **Community terms win over internal shorthand**: `aesthetics_export` (not `team_export`). Each
  team submits three exports — tactics (PES's own team export, the Texport the save editor reads and
  writes), aesthetics, and music — so "team export" is ambiguous, and it collides inside this very
  codebase with `pes_savefile`'s Texport support. "Aesthetics export" is what the community calls
  the thing; the internal "team export" only ever existed to distinguish it from referee exports
  for a referee compiler that was never built (referee exports are aesthetics exports with the
  `/refs/` team name). The same logic named `music_export`.
- **Role-descriptive names for non-format crates**: `model_convert`, `dds_convert` (it's a
  conversion pipeline, not a DDS codec — avoids confusion with format-codec crates like crates.io's
  `ddsfile`), `python_bindings`, `archives`, `pipeline`, `vtree`, `music_export` (the format is
  ".4ccm", which can't start a crate name; "music export" is the community's term), `audio_engine`,
  `match_feed`, `color_tools`.
- **The `studio` prefix is reserved for the platform itself**: `studio` (the binary) and
  `studio_core`, where it is meaningful rather than a lazy namespace.
- **CLI subcommands and tool ids keep hyphens** (`studio team-compiler ...`, id `"team-compiler"`) —
  that's CLI convention, independent of package names.

**`studio_core`** — the platform, containing only what runs with zero tools:
- The app shell: sidebar (logo, PES version selector, tool list, collapse, help and settings
  buttons) and the status bar (see "Status bar" under GUI Design)
- The `StudioTool` plugin trait (see below)
- The settings framework: common settings + per-tool settings sections, TOML persistence
- The help framework: the help window, its search, and the core chapters (see "Help window")
- Common widgets: the progress grid, the log panel, the run strip
- The `PipelineEvent` types these widgets consume, using `vtree::ScopePath` for canonical
  tool-neutral event paths

Its module tree is closed — "minimal" is enforced by there being no natural place to put anything
else:

```
crates/studio_core/src/
├── lib.rs            # re-exports only
├── tool.rs           # StudioTool trait, ToolContext (settings access, event sink, tool switch, notify)
├── events.rs         # PipelineEvent, PipelineEventEnvelope, FolderStatus, Message + Severity/Disposition
├── status.rs         # ShellCondition, ToolActivity, Notice — the status bar's typed items
├── help/
│   ├── mod.rs        #   HelpSection/HelpTopic, HelpTarget (tool, topic, anchor), catalog-topic generation
│   ├── search.rs     #   substring search over every topic's paragraphs and catalog rows
│   ├── export.rs     #   `studio help` printing and `--export` to Markdown files
│   └── core/         #   the core chapters' Markdown (setting up, data location, updates) + CHANGELOG.md
├── settings/
│   ├── mod.rs        #   settings framework: load/merge defaults/save, per-tool sections keyed by id
│   ├── common.rs     #   CommonSettings (PES version, paths, threads, memory cap, theme, update state)
│   └── location.rs   #   data-location resolution (portable data/ vs config dir), first-run choice
├── shell/
│   ├── mod.rs        #   StudioApp: eframe::App, panel layout, logic/ui split, tool registry
│   ├── title.rs      #   window title from activities / held notice / idle; ViewportCommand::Title on change
│   ├── sidebar.rs    #   logo, version selector (live exe check, running-PES auto-switch), tool list
│   ├── settings_menu.rs  # common section + injected per-tool sections
│   ├── help_window.rs    # chapter tree + rendered topic (egui_commonmark) + search box; opens on the active tool
│   ├── status_bar.rs #   conditions / background activity / latest notice; empty state = the version
│   ├── about.rs      #   About popup from the logo: detailed version block with Copy, links
│   └── launch.rs     #   argument parsing for the three launch modes, --gui autorun hand-off
├── widgets/
│   ├── grid.rs       #   progress grid (rows × cells, FolderStatus colors, fades)
│   ├── log_panel.rs  #   capped log, auto-scroll, filters, cross-linking
│   └── run_strip.rs
├── theme.rs          # the shared Visuals preset, fonts, egui-phosphor registration
├── pes_process.rs    # running-PES poll (desktop): version + exe path detection
└── updater.rs        # release check, download/verify, binary swap, rollback (desktop only)
```

Anything not on this list belongs in a lib crate or a tool crate. The two tempting misplacements:
format knowledge (team IDs, teams-list parsing → `teams_list`; export conventions →
`aesthetics_export`) and widgets used by one tool (→ that tool's `view/`).

**Tool crates** (`crates/tools/`) — each implements the `StudioTool` trait and owns:
- Its processing logic (or orchestration over lib crates)
- Its GUI view (rendered into the panel right of the tool selector)
- Its settings struct + settings UI section
- Its CLI subcommands

Every tool crate has the **same top-level skeleton**, so any session opening any tool finds the
same six things in the same places:

```
crates/tools/<tool>/
├── help/           # the tool's help chapter: numbered Markdown topics (01_overview.md, …), embedded by lib.rs
└── src/
    ├── lib.rs      # `Tool`: the StudioTool impl — wiring only, no logic
    ├── settings.rs # the tool's settings struct, defaults, settings_view
    ├── cli.rs      # clap Command; cli_run (and gui_run where the tool supports autorun)
    ├── messages.rs # the tool's message catalog (IDs, severity, disposition, templates)
    └── view/       # egui — render-only; reads tool state, emits user intents
        └── mod.rs
```

The small tools (Kit config editor, Refs arranger, Balls compiler, Export upgrader, Player
aesthetics editor, Music player, Music export editor) add their logic as flat modules next to these
(`compile.rs`, `arrange.rs`, …) and need no further prescription. The two flagship tools and the
Match tracker elaborate below the skeleton; their trees are in their own plans ("Crate layout" in
the [Team compiler](team_compiler.md), [Save editor](save_editor.md), and [Match
tracker](match_tracker.md) plans). Lib crates with a mandated layout are `pes_savefile`,
`model_convert`, and `aesthetics_export` (in their plans) plus the `format/`–`ops/` split of `fmdl` and
`pes_model` (in [libs](libs.md)); the remaining libs are single-concern and get no prescription.

**Lib crates** (`crates/libs/`) — shared functionality with no tool identity. Format and processing
libs stay GUI-neutral; explicitly shared widget libs (`color_tools`, future `model_viewport`) may
use egui/wgpu. Formats exclusive to one consumer still get their own crate when they are conceptually
standalone formats (e.g. `uniparam`); single-consumer helpers stay with their owner (e.g. CRILAYLA
lives inside `cpk`).

**No per-tool companion libs.** Even the largest tool — the Team compiler, at an estimated 15–25k
lines — stays a single crate, organized by internal modules mirroring its pipeline stages (`reader/`,
`check/`, `plan/`, `processing/`, `bins/`, `output/`, `view/` — the full tree and placement rules are
the "Crate layout" section of the [Team compiler plan](team_compiler.md)). A `team_compiler_lib` next to it
would have exactly one consumer, which rule 3 below calls
out, and it would buy nothing for navigation that the module tree doesn't already give. What *is*
extracted is the part with real multiple consumers: `aesthetics_export` (the export format's object model,
folder conventions, and validation) is shared by the Team compiler, the Export upgrader that writes
the format, the Kit config editor that edits kit folders inside exports, the Refs arranger that
edits referee slot allocations inside refs exports, and the Team creator — so all of them
agree on what a valid export is. Should the Team compiler ever need splitting for compile times or a
headless build, the principled seam is logic vs. egui view (see the trade-off note under "Workspace
guardrails"), not a catch-all lib. This rule has already survived its first pressure test: the [Refs
arranger](refs_arranger.md) was initially conceived as a "refs compiler" owning its own compile
step, which would have forced the engine extraction for one thin extra consumer; instead it only
prepares the refs export's `players.txt` and hands compilation off to the Team compiler (see "Why an
arranger, not a compiler" in its plan).

`fmdl` and `pes_model` have a second consumer outside this workspace: the Blender addons (see
"Blender integration" in the [Model conversion plan](model_conversion.md)). Their mesh splitting,
split vertex encoding, and anti-blur logic lives inside the format crate, not in `model_convert`, so
that a same-format FMDL or .model round-trip in Blender is lossless and never routes through the IR.
`model_convert` calls these format-native algorithms during cross-format conversion; the format
crates do not depend on `model_convert`.

`python_bindings` is a PyO3 shim crate that exposes `fmdl` and `pes_model` to Blender's bundled
Python as a native extension (`.pyd`/`.so`). It is not imported by `studio` or any tool crate. It is
built separately via `maturin` (not in the default `cargo build` path) and exists in the workspace
to enforce the PyO3-buildability guardrail below — if a change to `fmdl` or `pes_model` introduces a
dependency that breaks the `cdylib` build, CI catches it at the PR rather than at a Blender user's
runtime. It is a leaf crate: nothing depends on it, and it can be deleted in one commit when legacy
FMDL/.model Blender support is retired.

**`studio`** — the single binary. With no arguments it launches the egui GUI; with arguments it
dispatches to the named tool's CLI. Compiles to both native (desktop) and WASM (the eframe `wasm32`
target, browser deployment).

#### Launch modes

The binary has three launch modes, distinguished by its arguments:

| Invocation | Behavior |
|---|---|
| `studio` | GUI, last-used tool active |
| `studio <tool-id> <command> [args]` | Headless CLI: the tool's `cli_run`, console output, exit code |
| `studio --gui <tool-id> <command> [args]` | **GUI autorun**: the GUI opens with `<tool-id>` active and the tool performs `<command>` inside the GUI, exactly as if the user had pressed the corresponding button |

Besides the tools' subcommands, the shell owns `studio help`, `studio update` and `studio
--version` (see "Help window", "Distribution and updates", "Changelog and version display").

The `--gui` form exists for the release bundle's **`quick_compile.bat`** — the successor to Red's
`0_all_in_one.bat`, the one script almost every user runs: copy an export into `exports/`,
double-click, done. The launcher is two lines (`start "" "%~dp0studio.exe" --gui team-compiler
compile`) so its console window closes immediately and only the GUI remains; the Linux bundle ships
the equivalent `quick_compile.sh`. Red's other staged bats (`1_`–`3_`) have no successor: the
unified pipeline has no stages to run separately. Users who want the plain GUI double-click the exe.

`--gui` is a global flag parsed by `studio` before tool dispatch, so no tool's `clap::Command`
knows about it. The shell reuses the tool's own CLI parser for the arguments (one grammar for both
modes) and hands the parsed `ArgMatches` to the tool's `gui_run` hook (see the trait below) once
its view exists. Commands a tool does not implement in `gui_run` fail at startup with a clear
message rather than silently opening the GUI. A tool decides its own readiness: the Team compiler
queues the autorun until the initial exports check has completed, then fires it — the same
precondition as a manual Compile click, so the grid never compiles unchecked state.

**Windows subsystem.** A Rust exe is either a console or a GUI application, and this binary is both.
It is built as a GUI-subsystem exe (`#![windows_subsystem = "windows"]`) so double-clicking it, or
`quick_compile.bat`, never flashes a console window; when CLI arguments are present it calls
`AttachConsole(ATTACH_PARENT_PROCESS)` so console output reaches the terminal it was started from.
Known caveat of this standard arrangement: an *interactive* `cmd.exe` prompt does not wait for
GUI-subsystem processes, so output can interleave with the returned prompt (batch files,
`start /wait`, and PowerShell pipelines are unaffected). If that proves to annoy CLI users, the
escape hatch is a tiny console-subsystem `studio-cli.exe` shim that spawns the main exe and waits;
deferred until the annoyance is real. Linux has no subsystem split.

### Tool plugin interface

Tools are compile-time plugins: each tool crate implements a trait defined in `studio_core`, and the
`studio` binary registers them in a static list. This gives the modularity of plugins without
dynamic loading complexity.

```rust
// studio_core
pub trait StudioTool {
    /// Stable identifier, used for CLI dispatch and settings keys ("team-compiler")
    fn id(&self) -> &'static str;
    /// Sidebar label ("Team compiler")
    fn label(&self) -> &'static str;

    /// The tool's main view, rendered in the panel right of the tool selector
    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext);

    /// Per-frame background work, called for EVERY registered tool each frame —
    /// active view or not. Default no-op. This is where tools drain event
    /// channels and check timer deadlines (e.g. the music player's autopilot
    /// consuming the match feed while the tracker's view is on screen).
    /// The blanket pass stays cheap at any tool count: a no-op dyn call is
    /// nanoseconds, and frames only happen on input or requested repaints.
    fn tick(&mut self, ctx: &ToolContext) {}

    /// The tool's settings section, injected into the settings menu.
    /// The settings menu shows the current tool's section expanded, others collapsed.
    fn settings_view(&mut self, ui: &mut egui::Ui);
    /// Defaults merged into the settings file on first run / new versions
    fn default_settings(&self) -> toml::Table;

    /// The tool's chapter of the help window: its Markdown topics, embedded from
    /// the crate's `help/` folder with `include_str!`. The window appends the
    /// tool's message catalog as a generated topic, so `help()` carries prose
    /// only. See "Help window".
    fn help(&self) -> HelpSection;

    /// The tool's CLI subcommand definition and executor
    fn cli_command(&self) -> clap::Command;
    fn cli_run(&self, matches: &clap::ArgMatches, ctx: &ToolContext) -> anyhow::Result<()>;

    /// GUI autorun for `studio --gui <tool-id> <command>` (see "Launch modes"):
    /// perform the already-parsed CLI command inside the GUI, as if the user had
    /// pressed the corresponding button. Called once by the shell after the
    /// tool's view has been created; the tool may queue the action until its
    /// own readiness condition holds (the Team compiler waits for the initial
    /// exports check). Default: no command is GUI-runnable, so the launch fails
    /// with a message instead of silently opening the GUI.
    fn gui_run(&mut self, matches: &clap::ArgMatches, ctx: &ToolContext) -> anyhow::Result<()> {
        anyhow::bail!("{} has no GUI-runnable commands", self.id())
    }

    /// If the tool is in a state that blocks changing the global PES version
    /// (e.g. the match tracker holding live memory addresses), returns a short
    /// human-readable reason shown as the selector's disabled-state tooltip.
    /// Default: always changeable. The shell calls this before applying a
    /// version change; a non-`None` result blocks the change.
    fn version_change_blocker(&self) -> Option<&str> { None }

    /// What this tool is doing in the background. Shown in the status bar's
    /// activity slot only while a *different* tool's view is on screen (the active
    /// tool's own run strip already shows it), and in the window title for every
    /// tool, active or not. Default: nothing. See "Status bar", "Window title".
    fn activity(&self) -> Option<ToolActivity> { None }
}
```

`ToolContext` is the tool's one handle on the platform, cheap to clone. **Settings** are behind
one lock shared with the shell (`Arc<Mutex<Settings>>`, one lock per concept): `ctx.common()`
copies the common settings, `ctx.tool_settings(id)` copies the tool's own table, and
`ctx.set_tool_settings(id, table)` replaces it and queues a `ShellRequest::SettingsChanged` so
the shell saves best-effort at the end of the frame. Tools never see the file or its path.
**Events** go through `ctx.emit(envelope)`, the one place a gone receiver is handled (logged at
debug, dropped). Besides these, `ToolContext` exposes a **tool-switch
request**: `ctx.switch_to_tool("team-compiler")` asks the shell to make the named
tool the active view at the end of the frame. It is id-string based, so tools can
link to each other (the Refs arranger's handoff to the Team compiler) without
depending on each other's crates. It also exposes `ctx.notify(Notice)`, the one way a
tool puts something in the status bar (see "Status bar"); `Notice` is a typed item,
not a string, so the bar cannot become a free-text dumping ground.

```rust
// studio (binary)
fn tools(ctx: &egui::Context) -> Vec<Box<dyn StudioTool>> {
    // The match feed is owned by the binary and passed to the tools that need it
    // via constructor injection — studio_core does not depend on match_feed.
    // The wake callback lets the feed request a GUI repaint on publish, so
    // events are processed promptly even while the app sits idle without input.
    // It is GUI-neutral (Arc<dyn Fn() + Send + Sync>) so match_feed does not
    // depend on egui; the shell installs it after an egui::Context is available.
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new({
        let ctx = ctx.clone();
        move || ctx.request_repaint()
    });
    let match_feed = Arc::new(MatchFeed::new(wake));
    vec![
        Box::new(team_compiler::Tool::new()),
        Box::new(save_editor::Tool::new()),
        Box::new(stadium_compiler::Tool::new()),
        Box::new(export_upgrader::Tool::new()),
        Box::new(music_player::Tool::new(match_feed.subscribe())),
        Box::new(music_export_editor::Tool::new()),
        Box::new(match_tracker::Tool::new(match_feed.publisher())),
        Box::new(kit_config_editor::Tool::new()),
        Box::new(refs_arranger::Tool::new()),
        Box::new(balls_compiler::Tool::new()),
        Box::new(player_aesthetics_editor::Tool::new()),
    ]
}
```

CLI dispatch uses standard clap subcommands (each tool contributes one), which is the conventional
CLI shape:

```
studio team-compiler compile ./exports_folder
studio team-compiler check ./exports_folder
studio save-editor export-toml ./EDIT00000000 --team 701 -o ./aaa.toml
studio stadium-compiler compile ./stadium_exports
studio export-upgrader ./old_export.zip
studio music-export-editor check ./team.4ccm
studio match-tracker watch
studio kit-config-editor convert ./config.bin ./config.toml
studio refs-arranger chances ./exports/refs_export
studio balls-compiler compile
```

(An argument-flag style like `studio --team-compiler compile ...` would work too, but subcommands
are what clap and CLI conventions are built around — better help output, completion support, and
per-subcommand argument validation.)

### Workspace guardrails

The crate arrangement matches established Rust monorepo practice (Bevy's `Plugin` model,
rust-analyzer's crate graph). The following rules keep it healthy:

1. **Tool crates never depend on tool crates.** If two tools need the same logic, it moves down into
   a lib crate. This is the arrangement's main failure mode — a tool importing another tool drags in
   its whole pipeline and GUI, and creates hidden coupling.
2. **All shared dependencies are declared once** in the root `Cargo.toml` via
   `[workspace.dependencies]`; crates inherit with `dep.workspace = true`. This prevents version
   drift (egui especially, which every tool crate uses and which must be in lock-step
   workspace-wide).
3. **Don't split finer than this.** Warning signs of over-splitting: crates with one function, lib
   crates with a single permanent consumer, changes that routinely touch four crates at once. A lib
   crate earns its existence through multiple consumers or the standalone-format exception above.
4. **`fmdl` and `pes_model` must stay PyO3-buildable.** These two crates have a second consumer
   outside this workspace (the Blender addons, via `python_bindings`). They may not depend on
   `studio_core`, `studio` (the binary), any tool crate, `model_convert`, or any crate from this
   denylist: `egui`, `eframe`, `wgpu`, `tokio`, `smol`, `async-std`, `pyo3`. Pure-logic dependencies
   (`binrw`, `rayon`, `serde`, `nalgebra`, `anyhow`, etc.) are permitted. Enforce these exclusions
   with a dependency-graph CI check; a successful `cdylib` build does not enforce architectural
   boundaries. Separately build and smoke-test `python_bindings` to catch actual linking/Python
   compatibility failures. Runtime exclusions are deliberate dependency policy, not a claim that
   every async runtime necessarily fails in a Python extension. This rule and the `python_bindings`
   crate retire together when legacy FMDL/.model Blender support is dropped. `model_convert` is not
   subject to this rule — it is Studio-side and has no direct Blender consumer.
5. **`cargo clippy` is clean from the first commit.** Run clippy with `-D warnings` (warnings
   denied) on every change from the start of Phase 1, not as a pre-release cleanup. Clippy catches
   the Rust-specific footguns that are expensive to retrofit across a workspace this size — needless
   borrows, `clone`-on-Copy, manual `map`/`filter` chains that have idiomatic replacements,
   `to_string()` in hot loops, holds kept across `.await`/locks — and enforcing it early means the
   codebase never accumulates a backlog of suppressions that hide real issues. Workspace-wide lints
   live in the root `Cargo.toml` `[workspace.lints.clippy]` table (inherited by every crate via
   `[lints] workspace = true`), so the policy is declared once and applies uniformly; per-crate
   `#[allow(...)]` opt-outs are exceptions that get called out in review, not the default. This
   pairs with `rustfmt` for formatting: both run in CI on every PR. The toolchain is pinned in a
   root `rust-toolchain.toml` (an exact stable version plus the `wasm32-unknown-unknown` target)
   and bumped deliberately, not tracked as floating `stable`: every Rust release ships new clippy
   lints, so with `-D warnings` a floating toolchain turns a `rustup update` into a red gate on
   code nobody touched (EGG-Translator's gate broke this way on a single new 1.98 lint).
6. **Every lib crate must stay `wasm32`-checkable.** CI runs
   `cargo check --target wasm32-unknown-unknown` over `crates/libs/` (and `studio_core`) on every
   PR from Phase 1. Studio Web (see "Browser deployment") is a post-release deliverable, and the
   cheapest way to keep it cheap is never letting a lib crate pick up a dependency that only builds
   natively. Lib crates with legitimately native-only parts (`archives` on disk paths, `match_feed`'s
   Win32 source, `audio_engine`'s cpal output) gate those behind
   `#[cfg(not(target_arch = "wasm32"))]` or a cargo feature so the check still passes; the check is
   compile-only — no test execution in WASM is required. Tool crates are exempt until the web tier
   is scheduled, since their views may legitimately depend on native-only libs before adapters exist.

Accepted trade-off: the `StudioTool` trait references egui types, so every tool crate compiles egui
even in CLI use (dead code is stripped from the binary). If a truly headless build is ever needed
(CI, server-side compilation), the escape hatch is splitting the trait into `StudioTool` (id,
settings, CLI) and `StudioToolUi` (views) behind a `gui` cargo feature — deferred until that need is
real. A cheap addition the registry enables: one cargo feature per tool in `studio`, allowing
stripped single-tool builds.

### Event system

The pipeline emits structured events instead of printing to stdout. The CLI and GUI layers each
present these differently. Every event is wrapped in an envelope carrying stale-result protection:

```rust
pub struct PipelineEventEnvelope {
    pub run_id: RunId,
    pub export_id: Option<ExportId>,
    pub export_revision: Option<ExportRevision>,
    pub event: PipelineEvent,
}
```

The GUI rejects events whose `run_id` or `export_revision` is older than the current grid state, so
results from a stale filesystem snapshot cannot overwrite newer checks.

```rust
pub enum PipelineEvent {
    ExportStarted { export_id: ExportId, display_name: String },
    ExportProcessed { export_id: ExportId },
    FolderStatus { export_id: ExportId, path: vtree::ScopePath, status: FolderStatus },
    // Warnings/errors are structured messages (stable code + severity +
    // disposition + scope + context fields), not prebaked strings; each tool
    // defines its message catalog (see "Message catalog" in the Team compiler
    // plan). Display text is rendered from a per-catalog template table.
    Message(Message),
    Progress { processed: usize, total: usize },
    // The run strip needs stage, elapsed time, files written, and memory in
    // use — more than `Progress` carries. Extend `Progress` with these fields
    // or add separate `StageChanged`/`MetricsUpdated` events before implementing
    // the run strip view (see "Toolbar" in the Team compiler plan).
    Complete { success: bool, files_written: usize, skipped_folders: usize, duplicates: usize, folders_with_errors: usize },
    // The Team compiler needs separate compile and deployment outcomes (rollback,
    // "artifacts built but installation failed"). Replace the boolean `success`
    // with structured CompileOutcome/DeploymentOutcome/CancellationOutcome before
    // implementing the run-result semantics described in the Team compiler plan.
    // At that point the placeholder folder-only counters above become task/scoped-
    // outcome counts, so typed non-folder tasks are represented in the summary.
    UnmigratedContentFound,   // e.g. files the upgrader could not place
}
// ExportStarted stays tool-neutral. Team names, resolved team IDs, ball identities,
// and other tool-specific identity belong in tool state or Message context.

pub enum FolderStatus {
    // FolderStatus is retained as the widget/event status type for typed BuildTasks.
    // It applies to each task's owning UI scope: its folder/slot cell for model and
    // kit work, or the containing row label for non-folder task scopes.
    // Check phase (folder watcher)
    Unchecked,       // not yet validated
    Checking,        // validation in progress
    PartialOk,       // shallow-checked, no issues found (archive not extracted)
    FullOk,          // deep-checked, no issues
    Warning,         // checked with warnings (does not block compilation)
    PartialError,    // shallow check found errors (list may be incomplete)
    Error,           // deep check found errors (authoritative list)
    // Compile phase (Compile pressed). Every checked cell is implicitly
    // pending once compilation starts, so there is no pending state;
    // Processing keeps the check-phase color and bolds the label instead.
    // Processing eligibility depends on effective disposition, not severity
    // alone: DropSlot removes one roster assignment, DropFolder/DropExport scopes
    // skip, and DropFile scopes process remaining content and end as DoneWithErrors.
    Processing,      // being processed by the pipeline
    Done,            // processed successfully
    DoneWithWarning, // processed with warnings
    DoneWithErrors,  // written despite Error-level findings (pass_through, or DropFile where remaining content was processed)
}
```

**CLI** translates events to the current `-` prefixed console format for familiarity.
**GUI** receives events via channels and renders them as egui UI updates (grid cell colors, log
lines, run strip).

### Diagnostic logging

Events and findings are for the *user*. Developer diagnostics — "opened cache at X", "FPK had 12
entries, 34 ms", "watcher event dropped as stale" — go through the `log` facade, and nowhere else:
no `println!`/`eprintln!` outside `studio`'s CLI result output and `main` before the logger is
installed. Where output goes is the binary's decision, not a lib's: a `windows_subsystem =
"windows"` GUI has no stderr, `wasm32` has no stderr, `python_bindings` would print into Blender's
console, and a lib printing to stdout corrupts piped CLI output.

- Lib and tool crates depend on `log` only. `studio` installs the sink: `env_logger` (default
  features off) in CLI mode, level set by `-v`/`-vv` or `RUST_LOG`; in GUI mode a small
  `log::Log` impl writing `studio.log` in the data directory — the file a user attaches to a bug
  report. The GUI "Log panel" shows findings, not this file; an in-app diagnostic view is not
  planned for the first release. `python_bindings` installs `pyo3-log`, which forwards the libs'
  lines into Python's `logging` (Blender's console) with no lib changes.
- The split: if a user should see it, it is a finding (`Message`); if a developer should see it, it
  is a log line. Never both for one fact. Libs never `error!` — a lib returns the error and the
  caller decides what it means; libs use `debug!`/`trace!`, and `warn!` only for a recovered
  anomaly that is not a finding.
- Levels: `info` is one line per stage per run; `debug` is per export or per file; `trace` is per
  item, and never inside per-vertex or per-pixel loops (the arguments are not formatted when the
  level is off, but the call still sits in the hot path a reviewer reads). No `[module]` prefixes
  in messages: the target carries the module path.
- `log`, not `tracing`. The structured, timed data Studio needs (stage, elapsed, per-export
  outcome) already flows through `PipelineEvent`, so what remains is flat diagnostics, which is
  `log`'s job; `tracing`'s subscriber layers are the type-level complexity the code style excludes.
  The choice is no-regret: should spans become worth having, `tracing` can be adopted in the binary
  only, consuming the libs' `log` lines through `tracing-log`; the reverse migration is not clean.
  Profiling is a separate question (`puffin` integrates with egui) and is not what logging is for.

---

## GUI Design

### Shell layout

A shell built with egui panels: a left sidebar, a tool view, and a one-row status bar under both.
The tool view belongs to each tool; pipeline tools subdivide it into grid, toolbar, and log panel
(other tools, e.g. the save editor, use it differently). The status bar belongs to the shell.

```
┌──────────┬──────────────────────────────────────────────┐
│          │                                              │
│  [logo]  │   Tool view (fills remaining space)          │
│          │                                              │
│  PES 19▾ │   ┌────────────────────────────────────────┐ │
│          │   │ Team grid (scrollable)                 │ │
│  ──────  │   │  /aa/ [714]  [01][02][03][□][□]    [GK][K1]  │ │
│   SAVE   │   │  /bb/ [???]  [01][02][03][04][05]  [GK][K1]│ │
│ ▌ TEAM   │   │  /cc/ [809]  [01][□][□][□][□]      [GK][K1]    │ │
│   STAD   │   └────────────────────────────────────────┘ │
│   UPGR   │   [run strip]              [Compile ▾]    │
│   ⋯      │   ┌────────────────────────────────────────┐ │
│          │   │ Log panel (scrolling, collapsible)     │ │
│  ──────  │   │ - Warning - Nested folders ...         │ │
│  [gear]  │   │ - /cc/ processed                       │ │
│          │   └────────────────────────────────────────┘ │
├──────────┴──────────────────────────────────────────────┤
│ ⚠ data folder read-only          Compiled 48 · [Open output] │
└─────────────────────────────────────────────────────────┘
```

In egui, the shell is built with `Panel` (sides) + `CentralPanel`, shown on the whole-app `Ui`
that `eframe::App::ui` provides (API as of egui 0.36; see "egui version discipline" below). Panels
claim space in call order, so the status bar is added first to span the full width:

```rust
impl eframe::App for StudioApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Status bar: full width, under the sidebar too (added first)
        egui::Panel::bottom("status_bar").exact_size(24.0)
            .show(ui, |ui| self.render_status_bar(ui));

        // Left sidebar (collapsible width)
        egui::Panel::left("sidebar").exact_size(self.sidebar_width())
            .show(ui, |ui| self.render_sidebar(ui));

        // Bottom log panel (collapsible height)
        egui::Panel::bottom("log")
            .resizable(true).default_size(220.0)
            .show(ui, |ui| self.render_log(ui));

        // Toolbar (run strip + compile button)
        egui::Panel::bottom("toolbar").exact_size(32.0)
            .show(ui, |ui| self.render_toolbar(ui));

        // Central panel: the tool view (grid, save editor, etc.)
        egui::CentralPanel::default().show(ui, |ui| {
            self.render_tool_view(ui);
        });
    }
}
```

### egui version discipline

egui still ships renames and deprecations most releases (0.31 moved window options into
`ViewportBuilder`; 0.32 rewrote popups/menus and renamed `Rounding` to `CornerRadius`; 0.34
deprecated `SidePanel`/`TopBottomPanel` for `Panel`, replaced `App::update(&Context)` with
`App::ui(&mut Ui)`, and renamed `Context::style` to `global_style`). LLM-assisted coding reproduces
whichever API the model saw most, so it will emit pre-0.34 code by default. Rules:

- Pin exact `egui`/`eframe`/`egui-wgpu` versions once in `[workspace.dependencies]` (the
  lock-step rule under "Workspace guardrails") and upgrade deliberately, one release at a time,
  following each release's migration notes in `CHANGELOG.md` / the GitHub release.
- Treat deprecation warnings as errors in CI (`#![deny(deprecated)]` or `-D deprecated`) so stale
  API from generated code is caught at compile time rather than accumulating.
- The code sketches in these plans follow 0.36; they illustrate structure and must be re-checked
  against the pinned version when implemented, not pasted.

### Sidebar

Top to bottom:

1. **4cc logo** — doubles as the About click target. About is a small popup holding the
   *detailed* version block — `4cc Studio v1.4.0`, build kind (release, or `debug` from
   `cfg!(debug_assertions)`), target triple, data location, selected PES version — with a
   **Copy** button that puts exactly that block on the clipboard for a bug report, and links to
   the release page, the repository and the help. No version is shown beside the logo itself: the
   status bar already shows it 300 px away, and the collapsed icon rail has no room for it, so it
   would be the one element that came and went with the collapse.
2. **PES version selector** — a *global* setting, not per-tool: the Team compiler targets it, the
   save editor uses it as a selection/default hint (an open save keeps its detected schema), and
   integrated conversion uses it as the destination format. Compact dropdown. Changing it is blocked with a tooltip while any tool reports a
   `version_change_blocker` (e.g. a running compilation, or the match tracker holding live memory
   addresses).
   - **Live exe check**: the selector is colored against the exe actually present in the resolved
     `pes_folder_path` — **red** when the folder exists but holds a `PES20XX.exe` of a *different*
     version (wrong selection or wrong path), **yellow** when the folder is missing or holds no PES
     exe at all (likely invalid path); the tooltip names what was found. Re-checked on version/path
     changes and on window focus — a cheap single directory listing. This is the GUI rendering of
     the same condition the team compiler's `pes_version_mismatch` warning reports at compile time
     (the warning remains for the CLI).
   - **Auto-switch to the running PES** (Windows desktop): a lightweight background poll watches for
     running `PES20XX.exe` processes. **Once per program life**, when a running PES's version
     differs from the selection, the selector switches to it automatically — primarily so the match
     tracker attaches to the right game without ceremony — announced via a status-bar notice.
     Once-per-life means a user who then switches away is never fought; a manual version change also
     consumes the auto-switch. If a `version_change_blocker` is active at detection time the switch
     is deferred until it clears (not consumed). If several PES versions are running, the first
     detected wins. The switch writes the ordinary persisted setting — the selector always reflects
     the setting, no shadow session state. The poll also knows the running exe's *path*: when it
     differs from the resolved `pes_folder_path` (e.g. the selector sits red/yellow while the real
     install runs from elsewhere), the same notice offers a **"Fix PES path"** action that rewrites
     `pes_folder_path` from the running process — strictly opt-in via the notice's button, never
     applied automatically (re-inserting the `**` version placeholder when the path segment matches
     the running version, so the fix keeps working across versions).
3. **Tool list** — vertical nav with **full text labels** (small text, sidebar wide enough to fit),
   populated from the registered `StudioTool` list. The active tool is highlighted with an
   accent-colored left border bar plus a filled background. Unimplemented tools are listed grayed
   out, communicating the suite roadmap.
4. **Collapse button** — collapses the sidebar into an icon-only rail (tooltips carry the labels).
   Width toggles between ~160px and ~56px.
5. **Help `?` and settings gear** pinned at the bottom — open the help window and the settings
   menu. F1 opens the help too.

### Settings menu

The settings menu is assembled from injected sections:

- **Common settings** (defined by `studio_core`): PES version, PES folder path, exports folder path,
  thread count, memory cap, theme.
- **One collapsible section per registered tool**, provided by that tool's `settings_view()` (each
  tool defines its own settings struct and UI).
- Opening the settings menu shows the **currently selected tool's section expanded** and all other
  sections collapsed.
- Settings persist to a single file: `[common]` holds `CommonSettings`, and each tool's values
  live in a table under its `id()` key (`[team-compiler]`; `common` is therefore not a valid tool
  id), with `default_settings()` merged in for missing values, recursively into nested tables
  (this replaces Red's settings-transfer mechanism for new versions). Missing common keys load as
  their defaults the same way; unknown keys in a tool's table are kept as read. The file's location — `data/` next to the exe or the user config dir — is the
  user's first-run choice (see "Data location" under Distribution and updates).
- Saving is **best-effort**: if the write fails (typically a portable install unpacked somewhere
  unwritable), the in-memory settings keep working for the session and the shell raises the
  `data_dir_read_only` condition in the status bar — one condition covering settings and the
  teams list, since both live there. No elevation, no retry prompt.

### Help window

The help window is the user documentation — it replaces both of Red's readmes (the 114-line
README and the 913-line `readme_advanced.md`), and the bundled readme shrinks to a pointer at it.
Like the settings menu it is **assembled from the tools**, not written centrally:

- **Chapters.** `studio_core` contributes the core chapters (setting up, first run and data
  location, updates, and "What's new" — the repository's `CHANGELOG.md` embedded at build time,
  see "Changelog and version display"); every
  registered tool contributes one chapter through `StudioTool::help()`, a `HelpSection { title,
  topics: Vec<HelpTopic> }` whose topics are Markdown bodies embedded with `include_str!` from
  the crate's `help/` folder (numbered files fix the order: `01_overview.md`, `02_exports.md`,
  …). Chapters appear in sidebar order; an unregistered tool has no chapter, so the help can
  never describe a tool the build lacks. Red's `readme_advanced.md` maps onto this almost
  one-to-one: its refs, format, texture-handling and special-files headings become Team compiler
  topics, its updater and first-run headings core topics.
- **The Messages topic is generated.** Each chapter ends with a "Messages" topic rendered from the
  tool's `messages.rs` catalog — ID, severity, condition, consequence — so it is always complete
  and never hand-maintained; `help()` carries prose only. This is Red's "Message Types" chapter
  done once, for every tool.
- **Search.** One text box above the chapter tree, case-insensitive substring over every topic's
  paragraphs and every catalog row; results list as *Chapter › Topic › matching line*, click to
  jump with the hit highlighted. No index and no fuzzy matching: the whole corpus is tens of
  kilobytes, and a linear scan per keystroke costs nothing. Its first job is the one Red's users
  did by hand — paste a message ID from the log and find out what it means.
- **Opening.** The sidebar's `?` button or F1 opens the window on the **active tool's chapter**
  (as the settings menu opens on the active tool's section). A **message ID in the log panel is a
  link**: clicking it opens the help at that catalog row (`HelpTarget { tool, topic, anchor }`,
  the same target type search results and in-doc links resolve to). In-doc links use the form
  `[text](tool-id#topic-slug)` so a topic can point at another tool's topic without knowing where
  it renders.
- **Window.** An in-app egui window — resizable, position and size remembered in common settings,
  chapter tree left, rendered topic right. Not a second OS window: egui's multi-viewport support
  exists, but it adds a class of platform bugs that a text reader does not need. Rendering is
  `egui_commonmark` (CommonMark subset with tables, code blocks and links); a hand-rolled
  renderer was considered and rejected as a permanent maintenance item for a solved problem.
- **CLI equivalence.** `studio help [tool-id] [topic]` prints the Markdown to stdout, and
  `studio help --export <dir>` writes every chapter out as `.md` files — the wiki and the release
  page get the same text with no second source, and the `--export` output is what the release
  process publishes.
- **Authoring rule.** Topics are written for members, not maintainers: what to put where, what a
  message means and what to do about it, never how the code works (that is `docs/`). A behavior
  change that a member can observe is not done until its topic says so — the help lives in the
  tool's crate precisely so the same diff carries both.

### Log panel

- Fixed-height region below the toolbar, monospace, colored lines (info/warning/error mapped from
  `PipelineEvent`), collapsible via chevron or divider drag.
- **Capped display**: only the last few hundred lines are rendered (egui text rendering is not
  optimized for thousands of lines). Full logs are written to disk (`issues.log`, `suggestions.log`)
  as before.
- **Auto-scroll with interruption**: sticks to the bottom until the user scrolls up, which pauses
  auto-scroll and shows a "↓ N new" jump-back indicator.
- **Filter toggles**: All / Warnings / Errors.
- **Cross-linking**: clicking a warning/error line highlights the corresponding grid cell (they
  share the export/folder identity from the event payload); clicking the line's **message ID**
  opens the help window at that message's catalog row (see "Help window").

### Status bar

One row, 24px, full width under sidebar and tool view, present in every tool. It is the home for
everything that is *about the environment or about another tool* — the things that previously
had no home and were described as "toasts" in three places. It is not a second log: a message
about the current tool's own work (an export, a scope, a run) goes to that tool's Log panel or
run strip, never here. Three slots, each admitting one typed item kind from `studio_core::status`:

| Slot | Item | Admits | Lifetime |
|---|---|---|---|
| Left | `ShellCondition` | Persistent environment states the **shell** computes itself: `data_dir_read_only`, `update_available` (with an Install action), `pes_install_missing`. Each has a tooltip and at most one action. Not the PES-exe mismatch — the version selector's coloring already owns that | While the condition holds; disappears by itself |
| Middle | `ToolActivity` | Tools working in the background while a **different** tool's view is shown — `activity()` polled each frame for every registered tool except the active one, e.g. `Team compiler · 12/48 · 01:12` while the save editor is open. The active tool's own run strip is unchanged, so nothing shows twice | While the tool reports it |
| Right | `Notice` | The most recent one-off event, published by a tool through `ctx.notify(Notice)` or by the shell: `Switched to PES 2021 · [Fix PES path]`, `Compiled 48 teams · [Open output]`, `Teams list is read-only`. Text plus at most one action | A few seconds; with an action, until clicked or dismissed. Older notices are not kept — an event worth keeping is also a finding in a log |

Rules that keep it worth the row:

- **Typed, never free text.** Tools cannot draw into the bar; they return a `ToolActivity` and
  publish `Notice`s. Conditions are the shell's alone. A `set_status_text(String)` API is exactly
  how a status bar becomes clutter, and it does not exist.
- **Never modal, never a question.** Anything needing a decision (updater consent, the teams-list
  merge review) is a dialog. The bar announces; it does not ask.
- **Empty state is static, not blank**: `v1.2.0` in small text — the one always-visible version
  number, the line a user pastes into a bug report (see "Changelog and version display"), and
  nothing else. The data location is not shown: it is a choice made once and on purpose, its one
  failure mode already has the `data_dir_read_only` condition, and the About block carries it for
  the rare report that needs it. The row is always present at a fixed height, so a notice never
  shifts the tool view; auto-hiding was rejected for that reason.
- **After an update**, the first start of a new version publishes the shell notice `Updated to
  v1.4.0 · [What's new]`, whose action opens the help window at the "What's new" chapter. No
  popup: a notice can be ignored and dismissed; a what's-new dialog is read by nobody who did
  not ask for it.
- **CLI equivalence:** conditions print once at startup on stderr, notices print as they occur;
  the same items, rendered as lines.

### Window title

The title bar is the one surface a user sees **while the window is not in front** — the taskbar
button, alt-tab, a second monitor — so it carries what matters then: whether the Studio is
still working, and what it just finished. Red did this with the console title (`1 - …`, `2 - …`,
`Done - …`); the Studio does the same with more detail, from the same typed items the status bar
uses, so it costs no new API:

| State | Title |
|---|---|
| Idle | `4cc Studio` |
| A tool reports a `ToolActivity` | `Team compiler · 12/48 · 01:12 — 4cc Studio` |
| Idle, window unfocused, a `Notice` arrived since focus was lost | `Compiled 48 teams — 4cc Studio` |

Rules:

- **Progress first, name last.** Taskbar buttons truncate from the right; the part worth
  reading is the number, not the app name.
- **Every tool's activity counts, the active one included.** The status bar's middle slot hides
  the active tool's activity because its run strip already shows it; the title has no such
  duplicate, so the shell polls `activity()` for all registered tools. Several at once: the
  active tool's first, then sidebar order, joined with ` · ` — in practice one, since almost
  every activity is a compile.
- **The "Done" state is the last `Notice`, held while unfocused.** The shell keeps the latest
  notice's text in the title from the moment it arrives until the window next gains focus, then
  returns to idle. That is Red's `Done -` with the result in it, and it falls out of the notice
  the tool already publishes (`Compiled 48 teams · [Open output]` — the action is not in the
  title). While focused the notice is in the status bar and the title stays idle.
- **Never the version, never a condition.** The version lives in the status bar (see "Changelog
  and version display"); a `ShellCondition` is an environment fact that outlasts any glance at
  the taskbar and would read as a stuck process.
- Set through `ViewportCommand::Title` only when the string changes, once per frame at most.
  The Windows taskbar progress bar (`ITaskbarList3`) is **not** used: neither winit nor eframe
  exposes it, and a text title covers the need without platform code.

### Event wiring

The tool crates emit `PipelineEventEnvelope` values via channels. The GUI owns the receiving end and
renders state in the egui update loop:

```rust
// Pipeline workers send stable-ID events through the stale-result envelope:
tx.send(PipelineEventEnvelope {
    run_id,
    export_id: Some(export_id),
    export_revision: Some(export_revision),
    event: PipelineEvent::FolderStatus {
        export_id,
        path: folder_path.clone(),
        status,
    },
});
// Messages are sent as separate enveloped events; the GUI associates them by scope.
for msg in messages {
    tx.send(PipelineEventEnvelope {
        run_id,
        export_id: msg.scope.export_id(),
        export_revision: revision_for(&msg.scope),
        event: PipelineEvent::Message(msg),
    });
}

// The GUI drains the channel once per frame. eframe calls `logic` before every
// `ui` pass (and also while the window is hidden, when a repaint was requested),
// so event draining lives there and `ui` only renders the resulting state:
impl eframe::App for StudioApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drain all pending envelopes and reject stale revisions before applying.
        while let Ok(envelope) = self.event_rx.try_recv() {
            if self.is_current(&envelope) {
                self.apply_event(envelope.event);
            }
        }

        // Background pass: every tool ticks (event draining, timer deadlines),
        // then only the active tool's view renders
        for tool in &mut self.tools {
            tool.tick(&self.tool_ctx);
        }

        // Keep repainting while the pipeline is running. egui otherwise only
        // repaints on input, so background sources schedule their own wakeups:
        // timer deadlines via request_repaint_after, the match feed by
        // requesting a repaint on publish.
        if self.pipeline_running.load(Relaxed) {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Render: grid reads current state, log reads buffered lines
        // (panel layout as in "Shell layout")
        self.render_sidebar(ui);
        self.render_tool_view(ui);
    }
}
```

The grid, log panel, and run strip are all views over the same shared state, updated from the
same event channel. The stadium compiler view reuses the same grid rendering code with rows =
stadiums and cells = components.

### Other UI components

| Component | Implementation |
|-----------|---------------|
| Common and per-tool settings forms | egui widgets (`TextEdit`, `CheckBox`, `ComboBox`), saved to TOML |
| Export tree browser | `egui::CollapsingHeader` tree, shows object model |
| File/folder pickers | `rfd` for native dialogs and supported web file picking; web directory access needs a separate File System Access API adapter (`rfd` has no WASM folder picker) |
| Dark/light theme | egui dark/light modes through the shared `Visuals` theme preset in `studio_core`; icon glyphs from `egui-phosphor` |

### Cancellation

During a run the Compile button becomes Cancel. Rust's rayon scopes or spawned tasks support
cancellation via an `AtomicBool` flag. The pipeline checks the flag between tasks and aborts
cleanly.

### Browser deployment: Studio Web

The same `studio` binary crate compiles to WebAssembly (the eframe `wasm32` target), producing
**Studio Web**: a browser-served, deliberately *lite* edition of the suite. Its purpose is
convenience, not parity — a regular user on a PC other than their usual one opens a bookmark and
compiles their own team or edits its settings, without hunting for the tool's download link. It is
never meant to compile a full cup's worth of DLC; that stays the desktop binary's job. The name
"Studio Web" is user-facing, precisely so nobody expects the desktop feature set from it.

The user opens the URL, the WASM application loads, and from that point it works with local files
via the browser's File System Access API (`showDirectoryPicker`).

**Tool eligibility.** Tools split into three tiers by how much of the desktop platform they lean on:

| Tier | Tools | Web cost |
|---|---|---|
| Web-eligible, cheap | Save editor, Kit config editor, Music export editor, Export upgrader checks — tools that read a few user-picked files and write results back | Only the async file-access adapter; the lib crates and the egui shell compile unchanged |
| Web-eligible, pipeline-bound | Team compiler (single team), Balls compiler, Stadium compiler | The pipeline adapters below; single-export scale only |
| Desktop-only by construction | Match tracker (Win32 memory), Player aesthetics editor's Blender launch, self-update, elevation, live folder watching | Not offered; the WASM build hides them (saved-match viewing stays available — see the Match tracker plan) |

Differences from the desktop build:
- **No folder watching**: browsers don't support `notify`. The user picks their exports folder
  manually; a "Rescan" button replaces the automatic watcher.
- **File access is permission-gated**: the user must grant access to each folder (exports, output,
  PES install, savefile). This is a browser sandbox constraint, not an egui limitation.
  `showDirectoryPicker` requires a secure context (HTTPS) and is Chromium-only at the time of
  writing; Firefox/Safari users get single-file picking, which covers the cheap tier but not
  folder-based exports.
- **Pipeline adapters are the real cost**: filesystem access is asynchronous and handle-based;
  blocking I/O, ordinary threads, and the `Condvar`-based `MemoryBudget` backpressure are not
  drop-in browser APIs. The pure format/domain logic is reused as-is; `libs/pipeline`'s scheduling
  and I/O get browser variants. Parallel WASM (`wasm-bindgen-rayon`) needs `SharedArrayBuffer`,
  which requires the server to send COOP/COEP headers (`Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Embedder-Policy: require-corp`). Once built, every pipeline change carries a
  dual-target tax (`#[cfg(target_arch = "wasm32")]` forks in the scaffolding), which is why this
  tier ships after the desktop pipeline has settled, not alongside it.
- **Memory ceiling**: `wasm32` addresses at most 4 GB, and browsers typically grant less. Exports
  average ~200 MB and texture conversion expands them, so a single team compiles comfortably while a
  multi-team run does not. Studio Web enforces a small in-flight limit (one or two exports) rather
  than exposing the desktop thread/memory settings.
- **Embedded resources**: the ~6 MB of templates and fallback bins embedded in the desktop exe (see
  "Resolved decisions" in the Team compiler plan) are demand-paged on desktop but fully downloaded
  and resident in a `.wasm`. The template accessor (embedded → shadowed by the `templates/` override
  directory) is the single seam: the web build may swap the embedded source for lazy fetches behind
  it if the download size matters.
- **Native-only features stay unavailable**: live process tracking, Blender/PES launch, elevation,
  and binary self-update. Confirm archive and audio support on the chosen browser target before
  promising desktop parity.

**Hosting.** The intended host is the community's own server (the `implyingrigged.info` wiki
machine: nginx on Debian, admin access available), which can set the COOP/COEP headers that
GitHub Pages cannot; the `coi-serviceworker` workaround stays as a fallback for header-less hosts.
Deployment is a static directory (`.wasm`, JS glue, `index.html`) under an HTTPS location; the
build runs `trunk`/`wasm-bindgen` in CI and the release job copies the output over. Suitability is
evaluated when the web tier is scheduled — the requirements are just static files, HTTPS, and two
response headers.

**Keeping the door open from Phase 1.** Studio Web is not a first-release gate (see "Phase 18"),
but the codebase must not drift into unportable dependencies while it waits. Workspace guardrail 6
(`cargo check --target wasm32-unknown-unknown` for every lib crate in CI) enforces this from
the first commit; it costs minutes per PR and turns the later web build from an archaeology project
into UI wiring plus the pipeline adapters. The desktop binary remains the primary distribution.

### Architecture separation

```
lib crates (pes_savefile, cpk, fpk, fmdl, pes_model, ftex, dds_convert, fox2, vtree, archives, teams_list, aesthetics_export, pipeline, model_convert, aatf, music_export, audio_engine, match_feed, kit_config, ...)
    ↑ used by
tool crates (team_compiler, save_editor, stadium_compiler, export_upgrader, music_player, music_export_editor, match_tracker, kit_config_editor, refs_arranger, balls_compiler, player_aesthetics_editor) — each implements StudioTool
    ↑ registered by            ↖ trait + shell + common widgets defined in studio_core
studio (single binary: GUI when run without args, CLI subcommand dispatch with args;
        native binary + WASM web build)
```

The `pause()` pattern from Blue/Red disappears entirely. The pipeline returns warnings/errors as
data. The GUI or CLI layer decides how to present them (log panel, status-bar notices, dialogs for
decisions, console output).

---

## Parallelism

### Design

**One model: `rayon` for parallel work, `crossbeam-channel` for hand-off, plain threads for
background tasks (update check, folder watching). No async runtime anywhere in the workspace — and
"anywhere" includes dependencies: a crate that starts its own Tokio/smol runtime internally (as
`reqwest` does even in blocking mode) is not approved, because the rule is about what runs in the
process, not about which module wrote `async`. The reasons: every I/O-bound thing the suite does
fits a thread; an async runtime would be a second concurrency model for agents to mix with the
first; and the `fmdl`/`pes_model` PyO3 constraint (guardrail 4) already forbids runtimes in the
libs those crates sit next to.**

Replace Blue's `ThreadPoolExecutor` + `console_lock` with `rayon` work-stealing:

```rust
// The Team compiler resolves exports into a run-level BuildManifest
// (serial planning: IDs, dependencies, output paths, collision decisions),
// then processes each manifest task independently. The object model lives in
// libs/aesthetics_export; compile behavior is in tool-local free functions.
let refs_preflight = reader_duplicate_refs_preflight(&discovered_exports); // before validation
let mut plan = plan_run(resolved_exports, &compile_ctx); // only identity-resolved ResolvedAestheticsExport values
plan.merge_discovery_preflight(refs_preflight); // messages + dropped ExportIds
emit_messages(&plan.messages);
let Some(manifest) = plan.manifest else {
    return; // AbortRun/global fatal; scoped drops still return a manifest
};
manifest.tasks.into_par_iter().for_each(|task| {
    let batch: TaskBatch = process_task(task, &compile_ctx); // producer ID, entries, messages, permit
    writer.send(batch);
});
```

`PlanReport` carries the manifest, all planning messages, and the IDs of exports dropped by scoped
planning failures. The Team compiler separately performs duplicate-refs discovery preflight before
validation, then merges that preflight's messages and dropped IDs into the same final report;
`plan_run` continues to accept only identity-resolved `ResolvedAestheticsExport` values. Only an
`AbortRun`/global fatal produces no manifest. No processing-time locks are needed for output
allocation — manifest planning establishes task independence before parallel work starts. The borrow
checker enforces safe memory access, not disjoint output paths; manifest collision checks and tests
establish the latter. No GIL, no `console_lock`, no free-threaded Python workarounds.

### Thread count

```rust
fn thread_count_detect(requested: usize) -> usize {
    if requested > 0 { return requested; }
    let logical = available_parallelism().map(|count| count.get()).unwrap_or(1);
    // Reserve one core for the reader and writer; minimum 1 worker
    logical.saturating_sub(1).max(1)
}
```

### Memory budget

Blue's `MemoryBudget` with backpressure exists because exports are large — approximately 200MB on
average, meaning a full 48-team compilation has ~9.6GB of raw export data in flight. With 7 parallel
workers, that's 1.4GB minimum just for in-flight exports, before counting the writer's CPK buffers,
texture conversion temporaries, and the OS. Users with 8GB RAM (common in the modding community)
would hit memory pressure without backpressure.

**Rust does not eliminate the need for a budget.** Ownership gives deterministic lifetimes, but
allocator behavior, parsed representations, and conversion buffers still add overhead. There is no
fixed Rust-versus-Python RSS ratio to rely on; measure the actual pipeline on representative inputs.

**The budget works at loaded-`BuildTask`-scope granularity**, improving on Blue's per-export
accounting: an export's structure (folder tree, names, sizes) is loaded eagerly but is tiny; each
model, kit, logo, Common, collar, portrait, or referee-marker task loads content lazily and owns its
lease and derived Arc clones until the writer drains its `TaskBatch`. Memory therefore drains
continuously during a run rather than in whole-export steps. The bounded exception is a
Team-compiler `PlayerBatchGroup`: face/boots/gloves child processing remains independent, but
permit-charged bytes shared by those children stay charged until every child reports and the group's
one shared-texture batch commits. Solid `.7z` archives are the other exception (one-pass
decompression forces whole-export residency); plain folders and `.zip` support lazy per-folder
loading. Details are in the Team compiler plan's walkthrough.

A task's charge includes source bytes, decoded textures, converted/merged models, and packed
entries; shared bytes and the bounded conversion cache stay charged while retained. Source size
alone does not bound peak memory: texture decoding and conversion can expand it substantially.
The goal is to bound live work independently of the number of teams, not to promise a fixed 2×
source-size multiplier. Complete accounting is an implementation gate (see the Team compiler plan).

Admission must remain progress-safe when a task already owns source/archive bytes and needs
conversion workspace, and when canonical writer order delays later batches. Do not let all workers
wait for extra capacity that only their own completion could release. Cancellation must also wake
budget waiters; the semaphore sketch below shows capacity accounting, not the full cancellation or
multi-allocation admission protocol.

**Keep the memory budget.** The implementation is a straightforward semaphore:

```rust
pub struct MemoryBudget {
    cap: usize,
    // The condition and the value it guards use the same mutex. Updating an
    // atomic outside this mutex could notify between a check and wait, losing
    // the wake-up and leaving an acquirer asleep indefinitely.
    in_flight: Mutex<usize>,
    notify: Condvar,
}

impl MemoryBudget {
    pub fn acquire(&self, size: usize) {
        let mut in_flight = self.in_flight.lock().unwrap();
        if size > self.cap {
            // Oversized request: wait for the pipeline to drain, then
            // proceed alone — waiting for ordinary capacity would deadlock.
            while *in_flight > 0 {
                in_flight = self.notify.wait(in_flight).unwrap();
            }
        } else {
            while (*in_flight).checked_add(size).map_or(true, |total| total > self.cap) {
                in_flight = self.notify.wait(in_flight).unwrap();
            }
        }
        *in_flight += size;
    }

    pub fn release(&self, size: usize) {
        let mut in_flight = self.in_flight.lock().unwrap();
        assert!(*in_flight >= size, "memory-budget permit released twice");
        *in_flight -= size;
        self.notify.notify_all();
    }
}
```

---

## Distribution and updates

### License

**`MIT OR Apache-2.0`** — the Rust ecosystem's dual license, declared once as
`license = "MIT OR Apache-2.0"` in the workspace `Cargo.toml` and inherited by every crate;
`LICENSE-MIT` and `LICENSE-APACHE` at the repository root; contributions are accepted under the
same terms (the README says so). Red stays GPL-3.0; Studio is an independent reimplementation of
its ideas, not a derivative of its code (see "Relationship to legacy tools"), and Red's author is
this project's author, so nothing in Red's license binds Studio.

Permissive rather than copyleft because the suite is *designed as libraries with tools on top*,
for a community that writes its own tools in whatever language the author knows: a GPL
`pes_savefile` or `cpk` could go into a GPL Blender add-on and nowhere else, which would cancel
half the reason the crate boundaries exist. The copyleft threat — a closed fork — has never
materialized in this community's tooling and would do it no harm if it did; the open version
stays. Apache alongside MIT for the explicit patent grant and because it is what every dependency
uses. The door MIT closes (absorbing GPL code) leads nowhere: Red is the author's own, 4ccEditor
is zlib, and the other legacy tools carry no license at all — unusable verbatim under any license.

What the license does **not** cover: game-derived data kept for interoperability — the skeletons
in `resources/skeletons/`, `DpFileList.bin` and the other embedded `.bin` templates, the kit-icon
reference sheet. They are Konami's; the README states it. `just deps-check` enforces the
dependency side with a `cargo deny` license allowlist (`MIT`, `Apache-2.0`, `BSD-2-Clause`,
`BSD-3-Clause`, `ISC`, `Zlib`, `Unicode-*`, `MPL-2.0` for file-level copyleft crates only, plus
the font licenses `OFL-1.1` and `Ubuntu-font-1.0` that egui's bundled default fonts carry), so a
copyleft crate cannot enter the dependency graph unnoticed. The list lives in `deny.toml`.

### Distribution: portable .7z bundle

There is no installer. Each release is a `.7z` archive containing:

- `studio.exe` (or the Linux binary) — the single self-contained binary
- `quick_compile.bat` (`quick_compile.sh` on Linux) — the double-click launcher: opens the GUI on
  the Team compiler and starts a compile (see "Launch modes" under Architecture)
- a basic readme (pointing into the app: the full help and the changelog live in the help window
  — see "Help window" under GUI Design — and `studio help --export` produces the same text as
  files for the release page), with the two license files beside it

The current teams list is not a bundle file: it is embedded in the binary and materialized as
`data/teams_list.txt` on first run (see the [Team compiler plan](team_compiler.md), "Resolved
decisions", "Teams list").

The user extracts the archive anywhere and runs the binary — or, for the common case, drops an
export into `exports/` (shipped empty beside the binary) and double-clicks
`quick_compile.bat`, which is the whole Red workflow carried over. Where the suite's persisted data goes
(the settings file, state like the skipped-update version, logs) is the user's choice, asked once on
first run — see "Data location" below. This is the same folder model Red uses, minus the ~80MB
embedded Python runtime that made Red's updates a folder-replacement operation.

### Data location: asked on first run

Some users want a fully portable install, others want their data to survive replacing or moving the
program folder. Following EGG-Translator's first-start wizard, the first run asks where to store
settings and data, with two options (each with a one-line hint, like EGG's buttons):

- **Next to the executable** — a `data/` subfolder beside the binary. Fully portable: the folder
  carries the program and everything it persists; deleting it removes the program completely, moving
  it moves everything.
- **In the user config directory** — `%APPDATA%\4cc-studio` on Windows, `~/.config/4cc-studio` on
  Linux (via the `directories` crate). Survives deleting or replacing the program folder.

Resolution at startup is **presence-based, no marker files** (EGG's mechanism): if
`data/settings.toml` exists next to the binary → portable mode; otherwise if the config-dir settings
file exists → config-dir mode; if neither → first run, show the dialog, and write the settings file
to the chosen location — the file's existence is what makes the choice stick. Since the portable
location wins when both exist, choosing the config dir while a portable settings file is present
renames that file to `.bak` so it can't shadow the choice on the next start (EGG's exact fix for
this edge).

`data/teams_list.txt` *is* data-dir cargo (the grid writes into it; see the [Team compiler
plan](team_compiler.md), "Resolved decisions", "Teams list"), and writes to it are best-effort:
an unwritable data directory makes the list read-only, never triggers elevation. The following
things are *not* data-dir cargo:

- The `exports/` folder (the default `exports_folder_path`) lives **next to the binary in both
  modes**, beside `quick_compile.bat`. Copying exports into `%APPDATA%` is impractical and would
  break the "drop it in the folder, double-click" habit Red established; users who want it elsewhere
  point the setting at another path.
- The `output/` folder (the default `output_folder_path`) likewise lives next to the binary — it is
  Red's `patches_output/`, where users look for a CPK when a deployment could not happen or
  `--no-deploy` was given; it also hosts the run-scoped staging folder (see "Post-processing" in
  the Team compiler plan).
- The `old/` rollback binary is tied to the program folder, not the data.

The question is desktop-only: the browser build has no exe directory and persists to browser
storage.

### Self-update: in-place binary swap

Red updates by downloading the new release's 7z, extracting it as a **sibling folder**, and
migrating user data into it (settings via a transfer table, `teams_list.txt` via an interactive
diff, `exports/` contents, state files) — necessary because Red's install folder is a pile of
engine files that must be replaced wholesale, with user data interleaved. 4cc Studio's folder is
a few files, so the update inverts: **the folder stays, the binary is swapped in place**, and there
is almost nothing to migrate.

The desktop flow is coordinated by `studio` (the binary). `studio_core::updater` provides generic
release checking, download/verification, and binary replacement; `studio` separately calls
`teams_list`'s parse/reconcile logic for teams-list reconciliation. Core knows neither
teams-list/team-ID semantics nor team-format types. No plugin hook or new updater framework is
needed. This is desktop-only (`#[cfg(not(target_arch = "wasm32"))]`); the browser build updates
server-side:

1. **Check** (startup, non-blocking background task): if `check_for_updates` is on and the last
   check is older than the check interval, query the GitHub Releases API (`releases/latest`),
   compare the `tag_name` semver against the built-in `CARGO_PKG_VERSION`. The release `body`
   (release notes) comes back in the same response.
2. **Offer**: if a newer version exists and it isn't the `skipped_version`, show a dialog with the
   release notes inline and the options *Update now* / *Skip this version* / *Remind me later*; a
   *Disable update checks* option lives in settings (kept out of the dialog — Red's `fuckoff`). The
   CLI equivalent is a one-line notice plus a `studio update` subcommand that performs the
   check+apply non-interactively.
3. **Download + verify**: fetch the release's `.7z` asset (the same artifact users download manually
   — one artifact serves both paths) into a temp folder, verify its SHA256 against a checksum asset,
   and extract it with the already-in-workspace `sevenz-rust`. The download **streams** the
   response body (`ureq`'s `into_reader()`) into the temp file, hashing and counting bytes as it
   goes — that is where the dialog's progress comes from, and it sidesteps `ureq`'s 10 MB default
   in-memory body limit, which the convenience readers (`read_to_vec`/`read_to_string`/`read_json`)
   enforce and a release asset exceeds (`into_reader()` is unbounded; `into_with_config().limit()`
   changes either). Both HTTP calls run on a plain background thread.
4. **Swap**: first stage and verify the new binary on the executable's volume. Move the running
   binary into `old/`, then put the new binary in place; if replacement fails, restore the original.
   This is a recoverable sequence, not one atomic swap; handle file locks and permissions explicitly.
   If the folder isn't writable (e.g. Program Files), use the `elevation` lib's elevated-relaunch
   path or download-and-instruct. Restart only after active work is stopped and unsaved edits are
   handled by the ordinary close prompts.
5. **Teams list reconciliation** (on the first start of the new binary, not during the swap): the
   new binary carries the new upstream list embedded; the working `data/teams_list.txt` may contain
   user-added ID assignments (the grid's inline ID-cell writes — see the [Team compiler
   plan](team_compiler.md)), so it can't just be overwritten. The `studio` coordinator calls
   `teams_list`'s reconciliation, not the generic core updater, to **merge** instead
   of showing Red's pick-one diff: rows only in the embedded list are added, rows only in the
   working list (user additions) are kept, and conflicting rows (same team, different ID) take the
   embedded list's value. The result is presented as a reviewable summary (added / kept /
   overridden) before being written. Validate uniqueness of both team names and IDs before
   publication: a new team's ID may collide with a retained local team's ID even when their keys
   differ. Unresolved conflicts leave the list unchanged and are reported for review (the
   noninteractive CLI does not guess). An unwritable data directory skips the merge with
   `teams_list_read_only`; the embedded list is not used in place of the working one, so the
   user's assignments keep winning until they move the install. The readme and
   `quick_compile.bat` are overwritten without ceremony (not user-edited).
6. **Restart**: offer to relaunch. The settings file needs no transfer — it stays where it is, and
   the existing merge of `default_settings()` for missing keys (see "Settings menu") absorbs
   new-version settings changes, which is what retired Red's transfer-table mechanism.

### Rollback

`old/studio.exe` is kept (one version deep — each update replaces it), and the settings menu gets a
**"Roll back to previous version"** button that swaps the two binaries back and relaunches. This
covers the "new version is broken" case that Red handles by preserving the entire old folder.

### Update state

Stored with the settings (common settings, `studio_core`):

| Key | Replaces (Red) | Default |
|-----|----------------|---------|
| `check_for_updates` | `updates_check` ini key | `true` |
| `check_interval_hours` | hardcoded 60 minutes | `24` (a resident GUI app, not a per-compile CLI run) |
| `last_update_check` | `state/update_check_last.txt` | — |
| `skipped_version` | `state/update_skip_last.txt` | — |

This closes out Red's entire `Engines/state/` folder — the remaining five files either became
settings or lost their reason to exist:

| Red state file | Fate |
|---|---|
| `dt00_write_allowed.txt` | The `dt00_overwrite_allow` setting (team compiler) — standing consent as a setting, not a marker |
| `first_run_done.txt` | Presence-based data-location resolution: the settings file's existence *is* the first-run marker (see "Data location") |
| `admin_warned.txt` | Dropped — the one-time console explanation before Red's UAC relaunch becomes the `elevation` lib's GUI prompt, which explains itself every time it appears |
| `sideload_warned.txt` | Dropped — the one-time sideload explanation becomes the per-run `sideload_active` Info message (team compiler catalog) |
| `ver_mismatch_warned.txt` | Dropped — the suppressible console notice becomes the plain per-run `pes_version_mismatch` warning (team compiler catalog) plus the version selector's live red/yellow exe check (see "Sidebar") |

### Versioning: one workspace version, one exception

The suite uses a **single workspace version** for every crate that ships in the `studio` binary. Set
once in the root `Cargo.toml`:

```toml
[workspace.package]
version = "1.4.0"
```

Every tool and lib crate inherits it via `version.workspace = true`. Bumping is one line, applied to
all crates at once.

Rationale:

- **The binary is the release unit.** Users download `studio.exe`; the self-updater compares
  `CARGO_PKG_VERSION` against a GitHub Release tag. The version they see *is* the suite version.
  Independent crate versions would be invisible to every actual consumer.
- **Nothing is published to crates.io.** Per-crate semver exists to signal breaking changes to
  downstream *crate* consumers; there are none; every internal crate's only consumer is `studio` (or
  another internal crate), all compiled together in one binary.
- **Tools are compile-time plugins**, statically registered in `studio`. A "team_compiler 2.0 vs.
  studio 1.5" compatibility question is impossible by construction — a version mismatch can't ship.
- **Changelog and rollback are per-Studio-release.** Release notes are "4cc Studio 1.4 adds the
  Balls compiler"; the rollback button swaps two binaries in `old/`. Both only make sense on one
  version axis.

A release is cut (tag, GitHub Release, `.7z` asset) when there's something to distribute; not every
commit is a release. The workspace version is the next-release version; the tag is what the updater
compares against.

**One exception: `python_bindings`.** It is built via maturin into a wheel that Blender users
install independently of the Studio binary, and its consumers are not Studio users. Its version is
meaningful to them (which wheel matches their Blender / which `fmdl`+`pes_model` API), so it keeps
its own `version = "..."` and its own release cadence — shipping on demand when the
`fmdl`/`pes_model` API surface it exposes changes. This is also why the plan builds it outside the
default `cargo build` path.

### Changelog and version display

**One changelog, three readers.** `CHANGELOG.md` at the repository root, in the Keep a Changelog
shape: `## [Unreleased]` on top, then one `## [1.4.0] — 2026-10-02` section per release; inside a
section, bullets grouped by tool (`**Team compiler** — …`, `**Save editor** — …`, `**Studio** —
…`), because that is how a member reads it even though the version axis is one. It is written in
the step that changes the behavior, like the help topic (see "Help window"). Nothing else
describes releases; the three places a user meets release notes all read this file:

1. **The help window's "What's new" chapter** — the file embedded at build time, newest first.
2. **The GitHub release body** — the release process copies the released version's section
   verbatim into the release notes. The updater dialog shows that body inline (see "Distribution
   and updates" step 2), so the dialog, the release page and the in-app chapter are the same text
   by construction rather than by discipline.
3. **The post-update notice** — `Updated to v1.4.0 · [What's new]` in the status bar on the first
   start of a new version, opening reader 1 (see "Status bar").

**One version string, shown in one place.** `v{CARGO_PKG_VERSION}`, with ` (debug)` appended
when `cfg!(debug_assertions)` — enough to tell a local build from a release, which is the only
distinction that has ever mattered in a bug report. No git hash: it would need a `build.rs` or
`vergen` for information the tag already carries, since releases are built from tags. Where it
appears:

- **Status bar empty state** — the one always-visible place, and the line to paste into a report.
- **About** (logo click) — the detailed block with Copy (see "Sidebar" item 1).
- **`studio --version`** — the same one-liner on stdout (CLI equivalence).
- **Not the title bar**: the title is what the taskbar and alt-tab show — the surface for what
  the Studio is *doing* while the window is not in front (see "Window title"), and it is outside
  every screenshot a user crops to the window content. **Not beside the logo**: duplicate of the
  status bar, and homeless when the sidebar collapses to the icon rail.

---

## Development Plan

Phases 1–2 build the platform skeleton and every lib crate that can be verified on its own (against
fixtures or reference outputs); the tools follow, each bringing the consumer-shaped libs it is the
first to need. Each phase's detailed specification lives in the corresponding plan document: phase
2 in [libs](libs.md)/[model conversion](model_conversion.md)/[savefile](pes_savefile.md), phases
3–4 in [team compiler](team_compiler.md), phase 5 in [savefile](pes_savefile.md)/[save
editor](save_editor.md), phase 6 in [export upgrader](export_upgrader.md), phase 7 in [model
conversion](model_conversion.md), phase 9 in [stadium compiler](stadium_compiler.md), phase 10 in
[music player](music_player.md)/[music export editor](music_export_editor.md), phase 11 in [match
tracker](match_tracker.md), phase 12 in [kit config editor](kit_config_editor.md), phase 13 in [refs
arranger](refs_arranger.md), phase 14 in [balls compiler](balls_compiler.md), and phase 15 in
[player aesthetics editor](player_aesthetics_editor.md).

### Phase 1: Workspace bootstrap + core skeleton

Done (2026-09-13). The guardrails above apply "from the first commit", so the first commit is the
one that makes them enforceable:

- The root `Cargo.toml` is a virtual workspace (`crates/studio`, `crates/studio_core`,
  `crates/libs/*`; `crates/tools/*` joins with the first tool crate, since cargo rejects a member
  glob that matches nothing). `[workspace.dependencies]` declares every shared crate once, and
  `[workspace.lints]` holds the policy every member inherits via `[lints] workspace = true`:
  `rust::missing_docs` (the `///`-on-every-`pub` rule) and `clippy::let_underscore_must_use =
  "deny"` (a discarded `Result` is a compile error unless handled or `#[expect]`ed with a reason,
  see the code style rule in `CONTRIBUTING.md`), plus `dbg_macro`, `print_stdout`,
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
  plain PowerShell, the fallback `CONTRIBUTING.md` named; a planted clippy warning was verified to
  turn `just gates` red there. The remaining recipes the plan names (`acceptance`, `parity`,
  `bindings`, `release`) arrive with the phases that give them something to run.
- CI (`.github/workflows/ci.yml`) runs `just gates` on Ubuntu and Windows and `just deps-check`
  on Ubuntu, installing `just` and `cargo-deny` as prebuilt binaries; the `python_bindings` build
  job is added in Phase 2 with the crate.
- `crates/libs/`, `crates/tools/`, `crates/studio/`, `crates/studio_core/` exist, so every later
  crate lands in its planned place.

The non-GUI parts of `studio_core` exist as the closed module tree above prescribes: `tool.rs`
(the `StudioTool` trait as specified under "Tool plugin interface", `ToolContext`, and the
`ShellRequest` enum the context queues for the shell: `SwitchTool`, `Notify`, `SettingsChanged`),
`events.rs` (the "Event system" types, with `Message` and `MessageCode { tool_id, code }` as the
Team compiler plan's "Message structure" defines them), `status.rs` (`ShellCondition`,
`ToolActivity`, `Notice` with at most one typed `NoticeAction`), `help/mod.rs` (the `HelpSection`,
`HelpTopic`, `HelpTarget` types only; search, rendering and export come with the GUI), `settings/`
(`Settings::load` / `parse` / `save` / `merge_defaults`, the `[common]` + per-tool-table layout
under "Settings menu", `CommonSettings` with the defaults the Team compiler plan lists and the
update-state keys; saving is atomic through a `.tmp` rename and the caller decides what a failed
save means) and `shell/launch.rs` (the three launch modes, `-v` verbosity, one clap subcommand per
registered tool named after its id, duplicate or reserved ids rejected at registration, and
`run_cli` dispatch). Two leaf libs came with it: `vtree` (`ScopePath`, `RelativeScopePath`,
`VirtualTree<T>`, see `libs.md`) because `PipelineEvent` addresses by `vtree::ScopePath`, and
`pes_version` because `CommonSettings` holds a `PesVersion`. `crates/studio` is a stub binary:
it parses the launch mode, installs the CLI `env_logger` sink, and dispatches to a registry that
is still empty; the GUI modes exit with a message until Phase 8.

**Verification (done):** all four gates green on the workspace and `just deps-check` green; 38
unit tests, including settings round-trip and recursive default-merge, `ScopePath` rejection and
collision cases, and a stub tool registered through the trait and dispatched from
`studio stub ping x`. CI's first green run and one deliberately red run are recorded in the
worklog when the first push happens.

### Phase 2: Library crates

Every lib crate that can be verified on its own — against real-file fixtures, the legacy tools'
reference outputs, or documented data — is built here, before any tool. Libs whose API is shaped
by a consumer (`aesthetics_export`, `pipeline`, `aatf`, `music_export`, `audio_engine`,
`match_feed`) wait for the phase of the tool that first needs them. Leaf crates first, then their
dependents; `python_bindings` last, once `fmdl` and `pes_model` are stable.

**Format codecs** with `binrw`. Use Blue and the stadium compiler's parser variants
(`fmdl_file.py`, `cpk.py`, `ftex.py`) as format evidence checked against fixtures, not as
implementations to translate:
- `wezlib`
- `cpk` (read + write, CRILAYLA inside)
- `fpk`
- `ftex` + `dds_convert` (`texture2ddecoder`, `image`, and `block_compression` CPU BC1/BC3/BC7
  plus desktop GPU BC7). Establish the CPU reference and an early GPU integration proof using
  Vulkan on Windows/Linux and Metal on macOS, with CPU fallback rather than automatic DX12 fallback.
  Measure cold pipeline creation and upload/readback cost before relying on launch acceleration.
  BC7 output is PES 19–21 only; PES 15–18 use BC3 or eligible opaque-color BC1 — see [libs](libs.md).
- `fmdl` — `format/` (read/write, byte-identical round-trip), `ops/` (mesh splitting, split vertex
  encoding, anti-blur, multi-FMDL merging, path-table editing), `check.rs`
- `pes_model` — `format/` (.model + sibling .mtl), `ops/` (mesh splitting, split vertex encoding,
  merging, texture stem rewriting), `check.rs`
- `uniparam`
- `fox2` (+ CityHash64, verified against fixtures)
- `archives` (.zip/.7z via `sevenz-rust`)

**Data and rule leaves:**
- `fpc` (Full Player Customization as data: kit values, player presets, interference rules —
  dependency-free, so it precedes `kit_config` and `pes_savefile`; spec in [libs](libs.md))
- `teams_list` (`TeamName` fold, `TeamId` range, `teams_list.txt` parse/write/reconcile with the
  embedded upstream list — dependency-free; fixture: Red's current file; spec in [libs](libs.md))
- `kit_config` (binary ↔ TOML kit config codec, applies `fpc::kit` values — format documentation
  in the [Kit config editor plan](kit_config_editor.md))
- `color_tools` — dominant kit-color extraction only; the picker widget is Phase 8
- `elevation`

**`model_convert`** (spec: [Model conversion](model_conversion.md)) — native formats only; glTF
is Phase 7:
- The `CanonicalModel` IR
- Importers `fmdl_to_ir`, `model_to_ir`; exporters `ir_to_fmdl`, `ir_to_model`, calling the format
  crates' `ops/` for splitting, encoding and anti-blur rather than reimplementing them
- Hand auto-split in the IR, in-process without Blender: select weighted hand vertices, grow once,
  separate — see [Model conversion](model_conversion.md#hand-auto-split)
- The games' skeleton files embedded and exposed per version (`resources/skeletons/`), the render
  hierarchy and fold table beside them, and the retargeting pass
- Material conversion logic
- Cross-version player save data (`convertPlayerSaveData`) belongs to `pes_savefile` below

**`pes_savefile`** (spec: [Savefile](pes_savefile.md)) — the codec and its operations, no tool
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
guardrail 4 with a real `cdylib` build and a smoke test from Blender's Python.

**Verification:** per crate, per [libs](libs.md) "Testing": format round-trips on real fixtures
(byte-identical for `format/`); `color_tools` within perceptual tolerance of manager-picked
`colors.txt`; `model_convert` IR round-trips + conversion output compared against the existing
converters; `pes_savefile` round-trips on decrypted sections (fixed test RNG), per-version field
round-trips, comparison against 4ccEditor edits of the same savefile, transplant/diff parity
against Midcupping; `python_bindings` import + round-trip from Python.

### Phase 3: Team compiler skeleton (`tools/team_compiler`)

**Entry gates:** confirm normal-team/referee `ExportIdentity`, sanitized validated-versus-eligible
projection, and roster-entry scope/disposition semantics (details in the Team compiler plan).

- Define the tool's settings struct and CLI subcommands (via the `StudioTool` trait)
- Build `libs/aesthetics_export`: the source-neutral object model for the Studio export format (canonical
  `vtree::ScopePath` listing → `ParsedAestheticsExport` → sanitized `ValidatedAestheticsExport` →
  `ResolvedAestheticsExport`, with `ExportIdentity`, strong `ValidatedRoster`, `PlayerFolder`,
  `SharedModelFolder`, and `KitsFolder`) plus its folder conventions and validation rules — shared
  with the Export upgrader, Kit config editor, and Refs arranger. Tool-local `BuildManifest`,
  `PlanReport`, `plan_run`, and `process_task` stay in `team_compiler`.
- Implement the pipeline orchestration (reader, coordinator, writer) with `rayon` (shared scaffolding in
  `libs/pipeline`)
- Implement CLI execution with console output (`studio team-compiler compile ...`)

**Verification:** Structure parser and format-level validation tests on Studio-format fixtures;
pipeline scaffolding (reader/coordinator/writer) smoke tests. Packed-output comparison belongs to
Phase 4.

### Phase 4: Processing logic

**Entry gates:** finalize analyzed-output enumeration/namespace allocation, including
folder-internal deep-derived names without adding a separate run phase/type, and freeze every model
`BuildTask`'s planned model-ID assignments.

Turn the Phase 3 skeleton into a compiler that produces game-facing output for Studio-format
exports. `model_convert` (Phase 2) already exists, so cross-format conversion is in scope; savefile
writing arrives with Phase 5. The work is organized by the Team compiler's stages — see "Crate layout" and
the "Per-export pipeline walkthrough" in the [Team compiler plan](team_compiler.md) for the
specification of each. Red module names in parentheses locate reference *behavior*; they prescribe
nothing about the Rust code.

- **Deep validation** — content-level rules in `libs/aesthetics_export` on top of Phase 3's
  structural ones, and `fmdl`/`pes_model` `check.rs` findings, all mapped through the Team
  compiler's message catalog (`export_check`, `xml_check`, `texture_check`). Live-grid scheduling in
  `check/` waits for Phase 8.
- **`plan/`** — `BuildManifest` from a `ResolvedAestheticsExport`, `PlannedModelIds` (40-ID team
  blocks, deterministic boots/gloves assignment — assignment only; the savefile write is Phase 5),
  duplicate-refs preflight, `sideload/` precedence (`team_id_get`).
- **`processing/`** — per-task work over `rayon` with memory permits: model format selection,
  cross-format conversion via `model_convert`, multi-model merging and SKL pairing
  (`fmdl_editing`); texture conversion with the CPU/GPU
  converter integrated as bounded batches with readback, cancellation and CPU fallback — desktop
  GPU BC7 is a first-release feature — plus WESYS wrapping and relocation to common (`textures`);
  MTL / face XML / `materials.toml` editing (`xml_editing`); kit folders: config
  generation/reconciliation/emission via `libs/kit_config` (TOML in exports, binary at compile
  time), FPC patching via `fpc`, kit colors from per-kit `colors.txt` (Team Note entry format) or
  derived from the kit texture via `libs/color_tools` when absent; portraits, logos, collars and
  Common (`portraits_move`, `name_editing`); referee processing per the Team compiler plan's
  "Referee export processing" with Red's observable output as the reference (`referee_tools`,
  referee code in `fmdl_editing`/`export_move`); `materialize.rs` as the single output-mode seam:
  relocation to game paths and Fox FPK packing (`contents_packing`, `export_move`).
- **`bins/`** — UniColor / TeamColor / UniformParameter accumulation via `uniparam`, DpFileList
  read/write and slot discovery (`bins_update`).
- **`output/`** — canonical-order writer, `OutputSink` (CPK vs loose folder for test/sider),
  staging with `.partial` + rename and promotion, writability preflight; access-denied output paths
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
  assignments. `fpc.on`/`fpc.off` markers apply `pes_savefile`'s FPC presets here.
- `libs/aatf`: the AATF rules engine (TOML parameters + scripted checks; decide Rhai vs TOML+CEL
  here).
- The Save editor tool crate's non-GUI substance: settings, CLI, and the operations wiring over
  `pes_savefile` (editing, tactics, AATF checks, comparator, transplant, FPC toggle) that the Phase
  8 view will render — build order in the [Save editor plan](save_editor.md).

**Verification:** Team compiler parity now includes savefile output — compiled savefile compared
against 4ccEditor edits for the same assignments; AATF rule fixtures (known-violating and clean
saves); save editor operations exercised through the CLI on fixture saves.

### Phase 6: Export upgrader (`tools/export_upgrader`)

- Implement old-format export loading (the only place old-format knowledge lives)
- Implement savefile-driven ID→player mapping
- Implement player folder merging (single-user model folders) and shared folder + link file
  generation (multi-user folders)
- Implement kit restructuring (binary kit configs → `config.toml`, `colors.txt`/`icon.txt` from Team
  Note colors)
- Implement `settings.toml` generation from savefile aesthetics
- CLI: `studio export-upgrader ./old_export.zip`

**Verification:** Upgrade the full library of existing old-format exports; compile the results;
compare game-facing output against Red's output for the same exports.

### Phase 7: glTF support

- Add the `gltf` crate dependency to `model_convert` (Phase 2 built its native-format half)
- Implement `gltf_to_ir` and `ir_to_gltf` with a shared PES_bone/PES_mesh and materials.toml contract
- Coordinate with the Blender project's `pes-models` PES glTF codec (its implementation lives there)
- Test with real Blender-exported models

**Verification:** glTF → FMDL → import in Blender → re-export → compare.

### Phase 8: GUI

- Build the `studio_core` app shell (sidebar, tool registry rendering, settings menu with per-tool
  collapsible sections, status bar with its three slots and static empty state, window title
  from activities and the held notice, help window with chapter tree, generated Messages topics,
  search, log-panel message links, and `studio help`)
- Build the common widgets: progress grid, log panel (auto-scroll, filters, grid cross-linking),
  run strip
- Implement each tool's `view()`: Team compiler grid (live validation), save editor
  (player/team/tactics editing, AATF checks, comparator, transplant — build order in the [Save
  editor plan](save_editor.md); the tactics editor, card pickers and violations list are built as
  `libs/team_widgets`, which the Team creator later reuses), export upgrader
- Implement the exports folder watcher (`notify` + debouncing) and the two-tier check system with
  cache
- Wire event channels to egui rendering
- Implement cancellation
- Test on Windows and Linux

**Verification:** Manual testing + user feedback from the modding community.

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

- Release bundling, as the `just release <version>` recipe: the portable `.7z` (binary + launcher
  + readme) with a SHA256 checksum asset, the release body copied from that version's
  `CHANGELOG.md` section (see "Distribution and updates", "Changelog and version display")
- Self-updater: check, dialog, download/verify/swap, teams-list merge, rollback button (see
  "Distribution and updates")
- Documentation: every shipped tool's `help/` chapter written for members (Red's two readmes
  migrated into the Team compiler and core chapters), `CHANGELOG.md` started, `studio help
  --export` wired into the release process; the bundled readme stays a pointer (the window itself
  is Phase 8)

### Phase 17: Team creator (`tools/team_creator`, post-release)

Not a first-release gate: its users are the *next* cup's new managers, and it depends on the
widest set of finished pieces — `libs/team_widgets` and `libs/aatf` with `apply_tier` (Phases 5
and 8), the compiler's kit placeholder and layout-marker rules (Phase 4), `aesthetics_export`
writing, and the Player aesthetics editor to hand off to (Phase 15). Plan: [Team
creator](team_creator.md).

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
the eligibility tiers in "Browser deployment":

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
| `log` | Diagnostic logging facade in every lib and tool crate (see "Diagnostic logging") | Production-ready |
| `env_logger` | CLI-mode log sink in `studio` (`default-features = false`) | Production-ready |
| `pyo3-log` | Forwards `log` lines to Python's `logging` in `python_bindings` | Production-ready |
| `eframe` (egui) | GUI framework (pure Rust, immediate-mode, native + WASM) | Production-ready |
| `egui-phosphor` | Icon font glyphs for the shell's sidebar, toolbar, and status icons | Release not yet reviewed; pinned to the workspace egui version once selected |
| `egui_commonmark` | Markdown rendering for the help window (CommonMark subset: headings, lists, tables, code blocks, links) | Release not yet reviewed; pinned to the workspace egui version once selected. Chosen over a hand-rolled renderer (see "Help window") |
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
| `toml` | Export text formats (`settings.toml`, `config.toml`, `materials.toml`), app settings, AATF parameters | Production-ready |
| `toml_edit` | Comment/formatting-preserving edits to all auto-generated tomls (`settings.toml`, `materials.toml`, `config.toml`); comments are app-injected (predefined per-field documentation) and preserved across edits | Production-ready (the `toml` crate's own foundation) |
| `rhai` OR `cel-interpreter` | AATF rule logic (sandboxed scripting vs declarative expressions; see AATF section) | Production-ready (both) |
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
| Universal model source | glTF (`.glb`, single file; `.gltf`+`.bin` accepted) with *optional* PES extensions (PES_bone/PES_mesh) + materials.toml — own plan: [Unified model format](model_format.md) | Blender exports natively, mature Rust parser, extensible; material properties in comment-friendly TOML, not uncommented JSON. Design goal: one authored model converts to `.model` and FMDL with no loss of customizability in the simplest possible way, or users keep the native formats. Stock-Blender exports must compile (extensions have fallbacks, PBR material data is ignored), which is why a custom `.pes` extension was rejected — it would make the plugin mandatory |
| glTF material files | `materials.toml` (catch-all) + `*.materials.toml` (name-matched) + `.common` links to Common's files, mirroring `.model`/`.mtl` and Red's `.mtl.common` | Materials never embedded in glTF JSON — glTF files need textures anyway, JSON has no comments, and tomls-as-siblings make sharing unambiguous; name-matching (startsWith/endsWith) ports Red's `.mtl` logic for the same modular copy-paste workflow; Common-linked models resolve materials in Common first with local overrides layered on top (Red's cascade); missing material = hard error (no safe defaults without texture paths) |
| Material schema | Shader `family` + engine-neutral keys (booleans, canonical texture roles, parameters) + optional `[fox]`/`[prefox]` native tables | `[face]` alone is a valid material (family `shaded`, textures auto-detected by name); families are named as laymen know them (`shaded`/`shadeless`, not `blin`/`constant`) and encode the converters' shader mapping tables so both engines get a working material from one word; native tables keep same-format round-trips lossless and expose every knob; unknown keys warn (forward-compatible), invalid values error |
| Blender integration | Two complementary paths, packaged as one all-in-one extension (`pes-models`) | Path A: vendored `pes-fmdl`/`pes-model` 5.2 port codecs, hot paths accelerated by the `python_bindings` PyO3 wheel (lossless same-format round-trips, pure-Python fallback). Path B: PES glTF codec (native glTF I/O + `materials.toml` read/write + `PES_bone`/`PES_mesh` extension handling) for new authoring targeting both engines. Plus the Player aesthetics editor's loader module; replaces the standalone addons at release (plan lives with the Blender project) |
| Mesh splitting | CPU only (Rust) | GPU unsuitable for branch-heavy graph partitioning; Rust is fast enough |
| Texture conversion | In-process (`texture2ddecoder` + `image` + `block_compression` CPU/GPU backends + bounded cache) | BC7 supported only on PES 19–21, with first-release desktop GPU acceleration in Auto mode and CPU fallback/explicit CPU mode. BC7/raster sources for 15–18 use CPU BC3 or eligible opaque-color BC1. Codec, alpha, cache, and fallback policy live in [libs](libs.md) |
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
| License | `MIT OR Apache-2.0`; game-derived data excluded and named as Konami's | The suite is libraries first, for a community that writes tools in every language; copyleft would confine the libs to GPL consumers for a protection this community has never needed. See "License" |
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
| Auto-generated toml comments | App-injected (predefined per-field documentation), preserved by `toml_edit` | Users never write comments from scratch; the comments are the simplified documentation, present in every auto-generated `settings.toml`/`config.toml`/`materials.toml`/AATF file |
| FPC toggle | Empty `fpc.on`/`fpc.off` marker files in the player folder | Folder-level like link files; apply `pes_savefile`'s version-aware enable/disable presets (same as the editor's toggle); absent = savefile untouched |
| AATF rules | TOML parameters + scripted checks | Editable without recompiling; Rhai vs TOML+CEL to be decided in Phase 5 |
| Music tools | Two tool crates (`music_player`, `music_export_editor`) + `music_export`/`audio_engine` libs | Rigdio/RigDJ successors; descriptive crate names, old names kept in the user-facing labels ("Music player (Rigdio)") and old icons in the collapsed rail |
| .4ccm format | Canonical, unchanged, read + write | Interop with legacy Rigdio during transition; no new format, no migration for managers |
| Audio backend | Pure Rust: `kira` + `symphonia` (+ Rust Opus decoder) + `ebur128` | Drops libmpv-2.dll and ffmpeg.exe; single binary; in-process loudness; WASM-compatible |
| Match tracker | `tools/match_tracker` crate; per-version memory signatures (PES17 initially); Windows-only backend behind a memory-source trait | SEN:P-AI successor; offsets are data, not code; Win32 is the only backend for now |
| Match events interface | In-process `libs/match_feed` (typed pub/sub + snapshot replay); no pipe/socket, `watch` CLI for external scripts | Tracker and music player share one binary; the old pipe protocol has no surviving consumers |
| Match records | New JSON format (`.match.json`); binary `.sen` dropped | Readable and schema-stable; old archives keep old SEN:P-AI |
| Kit configs | `config.toml` in exports, game binary emitted at compile time; `libs/kit_config` shared by compiler/upgrader/editor | Format recovered by disassembling the abandoned closed-source Kit Manager (documented in the Kit config editor plan); text in exports matches the settings.toml philosophy |
| Music autopilot | Tracker-driven team loading/scoring/goalhorns/event clips; Full, Co-pilot (highlights + automatic buttonless clips), Assist (highlight-only), or Off (suppresses tracker-triggered playback/highlights but does not transfer score authority); manual override always live | The suite's end goal: the music player runs the match unattended, with Co-pilot/Assist/Off as trust-building middle steps |
| Player aesthetics viewing | Blender launch via a versioned JSON manifest consumed by the `pes-models` extension's loader; the in-app quick preview waits for `libs/model_viewport`, created only with the Team compiler's future 3D-preview feature | Blender's viewport/Outliner beat any bespoke wgpu pass and stay the accurate view (the future embedded preview is labeled approximate); export-format knowledge stays in Rust (the loader is manifest-driven); glTF conversion stays Studio-side in `model_convert` (`ir_to_gltf` + superset merge) |
