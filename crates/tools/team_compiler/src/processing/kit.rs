//! One kit (`team_compiler/pipeline.md` "4. Per-export non-model steps", Kits): its textures as
//! FTEX under the kit's game names, and its config encoded with those names.

use aesthetics_export::{KitFolder, KitLayout};
use anyhow::{Context, bail};
use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};
use pes_version::{Engine, PesVersion};

use super::{Entry, TaskFiles, take, texture};
use crate::paths;

/// The kit texture stems, in the order of the config's texture-name fields.
const KIT_TEXTURE_STEMS: [&str; 5] = ["kit", "kit_back", "kit_chest", "kit_leg", "kit_name"];

/// The kit `kit` in `slot` of team `team_id`, compiled for `version` from its files' bytes in
/// `files`: its textures and its config as CPK entries, and the config as a
/// `UniformParameter.bin` entry (name, bytes).
pub(super) fn kit(
    slot: KitSlot,
    kit: &KitFolder,
    team_id: u16,
    version: PesVersion,
    files: &mut TaskFiles,
) -> anyhow::Result<(Vec<Entry>, Entry)> {
    let target_layout = match version.engine() {
        Engine::Fox => KitLayout::Fox,
        Engine::PreFox => KitLayout::PreFox,
    };
    if kit.layout.is_some_and(|layout| layout != target_layout) {
        bail!("a kit drawn for the other engine's layout cannot be compiled yet");
    }
    if let Some(texture) = kit
        .textures
        .iter()
        .find(|texture| !KIT_TEXTURE_STEMS.contains(&texture.stem.as_str()))
    {
        bail!("{}: cannot be compiled yet", texture.file.path.as_str());
    }

    let has = |stem: &str| kit.textures.iter().any(|texture| texture.stem == stem);
    let names = texture_names(
        team_id,
        slot,
        TexturePresence {
            kit: has("kit"),
            back: has("kit_back"),
            chest: has("kit_chest"),
            leg: has("kit_leg"),
            name: has("kit_name"),
        },
    );

    let mut entries = Vec::new();
    for (stem, field) in KIT_TEXTURE_STEMS.iter().zip(&names) {
        let Some(texture) = kit.textures.iter().find(|texture| texture.stem == *stem) else {
            continue;
        };
        let name = std::str::from_utf8(field)
            .context("a kit texture name is ASCII")?
            .trim_end_matches('\0');
        let bytes = take(files, &texture.file);
        entries.push((
            paths::kit_texture(name),
            texture::to_ftex(texture.file.path.name(), bytes)?,
        ));
    }

    let config = match &kit.config {
        Some(file) => {
            let text = String::from_utf8(take(files, file))
                .with_context(|| format!("{}: not UTF-8", file.path.as_str()))?;
            KitConfig::from_toml(&text).with_context(|| file.path.as_str().to_owned())?
        }
        None => KitConfig::template(),
    };
    let config = config.encode_with_names(version, &names).to_vec();
    let entry_name = slot.config_name(team_id);
    entries.push((paths::kit_config(team_id, &entry_name), config.clone()));
    Ok((entries, (entry_name, config)))
}
