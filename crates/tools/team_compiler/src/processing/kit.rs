//! One kit (`team_compiler/pipeline.md` "4. Per-export non-model steps", Kits): its textures as
//! FTEX under the kit's game names, its config encoded with those names, and its menu colors
//! and icon as its `UniColor.bin` entry ("Bins accumulation", "Kit colors fallback").

use aesthetics_export::{KitFolder, KitLayout, read_colors_txt};
use anyhow::Context;
use dds_convert::{SourceFormat, Target, TextureRole, decode};
use kit_config::{KitConfig, KitSlot, TexturePresence, apply_fpc, matches_fpc, texture_names};
use pes_version::Engine;
use studio_core::Disposition;

use super::texture::TextureError;
use super::{CompileContext, Entry, Finding, TaskFailure, TaskFiles, kit_layout, take, texture};
use crate::bins::{KIT_COLORS, KitColorEntry, Rgb, kit_number};
use crate::messages::Code;
use crate::paths::{self, REFEREE_MARKER_COLLAR};
use crate::plan::subset::{KIT_TEXTURE_STEMS, texture_format};
use crate::plan::{EffectiveTeamKitFpc, TeamKitEdits};

/// The menu icon of a kit without an `icon_<N>` marker.
const DEFAULT_ICON: u8 = 3;

/// The menu colors of a kit with none to give: magenta, then black. A pair nobody would pick,
/// so the gap shows in the game's menus instead of looking chosen; it matches the placeholder
/// texture's checkerboard.
const MISSING_COLORS: [Rgb; KIT_COLORS] = [[255, 0, 255], [0, 0, 0]];

/// The kit `kit` in `slot` of team `team_id`, compiled for the run's version from its files'
/// bytes in `files`: its textures and its config as CPK entries, the config as a
/// `UniformParameter.bin` entry (name, bytes), and its `UniColor.bin` entry. A kit without a
/// `kit` texture gets the bundled placeholder as its main texture, converted like any kit
/// texture. The deep pass has already dropped a kit whose `config.toml` does not parse
/// (`kit_config_invalid`) or whose textures its checks find wrong, so a config that fails to
/// parse here is an ordinary failure; a finding conversion reports
/// (`texture_codec_unsupported`) fails the kit with the finding's code: the config names its
/// textures, so none goes out alone.
///
/// When the kit's layout marker names the other engine than the run's, its main texture, its
/// own or inherited (never the placeholder), is re-laid out to the run's layout before it is
/// converted, noted in `findings` as `kit_layout_converted` naming both layouts; its other
/// textures are converted as they are.
///
/// `edits` is what the team's export sets in every kit config of the team. When its kit-FPC
/// status is `On`, a config lacking the FPC values gets them before it is encoded, noted in
/// `findings` as `kit_config_fpc_adjusted` (the template already carries them); then the
/// team's collar, when it has one, becomes the config's collar and winter collar, the FPC
/// collar included; otherwise the config is encoded as it is. A config whose collar or winter
/// collar is then the referees' marker collar fails the kit with `kit_collar_reserved`,
/// naming the field.
///
/// The entry's colors are the two its `colors.txt` gives (the lines it refuses are the deep
/// pass's to report); else the two its main texture gives, its own or one inherited from
/// `all/`, noted in `findings` as `kit_colors_derived`; else, for a placeholder kit or a main
/// texture that gives none, magenta and black, noted as `kit_colors_missing`.
pub(super) fn kit(
    slot: KitSlot,
    kit: &KitFolder,
    edits: TeamKitEdits,
    team_id: u16,
    ctx: &CompileContext,
    files: &mut TaskFiles,
    findings: &mut Vec<Finding>,
) -> Result<(Vec<Entry>, Entry, KitColorEntry), TaskFailure> {
    let listed = kit
        .colors
        .as_ref()
        .and_then(|file| listed_colors(&take(files, file)));
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

    let drawn_for_other = kit
        .layout
        .filter(|layout| layout_engine(*layout) != ctx.version.engine());
    let mut derived = None;
    let mut entries = Vec::new();
    for (stem, field) in KIT_TEXTURE_STEMS.iter().zip(&names) {
        let texture = kit.textures.iter().find(|texture| texture.stem == *stem);
        let (file_name, bytes) = match texture {
            Some(texture) => (texture.file.path.name(), take(files, &texture.file)),
            None if *stem == "kit" => (
                "placeholder_kit.dds",
                ctx.templates.placeholder_kit().to_vec(),
            ),
            None => continue,
        };
        let name = std::str::from_utf8(field)
            .context("a kit texture name is ASCII")?
            .trim_end_matches('\0');
        let format = texture_format(file_name)
            .expect("a kit texture is classified by an extension `dds_convert` accepts");
        // Only the kit's own main texture gives colors, never the placeholder: its
        // checkerboard is no color anybody chose.
        if listed.is_none() && *stem == "kit" && texture.is_some() {
            derived = derived_colors(format, file_name, &bytes);
        }
        // The placeholder is engine-neutral, and only the main texture is mapped through the
        // sock islands: the number and name textures are glyph atlases.
        let converted = match drawn_for_other {
            Some(drawn_for) if *stem == "kit" && texture.is_some() => {
                let to = engine_layout(ctx.version.engine());
                findings.push((
                    Code::KitLayoutConverted,
                    Disposition::Keep,
                    vec![
                        ("from", marker_name(drawn_for).to_owned()),
                        ("to", marker_name(to).to_owned()),
                    ],
                ));
                relaid_main_texture(ctx, format, file_name, &bytes, drawn_for)?
            }
            Some(_) | None => texture::convert(ctx, format, file_name, &bytes)?,
        };
        entries.push((paths::kit_texture(name), converted));
    }

    let mut config = match &kit.config {
        Some(file) => {
            let text = String::from_utf8(take(files, file))
                .with_context(|| format!("{}: not UTF-8", file.path.as_str()))?;
            KitConfig::from_toml(&text).with_context(|| file.path.as_str().to_owned())?
        }
        None => KitConfig::template(),
    };
    // Only upward: with the status unknown, FPC values a config carries are kept, since no
    // export state shows the team stopped using FPC.
    match edits.fpc {
        EffectiveTeamKitFpc::On if !matches_fpc(&config) => {
            apply_fpc(&mut config);
            findings.push((Code::KitConfigFpcAdjusted, Disposition::Keep, Vec::new()));
        }
        EffectiveTeamKitFpc::On | EffectiveTeamKitFpc::Unknown => {}
    }
    // After the FPC values, so the team's collar wins over the FPC collar (`pipeline.md`
    // "Collar contract").
    if let Some(collar) = edits.collar {
        config.shirt.collar = collar;
        config.shirt.winter_collar = collar;
    }
    // On the collars the kit ends up with: FPC sets collar 105 and the team's collar its own
    // (never 77, which the deep pass refuses), so a config naming 77 under either no longer
    // wears the marker.
    if let Some(field) = reserved_collar_field(&config) {
        return Err(TaskFailure {
            code: Code::KitCollarReserved,
            context: vec![("field", field.to_owned())],
        });
    }
    let config = config.encode_with_names(ctx.version, &names).to_vec();
    let entry_name = slot.config_name(team_id);
    entries.push((paths::kit_config(team_id, &entry_name), config.clone()));

    let colors = match (listed, derived) {
        (Some(colors), _) => colors,
        (None, Some(colors)) => {
            findings.push((Code::KitColorsDerived, Disposition::Keep, Vec::new()));
            colors
        }
        (None, None) => {
            findings.push((Code::KitColorsMissing, Disposition::Keep, Vec::new()));
            MISSING_COLORS
        }
    };
    let entry = KitColorEntry {
        kit: kit_number(slot),
        icon: kit.icon.unwrap_or(DEFAULT_ICON),
        colors,
    };
    Ok((entries, (entry_name, config), entry))
}

