# 4cc Studio — Unified model format plan

The unified model format is the Studio's own authoring format for PES player models: a **glTF model (`.glb`)
with two small PES extensions** plus a **`materials.toml`** sibling holding the PES material
properties. One model folder authored in this format compiles to either PES engine — Fox (`.fmdl`,
PES 18–21) or pre-Fox (`.model` + `.mtl`, PES 15–17) — through the `model_convert` lib's IR (see the
[Model conversion plan](model_conversion.md), which owns the IR, the importers/exporters, and the
Blender integration that writes this format). The [Team compiler plan](team_compiler.md) owns where
model folders sit inside an export and how models are categorized and merged.

**Design goal: user friendliness.** The format exists so that a member can author one model that
converts to `.model` and FMDL alike **without losing any customizability, in the simplest possible
way**. If it were harder to write than the native formats, most users would keep using those. Hence:
the common case needs almost nothing (a material entry can be as short as `[face]` plus a
`family`), every engine-specific knob remains reachable for the few who need it, and every
auto-generated file carries inline comments explaining its fields.

---

## Why glTF

Instead of parsing `.blend` files directly (Blender's binary format changes between versions, no
mature Rust parser) or using FBX (proprietary, immature Rust support), the format is **glTF with PES
extensions**:

- **Blender exports glTF natively** — no addon needed for base mesh/skeleton/UV data; it has been a
  stock exporter since Blender 2.80
- **Rust has a mature parser** — the `gltf` crate is production-quality
- **Other 3D tools support glTF** — Maya, 3ds Max, Godot, Sketchfab, Windows 3D Viewer, etc.
- **Extension mechanism** — PES-specific structural metadata (bones, mesh splitting) stored as
  custom JSON extensions; material properties live in TOML files alongside the model
- **Binary bulk data** — vertex/face data is binary, not text; the JSON scene description is ~5-10%
  of total size.

