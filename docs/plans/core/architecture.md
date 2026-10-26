# 4cc Studio — Core plan: Architecture

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Architecture

### Crate structure

What exists of this is in `spec/core/architecture.md` "Crate structure".

These planned crates extend the spec's workspace tree:

```
4cc-studio/
├── crates/
│   ├── tools/
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
│   │   ├── ruleset_editor/           # AATF ruleset forms → rules file (libs/aatf's format)
│   │   ├── db_generator/             # The game's database tables from the teams list (libs/pesdb's formats)
│   │   └── ...                       # Future tools (CPK tools, ...)
│   └── libs/
│       ├── team_widgets/             # (Phase 8) Shared egui widgets over the pes_savefile team model: tactics
│       │                             #   pitch/instructions, card pickers, AATF violations list
│       │                             #   (Save editor + Team creator)
│       ├── model_convert/            # glTF+PES_bone/PES_mesh extension import/export + materials.toml
│       │                             #   read/write lives here, via the external `gltf` crate
│       ├── model_viewport/           # (future) shared wgpu model-preview widget — created with the
│       │                             #   Team compiler's 3D-preview feature; consumers: team_compiler
│       │                             #   and player_aesthetics_editor (labeled approximate preview)
│       ├── music_export/             # (Phase 10) .4ccm music export format: parse/write + condition model
│       ├── audio_engine/             # (Phase 10) Audio playback (kira/symphonia) + loudness analysis (ebur128)
│       └── match_feed/               # (Phase 11) Live match event feed (match tracker → music player)
```

**`studio_core`** also contains:
- The app shell: sidebar (logo, PES version selector, tool list, an icon rail opening on hover, help and settings
  buttons) and the status bar (see `gui.md` "Status bar" under GUI Design)
- The help framework: the help window, its search, and the core chapters (see `gui.md` "Help window")
- Common widgets: the progress grid, the log panel, the run strip

These planned modules extend the spec's `studio_core` module tree:

```
crates/studio_core/src/
├── help/
│   ├── search.rs     #   substring search over every topic's paragraphs and catalog rows
│   ├── export.rs     #   `4cc-studio help` printing and `--export` to Markdown files
│   └── core/         #   the core chapters' Markdown (setting up, data location, updates) + CHANGELOG.md
├── shell/
│   ├── title.rs      #   window title from activities / held notice / idle; ViewportCommand::Title on change
│   ├── sidebar.rs    #   logo, version selector (live exe check, running-PES auto-switch), tool list
│   ├── settings_menu.rs  # common section + injected per-tool sections
│   ├── help_window.rs    # chapter tree + rendered topic (egui_commonmark) + search box; opens on the active tool
│   ├── status_bar.rs #   conditions / background activity / latest notice; empty state = the version
│   └── about.rs      #   About popup from the logo: detailed version block with Copy, links
├── widgets/
│   ├── grid.rs       #   progress grid (rows × cells, FolderStatus colors, fades)
│   ├── log_panel.rs  #   capped log, auto-scroll, filters, cross-linking
│   └── run_strip.rs
├── theme.rs          # the shared Visuals preset, fonts, the icon font's registration
└── updater.rs        # release check, download/verify, binary swap, rollback (desktop only)
```

The running-PES probe (`pes_process.rs`, `spec/core/architecture.md` "Crate structure") is
built; the version selector's timer poll over it is Phase 8's (`gui.md` "Sidebar").

#### Launch modes

What exists of this is in `spec/core/architecture.md` "Launch modes".

| Invocation | Behavior |
|---|---|
| `4cc-studio --gui <tool-id> <command> [args]` | **GUI autorun**: the GUI opens with `<tool-id>` active and the tool performs `<command>` inside the GUI, exactly as if the user had pressed the corresponding button |

The bare `4cc-studio` start reopens the tool the GUI last closed on (the `last_tool` common
setting), not the first registered one.

