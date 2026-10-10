//! A player's pre-Fox face (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps",
//! steps 1, 4, 6 and 7): his folder's `.model` files, and those of the shared face folder he
//! links, his `.fmdl` files converted to `.model` files with their material sets,
//! packed under their `oral_<stem>_win32.model` names, the `.mtl` files with their
//! texture paths pointed at his common folder (a texture link's at the team's Common output),
//! each file at its path below its folder (`jessie/body/oral_x_win32.model`),
//! and the generated `face.xml` typing every model, his `.common` links to a Common model
//! included, his face diff as its `<dif>`. A per-kit model set is packed whole and listed once,
//! its kit token spelled `kitN` (`team_compiler/pipeline.md` "4. Per-export non-model steps",
//! Kit-dependent assets). A face with no `face_neck` model, the blank face of a folder with no
//! model included, gets the bundled dummy as one. A face with the member's own `face.xml`, his
//! folder's or his linked shared face's, gets that xml written back instead, with only the
//! files it names packed, and beside a shared face's xml the generated entries of his own
//! models it does not name appended (`user_xml_face`).
//! `materialize` packs the files into the face CPK.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use aesthetics_export::{
    FileDescriptor, FileKind, KitToken, ModelFormat, ModelSuffix, common_link_target, kit_token,
    variant_stem,
};
use pes_model::format::mtl::{Address, Filter, MaterialEntry, MaterialSet, Sampler};
use pes_model::ops::paths::rewrite_texture_paths;
use pes_version::{Engine, PesVersion};
use studio_core::Disposition;
use vtree::ScopePath;

use super::conversion::{
    ConvertedMaterials, PreFoxConversion, fmdl_for_pre_fox, model_for_pre_fox, source_name,
};
use super::materialize::PackageFiles;
use super::prefox_split::split_face_model;
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::deep::relative;
use crate::face_diff;
use crate::face_xml::{
    WrittenChild, XmlEntry, face_xml, packed_model_name, ratio, suffix, user_face_xml,
    version_type, xml_path,
};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::paths::{self, TextureDirectory, TextureHome};
use crate::plan::roles::{
    ModelPackage, PlayerFile, below_common, common_file, file_stem, is_common_file, path_stem,
    role_position, selected_common_model,
};
use crate::plan::{ENVIRONMENT_MAP_STEM, ModelFolder};
use crate::texture_lookup::{self, TextureFolders, TexturePlace};
use crate::user_face_xml::{
    Child, FaceFiles, ModelElement, Reference, UserFaceXml, parse, reference, resolve, variant_of,
};

/// The `face.xml` type the game loads a player's face model as. A face whose models include
/// none of it gets the dummy as its `face_neck`.
const FACE_NECK: &str = "face_neck";

/// The referee template's body (`blue_port.md` "The referee body"): each model's name below
/// the template tree's `common/999/refkit/`, without `_win32.model`, and the `face.xml` type
/// it is listed under, in the order a cup referee's own xml listed them in game.
const REFKIT_BODY: [(&str, &str); 10] = [
    ("oral_arm", "parts"),
    ("oral_thigh", "parts"),
    ("oral_refshirt", "parts"),
    ("oral_pants", "parts"),
    ("oral_pants_sub", "parts"),
    ("oral_sleeve", "parts"),
    ("oral_socks", "parts"),
    ("oral_hand_l", "gloveL"),
    ("oral_hand_r", "gloveR"),
    ("oral_boots", "boots"),
];

/// The `face.xml` types a model dressing a hand is listed under: a glove's and a hand's, both
/// the gloves' on Fox (`suffix_role`).
const HAND_TYPES: [&str; 4] = ["gloveL", "gloveR", "handL", "handR"];

/// One `.model` of the face, or an `.fmdl` converted to one, with the source folder it was
/// found in: its `.mtl` is searched for there (`mtl_for`), a shared face's never in the
/// player's folder.
struct FaceModel<'a> {
    /// The model file, or the `.common` link to a model of the team's Common output.
    file: &'a FileDescriptor,
    /// The model's stem: the linked model's for a link (`legs` for `legs.model.common`).
    stem: String,
    /// Its `face.xml` type.
    xml_type: String,
    /// The name it is packed under (`packed_model_name`), in the face or, for a link, in the
    /// team's Common output.
    packed: String,
    /// The directory it sits in below its source folder, `/`-terminated (`jessie/body/`),
    /// empty for a file directly in it: the face packs it, its hands and a converted model's
    /// material set in that directory, and lists them there. For a link, the linked model's
    /// directory below `Common/` (`jessie/` for `jessie/legs.model.common`, empty for a root
    /// link), where the team's Common output holds it and the face lists it.
    directory: String,
    /// Whether it is a link: the game loads the model from the team's Common output, and the
    /// face packs nothing of it.
    in_common: bool,
    /// Where its packed bytes and its `.mtl` come from.
    source: FaceSource<'a>,
    /// Its source folder's export path.
    source_path: &'a ScopePath,
}

/// Where a face model's packed bytes and its `.mtl` come from. One value carries both, so a
/// converted `.model` is never packed with a member's `.mtl`, nor a member's with a converted
/// one.
enum FaceSource<'a> {
    /// The member's own model, packed from its file, or moved onto the version's skeleton
    /// when the conversion pre-check flags it (`model_for_pre_fox`), and split when it is
    /// hand-split; named with the `.mtl` its search finds from its source folder (`mtl_for`),
    /// packed as the member wrote it whichever way the model is.
    Member { material: &'a FileDescriptor },
    /// An FMDL converted for the target (`fmdl_for_pre_fox`): the `.model` written, packed in
    /// its place, and its material set, packed as `<stem>.mtl` (`converted_material_name`).
    Converted(PreFoxConversion),
    /// A Common FMDL a `.common` link names, converted once by the export's Common models task
    /// (`prefox_common`): the face packs nothing of it and names its material set, `<stem>.mtl`
    /// in the team's Common output. The player's own `.mtl` files do not layer over it, as they
    /// do over a Common `.model`'s ("Common-linked models bring their own materials" in
    /// `model_format.md`): the FMDL carries its materials, and the set is the conversion's.
    CommonConversion,
}

/// The name a converted model's material set is packed under: `<stem>.mtl`, the stem as the
/// FMDL spells it.
pub(super) fn converted_material_name(stem: &str) -> String {
    format!("{stem}.mtl")
}

/// The directory the file at `path` sits in below its source folder at `source_path`,
/// `/`-terminated as spelled (`jessie/body/`), empty for a file directly in it: where the face
/// packs the file, as the player folder holds it (`player_folders.md` "Subfolders").
fn directory_below(path: &ScopePath, source_path: &ScopePath) -> String {
    texture_lookup::split(&relative(path, source_path))
        .0
        .to_owned()
}

/// A face model's place in a per-kit set the face holds (`kit_places`).
enum KitPlace {
    /// Not a variant, or a variant with no other of its set: an ordinary model.
    Alone,
    /// The set's lowest variant, of kit number `kit`: its entries stand for the set, their kit
    /// token spelled `kitN`.
    Listed { kit: u8 },
    /// Another variant of the set, of kit number `kit`, whose lowest variant is the face's
    /// model at index `listed`: packed under its own name, listed by nothing.
    Unlisted { kit: u8, listed: usize },
}

impl KitPlace {
    /// The name an entry gives the model packed as `packed`: a set's lowest variant's with its
    /// kit token spelled `kitN` (`oral_pants_kit1_win32.model` is named
    /// `oral_pants_kitN_win32.model`), for the game to respell for the kit picked; any other
    /// model's as it is packed.
    fn entry_name(&self, packed: &str) -> String {
        match self {
            KitPlace::Listed { .. } => {
                let stem = file_stem(packed);
                // `packed_model_name` lower-cases the stem and adds `_`-delimited affixes, so
                // the variant's token, already lower case, stays a token where it was. The
                // reference is respelled here, after packing, because lower-casing it would
                // lose its capital `N`.
                let (_, reference) = kit_token(stem).expect(
                    "a per-kit model's packed name keeps its kit token (`packed_model_name`)",
                );
                format!("{reference}{}", &packed[stem.len()..])
            }
            KitPlace::Alone | KitPlace::Unlisted { .. } => packed.to_owned(),
        }
    }
}

/// The place of each of `models`, the face's models in their order, in the per-kit sets the
/// face holds: the models packed in the face (not a link's) in one directory whose packed
/// names differ only in their kit token's digit form a set, its lowest number listed (of two
/// of one number, the first in `models`) and its others unlisted, and a set of one is an
/// ordinary model. Read from the packed paths, so a set split between a linked shared face and
/// the player's own root files is the one set the face holds once the shared files are copied
/// in under his; a set never spans two directories, since the game respells the listed
/// variant's entry, directory included, for the kit picked.
fn kit_places(models: &[FaceModel]) -> Vec<KitPlace> {
    // Each set's variants, by its folded directory and reference: (kit number, index), in
    // `models`' order.
    let mut sets: BTreeMap<String, Vec<(u8, usize)>> = BTreeMap::new();
    for (index, model) in models.iter().enumerate() {
        if model.in_common {
            continue;
        }
        if let Some((KitToken::Variant(kit), reference)) = kit_token(file_stem(&model.packed)) {
            sets.entry(vtree::fold_name(&format!("{}{reference}", model.directory)))
                .or_default()
                .push((kit, index));
        }
    }
    let mut places: Vec<KitPlace> = models.iter().map(|_| KitPlace::Alone).collect();
    for mut variants in sets.into_values() {
        // A stable sort: of two models of one number, the first in `models` is listed.
        variants.sort_by_key(|(kit, _)| *kit);
        let Some(((listed_kit, listed), others)) = variants.split_first() else {
            continue;
        };
        if others.is_empty() {
            continue;
        }
        places[*listed] = KitPlace::Listed { kit: *listed_kit };
        for (kit, index) in others {
            places[*index] = KitPlace::Unlisted {
                kit: *kit,
                listed: *listed,
            };
        }
    }
    places
}

/// A per-kit set a generated `face.xml` lists, short of some of the export's kit numbers: the
/// files to pack again for each number it lacks, and what its `kit_variant_missing` names.
struct SetCompletion {
    /// The set's reference, as its lowest variant's stem spells it, after the set's directory
    /// below its source folder (`pants_kitN`, `jessie/pants_kitN`).
    reference: String,
    /// The lowest variant's stem, whose files are copied, after the same directory
    /// (`pants_kit1`, `jessie/pants_kit1`).
    copied: String,
    /// The export's kit numbers the set has no variant of, ascending.
    missing: Vec<u8>,
    /// The paths, in the face, of the files the lowest variant's entries name that the face
    /// packs: its model, its hands' when it is hand-split, and its `.mtl` when that carries
    /// the variant's token and is the face's (a Common one is the Common output's, and a
    /// shared `pants.mtl` serves every number as it is).
    names: Vec<String>,
}

