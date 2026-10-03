//! Stages 3 and 4, one task's work (`team_compiler/pipeline.md` "3. Per-model-folder parallel
//! steps", "4. Per-export non-model steps"): its source files, already read, converted and
//! packed into the CPK entries the task commits, whole or not at all.

mod face_diff;
mod kit;
mod model;
mod texture;

use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::Arc;

use aesthetics_export::FileDescriptor;
use dds_convert::{CachePolicy, Converter};
use pes_version::PesVersion;
use pipeline::Permit;
use studio_core::{Disposition, Message, Scope};
use vtree::ScopePath;

use crate::messages::{Code, tool_message};
use crate::paths;
use crate::plan::subset::{ModelPackage, texture_format};
use crate::plan::{BuildTask, TaskGroup, TaskKind};

/// The bytes of every file a task reads (`TaskKind::files`), keyed by the file's export path:
/// read from the export's source by the coordinator before the task is processed.
pub(crate) type TaskFiles = BTreeMap<ScopePath, Vec<u8>>;

/// What every task of a run reads besides its own content.
pub(crate) struct CompileContext {
    /// The target version, which decides each texture's codec and container, the kit config's
    /// encoding and the portraits' file names.
    pub(crate) version: PesVersion,
    /// The run's texture converter, one for every task: the conversion cache in front of
    /// `dds_convert` (`libs/dds_convert.md` "In-memory conversion cache").
    pub(crate) converter: Converter,
    /// Whether the converter keeps what it converts, from the run's export count.
    pub(crate) cache: CachePolicy,
}

impl CompileContext {
    /// The context of a run for `version` compiling `compiled_exports` exports (those with at
    /// least one planned task), with a fresh converter.
    pub(crate) fn new(version: PesVersion, compiled_exports: usize) -> CompileContext {
        CompileContext {
            version,
            converter: Converter::new(),
            cache: cache_policy(compiled_exports),
        }
    }
}

/// Whether a run compiling `compiled_exports` exports caches its conversions: at most two,
/// the edit-and-test loop the cache is for; a larger run converts mostly first-seen textures
/// and would retain every one for nothing (`libs/dds_convert.md` "Engagement limit").
fn cache_policy(compiled_exports: usize) -> CachePolicy {
    if compiled_exports <= 2 {
        CachePolicy::Use
    } else {
        CachePolicy::Bypass
    }
}

/// One file in a container: its path in a CPK, or its name in a bin, and its bytes.
pub(crate) type Entry = (String, Vec<u8>);

