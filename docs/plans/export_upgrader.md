# 4cc Studio — Export upgrader plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

The Export upgrader is built through the CLI (`spec/export_upgrader.md`, Phase 6). What is
left is step 13, the optional glTF pass (Phase 7), and the tool's view (Phase 8).

---

## What it does

What exists of this is in `spec/export_upgrader.md` "What it does", steps 1 to 12. The last
step:

13. **Optional glTF conversion** (`--convert-to-gltf`, off by default) — after the structural
    migration is complete and verified (step 10's report is clean), a second pass converts all
    model files to glTF in place: every `.fmdl` (Fox exports) or `.model`+`.mtl` (pre-Fox exports)
    in player folders, shared folders, and Common is imported to the IR and exported as one `.glb`
    per model via `model_convert`'s `ir_to_gltf`. Old exports are always single-engine (Fox-only or
    pre-Fox-only — Red cannot handle mixed states), so there is no dual-set superset merge to
    automate: each model converts plainly through one path. Skeleton transforms come from the
    FMDL's companion SKL or the embedded template skeleton, or from a `.model`'s inline bone table
    (see "Skeleton reconstruction from FMDL" and "Skeleton in `.model`" in the [Model conversion
    plan](model_conversion/README.md)). Textures stay in place
    (existing DDS/FTEX referenced by stem from the generated `materials.toml`, whose entries carry
    the `family` inferred from the native shader plus the native engine table — see "Emission" in
    the [Unified model format plan](model_format.md)). A `.mtl.common` link converts to the
    matching `.materials.toml.common` link, and the Common `.mtl` it points at is converted in
    place like any other; texture links (`hair.dds.common`, created by step 7's path normalization
    or already present) are format-neutral and stay as they are. For pre-Fox
    exports, `.mtl` texture paths are stripped of extensions (normalized to stem-based references,
    matching the Studio format). Replacement is transactional per model folder: all parts convert
    first, and the structurally-migrated originals remain recoverable until publication completes.
    Multiple writes/deletes are not a single atomic filesystem operation. Conversion or publication
    failure restores the native folder, or retains a clearly identified recoverable copy if rollback
    itself fails; success is not reported for a partial migration. The two-step design keeps the
    structural migration lossless and parity-testable: the test suite validates the structural step against Red's reference output,
    and the glTF conversion has its own roundtrip tests in `model_convert`.

## CLI

What exists of this is in `spec/export_upgrader.md` "CLI". Step 13 adds `--convert-to-gltf` to
`upgrade` (off by default):

```text
4cc-studio export-upgrader upgrade <export> [--savefile <EDIT>] [--loose] [--convert-to-gltf] [-o <dir>]
```

## GUI

The tool view is the two commands as two drop targets. It is drafted first
(`docs/prototypes/tools/export_upgrader.js`, `core/gui.md` "Prototypes before implementation") and
refined with the maintainer before `view/` is written:

- **Two drop targets**, side by side: *Upgrade an export* (a folder, `.zip` or `.7z`) and
  *Upgrade model folders* (a folder of old-format model folders, "Model-folder mode"). A path is
  dropped on a target from outside the window or picked with the target's Browse button
  (`rfd`). The view holds one input: a drop on either target clears the other, and Upgrade runs
  the command of the target that holds it.
- **The run's arguments**, under the first target, the CLI's and nothing more: the savefile
  field, prefilled with the newest save `pes_savefile::discovery` finds for the selected PES version
  (empty when it finds none; a Browse button beside it), and the **loose** switch (step 11).
  Exactly one of the two is taken, so Upgrade is enabled once either is set; both are disabled
  for a referee export (its name's first word `refs`), which takes neither, so for it Upgrade is
  enabled as soon as the target is filled, and likewise while the second target holds the input. An **Output into**
  field holds the directory the output folder is written inside (`-o`): the input's parent by
  default, so the output lands beside the input, as the CLI's default. For the second target
  the same field is the output directory itself (`upgrade-models` writes its folders straight
  into its `-o`), `<input> upgraded` beside the input by default, as the CLI's.
- **The Upgrade button** runs the command on a thread of its own through the CLI's own parser,
  as the Team compiler's Compile does (`spec/core/development_plan.md` "Phase 3", the shell
  slice), so the view cannot upgrade anything the command line would refuse; a refusal (a
  savefile without the team, exit 2) and an abort (an output folder that already exists, exit
  3) show as the CLI prints them.
