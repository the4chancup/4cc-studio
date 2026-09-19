# 4cc Studio — Aesthetics export plan: Object model

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Object Model

The pipeline operates on typed objects, not a flat dict of path strings to bytes. The `VirtualTree`
remains as the serialization layer (used at load and pack time only).

**The model and its validation live in `libs/aesthetics_export`, not in the Team compiler.** The export format is
shared knowledge: the Export upgrader writes it, the Kit config editor edits kit folders inside it,
and the Team creator generates it from scratch — so one crate owns the object model, the
folder conventions described under "Player folders (the export format)", and the check rules behind
the format-level messages in the catalog. That includes the **structure parser** — canonical
folder/file listing, names/sizes, supplied small metadata such as raw `players.txt`, and link files
→ `ParsedAestheticsExport`, with no model or texture contents. The consumer creates the canonical listing
from a directory walk, ZIP central directory, or 7z entry table and chooses a source provider;
`aesthetics_export` does not own filesystem/archive I/O. `VirtualTree`, lazy per-folder *content* loading,
source-provider implementation, and the memory budget remain pipeline concerns. The Load step's
eager structure loading builds on this parser, and the Refs arranger supplies a plain-filesystem
listing. The Team compiler is the crate's biggest consumer, and the compile-time behavior
(processing steps, packing, bins, GUI, CLI) stays here. The Team compiler itself is deliberately
*not* split into a companion lib crate — see "Lib crates" in the [core plan](../core/README.md).

`TeamName`, `TeamId` and the `teams_list.txt` parse/reconcile logic live in the leaf crate
`libs/teams_list` (see the [library crates plan](../libs/README.md)); this crate depends on it for identity
resolution and contributes only the export-format half — splitting the display name into the
token that `TeamName::new` folds. `studio_core` never parses the list or interprets team IDs.

The eager object model stores **structure descriptors, not loaded contents**: a processing task
later resolves each descriptor through the pipeline's source access (direct file, ZIP entry, or
solid-7z buffer) and materializes the folder under its memory permit; loaded bytes stay owned by the
loading task until its written output drains, keeping the memory charge aligned with byte lifetime.
`ParsedAestheticsExport`, `ValidatedAestheticsExport`, `ResolvedAestheticsExport`, `PlayerFolder`, and the descriptor
types belong to `aesthetics_export`; loaded processing types such as `FaceModelFolder` and their
conversion/packing methods belong to `team_compiler`, avoiding inherent implementations of
shared-lib types in the tool crate.

**Validation boundary.** Structural validation (folder tree, naming, roster, link-file references,
file-type allowlists) lives in `aesthetics_export` and returns a `ValidationReport` containing the parsed
draft, every issue, and an optional **sanitized** `ValidatedAestheticsExport`: a bad optional file or
folder is omitted while valid siblings remain, with every original issue preserved for reporting.
Foundational failures — unsafe virtual paths, roster-wide ambiguity, unparsable required metadata —
never produce a valid affected scope and cannot be restored by `pass_through`; isolated malformed
roster lines remain `RosterEntry`/`DropSlot` findings. Invalid drafts remain renderable and
repairable in the GUI; unreadable sources and unsafe virtual paths are hard parser errors. Deep
format validation (FMDL/XML/MTL/glTF/texture/kit-config content checks) lives in the respective
format crates and applies the same sanitized-scope rule before planning. The consuming tool maps
`ValidationIssue`s to `Message`s with its own catalog policy — so the Refs arranger and Kit config
editor can consume the same structural checks without inheriting Team-compiler-specific catalog
entries.

The compile progression is **`ParsedAestheticsExport` → `ValidatedAestheticsExport` (structural + deep
validation, sanitized) → `ResolvedAestheticsExport` (identity attached) → build manifest (run-level
planning)**. Parsing retains raw roster entries and issues so malformed input stays reportable;
planning resolves dependencies, IDs, destinations, and collisions serially before parallel work
starts.

### `aesthetics_export` crate layout

Five consumers (Team compiler, Export upgrader, Kit config editor, Refs arranger, future Team
Creator) read this crate, so its layout is organized by the **progression stage** a consumer can
stop at, and each stage's types are only constructible through the previous one:

