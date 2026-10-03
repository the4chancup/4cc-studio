//! The deep pass (`team_compiler/pipeline.md` "2. Per-export serial steps", "Deep format
//! pass"): the checks only a file's contents can answer, run over the sanitized export the
//! structure pass leaves. What they find goes back to `aesthetics_export` as content findings,
//! which derive the sanitized export again, so a finding drops, cascades and passes through by
//! the structure pass's own rules.
//!
//! This module checks every Fox model (`.fmdl`) of the player folders, the shared folders and
//! `Common/` for a vertex more than 5000 units from the origin (`vertex_too_far_from_origin`),
//! whatever the target version: a Fox model is a source for a pre-Fox target too. A model that
//! does not parse gets no finding here; the task that builds it fails at `compile`.

use std::sync::Arc;

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, ModelFormat,
    ValidatedAestheticsExport,
};
use fmdl::{FmdlFile, Model};
use pipeline::MemoryBudget;
use vtree::ScopePath;

use crate::messages::Code;
use crate::reader::{ContentSource, ExportSource};

/// The content findings of `export`, the sanitized export read from `source`, in file order:
/// each player folder's models, then each shared folder's (faces, boots, gloves), then
/// `Common/`'s. A finding on a folder's model drops the folder; one on a `Common/` model drops
/// the file, and the cascade then drops the players linking it. Files are read one at a time
/// through one `ContentSource`, so only one file's bytes are held at once, and a solid `.7z`
/// is decompressed once, under its own permit from `budget`, released when the pass ends.
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
        for file in files.iter().filter(|file| is_fox_model(file)) {
            findings.extend(model_findings(
                &mut content,
                file,
                IssueScope::Folder(folder.clone()),
                Disposition::DropFolder,
                &relative(&file.path, folder),
            ));
        }
    }
    for file in export.common.iter().filter(|file| is_fox_model(file)) {
        findings.extend(model_findings(
            &mut content,
            file,
            IssueScope::File(file.path.clone()),
            Disposition::DropFile,
            file.path.name(),
        ));
    }
    findings
}

/// Whether `file` is a Fox model.
fn is_fox_model(file: &FileDescriptor) -> bool {
    file.kind == FileKind::Model(ModelFormat::Fmdl)
}

/// `path` below `folder`, its subfolder kept (`face/hair.fmdl`).
fn relative(path: &ScopePath, folder: &ScopePath) -> String {
    path.segments()
        .skip(folder.segments().count())
        .collect::<Vec<_>>()
        .join("/")
}

/// The findings of the Fox model `file`, each on `scope` with `disposition` and never
/// pass-through-eligible: `vertex_too_far_from_origin` per far finding of `fmdl`'s check,
/// naming the model `name`, or `source_read_failed` when the file cannot be read.
fn model_findings(
    content: &mut ContentSource,
    file: &FileDescriptor,
    scope: IssueScope,
    disposition: Disposition,
    name: &str,
) -> Vec<ContentFinding> {
    let finding = |code: Code, context: Vec<(&'static str, String)>| ContentFinding {
        code: code.as_str(),
        scope: scope.clone(),
        context,
        disposition,
        pass_through_eligible: false,
    };
    let bytes = match content.read(file.source.as_str()) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return vec![finding(
                Code::SourceReadFailed,
                vec![("path", failure.path), ("error", failure.error)],
            )];
        }
    };
    let model = match FmdlFile::read(&bytes).and_then(|parsed| Model::from_file(&parsed)) {
        Ok(model) => model,
        Err(error) => {
            // Building the model fails its task at `compile`, which reports it.
            log::debug!(
                "{}: not checked, it does not parse: {error}",
                file.path.as_str()
            );
            return Vec::new();
        }
    };
    fmdl::check::check(&model)
        .into_iter()
        .filter(|found| found.code == "fmdl_vertex_far_from_origin")
        .map(|found| {
            finding(
                Code::VertexTooFarFromOrigin,
                vec![
                    ("file", name.to_owned()),
                    ("count", found.count.to_string()),
                ],
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use aesthetics_export::{Disposition, IssueScope};
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

    /// The tracer's boots model with one vertex 6000 units from the origin.
    fn far_boots() -> Vec<u8> {
        fixture("deep/boots_far.fmdl")
    }

    /// The tracer's own boots model, all of it near the origin.
    fn tracer_boots() -> Vec<u8> {
        fixture("tracer/studio/egg Tracer/Players/05 - The Chad Stormworks Player/boots.fmdl")
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
        assert_eq!(
            findings,
            [ContentFinding {
                code: "vertex_too_far_from_origin",
                scope: IssueScope::File(path("Common/legs.fmdl")),
                context: vec![("file", "legs.fmdl".to_owned()), ("count", "1".to_owned())],
                disposition: Disposition::DropFile,
                pass_through_eligible: false,
            }]
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
        assert_eq!(
            findings,
            [ContentFinding {
                code: "vertex_too_far_from_origin",
                scope: IssueScope::Folder(path("Players/03 - A")),
                context: vec![
                    ("file", "face/hair.fmdl".to_owned()),
                    ("count", "1".to_owned())
                ],
                disposition: Disposition::DropFolder,
                pass_through_eligible: false,
            }]
        );
    }

    #[test]
    fn a_clean_export_and_a_model_that_does_not_parse_yield_no_finding() {
        let temp = scratch("deep_clean");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/boots.fmdl", tracer_boots()),
                ("Players/05 - B/boots.fmdl", b"not a model".to_vec()),
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
        let [finding] = findings.as_slice() else {
            panic!("{findings:?}");
        };
        assert_eq!(finding.code, "source_read_failed");
        assert_eq!(finding.scope, IssueScope::Folder(path("Players/03 - A")));
        assert_eq!(finding.disposition, Disposition::DropFolder);
        assert!(!finding.pass_through_eligible);
        let missing: PathBuf = temp.path().join("Players/03 - A/boots.fmdl");
        assert_eq!(finding.context[0], ("path", missing.display().to_string()));
        assert_eq!(finding.context[1].0, "error");
        assert_eq!(finding.context.len(), 2);
    }
}
