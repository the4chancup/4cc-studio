# Proposals for the maintainer

Ideas nobody asked for that would make the suite easier to use: a button, a shortcut, a
default, a message that saves a member a step. Any agent adds one when it thinks of it while
working (`AGENTS.md` "When the plan has gaps"); the maintainer vetoes or accepts entries as
they come across them. A proposal is not built while it is unruled: plan text that already
describes one says so and points here, and the phase that would build it lists the entry in
its turn report. A vetoed entry leaves with a one-line `DECISIONS.md` entry, so it is not
proposed again; an accepted one leaves into its plan section and a `DECISIONS.md` entry.

Each entry: a heading, where it came from, what it adds and for whom, and what it costs (code,
surface, things a member must learn). Newest last.

The first idea of this kind, the Export upgrader's "Move to exports and compile" button, was
asked in `QUESTIONS.md` before this file existed and approved.

The entries P16 to P22 come from the usability pass before the tools' drafts (worklog 8.4,
2026-10-20; P1-P15 of the same pass were accepted on 2026-10-25 and moved into their plans,
`DECISIONS.md`'s entries "Usability proposals P1-P14 accepted" and "Edits made in PES come
back to the export"): what members already touch (the export's folder conventions, the
compiler's and upgrader's findings, the defaults, the upgrade, compile and install handoffs), read for steps
a member could be spared. Managers (one or a few per team, about 50 teams a cup) lay out
exports by hand and compile their own team; organizers (a handful) run every export, mostly
through the CLI. Each entry says which it helps.

## P16. Import Red's settings on first run

From `spec/team_compiler/settings.md`: the settings are "inherited from Red/Blue
(`settings.ini`, `settings_default.ini`)", and nothing reads those files. **Adds**, for every
member coming from Red, once: an "I used Red" row in the first-run dialog picking the Red folder
and carrying the keys that map one to one (`pes_version`, `pes_folder_path`, `cpk_name`,
`pass_through`, ...). **Costs**: an INI read of a fixed key list and a mapping table. It reads a
member's own settings; "legacy tools are evidence, not source" is about how code is written.

## P17. `unmigrated_content` says which rule refused and what it collided with

From `spec/export_upgrader.md` step 10: one code covers about fifteen causes (a second face
folder for a slot, a second item folder for an ID, two kit textures landing on one name, a face
folder without a valid slot, a texture that does not decode, a folder loose mode cannot place,
...), and a refused folder is reported with no context (`players.rs`, `refused`). **Adds**, for
every manager whose upgrade is not clean (the one path where nothing is written and the input
must be fixed): a `reason` field (a closed set, one value per cause) and, for two candidates
for one place, the other candidate's path, so the line can be acted on without the spec.
**Costs**: an enum, a field at each of about fifteen emission sites, the expected lines.

## P18. The upgrader says which coverage tag it chose

