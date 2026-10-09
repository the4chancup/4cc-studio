//! Stage 2, run planning (`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"): the
//! identity-resolved exports become one manifest of tasks, each an atomic unit that commits
//! whole or not at all.

pub(crate) mod collars;
pub(crate) mod ids;
pub(crate) mod item_rows;
pub(crate) mod overrides;
pub(crate) mod subset;

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use aesthetics_export::{
    ExportCoverage, ExportIdentity, FileDescriptor, FileKind, FpcDirective, KitFolder, KitsFolder,
    LogoFiles, ModelFormat, PlayerFolder, PlayerIndex, ResolvedAestheticsExport, SharedKind,
    SharedModelFolder, ValidatedAestheticsExport, ValidatedRoster, common_link_name,
};
use kit_config::KitSlot;
use pes_version::{Engine, PesVersion};
use studio_core::{Disposition, ExportId, Message, Scope};
use teams_list::TeamId;
use vtree::ScopePath;

use crate::bins::Rgb;
use crate::kit_variants::{kit_number, model_variant_sets};
use crate::messages::{Code, tool_message};
use crate::mtl_search::mtl_for;
use crate::paths::{PackageKey, REFEREE_TEAM_ID, TextureHome};
use collars::export_collar;
use ids::{PlannedModelIds, shared_folders_taking_ids};
use item_rows::{ItemRow, RowPlayer, export_rows};
use subset::{
    FolderModels, ModelPackage, PlayerFile, common_file, common_skeleton, file_stem,
    first_not_compiled, is_direct_common_file, is_part_of, link_combines, link_feeds_own_package,
    link_name, linked_folder, named_as_face, package_of, player_file, selected_common_model,
    skeleton_slot, texture_format,
};

/// What planning produced: the manifest and the findings planning itself made.
pub(crate) struct PlanReport {
    /// Every task of the run, in canonical order.
    pub(crate) manifest: BuildManifest,
    /// Planning's findings (`content_not_yet_compiled`, `team_colors_missing`, `link_combined`,
    /// `kit_texture_not_used`, `kit_config_generated`, `kit_placeholder`,
    /// `kit_variant_model_fox`, `collar_id_conflict`, `model_gltf_unsupported`).
    pub(crate) messages: Vec<Message>,
}

/// The run's tasks in canonical order: by export, then each mapped player folder's tasks (its
/// face, boots and gloves packages, then its textures) by first roster slot, the shared boots
/// folders taking an id (each its package, then its textures) in id order, then the shared
/// gloves folders the same way, the export's Common textures as one task, the portraits by
/// player id, the kits by slot, the logo, then the collar. The writer lays the CPK out in this
/// order whatever order the tasks finish in, so the same exports always give the same bytes.
pub(crate) struct BuildManifest {
    /// The tasks, in canonical order.
    pub(crate) tasks: Vec<BuildTask>,
    /// Each planned team's id and the colors its root `colors.txt` gives, at most four, for
    /// its `TeamColor.bin` record, in export order. A team whose file gives no color, or that
    /// has no file, is not listed: its record keeps its bytes.
    pub(crate) team_colors: Vec<(u16, Vec<Rgb>)>,
    /// Each planned team export's kits, in export order, for its team's `UniColor.bin` record
    /// and `UniformParameter.bin` configs ("Bins accumulation").
    pub(crate) team_kits: Vec<TeamKits>,
    /// Every planned team export's compiled players' `BootsList.bin` and `GloveList.bin` rows,
    /// in export order ("Bins accumulation").
    pub(crate) item_rows: Vec<ItemRow>,
    /// Each planned export's team name as messages show it (`/co/`, `/refs/` for the
    /// referees) and the text of its root `notes.txt`, in export order, for `teamnotes.txt`.
    /// An export without a note is not listed.
    pub(crate) notes: Vec<(String, String)>,
}

/// A planned team export's kits.
pub(crate) struct TeamKits {
    /// The export, which an absent slot's FPC finding is reported on.
    pub(crate) export_id: ExportId,
    /// The export's team id.
    pub(crate) team_id: u16,
    /// Whether the export rebuilds its team's kits (`Full`) or adds to them (`Midcup`).
    pub(crate) coverage: ExportCoverage,
    /// What the export sets in every kit config of its team.
    pub(crate) edits: TeamKitEdits,
    /// The slots of its kit tasks, failed or not, ascending.
    pub(crate) slots: Vec<KitSlot>,
}

/// One unit of work: one package of a player folder's models, the folder's textures, one
/// player's portrait, one kit, the team's logo or its collar.
pub(crate) struct BuildTask {
    /// The export the task's content comes from.
    pub(crate) export_id: ExportId,
    /// The export's team id, which game paths and file names carry: 999 for a refs export
    /// (`REFEREE_TEAM_ID`).
    pub(crate) team_id: u16,
    /// What the task compiles.
    pub(crate) kind: TaskKind,
    /// The source bytes the task reads, what its memory permit is charged (`libs/pipeline.md`
    /// "Memory budget").
    pub(crate) charge: usize,
    /// The player folder's group the task belongs to, when its folder has textures; `None`
    /// for every other task, which the writer commits on its own.
    pub(crate) group: Option<TaskGroup>,
}

/// A model folder's tasks as one unit for the writer: its packages (face, boots, gloves in
/// canonical order; a shared folder has one), then its textures task last, contiguous in the
/// manifest. The writer holds the packages until the textures batch arrives and decides the
/// group, so a folder whose textures failed leaves no package pointing at textures that are
/// not in the CPK.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TaskGroup {
    /// The group's manifest positions, the textures task at `tasks.end - 1`.
    pub(crate) tasks: Range<usize>,
    /// The package each `Models` task of the group compiles, in manifest order: `packages[i]`
    /// is the package of task `tasks.start + i`, so the textures batch can name the task of a
    /// package it drops (`shared_texture_conflict`) for the writer to skip.
    pub(crate) packages: Vec<ModelPackage>,
    /// The sum of the members' charges: the coordinator acquires it once, as one permit the
    /// members share, since a textures task waiting for a permit of its own while the writer
    /// holds its packages' would wait forever.
    pub(crate) charge: usize,
}

/// The stem of the environment map a converted metal material names in its model folder's
/// texture home: the folder's own `env` texture, else the template the textures task emits
/// there (`ModelFolder::takes_template_environment_map`); a texture link of the stem makes it
/// the Common one.
pub(crate) const ENVIRONMENT_MAP_STEM: &str = "env";

/// The folder a `Models` or `Textures` task compiles: a mapped player folder, or a shared
/// boots or gloves folder a mapped player links plainly. Its files take the roles of a player
/// folder's (`subset::player_file`); where its textures go differs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelFolder {
    /// The folder's export path (`Players/05 - A`, `Boots/Crocs`): the scope its findings name.
    pub(crate) path: ScopePath,
    /// Its own files.
    pub(crate) files: Vec<FileDescriptor>,
    /// The player folder holds `ingame_face`: it gets no face package, and a model of its own
    /// that would be a part of the face's `fcl_hair` is a part of its boots
    /// (`player_folders.md` "`ingame_face` marker"). Never set for a shared folder.
    pub(crate) ingame_face: bool,
    /// The target's engine, which decides the role each of the folder's files takes
    /// (`subset::player_file`): each engine builds its own model format.
    pub(crate) engine: Engine,
    /// The shared folders a player folder combines, in link order. Empty for a shared folder,
    /// and for a player linking plainly or not at all.
    pub(crate) combined: Vec<CombinedFolder>,
    /// The player folder's `.common` model links, each resolved to the Common model it brings
    /// in as a part (`common_models`; on pre-Fox only an `ingame_face` player's). Empty for a
    /// shared folder, which holds no link.
    pub(crate) common_models: Vec<CommonModel>,
    /// The stems, folded, of the textures directly in the export's `Common/` folder, which the
    /// export's Common textures task emits into the team's Common output: a Common part's
    /// texture paths of these stems name that output, not the folder's texture home
    /// (`pipeline.md` "3. Per-model-folder parallel steps", step 6), and a texture any part
    /// names in that output is supplied when its stem is among them ("Resolved decisions", "A
    /// texture a model names must exist").
    pub(crate) common_texture_stems: BTreeSet<String>,
    /// The `.model`, FMDL, `.mtl` and texture files directly in the export's `Common/` folder,
    /// on a pre-Fox target for a player folder; empty on Fox, whose Common parts are resolved at
    /// planning (`common_models`), and for a shared folder, which holds no link. The face task
    /// tells a model link's Common `.model` from a Common FMDL the Common models task converts
    /// (`subset::selected_common_model`), its `.mtl` search looks in Common for a `.model.common`
    /// link and resolves a `.mtl.common` link there (`mtl_search::mtl_for`), and a texture link
    /// names its Common texture by the stem that file spells (`pipeline.md` "3. Per-model-folder
    /// parallel steps", steps 4 and 6);
    /// an `ingame_face` player's boots and gloves resolve a `.mtl.common` link there too
    /// (`ModelFolder::roles`) and point a copied Common `.mtl` at the textures among them.
    pub(crate) common_files: Vec<FileDescriptor>,
    /// The export paths of the player folder's **hand-split parts**: its face models whose
    /// vertices the deep pass found carrying hand weights (`hand_split_parts`). On Fox the face
    /// task packs each one's body, and the folder's gloves task, planned even with no
    /// glove-named file, reads each one and makes its hands `glove_l`/`glove_r` parts
    /// (`pipeline.md` "3. Per-model-folder parallel steps", step 3). On pre-Fox the face task
    /// alone splits each one, packing its hands as two more `face.xml` entries, and no gloves
    /// task is planned for them. Empty for a shared folder, whose models are never split.
    pub(crate) hand_split: BTreeSet<ScopePath>,
    /// One of the folder's packages converts an FMDL holding a metal material for PES 15-17
    /// (`converts_metal`): a player's face, an `ingame_face` player's boots or gloves, or a
    /// shared boots or gloves output. Its textures task emits the template environment map as
    /// `env.dds` in its texture home as that home spells it (a player's common folder; a shared
    /// output's own folder, which its `.mtl` files name as `./`), and is planned for it even
    /// when the folder holds no texture, unless one of its sources holds an `env` texture or a
    /// texture link of that stem (`env.dds.common`; `takes_template_environment_map`); the
    /// converting package points each converted `Basic_CNSR` material with no environment
    /// sampler at it (`model_format.md`, the `environment` role), at a link's in the team's
    /// Common output. Never set on PES 18-21.
    pub(crate) environment_map: bool,
    /// Where its textures go, which its models' texture paths are rewritten to name.
    pub(crate) textures: TextureHome,
}

/// A player folder's `.common` link to an FMDL or a `.model` (on pre-Fox an `ingame_face`
/// player's), resolved against the export's `Common/` folder as validation resolved it
/// (`player_folders.md` "Common model links and model merging"): the Common model is a part of
/// the package the link's role names, and its skeleton travels with it ("SKL pairing"), or
/// the `.mtl` a Common `.model` uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommonModel {
    /// The link file's export path (`Players/05 - A/legs.fmdl.common`), which the folder's
    /// files list and whose role (`PlayerFile::CommonModel`, or `PlayerFile::PreFoxPart` on
    /// pre-Fox) the Common model takes.
    pub(crate) link: ScopePath,
    /// The Common model the link loads (`Common/legs.fmdl`), the one of its stem in the
    /// target's own format when `Common/` holds both (`subset::selected_common_model`): what
    /// the Models task reads.
    pub(crate) model: FileDescriptor,
    /// The `.skl` of the model's stem directly in `Common/` (`Common/legs.skl`), when there is
    /// one and, on Fox, the role has a skeleton slot, on pre-Fox the model is an FMDL, whose
    /// conversion takes it as its bind pose; `None` otherwise, a slotless Fox role's skeleton
    /// being the structure pass's `skl_no_slot`.
    pub(crate) skeleton: Option<FileDescriptor>,
    /// For a Common `.model`, the `.mtl` the link's search finds (`mtl_search::mtl_for`): a
    /// `Common/` one, or the player's own override of it. On Fox the Models task converts the
    /// model with it (`pipeline.md` step 3 "Format conversion"); on pre-Fox it is read and
    /// written with the Common model, a part of his boots or gloves under `ingame_face`
    /// (`player_folders.md` "`ingame_face` with shared links"). `None` for a Common FMDL, whose
    /// materials travel inside it on Fox and whose conversion writes its material set on
    /// pre-Fox.
    pub(crate) material: Option<FileDescriptor>,
}

/// A shared folder a player folder combines (`player_folders.md` "A link plus local models
/// combines"), with the package it feeds: a `Faces/` folder the face, a `Boots/` folder the
/// boots, a `Gloves/` folder the gloves. Its models are parts of that package like the player's
/// own, and its textures join the player's textures task, counting for that package when a
/// stem conflicts (`pipeline.md` "3. Per-model-folder parallel steps", step 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CombinedFolder {
    /// The package the folder's models and textures feed.
    pub(crate) package: ModelPackage,
    /// The shared folder.
    pub(crate) folder: SharedModelFolder,
}

/// One source of a model folder's files with their roles (`ModelFolder::roles`): the package
/// the source feeds, the source folder's path, and each of its files `compile` builds with
/// its role.
pub(crate) type SourceRoles<'a> = (
    ModelPackage,
    &'a ScopePath,
    Vec<(&'a FileDescriptor, PlayerFile)>,
);

impl ModelFolder {
    /// The folder's files by source with each file's role, the folder's own files first and
    /// then each combined folder's. A role resolves against the file's own source
    /// (`Boots/Crocs`'s `boots.skl` pairs with `Boots/Crocs`'s `boots.fmdl`, not the
    /// player's). The face's `face_diff.bin` and `fcl_hair_sim.fclo` come once: a combined
    /// face folder's copy is left out when the player folder holds one, and never read
    /// (`player_folders.md` "A link plus local models combines"). A `face_diff.xml` counts as
    /// the `face_diff.bin` here; one source never holds both, the deep pass having dropped
    /// such a folder (`xml_dif_conflict`). A `.common` model link
    /// stands for the Common files it resolved to (`common_models`): the Common model under
    /// the link's role, in the link's place, and its Common skeleton under the role's slot,
    /// paired with it by their shared `Common/<stem>`; the empty link itself is never read. A
    /// Common `.model`'s `.mtl`, when its search finds one in `Common/`, follows it as a
    /// `PlayerFile::Material`, which the package converting it (Fox) or writing it (pre-Fox)
    /// reads. On pre-Fox only a link of a folder holding `ingame_face` is one (a part of his
    /// boots or gloves, `PlayerFile::PreFoxPart`), a Common FMDL bringing its Common `.skl`
    /// as a `PlayerFile::ConversionSkeleton`; under the marker a `.mtl.common`
    /// link likewise brings in its Common `.mtl` as one, the link kept beside it. Each Common
    /// file comes once per source, however many links stand for it.
    pub(crate) fn roles(&self) -> Vec<SourceRoles<'_>> {
        let mut own =
            FolderModels::of_player_files(&self.path, &self.files, self.ingame_face, self.engine);
        if self
            .combined
            .iter()
            .any(|shared| shared.package == ModelPackage::Face)
        {
            own = own.with_linked_face();
        }
        let mut sources = vec![(self.own_package(), &self.path, &self.files, own)];
        for shared in &self.combined {
            let path = &shared.folder.path;
            let files = &shared.folder.files;
            let models = FolderModels::of(path, files, self.engine);
            sources.push((shared.package, path, files, models));
        }
        // The names the face files kept so far pack as: a second copy of one file
        // (`face_diff.bin` beside `face/face_diff.bin`) is left out like an earlier source's.
        let mut packed: Vec<&'static str> = Vec::new();
        let mut roles = Vec::new();
        for (package, path, files, models) in sources {
            let mut source_roles = Vec::new();
            // The Common files already pushed for this source: two link spellings of one
            // Common file (`x.fmdl.common` and `x.fmdl.common.txt`) are one part, not two
            // reads of one model, and a Common `.mtl` two links find is read once.
            let mut common_pushed = BTreeSet::new();
            for file in files {
                let Some(role) = player_file(path, file, &models) else {
                    continue;
                };
                let packs_as = match role {
                    PlayerFile::Packed { name, .. } => Some(name),
                    PlayerFile::FaceDiffXml => Some("face_diff.bin"),
                    PlayerFile::CommonModel { package, name } => {
                        let resolved = self.common_model(&file.path);
                        if push_common(&mut source_roles, &mut common_pushed, &resolved.model, role)
                            && let Some(skeleton) = &resolved.skeleton
                        {
                            let name = skeleton_slot(package, name).expect(
                                "planning pairs a skeleton only with a part that has a slot",
                            );
                            source_roles.push((skeleton, PlayerFile::Skeleton { package, name }));
                        }
                        push_common_material(&mut source_roles, &mut common_pushed, resolved);
                        continue;
                    }
                    // Only a marked player's link to a model is a part: the Common model in
                    // its place, with a Common `.mtl` its search finds (his own one is among
                    // his files already), or a Common FMDL with its skeleton, the bind pose
                    // its conversion reads.
                    PlayerFile::PreFoxPart { .. } if file.kind == FileKind::CommonLink => {
                        let resolved = self.common_model(&file.path);
                        push_common(&mut source_roles, &mut common_pushed, &resolved.model, role);
                        if let Some(skeleton) = &resolved.skeleton {
                            push_common(
                                &mut source_roles,
                                &mut common_pushed,
                                skeleton,
                                PlayerFile::ConversionSkeleton,
                            );
                        }
                        push_common_material(&mut source_roles, &mut common_pushed, resolved);
                        continue;
                    }
                    // Under the marker a part's `.mtl` found through a link is copied in with
                    // it; the link stays too, for the part's search to see among his files.
                    PlayerFile::CommonMaterial if self.ingame_face => {
                        let linked = common_link_name(file.path.name())
                            .expect("a CommonMaterial role implies a `.common` link name");
                        let material = common_file(&self.common_files, &linked).expect(
                            "validation drops a player folder whose link names no Common file",
                        );
                        push_common(
                            &mut source_roles,
                            &mut common_pushed,
                            material,
                            PlayerFile::Material,
                        );
                        None
                    }
                    PlayerFile::Model { .. }
                    | PlayerFile::Skeleton { .. }
                    | PlayerFile::SlotlessSkeleton
                    | PlayerFile::UnusedFaceFile
                    | PlayerFile::LeftOutKitVariant
                    | PlayerFile::Texture(..)
                    | PlayerFile::CommonTexture(_)
                    | PlayerFile::PreFoxModel { .. }
                    | PlayerFile::PreFoxPart { .. }
                    | PlayerFile::PreFoxCommonModel { .. }
                    | PlayerFile::Material
                    | PlayerFile::CommonMaterial
                    | PlayerFile::FaceXml
                    | PlayerFile::ConversionSkeleton
                    | PlayerFile::UnsupportedGltf => None,
                };
                if let Some(packs_as) = packs_as {
                    if packed.contains(&packs_as) {
                        continue;
                    }
                    packed.push(packs_as);
                }
                source_roles.push((file, role));
            }
            roles.push((package, path, source_roles));
        }
        roles
    }

    /// The member's own `face.xml` the face reads: the first of the folder's own files with the
    /// role `PlayerFile::FaceXml` (pre-Fox, a face file, so never under `ingame_face`), when
    /// there is one. The deep pass checked every one; the face task writes this one back in
    /// place of a generated `face.xml` (`messages.md` "User-supplied `face.xml`").
    pub(crate) fn own_face_xml(&self) -> Option<&FileDescriptor> {
        let (_, _, own) = self.roles().into_iter().next()?;
        own.into_iter()
            .find(|(_, role)| *role == PlayerFile::FaceXml)
            .map(|(file, _)| file)
    }

    /// Whether the folder's textures task emits the template environment map: the folder is
    /// flagged (`environment_map`) and none of its sources holds an `env` texture or a texture
    /// link of that stem (`env.dds.common`), the one the converted metal materials then name.
    pub(crate) fn takes_template_environment_map(&self) -> bool {
        self.environment_map
            && !self.roles().into_iter().any(|(_, _, files)| {
                files.into_iter().any(|(_, role)| {
                    matches!(&role, PlayerFile::Texture(stem, _) | PlayerFile::CommonTexture(stem)
                        if vtree::fold_name(stem) == ENVIRONMENT_MAP_STEM)
                })
            })
    }

    /// Whether `file`, a skeleton (`PlayerFile::Skeleton`), is the `.skl` of one of the
    /// folder's hand-split parts (`hand_split`), paired with it as the Models task pairs a
    /// skeleton: by the path up to the extension, folded. The gloves task reads it, the bind
    /// pose of the part it converts again for its hands (`pipeline.md` step 3 "Format
    /// conversion").
    pub(crate) fn hand_split_skeleton(&self, file: &FileDescriptor) -> bool {
        let stem = vtree::fold_name(file_stem(file.path.as_str()));
        self.hand_split
            .iter()
            .any(|model| vtree::fold_name(file_stem(model.as_str())) == stem)
    }

    /// The Common model the folder's `.common` link at `link` resolved to (`common_models`).
    fn common_model(&self, link: &ScopePath) -> &CommonModel {
        self.common_models
            .iter()
            .find(|common| &common.link == link)
            .expect("planning resolves every `.common` model link of a folder")
    }

    /// The `.mtl` planning resolved for the Common `.model` at `model`, which a `.common` link
    /// of the folder loads (`CommonModel::material`); `None` for a Common FMDL. Every link to
    /// one model searches from the folder holding it, which may differ (`boots/` or the folder
    /// itself), but `roles` reads the model once, under its first link's role, so the first
    /// link's `.mtl` is the one.
    pub(crate) fn common_material(&self, model: &ScopePath) -> Option<&FileDescriptor> {
        self.common_models
            .iter()
            .find(|common| &common.model.path == model)
            .and_then(|common| common.material.as_ref())
    }

    /// The package the folder's own files feed, which its own textures count for when a stem
    /// conflicts and its pre-Fox models (`PlayerFile::PreFoxModel`) go into: a player folder's
    /// stand for its face, a shared folder's for its one package. A pre-Fox player holding
    /// `ingame_face` has no face, so his textures count for his boots. His models are parts of
    /// his boots or gloves, each saying which (`PlayerFile::PreFoxPart`), and his `.mtl` files
    /// are read by both (`TaskKind::files`): this answer is not where they go.
    fn own_package(&self) -> ModelPackage {
        match &self.textures {
            TextureHome::PlayerCommon { .. } => match self.engine {
                Engine::Fox => ModelPackage::Face,
                Engine::PreFox if self.ingame_face => ModelPackage::Boots,
                Engine::PreFox => ModelPackage::Face,
            },
            TextureHome::SharedOutput { package, .. } => *package,
        }
    }
}

