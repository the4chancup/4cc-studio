//! Step 5's `Kit Configs/` and `Kit Textures/` folders into `Kits/`
//! (`plans/export_upgrader.md` step 5, `DECISIONS.md` 2026-10-18 "the kit
//! configs' and kit textures' rules of 6.3a" and "the Note's kit entries'
//! grammar and the two kit Warnings of 6.3b"): every config becomes its kit
//! folder's `config.toml` (its texture-name fields dropped — the compiler
//! re-derives them from the textures present, its undecoded bits kept in the
//! `[unknown]` table), every kit texture lands in its kit's folder under the
//! generic `kit` name, and the Note's kit entries give each kit its
//! `colors.txt` and icon. What cannot be placed is left for the run's
//! fallback to report as unmigrated content.

use std::collections::{BTreeMap, BTreeSet};

use aesthetics_export::read_colors_txt;
use kit_config::{KitConfig, KitSlot};
use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, INPUT, read_failed, tool_message};
use crate::note;
use crate::players::path_scope;
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
    /// Findings on the input, in input-path order, a file's own findings in
    /// the order met.
    pub(crate) findings: Vec<Message>,
}

/// The slot a kit config's file name carries: `???_DEF_<suffix>_realUni.bin`
/// — any three characters, `_DEF_`, a kit-config entry suffix and
/// `_realUni.bin`, the fixed parts in any letter case.
fn config_slot(file_name: &str) -> Option<KitSlot> {
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
    KitSlot::from_config_suffix(tail.get(..suffix_len)?)
}

