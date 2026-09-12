# 4cc Studio — Aesthetics export plan

Covers the `aesthetics_export` lib crate and the **Studio aesthetics export format** it describes: the object
model and its validation progression, the player-folder format, `settings.toml` in exports, and the
FPC marker files. The format is shared knowledge — the [Team compiler](team_compiler.md) compiles
it, the [Export upgrader](export_upgrader.md) writes it, the [Kit config editor](kit_config_editor.md)
edits kit folders inside it, the [Refs arranger](refs_arranger.md) edits referee slot allocations
inside it, the [Player aesthetics editor](player_aesthetics_editor.md) browses and converts player
folders in it, and the Team creator generates it — so one plan and one crate own it.
Compile-time behavior (what the compiler *does* with an export) stays in the Team compiler plan;
model-level format details (glTF + PES extensions, `materials.toml`) are in the [Unified model format
plan](model_format.md). Platform context is in the [core plan](core.md).

---

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
*not* split into a companion lib crate — see "Lib crates" in the [core plan](core.md).

`TeamName`, `TeamId` and the `teams_list.txt` parse/reconcile logic live in the leaf crate
`libs/teams_list` (see the [library crates plan](libs.md)); this crate depends on it for identity
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
plan](export_upgrader.md)). Consequences:

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
merge (see the [Unified model format plan](model_format.md)'s "Material files"). For both, a
`.common` link file (`body.mtl.common`, `body.materials.toml.common`) stands in for the Common
file of that name, and a `.common`-linked *model* resolves its material files in Common first with
the player folder's matches layered on top (format plan, "Link files"). This represents
mixed local/shared/Common assemblies without forcing a model folder into one singular source format.
`BootsModelFolder` and `GlovesModelFolder` use the same `parts`/ancillary-`files` structure and
processing contract as `FaceModelFolder`.

### Player folders

The new per-player export format (see "Player folders (the export format)" below) is represented as
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

## Player folders (the Studio export format)

**This is a primary motivation for the project.** The export format itself is new: it is called the
**Studio export format** and was designed for 4cc Studio — no existing tool produces or consumes it
yet. A "player folder" holds all of a player's models —
face, boots, and gloves — in a single folder, without the old split into separate Faces/Boots/Gloves
item folders. (Red's referee export layout was the prototype of this idea, tested only on referees;
the unified format keeps its per-referee face/boots/gloves/common subfolders as optional **reserved
subfolders** — see below — and is used by team and referee exports alike — see "Referee export
processing" in the [Team compiler plan](team_compiler.md).)

```
Players/
├── 15 - Snuffy/
│   ├── face_high.fmdl        (or .model, or .glb — see the Unified model format plan)
│   ├── hair_high.fmdl
│   ├── oral.fmdl
│   ├── boots.fmdl            (player-exclusive boots)
│   ├── Crocs.boots           (link file: references Boots/Crocs/, shared with other players)
│   ├── glove_l.fmdl
│   ├── glove_r.fmdl
│   ├── *.dds / *.ftex / *.png / *.jpg / *.bmp / *.webp / *.tga / *.tiff
│   │                         (textures: any image format, referenced by stem; interchangeable)
│   ├── materials.toml        (glTF models only: PES material properties keyed by material name;
│   │                          the catch-all applies to all glTFs in the folder. Optional
│   │                          name-matched *.materials.toml files (e.g. body.materials.toml)
│   │                          override per-model, mirroring the .model/.mtl system; a
│   │                          *.materials.toml.common or *.mtl.common link pulls the file of
│   │                          that name from Common/ — see the Unified model format plan)
│   ├── ingame_face           (optional empty marker: remove custom face parts, keep other categories)
│   ├── fpc.on                (optional empty marker: apply the FPC preset at compile time; see "FPC toggle")
│   ├── settings.toml         (savefile settings; see below)
│   ├── face/  boots/  gloves/  common/
│   │                         (optional reserved subfolders: their files are that category's parts
│   │                          wholesale, common/ holds textures; see "Reserved subfolders")
│   └── ...
└── 16 - Another Player/
    └── ...
```

**Player numbering** — a normal team player folder gets its number(s) in one of two ways:

1. **In the folder name**: `NN - Name` with NN in 01–23, as above.
2. **Via a root `players.txt`**: lines of `NN <folder name>`; the listed folders then carry no
   number in their names. Sparse lists are allowed, and the same folder may be listed under multiple
   team slots. When `players.txt` is present it is authoritative: every player folder must be listed
   in it, and folder-name numbers are not used.

**Savefile identity.** Normal 4cc team rosters and player IDs are aligned: slot `NN` belongs to
player ID `team_id * 100 + NN`. Resolve that record by ID, not by name or physical file offset;
model/portrait destinations and savefile edits use the same identity. For team 701, slot 03 targets
70103 even if its savefile name is generic. A folder mapped to several slots applies to each slot's
corresponding ID. Names may be written through the opt-in `name` setting, but never select the
record; a missing record is not replaced by a name-based match.

**Multi-mapped processing.** For team and referee identities alike, shared preparation (parse,
convert, and merge) runs once per distinct source player folder; the results are then rendered and
packed per mapped slot (IDs, paths, portraits, savefile entries), while the folder's relocated
textures are emitted once into their name-keyed common subfolder shared by all mapped slots (see
"Texture relocation to common" in the [Team compiler plan](team_compiler.md)). Both stages commit as one atomic unit, so a partial set of slot
instances is never written. Referees use this same mechanism with their `refereeXXX`/`k99XX`/`g99XX`
rendering.

**Referee exception:** a refs export requires root `players.txt`; numbered referee folder names are
not a supported substitute. Its valid slots are 01–35, and repeating a folder under several slots is
how the export controls referee rarity. The same filename serves both identities, and the export
name's first word being `refs` is the referee discriminant. For legacy Red-era referee exports
only, a root **`refs.txt`** is accepted as a read-only alias of `players.txt` (identical grammar);
when both are present `players.txt` is authoritative and the alias is ignored with `refs_txt_ignored`.
Studio never writes the alias: the Refs arranger and the Export upgrader emit `players.txt` and
remove a `refs.txt` they read from. A refs export may also carry a root **`ref_lists.txt`** — the
referee lists the Fox referee hook applies per match (grammar and purpose in the [Refs arranger
plan](refs_arranger.md), "The lists file"). It is the arranger's file: the export model lists it
as a known root file so validation does not warn about it, and the compiler neither reads nor
packs it — it travels to the hook's folder beside the compiled CPK by hand.

**`players.txt` grammar.** Input is strict UTF-8 with an optional UTF-8 BOM; LF and CRLF are
accepted, blank lines are ignored, and comments are not supported. Every nonblank line starts with a
decimal slot followed by whitespace; the trimmed remainder of that line is the complete folder name.
Folder lookup is case-insensitive, but case/Unicode-colliding player folder names are rejected
rather than selected arbitrarily. Writers emit two-digit slots as canonical UTF-8/LF with one
trailing newline. The Refs arranger and Export upgrader read and write this same grammar. A referee
roster must retain at least one valid normalized assignment; an empty file or a file whose entries
all drop reports `players_txt_invalid` and drops the export. An empty authoritative normal-team
roster is allowed for a kit-only export; any present player folders are then unlisted and follow
`player_unlisted`.

**`ingame_face` marker.** Bare `ingame_face` and the tolerated Notepad form `ingame_face.txt` are
recognized before extension allowlist checks, represented as `PlayerFolder.ingame_face`, and never
reported as disallowed files. Parsing normalizes either spelling to the same boolean; generated or
upgraded exports write the bare marker. At categorization it removes face-classified parts and
face-specific ancillary files, and **no face folder is emitted** (on either engine — a face folder
containing only boots/gloves would override the ingame face, and an empty one would blank it).

The face-classified parts split into two groups with different fates:

- **Explicitly-named face models** (`face_high`, `hair_high`, `oral`) have no skeleton slot and
  belong only in a face folder — there is none under `ingame_face`, and `fcl_hair` cannot absorb
  them (it is a single body-skeleton model, not a container for the typed face set). Their presence
  combined with `ingame_face` is a hard error that drops the whole player folder
  (`ingame_face_explicit_face_model`).
- **Arbitrarily-named models** (those that would route to `fcl_hair`, which — like `boots` —
  supports the full body skeleton) **reroute to the boots folder** instead of being dropped:
  renamed to `boots`, or merged into one `boots` if several candidates are present (Fox: `fmdl`
  mesh merging; pre-Fox: `merge_ir_parts`, a lossless same-format IR round-trip), and carrying their paired SKL into
  the `boots.skl` slot (see "SKL pairing"). This generalizes the existing ingame_face boots
  relocation to the body-skeleton face content that would otherwise be lost.

Remaining gloves parts are relocated to **player-specific gloves folders** (`glove_l`, `glove_r`),
and the rerouted arbitrary-named models plus any explicit boots models form the **player-specific
boots folder** — the one case where pre-Fox produces player-exclusive boots/gloves folders with IDs
from the per-team block scheme and savefile writes. Without `ingame_face`, a player with boots/gloves
but no face models keeps a blank face folder (the FPC requirement — see the pipeline walkthrough).

**`ingame_face` with shared links.** A boots/gloves link with no local parts of that category
resolves as usual: on either engine the player simply wears the shared folder's ID, since nothing
needs a face folder. A link **combined with local parts** of the same category is where pre-Fox
must deviate from its no-merge rule: normally the shared folder loads by ID while the local parts
ride in the face XML, but under `ingame_face` there is no face XML, and relocating the local parts
to a player-exclusive folder would leave the player with two candidate IDs (the shared folder's and
the exclusive one) for a single savefile slot. Pre-Fox therefore behaves like Fox's `link_combined`
here: the linked shared model is one more input of the player-exclusive merge (`merge_ir_parts`, a
lossless same-format IR round-trip, like the multiple-boots case above), the exclusive ID wins and is written to
the savefile, and the shared folder is left untouched for players who link it plainly. Gloves follow
the same rule per side (`glove_l` with `glove_l`, `glove_r` with `glove_r`). A **face** link under
`ingame_face` is contradictory (the marker suppresses the face folder the link would fill) and drops
the player folder with `ingame_face_explicit_face_model`, exactly like explicitly-named local face
models.

**Reserved subfolders.** Inside a player folder, the subfolder names `face`, `boots`, `gloves` and
`common` are reserved (case-insensitive). This is the layout of Red's referee exports — the
prototype of the player-folder format — kept in the Studio format as **legacy support**, so that
current-day referee exports (and Red Pre-Studio aesthetics exports using the same subfolders) compile as
they are; it costs little. The flat player folder is the canonical layout — it lists a player's
contents more explicitly — so Studio-generated exports never use the subfolders and the Export
upgrader flattens them (see "Referee export processing" in the [Team compiler plan](team_compiler.md) and the Export upgrader plan). Any other
subfolder is `file_type_disallowed` for its files, as today. Red implements the same reservations
(its `pre_studio_plan.md`, "Reserved subfolders"); the two differ only in conflict handling, noted
below.

- A `face/`, `boots/` or `gloves/` subfolder's files are **that category's parts wholesale**: they
  skip filename categorization (a `hair_high.fmdl` inside `boots/` is a boots part), otherwise they
  are ordinary local parts — they take part in the same allowed-name resolution, SKL pairing, texture
  stem resolution (stems resolve across the player folder including its reserved subfolders) and
  merging as loose root files of the same category. Their textures follow the normal texture
  relocation to the per-player common output.
- `common/` holds textures only: they are texture sources like any other image in the player folder,
  and are emitted into the per-player common output as they are. Model files in `common/` are
  `file_type_disallowed`.
- **Combination follows the plan-wide rule, not Red's.** Red treats a labelled subfolder combined
  with a same-category link file, or with loose same-category root files, as a conflict that drops
  the player folder (it cannot merge). Studio can, so a subfolder's parts **combine** with a
  same-category link (`link_combined`, the shared folder as the base) and with loose root files of
  the same category, exactly as two loose files would — there is no subfolder-versus-file conflict.
  The one contradiction that remains is `ingame_face` plus a `face/` subfolder: the subfolder's
  files are face parts and follow the marker's rules (explicitly-named face models drop the folder
  with `ingame_face_explicit_face_model`; arbitrary-named models reroute to boots).
- An **empty `face/`** counts as face content: the player gets a blank face folder, as a player with
  no face models does (the FPC requirement — see the pipeline walkthrough); under `ingame_face` an
  empty `face/` is simply ignored (nothing to contradict). Empty `boots/`, `gloves/` and `common/`
  are ignored.
- Root normalization never treats a reserved subfolder as a nested export root, and `Players/`
  detection is unaffected: the reserved names are matched one level below a player folder only.

**Shared models** live in Faces/Boots/Gloves folders identified by **name only** (no embedded IDs —
player folders are the main reference point for each player). A player folder references a shared
folder with an empty link file named after it (`Crocs.boots`, `Longhair.face`, `Keeper
gloves.gloves`). A stray `.txt` suffix (`Crocs.boots.txt`) is accepted silently — Windows hides
known extensions by default, so users creating link files with Notepad often produce one. At most
one shared link per category (face, boots, and gloves) may appear in a player folder; multiple links
of any same kind are rejected. Multiple shared-face bases are ambiguous pre-Fox because copied trees
can collide, and unnecessary on Fox because one shared base can already merge with all local parts.
Fox merging therefore combines that one shared base with local parts, never several same-kind shared
links:

```
Boots/
└── Crocs/                    (shared boots, referenced by two+ players via link files)
    ├── boots.fmdl
    └── *.dds
```

**A link plus local models combines** — for every category, so there is no link-versus-model
conflict anywhere. The shared folder acts as a **reusable base** (a body, a head, a boot shape) and
the player's local models are parts layered over it. How that is realized differs by engine, because
pre-Fox is far more permissive:

- **Pre-Fox**: everything in a player folder that isn't a boots or gloves link file becomes part of
  that player's **face folder**. The `face.xml` has a per-entry model *type* property, so boots and
  gloves models load into their own skeletons from the face folder — meaning per-player boots/gloves
  folders are never needed, and nothing is ever merged. A shared face folder is copied per player
  and receives the local files on top; a boots/gloves link keeps loading its shared folder by ID
  while the player's local parts load from the face folder alongside it.
- **Fox**: there is no model type property, so per-player boots and gloves folders *are* required.
  Local boots/gloves models form a **new player-exclusive folder** with its own ID from the team's
  block, and a shared model referenced by a link becomes just another component of it, merged in
  (see below). The shared folder is left untouched for the players that link it plainly. A shared
  **face** link with no local face parts is the same mechanism one part deep: the shared face model
  becomes the player's face FMDL outright (a merge of one), since shared face folders take no ID and
  have no independent output to be left untouched.

Combining is reported as `link_combined`. Consequences for Fox ID assignment: a player who combines
gets their deterministic player-exclusive ID rather than the shared folder's, and a shared
boots/gloves folder that every referencing player combines is only a source of parts — it needs no
folder of its own in the output and consumes no ID from the scarce shared pool (it is still
"referenced", so not `shared_folder_orphaned`). Pre-Fox only ever assigns IDs to shared boots/gloves
folders, since player-exclusive ones don't exist there — except under `ingame_face`, where the
relocated gloves/boots parts form player-exclusive folders with their own IDs (see "ingame_face
marker").

**Common model links and model merging** — a model can be loaded from the export's `Common` folder
instead of shipping in the player folder: an empty link file named after the Common model plus a
`.common` extension (a stray `.txt` is accepted, as with shared-folder links) — Red's pre-Fox
`.model.common` link convention, extended to the Studio format's model file names. On pre-Fox targets
the link resolves to a real runtime reference: the generated XML points at the Common path (on PES16
via the patched exe — see the note under the XML/MTL checks). Fox engines cannot load models from
Common at all, so there the compiler **bakes the link away: the Common model's meshes are merged
into the player's output FMDL** at compile time via the `fmdl` crate.

The same link convention applies to **material definition files**: `body.mtl.common` and
`body.materials.toml.common` pull `Common/body.mtl` / `Common/body.materials.toml` into the player
folder's material resolution as if they were local files of that stem (Red's `find_mtl_file`
already resolves `.mtl.common` names). A Common-linked model needs no such link — its material files
resolve in Common first, with the player folder's matching files layered on top as overrides. And
to **textures**: `hair.png.common` stands in for `Common/hair.png` under the stem `hair`, for
auto-detection and explicit stems in material files and for native `.mtl`/FMDL references alike —
the way to share one texture among many players. A texture that resolves into Common is packed once
in the team's Common output and referenced there; one that resolves in the player folder is the
player's own and travels with it. Rules, same-stem layering, texture-stem resolution and the
packing rule are in the [Unified model format plan](model_format.md)'s "Link files"; the pre-Fox
XML points at the Common MTL path when the MTL was found in or linked from Common, as Red does.

Merging is driven by name resolution. Red's prefix convention (a prefixed allowed name is renamed to
it: `kit_boots.fmdl` → `boots.fmdl`) extends from pure renaming to multi-part assembly: **all models
— local files, shared-folder parts, `.common` links, or a mix — that resolve to the same allowed
output name are merged into that one model**, in alphabetical source-name order (deterministic
recompiles). `torso_fcl_hair.fmdl` + `legs_fcl_hair.fmdl.common` → one `fcl_hair.fmdl`; a `.common`
link with no local counterpart bakes the shared model in unchanged — the Fox substitute for pre-Fox
common-model sharing. Parts may be in any supported source format (converted to FMDL first, then
merged via the `fmdl` crate's mesh merging).

**Merging is Fox-only.** Fox model sets are closed — a face is a fixed list of FMDL names, and boots
and gloves are separate ID'd folders with one model each (`boots`, `glove_l`, `glove_r`) — so every
extra part has to be merged into one of those. Pre-Fox needs none of it: the typed `face.xml`
absorbs any number of entries of any type, and `.common` links stay runtime references. **Exception
— `ingame_face`**: with no face folder emitted, non-face parts are relocated to player-specific
folders; multiple boots models are merged into one (Fox: `fmdl` mesh merging; pre-Fox:
`model_convert::merge_ir_parts`, the IR-level counterpart with the same rules — see "IR part merge"
in the Model conversion plan — as a lossless same-format `.model` round-trip, the one pre-Fox merge
case), and gloves go to `glove_l`/`glove_r` folders (see "ingame_face marker").

**Model names: a free part plus a suffix.** A model file name is `<anything>_<suffix>` (or just
`<suffix>`), and the **suffix is always at the end** — one rule for every source format and both
engines. The suffix says what the model is; the compiler derives the Fox destination and the
pre-Fox `face.xml` type from it through one table:

| Suffix | Fox destination | Pre-Fox `face.xml` type |
|---|---|---|
| `face_high`, `hair_high`, `oral`, `fcl_hair` | that FMDL (merge if several) | `face_neck` for `face_high` (Red's rule); `parts` otherwise |
| `boots` | the `boots` folder (`boots.fmdl`, merge if several) | `parts` (boots use the body skeleton; today's typing) |
| `glove_l` / `gloveL` | the gloves folder, `glove_l.fmdl` | `gloveL` |
| `glove_r` / `gloveR` | the gloves folder, `glove_r.fmdl` | `gloveR` |
| `handL` / `handR` | the gloves folder (`glove_l`/`glove_r` — hand-skeleton models have nowhere else to go on Fox) | `handL` / `handR` |
| `uniform`, `shirt`, `pants_nocloth`, `eye`, `mouth`, `face_neck`, `parts` | face: the `fcl_hair.fmdl` merge | as named (`uniform` → `uniform_sub` on PES15, Red's rule) |
| `model_type_<x>` | face: the `fcl_hair.fmdl` merge | `<x>` verbatim — the escape hatch for a type this table does not know |
| *(none)* | face: the `fcl_hair.fmdl` merge (`fmdl_fcl_hair_fallback` reports each routed file) | `parts` (Red's default) |

The first column's Fox allowed names and the pre-Fox type names are both native vocabularies, so
both are accepted; `glove_l`/`gloveL` and `glove_r`/`gloveR` are aliases of each other. Matching is
case-insensitive and ignores underscores inside the suffix, as Red's typing did. A `_ratio_<n>` token
anywhere in the free part still sets the `face.xml` entry's `ratio` attribute (Red's convention).
Pre-Fox entries additionally get the `oral_`/`_win32` affixes on the emitted file name (a PES 16
loading requirement). The type column applies **only when targeting pre-Fox and only when the
compiler generates the `face.xml`**: a folder that ships its own xml carries the types there, and its
file names are not interpreted for typing. A user-written `face.xml` is accepted as a second-class
path — the way to experiment with what the pre-Fox engines can do beyond this table — and is checked
by the compiler with errors for what is known to break and warnings for everything outside the
vocabulary it generates itself ("User-supplied `face.xml`" in the [Team compiler
plan](team_compiler.md)). Fox has nothing resembling designable model types, so on
Fox a type maps to a destination and nothing more. glTF
models follow the same table — the type lives in the file name, not in the glTF or the material
toml, so a `.glb` authored once compiles typed on pre-Fox and merged on Fox.

**This inverts Red's typing, which matched types as prefixes** (`gloveL_foo.model`); the Studio
format matches them as suffixes (`foo_gloveL.model`), the same position the Fox allowed names already
occupied (`kit_boots.fmdl`), so there is one place to look. The Export upgrader swaps them (see its
plan). "Arbitrary model names are face content" is the last row: the generalization of Red's
fcl_hair fallback (which renamed a single arbitrary-named face FMDL to `fcl_hair.fmdl`) — a model
with no recognized suffix is a face part on both engines, which is what makes the plain-named case
work with no naming ceremony: `torso.fmdl` + `legs.fmdl.common` → one `fcl_hair.fmdl` on Fox, two
`parts` entries pre-Fox. Boots and gloves models must therefore *say so* by suffix — in a player
folder because an unsuffixed model would be taken for face content, and in a shared boots/gloves
folder, where nothing can be a face, unsuffixed names are `fmdl_name_invalid` errors.

**SKL pairing** — a model may carry a custom skeleton. Each source format keeps it differently: an
`.fmdl` in a companion `.skl` file named after the model's source basename (`commander.skl` for
`commander.fmdl`, `kit_boots.skl` for `kit_boots.fmdl`); a glTF in its native `skin` (see
"Skeleton: native glTF skin" in the [Unified model format plan](model_format.md)); a `.model`
**inline**, in its per-bone matrix table — pre-Fox has no sidecar to pair (see "Skeleton in
`.model`" in the [Model conversion plan](model_conversion.md)). The `.skl` pairing is recognized in
**Common folders, shared (Faces/Boots/Gloves) folders, and player folders** alike. When a model is
pulled in — via a `.common` link, a shared link, or used locally — its skeleton travels with it into
the destination output: on Fox targets as the destination's `.skl` (pass-through bytes for `.fmdl`
inputs; generated from the IR for glTF and `.model` inputs that carry bones outside the target's
skeleton tables), on pre-Fox targets inside the written `.model`'s bone table, for every source
format. This covers the rare case where a model has a custom pose that depends on a custom skeleton;
the common case needs no SKL and the compiler's template skeletons suffice. Independently of custom
skeletons, every model is **retargeted to the target version's body skeleton** at compile time — the
games' `body.skl` files differ per version in bone set and rest pose (PES15 markedly), and the
compiler ships all of them; see "Skeleton retargeting and bone conformance" in the [Model conversion
plan](model_conversion.md). This absorbs the standalone `4cc-model-simplifier-15` step.

Only two output destinations have SKL slots: **`fcl_hair`** (the `fcl_hair_sim.skl` slot) and
**`boots`** (the `boots.skl` slot) — both are full-body-skeleton models. `face_high`, `hair_high`,
and `oral` have no skeleton slot. A custom SKL arriving at a destination replaces the template
injection for that slot: if any part merged into the destination brings a custom SKL, the template
`boots.skl`/`fcl_hair_sim.skl` is not injected and the custom one is renamed to the slot's canonical
name. If no part brings a custom SKL, today's template injection stands. A custom SKL paired with a
model that resolves to `face_high`/`hair_high`/`oral` has no slot to land in and is reported as a
warning (`skl_no_slot`) — a no-op file the user likely authored by mistake.

**What the injected files are.** Both slot templates are the **body skeleton** under the slot's
name — Red ships PES21's `body.skl` and writes it as `boots.skl` / `fcl_hair_sim.skl`. The name
`boots.skl` is misleading: the game's own `boots.skl` (`resources/skeletons/pes*/boots.skl`) has
four bones (`sk_foot_*`, `dsk_toe_*`) and is never what a 4cc export ships. The community
convention this encodes is that the boots folder is the place for a **full-body model** whenever the
`fcl_hair` slot is not used — a body model saved as `boots.fmdl` needs the whole body skeleton, so
members used to add a renamed `body.skl` themselves; the compiler's template made the manual copy
unnecessary. Real boots models use a subset of the same bones, so the body skeleton serves them too.
PES21's file is the template because it has the largest bone set of the Fox versions, so any bone a
model may reference has a bind pose. **Once the retargeting pass folds bones the target lacks, that
superset rationale disappears** and injecting the *target version's* `body.skl` becomes the
consistent choice (its rest pose is what the game's animations drive). Whether the game behaves
differently with a same-version skeleton than with PES21's is untested in-game; the plan keeps PES21
as the injected file until a compile-and-play comparison on PES18/19 settles it (open point).

**Merge constraint** — parts merged into one output FMDL must reference the same skeleton. A part
with a custom SKL and a part using the default template skeleton reference different skeletons, as
do two parts with different custom SKLs. "Same" is decided by **content hash** for two `.skl`
files (identical bytes under different filenames are one skeleton) and by **bone-transform
comparison with tolerance** whenever a part's skeleton comes from the IR (glTF skins, `.model` bone
tables) — the same comparison the pre-Fox `merge_ir_parts` uses. A skeleton mismatch between merge
parts is a hard error (`skl_merge_conflict`) that drops the folder.

**Kits** live in a `Kits/` folder with one subfolder per kit (this per-kit granularity also drives
the GUI's per-kit grid cells). A kit folder is named by its **slot**, optionally followed by a
label in the same `<slot> - <label>` shape as player folders: `p1`, `p1 - Lakers`, `g1 - Goalie`.
The slot part (`p1`–`p9`, `g1`, or `all`, case-insensitive) is all the compiler reads; the label is
for the human — the GUI kit cell's tooltip and the Kit config editor's tab show it, nothing is
derived from it. Two folders resolving to the same slot (`p1/` and `p1 - Lakers/`) are an error
(`kit_slot_duplicate`), not a pick. The special folder **`all/`** holds textures shared by every
kit: for each kit, the effective texture set is the kit folder's own files plus every `all/` file
whose stem the kit does not have itself — own files win, per stem. Most teams use the same
`_back`/`_leg`/`_name` number and name textures on all their kits, and copying them into nine
folders is what made old exports heavy and what let one kit silently fall behind when the set was
updated. The rule is per stem, so a kit can inherit `kit_back` and still override `kit_name`; it
covers the main `kit.dds` too, for the rare team whose kits differ only by config. The five
texture-name fields of each kit's config are derived from the *effective* set — an inherited
`kit_back.dds` makes the kit's `back` field non-empty exactly as an own copy would. `all/` is not
a kit: it has no cell, no `config.toml`, `colors.txt` or `icon.txt` (such files there are
reported and ignored), and an `all/` beside no kit folder is reported as unused. A shared base
config was considered and rejected: a kit folder without `config.toml` means "the template", so
the folder describes its own look and can be copied between exports unchanged. With an inherited
config, the same folder would compile differently in each export, and a collar or shorts-number
change is invisible until the game shows it. An inherited *texture* changes a copied kit too, but
visibly, as itself — and often intentionally, since the kit takes on the number font of the team
it joins.

A kit folder may be **completely empty** — a bare `p1/` with nothing in it, and no `all/` to
inherit from. It still declares that the slot exists, and the compiler fills it in: a bundled
placeholder main texture (the magenta/black "missing texture" checkerboard), the template
`config.toml`, and a `UniColor` entry as for any kit (colors from the kit's `colors.txt`, else the
loud magenta/black "no colors chosen" pair — see "Kit colors fallback" in the [Team compiler
plan](team_compiler.md)). Teams whose whole roster is
full-body models never render a kit texture (except a pre-Fox player given the `uniform` model
type), yet the game expects every kit slot the team uses to have a texture and a config; the empty
folder says "pretend a kit exists here" without making the author draw one. The same fill applies
to any kit whose effective texture set lacks `kit.dds`, so a folder holding only a `kit_back.dds`
is a placeholder kit with a custom number font, not an error. Kit textures follow the export-wide
texture rule: matched by **stem**, in **any accepted image format** — `kit.png` is as good as
`kit.dds` (the compiler converts; the diagrams say `.dds` only by habit). Online kit designers
hand out PNGs, and a new manager should not need a DDS tool to use one.

**Kit layout marker.** The kit UV layout changed between the two engines: the shirt, sleeves and
collar strip are identical in PES 15–17 and 18–21, but the **sock islands are 68 px narrower** in
Fox (u 8–372 instead of 8–440 on the left, mirrored on the right; height unchanged) and the
**shorts islands keep their outline but are partitioned differently** (pre-Fox: a 444-px shorts
body at the outer edge plus a 212-px inner-thigh strip; Fox: a 152-px hem strip at the outer edge
plus a 456-px body). A kit drawn for one engine and compiled for the other therefore shows its
sock and shorts designs displaced. An optional empty **marker file** in the kit folder, named
`pre-fox` or `fox` (case-insensitive; the Notepad forms `pre-fox.txt` / `fox.txt` are tolerated,
as for `ingame_face`), declares which layout the kit's `kit.dds` and its `kit_mask.dds` /
`kit_srm.dds` are drawn for. When the marker names the other engine than the compile target, the
compiler re-lays those textures out (see "Kits" under "Processing" in the [Team compiler
plan](team_compiler.md)); when it names the target's engine, or is absent, nothing is edited —
**no marker means "drawn for whatever you compile for"**, which is today's behavior. Both markers
in one folder is an error (`kit_layout_conflict`, kit discarded), like `fpc_conflict`. The marker
belongs in kit folders only: in `all/` it is `kit_all_file_ignored` like any non-texture, and a
kit that inherits `all/kit.dds` states the layout of that inherited texture with its own marker.
It is a marker file, not a `config.toml` key, because `config.toml` holds only data that reaches
the game's kit config; the layout describes the texture and stays beside it. The Export upgrader
never writes one: kits have passed through several converters unchanged, so their true origin is
unknowable from the files, and a wrong guess would silently displace a correct kit. The only
authored kit config is the TOML file `config.toml` — the
binary game format is never part of an export and exists only as compile output; old exports' binary
configs are converted by the [Export upgrader](export_upgrader.md) (schema and binary layout in the
[Kit config editor plan](kit_config_editor.md)); each kit folder also has a `colors.txt` with the
kit's two menu colors, one per line, in the same color-entry format the old Team Note txt used, and
optionally an `icon.txt` holding the kit's menu icon number (0–23, the old Note txt kit entries'
trailing number — selects the two-color kit icon pattern shown next to the formations in the
prematch gameplan screens, which only PES 15/16 display; absent = default 3):

```
Kits/
├── all/                      (optional: textures every kit inherits unless it has its own file of that stem)
│   ├── kit_back.dds
│   ├── kit_leg.dds
│   └── kit_name.dds
├── p1 - Lakers/              (slot, optionally ` - ` and a free label)
│   ├── config.toml           (kit config — TOML; compiled to the game binary at compile time)
│   ├── colors.txt            (the two menu colors, one per line; old Team Note color-entry format)
│   ├── icon.txt              (optional: menu icon number, 0-23; absent = default 3)
│   ├── pre-fox               (optional empty marker, `pre-fox` or `fox`: which engine's kit layout the
│   │                          main texture and mask are drawn for; absent = the compile target's)
│   ├── kit.dds               (main kit texture)
│   ├── kit_mask.dds          (kit textures: generic "kit" prefix — no more u0XXXp1 naming;
│   ├── kit_srm.dds            the prefix is replaced with the kit's ID, e.g. u0701p2,
│   ├── kit_chest.dds          at compile time; `_mask` is the pre-Fox material map and `_srm`
│   └── kit_*.dds              the Fox one — each is emitted only for its engine, never converted)
├── p2/
│   ├── kit_name.dds          (overrides all/kit_name.dds for this kit; kit_back/kit_leg are inherited)
│   └── ...
├── p3/                       (empty: a placeholder kit — checkerboard texture, template config, menu
│                              colors from a colors.txt here or the loud "none" pair; for full-body teams)
└── g1/
    └── ...
```

**Portraits** — a player's portrait normally lives in their player folder (stem `portrait`, any
accepted image format; the compiler converts to the game's DDS). The
standalone `Portraits/` folder (`player_NN.dds` files) is deliberately kept alongside: some managers
make custom portrait sets — while keeping the players' models unchanged — depending on the opponent
team they are about to face, and the standalone folder supports those model-less portrait exports.
Conflicting portraits for the same player in both locations remain an error (`portrait_conflict`).

**Root files** — the old Team Note txt is dismissed entirely. In its place:

```
<export root>/
├── colors.txt                (optional: the team's colors, one per line — same format as a kit folder's colors.txt)
├── notes.txt                 (optional: strict UTF-8 free-form notes, collected into teamnotes.txt at compile time)
├── README.txt                (optional: for humans only — known to the model, ignored by the compiler;
│                              the Team creator writes one with next steps)
├── players.txt               (optional for teams, required for refs: slot → player folder mapping)
├── logo.png                  (optional: the team logo, one image in any accepted format, any size;
│                              a `_crop` / `_stretch` / `_fit` tag chooses how a non-square one
│                              becomes square — see "Logo")
├── logo_small.png            (optional: a different image for the game's smallest, 128² logo —
│                              menus use it; same tags; absent = downscaled from logo)
├── Players/ ...
├── Kits/ ...
└── ...
```

- **Team identity**: the canonical `team_name` is always derived from the first word of
  `export_display_name`; the complete stem remains presentation/source identity only. There is no
  `Team:` line anywhere.
- **Team colors**: optional root `colors.txt`, feeding `TeamColor.bin` (absent: the team's existing
  bin colors are left untouched).
- **Kit colors**: each kit folder's `colors.txt` (plus optional `icon.txt`), feeding `UniColor.bin`.
- **Notes**: optional strict-UTF-8 root `notes.txt`, replacing the old "Other Notes" section. An
  optional BOM is stripped, newlines normalize to LF, invalid encoding drops the note with
  `notes_encoding_invalid`, and empty/whitespace-only content produces no `teamnotes.txt` entry.
- **Player numbers**: optional root `players.txt` for normal teams as an alternative to numbered
  folder names; required for referee exports, whose authoritative slot mapping and repetition cannot
  be expressed through numbered folder names.
- **Logo**: optional, at most **two root image files**, any accepted raster format (the texture
  allowlist), any size. Stem grammar: `logo` `[_small]` `[_<fit>]`, tags in that order, matched
  case-insensitively. `logo*` (no `_small`) is the **main** logo: the compiler always produces the
  game's 512² and 256² PNGs from it, and the 128² one too unless `logo_small*` exists.
  `logo_small*` is an optional **different image for the 128² logo only** — the game's menus show
  that one, and teams sometimes want a simplified or joke version there; it never influences the
  two larger sizes. `<fit>` is one of `crop` (center-crop the long side), `stretch` (resample to
  square, distorting), `fit` (letterbox with a transparent border), each file carrying its own; a
  square source needs no tag; a non-square source without one is treated as `fit`, the only mode
  that loses nothing and distorts nothing — the tag exists to *opt into* loss. A `logo_small*`
  without a main `logo*` is an error: the game needs all three sizes and the small one is not a
  source for the large ones. There is no `Logo/` folder: the old format asked authors for a
  hand-made triplet with a placeholder team ID baked into three filenames, and one in eleven
  shipped exports got the sizes wrong; files the compiler resizes cannot. More than one file per
  role (two `logo*`, or two `logo_small*`) is an error, not a pick.

The Export upgrader generates all of these from the old Team Note txt and `Logo/` folder when
migrating.

At compile time, the pipeline:

0. **Hand auto-split** — in-process in Rust via `model_convert`, select each hand's vertices with
   positive `skf_*_l`/`skf_*_r` weights, grow the selection once along the mesh topology, and separate
   it into `glove_l`/`glove_r`, leaving the body. This is equivalent to Blender's select → `Ctrl +`
   → `P` workflow, not an invocation of Blender (see "Hand auto-split" in the
   [Model conversion plan](model_conversion.md)). Models without such weights pass through unchanged.
   The split parts appear as virtual model files before categorization.
1. **Categorizes** each model file as face/boots/gloves — by filename convention or by
   face.xml-style metadata (the same categorization problem the 16→21 converter already solves via
   `parseFaceXml` and filename matching). Anything the conventions don't claim is **face content by
   default**, so there is no "uncategorized model" failure. Pre-Fox the category only picks the
   model's *type* attribute in the generated `face.xml` — everything ships in the one face folder;
   Fox splits the categories into separate folders and merges each category's parts (see "Common
   model links and model merging")
2. **Assigns IDs automatically** (Fox needs both pools; pre-Fox only the shared one, having no
   player-exclusive boots/gloves folders — except under `ingame_face`, see "ingame_face marker") —
   from a **hardcoded per-team block of 40 IDs** in the 4-digit boots/gloves ID space: the first 100
   IDs are reserved for the stock PES boots, then each team gets a fixed block starting from team ID
   701 (`block_start = 101 + (team_id - 701) × 40`; team 701 gets 101–140). The block splits into
   the player-exclusive part (deterministic: `block_start + player_number - 1` for slots 01–23,
   extending the 16→21 converter's `bootsId = base_id + player_number - 1` scheme) and the **17
   shared-folder IDs** (the remainder, assigned in alphabetical folder-name order — deterministic,
   so recompiling an unchanged export yields the same IDs). Link files resolve to the assigned IDs.
   Boots and gloves are **disjoint game namespaces** (separate `boots/{id}/` and `glove/{id}/`
   folders, `k`/`g` prefixes), so the identical block layout applies independently in each — no
   boots-versus-gloves split of the block is needed. Sizing rationale: at 220 teams (IDs 701–920),
   the 40-ID blocks end at 8900, leaving 8901–9899 unallocated and keeping clear of the `99XX` band
   that referee `k99XX`/`g99XX` IDs occupy (the hard ceiling would be 44 per team, ending at 9780;
   45 would cross into the referee band and overflow into 5 digits). The block size is fixed for the
   lifetime of a scheme version — IDs are baked into distributed savefiles — so builds record
   `allocation_scheme_version = 1` in their metadata. The ABI is future-upgradeable: a later exe
   patch might allow 5-digit IDs, letting each player's boots/gloves ID equal their player ID; any
   such change is a new scheme version, and since a scheme change is always shipped with a
   from-scratch savefile remake, bumping the version is a sanctioned path rather than a
   compatibility break
3. **Splits and packs** everything into the correct game structures (face folder with player ID,
   boots/gloves folders with assigned IDs), rewriting FMDL/MTL/XML texture paths to match
4. **Plans conditional savefile mutations** for assigned IDs and independent accepted settings.
   Producer commits activate mutations that reference compiled assets; after all outcomes are known,
   `pes_savefile` serializes only activated asset mutations plus eligible independent settings. This
   makes the player wear successfully emitted boots/gloves without pointing at failed content — the
   coordination currently done manually with 4ccEditor.

**The old export format is not supported by the compiler.** Old exports are migrated once with the
Export upgrader tool (see the [Export upgrader plan](export_upgrader.md)), which also resolves old
embedded IDs to the new name-based structure.

---

## Player settings in exports (settings.toml)

Savefile settings that currently live only in the savefile move into the export as TOML, written to
the savefile at compile time. **Everything in the file is optional** — including the file itself: a
player folder with only models is valid, and an absent authored key means "leave that savefile
setting untouched"; model-derived ID assignments and FPC directives are separate inputs.

**The file can express every aesthetic setting the savefile holds.** Its schema is `pes_savefile`'s
`PlayerSettings`: all of the player appearance record — physique, strip style, wrist-tape and
spectacle colours, skin and iris colour, the motion block, the ingame-face parameters — except the
two compiler-owned groups (boots/gloves IDs, edit flags). That completeness is a tested invariant
in the [Savefile plan](pes_savefile.md) ("Player settings model"), and it matters here because this
file is the **only** route by which a team's aesthetics reach the official save: the compiler
resolves it into an aesthetics patch, the save editor applies the patch, and the editor's own
appearance fields are read-only by default (see "Read-only aesthetics" in the [Save editor
plan](save_editor.md)). The **template** — what the Export upgrader and the save editor generate,
and what a new player folder starts from — therefore lists every key: set ones with their value,
unset ones as commented lines whose injected comment gives the range, so a team can see what is
authorable without any other document. The example below is abridged.

**Boots/gloves IDs are not authored settings.** Export `settings.toml` neither accepts nor generates
boots/gloves ID keys. Local models and shared link files determine the compiler's player-exclusive
or shared assignments, using the existing per-target rules; users never copy numeric IDs into this
file. Pre-Fox local models embedded in face XML still need no standalone ID, as described above.

```toml
# settings.toml — inside a player folder

# Optional: the savefile player name. Absent = the savefile name is left
# untouched (the save editor keeps owning it). `name = true` derives it
# from the folder: the name part of the folder name (`15 - Snuffy` →
# "Snuffy"), or the whole folder name for players.txt-mapped folders.
# A string sets it explicitly. The number is never given here —
# numbering is owned by the folder name / players.txt mapping.
name = "Snuffy"

# The file never references models: what a player wears is fully determined
# by the folder contents (models present, link files pointing at shared
# folders), so the folder view alone tells the whole story.

[appearance]
skin_color = 2
iris_color = 1

[appearance.physique]
height = 180
neck_length = 2
shoulder_width = -1
# unspecified fields keep their savefile values

[appearance.strip]
sleeves = "short"
socks = "long"
untucked = true

[appearance.motion]
# hunching = 1            # 1–3
# arm_movement = 1        # 1–5
kick_motion = 3
# … every other key of the record, commented, with its range
```

Name-writing rules, in full — **not writing is the default**; writing is opt-in:

1. `name` absent → no name is written, ever. The savefile name stays whatever the save editor set.
   This is also what makes multi-mapped `players.txt` folders work as **generic model folders**:
   several players can share one folder's models without being renamed.
2. `name = true` → the name is derived from the folder (the name part of a numbered folder name, or
   the whole folder name under `players.txt`) and written.
3. `name = "..."` → the string is written as-is. This is also the only way to write a name carrying
   the savefile's **name colour codes** (`\x11c` + 8 hex digits, see the pes_savefile plan's "Name
   colour codes") — control characters cannot appear in a folder name, so a derived name (`true`)
   is always plain. Cup organizers colour names freely (not only medal players'), so tools that
   generate `settings.toml` from a savefile (the Export upgrader, the save editor) must emit the
   explicit string whenever the savefile name carries codes, or the next compile would strip them.
4. With `name = true` or a string in a folder mapped to multiple players via `players.txt`, the same
   name is applied to all of them, with a warning (`settings_toml_name_shared`).

Compile-time flow: resolve boots/gloves IDs from the models and link files present (a broken link is
caught as `link_target_missing` — a check impossible in the current split-tool workflow) → parse
accepted TOML → compile → resolve the settings and the IDs of content that was actually written into
the **aesthetics patch** written beside the CPK → if a local savefile is configured, apply that
patch to it through `pes_savefile` (decrypt, apply, re-encrypt, `.bak`). The patch is what a DLC
builder hands to the savefile builder, who applies it in the save editor; the compiler has no
other savefile write path (see "Post-processing" in the [Team compiler plan](team_compiler.md)).

The save editor view can also **generate** these TOML files from an existing savefile, giving teams
a migration path from the current workflow.

---

## FPC toggle (`fpc.on` / `fpc.off` marker files)

FPC (Full Player Customization — see the [Save editor plan](save_editor.md)) makes the default
player model invisible via a specific mix of savefile settings (blank-model boots/gloves IDs plus
strip settings, varying slightly per PES version), so that an FBM (Full Body Model) replaces the
player entirely. Setting it through individual `settings.toml` keys would be error-prone and
version-dependent; instead it gets a folder-level toggle, consistent with the principle that the
folder view tells the whole story about how the player's models render:

- **`fpc.on`** — an empty marker file in a player folder. When present, the compile-time savefile
  write applies the version-appropriate **FPC enable preset** from `pes_savefile` (the same preset
  the save editor's FPC toggle uses — one implementation, so the two tools can never drift).
- **`fpc.off`** — applies the disable preset (visible defaults) instead, for un-FPC'ing a previously
  FPC'd player at compile time without a trip to the save editor.
- **Absent = no FPC preset applied**: existing FPC settings are not reset merely because the marker
  is missing. Independent authored settings and model-derived ID assignments still apply.
- Tolerances, in the same spirit as link files: a stray `.txt` suffix (`fpc.on.txt`) is accepted
  silently, and a bare `fpc` file is accepted as `fpc.on` (presence reads as "on").
- Both markers in the same folder raise `fpc_conflict` (error; folder discarded).
- Either preset **overrides** any conflicting `[appearance.strip]` keys in the folder's
  `settings.toml`; explicitly-set keys that get overridden raise `fpc_strip_conflict`.
- In a `players.txt` multi-mapped folder, the preset applies to all mapped players (like the rest of
  the folder's settings).

**Boots/gloves ID precedence**, independently per category, for both `fpc.on` and `fpc.off`:

| Standalone asset outcome | Savefile ID |
|---|---|
| Requested local/shared output committed | Compiler-assigned ID wins over the preset's hide/default ID |
| Requested output failed or was dropped | Existing savefile ID is preserved; failure is not treated as absence |
| No standalone output requested | Apply the marker's preset ID, or preserve the existing ID if there is no marker |

Other preset fields still apply normally. Pre-Fox local models embedded in face XML do not request
a standalone boots/gloves output, so they follow the third row.

**Team kit-FPC status and kit configs.** FPC also requires settings on **every one of the team's kit
configs, including the goalkeeper kit** (modern system, per the [wiki's PES17 FPC
guide](https://implyingrigged.info/wiki/Pro_Evolution_Soccer_2017/Full_Player_Customization): shirt
model 176, shorts model 16, collar 105, winter collar 105 — stored as per-version constants
alongside the presets, since the retro pre-2024 system differed). These kit values are a **team-wide
prerequisite that enables per-player FPC**, not a per-player switch: with them in place, each
player's own savefile settings decide whether that player's body is hidden, and non-FPC (head-only)
players render normally on the same team. The markers are therefore strictly **player-level** —
`fpc.off` says "this player needs its body", not "this team's kits must not be FPC" — and mixed
teams (some `fpc.on` folders, some `fpc.off` or unmarked) are ordinary, supported usage;
`fpc_conflict` only rejects both markers inside *one* folder.

An export's **team kit-FPC status** is two-state — `EffectiveTeamKitFpc::{On, Unknown}`:

- **On** when at least one player folder carries `fpc.on` — the configs must then carry the FPC
  values for that player's hiding to work; **Unknown** otherwise — the absence of `fpc.on` markers
  makes no claim about the team (its FPC players may live only in the savefile, set through the save
  editor), and `fpc.off` markers contribute nothing here because they are per-player statements.
- **Generated kit configs** (the `kit_config_generated` path, when a kit folder has no
  `config.toml`) are created with the FPC values when On, and with the plain defaults when Unknown.
- **Supplied kit configs are reconciled only upward** (`kit_config_fpc_adjusted`): On → the FPC
  values are written into every config, which also protects against the classic user error of
  copying a kit config from another team. Unknown → supplied configs are left untouched. The
  compiler **never automatically reverts** FPC values found in supplied configs — no export state
  proves the team stopped using FPC, so de-FPC'ing a team's kits is a deliberate manual config edit
  (a future explicit team-level mechanism could revisit this).
- **Kit slots absent from the export are patched in place, not demanded back.** A midcup (partial)
  export can add an FPC player to a team whose kits need no other changes; requiring the untouched
  kit exports to be resent just so the compiler can see them would be easy to forget. When the
  status is On, the compiler edits the team's existing kit entries itself. Fox: the team's entries
  in the working `UniformParameter` get the FPC values — the working bins come from the installed
  cup CPKs (the bundled bases only serve from-scratch compiles), so a midcup compile patches the
  current cup state, and the `uniparam` container format is fully parsed (see the [library crates
  plan](libs.md)), so locating a team's entries is routine. Pre-Fox: the team's current kit-config
  bins are located in the same installed CPKs, patched, and re-emitted. Patched slots report
  `kit_config_fpc_adjusted` like supplied configs; a slot with no existing entry or config to patch
  reports `kit_config_fpc_unpatched` (warning) — that team genuinely needs a kit export.
- **A custom collar wins over the FPC collar value**: collar rewriting (see "Collars" in the [Team compiler plan](team_compiler.md)) runs after
  FPC reconciliation, so an FPC team with a custom collar keeps its replacement collar ID while the
  other FPC kit values stand.

The kit-config field logic this needs (reading/writing the shirt/shorts/collar model fields, plus
the compile-time binary emission) lives in **`libs/kit_config`**, shared with the [Kit config
editor](kit_config_editor.md) tool; the FPC values themselves, the player presets, and the
interference rules come from the leaf crate **`libs/fpc`** (see [libs](libs.md)), which `kit_config`
and `pes_savefile` both consume — the FPC system has exactly one description, so the compiler, the
kit config editor, and the save editor can never drift.
