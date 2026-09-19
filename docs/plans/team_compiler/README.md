# 4cc Studio — Team compiler plan

The Team compiler (`crates/tools/team_compiler`) is the flagship tool of the suite:
the successor to the AET Compiler (Red/Blue). It compiles aesthetics exports into CPK
archives, with compile-time model format conversion (see the
[Model conversion plan](../model_conversion/README.md)) and compile-time savefile writing
(see the [Savefile plan](../pes_savefile/README.md)). Platform context (crate structure, tool
plugin trait, GUI shell, parallelism) is in the [core plan](../core/README.md).

### Relationship to the older compilers

The older compilers are stopgaps, not templates. The original batch-file compiler
was rudimentary; **Red** is a direct translation of it into Python — feature-complete
and battle-tested, but slow at its core (it works by moving files around on
storage); **Blue** is an LLM-made prototype demonstrating that an in-memory,
parallel pipeline is possible. None of them is followed to the letter:

- **The contract is output parity with Red** (still the golden standard): given the
  same inputs, the game-facing CPK contents must match Red's — exact bytes for
  unaffected artifacts and normalized semantic parity for intentional ID/path/layout
  changes (see `testing.md` "Testing: parity against Red"). *How* that output
  is produced is free to change.
- **The implementation is built around the object model** (typed structs
  representing each element of an export — see the [Aesthetics export plan](../aesthetics_export/README.md)), not around Red's
  disk-shuffling stages or Blue's path-string processing functions. References to
  Red/Blue modules in this document identify *where the behavior to reproduce is
  specified*, not code to translate.
- **Warning/error messaging is overhauled**, not ported: the pipeline emits
  structured `PipelineEvent`s (see the core plan) carrying export/folder identity,
  and the GUI/CLI decide presentation. Red's console-text messages serve only as a
  checklist of conditions worth reporting.

---

The plan is split into parts; section headings are unchanged from the
single-file plan.

| Part | Covers |
|---|---|
| [Pipeline](pipeline.md) | Per-export pipeline walkthrough |
| [Message catalog](messages.md) | Message catalog |
| [Settings](settings.md) | Settings |
| [Blue features to port](blue_port.md) | Unimplemented Blue Features to Port |
| [GUI grid](gui.md) | GUI: the live-validating team grid; Live validation (no Check or Refresh buttons); Cell states; Toolbar |
| [Testing](testing.md) | Testing: parity against Red (adapted from Blue) |

## Data flow

```
User exports (folders / .zip / .7z)
        │
        ▼
┌──────────────────────────────────────────────┐
│  team_compiler: Reader                       │
│  ─ Scan exports/                             │
│  ─ Index each export's structure into        │
│    ParsedAestheticsExport descriptors              │
│  ─ Classify file kinds (.fmdl / .model / glTF)│
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  team_compiler: Coordinator                  │
│  ─ Structural validation                    │
│  ─ Team ID resolution                       │
│  ┐ Run-level manifest planning:              │
│  │  ─ Player split + dependency resolution   │
│  └  ─ IDs, targets, collisions, ordering     │
│  ┐ Per model folder (rayon parallel):        │
│  │  ─ Materialize content under budget       │
│  │  ─ model_convert via IR (if needed)       │
│  │  ─ XML/FMDL/model/name editing            │
│  │  ─ Texture conversion and relocation      │
│  │  ─ contents_packing (CPK/FPK)             │
│  └ ─ Emit packed entries to writer           │
│  ─ bins_update (team colors, uniparam)       │
│  ┐ Referee processing (spec: Red)            │
│  └ ─ Slot-mapped packing (players.txt)       │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  team_compiler: Writer (single-threaded)     │
│  ─ Incremental CPK writing                   │
│  ─ Per-task atomicity: staged entries of     │
│    failed tasks are discarded                │
│  ─ Sideload folder injection                 │
│  ─ Duplicate invariant check                 │
│  ─ Deploy CPKs to PES folder (or output/)    │
└──────────────────┬───────────────────────────┘
                   │
                   ▼
┌──────────────────────────────────────────────┐
│  pes_savefile: Aesthetics patch + savefile   │
│  ─ Resolve settings.toml + activated IDs     │
│  ─ Write aesthetics_patch.toml beside the CPK│
│  ─ Local save configured: decrypt, apply the │
│    patch, re-encrypt, save (.bak backup)     │
└──────────────────────────────────────────────┘
```

