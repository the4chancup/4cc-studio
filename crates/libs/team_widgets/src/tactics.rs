//! The tactics settings of `plans/save_editor.md` "Tactics editor" (the
//! draft's `tacticsTab`, minus the pitch canvas, the lineup and the role
//! palette, which land with the pitch slice): the preset selector, the
//! Formation settings group, the Advanced instructions table and the
//! Automatic settings group. `tactics_settings` edits `tactics` in place
//! and answers whether it changed; the host decides what the change means
//! (the contract in `plans/save_editor.md` "`libs/team_widgets`").

use pes_savefile::model::instruction::Instruction;
use pes_savefile::model::tactics::{
    AdvancedInstruction, TacticSliders, TacticStyle, TacticsPreset, TeamAutoFlags, TeamTactics,
};
use pes_savefile::schema::fields::{InstructionPart, InstructionSide, PresetField, TacticsField};
use pes_savefile::schema::{instruction, schema_for};
use pes_version::PesVersion;

/// The stored "no player" value of an instruction's target.
const NO_TARGET: u8 = 0xFF;

/// What the tactics editor shows that the record does not hold: the
/// preset on screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TacticsView {
    /// The preset the controls edit, 0 to 2, shown 1 to 3.
    pub preset: usize,
}

/// One of the five sliders (the draft's `SLIDERS`): its label, its range
/// top and the `TacticSliders` field it edits.
struct SliderSpec {
    /// The cell's label.
    label: &'static str,
    /// The range's top (1 is always the bottom).
    max: u8,
    /// The stored value.
    get: fn(&TacticSliders) -> u8,
    /// The stored write.
    set: fn(&mut TacticSliders, u8),
}

/// The five sliders, in the draft's order.
const SLIDERS: [SliderSpec; 5] = [
    SliderSpec {
        label: "Support range",
        max: 10,
        get: |sliders| sliders.support_range,
        set: |sliders, value| sliders.support_range = value,
    },
    SliderSpec {
        label: "Defensive line",
        max: 10,
        get: |sliders| sliders.defensive_line,
        set: |sliders, value| sliders.defensive_line = value,
    },
    SliderSpec {
        label: "Compactness",
        max: 10,
        get: |sliders| sliders.compactness,
        set: |sliders, value| sliders.compactness = value,
    },
    SliderSpec {
        label: "Numbers in attack",
        max: 3,
        get: |sliders| sliders.numbers_in_attack,
        set: |sliders, value| sliders.numbers_in_attack = value,
    },
    SliderSpec {
        label: "Numbers in defence",
        max: 3,
        get: |sliders| sliders.numbers_in_defence,
        set: |sliders, value| sliders.numbers_in_defence = value,
    },
];

/// One of the seven style pairs (the draft's `STYLE_TOGGLES`): its label,
/// the two choices' texts as the game names them and the `TacticStyle`
/// field it edits.
struct StyleToggle {
    /// The cell's label.
    label: &'static str,
    /// The stored 0's text.
    off: &'static str,
    /// The stored 1's text.
    on: &'static str,
    /// The stored flag.
    get: fn(&TacticStyle) -> bool,
    /// The stored write.
    set: fn(&mut TacticStyle, bool),
}

/// The seven style pairs, in the draft's order.
const STYLE_TOGGLES: [StyleToggle; 7] = [
    StyleToggle {
        label: "Attacking style",
        off: "Counter Attack",
        on: "Possession",
        get: |style| style.attacking_style,
        set: |style, value| style.attacking_style = value,
    },
    StyleToggle {
        label: "Attacking zone",
        off: "Centre",
        on: "Wide",
        get: |style| style.attacking_zone,
        set: |style, value| style.attacking_zone = value,
    },
    StyleToggle {
        label: "Buildup",
        off: "Long Pass",
        on: "Short Pass",
        get: |style| style.buildup,
        set: |style, value| style.buildup = value,
    },
    StyleToggle {
        label: "Positioning",
        off: "Maintain",
        on: "Flexible",
        get: |style| style.positioning,
        set: |style, value| style.positioning = value,
    },
    StyleToggle {
        label: "Defensive style",
        off: "Frontline Pressure",
        on: "All Out Defence",
        get: |style| style.defensive_style,
        set: |style, value| style.defensive_style = value,
    },
    StyleToggle {
        label: "Containment area",
        off: "Middle",
        on: "Wide",
        get: |style| style.containment_area,
        set: |style, value| style.containment_area = value,
    },
    StyleToggle {
        label: "Pressure",
        off: "Aggressive",
        on: "Conservative",
        get: |style| style.pressure,
        set: |style, value| style.pressure = value,
    },
];

