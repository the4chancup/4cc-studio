//! The "Advanced instructions" table (`plans/save_editor.md` "Tactics
//! editor", "What the controls write", the instruction bullet): the four
//! rows Attack 1, Attack 2, Defence 1, Defence 2, each its instruction and
//! its target — the version's stored list less Tight Marking, and a target
//! pick only for the instructions that take one of the team's own players.

use pes_savefile::model::instruction::Instruction;
use pes_savefile::model::tactics::{AdvancedInstruction, TacticsPreset};
use pes_savefile::schema::fields::{InstructionPart, InstructionSide, TacticsField};
use pes_savefile::schema::instruction;
use pes_version::PesVersion;

use super::{RosterRow, gate_tip, name_at};

/// The stored "no player" value of an instruction's target.
const NO_TARGET: u8 = 0xFF;

/// The "Advanced instructions" group (PES 17+): the four rows Attack 1,
/// Attack 2, Defence 1, Defence 2, each its instruction and its target.
/// A version with no instruction table keeps the group, disabled, with
/// the `PES NN+` tooltip naming the first version that has it.
pub(super) fn instruction_table(
    ui: &mut egui::Ui,
    version: PesVersion,
    preset: &mut TacticsPreset,
    starting_eleven: &[u8; 11],
    roster: &[RosterRow<'_>],
) {
    ui.strong("Advanced instructions");
    let (Some(attack), Some(defence)) = (
        preset.attack_instructions.as_mut(),
        preset.defence_instructions.as_mut(),
    ) else {
        ui.add_enabled_ui(false, |ui| {
            ui.weak("(none on this version)");
        })
        .response
        .on_disabled_hover_text(gate_tip(TacticsField::Instruction {
            preset: 0,
            side: InstructionSide::Attack,
            index: 0,
            part: InstructionPart::Instruction,
        }));
        return;
    };
    egui::Grid::new("advanced instructions")
        .num_columns(3)
        .show(ui, |ui| {
            ui.weak("");
            ui.weak("Instruction");
            ui.weak("Target");
            ui.end_row();
            instruction_row(
                ui,
                version,
                "attack 1",
                "Attack 1",
                &mut attack[0],
                starting_eleven,
                roster,
            );
            ui.end_row();
            instruction_row(
                ui,
                version,
                "attack 2",
                "Attack 2",
                &mut attack[1],
                starting_eleven,
                roster,
            );
            ui.end_row();
            instruction_row(
                ui,
                version,
                "defence 1",
                "Defence 1",
                &mut defence[0],
                starting_eleven,
                roster,
            );
            ui.end_row();
            instruction_row(
                ui,
                version,
                "defence 2",
                "Defence 2",
                &mut defence[1],
                starting_eleven,
                roster,
            );
            ui.end_row();
        });
}

/// One instruction row: the instruction's combo (the version's list in
/// its stored order less Tight Marking, which marks an opponent and is
/// set in a match, not here; a stored byte the table does not list shows
/// its hex and is kept until a pick), then the target's, enabled only
/// for an instruction that takes one of the team's own players
/// (`Instruction::takes_player`, the one place the rule lives). Picking
/// an instruction that takes no target clears the pick.
fn instruction_row(
    ui: &mut egui::Ui,
    version: PesVersion,
    id: &str,
    label: &str,
    entry: &mut AdvancedInstruction,
    starting_eleven: &[u8; 11],
    roster: &[RosterRow<'_>],
) {
    ui.label(label);
    let current = instruction::decode(version, entry.instruction)
        .ok()
        .flatten();
    let selected = match current {
        Some(inst) => instruction_label(inst).to_owned(),
        None => format!("Unknown (0x{:02X})", entry.instruction),
    };
    let mut pick = current;
    egui::ComboBox::from_id_salt((id, "instruction"))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            for inst in instruction::instructions(version)
                .iter()
                .copied()
                .filter(|inst| *inst != Instruction::TightMarking)
            {
                ui.selectable_value(&mut pick, Some(inst), instruction_label(inst));
            }
        });
    if pick != current
        && let Some(inst) = pick
    {
        entry.instruction =
            instruction::encode(version, inst).expect("the offered list is the version's own");
        if !inst.takes_player() {
            entry.player = NO_TARGET;
        }
    }
    let mut target = entry.player;
    let selected = if target == NO_TARGET {
        "Unassigned".to_owned()
    } else {
        match starting_eleven.iter().position(|slot| *slot == target) {
            Some(index) => format!("{} {}", index + 1, name_at(roster, target)),
            None => name_at(roster, target).to_owned(),
        }
    };
    ui.add_enabled_ui(current.is_some_and(|inst| inst.takes_player()), |ui| {
        egui::ComboBox::from_id_salt((id, "target"))
            .selected_text(selected)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut target, NO_TARGET, "Unassigned");
                for (index, slot) in starting_eleven.iter().enumerate() {
                    ui.selectable_value(
                        &mut target,
                        *slot,
                        format!("{} {}", index + 1, name_at(roster, *slot)),
                    );
                }
            });
    });
    entry.player = target;
}

