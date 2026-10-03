//! One kit (`team_compiler/pipeline.md` "4. Per-export non-model steps", Kits): its textures as
//! FTEX under the kit's game names, and its config encoded with those names.

use aesthetics_export::KitFolder;
use anyhow::Context;
use kit_config::{KitConfig, KitSlot, TexturePresence, texture_names};

use super::{CompileContext, Entry, TaskFailure, TaskFiles, take, texture};
use crate::paths;
use crate::plan::subset::{KIT_TEXTURE_STEMS, texture_format};
use crate::templates::PLACEHOLDER_KIT;

/// The kit `kit` in `slot` of team `team_id`, compiled for the run's version from its files'
/// bytes in `files`: its textures and its config as CPK entries, and the config as a
/// `UniformParameter.bin` entry (name, bytes). A kit without a `kit` texture gets the bundled
/// placeholder as its main texture, converted like any kit texture. The deep pass has already
/// dropped a kit whose `config.toml` does not parse (`kit_config_invalid`) or whose textures
/// its checks find wrong, so a config that fails to parse here is an ordinary failure; a
/// finding conversion reports (`texture_codec_unsupported`) fails the kit with the finding's
/// code: the config names its textures, so none goes out alone.
pub(super) fn kit(
    slot: KitSlot,
    kit: &KitFolder,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
) -> Result<(Vec<Entry>, Entry), TaskFailure> {
    let has = |stem: &str| kit.textures.iter().any(|texture| texture.stem == stem);
    let names = texture_names(
        team_id,
        slot,
        TexturePresence {
            // The game requires a main texture, so every compiled kit has one: the kit's own
            // or the placeholder. The other four are optional and absent ones stay zero.
            kit: true,
            back: has("kit_back"),
            chest: has("kit_chest"),
            leg: has("kit_leg"),
            name: has("kit_name"),
        },
    );

    let mut entries = Vec::new();
    for (stem, field) in KIT_TEXTURE_STEMS.iter().zip(&names) {
        let texture = kit.textures.iter().find(|texture| texture.stem == *stem);
        let (file_name, bytes) = match texture {
            Some(texture) => (texture.file.path.name(), take(files, &texture.file)),
            None if *stem == "kit" => ("placeholder_kit.dds", PLACEHOLDER_KIT.to_vec()),
            None => continue,
        };
        let name = std::str::from_utf8(field)
            .context("a kit texture name is ASCII")?
            .trim_end_matches('\0');
        let format = texture_format(file_name)
            .expect("a kit texture is classified by an extension `dds_convert` accepts");
        entries.push((
            paths::kit_texture(name),
            texture::convert(ctx, format, file_name, &bytes)?,
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
    let config = config.encode_with_names(ctx.version, &names).to_vec();
    let entry_name = slot.config_name(team_id);
    entries.push((paths::kit_config(team_id, &entry_name), config.clone()));
    Ok((entries, (entry_name, config)))
}
