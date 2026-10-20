# 4cc Studio — Core plan: GUI design

Part of the [Core plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## GUI Design

### Shell layout

In egui, the shell is built with `Panel` (sides) + `CentralPanel`, shown on the whole-app `Ui`
that `eframe::App::ui` provides (API as of egui 0.36; see "egui version discipline" below).

### egui version discipline

egui still ships renames and deprecations most releases (0.31 moved window options into
`ViewportBuilder`; 0.32 rewrote popups/menus and renamed `Rounding` to `CornerRadius`; 0.34
deprecated `SidePanel`/`TopBottomPanel` for `Panel`, replaced `App::update(&Context)` with
`App::ui(&mut Ui)`, and renamed `Context::style` to `global_style`). LLM-assisted coding reproduces
whichever API the model saw most, so it will emit pre-0.34 code by default. Rules:

- `eframe` and `egui` are pinned exact (`=0.36.2`) once in `[workspace.dependencies]` (the
  lock-step rule under `architecture.md` "Workspace guardrails") and upgraded deliberately, one
  release at a time, following each release's migration notes in `CHANGELOG.md` / the GitHub
  release. Exact rather than caret, because a caret already excludes the next minor (the
  releases that rename) and what remains to exclude is a patch release arriving through a bare
  `cargo update`, unreviewed. `egui-wgpu` is not declared: it is `eframe`'s own dependency, and
  the crates built on egui (`egui-wgpu`, `egui-winit`, `egui_extras`) each require the `egui` of
  their own release, so the exact `egui` pin holds them to it without a pin of their own. What
  it does not hold is egui's own dependencies (`epaint`, `emath`, `ecolor`), which egui requires
  by caret; they carry no API this workspace calls directly.
- Treat deprecation warnings as errors in CI (`#![deny(deprecated)]` or `-D deprecated`) so stale
  API from generated code is caught at compile time rather than accumulating.
- The code sketches in these plans follow 0.36; they illustrate structure and must be re-checked
  against the pinned version when implemented, not pasted.

### Settings menu

- Settings persist to a single file: `[common]` holds `CommonSettings`, and each tool's values
  live in a table under its `id()` key (`[team-compiler]`; `common` is therefore not a valid tool
  id), with `default_settings()` merged in for missing values, recursively into nested tables
  (this replaces Red's settings-transfer mechanism for new versions). Missing common keys load as
  their defaults the same way; unknown keys in a tool's table are kept as read. The file's location — `data/` next to the exe or the user config dir — is the
  user's first-run choice (see `distribution.md` "Data location" under Distribution and updates).

### Help window

- **Chapters.** Every registered tool contributes one chapter through `StudioTool::help()`, a
  `HelpSection { title, topics: Vec<HelpTopic> }` whose topics are Markdown bodies embedded with
  `include_str!` from the crate's `help/` folder (numbered files fix the order: `01_overview.md`,
  `02_exports.md`, …). Chapters appear in sidebar order; an unregistered tool has no chapter, so
  the help can never describe a tool the build lacks.
- **Authoring rule.** Topics are written for members, not maintainers: what to put where, what a
  message means and what to do about it, never how the code works (that is `docs/`). A behavior
  change that a member can observe is not done until its topic says so — the help lives in the
  tool's crate precisely so the same diff carries both.

### Status bar

Three slots, each admitting one typed item kind from `studio_core::status`:

| Slot | Item | Admits |
|---|---|---|
| Left | `ShellCondition` | Persistent environment states the **shell** computes itself: `data_dir_read_only`, `update_available`. Not a PES folder that is missing or holds no exe or another version's: the version selector's coloring already owns that, in every tool's sidebar, so no condition repeats it |
| Middle | `ToolActivity` | Tools working in the background while a **different** tool's view is shown — `activity()` polled each frame for every registered tool except the active one, e.g. `Team compiler · 12/48 · 01:12` while the save editor is open. The active tool's own run strip is unchanged, so nothing shows twice |
| Right | `Notice` | The most recent one-off event, published by a tool through `ctx.notify(Notice)` or by the shell: `Switched to PES 2021 · [Fix PES path]`, `Compiled 48 teams · [Open output]`, `Teams list is read-only`. Text plus at most one action |

Rules that keep it worth the row:

- **Typed, never free text.** Tools cannot draw into the bar; they return a `ToolActivity` and
  publish `Notice`s. Conditions are the shell's alone. A `set_status_text(String)` API is exactly
  how a status bar becomes clutter, and it does not exist.

### Event wiring

The tool crates emit `PipelineEventEnvelope` values via channels. eframe's `logic` pass ticks
every tool:

```rust
impl eframe::App for StudioApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Background pass: every tool ticks (event draining, timer deadlines),
        // then only the active tool's view renders
        for tool in &mut self.tools {
            tool.tick(&self.ctx);
        }
    }
}
```

### Cancellation

Rust's rayon scopes or spawned tasks support cancellation via a flag; the pipeline's lives in the
run's `MemoryBudget` (`cancel` wakes every waiter, `is_cancelled` checked before each task —
`spec/libs/pipeline.md`).

### Browser deployment: Studio Web

**Keeping the door open from Phase 1.** Studio Web is not a first-release gate (see `plans/core/development_plan.md` "Phase 18"),
but the codebase must not drift into unportable dependencies while it waits. Workspace guardrail 6
(`cargo check --target wasm32-unknown-unknown` for every lib crate in CI) enforces this from
the first commit; it costs minutes per PR and turns the later web build from an archaeology project
into UI wiring plus the pipeline adapters. The desktop binary remains the primary distribution.

### Architecture separation

```
lib crates (pes_savefile, cpk, fpk, fmdl, pes_model, ftex, dds_convert, fox2, vtree, archives, teams_list, aesthetics_export, pipeline, model_convert, aatf, music_export, audio_engine, match_feed, kit_config, ...)
    ↑ used by
tool crates (team_compiler, save_editor, stadium_compiler, export_upgrader, music_player, music_export_editor, match_tracker, kit_config_editor, refs_arranger, balls_compiler, player_aesthetics_editor, team_creator, ruleset_editor, db_generator) — each implements StudioTool
    ↑ registered by            ↖ trait + shell + common widgets defined in studio_core
studio (single binary: GUI when run without args, CLI subcommand dispatch with args;
        native binary + WASM web build)
```

The `pause()` pattern from Blue/Red disappears entirely. The pipeline returns warnings/errors as
data. The GUI or CLI layer decides how to present them (log panel, status-bar notices, dialogs for
decisions, console output).

---
