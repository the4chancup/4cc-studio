# 4cc Studio — Library crates plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## First-release desktop GPU BC7

GPU BC7 for PES 19–21 is a first-release (1.0.0) target, using `block_compression`'s existing wgpu
backend inside `dds_convert`, built in Phase 16 on the Team compiler's texture step; Release 0.1.0
ships the CPU encoder only (`plans/core/development_plan.md` "Releases"). The library supplies the kernels; Studio implements bounded batching,
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

## `vtree` path types

What exists of this is in `spec/libs/README.md` "`vtree` path types".

Filesystem source providers also enforce containment after resolving symlinks/junctions; a safe
virtual path alone does not prove that its physical source remains inside the export.
