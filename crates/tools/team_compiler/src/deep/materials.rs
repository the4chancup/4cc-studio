//! The deep pass's texture lookup for a pre-Fox `.mtl` (`team_compiler/messages.md`, the
//! paragraph after "Texture existence is checked **deep**"): each texture path a `.mtl` names
//! must be supplied, by the stem rule the face task points the paths with
//! (`processing::prefox_face`), so a path this calls supplied is one the task points and a path
//! it calls missing is one the task leaves as written. A missing texture of a material a paired
//! model's mesh binds is `mtl_texture_not_found`, a Warning that keeps what holds the `.mtl`:
//! which samplers a pre-Fox shader reads is not known, and the game plays faces with such a
//! miss. One only materials no mesh binds name is `mtl_texture_unused_missing`, an Info, since
//! the game never loads it.

use std::collections::BTreeSet;

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, IssueScope, SharedModelFolder,
};
use pes_version::Engine;
use vtree::ScopePath;

use super::model::MaterialRead;
use crate::kit_variants::has_variant_among;
use crate::messages::Code;
use crate::plan::subset::{FolderModels, PlayerFile, file_stem, player_file};
use crate::user_face_xml::{Reference, reference};

/// Where the textures a `.mtl`'s paths name may come from, each by its folded stem.
pub(super) struct TextureSources<'a> {
    /// The stems the `.mtl`'s folder holds: its own textures, those of the shared folders its
    /// face packs, and the stems its texture links stand for (`held_stems`); for a `Common/`
    /// `.mtl`, `Common/`'s.
    pub(super) held: &'a BTreeSet<String>,
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
/// Whatever directory it spells, it is supplied when its stem (folded), or a variant of its set
/// for a kit reference (`pants_kitN`), is one `sources.held` holds: that is the stem the face
/// task points at the folder's textures. A `dummy_` stem is never looked for. Past the folder, a
/// `./` path and a bare name (read as `./`) are missing: the face packs no texture the folder
/// does not hold. A path into the team's uniform Common folder
/// (`model/character/uniform/common/<team>/<name>`) is supplied when `Common/` or an installed
/// CPK holds its stem; an installed lookup that cannot be made holds nothing. Any other path
/// names the game's own files, which are not looked in.
fn supply(path: &str, sources: &TextureSources) -> Supply {
    let file_name = path.rsplit_once('/').map_or(path, |(_, name)| name);
    let stem = file_stem(file_name);
    let key = vtree::fold_name(stem);
    let holds = |stems: &BTreeSet<String>| {
        stems.contains(&key) || has_variant_among(stem, stems.iter().map(String::as_str))
    };
    if holds(sources.held) || key.starts_with("dummy_") {
        return Supply::Supplied;
    }
    if !path.contains('/') {
        return Supply::Missing;
    }
    match reference(path) {
        Reference::Local(_) => Supply::Missing,
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

/// The folded stems the model folder at `folder` holds for its `.mtl` paths, as the face task
/// collects them (`processing::prefox_face::face`): the stem of each of its `files` and of the
/// files of the `shared` folders its face packs that is a texture (`PlayerFile::Texture`), and
/// the stem each texture link among them stands for (`PlayerFile::CommonTexture`); `models`
/// are the folder's, read for a target of `engine`.
pub(super) fn held_stems(
    folder: &ScopePath,
    files: &[FileDescriptor],
    models: &FolderModels,
    shared: &[&SharedModelFolder],
    engine: Engine,
) -> BTreeSet<String> {
    let mut roles: Vec<PlayerFile> = files
        .iter()
        .filter_map(|file| player_file(folder, file, models))
        .collect();
    for source in shared {
        let source_models = FolderModels::of(&source.path, &source.files, engine);
        roles.extend(
            source
                .files
                .iter()
                .filter_map(|file| player_file(&source.path, file, &source_models)),
        );
    }
    roles
        .into_iter()
        .filter_map(|role| match role {
            PlayerFile::Texture(stem, _) | PlayerFile::CommonTexture(stem) => {
                Some(vtree::fold_name(&stem))
            }
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
            | PlayerFile::ConversionSkeleton => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The folded stems `stems`.
    fn stems(stems: &[&str]) -> BTreeSet<String> {
        stems.iter().map(|stem| (*stem).to_owned()).collect()
    }

    /// What `supply` makes of `path` with the folder holding `held`, `Common/` `common` and the
    /// installed CPKs `installed`.
    fn supplied(path: &str, held: &[&str], common: &[&str], installed: Option<&[&str]>) -> Supply {
        let installed = installed.map(stems);
        let sources = TextureSources {
            held: &stems(held),
            common: &stems(common),
            installed: installed.as_ref(),
        };
        supply(path, &sources)
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
        // A `./` path into a subfolder is no `./` reference (`reference` reads it as any
        // other form), so it is not looked up, although the face packs no subfolder.
        assert_eq!(
            supplied("./sub/skin.dds", &[], &[], Some(&[])),
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
            held: &BTreeSet::new(),
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
