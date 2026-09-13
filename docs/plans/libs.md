# 4cc Studio — Library crates plan

Covers the small shared lib crates: the binary format parsers (`cpk`, `fpk`, `fmdl`,
`pes_model`, `ftex`, `uniparam`, `wezlib`), `dds_convert`, `color_tools`,
`vtree`, `pes_version`, `archives`, `elevation`, `fpc`, and `teams_list`. The aesthetics export format's object model, folder conventions,
and validation live in `aesthetics_export` (shared by the Team compiler, Export upgrader,
Kit config editor, Refs arranger, and Team creator), specified in the
[Team compiler plan](team_compiler.md). The two big libs have their own plans:
[Model conversion](model_conversion.md) (`model_convert` + the mesh algorithms in
`fmdl`/`pes_model`) and [Savefile](pes_savefile.md) (`pes_savefile`). The `pipeline`
crate (reader/writer scaffolding, memory budget, folder watcher, check cache, shared `CpkStem`
validation) is
specified across the [core plan](core.md) (Parallelism) and the
[Team compiler plan](team_compiler.md) (walkthrough, live validation). The music lib
crates (`music_export`, `audio_engine`) are specified in the
[Music player plan](music_player.md), the live-match event feed (`match_feed`)
in the [Match tracker plan](match_tracker.md), the kit config codec
(`kit_config` — binary/TOML, reverse-engineered format documentation) in the
[Kit config editor plan](kit_config_editor.md), the AATF rules engine
(`aatf`) and the shared tactics/card widgets (`team_widgets`) in the [Save editor
plan](save_editor.md), and the fox2 parser in the
[Stadium compiler plan](stadium_compiler.md). Platform context is in the
[core plan](core.md).

---

## Format references in Blue

| Format | Blue file | Size | Notes |
|--------|-----------|------|-------|
| CPK (CRI archive) | `utils/cpk.py` | 550 lines | UTF table with XOR encryption, read + write |
| FMDL (Fox Model) | `utils/FmdlFile.py` | 1700 lines | Largest parser; segmented binary format |
| SKL (Fox Skeleton) | — | ~150 lines | Bone list + bind-pose transforms; reference parser at `examples/skl.py`; lives in the `fmdl` crate (same Fox format family) |
| FPK/FPKD | `utils/fpk.py` | 137 lines | Fox package with MD5 checksums |
| FTEX | `utils/ftex.py` | 400 lines | PES texture with mipmap/BCn block math |
| UniformParameter | `utils/uniparam.py` | 91 lines | Kit config container |
| CRILAYLA | `utils/crilayla.py` | 72 lines | Bitstream LZ decompression |
| WESYS/zlib | `utils/zlib_plus.py` | 146 lines | zlib wrapper with custom header |
| VirtualTree / `ScopePath` / `RelativeScopePath` | `utils/vtree.py` | 187 lines | In-memory file tree plus canonical absolute-within-export and scope-relative path types (case-insensitive lookup with explicit collision checks) |

## `vtree` path types

`vtree::ScopePath` is the canonical export/event path. `vtree::RelativeScopePath`
represents a canonical path relative to an already validated task or folder scope;
it is constructed only through that scope root, normalizes separators identically to
`ScopePath`, and rejects traversal, absolute paths, and empty/noncanonical segments.
Case-folded and Unicode-normalized collisions are detected before either type enters
a tree. Joining a `RelativeScopePath` back to its validated root returns a checked
`ScopePath` and cannot escape or silently collide with an existing entry.
Filesystem source providers also enforce containment after resolving symlinks/junctions; a safe
virtual path alone does not prove that its physical source remains inside the export.

## `pes_version`

`pes_version::PesVersion` is the closed set of supported versions (`Pes15`–`Pes21`) with
`engine()` (`PreFox` for 15–17, `Fox` for 18–21), `number()`/`year()`, `Display` (`PES 2021`),
`FromStr` (`21`, `2021`, `pes21`) and serde as the two-digit number. It is its own leaf crate,
not a type inside `pes_savefile`, because `studio_core`'s common settings, `model_convert`'s
skeleton tables, `kit_config` and the compilers all take it and none of them may depend on the
savefile crate (or on `studio_core`). Version-specific *facts* never live here: an offset, a key
or a path is data in the crate that owns the format, looked up by version, never a `match` on
`PesVersion` outside that crate.

## Format references in converters

| Format | Source | Size | Notes |
|--------|--------|------|-------|
| .model (PreFox) | `ModelFile.py` | ~1700 lines | Binary format, struct.unpack → binrw |

## Recommended approach

Use `binrw` for typed, declarative binary readers and writers. Derives catch type errors, not
incorrect format layouts: wrong offsets, field widths, and endianness can compile successfully.
Verify those against real fixtures and independently decoded fields; a reader and writer can share
the same mistake and still pass a roundtrip.

```rust
#[derive(BinRead)]
#[br(little, magic = b"FMDL")]
struct FmdlHeader {
    version: u32,
    descriptors_offset: u64,
    section0_bitmap: u64,
    section1_bitmap: u64,
    section0_block_count: u32,
    section1_block_count: u32,
    section0_offset: u32,
    section0_length: u32,
    section1_offset: u32,
    section1_length: u32,
}
```

### Layout of the format crates: `format/` and `ops/`

