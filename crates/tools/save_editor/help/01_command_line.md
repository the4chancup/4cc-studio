# Running from a terminal

The Save editor's batch commands run from a terminal opened in the folder that holds
`4cc-studio`. Every command names the save it reads (the `EDIT00000000` file of your PES
install's save folder, or a copy of it); none picks one for you.

```text
4cc-studio save-editor export-toml <save> --team <id> --out <file>
4cc-studio save-editor export-toml <save> --all-teams --out-dir <folder>
4cc-studio save-editor import-toml <team.toml> --base <save> --out <save>
4cc-studio save-editor apply-patch <aesthetics_patch.toml> --base <save> --out <save>
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

Each command ends with an exit code scripts can test: 0 when it did its work, 1 when it did
its work but reported an error, 2 when it was refused before writing anything (a bad argument,
a file it cannot read, a team it cannot find), 3 when a write failed (what was written before
it stands).
