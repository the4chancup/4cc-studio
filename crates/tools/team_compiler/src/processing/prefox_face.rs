//! A player's pre-Fox face (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps",
//! steps 4, 6 and 7): his folder's `.model` files, and those of the shared face folder he
//! links, packed under their `oral_<stem>_win32.model` names, the `.mtl` files with their
//! texture paths pointed at his common folder (a texture link's at the team's Common output),
//! and the generated `face.xml` typing every model, his `.common` links to a Common model
//! included, his face diff as its `<dif>`. A per-kit model set is packed whole and listed once,
//! its kit token spelled `kitN` (`team_compiler/pipeline.md` "4. Per-export non-model steps",
//! Kit-dependent assets). A face with no `face_neck` model, the blank face of a folder with no
//! model included, gets the bundled dummy as one. A folder holding the member's own `face.xml`
//! gets that xml written back instead, with only the files it names packed (`user_xml_face`).
//! `materialize` packs the files into the face CPK.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use aesthetics_export::{
    FileDescriptor, FileKind, KitToken, ModelFormat, common_link_name, kit_token, variant_stem,
};
use pes_model::format::mtl::MaterialSet;
use pes_model::ops::paths::rewrite_texture_paths;
use pes_version::{Engine, PesVersion};
use studio_core::Disposition;
use vtree::ScopePath;

use super::materialize::PackageFiles;
use super::prefox_split::split_face_model;
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::deep::relative;
use crate::face_diff;
use crate::face_xml::{
    WrittenChild, XmlEntry, face_xml, packed_model_name, ratio, user_face_xml, version_type,
    xml_path,
};
use crate::kit_variants::has_variant_among;
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{
    ModelPackage, PlayerFile, common_file, file_stem, in_folder_or_face, is_direct_common_file,
};
use crate::user_face_xml::{
    Child, FaceFiles, ModelElement, Reference, reference, resolve, variant_of,
};

/// The `face.xml` type the game loads a player's face model as. A face whose models include
/// none of it gets the dummy as its `face_neck`.
const FACE_NECK: &str = "face_neck";