/// The slot a kit texture's file name carries and its output name's tail:
/// `u`, any four characters, the slot's two characters (`p1` … `p9`, `g1`),
/// the rest of the stem (empty or starting with `_`), then `.dds` — the
/// fixed parts in any letter case. The rest is returned with the extension
/// as written (`_back.dds`, `.dds`), so the output name is `kit` + it.
fn texture_slot(file_name: &str) -> Option<(KitSlot, &str)> {
    let (stem, extension) = file_name.rsplit_once('.')?;
    if !extension.eq_ignore_ascii_case("dds") || !stem.get(..1)?.eq_ignore_ascii_case("u") {
        return None;
    }
    let slot_at = stem.char_indices().nth(5).map(|(index, _)| index)?;
    let slot = KitSlot::parse(stem.get(slot_at..slot_at + 2)?)?;
    let rest_of_stem = &stem[slot_at + 2..];
    if !rest_of_stem.is_empty() && !rest_of_stem.starts_with('_') {
        return None;
    }
    Some((slot, &file_name[slot_at + 2..]))
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

/// What step 5 makes of `files`' `Kit Configs/` and `Kit Textures/` and the
/// Note's kit entries (`note` the Note's path and text, `None` without one):
/// every config decoded and converted to its kit folder's `config.toml`,
/// every texture of a decided kit copied under the generic name, each kit
/// entry's colors and icon written beside it.
pub(crate) fn kit_folders(
    files: &[String],
    version: PesVersion,
    note: Option<(&str, &str)>,
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
        let name = rest.rsplit('/').next().unwrap_or(rest);
        let Some(slot) = config_slot(name) else {
            continue;
        };
        match rest.matches('/').count() {
            0 => roots.push((file, slot)),
            1 => below.push((file, slot)),
            _ => {}
        }
    }

    // A slot's config is its first candidate that reads and decodes — decided
    // before any byte compares, so a config that fails leaves the slot to the
    // next one; a later file of the slot is one config when its bytes match
    // the kit's, and not consumed otherwise.
    let mut decided: BTreeMap<KitSlot, (&String, Vec<u8>)> = BTreeMap::new();
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
                Some((_, kept)) if *kept == bytes => {
                    folders.consumed.insert((*file).clone());
                }
                Some(_) => {}
                None => match KitConfig::decode(&bytes, version) {
                    Ok(mut config) => {
                        config.source_texture_names = None;
                        folders.written.push((
                            format!("Kits/{}/config.toml", slot.as_str()),
                            config.to_toml().into_bytes(),
                        ));
                        decided.insert(*slot, (*file, bytes));
                        folders.consumed.insert((*file).clone());
                    }
                    Err(error) => {
                        failures
                            .entry(file.as_str().to_owned())
                            .or_default()
                            .push(tool_message(
                                Code::UnmigratedContent,
                                Scope::File {
                                    export_id: INPUT,
                                    path: path_scope(file),
                                },
                                Disposition::DropFile,
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
    // order. A second `gk:` entry or a player entry past the ninth has no
    // slot and is unmigrated content, and so is an entry whose colors do not
    // read — its slot still counts as given (the later entries keep theirs;
    // Red dropped it and shifted every kit after it down a slot).
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
        let unmigrated = |line: &str| {
            tool_message(
                Code::UnmigratedContent,
                Scope::File {
                    export_id: INPUT,
                    path: path_scope(note_path),
                },
                Disposition::DropFile,
                vec![("line", line.to_owned())],
            )
        };
        for entry in note::color_entries(note_text).kits {
            let slot = if entry.line.to_ascii_lowercase().contains("gk:") {
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
                    .push(unmigrated(&entry.line));
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
                        .push(unmigrated(&entry.line));
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
    for (slot, (config_path, _)) in &decided {
        if !entry_slots.contains(slot) {
            failures
                .entry((*config_path).clone())
                .or_default()
                .push(tool_message(
                    Code::ConfigWithoutKitEntry,
                    Scope::File {
                        export_id: INPUT,
                        path: path_scope(config_path),
                    },
                    Disposition::Keep,
                    vec![("slot", slot.as_str().to_owned())],
                ));
        }
    }

    // The findings in input-path order (`files` is sorted): the decisions
    // were met slot by slot.
    for file in files {
        if let Some(messages) = failures.remove(file.as_str()) {
            folders.findings.extend(messages);
        }
    }
    folders.findings.extend(failures.into_values().flatten());

    // `Kit Textures`, a file directly in the folder alone (a subfolder's is
    // not consumed): it lands only in a slot a config or a Note entry gives,
    // the first by input path of two landing on one name kept.
    let slots: BTreeSet<KitSlot> = decided
        .keys()
        .copied()
        .chain(entry_slots.iter().copied())
        .collect();
    let mut taken: BTreeSet<String> = BTreeSet::new();
    for file in files {
        let Some((top, name)) = file.split_once('/') else {
            continue;
        };
        if !top.eq_ignore_ascii_case("kit textures") || name.contains('/') {
            continue;
        }
        let Some((slot, rest)) = texture_slot(name) else {
            continue;
        };
        if !slots.contains(&slot) {
            continue;
        }
        let output = format!("Kits/{}/kit{rest}", slot.as_str());
        if !taken.insert(vtree::fold_name(&output)) {
            continue;
        }
        folders.copies.push((file.clone(), output));
        folders.consumed.insert(file.clone());
    }
    folders
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_1ST: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/4CCG VGL26/XXX_DEF_1st_realUni.bin");
    const CONFIG_2ND: &[u8] =
        include_bytes!("../tests/fixtures/kit_configs/4CCG VGL26/XXX_DEF_2nd_realUni.bin");

    /// `bytes` decoded and converted the way `kit_folders` writes it.
    fn expected_toml(bytes: &[u8]) -> Vec<u8> {
        let mut config = KitConfig::decode(bytes, PesVersion::Pes21).unwrap();
        config.source_texture_names = None;
        config.to_toml().into_bytes()
    }

    /// `kit_folders` over `listing`, each file's bytes behind `read`; a
    /// listed path without bytes is a test bug, so it panics rather than
    /// failing a read.
    fn plan(listing: &[&str], bytes: &[(&str, &[u8])]) -> KitFolders {
        let bytes: BTreeMap<&str, &[u8]> = bytes.iter().copied().collect();
        let files: Vec<String> = listing.iter().map(|file| (*file).to_owned()).collect();
        kit_folders(&files, PesVersion::Pes21, None, |path| {
            bytes
                .get(path)
                .map(|bytes| Ok(bytes.to_vec()))
                .unwrap_or_else(|| panic!("a listed file has no bytes: {path}"))
        })
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
    fn the_texture_name_grammar() {
        for (name, expected) in [
            ("u0XXXp1.dds", Some((KitSlot::P1, ".dds"))),
            ("u0714p2.dds", Some((KitSlot::P2, ".dds"))),
            ("U0XXXP1_Back.DDS", Some((KitSlot::P1, "_Back.DDS"))),
            ("u0XXXg1_name.dds", Some((KitSlot::G1, "_name.dds"))),
            ("u0XXXp0.dds", None),
            ("u0XXXg2.dds", None),
            ("u0XXXp1 copy.dds", None),
            ("u0XXXp1.png", None),
            ("kit.dds", None),
            ("u0XXXp1", None),
        ] {
            assert_eq!(texture_slot(name), expected, "{name}");
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
    fn a_second_config_of_a_slot_with_other_bytes_is_not_consumed() {
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
        // The root's is the kit's whatever order the listing puts them in.
        assert_eq!(
            plan.written,
            vec![("Kits/p1/config.toml".to_owned(), expected_toml(CONFIG_1ST))]
        );
        assert!(
            !plan
                .consumed
                .contains("Kit Configs/714/XXX_DEF_1st_realUni.bin")
        );
        assert!(
            plan.consumed
                .contains("Kit Configs/XXX_DEF_1st_realUni.bin")
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
            !plan
                .consumed
                .contains("Kit Configs/XXX_DEF_1st_realUni.bin")
        );
    }

    #[test]
    fn a_config_deeper_than_one_subfolder_is_not_consumed() {
        let plan = plan(
            &["Kit Configs/a/b/XXX_DEF_1st_realUni.bin"],
            &[("Kit Configs/a/b/XXX_DEF_1st_realUni.bin", CONFIG_1ST)],
        );
        assert!(plan.written.is_empty());
        assert!(plan.consumed.is_empty());
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
            vec![(
                "error".to_owned(),
                "kit config is 7 bytes, expected 120".to_owned()
            )]
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
    }

    #[test]
    fn a_texture_for_a_slot_without_a_config_is_not_consumed() {
        let plan = plan(
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
        );
        assert!(!plan.consumed.contains("Kit Textures/u0XXXp5.dds"));
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn a_texture_inside_a_subfolder_is_not_consumed() {
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
        assert!(!plan.consumed.contains("Kit Textures/u0XXXp1.dds"));
    }
}
