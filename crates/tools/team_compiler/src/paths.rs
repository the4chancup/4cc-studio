//! Where compiled content goes inside the CPK, for the Fox versions (PES 18-21) and, for the
//! faces, the shared boots and gloves, a model folder's textures, the team's Common output, the
//! portraits and the logo, the pre-Fox ones (PES 15-17) too:
//! `team_compiler/pipeline.md` "Game paths reference". The CPK paths have no leading `/`.

use std::fs;
use std::path::Path;

use aesthetics_export::RefSlot;
use anyhow::Context;
use pes_version::{Engine, PesVersion};

use crate::plan::subset::ModelPackage;

/// The team id the referees' content carries in game paths and FMDL texture paths
/// (`common/999/Ref A/`, `pipeline.md` "3. Per-model-folder parallel steps", step 6). It is
/// not a `TeamId`: no teams-list row has it, and nothing is written to a team record under it.
pub(crate) const REFEREE_TEAM_ID: u16 = 999;

/// The stock collar the referees' marker takes (`blue_port.md` "Referee export processing"):
/// the marker model is written as this collar and the referee kit configs name it. A stock
/// collar of every target version that no kit config on the maintainer's machine uses, so no
/// team's players wear the marker by accident; a team kit naming it is `kit_collar_reserved`.
pub(crate) const REFEREE_MARKER_COLLAR: u8 = 77;

/// The folder of the referee kit configs in the refs CPK, loose files as the referee template
/// tree ships them (`referee_DEF_1.bin` and the rest).
pub(crate) const REFEREE_KIT_CONFIGS: &str =
    "common/character0/model/character/uniform/team/referee/";

/// The bin holding every team's kit configs, keyed by entry name.
pub(crate) const UNIFORM_PARAMETER: &str =
    "common/character0/model/character/uniform/team/UniformParameter.bin";

/// The bin holding every team's colors, one record per team; the same path on every version.
pub(crate) const TEAM_COLOR: &str = "common/etc/TeamColor.bin";

/// The bin holding every team's kit menu colors and icons, one record per team; the same path
/// on every version.
pub(crate) const UNI_COLOR: &str = "common/character0/model/character/uniform/team/UniColor.bin";

/// The Fox table giving each player with custom boots his boots ID: (player id, boots ID)
/// pairs, sorted by player id.
pub(crate) const BOOTS_LIST: &str = "common/character0/model/character/boots/BootsList.bin";

/// The Fox table giving each player with custom gloves his gloves ID: (player id, gloves ID)
/// pairs, sorted by player id.
pub(crate) const GLOVE_LIST: &str = "common/character0/model/character/glove/GloveList.bin";

/// The Fox table of every player's look: 60-byte rows, the player id then 56 appearance bytes.
pub(crate) const PLAYER_APPEARANCE: &str =
    "common/character0/model/character/appearance/PlayerAppearance.bin";

/// What a model package's game folder is named by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PackageKey {
    /// A team player's package: his player id for the face, his boots or gloves id for
    /// the other two (`face/real/71405`, `k0625`, `g0625`).
    Id(u32),
    /// Referee slot NN's package: `face/real/referee0NN`, `k99NN`, `g99NN`.
    Referee(RefSlot),
}

/// The game folder of one `package` by `key`, without the `Asset/` or `/Assets/pes16/` head
/// the CPK paths and the FMDL texture paths put before it: the player id for the face, the
/// four-digit boots or gloves id for the other two (`k0625`, `g0625`); for referee slot NN
/// `referee0NN`, `k99NN` and `g99NN` ("Game paths reference").
fn model_folder(package: ModelPackage, key: PackageKey) -> String {
    match (package, key) {
        (ModelPackage::Face, PackageKey::Id(id)) => format!("model/character/face/real/{id}"),
        (ModelPackage::Boots, PackageKey::Id(id)) => format!("model/character/boots/k{id:04}"),
        (ModelPackage::Gloves, PackageKey::Id(id)) => format!("model/character/glove/g{id:04}"),
        (ModelPackage::Face, PackageKey::Referee(slot)) => {
            format!("model/character/face/real/referee{:03}", slot.get())
        }
        (ModelPackage::Boots, PackageKey::Referee(slot)) => {
            format!("model/character/boots/k99{:02}", slot.get())
        }
        (ModelPackage::Gloves, PackageKey::Referee(slot)) => {
            format!("model/character/glove/g99{:02}", slot.get())
        }
    }
}