From `spec/export_upgrader.md` "CLI": the output is `Midcup` when the old name holds `midcup`
or `additions`, else `Full`, and the report says nothing of it. **Adds**, for managers upgrading
an old mid-cup export named neither way ("co day 5 fix", which becomes `Full` and, compiled,
replaces the team's kits with placeholders): an Info naming the tag and why, and in the GUI a
Full/Midcup choice prefilled with the guess (`--coverage full|midcup` on the CLI). The guessing
rule stays the default. **Costs**: one code, a flag, a radio pair. How many old mid-cup exports
carry neither word is not measured.

## P19. Loose mode says the players' looks are not carried

From `spec/export_upgrader.md` step 11: in loose mode no `settings.toml` aesthetic data is
generated, and no finding says so; on Fox a `Full` compile then writes every player's
appearance from the defaults. **Adds**, for managers upgrading without a save: one Warning
(kept, so the run stays clean) naming the consequence and the remedy (upgrade with
`--savefile`), and the same sentence as the loose switch's tooltip. **Costs**: one code, one
tooltip.

## P20. Upgrade several exports at once

From `spec/export_upgrader.md` "CLI" (`upgrade` takes one export) and
`plans/export_upgrader.md` "GUI" ("The view holds one input"). **Adds**, for an organizer moving
a cup's library (about 50 teams) to Studio, once per cup: `upgrade <export>...` with one save,
one report block and one verdict per export, each written or not on its own (exit code the
worst export's). **Costs**: a loop and the report's grouping.

## P21. A leftover staging folder is cleared by the next run

From `spec/export_upgrader.md` "CLI": after a failed write "the incomplete staging folder [is]
left for the user to remove, since a second run refuses an existing one". **Adds**, for the rare
member whose write failed, who then meets exit 3 naming a folder they never made: a
`<name>.upgrading` folder is never a finished output, so the next run removes it and says so.
The finished output folder is still never replaced. **Costs**: one removal, one Info.

## P22. Findings carry the context their text will need

From the catalogs (no message text is written yet; the console prints the code and its
context). Codes whose context leaves the member unable to act, for the texts Phase 8 writes:
`players_txt_invalid` and `notes_encoding_invalid` (which cause, the first bad line, and how to
re-save as UTF-8 in Notepad); `player_folder_number_invalid` (the expected `NN - Name`, or
`players.txt`); `player_number_duplicate` (the other folder claiming the slot);
`kit_folder_invalid` (the allowed `p1`-`p9`, `g1`, `all`); `file_type_disallowed` (unknown
extension, hidden `.txt`, or a known file in the wrong place); `skl_merge_conflict`,
`merge_material_conflict`, `model_merge_flags_conflict` (which two parts disagree);
`model_conversion_failed` (that the converted model hits the limit, not the member's file, and
the remedy); `texture_type_mismatch` (what format the bytes are); `custom_model_missing` and
`settings_not_generated` (the ways to a clean upgrade); `export_tag_missing` (the two exact
names to rename to); `check-aatf`'s `aatf_violation` (the player's name and number, not his
id). **Adds**, for every member reading a finding: a text that says the fix. **Costs**: a context
field per code and its tests, mechanical.

## P23. A click on a swatch closes the color picker

From `plans/libs/color_tools.md` "Color-picker widget (Phase 8)" and the built picker
(`color_tools/src/picker.rs`, a popup with `PopupCloseBehavior::CloseOnClickOutside`, used by
the Save editor's team bar for the team's two colors): a click on a suggestion or recent swatch
commits the color and leaves the popup open, to be closed by Escape or a click outside. **Adds**, for a
member reusing a color (a team's two colors matched to an earlier pick), the common case of the
swatch rows: one click instead of two. The square, the hex and R/G/B fields keep it open, being
the fine-tuning path. **Costs**: one close call, one draft check.

## P24. Studio reopens at the size and place it was closed at

From `plans/core/gui.md` "Shell layout", "The window's size" (2026-10-21): the window opens at
1280 × 800 points (fitted to a smaller monitor), centered, every start. **Adds**, for a member who resized or moved
it (a second monitor, a window beside the game): it comes back where they left it, as most
desktop programs do; the default stays for the first start and for a window
whose saved place is off every monitor now. **Costs**: eframe's window persistence (its
`persistence` feature, which writes its own small file and adds `ron` to the build; `serde`
is in the workspace already), or two settings the shell writes on close; one manual check.

## P25. Browse on the PES folder row keeps the version placeholder

From `plans/core/gui.md` "Settings menu" (every path row's Browse, 8.9j1) and the
`pes_folder_path` setting, whose `**` stands for the selected version's two digits (the
default `…\Pro Evolution Soccer 20**`): Browse stores the folder picked, so a member who picks
`…\Pro Evolution Soccer 2019` and later selects PES 21 keeps the 2019 folder, and the exe check
reports it. **Adds**, for a member with several versions installed side by side: a picked folder
whose name ends in the selected version's year is stored with `**` in its place, so the version
selector keeps switching folders as it does for the default. **Costs**: one rule in the row's
pick (PES folder row only), its test, and a sentence in the row's tooltip.

## P32. A left-behind row opens the old export at that file

P3's Open folder (accepted for the Team compiler's findings), for the upgrader. From the Export
upgrader's draft (`docs/prototypes/tools/export_upgrader.js`) and
`plans/export_upgrader.md` "GUI": after a run that is not clean, the left-behind list names
each entry to fix by its path inside the old export, and nothing in the view goes there.
**Adds**, for a manager whose upgrade is not clean (the one outcome where the input must be
edited): a click on a row opens the file browser at that entry (the folder, the file selected
where the platform can), so the fix starts from the list instead of a walk through the
folders. For an archive input it opens the archive's folder. **Costs**: one row action
through the platform's file browser (the shell already opens the output folder), a tooltip,
one draft line.

## P33. The savefile field says which team the save must hold

From `spec/export_upgrader.md` "CLI" (a savefile that does not hold the resolved team is
refused, exit 2) and the draft's savefile field: the member learns the save is the wrong one
only after Upgrade. **Adds**, for a manager whose save comes from another cup or version: when
an export is dropped its team is resolved (the Note's `Team:` line, else the name), the field
shows `needs /egg/ (792)` beside it, and a picked save that lacks the team turns the field red
with the refusal's sentence as its tooltip, before Upgrade. **Costs**: the team resolution and
a save read moved to the drop and the pick (both are steps the run does anyway), one line of
state, one more GUI scenario.

## P34. An existing output folder can be opened from the abort

From `spec/export_upgrader.md` "CLI": an output folder that already exists aborts the run (exit
3), naming it, so edits made since an earlier upgrade are never lost. The member is left to
find that folder by its path. **Adds**, for a manager re-running an upgrade (after fixing the
input, or by mistake): the aborted done line carries an **Open the existing folder** button,
so they can see what they made last time and rename or remove it. **Costs**: one button on one
outcome, the path the abort already holds.

## P35. Copy the report

From the draft's report (the log panel and the left-behind list): a manager asking for help
in the cup's thread retypes or screenshots the lines. **Adds**, for managers and the
organizers who answer them: a **Copy report** button beside the done line, putting the
console's lines (the ones the CLI prints, so they read the same in both) on the clipboard.
**Costs**: one button, egui's clipboard output; nothing new to learn.

## P36. The left-behind list groups a folder's rows

From the draft's EGG sample (`docs/prototypes/tools/export_upgrader.js`, a real export's
upgrade with its save): 120 `leftover_dropped` rows, 18 of them one boots folder's files
(`Boots/k2177 - THIS BEAVER IS CONTAMINATED/...`), in a list 160 points tall. **Adds**, for a
manager reading what an upgrade left out: one collapsed row per folder with its count (`Boots/k2177
- THIS BEAVER IS CONTAMINATED: 18 files left out`), opened to its files, so the list reads as
the 25 folders it is about rather than the 120 files. A folder holding one entry stays one row.
**Costs**: a grouping by the path's folder, a collapsing header per group, one draft change.
