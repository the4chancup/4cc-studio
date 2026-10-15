# 4cc Studio — Team compiler plan: Settings

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Settings

### Team compiler settings

What exists of this is in `spec/team_compiler/settings.md` "Team compiler settings".

| Setting | Old default | New default | Controls |
|---|---|---|---|
| `run_pes` | 0 | same | Launch PES after compiling (only after a successful deployment; a degraded run never launches) |
| `quick_compile_close_on_success` | — | 0 | After a compile started by GUI autorun (`quick_compile.bat` → `4cc-studio --gui team-compiler compile`), close the window when the run completes with no Error-level findings and nothing skipped for errors; stay open on errors, failed deployment, or cancellation so the grid/log can be reviewed. Warnings alone still close (they are in the logs, as Red's `pause_allow = 0` reasoned). Never applies to a manual Compile click |

### Path resolution

What exists of this is in `spec/team_compiler/settings.md` "Path resolution".

| Artifact/input | Base location |
|---|---|
| Relative `exports_folder_path` | Created when missing (from Phase 8 also at GUI launch, for the watcher) |

### CLI

What exists of this is in `spec/team_compiler/settings.md` "CLI".

The GUI autorun form extends the spec's command list:

```text
4cc-studio --gui team-compiler compile [exports-root] [--mode normal|test|sideload]   # GUI autorun
```

A sideload-mode run with PES open skips the savefile step
(`savefile_skipped_pes_running`, see `pipeline.md` "Post-processing") — the model iteration lands through sideloading and
the savefile catches up on the first compile after PES is closed.

The `--gui` form (the core plan's "Launch modes") is what `quick_compile.bat` runs. The tool's
`gui_run` implementation accepts `compile` only — `check` is meaningless in the GUI, where checking
is live — and queues the compile until the initial exports check has completed, then triggers the
same code path as the Compile button with the parsed `--mode`. `quick_compile_close_on_success`
governs what happens when that run ends.
