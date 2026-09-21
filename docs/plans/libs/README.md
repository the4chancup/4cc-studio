# 4cc Studio — Library crates plan

Covers the small shared lib crates: the binary format parsers (`cpk`, `fpk`, `fmdl`,
`pes_model`, `ftex`, `uniparam`, `wezlib`), `dds_convert`, `color_tools`,
`vtree`, `pes_version`, `archives`, `elevation`, `fpc`, and `teams_list`. The aesthetics export format's object model, folder conventions,
and validation live in `aesthetics_export` (shared by the Team compiler, Export upgrader,
Kit config editor, Refs arranger, and Team creator), specified in the
[Team compiler plan](../team_compiler/README.md). The two big libs have their own plans:
[Model conversion](../model_conversion/README.md) (`model_convert` + the mesh algorithms in
`fmdl`/`pes_model`) and [Savefile](../pes_savefile/README.md) (`pes_savefile`). The `pipeline`
crate (reader/writer scaffolding, memory budget, folder watcher, check cache, shared `CpkStem`
validation) is
specified across the [core plan](../core/README.md) (Parallelism) and the
[Team compiler plan](../team_compiler/README.md) (walkthrough, live validation). The music lib
crates (`music_export`, `audio_engine`) are specified in the
[Music player plan](../music_player.md), the live-match event feed (`match_feed`)
in the [Match tracker plan](../match_tracker/README.md), the kit config codec
(`kit_config` — binary/TOML, reverse-engineered format documentation) in the
[Kit config editor plan](../kit_config_editor.md), the AATF rules engine
(`aatf`) and the shared tactics/card widgets (`team_widgets`) in the [Save editor
plan](../save_editor.md), the fox2 parser in the
[Stadium compiler plan](../stadium_compiler.md), and the Konami database tables (`pesdb`) in
the [DB generator plan](../db_generator.md). Platform context is in the
[core plan](../core/README.md).

---

The plan is split into parts; section headings are unchanged from the
single-file plan.

| Part | Covers |
|---|---|
| [Format crates](format_crates.md) | Recommended approach |
| [fox2](fox2.md) | `libs/fox2`: Fox Engine entity files |
| [archives](archives.md) | `libs/archives`: reading `.zip` and `.7z` exports |
| [dds_convert](dds_convert.md) | In-process DDS conversion (replaces texconv) |
| [color_tools](color_tools.md) | `libs/color_tools` |
| [elevation](elevation.md) | `libs/elevation` |
| [fpc](fpc.md) | `libs/fpc` |
| [teams_list](teams_list.md) | `libs/teams_list` |

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

## External tool dependencies

| Tool | Current use | Rust replacement |
|------|-------------|-----------------|
| `7z.exe` | Extract `.7z` exports | `sevenz-rust` crate (native, no external binary) |
| `texconv.exe` | DDS DXT5 conversion (rare) | **Dropped** — in-process conversion (see below) |
| `magick` (ImageMagick) | Texture conversion on Linux | **Dropped** — same in-process path on all platforms |

The `sevenz-rust` crate eliminates the 7z subprocess and temp-folder extraction dance.

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

`color_tools` extraction is tested on synthetic textures painted into the region constants
(every pick-order step, the merge and distinctness boundaries, the buffer contract) and on
one real oracle independent of those constants: the community's colored template sheet,
subsampled to 128x128, whose shirt zone is two reds and whose shorts zones are two yellows,
counted by a script rather than by the crate. Managers' `colors.txt` entries are *not* a
usable ground truth (`color_tools.md` "Calibration evidence": only 55 of 133 declared shirt
colors appear in the shirt region at all), so no tolerance test against them exists; the
thresholds came from the visual swatch-sheet check the calibration harness produced.

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