**Stock Blender is enough.** The plugin (`pes-models`, see the [Model conversion
plan](model_conversion.md)'s "Blender integration") adds material preview and toml editing inside
Blender, but a model exported from a plugin-less Blender with a hand-written `materials.toml` beside
it is a valid, compilable model. This is a stated guarantee, not an accident, and it constrains the
format: the PES extensions are optional with fallbacks (see "PES extension schema"), standard glTF
material data is ignored rather than rejected, and the file keeps the standard extension so every
glTF-aware tool opens it. A custom extension (`.pes`) was considered for friendliness and rejected:
it would make the plugin mandatory for any Blender work and tie the format's usability to the
plugin keeping pace with Blender releases.

**One file: `.glb`.** glTF has two containers for the same data. `.gltf` is a JSON text file that
references its binary vertex/index buffers by URI — conventionally a sibling `.bin` file, so a model
is always at least two files (the buffers *can* be inlined as base64 in the JSON, at ~33% size cost).
`.glb` packs the JSON and the binary buffer into one file. The Studio writes `.glb` (Blender's
default too) and the documentation talks about `.glb` models; `.gltf` + `.bin` is accepted as the
same format with no difference in handling. Textures are external files in both cases (see below),
so a model folder is: one `.glb` per model, the textures, and the material toml(s).

Materials are **never carried in the glTF**. The model's textures are sibling files anyway, JSON
has no comments, and tomls-as-siblings make sharing unambiguous: copy a `.glb` + its material toml +
textures into another player's folder and the materials travel with the model, touching nothing
already there — exactly like copying `.model` + `.mtl` files today.

---

## A model folder

```
15 - Snuffy/
├── face_high.glb             (the model: scene, skin, geometry; material objects carry only names)
├── face.png  face_nrm.png    (textures: any accepted image format, referenced by stem)
├── materials.toml            (PES material properties, keyed by material name; catch-all)
├── body.materials.toml       (optional name-matched material file — see "Material files")
├── hair.materials.toml.common (optional link to Common/hair.materials.toml — see "Link files")
└── hair.png.common           (optional link to Common/hair.png: a shared texture, packed once per team)
```

The compiler detects the model format by file extension (`.fmdl`, `.model`, `.glb`/`.gltf`) and
routes accordingly; glTF sources always go through the IR. A folder may hold both a native set and
glTF files ("dual-set folders"): `.mtl` serves `.model`, `.materials.toml` serves `.glb`/`.gltf` —
different extensions, different consumers, no ambiguity. Wherever this plan says "glTF" it means
either container; "the model's stem" is the file name without `.glb`/`.gltf`.

**The file name carries the part type.** A model's stem is `<anything>_<suffix>`, and the suffix —
always last — says what the model is: a Fox allowed name (`face_high`, `boots`, `glove_l`, …) or a
pre-Fox `face.xml` type (`gloveL`, `handL`, `uniform`, `eye`, …, or `model_type_<x>` for one the
table doesn't know). The compiler maps it to a Fox destination and a pre-Fox type through one table
("Model names: a free part plus a suffix" in the [Aesthetics export plan](aesthetics_export.md)); the type
is used only when targeting pre-Fox. Nothing about the part type lives inside the glTF or the
material toml, so the same `.glb` compiles typed on pre-Fox and merged into its Fox slot.

### Dependencies and images

- **URI safety and locality.** External buffer/image URIs are percent-decoded and canonicalized
  into the shared `vtree::ScopePath` type and must remain inside the **source model folder**, not
  merely somewhere beneath the export root. Absolute paths, parent traversal, cross-folder
  references, network URLs, and non-`data:` URI schemes are rejected before materialization. This
  keeps folder tasks independently ownable and keeps `model_convert` reusable by the Balls compiler
  without depending on the aesthetics export format.
- **Embedded data.** Standard GLB buffer views and base64 `data:` URIs are accepted. Malformed or
  unsupported MIME types are validation errors; remote fetching is never performed.
- **Accepted image formats.** Any of these may be used as a texture source for any model format
  (FMDL, .model, glTF) — they are fully interchangeable:

  | Format | Decoder | Extensions |
  |--------|---------|------------|
  | DDS (BC7/BC5/BC3/BC1/…) | `texture2ddecoder` | `.dds` |
  | FTEX (Fox) | `ftex` crate | `.ftex` |
  | PNG | `image` crate (pure Rust) | `.png` |
  | JPEG | `image` crate (pure Rust) | `.jpg`, `.jpeg` |
  | BMP | `image` crate (pure Rust) | `.bmp` |
  | WebP | `image` crate (pure Rust) | `.webp` |
  | TGA | `image` crate (pure Rust) | `.tga` |
  | TIFF | `image` crate (pure Rust) | `.tif`, `.tiff` |

  The `image` crate supports additional pure-Rust formats (QOI, PNM, Farbfeld, HDR, OpenEXR, GIF,
  ICO) that are not accepted because they serve no game-texture use case (animation, HDR, icons,
  obscure academic formats). AVIF remains outside the initial decoder set; its backend requirements
  depend on the selected library/version. QOI can be added if community demand arises. See the
  [library crates plan](libs.md) for the full format matrix and the PES texture boundary (which
  codecs each PES version gets).
- **Stem-based texture references.** Material references name textures by **stem** — filename
  without extension — as do FMDL path tables (the game appends `.ftex`) and Studio-format `.mtl`
  files. The compiler scans the model folder for image files, builds a stem→file map, and resolves
  each referenced stem to the actual file. Two image files with the same stem but different
  extensions (e.g. `face.dds` and `face.png`) is a conflict (`texture_stem_conflict`) — both would
  claim the same reference, and the compiler can't pick one. The rule applies within a single model
  folder; different folders may reuse the same stem independently.
- **PNG is the expected norm.** Blender cannot write DDS natively, so glTF-authored models naturally
  carry PNG textures; conversion to the target texture format happens at compile time (see the
  [library crates plan](libs.md)).

---

## Kit-dependent assets (`kitN`)

Some parts of a player model must change with the kit the user picks in the pre-match menu — a
shirt's cloth texture, undershorts, tape matching the socks. The community's modded exes (Fox and
pre-Fox alike) implement this with a **path magic**: a texture path — or, in a pre-Fox `face.xml`, a
model path — containing a reference token is rewritten at load time to the variant spelling for the
selected kit slot, and the game loads whichever variant file that names. The token may sit anywhere
in the path, including a folder name. The unified format keeps this mechanism and gives it a
readable spelling, which the exes are being modded to look for **in place of** the historical one:

| Spelling (export and game files alike) | Meaning |
|---|---|
| `kitN` (literal `N`) | the *reference*: "the variant for the kit number selected in PES" |
| `kit1` … `kit9` | the variant used when kit 1 … 9 is selected |

The reference is spelled `kitN`, not `kit0`, because `kit0` reads as "the texture with a 0 on it"
while `kitN` reads as the placeholder it is. The compiler writes both spellings **verbatim**; there
is no output-side rewrite.

**The historical `u0XXXp0` magic is legacy.** It is what the exes matched so far — `u0XXXp0`
rewritten to `u0XXXp1`, `u0XXXp2`, … — and the Studio compiler does **not** know it: the token is
just another stem fragment, so a leftover reference fails the ordinary texture-existence checks like
any other missing texture. There is no dedicated finding for it — a specific error would be partial
support, complexity for no gain, since every existing export goes through the Export upgrader before
Studio can use it. The upgrader is the only Studio component that reads the legacy spelling, and it
rewrites it to `kitN`/`kit1`…`kit9`. The legacy pattern is `u0` + **any three characters** + `p` +
digit: authors wrote the `XXX` placeholder literally *or* baked the real team ID in (`u0872p1.dds`),
and both forms mean the same thing, so both are detected and both are renamed.

There is one number. PES lets the user pick a single kit number per team, which selects the
outfield and goalkeeper kit textures together; with `kitN` the export supplies its *own* files and
only that number matters, so a goalkeeper's model uses `kitN` like anyone else's and its
`kit1`…`kit9` files are simply whatever that model should wear when that kit is picked. (The `p` in
the legacy magic was the exes' spelling, not an "outfielders only" restriction.)

Rules:

- The token is a stem fragment delimited by `_`, `-`, `.`, or the ends of the name — `pants_kitN`,
  `kitN_pants`, `kitN` — anywhere in a texture stem or a model file name. The conventional place is
  the suffix. `skitN` is not a token.
- **A `kitN` reference never exists as a file.** A material role set to `pants_kitN` means "the
  files `pants_kit1`, `pants_kit2`, … in this folder" (the folder of the material file that set the
  stem — so variants in `Common`, referenced from a `*.materials.toml.common` link or by a
  Common-linked model, are how one set of kit textures is shared by many players, which is what the
  old `Kit Textures` sharing did). Auto-detection gains a third candidate: for role `base` of
  material `pants`, the compiler tries `pants_bsm`, `pants`, then `pants_kitN` (present when
  `pants_kit1` exists); other roles try `{m}{suffix}` then `{m}{suffix}_kitN`.
- **Every kit number the export defines needs a variant.** The team's `Kits/` folder lists its
  player kit slots (`p1`…`pN`); a `kitN` reference whose variant for one of those numbers is
  missing is `kit_variant_missing` (W) and the compiler **copies the lowest existing variant** into
  the gap, so the model never shows a missing texture in game. Variants for numbers the export does
  not define are emitted as they are (harmless).
- **Per-kit models are pre-Fox only.** Model files `pants_kit1.glb` + `pants_kit2.glb` (any source
  format) form one variant set: pre-Fox emits a single `face.xml` entry naming `pants_kitN` and the
  variant files beside it; each variant's materials resolve by the normal stem name matching
  (`pants.materials.toml` matches both). Fox has no model-path indirection, so the compiler uses the
  lowest variant only and reports `kit_variant_model_fox` (W).
- **Output naming.** The token passes through unchanged into emitted file names and into every path
  the compiler writes — FMDL path tables, `.mtl` sampler paths, `face.xml` model paths. Nothing in
  Studio, the Blender side included, ever produces `u0XXX`.

**What this replaces.** Two older ways of getting "the kit texture" onto a model existed:

- The pre-Fox `uniform` model type in `face.xml`, which makes the game ignore the material's diffuse
  path and use the active kit texture plus the kit config's pattern normal map. It is a native game
  feature and stays available on pre-Fox targets through the existing filename-prefix typing (a model
  named `uniform*` gets the type); it has no Fox counterpart and is not part of the cross-engine
  format.
- `dummy_kit` (and `dummy_kit_back/_chest/_leg/_name/_nrm/_srm`), a reserved texture name the modded
  exes replace at load time with the active kit texture of that role. It is legacy — a second
  vocabulary for what `kitN` expresses with the export's own files — and gets **second-class
  support**: the compiler treats these stems as **game-provided** (like the Fox `dummy_nrm`/`dummy_srm`
  fallbacks), so a material may name them, the texture-existence check is skipped for them, and the
  path is emitted verbatim for the exe to substitute. Nothing is copied or generated for them: the
  substitution logic now lives in the exes and is stable, unlike the earlier Sider-script era that
  made Red's kit-1 fallback copies necessary. New exports should use `kitN`.

---

## glTF conventions

### Materials: only the name is read

The glTF material object's `name` is the key into the material tomls; that is the only material
field the compiler reads. The Studio and the `pes-models` codec write name-only materials:

```json
{
  "materials": [{ "name": "face_bsm" }]
}
```

Stock Blender writes standard PBR material data too (`pbrMetallicRoughness`, a base-color texture
reference, and — for `.glb` with the exporter's default image setting — the texture bytes embedded
in the file). All of it is **ignored, not rejected**: PES data has no representation in glTF
materials, and the tomls are the only source. Embedded images are never used as texture sources —
textures are the folder's files, referenced by stem — so an embedded copy only wastes space; the
compiler reports `gltf_embedded_image_ignored` (I) once per model to nudge the user towards the
exporter's "Images: None" option.

### PES extension schema

PES-specific structural metadata is stored in a `PES_` extension namespace in the glTF JSON:
`PES_bone` (skeleton metadata) and `PES_mesh` (mesh splitting metadata). Materials carry no
extension. **Both extensions are optional** — a model without them (a stock Blender export) compiles
with these fallbacks, which are the same ones native models without the data already get:

| Extension | Carries | When absent |
|---|---|---|
| `PES_bone` | `sklParent` (the SKL parent hierarchy, distinct from the render hierarchy) and `globalPosition` per bone | Looked up by bone name in the template skeleton; a bone name unknown to the template with no `PES_bone` is `gltf_bone_unknown` (E) — a custom skeleton must carry its metadata |
| `PES_mesh` | Mesh-splitting group identity, so a same-format round-trip re-splits meshes identically | The compiler splits from scratch at compile time (it must be able to anyway — hardware limits are checked on every model); only byte-stability of round-trips is lost |

The extensions are listed in `extensionsUsed`, never in `extensionsRequired`, so every glTF-aware
tool opens a Studio-written file (unknown extensions are skipped on import; a plugin-less re-export
drops them, which the fallbacks above absorb — the `pes-models` codec preserves them).

```json
{
  "nodes": [{
    "name": "sk_head",
    "extensions": {
      "PES_bone": {
        "matrix": [1,0,0,0, 0,1,0,0, 0,0,1,0],
        "sklParent": "sk_neck",
        "globalPosition": [0, 1.64, 0.054, 1.0]
      }
    }
  }]
}
```

### Skeleton: native glTF skin, not a companion SKL

A glTF model's skeleton is stored **natively in the glTF**, not as a companion `.skl` file:

| SKL field | glTF equivalent |
|---|---|
| Bone name | `node.name` |
| Parent index | `node.children` hierarchy |
| Model-space 3×4 bind transform | Parent-local `node.matrix`/TRS plus matching `skin.inverseBindMatrices`, with coordinate and matrix-layout conversion |
| Bone count / order | `skin.joints` array order |

glTF has first-class skeleton support via `skins` + `nodes` with `matrix`/`TRS` — no custom
extension is needed for the skeleton itself. `PES_bone` carries the PES-specific `globalPosition`
and `sklParent` metadata that don't map to standard glTF fields (the SKL parent hierarchy is
preserved separately from the render hierarchy).

At compile time the skin's bind data goes wherever the target keeps it: for Fox the compiler
reconstructs the `.skl` binary (only when the skeleton differs from the template — see "SKL
pairing" in the [Team compiler plan](team_compiler.md)); for pre-Fox it is written straight into the
`.model`'s inline per-bone matrix table, which is where that format has always stored its
skeleton. This keeps the glTF self-contained (one file to manage, link,
and pass through `.common` links) and matches the Blender workflow: armatures export as native glTF
skins automatically, with no separate SKL export step. The SKL binary layout and the
skeleton-reconstruction rules for FMDL sources are in the [Model conversion
plan](model_conversion.md) ("SKL binary format", "Skeleton reconstruction from FMDL").

### File size and compression

A `.glb` is comparable in size to the native formats:
- JSON scene description: 50-100KB (5-10% of total)
- binary vertex/face buffer: ~1.3MB for a 20K-vertex model
- Textures: same as current (1-10MB each)

**No format-level compression needed.** Exports may already be submitted as `.zip` or `.7z`
archives, which handle the submitted size. If size becomes a concern, enable
`KHR_mesh_quantization` in the Blender export (one checkbox — converts float32 to int16, ~50%
reduction with negligible quality loss). Do not use `KHR_draco_mesh_compression` — the decoding
complexity isn't worth it for models this size.

---

## Material files

### File pattern

Material files are identified by extension, not by name reservation:

| File | Role | Pre-Fox equivalent |
|------|------|-------------------|
| `materials.toml` | base layer — applies to every glTF in the folder, including those with name-matched files | `materials.mtl` |
| `body.materials.toml` | name-matched (example) — applies to glTFs whose stem startsWith or endsWith `body` | `body.mtl` |
| `body.materials.toml.common` | link to `Common/body.materials.toml`, participating as if it were local (see "Link files") | `body.mtl.common` |

The `.materials.toml` double extension is the structural marker: any file ending in
`.materials.toml` is a name-matched material file (stem = everything before `.materials.toml`);
`materials.toml` is the catch-all base layer; any other `.toml` file (`settings.toml`, etc.) is not
a material file. This avoids fragile name reservations and gets TOML syntax highlighting in every
text editor for free.

### Name matching

(Adapted from Red's `find_mtl_file` logic in `xml_editing.py`.) For each `.glb`/`.gltf` file in the
folder, find its material file(s):

1. **Base layer**: include `materials.toml` whenever present, whether or not any name-matched files
   exist.
2. **Name matches**: also include `*.materials.toml` files whose stem is a startsWith or endsWith
   of the glTF's stem (e.g. `body.materials.toml` matches `body_shirt.glb` and `body_main.glb`).
3. **No material file**: a glTF with neither the base file nor a name-matched toml is a hard error
   (`material_file_missing`) — glTF files at least carry material *names*, but the names alone can't
   render anything. There is no safe default: material settings include texture paths, and a default
   placeholder would produce an unusable model. Erroring at validation time is better than letting
   the user discover the problem after compiling and testing in PES.

### Layering

**Multiple matches layer, they don't compete.** A glTF may match several material files at once —
e.g. `body.materials.toml` providing generic definitions for every `body_*` model while
`body_andy.materials.toml` adds player-specific overrides for `body_andy_*` models. All matched files
apply, in a fixed order from least to most specific — deterministic, unlike Red's `os.listdir`-order
tiebreak. An informational `materials_file_ambiguous` diagnostic reports the layering when it
occurs; intentional layered setups are ordinary usage, not a problem.

**Resolution order** (deep merge per field). For material `face_bsm` in `body_shirt.glb`, the
matched files' `[face_bsm]` entries merge in this order:

1. **Base** `materials.toml` entry (if present) — always least specific.
2. **Name-matched** files' entries, ordered by ascending stem length (for `body_andy_shirt.glb`:
   `body.materials.toml`, then `body_andy.materials.toml`). For equal-length stems, a match at the
   beginning of the model filename wins over a suffix match: apply suffix-only matches first, then
   prefix matches. Thus `carlo_boots.glb` layers `materials.toml`, `boots.materials.toml`, then
   `carlo.materials.toml`, with the latter overriding conflicting fields.
3. **Same stem, linked and local**: a `.common` link and a local file with the same stem both
   apply; the linked (Common) file is the less specific layer (see "Link files").

Filesystem enumeration order never breaks ties.

Each layer is a field-level deep merge over the whole entry, sub-tables included — a more specific
entry that sets only `family` inherits the textures, flags, and engine tables from the less specific
layers; a more specific `[face.textures]` that sets only `normal` inherits `base` from below. This
lets you override one field per group without restating the whole material. A material name
referenced by the glTF with no definition in any matched toml is a hard error (`material_undefined`)
— the model can't render without texture paths, and defaults would only hide the problem.

Resolution fixtures cover base-field inheritance alongside name-matched overrides, longer-stem
precedence, the equal-length `carlo_boots` example under reversed file enumeration, and linked-vs-
local same-stem layering.

### Link files (`.common`)

Like models, material files and textures can be loaded from the export's `Common` folder instead of
being copied into every player folder: an empty link file named after the Common file plus a
`.common` extension (a stray `.txt` suffix is accepted silently, as with every link file — Windows
hides known extensions by default). This is Red's existing `.mtl.common` convention (its
`find_mtl_file` resolves `.mtl.common` names) extended to the Studio format and to textures; Studio
supports all of these:

| Link file | Resolves to | Serves |
|---|---|---|
| `materials.toml.common` | `Common/materials.toml` | this folder's glTFs, as the catch-all layer |
| `body.materials.toml.common` | `Common/body.materials.toml` | this folder's `body*`/`*body` glTFs, name-matched by stem `body` |
| `body.mtl.common` | `Common/body.mtl` | this folder's `body*`/`*body` `.model` files, via Red's MTL cascade |
| `hair.png.common` | `Common/hair.png` | the texture stem `hair` in this folder — for auto-detection, `""`, and explicit stems in any material file, and for native `.mtl`/FMDL references alike |

There is no in-toml spelling for "this texture is in Common" (a `<common>/hair` tag was considered):
one link mechanism covers models, material files and textures; a texture link works with
auto-detection, so the common case — a shared normal map or hair texture — needs no toml editing at
all, only a file that can be copied between player folders; it works for native-format models too,
which a toml tag could not; and Explorer shows where a texture comes from. The cost is one empty
file per shared texture per player folder, which is what the other link kinds already cost.

Rules:

- A link participates in name matching and layering **exactly as if the linked file were local
  with the same stem**. A folder with both `materials.toml` and `materials.toml.common` layers the
  Common file below the local one (field-level merge, local wins); `material_link_layered` (I)
  reports it. Common is the base, local is the override — the same relationship as a Common-linked
  model's own tomls versus the player folder's (below).
- A link whose target does not exist in Common — model, material file or texture — is
  `common_link_missing` (E, folder discarded); the finding's context names the link kind and the
  Common path it looked for.
- **Texture stems resolve in the folder of the material file that set them.** A stem written in
  `Common/body.materials.toml` resolves against Common's images; a stem written in the player
  folder's `body.materials.toml` resolves against the player folder. Auto-detected roles (see
  "Textures") look in the model's own folder. A texture link counts as the texture being present in
  its folder under the linked name. This is what makes a Common material file reusable across
  players: it names the textures it ships with, and a player who wants a different texture overrides
  the stem in a local file, which then resolves locally.
