//! Where compiled content goes inside the CPK, for the Fox versions (PES 18-21) and, for the
//! portraits, every version: `team_compiler/pipeline.md` "Game paths reference". The CPK paths
//! have no leading `/`.

use pes_version::PesVersion;

use crate::plan::subset::ModelPackage;

/// The bin holding every team's kit configs, keyed by entry name.
pub(crate) const UNIFORM_PARAMETER: &str =
    "common/character0/model/character/uniform/team/UniformParameter.bin";

/// The folder of one player's `package` (its `.fpk` and `.fpkd`), by `id`: the player id for
/// the face, the four-digit boots or gloves id for the other two (`k0625`, `g0625`).
pub(crate) fn package_folder(package: ModelPackage, id: u32) -> String {
    match package {
        ModelPackage::Face => format!("Asset/model/character/face/real/{id}/#Win"),
        ModelPackage::Boots => format!("Asset/model/character/boots/k{id:04}/#Win"),
        ModelPackage::Gloves => format!("Asset/model/character/glove/g{id:04}/#Win"),
    }
}

/// A player folder's own textures, as the FMDL texture-path table names their folder: the
/// per-player common subfolder of the team, keyed by the source folder's name.
pub(crate) fn player_common_directory(team_id: u16, folder_name: &str) -> String {
    format!("/Assets/pes16/model/character/common/{team_id}/{folder_name}/sourceimages/")
}

/// The CPK path of one texture in that per-player common subfolder.
pub(crate) fn player_common_texture(team_id: u16, folder_name: &str, stem: &str) -> String {
    format!(
        "Asset/model/character/common/{team_id}/{folder_name}/sourceimages/#windx11/{stem}.ftex"
    )
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
}
