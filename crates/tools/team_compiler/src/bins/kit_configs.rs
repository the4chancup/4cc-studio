//! A team's kit configs as a compile edits them besides adding its committed kits' configs
//! (`team_compiler/pipeline.md` "Bins accumulation", "Collars"): a `Full` export's team loses
//! the configs of kits it does not hold, and a `Midcup` export gives the configs of the kits
//! the game offers the team that the export does not hold the FPC values, when its team's
//! kit-FPC status is On, then its collar, when it has one. A team's configs are the entries
//! named for its team ID (`KitSlot::config_name`: `714_DEF_1st_realUni.bin`): in
//! `UniformParameter.bin` on Fox (`kit_configs`), loose files on PES 15-17
//! (`loose_kit_configs`).

use std::collections::BTreeMap;

use aesthetics_export::ExportCoverage;
use kit_config::{KitConfig, KitSlot, apply_fpc, matches_fpc};
use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};
use uniparam::UniformParameter;

use super::{UniColorBin, kit_slot};
use crate::messages::{Code, tool_message};
use crate::paths;
use crate::plan::{EffectiveTeamKitFpc, TeamKitEdits, TeamKits};
use crate::processing::Entry;

/// What `patch_fpc` did to one kit slot's config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FpcPatch {
    /// The config lacked the FPC values and now carries them (`kit_config_fpc_adjusted`).
    Adjusted,
    /// The config already carried them: nothing changed.
    AlreadyFpc,
    /// The slot has no config, or its config does not decode as one: it is left alone
    /// (`kit_config_fpc_unpatched`).
    Unpatched,
}

/// Edits `bin`, the working `UniformParameter.bin`, for `team_kits` in order, then inserts
/// `committed_configs` (name, bytes), the committed kits' configs (`pipeline.md` "Bins
/// accumulation"). A `Full` export's team loses its configs of kits it does not hold
/// (`remove_other_configs`), and nothing else of it is edited, there being no absent kit. A
/// `Midcup` export edits the configs of its `absent_slots` (`edit_absent_slot`). Returns
/// whether the bin changed (a config inserted, patched or removed) and the findings.
pub(crate) fn kit_configs(
    bin: &mut UniformParameter,
    uni_color: &UniColorBin,
    team_kits: &[TeamKits],
    committed_configs: Vec<(String, Vec<u8>)>,
    version: PesVersion,
) -> anyhow::Result<(bool, Vec<Message>)> {
    let mut changed = false;
    let mut messages = Vec::new();
    for team in team_kits {
        match team.coverage {
            ExportCoverage::Full => {
                changed |= remove_other_configs(bin, team.team_id, &team.slots);
            }
            ExportCoverage::Midcup => {
                for slot in absent_slots(uni_color, team)? {
                    let name = slot.config_name(team.team_id);
                    let current = bin.get(&name);
                    if let Some(edited) =
                        edit_absent_slot(current, team, slot, version, &mut messages)
                    {
                        bin.insert(name, edited)?;
                        changed = true;
                    }
                }
            }
        }
    }
    changed |= !committed_configs.is_empty();
    for (name, config) in committed_configs {
        bin.insert(name, config)?;
    }
    Ok((changed, messages))
}

