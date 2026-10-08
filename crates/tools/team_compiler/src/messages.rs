//! The Team compiler's message catalog (`team_compiler/messages.md` "Catalog"): each code's
//! severity, and the mapping of the structure pass's issues and the tool's own findings to
//! `Message`s. Until the help window needs them (Phase 8) a row holds no text: the console prints
//! the code with its context fields.

use std::borrow::Cow;

use aesthetics_export::{IssueScope, ValidationIssue};
use studio_core::{Disposition, ExportId, Message, MessageCode, Scope, Severity};

/// The tool's id: its CLI subcommand, its settings section and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "team-compiler";

/// The codes the tool reports itself, beside the structure pass's issues (whose codes are the
/// lib's strings): discovery, routing, identity, planning, processing and output findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    /// An archive cannot be opened, or its listing is refused.
    ExportExtractFailed,
    /// The scan of the exports folder found no folder, `.zip` or `.7z`: nothing is checked or
    /// compiled.
    NoExportsFound,
    /// A `NO_USE` marker disables the export.
    ExportDisabled,
    /// The export's team, or the referees, as identified.
    ExportIdentified,
    /// A balls export, which the Balls compiler owns.
    ExportBallsSkipped,
    /// More than one non-disabled refs export in the run.
    MultipleRefExports,
    /// Two or more exports resolve to one team: every one of them is skipped, since the
    /// compiler cannot tell which one is meant.
    DuplicateAestheticsExport,
    /// The export's first word names no teams-list row.
    TeamNameUnknown,
    /// A line of a kit's or the root `colors.txt` that does not hold exactly one color, or a
    /// valid color past the file's capacity (two for a kit, four for the team); the line is
    /// skipped.
    ColorEntryInvalid,
    /// A team export without a root `colors.txt`: its team keeps the colors `TeamColor.bin`
    /// had.
    TeamColorsMissing,
    /// A kit without `config.toml` gets the template config.
    KitConfigGenerated,
    /// A kit's `config.toml` that is not UTF-8 text or that `kit_config` refuses (a
    /// value of the wrong type or out of range); the kit is left out.
    KitConfigInvalid,
    /// A value of a kit's `config.toml` that the target version's kit config cannot hold;
    /// it is clamped to one the version holds when the config is emitted.
    KitConfigVersionClamped,
    /// The team's kit-FPC status is On and a kit config lacks the FPC values: a supplied one,
    /// which they are written into as it is emitted, or the installed one of a kit the team's
    /// `UniColor.bin` record holds that a `Midcup` export does not, patched in place.
    KitConfigFpcAdjusted,
    /// The team's kit-FPC status is On and a kit its `UniColor.bin` record holds, which a
    /// `Midcup` export does not, has no installed config or one that does not decode: the slot
    /// is left alone, and the team needs a kit export for it.
    KitConfigFpcUnpatched,
    /// A kit whose effective textures lack `kit.dds` (an empty folder included): the bundled
    /// checkerboard stands in as its main texture.
    KitPlaceholder,
    /// A kit's effective textures hold the other engine's map (a `kit_mask` on a Fox target,
    /// a `kit_srm` on a pre-Fox one), which the target does not read; the file is not
    /// emitted.
    KitTextureNotUsed,
    /// A kit whose `colors.txt` is missing or gives fewer than two valid colors: its menu
    /// colors are taken from its main texture.
    KitColorsDerived,
    /// A kit with neither two valid colors in its `colors.txt` nor a main texture of its own to
    /// take them from (a placeholder kit): magenta and black are written as its menu colors.
    KitColorsMissing,
    /// A kit whose layout marker names the other engine than the target's: the sock islands
    /// of its main texture, and of the mask or srm the target reads, are re-laid out to the
    /// target's layout.
    KitLayoutConverted,
    /// A team kit whose collar or winter collar, after FPC reconciliation, is the referees'
    /// reserved collar 77: its players would wear the referee marker, so the kit is left out.
    KitCollarReserved,
    /// A `Collars/` file whose name is not `collar_<ID>`, or whose ID is no stock collar of the
    /// target version that a kit config can name; the file is left out.
    CollarIdInvalid,
    /// A `Collars/` file replacing a collar someone else holds: the suite's own, 105 (FPC) or
    /// 77 (the referees' marker); the file is left out.
    CollarIdConflict,
    /// A non-square logo source was made square; names the file and the mode applied (`fit`
    /// for an untagged file, or its tag).
    LogoFitApplied,
    /// A logo source whose side the mode maps onto the target is under its largest target
    /// (512 pixels for the main file, 128 for the small one): it is emitted upscaled.
    LogoUpscaled,
    /// Phase 3 only: the export holds content `compile` cannot build yet; it is skipped.
    ContentNotYetCompiled,
    /// More shared boots folders take an id than the team's block has; the export is skipped.
    BootsIdPoolExhausted,
    /// More shared gloves folders take an id than the team's block has; the export is skipped.
    GlovesIdPoolExhausted,
    /// A slot with a portrait in its player folder and one in `Portraits/` whose bytes differ:
    /// the export is skipped, since the compiler cannot tell which one the manager means.
    PortraitConflict,
    /// A player holds a boots or gloves link beside a model of the same package: the shared
    /// folder's models become parts of the player's own package.
    LinkCombined,
    /// A player's `settings.toml` that is not UTF-8 text or does not parse as the settings
    /// schema; the file is ignored and the folder's models still compile.
    SettingsTomlInvalid,
    /// A Fox model whose name has no recognized suffix: it is face content, a part of the
    /// `fcl_hair.fmdl` merge.
    FmdlFclHairFallback,
    /// Several models of a package resolve to one allowed name and were merged into one FMDL.
    FmdlMerged,
    /// Pre-Fox: several boots models were merged into one `boots.model` and its `.mtl`.
    ModelMerged,
    /// Pre-Fox: merged parts' `.model` headers carry different `flags`, a field of unknown
    /// meaning, so the merge has no rule to combine them; the package is left out.
    ModelMergeFlagsConflict,
    /// A face model's vertices carry hand-skeleton (`skh_`) weights: its hands were cut off at
    /// the wrist into the player's gloves (the hand auto-split).
    ModelHandSplit,
    /// Two merged parts define a material of one name differently; the package is left out.
    MergeMaterialConflict,
    /// Merged parts disagree on their skeleton (a bone, or their paired `.skl` files); the
    /// package is left out.
    SklMergeConflict,
    /// A `.skl` paired with a `face_high`, `hair_high` or `oral` model, which have no skeleton
    /// slot on Fox; the file is ignored. Also the skeleton a `.model`'s conversion writes for a
    /// model of such a role, or of a glove, which is left out the same way.
    SklNoSlot,
    /// A `face_diff.bin`, `face_diff.xml` or `fcl_hair_sim.fclo` in a player folder with no
    /// face model (with or without `ingame_face`): there is no face for it to shape, so the
    /// file is not read.
    FaceFileNotUsed,
    /// A model folder's texture variant set (`pants_kit1`, `pants_kit3`) has no variant for
    /// a kit number the export defines: the lowest variant is copied into the gap.
    KitVariantMissing,
    /// Per-kit model files (`pants_kit1.fmdl`, `pants_kit2.fmdl`) on a Fox target, which
    /// cannot switch models with the kit: the lowest variant is used, the others ignored.
    KitVariantModelFox,
    /// PES 2015 to 2017: a per-kit model variant whose `.mtl` (the one its search finds, as a
    /// `face.xml` entry would name it, directory included) is not the one its set's entry
    /// names respelled for its kit number; the model is packed, and the game finds no `.mtl`
    /// for that kit.
    KitVariantMtlDiffers,
    /// A pre-Fox face holding a model none of whose `face.xml` entries is a `face_neck`: the
    /// bundled dummy model is listed last as the face's `face_neck` and packed with its `.mtl`.
    XmlFaceNeckAdded,
    /// PES 2015: a generated `face.xml` entry of type `uniform`, written `uniform_sub` instead
    /// (`face_xml::version_type`).
    XmlUniformPes15,
    /// A face's `face_diff.xml` that is not base64 text or a `<dif>` holding it, or a face
    /// diff (decoded, or a `face_diff.bin`) without the magic `FACE` or shorter than its
    /// header gives; the folder holding it is left out.
    FaceDiffInvalid,
    /// A folder gives its face diff twice: it holds both `face_diff.bin` and `face_diff.xml`,
    /// or its own `face.xml` holds a `<dif>` beside either; the folder is left out.
    XmlDifConflict,
    /// PES 2015 to 2017: a player folder's own `face.xml` that is not UTF-8 text or not
    /// well-formed XML (the error names the line and column); the folder is left out.
    XmlBroken,
    /// PES 2015 to 2017: a texture a `.mtl` names for a material one of its models' meshes
    /// uses, which nobody supplies: not in the folder, and for a path into the team's Common
    /// folder neither in the export's `Common/` nor in an installed CPK loaded before the
    /// run's. A Warning that keeps the folder: the game plays faces with such a miss.
    MtlTextureNotFound,
    /// PES 2015 to 2017: a texture nobody supplies that only `.mtl` materials no mesh uses
    /// name; the game never loads it, so nothing is left out.
    MtlTextureUnusedMissing,
    /// A member's own `face.xml` whose root element is not `<config>`; the folder is left out.
    XmlRootTagInvalid,
    /// A `<model>` of a member's own `face.xml` without a `type`; the folder is left out.
    XmlModelTypeMissing,
    /// A `<model>` of a member's own `face.xml` without a `path`; the folder is left out.
    XmlModelPathMissing,
    /// A `path` or `material` of a member's own `face.xml` naming, by `./` or by the
    /// uniform Common folder, a file the export does not hold; the folder is left out.
    XmlModelNotFound,
    /// A uniform Common `path` or `material` of a member's own `face.xml` whose folder after
    /// `common/` (the team's ID) is not three characters long; the folder is left out.
    XmlCommonPathInvalid,
    /// PES 2016: a model of a member's own `face.xml` whose file name starts with none of
    /// `face_high_`, `hair_high_`, `oral_`, which that game needs to load it; the folder is
    /// left out.
    XmlOralPrefixMissing,
    /// A child of a member's own `face.xml`'s `<config>` other than `<model>` and `<dif>`: the
    /// compiler cannot vouch for it, and keeps it as written.
    XmlElementUnknown,
    /// A `<model>` attribute of a member's own `face.xml` other than `level`, `type`, `path`,
    /// `material` and `ratio`: kept as written.
    XmlAttributeUnknown,
    /// A `type` of a member's own `face.xml` that the compiler never generates, so not
    /// verified in the game: kept as written.
    XmlTypeUnknown,
    /// A `level` other than 0 in a member's own `face.xml`: the model's level of detail,
    /// uncommon but known to work; kept as written.
    XmlLevelLod,
    /// A `ratio` of a member's own `face.xml` that is not a number: kept as written.
    XmlRatioInvalid,
    /// A `path` or `material` of a member's own `face.xml` in a form the compiler cannot
    /// resolve (a face Common path, any other game path): not checked to exist, kept as
    /// written.
    XmlPathUnchecked,
    /// A member's own `face.xml` with more than one `face_neck` entry: kept as written.
    XmlFaceNeckMultiple,
    /// A `.model` in a folder with its own `face.xml` that no entry lists: it is not in the
    /// game.
    XmlModelUnlisted,
    /// PES 2018 to 2021: a player folder's own `face.xml`, which Fox has no use for: it is
    /// ignored, and the folder's models compile as without it.
    XmlIgnoredFox,
    /// Two of a player's sources feeding different packages hold a texture of one stem with
    /// different bytes; the lower package in canonical order (face > boots > gloves) is left
    /// out with the textures only its sources hold.
    SharedTextureConflict,
    /// Two of a player's sources feeding one package hold a texture of one stem with
    /// different bytes, which the one model they build cannot choose between; the folder is
    /// left out.
    MergedTextureConflict,
    /// A texture with a side under 4 pixels, smaller than one block; the texture is discarded
    /// with what depends on it (`messages.md` "Textures": the model folder, the kit, the
    /// portrait; in `Common/`, the file alone).
    TextureTooSmall,
    /// A portrait with a side that is not a power of two, or a mipmapped Fox texture with
    /// one; discarded the same way.
    TextureNotPow2,
    /// PES 2015 to 2017: a texture with a side that is not a multiple of 4. Every pre-Fox texture
    /// is written block-compressed, which those games cannot create at such a size; discarded
    /// the same way, whatever `pass_through` says.
    TextureNotDiv4,
    /// A kit's main texture (`kit`, its own or inherited from `all/`) wider or taller than
    /// 2048 pixels, or with a side that is not a power of two; the kit is left out.
    KitTextureTooBig,
    /// A texture whose bytes open with another accepted format's signature than its
    /// extension's (renamed, not resaved); discarded the same way.
    TextureTypeMismatch,
    /// A texture in a codec, or with a feature, `dds_convert` cannot convert in-process;
    /// discarded the same way.
    TextureCodecUnsupported,
    /// A native sampler of the source engine's material table the target engine does not
    /// carry, dropped from a converted model's material (the conversion's form of the glTF
    /// role the target has no sampler for).
    MaterialTextureUnused,
    /// A used bone the target PES version's skeleton lacks had its weights transferred to the
    /// fold table's (or nearest) bone, in a model converted for the target.
    BoneFoldedForVersion,
    /// A converted model's bind pose was re-bound from its source version's skeleton to the
    /// target's (bones moved more than tolerance).
    SkeletonRetargeted,
    /// A converted model's bone has no companion `.skl` and a name in none of the target
    /// version's skeleton tables, so its bind matrix is the identity: the mesh it skins sits
    /// where the identity puts it.
    BoneMatrixUnknown,
    /// A converted mesh's vertex weighted a bone slot past the mesh's bone group: the slot was
    /// zeroed and the vertex's weights renormalized, so it skins differently.
    BoneSlotDropped,
    /// A used bone the target version lacks, with no fold-table entry, was folded onto the
    /// nearest target body bone by rest position: a guess.
    BoneFoldedByPosition,
    /// No shader rule named a converted material's shader, so its family, and the target
    /// shader, is the closest fit: the material may render differently.
    MaterialFamilyApproximated,
    /// Meshes of one FMDL material instance carried different flags, so the instance became
    /// two materials of the converted model.
    MaterialSplitByFlags,
    /// A native parameter of the source engine's material table the target shader does not
    /// carry, dropped from a converted model's material.
    MaterialParameterDropped,
    /// A `.mtl` sampler name with no stored settings table, written with the default sampler
    /// settings in the converted model.
    SamplerSettingsDefaulted,
    /// A converted mesh's bitangents, which an FMDL has no attribute for: the game derives
    /// them.
    VertexBitangentsDropped,
    /// A shaded or metal material with no normal or specular map got the game's dummy texture
    /// for the sampler a Fox model requires.
    DummyTextureAdded,
    /// A non-default field the target format has no home for, dropped with nothing the game
    /// would show (an FMDL's redundant local-space bone matrices, a `.skl` parent the FMDL's
    /// wins over, a normal or tangent `w` a `.model` cannot carry, a `.model` field).
    NativeFieldDropped,
    /// A Fox mesh flag a `.mtl` cannot express, `invisible` or `no_shadow_cast`, dropped: the
    /// mesh shows, or casts a shadow, where the source hid it.
    MeshFlagsDropped,
    /// An unskinned mesh was weighted to the `static` bone an FMDL needs.
    StaticBoneAdded,
    /// A converted mesh's vertices carried a weight above 1, clamped to the FMDL weight's
    /// maximum.
    WeightClamped,
    /// A vertexless mesh's bone group was written empty: it skins nothing.
    EmptyMeshBoneGroupDropped,
    /// A player folder's glTF model (`.glb` or `.gltf`) is its stem's selected representation,
    /// no model of the target's own format of its stem beside it, and the compiler does not
    /// read glTF yet: planning leaves the folder out, never compiling the other engine's
    /// `.model` or `.fmdl` beside it instead. Reported by `compile` alone, before the folder's
    /// tasks are made, so `pass_through` does not apply.
    ModelGltfUnsupported,
    /// A model the compiler must convert (an FMDL for PES 15-17, a `.model` for PES 18-21, or a
    /// face model to hand auto-split) cannot be read, converted or written back, or its
    /// converted form trips an Error of its format's check (the rule's code is the error); its
    /// package is left out.
    ModelConversionFailed,
    /// A model has a vertex more than 5000 units from the origin, which lags the game for the
    /// whole matchday; its folder is left out (a `Common/` model: the file), whatever
    /// `pass_through` says. Found on a source by the deep pass, and on a converted model's
    /// target form by its task, which leaves the model's package out.
    VertexTooFarFromOrigin,
    /// A model file (`.fmdl` or `.model`) that does not parse; its folder is left out (a
    /// `Common/` model: the file), whatever `pass_through` says, since it cannot be processed.
    ModelBroken,
    /// A `.mtl` that does not parse; its folder is left out (a `Common/` file: the file),
    /// whatever `pass_through` says.
    MtlBroken,
    /// A texture one of a Fox model's meshes uses, named in the team's Common output, that
    /// neither the export's `Common/`, the folder's links nor an installed CPK loaded before
    /// the run's supplies: the model's package is left out, or kept when the installed CPKs
    /// cannot be looked in.
    FmdlTextureNotFound,
    /// A file a task reads cannot be read from its export; its folder is left out.
    SourceReadFailed,
    /// A task could not build its entries; its folder is left out.
    FolderPackFailed,
    /// The PES folder has no `download/DpFileList.bin`, so every bin is built on its bundled
    /// base: an Error when the run deploys, a Warning when it deploys nothing.
    DpfilelistMissing,
    /// The installed `DpFileList.bin`, a CPK it lists or a bin in that CPK cannot be read; the
    /// run stops before any export is read, the previous CPK kept.
    InstalledBinUnreadable,
    /// The installed CPK a working bin was taken from, or `bundled` for the bundled base.
    BinSource,
    /// The run has `BootsList.bin` or `GloveList.bin` rows (Fox) and no installed CPK holds
    /// the table, which has no bundled base: the table is not written and the rows are left
    /// out.
    PlayerTableMissing,
    /// A file of the data directory's `templates/` folder cannot be read; the run stops before
    /// any export is read, the previous CPK kept.
    TemplateOverrideUnreadable,
    /// A file of the data directory's `templates/` folder replaces the embedded resource of
    /// its name for the run.
    TemplateOverrideActive,
    /// The `TeamColor.bin` or `UniColor.bin` the run built on held records whose header was
    /// not their position's; the headers are rewritten, the records' colors kept.
    BinHeaderRepaired,
    /// A task's entry or a bin at the path of a file of the `overrides/` folder: the override
    /// wins and the entry is left out.
    DuplicatePath,
    /// The CPK could not be written; the run's staging is discarded.
    CpkWriteFailed,
    /// The written CPK could not replace the previous one; the run's staging is discarded.
    OutputCommitFailed,
    /// `--no-deploy`: the CPK was promoted to the output folder instead of installed.
    DeploySkippedByFlag,
    /// A run that deploys, whose `pes_folder_path` is not a folder (no PES installed, or the
    /// setting is wrong): the CPK goes to the output folder.
    PesFolderNotFound,
    /// The PES folder holds no exe of the target version: `pes_version` or `pes_folder_path`
    /// may name the wrong game. The run still deploys.
    PesVersionMismatch,
    /// The installed `DpFileList.bin` lists the run's CPK but is not the official list: entries
    /// missing, entries the official list lacks, or another order. The run still deploys.
    DpfilelistNotOfficial,
    /// The installed `DpFileList.bin` names CPKs, other than the run's own, with no file in
    /// `download/`: the game then loads none of them. The run still deploys.
    DpfilelistCpkMissing,
    /// The installed `DpFileList.bin` does not list the run's CPK, which the official list
    /// names: an older list, so the game would not load it. The CPK goes to the output folder.
    DpfilelistOutdated,
    /// `upgrade-dpfl`: an old DLC's CPK renamed to an official name of its stem, so the game
    /// still loads it under the official list.
    DpfilelistCpkRenamed,
    /// `upgrade-dpfl`: the placeholder CPK written for an official entry with no file in
    /// `download/`, since the game loads none of the folder when a listed CPK is missing.
    DpfilelistPlaceholderWritten,
    /// `upgrade-dpfl`: the installed `DpFileList.bin` replaced by the official one, the old one
    /// kept as `DpFileList.bin.bak`.
    DpfilelistReplaced,
    /// `upgrade-dpfl`: an installed entry the official list lacks and that is not renamed: the
    /// game no longer loads it, and its file is left for the user to remove.
    DpfilelistCpkDropped,
    /// `upgrade-dpfl`: the installed list is the official one byte for byte and every CPK it
    /// names is in `download/`, so nothing is written.
    DpfilelistUpToDate,
    /// `upgrade-dpfl` without `--yes`: the findings before it are what `--yes` would do, and
    /// nothing was written.
    DpfilelistUpgradePlanned,
    /// The installed `DpFileList.bin` does not list the run's CPK, so the game would not load
    /// it: the CPK goes to the output folder.
    CpkNameUnlisted,
    /// Multi-CPK mode: a team does not fit the official list's teams slots left at
    /// `cpk_part_max_size`; the run is aborted and no part is written.
    CpkSlotsExhausted,
    /// Multi-CPK mode: one team's content is larger than `cpk_part_max_size` on its own, and a
    /// team is never split across parts; the run is aborted and no part is written.
    CpkTeamExceedsCap,
    /// A single CPK larger than `cpk_part_max_size`: the game loads it, but the cup DLC's
    /// repository cannot hold a file that large. The CPK is written whole.
    CpkSizeOverLimit,
    /// The old CPK in `download/` could not be replaced (PES is running): the CPK goes to the
    /// output folder.
    OldCpkLocked,
    /// `download/` denies writes (it needs administrator rights): the CPK goes to the output
    /// folder.
    DeployTargetUnwritable,
    /// The data directory's `overrides/` folder holds files, each put into the CPK at its path
    /// below the folder.
    OverridesActive,
    /// `teamnotes.txt` could not be written, or a previous one removed, in the output folder;
    /// the run's CPK stays in place.
    TeamnotesWriteFailed,
    /// A file a task read is gone, or its size or modified time is no longer what the
    /// export's listing gave (an archive: the archive file's); the run is aborted, its staging
    /// discarded.
    SourceChangedDuringRun,
}

