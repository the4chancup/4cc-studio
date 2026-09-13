# uniparam fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README).

| File | Source | Notes |
|---|---|---|
| `konami_pes21_UniformParameter.bin` | PES 2021 `dt00_x64` `common/character0/model/character/uniform/team/UniformParameter.bin` | 87514 bytes, WESYS-wrapped; unwrapped 326992 bytes, 2174 kit configs (96 or 120 bytes each), names from `0_DEF_1st.bin` to `referee_EU_4.bin`. Konami's layout differs from the writer's by 9 bytes, so this file is a read-only fixture |
| `pft_sample.bin` | written by the reference writer (Blue's `utils/uniparam.py`) from the synthetic entries on the right, inserted in the order listed (the writer sorts by name) | `u0702p1.bin` = bytes 1..=29; `u0701g1.bin` = 16 × 0x11; `u0701p1.bin` = 100 × 0x22; 240 bytes, unwrapped |