/// The completion of each per-kit set `models` holds (`places`, from `kit_places`) against
/// `kit_numbers`, the export's kit numbers, each model's entries naming the `material` of
/// `written` (directory, name); a set lacking none of them needs none. `hand_split` holds the
/// export paths of the models split at the wrists, whose hands are packed beside them.
fn set_completions(
    models: &[FaceModel],
    places: &[KitPlace],
    written: &[(String, String)],
    hand_split: &BTreeSet<ScopePath>,
    kit_numbers: &[u8],
) -> Vec<SetCompletion> {
    let mut completions = Vec::new();
    for (index, ((model, place), (directory, material))) in
        models.iter().zip(places).zip(written).enumerate()
    {
        let KitPlace::Listed { kit } = place else {
            continue;
        };
        // `kit_places` reads the packed name, lower-cased; a stem whose token is not spelled
        // `kit1` to `kit9` exactly (`pants_KIT1`) is no variant (`pipeline.md` "Kit-dependent
        // assets"), so it has no reference to complete.
        let Some((_, reference)) = kit_token(&model.stem) else {
            continue;
        };
        let others = places.iter().filter_map(|other| match other {
            KitPlace::Unlisted { kit, listed } if *listed == index => Some(*kit),
            KitPlace::Unlisted { .. } | KitPlace::Alone | KitPlace::Listed { .. } => None,
        });
        let held: Vec<u8> = iter::once(*kit).chain(others).collect();
        let missing: Vec<u8> = kit_numbers
            .iter()
            .copied()
            .filter(|number| !held.contains(number))
            .collect();
        if missing.is_empty() {
            continue;
        }
        let mut names = vec![format!("{}{}", model.directory, model.packed)];
        if hand_split.contains(&model.file.path) {
            names.extend(["glove_l", "glove_r"].map(|hand| {
                let packed = packed_model_name(&format!("{}_{hand}", model.stem));
                format!("{}{packed}", model.directory)
            }));
        }
        // The entry's `material` is respelled `kitN` only when the `.mtl` carries the
        // variant's token; the face packs it under the listed variant's spelling. A `./`
        // directory is the face's own, the `.mtl` packed below it; any other is the team's
        // Common output.
        let packed_material = respelled_material(material, *kit);
        if let Some(below) = directory.strip_prefix("./")
            && packed_material != *material
        {
            names.push(format!("{below}{packed_material}"));
        }
        // Named with the set's directory, as a member's `face.xml` names a subfolder's set: a
        // root set of the same stems is another set.
        completions.push(SetCompletion {
            reference: format!("{}{reference}", model.directory),
            copied: format!("{}{}", model.directory, model.stem),
            missing,
            names,
        });
    }
    completions
}

/// The file path `path`, whose file name holds a kit token, with that token spelled for `kit`
/// (`oral_pants_kit1_win32.model` and 3 give `oral_pants_kit3_win32.model`,
/// `jessie/oral_pants_kit1_win32.model` gives `jessie/oral_pants_kit3_win32.model`); the
/// directory is kept as it is.
fn variant_name(path: &str, kit: u8) -> String {
    let (directory, name) = texture_lookup::split(path);
    let stem = file_stem(name);
    let variant = variant_stem(stem, kit).expect("a per-kit set's file names hold a kit token");
    format!("{directory}{variant}{}", &name[stem.len()..])
}

/// Packs the file `contents` holds as `name`, its path in the face (case-folded), again under
/// its kit-`kit` spelling in its directory (`variant_name`), its bytes as packed, and returns
/// whether it did. Nothing is packed when
/// a file of that spelling, case-folded, is packed already, the member's own being the one
/// for that number (a `pants_kit3.mtl` of his beside a set with no `pants_kit3` model), or
/// when `contents` holds no `name`: a hand-split model that was all hand packs no body.
fn pack_variant_copy(contents: &mut PackageFiles, name: &str, kit: u8) -> bool {
    let copy = variant_name(name, kit);
    let copy_key = vtree::fold_name(&copy);
    if contents
        .keys()
        .any(|packed| vtree::fold_name(packed) == copy_key)
    {
        return false;
    }
    let key = vtree::fold_name(name);
    let Some(bytes) = contents
        .iter()
        .find(|(packed, _)| vtree::fold_name(packed) == key)
        .map(|(_, bytes)| bytes)
    else {
        return false;
    };
    // Each name of the package owns its bytes: the copy is a second buffer.
    let bytes = bytes.clone();
    contents.insert(copy, bytes);
    true
}

/// The `kit_variant_missing` of a per-kit model set, its reference spelled `reference`
/// (`pants_kitN`), completed for kit `kit` with the files of its lowest variant, `copied`
/// (`pants_kit1`).
fn model_variant_missing(reference: &str, kit: u8, copied: &str) -> Finding {
    (
        Code::KitVariantMissing,
        Disposition::Keep,
        vec![
            ("model", reference.to_owned()),
            ("kit", kit.to_string()),
            ("copied", copied.to_owned()),
        ],
    )
}

