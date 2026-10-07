//! A player's pre-Fox face (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps",
//! steps 4, 6 and 7): his folder's `.model` files, and those of the shared face folder he
//! links, packed under their `oral_<stem>_win32.model` names, the `.mtl` files with their
//! texture paths pointed at his common folder, and the generated `face.xml` typing every
//! model, his face diff as its `<dif>`. A face with no `face_neck` model, the blank face of a
//! folder with no model included, gets the bundled dummy as one. `materialize` packs the files
//! into the face CPK.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use aesthetics_export::FileDescriptor;
use pes_model::format::mtl::MaterialSet;
use pes_model::ops::paths::rewrite_texture_paths;
use pes_version::Engine;
use studio_core::Disposition;
use vtree::ScopePath;

use super::materialize::PackageFiles;
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::face_diff;
use crate::face_xml::{XmlEntry, face_xml, packed_model_name, ratio, xml_path};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem};

/// The `face.xml` type the game loads a player's face model as. A face whose models include
/// none of it gets the dummy as its `face_neck`.
const FACE_NECK: &str = "face_neck";

/// One `.model` of the face, with the source folder it was found in: its `.mtl` is searched
/// for there (`mtl_for`), a shared face's never in the player's folder.
struct FaceModel<'a> {
    /// The model file.
    file: &'a FileDescriptor,
    /// Its `face.xml` type.
    xml_type: String,
    /// The name it is packed under (`packed_model_name`).
    packed: String,
    /// Its source folder's export path.
    source_path: &'a ScopePath,
    /// Its source folder's files.
    source_files: &'a [FileDescriptor],
}

/// The files of `folder`'s pre-Fox face, compiled from its files' bytes in `files` for team
/// `team_id`, by their names in the face CPK: each `.model` under its packed name
/// (`packed_model_name`) and an entry of the `face.xml`, in the order of the models' export
/// paths, case-folded, each naming the `.mtl` its search finds in its own source folder; each
/// `.mtl` under its own name, every texture path naming one of the folder's textures by its
/// stem pointed at the folder's texture home as that texture's DDS; and the `face.xml`, its
/// `<dif>` the folder's face diff (`face_diff.bin`, else `face_diff.xml` decoded, else the
/// bundled one). A linked shared face's files are copied in under the player's own: a model
/// or `.mtl` packing under a name (case-folded) the player's folder already packs is left out,
/// the player's file replacing it as a copy would (`player_folders.md` "A link plus local
/// models combines"). When no entry is a `face_neck`, the dummy is listed last and packed as
/// `oral_dummy_win32.model` and `dummy.mtl`, noted in `findings` as `xml_face_neck_added` when
/// the face has a model. Two files of one source packing under one name fail the task:
/// neither can be dropped silently.
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
                    let packed = packed_model_name(file_stem(file.path.name()));
                    let key = vtree::fold_name(&packed);
                    if earlier.contains(&key) {
                        continue;
                    }
                    packed_here.push(key);
                    models.push(FaceModel {
                        file,
                        xml_type,
                        packed,
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
                // The Fox roles; and a face file with no face model, which is not read.
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::UnusedFaceFile
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::CommonTexture(_) => {}
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

    let mut contents = PackageFiles::new();
    let mut entries = Vec::new();
    for model in models {
        let stem = file_stem(model.file.path.name());
        let material = mtl_for(&model.file.path, model.source_path, model.source_files).expect(
            "the deep pass drops a folder holding a `.model` no `.mtl` is found for \
             (`model_material_undefined`)",
        );
        entries.push(XmlEntry {
            xml_type: model.xml_type,
            path: xml_path(&model.packed),
            material: format!("./{}", material.path.name()),
            ratio: ratio(stem).map(str::to_owned),
        });
        insert(
            &mut contents,
            ModelPackage::Face,
            model.packed,
            take(files, model.file),
        )?;
    }
    let home = folder.textures.directory(Engine::PreFox, team_id);
    for file in materials {
        let bytes = rewritten_materials(file, &take(files, file), &textures, &home)?;
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
            path: xml_path(&packed),
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
/// `textures` (folded stem → stem as the folder spells it) pointed at the directory `home` as
/// that texture's DDS, `<stem>.dds`; any other path is left as it is. A material set that does
/// not read is an error naming the file.
pub(super) fn rewritten_materials(
    file: &FileDescriptor,
    bytes: &[u8],
    textures: &BTreeMap<String, String>,
    home: &str,
) -> Result<Vec<u8>, TaskFailure> {
    let mut set = MaterialSet::read(bytes)
        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
    rewrite_texture_paths(&mut set, |path| {
        if let Some(stem) = textures.get(&vtree::fold_name(file_stem(&path.file_name))) {
            home.clone_into(&mut path.directory);
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