/// Pushes the Common `file` under `role` onto `roles` unless `pushed`, the Common files pushed
/// so far for the source, holds it already, and says whether it pushed: two links standing for
/// one Common file give it once.
fn push_common<'a>(
    roles: &mut Vec<(&'a FileDescriptor, PlayerFile)>,
    pushed: &mut BTreeSet<String>,
    file: &'a FileDescriptor,
    role: PlayerFile,
) -> bool {
    if !pushed.insert(file.path.fold_key()) {
        return false;
    }
    roles.push((file, role));
    true
}

/// Pushes the `.mtl` the Common `.model` of `common` takes onto `roles` as a
/// `PlayerFile::Material` (`push_common`), when its search found a `Common/` one: a player's
/// own override is among his files already. Nothing for a Common FMDL, which takes none.
fn push_common_material<'a>(
    roles: &mut Vec<(&'a FileDescriptor, PlayerFile)>,
    pushed: &mut BTreeSet<String>,
    common: &'a CommonModel,
) {
    if let Some(material) = common
        .material
        .as_ref()
        .filter(|material| is_direct_common_file(&material.path))
    {
        push_common(roles, pushed, material, PlayerFile::Material);
    }
}

/// What a task compiles.
pub(crate) enum TaskKind {
    /// One package of a model folder's models: a player folder's face, boots or gloves, or a
    /// shared folder's one package, with the files packed beside them. One task per package,
    /// whatever the number of roster slots mapping a player folder: the package is built once
    /// and emitted under each slot's key.
    Models {
        /// The model folder.
        folder: ModelFolder,
        /// Which of its packages.
        package: ModelPackage,
        /// The keys the package is emitted under: for a player folder one per roster slot
        /// mapping it, in slot order (the slot's player id for the face, its planned
        /// boots/gloves id for the other two; a referee slot's own key for all three); for a
        /// shared folder its one shared id.
        ids: Vec<PackageKey>,
    },
    /// A model folder's own textures, converted once into the folder's texture home, which
    /// every package of the folder points at. Always the last task of the folder's `TaskGroup`.
    Textures {
        /// The model folder.
        folder: ModelFolder,
        /// The export's kit numbers, against which its texture variant sets are completed.
        kits: Vec<u8>,
    },
    /// The textures directly in the export's `Common/` folder, converted once into the team's
    /// Common output, whether or not a `.common` link uses them (`pipeline.md` "Resolved
    /// decisions", "Common textures are one task of their export"). One task per export, in no
    /// group: its textures serve every linking player, so it commits on its own, and when it
    /// fails the linking players still commit.
    CommonTextures {
        /// The `Common/` folder's export path, the scope the task's findings name.
        folder: ScopePath,
        /// Its textures directly in it, in any accepted image format.
        textures: Vec<FileDescriptor>,
        /// The export's kit numbers, against which its texture variant sets are completed.
        kits: Vec<u8>,
        /// The Common models task converts an FMDL holding a metal material for PES 15-17
        /// (`ExportToPlan::metal_models`) and no texture directly in `Common/` has the stem
        /// `env`: the task emits the template environment map as `env.dds` in the team's Common
        /// output, and is planned for it even when `Common/` holds no texture; the conversion
        /// points each converted `Basic_CNSR` material with no environment sampler at it
        /// (`model_format.md`, the `environment` role). Never set on PES 18-21.
        environment_map: bool,
    },
    /// Pre-Fox: the `.model` and `.mtl` files directly in the export's `Common/` folder,
    /// written once into the team's Common output, whether or not a `.common` link names them
    /// (`pipeline.md` "3. Per-model-folder parallel steps", step 4): the game loads a linked
    /// Common model or `.mtl` from there. An FMDL there no `.model` of its stem beats is
    /// converted once into a `.model` and its material set, the `.skl` of its stem its bind
    /// pose. One task per export holding such a file, in no group, as `CommonTextures`.
    CommonModels {
        /// The `Common/` folder's export path, the scope the task's findings name.
        folder: ScopePath,
        /// Its `.model` and `.mtl` files directly in it, and each FMDL it converts with the
        /// `.skl` of that FMDL's stem (`common_model_files`).
        files: Vec<FileDescriptor>,
        /// The stems of its textures directly in it, folded, each as the folder spells it: a
        /// `.mtl` path naming one is pointed at that texture in the team's Common output.
        texture_stems: BTreeMap<String, String>,
    },
    /// One player's portrait, emitted as a DDS under the target version's file name: a DDS
    /// source as it is, any other accepted format encoded to BC3 (`player_folders.md`
    /// "Portraits"). One task per player id: a folder two roster slots map gives two tasks
    /// over its one `portrait.*`.
    Portrait {
        /// The player id the portrait is for.
        player_id: u32,
        /// The portrait file: the player folder's `portrait.*`, or `Portraits/player_NN.*`.
        file: FileDescriptor,
    },
    /// One kit, its `all/` inheritance already applied to its textures.
    Kit {
        /// The game's kit slot.
        slot: KitSlot,
        /// The kit folder.
        kit: KitFolder,
        /// What its team's export sets in every kit config of the team.
        edits: TeamKitEdits,
    },
    /// The team's logo: the game's three PNGs made from the export's root `logo*` file, the
    /// smallest from its `logo_small*` file when there is one (`pipeline.md` "4. Per-export
    /// non-model steps", Logo). One task per export that has a logo, all three or none.
    Logo {
        /// The main file and the small one, each with its fit tag.
        logo: LogoFiles,
    },
    /// The referees' marker (`blue_port.md` "Referee export processing"): the refs export's
    /// `ref_marker.dds` converted into the referees' Common output, and the bundled marker
    /// model written as their reserved collar, its texture pointed at it. Both or neither: a
    /// collar naming a texture the CPK lacks would draw untextured under every referee. One
    /// task per refs export holding the file.
    RefereeMarker {
        /// The export's root `ref_marker.dds`.
        marker: FileDescriptor,
    },
    /// The team's collar (`pipeline.md` "Collars"): its `Collars/collar_<ID>` model in the
    /// format the target reads (`.fmdl` on Fox, `.model` on pre-Fox) written unchanged in place
    /// of stock collar `id`, keeping the materials its author gave it. One task per team export
    /// holding a collar it claimed (`collars::export_collar`).
    Collar {
        /// The collar file.
        file: FileDescriptor,
        /// The stock collar it replaces.
        id: u8,
    },
}

/// What a team export sets in every kit config of its team, its own kits' and, for a `Midcup`
/// export, those of the kits it does not resend (`pipeline.md` "Collars", `fpc_toggle.md`
/// "Team kit-FPC status and kit configs"): the FPC values first, then its collar, which so
/// wins over the FPC collar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TeamKitEdits {
    /// The team's kit-FPC status.
    pub(crate) fpc: EffectiveTeamKitFpc,
    /// The stock collar the export's collar replaces, set as every config's collar and winter
    /// collar; `None` when the export holds no collar or lost its claim.
    pub(crate) collar: Option<u8>,
}

/// Whether the team's kit configs must carry the FPC values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EffectiveTeamKitFpc {
    /// At least one player folder of the validated export carries `fpc_on`.
    On,
    /// No `fpc_on` marker: no claim about the team, supplied configs are left as they are.
    Unknown,
}

impl EffectiveTeamKitFpc {
    /// The status of the team of `export`, the validated export: `On` when one of its player
    /// folders carries `fpc_on`. A folder validation dropped is not among them, so its marker
    /// does not count, and `fpc_off` is a statement about its own player alone.
    fn of(export: &ValidatedAestheticsExport) -> EffectiveTeamKitFpc {
        if export
            .players
            .iter()
            .any(|folder| folder.fpc == Some(FpcDirective::On))
        {
            EffectiveTeamKitFpc::On
        } else {
            EffectiveTeamKitFpc::Unknown
        }
    }
}

impl TaskKind {
    /// The folder the task compiles, as the export spells it (a portrait's is its file, the
    /// logo's its main file): the scope its findings name.
    pub(crate) fn folder_path(&self) -> ScopePath {
        match self {
            TaskKind::Models { folder, .. } | TaskKind::Textures { folder, .. } => {
                folder.path.clone()
            }
            TaskKind::CommonTextures { folder, .. } | TaskKind::CommonModels { folder, .. } => {
                folder.clone()
            }
            TaskKind::Portrait { file, .. } => file.path.clone(),
            TaskKind::Kit { kit, .. } => kit.path.clone(),
            TaskKind::Logo { logo } => logo.main.file.path.clone(),
            TaskKind::RefereeMarker { marker } => marker.path.clone(),
            TaskKind::Collar { file, .. } => file.path.clone(),
        }
    }

    /// Every file the task reads from its export: a package's models (a `.common` link's
    /// Common model and skeleton, never the link) and the files packed beside them, the Fox
    /// gloves' also the folder's hand-split face parts, whose hands they take, with each part's
    /// `.skl`, its bind pose (`ModelFolder::hand_split_skeleton`), a Fox package
    /// converting a `.model` also the `.mtl` files of the model's source (for a Common
    /// `.model`, the player's and the Common `.mtl` its search found), a pre-Fox package
    /// also the skeleton of each FMDL it may convert; a folder's
    /// textures; the Common textures; the Common models and `.mtl` files, a converted Common
    /// FMDL's skeleton included (pre-Fox); a
    /// portrait's one file; a kit's config and `colors.txt`,
    /// when it has them, and its effective textures; the logo's main file and its small one,
    /// when it has one; the referees' marker texture; the collar file.
    pub(crate) fn files(&self) -> Vec<&FileDescriptor> {
        match self {
            TaskKind::Models {
                folder, package, ..
            } => {
                let reads_model = |source, file: &FileDescriptor, role: &PlayerFile| {
                    let hands = match folder.engine {
                        Engine::Fox => {
                            *package == ModelPackage::Gloves
                                && (folder.hand_split.contains(&file.path)
                                    || (matches!(role, PlayerFile::Skeleton { .. })
                                        && folder.hand_split_skeleton(file)))
                        }
                        // A pre-Fox split model is the face's alone (`folder_tasks`).
                        Engine::PreFox => false,
                    };
                    // A Fox `.mtl` is the face's only by `PlayerFile::package`'s pre-Fox
                    // answer: it is read below, with the `.model` it may define.
                    let fox_material = matches!(role, PlayerFile::Material)
                        && match folder.engine {
                            Engine::Fox => true,
                            Engine::PreFox => false,
                        };
                    (role.package() == Some(*package) && !fox_material)
                        || hands
                        // A pre-Fox model goes into the package its source feeds: a shared
                        // boots or gloves folder's into its own output, a folder an
                        // `ingame_face` player combines into his package of its kind.
                        || (matches!(role, PlayerFile::PreFoxModel { .. }) && *package == source)
                };
                // On Fox only a package converting a `.model` reads a `.mtl`: the FMDLs carry
                // their materials, and a `.mtl` beside a `.model` an FMDL beats is read by
                // nothing (`pipeline.md` step 3 "Format conversion").
                let converts = match folder.engine {
                    Engine::Fox => folder_files(folder, |source, _, file, role| {
                        reads_model(source, file, role)
                    })
                    .iter()
                    .any(|file| file.kind == FileKind::Model(ModelFormat::PesModel)),
                    Engine::PreFox => true,
                };
                folder_files(folder, |source, source_path, file, role| {
                    reads_model(source, file, role)
                        // A `.mtl` goes where its source's models go, each package packing (on
                        // Fox, converting with) the ones its models use (`mtl_for`): a combined
                        // folder's into the player's package of its kind, the folder's own into
                        // each of its packages, a pre-Fox `ingame_face` player's models being
                        // parts of his boots and of his gloves (`PlayerFile::PreFoxPart`).
                        || (matches!(role, PlayerFile::Material)
                            && converts
                            && (*package == source || source_path == &folder.path))
                        // A pre-Fox FMDL's skeleton, its bind pose, goes where its models go,
                        // as a `.mtl` does: the face's, a shared boots or gloves output's, a
                        // combined folder's into the player's package of its kind, and an
                        // `ingame_face` player's own into his boots and his gloves, either of
                        // which may convert the FMDL it pairs with.
                        || (matches!(role, PlayerFile::ConversionSkeleton)
                            && (*package == source || source_path == &folder.path))
                })
            }
            TaskKind::Textures { folder, .. } => folder_files(folder, |_, _, _, role| {
                matches!(role, PlayerFile::Texture(..))
            }),
            TaskKind::CommonTextures { textures, .. } => textures.iter().collect(),
            TaskKind::CommonModels { files, .. } => files.iter().collect(),
            TaskKind::Portrait { file, .. } => vec![file],
            TaskKind::Kit { kit, .. } => kit
                .config
                .iter()
                .chain(&kit.colors)
                .chain(kit.textures.iter().map(|texture| &texture.file))
                .collect(),
            TaskKind::Logo { logo } => std::iter::once(&logo.main)
                .chain(&logo.small)
                .map(|file| &file.file)
                .collect(),
            TaskKind::RefereeMarker { marker } => vec![marker],
            TaskKind::Collar { file, .. } => vec![file],
        }
    }
}

/// The files of `folder` that `wanted` accepts, given each with its source (`ModelFolder::roles`:
/// the package it feeds and its export path) and its role: its own in their order, then each
/// combined folder's.
fn folder_files(
    folder: &ModelFolder,
    wanted: impl Fn(ModelPackage, &ScopePath, &FileDescriptor, &PlayerFile) -> bool,
) -> Vec<&FileDescriptor> {
    let mut wanted_files = Vec::new();
    for (source, source_path, files) in folder.roles() {
        for (file, role) in files {
            if wanted(source, source_path, file, &role) {
                wanted_files.push(file);
            }
        }
    }
    wanted_files
}

/// The export paths of `folder`'s hand-split parts on a target of `engine`
/// (`ModelFolder::hand_split`): its face models whose path is among `hand_weighted`, the
/// models the deep pass found carrying hand weights. On Fox those are its face parts, its own,
/// a combined folder's or a Common model's. On pre-Fox they are the `.model` files its face
/// packs and names as face content, its own and a combined shared face's: never a model a
/// `.common` link brings in, which the face lists by reference in the team's Common output and
/// packs nothing of, and never an `ingame_face` player's part, which has no face. A model
/// named as boots or gloves is never one on either engine, whatever its weights: an authored
/// glove is all hand, and a boots model is on the body skeleton already
/// (`model_conversion/hand_split.md` "Pipeline integration"). A folder holding its own
/// `face.xml` has none (`ModelFolder::own_face_xml`).
fn hand_split_parts(
    folder: &ModelFolder,
    hand_weighted: &BTreeSet<ScopePath>,
    engine: Engine,
) -> BTreeSet<ScopePath> {
    // The member's xml says what the face loads: a split would add glove entries he did not
    // write (`messages.md` "User-supplied `face.xml`", the paragraph "What is emitted").
    if folder.own_face_xml().is_some() {
        return BTreeSet::new();
    }
    folder
        .roles()
        .into_iter()
        .flat_map(|(_, source_path, files)| {
            files.into_iter().filter(move |(file, role)| match engine {
                Engine::Fox => is_part_of(role, ModelPackage::Face),
                Engine::PreFox => {
                    matches!(role, PlayerFile::PreFoxModel { .. })
                        && named_as_face(source_path, file)
                }
            })
        })
        .filter(|(file, _)| hand_weighted.contains(&file.path))
        .map(|(file, _)| file.path.clone())
        .collect()
}

/// Whether one of `folder`'s packages converts an FMDL among `metal_models`, the FMDLs the deep
/// pass found holding a metal material (`ModelFolder::environment_map`): one of its sources'
/// files a pre-Fox package converts, a face's or a shared boots or gloves output's model
/// (`PlayerFile::PreFoxModel`) or a part of an `ingame_face` player's boots or gloves
/// (`PlayerFile::PreFoxPart`), roles only a pre-Fox target gives (a `.model` with one is never
/// among the metal models, which are FMDLs). A folder holding its own `face.xml` converts
/// none: the xml lists the face's models, and names no converted one.
fn converts_metal(folder: &ModelFolder, metal_models: &BTreeSet<ScopePath>) -> bool {
    if folder.own_face_xml().is_some() {
        return false;
    }
    folder.roles().into_iter().any(|(_, _, files)| {
        files.into_iter().any(|(file, role)| {
            let converted = matches!(
                role,
                PlayerFile::PreFoxModel { .. }
                    | PlayerFile::PreFoxPart {
                        package: ModelPackage::Boots | ModelPackage::Gloves,
                        ..
                    }
            );
            converted && metal_models.contains(&file.path)
        })
    })
}

/// Every player folder a roster slot maps, in the export's folder order. A folder no slot maps
/// is not compiled.
pub(crate) fn mapped_players(export: &ValidatedAestheticsExport) -> Vec<&PlayerFolder> {
    let mapped: Vec<PlayerIndex> = match &export.roster {
        ValidatedRoster::Team(slots) => slots.values().copied().collect(),
        ValidatedRoster::Referees(slots) => slots.values().copied().collect(),
    };
    export
        .players
        .iter()
        .enumerate()
        .filter(|(index, _)| mapped.contains(&PlayerIndex(*index)))
        .map(|(_, folder)| folder)
        .collect()
}

/// One identity-resolved export as planning takes it, with what validation read beside it.
pub(crate) struct ExportToPlan {
    /// The export's id.
    pub(crate) export_id: ExportId,
    /// The export, its identity resolved.
    pub(crate) export: ResolvedAestheticsExport,
    /// The valid colors of its root `colors.txt`; `None` when it has no such file.
    pub(crate) team_colors: Option<Vec<Rgb>>,
    /// The text of its root `notes.txt`; `None` when it has none.
    pub(crate) notes: Option<String>,
    /// The export paths of its models (`.fmdl`, `.model`) whose vertices carry hand weights,
    /// as the deep pass found them (`deep::ContentPass::hand_weighted`): a player's face model
    /// among them is hand auto-split (`ModelFolder::hand_split`).
    pub(crate) hand_weighted: BTreeSet<ScopePath>,
    /// The export paths of its FMDLs holding a metal material, as the deep pass found them
    /// (`deep::ContentPass::metal_models`): a model folder one of whose pre-Fox packages
    /// converts one of them gets the template environment map (`ModelFolder::environment_map`).
    pub(crate) metal_models: BTreeSet<ScopePath>,
}

