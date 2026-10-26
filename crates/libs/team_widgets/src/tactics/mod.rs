//! The tactics settings of `plans/save_editor.md` "Tactics editor" (the
//! draft's `tacticsTab`, minus the pitch canvas and the lineup, which are
//! `crate::pitch`): the preset selector, the Formation settings group, the
//! Advanced instructions table (`instructions`), the Player assignments
//! palette (`assignments`) and the Automatic settings group.
//! `tactics_settings` edits `tactics` in place and answers whether it
//! changed; the host decides what the change means (the contract in
//! `plans/save_editor.md` "`libs/team_widgets`").

use pes_savefile::model::tactics::{
    TacticSliders, TacticStyle, TacticsPreset, TeamAutoFlags, TeamTactics,
};
use pes_savefile::schema::fields::{
    FormationPart, InstructionPart, InstructionSide, PresetField, TacticsField,
};
use pes_savefile::schema::{instruction, schema_for};
use pes_version::PesVersion;

use crate::pitch::{FORMATIONS, position_label};

/// The role palette's armed modes, chips and note.
mod assignments;
/// The "Advanced instructions" table and its rows.
mod instructions;

pub use assignments::assign_note;
pub(crate) use assignments::{Assign, OpenSlot};

/// What the tactics editor shows that the record does not hold: the
/// preset and the formation on screen, the armed assignment mode and the
/// open slot popover.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TacticsView {
    /// The preset the controls edit, 0 to 2, shown 1 to 3.
    pub preset: usize,
    /// The preset's formation the lineup and pitch show: 0 kick-off, 1
    /// in-possession, 2 out-of-possession. The pitch's own selector is
    /// its only writer.
    pub(crate) formation: usize,
    /// The armed assignment mode, `None` when none: a pitch click then
    /// writes the role instead of opening the slot popover.
    pub(crate) assign: Option<Assign>,
    /// The formation slot whose popover is open, `None` closed. `pitch`
    /// closes it when the preset, the formation or the team it opened on
    /// changed since.
    pub(crate) popover: Option<OpenSlot>,
}

/// What a roster slot shows where the editor lists it (the instruction
/// targets, the lineup and bench rows): the player's display name and
/// registered position, both `None`-equivalent on an empty slot ("" and
/// `None` read "(empty)" and "--").
#[derive(Debug, Clone, Copy)]
pub struct RosterRow<'a> {
    /// The slot's display name, "" an empty or nameless one.
    pub name: &'a str,
    /// The slot's raw savefile name, colour codes kept, "" an empty or
    /// nameless one: what the lineup rows and the slot popover draw for
    /// the coloured runs (the markers, chips and combos read `name`).
    pub raw_name: &'a str,
    /// The player's registered position byte; `None` an empty slot.
    pub registered_position: Option<u8>,
    /// The player's portrait, already uploaded by the host; `None` when none
    /// was found (or the host reads none): the marker then draws without one.
    pub portrait: Option<egui::TextureId>,
}

/// One of the five sliders (the draft's `SLIDERS`): its label, its range
/// top and the `TacticSliders` field it edits.
struct SliderSpec {
    /// The cell's label.
    label: &'static str,
    /// The preset field the slider writes (`tactics_term`'s key into the
    /// table, so a row's label is never a second copy).
    field: PresetField,
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
        field: PresetField::SupportRange,
        max: 10,
        get: |sliders| sliders.support_range,
        set: |sliders, value| sliders.support_range = value,
    },
    SliderSpec {
        label: "Defensive line",
        field: PresetField::DefensiveLine,
        max: 10,
        get: |sliders| sliders.defensive_line,
        set: |sliders, value| sliders.defensive_line = value,
    },
    SliderSpec {
        label: "Compactness",
        field: PresetField::Compactness,
        max: 10,
        get: |sliders| sliders.compactness,
        set: |sliders, value| sliders.compactness = value,
    },
    SliderSpec {
        label: "Numbers in attack",
        field: PresetField::NumbersInAttack,
        max: 3,
        get: |sliders| sliders.numbers_in_attack,
        set: |sliders, value| sliders.numbers_in_attack = value,
    },
    SliderSpec {
        label: "Numbers in defence",
        field: PresetField::NumbersInDefence,
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
    /// The preset field the pair writes (`tactics_term`'s key into the
    /// table, so a row's label and texts are never a second copy).
    field: PresetField,
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
        field: PresetField::AttackingStyle,
        off: "Counter Attack",
        on: "Possession",
        get: |style| style.attacking_style,
        set: |style, value| style.attacking_style = value,
    },
    StyleToggle {
        label: "Attacking zone",
        field: PresetField::AttackingZone,
        off: "Centre",
        on: "Wide",
        get: |style| style.attacking_zone,
        set: |style, value| style.attacking_zone = value,
    },
    StyleToggle {
        label: "Buildup",
        field: PresetField::Buildup,
        off: "Long Pass",
        on: "Short Pass",
        get: |style| style.buildup,
        set: |style, value| style.buildup = value,
    },
    StyleToggle {
        label: "Positioning",
        field: PresetField::Positioning,
        off: "Maintain",
        on: "Flexible",
        get: |style| style.positioning,
        set: |style, value| style.positioning = value,
    },
    StyleToggle {
        label: "Defensive style",
        field: PresetField::DefensiveStyle,
        off: "Frontline Pressure",
        on: "All Out Defence",
        get: |style| style.defensive_style,
        set: |style, value| style.defensive_style = value,
    },
    StyleToggle {
        label: "Containment area",
        field: PresetField::ContainmentArea,
        off: "Middle",
        on: "Wide",
        get: |style| style.containment_area,
        set: |style, value| style.containment_area = value,
    },
    StyleToggle {
        label: "Pressure",
        field: PresetField::Pressure,
        off: "Aggressive",
        on: "Conservative",
        get: |style| style.pressure,
        set: |style, value| style.pressure = value,
    },
];