---

## Crate layout

One crate, organized by **pipeline stage**, so a reader who knows the walkthrough below knows where
the code is. The layout is fixed here because the crate is large (15–25k lines) and largely
LLM-written: a predefined tree is what keeps generated code from scattering into `utils.rs` piles
and re-inventing placement per session.

```
crates/tools/team_compiler/
├── Cargo.toml
├── src/
│   ├── lib.rs            # `Tool`: the StudioTool impl — wiring only, no logic
│   ├── settings.rs       # TeamCompilerSettings, defaults, settings_view
│   ├── cli.rs            # clap Command; cli_run / gui_run dispatch (compile, check, upgrade-dpfl)
│   ├── messages.rs       # the message catalog: IDs, severity, disposition, templates, hints
│   ├── paths.rs          # per-version game-paths table (phf) and CpkStem/slot helpers
│   ├── templates.rs      # embedded resources (include_dir!) + templates/ override accessor
│   ├── reader/           # stage 1: where exports come from
│   │   ├── mod.rs        #   discovery: exports scan, NO_USE, team_name, /refs/ and /balls/ routing
│   │   └── source.rs     #   ExportSource: folder / .zip / .7z (via archives), eager structure,
│   │                     #   lazy per-folder content into VirtualTree, memory permit acquisition
│   ├── check/            # live validation orchestration (the lib validates; this schedules)
│   │   ├── mod.rs        #   two-tier check (shallow archive / deep folder), results cache
│   │   └── watcher.rs    #   notify + debounce → rescan via reader, recheck requests (desktop only)
│   ├── plan/             # stage 2, the coordinator: ResolvedAestheticsExport → BuildManifest
│   │   ├── mod.rs        #   plan_run → PlanReport; scoped drops; AbortRun
│   │   ├── manifest.rs   #   BuildManifest, BuildTask kinds, output-path collision checks
│   │   ├── ids.rs        #   PlannedModelIds: 40-ID team blocks, boots/gloves assignment
│   │   ├── refs.rs       #   duplicate-refs preflight, referee slot planning
│   │   └── sideload.rs   #   sideload/ tree precedence
│   ├── processing/       # stages 3–4: parallel per-task work, process_task(BuildTask) → TaskBatch
│   │   ├── mod.rs        #   dispatch by task kind; memory permits; message collection
│   │   ├── model.rs      #   model folders: format selection, conversion, merging, SKL pairing
│   │   ├── texture.rs    #   DDS/FTEX conversion, WESYS zlib, relocation to common
│   │   ├── material.rs   #   MTL / face XML / materials.toml editing and checks
│   │   ├── kit.rs        #   kit folders: config generation, colors, textures, FPC patching
│   │   ├── team_assets.rs#   portraits, logos, collars, Common folder
│   │   ├── referee.rs    #   referee-specific processing (markers, per-referee common layout)
│   │   └── materialize.rs#   THE seam: relocation to game paths + Fox FPK packing (skipped in test mode)
│   ├── bins/             # UniColor / TeamColor / UniformParameter accumulation
│   │   ├── mod.rs        #   working-bin lookup via the DpFileList walk, mutation commit
│   │   └── dpfl.rs       #   DpFileList.bin read/write, slot discovery, upgrade (override)
│   ├── output/           # stages 5–6: the writer and everything after it
│   │   ├── writer.rs     #   canonical-order incremental writing, teams parts, placeholders
│   │   ├── sink.rs       #   OutputSink: CPK (normal) vs loose folder (test/sider)
│   │   ├── deploy.rs     #   staging, .partial + rename, promotion, writability preflight
│   │   └── savefile.rs   #   builds the aesthetics patch from commit outcomes; applies it to the local save (via pes_savefile)
│   └── view/             # egui — renders state, owns no pipeline logic
│       ├── mod.rs        #   tool view layout (grid / toolbar / log composition)
│       ├── grid.rs       #   team grid on studio_core's grid widget; ID cell editing
│       └── toolbar.rs    #   split Compile button, writability badge, run strip
└── tests/
    ├── parity.rs         # upgrade + compile + compare against Red's reference (hash manifest)
    ├── pipeline.rs       # reader/plan/process/writer scenario tests on fixtures
    └── fixtures/         # small Studio-format exports; large trees regenerated, not committed
```