/// The files of `folder`'s pre-Fox face, compiled from its files' bytes in `files` for team
/// `team_id`, by their names in the face CPK. The face reads the sources feeding it (the
/// folder's own files and a combined shared face's); of a combined boots or gloves folder,
/// which its own package writes, it reads only the texture stems. Each `.model` under its
/// packed name (`packed_model_name`) and an entry of the `face.xml`, in the order of the
/// models' export paths, case-folded, each naming the `.mtl` its search finds from its own
/// source folder, the model moved onto `ctx.version`'s skeleton first when the conversion
/// pre-check flags it (`model_for_pre_fox`, reading that `.mtl`, a Common one too), its `.mtl`
/// packed as written; an `.fmdl` the face converts (`PlayerFile::PreFoxModel`) the same, as
/// the `.model` its conversion writes (`fmdl_for_pre_fox`, the `.skl` of its path stem as the
/// bind pose), named with the conversion's material set, packed as `<stem>.mtl` with its
/// texture paths pointed as a `.mtl`'s below; each `.mtl` under its own name, every texture
/// path naming one of the folder's textures by its stem pointed at the folder's texture home
/// as that texture's DDS, and one naming a stem a
/// texture link of the folder stands for at that texture in the team's Common output; and the
/// `face.xml`, its `<dif>` the folder's face diff (`face_diff.bin`, else `face_diff.xml`
/// decoded, else the bundled one). A `.common` link to a model is an entry naming the model in
/// the team's Common output, where the export's Common models task packs it, or converts a
/// Common FMDL, the entry then naming the converted `<stem>.mtl` there; a `.mtl` the
/// search finds in `Common/`, directly or through a link, is named there too, and the face
/// packs neither (`pipeline.md` "3. Per-model-folder parallel steps", step 4). A file of a
/// subfolder of its source folder is packed and listed at its path below that folder
/// (`jessie/body/x.model` as `jessie/body/oral_x_win32.model`, listed
/// `./jessie/body/oral_x_*.model`), its hands and converted material set beside it, and its
/// entry names its `.mtl` where the search found it (`./jessie/body/x.mtl` beside it, `./x.mtl`
/// in the root; `player_folders.md` "Subfolders"). A linked shared
/// face's files are copied in under the player's own: a model or `.mtl` packing under a name
/// (case-folded) the player's folder already packs is left out, the player's file replacing it
/// as a copy would (`player_folders.md` "A link plus local models combines"). A hand-split
/// model (`ModelFolder::hand_split`), moved first, is split at the wrists
/// (`split_face_model`): its body is packed and listed in its place, and each hand made
/// follows it as
/// `oral_<stem>_glove_l_win32.model` or `oral_<stem>_glove_r_win32.model`, an entry typed
/// `gloveL` or `gloveR` naming the model's `.mtl` and `ratio`; a split model whose `.mtl` is a
/// Common file is split with that file, every entry naming it in the team's Common output.
/// When no entry is a `face_neck`, the dummy is listed last and packed as `oral_dummy_win32.model` and
/// `dummy.mtl`, noted in `findings` as `xml_face_neck_added` when the face has a model. An
/// entry's type is written for `ctx.version` (`version_type`): each entry whose type that
/// rewrites (`uniform` to `uniform_sub` on PES 2015) is noted in `findings` as
/// `xml_uniform_pes15`, naming its model below its source folder. A per-kit set the face holds
/// (`kit_places`, across its sources once the shared files are copied in) is packed whole,
/// each variant under its own name, and listed once: its lowest variant's entries (its own,
/// and its hands' when it is split) name it with the kit token spelled `kitN`, their
/// `material` too when the `.mtl` carries the variant's own token (`pants_kit1.mtl` is named
/// `pants_kitN.mtl`, a shared `pants.mtl` as it is); the game respells the whole entry for the
/// kit picked. Each other variant has no entry, and one whose own `material`, as an entry
/// would write it (directory included), is not the listed one's respelled for its kit number
/// is noted in `findings` as `kit_variant_mtl_differs`. A set is completed against
/// `kit_numbers`, the export's kit numbers (`set_completions`): for each one it has no variant
/// of, the files its lowest variant's entries name that the face packs are packed again under
/// that number's spelling, as packed, and `kit_variant_missing` is noted in `findings`, since
/// the game skips an entry whose respelled model is missing. Two files of one source packing
/// under one name fail the task: neither can be dropped silently.
///
/// A face with a `face.xml` (`ModelFolder::face_xml`), the member's own or the one of the
/// shared face he links, has no generated xml: the textures are read as above, and the xml is
/// written back by `user_xml_face`, which packs only the models and `.mtl` files its
/// references name, under the names they give, a `kitN` reference's set completed against
/// `kit_numbers` too (`messages.md` "User-supplied `face.xml`"). No
/// model of the source holding the xml is typed by its name, split or listed by kit there; the
/// hand split is not planned for such a face. Beside a shared face's xml, each of the player's
/// own models the xml does not name (`names_model`) takes the route above, its generated entry
/// written after the xml's children, and his `.mtl` files are packed as above. The face's diff
/// is the player's own face diff beside a shared face's xml, else the xml's last `<dif>`, else
/// the face diff of the folder holding the xml, else the bundled one.
///
/// A referee folder holding `fpc_off` (`ModelFolder::refkit_body`) lists the referee
/// template's body after every model of his, generated or his xml's, and before the dummy
/// (`refkit_body`): the face packs none of it, the refs CPK carrying the template's files.
pub(super) fn face(
    folder: &ModelFolder,
    kit_numbers: &[u8],
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<PackageFiles, TaskFailure> {
    let xml = folder.face_xml();
    let mut models = Vec::new();
    let mut materials = Vec::new();
    // The face diff of a source other than the xml's, his own beside a shared face's xml,
    // which wins over the xml's `<dif>` as any of his files wins over the shared folder's; and
    // the one of the source holding the xml, which the `<dif>` replaces. `roles()` gives the
    // face one face diff, so at most one of them is set.
    let mut dif = None;
    let mut xml_source_dif = None;
    // The folder's textures by the folder of its tree holding each, and its combined folders',
    // each with the path its converted DDS has below the texture home (`folder_textures`).
    let mut textures = TextureFolders::default();
    // The stems the folder's texture links stand for, folded, each with the stem of the
    // `Common/` texture the link names: the name its DDS has in the team's Common output.
    let mut linked: BTreeMap<String, String> = BTreeMap::new();
    // The Common models, by linked name folded, the face's model links have listed so far.
    let mut linked_models: BTreeSet<String> = BTreeSet::new();
    // The packed names, folded, of the sources before the one being read: the player's own
    // files come first, so a shared face's file of one of these names is left out.
    let mut earlier: BTreeSet<String> = BTreeSet::new();
    // `roles()` yields the folder's own files, then each combined folder's in `combined`'s
    // order, so each source's roles pair with its own file list here.
    let source_files = iter::once(&folder.files).chain(
        folder
            .combined
            .iter()
            .map(|combined| &combined.folder.files),
    );
    for ((package, source_path, source_roles), source_files) in
        folder.roles().into_iter().zip(source_files)
    {
        // A source feeding another package, a referee's plain boots or gloves link, is that
        // package's task's to write as his slot's `k99NN`/`g99NN` folder: packed here too as
        // `parts` entries, it would dress him twice. Its textures still go to his texture
        // home (the textures task reads every source), so the face's `.mtl` files point a
        // path of one of their stems there.
        let source_roles: Vec<(&FileDescriptor, PlayerFile)> = match package {
            ModelPackage::Face => source_roles,
            ModelPackage::Boots | ModelPackage::Gloves => source_roles
                .into_iter()
                .filter(|(_, role)| matches!(role, PlayerFile::Texture { .. }))
                .collect(),
        };
        // Each model's `.mtl` is resolved here, before any entry: a set's other variants are
        // checked against its listed one's, wherever it sorts.
        // A combined shared folder's search sees no `Common/` file, as the deep pass's does
        // (`deep::pairings`): its `.common` links have no role, and the Common `.mtl` one names
        // is not among the task's files.
        let common: &[FileDescriptor] = if source_path == &folder.path {
            &folder.common_files
        } else {
            &[]
        };
        let material_of = |file: &FileDescriptor| {
            mtl_for(&file.path, source_path, source_files, common).expect(
                "the deep pass drops a folder holding a `.model`, or a link to one, no `.mtl` is \
                 found for (`model_material_undefined`)",
            )
        };
        // The skeletons of this source's FMDLs, each its FMDL's bind pose, paired by path stem.
        let skeletons: Vec<&FileDescriptor> = source_roles
            .iter()
            .filter(|(_, role)| matches!(role, PlayerFile::ConversionSkeleton))
            .map(|(file, _)| *file)
            .collect();
        // The source holding the face's xml: its models and `.mtl` files are the xml's to name.
        let xml_source = xml.is_some_and(|(source, _)| source == source_path);
        let source_dif = if xml_source {
            &mut xml_source_dif
        } else {
            &mut dif
        };
        let mut packed_here = Vec::new();
        for (file, role) in source_roles {
            match role {
                // The xml lists its folder's models (`user_xml_face`), and the deep pass, which
                // compared only the `.mtl` each entry names, does not promise a search finds
                // one for every model (an entry may name none). It packs only the `.mtl`
                // files its entries name.
                PlayerFile::PreFoxModel { .. }
                | PlayerFile::PreFoxCommonModel { .. }
                | PlayerFile::Material
                    if xml_source => {}
                PlayerFile::PreFoxModel { xml_type } => {
                    let stem = file_stem(file.path.name());
                    let packed = packed_model_name(stem);
                    let directory = directory_below(&file.path, source_path);
                    let key = vtree::fold_name(&format!("{directory}{packed}"));
                    if earlier.contains(&key) {
                        continue;
                    }
                    packed_here.push(key);
                    let source = if file.kind == FileKind::Model(ModelFormat::Fmdl) {
                        packed_here.push(vtree::fold_name(&format!(
                            "{directory}{}",
                            converted_material_name(stem)
                        )));
                        let path_fold = vtree::fold_name(path_stem(file));
                        let skeleton = skeletons
                            .iter()
                            .find(|skeleton| vtree::fold_name(path_stem(skeleton)) == path_fold)
                            .map(|skeleton| take(files, skeleton));
                        let bytes = take(files, file);
                        let conversion = fmdl_for_pre_fox(
                            &source_name(&file.path, &folder.path),
                            &bytes,
                            skeleton.as_deref(),
                            ctx,
                            findings,
                            ConvertedMaterials::Converted,
                        )?;
                        FaceSource::Converted(conversion)
                    } else {
                        FaceSource::Member {
                            material: material_of(file),
                        }
                    };
                    models.push(FaceModel {
                        file,
                        stem: stem.to_owned(),
                        xml_type,
                        packed,
                        directory,
                        in_common: false,
                        source,
                        source_path,
                    });
                }
                // Packing nothing into the face, a link takes no name from a shared face's
                // file.
                PlayerFile::PreFoxCommonModel { xml_type } => {
                    let linked_name = common_link_target(&file.path, source_path).expect(
                        "a PreFoxCommonModel role implies a `.common` link below its folder",
                    );
                    // Two spellings of one link (`legs.model.common` and the tolerated
                    // `legs.model.common.txt`) are one model, listed once: the first in the
                    // folder's file order.
                    if !linked_models.insert(vtree::fold_name(&linked_name)) {
                        continue;
                    }
                    let (directory, linked_file) = texture_lookup::split(&linked_name);
                    let stem = file_stem(linked_file);
                    let linked_model =
                        selected_common_model(&folder.common_files, &linked_name, Engine::PreFox)
                            .expect(
                                "validation drops a player folder whose link names no Common file",
                            );
                    // A Common `.model`'s `.mtl` is the one its search finds; a Common FMDL's
                    // material set is its conversion's (`material_of`'s `expect` is for a
                    // `.model`'s search).
                    let source = if linked_model.kind == FileKind::Model(ModelFormat::Fmdl) {
                        FaceSource::CommonConversion
                    } else {
                        FaceSource::Member {
                            material: material_of(file),
                        }
                    };
                    models.push(FaceModel {
                        file,
                        stem: stem.to_owned(),
                        xml_type,
                        packed: packed_model_name(stem),
                        directory: directory.to_owned(),
                        in_common: true,
                        source,
                        source_path,
                    });
                }
                PlayerFile::Material => {
                    let below = relative(&file.path, source_path);
                    let key = vtree::fold_name(&below);
                    if earlier.contains(&key) {
                        continue;
                    }
                    packed_here.push(key);
                    materials.push((file, below));
                }
                // On pre-Fox the face diff is the one packed file (`fcl_hair_sim.fclo` has no
                // role there); here it is the `<dif>`, never a file of the CPK.
                PlayerFile::Packed { .. } => *source_dif = Some(take(files, file)),
                // The deep pass has dropped a folder whose face diff fails to decode, so a
                // failure here is not a member's mistake.
                PlayerFile::FaceDiffXml => {
                    let bytes = face_diff::from_xml(&take(files, file))
                        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
                    *source_dif = Some(bytes);
                }
                PlayerFile::Texture { below, .. } => {
                    textures.insert(source_path == &folder.path, &below);
                }
                PlayerFile::CommonTexture(stem) => {
                    linked.insert(vtree::fold_name(&stem), linked_texture_stem(folder, file));
                }
                // The search resolves a material link where it finds it (`mtl_for`).
                PlayerFile::CommonMaterial => {}
                // Read after the sources, the first one (`ModelFolder::face_xml`).
                PlayerFile::FaceXml => {}
                // Read with the FMDL it is the bind pose of, above.
                PlayerFile::ConversionSkeleton => {}
                // Planning drops a player folder holding one, and `drop_gltf_folders` removes a
                // shared folder whose selected model is one before any task is made, so no
                // task meets it.
                PlayerFile::UnsupportedGltf => {}
                // The Fox roles; a face file with no face model, which is not read; and an
                // `ingame_face` player's part, which has no face.
                PlayerFile::Model { .. }
                | PlayerFile::PreFoxPart { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::UnusedFaceFile
                | PlayerFile::LeftOutKitVariant => {}
            }
        }
        earlier.extend(packed_here);
    }
    // The team's Common output, which a `face.xml` and a `.mtl` name a Common file in.
    let common_directory = paths::common_texture_directory(Engine::PreFox, team_id);
    let common_home = TextureDirectory::plain(common_directory.clone());
    let home = folder.textures.directory(Engine::PreFox, team_id);
    let places = FolderPlaces {
        folder: &folder.path,
        textures,
        linked,
        home: &home,
        common_directory: &common_home,
    };
    // An xml that does not parse is an error: the deep pass dropped its folder.
    let xml = match xml {
        Some((_, file)) => Some(
            parse(&take(files, file))
                .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?,
        ),
        None => None,
    };
    // Beside a shared face's xml, a model of his it names is packed by its entry; the others
    // are appended.
    if let Some(xml) = &xml {
        let named = face_files(folder);
        models.retain(|model| !names_model(xml, model, &named));
    }
    let xml_dif = xml.as_ref().and_then(last_dif).map(<[u8]>::to_vec);
    let dif = dif
        .or(xml_dif)
        .or(xml_source_dif)
        .unwrap_or_else(|| ctx.templates.face_diff().to_vec());
    // By export path, case-folded, then as spelled, so a recompile lists them alike.
    models.sort_by_cached_key(|model| {
        (
            model.file.path.fold_key(),
            model.file.path.as_str().to_owned(),
        )
    });

    let kits = kit_places(&models);
    // The `material` each model's entries write, as its directory and its name: `./` and the
    // `.mtl`'s directory below its source folder where the search found it (`./jessie/body/`
    // beside a subfolder's model, `./` in the root), a converted set's its model's, or, for a
    // `.mtl` the search found in `Common/`, which is the Common output's and never packed
    // here, its directory in the Common output (`jessie/` for `Common/jessie/legs.mtl`); the
    // name respelled for a set's listed variant.
    let written: Vec<(String, String)> = models
        .iter()
        .zip(&kits)
        .map(|(model, kit)| {
            let (directory, name) = match &model.source {
                FaceSource::Member { material } if is_common_file(&material.path) => {
                    let (directory, name) = texture_lookup::split(below_common(&material.path));
                    (format!("{common_directory}{directory}"), name.to_owned())
                }
                FaceSource::Member { material } => (
                    format!("./{}", directory_below(&material.path, model.source_path)),
                    material.path.name().to_owned(),
                ),
                FaceSource::Converted(_) => (
                    format!("./{}", model.directory),
                    converted_material_name(&model.stem),
                ),
                FaceSource::CommonConversion => (
                    format!("{common_directory}{}", model.directory),
                    converted_material_name(&model.stem),
                ),
            };
            let name = match kit {
                KitPlace::Listed { kit } => listed_material(&name, *kit),
                KitPlace::Alone | KitPlace::Unlisted { .. } => name,
            };
            (directory, name)
        })
        .collect();
    // Read before the loop below moves the models.
    let completions = set_completions(&models, &kits, &written, &folder.hand_split, kit_numbers);
    let mut contents = PackageFiles::new();
    let mut entries = Vec::new();
    for ((model, place), (material_directory, material_name)) in
        models.into_iter().zip(&kits).zip(&written)
    {
        let material = format!("{material_directory}{material_name}");
        let model_name = source_name(&model.file.path, &folder.path);
        // The game looks for a set's other variant's `.mtl` where the set's entry, respelled
        // for its kit number, names it.
        if let KitPlace::Unlisted { kit, listed } = place {
            let (listed_directory, listed_name) = &written[*listed];
            let expected = format!(
                "{listed_directory}{}",
                respelled_material(listed_name, *kit)
            );
            if vtree::fold_name(&expected) != vtree::fold_name(&material) {
                findings.push((
                    Code::KitVariantMtlDiffers,
                    Disposition::Keep,
                    vec![
                        ("model", model_name.clone()),
                        ("mtl", material.clone()),
                        ("expected", expected),
                    ],
                ));
            }
        }
        // An own model is listed and packed at its path below its source folder, a link's
        // model at its path below the team's Common output.
        let model_directory = if model.in_common {
            format!("{common_directory}{}", model.directory)
        } else {
            format!("./{}", model.directory)
        };
        // A set's other variants are listed by its lowest variant's entries.
        let listed = !matches!(place, KitPlace::Unlisted { .. });
        let xml_type = version_type(ctx.version, &model.xml_type).to_owned();
        if listed && xml_type != model.xml_type {
            findings.push((
                Code::XmlUniformPes15,
                Disposition::Keep,
                vec![("file", relative(&model.file.path, model.source_path))],
            ));
        }
        let entry = XmlEntry {
            xml_type,
            path: xml_path(&model_directory, &place.entry_name(&model.packed)),
            material,
            ratio: ratio(&model.stem).map(str::to_owned),
        };
        if model.in_common {
            entries.push(entry);
            continue;
        }
        // A member's `.mtl` is packed below, with the face's other `.mtl` files; a converted
        // model's material set is packed here, its texture paths pointed as a member's are.
        let packed_path = format!("{}{}", model.directory, model.packed);
        let converted_material_path = format!(
            "{}{}",
            model.directory,
            converted_material_name(&model.stem)
        );
        let (bytes, member_material) = match model.source {
            FaceSource::Member { material } => {
                let source = take(files, model.file);
                let mtl = files.get(&material.path).expect(
                    "the face task's files include every `.mtl` a member's model is paired \
                     with, a Common one too (`TaskKind::files`)",
                );
                let bytes = model_for_pre_fox(
                    &model_name,
                    source,
                    mtl,
                    ctx,
                    findings,
                    ConvertedMaterials::Converted,
                )?;
                (bytes, Some(material))
            }
            FaceSource::Converted(PreFoxConversion {
                model: converted,
                mut materials,
            }) => {
                // Before the pointing, which respells the environment map's path as the
                // folder spells its own `env` texture when it holds one.
                add_environment_map(&mut materials, &home.of(""));
                point_materials(&mut materials, &places.of(&model.file.path));
                point_reserved_kit_stems(&mut materials, &common_directory);
                insert(
                    &mut contents,
                    ModelPackage::Face,
                    converted_material_path.clone(),
                    materials.write(),
                )?;
                (converted, None)
            }
            FaceSource::CommonConversion => {
                unreachable!("a converted Common model is a link's, its entry pushed above")
            }
        };
        if !folder.hand_split.contains(&model.file.path) {
            if listed {
                entries.push(entry);
            }
            insert(&mut contents, ModelPackage::Face, packed_path, bytes)?;
            continue;
        }
        // A split model's `.mtl` is read in place: a member's is taken below, a converted
        // model's was packed above.
        let mtl = match member_material {
            Some(material) => files.get(&material.path).expect(
                "the face task's files include every `.mtl` a member's model is paired with, a \
                 Common one too (`TaskKind::files`)",
            ),
            None => contents
                .get(&converted_material_path)
                .expect("a converted model's material set is packed above"),
        };
        let split = split_face_model(&model_name, &bytes, mtl, ctx, findings)?;
        let gloves = [
            ("glove_l", "gloveL", split.glove_l),
            ("glove_r", "gloveR", split.glove_r),
        ];
        // A model that was all hand leaves no body: no entry, nothing packed under its name,
        // and its gloves listed where its entry would be.
        if let Some(body) = split.body {
            insert(&mut contents, ModelPackage::Face, packed_path, body)?;
            if listed {
                entries.push(entry.clone());
            }
        }
        for (hand, hand_type, part) in gloves {
            let Some(part) = part else {
                continue;
            };
            let packed = packed_model_name(&format!("{}_{hand}", model.stem));
            if listed {
                entries.push(XmlEntry {
                    xml_type: hand_type.to_owned(),
                    path: xml_path(&model_directory, &place.entry_name(&packed)),
                    material: entry.material.clone(),
                    ratio: entry.ratio.clone(),
                });
            }
            insert(
                &mut contents,
                ModelPackage::Face,
                format!("{}{packed}", model.directory),
                part,
            )?;
        }
    }
    // Each at its path below its source folder.
    for (file, below) in materials {
        let bytes = rewritten_materials(file, &take(files, file), &places.of(&file.path))?;
        insert(&mut contents, ModelPackage::Face, below, bytes)?;
    }
    // After every file is packed, a split model's hands and its `.mtl` included.
    for completion in completions {
        for kit in completion.missing {
            for name in &completion.names {
                pack_variant_copy(&mut contents, name, kit);
            }
            findings.push(model_variant_missing(
                &completion.reference,
                kit,
                &completion.copied,
            ));
        }
    }
    // After every model of his, before the `face_neck` dummy: with an xml, through
    // `appended`, after the xml's children.
    if folder.refkit_body {
        let body = refkit_body(folder, &entries, xml.as_ref());
        entries.extend(body);
    }
    if let Some(xml) = xml {
        let face = XmlFace::new(folder, kit_numbers, &common_directory, &places, contents);
        return user_xml_face(face, &xml, &entries, ctx, files, findings, &dif);
    }
    if !entries.iter().any(|entry| entry.xml_type == FACE_NECK) {
        // The blank face's dummy is the compiler's own placeholder, which tells the member
        // nothing; a face with models lacks the type its author may have meant to give one.
        if !entries.is_empty() {
            findings.push((Code::XmlFaceNeckAdded, Disposition::Keep, Vec::new()));
        }
        entries.push(packed_dummy(&mut contents, ctx)?);
    }
    insert(
        &mut contents,
        ModelPackage::Face,
        "face.xml".to_owned(),
        face_xml(&entries, &dif),
    )?;
    Ok(contents)
}

/// The `face_neck` entry of the bundled dummy, a face's placeholder model, after packing it into
/// `contents` as `oral_dummy_win32.model` with its `dummy.mtl`.
fn packed_dummy(
    contents: &mut PackageFiles,
    ctx: &CompileContext,
) -> Result<XmlEntry, TaskFailure> {
    let packed = packed_model_name("dummy");
    let entry = XmlEntry {
        xml_type: FACE_NECK.to_owned(),
        path: xml_path("./", &packed),
        material: "./dummy.mtl".to_owned(),
        ratio: None,
    };
    insert(
        contents,
        ModelPackage::Face,
        packed,
        ctx.templates.dummy_model().to_vec(),
    )?;
    insert(
        contents,
        ModelPackage::Face,
        "dummy.mtl".to_owned(),
        ctx.templates.dummy_mtl().to_vec(),
    )?;
    Ok(entry)
}

/// The directory a `face.xml` names the referee template's body in: its `refkit/` folder in
/// the referees' Common output, `model/character/uniform/common/999/refkit/`, where the refs
/// CPK carries the template tree's files (a refs export's `Common/refkit/` laid over them).
fn refkit_directory() -> String {
    let common = paths::common_texture_directory(Engine::PreFox, paths::REFEREE_TEAM_ID);
    format!("{common}refkit/")
}

/// The referee template's body entries (`REFKIT_BODY`) the face of `folder`, a referee folder
/// holding `fpc_off` (`ModelFolder::refkit_body`), lists after the models it lists already:
/// `entries`, its generated entries, and the `<model>` children of `xml`, his own or his
/// linked shared face's. What he has of his own is read from those, the refkit's own
/// directory left out: boots, a listed model named for the boots (`boots.model`) or his
/// boots link; gloves, a listed entry typed as a glove or a hand (a hand split's included)
/// or his gloves link (`refkit_body_entries`).
fn refkit_body(
    folder: &ModelFolder,
    entries: &[XmlEntry],
    xml: Option<&UserFaceXml>,
) -> Vec<XmlEntry> {
    let directory = refkit_directory();
    let xml_models: Vec<&ModelElement> = xml
        .into_iter()
        .flat_map(|xml| &xml.children)
        .filter_map(|child| match child {
            Child::Model(model) => Some(model),
            Child::Dif(_) | Child::Other(_) => None,
        })
        .collect();
    // The (type, path) of each listed model: an xml element lacking one names nothing by it.
    let listed: Vec<(Option<&str>, Option<&str>)> = entries
        .iter()
        .map(|entry| (Some(entry.xml_type.as_str()), Some(entry.path.as_str())))
        .chain(
            xml_models
                .iter()
                .map(|model| (model.attribute("type"), model.attribute("path"))),
        )
        .collect();
    let named: Vec<String> = listed
        .iter()
        .filter_map(|(_, path)| path.map(listed_model_name))
        .collect();
    // The refkit's entries he lists by hand are the template's body, not his own boots or
    // gloves: one refkit hand listed by hand leaves the other to be appended.
    let fold_directory = vtree::fold_name(&directory);
    let own: Vec<(Option<&str>, String)> = listed
        .iter()
        .filter_map(|(xml_type, path)| Some((*xml_type, (*path)?)))
        .filter(|(_, path)| !vtree::fold_name(path).starts_with(&fold_directory))
        .map(|(xml_type, path)| (xml_type, listed_model_name(path)))
        .collect();
    let combines = |package: ModelPackage| {
        folder
            .combined
            .iter()
            .any(|combined| combined.package == package)
    };
    let own_boots = combines(ModelPackage::Boots)
        || own
            .iter()
            .any(|(_, name)| suffix(file_stem(name)) == Some(ModelSuffix::Boots));
    let own_gloves = combines(ModelPackage::Gloves)
        || own
            .iter()
            .any(|(xml_type, _)| xml_type.is_some_and(|xml_type| HAND_TYPES.contains(&xml_type)));
    refkit_body_entries(&named, own_boots, own_gloves)
}

/// The packed name, folded, of the model a `face.xml` `path` names: its last segment, `*`
/// read as `win32` (`./oral_face_high_*.model` names `oral_face_high_win32.model`).
fn listed_model_name(path: &str) -> String {
    let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
    vtree::fold_name(&name.replace('*', "win32"))
}

/// The referee template's body as a referee's `face.xml` lists it (`blue_port.md` "The
/// referee body"): an entry per model of `REFKIT_BODY` in its order, naming the model in the
/// referees' Common output's `refkit/` with the `refkit.mtl` beside it, which the limbs take
/// rather than his own skin's `.mtl`. Left out, so he is never dressed twice: a model whose
/// packed name, folded, is among `named`, the face's listed models; the boots when
/// `own_boots`; the two hands when `own_gloves`.
fn refkit_body_entries(named: &[String], own_boots: bool, own_gloves: bool) -> Vec<XmlEntry> {
    let directory = refkit_directory();
    let material = format!("{directory}refkit.mtl");
    REFKIT_BODY
        .iter()
        .filter(|(name, xml_type)| {
            let packed = packed_model_name(name);
            // His own boots stand on his feet, and his gloves on his hands; one model the
            // face lists already is listed once.
            let dressed = (own_boots && *xml_type == "boots")
                || (own_gloves && HAND_TYPES.contains(xml_type));
            !dressed && !named.contains(&vtree::fold_name(&packed))
        })
        .map(|(name, xml_type)| XmlEntry {
            xml_type: (*xml_type).to_owned(),
            path: xml_path(&directory, &packed_model_name(name)),
            material: material.clone(),
            ratio: None,
        })
        .collect()
}

/// A member's own `face.xml` being written back (`user_xml_face`): the files its references
/// may name, where it points what the face does not pack, and the face's files packed so far.
struct XmlFace<'a> {
    /// The files a reference may name: the player's own, his linked shared face's and the
    /// export's `Common/` models, `.mtl` files and textures (`ModelFolder::common_files`; a
    /// Common `path` reference resolves nothing here, `written_path`).
    named: FaceFiles<'a>,
    /// The export's kit numbers, against which a `kitN` reference's set is completed.
    kits: &'a [u8],
    /// The team's Common output, where the game loads a Common file from.
    common_directory: &'a str,
    /// Where a packed `.mtl`'s texture paths are pointed (`point_materials`), as for a
    /// generated face.
    places: &'a FolderPlaces<'a>,
    /// The face's files packed so far, by their paths in the face.
    contents: PackageFiles,
    /// Their paths, folded: a reference naming a file already packed packs nothing more.
    packed: BTreeSet<String>,
}

impl<'a> XmlFace<'a> {
    /// The face of `folder` about to be written from its xml, `contents` packed already (the
    /// player's own models the xml does not name, with his `.mtl` files): a reference naming
    /// one of their names packs nothing more. A `kitN` reference's set is completed against
    /// `kits`, a Common file is named in `common_directory` and a `.mtl` points its texture
    /// paths at `places`.
    fn new(
        folder: &'a ModelFolder,
        kits: &'a [u8],
        common_directory: &'a str,
        places: &'a FolderPlaces<'a>,
        contents: PackageFiles,
    ) -> Self {
        let packed = contents.keys().map(|name| vtree::fold_name(name)).collect();
        XmlFace {
            named: face_files(folder),
            kits,
            common_directory,
            places,
            contents,
            packed,
        }
    }

    /// The attributes `model` is written with for `version`, in its order, the files its
    /// references name packed from `files`: `type` as `version_type` writes it, a rewrite
    /// noted in `findings` as `xml_uniform_pes15` naming the entry by its `path` as the member
    /// wrote it; `path` and `material` as `written_path` and `written_material` write them;
    /// any other attribute as it is.
    fn written_model(
        &mut self,
        model: &ModelElement,
        version: PesVersion,
        files: &mut TaskFiles,
        findings: &mut Vec<Finding>,
    ) -> Result<Vec<(String, String)>, TaskFailure> {
        let mut written = Vec::new();
        for (name, value) in &model.attributes {
            let value = match name.as_str() {
                "type" => {
                    let xml_type = version_type(version, value);
                    if xml_type != value {
                        let path = model
                            .attribute("path")
                            .map(|path| ("path", path.to_owned()));
                        findings.push((
                            Code::XmlUniformPes15,
                            Disposition::Keep,
                            path.into_iter().collect(),
                        ));
                    }
                    xml_type.to_owned()
                }
                "path" => self.written_path(value, files, findings)?,
                "material" => self.written_material(value, files, findings)?,
                _ => value.clone(),
            };
            written.push((name.clone(), value));
        }
        Ok(written)
    }

    /// The `path` value `value` as written: a `./` reference as it is, its file packed
    /// (`written_local`, noting in `findings`); a Common one naming the model as the team's
    /// Common output packs it (`oral_<stem>_*.model`, below its subfolder's path for a
    /// subfolder's file, the team ID in place of the 3-character segment), packing nothing; any
    /// other form as it is.
    fn written_path(
        &mut self,
        value: &str,
        files: &mut TaskFiles,
        findings: &mut Vec<Finding>,
    ) -> Result<String, TaskFailure> {
        match reference(value) {
            Reference::Local(name) => self.written_local(
                value,
                &name,
                FileKind::Model(ModelFormat::PesModel),
                files,
                findings,
            ),
            // The Common output packs every `Common/` model under its packed name, so the
            // member's spelling would name a file it does not hold (`prefox_common`); a
            // subfolder's, below the subfolder's own path there, kept as written.
            Reference::Common { file_name, .. } => {
                let (directory, name) = match file_name.rsplit_once('/') {
                    Some((directory, name)) => (format!("{directory}/"), name),
                    None => (String::new(), file_name.as_str()),
                };
                Ok(xml_path(
                    self.common_directory,
                    &format!("{directory}{}", packed_model_name(file_stem(name))),
                ))
            }
            Reference::Unchecked(_) => Ok(value.to_owned()),
        }
    }

    /// The `material` value `value` as written: a `./` reference as it is, its `.mtl` packed,
    /// or naming the Common one a `.mtl.common` link stands for (`written_local`, noting in
    /// `findings`); a Common one naming the `.mtl` in the team's Common output under its name,
    /// packing nothing; any other form as it is.
    fn written_material(
        &mut self,
        value: &str,
        files: &mut TaskFiles,
        findings: &mut Vec<Finding>,
    ) -> Result<String, TaskFailure> {
        match reference(value) {
            Reference::Local(name) => {
                self.written_local(value, &name, FileKind::Mtl, files, findings)
            }
            Reference::Common { file_name, .. } => {
                Ok(format!("{}{file_name}", self.common_directory))
            }
            Reference::Unchecked(_) => Ok(value.to_owned()),
        }
    }

    /// The `./` reference `value`, naming the file at `path` (relative to the xml's folder) of
    /// `kind`, as written, the file packed from `files` at `path` as the reference spells it,
    /// directory included: `value` as it is. A `kitN` name packs every variant of its set in
    /// the directory it names among the player's own files and his linked shared face's
    /// (`FaceFiles::own_local`, `FaceFiles::linked_local`), each at its own path, for the game
    /// to pick by kit, and completes the set (`complete_set`, noting in `findings`). A `.mtl`
    /// that is a `Common/` one, which a `.mtl.common` link stands for, is not packed: it is
    /// named in the team's Common output, where the Common models task packs it. A path
    /// naming no file is an error: the deep pass dropped such a folder (`xml_model_not_found`).
    fn written_local(
        &mut self,
        value: &str,
        path: &str,
        kind: FileKind,
        files: &mut TaskFiles,
        findings: &mut Vec<Finding>,
    ) -> Result<String, TaskFailure> {
        if let Some((KitToken::Reference, _)) = kit_token(file_stem(texture_lookup::split(path).1))
        {
            let variants: Vec<(String, &FileDescriptor)> = self
                .named
                .own_local(kind)
                .into_iter()
                .chain(self.named.linked_local(kind))
                // A `.mtl.common` link's Common file is the Common output's, never packed here.
                .filter(|(below, file)| {
                    !is_common_file(&file.path) && variant_of(path, below).is_some()
                })
                .collect();
            if variants.is_empty() {
                return Err(anyhow::anyhow!("{value} names no file of the face").into());
            }
            for (below, file) in &variants {
                self.pack(file, below, files)?;
            }
            self.complete_set(path, kind, &variants, findings);
            return Ok(value.to_owned());
        }
        let file = resolve(&Reference::Local(path.to_owned()), &self.named, kind)
            .ok_or_else(|| anyhow::anyhow!("{value} names no file of the face"))?;
        if is_common_file(&file.path) {
            return Ok(format!(
                "{}{}",
                self.common_directory,
                below_common(&file.path)
            ));
        }
        self.pack(file, path, files)?;
        Ok(value.to_owned())
    }

    /// Completes the set `variants`, the packed files of `kind` (each by its path in the face)
    /// the `kitN` path `referenced` names, against the export's kit numbers: for each one none
    /// of them is a variant of, ascending, the lowest variant is packed again under that
    /// number's spelling in its directory (`pack_variant_copy`), and for a `.model` set the copy
    /// is noted in `findings` as `kit_variant_missing`. A second reference to the set finds the
    /// copy packed and notes nothing more, nor does the `.mtl` set an entry names beside its
    /// model set.
    fn complete_set(
        &mut self,
        referenced: &str,
        kind: FileKind,
        variants: &[(String, &FileDescriptor)],
        findings: &mut Vec<Finding>,
    ) {
        let numbered: Vec<(u8, &str)> = variants
            .iter()
            .filter_map(|(below, _)| Some((variant_of(referenced, below)?, below.as_str())))
            .collect();
        // Of two variants of one number, the first, the player's own before his linked
        // face's, as `pack` packs it.
        let Some(&(_, lowest)) = numbered.iter().min_by_key(|(kit, _)| *kit) else {
            return;
        };
        for &kit in self.kits {
            if numbered.iter().any(|(held, _)| *held == kit) {
                continue;
            }
            if !pack_variant_copy(&mut self.contents, lowest, kit) {
                continue;
            }
            self.packed
                .insert(vtree::fold_name(&variant_name(lowest, kit)));
            if kind == FileKind::Model(ModelFormat::PesModel) {
                findings.push(model_variant_missing(
                    file_stem(referenced),
                    kit,
                    file_stem(lowest),
                ));
            }
        }
    }

    /// Packs `file`, read from `files`, at `name`, its path in the face, a `.mtl` with its
    /// texture paths pointed at `places` (`rewritten_materials`), unless a file of that path,
    /// case-folded, is packed already: two references to one file pack it once, under the
    /// first one's spelling.
    fn pack(
        &mut self,
        file: &FileDescriptor,
        name: &str,
        files: &mut TaskFiles,
    ) -> Result<(), TaskFailure> {
        if !self.packed.insert(vtree::fold_name(name)) {
            return Ok(());
        }
        let mut bytes = take(files, file);
        if file.kind == FileKind::Mtl {
            bytes = rewritten_materials(file, &bytes, &self.places.of(&file.path))?;
        }
        insert(
            &mut self.contents,
            ModelPackage::Face,
            name.to_owned(),
            bytes,
        )
    }
}

/// The files a `face.xml` of `folder`'s face may name: the player's own, his linked shared
/// face's, which a `./` reference looks in after his own, and the export's `Common/` models,
/// `.mtl` files and textures (`ModelFolder::common_files`). The referee template tree's files
/// (`FaceFiles::template_common`) are left out: the deep pass admitted a reference naming one,
/// and the face writes such a reference by its path, resolving nothing against the tree.
fn face_files(folder: &ModelFolder) -> FaceFiles<'_> {
    let linked_face = folder
        .combined
        .iter()
        .find(|combined| combined.package == ModelPackage::Face)
        .map(|combined| &combined.folder);
    FaceFiles {
        own: &folder.files,
        linked_face,
        common: &folder.common_files,
        template_common: &[],
        folder: &folder.path,
        // As `ModelFolder::roles` tells a shared folder: by where its textures go.
        shared: matches!(folder.textures, TextureHome::SharedOutput { .. }),
    }
}

