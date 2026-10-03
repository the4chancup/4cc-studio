//! Validation of every export source of a run, which `check` reports and `compile` starts
//! from: each source's route, the structure pass's issues, the deep pass's content findings
//! over the export it leaves (`team_compiler/pipeline.md` "2. Per-export serial steps"), and
//! the identity. The lib validates and `deep` reads the contents; this schedules them, export
//! by export.

use std::sync::Arc;

use aesthetics_export::{
    ExportIdentity, FileDescriptor, ModelSuffix, ResolvedAestheticsExport, SharedKind, SourceError,
    ValidationContext, common_link_name, model_suffix, parse_listing,
};
use pes_version::{Engine, PesVersion};
use pipeline::MemoryBudget;
use studio_core::{Disposition, ExportId, Message, Scope};
use vtree::ScopePath;

use crate::cli::RunInputs;
use crate::deep;
use crate::messages::{Code, issue_message, tool_message};
use crate::plan::ids::{SHARED_COUNT, shared_folders_taking_ids};
use crate::plan::mapped_players;
use crate::plan::subset::{
    FolderModels, ModelPackage, PlayerFile, common_skeleton, file_stem, player_file,
};
use crate::reader::{self, ExportSource, Route};

/// Validation's outcome for the whole run.
pub(crate) struct ValidationPass {
    /// Findings about the run rather than one export (the duplicate-refs summary), reported
    /// before any export's.
    pub(crate) run_messages: Vec<Message>,
    /// Every source, in discovery order.
    pub(crate) sources: Vec<CheckedSource>,
}

/// One source after validation.
pub(crate) struct CheckedSource {
    /// The source.
    pub(crate) source: ExportSource,
    /// Its findings: its route's when it was set aside, else its issues and its identity.
    pub(crate) messages: Vec<Message>,
    /// The export with its identity resolved; `None` when a finding dropped it.
    pub(crate) resolved: Option<ResolvedAestheticsExport>,
}

/// The run's memory budget, at the share of the available memory the settings give.
pub(crate) fn run_budget(inputs: &RunInputs) -> Arc<MemoryBudget> {
    MemoryBudget::new(pipeline::memory_cap(f64::from(
        inputs.common.memory_cap_percent,
    )))
}

/// Discovers the run's sources, routes them and runs the structure pass, the deep pass and
/// identity on each one headed for validation; both passes' `.7z` reads are charged to
/// `budget`. An exports folder holding no export is `no_exports_found`, on the run. Only an
/// exports folder that cannot be read is an error.
pub(crate) fn validation_pass(
    inputs: &RunInputs,
    budget: &Arc<MemoryBudget>,
) -> anyhow::Result<ValidationPass> {
    let sources = reader::discover(&inputs.exports_root, &inputs.exports)?;
    let routes = reader::route(&sources, budget);

    let mut run_messages = Vec::new();
    // Every `--export` path yields a source, so no source at all means the root's scan found
    // none.
    if sources.is_empty() {
        run_messages.push(tool_message(
            Code::NoExportsFound,
            Scope::Run,
            Disposition::Keep,
            vec![("folder", inputs.exports_root.display().to_string())],
        ));
    }
    let conflicting: Vec<&str> = sources
        .iter()
        .zip(&routes)
        .filter(|(_, route)| matches!(route, Route::ConflictingRefs))
        .map(|(source, _)| source.file_name.as_str())
        .collect();
    if !conflicting.is_empty() {
        run_messages.push(tool_message(
            Code::MultipleRefExports,
            Scope::Run,
            Disposition::DropExport,
            vec![("exports", conflicting.join(", "))],
        ));
    }

    let sources = sources
        .into_iter()
        .zip(routes)
        .map(|(source, route)| check_source(inputs, source, route, budget))
        .collect();
    Ok(ValidationPass {
        run_messages,
        sources,
    })
}