impl Code {
    /// Every code, for the catalog test: a variant missing here would make its first message
    /// panic in `severity`, so a new variant is added to this list too.
    #[cfg(test)]
    const ALL: [Code; 129] = [
        Code::ExportExtractFailed,
        Code::NoExportsFound,
        Code::ExportDisabled,
        Code::ExportIdentified,
        Code::ExportBallsSkipped,
        Code::MultipleRefExports,
        Code::DuplicateAestheticsExport,
        Code::TeamNameUnknown,
        Code::ColorEntryInvalid,
        Code::TeamColorsMissing,
        Code::KitConfigGenerated,
        Code::KitConfigInvalid,
        Code::KitConfigVersionClamped,
        Code::KitConfigFpcAdjusted,
        Code::KitConfigFpcUnpatched,
        Code::KitPlaceholder,
        Code::KitTextureNotUsed,
        Code::KitColorsDerived,
        Code::KitColorsMissing,
        Code::KitLayoutConverted,
        Code::KitCollarReserved,
        Code::CollarIdInvalid,
        Code::CollarIdConflict,
        Code::LogoFitApplied,
        Code::LogoUpscaled,
        Code::ContentNotYetCompiled,
        Code::BootsIdPoolExhausted,
        Code::GlovesIdPoolExhausted,
        Code::PortraitConflict,
        Code::LinkCombined,
        Code::SettingsTomlInvalid,
        Code::FmdlFclHairFallback,
        Code::FmdlMerged,
        Code::ModelMerged,
        Code::ModelMergeFlagsConflict,
        Code::ModelHandSplit,
        Code::MergeMaterialConflict,
        Code::SklMergeConflict,
        Code::SklNoSlot,
        Code::FaceFileNotUsed,
        Code::KitVariantMissing,
        Code::KitVariantModelFox,
        Code::KitVariantMtlDiffers,
        Code::XmlFaceNeckAdded,
        Code::XmlUniformPes15,
        Code::FaceDiffInvalid,
        Code::XmlDifConflict,
        Code::XmlBroken,
        Code::MtlTextureNotFound,
        Code::MtlTextureUnusedMissing,
        Code::XmlRootTagInvalid,
        Code::XmlModelTypeMissing,
        Code::XmlModelPathMissing,
        Code::XmlModelNotFound,
        Code::XmlCommonPathInvalid,
        Code::XmlOralPrefixMissing,
        Code::XmlElementUnknown,
        Code::XmlAttributeUnknown,
        Code::XmlTypeUnknown,
        Code::XmlLevelLod,
        Code::XmlRatioInvalid,
        Code::XmlPathUnchecked,
        Code::XmlFaceNeckMultiple,
        Code::XmlModelUnlisted,
        Code::XmlIgnoredFox,
        Code::SharedTextureConflict,
        Code::MergedTextureConflict,
        Code::TextureTooSmall,
        Code::TextureNotPow2,
        Code::TextureNotDiv4,
        Code::KitTextureTooBig,
        Code::TextureTypeMismatch,
        Code::TextureCodecUnsupported,
        Code::MaterialTextureUnused,
        Code::BoneFoldedForVersion,
        Code::SkeletonRetargeted,
        Code::BoneMatrixUnknown,
        Code::BoneSlotDropped,
        Code::BoneFoldedByPosition,
        Code::MaterialFamilyApproximated,
        Code::MaterialSplitByFlags,
        Code::MaterialParameterDropped,
        Code::SamplerSettingsDefaulted,
        Code::VertexBitangentsDropped,
        Code::DummyTextureAdded,
        Code::NativeFieldDropped,
        Code::MeshFlagsDropped,
        Code::StaticBoneAdded,
        Code::WeightClamped,
        Code::EmptyMeshBoneGroupDropped,
        Code::ModelGltfUnsupported,
        Code::ModelConversionFailed,
        Code::VertexTooFarFromOrigin,
        Code::ModelBroken,
        Code::MtlBroken,
        Code::FmdlTextureNotFound,
        Code::SourceReadFailed,
        Code::FolderPackFailed,
        Code::DpfilelistMissing,
        Code::InstalledBinUnreadable,
        Code::BinSource,
        Code::PlayerTableMissing,
        Code::TemplateOverrideUnreadable,
        Code::TemplateOverrideActive,
        Code::BinHeaderRepaired,
        Code::DuplicatePath,
        Code::CpkWriteFailed,
        Code::OutputCommitFailed,
        Code::DeploySkippedByFlag,
        Code::PesFolderNotFound,
        Code::PesVersionMismatch,
        Code::DpfilelistNotOfficial,
        Code::DpfilelistCpkMissing,
        Code::DpfilelistOutdated,
        Code::DpfilelistCpkRenamed,
        Code::DpfilelistPlaceholderWritten,
        Code::DpfilelistReplaced,
        Code::DpfilelistCpkDropped,
        Code::DpfilelistUpToDate,
        Code::DpfilelistUpgradePlanned,
        Code::CpkNameUnlisted,
        Code::CpkSlotsExhausted,
        Code::CpkTeamExceedsCap,
        Code::CpkSizeOverLimit,
        Code::OldCpkLocked,
        Code::DeployTargetUnwritable,
        Code::OverridesActive,
        Code::TeamnotesWriteFailed,
        Code::SourceChangedDuringRun,
    ];