```
crates/libs/aesthetics_export/src/
├── lib.rs              # re-exports; the progression's entry points
├── listing.rs          # CanonicalListing: what the consumer supplies (paths, sizes, small metadata)
├── conventions/        # the export format as data, no logic
│   ├── mod.rs          #   folder names, reserved team names (/refs/, /balls/), NO_USE markers
│   ├── player_folder.rs#   allowed model names per slot, link-file extensions, settings.toml name
│   ├── kits.rs         #   kit folder grammar (`<slot>[ - <label>]`, slots p1–p9, g1, all), `all/` inheritance, config.toml / colors.txt / icon.txt
│   └── file_types.rs   #   allowlists per folder kind (strict vs info)
├── parse/              # listing → ParsedAestheticsExport (raw, everything retained)
│   ├── mod.rs
│   ├── identity.rs     #   export_display_name → first token → teams_list::TeamName::new
│   ├── roster.rs       #   players.txt → raw RosterEntry list (malformed lines retained)
│   ├── links.rs        #   link files (Name.boots, .common) → unresolved references
│   └── descriptors.rs  #   PlayerFolderDesc, SharedModelFolderDesc, KitsFolderDesc (structure only)
├── validate/           # ParsedAestheticsExport → ValidationReport { draft, issues, sanitized: Option<ValidatedAestheticsExport> }
│   ├── mod.rs          #   the sanitized-scope rule; foundational vs isolated failures
│   ├── structure.rs    #   folder tree, nesting, naming
│   ├── roster.rs       #   ValidatedRoster: numbers, duplicates, DropSlot semantics
│   ├── links.rs        #   link resolution against shared folders and Common
│   └── issues.rs       #   ValidationIssue (stable code, scope, context) — no message text
├── resolve.rs          # ValidatedAestheticsExport + teams_list::TeamsList → ResolvedAestheticsExport (ExportIdentity, TeamId)
├── players_txt.rs      # players.txt writer (Refs arranger) with atomic replacement
└── kit_config_toml.rs  # config.toml ↔ kit_config binary bridge for exports (comment-preserving)
```

Placement rules:

- **No I/O.** Consumers supply a `CanonicalListing` (from a directory walk, ZIP central directory,
  or 7z entry table) and, where content is needed for deep checks, the bytes; this crate never opens
  a file or archive. `players_txt.rs` returns bytes and an atomic-replace *plan*; the caller writes.
- **No message text.** `validate/issues.rs` yields stable codes with scope and context; each tool
  maps them to its own catalog. This is what lets the Refs arranger and Kit config editor share the
  checks without inheriting Team compiler wording.
- **Structure, not contents.** Descriptor types carry names and sizes; loaded model/texture types
  (`FaceModelFolder`, …) and everything that converts or packs them live in `team_compiler`.
- **Conventions are data.** A new allowed model name or kit file is a table entry in
  `conventions/`, and every consumer picks it up; no consumer keeps its own list. Two rules that
  read like compiler behavior are conventions and live here, because a second consumer already
  depends on them: the **model source selection rule** (target-native first, then glTF, then the
  convertible opposite native format — the Player aesthetics editor uses it to decide which set
  opens visible and why leftover natives must be removed after a glTF conversion), and the
  **structural file-kind classification** of a model folder's files (model, texture, `GltfBuffer`,
  `PotentialGltfImage`, link file, marker, disallowed — from names alone; `model_convert`'s glTF
  dependency contract takes this classification as its input and only *promotes* candidates during
  deep URI resolution).
- **Deep format validation is not here.** FMDL/XML/MTL/glTF/texture/kit-config content checks live
  in their format crates; `validate/` only orchestrates the structural layer and the sanitized-scope
  rule that deep checks then reuse.
