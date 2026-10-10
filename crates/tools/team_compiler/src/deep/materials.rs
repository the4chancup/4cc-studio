//! The deep pass's texture lookup for a pre-Fox `.mtl` (`team_compiler/messages.md`, the
//! paragraph after "Texture existence is checked **deep**"): each texture path a `.mtl` names
//! must be supplied, by the stem rule the face task points the paths with
//! (`processing::prefox_face`; on Fox, which reads only a `.mtl` a selected `.model` pairs
//! with, the Models task, `processing::model`), so a path this calls supplied is one the task
//! points and a path it calls missing is one the task leaves as written. A missing texture of
//! a material a paired model's mesh binds is `mtl_texture_not_found`, a Warning that keeps
//! what holds the `.mtl`: which samplers a pre-Fox shader reads is not known, and the game
//! plays faces with such a miss. One only materials no mesh binds name is
//! `mtl_texture_unused_missing`, an Info, since the game never loads it.

use std::collections::BTreeSet;

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, IssueScope, SharedKind, SharedModelFolder,
};
use pes_version::Engine;
use vtree::ScopePath;

use super::model::MaterialRead;
use crate::kit_variants::has_variant_among;
use crate::messages::Code;
use crate::plan::roles::{FolderModels, PlayerFile, file_stem, player_file};
use crate::texture_lookup::{self, TextureFolders, TexturePlace};
use crate::user_face_xml::{Reference, reference};

/// Where the textures a `.mtl`'s paths name may come from, each by its folded stem.
pub(super) struct TextureSources<'a> {
    /// The places a texture name of the `.mtl` resolves in, nearest first
    /// (`HeldTextures::of`): its model folder's textures from the `.mtl`'s own folder up, those
    /// of the shared folders its face packs, and the stems its texture links stand for; for a
    /// `Common/` `.mtl`, `Common/`'s.
    pub(super) held: &'a [&'a TexturePlace],
    /// For a path below the `.mtl`'s folder (`texture_lookup::path_below`), which resolves at
    /// that path alone: its model folder's textures and the `.mtl`'s path (`HeldTextures::at`).
    /// `None` for a `Common/` `.mtl`, whose path below its folder is not looked for: the
    /// Common output holds that subfolder at its path, which `held` does not look in.
    pub(super) below: Option<(&'a HeldTextures, &'a ScopePath)>,
    /// The stems of the textures in `Common/` the pass keeps, which the export's Common
    /// textures task packs into the team's Common output.
    pub(super) common: &'a BTreeSet<String>,
    /// The stems the installed CPKs listed before the run's hold in the team's Common output;
    /// `None` when that lookup cannot be made or the export has no team ID, which holds
    /// nothing.
    pub(super) installed: Option<&'a BTreeSet<String>>,
}

/// Whether the texture a `.mtl` path names is supplied (`supply`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Supply {
    /// Supplied, or not looked for: the game's own texture, or a `dummy_` one it substitutes.
    Supplied,
    /// Nobody supplies it: neither the folder, nor for a Common path `Common/` or an installed
    /// CPK loaded before the run's.
    Missing,
}