/// The PES 15-17 counterpart of `kit_configs` (`fpc_toggle.md` "Kit slots absent from the
/// export are patched in place": on pre-Fox the team's current kit-config bins are located in
/// the installed CPKs, patched, and re-emitted; `pipeline.md` "Bins accumulation";
/// `messages.md` `kit_config_fpc_adjusted`, `kit_config_fpc_unpatched`). Each kit config is a
/// loose file there, so for each `Midcup` export of `team_kits`, in order, the config of each
/// of its `absent_slots` is taken from `installed` (the installed loose configs by entry name,
/// `WorkingBins::loose_kit_configs`) and edited for `version` as on Fox (`edit_absent_slot`),
/// with the same findings. Returns the configs the edit changed, each at its CPK path
/// (`paths::kit_config`), for the output CPK to carry above the installed ones, and the
/// findings; a config the edit left as it was is not returned, the installed copy standing. A
/// `Full` export gives nothing: its own kits' configs are its kit tasks' entries already, and
/// it cannot remove the installed ones of kits it does not hold (`pipeline.md`: "What a
/// `Full` export cannot do is remove files").
pub(crate) fn loose_kit_configs(
    installed: &BTreeMap<String, Vec<u8>>,
    uni_color: &UniColorBin,
    team_kits: &[TeamKits],
    version: PesVersion,
) -> anyhow::Result<(Vec<Entry>, Vec<Message>)> {
    let mut configs = Vec::new();
    let mut messages = Vec::new();
    for team in team_kits {
        match team.coverage {
            ExportCoverage::Full => {}
            ExportCoverage::Midcup => {
                for slot in absent_slots(uni_color, team)? {
                    let name = slot.config_name(team.team_id);
                    let current = installed.get(&name).map(Vec::as_slice);
                    if let Some(edited) =
                        edit_absent_slot(current, team, slot, version, &mut messages)
                    {
                        configs.push((paths::kit_config(team.team_id, &name), edited));
                    }
                }
            }
        }
    }
    Ok((configs, messages))
}

/// The slots of the kits `team`'s record in `uni_color` holds, ascending, that its `Midcup`
/// export has no kit task for: the slots whose configs it edits (`pipeline.md` "Bins
/// accumulation"). A kit number no slot has, a second goalkeeper kit, is skipped. None when the
/// export has nothing to give (its FPC status Unknown and no collar), the record not being read
/// then.
fn absent_slots(uni_color: &UniColorBin, team: &TeamKits) -> anyhow::Result<Vec<KitSlot>> {
    match team.edits {
        TeamKitEdits {
            fpc: EffectiveTeamKitFpc::Unknown,
            collar: None,
        } => Ok(Vec::new()),
        TeamKitEdits { .. } => Ok(uni_color
            .kits(team.team_id)?
            .into_iter()
            .filter_map(kit_slot)
            .filter(|slot| !team.slots.contains(slot))
            .collect()),
    }
}

/// Edits `team`'s config of `slot`, a kit its `Midcup` export does not hold, for `version`;
/// `current` is the config as installed, `None` when the slot has none. When the team's kit-FPC
/// status is On, the config is given the FPC values (`patch_fpc`) and reported in `messages` on
/// the export, naming the slot, as `kit_config_fpc_adjusted` when it lacked them or
/// `kit_config_fpc_unpatched` when there is no config or it does not decode; then, when the
/// export has a collar, the config wears it (`wear_collar`), with no finding: the FPC finding
/// has said already when there is no config to edit. Returns the edited config when it
/// changed.
fn edit_absent_slot(
    current: Option<&[u8]>,
    team: &TeamKits,
    slot: KitSlot,
    version: PesVersion,
    messages: &mut Vec<Message>,
) -> Option<Vec<u8>> {
    let name = slot.config_name(team.team_id);
    // A copy of the 120 bytes, so the collar edit starts from the FPC edit's result.
    let mut config = current.map(<[u8]>::to_vec);
    let mut changed = false;
    let code = match team.edits.fpc {
        EffectiveTeamKitFpc::On => match patch_fpc(&mut config, &name, version) {
            FpcPatch::Adjusted => {
                changed = true;
                Some(Code::KitConfigFpcAdjusted)
            }
            FpcPatch::AlreadyFpc => None,
            FpcPatch::Unpatched => Some(Code::KitConfigFpcUnpatched),
        },
        EffectiveTeamKitFpc::Unknown => None,
    };
    if let Some(code) = code {
        messages.push(tool_message(
            code,
            Scope::Export {
                export_id: team.export_id,
            },
            Disposition::Keep,
            vec![("slot", slot.as_str().to_owned())],
        ));
    }
    if let Some(collar) = team.edits.collar {
        changed |= wear_collar(&mut config, &name, collar, version);
    }
    config.filter(|_| changed)
}

