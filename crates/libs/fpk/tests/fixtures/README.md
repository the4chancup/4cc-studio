# fpk fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README).

| File | Source | Entries |
|---|---|---|
| `konami_d2_coaching_target.fpk` | PES 2021 `dt00_x64` `Asset/model/game2d/d2_coaching_target/#Win/d2_coaching_target.fpk` (1312 bytes) | `/Assets/pes16/model/game2d/d2_coaching_target/scenes/d2_couaching_target.fmdl` 1125 bytes |
| `konami_audiLowParts_model.fpk` | PES 2021 `dt00_x64` `Asset/model/bg/common/audi/#Win/audiLowParts_model.fpk` (11920 bytes) | `/Assets/pes16/model/bg/common/audi/scenes/au00.skl` 1088 bytes; `.../au_Low_parts.fmdl` 10567 bytes |
| `konami_audi_seat_model.fpkd` | PES 2021 `dt00_x64` `Asset/model/bg/common/audi/#Win/audi_seat_model.fpkd` (1056 bytes) | `/Assets/pes16/model/bg/common/audi/seatOnly_model.fox2` 896 bytes |
| `red_template_generic.fpkd` | 4cc aet compiler Red, `Engines/templates/generic.fpkd` (48 bytes) | none: an empty FPKD as pes-file-tools' writer emits it |
| `pft_sample.fpk` / `pft_sample.fpkd` | written by pes-file-tools' `FpkFile.write` (Blue's `utils/fpk.py`) from the synthetic entries on the right, inserted in the order listed (the writer sorts by name) | `.../70101/sourceimages/#windx11/face_bsm.ftex` = 33 × `B`; `.../70101/face_high.fmdl` = 100 × `A`; `.../70101/face.fmdl` = bytes 0..16 (17 bytes); all under `/Assets/pes16/model/character/face/real/` |
