# Running from a terminal

The Save editor's batch commands run from a terminal opened in the folder that holds
`4cc-studio`. Every command names the save it reads (the `EDIT00000000` file of your PES
install's save folder, or a copy of it); none picks one for you.

```text
4cc-studio save-editor export-toml <save> --team <id> --out <file>
4cc-studio save-editor export-toml <save> --all-teams --out-dir <folder>
```

`export-toml` writes a team of the save as a Team TOML file: the team's names, colors, manager
and stadium, its tactics, and every rostered player with every field, the look included (a
player whose look the save does not hold, on a stripped save, carries `stripped = true`). With
`--team` one team goes to the file `--out` names, replacing it when it exists; with
`--all-teams` every team of the save goes to its own `<id>.toml` in the `--out-dir` folder,
which is created when missing, its older files replaced. A team id the save does not hold is
refused and nothing is written.

Each command ends with an exit code scripts can test: 0 when it did its work, 1 when it did
its work but reported an error, 2 when it was refused before writing anything (a bad argument,
a file it cannot read, a team it cannot find), 3 when a write failed (what was written before
it stands).
