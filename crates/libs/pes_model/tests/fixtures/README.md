# pes_model fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). The
Konami `.model` and `.mtl` files ship WESYS-wrapped (`wezlib`); they are stored unwrapped here
except `konami_flag_close.wesys.model`, kept wrapped for the unwrap path. Counts are from the
reference parser. Version field: 16..19 in the header (the community parser accepts that range).

| File | Source | Notes |
|---|---|---|
| `konami_card.model` | PES 2017 `dt32_win_files/common/character1/model/character/parts/referee/card.model` | 1 bone, 1 mesh (140 vertices, 156 faces), 1 material; 10144 bytes |
| `konami_card_red.mtl` | same folder, `card_red.mtl` | the referee card's material set |
| `konami_flag_close.wesys.model` | same folder, `flag_close.model`, WESYS-wrapped as shipped | 0 bones, 1 mesh (174 vertices, 210 faces); 4383 bytes wrapped |
| `konami_modD_cap.model` + `.mtl` | `.../character/d/modD_cap.model`, `modD_cap.mtl` | 4 bones, 1 mesh (56 vertices, 78 faces), 1 material; 6336 bytes |
| `konami_hair_d_win32.model` | `.../face/edithair/model/sh_WD/hair_d_win32.model` | 2 bones, 1 mesh (165 vertices, 242 faces); 11648 bytes |
| `cardhead_face_high.model` + `cardhead_materials.mtl` | the community pre-Fox card-head template (`Models/Templates_prefox/card head`), add-on written | 1 bone, 1 mesh (16 vertices, 8 faces), material `card`, shader `Shadeless`; 1936 bytes |
| `cardhead_doublesided_face_high.model` | `.../card head doublesided/face_high.model` (its `materials.mtl` is byte-identical to the one above) | 1 bone, 1 mesh (8 vertices, 4 faces); 1400 bytes |

The reference writer cannot rebuild any parsed file (it expects a bounding box the importer sets),
so there is no legacy read-then-write parity to reproduce; the standard is our own reader/writer,
as for `fmdl`.