    /// The code as the catalog spells it, the stable id a message carries.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Code::ExportExtractFailed => "export_extract_failed",
            Code::NoExportsFound => "no_exports_found",
            Code::ExportDisabled => "export_disabled",
            Code::ExportIdentified => "export_identified",
            Code::ExportBallsSkipped => "export_balls_skipped",
            Code::MultipleRefExports => "multiple_ref_exports",
            Code::DuplicateAestheticsExport => "duplicate_aesthetics_export",
            Code::TeamNameUnknown => "team_name_unknown",
            Code::ColorEntryInvalid => "color_entry_invalid",
            Code::TeamColorsMissing => "team_colors_missing",
            Code::KitConfigGenerated => "kit_config_generated",
            Code::KitConfigInvalid => "kit_config_invalid",
            Code::KitConfigVersionClamped => "kit_config_version_clamped",
            Code::KitConfigFpcAdjusted => "kit_config_fpc_adjusted",
            Code::KitConfigFpcUnpatched => "kit_config_fpc_unpatched",
            Code::KitPlaceholder => "kit_placeholder",
            Code::KitTextureNotUsed => "kit_texture_not_used",
            Code::KitColorsDerived => "kit_colors_derived",
            Code::KitColorsMissing => "kit_colors_missing",
            Code::KitLayoutConverted => "kit_layout_converted",
            Code::KitCollarReserved => "kit_collar_reserved",
            Code::CollarIdInvalid => "collar_id_invalid",
            Code::CollarIdConflict => "collar_id_conflict",
            Code::LogoFitApplied => "logo_fit_applied",
            Code::LogoUpscaled => "logo_upscaled",
            Code::ContentNotYetCompiled => "content_not_yet_compiled",
            Code::BootsIdPoolExhausted => "boots_id_pool_exhausted",
            Code::GlovesIdPoolExhausted => "gloves_id_pool_exhausted",
            Code::PortraitConflict => "portrait_conflict",
            Code::LinkCombined => "link_combined",
            Code::SettingsTomlInvalid => "settings_toml_invalid",
            Code::FmdlFclHairFallback => "fmdl_fcl_hair_fallback",
            Code::FmdlMerged => "fmdl_merged",
            Code::ModelMerged => "model_merged",
            Code::ModelMergeFlagsConflict => "model_merge_flags_conflict",
            Code::ModelHandSplit => "model_hand_split",
            Code::MergeMaterialConflict => "merge_material_conflict",
            Code::SklMergeConflict => "skl_merge_conflict",
            Code::SklNoSlot => "skl_no_slot",
            Code::FaceFileNotUsed => "face_file_not_used",
            Code::KitVariantMissing => "kit_variant_missing",
            Code::KitVariantModelFox => "kit_variant_model_fox",
            Code::KitVariantMtlDiffers => "kit_variant_mtl_differs",
            Code::XmlFaceNeckAdded => "xml_face_neck_added",
            Code::XmlUniformPes15 => "xml_uniform_pes15",
            Code::FaceDiffInvalid => "face_diff_invalid",
            Code::XmlDifConflict => "xml_dif_conflict",
            Code::XmlBroken => "xml_broken",
            Code::MtlTextureNotFound => "mtl_texture_not_found",
            Code::MtlTextureUnusedMissing => "mtl_texture_unused_missing",
            Code::XmlRootTagInvalid => "xml_root_tag_invalid",
            Code::XmlModelTypeMissing => "xml_model_type_missing",
            Code::XmlModelPathMissing => "xml_model_path_missing",
            Code::XmlModelNotFound => "xml_model_not_found",
            Code::XmlCommonPathInvalid => "xml_common_path_invalid",
            Code::XmlOralPrefixMissing => "xml_oral_prefix_missing",
            Code::XmlElementUnknown => "xml_element_unknown",
            Code::XmlAttributeUnknown => "xml_attribute_unknown",
            Code::XmlTypeUnknown => "xml_type_unknown",
            Code::XmlLevelLod => "xml_level_lod",
            Code::XmlRatioInvalid => "xml_ratio_invalid",
            Code::XmlPathUnchecked => "xml_path_unchecked",
            Code::XmlFaceNeckMultiple => "xml_face_neck_multiple",
            Code::XmlModelUnlisted => "xml_model_unlisted",
            Code::XmlIgnoredFox => "xml_ignored_fox",
            Code::SharedTextureConflict => "shared_texture_conflict",
            Code::MergedTextureConflict => "merged_texture_conflict",
            Code::TextureTooSmall => "texture_too_small",
            Code::TextureNotPow2 => "texture_not_pow2",
            Code::TextureNotDiv4 => "texture_not_div4",
            Code::KitTextureTooBig => "kit_texture_too_big",
            Code::TextureTypeMismatch => "texture_type_mismatch",
            Code::TextureCodecUnsupported => "texture_codec_unsupported",
            Code::MaterialTextureUnused => "material_texture_unused",
            Code::BoneFoldedForVersion => "bone_folded_for_version",
            Code::SkeletonRetargeted => "skeleton_retargeted",
            Code::BoneMatrixUnknown => "bone_matrix_unknown",
            Code::BoneSlotDropped => "bone_slot_dropped",
            Code::BoneFoldedByPosition => "bone_folded_by_position",
            Code::MaterialFamilyApproximated => "material_family_approximated",
            Code::MaterialSplitByFlags => "material_split_by_flags",
            Code::MaterialParameterDropped => "material_parameter_dropped",
            Code::SamplerSettingsDefaulted => "sampler_settings_defaulted",
            Code::VertexBitangentsDropped => "vertex_bitangents_dropped",
            Code::DummyTextureAdded => "dummy_texture_added",
            Code::NativeFieldDropped => "native_field_dropped",
            Code::MeshFlagsDropped => "mesh_flags_dropped",
            Code::StaticBoneAdded => "static_bone_added",
            Code::WeightClamped => "weight_clamped",
            Code::EmptyMeshBoneGroupDropped => "empty_mesh_bone_group_dropped",
            Code::ModelGltfUnsupported => "model_gltf_unsupported",
            Code::ModelConversionFailed => "model_conversion_failed",
            Code::VertexTooFarFromOrigin => "vertex_too_far_from_origin",
            Code::ModelBroken => "model_broken",
            Code::MtlBroken => "mtl_broken",
            Code::FmdlTextureNotFound => "fmdl_texture_not_found",
            Code::SourceReadFailed => "source_read_failed",
            Code::FolderPackFailed => "folder_pack_failed",
            Code::DpfilelistMissing => "dpfilelist_missing",
            Code::InstalledBinUnreadable => "installed_bin_unreadable",
            Code::BinSource => "bin_source",
            Code::PlayerTableMissing => "player_table_missing",
            Code::TemplateOverrideUnreadable => "template_override_unreadable",
            Code::TemplateOverrideActive => "template_override_active",
            Code::BinHeaderRepaired => "bin_header_repaired",
            Code::DuplicatePath => "duplicate_path",
            Code::CpkWriteFailed => "cpk_write_failed",
            Code::OutputCommitFailed => "output_commit_failed",
            Code::DeploySkippedByFlag => "deploy_skipped_by_flag",
            Code::PesFolderNotFound => "pes_folder_not_found",
            Code::PesVersionMismatch => "pes_version_mismatch",
            Code::DpfilelistNotOfficial => "dpfilelist_not_official",
            Code::DpfilelistCpkMissing => "dpfilelist_cpk_missing",
            Code::DpfilelistOutdated => "dpfilelist_outdated",
            Code::DpfilelistCpkRenamed => "dpfilelist_cpk_renamed",
            Code::DpfilelistPlaceholderWritten => "dpfilelist_placeholder_written",
            Code::DpfilelistReplaced => "dpfilelist_replaced",
            Code::DpfilelistCpkDropped => "dpfilelist_cpk_dropped",
            Code::DpfilelistUpToDate => "dpfilelist_up_to_date",
            Code::DpfilelistUpgradePlanned => "dpfilelist_upgrade_planned",
            Code::CpkNameUnlisted => "cpk_name_unlisted",
            Code::CpkSlotsExhausted => "cpk_slots_exhausted",
            Code::CpkTeamExceedsCap => "cpk_team_exceeds_cap",
            Code::CpkSizeOverLimit => "cpk_size_over_limit",
            Code::OldCpkLocked => "old_cpk_locked",
            Code::DeployTargetUnwritable => "deploy_target_unwritable",
            Code::OverridesActive => "overrides_active",
            Code::TeamnotesWriteFailed => "teamnotes_write_failed",
            Code::SourceChangedDuringRun => "source_changed_during_run",
        }
    }
}

