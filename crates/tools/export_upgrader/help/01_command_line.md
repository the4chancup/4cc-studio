# Running from a terminal

The Export upgrader migrates an export in the old (AET) layout to the Studio
format the Team compiler reads. Its commands run from a terminal opened in the
folder that holds `4cc-studio`.

```text
4cc-studio export-upgrader upgrade <export> (--savefile <EDIT> | --loose) [-o <dir>]
4cc-studio export-upgrader upgrade-models <folder> [-o <dir>]
```

`upgrade` takes one old export: a folder, a `.zip` or a `.7z`. A team export
takes `--savefile`, naming the save (`EDIT00000000`) whose records say which
player wears which boots and gloves, or `--loose` to infer the wearers from the
IDs alone; a referee export (a name whose first word is `refs`) takes neither.

With `--loose` each boots or gloves folder goes to the player at its ID's
position in the team's old block of 25 IDs, the block its kind's lowest ID
falls in (`k0128` is player 03 of the 126 block). Each pairing is reported as
`wearer_assumed`, so it can be checked. An ID on the block's last two
positions or past it belongs to no player and is reported as unmigrated
content. Without a save no look is written: a player folder with a name gets a
`settings.toml` whose only setting is `name = true`.

The output is a new folder beside the input, or inside the folder `-o` names
(created when missing). Its name is the team's key with the coverage tag
inserted (`co - Spring` becomes `co Full - Spring`, an export named for
additions gets `Midcup`), and when that name is already the input's own,
` upgraded` is appended so the original is never written over. An output folder
that already exists is refused and left untouched.

A referee export is upgraded from its `refs.txt`: the output gets a
`players.txt` holding the same slots in the Studio spelling (two-digit slot
numbers, one entry per line) and no `refs.txt`, and its `ref_marker.dds` is
carried as it is. The roster may also be the older `Refs list.txt`, with
the referee folders standing directly in the export rather than under
`Players/`; the migration is the same. The old `face/`, `boots/`, `gloves/` and `common/`
subfolders of each referee's folder are folded into that folder, a file held
twice with the same bytes kept once. A file two of them hold with different
bytes, or a subfolder of one of those four names inside them, leaves the
referee's folder as it was, and it is reported. A model of `common/` that no
`face.xml` of the referee names is left out and reported. A referee export
without a `refs.txt` has no roster to migrate: it is reported
(`refs_txt_missing`) and nothing is written.

`upgrade-models` takes a folder of old model folders instead: the face, boots
and gloves folders lying directly in it, one level deep, plus an optional
`Common` folder. Each becomes one folder to drop into a Studio export. A face
folder becomes a player folder `NN - Name` (its number and name from its own
name, or its own name alone when it has no number), a boots or gloves folder
becomes a folder named by its name part (`k0430 - Crocs` becomes `Crocs`) or
by its ID when it has none, and `Common` is upgraded as an export's would be.
The report names each output folder with where it goes: a player folder drops
into `Players/`, `Common` at the export's root, and a boots or gloves folder
drops into `Boots/` or `Gloves/` with a link file added to the wearing
player's folder, or its files move into a player folder with the model
renamed `boots.fmdl` or `glove_*.fmdl`. A top-level folder holding no model
file directly in it is reported as unmigrated content and left out. No team
is resolved, so a `???` placeholder and the goalkeeper `u0???g` token stay as
written. The output is the folder `<folder> upgraded` beside the input, or
the folder `-o` names, which must not exist yet.

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
