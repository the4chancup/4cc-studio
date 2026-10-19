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
│ ▌ TEAM   │   │  /bb/ [???]  [01][02][03][04][05]  [GK][K1]│ │
│   SAVE   │   │  /cc/ [809]  [01][□][□][□][□]      [GK][K1]    │ │
│   UPGR   │   └────────────────────────────────────────┘ │
│  (STAD)  │   [run strip]              [Compile ▾]    │
│  (⋯)     │   ┌────────────────────────────────────────┐ │
│          │   │ Log panel (scrolling, collapsible)     │ │
│  ──────  │   │ - Warning - Nested folders ...         │ │
│ [?][gear]│   │ - /cc/ processed                       │ │
│          │   └────────────────────────────────────────┘ │
├──────────┴──────────────────────────────────────────────┤
│ ⚠ data folder read-only          Compiled 48 · [Open output] │
└─────────────────────────────────────────────────────────┘
```

The sidebar lists the registered tools in registry order (today the Team compiler, the Save
editor, the Export upgrader), then the planned ones in parentheses here, grayed in the window
("Sidebar").

The shell draws only what is its own. Panels claim space in call order, so the status bar is
added first to span the full width, the sidebar next, and the central panel, whatever is left,
is handed to the active tool's `view()`:

```rust
impl eframe::App for StudioApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Status bar: full width, under the sidebar too (added first)
        egui::Panel::bottom("status_bar").exact_size(24.0)
            .show(ui, |ui| self.render_status_bar(ui));

        // Left sidebar (collapsible width)
        egui::Panel::left("sidebar").exact_size(self.sidebar_width())
            .show(ui, |ui| self.render_sidebar(ui));

        // Central panel: the active tool's view, which lays itself out inside it
        egui::CentralPanel::default().show(ui, |ui| {
            self.tools[self.active].view(ui, &self.ctx);
        });
    }
}
```

A pipeline tool subdivides the `Ui` it is given with panels of its own, in the same claim
order: the log panel at the bottom, the toolbar above it, the grid in what remains. Since 0.34 a
panel's `show` takes any `Ui`, the whole-app one or a tool's (the older `show_inside` is
deprecated, and deprecations are errors here, `spec/core/gui.md` "egui version discipline"):

```rust
// team_compiler::view, inside `view()`
egui::Panel::bottom("log").resizable(true).default_size(220.0)
    .show(ui, |ui| log_panel::show(ui, &self.log));
egui::Panel::bottom("toolbar").exact_size(32.0)
    .show(ui, |ui| toolbar::show(ui, &self.run));