/// A code's severity as the catalog's "Sev" column gives it; some codes depend on the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CatalogSeverity {
    Info,
    Warning,
    Error,
    Fatal,
    /// `E/I`: an Error when `strict_file_type_check` is on, else an Info.
    ErrorOrInfo,
    /// `E/W`: an Error when the run deploys, a Warning when it deploys nothing.
    ErrorOrWarning,
    /// `E/F`: Fatal when the disposition aborts the run, else an Error.
    ErrorOrFatal,
    /// `E/W` keyed by the disposition: a Warning when the folder is kept (`Keep`), else an
    /// Error.
    ErrorUnlessKept,
}

/// Every code this tool emits in Phase 3, with its catalog severity: the structure pass's
/// (`aesthetics_export::ISSUE_CODES`) and the tool's own.
const CATALOG: &[(&str, CatalogSeverity)] = &[
    // The tool's own export-level codes.
    ("export_extract_failed", CatalogSeverity::Error),
    ("no_exports_found", CatalogSeverity::Warning),
    ("export_disabled", CatalogSeverity::Info),
    ("export_identified", CatalogSeverity::Info),
    ("export_balls_skipped", CatalogSeverity::Info),
    ("multiple_ref_exports", CatalogSeverity::Error),
    ("duplicate_aesthetics_export", CatalogSeverity::Error),
    ("color_entry_invalid", CatalogSeverity::Warning),
    ("team_colors_missing", CatalogSeverity::Info),
    ("kit_config_generated", CatalogSeverity::Info),
    ("kit_config_invalid", CatalogSeverity::Error),
    ("kit_config_version_clamped", CatalogSeverity::Warning),
    ("kit_config_fpc_adjusted", CatalogSeverity::Info),
    ("kit_config_fpc_unpatched", CatalogSeverity::Warning),
    ("kit_placeholder", CatalogSeverity::Info),
    ("kit_texture_not_used", CatalogSeverity::Info),
    ("kit_colors_derived", CatalogSeverity::Info),
    ("kit_colors_missing", CatalogSeverity::Warning),
    ("kit_layout_converted", CatalogSeverity::Info),
    ("kit_collar_reserved", CatalogSeverity::Error),
    ("collar_id_invalid", CatalogSeverity::Error),
    ("collar_id_conflict", CatalogSeverity::Error),
    ("logo_fit_applied", CatalogSeverity::Info),
    ("logo_upscaled", CatalogSeverity::Warning),
    ("content_not_yet_compiled", CatalogSeverity::Error),
    ("boots_id_pool_exhausted", CatalogSeverity::Error),
    ("gloves_id_pool_exhausted", CatalogSeverity::Error),
    ("portrait_conflict", CatalogSeverity::Error),
    ("link_combined", CatalogSeverity::Info),
    ("settings_toml_invalid", CatalogSeverity::Error),
    ("fmdl_fcl_hair_fallback", CatalogSeverity::Info),
    ("fmdl_merged", CatalogSeverity::Info),
    ("model_merged", CatalogSeverity::Info),
    ("model_merge_flags_conflict", CatalogSeverity::Error),
    ("model_hand_split", CatalogSeverity::Info),
    ("merge_material_conflict", CatalogSeverity::Error),
    ("skl_merge_conflict", CatalogSeverity::Error),
    ("skl_no_slot", CatalogSeverity::Warning),
    ("face_file_not_used", CatalogSeverity::Info),
    ("kit_variant_missing", CatalogSeverity::Warning),
    ("kit_variant_model_fox", CatalogSeverity::Warning),
    ("kit_variant_mtl_differs", CatalogSeverity::Warning),
    ("xml_face_neck_added", CatalogSeverity::Info),
    ("xml_uniform_pes15", CatalogSeverity::Info),
    ("face_diff_invalid", CatalogSeverity::Error),
    ("xml_dif_conflict", CatalogSeverity::Error),
    ("xml_broken", CatalogSeverity::Error),
    ("mtl_texture_not_found", CatalogSeverity::Warning),
    ("mtl_texture_unused_missing", CatalogSeverity::Info),
    ("xml_root_tag_invalid", CatalogSeverity::Error),
    ("xml_model_type_missing", CatalogSeverity::Error),
    ("xml_model_path_missing", CatalogSeverity::Error),
    ("xml_model_not_found", CatalogSeverity::Error),
    ("xml_common_path_invalid", CatalogSeverity::Error),
    ("xml_oral_prefix_missing", CatalogSeverity::Error),
    ("xml_element_unknown", CatalogSeverity::Warning),
    ("xml_attribute_unknown", CatalogSeverity::Warning),
    ("xml_type_unknown", CatalogSeverity::Warning),
    ("xml_level_lod", CatalogSeverity::Info),
    ("xml_ratio_invalid", CatalogSeverity::Warning),
    ("xml_path_unchecked", CatalogSeverity::Warning),
    ("xml_face_neck_multiple", CatalogSeverity::Warning),
    ("xml_model_unlisted", CatalogSeverity::Warning),
    ("xml_ignored_fox", CatalogSeverity::Info),
    ("shared_texture_conflict", CatalogSeverity::Error),
    ("merged_texture_conflict", CatalogSeverity::Error),
    ("texture_too_small", CatalogSeverity::Error),
    ("texture_not_pow2", CatalogSeverity::Error),
    ("texture_not_div4", CatalogSeverity::Error),
    ("kit_texture_too_big", CatalogSeverity::Error),
    ("texture_type_mismatch", CatalogSeverity::Error),
    ("texture_codec_unsupported", CatalogSeverity::Error),
    ("material_texture_unused", CatalogSeverity::Info),
    ("bone_folded_for_version", CatalogSeverity::Info),
    ("skeleton_retargeted", CatalogSeverity::Info),
    ("bone_matrix_unknown", CatalogSeverity::Warning),
    ("bone_slot_dropped", CatalogSeverity::Warning),
    ("bone_folded_by_position", CatalogSeverity::Warning),
    ("material_family_approximated", CatalogSeverity::Warning),
    ("material_split_by_flags", CatalogSeverity::Info),
    ("material_parameter_dropped", CatalogSeverity::Info),
    ("sampler_settings_defaulted", CatalogSeverity::Info),
    ("vertex_bitangents_dropped", CatalogSeverity::Info),
    ("dummy_texture_added", CatalogSeverity::Info),
    ("native_field_dropped", CatalogSeverity::Info),
    ("mesh_flags_dropped", CatalogSeverity::Warning),
    ("static_bone_added", CatalogSeverity::Info),
    ("weight_clamped", CatalogSeverity::Info),
    ("empty_mesh_bone_group_dropped", CatalogSeverity::Info),
    ("model_gltf_unsupported", CatalogSeverity::Error),
    ("model_conversion_failed", CatalogSeverity::Error),
    ("vertex_too_far_from_origin", CatalogSeverity::Error),
    ("model_broken", CatalogSeverity::Error),
    ("mtl_broken", CatalogSeverity::Error),
    ("fmdl_texture_not_found", CatalogSeverity::ErrorUnlessKept),
    ("folder_pack_failed", CatalogSeverity::ErrorOrFatal),
    ("dpfilelist_missing", CatalogSeverity::ErrorOrWarning),
    ("installed_bin_unreadable", CatalogSeverity::Fatal),
    ("bin_source", CatalogSeverity::Info),
    ("player_table_missing", CatalogSeverity::Warning),
    ("template_override_unreadable", CatalogSeverity::Fatal),
    ("template_override_active", CatalogSeverity::Info),
    ("bin_header_repaired", CatalogSeverity::Warning),
    // The catalog's `W/E`: only the override form, a Warning, has a trigger today.
    ("duplicate_path", CatalogSeverity::Warning),
    ("cpk_write_failed", CatalogSeverity::Fatal),
    ("output_commit_failed", CatalogSeverity::Fatal),
    ("deploy_skipped_by_flag", CatalogSeverity::Info),
    ("pes_folder_not_found", CatalogSeverity::Error),
    ("pes_version_mismatch", CatalogSeverity::Warning),
    ("dpfilelist_not_official", CatalogSeverity::Warning),
    ("dpfilelist_cpk_missing", CatalogSeverity::Warning),
    ("dpfilelist_outdated", CatalogSeverity::Error),
    ("dpfilelist_cpk_renamed", CatalogSeverity::Info),
    ("dpfilelist_placeholder_written", CatalogSeverity::Info),
    ("dpfilelist_replaced", CatalogSeverity::Info),
    ("dpfilelist_cpk_dropped", CatalogSeverity::Warning),
    ("dpfilelist_up_to_date", CatalogSeverity::Info),
    ("dpfilelist_upgrade_planned", CatalogSeverity::Info),
    ("cpk_name_unlisted", CatalogSeverity::Error),
    ("cpk_slots_exhausted", CatalogSeverity::Fatal),
    ("cpk_team_exceeds_cap", CatalogSeverity::Fatal),
    ("cpk_size_over_limit", CatalogSeverity::Warning),
    ("old_cpk_locked", CatalogSeverity::Error),
    ("deploy_target_unwritable", CatalogSeverity::Error),
    ("overrides_active", CatalogSeverity::Info),
    ("teamnotes_write_failed", CatalogSeverity::Error),
    ("source_changed_during_run", CatalogSeverity::Fatal),
    // The format crates' check codes (`fmdl::check::CODES`, `pes_model::check::CODES`), which
    // the deep pass reports under their own names, at the crate's severity: the far vertex
    // excepted, reported as `vertex_too_far_from_origin`. In the order of the plan's "Model
    // checks", then its `.mtl` rows and `model_material_undefined`.
    ("fmdl_mesh_over_bone_limit", CatalogSeverity::Error),
    ("fmdl_mesh_over_vertex_limit", CatalogSeverity::Error),
    ("fmdl_mesh_over_face_limit", CatalogSeverity::Error),
    ("fmdl_face_index_out_of_range", CatalogSeverity::Error),
    ("fmdl_mesh_unassigned", CatalogSeverity::Error),
    ("fmdl_bone_slot_out_of_range", CatalogSeverity::Warning),
    ("fmdl_mesh_empty", CatalogSeverity::Warning),
    ("fmdl_duplicate_bone_name", CatalogSeverity::Warning),
    ("fmdl_weights_not_normalized", CatalogSeverity::Info),
    ("fmdl_material_unused", CatalogSeverity::Info),
    ("model_mesh_over_bone_limit", CatalogSeverity::Error),
    ("model_mesh_over_vertex_limit", CatalogSeverity::Error),
    ("model_mesh_over_face_limit", CatalogSeverity::Error),
    ("model_face_index_out_of_range", CatalogSeverity::Error),
    ("model_bone_slot_out_of_range", CatalogSeverity::Warning),
    ("model_mesh_empty", CatalogSeverity::Warning),
    ("model_duplicate_bone_name", CatalogSeverity::Warning),
    ("model_lod_record_mismatch", CatalogSeverity::Warning),
    ("model_weights_not_normalized", CatalogSeverity::Info),
    ("model_degenerate_face", CatalogSeverity::Info),
    ("model_material_unused", CatalogSeverity::Info),
    ("mtl_material_duplicate", CatalogSeverity::Error),
    ("mtl_state_invalid", CatalogSeverity::Error),
    ("mtl_blendmode_nonzero", CatalogSeverity::Warning),
    ("mtl_state_missing", CatalogSeverity::Info),
    ("mtl_state_nonrecommended", CatalogSeverity::Info),
    ("mtl_state_unknown", CatalogSeverity::Info),
    ("model_material_undefined", CatalogSeverity::Error),
    // The structure pass's codes, in `ISSUE_CODES` order.
    ("nested_folders_fixed", CatalogSeverity::Warning),
    ("nested_root_ambiguous", CatalogSeverity::Error),
    ("nested_root_conflict", CatalogSeverity::Error),
    ("players_txt_invalid", CatalogSeverity::Error),
    ("refs_txt_ignored", CatalogSeverity::Warning),
    ("source_read_failed", CatalogSeverity::ErrorOrFatal),
    ("export_empty", CatalogSeverity::Error),
    ("team_name_unknown", CatalogSeverity::Error),
    ("export_tag_missing", CatalogSeverity::Error),
    ("players_txt_missing", CatalogSeverity::Error),
    ("players_txt_line_invalid", CatalogSeverity::Error),
    ("players_txt_slot_invalid", CatalogSeverity::Error),
    ("players_txt_slot_duplicate", CatalogSeverity::Error),
    ("players_txt_target_missing", CatalogSeverity::Error),
    ("player_unlisted", CatalogSeverity::Warning),
    ("player_folder_number_invalid", CatalogSeverity::Error),
    ("player_number_duplicate", CatalogSeverity::Error),
    ("file_type_disallowed", CatalogSeverity::ErrorOrInfo),
    ("common_file_disallowed", CatalogSeverity::ErrorOrInfo),
    ("shared_link_duplicate", CatalogSeverity::Error),
    ("link_target_missing", CatalogSeverity::Error),
    ("common_link_missing", CatalogSeverity::Error),
    ("link_target_dropped", CatalogSeverity::Error),
    ("shared_folder_orphaned", CatalogSeverity::Warning),
    ("fpc_conflict", CatalogSeverity::Error),
    ("ingame_face_explicit_face_model", CatalogSeverity::Error),
    ("texture_stem_conflict", CatalogSeverity::Error),
    ("fmdl_name_invalid", CatalogSeverity::Error),
    ("model_name_invalid", CatalogSeverity::Error),
    ("edithair_unsupported", CatalogSeverity::Error),
    ("kit_folder_invalid", CatalogSeverity::Error),
    ("kit_slot_duplicate", CatalogSeverity::Error),
    ("kit_texture_name_invalid", CatalogSeverity::Error),
    ("kit_layout_conflict", CatalogSeverity::Error),
    ("kit_icon_invalid", CatalogSeverity::Warning),
    ("kit_all_file_ignored", CatalogSeverity::Warning),
    ("kit_all_unused", CatalogSeverity::Warning),
    ("kit_textures_inherited", CatalogSeverity::Info),
    ("portrait_name_invalid", CatalogSeverity::Error),
    ("logo_file_invalid", CatalogSeverity::Error),
    ("logo_role_duplicate", CatalogSeverity::Error),
    ("logo_small_without_main", CatalogSeverity::Error),
    ("root_file_unexpected", CatalogSeverity::Warning),
    ("notes_found", CatalogSeverity::Info),
    ("notes_encoding_invalid", CatalogSeverity::Error),
];

