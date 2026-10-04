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
//! It also checks every texture of those folders, of `Common/`, of the kits and every portrait
//! from its header alone (`team_compiler/messages.md` "Textures"): a file renamed from another
//! format, a side under one block, and the size rules of a kit's main texture, of a mipmapped
//! Fox texture and of a portrait. A slot whose two portraits, its player folder's and its
//! `Portraits/` file, differ in bytes is `portrait_conflict`, which skips the export. A logo
//! source is the one texture decoded in full: one that does not decode is
//! `logo_file_invalid`.
//!
//! Last, it reads the small data files whole (`documents`): the face diff of each player
//! folder and shared face folder (`face_diff_invalid`, `xml_dif_conflict`), each kit's
//! `config.toml` (`kit_config_invalid`), each player's `settings.toml`
//! (`settings_toml_invalid`), and each kit's and the root `colors.txt`, one
//! `color_entry_invalid` per line the file refuses.
//!
//! The model folders and `Common/`'s files are checked in parallel on the caller's rayon
//! pool, each worker reading and holding one file at a time, and the findings are collected
//! in file order (`content_findings`).

mod documents;
mod model;
mod portrait;
mod texture;

use std::collections::BTreeMap;

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, KitTextureSource,
    ModelFormat, ValidatedAestheticsExport,
};
use dds_convert::SourceFormat;
use pes_version::PesVersion;
use rayon::prelude::*;
use vtree::ScopePath;

use crate::bins::{KIT_COLORS, TEAM_COLORS};
use crate::messages::Code;
use crate::plan::subset::{FolderModels, texture_format};
use crate::reader::ContentSource;
use documents::{colors_findings, face_diff_findings, kit_config_finding, settings_finding};
use model::{ModelKind, fired, summed};
use portrait::{folder_portrait, portrait_conflict, portrait_findings};
use texture::{SizeRule, texture_finding};

pub(crate) use model::FAR_VERTEX_CODES;