- **The report** is the view's product: step 10's findings as the console prints them, in the
  log panel, and above it the **left-behind** list, one row per `unmigrated_content` or
  `leftover_dropped` entry with its input path, the finding's code and the entry's context (a
  Note line, a kit config's error), so the author sees in one place what to fix in the input
  (an unmigrated entry, which kept anything from being written) or what the output left out (a
  leftover). For `upgrade-models` the per-folder "where it
  goes" lines are the log's, as the CLI prints them, and the list holds the folders it left
  out. The run ends with the exit code's meaning in words (clean; not clean, nothing written,
  see the list; refused; aborted). A clean run, the only one that writes output, also gets the
  notice `Upgraded <name> · [Open output folder]` in the status bar and, in the view's done
  line, a **Move to exports and compile** button: the output is moved into the exports folder
  and the Team compiler made active (`ctx.switch_to_tool`, as the Refs arranger hands off),
  the watcher then checking it like any dropped export. It is a button rather than the
  notice's action because a notice carries at most one (`spec/core/gui.md` "Status bar"), and
  Open output folder stays useful to a member who compiles elsewhere. Any other run publishes
  no notice, having nothing to open.
- No settings section: the tool has no settings (step 6: the teams list path is the compiler's).

## Dependencies

What exists of this is in `spec/export_upgrader.md` "Dependencies". The optional glTF
conversion (step 13) uses `model_convert`'s `ir_to_gltf` path, which makes `model_convert` a
build dependency; the runtime `--convert-to-gltf` flag only determines whether that path is
executed.

## Acceptance

What exists of this is in `spec/export_upgrader.md` "Acceptance", whose rules (format, the
defaults a scenario naming no version or export takes, the test data) the scenarios below
follow. Step 13's scenarios are written with Phase 7.

**Phase 8 scope.** Phase 8 delivers the view ("GUI"). What a scenario observes is what a frame
draws, through the headless harness (the frame's AccessKit nodes; a dropped path as egui's
dropped-files input; a click as press and release frames), the output written, and the notice
published; `manual` marks only what needs a native dialog. The inputs are Phase 6's fixtures,
copied under the test's own folder; the discovered save is one placed under a Documents folder
there. What the upgrade writes is the Phase 6 scenarios'; these observe that the view reaches
the same command with the arguments shown and shows its report.

```
EU-GUI-01  GIVEN the selector at PES 21, an old export folder dropped on the Upgrade-an-export
                 target and a PES 21 save under the test's Documents folder
           WHEN the view is drawn
           THEN the target names the export, the savefile field holds the discovered save, the
                loose switch is off, the Output-into field reads the input's parent folder, and
                Upgrade is enabled; GIVEN no discovered save THEN the field is empty and Upgrade
                is disabled until a save is picked or loose switched on; WHEN loose is switched
                on and Upgrade clicked THEN the run is `upgrade --loose` with no savefile
                argument, its output the one `upgrade --loose` writes for that export
EU-GUI-02  GIVEN EU-GUI-01's export and save, the export's Other/ holding one file
           WHEN Upgrade is clicked
           THEN the output folder is the one `upgrade --savefile` writes (EU-ROOT's naming), the
                log shows the report's lines as the console prints them, the left-behind list
                holds one row, the Other/ file's leftover_dropped with its path, the last line
                gives the exit code's meaning (clean), the notice "Upgraded <name>" with its
                Open output folder action is published, and the done line shows Move to
                exports and compile; WHEN that is clicked THEN the output sits in the exports
                folder and the Team compiler's view is active
EU-GUI-03  GIVEN the first target filled as in EU-GUI-01, then a folder of old model folders,
                 one of them holding no model file, dropped on the Upgrade-model-folders target
           WHEN Upgrade is clicked
           THEN the first target was cleared by the drop and the savefile field and loose switch
                disabled, the Output-into field read "<input> upgraded" beside the input, the run
                is `upgrade-models`', the left-behind list holds one row for the folder without
                models, with its path and unmigrated_content, nothing is written (EU-MF-03), no
                notice is published and no Move to exports and compile is shown
EU-GUI-04  GIVEN a referee export dropped on the first target
           WHEN the view is drawn
           THEN the savefile field and the loose switch are disabled and Upgrade is enabled;
                WHEN Upgrade is clicked THEN the run is `upgrade` with neither a savefile nor
                loose argument, the output EU-REF-01's (its players.txt from refs.txt)
EU-GUI-05  GIVEN an input whose output folder already exists
           WHEN Upgrade is clicked
           THEN the log shows the abort naming the folder (EU-CLI-05), the last line reads
                aborted, nothing is written and no notice is published
EU-GUI-06  manual  GIVEN the view open
                   WHEN a target's Browse button is clicked
                   THEN the native picker opens for a folder (or, on the first target, a folder or
                        archive) and the picked path fills the target as a drop would
EU-GUI-07  GIVEN EU-GUI-01's upgrade in flight, another tool's view active
           WHEN the window's close is requested
           THEN the Export upgrader's view is active with its prompt; GIVEN close chosen THEN
                the window closes after the run's last line, the output EU-GUI-01's; GIVEN
                the window kept THEN the run ends as it would have
```
