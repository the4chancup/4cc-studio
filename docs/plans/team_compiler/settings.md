# 4cc Studio — Team compiler plan: Settings

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Settings

### Team compiler settings

What exists of this is in `spec/team_compiler/settings.md` "Team compiler settings".

| Setting | Old default | New default | Controls |
|---|---|---|---|
| `run_pes` | 0 | same | Launch PES after compiling (only after a successful deployment; a degraded run never launches) |
| `savefile_path` | — | auto | `EDIT00000000` to update. `auto` runs `pes_savefile`'s discovery under the user's **Documents\KONAMI** folder for the selected version (the savefile is never inside the PES install, so `pes_folder_path` plays no part — see "Savefile discovery" in the Savefile plan); newest account wins when several exist (`savefile_autodetected`); nothing found → `savefile_missing` (Red never touches the savefile) |
| `quick_compile_close_on_success` | — | 0 | After a compile started by GUI autorun (`quick_compile.bat` → `4cc-studio --gui team-compiler compile`), close the window when the run completes with no Error-level findings and nothing skipped for errors; stay open on errors, failed deployment, or cancellation so the grid/log can be reviewed. Warnings alone still close (they are in the logs, as Red's `pause_allow = 0` reasoned). Never applies to a manual Compile click |

### Path resolution

What exists of this is in `spec/team_compiler/settings.md` "Path resolution".

| Artifact/input | Base location |
|---|---|
| `savefile_path` | Absolute when explicitly set; `auto` resolves under the shell's Documents folder → `KONAMI\{game folder}[\{account id}]\save\EDIT00000000` per the Savefile plan's discovery table — independent of `pes_folder_path` |

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