/// One `.model` of the face, with the source folder it was found in: its `.mtl` is searched
/// for there (`mtl_for`), a shared face's never in the player's folder.
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
    /// Whether it is a link: the game loads the model from the team's Common output, and the
    /// face packs nothing of it.
    in_common: bool,
    /// The `.mtl` its search finds from its source folder (`mtl_for`).
    material: &'a FileDescriptor,
    /// Its source folder's export path.
    source_path: &'a ScopePath,
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
/// face holds: the models packed in the face (not a link's) whose packed names differ only in
/// their kit token's digit form a set, its lowest number listed (of two of one number, the
/// first in `models`) and its others unlisted, and a set of one is an ordinary model. Read
/// from the packed names, so a set split between a linked shared face and the player's own
/// files is the one set the face holds once the shared files are copied in under his.
fn kit_places(models: &[FaceModel]) -> Vec<KitPlace> {
    // Each set's variants, by its folded reference: (kit number, index), in `models`' order.
    let mut sets: BTreeMap<String, Vec<(u8, usize)>> = BTreeMap::new();
    for (index, model) in models.iter().enumerate() {
        if model.in_common {
            continue;
        }
        if let Some((KitToken::Variant(kit), reference)) = kit_token(file_stem(&model.packed)) {
            sets.entry(vtree::fold_name(&reference))
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

/// The files of `folder`'s pre-Fox face, compiled from its files' bytes in `files` for team
/// `team_id`, by their names in the face CPK: each `.model` under its packed name
/// (`packed_model_name`) and an entry of the `face.xml`, in the order of the models' export
/// paths, case-folded, each naming the `.mtl` its search finds from its own source folder; each
/// `.mtl` under its own name, every texture path naming one of the folder's textures by its
/// stem pointed at the folder's texture home as that texture's DDS, and one naming a stem a
/// texture link of the folder stands for at that texture in the team's Common output; and the
/// `face.xml`, its `<dif>` the folder's face diff (`face_diff.bin`, else `face_diff.xml`
/// decoded, else the bundled one). A `.common` link to a model is an entry naming the model in
/// the team's Common output, where the export's Common models task packs it; a `.mtl` the
/// search finds in `Common/`, directly or through a link, is named there too, and the face
/// packs neither (`pipeline.md` "3. Per-model-folder parallel steps", step 4). A linked shared
/// face's files are copied in under the player's own: a model or `.mtl` packing under a name
/// (case-folded) the player's folder already packs is left out, the player's file replacing it
/// as a copy would (`player_folders.md` "A link plus local models combines"). A hand-split
/// model (`ModelFolder::hand_split`) is split at the wrists (`split_face_model`): its body is
/// packed and listed in its place, and each hand made follows it as
/// `oral_<stem>_glove_l_win32.model` or `oral_<stem>_glove_r_win32.model`, an entry typed
/// `gloveL` or `gloveR` naming the model's `.mtl` and `ratio`; a split model whose `.mtl` is a
/// Common file fails the task with `model_conversion_failed`. When no entry is
/// a `face_neck`, the dummy is listed last and packed as `oral_dummy_win32.model` and
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
/// is noted in `findings` as `kit_variant_mtl_differs`. Two files of one source packing under
/// one name fail the task: neither can be dropped silently.
///
/// A folder holding the member's own `face.xml` (`ModelFolder::own_face_xml`) has no generated
/// xml: the textures and the face diff are read as above, and the xml is written back by
/// `user_xml_face`, which packs only the models and `.mtl` files its references name, under
/// the names they give (`messages.md` "User-supplied `face.xml`"). No model is typed by its
/// name, split or listed by kit there; the hand split is not planned for such a folder.
pub(super) fn face(
    folder: &ModelFolder,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<PackageFiles, TaskFailure> {
    let own_xml = folder.own_face_xml();
    let mut models = Vec::new();
    let mut materials = Vec::new();
    let mut dif = None;
    // The folder's texture stems, folded, each with its stem as the folder spells it: the
    // name its converted DDS has in the texture home (`folder_textures`).
    let mut textures: BTreeMap<String, String> = BTreeMap::new();
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
    for ((_, source_path, source_roles), source_files) in
        folder.roles().into_iter().zip(source_files)
    {
        // Each model's `.mtl` is resolved here, before any entry: a set's other variants are
        // checked against its listed one's, wherever it sorts.
        let material_of = |file: &FileDescriptor| {
            mtl_for(&file.path, source_path, source_files, &folder.common_files).expect(
                "the deep pass drops a folder holding a `.model`, or a link to one, no `.mtl` is \
                 found for (`model_material_undefined`)",
            )
        };
        let mut packed_here = Vec::new();
        for (file, role) in source_roles {
            match role {
                // With the member's own xml the xml lists the face's models (`user_xml_face`),
                // and the deep pass, which compared only the `.mtl` each entry names, does not
                // promise a search finds one for every model (an entry may name none).
                PlayerFile::PreFoxModel { .. } | PlayerFile::PreFoxCommonModel { .. }
                    if own_xml.is_some() => {}
                PlayerFile::PreFoxModel { xml_type } => {
                    let stem = file_stem(file.path.name());
                    let packed = packed_model_name(stem);
                    let key = vtree::fold_name(&packed);
                    if earlier.contains(&key) {
                        continue;
                    }
                    packed_here.push(key);
                    models.push(FaceModel {
                        file,
                        stem: stem.to_owned(),
                        xml_type,
                        packed,
                        in_common: false,
                        material: material_of(file),
                        source_path,
                    });
                }
                // Packing nothing into the face, a link takes no name from a shared face's
                // file.
                PlayerFile::PreFoxCommonModel { xml_type } => {
                    let linked_name = common_link_name(file.path.name())
                        .expect("a PreFoxCommonModel role implies a `.common` link name");
                    // Two spellings of one link (`legs.model.common` and the tolerated
                    // `legs.model.common.txt`) are one model, listed once: the first in the
                    // folder's file order.
                    if !linked_models.insert(vtree::fold_name(&linked_name)) {
                        continue;
                    }
                    let stem = file_stem(&linked_name);
                    models.push(FaceModel {
                        file,
                        stem: stem.to_owned(),
                        xml_type,
                        packed: packed_model_name(stem),
                        in_common: true,
                        material: material_of(file),
                        source_path,
                    });
                }
                PlayerFile::Material => {
                    let key = vtree::fold_name(file.path.name());
                    if earlier.contains(&key) {
                        continue;
                    }
                    packed_here.push(key);
                    materials.push(file);
                }
                // On pre-Fox the face diff is the one packed file (`fcl_hair_sim.fclo` has no
                // role there); here it is the `<dif>`, never a file of the CPK.
                PlayerFile::Packed { .. } => dif = Some(take(files, file)),
                // The deep pass has dropped a folder whose face diff fails to decode, so a
                // failure here is not a member's mistake.
                PlayerFile::FaceDiffXml => {
                    let bytes = face_diff::from_xml(&take(files, file))
                        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
                    dif = Some(bytes);
                }
                PlayerFile::Texture(stem, _) => {
                    textures.insert(vtree::fold_name(&stem), stem);
                }
                PlayerFile::CommonTexture(stem) => {
                    linked.insert(vtree::fold_name(&stem), linked_texture_stem(folder, file));
                }
                // The search resolves a material link where it finds it (`mtl_for`).
                PlayerFile::CommonMaterial => {}
                // Read after the sources, the first one (`own_xml`), by `user_xml_face`.
                PlayerFile::FaceXml => {}
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
    let home = folder.textures.directory(Engine::PreFox, team_id);
    // A stem the folder holds is the player's own, before one a texture link stands for: a
    // combined shared face's texture of a link's stem wins, as on Fox.
    let places = [
        (&textures, home.as_str()),
        (&linked, common_directory.as_str()),
    ];
    let dif = dif.unwrap_or_else(|| ctx.templates.face_diff().to_vec());
    if let Some(xml_file) = own_xml {
        let face = XmlFace::new(folder, &common_directory, &places);
        return user_xml_face(face, xml_file, ctx, files, findings, &dif);
    }
    // By export path, case-folded, then as spelled, so a recompile lists them alike.
    models.sort_by_cached_key(|model| {
        (
            model.file.path.fold_key(),
            model.file.path.as_str().to_owned(),
        )
    });

    let kits = kit_places(&models);
    // The `material` each model's entries write, as its directory and its name: `./` or, for
    // a `.mtl` the search found in `Common/`, which is the Common output's and never packed
    // here, the Common directory; the name respelled for a set's listed variant.
    let written: Vec<(&str, String)> = models
        .iter()
        .zip(&kits)
        .map(|(model, kit)| {
            let directory = if is_direct_common_file(&model.material.path) {
                common_directory.as_str()
            } else {
                "./"
            };
            let name = model.material.path.name();
            let name = match kit {
                KitPlace::Listed { kit } => listed_material(name, *kit),
                KitPlace::Alone | KitPlace::Unlisted { .. } => name.to_owned(),
            };
            (directory, name)
        })
        .collect();
    let mut contents = PackageFiles::new();
    let mut entries = Vec::new();
    for ((model, place), (material_directory, material_name)) in
        models.into_iter().zip(&kits).zip(&written)
    {
        let material = format!("{material_directory}{material_name}");
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
                        ("model", model.file.path.name().to_owned()),
                        ("mtl", material.clone()),
                        ("expected", expected),
                    ],
                ));
            }
        }
        let model_directory = if model.in_common {
            common_directory.as_str()
        } else {
            "./"
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
            path: xml_path(model_directory, &place.entry_name(&model.packed)),
            material,
            ratio: ratio(&model.stem).map(str::to_owned),
        };
        if model.in_common {
            entries.push(entry);
            continue;
        }
        let bytes = take(files, model.file);
        if !folder.hand_split.contains(&model.file.path) {
            if listed {
                entries.push(entry);
            }
            insert(&mut contents, ModelPackage::Face, model.packed, bytes)?;
            continue;
        }
        // A split model's `.mtl` is read in place: it is packed below, with the face's other
        // `.mtl` files, and taken there.
        let Some(mtl) = files.get(&model.material.path) else {
            return Err(TaskFailure {
                code: Code::ModelConversionFailed,
                context: vec![
                    ("model", model.file.path.name().to_owned()),
                    (
                        "error",
                        format!(
                            "its .mtl, {}, is a Common file, which the face does not read",
                            model.material.path.as_str()
                        ),
                    ),
                ],
            });
        };
        let split = split_face_model(model.file.path.name(), &bytes, mtl, ctx, findings)?;
        let gloves = [
            ("glove_l", "gloveL", split.glove_l),
            ("glove_r", "gloveR", split.glove_r),
        ];
        // A model that was all hand leaves no body: no entry, nothing packed under its name,
        // and its gloves listed where its entry would be.
        if let Some(body) = split.body {
            insert(&mut contents, ModelPackage::Face, model.packed, body)?;
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
                    path: xml_path("./", &place.entry_name(&packed)),
                    material: entry.material.clone(),
                    ratio: entry.ratio.clone(),
                });
            }
            insert(&mut contents, ModelPackage::Face, packed, part)?;
        }
    }
    for file in materials {
        let bytes = rewritten_materials(file, &take(files, file), &places)?;
        insert(
            &mut contents,
            ModelPackage::Face,
            file.path.name().to_owned(),
            bytes,
        )?;
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

/// A member's own `face.xml` being written back (`user_xml_face`): the files its references
/// may name, where it points what the face does not pack, and the face's files packed so far.
struct XmlFace<'a> {
    /// The files a reference may name: the player's own, his linked shared face's and the
    /// export's `Common/` `.mtl` files and textures (`ModelFolder::common_files`, which hold
    /// no `Common/` `.model`).
    named: FaceFiles<'a>,
    /// The team's Common output, where the game loads a Common file from.
    common_directory: &'a str,
    /// Where a packed `.mtl`'s texture paths are pointed (`point_materials`), as for a
    /// generated face.
    places: &'a [(&'a BTreeMap<String, String>, &'a str)],
    /// The face's files packed so far, by their names in the face.
    contents: PackageFiles,
    /// Their names, folded: a reference naming a file already packed packs nothing more.
    packed: BTreeSet<String>,
}