- **Where a texture resolves decides where it is packed.** A texture whose resolved file is in the
  player folder is the player's own: the compiler relocates it to that player's common subfolder in
  the output. A texture whose resolved file is in `Common/` — through a texture link, a stem set in
  a Common material file, or a Common-linked model's own textures — is **referenced in place**: it
  is packed once, in the team's Common output, and every material that resolves to it points there
  (Red's `common/XXX/` path handling on both engines, with the team ID substituted at compile
  time). That is the point of putting a texture in Common: one 2048² hair texture for twenty
  players is one file in the CPK, not twenty. Compiler side: "Texture relocation to common" in the
  [Team compiler plan](team_compiler.md).
- Link files are folder-level references, not model content: the linked file is read from Common,
  never copied into the output, and the link file itself is not emitted. For pre-Fox `.mtl.common`
  the generated XML points the model at the Common MTL path (Red's behavior).

**Common-linked models bring their own materials.** A `.common` model link (`torso.glb.common`)
loads a model whose real folder is Common, so its material files resolve there first: Common's
`materials.toml` and name-matched `*.materials.toml` (matched against the model's stem inside
Common) form the base layers, and the player folder's matched material files layer on top as
overrides. No material link is needed for this case — the link is for the reverse situation, a
*local* model using *shared* material definitions. Pre-Fox `.model.common` links follow Red's MTL
cascade for the same effect: the link folder is checked for a name-matching MTL as an override,
then Common, then the link folder's default/any MTL, then the main folder.

### Comments are app-injected

All auto-generated tomls across the suite ship with predefined inline comments that serve as
simplified per-field documentation — the user never writes comments from scratch. `toml_edit`-based
edits preserve these comments and can inject comments for newly added keys. This principle applies to
`materials.toml`, `settings.toml`, `config.toml`, and AATF parameter files alike.

### Emission

`ir_to_gltf` (the [Player aesthetics editor](player_aesthetics_editor.md)'s convert-to-glTF and the
[Export upgrader](export_upgrader.md)'s optional glTF pass — the Team compiler never emits glTF)
writes **one `materials.toml` per folder** (the catch-all), with app-injected comments — most users
edit one file and have changes reflected across all models. The modular `*.materials.toml` layout and
`.common` links are an advanced opt-in that users create manually; the exporter doesn't generate
them. A native source's material is written with its `family` inferred from the native shader (see
"Shader families") **and** its native fields verbatim in the matching engine table, so the same-format
round-trip is lossless and the other engine gets the family's defaults. Existing textures are
referenced by stem, never re-encoded or embedded.

---

## `materials.toml` schema

One table per material, keyed by the glTF material name. Every key is optional; the shortest valid
entry is an empty table (`[face]`), which means "the standard face shader, textures named after the
material". Keys not in this schema are `material_key_unknown` (W) and ignored — a typo never
silently changes a material, and a toml written for a newer Studio still loads on an older one.
Wrong types or out-of-range values are `material_value_invalid` (E, folder discarded).

### Layout and precedence

```toml
[face]                       # engine-neutral: what the material *is*
family = "shaded"
two_sided = false
transparent = false

[face.textures]              # engine-neutral: canonical roles → texture stems
base = "face"
normal = "face_nrm"

[face.parameters]            # engine-neutral: shader parameters, name → [x, y, z, w]
MatParamIndex_0 = [0, 0, 0, 0]

[face.fox]                   # Fox-only knobs (FMDL); override the family's Fox defaults
shader = "fox3ddf_blin"
technique = "fox3DDF_Blin"

[face.prefox]                # pre-Fox-only knobs (.model/.mtl); override the family's pre-Fox defaults
shader = "Basic_CN"
```

Values resolve for each target engine in this order, later winning:

1. **Family defaults** for that engine (table below).
2. **Engine-neutral keys** (`two_sided`, `transparent`, `antiblur`, `textures`, `parameters`),
   translated to the engine's representation by the fixed tables below.
3. **The engine table** (`[mat.fox]` / `[mat.prefox]`): native fields, applied verbatim. A raw
   `alpha_flags` here still yields to an explicit `two_sided`/`transparent` on the bits those
   booleans own; everything else in the engine table is final.

The engine tables exist for two reasons: lossless round-trips of native models (a converted FMDL
keeps its exact shader, flags, and parameters) and the rare custom setup the families don't cover.
Ordinary authoring never needs them.

### Engine-neutral keys

| Key | Type | Default | Meaning |
|---|---|---|---|
| `family` | string | `"shaded"` | Shader family; selects both engines' shader and defaults. One of `shaded`, `shadeless`, `metal`, `glass`. Unknown → `material_value_invalid`. |
| `two_sided` | bool | `false` | Render both faces. Fox: alpha flag bit 32. Pre-Fox: state `twosided`. |
| `transparent` | bool | `false` | Alpha-blended (semi-transparent) rendering. Fox: alpha flag bit 128 on top of the family's default bits. Pre-Fox: the transparent state set (`zwrite 0`, `alphatest 0`, `alphablend 1`). Omitted = off (no texture inspection — deciding transparency from texture alpha, as the 19to16 converter does, is expensive and surprising). |
| `antiblur` | bool | family default | Fox: generate anti-blur duplicate meshes for this material's meshes (`Has-Antiblur-Meshes`). Pre-Fox: no equivalent (`.model` has no anti-blur). Listed as engine-neutral for discoverability — it describes the material, not a format quirk — even though only Fox implements it. |
| `textures` | table | see "Textures" | Canonical role → stem. |
| `parameters` | table | family default | Shader parameter name → `[x, y, z, w]` (four numbers). Passed to whichever engine defines the name (Fox material parameters, pre-Fox `<vector>` elements); names the target engine's shader doesn't know are dropped with `material_parameter_unused` (I). |

### Shader families

The family is the format's main friendliness device: it captures what the converters' mapping tables
know about which Fox and pre-Fox shaders correspond, so the author picks one word and both engines
get a working material. `shaded` and `shadeless` cover the vast majority of models and are named as
the community knows them (not after either engine's shader — `blin`, `constant`, `Basic_C`, and
`Shadeless` are the native spellings the families map to). Native imports infer the family from the
shader (right-hand column). There is deliberately no family for the rare shaders (the pre-Fox
`Overlay` decal shader, Fox `3ddc`/`eyeocclusion`/`translucent`): in a decade of exports they have not
been used by members, so they stay reachable through the engine tables only (see the advanced
example), and a native import of one keeps its shader verbatim there with the family that best
approximates it for the other engine.

| Family | Use | Fox shader / technique | Fox defaults | Pre-Fox shader | Pre-Fox defaults | Inferred from |
|---|---|---|---|---|---|---|
| `shaded` | Skin, hair, cloth, boots — the standard lit material (Fox "blin") | `fox3ddf_blin` / `fox3DDF_Blin` | alpha `128`, shadow `0`, `antiblur = false`; `MatParamIndex_0 = [0,0,0,0]`; `normal`/`specular` fall back to the game's `dummy_nrm`/`dummy_srm` | `Basic_C` + the letters of the roles that resolve (see the `Basic_*` ladder below) | opaque state set | Fox shaders containing `blin`/`3ddf`/`hair`; pre-Fox `Basic_*`, `Boots`, `Shirt_*`, `Pants_*` |
| `shadeless` | Unlit (eyes, decals baked into the texture, emissive parts) (Fox "constant") | `fox3dfw_constant_srgb_ndr_solid` / `fox3DFW_ConstantSRGB_NDR_Solid` | alpha `16`, shadow `5`, `antiblur = true`; no parameters | `Shadeless` — the community-made shader combining the advantages of the stock `Constant` and `Overlay`, which is why both of those infer this family | opaque state set | Fox shaders containing `constant`/`lambert` (or any other `3dfw` shader, with a warning); pre-Fox `Shadeless`, `Constant`, `Overlay` |
| `metal` | Metallic surfaces | `fox3ddf_ggx` / `fox3DDF_GGX` | as `shaded`; `metalness` role available | `Basic_CNSR` | opaque state set; `Reflection = [1,1,1,0]`, `Shininess = [0.9,0,0,1]`; `environment` falls back to the compiler's **template environment map** (a real cubemap shipped with the templates, taken from a working export's `env.dds`), so `Basic_CNSR` always has its `R` — a metal without a reflection isn't metal | Fox shaders containing `ggx` |
| `glass` | Visors, lenses | `pes3dfw_glass2` / `pes3DFW_Glass2` | alpha `16`, shadow `5`, `antiblur = false`; `MatParamIndex_0 = [54,0,0,0]`, `ReflectionIntensity = [1,0,0,0]`, `GlassRoughness = [0,0,0,0]`, `GlassFlatness = [0,0,0,0]`, `PCBoxCenter = [0,15,0,0]`, `PCBoxSize = [250,80,250,0]` | `Basic_C` | transparent state set | Fox shaders containing `glass` |

