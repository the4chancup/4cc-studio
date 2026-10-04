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
`glove_r`, or `handL`, `handR`), with their textures, their portraits (a `portrait` image in
the player's folder, or `player_NN` in the `Portraits` folder; a player with both gets one
portrait when the two files are identical, while two that differ are the line
`portrait_conflict` and the whole export is skipped, since the compiler cannot tell which one
is meant), and kits. A texture (a player's, a shared folder's, a Common one, a kit's or a
portrait) can be a `.dds`, `.ftex`, `.png`, `.jpg`, `.bmp`, `.webp`, `.tga` or `.tif` file:
each is converted to
what the chosen PES version reads, BC7 for PES 2019 to 2021 and BC3 for PES 2018 (BC1 when the
image has no transparency at all; a `.dds` or `.ftex` already in a format the version reads is
kept as it is), and an image without the smaller copies of itself the game needs (its mip
levels) gets them generated. A portrait is the exception: the game reads it as a `.dds` on
every version, so a `.dds` portrait is kept exactly as it is and a portrait in any other format
becomes a BC3 `.dds` of the same size with its mip levels. A texture whose name ends in `_nrm`
(`skin_nrm.png`) is a normal map and is encoded as one. Five lines name a texture that cannot be
used, each with the file: `texture_too_small` (a side under 4 pixels), `texture_not_pow2` (a
portrait whose width or height is not a power of two, or any other texture with mip levels
whose width or height is not: 256, 512, 1024, ...; a `.png` or other image always gets mip
levels, a `.dds` or `.ftex` with a single level passes at any size), `kit_texture_too_big` (a
kit's main `kit` texture, its own or the one from `all`, wider or taller than 2048 pixels or
with a side that is not a power of two), `texture_type_mismatch` (a file renamed to another
format instead of resaved, a PNG called `skin.dds`) and `texture_codec_unsupported` (a `.dds`
or `.ftex` in a format the compiler does not read, BC6H for one, or a 16-bit or interleaved
`.tga`: resave it). Both commands report the first four from the start of each file alone, so
`check` finds them too; `texture_codec_unsupported` is found when the texture is converted, by
`compile`. What is left out depends on where the texture is: a player's or shared folder's
texture leaves out the whole folder, a kit's texture the whole kit (a texture in `all` leaves
out every kit using it), a `Common` texture only that file (the rest of `Common` is still built,
and a player linking that file is left out with the line `link_target_dropped`), a portrait only
that portrait. With `pass_through` on, a texture with one of the three size lines, a portrait
included, is kept and converted as it is; a renamed one is still left out. A `logo` image at
the export's root that is not a readable image is the line `logo_file_invalid`, from both
commands, and the logo is left out even with `pass_through` on. A texture that
cannot be read for any other reason is reported as
`folder_pack_failed`, naming the file, and its folder is left out. A kit folder holding a
`kit_mask` texture compiled for PES 2018 to 2021 gets the line `kit_texture_not_used`: these
versions have no slot for it, so it is not built, and the rest of the kit is. The boots and
gloves get the ID reserved for the player's roster
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
cannot be read for a reason other than the texture lines above, the line
`folder_pack_failed` names it at `Common`, no Common texture is built, and the players linking
Common models are still built. Several boots models in one
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
a `fcl_hair_sim.fclo` beside it, gets the bundled default file. A face folder may give its
`face_diff.bin` as text instead, in a `face_diff.xml`: the file in base64, alone or inside a
`<dif>` element, the way an older face folder's `face.xml` holds it. It is decoded and packed
as `face_diff.bin`, and counts as one: the player's own, in either form, is used over a
linked `Faces` folder's. Both commands report a face diff that cannot be used as
`face_diff_invalid`, naming the file and the reason (text that is not base64, a `<dif>`
holding elements, or a file shorter than its own header says, which the game would read past
the end of), and a folder holding both `face_diff.bin` and `face_diff.xml` as
`xml_dif_conflict`, so `check` finds them too. Either leaves the whole folder out of the CPK,
even with `pass_through` on, and a player linking a `Faces` folder left out this way is left
out too, with the line `link_target_dropped`. A player's folder may sort its
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
place and reported as `kit_placeholder`, so a kit nobody drew shows as missing in the game. A
kit's `config.toml` that cannot be read (not UTF-8 text, a value of the wrong type or out of
range) is reported by both commands as `kit_config_invalid`, naming the error, and the kit is
left out, even with `pass_through` on. `compile`
skips any other export with the error `content_not_yet_compiled`, naming the first thing it
cannot build yet: a PES 2015 to 2017 target, a referee export, or content other than a player's
own face, boots and gloves models, their textures, portraits, kits, linked shared
`Faces`, `Boots` and `Gloves` folders, and a `Common` folder holding only `.fmdl`, `.skl` and
texture files, its models reached through `.common` links (a model in a `gloves` subfolder
whose name does not say which hand it is, or a `.common` link to a texture or a material file,
among others). `check` still checks
those exports. `compile` does not install the CPK into the game yet: it always
leaves it in the output folder.