/// The preset selector and the three groups, in the draft's order and
/// wording. `names` gives each roster slot's display name, indexed by
/// slot (an empty string an empty one); a slot past `names`, or an empty
/// one, shows "(empty)". Answers `true` only when `tactics` changed this
/// frame: the preset selector writes `view`, not the record, and an idle
/// frame, a hover or a re-pick of the stored value leaves it equal.
pub fn tactics_settings(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    version: PesVersion,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    names: &[&str],
) -> bool {
    ui.push_id(id_salt, |ui| {
        // The answer is the deep compare, not the responses' `changed()`:
        // a combo opened and closed, or a control re-picked at its own
        // value, reports nothing.
        let before = tactics.clone();
        ui.horizontal(|ui| {
            ui.weak("Preset");
            for preset in 0..3 {
                ui.selectable_value(&mut view.preset, preset, (preset + 1).to_string());
            }
        });
        let TeamTactics {
            presets,
            starting_eleven,
            auto,
            ..
        } = tactics;
        let preset = &mut presets[view.preset.min(2)];
        formation_settings(ui, preset);
        instruction_table(ui, version, preset, starting_eleven, names);
        automatic_settings(ui, auto);
        *tactics != before
    })
    .inner
}

/// The "Formation settings" group: the fluid-formation switch, the five
/// sliders and the seven style pairs (the draft's grid: five cells, then
/// four, then three).
fn formation_settings(ui: &mut egui::Ui, preset: &mut TacticsPreset) {
    ui.strong("Formation settings");
    match &mut preset.style.fluid {
        Some(fluid) => {
            ui.checkbox(fluid, "Fluid formation");
        }
        None => {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, "Fluid formation"))
                .on_disabled_hover_text(gate_tip(TacticsField::Preset {
                    preset: 0,
                    field: PresetField::FluidFormation,
                }));
        }
    }
    egui::Grid::new("formation settings").show(ui, |ui| {
        for spec in &SLIDERS {
            ui.horizontal(|ui| {
                ui.label(spec.label);
                let mut value = (spec.get)(&preset.sliders);
                if ui
                    .add(
                        egui::Slider::new(&mut value, 1..=spec.max)
                            .clamping(egui::SliderClamping::Edits),
                    )
                    .changed()
                {
                    (spec.set)(&mut preset.sliders, value);
                }
            });
        }
        ui.end_row();
        for chunk in STYLE_TOGGLES.chunks(4) {
            for toggle in chunk {
                style_toggle(ui, &mut preset.style, toggle);
            }
            ui.end_row();
        }
    });
}

/// One style pair's cell: its label and the two choices as the game names
/// them.
fn style_toggle(ui: &mut egui::Ui, style: &mut TacticStyle, toggle: &StyleToggle) {
    ui.horizontal(|ui| {
        ui.label(toggle.label);
        let mut value = (toggle.get)(style);
        // Both choices are drawn every frame: an `||` over the two would
        // skip the second on the frame the first is clicked.
        ui.selectable_value(&mut value, false, toggle.off);
        ui.selectable_value(&mut value, true, toggle.on);
        if value != (toggle.get)(style) {
            (toggle.set)(style, value);
        }
    });
}

/// The "Advanced instructions" group (PES 17+): the four rows Attack 1,
/// Attack 2, Defence 1, Defence 2, each its instruction and its target.
/// A version with no instruction table keeps the group, disabled, with
/// the `PES NN+` tooltip naming the first version that has it.
fn instruction_table(
    ui: &mut egui::Ui,
    version: PesVersion,
    preset: &mut TacticsPreset,
    starting_eleven: &[u8; 11],
    names: &[&str],
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
                names,
            );
            ui.end_row();
            instruction_row(
                ui,
                version,
                "attack 2",
                "Attack 2",
                &mut attack[1],
                starting_eleven,
                names,
            );
            ui.end_row();
            instruction_row(
                ui,
                version,
                "defence 1",
                "Defence 1",
                &mut defence[0],
                starting_eleven,
                names,
            );
            ui.end_row();
            instruction_row(
                ui,
                version,
                "defence 2",
                "Defence 2",
                &mut defence[1],
                starting_eleven,
                names,
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
    names: &[&str],
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
            Some(index) => format!("{} {}", index + 1, name_at(names, target)),
            None => name_at(names, target).to_owned(),
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
                        format!("{} {}", index + 1, name_at(names, *slot)),
                    );
                }
            });
    });
    entry.player = target;
}

