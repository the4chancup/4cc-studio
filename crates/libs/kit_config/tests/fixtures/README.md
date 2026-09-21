# kit_config fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). The
120-byte kit config format is documented in `docs/plans/kit_config_editor.md` "The format".

| File | Source | Notes |
|---|---|---|
| `template_XXX_DEF_xxx_realUni.bin` | 4cc aet compiler Red, `Engines/templates/XXX_DEF_xxx_realUni.bin` | the template defaults every generated config starts from: shirt model 176, long sleeves 0x3E, shorts 16, collar 105/105, texture names empty |
| `konami_pes21_100_DEF_GK1st_realUni.bin` | PES 2021 `dt00_x64` `UniformParameter.bin`, entry `100_DEF_GK1st_realUni.bin` | a stock goalkeeper config: shirt model 160, long sleeves 0x3E |
| `konami_pes21_model144_100_DEF_1st_realUni.bin` | same container, entry `100_DEF_1st_realUni.bin` | shirt model 144, long sleeves 0xBB (undershirt only), byte 0x13 nonzero |
| `konami_pes21_referee_EU_1.bin` | same container, entry `referee_EU_1.bin` | a referee config: short-sleeves bits = 3, a value the old editor never produced (13 of the 1372 stock configs carry it); shirt model 176 |
| `blue_pes18_UniformParameter.bin`, `blue_pes19_UniformParameter.bin` | 4cc aet compiler Blue, `lib/bins/UniformParameter18.bin` / `UniformParameter19.bin` (the 4cc kit configs of those seasons, written by the two Kit Manager variants and by Red/Blue), WESYS-wrapped by `scripts/provenance/fixtures/kit_config_containers.py` | the plan's "every entry of the bundled UniformParameter18/19.bin" mass round trip at PES 18 and PES 19: 2214 and 2210 entries, the 4cc configs with Name Y in the 5-bit encoding |

The whole PES 2021 container (1372 configs of 120 bytes; 802 further entries of 96 bytes are a
different record type and not kit configs) is `crates/libs/uniparam/tests/fixtures/konami_pes21_UniformParameter.bin`,
read through `uniparam`, and is the mass bit-identical round-trip fixture. Facts measured on it,
which the format table's "0 in samples" notes predate: bytes 0x25–0x27 are nonzero in 1368
configs, byte 0x13 in 558, short-sleeves bits are 1 (1323), 2 (36) or 3 (13); shirt models 144
(1248), 160 (111), 176 (13); long sleeves 0xBB (1081) or 0x3E (291).