/// The fluid switch's label (the checkbox's and `tactics_term`'s row's
/// one copy).
const FLUID_LABEL: &str = "Fluid formation";
/// The substitution field's label.
const SUBSTITUTION_LABEL: &str = "Substitution";
/// The offside-trap checkbox's label.
const OFFSIDE_TRAP_LABEL: &str = "Offside trap";
/// The preset-change checkbox's label.
const PRESET_CHANGE_LABEL: &str = "Preset change";
/// The attack/defence-levels checkbox's label.
const ATTACK_DEFENCE_LABEL: &str = "Attack/defence levels";

/// The preset selector (the draft's `presetSel`, in the tab's `tacbar`
/// beside the menus that act on the shown preset): "Preset" then 1, 2
/// and 3. Writes `view.preset`, never the record.
pub fn preset_selector(ui: &mut egui::Ui, view: &mut TacticsView) {
    ui.weak("Preset");
    for preset in 0..3 {
        ui.selectable_value(&mut view.preset, preset, (preset + 1).to_string());
    }
}

/// The four groups, in the draft's order and wording. `roster` gives each
/// roster slot's row, indexed by slot (an empty name or a slot past
/// `roster` shows "(empty)"). Answers `true` only when `tactics` changed
/// this frame: an idle frame, a hover or a re-pick of the stored value
/// leaves it equal.
pub fn tactics_settings(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    version: PesVersion,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) -> bool {
    ui.push_id(id_salt, |ui| {
        // The answer is the deep compare, not the responses' `changed()`:
        // a combo opened and closed, or a control re-picked at its own
        // value, reports nothing.
        let before = tactics.clone();
        let TeamTactics {
            presets,
            starting_eleven,
            set_pieces,
            players_to_join_attack,
            auto,
            ..
        } = tactics;
        let preset = &mut presets[view.preset.min(2)];
        formation_settings(ui, preset);
        instructions::instruction_table(ui, version, preset, starting_eleven, roster);
        assignments::player_assignments(ui, set_pieces, players_to_join_attack, view, roster);
        automatic_settings(ui, auto);
        *tactics != before
    })
    .inner
}

/// The "Formation settings" group: the fluid-formation switch, then one
/// `egui::Grid` of the five sliders and the seven style pairs, row-filled
/// in the draft's order — each pair a label cell and a control cell (the
/// slider; the two choices in a `ui.horizontal`). Two pairs a row where
/// the column's width holds two of the widest (`two_pairs_width`), else
/// one.
fn formation_settings(ui: &mut egui::Ui, preset: &mut TacticsPreset) {
    ui.strong("Formation settings");
    match &mut preset.style.fluid {
        Some(fluid) => {
            ui.checkbox(fluid, FLUID_LABEL);
        }
        None => {
            let mut off = false;
            ui.add_enabled(false, egui::Checkbox::new(&mut off, FLUID_LABEL))
                .on_disabled_hover_text(gate_tip(TacticsField::Preset {
                    preset: 0,
                    field: PresetField::FluidFormation,
                }));
        }
    }
    let per_row = if ui.available_width() >= two_pairs_width(ui) {
        2
    } else {
        1
    };
    egui::Grid::new("formation settings").show(ui, |ui| {
        let mut index = 0usize;
        for spec in &SLIDERS {
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
            index += 1;
            if index.is_multiple_of(per_row) {
                ui.end_row();
            }
        }
        for toggle in &STYLE_TOGGLES {
            ui.label(toggle.label);
            ui.horizontal(|ui| style_choices(ui, &mut preset.style, toggle));
            index += 1;
            if index.is_multiple_of(per_row) {
                ui.end_row();
            }
        }
    });
}