/// The "Automatic settings" group: the substitution byte as stored, the
/// offside trap, the preset change and the attack/defence levels.
fn automatic_settings(ui: &mut egui::Ui, auto: &mut TeamAutoFlags) {
    ui.strong("Automatic settings");
    ui.horizontal(|ui| {
        match &mut auto.substitution {
            Some(substitution) => {
                ui.label("Substitution");
                ui.add(egui::DragValue::new(substitution).range(0..=u8::MAX))
                    .on_hover_text(
                        "The automatic substitution setting as the save stores it: 0 is off; \
                         on PES 2021, 1 is very late, 2 flexible and 3 very early",
                    );
            }
            None => {
                ui.label("Substitution");
                ui.add_enabled(
                    false,
                    egui::Button::new("-").min_size(ui.spacing().interact_size),
                )
                .on_disabled_hover_text(gate_tip(TacticsField::AutoSubstitution));
            }
        }
        auto_flag(
            ui,
            "Offside trap",
            &mut auto.offside_trap,
            TacticsField::AutoOffsideTrap,
        );
        auto_flag(
            ui,
            "Preset change",
            &mut auto.preset_change,
            TacticsField::AutoPresetChange,
        );
        auto_flag(
            ui,
            "Attack/defence levels",
            &mut auto.attack_defence_levels,
            TacticsField::AutoAttackDefenceLevels,
        );
    });
}

/// One automatic-settings flag's checkbox; a version that does not store
/// it keeps it, disabled with the `PES NN+` tooltip.
fn auto_flag(ui: &mut egui::Ui, label: &str, value: &mut Option<bool>, field: TacticsField) {
    match value {
        Some(flag) => {
            ui.checkbox(flag, label);
        }
        None => {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, label))
                .on_disabled_hover_text(gate_tip(field));
        }
    }
}

/// The `PES NN+` tooltip naming the first version whose tactics record
/// stores `field` (the card pickers' rule for a field a version lacks).
fn gate_tip(field: TacticsField) -> String {
    format!("PES {}+", first_tactics_version(field).number())
}

/// The oldest version whose tactics record stores `field` (a field the
/// widget gates is stored by some version).
fn first_tactics_version(field: TacticsField) -> PesVersion {
    PesVersion::ALL
        .iter()
        .copied()
        .find(|version| schema_for(*version).tactic.has(field))
        .expect("a gated field is stored by some version")
}

