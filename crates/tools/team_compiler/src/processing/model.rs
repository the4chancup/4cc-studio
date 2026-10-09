//! One package of a model folder's Fox models (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", steps 1, 2, 3 and 7): its `.model` sources converted to
//! FMDL, its models renamed to their allowed
//! names, each part's texture paths pointed at where its textures go and the textures its
//! meshes use looked for, the parts resolving to one name merged into one model, with
//! the files that go beside them: the package's files, which `materialize` packs or places.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use aesthetics_export::{FileKind, ModelFormat};
use fmdl::ops::merge::{MergeError, merge};
use fmdl::ops::paths::{TexturePath, rewrite_texture_paths, used_texture_paths};
use fmdl::{FmdlFile, Model};
use model_convert::formats::fmdl::{fmdl_to_ir, ir_to_fmdl};
use model_convert::ir::CanonicalModel;
use model_convert::ops::hand_split::split_by_skeleton_group;
use pes_version::Engine;
use studio_core::Disposition;
use vtree::ScopePath;

use super::conversion::{fmdl_for_fox, model_for_fox, source_name};
use super::materialize::PackageFiles;
use super::{CompileContext, Finding, TaskFailure, TaskFiles, take};
use crate::face_diff;
use crate::kit_variants::has_variant_among;
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::roles::{
    ModelPackage, PlayerFile, file_stem, is_direct_common_file, skeleton_slot,
};
use crate::user_face_xml::{Reference, reference};

/// One model of the package: a part of the output model its allowed name names, from the
/// folder's own files, a combined shared folder's, or the export's `Common/` folder through a
/// `.common` link.
struct Part {
    /// The allowed name the part resolves to (`boots`), the output model's.
    name: &'static str,
    /// The part's export path, which with its file name orders the parts of one output.
    path: ScopePath,
    /// The part's bytes, an FMDL: a `.model`'s converted, a member's FMDL as it is or moved
    /// onto the version's skeleton (`convert_part`).
    bytes: Vec<u8>,
    /// The skeleton paired with the part: the `.skl` of its stem in its own source folder, or
    /// the one its conversion writes.
    skeleton: Option<Vec<u8>>,
    /// Where the part's own textures are packed.
    textures: PartTextures,
}

/// Where a part's own textures go, which its texture paths are rewritten to name
/// (`pipeline.md` "3. Per-model-folder parallel steps", step 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PartTextures {
    /// The folder's texture home: the part is the folder's own or a combined folder's, and
    /// its textures are the folder's textures task's.
    Folder,
    /// The team's Common output: the part is a Common model a `.common` link brings in, whose
    /// textures stay in `Common/` and are the export's Common textures task's, never relocated.
    Common,
    /// The folder's texture home for a stem the folder holds, else the team's Common output
    /// for a stem `Common/` holds: the part is the folder's `.model` converted with a `Common/`
    /// `.mtl` (a `.mtl.common` link), a set that names Common's textures (`pipeline.md` "Common
    /// textures are one task of their export").
    CommonSet,
}

