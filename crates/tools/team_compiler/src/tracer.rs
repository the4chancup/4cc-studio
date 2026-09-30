//! The tracer bullet: the smallest real export compiled end to end. This is
//! scaffolding for the parity test (`tests/parity.rs`) — a hardcoded walk of
//! `Players/` and `Kits/` covering only the Phase 3 compile subset, replaced by
//! the `aesthetics_export` object model and the pipeline in later steps.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail, ensure};
use fmdl::FmdlFile;
use fmdl::ops::paths::rewrite_texture_paths;
use fpk::{FpkFile, FpkKind};
use kit_config::{KitConfig, TexturePresence, texture_names};
use pes_version::{Engine, PesVersion};
use teams_list::TeamId;
use uniparam::UniformParameter;

const UNIPARAM_18: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/bins/UniformParameter18.bin"
));
const UNIPARAM_19: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/bins/UniformParameter19.bin"
));

/// The face model names the Fox engine loads.
const FACE_STEMS: [&str; 4] = ["face_high", "hair_high", "oral", "fcl_hair"];
/// The five kit texture stems, in `texture_names` field order.
const KIT_TEXTURE_STEMS: [&str; 5] = ["kit", "kit_back", "kit_chest", "kit_leg", "kit_name"];
/// The kit marker files: they declare the layout the textures were drawn for.
const LAYOUT_MARKERS: [&str; 4] = ["pre-fox", "fox", "pre-fox.txt", "fox.txt"];

/// A directory's entries by file name: `read_dir`'s order is unspecified and
/// the CPK's entry order would inherit the file system's.
fn read_dir_sorted(path: &Path) -> anyhow::Result<Vec<fs::DirEntry>> {
    let mut entries = fs::read_dir(path)
        .with_context(|| format!("cannot list {}", path.display()))?
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(entries)
}

/// Compiles the Phase 3 subset of one export into a CPK: the `Players/`
/// folders' Fox face content and the `Kits/` folders, written to `cpk_path`.
/// Anything the walk does not recognize — a non-Fox target, a root entry that
/// is not `Players`/`Kits`, a file that is not face content or kit content —
/// is an error naming it; nothing is skipped silently.
pub fn compile_tracer(
    export_root: &Path,
    team: TeamId,
    version: PesVersion,
    cpk_path: &Path,
) -> anyhow::Result<()> {
    ensure!(
        version.engine() == Engine::Fox,
        "tracer compiles Fox targets (PES 18-21) only, not {version}"
    );

    let root = read_dir_sorted(export_root)
        .with_context(|| format!("cannot list export root {}", export_root.display()))?;
    let mut players_dir = None;
    let mut kits_dir = None;
    for entry in root {
        let name = entry.file_name();
        match name.to_str() {
            Some("Players") if entry.path().is_dir() => players_dir = Some(entry.path()),
            Some("Kits") if entry.path().is_dir() => kits_dir = Some(entry.path()),
            _ => bail!("unrecognized root entry {}", name.to_string_lossy()),
        }
    }

    let file = fs::File::create(cpk_path)
        .with_context(|| format!("cannot create CPK {}", cpk_path.display()))?;
    let mut writer = cpk::CpkWriter::new(file, "4cc-studio")?;
    let mut kit_entries = Vec::new();

    if let Some(players) = players_dir {
        for folder in read_dir_sorted(&players)? {
            compile_player(&folder.path(), team, &mut writer)?;
        }
    }
    if let Some(kits) = kits_dir {
        for folder in read_dir_sorted(&kits)? {
            kit_entries.push(compile_kit(&folder.path(), team, version, &mut writer)?);
        }
    }
    if !kit_entries.is_empty() {
        let base = match version {
            PesVersion::Pes18 => UNIPARAM_18,
            PesVersion::Pes19 | PesVersion::Pes20 | PesVersion::Pes21 => UNIPARAM_19,
            PesVersion::Pes15 | PesVersion::Pes16 | PesVersion::Pes17 => {
                bail!("tracer compiles Fox targets (PES 18-21) only, not {version}");
            }
        };
        let mut uniparam = UniformParameter::read(base)
            .context("cannot read the bundled UniformParameter base")?;
        for (name, config) in kit_entries {
            uniparam.insert(name, config.to_vec())?;
        }
        writer.add(
            "common/character0/model/character/uniform/team/UniformParameter.bin",
            &uniparam.write(),
            None,
        )?;
    }
    writer.finish()?;
    Ok(())
}

