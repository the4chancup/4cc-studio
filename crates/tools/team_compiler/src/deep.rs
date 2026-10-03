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
//!
//! It also checks every texture of those folders, of `Common/` and of the kits from its header
//! alone (`team_compiler/messages.md` "Textures"): a file renamed from another format, a side
//! under one block, and the size rules of a kit's main texture and of a mipmapped Fox
//! texture. The player's portrait is not checked here.

use std::collections::BTreeMap;
use std::sync::Arc;

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, KitTextureSource,
    ModelFormat, ValidatedAestheticsExport,
};
use dds_convert::SourceFormat;
use fmdl::{FmdlFile, Model};
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;
use pes_version::{Engine, PesVersion};
use pipeline::MemoryBudget;
use vtree::ScopePath;

use crate::messages::Code;
use crate::plan::subset::texture_format;
use crate::reader::{ContentSource, ExportSource};

/// The format crates' far-vertex codes, both reported as `vertex_too_far_from_origin`.
pub(crate) const FAR_VERTEX_CODES: [&str; 2] = [
    "fmdl_vertex_far_from_origin",
    "model_vertex_far_from_origin",
];

/// The content findings of `export`, the sanitized export read from `source` and compiled for
/// `version`, in file order: each player folder's models, material sets and textures, then
/// each shared folder's (faces, boots, gloves), then `Common/`'s, then each kit's textures. An
/// Error on a folder's or a kit's file drops the folder; one on a `Common/` file drops the
/// file, and the cascade then drops the players linking it. Files are read one at a time
/// through one `ContentSource`, so only one file's bytes are held at once, and a solid `.7z`
/// is decompressed once, under its own permit from `budget`, released when the pass ends.
pub(crate) fn content_findings(
    export: &ValidatedAestheticsExport,
    source: &ExportSource,
    budget: &Arc<MemoryBudget>,
    version: PesVersion,
) -> Vec<ContentFinding> {
    let mut content = ContentSource::new(source, budget);
    let size_rule = SizeRule::of(version);
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
            let Some(checked) = checked_as(file, size_rule) else {
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
        let Some(checked) = checked_as(file, size_rule) else {
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
    // An `all/` texture is read once, however many kits inherit it, and its findings are kept
    // by its path; each inheriting kit gets them on its own scope.
    let mut inherited: BTreeMap<&str, Vec<ContentFinding>> = BTreeMap::new();
    for kit in export.kits.kits.values() {
        let scope = IssueScope::Folder(kit.path.clone());
        for texture in &kit.textures {
            let rule = if texture.stem == "kit" {
                SizeRule::MainKit
            } else {
                size_rule
            };
            let Some(checked) = checked_as(&texture.file, rule) else {
                continue;
            };
            match texture.source {
                KitTextureSource::Own => findings.extend(file_findings(
                    &mut content,
                    &texture.file,
                    checked,
                    &scope,
                    Disposition::DropFolder,
                    &relative(&texture.file.path, &kit.path),
                )),
                KitTextureSource::Shared => {
                    // Named by its path in the export, so the member sees it is not in the
                    // kit's own folder.
                    let path = texture.file.path.as_str();
                    let found = inherited.entry(path).or_insert_with(|| {
                        file_findings(
                            &mut content,
                            &texture.file,
                            checked,
                            &scope,
                            Disposition::DropFolder,
                            path,
                        )
                    });
                    findings.extend(found.iter().map(|finding| ContentFinding {
                        scope: scope.clone(),
                        ..finding.clone()
                    }));
                }
            }
        }
    }
    findings
}

/// What the deep pass reads a file as, and what checks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Checked {
    /// A model or material set, checked by its format crate.
    Model(ModelKind),
    /// A texture in the format its extension names, checked from its header.
    Texture(SourceFormat, SizeRule),
}

/// Which format crate checks a model or material set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelKind {
    /// A Fox model, checked by `fmdl`.
    Fmdl,
    /// A pre-Fox model, checked by `pes_model`.
    PreFoxModel,
    /// A pre-Fox material set, checked by `pes_model`.
    Mtl,
}