The single-concern format crates (`cpk`, `fpk`, `ftex`, `fox2`, `uniparam`, `wezlib`, `kit_config`)
need no prescribed layout — one module tree, read and write, tests. `fmdl` and `pes_model` are
different: each carries **format-native algorithms** on top of the codec (mesh splitting, split
vertex encoding, anti-blur decode/encode, multi-model merging, path editing) and has a **second
consumer outside the workspace** (the Blender extension via `python_bindings`). Both crates use the
same two-level split so that boundary is visible in the tree:

```
crates/libs/fmdl/src/                 crates/libs/pes_model/src/
├── lib.rs                            ├── lib.rs
├── format/     # the codec           ├── format/
│   ├── mod.rs  #  FmdlFile           │   ├── mod.rs   # PreFoxModel
│   ├── header.rs, sections.rs, …     │   ├── model.rs # .model blocks
│   └── write.rs                      │   ├── mtl.rs   # sibling .mtl XML
│                                     │   └── write.rs
├── ops/        # algorithms          ├── ops/
│   ├── split.rs    # mesh splitting  │   ├── split.rs
│   ├── vertex_enc.rs # split vertex  │   ├── vertex_enc.rs
│   ├── antiblur.rs                   │   ├── merge.rs
│   ├── merge.rs    # multi-FMDL      │   └── paths.rs  # texture stem rewriting
│   └── paths.rs    # path table edit └── check.rs    # deep validation → findings
└── check.rs    # deep validation
```

Rules:

- **`format/` is pure binrw**: structs, read, write, and nothing that changes a model's meaning. A
  round-trip through `format/` alone is byte-identical (tested).
- **`ops/` are format-native transformations** for lossless same-format work; they operate on the
  crate's own types, never on `model_convert`'s IR (the dependency points the other way).
- **`check.rs` returns findings, not messages** — stable codes with context, mapped to catalog
  entries by the consuming tool, mirroring `aesthetics_export`'s rule.
- **The `python_bindings` surface is `format/` + `ops/`**, exposed as-is; if a function is awkward to
  expose, that is a hint it belongs in the Team compiler rather than here.
- Core plan guardrail 4 (PyO3-buildable, dependency denylist) applies to the whole crate; the split
  does not relax it for `ops/`.

### `fmdl::model`: the semantic layer the ops work on

`format/` is records and buffers; the ops (splitting, anti-blur, merging, path editing) reason
about bones, materials, meshes and groups. `model.rs` is the layer between them: a `Model` built
from an `FmdlFile` and written back to a fresh one. It is not byte-identical (a fresh file is laid
out from scratch); it is *semantically* lossless: `Model::from_file(&FmdlFile::read(x)?)?.to_file()`
read back gives an equal `Model`, on Konami files as well as add-on-written ones (the legacy writer
could not rebuild Konami files at all; that is the bar this layer clears).

```rust
pub struct Model {
    pub bones: Vec<Bone>,
    pub materials: Vec<MaterialInstance>,
    pub meshes: Vec<Mesh>,
    pub mesh_groups: Vec<MeshGroup>,
    /// The `X-FMDL-Extensions` header: which encodings the file declares.
    pub extensions: Extensions,
    /// Section-1 block 1, 64 bytes per bone in Konami files; carried as is (the add-on writes it empty).
    pub bone_matrices: Option<Vec<u8>>,
}
pub struct Bone { pub name: String, pub parent: Option<usize>, pub bounding_box: BoundingBox, pub local_position: [f32; 4], pub world_position: [f32; 4] }
pub struct BoundingBox { pub max: [f32; 4], pub min: [f32; 4] }
pub struct Texture { pub file_name: String, pub directory: String }
pub struct MaterialInstance {
    pub name: String, pub shader: String, pub technique: String,
    /// (sampler name, texture), in file order.
    pub textures: Vec<(String, Texture)>,
    /// (parameter name, four floats), in file order.
    pub parameters: Vec<(String, [f32; 4])>,
}
pub struct Mesh {
    pub vertices: MeshVertices,          // the codec's type; bone indices index `bone_group`
    pub faces: Vec<[u16; 3]>,
    pub bone_group: Vec<usize>,          // indices into `Model::bones`, at most 32
    pub material: usize,                 // index into `Model::materials`
    pub alpha_flags: u8,
    pub shadow_flags: u8,
    pub has_antiblur_meshes: bool,       // per-mesh extension headers
    pub is_antiblur_mesh: bool,
    pub custom_bounding_box: Option<BoundingBox>,
}
pub struct MeshGroup { pub name: String, pub parent: Option<usize>, pub meshes: Vec<usize>, pub bounding_box: Option<BoundingBox>, pub visible: bool, pub split_mesh_group: bool }
pub struct Extensions { pub mesh_splitting: bool, pub antiblur: bool, pub vertex_loop_preservation: bool }
impl Model {
    pub fn from_file(file: &FmdlFile) -> Result<Model, FmdlError>;
    pub fn to_file(&self) -> Result<FmdlFile, FmdlError>;
}
```

