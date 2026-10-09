//! The deep pass (`team_compiler/pipeline.md` "2. Per-export serial steps", "Deep format
//! pass"): the checks only a file's contents can answer, run over the sanitized export the
//! structure pass leaves. What they find goes back to `aesthetics_export` as content findings,
//! which derive the sanitized export again, so a finding drops, cascades and passes through by
//! the structure pass's own rules.
//!
//! It checks what `compile` reads and nothing else: a file planning leaves unread has no
//! finding, which would drop what `compile` builds without it. This module reads every native
//! model (`.fmdl`, `.model`) and every `.mtl` of the player folders, the shared folders and
//! `Common/` that a task of the target reads, whatever the target version (a model of either
//! format is a source for either target): not a model another of its stem beats, a model with
//! no role, a per-kit variant left out, a file below a `Common/` subfolder, nor for PES 2018
//! to 2021 a model folder's `.mtl` no `.model` of the folder is converted with; and it reports
//! what the format crates' checks find
//! (`team_compiler/messages.md` "Model checks"): one finding per file and code, at the
//! severity the format crate gives it. An Error drops what holds the file and may pass through;
//! a Warning or an Info only informs. The far vertex, which both formats check, is reported as
//! `vertex_too_far_from_origin` and never passes through. A file that does not parse is
//! `model_broken` or `mtl_broken`. A `.model`, or a `.common` link to one, for which no `.mtl`
//! is found (`mtl_search`) has every material undefined, `model_material_undefined`, for PES
//! 2015 to 2017, and so has a model folder's `.model` no `.fmdl` of its stem beats for PES 2018
//! to 2021: the folder is dropped, `pass_through` or not. One whose `.mtl` lacks a
//! material it binds is `model_material_undefined` too, naming those materials, and may pass
//! through. glTF models are not read (Phase 7).
//!
//! It also checks every texture of those folders, of `Common/`, of the kits (but one the target
//! does not emit) and every portrait from its header alone (`team_compiler/messages.md`
//! "Textures"): a file renamed from another format, a side under one block, and the size rules
//! of a kit's main texture, of a mipmapped Fox texture and of a portrait. A slot whose two
//! portraits, its player folder's and its `Portraits/` file, differ in bytes is
//! `portrait_conflict`, which skips the export. A logo source is the one texture decoded in
//! full: one that does not decode is `logo_file_invalid`.
//!
//! Each file directly in `Collars/` (planning takes no other) must be named for a stock
//! collar of the target version that a kit config can name, and no collar the suite holds
//! itself (`collar`); a collar model is then checked as any model, its Errors dropping the
//! file.
//!
//! For PES 2015 to 2017 it also reads a face folder's own `face.xml` (`user_face_xml`), a
//! player's or a shared face folder's, and reports its content checks (`team_compiler/messages.md`
//! "XML/MTL content checks"); the xml then decides which `.mtl` a model's
//! `model_material_undefined` compares with: the one its entry names, for the models it lists
//! (a `Common/` model an entry names included), instead of the one the search finds. A player
//! folder holding its own `face.xml` and linking a shared face folder is
//! `xml_shared_face_conflict`: a face takes one xml.
//!
//! Last, it reads the small data files whole (`documents`): the face diff of each player
//! folder and shared face folder (`face_diff_invalid`, `xml_dif_conflict`), but on PES 2015
//! to 2017 not a `face_diff.bin` beside a member's `face.xml` holding a `<dif>`, which
//! replaces it; each kit's `config.toml` (`kit_config_invalid`), each player's `settings.toml`
//! (`settings_toml_invalid`), and each kit's and a team export's root `colors.txt`, one
//! `color_entry_invalid` per line the file refuses (a referee export's is read by nothing).
//!
//! The model folders and `Common/`'s files are checked in parallel on the caller's rayon
//! pool, each worker reading and holding one file at a time, and the findings are collected
//! in file order (`content_findings`). Each model it parses (`.fmdl`, `.model`) is also asked,
//! on the same parse, whether its vertices carry hand weights, which planning needs for the
//! hand auto-split.

pub(crate) mod collar;
mod documents;
mod materials;
mod model;
mod portrait;
mod texture;

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, FileKind, IssueScope, KitTextureSource,
    ModelFormat, PlayerFolder, SharedKind, SharedModelFolder, ValidatedAestheticsExport, classify,
    common_link_name,
};
use dds_convert::SourceFormat;
use pes_version::{Engine, PesVersion};
use rayon::prelude::*;
use vtree::ScopePath;

use crate::bins::{KIT_COLORS, TEAM_COLORS};
use crate::messages::Code;
use crate::mtl_search::mtl_for;
use crate::plan::roles::{
    FolderModels, PlayerFile, emits_kit_texture, file_stem, is_direct_root_folder_file,
    is_selected_common_model, is_user_face_xml, link_feeds_own_package, linked_folder, player_file,
    selected_common_model, texture_format,
};
use crate::reader::ContentSource;
use crate::user_face_xml::{
    self, Child, FaceFiles, UserFaceXml, XmlError, XmlFinding, reference, resolve,
};
use collar::collar_findings;
use documents::{colors_findings, face_diff_findings, kit_config_findings, settings_finding};
use materials::{TextureSources, held_stems, texture_findings};
use model::{MaterialRead, ModelKind, fired};
use portrait::{folder_portrait, portrait_conflict, portrait_findings};
use texture::{SizeRule, texture_finding};

pub(crate) use documents::version_clamped;
pub(crate) use model::{FAR_VERTEX_CODES, Fired, summed};

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
    /// The export paths of the FMDLs it parsed that hold a material of the `metal` family
    /// (`model::ModelRead::metal`), whatever the target, as `hand_weighted` is recorded:
    /// planning gives a player folder whose pre-Fox face converts one of them the template
    /// environment map (`plan::ModelFolder::environment_map`), and no Fox folder converts
    /// one. A file that does not parse is not among them.
    pub(crate) metal_models: BTreeSet<ScopePath>,
    /// The export path of each pre-Fox `.model` and `.mtl` it parsed, with its materials
    /// (`model::ModelRead::materials`), which a model's `model_material_undefined` compares and
    /// a `.mtl`'s texture lookup reads. A file that does not parse is not among them.
    materials: BTreeMap<ScopePath, Vec<MaterialRead>>,
}

impl ContentPass {
    /// The pass of a file that gives `findings`, no weighted model and no material names.
    fn findings_only(findings: Vec<ContentFinding>) -> ContentPass {
        ContentPass {
            findings,
            ..ContentPass::default()
        }
    }

    /// `other`'s findings after this pass's, and its weighted models, metal models and material
    /// names with this pass's.
    fn append(&mut self, other: ContentPass) {
        self.findings.extend(other.findings);
        self.hand_weighted.extend(other.hand_weighted);
        self.metal_models.extend(other.metal_models);
        self.materials.extend(other.materials);
    }
}