/// The `NN - name` part of a player folder name gives the slot; the player ID
/// is `team * 100 + NN`.
fn player_folder(path: &Path) -> anyhow::Result<(u16, String)> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("player path has no name: {}", path.display()))?;
    let (number, _) = name
        .split_once(" - ")
        .with_context(|| format!("player folder {name:?} is not `NN - name`"))?;
    let slot: u16 = number
        .parse()
        .with_context(|| format!("player folder {name:?} starts with no number"))?;
    ensure!(
        (1..=23).contains(&slot),
        "player folder {name:?} claims slot {slot}, outside 01-23"
    );
    Ok((slot, name.to_owned()))
}

/// One player folder: its Fox face models packed into `face.fpk` under the
/// player's ID, its textures relocated to the per-player common subfolder.
fn compile_player(
    folder: &Path,
    team: TeamId,
    writer: &mut cpk::CpkWriter<fs::File>,
) -> anyhow::Result<()> {
    let (slot, folder_name) = player_folder(folder)?;
    let player_id = u32::from(team.get()) * 100 + u32::from(slot);
    let common_dir = format!(
        "/Assets/pes16/model/character/common/{}/{folder_name}/sourceimages/",
        team.get()
    );

    let mut models = Vec::new();
    let mut skeleton = None;
    let mut packed_files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut textures: BTreeMap<String, PathBuf> = BTreeMap::new();
    for entry in read_dir_sorted(folder)? {
        ensure!(
            entry.path().is_file(),
            "player folder {folder_name:?} holds an unexpected folder {}",
            entry.file_name().to_string_lossy()
        );
        let name = entry.file_name().to_string_lossy().into_owned();
        let path = entry.path();
        let stem = Path::new(&name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        let extension = Path::new(&name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("");
        match extension {
            "fmdl" => {
                ensure!(
                    FACE_STEMS.contains(&stem),
                    "player folder {folder_name:?} holds {name:?}, not an allowed face name"
                );
                models.push((stem.to_owned(), FmdlFile::read(&fs::read(&path)?)?));
            }
            "skl" => {
                ensure!(
                    stem == "fcl_hair",
                    "player folder {folder_name:?} holds {name:?}: only fcl_hair has a skeleton slot"
                );
                skeleton = Some(fs::read(&path)?);
            }
            "bin" if name == "face_diff.bin" => {
                packed_files.insert(name, fs::read(&path)?);
            }
            "fclo" if name == "fcl_hair_sim.fclo" => {
                packed_files.insert(name, fs::read(&path)?);
            }
            "dds" | "ftex" => {
                ensure!(
                    textures.insert(stem.to_owned(), path).is_none(),
                    "player folder {folder_name:?} holds two textures of stem {stem:?}"
                );
            }
            _ => bail!("player folder {folder_name:?} holds unrecognized file {name:?}"),
        }
    }

    // Every image lands in the per-player common subfolder; the models'
    // references to resolvable stems are pointed there, and unresolvable
    // references (the game's own dummy textures) keep their directory except
    // the zero-team-ID segment, which becomes the team's.
    for (stem, path) in &textures {
        let bytes = fs::read(path)?;
        let ftex = match path.extension().and_then(|extension| extension.to_str()) {
            Some("ftex") => bytes,
            _ => ftex::dds_to_ftex(&bytes, ftex::ColorSpace::Normal)
                .with_context(|| format!("cannot convert {}", path.display()))?,
        };
        writer.add(
            &format!(
                "Asset/model/character/common/{}/{folder_name}/sourceimages/#windx11/{stem}.ftex",
                team.get()
            ),
            &ftex,
            None,
        )?;
    }
    let mut fpk = FpkFile::new(FpkKind::Fpk);
    for (stem, mut file) in models {
        rewrite_texture_paths(&mut file, |path| {
            let stem = Path::new(&path.file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or("");
            if textures.contains_key(stem) {
                path.directory = common_dir.clone();
            } else {
                path.directory = path
                    .directory
                    .replace("/000/", &format!("/{}/", team.get()));
            }
        })?;
        fpk.insert(format!("{stem}.fmdl"), file.write());
    }
    if let Some(skeleton) = skeleton {
        ensure!(
            fpk.entries().any(|(name, _)| name == "fcl_hair.fmdl"),
            "player folder {folder_name:?} carries fcl_hair.skl with no fcl_hair.fmdl"
        );
        fpk.insert("fcl_hair_sim.skl".to_owned(), skeleton);
    }
    for (name, bytes) in packed_files {
        fpk.insert(name, bytes);
    }

    let base = format!("Asset/model/character/face/real/{player_id}/#Win");
    writer.add(&format!("{base}/face.fpk"), &fpk.write(), None)?;
    writer.add(
        &format!("{base}/face.fpkd"),
        &FpkFile::new(FpkKind::Fpkd).write(),
        None,
    )?;
    Ok(())
}

/// One kit folder: its textures converted to FTEX under the `u0{team}{slot}`
/// name and its config encoded, with the UniformParameter entry returned for
/// the caller's bin.
fn compile_kit(
    folder: &Path,
    team: TeamId,
    version: PesVersion,
    writer: &mut cpk::CpkWriter<fs::File>,
) -> anyhow::Result<(String, [u8; 120])> {
    let slot = folder
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("kit path has no name: {}", folder.display()))?;
    let suffix = kit_name_suffix(slot)
        .with_context(|| format!("kit folder {slot:?} is not a plain slot (p1-p9 or g1)"))?;

    let mut config = None;
    let mut presence = TexturePresence::default();
    let mut images: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for entry in read_dir_sorted(folder)? {
        ensure!(
            entry.path().is_file(),
            "kit folder {slot:?} holds an unexpected folder {}",
            entry.file_name().to_string_lossy()
        );
        let name = entry.file_name().to_string_lossy().into_owned();
        let stem = Path::new(&name)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        match name.as_str() {
            "config.toml" => {
                config = Some(
                    KitConfig::from_toml(&fs::read_to_string(entry.path())?)
                        .with_context(|| format!("kit {slot:?}: cannot parse config.toml"))?,
                );
            }
            "icon.txt" | "colors.txt" => {
                // Read by nothing in the tracer subset; the fields they feed
                // are Phase 4's bins.
            }
            _ if LAYOUT_MARKERS.contains(&name.as_str()) => {
                bail!("kit folder {slot:?} carries a layout marker, which the tracer cannot honor");
            }
            _ => match KIT_TEXTURE_STEMS.iter().position(|known| *known == stem) {
                Some(index) if name.ends_with(".dds") => {
                    let present = [
                        &mut presence.kit,
                        &mut presence.back,
                        &mut presence.chest,
                        &mut presence.leg,
                        &mut presence.name,
                    ];
                    *present[index] = true;
                    images.insert(stem.to_owned(), fs::read(entry.path())?);
                }
                _ => bail!("kit folder {slot:?} holds unrecognized file {name:?}"),
            },
        }
    }

    let names = texture_names(team.get(), slot, presence);
    for (index, field) in names.iter().enumerate() {
        if field.iter().all(|byte| *byte == 0) {
            continue;
        }
        let stem = KIT_TEXTURE_STEMS[index];
        let ftex = ftex::dds_to_ftex(&images[stem], ftex::ColorSpace::Normal)
            .with_context(|| format!("kit {slot:?}: cannot convert {stem}.dds"))?;
        let texture_name = std::str::from_utf8(field)
            .context("kit texture name is not ASCII")?
            .trim_end_matches('\0');
        writer.add(
            &format!("Asset/model/character/uniform/texture/#windx11/{texture_name}.ftex"),
            &ftex,
            None,
        )?;
    }

    let config = config.unwrap_or_else(KitConfig::template);
    let bytes = config.encode_with_names(version, &names);
    let entry_name = format!("{}_DEF_{suffix}_realUni.bin", team.get());
    writer.add(
        &format!(
            "common/character0/model/character/uniform/team/{}/{entry_name}",
            team.get()
        ),
        &bytes,
        None,
    )?;
    Ok((entry_name, bytes))
}

/// The name a kit's output files carry: `1st`..`9th` for `p1`..`p9`, `GK1st`
/// for `g1`.
fn kit_name_suffix(slot: &str) -> Option<&'static str> {
    if slot == "g1" {
        return Some("GK1st");
    }
    let index = slot.strip_prefix('p')?.parse::<u8>().ok()?;
    [
        "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th",
    ]
    .get(usize::from(index).checked_sub(1)?)
    .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kit_name_suffix_maps_slots_or_refuses_them() {
        assert_eq!(kit_name_suffix("p1"), Some("1st"));
        assert_eq!(kit_name_suffix("p9"), Some("9th"));
        assert_eq!(kit_name_suffix("g1"), Some("GK1st"));
        for slot in ["p0", "p10", "g2", "x1"] {
            assert_eq!(kit_name_suffix(slot), None);
        }
    }
}
