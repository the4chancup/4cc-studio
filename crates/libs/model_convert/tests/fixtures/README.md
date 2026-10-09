# model_convert fixtures

Every file is a copy of a `fmdl` or `pes_model` fixture (small samples needed by several crates
are copied, per `CONTRIBUTING.md` "Testing"); the provenance and per-file notes are in those
crates' `tests/fixtures/README.md`. Konami-derived files are Konami's, kept for interoperability.

| File | Copied from | Why here |
|---|---|---|
| `konami_highneck.fmdl` | `fmdl` | a skinned PES 2021 player part on the body skeleton (7 bones, 1 mesh) |
| `konami_au_Low_parts.fmdl` + `konami_au00.skl` | `fmdl` | the one FMDL + companion SKL pair: bind pose read from the SKL, positions consistent with it |
| `addon_oral.fmdl` | `fmdl` | add-on written (the writer 4cc exports come from) |
| `konami_modD_cap.model` + `.mtl` | `pes_model` | `Basic_C`, four body bones including `dsk_deltoid_l` and `dsk_upperarm_long_l` (bones PES15 lacks: the retargeting fixture) |
| `konami_card.model` + `konami_card_red.mtl` | `pes_model` | bitangents, one bone |
| `konami_glasses_02.wesys.model` + `konami_accessory.mtl` | `pes_model` | two meshes, two materials, WESYS-wrapped |
| `cardhead_face_high.model` + `cardhead_materials.mtl` | `pes_model` | add-on written, `Shadeless`, `Skeleton-Type: Simplified` header |
| `konami17_boots_k0051.model` + `.mtl` | PES 2017 `dt33_win.cpk`, `common/character0/model/character/boots/k0051/` (WESYS unwrapped) | stock boots on the boots pose, 7° off the body table, both feet in one mesh: the pre-check must not flag it on PES 17 |
| `konami17_glove_r_g101.model` + `.mtl` | PES 2017 `dt33_win.cpk`, `.../glove/g101/` | stock glove 0.0025 off the hand table on three finger bones: the tolerance's floor |
| `konami21_boots_k0051.fmdl` + `.skl` | PES 2021 `dt33_g4.cpk`, `Asset/model/character/boots/k0051/#Win/boots.fpk` | stock Fox boots with their own six-bone `boots.skl` (the boots pose): not flagged on PES 21 |
| `tracer_prefox_boots.model` + `.mtl`, `tracer_prefox_glove_l.model` + `.mtl`, `tracer_prefox_face_high.model` + `tracer_prefox_face.mtl` | `team_compiler`'s `tracer_prefox` fixture (the pre-Fox parity export, Fumos) | community models that draw right on PES 17: boots on the boots pose, gloves on PES 15's arm pose with every bone sharing one delta, a face with its own `skf_*` pose; none may be flagged on PES 17 |

`legacy19to16_oral.model` + `.mtl` are the 19to16 converter's own output for `addon_oral.fmdl`
(`Engines.lib.convertFaceFolder` on a folder holding only the FMDL, 2026-09-14): the semantic
reference the Fox→pre-Fox path is compared against. Its texture path is `./.dds` because the
converter looks for the texture file next to the model and none was there; ours keeps the FMDL's
path, so the comparison excludes the path. Its material is `Basic_C` with the diffuse sampler
alone, and so is ours: the FMDL's `NormalMap_Tex_NRM`/`SpecularMap_Tex_LIN` name the game's
`dummy_nrm`/`dummy_srm`, which stand for no map, so the import gives them no role and the
`Basic_*` ladder stops at `C` (the converter dropped them for another reason, as maps it could
not resolve on disk; the result is the same). Geometry, bone matrix, states, the shader, the
sampler names and the diffuse sampler's attributes are compared. The same run dropped `konami_highneck.fmdl` entirely
(its `translucent` shader is a decal the converter keeps only with a texture file present), so
no highneck reference exists.

`hand_split_wrist.json` is not a copy: it is Blender's own result (select the `skh_*_l`-weighted
vertices, Select More once, Separate by selection) on the synthetic connected-wrist mesh that
`hand_split_wrist.py` builds, the semantics the plan's hand auto-split reproduces. Regenerate with
`blender -b --python hand_split_wrist.py -- hand_split_wrist.json` (made with Blender 5.2.1); the
Rust test builds the same mesh and compares face sets, so Blender is never needed at test time.