/// The content findings of `export`, the sanitized export read from `content` and compiled for
/// `version`, in file order: each player folder's models, material sets and textures, then
/// its face diff, its portrait and its `settings.toml`; then each shared folder's (faces with
/// their face diff, boots, gloves), then `Common/`'s, then each `Portraits/` file with its
/// slot's `portrait_conflict`, then each kit's `config.toml`, `colors.txt` and textures, then
/// the logo's, then the root `colors.txt`'s. An Error on a folder's or a kit's file drops the
/// folder; one on a `Common/` file drops the file, and the cascade then drops the players
/// linking it; one on a portrait, a `settings.toml` or a logo file drops that file. A refused
/// `colors.txt` line is a Warning on the file, which drops nothing.
///
/// The player folders, the shared folders, the files of each folder and `Common/`'s files are
/// checked in parallel, on the rayon pool the caller runs this in; the rest in order. Every
/// file is read through `content` and each worker holds one file's bytes at a time, so at most
/// one file per worker thread is held (a slot's two portraits while they are compared). A
/// solid `.7z` is decompressed once per `content`, on its first read (the caller's metadata
/// read, when the export has a metadata file), and stays charged until `content` is dropped.
pub(crate) fn content_findings(
    export: &ValidatedAestheticsExport,
    content: &ContentSource,
    version: PesVersion,
) -> Vec<ContentFinding> {
    let size_rule = SizeRule::of(version);
    // Each group below is collected in its items' order (rayon's indexed `collect`), so the
    // findings come out in file order whatever the workers' scheduling.
    let players: Vec<Vec<ContentFinding>> = export
        .players
        .par_iter()
        .map(|player| {
            let folder = &player.path;
            let mut findings = folder_findings(content, folder, &player.files, size_rule);
            findings.extend(face_diff_findings(
                content,
                folder,
                &player.files,
                &FolderModels::of_player(player),
            ));
            // Not among the folder's files: a portrait and a `settings.toml` are dropped
            // alone, the folder keeping the rest.
            if let Some(portrait) = &player.portrait {
                findings.extend(portrait_findings(
                    content,
                    portrait,
                    &relative(&portrait.path, folder),
                ));
            }
            findings.extend(settings_finding(content, player));
            findings
        })
        .collect();
    // A shared face folder is part of the face of each player linking it, so its face diff is
    // checked as a player folder's is, against its own models as planning resolves it.
    let faces: Vec<Vec<ContentFinding>> = export
        .faces
        .par_iter()
        .map(|face| {
            let mut findings = folder_findings(content, &face.path, &face.files, size_rule);
            findings.extend(face_diff_findings(
                content,
                &face.path,
                &face.files,
                &FolderModels::of(&face.path, &face.files),
            ));
            findings
        })
        .collect();
    let boots_and_gloves: Vec<Vec<ContentFinding>> = export
        .boots
        .par_iter()
        .chain(&export.gloves)
        .map(|shared| folder_findings(content, &shared.path, &shared.files, size_rule))
        .collect();
    let common: Vec<Vec<ContentFinding>> = export
        .common
        .par_iter()
        .map(|file| {
            let Some(checked) = checked_as(file, size_rule) else {
                return Vec::new();
            };
            file_findings(
                content,
                file,
                checked,
                &IssueScope::File(file.path.clone()),
                Disposition::DropFile,
                file.path.name(),
            )
        })
        .collect();
    let mut findings: Vec<ContentFinding> = [players, faces, boots_and_gloves, common]
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    for (slot, file) in &export.portraits {
        findings.extend(portrait_findings(content, file, file.path.name()));
        if let Some(folder_portrait) = folder_portrait(export, *slot) {
            findings.extend(portrait_conflict(content, folder_portrait, file));
        }
    }
    // An `all/` texture is read once, however many kits inherit it, and its findings are kept
    // by its path; each inheriting kit gets them on its own scope.
    let mut inherited: BTreeMap<&str, Vec<ContentFinding>> = BTreeMap::new();
    for kit in export.kits.kits.values() {
        findings.extend(kit_config_finding(content, kit));
        if let Some(colors) = &kit.colors {
            findings.extend(colors_findings(content, colors, KIT_COLORS));
        }
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
                    content,
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
                            content,
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
    let logo_files = export
        .logo
        .iter()
        .flat_map(|logo| std::iter::once(&logo.main).chain(&logo.small));
    for logo in logo_files {
        let file = &logo.file;
        let format = texture_format(file.path.name())
            .expect("a logo is classified a texture by an extension `dds_convert` accepts");
        findings.extend(file_findings(
            content,
            file,
            Checked::Logo(format),
            &IssueScope::File(file.path.clone()),
            Disposition::DropFile,
            file.path.name(),
        ));
    }
    if let Some(colors) = &export.root.team_colors {
        findings.extend(colors_findings(content, colors, TEAM_COLORS));
    }
    findings
}

/// The structure pass's code for a root `logo*` file that cannot be the team's logo, which the
/// deep pass also reports for a logo source that does not decode.
const LOGO_FILE_INVALID: &str = "logo_file_invalid";

/// What the deep pass reads a file as, and what checks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Checked {
    /// A model or material set, checked by its format crate.
    Model(ModelKind),
    /// A texture in the format its extension names, checked from its header.
    Texture(SourceFormat, SizeRule),
    /// A logo source in the format its extension names, decoded in full: an export has at
    /// most two, and only the decoded image shows that the game's logo sizes can be made
    /// from it.
    Logo(SourceFormat),
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

/// The findings of the files among `files`, those of the model folder at `folder`, that the
/// deep pass reads (`checked_as`, textures held to `size_rule`): each on the folder's scope,
/// an Error dropping the folder, the file named below the folder. The files are checked in
/// parallel.
fn folder_findings(
    content: &ContentSource,
    folder: &ScopePath,
    files: &[FileDescriptor],
    size_rule: SizeRule,
) -> Vec<ContentFinding> {
    // Collected in file order (an indexed `collect`), whatever the scheduling.
    let per_file: Vec<Vec<ContentFinding>> = files
        .par_iter()
        .map(|file| {
            let Some(checked) = checked_as(file, size_rule) else {
                return Vec::new();
            };
            file_findings(
                content,
                file,
                checked,
                &IssueScope::Folder(folder.clone()),
                Disposition::DropFolder,
                &relative(&file.path, folder),
            )
        })
        .collect();
    per_file.into_iter().flatten().collect()
}

/// The bytes of `file`, or, when they cannot be read, its `source_read_failed` on `scope`
/// with `disposition`, never eligible: there is nothing to keep.
fn read(
    content: &ContentSource,
    file: &FileDescriptor,
    scope: &IssueScope,
    disposition: Disposition,
) -> Result<Vec<u8>, ContentFinding> {
    content
        .read(file.source.as_str())
        .map_err(|failure| ContentFinding {
            code: Code::SourceReadFailed.as_str(),
            scope: scope.clone(),
            context: vec![("path", failure.path), ("error", failure.error)],
            disposition,
            pass_through_eligible: false,
        })
}

/// `path` below `folder`, its subfolder kept (`face/hair.fmdl`).
fn relative(path: &ScopePath, folder: &ScopePath) -> String {
    path.segments()
        .skip(folder.segments().count())
        .collect::<Vec<_>>()
        .join("/")
}

/// The findings of `file`, read as `checked`, each on `scope` and naming the file `name`. A
/// model or material set gets one per code its format crate's check fires, with the summed
/// count (an Error with `disposition`, pass-through-eligible unless it is the far vertex; a
/// Warning or an Info `Keep`); one that does not parse is `model_broken` or `mtl_broken`, with
/// `disposition` and never eligible: there is nothing to pack. A texture gets at most one
/// (`texture_finding`), with `disposition`. A logo source that does not decode is
/// `logo_file_invalid`, with `disposition` and never eligible: no logo can be made from it. A
/// file that cannot be read is `source_read_failed`, with `disposition` and never eligible.
fn file_findings(
    content: &ContentSource,
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
    let bytes = match read(content, file, scope, disposition) {
        Ok(bytes) => bytes,
        Err(unread) => return vec![unread],
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
        Checked::Logo(format) => {
            return dds_convert::decode(&bytes, format)
                .err()
                .map(|error| {
                    finding(
                        LOGO_FILE_INVALID,
                        vec![("file", name.to_owned()), ("error", error.to_string())],
                        disposition,
                        false,
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
    use fmdl::{FmdlFile, Model};
    use pipeline::MemoryBudget;
    use studio_core::ExportId;
    use vtree::ScopePath;

    use super::*;
    use crate::reader::{ExportSource, SourceKind};
    use crate::testing::{resolved_with_issues, scratch};

    /// The bytes of `tests/fixtures/<relative>`.
    pub(super) fn fixture(relative: &str) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(relative),
        )
        .unwrap()
    }

    /// The bytes of the tracer player's file `name`.
    pub(super) fn tracer_file(name: &str) -> Vec<u8> {
        fixture(&format!(
            "tracer/studio/egg Tracer/Players/05 - The Chad Stormworks Player/{name}"
        ))
    }

    /// The tracer's boots model with one vertex 6000 units from the origin.
    pub(super) fn far_boots() -> Vec<u8> {
        fixture("deep/boots_far.fmdl")
    }

    /// The tracer's own boots model, all of it near the origin; 1662 of its vertices carry
    /// weights that sum to neither 255 nor 0 (1626, 18 and 18 in its three meshes).
    pub(super) fn tracer_boots() -> Vec<u8> {
        tracer_file("boots.fmdl")
    }

    /// The Fox model `bytes` with `edit` applied, written back through `fmdl`.
    pub(super) fn edited(bytes: &[u8], edit: impl FnOnce(&mut Model)) -> Vec<u8> {
        let mut model = Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap();
        edit(&mut model);
        model.to_file().unwrap().write()
    }

    /// The tracer's right glove, in which `fmdl`'s check finds nothing, with its mesh holding
    /// 21846 faces (its first, repeated), one over the hard limit: one
    /// `fmdl_mesh_over_face_limit`, an Error. (`fmdl` refuses to write a face naming a
    /// vertex that does not exist, or a mesh no group lists.)
    pub(super) fn glove_over_the_face_limit() -> Vec<u8> {
        edited(&tracer_file("glove_r.fmdl"), |model| {
            let mesh = &mut model.meshes[0];
            mesh.faces = vec![mesh.faces[0]; 21_846];
        })
    }

    /// The deep pass for PES 21 over the folder export `co - Deep` at `root`, holding `files`
    /// (path, bytes) and listing `unwritten` too, which is not on disk; the structure pass
    /// finds nothing in it.
    pub(super) fn findings_of(
        root: &Path,
        files: &[(&str, Vec<u8>)],
        unwritten: &[&str],
    ) -> Vec<ContentFinding> {
        findings_for(PesVersion::Pes21, root, files, unwritten, &[])
    }

    /// `findings_of` for PES `version`, the structure pass finding exactly the codes
    /// `structure_codes` (a kit inheriting from `all/` is `kit_textures_inherited`).
    pub(super) fn findings_for(
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
        let content = ContentSource::new(&source, &MemoryBudget::new(1 << 30));
        content_findings(&resolved.export, &content, version)
    }

    /// The bytes of `tests/fixtures/textures/<name>` (that folder's `README.md`).
    pub(super) fn texture(name: &str) -> Vec<u8> {
        fixture(&format!("textures/{name}"))
    }

    /// A single-level BC1 DDS of `width`x`height`, its blocks zero: built from the blocks, so
    /// nothing is encoded.
    pub(super) fn bc1_dds(width: u32, height: u32) -> Vec<u8> {
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
    pub(super) fn texture_finding_on(
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

    pub(super) fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    pub(super) fn folder(text: &str) -> IssueScope {
        IssueScope::Folder(path(text))
    }

    /// The finding `code` on `scope` about the file `file`, `count` items of which tripped
    /// the rule.
    pub(super) fn counted(
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
    fn a_logo_that_does_not_decode_is_logo_file_invalid_on_its_file() {
        let temp = scratch("deep_logo_invalid");
        let findings = findings_of(temp.path(), &[("logo.png", b"not an image".to_vec())], &[]);
        let error = dds_convert::decode(b"not an image", SourceFormat::Png)
            .unwrap_err()
            .to_string();
        assert_eq!(
            findings,
            [ContentFinding {
                code: "logo_file_invalid",
                scope: IssueScope::File(path("logo.png")),
                context: vec![("file", "logo.png".to_owned()), ("error", error)],
                disposition: Disposition::DropFile,
                pass_through_eligible: false,
            }]
        );
        assert!(aesthetics_export::ISSUE_CODES.contains(&"logo_file_invalid"));
        let temp = scratch("deep_logo_valid");
        let findings = findings_of(temp.path(), &[("logo.png", texture("portrait.png"))], &[]);
        assert_eq!(findings, []);
    }

    #[test]
    fn findings_keep_the_file_order_whatever_the_workers_scheduling() {
        let temp = scratch("deep_parallel_order");
        let players = ["03 - A", "05 - B", "07 - C", "09 - D"];
        let files: Vec<(String, Vec<u8>)> = players
            .iter()
            .flat_map(|player| {
                [
                    (format!("Players/{player}/skin.png"), texture("tiny.png")),
                    (format!("Players/{player}/boots.fmdl"), tracer_boots()),
                    (format!("Players/{player}/hair.png"), texture("tiny.png")),
                ]
            })
            .collect();
        let files: Vec<(&str, Vec<u8>)> = files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.clone()))
            .collect();
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();

        let runs: Vec<Vec<ContentFinding>> = (0..20)
            .map(|_| pool.install(|| findings_of(temp.path(), &files, &[])))
            .collect();

        for (i, run) in runs.iter().enumerate() {
            assert_eq!(*run, runs[0], "run {i}");
        }
        let expected: Vec<ContentFinding> = players
            .iter()
            .flat_map(|player| {
                let scope = folder(&format!("Players/{player}"));
                [
                    counted(
                        "fmdl_weights_not_normalized",
                        &scope,
                        "boots.fmdl",
                        1662,
                        Disposition::Keep,
                        false,
                    ),
                    texture_finding_on(
                        "texture_too_small",
                        &scope,
                        "hair.png",
                        Disposition::DropFolder,
                        true,
                    ),
                    texture_finding_on(
                        "texture_too_small",
                        &scope,
                        "skin.png",
                        Disposition::DropFolder,
                        true,
                    ),
                ]
            })
            .collect();
        assert_eq!(runs[0], expected);
    }

    #[test]
    fn portraits_stand_with_their_folder_and_after_common_and_the_logo_last() {
        let temp = scratch("deep_portrait_order");
        let findings = findings_of(
            temp.path(),
            &[
                ("logo.png", b"not an image".to_vec()),
                ("Kits/p1/kit_back.png", texture("tiny.png")),
                ("Kits/p1/kit.png", texture("kit.png")),
                ("Portraits/player_07.png", texture("tiny.png")),
                ("Common/hair.png", texture("tiny.png")),
                ("Players/05 - B/portrait.png", texture("tiny.png")),
                ("Players/03 - A/portrait.png", texture("tiny.png")),
                ("Players/03 - A/skin.png", texture("tiny.png")),
            ],
            &[],
        );
        let order: Vec<(&str, &str)> = findings
            .iter()
            .map(|finding| (finding.code, finding.context[0].1.as_str()))
            .collect();
        assert_eq!(
            order,
            [
                ("texture_too_small", "skin.png"),
                ("texture_too_small", "portrait.png"),
                ("texture_too_small", "portrait.png"),
                ("texture_too_small", "hair.png"),
                ("texture_too_small", "player_07.png"),
                ("texture_too_small", "kit_back.png"),
                ("logo_file_invalid", "logo.png"),
            ]
        );
        assert_eq!(
            findings[1].scope,
            IssueScope::File(path("Players/03 - A/portrait.png"))
        );
        assert_eq!(
            findings[2].scope,
            IssueScope::File(path("Players/05 - B/portrait.png"))
        );
    }
}
