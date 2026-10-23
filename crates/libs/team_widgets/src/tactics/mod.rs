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
use pes_savefile::schema::fields::{PresetField, TacticsField};
use pes_savefile::schema::schema_for;
use pes_version::PesVersion;

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

/// The "Formation settings" group: the fluid-formation switch, the five
/// sliders and the seven style pairs, in rows that wrap (the draft's
/// `.row`, `flex-wrap: wrap`): each cell — a slider's label and slider,
/// a pair's label and two choices — stays whole on one row, and a host's
/// vertical-only scroll never clips a row's end at a narrow pane.
/// egui's wrap sees only widgets, so each cell is allocated at its
/// measured width: a bare `ui.horizontal` inside `horizontal_wrapped`
/// claims the row's remaining width and never reaches the wrap check.
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
    ui.horizontal_wrapped(|ui| {
        let spacing = ui.spacing().item_spacing.x;
        for spec in &SLIDERS {
            // The cell's drawn width: the label, the rail
            // (`slider_width`) and the value field (a DragValue's
            // `interact_size.x`), plus the gaps.
            let width = text_width(ui, spec.label)
                + ui.spacing().slider_width
                + ui.spacing().interact_size.x
                + 2.0 * spacing;
            cell(ui, width, |ui| {
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
        for toggle in &STYLE_TOGGLES {
            // The pair cell: the label and the two choices, each choice a
            // selectable's text and padding.
            let width = text_width(ui, toggle.label)
                + choice_width(ui, toggle.off)
                + choice_width(ui, toggle.on)
                + 2.0 * spacing;
            cell(ui, width, |ui| style_toggle(ui, &mut preset.style, toggle));
        }
    });
}

/// `text`'s galley width in the body font.
fn text_width(ui: &egui::Ui, text: &str) -> f32 {
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

/// One style pair's cell's contents: its label and the two choices as the
/// game names them.
fn style_toggle(ui: &mut egui::Ui, style: &mut TacticStyle, toggle: &StyleToggle) {
    ui.label(toggle.label);
    let mut value = (toggle.get)(style);
    // Both choices are drawn every frame: an `||` over the two would
    // skip the second on the frame the first is clicked.
    ui.selectable_value(&mut value, false, toggle.off);
    ui.selectable_value(&mut value, true, toggle.on);
    if value != (toggle.get)(style) {
        (toggle.set)(style, value);
    }
}

/// The "Automatic settings" group: the substitution byte as stored, the
/// offside trap, the preset change and the attack/defence levels. The
/// row wraps like the formation settings' (the same `cell` measure for
/// the label's pair).
fn automatic_settings(ui: &mut egui::Ui, auto: &mut TeamAutoFlags) {
    ui.strong("Automatic settings");
    ui.horizontal_wrapped(|ui| {
        let width = text_width(ui, "Substitution")
            + ui.spacing().interact_size.x
            + 2.0 * ui.spacing().item_spacing.x;
        cell(ui, width, |ui| {
            ui.label("Substitution");
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

#[cfg(test)]
mod tests;
