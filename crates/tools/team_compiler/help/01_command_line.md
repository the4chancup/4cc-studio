# Running from a terminal

The Team compiler also runs without its window, from a terminal opened in the folder that holds
`4cc-studio`:

```text
4cc-studio team-compiler check [exports-root] [--export <path>]...
4cc-studio team-compiler compile [exports-root] [--mode normal|test|sider] [--export <path>]... [--no-deploy]
```

`check` reads your exports and reports what it finds, without writing anything. `compile` reads
them and builds the CPK.

`check` prints one line per finding: the export it is about, how serious it is, its code, where
in the export it is, and its details in parentheses. The line `Info export_identified (team=/co/,
id=701)` tells you which team the export was recognized as: its name's first word, looked up in
the teams list (`team=referees` for a `refs` export).

Both commands read every export in the exports folder from the settings (`exports/` beside
`4cc-studio` unless you changed it). To use another folder for one run, give its path as
`exports-root`; the setting is not changed.

An export can be a folder, a `.zip` or a `.7z`, read where it is: nothing is extracted. An archive
that cannot be read (damaged, password-protected, or holding two files whose names differ only in
case) is reported as `export_extract_failed` and left out of the run.

`--export <path>` limits the run to one export: a folder, a `.zip` or a `.7z`, which does not
have to be inside the exports folder. Repeat it to name several exports. A path that does not
exist, or a file that is not a `.zip` or `.7z`, stops the command before anything runs.

`--no-deploy` builds the CPK into the output folder without installing it into the game. It
cannot be combined with `--mode test` or `--mode sider`.

In this version `compile` refuses `--mode test` and `--mode sider`, and refuses to run while the
`multicpk_mode` setting is on. Use the normal mode with `multicpk_mode` off.

A relative path typed in the terminal is taken from the folder the terminal is in.

When a command ends, its exit code tells a script how it went:

| Code | Meaning |
|---|---|
| 0 | Finished cleanly. Warnings and notes may have been reported. |
| 1 | Finished, but some export had an error: something was left out, or kept because `pass_through` is on. |
| 2 | The command line or a setting is wrong, for example a `cpk_name` that is not a valid file name. Nothing ran. |
| 3 | The run stopped early, for example because the teams list cannot be read or the output folder cannot be written. |