egui::CentralPanel::default().show(ui, |ui| grid::show(ui, &self.rows));
```

The shell never draws a tool's log or toolbar: an envelope names no tool (`spec/core/architecture.md`
"Event system"), so a shell-owned log would have to guess whose run a line belongs to, and the
Save editor has no run to show. The widgets are `studio_core`'s (the progress grid, the log panel
and the run strip under `widgets/`, `architecture.md` "Crate structure"); where they are placed
is each tool's.

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
   accent-colored left border bar plus a filled background. The suite's planned tools not in
   this build (the registry's, `architecture.md` "Tool plugin interface") are listed after the
   registered ones, grayed and not clickable, each tooltip saying the tool is not in this build:
   the list communicates the suite roadmap.
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
  see `distribution.md` "Changelog and version display"), which come first in the chapter
  tree; the tools' chapters follow in sidebar order. Red's `readme_advanced.md` maps onto
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
  `PipelineEvent`); its chevron collapses it to its header row and back, and dragging its
  divider resizes it, the height kept across frames.
- **Capped display**: only the last few hundred lines are rendered (egui text rendering is not
  optimized for thousands of lines). Full logs are written to disk (`issues.log`, `suggestions.log`)
  as before.
- **Auto-scroll with interruption**: sticks to the bottom until the user scrolls up, which pauses
  auto-scroll and shows a "↓ N new" jump-back indicator.
- **Filter toggles**: All; Warnings (the warning, error and fatal lines); Errors (the error and
  fatal lines). A filter hides lines, never drops them: All shows every line again.
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
| A tool reports a `ToolActivity` | `Team compiler · 12/48 · 01:12 - 4cc Studio` |
| Idle, window unfocused, a `Notice` arrived since focus was lost | `Compiled 48 teams - 4cc Studio` |

Rules:

- **A plain hyphen before the name.** The title is a UI string, so it carries no em dash
  (`CONTRIBUTING.md` "Code style"); ` - ` is also what most desktop apps put before their name.

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

Each tool owns the receiving end of its own runs, as the spec's shell slice built it: a run gets
its sink through `ctx.with_events(sender)`, the tool drains the receiver in `tick`, and `view()`
renders the resulting state and reports the user's intent (a Compile click starts the next
run). The block extends the spec's `logic` pass, which ticks every tool:

```rust
// Pipeline workers (on the run's thread or the rayon pool) send stable-ID events through the
// stale-result envelope, to the sender the run was started with:
ctx.emit(PipelineEventEnvelope {
    run_id,
    export_id: Some(export_id),
    export_revision: Some(export_revision),
    event: PipelineEvent::FolderStatus { export_id, path: folder_path.clone(), status },
});
// Messages are sent as separate enveloped events; the tool associates them by scope.
for msg in messages {
    ctx.emit(PipelineEventEnvelope {
        run_id,
        export_id: msg.scope.export_id(),
        export_revision: revision_for(&msg.scope),
        event: PipelineEvent::Message(msg),
    });
}

// The tool drains its receivers once per frame, in `tick`, which the shell calls for every
// registered tool before `ui` (and also while the window is hidden, when a repaint was
// requested), so a compile behind the Save editor's view keeps its grid current:
impl StudioTool for team_compiler::Tool {
    fn tick(&mut self, _ctx: &ToolContext) {
        // One receiver per live source: the compile in flight, the live checks.
        while let Ok(envelope) = self.run.events.try_recv() {
            // Reject what an older run or an older export revision computed.
            if self.rows.is_current(&envelope) {
                self.rows.apply(envelope.event);
            }
        }
    }