`from_file` resolves every index through the tables (strings, bounding boxes, bone groups,
materials, textures, parameter assignments, mesh-group assignments) and reads the extension
headers from the string table's tail (`X-FMDL-Extensions:` plus per-object headers such as
`Has-Antiblur-Meshes: 0,3` listing mesh indices, `Split-Mesh-Groups: 2` listing group indices,
`Custom-Bounding-Box-Meshes`), the `key: value, value` grammar the add-ons write. Every dangling
index is an error, never a panic. `to_file` lays a file out the way the add-on writer does, which
years of add-on-written models prove PES accepts: positions in buffer 0 (stride 12), the other
attributes interleaved in buffer 1 in the order normal, tangent, color, bone weights, bone indices,
uv maps (a uv map identical to an earlier one shares its offset), faces in buffer 2; strings
de-duplicated; a bounding box per bone, mesh group and mesh (computed from the vertices when the
source had none, which is why the legacy writer failed on Konami files); one level-of-detail record;
the fixed blocks 18 and 20 as the add-on writes them; the extension headers re-emitted after the
last string. The codec's per-mesh vertex kinds (normal, tangent, color, bone mapping, uv count and
precision) are exactly what `MeshVertices` carries, so no separate "vertex fields" record exists.

## External tool dependencies

| Tool | Current use | Rust replacement |
|------|-------------|-----------------|
| `7z.exe` | Extract `.7z` exports | `sevenz-rust` crate (native, no external binary) |
| `texconv.exe` | DDS DXT5 conversion (rare) | **Dropped** — in-process conversion (see below) |
| `magick` (ImageMagick) | Texture conversion on Linux | **Dropped** — same in-process path on all platforms |

The `sevenz-rust` crate eliminates the 7z subprocess and temp-folder extraction dance.

## In-process DDS conversion (replaces texconv)

The PES Stadium Compiler (`Tools_4cc/pes-stadium-compiler-v1.0.0`) already proves the in-process
approach in production, in `Engines/stages/lib/dxt.py` + the `texture2ddecoder` package:

- **Decoding (DDS/FTEX)**: `block_compression`'s `decode` module (BC1–BC5, BC7), the same crate
  that encodes, so one dependency covers both directions; the `texture2ddecoder` crate the plan
  first named is not used. `dds_convert`/`ftex` handle the DDS/FTEX containers and normalize
  decoded channel order to RGBA. Parity is verified against DirectXTex `texconv`, the reference
  decoder: the `dds_convert` fixtures are texconv-encoded BC1/BC3/BC5/BC7 files with texconv's own
  decode of every mip as the expected output, and the tests require an exact match.
- **Decoding (raster sources)**: the `image` crate decodes all accepted raster formats to RGBA
  in pure Rust (no C dependencies). See the format matrix below.
- **Encoding**: use `block_compression`'s Rust CPU backend for BC1, BC3, and BC7, rather than
  translating the stadium compiler's numpy encoder. Codec selection follows the PES-version and
  texture-role rules below; desktop GPU BC7 is included in the first release with CPU fallback.
- **DXT5nm handling**: a normal map that has to be encoded (a BC5 source on PES 15–18, or any
  normal-role source that is not already BC3, raster and BC7 included, on every version) becomes
  BC3 with X in alpha and Y in green, which is what both
  engines ship: every Konami normal map measured is BC3 with `A = X`, `G = Y` (PES 17 `oral_nrm`,
  `bibs_nrm`, `skin_nrm`; PES 21 `dummy_nrm`). The two engines differ only in the channels the
  shader ignores, and the output copies each engine's own files rather than guessing what the
  shader reads: pre-Fox `R = G = B = Y` (the color block is a grey Y), Fox `R = 255`, `B = 0` (the
  DirectXTex `DXT5nm` layout). The legacy compilers never produced this conversion at all: they
  invoked texconv with a format name it rejects (`DX5nm`), and the current texconv drops X when the
  source is BC5, so there is no legacy output to be parity with; the Konami files are the standard.
- **Passthrough**: compressed blocks a target can read are kept and only the container changes.
  PES 15–18 read BC1, BC2 and BC3; PES 19–21 also read BC4, BC5 and BC7 (the legacy compilers
  passed BC5 through to PES 19–21 as FTEX format 9). The role narrows this: a normal-role source
  passes through only as BC3 (taken to be laid out already) or, on PES 19–21, as BC5; a
  normal-role BC7 or BC1 source is decoded and encoded to DXT5nm, because keeping its blocks would
  keep a color layout the shader reads as a normal map. A DX10-header BC1/BC3 is therefore rewritten
  with a legacy header for PES 15–18 rather than re-encoded (the legacy compilers re-encoded every
  DX10 header to DXT5, a lossy step with no purpose). Uncompressed sources are always encoded:
  PES 15–17 crash on uncompressed kit textures.
- **Mipmaps**: each mip level is decoded and re-encoded individually; a DDS/FTEX source keeps its
  own mip count whatever its codec, uncompressed included (Konami ships single-mip pre-Fox
  textures, and a mipped non-power-of-two texture is invalid on Fox), a raster source gets the
  full chain down to 1×1 by 2×2 box averaging on
  straight alpha (odd sides floor, never below 1). Output headers are rebuilt from the emitted
  layout: legacy DX9 headers for BC1/BC3 (what texconv wrote for the legacy compilers and what
  Konami ships), a DX10 header for BC7.