**The pre-Fox `Basic_*` ladder.** The `Basic_` shader's suffix letters spell the maps it samples:
`C`olor, `N`ormal, `S`pecular, `R`eflection (an `EnvironmentMap` cubemap plus the `Reflection` and
`Shininess` vectors — see the AUO lab export's `fbm*.mtl`, where every `Basic_CNSR` material with
an `env.dds` also carries those two vectors). For `shaded`, the compiler picks the shortest rung
covering the roles that resolve for the material: `Basic_C` → `Basic_CN` (normal) → `Basic_CNS`
(normal + specular) → `Basic_CNSR` (+ `environment`). Higher rungs tolerate missing lower maps (the
same export uses `Basic_CNSR` with only diffuse + specular), so an explicit `[prefox] shader` is
never wrong, only the automatic pick is minimal. Rungs other than these four (`Basic_CS`, …) are not
assumed to exist; verify against the game's shader list before adding any.

State sets (pre-Fox `[prefox.states]` defaults):

| Set | `ztest` | `zwrite` | `twosided` | `alphatest` | `alpharef` | `alphablend` | `blendmode` |
|---|---|---|---|---|---|---|---|
| opaque | 1 | 1 | 0 | 1 | 0 | 0 | 0 |
| transparent | 1 | 0 | 0 | 0 | 0 | 1 | 0 |

