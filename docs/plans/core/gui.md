# 4cc Studio — Core plan: GUI design

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## GUI Design

### Shell layout

What exists of this is in `spec/core/gui.md` "Shell layout".

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

Panels claim space in call order, so the status bar is added first to span the full width:

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

What exists of this is in `spec/core/gui.md` "Settings menu".

The settings menu is assembled from injected sections:

- **Common settings** (defined by `studio_core`): PES version, PES folder path, exports folder path,
  thread count, memory cap, theme.
- **One collapsible section per registered tool**, provided by that tool's `settings_view()` (each
  tool defines its own settings struct and UI).
- Opening the settings menu shows the **currently selected tool's section expanded** and all other
  sections collapsed.
- Saving is **best-effort**: if the write fails (typically a portable install unpacked somewhere
  unwritable), the in-memory settings keep working for the session and the shell raises the
  `data_dir_read_only` condition in the status bar — one condition covering settings and the
  teams list, since both live there. No elevation, no retry prompt.

### Help window

What exists of this is in `spec/core/gui.md` "Help window".

The help window is the user documentation — it replaces both of Red's readmes (the 114-line
README and the 913-line `readme_advanced.md`), and the bundled readme shrinks to a pointer at it.
Like the settings menu it is **assembled from the tools**, not written centrally:

- **Chapters.** `studio_core` contributes the core chapters (setting up, first run and data
  location, updates, and "What's new" — the repository's `CHANGELOG.md` embedded at build time,
  see `distribution.md` "Changelog and version display"). Red's `readme_advanced.md` maps onto
  this almost one-to-one: its refs, format, texture-handling and special-files headings become
  Team compiler topics, its updater and first-run headings core topics.
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
- **CLI equivalence.** `4cc-studio help [tool-id] [topic]` prints the Markdown to stdout, and
  `4cc-studio help --export <dir>` writes every chapter out as `.md` files — the wiki and the release
  page get the same text with no second source, and the `--export` output is what the release
  process publishes.

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

What exists of this is in `spec/core/gui.md` "Status bar".

One row, 24px, full width under sidebar and tool view, present in every tool. It is the home for
everything that is *about the environment or about another tool* — the things that previously
had no home and were described as "toasts" in three places. It is not a second log: a message
about the current tool's own work (an export, a scope, a run) goes to that tool's Log panel or
run strip, never here.

Each item kind stays in the bar for its lifetime:

| Item | Lifetime |
|---|---|
| `ShellCondition` | While the condition holds; disappears by itself |
| `ToolActivity` | While the tool reports it |
| `Notice` | A few seconds; with an action, until clicked or dismissed. Older notices are not kept — an event worth keeping is also a finding in a log |

`update_available` carries an Install action; each condition shows a tooltip and at most one action.

More rules that keep it worth the row:

- **Never modal, never a question.** Anything needing a decision (updater consent, the teams-list
  merge review) is a dialog. The bar announces; it does not ask.
- **Empty state is static, not blank**: `v1.2.0` in small text — the one always-visible version
  number, the line a user pastes into a bug report (see `distribution.md` "Changelog and version display"), and
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

What exists of this is in `spec/core/gui.md` "Event wiring".

The GUI owns the receiving end and renders state in the egui update loop; the block extends the
spec's `logic` pass, which ticks every tool:

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

What exists of this is in `spec/core/gui.md` "Cancellation".

During a run the Compile button becomes Cancel. A texture task checks the flag between its
textures, since it converts them one after another and a folder of large BC7 encodes can be one
of a run's longest tasks. It aborts cleanly: the run's staging is discarded and the previous
outputs stay as they were.

### Browser deployment: Studio Web

What exists of this is in `spec/core/gui.md` "Browser deployment: Studio Web".

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
- **Transfer size:** a target, not a limit. The first load should stay at
  a few MB as the browser receives it (brotli or gzip compressed), so a member on an ordinary
  home connection waits seconds, not minutes; Phase 18 measures it. Past that, the embedded
  resources are the first thing to fetch lazily (the bullet above).
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
