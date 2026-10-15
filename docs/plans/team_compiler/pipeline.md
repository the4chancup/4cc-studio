# 4cc Studio — Team compiler plan: Pipeline

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Per-export pipeline walkthrough

### 1. Reader

What exists of this is in `spec/team_compiler/pipeline.md` "1. Reader".

In **Discovery** (step 1): In the GUI this scan is
continuous (folder watcher, see `plans/team_compiler/gui.md` "Live validation"); in the CLI it runs once.
A disabled export is omitted from the grid.

In **Load** (step 4): the eagerly loaded structure is also what the grid needs.

### 2. Per-export serial steps (coordinator)

What exists of this is in `spec/team_compiler/pipeline.md` "2. Per-export serial steps (coordinator)".

In **Validation, in two passes** (step 2), after the deep pass: Plain-folder full checks run both
passes; packed archives run the structure pass during live
checking and this deferred deep pass at compile start before eligibility is decided. Among the
categories the two passes cover, **Shared folders** (`Faces/`, `Boots/`, `Gloves/`): same model
checks, including glTF-only
material files (`materials.toml` catch-all and `*.materials.toml` name-matched; see the
[Unified model format plan](../model_format.md)'s "Material files"). Among the **Root files**,
`ref_lists.txt` is the Refs arranger's file for the Fox referee hook, and a missing referee
roster's invalid `ParsedAestheticsExport` remains available to the Refs arranger for repair.
Under **Portraits/Logo/Common/Collars**: Collar findings map to the team-name cell (collars
have no cell of their own).

In **Export identity resolution** (step 3): The GUI also performs an early lookup after
structural validation so the live grid can show or edit the ID immediately. An unresolvable
normal team turns the grid's **team ID cell** red and editable — the user types an ID, confirms
with Enter, and it is written to the teams list (see "GUI: the live-validating team grid").

In **Player folder categorization** (step 6), assigned IDs are written on pre-Fox into the
savefile, so a compiled team CPK and its savefile must be deployed
together, and the aesthetics patch written with the CPK is how they travel together when the
savefile is on someone else's machine (see "Post-processing"). A missing local savefile stays a **warning** that
skips only the savefile step: the patch is still written, and because allocation is
deterministic (fixed per-team blocks, exclusive IDs by player number, shared IDs in alphabetical
order), recompiling the same exports later with a savefile configured produces identical IDs, so
a CPK-now/savefile-later split remains consistent. The deploy-together rule is distribution
guidance, not a compile blocker.

### 3. Per-model-folder parallel steps (rayon)

What exists of this is in `spec/team_compiler/pipeline.md` "3. Per-model-folder parallel steps (rayon)".

In **Format conversion** (step 1): duplicate sources within the selected representation are
ambiguous and drop the folder (`model_source_ambiguous`).

In **Packing** (step 7), a gloves output on PES 15-17 (a shared folder's, or an `ingame_face`
player's own, built by the same code; his textures name game paths, so only the `.mtl` names
change for him): every `.mtl` the output packs
(a member's, or a converted model's `<stem>.mtl`) sits at the output's root, named by its path
below the folder with each `/` written `_`, lowercased (`grip/materials.mtl` packs as
`grip_materials.mtl`, a converted `grip/glove_r.fmdl`'s set as `grip_glove_r.mtl`); `glove.xml`
names it there (`material="./grip_materials.mtl"`) while the model keeps its subfolder path
(`./grip/glove_r.model`). Its textures are then named from the root, `./<stem>.dds` or
`./<sub>/<stem>.dds`, never `../`; the textures stay where they are packed now, and the lookup
still resolves nearest first from the `.mtl`'s source folder. The game does not load a `.mtl`
texture path that climbs out of the `.mtl`'s folder: in PES 17 Edit mode (2026-10-15) a right
glove whose `grip/` `.mtl` named `../magenta.dds` drew white, and so did the game-rooted
`model/character/glove/<id>/magenta.dds`, while a root `.mtl` naming `./magenta.dds` or
`./sub/magenta.dds`, a `grip/` `.mtl` naming `./magenta.dds` beside it, and a `grip/` model
whose `.mtl` sat at the root all loaded. We move the `.mtl` files rather than copy each
texture a subfolder `.mtl` reaches above into its folder, which the textures task, emitting
each texture once at its path, would have to learn from the `.mtl` files; and rather than pack
the folder flat, which would collide two subfolders' `glove_r.model`. Two `.mtl` files whose
flattened names meet (`grip/a.mtl` and a root `grip_a.mtl`) fail the task as any two files of
one packed name do. PES 15 and 16 are unmeasured and taken to read as PES 17 does (the same
engine).

### 4. Per-export non-model steps

What exists of this is in `spec/team_compiler/pipeline.md` "4. Per-export non-model steps".

On Fox the **player appearance tables** accumulate the same way as the kit bins: every compiled
player gets his `PlayerAppearance.bin` row (the appearance bytes built from his resolved
`settings.toml`, its defaults for absent keys, nothing from the installed row) and his
`BootsList.bin` and `GloveList.bin` rows (the resolved IDs; a failed output still keeps its
installed row), and a compiled player of a `Full` export's team whose folder holds no boots or no
gloves gets the default-ID rows.

### 5. Writer (single-threaded)

What exists of this is in `spec/team_compiler/pipeline.md` "5. Writer (single-threaded)".

Under **Output modes and targets** (step 5), sideload mode: for PES 15/16, where no sideloading
runtime exists, the GUI disables the mode.

### 6. Post-processing

What exists of this is in `spec/team_compiler/pipeline.md` "6. Post-processing".

- **Degraded run: promotion to `output/`** — the savefile step is skipped, and the GUI offers
  **Retry** (for the locked case — the user closes PES and retries the deployment alone, no
  recompile) and **Open output folder** (a status-bar notice action after the run, and a
  permanent entry in the Compile dropdown).
- **`--no-deploy` (CLI only)** — skips the savefile step as well as the deployment.
- **Why no `move_cpks`.** The savefile step reads the install's save, one more way the pipeline
  is built around a present install.
- **DpFileList upgrade** — with any of the preflight's DpFileList findings the GUI shows
  **Upgrade DpFileList**, which asks before it writes. The dialog is honest about the override:
  it lists every installed entry that the official list does not contain and that is not
  renamed as "will no longer be loaded", and shows every rename before anything is done.

- **DpFileList upgrade** — for each entry the dialog lists as "will no longer be loaded" whose
  `.cpk` still sits in `download/`, it shows the size and offers deletion (renaming a 3.7 GB file
  as a backup is pointless; the user confirms per file, and nothing is deleted without that
  confirmation).
- **Destination writability preflight** — the two failure modes users actually hit (the download
  folder needs elevation because PES sits under `Program Files`; the old CPK is locked because PES
  is still running) are detectable long before Compile, so they are checked **live**, like the
  version selector's exe check, not discovered at the end of a multi-minute run. The probe's failure
  kind is read from the OS error — `ERROR_ACCESS_DENIED` → elevation needed, `ERROR_SHARING_VIOLATION`
  → in use (cross-checked against the core plan's running-PES poll, which names the culprit). The
  check runs at startup, on window focus, whenever a relevant setting changes (`pes_folder_path`,
  `pes_version`, CPK names, `multicpk_mode`), and whenever the PES process poll changes
  state. Results show on the Compile button as a warning badge with the reason in the tooltip and a
  line in the run strip: *"PES 2019 is running — close it before compiling"* / *"The download
  folder needs administrator rights"* with a **Relaunch as administrator** action (the `elevation`
  lib). The button stays enabled — the user may close PES during the compile — except for the
  elevation case, where compiling first would waste the run (relaunching loses it), so the click
  prompts for elevation up front.
- **Aesthetics patch** — written to the output folder (`output_folder_path`) as
  `aesthetics_patch.toml`, one per run in every output mode, its header naming the CPKs the run
  publishes (none for a sideload tree): the resolved
  savefile writes for the compiled players (format and rules in "Aesthetics patch" in the
  [Savefile plan](../pes_savefile/operations.md)). On **Fox** it holds the compiled players'
  motions and a `write_names` export's names (the rest is in the CPK's player appearance tables,
  "Bins accumulation") and, as on pre-Fox, is written with the CPKs as one more file of the
  run's publication, test and sideload trees included (none when the run publishes nothing,
  the previous patch staying as it was; a failed write is `patch_write_failed`), except that a run with nothing to put in it writes
  none and moves an earlier patch to `aesthetics_patch.toml.bak`; so an earlier run's patch
  never stands beside a newer CPK. The
  savefile builder applies it whole; names come only from an export carrying the `write_names`
  marker, so a managed team's players keep their manager's names. On
  **pre-Fox** it holds everything (names only from a `write_names` export): settings.toml settings with
  `name = true` and FPC markers resolved to concrete values, the authored stock boots/gloves IDs
  (else the marker's default) for categories requesting no standalone output, and the
  auto-assigned boots/gloves IDs **only for content that was actually packed**: if a boots/gloves task failed, the affected players
  keep their existing savefile IDs rather than pointing at absent CPK content, while their
  independent valid settings still apply. The patch travels with the CPKs: its header names the CPKs it
  describes, whether or not this machine deployed them. This is what separates the **DLC builder**
  from the **savefile builder**: the former compiles every export and hands over CPK + patch, the
  latter applies the patch in the save editor to the official save, which the former never touches
  (it holds the teams' custom tactics, which only the savefile builder receives). Reported as
  `patch_written` (I) with the path.
- **Savefile update** — the local savefile, when one is configured, is updated by **applying the
  patch just written** through `pes_savefile`'s `apply_patch` — the compiler has no second write
  path, so the local result and the savefile builder's later result are the same bytes by
  construction. Saving uses a `.bak` backup, the same convention as the save editor's, and is
  reported as `savefile_written` (I) with the path, as every other output of the run reports
  itself. A team or player of the patch the save lacks is reported per item
  (`savefile_team_missing`, `savefile_player_missing`, the save editor's `apply-patch` skips)
  and the rest is applied; a save holding none of the patch's players is not rewritten, so its
  `.bak` stays the user's previous backup. A missing
  savefile stays a warning (`savefile_missing`, naming the patch): even the DLC builder is expected
  to compile against a template save, to test the DLC in PES. **The savefile is never written while
  the compile target's PES is running**, in any output mode: the game holds its edit data in memory
  and writes it back on exit, so a write made now would not go live without a restart and would be overwritten by whatever
  the player changed in-game before closing. The step is skipped with `savefile_skipped_pes_running`
  (W) telling the user the settings will be applied by the next compile made with PES closed; the
  rest of the run is unaffected. This matters mostly for **sideload mode**, whose whole point is
  compiling while PES stays open — model iterations land immediately through sideloading, and the savefile
  half of the export (on Fox the motions and names; on pre-Fox also boots/gloves IDs and player
  settings) catches up on the first compile after PES is closed. On Fox the patch's write leaves
  every appearance id as it is, so a stripped local save stays stripped. In normal mode a running PES already fails deployment on the locked CPK, so the rule
  adds nothing there. The check uses the same running-PES detection as the core plan's
  version-selector poll; the CLI performs it once, at the savefile stage. Only the target
  version's game counts, the Save editor's rule for its writing commands (`save_editor.md`
  "Writing a save"): the savefile is always the target's version (another is
  `savefile_invalid`), and a game only writes back its own version's save, so a PES of another
  version running leaves the step to run. The savefile is likewise
  **skipped whenever the CPKs were not deployed** — a degraded run promoted to `output/`, a
  deployment refused by its preflight, `--no-deploy`, or test mode, whose loose tree in
  `output/` the game never reads. Sideload mode counts as deployed: its tree in `livecpk\` is
  what the game loads, and its savefile half catching up once PES is closed is the reason the
  running-PES skip exists. The incoherence it prevents is pre-Fox's, where a savefile can point at
  boots/gloves IDs whose content is not installed — exactly what the "only for content actually
  written" rule exists to prevent; on Fox a compiled player's IDs live in the CPK's own table
  rows, but the same skip keeps one rule for both engines.
- **One transaction: CPKs, patch, savefile** — a deployment keeps every old CPK it replaces
  aside as `{name}.cpk.old` (one CPK too, not only a multi-CPK run) until the patch is written
  and the savefile step has ended without `savefile_invalid` or `savefile_write_failed`; only
  then are the `.old` files removed. Either of those two findings, or `patch_write_failed`,
  rolls the deployment back: the run's CPKs leave `download/` and the old ones return, so the
  game keeps loading what it loaded before the run, and the finding is Fatal (`AbortRun`,
  exit 3). After a savefile failure the staged CPKs are promoted to `output/` beside the
  patch, which names them, so the compiled artifacts remain available; after a patch failure
  nothing more is published, as for every `AbortRun`. A `.old` left by a run that was killed
  before its commit needs nothing: the next run's move aside replaces it (a rename replaces an
  existing file) and its commit removes it. We keep the old CPK
  aside, not rename the new one over it as the one-CPK deployment did, because a rename over
  it cannot be undone once the savefile fails. Sideload mode has no rollback: its previous
  `livecpk\` tree is gone once the new one is in place, and keeping it would cost a copy of
  every model on each iteration, the mode's whole point being speed; its savefile failures
  stay Errors (exit 1), the new tree in place. `savefile_team_missing` and
  `savefile_player_missing` roll nothing back: the rest of the patch is applied.
- **Run PES** — optional launch of `PES20{version}.exe`, only after the complete deployment
  transaction succeeds. Built in Phase 8 together with sideload mode's Launch PES button, as one
  launcher: both start the same exe the same way, and the GUI phase is where its only manual
  check lives.

### Resolved decisions and open questions

What exists of this is in `spec/team_compiler/pipeline.md` "Resolved decisions and open questions".

Resolved decisions:

- **The aesthetics patch is the compiler's only savefile write path**: a compile writes
  `aesthetics_patch.toml` to its output folder with the CPKs it publishes (on Fox only motions and a `write_names` export's names, the rest travelling in the CPK's
  player appearance tables); a configured local savefile is updated by applying that
  patch, never by a separate write. Reason: the DLC builder and the savefile builder are different
  roles — the official save carries the teams' custom tactics, which the DLC builder must not have —
  so the compile's savefile half has to be a file that can be carried to another machine and applied
  there; making the local write use the same file removes the possibility of the two drifting. A
  missing local savefile remains a warning, not a supported quiet mode, because the DLC builder
  still compiles against a template save to test in PES.
- **Teams list: embedded upstream, one working copy in the data directory.** `data/teams_list.txt`
  is edited by the grid's ID cell and by the updater's merge (see the core plan's "Self-update"). We do this,
  not Red's loose file beside the exe, because the grid writes into the list, which makes it user
  data: it must follow the settings file into the user config directory when that mode is chosen,
  or a replaced program folder loses the user's ID assignments; and a single on-disk file leaves
  nothing for a user to edit by mistake. **Writes are best-effort, never
  elevated**: when the data directory is not writable (the typical case is a portable install
  unpacked under Program Files), the ID cell stays read-only with the reason in its tooltip and
  the updater's merge is reported as skipped (`teams_list_read_only`) — Studio is a portable app,
  and elevating for a one-line edit is handling a corner case nobody should be in. Detection is by
  attempting the write, not by inspecting the path: there is no list of protected directories to
  keep right. **The savefile is a second source for the same merge.** In-game team names in a 4cc
  save are the `/xx/` names, so the savefile's `TeamEntry { id, name }` rows for IDs 701–920 are
  a teams list — and the authoritative one for the cup the user is actually playing, ahead of
  whatever the last release embedded. The [Save editor](../save_editor.md)'s **Export teams list**
  action (it owns the open savefile; the compiler only consumes the list) runs the *same*
  reconciliation as the updater with the savefile as the incoming list (added / kept / overridden,
  reviewed before writing; savefile value wins a conflict), never a blind overwrite. It is
  on-demand, not automatic: the list must keep working with no savefile present
  (`savefile_missing` is a supported mode), and rewriting user data as a side effect of a compile
  would be a surprise. Placeholder rows (`Backup N`, `Invitational N`) come along and are inert.
- **Missing savefile stays a warning**: deterministic ID allocation keeps a later savefile-inclusive
  recompile of the same exports consistent, so the CPK step may proceed.
- **Check-cache identity**: recursive manifest fingerprint combined with teams-list and
  validation-settings revisions; the watcher only triggers re-fingerprinting (see "Live
  validation").
- **Shallow archive checks never decompress solid data**: `.zip` per-entry access may read small
  metadata; solid-`.7z` content checks defer to the compile-start deep check, with the roster
  recorded as unread.
- **Model source selection**: in-representation duplicates are rejected (`model_source_ambiguous`).
- **Source snapshot**: The GUI watcher (Phase 8) keys its stale-result envelopes on
  the same value.

Open questions — **implementation/spec work** (no user preference involved; resolved during the
phase that owns them):

- **Shared/Common dependency graph.** Define cache ownership when a player task needs shared or
  Common models for a Fox merge while those folders also have independent output work (each
  player task reads and converts its parts itself today; the ordering of Fox's Common textures is
  settled above).
- **`teams_list.txt` contract.** Still open: atomic writes
  and concurrency between the grid's ID-cell write, the updater merge and the savefile import
  (callers own I/O; the one writer so far, `compile`, only creates a missing file, so the
  question opens with the grid's ID-cell write in Phase 8 and the update merge of Release 0.1.0).
- **Run-result semantics.** Define separate compile and deployment outcome types so scoped errors,
  `pass_through`, failed moves, and failed savefile writes map predictably to GUI state and CLI exit
  codes. The exit codes are fixed (`spec/team_compiler/settings.md` "CLI"): deployment adds none, a degraded run
  exits 1 and an aborted one 3; what remains is the outcome types for GUI state. Deployment is
  transactional across generated CPKs and the savefile, with backups and rollback; PES launches
  only after the complete transaction succeeds. (No system CPK is part of it: the Fox referee
  marker is a reserved collar inside the refs CPK, `spec/team_compiler/blue_port.md` "Referee export processing".)
- **Superseded installed outputs.** Within the teams slot run this is solved: every slot is written
  each multi-CPK run (content or the empty placeholder), so stale parts cannot survive. What remains
  is the single-CPK-versus-multi-CPK crossover (a `4cc_99_test` left from a test compile is not
  touched by a DLC compile, and vice versa — decide whether that needs a message) and the rule that
  an unrecognized user file is never deleted; retired official names are handled by the DpFileList
  upgrade's confirmed per-file deletion.