Fox `fuzzblock` shaders (`fox3ddf_blin_fuzzblock`, `…_uvscroll`) are anti-blur helper materials the
exporter generates; they never appear in a toml — importing an FMDL turns them back into
`antiblur = true` on the source material.

### Textures

`[mat.textures]` maps **canonical roles** to texture stems. The role vocabulary is engine-neutral;
a fixed table maps each role to the engine's sampler name, and roles an engine has no sampler for
are dropped for that engine (`material_texture_unused`, I):

| Role | Fox sampler | Pre-Fox sampler | Auto-name suffix | Family default when omitted and not found |
|---|---|---|---|---|
| `base` | `Base_Tex_SRGB` (`Base_Tex_LIN` when `[fox] base_linear = true`) | `DiffuseMap` (`srgb="1"`) | `_bsm`, falling back to the bare material name | required: `material_texture_not_found` |
| `normal` | `NormalMap_Tex_NRM` | `NormalMap` (`srgb="0"`) | `_nrm` | Fox: game `dummy_nrm`; pre-Fox: no sampler |
| `specular` | `SpecularMap_Tex_LIN` | `SpecularMap` (`srgb="1"`) | `_srm` | Fox: game `dummy_srm`; pre-Fox: no sampler |
| `metalness` | `MetalnessMap_Tex_LIN` | — | `_mtl` | none |
| `environment` | — | `EnvironmentMap` (`srgb="0"`, `minfilter="anisotropic"`, wrap addressing, `maxaniso="2"`) — the `R` in `Basic_CNSR` | `_env` | `metal` family: the template environment map, emitted into the player's common textures like any other fallback texture; else none |
| `reflection` | `GlassReflection_Tex_SRGB` | — | `_cbm` | none |
| `reflection_mask` | `GlassReflectionMask_Tex_LIN` | — | `_rfm` | none |
| `detail_material` | — | `DetailMaterialMap` | `_mtm` | none |
| `detail_normal` | — | `DetailNormalMap` | `_dnrm` | none |