/// The instruction's text in the combo and its rows (the draft's
/// `INSTRUCTION_LABEL`; `Off` reads "Unassigned").
fn instruction_label(instruction: Instruction) -> &'static str {
    match instruction {
        Instruction::Off => "Unassigned",
        Instruction::HugTheTouchline => "Hug the Touchline",
        Instruction::FalseNo9 => "False No. 9",
        Instruction::FalseFullBacks => "False Full-Backs",
        Instruction::AttackingFullBacks => "Attacking Full-Backs",
        Instruction::WingRotation => "Wing Rotation",
        Instruction::TikiTaka => "Tiki-Taka",
        Instruction::CenteringTargets => "Centering Targets",
        Instruction::SwarmTheBox => "Swarm the Box",
        Instruction::DeepDefensiveLine => "Deep Defensive Line",
        Instruction::Gegenpress => "Gegenpress",
        Instruction::TightMarking => "Tight Marking",
        Instruction::CounterTarget => "Counter Target",
        Instruction::Defensive => "Defensive",
        Instruction::FalseWinger => "False Winger",
        Instruction::WingBack => "Wing-Back",
        Instruction::Anchoring => "Anchoring",
    }
}

#[cfg(test)]
mod tests {
    use egui::accesskit::Role;

    use super::*;
    use crate::tactics::TacticsView;
    use crate::tactics::tests::click;
    use crate::test_support::{
        Headless, button_labels, control_right_of, controls_right_of, entry_id, fixture_rows,
        fixture_tactics, roster_rows,
    };

    // plans/save_editor.md "Tactics editor", "What the controls write": the
    // instruction list is the version's stored table in its stored order,
    // less Tight Marking (set in a match, not here).
    #[test]
    fn the_instruction_combo_offers_the_versions_list_less_tight_marking() {
        let mut window = Headless::sized(800.0, 900.0);
        for (version, count) in [
            (PesVersion::Pes17, 12),
            (PesVersion::Pes18, 15),
            (PesVersion::Pes21, 16),
        ] {
            let mut tactics = fixture_tactics(version);
            let mut view = TacticsView::default();
            let rows = fixture_rows(version);
            let roster = roster_rows(&rows);
            let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
            let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
            let (nodes, _) = click(
                &mut window,
                combo,
                version,
                &mut tactics,
                &mut view,
                &roster,
            );
            let offered = instruction::instructions(version)
                .iter()
                .copied()
                .filter(|inst| *inst != Instruction::TightMarking)
                .map(instruction_label)
                .collect::<Vec<_>>();
            let labels: std::collections::BTreeSet<&str> = offered.iter().copied().collect();
            let drawn = button_labels(&nodes, |label| labels.contains(label));
            assert_eq!(drawn.len(), count, "{version:?}'s item count");
            assert!(
                !drawn.iter().any(|label| label == "Tight Marking"),
                "{version:?} does not offer Tight Marking"
            );
            assert_eq!(drawn, offered, "{version:?}'s items in stored order");
            // Close the combo again for the next version's frame.
            let _ = click(
                &mut window,
                combo,
                version,
                &mut tactics,
                &mut view,
                &roster,
            );
        }
    }

