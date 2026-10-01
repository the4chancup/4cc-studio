//! Where compiled content goes inside the CPK, for the Fox versions (PES 18-21):
//! `team_compiler/pipeline.md` "Game paths reference". The CPK paths have no leading `/`.

/// The bin holding every team's kit configs, keyed by entry name.
pub(crate) const UNIFORM_PARAMETER: &str =
    "common/character0/model/character/uniform/team/UniformParameter.bin";

/// The folder of one player's face package (`face.fpk`, `face.fpkd`), by player id.
pub(crate) fn face_folder(player_id: u32) -> String {
    format!("Asset/model/character/face/real/{player_id}/#Win")
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
