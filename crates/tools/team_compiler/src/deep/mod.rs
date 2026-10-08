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
//! `model_broken` or `mtl_broken`. A `.model`, or a `.common` link to one, for which no `.mtl`
//! is found (`mtl_search`) has every material undefined, `model_material_undefined`, for PES
//! 2015 to 2017: the folder is dropped, `pass_through` or not. One whose `.mtl` lacks a
//! material it binds is `model_material_undefined` too, naming those materials, and may pass
//! through. glTF models are not read (Phase 7).
//!
//! It also checks every texture of those folders, of `Common/`, of the kits and every portrait
//! from its header alone (`team_compiler/messages.md` "Textures"): a file renamed from another
//! format, a side under one block, and the size rules of a kit's main texture, of a mipmapped
//! Fox texture and of a portrait. A slot whose two portraits, its player folder's and its
//! `Portraits/` file, differ in bytes is `portrait_conflict`, which skips the export. A logo
//! source is the one texture decoded in full: one that does not decode is
//! `logo_file_invalid`.
//!
//! Each `Collars/` file's name must give a stock collar of the target version that a kit
//! config can name, and no collar the suite holds itself (`collar`); a collar model is then
//! checked as any model, its Errors dropping the file.
//!
//! For PES 2015 to 2017 it also reads a face folder's own `face.xml` (`user_face_xml`) and
//! reports its content checks (`team_compiler/messages.md` "XML/MTL content checks"); the xml
//! then decides which `.mtl` a model's `model_material_undefined` compares with: the one its
//! entry names, for the models it lists, instead of the one the search finds.
//!
//! Last, it reads the small data files whole (`documents`): the face diff of each player
//! folder and shared face folder (`face_diff_invalid`, `xml_dif_conflict`), each kit's
//! `config.toml` (`kit_config_invalid`), each player's `settings.toml`
//! (`settings_toml_invalid`), and each kit's and the root `colors.txt`, one
//! `color_entry_invalid` per line the file refuses.
//!
//! The model folders and `Common/`'s files are checked in parallel on the caller's rayon
//! pool, each worker reading and holding one file at a time, and the findings are collected
//! in file order (`content_findings`). Each model it parses (`.fmdl`, `.model`) is also asked,
//! on the same parse, whether its vertices carry hand weights, which planning needs for the
//! hand auto-split.

pub(crate) mod collar;
mod documents;
mod model;
mod portrait;
mod texture;

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, KitTextureSource,
    ModelFormat, PlayerFolder, SharedKind, ValidatedAestheticsExport, common_link_name,
};
use dds_convert::SourceFormat;
use pes_version::{Engine, PesVersion};
use rayon::prelude::*;
use vtree::ScopePath;

use crate::bins::{KIT_COLORS, TEAM_COLORS};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::plan::subset::{
    FolderModels, PlayerFile, common_file, is_direct_common_file, linked_folder, player_file,
    texture_format,
};
use crate::reader::ContentSource;
use crate::user_face_xml::{
    self, Child, FaceFiles, UserFaceXml, XmlError, XmlFinding, reference, resolve,
};
use collar::collar_findings;
use documents::{colors_findings, face_diff_findings, kit_config_findings, settings_finding};
use model::{ModelKind, fired, summed};
use portrait::{folder_portrait, portrait_conflict, portrait_findings};
use texture::{SizeRule, texture_finding};

pub(crate) use model::FAR_VERTEX_CODES;

/// What the deep pass found in one export (`content_findings`).
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ContentPass {
    /// The content findings, in file order.
    pub(crate) findings: Vec<ContentFinding>,
    /// The export paths of the models it parsed (`.fmdl`, `.model`; a player folder's, a shared
    /// folder's or a `Common/` one) whose vertices carry a positive weight on a hand-skeleton bone, whatever
    /// their role and the target: planning decides which of them the hand auto-split takes
    /// (`pipeline.md` "2. Per-export serial steps", step 6). A file that does not parse is
    /// not among them.
    pub(crate) hand_weighted: BTreeSet<ScopePath>,
    /// The export path of each pre-Fox `.model` and `.mtl` it parsed, with its material names
    /// (`model::ModelRead::materials`), which a model's `model_material_undefined` compares.
    /// A file that does not parse is not among them.
    materials: BTreeMap<ScopePath, Vec<String>>,
}

impl ContentPass {
    /// The pass of a file that gives `findings`, no weighted model and no material names.
    fn findings_only(findings: Vec<ContentFinding>) -> ContentPass {
        ContentPass {
            findings,
            ..ContentPass::default()
        }
    }

