//! What `compile` builds so far, and the gate that skips any export holding anything else
//! (`team_compiler/README.md` "Phase 3 scope", narrowed step by step through Phase 4): one
//! classification, which planning reads to skip an export and processing reads for each
//! file's role, so the two never disagree. The gate is withdrawn when `compile` builds
//! everything an export holds; the classification stays.

use aesthetics_export::{
    ExportIdentity, FileDescriptor, FileKind, ModelFormat, ModelSuffix, PlayerFolder,
    ResolvedAestheticsExport, SharedKind, SharedLink, SharedModelFolder, ValidatedAestheticsExport,
    ValidatedRoster, classify, common_link_name, kit_token, model_suffix,
};
use dds_convert::SourceFormat;
use pes_version::{Engine, PesVersion};
use vtree::ScopePath;

use super::ids::shared_folders_taking_ids;
use super::mapped_players;
use crate::face_xml;
use crate::kit_variants::model_variant_sets;

/// The kit texture stems `compile` builds: first the five the kit config names, in the order
/// of its texture-name fields, then the two maps the games read by name convention alone,
/// `kit_mask` (PES 15-17) and `kit_srm` (PES 18-21). Planning drops the map the target's
/// engine does not read before the gate asks (`pipeline.md` "4. Per-export non-model steps",
/// Kits: mask and srm are engine-specific).
pub(crate) const KIT_TEXTURE_STEMS: [&str; 7] = [
    "kit",
    "kit_back",
    "kit_chest",
    "kit_leg",
    "kit_name",
    "kit_mask",
    "kit_srm",
];

/// The source format of the texture file `name`, by its extension in any case: one of the
/// image formats `dds_convert` converts (`libs/dds_convert.md` "Accepted image formats");
/// `None` for a name without one.
pub(crate) fn texture_format(name: &str) -> Option<SourceFormat> {
    let (_, extension) = name.rsplit_once('.')?;
    SourceFormat::from_extension(extension)
}

/// The three Fox packages a player folder's models go to, each a `.fpk` and `.fpkd` pair in
/// its own game folder: the face by player id, the boots and the gloves by the player's
/// planned model id, all three of a referee's by his slot (`paths::PackageKey`).
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

/// Whether `player`'s `link` combines on a target of `engine`: the shared folder's models
/// become parts of the player's own package instead of the shared output being loaded as it
/// is. A face link always does, on both engines, a shared face having no output of its own: on
/// Fox it is merged into the player's face, on pre-Fox copied into his face CPK. A boots or
/// gloves link does when the player holds a model of that package (`holds_model`;
/// `player_folders.md` "A link plus local models combines"): on Fox any such model, one
/// `ingame_face` makes a boots part included; on pre-Fox only a part `ingame_face` gives him
/// (`player_folders.md` "`ingame_face` with shared links"), since without the marker his boots
/// and gloves models are parts of his face, and the link keeps its plain meaning.
pub(crate) fn link_combines(player: &PlayerFolder, link: &SharedLink, engine: Engine) -> bool {
    match link.kind {
        SharedKind::Face => true,
        SharedKind::Boots | SharedKind::Gloves => {
            holds_model(player, package_of(link.kind), engine)
        }
    }
}