- **FTEX texture type**: every Fox output is written with texture type `0x9`, the value the legacy
  compilers wrote for every texture, color and normal alike, for years of PES 19–21 exports; Konami
  uses `0x1`/`0x3` for color textures, but the parity standard is what is known to render.

This covers the actual conversion cases (DX10 BC7 → DXT5, BC5/ATI2 → DXT5nm) that `texconv -f
DXT5` was handling, or failing to handle, in practice. Benefits over the subprocess approach:

- No Windows-only binary; identical behavior on Linux (removes the ImageMagick fallback path
  entirely)
- No temp-folder write/read roundtrip; operates on in-memory buffers, matching the VirtualTree
  pipeline
- Much faster for batches (no process spawn per texture)

Any texture in a format outside the accepted set fails with a clear error asking the user to
resave it — the same behavior the current compilers have for unsupported codecs.

### Target codecs and CPU encoder

**BC7 is supported only by PES 19–21.** PES18 uses Fox models/FTEX but does not support BC7;
model-engine generation alone is therefore not the texture-codec selector.

| Target PES | BC7 input | Newly encoded ordinary raster/color input |
|---|---|---|
| 15–18 | Transcode to BC3, or BC1 when eligible below | BC3, or BC1 when eligible below |
| 19–21 | Retain compatible BC7 blocks | BC7 |

Keep already-compatible compressed blocks when only DDS/FTEX container/header adaptation is needed.
Normal/data textures retain their role-specific channel and codec requirements.

**BC1 optimization:** use BC1 for fully opaque ordinary color textures (alpha 255 in every emitted mip)
that do not require alpha as a data channel. It uses 8 bytes per block instead of BC3's 16 and uses
compatible four-color RGB coding, but neither encoding is lossless relative to raster/BC7 sources.
“No semitransparent pixels” alone is insufficient: BC1's one-bit transparency reduces affected
blocks to three visible colors, and generated mips may introduce intermediate alpha. Keep such
textures in BC3 for the older targets rather than promising unchanged quality. DXT5nm keeps BC3:
its alpha stores a normal component, not opacity.

**Selected encoder:** `block_compression` (reviewed release **0.10.0**) supplies both the CPU
reference/fallback and first-release desktop GPU BC7 backend. Use `default-features = false`,
with `bc15`/`bc7` and desktop `wgpu` enabled; BC6H is not needed. CPU-only builds remain possible,
and browser GPU acceleration is deferred. Use basic opaque/alpha BC7 presets initially, preserving
alpha when required; tune against representative textures, not assumed timings. `texpresso` lacks
BC7, while `intel_tex_2` adds native ISPC kernels; one library serves both paths without maintaining
our own encoders. This is a best-fit selection, not a measured claim of optimal throughput.

Before integration, check decode/quality parity, repeatability, and native/WASM builds. Exercise
PES18 versus PES19 output, opaque versus alpha-bearing textures, DXT5nm, and the complete mip chain;
the CPU API requires 4-aligned input dimensions, so edge padding must also handle 1×1/2×2 tail mips
without changing their logical DDS/FTEX dimensions.

### Accepted image formats

All image formats are interchangeable as texture sources for any model format (FMDL, .model,
glTF). Material references name textures by stem (filename without extension); the compiler
resolves each stem to whichever file exists in the model folder and converts to the target
format. Two image files with the same stem but different extensions is a conflict
(`texture_stem_conflict`).

**Accepted:**

| Format | Decoder | Extensions | Notes |
|--------|---------|------------|-------|
| DDS | `block_compression::decode` | `.dds` | BC7, BC5, BC4, BC3/DXT5, BC2, BC1/DXT1, uncompressed 32-bit |
| FTEX | `ftex` crate | `.ftex` | Fox Engine native texture |
| PNG | `image` crate | `.png` | Pure Rust |
| JPEG | `image` crate | `.jpg`, `.jpeg` | Pure Rust |
| BMP | `image` crate | `.bmp` | Pure Rust |
| WebP | `image` crate | `.webp` | Pure Rust |
| TGA | `image` crate | `.tga` | Pure Rust |
| TIFF | `image` crate | `.tif`, `.tiff` | Pure Rust; Photoshop users |

**Not accepted** (enable only the accepted decoders explicitly; the `image` crate's default
features and AVIF backends vary by release):

| Format | Reason |
|--------|--------|
| AVIF | Outside the accepted source set; decoder/backend portability is not part of the initial scope |
| GIF | Animation format; not suitable for static textures |
| ICO | Windows icon container; not a texture format |
| HDR (Radiance RGBE) | HDR photography; not used in game modding |
| OpenEXR | Film/VFX HDR; not used in game modding |
| Farbfeld | Obscure pure-Rust format; no community use |
| PNM (PPM/PGM/PBM) | Simple academic format; no community use |
| QOI | Modern pure-Rust format; no community adoption yet — can be added if demand arises |

### `dds_convert` API

The crate is three pure steps and one stateful wrapper. `decode` turns any accepted source into
straight-alpha RGBA8 mips plus, for DDS/FTEX, the compressed blocks it carried; `convert` applies
the codec rules above to a decoded texture and returns the finished container bytes (a DDS for
PES 15–17, an FTEX for PES 18–21); `Converter` is the session cache in front of both.

