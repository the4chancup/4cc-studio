# Running from a terminal

The Export upgrader migrates an export in the old (AET) layout to the Studio
format the Team compiler reads. Its commands run from a terminal opened in the
folder that holds `4cc-studio`.

```text
4cc-studio export-upgrader upgrade <export> (--savefile <EDIT> | --loose) [-o <dir>]
```

`upgrade` takes one old export: a folder, a `.zip` or a `.7z`. A team export
takes `--savefile`, naming the save (`EDIT00000000`) whose records say which
player wears which boots and gloves, or `--loose` to infer the wearers from the
IDs alone; a referee export (a name whose first word is `refs`) takes neither.

The output is a new folder beside the input, or inside the folder `-o` names
(created when missing). Its name is the team's key with the coverage tag
inserted (`co - Spring` becomes `co Full - Spring`, an export named for
additions gets `Midcup`), and when that name is already the input's own,
` upgraded` is appended so the original is never written over. An output folder
that already exists is refused and left untouched.

A referee export is upgraded from its `refs.txt`: the output gets a
`players.txt` holding the same slots in the Studio spelling (two-digit slot
numbers, one entry per line) and no `refs.txt`, and its `ref_marker.dds` is
carried as it is. A referee export without a `refs.txt` has no roster to
migrate: it is reported (`refs_txt_missing`) and nothing is written.

The original export is never modified. A file the migration cannot place yet
is listed in the report as `unmigrated_content`, and the upgraded folder is
checked against the same rules the Team compiler applies: every finding prints
as one line naming its code and the input path it is about, ending with
`migration_clean` or `migration_not_clean` (the output is a draft for manual
repair).

The exit code: 0 when the migration is clean; 1 when it is not clean: an
error, or a finding that drops something (a warning that drops a folder means
the compile would lose it); 2 when the command was refused before anything
was written (a bad argument, an export that is not there, a savefile that
does not read as a save or does not hold the team); 3 when the run aborted:
before anything is written (the output folder exists or cannot be created),
or when a file of the output cannot be written, the incomplete output left.