/// The field of `config` naming the referees' marker collar, `collar` before `winter_collar`;
/// `None` when neither does. Every player wearing that collar would wear the marker.
fn reserved_collar_field(config: &KitConfig) -> Option<&'static str> {
    if config.shirt.collar == REFEREE_MARKER_COLLAR {
        Some("collar")
    } else if config.shirt.winter_collar == REFEREE_MARKER_COLLAR {
        Some("winter_collar")
    } else {
        None
    }
}

/// The engine whose kit UV layout `layout` names.
fn layout_engine(layout: KitLayout) -> Engine {
    match layout {
        KitLayout::PreFox => Engine::PreFox,
        KitLayout::Fox => Engine::Fox,
    }
}

/// The layout marker's name for `layout`, as `kit_layout_converted` reports it.
fn marker_name(layout: KitLayout) -> &'static str {
    match layout {
        KitLayout::PreFox => "pre-fox",
        KitLayout::Fox => "fox",
    }
}

/// The kit layout `engine`'s uniform models map.
fn engine_layout(engine: Engine) -> KitLayout {
    match engine {
        Engine::PreFox => KitLayout::PreFox,
        Engine::Fox => KitLayout::Fox,
    }
}

/// The kit's main texture `file_name`, in `format`, holding `bytes`, drawn for the `drawn_for`
/// layout: decoded, re-laid out for the other one (`kit_layout::relaid`) and converted for the
/// run's version. Not through the run's converter, whose cache holds a source file's own
/// conversion, which this is not. A failure is the file's, as `texture::convert`'s are.
fn relaid_main_texture(
    ctx: &CompileContext,
    format: SourceFormat,
    file_name: &str,
    bytes: &[u8],
    drawn_for: KitLayout,
) -> Result<Vec<u8>, TextureError> {
    let failure = |error| texture::conversion_failure(file_name, error);
    let decoded = decode(bytes, format).map_err(failure)?;
    let relaid = kit_layout::relaid(&decoded, drawn_for).map_err(failure)?;
    let target = Target {
        version: ctx.version,
        role: TextureRole::Color,
    };
    dds_convert::convert(&relaid, target).map_err(failure)
}

/// The two colors a kit's `colors.txt` holding `bytes` gives, or `None` when it gives fewer.
fn listed_colors(bytes: &[u8]) -> Option<[Rgb; KIT_COLORS]> {
    match read_colors_txt(bytes, KIT_COLORS).colors.as_slice() {
        [first, second] => Some([*first, *second]),
        _ => None,
    }
}

/// The two menu colors the main texture `file_name`, in `format`, holding `bytes`, gives: its
/// top level's dominant colors (`libs/color_tools.md` "Dominant kit-color extraction"). `None`
/// when it does not decode or its shirt region has no opaque pixel. The texture is decoded
/// here and again by its conversion: `dds_convert` has no decode that both could share
/// through the conversion cache, and only a kit without two listed colors pays the second.
fn derived_colors(
    format: SourceFormat,
    file_name: &str,
    bytes: &[u8],
) -> Option<[Rgb; KIT_COLORS]> {
    let decoded = match decode(bytes, format) {
        Ok(decoded) => decoded,
        // Its conversion fails on the same error and reports it, failing the kit.
        Err(error) => {
            log::debug!("{file_name}: no kit colors from a texture that does not decode: {error}");
            return None;
        }
    };
    let top = decoded.mips.first()?;
    color_tools::kit::extract_kit_colors(top, decoded.width, decoded.height)
        .map(|colors| [colors.color1, colors.color2])
}
