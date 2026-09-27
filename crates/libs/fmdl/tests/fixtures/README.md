# fmdl fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). FMDL
version is 2.03 in every file. Mesh/bone/material counts are from the reference parser.

| File | Source | Notes |
|---|---|---|
| `konami_highneck.fmdl` | PES 2021 `common_package_fpk/Assets/pes16/model/character/common/highneck.fmdl` | skinned character part: 1 mesh, 7 bones, 1 material; 12527 bytes |
| `konami_mouth.fmdl` | same folder, `mouth.fmdl` | 1 mesh, 6 bones, 1 material; 18442 bytes |
| `konami_au_Low_parts.fmdl` | unpacked from `fpk`'s `konami_audiLowParts_model.fpk` (`.../bg/common/audi/scenes/au_Low_parts.fmdl`) | an audience (crowd) body model, human `sk_*` bones: 4 meshes, 16 bones, 1 material; 10567 bytes |
| `addon_oral.fmdl` | a 4cc export (MARISA, `Faces/XXX01/oral.fmdl`), written by the community Blender add-on | 1 mesh, 1 bone, 1 material; 1498 bytes. The reference writer round-trips this file byte-identically; it cannot write the Konami files at all |
| `addon_placeholder.fmdl` | `Models/Model18/placeholder.fmdl`, add-on written | 1 mesh, 0 bones, 1 material; 1218 bytes |
| `konami_boots.skl` | PES 2021 `common_package_fpk/.../character/common/boots.skl` | SKL: 276 bytes |
| `konami_referee_f_close.skl` | `.../referee_flag/scenes/referee_f_close.skl` | SKL: 156 bytes |
| `konami_au00.skl` | unpacked from `konami_audiLowParts_model.fpk` | SKL: 1088 bytes, 16 bones (pairs with `konami_au_Low_parts.fmdl`) |
| `f16_golden.tsv` | `scripts/provenance/fixtures/fmdl_f16_golden.py` (numpy 2.4.5) | `F32BITS<TAB>F16BITS<TAB>EXACT`, 1870 rows: numpy's float32 to float16 cast (ties to even) as the encoder's oracle; `EXACT` = 1 when the input is a half value, so the row also checks the decoder. Every half exponent at ten mantissas, the tie to the next half and one ulp either side, negated copies, the overflow and subnormal edges. No NaN rows |