/// The width a two-pairs row of the grid needs: the pairs' labels land in
/// two columns, their controls in two more, so the row is the widest
/// label and the widest control of each column plus the grid's gaps (one
/// per cell boundary and a little slack — the bound stays an upper one,
/// so a miss wraps the row to one pair early rather than clipping it).
fn two_pairs_width(ui: &egui::Ui) -> f32 {
    let gap = ui.spacing().item_spacing.x;
    // A slider's control is the rail (`slider_width`), a gap and the
    // value's field (a DragValue's `interact_size.x`); a pair's is the
    // two choices in a `ui.horizontal` and their gap.
    let slider_control = ui.spacing().slider_width + gap + ui.spacing().interact_size.x;
    let mut label = [0.0f32; 2];
    let mut control = [0.0f32; 2];
    let mut widest = |column: usize, label_w: f32, control_w: f32| {
        label[column] = label[column].max(label_w);
        control[column] = control[column].max(control_w);
    };
    for (index, spec) in SLIDERS.iter().enumerate() {
        widest(index % 2, text_width(ui, spec.label), slider_control);
    }
    for (index, toggle) in STYLE_TOGGLES.iter().enumerate() {
        let choices = choice_width(ui, toggle.off) + choice_width(ui, toggle.on) + gap;
        widest(
            (SLIDERS.len() + index) % 2,
            text_width(ui, toggle.label),
            choices,
        );
    }
    label[0] + control[0] + label[1] + control[1] + 4.0 * gap
}

/// The Substitution pair's drawn width: the label and the DragValue
/// field (`interact_size.x`), plus the gap.
fn sub_cell_width(ui: &egui::Ui) -> f32 {
    text_width(ui, SUBSTITUTION_LABEL) + ui.spacing().interact_size.x + ui.spacing().item_spacing.x
}

/// `text`'s galley width in the body font.
pub(crate) fn text_width(ui: &egui::Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.to_owned(),
            egui::TextStyle::Body.resolve(ui.style()),
            egui::Color32::PLACEHOLDER,
        )
        .size()
        .x
}

/// A style choice's drawn width: its text plus a button's padding.
fn choice_width(ui: &egui::Ui, text: &str) -> f32 {
    text_width(ui, text) + ui.spacing().button_padding.x * 2.0
}

/// One cell of a wrapping row at `width`: a child row laid out like
/// `ui.horizontal`, so the cell's label and control keep one row.
fn cell(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, ui.spacing().interact_size.y),
        egui::Layout::left_to_right(egui::Align::Center),
        add,
    );
}

/// One style pair's control cell: the two choices as the game names
/// them, both drawn every frame — an `||` over the two would skip the
/// second on the frame the first is clicked.
fn style_choices(ui: &mut egui::Ui, style: &mut TacticStyle, toggle: &StyleToggle) {
    let mut value = (toggle.get)(style);
    ui.selectable_value(&mut value, false, toggle.off);
    ui.selectable_value(&mut value, true, toggle.on);
    if value != (toggle.get)(style) {
        (toggle.set)(style, value);
    }
}

