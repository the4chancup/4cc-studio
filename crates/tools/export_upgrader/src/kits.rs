//! Step 5's `Kit Configs/` and `Kit Textures/` folders into `Kits/`
//! (`spec/export_upgrader.md` step 5, `DECISIONS.md` 2026-10-18 "the kit
//! configs' and kit textures' rules of 6.3a", "the Note's kit entries'
//! grammar and the two kit Warnings of 6.3b" and "the kit-texture copy
//! covers every field naming another texture"): every config becomes its
//! kit folder's `config.toml` (its texture-name fields dropped — the
//! compiler re-derives them from the textures present, its undecoded bits
//! kept in the `[unknown]` table), every kit texture lands in its kit's
//! folder under the generic `kit` name, the Note's kit entries give each
//! kit its `colors.txt` and icon, a field naming another texture gives the
//! kit a copy of it, and a texture every kit shares moves to `Kits/all/`.
//! What never reached the game lands in `leftovers` and is reported
//! `leftover_dropped`, the run clean; what cannot be placed is left for the
//! run's fallback to report as unmigrated content.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::read_colors_txt;
use kit_config::{KitConfig, KitSlot};
use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, INPUT, UnmigratedReason, read_failed, tool_message, unmigrated};
use crate::note;
use crate::players::{file_name, path_scope};
use crate::source::SourceFailure;

/// What step 5 makes of an export's `Kit Configs/` and `Kit Textures/`: the
/// files written into `Kits/`, the input files they account for, and the
/// findings on the input.
#[derive(Debug, Default)]
pub(crate) struct KitFolders {
    /// Generated files: output path (`Kits/p1/config.toml`) and contents.
    pub(crate) written: Vec<(String, Vec<u8>)>,
    /// Input path → output path, copied as they are.
    pub(crate) copies: Vec<(String, String)>,
    /// Input files accounted for; the run's fallback reports every other one.
    pub(crate) consumed: BTreeSet<String>,
    /// Input files the step recognized as leftovers (step 10): the run's
    /// fallback reports each `leftover_dropped`, the run clean, rather than
    /// `unmigrated_content`. A file here is not in `consumed`.
    pub(crate) leftovers: BTreeSet<String>,
    /// Findings on the input, in input-path order, a file's own findings in
    /// the order met.
    pub(crate) findings: Vec<Message>,
}

/// The `<suffix>` a config name's `???_DEF_<suffix>_realUni.bin` shape
/// carries — any three characters, `_DEF_`, the suffix and `_realUni.bin`,
/// the fixed parts in any letter case.
fn config_suffix(file_name: &str) -> Option<&str> {
    let body = file_name
        .char_indices()
        .nth(3)
        .map(|(index, _)| &file_name[index..])?;
    if !body.get(..5)?.eq_ignore_ascii_case("_DEF_") {
        return None;
    }
    let tail = &body[5..];
    let suffix_len = tail.len().checked_sub(12)?;
    if !tail.get(suffix_len..)?.eq_ignore_ascii_case("_realUni.bin") {
        return None;
    }
    tail.get(..suffix_len)
}

/// The slot a kit config's file name carries: `config_suffix`'s shape, the
/// suffix a kit-config entry suffix (`1st` … `9th`, `GK1st`).
fn config_slot(file_name: &str) -> Option<KitSlot> {
    KitSlot::from_config_suffix(config_suffix(file_name)?)
}

/// Whether `file_name` is a config name for a goalkeeper past the first —
/// `???_DEF_GK2nd_realUni.bin` … `GK9th`, in any letter case: a second or
/// later goalkeeper kit has no Studio slot and is a leftover (step 10).
fn gk_config_beyond_first(file_name: &str) -> bool {
    let Some(suffix) = config_suffix(file_name) else {
        return false;
    };
    suffix
        .get(..2)
        .is_some_and(|head| head.eq_ignore_ascii_case("GK"))
        && suffix.get(2..).is_some_and(|ordinal| {
            ["2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th"]
                .iter()
                .any(|known| ordinal.eq_ignore_ascii_case(known))
        })
}

/// The slot a kit texture's file name carries, its output name's tail and
/// whether it is an FTEX to convert: `u`, any four characters, the slot's
/// two characters (`p1` … `p9`, `g1`), the rest of the stem (empty or
/// starting with `_`), then `.dds` or `.ftex` — the fixed parts in any
/// letter case. The rest is returned with the extension as written for a
/// `.dds`, rewritten `.dds` for an `.ftex` (`_back.dds`, `.dds`), so the
/// output name is `kit` + it.
fn texture_slot(file_name: &str) -> Option<(KitSlot, Cow<'_, str>, bool)> {
    let (stem, extension) = file_name.rsplit_once('.')?;
    let ftex = if extension.eq_ignore_ascii_case("dds") {
        false
    } else if extension.eq_ignore_ascii_case("ftex") {
        true
    } else {
        return None;
    };
    if !stem.get(..1)?.eq_ignore_ascii_case("u") {
        return None;
    }
    let slot_at = stem.char_indices().nth(5).map(|(index, _)| index)?;
    let slot = KitSlot::parse(stem.get(slot_at..slot_at + 2)?)?;
    let rest_of_stem = &stem[slot_at + 2..];
    if !rest_of_stem.is_empty() && !rest_of_stem.starts_with('_') {
        return None;
    }
    let tail = if ftex {
        Cow::Owned(format!("{rest_of_stem}.dds"))
    } else {
        Cow::Borrowed(&file_name[slot_at + 2..])
    };
    Some((slot, tail, ftex))
}

/// What a Note kit entry's colors give its kit: the two colors as the Note
/// spells them and the icon marker number.
struct KitEntry {
    /// The two colors, each as the Note spells it.
    colors: [String; 2],
    /// The `icon_<N>` number; `None` for none or the default 3.
    icon: Option<u8>,
}

/// A kit entry's colors (step 5): split at each `-`, the parts trimmed. Two
/// or three parts read; one, or four and more, do not. The first two are each
/// one color by the `colors.txt` grammar; either failing, the entry does not
/// read. A third part that parses as a whole number from 0 to 23 other than
/// 3 (the default) is the icon; any other third part gives none.
fn kit_entry(colors: &str) -> Option<KitEntry> {
    let parts: Vec<&str> = colors.split('-').map(str::trim).collect();
    let (first, second, third) = match parts.as_slice() {
        [first, second] => (*first, *second, None),
        [first, second, third] => (*first, *second, Some(*third)),
        _ => return None,
    };
    let one_color = |part: &str| {
        let read = read_colors_txt(part.as_bytes(), 1);
        (read.colors.len() == 1 && read.refused.is_empty()).then(|| part.to_owned())
    };
    let icon = third
        .and_then(|part| part.parse::<u8>().ok())
        .filter(|number| *number != 3 && *number <= 23);
    Some(KitEntry {
        colors: [one_color(first)?, one_color(second)?],
        icon,
    })
}

/// What a config's texture-name field names (step 5): its bytes up to the
/// first NUL, then by position `u`/`U`, any four characters, the slot
/// (`p0` in any case is the config's own), then the rest (`_back`,
/// `_front`, …). Anything else names nothing.
fn field_texture(field: &[u8; 16], own_slot: KitSlot) -> Option<(KitSlot, &str)> {
    let end = field.iter().position(|byte| *byte == 0).unwrap_or(16);
    let text = std::str::from_utf8(&field[..end]).ok()?;
    if !text.get(..1)?.eq_ignore_ascii_case("u") {
        return None;
    }
    let slot_at = text.char_indices().nth(5).map(|(index, _)| index)?;
    let slot_text = text.get(slot_at..slot_at + 2)?;
    let slot = if slot_text.eq_ignore_ascii_case("p0") {
        own_slot
    } else {
        KitSlot::parse(slot_text)?
    };
    Some((slot, &text[slot_at + 2..]))
}

