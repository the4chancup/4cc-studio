//! One package of a model folder's Fox models (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", steps 2, 3 and 7): its models renamed to their allowed
//! names, each part's texture paths pointed at where its textures go, the parts resolving to
//! one name merged into one model, packed with the files that go beside them into one `.fpk`
//! emitted under each of the package's ids.

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::FileDescriptor;
use fmdl::ops::merge::{MergeError, merge};
use fmdl::ops::paths::{TexturePath, rewrite_texture_paths};
use fmdl::{FmdlFile, Model};
use fpk::{FpkFile, FpkKind};
use studio_core::Disposition;
use vtree::ScopePath;

use super::face_diff::{self, FaceDiffError};
use super::{Entry, Finding, TaskFailure, TaskFiles, take};
use crate::messages::Code;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem};
use crate::templates;

/// One model of the package: a part of the output model its allowed name names, from the
/// folder's own files, a combined shared folder's, or the export's `Common/` folder through a
/// `.common` link.
struct Part {
    /// The allowed name the part resolves to (`boots`), the output model's.
    name: &'static str,
    /// The part's export path, which with its file name orders the parts of one output.
    path: ScopePath,
    /// The part's bytes, an FMDL.
    bytes: Vec<u8>,
    /// The skeleton paired with the part: the `.skl` of its stem in its own source folder.
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
}

/// The `package` of `folder`, compiled from its files' bytes in `files` for team `team_id`
/// and emitted under each of `ids`: the package's `.fpk` and an empty `.fpkd` per id. A
/// merge of several parts into one model is noted in `findings` as `fmdl_merged`.
pub(super) fn package(
    folder: &ModelFolder,
    package: ModelPackage,
    ids: &[u32],
    team_id: u16,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Entry>, TaskFailure> {
    let mut parts: Vec<Part> = Vec::new();
    let mut texture_stems = BTreeSet::new();
    let mut fpk = FpkFile::new(FpkKind::Fpk);
    for (_, _, source_files) in folder.roles() {
        if package == ModelPackage::Face
            && let Some(xml) = face_diff_given_twice(&source_files)
        {
            return Err(TaskFailure {
                code: Code::XmlDifConflict,
                context: vec![("file", xml.path.as_str().to_owned())],
            });
        }
        // A skeleton pairs with the model of its stem in the same directory: keyed by the
        // path up to the extension, since a player folder's reserved subfolder may hold a
        // model of the same name as one directly in the folder.
        let mut skeletons: BTreeMap<&str, Vec<u8>> = BTreeMap::new();
        let mut source_parts: Vec<Part> = Vec::new();
        for (file, role) in source_files {
            let mut part = |name, textures| Part {
                name,
                path: file.path.clone(),
                bytes: take(files, file),
                skeleton: None,
                textures,
            };
            match role {
                PlayerFile::Model {
                    package: owner,
                    name,
                } if owner == package => source_parts.push(part(name, PartTextures::Folder)),
                PlayerFile::CommonModel {
                    package: owner,
                    name,
                } if owner == package => source_parts.push(part(name, PartTextures::Common)),
                PlayerFile::Skeleton { package: owner, .. } if owner == package => {
                    skeletons.insert(file_stem(file.path.as_str()), take(files, file));
                }
                PlayerFile::Packed {
                    package: owner,
                    name,
                } if owner == package => {
                    let bytes = take(files, file);
                    if name == "face_diff.bin" {
                        face_diff::check(&bytes)
                            .map_err(|error| face_diff_invalid(file, &error))?;
                    }
                    fpk.insert(name.to_owned(), bytes);
                }
                PlayerFile::FaceDiffXml if package == ModelPackage::Face => {
                    let bytes = face_diff::from_xml(&take(files, file))
                        .map_err(|error| face_diff_invalid(file, &error))?;
                    fpk.insert("face_diff.bin".to_owned(), bytes);
                }
                // The textures are the textures task's; this task only points its models at
                // them.
                PlayerFile::Texture(stem, _) => {
                    texture_stems.insert(stem);
                }
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel { .. }
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::FaceDiffXml
                | PlayerFile::Packed { .. } => {}
            }
        }
        for part in &mut source_parts {
            part.skeleton = skeletons.remove(file_stem(part.path.as_str()));
        }
        parts.extend(source_parts);
    }

    // The parts of one output model go in alphabetical source order, by file name folded as
    // the file system folds it and then by export path, so a recompile gives the same model.
    parts.sort_by_cached_key(|part| {
        (
            vtree::fold_name(part.path.name()),
            part.path.as_str().to_owned(),
        )
    });
    let mut by_name: BTreeMap<&'static str, Vec<Part>> = BTreeMap::new();
    for part in parts {
        by_name.entry(part.name).or_default().push(part);
    }

    // A folder's textures sit in its one texture home, once however many ids the package is
    // emitted under: every copy of the model points at that one location. A Common part's own
    // textures stay in the team's Common output, where the export's Common textures task puts
    // them once for every player linking the model (`pipeline.md` step 6: a texture resolved
    // in Common is never relocated).
    let texture_directory = folder.textures.directory(team_id);
    let common_directory = paths::common_texture_directory(team_id);
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
            let (stems, directory) = match part.textures {
                PartTextures::Folder => (&texture_stems, &texture_directory),
                PartTextures::Common => (&folder.common_texture_stems, &common_directory),
            };
            let mut model = FmdlFile::read(&part.bytes)?;
            rewrite_texture_paths(&mut model, |path| {
                point_texture(path, stems, directory, &team_segment);
            })?;
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
        fpk.insert(format!("{name}.fmdl"), bytes);
    }
    // The game loads the boots and the hair with a skeleton beside them, under the slot's
    // name (`player_folders.md` "SKL pairing"): the parts' own when they bring one, else the
    // standard full-body one. A face also needs its `face_diff.bin`, and a hair its
    // `fcl_hair_sim.fclo`: a source's own when one holds it, else the bundled template
    // ("What the injected files are").
    match package {
        ModelPackage::Boots => {
            fpk.insert(
                "boots.skl".to_owned(),
                skeleton.unwrap_or_else(|| templates::BODY_SKELETON.to_vec()),
            );
        }
        ModelPackage::Face => {
            if fpk.get("face_diff.bin").is_none() {
                fpk.insert("face_diff.bin".to_owned(), templates::FACE_DIFF.to_vec());
            }
            if fpk.get("fcl_hair.fmdl").is_some() {
                if fpk.get("fcl_hair_sim.fclo").is_none() {
                    fpk.insert(
                        "fcl_hair_sim.fclo".to_owned(),
                        templates::FCL_HAIR_SIM_FCLO.to_vec(),
                    );
                }
                fpk.insert(
                    "fcl_hair_sim.skl".to_owned(),
                    skeleton.unwrap_or_else(|| templates::BODY_SKELETON.to_vec()),
                );
            }
        }
        // The gloves have no skeleton slot, and no `.skl` pairs with a glove.
        ModelPackage::Gloves => {}
    }

    let fpk = fpk.write();
    // The game opens a package's `.fpkd` beside its `.fpk`; with the textures in the common
    // folder there is nothing to put in it, so it is an empty package.
    let empty = FpkFile::new(FpkKind::Fpkd).write();
    let stem = package.file_stem();
    let mut entries = Vec::new();
    // The game finds a package by its id, so each slot gets its own copy; the last slot takes
    // the buffer itself rather than one more copy.
    if let Some((last_id, other_ids)) = ids.split_last() {
        for id in other_ids {
            let folder = paths::package_folder(package, *id);
            entries.push((format!("{folder}/{stem}.fpk"), fpk.clone()));
            entries.push((format!("{folder}/{stem}.fpkd"), empty.clone()));
        }
        let folder = paths::package_folder(package, *last_id);
        entries.push((format!("{folder}/{stem}.fpk"), fpk));
        entries.push((format!("{folder}/{stem}.fpkd"), empty));
    }
    Ok(entries)
}

