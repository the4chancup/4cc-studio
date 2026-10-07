//! A player's pre-Fox face (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps",
//! steps 4, 6 and 7): his folder's `.model` files, and those of the shared face folder he
//! links, packed under their `oral_<stem>_win32.model` names, the `.mtl` files with their
//! texture paths pointed at his common folder (a texture link's at the team's Common output),
//! and the generated `face.xml` typing every model, his `.common` links to a Common model
//! included, his face diff as its `<dif>`. A face with no `face_neck` model, the blank face of
//! a folder with no model included, gets the bundled dummy as one. `materialize` packs the
//! files into the face CPK.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use aesthetics_export::{FileDescriptor, common_link_name};
use pes_model::format::mtl::MaterialSet;
use pes_model::ops::paths::rewrite_texture_paths;
use pes_version::Engine;
use studio_core::Disposition;
use vtree::ScopePath;

use super::materialize::PackageFiles;
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::deep::relative;
use crate::face_diff;
use crate::face_xml::{XmlEntry, face_xml, packed_model_name, ratio, version_type, xml_path};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{
    ModelPackage, PlayerFile, common_file, file_stem, is_direct_common_file,
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
    /// Its source folder's export path.
    source_path: &'a ScopePath,
    /// Its source folder's files.
    source_files: &'a [FileDescriptor],
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
/// as a copy would (`player_folders.md` "A link plus local models combines"). When no entry is
/// a `face_neck`, the dummy is listed last and packed as `oral_dummy_win32.model` and
/// `dummy.mtl`, noted in `findings` as `xml_face_neck_added` when the face has a model. An
/// entry's type is written for `ctx.version` (`version_type`): each entry whose type that
/// rewrites (`uniform` to `uniform_sub` on PES 2015) is noted in `findings` as
/// `xml_uniform_pes15`, naming its model below its source folder. Two files of one source
/// packing under one name fail the task: neither can be dropped silently.
pub(super) fn face(
    folder: &ModelFolder,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<PackageFiles, TaskFailure> {
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
        let mut packed_here = Vec::new();
        for (file, role) in source_roles {
            match role {
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
                        source_path,
                        source_files,
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
                        source_path,
                        source_files,
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
                    let linked_name = common_link_name(file.path.name())
                        .expect("a CommonTexture role implies a `.common` link name");
                    // On pre-Fox no installed texture satisfies a link (validation's
                    // `installed_common_textures` is empty there).
                    let target = common_file(&folder.common_files, &linked_name).expect(
                        "validation drops a player folder whose texture link names no Common \
                         file",
                    );
                    let target_stem = file_stem(target.path.name()).to_owned();
                    linked.insert(vtree::fold_name(&stem), target_stem);
                }
                // The search resolves a material link where it finds it (`mtl_for`).
                PlayerFile::CommonMaterial => {}
                // The Fox roles; and a face file with no face model, which is not read.
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::UnusedFaceFile
                | PlayerFile::LeftOutKitVariant => {}
            }
        }
        earlier.extend(packed_here);
    }
    // By export path, case-folded, then as spelled, so a recompile lists them alike.
    models.sort_by_cached_key(|model| {
        (
            model.file.path.fold_key(),
            model.file.path.as_str().to_owned(),
        )
    });

    // The team's Common output, which a `face.xml` and a `.mtl` name a Common file in.
    let common_directory = paths::common_texture_directory(Engine::PreFox, team_id);
    let mut contents = PackageFiles::new();
    let mut entries = Vec::new();
    for model in models {
        let material = mtl_for(
            &model.file.path,
            model.source_path,
            model.source_files,
            &folder.common_files,
        )
        .expect(
            "the deep pass drops a folder holding a `.model`, or a link to one, no `.mtl` is \
             found for (`model_material_undefined`)",
        );
        // A `.mtl` the search found in `Common/` is the Common output's, never packed here.
        let material_directory = if is_direct_common_file(&material.path) {
            common_directory.as_str()
        } else {
            "./"
        };
        let model_directory = if model.in_common {
            common_directory.as_str()
        } else {
            "./"
        };
        let xml_type = version_type(ctx.version, &model.xml_type).to_owned();
        if xml_type != model.xml_type {
            findings.push((
                Code::XmlUniformPes15,
                Disposition::Keep,
                vec![("file", relative(&model.file.path, model.source_path))],
            ));
        }
        entries.push(XmlEntry {
            xml_type,
            path: xml_path(model_directory, &model.packed),
            material: format!("{material_directory}{}", material.path.name()),
            ratio: ratio(&model.stem).map(str::to_owned),
        });
        if !model.in_common {
            insert(
                &mut contents,
                ModelPackage::Face,
                model.packed,
                take(files, model.file),
            )?;
        }
    }
    let home = folder.textures.directory(Engine::PreFox, team_id);
    // A stem the folder holds is the player's own, before one a texture link stands for: a
    // combined shared face's texture of a link's stem wins, as on Fox.
    let places = [
        (&textures, home.as_str()),
        (&linked, common_directory.as_str()),
    ];
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
        let packed = packed_model_name("dummy");
        entries.push(XmlEntry {
            xml_type: FACE_NECK.to_owned(),
            path: xml_path("./", &packed),
            material: "./dummy.mtl".to_owned(),
            ratio: None,
        });
        insert(
            &mut contents,
            ModelPackage::Face,
            packed,
            ctx.templates.dummy_model().to_vec(),
        )?;
        insert(
            &mut contents,
            ModelPackage::Face,
            "dummy.mtl".to_owned(),
            ctx.templates.dummy_mtl().to_vec(),
        )?;
    }
    let dif = dif.unwrap_or_else(|| ctx.templates.face_diff().to_vec());
    insert(
        &mut contents,
        ModelPackage::Face,
        "face.xml".to_owned(),
        face_xml(&entries, &dif),
    )?;
    Ok(contents)
}

/// `bytes`, the `.mtl` `file`, with every texture path whose file stem (case-folded) is one of
/// a place's textures pointed at that place's directory as that texture's DDS, `<stem>.dds`:
/// `places` are (textures, directory) pairs, each texture by its folded stem with its stem as
/// spelled where it is packed, and the first place holding a stem wins. Any other path is left
/// as it is. A material set that does not read is an error naming the file.
pub(super) fn rewritten_materials(
    file: &FileDescriptor,
    bytes: &[u8],
    places: &[(&BTreeMap<String, String>, &str)],
) -> Result<Vec<u8>, TaskFailure> {
    let mut set = MaterialSet::read(bytes)
        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
    rewrite_texture_paths(&mut set, |path| {
        let key = vtree::fold_name(file_stem(&path.file_name));
        let found = places
            .iter()
            .find_map(|(textures, directory)| Some((textures.get(&key)?, *directory)));
        if let Some((stem, directory)) = found {
            directory.clone_into(&mut path.directory);
            path.file_name = format!("{stem}.dds");
        }
    });
    Ok(set.write())
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