    /// `other`'s findings after this pass's, and its weighted models and material names with
    /// this pass's.
    fn append(&mut self, other: ContentPass) {
        self.findings.extend(other.findings);
        self.hand_weighted.extend(other.hand_weighted);
        self.materials.extend(other.materials);
    }
}

/// The `Common/` files the deep pass keeps, which a model's `.mtl` search looks among, with the
/// material names of those that are parsed pre-Fox models and material sets
/// (`ContentPass::materials`).
#[derive(Debug, Default)]
struct KeptCommon {
    /// The kept files, in `Common/`'s order.
    files: Vec<FileDescriptor>,
    /// The material names of the kept pre-Fox models and material sets, by export path.
    materials: BTreeMap<ScopePath, Vec<String>>,
}

/// The content findings of `export`, the sanitized export read from `content` and compiled for
/// `version`, with the models among its files that carry hand weights (`ContentPass`). The
/// findings come in file order: each player folder's models, material sets and textures, then
/// its face diff, its portrait and its `settings.toml`; then each shared folder's (faces with
/// their face diff, boots, gloves), then `Common/`'s, then each `Collars/` file's, then each
/// `Portraits/` file with its slot's `portrait_conflict`, then each kit's `config.toml`,
/// `colors.txt` and textures, then the logo's, then the root `colors.txt`'s. An Error on a
/// folder's or a kit's file drops the folder; one on a `Common/` file drops the file, and the
/// cascade then drops the players linking it; one on a collar, a portrait, a `settings.toml` or
/// a logo file drops that file. A refused
/// `colors.txt` line is a Warning on the file, which drops nothing. `Common/`'s files are
/// checked before the folders, whose models' `.mtl` search sees only the ones kept, and whose
/// models' materials are compared with the kept ones' names (`KeptCommon`).
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
) -> ContentPass {
    let size_rule = SizeRule::of(version);
    let engine = version.engine();
    // Each group below is collected in its items' order (rayon's indexed `collect`), so the
    // findings come out in file order whatever the workers' scheduling.
    let common: Vec<ContentPass> = export
        .common
        .par_iter()
        .map(|file| {
            let Some(checked) = checked_as(file, size_rule) else {
                return ContentPass::default();
            };
            file_outcome(
                content,
                file,
                checked,
                &IssueScope::File(file.path.clone()),
                Disposition::DropFile,
                file.path.name(),
            )
        })
        .collect();
    // A model's `.mtl` search looks only among the `Common/` files this pass keeps: planning
    // sees the export after the drops, so the face task searches the same files, and a model
    // whose only `.mtl` is a dropped Common one is `model_material_undefined` here rather than
    // a material the face task cannot find. A file under `pass_through` that an eligible
    // Error would keep is left out too: the pass does not know the setting, and leaving it
    // out can only report a folder `compile` could have built, never let one through that
    // it cannot.
    let mut kept_common = KeptCommon::default();
    for (file, pass) in export.common.iter().zip(&common) {
        let dropped = pass
            .findings
            .iter()
            .any(|finding| matches!(finding.disposition, Disposition::DropFile));
        if dropped {
            continue;
        }
        kept_common.files.push(file.clone());
        if let Some(names) = pass.materials.get(&file.path) {
            kept_common
                .materials
                .insert(file.path.clone(), names.clone());
        }
    }
    let players: Vec<ContentPass> = export
        .players
        .par_iter()
        .map(|player| {
            let folder = &player.path;
            // Under the marker the face files are not used, his own `face.xml` among them.
            let face = (!player.ingame_face).then(|| linked_face_files(export, player));
            let mut pass = folder_findings(
                content,
                folder,
                &player.files,
                &kept_common,
                size_rule,
                version,
                face,
            );
            let findings = &mut pass.findings;
            findings.extend(face_diff_findings(
                content,
                folder,
                &player.files,
                &FolderModels::of_player(player, engine),
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
            pass
        })
        .collect();
    // A shared face folder is part of the face of each player linking it, so its face diff is
    // checked as a player folder's is, against its own models as planning resolves it.
    let faces: Vec<ContentPass> = export
        .faces
        .par_iter()
        .map(|face| {
            // A shared face links no other.
            let mut pass = folder_findings(
                content,
                &face.path,
                &face.files,
                &kept_common,
                size_rule,
                version,
                Some(&[]),
            );
            pass.findings.extend(face_diff_findings(
                content,
                &face.path,
                &face.files,
                &FolderModels::of(&face.path, &face.files, engine),
            ));
            pass
        })
        .collect();
    let boots_and_gloves: Vec<ContentPass> = export
        .boots
        .par_iter()
        .chain(&export.gloves)
        .map(|shared| {
            // A boots or gloves folder has no face.
            folder_findings(
                content,
                &shared.path,
                &shared.files,
                &kept_common,
                size_rule,
                version,
                None,
            )
        })
        .collect();
    let mut pass = ContentPass::default();
    for group in [players, faces, boots_and_gloves, common] {
        for found in group {
            pass.append(found);
        }
    }
    let findings = &mut pass.findings;
    for file in &export.collars {
        findings.extend(collar_findings(content, file, version));
    }
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
        findings.extend(kit_config_findings(content, kit, version));
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
    pass
}

/// The structure pass's code for a root `logo*` file that cannot be the team's logo, which the
/// deep pass also reports for a logo source that does not decode.
const LOGO_FILE_INVALID: &str = "logo_file_invalid";

/// `pes_model`'s code for a model material with no entry in the `.mtl` the model uses, which
/// the deep pass reports for a `.model` whose `.mtl` (`mtl_search::mtl_for`) lacks a material
/// it binds, and for one no `.mtl` is found for: every one of its materials is undefined
/// (`material_finding`).
const MODEL_MATERIAL_UNDEFINED: &str = "model_material_undefined";

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

/// The files of the shared face folder `player` links in `export`, as planning resolves the
/// link (`linked_folder`); empty when he links none.
fn linked_face_files<'a>(
    export: &'a ValidatedAestheticsExport,
    player: &PlayerFolder,
) -> &'a [FileDescriptor] {
    player
        .links
        .iter()
        .find(|link| matches!(link.kind, SharedKind::Face))
        .and_then(|link| linked_folder(export, link))
        .map_or(&[], |face| face.files.as_slice())
}