- `wasm32`-checkable and GUI-free (it is the Studio Web cheap tier's export knowledge).

### Core types

```rust
pub struct ParsedAestheticsExport {
    pub draft: AestheticsExportDraft,
    pub raw_roster: Vec<RawRosterEntry>,
    pub issues: Vec<ValidationIssue>,
}

pub struct ValidationReport {
    pub parsed: ParsedAestheticsExport,
    pub validated: Option<ValidatedAestheticsExport>,
    pub issues: Vec<ValidationIssue>,
}

pub struct ValidatedAestheticsExport {
    pub export_display_name: String,     // complete archive/folder stem; presentation/source identity
    pub team_name: TeamName,               // canonical /xx/ name derived from the first word
    pub players: Vec<PlayerFolder>,      // sanitized main reference point for each eligible player
    pub roster: ValidatedRoster,         // normalized strong slots → player indices
    pub faces: Vec<SharedModelFolder>,   // shared folders, referenced by name from player folders
    pub boots: Vec<SharedModelFolder>,   //   (no IDs — IDs are assigned by run planning)
    pub gloves: Vec<SharedModelFolder>,
    pub kits: KitsFolder,                // per-kit subfolders (config.toml + colors.txt + textures)
    pub portraits: PortraitFolder,
    pub logo: Option<LogoFiles>,         // root `logo*` (+ optional `logo_small*`), each with its fit mode (see "Root files")
    pub collars: CollarFolder,
    pub common: CommonFolder,
    pub root: RootArtifacts,             // sanitized root colors/notes/referee marker
}

pub struct RootArtifacts {
    pub team_colors: Option<FileDescriptor>,
    pub notes: Option<FileDescriptor>,
    pub referee_marker: Option<FileDescriptor>,
}
// Invalid optional root artifacts are absent here but remain in ValidationReport.

pub struct SharedModelFolder {
    pub folder_name: String,
    pub files: Vec<FileDescriptor>,
}

pub struct KitsFolder {
    pub kits: BTreeMap<KitSlot, KitFolder>,  // one per kit folder; `all/` is not a kit
    pub shared: Vec<FileDescriptor>,         // `all/` textures as found (each also appears, as Shared, in every kit lacking that stem)
}

pub struct KitFolder {
    pub folder_name: String,                 // `p1` or `p1 - Lakers`
    pub label: Option<String>,               // the free part after ` - `; GUI/editor display only
    pub config: Option<FileDescriptor>,      // config.toml (absent → generated at compile time)
    pub colors: Option<FileDescriptor>,      // colors.txt
    pub icon: Option<FileDescriptor>,        // icon.txt
    pub layout: Option<KitLayout>,           // `fox` / `pre-fox` marker file; None = drawn for the target engine
    pub textures: Vec<KitTexture>,           // the *effective* set: own files, plus `all/` files for stems the kit lacks
}

pub enum KitLayout { PreFox, Fox }          // which engine's kit UV layout `kit` and its mask/srm are drawn for

pub struct KitTexture {
    pub stem: String,                        // `kit`, `kit_back`, … (lowercased)
    pub file: FileDescriptor,
    pub source: KitTextureSource,            // Own | Shared — provenance for the editor and `kit_textures_inherited`
}
// Inheritance is resolved here, once, so the compiler, the Kit config editor and the upgrader
// see the same effective set and derive the same texture-name fields from it.

pub struct PlayerIndex(pub usize);

pub enum ValidatedRoster {
    Team(BTreeMap<PlayerSlot, PlayerIndex>),
    Referees(BTreeMap<RefSlot, PlayerIndex>),
}
// Parsing retains raw roster entries and issues so duplicate slots remain
// reportable. A referee roster may map several RefSlots to the same PlayerIndex.

pub enum ExportIdentity {
    Team { id: TeamId, name: TeamName },   // TeamId is strictly 701–920
    Referees,                            // rendered as fixed game ID 999 only at format boundaries
}

pub struct ResolvedAestheticsExport {
    pub export: ValidatedAestheticsExport,
    pub identity: ExportIdentity,
}
```

The compiler supports **only the Studio export format**. The old format (separate Faces/Boots/Gloves
item folders with embedded IDs, Kit Configs/Kit Textures folders, the Other folder) is not supported
— old exports are migrated with the Export upgrader tool (see the [Export upgrader
plan](../export_upgrader.md)). Consequences:

- No `KitConfigFolder`/`KitTextureFolder`: kits live in the `Kits/` folder, one subfolder per kit.
- No `Other` folder support.
- `Faces`/`Boots`/`Gloves` folders still exist in the Studio format for models **shared between
  players**, but they are identified by **name only** (no embedded IDs). Player folders are the main
  reference point: a player folder references a shared folder with a link file (e.g. `Crocs.boots`).
  At compile time, each shared **boots/gloves** folder gets an auto-assigned ID, and every linked
  player gets that ID written to the savefile (only if the shared folder's output actually commits).
  Shared *face* folders take no ID: pre-Fox copies the shared face folder per linked player (local
  files layered on top), and Fox merges the parts into each player's own FMDL — see "A link plus
  local models combines" below.

```rust
// libs/aesthetics_export: parsing preserves invalid raw input for diagnostics and is
// source-neutral: the consumer supplies a canonical listing (from a directory walk,
// ZIP directory, or 7z entry table) plus small metadata; aesthetics_export owns no I/O.
pub fn parse_listing(
    listing: CanonicalListing,
    metadata: SmallMetadata,
) -> Result<ParsedAestheticsExport, SourceError>;
impl ParsedAestheticsExport {
    pub fn validate(self, context: &ValidationContext)
        -> Result<ValidationReport, FatalValidationError>;
}
impl ValidatedAestheticsExport {
    pub fn resolve_identity(self, teams_list: &TeamsList)
        -> Result<ResolvedAestheticsExport, IdentityError>;
}

// tools/team_compiler: serial run-level planning over all resolved exports
// produces the immutable manifest; scoped failures drop tasks/exports while
// a manifest for valid siblings still proceeds (partial planning). The
// planning result also carries all planning messages and the dropped exports.
pub fn plan_run(
    exports: Vec<ResolvedAestheticsExport>,
    ctx: &CompileContext,
) -> PlanReport;
pub fn process_task(task: BuildTask, ctx: &CompileContext) -> TaskBatch;
// A task batch carries the packed entries, messages, and the task's memory permit.
// Model tasks carry immutable IDs assigned by plan_run; processing only consumes
// those assignments, never allocates. The writer commits successful batches and
// releases their permits.
```

### Model files

Files within model folders are typed. Files that the processing code needs to look inside get rich
structs; files that are just passed through stay opaque.

```rust
pub enum ModelFile {
    // Ancillary files only: every FMDL/.model/glTF source and sibling MTL is owned
    // exactly once by a ModelPart below.
    Texture(TextureFile),    // Has format, mipmaps, conversion methods
    Xml(XmlFile),            // Parsed DOM, has editing methods
    Skl(SklFile),            // Paired with a same-basename .fmdl; content-hashed for merge checks
    Fclo(FcloFile),          // Opaque or parsed depending on needs
    RawBin(RawFile),         // Opaque game/template bin
    Other(RawFile),          // Opaque
}

pub struct RawFile {
    pub relative_path: vtree::RelativeScopePath, // canonical within the folder
    data: Arc<[u8]>,
}
```

### Model folders

```rust
pub struct FaceModelFolder {
    pub model_id: String,
    pub folder_name: String,
    pub files: Vec<ModelFile>, // ancillary textures/XML/SKL/FCLO/raw bins only
    pub parts: Vec<ModelPart>, // owns every selected model source exactly once
}

pub struct ModelPart {
    pub logical_output: String,             // resolved allowed output name / XML entry
    pub source: ModelSource,
    pub material_source: Option<MaterialSource>, // optional folder/glTF material overrides
    pub provenance: ModelProvenance,        // local, shared-folder, or Common link
}

pub enum ModelSource {
    Fox(FmdlFile),
    PreFox { model: PreFoxModel, mtl: MtlFile }, // one atomic native bundle
    Gltf(GltfModel),
    Intermediate(CanonicalModel),  // after conversion, before re-serialization
}

impl FaceModelFolder {
    pub fn fmdls(&self) -> impl Iterator<Item = &FmdlFile>;
    pub fn textures(&self) -> impl Iterator<Item = &TextureFile>;
    pub fn validate(&self, context: &ValidationContext) -> Vec<ValidationIssue>;
    pub fn convert_textures(&mut self, settings: &Settings) -> Result<()>;
    pub fn convert_to(&mut self, target: ModelFormat) -> Result<()>;
    pub fn pack(&self, settings: &Settings) -> Vec<PackedEntry>;
}
```

`convert_to` converts every selected part to the target-native format before parts are grouped by
`logical_output` for Fox merging or pre-Fox XML generation. A pre-Fox part always owns its `.model`
and sibling `.mtl` as one native bundle; conversion consumes or produces both atomically and commits
the replacement only after both succeed. Material-definition files pair with their models here for
both formats by the same stem name-matching, each with its own resolution rule: pre-Fox `.mtl` via
Red's cascade (exactly one per model), glTF `*.materials.toml` via the layered least→most-specific
merge (see the [Unified model format plan](../model_format.md)'s "Material files"). For both, a
`.common` link file (`body.mtl.common`, `body.materials.toml.common`) stands in for the Common
file of that name, and a `.common`-linked *model* resolves its material files in Common first with
the player folder's matches layered on top (format plan, "Link files"). This represents
mixed local/shared/Common assemblies without forcing a model folder into one singular source format.
`BootsModelFolder` and `GlovesModelFolder` use the same `parts`/ancillary-`files` structure and
processing contract as `FaceModelFolder`.

### Player folders

The new per-player export format (see `player_folders.md` "Player folders (the Studio export format)") is represented as
its own type, which the coordinator splits into standard model folders:

```rust
pub struct PlayerFolder {
    // No number field: numbering is a roster property, not folder content —
    // slots map to folders in ValidatedAestheticsExport.roster (a referee folder is identical
    // regardless of how many slots point at it). Folder→slots, when needed
    // (process once, pack per slot), is derived by grouping the roster.
    pub player_name: String,
    pub files: Vec<FileDescriptor>, // names, sizes, kinds — contents load later, per task
    pub links: Vec<SharedLink>,     // shared-folder link files (e.g. "Crocs.boots" → shared Boots/Crocs/);
                                    //   `.common` links (models, material files) are `files` entries
                                    //   of link kind, resolved against the Common folder by the pipeline
    pub ingame_face: bool,          // recognized ingame_face / ingame_face.txt marker
    pub fpc: Option<FpcDirective>,  // normalized fpc.on/fpc.off directive
    pub portrait: Option<FileDescriptor>,
    pub settings: Option<FileDescriptor>,  // settings.toml
}
// Every source file has exactly one semantic role. `files` holds model-pipeline
// content — everything categorization routes and the model tasks consume
// (models, textures, material definition files, skeletons), however it reaches
// the output: emitted transformed, converted, or embedded by conversion. Typed
// fields hold savefile-stage inputs (settings, portrait, FPC markers) and
// folder-level references (links). Typed metadata and glTF dependencies are not
// also retained as pass-through output files. Files inside a reserved
// subfolder (face/, boots/, gloves/, common/ — see "Reserved subfolders") are
// ordinary `files` entries: their ScopePath keeps the subfolder, and
// categorization reads the forced category from it instead of the file name.

pub struct SharedLink {
    pub kind: SharedKind,           // Face / Boots / Gloves
    pub name: String,               // "Crocs"
}

pub struct FileDescriptor {
    pub path: vtree::ScopePath,     // canonical virtual path within the export
    pub size: u64,
    pub kind: FileKind,             // classified from extension/name (Fmdl, Model, Gltf,
                                    // GltfBuffer, GltfImage, Texture, Xml, Mtl, MaterialsToml,
                                    // Skl, Fclo, RawBin, Other)
    // glTF files may reference external .bin buffers and image files (any
    // accepted format: DDS, FTEX, PNG, JPEG, BMP, WebP, TGA, TIFF): those
    // dependencies are restricted to the source model folder, loaded once each,
    // and never doubled as pass-through output. Whether an image is a glTF
    // dependency is only known after deep glTF parsing; until then it is a
    // candidate, and unreferenced candidates fall back to the ordinary allowlist.
    // Material references (FMDL/MTL/materials.toml) name textures by stem; the
    // compiler resolves stems to actual files. Two image files with the same
    // stem but different extensions is a `texture_stem_conflict` error.
}

// tools/team_compiler free functions operate on a materialized folder rather
// than adding inherent compile methods to aesthetics_export::PlayerFolder.
pub fn categorize_player(folder: &LoadedPlayerFolder) -> CategorizedModels;

/// Split into standard folders using boots/gloves IDs frozen by plan_run
/// (processing never allocates IDs). `slots` comes from ValidatedAestheticsExport.roster
/// (one entry for ordinary team players, possibly several for referee folders).
pub fn split_player(
    folder: LoadedPlayerFolder,
    identity: &ExportIdentity,
    slots: &[ExportSlot],
    planned_ids: &PlannedModelIds,
) -> (Option<FaceModelFolder>, Option<BootsModelFolder>, Option<GlovesModelFolder>);
// Team and referee folders alike may contain only boots, gloves, portraits,
// or settings — any category may be absent.
```

### Design constraints

- **No parent references.** Pass the resolved `ExportIdentity` and other parent context into child
  processing as parameters rather than storing parent references. Copy only presentation strings
  that a child result must own. This avoids Rust's ownership friction with parent-child references.
- **Strong domain types.** Use validated `TeamName`, `TeamId`, `ExportIdentity`, `PlayerSlot`,
  `RefSlot`, `ExportSlot` (distinguishing team vs referee slots), `BootsId`, `GlovesId`, and
  `CpkStem` types at boundaries rather than passing unvalidated strings and integers through
  processing code.
- **Shared textures.** Rayon's model-folder tasks are cross-thread by construction, so textures
  referenced by multiple tasks use `Arc<[u8]>` exclusively; shared bytes stay charged to the memory
  budget until their last consumer is written.
- **Enums over trait objects.** Use `enum ModelFile` with pattern matching, not `Box<dyn
  ModelFile>`. Exhaustive matching catches missing cases at compile time.
- **Pass-through files stay simple.** A file the compiler moves without processing is `RawFile {
  relative_path, data }` — no behavior-bearing wrapper even when the file is semantically rich (the
  Collars folder's contents are model files, but the compiler never parses them). Only files with
  processing logic get rich types.

---
