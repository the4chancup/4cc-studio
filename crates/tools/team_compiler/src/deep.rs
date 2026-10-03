//! The deep pass (`team_compiler/pipeline.md` "2. Per-export serial steps", "Deep format
//! pass"): the checks only a file's contents can answer, run over the sanitized export the
//! structure pass leaves. What they find goes back to `aesthetics_export` as content findings,
//! which derive the sanitized export again, so a finding drops, cascades and passes through by
//! the structure pass's own rules.
//!
//! This module reads every native model (`.fmdl`, `.model`) and every `.mtl` of the player
//! folders, the shared folders and `Common/`, whatever the target version (a model of either
//! format is a source for either target), and reports what the format crates' checks find
//! (`team_compiler/messages.md` "Model checks"): one finding per file and code, at the
//! severity the format crate gives it. An Error drops what holds the file and may pass through;
//! a Warning or an Info only informs. The far vertex, which both formats check, is reported as
//! `vertex_too_far_from_origin` and never passes through. A file that does not parse is
//! `model_broken` or `mtl_broken`. glTF models are not read (Phase 7).

use std::sync::Arc;

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, ModelFormat,
    ValidatedAestheticsExport,
};
use fmdl::{FmdlFile, Model};
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;
use pipeline::MemoryBudget;
use vtree::ScopePath;

use crate::messages::Code;
use crate::reader::{ContentSource, ExportSource};

/// The format crates' far-vertex codes, both reported as `vertex_too_far_from_origin`.
pub(crate) const FAR_VERTEX_CODES: [&str; 2] = [
    "fmdl_vertex_far_from_origin",
    "model_vertex_far_from_origin",
];

/// The content findings of `export`, the sanitized export read from `source`, in file order:
/// each player folder's models and material sets, then each shared folder's (faces, boots,
/// gloves), then `Common/`'s. An Error on a folder's file drops the folder; one on a `Common/`
/// file drops the file, and the cascade then drops the players linking it. Files are read one
/// at a time through one `ContentSource`, so only one file's bytes are held at once, and a
/// solid `.7z` is decompressed once, under its own permit from `budget`, released when the
/// pass ends.
pub(crate) fn content_findings(
    export: &ValidatedAestheticsExport,
    source: &ExportSource,
    budget: &Arc<MemoryBudget>,
) -> Vec<ContentFinding> {
    let mut content = ContentSource::new(source, budget);
    let players = export
        .players
        .iter()
        .map(|player| (&player.path, &player.files));
    let shared = [&export.faces, &export.boots, &export.gloves]
        .into_iter()
        .flatten()
        .map(|folder| (&folder.path, &folder.files));
    let mut findings = Vec::new();
    for (folder, files) in players.chain(shared) {
        for file in files {
            let Some(checked) = checked_as(file.kind) else {
                continue;
            };
            findings.extend(file_findings(
                &mut content,
                file,
                checked,
                &IssueScope::Folder(folder.clone()),
                Disposition::DropFolder,
                &relative(&file.path, folder),
            ));
        }
    }
    for file in &export.common {
        let Some(checked) = checked_as(file.kind) else {
            continue;
        };
        findings.extend(file_findings(
            &mut content,
            file,
            checked,
            &IssueScope::File(file.path.clone()),
            Disposition::DropFile,
            file.path.name(),
        ));
    }
    findings
}

/// What the deep pass reads a file as, and which format crate checks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Checked {
    /// A Fox model, checked by `fmdl`.
    Fmdl,
    /// A pre-Fox model, checked by `pes_model`.
    PreFoxModel,
    /// A pre-Fox material set, checked by `pes_model`.
    Mtl,
}

