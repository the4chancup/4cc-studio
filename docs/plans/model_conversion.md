# 4cc Studio — Model conversion plan

Covers the `model_convert` lib crate (the IR plus importers/exporters and skeleton
data) and the format-extension algorithms that live in the `fmdl` and `pes_model`
format crates (mesh splitting, split vertex encoding, anti-blur). Model format
conversion is not a standalone tool — it runs inside the
[Team compiler](team_compiler.md) at compile time, converting models to the target
PES version's format. The glTF + `materials.toml` authoring format that `gltf_to_ir`
reads and `ir_to_gltf` writes is specified in the [Unified model format
plan](model_format.md). Platform context is in the [core plan](core.md).

---

## Crate layout

The crate is a hub: one IR in the middle, one importer/exporter pair per format around it, and
IR-level operations that never know which format the data came from. The layout enforces the two
boundaries the design depends on — **format code never touches the IR's internals beyond the public
struct, and IR operations never touch a format** — and keeps the format crates (`fmdl`,
`pes_model`) as dependencies, never dependents.

```
crates/libs/model_convert/src/
├── lib.rs              # re-exports; ModelFormat; NativeModelBundle; convert_to routing
├── ir/                 # the canonical model — a data structure, not a behavior
│   ├── mod.rs          #   CanonicalModel, Mesh, Vertex layout, Bone, Material refs
│   ├── skeleton.rs     #   normalized bone mapping (name ↔ index), bone name conventions (skf_, dsk_)
│   └── validate.rs     #   IR invariants (indices in range, weights normalized, one skin per mesh)
├── formats/            # one module per format: to_ir + from_ir, nothing else
│   ├── fmdl.rs         #   fmdl_to_ir / ir_to_fmdl (calls the fmdl crate's ops for splitting/encoding)
│   ├── pes_model.rs    #   model_to_ir / ir_to_model (+ .mtl pairing; commits both or neither)
│   └── gltf/
│       ├── mod.rs      #   gltf_to_ir / ir_to_gltf via the gltf crate
│       ├── extensions.rs   # PES_bone / PES_mesh read + write, fallbacks when absent
│       └── images.rs   #   the image contract (embedded images ignored, stems resolved from folder)
├── materials/          # the engine-neutral material schema (spec: Unified model format plan)
│   ├── mod.rs          #   Material, ShaderFamily, canonical texture roles, parameters
│   ├── toml.rs         #   materials.toml / *.materials.toml / .common link read + write (toml_edit)
│   ├── matching.rs     #   name matching (startsWith/endsWith), layering, Common-first cascade
│   ├── to_fox.rs       #   family → FMDL shader/params tables
│   └── to_prefox.rs    #   family → .mtl shader/params tables
├── skeletons/          # per-version skeleton data and retargeting
│   ├── mod.rs          #   Skeleton, skeleton_for(version)
│   ├── data.rs         #   GENERATED from resources/skeletons/pes*/ — do not edit by hand
│   ├── fold.rs         #   fold tables (bones a version lacks → nearest ancestor)
│   └── retarget.rs     #   retargeting + bone conformance (the cross-version IR operation)
├── ops/                # IR-level operations, format-agnostic
│   ├── hand_split.rs   #   split_by_skeleton_group (gloves hand auto-split)
│   ├── merge_parts.rs  #   merge_ir_parts (deferred: no planned caller, see "IR part merge")
│   └── superset.rs     #   dual-set superset merge for ir_to_gltf (Player aesthetics editor)
└── loss.rs             # data-loss reporting: what a target format cannot represent, as findings
```

Placement rules:

- **`formats/*` are the only modules that import `fmdl`, `pes_model`, or `gltf`.** Everything else
  sees the IR. A format detail needed by an op is a field on the IR, not an import.
- **`ir/` has no logic beyond validation.** Transformations are `ops/`; conversions are `formats/`.
- **`skeletons/data.rs` is generated** (build script or checked-in dev tool over
  `resources/skeletons/`) and carries a header saying so; hand edits are reverted by regeneration.
  Provenance stays with the `.skl` files, never with Python literals.
- **`materials/` is shared by all three formats** and by the Blender project's glTF codec contract;
  it must remain independent of `formats/` so the schema can be tested without any model file.
- **Same-format paths do not enter this crate** unless an explicitly planned IR operation is
  requested (hand split, part merge, retargeting); ordinary FMDL→FMDL processing uses the `fmdl`
  crate directly from the Team compiler. This is a consumer-side rule, restated here because the
  temptation is to add a "convenient" pass-through.
- Stays `wasm32`-checkable and GUI-free (consumers: Team compiler, Balls compiler, Export upgrader,
  Player aesthetics editor).

---

## Intermediate Representation (IR)

### Purpose

The IR is a canonical PES model representation that is a **superset** of both FMDL and .model
formats. It eliminates duplicated conversion logic by providing one decode path and one encode path
per format, instead of N×N direct converters.

### When the IR is used