Placement rules:

- **If it validates an export, it is not here** — it belongs in `libs/aesthetics_export`. `check/` only
  schedules and caches those validations for the live grid.
- **If two tools need it, it is not here** — it moves to a lib (`pipeline` for scaffolding such as
  `MemoryBudget`, staging/promotion, `CpkStem`; `aesthetics_export` for format knowledge). The same rule
  in reverse: `bins/dpfl.rs` stays here until a second consumer appears (the Balls compiler only
  *checks* the DPFL, via a read helper that can move to `pipeline` when needed).
- **`processing/materialize.rs` is the single output-mode seam.** No other module asks which output
  mode is active; `output/sink.rs` is the only other place that differs by mode, and it differs by
  *where bytes go*, not by *what is produced*.
- **`reader/` is the only module that touches export sources.** `check/` asks it for the current
  export set; `plan/` and `processing/` receive its `ExportSource`s; nothing else opens a folder or
  archive.
- **`view/` is render-only.** It reads tool state built from `PipelineEvent`s and emits user intents
  (compile, cancel, mode change, ID edit); it never calls into `processing/` or `output/` directly.
- **Module names follow the walkthrough's stage names** (`reader`, `check`, `plan`, `processing`,
  `bins`, `output`) so the plan and the tree use one vocabulary. `pipeline` is deliberately *not*
  used for a tool-local module: it is the shared lib's name, and `pipeline::MemoryBudget` next to a
  `crate::pipeline::…` would be a permanent source of confusion.
- **Files, not folders, until a module exceeds roughly a thousand lines** — then it becomes a
  folder with a `mod.rs` and the same name, so paths in this plan stay valid.
- No `utils.rs`, `helpers.rs`, or `misc.rs`. A helper belongs to the stage that uses it; one used by
  several stages belongs to `pipeline`.

---

## Object model and export format

The export object model, its validation progression (`ParsedAestheticsExport` → `ValidatedAestheticsExport` →
`ResolvedAestheticsExport`), the `aesthetics_export` crate layout, the player-folder format, `settings.toml` in
exports, and the FPC marker files are specified in the [Aesthetics export plan](../aesthetics_export/README.md). The
compiler consumes them; only the compile-time behavior — the loaded processing types
(`FaceModelFolder` and friends, with their conversion and packing methods), the `BuildManifest`,
planning, processing, packing, bins, GUI, and CLI — lives in this crate and this plan.

---

## Development phases

**Phase 3 prerequisites:** confirm the `ExportIdentity` boundary, sanitized
validated-versus-eligible projection, and roster-entry scope/disposition semantics before
implementation begins.

- **Phase 3 — skeleton and shared export model:** implement `libs/aesthetics_export` structure parsing and
  format-level validation, the Team compiler settings/CLI surface, and reader/coordinator/writer
  scaffolding over `libs/pipeline`.
- **Phase 4 prerequisites:** finalize output enumeration/namespace allocation, including
  folder-internal deep-derived names, and freeze every model task's planned ID assignments in the
  manifest.
- **Phase 4 — processing logic:** implement deep validation, model/non-model processing (including
  cross-format conversion via the Phase 2 `model_convert`), packing, bins, and referee behavior on
  synthetic Studio-format fixtures. This phase can test its components and outputs, but its full Red
  parity gate depends on Phase 6's Export upgrader (and savefile/ID integration from Phase 5), so
  end-to-end upgraded-export parity runs only after those phases land.

This documents the dependency without changing the phase order in the core plan.

---

## First-release GPU BC7

Desktop GPU BC7 for PES 19–21 uses `block_compression`'s library backend in the first release.
Auto is the default, with CPU fallback and an explicit CPU mode for reproducible builds. The
[library crates plan](../libs/README.md) owns backend selection, startup, batching, memory, and fallback requirements; the
the reproducibility paragraph in `testing.md` "Testing: parity against Red (adapted from Blue)" records the accepted texture-byte exception. Integrate this into
Phase 4's processing pipeline after the early Phase 2 GPU proof, rather than waiting for 3D preview.

## Future Features