    fn view(&mut self, ui: &mut egui::Ui, ctx: &ToolContext) {
        // Render: grid, toolbar and log read the tool's state (layout as in "Shell layout").
    }
}
```

egui repaints only on input, so a run's events wake the GUI themselves: the shell installs a
wake callback in the `ToolContext` it hands out (`Arc<dyn Fn(Duration) + Send + Sync>`
calling `egui::Context::request_repaint_after`: GUI-neutral like the match feed's `wake` in
`architecture.md` "Tool plugin interface", but taking a delay; the feed's own stays `Fn()`,
since a publish always wants the next frame), and `ToolContext::emit` calls it with no delay
after each send, on every sink `with_events` makes. The next frame's `tick` then drains the
event whichever view is drawn, or none while the window is hidden, so a compile behind the Save
editor's view still ends its activity and updates the window title (ST-TTL-03). A tool's own
deadlines (the music player's chant and VA timers) go through the same callback, as
`ToolContext::request_repaint_after(delay)` called from `tick`, which is what keeps them firing
while another tool is in view. We wake from the context, not from the tool's view
(`request_repaint_after` while a run is in flight, as the Phase 3 slice does), because a view
not drawn requests nothing: the run's last events, or a hidden tool's next deadline, would wait
for unrelated input. In the CLI the callback does nothing. The match feed wakes the GUI the same
way, on publish.

The grid, log panel, and run strip are all views over the same tool state, updated from the
same receivers. The stadium compiler view reuses the same grid rendering code with rows =
stadiums and cells = components.

### Other UI components

| Component | Implementation |
|-----------|---------------|
| Common and per-tool settings forms | egui widgets (`TextEdit`, `CheckBox`, `ComboBox`), saved to TOML |
| Export tree browser | `egui::CollapsingHeader` tree, shows object model |
| File/folder pickers | `rfd` for native dialogs and supported web file picking; web directory access needs a separate File System Access API adapter (`rfd` has no WASM folder picker) |
| Dark/light theme | `studio_core`'s `theme.rs`: the curated dark `Visuals` preset, the default; light, second-class, egui's light `Visuals` with the same accent, rounding and status colors; chosen by the common `theme` setting and applied live (`Context::set_visuals`, no restart); icon glyphs from `egui-phosphor` |

### Prototypes before implementation

Every tool's GUI, and every new shell element, is drafted first as an interactive web page and
refined with the maintainer (and shown to the members who asked to help shape it) before any
egui code is written: a page is quicker to change than a view, and a member can open it without
building anything. The draft then becomes the view's specification, so it uses only the CSS
and HTML elements the UI crates we plan to use can represent: `egui`/`eframe` and the UI crates
of the dependency table in `README.md` "External Dependencies" (today `egui-phosphor`,
`egui_commonmark`, `egui_animation`, `egui_extras`, `rfd`). Anything else would promise a screen the
implementation cannot deliver. An element only an unlisted crate can draw needs that crate added
to the table first (a new dependency, asked for first):

- **Representable, for example:** egui's widgets (label, button, checkbox, radio button,
  selectable label, combo box, slider, drag value, single- and multi-line text edit, hyperlink,
  image, progress bar, spinner, separator, colour button), its containers (collapsing header,
  grid, scroll area, horizontal and vertical layouts, top, bottom, side and central panels,
  window, modal), tooltips, context and bar menus, drag and drop between widgets and of files
  dropped on the window from outside, native file dialogs (`rfd`), Markdown text
  (`egui_commonmark`), `egui-phosphor` icons, colour or position transitions with easing
  (egui's `animate_*`, `egui_animation`), cards and rows that change on hover (a `Frame` over
  a container that senses hover, `UiBuilder::sense`), mixed inline formatting in one line
  (colour, size, background highlight, underline, strikethrough: egui's `LayoutJob`), and
  tables with sortable or resizable columns and a sticky header (`egui_extras`' `TableBuilder`;
  the sorting is ours).
- **Not representable, for example:** free CSS layout (flexbox or grid placement egui's layouts
  and `egui_extras`' `StripBuilder` cannot express), CSS effects beyond fill, stroke, rounding
  and shadow (gradients, blur, filters), and per-character text effects.
- **Look:** one shared stylesheet in Studio's dark theme, so every draft looks like Studio and
  the drafts look like one another. Drafts are not reviewed in the light theme, which is
  second-class.

A draft is one self-contained HTML file (its script and styles inline), so it can be sent as a
single attachment; its behavior is faked with sample data, never wired to the real libs. The
drafts live in `docs/prototypes/`, one file per tool (`team_compiler.html`, `save_editor.html`,
`export_upgrader.html`), and one for the shell (`shell.html`: sidebar, status bar, settings
menu, help window, the first-run dialog). Each is the first step of its view's build: Phase 8's
list in `development_plan.md` names it ahead of every view, and each tool plan says where its
draft sits in that tool's build order. A draft is drawn whole, every screen the tool's plan
describes with sample data, before the first slice of the view is coded; a slice that changes
what the draft shows changes the draft first, so the draft stays the specification the view
is checked against.

### Cancellation

What exists of this is in `spec/core/gui.md` "Cancellation".

During a run the Compile button becomes Cancel. A texture task checks the flag between its
textures, since it converts them one after another and a folder of large BC7 encodes can be one
of a run's longest tasks. It aborts cleanly: the run's staging is discarded and the previous
outputs stay as they were. In the grid, the cells not yet processed return to their check-phase
state (the run wrote nothing of theirs, so nothing is "done"), and the log's last line says the
run was cancelled, so a later reader of the log does not take a short run for a finished one.

**Closing the window** asks every tool, not only the one in view, and so does every close the
shell performs itself (the Team compiler's quick-compile close on success included): a tool
holding unsaved work (the Save editor's edits) reports it through a `StudioTool` hook beside
`version_change_blocker`, and the shell then cancels the close, makes the first such tool the
active view and lets its own prompt decide. Save or discard sends a close request back to the
shell (a `ShellRequest`, as a tool switch is), which asks every tool again, so a second tool
with unsaved work gets its own prompt in turn; cancel keeps the window and ends the close. A
close the shell decided alone would drop the edits of a tool whose view was not drawn. A tool
with a run in flight (a compile, an upgrade) reports it through the same hook, and its prompt
offers to cancel the run and close, the close request going back once the cancellation above
has ended, or to keep running: an exit cannot discard the run's staging as a cancel does, and
a run killed while deploying can leave a torn output (ST-RUN-05, ST-RUN-06).

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

### Acceptance

Format and rules: `CONTRIBUTING.md` "Testing". `ST` is the shell's code (`studio_core`'s shell
and widgets, and the `studio` binary's launch modes); IDs are never reused; later phases append.
The shell is not a tool, so its scenarios live here, beside the sections they prove.

**Phase 8 scope.** Phase 8 completes the shell the Phase 3 slice started: the sidebar, the
settings menu, the status bar, the window title, the help window and `4cc-studio help`, the
log panel widget, the first-run data-location dialog, the `last_tool` reopening and the
`--gui` autorun. What a scenario observes is what a frame draws (through the headless harness
of `studio_core/src/shell/mod.rs`: the frame's AccessKit nodes, a click as press and release
frames, the viewport and output commands a frame emits, a title or a clipboard copy; a color,
which no AccessKit node carries, is read
back from the shell state the frame paints from), the settings file, and what the binary
prints; `manual` marks only what needs a display, a native dialog or a running game. The tools in a
scenario are stub tools registered with the test's shell unless a real one is named; "the
settings file" is one under the test's own folder (`AppPaths` pointed there), never the
developer's. The Team compiler's own toolbar, grid and cancellation are `TC-GUI` scenarios
(`plans/team_compiler/README.md` "Acceptance"), the Save editor's view `SE-GUI` and the Export
upgrader's `EU-GUI`.

**Sidebar**

```
ST-SIDE-01  GIVEN three registered tools and a settings file whose last_tool names the second
            WHEN the GUI opens
            THEN the sidebar lists the three with their full labels in registration order, the
                 second is active and its view is the one drawn