/// Whether the face's `face.xml` `xml` names `model`, one of the player's own models beside a
/// linked shared face's xml, so that the xml's entry packs it and no generated entry is
/// appended for it: a `path` reference resolving to its `.model` among `named`, his own files
/// first (`resolve`), or a `kitN` one naming a set it is a variant of, whose entry packs
/// every variant of his (`XmlFace::written_local`); for a `.common` link, a Common reference
/// resolving to the Common `.model` the link loads. An FMDL he holds is never named: a
/// reference names a `.model`.
fn names_model(xml: &UserFaceXml, model: &FaceModel, named: &FaceFiles) -> bool {
    let model_kind = FileKind::Model(ModelFormat::PesModel);
    let target = if model.in_common {
        common_link_target(&model.file.path, model.source_path)
            .and_then(|linked| selected_common_model(named.common, &linked, Engine::PreFox))
    } else {
        Some(model.file)
    };
    let Some(target) = target.filter(|target| target.kind == model_kind) else {
        return false;
    };
    xml.children
        .iter()
        .filter_map(|child| match child {
            Child::Model(entry) => entry.attribute("path").map(reference),
            Child::Dif(_) | Child::Other(_) => None,
        })
        .any(|path| {
            let set_variant = match &path {
                Reference::Local(referenced) => {
                    !model.in_common
                        && role_position(named.folder, target, named.shared)
                        && variant_of(referenced, &relative(&target.path, named.folder)).is_some()
                }
                Reference::Common { .. } | Reference::Unchecked(_) => false,
            };
            set_variant
                || resolve(&path, named, model_kind).is_some_and(|file| file.path == target.path)
        })
}