/// The "Automatic settings" group: the substitution byte as stored, the
/// offside trap, the preset change and the attack/defence levels. The
/// row wraps on the measured `cell` for the label's pair, so a narrow
/// pane wraps the last checkbox below rather than clipping it.
fn automatic_settings(ui: &mut egui::Ui, auto: &mut TeamAutoFlags) {
    ui.strong("Automatic settings");
    ui.horizontal_wrapped(|ui| {
        cell(ui, sub_cell_width(ui), |ui| {
            ui.label(SUBSTITUTION_LABEL);
            match &mut auto.substitution {
                Some(substitution) => {
                    ui.add(egui::DragValue::new(substitution).range(0..=u8::MAX))
                        .on_hover_text(
                            "The automatic substitution setting as the save stores it: 0 is off; \
                             on PES 2021, 1 is very late, 2 flexible and 3 very early",
                        );
                }
                None => {
                    ui.add_enabled(
                        false,
                        egui::Button::new("-").min_size(ui.spacing().interact_size),
                    )
                    .on_disabled_hover_text(gate_tip(TacticsField::AutoSubstitution));
                }
            }
        });
        auto_flag(
            ui,
            OFFSIDE_TRAP_LABEL,
            &mut auto.offside_trap,
            TacticsField::AutoOffsideTrap,
        );
        auto_flag(
            ui,
            PRESET_CHANGE_LABEL,
            &mut auto.preset_change,
            TacticsField::AutoPresetChange,
        );
        auto_flag(
            ui,
            ATTACK_DEFENCE_LABEL,
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

/// Roster `slot`'s display name, `(empty)` when `roster` is shorter than
/// the slot or the slot names nobody.
pub(crate) fn name_at<'a>(roster: &[RosterRow<'a>], slot: u8) -> &'a str {
    roster
        .get(usize::from(slot))
        .map(|row| row.name)
        .filter(|name| !name.is_empty())
        .unwrap_or("(empty)")
}

/// `name_at` on the raw name (the colour codes kept): the lineup rows
/// and the slot popover draw it for the coloured text runs.
pub(crate) fn raw_name_at<'a>(roster: &[RosterRow<'a>], slot: u8) -> &'a str {
    roster
        .get(usize::from(slot))
        .map(|row| row.raw_name)
        .filter(|name| !name.is_empty())
        .unwrap_or("(empty)")
}

