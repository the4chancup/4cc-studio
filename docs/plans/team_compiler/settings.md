# 4cc Studio — Team compiler plan: Settings

Part of the [Team compiler plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Settings

Inherited from Red/Blue (`settings.ini`, `settings_default.ini`), reorganized into the suite's
two-level settings model: **common settings** live in `studio_core` (shared by all tools), **tool
settings** live in the Team compiler's own settings section.

### Common settings (studio_core)

| Setting | Old default | Notes |
|---|---|---|
| `pes_version` | 15 | Sidebar selector; 15–21. 4cc Studio defaults to 19 as the version most relevant to the current community; derives `fox_mode` (≥18), per-version file extensions, game paths, bin names |
| `pes_folder_path` | `C:\Program Files (x86)\Pro Evolution Soccer 20**` | `**` replaced with the version; derives `download/` path and `PES20{version}.exe` path |
| `exports_folder_path` | `exports/` | Watched by the GUI and scanned once by the CLI; belongs in `studio_core` so the Team, Refs, and Balls tools share one export source. Relative paths resolve **beside the executable** in both data-location modes (see "Path resolution"), so the folder sits next to `quick_compile.bat` like Red's `exports_to_add/` |
| `thread_count` | 0 (auto) | Was a Blue CLI arg; auto = logical cores − 1 |
| `memory_cap_percent` | 80.0 | Was a Blue CLI arg; memory budget cap |
| `check_for_updates` | 1 | Was per-compiler (`updates_check` in Red's ini); becomes a suite-wide update check |

### Team compiler settings

"Old default" is Red's value (— for settings Red did not have); "New default" is the Studio's. A
differing new default is a deliberate decision, explained in the Controls column.

| Setting | Old default | New default | Controls |
|---|---|---|---|
| `cpk_name` | `4cc_90_test` | same | Single-CPK output `CpkStem` (shared validation contract in the Writer section) |
| `output_folder_path` | `patches_output/` (hardcoded) | `output/` | Root for promoted CPKs (degraded runs and `--no-deploy`), the `.staging/` folder, `test_output/`, `sider_output/`, and `teamnotes.txt`. Relative paths resolve **beside the executable** (Red's `patches_output/` in its new place; `%APPDATA%` is no place to look for a CPK) |
| `sider_output_path` | — | `sider_output/` | Where Sider-mode runs write the unpacked PES folder structure; relative paths resolve beneath `output_folder_path`. For the prototyping loop the user either points this at Sider's livecpk root directly or adds the default folder to `sider.ini`'s `cpk.root` list — the settings UI shows the resolved absolute path for copy-pasting into `sider.ini` |
| `run_pes` | 0 | same | Launch PES after compiling (only after a successful deployment; a degraded run never launches) |
| `multicpk_mode` | 0 | same | Cup DLC mode: team content into size-split `teams` parts + the bins CPK (see "Multi-CPK mode: teams parts"); replaces Red's faces/uniform/bins content split |
| `teams_cpk_name` | `4cc_40_faces` + `4cc_45_uniform` | `teams` | Stem of the teams part slots; the slots themselves are every DpFileList entry matching `{prefix}_{NN}_{stem}` (`4cc_40_teams`, `4cc_41_teams`, …), ordered by number. Replaces `faces_cpk_name` and `uniform_cpk_name` |
| `cpk_part_max_size` | — | `3 GB` | Cap per teams part (TOC included); the writer places whole teams, rolling to the next slot when the next team would not fit — teams are never split across parts. Chosen comfortably under Git for Windows' 4 GiB object ceiling; five slots give 15 GB. Single-CPK runs are not split, only warned (`cpk_size_over_limit`) |
| `bins_cpk_name` | `4cc_08_bins` | same | Multi-CPK bins name |
| `refs_cpk_name` | `4cc_35_referees` | same | Referee CPK name |
| `dds_compression` | 0 | **auto** | WESYS-zlib every emitted DDS. Tri-state `auto` / `1` / `0`; `auto` **follows `multicpk_mode`**, since multi-CPK is the cup DLC workflow and containing the full-cup DLC's size is the whole point of the setting — a manager compiling one team gets no compression and no cost, a cup maintainer gets it without remembering a second switch. **Pre-Fox only (PES ≤17)**: on Fox versions the setting is ignored whatever its value, because FTEX conversion already provides the size reduction there and the game does not expect zlibbed FTEX. Cheap in-process either way (see "DDS compression cost" below) |
| `strict_file_type_check` | 1 | same | Disallowed file types are errors vs info notes (`file_type_disallowed`) |
| `pass_through` | 0 | same | Keep folders with errors instead of discarding them (cup DLC workflow) |
| `savefile_path` | — | auto | `EDIT00000000` to update. `auto` runs `pes_savefile`'s discovery under the user's **Documents\KONAMI** folder for the selected version (the savefile is never inside the PES install, so `pes_folder_path` plays no part — see "Savefile discovery" in the Savefile plan); newest account wins when several exist (`savefile_autodetected`); nothing found → `savefile_missing` (Red never touches the savefile) |
| `teams_list_path` | `teams_list.txt` (hardcoded, beside the exe) | `teams_list.txt` | Location of the working teams list; relative paths resolve in the data directory (see "Path resolution"). Created from the embedded list on first run |
| `dt00_overwrite_allow` | — | 0 | Allow `ref_marker.dds` injection into the `dt00_x64.cpk` system file (replaces Red's interactive prompt) |
| `quick_compile_close_on_success` | — | 0 | After a compile started by GUI autorun (`quick_compile.bat` → `studio --gui team-compiler compile`), close the window when the run completes with no Error-level findings and nothing skipped for errors; stay open on errors, failed deployment, or cancellation so the grid/log can be reviewed. Warnings alone still close (they are in the logs, as Red's `pause_allow = 0` reasoned). Never applies to a manual Compile click |

Output mode (normal / test / sider) is not a persisted setting: it is the Compile button's dropdown
in the GUI and a flag on the CLI subcommand, as in Blue's `--mode` argument.

**DDS compression cost.** Red's `dds_compression` is a separate pass at the end: walk
`patches_contents/`, read each DDS, Python `zlib.compress` at level 6, wrap in the 16-byte WESYS
header, write it back — I/O-bound and serial. In the Studio it is one more step in the texture task,
on bytes already in memory, running on rayon workers like everything else. Rough budget for a
200 MB pre-Fox export that is mostly DDS: `flate2` deflate at level 6 runs at roughly 30–60 MB/s
per core on block-compressed texture data (DXT blocks are high-entropy, so it is neither fast nor
very effective — expect 10–30 % size reduction), i.e. ~4–7 s of CPU spread over ~7 workers, well
under a second of wall time per export and far below the CPU BC3 encode of any raster-source
texture in the same task. Two implementation notes: prefer the `zlib-rs` backend of `flate2`
(faster than `miniz_oxide`, pure Rust) and measure whether a lower level (1–3) loses meaningfully
on DXT data — it usually does not, and halves the cost; and the Red parity harness must compare
WESYS-wrapped entries **decompressed**, since deflate output is not byte-stable across
implementations and levels. Textures already WESYS-wrapped in the export are passed through, not
re-compressed (Red's `zlib_file` skip). Measure in Phase 4. With the `auto` default the cost is only
ever paid on cup-DLC compiles, where a few seconds against a 48-team run is irrelevant — the
measurement decides the compression level, not whether the feature is on.

### Path resolution

| Artifact/input | Base location |
|---|---|
| Logs and persistent state | Selected data directory |
| Relative `exports_folder_path` | Executable directory in both data-location modes, so `exports/` sits beside `quick_compile.bat`; created on first run |
| Relative `output_folder_path` | Executable directory in both data-location modes (`output/` beside the exe, created on demand); holds `.staging/{run_id}/` during a run |
| `sideload/` input | Selected data directory (an explicit sideload setting may be added later) |
| Relative `teams_list_path` | Selected data directory — the file is user data once the grid writes ID assignments into it, so it follows the settings file (see `pipeline.md` "Resolved decisions", "Teams list"); nothing is bundled beside the binary |
| `templates/` override directory | Selected data directory; each file shadows the matching embedded template/fallback-bin resource (see `pipeline.md` "Resolved decisions") |
| `savefile_path` | Absolute when explicitly set; `auto` resolves under the shell's Documents folder → `KONAMI\{game folder}[\{account id}]\save\EDIT00000000` per the Savefile plan's discovery table — independent of `pes_folder_path` |
| `pes_folder_path` | Absolute PES installation path (after expanding its documented `**` version placeholder) |
| Generated CPKs, `test_output/`, `sider_output/`, `teamnotes.txt` | Resolved `output_folder_path` |
| Installed CPKs | `{pes_folder_path}/download/` |

The GUI and CLI use these same bases; a CLI exports-root override changes only that invocation's
resolved export source.

### Dropped settings

| Setting | Why it disappears |
|---|---|
| `cache_clear` | No disk cache: the pipeline is in-memory (VirtualTree), so there is no equivalent of Red's `patches_contents/` staging folder for the setting to clear |
| `move_cpks` | Deployment is always attempted; a run that cannot deploy degrades to `output/` by itself, and the deliberate "build but don't install" case is the CLI-only `--no-deploy` flag. See "Why no `move_cpks`" under `pipeline.md` "Post-processing" |
| `pause_allow` | The `pause()` pattern is gone; errors/warnings are events rendered by the GUI/CLI. Its unattended-run half (`pause_allow = 0`: finish without stopping) survives as `quick_compile_close_on_success` for launcher-started runs |
| `admin_mode` | Manifest-based UAC / GUI prompt (see core plan) |

### CLI

```text
studio team-compiler compile [exports-root] [--mode normal|test|sider] [--export <path>]... [--no-deploy]
studio team-compiler check [exports-root] [--export <path>]...
studio team-compiler upgrade-dpfl [--yes]      # replace the installed DpFileList with the bundled official one
studio --gui team-compiler compile [exports-root] [--mode normal|test|sider]   # GUI autorun
```

(Refreshing `teams_list.txt` from a savefile is the Save editor's `export-teams-list`; see the
[Save editor plan](../save_editor.md) "CLI".)

`upgrade-dpfl` prints what the override changes (entries that will no longer be loaded, with the
sizes of any matching `.cpk` files found in `download/`) and stops unless `--yes` is given; it never
deletes CPK files non-interactively — those are listed for the user to remove. See "DpFileList
upgrade" under "Post-processing".

The optional positional argument is an exports-root override for that invocation; when omitted, both
commands use the common `exports_folder_path`. It is not a single-export path and is never persisted
back to settings.

`--no-deploy` builds the CPKs into `output/` without touching the PES install or the savefile (see
`pipeline.md` "Post-processing"). It is CLI-only and has no GUI or settings counterpart: it is the cup maintainer's
DLC-production flag and the build-box flag, not something a regular user should find and leave on.
It is rejected together with `--mode test|sider`, which have no deployment to skip.

`--export <path>` (repeatable) restricts the run to the named exports — a path to an export folder
or archive, anywhere on disk, not necessarily under the exports root. This is the contract for
**external callers driving a Sider prototyping loop**, first of all the `pes-models` Blender
extension's "export model and compile for Sider" button: it saves the model into the player folder
it was loaded from (it knows the export from the launch manifest — see the Player aesthetics editor
plan), then runs

```text
studio team-compiler compile --mode sider --export "D:\exports\aaa_export"
```

and PES, with Sider running, shows the result on the next model load. Without `--export` the plugin
would recompile every export in the folder on each iteration. Console output follows the ordinary
`-` prefixed format, and the exit code distinguishes at least clean / errors in some scope / aborted
(the exact mapping is fixed by the "Run-result semantics" open question) so a caller can show a
one-line verdict and point at the log for detail. The CLI is a separate process with its own run
and does not require the GUI to be closed. A Sider-mode run with PES open skips the savefile step
(`savefile_skipped_pes_running`, see `pipeline.md` "Post-processing") — the model iteration lands via livecpk and
the savefile catches up on the first compile after PES is closed; bins routing in Sider mode is part
of the "Output-mode artifact routing" open question.

The `--gui` form (the core plan's "Launch modes") is what `quick_compile.bat` runs. The tool's
`gui_run` implementation accepts `compile` only — `check` is meaningless in the GUI, where checking
is live — and queues the compile until the initial exports check has completed, then triggers the
same code path as the Compile button with the parsed `--mode`. `quick_compile_close_on_success`
governs what happens when that run ends.

---
