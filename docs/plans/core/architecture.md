# 4cc Studio — Core plan: Architecture

Part of the [Core plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

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
│       ├── aatf/                     # AATF rules engine (one Rhai rules file: parameters + checks)
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
  buttons) and the status bar (see `gui.md` "Status bar" under GUI Design)
- The `StudioTool` plugin trait (see below)
- The settings framework: common settings + per-tool settings sections, TOML persistence
- The help framework: the help window, its search, and the core chapters (see `gui.md` "Help window")
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
the [Team compiler](../team_compiler/README.md), [Save editor](../save_editor.md), and [Match
tracker](../match_tracker/README.md) plans). Lib crates with a mandated layout are `pes_savefile`,
`model_convert`, and `aesthetics_export` (in their plans) plus the `format/`–`ops/` split of `fmdl` and
`pes_model` (in [libs](../libs/README.md)); the remaining libs are single-concern and get no prescription.

**Lib crates** (`crates/libs/`) — shared functionality with no tool identity. Format and processing
libs stay GUI-neutral; explicitly shared widget libs (`color_tools`, future `model_viewport`) may
use egui/wgpu. Formats exclusive to one consumer still get their own crate when they are conceptually
standalone formats (e.g. `uniparam`); single-consumer helpers stay with their owner (e.g. CRILAYLA
lives inside `cpk`).

**No per-tool companion libs.** Even the largest tool — the Team compiler, at an estimated 15–25k
lines — stays a single crate, organized by internal modules mirroring its pipeline stages (`reader/`,
`check/`, `plan/`, `processing/`, `bins/`, `output/`, `view/` — the full tree and placement rules are
the "Crate layout" section of the [Team compiler plan](../team_compiler/README.md)). A `team_compiler_lib` next to it
would have exactly one consumer, which rule 3 below calls
out, and it would buy nothing for navigation that the module tree doesn't already give. What *is*
extracted is the part with real multiple consumers: `aesthetics_export` (the export format's object model,
folder conventions, and validation) is shared by the Team compiler, the Export upgrader that writes
the format, the Kit config editor that edits kit folders inside exports, the Refs arranger that
edits referee slot allocations inside refs exports, and the Team creator — so all of them
agree on what a valid export is. Should the Team compiler ever need splitting for compile times or a
headless build, the principled seam is logic vs. egui view (see the trade-off note under "Workspace
guardrails"), not a catch-all lib. This rule has already survived its first pressure test: the [Refs
arranger](../refs_arranger.md) was initially conceived as a "refs compiler" owning its own compile
step, which would have forced the engine extraction for one thin extra consumer; instead it only
prepares the refs export's `players.txt` and hands compilation off to the Team compiler (see "Why an
arranger, not a compiler" in its plan).

`fmdl` and `pes_model` have a second consumer outside this workspace: the Blender addons (see
"Blender integration" in the [Model conversion plan](../model_conversion/gltf.md)). Their mesh splitting,
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
--version` (see `gui.md` "Help window", `distribution.md` "Distribution and updates", `distribution.md` "Changelog and version display").

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
    /// only. See `gui.md` "Help window".
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
    /// tool, active or not. Default: nothing. See `gui.md` "Status bar", `gui.md` "Window title".
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
tool puts something in the status bar (see `gui.md` "Status bar"); `Notice` is a typed item,
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
   PR from Phase 1. Studio Web (see `gui.md` "Browser deployment") is a post-release deliverable, and the
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
  report. The GUI `gui.md` "Log panel" shows findings, not this file; an in-app diagnostic view is not
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
