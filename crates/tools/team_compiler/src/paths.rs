//! Where compiled content goes inside the CPK, for the Fox versions (PES 18-21) and, for the
//! portraits and the logo, every version: `team_compiler/pipeline.md` "Game paths reference".
//! The CPK paths have no leading `/`.

use pes_version::PesVersion;

use crate::plan::subset::ModelPackage;

/// The bin holding every team's kit configs, keyed by entry name.
pub(crate) const UNIFORM_PARAMETER: &str =
    "common/character0/model/character/uniform/team/UniformParameter.bin";

/// The bin holding every team's colors, one record per team; the same path on every version.
pub(crate) const TEAM_COLOR: &str = "common/etc/TeamColor.bin";

/// The bin holding every team's kit menu colors and icons, one record per team; the same path
/// on every version.
pub(crate) const UNI_COLOR: &str = "common/character0/model/character/uniform/team/UniColor.bin";

/// The game folder of one `package` by `id`, without the `Asset/` or `/Assets/pes16/` head
/// the CPK paths and the FMDL texture paths put before it: the player id for the face, the
/// four-digit boots or gloves id for the other two (`k0625`, `g0625`).
fn model_folder(package: ModelPackage, id: u32) -> String {
    match package {
        ModelPackage::Face => format!("model/character/face/real/{id}"),
        ModelPackage::Boots => format!("model/character/boots/k{id:04}"),
        ModelPackage::Gloves => format!("model/character/glove/g{id:04}"),
    }
}

/// The folder of one `package` (its `.fpk` and `.fpkd`), by `id`.
pub(crate) fn package_folder(package: ModelPackage, id: u32) -> String {
    format!("Asset/{}/#Win", model_folder(package, id))
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
    /// A shared boots or gloves folder's: the output's own folder, beside its package, so
    /// the one output every linking player loads carries its own textures. Only the boots
    /// and the gloves are built this way; a shared face has no output of its own.
    SharedOutput {
        /// The output's package, boots or gloves.
        package: ModelPackage,
        /// Its shared id from the team's block.
        id: u32,
    },
}

impl TextureHome {
    /// The directory an FMDL texture-path table names for a texture here, for team `team_id`.
    pub(crate) fn directory(&self, team_id: u16) -> String {
        match self {
            TextureHome::PlayerCommon { folder_name } => format!(
                "/Assets/pes16/model/character/common/{team_id}/{folder_name}/sourceimages/"
            ),
            TextureHome::SharedOutput { package, id } => {
                format!("/Assets/pes16/{}/", model_folder(*package, *id))
            }
        }
    }

    /// The CPK path of the texture `stem` here, converted to FTEX, for team `team_id`.
    pub(crate) fn texture(&self, team_id: u16, stem: &str) -> String {
        match self {
            TextureHome::PlayerCommon { folder_name } => format!(
                "Asset/model/character/common/{team_id}/{folder_name}/sourceimages/#windx11/{stem}.ftex"
            ),
            TextureHome::SharedOutput { package, id } => {
                format!("Asset/{}/#windx11/{stem}.ftex", model_folder(*package, *id))
            }
        }
    }
}

/// The directory an FMDL texture-path table names for a texture of the team's Common output
/// (the Common row of "Game paths reference"), for team `team_id`: where a texture that
/// resolves into the export's `Common/` folder stays, packed once for every player pointing at
/// it (`pipeline.md` "3. Per-model-folder parallel steps", step 6).
pub(crate) fn common_texture_directory(team_id: u16) -> String {
    format!("/Assets/pes16/model/character/common/{team_id}/sourceimages/")
}

/// The CPK path of the texture `stem` of the team's Common output, converted to FTEX, for team
/// `team_id`.
pub(crate) fn common_texture(team_id: u16, stem: &str) -> String {
    format!("Asset/model/character/common/{team_id}/sourceimages/#windx11/{stem}.ftex")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_face_goes_by_player_id_and_boots_and_gloves_by_four_digit_model_id() {
        assert_eq!(
            package_folder(ModelPackage::Face, 71405),
            "Asset/model/character/face/real/71405/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Boots, 625),
            "Asset/model/character/boots/k0625/#Win"
        );
        assert_eq!(
            package_folder(ModelPackage::Gloves, 3745),
            "Asset/model/character/glove/g3745/#Win"
        );
    }

    #[test]
    fn a_player_s_textures_go_to_its_common_subfolder_and_a_shared_output_s_to_its_own() {
        let player = TextureHome::PlayerCommon {
            folder_name: "05 - A".to_owned(),
        };
        assert_eq!(
            player.directory(714),
            "/Assets/pes16/model/character/common/714/05 - A/sourceimages/"
        );
        assert_eq!(
            player.texture(714, "shirt"),
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex"
        );
        let boots = TextureHome::SharedOutput {
            package: ModelPackage::Boots,
            id: 644,
        };
        assert_eq!(
            boots.directory(714),
            "/Assets/pes16/model/character/boots/k0644/"
        );
        assert_eq!(
            boots.texture(714, "shirt"),
            "Asset/model/character/boots/k0644/#windx11/shirt.ftex"
        );
        let gloves = TextureHome::SharedOutput {
            package: ModelPackage::Gloves,
            id: 644,
        };
        assert_eq!(
            gloves.directory(714),
            "/Assets/pes16/model/character/glove/g0644/"
        );
        assert_eq!(
            gloves.texture(714, "grip"),
            "Asset/model/character/glove/g0644/#windx11/grip.ftex"
        );
    }

    #[test]
    fn the_team_s_common_textures_sit_one_level_above_the_players_subfolders() {
        assert_eq!(
            common_texture_directory(714),
            "/Assets/pes16/model/character/common/714/sourceimages/"
        );
        assert_eq!(
            common_texture(714, "cloth"),
            "Asset/model/character/common/714/sourceimages/#windx11/cloth.ftex"
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