/// The severity `code` is shown at, given what was done about it and `raised`, the run setting
/// that picks the higher severity of a row giving two: `strict_file_type_check` for `E/I`, a
/// run that deploys for `E/W`.
fn severity(code: &str, disposition: Disposition, raised: bool) -> Severity {
    let catalog = CATALOG
        .iter()
        .find(|(known, _)| *known == code)
        .map(|(_, severity)| *severity)
        .expect(
            "every code has a catalog row: the catalog test checks `Code::ALL` and `ISSUE_CODES`",
        );
    match catalog {
        CatalogSeverity::Info => Severity::Info,
        CatalogSeverity::Warning => Severity::Warning,
        CatalogSeverity::Error => Severity::Error,
        CatalogSeverity::Fatal => Severity::Fatal,
        // Keyed by the setting, not the disposition: `pass_through` keeps a disallowed file
        // (`Keep`) but its finding stays an Error.
        CatalogSeverity::ErrorOrInfo if raised => Severity::Error,
        CatalogSeverity::ErrorOrInfo => Severity::Info,
        CatalogSeverity::ErrorOrWarning if raised => Severity::Error,
        CatalogSeverity::ErrorOrWarning => Severity::Warning,
        CatalogSeverity::ErrorOrFatal if disposition == Disposition::AbortRun => Severity::Fatal,
        CatalogSeverity::ErrorOrFatal => Severity::Error,
        CatalogSeverity::ErrorUnlessKept if disposition == Disposition::Keep => Severity::Warning,
        CatalogSeverity::ErrorUnlessKept => Severity::Error,
    }
}

