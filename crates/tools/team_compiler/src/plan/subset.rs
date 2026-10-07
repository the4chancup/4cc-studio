//! What `compile` builds so far, and the gate that skips any export holding anything else
//! (`team_compiler/README.md` "Phase 3 scope", narrowed step by step through Phase 4): one
//! classification, which planning reads to skip an export and processing reads for each
//! file's role, so the two never disagree. The gate is withdrawn when `compile` builds
//! everything an export holds; the classification stays.

use aesthetics_export::{
    ExportIdentity, FileDescriptor, FileKind, ModelFormat, ModelSuffix, PlayerFolder,
    ResolvedAestheticsExport, SharedKind, SharedLink, SharedModelFolder, ValidatedAestheticsExport,
    classify, common_link_name, model_suffix,
};
use dds_convert::SourceFormat;
use pes_version::{Engine, PesVersion};
use vtree::ScopePath;

use super::ids::shared_folders_taking_ids;
use super::mapped_players;
use crate::kit_variants::model_variant_sets;

/// The kit texture stems, in the order of the kit config's texture-name fields.
pub(crate) const KIT_TEXTURE_STEMS: [&str; 5] =
    ["kit", "kit_back", "kit_chest", "kit_leg", "kit_name"];

/// The source format of the texture file `name`, by its extension in any case: one of the
/// image formats `dds_convert` converts (`libs/dds_convert.md` "Accepted image formats");
/// `None` for a name without one.
pub(crate) fn texture_format(name: &str) -> Option<SourceFormat> {
    let (_, extension) = name.rsplit_once('.')?;
    SourceFormat::from_extension(extension)
}

/// The three Fox packages a player folder's models go to, each a `.fpk` and `.fpkd` pair in
/// its own game folder: the face by player id, the boots and the gloves by the player's
/// planned model id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelPackage {
    /// `face/real/{player id}/#Win/face.fpk`.
    Face,
    /// `boots/k{id}/#Win/boots.fpk`.
    Boots,
    /// `glove/g{id}/#Win/glove.fpk`.
    Gloves,
}

impl ModelPackage {
    /// The three packages in a folder's canonical order: the face, the boots, the gloves.
    pub(crate) const ALL: [ModelPackage; 3] = [
        ModelPackage::Face,
        ModelPackage::Boots,
        ModelPackage::Gloves,
    ];

    /// The package's file stem: `face.fpk`, `boots.fpk`, `glove.fpk`.
    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            ModelPackage::Face => "face",
            ModelPackage::Boots => "boots",
            ModelPackage::Gloves => "glove",
        }
    }

    /// The package as a finding names it (`shared_texture_conflict`'s `dropped`): `face`,
    /// `boots`, `gloves`.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ModelPackage::Face => "face",
            ModelPackage::Boots => "boots",
            ModelPackage::Gloves => "gloves",
        }
    }
}

/// The package a shared folder of `kind` is loaded as: `Boots/` holds boots models, `Gloves/`
/// gloves models, `Faces/` face parts.
pub(crate) fn package_of(kind: SharedKind) -> ModelPackage {
    match kind {
        SharedKind::Face => ModelPackage::Face,
        SharedKind::Boots => ModelPackage::Boots,
        SharedKind::Gloves => ModelPackage::Gloves,
    }
}

/// The link file for the shared folder `name` of `kind`, as the export spells it without the
/// tolerated `.txt` tail (`Crocs.boots`).
pub(crate) fn link_name(kind: SharedKind, name: &str) -> String {
    let extension = match kind {
        SharedKind::Face => "face",
        SharedKind::Boots => "boots",
        SharedKind::Gloves => "gloves",
    };
    format!("{name}.{extension}")
}

/// Whether `player`'s `link` combines on Fox: the shared folder's models become parts of the
/// player's own package instead of the shared output being loaded as it is. A face link
/// always does, a shared face having no output of its own; a boots or gloves link does when
/// the player holds a model of that package (`player_folders.md` "A link plus local models
/// combines"), a model `ingame_face` makes a boots part included. Pre-Fox has no exclusive
/// packages, so there every link is plain.
pub(crate) fn link_combines(player: &PlayerFolder, link: &SharedLink) -> bool {
    match link.kind {
        SharedKind::Face => true,
        SharedKind::Boots | SharedKind::Gloves => holds_model(player, package_of(link.kind)),
    }
}

/// The shared folder `link` names in `export`, matched as validation matched it: by
/// case-folded name. `None` only for a link validation did not resolve, and it drops the
/// folder of such a link.
pub(crate) fn linked_folder<'a>(
    export: &'a ValidatedAestheticsExport,
    link: &SharedLink,
) -> Option<&'a SharedModelFolder> {
    let folders = match link.kind {
        SharedKind::Face => &export.faces,
        SharedKind::Boots => &export.boots,
        SharedKind::Gloves => &export.gloves,
    };
    let name_key = vtree::fold_name(&link.name);
    folders
        .iter()
        .find(|folder| vtree::fold_name(&folder.folder_name) == name_key)
}

/// What one file of a player folder becomes in the Fox output.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PlayerFile {
    /// A model, packed into `package` as `<name>.fmdl` with its texture paths rewritten:
    /// `name` is its allowed name, the free prefix stripped (`kit_boots.fmdl` packs as
    /// `boots.fmdl`).
    Model {
        package: ModelPackage,
        name: &'static str,
    },
    /// A `.common` link to an FMDL (`legs.fmdl.common` → `Common/legs.fmdl`): the Common model
    /// is a part of `package` under `name`, merged with the player's parts of that name, since
    /// Fox cannot load a model from Common (`player_folders.md` "Common model links and model
    /// merging"). The role is the link's; the Models task reads the Common model, which
    /// planning resolves (`plan::CommonModel`), never the empty link.
    CommonModel {
        package: ModelPackage,
        name: &'static str,
    },
    /// A face file packed into `package` as it is, under `name`: `face_diff.bin` and
    /// `fcl_hair_sim.fclo`. Not a part to merge: the package holds one, the player folder's
    /// own over a combined face folder's (`player_folders.md` "A link plus local models
    /// combines").
    Packed {
        package: ModelPackage,
        name: &'static str,
    },
    /// A `face_diff.xml`: the face's `face_diff.bin` given as text, the base64 of the binary
    /// file (`player_folders.md` "`face_diff.xml`"), decoded and packed into the face package
    /// as `face_diff.bin`. It goes wherever a `face_diff.bin` would, and stands for one: the
    /// player folder's own, in either form, over a combined face folder's.
    FaceDiffXml,
    /// A `face_diff.bin`, `face_diff.xml` or `fcl_hair_sim.fclo` in a folder with no face
    /// model and no face link, `ingame_face` or not: there is no face for it to shape, so it
    /// is reported as `face_file_not_used` by the structure pass and never read; a player
    /// folder's blank face takes the bundled face diff (`pipeline.md` "2. Per-export serial
    /// steps", item 4). In a shared folder the subset gate names it instead
    /// (`shared_not_compiled`).
    UnusedFaceFile,
    /// A model's skeleton, the `.skl` named after a `fcl_hair` or `boots` part, packed into
    /// `package` under its slot's `name` (`fcl_hair_sim.skl`, `boots.skl`). Every part may
    /// bring one, and the merge decides whose is packed (`player_folders.md` "Merge
    /// constraint").
    Skeleton {
        package: ModelPackage,
        name: &'static str,
    },
    /// The `.skl` named after a `face_high`, `hair_high` or `oral` model, which have no
    /// skeleton slot (`player_folders.md` "SKL pairing"): reported as `skl_no_slot` by the
    /// structure pass, and never read.
    SlotlessSkeleton,
    /// A per-kit model with a lower variant of its set beside it (`pants_kit2.fmdl` beside
    /// `pants_kit1.fmdl`): a Fox target cannot switch models with the kit, so planning reports
    /// `kit_variant_model_fox` and the file is never read (`kit_variants::model_variant_sets`).
    LeftOutKitVariant,
    /// A texture with this stem and source format, converted once into the player's common
    /// folder.
    Texture(String, SourceFormat),
    /// A `.common` link to a texture (`hair.dds.common` → `Common/hair.dds`): the stem, here
    /// `hair`, stands for the Common texture, which the export's Common textures task packs
    /// once in the team's Common output (`model_format.md` "Link files (`.common`)"). The
    /// folder's models point the stem there; nothing reads the empty link.
    CommonTexture(String),
}

impl PlayerFile {
    /// The package the file goes into; `None` for a texture, which goes to the player's
    /// common folder for every package to point at, for a texture link, whose texture is the
    /// team's, and for a skeleton with no slot, an unused face file and a left-out kit
    /// variant, which go nowhere.
    pub(crate) fn package(&self) -> Option<ModelPackage> {
        match self {
            PlayerFile::Model { package, .. }
            | PlayerFile::CommonModel { package, .. }
            | PlayerFile::Packed { package, .. }
            | PlayerFile::Skeleton { package, .. } => Some(*package),
            PlayerFile::FaceDiffXml => Some(ModelPackage::Face),
            PlayerFile::UnusedFaceFile
            | PlayerFile::SlotlessSkeleton
            | PlayerFile::LeftOutKitVariant
            | PlayerFile::Texture(..)
            | PlayerFile::CommonTexture(_) => None,
        }
    }
}

/// The name the skeleton of a part of `package` under `name` packs as: `fcl_hair_sim.skl` for
/// the hair, `boots.skl` for the boots, the two destinations with a skeleton slot
/// (`player_folders.md` "SKL pairing"); `None` for `face_high`, `hair_high`, `oral` and the
/// gloves.
pub(crate) fn skeleton_slot(package: ModelPackage, name: &str) -> Option<&'static str> {
    match (package, name) {
        (ModelPackage::Face, "fcl_hair") => Some("fcl_hair_sim.skl"),
        (ModelPackage::Boots, _) => Some("boots.skl"),
        (ModelPackage::Face | ModelPackage::Gloves, _) => None,
    }
}

/// The FMDL a `.common` link named `link_name` stands for (`legs.fmdl.common` → `legs.fmdl`);
/// `None` for a link to anything else.
fn linked_fmdl(link_name: &str) -> Option<String> {
    common_link_name(link_name).filter(|name| classify(name) == FileKind::Model(ModelFormat::Fmdl))
}

