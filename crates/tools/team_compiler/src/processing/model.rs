//! One package of a model folder's Fox models (`team_compiler/pipeline.md` "3.
//! Per-model-folder parallel steps", steps 2, 3 and 7): its models renamed to their allowed
//! names, each part's texture paths pointed at where its textures go and the textures its
//! meshes use looked for, the parts resolving to one name merged into one model, packed with
//! the files that go beside them into one `.fpk` emitted under each of the package's ids.

use std::collections::{BTreeMap, BTreeSet};

use fmdl::ops::merge::{MergeError, merge};
use fmdl::ops::paths::{TexturePath, rewrite_texture_paths, used_texture_paths};
use fmdl::{FmdlFile, Model};
use fpk::{FpkFile, FpkKind};
use studio_core::Disposition;
use vtree::ScopePath;

use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, take};
use crate::face_diff;
use crate::kit_variants::{KitToken, kit_token};
use crate::messages::Code;
use crate::paths;
use crate::plan::ModelFolder;
use crate::plan::subset::{ModelPackage, PlayerFile, file_stem};

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
/// and emitted under each of `ids`: the package's `.fpk` and an empty `.fpkd` per id, a file
/// the game needs beside the models that no source holds taken from the run's templates. A
/// merge of several parts into one model is noted in `findings` as `fmdl_merged`. A texture a
/// part's mesh uses that nothing supplies (`texture_supply`) fails the task with
/// `fmdl_texture_not_found` at the first one, or, when the installed CPKs cannot be looked in,
/// is noted in `findings` as `fmdl_texture_not_found`, once per texture.
pub(super) fn package(
    folder: &ModelFolder,
    package: ModelPackage,
    ids: &[u32],
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<Vec<Entry>, TaskFailure> {
    let mut parts: Vec<Part> = Vec::new();
    // The stems of the folder's own textures, and of the Common textures its `.common` links
    // stand for.
    let mut texture_stems = BTreeSet::new();
    let mut linked_stems = BTreeSet::new();
    let mut fpk = FpkFile::new(FpkKind::Fpk);
    for (_, _, source_files) in folder.roles() {
        // A skeleton pairs with the model of its stem in the same directory: keyed by the
        // path up to the extension, case-folded as the file system folds it (planning pairs
        // a Common skeleton with its model folded too), since a player folder's reserved
        // subfolder may hold a model of the same name as one directly in the folder.
        let mut skeletons: BTreeMap<String, Vec<u8>> = BTreeMap::new();
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
                    skeletons.insert(
                        vtree::fold_name(file_stem(file.path.as_str())),
                        take(files, file),
                    );
                }
                PlayerFile::Packed {
                    package: owner,
                    name,
                } if owner == package => {
                    fpk.insert(name.to_owned(), take(files, file));
                }
                // The deep pass has dropped a folder whose face diff fails to decode, or that
                // gives it in both forms, so a failure here is not a member's mistake.
                PlayerFile::FaceDiffXml if package == ModelPackage::Face => {
                    let bytes = face_diff::from_xml(&take(files, file))
                        .map_err(|error| anyhow::anyhow!("{}: {error}", file.path.as_str()))?;
                    fpk.insert("face_diff.bin".to_owned(), bytes);
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
                | PlayerFile::Skeleton { .. }
                | PlayerFile::SlotlessSkeleton
                | PlayerFile::UnusedFaceFile
                | PlayerFile::LeftOutKitVariant
                | PlayerFile::FaceDiffXml
                | PlayerFile::Packed { .. } => {}
            }
        }
        for part in &mut source_parts {
            part.skeleton = skeletons.remove(&vtree::fold_name(file_stem(part.path.as_str())));
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
    // emitted under: every copy of the model points at that one location. A texture resolved
    // in Common, a Common part's own or one a folder's link stands for, stays in the team's
    // Common output, where the export's Common textures task puts it once for every player
    // (`pipeline.md` step 6: a texture resolved in Common is never relocated). A folder part
    // looks in the folder's textures first. Validation refuses a player folder holding a
    // texture and a link of one stem (`texture_stem_conflict`), but not a link beside a
    // combined shared folder's texture of its stem: there the shared folder's texture wins.
    let texture_directory = folder.textures.directory(team_id);
    let common_directory = paths::common_texture_directory(team_id);
    let folder_places = [
        (&texture_stems, texture_directory.as_str()),
        (&linked_stems, common_directory.as_str()),
    ];
    let common_places = [(&folder.common_texture_stems, common_directory.as_str())];
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
            };
            let mut model = FmdlFile::read(&part.bytes)?;
            rewrite_texture_paths(&mut model, |path| {
                point_texture(path, places, &team_segment);
            })?;
            for path in used_texture_paths(&model)? {
                let installed_holds =
                    |stem: &str| ctx.installed.holds(&paths::common_texture(team_id, stem));
                let context = || {
                    vec![
                        ("model", part.path.name().to_owned()),
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
        fpk.insert(format!("{name}.fmdl"), bytes);
    }
    // The game loads the boots and the hair with a skeleton beside them, under the slot's
    // name (`player_folders.md` "SKL pairing"): the parts' own when they bring one, else the
    // standard full-body one. A face also needs its `face_diff.bin`, and a hair its
    // `fcl_hair_sim.fclo`: a source's own when one holds it, else the run's template
    // ("What the injected files are").
    match package {
        ModelPackage::Boots => {
            fpk.insert(
                "boots.skl".to_owned(),
                skeleton.unwrap_or_else(|| ctx.templates.body_skeleton().to_vec()),
            );
        }
        ModelPackage::Face => {
            if fpk.get("face_diff.bin").is_none() {
                fpk.insert(
                    "face_diff.bin".to_owned(),
                    ctx.templates.face_diff().to_vec(),
                );
            }
            if fpk.get("fcl_hair.fmdl").is_some() {
                if fpk.get("fcl_hair_sim.fclo").is_none() {
                    fpk.insert(
                        "fcl_hair_sim.fclo".to_owned(),
                        ctx.templates.fcl_hair_sim().to_vec(),
                    );
                }
                fpk.insert(
                    "fcl_hair_sim.skl".to_owned(),
                    skeleton.unwrap_or_else(|| ctx.templates.body_skeleton().to_vec()),
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

/// Points `path`, one texture reference of a part, at where its texture is: the directory of
/// the first of `places`, each the stems of the textures packed in a directory for the part,
/// that holds its stem, or a variant of its set when it is a kit reference (`pants_kitN`); any
/// other texture is one of the game's own, whose directory names the team as `000`, replaced
/// by `team_segment`. The file name is never changed: the game itself respells a reference for
/// the kit picked.
fn point_texture(path: &mut TexturePath, places: &[(&BTreeSet<String>, &str)], team_segment: &str) {
    let stem = file_stem(&path.file_name);
    let place = places.iter().find(|(stems, _)| {
        stems.contains(&vtree::fold_name(stem)) || has_variant_among(stem, stems)
    });
    path.directory = match place {
        Some((_, directory)) => (*directory).to_owned(),
        None => path.directory.replace("/000/", team_segment),
    };
}

/// Whether `stem` is a kit reference (`pants_kitN`, never a file of its own) and `stems` hold a
/// variant of its set (`pants_kit2`), compared as `stems` are, as spelled.
fn has_variant_among(stem: &str, stems: &BTreeSet<String>) -> bool {
    let Some((KitToken::Reference, reference)) = kit_token(stem) else {
        return false;
    };
    stems.iter().any(|held| {
        matches!(kit_token(held), Some((KitToken::Variant(_), held_reference)) if vtree::fold_name(&held_reference) == vtree::fold_name(&reference))
    })
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
    if common_stems
        .iter()
        .any(|stems| stems.contains(&folded) || has_variant_among(stem, stems))
    {
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
            "/792/",
        );
        assert_eq!(path.file_name, file_name);
        path.directory
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