/// The message for one validation issue (the structure pass's own, or a content finding the
/// deep pass gave it) of the export `export_id`: the issue's scope placed
/// in that export, its effective disposition, and its severity from the catalog.
pub(crate) fn issue_message(
    issue: &ValidationIssue,
    export_id: ExportId,
    strict_file_type_check: bool,
) -> Message {
    let scope = match &issue.scope {
        IssueScope::Export => Scope::Export { export_id },
        IssueScope::Folder(path) => Scope::Folder {
            export_id,
            path: path.clone(),
        },
        IssueScope::File(path) => Scope::File {
            export_id,
            path: path.clone(),
        },
        IssueScope::RosterEntry { file, line, slot } => Scope::RosterEntry {
            export_id,
            file: file.clone(),
            line: *line,
            slot: *slot,
        },
    };
    let disposition = match issue.disposition {
        aesthetics_export::Disposition::Keep => Disposition::Keep,
        aesthetics_export::Disposition::DropFile => Disposition::DropFile,
        aesthetics_export::Disposition::DropSlot => Disposition::DropSlot,
        aesthetics_export::Disposition::DropFolder => Disposition::DropFolder,
        aesthetics_export::Disposition::DropExport => Disposition::DropExport,
    };
    let context = issue
        .context
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect();
    message(
        issue.code,
        scope,
        disposition,
        context,
        strict_file_type_check,
    )
}