/// The stem of the texture a `.common` link named `link_name` stands for, as the link spells
/// it (`hair.dds.common` → `hair`), when the linked name is in an image format `dds_convert`
/// converts; `None` for a link to anything else (a model, a material file, which `compile`
/// does not build yet).
fn linked_texture_stem(link_name: &str) -> Option<String> {
    let linked = common_link_name(link_name)?;
    texture_format(&linked)?;
    Some(file_stem(&linked).to_owned())
}

/// Whether `path` is a file directly in the export's `Common/` folder: the only place a link
/// resolves, and the only place `compile` reads a Common file from.
fn is_direct_common_file(path: &ScopePath) -> bool {
    path.segments().count() == 2
}

/// The file named `name` directly in `Common/`, among `common` (the export's `Common/` files),
/// matched as validation matched a link's target: by case-folded name.
pub(crate) fn common_file<'a>(
    common: &'a [FileDescriptor],
    name: &str,
) -> Option<&'a FileDescriptor> {
    let key = vtree::fold_name(name);
    common
        .iter()
        .find(|file| is_direct_common_file(&file.path) && vtree::fold_name(file.path.name()) == key)
}

/// The `.skl` directly in `Common/` paired with the Common model named `model_name`
/// (`legs.skl` for `legs.fmdl`), when there is one: the skeleton that travels with a `.common`
/// link (`player_folders.md` "SKL pairing").
pub(crate) fn common_skeleton<'a>(
    common: &'a [FileDescriptor],
    model_name: &str,
) -> Option<&'a FileDescriptor> {
    common_file(common, &format!("{}.skl", file_stem(model_name)))
}

/// Where a file sits in its model folder: directly in it, or one level down in one of the
/// four reserved subfolders (`player_folders.md` "Reserved subfolders"), whose name forces the
/// category of the models in it. A file anywhere else has no role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Position {
    /// Directly in the folder.
    Direct,
    /// In `face/`: face parts, whatever their names.
    Face,
    /// In `boots/`: the boots, whatever their names.
    Boots,
    /// In `gloves/`: gloves, each named for its hand.
    Gloves,
    /// In `common/`: textures only.
    Common,
}

/// `file`'s position in the folder at `folder`; `None` for any other nesting. The reserved
/// names are matched in any case, as validation matches them.
fn position(folder: &ScopePath, file: &FileDescriptor) -> Option<Position> {
    let parent = file.path.parent()?;
    if &parent == folder {
        return Some(Position::Direct);
    }
    if parent.parent().as_ref() != Some(folder) {
        return None;
    }
    let name = parent.name();
    [
        ("face", Position::Face),
        ("boots", Position::Boots),
        ("gloves", Position::Gloves),
        ("common", Position::Common),
    ]
    .into_iter()
    .find(|(reserved, _)| name.eq_ignore_ascii_case(reserved))
    .map(|(_, position)| position)
}

/// The package and allowed name a model named `<free part>_<suffix>` goes to by its suffix
/// alone (`player_folders.md` "Model names").
fn suffix_role(suffix: Option<ModelSuffix>) -> (ModelPackage, &'static str) {
    match suffix {
        Some(ModelSuffix::FaceHigh) => (ModelPackage::Face, "face_high"),
        Some(ModelSuffix::HairHigh) => (ModelPackage::Face, "hair_high"),
        Some(ModelSuffix::Oral) => (ModelPackage::Face, "oral"),
        // A model whose name says nothing about what it is is face content, a part of the
        // hair merge; the structure pass reports each one as `fmdl_fcl_hair_fallback`.
        Some(ModelSuffix::FclHair) | None => (ModelPackage::Face, "fcl_hair"),
        Some(ModelSuffix::Boots) => (ModelPackage::Boots, "boots"),
        // Hand-skeleton models have nowhere else to go on Fox.
        Some(ModelSuffix::GloveL | ModelSuffix::HandL) => (ModelPackage::Gloves, "glove_l"),
        Some(ModelSuffix::GloveR | ModelSuffix::HandR) => (ModelPackage::Gloves, "glove_r"),
    }
}

/// The package and allowed name of the model with file stem `stem` at `position`: by its
/// suffix when directly in the folder; in a reserved subfolder, the subfolder's category,
/// where a suffix of that category keeps its name and any other model takes the category's
/// one name. The gloves have no such name (a glove must say which hand it is), so a glove
/// whose suffix gives no side has no role, and `common/` holds no models. When `ingame_face`
/// is set (the folder holds the marker), a model that would be a part of the face's
/// `fcl_hair` is a part of the boots named `boots`.
fn model_role(
    position: Position,
    stem: &str,
    ingame_face: bool,
) -> Option<(ModelPackage, &'static str)> {
    let (package, name) = suffix_role(model_suffix(stem));
    let role = match position {
        Position::Direct => (package, name),
        Position::Face if package == ModelPackage::Face => (package, name),
        Position::Face => (ModelPackage::Face, "fcl_hair"),
        Position::Boots => (ModelPackage::Boots, "boots"),
        Position::Gloves if package == ModelPackage::Gloves => (package, name),
        Position::Gloves | Position::Common => return None,
    };
    // The marker means no face package: the hair's parts would be lost, and the boots take
    // the full body skeleton as the hair does (`player_folders.md` "`ingame_face` marker").
    // The explicit face names have no such home; validation drops a marked folder holding one.
    if ingame_face && role == (ModelPackage::Face, "fcl_hair") {
        return Some((ModelPackage::Boots, "boots"));
    }
    Some(role)
}

/// `file`'s export path up to its extension (`Players/05 - A/boots/kit_boots`): what a model
/// and the skeleton named after it share, and nothing in another directory does.
fn path_stem(file: &FileDescriptor) -> &str {
    file_stem(file.path.as_str())
}

/// What a player folder's models say about its other files: which have a package to go in,
/// and which model a `.skl` pairs with. Computed once per folder and passed to `player_file`.
/// A model is listed by its `path_stem`, so a skeleton pairs with the model beside it, not
/// with one of the same name in another of the folder's directories.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FolderModels {
    /// The player folder holds `ingame_face`, so its models take the roles `model_role` gives
    /// under the marker. A shared folder has none.
    ingame_face: bool,
    /// The folder's face files have a package to go in: it holds a face model, or links a
    /// shared face (`with_linked_face`).
    face: bool,
    /// The path stems of its `fcl_hair` parts, a model with no recognized suffix included
    /// (`player_folders.md` "Model names": face content the hair merge takes), each of whose
    /// skeletons packs as `fcl_hair_sim.skl` (several parts merge into one `fcl_hair.fmdl`,
    /// and the merge decides whose skeleton that is).
    hair_stems: Vec<String>,
    /// The path stems of its boots models, each of whose skeletons packs as `boots.skl`, by
    /// the same rule.
    boots_stems: Vec<String>,
    /// The path stems of its `face_high`, `hair_high` and `oral` models, which have no
    /// skeleton slot: a skeleton named after one is `skl_no_slot`. The gloves have none
    /// either, and no `.skl` pairs with a glove.
    slotless_stems: Vec<String>,
    /// The export paths of its per-kit models with a lower variant of their set beside them,
    /// which are not models of the folder on Fox (`PlayerFile::LeftOutKitVariant`): no
    /// skeleton pairs with one.
    left_out_variants: Vec<ScopePath>,
}

impl FolderModels {
    /// The models among `files` of the folder at `folder`, a folder without `ingame_face`
    /// (`of_player_files`).
    pub(crate) fn of(folder: &ScopePath, files: &[FileDescriptor]) -> FolderModels {
        FolderModels::of_player_files(folder, files, false)
    }

    /// The models among `files` of the folder at `folder`, which holds `ingame_face` when
    /// `ingame_face` is set: its `.fmdl` files and its `.common` links to one, directly in it
    /// or in a reserved subfolder, each with its resolved role. A link counts as a model of
    /// its role's package, but pairs no skeleton of the folder's: a Common model's skeleton
    /// is Common's, resolved at planning (`common_skeleton`). A per-kit model with a lower
    /// variant of its set beside it is left out (`model_variant_sets`).
    pub(crate) fn of_player_files(
        folder: &ScopePath,
        files: &[FileDescriptor],
        ingame_face: bool,
    ) -> FolderModels {
        let mut models = FolderModels {
            ingame_face,
            left_out_variants: model_variant_sets(files)
                .into_iter()
                .flat_map(|set| set.left_out)
                .collect(),
            ..FolderModels::default()
        };
        for file in files {
            let Some(position) = position(folder, file) else {
                continue;
            };
            if models.left_out_variants.contains(&file.path) {
                continue;
            }
            let file_name = file.path.name();
            let (model_name, local) = if file.kind == FileKind::Model(ModelFormat::Fmdl) {
                (file_name.to_owned(), true)
            } else if file.kind == FileKind::CommonLink
                && let Some(linked) = linked_fmdl(file_name)
            {
                (linked, false)
            } else {
                continue;
            };
            let Some((package, name)) = model_role(position, file_stem(&model_name), ingame_face)
            else {
                continue;
            };
            if !local {
                models.face |= package == ModelPackage::Face;
                continue;
            }
            // The stems fold, the way the file system folds the pair: `boots.fmdl`
            // beside `Boots.skl` is one model and its skeleton.
            let stem = vtree::fold_name(path_stem(file));
            match (package, name) {
                (ModelPackage::Face, "fcl_hair") => {
                    models.face = true;
                    models.hair_stems.push(stem);
                }
                (ModelPackage::Face, _) => {
                    models.face = true;
                    models.slotless_stems.push(stem);
                }
                (ModelPackage::Boots, _) => models.boots_stems.push(stem),
                (ModelPackage::Gloves, _) => {}
            }
        }
        models
    }

    /// The models of the player folder `folder`: its own, under its `ingame_face` marker
    /// when it holds one, a linked shared face counting as a face model of the folder's
    /// (`with_linked_face`).
    pub(crate) fn of_player(folder: &PlayerFolder) -> FolderModels {
        let models = FolderModels::of_player_files(&folder.path, &folder.files, folder.ingame_face);
        if folder
            .links
            .iter()
            .any(|link| matches!(link.kind, SharedKind::Face))
        {
            return models.with_linked_face();
        }
        models
    }

    /// The models of a player folder linking a shared face: the shared face is the player's
    /// face (a merge of one, `player_folders.md` "A link plus local models combines"), so the
    /// folder's `face_diff.bin` and `fcl_hair_sim.fclo` have a package to go in with no face
    /// model of the folder's own.
    pub(crate) fn with_linked_face(mut self) -> FolderModels {
        self.face = true;
        self
    }
}