/// The findings of the files among `files`, those of the model folder at `folder`, that the
/// deep pass reads (`checked_as`, textures held to `size_rule`): each on the folder's scope,
/// an Error dropping the folder, the file named below the folder; when the target `version`
/// is pre-Fox, each `.model`'s, and each typed `.common` link's to one,
/// `model_material_undefined` (`material_finding`, its `.mtl` searched among the folder's
/// files and `common`'s), right after the model's own findings; with the models among them
/// that carry hand weights and the material names of the pre-Fox ones. The files are read and
/// checked in parallel, each worker holding one file, and the models' material names compared
/// after, from the names each read kept.
///
/// `face` is `Some` when the folder's face files are used (a player folder without
/// `ingame_face`, or a shared face folder), holding the files of the shared face the folder
/// links (empty when none). Then each member's own `face.xml` among `files`
/// (`PlayerFile::FaceXml`) is read and checked (`user_xml_findings`), its findings at its place
/// in file order, and the xml overrides the search: `model_material_undefined` compares only
/// the models it lists with the `.mtl` each entry names (`listed_materials`), and none at all
/// when an xml has an Error, which drops the folder.
fn folder_findings(
    content: &ContentSource,
    folder: &ScopePath,
    files: &[FileDescriptor],
    common: &KeptCommon,
    size_rule: SizeRule,
    version: PesVersion,
    face: Option<&[FileDescriptor]>,
) -> ContentPass {
    let engine = version.engine();
    let models = FolderModels::of(folder, files, engine);
    let scope = IssueScope::Folder(folder.clone());
    let xmls: Vec<(&FileDescriptor, XmlOutcome)> = match face {
        Some(linked_face) => {
            let face_files = FaceFiles {
                own: files,
                linked_face,
                common: &common.files,
                folder,
            };
            files
                .iter()
                .filter(|file| player_file(folder, file, &models) == Some(PlayerFile::FaceXml))
                .map(|file| {
                    let outcome = user_xml_findings(content, file, &face_files, version, &scope);
                    (file, outcome)
                })
                .collect()
        }
        None => Vec::new(),
    };
    // With an xml, the models it lists and the `.mtl` each entry names; `None` when there is
    // no xml, and the search pairs each model.
    let listed = (!xmls.is_empty()).then(|| listed_materials(&xmls, files, common, folder));
    // Collected in file order (an indexed `collect`), whatever the scheduling.
    let mut per_file: Vec<ContentPass> = files
        .par_iter()
        .map(|file| match checked_as(file, size_rule) {
            Some(checked) => file_outcome(
                content,
                file,
                checked,
                &scope,
                Disposition::DropFolder,
                &relative(&file.path, folder),
            ),
            None => ContentPass::default(),
        })
        .collect();
    let mut materials = BTreeMap::new();
    for found in &mut per_file {
        materials.append(&mut found.materials);
    }
    let mut pass = ContentPass::default();
    for (file, found) in files.iter().zip(per_file) {
        pass.append(found);
        if let Some((_, outcome)) = xmls.iter().find(|(xml, _)| xml.path == file.path) {
            pass.findings.extend(outcome.findings.iter().cloned());
        }
        if let Some(listed) = &listed {
            for (_, mtl) in listed.iter().filter(|(model, _)| model.path == file.path) {
                pass.findings.extend(undefined_materials(
                    relative(&file.path, folder),
                    &file.path,
                    mtl,
                    folder,
                    common,
                    &materials,
                    &scope,
                ));
            }
            continue;
        }
        // On Fox a `.model` is not read yet: a `boots.model` beside `boots.fmdl` is never
        // the selected source, so dropping the folder for it would lose a working FMDL
        // (4.17 adds Fox where it is the source).
        let searched = match engine {
            Engine::Fox => false,
            // The roles are read without the `ingame_face` marker (`FolderModels::of`), so a
            // model link is `PreFoxCommonModel` here even in a marked folder, where planning
            // makes it a part of his boots or gloves whose `.mtl` is needed all the same.
            Engine::PreFox => {
                file.kind == FileKind::Model(ModelFormat::PesModel)
                    || matches!(
                        player_file(folder, file, &models),
                        Some(PlayerFile::PreFoxCommonModel { .. })
                    )
            }
        };
        if searched {
            pass.findings.extend(material_finding(
                file, folder, files, common, &materials, &scope,
            ));
        }
    }
    pass.materials = materials;
    pass
}