Besides the tools' subcommands, the shell owns `4cc-studio help`, `4cc-studio update` and `4cc-studio
--version` (see `gui.md` "Help window", `distribution.md` "Distribution and updates", `distribution.md` "Changelog and version display").

The `--gui` form exists for the release bundle's **`quick_compile.bat`** — the successor to Red's
`0_all_in_one.bat`, the one script almost every user runs: copy an export into `exports/`,
double-click, done. The launcher is two lines (`start "" "%~dp04cc-studio.exe" --gui team-compiler
compile`) so its console window closes immediately and only the GUI remains; the Linux bundle ships
the equivalent `quick_compile.sh`. Red's other staged bats (`1_`–`3_`) have no successor: the
unified pipeline has no stages to run separately. Users who want the plain GUI double-click the exe.
Until the Team compiler's view exists, the 0.0.1 bundle's `quick_compile.bat` runs the plain CLI
compile in its console instead, beside an interim `quick_upgrade.bat` (`development_plan.md`
"The /vg/ League 27 (VGL)").

The shell reuses the tool's own CLI parser for the arguments (one grammar for both
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
`start /wait`, and PowerShell pipelines are unaffected). The caveat is accepted rather than
fixed with a console-subsystem shim exe beside the main one: a second exe in the bundle would
confuse the members who only double-click, and the command line is second-class to the window.
So no command reads the keyboard; a command that would need an answer takes it as a flag
(`--yes`, `--edit-mode-closed`). Linux has no subsystem split.

Debug builds are GUI-subsystem too, not only releases: the gates' CLI tests then run the
binary's piped commands with the subsystem the bundle ships, so a redirected run losing its
output fails a test. The binary's unit-test harness, built from the same `main.rs`, keeps the
console subsystem (`cfg_attr(not(test), …)`): a GUI-subsystem test binary's output never
reaches the test runner, so its failing assertions would print nothing. Running the window with no arguments attaches to no console, so what the
window mode would print (a settings file that does not load, a window that cannot open) shows
in a message dialog instead, with the same text and exit code: printed, nobody would see it.

### Tool plugin interface

What exists of this is in `spec/core/architecture.md` "Tool plugin interface".

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
        Box::new(team_creator::Tool::new()),
        Box::new(ruleset_editor::Tool::new()),
        Box::new(db_generator::Tool::new()),
    ]
}
```

That is the full registry; the three tools registered today are the first, second and fourth
(`spec/core/architecture.md` "Tool plugin interface").

The sidebar lists the planned tools a build does not register (`gui.md` "Sidebar", item 3), so
`studio` also keeps the full registry as data: one `PlannedTool { id, label, icon }` per tool
above, in that order, handed to the shell with the registered tools; the shell lists, after the
registered ones, every entry whose `id` no registered tool has. A tool's icon on the sidebar's
rail is a trait method, `fn icon(&self) -> &'static str`, one glyph of the shell's icon font
(`README.md` "GUI framework choice: egui"), and the
planned entry repeats it. Not a second `Vec<Box<dyn StudioTool>>` of disabled tools, because a
planned tool has no crate to construct; not the labels alone, because the rail draws icons.

These commands extend the spec's CLI examples:

```
4cc-studio save-editor export-toml ./EDIT00000000 --team 701 -o ./aaa.toml
4cc-studio stadium-compiler compile ./stadium_exports
4cc-studio export-upgrader upgrade ./old_export.zip --savefile ./EDIT00000000
4cc-studio music-export-editor check ./team.4ccm
4cc-studio match-tracker watch
4cc-studio kit-config-editor convert ./config.bin ./config.toml
4cc-studio refs-arranger chances ./exports/refs_export
4cc-studio balls-compiler compile
```

### Event system

What exists of this is in `spec/core/architecture.md` "Event system".

The GUI rejects events whose `run_id` or `export_revision` is older than the current grid state, so
results from a stale filesystem snapshot cannot overwrite newer checks.