/// The folded stem of an output path's file name (`kit_back.dds` →
/// `kit_back`, `kit_Back.DDS` the same): the name two files share or not.
fn folded_stem(output: &str) -> String {
    let name = output.rsplit('/').next().unwrap_or(output);
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    vtree::fold_name(stem)
}

/// The kit folder a decoded config became: its input file, its bytes (the
/// one-config compare) and its five texture-name fields, kept for the copy
/// pass after `config.toml` drops them.
struct Decided<'a> {
    /// The `Kit Configs` file it came from.
    path: &'a String,
    /// Its raw bytes.
    bytes: Vec<u8>,
    /// `source_texture_names` as decoded.
    fields: [[u8; 16]; 5],
}

/// What step 5 makes of `files`' `Kit Configs/` and `Kit Textures/` and the
/// Note's kit entries (`note` the Note's path and text, `None` without one;
/// `full` whether the output is a Full export, which gains an empty `p1`
/// for the `all/` hoist when no player kit is written and an empty `g1`
/// when no `g1` is): every config decoded and converted to its kit folder's
/// `config.toml`, every texture of a decided kit copied under the generic
/// name, each kit entry's colors and icon written beside it, then the copy
/// and the hoist.
pub(crate) fn kit_folders(
    files: &[String],
    version: PesVersion,
    note: Option<(&str, &str)>,
    full: bool,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> KitFolders {
    let mut folders = KitFolders::default();
    let mut failures: BTreeMap<String, Vec<Message>> = BTreeMap::new();

    // The `Kit Configs` candidates, a file directly in the folder or in one
    // subfolder of it whose name reads (a deeper file or another name is left
    // to the fallback), roots first then the subfolder level. `files` is
    // sorted, so each group holds its paths in order.
    let mut roots = Vec::new();
    let mut below = Vec::new();
    for file in files {
        let Some((top, rest)) = file.split_once('/') else {
            continue;
        };
        if !top.eq_ignore_ascii_case("kit configs") {
            continue;
        }
        let name = file_name(file);
        // Step 10's stray rule, wherever the file sits: what `classify`
        // reads as no known kind and is no `.txt` note never reached the
        // game.
        if crate::players::is_stray(name, None) {
            folders.leftovers.insert(file.clone());
            continue;
        }
        let Some(slot) = config_slot(name) else {
            // A config for a goalkeeper past the first is a leftover
            // (step 10); every other name `config_slot` refuses is
            // unmigrated content, reported here and consumed.
            if gk_config_beyond_first(name) {
                folders.leftovers.insert(file.clone());
            } else {
                failures.entry(file.clone()).or_default().push(unmigrated(
                    Scope::File {
                        export_id: INPUT,
                        path: path_scope(file),
                    },
                    Disposition::DropFile,
                    UnmigratedReason::ConfigNameInvalid,
                    None,
                    vec![],
                ));
                folders.consumed.insert(file.clone());
            }
            continue;
        };
        match rest.matches('/').count() {
            0 => roots.push((file, slot)),
            1 => below.push((file, slot)),
            // Deeper than the one subfolder the old layout flattened.
            _ => {
                failures.entry(file.clone()).or_default().push(unmigrated(
                    Scope::File {
                        export_id: INPUT,
                        path: path_scope(file),
                    },
                    Disposition::DropFile,
                    UnmigratedReason::ConfigTooDeep,
                    None,
                    vec![],
                ));
                folders.consumed.insert(file.clone());
            }
        }
    }

    // A slot's config is its first candidate that reads and decodes — decided
    // before any byte compares, so a config that fails leaves the slot to the
    // next one; a later file of the slot is one config when its bytes match
    // the kit's, and not consumed otherwise.
    let mut decided: BTreeMap<KitSlot, Decided> = BTreeMap::new();
    for (file, slot) in roots.iter().chain(&below) {
        match read(file) {
            Err(failure) => {
                failures
                    .entry(file.as_str().to_owned())
                    .or_default()
                    .push(read_failed(INPUT, failure, Disposition::DropFile));
                folders.consumed.insert((*file).clone());
            }
            Ok(bytes) => match decided.get(slot) {
                Some(kept) if kept.bytes == bytes => {
                    folders.consumed.insert((*file).clone());
                }
                // A second config of the slot, its bytes different:
                // unmigrated, `other` the config that became the kit's.
                Some(kept) => {
                    failures
                        .entry((*file).clone())
                        .or_default()
                        .push(unmigrated(
                            Scope::File {
                                export_id: INPUT,
                                path: path_scope(file),
                            },
                            Disposition::DropFile,
                            UnmigratedReason::ConfigTaken,
                            Some(kept.path.clone()),
                            vec![],
                        ));
                    folders.consumed.insert((*file).clone());
                }
                None => match KitConfig::decode(&bytes, version) {
                    Ok(mut config) => {
                        let fields = config
                            .source_texture_names
                            .expect("a decoded config carries its five texture-name fields");
                        config.source_texture_names = None;
                        folders.written.push((
                            format!("Kits/{}/config.toml", slot.as_str()),
                            config.to_toml().into_bytes(),
                        ));
                        decided.insert(
                            *slot,
                            Decided {
                                path: file,
                                bytes,
                                fields,
                            },
                        );
                        folders.consumed.insert((*file).clone());
                    }
                    Err(error) => {
                        failures
                            .entry(file.as_str().to_owned())
                            .or_default()
                            .push(unmigrated(
                                Scope::File {
                                    export_id: INPUT,
                                    path: path_scope(file),
                                },
                                Disposition::DropFile,
                                UnmigratedReason::ConfigUndecodable,
                                None,
                                vec![("error", error.to_string())],
                            ));
                        folders.consumed.insert((*file).clone());
                    }
                },
            },
        }
    }

    // The Note's kit entries (step 5): `gk:` in any letter case is the
    // goalkeeper's, the first giving g1; the others give p1 … p9 in Note
    // order. A second `gk:` entry has no slot and is a leftover (step 10:
    // no version the cup plays shows a second goalkeeper kit); a player
    // entry past the ninth has none either and stays unmigrated content,
    // and so is an entry whose colors do not read — its slot still counts
    // as given, so the later entries keep theirs: dropping it would shift
    // every kit after it down a slot, onto another kit's config.
    let mut entry_slots: Vec<KitSlot> = Vec::new();
    if let Some((note_path, note_text)) = note {
        let mut players_taken = 0usize;
        let mut gk_taken = false;
        const PLAYER_SLOTS: [KitSlot; 9] = [
            KitSlot::P1,
            KitSlot::P2,
            KitSlot::P3,
            KitSlot::P4,
            KitSlot::P5,
            KitSlot::P6,
            KitSlot::P7,
            KitSlot::P8,
            KitSlot::P9,
        ];
        let note_scope = || Scope::File {
            export_id: INPUT,
            path: path_scope(note_path),
        };
        for entry in note::color_entries(note_text).kits {
            let goalkeeper = entry.line.to_ascii_lowercase().contains("gk:");
            let slot = if goalkeeper {
                if gk_taken {
                    None
                } else {
                    gk_taken = true;
                    Some(KitSlot::G1)
                }
            } else {
                players_taken += 1;
                PLAYER_SLOTS.get(players_taken - 1).copied()
            };
            let Some(slot) = slot else {
                failures
                    .entry(note_path.to_owned())
                    .or_default()
                    .push(if goalkeeper {
                        tool_message(
                            Code::LeftoverDropped,
                            note_scope(),
                            Disposition::DropFile,
                            vec![("line", entry.line)],
                        )
                    } else {
                        unmigrated(
                            note_scope(),
                            Disposition::DropFile,
                            UnmigratedReason::KitEntryPastNinth,
                            None,
                            vec![("line", entry.line)],
                        )
                    });
                continue;
            };
            entry_slots.push(slot);
            match kit_entry(&entry.colors) {
                Some(parsed) => {
                    let colors = format!("{}\n{}\n", parsed.colors[0], parsed.colors[1]);
                    folders.written.push((
                        format!("Kits/{}/colors.txt", slot.as_str()),
                        colors.into_bytes(),
                    ));
                    if let Some(icon) = parsed.icon {
                        folders
                            .written
                            .push((format!("Kits/{}/icon_{icon}", slot.as_str()), Vec::new()));
                    }
                }
                None => {
                    failures
                        .entry(note_path.to_owned())
                        .or_default()
                        .push(unmigrated(
                            note_scope(),
                            Disposition::DropFile,
                            UnmigratedReason::ColorsUnreadable,
                            None,
                            vec![("line", entry.line)],
                        ));
                }
            }
        }
        // Each slot an entry gives and no config does is a placeholder kit,
        // kept with a Warning on the Note.
        for slot in &entry_slots {
            if !decided.contains_key(slot) {
                failures
                    .entry(note_path.to_owned())
                    .or_default()
                    .push(tool_message(
                        Code::KitWithoutConfig,
                        Scope::File {
                            export_id: INPUT,
                            path: path_scope(note_path),
                        },
                        Disposition::Keep,
                        vec![("slot", slot.as_str().to_owned())],
                    ));
            }
        }
    }
    // And each slot a config gives and no entry does, a Warning on the config
    // that became its `config.toml` (every config without a Note).
    for (slot, decided) in &decided {
        if !entry_slots.contains(slot) {
            failures
                .entry(decided.path.clone())
                .or_default()
                .push(tool_message(
                    Code::ConfigWithoutKitEntry,
                    Scope::File {
                        export_id: INPUT,
                        path: path_scope(decided.path),
                    },
                    Disposition::Keep,
                    vec![("slot", slot.as_str().to_owned())],
                ));
        }
    }

    // `Kit Textures`, a file directly in the folder alone. Step 10's
    // leftovers, which neither pass below consumes: a file below a
    // subfolder, a name `texture_slot` does not read, and — a `Full`
    // export's alone — a texture naming a slot no config and no Note
    // entry gives. A `Midcup` export's such texture is that kit's texture
    // update and migrates, the folder holding no `config.toml`.
    let slots: BTreeSet<KitSlot> = decided
        .keys()
        .copied()
        .chain(entry_slots.iter().copied())
        .collect();
    for file in files {
        let Some((top, name)) = file.split_once('/') else {
            continue;
        };
        if !top.eq_ignore_ascii_case("kit textures") {
            continue;
        }
        if name.contains('/')
            || texture_slot(name).is_none_or(|(slot, _, _)| full && !slots.contains(&slot))
        {
            folders.leftovers.insert(file.clone());
        }
    }
    // Of two landing on one folded name a `.dds` wins over an `.ftex`
    // whatever the input order (step 5, `u0XXXp1.dds` before `u0XXXp1.ftex`,
    // the DDS kept); between two of one kind the first by input path wins.
    // The `.dds` pass runs first, so an `.ftex` finding its name taken
    // loses, reported with `other` the input path of the texture that took
    // it. An `.ftex` is converted to the DDS it holds, one that does not
    // convert reported as a config that does not decode is.
    let mut taken: BTreeMap<String, String> = BTreeMap::new();
    for ftex_pass in [false, true] {
        for file in files {
            let Some((top, name)) = file.split_once('/') else {
                continue;
            };
            if !top.eq_ignore_ascii_case("kit textures") || folders.leftovers.contains(file) {
                continue;
            }
            let Some((slot, rest, ftex)) = texture_slot(name) else {
                continue;
            };
            if ftex != ftex_pass {
                continue;
            }
            let output = format!("Kits/{}/kit{rest}", slot.as_str());
            if let Some(winner) = taken.get(&vtree::fold_name(&output)) {
                let winner = winner.clone();
                failures
                    .entry(file.as_str().to_owned())
                    .or_default()
                    .push(unmigrated(
                        Scope::File {
                            export_id: INPUT,
                            path: path_scope(file),
                        },
                        Disposition::DropFile,
                        UnmigratedReason::TextureNameTaken,
                        Some(winner),
                        vec![],
                    ));
                folders.consumed.insert(file.clone());
                continue;
            }
            taken.insert(vtree::fold_name(&output), file.clone());
            if ftex {
                match read(file) {
                    Ok(bytes) => {
                        match ftex::ftex_to_dds(&bytes) {
                            Ok(dds) => folders.written.push((output, dds)),
                            Err(error) => failures
                                .entry(file.as_str().to_owned())
                                .or_default()
                                .push(unmigrated(
                                    Scope::File {
                                        export_id: INPUT,
                                        path: path_scope(file),
                                    },
                                    Disposition::DropFile,
                                    UnmigratedReason::TextureUndecodable,
                                    None,
                                    vec![("error", error.to_string())],
                                )),
                        }
                    }
                    Err(failure) => failures
                        .entry(file.as_str().to_owned())
                        .or_default()
                        .push(read_failed(INPUT, failure, Disposition::DropFile)),
                }
            } else {
                folders.copies.push((file.clone(), output));
            }
            folders.consumed.insert(file.clone());
        }
    }

    // The textures each kit holds before any copy of its own, by slot then
    // by folded stem (`Kits/<slot>/<name>`): the copy pass's source pool and
    // "the kit holds its own" check — a copy is never the source of another.
    let mut held: BTreeMap<KitSlot, BTreeMap<String, (String, String)>> = BTreeMap::new();
    for (input, output) in &folders.copies {
        if let Some(slot) = output.split('/').nth(1).and_then(KitSlot::parse) {
            held.entry(slot)
                .or_default()
                .insert(folded_stem(output), (input.clone(), output.clone()));
        }
    }

    // The copy (step 5): in slot order, each config's `back`, `chest`, `leg`,
    // `name` field (the `kit` field is never read). A field naming the kit's
    // own texture of that stem does nothing; one naming another texture — a
    // slot and a rest — gets a copy of the named slot's output texture of
    // `kit<rest>` under `kit_<field>.<the source's extension>`, when there
    // is a source and the kit holds no file of the field's name.
    const FIELD_STEMS: [(usize, &str); 4] = [(1, "back"), (2, "chest"), (3, "leg"), (4, "name")];
    for (slot, decided) in &decided {
        for (index, field) in FIELD_STEMS {
            let Some((named, rest)) = field_texture(&decided.fields[index], *slot) else {
                continue;
            };
            let stem = format!("kit_{field}");
            if named == *slot && vtree::fold_name(&format!("kit{rest}")) == vtree::fold_name(&stem)
            {
                continue;
            }
            let kit_files = held.get(slot);
            if kit_files.is_some_and(|files| files.contains_key(&vtree::fold_name(&stem))) {
                continue;
            }
            let wanted = vtree::fold_name(&format!("kit{rest}"));
            let Some((input, source)) = held.get(&named).and_then(|files| files.get(&wanted))
            else {
                continue;
            };
            let extension = source.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
            folders.copies.push((
                input.clone(),
                format!("Kits/{}/{stem}.{extension}", slot.as_str()),
            ));
            let source_stem = source
                .rsplit('/')
                .next()
                .and_then(|file| file.rsplit_once('.'))
                .map(|(stem, _)| stem)
                .unwrap_or_default();
            let from = format!("{}/{source_stem}", named.as_str());
            failures
                .entry(decided.path.clone())
                .or_default()
                .push(tool_message(
                    Code::KitTextureCopied,
                    Scope::File {
                        export_id: INPUT,
                        path: path_scope(decided.path),
                    },
                    Disposition::Keep,
                    vec![
                        ("kit", slot.as_str().to_owned()),
                        ("stem", stem),
                        ("from", from),
                    ],
                ));
        }
    }

    // The hoist (step 5): with two kit folders or more — every slot the plan
    // writes into, plus the empty `p1` a Full export gains when no player
    // kit is written and the empty `g1` when no `g1` is — a texture every
    // kit folder holds with byte-identical contents moves to `Kits/all/`
    // under the first kit's spelling, its per-kit copies dropped from the
    // plan.
    let mut kits: BTreeSet<KitSlot> = BTreeSet::new();
    for output in folders
        .written
        .iter()
        .map(|(output, _)| output)
        .chain(folders.copies.iter().map(|(_, output)| output))
    {
        if let Some(slot) = output.split('/').nth(1).and_then(KitSlot::parse) {
            kits.insert(slot);
        }
    }
    if full {
        // As the shared validation gives a Full export its empty kits: `p1`
        // only when no player slot is written, `g1` only when no `g1` is.
        if !kits.iter().any(|slot| *slot != KitSlot::G1) {
            kits.insert(KitSlot::P1);
        }
        if !kits.contains(&KitSlot::G1) {
            kits.insert(KitSlot::G1);
        }
    }
    let mut export_findings = Vec::new();
    if kits.len() >= 2 {
        // Per kit, folded stem → index into `copies` (its own texture files
        // alone: written files are never hoisted).
        let per_kit: Vec<BTreeMap<String, usize>> = kits
            .iter()
            .map(|slot| {
                folders
                    .copies
                    .iter()
                    .enumerate()
                    .filter(|(_, (_, output))| {
                        output.starts_with(&format!("Kits/{}/", slot.as_str()))
                    })
                    .map(|(index, (_, output))| (folded_stem(output), index))
                    .collect()
            })
            .collect();
        let mut dropped = BTreeSet::new();
        for (stem, first_index) in &per_kit[0] {
            let first_index = *first_index;
            // Every other kit must hold the stem before any byte is read, and
            // the first kit's texture is read once for all the compares; a
            // read failure is no match (the copy's own read reports it).
            if !per_kit[1..]
                .iter()
                .all(|kit_files| kit_files.contains_key(stem))
            {
                continue;
            }
            let Ok(first_bytes) = read(&folders.copies[first_index].0) else {
                continue;
            };
            let shared = per_kit[1..].iter().all(|kit_files| {
                read(&folders.copies[kit_files[stem]].0).is_ok_and(|bytes| bytes == first_bytes)
            });
            if !shared {
                continue;
            }
            let file = folders.copies[first_index]
                .1
                .rsplit('/')
                .next()
                .expect("a copy's output always holds a file name")
                .to_owned();
            folders.copies[first_index].1 = format!("Kits/all/{file}");
            for kit_files in &per_kit[1..] {
                dropped.insert(kit_files[stem]);
            }
            export_findings.push(tool_message(
                Code::KitTextureHoisted,
                Scope::Export { export_id: INPUT },
                Disposition::Keep,
                vec![(
                    "stem",
                    file.rsplit_once('.')
                        .map(|(stem, _)| stem)
                        .unwrap_or(&file)
                        .to_owned(),
                )],
            ));
        }
        folders.copies = std::mem::take(&mut folders.copies)
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !dropped.contains(index))
            .map(|(_, copy)| copy)
            .collect();
    }

    // The findings in input-path order (`files` is sorted): the decisions
    // were met slot by slot; the export-scoped findings (the hoists) come
    // last, met last.
    for file in files {
        if let Some(messages) = failures.remove(file.as_str()) {
            folders.findings.extend(messages);
        }
    }
    folders.findings.extend(failures.into_values().flatten());
    folders.findings.extend(export_findings);
    folders
}