/// The last `<dif>` of the `face.xml` `xml`, decoded, when it has one: the one written.
fn last_dif(xml: &UserFaceXml) -> Option<&[u8]> {
    xml.children.iter().rev().find_map(|child| match child {
        Child::Dif(bytes) => Some(bytes.as_slice()),
        Child::Model(_) | Child::Other(_) => None,
    })
}

/// The files of a pre-Fox face with a `face.xml`, `xml` (the member's own or his linked shared
/// face's), read from `files` (`messages.md` "User-supplied `face.xml`", "How a `path` or
/// `material` is resolved and written" and "What is emitted from such a folder"): the xml
/// written back (`user_face_xml`) with its children in their order, each `<model>` as
/// `face.written_model` writes it and packing the files it names, any other element as the
/// member wrote it; then `appended`, the generated entries of the player's own models the
/// shared xml does not name, which `face` packed already; then, when no written `<model>` is a
/// `face_neck`, the dummy's entry (`packed_dummy`), noted in `findings` as
/// `xml_face_neck_added` when the face has a `<model>`; then `dif`, the face's diff. Only what
/// the references name is packed beside what `face` holds: no other `.model` of the xml's
/// folder (the deep pass reported each as `xml_model_unlisted`) and no other `.mtl` of it.
fn user_xml_face(
    mut face: XmlFace,
    xml: &UserFaceXml,
    appended: &[XmlEntry],
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
    dif: &[u8],
) -> Result<PackageFiles, TaskFailure> {
    let mut children = Vec::new();
    for child in &xml.children {
        match child {
            Child::Model(model) => {
                let attributes = face.written_model(model, ctx.version, files, findings)?;
                children.push(WrittenChild::Model(attributes));
            }
            // The face's diff, the last `<dif>` or another, is written last (`dif`).
            Child::Dif(_) => {}
            Child::Other(element) => children.push(WrittenChild::Other(element)),
        }
    }
    children.extend(
        appended
            .iter()
            .map(|entry| WrittenChild::Model(entry.attributes())),
    );
    let has_model = children
        .iter()
        .any(|child| matches!(child, WrittenChild::Model(_)));
    let has_face_neck = children.iter().any(|child| match child {
        WrittenChild::Model(attributes) => attributes
            .iter()
            .any(|(name, value)| name == "type" && value == FACE_NECK),
        WrittenChild::Other(_) => false,
    });
    if !has_face_neck {
        // An xml with no `<model>` is the member's blank face: the dummy tells him nothing.
        if has_model {
            findings.push((Code::XmlFaceNeckAdded, Disposition::Keep, Vec::new()));
        }
        let dummy = packed_dummy(&mut face.contents, ctx)?;
        children.push(WrittenChild::Model(dummy.attributes()));
    }
    let written = user_face_xml(&children, dif);
    insert(
        &mut face.contents,
        ModelPackage::Face,
        "face.xml".to_owned(),
        written,
    )?;
    Ok(face.contents)
}