/// The `face_diff.xml` among `source_files`, the files of one source with their roles, when
/// the source also holds a `face_diff.bin`: one folder giving its face diff twice
/// (`player_folders.md` "`face_diff.xml`"). Neither file is read: the conflict is the finding.
fn face_diff_given_twice<'a>(
    source_files: &[(&'a FileDescriptor, PlayerFile)],
) -> Option<&'a FileDescriptor> {
    let holds_bin = source_files.iter().any(|(_, role)| {
        matches!(
            role,
            PlayerFile::Packed {
                name: "face_diff.bin",
                ..
            }
        )
    });
    source_files
        .iter()
        .find(|(_, role)| *role == PlayerFile::FaceDiffXml)
        .filter(|_| holds_bin)
        .map(|(file, _)| *file)
}

/// The `face_diff_invalid` failure of `file`, a face diff in either form, for `error`.
fn face_diff_invalid(file: &FileDescriptor, error: &FaceDiffError) -> TaskFailure {
    TaskFailure {
        code: Code::FaceDiffInvalid,
        context: vec![
            ("file", file.path.as_str().to_owned()),
            ("reason", error.to_string()),
        ],
    }
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
        return Err(TaskFailure {
            code: Code::SklMergeConflict,
            context: vec![("skeleton", "differs".to_owned())],
        });
    }
    Ok(first)
}

/// Points `path`, one texture reference of a part, at where its texture is: `directory` when
/// its stem is one of `stems`, the textures packed there for the part; any other texture is one
/// of the game's own, whose directory names the team as `000`, replaced by `team_segment`.
fn point_texture(
    path: &mut TexturePath,
    stems: &BTreeSet<String>,
    directory: &str,
    team_segment: &str,
) {
    if stems.contains(file_stem(&path.file_name)) {
        path.directory = directory.to_owned();
    } else {
        path.directory = path.directory.replace("/000/", team_segment);
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
