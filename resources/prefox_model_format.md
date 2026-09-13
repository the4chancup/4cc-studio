# The PES `.model` format (PES 2015 to 2017)

A description of the pre-Fox `.model` container as measured on every `.model` file of a PES 2017
install, written for the author of the pes-model Blender plugin so the parts the plugin does not
yet read can be checked against evidence rather than guessed at. Where the plugin's own notes
already describe a structure, this document confirms, refines or contradicts them, and says which.

## How this was established, and how far it reaches

- **Census.** All 2610 `.model` files found on a PES 2017 install were parsed field by field and
  every value distribution recorded. 2607 are Konami's: `character/face` (1836), `character/d`
  (534, the kit parts with LODs), `character/uniform` (195), `accessory` (18), `parts` (14, the
  referee props), `body` (6), `hand` (2), `shadow` (1) and one ball. The remaining 3 ship with the
  Sider mod loader and are not Konami's. No stadium, staff, billboard or prop model was available,
  so everything the plugin's notes say about those (editor data, the order word) is neither
  confirmed nor contradicted here. A second, smaller census covered all 945 `.mtl` files.
- **Reader and writer.** A Rust reader and writer were built from these measurements. Its
  container layer reproduces every one of the 2610 files byte for byte after a read; its typed
  layer resolves every pointer, rejects any word whose value is not one of the measured ones, and
  writes a fresh file that reads back equal. So each layout statement below was executed against
  every file, not read off a hex dump.
- **Confidence tags.** *Measured* means the census saw the value in every file it applies to.
  *Inferred* means a reading consistent with the data but not proven by it. *Unverified* means the
  layout comes from the plugin's notes and no measured file exercises it.

Everything is little-endian. "File address" is an offset in the unwrapped file. "Section-relative"
is an offset from the section's first byte. "Array-relative" is an offset from the first byte of
the record array that holds the pointing record.

## 1. Wrapping

2565 of the 2610 files are WESYS-wrapped, 45 plain, and the game reads both. The wrapper is a 16-byte header, `00 10 01 'W' 'E' 'S' 'Y' 'S'`, `u32 compressed_length`,
`u32 uncompressed_length`, followed by a zlib stream. The `.mtl` files use the same wrapper (941
of 945 wrapped). Everything below describes the unwrapped bytes.

## 2. Header (24 bytes)

| Offset | Type | Value | Notes |
|---|---|---|---|
| 0 | `u8[8]` | `MODEL\0\0\0` | Measured, all files |
| 8 | `u32` | 16 | Section-table offset, counted from offset 8; the table sits at 24. Measured constant |
| 12 | `u16` | 0 | Measured constant |
| 14 | `u16` | version | 19 in 2609 files, 17 in `character/shadow/shadow_win32.model` (see section 12) |
| 16 | `u32` | 9 | Measured constant |
| 20 | `u32` | flags | 0 in 2608 files; 4 in two face-montage models. Meaning unknown |

The plugin accepts versions 16 to 19 and warns on `flags != 0`. Only 17 and 19 exist on the
install, and the two `flags == 4` files ship with the game, so a reader should carry the word
verbatim rather than warn.

## 3. The record array

Every section and every sub-table inside a section uses one shape, a *record array*:

```
u32 first_record_offset    array-relative offset of the first record; 12 when there is no header
u32 record_count
u32 record_size            bytes per record
u8  header[first_record_offset - 12]
u8  records[record_count][record_size]
... the blobs the records point at, in no fixed order
```

Two measured quirks the plugin's `RecordArray.parse` does not model:

- Konami's empty section 8 has `first_record_offset == 0` in every one of the 2607 files (the
  other empty arrays say 12). An empty array with an offset under 12 has no header and no records.
- Section 10's array has a 4-byte header of zeros and `record_size` 16 (12 in the version-17
  file) while holding no records, so `record_size` carries information even when
  `record_count == 0`.

A hostile `record_count * record_size` can overflow 32-bit arithmetic; the whole record run should
be bounds-checked before any record is read.

## 4. Section table and section order

The section table is a record array at file offset 24 with no header, eleven `u32` records of
size 4. Each record is the offset of that section from offset 24, so section i starts at file
address `24 + entry[i]`. The first section always starts at 80 (24 + 12 + 44), measured on all
files. The sections run back to back to the end of the file; a section's extent is from its start
to the next section's start in *file* order, and the last one runs to the end of the file.

Table index and file order are different things. The table is always indexed by role (0 = bone
data, ..., 10 = locators); the order in which the sections are laid out in the file is:

- Konami: `0 1 2 3 4 5 6 8 9 10 7`, in all 2607 files.
- The plugin: `7 0 5 6 3 8 9 10 1 2 4`.