/// The `Common/` files the deep pass keeps, which a model's `.mtl` search looks among, with the
/// materials of those that are parsed pre-Fox models and material sets
/// (`ContentPass::materials`), and the textures the team's Common output will hold, which a
/// pre-Fox `.mtl`'s Common path may name (`materials::supply`).
#[derive(Debug, Default)]
struct KeptCommon {
    /// The kept files, in `Common/`'s order.
    files: Vec<FileDescriptor>,
    /// The materials of the kept pre-Fox models and material sets, by export path.
    materials: BTreeMap<ScopePath, Vec<MaterialRead>>,
    /// The stems, folded, of the kept textures, which the export's Common textures task packs.
    texture_stems: BTreeSet<String>,
    /// The stems, folded, the installed CPKs loaded before the run's hold in the team's Common
    /// output; `None` when that lookup cannot be made or the export has no team ID.
    installed: Option<BTreeSet<String>>,
}

impl KeptCommon {
    /// The materials of the parsed pre-Fox file at `path`: a folder's, among
    /// `folder_materials`, or a kept `Common/` one's; `None` for a file that did not parse.
    fn materials_of<'a>(
        &'a self,
        path: &ScopePath,
        folder_materials: &'a BTreeMap<ScopePath, Vec<MaterialRead>>,
    ) -> Option<&'a Vec<MaterialRead>> {
        folder_materials
            .get(path)
            .or_else(|| self.materials.get(path))
    }
}

