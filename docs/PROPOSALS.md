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

The entries P1 to P22 come from the usability pass before the tools' drafts (worklog 8.4,
2026-10-20): what members already touch (the export's folder conventions, the compiler's and
upgrader's findings, the defaults, the upgrade, compile and install handoffs), read for steps
a member could be spared. Managers (one or a few per team, about 50 teams a cup) lay out
exports by hand and compile their own team; organizers (a handful) run every export, mostly
through the CLI. Each entry says which it helps.

## P1. Every finding line says what was lost

From `studio_core` `events.rs` `message_line`: a line is `<export>: <Severity> <code> at
<path> (<context>)`, and the finding's disposition is not printed. Two Warnings drop a whole
folder (`player_unlisted`, `shared_folder_orphaned`), so a manager reading only the Errors, or
skimming the yellow lines, never learns the folder is gone. **Adds**, for every member on every
run: a fixed phrase per disposition after the line (`folder left out`, `file left out`, `slot
skipped`, `export skipped`, `run stopped`, `kept anyway` when passed through), in the console
and the cell's tooltip, so no catalog text has to repeat it. **Costs**: one disposition-to-phrase
mapping in `studio_core`; the CLI tests that pin whole lines change.

## P2. A failed name lookup names the nearest existing name

From `link_target_missing`, `players_txt_target_missing`, `common_link_missing` and
`kit_folder_invalid` (`spec/team_compiler/messages.md` "Catalog"), and `file_type_disallowed`
on a misspelled marker (`fpc-on`, `ingameface`). **Adds**, for managers editing an export by
hand: a `closest` context field (the nearest name of the right kind, by edit distance over
case-folded names, under a small threshold), so the text can say "no shared boots folder
`Crocs`; `crocs new` exists". Nothing is resolved by the guess: the finding and its drop stay.
**Costs**: one distance function in `aesthetics_export`, a field on five codes. How often
members mistype is not measured: no Studio export has been written by hand yet.

## P3. Open the folder a finding is about

From `plans/team_compiler/gui.md` "Player cells" (a click opens a detail panel; only "Open
output folder" opens a file manager anywhere in the plans). **Adds**, for every manager fixing
a finding (the fix is always a rename, a link or a removal in the file manager): an **Open
folder** button in the detail panel and on a log line's context menu, opening the scope's
folder (a file's folder with the file selected; for an archive export, the archive's folder).
**Costs**: one platform call (`explorer /select,`, `xdg-open`) and two widgets.

## P4. One-click fixes for the mechanical cases

From the untagged row's **Full** / **Midcup** buttons, which rename the export
(`plans/team_compiler/gui.md` "Untagged export"). Cases with one obvious fix: a known file
with a hidden `.txt` (`settings.toml.txt`, kept disallowed "so the mistake is named"); an old
`Portraits/player_05.dds`; a player folder `15 Snuffy` or `15_Snuffy` for `15 - Snuffy`.
**Adds**, for managers: the rename on a click in the detail panel, the member still choosing.
**Costs**: per fix a rename with a collision check, a button and a scenario; archives get no
button. It widens the tag buttons' precedent of the GUI writing into an export.

## P5. An old export in the exports folder offers its upgrade

From `export_layout_old` and `player_layout_proto`, whose hint names the Export upgrader; the
planned handoff runs the other way only ("Move to exports and compile",
`plans/export_upgrader.md` "GUI"). **Adds**, for every manager once during the move to Studio
and every organizer receiving an old export later: an **Upgrade** button on that red row,
switching to the upgrader with the export as its input; after a clean upgrade, an offer
(asked, not silent) to move the old export into a sibling `old exports/` folder, so its red row
goes. **Costs**: a row button, a tool switch with an argument (the Refs arranger handoff is the
precedent), one confirmation.

## P6. A fresh install finds the PES version and folder

From `spec/team_compiler/settings.md` "Common settings": `pes_version` defaults to 19 and the
folder to `C:\Program Files (x86)\Pro Evolution Soccer 20**`; the plans fix the path only from
a running PES ("Fix PES path"). **Adds**, for every new member, before their first compile: the
first-run dialog offers the versions whose `Documents\KONAMI` save folders exist (the save
discovery already walks them, the newest a better default than 19) and the installs found by
probing the default path and Steam's library folders for `PES20xx.exe`. **Costs**: a probe and a
dialog row. The Steam folder names per version are not checked on a real install yet.

## P7. The empty grid says where exports go, and takes a drop

From `no_exports_found` ("what a fresh install's created `exports/` reports until the user
fills it"); the grid plan has no empty state. **Adds**, for new managers and for every manager
receiving an export from Discord: an empty grid naming the exports folder with an **Open
exports folder** button, and a grid accepting a folder, `.zip` or `.7z` dropped on it (copied
into the exports folder, refused when the name exists). **Costs**: an empty-state panel and a
drop handler.

## P8. Compile one export from its row

From the CLI's `--export <path>` (`spec/team_compiler/settings.md` "CLI"); the GUI's Compile
always takes the whole folder. **Adds**, for managers testing their team beside other exports
and organizers re-checking one team: **Compile only this export** in a row's context menu, in
the current output mode. **Costs**: a menu entry and the run's argument.

## P9. Show disabled exports greyed, and offer Disable where two exports clash

From `plans/team_compiler/pipeline.md` "1. Reader" ("A disabled export is omitted from the
grid") and `duplicate_aesthetics_export`/`multiple_ref_exports`, which skip every export
involved. **Adds**, for managers keeping an older copy beside the new one (both skipped today)
and organizers juggling refs exports: a **Disable** action on a clashing folder export (writing
its `NO_USE` marker; an archive is offered a move out instead), and disabled exports as greyed
rows with **Enable**, so what was disabled can be seen and undone. **Costs**: a greyed row, two
actions, the marker write. It changes the plan's "omitted from the grid".

## P10. A Fox model's missing texture shows before Compile

From `fmdl_texture_not_found`, "reported by `compile`, where the model is compiled"; the live
grid runs the deep pass, so the cell looks fully checked and turns red only at the compile.
**Adds**, for managers on Fox (a renamed or forgotten texture is the most common model
mistake): the deep pass runs the same lookup for a source model's own texture names, leaving
merged and retargeted cases to the task. **Costs**: the lookup shared by both callers, never
copied, and a test pair keeping the two in agreement.

## P11. A model whose type is not at the end of its name is flagged

From `spec/aesthetics_export/player_folders.md` "Model names": a name with no recognized suffix
is face content, reported as Info (`fmdl_fcl_hair_fallback`). **Adds**, for model makers:
`boots2.fmdl`, `boots - Copy.fmdl` or `glove_l (1).fmdl` merge into the hair today, and the game
shows boots skinned as hair; a Warning when an unsuffixed name holds a type word names the
intended rename (`boots2.fmdl` → `new_boots.fmdl`). The routing stays. **Costs**: a token check
over the suffix table, one code.

## P12. An invalid `settings.toml` keeps the player's installed look

From `settings_toml_invalid` ("file ignored; the player compiles with default settings").
**Adds**, for managers who mistype one value: on Fox his appearance row is written from the
defaults today, so his look resets in game while his models compile; keeping his installed row
instead, as a failed boots output keeps the installed ID, loses nothing he had. **Costs**: a
planning branch and a scenario. It contradicts `settings_toml.md`'s "his export and nothing
else" (which has two exceptions already).

## P13. A Full compile names the kits it is about to reset

From `kit_placeholder` (Info) and the coverage tag: a `Full` team export with no surviving kit
gets placeholder `p1` and `g1`, and "the reverse wipes a team's kits" (`object_model.md`
"Coverage tag"). **Adds**, for managers who forget `Kits/` or meant `Midcup`: a Warning when a
`Full` export has no `Kits/` folder at all, naming the two ways out (add the kits, or rename it
Midcup). A full-body team writes `Kits/p1/` itself, so it is not warned. **Costs**: one code.

## P14. The savefile is checked in the preflight

From `spec/team_compiler/pipeline.md` "Destination writability preflight", which checks the PES
folder before any export is read, while the savefile is read only at the end, where
`savefile_invalid` (unreadable, or another version's) rolls the whole deployment back.
**Adds**, for members with a save path of another version or a damaged save: the save's header
read in the preflight reports it before minutes of compiling. Rare with `savefile_path = auto`.
**Costs**: one read; the end-of-run read stays, since the file can change.

## P15. Compile findings open the Save editor where the fix is

From `savefile_player_not_stripped` (naming `strip-and-seed` and `check-stripped
--strip-again`), `savefile_missing` (naming `apply-patch`) and `savefile_skipped_pes_running`
("caught up on the next compile with PES closed"). **Adds**, for Fox managers iterating with
the game open, and members whose save was never stripped: an action on the finding opens the
Save editor on that save at the matching step; and when the PES poll sees the game close after
a skipped savefile step, a notice offers **Apply the pending settings** (the patch already
written) without a recompile. **Costs**: a tool switch with an argument (as in P5) and one
notice; the patch goes through the same apply, the compiler keeping one savefile write path.

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

From `plans/libs/color_tools.md` "Color-picker widget (Phase 8)" and the picker drawn in
`docs/prototypes/save_editor.html` (Team tab): a click on a suggestion or recent swatch commits
the color and leaves the popup open, to be closed by Escape or a click outside. **Adds**, for a
member reusing a color (a team's two colors matched to an earlier pick), the common case of the
swatch rows: one click instead of two. The square, the hex and R/G/B fields keep it open, being
the fine-tuning path. **Costs**: one close call, one draft check.

## P24. Studio reopens at the size and place it was closed at

From `plans/core/gui.md` "Shell layout", "The window's size" (2026-10-21): the window opens at
1440 × 900 points (fitted to a smaller monitor), centered, every start. **Adds**, for a member who resized or moved
it (a second monitor, a window beside the game): it comes back where they left it, as most
desktop programs do; the default stays for the first start and for a window
whose saved place is off every monitor now. **Costs**: eframe's window persistence (its
`persistence` feature, which writes its own small file and adds `ron` to the build; `serde`
is in the workspace already), or two settings the shell writes on close; one manual check.
