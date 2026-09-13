# fox2 fixtures

Konami-derived files are Konami's, kept for interoperability (see the repository README). The
`.fox2` binaries were extracted from FPKD archives on the writing machine (`.tmp/fox2_fixtures.py`
in the writing session); the goldens beside each were produced by the reference decompiler and
compiler (the Stadium compiler's Python port of FoxTool): `*.fox2.xml` is the decompile with no
dictionary, `*.compiled.fox2` the compile of that XML with the reference writer's trailing buffer
slack cut at the aligned `end` trailer (its entity region is byte-identical to the original; only
the string-table order differs). Sizes are the byte counts.

| Fixture | Source (FPKD entry) | Notes |
|---|---|---|
| `audi_low_parts.fox2` (896) | PES 16 `dt00_x64_files/Asset/model/bg/common/audi/#Win/audiLowParts_model.fpkd`, `/Assets/pes16/model/bg/common/audi/audiLowParts_model.fox2` | 2 entities (`DataSet`, `AudienceModel`); String, FilePtr, EntityPtr, EntityHandle; StaticArray, DynamicArray, StringMap; the empty-string hash as a value; byte-sorted string table. `audi_low_parts.dict.fox2.xml` is the decompile with a dictionary of one empty line, which resolves that hash to `<value></value>` |
| `boots_edit_k0051.fox2` (2272) | PES 18 boots k0051 `boots_edit.fpkd`, `/Assets/pes16/model/character/boots/k0051/boots_edit.fox2` | 3 entities (`DataSet`, `TransformEntity`, `StadiumModel`); adds float, bool, int32, uint32, Vector3, Quat, Color, List; traversal-order string table |
| `steward_sit_st074.fox2` (15552) | stadium st074 staff `staff_st074.fpkd`, `/Assets/pes16/model/bg/st074/staff/st074_st2019_steward_sit.fox2` | 20 entities; adds EntityLink |
| `edit_spike.fox2` (24240) | PES 16 `dt00_x64_files/Asset/model/light/#Win/EditSpike.fpkd`, `/Assets/pes16/model/light/EditSpike.fox2` | light entities; adds Path, Vector4, uint8 |
| `hash_golden.tsv` | reference `hash_string` | 46 rows `HASH<TAB>text`: one dictionary line per byte length across every CityHash length class (0 to 200), the empty string, short names, non-ASCII |
| `float_golden.tsv` | reference `float_to_str` | 36 rows `F32BITS<TAB>text`: the C# round-trip float formatting (7 then 9 digits, `E+NN` exponents, `-0`, denormals) |

Data types with no occurrence in any `.fox2` on the machine (int8, int16, int64, uint16, uint64,
double, Matrix3, Matrix4, WideVector3) have no fixture; their codec is tested synthetically
against the layout the plan describes.