    // The same ruling's kept half: a save already holding Tight Marking
    // shows it as the row's value, and drawing keeps it.
    #[test]
    fn a_stored_tight_marking_shows_its_label_and_stays() {
        let mut window = Headless::sized(800.0, 900.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0]
            .instruction = instruction::encode(PesVersion::Pes21, Instruction::TightMarking)
            .expect("PES 21 stores Tight Marking");
        let before = tactics.clone();
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);

        let (nodes, changed) =
            window.tactics(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
        assert_eq!(
            combo.value(),
            Some("Tight Marking"),
            "the stored Tight Marking shows as the combo's text"
        );
        assert!(!changed, "the frame reports no change");
        let (_, changed) =
            window.tactics(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        assert!(!changed, "an idle frame reports no change");
        assert_eq!(tactics, before, "and leaves the record as stored");
    }

    // A stored byte the version's table does not list shows its hex and
    // is kept until a pick.
    #[test]
    fn an_unknown_stored_instruction_shows_its_hex() {
        let mut window = Headless::sized(800.0, 900.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0]
            .instruction = 0xAB;
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let (nodes, _) =
            window.tactics(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
        assert_eq!(combo.value(), Some("Unknown (0xAB)"));
    }

    // The pick itself: Counter Target enables the target, which writes the
    // picked starter's roster slot; an instruction that takes no target
    // writes the byte and clears the pick to 255, its combo disabled.
    #[test]
    fn the_instruction_and_target_picks_write_the_stored_values() {
        let mut window = Headless::sized(800.0, 900.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);

        // Open the instruction combo and pick Counter Target.
        let (_, _) = click(
            &mut window,
            combo,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let item = entry_id(&ids, Role::Button, "Counter Target");
        let (_, changed) =
            window.action_click_tactics(item, version, &mut tactics, &mut view, &roster);
        assert!(changed, "the pick changed the record");
        let attack = &mut tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("instructions")[0];
        assert_eq!(
            attack.instruction,
            instruction::encode(version, Instruction::CounterTarget)
                .expect("PES 21 stores Counter Target"),
            "the pick writes the stored value"
        );

        // The target combo is enabled and lists the starters; pick the
        // third: its roster slot is written.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let target = controls_right_of(&nodes, "Attack 1", Role::ComboBox)
            .into_iter()
            .nth(1)
            .expect("the target combo");
        assert!(!target.is_disabled(), "Counter Target enables the target");
        let (_, _) = click(
            &mut window,
            target,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        // One idle frame: the instruction popup draws a last time on the
        // frame that closed it, so the items below are the target's alone.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let starters: Vec<String> = (0..11)
            .map(|index| {
                format!(
                    "{} {}",
                    index + 1,
                    name_at(&roster, tactics.starting_eleven[index])
                )
            })
            .collect();
        let drawn = button_labels(&nodes, |label| {
            label == "Unassigned" || starters.iter().any(|s| s == label)
        });
        assert_eq!(
            drawn.len(),
            12,
            "Unassigned plus the eleven starters, got {drawn:?}"
        );
        assert_eq!(drawn[0], "Unassigned");
        assert_eq!(drawn[1..], starters[..], "the starters in lineup order");
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let third = entry_id(&ids, Role::Button, &starters[2]);
        let (_, changed) =
            window.action_click_tactics(third, version, &mut tactics, &mut view, &roster);
        assert!(changed, "the target pick changed the record");
        assert_eq!(
            tactics.presets[0]
                .attack_instructions
                .as_ref()
                .expect("instructions")[0]
                .player,
            tactics.starting_eleven[2],
            "the third starter's roster slot is written"
        );

        // Pick Gegenpress: the byte changes, the target clears to 255 and
        // its combo disables.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
        let (_, _) = click(
            &mut window,
            combo,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let item = entry_id(&ids, Role::Button, "Gegenpress");
        let (_, changed) =
            window.action_click_tactics(item, version, &mut tactics, &mut view, &roster);
        assert!(changed, "the pick changed the record");
        let attack = &tactics.presets[0]
            .attack_instructions
            .as_ref()
            .expect("instructions")[0];
        assert_eq!(
            attack.instruction,
            instruction::encode(version, Instruction::Gegenpress)
                .expect("PES 21 stores Gegenpress")
        );
        assert_eq!(
            attack.player, NO_TARGET,
            "a target-less pick clears the target"
        );
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let target = controls_right_of(&nodes, "Attack 1", Role::ComboBox)
            .into_iter()
            .nth(1)
            .expect("the target combo");
        assert!(target.is_disabled(), "Gegenpress disables the target combo");
        assert_eq!(
            target.value(),
            Some("Unassigned"),
            "and it shows Unassigned"
        );
    }

    // The Target combo's shown text: a starter as his lineup position and
    // name, a bench player as the name alone (`instruction_row`'s
    // `starting_eleven.position`).
    #[test]
    fn the_target_combo_shows_the_starters_position_or_the_name_alone() {
        let mut window = Headless::sized(800.0, 900.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let starter = tactics.starting_eleven[3];
        let bench = tactics.bench_order[0];
        assert_ne!(starter, bench, "premise: two different slots");
        let attack = &mut tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0];
        attack.instruction =
            instruction::encode(version, Instruction::CounterTarget).expect("stored");
        attack.player = starter;
        let mut view = TacticsView::default();
        // A name per slot: the fixture's are all "PLACEHOLDER" here.
        let rows: Vec<(String, Option<u8>)> = (0..32)
            .map(|slot| (format!("Player {slot}"), Some(0)))
            .collect();
        let roster = roster_rows(&rows);

        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let target = controls_right_of(&nodes, "Attack 1", Role::ComboBox)
            .into_iter()
            .nth(1)
            .expect("the target combo");
        let expected = format!("4 Player {starter}");
        assert_eq!(
            target.value(),
            Some(expected.as_str()),
            "a starter shows his lineup position and name"
        );

        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0]
            .player = bench;
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let target = controls_right_of(&nodes, "Attack 1", Role::ComboBox)
            .into_iter()
            .nth(1)
            .expect("the target combo");
        let expected = format!("Player {bench}");
        assert_eq!(
            target.value(),
            Some(expected.as_str()),
            "a bench player shows the name alone"
        );
    }

    // `roster` shorter than a slot, or an empty one, shows "(empty)" in
    // the target items.
    #[test]
    fn a_short_or_empty_roster_slice_shows_empty() {
        let mut window = Headless::sized(800.0, 900.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0]
            .instruction =
            instruction::encode(version, Instruction::CounterTarget).expect("stored");
        let mut view = TacticsView::default();
        // Three rows for forty roster slots: slots past the slice, and
        // an empty one, show "(empty)".
        let rows = [
            ("ALPHA".to_owned(), Some(0)),
            ("BETA".to_owned(), Some(0)),
            (String::new(), None),
        ];
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let target = controls_right_of(&nodes, "Attack 1", Role::ComboBox)
            .into_iter()
            .nth(1)
            .expect("the target combo");
        let (_, _) = click(
            &mut window,
            target,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let starters: Vec<String> = (0..11)
            .map(|index| {
                format!(
                    "{} {}",
                    index + 1,
                    name_at(&roster, tactics.starting_eleven[index])
                )
            })
            .collect();
        assert_eq!(starters[0], "1 ALPHA");
        assert_eq!(starters[1], "2 BETA");
        assert_eq!(starters[2], "3 (empty)", "an empty name shows (empty)");
        for label in &starters[3..] {
            assert!(
                label.ends_with("(empty)"),
                "{label}: a short slice shows (empty)"
            );
        }
        let drawn = button_labels(&nodes, |label| {
            label == "Unassigned" || starters.iter().any(|s| s == label)
        });
        assert_eq!(drawn.len(), 12, "Unassigned plus the eleven starters");
        assert_eq!(drawn[1..], starters[..]);
    }
}