/// The material the entry of a per-kit set's lowest variant, of kit number `kit`, gives for
/// the `.mtl` named `name`: its kit token spelled `kitN` when it is the variant's own
/// (`pants_kit1.mtl` gives `pants_kitN.mtl`), else `name` as it is (`pants.mtl`).
fn listed_material(name: &str, kit: u8) -> String {
    let stem = file_stem(name);
    match kit_token(stem) {
        Some((KitToken::Variant(own), reference)) if own == kit => {
            format!("{reference}{}", &name[stem.len()..])
        }
        Some((KitToken::Variant(_) | KitToken::Reference, _)) | None => name.to_owned(),
    }
}

/// The `.mtl` name the game looks for when kit `kit` is picked, given `listed`, the material
/// a per-kit set's entry gives: its `kitN` respelled for `kit` (`pants_kitN.mtl` gives
/// `pants_kit2.mtl`), else `listed` itself (`pants.mtl`).
fn respelled_material(listed: &str, kit: u8) -> String {
    let stem = file_stem(listed);
    if let Some((KitToken::Reference, _)) = kit_token(stem)
        && let Some(variant) = variant_stem(stem, kit)
    {
        return format!("{variant}{}", &listed[stem.len()..]);
    }
    listed.to_owned()
}

/// Where the `.mtl` files of a model folder's pre-Fox output, a player's face or a boots or
/// gloves output (`prefox_shared`), point their texture paths (`point_materials`): a texture
/// of the package's sources nearest first from the `.mtl`'s own folder (`texture_lookup`), in the
/// folder's texture home, then a stem a texture link stands for, in the team's Common output;
/// a path below the `.mtl`'s folder at that path alone. A stem the package's sources hold is
/// the player's own before one a link stands for: a combined shared face's texture of a link's
/// stem wins, as on Fox.
pub(super) struct FolderPlaces<'a> {
    /// The model folder: the player folder, or a shared boots or gloves folder.
    pub(super) folder: &'a ScopePath,
    /// The textures of the package's sources, the model folder's own and its combined
    /// folders'.
    pub(super) textures: TextureFolders,
    /// The stems the folder's texture links stand for, folded, each with the stem of the
    /// `Common/` texture the link names: the name its DDS has in the team's Common output.
    pub(super) linked: TexturePlace,
    /// The directory a `.mtl` names the texture home by.
    pub(super) home: &'a TextureDirectory,
    /// The directory a `.mtl` names the team's Common output by.
    pub(super) common_directory: &'a TextureDirectory,
}

impl FolderPlaces<'_> {
    /// Where the paths of the `.mtl` at `path` (a converted model's material set: its
    /// model's) are pointed: a name in the places it resolves in, in order; a path below its
    /// folder at that path.
    pub(super) fn of(&self, path: &ScopePath) -> MaterialPlaces<'_> {
        let mut by_name = self.textures.places(self.folder, path, self.home);
        by_name.push((&self.linked, self.common_directory));
        MaterialPlaces {
            by_name,
            below: Some((self, path.clone())),
        }
    }
}

/// Where `point_materials` points the texture paths of one `.mtl`.
pub(super) struct MaterialPlaces<'a> {
    /// The places a texture name resolves in, in lookup order, each with the directory a `.mtl`
    /// names its textures by.
    pub(super) by_name: Vec<(&'a TexturePlace, &'a TextureDirectory)>,
    /// For a path below the `.mtl`'s folder (`texture_lookup::path_below`), which resolves at
    /// that path alone (`TextureFolders::at`): the model folder's places and the `.mtl`'s path
    /// (a converted set's model's). `None` for a `Common/` `.mtl`, whose paths, one below its
    /// folder included, are looked up by name among its directory's textures
    /// (`prefox_common::common_models`).
    pub(super) below: Option<(&'a FolderPlaces<'a>, ScopePath)>,
}

/// The stem of the `Common/` texture the `.common` texture link `file` of the player `folder`
/// names, as `Common/` spells it: the name its DDS has in the team's Common output.
pub(super) fn linked_texture_stem(folder: &ModelFolder, file: &FileDescriptor) -> String {
    let linked_name = common_link_target(&file.path, &folder.path)
        .expect("a CommonTexture role implies a `.common` link below its folder");
    // On pre-Fox no installed texture satisfies a link (validation's
    // `installed_common_textures` is empty there).
    let target = common_file(&folder.common_files, &linked_name)
        .expect("validation drops a player folder whose texture link names no Common file");
    file_stem(target.path.name()).to_owned()
}

/// `bytes`, the `.mtl` `file`, with its texture paths pointed as `point_materials` points
/// them. A material set that does not read is an error naming the file.
pub(super) fn rewritten_materials(
    file: &FileDescriptor,
    bytes: &[u8],
    places: &MaterialPlaces,
) -> Result<Vec<u8>, TaskFailure> {
    Ok(read_materials(file, bytes, places)?.write())
}

/// `bytes`, the `.mtl` `file`, read, its texture paths pointed as `point_materials` points
/// them. A material set that does not read is an error naming the file.
pub(super) fn read_materials(
    file: &FileDescriptor,
    bytes: &[u8],
    places: &MaterialPlaces,
) -> Result<MaterialSet, TaskFailure> {
    let mut set = MaterialSet::read(bytes)
        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
    point_materials(&mut set, places);
    Ok(set)
}