/// A message for one of the tool's own codes, whose severity the strict setting never changes.
pub(crate) fn tool_message(
    code: Code,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(&'static str, String)>,
) -> Message {
    let context = context
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    message(code.as_str(), scope, disposition, context, false)
}

/// A message for one of the tool's own codes whose severity depends on whether the run
/// `deploys` (the catalog's `E/W`): an Error when it does, a Warning when it deploys nothing.
pub(crate) fn deploy_message(
    code: Code,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(&'static str, String)>,
    deploys: bool,
) -> Message {
    let context = context
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();
    message(code.as_str(), scope, disposition, context, deploys)
}

/// The message for `code`, its severity from the catalog given `disposition` and `raised`, the
/// run setting that picks the higher severity of a row giving two: `strict_file_type_check` for
/// `E/I`, a run that deploys for `E/W`.
fn message(
    code: &'static str,
    scope: Scope,
    disposition: Disposition,
    context: Vec<(String, String)>,
    raised: bool,
) -> Message {
    Message {
        code: MessageCode {
            tool_id: TOOL_ID,
            code: Cow::Borrowed(code),
        },
        severity: severity(code, disposition, raised),
        disposition,
        scope,
        context,
    }
}

/// A file size as a member reads it: `N bytes` under 1 KiB, else in the largest of KiB, MiB
/// and GiB it reaches, to one decimal, a trailing `.0` left out (`1.5 KiB`, `3.7 GiB`).
pub(crate) fn size_text(bytes: u64) -> String {
    const UNITS: [(&str, u64); 3] = [("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];
    let Some((unit, size)) = UNITS.into_iter().find(|(_, size)| bytes >= *size) else {
        return format!("{bytes} bytes");
    };
    // Tenths of the unit, rounded to the nearest, in integers: no float cast to round.
    let tenths = (u128::from(bytes) * 10 + u128::from(size) / 2) / u128::from(size);
    match tenths % 10 {
        0 => format!("{} {unit}", tenths / 10),
        digit => format!("{}.{digit} {unit}", tenths / 10),
    }
}

#[cfg(test)]
mod tests {
    use aesthetics_export::ISSUE_CODES;
    use vtree::ScopePath;

    use super::*;
    use crate::deep::FAR_VERTEX_CODES;

    fn issue(
        code: &'static str,
        scope: IssueScope,
        disposition: aesthetics_export::Disposition,
    ) -> ValidationIssue {
        ValidationIssue {
            code,
            scope,
            context: vec![("file", "readme.txt".to_owned()), ("line", "x3".to_owned())],
            disposition,
            passed_through: false,
        }
    }

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    #[test]
    fn a_size_is_in_bytes_under_a_kib_and_else_in_the_largest_unit_to_one_decimal() {
        assert_eq!(size_text(0), "0 bytes");
        assert_eq!(size_text(1023), "1023 bytes");
        assert_eq!(size_text(1024), "1 KiB");
        assert_eq!(size_text(1536), "1.5 KiB");
        assert_eq!(size_text(3_972_844_748), "3.7 GiB");
        assert_eq!(size_text(1 << 30), "1 GiB");
    }

    #[test]
    fn every_issue_code_has_exactly_one_row_and_no_row_is_doubled() {
        for code in ISSUE_CODES {
            let rows = CATALOG.iter().filter(|(known, _)| known == code).count();
            assert_eq!(rows, 1, "{code}");
        }
        for (index, (code, _)) in CATALOG.iter().enumerate() {
            assert!(
                CATALOG[index + 1..].iter().all(|(other, _)| other != code),
                "{code} appears twice"
            );
        }
        for code in Code::ALL {
            let rows = CATALOG
                .iter()
                .filter(|(known, _)| *known == code.as_str())
                .count();
            assert_eq!(rows, 1, "{code:?}");
        }
        for (code, severity) in format_codes() {
            let rows: Vec<CatalogSeverity> = CATALOG
                .iter()
                .filter(|(known, _)| *known == code)
                .map(|(_, severity)| *severity)
                .collect();
            assert_eq!(rows, [severity], "{code}");
        }
        for code in FAR_VERTEX_CODES {
            assert!(CATALOG.iter().all(|(known, _)| *known != code), "{code}");
        }
        // The rows beyond the structure pass's are exactly the tool's own codes, in `Code::ALL`
        // order, and the format crates', in the plan table's order, which no list in code
        // holds: those are compared as a set.
        let mut format: Vec<&str> = format_codes().into_iter().map(|(code, _)| code).collect();
        let own: Vec<&str> = Code::ALL
            .iter()
            .map(|code| code.as_str())
            .filter(|code| !ISSUE_CODES.contains(code))
            .collect();
        let beyond_structure = || {
            CATALOG
                .iter()
                .map(|(code, _)| *code)
                .filter(|code| !ISSUE_CODES.contains(code))
        };
        let own_rows: Vec<&str> = beyond_structure()
            .filter(|code| !format.contains(code))
            .collect();
        assert_eq!(own_rows, own);
        let mut format_rows: Vec<&str> = beyond_structure()
            .filter(|code| !own.contains(code))
            .collect();
        format.sort_unstable();
        format_rows.sort_unstable();
        assert_eq!(format_rows, format);
    }

    /// The format crates' check codes the tool reports under their own names (every one but
    /// the far vertex, reported as `vertex_too_far_from_origin`), at the crate's severity.
    fn format_codes() -> Vec<(&'static str, CatalogSeverity)> {
        let fox = fmdl::check::CODES.iter().map(|(code, severity)| {
            let severity = match severity {
                fmdl::check::Severity::Info => CatalogSeverity::Info,
                fmdl::check::Severity::Warning => CatalogSeverity::Warning,
                fmdl::check::Severity::Error => CatalogSeverity::Error,
            };
            (*code, severity)
        });
        let pre_fox = pes_model::check::CODES.iter().map(|(code, severity)| {
            let severity = match severity {
                pes_model::check::Severity::Info => CatalogSeverity::Info,
                pes_model::check::Severity::Warning => CatalogSeverity::Warning,
                pes_model::check::Severity::Error => CatalogSeverity::Error,
            };
            (*code, severity)
        });
        fox.chain(pre_fox)
            .filter(|(code, _)| !FAR_VERTEX_CODES.contains(code))
            .collect()
    }

    #[test]
    fn a_disallowed_file_is_an_error_only_under_the_strict_check() {
        for code in ["file_type_disallowed", "common_file_disallowed"] {
            assert_eq!(
                severity(code, Disposition::DropFolder, true),
                Severity::Error
            );
            // pass_through's Keep does not lower it.
            assert_eq!(severity(code, Disposition::Keep, true), Severity::Error);
            assert_eq!(severity(code, Disposition::Keep, false), Severity::Info);
        }
    }

    #[test]
    fn a_failed_pack_is_fatal_only_when_it_aborts_the_run() {
        for disposition in [
            Disposition::Keep,
            Disposition::DropFile,
            Disposition::DropSlot,
            Disposition::DropFolder,
            Disposition::DropExport,
        ] {
            let message = tool_message(Code::FolderPackFailed, Scope::Run, disposition, vec![]);
            assert_eq!(message.severity, Severity::Error, "{disposition:?}");
        }
        let message = tool_message(
            Code::FolderPackFailed,
            Scope::Run,
            Disposition::AbortRun,
            vec![],
        );
        assert_eq!(message.severity, Severity::Fatal);
    }

    #[test]
    fn a_texture_not_found_is_a_warning_only_when_the_folder_is_kept() {
        let message = |disposition| {
            tool_message(
                Code::FmdlTextureNotFound,
                Scope::Run,
                disposition,
                vec![("model", "face_high.fmdl".to_owned())],
            )
        };
        assert_eq!(message(Disposition::Keep).severity, Severity::Warning);
        assert_eq!(message(Disposition::DropFolder).severity, Severity::Error);
        assert_eq!(
            message(Disposition::Keep).code.code,
            "fmdl_texture_not_found"
        );
    }

    #[test]
    fn a_missing_dpfilelist_is_an_error_only_when_the_run_deploys() {
        let message = |deploys| {
            deploy_message(
                Code::DpfilelistMissing,
                Scope::Run,
                Disposition::Keep,
                vec![("path", "download/DpFileList.bin".to_owned())],
                deploys,
            )
        };
        assert_eq!(message(true).severity, Severity::Error);
        assert_eq!(message(false).severity, Severity::Warning);
        assert_eq!(message(false).code.code, "dpfilelist_missing");
        assert_eq!(
            message(false).context,
            [("path".to_owned(), "download/DpFileList.bin".to_owned())]
        );
    }

    #[test]
    fn plain_rows_keep_their_severity_whatever_the_settings() {
        for strict in [true, false] {
            assert_eq!(
                severity("player_unlisted", Disposition::DropFolder, strict),
                Severity::Warning
            );
            assert_eq!(
                severity("notes_found", Disposition::Keep, strict),
                Severity::Info
            );
            assert_eq!(
                severity("fpc_conflict", Disposition::DropFolder, strict),
                Severity::Error
            );
        }
    }

    #[test]
    #[should_panic(expected = "the catalog test checks")]
    fn a_code_with_no_row_is_a_programming_error() {
        severity("no_such_code", Disposition::Keep, true);
    }

    #[test]
    fn each_issue_scope_lands_in_the_export() {
        let export_id = ExportId(4);
        let cases = [
            (IssueScope::Export, Scope::Export { export_id }),
            (
                IssueScope::Folder(path("Players/03 - A")),
                Scope::Folder {
                    export_id,
                    path: path("Players/03 - A"),
                },
            ),
            (
                IssueScope::File(path("notes.txt")),
                Scope::File {
                    export_id,
                    path: path("notes.txt"),
                },
            ),
            (
                IssueScope::RosterEntry {
                    file: path("players.txt"),
                    line: 2,
                    slot: Some(3),
                },
                Scope::RosterEntry {
                    export_id,
                    file: path("players.txt"),
                    line: 2,
                    slot: Some(3),
                },
            ),
        ];
        for (issue_scope, scope) in cases {
            let issue = issue(
                "player_unlisted",
                issue_scope,
                aesthetics_export::Disposition::Keep,
            );
            assert_eq!(issue_message(&issue, export_id, true).scope, scope);
        }
    }

    #[test]
    fn each_disposition_maps_to_its_counterpart_with_the_code_and_context() {
        let cases = [
            (aesthetics_export::Disposition::Keep, Disposition::Keep),
            (
                aesthetics_export::Disposition::DropFile,
                Disposition::DropFile,
            ),
            (
                aesthetics_export::Disposition::DropSlot,
                Disposition::DropSlot,
            ),
            (
                aesthetics_export::Disposition::DropFolder,
                Disposition::DropFolder,
            ),
            (
                aesthetics_export::Disposition::DropExport,
                Disposition::DropExport,
            ),
        ];
        for (issue_disposition, disposition) in cases {
            let issue = issue("fpc_conflict", IssueScope::Export, issue_disposition);
            let message = issue_message(&issue, ExportId(0), true);
            assert_eq!(message.disposition, disposition);
            assert_eq!(
                message.code,
                MessageCode {
                    tool_id: "team-compiler",
                    code: Cow::Borrowed("fpc_conflict")
                }
            );
            assert_eq!(message.severity, Severity::Error);
            assert_eq!(
                message.context,
                [
                    ("file".to_owned(), "readme.txt".to_owned()),
                    ("line".to_owned(), "x3".to_owned())
                ]
            );
        }
    }

    #[test]
    fn an_issue_message_reads_the_strict_setting() {
        let issue = issue(
            "file_type_disallowed",
            IssueScope::Folder(path("Players/03 - A")),
            aesthetics_export::Disposition::Keep,
        );
        assert_eq!(
            issue_message(&issue, ExportId(0), true).severity,
            Severity::Error
        );
        assert_eq!(
            issue_message(&issue, ExportId(0), false).severity,
            Severity::Info
        );
    }

    #[test]
    fn a_tool_message_carries_its_scope_disposition_and_context() {
        let message = tool_message(
            Code::MultipleRefExports,
            Scope::Run,
            Disposition::DropExport,
            vec![("exports", "refs a, refs b".to_owned())],
        );
        assert_eq!(message.code.tool_id, "team-compiler");
        assert_eq!(message.code.code, "multiple_ref_exports");
        assert_eq!(message.severity, Severity::Error);
        assert_eq!(message.scope, Scope::Run);
        assert_eq!(message.disposition, Disposition::DropExport);
        assert_eq!(
            message.context,
            [("exports".to_owned(), "refs a, refs b".to_owned())]
        );
    }
}