```rust
/// The accepted source formats (table above), named by the file extension the
/// compiler resolved the texture stem to. TGA has no magic, so the format is
/// never sniffed from bytes.
pub enum SourceFormat { Dds, Ftex, Png, Jpeg, Bmp, WebP, Tga, Tiff }
impl SourceFormat {
    /// Case-insensitive, without the dot; `jpg`/`jpeg` and `tif`/`tiff` both accepted.
    pub fn from_extension(extension: &str) -> Option<Self>;
}

/// What the texture is for; with the PES version it selects the codec and the channel layout.
pub enum TextureRole { Color, Normal }

/// Block codecs this crate keeps or emits.
pub enum BlockCodec { Bc1, Bc2, Bc3, Bc4, Bc5, Bc7 }

/// Compressed blocks a DDS/FTEX source carried, one buffer per mip, kept so a
/// target that reads the codec gets them unchanged.
pub struct Blocks { pub codec: BlockCodec, pub mips: Vec<Vec<u8>> }

/// A source decoded to straight-alpha RGBA8, top mip first, every mip the source carried.
pub struct Decoded { pub width: u32, pub height: u32, pub mips: Vec<Vec<u8>>, pub blocks: Option<Blocks>, pub authored_mips: bool }

pub struct Target { pub version: PesVersion, pub role: TextureRole }

pub fn decode(bytes: &[u8], format: SourceFormat) -> Result<Decoded, ConvertError>;
pub fn convert(decoded: &Decoded, target: Target) -> Result<Vec<u8>, ConvertError>;

/// SHA-256 of the source bytes, computed once when the file is materialized and reused.
pub struct SourceHash(pub [u8; 32]);
pub fn source_hash(bytes: &[u8]) -> SourceHash;

pub enum CachePolicy { Use, Bypass }

/// The session cache: finished container bytes by (source hash, source format, target).
pub struct Converter { /* Mutex<HashMap<CacheKey, Arc<[u8]>>> */ }
impl Converter {
    pub fn new() -> Self;
    pub fn convert(&self, hash: SourceHash, bytes: &[u8], format: SourceFormat, target: Target, cache: CachePolicy) -> Result<Arc<[u8]>, ConvertError>;
    /// Bytes currently retained, for the pipeline's memory budget.
    pub fn retained_bytes(&self) -> usize;
    pub fn clear(&self);
}
```

`decode` rejects cube maps, volume textures, texture arrays and the signed BC4/BC5 formats
(`ConvertError::Unsupported`): nothing in an export is one, and a signed block relabelled unsigned
would silently change a normal map. An uncompressed DDS whose header declares a row pitch wider
than its rows (some exporters pad rows to 4 bytes) is read at that pitch, each lower mip at the
same 4-byte row alignment. `Decoded.authored_mips` says whether the source format carries a mip
chain (DDS/FTEX, its level count is kept) or not (raster, a chain is generated). A WESYS-wrapped DDS (PES 15–17 sources) is unwrapped first through `wezlib`. The DDS
header knowledge (`DdsHeader`, `Dx10Header`, the FourCC/mask/DXGI table) lives in `ftex`, which
already needed it in both directions; `ftex` exposes it as `ftex::dds` (`read_layout`,
`header_bytes`) and `dds_convert` uses that instead of carrying a second copy. Encoder settings
join the cache key with 2.5b, when a second encoder exists.

### In-memory conversion cache

CPU texture conversion is reusable for a given source hash and complete conversion configuration
(the key below), so the Studio caches converted output bytes in memory across compiles. This targets the edit→compile→test loop — the hot path, where
the same export is compiled dozens of times with minor edits to one or two files. While an entry is retained,
unchanged textures reuse its converted bytes; eviction may require conversion again.

**Design:**
- A `Converter` struct in `dds_convert` holds an internal `HashMap<CacheKey, Arc<[u8]>>` mapping
  to the finished DDS/FTEX bytes (not the decoded RGBA — the output is what the packing step
  needs).
- The cache key includes the source content hash and every output-affecting conversion choice:
  target codec/container, texture role and channel swizzle, mip/color-space settings, and encoder
  configuration. The source hash is computed at materialization and reused; DXT5 color data and
  DXT5nm normal data must not alias merely because they share a codec.
- The Studio creates one `Converter` at startup and passes it into each Team compiler run. The
  cache lives for the application session and is dropped on exit — **no disk persistence**, so
  it never fills the user's drive with temporary files.
- Cache misses fall through to the normal decode → mipmap → encode path. The cache is
  consulted after the build manifest has determined the target codec for each texture, so the
  same source PNG cached as DXT5 for a pre-Fox compile is not wrongly reused for a Fox BC7
  compile (different `target_codec` → different key → separate entry).
- **Engagement limit:** the cache engages only when compiling **≤2 teams**. The build manifest
  knows the team count at planning time; for larger compiles (3+ teams, e.g. a 48-team DLC),
  the cache is bypassed entirely — entries are never inserted. Rationale: a large compile has
  mostly first-seen textures (no cache benefit), and caching thousands of converted DDS/FTEX
  buffers would consume a sizeable amount of memory for no gain, hurting parallelism by
  reducing the budget available to worker tasks. The ≤2 team threshold covers the typical
  edit→test scenario (one or two teams) while keeping large compiles from adding cache entries.
