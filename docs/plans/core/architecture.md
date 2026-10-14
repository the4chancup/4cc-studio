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
│       ├── aatf/                     # (Phase 5, in progress) AATF rules: the ruleset schema, the
│       │                             #   Rhai engine and the host views exist; the interpreter's
│       │                             #   checks and `apply_tier` join it (Save editor, Team creator,
│       │                             #   Ruleset editor)
│       ├── music_export/             # (Phase 10) .4ccm music export format: parse/write + condition model
│       ├── audio_engine/             # (Phase 10) Audio playback (kira/symphonia) + loudness analysis (ebur128)
│       └── match_feed/               # (Phase 11) Live match event feed (match tracker → music player)
```

**`studio_core`** also contains:
- The app shell: sidebar (logo, PES version selector, tool list, collapse, help and settings
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
├── theme.rs          # the shared Visuals preset, fonts, egui-phosphor registration
├── pes_process.rs    # running-PES poll (desktop): version + exe path detection
└── updater.rs        # release check, download/verify, binary swap, rollback (desktop only)
```

**The running-PES probe.** `pes_process::running_pes() -> Vec<RunningPes>` lists the PES games
running now, each a `RunningPes { version: PesVersion, exe: Option<PathBuf> }`, matched by
executable name: `PES2015.exe` to `PES2021.exe`, in any letter case (the names the games ship
with; a renamed exe is not seen). On Windows it walks a Toolhelp process snapshot and asks each
match for its image path (the `unsafe` the code style allows here), `None` when the process
refuses (another user's or a protected process; the limited query right is granted on an
elevated game too); on Linux it reads each
`/proc/<pid>/cmdline`, whose first argument under Wine or Proton is the exe's Windows path, and
matches its last segment after a `\` or `/`; elsewhere it returns none. A process it cannot read
is skipped. The CLI calls it once where a step must not run while the game of the save's version
does (the compiler's savefile step, the Save editor's writing commands; a game of another
version never writes that save); the GUI's version-selector poll calls it on a
timer. We match process names, not a lock file or a window title, because the game takes no lock
on its save and its window title changes with its language.

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
`start /wait`, and PowerShell pipelines are unaffected). If that proves to annoy CLI users, the
escape hatch is a tiny console-subsystem `4cc-studio-cli.exe` shim that spawns the main exe and waits;
deferred until the annoyance is real. Linux has no subsystem split.

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
    ]
}
```

That is the full registry.

These commands extend the spec's CLI examples:

```
4cc-studio save-editor export-toml ./EDIT00000000 --team 701 -o ./aaa.toml
4cc-studio stadium-compiler compile ./stadium_exports
4cc-studio export-upgrader ./old_export.zip
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

**A command's result text.** `PipelineEvent::Output(String)` carries text a CLI command prints
as its result, a JSON report or a document: `EventLines` gives it back as it is, so the console
printer writes it to stdout with no prefix or folding, and a tool's log shows it as one entry. A
command whose result is such text reports nothing else on success (its findings go in the text),
so stdout parses (`save-editor diff --json`). We do this, not return the text from `cli_run`,
because the printer thread owns stdout while a command runs: a second writer would interleave with
its lines.

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
    // At that point the placeholder folder-only counters above become task/scoped-
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
