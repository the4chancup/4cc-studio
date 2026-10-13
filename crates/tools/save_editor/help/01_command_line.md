# Running from a terminal

The Save editor's batch commands run from a terminal opened in the folder that holds
`4cc-studio`. Every command names the save it reads (the `EDIT00000000` file of your PES
install's save folder, or a copy of it); none picks one for you.

```text
4cc-studio save-editor export-toml <save> --team <id> --out <file>
4cc-studio save-editor export-toml <save> --all-teams --out-dir <folder>
4cc-studio save-editor import-toml <team.toml> --base <save> --out <save>
4cc-studio save-editor apply-patch <aesthetics_patch.toml> --base <save> --out <save>
4cc-studio save-editor transplant <donor save> --base <save> --out <save> --select <list>
4cc-studio save-editor diff <first save> <second save> [--aesthetics | --gameplay]
4cc-studio save-editor export-teams-list <save> [--yes]
```

A command that writes a save reads the one `--base` names and writes the one `--out` names,
which may be the same file; a file already at `--out` is first copied to `<out>.bak` (one
level: the next write replaces that copy). No command creates a save from nothing.

`export-toml` writes a team of the save as a Team TOML file: the team's names, colors, manager
and stadium, its tactics, and every rostered player with every field, the look included (a
player whose look the save does not hold, on a stripped save, carries `stripped = true`). With
`--team` one team goes to the file `--out` names, replacing it when it exists; with
`--all-teams` every team of the save goes to its own `<id>.toml` in the `--out-dir` folder,
which is created when missing, its older files replaced. A team id the save does not hold is
refused and nothing is written.

`import-toml` writes a Team TOML file onto the team its `[team] id` names. The file's keys
are what it writes: a key, a player or a whole section left out of the file is left as the
save has it, so a file holding only `[team] id` and `[players.03] name = "Snuffy"` renames
one player and nothing else. A file written from another PES version is converted on the
way in, and each value that could not be taken as written is one line: a key the save's
version has no field for (`import_key_not_in_version`), a value it cannot store
(`import_value_not_encodable`), a face type or skin above its range, written as 0
(`import_value_capped`), a player table for an empty roster slot (`import_slot_empty`).
Those lines do not change the exit code. A file with no `[team] id`, one naming a team the
save does not hold, or one with a name longer than the version's field is refused, naming
the team or the player, and nothing is written. The file never sets a player's look
on or off: on a stripped save his look stays the database's.

`apply-patch` applies the `aesthetics_patch.toml` the Team compiler writes beside its CPKs:
the compiled players' motions and, for teams compiled on autopilot, their names. The patch
must be for the save's PES version and the suite's boots and gloves allocation scheme, or it
is refused with nothing written. Each team of the patch is one line, `patch_team_applied`;
a team or a player the save does not hold is the error line `patch_team_not_in_save` or
`patch_player_not_in_save`, the rest of the patch is applied and written, and the exit code
is 1.

`transplant` copies players' looks from a donor save of the same PES version onto the
`--base` save: the whole look (boots, gloves, base copy, physique, strip settings, the
in-game face) and the face, hair, physique and strip edit flags, nothing else. `--select`
says who takes whose look, comma-separated: a player id alone (`70105`) takes his own look
from the donor, `70107:70207` gives 70107 the look of the donor's 70207, a team id (`703`)
does that for its 23 slots, a range (`704-705`) for every team in it. The saves must be of
one version, every target in the base and every donor in the donor save, or the command is
refused and nothing is written. On a stripped save a player's record is not the look the
game shows, so a target at -1 gets the warning line `transplant_target_stripped` (the game
does not read what was written into his record) and a donor at -1
`transplant_donor_stripped` (his record was copied, not the look the game shows for him).

`diff` lists what differs between two saves of one version, player by player, paired by
player id: one line per changed field, `player_diff`, with the field, the first save's
value and the second's; a player in one save alone is `player_only_in`; a player whose look
is stripped (appearance id -1) in either save is marked `player_stripped`, and the
appearance id itself is never a line. `--aesthetics` keeps the look's fields alone (boots,
gloves, base copy, physique, strip, face), `--gameplay` everything else (names, basics,
abilities, positions, skills, motion). Differences are the output, not errors: the exit
code is 0. Two saves of different versions are refused.

`export-teams-list` merges the save's teams into the `teams_list.txt` of the data folder
(the list the Team compiler resolves export names against; the list shipped with 4cc Studio
when there is no file yet): every team whose name is slash-wrapped (`/co/`) and whose id is
in the cup range, 701 to 920. It prints what the merge does, one line each: a team the
list lacked (`teams_list_added`), a name listed under another id, which takes the save's
(`teams_list_overridden`), a change dropped because two teams would share one id
(`teams_list_unresolved`, an error line), a team left out because an earlier team of the
save has the same name, upper and lower case aside (`teams_list_name_duplicate`, an error
line naming the team kept: fix the save), and how many rows stay as they were
(`teams_list_kept`). Without `--yes` it stops there (`teams_list_not_written`); with
`--yes` it writes the merged list (`teams_list_written`), the dropped changes left out. A
data folder that cannot take the file, or none chosen yet, is the error line
`teams_list_read_only` and nothing is written. The exit code is 1 when an error line was
printed, 0 otherwise.

Each command ends with an exit code scripts can test: 0 when it did its work, 1 when it did
its work but reported an error, 2 when it was refused before writing anything (a bad argument,
a file it cannot read, a team it cannot find), 3 when a write failed (what was written before
it stands).