/// Points every texture path of `set` whose file stem (case-folded) is one of a place's
/// textures at that texture's DDS where it is packed: `places.by_name` are (textures,
/// directory) pairs in lookup order, each texture by its folded stem with its path below the
/// directory as spelled (`texture_lookup::TexturePlace`), and the first place holding a stem
/// wins, the path becoming the directory with the texture's own subdirectory below it
/// (`jessie/`, `TextureDirectory::of`) and `<stem>.dds`. A path whose stem no place holds but
/// that is a kit reference (`pants_kitN`) is pointed at the directory of the first place
/// holding a variant of its set (`texture_lookup::variant`), the variant's subdirectory
/// included, its file name kept as it is: the game respells it for the kit picked. A path
/// below the `.mtl`'s folder (`texture_lookup::path_below`), when `places.below` is set, is
/// looked up the same way in the place that path names alone, in the folder's texture home.
/// Any other path is left as it is.
pub(super) fn point_materials(set: &mut MaterialSet, places: &MaterialPlaces) {
    rewrite_texture_paths(set, |path| {
        let below = places.below.as_ref().and_then(|(folder, file)| {
            let subdirectory = texture_lookup::path_below(&path.directory)?;
            Some((
                *folder,
                folder.textures.at(folder.folder, file, subdirectory),
            ))
        });
        let lookup: Vec<(&TexturePlace, &TextureDirectory)> = match below {
            // Resolves at that path alone, and is left as written when nothing sits there.
            Some((folder, place)) => place
                .map(|place| (place, folder.home))
                .into_iter()
                .collect(),
            None => places.by_name.clone(),
        };
        let stem = file_stem(&path.file_name);
        let key = vtree::fold_name(stem);
        let found = lookup
            .iter()
            .find_map(|(textures, directory)| Some((textures.get(&key)?, *directory)));
        if let Some((below, directory)) = found {
            let (subdirectory, name) = texture_lookup::split(below);
            path.directory = directory.of(subdirectory);
            path.file_name = format!("{name}.dds");
            return;
        }
        let variant_place = lookup.iter().find_map(|(textures, directory)| {
            Some((texture_lookup::variant(textures, stem)?, *directory))
        });
        if let Some((below, directory)) = variant_place {
            let (subdirectory, _) = texture_lookup::split(below);
            path.directory = directory.of(subdirectory);
        }
    });
}

/// Points every texture path of `set`, a converted model's material set, whose file stem is a
/// reserved kit stem (`dummy_kit`, `dummy_kit_<role>`, case-folded) at `common_directory`, the
/// team's Common texture directory, as `<stem>.dds`, the stem as spelled: the modded exes
/// substitute the active kit's texture for that stem there, and the Fox directory a converted
/// FMDL names means nothing to PES 15-17. A member's own `.mtl` already names the pre-Fox
/// place and is emitted as written, so only a converted set goes through this.
pub(super) fn point_reserved_kit_stems(set: &mut MaterialSet, common_directory: &str) {
    rewrite_texture_paths(set, |path| {
        let stem = file_stem(&path.file_name);
        let key = vtree::fold_name(stem);
        if key == "dummy_kit" || key.starts_with("dummy_kit_") {
            common_directory.clone_into(&mut path.directory);
            path.file_name = format!("{stem}.dds");
        }
    });
}

/// Gives every `Basic_CNSR` material of `set`, a converted model's material set, that has no
/// `EnvironmentMap` sampler one naming `env.dds` in `home`, its model folder's texture home (a
/// player's common folder, or `./` beside a shared boots or gloves output's models), with the
/// `environment` role's sampler settings (`model_format.md`, the role table): the `R` of the
/// shader is a reflection of that cubemap, which a material without one does not have.
/// A Fox metal material converts to `Basic_CNSR` naming no environment texture, Fox having no
/// such sampler. The sampler goes after the material's other samplers (a converted Fox
/// material's base, normal and specular maps, which the converter writes before an
/// environment sampler) and before its states and vectors. A material that has one is left as
/// it is.
pub(super) fn add_environment_map(set: &mut MaterialSet, home: &str) {
    for material in &mut set.materials {
        if material.shader != "Basic_CNSR" {
            continue;
        }
        let has_environment = material.entries.iter().any(|entry| {
            matches!(entry, MaterialEntry::Sampler(sampler) if sampler.name == "EnvironmentMap")
        });
        if has_environment {
            continue;
        }
        // The settings of `model_convert`'s `sampler_for_role(TextureRole::Environment)`, which
        // the crate does not export.
        let sampler = Sampler {
            name: "EnvironmentMap".to_owned(),
            path: format!("{home}{ENVIRONMENT_MAP_STEM}.dds"),
            srgb: Some(false),
            minfilter: Some(Filter::Anisotropic),
            maxfilter: None,
            magfilter: Some(Filter::Linear),
            mipfilter: None,
            uaddr: Some(Address::Wrap),
            vaddr: Some(Address::Wrap),
            waddr: Some(Address::Wrap),
            maxaniso: Some(2),
        };
        let after_samplers = material
            .entries
            .iter()
            .rposition(|entry| matches!(entry, MaterialEntry::Sampler(_)))
            .map_or(0, |last| last + 1);
        material
            .entries
            .insert(after_samplers, MaterialEntry::Sampler(sampler));
    }
}

