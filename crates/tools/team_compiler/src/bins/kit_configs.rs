//! A team's kit configs in `UniformParameter.bin` as a compile edits them besides adding its
//! committed kits' configs (`team_compiler/pipeline.md` "Bins accumulation"): a `Full`
//! export's team loses the configs of kits it does not hold, and a `Midcup` export whose team's
//! kit-FPC status is On gives the FPC values to the configs of the kits the game offers the
//! team that the export does not hold. A team's configs are the entries named for its team ID
//! (`KitSlot::config_name`: `714_DEF_1st_realUni.bin`).

use aesthetics_export::ExportCoverage;
use kit_config::{KitConfig, KitSlot, apply_fpc, matches_fpc};
use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};
use uniparam::UniformParameter;

use super::{UniColorBin, kit_slot};
use crate::messages::{Code, tool_message};
use crate::plan::{EffectiveTeamKitFpc, TeamKits};

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
/// (`remove_other_configs`). A `Midcup` export whose team's kit-FPC status is On has the
/// configs of the kits its team's record in `uni_color` holds, ascending, that it has no kit
/// task for, given the FPC values (`patch_fpc`), each reported on the export, naming the slot:
/// `kit_config_fpc_adjusted` when it lacked them, `kit_config_fpc_unpatched` when there is no
/// config or it does not decode; a kit number no slot has, a second goalkeeper kit, is skipped.
/// Returns whether the bin changed (a config inserted, patched or removed) and the findings.
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
        match (team.coverage, team.fpc) {
            // No absent slot is patched: a `Full` export's team has no kit but its own.
            (ExportCoverage::Full, EffectiveTeamKitFpc::On | EffectiveTeamKitFpc::Unknown) => {
                changed |= remove_other_configs(bin, team.team_id, &team.slots);
            }
            (ExportCoverage::Midcup, EffectiveTeamKitFpc::On) => {
                let absent = uni_color
                    .kits(team.team_id)?
                    .into_iter()
                    .filter_map(kit_slot)
                    .filter(|slot| !team.slots.contains(slot));
                for slot in absent {
                    let code = match patch_fpc(bin, team.team_id, slot, version)? {
                        FpcPatch::Adjusted => {
                            changed = true;
                            Code::KitConfigFpcAdjusted
                        }
                        FpcPatch::AlreadyFpc => continue,
                        FpcPatch::Unpatched => Code::KitConfigFpcUnpatched,
                    };
                    messages.push(tool_message(
                        code,
                        Scope::Export {
                            export_id: team.export_id,
                        },
                        Disposition::Keep,
                        vec![("slot", slot.as_str().to_owned())],
                    ));
                }
            }
            (ExportCoverage::Midcup, EffectiveTeamKitFpc::Unknown) => {}
        }
    }
    changed |= !committed_configs.is_empty();
    for (name, config) in committed_configs {
        bin.insert(name, config)?;
    }
    Ok((changed, messages))
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

/// Gives the FPC values to team `team_id`'s config of `slot` in `bin`, decoded and encoded
/// again for `version`, its texture names kept, when it lacks them.
fn patch_fpc(
    bin: &mut UniformParameter,
    team_id: u16,
    slot: KitSlot,
    version: PesVersion,
) -> anyhow::Result<FpcPatch> {
    let name = slot.config_name(team_id);
    let Some(bytes) = bin.get(&name) else {
        return Ok(FpcPatch::Unpatched);
    };
    let mut config = match KitConfig::decode(bytes, version) {
        Ok(config) => config,
        Err(error) => {
            log::debug!("{name}: not a kit config, left alone: {error}");
            return Ok(FpcPatch::Unpatched);
        }
    };
    if matches_fpc(&config) {
        return Ok(FpcPatch::AlreadyFpc);
    }
    apply_fpc(&mut config);
    bin.insert(name, config.encode(version).to_vec())?;
    Ok(FpcPatch::Adjusted)
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

    /// Team 714's export `EXPORT` of `coverage`, its status `fpc`, holding the kits `slots`.
    fn team_714(coverage: ExportCoverage, fpc: EffectiveTeamKitFpc, slots: &[KitSlot]) -> TeamKits {
        TeamKits {
            export_id: EXPORT,
            team_id: 714,
            coverage,
            fpc,
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
        let mut bin = installed(&[(714, KitSlot::P1)]);

        let patched = patch_fpc(&mut bin, 714, KitSlot::P1, PesVersion::Pes21).unwrap();

        assert_eq!(patched, FpcPatch::Adjusted);
        let installed = KitConfig::decode(&shirt_144(714, KitSlot::P1), PesVersion::Pes21).unwrap();
        let mut expected = installed.clone();
        expected.shirt.model = 176;
        expected.shorts.model = 16;
        expected.shirt.collar = 105;
        expected.shirt.winter_collar = 105;
        assert_eq!(
            bin.get("714_DEF_1st_realUni.bin"),
            Some(&expected.encode(PesVersion::Pes21)[..])
        );
        // Its five texture names, from 0x28 on, are the installed config's.
        assert_eq!(
            bin.get("714_DEF_1st_realUni.bin").unwrap()[0x28..],
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
        let mut bin = UniformParameter::new();
        bin.insert("714_DEF_2nd_realUni.bin".to_owned(), bytes.clone())
            .unwrap();

        let patched = patch_fpc(&mut bin, 714, KitSlot::P2, PesVersion::Pes21).unwrap();

        assert_eq!(patched, FpcPatch::AlreadyFpc);
        assert_eq!(bin.get("714_DEF_2nd_realUni.bin"), Some(&bytes[..]));
    }

    #[test]
    fn a_slot_with_no_config_or_one_that_does_not_decode_is_unpatched() {
        let mut bin = installed(&[(702, KitSlot::P3)]);
        bin.insert("714_DEF_3rd_realUni.bin".to_owned(), vec![0; 119])
            .unwrap();

        for slot in [KitSlot::P2, KitSlot::P3] {
            let patched = patch_fpc(&mut bin, 714, slot, PesVersion::Pes21).unwrap();
            assert_eq!(patched, FpcPatch::Unpatched, "{slot:?}");
        }

        assert_eq!(bin.get("714_DEF_3rd_realUni.bin"), Some(&[0; 119][..]));
        assert_eq!(
            names(&bin),
            ["702_DEF_3rd_realUni.bin", "714_DEF_3rd_realUni.bin"],
            "no config added"
        );
    }
}