- **Retention is separately bounded and budgeted.** Repeated edits or switching between teams must
  not accumulate every historical conversion for the whole session. Retained bytes count against
  the shared memory budget even after a task finishes; evict reusable cache entries under pressure
  so they cannot prevent working tasks from acquiring memory. Bypassing the cache for a large run
  does not by itself release entries retained by earlier small runs.

**What it covers and doesn't:** repeated one/two-team compiles benefit when their working set
fits the cache. A fresh full-cup compile bypasses it and must pay for every required conversion.
Benchmark both workloads with the selected codecs, representative textures, and intended worker
count; a cache does not establish a fixed compile-time guarantee.

### PNG-texture trajectory

Blender cannot write DDS natively, and the ported addons convert DDS→PNG on import. As the
community migrates to glTF authoring (the plan's end-state path), PNG becomes the norm for
source textures, not the corner case. The conversion pipeline is designed for this: the `image`
crate handles decoding, mipmap generation fills the chain that raster sources don't carry, and
the in-memory cache (above) keeps the edit→compile→test loop fast. The worst-case first-compile
cost is accepted as the price of dropping the texconv.exe dependency and enabling glTF authoring
— strictly better than the current workflow (manual texconv runs or no PNG support at all).

---

## `libs/color_tools`

One small crate for the suite's color work: kit-color extraction (pure logic) plus
the shared color-picker widget (egui). Splitting the widget into its own crate isn't
worth the boilerplate at this size; if the egui dependency ever bothers a headless
consumer, the widget module can move behind a feature flag.

### Dominant kit-color extraction

Kit colors (the two per-kit menu colors compiled into `UniColor.bin`) matter: they
drive the match UI scoreboard and the kit-selection color dots, which is what lets
teams pick non-clashing kits when a preview player's custom body hides the actual
kit. Managers often don't bother providing them, so the Team compiler derives them
from the kit's main texture when a kit folder has no `colors.txt` (see the Team
compiler plan's kit steps and `kit_colors_derived` message).

The extraction:

1. **Decode** the kit's main texture (`kit.dds`) to RGBA via `dds_convert`, top
   mip only.
2. **Sample fixed template regions.** The community kit template has a fixed UV
   layout (the same for every kit — the config's "shirt model" field does not
   affect it, see the Kit config editor plan), so two coarse normalized rectangles
   are enough:
   - **Shirt**: the center vertical band (front + back), ≈ x 0.34–0.66,
     y 0.02–0.90.
   - **Shorts**: the two lower side panels, ≈ x 0.02–0.31 and 0.69–0.98,
     y 0.59–0.90.
   The exact rectangles are lib constants, calibrated against a set of real kit
   textures during implementation (insetting them a little keeps trim/seam pixels
   out).
3. **Cluster** each region's pixels (coarse RGB histogram quantization, then merge
   near-identical bins — cheap and deterministic; sponsor logos and badges end up
   in small clusters and lose automatically).
4. **Pick**:
   - Color 1 = the shirt region's largest cluster.
   - Color 2 = the shirt region's second cluster **if** it holds a meaningful
     share of the region (two-tone kit; threshold calibrated, ~25%) **and** is
     perceptually distinct from color 1 — otherwise the shorts region's largest
     cluster.

The same routine powers suggestion swatches in GUI tools, so it returns the full
ranked cluster list per region, not just the two winners.

### Color-picker widget

egui ships a bare HSVA picker (`egui::color_picker`); this widget wraps it into the
suite's standard picker popup: hex entry (`#RRGGBB`, matching the export text
formats), RGB fields, suggestion swatches (e.g. the extraction results for the kit
being edited), and recent colors. The widget also reports a **hover candidate**:
while a swatch is hovered or the picker is dragged, the host gets the candidate
color to live-preview in its own UI (the Kit config editor tints its menu-UI
mockup and icon gallery with it), reverting when the hover ends without a click.
Used by the Kit config editor (config colors and
kit `colors.txt` colors) and by the Team creator; any later tool that
needs a color field uses it too.

### Kit menu icon rendering