ST-SIDE-02  GIVEN the sidebar expanded
            WHEN the collapse button is clicked
            THEN the sidebar is an icon rail whose entries carry their labels as tooltips, a
                 click on an entry still switches to that tool, and a second click on the
                 collapse button restores the labels
ST-SIDE-03  GIVEN the selector at PES 21 and pes_folder_path resolving to a folder holding
                 PES2019.exe
            WHEN the sidebar is drawn
            THEN the selector is red and its tooltip names the exe found; GIVEN the folder
                 holding no PES exe, or missing, THEN the selector is yellow and the tooltip
                 says what was found; GIVEN it holding PES2021.exe THEN neither
ST-SIDE-04  GIVEN a tool reporting a version_change_blocker
            WHEN the sidebar is drawn
            THEN the selector is disabled with the tool's reason as its tooltip, and the setting keeps its
                 value; GIVEN no tool blocking WHEN another version is picked THEN the settings
                 file holds it and every tool reads it at its next frame
ST-SIDE-05  manual  GIVEN the selector at PES 21 and PES2019.exe running from a folder other than
                         the resolved pes_folder_path
                    WHEN the poll next runs
                    THEN the selector switches to PES 19 once, the status bar shows the notice
                         naming the switch with a "Fix PES path" action, the action rewrites
                         pes_folder_path from the running process (the ** placeholder kept where
                         the path segment matches the version), and a later manual change back
                         to PES 21 is not switched again while the program lives