/// Plans the run over the identity-resolved exports, given in `ExportId` order, for the target
/// `version`. A mapped player folder whose model is a selected glTF is dropped first, and so
/// is a shared folder whose model is one, with every player folder linking it
/// (`drop_gltf_folders`). An export holding anything Phase 3 cannot compile yet plans no task
/// and reports `content_not_yet_compiled` naming the first such item. Every other export's
/// note goes into the manifest, and a team export's colors; one with no root `colors.txt` reports
/// `team_colors_missing`, and its team keeps the colors it had. A team export's collar is
/// claimed against the run-wide list of the collars earlier exports claimed
/// (`collars::export_collar`), and every kit config of the team wears the collar it keeps. A
/// refs export plans its referee folders, shared folders, Common textures and marker under
/// team id 999 (`REFEREE_TEAM_ID`), each folder's packages keyed by its referee slots, and
/// nothing else: it has no colors record, kits, rows, portraits, logo or collar.
pub(crate) fn plan_run(exports: Vec<ExportToPlan>, version: PesVersion) -> PlanReport {
    let mut tasks = Vec::new();
    let mut team_colors = Vec::new();
    let mut team_kits = Vec::new();
    let mut item_rows = Vec::new();
    let mut notes = Vec::new();
    let mut messages = Vec::new();
    // Each claimed collar's ID and its claimant's name. Planning is serial and takes the
    // exports in canonical order, so the earlier export keeps a collar two exports claim.
    let mut claimed_collars: BTreeMap<u8, String> = BTreeMap::new();
    for ExportToPlan {
        export_id,
        export: mut resolved,
        team_colors: colors,
        notes: note,
        hand_weighted,
        metal_models,
    } in exports
    {
        drop_other_engine_map(
            version.engine(),
            export_id,
            &mut resolved.export.kits,
            &mut messages,
        );
        drop_gltf_folders(export_id, &mut resolved, version, &mut messages);
        if let Some(item) = first_not_compiled(&resolved, version) {
            messages.push(tool_message(
                Code::ContentNotYetCompiled,
                Scope::Export { export_id },
                Disposition::DropExport,
                vec![item],
            ));
            continue;
        }
        // The referees have no team record (no colors, kits or rows) and no player ids; the
        // gate has named their kits, logo and portraits, so only their folders compile.
        let team = match resolved.identity {
            ExportIdentity::Team { id, .. } => Some(id),
            ExportIdentity::Referees => None,
        };
        let team_id = team.map_or(REFEREE_TEAM_ID, TeamId::get);
        if team.is_some() {
            match colors {
                None => messages.push(tool_message(
                    Code::TeamColorsMissing,
                    Scope::Export { export_id },
                    Disposition::Keep,
                    vec![],
                )),
                // A file whose every line was refused: the deep pass reported the lines, and
                // there is nothing to write.
                Some(colors) if colors.is_empty() => {}
                Some(colors) => team_colors.push((team_id, colors)),
            }
        }
        let mut export = resolved.export;
        // The export's team name (`/co/`, `/refs/`) heads its note.
        if let Some(note) = note {
            notes.push((export.team_name.as_str().to_owned(), note));
        }
        let model_ids = team.map(PlannedModelIds::for_team);
        let fpc = EffectiveTeamKitFpc::of(&export);
        // The kit numbers the export defines, ascending (its kits go by slot), which each
        // textures task completes its variant sets against.
        let kits: Vec<u8> = export
            .kits
            .kits
            .keys()
            .filter_map(|slot| kit_number(*slot))
            .collect();
        kit_variant_model_messages(export_id, &export, version.engine(), &mut messages);
        // The textures directly in `Common/`: one task of the export's, and the stems a Common
        // part's paths name that task's output for, which a model of any folder may name.
        let common_textures: Vec<FileDescriptor> = export
            .common
            .iter()
            .filter(|file| texture_format(file.path.name()).is_some())
            .cloned()
            .collect();
        let common_texture_stems: BTreeSet<String> = common_textures
            .iter()
            .map(|file| vtree::fold_name(file_stem(file.path.name())))
            .collect();
        // Pre-Fox loads Common at run time: its `.model` and `.mtl` files, and its FMDLs
        // converted, are one more task of the export's, and a player's face names them and its
        // textures, which its task finds among the model, `.mtl` and texture files. The gate
        // has named any other `Common/` file.
        let (common_model_files, player_common_files): (Vec<FileDescriptor>, Vec<FileDescriptor>) =
            match version.engine() {
                Engine::Fox => (Vec::new(), Vec::new()),
                Engine::PreFox => (
                    common_model_files(&export.common),
                    export
                        .common
                        .iter()
                        .filter(|file| {
                            matches!(
                                file.kind,
                                FileKind::Model(ModelFormat::PesModel | ModelFormat::Fmdl)
                                    | FileKind::Mtl
                            )
                        })
                        .chain(&common_textures)
                        .cloned()
                        .collect(),
                ),
            };
        // The template environment map goes into the team's Common output for a converted
        // Common FMDL holding a metal material, unless `Common/` holds an `env` texture, which
        // the converted material then names.
        let common_environment_map = common_model_files.iter().any(|file| {
            file.kind == FileKind::Model(ModelFormat::Fmdl) && metal_models.contains(&file.path)
        }) && !common_texture_stems.contains(ENVIRONMENT_MAP_STEM);
        // The shared folders taking an id, each with its package and that id, in the id order
        // of the kind: the boots folders, then the gloves folders.
        let mut shared: Vec<(ModelFolder, ModelPackage, u32)> = Vec::new();
        for kind in [SharedKind::Boots, SharedKind::Gloves] {
            let package = package_of(kind);
            let folders = shared_folders_taking_ids(&export, version.engine(), kind);
            for (index, folder) in folders.into_iter().enumerate() {
                let shared_id = u32::from(model_ids.and_then(|ids| ids.shared(index)).expect(
                    "a referee's links take no shared id, and the structure pass drops a \
                         team export whose shared pool is exhausted",
                ));
                let mut folder = ModelFolder {
                    path: folder.path.clone(),
                    files: folder.files.clone(),
                    ingame_face: false,
                    engine: version.engine(),
                    combined: Vec::new(),
                    common_models: Vec::new(),
                    common_texture_stems: common_texture_stems.clone(),
                    common_files: Vec::new(),
                    hand_split: BTreeSet::new(),
                    environment_map: false,
                    textures: TextureHome::SharedOutput {
                        package,
                        id: shared_id,
                    },
                };
                folder.environment_map = converts_metal(&folder, &metal_models);
                shared.push((folder, package, shared_id));
            }
        }
        // A folder's portrait goes out once per slot mapping the folder. A player id with a
        // `Portraits/` file too comes up twice; the deep pass has skipped an export whose two
        // files differ, so they are the same bytes and the folder's is packed once (below).
        let mut portraits: Vec<(u32, FileDescriptor)> = Vec::new();
        // The player folders are taken out so the rest of the export (its shared folders)
        // stays readable while each folder's links are resolved against it.
        let players = std::mem::take(&mut export.players);
        // The export's first task, from which its compiled players' rows look for the tasks
        // building their boots and gloves.
        let first_task = tasks.len();
        let mut row_players = Vec::new();
        let mapped = mapped_folders(players, &export.roster, team);
        for MappedFolder {
            folder,
            packages,
            player_ids,
        } in mapped
        {
            row_players.push(RowPlayer {
                path: folder.path.clone(),
                player_ids: player_ids.clone(),
                linked: folder
                    .links
                    .iter()
                    .filter_map(|link| linked_folder(&export, link))
                    .map(|shared| shared.path.clone())
                    .collect(),
            });
            if let Some(portrait) = &folder.portrait {
                portraits.extend(
                    player_ids
                        .iter()
                        .map(|player_id| (*player_id, portrait.clone())),
                );
            }
            // A link feeding the player's own package (`link_feeds_own_package`) makes the
            // shared folder's files a second source of the player's folder. One beside a
            // local model of its package combines and is reported, once however many slots
            // map the folder; a referee's plain link is his package as it is, which tells the
            // member nothing new.
            let mut combined = Vec::new();
            for link in folder.links.iter().filter(|link| {
                link_feeds_own_package(&export.roster, version.engine(), &folder, link)
            }) {
                let shared = linked_folder(&export, link)
                    .expect("validation drops a player folder whose link names no shared folder");
                if link_combines(&folder, link, version.engine()) {
                    messages.push(tool_message(
                        Code::LinkCombined,
                        Scope::Folder {
                            export_id,
                            path: folder.path.clone(),
                        },
                        Disposition::Keep,
                        vec![("link", link_name(link.kind, &link.name))],
                    ));
                }
                combined.push(CombinedFolder {
                    package: package_of(link.kind),
                    folder: shared.clone(),
                });
            }
            let mut model_folder = ModelFolder {
                textures: TextureHome::PlayerCommon {
                    folder_name: folder.path.name().to_owned(),
                },
                common_models: common_models(&folder, &export.common, version.engine()),
                common_texture_stems: common_texture_stems.clone(),
                common_files: player_common_files.clone(),
                hand_split: BTreeSet::new(),
                environment_map: false,
                path: folder.path,
                files: folder.files,
                ingame_face: folder.ingame_face,
                engine: version.engine(),
                combined,
            };
            model_folder.hand_split =
                hand_split_parts(&model_folder, &hand_weighted, version.engine());
            model_folder.environment_map = converts_metal(&model_folder, &metal_models);
            // Without a face folder the game shows the head made in its face editor, which
            // `ingame_face` asks for; every other player gets one, blank when it holds no
            // face model (the last part of FPC: the body brings its own head, or none).
            let blank_face = !model_folder.ingame_face;
            folder_tasks(
                export_id,
                team_id,
                model_folder,
                &packages,
                blank_face,
                &kits,
                &mut tasks,
            );
        }
        for (folder, package, shared_id) in shared {
            folder_tasks(
                export_id,
                team_id,
                folder,
                &[(package, vec![PackageKey::Id(shared_id)])],
                false,
                &kits,
                &mut tasks,
            );
        }
        // After the shared folders' tasks, which a player linking one takes his row from. A
        // referee has none: the game's referee hook loads slot NN's `k99NN`/`g99NN` by number.
        // Nor does a pre-Fox target, which writes no `BootsList.bin` or `GloveList.bin`
        // ("Game paths reference"): rows there would only be reported as
        // `player_table_missing`.
        let writes_item_rows = match version.engine() {
            Engine::Fox => team.is_some(),
            Engine::PreFox => false,
        };
        if writes_item_rows {
            item_rows.extend(export_rows(
                &tasks,
                first_task,
                &row_players,
                export.coverage,
            ));
        }
        // The stems the Common `.mtl` files' paths are pointed at, each as `Common/` spells it:
        // the name its converted DDS has in the team's Common output.
        let texture_stems: BTreeMap<String, String> = common_textures
            .iter()
            .map(|file| {
                let stem = file_stem(file.path.name());
                (vtree::fold_name(stem), stem.to_owned())
            })
            .collect();
        // Planned for the template environment map alone when `Common/` holds no texture.
        if !common_textures.is_empty() || common_environment_map {
            let folder = common_textures
                .iter()
                .chain(&common_model_files)
                .next()
                .and_then(|first| first.path.parent())
                .expect("a Common texture or model sits in the export's Common/ folder");
            tasks.push(task(
                export_id,
                team_id,
                TaskKind::CommonTextures {
                    folder,
                    textures: common_textures,
                    kits,
                    environment_map: common_environment_map,
                },
            ));
        }
        if let Some(first) = common_model_files.first() {
            let folder = first
                .path
                .parent()
                .expect("a Common model or `.mtl` sits in the export's Common/ folder");
            tasks.push(task(
                export_id,
                team_id,
                TaskKind::CommonModels {
                    folder,
                    files: common_model_files,
                    texture_stems,
                },
            ));
        }
        // Validation keeps a `ref_marker.dds` on a refs export alone. Its task lies in the
        // export's range, so its entries go where the referees' do.
        if let Some(marker) = export.root.referee_marker.take() {
            tasks.push(task(export_id, team_id, TaskKind::RefereeMarker { marker }));
        }
        // The rest is a team's: the gate has named any portrait, kit or logo of a refs export,
        // and the referees have no `UniColor.bin` record for a kits entry to edit.
        let Some(id) = team else {
            continue;
        };
        portraits.extend(
            export
                .portraits
                .into_iter()
                .map(|(slot, file)| (slot.player_id(id), file)),
        );
        // A stable sort keeps the folder's file, pushed first, ahead of the `Portraits/` file
        // of its player id, and `dedup` keeps the first of each id.
        portraits.sort_by_key(|(player_id, _)| *player_id);
        portraits.dedup_by_key(|(player_id, _)| *player_id);
        for (player_id, file) in portraits {
            tasks.push(task(
                export_id,
                team_id,
                TaskKind::Portrait { player_id, file },
            ));
        }
        // An export the gate skipped has claimed nothing, so its collar takes no other's.
        let collar = export_collar(
            export_id,
            &export.export_display_name,
            &export.collars,
            version.engine(),
            &mut claimed_collars,
            &mut messages,
        );
        let edits = TeamKitEdits {
            fpc,
            collar: collar.as_ref().map(|(_, id)| *id),
        };
        // The kits go by slot, so `slots` is ascending.
        let mut slots = Vec::new();
        for (slot, kit) in export.kits.kits {
            slots.push(slot);
            let folder = || Scope::Folder {
                export_id,
                path: kit.path.clone(),
            };
            if kit.config.is_none() {
                messages.push(tool_message(
                    Code::KitConfigGenerated,
                    folder(),
                    Disposition::Keep,
                    vec![],
                ));
            }
            if !kit.textures.iter().any(|texture| texture.stem == "kit") {
                messages.push(tool_message(
                    Code::KitPlaceholder,
                    folder(),
                    Disposition::Keep,
                    vec![],
                ));
            }
            tasks.push(task(export_id, team_id, TaskKind::Kit { slot, kit, edits }));
        }
        team_kits.push(TeamKits {
            export_id,
            team_id,
            coverage: export.coverage,
            edits,
            slots,
        });
        if let Some(logo) = export.logo {
            tasks.push(task(export_id, team_id, TaskKind::Logo { logo }));
        }
        if let Some((file, id)) = collar {
            tasks.push(task(export_id, team_id, TaskKind::Collar { file, id }));
        }
    }
    PlanReport {
        manifest: BuildManifest {
            tasks,
            team_colors,
            team_kits,
            item_rows,
            notes,
        },
        messages,
    }
}

/// Pushes `folder`'s tasks onto `tasks`: one `Models` task for each of `packages` any of the
/// folder's sources holds a model of (a face link alone makes the shared face the player's),
/// for the face whatever the folder holds when `blank_face` is set, and on Fox for the gloves
/// when the folder has a hand-split part (`ModelFolder::hand_split`), emitted under that
/// package's keys, then, when the folder has textures or takes the template environment map
/// (`ModelFolder::environment_map`), its `Textures` task completing its variant sets against
/// `kits`, the export's kit numbers, the lot as one `TaskGroup`.
fn folder_tasks(
    export_id: ExportId,
    team_id: u16,
    folder: ModelFolder,
    packages: &[(ModelPackage, Vec<PackageKey>)],
    blank_face: bool,
    kits: &[u8],
    tasks: &mut Vec<BuildTask>,
) {
    let first = tasks.len();
    let mut held = Vec::new();
    for (package, ids) in packages {
        // A pre-Fox model is a part of the package its source feeds, whatever its `face.xml`
        // type: a player's face, a shared folder's boots or gloves, the boots or gloves of an
        // `ingame_face` player combining a folder of their kind. His own parts say their
        // package.
        let models = folder_files(&folder, |source, _, _, role| {
            is_part_of(role, *package)
                || (matches!(role, PlayerFile::PreFoxModel { .. }) && *package == source)
        });
        let blank = blank_face && *package == ModelPackage::Face;
        let hands = match folder.engine {
            // A hand-split face part gives the folder gloves, whatever its files are named.
            Engine::Fox => *package == ModelPackage::Gloves && !folder.hand_split.is_empty(),
            // The face task packs a split model's hands as entries of its own `face.xml`.
            Engine::PreFox => false,
        };
        if models.is_empty() && !blank && !hands {
            continue;
        }
        held.push(*package);
        tasks.push(task(
            export_id,
            team_id,
            TaskKind::Models {
                folder: folder.clone(),
                package: *package,
                ids: ids.clone(),
            },
        ));
    }
    // The template environment map is one of the folder's textures, so a folder holding no
    // texture of its own still gets a textures task for it. One emitting nothing is not
    // planned: the writer takes an empty textures batch for a failed one and drops the folder.
    if !folder.takes_template_environment_map()
        && folder_files(&folder, |_, _, _, role| {
            matches!(role, PlayerFile::Texture(..))
        })
        .is_empty()
    {
        return;
    }
    tasks.push(task(
        export_id,
        team_id,
        TaskKind::Textures {
            folder,
            kits: kits.to_vec(),
        },
    ));
    let members = &mut tasks[first..];
    let charge = members
        .iter()
        .fold(0usize, |sum, task| sum.saturating_add(task.charge));
    let group = TaskGroup {
        tasks: first..first + members.len(),
        packages: held,
        charge,
    };
    for task in members {
        task.group = Some(group.clone());
    }
}

/// Removes from every kit's effective textures the map a target of `engine` does not read, its
/// `kit_srm` on PES 15-17 and its `kit_mask` on PES 18-21, each reported as
/// `kit_texture_not_used` on its kit folder, naming the file (`pipeline.md` "4. Per-export
/// non-model steps", Kits: mask and srm are engine-specific, and neither is converted into
/// the other). It goes before the subset gate and before the kit's task, which therefore never
/// read it. A kit's effective set holds one file per stem, so one finding per kit.
fn drop_other_engine_map(
    engine: Engine,
    export_id: ExportId,
    kits: &mut KitsFolder,
    messages: &mut Vec<Message>,
) {
    let not_read = match engine {
        Engine::PreFox => "kit_srm",
        Engine::Fox => "kit_mask",
    };
    for kit in kits.kits.values_mut() {
        let Some(at) = kit
            .textures
            .iter()
            .position(|texture| texture.stem == not_read)
        else {
            continue;
        };
        let map = kit.textures.remove(at);
        messages.push(tool_message(
            Code::KitTextureNotUsed,
            Scope::Folder {
                export_id,
                path: kit.path.clone(),
            },
            Disposition::DropFile,
            vec![("file", map.file.path.name().to_owned())],
        ));
    }
}