/// Roster `slot`'s display name, `(empty)` when `names` is shorter than
/// the slot or the slot names nobody.
fn name_at<'a>(names: &[&'a str], slot: u8) -> &'a str {
    names
        .get(usize::from(slot))
        .copied()
        .filter(|name| !name.is_empty())
        .unwrap_or("(empty)")
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
    use egui::accesskit::{self, Role};
    use pes_savefile::EditFile;
    use pes_savefile::model::names::display_name;

    use super::*;
    use crate::test_support::{Headless, centre, pointer_at};

    impl Headless {
        /// One frame of `tactics_settings` over `tactics`, `view` and
        /// `names`: the frame's AccessKit nodes and whether the frame
        /// changed `tactics`.
        fn tactics(
            &mut self,
            version: PesVersion,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            names: &[&str],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, bool) {
            self.tactics_at(None, version, tactics, view, names, events)
        }

        /// `tactics` with `time` as the frame's timestamp (tooltips count
        /// their delay on it).
        fn tactics_at(
            &mut self,
            time: Option<f64>,
            version: PesVersion,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            names: &[&str],
            events: Vec<egui::Event>,
        ) -> (Vec<accesskit::Node>, bool) {
            let mut changed = None;
            let (nodes, _) = self.frame_at(time, events, |ui| {
                changed = Some(tactics_settings(
                    ui, "tactics", version, tactics, view, names,
                ));
            });
            (nodes, changed.expect("the widget ran"))
        }

        /// A click at `pos` over the widget (press frame, then the release
        /// frame `clicked()` lands on).
        fn click_tactics(
            &mut self,
            pos: egui::Pos2,
            version: PesVersion,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            names: &[&str],
        ) -> (Vec<accesskit::Node>, bool) {
            self.tactics(version, tactics, view, names, pointer_at(pos, true));
            self.tactics(version, tactics, view, names, pointer_at(pos, false))
        }

        /// `tactics` keeping the nodes' AccessKit ids (for
        /// `AccessKitActionRequest` events).
        fn tactics_ids(
            &mut self,
            version: PesVersion,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            names: &[&str],
            events: Vec<egui::Event>,
        ) -> (Vec<(accesskit::NodeId, accesskit::Node)>, bool) {
            let mut changed = None;
            let (nodes, _) = self.frame_at_ids(None, events, |ui| {
                changed = Some(tactics_settings(
                    ui, "tactics", version, tactics, view, names,
                ));
            });
            (nodes, changed.expect("the widget ran"))
        }

        /// An AccessKit `Click` action on `id`: a click delivered as the
        /// accessibility event, for widgets a pointer click does not reach
        /// (a combo's popup items).
        fn action_click_tactics(
            &mut self,
            id: accesskit::NodeId,
            version: PesVersion,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            names: &[&str],
        ) -> (Vec<accesskit::Node>, bool) {
            self.tactics(
                version,
                tactics,
                view,
                names,
                vec![egui::Event::AccessKitActionRequest(
                    accesskit::ActionRequest {
                        action: accesskit::Action::Click,
                        target_tree: accesskit::TreeId::ROOT,
                        target_node: id,
                        data: None,
                    },
                )],
            )
        }

        /// The text of the tooltip drawn while the pointer rests at `pos`
        /// containing `needle` (bounded: a missing tooltip fails rather
        /// than hangs; needs the context's `tooltip_delay` at 0). The
        /// pointer first moves away for a frame: egui refuses a tooltip
        /// while the last click is more recent than the last movement.
        fn tactics_tooltip(
            &mut self,
            pos: egui::Pos2,
            needle: &str,
            version: PesVersion,
            tactics: &mut TeamTactics,
            view: &mut TacticsView,
            names: &[&str],
        ) -> Option<String> {
            self.tactics(
                version,
                tactics,
                view,
                names,
                vec![egui::Event::PointerMoved(egui::Pos2::ZERO)],
            );
            self.tactics(
                version,
                tactics,
                view,
                names,
                vec![egui::Event::PointerMoved(pos)],
            );
            for _ in 0..90 {
                let (nodes, _) = self.tactics(version, tactics, view, names, vec![]);
                if let Some(value) = nodes.iter().find_map(|node| {
                    node.value()
                        .filter(|value| value.contains(needle))
                        .map(str::to_owned)
                }) {
                    return Some(value);
                }
            }
            None
        }
    }

    /// Team 701's `TeamTactics` of `version`'s fixture save.
    fn fixture_tactics(version: PesVersion) -> TeamTactics {
        EditFile::from_bytes(&pes_savefile::fixture_save(version))
            .expect("fixture save")
            .team(701)
            .expect("team 701 is in the fixture")
            .tactics
            .clone()
    }

    /// Team 701's roster's display names, indexed by slot ("" an empty
    /// one), as the host builds them for `names`.
    fn fixture_names(version: PesVersion) -> Vec<String> {
        let file =
            EditFile::from_bytes(&pes_savefile::fixture_save(version)).expect("fixture save");
        let team = file.team(701).expect("team 701 is in the fixture");
        team.roster
            .iter()
            .map(|slot| {
                file.player(slot.player_id)
                    .map(|player| display_name(&player.name))
                    .unwrap_or_default()
            })
            .collect()
    }

    /// `names` as the `&[&str]` the widget takes.
    fn name_refs(names: &[String]) -> Vec<&str> {
        names.iter().map(String::as_str).collect()
    }

    /// The widget node of `role` labeled `label`.
    fn entry<'a>(nodes: &'a [accesskit::Node], role: Role, label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == role && node.label() == Some(label))
            .unwrap_or_else(|| panic!("no {role:?} labeled {label:?}"))
    }

    /// The nodes of `role` right of `label`'s, on its row, in x order.
    fn controls_right_of<'a>(
        nodes: &'a [accesskit::Node],
        label: &str,
        role: Role,
    ) -> Vec<&'a accesskit::Node> {
        let label_node = nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(label))
            .unwrap_or_else(|| panic!("no label {label:?}"));
        let bounds = label_node.bounds().expect("the label's bounds");
        let mut found: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| {
                node.role() == role
                    && node
                        .bounds()
                        .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
            })
            .collect();
        found.sort_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        });
        found
    }

    /// The first node of `role` right of `label`'s, on its row.
    fn control_right_of<'a>(
        nodes: &'a [accesskit::Node],
        label: &str,
        role: Role,
    ) -> &'a accesskit::Node {
        controls_right_of(nodes, label, role)
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("no {role:?} right of {label:?}"))
    }

    /// `nodes`' buttons' labels in draw order (top to bottom, left to
    /// right), kept only when `keep` holds.
    fn button_labels(nodes: &[accesskit::Node], keep: impl Fn(&str) -> bool) -> Vec<String> {
        let mut buttons: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| node.role() == Role::Button && node.label().is_some_and(&keep))
            .collect();
        buttons.sort_by(|a, b| {
            let key = |node: &&accesskit::Node| {
                let bounds = node.bounds().expect("drawn bounds");
                (bounds.y0, bounds.x0)
            };
            key(a).partial_cmp(&key(b)).expect("ordered bounds")
        });
        buttons
            .iter()
            .filter_map(|node| node.label().map(str::to_owned))
            .collect()
    }

    /// The AccessKit id of the widget node of `role` labeled `label`.
    fn entry_id(
        nodes: &[(accesskit::NodeId, accesskit::Node)],
        role: Role,
        label: &str,
    ) -> accesskit::NodeId {
        nodes
            .iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .map(|(id, _)| *id)
            .unwrap_or_else(|| panic!("no {role:?} labeled {label:?}"))
    }

    /// A click on `node`: press on one frame, release on the next.
    fn click(
        window: &mut Headless,
        node: &accesskit::Node,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        names: &[&str],
    ) -> (Vec<accesskit::Node>, bool) {
        window.click_tactics(centre(node), version, tactics, view, names)
    }

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
            let names = fixture_names(version);
            let names = name_refs(&names);
            let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
            let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
            let (nodes, _) = click(&mut window, combo, version, &mut tactics, &mut view, &names);
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
            let _ = click(&mut window, combo, version, &mut tactics, &mut view, &names);
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
        let names = fixture_names(PesVersion::Pes21);
        let names = name_refs(&names);

        let (nodes, changed) =
            window.tactics(PesVersion::Pes21, &mut tactics, &mut view, &names, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
        assert_eq!(
            combo.value(),
            Some("Tight Marking"),
            "the stored Tight Marking shows as the combo's text"
        );
        assert!(!changed, "the frame reports no change");
        let (_, changed) =
            window.tactics(PesVersion::Pes21, &mut tactics, &mut view, &names, vec![]);
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
        let names = fixture_names(PesVersion::Pes21);
        let names = name_refs(&names);
        let (nodes, _) = window.tactics(PesVersion::Pes21, &mut tactics, &mut view, &names, vec![]);
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
        let names = fixture_names(version);
        let names = name_refs(&names);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);

        // Open the instruction combo and pick Counter Target.
        let (_, _) = click(&mut window, combo, version, &mut tactics, &mut view, &names);
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &names, vec![]);
        let item = entry_id(&ids, Role::Button, "Counter Target");
        let (_, changed) =
            window.action_click_tactics(item, version, &mut tactics, &mut view, &names);
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
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
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
            &names,
        );
        // One idle frame: the instruction popup draws a last time on the
        // frame that closed it, so the items below are the target's alone.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let starters: Vec<String> = (0..11)
            .map(|index| {
                format!(
                    "{} {}",
                    index + 1,
                    name_at(&names, tactics.starting_eleven[index])
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
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &names, vec![]);
        let third = entry_id(&ids, Role::Button, &starters[2]);
        let (_, changed) =
            window.action_click_tactics(third, version, &mut tactics, &mut view, &names);
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
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
        let (_, _) = click(&mut window, combo, version, &mut tactics, &mut view, &names);
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &names, vec![]);
        let item = entry_id(&ids, Role::Button, "Gegenpress");
        let (_, changed) =
            window.action_click_tactics(item, version, &mut tactics, &mut view, &names);
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
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
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

    // plans/save_editor.md "Editing UI" (the version-gated bullet) on this
    // tab: a field the version lacks stays in its place, disabled, its
    // hover naming the first version that has it.
    #[test]
    fn fields_the_version_lacks_draw_disabled_with_their_versions() {
        let mut window = Headless::sized(800.0, 900.0);
        window
            .ctx
            .global_style_mut(|style| style.interaction.tooltip_delay = 0.0);
        // PES 16: the instruction table and Attack/defence levels are
        // disabled with "PES 17+".
        let mut tactics = fixture_tactics(PesVersion::Pes16);
        let mut view = TacticsView::default();
        let names = fixture_names(PesVersion::Pes16);
        let names = name_refs(&names);
        let (nodes, _) = window.tactics(PesVersion::Pes16, &mut tactics, &mut view, &names, vec![]);
        let table = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Label && node.value() == Some("(none on this version)")
            })
            .expect("the table's stand-in");
        assert!(table.is_disabled(), "the table's stand-in is disabled");
        assert_eq!(
            window.tactics_tooltip(
                centre(table),
                "PES 17+",
                PesVersion::Pes16,
                &mut tactics,
                &mut view,
                &names,
            ),
            Some("PES 17+".to_string()),
            "the instruction table's hover"
        );
        let levels = entry(&nodes, Role::CheckBox, "Attack/defence levels");
        assert!(levels.is_disabled(), "the flag is disabled on PES 16");
        assert_eq!(
            window.tactics_tooltip(
                centre(levels),
                "PES 17+",
                PesVersion::Pes16,
                &mut tactics,
                &mut view,
                &names,
            ),
            Some("PES 17+".to_string()),
            "Attack/defence levels' hover"
        );

        // PES 15: Fluid formation is disabled with "PES 16+".
        let mut tactics = fixture_tactics(PesVersion::Pes15);
        let mut view = TacticsView::default();
        let names = fixture_names(PesVersion::Pes15);
        let names = name_refs(&names);
        let (nodes, _) = window.tactics(PesVersion::Pes15, &mut tactics, &mut view, &names, vec![]);
        let fluid = entry(&nodes, Role::CheckBox, "Fluid formation");
        assert!(fluid.is_disabled(), "the flag is disabled on PES 15");
        assert_eq!(
            window.tactics_tooltip(
                centre(fluid),
                "PES 16+",
                PesVersion::Pes15,
                &mut tactics,
                &mut view,
                &names,
            ),
            Some("PES 16+".to_string()),
            "Fluid formation's hover"
        );
    }

    // Every control edits `presets[view.preset]` or the team-wide fields,
    // nothing else: with preset 2 on screen, a slider move and a style
    // click write the second preset alone.
    #[test]
    fn the_controls_edit_the_preset_on_screen() {
        let mut window = Headless::sized(800.0, 900.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let mut view = TacticsView { preset: 1 };
        let names = fixture_names(version);
        let names = name_refs(&names);
        let presets_before = tactics.presets.clone();

        // The preset selector: pick 2 (writes `view`, not the record).
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let two = entry(&nodes, Role::Button, "2");
        let (_, changed) = click(&mut window, two, version, &mut tactics, &mut view, &names);
        assert_eq!(view.preset, 1, "preset 2 stays on screen");
        assert!(!changed, "the preset selector writes no record");
        assert_eq!(tactics.presets, presets_before, "nor touches one");

        // Move Support range's slider one stop up.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let slider = control_right_of(&nodes, "Support range", Role::Slider);
        let bounds = slider.bounds().expect("slider bounds");
        let from = egui::pos2(((bounds.x0 + bounds.x1) / 2.0) as f32, centre(slider).y);
        let to = egui::pos2(bounds.x1 as f32 - 4.0, from.y);
        let mut changed = false;
        for events in [
            pointer_at(from, true),
            vec![egui::Event::PointerMoved(to)],
            pointer_at(to, false),
        ] {
            let (_, frame_changed) =
                window.tactics(version, &mut tactics, &mut view, &names, events);
            changed |= frame_changed;
        }
        assert!(changed, "the drag changed the record");
        assert_ne!(
            tactics.presets[1].sliders, presets_before[1].sliders,
            "preset 2's sliders moved"
        );
        assert_eq!(tactics.presets[0], presets_before[0], "preset 1 untouched");
        assert_eq!(tactics.presets[2], presets_before[2], "preset 3 untouched");

        // Click a style pair's on choice.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let possession = entry(&nodes, Role::Button, "Possession");
        let (_, changed) = click(
            &mut window,
            possession,
            version,
            &mut tactics,
            &mut view,
            &names,
        );
        assert!(changed, "the click changed the record");
        assert!(
            tactics.presets[1].style.attacking_style,
            "preset 2's attacking style is Possession"
        );
        assert_eq!(tactics.presets[0], presets_before[0], "preset 1 untouched");
        assert_eq!(tactics.presets[2], presets_before[2], "preset 3 untouched");
    }

    // `names` shorter than a slot, or an empty one, shows "(empty)" in the
    // target items.
    #[test]
    fn a_short_or_empty_names_slice_shows_empty() {
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
        // Three names for forty roster slots: slots past the slice, and
        // an empty one, show "(empty)".
        let names = ["ALPHA".to_owned(), "BETA".to_owned(), String::new()];
        let names = name_refs(&names);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
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
            &names,
        );
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
        let starters: Vec<String> = (0..11)
            .map(|index| {
                format!(
                    "{} {}",
                    index + 1,
                    name_at(&names, tactics.starting_eleven[index])
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

    // plans/save_editor.md "Tactics editor" (the widget's contract): an
    // idle frame, a hover, opening a combo and closing it, or re-picking
    // the current value reports no change and leaves the record equal.
    #[test]
    fn no_gesture_no_change() {
        let mut window = Headless::sized(800.0, 900.0);
        for version in [PesVersion::Pes15, PesVersion::Pes17, PesVersion::Pes21] {
            let mut tactics = fixture_tactics(version);
            let before = tactics.clone();
            let mut view = TacticsView::default();
            let names = fixture_names(version);
            let names = name_refs(&names);
            let (nodes, changed) = window.tactics(version, &mut tactics, &mut view, &names, vec![]);
            assert!(!changed, "{version:?}: an idle frame reports no change");
            assert_eq!(tactics, before, "{version:?}: and edits nothing");

            // Hover over every interactive control: each frame reports no
            // change and edits nothing.
            let controls: Vec<egui::Pos2> = nodes
                .iter()
                .filter(|node| {
                    matches!(
                        node.role(),
                        Role::Button
                            | Role::CheckBox
                            | Role::ComboBox
                            | Role::Slider
                            | Role::SpinButton
                    ) && !node.is_disabled()
                })
                .map(centre)
                .collect();
            for pos in controls {
                let (_, changed) = window.tactics(
                    version,
                    &mut tactics,
                    &mut view,
                    &names,
                    vec![egui::Event::PointerMoved(pos)],
                );
                assert!(!changed, "{version:?}: a hover reports no change");
                assert_eq!(tactics, before, "{version:?}: a hover edits nothing");
            }

            if version == PesVersion::Pes15 {
                continue;
            }
            // Open the Attack 1 instruction combo and close it again.
            let combo = control_right_of(&nodes, "Attack 1", Role::ComboBox);
            let (_, changed) = click(&mut window, combo, version, &mut tactics, &mut view, &names);
            assert!(!changed, "{version:?}: opening the combo writes nothing");
            assert_eq!(tactics, before, "{version:?}: and edits nothing");

            // Re-pick the current value inside it (the fixture's Attack 1
            // is Off, "Unassigned"): no change, nothing written.
            let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &names, vec![]);
            let same = entry_id(&ids, Role::Button, "Unassigned");
            let (_, changed) =
                window.action_click_tactics(same, version, &mut tactics, &mut view, &names);
            assert!(!changed, "{version:?}: re-picking the value writes nothing");
            assert_eq!(tactics, before, "{version:?}: and edits nothing");

            // An out-of-range stored slider value shows as stored and is
            // not rewritten until the member moves it.
            let mut out_of_range = before.clone();
            out_of_range.presets[0].sliders.support_range = 0;
            let expected = out_of_range.clone();
            let (nodes, changed) =
                window.tactics(version, &mut out_of_range, &mut view, &names, vec![]);
            assert!(
                !changed,
                "{version:?}: an out-of-range value is not rewritten"
            );
            assert_eq!(out_of_range, expected, "{version:?}: nor clamped on draw");
            let slider = control_right_of(&nodes, "Support range", Role::Slider);
            assert_eq!(
                slider.numeric_value(),
                Some(0.0),
                "{version:?}: the slider shows the stored value"
            );

            // A click on the open combo toggles it closed again.
            let (_, changed) = click(&mut window, combo, version, &mut tactics, &mut view, &names);
            assert!(!changed, "{version:?}: closing the combo writes nothing");
            assert_eq!(tactics, before, "{version:?}: and edits nothing");
        }
    }
}