/// What `file` of the player folder at `folder`, whose models are `models`, becomes in the
/// Fox output; `None` when it has no role `compile` builds yet. A file in a reserved
/// subfolder is a part of the folder like a file directly in it, its category forced by the
/// subfolder's name (`model_role`); its textures are the folder's own, and the face's files
/// may sit in `face/`.
pub(crate) fn player_file(
    folder: &ScopePath,
    file: &FileDescriptor,
    models: &FolderModels,
) -> Option<PlayerFile> {
    let position = position(folder, file)?;
    let name = file.path.name();
    let stem = file_stem(name);
    let path_stem = path_stem(file);
    let path_fold = vtree::fold_name(path_stem);
    // A face file sits in the folder or its `face/`; without a face to shape it is not used.
    let face_file = |role| {
        if !matches!(position, Position::Direct | Position::Face) {
            return None;
        }
        Some(if models.face {
            role
        } else {
            PlayerFile::UnusedFaceFile
        })
    };
    let face_packed = |name| {
        face_file(PlayerFile::Packed {
            package: ModelPackage::Face,
            name,
        })
    };
    match file.kind {
        // A left-out kit variant is one only where it would be a model: a file planning gives no
        // role keeps none.
        FileKind::Model(ModelFormat::Fmdl) => {
            model_role(position, stem, models.ingame_face).map(|(package, name)| {
                if models.left_out_variants.contains(&file.path) {
                    PlayerFile::LeftOutKitVariant
                } else {
                    PlayerFile::Model { package, name }
                }
            })
        }
        // A model link takes the role the linked model would have in its place; a texture link
        // stands for its stem wherever validation resolves a link, which is not in `common/`
        // (only textures may sit there, so a link there is never checked against `Common/`).
        // A link to anything else has no role yet.
        FileKind::CommonLink => match linked_fmdl(name) {
            Some(linked) => model_role(position, file_stem(&linked), models.ingame_face)
                .map(|(package, name)| PlayerFile::CommonModel { package, name }),
            None if position == Position::Common => None,
            None => linked_texture_stem(name).map(PlayerFile::CommonTexture),
        },
        FileKind::Texture => {
            texture_format(name).map(|format| PlayerFile::Texture(stem.to_owned(), format))
        }
        // The game loads a skeleton under its slot's name; the export names it after the
        // model it pairs with, and without that model the skeleton has nothing to drive.
        // The stems fold, as `hair_stems`/`boots_stems`/`slotless_stems` build them.
        FileKind::Skl if models.hair_stems.iter().any(|hair| hair == &path_fold) => {
            Some(PlayerFile::Skeleton {
                package: ModelPackage::Face,
                name: "fcl_hair_sim.skl",
            })
        }
        FileKind::Skl if models.boots_stems.iter().any(|boots| boots == &path_fold) => {
            Some(PlayerFile::Skeleton {
                package: ModelPackage::Boots,
                name: "boots.skl",
            })
        }
        FileKind::Skl
            if models
                .slotless_stems
                .iter()
                .any(|model| model == &path_fold) =>
        {
            Some(PlayerFile::SlotlessSkeleton)
        }
        FileKind::Bin if name.eq_ignore_ascii_case("face_diff.bin") => face_packed("face_diff.bin"),
        FileKind::Fclo if name.eq_ignore_ascii_case("fcl_hair_sim.fclo") => {
            face_packed("fcl_hair_sim.fclo")
        }
        FileKind::Xml if name.eq_ignore_ascii_case("face_diff.xml") => {
            face_file(PlayerFile::FaceDiffXml)
        }
        FileKind::Model(_)
        | FileKind::Skl
        | FileKind::Bin
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::SharedLink(_)
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// Whether `role` is a part of `package`: a model of the folder's own, or a Common model its
/// link brings in.
pub(crate) fn is_part_of(role: &PlayerFile, package: ModelPackage) -> bool {
    matches!(
        role,
        PlayerFile::Model { package: owner, .. } | PlayerFile::CommonModel { package: owner, .. }
            if *owner == package
    )
}

/// Whether the player folder `player` has a model that packs into `package`, its own or one a
/// `.common` link brings in, under its `ingame_face` marker when it holds one: on Fox, what
/// gives a player its own package of that kind.
pub(crate) fn holds_model(player: &PlayerFolder, package: ModelPackage) -> bool {
    let models = FolderModels::of_player(player);
    player.files.iter().any(|file| {
        player_file(&player.path, file, &models).is_some_and(|role| is_part_of(&role, package))
    })
}

/// A file name's stem: the name up to its last `.`.
pub(crate) fn file_stem(name: &str) -> &str {
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// The first thing in `resolved` that `compile` cannot build yet for `version`, as the context
/// entry of `content_not_yet_compiled`: `what`, a path, the target or `refs`. `None` when
/// `compile` builds all of it. Planning drops a Fox target's `kit_mask` before asking: the
/// gate would count it.
pub(crate) fn first_not_compiled(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
) -> Option<(&'static str, String)> {
    // The walk's order is the plan's ("Phase 3 scope"), so one export always names the same
    // item; reordering the checks changes which item a member is told about.
    match version.engine() {
        Engine::Fox => {}
        Engine::PreFox => return Some(("what", version.to_string())),
    }
    if resolved.identity == ExportIdentity::Referees {
        return Some(("what", "refs".to_owned()));
    }
    let export = &resolved.export;
    // A folder no roster slot maps is not compiled, so whatever it holds does not count.
    for folder in mapped_players(export) {
        if let Some(item) = player_not_compiled(export, folder) {
            return Some(item);
        }
    }
    // A shared boots or gloves folder compiles on its own when a mapped player links it
    // plainly; one no such player links has no output and is not counted. A shared face has
    // no output of its own and is walked above, as a source of each player linking it:
    // validation drops a shared folder no mapped player links, so every `Faces/` folder has
    // a linking player.
    for kind in [SharedKind::Boots, SharedKind::Gloves] {
        for folder in shared_folders_taking_ids(export, version.engine(), kind) {
            if let Some(item) = shared_not_compiled(kind, folder) {
                return Some(item);
            }
        }
    }
    for kit in export.kits.kits.values() {
        let refused = kit.textures.iter().find(|texture| {
            !KIT_TEXTURE_STEMS.contains(&texture.stem.as_str())
                || texture_format(texture.file.path.name()).is_none()
        });
        if let Some(texture) = refused {
            return Some(what_entry(&texture.file));
        }
    }
    // A slot with a portrait from both sources is not refused: the deep pass has skipped an
    // export whose two files differ (`portrait_conflict`), so identical ones are one portrait.
    // The logo is compiled by the export's logo task, so it is not walked.
    let mut rest = export.collars.iter().chain(
        export
            .common
            .iter()
            .filter(|file| !common_file_compiled(file)),
    );
    rest.next().map(what_entry)
}

/// Whether `compile` builds the `Common/` file, or accepts it: directly in the folder, an
/// FMDL or a `.skl` (reached through a player's `.common` link; one no link names builds
/// nothing) or a texture (the export's Common textures task). Any other kind, and any file
/// deeper in the folder, is named.
fn common_file_compiled(file: &FileDescriptor) -> bool {
    is_direct_common_file(&file.path)
        && match file.kind {
            FileKind::Model(ModelFormat::Fmdl) | FileKind::Skl => true,
            FileKind::Texture => texture_format(file.path.name()).is_some(),
            FileKind::Model(ModelFormat::PesModel | ModelFormat::Gltf)
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::Mtl
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::SharedLink(_)
            | FileKind::CommonLink
            | FileKind::Marker(_)
            | FileKind::Metadata(_)
            | FileKind::Other => false,
        }
}

/// The first thing in the mapped player `folder` of `export` that `compile` cannot build into
/// its Fox packages yet. A folder holding no model compiles too: without `ingame_face` to its
/// blank face, with the marker to nothing but what its links load.
fn player_not_compiled(
    export: &ValidatedAestheticsExport,
    folder: &PlayerFolder,
) -> Option<(&'static str, String)> {
    let models = FolderModels::of_player(folder);
    if let Some(file) = folder
        .files
        .iter()
        .find(|file| player_file(&folder.path, file, &models).is_none())
    {
        return Some(what_entry(file));
    }
    // A boots or gloves link alone loads the shared output as it is; one beside a local model
    // of its package combines, as every face link does: the shared folder's files become the
    // player's own, with the roles a shared folder's files have.
    for link in folder
        .links
        .iter()
        .filter(|link| link_combines(folder, link))
    {
        let shared = linked_folder(export, link)
            .expect("validation drops a player folder whose link names no shared folder");
        if let Some(item) = shared_not_compiled(link.kind, shared) {
            return Some(item);
        }
    }
    None
}

/// The first thing in the shared `folder` of `kind` that `compile` cannot build into a Fox
/// package yet, its own or a combining player's: the files go by a player folder's roles (a
/// `Faces/` folder's face files and hair skeletons included), but only the package the folder
/// is loaded as and its textures have a place to go, and a folder with no model is named as a
/// whole.
fn shared_not_compiled(
    kind: SharedKind,
    folder: &SharedModelFolder,
) -> Option<(&'static str, String)> {
    let path = &folder.path;
    let package = package_of(kind);
    let models = FolderModels::of(path, &folder.files);
    let mut has_model = false;
    for file in &folder.files {
        let Some(role) = player_file(path, file, &models) else {
            return Some(what_entry(file));
        };
        // A model of another package has no package here, a `.common` link (kept by a
        // non-strict file-type check) resolves only from a player folder, and a face file
        // with no face model has no place in a shared folder.
        if role.package().is_some_and(|owner| owner != package)
            || matches!(
                role,
                PlayerFile::CommonModel { .. }
                    | PlayerFile::CommonTexture(_)
                    | PlayerFile::UnusedFaceFile
            )
        {
            return Some(what_entry(file));
        }
        has_model |= matches!(role, PlayerFile::Model { .. });
    }
    if !has_model {
        return Some(("what", path.as_str().to_owned()));
    }
    None
}

/// The `what` context entry naming `file` by its export path.
fn what_entry(file: &FileDescriptor) -> (&'static str, String) {
    ("what", file.path.as_str().to_owned())
}

#[cfg(test)]
mod tests {
    use studio_core::ExportId;

    use super::*;
    use crate::plan::{TaskKind, plan_run};
    use crate::testing::{resolved, resolved_with_issues, two_team_colors};

    /// A Fox face folder Phase 3 compiles: a face model and `face_diff.bin`.
    const FACE: [&str; 2] = [
        "Players/03 - A/face_high.fmdl",
        "Players/03 - A/face_diff.bin",
    ];

    /// The gate's first hit in the export `co Midcup Gate` holding `files`, for PES 21.
    fn first_hit(files: &[&str]) -> Option<(&'static str, String)> {
        let files: Vec<(&str, u64)> = files.iter().map(|path| (*path, 1)).collect();
        first_not_compiled(
            &resolved("co Midcup Gate", &files, &[], None),
            PesVersion::Pes21,
        )
    }

    /// The gate's first hit in the export `co Midcup Gate` holding `FACE` and `files`, for PES 21.
    fn gate(files: &[&str]) -> Option<(&'static str, String)> {
        first_hit(&[FACE.as_slice(), files].concat())
    }

    fn what(path: &str) -> Option<(&'static str, String)> {
        Some(("what", path.to_owned()))
    }

    #[test]
    fn fox_player_folders_dds_portraits_and_kits_are_compiled() {
        assert_eq!(gate(&[]), None);
        assert_eq!(
            gate(&[
                "Players/03 - A/hair_high.fmdl",
                "Players/03 - A/oral.fmdl",
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
                "Players/03 - A/kit_boots.fmdl",
                "Players/03 - A/kit_boots.skl",
                "Players/03 - A/glove_l.fmdl",
                "Players/03 - A/keeper_gloveR.fmdl",
                "Players/03 - A/skin.dds",
                "Players/03 - A/hair.FTEX",
                "Players/03 - A/portrait.DDS",
                "Portraits/player_07.dds",
                "Kits/g1/kit.dds",
                "Kits/g1/config.toml",
                "Kits/g1/fox",
                "Kits/p1/kit_back.ftex",
                "Kits/p1/kit_chest.dds",
                "Kits/p1/kit_leg.dds",
                "Kits/p1/kit_name.dds",
            ]),
            None
        );
        // A folder needs a model, not a face: boots alone compile, with or without their
        // skeleton, and so do gloves.
        assert_eq!(
            first_hit(&["Players/03 - A/boots.fmdl", "Players/03 - A/skin.dds"]),
            None
        );
        assert_eq!(first_hit(&["Players/03 - A/handL.fmdl"]), None);
        // So does a folder holding only a link to a shared folder.
        assert_eq!(
            first_hit(&["Players/03 - A/Crocs.boots", "Boots/Crocs/boots.fmdl"]),
            None
        );
    }

    #[test]
    fn parts_packing_under_one_name_merge_for_every_package() {
        assert_eq!(gate(&["Players/03 - A/x_face_high.fmdl"]), None);
        for (first, second) in [
            ("boots.fmdl", "kit_boots.fmdl"),
            ("glove_l.fmdl", "gloveL.fmdl"),
            ("glove_r.fmdl", "handR.fmdl"),
        ] {
            assert_eq!(
                first_hit(&[
                    &format!("Players/03 - A/{first}"),
                    &format!("Players/03 - A/{second}"),
                ]),
                None,
                "{first} + {second}"
            );
        }
        // Each boots part's skeleton, too, and each hair part's.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/boots.fmdl",
                "Players/03 - A/boots.skl",
                "Players/03 - A/kit_boots.fmdl",
                "Players/03 - A/kit_boots.skl",
            ]),
            None
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                "Players/03 - A/x_fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
            ]),
            None
        );
        // A hair part without a skeleton beside one with is the merge's conflict, not the
        // gate's; hair parts with none get the standard skeleton.
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.skl",
            ]),
            None
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.fmdl",
            ]),
            None
        );
    }

    #[test]
    fn a_pre_fox_target_is_named() {
        let export = resolved("co Midcup Gate", &[(FACE[0], 1), (FACE[1], 1)], &[], None);
        assert_eq!(
            first_not_compiled(&export, PesVersion::Pes17),
            what("PES 2017")
        );
        for version in [PesVersion::Pes15, PesVersion::Pes16] {
            assert_eq!(
                first_not_compiled(&export, version),
                what(&version.to_string())
            );
        }
    }

    #[test]
    fn a_referee_export_is_named_refs() {
        let export = resolved(
            "refs Cup",
            &[
                ("Players/Keeper/face_high.fmdl", 1),
                ("Players/Keeper/face_diff.bin", 1),
            ],
            &[],
            Some(b"01 Keeper\n"),
        );
        assert_eq!(first_not_compiled(&export, PesVersion::Pes21), what("refs"));
    }

    #[test]
    fn a_player_file_with_no_role_is_named() {
        // A file under any other subfolder, or a model in `common/`, is validation's
        // `file_type_disallowed` and never reaches the gate.
        for file in [
            // A glove must say which hand it is.
            "Players/03 - A/gloves/keeper.fmdl",
            "Players/03 - A/boots/face_diff.bin",
            "Players/03 - A/boots/face_diff.xml",
        ] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
    }

    #[test]
    fn a_texture_in_any_accepted_image_format_is_compiled_wherever_a_texture_goes() {
        // A player folder's, a combined shared folder's, Common's, a kit's and a portrait from
        // either source, in the formats `dds_convert` converts.
        assert_eq!(
            gate(&[
                "Players/03 - A/skin.tga",
                "Players/03 - A/hair.webp",
                "Players/03 - A/oral.jpg",
                "Players/03 - A/portrait.tif",
                "Players/03 - A/Crocs.boots",
                "Players/03 - A/kit_boots.fmdl",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/sole.bmp",
                "Common/hair.png",
                "Kits/p1/kit.png",
                "Kits/p1/kit_back.tga",
                "Portraits/player_07.png",
            ]),
            None
        );
    }

    #[test]
    fn a_reserved_subfolder_s_files_are_compiled_as_parts_of_its_category() {
        // Boots alone, with a texture in `common/`: no face needed (TC-MOD-14's gate half).
        assert_eq!(
            first_hit(&[
                "Players/03 - A/boots/hair_high.fmdl",
                "Players/03 - A/common/skin.dds",
            ]),
            None
        );
        assert_eq!(first_hit(&["Players/03 - A/boots/boots.fmdl"]), None);
        // A face subfolder's unsuffixed model is a hair part, with its skeleton and the
        // face's files; the gloves by side, beside a loose model of the other hand.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/face/torso.fmdl",
                "Players/03 - A/face/torso.skl",
                "Players/03 - A/face/face_diff.bin",
                "Players/03 - A/face/fcl_hair_sim.fclo",
                "Players/03 - A/face/skin.dds",
            ]),
            None
        );
        // The face diff as text, directly in the folder or in `face/`.
        for face_diff in [
            "Players/03 - A/face_diff.xml",
            "Players/03 - A/face/face_diff.xml",
        ] {
            assert_eq!(
                first_hit(&["Players/03 - A/face_high.fmdl", face_diff]),
                None,
                "{face_diff}"
            );
        }
        assert_eq!(
            first_hit(&[
                "Players/03 - A/gloves/glove_r.fmdl",
                "Players/03 - A/glove_l.fmdl",
            ]),
            None
        );
        // A subfolder's model counts as a local model of its package: the link combines.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/boots/boots.fmdl",
                "Players/03 - A/Crocs.boots",
                "Boots/Crocs/boots.fmdl",
            ]),
            None
        );
        let folder = folder(&["boots/x.fmdl", "gloves/glove_l.fmdl"]);
        assert!(holds_model(&folder, ModelPackage::Boots));
        assert!(holds_model(&folder, ModelPackage::Gloves));
        assert!(!holds_model(&folder, ModelPackage::Face));
    }

    #[test]
    fn an_unsuffixed_model_is_a_face_part_with_its_skeleton_and_a_slotless_skeleton_is_ignored() {
        // A model whose name says nothing about what it is, alone or beside the hair, with
        // or without a skeleton named after it.
        assert_eq!(first_hit(&["Players/03 - A/torso.fmdl"]), None);
        assert_eq!(
            first_hit(&["Players/03 - A/torso.fmdl", "Players/03 - A/torso.skl"]),
            None
        );
        assert_eq!(
            first_hit(&["Players/03 - A/torso.fmdl", "Players/03 - A/fcl_hair.fmdl"]),
            None
        );
        // The hair alone: its face files and skeleton are injected (TC-MOD-12's gate half).
        assert_eq!(first_hit(&["Players/03 - A/fcl_hair.fmdl"]), None);
        // A skeleton named after a face model with no skeleton slot is accepted and ignored.
        for model in ["face_high", "hair_high", "oral"] {
            assert_eq!(
                first_hit(&[
                    &format!("Players/03 - A/{model}.fmdl"),
                    &format!("Players/03 - A/{model}.skl"),
                ]),
                None,
                "{model}"
            );
        }
    }

    #[test]
    fn a_skeleton_without_the_model_of_its_name_is_named() {
        assert_eq!(
            gate(&["Players/03 - A/fcl_hair.skl"]),
            what("Players/03 - A/fcl_hair.skl")
        );
        assert_eq!(
            gate(&["Players/03 - A/torso.skl"]),
            what("Players/03 - A/torso.skl")
        );
        // `boots.skl` beside `kit_boots.fmdl` pairs with nothing: the skeleton is named
        // after the model's source name.
        assert_eq!(
            gate(&["Players/03 - A/kit_boots.fmdl", "Players/03 - A/boots.skl"]),
            what("Players/03 - A/boots.skl")
        );
        // The gloves have no skeleton slot.
        assert_eq!(
            gate(&["Players/03 - A/glove_l.fmdl", "Players/03 - A/glove_l.skl"]),
            what("Players/03 - A/glove_l.skl")
        );
    }

    #[test]
    fn a_portrait_in_any_accepted_format_is_compiled() {
        for file in [
            "Players/03 - A/portrait.png",
            "Players/03 - A/portrait.ftex",
            "Portraits/player_03.png",
            "Portraits/player_07.ftex",
        ] {
            assert_eq!(gate(&[file]), None, "{file}");
        }
    }

    #[test]
    fn ingame_face_and_fpc_on_are_compiled() {
        // `fpc_on` puts the FPC values into the team's kit configs, which `compile` builds.
        assert_eq!(gate(&["Players/03 - A/fpc_on"]), None);
        // `ingame_face` alone, beside a model the face would take, and beside a face file.
        assert_eq!(first_hit(&["Players/03 - A/ingame_face"]), None);
        assert_eq!(
            first_hit(&["Players/03 - A/ingame_face", "Players/03 - A/torso.fmdl"]),
            None
        );
        assert_eq!(
            first_hit(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/face_diff.bin",
                "Players/03 - A/ingame_face",
            ]),
            None
        );
    }

    #[test]
    fn a_folder_without_a_model_and_a_face_file_without_a_face_model_are_compiled() {
        // The folder gets its blank face, whatever else it holds, and a face file without a
        // face model is not used.
        for files in [
            &["Players/03 - A/portrait.dds"][..],
            &["Players/03 - A/skin.dds"],
            &["Players/03 - A/face_diff.bin", "Players/03 - A/skin.dds"],
            &["Players/03 - A/boots.fmdl", "Players/03 - A/face_diff.bin"],
            &[
                "Players/03 - A/face_diff.xml",
                "Players/03 - A/fcl_hair_sim.fclo",
            ],
        ] {
            assert_eq!(first_hit(files), None, "{files:?}");
        }
        // A face model alone: the files it lacks are injected.
        assert_eq!(first_hit(&["Players/03 - A/face_high.fmdl"]), None);
        // Without `fcl_hair.fmdl` the simulation still packs.
        assert_eq!(gate(&["Players/03 - A/fcl_hair_sim.fclo"]), None);
    }

    #[test]
    fn a_shared_boots_or_gloves_folder_a_player_links_plainly_is_compiled() {
        // A player holding only the link, and one holding a face beside it.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/Crocs.boots",
                "Players/03 - A/Grip.gloves",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/boots.skl",
                "Boots/Crocs/shirt.dds",
                "Gloves/Grip/glove_l.fmdl",
                "Gloves/Grip/gloveR.fmdl",
            ]),
            None
        );
        assert_eq!(
            gate(&["Players/03 - A/Crocs.boots", "Boots/Crocs/kit_boots.fmdl"]),
            None
        );
    }

    #[test]
    fn a_face_link_combines_and_a_boots_or_gloves_link_beside_a_local_model_of_its_kind_does() {
        for link in ["Round.face", "Round.face.txt"] {
            assert_eq!(
                gate(&[
                    &format!("Players/03 - A/{link}"),
                    "Faces/Round/face_high.fmdl"
                ]),
                None,
                "{link}"
            );
        }
        for (link, local, shared) in [
            ("Crocs.boots", "kit_boots.fmdl", "Boots/Crocs/boots.fmdl"),
            ("Grip.gloves", "handL.fmdl", "Gloves/Grip/glove_l.fmdl"),
        ] {
            assert_eq!(
                gate(&[
                    &format!("Players/03 - A/{link}"),
                    &format!("Players/03 - A/{local}"),
                    shared,
                ]),
                None,
                "{link}"
            );
        }
        // A local model of the other kind leaves the link plain.
        assert_eq!(
            gate(&[
                "Players/03 - A/Crocs.boots",
                "Players/03 - A/glove_l.fmdl",
                "Boots/Crocs/boots.fmdl",
            ]),
            None
        );
    }

    #[test]
    fn a_combined_shared_folder_s_files_are_walked_and_a_texture_stem_held_twice_is_compiled() {
        let combining = [
            "Players/03 - A/Crocs.boots",
            "Players/03 - A/kit_boots.fmdl",
        ];
        // What a plainly linked shared folder may hold, a combined one may too.
        assert_eq!(
            gate(&[
                combining[0],
                combining[1],
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/boots.skl",
                "Boots/Crocs/kit_boots.fmdl",
                "Boots/Crocs/sole.dds",
            ]),
            None
        );
        // A file the shared folder cannot hold is named, and a folder with no model as a whole.
        for face_diff in ["Boots/Crocs/face_diff.bin", "Boots/Crocs/face_diff.xml"] {
            assert_eq!(
                gate(&[
                    combining[0],
                    combining[1],
                    "Boots/Crocs/boots.fmdl",
                    face_diff,
                ]),
                what(face_diff)
            );
        }
        assert_eq!(
            gate(&[combining[0], combining[1], "Boots/Crocs/sole.dds"]),
            what("Boots/Crocs")
        );
        // A texture of a stem the player holds too, in any case or format, or one an earlier
        // combined folder holds, is compiled: whether the bytes agree is the textures task's
        // comparison.
        assert_eq!(
            gate(&[
                combining[0],
                combining[1],
                "Players/03 - A/Sole.dds",
                "Players/03 - A/Grip.gloves",
                "Players/03 - A/glove_l.fmdl",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/sole.ftex",
                "Gloves/Grip/glove_r.fmdl",
                "Gloves/Grip/sole.dds",
            ]),
            None
        );
    }

    #[test]
    fn a_combined_face_folder_s_files_take_a_face_s_roles() {
        let link = "Players/03 - A/Round.face";
        // Face models, the hair's skeleton, the face's files and textures.
        assert_eq!(
            gate(&[
                link,
                "Faces/Round/face_high.fmdl",
                "Faces/Round/hair_high.fmdl",
                "Faces/Round/oral.fmdl",
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/x_fcl_hair.skl",
                "Faces/Round/face_diff.bin",
                "Faces/Round/fcl_hair_sim.fclo",
                "Faces/Round/skin.dds",
                "Faces/Round/hair.ftex",
            ]),
            None
        );
        // An unsuffixed model is a hair part here too, with its skeleton, and a skeleton
        // named after a slotless face model is ignored here too.
        assert_eq!(
            gate(&[
                link,
                "Faces/Round/torso.fmdl",
                "Faces/Round/torso.skl",
                "Faces/Round/oral.fmdl",
                "Faces/Round/oral.skl",
            ]),
            None
        );
        // A model of another package, a face file without a face model, a skeleton pairing
        // with no model: named, as in a player folder.
        for named in [
            "Faces/Round/boots.fmdl",
            "Faces/Round/glove_l.fmdl",
            "Faces/Round/face_diff.bin",
            "Faces/Round/fcl_hair.skl",
        ] {
            assert_eq!(gate(&[link, named]), what(named), "{named}");
        }
        // The other files have a package to go in once a face model is in the folder.
        for face_diff in ["Faces/Round/face_diff.bin", "Faces/Round/face_diff.xml"] {
            assert_eq!(
                gate(&[link, face_diff, "Faces/Round/hair_high.fmdl"]),
                None,
                "{face_diff}"
            );
        }
        // A folder with no model, textures alone, is named as a whole.
        assert_eq!(gate(&[link, "Faces/Round/skin.dds"]), what("Faces/Round"));
    }

    #[test]
    fn the_face_s_files_may_come_from_either_of_its_sources_or_from_neither() {
        let link = "Players/03 - A/Round.face";
        let face_high = "Players/03 - A/face_high.fmdl";
        let shared_face = "Faces/Round/face_high.fmdl";
        // `face_diff.bin` in the shared folder serves a face model in the player's, and in the
        // player's a face model in the shared folder; missing from both, it is injected.
        assert_eq!(
            first_hit(&[face_high, link, shared_face, "Faces/Round/face_diff.bin"]),
            None
        );
        assert_eq!(
            first_hit(&[link, "Players/03 - A/face_diff.bin", shared_face]),
            None
        );
        assert_eq!(first_hit(&[link, shared_face]), None);
        // The hair's skeleton may pair with the shared folder's hair part, or the player's.
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                link,
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/x_fcl_hair.skl",
            ]),
            None
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                link,
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/fcl_hair_sim.fclo",
            ]),
            None
        );
    }

    /// The export's content folder holding the shared folders of `kind`.
    fn content_folder(kind: SharedKind) -> &'static str {
        match kind {
            SharedKind::Face => "Faces",
            SharedKind::Boots => "Boots",
            SharedKind::Gloves => "Gloves",
        }
    }

    /// `shared_not_compiled` over the shared `Boots/Crocs` (or `Gloves/Crocs`) holding `names`.
    fn shared_hit(kind: SharedKind, names: &[&str]) -> Option<(&'static str, String)> {
        let folder_path = format!("{}/Crocs", content_folder(kind));
        let files = names
            .iter()
            .map(|name| {
                let path = ScopePath::new(&format!("{folder_path}/{name}")).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect();
        let folder = SharedModelFolder {
            path: ScopePath::new(&folder_path).unwrap(),
            folder_name: "Crocs".to_owned(),
            files,
        };
        shared_not_compiled(kind, &folder)
    }

    #[test]
    fn a_shared_folder_takes_only_its_own_kind_s_models_with_their_files() {
        assert_eq!(
            shared_hit(
                SharedKind::Boots,
                &["boots.fmdl", "boots.skl", "shirt.dds", "sole.ftex"]
            ),
            None
        );
        assert_eq!(
            shared_hit(
                SharedKind::Gloves,
                &["glove_l.fmdl", "glove_r.fmdl", "grip.dds"]
            ),
            None
        );
        // Several parts under one name merge, each with its own skeleton.
        assert_eq!(
            shared_hit(
                SharedKind::Boots,
                &["boots.fmdl", "boots.skl", "kit_boots.fmdl", "kit_boots.skl"]
            ),
            None
        );
        assert_eq!(
            shared_hit(
                SharedKind::Face,
                &[
                    "face_high.fmdl",
                    "face_high.skl",
                    "fcl_hair.fmdl",
                    "fcl_hair.skl",
                    "x_fcl_hair.fmdl",
                    "torso.fmdl",
                    "torso.skl",
                    "face_diff.bin",
                    "fcl_hair_sim.fclo",
                    "skin.dds"
                ]
            ),
            None
        );
        for (kind, names, named) in [
            (
                SharedKind::Boots,
                &["boots.fmdl", "glove_l.fmdl"][..],
                "glove_l.fmdl",
            ),
            (
                SharedKind::Gloves,
                &["boots.fmdl", "glove_l.fmdl"],
                "boots.fmdl",
            ),
            (
                SharedKind::Face,
                &["boots.fmdl", "face_high.fmdl"],
                "boots.fmdl",
            ),
            (
                SharedKind::Boots,
                &["boots.fmdl", "face_diff.bin"],
                "face_diff.bin",
            ),
            (SharedKind::Face, &["face_diff.bin"], "face_diff.bin"),
            (
                SharedKind::Boots,
                &["boots.fmdl", "kit_boots.skl"],
                "kit_boots.skl",
            ),
        ] {
            assert_eq!(
                shared_hit(kind, names),
                what(&format!("{}/Crocs/{named}", content_folder(kind))),
                "{names:?}"
            );
        }
        // A folder with no model, textures alone or nothing at all, is named as a whole.
        assert_eq!(
            shared_hit(SharedKind::Boots, &["shirt.dds"]),
            what("Boots/Crocs")
        );
        assert_eq!(shared_hit(SharedKind::Gloves, &[]), what("Gloves/Crocs"));
    }

    #[test]
    fn a_kit_drawn_for_either_engine_is_compiled() {
        for marker in ["Kits/g1/pre-fox", "Kits/g1/fox"] {
            assert_eq!(gate(&["Kits/g1/kit.dds", marker]), None, "{marker}");
        }
    }

    #[test]
    fn a_kit_texture_the_config_has_no_field_for_is_named() {
        let file = "Kits/g1/kit_srm.dds";
        assert_eq!(gate(&["Kits/g1/kit.dds", file]), what(file));
    }

    #[test]
    fn the_logo_is_compiled_collars_are_named_and_common_holds_models_skeletons_and_textures() {
        assert_eq!(gate(&["logo.dds", "logo_small_crop.png"]), None);
        let collar = "Collars/collar.dds";
        assert_eq!(gate(&[collar]), what(collar));
        // Common's FMDLs and skeletons are reached through links, and one no link names
        // builds nothing; its textures are the export's Common textures task's.
        assert_eq!(
            gate(&[
                "Common/spare.fmdl",
                "Common/spare.skl",
                "Common/skin.dds",
                "Common/hair.FTEX",
                "Common/cloth.png",
            ]),
            None
        );
        // Any other kind, and any file deeper in the folder, is named.
        for file in ["Common/body.mtl", "Common/legs.model"] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
        // A file under a subfolder of `Common/` is validation's `common_file_disallowed`, which
        // drops it under the strict file-type check (so the gate never sees it) and keeps it
        // otherwise: then the gate names it.
        let (export, issues) = resolved_with_issues(
            "co Midcup Gate",
            &[(FACE[0], 1), (FACE[1], 1), ("Common/sub/x.dds", 1)],
            &[],
            None,
        );
        assert_eq!(issues, ["common_file_disallowed"]);
        assert!(export.export.common.is_empty());
        for (path, compiled) in [
            ("Common/sub/x.dds", false),
            ("Common/sub/x.fmdl", false),
            ("Common/x.dds", true),
        ] {
            let path = ScopePath::new(path).unwrap();
            let file = FileDescriptor {
                size: 0,
                kind: aesthetics_export::classify(path.name()),
                source: path.clone(),
                path,
            };
            assert_eq!(
                common_file_compiled(&file),
                compiled,
                "{}",
                file.path.as_str()
            );
        }
    }

    #[test]
    fn a_common_link_to_an_fmdl_is_a_model_a_texture_link_is_compiled_and_any_other_is_named() {
        let legs = "Common/legs.fmdl";
        // Alone, the link is the folder's model: no face needed with the link in `boots/`.
        assert_eq!(first_hit(&["Players/03 - A/legs.fmdl.common", legs]), None);
        assert_eq!(
            first_hit(&["Players/03 - A/boots/legs.fmdl.common", legs]),
            None
        );
        assert_eq!(
            first_hit(&["Players/03 - A/face/legs.fmdl.common.txt", legs]),
            None
        );
        // Beside a local model of its package, with the face's files, and with a skeleton of
        // the model's stem in Common.
        assert_eq!(
            gate(&[
                "Players/03 - A/torso.fmdl",
                "Players/03 - A/legs.fmdl.common",
                legs,
                "Common/legs.skl",
                "Common/cloth.dds",
            ]),
            None
        );
        // A boots link beside a shared boots link combines, as a local boots model would.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/kit_boots.fmdl.common",
                "Players/03 - A/Crocs.boots",
                "Boots/Crocs/boots.fmdl",
                "Common/kit_boots.fmdl",
            ]),
            None
        );
        let folder = folder(&["kit_boots.fmdl.common"]);
        assert!(holds_model(&folder, ModelPackage::Boots));
        assert!(!holds_model(&folder, ModelPackage::Face));
        // A link to a texture is compiled beside a model, in the folder or a reserved
        // subfolder, and alone, as a folder of textures is.
        let hair = "Common/hair.dds";
        for link in ["hair.dds.common", "boots/hair.dds.common"] {
            assert_eq!(
                gate(&[&format!("Players/03 - A/{link}"), hair]),
                None,
                "{link}"
            );
        }
        assert_eq!(first_hit(&["Players/03 - A/hair.dds.common", hair]), None);
        // A link to a material file is named; so is a glove link that gives no side, and a
        // `.skl` beside the link, which pairs with no model of the folder's.
        for (link, target) in [
            ("body.mtl.common", Some("Common/body.mtl")),
            ("materials.toml.common", Some("Common/materials.toml")),
            ("gloves/legs.fmdl.common", None),
            ("legs.skl", None),
        ] {
            let file = format!("Players/03 - A/{link}");
            let mut files = vec![file.as_str(), "Players/03 - A/legs.fmdl.common", legs];
            files.extend(target);
            assert_eq!(gate(&files), what(&file), "{link}");
        }
        // A shared folder's link (kept by a non-strict file-type check) resolves from no
        // player folder: named.
        for link in ["legs.fmdl.common", "hair.dds.common"] {
            assert_eq!(
                shared_hit(SharedKind::Boots, &["boots.fmdl", link]),
                what(&format!("Boots/Crocs/{link}")),
                "{link}"
            );
        }
    }

    #[test]
    fn the_first_hit_follows_players_files_links_shared_folders_kits_then_the_rest() {
        let face_high = "Players/03 - A/face_high.fmdl";
        // In each place, something the gate names there: a skeleton pairing with no model,
        // a kit texture the config has no field for, a collar (not emitted yet).
        let loose_skl = "Players/03 - A/torso.skl";
        let face_link = "Players/03 - A/Round.face";
        let shared_face = "Faces/Round/face_high.fmdl";
        let shared_face_skl = "Faces/Round/fcl_hair.skl";
        let link = "Players/03 - A/Crocs.boots";
        let shared = "Boots/Crocs/boots.fmdl";
        let shared_skl = "Boots/Crocs/kit_boots.skl";
        let kit = "Kits/g1/kit.dds";
        let kit_extra = "Kits/g1/kit_srm.dds";
        let collar = "Collars/collar.dds";
        // A folder's own files come before its links' folders.
        assert_eq!(
            first_hit(&[
                face_high,
                loose_skl,
                face_link,
                shared_face,
                shared_face_skl,
                link,
                shared,
                shared_skl,
                kit,
                kit_extra,
                collar
            ]),
            what(loose_skl)
        );
        assert_eq!(
            first_hit(&[
                face_high,
                face_link,
                shared_face,
                shared_face_skl,
                link,
                shared,
                shared_skl,
                kit,
                kit_extra,
                collar
            ]),
            what(shared_face_skl)
        );
        assert_eq!(
            first_hit(&[face_high, link, shared, shared_skl, kit, kit_extra, collar]),
            what(shared_skl)
        );
        assert_eq!(
            gate(&[link, shared, kit, kit_extra, collar]),
            what(kit_extra)
        );
        assert_eq!(gate(&[collar]), what(collar));
    }

    #[test]
    fn what_is_never_emitted_is_not_counted() {
        // `fpc_off` and `settings.toml` go to the savefile; a kit's colors and icon and the
        // root's metadata emit nothing yet.
        assert_eq!(
            gate(&[
                "Players/03 - A/fpc_off",
                "Players/03 - A/settings.toml",
                "Kits/g1/kit.dds",
                "Kits/g1/colors.txt",
                "Kits/g1/icon_11",
                "colors.txt",
            ]),
            None
        );

        // A folder no roster slot maps, and an `all/` no kit inherits from, emit nothing.
        let (export, issues) = resolved_with_issues(
            "co Midcup Gate",
            &[
                ("Players/A/face_high.fmdl", 1),
                ("Players/A/face_diff.bin", 1),
                ("Players/Unlisted/torso.fmdl", 1),
                ("Kits/all/kit_back.png", 1),
            ],
            &[],
            Some(b"03 A\n"),
        );
        assert_eq!(issues, ["player_unlisted", "kit_all_unused"]);
        assert_eq!(first_not_compiled(&export, PesVersion::Pes21), None);
    }

    #[test]
    fn a_kit_mask_is_dropped_on_a_fox_target_before_the_gate_and_the_task() {
        let files: Vec<(&str, u64)> = FACE
            .iter()
            .chain(&[
                "Kits/g1/config.toml",
                "Kits/g1/kit.dds",
                "Kits/g1/kit_mask.dds",
            ])
            .map(|path| (*path, 1))
            .collect();
        let export = resolved("co Midcup Gate", &files, &[], None);

        let report = plan_run(
            vec![(ExportId(0), export, two_team_colors(), None)],
            PesVersion::Pes21,
        );

        // The drop is reported once, on the kit, naming the file.
        let [message] = report.messages.as_slice() else {
            panic!("{:?}", report.messages);
        };
        assert_eq!(message.code.code, "kit_texture_not_used");
        assert_eq!(
            message.scope,
            studio_core::Scope::Folder {
                export_id: ExportId(0),
                path: ScopePath::new("Kits/g1").unwrap(),
            }
        );
        assert_eq!(message.disposition, studio_core::Disposition::DropFile);
        assert_eq!(
            message.context,
            [("file".to_owned(), "kit_mask.dds".to_owned())]
        );
        let TaskKind::Kit { kit, .. } = &report.manifest.tasks[1].kind else {
            panic!("the second task is the kit's");
        };
        let stems: Vec<&str> = kit
            .textures
            .iter()
            .map(|texture| texture.stem.as_str())
            .collect();
        assert_eq!(stems, ["kit"]);
        assert_eq!(
            report.manifest.tasks[1].charge, 2,
            "the config and the kit are read, the mask is not"
        );
    }

    /// The folder `Players/03 - A` holding the files `names`.
    fn folder(names: &[&str]) -> PlayerFolder {
        let files = names
            .iter()
            .map(|name| {
                let path = ScopePath::new(&format!("Players/03 - A/{name}")).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect();
        PlayerFolder {
            path: ScopePath::new("Players/03 - A").unwrap(),
            player_name: "A".to_owned(),
            files,
            links: Vec::new(),
            ingame_face: false,
            fpc: None,
            portrait: None,
            settings: None,
        }
    }

    /// The role of each file of `names` in `Players/03 - A`.
    fn roles(names: &[&str]) -> Vec<Option<PlayerFile>> {
        let folder = folder(names);
        let models = FolderModels::of(&folder.path, &folder.files);
        folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect()
    }

    /// The role of each file of `names` in `Players/03 - A` holding `ingame_face` too.
    fn marked_roles(names: &[&str]) -> Vec<Option<PlayerFile>> {
        let folder = PlayerFolder {
            ingame_face: true,
            ..folder(names)
        };
        let models = FolderModels::of_player(&folder);
        folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect()
    }

    /// The role of the file `name` in a folder holding it beside `fcl_hair.fmdl` and
    /// `kit_boots.fmdl`.
    fn role(name: &str) -> Option<PlayerFile> {
        roles(&["fcl_hair.fmdl", "kit_boots.fmdl", name])
            .pop()
            .unwrap()
    }

    fn model(package: ModelPackage, name: &'static str) -> Option<PlayerFile> {
        Some(PlayerFile::Model { package, name })
    }

    fn packed(package: ModelPackage, name: &'static str) -> Option<PlayerFile> {
        Some(PlayerFile::Packed { package, name })
    }

    fn skeleton(package: ModelPackage, name: &'static str) -> Option<PlayerFile> {
        Some(PlayerFile::Skeleton { package, name })
    }

    #[test]
    fn each_model_goes_to_its_package_under_its_allowed_name() {
        for (name, package, allowed) in [
            ("face_high.fmdl", ModelPackage::Face, "face_high"),
            ("hair_high.fmdl", ModelPackage::Face, "hair_high"),
            ("oral.fmdl", ModelPackage::Face, "oral"),
            ("fcl_hair.fmdl", ModelPackage::Face, "fcl_hair"),
            ("x_fcl_hair.fmdl", ModelPackage::Face, "fcl_hair"),
            ("boots.fmdl", ModelPackage::Boots, "boots"),
            ("kit_boots.fmdl", ModelPackage::Boots, "boots"),
            ("Kit_BOOTS.fmdl", ModelPackage::Boots, "boots"),
            ("glove_l.fmdl", ModelPackage::Gloves, "glove_l"),
            ("gloveL.fmdl", ModelPackage::Gloves, "glove_l"),
            ("keeper_gloveR.fmdl", ModelPackage::Gloves, "glove_r"),
            ("glove_r.fmdl", ModelPackage::Gloves, "glove_r"),
            ("handL.fmdl", ModelPackage::Gloves, "glove_l"),
            ("handR.fmdl", ModelPackage::Gloves, "glove_r"),
            // No recognized suffix: face content the hair merge takes.
            ("torso.fmdl", ModelPackage::Face, "fcl_hair"),
            ("myboots.fmdl", ModelPackage::Face, "fcl_hair"),
        ] {
            assert_eq!(roles(&[name]), [model(package, allowed)], "{name}");
        }
    }

    #[test]
    fn each_other_player_file_has_its_role_or_none() {
        assert_eq!(
            role("skin.dds"),
            Some(PlayerFile::Texture("skin".to_owned(), SourceFormat::Dds))
        );
        assert_eq!(
            role("skin.ftex"),
            Some(PlayerFile::Texture("skin".to_owned(), SourceFormat::Ftex))
        );
        assert_eq!(
            role("skin.png"),
            Some(PlayerFile::Texture("skin".to_owned(), SourceFormat::Png))
        );
        assert_eq!(
            role("fcl_hair.skl"),
            skeleton(ModelPackage::Face, "fcl_hair_sim.skl")
        );
        assert_eq!(
            role("kit_boots.skl"),
            skeleton(ModelPackage::Boots, "boots.skl")
        );
        assert_eq!(
            role("face_diff.bin"),
            packed(ModelPackage::Face, "face_diff.bin")
        );
        assert_eq!(
            role("Face_Diff.bin"),
            packed(ModelPackage::Face, "face_diff.bin")
        );
        assert_eq!(
            role("fcl_hair_sim.fclo"),
            packed(ModelPackage::Face, "fcl_hair_sim.fclo")
        );
        assert_eq!(role("face_diff.xml"), Some(PlayerFile::FaceDiffXml));
        assert_eq!(
            role("hair.png.common"),
            Some(PlayerFile::CommonTexture("hair".to_owned()))
        );
        for refused in [
            "face_high.model",
            "torso.model",
            "boots.skl",
            "glove_l.skl",
            "face.xml",
            "face_diff2.xml",
            "body.mtl.common",
            "face_diff2.bin",
            "fcl_hair.fclo",
            "other/face_high.fmdl",
            "face/deep/face_high.fmdl",
        ] {
            assert_eq!(role(refused), None, "{refused}");
        }
    }

    #[test]
    fn a_reserved_subfolder_forces_its_category_on_its_models() {
        // `face/`: a face name keeps its slot, any other model is hair content.
        for (name, allowed) in [
            ("face/face_high.fmdl", "face_high"),
            ("face/x_hair_high.fmdl", "hair_high"),
            ("FACE/oral.fmdl", "oral"),
            ("face/fcl_hair.fmdl", "fcl_hair"),
            ("face/torso.fmdl", "fcl_hair"),
            ("face/boots.fmdl", "fcl_hair"),
            ("face/glove_l.fmdl", "fcl_hair"),
        ] {
            assert_eq!(
                roles(&[name]),
                [model(ModelPackage::Face, allowed)],
                "{name}"
            );
        }
        // `boots/`: every model is the boots, whatever its name.
        for name in [
            "boots/boots.fmdl",
            "boots/hair_high.fmdl",
            "Boots/torso.fmdl",
            "boots/glove_l.fmdl",
        ] {
            assert_eq!(
                roles(&[name]),
                [model(ModelPackage::Boots, "boots")],
                "{name}"
            );
        }
        // `gloves/`: the side comes from the suffix; a model that gives none has no role.
        for (name, allowed) in [
            ("gloves/glove_l.fmdl", "glove_l"),
            ("gloves/gloveL.fmdl", "glove_l"),
            ("Gloves/keeper_handR.fmdl", "glove_r"),
            ("gloves/glove_r.fmdl", "glove_r"),
        ] {
            assert_eq!(
                roles(&[name]),
                [model(ModelPackage::Gloves, allowed)],
                "{name}"
            );
        }
        for refused in [
            "gloves/keeper.fmdl",
            "gloves/boots.fmdl",
            "gloves/face_high.fmdl",
            "common/boots.fmdl",
        ] {
            assert_eq!(roles(&[refused]), [None], "{refused}");
        }
        // Textures are the player's own from any of the four, `common/` included.
        for (name, expected) in [
            ("common/skin.dds", "skin"),
            ("COMMON/skin.FTEX", "skin"),
            ("boots/sole.dds", "sole"),
        ] {
            let roles = roles(&[name]);
            let [Some(PlayerFile::Texture(stem, _))] = roles.as_slice() else {
                panic!("{name}");
            };
            assert_eq!(stem, expected, "{name}");
        }
    }

    #[test]
    fn a_common_link_takes_the_role_the_linked_model_would_have_in_its_place() {
        for (link, package, allowed) in [
            ("legs.fmdl.common", ModelPackage::Face, "fcl_hair"),
            ("legs.FMDL.common.txt", ModelPackage::Face, "fcl_hair"),
            ("x_face_high.fmdl.common", ModelPackage::Face, "face_high"),
            ("kit_boots.fmdl.common", ModelPackage::Boots, "boots"),
            ("boots/legs.fmdl.common", ModelPackage::Boots, "boots"),
            ("face/kit_boots.fmdl.common", ModelPackage::Face, "fcl_hair"),
            ("gloves/handL.fmdl.common", ModelPackage::Gloves, "glove_l"),
        ] {
            assert_eq!(
                roles(&[link]),
                [Some(PlayerFile::CommonModel {
                    package,
                    name: allowed
                })],
                "{link}"
            );
        }
        for refused in [
            "materials.toml.common",
            "body.mtl.common",
            "legs.model.common",
            "gloves/legs.fmdl.common",
            "common/legs.fmdl.common",
            "other/legs.fmdl.common",
        ] {
            assert_eq!(roles(&[refused]), [None], "{refused}");
        }
        // The link is a face model for the face's files, but pairs no skeleton of the
        // folder's: the Common model's skeleton is Common's.
        assert_eq!(
            roles(&["legs.fmdl.common", "face_diff.bin", "legs.skl"]),
            [
                Some(PlayerFile::CommonModel {
                    package: ModelPackage::Face,
                    name: "fcl_hair"
                }),
                packed(ModelPackage::Face, "face_diff.bin"),
                None
            ]
        );
        assert_eq!(
            roles(&["kit_boots.fmdl.common", "face_diff.bin"]),
            [
                Some(PlayerFile::CommonModel {
                    package: ModelPackage::Boots,
                    name: "boots"
                }),
                Some(PlayerFile::UnusedFaceFile)
            ]
        );
        assert_eq!(
            PlayerFile::CommonModel {
                package: ModelPackage::Gloves,
                name: "glove_l"
            }
            .package(),
            Some(ModelPackage::Gloves)
        );
    }

    #[test]
    fn a_common_link_to_a_texture_stands_for_its_stem_and_a_link_to_a_material_file_has_none() {
        let texture = |stem: &str| Some(PlayerFile::CommonTexture(stem.to_owned()));
        assert_eq!(
            roles(&[
                "hair.dds.common",
                "skin_nrm.png.common",
                "body.mtl.common",
                "materials.toml.common",
                "legs.fmdl.common",
            ]),
            [
                texture("hair"),
                texture("skin_nrm"),
                None,
                None,
                Some(PlayerFile::CommonModel {
                    package: ModelPackage::Face,
                    name: "fcl_hair"
                }),
            ]
        );
        // Wherever validation resolves a link: with the tolerated `.txt` tail, and in the
        // three reserved subfolders that hold models. In `common/` a link is not resolved
        // (`file_type_disallowed`), so it has no role.
        for (link, stem) in [
            ("Hair.DDS.common.txt", "Hair"),
            ("face/hair.ftex.common", "hair"),
            ("boots/sole.tga.common", "sole"),
            ("gloves/grip.dds.common", "grip"),
        ] {
            assert_eq!(roles(&[link]), [texture(stem)], "{link}");
        }
        for refused in [
            "common/hair.dds.common",
            "hair.gif.common",
            "body.materials.toml.common",
        ] {
            assert_eq!(roles(&[refused]), [None], "{refused}");
        }
        assert_eq!(PlayerFile::CommonTexture("hair".to_owned()).package(), None);
    }

    #[test]
    fn only_the_hair_and_the_boots_have_a_skeleton_slot() {
        assert_eq!(
            skeleton_slot(ModelPackage::Face, "fcl_hair"),
            Some("fcl_hair_sim.skl")
        );
        assert_eq!(
            skeleton_slot(ModelPackage::Boots, "boots"),
            Some("boots.skl")
        );
        for name in ["face_high", "hair_high", "oral"] {
            assert_eq!(skeleton_slot(ModelPackage::Face, name), None, "{name}");
        }
        assert_eq!(skeleton_slot(ModelPackage::Gloves, "glove_l"), None);
    }

    #[test]
    fn a_common_file_is_found_directly_in_common_by_folded_name() {
        let common: Vec<FileDescriptor> =
            ["Common/Legs.fmdl", "Common/legs.skl", "Common/sub/x.fmdl"]
                .iter()
                .map(|path| {
                    let path = ScopePath::new(path).unwrap();
                    FileDescriptor {
                        size: 0,
                        kind: aesthetics_export::classify(path.name()),
                        source: path.clone(),
                        path,
                    }
                })
                .collect();
        assert_eq!(
            common_file(&common, "legs.fmdl").map(|file| file.path.as_str()),
            Some("Common/Legs.fmdl")
        );
        assert_eq!(common_file(&common, "x.fmdl"), None);
        assert_eq!(
            common_skeleton(&common, "LEGS.fmdl").map(|file| file.path.as_str()),
            Some("Common/legs.skl")
        );
        assert_eq!(common_skeleton(&common, "torso.fmdl"), None);
    }

    #[test]
    fn a_subfolder_s_skeleton_pairs_with_the_model_beside_it_under_the_forced_category_s_slot() {
        // The skeleton is named after the model in its own directory: the root's `kit_boots.skl`
        // does not pair with `boots/kit_boots.fmdl`, nor the other way round.
        assert_eq!(
            roles(&[
                "boots/kit_boots.fmdl",
                "boots/kit_boots.skl",
                "kit_boots.skl"
            ]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl"),
                None
            ]
        );
        assert_eq!(
            roles(&["kit_boots.fmdl", "boots/kit_boots.skl"]),
            [model(ModelPackage::Boots, "boots"), None]
        );
        // The slot follows the model's forced name: a `hair_high` in `boots/` is the boots,
        // so its skeleton is the boots'; a `face_high` in `face/` still has no slot.
        assert_eq!(
            roles(&["boots/hair_high.fmdl", "boots/hair_high.skl"]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl")
            ]
        );
        assert_eq!(
            roles(&["face/torso.fmdl", "face/torso.skl"]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl")
            ]
        );
        assert_eq!(
            roles(&["face/face_high.fmdl", "face/face_high.skl"]),
            [
                model(ModelPackage::Face, "face_high"),
                Some(PlayerFile::SlotlessSkeleton)
            ]
        );
        // The face's files may sit in `face/`, given a face model anywhere in the folder, and
        // nowhere else.
        assert_eq!(
            roles(&[
                "face/face_diff.bin",
                "face/fcl_hair_sim.fclo",
                "face/torso.fmdl",
                "boots/face_diff.bin"
            ]),
            [
                packed(ModelPackage::Face, "face_diff.bin"),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo"),
                model(ModelPackage::Face, "fcl_hair"),
                None
            ]
        );
        assert_eq!(
            roles(&[
                "face/face_diff.xml",
                "face/torso.fmdl",
                "boots/face_diff.xml"
            ]),
            [
                Some(PlayerFile::FaceDiffXml),
                model(ModelPackage::Face, "fcl_hair"),
                None
            ]
        );
        assert_eq!(
            roles(&["face_diff.bin", "boots/boots.fmdl"]),
            [
                Some(PlayerFile::UnusedFaceFile),
                model(ModelPackage::Boots, "boots")
            ]
        );
    }

    #[test]
    fn a_skeleton_pairs_with_the_model_of_its_name_and_face_files_need_a_face_model() {
        // The skeleton is named after the model, whatever the model's free prefix.
        assert_eq!(
            roles(&["x_fcl_hair.fmdl", "x_fcl_hair.skl", "fcl_hair.skl"]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl"),
                None
            ]
        );
        assert_eq!(
            roles(&["boots.fmdl", "boots.skl", "kit_boots.skl"]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl"),
                None
            ]
        );
        // Every boots part pairs its own skeleton, not only the first, and so does every hair
        // part.
        assert_eq!(
            roles(&[
                "a_boots.fmdl",
                "a_boots.skl",
                "kit_boots.fmdl",
                "kit_boots.skl"
            ]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl"),
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl")
            ]
        );
        assert_eq!(
            roles(&[
                "a_fcl_hair.fmdl",
                "a_fcl_hair.skl",
                "fcl_hair.fmdl",
                "fcl_hair.skl"
            ]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl"),
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl")
            ]
        );
        // An unsuffixed model is a hair part: its skeleton is the hair's.
        assert_eq!(
            roles(&["torso.fmdl", "torso.skl"]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl")
            ]
        );
        // A skeleton named after a face model with no skeleton slot pairs with nothing to
        // pack, whatever the model's free prefix.
        assert_eq!(
            roles(&[
                "face_high.fmdl",
                "face_high.skl",
                "x_hair_high.fmdl",
                "x_hair_high.skl",
                "oral.fmdl",
                "oral.skl"
            ]),
            [
                model(ModelPackage::Face, "face_high"),
                Some(PlayerFile::SlotlessSkeleton),
                model(ModelPackage::Face, "hair_high"),
                Some(PlayerFile::SlotlessSkeleton),
                model(ModelPackage::Face, "oral"),
                Some(PlayerFile::SlotlessSkeleton)
            ]
        );
        // Without a face model the face's files are not used, unless a shared face is
        // linked: then the shared face is the player's. An unsuffixed model is a face model.
        assert_eq!(
            roles(&["boots.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]),
            [
                model(ModelPackage::Boots, "boots"),
                Some(PlayerFile::UnusedFaceFile),
                Some(PlayerFile::UnusedFaceFile)
            ]
        );
        assert_eq!(
            roles(&["torso.fmdl", "face_diff.bin"]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                packed(ModelPackage::Face, "face_diff.bin")
            ]
        );
        let folder = folder(&["boots.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]);
        let models = FolderModels::of(&folder.path, &folder.files).with_linked_face();
        let linked: Vec<Option<PlayerFile>> = folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect();
        assert_eq!(
            linked,
            [
                model(ModelPackage::Boots, "boots"),
                packed(ModelPackage::Face, "face_diff.bin"),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo")
            ]
        );
        assert_eq!(
            roles(&["face_high.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]),
            [
                model(ModelPackage::Face, "face_high"),
                packed(ModelPackage::Face, "face_diff.bin"),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo")
            ]
        );
    }

    #[test]
    fn under_ingame_face_a_model_the_hair_would_take_is_the_boots_with_its_skeleton() {
        let boots = || model(ModelPackage::Boots, "boots");
        assert_eq!(
            marked_roles(&[
                "torso.fmdl",
                "torso.skl",
                "fcl_hair.fmdl",
                "face/hat.fmdl",
                "legs.fmdl.common",
                "glove_l.fmdl",
                "face_diff.bin",
                "face_diff.xml",
                "fcl_hair_sim.fclo",
            ]),
            [
                boots(),
                skeleton(ModelPackage::Boots, "boots.skl"),
                boots(),
                boots(),
                Some(PlayerFile::CommonModel {
                    package: ModelPackage::Boots,
                    name: "boots"
                }),
                model(ModelPackage::Gloves, "glove_l"),
                Some(PlayerFile::UnusedFaceFile),
                Some(PlayerFile::UnusedFaceFile),
                Some(PlayerFile::UnusedFaceFile),
            ]
        );
    }

    #[test]
    fn a_face_file_is_not_used_without_a_face_model_and_packed_with_one() {
        let unused = || Some(PlayerFile::UnusedFaceFile);
        assert_eq!(
            roles(&[
                "boots.fmdl",
                "face_diff.bin",
                "face/face_diff.xml",
                "fcl_hair_sim.fclo"
            ]),
            [
                model(ModelPackage::Boots, "boots"),
                unused(),
                unused(),
                unused()
            ]
        );
        assert_eq!(
            roles(&[
                "face_high.fmdl",
                "face_diff.bin",
                "face/face_diff.xml",
                "fcl_hair_sim.fclo"
            ]),
            [
                model(ModelPackage::Face, "face_high"),
                packed(ModelPackage::Face, "face_diff.bin"),
                Some(PlayerFile::FaceDiffXml),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo")
            ]
        );
        assert_eq!(PlayerFile::UnusedFaceFile.package(), None);
    }

    #[test]
    fn a_package_is_the_models_and_packed_files_but_not_the_textures() {
        let texture = PlayerFile::Texture("skin".to_owned(), SourceFormat::Dds);
        assert_eq!(texture.package(), None);
        assert_eq!(PlayerFile::SlotlessSkeleton.package(), None);
        assert_eq!(
            skeleton(ModelPackage::Boots, "boots.skl")
                .unwrap()
                .package(),
            Some(ModelPackage::Boots)
        );
        assert_eq!(
            packed(ModelPackage::Face, "face_diff.bin")
                .unwrap()
                .package(),
            Some(ModelPackage::Face)
        );
        assert_eq!(
            model(ModelPackage::Gloves, "glove_l").unwrap().package(),
            Some(ModelPackage::Gloves)
        );
        assert_eq!(
            ModelPackage::ALL.map(ModelPackage::file_stem),
            ["face", "boots", "glove"]
        );
        assert_eq!(
            ModelPackage::ALL.map(ModelPackage::name),
            ["face", "boots", "gloves"]
        );
    }

    #[test]
    fn a_texture_format_is_its_last_extension_in_any_case() {
        assert_eq!(texture_format("kit.dds"), Some(SourceFormat::Dds));
        assert_eq!(texture_format("kit.DDS"), Some(SourceFormat::Dds));
        assert_eq!(texture_format("kit.FTEX"), Some(SourceFormat::Ftex));
        assert_eq!(texture_format("kit.png"), Some(SourceFormat::Png));
        assert_eq!(texture_format("kit.Tga"), Some(SourceFormat::Tga));
        assert_eq!(texture_format("kit.dds.png"), Some(SourceFormat::Png));
        for refused in ["kit", "kit.gif", "kit.dds."] {
            assert_eq!(texture_format(refused), None, "{refused}");
        }
    }

    #[test]
    fn a_stem_is_the_name_up_to_its_last_dot() {
        assert_eq!(file_stem("hair.png.common"), "hair.png");
        assert_eq!(file_stem("shirt.dds"), "shirt");
        assert_eq!(file_stem("NO_USE"), "NO_USE");
    }
}
