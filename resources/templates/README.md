# Face templates

The files the Team compiler packs into a Fox face package when the player folder does not bring
its own (Team compiler plan, `pipeline.md` "3. Per-model-folder parallel steps", step 3 "Fox mode
fixups"; `player_folders.md` "What the injected files are"). They are embedded in the Studio
binary.

| File | Packed as | When | Source |
|---|---|---|---|
| `face_diff.bin` | `face_diff.bin` | every Fox face package whose folder has none (nor a `face_diff.xml`) | Red's `Engines/templates/`, commit `cbf16b2` (2024-04-29, "Rename template folder to templates") |
| `fcl_hair_sim.fclo` | `fcl_hair_sim.fclo` | a face package holding `fcl_hair.fmdl` whose folder has none | Red's `Engines/templates/`, commit `330bf38` (2026-06-02, "Add fcl_hair_sim template copying for face folders") |

Both are byte-identical to Blue's `lib/templates/` copies (SHA-256 `2b4ce11c…` and `ee1ee73d…`).
`face_diff.bin` is 960 bytes beginning with the `FACE` magic, version 5; `fcl_hair_sim.fclo` is
36 bytes, the `olcf` magic and zeros: a cloth simulation with nothing in it. The tracer
fixture's `fcl_hair_sim.fclo` is this file; its `face_diff.bin` is the player's own.

The `fcl_hair_sim.skl` injected beside a hair model with no skeleton of its own is not here: it
is PES 21's `body.skl` (`../skeletons/pes21/`), as for `boots.skl`.

## `DpFileList.txt`

The official 4cc DpFileList as a list, one CPK per line in load order, with comments: the
source that worklog step 4.25 writes the embedded `DpFileList.bin` from (Team compiler plan,
`pipeline.md` "6. Post-processing", DpFileList upgrade). A draft until the maintainer has
adjusted it: the newest official list (the PES 2015 and PES 2017 installs', 39 entries) with
the faces/uniform CPKs replaced by the size-split `teams` run and the `other_` pair by a
`teams2` run. Not embedded in the binary yet.