/// Removes from `bin` team `team_id`'s configs (the entries whose name starts with its
/// three-digit ID and `_`) that none of `slots` names. Returns whether one was removed.
fn remove_other_configs(bin: &mut UniformParameter, team_id: u16, slots: &[KitSlot]) -> bool {
    let prefix = format!("{team_id:03}_");
    let kept: Vec<String> = slots.iter().map(|slot| slot.config_name(team_id)).collect();
    let removed: Vec<String> = bin
        .entries()
        .map(|(name, _)| name)
        .filter(|name| name.starts_with(&prefix) && !kept.iter().any(|kept| kept == name))
        .map(str::to_owned)
        .collect();
    for name in &removed {
        bin.remove(name);
    }
    !removed.is_empty()
}

/// Gives the FPC values to `config`, the slot's config named `name`, when it lacks them
/// (`edit_config`).
fn patch_fpc(config: &mut Option<Vec<u8>>, name: &str, version: PesVersion) -> FpcPatch {
    let patched = edit_config(config, name, version, |config| {
        if matches_fpc(config) {
            return false;
        }
        apply_fpc(config);
        true
    });
    match patched {
        Some(true) => FpcPatch::Adjusted,
        Some(false) => FpcPatch::AlreadyFpc,
        None => FpcPatch::Unpatched,
    }
}

/// Sets `collar` as the collar and the winter collar of `config`, the slot's config named
/// `name` (`edit_config`). Returns whether the config changed: not when it wears the collar
/// already, is absent or does not decode.
fn wear_collar(config: &mut Option<Vec<u8>>, name: &str, collar: u8, version: PesVersion) -> bool {
    let worn = edit_config(config, name, version, |config| {
        if (config.shirt.collar, config.shirt.winter_collar) == (collar, collar) {
            return false;
        }
        config.shirt.collar = collar;
        config.shirt.winter_collar = collar;
        true
    });
    worn == Some(true)
}

/// Edits `config`, one kit slot's config bytes, named `name` (`None` when the slot has no
/// config), with `edit`, which returns whether it changed the config: decoded for `version`,
/// and, when changed, encoded again for `version`, its texture names kept, in place of the
/// bytes. `None` when there is no config or it does not decode as one, which is left alone;
/// else whether it changed.
fn edit_config(
    config: &mut Option<Vec<u8>>,
    name: &str,
    version: PesVersion,
    edit: impl FnOnce(&mut KitConfig) -> bool,
) -> Option<bool> {
    let bytes = config.as_deref()?;
    let mut decoded = match KitConfig::decode(bytes, version) {
        Ok(decoded) => decoded,
        Err(error) => {
            log::debug!("{name}: not a kit config, left alone: {error}");
            return None;
        }
    };
    if !edit(&mut decoded) {
        return Some(false);
    }
    *config = Some(decoded.encode(version).to_vec());
    Some(true)
}

#[cfg(test)]
mod tests {
    use kit_config::{TexturePresence, texture_names};
    use studio_core::ExportId;

    use super::*;
    use crate::bins::WorkingBins;
    use crate::templates::Templates;

    /// The export every `kit_configs` test reports on.
    const EXPORT: ExportId = ExportId(3);

    /// Team `team_id`'s config of `slot` holding shirt model 144, which lacks the FPC values,
    /// encoded for PES 21 with the slot's main texture name.
    fn shirt_144(team_id: u16, slot: KitSlot) -> Vec<u8> {
        let mut config = KitConfig::template();
        config.shirt.model = 144;
        assert!(!matches_fpc(&config));
        let names = texture_names(
            team_id,
            slot,
            TexturePresence {
                kit: true,
                ..TexturePresence::default()
            },
        );
        config.encode_with_names(PesVersion::Pes21, &names).to_vec()
    }

    /// A `UniformParameter.bin` holding `shirt_144` of each of `configs` (team ID, slot).
    fn installed(configs: &[(u16, KitSlot)]) -> UniformParameter {
        let mut bin = UniformParameter::new();
        for (team_id, slot) in configs {
            bin.insert(slot.config_name(*team_id), shirt_144(*team_id, *slot))
                .unwrap();
        }
        bin
    }

    /// The names of `bin`'s entries, in name order.
    fn names(bin: &UniformParameter) -> Vec<&str> {
        bin.entries().map(|(name, _)| name).collect()
    }