The 24 kit menu icons (`icon.txt`, 0–23) are simple two-color kit icon patterns —
plain, striped, hooped, sashed, halved, contrast-sleeves, and so on. In game they
only ever appear in the PES 15/16 prematch gameplan screens, but suite-side they
make a great compact kit-color visual, so the crate recreates them as **vector
draw routines** (egui painter) parameterized by the icon number, a size, and the
kit's two menu colors — no game bitmaps shipped, tintable with any colors, crisp
at any size. Two render modes: the **full shirt shape** (the editor's icon
gallery) and **pattern-only** — just the two-color fill pattern as a flat square
swatch, for sizes where a shirt silhouette would turn to mush (the Team
compiler's grid cells). The pattern shapes are transcribed from the PES 15
settings reference sheet (`Kiticons.png`, kept with the crate's test data).

Consumers: `team_compiler` (extraction fallback; grid kit-cell miniatures),
`kit_config_editor` (picker + swatches; icon gallery), the Team creator.

---

## `libs/elevation`

Admin rights for the tools that need them, in one small platform crate. Compiling
into the PES install directory fails on default Windows installs when PES lives
under `Program Files` — the compilers' normal output target — so all compiler
tools that write to the PES install directory need an elevation path. The desktop updater can
also need it for a protected program directory. The match tracker normally reads without
elevation, but uses the same helpers when process permissions require it.

- **Manifest execution level**: the `studio` binary ships `asInvoker`; a
  `requireAdministrator` cargo feature flips the manifest for always-elevated
  setups.
- **Detect + relaunch**: `is_elevated()` plus an elevated relaunch of the current
  command line (`ShellExecuteExW` with `runas`, i.e. the UAC consent prompt) — the
  GUI's "run as administrator" action and Red's `admin_tools.py` replacement. The
  relaunch re-attaches to the same workspace state via ordinary startup (settings
  file, CLI args preserved).
- **CLI behavior**: an access-denied output path fails cleanly — a clear error and
  a non-zero exit — instead of half-written CPKs.

POSIX builds get euid checks and a sudo hint in place of UAC. Consumers:
`team_compiler`, `stadium_compiler`, `balls_compiler`, `match_tracker` (when access requires it),
and the desktop updater.

---

## `libs/fpc`

Full Player Customization is one *system* that spans two formats: kit configs (every kit of the
team, GK included, must carry shirt model 176, shorts model 16, collar 105, winter collar 105) and
the savefile's player appearance (the hide preset: long sleeves, tucked shirt, short socks, a
nonexistent boots ID such as 55, a nonexistent gloves ID such as 11; the un-hide preset: short
sleeves, untucked, standard/long socks, boots 0, gloves 0 or 1–10 for keepers; the partial-hide
preset for a custom body inside a stock jersey: short sleeves, untucked, standard socks, skin color
Custom). Neither `kit_config` nor `pes_savefile` is the natural owner, and having each hold half
would either duplicate the knowledge or make one format crate depend on the other. So the system's
knowledge is a **leaf crate with no dependencies**, holding data and pure rules only:

- `kit.rs` — the kit-config FPC values per PES version (the modern system: PES 19+ and the 2024
  reimplementation for 16/17; the retro 16/17 system is documented as legacy and not supported).
  `kit_values(version)` returns the same four values (shirt model 176, shorts model 16, collar
  105, winter collar 105) for PES 16, 17, 19, 20 and 21 and `None` for PES 15 and 18, where no FPC
  system exists. The wiki page documents the PES 17 values and calls the PES 19+ system "mostly
  identical"; that the four values are the same on PES 19+ is to be confirmed against a PES 21 FPC
  kit config when `kit_config` lands (Phase 2.13).
- `player.rs` — the three appearance presets above, expressed in the crate's own small vocabulary
  (`Sleeves`, `Tuck`, `Socks`, boots/gloves IDs, skin color), not in `pes_savefile` field terms.
- `interference.rs` — the settings that break or bend FPC, as findings with severity: inners ≠ None
  and undershorts ≠ Off/Off break hiding (error on an FPC player); wrist/ankle taping ≠ None
  selectively shows pieces (info — deliberate in custom setups); the Gloves checkbox with gloves ID
  0 causes winter gloves in winter conditions (warning); skin color Custom on a non-FPC player hides
  the body but not the jersey (warning).

Consumers: `kit_config` (`apply_fpc` / `matches_fpc` use `fpc::kit`; there is no revert, since the
template config already carries the FPC values and the compiler never auto-reverts them), `pes_savefile`
(`ops/fpc.rs` maps `fpc::player` presets onto `PlayerEntry` and runs the interference check), and
through them the Team compiler (`fpc.on`/`fpc.off` markers, kit reconciliation, `settings.toml`
validation), the Save editor (FPC toggle, Appearance-tab warnings), the Kit config editor (FPC
indicator), and the Player aesthetics editor (marker toggles). The wiki's FPC guide
(`resources/FPC.wikitext` in this repo) is the source text; each constant carries a doc comment pointing at
its line there.

---

## `libs/teams_list`

The 4cc team identity space and the file that records it, in one **leaf crate with no
dependencies beyond `thiserror`**. Three consumers need it and none is a natural owner: the Team
compiler resolves export names against it, the Save editor writes it from a savefile ("Export
teams list"), and the `studio` binary merges a new release's embedded list into the working copy
after an update. It used to be a module of `aesthetics_export`, which made the Save editor and the
updater depend on the whole export object model for one file format.

- `name.rs` — `TeamName`: the canonical `/xx/` form. `TeamName::new(token)` applies **the** fold
  (Unicode `str::to_lowercase`, wrap in slashes; empty token rejected) — the one place the rule
  lives, so the export-name side and the list side cannot drift. `is_referees()` / `is_balls()` /
  `is_reserved()`. Splitting an export display name into its first token is export-format
  knowledge and stays in `aesthetics_export::identity`, which calls `TeamName::new`.
- `id.rs` — `TeamId`: validated 701–920. Referees' fixed 999 is not a `TeamId` (see the Team
  compiler plan, "Export identity resolution").
- `file.rs` — `TeamsList`: parse / write `teams_list.txt` per the contract in the Team compiler
  plan ("Export identity resolution" and the "`teams_list.txt` contract" open question): tab-split,
  `ID` and `Name` columns located by header label in any order, every other cell carried verbatim
  and written back in place, Name folded through `TeamName::new` on load, rows that do not fold to
  a slash-wrapped name kept verbatim as inert placeholders whose numeric ids still count in the
  duplicate check; lookup by `TeamName`; the embedded upstream list as a `const`.
- `reconcile.rs` — the merge of an incoming list (embedded upstream, or a savefile's team table)
  into the working list: rows only in the incoming list added, rows only in the working list kept,
  conflicts (same name, different ID) taken from the incoming list, an incoming team whose ID a
  placeholder holds taking that slot, then uniqueness of IDs validated on the *final* mapping (so
  two teams swapping IDs merge cleanly) with every incoming change behind a remaining collision
  reverted; returns a `MergeSummary` (added / kept / overridden / unresolved) for the caller to
  show before writing. Never writes: callers own filesystem I/O and the read-only handling
  (`teams_list_read_only`).

Consumers: `aesthetics_export` (identity resolution), `team_compiler` (ID cell write),
`save_editor` (`export-teams-list`), `studio` (post-update merge). `wasm32`-clean. Tested against
Red's current `teams_list.txt` as a fixture (220 rows, `/umaJP/` mixed case, `/@/`, the `Backup N`
placeholders) plus reconcile cases for each summary bucket.

---

## Testing: format parser unit tests

Each binary format parser has unit tests with real file samples. Fixtures live in the crate's own
`tests/fixtures/` (modularity over a shared tree; a small sample needed by several crates is
copied, a large one is not duplicated without a decision). Real PES-derived files may be committed,
kept minimal so cloning stays fast: the smallest file that exercises the code path, one per format
variant rather than one per team, an FPK or FMDL rather than the CPK that contained it.
```rust
#[test]
fn cpk_roundtrip() {
    let original = parse_cpk("test_data/sample.cpk");
    let bytes = serialize_cpk(&original);
    let reparsed = parse_cpk_bytes(&bytes);
    assert_eq!(original, reparsed);
}
```

`color_tools` extraction is tested against a set of real kit textures with
manager-provided `colors.txt` entries as ground truth: the derived pair doesn't
have to match exactly, but must land within a perceptual-distance tolerance of
the human-picked colors (also the calibration harness for the region rectangles
and thresholds).

---

## First-release desktop GPU BC7

GPU BC7 for PES 19–21 is a first-release target, using `block_compression`'s existing wgpu backend
inside `dds_convert`. The library supplies the kernels; Studio implements bounded batching,
upload/readback, pipeline integration, and fallback. It can remain the permanent GPU backend.
BC1/BC3 stay on the CPU initially; already-compatible compressed inputs and cache hits need no
encoding at all.

- **Auto / CPU:** Auto is the desktop default, using the GPU when ready, supported, and beneficial.
  CPU mode explicitly selects the deterministic reference. Unsupported/unready GPUs and recoverable
  GPU failures fall back to CPU; report the active backend and fallback reason. The CLI can obtain
  a headless compute device without opening the GUI.
- **Compression backend:** prefer Vulkan on Windows/Linux and Metal on macOS. Studio selects the
  wgpu device/queue passed to `GpuBlockCompressor`; this does not require a different encoder or
  custom kernels. If that backend is unavailable or unsuitable, use CPU, with no automatic DX12
  fallback initially. The library documents slow BC7 pipeline creation specifically under DX12's
  DXC shader compiler; do not reintroduce that path through fallback. This policy concerns texture
  compression, not a requirement to change the GUI's rendering backend.
- **Cold-start behavior:** measure cold pipeline creation on Vulkan/Metal and upload/readback cost
  early. Prepare off the UI thread, reuse the compute device and pipelines, and allow CPU work while
  GPU preparation is pending. A cold first compile and end-to-end throughput matter more than
  kernel-only numbers; avoiding the documented DX12 issue does not guarantee fast startup on every
  Vulkan/Metal driver.
- **Bounded work:** coordinate GPU batches with the existing memory budget, cancellation, and writer
  progress. Read compressed blocks back for packing; avoid unbounded submission from rayon workers.
  All consumers of a shared texture conversion reuse one encoded result, including after fallback,
  rather than creating conflicting CPU/GPU copies of the same output.
- **Integration constraints:** use a compatible suite-wide wgpu version; query adapter capabilities
  and limits, not Windows display enumeration. Preserve raw channel values and mip dimensions when
  uploading, including the API's non-sRGB view requirement and block-aligned edge padding.
- **Verification:** both implementations need tests against real opaque/alpha/normal-map fixtures.
  Check supported target versions, startup, full-path timing, resource pressure, cancellation, and
  fallback on representative hardware. No measured speedup is assumed in this plan.

**Reproducibility:** explicit CPU mode remains the byte-reproducible reference. The user accepts
valid GPU-encoded texture bytes differing across GPUs/drivers; containing CPKs may differ as a
consequence. GPU output must still satisfy decoded-content/quality checks. Asset paths, model/bin
output, ordering, and collision decisions remain deterministic. The cache distinguishes encoder
implementations/configurations and invalidates on device changes.

## Future Features

### Custom GPU optimization and browser acceleration

Custom maximum-throughput kernels are considered only if measurements expose a worthwhile gap in
the library backend. Possible algorithm references include AMD Compressonator and bc7e, subject to
normal license and quality checks. Browser GPU acceleration remains deferred; its CPU path remains
available. Neither requires a second bespoke encoder for the initial desktop release.