/// One source through its route, the structure pass, the deep pass and identity.
fn check_source(
    inputs: &RunInputs,
    source: ExportSource,
    route: Route,
    budget: &Arc<MemoryBudget>,
) -> CheckedSource {
    let export = Scope::Export {
        export_id: source.export_id,
    };
    // Each route's catalog consequence is "export skipped".
    let skipped = |source, code, context| CheckedSource {
        source,
        messages: vec![tool_message(
            code,
            export.clone(),
            Disposition::DropExport,
            context,
        )],
        resolved: None,
    };
    let (listing, metadata) = match route {
        Route::Unreadable(failure) => {
            return skipped(
                source,
                Code::ExportExtractFailed,
                vec![("path", failure.path), ("error", failure.error)],
            );
        }
        Route::Disabled => return skipped(source, Code::ExportDisabled, vec![]),
        Route::Balls => return skipped(source, Code::ExportBallsSkipped, vec![]),
        Route::ConflictingRefs => return skipped(source, Code::MultipleRefExports, vec![]),
        Route::Validate { listing, metadata } => (listing, metadata),
    };

    let parsed = match parse_listing(listing, metadata) {
        Ok(parsed) => parsed,
        Err(error) => {
            let (path, error) = match error {
                SourceError::Path { path, error } => (path, error.to_string()),
                SourceError::Collision { path, error } => (path, error.to_string()),
            };
            return skipped(
                source,
                Code::ExportExtractFailed,
                vec![("path", path), ("error", error)],
            );
        }
    };
    let strict = inputs.settings.strict_file_type_check;
    let context = ValidationContext {
        version: inputs.common.pes_version,
        strict_file_type_check: strict,
        pass_through: inputs.settings.pass_through,
    };
    let mut report = parsed.validate(&context);
    // The deep pass reads only what the structure pass kept; its findings derive the report
    // again, so they drop, cascade and pass through as the structure pass's own do.
    if let Some(validated) = &report.validated {
        let findings =
            deep::content_findings(validated, &source, budget, inputs.common.pes_version);
        if !findings.is_empty() {
            report = report.with_content_findings(findings, &context);
        }
    }
    let mut messages: Vec<Message> = report
        .issues
        .iter()
        .map(|issue| issue_message(issue, source.export_id, strict))
        .collect();
    // `None` means an issue dropped the export, and that issue is already among the messages.
    let resolved = report.validated.and_then(|validated| {
        match validated.resolve_identity(&inputs.teams_list) {
            Ok(resolved) => {
                messages.push(identified_message(export.clone(), &resolved.identity));
                messages.extend(model_name_messages(
                    &resolved,
                    inputs.common.pes_version,
                    source.export_id,
                ));
                let exhausted = pool_messages(&resolved, inputs.common.pes_version, &export);
                if exhausted.is_empty() {
                    Some(resolved)
                } else {
                    messages.extend(exhausted);
                    None
                }
            }
            Err(error) => {
                messages.push(tool_message(
                    Code::TeamNameUnknown,
                    export.clone(),
                    Disposition::DropExport,
                    vec![("team_name", error.team_name.as_str().to_owned())],
                ));
                None
            }
        }
    });
    CheckedSource {
        source,
        messages,
        resolved,
    }
}

/// `boots_id_pool_exhausted` and `gloves_id_pool_exhausted`, each when more shared folders of
/// its kind take an id for `version` than the team's block holds, naming the count; the export
/// is dropped (`team_compiler/README.md` TC-MOD-06). A referee export has no block: its ids
/// are its slots' (`k99NN`).
fn pool_messages(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
    export: &Scope,
) -> Vec<Message> {
    match resolved.identity {
        ExportIdentity::Team { .. } => {}
        ExportIdentity::Referees => return Vec::new(),
    }
    [
        (SharedKind::Boots, Code::BootsIdPoolExhausted),
        (SharedKind::Gloves, Code::GlovesIdPoolExhausted),
    ]
    .into_iter()
    .filter_map(|(kind, code)| {
        let count = shared_folders_taking_ids(&resolved.export, version.engine(), kind).len();
        (count > usize::from(SHARED_COUNT)).then(|| {
            tool_message(
                code,
                export.clone(),
                Disposition::DropExport,
                vec![("count", count.to_string())],
            )
        })
    })
    .collect()
}