ST-SIDE-06  GIVEN the GUI open
            WHEN the logo is clicked
            THEN the About popup shows the version block (version, build kind, target triple,
                 data location, selected PES version), links to the release page, the repository
                 and the help, and its Copy button puts exactly that block on the clipboard
ST-SIDE-07  GIVEN a build registering two of the suite's tools
            WHEN the sidebar is drawn
            THEN the suite's other planned tools are listed after them, grayed and not
                 clickable, each tooltip saying the tool is not in this build
```

**Settings menu**

```
ST-SET-01   GIVEN two registered tools with settings sections and the second active
            WHEN the settings gear is clicked
            THEN the menu shows the common section and one collapsible section per tool, the
                 second tool's expanded and the first's collapsed
ST-SET-02   GIVEN the menu open
            WHEN a common setting (`thread_count`) is changed
            THEN the settings file holds the new value under [common], every tool table as it
                 was, and ctx.common() returns the new value on the next frame
ST-SET-03   GIVEN a data directory that cannot be written
            WHEN a setting is changed
            THEN the change applies for the session, the status bar's left slot shows the
                 data_dir_read_only condition, and no dialog or prompt opens
```

**Status bar and window title**

```
ST-BAR-01   GIVEN no condition, no background activity and no notice
            WHEN the status bar is drawn
            THEN it shows the version number in small text and nothing else, at its fixed height
ST-BAR-02   GIVEN the first tool active and the second reporting a ToolActivity
            WHEN the status bar is drawn
            THEN its middle slot shows the second tool's activity; GIVEN the second tool made
                 active THEN the slot is empty (its own run strip shows it)
ST-BAR-03   GIVEN a tool publishing a Notice with an action
            WHEN the status bar is drawn
            THEN its right slot shows the text and the one action until the action is clicked
                 or the notice dismissed; GIVEN a notice without an action THEN it leaves the
                 slot by itself after a few seconds, and a later notice replaces an earlier one
ST-TTL-01   GIVEN a tool reporting a ToolActivity (label "Team compiler", 12 of 48 done, 72 s
                 elapsed)
            WHEN a frame is drawn
            THEN the viewport title command reads "Team compiler · 12/48 · 01:12 - 4cc Studio",
                 sent once and not again while the string is unchanged; GIVEN no activity THEN
                 "4cc Studio"
ST-TTL-02   GIVEN the window unfocused and no tool reporting an activity
            WHEN a notice "Compiled 48 teams" arrives
            THEN the title reads "Compiled 48 teams - 4cc Studio" until the window gains focus,
                 then "4cc Studio"; GIVEN the window focused when it arrives THEN the title stays
                 "4cc Studio"
ST-TTL-03   GIVEN the window focused, two stub tools, the second's view active, and the first's
                 run on a thread of its own
            WHEN the run emits its last event through its ToolContext and no input follows
            THEN the emit requests a repaint, and the next frame's tick drains the event into
                 the first tool's state (its activity ended, the title "4cc Studio") without its
                 view being drawn
```

**Help**

```
ST-HELP-01  GIVEN the Team compiler active
            WHEN F1 is pressed, or the sidebar's ? clicked
            THEN the help window opens on the Team compiler's chapter, the chapter tree lists
                 one chapter per registered tool in sidebar order after the core chapters, and
                 the chapter's last topic, Messages, lists every code of the tool's catalog with
                 its severity
ST-HELP-02  GIVEN the help window open
            WHEN a message code is typed in the search box
            THEN the results list its catalog row as "Chapter › Topic › line", and clicking the
                 result shows that topic with the line highlighted; WHEN a word of a topic's
                 body is typed, in another letter case, THEN that topic's line is listed the same
                 way
