# pes_model fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). Konami
`.model` and `.mtl` files ship WESYS-wrapped (`wezlib`). The four `.model` files copied first are
stored unwrapped; every later one (`*.wesys.model`) is stored as shipped, wrapped, which is
smaller and exercises the unwrap path. `.mtl` files are stored unwrapped (they are XML). Counts
are from the layout census (`scripts/provenance/pes_model/model_census.py`; the numbers below are
the record it left). Header version 19 everywhere except `konami_shadow_win32` (17).

| File | Source (under PES 2017 `Data/`) | Notes |
|---|---|---|
| `konami_card.model` | `dt32_win_files/common/character1/model/character/parts/referee/card.model` | 1 bone, 1 mesh (140 vertices, 156 faces), 1 material, bitangents, no annotation; 10144 bytes |
| `konami_card_red.mtl` | same folder, `card_red.mtl` | the referee card's material set |
| `konami_flag_close.wesys.model` | same folder, `flag_close.model` | 0 bones (section 0 holds one empty entry), 1 mesh (174 vertices, 210 faces), annotation type 1; 4383 bytes wrapped, 7520 unwrapped |
| `konami_modD_cap.model` + `.mtl` | `.../character/d/modD_cap.model`, `modD_cap.mtl` | 4 bones, 1 mesh (56 vertices, 78 faces), uv1, `quadFloat32` weights, annotation type 1; 6336 bytes |
| `konami_hair_d_win32.model` | `.../face/edithair/model/sh_WD/hair_d_win32.model` | 2 bones, 1 mesh (165 vertices, 242 faces), `doubleFloat32` weights, annotation types 1 and 10; 11648 bytes |
| `konami_shadow_win32.wesys.model` + `konami_shadow.mtl` | `.../character/shadow/shadow_win32.model`, `shadow.mtl` | **version 17 layout**: 20-byte mesh records (no editor-data pointer), two per-mesh extras (no order word), 12-byte section-10 records; 19 bones, 1 mesh (position, normal, uv0 only; 2301 face vertices), no annotation; 7938 wrapped, 16928 unwrapped |
| `konami_glasses_02.wesys.model` + `konami_accessory.mtl` | `.../character/accessory/glasses_02.model`, `accessory.mtl` | 2 meshes, 2 materials, 2 bone groups over 1 bone, one annotation string used by both meshes (types 1 and 10 each), 2 section-3 records; 12118 wrapped, 27936 unwrapped |
| `konami_headHi.wesys.model` + `konami_headHi.mtl` | `.../face/common/headHi.model`, `headHi.mtl` | 4 bones, 1 mesh (341 vertices), annotation types 1 (`head_color`) and 7 (`HeadNormal`); 19339 wrapped, 30528 unwrapped |
| `konami_taping.wesys.model` | `.../character/accessory/taping.model` (`accessory.mtl` above is its material set) | `tripleFloat32` bone weights; 19394 wrapped, 37264 unwrapped |
| `konami_hair_high_sp_ty004.wesys.model` + `konami_hair.mtl` | `.../face/edithair/model/sp_ty004/hair_high_win32.model`, `hair.mtl` | vertex colors (`quadFloat8`), 2 meshes, 2 materials; 41388 wrapped, 64864 unwrapped |
| `konami_collar_052.wesys.model` | `dt35_win_files/common/character0/model/character/d/modD_shirt_tight_in_collar_052.model` | **6 LOD levels** (face ranges 0-12240, -21510, -28353, -33231, -36510, -38496; section 7 LOD record `6, 0.0625, 4.0, 0.3`), 23 bones, 2646 vertices, annotation types 1 (`prt`), 2 (`DSpecularS`), 7 (`DNormalS`); the largest fixture at 148672 wrapped, 280848 unwrapped, kept because no smaller Konami file carries a LOD table |
| `cardhead_face_high.model` + `cardhead_materials.mtl` | the community pre-Fox card-head template (`Models/Templates_prefox/card head`), add-on written | 1 bone, 1 mesh (16 vertices, 8 faces), material `card`, shader `Shadeless`, annotation types 128 (mesh name) and 129 (extension header); add-on section order; 1936 bytes |
| `cardhead_doublesided_face_high.model` | `.../card head doublesided/face_high.model` (its `materials.mtl` is byte-identical to the one above) | 1 bone, 1 mesh (8 vertices, 4 faces); 1400 bytes |

The `.mtl` files differ in whitespace between Konami folders (`konami_shadow.mtl` and
`konami_accessory.mtl` are CRLF with tabs, `konami_headHi.mtl` LF with tabs, `konami_hair.mtl`
CRLF with four-space indents), so `.mtl` byte parity is per file, not one canonical formatting.

The reference writer cannot rebuild any parsed file (it expects a bounding box the importer sets),
so there is no legacy read-then-write parity to reproduce; the standard is our own reader/writer,
as for `fmdl`.