/// Drops every mapped player folder of `export` holding a glTF model selected for its stem on
/// `version` (`PlayerFile::UnsupportedGltf`), and every shared folder holding one with each
/// mapped player folder linking it (`pipeline.md` step 3 "Format conversion": a selected glTF
/// drops its folder until Phase 7 rather than falling through to the other engine's format,
/// and a player without the face, boots or gloves he linked would compile to something he did
/// not ask for). `model_gltf_unsupported` is reported on the player folder once per such file:
/// his own, named below his folder, or, when he holds none, those of the shared folders he
/// links, named by their export paths; each in the export's folder order. The player folder's
/// roster slots are removed, as validation removes a dropped folder's, so it plans no task and
/// its slots compile as empty ones do, and the shared folder is removed from the export, as is
/// every shared folder no remaining mapped player links, with no finding of its own (the
/// drop's names the cause, and such a folder would get no id and no task). It goes before the
/// subset gate, which therefore never walks any of them.
fn drop_gltf_folders(
    export_id: ExportId,
    export: &mut ResolvedAestheticsExport,
    version: PesVersion,
    messages: &mut Vec<Message>,
) {
    let export = &mut export.export;
    let engine = version.engine();
    let mapped: Vec<PlayerIndex> = match &export.roster {
        ValidatedRoster::Team(slots) => slots.values().copied().collect(),
        ValidatedRoster::Referees(slots) => slots.values().copied().collect(),
    };
    // Each selected glTF of a shared folder, with that folder's path, faces first, then boots,
    // then gloves, as the export lists them.
    let mut shared_gltfs: Vec<(ScopePath, ScopePath)> = Vec::new();
    for folder in export
        .faces
        .iter()
        .chain(&export.boots)
        .chain(&export.gloves)
    {
        let models = FolderModels::of(&folder.path, &folder.files, engine);
        for file in &folder.files {
            if player_file(&folder.path, file, &models) == Some(PlayerFile::UnsupportedGltf) {
                shared_gltfs.push((folder.path.clone(), file.path.clone()));
            }
        }
    }
    let mut dropped = Vec::new();
    for (index, folder) in export.players.iter().enumerate() {
        let index = PlayerIndex(index);
        if !mapped.contains(&index) {
            continue;
        }
        let models = FolderModels::of_player(folder, engine);
        // Named below the folder, as `xml_ignored_fox` names its file: two glTFs of one name
        // in different subfolders are two findings a member can tell apart.
        let mut files: Vec<String> = folder
            .files
            .iter()
            .filter(|file| {
                player_file(&folder.path, file, &models) == Some(PlayerFile::UnsupportedGltf)
            })
            .map(|file| crate::deep::relative(&file.path, &folder.path))
            .collect();
        // A folder dropped for its own glTF is not told about a linked one as well.
        if files.is_empty() {
            let linked: Vec<&ScopePath> = folder
                .links
                .iter()
                .filter_map(|link| linked_folder(export, link))
                .map(|shared| &shared.path)
                .collect();
            files = shared_gltfs
                .iter()
                .filter(|(shared, _)| linked.contains(&shared))
                .map(|(_, file)| file.as_str().to_owned())
                .collect();
        }
        if !files.is_empty() {
            dropped.push(index);
        }
        for file in files {
            messages.push(tool_message(
                Code::ModelGltfUnsupported,
                Scope::Folder {
                    export_id,
                    path: folder.path.clone(),
                },
                Disposition::DropFolder,
                vec![("file", file)],
            ));
        }
    }
    let holds_gltf = |folder: &SharedModelFolder| {
        shared_gltfs
            .iter()
            .any(|(shared, _)| *shared == folder.path)
    };
    export.faces.retain(|folder| !holds_gltf(folder));
    export.boots.retain(|folder| !holds_gltf(folder));
    export.gloves.retain(|folder| !holds_gltf(folder));
    match &mut export.roster {
        ValidatedRoster::Team(slots) => slots.retain(|_, index| !dropped.contains(index)),
        ValidatedRoster::Referees(slots) => slots.retain(|_, index| !dropped.contains(index)),
    }
    // A shared folder only dropped players linked would get no id and no task; validation
    // drops such orphans before planning, so outside a drop this removes nothing.
    let linked: Vec<ScopePath> = mapped_players(export)
        .iter()
        .flat_map(|folder| &folder.links)
        .filter_map(|link| linked_folder(export, link))
        .map(|shared| shared.path.clone())
        .collect();
    export.faces.retain(|folder| linked.contains(&folder.path));
    export.boots.retain(|folder| linked.contains(&folder.path));
    export.gloves.retain(|folder| linked.contains(&folder.path));
}

/// `kit_variant_model_fox` for each set of per-kit model files (`pants_kit1.fmdl`,
/// `pants_kit2.model`) in a folder of `export` that is compiled, on that folder: a mapped player
/// folder, or a shared folder a mapped player links (validation drops one no mapped player
/// links, and `drop_gltf_folders` one only the players it dropped linked). Each folder is
/// walked once, so a shared folder several players combine reports its set once. Nothing on a
/// pre-Fox target, where the whole set is packed and listed once as `pants_kitN` for the game
/// to respell (`pipeline.md` "Kit-dependent assets"), converted FMDLs included: the warning
/// is about Fox having no such indirection.
fn kit_variant_model_messages(
    export_id: ExportId,
    export: &ValidatedAestheticsExport,
    engine: Engine,
    messages: &mut Vec<Message>,
) {
    if engine == Engine::PreFox {
        return;
    }
    let players = mapped_players(export)
        .into_iter()
        .map(|folder| (&folder.path, &folder.files));
    let shared = export
        .faces
        .iter()
        .chain(&export.boots)
        .chain(&export.gloves)
        .map(|folder| (&folder.path, &folder.files));
    for (path, files) in players.chain(shared) {
        for set in model_variant_sets(files) {
            messages.push(tool_message(
                Code::KitVariantModelFox,
                Scope::Folder {
                    export_id,
                    path: path.clone(),
                },
                Disposition::Keep,
                vec![("model", set.reference), ("used", set.used)],
            ));
        }
    }
}

/// A roster-mapped player folder as planning builds it: the keys each of its packages is
/// emitted under, one per slot mapping it in slot order, and those slots' player ids, which
/// its portrait and its rows go by (none for a referee, who has no player id).
struct MappedFolder {
    /// The player folder.
    folder: PlayerFolder,
    /// The face, boots and gloves packages, each with its keys.
    packages: [(ModelPackage, Vec<PackageKey>); 3],
    /// The player id of each slot mapping the folder, in slot order.
    player_ids: Vec<u32>,
}

/// The roster-mapped folders of `players`, by `player_folders`, as `MappedFolder`s: of a team
/// export, `team` given, a slot's face keyed by its player id and its boots and gloves by
/// its exclusive id of the team's block; of a refs export, every package of referee slot NN
/// keyed by the slot (`referee0NN`, `k99NN`, `g99NN`).
fn mapped_folders(
    players: Vec<PlayerFolder>,
    roster: &ValidatedRoster,
    team: Option<TeamId>,
) -> Vec<MappedFolder> {
    match (roster, team) {
        (ValidatedRoster::Team(slots), Some(id)) => {
            let model_ids = PlannedModelIds::for_team(id);
            player_folders(players, slots)
                .into_iter()
                .map(|(folder, slots)| MappedFolder {
                    packages: ModelPackage::ALL.map(|package| {
                        let keys = slots
                            .iter()
                            .map(|slot| match package {
                                ModelPackage::Face => PackageKey::Id(slot.player_id(id)),
                                ModelPackage::Boots | ModelPackage::Gloves => {
                                    PackageKey::Id(u32::from(model_ids.exclusive(*slot)))
                                }
                            })
                            .collect();
                        (package, keys)
                    }),
                    player_ids: slots.iter().map(|slot| slot.player_id(id)).collect(),
                    folder,
                })
                .collect()
        }
        (ValidatedRoster::Referees(slots), None) => player_folders(players, slots)
            .into_iter()
            .map(|(folder, slots)| MappedFolder {
                packages: ModelPackage::ALL.map(|package| {
                    (
                        package,
                        slots.iter().copied().map(PackageKey::Referee).collect(),
                    )
                }),
                player_ids: Vec::new(),
                folder,
            })
            .collect(),
        (ValidatedRoster::Team(_), None) | (ValidatedRoster::Referees(_), Some(_)) => {
            unreachable!("validation and identity both read a refs export from its `/refs/` name")
        }
    }
}

/// Every player folder `slots` maps, with the slots mapping it, in slot order, the folders
/// ordered by their first slot. A folder no slot maps is not compiled.
fn player_folders<Slot: Copy>(
    players: Vec<PlayerFolder>,
    slots: &BTreeMap<Slot, PlayerIndex>,
) -> Vec<(PlayerFolder, Vec<Slot>)> {
    let mut mapped: Vec<(PlayerIndex, Vec<Slot>)> = Vec::new();
    for (slot, index) in slots {
        match mapped.iter_mut().find(|(known, _)| known == index) {
            Some((_, folder_slots)) => folder_slots.push(*slot),
            None => mapped.push((*index, vec![*slot])),
        }
    }
    let mut players: Vec<Option<PlayerFolder>> = players.into_iter().map(Some).collect();
    mapped
        .into_iter()
        .filter_map(|(index, folder_slots)| {
            let folder = players.get_mut(index.0)?.take()?;
            Some((folder, folder_slots))
        })
        .collect()
}

/// The files among `common`, the export's `Common/` files, that the pre-Fox Common models task
/// reads (`TaskKind::CommonModels`), in `common`'s order: every `.model` and `.mtl`, every FMDL
/// no `.model` of its stem beats (`selected_common_model`), which the task converts, and the
/// `.skl` of each such FMDL's stem, its bind pose. A beaten FMDL and a `.skl` no converted FMDL
/// pairs are ignored, as a player folder's are.
fn common_model_files(common: &[FileDescriptor]) -> Vec<FileDescriptor> {
    let converted = |file: &FileDescriptor| {
        selected_common_model(common, file.path.name(), Engine::PreFox)
            .is_some_and(|selected| selected.path == file.path)
    };
    // The stems, folded, of the FMDLs converted: a `.skl` of one of them is its bind pose.
    let converted_stems: BTreeSet<String> = common
        .iter()
        .filter(|file| file.kind == FileKind::Model(ModelFormat::Fmdl) && converted(file))
        .map(|file| vtree::fold_name(file_stem(file.path.name())))
        .collect();
    common
        .iter()
        .filter(|file| match file.kind {
            FileKind::Model(ModelFormat::PesModel) | FileKind::Mtl => true,
            FileKind::Model(ModelFormat::Fmdl) => converted(file),
            FileKind::Skl => {
                converted_stems.contains(&vtree::fold_name(file_stem(file.path.name())))
            }
            FileKind::Model(ModelFormat::Gltf)
            | FileKind::Texture
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::SharedLink(_)
            | FileKind::CommonLink
            | FileKind::Marker(_)
            | FileKind::Metadata(_)
            | FileKind::Other => false,
        })
        .cloned()
        .collect()
}

/// The player folder's `.common` model links resolved against `common`, the export's `Common/`
/// files, exactly as validation resolved them (a file directly in `Common/`, matched by
/// case-folded name), each with the Common model it loads on a target of `engine`
/// (`selected_common_model`: a model of the linked stem in the target's own format beats the
/// other one). On Fox each link to an FMDL or a `.model` (`PlayerFile::CommonModel`), with,
/// when the link's role has a skeleton slot, the Common `.skl` of the model's stem; on
/// pre-Fox each link that is a part of an `ingame_face` player's boots or gloves
/// (`PlayerFile::PreFoxPart`), an FMDL with the Common `.skl` of its stem, its bind pose, and
/// a `.model` with none, carrying its own. On both engines a `.model` is given the `.mtl` its
/// search finds (`mtl_for`), which a pre-Fox package writes and a Fox one converts it with,
/// and an FMDL none, its materials travelling inside it on Fox and its conversion writing its
/// material set on pre-Fox. A pre-Fox link without the marker stays a link the face's
/// `face.xml` names (`PlayerFile::PreFoxCommonModel`) and is not resolved here. Validation
/// drops a folder whose link names no Common file, so every link here resolves.
fn common_models(
    folder: &PlayerFolder,
    common: &[FileDescriptor],
    engine: Engine,
) -> Vec<CommonModel> {
    let models = FolderModels::of_player(folder, engine);
    folder
        .files
        .iter()
        .filter_map(|file| {
            let slot = match player_file(&folder.path, file, &models)? {
                PlayerFile::CommonModel { package, name } => skeleton_slot(package, name),
                // Only a marked folder gives a link a part's role. Its skeleton has no slot:
                // only a converted FMDL reads one, its bind pose (below).
                PlayerFile::PreFoxPart { .. } if file.kind == FileKind::CommonLink => None,
                PlayerFile::Model { .. }
                | PlayerFile::PreFoxPart { .. }
                | PlayerFile::Packed { .. }
                | PlayerFile::FaceDiffXml
                | PlayerFile::UnusedFaceFile
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::Texture(..)
                | PlayerFile::CommonTexture(_)
                | PlayerFile::PreFoxModel { .. }
                | PlayerFile::Material
                | PlayerFile::PreFoxCommonModel { .. }
                | PlayerFile::CommonMaterial
                | PlayerFile::FaceXml
                | PlayerFile::ConversionSkeleton
                | PlayerFile::UnsupportedGltf => return None,
            };
            let linked = common_link_name(file.path.name())
                .expect("a model link's role implies a `.common` link name");
            let model = selected_common_model(common, &linked, engine)
                .expect("validation drops a player folder whose link names no Common file");
            let fmdl = model.kind == FileKind::Model(ModelFormat::Fmdl);
            let skeleton = match engine {
                Engine::Fox => slot.and_then(|_| common_skeleton(common, &linked)),
                Engine::PreFox if fmdl => common_skeleton(common, &linked),
                Engine::PreFox => None,
            };
            let material = (!fmdl).then(|| {
                mtl_for(&file.path, &folder.path, &folder.files, common).expect(
                    "the deep pass drops a folder holding a link to a `.model` no `.mtl` is \
                     found for (`model_material_undefined`)",
                )
            });
            Some(CommonModel {
                link: file.path.clone(),
                model: model.clone(),
                skeleton: skeleton.cloned(),
                material: material.cloned(),
            })
        })
        .collect()
}