/// `model_material_undefined` on `scope` for `file`, a `.model` among `files` (those of the
/// model folder at `folder`) or a typed `.common` link among them to a `Common/` one, when the
/// target is pre-Fox; `folder_materials` are the material names of the folder's parsed
/// pre-Fox files, `common`'s of the kept `Common/` ones (`ContentPass::materials`).
///
/// When its search (`mtl_search::mtl_for`) finds no `.mtl`, the finding names the model and is
/// not pass-through-eligible: the face's `face.xml` must name a material set for the model,
/// and there is none to name. When it finds one, the material names the model binds (a link's:
/// the linked `Common/` model's) that the `.mtl` does not define are the finding's, in the
/// model's order, naming the model, the `.mtl` (below the folder, or by its export path in
/// `Common/`) and those names; it is pass-through-eligible: the file packs as it is, and the
/// game renders those meshes with its fallback material. No finding when every name is
/// defined, or when either file did not parse (its own `model_broken` or `mtl_broken` drops
/// it).
fn material_finding(
    file: &FileDescriptor,
    folder: &ScopePath,
    files: &[FileDescriptor],
    common: &KeptCommon,
    folder_materials: &BTreeMap<ScopePath, Vec<String>>,
    scope: &IssueScope,
) -> Option<ContentFinding> {
    let name = relative(&file.path, folder);
    let Some(mtl) = mtl_for(&file.path, folder, files, &common.files) else {
        return Some(ContentFinding {
            code: MODEL_MATERIAL_UNDEFINED,
            scope: scope.clone(),
            context: vec![("file", name)],
            disposition: Disposition::DropFolder,
            pass_through_eligible: false,
        });
    };
    let model = match common_link_name(file.path.name()) {
        Some(linked) => &common_file(&common.files, &linked)?.path,
        None => &file.path,
    };
    undefined_materials(name, model, mtl, folder, common, folder_materials, scope)
}

/// `model_material_undefined` on `scope` for the model at `model`, which the finding names
/// `name`, paired with `mtl`: the material names the model binds that the `.mtl` does not
/// define, in the model's order, naming the `.mtl` below `folder` or by its export path in
/// `Common/`; pass-through-eligible, the file packing as it is. `None` when every name is
/// defined, or when either file did not parse. `folder_materials` and `common`'s are the
/// material names of the parsed pre-Fox files (`ContentPass::materials`).
fn undefined_materials(
    name: String,
    model: &ScopePath,
    mtl: &FileDescriptor,
    folder: &ScopePath,
    common: &KeptCommon,
    folder_materials: &BTreeMap<ScopePath, Vec<String>>,
    scope: &IssueScope,
) -> Option<ContentFinding> {
    let names_of = |path: &ScopePath| {
        folder_materials
            .get(path)
            .or_else(|| common.materials.get(path))
    };
    let defined = names_of(&mtl.path)?;
    let undefined: Vec<&str> = names_of(model)?
        .iter()
        .filter(|used| !defined.contains(used))
        .map(String::as_str)
        .collect();
    if undefined.is_empty() {
        return None;
    }
    let mtl_name = if is_direct_common_file(&mtl.path) {
        mtl.path.as_str().to_owned()
    } else {
        relative(&mtl.path, folder)
    };
    Some(ContentFinding {
        code: MODEL_MATERIAL_UNDEFINED,
        scope: scope.clone(),
        context: vec![
            ("file", name),
            ("mtl", mtl_name),
            ("materials", undefined.join(", ")),
        ],
        disposition: Disposition::DropFolder,
        pass_through_eligible: true,
    })
}