Custom GPU kernels and browser GPU acceleration remain deferred until justified by measurements;
the shared library backend does not have to be replaced merely because it shipped first.

### 3D model preview viewport

A future enhancement to the GUI: when the user clicks a player on the progress grid, a 3D viewport
shows a preview of the model.

Scope note: this stays a quick *glance* ("is this the right model") — for actually inspecting a
player folder there is the [Player aesthetics editor](../player_aesthetics_editor.md), which launches
Blender instead of rendering in-app. When this feature is built, the viewport widget goes into a
shared **`libs/model_viewport`** crate (created then, not before — until this feature lands, no
viewport code exists anywhere): the Player aesthetics editor embeds the same widget as a quick
approximate preview beside its launcher, labeled as a preview with Blender as the accurate view,
which gives the crate its second consumer per the core plan's workspace guardrails.

**Feasibility:** High. egui renders via `wgpu`, which supports custom rendering alongside the
immediate-mode UI. The selected folder's structure descriptor can lazily load and parse its model
under the normal memory budget; the resulting vertex/face data is uploaded as GPU buffers and drawn
in the egui painter's render pass.

**Architecture:**

```
User clicks a player on the grid
        │
        ▼
GUI: submit the folder descriptor + source provider to a cancellable background load
  ─ Acquire a memory permit and materialize the selected FmdlFile/ModelFile off the UI thread
  ─ Read vertices (positions, normals, UVs) from the materialized model
  ─ Read faces (index buffer)
  ─ Read bone data (for optional skeleton overlay)
  ─ Convert texture (DDS/FTEX → raw RGBA via texture2ddecoder)
        │
        ▼
GUI: render with wgpu alongside egui
  ─ Upload vertex/index buffers to GPU
  ─ Create texture from raw RGBA
  ─ Draw mesh in an egui custom painter callback
  ─ Camera: orbit/zoom/pan via egui input
  ─ Optional: skeleton wireframe overlay
```

**Implementation:**

```rust
pub struct ModelPreview {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub bones: Vec<BonePreview>,
    pub texture: Option<TexturePreview>,  // raw RGBA
}
// Loaded on a cancellable background worker under a normal memory permit,
// never on the UI thread; the result is tagged with the export revision so
// a stale preview is discarded.

// Conceptual egui_wgpu pseudocode: the exact constructor/API shape depends on
// the egui version pinned when this future feature is implemented.
let (rect, response) = ui.allocate_exact_size(
    egui::vec2(400.0, 300.0),
    egui::Sense::drag(),
);
let callback_state: Arc<dyn egui_wgpu::CallbackTrait> = Arc::new(ModelPreviewCallback {
    gpu_resources,
    camera,
});
// Wrap callback_state in the pinned version's PaintCallback helper and add it
// to the painter; CallbackTrait owns prepare/finish/render-pass integration.
ui.painter().add(make_paint_callback(rect, callback_state));
```

**Performance:** A 20K-vertex model is ~640KB of vertex data — uploaded to GPU once, rendered at
negligible cost. `wgpu` handles millions of vertices trivially. Texture conversion (one 2K DDS → raw
RGBA via `texture2ddecoder`) takes ~10ms. Only one model is previewed at a time, so memory is not a
concern.

**Limitations:**
- **Bind pose only:** The model is shown in its T-pose. PES applies bone transforms at runtime via
  the skeleton and `face_diff.bin`. Showing a posed model would require implementing the PES
  skinning pipeline. Bind pose is sufficient for "is this the right model, does it look right."
- **Not a perfect PES render:** `wgpu` rendering uses standard shading. PES uses custom shaders
  (fox3ddf_blin, fox3dfw_constant, etc.) with specific alpha/blending behavior. The preview shows
  geometry and base texture accurately, but advanced material effects (anti-blur, UV scrolling,
  transparency) won't match the in-game appearance exactly.
- **One texture at a time:** A face model may have multiple materials (face, hair, oral). The
  preview shows the base texture of the selected material, or cycles through them.
- **WASM compatibility:** `wgpu` works in browsers via WebGPU. The 3D preview is available in both
  the desktop and web builds.

**When to implement:** After the GUI is functional (post-Phase 8). The core architecture doesn't
need to change; it's a `wgpu` rendering pass alongside egui. The object model already has all the
data needed.