/// `fmdl_fcl_hair_fallback` for each Fox model the `fcl_hair` merge takes without being named
/// for it: one whose name says nothing about what it is, or one a `face/` subfolder makes
/// face content, and `skl_no_slot` for each `.skl` paired with a `face_high`, `hair_high` or
/// `oral` model, which has no slot to land in and is ignored (`player_folders.md` "Model
/// names", "Reserved subfolders", "SKL pairing"; `team_compiler/README.md` TC-MOD-13):
/// over every mapped player folder and every shared face folder, each finding on the folder
/// holding the file. A `.common` model link is reported like the model it brings in, naming
/// the link: the fallback by the linked name's suffix, `skl_no_slot` when `Common/` holds the
/// `.skl` of a slotless model's stem. The roles are `subset::player_file`'s, so a finding never
/// disagrees with the routing. A pre-Fox target types a model by its name and reads no `.skl`,
/// so it reports neither.
fn model_name_messages(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
    export_id: ExportId,
) -> Vec<Message> {
    match version.engine() {
        Engine::Fox => {}
        Engine::PreFox => return Vec::new(),
    }
    let export = &resolved.export;
    let mut messages = Vec::new();
    for folder in mapped_players(export) {
        let models = FolderModels::of_player(folder);
        file_role_messages(
            &folder.path,
            &folder.files,
            &models,
            &export.common,
            export_id,
            &mut messages,
        );
    }
    // Validation drops a shared folder no mapped player links, so every face folder here is
    // one some player's face is assembled from.
    for folder in &export.faces {
        let models = FolderModels::of(&folder.path, &folder.files);
        file_role_messages(
            &folder.path,
            &folder.files,
            &models,
            &export.common,
            export_id,
            &mut messages,
        );
    }
    messages
}

/// `model_name_messages`'s findings on `files`, the files of the folder at `path` whose models
/// are `models`, in file order; `common` is the export's `Common/` files, where a `.common`
/// link's model and skeleton are.
fn file_role_messages(
    path: &ScopePath,
    files: &[FileDescriptor],
    models: &FolderModels,
    common: &[FileDescriptor],
    export_id: ExportId,
    messages: &mut Vec<Message>,
) {
    for file in files {
        let name = file.path.name();
        let code = match player_file(path, file, models) {
            Some(PlayerFile::Model {
                package: ModelPackage::Face,
                name: "fcl_hair",
            }) if model_suffix(file_stem(name)) != Some(ModelSuffix::FclHair) => {
                Code::FmdlFclHairFallback
            }
            Some(PlayerFile::SlotlessSkeleton) => Code::SklNoSlot,
            Some(PlayerFile::CommonModel {
                package: ModelPackage::Face,
                name: allowed,
            }) => {
                let linked = common_link_name(name)
                    .expect("a CommonModel role implies a `.common` link name");
                if allowed == "fcl_hair" {
                    if model_suffix(file_stem(&linked)) == Some(ModelSuffix::FclHair) {
                        continue;
                    }
                    Code::FmdlFclHairFallback
                } else if common_skeleton(common, &linked).is_some() {
                    Code::SklNoSlot
                } else {
                    continue;
                }
            }
            Some(
                PlayerFile::Model { .. }
                | PlayerFile::CommonModel {
                    package: ModelPackage::Boots | ModelPackage::Gloves,
                    ..
                }
                | PlayerFile::Packed { .. }
                | PlayerFile::FaceDiffXml
                | PlayerFile::Skeleton { .. }
                | PlayerFile::Texture(..),
            )
            | None => continue,
        };
        messages.push(tool_message(
            code,
            Scope::Folder {
                export_id,
                path: path.clone(),
            },
            Disposition::Keep,
            vec![("file", name.to_owned())],
        ));
    }
}