/// What reading a member's own `face.xml` gave (`user_xml_findings`).
struct XmlOutcome {
    /// Its findings, each on the folder.
    findings: Vec<ContentFinding>,
    /// The xml, when it was read and parsed.
    parsed: Option<UserFaceXml>,
}

/// The findings of the member's own `face.xml` `file` among `files`, for the target
/// `version`, each on `scope` and never pass-through-eligible (a malformed xml can crash the
/// game): when it cannot be read, `source_read_failed`; when it does not parse, one finding
/// dropping the folder, `xml_broken` (not UTF-8, not well-formed XML), `xml_root_tag_invalid`
/// (its root is not `<config>`) or `face_diff_invalid` (its `<dif>` is no face diff the game
/// reads), naming the file and the reason; else `user_face_xml::check`'s, with the xml.
fn user_xml_findings(
    content: &ContentSource,
    file: &FileDescriptor,
    files: &FaceFiles,
    version: PesVersion,
    scope: &IssueScope,
) -> XmlOutcome {
    let finding = |(code, disposition, context): XmlFinding| ContentFinding {
        code: code.as_str(),
        scope: scope.clone(),
        context,
        disposition,
        pass_through_eligible: false,
    };
    let unparsed = |findings| XmlOutcome {
        findings,
        parsed: None,
    };
    let bytes = match read(content, file, scope, Disposition::DropFolder) {
        Ok(bytes) => bytes,
        Err(unread) => return unparsed(vec![unread]),
    };
    let name = relative(&file.path, files.folder);
    let xml = match user_face_xml::parse(&bytes) {
        Ok(xml) => xml,
        Err(error) => {
            let (code, context) = match &error {
                XmlError::Utf8 | XmlError::Xml { .. } => {
                    (Code::XmlBroken, ("error", error.to_string()))
                }
                XmlError::Root(root) => (Code::XmlRootTagInvalid, ("root", root.clone())),
                XmlError::Dif(reason) => (Code::FaceDiffInvalid, ("reason", reason.to_string())),
            };
            let context = vec![("file", name), context];
            return unparsed(vec![finding((code, Disposition::DropFolder, context))]);
        }
    };
    XmlOutcome {
        findings: user_face_xml::check(&xml, &name, files, version)
            .into_iter()
            .map(finding)
            .collect(),
        parsed: Some(xml),
    }
}