/// A tactics field's row as the Tactics tab words it, for the Save
/// editor's unsaved-changes prompt: the field's label, prefixed by its
/// preset ("Preset 1 · Support range"), and the two stored values as the
/// tab shows them (a style pair's texts, a taker's or the captain's
/// player name, a slider's number). `version` decodes the stored
/// instruction bytes; `new_tactics` names the formation slots' players
/// (the starting eleven's roster slots) and `old_tactics`/`new_tactics`
/// carry the instruction each side's target byte is worded under.
pub fn tactics_term(
    version: PesVersion,
    field: TacticsField,
    old: u32,
    new: u32,
    old_tactics: &TeamTactics,
    new_tactics: &TeamTactics,
    roster: &[RosterRow<'_>],
) -> (String, String, String) {
    let byte = |stored: u32| u8::try_from(stored).expect("a tactics field is a byte");
    match field {
        TacticsField::TeamId => ("Team ID".to_owned(), old.to_string(), new.to_string()),
        TacticsField::Preset { preset, field: pf } => (
            format!("Preset {} · {}", preset + 1, preset_label(pf)),
            preset_value(pf, old),
            preset_value(pf, new),
        ),
        TacticsField::Formation {
            preset,
            formation,
            slot,
            part,
        } => {
            let starter = new_tactics.starting_eleven[usize::from(slot)];
            let label = format!(
                "Preset {} · {} · {} · {}",
                preset + 1,
                FORMATIONS[usize::from(formation)],
                name_at(roster, starter),
                match part {
                    FormationPart::Position => "Position",
                    FormationPart::X => "X",
                    FormationPart::Y => "Y",
                }
            );
            let value = |stored: u32| match part {
                FormationPart::Position => position_label(byte(stored)).to_owned(),
                FormationPart::X | FormationPart::Y => stored.to_string(),
            };
            (label, value(old), value(new))
        }
        TacticsField::Instruction {
            preset,
            side,
            index,
            part,
        } => {
            let row = format!(
                "Preset {} · {} {}",
                preset + 1,
                match side {
                    InstructionSide::Attack => "Attack",
                    InstructionSide::Defence => "Defence",
                },
                index + 1
            );
            match part {
                InstructionPart::Instruction => {
                    let value = |stored: u32| {
                        let stored = byte(stored);
                        instruction::decode(version, stored)
                            .ok()
                            .flatten()
                            .map_or_else(
                                || format!("Unknown (0x{stored:02X})"),
                                |inst| instructions::instruction_label(inst).to_owned(),
                            )
                    };
                    (row, value(old), value(new))
                }
                InstructionPart::PlayerId => (
                    format!("{row} · Target"),
                    instructions::target_text(
                        instructions::instruction_at(version, old_tactics, preset, side, index),
                        byte(old),
                        &old_tactics.starting_eleven,
                        roster,
                    ),
                    instructions::target_text(
                        instructions::instruction_at(version, new_tactics, preset, side, index),
                        byte(new),
                        &new_tactics.starting_eleven,
                        roster,
                    ),
                ),
            }
        }
        TacticsField::ManMarking { preset, slot } => {
            let starter = new_tactics.starting_eleven[usize::from(slot)];
            let value = |stored: u32| {
                if stored == u32::from(u8::MAX) {
                    "None".to_owned()
                } else {
                    stored.to_string()
                }
            };
            (
                format!(
                    "Preset {} · Man marking · {}",
                    preset + 1,
                    name_at(roster, starter)
                ),
                value(old),
                value(new),
            )
        }
        TacticsField::FreeKickTakerLong
        | TacticsField::FreeKickTakerShort
        | TacticsField::FreeKickTakerSecond
        | TacticsField::CornerTakerLeft
        | TacticsField::CornerTakerRight
        | TacticsField::PenaltyTaker
        | TacticsField::Captain => (
            assignments::role_label(field)
                .expect("the roles are the chips' own table")
                .to_owned(),
            role_holder(roster, byte(old)),
            role_holder(roster, byte(new)),
        ),
        TacticsField::AutoSubstitution => (
            SUBSTITUTION_LABEL.to_owned(),
            old.to_string(),
            new.to_string(),
        ),
        TacticsField::AutoOffsideTrap => (
            OFFSIDE_TRAP_LABEL.to_owned(),
            on_off(old).to_owned(),
            on_off(new).to_owned(),
        ),
        TacticsField::AutoPresetChange => (
            PRESET_CHANGE_LABEL.to_owned(),
            on_off(old).to_owned(),
            on_off(new).to_owned(),
        ),
        TacticsField::AutoAttackDefenceLevels => (
            ATTACK_DEFENCE_LABEL.to_owned(),
            on_off(old).to_owned(),
            on_off(new).to_owned(),
        ),
        TacticsField::Starting(index) => (
            format!("Starting eleven · {}", index + 1),
            name_at(roster, byte(old)).to_owned(),
            name_at(roster, byte(new)).to_owned(),
        ),
        TacticsField::Bench(index) => (
            format!("Bench · {}", index + 1),
            name_at(roster, byte(old)).to_owned(),
            name_at(roster, byte(new)).to_owned(),
        ),
        TacticsField::PlayerToJoinAttack(index) => (
            format!("Join attack · {}", index + 1),
            join_text(roster, byte(old)),
            join_text(roster, byte(new)),
        ),
    }
}

/// A preset field's label on the tab (`SLIDERS`, `STYLE_TOGGLES` or the
/// fluid switch): `tactics_term` reads the tab's own tables, never a
/// second copy.
fn preset_label(field: PresetField) -> &'static str {
    if field == PresetField::FluidFormation {
        return FLUID_LABEL;
    }
    SLIDERS
        .iter()
        .find(|spec| spec.field == field)
        .map(|spec| spec.label)
        .or_else(|| {
            STYLE_TOGGLES
                .iter()
                .find(|toggle| toggle.field == field)
                .map(|toggle| toggle.label)
        })
        .expect("a preset field is a slider, a style pair or the fluid switch")
}

/// `stored` as the preset field's control shows it: a style pair's text,
/// "off"/"on" for the fluid switch, the number else.
fn preset_value(field: PresetField, stored: u32) -> String {
    if let Some(toggle) = STYLE_TOGGLES.iter().find(|toggle| toggle.field == field) {
        return if stored == 0 { toggle.off } else { toggle.on }.to_owned();
    }
    if field == PresetField::FluidFormation {
        return on_off(stored).to_owned();
    }
    stored.to_string()
}

/// A role chip's holder text: "Unassigned" for the none byte, the roster
/// slot's name else (the chips' own wording).
fn role_holder(roster: &[RosterRow<'_>], holder: u8) -> String {
    if holder == u8::MAX {
        return "Unassigned".to_owned();
    }
    name_at(roster, holder).to_owned()
}

/// A join-attack member's text as the chip lists it: the first name,
/// "none" for the none byte.
fn join_text(roster: &[RosterRow<'_>], slot: u8) -> String {
    if slot == u8::MAX {
        return "none".to_owned();
    }
    name_at(roster, slot)
        .split_whitespace()
        .next()
        .unwrap_or("(empty)")
        .to_owned()
}

/// A flag byte's text: a set bit reads "on".
fn on_off(stored: u32) -> &'static str {
    if stored == 0 { "off" } else { "on" }
}

#[cfg(test)]
mod tests;