#[cfg(test)]
mod tests {
    use super::*;

    use studio_core::Severity;

    const CONFIG_1ST: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/4CCG VGL26/XXX_DEF_1st_realUni.bin");
    const CONFIG_2ND: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/4CCG VGL26/XXX_DEF_2nd_realUni.bin");
    const CONFIG_GK1ST: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/4CCG VGL26/XXX_DEF_GK1st_realUni.bin");
    const MGGG_1ST: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/MGGG VGL26/XXX_DEF_1st_realUni.bin");
    const MGGG_2ND: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/MGGG VGL26/XXX_DEF_2nd_realUni.bin");
    const MGGG_GK1ST: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/MGGG VGL26/XXX_DEF_GK1st_realUni.bin");

    /// `bytes` decoded and converted the way `kit_folders` writes it.
    fn expected_toml(bytes: &[u8]) -> Vec<u8> {
        let mut config = KitConfig::decode(bytes, PesVersion::Pes21).unwrap();
        config.source_texture_names = None;
        config.to_toml().into_bytes()
    }

    /// `kit_folders` over `listing`, each file's bytes behind `read`; a
    /// listed path without bytes is a test bug, so it panics rather than
    /// failing a read. Not a Full export (no empty kits for the hoist).
    fn plan(listing: &[&str], bytes: &[(&str, &[u8])]) -> KitFolders {
        plan_full(listing, bytes, false)
    }