Each role's value is one of:

| Value | Meaning |
|---|---|
| *(key absent)* | **Auto-detect**: use the folder texture named `{material}{suffix}` if it exists; otherwise the family default for that engine (a game dummy texture or no sampler at all). A `[face]` entry with no `textures` table and `face.png` + `face_nrm.png` beside it therefore gets both maps on both engines with nothing written. |
| `""` | **Auto-name, required**: the folder texture `{material}{suffix}` must exist (`material_texture_not_found` otherwise). Use it to be told when a texture is missing instead of silently getting the dummy. |
| `"stem"` | **Explicit**: that folder texture (filename without extension). |
| `"none"` | **Disabled**: no sampler for this role even if an auto-named texture exists (game dummy on Fox where the shader needs one). |

The suffix table is fixed because Fox and pre-Fox use different sampler names for the same role, so
the suffix can't be parsed from the sampler name consistently. `base` tries two candidates in order:
`{material}_bsm` (the convention the `pes-fmdl` presets and Red's templates follow) and then the
bare material name (`face` → `face_bsm`, else `face`); both present is not a conflict, `_bsm` wins.
Auto-fill
and auto-detection happen during deep material parsing (not the structure pass), so a missing texture
reports `material_texture_not_found` at that stage, not during shallow validation. Game-provided
textures (the Fox `dummy_nrm`/`dummy_srm` in `/Assets/pes16/model/character/common/sourceimages/`)
are only ever reached through family defaults — a toml never names a game path, so exports stay
self-contained and portable across PES versions. The one set of **reserved stems** a toml may name
without a file behind them is the legacy `dummy_kit*` family, substituted by the modded exes (see
"Kit-dependent assets"); the existence check skips them.

### `[mat.fox]` — Fox (FMDL) table

| Key | Type | Meaning |
|---|---|---|
| `shader` | string | FMDL shader name (`fox3ddf_blin`, …). Default: family's. Setting it without `technique` keeps the family's technique. |
| `technique` | string | FMDL technique name (`fox3DDF_Blin`, …). Default: family's. |
| `alpha_flags` | integer 0–255 | Raw per-mesh alpha flags. Default: family's. Known bits: 32 two-sided, 128 transparent — owned by `two_sided`/`transparent` when those are set explicitly; the other bits are applied verbatim. |
| `shadow_flags` | integer 0–255 | Raw per-mesh shadow flags. Default: family's. Known bits: 1 no shadow cast, 2 invisible — owned by `cast_shadow`/`invisible` when those are set explicitly; the other bits are applied verbatim. |
| `cast_shadow` | bool | Shadow flag bit 1 *clear*. Default `true`. Fox-only and practically never changed, hence here rather than engine-neutral. |
| `invisible` | bool | Shadow flag bit 2: the mesh renders nothing but still casts a shadow if enabled. Default `false`. Same remark. |
| `base_linear` | bool | Bind the `base` role as `Base_Tex_LIN` instead of `Base_Tex_SRGB`. Default `false`. |
| `textures` | table | Native sampler name → stem, for samplers outside the canonical role table (custom shaders). Same value grammar as canonical roles except no auto-name suffix (absent = no sampler). A native name that *is* a canonical role's Fox sampler is `material_value_invalid` — use the role. |
| `parameters` | table | Fox-only parameters, name → `[x, y, z, w]`; merged over the engine-neutral `parameters`. |

FMDL stores alpha/shadow flags **per mesh**, not per material. Export writes the material's
resolved flags to every mesh using it. Import (`fmdl_to_ir`, and the Blender codec) splits a
material instance whose meshes carry differing flags into one material per distinct
`(alpha_flags, shadow_flags, antiblur)` combination, suffixed `_2`, `_3`, … — the same rule the
`pes-fmdl` addon applies when creating Blender materials.

### `[mat.prefox]` — pre-Fox (.model/.mtl) table

| Key | Type | Meaning |
|---|---|---|
| `shader` | string | `.mtl` shader name (`Basic_C`, `Basic_CN`, `Basic_CNSR`, `Shadeless`, `Constant`, `Overlay`, `Boots`, `Shirt_NB`, `Pants_NB`, …). Default: family's (texture-dependent for `shaded`). |
| `states` | table | `<state>` name → integer: `ztest`, `zwrite`, `twosided`, `alphatest`, `alpharef`, `alphablend`, `blendmode`. Default: family's state set, with `twosided`/`transparent` applied. Validated exactly like a `.mtl`: `mtl_state_invalid` (`ztest` ≠ 1, `blendmode`/`alphablend` ∉ {0,1}), `mtl_blendmode_nonzero`, `mtl_state_nonrecommended`. Unknown state names → `material_key_unknown`. |
| `samplers` | table of tables | Per-sampler `<sampler>` attributes, keyed by **native sampler name**: `srgb` (0/1), `minfilter`/`magfilter`/`mipfilter` (`linear`, `point`), `uaddr`/`vaddr`/`waddr` (`wrap`, `clamp`, `repeat`), `maxaniso` (integer). Defaults: `srgb` per the role table, `minfilter = magfilter = "linear"`, nothing else emitted. A `path` key here is `material_value_invalid` — paths come from `textures`. Extra native samplers (no canonical role) are declared here with a `stem` key. |
| `parameters` | table | Pre-Fox-only `<vector>` parameters, name → `[x, y, z, w]`; merged over the engine-neutral `parameters`. |

Output `.mtl` files written at compile time keep the `.dds` extension on texture paths, as the game
expects, and order states `ztest, zwrite, twosided, alphatest, alpharef, alphablend, blendmode`
(the 19to16 converter's order) for byte-reproducible output.

### Examples

Minimal — a face with `face.png` and `face_nrm.png` beside it:

```toml
[face]
```

Typical — as `ir_to_gltf` writes it for a plain FMDL face, comments included:

```toml
[face]
family = "shaded"            # shaded (lit, standard) | shadeless (unlit) | metal | glass
two_sided = false            # render both sides of each face
transparent = false          # alpha-blended (semi-transparent) rendering

[face.textures]              # role = "stem" (filename without extension). "" = must exist,
base = "face"                # omitted = use "<material><suffix>" if present, "none" = off
normal = "face_nrm"          # suffix _nrm; falls back to the game's flat normal map on Fox
specular = "face_srm"        # suffix _srm; falls back to the game's flat specular map on Fox

[face.fox]                   # written from the source FMDL so the same-format round-trip is exact
shader = "fox3ddf_blin"
technique = "fox3DDF_Blin"
alpha_flags = 128
shadow_flags = 0
```

Advanced — a two-sided hair card with tweaked pre-Fox states and a Fox-only parameter, and an eye
occlusion decal using a shader no family covers (the AUO lab export's `eyeocclusion_lambert`):

```toml
[hair]
family = "shaded"
two_sided = true
transparent = true
antiblur = true

[hair.textures]
base = "hair"
normal = "none"

[hair.prefox.states]
alpharef = 128
[hair.prefox.samplers.DiffuseMap]
uaddr = "clamp"
vaddr = "clamp"

[hair.fox.parameters]
MatParamIndex_0 = [3, 0, 0, 0]

[eye_occlusion]
family = "shadeless"         # what Fox gets; pre-Fox is overridden below
transparent = true

[eye_occlusion.prefox]
shader = "Overlay"
[eye_occlusion.prefox.states]
alphatest = 1
[eye_occlusion.prefox.parameters]
DepthBias = [0.005, 0, 0, 0]
```

### Validation summary

| ID | Sev | Condition |
|---|---|---|
| `materials_toml_invalid` | E | a material file fails to parse as TOML |
| `material_file_missing` | E | a glTF matches no material file |
| `material_undefined` | E | a glTF material name has no entry in any matched file |
| `material_key_unknown` | W | unknown key in an entry (ignored) |
| `material_value_invalid` | E | wrong type / out of range / unknown family or role / reserved native name |
| `material_texture_not_found` | E | a mesh-used material's `base`, `""`-marked, or explicit stem resolves to no image |
| `material_texture_unused` | I | a role the target engine has no sampler for |
| `material_parameter_unused` | I | a parameter the target engine's shader doesn't define |
| `material_entry_unused` | I | an entry matching no material in the folder's glTFs |
| `materials_file_ambiguous` | I | several name-matched files apply to one glTF (layering reported) |
| `material_link_layered` | I | a `.common` material link and a local file share a stem (Common below local) |
| `common_link_missing` | E | a `.common` link (model, material file or texture) names a file missing from Common |
| `gltf_embedded_image_ignored` | I | the glTF embeds texture bytes; only the folder's image files are texture sources |
| `gltf_bone_unknown` | E | a skin joint has no `PES_bone` metadata and no name in the template skeleton |
| `kit_variant_missing` | W | a `kitN` reference lacks the variant for a kit number the export defines (lowest variant copied) |
| `kit_variant_model_fox` | W | per-kit model variants on a Fox target (lowest variant used) |
| `mtl_state_invalid` / `mtl_blendmode_nonzero` / `mtl_state_nonrecommended` | E / W / I | `[prefox.states]` checks, shared with `.mtl` validation |

Consequences follow the Team compiler's disposition rules (E → folder discarded). The full
catalog is in the [Team compiler plan](team_compiler.md).

---

## One schema, one canonical converter

The `PES_bone`/`PES_mesh` extension schema and the `materials.toml` schema are the single contracts:
the `pes-models` Blender extension's PES glTF codec writes them (and reads them back into Blender
custom properties — see "Blender integration" in the [Model conversion plan](model_conversion.md)),
`gltf_to_ir` validates them, and any glTF + tomls the extension exports must import cleanly through
`gltf_to_ir`. Blender round-trips are for *authoring*; migrating a player folder between formats is
the Studio's IR conversion (deterministic, tested, superset-merging — see the [Player aesthetics
editor plan](player_aesthetics_editor.md)), so the two paths cannot drift apart on what a converted
folder looks like.
