# Running from a terminal

The Team compiler also runs without its window, from a terminal opened in the folder that holds
`4cc-studio`:

```text
4cc-studio team-compiler check [exports-root] [--export <path>]...
4cc-studio team-compiler compile [exports-root] [--mode normal|test|sideload] [--export <path>]... [--no-deploy]
```

`check` reads your exports and reports what it finds, without writing anything. `compile` reads
them, reports the same findings, and builds the CPK named by the `cpk_name` setting in the output
folder (`output/` beside `4cc-studio` unless you changed the `output_folder_path` setting), as
`<cpk_name>.cpk`. The CPK is written in full before it replaces the previous one, so
`<cpk_name>.cpk` is never half-written. A run that finds nothing to compile writes nothing and leaves the
previous CPK as it was. In this version `compile` builds exports for PES 2018 to 2021 that hold
only the models of the players in the roster, kept in each player's own folder: face models,
boots (a model whose name ends in `boots`, such as `kit_boots.fmdl`) and gloves (`glove_l`,
`glove_r`, or `handL`, `handR`), with their textures, their portraits as `.dds` files (a
`portrait.dds` in the player's folder, or `player_NN.dds` in the `Portraits` folder, not both
for one player), and kits. A texture (a player's, a shared folder's, a Common one or a kit's)
can be a `.dds`, `.ftex`, `.png`, `.jpg`, `.bmp`, `.webp`, `.tga` or `.tif` file: each is
converted to what the chosen PES version reads, BC7 for PES 2019 to 2021 and BC3 for PES 2018
(BC1 when the image has no transparency at all; a `.dds` or `.ftex` already in a format the
version reads is kept as it is), and an image
without the smaller copies of itself the game needs (its mip levels) gets them generated. A
texture whose name ends in `_nrm` (`skin_nrm.png`) is a normal map and is encoded as one. A
texture that cannot be read is reported as `folder_pack_failed`, naming the file, and its
folder is left out. The boots and gloves get the ID reserved for the player's roster
slot, the same number for both. A shared `Boots` or `Gloves` folder that players point at with
a link file (an empty `Crocs.boots` in the player's folder names `Boots/Crocs`) is built once,
with its own textures, under one of the 17 IDs the team keeps for shared folders, given out in
the folders' name order (`Apple` before `Zebra`), and a player linking it gets no boots or gloves
of their own. A player folder holding both a link file and a boots (or gloves) model of its own
combines the two: the shared folder's models are merged into the player's own boots (or gloves)
under the player's ID, its textures travel with them, and the line `link_combined` says so; the
shared folder is still built on its own for any player linking it plainly. A link to a shared
`Faces` folder (`Longhair.face` names `Faces/Longhair`) always combines: the shared folder's
face models become part of the player's face, beside any face models of the player's own, and
its `face_diff.bin` and `fcl_hair_sim.fclo` are used only when the player's folder has none of
its own. A player can also load a model kept in the export's `Common` folder through an empty
link file named after it plus `.common` (`legs.fmdl.common` in the player's folder, or in its
`face`, `boots` or `gloves` subfolder, names `Common/legs.fmdl`): the game cannot read a model
from there, so the Common model is built into the player's own face, boots or gloves as if it
were a model of the player's folder, under the name its own name or the subfolder gives it
(`legs.fmdl.common` is face content, reported as `fmdl_fcl_hair_fallback` like a local
`legs.fmdl`), and merged with the player's models of that name. A skeleton file named after it
in `Common` (`Common/legs.skl`) comes with it. The textures in `Common` (image files directly
in it) are built once for the whole team, whether a player uses them or not, and a
Common model's textures of those names are read from there, so twenty players sharing one
Common model share one copy of its textures. A model of the player's own and a Common model
must not define a material of the same name over textures kept in different places (the
player's folder and `Common`): that is a `merge_material_conflict`. When a texture in `Common`
cannot be converted, the line `folder_pack_failed` names it at `Common`, no Common texture is
built, and the players linking Common models are still built. Several boots models in one
folder (`a_boots.fmdl` beside `kit_boots.fmdl`), several gloves models for one hand, or
several face models under one name (`face_high.fmdl` beside `old_face_high.fmdl`), are merged into one model in the alphabetical order of their file names,
reported as `fmdl_merged`. Models merged into one must agree with each other: a material they
define differently is reported as `merge_material_conflict`, and skeletons that differ (one
model with a skeleton file and one without, two different files, or a bone placed differently)
as `skl_merge_conflict`; either leaves that folder's face, boots or gloves out of the CPK. A
texture under one name in two of the places a player's models come from (the player's own
folder, a shared folder it combines) is packed once when the two files are the same. When they
differ, the face's copy wins over the boots' and the boots' over the gloves': the losing part
is left out of the CPK, with the textures only it brought, and the line
`shared_texture_conflict` names the texture and what was left out. When the two files belong
to the face itself (the player's folder and a shared `Faces` folder), nothing can choose
between them: nothing of that player is built, reported as `merged_texture_conflict`. An export
using more than 17 shared boots folders, or more than 17 shared gloves folders, is skipped by
both commands with the error `boots_id_pool_exhausted` or `gloves_id_pool_exhausted`, naming
the count. A model whose name says nothing about what it is (`torso.fmdl`) is taken for face
content: it is merged into the face's `fcl_hair.fmdl`, and the line `fmdl_fcl_hair_fallback`
names it, so boots and gloves models must say so in their names. A skeleton file named after a
model (`kit_boots.skl` beside `kit_boots.fmdl`, `fcl_hair.skl` beside `fcl_hair.fmdl`,
`torso.skl` beside `torso.fmdl`, `boots.skl` beside a shared folder's `boots.fmdl`) is packed
with it; boots and hair without one get the standard body skeleton. A skeleton named after a
`face_high`, `hair_high` or `oral` model has no place in the game's face package: it is left out
and reported as `skl_no_slot`. A face folder without a `face_diff.bin`, or a hair model without
a `fcl_hair_sim.fclo` beside it, gets the bundled default file. A player's folder may sort its
files into the subfolders `face`, `boots`, `gloves` and `common` (the layout of older referee
exports): every model in `boots` is the boots and every model in `face` is a face part,
whatever their names (`boots/hair_high.fmdl` is packed as `boots.fmdl`; a model in `face`
whose name is not a face name goes into `fcl_hair.fmdl` and is reported as
`fmdl_fcl_hair_fallback`), a model in `gloves` must still say which hand it is, and `common`
holds textures. The files in these subfolders work like files in the folder itself: a skeleton
file pairs with the model beside it in the same subfolder, textures from any of them go to the
player's common folder, and parts in a subfolder are merged with the folder's own models and
with a linked shared folder of the same kind. A kit folder with
no `kit` texture, an empty one included, is built with a magenta and black checkerboard in its
place and reported as `kit_placeholder`, so a kit nobody drew shows as missing in the game. `compile`
skips any other export with the error `content_not_yet_compiled`, naming the first thing it
cannot build yet: a PES 2015 to 2017 target, a referee export, or content other than a player's
own face, boots and gloves models, their textures, `.dds` portraits, kits, linked shared
`Faces`, `Boots` and `Gloves` folders, and a `Common` folder holding only `.fmdl`, `.skl` and
texture files, its models reached through `.common` links (a portrait in a format other than
`.dds`, a model in a `gloves` subfolder whose name does not say which hand it is, or a `.common`
link to a texture or a material file, among others). `check` still checks
those exports. `compile` does not install the CPK into the game yet: it always
leaves it in the output folder.

`check` prints one line per finding: the export it is about, how serious it is, its code, where
in the export it is, and its details in parentheses. The line `Info export_identified (team=/co/,
id=714)` tells you which team the export was recognized as: its name's first word, looked up in
the teams list (`team=referees` for a `refs` export).

Both commands read every export in the exports folder from the settings (`exports/` beside
`4cc-studio` unless you changed it). To use another folder for one run, give its path as
`exports-root`; the setting is not changed. The `exports/` folder beside `4cc-studio` is created
for you when it is missing; a folder you named yourself, in the `exports_folder_path` setting or
as `exports-root`, must already exist, or the command stops before anything runs and tells you
so. An exports folder with no export in it is reported as `no_exports_found`, naming the folder,
and nothing is checked or compiled.

An export can be a folder, a `.zip` or a `.7z`, read where it is: nothing is extracted. An archive
that cannot be read (damaged, password-protected, or holding two files whose names differ only in
case) is reported as `export_extract_failed` and left out of the run.

`--export <path>` limits the run to one export: a folder, a `.zip` or a `.7z`, which does not
have to be inside the exports folder. Repeat it to name several exports. A path that does not
exist, or a file that is not a `.zip` or `.7z`, stops the command before anything runs.

`--no-deploy` builds the CPK into the output folder without installing it into the game, and
says where it is with the line `Info deploy_skipped_by_flag (path=...)`. It cannot be combined
with `--mode test` or `--mode sideload`.

In this version `compile` refuses `--mode test` and `--mode sideload`, and refuses to run while the
`multicpk_mode` setting is on. Use the normal mode with `multicpk_mode` off.

A relative path typed in the terminal is taken from the folder the terminal is in.

When a command ends, its exit code tells a script how it went:

| Code | Meaning |
|---|---|
| 0 | Finished cleanly. Warnings and notes may have been reported. |
| 1 | Finished, but some export had an error: something was left out, or kept because `pass_through` is on. |
| 2 | The command line or a setting is wrong, for example a `cpk_name` that is not a valid file name. Nothing ran. |
| 3 | The run stopped early, for example because the teams list cannot be read or the output folder cannot be written. |
