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
