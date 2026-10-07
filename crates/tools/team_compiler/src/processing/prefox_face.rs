//! A player's pre-Fox face (`team_compiler/pipeline.md` "3. Per-model-folder parallel steps",
//! steps 4, 6 and 7): his folder's `.model` files packed under their `oral_<stem>_win32.model`
//! names, his `.mtl` files with their texture paths pointed at his common folder, and the
//! generated `face.xml` typing every model, his face diff as its `<dif>`. A face with no
//! `face_neck` model, the blank face of a folder with no model included, gets the bundled dummy
//! as one. `materialize` packs the files into the face CPK.

use std::collections::BTreeMap;

use pes_model::format::mtl::MaterialSet;
use pes_model::ops::paths::rewrite_texture_paths;
use pes_version::Engine;
use studio_core::Disposition;

use super::materialize::PackageFiles;
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::face_diff;
use crate::face_xml::{XmlEntry, face_xml, packed_model_name, ratio, xml_path};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::plan::ModelFolder;
use crate::plan::subset::{PlayerFile, file_stem};

/// The `face.xml` type the game loads a player's face model as. A face whose models include
/// none of it gets the dummy as its `face_neck`.
const FACE_NECK: &str = "face_neck";

/// The files of `folder`'s pre-Fox face, compiled from its files' bytes in `files` for team
/// `team_id`, by their names in the face CPK: each `.model` under its packed name
/// (`packed_model_name`) and an entry of the `face.xml`, in the order of the models' export
/// paths, case-folded; each `.mtl` under its own name, every texture path naming one of the
/// folder's textures by its stem pointed at the folder's texture home as that texture's DDS;
/// and the `face.xml`, its `<dif>` the folder's face diff (`face_diff.bin`, else
/// `face_diff.xml` decoded, else the bundled one). When no entry is a `face_neck`, the dummy is
/// listed last and packed as `oral_dummy_win32.model` and `dummy.mtl`, noted in `findings` as
/// `xml_face_neck_added` when the face has a model. Two files packing under one name fail the
/// task: neither can be dropped silently.
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
    for (_, _, source_files) in folder.roles() {
        for (file, role) in source_files {
            match role {
                PlayerFile::PreFoxModel { xml_type } => models.push((file, xml_type)),
                PlayerFile::Material => materials.push(file),
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
    }
    // By export path, case-folded, then as spelled, so a recompile lists them alike.
    models.sort_by_cached_key(|(file, _)| (file.path.fold_key(), file.path.as_str().to_owned()));

    let mut contents = PackageFiles::new();
    let mut entries = Vec::new();
    for (file, xml_type) in models {
        let stem = file_stem(file.path.name());
        let material = mtl_for(&file.path, &folder.path, &folder.files).expect(
            "the deep pass drops a folder holding a `.model` no `.mtl` is found for \
             (`model_material_undefined`)",
        );
        let packed = packed_model_name(stem);
        entries.push(XmlEntry {
            xml_type,
            path: xml_path(&packed),
            material: format!("./{}", material.path.name()),
            ratio: ratio(stem).map(str::to_owned),
        });
        insert(&mut contents, packed, take(files, file))?;
    }
    let home = folder.textures.directory(Engine::PreFox, team_id);
    for file in materials {
        let mut set = MaterialSet::read(&take(files, file))
            .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
        rewrite_texture_paths(&mut set, |path| {
            if let Some(stem) = textures.get(&vtree::fold_name(file_stem(&path.file_name))) {
                path.directory.clone_from(&home);
                path.file_name = format!("{stem}.dds");
            }
        });
        insert(&mut contents, file.path.name().to_owned(), set.write())?;
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
        insert(&mut contents, packed, ctx.templates.dummy_model().to_vec())?;
        insert(
            &mut contents,
            "dummy.mtl".to_owned(),
            ctx.templates.dummy_mtl().to_vec(),
        )?;
    }
    let dif = dif.unwrap_or_else(|| ctx.templates.face_diff().to_vec());
    insert(
        &mut contents,
        "face.xml".to_owned(),
        face_xml(&entries, &dif),
    )?;
    Ok(contents)
}

/// Adds `bytes` to `contents` as `name`; a name already there is an error naming it, since
/// the face CPK holds one file of a name and dropping either would lose a model or its
/// materials without a word.
fn insert(contents: &mut PackageFiles, name: String, bytes: Vec<u8>) -> Result<(), TaskFailure> {
    if contents.contains_key(&name) {
        return Err(anyhow::anyhow!("two files of the face are packed as {name}").into());
    }
    contents.insert(name, bytes);
    Ok(())
}