    /// `plan` with `full` as the caller gives it.
    fn plan_full(listing: &[&str], bytes: &[(&str, &[u8])], full: bool) -> KitFolders {
        let bytes: BTreeMap<&str, &[u8]> = bytes.iter().copied().collect();
        let files: Vec<String> = listing.iter().map(|file| (*file).to_owned()).collect();
        kit_folders(&files, PesVersion::Pes21, None, full, |path| {
            bytes
                .get(path)
                .map(|bytes| Ok(bytes.to_vec()))
                .unwrap_or_else(|| panic!("a listed file has no bytes: {path}"))
        })
    }

    #[test]
    fn a_classify_unknown_file_in_kit_configs_is_a_leftover() {
        // Step 10's stray rule wherever it sits: `notes.doc` is no `.txt`
        // note, `notes.txt` is unmigrated content — its name is no config
        // name.
        let plan = plan(
            &[
                "kit configs/XXX_DEF_1st_realUni.bin",
                "kit configs/notes.doc",
                "kit configs/notes.txt",
            ],
            &[("kit configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST)],
        );
        assert!(plan.leftovers.contains("kit configs/notes.doc"));
        assert!(!plan.consumed.contains("kit configs/notes.doc"));
        assert!(plan.consumed.contains("kit configs/notes.txt"));
        assert!(!plan.leftovers.contains("kit configs/notes.txt"));
        let finding = plan
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "unmigrated_content")
            .expect("unmigrated_content expected");
        assert_eq!(
            finding.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("kit configs/notes.txt")
            }
        );
        assert_eq!(
            finding.context,
            [("reason".to_owned(), "config_name_invalid".to_owned())]
        );
    }

    #[test]
    fn the_config_name_grammar() {
        for (name, expected) in [
            ("XXX_DEF_1st_realUni.bin", Some(KitSlot::P1)),
            ("714_def_2nd_realuni.bin", Some(KitSlot::P2)),
            ("XXX_DEF_GK1st_realUni.bin", Some(KitSlot::G1)),
            ("XXX_DEF_GK2nd_realUni.bin", None),
            ("XXX_DEF_10th_realUni.bin", None),
            ("XXX-DEF_1st_realUni.bin", None),
            ("XXX_DEF_1st_realUni.txt", None),
            ("XX_DEF_1st_realUni.bin", None),
        ] {
            assert_eq!(config_slot(name), expected, "{name}");
        }
    }