    /// Every entry of `bin`, by name, as it would be written.
    fn contents(bin: &UniformParameter) -> Vec<(String, Vec<u8>)> {
        bin.entries()
            .map(|(name, bytes)| (name.to_owned(), bytes.to_vec()))
            .collect()
    }

    /// The bundled `UniColor.bin` with team 714's record holding the kits `numbers` (icon 3,
    /// black) under the count `count`.
    fn uni_color_714(count: u8, numbers: &[u8]) -> UniColorBin {
        let mut record = vec![0xca, 0x02, 0x00, 0x00, count];
        for index in 0..10 {
            match numbers.get(index) {
                Some(number) => record.extend([*number, 3, 0, 0, 0, 0, 0, 0]),
                None => record.extend([0xff, 0, 0, 0, 0, 0, 0, 0]),
            }
        }
        let mut bytes = Templates::embedded().uni_color().to_vec();
        // Records are 85 bytes, team 100's first.
        let start = (714 - 100) * 85;
        bytes[start..start + 85].copy_from_slice(&record);
        UniColorBin::read(bytes).unwrap()
    }

    /// Team 714's export `EXPORT` of `coverage`, its status `fpc`, holding the kits `slots` and
    /// no collar.
    fn team_714(coverage: ExportCoverage, fpc: EffectiveTeamKitFpc, slots: &[KitSlot]) -> TeamKits {
        team_714_wearing(coverage, fpc, None, slots)
    }

    /// Team 714's export `EXPORT` of `coverage`, its status `fpc`, its collar `collar`, holding
    /// the kits `slots`.
    fn team_714_wearing(
        coverage: ExportCoverage,
        fpc: EffectiveTeamKitFpc,
        collar: Option<u8>,
        slots: &[KitSlot],
    ) -> TeamKits {
        TeamKits {
            export_id: EXPORT,
            team_id: 714,
            coverage,
            edits: TeamKitEdits { fpc, collar },
            slots: slots.to_vec(),
        }
    }

    /// The finding `code` on `EXPORT` naming `slot`.
    fn fpc_finding(code: Code, slot: &str) -> Message {
        tool_message(
            code,
            Scope::Export { export_id: EXPORT },
            Disposition::Keep,
            vec![("slot", slot.to_owned())],
        )
    }

    /// `kit_configs` on `bin` for PES 21 with no committed config.
    fn patched(
        bin: &mut UniformParameter,
        uni_color: &UniColorBin,
        team: TeamKits,
    ) -> (bool, Vec<Message>) {
        kit_configs(bin, uni_color, &[team], Vec::new(), PesVersion::Pes21).unwrap()
    }

    #[test]
    fn a_midcup_fpc_on_export_patches_the_record_s_kits_it_does_not_hold() {
        let mut bin = installed(&[(714, KitSlot::P1), (714, KitSlot::P2)]);
        bin.insert("714_DEF_3rd_realUni.bin".to_owned(), vec![0; 119])
            .unwrap();
        let before = contents(&bin);
        // 0x11, a second goalkeeper kit, has no slot.
        let uni_color = uni_color_714(4, &[0, 1, 2, 0x11]);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color,
            team_714(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::On,
                &[KitSlot::P2],
            ),
        );