`check` prints one line per finding: the export it is about, how serious it is, its code, where
in the export it is, and its details in parentheses. The line `Info export_identified (team=/co/,
id=714)` tells you which team the export was recognized as: its name's first word, looked up in
the teams list (`team=referees` for a `refs` export).

Both commands also read every `.fmdl` and `.model` model and every `.mtl` material file of the
export, an archive's included, and report what is wrong or suspicious in each, one line per file
and problem, naming the file and how many faces, vertices or bones are concerned. A
problem the game cannot live with (a mesh over a hard limit, a face naming a vertex that does not
exist) is an Error: the folder is left out (a file in `Common`, only that file), unless
`pass_through` is on, which keeps it as it is. A Warning or an Info leaves everything in. A model
with a vertex more than 5000 units from the origin lags the game for the whole matchday: it is
reported as `vertex_too_far_from_origin` and its folder is left out even when `pass_through` is
on. So is a file that cannot be read as a model or a material file at all, reported as
`model_broken` or `mtl_broken`. A player linking a shared folder or `Common` model left out this
way is left out too, with the line `link_target_dropped`.

Both commands also read each player's `settings.toml`. One that cannot be read (not UTF-8
text, a key it does not know, a value of the wrong type or out of range) is reported as
`settings_toml_invalid`, naming the error: the file is ignored, even with `pass_through` on,
and the player's models are still built.

The socks are laid out differently on a kit's main texture in PES 2015 to 2017 and in PES 2018
to 2021, so a kit drawn for one shows its sock design shifted around the leg in the other. An
empty file named `pre-fox` or `fox` in a kit folder says which games the kit's main texture was
drawn for: `pre-fox` for PES 2015 to 2017, `fox` for PES 2018 to 2021. With no such file the
compiler takes the kit as drawn for the game it compiles for. When the file names the other
games, `compile` moves the socks to the layout of the game it compiles for and says so with the
note `kit_layout_converted`, naming both layouts. The shirt, sleeves and shorts are laid out the
same in every version and are not touched, and neither are the number and name textures.

A kit folder may hold a `colors.txt` giving the kit's two menu colors, and the export's root a
`colors.txt` giving the team's colors, up to four. Both files hold one color per line, written
`#c11200` or as three numbers from 0 to 255 separated by spaces or commas (`211 74 79`,
`211, 74, 79`), optionally after a label ending in a colon (`Shirt: #c11200`). A kit's file:

```text
211 74 79
162 62 77
```

Both commands read these files and report each line that does not give exactly one color (two
colors on one line, the way an old Team Note kit entry writes them, a number over 255, a hex
color without its `#`), and each color past the second in a kit's file or the fourth in the
root one, as the warning `color_entry_invalid`, naming the line and the reason. The line is
skipped and the rest of the file is still read. `compile` writes the team's colors into the
game's team color file, which the menus and the scoreboard show; a color the file does not
give keeps the one the team had. A team export with no root `colors.txt` keeps all the colors
the team had, and `compile` says so with the note `team_colors_missing`.

`compile` also writes each kit's two colors and its icon into the game's kit color file, which
the kit selection menu and the scoreboard show. The icon is the number of the kit's `icon_<N>`
marker, or icon 3 when the kit has none. A kit whose `colors.txt` is missing or gives fewer
than two colors has its colors taken from its main texture, and `compile` says so with the
note `kit_colors_derived`. A kit with no main texture to take them from (an empty kit folder,
or one holding only other textures) gets magenta and black, so the gap shows in the game's
menus, with the warning `kit_colors_missing`. A kit the export does not hold keeps the colors
and icon it had.

When the team color file or the kit color file `compile` starts from has damaged entries (a
team's colors written over the start of its entry, so the game no longer finds that team),
`compile` repairs them and reports the warning `bin_header_repaired`, naming the file and the
teams. Those teams' colors may be wrong until their exports are compiled again.

An FPC player's body is hidden only when every kit config of the team, the goalkeeper kit's
included, carries the FPC values (shirt model 176, shorts model 16, collar 105, winter collar
105). So when any player folder of an export holds the marker `fpc_on`, `compile` builds every
kit config the export supplies with these values, and for each `config.toml` that lacked them
says so with the note `kit_config_fpc_adjusted` (your file is not changed). A kit without a
`config.toml` gets them anyway. The compiler never removes FPC values from a kit config: an
export without `fpc_on` has its configs built as they are, FPC values or not, and taking a team
off FPC is an edit you make in the configs yourself.

A kit config value the chosen PES version cannot hold (a name position `y` over 16 before PES
2021, or a shirt pattern of 12 or 13 on PES 2015) is lowered to one the version can hold when
the kit is built. Both commands report it as the warning `kit_config_version_clamped`, naming
the field, the value and the maximum.

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
