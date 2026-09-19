# 4cc Studio — Model conversion plan: Hand auto-split

Part of the [Model conversion plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

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
bones** are identified by name convention: they are prefixed with `skh_` (the finger bones:
`skh_thumb_mata_l`, `skh_index_dip_r`, ...) and suffixed with `_l` or `_r` for laterality. No
pre-defined list or `PES_SKELETON` lookup is needed — the prefix/suffix match is the complete
identification. The games' own `hand_l.skl`/`hand_r.skl` (PES 18, 19, 21) hold 19 `skh_` bones each
plus five bones shared with the body (`sk_forearm`, `sk_hand`, `dsk_forearm`, `dsk_wrist`,
`dsk_forearm_t`), and the `skf_` prefix belongs to the **face** skeleton (33 bones in `face.skl`);
an earlier version of this plan named `skf_` here, which would have cut faces off at the jaw.

No opt-in is needed: vertices with positive `skh_` weights seed the hand selection. Unused `skh_`
names in a bone list do not trigger splitting; the check follows actual vertex weights.

### Split

When hand weights are detected, import to the IR and perform the following mesh operations
**in-process in Rust**, independently for each hand. Blender's commands describe the intended
selection/separation semantics only: the compiler neither launches Blender nor uses `bpy`, and
compiling hundreds of models must not require a Blender installation or per-model external process.

1. **Select** vertices with any positive weight on that hand's `skh_` groups (`_l` or `_r`).
2. **Grow once** along the mesh topology — equivalent to one Blender **Select More** (`Ctrl +`)
   step in vertex mode, bringing in the neighboring row at the wrist. This is not a distance-based
   cut, and shared wrist/forearm weights do not exclude a vertex from the expanded selection.
   A *vertex* here is Blender's: the native formats store one entry per loop, so entries sharing
   position, bone indices and weights (the vertex-loop encoding's topological key) are one vertex
   for selection and growth, and a UV or normal seam at the wrist does not stop the growth.
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
compiler plan](../team_compiler/README.md)). The split parts appear as virtual model files in the player
folder's model list; categorization handles them normally — the body becomes face content, the
gloves become `glove_l`/`glove_r`. The rest of the pipeline (merge, ID assignment, packing) is
transparent to whether the gloves were auto-split or authored as separate files.

### Where it lives

`model_convert`, as an IR-level operation (`split_by_skeleton_group`). It operates on the IR's
normalized bone mapping and identifies hand bones by the `skh_` name convention. It is a one-way
semantic transformation, not a same-format round-trip, so it does not belong in the
`fmdl`/`pes_model` format crates (which own format-native operations for lossless round-trips).

### Scope

Gloves only. Boots use the same body skeleton as the face model — there is no skeleton-exclusivity
problem to solve, so no auto-split is needed or offered for boots.

---
