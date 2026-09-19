# 4cc Studio — Stadium compiler plan

The stadium compiler (`crates/tools/stadium_compiler`) is the successor to the PES
Stadium Compiler. Platform context is in the [core plan](core/README.md); the fox2 format
crate is outlined below, and the in-process DDS conversion it contributes is detailed
in the [library crates plan](libs/README.md).

---

## Background

The PES Stadium Compiler (`Tools_4cc/pes-stadium-compiler-v1.0.0`) is a Python tool that compiles
stadium export folders into CPK files, using a three-stage pipeline like Red:

1. **exports_to_extracted**: validate stadium exports, copy components to
   `extracted_exports/{Component}/st{sid}/`, convert DDS textures to FTEX, remap stadium IDs in all
   files (FMDL, XML, fox2.xml, FPK/FPKD)
2. **extracted_to_contents**: generate fox2 files, pack FPK/FPKD archives, assemble the directory
   structure (stadium content and billboard content separately)
3. **contents_to_patches**: generate stadium database bin files and pack everything into a CRIWARE
   CPK

It becomes the `stadium_compiler` tool crate in the suite.

## What it contributes to the whole suite

Beyond the stadium tool itself, this codebase contains two assets that upgrade the shared platform:

**1. In-process DDS conversion (drops texconv.exe and ImageMagick).** Its
`Engines/stages/lib/dxt.py` + Python `texture2ddecoder` replace the texconv subprocess entirely.
The same-named Rust decoder is a separate port; its output needs parity checks, including channel
order, against the Python package's C++ implementation.
See "In-process DDS conversion" in the [library crates plan](libs/dds_convert.md).

**2. Battle-tested, optimized parser variants.** Its `fmdl_file.py` (96KB), `cpk.py` (44KB), and
`ftex.py` (20KB) are extended/optimized versions of the shared parsers (pre-compiled struct formats
for hot paths, additional features). Use them as an additional reference when porting the shared
parsers to the format lib crates.

## New formats to port

| Format | Source | Size | Placement |
|--------|--------|------|-----------|
| **fox2** (Fox Engine entity files) | `Engines/stages/lib/fox2.py` + `Engines/stages/lib/fox2_xml.py` | 72KB | `libs/fox2` (general Fox Engine format, reusable) |
| **Stadium DB bins** (Stadium.bin per version) | `Engines/stages/lib/st_bin_gen.py` | 9KB | `tools/stadium_compiler` (stadium-specific) |

The fox2 parser is a pure-Python port of Atvaark's FoxTool (C#), including a CityHash64
implementation for string hash lookup. It compiles XML to fox2 binaries and back, byte-identical to
the original tool. In Rust: the `cityhash` crate (or a direct port of the hash — it must match
CityHash64 v1.0.3 exactly) plus a `binrw`-based fox2 structure. The byte-identical requirement makes
this a good roundtrip-test target.

## Tool-specific logic to port

| Component | Source | Size | Notes |
|-----------|--------|------|-------|
| Stage orchestration | `stadium_compiler.py`, `stages/*.py` | 61KB | Maps to the same reader/coordinator/writer pattern as the Team compiler |
| Components handling | `util/components.py` | 27KB | Stadium component validation and copying |
| Billboards | `util/billboards.py` | 30KB | Billboard content processing |
| Packing | `util/packing.py` | 16KB | FPK/FPKD + directory assembly |
| ID remapping | `util/remap.py` | 2KB | Stadium ID rewriting across file types |
| Stadium configs | `util/stadium_configs.py` | 7KB | Per-stadium configuration |

The stadium pipeline emits the same `PipelineEvent` stream as the Team compiler, so the GUI progress
grid works for stadium compilation without modification (rows = stadiums, cells = components). Its
warnings/errors likewise use the shared `Message` structure with a tool-owned `MessageCode` catalog,
to be specified when the tool is planned in detail (see "Message catalog" in the [Team compiler
plan](team_compiler/README.md)).