**The console's progress line and run summary.** When stderr is a terminal
(`std::io::IsTerminal`), the console printer draws one progress line on stderr
(`[ 37/120 exports | 1,284/4,010 parts | 0:42 ]`, the stage once `Progress` carries it), redrawn
at most ten times a second and cleared while a finding line goes to stdout, then drawn again; on
`Complete` it leaves one summary line in its place (files written, exports skipped, folders
skipped, folders with errors, elapsed time; each a count of distinct exports or folders, never
of findings, since one folder often has several). Exports skipped are counted apart from
folders: an export refused whole (two exports of one team, a disabled one) names no folder,
and counting it as none would end a run that left a team out on "0 folders skipped". When stderr is not a terminal (piped, redirected, a script or CI) neither is
drawn, and stdout carries exactly the finding lines and result text it carries without them, so
the CLI tests and any script reading stdout see no change; no flag turns it on or off. The printer
is tool-neutral, so every tool that emits `Progress` gets the line; the Team compiler is the first.
The exports counter is the printer's own count of the run's `ExportStarted` (the total: every
export starts in the validation pass, before any is compiled) and `ExportProcessed` events, so it
needs no event of its own and draws from the first export. The parts counter is `Progress`, which
the Team compiler emits as each task of its build manifest commits (`spec/team_compiler/pipeline.md`
"3. Per-model-folder parallel steps (rayon)", its first paragraph: a model folder, a kit, the logo, a Common directory, the collars, the
portraits, the referee marker), `total` being the manifest's task count; "parts", not "folders",
because a task is not always a folder, and a counter that named folders would not add up to the
export's folders. A run whose stream ends with no `Complete` (a `check`, an upgrade, a compile
stopped before it compiled) leaves no line: the printer cannot know a `Complete` is coming
until the stream ends, so it draws from the first export and clears the line when the stream
ends without one, rather than leaving a count that reads as unfinished.
We draw it with `indicatif` (`default-features = false`) rather than a hand-written `\r` line
because it already handles the terminal width, the Windows console and the draw-rate cap; it adds
four small crates and about 145 KiB to a release binary (measured 2026-10-16, 0.18.6). The
progress grid has no terminal version: its value is the GUI's per-folder colours and click-through
to the findings, a cup's grid is wider than a terminal, and a terminal grid would need a TUI
library; the finding lines already name each failing folder, and the summary gives the counts.

**Every finding line says what was lost.** `EventLines`' finding line
(`<export>: <Severity> <code> at <path> (<context>)`) ends with a fixed phrase per effective
disposition: `folder left out`, `file left out`, `slot skipped`, `export skipped`, `run stopped`,
and `kept anyway` for a finding passed through; a finding whose disposition drops nothing
(an Info, a kept Warning) gets none. The same phrase ends the finding in a grid cell's tooltip.
Severity alone does not say it: two Warnings drop a whole folder (`player_unlisted`,
`shared_folder_orphaned`), so a manager reading only the Errors, or skimming the yellow lines,
never learns the folder is gone. One disposition-to-phrase mapping in `studio_core`, never a
phrase in each catalog text, so a text cannot contradict the disposition the run applied; the
CLI tests that pin whole lines change with it.

These notes extend the spec's `PipelineEvent` enum, the first at `Progress`, the second at `Complete`:

```rust
    // The run strip needs stage, elapsed time, files written, and memory in
    // use — more than `Progress` carries. Extend `Progress` with these fields
    // or add separate `StageChanged`/`MetricsUpdated` events before implementing
    // the run strip view (see "Toolbar" in the Team compiler plan).

    // The Team compiler needs separate compile and deployment outcomes (rollback,
    // "artifacts built but installation failed"). Replace the boolean `success`
    // with structured CompileOutcome/DeploymentOutcome/CancellationOutcome before
    // implementing the run-result semantics described in the Team compiler plan.
    // At that point the placeholder folder-only counters below become task/scoped-
    // outcome counts, so typed non-folder tasks are represented in the summary.
```