impl<'a> XmlFace<'a> {
    /// The face of `folder` about to be written from its own xml, nothing packed yet; a
    /// Common file is named in `common_directory` and a `.mtl` points its texture paths at
    /// `places`.
    fn new(
        folder: &'a ModelFolder,
        common_directory: &'a str,
        places: &'a [(&'a BTreeMap<String, String>, &'a str)],
    ) -> Self {
        // The shared face the player combines, which a `./` reference looks in after his own
        // files.
        let linked_face = folder
            .combined
            .iter()
            .find(|combined| combined.package == ModelPackage::Face)
            .map_or(&[][..], |combined| combined.folder.files.as_slice());
        XmlFace {
            named: FaceFiles {
                own: &folder.files,
                linked_face,
                common: &folder.common_files,
                folder: &folder.path,
            },
            common_directory,
            places,
            contents: PackageFiles::new(),
            packed: BTreeSet::new(),
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
                "path" => self.written_path(value, files)?,
                "material" => self.written_material(value, files)?,
                _ => value.clone(),
            };
            written.push((name.clone(), value));
        }
        Ok(written)
    }

    /// The `path` value `value` as written: a `./` reference as it is, its file packed
    /// (`written_local`); a Common one naming the model as the team's Common output packs it
    /// (`oral_<stem>_*.model`, the team ID in place of the 3-character segment), packing
    /// nothing; any other form as it is.
    fn written_path(&mut self, value: &str, files: &mut TaskFiles) -> Result<String, TaskFailure> {
        match reference(value) {
            Reference::Local(name) => {
                self.written_local(value, &name, FileKind::Model(ModelFormat::PesModel), files)
            }
            // The Common output packs every `Common/` model under its packed name, so the
            // member's spelling would name a file it does not hold (`prefox_common`).
            Reference::Common { file_name, .. } => Ok(xml_path(
                self.common_directory,
                &packed_model_name(file_stem(&file_name)),
            )),
            Reference::Unchecked(_) => Ok(value.to_owned()),
        }
    }

    /// The `material` value `value` as written: a `./` reference as it is, its `.mtl` packed,
    /// or naming the Common one a `.mtl.common` link stands for (`written_local`); a Common
    /// one naming the `.mtl` in the team's Common output under its name, packing nothing; any
    /// other form as it is.
    fn written_material(
        &mut self,
        value: &str,
        files: &mut TaskFiles,
    ) -> Result<String, TaskFailure> {
        match reference(value) {
            Reference::Local(name) => self.written_local(value, &name, FileKind::Mtl, files),
            Reference::Common { file_name, .. } => {
                Ok(format!("{}{file_name}", self.common_directory))
            }
            Reference::Unchecked(_) => Ok(value.to_owned()),
        }
    }

    /// The `./` reference `value`, naming the file `name` of `kind`, as written, the file
    /// packed from `files` under `name` as the reference spells it: `value` as it is. A `kitN`
    /// name packs every variant of its set among the player's own files (directly in his
    /// folder or in `face/`) and his linked shared face's, each under its own name, for the
    /// game to pick by kit. A `.mtl` that is a `Common/` one, which a `.mtl.common` link stands
    /// for, is not packed: it is named in the team's Common output, where the Common models
    /// task packs it. A name naming no file is an error: the deep pass dropped such a folder
    /// (`xml_model_not_found`).
    fn written_local(
        &mut self,
        value: &str,
        name: &str,
        kind: FileKind,
        files: &mut TaskFiles,
    ) -> Result<String, TaskFailure> {
        if let Some((KitToken::Reference, _)) = kit_token(file_stem(name)) {
            let FaceFiles {
                own,
                linked_face,
                folder,
                ..
            } = self.named;
            let variants: Vec<&FileDescriptor> = own
                .iter()
                .filter(|file| in_folder_or_face(folder, file))
                .chain(linked_face)
                .filter(|file| file.kind == kind && variant_of(name, file.path.name()).is_some())
                .collect();
            if variants.is_empty() {
                return Err(anyhow::anyhow!("{value} names no file of the face").into());
            }
            for file in variants {
                self.pack(file, file.path.name(), files)?;
            }
            return Ok(value.to_owned());
        }
        let file = resolve(&Reference::Local(name.to_owned()), &self.named, kind)
            .ok_or_else(|| anyhow::anyhow!("{value} names no file of the face"))?;
        if is_direct_common_file(&file.path) {
            return Ok(format!("{}{}", self.common_directory, file.path.name()));
        }
        self.pack(file, name, files)?;
        Ok(value.to_owned())
    }

    /// Packs `file`, read from `files`, under `name`, a `.mtl` with its texture paths pointed
    /// at `places` (`rewritten_materials`), unless a file of that name, case-folded, is packed
    /// already: two references to one file pack it once, under the first one's spelling.
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
            bytes = rewritten_materials(file, &bytes, self.places)?;
        }
        insert(
            &mut self.contents,
            ModelPackage::Face,
            name.to_owned(),
            bytes,
        )
    }
}