/// The content findings of `export`, the sanitized export read from `content` and compiled for
/// `version`, with the models among its files that carry hand weights and the FMDLs among
/// them holding a metal material (`ContentPass`). The
/// findings come in file order: each player folder's models, material sets and textures, then
/// its face diff, its portrait and its `settings.toml`; then each shared folder's (faces with
/// their face diff, boots, gloves), then `Common/`'s, then each collar's (a file directly in
/// `Collars/`), then each `Portraits/` file with its slot's `portrait_conflict`, then each
/// kit's `config.toml`, `colors.txt` and textures, then the logo's, then a team export's root
/// `colors.txt`'s. An Error on a folder's or a kit's file drops the folder; one on a `Common/`
/// file drops the file, and the cascade then drops the players linking it; one on a collar, a
/// portrait, a `settings.toml` or a logo file drops that file. A refused
/// `colors.txt` line is a Warning on the file, which drops nothing. `Common/`'s files are
/// checked before the folders, whose models' `.mtl` search sees only the ones the validation
/// report keeps, with `pass_through` as set, and whose models' materials are compared with
/// the kept ones' names (`KeptCommon`).
///
/// It checks only what `compile` reads: not a file below a `Common/` subfolder, nor a
/// `Common/` model another of its stem beats (`is_selected_common_model`), nor a kit texture
/// the target does not emit (`emits_kit_texture`), nor a folder's file `folder_findings`
/// leaves unread.
///
/// For PES 2015 to 2017 each `.mtl`'s texture paths are looked up (`materials`), and for PES
/// 2018 to 2021 those of each folder `.mtl` a selected `.model` pairs with, right after its own
/// findings, and of each `Common/` `.mtl` a player's link to a Common `.model` pairs with, on
/// his folder against `Common/`'s textures and `installed` (`folder_findings`), each finding
/// keeping what holds the `.mtl`: a folder's against the
/// textures the folder holds, `Common/`'s and `installed`, the stems the installed CPKs loaded
/// before the run's hold in the team's Common output (`None` when that lookup cannot be made or
/// the export has no team ID); a kept `Common/` `.mtl`'s against `Common/`'s textures and
/// `installed`, its mesh-used materials being those the kept `Common/` models bind.
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
    installed: Option<BTreeSet<String>>,
    pass_through: bool,
) -> ContentPass {
    let size_rule = SizeRule::of(version);
    let engine = version.engine();
    // Each group below is collected in its items' order (rayon's indexed `collect`), so the
    // findings come out in file order whatever the workers' scheduling.
    let mut common: Vec<ContentPass> = export
        .common
        .par_iter()
        .map(|file| {
            // The Common tasks read only the files directly in `Common/`, and a model there
            // only when the target selects it for its stem: nothing reads the others.
            let unread = !is_direct_root_folder_file(&file.path)
                || (matches!(file.kind, FileKind::Model(_))
                    && !is_selected_common_model(&export.common, file, engine));
            let Some(checked) = checked_as(file, size_rule).filter(|_| !unread) else {
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
    // A model's `.mtl` search looks only among the `Common/` files the report keeps: planning
    // sees the export after the drops, so the face task searches the same files, and a model
    // whose only `.mtl` is a dropped Common one is `model_material_undefined` here rather than
    // a material the face task cannot find. Under `pass_through` a file whose every Error is
    // eligible is kept and packed, so it is found here too: leaving it out would drop a
    // player for a file `compile` packs. A file below a subfolder is found by no lookup.
    let mut kept_common = KeptCommon {
        installed,
        ..KeptCommon::default()
    };
    for (file, pass) in export.common.iter().zip(&common) {
        if !is_direct_root_folder_file(&file.path) || drops_file(&pass.findings, pass_through) {
            continue;
        }
        kept_common.files.push(file.clone());
        if let Some(read) = pass.materials.get(&file.path) {
            kept_common
                .materials
                .insert(file.path.clone(), read.clone());
        }
    }
    kept_common.texture_stems = kept_common
        .files
        .iter()
        .filter(|file| file.kind == FileKind::Texture)
        .map(|file| vtree::fold_name(file_stem(file.path.name())))
        .collect();
    match engine {
        Engine::Fox => {}
        Engine::PreFox => common_mtl_findings(&export.common, &mut common, &kept_common),
    }
    let players: Vec<ContentPass> = export
        .players
        .par_iter()
        .map(|player| {
            let folder = &player.path;
            // Under the marker the face files are not used, his own `face.xml` among them.
            let face = FaceUse::of_player(export, player, engine);
            let conflict = shared_face_conflict(folder, &player.files, &face, engine);
            let (mut pass, xml_dif) = folder_findings(
                content,
                folder,
                &player.files,
                &FolderModels::of(folder, &player.files, engine),
                &kept_common,
                version,
                face,
            );
            let findings = &mut pass.findings;
            findings.extend(conflict);
            findings.extend(face_diff_findings(
                content,
                folder,
                &player.files,
                &FolderModels::of_player(player, engine),
                xml_dif,
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
            let (mut pass, xml_dif) = folder_findings(
                content,
                &face.path,
                &face.files,
                &FolderModels::of_shared(&face.path, &face.files, SharedKind::Face, engine),
                &kept_common,
                version,
                FaceUse::Used {
                    linked_face: None,
                    combined: Vec::new(),
                },
            );
            pass.findings.extend(face_diff_findings(
                content,
                &face.path,
                &face.files,
                &FolderModels::of_shared(&face.path, &face.files, SharedKind::Face, engine),
                xml_dif,
            ));
            pass
        })
        .collect();
    let boots = export
        .boots
        .par_iter()
        .map(|folder| (SharedKind::Boots, folder));
    let gloves = export
        .gloves
        .par_iter()
        .map(|folder| (SharedKind::Gloves, folder));
    let boots_and_gloves: Vec<ContentPass> = boots
        .chain(gloves)
        .map(|(kind, shared)| {
            // A boots or gloves folder has no face, so no xml.
            let (pass, _) = folder_findings(
                content,
                &shared.path,
                &shared.files,
                &FolderModels::of_shared(&shared.path, &shared.files, kind, engine),
                &kept_common,
                version,
                FaceUse::Unused,
            );
            pass
        })
        .collect();
    let mut pass = ContentPass::default();
    for group in [players, faces, boots_and_gloves, common] {
        for found in group {
            pass.append(found);
        }
    }
    let findings = &mut pass.findings;
    // Planning takes a collar only directly in `Collars/`: nothing reads a subfolder's file.
    let collars = export
        .collars
        .iter()
        .filter(|file| is_direct_root_folder_file(&file.path));
    for file in collars {
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
        // Planning drops a texture the target does not emit before the kit's task reads any.
        for texture in kit
            .textures
            .iter()
            .filter(|texture| emits_kit_texture(engine, &texture.stem))
        {
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
    // A referee export has no team record, so nothing reads its root `colors.txt`.
    if let Some(colors) = &export.root.team_colors
        && !export.team_name.is_referees()
    {
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

/// Whether `findings`, a `Common/` file's, drop it from the validation report: one drops the
/// file and `pass_through` cannot keep it, being off or the finding not eligible
/// (`object_model.md` "Validation semantics").
fn drops_file(findings: &[ContentFinding], pass_through: bool) -> bool {
    findings.iter().any(|finding| {
        finding.disposition == Disposition::DropFile
            && !(pass_through && finding.pass_through_eligible)
    })
}

/// The texture findings of each `.mtl` among `common`, the export's `Common/` files whose
/// passes are `passes`, that `kept` holds, a pre-Fox target's (`materials::texture_findings`):
/// each on its file, keeping it, appended to its pass after its own findings. A `Common/`
/// `.mtl` is packed once for the team, before any player's pairing is known, so its mesh-used
/// materials are those every kept `Common/` model's meshes bind, and its paths resolve among
/// the kept `Common/` textures and the installed CPKs alone.
fn common_mtl_findings(common: &[FileDescriptor], passes: &mut [ContentPass], kept: &KeptCommon) {
    let model_kind = FileKind::Model(ModelFormat::PesModel);
    let used: BTreeSet<&str> = kept
        .files
        .iter()
        .filter(|file| file.kind == model_kind)
        .filter_map(|file| kept.materials.get(&file.path))
        .flatten()
        .filter(|material| material.mesh_used)
        .map(|material| material.name.as_str())
        .collect();
    let sources = TextureSources {
        held: &kept.texture_stems,
        common: &kept.texture_stems,
        installed: kept.installed.as_ref(),
    };
    for (file, pass) in common.iter().zip(passes) {
        if file.kind != FileKind::Mtl {
            continue;
        }
        // A file the pass dropped, or that did not parse, has no kept materials.
        let Some(read) = kept.materials.get(&file.path) else {
            continue;
        };
        pass.findings.extend(texture_findings(
            file.path.name(),
            read,
            &used,
            &sources,
            &IssueScope::File(file.path.clone()),
        ));
    }
}

/// Whether a model folder's face files are used, and which shared folders the face packs
/// (`folder_findings`).
enum FaceUse<'a> {
    /// The face files are not used: a boots or gloves folder, or a player under `ingame_face`.
    Unused,
    /// The face files are used: a player folder without `ingame_face`, or a shared face
    /// folder.
    Used {
        /// The shared face folder the folder links, whose files its own `face.xml` may name and
        /// whose textures its `.mtl` paths may name; `None` for a shared face folder and a
        /// player linking none.
        linked_face: Option<&'a SharedModelFolder>,
        /// The shared boots and gloves folders whose textures the face packs too, those whose
        /// link feeds the player's own package (`link_feeds_own_package`): a referee's, every
        /// one of whose links does. Empty for a team player, whose boots and gloves links stay
        /// plain while his face is used.
        combined: Vec<(SharedKind, &'a SharedModelFolder)>,
    },
}

impl<'a> FaceUse<'a> {
    /// How the face files of `player`, a player folder of `export` compiled for a target of
    /// `engine`, are used, its links resolved as planning resolves them (`linked_folder`).
    fn of_player(
        export: &'a ValidatedAestheticsExport,
        player: &PlayerFolder,
        engine: Engine,
    ) -> FaceUse<'a> {
        if player.ingame_face {
            return FaceUse::Unused;
        }
        let linked_face = player
            .links
            .iter()
            .find(|link| matches!(link.kind, SharedKind::Face))
            .and_then(|link| linked_folder(export, link));
        let combined = player
            .links
            .iter()
            .filter(|link| match link.kind {
                SharedKind::Face => false,
                // This pass is what finds the hand-weighted models, so it cannot see a gloves
                // link that combines only with split hands: that folder's textures do not count
                // here for the player's `.mtl` paths, though planning puts them in his textures
                // task.
                SharedKind::Boots | SharedKind::Gloves => {
                    link_feeds_own_package(export, engine, player, link, &BTreeSet::new())
                }
            })
            .filter_map(|link| Some((link.kind, linked_folder(export, link)?)))
            .collect();
        FaceUse::Used {
            linked_face,
            combined,
        }
    }
}

/// `xml_shared_face_conflict` on the player folder at `folder`, holding `files`, when on
/// pre-Fox it holds its own `face.xml` (`is_user_face_xml`) and its face links a shared face
/// folder (`face`), whether or not that folder holds one: the face's xml is the shared
/// folder's, or the one generated, and two xmls leave no rule for which one speaks
/// (`messages.md` "User-supplied `face.xml`"). It names his xml below his folder and the shared
/// folder by its export path, and drops his folder, never passing through: no choice of xml
/// builds the face the member meant. Fox ignores every `face.xml` (`xml_ignored_fox`).
fn shared_face_conflict(
    folder: &ScopePath,
    files: &[FileDescriptor],
    face: &FaceUse,
    engine: Engine,
) -> Option<ContentFinding> {
    match engine {
        Engine::Fox => return None,
        Engine::PreFox => {}
    }
    let FaceUse::Used {
        linked_face: Some(shared),
        ..
    } = face
    else {
        return None;
    };
    let xml = files.iter().find(|file| is_user_face_xml(folder, file))?;
    Some(ContentFinding {
        code: Code::XmlSharedFaceConflict.as_str(),
        scope: IssueScope::Folder(folder.clone()),
        context: vec![
            ("file", relative(&xml.path, folder)),
            ("link", shared.path.as_str().to_owned()),
        ],
        disposition: Disposition::DropFolder,
        pass_through_eligible: false,
    })
}

/// The findings of the files among `files`, those of the model folder at `folder`, that the
/// deep pass reads (`checked_as`, textures held to `size_rule`): each on the folder's scope,
/// an Error dropping the folder, the file named below the folder (not a file with no role, such
/// as a model the target's own format beats, nor a per-kit model variant left out, nor on Fox
/// a `.mtl` no `.model` pairs with, which nothing reads); for each model `pairings` pairs (on
/// pre-Fox each `.model` with a role and each typed `.common` link to one, on Fox each
/// `.model` no FMDL beats and each `.common` link loading a Common `.model`),
/// `model_material_undefined` (`material_finding`, its `.mtl` searched among the folder's files
/// and `common`'s), right after the model's own findings; each read `.mtl`'s texture lookup
/// (`materials::texture_findings`), right after the `.mtl`'s own findings, and on Fox that of
/// a `Common/` `.mtl` a link pairs with, against `Common/`'s textures, right after the link's
/// (once per folder); with the models among them that carry hand weights and the materials of the
/// pre-Fox ones. The files are read and checked in parallel, each worker holding one file, and
/// the models' materials compared after, from what each read kept.
///
/// When `face` says the folder's face files are used (`FaceUse::Used`), each member's own
/// `face.xml` among `files` (`PlayerFile::FaceXml`) is read and checked (`user_xml_findings`),
/// its findings at its place in file order, and the xml overrides the search:
/// `model_material_undefined` compares only the models it lists with the `.mtl` each entry
/// names (`listed_materials`), a `Common/` model an entry names included, its finding after
/// every file's, naming it by its export path, and none at all when an xml has an Error,
/// which drops the folder; nor is any `.mtl`'s texture looked up then. The textures of the
/// shared folders the face packs count for the folder's `.mtl` paths, as they do for the face
/// task. Returned with whether such an xml holds a `<dif>`, which the face writes in place of
/// the folder's `face_diff.bin` (`face_diff_findings`).
fn folder_findings(
    content: &ContentSource,
    folder: &ScopePath,
    files: &[FileDescriptor],
    models: &FolderModels,
    common: &KeptCommon,
    version: PesVersion,
    face: FaceUse,
) -> (ContentPass, bool) {
    let engine = version.engine();
    let size_rule = SizeRule::of(version);
    let scope = IssueScope::Folder(folder.clone());
    let xmls: Vec<(&FileDescriptor, XmlOutcome)> = match &face {
        FaceUse::Used { linked_face, .. } => {
            let face_files = FaceFiles {
                own: files,
                linked_face: linked_face.map_or(&[], |face| face.files.as_slice()),
                common: &common.files,
                folder,
            };
            files
                .iter()
                .filter(|file| player_file(folder, file, models) == Some(PlayerFile::FaceXml))
                .map(|file| {
                    let outcome = user_xml_findings(content, file, &face_files, version, &scope);
                    (file, outcome)
                })
                .collect()
        }
        FaceUse::Unused => Vec::new(),
    };
    let xml_drops_folder = xmls.iter().any(|(_, outcome)| {
        outcome.parsed.is_none()
            || outcome
                .findings
                .iter()
                .any(|finding| finding.disposition == Disposition::DropFolder)
    });
    // With an xml, the models it lists and the `.mtl` each entry names, none when an xml
    // drops the folder; `None` when there is no xml, and the search pairs each model.
    let listed = (!xmls.is_empty()).then(|| {
        if xml_drops_folder {
            Vec::new()
        } else {
            listed_materials(&xmls, files, common, folder)
        }
    });
    let pairings = pairings(folder, files, models, engine, listed, common);
    // A file with no role (a model the target's own format beats, `FolderModels::beaten`; a
    // model in `common/`) and a per-kit model variant left out are read by nothing, and on
    // Fox a `.mtl` is read only by the conversion of a `.model` paired with it, so one no
    // such model pairs with (beside only FMDLs, or a `.model` an FMDL beats) is read by
    // nothing either. None is checked: its findings would drop a folder `compile` builds
    // without it (`pipeline.md` step 3 "Format conversion").
    let unread = |file: &FileDescriptor| {
        matches!(
            player_file(folder, file, models),
            None | Some(PlayerFile::LeftOutKitVariant)
        ) || match engine {
            Engine::Fox => {
                file.kind == FileKind::Mtl
                    && !pairings
                        .iter()
                        .any(|pairing| pairing.mtl.is_some_and(|mtl| mtl.path == file.path))
            }
            Engine::PreFox => false,
        }
    };
    // Collected in file order (an indexed `collect`), whatever the scheduling.
    let mut per_file: Vec<ContentPass> = files
        .par_iter()
        .map(|file| match checked_as(file, size_rule) {
            Some(checked) if !unread(file) => file_outcome(
                content,
                file,
                checked,
                &scope,
                Disposition::DropFolder,
                &relative(&file.path, folder),
            ),
            Some(_) | None => ContentPass::default(),
        })
        .collect();
    let mut materials = BTreeMap::new();
    for found in &mut per_file {
        materials.append(&mut found.materials);
    }
    let shared: Vec<(SharedKind, &SharedModelFolder)> = match &face {
        FaceUse::Used {
            linked_face,
            combined,
        } => linked_face
            .iter()
            .map(|face| (SharedKind::Face, *face))
            .chain(combined.iter().copied())
            .collect(),
        FaceUse::Unused => Vec::new(),
    };
    let held = match engine {
        // A folder an xml Error drops gets no texture finding (`messages.md`, the paragraph
        // starting "On pre-Fox the check runs in the deep pass"): the member fixes the xml
        // first, and its entries may name other `.mtl` files.
        Engine::PreFox if xml_drops_folder => None,
        // The `.mtl` checked is the member's source, the same whatever the target, so on Fox
        // the one a selected `.model` pairs with, the only one the pass reads there, is looked
        // up as on pre-Fox (`messages.md`, the `mtl_texture_not_found` row).
        Engine::Fox | Engine::PreFox => Some(held_stems(folder, files, models, &shared, engine)),
    };
    let sources = held.as_ref().map(|held| TextureSources {
        held,
        common: &common.texture_stems,
        installed: common.installed.as_ref(),
    });
    // A Common part's texture paths are pointed among `Common/`'s textures alone
    // (`processing::model`), so a Common `.mtl` is looked up against them, as pre-Fox looks up
    // its Common `.mtl` files (`common_mtl_findings`).
    let common_sources = TextureSources {
        held: &common.texture_stems,
        common: &common.texture_stems,
        installed: common.installed.as_ref(),
    };
    // The Common `.mtl` files looked up for this folder so far: two links finding one are one
    // lookup.
    let mut looked_up: BTreeSet<&ScopePath> = BTreeSet::new();
    let mut pass = ContentPass::default();
    for (file, found) in files.iter().zip(per_file) {
        pass.append(found);
        if let Some(sources) = &sources
            && file.kind == FileKind::Mtl
            && let Some(read) = materials.get(&file.path)
        {
            let used = used_names(&pairings, &file.path, &materials, common);
            pass.findings.extend(texture_findings(
                &relative(&file.path, folder),
                read,
                &used,
                sources,
                &scope,
            ));
        }
        if let Some((_, outcome)) = xmls.iter().find(|(xml, _)| xml.path == file.path) {
            pass.findings.extend(outcome.findings.iter().cloned());
        }
        for pairing in pairings
            .iter()
            .filter(|pairing| pairing.file.path == file.path)
        {
            pass.findings.extend(material_finding(
                pairing, folder, common, &materials, &scope,
            ));
            let common_mtl = match engine {
                // Fox has no Common model output: each linking player's Models task converts
                // the Common `.model` with this `.mtl`, so its lookup is the folder's, right
                // after the link's own finding.
                Engine::Fox => pairing
                    .mtl
                    .filter(|mtl| is_direct_root_folder_file(&mtl.path)),
                // Pre-Fox packs a Common `.mtl` once for the team and looks it up on its own
                // file (`common_mtl_findings`).
                Engine::PreFox => None,
            };
            if let Some(mtl) = common_mtl
                && looked_up.insert(&mtl.path)
                && let Some(read) = common.materials.get(&mtl.path)
            {
                let used = used_names(&pairings, &mtl.path, &materials, common);
                pass.findings.extend(texture_findings(
                    mtl.path.as_str(),
                    read,
                    &used,
                    &common_sources,
                    &scope,
                ));
            }
        }
    }
    // An xml entry may name a `Common/` model, which is no file of the folder's.
    for pairing in pairings
        .iter()
        .filter(|pairing| !files.iter().any(|file| file.path == pairing.file.path))
    {
        pass.findings.extend(material_finding(
            pairing, folder, common, &materials, &scope,
        ));
    }
    let xml_dif = xmls.iter().any(|(_, outcome)| {
        outcome.parsed.as_ref().is_some_and(|xml| {
            xml.children
                .iter()
                .any(|child| matches!(child, Child::Dif(_)))
        })
    });
    pass.materials = materials;
    (pass, xml_dif)
}

/// A `.model` of a model folder paired with the `.mtl` it binds its materials from: what
/// `model_material_undefined` compares, what makes a `.mtl` material mesh-used for the
/// texture lookup, and on Fox what makes a `.mtl` read at all.
struct Pairing<'a> {
    /// The folder's file the pairing is about, which a finding names: a `.model`, or a typed
    /// `.common` link to a `Common/` one.
    file: &'a FileDescriptor,
    /// The model whose materials count: the file itself, or a link's kept `Common/` model;
    /// `None` when the pass dropped that model.
    model: Option<&'a ScopePath>,
    /// The `.mtl`: the one the folder's `face.xml` entry names, or the search's (`mtl_for`);
    /// `None` when the search finds none.
    mtl: Option<&'a FileDescriptor>,
}

/// The pairings of the models among `files`, those of the model folder at `folder` whose
/// models are `models`, read for a target of `engine`: with the folder's own `face.xml`, the
/// models it lists with the `.mtl` each entry names (`listed`, empty when an xml drops the
/// folder); without, on pre-Fox, each `.model` the face or the boots and gloves read
/// (`PlayerFile::PreFoxModel`, `PlayerFile::PreFoxPart`) and each typed `.common` link loading
/// a Common `.model` (`selected_common_model`; one loading a Common FMDL pairs none, the FMDL's
/// conversion writing its material set) with the `.mtl` its search finds among `files` and
/// `common`'s; on Fox each `.model` with a role (`PlayerFile::Model`: no FMDL of its stem
/// beats it) and each `.common` link with a role loading a Common `.model` (one loading a
/// Common FMDL pairs none, the FMDL carrying its materials) the same way. A shared folder's
/// search sees no `Common/` file: its `.common` links resolve nothing (`FolderModels::shared`),
/// as its tasks' search finds none, so a `.model` whose only `.mtl` is such a link has none.
fn pairings<'a>(
    folder: &'a ScopePath,
    files: &'a [FileDescriptor],
    models: &FolderModels,
    engine: Engine,
    listed: Option<Vec<(&'a FileDescriptor, &'a FileDescriptor)>>,
    common: &'a KeptCommon,
) -> Vec<Pairing<'a>> {
    if let Some(listed) = listed {
        return listed
            .into_iter()
            .map(|(model, mtl)| Pairing {
                file: model,
                model: Some(&model.path),
                mtl: Some(mtl),
            })
            .collect();
    }
    let link_targets: &[FileDescriptor] = if models.is_shared() {
        &[]
    } else {
        &common.files
    };
    files
        .iter()
        .filter(|file| match engine {
            // On Fox a selected `.model` is converted with the `.mtl` its search finds, and so
            // is a linked Common `.model`. One an FMDL of its stem beats has no role and is read
            // by nothing: dropping the folder for it would lose a working FMDL.
            Engine::Fox => {
                let role = player_file(folder, file, models);
                (file.kind == FileKind::Model(ModelFormat::PesModel)
                    && matches!(role, Some(PlayerFile::Model { .. })))
                    || (matches!(role, Some(PlayerFile::CommonModel { .. }))
                        && !links_common_fmdl(file, common, engine))
            }
            // The roles are read without the `ingame_face` marker (`FolderModels::of`), so a
            // model link is `PreFoxCommonModel` here even in a marked folder, where planning
            // makes it a part of his boots or gloves whose `.mtl` is needed all the same. A
            // link loading a Common FMDL pairs none: its material set is its conversion's. A
            // `.model` with no role, or a per-kit variant left out, is read by nothing.
            Engine::PreFox => {
                let role = player_file(folder, file, models);
                (file.kind == FileKind::Model(ModelFormat::PesModel)
                    && matches!(
                        role,
                        Some(PlayerFile::PreFoxModel { .. } | PlayerFile::PreFoxPart { .. })
                    ))
                    || (matches!(role, Some(PlayerFile::PreFoxCommonModel { .. }))
                        && !links_common_fmdl(file, common, engine))
            }
        })
        .map(|file| {
            let model = match common_link_name(file.path.name()) {
                Some(linked) => {
                    selected_common_model(&common.files, &linked, engine).map(|model| &model.path)
                }
                None => Some(&file.path),
            };
            Pairing {
                file,
                model,
                mtl: mtl_for(&file.path, folder, files, link_targets),
            }
        })
        .collect()
}

/// Whether the `.common` model link `file` loads a Common FMDL on a target of `engine`, which
/// takes no `.mtl`: on Fox it carries its materials, on pre-Fox the Common models task converts
/// it with the material set its conversion writes. The Common model it loads is among
/// `common`'s kept files (`selected_common_model`), or, when the pass dropped that file, the
/// model its name links.
fn links_common_fmdl(file: &FileDescriptor, common: &KeptCommon, engine: Engine) -> bool {
    let Some(linked) = common_link_name(file.path.name()) else {
        return false;
    };
    let kind = selected_common_model(&common.files, &linked, engine)
        .map_or_else(|| classify(&linked), |model| model.kind);
    kind == FileKind::Model(ModelFormat::Fmdl)
}

/// The names of the materials the meshes of the models `pairings` pair with the `.mtl` at
/// `mtl` bind: its mesh-used materials. A model that did not parse binds none.
/// `folder_materials` and `common`'s are the parsed pre-Fox files' (`ContentPass::materials`).
fn used_names<'a>(
    pairings: &[Pairing],
    mtl: &ScopePath,
    folder_materials: &'a BTreeMap<ScopePath, Vec<MaterialRead>>,
    common: &'a KeptCommon,
) -> BTreeSet<&'a str> {
    pairings
        .iter()
        .filter(|pairing| pairing.mtl.is_some_and(|paired| &paired.path == mtl))
        .filter_map(|pairing| common.materials_of(pairing.model?, folder_materials))
        .flatten()
        .filter(|material| material.mesh_used)
        .map(|material| material.name.as_str())
        .collect()
}

/// `model_material_undefined` on `scope` for `pairing`, a pre-Fox model of the model folder at
/// `folder`; `folder_materials` are the materials of the folder's parsed pre-Fox files,
/// `common`'s of the kept `Common/` ones (`ContentPass::materials`).
///
/// Each file is named below the folder, or by its export path in `Common/`
/// (`named_on_folder`): an entry of the folder's `face.xml` may pair a `Common/` model with
/// the folder's `.mtl`. When its search (`mtl_search::mtl_for`) finds no `.mtl`, the finding
/// names the model and is not pass-through-eligible: the face's `face.xml` must name a
/// material set for the model, and there is none to name. When it has one, the material
/// names the model binds (a link's: the linked `Common/` model's) that the `.mtl` does not
/// define are the finding's, in the model's order, naming the model, the `.mtl` and those
/// names; it is pass-through-eligible: the file packs as it is, and the game renders those
/// meshes with its fallback material. No finding when every name is defined, or when either
/// file did not parse (its own `model_broken` or `mtl_broken` drops it).
fn material_finding(
    pairing: &Pairing,
    folder: &ScopePath,
    common: &KeptCommon,
    folder_materials: &BTreeMap<ScopePath, Vec<MaterialRead>>,
    scope: &IssueScope,
) -> Option<ContentFinding> {
    let name = named_on_folder(&pairing.file.path, folder);
    let Some(mtl) = pairing.mtl else {
        return Some(ContentFinding {
            code: MODEL_MATERIAL_UNDEFINED,
            scope: scope.clone(),
            context: vec![("file", name)],
            disposition: Disposition::DropFolder,
            pass_through_eligible: false,
        });
    };
    undefined_materials(
        name,
        pairing.model?,
        mtl,
        folder,
        common,
        folder_materials,
        scope,
    )
}

/// `model_material_undefined` on `scope` for the model at `model`, which the finding names
/// `name`, paired with `mtl`: the material names the model binds that the `.mtl` does not
/// define, in the model's order, naming the `.mtl` (`named_on_folder`); pass-through-eligible,
/// the file packing as it is. `None` when every name is defined, or when either file did not
/// parse. `folder_materials` and `common`'s are the materials of the parsed pre-Fox files
/// (`ContentPass::materials`).
fn undefined_materials(
    name: String,
    model: &ScopePath,
    mtl: &FileDescriptor,
    folder: &ScopePath,
    common: &KeptCommon,
    folder_materials: &BTreeMap<ScopePath, Vec<MaterialRead>>,
    scope: &IssueScope,
) -> Option<ContentFinding> {
    let defined = common.materials_of(&mtl.path, folder_materials)?;
    let undefined: Vec<&str> = common
        .materials_of(model, folder_materials)?
        .iter()
        .filter(|used| !defined.iter().any(|material| material.name == used.name))
        .map(|used| used.name.as_str())
        .collect();
    if undefined.is_empty() {
        return None;
    }
    Some(ContentFinding {
        code: MODEL_MATERIAL_UNDEFINED,
        scope: scope.clone(),
        context: vec![
            ("file", name),
            ("mtl", named_on_folder(&mtl.path, folder)),
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
/// naming no file is `xml_model_not_found` already. The caller asks only when no xml drops the
/// folder.
fn listed_materials<'a>(
    xmls: &[(&FileDescriptor, XmlOutcome)],
    files: &'a [FileDescriptor],
    common: &'a KeptCommon,
    folder: &'a ScopePath,
) -> Vec<(&'a FileDescriptor, &'a FileDescriptor)> {
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

/// How a finding on the model folder at `folder` names the file at `path`: below the folder
/// (`relative`), or by its export path when it sits directly in `Common/`, outside the folder
/// (`Common/legs.model`).
fn named_on_folder(path: &ScopePath, folder: &ScopePath) -> String {
    if is_direct_root_folder_file(path) {
        path.as_str().to_owned()
    } else {
        relative(path, folder)
    }
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
/// and carries hand weights (`ContentPass::hand_weighted`), among its metal models when it is an
/// FMDL that parses and holds a metal material (`ContentPass::metal_models`), and with its
/// material names when it is a pre-Fox model or material set that parses
/// (`ContentPass::materials`).
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
    let mut metal_models = BTreeSet::new();
    if read.metal {
        metal_models.insert(file.path.clone());
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
        metal_models,
        materials,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::slice;

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
        export_pass(
            "co Midcup Deep",
            None,
            version,
            root,
            files,
            unwritten,
            structure_codes,
        )
    }

    /// `pass_for` over the folder export `name`, whose `players.txt` is `players_txt`.
    fn export_pass(
        name: &str,
        players_txt: Option<&[u8]>,
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
        let (resolved, codes) = resolved_with_issues(name, &listed, &[], players_txt);
        assert_eq!(codes, structure_codes, "the structure pass's findings");
        let source = ExportSource {
            export_id: ExportId(0),
            path: root.to_path_buf(),
            kind: SourceKind::Folder,
            file_name: name.to_owned(),
            display_name: name.to_owned(),
            team_name: None,
        };
        let content = ContentSource::new(&source, &MemoryBudget::new(1 << 30));
        content_findings(&resolved.export, &content, version, None, false)
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
    fn the_fmdls_holding_a_metal_material_are_recorded_whatever_the_target() {
        // The tracer's right glove with its material's shader spelled as Fox techniques are,
        // which the converter's rule reads in any letter case; its boots with a shader the
        // converter reads as glass although it names `ggx`; the tracer's own models are not
        // metal.
        let metal = edited(&tracer_file("glove_r.fmdl"), |model| {
            "fox3DDF_GGX".clone_into(&mut model.materials[0].shader);
        });
        let glass = edited(&tracer_boots(), |model| {
            "fox3ddf_glass_ggx".clone_into(&mut model.materials[0].shader);
        });
        let files = [
            ("Players/05 - A/glove_r.fmdl", metal),
            ("Players/05 - A/boots.fmdl", glass),
            ("Players/06 - B/fcl_hair.fmdl", tracer_file("fcl_hair.fmdl")),
            ("Players/06 - B/glove_l.fmdl", tracer_file("glove_l.fmdl")),
        ];
        let recorded = |version: PesVersion, name: &str| {
            let temp = scratch(name);
            let pass = pass_for(version, temp.path(), &files, &[], &[]);
            pass.metal_models
                .iter()
                .map(|path| path.as_str().to_owned())
                .collect::<Vec<_>>()
        };
        // Planning, not the pass, knows which target converts one (`plan::converts_metal`).
        assert_eq!(
            recorded(PesVersion::Pes17, "deep_metal_17"),
            ["Players/05 - A/glove_r.fmdl"]
        );
        assert_eq!(
            recorded(PesVersion::Pes21, "deep_metal_21"),
            ["Players/05 - A/glove_r.fmdl"]
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
    fn a_common_file_is_dropped_unless_pass_through_keeps_its_every_dropping_error() {
        let file = IssueScope::File(path("Common/legs.mtl"));
        let finding = |code, disposition, eligible| {
            counted(code, &file, "legs.mtl", 1, disposition, eligible)
        };
        let eligible = finding("mtl_state_invalid", Disposition::DropFile, true);
        let not_eligible = finding("mtl_broken", Disposition::DropFile, false);
        let warning = finding("mtl_texture_not_found", Disposition::Keep, false);
        for pass_through in [false, true] {
            assert!(!drops_file(&[], pass_through));
            assert!(!drops_file(slice::from_ref(&warning), pass_through));
            assert!(drops_file(slice::from_ref(&not_eligible), pass_through));
            assert!(drops_file(
                &[eligible.clone(), not_eligible.clone()],
                pass_through
            ));
        }
        assert!(drops_file(slice::from_ref(&eligible), false));
        assert!(!drops_file(slice::from_ref(&eligible), true));
        assert!(!drops_file(&[eligible.clone(), warning.clone()], true));
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
                ("texture.dds", bc1_dds(4, 4)),
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
                ("texture.dds", bc1_dds(4, 4)),
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
        let findings_on = |version| {
            findings_for(
                version,
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
                // The texture both `.mtl` files name: the player's through his linked face.
                ("Faces/Round/texture.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
            )
        };
        // No `xml_model_not_found` and no `model_material_undefined`: the reference is found.
        // His own xml beside the link is the one finding.
        assert_eq!(
            findings_on(PesVersion::Pes17),
            [ContentFinding {
                code: "xml_shared_face_conflict",
                scope: IssueScope::Folder(ScopePath::new("Players/05 - A").unwrap()),
                context: vec![
                    ("file", "face.xml".to_owned()),
                    ("link", "Faces/Round".to_owned()),
                ],
                disposition: Disposition::DropFolder,
                pass_through_eligible: false,
            }]
        );
        // Fox ignores every `face.xml` (`xml_ignored_fox`, a validation finding): no conflict.
        assert_eq!(findings_on(PesVersion::Pes21), []);
    }

    #[test]
    fn an_xml_naming_an_absent_model_drops_its_folder_and_compares_no_material() {
        let findings = slot_05_findings(
            "deep_xml_absent",
            &[
                ("hat.model", card()),
                // Both name `./texture.dds`, which the folder lacks: with the xml's Error
                // dropping the folder, their textures are not looked for either.
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
                ("texture.dds", bc1_dds(4, 4)),
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

    /// The card head's material set with one material per entry of `materials`, each the card
    /// material renamed, its one sampler repeated once per path it names.
    fn materials_naming(materials: &[(&str, &[&str])]) -> Vec<u8> {
        use pes_model::format::mtl::{MaterialEntry, MaterialSet};
        let mut set = MaterialSet::read(&card_materials()).unwrap();
        let card = set.materials[0].clone();
        let MaterialEntry::Sampler(sampler) = &card.entries[0] else {
            panic!("the card material's first entry is its sampler");
        };
        set.materials = materials
            .iter()
            .map(|(name, paths)| {
                let mut material = card.clone();
                material.name = (*name).to_owned();
                let samplers = paths.iter().map(|path| {
                    let mut named = sampler.clone();
                    named.path = (*path).to_owned();
                    MaterialEntry::Sampler(named)
                });
                material.entries.splice(0..1, samplers);
                material
            })
            .collect();
        set.write()
    }

    /// The texture finding `code` of the `.mtl` named `file` on `scope`, about `texture`,
    /// named by `materials`; every texture finding keeps what holds the file.
    pub(super) fn mtl_texture(
        code: &'static str,
        scope: IssueScope,
        file: &str,
        texture: &str,
        materials: &str,
    ) -> ContentFinding {
        ContentFinding {
            code,
            scope,
            context: vec![
                ("file", file.to_owned()),
                ("texture", texture.to_owned()),
                ("materials", materials.to_owned()),
            ],
            disposition: Disposition::Keep,
            pass_through_eligible: false,
        }
    }

    #[test]
    fn a_shared_face_s_mtl_counts_its_own_textures_and_a_player_s_his_linked_face_s() {
        let temp = scratch("deep_mtl_shared_face");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - A/Round.face", Vec::new()),
                ("Players/05 - A/face_high.model", card()),
                (
                    "Players/05 - A/face_high.mtl",
                    materials_naming(&[("card", &["./round.dds"])]),
                ),
                ("Players/05 - A/skin.dds", bc1_dds(4, 4)),
                ("Faces/Round/hair_high.model", card()),
                (
                    "Faces/Round/hair_high.mtl",
                    materials_naming(&[("card", &["./skin.dds"])]),
                ),
                ("Faces/Round/round.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
        );
        // A shared face is complete by itself: the player's `skin.dds` does not count for it.
        assert_eq!(
            findings,
            [mtl_texture(
                "mtl_texture_not_found",
                folder("Faces/Round"),
                "hair_high.mtl",
                "./skin.dds",
                "card"
            )]
        );
    }

    /// The player folder `player` holding the card head's `face_high.model`, a `face_high.mtl`
    /// naming `./studs.dds` and a link to `Boots/Studs`, which holds the card head as
    /// `boots.model`, a `boots.mtl` naming `./studs.dds`, and `studs.dds` itself.
    fn studs_boots_layout(player: &str) -> Vec<(String, Vec<u8>)> {
        let studs = || materials_naming(&[("card", &["./studs.dds"])]);
        vec![
            (format!("{player}/Studs.boots"), Vec::new()),
            (format!("{player}/face_high.model"), card()),
            (format!("{player}/face_high.mtl"), studs()),
            ("Boots/Studs/boots.model".to_owned(), card()),
            ("Boots/Studs/boots.mtl".to_owned(), studs()),
            ("Boots/Studs/studs.dds".to_owned(), bc1_dds(4, 4)),
        ]
    }

    /// `files` with borrowed paths, as the passes take them.
    fn borrowed(files: &[(String, Vec<u8>)]) -> Vec<(&str, Vec<u8>)> {
        files
            .iter()
            .map(|(path, bytes)| (path.as_str(), bytes.clone()))
            .collect()
    }

    #[test]
    fn a_referee_s_face_holds_his_combined_boots_textures_and_a_team_player_s_does_not() {
        // A referee's every link feeds his own package, so his face packs the boots' textures.
        let temp = scratch("deep_mtl_referee_boots");
        let pass = export_pass(
            "refs Midcup Deep",
            Some(b"01 Ref A\n"),
            PesVersion::Pes17,
            temp.path(),
            &borrowed(&studs_boots_layout("Players/Ref A")),
            &[],
            &[],
        );
        assert_eq!(pass.findings, []);
        // A team player's boots link stays plain: his face holds no `studs.dds`.
        let temp = scratch("deep_mtl_team_boots");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &borrowed(&studs_boots_layout("Players/05 - A")),
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [mtl_texture(
                "mtl_texture_not_found",
                folder("Players/05 - A"),
                "face_high.mtl",
                "./studs.dds",
                "card"
            )]
        );
    }

    #[test]
    fn the_stem_a_texture_link_stands_for_is_supplied() {
        let temp = scratch("deep_mtl_texture_link");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - A/face_high.model", card()),
                (
                    "Players/05 - A/face_high.mtl",
                    materials_naming(&[("card", &["./hair.dds"])]),
                ),
                ("Players/05 - A/hair.dds.common", Vec::new()),
                ("Common/hair.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
        );
        assert_eq!(findings, []);
    }

    #[test]
    fn a_mtl_no_parsed_model_binds_reports_its_misses_as_info() {
        let temp = scratch("deep_mtl_unpaired");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - A/face_high.model", card()),
                ("Players/05 - A/face_high.mtl", card_materials()),
                ("Players/05 - A/texture.dds", bc1_dds(4, 4)),
                // No model's search finds it.
                (
                    "Players/05 - A/spare.mtl",
                    materials_naming(&[("card", &["./gone.dds"])]),
                ),
                // The model does not parse, so it binds nothing.
                ("Players/06 - B/face_high.model", b"not a model".to_vec()),
                ("Players/06 - B/face_high.mtl", card_materials()),
            ],
            &[],
            &[],
        );
        let error = pes_model::format::PreFoxModel::read(b"not a model")
            .unwrap_err()
            .to_string();
        assert_eq!(
            findings,
            [
                mtl_texture(
                    "mtl_texture_unused_missing",
                    folder("Players/05 - A"),
                    "spare.mtl",
                    "./gone.dds",
                    "card"
                ),
                dropping(
                    "model_broken",
                    "Players/06 - B",
                    &[("file", "face_high.model"), ("error", error.as_str())]
                ),
                mtl_texture(
                    "mtl_texture_unused_missing",
                    folder("Players/06 - B"),
                    "face_high.mtl",
                    "./texture.dds",
                    "card"
                ),
            ]
        );
    }

    #[test]
    fn a_material_the_model_lists_but_no_mesh_binds_is_not_mesh_used() {
        let temp = scratch("deep_mtl_listed_unused");
        let file = pes_model::format::PreFoxModel::read(&card()).unwrap();
        let mut model = pes_model::model::Model::from_file(&file).unwrap();
        model.materials.push("spare".to_owned());
        let listing_spare = model.to_file().unwrap().write().unwrap();
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - A/face_high.model", listing_spare),
                (
                    "Players/05 - A/face_high.mtl",
                    materials_naming(&[("card", &["./skin.dds"]), ("spare", &["./gone.dds"])]),
                ),
                ("Players/05 - A/skin.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [
                counted(
                    "model_material_unused",
                    &folder("Players/05 - A"),
                    "face_high.model",
                    1,
                    Disposition::Keep,
                    false
                ),
                mtl_texture(
                    "mtl_texture_unused_missing",
                    folder("Players/05 - A"),
                    "face_high.mtl",
                    "./gone.dds",
                    "spare"
                ),
            ]
        );
    }

    #[test]
    fn a_common_mtl_s_used_materials_are_the_common_models_and_its_miss_is_a_warning_on_the_file() {
        let temp = scratch("deep_mtl_common");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Common/legs.model", card()),
                (
                    "Common/legs.mtl",
                    materials_naming(&[("card", &["./gone.dds"])]),
                ),
                (
                    "Common/spare.mtl",
                    materials_naming(&[("other", &["./lost.dds"])]),
                ),
                // Its search finds `legs.mtl`, still kept, which defines `card`: nothing here.
                ("Players/05 - B/legs.model.common", Vec::new()),
            ],
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [
                mtl_texture(
                    "mtl_texture_not_found",
                    IssueScope::File(path("Common/legs.mtl")),
                    "legs.mtl",
                    "./gone.dds",
                    "card"
                ),
                mtl_texture(
                    "mtl_texture_unused_missing",
                    IssueScope::File(path("Common/spare.mtl")),
                    "spare.mtl",
                    "./lost.dds",
                    "other"
                ),
            ]
        );
    }
}