### Diagnostic logging

What exists of this is in `spec/core/architecture.md` "Diagnostic logging".

- In GUI mode `studio` installs a small `log::Log` impl writing `studio.log` in the data
  directory — the file a user attaches to a bug report. The GUI `gui.md` "Log panel" shows
  findings, not this file; an in-app diagnostic view is not planned for the first release.
- `studio.log` is a bug report's attachment, so the GUI sink also: installs a panic hook that
  writes the panic message, its location and a backtrace to the log before the process ends, and
  tells the user (a dialog when the GUI is up) to post the file, since a GUI binary has no
  console to show the panic in; renames the previous run's log to `studio.old.log` at start,
  replacing the older one, so a crash followed by a relaunch keeps the crash's log; writes the
  Studio version as the log's first line; and writes the user's home directory as `~` in every
  line, because findings carry absolute paths and the file is posted on the cup's server.
- The backtrace is written as each frame's module base and offset (`0x7ff6a1230000+0x59c4`,
  through the `backtrace` crate), not as names: a release exe names its frames only with its
  `.pdb` beside it, and the bundle does not ship the `.pdb` (`distribution.md`, the release
  recipe). `scripts/symbolize_log.py <studio.log> <symbols folder>` turns a posted log's frames
  of the exe's module into function, file and line through `llvm-symbolizer` (MSVC's toolset
  ships it), against the release's kept exe and `.pdb`; frames of other modules (the system's
  DLLs) stay as offsets. Off Windows the `backtrace` crate reports no module base (its
  `libunwind` backend's `module_base_address` is `None`), so a frame is an absolute address,
  which a position-independent exe loaded at a random base leaves unnamed; before the Linux
  bundle (1.0.0) the hook also writes one known function's address, from which the script
  finds the exe's load base. We do not ship the `.pdb`, deflated beside or inside the exe, because
  either costs every member's download about 3.8 MB for a file only the maintainer reads,
  and the inside variant also has the crash write 17 MB and point the backtrace at it.
- A session that ends without a word is reported at the next start. The window's normal
  close writes a closing line as the log's last, and so does a crash after its dialog; at
  start, a previous log (`studio.old.log` once renamed) that does not end with one of the two
  shows a dialog before the window opens: "The last session did not close normally. If it
  crashed, please post this file where you report problems: <path>", naming the home folder's
  `~`. This catches what the panic hook cannot see (a stack overflow, a failed allocation, a
  crash in native code) without a monitor process. The wording is neutral because the dialog also follows a session Windows
  ended: winit 0.30 handles neither `WM_QUERYENDSESSION` nor `WM_ENDSESSION`, so a shutdown
  with the window open ends the process with no closing line, as a kill from Task Manager
  or a power cut does. We do not tell the two apart through the boot time: a crash followed
  by a restart before the next start would then go unreported.
- The level is `info`, with no `-v` or `RUST_LOG` (a double-click passes neither). Each line is
  flushed as it is written, so a crash keeps everything before it.
- A first run writes no log: the file lives in the data location, which does not exist until
  the first-run dialog's choice, and a log written anywhere else would leave a folder behind
  that a portable choice promised not to. The next run logs.
- The hook logs every panic, but the dialog shows only for the panic that ends the window, caught
  around the GUI run in `main`; the process then exits with 101, the code an unguarded panic
  exits with. A panic on another thread is logged without a dialog: the hook cannot tell a
  panic that will be recovered (the GPU encoder's, which `dds_convert` turns into a fallback
  to the CPU encoder) from one that will not, and a dialog over a recovered panic would ask a
  member to report a run that went fine. The dialog repeats the panic's message, so a first
  run's crash, which has no file, still leaves something to post.
