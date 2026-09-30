# Tracer bullet fixture

One real player and one kit, as an old-layout export, its Studio-layout twin, and Red's compiled
output for the old one (`core/development_plan.md` "Phase 3", tracer bullet).

## `old/egg Tracer/`

Cut down from the /egg/ (team 792) export "EGG Aesthetics Export VGL26", a Fox-format league export:
the one player with a folder in all three of `Faces/`, `Boots/` and `Gloves/` whose files are the
smallest ("The Chad Stormworks Player", slot 05, all three folders whole, portrait included), kit
`g1` only (`Kit Textures/u0XXXg1.dds`, `Kit Configs/XXX_DEF_GK1st_realUni.bin`, both unchanged), and
`egg note.txt` with the four player-kit color lines removed (CRLF kept). The export's `Common/` is
left out: no model references it. Everything else is byte-identical to the source, including
`face.fpk.xml` listing a `face_high.fmdl` the folder does not have.

## `studio/egg Tracer/`

`old/` migrated to the Studio layout by hand, for what Phase 3's `compile` handles (Team compiler
plan, "Acceptance", Phase 3 scope): the face and the kit. The boots, gloves and portrait join it
in Phase 4, which compiles them.

- `Players/05 - The Chad Stormworks Player/`: `fcl_hair.fmdl`, `shirt.dds`, `face_diff.bin` and
  `fcl_hair_sim.fclo` unchanged; `fcl_hair_sim.skl` renamed `fcl_hair.skl`, since the Studio format
  pairs a skeleton with its model by source name ("SKL pairing"). The `.fclo` and the `.skl` are
  byte-identical to Red's templates (the skeleton is PES 21's `body.skl`); `face_diff.bin` is the
  player's own. `face.fpk.xml` is dropped: the compiler writes the FPK.
- `Kits/g1/`: `kit.dds` is `u0XXXg1.dds`; `config.toml` is `XXX_DEF_GK1st_realUni.bin` decoded as
  PES 21 by `kit_config` (`KitConfig::decode` then `to_toml`), and re-encoded with the `u0792g1`
  texture name it gives back Red's `792_DEF_GK1st_realUni.bin` byte for byte; `icon.txt` is `11`,
  the kit's icon in the note. No `colors.txt`: its grammar is a Phase 4 question.

## `red/`

The entries of the CPK Red compiled from `old/`, extracted as stored (17 files). Red 5.0.0-dev,
commit `e12aa01` of `4cc-aet-compiler-red`, run from a copy of its folder with `settings.ini` at
its defaults except `pes_version = 21`, `cpk_name = 4cc_90_tracer`, `move_cpks = 0`,
`pause_allow = 0`, `updates_check = 0`, and `pes_folder_path` pointing at a folder that does not
exist, so every bin was built on Red's fallback base (`resources/bins/`): "No issues were found".
Command: `Engines\embed\python.exe Engines\compiler_main.py 0` with `exports_to_add\egg Tracer\`.

The tree is committed whole rather than as the hash manifest the parity plan describes for the
multi-GB reference trees (`testing.md` "Testing: parity against Red"): it is under 1 MB, and
committed it lets the parity test run anywhere, CI included, without Red.