/// What a file of `kind` is read as, or `None` for a file the deep pass does not read.
fn checked_as(kind: FileKind) -> Option<Checked> {
    match kind {
        FileKind::Model(ModelFormat::Fmdl) => Some(Checked::Fmdl),
        FileKind::Model(ModelFormat::PesModel) => Some(Checked::PreFoxModel),
        FileKind::Mtl => Some(Checked::Mtl),
        // glTF is read from Phase 7; nothing else is checked by a format crate.
        FileKind::Model(ModelFormat::Gltf)
        | FileKind::Texture
        | FileKind::Skl
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::MaterialsToml
        | FileKind::Bin
        | FileKind::SharedLink(_)
        | FileKind::CommonLink
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// `path` below `folder`, its subfolder kept (`face/hair.fmdl`).
fn relative(path: &ScopePath, folder: &ScopePath) -> String {
    path.segments()
        .skip(folder.segments().count())
        .collect::<Vec<_>>()
        .join("/")
}

/// One rule a format crate's check fired on a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Fired {
    /// The format crate's code.
    code: &'static str,
    /// Whether the format crate rates it an Error, which drops what holds the file.
    error: bool,
    /// How many items tripped the rule.
    count: usize,
}

impl Fired {
    /// `fmdl`'s finding `found` as a rule fired: its code, whether it is an Error, its count.
    fn fox(found: fmdl::check::Finding) -> Fired {
        let error = match found.severity {
            fmdl::check::Severity::Error => true,
            fmdl::check::Severity::Warning | fmdl::check::Severity::Info => false,
        };
        Fired {
            code: found.code,
            error,
            count: found.count,
        }
    }

    /// `pes_model`'s finding `found` as a rule fired: its code, whether it is an Error, its
    /// count.
    fn pre_fox(found: pes_model::check::Finding) -> Fired {
        let error = match found.severity {
            pes_model::check::Severity::Error => true,
            pes_model::check::Severity::Warning | pes_model::check::Severity::Info => false,
        };
        Fired {
            code: found.code,
            error,
            count: found.count,
        }
    }
}

/// The rules the format crate's check fires on `bytes`, read as `checked`, or the reader's
/// error text when they do not parse.
fn fired(checked: Checked, bytes: &[u8]) -> Result<Vec<Fired>, String> {
    let fired = match checked {
        Checked::Fmdl => {
            let model = FmdlFile::read(bytes)
                .and_then(|file| Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            fmdl::check::check(&model)
                .into_iter()
                .map(Fired::fox)
                .collect()
        }
        Checked::PreFoxModel => {
            let model = PreFoxModel::read(bytes)
                .and_then(|file| pes_model::model::Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            pes_model::check::check(&model)
                .into_iter()
                .map(Fired::pre_fox)
                .collect()
        }
        Checked::Mtl => {
            let set = MaterialSet::read(bytes).map_err(|error| error.to_string())?;
            pes_model::check::check_materials(&set)
                .into_iter()
                .map(Fired::pre_fox)
                .collect()
        }
    };
    Ok(fired)
}

/// `fired` with one entry per code, in the order each code first fired, its counts summed:
/// the checks report per mesh or per material, a member reads one line per file.
fn summed(fired: Vec<Fired>) -> Vec<Fired> {
    let mut summed: Vec<Fired> = Vec::new();
    for rule in fired {
        match summed.iter_mut().find(|known| known.code == rule.code) {
            Some(known) => known.count += rule.count,
            None => summed.push(rule),
        }
    }
    summed
}

/// The findings of `file`, read as `checked`, each on `scope` and naming the file `name`: one
/// per code its format crate's check fires, with the summed count (an Error with
/// `disposition`, pass-through-eligible unless it is the far vertex; a Warning or an Info
/// `Keep`). A file that does not parse is `model_broken` or `mtl_broken`, and one that
/// cannot be read `source_read_failed`, both with `disposition` and never eligible: there is
/// nothing to pack.
fn file_findings(
    content: &mut ContentSource,
    file: &FileDescriptor,
    checked: Checked,
    scope: &IssueScope,
    disposition: Disposition,
    name: &str,
) -> Vec<ContentFinding> {
    let finding = |code: &'static str,
                   context: Vec<(&'static str, String)>,
                   disposition: Disposition,
                   pass_through_eligible: bool| ContentFinding {
        code,
        scope: scope.clone(),
        context,
        disposition,
        pass_through_eligible,
    };
    let bytes = match content.read(file.source.as_str()) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return vec![finding(
                Code::SourceReadFailed.as_str(),
                vec![("path", failure.path), ("error", failure.error)],
                disposition,
                false,
            )];
        }
    };
    let fired = match fired(checked, &bytes) {
        Ok(fired) => fired,
        Err(error) => {
            let broken = match checked {
                Checked::Fmdl | Checked::PreFoxModel => Code::ModelBroken,
                Checked::Mtl => Code::MtlBroken,
            };
            return vec![finding(
                broken.as_str(),
                vec![("file", name.to_owned()), ("error", error)],
                disposition,
                false,
            )];
        }
    };
    summed(fired)
        .into_iter()
        .map(|rule| {
            let far = FAR_VERTEX_CODES.contains(&rule.code);
            let code = if far {
                Code::VertexTooFarFromOrigin.as_str()
            } else {
                rule.code
            };
            let context = vec![("file", name.to_owned()), ("count", rule.count.to_string())];
            if rule.error {
                // Pass-through packs the file as it is; far geometry lags the whole matchday.
                finding(code, context, disposition, !far)
            } else {
                finding(code, context, Disposition::Keep, false)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use aesthetics_export::{Disposition, IssueScope};
    use pes_model::format::PreFoxModel;
    use pes_model::format::mtl::MaterialSet;
    use studio_core::ExportId;
    use vtree::ScopePath;

    use super::*;
    use crate::reader::SourceKind;
    use crate::testing::{resolved, scratch};

    /// The bytes of `tests/fixtures/<relative>`.
    fn fixture(relative: &str) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(relative),
        )
        .unwrap()
    }

    /// The bytes of `pes_model`'s fixture `name`: real pre-Fox models and material sets,
    /// which this crate's own fixtures do not hold (`pes_model/tests/fixtures/README.md`).
    fn pre_fox_fixture(name: &str) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../libs/pes_model/tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    /// The bytes of the tracer player's file `name`.
    fn tracer_file(name: &str) -> Vec<u8> {
        fixture(&format!(
            "tracer/studio/egg Tracer/Players/05 - The Chad Stormworks Player/{name}"
        ))
    }

    /// The tracer's boots model with one vertex 6000 units from the origin.
    fn far_boots() -> Vec<u8> {
        fixture("deep/boots_far.fmdl")
    }

    /// The tracer's own boots model, all of it near the origin; 1662 of its vertices carry
    /// weights that sum to neither 255 nor 0 (1626, 18 and 18 in its three meshes).
    fn tracer_boots() -> Vec<u8> {
        tracer_file("boots.fmdl")
    }

    /// The Fox model `bytes` with `edit` applied, written back through `fmdl`.
    fn edited(bytes: &[u8], edit: impl FnOnce(&mut Model)) -> Vec<u8> {
        let mut model = Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap();
        edit(&mut model);
        model.to_file().unwrap().write()
    }

    /// The tracer's right glove, in which `fmdl`'s check finds nothing, with its mesh holding
    /// 21846 faces (its first, repeated), one over the hard limit: one
    /// `fmdl_mesh_over_face_limit`, an Error. (`fmdl` refuses to write a face naming a
    /// vertex that does not exist, or a mesh no group lists.)
    fn glove_over_the_face_limit() -> Vec<u8> {
        edited(&tracer_file("glove_r.fmdl"), |model| {
            let mesh = &mut model.meshes[0];
            mesh.faces = vec![mesh.faces[0]; 21_846];
        })
    }

    /// The deep pass over the folder export `co - Deep` at `root`, holding `files` (path,
    /// bytes) and listing `unwritten` too, which is not on disk.
    fn findings_of(
        root: &Path,
        files: &[(&str, Vec<u8>)],
        unwritten: &[&str],
    ) -> Vec<ContentFinding> {
        for (path, bytes) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        let listed: Vec<(&str, u64)> = files
            .iter()
            .map(|(path, bytes)| (*path, bytes.len() as u64))
            .chain(unwritten.iter().map(|path| (*path, 1)))
            .collect();
        let export = resolved("co - Deep", &listed, &[], None).export;
        let source = ExportSource {
            export_id: ExportId(0),
            path: root.to_path_buf(),
            kind: SourceKind::Folder,
            file_name: "co - Deep".to_owned(),
            display_name: "co - Deep".to_owned(),
            team_name: None,
        };
        content_findings(&export, &source, &MemoryBudget::new(1 << 30))
    }

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    fn folder(text: &str) -> IssueScope {
        IssueScope::Folder(path(text))
    }

    /// The finding `code` on `scope` about the file `file`, `count` items of which tripped
    /// the rule.
    fn counted(
        code: &'static str,
        scope: &IssueScope,
        file: &str,
        count: usize,
        disposition: Disposition,
        pass_through_eligible: bool,
    ) -> ContentFinding {
        ContentFinding {
            code,
            scope: scope.clone(),
            context: vec![("file", file.to_owned()), ("count", count.to_string())],
            disposition,
            pass_through_eligible,
        }
    }

    /// `code` on the folder `scope`: the file `file` does not parse, for `error`.
    fn broken(code: &'static str, scope: &str, file: &str, error: String) -> ContentFinding {
        ContentFinding {
            code,
            scope: folder(scope),
            context: vec![("file", file.to_owned()), ("error", error)],
            disposition: Disposition::DropFolder,
            pass_through_eligible: false,
        }
    }

    #[test]
    fn a_common_model_s_far_vertex_drops_the_file() {
        let temp = scratch("deep_common");
        let findings = findings_of(
            temp.path(),
            &[
                ("Common/legs.fmdl", far_boots()),
                ("Players/03 - A/legs.fmdl.common", Vec::new()),
            ],
            &[],
        );
        let file = IssueScope::File(path("Common/legs.fmdl"));
        assert_eq!(
            findings,
            [
                counted(
                    "vertex_too_far_from_origin",
                    &file,
                    "legs.fmdl",
                    1,
                    Disposition::DropFile,
                    false
                ),
                counted(
                    "fmdl_weights_not_normalized",
                    &file,
                    "legs.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_model_in_a_subfolder_is_named_with_its_subfolder_and_drops_the_folder() {
        let temp = scratch("deep_subfolder");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/face/hair.fmdl", far_boots()),
                ("Players/05 - B/boots.fmdl", tracer_boots()),
            ],
            &[],
        );
        let a = folder("Players/03 - A");
        let b = folder("Players/05 - B");
        assert_eq!(
            findings,
            [
                counted(
                    "vertex_too_far_from_origin",
                    &a,
                    "face/hair.fmdl",
                    1,
                    Disposition::DropFolder,
                    false
                ),
                counted(
                    "fmdl_weights_not_normalized",
                    &a,
                    "face/hair.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
                counted(
                    "fmdl_weights_not_normalized",
                    &b,
                    "boots.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn far_vertices_in_two_meshes_are_one_finding_with_their_summed_count() {
        let temp = scratch("deep_two_meshes");
        let boots = edited(&far_boots(), |model| {
            model.meshes[1].vertices.positions[0] = [0.0, 0.0, 7000.0];
        });
        let findings = findings_of(temp.path(), &[("Players/03 - A/boots.fmdl", boots)], &[]);
        let a = folder("Players/03 - A");
        assert_eq!(
            findings,
            [
                counted(
                    "vertex_too_far_from_origin",
                    &a,
                    "boots.fmdl",
                    2,
                    Disposition::DropFolder,
                    false
                ),
                counted(
                    "fmdl_weights_not_normalized",
                    &a,
                    "boots.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_format_error_keeps_its_code_drops_the_folder_and_may_pass_through() {
        let temp = scratch("deep_format_error");
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/glove_r.fmdl", glove_over_the_face_limit())],
            &[],
        );
        assert_eq!(
            findings,
            [counted(
                "fmdl_mesh_over_face_limit",
                &folder("Players/03 - A"),
                "glove_r.fmdl",
                21_846,
                Disposition::DropFolder,
                true
            )]
        );
    }

    #[test]
    fn a_common_model_s_format_error_drops_the_file() {
        let temp = scratch("deep_common_error");
        let findings = findings_of(
            temp.path(),
            &[
                ("Common/legs.fmdl", glove_over_the_face_limit()),
                ("Players/03 - A/legs.fmdl.common", Vec::new()),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [counted(
                "fmdl_mesh_over_face_limit",
                &IssueScope::File(path("Common/legs.fmdl")),
                "legs.fmdl",
                21_846,
                Disposition::DropFile,
                true
            )]
        );
    }

    #[test]
    fn the_tracer_s_boots_keep_their_folder_and_a_model_that_does_not_parse_drops_its_own() {
        let temp = scratch("deep_clean");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/boots.fmdl", tracer_boots()),
                ("Players/05 - B/boots.fmdl", b"not a model".to_vec()),
            ],
            &[],
        );
        let error = FmdlFile::read(b"not a model").unwrap_err().to_string();
        assert_eq!(
            findings,
            [
                counted(
                    "fmdl_weights_not_normalized",
                    &folder("Players/03 - A"),
                    "boots.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
                broken("model_broken", "Players/05 - B", "boots.fmdl", error),
            ]
        );
    }

    #[test]
    fn pre_fox_models_and_material_sets_are_checked() {
        let temp = scratch("deep_pre_fox");
        let findings = findings_of(
            temp.path(),
            &[
                (
                    "Players/03 - A/face_high.model",
                    pre_fox_fixture("konami_card.model"),
                ),
                (
                    "Players/03 - A/materials.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
                (
                    "Players/05 - B/boots.model",
                    pre_fox_fixture("community_empty_geometry_boots.wesys.model"),
                ),
                (
                    "Players/05 - B/boots.mtl",
                    pre_fox_fixture("konami_shadow.mtl"),
                ),
            ],
            &[],
        );
        let b = folder("Players/05 - B");
        // The boots' two meshes are empty; the shadow's one material names no state.
        assert_eq!(
            findings,
            [
                counted(
                    "model_mesh_empty",
                    &b,
                    "boots.model",
                    2,
                    Disposition::Keep,
                    false
                ),
                counted(
                    "mtl_state_missing",
                    &b,
                    "boots.mtl",
                    7,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_pre_fox_model_s_far_vertex_is_vertex_too_far_from_origin() {
        let temp = scratch("deep_pre_fox_far");
        // Konami's referee card, clean, with its first vertex 6000 units from the origin.
        let file = PreFoxModel::read(&pre_fox_fixture("konami_card.model")).unwrap();
        let mut model = pes_model::model::Model::from_file(&file).unwrap();
        model.meshes[0].vertices.positions[0] = [6000.0, 0.0, 0.0];
        let card = model.to_file().unwrap().write().unwrap();
        let findings = findings_of(temp.path(), &[("Players/03 - A/card.model", card)], &[]);
        assert_eq!(
            findings,
            [counted(
                "vertex_too_far_from_origin",
                &folder("Players/03 - A"),
                "card.model",
                1,
                Disposition::DropFolder,
                false
            )]
        );
    }

    #[test]
    fn a_pre_fox_error_keeps_its_code_drops_the_folder_and_may_pass_through() {
        let temp = scratch("deep_pre_fox_error");
        // The card head's material set, clean, with its one material listed twice.
        let mut set = MaterialSet::read(&pre_fox_fixture("cardhead_materials.mtl")).unwrap();
        set.materials.push(set.materials[0].clone());
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/materials.mtl", set.write())],
            &[],
        );
        assert_eq!(
            findings,
            [counted(
                "mtl_material_duplicate",
                &folder("Players/03 - A"),
                "materials.mtl",
                1,
                Disposition::DropFolder,
                true
            )]
        );
    }

    #[test]
    fn a_pre_fox_model_or_material_set_that_does_not_parse_drops_its_folder() {
        let temp = scratch("deep_pre_fox_broken");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/boots.model", b"not a model".to_vec()),
                ("Players/05 - B/boots.mtl", b"not a material set".to_vec()),
            ],
            &[],
        );
        let model_error = PreFoxModel::read(b"not a model").unwrap_err().to_string();
        let mtl_error = MaterialSet::read(b"not a material set")
            .unwrap_err()
            .to_string();
        assert_eq!(
            findings,
            [
                broken("model_broken", "Players/03 - A", "boots.model", model_error),
                broken("mtl_broken", "Players/05 - B", "boots.mtl", mtl_error),
            ]
        );
    }

    #[test]
    fn files_that_are_not_models_or_material_sets_are_not_read() {
        let temp = scratch("deep_not_models");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/05 - B/glove_r.fmdl", tracer_file("glove_r.fmdl")),
                ("Players/05 - B/shirt.dds", tracer_file("shirt.dds")),
                ("Players/05 - B/fcl_hair.skl", tracer_file("fcl_hair.skl")),
                ("Players/05 - B/face_diff.bin", tracer_file("face_diff.bin")),
            ],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn a_model_that_cannot_be_read_is_source_read_failed_on_its_folder() {
        let temp = scratch("deep_unreadable");
        let findings = findings_of(
            temp.path(),
            &[("Players/05 - B/boots.fmdl", tracer_boots())],
            &["Players/03 - A/boots.fmdl"],
        );
        let [finding, readable] = findings.as_slice() else {
            panic!("{findings:?}");
        };
        assert_eq!(finding.code, "source_read_failed");
        assert_eq!(finding.scope, folder("Players/03 - A"));
        assert_eq!(finding.disposition, Disposition::DropFolder);
        assert!(!finding.pass_through_eligible);
        let missing: PathBuf = temp.path().join("Players/03 - A/boots.fmdl");
        assert_eq!(finding.context[0], ("path", missing.display().to_string()));
        assert_eq!(finding.context[1].0, "error");
        assert_eq!(finding.context.len(), 2);
        assert_eq!(
            *readable,
            counted(
                "fmdl_weights_not_normalized",
                &folder("Players/05 - B"),
                "boots.fmdl",
                1662,
                Disposition::Keep,
                false
            )
        );
    }
}