        assert!(changed);
        assert_eq!(
            messages,
            [
                fpc_finding(Code::KitConfigFpcAdjusted, "p1"),
                fpc_finding(Code::KitConfigFpcUnpatched, "p3"),
            ]
        );
        assert_eq!(messages[1].severity, studio_core::Severity::Warning);
        let mut expected =
            KitConfig::decode(&shirt_144(714, KitSlot::P1), PesVersion::Pes21).unwrap();
        expected.shirt.model = 176;
        expected.shorts.model = 16;
        expected.shirt.collar = 105;
        expected.shirt.winter_collar = 105;
        let mut after = before;
        after[0].1 = expected.encode(PesVersion::Pes21).to_vec();
        assert_eq!(
            contents(&bin),
            after,
            "p1 patched, p2 (the export's) and p3 (unpatched) as they were"
        );
    }

    #[test]
    fn a_config_already_carrying_the_fpc_values_is_left_and_not_reported() {
        let mut config = KitConfig::template();
        assert!(matches_fpc(&config), "the template carries the FPC values");
        config.shirt.model = 176;
        let mut bin = UniformParameter::new();
        bin.insert(
            "714_DEF_1st_realUni.bin".to_owned(),
            config.encode(PesVersion::Pes21).to_vec(),
        )
        .unwrap();
        let before = contents(&bin);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color_714(1, &[0]),
            team_714(ExportCoverage::Midcup, EffectiveTeamKitFpc::On, &[]),
        );

        assert_eq!((changed, messages), (false, Vec::new()));
        assert_eq!(contents(&bin), before);
    }

    #[test]
    fn a_midcup_export_with_its_status_unknown_patches_nothing() {
        let mut bin = installed(&[(714, KitSlot::P1), (714, KitSlot::P2)]);
        let before = contents(&bin);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color_714(3, &[0, 1, 2]),
            team_714(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::Unknown,
                &[KitSlot::P2],
            ),
        );

        assert_eq!((changed, messages), (false, Vec::new()));
        assert_eq!(contents(&bin), before);
    }

    #[test]
    fn a_placeholder_record_holds_no_kit_to_patch() {
        let mut bin = installed(&[(714, KitSlot::P1)]);
        let before = contents(&bin);
        // Two entries numbered 0 under a count of 2: the base game's placeholder.
        let placeholder = uni_color_714(2, &[0, 0]);
        assert_eq!(placeholder.kits(714).unwrap(), Vec::<u8>::new());

        let (changed, messages) = patched(
            &mut bin,
            &placeholder,
            team_714(ExportCoverage::Midcup, EffectiveTeamKitFpc::On, &[]),
        );

        assert_eq!((changed, messages), (false, Vec::new()));
        assert_eq!(contents(&bin), before);
    }

    #[test]
    fn a_full_export_keeps_its_team_s_configs_of_its_kits_and_patches_nothing() {
        let mut bin = installed(&[
            (714, KitSlot::P1),
            (714, KitSlot::P2),
            (714, KitSlot::P3),
            (714, KitSlot::G1),
            (702, KitSlot::P1),
        ]);
        // g1's task failed: only p1's config committed.
        let committed = vec![("714_DEF_1st_realUni.bin".to_owned(), vec![7; 120])];

        let (changed, messages) = kit_configs(
            &mut bin,
            &uni_color_714(3, &[0, 1, 2]),
            &[team_714(
                ExportCoverage::Full,
                EffectiveTeamKitFpc::On,
                &[KitSlot::P1, KitSlot::G1],
            )],
            committed,
            PesVersion::Pes21,
        )
        .unwrap();

        assert!(changed);
        assert_eq!(messages, [], "no absent slot is patched");
        assert_eq!(
            contents(&bin),
            [
                (
                    "702_DEF_1st_realUni.bin".to_owned(),
                    shirt_144(702, KitSlot::P1)
                ),
                ("714_DEF_1st_realUni.bin".to_owned(), vec![7; 120]),
                (
                    "714_DEF_GK1st_realUni.bin".to_owned(),
                    shirt_144(714, KitSlot::G1)
                ),
            ],
            "p2 and p3 removed, the failed g1's installed config kept"
        );
    }

    #[test]
    fn a_midcup_export_s_collar_goes_into_the_record_s_kits_it_does_not_hold_and_nothing_else() {
        let mut bin = installed(&[(714, KitSlot::P1), (714, KitSlot::P2), (714, KitSlot::G1)]);
        bin.insert("714_DEF_4th_realUni.bin".to_owned(), vec![0; 119])
            .unwrap();
        let before = contents(&bin);
        // p3 has no config; p4's does not decode.
        let uni_color = uni_color_714(5, &[0, 1, 2, 3, 0x10]);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color,
            team_714_wearing(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::Unknown,
                Some(12),
                &[KitSlot::P2],
            ),
        );

        assert!(changed);
        assert_eq!(messages, [], "a collar reports nothing");
        let after = contents(&bin);
        assert_eq!(
            names(&bin),
            [
                "714_DEF_1st_realUni.bin",
                "714_DEF_2nd_realUni.bin",
                "714_DEF_4th_realUni.bin",
                "714_DEF_GK1st_realUni.bin",
            ],
            "no config added"
        );
        // p1 and g1 wear the collar, their bytes otherwise as they were; p2 (the export's)
        // and p4 (no config) as they were.
        for ((name, ours), (_, theirs)) in after.iter().zip(&before) {
            let differing: Vec<usize> = (0..theirs.len())
                .filter(|offset| ours.get(*offset) != theirs.get(*offset))
                .collect();
            let wears = ["714_DEF_1st_realUni.bin", "714_DEF_GK1st_realUni.bin"];
            if wears.contains(&name.as_str()) {
                assert_eq!(ours.len(), theirs.len(), "{name}");
                assert_eq!(differing, [0x14, 0x15], "{name}");
                assert_eq!((ours[0x14], ours[0x15]), (12, 12), "{name}");
            } else {
                assert_eq!(ours, theirs, "{name}");
            }
        }
    }

    #[test]
    fn with_fpc_on_an_absent_slot_gets_the_fpc_values_then_the_collar() {
        let mut bin = installed(&[(714, KitSlot::P1)]);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color_714(2, &[0, 1]),
            team_714_wearing(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::On,
                Some(12),
                &[],
            ),
        );

        assert!(changed);
        assert_eq!(
            messages,
            [
                fpc_finding(Code::KitConfigFpcAdjusted, "p1"),
                fpc_finding(Code::KitConfigFpcUnpatched, "p2"),
            ]
        );
        let mut expected =
            KitConfig::decode(&shirt_144(714, KitSlot::P1), PesVersion::Pes21).unwrap();
        expected.shirt.model = 176;
        expected.shorts.model = 16;
        expected.shirt.collar = 12;
        expected.shirt.winter_collar = 12;
        assert_eq!(
            bin.get("714_DEF_1st_realUni.bin"),
            Some(&expected.encode(PesVersion::Pes21)[..])
        );
    }

    #[test]
    fn a_config_wearing_the_collar_already_is_left_and_the_bin_unchanged() {
        let mut config = KitConfig::template();
        config.shirt.collar = 12;
        config.shirt.winter_collar = 12;
        let mut bin = UniformParameter::new();
        bin.insert(
            "714_DEF_1st_realUni.bin".to_owned(),
            config.encode(PesVersion::Pes21).to_vec(),
        )
        .unwrap();
        let before = contents(&bin);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color_714(1, &[0]),
            team_714_wearing(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::Unknown,
                Some(12),
                &[],
            ),
        );

        assert_eq!((changed, messages), (false, Vec::new()));
        assert_eq!(contents(&bin), before);
    }

    #[test]
    fn a_full_export_s_collar_patches_nothing() {
        let mut bin = installed(&[(714, KitSlot::P1), (714, KitSlot::P2)]);

        let (changed, messages) = patched(
            &mut bin,
            &uni_color_714(2, &[0, 1]),
            team_714_wearing(
                ExportCoverage::Full,
                EffectiveTeamKitFpc::On,
                Some(12),
                &[KitSlot::P1],
            ),
        );

        assert!(changed, "p2's config removed");
        assert_eq!(messages, []);
        assert_eq!(
            contents(&bin),
            [(
                "714_DEF_1st_realUni.bin".to_owned(),
                shirt_144(714, KitSlot::P1)
            )],
            "p1's installed config as it was, its task's config not committed here"
        );
    }

    #[test]
    fn a_full_export_whose_every_kit_failed_still_removes_its_team_s_other_configs() {
        let mut bin = installed(&[(714, KitSlot::P1), (714, KitSlot::P2)]);

        // p1's task failed: no config committed, only the removal changes the bin.
        let (changed, messages) = patched(
            &mut bin,
            &uni_color_714(2, &[0, 1]),
            team_714(
                ExportCoverage::Full,
                EffectiveTeamKitFpc::Unknown,
                &[KitSlot::P1],
            ),
        );

        assert!(changed, "a removed config changes the bin");
        assert_eq!(messages, []);
        assert_eq!(
            contents(&bin),
            [(
                "714_DEF_1st_realUni.bin".to_owned(),
                shirt_144(714, KitSlot::P1)
            )],
            "p2 removed, the failed p1's installed config kept"
        );
    }

    #[test]
    fn the_bundled_base_s_team_714_configs_carry_the_fpc_values() {
        // What a from-scratch `Midcup` `fpc_on` compile of `/co/` patches: nothing, its eight
        // kits' base configs carrying the values already.
        let bins = WorkingBins::bundled(PesVersion::Pes21, &Templates::embedded());
        let mut bin = bins.uniform_parameter.unwrap();
        let (changed, messages) = patched(
            &mut bin,
            &bins.uni_color,
            team_714(ExportCoverage::Midcup, EffectiveTeamKitFpc::On, &[]),
        );
        assert_eq!((changed, messages), (false, Vec::new()));
    }

    #[test]
    fn the_team_s_configs_no_slot_names_are_removed_and_another_team_s_kept() {
        let mut bin = installed(&[
            (714, KitSlot::P1),
            (714, KitSlot::P2),
            (714, KitSlot::P3),
            (714, KitSlot::G1),
            (702, KitSlot::P1),
        ]);

        assert!(remove_other_configs(
            &mut bin,
            714,
            &[KitSlot::P1, KitSlot::G1]
        ));

        assert_eq!(
            names(&bin),
            [
                "702_DEF_1st_realUni.bin",
                "714_DEF_1st_realUni.bin",
                "714_DEF_GK1st_realUni.bin"
            ]
        );
        assert_eq!(
            bin.get("714_DEF_GK1st_realUni.bin"),
            Some(&shirt_144(714, KitSlot::G1)[..]),
            "a kept config keeps its bytes"
        );
    }

    #[test]
    fn removing_nothing_says_so() {
        let mut bin = installed(&[(714, KitSlot::P1), (702, KitSlot::P2)]);
        assert!(!remove_other_configs(&mut bin, 714, &[KitSlot::P1]));
        assert_eq!(
            names(&bin),
            ["702_DEF_2nd_realUni.bin", "714_DEF_1st_realUni.bin"]
        );
    }

    #[test]
    fn a_config_lacking_the_fpc_values_gets_them_and_keeps_every_other_byte() {
        let mut config = Some(shirt_144(714, KitSlot::P1));

        let patched = patch_fpc(&mut config, "714_DEF_1st_realUni.bin", PesVersion::Pes21);

        assert_eq!(patched, FpcPatch::Adjusted);
        let installed = KitConfig::decode(&shirt_144(714, KitSlot::P1), PesVersion::Pes21).unwrap();
        let mut expected = installed.clone();
        expected.shirt.model = 176;
        expected.shorts.model = 16;
        expected.shirt.collar = 105;
        expected.shirt.winter_collar = 105;
        assert_eq!(
            config.as_deref(),
            Some(&expected.encode(PesVersion::Pes21)[..])
        );
        // Its five texture names, from 0x28 on, are the installed config's.
        assert_eq!(
            config.as_deref().unwrap()[0x28..],
            shirt_144(714, KitSlot::P1)[0x28..]
        );
        assert_eq!(&shirt_144(714, KitSlot::P1)[0x28..0x2f], b"u0714p1");
    }

    #[test]
    fn a_config_carrying_the_fpc_values_is_left_as_it_is() {
        let mut config = KitConfig::template();
        assert!(matches_fpc(&config), "the template carries the FPC values");
        config.shirt.model = 176;
        let bytes = config.encode(PesVersion::Pes21).to_vec();
        let mut config = Some(bytes.clone());

        let patched = patch_fpc(&mut config, "714_DEF_2nd_realUni.bin", PesVersion::Pes21);

        assert_eq!(patched, FpcPatch::AlreadyFpc);
        assert_eq!(config.as_deref(), Some(&bytes[..]));
    }

    #[test]
    fn a_slot_with_no_config_or_one_that_does_not_decode_is_unpatched() {
        let mut absent = None;
        let mut undecodable = Some(vec![0; 119]);

        for config in [&mut absent, &mut undecodable] {
            let patched = patch_fpc(config, "714_DEF_3rd_realUni.bin", PesVersion::Pes21);
            assert_eq!(patched, FpcPatch::Unpatched, "{config:?}");
        }

        assert_eq!(undecodable.as_deref(), Some(&[0; 119][..]));
        assert_eq!(absent, None, "no config added");
    }

    /// The installed loose configs of PES 15-17 holding `shirt_144` of each of `configs` (team
    /// ID, slot), by entry name.
    fn installed_loose(configs: &[(u16, KitSlot)]) -> BTreeMap<String, Vec<u8>> {
        configs
            .iter()
            .map(|(team_id, slot)| (slot.config_name(*team_id), shirt_144(*team_id, *slot)))
            .collect()
    }

    /// The CPK path of team 714's loose p1 config.
    const P1_LOOSE: &str =
        "common/character0/model/character/uniform/team/714/714_DEF_1st_realUni.bin";

    /// `loose_kit_configs` on `installed` for PES 17.
    fn loose(
        installed: &BTreeMap<String, Vec<u8>>,
        uni_color: &UniColorBin,
        team: TeamKits,
    ) -> (Vec<Entry>, Vec<Message>) {
        loose_kit_configs(installed, uni_color, &[team], PesVersion::Pes17).unwrap()
    }

    #[test]
    fn a_midcup_fpc_on_export_re_emits_the_installed_loose_configs_it_patches() {
        // p2's config is the export's; p3 has none.
        let installed = installed_loose(&[(714, KitSlot::P1), (714, KitSlot::P2)]);

        let (configs, messages) = loose(
            &installed,
            &uni_color_714(3, &[0, 1, 2]),
            team_714(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::On,
                &[KitSlot::P2],
            ),
        );

        let mut expected =
            KitConfig::decode(&shirt_144(714, KitSlot::P1), PesVersion::Pes17).unwrap();
        apply_fpc(&mut expected);
        assert_eq!(
            configs,
            [(
                P1_LOOSE.to_owned(),
                expected.encode(PesVersion::Pes17).to_vec()
            )],
            "p1 patched, nothing else re-emitted"
        );
        assert_eq!(
            messages,
            [
                fpc_finding(Code::KitConfigFpcAdjusted, "p1"),
                fpc_finding(Code::KitConfigFpcUnpatched, "p3"),
            ]
        );
    }

    #[test]
    fn a_midcup_export_s_collar_alone_re_emits_the_loose_configs_wearing_it_silently() {
        let installed = installed_loose(&[(714, KitSlot::P1)]);

        let (configs, messages) = loose(
            &installed,
            &uni_color_714(3, &[0, 1, 2]),
            team_714_wearing(
                ExportCoverage::Midcup,
                EffectiveTeamKitFpc::Unknown,
                Some(12),
                &[KitSlot::P2],
            ),
        );

        let mut expected =
            KitConfig::decode(&shirt_144(714, KitSlot::P1), PesVersion::Pes17).unwrap();
        expected.shirt.collar = 12;
        expected.shirt.winter_collar = 12;
        assert_eq!(
            configs,
            [(
                P1_LOOSE.to_owned(),
                expected.encode(PesVersion::Pes17).to_vec()
            )],
            "p1 wears the collar; p3 has no config to wear it"
        );
        assert_eq!(messages, [], "a collar reports nothing");
    }

    #[test]
    fn a_full_export_re_emits_no_loose_config() {
        let installed = installed_loose(&[(714, KitSlot::P1), (714, KitSlot::P3)]);

        let (configs, messages) = loose(
            &installed,
            &uni_color_714(3, &[0, 1, 2]),
            team_714_wearing(
                ExportCoverage::Full,
                EffectiveTeamKitFpc::On,
                Some(12),
                &[KitSlot::P2],
            ),
        );

        assert_eq!((configs, messages), (Vec::new(), Vec::new()));
    }
}