/// A finding a task that succeeded makes on its folder: the code, what was done about it
/// (`Keep` for `fmdl_merged`; `DropFolder` for a `shared_texture_conflict`, whose losing
/// package the writer leaves out; `DropFile` for a Common texture left out on a texture
/// finding) and its context.
pub(crate) type Finding = (Code, Disposition, Vec<(&'static str, String)>);

/// Why a task failed: the finding reported on its folder (its file, for a portrait). A merge
/// conflict between parts has its own code (`merge_material_conflict`, `skl_merge_conflict`),
/// and so has a texture finding (`texture_too_small`, ...); any other error is
/// `folder_pack_failed` carrying the error chain.
pub(crate) struct TaskFailure {
    /// The finding's code.
    pub(crate) code: Code,
    /// Its context entries (`material=shirt`, `error=<chain>`).
    pub(crate) context: Vec<(&'static str, String)>,
}

impl From<anyhow::Error> for TaskFailure {
    fn from(error: anyhow::Error) -> TaskFailure {
        TaskFailure {
            code: Code::FolderPackFailed,
            context: vec![("error", format!("{error:#}"))],
        }
    }
}

impl From<fmdl::FmdlError> for TaskFailure {
    fn from(error: fmdl::FmdlError) -> TaskFailure {
        TaskFailure::from(anyhow::Error::from(error))
    }
}

/// One task's result, handed to the writer.
pub(crate) struct TaskBatch {
    /// The task's position in the manifest: the writer commits batches in this order.
    pub(crate) index: usize,
    /// The CPK entries the task commits; empty when the task failed.
    pub(crate) entries: Vec<Entry>,
    /// The manifest positions of the player folder's group the task belongs to
    /// (`TaskGroup::tasks`), its textures batch last: the writer holds the group's packages
    /// until that batch arrives and decides them all. `None` for a task the writer commits on
    /// its own.
    pub(crate) group: Option<Range<usize>>,
    /// The manifest positions of the group's packages a textures batch drops as the losers of
    /// a `shared_texture_conflict`, which the writer skips: their textures are not in the CPK.
    /// Empty for every other batch.
    pub(crate) skipped: Vec<usize>,
    /// A kit's config as a `UniformParameter.bin` entry, applied only when the batch is
    /// committed.
    pub(crate) uniparam: Option<Entry>,
    /// The task's findings.
    pub(crate) messages: Vec<Message>,
    /// The memory the task was charged, released once the writer has its entries. A `.7z`
    /// export's tasks share one, its whole decompressed size, released with the last of them.
    pub(crate) permit: Option<Arc<Permit>>,
}

/// Runs `task`, the manifest's task number `index`, over `files`, the bytes of every file it
/// reads. A task that fails commits nothing: its batch has no entries and reports its
/// `TaskFailure` on its folder, which is left out of the CPK.
pub(crate) fn process_task(
    index: usize,
    task: BuildTask,
    mut files: TaskFiles,
    ctx: &CompileContext,
) -> TaskBatch {
    let mut findings = Vec::new();
    let mut skipped = Vec::new();
    let result = match &task.kind {
        TaskKind::Models {
            folder,
            package,
            ids,
        } => model::package(
            folder,
            *package,
            ids,
            task.team_id,
            &mut files,
            &mut findings,
        )
        .map(|entries| (entries, None)),
        TaskKind::Textures { folder, .. } => {
            texture::folder_textures(folder, task.team_id, ctx, &mut files, &mut findings).map(
                |(entries, dropped)| {
                    skipped = dropped_positions(task.group.as_ref(), &dropped);
                    (entries, None)
                },
            )
        }
        TaskKind::CommonTextures { textures, .. } => {
            texture::common_textures(textures, task.team_id, ctx, &mut files, &mut findings)
                .map(|entries| (entries, None))
        }
        TaskKind::Portrait { player_id, file } => {
            let name = file.path.name();
            let format = texture_format(name)
                .expect("planning lists a portrait by an extension `dds_convert` accepts");
            texture::portrait(format, name, take(&mut files, file))
                .map(|bytes| {
                    (
                        vec![(paths::portrait(ctx.version, *player_id), bytes)],
                        None,
                    )
                })
                .map_err(TaskFailure::from)
        }
        TaskKind::Kit { slot, kit } => kit::kit(*slot, kit, task.team_id, ctx, &mut files)
            .map(|(entries, config)| (entries, Some(config))),
    };
    let mut batch = TaskBatch {
        index,
        entries: Vec::new(),
        group: task.group.as_ref().map(|group| group.tasks.clone()),
        skipped,
        uniparam: None,
        messages: Vec::new(),
        permit: None,
    };
    let scope = Scope::Folder {
        export_id: task.export_id,
        path: task.kind.folder_path(),
    };
    match result {
        Ok((entries, uniparam)) => {
            batch.entries = entries;
            batch.uniparam = uniparam;
            batch.messages = findings
                .into_iter()
                .map(|(code, disposition, context)| {
                    tool_message(code, scope.clone(), disposition, context)
                })
                .collect();
        }
        // A failed task reports its failure alone: a note about a merge whose output is not
        // in the CPK would describe nothing the member can find. What was dropped is the
        // task's unit: a portrait task is its one file, every other task a folder.
        Err(failure) => {
            let disposition = match task.kind {
                TaskKind::Portrait { .. } => Disposition::DropFile,
                TaskKind::Models { .. }
                | TaskKind::Textures { .. }
                | TaskKind::CommonTextures { .. }
                | TaskKind::Kit { .. } => Disposition::DropFolder,
            };
            batch.messages.push(tool_message(
                failure.code,
                scope,
                disposition,
                failure.context,
            ));
        }
    }
    batch
}

/// The manifest positions of the tasks of the `dropped` packages in `group`, for the writer
/// to skip. A dropped package the folder has no task of (its textures came from a source
/// alone, a player's own folder with no face model standing for the face) needs no skip, and
/// a textures task outside a group has no package task beside it.
fn dropped_positions(group: Option<&TaskGroup>, dropped: &[ModelPackage]) -> Vec<usize> {
    let Some(group) = group else {
        return Vec::new();
    };
    dropped
        .iter()
        .filter_map(|package| group.packages.iter().position(|held| held == package))
        .map(|offset| group.tasks.start + offset)
        .collect()
}

/// The bytes of `file`, taken out of the task's `files`.
fn take(files: &mut TaskFiles, file: &FileDescriptor) -> Vec<u8> {
    files
        .remove(&file.path)
        .expect("the coordinator reads every file `TaskKind::files` lists, each once")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use aesthetics_export::{
        KitFolder, KitLayout, KitTexture, KitTextureSource, SharedModelFolder,
    };
    use fmdl::ops::paths::texture_paths;
    use fmdl::{FmdlFile, Model};
    use fpk::{FpkFile, FpkKind};
    use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};
    use studio_core::{ExportId, Severity};

    use super::*;
    use crate::paths::TextureHome;
    use crate::plan::{CombinedFolder, CommonModel, ModelFolder};
    use crate::templates;

    const PLAYER: &str = "Players/05 - The Chad Stormworks Player";
    const COMMON: &str = "Asset/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/#windx11/shirt.ftex";
    const COMMON_DIRECTORY: &str =
        "/Assets/pes16/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/";

    /// The tracer fixture's export folder.
    fn tracer() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tracer/studio/egg Tracer")
    }

    /// The export file at `relative`, described as the structure pass would.
    fn file(relative: &str) -> FileDescriptor {
        let path = ScopePath::new(relative).unwrap();
        FileDescriptor {
            size: std::fs::metadata(tracer().join(relative)).map_or(0, |data| data.len()),
            kind: aesthetics_export::classify(path.name()),
            source: path.clone(),
            path,
        }
    }

    /// The tracer's player folder holding the files `names`, its textures going to its common
    /// subfolder.
    fn player(names: &[&str]) -> ModelFolder {
        ModelFolder {
            path: ScopePath::new(PLAYER).unwrap(),
            files: names
                .iter()
                .map(|name| file(&format!("{PLAYER}/{name}")))
                .collect(),
            combined: Vec::new(),
            common_models: Vec::new(),
            common_texture_stems: BTreeSet::new(),
            textures: TextureHome::PlayerCommon {
                folder_name: "05 - The Chad Stormworks Player".to_owned(),
            },
        }
    }

    fn kit(config: bool, layout: Option<KitLayout>) -> KitFolder {
        KitFolder {
            path: ScopePath::new("Kits/g1").unwrap(),
            label: None,
            config: config.then(|| file("Kits/g1/config.toml")),
            colors: None,
            icon: Some(11),
            layout,
            textures: vec![KitTexture {
                stem: "kit".to_owned(),
                file: file("Kits/g1/kit.dds"),
                source: KitTextureSource::Own,
            }],
        }
    }

    /// `kind` processed for PES 21 as task 3 of export 4, over its files read from the tracer
    /// folder.
    fn run(kind: TaskKind) -> TaskBatch {
        run_for(kind, PesVersion::Pes21)
    }

    /// `run` for the target `version`, the task in no group.
    fn run_for(kind: TaskKind, version: PesVersion) -> TaskBatch {
        run_task(kind, version, None)
    }

    /// `kind` processed for `version` as task 3 of export 4, in `group`, over its files read
    /// from the tracer folder.
    fn run_task(kind: TaskKind, version: PesVersion, group: Option<TaskGroup>) -> TaskBatch {
        let files = kind
            .files()
            .into_iter()
            .map(|file| {
                let bytes = std::fs::read(tracer().join(file.source.as_str())).unwrap();
                (file.path.clone(), bytes)
            })
            .collect();
        process(kind, version, group, files)
    }

    /// `kind` processed for `version` as task 3 of export 4, in `group`, over `files`.
    fn process(
        kind: TaskKind,
        version: PesVersion,
        group: Option<TaskGroup>,
        files: TaskFiles,
    ) -> TaskBatch {
        let task = BuildTask {
            export_id: ExportId(4),
            team_id: 792,
            kind,
            charge: 0,
            group,
        };
        process_task(3, task, files, &CompileContext::new(version, 1))
    }

    fn paths(batch: &TaskBatch) -> Vec<&str> {
        batch
            .entries
            .iter()
            .map(|(path, _)| path.as_str())
            .collect()
    }

    /// The tracer's player folder, every file the lead's fixture holds.
    fn whole_player() -> ModelFolder {
        player(&[
            "boots.fmdl",
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair.skl",
            "fcl_hair_sim.fclo",
            "glove_l.fmdl",
            "glove_r.fmdl",
            "shirt.dds",
        ])
    }

    /// The (file name, directory) of every texture path of the package's model `name`.
    fn texture_directories(package: &FpkFile, name: &str) -> Vec<(String, String)> {
        let model = FmdlFile::read(package.get(name).unwrap()).unwrap();
        texture_paths(&model)
            .unwrap()
            .into_iter()
            .map(|path| (path.file_name, path.directory))
            .collect()
    }

    /// Asserts the model's own texture points at the player's common folder and the game's
    /// own at the team's, with no `000` placeholder left.
    fn assert_rewritten(directories: &[(String, String)]) {
        assert!(
            directories.contains(&("shirt.dds".to_owned(), COMMON_DIRECTORY.to_owned())),
            "{directories:?}"
        );
        assert!(
            directories.iter().any(|(_, directory)| directory
                == "/Assets/pes16/model/character/common/792/sourceimages/"),
            "{directories:?}"
        );
        assert!(
            directories
                .iter()
                .all(|(_, directory)| !directory.contains("/000/")),
            "{directories:?}"
        );
    }

    #[test]
    fn a_face_package_is_packed_once_per_slot_without_the_textures() {
        let batch = run(TaskKind::Models {
            folder: whole_player(),
            package: ModelPackage::Face,
            ids: vec![79205, 79207],
        });

        assert_eq!(batch.index, 3);
        assert!(batch.messages.is_empty() && batch.uniparam.is_none());
        assert_eq!(batch.group, None);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/face/real/79205/#Win/face.fpk",
                "Asset/model/character/face/real/79205/#Win/face.fpkd",
                "Asset/model/character/face/real/79207/#Win/face.fpk",
                "Asset/model/character/face/real/79207/#Win/face.fpkd",
            ]
        );
        assert_eq!(batch.entries[0].1, batch.entries[2].1, "one package, twice");
        let fpkd = FpkFile::read(&batch.entries[1].1).unwrap();
        assert_eq!((fpkd.kind(), fpkd.len()), (FpkKind::Fpkd, 0));

        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(
            names,
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        // The folder's own `face_diff.bin` is packed, not the template. (Its `.fclo` and
        // `.skl` are byte-identical to the templates, so they prove nothing here.)
        let own = std::fs::read(tracer().join(format!("{PLAYER}/face_diff.bin"))).unwrap();
        assert_ne!(own, templates::FACE_DIFF);
        assert_eq!(package.get("face_diff.bin").unwrap(), own);
        assert_rewritten(&texture_directories(&package, "fcl_hair.fmdl"));
    }

    /// PES 19's `body.skl`: a real skeleton that differs from the bundled PES 21 template,
    /// standing in for a custom one.
    fn other_skeleton() -> Vec<u8> {
        let bytes = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../resources/skeletons/pes19/body.skl"),
        )
        .unwrap();
        assert_ne!(bytes, templates::BODY_SKELETON);
        bytes
    }

    /// `kind`'s files read from the tracer folder, each file at a path of `replacements`
    /// replaced by the bytes given with it, processed for PES 21 in no group.
    fn run_with(kind: TaskKind, replacements: &[(&str, &[u8])]) -> TaskBatch {
        let mut files: TaskFiles = kind
            .files()
            .into_iter()
            .map(|file| {
                let bytes = std::fs::read(tracer().join(file.source.as_str())).unwrap();
                (file.path.clone(), bytes)
            })
            .collect();
        for (path, bytes) in replacements {
            files.insert(ScopePath::new(path).unwrap(), bytes.to_vec());
        }
        process(kind, PesVersion::Pes21, None, files)
    }

    #[test]
    fn a_face_package_gets_the_face_files_and_the_skeleton_its_sources_lack() {
        // The hair alone: all three injected.
        let batch = run(TaskKind::Models {
            folder: player(&["fcl_hair.fmdl"]),
            package: ModelPackage::Face,
            ids: vec![79205],
        });
        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(
            names,
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        assert_eq!(package.get("face_diff.bin").unwrap(), templates::FACE_DIFF);
        assert_eq!(
            package.get("fcl_hair_sim.fclo").unwrap(),
            templates::FCL_HAIR_SIM_FCLO
        );
        assert_eq!(
            package.get("fcl_hair_sim.skl").unwrap(),
            templates::BODY_SKELETON
        );

        // The hair with its own skeleton: that one is packed.
        let custom = other_skeleton();
        let batch = run_with(
            TaskKind::Models {
                folder: player(&["fcl_hair.fmdl", "fcl_hair.skl"]),
                package: ModelPackage::Face,
                ids: vec![79205],
            },
            &[(&format!("{PLAYER}/fcl_hair.skl"), &custom)],
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(package.get("fcl_hair_sim.skl").unwrap(), custom);

        // A face without a hair part gets only the `face_diff.bin`: no simulation, no
        // skeleton, and a skeleton named after the face model is never read.
        let batch = run(TaskKind::Models {
            folder: player_with(
                vec![
                    named(&format!("{PLAYER}/face_high.fmdl"), "fcl_hair.fmdl"),
                    named(&format!("{PLAYER}/face_high.skl"), "fcl_hair.skl"),
                ],
                Vec::new(),
            ),
            package: ModelPackage::Face,
            ids: vec![79205],
        });
        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["face_diff.bin", "face_high.fmdl"]);
        assert_eq!(package.get("face_diff.bin").unwrap(), templates::FACE_DIFF);
    }

    #[test]
    fn an_unsuffixed_model_is_a_hair_part_whose_skeleton_is_the_hair_s() {
        let custom = other_skeleton();
        let batch = run_with(
            TaskKind::Models {
                folder: player_with(
                    vec![
                        named(&format!("{PLAYER}/torso.fmdl"), "fcl_hair.fmdl"),
                        named(&format!("{PLAYER}/torso.skl"), "fcl_hair.skl"),
                    ],
                    Vec::new(),
                ),
                package: ModelPackage::Face,
                ids: vec![79205],
            },
            &[(&format!("{PLAYER}/torso.skl"), &custom)],
        );

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(
            names,
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        assert_eq!(package.get("fcl_hair_sim.skl").unwrap(), custom);
        assert_eq!(
            packed_model(&package, "fcl_hair.fmdl").meshes.len(),
            tracer_model("fcl_hair.fmdl").meshes.len()
        );
    }

    #[test]
    fn a_boots_package_holds_the_model_and_the_standard_skeleton_when_it_brings_none() {
        let batch = run(TaskKind::Models {
            folder: whole_player(),
            package: ModelPackage::Boots,
            ids: vec![3745],
        });

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/boots/k3745/#Win/boots.fpk",
                "Asset/model/character/boots/k3745/#Win/boots.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"]);
        assert_eq!(package.get("boots.skl").unwrap(), templates::BODY_SKELETON);
        assert_rewritten(&texture_directories(&package, "boots.fmdl"));
    }

    #[test]
    fn a_skeleton_named_after_the_boots_model_is_packed_in_the_standard_one_s_place() {
        let custom = other_skeleton();
        let folder = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/kit_boots.skl"), "fcl_hair.skl"),
            ],
            Vec::new(),
        );

        let batch = run_with(
            TaskKind::Models {
                folder,
                package: ModelPackage::Boots,
                ids: vec![3745, 3747],
            },
            &[(&format!("{PLAYER}/kit_boots.skl"), &custom)],
        );

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/boots/k3745/#Win/boots.fpk",
                "Asset/model/character/boots/k3745/#Win/boots.fpkd",
                "Asset/model/character/boots/k3747/#Win/boots.fpk",
                "Asset/model/character/boots/k3747/#Win/boots.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"]);
        assert_eq!(package.get("boots.skl").unwrap(), custom);
    }

    #[test]
    fn a_gloves_package_holds_both_hands_under_their_allowed_names() {
        let batch = run(TaskKind::Models {
            folder: whole_player(),
            package: ModelPackage::Gloves,
            ids: vec![3745],
        });

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/glove/g3745/#Win/glove.fpk",
                "Asset/model/character/glove/g3745/#Win/glove.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["glove_l.fmdl", "glove_r.fmdl"]);
        let fpkd = FpkFile::read(&batch.entries[1].1).unwrap();
        assert_eq!((fpkd.kind(), fpkd.len()), (FpkKind::Fpkd, 0));
    }

    #[test]
    fn a_shared_gloves_folder_s_package_and_textures_go_under_its_shared_id() {
        // The tracer's boots model stands in for a left glove: a real FMDL naming `shirt.dds`.
        let folder_path = ScopePath::new("Gloves/Grip").unwrap();
        let glove = ScopePath::new("Gloves/Grip/glove_l.fmdl").unwrap();
        let shirt = ScopePath::new("Gloves/Grip/shirt.dds").unwrap();
        let folder = ModelFolder {
            path: folder_path,
            files: vec![
                FileDescriptor {
                    kind: aesthetics_export::classify(glove.name()),
                    path: glove,
                    ..file(&format!("{PLAYER}/boots.fmdl"))
                },
                FileDescriptor {
                    kind: aesthetics_export::classify(shirt.name()),
                    path: shirt,
                    ..file(&format!("{PLAYER}/shirt.dds"))
                },
            ],
            combined: Vec::new(),
            common_models: Vec::new(),
            common_texture_stems: BTreeSet::new(),
            textures: TextureHome::SharedOutput {
                package: ModelPackage::Gloves,
                id: 644,
            },
        };

        let package = run(TaskKind::Models {
            folder: folder.clone(),
            package: ModelPackage::Gloves,
            ids: vec![644],
        });
        let textures = run(TaskKind::Textures { folder });

        assert!(package.messages.is_empty(), "{:?}", package.messages);
        assert_eq!(
            paths(&package),
            [
                "Asset/model/character/glove/g0644/#Win/glove.fpk",
                "Asset/model/character/glove/g0644/#Win/glove.fpkd",
            ]
        );
        let fpk = FpkFile::read(&package.entries[0].1).unwrap();
        let names: Vec<&str> = fpk.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["glove_l.fmdl"]);
        let directories = texture_directories(&fpk, "glove_l.fmdl");
        assert!(
            directories.contains(&(
                "shirt.dds".to_owned(),
                "/Assets/pes16/model/character/glove/g0644/".to_owned()
            )),
            "{directories:?}"
        );
        assert!(textures.messages.is_empty(), "{:?}", textures.messages);
        assert_eq!(
            paths(&textures),
            ["Asset/model/character/glove/g0644/#windx11/shirt.ftex"]
        );
    }

    #[test]
    fn a_folder_s_textures_are_converted_once_into_its_common_folder_for_its_packages() {
        let batch = run_task(
            TaskKind::Textures {
                folder: whole_player(),
            },
            PesVersion::Pes21,
            Some(TaskGroup {
                tasks: 0..4,
                packages: ModelPackage::ALL.to_vec(),
                charge: 0,
            }),
        );

        assert!(batch.messages.is_empty() && batch.uniparam.is_none());
        assert_eq!(
            batch.group,
            Some(0..4),
            "the batch carries its task's group"
        );
        assert_eq!(paths(&batch), [COMMON]);
        let dds = std::fs::read(tracer().join(format!("{PLAYER}/shirt.dds"))).unwrap();
        assert_eq!(
            batch.entries[0].1,
            ftex::dds_to_ftex(&dds, ftex::ColorSpace::Normal).unwrap()
        );
    }

    #[test]
    fn the_conversion_cache_engages_for_at_most_two_exports() {
        assert_eq!(cache_policy(0), CachePolicy::Use);
        assert_eq!(cache_policy(2), CachePolicy::Use);
        assert_eq!(cache_policy(3), CachePolicy::Bypass);
        assert_eq!(
            CompileContext::new(PesVersion::Pes21, 48).cache,
            CachePolicy::Bypass
        );
    }

    #[test]
    fn a_portrait_is_emitted_as_it_is_under_the_version_s_name() {
        let file = file(&format!("{PLAYER}/portrait.dds"));
        let source = std::fs::read(tracer().join(file.source.as_str())).unwrap();
        assert!(source.starts_with(b"DDS "), "the fixture is a DDS");
        for (version, path) in [
            (PesVersion::Pes21, "common/render/symbol/player/79205.dds"),
            (
                PesVersion::Pes18,
                "common/render/symbol/player/player_79205.dds",
            ),
        ] {
            let batch = run_for(
                TaskKind::Portrait {
                    player_id: 79205,
                    file: file.clone(),
                },
                version,
            );

            assert!(batch.messages.is_empty() && batch.uniparam.is_none());
            assert_eq!(paths(&batch), [path], "{version}");
            assert_eq!(batch.entries[0].1, source, "{version}");
        }
    }

    #[test]
    fn a_kit_commits_its_texture_and_its_config_as_entry_and_bin_entry() {
        let batch = run(TaskKind::Kit {
            slot: KitSlot::G1,
            kit: kit(true, Some(KitLayout::Fox)),
        });

        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/uniform/texture/#windx11/u0792g1.ftex",
                "common/character0/model/character/uniform/team/792/792_DEF_GK1st_realUni.bin",
            ]
        );
        let (name, config) = batch.uniparam.as_ref().unwrap();
        assert_eq!(name, "792_DEF_GK1st_realUni.bin");
        assert_eq!(config, &batch.entries[1].1);
    }

    #[test]
    fn a_kit_without_config_toml_is_encoded_from_the_template() {
        let batch = run(TaskKind::Kit {
            slot: KitSlot::G1,
            kit: kit(false, None),
        });

        let presence = TexturePresence {
            kit: true,
            ..TexturePresence::default()
        };
        let names = texture_names(792, KitSlot::G1, presence);
        let template = KitConfig::template().encode_with_names(PesVersion::Pes21, &names);
        assert_eq!(batch.uniparam.unwrap().1, template);
    }

    /// The export file at `path`, described as the structure pass would, whose bytes `run`
    /// reads from the tracer's player file `tracer_name`.
    fn named(path: &str, tracer_name: &str) -> FileDescriptor {
        let path = ScopePath::new(path).unwrap();
        FileDescriptor {
            kind: aesthetics_export::classify(path.name()),
            path,
            ..file(&format!("{PLAYER}/{tracer_name}"))
        }
    }

    /// The tracer's player folder holding `files` and combining the shared folders
    /// `combined`, its textures going to its common subfolder.
    fn player_with(files: Vec<FileDescriptor>, combined: Vec<CombinedFolder>) -> ModelFolder {
        ModelFolder {
            combined,
            files,
            ..player(&[])
        }
    }

    /// The shared folder at `path` (`Boots/Crocs`) holding `files`, combined into `package`.
    fn shared(package: ModelPackage, path: &str, files: Vec<FileDescriptor>) -> CombinedFolder {
        let path = ScopePath::new(path).unwrap();
        CombinedFolder {
            package,
            folder: SharedModelFolder {
                folder_name: path.name().to_owned(),
                path,
                files,
            },
        }
    }

    /// The boots task over the player folder `folder` under id 3745.
    fn boots(folder: ModelFolder) -> TaskKind {
        TaskKind::Models {
            folder,
            package: ModelPackage::Boots,
            ids: vec![3745],
        }
    }

    /// The tracer's model `name`, decoded.
    fn tracer_model(name: &str) -> Model {
        let bytes = std::fs::read(tracer().join(format!("{PLAYER}/{name}"))).unwrap();
        Model::from_file(&FmdlFile::read(&bytes).unwrap()).unwrap()
    }

    /// The package's model `name`, decoded.
    fn packed_model(package: &FpkFile, name: &str) -> Model {
        Model::from_file(&FmdlFile::read(package.get(name).unwrap()).unwrap()).unwrap()
    }

    /// The batch's one message as (code, severity, disposition, context), asserting it is on
    /// the tracer's player folder.
    fn one_message(batch: &TaskBatch) -> (&str, Severity, Disposition, &[(String, String)]) {
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(4),
                path: ScopePath::new(PLAYER).unwrap(),
            }
        );
        (
            &message.code.code,
            message.severity,
            message.disposition,
            &message.context,
        )
    }

    #[test]
    fn two_boots_parts_merge_into_one_model_with_the_standard_skeleton_and_the_merge_is_noted() {
        let folder = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/a_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/shirt.dds"), "shirt.dds"),
            ],
            Vec::new(),
        );

        let batch = run(boots(folder));

        assert_eq!(
            one_message(&batch),
            (
                "fmdl_merged",
                Severity::Info,
                Disposition::Keep,
                &[("model".to_owned(), "boots.fmdl".to_owned())][..]
            )
        );
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/boots/k3745/#Win/boots.fpk",
                "Asset/model/character/boots/k3745/#Win/boots.fpkd",
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"]);
        assert_eq!(package.get("boots.skl").unwrap(), templates::BODY_SKELETON);
        let merged = packed_model(&package, "boots.fmdl");
        let part = tracer_model("boots.fmdl");
        assert_eq!(merged.meshes.len(), 2 * part.meshes.len());
        assert_eq!(
            merged.bones.len(),
            part.bones.len(),
            "one skeleton, unioned by name"
        );
        // The merged model's own texture points at the player's common folder.
        assert_rewritten(&texture_directories(&package, "boots.fmdl"));
    }

    #[test]
    fn parts_merge_in_case_folded_name_order_whatever_the_folder_s_order() {
        // `B_boots` is the tracer's boots with its first material renamed, so the merged
        // model's first material says which part came first: `a_boots` folds before
        // `B_boots`, though byte order would put `B` first.
        let mut renamed = tracer_model("boots.fmdl");
        renamed.materials[0].name = "kit_b".to_owned();
        let renamed = renamed.to_file().unwrap().write();
        let b_boots = ScopePath::new(&format!("{PLAYER}/B_boots.fmdl")).unwrap();
        let folder = player_with(
            vec![
                FileDescriptor {
                    kind: aesthetics_export::classify(b_boots.name()),
                    path: b_boots.clone(),
                    ..file(&format!("{PLAYER}/boots.fmdl"))
                },
                named(&format!("{PLAYER}/a_boots.fmdl"), "boots.fmdl"),
            ],
            Vec::new(),
        );
        let kind = boots(folder);
        let mut files: TaskFiles = kind
            .files()
            .into_iter()
            .map(|file| {
                let bytes = std::fs::read(tracer().join(file.source.as_str())).unwrap();
                (file.path.clone(), bytes)
            })
            .collect();
        files.insert(b_boots, renamed);

        let batch = process(kind, PesVersion::Pes21, None, files);

        assert_eq!(one_message(&batch).0, "fmdl_merged");
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let merged = packed_model(&package, "boots.fmdl");
        let names: Vec<&str> = merged
            .materials
            .iter()
            .map(|material| material.name.as_str())
            .collect();
        assert_eq!(names, ["kit", "shirt", "shirt antiblur", "kit_b"]);
    }

    #[test]
    fn parts_with_one_skeleton_pack_it_and_a_part_without_one_beside_one_with_is_a_conflict() {
        // One custom skeleton under both parts' names.
        let custom = other_skeleton();
        let kit_boots_skl = format!("{PLAYER}/kit_boots.skl");
        let a_boots_skl = format!("{PLAYER}/a_boots.skl");
        let both = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&kit_boots_skl, "fcl_hair.skl"),
                named(&format!("{PLAYER}/a_boots.fmdl"), "boots.fmdl"),
                named(&a_boots_skl, "fcl_hair.skl"),
            ],
            Vec::new(),
        );
        let batch = run_with(
            boots(both),
            &[(&kit_boots_skl, &custom), (&a_boots_skl, &custom)],
        );
        assert_eq!(one_message(&batch).0, "fmdl_merged");
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(package.get("boots.skl").unwrap(), custom);

        let one = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/kit_boots.skl"), "fcl_hair.skl"),
                named(&format!("{PLAYER}/a_boots.fmdl"), "boots.fmdl"),
            ],
            Vec::new(),
        );
        let batch = run(boots(one));
        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        assert_eq!(
            one_message(&batch),
            (
                "skl_merge_conflict",
                Severity::Error,
                Disposition::DropFolder,
                &[("skeleton".to_owned(), "differs".to_owned())][..]
            )
        );
    }

    #[test]
    fn a_boots_subfolder_s_skeleton_pairs_with_the_model_beside_it_and_a_root_part_keeps_its_own() {
        let custom = other_skeleton();
        let sub_skl = format!("{PLAYER}/boots/kit_boots.skl");
        let alone = player_with(
            vec![
                named(&format!("{PLAYER}/boots/kit_boots.fmdl"), "boots.fmdl"),
                named(&sub_skl, "fcl_hair.skl"),
            ],
            Vec::new(),
        );
        let batch = run_with(boots(alone), &[(&sub_skl, &custom)]);
        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"]);
        assert_eq!(package.get("boots.skl").unwrap(), custom);

        // A root part of the same name beside it: each pairs the skeleton in its own
        // directory, and the two, one file, merge without a conflict.
        let root_skl = format!("{PLAYER}/kit_boots.skl");
        let both = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&root_skl, "fcl_hair.skl"),
                named(&format!("{PLAYER}/boots/kit_boots.fmdl"), "boots.fmdl"),
                named(&sub_skl, "fcl_hair.skl"),
            ],
            Vec::new(),
        );
        let batch = run_with(boots(both), &[(&root_skl, &custom), (&sub_skl, &custom)]);
        assert_eq!(one_message(&batch).0, "fmdl_merged");
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(package.get("boots.skl").unwrap(), custom);
        assert_eq!(
            packed_model(&package, "boots.fmdl").meshes.len(),
            2 * tracer_model("boots.fmdl").meshes.len()
        );
    }

    #[test]
    fn parts_disagreeing_on_a_bone_or_a_material_fail_the_package_with_the_conflict_s_code() {
        // The tracer's left glove as a boots part: its `sk_forearm_l` differs from the boots'.
        let bone = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/a_boots.fmdl"), "glove_l.fmdl"),
            ],
            Vec::new(),
        );
        let batch = run(boots(bone));
        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        assert_eq!(
            one_message(&batch),
            (
                "skl_merge_conflict",
                Severity::Error,
                Disposition::DropFolder,
                &[("bone".to_owned(), "sk_forearm_l".to_owned())][..]
            )
        );

        // The tracer's hair as a boots part: its `shirt` material names its texture in another
        // directory than the boots', and no `shirt.dds` of the player's points both at one.
        let material = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/a_boots.fmdl"), "fcl_hair.fmdl"),
            ],
            Vec::new(),
        );
        let batch = run(boots(material));
        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        assert_eq!(
            one_message(&batch),
            (
                "merge_material_conflict",
                Severity::Error,
                Disposition::DropFolder,
                &[("material".to_owned(), "shirt".to_owned())][..]
            )
        );
    }

    #[test]
    fn a_combined_shared_folder_s_models_are_parts_and_its_textures_go_to_the_player_s_common() {
        let folder = player_with(
            vec![named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl")],
            vec![shared(
                ModelPackage::Boots,
                "Boots/Crocs",
                vec![
                    named("Boots/Crocs/boots.fmdl", "boots.fmdl"),
                    named("Boots/Crocs/shirt.dds", "shirt.dds"),
                ],
            )],
        );

        let package = run(boots(folder.clone()));
        let textures = run(TaskKind::Textures { folder });

        assert_eq!(one_message(&package).0, "fmdl_merged");
        let fpk = FpkFile::read(&package.entries[0].1).unwrap();
        assert_eq!(
            packed_model(&fpk, "boots.fmdl").meshes.len(),
            2 * tracer_model("boots.fmdl").meshes.len()
        );
        // The shared folder's texture is the player's now: the merged model points at the
        // player's common folder, where the textures task puts it.
        assert_rewritten(&texture_directories(&fpk, "boots.fmdl"));
        assert!(textures.messages.is_empty(), "{:?}", textures.messages);
        assert_eq!(paths(&textures), [COMMON]);
    }

    #[test]
    fn a_combined_gloves_folder_bringing_the_other_hand_merges_nothing() {
        let folder = player_with(
            vec![named(&format!("{PLAYER}/glove_l.fmdl"), "glove_l.fmdl")],
            vec![shared(
                ModelPackage::Gloves,
                "Gloves/Grip",
                vec![named("Gloves/Grip/glove_r.fmdl", "glove_r.fmdl")],
            )],
        );

        let batch = run(TaskKind::Models {
            folder,
            package: ModelPackage::Gloves,
            ids: vec![3745],
        });

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["glove_l.fmdl", "glove_r.fmdl"]);
        assert_eq!(
            packed_model(&package, "glove_r.fmdl").meshes.len(),
            tracer_model("glove_r.fmdl").meshes.len(),
            "a single part is packed as it is"
        );
    }

    /// The tracer's hair model as a Common model of its own: its materials renamed with a
    /// `_common` tail and its `shirt.dds` renamed `cloth.dds`, so it merges with the hair itself
    /// with no material in common.
    fn common_hair_model() -> Vec<u8> {
        let mut model = tracer_model("fcl_hair.fmdl");
        for material in &mut model.materials {
            material.name.push_str("_common");
            for (_, texture) in &mut material.textures {
                if texture.file_name == "shirt.dds" {
                    texture.file_name = "cloth.dds".to_owned();
                }
            }
        }
        model.to_file().unwrap().write()
    }

    /// The tracer's player folder holding `torso.fmdl` (the hair model), `shirt.dds` and the
    /// link `legs.fmdl.common` resolved to `Common/legs.fmdl`, the Common textures' stems being
    /// `common_stems`; with `skeletons`, `torso.skl` beside the model and `Common/legs.skl` with
    /// the Common one. The Common files' bytes come from the tracer's hair model and skeleton
    /// unless `run_with` replaces them.
    fn player_linking_common(common_stems: &[&str], skeletons: bool) -> ModelFolder {
        let link = ScopePath::new(&format!("{PLAYER}/legs.fmdl.common")).unwrap();
        let mut files = vec![
            named(&format!("{PLAYER}/torso.fmdl"), "fcl_hair.fmdl"),
            named(&format!("{PLAYER}/shirt.dds"), "shirt.dds"),
            FileDescriptor {
                kind: aesthetics_export::classify(link.name()),
                path: link.clone(),
                size: 0,
                ..file(&format!("{PLAYER}/face_diff.bin"))
            },
        ];
        if skeletons {
            files.push(named(&format!("{PLAYER}/torso.skl"), "fcl_hair.skl"));
        }
        ModelFolder {
            common_models: vec![CommonModel {
                link,
                model: named("Common/legs.fmdl", "fcl_hair.fmdl"),
                skeleton: skeletons.then(|| named("Common/legs.skl", "fcl_hair.skl")),
            }],
            common_texture_stems: common_stems.iter().map(|stem| (*stem).to_owned()).collect(),
            ..player_with(files, Vec::new())
        }
    }

    #[test]
    fn a_common_part_merges_in_with_its_skeleton_and_its_paths_name_the_team_s_common_output() {
        let custom = other_skeleton();
        let folder = player_linking_common(&["cloth"], true);
        let kind = face(folder.clone());
        let read: Vec<&str> = kind.files().iter().map(|file| file.path.as_str()).collect();
        assert_eq!(
            read,
            [
                "Players/05 - The Chad Stormworks Player/torso.fmdl",
                "Common/legs.fmdl",
                "Common/legs.skl",
                "Players/05 - The Chad Stormworks Player/torso.skl",
            ],
            "the Common files in the link's place; the link itself is never read"
        );

        // Both parts bring the one custom skeleton: the Common part's pairs with it by their
        // shared `Common/legs` stem.
        let torso_skl = format!("{PLAYER}/torso.skl");
        let batch = run_with(
            face(folder),
            &[
                ("Common/legs.fmdl", &common_hair_model()),
                ("Common/legs.skl", &custom),
                (&torso_skl, &custom),
            ],
        );

        assert_eq!(
            one_message(&batch),
            (
                "fmdl_merged",
                Severity::Info,
                Disposition::Keep,
                &[("model".to_owned(), "fcl_hair.fmdl".to_owned())][..]
            )
        );
        assert_eq!(
            package_names(&batch),
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(package.get("fcl_hair_sim.skl").unwrap(), custom);
        assert_eq!(
            packed_model(&package, "fcl_hair.fmdl").meshes.len(),
            2 * tracer_model("fcl_hair.fmdl").meshes.len()
        );
        // The player's own `shirt` points at its common subfolder, the Common model's `cloth`
        // at the team's Common output, and the game's own textures at the team's.
        let directories = texture_directories(&package, "fcl_hair.fmdl");
        assert_rewritten(&directories);
        assert!(
            directories.contains(&(
                "cloth.dds".to_owned(),
                "/Assets/pes16/model/character/common/792/sourceimages/".to_owned()
            )),
            "{directories:?}"
        );
        assert!(
            !directories.contains(&("cloth.dds".to_owned(), COMMON_DIRECTORY.to_owned())),
            "{directories:?}"
        );
    }

    #[test]
    fn a_material_two_parts_define_over_textures_in_different_places_is_a_conflict() {
        // The same hair model as the local part and the Common part, both naming `shirt.dds`:
        // the player's copy is the player's own, Common's copy is the team's, so the two
        // `shirt` materials no longer agree. (No skeleton with the Common part: the local part
        // brings none, and a mix would be the skeleton's conflict first.)
        let linking = |common_stems: &[&str]| player_linking_common(common_stems, false);
        let batch = run(face(linking(&["shirt"])));
        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        assert_eq!(
            one_message(&batch),
            (
                "merge_material_conflict",
                Severity::Error,
                Disposition::DropFolder,
                &[("material".to_owned(), "shirt".to_owned())][..]
            )
        );

        // With no `shirt` in Common, the Common part's `shirt` is one of the game's own and
        // still differs from the player's; renamed materials merge.
        let batch = run(face(linking(&[])));
        assert_eq!(one_message(&batch).0, "merge_material_conflict");
        let batch = run_with(
            face(linking(&[])),
            &[("Common/legs.fmdl", &common_hair_model())],
        );
        assert_eq!(one_message(&batch).0, "fmdl_merged");
    }

    #[test]
    fn a_common_part_goes_only_into_the_package_its_link_s_role_names() {
        // A local hair model beside a Common boots link: the Common boots are the boots
        // package's part and nothing of the face's.
        let link = ScopePath::new(&format!("{PLAYER}/kit_boots.fmdl.common")).unwrap();
        let folder = ModelFolder {
            common_models: vec![CommonModel {
                link: link.clone(),
                model: named("Common/kit_boots.fmdl", "boots.fmdl"),
                skeleton: None,
            }],
            ..player_with(
                vec![
                    named(&format!("{PLAYER}/fcl_hair.fmdl"), "fcl_hair.fmdl"),
                    FileDescriptor {
                        kind: aesthetics_export::classify(link.name()),
                        source: link.clone(),
                        path: link,
                        size: 0,
                    },
                ],
                Vec::new(),
            )
        };

        // The face task is handed the Common boots' bytes too, which it never lists: a face
        // package taking them as a part then fails on its contents, not on the coordinator's
        // "every file listed" invariant.
        let face_batch = run_with(
            face(folder.clone()),
            &[("Common/kit_boots.fmdl", &tracer_player_file("boots.fmdl"))],
        );
        let boots_batch = run(boots(folder));

        assert!(face_batch.messages.is_empty(), "{:?}", face_batch.messages);
        assert_eq!(
            package_names(&face_batch),
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        assert!(
            boots_batch.messages.is_empty(),
            "{:?}",
            boots_batch.messages
        );
        assert_eq!(package_names(&boots_batch), ["boots.fmdl", "boots.skl"]);
        let package = FpkFile::read(&boots_batch.entries[0].1).unwrap();
        assert_eq!(
            packed_model(&package, "boots.fmdl").meshes.len(),
            tracer_model("boots.fmdl").meshes.len()
        );
    }

    #[test]
    fn the_common_textures_go_once_into_the_team_s_common_output_or_fail_as_one() {
        let folder = ScopePath::new("Common").unwrap();
        let textures = vec![
            named("Common/Cloth.dds", "shirt.dds"),
            named("Common/hair.ftex", "shirt.dds"),
        ];
        let converted =
            ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap();

        let batch = run_with(
            TaskKind::CommonTextures {
                folder: folder.clone(),
                textures: textures.clone(),
            },
            &[("Common/hair.ftex", &converted)],
        );

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(batch.group, None);
        assert_eq!(
            paths(&batch),
            [
                "Asset/model/character/common/792/sourceimages/#windx11/Cloth.ftex",
                "Asset/model/character/common/792/sourceimages/#windx11/hair.ftex",
            ],
            "each stem as spelled"
        );
        assert_eq!(batch.entries[0].1, converted);
        assert_eq!(batch.entries[1].1, converted);

        // One texture that cannot convert fails the task, on the Common folder.
        let batch = run_with(
            TaskKind::CommonTextures { folder, textures },
            &[("Common/Cloth.dds", b"not a DDS")],
        );
        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(message.code.code, "folder_pack_failed");
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(4),
                path: ScopePath::new("Common").unwrap(),
            }
        );
        let [(key, error)] = message.context.as_slice() else {
            panic!("{:?}", message.context);
        };
        assert_eq!(key, "error");
        assert!(error.starts_with("Cloth.dds: cannot convert"), "{error}");
    }

    #[test]
    fn a_common_texture_with_a_finding_is_left_out_alone_and_the_rest_emitted() {
        let folder = ScopePath::new("Common").unwrap();
        let textures = vec![
            named("Common/tiny.png", "shirt.dds"),
            named("Common/hair.dds", "shirt.dds"),
        ];
        let tiny = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/textures/tiny.png"),
        )
        .unwrap();

        let batch = run_with(
            TaskKind::CommonTextures { folder, textures },
            &[("Common/tiny.png", &tiny)],
        );

        assert_eq!(
            paths(&batch),
            ["Asset/model/character/common/792/sourceimages/#windx11/hair.ftex"]
        );
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(message.code.code, "texture_too_small");
        assert_eq!(
            (message.severity, message.disposition),
            (Severity::Error, Disposition::DropFile)
        );
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(4),
                path: ScopePath::new("Common").unwrap(),
            }
        );
        assert_eq!(
            message.context,
            [("file".to_owned(), "tiny.png".to_owned())]
        );
    }

    #[test]
    fn a_portrait_with_a_finding_fails_its_task_dropping_the_file() {
        let file = file(&format!("{PLAYER}/portrait.dds"));
        let odd = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/textures/odd.png"),
        )
        .unwrap();
        // PNG bytes under the `.dds` name: the signature check comes first.
        let batch = run_with(
            TaskKind::Portrait {
                player_id: 79205,
                file: file.clone(),
            },
            &[(&format!("{PLAYER}/portrait.dds"), &odd)],
        );

        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(message.code.code, "texture_type_mismatch");
        assert_eq!(
            (message.severity, message.disposition),
            (Severity::Error, Disposition::DropFile)
        );
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(4),
                path: file.path,
            }
        );
        assert_eq!(
            message.context,
            [("file".to_owned(), "portrait.dds".to_owned())]
        );
    }

    /// The face task over the player folder `folder` under id 79205.
    fn face(folder: ModelFolder) -> TaskKind {
        TaskKind::Models {
            folder,
            package: ModelPackage::Face,
            ids: vec![79205],
        }
    }

    /// The entry names of the batch's first package.
    fn package_names(batch: &TaskBatch) -> Vec<String> {
        FpkFile::read(&batch.entries[0].1)
            .unwrap()
            .entries()
            .map(|(name, _)| name.to_owned())
            .collect()
    }

    #[test]
    fn two_face_parts_under_one_name_merge_and_hair_parts_bringing_one_skeleton_pack_it() {
        // The tracer's hair model stands in for two `face_high` parts and two hair parts.
        let faces = player_with(
            vec![
                named(&format!("{PLAYER}/face_diff.bin"), "face_diff.bin"),
                named(&format!("{PLAYER}/face_high.fmdl"), "fcl_hair.fmdl"),
                named(&format!("{PLAYER}/x_face_high.fmdl"), "fcl_hair.fmdl"),
            ],
            Vec::new(),
        );
        let batch = run(face(faces));
        assert_eq!(
            one_message(&batch),
            (
                "fmdl_merged",
                Severity::Info,
                Disposition::Keep,
                &[("model".to_owned(), "face_high.fmdl".to_owned())][..]
            )
        );
        assert_eq!(package_names(&batch), ["face_diff.bin", "face_high.fmdl"]);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(
            packed_model(&package, "face_high.fmdl").meshes.len(),
            2 * tracer_model("fcl_hair.fmdl").meshes.len()
        );

        let custom = std::fs::read(tracer().join(format!("{PLAYER}/fcl_hair.skl"))).unwrap();
        let hairs = player_with(
            vec![
                named(&format!("{PLAYER}/face_diff.bin"), "face_diff.bin"),
                named(&format!("{PLAYER}/fcl_hair.fmdl"), "fcl_hair.fmdl"),
                named(&format!("{PLAYER}/fcl_hair.skl"), "fcl_hair.skl"),
                named(&format!("{PLAYER}/fcl_hair_sim.fclo"), "fcl_hair_sim.fclo"),
                named(&format!("{PLAYER}/x_fcl_hair.fmdl"), "fcl_hair.fmdl"),
                named(&format!("{PLAYER}/x_fcl_hair.skl"), "fcl_hair.skl"),
            ],
            Vec::new(),
        );
        let batch = run(face(hairs));
        assert_eq!(one_message(&batch).0, "fmdl_merged");
        assert_eq!(
            package_names(&batch),
            [
                "face_diff.bin",
                "fcl_hair.fmdl",
                "fcl_hair_sim.fclo",
                "fcl_hair_sim.skl"
            ]
        );
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(package.get("fcl_hair_sim.skl").unwrap(), custom);

        // A hair part without a skeleton beside one with is a conflict.
        let mixed = player_with(
            vec![
                named(&format!("{PLAYER}/face_diff.bin"), "face_diff.bin"),
                named(&format!("{PLAYER}/fcl_hair.fmdl"), "fcl_hair.fmdl"),
                named(&format!("{PLAYER}/fcl_hair.skl"), "fcl_hair.skl"),
                named(&format!("{PLAYER}/fcl_hair_sim.fclo"), "fcl_hair_sim.fclo"),
                named(&format!("{PLAYER}/x_fcl_hair.fmdl"), "fcl_hair.fmdl"),
            ],
            Vec::new(),
        );
        let batch = run(face(mixed));
        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        assert_eq!(
            one_message(&batch),
            (
                "skl_merge_conflict",
                Severity::Error,
                Disposition::DropFolder,
                &[("skeleton".to_owned(), "differs".to_owned())][..]
            )
        );
    }

    #[test]
    fn a_combined_face_folder_is_the_face_and_the_player_s_own_face_files_win_over_its() {
        // The shared face alone: the player folder holds nothing of the face but the link.
        let shared_face = shared(
            ModelPackage::Face,
            "Faces/Round",
            vec![
                named("Faces/Round/face_diff.bin", "face_diff.bin"),
                named("Faces/Round/hair_high.fmdl", "boots.fmdl"),
                named("Faces/Round/shirt.dds", "shirt.dds"),
            ],
        );
        let alone = player_with(Vec::new(), vec![shared_face.clone()]);
        let batch = run(face(alone.clone()));
        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(package_names(&batch), ["face_diff.bin", "hair_high.fmdl"]);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        // The shared folder's texture is the player's now.
        assert_rewritten(&texture_directories(&package, "hair_high.fmdl"));
        let textures = run(TaskKind::Textures { folder: alone });
        assert_eq!(paths(&textures), [COMMON]);

        // Both sources holding `face_diff.bin`: the player's own is packed, the shared one
        // never read (the simulation file's bytes stand in for a differing shared copy).
        let own = tracer_player_file("face_diff.bin");
        let both = player_with(
            vec![named(&format!("{PLAYER}/face_diff.bin"), "face_diff.bin")],
            vec![shared(
                ModelPackage::Face,
                "Faces/Round",
                vec![
                    named("Faces/Round/face_diff.bin", "fcl_hair_sim.fclo"),
                    named("Faces/Round/hair_high.fmdl", "boots.fmdl"),
                ],
            )],
        );
        let kind = face(both);
        let read: Vec<&str> = kind.files().iter().map(|file| file.path.as_str()).collect();
        assert_eq!(
            read,
            [
                "Players/05 - The Chad Stormworks Player/face_diff.bin",
                "Faces/Round/hair_high.fmdl"
            ]
        );
        let batch = run(kind);
        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        let package = FpkFile::read(&batch.entries[0].1).unwrap();
        assert_eq!(package.get("face_diff.bin").unwrap(), own);
    }

    /// The tracer's player folder's `face_diff.bin` bytes.
    fn tracer_player_file(name: &str) -> Vec<u8> {
        std::fs::read(tracer().join(format!("{PLAYER}/{name}"))).unwrap()
    }

    /// The export file at `path`, described as the structure pass would, whose bytes `run`
    /// reads from the tracer's `kit.dds`: a real DDS whose bytes differ from `shirt.dds`.
    fn other_dds(path: &str) -> FileDescriptor {
        let path = ScopePath::new(path).unwrap();
        FileDescriptor {
            kind: aesthetics_export::classify(path.name()),
            path,
            ..file("Kits/g1/kit.dds")
        }
    }

    /// The textures task over `folder` as task 3 of a group at 0..4 holding its face, boots
    /// and gloves tasks.
    fn textures_in_group(folder: ModelFolder) -> TaskBatch {
        run_task(
            TaskKind::Textures { folder },
            PesVersion::Pes21,
            Some(TaskGroup {
                tasks: 0..4,
                packages: ModelPackage::ALL.to_vec(),
                charge: 0,
            }),
        )
    }

    /// The (code, context) of each of the batch's messages.
    fn message_codes(batch: &TaskBatch) -> Vec<(&str, &[(String, String)])> {
        batch
            .messages
            .iter()
            .map(|message| (message.code.code.as_ref(), message.context.as_slice()))
            .collect()
    }

    #[test]
    fn a_stem_two_sources_hold_with_the_same_bytes_is_one_texture_and_no_finding() {
        // The player's own `shirt.dds` and the combined boots folder's `Shirt.dds`: one
        // stem, compared case-folded, written once under the player's spelling.
        let folder = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/shirt.dds"), "shirt.dds"),
            ],
            vec![shared(
                ModelPackage::Boots,
                "Boots/Crocs",
                vec![
                    named("Boots/Crocs/boots.fmdl", "boots.fmdl"),
                    named("Boots/Crocs/Shirt.dds", "shirt.dds"),
                ],
            )],
        );

        let batch = textures_in_group(folder);

        assert!(batch.messages.is_empty(), "{:?}", batch.messages);
        assert_eq!(batch.skipped, Vec::<usize>::new());
        assert_eq!(paths(&batch), [COMMON]);
    }

    #[test]
    fn a_stem_two_packages_hold_with_different_bytes_drops_the_lower_package_with_its_textures() {
        let folder = player_with(
            vec![
                named(&format!("{PLAYER}/face_diff.bin"), "face_diff.bin"),
                named(&format!("{PLAYER}/fcl_hair.fmdl"), "fcl_hair.fmdl"),
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/shirt.dds"), "shirt.dds"),
            ],
            vec![shared(
                ModelPackage::Boots,
                "Boots/Crocs",
                vec![
                    named("Boots/Crocs/boots.fmdl", "boots.fmdl"),
                    other_dds("Boots/Crocs/shirt.dds"),
                    named("Boots/Crocs/sole.dds", "shirt.dds"),
                ],
            )],
        );

        let batch = textures_in_group(folder);

        assert_eq!(
            message_codes(&batch),
            [(
                "shared_texture_conflict",
                &[
                    ("texture".to_owned(), "shirt".to_owned()),
                    ("dropped".to_owned(), "boots".to_owned())
                ][..]
            )]
        );
        let message = &batch.messages[0];
        assert_eq!(
            (message.severity, message.disposition),
            (Severity::Error, Disposition::DropFolder)
        );
        // The face wins: its `shirt` is written, and the boots' `sole`, which only the
        // dropped folder holds, is not. The boots task sits at the group's second position.
        assert_eq!(paths(&batch), [COMMON]);
        assert_eq!(
            batch.entries[0].1,
            ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap()
        );
        assert_eq!(batch.skipped, [1]);
    }

    #[test]
    fn a_stem_a_combined_boots_and_gloves_folder_hold_with_different_bytes_drops_the_gloves() {
        let folder = player_with(
            vec![
                named(&format!("{PLAYER}/kit_boots.fmdl"), "boots.fmdl"),
                named(&format!("{PLAYER}/glove_l.fmdl"), "glove_l.fmdl"),
            ],
            vec![
                shared(
                    ModelPackage::Gloves,
                    "Gloves/Grip",
                    vec![
                        named("Gloves/Grip/glove_r.fmdl", "glove_r.fmdl"),
                        other_dds("Gloves/Grip/shirt.dds"),
                        named("Gloves/Grip/grip.dds", "shirt.dds"),
                    ],
                ),
                shared(
                    ModelPackage::Boots,
                    "Boots/Crocs",
                    vec![
                        named("Boots/Crocs/boots.fmdl", "boots.fmdl"),
                        named("Boots/Crocs/shirt.dds", "shirt.dds"),
                    ],
                ),
            ],
        );

        let batch = run_task(
            TaskKind::Textures { folder },
            PesVersion::Pes21,
            Some(TaskGroup {
                tasks: 4..7,
                packages: vec![ModelPackage::Boots, ModelPackage::Gloves],
                charge: 0,
            }),
        );

        assert_eq!(
            message_codes(&batch),
            [(
                "shared_texture_conflict",
                &[
                    ("texture".to_owned(), "shirt".to_owned()),
                    ("dropped".to_owned(), "gloves".to_owned())
                ][..]
            )]
        );
        assert_eq!(paths(&batch), [COMMON], "no grip: only the gloves hold it");
        assert_eq!(batch.skipped, [5]);
    }

    #[test]
    fn a_stem_two_sources_of_one_package_hold_with_different_bytes_fails_the_textures() {
        let folder = player_with(
            vec![
                named(&format!("{PLAYER}/face_diff.bin"), "face_diff.bin"),
                named(&format!("{PLAYER}/fcl_hair.fmdl"), "fcl_hair.fmdl"),
                named(&format!("{PLAYER}/shirt.dds"), "shirt.dds"),
            ],
            vec![shared(
                ModelPackage::Face,
                "Faces/Round",
                vec![
                    named("Faces/Round/hair_high.fmdl", "boots.fmdl"),
                    other_dds("Faces/Round/shirt.dds"),
                ],
            )],
        );

        let batch = textures_in_group(folder);

        assert!(batch.entries.is_empty(), "{:?}", paths(&batch));
        assert_eq!(batch.skipped, Vec::<usize>::new());
        assert_eq!(
            one_message(&batch),
            (
                "merged_texture_conflict",
                Severity::Error,
                Disposition::DropFolder,
                &[("texture".to_owned(), "shirt".to_owned())][..]
            )
        );
    }

    #[test]
    fn a_model_that_cannot_be_read_fails_its_task() {
        // `face_diff.bin`'s bytes under a face model's name: a file with its Phase 3 role
        // whose content cannot be read as an FMDL.
        let mut folder = player(&["face_diff.bin"]);
        let model = ScopePath::new(&format!("{PLAYER}/face_high.fmdl")).unwrap();
        folder.files.push(FileDescriptor {
            kind: aesthetics_export::classify(model.name()),
            path: model,
            ..file(&format!("{PLAYER}/face_diff.bin"))
        });

        let batch = run(TaskKind::Models {
            folder,
            package: ModelPackage::Face,
            ids: vec![79205],
        });

        assert!(batch.entries.is_empty() && batch.uniparam.is_none());
        let [message] = batch.messages.as_slice() else {
            panic!("{:?}", batch.messages);
        };
        assert_eq!(message.code.code, "folder_pack_failed");
        assert_eq!(
            (message.severity, message.disposition),
            (Severity::Error, Disposition::DropFolder)
        );
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: ExportId(4),
                path: ScopePath::new(PLAYER).unwrap(),
            }
        );
    }
}