    #[test]
    fn a_goalkeeper_config_past_the_first_is_a_leftover() {
        // `GK2nd` … `GK9th`, letter case aside, are the config names a
        // second or later goalkeeper kit carries (step 10); `GK1st` reads
        // a slot, `GK10th` and any other refused name stay for the
        // fallback.
        for (name, expected) in [
            ("XXX_DEF_GK2nd_realUni.bin", true),
            ("714_def_gk9th_realuni.bin", true),
            ("XXX_DEF_GK1st_realUni.bin", false),
            ("XXX_DEF_GK10th_realUni.bin", false),
            ("XXX_DEF_1st_realUni.bin", false),
            ("XXX_DEF_GK2nd.bin", false),
        ] {
            assert_eq!(gk_config_beyond_first(name), expected, "{name}");
        }
        let plan = plan(
            &["Kit Configs/XXX_DEF_GK2nd_realUni.bin"],
            &[("Kit Configs/XXX_DEF_GK2nd_realUni.bin", CONFIG_GK1ST)],
        );
        assert!(
            plan.leftovers
                .contains("Kit Configs/XXX_DEF_GK2nd_realUni.bin")
        );
        assert!(
            !plan
                .consumed
                .contains("Kit Configs/XXX_DEF_GK2nd_realUni.bin")
        );
        assert!(plan.written.is_empty());
    }

    #[test]
    fn the_texture_name_grammar() {
        for (name, expected) in [
            ("u0XXXp1.dds", Some((KitSlot::P1, ".dds", false))),
            ("u0714p2.dds", Some((KitSlot::P2, ".dds", false))),
            ("U0XXXP1_Back.DDS", Some((KitSlot::P1, "_Back.DDS", false))),
            ("u0XXXg1_name.dds", Some((KitSlot::G1, "_name.dds", false))),
            ("u0XXXp1.ftex", Some((KitSlot::P1, ".dds", true))),
            ("u0XXXg1_name.FTEX", Some((KitSlot::G1, "_name.dds", true))),
            ("u0XXXp0.dds", None),
            ("u0XXXg2.dds", None),
            ("u0XXXp1 copy.dds", None),
            ("u0XXXp1.png", None),
            ("u0XXXp1.ftx", None),
            ("kit.dds", None),
            ("u0XXXp1", None),
        ] {
            assert_eq!(
                texture_slot(name).map(|(slot, tail, ftex)| (slot, tail.into_owned(), ftex)),
                expected.map(|(slot, tail, ftex)| (slot, tail.to_owned(), ftex)),
                "{name}"
            );
        }
    }

    #[test]
    fn the_kit_entry_grammar() {
        let icon = |entry: Option<KitEntry>| entry.map(|entry| entry.icon);
        // Two colors read as spelled; a third part 0-23 other than 3 is the
        // icon (3 is the default and writes none).
        let entry = kit_entry("211 74 79 - 162 62 77 - 3").expect("reads");
        assert_eq!(entry.colors, ["211 74 79", "162 62 77"]);
        assert_eq!(entry.icon, None);
        assert_eq!(icon(kit_entry("30 30 30 - 250 250 250 - 5")), Some(Some(5)));
        assert_eq!(icon(kit_entry("0 0 0 - 1 1 1 - 0")), Some(Some(0)));
        let entry = kit_entry("#D34A4F - #A23E4D").expect("reads");
        assert_eq!(entry.colors, ["#D34A4F", "#A23E4D"]);
        assert_eq!(entry.icon, None);
        assert_eq!(icon(kit_entry("1 2 3 - 4 5 6 - 24")), Some(None));
        assert_eq!(icon(kit_entry("1 2 3 - 4 5 6 - x")), Some(None));
        // An empty part (a trailing `-`) is no color: it reads nothing and
        // refuses nothing.
        for colors in [
            "#RRGGBB - #RRGGBB",
            "1 2 3",
            "1 2 3 - 4 5 6 - 7 - 8",
            "1 2 3 - ",
        ] {
            assert_eq!(
                kit_entry(colors).map(|entry| entry.colors),
                None,
                "{colors}"
            );
        }
    }