/// The files of a pre-Fox face whose folder holds the member's own `face.xml`, `xml_file`, read
/// from `files` (`messages.md` "User-supplied `face.xml`", "How a `path` or `material` is
/// resolved and written" and "What is emitted from such a folder"): the xml written back
/// (`user_face_xml`) with its children in their order, each `<model>` as `face.written_model`
/// writes it and packing the files it names, any other element as the member wrote it; then,
/// when no written `<model>` is a `face_neck`, the dummy's entry (`packed_dummy`), noted in
/// `findings` as `xml_face_neck_added` when the xml has a `<model>`; then its last `<dif>`,
/// else `dif`, the folder's face diff or the bundled one. Only what the references name is
/// packed: no other `.model` (the deep pass reported each as `xml_model_unlisted`) and no
/// other `.mtl`. An xml that does not parse is an error: the deep pass dropped its folder.
fn user_xml_face(
    mut face: XmlFace,
    xml_file: &FileDescriptor,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
    dif: &[u8],
) -> Result<PackageFiles, TaskFailure> {
    let xml = crate::user_face_xml::parse(&take(files, xml_file))
        .map_err(|error| anyhow::anyhow!("{}: {error}", xml_file.path.as_str()))?;
    let mut children = Vec::new();
    let mut xml_dif = None;
    let mut has_model = false;
    for child in &xml.children {
        match child {
            Child::Model(model) => {
                has_model = true;
                let attributes = face.written_model(model, ctx.version, files, findings)?;
                children.push(WrittenChild::Model(attributes));
            }
            Child::Dif(bytes) => xml_dif = Some(bytes.as_slice()),
            Child::Other(element) => children.push(WrittenChild::Other(element)),
        }
    }
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
    let written = user_face_xml(&children, xml_dif.unwrap_or(dif));
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

/// The stem of the `Common/` texture the `.common` texture link `file` of the player `folder`
/// names, as `Common/` spells it: the name its DDS has in the team's Common output.
pub(super) fn linked_texture_stem(folder: &ModelFolder, file: &FileDescriptor) -> String {
    let linked_name = common_link_name(file.path.name())
        .expect("a CommonTexture role implies a `.common` link name");
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
    places: &[(&BTreeMap<String, String>, &str)],
) -> Result<Vec<u8>, TaskFailure> {
    Ok(read_materials(file, bytes, places)?.write())
}

/// `bytes`, the `.mtl` `file`, read, its texture paths pointed as `point_materials` points
/// them. A material set that does not read is an error naming the file.
pub(super) fn read_materials(
    file: &FileDescriptor,
    bytes: &[u8],
    places: &[(&BTreeMap<String, String>, &str)],
) -> Result<MaterialSet, TaskFailure> {
    let mut set = MaterialSet::read(bytes)
        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
    point_materials(&mut set, places);
    Ok(set)
}

/// Points every texture path of `set` whose file stem (case-folded) is one of a place's
/// textures at that place's directory as that texture's DDS, `<stem>.dds`: `places` are
/// (textures, directory) pairs, each texture by its folded stem with its stem as spelled where
/// it is packed, and the first place holding a stem wins. A path whose stem no place holds but
/// that is a kit reference (`pants_kitN`) is pointed at the directory of the first place
/// holding a variant of its set (`has_variant_among`), its file name kept as it is: the game
/// respells it for the kit picked. Any other path is left as it is.
fn point_materials(set: &mut MaterialSet, places: &[(&BTreeMap<String, String>, &str)]) {
    rewrite_texture_paths(set, |path| {
        let stem = file_stem(&path.file_name);
        let key = vtree::fold_name(stem);
        let found = places
            .iter()
            .find_map(|(textures, directory)| Some((textures.get(&key)?, *directory)));
        if let Some((stem, directory)) = found {
            directory.clone_into(&mut path.directory);
            path.file_name = format!("{stem}.dds");
            return;
        }
        let variant_place = places
            .iter()
            .find(|(textures, _)| has_variant_among(stem, textures.values().map(String::as_str)));
        if let Some((_, directory)) = variant_place {
            (*directory).clone_into(&mut path.directory);
        }
    });
}

/// Adds `bytes` to `contents`, the files of `package`, as `name`; a name already there is an
/// error naming it, since the package holds one file of a name and dropping either would lose
/// a model or its materials without a word.
pub(super) fn insert(
    contents: &mut PackageFiles,
    package: ModelPackage,
    name: String,
    bytes: Vec<u8>,
) -> Result<(), TaskFailure> {
    if contents.contains_key(&name) {
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

    #[test]
    fn a_kit_reference_with_a_variant_in_a_place_is_pointed_there_its_name_kept() {
        let textures = BTreeMap::from([("pants_kit1".to_owned(), "Pants_kit1".to_owned())]);
        let linked = BTreeMap::new();
        let places = [(&textures, "home/"), (&linked, "common/")];

        let mut set = card_set_naming("pants_kitN.dds");
        point_materials(&mut set, &places);
        assert_eq!(paths(&set), ["home/pants_kitN.dds"]);

        // No variant of `other_kitN` anywhere: left as it is.
        let mut set = card_set_naming("other_kitN.dds");
        point_materials(&mut set, &places);
        assert_eq!(paths(&set), ["./other_kitN.dds"]);
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
        let textures = BTreeMap::from([("skin".to_owned(), "Skin".to_owned())]);
        let linked = BTreeMap::new();
        let places = [(&textures, "home/"), (&linked, "common/")];
        let mut face = XmlFace {
            named: FaceFiles {
                own: &own,
                linked_face: &[],
                common: &[],
                folder: &folder,
            },
            common_directory: "common/",
            places: &places,
            contents: PackageFiles::new(),
            packed: BTreeSet::new(),
        };
        let mut files = TaskFiles::from([(path, card_set_naming("skin.dds").write())]);

        for value in ["./Face.mtl", "./face.mtl"] {
            let Ok(written) = face.written_material(value, &mut files) else {
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
        let textures = BTreeMap::new();
        let linked = BTreeMap::new();
        let places = [(&textures, "home/"), (&linked, "common/")];
        let mut face = XmlFace {
            named: FaceFiles {
                own: &own,
                linked_face: &[],
                common: &[],
                folder: &folder,
            },
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

        let Ok(written) = face.written_path("./pants_kitN.model", &mut files) else {
            panic!("the kit reference is not written");
        };

        assert_eq!(written, "./pants_kitN.model");
        let names: Vec<&str> = face.contents.keys().map(String::as_str).collect();
        assert_eq!(names, ["pants_kit1.model", "pants_kit2.model"]);
    }
}