/// The `.model` files among `files`, those of the folder at `folder`, that the folder's own
/// `face.xml` files list, each with the `.mtl` its entry names (`user_face_xml::resolve`),
/// for `model_material_undefined` to compare: the xml overrides the search. An entry naming no
/// `material`, or one the compiler cannot resolve, is compared with nothing, and a `material`
/// naming no file is `xml_model_not_found` already. Empty when an xml did not parse or has an
/// Error: the folder is dropped.
fn listed_materials<'a>(
    xmls: &[(&FileDescriptor, XmlOutcome)],
    files: &'a [FileDescriptor],
    common: &'a KeptCommon,
    folder: &'a ScopePath,
) -> Vec<(&'a FileDescriptor, &'a FileDescriptor)> {
    let dropped = xmls.iter().any(|(_, outcome)| {
        outcome.parsed.is_none()
            || outcome
                .findings
                .iter()
                .any(|finding| finding.disposition == Disposition::DropFolder)
    });
    if dropped {
        return Vec::new();
    }
    // The shared face's files are not the folder's own: a model there is checked in that
    // folder's pass.
    let face_files = FaceFiles {
        own: files,
        linked_face: &[],
        common: &common.files,
        folder,
    };
    let model_kind = FileKind::Model(ModelFormat::PesModel);
    xmls.iter()
        .filter_map(|(_, outcome)| outcome.parsed.as_ref())
        .flat_map(|xml| &xml.children)
        .filter_map(|child| match child {
            Child::Model(model) => Some(model),
            Child::Dif(_) | Child::Other(_) => None,
        })
        .filter_map(|model| {
            let path = reference(model.attribute("path")?);
            let listed = resolve(&path, &face_files, model_kind)?;
            let material = reference(model.attribute("material")?);
            let mtl = resolve(&material, &face_files, FileKind::Mtl)?;
            Some((listed, mtl))
        })
        .collect()
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

/// `path` below `folder`, its subfolder kept (`face/hair.fmdl`): how a finding on a folder
/// names one of its files.
pub(crate) fn relative(path: &ScopePath, folder: &ScopePath) -> String {
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
    file_outcome(content, file, checked, scope, disposition, name).findings
}

/// `file_findings`, with `file` among the pass's weighted models when it is a model that parses
/// and carries hand weights (`ContentPass::hand_weighted`), and with its material names when it
/// is a pre-Fox model or material set that parses (`ContentPass::materials`).
fn file_outcome(
    content: &ContentSource,
    file: &FileDescriptor,
    checked: Checked,
    scope: &IssueScope,
    disposition: Disposition,
    name: &str,
) -> ContentPass {
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
        Err(unread) => return ContentPass::findings_only(vec![unread]),
    };
    let kind = match checked {
        Checked::Model(kind) => kind,
        Checked::Texture(format, rule) => {
            let found = texture_finding(format, rule, &bytes)
                .map(|code| {
                    // A renamed file cannot be converted as the format its name declares; nor
                    // can a pre-Fox side that is no multiple of 4 be kept: every pre-Fox
                    // texture is written block-compressed, and Direct3D 9 cannot create a
                    // block-compressed texture whose sides are not multiples of 4. A texture
                    // of another odd size converts, and what the game makes of it is the
                    // member's risk.
                    let eligible =
                        !matches!(code, Code::TextureTypeMismatch | Code::TextureNotDiv4);
                    finding(
                        code.as_str(),
                        vec![("file", name.to_owned())],
                        disposition,
                        eligible,
                    )
                })
                .into_iter()
                .collect();
            return ContentPass::findings_only(found);
        }
        Checked::Logo(format) => {
            let found = dds_convert::decode(&bytes, format)
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
            return ContentPass::findings_only(found);
        }
    };
    let read = match fired(kind, &bytes) {
        Ok(read) => read,
        Err(error) => {
            let broken = match kind {
                ModelKind::Fmdl | ModelKind::PreFoxModel => Code::ModelBroken,
                ModelKind::Mtl => Code::MtlBroken,
            };
            return ContentPass::findings_only(vec![finding(
                broken.as_str(),
                vec![("file", name.to_owned()), ("error", error)],
                disposition,
                false,
            )]);
        }
    };
    let findings = summed(read.fired)
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
        .collect();
    let mut hand_weighted = BTreeSet::new();
    if read.hand_weighted {
        hand_weighted.insert(file.path.clone());
    }
    let mut materials = BTreeMap::new();
    match kind {
        ModelKind::PreFoxModel | ModelKind::Mtl => {
            materials.insert(file.path.clone(), read.materials);
        }
        ModelKind::Fmdl => {}
    }
    ContentPass {
        findings,
        hand_weighted,
        materials,
    }
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

    /// The bytes of `pes_model`'s fixture `name`: real pre-Fox models and material sets,
    /// which this crate's own fixtures do not hold (`pes_model/tests/fixtures/README.md`).
    pub(super) fn pre_fox_fixture(name: &str) -> Vec<u8> {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../libs/pes_model/tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    /// The bytes of the tracer player's file `name`.
    pub(super) fn tracer_file(name: &str) -> Vec<u8> {
        fixture(&format!(
            "tracer/studio/egg Midcup Tracer/Players/05 - The Chad Stormworks Player/{name}"
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

    /// The deep pass for PES 21 over the folder export `co Midcup Deep` at `root`, holding `files`
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
        pass_for(version, root, files, unwritten, structure_codes).findings
    }

    /// `findings_for`'s whole pass, the weighted FMDLs with the findings.
    fn pass_for(
        version: PesVersion,
        root: &Path,
        files: &[(&str, Vec<u8>)],
        unwritten: &[&str],
        structure_codes: &[&str],
    ) -> ContentPass {
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
        let (resolved, codes) = resolved_with_issues("co Midcup Deep", &listed, &[], None);
        assert_eq!(codes, structure_codes, "the structure pass's findings");
        let source = ExportSource {
            export_id: ExportId(0),
            path: root.to_path_buf(),
            kind: SourceKind::Folder,
            file_name: "co Midcup Deep".to_owned(),
            display_name: "co Midcup Deep".to_owned(),
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

    /// The finding `code` on the folder `scope` with `context`, never passed through, with
    /// `disposition`.
    fn on_folder(
        code: &'static str,
        scope: &str,
        context: &[(&'static str, &str)],
        disposition: Disposition,
    ) -> ContentFinding {
        ContentFinding {
            code,
            scope: folder(scope),
            context: context
                .iter()
                .map(|(key, value)| (*key, (*value).to_owned()))
                .collect(),
            disposition,
            pass_through_eligible: false,
        }
    }

    /// The finding `code` on the folder `scope`, dropping it and never passed through, with
    /// `context`.
    pub(super) fn dropping(
        code: &'static str,
        scope: &str,
        context: &[(&'static str, &str)],
    ) -> ContentFinding {
        on_folder(code, scope, context, Disposition::DropFolder)
    }

    /// The finding `code` on the folder `scope`, which keeps it, with `context`.
    fn keeping(
        code: &'static str,
        scope: &str,
        context: &[(&'static str, &str)],
    ) -> ContentFinding {
        on_folder(code, scope, context, Disposition::Keep)
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
    fn the_fmdls_weighted_to_hand_bones_are_recorded_by_their_export_path() {
        let temp = scratch("deep_hand_weighted");
        let body = fixture("hand_split/body.fmdl");
        let pass = pass_for(
            PesVersion::Pes21,
            temp.path(),
            &[
                ("Players/05 - A/body.fmdl", body.clone()),
                ("Players/05 - A/boots.fmdl", tracer_boots()),
                ("Players/06 - B/torso.fmdl.common", Vec::new()),
                ("Common/torso.fmdl", body),
                ("Players/07 - C/body.fmdl", b"not a model".to_vec()),
            ],
            &[],
            &[],
        );
        let recorded: Vec<&str> = pass.hand_weighted.iter().map(ScopePath::as_str).collect();
        assert_eq!(recorded, ["Common/torso.fmdl", "Players/05 - A/body.fmdl"]);
        // The model's own findings are unchanged: its check finds nothing, the boots their
        // usual weights, and the broken file is `model_broken`.
        let codes: Vec<&str> = pass.findings.iter().map(|finding| finding.code).collect();
        assert_eq!(codes, ["fmdl_weights_not_normalized", "model_broken"]);
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

    /// The card head's model, clean, binding one material, `card`.
    fn card() -> Vec<u8> {
        pre_fox_fixture("cardhead_face_high.model")
    }

    /// The card head's material set, defining `card`.
    fn card_materials() -> Vec<u8> {
        pre_fox_fixture("cardhead_materials.mtl")
    }

    /// The card head's material set with its one material renamed `other`.
    fn other_materials() -> Vec<u8> {
        let mut set = pes_model::format::mtl::MaterialSet::read(&card_materials()).unwrap();
        set.materials[0].name = "other".to_owned();
        set.write()
    }

    /// The PES 17 deep pass's findings over `files`, slot 05's folder holding them by name.
    fn slot_05_findings(name: &str, files: &[(&str, Vec<u8>)]) -> Vec<ContentFinding> {
        let temp = scratch(name);
        let files: Vec<(String, Vec<u8>)> = files
            .iter()
            .map(|(file, bytes)| (format!("Players/05 - A/{file}"), bytes.clone()))
            .collect();
        let files: Vec<(&str, Vec<u8>)> = files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.clone()))
            .collect();
        findings_for(PesVersion::Pes17, temp.path(), &files, &[], &[])
    }

    /// A `face.xml` whose one `<model>`, typed `parts`, holds `attributes`.
    fn one_model_xml(attributes: &str) -> Vec<u8> {
        format!(r#"<config><model level="0" type="parts" {attributes}/></config>"#).into_bytes()
    }

    #[test]
    fn a_model_an_xml_lists_is_compared_with_the_mtl_its_entry_names_not_the_searched_one() {
        let findings = slot_05_findings(
            "deep_xml_material",
            &[
                ("hat.model", card()),
                ("hat.mtl", card_materials()),
                ("other.mtl", other_materials()),
                (
                    "face.xml",
                    one_model_xml(r#"path="./hat.model" material="./other.mtl""#),
                ),
            ],
        );
        let undefined = ContentFinding {
            code: "model_material_undefined",
            scope: folder("Players/05 - A"),
            context: vec![
                ("file", "hat.model".to_owned()),
                ("mtl", "other.mtl".to_owned()),
                ("materials", "card".to_owned()),
            ],
            disposition: Disposition::DropFolder,
            pass_through_eligible: true,
        };
        assert_eq!(findings, std::slice::from_ref(&undefined));
        // A Warning on the xml keeps the folder, so the comparison still runs.
        let findings = slot_05_findings(
            "deep_xml_material_warned",
            &[
                ("hat.model", card()),
                ("hat.mtl", card_materials()),
                ("other.mtl", other_materials()),
                (
                    "face.xml",
                    br#"<config><model level="0" type="cape" path="./hat.model" material="./other.mtl"/></config>"#.to_vec(),
                ),
            ],
        );
        assert_eq!(
            findings,
            [
                keeping("xml_type_unknown", "Players/05 - A", &[("type", "cape")]),
                undefined,
            ]
        );
    }

    #[test]
    fn a_reference_to_a_linked_face_s_model_is_found_by_the_deep_pass() {
        let temp = scratch("deep_xml_linked_face");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - A/hat.model", card()),
                ("Players/05 - A/hat.mtl", card_materials()),
                ("Players/05 - A/Round.face", Vec::new()),
                (
                    "Players/05 - A/face.xml",
                    br#"<config><model level="0" type="parts" path="./hat.model" material="./hat.mtl"/><model level="0" type="parts" path="./hair_high.model" material="./hair_high.mtl"/></config>"#.to_vec(),
                ),
                ("Faces/Round/hair_high.model", card()),
                ("Faces/Round/hair_high.mtl", card_materials()),
            ],
            &[],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn an_xml_naming_an_absent_model_drops_its_folder_and_compares_no_material() {
        let findings = slot_05_findings(
            "deep_xml_absent",
            &[
                ("hat.model", card()),
                ("hat.mtl", card_materials()),
                ("other.mtl", other_materials()),
                // The second entry lists `hat.model` with a `.mtl` lacking its material: with
                // the xml's Error dropping the folder, it is not compared.
                (
                    "face.xml",
                    br#"<config><model level="0" type="parts" path="./absent.model" material="./other.mtl"/><model level="0" type="parts" path="./hat.model" material="./other.mtl"/></config>"#.to_vec(),
                ),
            ],
        );
        assert_eq!(
            findings,
            [dropping(
                "xml_model_not_found",
                "Players/05 - A",
                &[("attribute", "path"), ("value", "./absent.model")]
            )]
        );
    }

    #[test]
    fn a_model_the_xml_does_not_list_is_unlisted_and_not_compared() {
        let unlisted = [keeping(
            "xml_model_unlisted",
            "Players/05 - A",
            &[("file", "hat.model")],
        )];
        let findings = slot_05_findings(
            "deep_xml_unlisted",
            &[
                ("hat.model", card()),
                ("hat.mtl", other_materials()),
                ("other.model", card()),
                ("other.mtl", card_materials()),
                (
                    "face.xml",
                    one_model_xml(r#"path="./other.model" material="./other.mtl""#),
                ),
            ],
        );
        assert_eq!(findings, unlisted);
        // With no `.mtl` at all, the search would leave both models every material undefined.
        let findings = slot_05_findings(
            "deep_xml_unlisted_no_mtl",
            &[
                ("hat.model", card()),
                ("other.model", card()),
                ("face.xml", one_model_xml(r#"path="./other.model""#)),
            ],
        );
        assert_eq!(findings, unlisted);
    }

    #[test]
    fn an_xml_that_cannot_be_read_drops_its_folder_with_one_finding_and_compares_no_material() {
        // The model has no `.mtl`: the search would make it `model_material_undefined`.
        let broken = |name: &str, xml: &[u8]| {
            slot_05_findings(name, &[("hat.model", card()), ("face.xml", xml.to_vec())])
        };
        assert_eq!(
            broken("deep_xml_broken", b"<config><model"),
            [dropping(
                "xml_broken",
                "Players/05 - A",
                &[
                    ("file", "face.xml"),
                    (
                        "error",
                        "the file is not well-formed XML: the root node was opened but never closed at 1:15"
                    ),
                ]
            )]
        );
        assert_eq!(
            broken("deep_xml_root", b"<cfg/>"),
            [dropping(
                "xml_root_tag_invalid",
                "Players/05 - A",
                &[("file", "face.xml"), ("root", "cfg")]
            )]
        );
        assert_eq!(
            broken("deep_xml_dif", b"<config><dif>RkFD</dif></config>"),
            [dropping(
                "face_diff_invalid",
                "Players/05 - A",
                &[
                    ("file", "face.xml"),
                    (
                        "reason",
                        "the face diff is 3 bytes long, shorter than its 80-byte header"
                    ),
                ]
            )]
        );
    }

    #[test]
    fn an_xml_is_read_in_face_but_not_under_ingame_face_nor_on_fox() {
        let xml = b"<config><model/></config>".to_vec();
        let missing = [
            dropping(
                "xml_model_type_missing",
                "Players/05 - A",
                &[("entry", "1")],
            ),
            dropping(
                "xml_model_path_missing",
                "Players/05 - A",
                &[("entry", "1")],
            ),
        ];
        assert_eq!(
            slot_05_findings("deep_xml_in_face", &[("face/face.xml", xml.clone())]),
            missing
        );
        assert_eq!(
            slot_05_findings(
                "deep_xml_marked",
                &[("ingame_face", Vec::new()), ("face.xml", xml.clone())]
            ),
            []
        );
        let temp = scratch("deep_xml_fox");
        assert_eq!(
            findings_for(
                PesVersion::Pes21,
                temp.path(),
                &[("Players/05 - A/face.xml", xml)],
                &[],
                &[]
            ),
            []
        );
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