/// Whether the texture `path`, a `.mtl` sampler's path as written, is supplied from `sources`.
/// A `dummy_` stem is never looked for. A path below the `.mtl`'s folder
/// (`./shorts/y.dds`, `texture_lookup::path_below`), when `sources.below` is set, is supplied
/// when the place that path names holds its stem (folded), or a variant of its set for a kit
/// reference, and missing otherwise: it resolves there alone; for a `Common/` `.mtl`
/// (`sources.below` unset) it is not looked for. Whatever directory any other
/// path spells, it is supplied when its stem, or a variant of its set for a kit reference
/// (`pants_kitN`, `texture_lookup::variant`), is one a place of `sources.held` holds: that is
/// the stem the face task points at the folder's textures. Past the folder, a `./` path and a
/// bare name (read as `./`) are missing: the face packs no texture the folder does not hold. A
/// path into the team's uniform Common folder
/// (`model/character/uniform/common/<team>/<name>`) is supplied when `Common/` or an installed
/// CPK holds its stem; an installed lookup that cannot be made holds nothing. Any other path
/// names the game's own files, which are not looked in.
fn supply(path: &str, sources: &TextureSources) -> Supply {
    let (directory, file_name) = texture_lookup::split(path);
    let stem = file_stem(file_name);
    let key = vtree::fold_name(stem);
    let holds = |stems: &BTreeSet<String>| {
        stems.contains(&key) || has_variant_among(stem, stems.iter().map(String::as_str))
    };
    let place_holds = |place: &TexturePlace| {
        place.contains_key(&key) || texture_lookup::variant(place, stem).is_some()
    };
    if key.starts_with("dummy_") {
        return Supply::Supplied;
    }
    if let Some(subdirectory) = texture_lookup::path_below(directory) {
        return match sources.below {
            Some((held, mtl))
                if held
                    .at(mtl, subdirectory)
                    .iter()
                    .any(|place| place_holds(place)) =>
            {
                Supply::Supplied
            }
            Some(_) => Supply::Missing,
            // The Common output holds a `Common/` `.mtl`'s subfolder at its path, which
            // `sources`, holding `Common/`'s own stems, does not look in: not looked for.
            None => Supply::Supplied,
        };
    }
    if sources.held.iter().any(|place| place_holds(place)) {
        return Supply::Supplied;
    }
    if !path.contains('/') {
        return Supply::Missing;
    }
    match reference(path) {
        Reference::Local(_) => Supply::Missing,
        // A Common subfolder's textures are not among `sources`, which hold `Common/`'s own
        // stems: such a path is not looked for, as a path of the game's own is not.
        Reference::Common { file_name, .. } if file_name.contains('/') => Supply::Supplied,
        Reference::Common { .. } if holds(sources.common) => Supply::Supplied,
        Reference::Common { .. } if sources.installed.is_some_and(holds) => Supply::Supplied,
        Reference::Common { .. } => Supply::Missing,
        Reference::Unchecked(_) => Supply::Supplied,
    }
}

/// `mtl_texture_not_found` and `mtl_texture_unused_missing` on `scope` for the `.mtl` whose
/// `materials` are read, which the findings name `name`: one finding per distinct texture path
/// (compared as written, in the order the `.mtl` first names it) that `sources` do not supply,
/// naming the file, the path as written and the materials naming it, in the `.mtl`'s order.
/// When one of those materials is among `used`, the names the meshes of the models paired with
/// the `.mtl` bind, it is `mtl_texture_not_found`, else `mtl_texture_unused_missing`; both keep
/// what holds the `.mtl`.
pub(super) fn texture_findings(
    name: &str,
    materials: &[MaterialRead],
    used: &BTreeSet<&str>,
    sources: &TextureSources,
    scope: &IssueScope,
) -> Vec<ContentFinding> {
    // Each distinct path with the materials naming it, in the order the `.mtl` names them.
    let mut named: Vec<(&str, Vec<&str>)> = Vec::new();
    for material in materials {
        for path in &material.paths {
            let index = match named.iter().position(|(known, _)| known == path) {
                Some(index) => index,
                None => {
                    named.push((path, Vec::new()));
                    named.len() - 1
                }
            };
            let naming = &mut named[index].1;
            if !naming.contains(&material.name.as_str()) {
                naming.push(&material.name);
            }
        }
    }
    named
        .into_iter()
        .filter(|(path, _)| supply(path, sources) == Supply::Missing)
        .map(|(path, naming)| {
            let code = if naming.iter().any(|material| used.contains(material)) {
                Code::MtlTextureNotFound
            } else {
                Code::MtlTextureUnusedMissing
            };
            ContentFinding {
                code: code.as_str(),
                scope: scope.clone(),
                context: vec![
                    ("file", name.to_owned()),
                    ("texture", path.to_owned()),
                    ("materials", naming.join(", ")),
                ],
                disposition: Disposition::Keep,
                pass_through_eligible: false,
            }
        })
        .collect()
}

/// The textures a model folder's `.mtl` paths may name, as the face task collects them
/// (`processing::prefox_face::face`), built once per folder (`held_textures`) and sliced per
/// `.mtl` (`of`).
pub(super) struct HeldTextures {
    /// The model folder.
    folder: ScopePath,
    /// Its textures (`PlayerFile::Texture`) by the folder of its tree holding each, those of
    /// the shared folders its packages are built from, and their texture links the same way
    /// (`PlayerFile::CommonTexture`): a link counts as the texture it stands for being present
    /// in the folder holding it.
    textures: TextureFolders,
}