ST-HELP-03  GIVEN a tool's log panel showing a finding line
            WHEN its message code is clicked
            THEN the help window opens at that code's catalog row; GIVEN a topic holding an
                 in-doc link `[text](tool-id#topic-slug)` to another tool's topic WHEN the link
                 is clicked THEN that topic is shown
ST-HELP-04  GIVEN the built binary
            WHEN `4cc-studio help team-compiler` runs
            THEN stdout is the chapter's Markdown; WHEN `4cc-studio help --export <dir>` runs
                 THEN <dir> holds one .md file per chapter, each ending with its Messages topic
```

**Log panel widget**

```
ST-LOG-01   GIVEN a log holding more lines than the display cap
            WHEN the panel is drawn
            THEN only the last lines up to the cap are drawn, the newest at the bottom
ST-LOG-02   GIVEN the panel scrolled up by the user
            WHEN new lines arrive
            THEN the view does not move and a "↓ N new" indicator counts them; clicking it
                 returns to the bottom and auto-scroll resumes
ST-LOG-03   GIVEN a log holding info, warning, error and fatal lines
            WHEN the Warnings filter is chosen
            THEN the warning, error and fatal lines are drawn and the info lines are not; Errors,
                 the error and fatal lines alone; All, every line again
ST-LOG-04   GIVEN the log panel open at its default height
            WHEN its chevron is clicked
            THEN the panel collapses to its header row and the view above it takes the space,
                 and a second click restores it at its height; WHEN its divider is dragged THEN the panel takes the dragged height, kept across
                 frames
```

**Launch and close**

```
ST-RUN-01   GIVEN neither settings file exists
            WHEN the GUI starts
            THEN a dialog asks for the data location (portable data/ or the user config
                 directory) before any tool view, and the chosen location's settings.toml exists
                 afterwards; GIVEN one exists THEN no dialog and that location is used
ST-RUN-02   GIVEN the GUI closed with the second tool active
            WHEN `4cc-studio` starts again
            THEN the second tool is active
ST-RUN-03   GIVEN a tool whose gui_run does not implement the command
            WHEN `4cc-studio --gui <tool-id> <command>` runs
            THEN it exits with a message naming the tool, and no window opens;
                 GIVEN a tool implementing it THEN the GUI opens with that tool active and the
                 command runs as if its button had been pressed
ST-RUN-04   manual  GIVEN the Windows bundle
                    WHEN the exe is double-clicked
                    THEN no console window opens beside the GUI; WHEN `4cc-studio team-compiler
                         check` runs from a terminal THEN its output reaches that terminal
ST-RUN-05   GIVEN two stub tools each reporting unsaved work, a third tool's view active
            WHEN the window's close is requested
            THEN the window stays and the first stub's view is active with its prompt; WHEN
                 it discards THEN the second stub's view is active with its prompt; WHEN that
                 one discards THEN the window closes; GIVEN either prompt cancelled instead
                 THEN the window stays and no further prompt shows
ST-RUN-06   GIVEN a stub tool reporting a run in flight, another tool's view active
            WHEN the window's close is requested
            THEN the window stays and the stub's view is active with its prompt; WHEN it
                 chooses cancel-and-close THEN the run's cancel flag is set and the window
                 closes once the run has ended, not before; WHEN it chooses to keep running
                 THEN the window stays and the run goes on
ST-CLI-01   GIVEN a data directory that cannot be written
            WHEN a CLI command runs
            THEN stderr starts with the data_dir_read_only condition as one line, and a notice a
                 tool publishes during the command prints as one line when it occurs
ST-CLI-02   manual  GIVEN a compile run from a terminal
                    WHEN it runs
                    THEN one progress line is drawn on stderr and redrawn in place, cleared while
                         a finding line goes to stdout, and one summary line remains when the run
                         ends; GIVEN the same command with stderr piped THEN neither is drawn and
                         stdout holds exactly the lines it holds in a terminal
```