    #[test]
    fn a_root_and_a_subfolder_config_with_equal_bytes_are_one() {
        let plan = plan(
            &[
                "Kit Configs/714/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_1st_realUni.bin",
            ],
            &[
                ("Kit Configs/714/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
            ],
        );
        assert_eq!(
            plan.written,
            vec![("Kits/p1/config.toml".to_owned(), expected_toml(CONFIG_1ST))]
        );
        for file in [
            "Kit Configs/714/XXX_DEF_1st_realUni.bin",
            "Kit Configs/XXX_DEF_1st_realUni.bin",
        ] {
            assert!(plan.consumed.contains(file), "{file}");
        }
    }

    #[test]
    fn a_second_config_of_a_slot_with_other_bytes_is_unmigrated_content() {
        let plan = plan(
            &[
                "Kit Configs/714/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_1st_realUni.bin",
            ],
            &[
                ("Kit Configs/714/XXX_DEF_1st_realUni.bin", CONFIG_2ND),
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
            ],
        );
        // The root's is the kit's whatever order the listing puts them in;
        // the second is unmigrated, `other` naming the kept config.
        assert_eq!(
            plan.written,
            vec![("Kits/p1/config.toml".to_owned(), expected_toml(CONFIG_1ST))]
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/714/XXX_DEF_1st_realUni.bin")
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/XXX_DEF_1st_realUni.bin")
        );
        let finding = plan
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "unmigrated_content")
            .expect("unmigrated_content expected");
        assert_eq!(
            finding.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Kit Configs/714/XXX_DEF_1st_realUni.bin")
            }
        );
        assert_eq!(
            finding.context,
            [
                ("reason".to_owned(), "config_taken".to_owned()),
                (
                    "other".to_owned(),
                    "Kit Configs/XXX_DEF_1st_realUni.bin".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn the_first_root_config_of_a_slot_by_path_is_the_kits() {
        let plan = plan(
            &[
                "Kit Configs/714_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_1st_realUni.bin",
            ],
            &[
                ("Kit Configs/714_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_2ND),
            ],
        );
        assert_eq!(plan.written[0].1, expected_toml(CONFIG_1ST));
        assert!(
            plan.consumed
                .contains("Kit Configs/XXX_DEF_1st_realUni.bin")
        );
        // The second of the slot is unmigrated, `other` the kept config.
        let finding = plan
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "unmigrated_content")
            .expect("unmigrated_content expected");
        assert_eq!(
            finding.context,
            [
                ("reason".to_owned(), "config_taken".to_owned()),
                (
                    "other".to_owned(),
                    "Kit Configs/714_DEF_1st_realUni.bin".to_owned()
                ),
            ]
        );
    }

    #[test]
    fn a_config_deeper_than_one_subfolder_is_unmigrated_content() {
        let plan = plan(
            &["Kit Configs/a/b/XXX_DEF_1st_realUni.bin"],
            &[("Kit Configs/a/b/XXX_DEF_1st_realUni.bin", CONFIG_1ST)],
        );
        assert!(plan.written.is_empty());
        assert!(
            plan.consumed
                .contains("Kit Configs/a/b/XXX_DEF_1st_realUni.bin")
        );
        assert_eq!(plan.findings.len(), 1, "{:?}", plan.findings);
        assert_eq!(
            plan.findings[0].context,
            [("reason".to_owned(), "config_too_deep".to_owned())]
        );
    }

    #[test]
    fn a_config_that_does_not_decode_is_unmigrated_content() {
        let plan = plan(
            &["Kit Configs/XXX_DEF_2nd_realUni.bin"],
            &[("Kit Configs/XXX_DEF_2nd_realUni.bin", b"bad cfg")],
        );
        assert_eq!(plan.findings.len(), 1, "{:?}", plan.findings);
        let finding = &plan.findings[0];
        assert_eq!(finding.code.code.as_ref(), "unmigrated_content");
        assert_eq!(
            finding.context,
            vec![
                ("reason".to_owned(), "config_undecodable".to_owned()),
                (
                    "error".to_owned(),
                    "kit config is 7 bytes, expected 120".to_owned()
                ),
            ]
        );
        assert_eq!(
            finding.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Kit Configs/XXX_DEF_2nd_realUni.bin")
            }
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/XXX_DEF_2nd_realUni.bin")
        );
        assert!(plan.written.is_empty());
    }

    #[test]
    fn a_decode_failure_leaves_the_slot_to_the_next_config() {
        let plan = plan(
            &[
                "Kit Configs/714/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_1st_realUni.bin",
            ],
            &[
                ("Kit Configs/714/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_1st_realUni.bin", b"bad cfg"),
            ],
        );
        // The root's failed to decode; the subfolder's becomes the kit's.
        assert_eq!(
            plan.written,
            vec![("Kits/p1/config.toml".to_owned(), expected_toml(CONFIG_1ST))]
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/XXX_DEF_1st_realUni.bin")
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/714/XXX_DEF_1st_realUni.bin")
        );
        // Input-path order: the surviving config's warning on `714/`'s file,
        // then the root file's decode failure.
        assert_eq!(plan.findings.len(), 2, "{:?}", plan.findings);
        assert_eq!(
            plan.findings[0].code.code.as_ref(),
            "config_without_kit_entry"
        );
        assert_eq!(plan.findings[1].code.code.as_ref(), "unmigrated_content");
        assert_eq!(
            plan.findings[1].context[0],
            ("reason".to_owned(), "config_undecodable".to_owned())
        );
    }

    #[test]
    fn an_ftex_that_does_not_convert_is_unmigrated_content_with_its_error() {
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/u0XXXp1.ftex",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Textures/u0XXXp1.ftex", b"not an ftex"),
            ],
        );
        // No `kit.dds` is written; the input is consumed and reported with
        // the conversion's error as its `error` context.
        assert!(
            plan.written
                .iter()
                .all(|(output, _)| output != "Kits/p1/kit.dds")
        );
        assert!(plan.copies.is_empty());
        assert!(plan.consumed.contains("Kit Textures/u0XXXp1.ftex"));
        let finding = plan
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "unmigrated_content")
            .expect("an unmigrated_content finding");
        assert_eq!(
            finding.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Kit Textures/u0XXXp1.ftex")
            }
        );
        assert_eq!(
            finding.context[0],
            ("reason".to_owned(), "texture_undecodable".to_owned())
        );
        let error = finding
            .context
            .iter()
            .find(|(key, _)| key == "error")
            .map(|(_, value)| value);
        assert!(error.is_some_and(|value| !value.is_empty()), "{finding:?}");
    }

    #[test]
    fn a_full_exports_texture_of_an_ungiven_slot_is_a_leftover() {
        // A `Full` export's (`full` true): the slot no config and no Note
        // entry gives is a kit row that never named it (step 10).
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp5.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", CONFIG_2ND),
                ("Kit Textures/u0XXXp5.dds", b"tex"),
            ],
            true,
        );
        assert!(plan.leftovers.contains("Kit Textures/u0XXXp5.dds"));
        assert!(!plan.consumed.contains("Kit Textures/u0XXXp5.dds"));
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn a_midcup_texture_of_an_ungiven_slot_is_its_kits_update() {
        // A `Midcup` export's (`full` false) migrates: the slot's kit
        // folder holds its textures and no `config.toml`.
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/u0XXXp3.dds",
                "Kit Textures/u0XXXp3_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Textures/u0XXXp3.dds", b"main"),
                ("Kit Textures/u0XXXp3_back.dds", b"back"),
            ],
        );
        for (input, output) in [
            ("Kit Textures/u0XXXp3.dds", "Kits/p3/kit.dds"),
            ("Kit Textures/u0XXXp3_back.dds", "Kits/p3/kit_back.dds"),
        ] {
            assert!(
                plan.copies.contains(&(input.to_owned(), output.to_owned())),
                "{output}: {:?}",
                plan.copies
            );
            assert!(plan.consumed.contains(input), "{input}");
        }
        assert!(
            !plan
                .written
                .iter()
                .any(|(output, _)| output.starts_with("Kits/p3/")),
            "no config.toml or colors.txt for p3: {:?}",
            plan.written
        );
    }

    #[test]
    fn a_name_texture_slot_does_not_read_is_a_leftover() {
        // In any export kind (step 10): a name of no kit texture of a
        // given slot — no `u`-led stem, a rest not `_`-led, a `g2`/`p0`
        // slot, a non-DDS/FTEX extension.
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/back.dds",
                "Kit Textures/u0XXXp2 - Copy.dds",
                "Kit Textures/u0XXXg2.dds",
                "Kit Textures/u0XXXp0.dds",
                "Kit Textures/u0XXXp3.png",
            ],
            &[("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST)],
        );
        for file in [
            "Kit Textures/back.dds",
            "Kit Textures/u0XXXp2 - Copy.dds",
            "Kit Textures/u0XXXg2.dds",
            "Kit Textures/u0XXXp0.dds",
            "Kit Textures/u0XXXp3.png",
        ] {
            assert!(plan.leftovers.contains(file), "{file}");
            assert!(!plan.consumed.contains(file), "{file}");
        }
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn a_texture_inside_a_subfolder_is_a_leftover() {
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/714/u0XXXp1.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Textures/714/u0XXXp1.dds", b"tex"),
            ],
        );
        assert!(plan.leftovers.contains("Kit Textures/714/u0XXXp1.dds"));
        assert!(!plan.consumed.contains("Kit Textures/714/u0XXXp1.dds"));
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn a_kit_texture_name_outside_kit_textures_is_not_consumed() {
        let plan = plan(
            &["Kit Configs/XXX_DEF_1st_realUni.bin", "Other/u0XXXp1.dds"],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Other/u0XXXp1.dds", b"tex"),
            ],
        );
        assert!(!plan.consumed.contains("Other/u0XXXp1.dds"));
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn two_textures_landing_on_one_name_keep_the_first() {
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/u0714p1.dds",
                "Kit Textures/u0XXXp1.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Textures/u0714p1.dds", b"714"),
                ("Kit Textures/u0XXXp1.dds", b"xxx"),
            ],
        );
        assert_eq!(
            plan.copies,
            vec![(
                "Kit Textures/u0714p1.dds".to_owned(),
                "Kits/p1/kit.dds".to_owned()
            )]
        );
        assert!(plan.consumed.contains("Kit Textures/u0714p1.dds"));
        // The second is unmigrated, `other` the texture that took the name.
        assert!(plan.consumed.contains("Kit Textures/u0XXXp1.dds"));
        let finding = plan
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "unmigrated_content")
            .expect("unmigrated_content expected");
        assert_eq!(
            finding.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Kit Textures/u0XXXp1.dds")
            }
        );
        assert_eq!(
            finding.context,
            [
                ("reason".to_owned(), "texture_name_taken".to_owned()),
                ("other".to_owned(), "Kit Textures/u0714p1.dds".to_owned()),
            ]
        );
    }

    #[test]
    fn the_top_folders_match_in_any_letter_case() {
        let plan = plan(
            &[
                "kit configs/XXX_DEF_1st_realUni.bin",
                "KIT TEXTURES/u0XXXp1.dds",
            ],
            &[
                ("kit configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("KIT TEXTURES/u0XXXp1.dds", b"main"),
            ],
        );
        assert_eq!(plan.written[0].0, "Kits/p1/config.toml");
        assert!(plan.copies.contains(&(
            "KIT TEXTURES/u0XXXp1.dds".to_owned(),
            "Kits/p1/kit.dds".to_owned()
        )));
    }

    #[test]
    fn a_configs_read_failure_is_reported_and_the_next_candidate_decides() {
        let files: Vec<String> = [
            "Kit Configs/714/XXX_DEF_1st_realUni.bin",
            "Kit Configs/XXX_DEF_1st_realUni.bin",
        ]
        .iter()
        .map(|file| (*file).to_owned())
        .collect();
        let plan = kit_folders(&files, PesVersion::Pes21, None, false, |path| {
            if path == "Kit Configs/XXX_DEF_1st_realUni.bin" {
                Err(SourceFailure {
                    path: path.to_owned(),
                    error: "denied".to_owned(),
                })
            } else {
                Ok(CONFIG_1ST.to_vec())
            }
        });
        // The subfolder's decides p1; the root file's read failure is
        // reported and the file consumed.
        assert_eq!(plan.written[0].0, "Kits/p1/config.toml");
        assert!(
            plan.consumed
                .contains("Kit Configs/XXX_DEF_1st_realUni.bin")
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/714/XXX_DEF_1st_realUni.bin")
        );
        // Input-path order: `714/`'s warning (its config survives), then the
        // root file's `source_read_failed`.
        let failed: Vec<&Message> = plan
            .findings
            .iter()
            .filter(|m| m.code.code.as_ref() == "source_read_failed")
            .collect();
        assert_eq!(failed.len(), 1, "{:?}", plan.findings);
        assert_eq!(failed[0].disposition, Disposition::DropFile);
        assert_eq!(
            failed[0].scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Kit Configs/XXX_DEF_1st_realUni.bin")
            }
        );
    }

    #[test]
    fn a_field_names_its_texture_by_position() {
        let field = |text: &str| {
            let mut field = [0u8; 16];
            field[..text.len()].copy_from_slice(text.as_bytes());
            field
        };
        // `u`, four characters, the slot, the rest — `p0` the config's own
        // in any letter case.
        assert_eq!(
            field_texture(&field("u0XXXp1_back"), KitSlot::P1),
            Some((KitSlot::P1, "_back"))
        );
        assert_eq!(
            field_texture(&field("u0728p1_front"), KitSlot::P1),
            Some((KitSlot::P1, "_front"))
        );
        assert_eq!(
            field_texture(&field("u0XXXp0_back"), KitSlot::P2),
            Some((KitSlot::P2, "_back"))
        );
        assert_eq!(
            field_texture(&field("U0XXXP0_leg"), KitSlot::G1),
            Some((KitSlot::G1, "_leg"))
        );
        for text in ["", "kit_back", "u0p1_back", "u0XXXxx"] {
            assert_eq!(field_texture(&field(text), KitSlot::P1), None, "{text:?}");
        }
        // A field that is not UTF-8 names nothing either.
        let mut field = [0xFFu8; 16];
        field[0] = b'u';
        assert_eq!(field_texture(&field, KitSlot::P1), None);
    }

    #[test]
    fn a_field_naming_another_stem_of_its_own_kit_copies_it() {
        // MGGG `1st`: its leg field names `u0811p1_back` — its own slot, the
        // back's stem — so `kit_leg.dds` is a copy of its `kit_back.dds`.
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/u0XXXp1.dds",
                "Kit Textures/u0XXXp1_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", MGGG_1ST),
                ("Kit Textures/u0XXXp1.dds", b"main"),
                ("Kit Textures/u0XXXp1_back.dds", b"back"),
            ],
        );
        assert!(plan.copies.contains(&(
            "Kit Textures/u0XXXp1_back.dds".to_owned(),
            "Kits/p1/kit_leg.dds".to_owned()
        )));
        // Its name field names `u0811p1_name`, which the export does not
        // ship: no `kit_name.dds` copy.
        assert!(
            !plan
                .copies
                .iter()
                .any(|(_, to)| to == "Kits/p1/kit_name.dds")
        );
        let copied: Vec<&Message> = plan
            .findings
            .iter()
            .filter(|m| m.code.code.as_ref() == "kit_texture_copied")
            .collect();
        assert_eq!(copied.len(), 1, "{:?}", plan.findings);
        assert_eq!(copied[0].severity, Severity::Info);
        assert_eq!(copied[0].disposition, Disposition::Keep);
        assert_eq!(
            copied[0].context,
            vec![
                ("kit".to_owned(), "p1".to_owned()),
                ("stem".to_owned(), "kit_leg".to_owned()),
                ("from".to_owned(), "p1/kit_back".to_owned()),
            ]
        );
        assert_eq!(
            copied[0].scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Kit Configs/XXX_DEF_1st_realUni.bin")
            }
        );
    }

    #[test]
    fn each_field_naming_another_kits_texture_gets_its_own_copy() {
        // MGGG `1st` and `2nd`: the 2nd's back, leg and name fields all name
        // `u0811p1_*` — three copies, three findings on its config file.
        // `full` as a Full export's: the empty `g1` keeps every stem
        // per-kit.
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp1.dds",
                "Kit Textures/u0XXXp1_back.dds",
                "Kit Textures/u0XXXp1_name.dds",
                "Kit Textures/u0XXXp2.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", MGGG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", MGGG_2ND),
                ("Kit Textures/u0XXXp1.dds", b"p1 main"),
                ("Kit Textures/u0XXXp1_back.dds", b"p1 back"),
                ("Kit Textures/u0XXXp1_name.dds", b"p1 name"),
                ("Kit Textures/u0XXXp2.dds", b"p2 main"),
            ],
            true,
        );
        for to in [
            "Kits/p2/kit_back.dds",
            "Kits/p2/kit_leg.dds",
            "Kits/p2/kit_name.dds",
        ] {
            assert!(plan.copies.iter().any(|(_, target)| target == to), "{to}");
        }
        let p2: Vec<&Message> = plan
            .findings
            .iter()
            .filter(|m| {
                m.code.code.as_ref() == "kit_texture_copied"
                    && m.scope
                        == Scope::File {
                            export_id: INPUT,
                            path: path_scope("Kit Configs/XXX_DEF_2nd_realUni.bin"),
                        }
            })
            .collect();
        // In `back`, `chest`, `leg`, `name` order: back, leg, name.
        assert_eq!(p2.len(), 3, "{:?}", plan.findings);
        assert_eq!(
            p2[0].context,
            vec![
                ("kit".to_owned(), "p2".to_owned()),
                ("stem".to_owned(), "kit_back".to_owned()),
                ("from".to_owned(), "p1/kit_back".to_owned()),
            ]
        );
        assert_eq!(p2[1].context[1].1, "kit_leg");
        assert_eq!(p2[2].context[1].1, "kit_name");
        assert_eq!(p2[2].context[2].1, "p1/kit_name");
    }

    #[test]
    fn a_kit_shipping_its_own_file_of_the_fields_name_gets_no_copy() {
        // MGGG `GK1st` ships its own `u0XXXg1_back.dds` while its config
        // names `u0811p1_back`: its own file wins (its leg field still gets
        // the copy, holding no `kit_leg` of its own).
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_GK1st_realUni.bin",
                "Kit Textures/u0XXXp1_back.dds",
                "Kit Textures/u0XXXg1.dds",
                "Kit Textures/u0XXXg1_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_GK1st_realUni.bin", MGGG_GK1ST),
                ("Kit Textures/u0XXXp1_back.dds", b"p1 back"),
                ("Kit Textures/u0XXXg1.dds", b"g1 main"),
                ("Kit Textures/u0XXXg1_back.dds", b"g1 back"),
            ],
        );
        assert!(plan.copies.contains(&(
            "Kit Textures/u0XXXg1_back.dds".to_owned(),
            "Kits/g1/kit_back.dds".to_owned()
        )));
        assert!(!plan.copies.iter().any(|(from, to)| {
            from == "Kit Textures/u0XXXp1_back.dds" && to == "Kits/g1/kit_back.dds"
        }));
        assert!(plan.copies.contains(&(
            "Kit Textures/u0XXXp1_back.dds".to_owned(),
            "Kits/g1/kit_leg.dds".to_owned()
        )));
    }

    #[test]
    fn a_field_naming_its_own_kit_of_the_fields_stem_does_nothing() {
        // 4CCG `1st` names its own back and name; a `p0` back field is the
        // config's own too.
        let mut second = CONFIG_2ND.to_vec();
        // The back field (index 1) at 0x38: `u0XXXp0_back` names the config's
        // own back as `p0`.
        second[0x38..0x38 + 12].copy_from_slice(b"u0XXXp0_back");
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp1_back.dds",
                "Kit Textures/u0XXXp2_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", &second),
                ("Kit Textures/u0XXXp1_back.dds", b"p1 back"),
                ("Kit Textures/u0XXXp2_back.dds", b"p2 back"),
            ],
        );
        // Only the two textures' own copies: no `kit_texture_copied` at all.
        assert_eq!(plan.copies.len(), 2, "{:?}", plan.copies);
        assert!(
            plan.findings
                .iter()
                .all(|m| m.code.code.as_ref() != "kit_texture_copied")
        );
    }

    #[test]
    fn a_chest_field_naming_its_own_kits_front_texture_copies_it() {
        // A chest field `u0728p1_front` in p1's config (JP's shape): `p1`
        // ships `u0XXXp1_front.dds`, so `kit_chest.dds` is its copy.
        let mut first = CONFIG_1ST.to_vec();
        // The chest field (index 2) at 0x48 is empty in the fixture.
        first[0x48..0x48 + 13].copy_from_slice(b"u0728p1_front");
        let plan = plan(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/u0XXXp1_front.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", &first),
                ("Kit Textures/u0XXXp1_front.dds", b"front"),
            ],
        );
        assert!(plan.copies.contains(&(
            "Kit Textures/u0XXXp1_front.dds".to_owned(),
            "Kits/p1/kit_chest.dds".to_owned()
        )));
        let copied = plan
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "kit_texture_copied")
            .expect("kit_texture_copied expected");
        assert_eq!(
            copied.context,
            vec![
                ("kit".to_owned(), "p1".to_owned()),
                ("stem".to_owned(), "kit_chest".to_owned()),
                ("from".to_owned(), "p1/kit_front".to_owned()),
            ]
        );
    }

    #[test]
    fn a_texture_every_kit_shares_moves_to_all() {
        // Midcup (`full` false): the two kits' equal `kit_back` hoists.
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp1_back.dds",
                "Kit Textures/u0XXXp2_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", CONFIG_2ND),
                ("Kit Textures/u0XXXp1_back.dds", b"back"),
                ("Kit Textures/u0XXXp2_back.dds", b"back"),
            ],
            false,
        );
        assert_eq!(
            plan.copies,
            vec![(
                "Kit Textures/u0XXXp1_back.dds".to_owned(),
                "Kits/all/kit_back.dds".to_owned()
            )]
        );
        let hoisted: Vec<&Message> = plan
            .findings
            .iter()
            .filter(|m| m.code.code.as_ref() == "kit_texture_hoisted")
            .collect();
        assert_eq!(hoisted.len(), 1, "{:?}", plan.findings);
        assert_eq!(hoisted[0].severity, Severity::Info);
        assert_eq!(hoisted[0].disposition, Disposition::Keep);
        assert_eq!(hoisted[0].scope, Scope::Export { export_id: INPUT });
        assert_eq!(
            hoisted[0].context,
            vec![("stem".to_owned(), "kit_back".to_owned())]
        );
    }

    #[test]
    fn a_full_exports_empty_p1_counts_only_when_no_player_kit_is() {
        // `p2` is a player kit, so the empty `p1` is not added; `g1` is
        // present — `kit_back` shared byte for byte hoists.
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Configs/XXX_DEF_GK1st_realUni.bin",
                "Kit Textures/u0XXXp2_back.dds",
                "Kit Textures/u0XXXg1_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", CONFIG_2ND),
                ("Kit Configs/XXX_DEF_GK1st_realUni.bin", CONFIG_GK1ST),
                ("Kit Textures/u0XXXp2_back.dds", b"back"),
                ("Kit Textures/u0XXXg1_back.dds", b"back"),
            ],
            true,
        );
        assert_eq!(
            plan.copies,
            vec![(
                "Kit Textures/u0XXXp2_back.dds".to_owned(),
                "Kits/all/kit_back.dds".to_owned()
            )]
        );
        assert!(
            plan.findings
                .iter()
                .any(|m| m.code.code.as_ref() == "kit_texture_hoisted")
        );
    }

    #[test]
    fn the_empty_g1_a_full_export_gains_counts_for_the_hoist() {
        // Full (`full` true): the same two kits, and the empty `g1` they
        // gain — no stem is in every kit folder, so nothing hoists.
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp1_back.dds",
                "Kit Textures/u0XXXp2_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", CONFIG_2ND),
                ("Kit Textures/u0XXXp1_back.dds", b"back"),
                ("Kit Textures/u0XXXp2_back.dds", b"back"),
            ],
            true,
        );
        assert!(plan.copies.contains(&(
            "Kit Textures/u0XXXp2_back.dds".to_owned(),
            "Kits/p2/kit_back.dds".to_owned()
        )));
        assert!(
            !plan
                .copies
                .iter()
                .any(|(_, to)| to.starts_with("Kits/all/"))
        );
        assert!(
            plan.findings
                .iter()
                .all(|m| m.code.code.as_ref() != "kit_texture_hoisted")
        );
    }

    #[test]
    fn one_kit_alone_never_hoists() {
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Textures/u0XXXp1_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Textures/u0XXXp1_back.dds", b"back"),
            ],
            false,
        );
        assert_eq!(
            plan.copies,
            vec![(
                "Kit Textures/u0XXXp1_back.dds".to_owned(),
                "Kits/p1/kit_back.dds".to_owned()
            )]
        );
        assert!(
            plan.findings
                .iter()
                .all(|m| m.code.code.as_ref() != "kit_texture_hoisted")
        );
    }

    #[test]
    fn textures_of_different_bytes_do_not_hoist() {
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp1_back.dds",
                "Kit Textures/u0XXXp2_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", CONFIG_2ND),
                ("Kit Textures/u0XXXp1_back.dds", b"one"),
                ("Kit Textures/u0XXXp2_back.dds", b"two"),
            ],
            false,
        );
        for to in ["Kits/p1/kit_back.dds", "Kits/p2/kit_back.dds"] {
            assert!(plan.copies.iter().any(|(_, target)| target == to), "{to}");
        }
        assert!(
            plan.findings
                .iter()
                .all(|m| m.code.code.as_ref() != "kit_texture_hoisted")
        );
    }

    #[test]
    fn a_case_folded_pair_of_equal_bytes_hoists_as_one_name() {
        // `kit_Back.DDS` and `kit_back.dds` fold to one name; the first
        // kit's spelling is kept under `all/`.
        let plan = plan_full(
            &[
                "Kit Configs/XXX_DEF_1st_realUni.bin",
                "Kit Configs/XXX_DEF_2nd_realUni.bin",
                "Kit Textures/u0XXXp1_Back.DDS",
                "Kit Textures/u0XXXp2_back.dds",
            ],
            &[
                ("Kit Configs/XXX_DEF_1st_realUni.bin", CONFIG_1ST),
                ("Kit Configs/XXX_DEF_2nd_realUni.bin", CONFIG_2ND),
                ("Kit Textures/u0XXXp1_Back.DDS", b"back"),
                ("Kit Textures/u0XXXp2_back.dds", b"back"),
            ],
            false,
        );
        assert_eq!(
            plan.copies,
            vec![(
                "Kit Textures/u0XXXp1_Back.DDS".to_owned(),
                "Kits/all/kit_Back.DDS".to_owned()
            )]
        );
        assert!(
            plan.findings
                .iter()
                .any(|m| m.code.code.as_ref() == "kit_texture_hoisted")
        );
    }
}