/// `export_identified` naming the team and its id, or the referees.
fn identified_message(export: Scope, identity: &ExportIdentity) -> Message {
    let context = match identity {
        ExportIdentity::Team { id, name } => {
            vec![("team", name.as_str().to_owned()), ("id", id.to_string())]
        }
        ExportIdentity::Referees => vec![("team", "referees".to_owned())],
    };
    tool_message(Code::ExportIdentified, export, Disposition::Keep, context)
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::{resolved, resolved_with_issues};

    /// The export `co - Pool` whose slots 01 to `count` each link their own shared folder of
    /// `kind` (`Boots/S01/` for slot 01), each player folder also holding `local`.
    fn linking_export(count: u8, kind: SharedKind, local: &[&str]) -> ResolvedAestheticsExport {
        let (extension, content_folder, model) = match kind {
            SharedKind::Face => ("face", "Faces", "face_high.fmdl"),
            SharedKind::Boots => ("boots", "Boots", "boots.fmdl"),
            SharedKind::Gloves => ("gloves", "Gloves", "glove_l.fmdl"),
        };
        let mut files = Vec::new();
        for slot in 1..=count {
            let player = format!("Players/{slot:02} - P{slot:02}");
            files.push(format!("{player}/S{slot:02}.{extension}"));
            for name in local {
                files.push(format!("{player}/{name}"));
            }
            files.push(format!("{content_folder}/S{slot:02}/{model}"));
        }
        let files: Vec<(&str, u64)> = files.iter().map(|path| (path.as_str(), 1)).collect();
        resolved("co - Pool", &files, &[], None)
    }

    /// `pool_messages` over `export` for `version`, each as (code, disposition, count).
    fn pool(
        export: &ResolvedAestheticsExport,
        version: PesVersion,
    ) -> Vec<(String, Disposition, String)> {
        let scope = Scope::Export {
            export_id: ExportId(0),
        };
        pool_messages(export, version, &scope)
            .into_iter()
            .map(|message| {
                assert_eq!(message.scope, scope);
                assert_eq!(message.severity, Severity::Error);
                let [(key, count)] = message.context.as_slice() else {
                    panic!("{:?}", message.context);
                };
                assert_eq!(key, "count");
                (
                    message.code.code.to_string(),
                    message.disposition,
                    count.clone(),
                )
            })
            .collect()
    }

    /// `model_name_messages` over `export` for `version`, each as one line: severity, code,
    /// disposition, folder and context.
    fn names(export: &ResolvedAestheticsExport, version: PesVersion) -> Vec<String> {
        model_name_messages(export, version, ExportId(2))
            .into_iter()
            .map(|message| {
                let Scope::Folder { export_id, path } = &message.scope else {
                    panic!("{:?}", message.scope);
                };
                assert_eq!(*export_id, ExportId(2));
                let context: Vec<String> = message
                    .context
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect();
                format!(
                    "{:?} {} [{:?}] at {} ({})",
                    message.severity,
                    message.code.code,
                    message.disposition,
                    path.as_str(),
                    context.join(", ")
                )
            })
            .collect()
    }

    #[test]
    fn an_unsuffixed_model_and_a_slotless_skeleton_are_reported_on_their_folder_on_fox_only() {
        let files = [
            ("Players/03 - A/torso.fmdl", 1),
            ("Players/03 - A/torso.skl", 1),
            ("Players/03 - A/face_high.fmdl", 1),
            ("Players/03 - A/face_high.skl", 1),
            ("Players/03 - A/x_fcl_hair.fmdl", 1),
            ("Players/03 - A/x_fcl_hair.skl", 1),
            ("Players/07 - B/Round.face", 0),
            ("Players/07 - B/kit_boots.fmdl", 1),
            ("Players/07 - B/kit_boots.skl", 1),
            ("Faces/Round/legs.fmdl", 1),
            ("Faces/Round/oral.fmdl", 1),
            ("Faces/Round/oral.skl", 1),
        ];
        let export = resolved("co - Names", &files, &[], None);
        // The hair's and the boots' skeletons have their slots; a shared face folder's files
        // are reported on that folder, once, however many players link it. Within a folder
        // the findings follow its files, in the folded name order validation keeps them in.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning skl_no_slot [Keep] at Players/03 - A (file=face_high.skl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=torso.fmdl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Faces/Round (file=legs.fmdl)",
                "Warning skl_no_slot [Keep] at Faces/Round (file=oral.skl)",
            ]
        );
        assert_eq!(names(&export, PesVersion::Pes17), Vec::<String>::new());
    }

    #[test]
    fn a_face_subfolder_s_forced_hair_part_is_reported_and_a_boots_subfolder_s_skeleton_is_not() {
        let files = [
            ("Players/03 - A/boots/hair_high.fmdl", 1),
            ("Players/03 - A/boots/hair_high.skl", 1),
            ("Players/03 - A/face/boots.fmdl", 1),
            ("Players/03 - A/face/torso.fmdl", 1),
            ("Players/03 - A/face/face_high.fmdl", 1),
            ("Players/03 - A/face/face_high.skl", 1),
            ("Players/03 - A/fcl_hair.fmdl", 1),
        ];
        let export = resolved("co - Names", &files, &[], None);
        // `boots/` makes `hair_high` the boots, so its skeleton has a slot; `face/` makes
        // `boots.fmdl` hair content, reported like an unsuffixed model.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=boots.fmdl)",
                "Warning skl_no_slot [Keep] at Players/03 - A (file=face_high.skl)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=torso.fmdl)",
            ]
        );
    }

    #[test]
    fn a_common_link_is_reported_like_the_model_it_brings_in_naming_the_link() {
        let files = [
            ("Players/03 - A/legs.fmdl.common", 0),
            ("Players/03 - A/x_fcl_hair.fmdl.common", 0),
            ("Players/03 - A/face_high.fmdl.common", 0),
            ("Players/03 - A/oral.fmdl.common.txt", 0),
            ("Players/03 - A/boots/hair_high.fmdl.common", 0),
            ("Common/legs.fmdl", 1),
            ("Common/legs.skl", 1),
            ("Common/x_fcl_hair.fmdl", 1),
            ("Common/face_high.fmdl", 1),
            ("Common/face_high.skl", 1),
            ("Common/oral.fmdl", 1),
            ("Common/hair_high.fmdl", 1),
            ("Common/hair_high.skl", 1),
        ];
        let export = resolved("co - Names", &files, &[], None);
        // `legs` is hair content by its name; `face_high` has no slot for Common's skeleton,
        // `oral` has none to report, and `boots/` makes `hair_high` the boots, whose skeleton
        // has a slot.
        assert_eq!(
            names(&export, PesVersion::Pes21),
            [
                "Warning skl_no_slot [Keep] at Players/03 - A (file=face_high.fmdl.common)",
                "Info fmdl_fcl_hair_fallback [Keep] at Players/03 - A (file=legs.fmdl.common)",
            ]
        );
        assert_eq!(names(&export, PesVersion::Pes17), Vec::<String>::new());
    }

    #[test]
    fn a_player_folder_no_slot_maps_is_not_walked() {
        let (export, issues) = resolved_with_issues(
            "co - Names",
            &[
                ("Players/A/face_high.fmdl", 1),
                ("Players/Unlisted/torso.fmdl", 1),
            ],
            &[],
            Some(b"03 A\n"),
        );
        assert_eq!(issues, ["player_unlisted"]);
        assert_eq!(names(&export, PesVersion::Pes21), Vec::<String>::new());
    }

    #[test]
    fn eighteen_shared_gloves_folders_exhaust_the_pool_and_seventeen_fit() {
        assert_eq!(
            pool(
                &linking_export(18, SharedKind::Gloves, &[]),
                PesVersion::Pes21
            ),
            [(
                "gloves_id_pool_exhausted".to_owned(),
                Disposition::DropExport,
                "18".to_owned()
            )]
        );
        assert_eq!(
            pool(
                &linking_export(17, SharedKind::Gloves, &[]),
                PesVersion::Pes21
            ),
            []
        );
    }

    #[test]
    fn links_beside_local_boots_count_only_pre_fox() {
        // On Fox each player combines its link into its own package; pre-Fox has no package
        // of the player's own, so every shared folder takes an id.
        let export = linking_export(18, SharedKind::Boots, &["kit_boots.fmdl"]);
        assert_eq!(pool(&export, PesVersion::Pes21), []);
        assert_eq!(
            pool(&export, PesVersion::Pes17),
            [(
                "boots_id_pool_exhausted".to_owned(),
                Disposition::DropExport,
                "18".to_owned()
            )]
        );
    }
}
