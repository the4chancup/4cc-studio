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

One addition to step 10, the built report: **`unmigrated_content` names its cause.** The code
covers about fifteen causes (a second face folder for a slot, a second item folder for an ID,
two kit textures landing on one name, a face folder without a valid slot, a texture that does
not decode, a folder loose mode cannot place, ...), and a refused entry is reported with no
cause (its context holds at most a Note `line` or a decode's `error`), so a manager whose
upgrade is not clean, the one outcome where the input must be edited, cannot act on the line
without this document. Its context gains `reason`, a closed set with one value per cause, each
named from its emission site, and, where two candidates claim one place, `other`, the other
candidate's input path. A closed set rather than free text, because the message text (Phase 8)
branches on it, a sentence per cause; the view's left-behind list shows it as the console
prints it.

## CLI

What exists of this is in `spec/export_upgrader.md` "CLI". Step 13 adds `--convert-to-gltf` to
`upgrade` (off by default):

```text
4cc-studio export-upgrader upgrade <export>... [--savefile <EDIT>] [--loose] [--coverage full|midcup] [--convert-to-gltf] [-o <dir>]
```

Three additions to the built commands:

- **The coverage tag is reported, and can be set.** Every `upgrade` of a team export reports
  the tag it used, an Info `coverage_tag_chosen` with the `tag` and its `source`, the first of
  these that holds: `name` when the old name's second word is already `Full` or `Midcup` in
  any letter case (below); `flag`; `midcup` or `additions`, the text the name was found to
  hold, a substring as the built rule matches it, the first in the name if both; `default`
  when it holds neither. `--coverage full|midcup` sets the tag in place of the name's guess,
  which stays the default. A manager upgrading an old mid-cup export named neither way ("co
  day 5 fix") gets `Full`, and its first compile rebuilds the team from the export alone,
  everything it does not carry becoming the default or a placeholder; the Info says so before
  that compile. A referee export takes no tag and refuses the flag (exit 2). An old
  name whose second word is already `Full` or `Midcup` keeps it (source `name`), and a flag
  naming the other tag is refused (exit 2): the member wrote that word, and renaming it under
  them would surprise more than the refusal.
- **Several exports at once.** `upgrade` takes one or more exports, with one `--savefile` or
  `--loose` for all of them: each is upgraded as if run alone (its own verdict, its output
  written or not; with more than one export, its report block under a line naming it, so a
  one-export run prints as today), every output inside `-o` or beside
  its input, and the exit code is the highest any export's run gave. An argument error that
  is the same for all (a flag, a savefile that does not read as a save) refuses the whole run
  before any export is read; a missing export path, a save lacking one export's team, or a
  flag that export's kind refuses (`--savefile`, `--loose` or `--coverage` given with a
  referee export among team exports) refuses that export alone, its block saying so. For an organizer moving a cup's library
  (about 50 teams) to Studio once per cup; the launcher keeps calling one export at a time,
  since its summary line counts them. `upgrade-models` stays one folder.
- **A leftover staging folder is cleared.** A `<output>.upgrading` folder left by an earlier
  run (a write that failed, a rename or removal the verdict could not finish) is removed by the next run of the same output, reported as an Info
  `staging_folder_removed` naming it, instead of aborting the run (exit 3): a staging folder is
  never a finished output, and the member who meets it never made it. A finished output folder
  is still never replaced. A removal that fails (a file of it held open: on Windows another
  run writing it at that moment) aborts as before (exit 3), naming the folder; a second
  upgrade of the same export started while the first writes, the one way to meet a live
  staging folder, then loses at most that run, never a finished output.

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
  Exactly one of the two is taken, so Upgrade is enabled once either is set; loose switched on
  disables the savefile field, its value kept for when loose goes off and not passed, rather than
  clearing it, since a member trying both ways would pick the save again. The field follows the
  selected PES version's discovered save until the member edits or picks one. Both are disabled
  for a referee export (its name's first word `refs`), which takes neither, so for it Upgrade is
  enabled as soon as the target is filled, and likewise while the second target holds the input. An **Output into**
  field holds the directory the output folder is written inside (`-o`): the input's parent by
  default, so the output lands beside the input, as the CLI's default. For the second target
  the same field is the output directory itself (`upgrade-models` writes its folders straight
  into its `-o`), `<input> upgraded` beside the input by default, as the CLI's. A **Full /
  Midcup** pair (`--coverage`) is prefilled with the name's guess once a team export is
  dropped, its tooltip naming what decided it (the `source` above); disabled for a referee
  export and for the second target, and held on the name's tag when the name already carries
  one (the CLI refuses the other there). The view passes `--coverage` only when the pair
  differs from the name's guess, so the report's `source` is the name's whenever the member
  left the pair alone.
- **The savefile field names the team it needs.** Once an export is dropped, its team is
  resolved (the Note's `Team:` line, else the name, step 6) and the field shows
  `needs /egg/ (792)` beside it; a save picked or discovered that lacks the team turns the
  field red, the refusal's sentence as its tooltip, and Upgrade stays disabled, so a manager
  whose save comes from another cup or version learns it before the run rather than from its
  exit 2. A team that resolves to no row of the list shows `team unknown: <name>` instead and
  leaves Upgrade enabled, the run then reporting `team_name_unknown` as the CLI does; a picked
  file that does not read as a save turns the field red with the read error as its tooltip,
  Upgrade disabled. An export whose Note or archive does not read at the drop leaves the field
  plain: the run reports it (`source_read_failed`, `export_extract_failed`) as the CLI does.
  The resolution and the save read are the run's own steps, moved to the drop
  and the pick; the run still checks both.
- **Upgrading without the save asks first.** With loose on, Upgrade opens a dialog before the
  run: the save is where the players' looks live (the appearance step 8 reads into
  `settings.toml`, and the in-game faces), the old export holds none of it, so an upgrade
  without the save carries none, and a compile of the output then gives the players it holds
  the default look. The dialog says that and the remedy (cancel, and pick the save the team
  was made in), and its Upgrade button stays disabled until the member ticks `I understand
  the players' looks will be lost`; Cancel returns to the view, loose still on. Asked on every
  loose run rather than once, since each export's loss is its own. The CLI asks nothing:
  `--loose` is already the opt-in step 11 requires, and a prompt would stall a script.
- **The Upgrade button** runs the command on a thread of its own through the CLI's own parser,
  as the Team compiler's Compile does (`spec/core/development_plan.md` "Phase 3", the shell
  slice), so the view cannot upgrade anything the command line would refuse; a refusal (a
  savefile without the team, exit 2) and an abort (an output folder that already exists, exit
  3) show as the CLI prints them. The existing-output abort's done line carries **Open the
  existing folder**, so a manager re-running an upgrade sees what the earlier run made and can
  rename or remove it; the command returns the folder's path with that abort (a field of the
  error, not parsed back from its text, which a Windows path's `:` would break). An abort over
  a file that could not be written carries none: its folder is the incomplete staging one.
- **The report** is the view's product: step 10's findings as the console prints them, in the
  log panel, and above it the **left-behind** list, one row per `unmigrated_content` or
  `leftover_dropped` entry with its input path, the finding's code and the entry's context (a
  Note line, a kit config's error), so the author sees in one place what to fix in the input
  (an unmigrated entry, which kept anything from being written) or what the output left out (a
  leftover). Entries are grouped by their parent folder: a parent holding two or more is one
  collapsing row with the count (`Boots/k2177 - THIS BEAVER IS CONTAMINATED: 18 files left
  out`), its chevron opening it to its entries, and a parent holding one stays that one row;
  an entry that is itself a folder (a second face folder) groups under its parent like a
  file. A real export's upgrade (Lab's EGG, in the draft) left 120 rows, 18 of them one boots
  folder's, which then read as the 25 folders they are about. A click on an entry's row (not
  the group's chevron) opens the system file browser at that entry (the folder, the file
  selected where the platform can; for an archive input, the folder the archive sits in, the
  archive selected), its tooltip saying so,
  so the fix starts from the list rather than a walk through the folders. For `upgrade-models` the per-folder "where it
  goes" lines are the log's, as the CLI prints them, and the list holds the folders it left
  out. A **Copy report** button beside the done line puts the console's lines on the clipboard,
  as the CLI prints them, for a manager asking for help in the cup's thread. The run ends with the exit code's meaning in words (clean; not clean, nothing written,
  see the list; refused; aborted). A clean run, the only one that writes output, also gets the
  notice `Upgraded <name> · [Open output folder]` in the status bar and, in the view's done
  line, a **Move to exports and compile** button: the output is moved into the exports folder
  and the Team compiler made active (`ctx.switch_to_tool`, as the Refs arranger hands off),
  the watcher then checking it like any dropped export. It is a button rather than the
  notice's action because a notice carries at most one (`spec/core/gui.md` "Status bar"), and
  Open output folder stays useful to a member who compiles elsewhere. Once clicked, the done
  line says where the output moved to and the button goes. A clean `upgrade-models` run gets
  the notice and no Move button: its output is folders to drop into an export, not an export.
  Any other run publishes no notice, having nothing to open. A new input clears the previous
  run's report, the log and the list included: the report is about the input it ran on.
- No settings section: the tool has no settings (step 6: the teams list path is the compiler's).

## Message catalog

What exists of this is in `spec/export_upgrader.md` "Message catalog". The additions above
bring two codes and two context fields:

| Code | Sev | When | Context |
|---|---|---|---|
| `coverage_tag_chosen` | I | `upgrade` of a team export: the coverage tag the output's name took ("CLI") | `tag`, `source` |
| `staging_folder_removed` | I | an earlier run's staging folder for this output removed before the write ("CLI") | `path` |

`unmigrated_content` gains `reason` on every entry and `other` where two candidates claim one
place ("What it does").

## Dependencies

What exists of this is in `spec/export_upgrader.md` "Dependencies". The optional glTF
conversion (step 13) uses `model_convert`'s `ir_to_gltf` path, which makes `model_convert` a
build dependency; the runtime `--convert-to-gltf` flag only determines whether that path is
executed.

## Acceptance

What exists of this is in `spec/export_upgrader.md` "Acceptance", whose rules (format, the
defaults a scenario naming no version or export takes, the test data) the scenarios below
follow. Step 13's scenarios are written with Phase 7.

**The CLI additions** ("What it does"'s last paragraph, "CLI"). The clause of EU-CLI-05 on a
staging folder left by an earlier run, which aborted, is replaced by EU-CLI-10.

```
EU-REP-05  GIVEN EU-PLY-10's export: face folders "XXX23 - Bahsaka" and "XXX23 - Smug
                 Redman", boots folders "k0430 - A" and "k0430 - B", and the misnamed
                 "XXX24 - Ann" and "K0431 - Rex"
           WHEN it is upgraded with its save
           THEN each unmigrated_content line carries its reason: the second face folder
                slot_taken with `other` naming "Faces/XXX23 - Bahsaka", the second boots folder
                id_taken with `other` "Boots/k0430 - A", the two misnamed slot_invalid and
                id_invalid with no `other`; every reason value the crate emits is reached by
                a test
EU-CLI-09  GIVEN EU-CLI-01's exports and EU-REF-01's referee export
           WHEN each is upgraded
           THEN each team export's report holds coverage_tag_chosen with its tag and source
                (`co - Spring`: Full, default; `co Day 8 Additions`: Midcup, additions; `co
                midcup Day 5`: Midcup, name; `co Full Spring`: Full, name) and the referee
                export's none; WHEN `co - Spring` is upgraded with --coverage midcup THEN the
                output is `co Midcup - Spring`, source flag; WHEN `co Full Spring` is upgraded
                with --coverage midcup THEN exit 2 and nothing written; GIVEN the referee
                export and --coverage, or an unknown value, THEN exit 2
EU-CLI-10  GIVEN `co - Spring` and its output's staging folder `co Full - Spring.upgrading`
                 holding a file, left by an earlier run
           WHEN it is upgraded
           THEN the staging folder is removed, staging_folder_removed names it, and the run is
                clean with its output written; GIVEN (Windows, where an open file blocks its
                removal) a file of the staging folder held open THEN exit 3 naming the
                folder, nothing written
EU-CLI-11  GIVEN two of EU-CLI-01's /co/ exports, a third export of another team, and a save
                 holding /co/ but not that team
           WHEN `upgrade` names all three with that save
           THEN the two are written, each block under a line naming its export, the third's
                block refuses it for its team, and the exit code is 2; WHEN one export is named
                alone THEN no naming line is printed; GIVEN an unknown flag THEN exit 2 before
                any export is read
```

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
                on and Upgrade clicked THEN the loose dialog opens and no run starts, its
                Upgrade disabled until its box is ticked; WHEN Cancel is clicked THEN the view
                is back, loose still on; WHEN the box is ticked and the dialog's Upgrade
                clicked THEN the run is `upgrade --loose` with no savefile argument, its output
                the one `upgrade --loose` writes for that export
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