/// Whether the shared folder `player`'s `link` names is built, for `engine`, into the
/// player's own package of its kind instead of compiled on its own under a shared id;
/// `roster` is the roster of the player's export. A referee's every link is: he has no team
/// block to give the shared folder an id of its own, so it is written as his slot's
/// `k99NN`/`g99NN` (`blue_port.md` "Referee export processing"). A team player's link is
/// when it combines (`link_combines`).
pub(crate) fn link_feeds_own_package(
    roster: &ValidatedRoster,
    engine: Engine,
    player: &PlayerFolder,
    link: &SharedLink,
) -> bool {
    match roster {
        ValidatedRoster::Referees(_) => true,
        ValidatedRoster::Team(_) => link_combines(player, link, engine),
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

/// What one file of a player folder becomes in the output of the target's engine.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PlayerFile {
    /// A model, packed into `package` as `<name>.fmdl` with its texture paths rewritten:
    /// `name` is its allowed name, the free prefix stripped (`kit_boots.fmdl` packs as
    /// `boots.fmdl`). A `.model` with no `.fmdl` of its path stem beside it takes the role too:
    /// the Models task converts it to an FMDL with the `.mtl` its search finds (`pipeline.md`
    /// step 3 "Format conversion").
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
    /// Pre-Fox: a `.model`, packed into the player's face CPK as `oral_<stem>_win32.model`
    /// and listed in its generated `face.xml` under `xml_type` (`face_xml::xml_type`; `parts`
    /// in `boots/`, its hand's type in `gloves/`). Boots and gloves models included: the
    /// typed XML is why a pre-Fox player needs no boots or gloves folder of his own. An `.fmdl`
    /// with no `.model` of its stem beside it takes the role too: the face task converts it to
    /// a `.model` and its material set, packed as `<stem>.mtl` (`pipeline.md` step 3 "Format
    /// conversion").
    PreFoxModel {
        /// The model's `face.xml` type.
        xml_type: String,
    },
    /// Pre-Fox, in a player folder holding `ingame_face`: a `.model` that is a part of the
    /// player's own `package`, the one Fox gives it under the marker (`model_role`), since the
    /// marker means no face and so no `face.xml` to list it. The boots take every model but a
    /// glove, a model the face would take included, and are written as one `boots.model`, its
    /// parts merged; the gloves are written unmerged, each model under its own name and listed
    /// in the gloves' `glove.xml` (`player_folders.md` "`ingame_face` marker"). The folder's
    /// `.mtl` files go into each of the two, which packs the ones its parts use
    /// (`TaskKind::files`). A `.common` link to a `.model` takes this role too, as a `.model`
    /// of the linked stem would in its place: with no `face.xml` to name the Common path, the
    /// Common model is copied in as the part, planning putting it in the link's place with the
    /// `.mtl` its search finds (`ModelFolder::roles`; `player_folders.md` "`ingame_face` with
    /// shared links").
    PreFoxPart {
        /// The package the model is a part of.
        package: ModelPackage,
        /// The type a `glove.xml` lists the model under, as it types a shared gloves folder's
        /// (`pre_fox_model_type`). The boots list no model, so a part of theirs leaves it
        /// unread.
        xml_type: String,
    },
    /// A `.mtl` outside `common/`. On pre-Fox it is packed into the player's face CPK under its
    /// own name with its texture paths pointed at the player's common folder, and the
    /// `face.xml` names it for each model whose search finds it (`mtl_search::mtl_for`). On Fox
    /// it is packed nowhere: a Models task converting a `.model` reads the folder's `.mtl`
    /// files (`TaskKind::files`) and gives the model the one its search finds, so one beside
    /// only FMDLs, a `.model` an FMDL beats included, is read by nothing.
    Material,
    /// Pre-Fox: a `.common` link to a `.model` (`legs.model.common` → `Common/legs.model`),
    /// listed in the player's generated `face.xml` under `xml_type`, the type a `.model` of
    /// the linked stem would have in the link's place, at the team's Common output, where the
    /// export's Common models task packs the Common model once. The game loads it from there
    /// (`player_folders.md` "Common model links and model merging"), so nothing is packed into
    /// the face and nothing reads the empty link. Under `ingame_face` such a link is a
    /// `PreFoxPart` instead.
    PreFoxCommonModel {
        /// The linked model's `face.xml` type.
        xml_type: String,
    },
    /// Pre-Fox: a `.common` link to a `.mtl` (`body.mtl.common` → `Common/body.mtl`), which
    /// the `.mtl` search counts as a `.mtl` of the linked name in the link's folder
    /// (`mtl_search::mtl_for`). A model whose search finds it names the Common `.mtl` in the
    /// team's Common output; nothing reads the empty link. Under `ingame_face` the Common
    /// `.mtl` is also copied into his boots and gloves, which pack it when a part of theirs
    /// uses it (`ModelFolder::roles`).
    CommonMaterial,
    /// Pre-Fox: a member's own `face.xml`, directly in the folder or in `face/`, the authority
    /// on what the face loads in place of the one the compiler generates (`messages.md`
    /// "User-supplied `face.xml`"): the deep pass reads and checks it (`user_face_xml`). Its
    /// folder has a face whatever models it holds (`FolderModels::of_player_files`).
    FaceXml,
    /// Pre-Fox: the `.skl` paired with an `.fmdl` the face converts (`PreFoxModel`), by the
    /// path stem a Fox skeleton pairs by: the conversion's bind pose
    /// (`NativeModelBundle::Fox { skl }`), packed nowhere.
    ConversionSkeleton,
    /// A `.glb` or `.gltf` model with no model of the target's own format of its path stem
    /// beside it: the selected representation of its stem, which beats a model of the other
    /// engine's format beside it (`pipeline.md` step 3 "Format conversion"). The compiler does
    /// not read glTF until Phase 7, so planning drops the player folder holding one
    /// (`model_gltf_unsupported`) rather than compile the other format in its place: the
    /// same export never compiles differently once glTF is read.
    UnsupportedGltf,
}

impl PlayerFile {
    /// The package the file goes into; `None` for a texture, which goes to the player's
    /// common folder for every package to point at, for a `.common` link, whose file is the
    /// team's (Common's texture, `.model` or `.mtl`, packed once in the team's Common output),
    /// and for a skeleton with no slot, an unused face file, a left-out kit variant, a
    /// conversion's skeleton and a glTF, which go nowhere (the face task reads the
    /// conversion's skeleton, `TaskKind::files`; planning drops a glTF's folder). A Fox model
    /// link's role is its Common model's, and so is a pre-Fox one's under `ingame_face`, which
    /// planning puts in the link's place (`ModelFolder::roles`).
    pub(crate) fn package(&self) -> Option<ModelPackage> {
        match self {
            PlayerFile::Model { package, .. }
            | PlayerFile::CommonModel { package, .. }
            | PlayerFile::Packed { package, .. }
            | PlayerFile::Skeleton { package, .. }
            | PlayerFile::PreFoxPart { package, .. } => Some(*package),
            PlayerFile::FaceDiffXml
            | PlayerFile::PreFoxModel { .. }
            | PlayerFile::Material
            | PlayerFile::FaceXml => Some(ModelPackage::Face),
            PlayerFile::UnusedFaceFile
            | PlayerFile::SlotlessSkeleton
            | PlayerFile::LeftOutKitVariant
            | PlayerFile::Texture(..)
            | PlayerFile::CommonTexture(_)
            | PlayerFile::PreFoxCommonModel { .. }
            | PlayerFile::CommonMaterial
            | PlayerFile::ConversionSkeleton
            | PlayerFile::UnsupportedGltf => None,
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
/// converts; `None` for a link to anything else (a model, a material file).
fn linked_texture_stem(link_name: &str) -> Option<String> {
    let linked = common_link_name(link_name)?;
    texture_format(&linked)?;
    Some(file_stem(&linked).to_owned())
}

/// Whether `path` is a file directly in the export's `Common/` folder: the only place a link
/// resolves, and the only place `compile` reads a Common file from. Every other file of an
/// export sits at least two folders deep (`Players/05 - A/x.mtl`).
pub(crate) fn is_direct_common_file(path: &ScopePath) -> bool {
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

/// Whether `file` sits directly in the folder at `folder` or in its `face/`, where a face's
/// files go (`face_file`).
pub(crate) fn in_folder_or_face(folder: &ScopePath, file: &FileDescriptor) -> bool {
    matches!(
        position(folder, file),
        Some(Position::Direct | Position::Face)
    )
}

/// Whether `file` of the folder at `folder` is a member's own `face.xml`: named so in any case,
/// where a face's files go (`in_folder_or_face`). Pre-Fox reads it (`PlayerFile::FaceXml`);
/// Fox has no `face.xml` and ignores it (`xml_ignored_fox`).
pub(crate) fn is_user_face_xml(folder: &ScopePath, file: &FileDescriptor) -> bool {
    file.kind == FileKind::Xml
        && file.path.name().eq_ignore_ascii_case("face.xml")
        && in_folder_or_face(folder, file)
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

/// The `face.xml` type of a pre-Fox model with file stem `stem` at `position`
/// (`PlayerFile::PreFoxModel`): by its name directly in the folder or in `face/`; `parts` in
/// `boots/`, the type boots take; in `gloves/` its hand's type, so a glove whose name gives no
/// side has none, and `common/` holds no models. A per-kit model (one whose stem holds a kit
/// token) is typed as its set is, read without the token (`face_xml::xml_type`): `pants_kit1`
/// is `parts`, as `pants` is.
fn pre_fox_model_type(position: Position, stem: &str) -> Option<String> {
    match position {
        Position::Direct | Position::Face => Some(face_xml::xml_type(stem)),
        Position::Boots => Some("parts".to_owned()),
        Position::Gloves => match face_xml::suffix(stem) {
            Some(
                ModelSuffix::GloveL | ModelSuffix::GloveR | ModelSuffix::HandL | ModelSuffix::HandR,
            ) => Some(face_xml::xml_type(stem)),
            Some(
                ModelSuffix::FaceHigh
                | ModelSuffix::HairHigh
                | ModelSuffix::Oral
                | ModelSuffix::FclHair
                | ModelSuffix::Boots,
            )
            | None => None,
        },
        Position::Common => None,
    }
}

/// `file`'s export path up to its extension (`Players/05 - A/boots/kit_boots`): what a model
/// and the skeleton named after it share, and nothing in another directory does.
pub(crate) fn path_stem(file: &FileDescriptor) -> &str {
    file_stem(file.path.as_str())
}

/// What a player folder's models say about its other files, for the target engine they were
/// computed for: which have a package to go in, and which model a `.skl` pairs with. Computed
/// once per folder and passed to `player_file`, which gives the roles of that engine. A model
/// is listed by its `path_stem`, so a skeleton pairs with the model beside it, not with one of
/// the same name in another of the folder's directories.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FolderModels {
    /// The engine of the target the models were read for: each engine builds its own model
    /// format, so the same files give other models, and `player_file` other roles.
    engine: Engine,
    /// The player folder holds `ingame_face`, so its models take the roles `model_role` gives
    /// under the marker. A shared folder has none.
    ingame_face: bool,
    /// The folder's face files have a package to go in: it holds a face model, on pre-Fox its
    /// own `face.xml`, or links a shared face (`with_linked_face`).
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
    /// The path stems of its models in the target engine's own format, `.fmdl` files on Fox
    /// and `.model` files on pre-Fox, each of which beats a glTF or a model of the other format
    /// of its path stem (`pipeline.md` step 3 "Format conversion": target-native first), that
    /// model then having no role and no finding (TC-MOD-26).
    native_model_stems: Vec<String>,
    /// The path stems of its `.glb` and `.gltf` models, each of which beats a model of the
    /// other engine's format of its path stem and is beaten by one of the target's
    /// (`pipeline.md` step 3 "Format conversion": target-native first, then glTF).
    gltf_stems: Vec<String>,
    /// Pre-Fox, without `ingame_face`: the path stems of the `.fmdl` files the face converts
    /// (`PlayerFile::PreFoxModel`), each of whose skeletons is the conversion's bind pose
    /// (`PlayerFile::ConversionSkeleton`).
    converted_stems: Vec<String>,
}

impl FolderModels {
    /// The models among `files` of the folder at `folder`, a folder without `ingame_face`,
    /// for a target of `engine` (`of_player_files`).
    pub(crate) fn of(folder: &ScopePath, files: &[FileDescriptor], engine: Engine) -> FolderModels {
        FolderModels::of_player_files(folder, files, false, engine)
    }

    /// The models among `files` of the folder at `folder`, which holds `ingame_face` when
    /// `ingame_face` is set, for a target of `engine`. On Fox: its `.fmdl` files, its `.model`
    /// files with no `.fmdl` or glTF of their path stem, which the Models task converts, and its
    /// `.common` links to an FMDL, directly in it or in a reserved subfolder, each with its
    /// resolved role. A link counts as a model of its role's package, but pairs no skeleton of
    /// the folder's: a Common model's skeleton is Common's, resolved at planning
    /// (`common_skeleton`). A per-kit model with a lower variant of its set beside it is left
    /// out (`model_variant_sets`). On pre-Fox: its `.model` files, its `.fmdl` files with no
    /// `.model` or glTF of their path stem, which the face converts, and its `.common` links to a
    /// `.model` with a role, every one of them a part of the face (`PlayerFile::PreFoxModel`,
    /// `PlayerFile::PreFoxCommonModel`), every variant of a per-kit set included; only a
    /// converted FMDL pairs a skeleton, its bind pose; under `ingame_face` none, the folder
    /// having no face (`PlayerFile::PreFoxPart`).
    pub(crate) fn of_player_files(
        folder: &ScopePath,
        files: &[FileDescriptor],
        ingame_face: bool,
        engine: Engine,
    ) -> FolderModels {
        let mut models = FolderModels {
            engine,
            ingame_face,
            face: false,
            hair_stems: Vec::new(),
            boots_stems: Vec::new(),
            slotless_stems: Vec::new(),
            left_out_variants: match engine {
                Engine::Fox => model_variant_sets(files)
                    .into_iter()
                    .flat_map(|set| set.left_out)
                    .collect(),
                // Every variant is packed there, and the face task lists the set once.
                Engine::PreFox => Vec::new(),
            },
            // Read before the loop: on pre-Fox `boots.fmdl` sorts before the `boots.model`
            // or `boots.glb` beating it.
            native_model_stems: model_stems(folder, files, native_format(engine)),
            gltf_stems: model_stems(folder, files, ModelFormat::Gltf),
            converted_stems: Vec::new(),
        };
        for file in files {
            let Some(position) = position(folder, file) else {
                continue;
            };
            match engine {
                Engine::Fox => {}
                // Under the marker there is no face for any model to make (`PreFoxPart`), so
                // the face files are not used.
                Engine::PreFox if ingame_face => continue,
                Engine::PreFox => {
                    let name = file.path.name();
                    let model_link = file.kind == FileKind::CommonLink
                        && matches!(
                            pre_fox_link(position, name),
                            Some(PlayerFile::PreFoxCommonModel { .. })
                        );
                    let path_fold = vtree::fold_name(path_stem(file));
                    let converted =
                        file.kind == FileKind::Model(ModelFormat::Fmdl) && !models.beaten(file);
                    let typed = pre_fox_model_type(position, file_stem(name)).is_some();
                    if converted && typed {
                        models.converted_stems.push(path_fold);
                    }
                    // A folder holding its own `face.xml` has a face whatever models it
                    // holds: the xml may name only Common models (`messages.md`
                    // "User-supplied `face.xml`").
                    models.face |= model_link
                        || ((file.kind == FileKind::Model(ModelFormat::PesModel) || converted)
                            && typed)
                        || is_user_face_xml(folder, file);
                    continue;
                }
            }
            if models.left_out_variants.contains(&file.path) {
                continue;
            }
            let file_name = file.path.name();
            // A `.model` an FMDL or a glTF of its path stem beats is no model of the folder's.
            let converted =
                file.kind == FileKind::Model(ModelFormat::PesModel) && !models.beaten(file);
            let (model_name, local) =
                if file.kind == FileKind::Model(ModelFormat::Fmdl) || converted {
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

    /// The models of the player folder `folder` for a target of `engine`: its own, under its
    /// `ingame_face` marker when it holds one, a linked shared face counting as a face model
    /// of the folder's (`with_linked_face`).
    pub(crate) fn of_player(folder: &PlayerFolder, engine: Engine) -> FolderModels {
        let models =
            FolderModels::of_player_files(&folder.path, &folder.files, folder.ingame_face, engine);
        if folder
            .links
            .iter()
            .any(|link| matches!(link.kind, SharedKind::Face))
        {
            return models.with_linked_face();
        }
        models
    }

    /// The engine of the target the models were read for.
    pub(crate) fn engine(&self) -> Engine {
        self.engine
    }

    /// Whether `file` is a model another representation of its path stem beats (`pipeline.md`
    /// step 3 "Format conversion": target-native first, then glTF, then the other engine's
    /// format): a glTF with a model of the target's format beside it, or a model in the other
    /// engine's format, a `.model` on Fox or an `.fmdl` on pre-Fox, with a model of the
    /// target's format or a glTF beside it. The beaten model has no role and no finding
    /// (TC-MOD-26): nothing reads it.
    pub(crate) fn beaten(&self, file: &FileDescriptor) -> bool {
        let FileKind::Model(format) = file.kind else {
            return false;
        };
        if format == native_format(self.engine) {
            return false;
        }
        let stem = vtree::fold_name(path_stem(file));
        let native_beside = self.native_model_stems.contains(&stem);
        match format {
            ModelFormat::Gltf => native_beside,
            ModelFormat::Fmdl | ModelFormat::PesModel => {
                native_beside || self.gltf_stems.contains(&stem)
            }
        }
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
/// output of a target of the engine `models` were computed for; `None` when it has no role
/// `compile` builds yet. A file in a reserved subfolder is a part of the folder like a file
/// directly in it, its category forced by the subfolder's name (`model_role`); its textures
/// are the folder's own, and the face's files may sit in `face/`. Each engine builds its own
/// model format (`fox_file`, `pre_fox_file`); the textures and the face diff take the same
/// roles on both (`texture_or_face_diff`).
pub(crate) fn player_file(
    folder: &ScopePath,
    file: &FileDescriptor,
    models: &FolderModels,
) -> Option<PlayerFile> {
    let position = position(folder, file)?;
    match models.engine {
        Engine::Fox => fox_file(position, file, models),
        Engine::PreFox => pre_fox_file(position, file, models),
    }
}

/// The folded path stems of the models of `format` among `files` of the folder at `folder`,
/// directly in it or in a reserved subfolder.
fn model_stems(folder: &ScopePath, files: &[FileDescriptor], format: ModelFormat) -> Vec<String> {
    files
        .iter()
        .filter(|file| file.kind == FileKind::Model(format) && position(folder, file).is_some())
        .map(|file| vtree::fold_name(path_stem(file)))
        .collect()
}

/// The model format a target of `engine` reads natively: FMDL on Fox, `.model` on pre-Fox.
fn native_format(engine: Engine) -> ModelFormat {
    match engine {
        Engine::Fox => ModelFormat::Fmdl,
        Engine::PreFox => ModelFormat::PesModel,
    }
}

/// `player_file` on Fox, for `file` at `position`: a `.model` with no `.fmdl` or glTF of its
/// path stem beside it takes the role an FMDL of its name would, the Models task converting it
/// with the `.mtl` its search finds, every `.mtl` outside `common/` is a material set such a
/// conversion may read (`PlayerFile::Material`), and a glTF with no `.fmdl` of its path stem
/// beside it is `PlayerFile::UnsupportedGltf`. A model `FolderModels::beaten` names has no
/// role.
fn fox_file(
    position: Position,
    file: &FileDescriptor,
    models: &FolderModels,
) -> Option<PlayerFile> {
    let name = file.path.name();
    let stem = file_stem(name);
    let path_fold = vtree::fold_name(path_stem(file));
    match file.kind {
        // A left-out kit variant is one only where it would be a model: a file planning gives no
        // role keeps none.
        FileKind::Model(ModelFormat::Fmdl | ModelFormat::PesModel) => {
            if models.beaten(file) {
                return None;
            }
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
        FileKind::Fclo if name.eq_ignore_ascii_case("fcl_hair_sim.fclo") => face_file(
            position,
            models,
            PlayerFile::Packed {
                package: ModelPackage::Face,
                name: "fcl_hair_sim.fclo",
            },
        ),
        FileKind::Mtl if position != Position::Common => Some(PlayerFile::Material),
        FileKind::Texture | FileKind::Bin | FileKind::Xml => {
            texture_or_face_diff(position, file, models)
        }
        FileKind::Model(ModelFormat::Gltf) if !models.beaten(file) => {
            Some(PlayerFile::UnsupportedGltf)
        }
        FileKind::Model(ModelFormat::Gltf)
        | FileKind::Skl
        | FileKind::Fclo
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::SharedLink(_)
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// `player_file` on pre-Fox, for `file` at `position`: a `.model` typed for the face's
/// `face.xml` (`pre_fox_model_type`), or under `ingame_face` a part of the package Fox gives
/// it (`pre_fox_part`), an `.fmdl` with no `.model` or glTF of its path stem beside it the
/// same, the `.skl` paired with such an FMDL in the face its conversion's bind pose
/// (`PlayerFile::ConversionSkeleton`), a `.mtl` beside the models, outside `common/`, a
/// `.common` link to a `.model`, a `.mtl` or a texture (`pre_fox_link`), under `ingame_face` a
/// link to a `.model` being a part as the `.model` itself would be, a member's own `face.xml`,
/// a face file (`PlayerFile::FaceXml`), and a glTF with no `.model` of its path stem beside it
/// (`PlayerFile::UnsupportedGltf`). A model `FolderModels::beaten` names, any other `.skl` and
/// `fcl_hair_sim.fclo` have no role, nor any other link or any `.xml` but the face diff.
fn pre_fox_file(
    position: Position,
    file: &FileDescriptor,
    models: &FolderModels,
) -> Option<PlayerFile> {
    let path_fold = vtree::fold_name(path_stem(file));
    match file.kind {
        FileKind::Model(ModelFormat::PesModel | ModelFormat::Fmdl) => {
            if models.beaten(file) {
                return None;
            }
            let stem = file_stem(file.path.name());
            if models.ingame_face {
                return pre_fox_part(position, stem);
            }
            pre_fox_model_type(position, stem).map(|xml_type| PlayerFile::PreFoxModel { xml_type })
        }
        FileKind::Mtl if position != Position::Common => Some(PlayerFile::Material),
        // Under the marker a link to a `.model` stands for the Common model as a part of his
        // own, copied in, since no `face.xml` names the Common path.
        FileKind::CommonLink if models.ingame_face => {
            match linked_pre_fox_model(file.path.name()) {
                Some(linked) => pre_fox_part(position, file_stem(&linked)),
                None => pre_fox_link(position, file.path.name()),
            }
        }
        FileKind::CommonLink => pre_fox_link(position, file.path.name()),
        FileKind::Xml if file.path.name().eq_ignore_ascii_case("face.xml") => {
            face_file(position, models, PlayerFile::FaceXml)
        }
        FileKind::Texture | FileKind::Bin | FileKind::Xml => {
            texture_or_face_diff(position, file, models)
        }
        FileKind::Skl if models.converted_stems.contains(&path_fold) => {
            Some(PlayerFile::ConversionSkeleton)
        }
        FileKind::Model(ModelFormat::Gltf) if !models.beaten(file) => {
            Some(PlayerFile::UnsupportedGltf)
        }
        FileKind::Model(ModelFormat::Gltf)
        | FileKind::Mtl
        | FileKind::Skl
        | FileKind::Fclo
        | FileKind::MaterialsToml
        | FileKind::SharedLink(_)
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// The pre-Fox role of a model with file stem `stem` at `position` in a player folder holding
/// `ingame_face`, a `.model` of his or the Common one a `.common` link of his names: a part of
/// the package Fox gives it under the marker (`PlayerFile::PreFoxPart`), since the marker
/// means no face and so no `face.xml` to list it, typed as it would be there
/// (`pre_fox_model_type`) for a `glove.xml` to list. The two engines agree on which models
/// have a role. A model named as face content never reaches here: validation drops a marked
/// folder holding one, file or link (`ingame_face_explicit_face_model`). A per-kit model has
/// none: its set is listed through a `face.xml` entry naming `kitN`, and as parts of his boots
/// or gloves every variant would be worn at once.
fn pre_fox_part(position: Position, stem: &str) -> Option<PlayerFile> {
    let xml_type = pre_fox_model_type(position, stem)?;
    if kit_token(stem).is_some() {
        return None;
    }
    model_role(position, stem, true)
        .map(|(package, _)| PlayerFile::PreFoxPart { package, xml_type })
}

/// The `.model` a `.common` link named `link_name` stands for (`legs.model.common` →
/// `legs.model`); `None` for a link to anything else.
fn linked_pre_fox_model(link_name: &str) -> Option<String> {
    common_link_name(link_name)
        .filter(|name| classify(name) == FileKind::Model(ModelFormat::PesModel))
}

/// The pre-Fox role of the `.common` link named `name` at `position`: a link to a `.model`
/// takes the type a `.model` of the linked stem would have there (`pre_fox_model_type`; none
/// for a glove naming no hand), a link to a `.mtl` is a material link, a link to a texture
/// stands for its stem, as on Fox. A link to a per-kit model has none yet: the Common models
/// task, which packs the linked model, would have to list its set. A link in `common/` has
/// none (only textures may sit there, so validation never resolves a link there), nor has a
/// link to anything else.
fn pre_fox_link(position: Position, name: &str) -> Option<PlayerFile> {
    if position == Position::Common {
        return None;
    }
    let linked = common_link_name(name)?;
    match classify(&linked) {
        FileKind::Model(ModelFormat::PesModel) => {
            let stem = file_stem(&linked);
            if kit_token(stem).is_some() {
                return None;
            }
            pre_fox_model_type(position, stem)
                .map(|xml_type| PlayerFile::PreFoxCommonModel { xml_type })
        }
        FileKind::Mtl => Some(PlayerFile::CommonMaterial),
        FileKind::Texture => linked_texture_stem(name).map(PlayerFile::CommonTexture),
        FileKind::Model(ModelFormat::Fmdl | ModelFormat::Gltf)
        | FileKind::Skl
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::MaterialsToml
        | FileKind::Bin
        | FileKind::SharedLink(_)
        | FileKind::CommonLink
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// The role of `file` at `position` that both engines give alike: a texture by its stem, and
/// the face diff, `face_diff.bin` or `face_diff.xml`, a face file (`face_file`); `None` for any
/// other file.
fn texture_or_face_diff(
    position: Position,
    file: &FileDescriptor,
    models: &FolderModels,
) -> Option<PlayerFile> {
    let name = file.path.name();
    match file.kind {
        FileKind::Texture => texture_format(name)
            .map(|format| PlayerFile::Texture(file_stem(name).to_owned(), format)),
        FileKind::Bin if name.eq_ignore_ascii_case("face_diff.bin") => face_file(
            position,
            models,
            PlayerFile::Packed {
                package: ModelPackage::Face,
                name: "face_diff.bin",
            },
        ),
        FileKind::Xml if name.eq_ignore_ascii_case("face_diff.xml") => {
            face_file(position, models, PlayerFile::FaceDiffXml)
        }
        FileKind::Model(_)
        | FileKind::Skl
        | FileKind::Bin
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::SharedLink(_)
        | FileKind::CommonLink
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// `role` for a face file at `position` in a folder whose models are `models`: a face file sits
/// in the folder or its `face/`, and without a face to shape it is not used
/// (`PlayerFile::UnusedFaceFile`).
fn face_file(position: Position, models: &FolderModels, role: PlayerFile) -> Option<PlayerFile> {
    if !matches!(position, Position::Direct | Position::Face) {
        return None;
    }
    Some(if models.face {
        role
    } else {
        PlayerFile::UnusedFaceFile
    })
}

/// Whether `role` is a part of `package`: a model of the folder's own, or a Common model its
/// link brings in, on Fox; a model of an `ingame_face` player's own on pre-Fox.
pub(crate) fn is_part_of(role: &PlayerFile, package: ModelPackage) -> bool {
    matches!(
        role,
        PlayerFile::Model { package: owner, .. }
            | PlayerFile::CommonModel { package: owner, .. }
            | PlayerFile::PreFoxPart { package: owner, .. }
            if *owner == package
    )
}

/// Whether the model `file` of the folder at `folder` is named as face content: the package
/// its name and position give it (`model_role`, without the `ingame_face` marker) is the
/// face. A pre-Fox face packs boots and gloves models beside its face models, all typed by
/// `face.xml`, so this is what keeps a model named as boots or gloves, or sitting in `boots/`
/// or `gloves/`, out of the pre-Fox hand split, as its package keeps it out on Fox
/// (`model_conversion/hand_split.md` "Pipeline integration").
pub(crate) fn named_as_face(folder: &ScopePath, file: &FileDescriptor) -> bool {
    position(folder, file)
        .and_then(|position| model_role(position, file_stem(file.path.name()), false))
        .is_some_and(|(package, _)| package == ModelPackage::Face)
}

/// Whether the player folder `player` has a model that packs into `package` on a target of
/// `engine`, under its `ingame_face` marker when it holds one (`is_part_of`): on Fox its own
/// or one a `.common` link brings in; on pre-Fox a part of his own the marker gives him
/// (`PlayerFile::PreFoxPart`), none without it. What gives a player his own package of that
/// kind, which a link of its kind then combines with (`link_combines`).
pub(crate) fn holds_model(player: &PlayerFolder, package: ModelPackage, engine: Engine) -> bool {
    let models = FolderModels::of_player(player, engine);
    player.files.iter().any(|file| {
        player_file(&player.path, file, &models).is_some_and(|role| is_part_of(&role, package))
    })
}

/// A file name's stem: the name up to its last `.`.
pub(crate) fn file_stem(name: &str) -> &str {
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// What `compile` does with a team export's `Collars/` file for a target of one engine
/// (`pipeline.md` "Collars").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CollarFile {
    /// A model in the format the engine reads, an FMDL on Fox and a `.model` on pre-Fox: it
    /// claims the stock collar its name gives and is written in its place.
    Compiled,
    /// A model in another format: converting a collar between formats is not built yet, so
    /// the gate names it.
    NotCompiled,
    /// Any other kind, which the structure pass keeps only with the strict file-type check
    /// off (`file_type_disallowed`): passed over, with no task and no finding.
    PassedOver,
}

/// What `compile` does with the `Collars/` file `file` for a target of `engine`.
pub(crate) fn collar_file(file: &FileDescriptor, engine: Engine) -> CollarFile {
    match file.kind {
        FileKind::Model(format) => match (engine, format) {
            (Engine::Fox, ModelFormat::Fmdl) | (Engine::PreFox, ModelFormat::PesModel) => {
                CollarFile::Compiled
            }
            (Engine::Fox, ModelFormat::PesModel | ModelFormat::Gltf)
            | (Engine::PreFox, ModelFormat::Fmdl | ModelFormat::Gltf) => CollarFile::NotCompiled,
        },
        FileKind::Texture
        | FileKind::Skl
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::Bin
        | FileKind::SharedLink(_)
        | FileKind::CommonLink
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => CollarFile::PassedOver,
    }
}

/// The first thing in `resolved` that `compile` cannot build yet for `version`, as the context
/// entry of `content_not_yet_compiled`: `what`, a path or the target. `None` when `compile`
/// builds all of it. A team export's FMDL collars are compiled, a collar model in another
/// format is named, and any other file in `Collars/` is passed over (`collar_file`). A refs
/// export compiles its mapped folders like a team's, but is named by its first kit, its logo,
/// its first portrait or its first collar file (`referee_not_compiled`). A kit is named by its
/// first texture no engine compiles (`kit_texture_not_compiled`). A pre-Fox target has a walk
/// of its own (`pre_fox_not_compiled`).
pub(crate) fn first_not_compiled(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
) -> Option<(&'static str, String)> {
    // The walk's order is the plan's ("Phase 3 scope"), so one export always names the same
    // item; reordering the checks changes which item a member is told about.
    match version.engine() {
        Engine::Fox => {}
        Engine::PreFox => return pre_fox_not_compiled(resolved, version),
    }
    let export = &resolved.export;
    match resolved.identity {
        ExportIdentity::Team { .. } => {}
        ExportIdentity::Referees => {
            if let Some(item) = referee_not_compiled(export) {
                return Some(item);
            }
        }
    }
    // A folder no roster slot maps is not compiled, so whatever it holds does not count.
    for folder in mapped_players(export) {
        if let Some(item) = player_not_compiled(export, folder) {
            return Some(item);
        }
    }
    // A shared boots or gloves folder compiles on its own when a mapped team player links it
    // plainly; one no such player links has no output and is not counted (a referee's link
    // is walked above, as a source of his own package). A shared face has
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
    if let Some(item) = kit_texture_not_compiled(export) {
        return Some(item);
    }
    // A slot with a portrait from both sources is not refused: the deep pass has skipped an
    // export whose two files differ (`portrait_conflict`), so identical ones are one portrait.
    // The logo is compiled by the export's logo task, so it is not walked.
    let mut rest = export
        .collars
        .iter()
        .filter(|file| collar_file(file, Engine::Fox) == CollarFile::NotCompiled)
        .chain(
            export
                .common
                .iter()
                .filter(|file| !common_file_compiled(file, Engine::Fox)),
        );
    rest.next().map(what_entry)
}

/// `first_not_compiled` for the pre-Fox `version`, where `compile` builds a team's player
/// folders' own `.model` files with their `.mtl` files, textures and face diff, their own
/// `.fmdl` files converted for the face, their own
/// `face.xml`, their `.common` links to a `.model`, a `.mtl` or a texture, the shared face,
/// boots and gloves folders they link, the export's `Common/` folder, its kits, its `.model`
/// collars, its portraits and its logo. A refs export
/// is named by the target. Otherwise, in this order: each mapped player folder's first file
/// with no pre-Fox role (`player_file`: a link to an `.fmdl`, a per-kit model under
/// `ingame_face` or behind a link, among others; an `.fmdl`, `.skl` or `.fclo` with none is
/// ignored instead) or, under
/// `ingame_face`, with a role not built there yet (`compiled_under_ingame_face`, or an
/// `.fmdl`, which only the face converts), then the
/// first item `pre_fox_shared_not_compiled` names in each shared folder a link of his feeds
/// his own package from
/// (`link_feeds_own_package`: the `Faces/` folder his face link names, the `Boots/` or
/// `Gloves/` folder a link combines under the marker); then each shared boots folder taking an
/// id, then each such gloves folder (`pre_fox_shared_not_compiled`); then the first kit
/// texture no engine compiles (`kit_texture_not_compiled`), then the first collar file
/// `compile` does not build on pre-Fox (`collar_file`), then the first `Common/` file the
/// pre-Fox Common output does not hold (`common_file_compiled`). Each is a later step's
/// content.
fn pre_fox_not_compiled(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
) -> Option<(&'static str, String)> {
    let export = &resolved.export;
    match resolved.identity {
        ExportIdentity::Team { .. } => {}
        ExportIdentity::Referees => return Some(("what", version.to_string())),
    }
    for folder in mapped_players(export) {
        let models = FolderModels::of_player(folder, Engine::PreFox);
        if let Some(file) = folder.files.iter().find(|file| {
            match player_file(&folder.path, file, &models) {
                // A Fox file with no role is ignored (`pipeline.md` step 3 "Format
                // conversion"): an `.fmdl` a `.model` or a glTF of its stem beats, a `.skl` no
                // converted FMDL pairs, `fcl_hair_sim.fclo`; so is a glTF a `.model` of its
                // stem beats. The skip goes with this gate at 4.20, when a file with no role
                // is simply ignored.
                None => {
                    !models.beaten(file)
                        && !matches!(
                            file.kind,
                            FileKind::Model(ModelFormat::Fmdl) | FileKind::Skl | FileKind::Fclo
                        )
                }
                // Only the face converts an FMDL: an `ingame_face` player's boots and gloves
                // read their parts as `.model` files.
                Some(role) => {
                    folder.ingame_face
                        && (!compiled_under_ingame_face(&role)
                            || file.kind == FileKind::Model(ModelFormat::Fmdl))
                }
            }
        }) {
            return Some(what_entry(file));
        }
        // A link feeding his own package makes the shared folder a source of it; his other
        // boots and gloves links load the shared outputs walked below.
        for link in folder
            .links
            .iter()
            .filter(|link| link_feeds_own_package(&export.roster, Engine::PreFox, folder, link))
        {
            let shared = linked_folder(export, link)
                .expect("validation drops a player folder whose link names no shared folder");
            if let Some(item) = pre_fox_shared_not_compiled(link.kind, shared) {
                return Some(item);
            }
        }
    }
    for kind in [SharedKind::Boots, SharedKind::Gloves] {
        for folder in shared_folders_taking_ids(export, version.engine(), kind) {
            if let Some(item) = pre_fox_shared_not_compiled(kind, folder) {
                return Some(item);
            }
        }
    }
    if let Some(item) = kit_texture_not_compiled(export) {
        return Some(item);
    }
    let mut rest = export
        .collars
        .iter()
        .filter(|file| collar_file(file, Engine::PreFox) == CollarFile::NotCompiled)
        .chain(
            export
                .common
                .iter()
                .filter(|file| !common_file_compiled(file, Engine::PreFox)),
        );
    rest.next().map(what_entry)
}

/// The first kit texture of `export`, kit by kit, that `compile` does not build on either
/// engine: a stem outside `KIT_TEXTURE_STEMS` (`kit_spec`), or a file in no format
/// `dds_convert` converts. Every kit compiles otherwise, a placeholder kit included.
fn kit_texture_not_compiled(export: &ValidatedAestheticsExport) -> Option<(&'static str, String)> {
    export.kits.kits.values().find_map(|kit| {
        kit.textures
            .iter()
            .find(|texture| {
                !KIT_TEXTURE_STEMS.contains(&texture.stem.as_str())
                    || texture_format(texture.file.path.name()).is_none()
            })
            .map(|texture| what_entry(&texture.file))
    })
}

/// Whether a pre-Fox player file of `role`, in a folder holding `ingame_face`, is compiled: a
/// part of his boots or gloves is (`PlayerFile::PreFoxPart`), his own model or a Common one
/// his `.common` link copies in, merged into his own `boots.model` or listed in his own
/// `glove.xml`, and so is every role that does not name a model, a `.mtl` link included (the
/// Common `.mtl` is copied in with the part using it).
fn compiled_under_ingame_face(role: &PlayerFile) -> bool {
    match role {
        PlayerFile::PreFoxPart {
            package: ModelPackage::Boots | ModelPackage::Gloves,
            ..
        } => true,
        // Validation drops a marked folder holding an explicit face model, so a part of the
        // face is not met; it would have no package to go in. Under the marker a model link
        // is a part (`pre_fox_part`), so a link listed by reference is not met either.
        PlayerFile::PreFoxPart {
            package: ModelPackage::Face,
            ..
        }
        | PlayerFile::PreFoxCommonModel { .. } => false,
        // Not met: under the marker the face files are not used, the xml among them
        // (`UnusedFaceFile`), and no FMDL is converted, so no skeleton is its bind pose.
        PlayerFile::FaceXml | PlayerFile::ConversionSkeleton => false,
        // Not the gate's: planning drops the folder holding one before the gate walks it
        // (`model_gltf_unsupported`).
        PlayerFile::UnsupportedGltf => true,
        PlayerFile::Model { .. }
        | PlayerFile::CommonModel { .. }
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
        | PlayerFile::CommonMaterial => true,
    }
}

/// The first of the refs `export`'s kits by slot (its folder), then its logo (the main file),
/// then its first portrait (a mapped folder's `portrait.*` in folder order, then a
/// `Portraits/` file), then its first collar file, whatever its kind. A referee has no kit
/// slot, team logo or player id, so none of them has a place to go: the referees' kits are the
/// template tree's (`blue_port.md` "Referee export processing"), and with no kit of their own
/// they have none to put a collar on.
fn referee_not_compiled(export: &ValidatedAestheticsExport) -> Option<(&'static str, String)> {
    if let Some(kit) = export.kits.kits.values().next() {
        return Some(("what", kit.path.as_str().to_owned()));
    }
    if let Some(logo) = &export.logo {
        return Some(what_entry(&logo.main.file));
    }
    let mut files = mapped_players(export)
        .into_iter()
        .filter_map(|folder| folder.portrait.as_ref())
        .chain(export.portraits.values())
        .chain(&export.collars);
    files.next().map(what_entry)
}

/// Whether `compile` for a target of `engine` builds the `Common/` file, or accepts it:
/// directly in the folder, a texture (the export's Common textures task); on Fox an FMDL or a
/// `.skl` (reached through a player's `.common` link; one no link names builds nothing); on
/// pre-Fox a `.model` or a `.mtl` (the export's Common models task, which writes every one into
/// the team's Common output). Any other kind, and any file deeper in the folder, is named.
fn common_file_compiled(file: &FileDescriptor, engine: Engine) -> bool {
    if !is_direct_common_file(&file.path) {
        return false;
    }
    // Each engine builds its own model format.
    match file.kind {
        FileKind::Texture => texture_format(file.path.name()).is_some(),
        FileKind::Model(ModelFormat::Fmdl) | FileKind::Skl => match engine {
            Engine::Fox => true,
            Engine::PreFox => false,
        },
        FileKind::Model(ModelFormat::PesModel) | FileKind::Mtl => match engine {
            Engine::Fox => false,
            Engine::PreFox => true,
        },
        FileKind::Model(ModelFormat::Gltf)
        | FileKind::Fclo
        | FileKind::Xml
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
    let models = FolderModels::of_player(folder, Engine::Fox);
    // A member's own `face.xml` has no Fox role and is ignored (`xml_ignored_fox`): Fox has no
    // `face.xml`, and the models compile as without it. A pre-Fox file with no role is ignored
    // too (`pipeline.md` step 3 "Format conversion"): a `.model` an FMDL or a glTF of its stem
    // beats, a `.mtl` in `common/`; so is a glTF an FMDL of its stem beats. The skip goes with
    // this gate at 4.20, when a file with no role is simply ignored. A selected glTF has a
    // role, and planning has dropped its folder before this walk.
    if let Some(file) = folder.files.iter().find(|file| {
        player_file(&folder.path, file, &models).is_none()
            && !is_user_face_xml(&folder.path, file)
            && !models.beaten(file)
            && !matches!(
                file.kind,
                FileKind::Model(ModelFormat::PesModel) | FileKind::Mtl
            )
    }) {
        return Some(what_entry(file));
    }
    // A team player's boots or gloves link alone loads the shared output as it is; one beside
    // a local model of its package combines, as every face link and every referee's link
    // does: the shared folder's files become the player's own, with the roles a shared
    // folder's files have.
    for link in folder
        .links
        .iter()
        .filter(|link| link_feeds_own_package(&export.roster, Engine::Fox, folder, link))
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
/// is loaded as and its textures have a place to go, a `.model` or a `.mtl` is named (a shared
/// folder's `.model` is not converted yet), and a folder with no model is named as a whole.
fn shared_not_compiled(
    kind: SharedKind,
    folder: &SharedModelFolder,
) -> Option<(&'static str, String)> {
    let path = &folder.path;
    let package = package_of(kind);
    let models = FolderModels::of(path, &folder.files, Engine::Fox);
    let mut has_model = false;
    for file in &folder.files {
        // A shared face's own `face.xml` is ignored as a player folder's is.
        if kind == SharedKind::Face && is_user_face_xml(path, file) {
            continue;
        }
        let Some(role) = player_file(path, file, &models) else {
            return Some(what_entry(file));
        };
        // A model of another package has no package here, a `.common` link (kept by a
        // non-strict file-type check) resolves only from a player folder, and a face file
        // with no face model has no place in a shared folder. Nor is a `.model` a shared
        // folder holds converted yet, nor its `.mtl` read, nor is its glTF's folder dropped
        // at planning as a player folder's is.
        if role.package().is_some_and(|owner| owner != package)
            || matches!(
                file.kind,
                FileKind::Model(ModelFormat::PesModel) | FileKind::Mtl
            )
            || matches!(
                role,
                PlayerFile::CommonModel { .. }
                    | PlayerFile::CommonTexture(_)
                    | PlayerFile::UnusedFaceFile
                    | PlayerFile::UnsupportedGltf
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

/// The first thing in the shared `folder` of `kind` that `compile` cannot build for a pre-Fox
/// target yet: its first file with no pre-Fox role (`player_file`), that is a `.common` link
/// (kept by a non-strict file-type check, it resolves only from a player folder), that is its
/// own `face.xml` (not supported in a shared folder yet), that is an `.fmdl` or the skeleton
/// paired with one (not converted in a shared folder yet) or, in a boots
/// or gloves folder, with a role other than a model, a `.mtl` or a texture (a face diff has no
/// face there to shape), or that is a per-kit model; a folder with no model is named as a
/// whole. A boots folder holding several models compiles: they are merged into its one
/// `boots.model`. A `Faces/` folder's files are copied into each linking player's face, so its
/// face files are kept, and its per-kit sets are listed there.
fn pre_fox_shared_not_compiled(
    kind: SharedKind,
    folder: &SharedModelFolder,
) -> Option<(&'static str, String)> {
    let path = &folder.path;
    let models = FolderModels::of(path, &folder.files, Engine::PreFox);
    let mut has_model = false;
    for file in &folder.files {
        let Some(role) = player_file(path, file, &models) else {
            return Some(what_entry(file));
        };
        // A shared face's own `face.xml` is not supported yet: whether it rules every player
        // combining the face is an open question (`messages.md` "User-supplied `face.xml`").
        // Nor is an FMDL a shared folder holds converted yet, nor its skeleton read, nor is
        // its glTF's folder dropped at planning as a player folder's is.
        if matches!(
            role,
            PlayerFile::PreFoxCommonModel { .. }
                | PlayerFile::CommonMaterial
                | PlayerFile::CommonTexture(_)
                | PlayerFile::FaceXml
                | PlayerFile::ConversionSkeleton
                | PlayerFile::UnsupportedGltf
        ) || file.kind == FileKind::Model(ModelFormat::Fmdl)
        {
            return Some(what_entry(file));
        }
        let loose_output_file = matches!(
            role,
            PlayerFile::PreFoxModel { .. } | PlayerFile::Material | PlayerFile::Texture(..)
        );
        // The boots merge into one `boots.model` and the shared `glove.xml` lists every glove,
        // so neither can list a per-kit set once yet: every variant would be worn at once.
        let per_kit_model = matches!(role, PlayerFile::PreFoxModel { .. })
            && kit_token(file_stem(file.path.name())).is_some();
        match kind {
            SharedKind::Face => {}
            SharedKind::Boots | SharedKind::Gloves if loose_output_file && !per_kit_model => {}
            SharedKind::Boots | SharedKind::Gloves => return Some(what_entry(file)),
        }
        has_model |= matches!(role, PlayerFile::PreFoxModel { .. });
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
    use crate::testing::{resolved, resolved_with_issues, to_plan, two_team_colors};

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
    fn a_fox_target_converts_a_player_s_own_model_and_ignores_one_an_fmdl_beats() {
        // A `.model` with no FMDL of its stem, and its `.mtl`.
        assert_eq!(
            gate(&[
                "Players/03 - A/boots.model",
                "Players/03 - A/boots.mtl",
                "Players/03 - A/torso.model",
                "Players/03 - A/face/hat.mtl",
            ]),
            None
        );
        assert_eq!(
            first_hit(&["Players/03 - A/boots.model", "Players/03 - A/materials.mtl"]),
            None
        );
        // `FACE`'s `face_high.fmdl` beats the `.model` of its stem, ignored with its `.mtl`.
        assert_eq!(
            gate(&[
                "Players/03 - A/face_high.model",
                "Players/03 - A/face_high.mtl"
            ]),
            None
        );
        // A glTF beats the `.model` of its stem, and its folder is planning's to drop
        // (`model_gltf_unsupported`); one an FMDL of its stem beats is ignored.
        let glb = "Players/03 - A/boots.glb";
        assert_eq!(gate(&["Players/03 - A/boots.model", glb]), None);
        assert_eq!(gate(&["Players/03 - A/boots.fmdl", glb]), None);
        // A shared folder's glTF, `.model` or `.mtl`, a `Common/` one and a link to it are
        // still named.
        let shared_glb = "Boots/Crocs/boots.glb";
        assert_eq!(
            gate(&["Players/05 - B/Crocs.boots", shared_glb]),
            what(shared_glb)
        );
        let face_glb = "Faces/Round/hair_high.glb";
        assert_eq!(
            gate(&["Players/05 - B/Round.face", face_glb]),
            what(face_glb)
        );
        let shared_model = "Boots/Crocs/boots.model";
        assert_eq!(
            gate(&["Players/05 - B/Crocs.boots", shared_model]),
            what(shared_model)
        );
        let shared_mtl = "Faces/Round/hair_high.mtl";
        assert_eq!(
            gate(&[
                "Players/05 - B/Round.face",
                "Faces/Round/hair_high.fmdl",
                shared_mtl
            ]),
            what(shared_mtl)
        );
        let link = "Players/03 - A/legs.model.common";
        assert_eq!(gate(&[link, "Common/legs.model"]), what(link));
        assert_eq!(gate(&["Common/legs.model"]), what("Common/legs.model"));
    }

    #[test]
    fn a_fox_target_ignores_a_face_s_own_face_xml() {
        assert_eq!(gate(&["Players/03 - A/face.xml"]), None);
        assert_eq!(gate(&["Players/03 - A/face/face.xml"]), None);
        assert_eq!(
            gate(&[
                "Players/05 - B/Round.face",
                "Faces/Round/hair_high.fmdl",
                "Faces/Round/face.xml",
            ]),
            None
        );
        // Anywhere else an `.xml` is still named, a shared boots folder's included.
        let deep = "Players/03 - A/boots/face.xml";
        assert_eq!(gate(&[deep]), what(deep));
        let boots_xml = "Boots/Crocs/face.xml";
        assert_eq!(
            gate(&[
                "Players/05 - B/Crocs.boots",
                "Boots/Crocs/boots.fmdl",
                boots_xml
            ]),
            what(boots_xml)
        );
    }

    /// A pre-Fox face folder: a model, its material set and its texture.
    const PRE_FOX_FACE: [&str; 3] = [
        "Players/03 - A/face_high.model",
        "Players/03 - A/face_high.mtl",
        "Players/03 - A/skin.dds",
    ];

    /// The gate's first hit in the export `co Midcup Gate` holding `PRE_FOX_FACE` and `files`,
    /// for `version`.
    fn pre_fox_hit(files: &[&str], version: PesVersion) -> Option<(&'static str, String)> {
        let files: Vec<(&str, u64)> = PRE_FOX_FACE
            .iter()
            .chain(files)
            .map(|path| (*path, 1))
            .collect();
        first_not_compiled(&resolved("co Midcup Gate", &files, &[], None), version)
    }

    #[test]
    fn a_pre_fox_target_compiles_a_player_s_own_models_and_names_the_rest_in_order() {
        for version in [PesVersion::Pes15, PesVersion::Pes16, PesVersion::Pes17] {
            assert_eq!(pre_fox_hit(&[], version), None, "{version}");
        }
        let pre_fox = |files: &[&str]| pre_fox_hit(files, PesVersion::Pes17);
        // A face file, a model in each reserved subfolder and a portrait compile too.
        assert_eq!(
            pre_fox(&[
                "Players/03 - A/face_diff.bin",
                "Players/03 - A/boots/x.model",
                "Players/03 - A/gloves/left_gloveL.model",
                "Players/03 - A/face/hat.model",
                "Players/03 - A/portrait.dds",
            ]),
            None
        );
        // A linked shared face, boots or gloves folder holding models compiles.
        let round = [
            "Players/03 - A/Round.face",
            "Faces/Round/hair_high.model",
            "Faces/Round/hair_high.mtl",
        ];
        assert_eq!(pre_fox(&round), None);
        let crocs = [
            "Players/03 - A/Crocs.boots",
            "Boots/Crocs/boots.model",
            "Boots/Crocs/boots.mtl",
            "Boots/Crocs/crocs.dds",
        ];
        assert_eq!(pre_fox(&crocs), None);
        let keeper = [
            "Players/03 - A/Keeper.gloves",
            "Gloves/Keeper/glove_l.model",
            "Gloves/Keeper/glove_r.model",
            "Gloves/Keeper/materials.mtl",
        ];
        assert_eq!(pre_fox(&keeper), None);
        // A shared folder's FMDL is named, a face folder's too: only a player's own is
        // converted.
        let hat = "Faces/Round/hat.fmdl";
        assert_eq!(pre_fox(&[round.as_slice(), &[hat]].concat()), what(hat));
        // A shared boots folder holding several boots models compiles: they are merged.
        let link = "Players/03 - A/Crocs.boots";
        assert_eq!(
            pre_fox(&[
                link,
                "Boots/Crocs/b_boots.model",
                "Boots/Crocs/a_boots.model"
            ]),
            None
        );
        // A shared folder with no model is named as a whole.
        assert_eq!(
            pre_fox(&[link, "Boots/Crocs/crocs.dds"]),
            what("Boots/Crocs")
        );
        // A face file has no place in a shared boots folder.
        let diff = "Boots/Crocs/face_diff.bin";
        assert_eq!(
            pre_fox(&[link, "Boots/Crocs/boots.model", diff]),
            what(diff)
        );
        // A player's own FMDL compiles, converted for his face: its skeleton is the
        // conversion's bind pose, and an FMDL a `.model` of its stem beats, a skeleton no
        // converted FMDL pairs and a `.fclo` are ignored.
        assert_eq!(
            pre_fox(&[
                "Players/03 - A/hat.fmdl",
                "Players/03 - A/hat.skl",
                "Players/03 - A/face_high.fmdl",
                "Players/03 - A/boots.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
            ]),
            None
        );
        // A glTF beats the FMDL of its stem, and its folder is planning's to drop
        // (`model_gltf_unsupported`); one a `.model` of its stem beats is ignored. A shared
        // folder's glTF is still named.
        assert_eq!(
            pre_fox(&["Players/03 - A/hat.fmdl", "Players/03 - A/hat.glb"]),
            None
        );
        assert_eq!(
            pre_fox(&["Players/03 - A/hat.model", "Players/03 - A/hat.glb"]),
            None
        );
        let shared_glb = "Faces/Round/hat.glb";
        assert_eq!(
            pre_fox(&[round.as_slice(), &[shared_glb]].concat()),
            what(shared_glb)
        );
        let fmdl_link = "Players/05 - B/x.fmdl.common";
        let face_link = "Players/05 - B/Round.face";
        let boots_link = "Players/05 - B/Crocs.boots";
        let boots_fmdl = "Boots/Crocs/boots.fmdl";
        let kit = "Kits/g1/kit.dds";
        // A `.model` collar compiles; an FMDL one is named until collars are converted.
        assert_eq!(pre_fox(&["Collars/collar_12.model"]), None);
        let collar = "Collars/collar_12.fmdl";
        let common = "Common/x.fmdl";
        let kit_extra = "Kits/g1/kit_spec.dds";
        // In a folder, its first file with no role, then its linked face folder's first; then
        // the shared boots and gloves folders, the kits' textures, the collars and `Common/`.
        let all = [
            fmdl_link,
            face_link,
            "Faces/Round/hair_high.model",
            hat,
            boots_link,
            boots_fmdl,
            kit,
            kit_extra,
            collar,
            common,
        ];
        assert_eq!(pre_fox(&all), what(fmdl_link));
        assert_eq!(pre_fox(&all[1..]), what(hat));
        assert_eq!(pre_fox(&all[4..]), what(boots_fmdl));
        assert_eq!(pre_fox(&all[6..]), what(kit_extra));
        assert_eq!(pre_fox(&all[8..]), what(collar));
        assert_eq!(pre_fox(&all[9..]), what(common));
        // A kit compiles, its config, both maps (planning drops the srm before the gate
        // asks) and a layout marker with it.
        assert_eq!(
            pre_fox(&[
                kit,
                "Kits/g1/config.toml",
                "Kits/g1/kit_mask.dds",
                "Kits/g1/kit_srm.dds",
                "Kits/g1/fox",
                "Kits/p1/kit_back.ftex",
                "Kits/p1/kit_chest.dds",
                "Kits/p1/kit_leg.dds",
                "Kits/p1/kit_name.dds",
            ]),
            None
        );
        // A member's own `face.xml` compiles, directly in his folder or in `face/`; a per-kit
        // set compiles.
        let own_xml = "Players/03 - A/face.xml";
        assert_eq!(pre_fox(&[own_xml]), None);
        let xml_in_face = "Players/03 - A/face/face.xml";
        assert_eq!(pre_fox(&[xml_in_face]), None);
        // A shared face's own is not supported yet.
        let shared_xml = "Faces/Round/face.xml";
        assert_eq!(
            pre_fox(&[round.as_slice(), &[shared_xml]].concat()),
            what(shared_xml)
        );
        assert_eq!(
            pre_fox(&[
                "Players/03 - A/pants_kit1.model",
                "Players/03 - A/pants_kit2.model"
            ]),
            None
        );
    }

    #[test]
    fn under_ingame_face_a_pre_fox_target_compiles_boots_and_gloves_parts_and_common_links() {
        let pre_fox = |files: &[&str]| pre_fox_hit(files, PesVersion::Pes17);
        let marker = "Players/05 - B/ingame_face";
        let boots = "Players/05 - B/kit_boots.model";
        let glove = "Players/05 - B/x_gloveL.model";
        // Boots parts, a model the face would take included, gloves parts, in the folder and
        // in `gloves/`, with their `.mtl`, a texture, a texture link and a face diff the folder
        // does not use.
        assert_eq!(
            pre_fox(&[
                marker,
                boots,
                "Players/05 - B/kit_boots.mtl",
                "Players/05 - B/torso.model",
                "Players/05 - B/face/hat.model",
                "Players/05 - B/boots/x.model",
                glove,
                "Players/05 - B/gloves/keeper_handR.model",
                "Players/05 - B/skin.dds",
                "Players/05 - B/hair.dds.common",
                "Common/hair.dds",
                "Players/05 - B/face_diff.bin",
            ]),
            None
        );
        // A link to a Common model or `.mtl` compiles: its Common files are copied in as
        // parts of his boots or gloves.
        for file in [
            "Players/05 - B/legs.model.common",
            "Players/05 - B/glove_l.model.common",
            "Players/05 - B/body.mtl.common",
        ] {
            assert_eq!(
                pre_fox(&[
                    marker,
                    boots,
                    "Common/legs.model",
                    "Common/glove_l.model",
                    "Common/body.mtl",
                    file
                ]),
                None,
                "{file}"
            );
        }
        // An FMDL is a part under the marker, but only the face converts one: it is named,
        // and one a `.model` of its stem beats is not.
        let fmdl = "Players/05 - B/x_gloveL.fmdl";
        assert_eq!(pre_fox(&[marker, boots, fmdl]), what(fmdl));
        assert_eq!(pre_fox(&[marker, glove, fmdl]), None);
        // A link to a per-kit Common model has no role under the marker, as a per-kit file.
        let per_kit = "Players/05 - B/pants_kit1.model.common";
        assert_eq!(
            pre_fox(&[marker, boots, "Common/pants_kit1.model", per_kit]),
            what(per_kit)
        );
        // A gloves link alone loads the shared folder, as without the marker.
        assert_eq!(
            pre_fox(&[
                marker,
                boots,
                "Players/05 - B/Keeper.gloves",
                "Gloves/Keeper/glove_l.model",
            ]),
            None
        );
        // A boots link beside a boots part combines: its folder is walked as a source of the
        // player's boots, though it takes no id.
        let diff = "Boots/Crocs/face_diff.bin";
        let crocs = [
            marker,
            boots,
            "Players/05 - B/Crocs.boots",
            "Boots/Crocs/boots.model",
        ];
        assert_eq!(pre_fox(&crocs), None);
        assert_eq!(pre_fox(&[crocs.as_slice(), &[diff]].concat()), what(diff));
        // A gloves link beside a gloves part likewise, as a source of his gloves.
        let diff = "Gloves/Keeper/face_diff.bin";
        let keeper = [
            marker,
            glove,
            "Players/05 - B/Keeper.gloves",
            "Gloves/Keeper/glove_l.model",
        ];
        assert_eq!(pre_fox(&keeper), None);
        assert_eq!(pre_fox(&[keeper.as_slice(), &[diff]].concat()), what(diff));
    }

    #[test]
    fn a_pre_fox_target_compiles_common_s_models_mtl_files_and_textures_and_the_links_to_them() {
        let pre_fox = |files: &[&str]| pre_fox_hit(files, PesVersion::Pes17);
        // A player's link to a `.model`, a `.mtl` or a texture, each target in `Common/`.
        let common = ["Common/legs.model", "Common/body.mtl", "Common/hair.dds"];
        assert_eq!(pre_fox(&common), None);
        assert_eq!(
            pre_fox(
                &[
                    common.as_slice(),
                    &[
                        "Players/03 - A/legs.model.common",
                        "Players/03 - A/face_high.model.common",
                        "Players/03 - A/body.mtl.common",
                        "Players/03 - A/hair.dds.common",
                    ],
                    &["Common/face_high.model"],
                ]
                .concat()
            ),
            None
        );
        // A `Common/` file of a Fox format is named.
        for refused in ["Common/x.fmdl", "Common/legs.skl"] {
            assert_eq!(pre_fox(&[refused]), what(refused), "{refused}");
        }
        // So is one in a subfolder, which only a non-strict file-type check keeps (see the Fox
        // test above), asked directly here.
        for (path, compiled) in [
            ("Common/sub/y.dds", false),
            ("Common/sub/legs.model", false),
            ("Common/y.dds", true),
            ("Common/legs.model", true),
            ("Common/body.mtl", true),
        ] {
            let path = ScopePath::new(path).unwrap();
            let file = FileDescriptor {
                size: 0,
                kind: aesthetics_export::classify(path.name()),
                source: path.clone(),
                path,
            };
            assert_eq!(
                common_file_compiled(&file, Engine::PreFox),
                compiled,
                "{}",
                file.path.as_str()
            );
        }
        // A link to an FMDL has no pre-Fox role.
        let fmdl_link = "Players/03 - A/x.fmdl.common";
        assert_eq!(pre_fox(&[fmdl_link, "Common/x.fmdl"]), what(fmdl_link));
    }

    /// The gate's first hit in the export `refs Cup` mapping `Ref A` to slot 01, holding a Fox
    /// face folder and `files`, for `version`.
    fn referee_hit(files: &[&str], version: PesVersion) -> Option<(&'static str, String)> {
        let files: Vec<(&str, u64)> = [
            "Players/Ref A/face_high.fmdl",
            "Players/Ref A/face_diff.bin",
        ]
        .iter()
        .chain(files)
        .map(|path| (*path, 1))
        .collect();
        first_not_compiled(
            &resolved("refs Cup", &files, &[], Some(b"01 Ref A\n")),
            version,
        )
    }

    #[test]
    fn a_referee_export_s_folders_compile_on_fox_and_the_target_is_named_pre_fox() {
        assert_eq!(referee_hit(&[], PesVersion::Pes21), None);
        assert_eq!(referee_hit(&[], PesVersion::Pes17), what("PES 2017"));
    }

    #[test]
    fn a_referee_export_s_kits_then_logo_then_portraits_then_collars_are_named() {
        assert_eq!(
            referee_hit(&["Kits/p2/kit.dds", "Kits/p1/kit.dds"], PesVersion::Pes21),
            what("Kits/p1")
        );
        assert_eq!(
            referee_hit(&["logo.png", "Kits/g1/kit.dds"], PesVersion::Pes21),
            what("Kits/g1")
        );
        assert_eq!(
            referee_hit(
                &["logo.png", "Players/Ref A/portrait.dds"],
                PesVersion::Pes21
            ),
            what("logo.png")
        );
        assert_eq!(
            referee_hit(
                &["Portraits/player_01.dds", "Players/Ref A/portrait.dds"],
                PesVersion::Pes21
            ),
            what("Players/Ref A/portrait.dds")
        );
        assert_eq!(
            referee_hit(&["Portraits/player_01.dds"], PesVersion::Pes21),
            what("Portraits/player_01.dds")
        );
        // A collar file, of any model format, after the portraits: the referees have no kit
        // of their own to put it on.
        assert_eq!(
            referee_hit(
                &["Collars/collar_12.fmdl", "Portraits/player_01.dds"],
                PesVersion::Pes21
            ),
            what("Portraits/player_01.dds")
        );
        assert_eq!(
            referee_hit(&["Collars/collar_12.fmdl"], PesVersion::Pes21),
            what("Collars/collar_12.fmdl")
        );
    }

    #[test]
    fn a_referee_s_plain_link_is_walked_as_a_part_of_his_own_packages() {
        assert_eq!(
            referee_hit(
                &["Players/Ref A/Studs.boots", "Boots/Studs/boots.fmdl"],
                PesVersion::Pes21
            ),
            None
        );
        // A referee's shared boots take no id of their own, so only his link reaches them.
        assert_eq!(
            referee_hit(
                &["Players/Ref A/Studs.boots", "Boots/Studs/shirt.dds"],
                PesVersion::Pes21
            ),
            what("Boots/Studs")
        );
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
        assert!(holds_model(&folder, ModelPackage::Boots, Engine::Fox));
        assert!(holds_model(&folder, ModelPackage::Gloves, Engine::Fox));
        assert!(!holds_model(&folder, ModelPackage::Face, Engine::Fox));
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
        shared_not_compiled(kind, &shared_folder(kind, names))
    }

    /// The shared folder `Crocs` of `kind` holding the files `names`.
    fn shared_folder(kind: SharedKind, names: &[&str]) -> SharedModelFolder {
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
        SharedModelFolder {
            path: ScopePath::new(&folder_path).unwrap(),
            folder_name: "Crocs".to_owned(),
            files,
        }
    }

    #[test]
    fn on_pre_fox_a_shared_folder_s_common_link_is_named() {
        for kind in [SharedKind::Face, SharedKind::Boots, SharedKind::Gloves] {
            let model = match kind {
                SharedKind::Face => "hair_high.model",
                SharedKind::Boots => "boots.model",
                SharedKind::Gloves => "glove_l.model",
            };
            let folder = content_folder(kind);
            assert_eq!(
                pre_fox_shared_not_compiled(kind, &shared_folder(kind, &[model, "x.mtl"])),
                None,
                "{kind:?}"
            );
            for link in ["legs.model.common", "x.mtl.common", "hair.dds.common"] {
                assert_eq!(
                    pre_fox_shared_not_compiled(kind, &shared_folder(kind, &[model, link])),
                    what(&format!("{folder}/Crocs/{link}")),
                    "{kind:?} {link}"
                );
            }
        }
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
    fn a_kit_texture_outside_the_config_s_five_and_the_two_maps_is_named() {
        // The two maps are compiled: the srm on Fox, and the mask on Fox is planning's to
        // drop before the gate asks.
        for map in ["Kits/g1/kit_srm.dds", "Kits/g1/kit_mask.dds"] {
            assert_eq!(gate(&["Kits/g1/kit.dds", map]), None, "{map}");
        }
        let file = "Kits/g1/kit_spec.dds";
        assert_eq!(gate(&["Kits/g1/kit.dds", file]), what(file));
    }

    #[test]
    fn the_logo_and_fmdl_collars_are_compiled_and_common_holds_models_skeletons_and_textures() {
        assert_eq!(gate(&["logo.dds", "logo_small_crop.png"]), None);
        assert_eq!(gate(&["Collars/collar_12.fmdl"]), None);
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
                common_file_compiled(&file, Engine::Fox),
                compiled,
                "{}",
                file.path.as_str()
            );
        }
    }

    #[test]
    fn a_collar_model_of_another_format_is_named_and_a_file_of_another_kind_passed_over() {
        // Converting a collar between engines is not built yet.
        for collar in ["Collars/collar_12.model", "Collars/collar_12.glb"] {
            assert_eq!(
                gate(&[collar, "Collars/collar_13.fmdl"]),
                what(collar),
                "{collar}"
            );
        }
        for collar in ["Collars/collar_12.fmdl", "Collars/collar_12.glb"] {
            assert_eq!(
                pre_fox_hit(&[collar, "Collars/collar_13.model"], PesVersion::Pes17),
                what(collar),
                "{collar}"
            );
        }
        // A texture is kept in `Collars/` only with the strict file-type check off, which
        // `resolved` has on: added as the structure pass would keep it.
        let path = ScopePath::new("Collars/collar_12.dds").unwrap();
        let file = FileDescriptor {
            size: 1,
            kind: classify(path.name()),
            source: path.clone(),
            path,
        };
        for (face, version) in [
            (FACE.as_slice(), PesVersion::Pes21),
            (PRE_FOX_FACE.as_slice(), PesVersion::Pes17),
        ] {
            let face: Vec<(&str, u64)> = face.iter().map(|path| (*path, 1)).collect();
            let mut export = resolved("co Midcup Gate", &face, &[], None);
            export.export.collars.push(file.clone());
            assert_eq!(first_not_compiled(&export, version), None, "{version}");
        }
    }

    #[test]
    fn a_collar_compiles_in_the_model_format_its_target_s_engine_reads() {
        let collar = |name: &str, engine| {
            let path = ScopePath::new(&format!("Collars/{name}")).unwrap();
            let file = FileDescriptor {
                size: 1,
                kind: classify(path.name()),
                source: path.clone(),
                path,
            };
            collar_file(&file, engine)
        };
        for (name, fox, pre_fox) in [
            (
                "collar_12.fmdl",
                CollarFile::Compiled,
                CollarFile::NotCompiled,
            ),
            (
                "collar_12.model",
                CollarFile::NotCompiled,
                CollarFile::Compiled,
            ),
            (
                "collar_12.glb",
                CollarFile::NotCompiled,
                CollarFile::NotCompiled,
            ),
            (
                "collar_12.dds",
                CollarFile::PassedOver,
                CollarFile::PassedOver,
            ),
        ] {
            assert_eq!(collar(name, Engine::Fox), fox, "{name} on Fox");
            assert_eq!(collar(name, Engine::PreFox), pre_fox, "{name} on pre-Fox");
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
        assert!(holds_model(&folder, ModelPackage::Boots, Engine::Fox));
        assert!(!holds_model(&folder, ModelPackage::Face, Engine::Fox));
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
        // a kit texture the config has no field for, a pre-Fox collar (not converted yet).
        let loose_skl = "Players/03 - A/torso.skl";
        let face_link = "Players/03 - A/Round.face";
        let shared_face = "Faces/Round/face_high.fmdl";
        let shared_face_skl = "Faces/Round/fcl_hair.skl";
        let link = "Players/03 - A/Crocs.boots";
        let shared = "Boots/Crocs/boots.fmdl";
        let shared_skl = "Boots/Crocs/kit_boots.skl";
        let kit = "Kits/g1/kit.dds";
        let kit_extra = "Kits/g1/kit_spec.dds";
        let collar = "Collars/collar_12.model";
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
    fn the_other_engine_s_map_is_dropped_before_the_gate_and_the_task() {
        // (version, a face folder it compiles, the map it reads, the map it drops)
        let cases = [
            (PesVersion::Pes21, FACE.as_slice(), "kit_srm", "kit_mask"),
            (
                PesVersion::Pes17,
                PRE_FOX_FACE.as_slice(),
                "kit_mask",
                "kit_srm",
            ),
        ];
        for (version, face, kept, dropped) in cases {
            let files: Vec<(&str, u64)> = face
                .iter()
                .chain(&[
                    "Kits/g1/config.toml",
                    "Kits/g1/kit.dds",
                    "Kits/g1/kit_mask.dds",
                    "Kits/g1/kit_srm.dds",
                ])
                .map(|path| (*path, 1))
                .collect();
            let export = resolved("co Midcup Gate", &files, &[], None);

            let report = plan_run(
                vec![to_plan(ExportId(0), export, two_team_colors(), None)],
                version,
            );

            // The drop is reported once, on the kit, naming the file.
            let [message] = report.messages.as_slice() else {
                panic!("{version}: {:?}", report.messages);
            };
            assert_eq!(message.code.code, "kit_texture_not_used", "{version}");
            assert_eq!(
                message.scope,
                studio_core::Scope::Folder {
                    export_id: ExportId(0),
                    path: ScopePath::new("Kits/g1").unwrap(),
                },
                "{version}"
            );
            assert_eq!(
                message.disposition,
                studio_core::Disposition::DropFile,
                "{version}"
            );
            assert_eq!(
                message.context,
                [("file".to_owned(), format!("{dropped}.dds"))],
                "{version}"
            );
            let (kit, charge) = report
                .manifest
                .tasks
                .iter()
                .find_map(|task| {
                    if let TaskKind::Kit { kit, .. } = &task.kind {
                        Some((kit, task.charge))
                    } else {
                        None
                    }
                })
                .expect("the kit's task");
            let stems: Vec<&str> = kit
                .textures
                .iter()
                .map(|texture| texture.stem.as_str())
                .collect();
            assert_eq!(stems, ["kit", kept], "{version}");
            assert_eq!(
                charge, 3,
                "{version}: the config, the kit and the map read are read, the other is not"
            );
        }
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
        let models = FolderModels::of(&folder.path, &folder.files, Engine::Fox);
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
        let models = FolderModels::of_player(&folder, Engine::Fox);
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

    /// The role of each file of `names` in `Players/03 - A` on a target of `engine`.
    fn engine_roles(names: &[&str], engine: Engine) -> Vec<Option<PlayerFile>> {
        let folder = folder(names);
        let models = FolderModels::of_player(&folder, engine);
        folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect()
    }

    fn pre_fox_model(xml_type: &str) -> Option<PlayerFile> {
        Some(PlayerFile::PreFoxModel {
            xml_type: xml_type.to_owned(),
        })
    }

    #[test]
    fn on_pre_fox_a_model_a_material_set_and_an_fmdl_to_convert_have_roles_and_the_fclo_none() {
        assert_eq!(
            engine_roles(
                &[
                    "face_high.model",
                    "face_high.mtl",
                    "skin.dds",
                    "face_diff.bin",
                    "boots/x.model",
                    "gloves/left.model",
                    "hat.fmdl",
                    "fcl_hair_sim.fclo",
                    "pants_kit1.model",
                    "face.xml",
                ],
                Engine::PreFox
            ),
            [
                pre_fox_model("face_neck"),
                Some(PlayerFile::Material),
                Some(PlayerFile::Texture("skin".to_owned(), SourceFormat::Dds)),
                packed(ModelPackage::Face, "face_diff.bin"),
                pre_fox_model("parts"),
                None,
                pre_fox_model("parts"),
                None,
                pre_fox_model("parts"),
                Some(PlayerFile::FaceXml),
            ]
        );
        // On Fox the same files are a model the Models task converts and its material set.
        assert_eq!(
            engine_roles(&["face_high.model", "face_high.mtl"], Engine::Fox),
            [
                model(ModelPackage::Face, "face_high"),
                Some(PlayerFile::Material)
            ]
        );
        // A glove names its hand, and a model in `face/` takes its own name's type.
        assert_eq!(
            engine_roles(
                &[
                    "gloves/keeper_glove_r.model",
                    "gloves/x_handL.model",
                    "face/visor_ratio_2_parts.model",
                    "common/hat.model",
                    "common/hat.mtl",
                ],
                Engine::PreFox
            ),
            [
                pre_fox_model("gloveR"),
                pre_fox_model("handL"),
                pre_fox_model("parts"),
                None,
                None,
            ]
        );
    }

    #[test]
    fn on_pre_fox_a_member_s_own_face_xml_is_a_face_file_and_gives_the_folder_a_face() {
        // In the folder or in `face/`, in any case; nowhere else, and never on Fox.
        let names = ["face.xml", "face/Face.XML", "boots/face.xml", "other.xml"];
        assert_eq!(
            engine_roles(&names, Engine::PreFox),
            [
                Some(PlayerFile::FaceXml),
                Some(PlayerFile::FaceXml),
                None,
                None
            ]
        );
        assert_eq!(engine_roles(&names, Engine::Fox), [None, None, None, None]);
        assert_eq!(PlayerFile::FaceXml.package(), Some(ModelPackage::Face));
        // A folder holding no model but its xml has a face: its face diff is used.
        assert_eq!(
            engine_roles(&["face.xml", "face_diff.bin"], Engine::PreFox),
            [
                Some(PlayerFile::FaceXml),
                packed(ModelPackage::Face, "face_diff.bin")
            ]
        );
        assert_eq!(
            engine_roles(&["face_diff.bin"], Engine::PreFox),
            [Some(PlayerFile::UnusedFaceFile)]
        );
        // Under `ingame_face` there is no face: the xml is not used.
        let marked = PlayerFolder {
            ingame_face: true,
            ..folder(&["kit_boots.model", "face.xml"])
        };
        let models = FolderModels::of_player(&marked, Engine::PreFox);
        let roles: Vec<Option<PlayerFile>> = marked
            .files
            .iter()
            .map(|file| player_file(&marked.path, file, &models))
            .collect();
        assert_eq!(roles[1], Some(PlayerFile::UnusedFaceFile));
    }

    #[test]
    fn on_pre_fox_a_common_link_to_a_model_a_mtl_or_a_texture_has_a_role() {
        let common_model = |xml_type: &str| {
            Some(PlayerFile::PreFoxCommonModel {
                xml_type: xml_type.to_owned(),
            })
        };
        assert_eq!(
            engine_roles(
                &[
                    "legs.model.common",
                    "face_high.model.common",
                    "body.mtl.common",
                    "hair.png.common",
                    "x.fmdl.common",
                    "common/legs.model.common",
                    "common/body.mtl.common",
                    "boots/Legs.MODEL.common.txt",
                    "gloves/left.model.common",
                    "pants_kit1.model.common",
                ],
                Engine::PreFox
            ),
            [
                common_model("parts"),
                common_model("face_neck"),
                Some(PlayerFile::CommonMaterial),
                Some(PlayerFile::CommonTexture("hair".to_owned())),
                None,
                None,
                None,
                common_model("parts"),
                None,
                None,
            ]
        );
        // A typed model link is a face model: the face diff beside it is used.
        assert_eq!(
            engine_roles(&["face_diff.bin", "legs.model.common"], Engine::PreFox),
            [
                packed(ModelPackage::Face, "face_diff.bin"),
                common_model("parts")
            ]
        );
        assert_eq!(
            engine_roles(&["face_diff.bin", "body.mtl.common"], Engine::PreFox),
            [
                Some(PlayerFile::UnusedFaceFile),
                Some(PlayerFile::CommonMaterial)
            ]
        );
        // The team's files: packed into no package of the player's.
        assert_eq!(common_model("parts").unwrap().package(), None);
        assert_eq!(PlayerFile::CommonMaterial.package(), None);
    }

    #[test]
    fn on_pre_fox_an_fmdl_is_converted_unless_a_model_of_its_stem_beats_it() {
        // No `.model` of its stem: the face converts it, the `.skl` of its stem its bind pose.
        assert_eq!(
            engine_roles(
                &[
                    "fcl_hair.fmdl",
                    "Fcl_Hair.skl",
                    "boots.skl",
                    "fcl_hair_sim.fclo"
                ],
                Engine::PreFox
            ),
            [
                pre_fox_model("parts"),
                Some(PlayerFile::ConversionSkeleton),
                None,
                None,
            ]
        );
        // A `.model` of its stem beats it, and its skeleton pairs nothing; a model of its
        // name in another directory does not.
        assert_eq!(
            engine_roles(
                &[
                    "boots.fmdl",
                    "boots.model",
                    "boots.skl",
                    "face/hat.model",
                    "hat.fmdl"
                ],
                Engine::PreFox
            ),
            [
                None,
                pre_fox_model("parts"),
                None,
                pre_fox_model("parts"),
                pre_fox_model("parts"),
            ]
        );
        // A converted FMDL is a face model: the face diff beside it is used.
        assert_eq!(
            engine_roles(&["face_diff.xml", "hat.fmdl"], Engine::PreFox),
            [Some(PlayerFile::FaceDiffXml), pre_fox_model("parts")]
        );
        assert_eq!(PlayerFile::ConversionSkeleton.package(), None);
    }

    #[test]
    fn on_fox_a_model_is_converted_unless_an_fmdl_of_its_stem_beats_it() {
        // No `.fmdl` of its stem: it takes the role an FMDL of its name would, and its `.mtl`
        // is a material set, wherever it sits but in `common/`.
        assert_eq!(
            engine_roles(
                &[
                    "boots.model",
                    "boots.mtl",
                    "x_hair_high.model",
                    "face/hat.model",
                    "gloves/keeper_glove_r.model",
                    "materials.mtl",
                    "boots/materials.mtl",
                    "common/hat.mtl",
                ],
                Engine::Fox
            ),
            [
                model(ModelPackage::Boots, "boots"),
                Some(PlayerFile::Material),
                model(ModelPackage::Face, "hair_high"),
                model(ModelPackage::Face, "fcl_hair"),
                model(ModelPackage::Gloves, "glove_r"),
                Some(PlayerFile::Material),
                Some(PlayerFile::Material),
                None,
            ]
        );
        // An FMDL of its stem beats it, its `.mtl` keeping the role nothing reads; one of its
        // name in another directory does not, nor does one of another stem.
        assert_eq!(
            engine_roles(
                &[
                    "boots.fmdl",
                    "Boots.model",
                    "boots.mtl",
                    "boots/boots.fmdl",
                    "fcl_hair.model",
                    "fcl_hair_x.fmdl",
                ],
                Engine::Fox
            ),
            [
                model(ModelPackage::Boots, "boots"),
                None,
                Some(PlayerFile::Material),
                model(ModelPackage::Boots, "boots"),
                model(ModelPackage::Face, "fcl_hair"),
                model(ModelPackage::Face, "fcl_hair"),
            ]
        );
        // Under `ingame_face` a converted model takes the role an FMDL of its name would.
        let marked = |names: &[&str]| -> Vec<Option<PlayerFile>> {
            let folder = PlayerFolder {
                ingame_face: true,
                ..folder(names)
            };
            let models = FolderModels::of_player(&folder, Engine::Fox);
            folder
                .files
                .iter()
                .map(|file| player_file(&folder.path, file, &models))
                .collect()
        };
        assert_eq!(
            marked(&["torso.model", "torso.skl"]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl")
            ]
        );
    }

    #[test]
    fn a_model_is_beaten_by_one_of_the_target_s_format_of_its_path_stem() {
        let folder = folder(&[
            "boots.fmdl",
            "Boots.model",
            "boots.mtl",
            "boots/boots.model",
            "face/hat.fmdl",
            "hat.model",
            // A glTF is beaten by the target's format and beats the other engine's.
            "boots.glb",
            "face/hat.glb",
            "boots/boots.gltf",
        ]);
        for (engine, expected) in [
            (
                Engine::Fox,
                [false, true, false, true, false, false, true, true, false],
            ),
            (
                Engine::PreFox,
                [true, false, false, false, true, false, true, false, true],
            ),
        ] {
            let models = FolderModels::of_player(&folder, engine);
            let beaten: Vec<bool> = folder
                .files
                .iter()
                .map(|file| models.beaten(file))
                .collect();
            assert_eq!(beaten, expected, "{engine:?}");
        }
    }

    #[test]
    fn a_gltf_beats_the_other_engine_s_model_of_its_stem_and_loses_to_the_target_s() {
        let gltf = || Some(PlayerFile::UnsupportedGltf);
        for engine in [Engine::Fox, Engine::PreFox] {
            // Alone, in either spelling and at any position, a glTF is its stem's selection.
            for name in ["boots.glb", "boots.gltf", "Boots.GLB", "boots/boots.glb"] {
                assert_eq!(engine_roles(&[name], engine), [gltf()], "{name} {engine:?}");
            }
            // In another directory of the folder it is another stem: the FMDL and the `.model`
            // beside it decide nothing about it.
            assert_eq!(
                engine_roles(&["boots/boots.glb", "boots.fmdl", "boots.model"], engine)[0],
                gltf(),
                "{engine:?}"
            );
        }
        for extension in ["glb", "gltf"] {
            let gltf_name = format!("boots.{extension}");
            // Fox: the FMDL beats the glTF, and the glTF the `.model`.
            assert_eq!(
                engine_roles(&["boots.fmdl", &gltf_name], Engine::Fox),
                [model(ModelPackage::Boots, "boots"), None],
                "{gltf_name}"
            );
            assert_eq!(
                engine_roles(&[&gltf_name, "Boots.model"], Engine::Fox),
                [gltf(), None],
                "{gltf_name}"
            );
            // Pre-Fox: the glTF beats the FMDL, and the `.model` the glTF.
            assert_eq!(
                engine_roles(&["boots.fmdl", &gltf_name], Engine::PreFox),
                [None, gltf()],
                "{gltf_name}"
            );
            assert_eq!(
                engine_roles(&[&gltf_name, "boots.model"], Engine::PreFox),
                [None, pre_fox_model("parts")],
                "{gltf_name}"
            );
        }
        // A model the glTF beats is no model of the folder's: no skeleton pairs with it.
        assert_eq!(
            engine_roles(&["boots.glb", "boots.model", "boots.skl"], Engine::Fox),
            [gltf(), None, None]
        );
        assert_eq!(
            engine_roles(&["boots.fmdl", "boots.glb", "boots.skl"], Engine::PreFox),
            [None, gltf(), None]
        );
    }

    #[test]
    fn on_fox_a_converted_model_is_a_model_of_the_folder_its_skeleton_pairs_with() {
        // The `.skl` of a converted model's stem packs under its slot, as an FMDL's does; a
        // slotless model's is `skl_no_slot`; the face files beside a converted face model are
        // used.
        assert_eq!(
            engine_roles(
                &[
                    "kit_boots.model",
                    "Kit_Boots.skl",
                    "fcl_hair.model",
                    "fcl_hair.skl",
                    "face_high.model",
                    "face_high.skl",
                    "face_diff.bin",
                    "fcl_hair_sim.fclo",
                ],
                Engine::Fox
            ),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl"),
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl"),
                model(ModelPackage::Face, "face_high"),
                Some(PlayerFile::SlotlessSkeleton),
                packed(ModelPackage::Face, "face_diff.bin"),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo"),
            ]
        );
        assert_eq!(
            engine_roles(&["boots.fmdl", "hat.model", "hat.skl"], Engine::Fox),
            [
                model(ModelPackage::Boots, "boots"),
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl"),
            ]
        );
        // A converted face model gives the folder a face, which the boots alone do not.
        assert_eq!(
            engine_roles(
                &["face_diff.bin", "boots.fmdl", "face_high.model"],
                Engine::Fox
            )[0],
            packed(ModelPackage::Face, "face_diff.bin")
        );
        assert_eq!(
            engine_roles(&["face_diff.bin", "boots.fmdl", "boots.model"], Engine::Fox)[0],
            Some(PlayerFile::UnusedFaceFile)
        );
    }

    #[test]
    fn on_pre_fox_a_face_file_is_used_only_beside_a_model_with_a_role() {
        assert_eq!(
            engine_roles(&["face_diff.xml", "gloves/hat.fmdl"], Engine::PreFox),
            [Some(PlayerFile::UnusedFaceFile), None]
        );
        assert_eq!(
            engine_roles(&["face_diff.xml", "boots/hat.model"], Engine::PreFox),
            [Some(PlayerFile::FaceDiffXml), pre_fox_model("parts")]
        );
    }

    #[test]
    fn on_pre_fox_under_ingame_face_every_model_but_a_glove_is_a_boots_part() {
        let names = [
            "shirt.model",
            "kit_boots.model",
            "torso.model",
            "shirt.mtl",
            "face_diff.bin",
            "x_gloveL.model",
            "gloves/keeper_handR.model",
        ];
        // Each typed as without the marker: a glove's type is what its `glove.xml` lists.
        let part = |package, xml_type: &str| {
            Some(PlayerFile::PreFoxPart {
                package,
                xml_type: xml_type.to_owned(),
            })
        };
        let folder = PlayerFolder {
            ingame_face: true,
            ..folder(&names)
        };
        let models = FolderModels::of_player(&folder, Engine::PreFox);
        let marked: Vec<Option<PlayerFile>> = folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect();
        assert_eq!(
            marked,
            [
                part(ModelPackage::Boots, "shirt"),
                part(ModelPackage::Boots, "parts"),
                part(ModelPackage::Boots, "parts"),
                Some(PlayerFile::Material),
                Some(PlayerFile::UnusedFaceFile),
                part(ModelPackage::Gloves, "gloveL"),
                part(ModelPackage::Gloves, "handR"),
            ]
        );
        assert_eq!(
            part(ModelPackage::Boots, "parts").unwrap().package(),
            Some(ModelPackage::Boots)
        );
        // Without the marker the same files are the face's.
        assert_eq!(
            engine_roles(&names, Engine::PreFox),
            [
                pre_fox_model("shirt"),
                pre_fox_model("parts"),
                pre_fox_model("parts"),
                Some(PlayerFile::Material),
                packed(ModelPackage::Face, "face_diff.bin"),
                pre_fox_model("gloveL"),
                pre_fox_model("handR"),
            ]
        );
    }

    #[test]
    fn on_pre_fox_under_ingame_face_a_common_model_link_is_a_part_as_a_model_of_its_stem_is() {
        let names = [
            "glove_l.model.common",
            "kit_boots.model.common",
            "legs.model.common",
            "gloves/keeper_handR.model.common",
            "body.mtl.common",
            "hair.png.common",
        ];
        let part = |package, xml_type: &str| {
            Some(PlayerFile::PreFoxPart {
                package,
                xml_type: xml_type.to_owned(),
            })
        };
        let folder = PlayerFolder {
            ingame_face: true,
            ..folder(&names)
        };
        let models = FolderModels::of_player(&folder, Engine::PreFox);
        let marked: Vec<Option<PlayerFile>> = folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect();
        assert_eq!(
            marked,
            [
                part(ModelPackage::Gloves, "gloveL"),
                part(ModelPackage::Boots, "parts"),
                part(ModelPackage::Boots, "parts"),
                part(ModelPackage::Gloves, "handR"),
                Some(PlayerFile::CommonMaterial),
                Some(PlayerFile::CommonTexture("hair".to_owned())),
            ]
        );
        // Without the marker each model link is listed by reference in the face's `face.xml`.
        let common_model = |xml_type: &str| {
            Some(PlayerFile::PreFoxCommonModel {
                xml_type: xml_type.to_owned(),
            })
        };
        assert_eq!(
            engine_roles(&names, Engine::PreFox),
            [
                common_model("gloveL"),
                common_model("parts"),
                common_model("parts"),
                common_model("handR"),
                Some(PlayerFile::CommonMaterial),
                Some(PlayerFile::CommonTexture("hair".to_owned())),
            ]
        );
        // A `.mtl` link is compiled under the marker; a model link never reaches the check
        // as one.
        assert!(compiled_under_ingame_face(&PlayerFile::CommonMaterial));
        assert!(!compiled_under_ingame_face(&common_model("parts").unwrap()));
    }

    #[test]
    fn on_pre_fox_a_per_kit_model_is_typed_as_its_set_where_a_face_xml_lists_it() {
        assert_eq!(
            pre_fox_model_type(Position::Direct, "pants_kit1"),
            Some("parts".to_owned())
        );
        assert_eq!(
            pre_fox_model_type(Position::Direct, "boots_kit1"),
            Some("parts".to_owned())
        );
        // Behind a link, or under `ingame_face`, no `face.xml` entry names the set.
        assert_eq!(
            pre_fox_link(Position::Direct, "pants_kit1.model.common"),
            None
        );
        let folder = PlayerFolder {
            ingame_face: true,
            ..folder(&["boots_kit1.model"])
        };
        let models = FolderModels::of_player(&folder, Engine::PreFox);
        assert_eq!(player_file(&folder.path, &folder.files[0], &models), None);
    }

    #[test]
    fn on_pre_fox_a_shared_boots_or_gloves_folder_s_per_kit_model_is_named() {
        let names = ["boots_kit1.model", "boots_kit2.model", "boots.mtl"];
        assert_eq!(
            pre_fox_shared_not_compiled(
                SharedKind::Boots,
                &shared_folder(SharedKind::Boots, &names)
            ),
            what("Boots/Crocs/boots_kit1.model")
        );
        // A shared face's set is copied into each linking player's face, which lists it.
        assert_eq!(
            pre_fox_shared_not_compiled(SharedKind::Face, &shared_folder(SharedKind::Face, &names)),
            None
        );
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
            // A `.model` the FMDL of its stem beats, and a `.mtl` where only textures go.
            "kit_boots.model",
            "Fcl_Hair.model",
            "common/body.mtl",
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
        let models = FolderModels::of(&folder.path, &folder.files, Engine::Fox).with_linked_face();
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