- **Same format as target**: ordinary processing skips the IR; hand auto-split, pre-Fox
  `ingame_face` merging, and skeleton retargeting across PES versions (only when a native-format
  pre-check finds a used bone the target lacks or poses differently — see "Skeleton retargeting and
  bone conformance") are the explicit IR-operation exceptions described below.
- **FMDL → .model**: FMDL → IR → .model
- **.model → FMDL**: .model → IR → FMDL
- **glTF → FMDL**: glTF → IR → FMDL
- **glTF → .model**: glTF → IR → .model
- **FMDL / .model → glTF**: native → IR → glTF (`ir_to_gltf` — the [Player aesthetics
  editor](player_aesthetics_editor.md)'s conversion; the Team compiler never emits glTF)

### IR struct

The IR uses `Option<T>` for format-specific fields. `Some` when imported from the format that has
the field, `None` when imported from a format that doesn't. Exporters fill in defaults for missing
fields.

```rust
pub struct CanonicalModel {
    pub bones: Vec<Bone>,
    pub meshes: Vec<Mesh>,
    pub mesh_groups: Vec<MeshGroup>,
    pub materials: Vec<Material>,
    pub textures: Vec<Texture>,
    pub extension_headers: BTreeMap<String, Vec<String>>,
    pub source_format: SourceFormat,
}

pub struct Bone {
    pub name: String,
    pub parent: Option<usize>,       // hierarchy: FMDL stores it; .model does not (resolved by
    pub children: Vec<usize>,        //   name from the template skeleton's render parents)
    pub matrix: [f32; 12],           // model-space 3×4 bind transform. Sources: FMDL's companion
                                     //   .skl (or the template); .model's inline per-bone inverse
                                     //   bind matrix, inverted; glTF skin bind data
    // FMDL-specific
    pub global_position: Option<Vector4>,
    pub local_position: Option<Vector4>,
    pub bounding_box: Option<BoundingBox>,
    // Template-skeleton display data (PesSkeletonData start/end positions); not stored
    // in any file, looked up by name for Blender bone geometry
    pub start_position: Option<Vector3>,
    pub end_position: Option<Vector3>,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<Face>,
    pub bone_group: Vec<usize>,
    pub material: usize,
    pub vertex_fields: VertexFields,
    // FMDL-specific
    pub alpha_flags: Option<u8>,
    pub shadow_flags: Option<u8>,
    pub extension_headers: BTreeSet<String>,
    // Mesh splitting metadata (decoded on import, re-encoded on export)
    pub split_group_id: Option<usize>,
    // Anti-blur metadata
    pub is_antiblur: bool,
    pub antiblur_source: Option<usize>,
}

pub struct Vertex {
    pub position: Vector3,
    pub normal: Option<Vector4>,
    pub tangent: Option<Vector4>,
    pub color: Option<[u8; 4]>,
    pub uv: Vec<Vector2>,
    pub bone_mapping: Option<Vec<(usize, f32)>>,
}

// Mirrors the materials.toml schema (see the Unified model format plan): an
// engine-neutral core plus one optional table per engine. `Some` when imported
// from that engine's format or from a toml that fills it; exporters derive a
// missing engine table from `family` via the shader-family defaults.
pub struct Material {
    pub name: String,
    pub family: MaterialFamily,                  // Shaded | Shadeless | Metal | Glass
    pub two_sided: Option<bool>,
    pub transparent: Option<bool>,
    pub antiblur: Option<bool>,
    pub textures: Vec<(TextureRole, usize)>,     // canonical role → index into `textures`
    pub parameters: Vec<(String, [f32; 4])>,     // engine-neutral shader parameters
    pub fox: Option<FoxMaterial>,
    pub prefox: Option<PreFoxMaterial>,
}

pub struct FoxMaterial {
    pub shader: String,
    pub technique: String,
    pub alpha_flags: u8,                         // per-mesh in FMDL; see the format plan's flag rule
    pub shadow_flags: u8,
    pub cast_shadow: Option<bool>,               // Fox-only booleans over shadow_flags bits 1/2
    pub invisible: Option<bool>,
    pub base_linear: bool,
    pub textures: Vec<(String, usize)>,          // native sampler name → texture, non-canonical only
    pub parameters: Vec<(String, [f32; 4])>,
}

pub struct PreFoxMaterial {
    pub shader: String,
    pub states: Vec<(String, i32)>,              // ztest, zwrite, twosided, alphatest, alpharef, alphablend, blendmode
    pub samplers: Vec<(String, SamplerSettings)>, // native sampler name → srgb/filters/addressing/maxaniso
    pub parameters: Vec<(String, [f32; 4])>,     // .mtl <vector> elements
}
```

FMDL stores four-component normals and tangents; the IR must not silently discard their fourth
components. glTF uses three-component normals and four-component tangents (including handedness),
so conversion must distinguish semantic tangent handedness from native metadata and preserve
otherwise unrepresented native components through the PES metadata path. Roundtrip fixtures cover
non-default fourth components, not only ordinary XYZ data.

**Deterministic collections.** Order-bearing IR metadata uses `BTreeMap`/`BTreeSet` (or equivalently
sorted vectors), never serialization by `HashMap`/`HashSet` iteration. Importers normalize extension
keys, and exporters/merge routines emit them in canonical order. This is required by the Team
compiler's byte-reproducible Rust-output contract.

### Material sources per format

The three formats keep material data (shader, texture paths, render states) in
different places; the IR's `Material` is the merge point:

- **FMDL** embeds materials in the model binary — `fmdl_to_ir` reads them from
  the file itself. FMDL texture path tables store **stems** (no extension) — the
  game appends `.ftex` at load time — so FMDL has always been extension-agnostic.
- **.model** keeps them in the sibling **`.mtl` XML**, with meshes binding
  materials by name. A pre-Fox native source is therefore an atomic bundle
  (`.model` + `.mtl`): `model_to_ir` consumes both and `ir_to_model` emits both.
  Conversion never changes one without the other. The `.mtl` codec lives in
  `pes_model` alongside the `.model` codec (it's the same format family; the
  Team compiler's MTL validation and path rewriting use the same parser).
- **glTF** keeps them in **TOML files** alongside the glTF, mirroring the
  `.model`/`.mtl` split: the glTF's `primitive.material` references name
  materials, and the PES-specific properties live in `materials.toml` /
  `*.materials.toml` files in the same folder (never embedded in the glTF
  JSON). The file pattern, name matching, layering, `.common` link files, and
  the complete schema are in the [Unified model format plan](model_format.md).
  `gltf_to_ir` takes the folder's resolved material files as an argument and
  fills the IR's `Material` structs from them; the glTF files themselves are
  never rewritten (exports stay read-only). glTF-only — FMDL and .model keep
  their native mechanisms.

**Engine mapping.** The IR `Material` is engine-neutral in its core (`family`,
booleans, canonical texture roles, parameters) with an optional native table per
engine. Importers fill the native table of their own engine verbatim and infer
`family` from the shader; exporters use the target engine's native table when
present and otherwise derive it from `family` through the shader-family defaults
and role→sampler tables in the format plan. So a `.model` → IR → FMDL conversion
picks the Fox shader from the family (as `model2fmdl.py` does today with its
`Basic_C` → `fox3ddf_blin` table), while an FMDL → IR → FMDL round-trip never
touches the shader at all. Per-mesh FMDL flags are lifted to the material on
import (splitting instances whose meshes differ) and pushed back down on export.

**Stem-based texture references (input).** All three formats reference textures
by **stem** (filename without extension) in their **source/authoring** form; the
compiler resolves each stem to whatever accepted image file exists in the model
folder and converts to the target format at compile time (format list, stem
conflicts, and auto-naming rules: format plan, "Dependencies and images" and
"Textures"). Studio-format `.mtl` and `.materials.toml` files write stems; FMDL
path tables were already stem-based. The Export upgrader strips extensions from
old `.mtl` texture paths when migrating. **Output** `.mtl` files (written by
`ir_to_model` at compile time) keep the `.dds` extension on texture paths, as
the game expects; FMDL output path tables remain stem-based (the game appends
`.ftex`). Auto-naming and auto-detection of texture roles happen during deep
material parsing (not the structure pass), so a missing texture reports
`mtl_texture_not_found` / `material_texture_not_found` at that stage, not during
shallow validation.

### Extension algorithms (one implementation each)

Each format extension (mesh splitting, anti-blur, split vertex encoding) lives in the format crate
that owns it, not in `model_convert`. The IR importers/exporters call the format crate's
decode/encode functions. This keeps same-format round-trips (e.g. FMDL→Blender→FMDL via the PyO3
bindings) lossless and IR-free, while cross-format conversions still benefit from a single
implementation:

| Algorithm | Owner crate | Called by (decode) | Called by (encode) |
|-----------|-------------|---------------------|---------------------|
| FMDL mesh splitting | `fmdl` | `fmdl_to_ir` | `ir_to_fmdl` |
| .model mesh splitting | `pes_model` | `model_to_ir` | `ir_to_model` |
| FMDL anti-blur | `fmdl` | `fmdl_to_ir` | `ir_to_fmdl` |
| Split vertex encoding (FMDL) | `fmdl` | `fmdl_to_ir` | `ir_to_fmdl` |
| Split vertex encoding (.model) | `pes_model` | `model_to_ir` | `ir_to_model` |

A bug fix in FMDL mesh splitting decode benefits every path starting from FMDL, regardless of target
format — and also benefits the Blender addon's direct FMDL round-trip, which calls the same `fmdl`
function without going through the IR at all.

`fmdl` also owns **multi-FMDL mesh merging** — bone lists unioned by name (per-mesh bone groups
remapped), meshes and mesh groups concatenated, materials merged by name (same-name materials with
differing definitions rejected), buffers and headers rebuilt. It is not an IR operation: the Team
compiler calls it directly to assemble multi-part models and to bake Common-linked models into
player FMDLs on Fox targets, where the engine cannot load models from the Common folder (see "Common
model links and model merging" in the [Team compiler plan](team_compiler.md)). The pre-Fox
counterpart is `pes_model::ops::merge`, native over `Model` + `MaterialSet` (see
"`pes_model::ops::merge`" in the [Libraries plan](libs.md)), which follows the same rules.

### Conversion routing

```rust
pub enum NativeModelBundle {
    Fox(FmdlFile),
    PreFox { model: PreFoxModel, mtl: MtlFile },
    Gltf(GltfModel),
}

pub fn convert_to(&mut self, target: ModelFormat) -> Result<()> {
    match (&self.source_bundle, target) {
        (NativeModelBundle::Fox(_), ModelFormat::Fox) => Ok(()),
        (NativeModelBundle::PreFox { .. }, ModelFormat::PreFox) => Ok(()),

        (NativeModelBundle::Fox(fmdl), ModelFormat::PreFox) => {
            let ir = fmdl_to_ir(fmdl)?;
            let (model, mtl) = ir_to_model(&ir)?;
            self.source_bundle = NativeModelBundle::PreFox { model, mtl };
            Ok(())
        }
        (NativeModelBundle::PreFox { model, mtl }, ModelFormat::Fox) => {
            let ir = model_to_ir(model, mtl)?;
            self.source_bundle = NativeModelBundle::Fox(ir_to_fmdl(&ir)?);
            Ok(())
        }
        (NativeModelBundle::Gltf(gltf), target) => {
            let ir = gltf_to_ir(gltf)?;
            self.source_bundle = match target {
                ModelFormat::Fox => NativeModelBundle::Fox(ir_to_fmdl(&ir)?),
                ModelFormat::PreFox => {
                    let (model, mtl) = ir_to_model(&ir)?;
                    NativeModelBundle::PreFox { model, mtl }
                }
            };
            Ok(())
        }
    }
}
```

The replacement is committed only after both members of a pre-Fox bundle are
produced successfully, so an error can never leave a converted `.model` paired
with an old or missing `.mtl`.

### Roundtrip testing

The IR enables lossless roundtrip tests for same-format paths (see "Testing: IR roundtrips" below).
Cross-format roundtrips (FMDL→.model→FMDL) are lossy in known, accepted ways (anti-blur regenerated,
mesh splits may differ). The IR preserves the metadata so the losses are explicit, not accidental.

### Data loss behavior

The existing direct converters are already lossy:
- FMDL→.model: anti-blur stripped (.model doesn't support it)
- .model→FMDL: anti-blur regenerated with defaults
- Both: bone names remapped through `missingBones` table
- Both: mesh splitting undone and redone
- FMDL→.model: custom bone transforms replaced by the template skeleton's (the converter never
  read the companion `.skl`)

Routing through the IR doesn't add new data loss, and removes the last one: bind transforms travel
in `Bone.matrix` from the `.skl` or the `.model`'s inline bone table to whichever format is written
(see "Skeleton in `.model`"). The other losses happen at different pipeline points. The IR
preserves metadata (e.g., `antiblur_source`), so if exporting back to FMDL, the metadata is
available.

---

## Hand auto-split

PES glove models use the **hand skeleton**, which has bones only up to the wrists and forearms —
separate from the body skeleton used by face and boots models, though they share wrist/forearm bones. On Fox, the hand skeleton is
inaccessible from the face folder, so users must provide gloves as separate model files; on pre-Fox,
gloves can coexist in the face folder (typed as `gloveL`/`gloveR` in `face.xml`), but the hand
geometry still has to be a separate model loaded with the hand skeleton. In both cases, the user
must manually cut the hands at the wrist from a full-body model and export them separately — the one
remaining authoring annoyance that the unified player folder format doesn't otherwise address.

The compiler eliminates this by **auto-splitting** any model whose vertices carry
hand-skeleton-exclusive bone weights into separate body and glove parts at compile time. The user
authors a single full-body model (e.g., `face_high.gltf`); the compiler produces the body (face)
model plus `glove_l` and `glove_r` models automatically.

### Detection

The check reads bone assignments **directly from the native format** — FMDL bone groups, `.model`
bone mappings, glTF `JOINTS_n`/`WEIGHTS_n` accessors resolved through `skin.joints` — without importing to the IR. The **hand-skeleton-exclusive
bones** are identified by name convention: they are prefixed with `skf_` (finger skeleton family)
and suffixed with `_l` or `_r` for laterality. No pre-defined list or `PES_SKELETON` lookup is
needed — the prefix/suffix match is the complete identification.

No opt-in is needed: vertices with positive `skf_` weights seed the hand selection. Unused `skf_`
names in a bone list do not trigger splitting; the check follows actual vertex weights.

### Split

When hand weights are detected, import to the IR and perform the following mesh operations
**in-process in Rust**, independently for each hand. Blender's commands describe the intended
selection/separation semantics only: the compiler neither launches Blender nor uses `bpy`, and
compiling hundreds of models must not require a Blender installation or per-model external process.

1. **Select** vertices with any positive weight on that hand's `skf_` groups (`_l` or `_r`).
2. **Grow once** along the mesh topology — equivalent to one Blender **Select More** (`Ctrl +`)
   step in vertex mode, bringing in the neighboring row at the wrist. This is not a distance-based
   cut, and shared wrist/forearm weights do not exclude a vertex from the expanded selection.
3. **Separate the selection** into `glove_l` or `glove_r`, equivalent to Blender's `P` → Selection.
   Fully selected faces move with it; the remaining faces stay with the body, retaining copies of
   boundary vertices where needed. Preserve positions, weights, UVs, normals, and materials — no
   geometric cutting, invented wrist plane, or reweighting.

Prune unused bones from each result and remap indices to the target skeleton as usual. The extra
row is part of the rule, not an optional refinement: vertices without finger weights can belong to
the separated glove. Validate against stored expected output prepared with the same select → grow
once → separate operation in Blender on a matching-topology fixture with a connected wrist, not
only already separated hand meshes. Blender is a development-time reference for preparing that
fixture, not part of compilation or the Rust regression-test runtime. Benchmark the in-process
split on a representative batch of hundreds of models without Blender installed.

Models without positively weighted hand vertices **pass through unchanged** in this step. The
native weight check needs no IR import or re-export; the original model is retained.

### Pipeline integration

The auto-split runs **before categorization** (see "Player folder categorization" in the [Team
compiler plan](team_compiler.md)). The split parts appear as virtual model files in the player
folder's model list; categorization handles them normally — the body becomes face content, the
gloves become `glove_l`/`glove_r`. The rest of the pipeline (merge, ID assignment, packing) is
transparent to whether the gloves were auto-split or authored as separate files.

### Where it lives

`model_convert`, as an IR-level operation (`split_by_skeleton_group`). It operates on the IR's
normalized bone mapping and identifies hand bones by the `skf_` name convention. It is a one-way
semantic transformation, not a same-format round-trip, so it does not belong in the
`fmdl`/`pes_model` format crates (which own format-native operations for lossless round-trips).

### Scope

Gloves only. Boots use the same body skeleton as the face model — there is no skeleton-exclusivity
problem to solve, so no auto-split is needed or offered for boots.

---

## glTF as the universal model source

The suite's authoring format is **glTF with PES extensions plus `materials.toml`** — the [Unified
model format plan](model_format.md) specifies it (rationale, folder layout, `PES_bone`/`PES_mesh`
extension schema, native-skin skeleton, material files and their `.common` links, the complete
material schema). This plan covers what `model_convert` does with it.

Unlike FMDL and .model, glTF gets **no workspace format crate of its own**: parsing is the external
`gltf` crate's job (a mature codec — there is no bespoke binary format to own), and everything the
suite adds on top — reading/writing the `PES_bone`/`PES_mesh` extensions and the `materials.toml`
files, `gltf_to_ir`, and `ir_to_gltf` — is
IR-coupled, so it lives inside `model_convert`. A separate wrapper crate would have `model_convert`
as its single permanent consumer, which the core plan's workspace guardrails flag as over-splitting;
there are also no format-native mesh algorithms or outside consumers to justify one (Blender-side
glTF handling is Blender's own pipeline plus the `pes-models` extension's PES glTF codec, no Rust
involved).

### glTF import (`gltf_to_ir`)

`gltf_to_ir` takes the glTF document plus the folder's **resolved material layers** for that glTF
(the caller — the Team compiler's folder task — performs name matching, `.common` link resolution,
and the least→most-specific ordering per the format plan; `model_convert` performs the field-level
deep merge and schema validation, so the merge rule has one implementation shared by every
consumer). Each layer is tagged with its origin folder (player folder or Common), because a texture
stem resolves in the folder of the material file that set it. It fills the IR's `Material` structs
from the merged entries, resolving canonical texture roles to `Texture` entries through the
caller-supplied asset resolver (see "glTF dependency and image contract"). Schema violations surface as typed errors the caller maps to the
`material_*` message catalog. The glTF files themselves are never rewritten (exports stay
read-only).

### glTF export (`ir_to_gltf`) and dual-set superset merge

glTF is also an IR **target**, for the [Player aesthetics editor](player_aesthetics_editor.md)'s
convert-to-glTF migration and the [Export upgrader](export_upgrader.md)'s optional glTF conversion
pass (the Team compiler never emits glTF — its targets are the game formats). `ir_to_gltf` emits
a single `.glb` per model carrying the `PES_bone`/`PES_mesh` extension schema that `gltf_to_ir` reads — the
exporter and importer share the schema tables, so a Studio-emitted file is by construction
importable, and `gltf_to_ir(ir_to_gltf(x))` round-trips are part of the test suite. Existing
textures in the source folder are referenced externally from the material tomls by stem, never
re-encoded or embedded. Materials are emitted as a single catch-all `materials.toml` per folder with
app-injected comments, each entry carrying the inferred `family` plus the source engine's native
table verbatim (see "Emission" in the format plan).

**Skeleton reconstruction from FMDL.** The FMDL bone data carries only local and global positions
(Vector4) — not the 3×3 rotation matrix. The full bind-pose transform (rotation + translation)
lives in the companion `.skl` file. `fmdl_to_ir` resolves the bone transforms as follows:

1. If the FMDL has a companion `.skl` (same basename, per the Aesthetics export plan's SKL pairing rule),
   read the full 3×4 transforms from the SKL via the `fmdl` crate's SKL codec.
2. If no companion SKL exists, use the embedded template skeleton (`body.skl`) for the standard
   bone transforms, supplemented by the FMDL's own position data.

**Skeleton in `.model`.** The pre-Fox format needs no sidecar: a `.model` stores its bone name
table and, per bone, a 3×4 **inverse bind matrix** inline (section 0's first entry, one 12-float
record per bone in name order — `ModelFile.parseBoneGroups` in the `pes-model` addon), which is
exactly the data an FMDL keeps in its `.skl`. What it lacks is the hierarchy: parents are resolved
by looking each bone name up in the template skeleton's render parents (the addon's `addBone`), so
a bone unknown to the template is a root. `model_to_ir` inverts the stored matrices into
`Bone.matrix`; `ir_to_model` writes the inverse of `Bone.matrix` back. What ends up in `Bone.matrix`
at export time is decided by the skeleton retargeting pass (see "Skeleton retargeting and bone
conformance"): standard bones carry the **target version's** bind pose — which is what a `.model`
must contain to skin correctly against that game's runtime skeleton — and only bones unknown to the
target's tables (a genuine custom skeleton) keep the model's own matrices. **This deviates from the
19to16 converter**, which always wrote PES17's matrices (`fmdl2model.py`, identity for unknown
bones) regardless of target and flattened custom skeletons. In the other direction, `.model` → Fox
generates a companion `.skl` from the IR only when a custom bone survives; a model on the standard
skeleton gets none, and the compiler's template injection stands.

The IR `Bone.matrix` carries the full PES bind transform. This is not a direct byte-layout copy
into glTF: the sample SKL positions are model-space, whereas glTF nodes use parent-local transforms
and serialize matrices column-major. `ir_to_gltf` converts coordinate conventions and hierarchy,
constructs matching `skin.joints` and `inverseBindMatrices`, and preserves PES-only metadata such as
`globalPosition` and `sklParent` in `PES_bone`. The reverse path reconstructs the PES transforms.
Verify a multi-level, rotated skeleton and skinned mesh in both directions; merely expanding 3×4
matrices to 4×4 would apply parent transforms twice.

For folders holding **both native sets**, `model_convert` provides the superset
merge the editor's conversion uses:

```rust
/// Fill `preferred`'s format-exclusive gaps (None fields) from `other`,
/// matching materials by name and meshes by part identity. Geometry, bones,
/// and all populated fields of `preferred` win; nothing is averaged.
pub fn merge_ir_superset(preferred: CanonicalModel, other: &CanonicalModel) -> CanonicalModel;
```

The result is one glTF per part holding the preferred geometry plus compatible format-exclusive
metadata from both sources. If the two native sets have different geometry, it cannot reproduce
both originals: choosing the preferred set is an intentional geometry replacement. Report
unmatched or incompatible metadata rather than claiming an unconditional lossless superset.
Which set supplies the geometry is the caller's choice, surfaced at conversion time.

### IR part merge (`merge_ir_parts`)

`merge_ir_superset` merges *one part across two formats*; it never adds geometry. Assembling
*several parts into one model* is a different operation, and it is **format-native on both
engines**: `fmdl::ops::merge` for Fox, `pes_model::ops::merge` for pre-Fox (the `ingame_face`
boots merge, and a shared link combined with local parts under `ingame_face` — see the [Aesthetics
export plan](aesthetics_export.md)'s "ingame_face marker"), with the same rules, so a same-format
merge never round-trips through the IR. An earlier version of this plan routed the pre-Fox cases
through the IR-level `merge_ir_parts` below; that was dropped because it made them the one
exception to "same format skips the IR" for no gain (the `.mtl` merge is a material-name merge
either way). `merge_ir_parts` stays specified here as the IR-level equivalent, **deferred**:
nothing in the plan calls it, and it is implemented only if a caller appears (a merge of parts
that must stay in the IR, inside the Player aesthetics editor for instance). The rules below are
the contract all three share.

```rust
/// Assemble several parts into one model. `parts` is in canonical (alphabetical
/// source-name) order and that order is preserved in the output.
pub fn merge_ir_parts(parts: Vec<CanonicalModel>) -> Result<CanonicalModel, MergeError>;

pub enum MergeError {
    MaterialConflict { name: String },          // → team-compiler `merge_material_conflict`
    SkeletonConflict,                           // → team-compiler `skl_merge_conflict`
    SourceFormatMismatch,                       // parts must share `source_format`
}
```

Rules (identical to the `fmdl` merge unless stated):

- **Bones** are unioned by name; a bone present in several parts must have the same `matrix`
  (and, when populated, the same positions) or the merge fails with `SkeletonConflict`. Each mesh's
  `bone_group` and every vertex `bone_mapping` are remapped to the unioned indices. In practice
  pre-Fox parts share the standard body skeleton, so the union is the identity; the rule exists so a
  custom-skeleton part cannot be silently merged with a default-skeleton one.
- **Meshes and mesh groups** are concatenated in part order; `split_group_id` and anti-blur
  metadata stay per mesh (they are re-encoded on export as usual).
- **Materials** are merged by name. Two definitions with the same name must be equal in every
  populated field — for `.model` sources that includes `state_settings`, i.e. the `.mtl` content —
  otherwise `MaterialConflict`. Mesh `material` indices are remapped. Because `model_to_ir` consumes
  the `.model` + `.mtl` bundle, merging materials *is* the `.mtl` merge: `ir_to_model` emits one
  `.mtl` for the merged model, so the Team compiler never has to splice MTL XML itself.
- **Textures** are unioned by path (same path → one entry, indices remapped).
- **`extension_headers`** are unioned per key with values deduplicated, in canonical order.
- **`source_format`** must be the same for all parts (the caller converts foreign-format parts
  first, as the Team compiler already does before Fox merging).

Deterministic by construction: the caller supplies canonical order and every union uses ordered
collections, satisfying the byte-reproducible output contract.

### glTF dependency and image contract

- **URI safety and locality:** `model_convert` accepts dependencies through a
  caller-supplied asset resolver rather than opening paths itself. The caller
  percent-decodes and canonicalizes external buffer/image URIs into the shared
  `vtree::ScopePath` type and requires them to remain
  inside the **source model folder**, not merely somewhere beneath the export root.
  Absolute paths, parent traversal, cross-folder references, network URLs, and
  non-`data:` URI schemes are rejected before materialization; this keeps folder
  tasks independently ownable and keeps `model_convert` reusable by the Balls
  compiler without depending on the aesthetics export format.
- **Dependency ownership:** `.gltf` external `.bin` entries are classified as
  `GltfBuffer`, while `.glb` buffers remain embedded. Callers classify candidate
  external image files as `PotentialGltfImage` from structure alone; deep URI
  resolution promotes referenced candidates to `GltfImage` and returns
  unreferenced candidates to the caller's ordinary allowlist policy. A
  materialized source folder owns one permit-charged `Arc<[u8]>` for each unique
  resolved external buffer/image; multiple `GltfModel`/`ModelPart` consumers clone
  that Arc without a second charge. Embedded GLB/data-URI bytes are owned once by
  their `GltfModel`. Dependency files are not also emitted as ancillary/pass-through
  game files: a source image cannot have both dependency and pass-through roles
  simultaneously.
- **Embedded data:** standard GLB buffer views and base64 `data:` URIs are accepted
  and charged to the same folder memory permit. Malformed or unsupported MIME types
  are validation errors; remote fetching is never performed.
- **Accepted image formats and stem-based resolution:** the accepted image formats (all
  interchangeable as sources for any model format), the stem→file resolution, and the
  `texture_stem_conflict` rule are specified in the format plan ("Dependencies and images"); the
  [library crates plan](libs.md) has the full decoder matrix.
- **PES texture boundary:** `dds_convert` selects codecs by PES version and material role,
  independently of the model's source format. BC7 is supported on PES 19–21, not PES18 despite
  its Fox model format. PES 15–18 convert BC7/raster inputs to BC3 or eligible opaque-color BC1;
  PES 19–21 retain supported BC7 and encode ordinary raster sources to BC7. The [library
  crates plan](libs.md) owns alpha/mipmap/normal-map rules and the CPU encoder choice.
  Already-compatible blocks are retained where only DDS/FTEX container adaptation is needed.
  `model_convert` supplies pixels and material roles, not final player/team paths; the run-level
  `BuildManifest` and texture-relocation step own those destinations.
- **PNG-texture trajectory:** Blender cannot write DDS natively, and the ported
  addons convert DDS→PNG on import, so glTF-authored models (the plan's end-state
  authoring path) naturally carry PNG textures. As the community migrates from
  direct FMDL/.model editing to glTF authoring, PNG sources become the norm rather
  than the corner case. Measure decode, mip generation, and the selected CPU encoder on real
  textures; earlier hypothetical timings are not benchmarks of this implementation. The
  [in-memory conversion cache](libs.md) reuses retained results on the edit→compile→test loop
  (engaging only for ≤2-team compiles); a fresh full-cup compile still pays for conversion.
  See the [library crates plan](libs.md) for cache design and engagement rules.

### Blender integration

The community works with PES models in Blender through two complementary paths. Both are packaged as
codec modules of **one Blender 5.2 extension** — the all-in-one `pes-models` plugin, which also
hosts the [Player aesthetics editor](player_aesthetics_editor.md)'s loader module and **replaces the
standalone ported `pes-fmdl`/`pes-model` addons at its release** (plan:
`blender_project/pes_models_plan.md`, kept with the Blender project outside this repo):

**Path A — legacy FMDL/.model import-export (lossless, direct).** The existing `pes-fmdl` and
`pes-model` addons have working Blender 5.2 ports; `pes-models` vendors them as its `fmdl` and
`pes_model` codec modules, minimally modified (packaging consolidation, not a rewrite — the ported
code is the battle-tested asset). As a successor step, the hot paths — `parseVertices`,
`parseFaces`, `encodeVertices`, mesh splitting, split vertex encoding, anti-blur — are replaced by
calls into `fmdl` and `pes_model` via the `python_bindings` PyO3 wheel. Same-format round-trips
(FMDL→Blender→FMDL, .model→Blender→.model) are lossless because they call the format crate directly
and never route through the IR. The codecs retain a pure-Python fallback so they work even if the
wheel is not installed or has rotted against a newer Blender Python. This path serves members
editing existing `.fmdl`/`.model` files directly.

**SKL on FMDL import.** The FMDL bone data carries only local and global positions (Vector4), not
the 3×3 rotation matrices — those live in the companion `.skl`. The current `pes-fmdl` addon builds
the armature from `PesSkeletonData.py`'s hardcoded reference skeleton, ignoring any companion SKL.
This loses custom skeleton data when the user later exports to glTF via Path B (the glTF would
carry the reference skeleton, not the custom one). The `pes-models` FMDL codec fixes this: on
import, it reads the companion `.skl` (same-basename pairing, per the Aesthetics export plan's SKL pairing
rule) and builds the armature from the SKL's full 3×4 transforms. When no companion SKL exists, it
falls back to the current `PesSkeletonData` behavior (the reference skeleton's positions, which
were extracted from `body.skl` once). This ensures a Blender→glTF round-trip preserves the skeleton
regardless of source format. The SKL format is simple enough (3-u32 header, 56-byte fixed records,
null-terminated name table — ~150 lines) that a pure-Python reader ships with the initial
`pes-models` release, no wheel needed; when the PyO3 wheel arrives, the Python reader is replaced
by the `fmdl` crate's SKL codec alongside the other hot paths.

**Path B — unified glTF authoring format (new, IR-routed).** The extension's PES glTF codec wraps
Blender's native glTF exporter and emits glTF with name-only material objects **plus the folder's
`materials.toml`** — it does not inject PES material data into the glTF JSON. On import, it reads
the tomls from the model's folder and maps each material's PES properties to Blender custom
properties, so glTF-authored models round-trip through Blender without losing their PES data
(essential once glTF folders are the norm). This replaces the ~260KB of Python across the legacy
addons for *new* authoring. The tomls hold the superset of both engines' settings (common,
FMDL-exclusive, and .model-exclusive), so a model authored once in glTF can be compiled to either
PES engine with the IR handling the format-specific losses explicitly. It reads the custom
properties that the legacy addons already set (`fmdl_material_shader`, `fmdl_texture_role`,
`fmdl_alpha_flags`, etc.) and writes them to the tomls. This path needs no Rust inside Blender —
conversion happens at compile time in `model_convert`.

**Material editing panel.** All PES material settings — both those that map to Blender material
properties (for viewport preview) and those that have no Blender equivalent (pure PES fields) —
are shown in a single panel, so the user edits everything in one place without jumping between
Blender's native material tabs and a separate PES extension panel.

**What users call it.** The community names model files by engine, not by format: `.fmdl` is a
"fox model file", `.model` a "pre-fox model file". The authoring format joins that family as the
**"global model file"**, extension `.glb` — one file that compiles for both engines, opens in
Blender and any glTF viewer, and echoes its letters the way `fmdl` echoes "fox model". This is a
nickname, never an expansion: user-facing text (GUI labels, messages, the user docs, the
extension's export menu item) says "global model file" or ".glb" and **never writes "glb stands
for"**; the one bridge to the real name sits beside the Blender export settings — *the format is
glTF 2.0, which is what Blender's own exporter is called; choose `.glb`* — so a member who meets
"glTF" in a menu is not contradicted. `.gltf` + `.bin` stays accepted and is mentioned once, in
the format plan's file rules, never in messages (a message names the file, not the format).
Plans, code, identifiers and the `PES_*` extension names keep **glTF**, the correct technical term
and the one the `gltf` crate, the spec and Blender's Python API use.

The two paths coexist as modules: Path A for legacy compatibility and direct editing, Path B for new
authoring targeting both engines. When the community fully migrates to glTF authoring, the
extension's `fmdl`/`pes_model` codec modules and the `python_bindings` crate are retired together.

**One schema, one canonical converter.** The extension's codecs write the format plan's schemas,
`gltf_to_ir` validates them, and any glTF + tomls the extension exports must import cleanly through
`gltf_to_ir`. Since the extension can technically cross formats itself (import `.model`, export
`.fmdl`), the documented rule is: Blender round-trips are for *authoring*; migrating a player folder
between formats is the Studio's IR conversion (see "One schema, one canonical converter" in the
format plan).

The compiler materializes ZIP entries lazily or decompresses solid 7z data into an archive-owned
buffer without creating a mutable extraction tree, so archive-level compression handles submitted
size and the format needs none of its own (see "File size and compression" in the format plan).

### Pipeline integration

```
Path A (legacy):  Blender ──pes-models fmdl/pes_model codecs──→ .fmdl / .model
                         (PyO3 wheel from python_bindings crate)

Path B (new):     Blender ──pes-models glTF codec (native export + materials.toml)──→ .gltf + .bin + textures + materials.toml
                                                            │
                                                    ┌───────┴───────┐
                                                    │  model_       │
                                                    │  convert      │
                                                    │  glTF→IR      │
                                                    │  →FMDL or     │
                                                    │   .model      │
                                                    └───────────────┘
```

The compiler detects the model format by file extension (`.fmdl`, `.model`, `.gltf`/`.glb`) and
routes accordingly. glTF sources always go through the IR. PES-native sources skip the IR for
same-format targets — **except** when an IR-level operation applies: hand auto-split (models with
`skf_` bone weights are imported, partitioned, and exported back to the same format) and
cross-version skeleton retargeting (see "Skeleton retargeting and bone conformance"). Both are lossless
same-format IR round-trips. All other same-format processing (ID replacement, texture path
rewriting, model merging on both engines, validation) is format-native and never touches the IR.

---

## Model Format Conversion

**Conversion is not offered as a standalone tool.** It is integrated into the Team compiler: exports
may contain models in any supported format (`.fmdl`, `.model`, `.gltf`), and the pipeline converts
them to the target PES version's format at compile time. The logic lives in the `model_convert` lib
crate.

### What the converters do

**FMDL → .model (19→16):**
1. Parse FMDL binary → FMDL object
2. Decode mesh splitting (reassemble sub-meshes)
3. Decode split vertex encoding (restore vertex order)
4. Decode anti-blur (strip duplicate meshes)
5. Remap bones through `missingBones` table (FMDL has more bones than .model) — in the Studio, the
   per-version retargeting pass (see "Skeleton retargeting and bone conformance")
6. Convert vertices/faces (reverse face winding, Vector4→Vector3 normals)
7. Convert materials (FMDL shader/technique → .model shader)
8. Encode mesh splitting for .model format
9. Encode split vertex encoding
10. Write .model binary

**.model → FMDL (16→21):**
1. Parse .model binary → Model object
2. Decode mesh splitting
3. Decode split vertex encoding
4. Build bone hierarchy (lookup in PesSkeletonData, set parent/children)
5. Convert vertices/faces (reverse winding, Vector3→Vector4 normals/tangents)
6. Create static bone for boneless meshes (fox PES requires bone assignment)
7. Convert materials (.model shader → FMDL shader/technique, assign textures, calculate alpha/shadow
   flags)
8. Calculate bone bounding boxes
9. Create mesh groups
10. Encode mesh splitting for FMDL format
11. Encode split vertex encoding
12. Encode anti-blur (generate duplicate meshes)
13. Write FMDL binary

### Performance-critical operation: mesh splitting

Mesh splitting is a graph partitioning algorithm that cuts meshes exceeding hardware limits (32
bones, 65535 vertices, 21845 faces) into sub-meshes. It involves:
- Building bone-to-vertex maps
- Greedy graph growing: iteratively grow sub-meshes by adding adjacent faces, checking bone count
  constraints
- Vertex deduplication across sub-mesh boundaries
- Face reordering

This is O(V×F) in the worst case — tight loops over vertex arrays with per-iteration set operations.
This is where Rust's performance is essential:

| Language | Estimated time per complex model |
|----------|--------------------------------|
| Python | 1–5 seconds |
| Rust | 10–50 milliseconds |

### Skeleton data

`PesSkeletonData.py` (55KB) is hardcoded bone matrices, parent relationships, and positions for
*one* PES skeleton. The Studio replaces it with the games' own skeleton files, **one set per PES
version**, embedded in the binary (a few KB each) and exposed as constants:

```rust
pub struct PesBone {
    pub skl_parent: Option<&'static str>,    // the SKL file's own parent_index
    pub render_parent: Option<&'static str>, // the mesh-splitting/Blender hierarchy (PesSkeletonData's)
    pub matrix: [f32; 12],                   // model-space 3×4 bind transform from the SKL
    pub start_position: Option<Vector3>,     // Blender bone display only
    pub end_position: Option<Vector3>,
}

pub struct VersionSkeletons {
    pub body: phf::Map<&'static str, PesBone>,
    pub face: Option<phf::Map<&'static str, PesBone>>,   // Fox versions ship face.skl
    pub hand_l: Option<phf::Map<&'static str, PesBone>>, // Fox versions ship hand_l/hand_r.skl
    pub hand_r: Option<phf::Map<&'static str, PesBone>>,
    pub body_skl_bytes: &'static [u8],       // the verbatim game file, for Fox template injection
}

pub fn skeletons(version: PesVersion) -> &'static VersionSkeletons;
```

**Provenance.** Every table is transcribed from a game file, never invented. The files themselves
are checked in under `resources/skeletons/pes{15,16,17,18,19,21}/` (extracted once, stored
decompressed; see the README there) and are what the build embeds and what a `build.rs`/dev tool
generates the constants from — the Python literals are not copied. Where each version keeps its
player skeletons in the game data:

| Version | Archive | Path | Files |
|---|---|---|---|
| PES15 | `dt32.cpk` | `common/character1/model/character/body/body.skl` (WESYS-compressed) | body only |
| PES16 | `dt32_win.cpk` | same path | body only |
| PES17 | `dt32_win.cpk` | same path | body only |
| PES18 | `dt00_x64.cpk` | `Asset/model/character/#Win/common_package.fpk` → `/Assets/pes16/model/character/common/{body,face,hand_l,hand_r,boots}.skl` | body, face, hands, boots |
| PES19 | `dt00_x64.cpk` | same | same |
| PES21 | `dt00_x64.cpk` | same | same |

PES20 was never used in a 4cc event and is not installed; **PES20 is assumed to equal PES21** until
its files are checked. Pre-Fox ships no hand or face SKL (its remaining `.skl` files are cutscene
mobs/props and 2D overlays), which is why `.model` files carry only names and matrices and leave the
hierarchy to the table; the hand and face hierarchies for pre-Fox targets come from PES19's files,
as `PesSkeletonData.py` already did. The `render_parent` column is `PesSkeletonData`'s own (the SKL
files' `parent_index` makes almost every `dsk_*` bone a root, so it cannot serve as a hierarchy for
mesh splitting or Blender armatures).

**The body skeleton is not one skeleton.** Parsing all six `body.skl` files side by side:

| Version | Bones | Compared with PES21 |
|---|---|---|
| PES15 | 76 | 12 exclusive bones (`dsk_belly`, `dsk_chest`, `dsk_upperarm_{b,m,t}_{l,r}`, `tip_belly`, `tip_chest`; `dsk_thigh_{l,r}`, which return in PES19); **lacks** `dsk_deltoid_*`, `dsk_trapezius_*`, `dsk_upperarm_long_*`; 20 shared bones have a different rest pose — `dsk_scapula_r` (1.89 max component delta), `dsk_scapula_l`, `dsk_pectoralis_*` (0.52), `dsk_belly_scale`, and the **whole forearm/hand chain** (`sk_forearm_*`, `sk_hand_*`, `dsk_wrist_*`, `dsk_forearm*_*`, `dsk_elbow_*`, 0.18–0.50) |
| PES16 | 70 | identical bone set and transforms to PES17 |
| PES17 | 70 | `dsk_deltoid_*` rest pose differs (0.25) |
| PES18 | 76 | PES17 + `dsk_belly_{f,o,ba}_{l,r}` (as roots; PES21 hangs them under `dsk_pos_belly_*`) |
| PES19 | 116 | adds `dsk_pos_*` helpers, `dsk_hip/leg/pants/thigh_*`, `dsk_upperarm_skin_*`; **8 bones PES21 lacks**: `dsk_pos_{1,2,3}_wrist_*`, `dsk_pos_clavicle_*`, `dsk_pos_trapezius_*` |
| PES21 | 175 | adds sleeves, sternum, underarm, kneeback, thighmain, hem fakes, `pos_arm_target_*`; not a superset of PES19 |

Consequences: PES21 is the right *Fox injection template* (Red's practice; older Fox games tolerate
extra bones in an injected `.skl`), but it is the wrong source of **bind matrices for a `.model`**
and the wrong **bone-set reference** for any version but itself.

### Skeleton retargeting and bone conformance

A model authored against one version's skeleton and compiled for another has two distinct problems,
which the legacy tools conflated:

- **A. Bones the target lacks.** Their weights must be transferred to a bone the target has.
- **B. Bones the target has at a different rest pose.** The mesh is bound to the *source* pose:
  its inverse bind matrices (inline in a `.model`, in the glTF skin, in an FMDL's companion `.skl`
  or the template) describe where each bone was when the vertices were authored. If those matrices
  do not match the target game's runtime skeleton, vertices rotate about the wrong pivots — fine at
  rest, wrong in any pose that flexes the joint.

The `4cc-model-simplifier-15` (`Engines/python/simplify.py`) and the 19to16 converter are the
prototypes. Both solve A with a hand-written table (`missingBones`: `dsk_deltoid_*` →
`dsk_upperarm_*`, `dsk_trapezius_*` → `sk_shoulder_*`, `dsk_upperarm_long_*` → `dsk_upperarm_*`).
Neither solves B: both write **PES17's** matrices (`PesSkeletonData`) into models destined for other
versions, and the simplifier's "full" mode papers over the worst B cases by transferring weights
away from the displaced bones (`movedBones`: `dsk_belly_scale` → `sk_belly`, `dsk_pectoralis_*` →
`sk_chest`, `dsk_scapula_r` → `sk_shoulder_r`) — losing the deformation those bones provide. The
forearm/hand chain, displaced just as much in PES15, was never handled, which is the probable cause
of the **broken wrist pose in some PES15 animations (the pre-match entrance) on 17→15 ports** — a
pose-dependent error is exactly what wrong bind pivots produce. *Hypothesis; verify on a known-broken
port once the pass exists.*

The Studio runs one IR pass, `retarget_skeleton(ir, source_pose, target_version)`, in every export
path after conversion and before mesh splitting:

1. **Source bind pose.** `.model`: the inline matrices, inverted. glTF: `skin.inverseBindMatrices`.
   FMDL: the companion `.skl`; without one, the source is assumed to be **PES21**'s skeleton (the
   only Fox versions in 4cc use are 18/19/21 and their shared bones differ by ≤0.25 on `dsk_deltoid`
   and the toe tips; FMDL files do not record their game version).
2. **Fold bones the target lacks** (problem A). For each used bone absent from the target's body
   table, transfer its weight to the bone named by the **fold table** for that bone — a per-removed-
   bone table, seeded from the two legacy tables and `PesSkeletonData`'s render parents (`dsk_chest`
   → `sk_chest`, `dsk_belly` → `sk_belly`, PES19's `dsk_pos_*_wrist_*` → `sk_hand_*`, …), maintained
   as data next to the skeleton constants. A removed bone with no table entry falls back to the
   **nearest target bone by rest position** and is reported (`bone_folded_for_version`, I, per
   bone); an entry that names a bone the target also lacks is a table bug caught by a unit test.
   Bone groups and vertex mappings are remapped exactly as the simplifier's `simplifyModel` does.
   The SKL parent chain is *not* used: almost every `dsk_*` bone is a root in the games' own files.
3. **Re-bind to the target pose** (problem B). For every surviving bone, re-pose the vertices from
   the source rest pose into the target's — `v' = Σᵢ wᵢ · B_target,ᵢ · B_source,ᵢ⁻¹ · v` over the
   vertex's weights, normals/tangents by the rotational part — and write the **target version's**
   inverse bind matrices into the emitted `.model` bone table or generated `.skl`. Weights are
   untouched: `dsk_scapula_r` keeps driving the shoulder blade, sitting where PES15 expects it.
   Reported once per model as `skeleton_retargeted` (I) when any bone moved more than a tolerance
   (1e-3); a same-version compile is a no-op by construction. Bones the target's tables do not know
   at all (a genuine custom skeleton) keep the model's own matrices, as before.
4. **Hands and face** conform the same way against the target's `hand_*`/`face` tables where it
   ships them (Fox) and PES19's otherwise.

The two legacy tables become regression fixtures: PES19→PES16 and PES17→PES15 must fold exactly the
`missingBones` those tools folded, and must *not* fold the `movedBones` (retargeting handles them).
Retargeting is also what makes the **PES19→PES21** compile correct — today Red leaves PES19's eight
`dsk_pos_*` bones dangling in a PES21 FMDL; whether the game ignores or misrenders them is untested,
and folding makes it moot. A manual per-folder override for fold targets is deliberately not
specified: the table plus nearest-bone fallback should cover real exports, and `settings.toml` is
kept to savefile settings; revisit if a case turns up.

**Cost.** Re-binding is linear-blend skinning evaluated once, offline. Per surviving bone one delta
matrix `Dᵢ = B_target,ᵢ · B_source,ᵢ⁻¹` is precomputed (≤ 175 small matrix products); per vertex the
position is `Σ wᵢ Dᵢ v` over at most four weights (≈ 50 multiply-adds) and the normal/tangent take
the rotational part (≈ 40 more) — on the order of 100 flops per vertex, done in place. A 20k-vertex
player model is ~2 M flops, well under a millisecond single-threaded; the largest 4cc models
(~100k vertices) stay in single-digit milliseconds. It is not measurable next to mesh splitting
(10–50 ms per model) or texture encoding (hundreds of ms), and it runs at all only when a
native-format pre-check (bone names and the two versions' tables, no geometry read) finds a used
bone that the target lacks or poses differently beyond tolerance — same-version compiles, and
cross-version compiles of models that only use unchanged bones, never enter the pass. What it does
add for a **same-format** cross-version compile (a PES21-authored FMDL for PES18) is the IR
round-trip it needs to run at all, which is why it is listed with hand auto-split as an explicit
exception to the "same format skips the IR" rule; that round-trip
is the lossless one the plan already tests, and it costs a few milliseconds of parse/serialize.

### SKL binary format

Fox-engine `.skl` files carry the bone list and bind-pose transforms for a corresponding `.fmdl`.
Most models use the compiler's template skeletons (`body.skl` → `boots.skl` / `fcl_hair_sim.skl`);
custom `.skl` files are rare and only needed for models with a custom pose. The binary format is
simple (reference parser: `examples/skl.py`):

```
Header (12 bytes — 3 u32s):
  u32 magic       = 12        (version/identifier)
  u32 bone_count  = N
  u32 record_size = 56

Bone records (N × 56 bytes, starting at offset 12):
  u32 name_offset     — byte offset into the file for this bone's null-terminated name
  i32 parent_index    — -1 for root, else 0-based index into the bone array
  f32[12] transform   — 3×4 row-major [rotation_3x3 | translation_3x1]

Name table (at offset 12 + N×56):
  concatenated null-terminated ASCII strings, padded to 4-byte alignment
```

Each transform row is `[rot_x, rot_y, rot_z, translation]`, giving a 3×3 bind-pose rotation plus a
model-space position per bone. The positions correspond to the `PesSkeletonData._positions`
reference, but template variants can differ numerically; do not require exact equality to that
hardcoded reference. Preserve the SKL parent hierarchy separately from the render hierarchy. The `fmdl` crate owns the SKL codec alongside
the FMDL codec (same format family — both are Fox-engine binary formats consumed together at pack
time).

### SKL and glTF: native integration, not a companion

A glTF-authored model stores its skeleton natively in the glTF `skin` (field mapping in the format
plan, "Skeleton: native glTF skin, not a companion SKL"). When the target is Fox, `ir_to_fmdl`'s
caller reconstructs the `.skl` binary from the IR bones — which `gltf_to_ir` filled from the skin's
joint hierarchy and bind data, applying the inverse coordinate/local-to-model-space mapping.

For Fox-native `.fmdl` inputs staying native, the companion `.skl` can travel as opaque
pass-through bytes. Native→IR conversion and Blender import also decode it for skeleton
reconstruction; the SKL codec is not limited to glTF-authored inputs. The [Aesthetics export plan's "SKL pairing"
section](aesthetics_export.md) describes how paired SKLs travel with their models through links and
merges regardless of source format.

### GPU offloading

**Not worth it.** The workload doesn't match GPU strengths:
- Mesh splitting: branch-heavy, irregular memory access, set operations — GPUs perform poorly
- Vertex conversion: 5K-20K vertices is too small to justify GPU dispatch overhead (~100μs)
- Matrix inversion: ~100 tiny 3×4 matrices — CPU does this in microseconds

Keep model geometry conversion on the CPU and benchmark the port rather than promising specific
latencies. Texture encoding is a separate workload: the [library crates plan](libs.md) includes
first-release desktop GPU BC7 for PNG-heavy PES 19–21 builds, with CPU fallback. That texture-only
exception does not change this plan's CPU geometry processing or deterministic model output.

---

## Testing: IR roundtrips

The equality assertions below are schematic: compare normalized geometry, bindings, and retained
metadata, not arbitrary native bytes or glTF document structure. Decoding/re-encoding split meshes,
regenerating anti-blur, or consolidating material files can change representation without changing
the supported semantics. Native codec byte-preservation tests remain separate; unsupported fields
must not disappear under a blanket claim that IR routing is lossless.

```rust
#[test]
fn fmdl_roundtrip_lossless() {
    let original = parse_fmdl("test_data/face_high.fmdl");
    let ir = fmdl_to_ir(&original);
    let roundtripped = ir_to_fmdl(&ir);
    assert_eq!(original, roundtripped);
}

#[test]
fn model_roundtrip_lossless() {
    let original_model = parse_model("test_data/face.model");
    let original_mtl = parse_mtl("test_data/face.mtl");
    let ir = model_to_ir(&original_model, &original_mtl);
    let (roundtripped_model, roundtripped_mtl) = ir_to_model(&ir);
    assert_eq!((original_model, original_mtl), (roundtripped_model, roundtripped_mtl));
}

#[test]
fn gltf_roundtrip_lossless() {
    let original_gltf = parse_gltf("test_data/face_high.gltf");
    let original_mats = parse_materials_toml("test_data/materials.toml");
    let ir = gltf_to_ir(&original_gltf, &original_mats);
    let (roundtripped_gltf, roundtripped_mats) = ir_to_gltf(&ir);
    assert_eq!(original_gltf, roundtripped_gltf);       // PES_bone/PES_mesh extensions preserved
    assert_eq!(original_mats, roundtripped_mats);       // material definitions preserved
}
```

---

## Testing: conversion against reference outputs

Test files with known-good conversions (from the existing converters) serve as reference:
```rust
#[test]
fn fmdl_to_model_matches_reference() {
    let fmdl = parse_fmdl("test_data/face_high.fmdl");
    let ir = fmdl_to_ir(&fmdl);
    let (model, mtl) = ir_to_model(&ir);
    let reference_model = parse_model("test_data/face_high_converted.model");
    let reference_mtl = parse_mtl("test_data/face_high_converted.mtl");
    assert_eq!((model, mtl), (reference_model, reference_mtl));
}
```