/// Adds `bytes` to `contents`, the files of `package`, as `name`; a name already there, in any
/// letter case (`vtree::fold_name`, as the file system and planning's duplicate check fold), is
/// an error naming `name`, since the package holds one file of a name and dropping either
/// would lose a model or its materials without a word.
pub(super) fn insert(
    contents: &mut PackageFiles,
    package: ModelPackage,
    name: String,
    bytes: Vec<u8>,
) -> Result<(), TaskFailure> {
    let key = vtree::fold_name(&name);
    if contents
        .keys()
        .any(|packed| vtree::fold_name(packed) == key)
    {
        return Err(
            anyhow::anyhow!("two files of the {} are packed as {name}", package.name()).into(),
        );
    }
    contents.insert(name, bytes);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::sync::LazyLock;

    use pes_model::ops::paths::texture_paths;

    use super::*;

    /// The card-head template's material set, its one texture path `./texture.dds` renamed to
    /// `./<file_name>`.
    fn card_set_naming(file_name: &str) -> MaterialSet {
        let bytes = fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../libs/pes_model/tests/fixtures/cardhead_materials.mtl"),
        )
        .unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("./texture.dds"), "{text}");
        MaterialSet::read(
            text.replace("./texture.dds", &format!("./{file_name}"))
                .as_bytes(),
        )
        .unwrap()
    }

    /// The texture paths of `set`, each its directory then its file name.
    fn paths(set: &MaterialSet) -> Vec<String> {
        texture_paths(set)
            .into_iter()
            .map(|path| format!("{}{}", path.directory, path.file_name))
            .collect()
    }

    /// A material set of two materials as a converted metal one and a converted shaded one
    /// are written: `metal`, `Basic_CNSR` with its base map, a state and the two vectors, with
    /// `environment` as its last sampler when given; `cloth`, `Basic_C` with its base map.
    fn metal_and_cloth(environment: Option<&str>) -> MaterialSet {
        let environment = environment.map_or(String::new(), |path| {
            format!("<sampler name=\"EnvironmentMap\" path=\"{path}\" srgb=\"0\" />")
        });
        let text = format!(
            "<materialset><material name=\"metal\" shader=\"Basic_CNSR\">\
             <sampler name=\"DiffuseMap\" path=\"./metal.dds\" srgb=\"1\" />{environment}\
             <state name=\"ztest\" value=\"1\" />\
             <vector name=\"Reflection\" x=\"1\" y=\"1\" z=\"1\" w=\"0\" />\
             <vector name=\"Shininess\" x=\"0.9\" y=\"0\" z=\"0\" w=\"1\" /></material>\
             <material name=\"cloth\" shader=\"Basic_C\">\
             <sampler name=\"DiffuseMap\" path=\"./cloth.dds\" srgb=\"1\" /></material>\
             </materialset>"
        );
        MaterialSet::read(text.as_bytes()).unwrap()
    }

    /// Each material of `set` by name, with its entries' kinds and names in order.
    fn layout(set: &MaterialSet) -> Vec<(String, Vec<String>)> {
        set.materials
            .iter()
            .map(|material| {
                let entries = material
                    .entries
                    .iter()
                    .map(|entry| match entry {
                        MaterialEntry::Sampler(sampler) => format!("sampler {}", sampler.name),
                        MaterialEntry::State(state) => format!("state {}", state.name),
                        MaterialEntry::Vector(vector) => format!("vector {}", vector.name),
                    })
                    .collect();
                (material.name.clone(), entries)
            })
            .collect()
    }

    #[test]
    fn a_basic_cnsr_material_with_no_environment_sampler_gets_one_after_its_samplers() {
        let mut set = metal_and_cloth(None);

        add_environment_map(&mut set, "home/");

        let owned = |names: &[&str]| names.iter().map(|name| (*name).to_owned()).collect();
        assert_eq!(
            layout(&set),
            [
                (
                    "metal".to_owned(),
                    owned(&[
                        "sampler DiffuseMap",
                        "sampler EnvironmentMap",
                        "state ztest",
                        "vector Reflection",
                        "vector Shininess",
                    ])
                ),
                ("cloth".to_owned(), owned(&["sampler DiffuseMap"])),
            ]
        );
        let MaterialEntry::Sampler(added) = &set.materials[0].entries[1] else {
            panic!("the second entry is a sampler");
        };
        assert_eq!(
            *added,
            Sampler {
                name: "EnvironmentMap".to_owned(),
                path: "home/env.dds".to_owned(),
                srgb: Some(false),
                minfilter: Some(Filter::Anisotropic),
                maxfilter: None,
                magfilter: Some(Filter::Linear),
                mipfilter: None,
                uaddr: Some(Address::Wrap),
                vaddr: Some(Address::Wrap),
                waddr: Some(Address::Wrap),
                maxaniso: Some(2),
            }
        );
    }

    #[test]
    fn a_basic_cnsr_material_with_an_environment_sampler_is_left_as_it_is() {
        let mut set = metal_and_cloth(Some("./own_env.dds"));
        let before = set.clone();

        add_environment_map(&mut set, "home/");

        assert_eq!(set, before);
    }

    #[test]
    fn the_environment_map_is_respelled_as_the_folder_spells_its_own_env_texture() {
        let textures = BTreeMap::from([("env".to_owned(), "Env".to_owned())]);
        let linked = BTreeMap::new();
        let places = by_name(&[(&textures, &HOME), (&linked, &COMMON)]);
        let mut set = metal_and_cloth(None);

        add_environment_map(&mut set, "home/");
        point_materials(&mut set, &places);

        assert_eq!(paths(&set), ["./metal.dds", "home/Env.dds", "./cloth.dds"]);
    }

    #[test]
    fn a_kit_reference_with_a_variant_in_a_place_is_pointed_there_its_name_kept() {
        let textures = BTreeMap::from([("pants_kit1".to_owned(), "Pants_kit1".to_owned())]);
        let linked = BTreeMap::new();
        let places = by_name(&[(&textures, &HOME), (&linked, &COMMON)]);

        let mut set = card_set_naming("pants_kitN.dds");
        point_materials(&mut set, &places);
        assert_eq!(paths(&set), ["home/pants_kitN.dds"]);

        // No variant of `other_kitN` anywhere: left as it is.
        let mut set = card_set_naming("other_kitN.dds");
        point_materials(&mut set, &places);
        assert_eq!(paths(&set), ["./other_kitN.dds"]);
    }

    #[test]
    fn a_converted_material_set_is_pointed_as_a_member_s_mtl_its_fox_texture_name_included() {
        // An FMDL names its textures as `.ftex` (or `.dds`) under a Fox directory; the stem
        // alone decides where a path is pointed.
        let set = card_set_naming("Skin.ftex");
        let textures = BTreeMap::from([("skin".to_owned(), "Skin".to_owned())]);
        let linked = BTreeMap::new();
        let places = by_name(&[(&textures, &HOME), (&linked, &COMMON)]);
        let path = ScopePath::new("Players/05 - A/face_high.mtl").unwrap();
        let file = FileDescriptor {
            size: 1,
            kind: FileKind::Mtl,
            source: path.clone(),
            path,
        };

        let Ok(member) = rewritten_materials(&file, &set.write(), &places) else {
            panic!("the member's `.mtl` reads");
        };
        let mut converted = set;
        point_materials(&mut converted, &places);

        assert_eq!(converted.write(), member);
        assert_eq!(paths(&converted), ["home/Skin.dds"]);
    }

    #[test]
    fn a_converted_model_s_reserved_kit_stem_is_pointed_at_the_team_s_common_directory() {
        let fox_directory = "/Assets/pes16/model/character/common/000/sourceimages/";
        let common = "model/character/uniform/common/792/";
        // The card's `.mtl` with its one texture path under the Fox directory an FMDL names.
        let fox_set = |file_name: &str| {
            let mut set = card_set_naming(file_name);
            rewrite_texture_paths(&mut set, |path| {
                fox_directory.clone_into(&mut path.directory)
            });
            set
        };
        for (file_name, pointed) in [
            ("dummy_kit.dds", "dummy_kit.dds"),
            ("Dummy_Kit_SRM.ftex", "Dummy_Kit_SRM.dds"),
        ] {
            let mut set = fox_set(file_name);
            point_reserved_kit_stems(&mut set, common);
            assert_eq!(paths(&set), [format!("{common}{pointed}")]);
        }
        // The game's other dummies, and a stem that only starts alike, are not reserved.
        for file_name in ["dummy_bsm.dds", "dummy_kitten.dds"] {
            let mut set = fox_set(file_name);
            point_reserved_kit_stems(&mut set, common);
            assert_eq!(paths(&set), [format!("{fox_directory}{file_name}")]);
        }
        // A member's own `.mtl` keeps a reserved stem's path as written.
        let textures = BTreeMap::new();
        let common_home = TextureDirectory::plain(common.to_owned());
        let places = by_name(&[(&textures, &HOME), (&textures, &common_home)]);
        let path = ScopePath::new("Players/05 - A/face_high.mtl").unwrap();
        let file = FileDescriptor {
            size: 1,
            kind: FileKind::Mtl,
            source: path.clone(),
            path,
        };
        let Ok(member) = read_materials(&file, &fox_set("dummy_kit.dds").write(), &places) else {
            panic!("the member's `.mtl` reads");
        };
        assert_eq!(paths(&member), [format!("{fox_directory}dummy_kit.dds")]);
    }

    /// The directory `home/` a test's texture home is named by.
    static HOME: LazyLock<TextureDirectory> =
        LazyLock::new(|| TextureDirectory::plain("home/".to_owned()));
    /// The directory `common/` a test's Common output is named by.
    static COMMON: LazyLock<TextureDirectory> =
        LazyLock::new(|| TextureDirectory::plain("common/".to_owned()));

    /// The places of `places` for a texture name, with no path below the `.mtl`'s folder
    /// looked up at its path.
    fn by_name<'a>(places: &[(&'a TexturePlace, &'a TextureDirectory)]) -> MaterialPlaces<'a> {
        MaterialPlaces {
            by_name: places.to_vec(),
            below: None,
        }
    }

    /// The places of a face of the player folder `folder` whose own textures are `below`,
    /// each its path below the folder, its texture home `home/` and no texture link.
    fn face_places<'a>(folder: &'a ScopePath, below: &[&str]) -> FolderPlaces<'a> {
        let mut textures = TextureFolders::default();
        for below in below {
            textures.insert(true, below);
        }
        FolderPlaces {
            folder,
            textures,
            linked: TexturePlace::new(),
            home: &HOME,
            common_directory: &COMMON,
        }
    }

    #[test]
    fn a_mtl_points_a_texture_name_at_the_nearest_folder_holding_it() {
        let folder = ScopePath::new("Players/05 - A").unwrap();
        let places = face_places(&folder, &["skin", "jessie/skin", "jessie/pants_kit2"]);
        let pointed = |mtl: &str, file_name: &str| {
            let mut set = card_set_naming(file_name);
            point_materials(&mut set, &places.of(&ScopePath::new(mtl).unwrap()));
            paths(&set)
        };
        let deep = "Players/05 - A/jessie/body/x.mtl";
        assert_eq!(pointed(deep, "skin.dds"), ["home/jessie/skin.dds"]);
        assert_eq!(
            pointed(deep, "pants_kitN.dds"),
            ["home/jessie/pants_kitN.dds"]
        );
        let root = "Players/05 - A/face_high.mtl";
        assert_eq!(pointed(root, "skin.dds"), ["home/skin.dds"]);
        // The root's lookup never goes down into `jessie/`.
        assert_eq!(pointed(root, "pants_kitN.dds"), ["./pants_kitN.dds"]);
    }

    #[test]
    fn a_mtl_path_below_its_folder_resolves_at_that_path_alone() {
        let folder = ScopePath::new("Players/05 - A").unwrap();
        let places = face_places(
            &folder,
            &["y", "jessie/body/shorts/Y", "jessie/body/shorts/pants_kit1"],
        );
        let pointed = |mtl: &str, file_name: &str| {
            let mut set = card_set_naming(file_name);
            point_materials(&mut set, &places.of(&ScopePath::new(mtl).unwrap()));
            paths(&set)
        };
        let body = "Players/05 - A/jessie/body/x.mtl";
        assert_eq!(
            pointed(body, "shorts/y.dds"),
            ["home/jessie/body/shorts/Y.dds"]
        );
        assert_eq!(
            pointed(body, "shorts/pants_kitN.dds"),
            ["home/jessie/body/shorts/pants_kitN.dds"]
        );
        // From the root, `shorts/` holds nothing: left as written, the root's `y` unused.
        let root = "Players/05 - A/face_high.mtl";
        assert_eq!(pointed(root, "shorts/y.dds"), ["./shorts/y.dds"]);
        // A `Common/` `.mtl`'s places look every path up by name.
        let common = TexturePlace::from([("y".to_owned(), "Y".to_owned())]);
        let mut set = card_set_naming("shorts/y.dds");
        point_materials(&mut set, &by_name(&[(&common, &COMMON)]));
        assert_eq!(paths(&set), ["common/Y.dds"]);
    }

    #[test]
    fn a_listed_material_is_respelled_only_when_it_carries_the_variant_s_own_token() {
        assert_eq!(listed_material("pants_kit1.mtl", 1), "pants_kitN.mtl");
        // A shared `.mtl`, and one the search's "any `.mtl`" fallback found under another
        // number's token (`a_kit2.mtl` for `pants_kit1.model`): the game must look for the
        // name as it is.
        assert_eq!(listed_material("pants.mtl", 1), "pants.mtl");
        assert_eq!(listed_material("a_kit2.mtl", 1), "a_kit2.mtl");
        assert_eq!(respelled_material("pants_kitN.mtl", 2), "pants_kit2.mtl");
        assert_eq!(respelled_material("a_kit2.mtl", 1), "a_kit2.mtl");
    }

    #[test]
    fn a_mtl_two_entries_name_under_two_spellings_is_packed_once_under_the_first() {
        let folder = ScopePath::new("Players/05 - A").unwrap();
        let path = ScopePath::new("Players/05 - A/face.mtl").unwrap();
        let own = [FileDescriptor {
            size: 1,
            kind: FileKind::Mtl,
            source: path.clone(),
            path: path.clone(),
        }];
        let places = face_places(&folder, &["Skin"]);
        let mut face = XmlFace {
            named: FaceFiles {
                own: &own,
                linked_face: None,
                common: &[],
                template_common: &[],
                folder: &folder,
                shared: false,
            },
            kits: &[],
            common_directory: "common/",
            places: &places,
            contents: PackageFiles::new(),
            packed: BTreeSet::new(),
        };
        let mut files = TaskFiles::from([(path, card_set_naming("skin.dds").write())]);

        for value in ["./Face.mtl", "./face.mtl"] {
            let Ok(written) = face.written_material(value, &mut files, &mut Vec::new()) else {
                panic!("{value} is not written");
            };
            assert_eq!(written, value);
        }

        let names: Vec<&str> = face.contents.keys().map(String::as_str).collect();
        assert_eq!(names, ["Face.mtl"]);
        let set = MaterialSet::read(&face.contents["Face.mtl"]).unwrap();
        assert_eq!(paths(&set), ["home/Skin.dds"]);
    }

    #[test]
    fn a_kit_model_reference_packs_the_set_s_models_and_no_mtl_variant_beside_them() {
        let folder = ScopePath::new("Players/05 - A").unwrap();
        let descriptor = |name: &str, kind: FileKind| {
            let path = ScopePath::new(&format!("Players/05 - A/{name}")).unwrap();
            FileDescriptor {
                size: 1,
                kind,
                source: path.clone(),
                path,
            }
        };
        let model = FileKind::Model(ModelFormat::PesModel);
        // A `.mtl` variant sits beside the models: a `.model` reference packs the models only.
        let own = [
            descriptor("pants_kit1.model", model),
            descriptor("pants_kit1.mtl", FileKind::Mtl),
            descriptor("pants_kit2.model", model),
        ];
        let places = face_places(&folder, &[]);
        let mut face = XmlFace {
            named: FaceFiles {
                own: &own,
                linked_face: None,
                common: &[],
                template_common: &[],
                folder: &folder,
                shared: false,
            },
            kits: &[],
            common_directory: "common/",
            places: &places,
            contents: PackageFiles::new(),
            packed: BTreeSet::new(),
        };
        let mut files = TaskFiles::from([
            (own[0].path.clone(), b"kit1".to_vec()),
            (own[1].path.clone(), card_set_naming("skin.dds").write()),
            (own[2].path.clone(), b"kit2".to_vec()),
        ]);

        let Ok(written) = face.written_path("./pants_kitN.model", &mut files, &mut Vec::new())
        else {
            panic!("the kit reference is not written");
        };

        assert_eq!(written, "./pants_kitN.model");
        let names: Vec<&str> = face.contents.keys().map(String::as_str).collect();
        assert_eq!(names, ["pants_kit1.model", "pants_kit2.model"]);
    }

    #[test]
    fn a_name_the_package_holds_in_another_letter_case_is_two_files_of_one_name() {
        let mut contents = PackageFiles::new();
        assert!(
            insert(
                &mut contents,
                ModelPackage::Face,
                "face_high.mtl".to_owned(),
                Vec::new(),
            )
            .is_ok()
        );
        let failure = insert(
            &mut contents,
            ModelPackage::Face,
            "Face_High.mtl".to_owned(),
            Vec::new(),
        )
        .unwrap_err();
        assert_eq!(failure.code, Code::FolderPackFailed);
        assert_eq!(
            failure.context,
            [(
                "error",
                format!(
                    "two files of the {} are packed as Face_High.mtl",
                    ModelPackage::Face.name()
                )
            )]
        );
        let names: Vec<&str> = contents.keys().map(String::as_str).collect();
        assert_eq!(names, ["face_high.mtl"]);
    }
}