/// The folder of one `package` (its `.fpk` and `.fpkd`), by `key`.
pub(crate) fn package_folder(package: ModelPackage, key: PackageKey) -> String {
    format!("Asset/{}/#Win", model_folder(package, key))
}

/// The pre-Fox folder of one `package` by `key`
/// (`common/character0/model/character/face/real/71405`, `…/boots/k0644`): for the face, the
/// face CPK is this path with `.cpk`, and each of its entries is a file of this folder, its own
/// entries repeating the outer path as the game's face CPKs do; the boots and the gloves are
/// loose files in it.
pub(crate) fn pre_fox_package_folder(package: ModelPackage, key: PackageKey) -> String {
    format!("common/character0/{}", model_folder(package, key))
}

/// Where a model folder's textures go, which its models' texture paths name and its textures
/// task writes to (`pipeline.md` "3. Per-model-folder parallel steps", step 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TextureHome {
    /// A player folder's: the team's per-player common subfolder, keyed by the source
    /// folder's name, so a folder several slots map emits its textures once.
    PlayerCommon {
        /// The source folder's name (`05 - A`).
        folder_name: String,
    },
    /// A shared boots or gloves folder's: the output's own folder, beside its package (on
    /// pre-Fox beside its models, which name them `./<stem>.dds`), so the one output every
    /// linking player loads carries its own textures. Only the boots and the gloves are built
    /// this way; a shared face has no output of its own.
    SharedOutput {
        /// The output's package, boots or gloves.
        package: ModelPackage,
        /// Its shared id from the team's block.
        id: u32,
    },
}

impl TextureHome {
    /// The directory a model's texture paths name for a texture here, on a target of `engine`,
    /// for team `team_id`: an FMDL texture-path table's on Fox, a `.mtl` sampler's on pre-Fox.
    pub(crate) fn directory(&self, engine: Engine, team_id: u16) -> String {
        match (engine, self) {
            (Engine::Fox, TextureHome::PlayerCommon { folder_name }) => format!(
                "/Assets/pes16/model/character/common/{team_id}/{folder_name}/sourceimages/"
            ),
            (Engine::Fox, TextureHome::SharedOutput { package, id }) => {
                format!(
                    "/Assets/pes16/{}/",
                    model_folder(*package, PackageKey::Id(*id))
                )
            }
            (Engine::PreFox, TextureHome::PlayerCommon { folder_name }) => {
                format!("model/character/uniform/common/{team_id}/{folder_name}/")
            }
            // The game's own boots `.mtl` files name their textures beside them.
            (Engine::PreFox, TextureHome::SharedOutput { .. }) => "./".to_owned(),
        }
    }

    /// The CPK path of the texture `stem` here, converted for a target of `engine` (an FTEX on
    /// Fox, a DDS on pre-Fox), for team `team_id`.
    pub(crate) fn texture(&self, engine: Engine, team_id: u16, stem: &str) -> String {
        match (engine, self) {
            (Engine::Fox, TextureHome::PlayerCommon { folder_name }) => format!(
                "Asset/model/character/common/{team_id}/{folder_name}/sourceimages/#windx11/{stem}.ftex"
            ),
            (Engine::Fox, TextureHome::SharedOutput { package, id }) => format!(
                "Asset/{}/#windx11/{stem}.ftex",
                model_folder(*package, PackageKey::Id(*id))
            ),
            (Engine::PreFox, TextureHome::PlayerCommon { folder_name }) => format!(
                "common/character1/model/character/uniform/common/{team_id}/{folder_name}/{stem}.dds"
            ),
            (Engine::PreFox, TextureHome::SharedOutput { package, id }) => format!(
                "{}/{stem}.dds",
                pre_fox_package_folder(*package, PackageKey::Id(*id))
            ),
        }
    }
}

/// The directory a model's texture paths name for a texture of the team's Common output (the
/// Common row of "Game paths reference"), on a target of `engine`, for team `team_id`: an FMDL
/// texture-path table's on Fox, a `.mtl` sampler's on pre-Fox, where it is also the directory
/// a `face.xml` names a Common model or `.mtl` by. Where a texture that resolves into the
/// export's `Common/` folder stays, packed once for every player pointing at it (`pipeline.md`
/// "3. Per-model-folder parallel steps", step 6).
pub(crate) fn common_texture_directory(engine: Engine, team_id: u16) -> String {
    match engine {
        Engine::Fox => format!("/Assets/pes16/model/character/common/{team_id}/sourceimages/"),
        Engine::PreFox => format!("model/character/uniform/common/{team_id}/"),
    }
}