/// The task compiling `kind`, charged the bytes of the files it reads. A sum past `usize` (a
/// 32-bit host only) is over any memory cap, and so is the saturated value.
fn task(export_id: ExportId, team_id: u16, kind: TaskKind) -> BuildTask {
    let size: u64 = kind.files().iter().map(|file| file.size).sum();
    BuildTask {
        export_id,
        team_id,
        charge: usize::try_from(size).unwrap_or(usize::MAX),
        kind,
        group: None,
    }
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::{resolved, resolved_with_issues, to_plan, two_team_colors};

    /// `keys` as a list (`[71405, 71407]`, `[referee 1, referee 20]`).
    fn keys_text(keys: &[PackageKey]) -> String {
        let keys: Vec<String> = keys
            .iter()
            .map(|key| match key {
                PackageKey::Id(id) => id.to_string(),
                PackageKey::Referee(slot) => format!("referee {}", slot.get()),
            })
            .collect();
        format!("[{}]", keys.join(", "))
    }

    /// Each task as one line: export, team, what it compiles, charge.
    fn summary(report: &PlanReport) -> Vec<String> {
        report
            .manifest
            .tasks
            .iter()
            .map(|task| {
                let what = match &task.kind {
                    TaskKind::Models {
                        folder,
                        package,
                        ids,
                    } => format!("{package:?} {} {}", folder.path.as_str(), keys_text(ids)),
                    TaskKind::Textures { folder, .. } => {
                        format!("textures {}", folder.path.as_str())
                    }
                    TaskKind::CommonTextures {
                        folder, textures, ..
                    } => {
                        format!("common textures {} ({})", folder.as_str(), textures.len())
                    }
                    TaskKind::CommonModels { folder, files, .. } => {
                        format!("common models {} ({})", folder.as_str(), files.len())
                    }
                    TaskKind::Portrait { player_id, file } => {
                        format!("portrait {player_id} {}", file.path.as_str())
                    }
                    TaskKind::Kit { slot, kit, .. } => {
                        format!("kit {} {}", slot.as_str(), kit.path.as_str())
                    }
                    TaskKind::Logo { logo } => format!("logo {}", logo.main.file.path.as_str()),
                    TaskKind::RefereeMarker { marker } => {
                        format!("referee marker {}", marker.path.as_str())
                    }
                    TaskKind::Collar { file, id } => {
                        format!("collar {id} {}", file.path.as_str())
                    }
                };
                format!(
                    "{} {} {what} charge {}",
                    task.export_id.0, task.team_id, task.charge
                )
            })
            .collect()
    }

    #[test]
    fn tasks_go_by_export_then_faces_by_first_slot_then_kits_by_slot() {
        let first = resolved(
            "dbg Midcup Two",
            &[
                ("Players/Zed/face_high.fmdl", 10),
                ("Players/Zed/face_diff.bin", 0),
                ("Players/Zed/hair.dds", 5),
                ("Players/Amy/face_high.fmdl", 20),
                ("Players/Amy/face_diff.bin", 0),
                ("Kits/g1/kit.dds", 7),
                ("Kits/g1/config.toml", 3),
                ("Kits/p2 - Away/kit.dds", 8),
            ],
            &["Kits/p1"],
            // Amy is listed first in the file but holds the later slot.
            Some(b"07 Amy\n03 Zed\n09 Amy\n"),
        );
        let second = resolved(
            "co Midcup One",
            &[
                ("Players/04 - B/face_high.fmdl", 1),
                ("Players/04 - B/face_diff.bin", 0),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![
                to_plan(ExportId(0), first, two_team_colors(), None),
                to_plan(ExportId(1), second, two_team_colors(), None),
            ],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 790 Face Players/Zed [79003] charge 10",
                "0 790 textures Players/Zed charge 5",
                "0 790 Face Players/Amy [79007, 79009] charge 20",
                "0 790 kit p1 Kits/p1 charge 0",
                "0 790 kit p2 Kits/p2 - Away charge 8",
                "0 790 kit g1 Kits/g1 charge 10",
                "1 714 Face Players/04 - B [71404] charge 1",
            ]
        );
        // Zed's two tasks are one group charged as one; Amy, with no textures, is ungrouped.
        let groups: Vec<Option<TaskGroup>> = report
            .manifest
            .tasks
            .iter()
            .map(|task| task.group.clone())
            .collect();
        let zed = Some(TaskGroup {
            tasks: 0..2,
            packages: vec![ModelPackage::Face],
            charge: 15,
        });
        assert_eq!(groups, [zed.clone(), zed, None, None, None, None, None]);
    }

    #[test]
    fn an_export_s_logo_is_one_task_after_its_kits_reading_both_files() {
        let export = resolved(
            "co Midcup Logo",
            &[
                ("logo_small_crop.dds", 9),
                ("Players/04 - B/face_high.fmdl", 1),
                ("Players/04 - B/face_diff.bin", 0),
                ("Kits/p1/kit.dds", 8),
                ("logo.png", 40),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert!(
            report
                .messages
                .iter()
                .all(|message| message.code.code != "content_not_yet_compiled"),
            "{:?}",
            report.messages
        );
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/04 - B [71404] charge 1",
                "0 714 kit p1 Kits/p1 charge 8",
                "0 714 logo logo.png charge 49",
            ]
        );
        let logo = &report.manifest.tasks[2];
        assert_eq!(logo.group, None);
        assert_eq!(logo.kind.folder_path(), scope_path("logo.png"));
        let files: Vec<&str> = logo
            .kind
            .files()
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        assert_eq!(files, ["logo.png", "logo_small_crop.dds"]);
    }

    #[test]
    fn a_folder_s_packages_go_face_boots_gloves_then_its_textures_under_the_planned_ids() {
        let export = resolved(
            "co Midcup Models",
            &[
                ("Players/A/glove_l.fmdl", 4),
                ("Players/A/kit_boots.skl", 2),
                ("Players/A/kit_boots.fmdl", 8),
                ("Players/A/shirt.dds", 16),
                ("Players/A/face_high.fmdl", 32),
                ("Players/A/face_high.skl", 128),
                ("Players/A/face_diff.bin", 1),
                ("Players/23 - B/boots.fmdl", 64),
            ],
            &[],
            Some(b"05 A\n07 A\n23 23 - B\n"),
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // Slots 05 and 07 of team 714 own the exclusive ids 625 and 627; slot 23 owns 643.
        // `face_high.skl` has no slot: no task reads it, so no charge counts it. Slot 23
        // holds no face model: its face is the blank one, reading nothing.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/A [71405, 71407] charge 33",
                "0 714 Boots Players/A [625, 627] charge 10",
                "0 714 Gloves Players/A [625, 627] charge 4",
                "0 714 textures Players/A charge 16",
                "0 714 Face Players/23 - B [71423] charge 0",
                "0 714 Boots Players/23 - B [643] charge 64",
            ]
        );
        let group = Some(TaskGroup {
            tasks: 0..4,
            packages: ModelPackage::ALL.to_vec(),
            charge: 63,
        });
        for index in 0..4 {
            assert_eq!(report.manifest.tasks[index].group, group, "task {index}");
        }
        assert_eq!(report.manifest.tasks[4].group, None);
        assert_eq!(report.manifest.tasks[5].group, None);
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.name())
                .collect()
        };
        assert_eq!(files(0), ["face_diff.bin", "face_high.fmdl"]);
        assert_eq!(files(1), ["kit_boots.fmdl", "kit_boots.skl"]);
        assert_eq!(files(2), ["glove_l.fmdl"]);
        assert_eq!(files(3), ["shirt.dds"]);
        for index in 0..4 {
            assert_eq!(
                report.manifest.tasks[index].kind.folder_path(),
                scope_path("Players/A"),
                "task {index}"
            );
        }
    }

    #[test]
    fn per_kit_models_plan_the_lowest_variant_alone_and_report_the_set_once() {
        let export = resolved(
            "co Midcup Variants",
            &[
                ("Players/05 - A/pants_kit2.fmdl", 16),
                ("Players/05 - A/pants_kit1.fmdl", 8),
                ("Players/05 - A/face_diff.bin", 1),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        let [message] = report.messages.as_slice() else {
            panic!("{:?}", report.messages);
        };
        assert_eq!(message.code.code, "kit_variant_model_fox");
        assert_eq!(
            (message.severity, message.disposition),
            (Severity::Warning, Disposition::Keep)
        );
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(0),
                path: scope_path("Players/05 - A"),
            }
        );
        assert_eq!(
            message.context,
            [
                ("model".to_owned(), "pants_kitN.fmdl".to_owned()),
                ("used".to_owned(), "pants_kit1.fmdl".to_owned())
            ]
        );
        // `pants_kit2.fmdl` is read by no task, so no charge counts it.
        assert_eq!(
            summary(&report),
            ["0 714 Face Players/05 - A [71405] charge 9"]
        );
        let files: Vec<&str> = report.manifest.tasks[0]
            .kind
            .files()
            .iter()
            .map(|file| file.path.name())
            .collect();
        assert_eq!(files, ["face_diff.bin", "pants_kit1.fmdl"]);

        // A variant alone is an ordinary model, reported by nothing.
        let alone = resolved(
            "co Midcup Alone",
            &[("Players/05 - A/pants_kit2.fmdl", 16)],
            &[],
            None,
        );
        let report = plan_run(
            vec![to_plan(ExportId(0), alone, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            ["0 714 Face Players/05 - A [71405] charge 16"]
        );

        // On a pre-Fox target the face converts and packs the whole set, listed once for the
        // game to respell: nothing to warn about, both variants read.
        let export = resolved(
            "co Midcup Variants",
            &[
                ("Players/05 - A/pants_kit2.fmdl", 16),
                ("Players/05 - A/pants_kit1.fmdl", 8),
            ],
            &[],
            None,
        );
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes17,
        );
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        let files: Vec<&str> = report.manifest.tasks[0]
            .kind
            .files()
            .iter()
            .map(|file| file.path.name())
            .collect();
        assert_eq!(files, ["pants_kit1.fmdl", "pants_kit2.fmdl"]);
    }

    #[test]
    fn the_textures_tasks_carry_the_export_s_player_kit_numbers() {
        let with_kits = resolved(
            "co Midcup Kits",
            &[
                ("Players/05 - A/face_high.fmdl", 1),
                ("Players/05 - A/pants_kit1.dds", 1),
                ("Common/tape_kit1.dds", 1),
                ("Kits/g1/kit.dds", 1),
                ("Kits/p3/kit.dds", 1),
            ],
            &["Kits/p1"],
            None,
        );
        let without_kits = resolved(
            "co Midcup Plain",
            &[
                ("Players/05 - A/face_high.fmdl", 1),
                ("Players/05 - A/pants_kit1.dds", 1),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![
                to_plan(ExportId(0), with_kits, two_team_colors(), None),
                to_plan(ExportId(1), without_kits, two_team_colors(), None),
            ],
            PesVersion::Pes21,
        );

        // The goalkeeper's `g1` is not a number of its own.
        let kits: Vec<(String, &[u8])> = report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Textures { folder, kits } => Some((
                    format!("{} {}", task.export_id.0, folder.path.as_str()),
                    kits.as_slice(),
                )),
                TaskKind::CommonTextures { folder, kits, .. } => Some((
                    format!("{} {}", task.export_id.0, folder.as_str()),
                    kits.as_slice(),
                )),
                TaskKind::Models { .. }
                | TaskKind::CommonModels { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
            .collect();
        assert_eq!(
            kits,
            [
                ("0 Players/05 - A".to_owned(), &[1, 3][..]),
                ("0 Common".to_owned(), &[1, 3][..]),
                ("1 Players/05 - A".to_owned(), &[][..]),
            ]
        );
    }

    #[test]
    fn shared_folders_follow_the_players_boots_then_gloves_under_the_shared_ids_in_name_order() {
        let export = resolved(
            "co Midcup Shared",
            &[
                ("Players/03 - A/Zebra.boots", 0),
                ("Players/03 - A/Grip.gloves", 0),
                ("Players/07 - B/face_high.fmdl", 32),
                ("Players/07 - B/face_diff.bin", 1),
                ("Players/07 - B/apple.boots", 0),
                ("Players/11 - C/Zebra.boots", 0),
                ("Boots/Zebra/boots.fmdl", 8),
                ("Boots/Zebra/boots.skl", 2),
                ("Boots/Zebra/shirt.dds", 16),
                ("Boots/apple/kit_boots.fmdl", 4),
                ("Gloves/Grip/glove_l.fmdl", 64),
                ("Gloves/Grip/grip.dds", 128),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // Slots 03 and 11 wear Zebra and 07 apple, so no player has a boots package; team
        // 714's shared ids start at 644, in case-folded name order. 03 and 11 hold no face
        // model: theirs are blank.
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/03 - A [71403] charge 0",
                "0 714 Face Players/07 - B [71407] charge 33",
                "0 714 Face Players/11 - C [71411] charge 0",
                "0 714 Boots Boots/apple [644] charge 4",
                "0 714 Boots Boots/Zebra [645] charge 10",
                "0 714 textures Boots/Zebra charge 16",
                "0 714 Gloves Gloves/Grip [644] charge 64",
                "0 714 textures Gloves/Grip charge 128",
            ]
        );
        let groups: Vec<Option<TaskGroup>> = report
            .manifest
            .tasks
            .iter()
            .map(|task| task.group.clone())
            .collect();
        let zebra = Some(TaskGroup {
            tasks: 4..6,
            packages: vec![ModelPackage::Boots],
            charge: 26,
        });
        let grip = Some(TaskGroup {
            tasks: 6..8,
            packages: vec![ModelPackage::Gloves],
            charge: 192,
        });
        assert_eq!(
            groups,
            [
                None,
                None,
                None,
                None,
                zebra.clone(),
                zebra,
                grip.clone(),
                grip
            ]
        );
        let homes: Vec<&TextureHome> = report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Models { folder, .. } | TaskKind::Textures { folder, .. } => {
                    Some(&folder.textures)
                }
                TaskKind::CommonTextures { .. }
                | TaskKind::CommonModels { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
            .collect();
        assert_eq!(
            homes,
            [
                &TextureHome::PlayerCommon {
                    folder_name: "03 - A".to_owned()
                },
                &TextureHome::PlayerCommon {
                    folder_name: "07 - B".to_owned()
                },
                &TextureHome::PlayerCommon {
                    folder_name: "11 - C".to_owned()
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Boots,
                    id: 644
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Boots,
                    id: 645
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Boots,
                    id: 645
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Gloves,
                    id: 644
                },
                &TextureHome::SharedOutput {
                    package: ModelPackage::Gloves,
                    id: 644
                },
            ]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert!(files(0).is_empty());
        assert_eq!(
            files(4),
            ["Boots/Zebra/boots.fmdl", "Boots/Zebra/boots.skl"]
        );
        assert_eq!(files(5), ["Boots/Zebra/shirt.dds"]);
        assert_eq!(
            report.manifest.tasks[4].kind.folder_path(),
            scope_path("Boots/Zebra")
        );
    }

    #[test]
    fn a_selected_gltf_drops_its_player_folder_at_planning_with_its_textures() {
        let files = [
            ("Players/05 - A/boots.glb", 4),
            ("Players/05 - A/boots.model", 8),
            ("Players/05 - A/skin.png", 2),
            ("Players/07 - B/boots.fmdl", 16),
        ];
        let export = resolved("co Midcup Gltf", &files, &[], None);

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // Slot 05's folder plans nothing: no face 71405, no boots 625, no textures; the
        // `.model` the glTF beats is not converted in its place.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/07 - B [71407] charge 0",
                "0 714 Boots Players/07 - B [627] charge 16",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [(
                "model_gltf_unsupported",
                "Players/05 - A",
                Disposition::DropFolder
            )]
        );
        let message = &report.messages[0];
        assert_eq!(message.severity, Severity::Error);
        assert_eq!(
            message.context,
            [("file".to_owned(), "boots.glb".to_owned())]
        );

        // On pre-Fox the glTF beats the FMDL beside it the same way, and each selected glTF
        // is reported; a glTF a `.model` of its stem beats is ignored.
        let files = [
            ("Players/05 - A/boots.fmdl", 8),
            ("Players/05 - A/boots.glb", 4),
            ("Players/05 - A/face/hat.gltf", 4),
            ("Players/07 - B/boots.glb", 4),
            ("Players/07 - B/boots.model", 16),
        ];
        let export = resolved("co Midcup Gltf", &files, &[], None);
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes17,
        );
        let files: Vec<&str> = report
            .messages
            .iter()
            .map(|message| message.context[0].1.as_str())
            .collect();
        assert_eq!(files, ["boots.glb", "face/hat.gltf"]);
        assert_eq!(
            message_summary(&report),
            [
                (
                    "model_gltf_unsupported",
                    "Players/05 - A",
                    Disposition::DropFolder
                ),
                (
                    "model_gltf_unsupported",
                    "Players/05 - A",
                    Disposition::DropFolder
                ),
            ]
        );
        let folders: Vec<ScopePath> = report
            .manifest
            .tasks
            .iter()
            .map(|task| task.kind.folder_path())
            .collect();
        assert!(
            !folders.is_empty()
                && folders
                    .iter()
                    .all(|folder| folder.as_str() == "Players/07 - B"),
            "{folders:?}"
        );

        // A referee's folder leaves the referee roster the same way.
        let referees = resolved(
            "refs Cup",
            &[
                ("Players/Keeper/boots.glb", 4),
                ("Players/Ref B/boots.fmdl", 16),
            ],
            &[],
            Some(b"01 Keeper\n02 Ref B\n"),
        );
        let report = plan_run(
            vec![to_plan(ExportId(0), referees, None, None)],
            PesVersion::Pes21,
        );
        assert_eq!(
            message_summary(&report),
            [(
                "model_gltf_unsupported",
                "Players/Keeper",
                Disposition::DropFolder
            )]
        );
        assert_eq!(
            summary(&report),
            [
                "0 999 Face Players/Ref B [referee 2] charge 0",
                "0 999 Boots Players/Ref B [referee 2] charge 16",
            ]
        );
    }

    #[test]
    fn a_shared_folder_s_selected_gltf_drops_it_with_every_player_linking_it() {
        let files = [
            ("Players/05 - A/Crocs.boots", 0),
            // Dropped for his own glTF, which is the one reported.
            ("Players/06 - B/Crocs.boots", 0),
            ("Players/06 - B/face/hat.gltf", 4),
            ("Players/07 - C/Round.face", 0),
            ("Players/09 - D/Mud.boots", 0),
            ("Boots/Crocs/boots.glb", 4),
            ("Boots/Mud/boots.glb", 4),
            ("Faces/Round/hat.glb", 4),
        ];
        // A model of the target's format beats Mud's glTF of its stem: Mud compiles.
        for (version, mud) in [
            (PesVersion::Pes21, &[("Boots/Mud/boots.fmdl", 16)][..]),
            (
                PesVersion::Pes17,
                &[("Boots/Mud/boots.model", 16), ("Boots/Mud/boots.mtl", 1)][..],
            ),
        ] {
            let export = resolved("co Midcup Gltf", &[&files[..], mud].concat(), &[], None);

            let mut kept = export.clone();
            let mut messages = Vec::new();
            drop_gltf_folders(ExportId(0), &mut kept, version, &mut messages);
            let names = |folders: &[SharedModelFolder]| -> Vec<String> {
                folders
                    .iter()
                    .map(|folder| folder.folder_name.clone())
                    .collect()
            };
            assert_eq!(names(&kept.export.boots), ["Mud"], "{version}");
            assert_eq!(names(&kept.export.faces), Vec::<String>::new(), "{version}");

            let report = plan_run(
                vec![to_plan(ExportId(0), export, two_team_colors(), None)],
                version,
            );

            assert_eq!(
                message_summary(&report),
                [
                    (
                        "model_gltf_unsupported",
                        "Players/05 - A",
                        Disposition::DropFolder
                    ),
                    (
                        "model_gltf_unsupported",
                        "Players/06 - B",
                        Disposition::DropFolder
                    ),
                    (
                        "model_gltf_unsupported",
                        "Players/07 - C",
                        Disposition::DropFolder
                    ),
                ],
                "{version}"
            );
            // A shared file by its export path, the folder's own below the folder.
            let files: Vec<&str> = report
                .messages
                .iter()
                .map(|message| message.context[0].1.as_str())
                .collect();
            assert_eq!(
                files,
                [
                    "Boots/Crocs/boots.glb",
                    "face/hat.gltf",
                    "Faces/Round/hat.glb"
                ],
                "{version}"
            );
            // Only slot 09 and Mud plan tasks.
            let folders: BTreeSet<String> = report
                .manifest
                .tasks
                .iter()
                .map(|task| task.kind.folder_path().as_str().to_owned())
                .collect();
            assert_eq!(
                folders,
                BTreeSet::from(["Boots/Mud".to_owned(), "Players/09 - D".to_owned()]),
                "{version}"
            );
        }
    }

    #[test]
    fn a_shared_folder_the_gltf_drop_orphans_is_removed_and_reports_nothing() {
        let files = [
            ("Players/05 - A/boots.glb", 4),
            ("Players/05 - A/Round.face", 0),
            ("Players/07 - C/face_high.fmdl", 8),
            ("Faces/Round/pants_kit1.fmdl", 8),
            ("Faces/Round/pants_kit2.fmdl", 16),
        ];
        let export = resolved("co Midcup Orphan", &files, &[], None);

        let mut kept = export.clone();
        drop_gltf_folders(ExportId(0), &mut kept, PesVersion::Pes21, &mut Vec::new());
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            message_summary(&report),
            [(
                "model_gltf_unsupported",
                "Players/05 - A",
                Disposition::DropFolder
            )]
        );
        assert!(kept.export.faces.is_empty(), "{:?}", kept.export.faces);
        assert_eq!(
            summary(&report),
            ["0 714 Face Players/07 - C [71407] charge 8"]
        );

        // Slot 07 linking it too keeps the folder, whose set is reported once.
        let linked = [&files[..], &[("Players/07 - C/Round.face", 0)]].concat();
        let export = resolved("co Midcup Orphan", &linked, &[], None);
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            message_summary(&report),
            [
                (
                    "model_gltf_unsupported",
                    "Players/05 - A",
                    Disposition::DropFolder
                ),
                ("kit_variant_model_fox", "Faces/Round", Disposition::Keep),
                ("link_combined", "Players/07 - C", Disposition::Keep),
            ]
        );
        let variants: Vec<&Message> = report
            .messages
            .iter()
            .filter(|message| message.code.code == "kit_variant_model_fox")
            .collect();
        assert_eq!(
            variants[0].context,
            [
                ("model".to_owned(), "pants_kitN.fmdl".to_owned()),
                ("used".to_owned(), "pants_kit1.fmdl".to_owned())
            ]
        );
    }

    #[test]
    fn a_link_beside_a_local_model_combines_the_shared_folder_into_the_player_s_package() {
        let files = [
            ("Players/05 - A/Crocs.boots", 0),
            ("Players/05 - A/kit_boots.fmdl", 8),
            ("Players/05 - A/kit_boots.skl", 2),
            ("Players/05 - A/Grip.gloves", 0),
            ("Players/05 - A/glove_l.fmdl", 4),
            ("Boots/Crocs/boots.fmdl", 16),
            ("Boots/Crocs/boots.skl", 1),
            ("Boots/Crocs/sole.dds", 32),
            ("Gloves/Grip/glove_r.fmdl", 64),
        ];
        let export = resolved("co Midcup Combined", &files, &[], None);

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // The shared folders are only sources of parts: no shared id, no output of their own.
        // The player holds no face model: its face is blank.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 0",
                "0 714 Boots Players/05 - A [625] charge 27",
                "0 714 Gloves Players/05 - A [625] charge 68",
                "0 714 textures Players/05 - A charge 32",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [
                ("link_combined", "Players/05 - A", Disposition::Keep),
                ("link_combined", "Players/05 - A", Disposition::Keep),
            ]
        );
        let links: Vec<&str> = report
            .messages
            .iter()
            .map(|message| message.context[0].1.as_str())
            .collect();
        assert_eq!(links, ["Crocs.boots", "Grip.gloves"]);
        assert!(
            report
                .messages
                .iter()
                .all(|message| message.context[0].0 == "link"),
            "{:?}",
            report.messages
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        // The player's own files first, then each combined folder's, each skeleton pairing
        // with the model of its stem in its own folder.
        assert!(files(0).is_empty());
        assert_eq!(
            files(1),
            [
                "Players/05 - A/kit_boots.fmdl",
                "Players/05 - A/kit_boots.skl",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/boots.skl",
            ]
        );
        assert_eq!(
            files(2),
            ["Players/05 - A/glove_l.fmdl", "Gloves/Grip/glove_r.fmdl"]
        );
        assert_eq!(files(3), ["Boots/Crocs/sole.dds"]);
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[1].kind else {
            panic!("a package task");
        };
        let combined: Vec<(ModelPackage, &str)> = folder
            .combined
            .iter()
            .map(|shared| (shared.package, shared.folder.path.as_str()))
            .collect();
        assert_eq!(
            combined,
            [
                (ModelPackage::Boots, "Boots/Crocs"),
                (ModelPackage::Gloves, "Gloves/Grip")
            ]
        );
        assert_eq!(
            folder.textures,
            TextureHome::PlayerCommon {
                folder_name: "05 - A".to_owned()
            }
        );
        let group = Some(TaskGroup {
            tasks: 0..4,
            packages: ModelPackage::ALL.to_vec(),
            charge: 127,
        });
        for index in 0..4 {
            assert_eq!(report.manifest.tasks[index].group, group, "task {index}");
        }
    }

    #[test]
    fn a_reserved_subfolder_s_parts_join_their_category_s_task_and_a_link_of_it_combines() {
        let export = resolved(
            "co Midcup Subfolders",
            &[
                ("Players/05 - A/boots/boots.fmdl", 8),
                ("Players/05 - A/boots/boots.skl", 2),
                ("Players/05 - A/common/skin.dds", 16),
                ("Players/05 - A/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 4),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // The boots alone: the face of a folder whose only model is in `boots/` is blank.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 0",
                "0 714 Boots Players/05 - A [625] charge 14",
                "0 714 textures Players/05 - A charge 16",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert!(files(0).is_empty());
        assert_eq!(
            files(1),
            [
                "Players/05 - A/boots/boots.fmdl",
                "Players/05 - A/boots/boots.skl",
                "Boots/Crocs/boots.fmdl",
            ]
        );
        assert_eq!(files(2), ["Players/05 - A/common/skin.dds"]);
    }

    #[test]
    fn a_common_link_s_task_reads_the_common_model_and_skeleton_and_the_common_textures_are_one_task()
     {
        let export = resolved(
            "co Midcup Common",
            &[
                ("Players/05 - A/torso.fmdl", 8),
                ("Players/05 - A/legs.fmdl.common", 0),
                ("Players/05 - A/boots/Kit_Boots.fmdl.common", 0),
                ("Players/05 - A/face_high.fmdl.common", 0),
                ("Players/05 - A/skin.dds", 4),
                ("Players/07 - B/legs.fmdl.common", 0),
                ("Players/07 - B/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 2),
                ("Common/Legs.fmdl", 16),
                ("Common/legs.skl", 1),
                ("Common/kit_boots.fmdl", 32),
                ("Common/kit_boots.skl", 64),
                ("Common/face_high.fmdl", 128),
                ("Common/face_high.skl", 256),
                ("Common/spare.fmdl", 512),
                ("Common/Cloth.dds", 1024),
                ("Common/hair.ftex", 2048),
                ("Portraits/player_05.dds", 3),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // Slot 05's face reads the local part, the two Common models and the hair's skeleton,
        // never the links nor the slotless `face_high.skl`; its boots read Common's model and
        // skeleton. The Common textures follow the shared folders and precede the portraits,
        // and the unlinked `spare.fmdl` is read by nothing.
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 153",
                "0 714 Boots Players/05 - A [625] charge 96",
                "0 714 textures Players/05 - A charge 4",
                "0 714 Face Players/07 - B [71407] charge 17",
                "0 714 Boots Boots/Crocs [644] charge 2",
                "0 714 common textures Common (2) charge 3072",
                "0 714 portrait 71405 Portraits/player_05.dds charge 3",
            ]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(
            files(0),
            [
                "Common/face_high.fmdl",
                "Common/Legs.fmdl",
                "Common/legs.skl",
                "Players/05 - A/torso.fmdl",
            ]
        );
        assert_eq!(files(1), ["Common/kit_boots.fmdl", "Common/kit_boots.skl"]);
        assert_eq!(files(3), ["Common/Legs.fmdl", "Common/legs.skl"]);
        assert_eq!(files(5), ["Common/Cloth.dds", "Common/hair.ftex"]);
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[0].kind else {
            panic!("a package task");
        };
        let links: Vec<(&str, &str, Option<&str>)> = folder
            .common_models
            .iter()
            .map(|common| {
                (
                    common.link.as_str(),
                    common.model.path.as_str(),
                    common.skeleton.as_ref().map(|file| file.path.as_str()),
                )
            })
            .collect();
        assert_eq!(
            links,
            [
                (
                    "Players/05 - A/boots/Kit_Boots.fmdl.common",
                    "Common/kit_boots.fmdl",
                    Some("Common/kit_boots.skl")
                ),
                (
                    "Players/05 - A/face_high.fmdl.common",
                    "Common/face_high.fmdl",
                    None
                ),
                (
                    "Players/05 - A/legs.fmdl.common",
                    "Common/Legs.fmdl",
                    Some("Common/legs.skl")
                ),
            ]
        );
        assert_eq!(
            folder.common_texture_stems,
            BTreeSet::from(["cloth".to_owned(), "hair".to_owned()])
        );
        // The Common task is in no group: a player's packages never wait for it.
        let common = &report.manifest.tasks[5];
        assert_eq!(common.group, None);
        assert_eq!(common.kind.folder_path(), scope_path("Common"));
        assert_eq!(
            report.manifest.tasks[2].group,
            Some(TaskGroup {
                tasks: 0..3,
                packages: vec![ModelPackage::Face, ModelPackage::Boots],
                charge: 253,
            })
        );
    }

    #[test]
    fn a_texture_link_is_read_by_no_task_of_its_folder_and_its_texture_by_the_common_task() {
        let export = resolved(
            "co Midcup Links",
            &[
                ("Players/05 - A/face_high.fmdl", 8),
                ("Players/05 - A/skin.dds", 4),
                ("Players/05 - A/hair.dds.common", 0),
                ("Players/07 - B/face_high.fmdl", 16),
                ("Players/07 - B/hair.dds.common", 0),
                ("Players/07 - B/boots/sole.png.common", 0),
                ("Common/hair.dds", 32),
                ("Common/sole.png", 64),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // Slot 05's textures task reads its own `skin.dds` alone; slot 07, holding a model
        // and texture links only, has no textures task.
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 8",
                "0 714 textures Players/05 - A charge 4",
                "0 714 Face Players/07 - B [71407] charge 16",
                "0 714 common textures Common (2) charge 96",
            ]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(files(0), ["Players/05 - A/face_high.fmdl"]);
        assert_eq!(files(1), ["Players/05 - A/skin.dds"]);
        assert_eq!(files(2), ["Players/07 - B/face_high.fmdl"]);
        assert_eq!(files(3), ["Common/hair.dds", "Common/sole.png"]);
        assert_eq!(report.manifest.tasks[2].group, None);
    }

    #[test]
    fn a_face_link_alone_makes_the_shared_face_the_player_s_and_the_player_s_face_files_win() {
        let export = resolved(
            "co Midcup Faces",
            &[
                ("Players/05 - A/Longhair.face", 0),
                ("Players/05 - A/face_diff.bin", 3),
                ("Players/05 - A/skin.dds", 5),
                ("Faces/Longhair/hair_high.fmdl", 16),
                ("Faces/Longhair/face_diff.bin", 7),
                ("Faces/Longhair/hair.dds", 32),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // The face task exists for the shared model alone, and is charged the player's
        // `face_diff.bin`, not the shared folder's, which is never read.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 19",
                "0 714 textures Players/05 - A charge 37",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        assert_eq!(report.messages[0].context[0].1, "Longhair.face");
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(
            files(0),
            [
                "Players/05 - A/face_diff.bin",
                "Faces/Longhair/hair_high.fmdl"
            ]
        );
        assert_eq!(
            files(1),
            ["Players/05 - A/skin.dds", "Faces/Longhair/hair.dds"]
        );
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[0].kind else {
            panic!("a package task");
        };
        assert_eq!(folder.combined[0].package, ModelPackage::Face);
        let group = Some(TaskGroup {
            tasks: 0..2,
            packages: vec![ModelPackage::Face],
            charge: 56,
        });
        assert_eq!(report.manifest.tasks[0].group, group);
    }

    /// The files the face task of `co Midcup Faces` reads, its player folder `Players/05 - A`
    /// linking `Faces/Longhair` and holding `player_files`, the shared folder holding
    /// `hair_high.fmdl` and `shared_files`.
    fn face_task_files(player_files: &[&str], shared_files: &[&str]) -> Vec<String> {
        let mut files = vec![
            ("Players/05 - A/Longhair.face".to_owned(), 0),
            ("Faces/Longhair/hair_high.fmdl".to_owned(), 16),
        ];
        files.extend(
            player_files
                .iter()
                .map(|name| (format!("Players/05 - A/{name}"), 3)),
        );
        files.extend(
            shared_files
                .iter()
                .map(|name| (format!("Faces/Longhair/{name}"), 7)),
        );
        let files: Vec<(&str, u64)> = files
            .iter()
            .map(|(path, size)| (path.as_str(), *size))
            .collect();
        let export = resolved("co Midcup Faces", &files, &[], None);
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        assert_eq!(
            report
                .messages
                .iter()
                .map(|message| message.code.code.as_ref())
                .collect::<Vec<&str>>(),
            ["link_combined"]
        );
        report.manifest.tasks[0]
            .kind
            .files()
            .iter()
            .map(|file| file.path.as_str().to_owned())
            .collect()
    }

    #[test]
    fn a_face_diff_xml_stands_for_the_face_diff_bin_and_a_second_copy_is_left_out() {
        // The player's own face diff wins over the shared folder's, whichever form each has,
        // and the shared one is never read.
        assert_eq!(
            face_task_files(&["face_diff.xml"], &["face_diff.bin"]),
            [
                "Players/05 - A/face_diff.xml",
                "Faces/Longhair/hair_high.fmdl"
            ]
        );
        assert_eq!(
            face_task_files(&["face_diff.bin"], &["face_diff.xml"]),
            [
                "Players/05 - A/face_diff.bin",
                "Faces/Longhair/hair_high.fmdl"
            ]
        );
        // Two copies of one form, one in `face/`, are one file: the first in the folder's
        // order is kept.
        assert_eq!(
            face_task_files(&["face_diff.xml", "face/face_diff.xml"], &[]),
            [
                "Players/05 - A/face/face_diff.xml",
                "Faces/Longhair/hair_high.fmdl"
            ]
        );
    }

    #[test]
    fn a_combined_shared_folder_still_compiles_on_its_own_for_a_player_linking_it_plainly() {
        let export = resolved(
            "co Midcup Combined",
            &[
                ("Players/05 - A/Crocs.boots", 0),
                ("Players/05 - A/kit_boots.fmdl", 8),
                ("Players/07 - B/Crocs.boots", 0),
                ("Boots/Crocs/boots.fmdl", 16),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 0",
                "0 714 Boots Players/05 - A [625] charge 24",
                "0 714 Face Players/07 - B [71407] charge 0",
                "0 714 Boots Boots/Crocs [644] charge 16",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[3].kind else {
            panic!("a package task");
        };
        assert!(folder.combined.is_empty(), "the shared folder's own task");
    }

    #[test]
    fn a_marked_folder_plans_no_face_and_every_other_folder_a_face_blank_without_a_face_model() {
        let export = resolved(
            "co Midcup Faces",
            &[
                ("Players/05 - A/ingame_face", 0),
                ("Players/05 - A/torso.fmdl", 8),
                ("Players/05 - A/glove_l.fmdl", 4),
                ("Players/07 - B/boots.fmdl", 16),
            ],
            &["Players/09 - C/face"],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 8",
                "0 714 Gloves Players/05 - A [625] charge 4",
                "0 714 Face Players/07 - B [71407] charge 0",
                "0 714 Boots Players/07 - B [627] charge 16",
                "0 714 Face Players/09 - C [71409] charge 0",
            ]
        );
        let files = |index: usize| -> Vec<&str> {
            report.manifest.tasks[index]
                .kind
                .files()
                .iter()
                .map(|file| file.path.as_str())
                .collect()
        };
        assert_eq!(files(0), ["Players/05 - A/torso.fmdl"]);
        assert!(files(2).is_empty());
        assert!(files(4).is_empty());
    }

    #[test]
    fn under_ingame_face_a_boots_link_beside_a_model_the_face_would_take_combines() {
        let plan = |files: &[(&str, u64)]| {
            let export = resolved("co Midcup Marked", files, &[], None);
            plan_run(
                vec![to_plan(ExportId(0), export, two_team_colors(), None)],
                PesVersion::Pes21,
            )
        };
        let shared = ("Boots/Crocs/boots.fmdl", 16);

        // The rerouted model is a boots part, so the link combines and Crocs takes no id.
        let report = plan(&[
            ("Players/05 - A/ingame_face", 0),
            ("Players/05 - A/torso.fmdl", 8),
            ("Players/05 - A/Crocs.boots", 0),
            shared,
        ]);
        assert_eq!(
            summary(&report),
            ["0 714 Boots Players/05 - A [625] charge 24"]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );

        // The link alone loads Crocs as it is, under its shared id; the player has no task.
        let report = plan(&[
            ("Players/05 - A/ingame_face", 0),
            ("Players/05 - A/Crocs.boots", 0),
            shared,
        ]);
        assert_eq!(
            summary(&report),
            ["0 714 Boots Boots/Crocs [644] charge 16"]
        );
        assert!(report.messages.is_empty(), "{:?}", report.messages);
    }

    #[test]
    fn portraits_follow_the_faces_by_player_id_one_per_slot_of_their_folder() {
        let export = resolved(
            "dbg Midcup Portraits",
            &[
                ("Players/Zed/face_high.fmdl", 10),
                ("Players/Zed/face_diff.bin", 0),
                ("Players/Zed/portrait.dds", 4),
                ("Portraits/player_05.dds", 6),
                ("Portraits/player_01.dds", 5),
                ("Kits/g1/kit.dds", 7),
            ],
            &[],
            Some(b"07 Zed\n03 Zed\n"),
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 790 Face Players/Zed [79003, 79007] charge 10",
                "0 790 portrait 79001 Portraits/player_01.dds charge 5",
                "0 790 portrait 79003 Players/Zed/portrait.dds charge 4",
                "0 790 portrait 79005 Portraits/player_05.dds charge 6",
                "0 790 portrait 79007 Players/Zed/portrait.dds charge 4",
                "0 790 kit g1 Kits/g1 charge 7",
            ]
        );
        let portrait = &report.manifest.tasks[2].kind;
        assert_eq!(
            portrait.folder_path(),
            scope_path("Players/Zed/portrait.dds")
        );
        let files: Vec<&str> = portrait
            .files()
            .iter()
            .map(|file| file.path.as_str())
            .collect();
        assert_eq!(files, ["Players/Zed/portrait.dds"]);
    }

    #[test]
    fn a_player_id_with_a_portrait_from_both_sources_gets_one_task_over_the_folder_s_file() {
        // Slot 07's `Portraits/` file beside the folder slots 03 and 07 map: the deep pass
        // has found the two identical, so the folder's file is the one portrait.
        let export = resolved(
            "dbg Midcup Portraits",
            &[
                ("Players/Zed/face_high.fmdl", 10),
                ("Players/Zed/face_diff.bin", 0),
                ("Players/Zed/portrait.dds", 4),
                ("Portraits/player_07.dds", 4),
                ("Portraits/player_05.dds", 6),
            ],
            &[],
            Some(b"07 Zed\n03 Zed\n"),
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 790 Face Players/Zed [79003, 79007] charge 10",
                "0 790 portrait 79003 Players/Zed/portrait.dds charge 4",
                "0 790 portrait 79005 Portraits/player_05.dds charge 6",
                "0 790 portrait 79007 Players/Zed/portrait.dds charge 4",
            ]
        );
    }

    #[test]
    fn a_kit_without_config_toml_reports_its_generated_config() {
        let export = resolved(
            "co Midcup Kits",
            &[("Kits/p1/config.toml", 3), ("Kits/p2 - Away/kit.dds", 8)],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(3), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        let messages: Vec<(&str, &Scope, Disposition)> = report
            .messages
            .iter()
            .map(|message| {
                (
                    message.code.code.as_ref(),
                    &message.scope,
                    message.disposition,
                )
            })
            .collect();
        let scope = Scope::Folder {
            export_id: ExportId(3),
            path: ScopePath::new("Kits/p2 - Away").unwrap(),
        };
        // p1 holds only its config, so it is a placeholder kit.
        let p1 = Scope::Folder {
            export_id: ExportId(3),
            path: ScopePath::new("Kits/p1").unwrap(),
        };
        assert_eq!(
            messages,
            [
                ("kit_placeholder", &p1, Disposition::Keep),
                ("kit_config_generated", &scope, Disposition::Keep),
            ]
        );
        assert_eq!(
            report.manifest.tasks[1].kind.folder_path(),
            scope_path("Kits/p2 - Away")
        );
        assert_eq!(
            report.manifest.tasks[0].kind.folder_path(),
            scope_path("Kits/p1")
        );
    }

    /// Each planning message as (code, scope path, disposition).
    fn message_summary(report: &PlanReport) -> Vec<(&str, &str, Disposition)> {
        report
            .messages
            .iter()
            .map(|message| {
                let Scope::Folder { path, .. } = &message.scope else {
                    panic!("{:?}", message.scope);
                };
                (
                    message.code.code.as_ref(),
                    path.as_str(),
                    message.disposition,
                )
            })
            .collect()
    }

    #[test]
    fn a_kit_without_a_main_texture_reports_the_placeholder_after_its_config() {
        let export = resolved(
            "co Midcup Kits",
            &[
                ("Kits/p1/kit.dds", 8),
                ("Kits/p3/kit_back.dds", 8),
                ("Kits/p4/config.toml", 3),
            ],
            &["Kits/p2"],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            message_summary(&report),
            [
                ("kit_config_generated", "Kits/p1", Disposition::Keep),
                ("kit_config_generated", "Kits/p2", Disposition::Keep),
                ("kit_placeholder", "Kits/p2", Disposition::Keep),
                ("kit_config_generated", "Kits/p3", Disposition::Keep),
                ("kit_placeholder", "Kits/p3", Disposition::Keep),
                ("kit_placeholder", "Kits/p4", Disposition::Keep),
            ]
        );
        assert!(
            report
                .messages
                .iter()
                .all(|message| message.context.is_empty()),
            "{:?}",
            report.messages
        );
    }

    #[test]
    fn a_kit_inheriting_the_main_texture_from_all_is_no_placeholder() {
        // The inheritance finding is validation's, not planning's.
        let (export, issues) = resolved_with_issues(
            "co Midcup Kits",
            &[("Kits/all/kit.dds", 8), ("Kits/p1/config.toml", 3)],
            &[],
            None,
        );
        assert_eq!(issues, ["kit_textures_inherited"]);

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        let [task] = report.manifest.tasks.as_slice() else {
            panic!("{}", report.manifest.tasks.len());
        };
        let TaskKind::Kit { kit, .. } = &task.kind else {
            panic!("a kit task");
        };
        let stems: Vec<&str> = kit
            .textures
            .iter()
            .map(|texture| texture.stem.as_str())
            .collect();
        assert_eq!(stems, ["kit"]);
    }

    /// The team kit-FPC status each kit task of the export `co Midcup Fpc` carries, the export
    /// holding the kits `p1` and `g1`, `players` (path, size) and the roster `players_txt`, and
    /// validation reporting exactly the codes `issues` on it.
    fn kit_fpc(
        players: &[(&str, u64)],
        players_txt: Option<&[u8]>,
        issues: &[&str],
    ) -> Vec<EffectiveTeamKitFpc> {
        let files: Vec<(&str, u64)> = [("Kits/p1/kit.dds", 1), ("Kits/g1/kit.dds", 1)]
            .into_iter()
            .chain(players.iter().copied())
            .collect();
        let (export, codes_found) = resolved_with_issues("co Midcup Fpc", &files, &[], players_txt);
        assert_eq!(codes_found, issues, "validation's issues");
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        assert!(
            codes(&report)
                .iter()
                .all(|code| *code != "content_not_yet_compiled"),
            "{:?}",
            report.messages
        );
        report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Kit { edits, .. } => Some(edits.fpc),
                TaskKind::Models { .. }
                | TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::CommonModels { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
            .collect()
    }

    #[test]
    fn one_fpc_on_folder_among_others_makes_every_kit_of_the_team_fpc() {
        use EffectiveTeamKitFpc::{On, Unknown};
        let a = ("Players/03 - A/face_high.fmdl", 1);
        let b = ("Players/05 - B/face_high.fmdl", 1);
        let c = ("Players/07 - C/face_high.fmdl", 1);
        let on = ("Players/05 - B/fpc_on", 0);
        let off = ("Players/07 - C/fpc_off", 0);

        assert_eq!(kit_fpc(&[a, b, on, c, off], None, &[]), [On, On]);
        assert_eq!(kit_fpc(&[a, b, c, off], None, &[]), [Unknown, Unknown]);
        assert_eq!(kit_fpc(&[a, b, c], None, &[]), [Unknown, Unknown]);
        // A folder validation dropped (no roster line maps it) compiles no player.
        assert_eq!(
            kit_fpc(&[a, b, on], Some(b"03 03 - A\n"), &["player_unlisted"]),
            [Unknown, Unknown]
        );
    }

    #[test]
    fn a_face_task_names_its_player_folder() {
        let export = resolved(
            "co Midcup One",
            &[
                ("Players/04 - B/face_high.fmdl", 1),
                ("Players/04 - B/face_diff.bin", 0),
            ],
            &[],
            None,
        );
        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        assert_eq!(
            report.manifest.tasks[0].kind.folder_path(),
            scope_path("Players/04 - B")
        );
    }

    #[test]
    fn an_export_the_subset_gate_refuses_plans_no_task_and_reports_why() {
        // A referee has no kit slot, so a refs export's kit is named.
        let referees = resolved(
            "refs Cup",
            &[
                ("Players/Keeper/face_high.fmdl", 1),
                ("Players/Keeper/face_diff.bin", 0),
                ("Kits/p1/kit.dds", 1),
            ],
            &[],
            Some(b"01 Keeper\n"),
        );
        let kit = resolved("co Midcup Kit", &[("Kits/g1/kit.dds", 1)], &[], None);

        let report = plan_run(
            vec![
                to_plan(ExportId(2), referees, None, None),
                to_plan(ExportId(3), kit, two_team_colors(), None),
            ],
            PesVersion::Pes21,
        );

        assert_eq!(summary(&report), ["3 714 kit g1 Kits/g1 charge 1"]);
        assert_eq!(report.manifest.tasks[0].kind.files().len(), 1);
        let [skipped, generated] = report.messages.as_slice() else {
            panic!("{:?}", report.messages);
        };
        assert_eq!(skipped.code.code, "content_not_yet_compiled");
        assert_eq!(
            (skipped.severity, skipped.disposition),
            (Severity::Error, Disposition::DropExport)
        );
        assert_eq!(
            skipped.scope,
            Scope::Export {
                export_id: ExportId(2)
            }
        );
        assert_eq!(skipped.context, [("what".to_owned(), "Kits/p1".to_owned())]);
        assert_eq!(generated.code.code, "kit_config_generated");
    }

    /// Each planning message's code, on the export or one of its folders.
    fn codes(report: &PlanReport) -> Vec<&str> {
        report
            .messages
            .iter()
            .map(|message| message.code.code.as_ref())
            .collect()
    }

    #[test]
    fn a_team_s_colors_go_into_the_manifest_and_a_team_without_them_reports_it() {
        let kit = || resolved("co Midcup Kit", &[("Kits/p1/kit.dds", 1)], &[], None);
        let colors = vec![[0xc1, 0x12, 0x00], [0x41, 0x41, 0x41]];

        let with = plan_run(
            vec![to_plan(ExportId(0), kit(), Some(colors.clone()), None)],
            PesVersion::Pes21,
        );
        assert_eq!(with.manifest.team_colors, [(714, colors.clone())]);
        assert_eq!(codes(&with), ["kit_config_generated"]);

        let without = plan_run(
            vec![to_plan(ExportId(4), kit(), None, None)],
            PesVersion::Pes21,
        );
        assert_eq!(without.manifest.team_colors, []);
        let [missing, generated] = without.messages.as_slice() else {
            panic!("{:?}", without.messages);
        };
        assert_eq!(missing.code.code, "team_colors_missing");
        assert_eq!(
            (missing.severity, missing.disposition),
            (Severity::Info, Disposition::Keep)
        );
        assert_eq!(
            missing.scope,
            Scope::Export {
                export_id: ExportId(4)
            }
        );
        assert!(missing.context.is_empty());
        assert_eq!(generated.code.code, "kit_config_generated");

        // A file whose every line was refused: the deep pass reported the lines.
        let refused_lines = plan_run(
            vec![to_plan(ExportId(0), kit(), Some(Vec::new()), None)],
            PesVersion::Pes21,
        );
        assert_eq!(refused_lines.manifest.team_colors, []);
        assert_eq!(codes(&refused_lines), ["kit_config_generated"]);

        // Two teams' colors, in export order.
        let dbg = resolved("dbg Midcup Kit", &[("Kits/p1/kit.dds", 1)], &[], None);
        let both = plan_run(
            vec![
                to_plan(ExportId(0), dbg, Some(vec![[1, 2, 3]]), None),
                to_plan(ExportId(1), kit(), Some(colors.clone()), None),
            ],
            PesVersion::Pes21,
        );
        assert_eq!(
            both.manifest.team_colors,
            [(790, vec![[1, 2, 3]]), (714, colors)]
        );
    }

    #[test]
    fn the_notes_of_the_planned_exports_go_into_the_manifest_in_export_order() {
        let kit = |name| resolved(name, &[("Kits/p1/kit.dds", 1)], &[], None);
        let referees = resolved(
            "refs Cup",
            &[
                ("Players/Keeper/face_high.fmdl", 1),
                ("Players/Keeper/face_diff.bin", 0),
            ],
            &[],
            Some(b"01 Keeper\n"),
        );
        let note = |text: &str| Some(text.to_owned());

        let report = plan_run(
            vec![
                to_plan(ExportId(0), kit("dbg Midcup Kit"), None, note("dbg's note")),
                to_plan(ExportId(1), kit("co Midcup Plain"), None, None),
                to_plan(ExportId(2), referees, None, note("the referees' note")),
                to_plan(ExportId(3), kit("co Midcup Kit"), None, note("co's note")),
            ],
            PesVersion::Pes21,
        );

        assert_eq!(
            report.manifest.notes,
            [
                ("/dbg/".to_owned(), "dbg's note".to_owned()),
                ("/refs/".to_owned(), "the referees' note".to_owned()),
                ("/co/".to_owned(), "co's note".to_owned())
            ]
        );
    }

    fn referee(slot: u8) -> PackageKey {
        PackageKey::Referee(aesthetics_export::RefSlot::new(slot).unwrap())
    }

    /// The keys of `report`'s `Models` tasks, in manifest order.
    fn model_keys(report: &PlanReport) -> Vec<Vec<PackageKey>> {
        report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Models { ids, .. } => Some(ids.clone()),
                TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::CommonModels { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
            .collect()
    }

    #[test]
    fn a_referee_folder_is_prepared_once_and_emitted_under_each_of_his_slots() {
        let referees = resolved(
            "refs Cup",
            &[
                ("Players/Ref A/face_high.fmdl", 10),
                ("Players/Ref A/boots.fmdl", 20),
                ("Players/Ref A/skin.dds", 5),
            ],
            &[],
            Some(b"20 Ref A\n01 Ref A\n35 Ref A\n"),
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), referees, None, None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 999 Face Players/Ref A [referee 1, referee 20, referee 35] charge 10",
                "0 999 Boots Players/Ref A [referee 1, referee 20, referee 35] charge 20",
                "0 999 textures Players/Ref A charge 5",
            ]
        );
        let slots = vec![referee(1), referee(20), referee(35)];
        assert_eq!(model_keys(&report), [slots.clone(), slots]);
        let TaskKind::Textures { folder, .. } = &report.manifest.tasks[2].kind else {
            panic!("the third task is the folder's textures");
        };
        assert_eq!(
            folder.textures,
            TextureHome::PlayerCommon {
                folder_name: "Ref A".to_owned()
            }
        );
        assert!(report.messages.is_empty(), "{:?}", report.messages);
        assert!(report.manifest.team_colors.is_empty());
        assert!(report.manifest.team_kits.is_empty());
        assert!(report.manifest.item_rows.is_empty());
    }

    #[test]
    fn a_referee_s_plain_link_is_a_part_of_his_slots_own_package() {
        let referees = resolved(
            "refs Cup",
            &[
                ("Players/Ref A/Studs.boots", 0),
                ("Boots/Studs/boots.fmdl", 20),
            ],
            &[],
            Some(b"01 Ref A\n20 Ref A\n"),
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), referees, None, None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 999 Face Players/Ref A [referee 1, referee 20] charge 0",
                "0 999 Boots Players/Ref A [referee 1, referee 20] charge 20",
            ],
            "no task of Boots/Studs's own"
        );
        let TaskKind::Models { folder, .. } = &report.manifest.tasks[1].kind else {
            panic!("the second task is the boots");
        };
        assert_eq!(
            folder
                .combined
                .iter()
                .map(|combined| (combined.package, combined.folder.path.as_str()))
                .collect::<Vec<_>>(),
            [(ModelPackage::Boots, "Boots/Studs")]
        );
        assert!(report.messages.is_empty(), "{:?}", report.messages);
    }

    #[test]
    fn a_pre_fox_face_link_combines_and_a_boots_link_loads_the_shared_output_with_no_rows() {
        let export = resolved(
            "co Midcup Shared",
            &[
                ("Players/05 - A/face_high.model", 4),
                ("Players/05 - A/face_high.mtl", 1),
                ("Players/05 - A/Round.face", 0),
                ("Players/06 - B/Crocs.boots", 0),
                ("Faces/Round/hair_high.model", 8),
                ("Faces/Round/hair_high.mtl", 2),
                ("Boots/Crocs/boots.model", 16),
                ("Boots/Crocs/boots.mtl", 1),
                ("Boots/Crocs/crocs.dds", 32),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes17,
        );

        // The shared face is a second source of slot 05's face; the shared boots are their
        // own output under team 714's first shared id, which slot 06 loads by that id.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 15",
                "0 714 Face Players/06 - B [71406] charge 0",
                "0 714 Boots Boots/Crocs [644] charge 17",
                "0 714 textures Boots/Crocs charge 32",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [("link_combined", "Players/05 - A", Disposition::Keep)]
        );
        assert_eq!(
            report.messages[0].context,
            [("link".to_owned(), "Round.face".to_owned())]
        );
        let combined = |index: usize| -> Vec<(ModelPackage, &str)> {
            models_folder(&report.manifest.tasks[index])
                .combined
                .iter()
                .map(|shared| (shared.package, shared.folder.path.as_str()))
                .collect()
        };
        assert_eq!(combined(0), [(ModelPackage::Face, "Faces/Round")]);
        assert_eq!(combined(1), []);
        assert_eq!(
            task_files(&report.manifest.tasks[2]),
            ["Boots/Crocs/boots.model", "Boots/Crocs/boots.mtl"]
        );
        // PES 15-17 have no player tables to point a player at his boots.
        assert_eq!(report.manifest.item_rows, []);
    }

    #[test]
    fn a_pre_fox_face_reads_the_skeleton_of_each_fmdl_it_converts_and_not_the_fclo() {
        let export = resolved(
            "co Midcup Convert",
            &[
                ("Players/05 - A/face_diff.bin", 1),
                ("Players/05 - A/fcl_hair.fmdl", 4),
                ("Players/05 - A/fcl_hair.skl", 2),
                ("Players/05 - A/fcl_hair_sim.fclo", 8),
                ("Players/05 - A/shirt.dds", 16),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes17,
        );

        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 7",
                "0 714 textures Players/05 - A charge 16",
            ]
        );
        assert_eq!(
            task_files(&report.manifest.tasks[0]),
            [
                "Players/05 - A/face_diff.bin",
                "Players/05 - A/fcl_hair.fmdl",
                "Players/05 - A/fcl_hair.skl",
            ]
        );
    }

    #[test]
    fn a_fox_package_converting_a_model_reads_the_folder_s_mtl_files_and_one_of_fmdls_none() {
        let export = resolved(
            "co Midcup Convert",
            &[
                ("Players/05 - A/boots.model", 4),
                ("Players/05 - A/boots.mtl", 1),
                ("Players/05 - A/materials.mtl", 2),
                // The face holds only FMDLs: the `.model` of the hair's stem is beaten.
                ("Players/05 - A/fcl_hair.fmdl", 8),
                ("Players/05 - A/fcl_hair.model", 16),
                ("Players/05 - A/shirt.dds", 32),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 8",
                "0 714 Boots Players/05 - A [625] charge 7",
                "0 714 textures Players/05 - A charge 32",
            ]
        );
        assert_eq!(
            task_files(&report.manifest.tasks[1]),
            [
                "Players/05 - A/boots.model",
                "Players/05 - A/boots.mtl",
                "Players/05 - A/materials.mtl",
            ]
        );
    }

    #[test]
    fn a_pre_fox_marked_player_s_mtl_files_are_read_by_his_boots_and_his_gloves() {
        let export = resolved(
            "co Midcup Marked",
            &[
                ("Players/05 - A/ingame_face", 0),
                ("Players/05 - A/kit_boots.model", 4),
                ("Players/05 - A/x_gloveL.model", 2),
                ("Players/05 - A/materials.mtl", 1),
                ("Players/05 - A/Keeper.gloves", 0),
                ("Players/07 - B/ingame_face", 0),
                ("Players/07 - B/x_gloveR.model", 2),
                ("Players/07 - B/x_gloveR.mtl", 1),
                ("Players/07 - B/Crocs.boots", 0),
                ("Players/07 - B/Keeper.gloves", 0),
                ("Gloves/Keeper/glove_r.model", 8),
                ("Gloves/Keeper/glove_r.mtl", 1),
                ("Boots/Crocs/boots.model", 16),
                ("Boots/Crocs/boots.mtl", 1),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes17,
        );

        // Each marked player's boots and gloves under his exclusive id, a package for each
        // kind he holds a part of: slot 07 has no boots part, so his Crocs link loads the
        // shared output, and Keeper, which both combine, takes no id.
        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 5",
                "0 714 Gloves Players/05 - A [625] charge 12",
                "0 714 Gloves Players/07 - B [627] charge 12",
                "0 714 Boots Boots/Crocs [644] charge 17",
            ]
        );
        assert_eq!(
            message_summary(&report),
            [
                ("link_combined", "Players/05 - A", Disposition::Keep),
                ("link_combined", "Players/07 - B", Disposition::Keep),
            ]
        );
        // His `.mtl` is a file of both; Keeper's goes with Keeper's model, into the gloves.
        assert_eq!(
            task_files(&report.manifest.tasks[0]),
            [
                "Players/05 - A/kit_boots.model",
                "Players/05 - A/materials.mtl"
            ]
        );
        assert_eq!(
            task_files(&report.manifest.tasks[1]),
            [
                "Players/05 - A/materials.mtl",
                "Players/05 - A/x_gloveL.model",
                "Gloves/Keeper/glove_r.model",
                "Gloves/Keeper/glove_r.mtl",
            ]
        );
    }

    /// The PES 17 plan of slot 05 holding `ingame_face`, a boots and a gloves FMDL each with
    /// its skeleton, and linking `Gloves/Keeper/` (an FMDL with its skeleton), which his
    /// gloves part combines; and of slot 07 linking `Boots/Crocs/` (an FMDL with its
    /// skeleton) plainly. `metal_models` are the deep pass's metal models.
    fn converting_plan(metal_models: &[&str]) -> PlanReport {
        let export = resolved(
            "co Midcup Convert",
            &[
                ("Players/05 - A/ingame_face", 0),
                ("Players/05 - A/boots.fmdl", 4),
                ("Players/05 - A/boots.skl", 1),
                ("Players/05 - A/x_gloveL.fmdl", 2),
                ("Players/05 - A/x_gloveL.skl", 1),
                ("Players/05 - A/Keeper.gloves", 0),
                ("Players/07 - B/Crocs.boots", 0),
                ("Gloves/Keeper/glove_r.fmdl", 8),
                ("Gloves/Keeper/glove_r.skl", 1),
                ("Boots/Crocs/boots.fmdl", 16),
                ("Boots/Crocs/boots.skl", 1),
            ],
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        planned.metal_models = metal_models.iter().map(|path| scope_path(path)).collect();
        plan_run(vec![planned], PesVersion::Pes17)
    }

    #[test]
    fn a_pre_fox_boots_or_gloves_package_reads_the_skeleton_of_each_fmdl_it_may_convert() {
        let report = converting_plan(&[]);

        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 6",
                "0 714 Gloves Players/05 - A [625] charge 13",
                "0 714 Face Players/07 - B [71407] charge 0",
                "0 714 Boots Boots/Crocs [644] charge 17",
            ]
        );
        // His own skeletons go into both his packages, as his `.mtl` files do, each of which
        // converts the FMDL one pairs with; a combined folder's into his package of its kind,
        // and a shared folder's into its own output.
        let tasks = &report.manifest.tasks;
        assert_eq!(
            task_files(&tasks[0]),
            [
                "Players/05 - A/boots.fmdl",
                "Players/05 - A/boots.skl",
                "Players/05 - A/x_gloveL.skl",
            ]
        );
        assert_eq!(
            task_files(&tasks[1]),
            [
                "Players/05 - A/boots.skl",
                "Players/05 - A/x_gloveL.fmdl",
                "Players/05 - A/x_gloveL.skl",
                "Gloves/Keeper/glove_r.fmdl",
                "Gloves/Keeper/glove_r.skl",
            ]
        );
        assert_eq!(
            task_files(&tasks[3]),
            ["Boots/Crocs/boots.fmdl", "Boots/Crocs/boots.skl"]
        );
    }

    #[test]
    fn a_marked_player_s_part_or_a_shared_folder_s_metal_fmdl_gets_the_template_in_its_home() {
        let owned = |package: &str| package.to_owned();
        // His gloves part and the shared boots: each folder gets the template, a textures task
        // planned for it with no texture of its own.
        let report = converting_plan(&["Players/05 - A/x_gloveL.fmdl", "Boots/Crocs/boots.fmdl"]);
        assert_eq!(
            environment_maps(&report),
            [
                (owned("Boots"), "Players/05 - A", true),
                (owned("Gloves"), "Players/05 - A", true),
                (owned("textures"), "Players/05 - A", true),
                (owned("Face"), "Players/07 - B", false),
                (owned("Boots"), "Boots/Crocs", true),
                (owned("textures"), "Boots/Crocs", true),
            ]
        );
        // A combined folder's metal FMDL is one his gloves convert.
        let report = converting_plan(&["Gloves/Keeper/glove_r.fmdl"]);
        assert_eq!(
            environment_maps(&report)[..3],
            [
                (owned("Boots"), "Players/05 - A", true),
                (owned("Gloves"), "Players/05 - A", true),
                (owned("textures"), "Players/05 - A", true),
            ]
        );
        // With none, no folder gets one.
        let report = converting_plan(&[]);
        assert!(
            environment_maps(&report)
                .iter()
                .all(|(package, _, environment_map)| package != "textures" && !environment_map),
            "{:?}",
            environment_maps(&report)
        );
    }

    #[test]
    fn a_pre_fox_marked_player_s_common_links_are_parts_read_with_the_mtl_their_search_finds() {
        let export = resolved(
            "co Midcup Common",
            &[
                ("Players/05 - A/ingame_face", 0),
                // Two spellings of one link: one part, its files read once.
                ("Players/05 - A/kit_boots.model.common", 0),
                ("Players/05 - A/kit_boots.model.common.txt", 0),
                // A link whose `.mtl` is his own override of the Common one.
                ("Players/05 - A/glove_l.model.common", 0),
                ("Players/05 - A/glove_l.mtl", 1),
                // A model of his whose `.mtl` is a Common one, through two link spellings.
                ("Players/05 - A/glove_r.model", 2),
                ("Players/05 - A/glove_r.mtl.common", 0),
                ("Players/05 - A/glove_r.mtl.common.txt", 0),
                ("Common/kit_boots.model", 4),
                ("Common/kit_boots.mtl", 8),
                ("Common/glove_l.model", 16),
                ("Common/glove_l.mtl", 32),
                ("Common/glove_r.mtl", 64),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes17,
        );

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        // Each package reads its parts and every `.mtl` of his, the Common ones his parts'
        // searches find included; the Common models task reads all of `Common/` again.
        assert_eq!(
            summary(&report),
            [
                "0 714 Boots Players/05 - A [625] charge 77",
                "0 714 Gloves Players/05 - A [625] charge 91",
                "0 714 common models Common (5) charge 124",
            ]
        );
        let tasks = &report.manifest.tasks;
        assert_eq!(
            task_files(&tasks[0]),
            [
                "Players/05 - A/glove_l.mtl",
                "Common/glove_r.mtl",
                "Common/kit_boots.model",
                "Common/kit_boots.mtl",
            ]
        );
        assert_eq!(
            task_files(&tasks[1]),
            [
                "Common/glove_l.model",
                "Players/05 - A/glove_l.mtl",
                "Players/05 - A/glove_r.model",
                "Common/glove_r.mtl",
                "Common/kit_boots.mtl",
            ]
        );
        let folder = models_folder(&tasks[0]);
        let links: Vec<(&str, &str, Option<&str>)> = folder
            .common_models
            .iter()
            .map(|common| {
                assert_eq!(common.skeleton, None, "a `.model` carries its own skeleton");
                (
                    common.link.as_str(),
                    common.model.path.as_str(),
                    common.material.as_ref().map(|file| file.path.as_str()),
                )
            })
            .collect();
        assert_eq!(
            links,
            [
                (
                    "Players/05 - A/glove_l.model.common",
                    "Common/glove_l.model",
                    Some("Players/05 - A/glove_l.mtl")
                ),
                (
                    "Players/05 - A/kit_boots.model.common",
                    "Common/kit_boots.model",
                    Some("Common/kit_boots.mtl")
                ),
                (
                    "Players/05 - A/kit_boots.model.common.txt",
                    "Common/kit_boots.model",
                    Some("Common/kit_boots.mtl")
                ),
            ]
        );
        // The Common model in the link's place under the link's role, each Common `.mtl`
        // once, before the `.mtl` links, which are kept for the search to see.
        let (_, _, roles) = folder.roles().swap_remove(0);
        let roles: Vec<(&str, PlayerFile)> = roles
            .into_iter()
            .map(|(file, role)| (file.path.as_str(), role))
            .collect();
        let part = |package, xml_type: &str| PlayerFile::PreFoxPart {
            package,
            xml_type: xml_type.to_owned(),
        };
        assert_eq!(
            roles,
            [
                ("Common/glove_l.model", part(ModelPackage::Gloves, "gloveL")),
                ("Players/05 - A/glove_l.mtl", PlayerFile::Material),
                (
                    "Players/05 - A/glove_r.model",
                    part(ModelPackage::Gloves, "gloveR")
                ),
                ("Common/glove_r.mtl", PlayerFile::Material),
                (
                    "Players/05 - A/glove_r.mtl.common",
                    PlayerFile::CommonMaterial
                ),
                (
                    "Players/05 - A/glove_r.mtl.common.txt",
                    PlayerFile::CommonMaterial
                ),
                ("Common/kit_boots.model", part(ModelPackage::Boots, "parts")),
                ("Common/kit_boots.mtl", PlayerFile::Material),
            ]
        );

        // On Fox a link's materials travel inside the FMDL: none is resolved.
        let fox = resolved(
            "co Midcup Fox",
            &[
                ("Players/05 - A/legs.fmdl.common", 0),
                ("Common/legs.fmdl", 4),
            ],
            &[],
            None,
        );
        let report = plan_run(
            vec![to_plan(ExportId(0), fox, two_team_colors(), None)],
            PesVersion::Pes21,
        );
        let folder = models_folder(&report.manifest.tasks[0]);
        assert_eq!(folder.common_models.len(), 1);
        assert_eq!(folder.common_models[0].material, None);
    }

    #[test]
    fn a_fox_link_to_a_common_model_is_converted_with_its_common_mtl_unless_an_fmdl_beats_it() {
        let export = resolved(
            "co Midcup Fox",
            &[
                ("Players/05 - A/legs.model.common", 0),
                // An FMDL of its stem beats the linked `.model`: the link loads the FMDL.
                ("Players/05 - A/hat.model.common", 0),
                ("Common/legs.model", 1),
                ("Common/legs.mtl", 2),
                ("Common/hat.model", 4),
                ("Common/hat.fmdl", 8),
            ],
            &[],
            None,
        );

        let report = plan_run(
            vec![to_plan(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        let task = &report.manifest.tasks[0];
        let folder = models_folder(task);
        let links: Vec<(&str, &str, Option<&str>)> = folder
            .common_models
            .iter()
            .map(|common| {
                (
                    common.link.as_str(),
                    common.model.path.as_str(),
                    common.material.as_ref().map(|file| file.path.as_str()),
                )
            })
            .collect();
        assert_eq!(
            links,
            [
                ("Players/05 - A/hat.model.common", "Common/hat.fmdl", None),
                (
                    "Players/05 - A/legs.model.common",
                    "Common/legs.model",
                    Some("Common/legs.mtl")
                ),
            ]
        );
        // The face converts the Common `.model` with the Common `.mtl`, which follows it as a
        // `.mtl` of the player's, and packs the FMDL as it is.
        let (_, _, roles) = folder.roles().swap_remove(0);
        let roles: Vec<(&str, PlayerFile)> = roles
            .into_iter()
            .map(|(file, role)| (file.path.as_str(), role))
            .collect();
        let hair = || PlayerFile::CommonModel {
            package: ModelPackage::Face,
            name: "fcl_hair",
        };
        assert_eq!(
            roles,
            [
                ("Common/hat.fmdl", hair()),
                ("Common/legs.model", hair()),
                ("Common/legs.mtl", PlayerFile::Material),
            ]
        );
        assert_eq!(
            task_files(task),
            ["Common/hat.fmdl", "Common/legs.model", "Common/legs.mtl"]
        );
    }

    /// The PES 17 plan of an export whose `Common/` holds `legs.fmdl` with `legs.skl`, a
    /// `hat.fmdl` its `hat.model` (with `hat.mtl`) beats with `hat.skl`, a stray `stray.skl`
    /// and `extra`; slot 05 holding `ingame_face` links `legs.fmdl.common` and `hat.fmdl.common`,
    /// slot 06 `legs.fmdl.common`. `metal_models` are the deep pass's metal models.
    fn common_fmdl_plan(extra: &[(&str, u64)], metal_models: &[&str]) -> PlanReport {
        let files = [
            ("Players/05 - A/ingame_face", 0),
            ("Players/05 - A/legs.fmdl.common", 0),
            ("Players/05 - A/hat.fmdl.common", 0),
            ("Players/06 - B/legs.fmdl.common", 0),
            ("Common/legs.fmdl", 1),
            ("Common/legs.skl", 2),
            ("Common/hat.fmdl", 4),
            ("Common/hat.skl", 8),
            ("Common/hat.model", 16),
            ("Common/hat.mtl", 32),
            ("Common/stray.skl", 64),
        ];
        let export = resolved(
            "co Midcup Common",
            &[files.as_slice(), extra].concat(),
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        planned.metal_models = metal_models.iter().map(|path| scope_path(path)).collect();
        plan_run(vec![planned], PesVersion::Pes17)
    }

    /// The `environment_map` flag of the plan's Common textures task, `None` when it has none.
    fn common_environment_map(report: &PlanReport) -> Option<bool> {
        report
            .manifest
            .tasks
            .iter()
            .find_map(|task| match &task.kind {
                TaskKind::CommonTextures {
                    environment_map, ..
                } => Some(*environment_map),
                TaskKind::Models { .. }
                | TaskKind::Textures { .. }
                | TaskKind::CommonModels { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
    }

    #[test]
    fn a_pre_fox_common_fmdl_is_read_by_the_common_models_task_with_its_skeleton() {
        let report = common_fmdl_plan(&[], &[]);

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        // The beaten FMDL, its skeleton and the stray one are read by nothing; no metal model,
        // so no Common textures task.
        let common = report
            .manifest
            .tasks
            .iter()
            .find(|task| matches!(task.kind, TaskKind::CommonModels { .. }))
            .unwrap();
        assert_eq!(
            task_files(common),
            [
                "Common/hat.model",
                "Common/hat.mtl",
                "Common/legs.fmdl",
                "Common/legs.skl",
            ]
        );
        assert_eq!(common_environment_map(&report), None);
        // His FMDL link brings in the Common FMDL with its skeleton, no `.mtl`; his link to the
        // beaten FMDL loads the `.model` with the `.mtl` its search finds.
        let boots = report
            .manifest
            .tasks
            .iter()
            .find(|task| {
                matches!(
                    task.kind,
                    TaskKind::Models {
                        package: ModelPackage::Boots,
                        ..
                    }
                )
            })
            .unwrap();
        let folder = models_folder(boots);
        let links: Vec<(&str, Option<&str>, Option<&str>)> = folder
            .common_models
            .iter()
            .map(|common| {
                (
                    common.model.path.as_str(),
                    common.skeleton.as_ref().map(|file| file.path.as_str()),
                    common.material.as_ref().map(|file| file.path.as_str()),
                )
            })
            .collect();
        assert_eq!(
            links,
            [
                ("Common/hat.model", None, Some("Common/hat.mtl")),
                ("Common/legs.fmdl", Some("Common/legs.skl"), None),
            ]
        );
        assert_eq!(
            task_files(boots),
            [
                "Common/hat.model",
                "Common/hat.mtl",
                "Common/legs.fmdl",
                "Common/legs.skl",
            ]
        );
    }

    #[test]
    fn a_metal_common_fmdl_plans_the_common_template_environment_map_unless_common_has_env() {
        // No texture in `Common/`: the task is planned for the template alone.
        let report = common_fmdl_plan(&[], &["Common/legs.fmdl"]);
        assert_eq!(common_environment_map(&report), Some(true));
        // A beaten FMDL converts nothing.
        let report = common_fmdl_plan(&[], &["Common/hat.fmdl"]);
        assert_eq!(common_environment_map(&report), None);
        // `Common/`'s own `env` texture, in any case, is the map.
        let report = common_fmdl_plan(&[("Common/ENV.png", 1)], &["Common/legs.fmdl"]);
        assert_eq!(common_environment_map(&report), Some(false));
        // On Fox, never.
        let export = resolved(
            "co Midcup Common",
            &[("Common/legs.fmdl", 1), ("Common/shirt.dds", 1)],
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        planned.metal_models = [scope_path("Common/legs.fmdl")].into();
        let report = plan_run(vec![planned], PesVersion::Pes21);
        assert_eq!(common_environment_map(&report), Some(false));
    }

    #[test]
    fn an_export_the_subset_gate_refuses_lists_no_colors_and_reports_none_missing() {
        // A Common glTF, which the PES 17 Common output does not hold.
        let refused = || resolved("co Midcup Common", &[("Common/x.glb", 1)], &[], None);
        let report = plan_run(
            vec![
                to_plan(ExportId(0), refused(), Some(vec![[1, 2, 3]]), None),
                to_plan(ExportId(1), refused(), None, None),
            ],
            PesVersion::Pes17,
        );
        assert_eq!(report.manifest.team_colors, []);
        assert_eq!(
            codes(&report),
            ["content_not_yet_compiled", "content_not_yet_compiled"]
        );
    }

    fn scope_path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    /// The `/co/` export `co Midcup Hands` planned for `version`: slot 05 holds `body.fmdl`, a
    /// face part by its name, and slot 06 `boots.fmdl`, both of which the deep pass found
    /// weighted to hand bones.
    fn hand_weighted_plan(version: PesVersion) -> PlanReport {
        let export = resolved(
            "co Midcup Hands",
            &[
                ("Players/05 - A/body.fmdl", 3),
                ("Players/06 - B/boots.fmdl", 5),
            ],
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        planned.hand_weighted = [
            scope_path("Players/05 - A/body.fmdl"),
            scope_path("Players/06 - B/boots.fmdl"),
        ]
        .into();
        plan_run(vec![planned], version)
    }

    /// The paths of the files `task` reads.
    fn task_files(task: &BuildTask) -> Vec<&str> {
        task.kind
            .files()
            .into_iter()
            .map(|file| file.path.as_str())
            .collect()
    }

    /// The model folder of the `Models` task `task`.
    fn models_folder(task: &BuildTask) -> &ModelFolder {
        let TaskKind::Models { folder, .. } = &task.kind else {
            panic!("a Models task");
        };
        folder
    }

    #[test]
    fn a_hand_weighted_face_part_gives_its_fox_folder_a_gloves_task_reading_it() {
        let report = hand_weighted_plan(PesVersion::Pes21);

        // Slot 05's gloves are planned under its exclusive ID, 625 (block 621 + 5 - 1), with
        // no glove-named file; slot 06's boots are not split, so it has no gloves.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 3",
                "0 714 Gloves Players/05 - A [625] charge 3",
                "0 714 Face Players/06 - B [71406] charge 0",
                "0 714 Boots Players/06 - B [626] charge 5",
            ]
        );
        let tasks = &report.manifest.tasks;
        // Both the face and the gloves read the model; the face reads what it did before.
        assert_eq!(task_files(&tasks[0]), ["Players/05 - A/body.fmdl"]);
        assert_eq!(task_files(&tasks[1]), ["Players/05 - A/body.fmdl"]);
        let body: BTreeSet<ScopePath> = [scope_path("Players/05 - A/body.fmdl")].into();
        assert_eq!(models_folder(&tasks[1]).hand_split, body);
        assert_eq!(models_folder(&tasks[3]).hand_split, BTreeSet::new());
        // The player's `GloveList.bin` row follows the gloves task, as an authored glove's.
        let gloves: Vec<&ItemRow> = report
            .manifest
            .item_rows
            .iter()
            .filter(|row| row.table == crate::bins::player_tables::ItemTable::Gloves)
            .collect();
        assert_eq!(
            gloves,
            [&ItemRow {
                table: crate::bins::player_tables::ItemTable::Gloves,
                player_id: 71405,
                change: item_rows::RowChange::Set { id: 625, task: 1 },
            }]
        );
    }

    #[test]
    fn a_pre_fox_face_model_weighted_to_hand_bones_is_split_by_the_face_alone() {
        let export = resolved(
            "co Midcup Hands",
            &[
                // A face model of his own and one of the shared face he links: both split.
                ("Players/05 - A/body.model", 3),
                ("Players/05 - A/body.mtl", 1),
                ("Players/05 - A/Round.face", 0),
                ("Faces/Round/hair_high.model", 5),
                ("Faces/Round/hair_high.mtl", 1),
                // A Common model behind a link: the face lists it by reference, packs nothing.
                ("Players/06 - B/legs.model.common", 0),
                ("Common/legs.model", 7),
                ("Common/legs.mtl", 1),
                // Under `ingame_face` no model is face content.
                ("Players/07 - C/ingame_face", 0),
                ("Players/07 - C/torso.model", 9),
                ("Players/07 - C/torso.mtl", 1),
                // Models named as gloves or boots, packed in his face beside nothing else.
                ("Players/08 - D/glove_l.model", 2),
                ("Players/08 - D/boots.model", 4),
                ("Players/08 - D/x.mtl", 1),
            ],
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        // Every model weighted to hand bones; the link's own path too, so a link taken by its
        // role would be seen whichever path it were recorded under.
        planned.hand_weighted = [
            "Players/05 - A/body.model",
            "Faces/Round/hair_high.model",
            "Players/06 - B/legs.model.common",
            "Common/legs.model",
            "Players/07 - C/torso.model",
            "Players/08 - D/glove_l.model",
            "Players/08 - D/boots.model",
        ]
        .map(scope_path)
        .into();

        let report = plan_run(vec![planned], PesVersion::Pes17);

        // One face task per face, and no gloves task for the split: its hands are entries of
        // the face's own `face.xml`.
        assert_eq!(
            summary(&report),
            [
                "0 714 Face Players/05 - A [71405] charge 10",
                "0 714 Face Players/06 - B [71406] charge 0",
                "0 714 Boots Players/07 - C [627] charge 10",
                "0 714 Face Players/08 - D [71408] charge 7",
                "0 714 common models Common (2) charge 8",
            ]
        );
        let tasks = &report.manifest.tasks;
        assert_eq!(
            models_folder(&tasks[0]).hand_split,
            ["Players/05 - A/body.model", "Faces/Round/hair_high.model"]
                .map(scope_path)
                .into()
        );
        for task in &tasks[1..4] {
            let folder = models_folder(task);
            assert_eq!(
                folder.hand_split,
                BTreeSet::new(),
                "{}",
                folder.path.as_str()
            );
        }
        // The face reads what it did before the split: its models and their `.mtl` files.
        assert_eq!(
            task_files(&tasks[0]),
            [
                "Players/05 - A/body.model",
                "Players/05 - A/body.mtl",
                "Faces/Round/hair_high.model",
                "Faces/Round/hair_high.mtl",
            ]
        );
    }

    #[test]
    fn a_pre_fox_folder_holding_its_own_face_xml_splits_no_hand_weighted_model() {
        let export = resolved(
            "co Midcup Hands",
            &[
                ("Players/05 - A/body.model", 3),
                ("Players/05 - A/body.mtl", 1),
                ("Players/05 - A/face.xml", 1),
            ],
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        planned.hand_weighted = [scope_path("Players/05 - A/body.model")].into();

        let report = plan_run(vec![planned], PesVersion::Pes17);

        let tasks = &report.manifest.tasks;
        assert_eq!(
            summary(&report),
            ["0 714 Face Players/05 - A [71405] charge 5"]
        );
        // The xml says what the face loads: the face task splits nothing, so it reports no
        // `model_hand_split`, and lists no glove entry the member did not write.
        assert_eq!(models_folder(&tasks[0]).hand_split, BTreeSet::new());
        // The face reads the xml beside the model and its `.mtl`.
        assert_eq!(
            task_files(&tasks[0]),
            [
                "Players/05 - A/body.model",
                "Players/05 - A/body.mtl",
                "Players/05 - A/face.xml",
            ]
        );
    }

    /// The plan for `version` of slot 05 holding `boots.fmdl` with no texture, slot 06
    /// `boots.fmdl` beside `boots.model` and its `.mtl`, which the `.model` beats on PES
    /// 15-17, and slot 07 `boots.fmdl` beside its own `face.xml`; every FMDL is among the deep
    /// pass's metal models.
    fn metal_plan(version: PesVersion) -> PlanReport {
        let export = resolved(
            "co Midcup Metal",
            &[
                ("Players/05 - A/boots.fmdl", 3),
                ("Players/06 - B/boots.fmdl", 3),
                ("Players/06 - B/boots.model", 5),
                ("Players/06 - B/boots.mtl", 1),
                ("Players/07 - C/boots.fmdl", 3),
                ("Players/07 - C/face.xml", 1),
            ],
            &[],
            None,
        );
        let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
        planned.metal_models = [
            scope_path("Players/05 - A/boots.fmdl"),
            scope_path("Players/06 - B/boots.fmdl"),
            scope_path("Players/07 - C/boots.fmdl"),
        ]
        .into();
        plan_run(vec![planned], version)
    }

    /// Each `Models` and `Textures` task of `report` as its folder's path with whether the
    /// folder takes the template environment map.
    fn environment_maps(report: &PlanReport) -> Vec<(String, &str, bool)> {
        report
            .manifest
            .tasks
            .iter()
            .filter_map(|task| match &task.kind {
                TaskKind::Models {
                    folder, package, ..
                } => Some((
                    format!("{package:?}"),
                    folder.path.as_str(),
                    folder.environment_map,
                )),
                TaskKind::Textures { folder, .. } => Some((
                    "textures".to_owned(),
                    folder.path.as_str(),
                    folder.environment_map,
                )),
                TaskKind::CommonTextures { .. }
                | TaskKind::CommonModels { .. }
                | TaskKind::Portrait { .. }
                | TaskKind::Kit { .. }
                | TaskKind::Logo { .. }
                | TaskKind::RefereeMarker { .. }
                | TaskKind::Collar { .. } => None,
            })
            .collect()
    }

    #[test]
    fn a_folder_whose_pre_fox_face_converts_a_metal_fmdl_gets_a_textures_task_for_the_template() {
        let report = metal_plan(PesVersion::Pes17);

        // Slot 05's face converts its metal FMDL: its textures task is planned, with no
        // texture of its own, to emit the template. Slot 06's `.model` beats its FMDL, and
        // slot 07's own `face.xml` lists the face's models, so neither face converts one.
        let owned = |package: &str| package.to_owned();
        assert_eq!(
            environment_maps(&report),
            [
                (owned("Face"), "Players/05 - A", true),
                (owned("textures"), "Players/05 - A", true),
                (owned("Face"), "Players/06 - B", false),
                (owned("Face"), "Players/07 - C", false),
            ]
        );
        let tasks = &report.manifest.tasks;
        assert_eq!(task_files(&tasks[1]), Vec::<&str>::new());
        assert_eq!(tasks[0].group, tasks[1].group);
        assert!(tasks[1].group.is_some());

        // PES 21 converts no FMDL: the same models plan no environment map, nor a textures
        // task for one.
        let report = metal_plan(PesVersion::Pes21);
        assert!(
            environment_maps(&report)
                .iter()
                .all(|(package, _, environment_map)| package != "textures" && !environment_map),
            "{:?}",
            environment_maps(&report)
        );
    }

    #[test]
    fn an_env_texture_link_plans_no_textures_task_for_the_template() {
        let files = [
            ("Players/05 - A/boots.fmdl", 3),
            ("Players/05 - A/env.dds.common", 0),
            ("Common/env.dds", 4),
        ];
        let plan = |files: &[(&str, u64)]| {
            let export = resolved("co Midcup Metal", files, &[], None);
            let mut planned = to_plan(ExportId(0), export, two_team_colors(), None);
            planned.metal_models = [scope_path("Players/05 - A/boots.fmdl")].into();
            plan_run(vec![planned], PesVersion::Pes17)
        };

        // The link's Common texture is the map: a textures task would emit nothing.
        let report = plan(&files);
        assert_eq!(
            environment_maps(&report),
            [("Face".to_owned(), "Players/05 - A", true)]
        );
        assert_eq!(report.manifest.tasks[0].group, None);

        // A texture of his own still plans one, for that texture alone.
        let report = plan(&[&files[..], &[("Players/05 - A/hair.dds", 4)]].concat());
        let tasks = &report.manifest.tasks;
        assert_eq!(
            environment_maps(&report),
            [
                ("Face".to_owned(), "Players/05 - A", true),
                ("textures".to_owned(), "Players/05 - A", true),
            ]
        );
        assert_eq!(task_files(&tasks[1]), ["Players/05 - A/hair.dds"]);
    }
}