/// The files of `folder`'s `package`, compiled from its files' bytes in `files` for team
/// `team_id`, by their names in the package, a file the game needs beside the models that no
/// source holds taken from the run's templates. Every model is first put through
/// `convert_part`: a `.model` with no `.fmdl` of its stem is converted, with the `.mtl` its
/// search finds among its source's files, and so is a Common `.model` a `.common` link brings
/// in, with the `.mtl` planning resolved for it (`ModelFolder::common_material`); an FMDL, a
/// member's own, a shared folder's or a Common one, runs the conversion pre-check with the
/// `.skl` of its stem as its bind pose, and is moved onto the version's skeleton when it is
/// posed off it. The skeleton a conversion writes is the part's, as a member's `.skl` of its
/// stem would be (one beside it of other bytes is `skl_merge_conflict`), and is dropped with
/// no finding for a role with no skeleton slot. A hand-split face part, converted or moved
/// first, gives the face its body and the gloves its hands (`parts_of`), the gloves reading
/// the part's `.skl` for that, before any texture path is rewritten. A merge of several parts
/// into one model is noted in `findings` as `fmdl_merged`. A texture a
/// part's mesh uses that nothing supplies (`texture_supply`) fails the task with
/// `fmdl_texture_not_found` at the first one, or, when the installed CPKs cannot be looked in,
/// is noted in `findings` as `fmdl_texture_not_found`, once per texture.
pub(super) fn package(
    folder: &ModelFolder,
    package: ModelPackage,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<PackageFiles, TaskFailure> {
    let mut parts: Vec<Part> = Vec::new();
    // The stems of the folder's own textures, and of the Common textures its `.common` links
    // stand for.
    let mut texture_stems = BTreeSet::new();
    let mut linked_stems = BTreeSet::new();
    let mut contents = PackageFiles::new();
    // `roles()` yields the folder's own files, then each combined folder's in `combined`'s
    // order, so each source's roles pair with its own file list here: a `.model`'s `.mtl` is
    // searched for among its source's files.
    let source_files = iter::once(&folder.files).chain(
        folder
            .combined
            .iter()
            .map(|combined| &combined.folder.files),
    );
    // A hand-split face part (`ModelFolder::hand_split`) is read by the face task, which keeps
    // its body, and by the gloves task, which keeps its hands, each with the part's `.skl`.
    let reads_hand_split = match package {
        ModelPackage::Face | ModelPackage::Gloves => true,
        ModelPackage::Boots => false,
    };
    for ((_, source_path, source_roles), source_files) in
        folder.roles().into_iter().zip(source_files)
    {
        // A skeleton pairs with the model of its stem in the same directory: keyed by the
        // path up to the extension, case-folded as the file system folds it (planning pairs
        // a Common skeleton with its model folded too), since a player folder's reserved
        // subfolder may hold a model of the same name as one directly in the folder. They are
        // read before the models: a `.skl` may sort after its model, and an FMDL needs its
        // skeleton, its bind pose, for the conversion pre-check.
        let mut skeletons: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        for (file, role) in &source_roles {
            if let PlayerFile::Skeleton { package: owner, .. } = role
                && (*owner == package || (reads_hand_split && folder.hand_split_skeleton(file)))
            {
                skeletons.insert(
                    vtree::fold_name(file_stem(file.path.as_str())),
                    take(files, file),
                );
            }
        }
        for (file, role) in source_roles {
            let hand_split = reads_hand_split && folder.hand_split.contains(&file.path);
            let mut part = |name, textures| Part {
                name,
                path: file.path.clone(),
                bytes: take(files, file),
                skeleton: skeletons.remove(&vtree::fold_name(file_stem(file.path.as_str()))),
                textures,
            };
            match role {
                PlayerFile::Model {
                    package: owner,
                    name,
                } if owner == package || hand_split => {
                    let mut part = part(name, PartTextures::Folder);
                    let model = source_name(&file.path, &folder.path);
                    let mtl = if file.kind == FileKind::Model(ModelFormat::PesModel) {
                        let mtl = mtl_for(
                            &file.path,
                            source_path,
                            source_files,
                            &folder.common_files,
                        )
                        .expect(
                            "the deep pass drops a folder holding a selected `.model` no `.mtl` \
                             is found for (`model_material_undefined`)",
                        );
                        // A Common set names Common's textures, and the deep pass checked them
                        // against `Common/`: a stem the folder lacks is looked for there.
                        if is_direct_common_file(&mtl.path) {
                            part.textures = PartTextures::CommonSet;
                        }
                        let mtl = files.get(&mtl.path).expect(
                            "a package converting a `.model` reads its source's `.mtl` files \
                             (`TaskKind::files`)",
                        );
                        Some(mtl.as_slice())
                    } else {
                        None
                    };
                    convert_part(&mut part, &model, mtl, owner, package, ctx, findings)?;
                    parts.extend(parts_of(part, &model, hand_split, package, ctx, findings)?);
                }
                // A Common `.model` converts here, in each linking player's task, as his own
                // `.model` does: Fox has no Common model output for it to convert once into,
                // so two players linking it convert it twice, as a Common FMDL is baked into
                // each of their packages (`pipeline.md` step 3 "Format conversion").
                PlayerFile::CommonModel {
                    package: owner,
                    name,
                } if owner == package || hand_split => {
                    let mut part = part(name, PartTextures::Common);
                    let model = source_name(&file.path, &folder.path);
                    let mtl = if file.kind == FileKind::Model(ModelFormat::PesModel) {
                        let mtl = folder.common_material(&file.path).expect(
                            "planning resolves a Common `.model`'s `.mtl`, the deep pass having \
                             dropped a folder linking one with none (`model_material_undefined`)",
                        );
                        let mtl = files.get(&mtl.path).expect(
                            "a package converting a Common `.model` reads the `.mtl` planning \
                             resolved for it (`TaskKind::files`)",
                        );
                        Some(mtl.as_slice())
                    } else {
                        None
                    };
                    convert_part(&mut part, &model, mtl, owner, package, ctx, findings)?;
                    parts.extend(parts_of(part, &model, hand_split, package, ctx, findings)?);
                }
                PlayerFile::Packed {
                    package: owner,
                    name,
                } if owner == package => {
                    contents.insert(name.to_owned(), take(files, file));
                }
                // The deep pass has dropped a folder whose face diff fails to decode, or that
                // gives it in both forms, so a failure here is not a member's mistake.
                PlayerFile::FaceDiffXml if package == ModelPackage::Face => {
                    let bytes = face_diff::from_xml(&take(files, file))
                        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
                    contents.insert("face_diff.bin".to_owned(), bytes);
                }
                // The textures are the textures task's, and a linked one the Common textures
                // task's; this task only points its models at them. Stems fold, as validation
                // folds the name a texture claims.
                PlayerFile::Texture(stem, _) => {
                    texture_stems.insert(vtree::fold_name(&stem));
                }
                PlayerFile::CommonTexture(stem) => {
                    linked_stems.insert(vtree::fold_name(&stem));
                }
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::UnusedFaceFile
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::FaceDiffXml
                | PlayerFile::Packed { .. } => {}
                // Read in place with the `.model` it defines, above.
                PlayerFile::Material => {}
                // Read before the models, above.
                PlayerFile::Skeleton { .. } => {}
                // The search resolves a material link where it finds it (`mtl_for`), and
                // the task reads the Common `.mtl` it names (`TaskKind::files`).
                PlayerFile::CommonMaterial => {}
                // Pre-Fox roles: a Fox target gives no file one.
                PlayerFile::PreFoxModel { .. }
                | PlayerFile::PreFoxPart { .. }
                | PlayerFile::PreFoxCommonModel { .. }
                | PlayerFile::FaceXml
                | PlayerFile::ConversionSkeleton => {}
                // Planning drops a player folder holding one, and `drop_gltf_folders` removes a
                // shared folder whose selected model is one before any task is made, so no
                // task meets it.
                PlayerFile::UnsupportedGltf => {}
            }
        }
    }

    // The parts of one output model go in alphabetical source order, by file name folded as
    // the file system folds it and then by export path, so a recompile gives the same model.
    parts.sort_by_cached_key(|part| {
        (
            vtree::fold_name(part.path.name()),
            part.path.as_str().to_owned(),
        )
    });
    // The parsed parts, their merges and their written models, charged at the parts' source
    // size, an estimate of each form, until the package's files are built.
    let parts_size = parts.iter().map(|part| part.bytes.len()).sum();
    let _parts_charge = ctx.budget.charge(parts_size);
    let mut by_name: BTreeMap<&'static str, Vec<Part>> = BTreeMap::new();
    for part in parts {
        by_name.entry(part.name).or_default().push(part);
    }

    // A folder's textures sit in its one texture home, once however many ids the package is
    // emitted under: every copy of the model points at that one location. A texture resolved
    // in Common, a Common part's own or one a folder's link stands for, stays in the team's
    // Common output, where the export's Common textures task puts it once for every player
    // (`pipeline.md` step 6: a texture resolved in Common is never relocated). A folder part
    // looks in the folder's textures first, and a path of its into the team's pre-Fox Common
    // folder reaches `Common/`'s (`point_texture`); one converted with a `Common/` `.mtl`
    // looks in the folder's, then in `Common/`'s (`PartTextures::CommonSet`). Validation
    // refuses a player folder holding a texture and a link of one stem
    // (`texture_stem_conflict`), but not a link beside a combined shared folder's texture of
    // its stem: there the shared folder's texture wins.
    let texture_directory = folder.textures.directory(ctx.version.engine(), team_id);
    let common_directory = paths::common_texture_directory(Engine::Fox, team_id);
    let folder_places = [
        (&texture_stems, texture_directory.as_str()),
        (&linked_stems, common_directory.as_str()),
    ];
    let common_places = [(&folder.common_texture_stems, common_directory.as_str())];
    let common_set_places = [folder_places[0], folder_places[1], common_places[0]];
    // A texture pointed at the team's Common output is there when the export's Common
    // textures task packs it: a texture directly in `Common/`, or one a link stands for.
    let common_stems = [&folder.common_texture_stems, &linked_stems];
    // A texture the part's source does not hold is one of the game's own; its directory names
    // the team as `000`, which becomes the team's id.
    let team_segment = format!("/{team_id}/");
    // The skeleton the package's parts bring. Only the `fcl_hair` and the `boots` parts pair
    // one (`player_file`), so at most one name's parts have any.
    let mut skeleton = None;
    for (name, mut parts) in by_name {
        if let Some(found) = merged_skeleton(&mut parts)? {
            skeleton = Some(found);
        }
        // Each part's paths are rewritten before the merge: which textures are a part's own
        // depends on where the part came from, and the merged model no longer tells its parts
        // apart. One material two parts define over textures that now sit in different
        // directories is the merge's `merge_material_conflict`, as intended.
        let mut models = Vec::with_capacity(parts.len());
        for part in &parts {
            let places: &[(&BTreeSet<String>, &str)] = match part.textures {
                PartTextures::Folder => &folder_places,
                PartTextures::Common => &common_places,
                PartTextures::CommonSet => &common_set_places,
            };
            let mut model = FmdlFile::read(&part.bytes)?;
            rewrite_texture_paths(&mut model, |path| {
                point_texture(path, places, common_places[0], &team_segment);
            })?;
            for path in used_texture_paths(&model)? {
                let installed_holds = |stem: &str| {
                    ctx.installed
                        .holds(&paths::common_texture(Engine::Fox, team_id, stem))
                };
                let context = || {
                    vec![
                        ("model", source_name(&part.path, &folder.path)),
                        ("texture", format!("{}{}", path.directory, path.file_name)),
                    ]
                };
                match texture_supply(&path, &common_stems, &common_directory, installed_holds) {
                    TextureSupply::Supplied => {}
                    TextureSupply::Missing => {
                        return Err(TaskFailure {
                            code: Code::FmdlTextureNotFound,
                            context: context(),
                        });
                    }
                    // Once per texture: two entries of the table may name one path.
                    TextureSupply::Unknown => {
                        let finding = (Code::FmdlTextureNotFound, Disposition::Keep, context());
                        if !findings.contains(&finding) {
                            findings.push(finding);
                        }
                    }
                }
            }
            models.push(model);
        }
        let bytes = match models.as_slice() {
            [model] => model.write(),
            _ => {
                let merged = merge_parts(&models)?;
                findings.push((
                    Code::FmdlMerged,
                    Disposition::Keep,
                    vec![("model", format!("{name}.fmdl"))],
                ));
                merged.write()
            }
        };
        contents.insert(format!("{name}.fmdl"), bytes);
    }
    // The game loads the boots and the hair with a skeleton beside them, under the slot's
    // name (`player_folders.md` "SKL pairing"): the parts' own when they bring one, else the
    // standard full-body one. A face also needs its `face_diff.bin`, and a hair its
    // `fcl_hair_sim.fclo`: a source's own when one holds it, else the run's template
    // ("What the injected files are").
    match package {
        ModelPackage::Boots => {
            contents.insert(
                "boots.skl".to_owned(),
                skeleton.unwrap_or_else(|| ctx.templates.body_skeleton().to_vec()),
            );
        }
        ModelPackage::Face => {
            if !contents.contains_key("face_diff.bin") {
                contents.insert(
                    "face_diff.bin".to_owned(),
                    ctx.templates.face_diff().to_vec(),
                );
            }
            if contents.contains_key("fcl_hair.fmdl") {
                if !contents.contains_key("fcl_hair_sim.fclo") {
                    contents.insert(
                        "fcl_hair_sim.fclo".to_owned(),
                        ctx.templates.fcl_hair_sim().to_vec(),
                    );
                }
                contents.insert(
                    "fcl_hair_sim.skl".to_owned(),
                    skeleton.unwrap_or_else(|| ctx.templates.body_skeleton().to_vec()),
                );
            }
        }
        // The gloves have no skeleton slot, and no `.skl` pairs with a glove.
        ModelPackage::Gloves => {}
    }

    Ok(contents)
}

/// The skeleton the parts of one output model share, taken out of them: the one `.skl` every
/// part brings (byte-identical files under several names are one skeleton), or `None` when no
/// part brings one. Parts merged into one model must reference one skeleton
/// (`player_folders.md` "Merge constraint"), so any other mix, a part with a skeleton beside
/// one without included, is `skl_merge_conflict`.
fn merged_skeleton(parts: &mut [Part]) -> Result<Option<Vec<u8>>, TaskFailure> {
    let mut skeletons = parts.iter_mut().map(|part| part.skeleton.take());
    let Some(first) = skeletons.next() else {
        return Ok(None);
    };
    if skeletons.any(|skeleton| skeleton != first) {
        return Err(skeleton_conflict());
    }
    Ok(first)
}

/// `skl_merge_conflict` for one output model whose parts bring different skeletons.
fn skeleton_conflict() -> TaskFailure {
    TaskFailure {
        code: Code::SklMergeConflict,
        context: vec![("skeleton", "differs".to_owned())],
    }
}

/// Converts `part`, a model that the task's findings name `model`, for the Fox target: a
/// `.model`, with `mtl`, the material set its search found, becomes an FMDL
/// (`conversion::model_for_fox`); an FMDL (`mtl` `None`) is packed as it is unless the
/// pre-check finds it posed off the version's skeleton, when it is moved onto it
/// (`conversion::fmdl_for_fox`). The skeleton the conversion writes takes the path a member's
/// `.skl` of the part's stem would: beside one, they are two skeletons of the part, one when
/// they are equal, and an FMDL moved off the pose its `.skl` describes drops that `.skl` for
/// the conversion's (`pipeline.md` step 3 "Format conversion"). A part of `owner` under a name
/// with no skeleton slot keeps none, with no finding: the member authored no file
/// (`messages.md` `skl_no_slot`). What the conversion reports goes to `findings` only when the
/// task builds `owner`, its `package`: the gloves task converts a hand-split face part again,
/// for its hands, and that is the face's to tell. A conversion that fails, or whose FMDL the
/// game cannot load, fails the task.
fn convert_part(
    part: &mut Part,
    model: &str,
    mtl: Option<&[u8]>,
    owner: ModelPackage,
    package: ModelPackage,
    ctx: &CompileContext,
    findings: &mut Vec<Finding>,
) -> Result<(), TaskFailure> {
    let mut reported = Vec::new();
    match mtl {
        Some(mtl) => {
            let converted = model_for_fox(model, &part.bytes, mtl, ctx, &mut reported)?;
            part.bytes = converted.model;
            part.skeleton = match (converted.skeleton, part.skeleton.take()) {
                (Some(converted), Some(member)) if converted != member => {
                    return Err(skeleton_conflict());
                }
                (converted, member) => member.or(converted),
            };
        }
        None => {
            let skeleton = part.skeleton.as_deref();
            if let Some(converted) = fmdl_for_fox(model, &part.bytes, skeleton, ctx, &mut reported)?
            {
                part.bytes = converted.model;
                part.skeleton = converted.skeleton;
            }
        }
    }
    // A member's own `.skl` of a slotless role never gets here (`PlayerFile::SlotlessSkeleton`,
    // reported by validation and never read): only a conversion's can.
    if skeleton_slot(owner, part.name).is_none() {
        part.skeleton = None;
    }
    if owner == package {
        findings.extend(reported);
    }
    Ok(())
}

/// The parts `part`, a model of the folder's that its task's findings name `model`
/// (`conversion::source_name`), gives `package`: the part itself, or, when it is a hand-split
/// face part (`hand_split`), what the hand auto-split leaves the package
/// (`model_conversion/hand_split.md`). The face keeps the body, under the part's name and
/// path, so it pairs the part's skeleton, and is told so by `model_hand_split` naming the
/// model and the gloves made; a body left with no face (a model that was all hand) is
/// no part at all. The gloves get a `glove_l` and a `glove_r` part, for each hand the model
/// has, with the part's path and textures and no skeleton, merged with any authored glove of
/// that name like any other part. A model the split cannot read or write fails the task with
/// `model_conversion_failed`.
fn parts_of(
    part: Part,
    model: &str,
    hand_split: bool,
    package: ModelPackage,
    ctx: &CompileContext,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Part>, TaskFailure> {
    if !hand_split {
        return Ok(vec![part]);
    }
    let split = {
        // The split's model, its IR and its written parts, charged at the source's size, an
        // estimate of each form, while they are built.
        let _split_charge = ctx.budget.charge(part.bytes.len());
        split_fmdl(&part.bytes).map_err(|error| TaskFailure {
            code: Code::ModelConversionFailed,
            context: vec![("model", model.to_owned()), ("error", format!("{error:#}"))],
        })?
    };
    let gloves = [("glove_l", split.glove_l), ("glove_r", split.glove_r)];
    match package {
        ModelPackage::Face => {
            let made: Vec<&str> = gloves
                .iter()
                .filter(|(_, bytes)| bytes.is_some())
                .map(|(name, _)| *name)
                .collect();
            findings.push((
                Code::ModelHandSplit,
                Disposition::Keep,
                vec![("model", model.to_owned()), ("gloves", made.join(", "))],
            ));
            Ok(split
                .body
                .map(|bytes| Part { bytes, ..part })
                .into_iter()
                .collect())
        }
        ModelPackage::Gloves => Ok(gloves
            .into_iter()
            .filter_map(|(name, bytes)| {
                Some(Part {
                    name,
                    path: part.path.clone(),
                    bytes: bytes?,
                    skeleton: None,
                    textures: part.textures,
                })
            })
            .collect()),
        ModelPackage::Boots => {
            unreachable!("only the face and gloves tasks read a hand-split part (`package`)")
        }
    }
}

/// An FMDL after the hand auto-split, each part written back as an FMDL.
struct SplitFmdl {
    /// The model without its hands; `None` when no face is left.
    body: Option<Vec<u8>>,
    /// The left hand, when the model weighs vertices on an `skh_*_l` bone.
    glove_l: Option<Vec<u8>>,
    /// The right hand, when the model weighs vertices on an `skh_*_r` bone.
    glove_r: Option<Vec<u8>>,
}

/// The FMDL `bytes` split at the wrists (`model_convert`'s `split_by_skeleton_group`), through
/// `model_convert`'s IR and back.
fn split_fmdl(bytes: &[u8]) -> anyhow::Result<SplitFmdl> {
    let model = Model::from_file(&FmdlFile::read(bytes)?)?;
    // Imported without the paired `.skl`: the round trip then keeps the FMDL's own bone table,
    // which is what the split keeps (measured, `hand_split.md` "Pipeline integration"). The
    // import's and the export's findings, `model_convert`'s loss codes, are left unreported
    // on purpose: cross-format conversion, a later step, maps those codes for every converted
    // model, and on the measured model the round trip reported only the dropped bone-matrix
    // block, which no community FMDL carries.
    let ir = fmdl_to_ir(&model, None)?.model;
    let split = split_by_skeleton_group(&ir);
    // The export's own `.skl`, for bones the game's template lacks, is not packed: the body
    // keeps the part's paired skeleton, and a glove has no skeleton slot.
    let write = |part: &CanonicalModel| -> anyhow::Result<Vec<u8>> {
        Ok(ir_to_fmdl(part)?.model.to_file()?.write())
    };
    let body = if split.body.meshes.is_empty() {
        None
    } else {
        Some(write(&split.body)?)
    };
    Ok(SplitFmdl {
        body,
        glove_l: split.glove_l.as_ref().map(write).transpose()?,
        glove_r: split.glove_r.as_ref().map(write).transpose()?,
    })
}

/// Points `path`, one texture reference of a part, at where its texture is: the directory of
/// the first of `places`, each the stems of the textures packed in a directory for the part,
/// that holds its stem, or a variant of its set when it is a kit reference (`pants_kitN`).
/// Else a path into the team's pre-Fox Common folder (`model/character/uniform/common/<team>/`,
/// how a member's pre-Fox `.mtl` names a texture of `Common/`), read as the deep pass reads it
/// (`user_face_xml::reference`), goes to the directory of `common`, the `Common/` textures, when
/// they hold its stem: the evidence on which the deep pass calls it supplied (`pipeline.md`
/// step 3 "Format conversion"). Any other texture is one of the game's own, whose directory
/// names the team as `000`, replaced by `team_segment`. The file name is never changed: the
/// game itself respells a reference for the kit picked.
fn point_texture(
    path: &mut TexturePath,
    places: &[(&BTreeSet<String>, &str)],
    common: (&BTreeSet<String>, &str),
    team_segment: &str,
) {
    let stem = file_stem(&path.file_name);
    let holds = |stems: &BTreeSet<String>| {
        stems.contains(&vtree::fold_name(stem))
            || has_variant_among(stem, stems.iter().map(String::as_str))
    };
    let names_pre_fox_common = || {
        let written = format!("{}{}", path.directory, path.file_name);
        matches!(reference(&written), Reference::Common { .. })
    };
    let (common_stems, common_directory) = common;
    let place = places
        .iter()
        .find(|(stems, _)| holds(stems))
        .map(|(_, directory)| *directory)
        .or_else(|| (names_pre_fox_common() && holds(common_stems)).then_some(common_directory));
    path.directory = match place {
        Some(directory) => directory.to_owned(),
        None => path.directory.replace("/000/", team_segment),
    };
}

/// Whether a texture one of a part's meshes uses is supplied (`pipeline.md` "Resolved
/// decisions", "A texture a model names must exist").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextureSupply {
    /// Supplied, or not looked for: a `dummy_` stem, which the game substitutes, or a path
    /// outside the team's Common output (the folder's own textures, the game's own).
    Supplied,
    /// Neither the export nor an installed CPK loaded before the run's holds it.
    Missing,
    /// The export does not hold it, and the installed CPKs cannot be looked in.
    Unknown,
}

/// Whether the texture at `path`, already pointed where it goes, is supplied: not looked for
/// when its stem starts with `dummy_` or its directory is not `common_directory`, the team's
/// Common texture directory; supplied when its stem, or a variant of its set for a kit
/// reference (`pants_kitN`), is among `common_stems` (the export's Common textures and the
/// folder's links, folded), or when `installed_holds` says an installed CPK holds the stem's
/// Common texture (`None`: they cannot be looked in). Directories and stems compare folded.
fn texture_supply(
    path: &TexturePath,
    common_stems: &[&BTreeSet<String>],
    common_directory: &str,
    installed_holds: impl Fn(&str) -> Option<bool>,
) -> TextureSupply {
    let stem = file_stem(&path.file_name);
    let folded = vtree::fold_name(stem);
    if folded.starts_with("dummy_")
        || vtree::fold_name(&path.directory) != vtree::fold_name(common_directory)
    {
        return TextureSupply::Supplied;
    }
    if common_stems.iter().any(|stems| {
        stems.contains(&folded) || has_variant_among(stem, stems.iter().map(String::as_str))
    }) {
        return TextureSupply::Supplied;
    }
    match installed_holds(stem) {
        Some(true) => TextureSupply::Supplied,
        Some(false) => TextureSupply::Missing,
        None => TextureSupply::Unknown,
    }
}

/// `parts`, several models resolving to one allowed name with their texture paths rewritten,
/// merged into one FMDL in the given order.
fn merge_parts(parts: &[FmdlFile]) -> Result<FmdlFile, TaskFailure> {
    let models = parts
        .iter()
        .map(Model::from_file)
        .collect::<Result<Vec<Model>, fmdl::FmdlError>>()?;
    Ok(merge(&models)?.to_file()?)
}

impl From<MergeError> for TaskFailure {
    fn from(error: MergeError) -> TaskFailure {
        match error {
            MergeError::MaterialConflict { name } => TaskFailure {
                code: Code::MergeMaterialConflict,
                context: vec![("material", name)],
            },
            MergeError::SkeletonConflict { name } | MergeError::DuplicateBoneName { name } => {
                TaskFailure {
                    code: Code::SklMergeConflict,
                    context: vec![("bone", name)],
                }
            }
            // Not a disagreement between the parts a member resolves by name: a part that
            // fails validation, or parts whose anti-blur duplicates are encoded in some and
            // not others.
            MergeError::MixedAntiblur | MergeError::Other(_) => {
                TaskFailure::from(anyhow::Error::from(error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Team 714's Common texture directory.
    const COMMON_714: &str = "/Assets/pes16/model/character/common/714/sourceimages/";

    /// What `texture_supply` makes of `file_name` in `directory` for team 714, the export's
    /// Common stems being `common` and the folder's links `linked`, an installed CPK answering
    /// `installed` for every stem.
    fn supply(
        directory: &str,
        file_name: &str,
        common: &[&str],
        linked: &[&str],
        installed: Option<bool>,
    ) -> TextureSupply {
        let path = TexturePath {
            file_name: file_name.to_owned(),
            directory: directory.to_owned(),
        };
        let set = |stems: &[&str]| -> BTreeSet<String> {
            stems.iter().map(|stem| (*stem).to_owned()).collect()
        };
        let (common, linked) = (set(common), set(linked));
        texture_supply(&path, &[&common, &linked], COMMON_714, |_| installed)
    }

    #[test]
    fn a_dummy_stem_and_a_path_outside_the_team_s_common_output_are_not_looked_for() {
        for file_name in ["dummy_kit.dds", "dummy_kit_srm.dds", "Dummy_Kit.dds"] {
            assert_eq!(
                supply(COMMON_714, file_name, &[], &[], Some(false)),
                TextureSupply::Supplied,
                "{file_name}"
            );
        }
        let home = "/Assets/pes16/model/character/common/714/05 - A/sourceimages/";
        assert_eq!(
            supply(home, "skin.dds", &[], &[], Some(false)),
            TextureSupply::Supplied
        );
        let game = "/Assets/pes16/model/character/common/sourceimages/";
        assert_eq!(
            supply(game, "skin.dds", &[], &[], None),
            TextureSupply::Supplied
        );
        // Not `dummy_`: looked for like any other stem.
        assert_eq!(
            supply(COMMON_714, "dummy.dds", &[], &[], Some(false)),
            TextureSupply::Missing
        );
    }

    #[test]
    fn a_common_path_is_supplied_by_common_a_link_or_an_installed_cpk() {
        let missing = Some(false);
        assert_eq!(
            supply(COMMON_714, "hair.dds", &["hair"], &[], missing),
            TextureSupply::Supplied
        );
        assert_eq!(
            supply(COMMON_714, "hair.dds", &[], &["hair"], missing),
            TextureSupply::Supplied
        );
        // Stems and directories compare folded.
        let shouted = "/ASSETS/pes16/model/character/common/714/sourceimages/";
        assert_eq!(
            supply(shouted, "Hair.dds", &["hair"], &[], missing),
            TextureSupply::Supplied
        );
        assert_eq!(
            supply(shouted, "Hair.dds", &[], &[], missing),
            TextureSupply::Missing
        );
        assert_eq!(
            supply(COMMON_714, "pants_kitN.dds", &["pants_kit1"], &[], missing),
            TextureSupply::Supplied
        );
        assert_eq!(
            supply(COMMON_714, "pants_kitN.dds", &["socks_kit1"], &[], missing),
            TextureSupply::Missing
        );
        assert_eq!(
            supply(COMMON_714, "hair.dds", &["skin"], &["face"], Some(true)),
            TextureSupply::Supplied
        );
        assert_eq!(
            supply(COMMON_714, "hair.dds", &["skin"], &["face"], Some(false)),
            TextureSupply::Missing
        );
        assert_eq!(
            supply(COMMON_714, "hair.dds", &["skin"], &["face"], None),
            TextureSupply::Unknown
        );
    }

    /// The directory `point_texture` gives the path `file_name` in the game's team `000`
    /// folder for team 792, the part's own stems being `own`, going to `/home/`, and its
    /// linked stems `linked`, going to `/common/`; asserts the file name is kept.
    fn pointed_between(file_name: &str, own: &[&str], linked: &[&str]) -> String {
        let mut path = TexturePath {
            file_name: file_name.to_owned(),
            directory: "/Assets/pes16/model/character/common/000/sourceimages/".to_owned(),
        };
        let set = |stems: &[&str]| -> BTreeSet<String> {
            stems.iter().map(|stem| (*stem).to_owned()).collect()
        };
        let (own, linked) = (set(own), set(linked));
        point_texture(
            &mut path,
            &[(&own, "/home/"), (&linked, "/common/")],
            (&linked, "/common/"),
            "/792/",
        );
        assert_eq!(path.file_name, file_name);
        path.directory
    }

    /// The directory `point_texture` gives `file_name` in `directory` for team 714, the part's
    /// own stems being `own`, going to `/home/`, and `Common/`'s `common`, going to `/common/`
    /// for a pre-Fox Common path.
    fn pointed_from(directory: &str, file_name: &str, own: &[&str], common: &[&str]) -> String {
        let mut path = TexturePath {
            file_name: file_name.to_owned(),
            directory: directory.to_owned(),
        };
        let set = |stems: &[&str]| -> BTreeSet<String> {
            stems.iter().map(|stem| (*stem).to_owned()).collect()
        };
        let (own, common) = (set(own), set(common));
        point_texture(
            &mut path,
            &[(&own, "/home/")],
            (&common, "/common/"),
            "/714/",
        );
        assert_eq!(path.file_name, file_name);
        path.directory
    }

    #[test]
    fn a_pre_fox_common_path_goes_to_the_common_directory_when_common_holds_its_stem() {
        let pre_fox = "model/character/uniform/common/714/";
        assert_eq!(
            pointed_from(pre_fox, "shirt.dds", &[], &["shirt"]),
            "/common/"
        );
        // The team's segment as a member's pre-Fox `.mtl` may spell it, and a kit reference.
        for segment in ["XXX", "000"] {
            let directory = format!("model/character/uniform/common/{segment}/");
            assert_eq!(
                pointed_from(&directory, "shirt.dds", &[], &["shirt"]),
                "/common/",
                "{segment}"
            );
        }
        assert_eq!(
            pointed_from(pre_fox, "pants_kitN.dds", &[], &["pants_kit2"]),
            "/common/"
        );
        // The folder's own texture of the stem comes first, as for any path.
        assert_eq!(
            pointed_from(pre_fox, "shirt.dds", &["shirt"], &["shirt"]),
            "/home/"
        );
        // Left as written when `Common/` does not hold the stem.
        assert_eq!(pointed_from(pre_fox, "shirt.dds", &[], &["skin"]), pre_fox);
        // Only a pre-Fox Common path reaches `Common/`: a `./` path, a player's pre-Fox texture
        // home below the team's folder, or a Fox path of the game's own does not.
        for directory in [
            "./",
            "model/character/uniform/common/714/05 - A/",
            "/Assets/pes16/model/character/common/sourceimages/",
        ] {
            assert_eq!(
                pointed_from(directory, "shirt.dds", &[], &["shirt"]),
                directory,
                "{directory}"
            );
        }
    }

    /// `pointed_between` with the part's own stems `stems` and no linked stem.
    fn pointed(file_name: &str, stems: &[&str]) -> String {
        pointed_between(file_name, stems, &[])
    }

    #[test]
    fn a_stem_goes_to_the_first_place_holding_it_and_a_linked_one_to_the_common_directory() {
        let own = ["skin"];
        let linked = ["hair", "pants_kit2"];
        let game = "/Assets/pes16/model/character/common/792/sourceimages/";
        assert_eq!(pointed_between("skin.dds", &own, &linked), "/home/");
        assert_eq!(pointed_between("hair.dds", &own, &linked), "/common/");
        // Kit references resolve in each place by the variants that place holds.
        assert_eq!(pointed_between("pants_kitN.dds", &own, &linked), "/common/");
        assert_eq!(
            pointed_between("pants_kitN.dds", &["pants_kit1"], &linked),
            "/home/"
        );
        assert_eq!(pointed_between("other.dds", &own, &linked), game);
        // A stem held in both places goes to the first.
        assert_eq!(pointed_between("hair.dds", &["hair"], &linked), "/home/");
    }

    #[test]
    fn a_kit_reference_is_the_part_s_own_texture_when_a_variant_of_its_set_is() {
        let game = "/Assets/pes16/model/character/common/792/sourceimages/";
        assert_eq!(pointed("pants_kitN.dds", &["pants_kit2"]), "/home/");
        assert_eq!(pointed("shirt.dds", &["shirt", "pants_kit2"]), "/home/");
        // With no variant of its own set among the stems, it is any other path: one of the
        // game's own.
        assert_eq!(pointed("pants_kitN.dds", &[]), game);
        assert_eq!(
            pointed("pants_kitN.dds", &["socks_kit2", "pants", "pants_kitN_x"]),
            game
        );
        // The legacy `dummy_kit` holds no token: only its team folder is replaced.
        assert_eq!(pointed("dummy_kit.dds", &["pants_kit2"]), game);
    }
}