/// The CPK path of the texture `stem` of the team's Common output, converted for a target of
/// `engine` (an FTEX on Fox, a DDS on pre-Fox), for team `team_id`.
pub(crate) fn common_texture(engine: Engine, team_id: u16, stem: &str) -> String {
    match engine {
        Engine::Fox => {
            format!("Asset/model/character/common/{team_id}/sourceimages/#windx11/{stem}.ftex")
        }
        Engine::PreFox => pre_fox_common_file(team_id, &format!("{stem}.dds")),
    }
}

/// The CPK path of the file `name` in team `team_id`'s pre-Fox Common output: where a `Common/`
/// model goes under its packed name and a `Common/` `.mtl` under its own (`pipeline.md` "3.
/// Per-model-folder parallel steps", step 4).
pub(crate) fn pre_fox_common_file(team_id: u16, name: &str) -> String {
    format!(
        "common/character1/{}{name}",
        common_texture_directory(Engine::PreFox, team_id)
    )
}

/// The CPK path of stock collar `id`'s model on Fox, which a model at that path replaces: the
/// Collars row of "Game paths reference", the id zero-padded to three digits
/// (`collar_105.fmdl`). The pre-Fox collar path is not built yet.
pub(crate) fn collar(id: u8) -> String {
    format!("Asset/model/character/uniform/nocloth/#Win/collar_{id:03}.fmdl")
}

/// The CPK path of one kit texture, by its game name (`u0792g1`, `u0792g1_back`).
pub(crate) fn kit_texture(name: &str) -> String {
    format!("Asset/model/character/uniform/texture/#windx11/{name}.ftex")
}

/// The CPK path of one kit config, by its entry name (`792_DEF_GK1st_realUni.bin`).
pub(crate) fn kit_config(team_id: u16, entry_name: &str) -> String {
    format!("common/character0/model/character/uniform/team/{team_id}/{entry_name}")
}

/// The CPK path of one player's portrait, a DDS, by player id: the same folder on every
/// version, the file named `player_{player id}.dds` up to PES 2018 and `{player id}.dds` from
/// PES 2019.
pub(crate) fn portrait(version: PesVersion, player_id: u32) -> String {
    let prefix = match version {
        PesVersion::Pes15 | PesVersion::Pes16 | PesVersion::Pes17 | PesVersion::Pes18 => "player_",
        PesVersion::Pes19 | PesVersion::Pes20 | PesVersion::Pes21 => "",
    };
    format!("common/render/symbol/player/{prefix}{player_id}.dds")
}

/// The CPK paths of team `team_id`'s three logo PNGs, the 512-pixel one first, then the 256 and
/// the 128 (`_r_ll`, `_r_l`, `_r`): the same folder on every version, the files named
/// `emblem_0{team id}` up to PES 2019 and `e_000{team id}` from PES 2020.
pub(crate) fn logo(version: PesVersion, team_id: u16) -> [String; 3] {
    let stem = match version {
        PesVersion::Pes15
        | PesVersion::Pes16
        | PesVersion::Pes17
        | PesVersion::Pes18
        | PesVersion::Pes19 => format!("emblem_0{team_id}"),
        PesVersion::Pes20 | PesVersion::Pes21 => format!("e_000{team_id}"),
    };
    ["_r_ll", "_r_l", "_r"].map(|size| format!("common/render/symbol/flag/{stem}{size}.png"))
}