/// The size rule a texture is held to past the smallest side (`messages.md` "Textures").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SizeRule {
    /// A kit's main texture, on any target: `kit_texture_too_big` past 2048 pixels a side or
    /// on a side that is not a power of two.
    MainKit,
    /// Any other texture on a Fox target: `texture_not_pow2` on a side that is not a power of
    /// two when the converted texture carries a mip chain.
    FoxMipmapped,
    /// Any other texture on a pre-Fox target: no size rule here.
    PreFox,
}

impl SizeRule {
    /// The rule of every texture but a kit's main one, on `version`'s engine.
    fn of(version: PesVersion) -> SizeRule {
        match version.engine() {
            Engine::Fox => SizeRule::FoxMipmapped,
            Engine::PreFox => SizeRule::PreFox,
        }
    }
}

/// What `file` is read as, a texture held to `size_rule`, or `None` for a file the deep pass
/// does not read.
fn checked_as(file: &FileDescriptor, size_rule: SizeRule) -> Option<Checked> {
    match file.kind {
        FileKind::Model(ModelFormat::Fmdl) => Some(Checked::Model(ModelKind::Fmdl)),
        FileKind::Model(ModelFormat::PesModel) => Some(Checked::Model(ModelKind::PreFoxModel)),
        FileKind::Mtl => Some(Checked::Model(ModelKind::Mtl)),
        FileKind::Texture => {
            let format = texture_format(file.path.name())
                .expect("a texture is classified by an extension `dds_convert` accepts");
            Some(Checked::Texture(format, size_rule))
        }
        // glTF is read from Phase 7; nothing else is checked.
        FileKind::Model(ModelFormat::Gltf)
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

/// The rules the format crate's check fires on `bytes`, read as `kind`, or the reader's error
/// text when they do not parse.
fn fired(kind: ModelKind, bytes: &[u8]) -> Result<Vec<Fired>, String> {
    let fired = match kind {
        ModelKind::Fmdl => {
            let model = FmdlFile::read(bytes)
                .and_then(|file| Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            fmdl::check::check(&model)
                .into_iter()
                .map(Fired::fox)
                .collect()
        }
        ModelKind::PreFoxModel => {
            let model = PreFoxModel::read(bytes)
                .and_then(|file| pes_model::model::Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            pes_model::check::check(&model)
                .into_iter()
                .map(Fired::pre_fox)
                .collect()
        }
        ModelKind::Mtl => {
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

/// The accepted formats that open with a fixed signature, and that signature; WebP's `RIFF`
/// is checked with its `WEBP` tag below, and TGA has none.
const SIGNATURES: [(&[u8], SourceFormat); 7] = [
    (b"DDS ", SourceFormat::Dds),
    (b"FTEX", SourceFormat::Ftex),
    (b"\x89PNG", SourceFormat::Png),
    (&[0xff, 0xd8, 0xff], SourceFormat::Jpeg),
    (b"BM", SourceFormat::Bmp),
    (b"II*\0", SourceFormat::Tiff),
    (b"MM\0*", SourceFormat::Tiff),
];

/// The accepted format whose signature `bytes` open with, if any.
pub(crate) fn signature_format(bytes: &[u8]) -> Option<SourceFormat> {
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some(SourceFormat::WebP);
    }
    SIGNATURES
        .iter()
        .find(|(signature, _)| bytes.starts_with(signature))
        .map(|(_, format)| *format)
}

/// The one texture finding `bytes`, a texture in `format` held to `rule`, get, by the first
/// rule that fires: `texture_type_mismatch` when they open with another accepted format's
/// signature (a file renamed, not resaved; its header is not read), then, from the header,
/// `texture_too_small` on a side under 4 pixels (one block), then the size rule. A header
/// that cannot be read is no finding: converting the texture fails its task.
fn texture_finding(format: SourceFormat, rule: SizeRule, bytes: &[u8]) -> Option<Code> {
    if signature_format(bytes).is_some_and(|sniffed| sniffed != format) {
        return Some(Code::TextureTypeMismatch);
    }
    // The error is not reported here: the texture's task meets it again and fails on it.
    let Ok(probe) = dds_convert::probe(bytes, format) else {
        return None;
    };
    if probe.width < 4 || probe.height < 4 {
        return Some(Code::TextureTooSmall);
    }
    let power_of_two = probe.width.is_power_of_two() && probe.height.is_power_of_two();
    match rule {
        SizeRule::MainKit => (probe.width > 2048 || probe.height > 2048 || !power_of_two)
            .then_some(Code::KitTextureTooBig),
        SizeRule::FoxMipmapped => {
            (probe.mipmaps > 1 && !power_of_two).then_some(Code::TextureNotPow2)
        }
        SizeRule::PreFox => None,
    }
}

/// The findings of `file`, read as `checked`, each on `scope` and naming the file `name`. A
/// model or material set gets one per code its format crate's check fires, with the summed
/// count (an Error with `disposition`, pass-through-eligible unless it is the far vertex; a
/// Warning or an Info `Keep`); one that does not parse is `model_broken` or `mtl_broken`, with
/// `disposition` and never eligible: there is nothing to pack. A texture gets at most one
/// (`texture_finding`), with `disposition`. A file that cannot be read is
/// `source_read_failed`, with `disposition` and never eligible.
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
    let kind = match checked {
        Checked::Model(kind) => kind,
        Checked::Texture(format, rule) => {
            return texture_finding(format, rule, &bytes)
                .map(|code| {
                    // A renamed file cannot be converted as the format its name declares; a
                    // texture of an odd size converts, and what the game makes of it is the
                    // member's risk.
                    let eligible = code != Code::TextureTypeMismatch;
                    finding(
                        code.as_str(),
                        vec![("file", name.to_owned())],
                        disposition,
                        eligible,
                    )
                })
                .into_iter()
                .collect();
        }
    };
    let fired = match fired(kind, &bytes) {
        Ok(fired) => fired,
        Err(error) => {
            let broken = match kind {
                ModelKind::Fmdl | ModelKind::PreFoxModel => Code::ModelBroken,
                ModelKind::Mtl => Code::MtlBroken,
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
    use dds_convert::{BlockCodec, Blocks, Decoded, encode_dds};
    use pes_model::format::PreFoxModel;
    use pes_model::format::mtl::MaterialSet;
    use studio_core::ExportId;
    use vtree::ScopePath;

    use super::*;
    use crate::reader::SourceKind;
    use crate::testing::{resolved_with_issues, scratch};

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

    /// The deep pass for PES 21 over the folder export `co - Deep` at `root`, holding `files`
    /// (path, bytes) and listing `unwritten` too, which is not on disk; the structure pass
    /// finds nothing in it.
    fn findings_of(
        root: &Path,
        files: &[(&str, Vec<u8>)],
        unwritten: &[&str],
    ) -> Vec<ContentFinding> {
        findings_for(PesVersion::Pes21, root, files, unwritten, &[])
    }

    /// `findings_of` for PES `version`, the structure pass finding exactly the codes
    /// `structure_codes` (a kit inheriting from `all/` is `kit_textures_inherited`).
    fn findings_for(
        version: PesVersion,
        root: &Path,
        files: &[(&str, Vec<u8>)],
        unwritten: &[&str],
        structure_codes: &[&str],
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
        let (resolved, codes) = resolved_with_issues("co - Deep", &listed, &[], None);
        assert_eq!(codes, structure_codes, "the structure pass's findings");
        let source = ExportSource {
            export_id: ExportId(0),
            path: root.to_path_buf(),
            kind: SourceKind::Folder,
            file_name: "co - Deep".to_owned(),
            display_name: "co - Deep".to_owned(),
            team_name: None,
        };
        content_findings(
            &resolved.export,
            &source,
            &MemoryBudget::new(1 << 30),
            version,
        )
    }

    /// The bytes of `tests/fixtures/textures/<name>` (that folder's `README.md`).
    fn texture(name: &str) -> Vec<u8> {
        fixture(&format!("textures/{name}"))
    }

    /// A single-level BC1 DDS of `width`x`height`, its blocks zero: built from the blocks, so
    /// nothing is encoded.
    fn bc1_dds(width: u32, height: u32) -> Vec<u8> {
        let blocks = width.div_ceil(4) * height.div_ceil(4) * 8;
        let decoded = Decoded {
            width,
            height,
            mips: vec![vec![0; (width * height * 4) as usize]],
            blocks: Some(Blocks {
                codec: BlockCodec::Bc1,
                mips: vec![vec![0; blocks as usize]],
            }),
            authored_mips: true,
        };
        encode_dds(&decoded, BlockCodec::Bc1).unwrap()
    }

    /// The texture finding `code` on `scope` about the file `file`.
    fn texture_finding_on(
        code: &'static str,
        scope: &IssueScope,
        file: &str,
        disposition: Disposition,
        pass_through_eligible: bool,
    ) -> ContentFinding {
        ContentFinding {
            code,
            scope: scope.clone(),
            context: vec![("file", file.to_owned())],
            disposition,
            pass_through_eligible,
        }
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
    fn the_tracer_s_texture_passes_and_files_that_are_not_checked_are_not_read() {
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

    #[test]
    fn each_accepted_format_s_signature_is_recognized_and_tga_has_none() {
        // A real file of each format opens with its signature (the kit and portrait fixtures
        // are real encoder output); a signature pasted onto nothing is still that format's.
        assert_eq!(
            signature_format(&fixture("tracer/studio/egg Tracer/Kits/g1/kit.dds")),
            Some(SourceFormat::Dds),
            "dds"
        );
        assert_eq!(
            signature_format(&texture("kit.png")),
            Some(SourceFormat::Png),
            "png"
        );
        assert_eq!(
            signature_format(&texture("portrait.webp")),
            Some(SourceFormat::WebP),
            "webp"
        );
        assert_eq!(
            signature_format(&texture("kit_back.tga")),
            None,
            "tga has no signature"
        );
        for (bytes, format) in [
            (&b"FTEX\x00\x00\x00\x00"[..], SourceFormat::Ftex),
            (&[0xff, 0xd8, 0xff, 0xe0, 0, 0x10], SourceFormat::Jpeg),
            (&b"BM\x36\x00\x00\x00"[..], SourceFormat::Bmp),
            (&b"II*\0\x08\x00\x00\x00"[..], SourceFormat::Tiff),
            (&b"MM\0*\x00\x00\x00\x08"[..], SourceFormat::Tiff),
            (&b"RIFF\x00\x00\x00\x00WEBPVP8 "[..], SourceFormat::WebP),
        ] {
            assert_eq!(signature_format(bytes), Some(format), "{format:?}");
        }
        // A RIFF that is not WebP, and bytes opening with none of them, are no format.
        assert_eq!(signature_format(b"RIFF\x00\x00\x00\x00WAVEfmt "), None);
        assert_eq!(signature_format(b"RIFF"), None, "too short for the tag");
        assert_eq!(signature_format(b"not a texture"), None);
        assert_eq!(signature_format(b""), None);
    }

    #[test]
    fn each_texture_rule_drops_its_player_folder_with_one_finding() {
        let temp = scratch("deep_texture_rules");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/tiny.png", texture("tiny.png")),
                ("Players/04 - B/face/odd.png", texture("odd.png")),
                // PNG bytes under a `.dds` and a `.tga` name: renamed, not resaved; TGA has
                // no signature, but PNG's is another's.
                ("Players/05 - C/skin.dds", texture("portrait.png")),
                ("Players/06 - D/skin.tga", texture("tiny.png")),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [
                texture_finding_on(
                    "texture_too_small",
                    &folder("Players/03 - A"),
                    "tiny.png",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "texture_not_pow2",
                    &folder("Players/04 - B"),
                    "face/odd.png",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "texture_type_mismatch",
                    &folder("Players/05 - C"),
                    "skin.dds",
                    Disposition::DropFolder,
                    false
                ),
                texture_finding_on(
                    "texture_type_mismatch",
                    &folder("Players/06 - D"),
                    "skin.tga",
                    Disposition::DropFolder,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_single_level_odd_dds_on_fox_and_an_odd_png_on_pre_fox_pass() {
        let temp = scratch("deep_texture_fox_single_level");
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/skin.dds", bc1_dds(300, 300))],
            &[],
        );
        assert_eq!(findings, []);
        let temp = scratch("deep_texture_pre_fox");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[("Players/03 - A/odd.png", texture("odd.png"))],
            &[],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn a_texture_whose_header_is_cut_gets_no_finding() {
        let temp = scratch("deep_texture_cut");
        let findings = findings_of(
            temp.path(),
            &[
                (
                    "Players/03 - A/skin.dds",
                    texture("bc7.dds")[..100].to_vec(),
                ),
                (
                    "Players/03 - A/hair.png",
                    texture("tiny.png")[..20].to_vec(),
                ),
            ],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn a_common_texture_with_a_finding_drops_the_file() {
        let temp = scratch("deep_texture_common");
        let findings = findings_of(
            temp.path(),
            &[
                ("Common/tiny.png", texture("tiny.png")),
                ("Common/hair.png", texture("kit.png")),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [texture_finding_on(
                "texture_too_small",
                &IssueScope::File(path("Common/tiny.png")),
                "tiny.png",
                Disposition::DropFile,
                true
            )]
        );
    }

    #[test]
    fn a_kit_s_main_texture_too_big_or_odd_is_kit_texture_too_big_alone() {
        let temp = scratch("deep_texture_kits");
        let findings = findings_of(
            temp.path(),
            &[
                ("Kits/p1/kit.dds", bc1_dds(4096, 4096)),
                ("Kits/p2/kit.png", texture("odd.png")),
                ("Kits/g1/kit.dds", bc1_dds(2048, 2048)),
                ("Kits/p3/kit.png", texture("kit.png")),
                ("Kits/p3/kit_back.png", texture("odd.png")),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [
                texture_finding_on(
                    "kit_texture_too_big",
                    &folder("Kits/p1"),
                    "kit.dds",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "kit_texture_too_big",
                    &folder("Kits/p2"),
                    "kit.png",
                    Disposition::DropFolder,
                    true
                ),
                texture_finding_on(
                    "texture_not_pow2",
                    &folder("Kits/p3"),
                    "kit_back.png",
                    Disposition::DropFolder,
                    true
                ),
            ]
        );
    }

    #[test]
    fn an_all_texture_s_finding_goes_to_each_kit_inheriting_it_naming_its_path() {
        let temp = scratch("deep_texture_all");
        let findings = findings_for(
            PesVersion::Pes21,
            temp.path(),
            &[
                ("Kits/all/kit_back.png", texture("tiny.png")),
                ("Kits/p1/kit.png", texture("kit.png")),
                ("Kits/p2/kit.png", texture("kit.png")),
                ("Kits/g1/kit.png", texture("kit.png")),
                ("Kits/g1/kit_back.png", texture("kit.png")),
            ],
            &[],
            &["kit_textures_inherited", "kit_textures_inherited"],
        );
        assert_eq!(
            findings,
            ["Kits/p1", "Kits/p2"].map(|kit| texture_finding_on(
                "texture_too_small",
                &folder(kit),
                "Kits/all/kit_back.png",
                Disposition::DropFolder,
                true
            ))
        );
    }

    #[test]
    fn each_size_rule_reads_each_side_on_its_own() {
        // Single-level DDS headers, so only the size matters: one side past a bound and the
        // other not, and a side exactly at a bound.
        for (rule, width, height, expected) in [
            (SizeRule::FoxMipmapped, 3, 8, Some(Code::TextureTooSmall)),
            (SizeRule::FoxMipmapped, 8, 3, Some(Code::TextureTooSmall)),
            (SizeRule::FoxMipmapped, 4, 8, None),
            (SizeRule::FoxMipmapped, 8, 4, None),
            (SizeRule::MainKit, 256, 300, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 300, 256, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 4096, 2048, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 2048, 4096, Some(Code::KitTextureTooBig)),
            (SizeRule::MainKit, 2048, 1024, None),
            (SizeRule::PreFox, 3, 8, Some(Code::TextureTooSmall)),
        ] {
            assert_eq!(
                texture_finding(SourceFormat::Dds, rule, &bc1_dds(width, height)),
                expected,
                "{rule:?} {width}x{height}"
            );
        }
    }

    #[test]
    fn the_player_s_portrait_is_not_checked_here() {
        let temp = scratch("deep_texture_portrait");
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/portrait.png", texture("tiny.png"))],
            &[],
        );
        assert_eq!(findings, []);
    }
}