The game accepts both, so the order is a writer's choice. Konami's cross-section pointers
(section 4, below) are signed for exactly this reason: a section may sit before or after the one
pointing at it.

## 5. Section 0: bone data

Top-level array: no header, `record_size` 4, one `u32` section-relative offset per *entry*.
Each entry is itself a record array with a 4-byte header holding `u32 2` (meaning unknown,
measured constant in all 7497 entries) and `record_size` 12:

```
u32 offset      entry-relative offset of the data
u32 format      a datum-format word, the same vocabulary as vertex fields (section 6.3)
u32 count
```

- **Entry 0** holds the bones: one record per bone, `format == 7` (`float32Matrix34`),
  `count == 1`, `offset` pointing at 48 bytes: twelve `f32`, the bone's inverse bind matrix as
  the plugin already interprets it. Records are in bone-name order (section 5 of the file); the
  two tables must have the same length. A model without bones still has entry 0, with zero
  records (3 files: `flag_close` and two others). Measured across files: a bone name shared by
  two files of one skeleton has matrices that agree only to float noise, up to `3.6e-5` per
  component (8268 such pairs; `modD_cap` against a collar part differs on four shoulder bones),
  while a real bind-pose difference is at least `2.0e-4` and usually far more (the gloves'
  forearm and hand bones differ from the collars' by 0.5). Code that decides whether two parts
  share a skeleton should compare with a tolerance around `1e-4`, not for equality.
- **Every further entry** is one bone group: exactly one record, `format == 1` (`uint16`),
  `count == N`, `offset` pointing at `N` `u16` indices into the bone table. Group sizes run from
  1 to 51.

The plugin distinguishes matrices from groups by the format word and notes it is "unclear
whether this is a requirement" that matrices come first. Measured: in all 2607 files the matrices
are entry 0 and only entry 0, and every other entry has exactly one record. A writer can rely on
that; a reader may keep the format-word check as a guard.

## 6. Section 1: geometry

Top-level array: no header, `record_size` 20 (measured on all files including the version-17
one; the plugin also accepts 16, which no file on the install uses). One record per geometry;
section 4 has exactly one mesh per geometry in every file.

```
u32 1                          measured constant
u32 vertex_set                 section-relative
u32 face_descriptor            section-relative
u32 0                          measured constant
u32 extras                     section-relative
```

### 6.1 Vertex set

A record array, no header, `record_size` 4, exactly one record: the section-relative offset of
the field-descriptor array. (The plugin raises on any other count; the census agrees: always 1.)

### 6.2 Field descriptors

A record array with an 8-byte header and `record_size` 20.

Header: `u32 cloth, u32 0`. The plugin's note says the first word is nonzero for models with
section-8 content and a `.cloth` file. It is 0 in all 4890 geometries measured (the install has
no cloth model), so the note stands unconfirmed. The second word is 0 everywhere.

Each record:

```
u32 data            section-relative offset of the field's data
u32 datum_type
u32 datum_format
u32 vertex_count    identical across every field of one geometry (measured, no exception)
u32 0               measured constant
```

The field data is `vertex_count * size(datum_format)` contiguous bytes. Whether two descriptors
of one geometry ever point at the same data (the plugin has a commented-out check for uv maps
sharing an offset) was not measured.

### 6.3 Datum types and formats

The words are the plugin's, confirmed. Sizes are per vertex.

| Format word | Meaning | Size |
|---|---|---|
| 1 | `uint16` | 2 |
| 2 | `uint32` | 4 |
| 3 | `float32` | 4 |
| 4 | two `f32` | 8 |
| 5 | three `f32` | 12 |
| 6 | four `f32` | 16 |
| 7 | 3x4 `f32` matrix | 48 |
| 8 | four `i8` | 4 |
| 9 | four `u8` fixed-point (value / 255) | 4 |

| Type word | Meaning | Formats measured | Geometries |
|---|---|---|---|
| 2 | position | 5 | 4890 (every geometry) |
| 3 | normal | 5 | 4886 |
| 4 | color | 9 | 1818 |
| 7 | uv0 | 4 | 4889 |
| 8 | uv1 | 4 | 1206 |
| 9, 10 | uv2, uv3 | (none on the install) | 0 |
| 15 | tangent | 5 | 4567 |
| 16 | bitangent | 5 | 1835 |
| 17 | bone weights | 6 (3602), 4 (1106), 5 (50) | 4758 |
| 18 | bone indices | 8 | 4887 |

Bone weights are stored two, three or four `f32` per vertex; the width is the format word. A
writer that only ever emits `quadFloat32` produces valid files, but a lossless rewrite must keep
the width the file had. Bone indices index the mesh's bone group, not the bone table. No
geometry repeats a datum type (the plugin's duplicate check never fires on Konami files).

**Field order.** Konami writes the descriptors in one fixed order, measured across 16 distinct
combinations: position, normal, bitangent, tangent, uv0, uv1, color, bone indices, bone weights,
each present or absent. Note bitangent (16) precedes tangent (15). A writer that emits its own
order still produces a valid file; this is recorded because the order is the one thing a
byte-exact rewrite has to reproduce.

### 6.4 Face descriptor

A record array, no header, `record_size` 24, exactly one record:

```
u32 start               section-relative offset of the index stream
u32 1                   measured constant (the plugin's unknown6)
u32 format              1 (uint16) in every file
u32 count               face-vertex count; a multiple of 3 in every file
u32 lod_levels          0, or 4 to 7
u32 lod_table           section-relative offset of the LOD table
```

The index stream is `count` `u16` indices, three per triangle, the full stream including every
LOD level.

**LOD table.** `lod_levels` records of `u32 start, u32 end` (face-vertex indices, `end`
exclusive), level 0 first. Measured on all 532 files that have one: the ranges are contiguous and
partition the stream (`start[0] == 0`, `start[n+1] == end[n]`, `end[last] == count`), and each
level is smaller than the one before it (inferred: level 0 is the most detailed, as the plugin
assumes). Example (`modD_shirt_in_collar_001`, 40023 face vertices, 7 levels):
`0-12378, -21762, -28704, -33660, -37008, -39048, -40023`.

The plugin reads level 0 and drops the rest; a rewrite therefore loses the LODs of every
`character/d` shirt and collar part. Keeping the table and the full stream is enough to preserve
them; nothing else in the file refers to the levels except section 7's LOD record (section 11).

**`lod_table` when there are no LODs.** Konami's value is not 0: it is the section-relative
offset just past the last byte of data in the section (equal to the section length, or two bytes
short of it when the section ends on a 2-byte pad, as in the version-17 file). The plugin writes
0. Both load, so this is a curiosity, not a requirement.

### 6.5 Extras (the plugin's "misc" array)

A record array, no header, `record_size` 4, three records (two in the version-17 file), each an
*array-relative* offset:

1. **Bounds**, 32 bytes: `f32 min[4], f32 max[4]`. The fourth component is 0.0 in every file.
2. **Material-combination flags**, 16 bytes: `u32[4]`, `1, 0, 0, 0` in all 4890 geometries. The
   plugin's note says bit 0 "is always 0?"; measured, the first word is always exactly 1 and the
   others 0. Nothing on the install references section 9.
3. **Order word**, 4 bytes: `u32`, 0 in all 4889 version-19 geometries; absent in the version-17
   file. The plugin's stadium observation ("1 to N across parts") could not be checked.

## 7. Section 2: annotation strings

A record array, no header, `record_size` 8: `u32 offset` (section-relative, to a NUL-terminated
string), `u32 0` (measured constant in all 6469 records). Strings are referenced from mesh
annotations (section 9 of this document); a string no mesh references is a model-level
annotation, which is how the plugin stores its extension headers such as
`Skeleton-Type: Simplified`.

## 8. Section 3: annotation records

A record array, no header, `record_size` 28: seven `u32`, `0 0 0 2 2 2 0` in all 6494 records on
the install. Konami writes one per mesh annotation and points at it from the annotation (below).
Their meaning is unknown; the plugin neither reads nor writes them (its annotations carry a zero
pointer, which the game accepts). 3 files carry 2 section-3 records no annotation points at.

## 9. Section 4: meshes

Top-level array: no header, `record_size` 24 (version 19) or 20 (version 17). Every pointer that
leaves this section is a **signed `i32` delta from section 4's file address**, so
`file_address = section4_start + delta`, and the result is looked up in whichever section it
lands in. Pointers that stay inside section 4 are unsigned and section-relative.

```
i32 geometry            signed delta to a section-1 geometry record
u32 bone_group_array    section-relative; 0 never seen from Konami, but the plugin accepts it
u32 annotation_array    section-relative; 0 when there are none (plugin); Konami writes the array
i32 material            signed delta to a section-6 material-name record
i32 locator             signed delta into section 10; 0 in every file
u32 editor_data         version 19 only: section-relative; Konami writes the array, present and empty
```

Because section 6 follows section 4 in Konami's order and precedes it in the plugin's, the
`material` delta is positive in Konami's files and negative in the plugin's; both are correct.

### 9.1 Bone-group array

A record array, no header, `record_size` 4, zero or one record (`i32`, a signed delta to a
*section-0 entry*, never entry 0). 4887 of 4890 meshes have one record; the 3 boneless meshes
have an empty array, not a zero pointer.

### 9.2 Annotation array

A record array, no header, `record_size` 16, one record per annotation:

```
i32 string      signed delta to a section-2 record
i32 record      signed delta to a section-3 record; 0 in plugin-written files
u32 7           measured constant in Konami files (8 in the three non-Konami files)
u32 kind
```

Konami's annotation kinds, all with a section-3 record:

| Kind | Count | Text | Where |
|---|---|---|---|
| 1 | 4514 | the part name: `prt`, `glasses_02`, `head_color`, `hair_...` | Always the first annotation of a mesh, when any |
| 10 | 2753 | the same string as kind 1 again | Hair and glasses |
| 2 | 532 | `DSpecularS` | The LOD-bearing kit parts |
| 7 | 1448 | a normal-map name: `DNormalS`, `HeadNormal`, `head_normal_default`, `<hair>_face_normal` | Faces, hair, kit parts |

Measured sequences per mesh: `(1, 10)` 2751, `(1, 7)` 914, `(1, 2, 7)` 532, none 376, `(1)`
315, `(1, 10, 7)` 2. The plugin's own kinds are 128 (mesh name) and 129 (extension header), with
no section-3 record. The plugin reads only 128 and 129 and so drops all four Konami kinds on a
rewrite; whether the game uses them is not known, but the strings look like material or shader
hints (`DSpecularS`, `DNormalS`) rather than editor leftovers. Keeping them costs one string and
one 28-byte record each.

### 9.3 Editor data (unverified)

Section-relative offset of a record array, no header, `record_size` 4, each record an
array-relative `u32` offset to an item:

```
u32 kind
u32 unknown
value       kind == 1: NUL-terminated string padded to 4; otherwise u32
```

This is the plugin's documented layout, repeated here because it could not be checked: all 4889
version-19 meshes on the install carry the array present and *empty* (the version-17 record has
no pointer). A parser must at least accept the empty array; a writer that emits 0 (as the plugin
does) produces files the game loads. The plugin's example content (`TI_Mod_WEInfo_Stadium`,
`PES2010`, ...) is the only known sample.

## 10. Sections 5 and 6: bone names and material names

Both are the same shape: a record array, no header, `record_size` 4, one `u32` section-relative
offset per NUL-terminated string. Section 5 has one name per bone, in entry-0 order. Section 6
has one name per material; meshes point at the *record*, not the string. Material counts per
file: 1 (1541), 2 (161), 3 (605), 4 (303), the same distribution as the mesh count. Names are the
`<material name>` values of the sibling `.mtl` (appendix A).

## 11. Section 7: model bounds and the LOD record

A record array, no header, `record_size` 4, exactly two records, each a section-relative offset:

1. **Bounds**, 32 bytes, `f32 min[4], f32 max[4]`, fourth components 0.0 (all files).
2. **LOD record**, 16 bytes: `u32 level_count, f32 a, f32 b, f32 c`. Measured values:
   `0, 0.0625, 4.0, 0.0` in the 2078 files without LODs, and `N, 0.0625, 4.0, 0.3` (with N the
   face descriptor's `lod_levels`, 4 to 7) in the 532 with. The meaning of the three floats is
   not known; the plugin's "sensible ones on export" are consistent with the no-LOD row.

## 12. Sections 8, 9 and 10

Empty in all 2610 files, and nothing references them (no cloth word, no material-combination bit,
no locator delta). What can be said is their empty shape: section 8 is a 12-byte array with the
offset-0 quirk and `record_size` 4; section 9 a 12-byte array with `record_size` 4; section 10 a
16-byte array with a 4-byte zero header and `record_size` 16 (12 in the version-17 file). The
plugin's notes on their content (cloth, material combinations, locator geometry) are neither
confirmed nor contradicted.

## 13. Version 17

One file on the install, `shadow_win32.model`, differs from the 2609 version-19 files in exactly
three places:

- Section 4 records are 20 bytes: no editor-data pointer.
- The geometry extras array has two records: bounds and flags, no order word.
- Section 10's empty array declares `record_size` 12 instead of 16.

Everything else (header, record arrays, section 0, field descriptors, face descriptor) is
identical. A reader that keys the mesh record layout on `record_size` rather than on the version
word handles both. Whether a version-17 header over version-19 record sizes (or the reverse)
loads was not tested; the safe writer choice is the version-19 layout with version 19.

## 14. Alignment and padding

Konami aligns some pointed-at blobs to 8 or 16 bytes with zero padding, in no pattern the census
could name (field data starts at every residue mod 16; sections start at 0, 4, 8 or 12 mod 16).
533 files contain bytes no pointer covers, all of them zero. No file has an unreferenced
non-zero byte, so a reader can treat unreferenced bytes as padding. The plugin pads every blob
to 4 and every section to 4 with no gaps; those files load, so the larger alignment is not a
requirement.

The three non-Konami files (Sider) are where every remaining oddity of the census sits: sections
at odd offsets (eight, across two files), annotation word 8 instead of 7, and in `cs.model` two
annotation string pointers that resolve to nothing. A strict reader
should tolerate the first two and treat the third as an unresolved reference.

## 15. Summary of differences between Konami's files and plugin-written files

Everything in this list loads in the game either way; it is a list of what a *lossless* reader
or writer would need to change, not of bugs.

| Item | Konami | Plugin |
|---|---|---|
| Section file order | `0 1 2 3 4 5 6 8 9 10 7` | `7 0 5 6 3 8 9 10 1 2 4` |
| Annotation kinds | 1, 2, 7, 10, each with a section-3 record | 128, 129, zero record pointer; Konami's kinds dropped on read |
| LOD table and lower levels | kept, 532 files | level 0 only |
| `lod_table` without LODs | offset past the section's data | 0 |
| Editor-data pointer | present, empty array | 0 |
| Bone-weight width | 2, 3 or 4 `f32` | always 4 on write |
| Field-descriptor order | position, normal, bitangent, tangent, uv0.., color, indices, weights | position, normal, tangent, bitangent, color, uv0.., indices, weights |
| Degenerate triangles | not measured | dropped on read |
| Empty section 8 | `first_record_offset` 0 | 12 |
| Padding | blobs aligned to 8 or 16 in places | 4 |

Two more observations that may help the plugin's writer rather than its reader: the writer
requires a bounding box the importer computes, so a file that was only parsed cannot be written
back; and the field-descriptor order (section 6.3) is the only ordering a byte-exact rewrite has
to reproduce, the rest being pointer-addressed.

## Appendix A. The sibling `.mtl`

Every `.model` ships with a `.mtl` of the same base name, WESYS-wrapped like the model (941 of
945), plain XML inside: no declaration, no BOM, no text content.

```xml
<materialset>
    <material name="..." shader="...">
        <sampler name="NormalMap" path="model/character/.../x.dds" srgb="0"
                 minfilter="linear" magfilter="linear" mipfilter="linear"
                 uaddr="clamp" vaddr="clamp" waddr="wrap" maxaniso="2" />
        <state name="twosided" value="0" />
        <vector name="Shininess" x="0.035" y="0" z="0" w="0" />
    </material>
</materialset>
```

Measured on all 945 files: `<material>` attributes are always `name, shader` (24 shader names;
`Hair` 1808, `Wrinkle` 916, `Accessory` 311, the rest under 20 each). `<sampler>` attributes come
in the order `name, path, srgb, minfilter, magfilter, mipfilter, uaddr, vaddr, waddr, maxaniso`
with trailing ones omitted (four subsets seen); `srgb` is 0 or 1; filters are `linear` (1319 of
1328 `minfilter`) or `anisotropic` (9); addresses `clamp`, `wrap` or `repeat`; `maxaniso` is 2;
`path` ends in `.dds` and is either `./name.dds` (96) or `model/...` from the data root (1232).
Sampler names: `NormalMap` (945), `RoughnessMap` (312), `DiffuseMap` (47), `SpecularMap` (11),
`Normal2`, `Mapping`, `DetailMaterialMap`, `DetailNormalMap`, `TranslucentMap`, `DiffuseMap2`,
`ViewEnvMap`. `<state name value>`: `twosided`, `ztest`, `zwrite`, `alphatest`, `alpharef`,
`alphablend`, `blendmode` (about 3070 each) and `shadowcaster` (18); values 0, 1, 64, 254.
`<vector name x y z w>` (two files carry `x` alone): `Shininess`, `Reflection`, `DepthBias`,
`SpecularColor`, `EyeColor`, `EyeColorCenter`, `EyeParam`. Children appear in any order (states
before or after samplers). 917 files are CRLF with four-space indents, 28 LF; 15 use tabs.

## Appendix B. Where the evidence lives

The reader, writer and tests described here are the `pes_model` crate of the 4cc Studio
repository (`crates/libs/pes_model/`), with one fixture per measured variant under
`tests/fixtures/` and their provenance in that folder's `README.md`. The layout notes the crate
was built from are in `docs/plans/libs.md`, section "`pes_model::format`".