/// Puts `bytes` in place at `path` through a temporary file beside it, renamed onto it, so
/// `path` holds the previous file (or none) or the whole new one. The temporary file's name
/// carries the process id, so two runs writing into one folder stay apart; a failure of the
/// write or the rename removes it. Each failure names the file it concerns.
pub(crate) fn replace_file(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let name = path.file_name().map_or_else(
        || "file".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let temporary = path.with_file_name(format!(".{name}-{}.tmp", std::process::id()));
    if let Err(error) = fs::write(&temporary, bytes) {
        if let Err(removal) = fs::remove_file(&temporary) {
            log::debug!("{}: kept: {removal}", temporary.display());
        }
        return Err(error).with_context(|| format!("{}: cannot write it", temporary.display()));
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if let Err(removal) = fs::remove_file(&temporary) {
            log::debug!("{}: kept: {removal}", temporary.display());
        }
        return Err(error).with_context(|| format!("{}: cannot replace it", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::scratch;

    /// The names of the entries in `folder`, sorted.
    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn a_file_is_put_in_place_through_a_temporary_that_leaves_no_trace() {
        let temp = scratch("paths_replace_file");
        let root = temp.path();
        let path = root.join("the-file.txt");

        replace_file(&path, b"first").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"first");
        assert_eq!(names(root), ["the-file.txt"], "no temporary left");

        replace_file(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(names(root), ["the-file.txt"], "no temporary left");
    }

    #[test]
    fn a_target_that_cannot_be_replaced_is_an_error_naming_it_and_keeps_no_temporary() {
        let temp = scratch("paths_replace_file_blocked");
        let root = temp.path();
        // A folder in the file's place, with a file in it so no platform replaces it.
        let path = root.join("the-file.txt");
        fs::create_dir_all(path.join("kept")).unwrap();

        let error = replace_file(&path, b"text").unwrap_err();

        let chain = format!("{error:#}");
        assert!(
            chain.starts_with(&format!("{}: ", path.display())),
            "{chain}"
        );
        assert!(path.join("kept").is_dir());
        assert_eq!(names(root), ["the-file.txt"], "no temporary left");
    }

    #[test]
    fn a_face_goes_by_player_id_and_boots_and_gloves_by_four_digit_model_id() {
        assert_eq!(
            package_folder(ModelPackage::Face, PackageKey::Id(71405)),
            "Asset/model/character/face/real/71405/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Boots, PackageKey::Id(625)),
            "Asset/model/character/boots/k0625/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Gloves, PackageKey::Id(3745)),
            "Asset/model/character/glove/g3745/#Win"
        );
    }

    #[test]
    fn a_referee_s_packages_go_by_his_slot() {
        let referee = |slot| PackageKey::Referee(RefSlot::new(slot).unwrap());
        assert_eq!(
            package_folder(ModelPackage::Face, referee(1)),
            "Asset/model/character/face/real/referee001/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Boots, referee(1)),
            "Asset/model/character/boots/k9901/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Gloves, referee(1)),
            "Asset/model/character/glove/g9901/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Face, referee(35)),
            "Asset/model/character/face/real/referee035/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Boots, referee(35)),
            "Asset/model/character/boots/k9935/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Gloves, referee(20)),
            "Asset/model/character/glove/g9920/#Win"
        );
    }

    #[test]
    fn a_player_s_textures_go_to_its_common_subfolder_and_a_shared_output_s_to_its_own() {
        let player = TextureHome::PlayerCommon {
            folder_name: "05 - A".to_owned(),
        };
        assert_eq!(
            player.directory(Engine::Fox, 714),
            "/Assets/pes16/model/character/common/714/05 - A/sourceimages/"
        );
        assert_eq!(
            player.texture(Engine::Fox, 714, "shirt"),
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex"
        );
        let boots = TextureHome::SharedOutput {
            package: ModelPackage::Boots,
            id: 644,
        };
        assert_eq!(
            boots.directory(Engine::Fox, 714),
            "/Assets/pes16/model/character/boots/k0644/"
        );
        assert_eq!(
            boots.texture(Engine::Fox, 714, "shirt"),
            "Asset/model/character/boots/k0644/#windx11/shirt.ftex"
        );
        let gloves = TextureHome::SharedOutput {
            package: ModelPackage::Gloves,
            id: 644,
        };
        assert_eq!(
            gloves.directory(Engine::Fox, 714),
            "/Assets/pes16/model/character/glove/g0644/"
        );
        assert_eq!(
            gloves.texture(Engine::Fox, 714, "grip"),
            "Asset/model/character/glove/g0644/#windx11/grip.ftex"
        );
    }

    #[test]
    fn a_pre_fox_player_s_textures_go_to_its_common_subfolder_under_character1_as_dds() {
        let player = TextureHome::PlayerCommon {
            folder_name: "05 - A".to_owned(),
        };
        assert_eq!(
            player.directory(Engine::PreFox, 714),
            "model/character/uniform/common/714/05 - A/"
        );
        assert_eq!(
            player.texture(Engine::PreFox, 714, "skin"),
            "common/character1/model/character/uniform/common/714/05 - A/skin.dds"
        );
    }

    #[test]
    fn a_pre_fox_package_folder_goes_by_player_id_or_referee_slot_and_four_digit_model_id() {
        assert_eq!(
            pre_fox_package_folder(ModelPackage::Face, PackageKey::Id(71405)),
            "common/character0/model/character/face/real/71405"
        );
        assert_eq!(
            pre_fox_package_folder(
                ModelPackage::Face,
                PackageKey::Referee(RefSlot::new(3).unwrap())
            ),
            "common/character0/model/character/face/real/referee003"
        );
        assert_eq!(
            pre_fox_package_folder(ModelPackage::Boots, PackageKey::Id(644)),
            "common/character0/model/character/boots/k0644"
        );
        assert_eq!(
            pre_fox_package_folder(ModelPackage::Gloves, PackageKey::Id(644)),
            "common/character0/model/character/glove/g0644"
        );
    }

    #[test]
    fn a_pre_fox_shared_output_s_textures_sit_beside_its_models_named_from_its_folder() {
        let boots = TextureHome::SharedOutput {
            package: ModelPackage::Boots,
            id: 644,
        };
        assert_eq!(boots.directory(Engine::PreFox, 714), "./");
        assert_eq!(
            boots.texture(Engine::PreFox, 714, "crocs"),
            "common/character0/model/character/boots/k0644/crocs.dds"
        );
        let gloves = TextureHome::SharedOutput {
            package: ModelPackage::Gloves,
            id: 645,
        };
        assert_eq!(gloves.directory(Engine::PreFox, 714), "./");
        assert_eq!(
            gloves.texture(Engine::PreFox, 714, "grip"),
            "common/character0/model/character/glove/g0645/grip.dds"
        );
    }

    #[test]
    fn the_team_s_common_textures_sit_one_level_above_the_players_subfolders() {
        assert_eq!(
            common_texture_directory(Engine::Fox, 714),
            "/Assets/pes16/model/character/common/714/sourceimages/"
        );
        assert_eq!(
            common_texture(Engine::Fox, 714, "cloth"),
            "Asset/model/character/common/714/sourceimages/#windx11/cloth.ftex"
        );
    }

    #[test]
    fn the_team_s_pre_fox_common_output_holds_its_textures_models_and_mtl_files_under_character1() {
        assert_eq!(
            common_texture_directory(Engine::PreFox, 714),
            "model/character/uniform/common/714/"
        );
        assert_eq!(
            common_texture(Engine::PreFox, 714, "Cloth"),
            "common/character1/model/character/uniform/common/714/Cloth.dds"
        );
        assert_eq!(
            pre_fox_common_file(714, "oral_legs_win32.model"),
            "common/character1/model/character/uniform/common/714/oral_legs_win32.model"
        );
        assert_eq!(
            pre_fox_common_file(714, "Legs.mtl"),
            "common/character1/model/character/uniform/common/714/Legs.mtl"
        );
    }

    #[test]
    fn a_collar_goes_by_its_three_digit_id_among_the_nocloth_models() {
        assert_eq!(
            collar(105),
            "Asset/model/character/uniform/nocloth/#Win/collar_105.fmdl"
        );
        assert_eq!(
            collar(REFEREE_MARKER_COLLAR),
            "Asset/model/character/uniform/nocloth/#Win/collar_077.fmdl"
        );
        assert_eq!(
            collar(1),
            "Asset/model/character/uniform/nocloth/#Win/collar_001.fmdl"
        );
    }

    #[test]
    fn a_portrait_is_prefixed_up_to_pes_2018_and_bare_from_pes_2019() {
        for version in [
            PesVersion::Pes15,
            PesVersion::Pes16,
            PesVersion::Pes17,
            PesVersion::Pes18,
        ] {
            assert_eq!(
                portrait(version, 71405),
                "common/render/symbol/player/player_71405.dds",
                "{version}"
            );
        }
        for version in [PesVersion::Pes19, PesVersion::Pes20, PesVersion::Pes21] {
            assert_eq!(
                portrait(version, 71405),
                "common/render/symbol/player/71405.dds",
                "{version}"
            );
        }
    }

    #[test]
    fn a_logo_is_named_emblem_up_to_pes_2019_and_e_from_pes_2020_largest_first() {
        assert_eq!(
            logo(PesVersion::Pes19, 714),
            [
                "common/render/symbol/flag/emblem_0714_r_ll.png",
                "common/render/symbol/flag/emblem_0714_r_l.png",
                "common/render/symbol/flag/emblem_0714_r.png",
            ]
        );
        assert_eq!(
            logo(PesVersion::Pes20, 714),
            [
                "common/render/symbol/flag/e_000714_r_ll.png",
                "common/render/symbol/flag/e_000714_r_l.png",
                "common/render/symbol/flag/e_000714_r.png",
            ]
        );
        for version in [
            PesVersion::Pes15,
            PesVersion::Pes16,
            PesVersion::Pes17,
            PesVersion::Pes18,
        ] {
            assert_eq!(
                logo(version, 714),
                logo(PesVersion::Pes19, 714),
                "{version}"
            );
        }
        assert_eq!(logo(PesVersion::Pes21, 714), logo(PesVersion::Pes20, 714));
    }
}