impl HeldTextures {
    /// The places a texture name of the `.mtl` at `mtl` resolves in, nearest first from its
    /// own folder (`texture_lookup`): each folder's textures, then its links.
    pub(super) fn of(&self, mtl: &ScopePath) -> Vec<&TexturePlace> {
        self.textures
            .nearest_first(&self.folder, mtl)
            .into_iter()
            .map(|(place, _)| place)
            .collect()
    }

    /// The places a path of the `.mtl` at `mtl` naming `subdirectory` below its folder
    /// resolves in alone (`TextureFolders::at`).
    pub(super) fn at(&self, mtl: &ScopePath, subdirectory: &str) -> Vec<&TexturePlace> {
        self.textures
            .at(&self.folder, mtl, subdirectory)
            .into_iter()
            .map(|(place, _)| place)
            .collect()
    }
}

/// The textures the model folder at `folder` holds for its `.mtl` paths: each of its `files`
/// and of the files of the `shared` folders its packages are built from that is a texture
/// (`PlayerFile::Texture`), and the `Common/` path each texture link among them stands for
/// (`PlayerFile::CommonTexture`; the link's own spelling, `Common/`'s being read only at task
/// time, and only the stems being looked up here); `models` are the folder's, read for a
/// target of `engine`.
pub(super) fn held_textures(
    folder: &ScopePath,
    files: &[FileDescriptor],
    models: &FolderModels,
    shared: &[(SharedKind, &SharedModelFolder)],
    engine: Engine,
) -> HeldTextures {
    let mut held = HeldTextures {
        folder: folder.clone(),
        textures: TextureFolders::default(),
    };
    // Each role with whether it is one of the folder's own files.
    let mut roles: Vec<(bool, PlayerFile)> = files
        .iter()
        .filter_map(|file| player_file(folder, file, models))
        .map(|role| (true, role))
        .collect();
    for (kind, source) in shared {
        let source_models = FolderModels::of_shared(&source.path, &source.files, *kind, engine);
        roles.extend(
            source
                .files
                .iter()
                .filter_map(|file| player_file(&source.path, file, &source_models))
                .map(|role| (false, role)),
        );
    }
    for (own, role) in roles {
        match role {
            PlayerFile::Texture { below, .. } => held.textures.insert(own, &below),
            PlayerFile::CommonTexture(below) => held.textures.insert_link(own, &below),
            PlayerFile::Model { .. }
            | PlayerFile::CommonModel { .. }
            | PlayerFile::Packed { .. }
            | PlayerFile::FaceDiffXml
            | PlayerFile::Skeleton { .. }
            | PlayerFile::SlotlessSkeleton
            | PlayerFile::LeftOutKitVariant
            | PlayerFile::PreFoxModel { .. }
            | PlayerFile::PreFoxPart { .. }
            | PlayerFile::Material
            | PlayerFile::PreFoxCommonModel { .. }
            | PlayerFile::CommonMaterial
            | PlayerFile::FaceXml
            | PlayerFile::UnusedFaceFile
            | PlayerFile::ConversionSkeleton
            | PlayerFile::UnsupportedGltf => {}
        }
    }
    held
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The folded stems `stems`.
    fn stems(stems: &[&str]) -> BTreeSet<String> {
        stems.iter().map(|stem| (*stem).to_owned()).collect()
    }

    /// A place holding the textures `stems`, each directly in its folder.
    fn place(stems: &[&str]) -> TexturePlace {
        stems
            .iter()
            .map(|stem| (vtree::fold_name(stem), (*stem).to_owned()))
            .collect()
    }

    /// What `supply` makes of `path` with the folder holding `held`, `Common/` `common` and the
    /// installed CPKs `installed`.
    fn supplied(path: &str, held: &[&str], common: &[&str], installed: Option<&[&str]>) -> Supply {
        let installed = installed.map(stems);
        let sources = TextureSources {
            held: &[&place(held)],
            below: None,
            common: &stems(common),
            installed: installed.as_ref(),
        };
        supply(path, &sources)
    }

    /// Slot 05's held textures, its own at each of `below` and its links standing for
    /// `linked`.
    fn slot_05(below: &[&str], linked: &[&str]) -> HeldTextures {
        let mut held = HeldTextures {
            folder: ScopePath::new("Players/05 - A").unwrap(),
            textures: TextureFolders::default(),
        };
        for below in below {
            held.textures.insert(true, below);
        }
        for linked in linked {
            held.textures.insert_link(true, linked);
        }
        held
    }

    /// What `supply` makes of `path` named by the `.mtl` at `mtl` of the folder holding `held`,
    /// with nothing in `Common/` and no installed lookup.
    fn supplied_to(held: &HeldTextures, mtl: &str, path: &str) -> Supply {
        let mtl = ScopePath::new(mtl).unwrap();
        let places = held.of(&mtl);
        let sources = TextureSources {
            held: &places,
            below: Some((held, &mtl)),
            common: &BTreeSet::new(),
            installed: None,
        };
        supply(path, &sources)
    }

    #[test]
    fn a_mtl_path_below_its_folder_is_supplied_by_the_texture_at_that_path_alone() {
        let body = "Players/05 - A/jessie/body/x.mtl";
        let root = "Players/05 - A/face_high.mtl";
        let below = slot_05(
            &["jessie/body/shorts/y", "jessie/body/shorts/pants_kit1"],
            &[],
        );
        assert_eq!(
            supplied_to(&below, body, "./shorts/y.dds"),
            Supply::Supplied
        );
        assert_eq!(
            supplied_to(&below, body, "./Shorts/pants_kitN.dds"),
            Supply::Supplied
        );
        assert_eq!(supplied_to(&below, root, "./shorts/y.dds"), Supply::Missing);
        // The root's own `y` and a link of its stem are not at that path.
        let beside = slot_05(&["y"], &["y"]);
        assert_eq!(
            supplied_to(&beside, root, "./shorts/y.dds"),
            Supply::Missing
        );
        assert_eq!(
            supplied_to(&beside, root, "./shorts/dummy_kit.dds"),
            Supply::Supplied
        );
    }

    #[test]
    fn a_mtl_s_texture_name_is_held_in_its_folder_or_a_parent_never_a_subfolder() {
        let held = slot_05(&["jessie/skin", "jessie/shorts"], &["hair"]);
        let supplied_to = |mtl: &str, path: &str| supplied_to(&held, mtl, path);
        let deep = "Players/05 - A/jessie/body/x.mtl";
        assert_eq!(supplied_to(deep, "./skin.dds"), Supply::Supplied);
        assert_eq!(supplied_to(deep, "./hair.dds"), Supply::Supplied);
        let root = "Players/05 - A/face_high.mtl";
        assert_eq!(supplied_to(root, "./shorts.dds"), Supply::Missing);
        assert_eq!(supplied_to(root, "./hair.dds"), Supply::Supplied);
    }

    const COMMON_HAIR: &str = "model/character/uniform/common/XXX/hair.dds";

    #[test]
    fn a_stem_the_folder_holds_is_supplied_whatever_directory_and_case_the_path_spells() {
        for path in [
            "./skin.dds",
            "./Skin.DDS",
            "skin.dds",
            "model/character/uniform/common/XXX/SKIN.dds",
            "model/character/face/real/skin.dds",
        ] {
            assert_eq!(
                supplied(path, &["skin"], &[], None),
                Supply::Supplied,
                "{path}"
            );
        }
    }

    #[test]
    fn a_kit_reference_is_supplied_by_a_variant_of_its_set() {
        assert_eq!(
            supplied("./pants_kitN.dds", &["pants_kit1"], &[], None),
            Supply::Supplied
        );
        assert_eq!(
            supplied("./pants_kitN.dds", &["socks_kit1"], &[], None),
            Supply::Missing
        );
    }

    #[test]
    fn a_dummy_stem_and_a_game_path_are_not_looked_for() {
        assert_eq!(
            supplied("./dummy_kit_01.dds", &[], &[], Some(&[])),
            Supply::Supplied
        );
        assert_eq!(
            supplied("./DUMMY_kit_01.dds", &[], &[], Some(&[])),
            Supply::Supplied
        );
        assert_eq!(
            supplied(
                "model/character/face/common/face_eyelash.dds",
                &[],
                &[],
                Some(&[])
            ),
            Supply::Supplied
        );
        // A `./` path into a subfolder of a `Common/` `.mtl` (no folder to look below) names a
        // texture the Common output holds at that path, which `sources` do not look in: it is
        // not looked for.
        assert_eq!(
            supplied("./sub/skin.dds", &[], &[], Some(&[])),
            Supply::Supplied
        );
        assert_eq!(
            supplied("./sub/skin.dds", &["skin"], &[], Some(&[])),
            Supply::Supplied
        );
        // A Common path into a subfolder is not looked up among `Common/`'s own stems.
        assert_eq!(
            supplied(
                "model/character/uniform/common/XXX/refkit/skin.dds",
                &[],
                &[],
                Some(&[])
            ),
            Supply::Supplied
        );
    }

    #[test]
    fn a_local_or_bare_path_the_folder_does_not_hold_is_missing() {
        // `Common/` and the installed CPKs do not count for a texture of the face.
        for path in ["./skin.dds", "skin.dds"] {
            assert_eq!(
                supplied(path, &["hair"], &["skin"], Some(&["skin"])),
                Supply::Missing,
                "{path}"
            );
        }
    }

    #[test]
    fn a_common_path_is_supplied_by_common_or_an_installed_cpk_and_missing_without_a_lookup() {
        assert_eq!(
            supplied(COMMON_HAIR, &[], &["hair"], None),
            Supply::Supplied
        );
        assert_eq!(
            supplied(COMMON_HAIR, &[], &[], Some(&["hair"])),
            Supply::Supplied
        );
        assert_eq!(
            supplied(COMMON_HAIR, &[], &["skin"], Some(&["skin"])),
            Supply::Missing
        );
        // A lookup that cannot be made holds nothing.
        assert_eq!(supplied(COMMON_HAIR, &[], &[], None), Supply::Missing);
    }

    /// A material set's `name` naming `paths`.
    fn material(name: &str, paths: &[&str]) -> MaterialRead {
        MaterialRead {
            name: name.to_owned(),
            paths: paths.iter().map(|path| (*path).to_owned()).collect(),
            mesh_used: false,
        }
    }

    /// The findings of `texture_findings` for `materials` of `face.mtl` in `Players/05 - A`,
    /// `used` mesh-used, nothing held but `Common/`'s `hair` and no installed lookup.
    fn findings(materials: &[MaterialRead], used: &[&str]) -> Vec<ContentFinding> {
        let common = stems(&["hair"]);
        let sources = TextureSources {
            held: &[],
            below: None,
            common: &common,
            installed: None,
        };
        let scope = IssueScope::Folder(ScopePath::new("Players/05 - A").unwrap());
        texture_findings(
            "face.mtl",
            materials,
            &used.iter().copied().collect(),
            &sources,
            &scope,
        )
    }

    /// The finding `code` on `Players/05 - A` about `face.mtl`'s `texture`, named by
    /// `materials`; every texture finding keeps the folder.
    fn finding(code: &'static str, texture: &str, materials: &str) -> ContentFinding {
        ContentFinding {
            code,
            scope: IssueScope::Folder(ScopePath::new("Players/05 - A").unwrap()),
            context: vec![
                ("file", "face.mtl".to_owned()),
                ("texture", texture.to_owned()),
                ("materials", materials.to_owned()),
            ],
            disposition: Disposition::Keep,
            pass_through_eligible: false,
        }
    }

    #[test]
    fn one_path_several_samplers_name_is_one_finding_naming_each_material_once() {
        let found = findings(
            &[
                material("unused", &["./skin.dds"]),
                material("head", &["./skin.dds", "./skin.dds", "./eye.dds"]),
            ],
            &["head"],
        );
        assert_eq!(
            found,
            [
                finding("mtl_texture_not_found", "./skin.dds", "unused, head"),
                finding("mtl_texture_not_found", "./eye.dds", "head"),
            ]
        );
    }

    #[test]
    fn a_miss_only_unused_materials_name_is_info_and_one_without_a_lookup_is_the_same_warning() {
        let gone = "model/character/uniform/common/XXX/gone.dds";
        let cap = "model/character/uniform/common/XXX/cap.dds";
        let found = findings(
            &[
                material("unused", &["./skin.dds", COMMON_HAIR, gone]),
                material("head", &[cap, "./eye.dds"]),
            ],
            &["head"],
        );
        assert_eq!(
            found,
            [
                finding("mtl_texture_unused_missing", "./skin.dds", "unused"),
                finding("mtl_texture_unused_missing", gone, "unused"),
                finding("mtl_texture_not_found", cap, "head"),
                finding("mtl_texture_not_found", "./eye.dds", "head"),
            ]
        );
    }
}
