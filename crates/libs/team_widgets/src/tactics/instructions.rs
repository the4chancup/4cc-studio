//! The "Advanced instructions" grid (`plans/save_editor.md` "Tactics
//! editor", the numbered row 3 under "The pitch is the editor" and "What
//! the controls write", the instruction bullet): the headers Attack and
//! Defence over their two equal columns, each instruction's number
//! beside its two combos — the attack ones listing the attacking
//! instructions, the defence ones the defending, less Tight Marking —
//! and each target a chip under its combo, armed to a pitch click.

use pes_savefile::model::instruction::Instruction;
use pes_savefile::model::tactics::{AdvancedInstruction, TacticsPreset, TeamTactics};
use pes_savefile::schema::fields::{InstructionPart, InstructionSide, TacticsField};
use pes_savefile::schema::instruction;
use pes_version::PesVersion;

use super::assignments::{Assign, chip};
use super::{RosterRow, TacticsView, gate_tip, name_at, text_width};

/// The stored "no player" value of an instruction's target.
pub(super) const NO_TARGET: u8 = 0xFF;

/// The instructions' columns' and rows' gaps (the draft's `.tacinstr`
/// `column-gap: 8px; row-gap: 3px`).
const INSTR_COLUMN_GAP: f32 = 8.0;
const INSTR_ROW_GAP: f32 = 3.0;

/// What a side's combo offers (the draft's `instructionsFor`): the
/// version's stored table in its stored order, its own side's
/// instructions plus `Off`, less Tight Marking — it marks an opponent,
/// which Edit mode does not offer.
fn offered(version: PesVersion, side: InstructionSide) -> Vec<Instruction> {
    instruction::instructions(version)
        .iter()
        .copied()
        .filter(|inst| *inst != Instruction::TightMarking)
        .filter(|inst| *inst == Instruction::Off || inst.side() == Some(side))
        .collect()
}

/// One instruction's combo (the draft's `select`, filling its cell):
/// the side's list in stored order — a held value the list lacks
/// (Tight Marking or one of the other side's) appended as the current
/// choice — a stored byte the table does not list shown as its hex
/// and kept until a pick. Picking an instruction that takes no target
/// clears the pick.
fn instruction_combo(
    ui: &mut egui::Ui,
    id_salt: impl egui::AsIdSalt,
    version: PesVersion,
    side: InstructionSide,
    entry: &mut AdvancedInstruction,
) {
    let current = instruction::decode(version, entry.instruction)
        .ok()
        .flatten();
    let mut list = offered(version, side);
    if let Some(inst) = current
        && !list.contains(&inst)
    {
        list.push(inst);
    }
    let selected = match current {
        Some(inst) => instruction_label(inst).to_owned(),
        None => format!("Unknown (0x{:02X})", entry.instruction),
    };
    let mut pick = current;
    egui::ComboBox::from_id_salt(id_salt)
        .width(ui.available_width())
        .selected_text(selected.clone())
        .truncate()
        .show_ui(ui, |ui| {
            for inst in &list {
                ui.selectable_value(&mut pick, Some(*inst), instruction_label(*inst));
            }
        });
    if let Some(inst) = pick
        && pick != current
    {
        entry.instruction =
            instruction::encode(version, inst).expect("the offered list is the version's own");
        if !inst.takes_player() {
            entry.player = NO_TARGET;
        }
    }
}

/// One instruction's target chip (the draft's `pick`, a `rolechip` of
/// its column's width): armed, the next pitch click writes the clicked
/// starter to the target (`mode`, the slot's `Assign::Target`) — a
/// same-player write skipped — and a click on the armed chip disarms;
/// under an instruction that takes none the chip is disabled, reading
/// `No target` — or `An opponent` under a held Tight Marking — the
/// hovers the draft's titles.
fn target_chip(
    ui: &mut egui::Ui,
    version: PesVersion,
    mode: Assign,
    entry: &AdvancedInstruction,
    starting_eleven: &[u8; 11],
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) {
    let current = instruction::decode(version, entry.instruction)
        .ok()
        .flatten();
    let targetable = current.is_some_and(|inst| inst.takes_player());
    let marking = current == Some(Instruction::TightMarking);
    let armed = view.assign == Some(mode);
    let text = if marking {
        "An opponent".to_owned()
    } else if !targetable {
        "No target".to_owned()
    } else {
        target_text(current, entry.player, starting_eleven, roster)
    };
    let title = if marking {
        "Its target is an opponent, picked in a match"
    } else if !targetable {
        "This instruction takes no target"
    } else {
        "The instruction's target: click, then a pitch player"
    };
    if !targetable {
        ui.add_enabled_ui(false, |ui| chip(ui, &text, false, title))
            .response
            .on_disabled_hover_text(title);
    } else if chip(ui, &text, armed, title).clicked() {
        view.assign = (!armed).then_some(mode);
        view.popover = None;
    }
}

/// The "Advanced instructions" group's grid (the draft's `.tacinstr`,
/// PES 17+): the headers Attack and Defence over their two equal
/// columns, each instruction's number weak beside its two combos and
/// its target chip under its combo. A version with no instruction
/// table holds the weak line `(none on this version)` with the `PES
/// NN+` tooltip instead of disabled rows. The panel's heading is the
/// caller's `panel`.
pub(super) fn instruction_table(
    ui: &mut egui::Ui,
    version: PesVersion,
    preset: &mut TacticsPreset,
    starting_eleven: &[u8; 11],
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) {
    let (Some(attack), Some(defence)) = (
        preset.attack_instructions.as_mut(),
        preset.defence_instructions.as_mut(),
    ) else {
        ui.add_enabled_ui(false, |ui| ui.weak("(none on this version)"))
            .response
            .on_disabled_hover_text(gate_tip(TacticsField::Instruction {
                preset: 0,
                side: InstructionSide::Attack,
                index: 0,
                part: InstructionPart::Instruction,
            }));
        return;
    };
    // `.tacinstr`: the numbers' `auto` column, then the two equal combo
    // columns `1fr 1fr`, all `INSTR_COLUMN_GAP` apart, `INSTR_ROW_GAP`
    // between rows.
    let inner = ui.max_rect();
    let number_width = text_width(ui, "1").max(text_width(ui, "2"));
    let column = (inner.width() - number_width - 2.0 * INSTR_COLUMN_GAP) / 2.0;
    let attack_x = inner.min.x + number_width + INSTR_COLUMN_GAP;
    let defence_x = attack_x + column + INSTR_COLUMN_GAP;
    let combo_height = ui.spacing().interact_size.y;
    let cell = |ui: &mut egui::Ui, left: f32, top: f32, height: f32| {
        ui.new_child(
            egui::UiBuilder::new()
                .max_rect(egui::Rect::from_min_size(
                    egui::pos2(left, top),
                    egui::vec2(column, height),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        )
    };
    // The header row: an empty number cell, the side names over their
    // columns.
    let mut top = ui.cursor().min.y;
    for (x, header) in [(attack_x, "Attack"), (defence_x, "Defence")] {
        cell(ui, x, top, ui.spacing().interact_size.y).label(header);
    }
    top += ui.spacing().interact_size.y + INSTR_ROW_GAP;
    for index in 0..2 {
        let mut number_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(egui::Rect::from_min_size(
                    egui::pos2(inner.min.x, top),
                    egui::vec2(number_width, combo_height),
                ))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        number_ui.weak((index + 1).to_string());
        for (x, side, entry) in [
            (attack_x, InstructionSide::Attack, &mut attack[index]),
            (defence_x, InstructionSide::Defence, &mut defence[index]),
        ] {
            instruction_combo(
                &mut cell(ui, x, top, combo_height),
                (side, index),
                version,
                side,
                entry,
            );
        }
        top += combo_height + INSTR_ROW_GAP;
        for (x, side, entries) in [
            (attack_x, InstructionSide::Attack, &*attack),
            (defence_x, InstructionSide::Defence, &*defence),
        ] {
            let mut chip_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(egui::Rect::from_min_size(
                        egui::pos2(x, top),
                        egui::vec2(column, combo_height),
                    ))
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );
            target_chip(
                &mut chip_ui,
                version,
                Assign::Target { side, index },
                &entries[index],
                starting_eleven,
                view,
                roster,
            );
        }
        top += combo_height + INSTR_ROW_GAP;
    }
    // `new_child` cells don't grow this `Ui`'s `min_rect`: advance over
    // the grid's whole extent so the frame wraps the last chip and the
    // next panel starts below it.
    ui.advance_cursor_after_rect(egui::Rect::from_min_max(
        inner.min,
        egui::pos2(inner.max.x, top - INSTR_ROW_GAP),
    ));
}

/// The instruction byte `tactics` stores on `side` at `index` of
/// `preset`, decoded; `None` where the version or the entry holds none —
/// `target_text` then reads the byte as a player's, as the tab's `None`
/// arm does.
pub(super) fn instruction_at(
    version: PesVersion,
    tactics: &TeamTactics,
    preset: u8,
    side: InstructionSide,
    index: u8,
) -> Option<Instruction> {
    let preset = tactics.presets.get(usize::from(preset))?;
    let entries = match side {
        InstructionSide::Attack => &preset.attack_instructions,
        InstructionSide::Defence => &preset.defence_instructions,
    };
    let entry = entries.as_ref()?.get(usize::from(index))?;
    instruction::decode(version, entry.instruction)
        .ok()
        .flatten()
}

/// The target chip's text under the instruction in force: "An opponent"
/// for a held Tight Marking (its byte is the opposing formation's slot,
/// not one of ours), "No target" under an instruction that takes none
/// (the game reads no target there), "Unassigned" for an unset byte,
/// "<lineup index> <name>" for a starter and the roster slot's name
/// alone else — `target_chip`'s text and the unsaved-changes row's one
/// wording, so the two cannot drift.
pub(crate) fn target_text(
    instruction: Option<Instruction>,
    target: u8,
    starting_eleven: &[u8; 11],
    roster: &[RosterRow<'_>],
) -> String {
    match instruction {
        Some(Instruction::TightMarking) => "An opponent".to_owned(),
        Some(inst) if !inst.takes_player() => "No target".to_owned(),
        _ if target == NO_TARGET => "Unassigned".to_owned(),
        _ => match starting_eleven.iter().position(|slot| *slot == target) {
            Some(index) => format!("{} {}", index + 1, name_at(roster, target)),
            None => name_at(roster, target).to_owned(),
        },
    }
}

/// The instruction's text in the combo and its rows (the draft's
/// `INSTRUCTION_LABEL`; `Off` reads "Unassigned").
pub(super) fn instruction_label(instruction: Instruction) -> &'static str {
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
        Headless, button_labels, centre, chip_under, combo_row, entry_id, fixture_rows,
        fixture_tactics, named_assignments, pointer_at, roster_rows,
    };

    // plans/save_editor.md "Tactics editor", "What the controls write"
    // ("An advanced instruction's list") and the draft's
    // `instructionsFor`: an attack combo lists the version's stored
    // table in its stored order, the attacking instructions plus `Off`,
    // less Tight Marking; a defence combo the defending ones plus
    // `Off`, less Tight Marking. A defence slot holding an attacking
    // instruction still shows it as the current choice.
    #[test]
    fn each_combo_lists_its_side_less_tight_marking() {
        let mut window = Headless::sized(800.0, 1200.0);
        for version in [PesVersion::Pes17, PesVersion::Pes19, PesVersion::Pes21] {
            let mut tactics = fixture_tactics(version);
            // Every chip shows a name, so no "Unassigned" chip counts
            // as an offered item below; defence 2 holds an attacking
            // instruction, kept as its current choice.
            named_assignments(&mut tactics);
            // Every instruction `Off` but defence 2's held attacking
            // one, so the target chips read "No target" and cannot
            // count as the combos' "Unassigned" item below.
            let encode = |inst| instruction::encode(version, inst).expect("the version stores it");
            for entry in tactics.presets[0]
                .attack_instructions
                .as_mut()
                .expect("instructions")
                .iter_mut()
                .chain(
                    tactics.presets[0]
                        .defence_instructions
                        .as_mut()
                        .expect("instructions")
                        .iter_mut(),
                )
            {
                entry.instruction = encode(Instruction::Off);
            }
            tactics.presets[0]
                .defence_instructions
                .as_mut()
                .expect("instructions")[1]
                .instruction = encode(Instruction::HugTheTouchline);
            let mut view = TacticsView::default();
            let rows = fixture_rows(version);
            let roster = roster_rows(&rows);
            for (column, side, index, wrong_side) in [
                (0, InstructionSide::Attack, 0, false),
                (1, InstructionSide::Defence, 0, false),
                (1, InstructionSide::Defence, 1, true),
            ] {
                let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
                let combo = combo_row(&nodes, index)[column];
                click(
                    &mut window,
                    combo,
                    version,
                    &mut tactics,
                    &mut view,
                    &roster,
                );
                // The popup's first frame is a sizing pass; egui 0.37
                // hides invisible widgets from the AccessKit tree, so
                // its items join from the next frame on.
                let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
                // The popup's buttons, unfiltered: offering Tight
                // Marking must fail here, not inside the `labels`
                // filter below (a filter that already excludes it
                // would hide the offer).
                assert!(
                    !nodes.iter().any(|node| {
                        node.role() == Role::Button && node.label() == Some("Tight Marking")
                    }),
                    "{version:?} {side:?} does not offer Tight Marking"
                );
                let mut expected: Vec<&str> = offered(version, side)
                    .iter()
                    .map(|inst| instruction_label(*inst))
                    .collect();
                if wrong_side {
                    expected.push("Hug the Touchline");
                }
                let labels: std::collections::BTreeSet<&str> = expected.iter().copied().collect();
                let drawn = button_labels(&nodes, |label| labels.contains(label));
                assert_eq!(
                    drawn.len(),
                    expected.len(),
                    "{version:?} {side:?}'s item count, got {drawn:?}"
                );
                assert_eq!(
                    drawn, expected,
                    "{version:?} {side:?}'s items in stored order"
                );
                if wrong_side {
                    assert_eq!(
                        combo_row(&nodes, index)[column].value(),
                        Some("Hug the Touchline"),
                        "{version:?} a held wrong-side instruction shows selected"
                    );
                }
                // Close the combo again for the next case's frame.
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
    }

    // plans/save_editor.md "Tactics editor" (the `.tacinstr` grid):
    // the headers Attack and Defence left-aligned over their columns,
    // each instruction's two combos on one y and equal in width, and
    // each target chip its combo's own x and width, 3 under it (the
    // draft's `column-gap: 8px; row-gap: 3px`).
    #[test]
    fn the_instructions_grid_is_two_equal_columns() {
        // The settings column's width as the window's: the draft's
        // 414 panel holds a 400-wide content.
        let mut window = Headless::sized(414.0, 1200.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let bounds = |node: &egui::accesskit::Node| node.bounds().expect("bounds");

        for row in 0..2 {
            let [attack, defence] = combo_row(&nodes, row);
            let (a, d) = (bounds(attack), bounds(defence));
            assert!(
                (a.y0 - d.y0).abs() < 0.5,
                "row {row}'s combos share a y ({a:?} vs {d:?})"
            );
            assert!(
                ((a.x1 - a.x0) - (d.x1 - d.x0)).abs() <= 0.5,
                "row {row}'s columns are equal ({a:?} vs {d:?})"
            );
            for (column, combo) in [(0, attack), (1, defence)] {
                let chip = chip_under(&nodes, combo);
                let (c, k) = (bounds(combo), bounds(chip));
                assert!(
                    (k.x0 - c.x0).abs() <= 0.5,
                    "row {row} column {column}: the chip starts at its combo's x ({k:?} vs {c:?})"
                );
                assert!(
                    ((k.x1 - k.x0) - (c.x1 - c.x0)).abs() <= 0.5,
                    "row {row} column {column}: the chip is its combo's width ({k:?} vs {c:?})"
                );
                assert!(
                    (k.y0 - c.y1 - 3.0).abs() <= 0.5,
                    "row {row} column {column}: the chip sits 3 under its combo ({k:?} vs {c:?})"
                );
            }
        }
        // The headers are plain left-aligned cells at their columns' x.
        for (header, combo) in [
            ("Attack", combo_row(&nodes, 0)[0]),
            ("Defence", combo_row(&nodes, 0)[1]),
        ] {
            let header_bounds = nodes
                .iter()
                .find(|node| node.role() == Role::Label && node.value() == Some(header))
                .and_then(|node| node.bounds())
                .unwrap_or_else(|| panic!("no {header} header"));
            assert!(
                (header_bounds.x0 - bounds(combo).x0).abs() <= 0.5,
                "{header} starts at its column's x ({header_bounds:?} vs {:?})",
                bounds(combo)
            );
        }
    }

    // The plan bullet's disabled-target rule and the draft's `pick`:
    // under an instruction that takes none of ours the chip is
    // disabled and names no player of ours — "An opponent" under a
    // held Tight Marking (its byte is the opposing formation's slot),
    // "No target" under a target-less one.
    #[test]
    fn a_target_less_or_opponent_target_shows_no_player_of_ours() {
        let mut window = Headless::sized(800.0, 1200.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let attack = tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions");
        attack[0].instruction =
            instruction::encode(version, Instruction::TightMarking).expect("stored");
        attack[0].player = 4;
        attack[1].instruction =
            instruction::encode(version, Instruction::TikiTaka).expect("stored");
        attack[1].player = 0;
        let defence = tactics.presets[0]
            .defence_instructions
            .as_mut()
            .expect("PES 21 has instructions");
        defence[0].instruction =
            instruction::encode(version, Instruction::CounterTarget).expect("stored");
        defence[0].player = 0xFF;
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);

        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let combos = |index| combo_row(&nodes, index);
        let chip1 = chip_under(&nodes, combos(0)[0]);
        assert!(chip1.is_disabled(), "the Tight Marking chip is disabled");
        assert_eq!(
            chip1.label(),
            Some("An opponent"),
            "a held Tight Marking's byte is the opponent's"
        );
        let chip2 = chip_under(&nodes, combos(1)[0]);
        assert!(chip2.is_disabled(), "the target-less chip is disabled");
        assert_eq!(
            chip2.label(),
            Some("No target"),
            "a target-less instruction names none"
        );
        // An unset target under a player-taking instruction reads
        // "Unassigned" (no "(empty)" name).
        let chip3 = chip_under(&nodes, combos(0)[1]);
        assert!(!chip3.is_disabled(), "the Counter Target chip is enabled");
        assert_eq!(
            chip3.label(),
            Some("Unassigned"),
            "an unset target under a player-taking instruction names none"
        );
        for chip in [chip1, chip2] {
            let shown = chip.label().expect("the chip's text");
            assert!(
                roster
                    .iter()
                    .all(|row| row.name.is_empty() || !shown.ends_with(row.name)),
                "{shown}: no roster name appears in a disabled chip"
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
        let combo = combo_row(&nodes, 0)[0];
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
        let combo = combo_row(&nodes, 0)[0];
        assert_eq!(combo.value(), Some("Unknown (0xAB)"));
    }

    // The pick itself: an instruction's combo writes the byte, and one
    // that takes no target clears the pick to 255, its chip disabled.
    // The chip under a player-taking one arms the target mode
    // (`Assign::Target`) — the pitch pick's write is the popover
    // tests'.
    #[test]
    fn the_instruction_pick_writes_the_byte_and_clears_the_target() {
        let mut window = Headless::sized(800.0, 1200.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        // Every chip shows a name: the counted buttons are the popup's.
        named_assignments(&mut tactics);
        // A non-identity eleven: an arm writing the position index for
        // the roster slot would look right on the fixture's [0, 1, ...].
        tactics.starting_eleven.swap(2, 9);
        let picked = tactics.starting_eleven[2];
        assert_ne!(picked, 2, "premise: slot and index differ");
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let combo = combo_row(&nodes, 0)[0];

        // Open the instruction combo and pick Defensive (an attacking
        // instruction that takes a player — Counter Target is
        // defending, and the attack combo does not offer it).
        let (_, _) = click(
            &mut window,
            combo,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let item = entry_id(&ids, Role::Button, "Defensive");
        let (_, changed) =
            window.action_click_tactics(item, version, &mut tactics, &mut view, &roster);
        assert!(changed, "the pick changed the record");
        let attack = &mut tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("instructions")[0];
        assert_eq!(
            attack.instruction,
            instruction::encode(version, Instruction::Defensive).expect("PES 21 stores Defensive"),
            "the pick writes the stored value"
        );

        // The target chip under it is enabled and arms the mode. The
        // popup stays open past an AccessKit pick (no `any_click`), so a
        // pointer click on empty space closes it first.
        window.tactics(
            version,
            &mut tactics,
            &mut view,
            &roster,
            pointer_at(egui::pos2(5.0, 5.0), true),
        );
        window.tactics(
            version,
            &mut tactics,
            &mut view,
            &roster,
            pointer_at(egui::pos2(5.0, 5.0), false),
        );
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        assert!(!chip.is_disabled(), "Defensive enables the chip");
        let (_, changed) = click(&mut window, chip, version, &mut tactics, &mut view, &roster);
        assert!(!changed, "arming writes no record");
        assert_eq!(
            view.assign,
            Some(Assign::Target {
                side: InstructionSide::Attack,
                index: 0,
            }),
            "the mode is armed"
        );

        // Pick Tiki-Taka (attacking, no target): the byte changes, the
        // target clears to 255 and its chip disables.
        view.assign = None;
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let combo = combo_row(&nodes, 0)[0];
        let (_, _) = click(
            &mut window,
            combo,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
        let item = entry_id(&ids, Role::Button, "Tiki-Taka");
        let (_, changed) =
            window.action_click_tactics(item, version, &mut tactics, &mut view, &roster);
        assert!(changed, "the pick changed the record");
        let attack = &tactics.presets[0]
            .attack_instructions
            .as_ref()
            .expect("instructions")[0];
        assert_eq!(
            attack.instruction,
            instruction::encode(version, Instruction::TikiTaka).expect("PES 21 stores Tiki-Taka")
        );
        assert_eq!(
            attack.player, NO_TARGET,
            "a target-less pick clears the target"
        );
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        assert!(chip.is_disabled(), "Gegenpress disables the chip");
        assert_eq!(chip.label(), Some("No target"), "and it shows No target");
    }

    // The four rows route to their own entries: an instruction and a
    // target picked on one row lands on that row's entry alone — the
    // other three rows, the other presets and every other field stay.
    #[test]
    fn each_instruction_row_writes_its_own_entry() {
        let mut window = Headless::sized(800.0, 900.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        // The four rows start distinct and a non-identity eleven (a
        // roster slot that equals its index would hide an index write).
        let encode = |inst| instruction::encode(version, inst).expect("PES 21 stores it");
        let rows_init = [
            Instruction::HugTheTouchline,
            Instruction::WingRotation,
            Instruction::DeepDefensiveLine,
            Instruction::Gegenpress,
        ];
        let attack = &mut tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions");
        attack[0].instruction = encode(rows_init[0]);
        attack[0].player = 0;
        attack[1].instruction = encode(rows_init[1]);
        attack[1].player = 1;
        let defence = &mut tactics.presets[0]
            .defence_instructions
            .as_mut()
            .expect("PES 21 has instructions");
        defence[0].instruction = encode(rows_init[2]);
        defence[0].player = 2;
        defence[1].instruction = encode(rows_init[3]);
        defence[1].player = 3;
        tactics.starting_eleven.swap(2, 9);
        tactics.starting_eleven.swap(3, 10);
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);

        for (index, (column, side, inst, pick)) in [
            (
                0usize,
                InstructionSide::Attack,
                Instruction::Defensive,
                2_usize,
            ),
            (0, InstructionSide::Attack, Instruction::Defensive, 3_usize),
            (
                1,
                InstructionSide::Defence,
                Instruction::CounterTarget,
                4_usize,
            ),
            (
                1,
                InstructionSide::Defence,
                Instruction::CounterTarget,
                5_usize,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let row = index % 2;
            let before = tactics.clone();
            let mut expected = before.clone();
            let entries = if index < 2 {
                &mut expected.presets[0]
                    .attack_instructions
                    .as_mut()
                    .expect("instructions")
            } else {
                &mut expected.presets[0]
                    .defence_instructions
                    .as_mut()
                    .expect("instructions")
            };
            let entry = &mut entries[row];
            entry.instruction = encode(inst);
            entry.player = tactics.starting_eleven[pick];
            assert!(
                inst.takes_player(),
                "premise: the picked instruction takes a player"
            );

            // Pick the instruction on this row's combo.
            let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
            let combo = combo_row(&nodes, row)[column];
            let (_, _) = click(
                &mut window,
                combo,
                version,
                &mut tactics,
                &mut view,
                &roster,
            );
            let (ids, _) = window.tactics_ids(version, &mut tactics, &mut view, &roster, vec![]);
            let item = entry_id(&ids, Role::Button, instruction_label(inst));
            let (_, changed) =
                window.action_click_tactics(item, version, &mut tactics, &mut view, &roster);
            assert!(changed, "{side:?} {row}: the pick changed the record");

            // Then the chip's pick: arm it, the pitch's write puts the
            // starter's roster slot there and disarms. The popup stays
            // open past an AccessKit pick — a pointer click on empty
            // space closes it first.
            window.tactics(
                version,
                &mut tactics,
                &mut view,
                &roster,
                pointer_at(egui::pos2(5.0, 5.0), true),
            );
            window.tactics(
                version,
                &mut tactics,
                &mut view,
                &roster,
                pointer_at(egui::pos2(5.0, 5.0), false),
            );
            let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
            let chip = chip_under(&nodes, combo_row(&nodes, row)[column]);
            assert!(!chip.is_disabled(), "{side:?} {row}: the chip is enabled");
            let (_, changed) = click(&mut window, chip, version, &mut tactics, &mut view, &roster);
            assert!(!changed, "{side:?} {row}: arming writes no record");
            Assign::Target { side, index: row }.write(version, &mut tactics, &mut view, pick);
            assert_eq!(
                view.assign, None,
                "{side:?} {row}: disarmed after the write"
            );

            assert_eq!(
                tactics, expected,
                "{side:?} {row}: only that row's entry changed"
            );
        }
    }

    // The Target chip's text: a starter as his lineup position and
    // name, a bench player as the name alone (`target_text`'s
    // `starting_eleven.position`), "(empty)" for a roster slot that
    // holds nobody.
    #[test]
    fn the_target_chip_shows_the_starters_position_or_the_name_alone() {
        let mut window = Headless::sized(800.0, 1200.0);
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
        let rows: Vec<(String, String, Option<u8>)> = (0..32)
            .map(|slot| (format!("Player {slot}"), format!("Player {slot}"), Some(0)))
            .collect();
        let roster = roster_rows(&rows);

        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        let expected = format!("4 Player {starter}");
        assert_eq!(
            chip.label(),
            Some(expected.as_str()),
            "a starter shows his lineup position and name"
        );

        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0]
            .player = bench;
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        let expected = format!("Player {bench}");
        assert_eq!(
            chip.label(),
            Some(expected.as_str()),
            "a bench player shows the name alone"
        );
    }

    // `roster` shorter than a slot, or an empty one, shows "(empty)" as
    // the chip's text under a player-taking instruction.
    #[test]
    fn a_short_or_empty_roster_slice_shows_empty_on_the_chip() {
        let mut window = Headless::sized(800.0, 1200.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0]
            .instruction =
            instruction::encode(version, Instruction::CounterTarget).expect("stored");
        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("instructions")[0]
            .player = tactics.starting_eleven[2];
        let mut view = TacticsView::default();
        // Three rows for forty roster slots: slot 2, an empty name,
        // shows "(empty)".
        let rows = [
            ("ALPHA".to_owned(), "ALPHA".to_owned(), Some(0)),
            ("BETA".to_owned(), "BETA".to_owned(), Some(0)),
            (String::new(), String::new(), None),
        ];
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        assert_eq!(
            chip.label(),
            Some("3 (empty)"),
            "an empty name shows (empty)"
        );

        tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("instructions")[0]
            .player = 40;
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        assert_eq!(
            chip.label(),
            Some("(empty)"),
            "a slot past the slice shows (empty)"
        );
    }

    // The chip's own elide: a selected name too wide for its column
    // truncates, and the closed chip's hover shows it whole (the
    // `rolechip`'s `title` is the pick's hint, so a name that does not
    // fit is the hover instead). The target cell is the grid's column,
    // ~183 wide at this pane; a real player's name can be wider.
    #[test]
    fn an_elided_target_shows_the_whole_name_on_hover() {
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        // Counter Target takes a player; the ninth starter's name,
        // made wider than the chip's column, elides.
        let ninth = tactics.starting_eleven[8];
        let attack = &mut tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0];
        attack.instruction =
            instruction::encode(version, Instruction::CounterTarget).expect("stored");
        attack.player = ninth;
        let mut view = TacticsView::default();
        let selected = "9 3DSMAX HAS STOPPED WORKING".to_owned();
        let rows: Vec<(String, String, Option<u8>)> = (0..32)
            .map(|slot| {
                let name = if slot == usize::from(ninth) {
                    "3DSMAX HAS STOPPED WORKING".to_owned()
                } else {
                    format!("Player {slot}")
                };
                (name.clone(), name, Some(0))
            })
            .collect();
        let roster = roster_rows(&rows);
        // The settings column's width as the window's (the real ~190
        // half at 414): the name is wider.
        let mut window = Headless::sized(414.0, 1200.0);
        window
            .ctx
            .global_style_mut(|style| style.interaction.tooltip_delay = 0.0);

        let mut measured = 0.0f32;
        window.frame(vec![], |ui| measured = text_width(ui, &selected));
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        let width = chip
            .bounds()
            .map(|b| b.x1 - b.x0)
            .expect("the chip's bounds");
        assert!(
            f64::from(measured) > width - 12.0,
            "premise: {selected:?} elides inside the chip ({measured} vs {width})"
        );
        assert_eq!(
            window.tactics_tooltip(
                centre(chip),
                &selected,
                version,
                &mut tactics,
                &mut view,
                &roster,
            ),
            Some(selected.clone()),
            "the pointer resting on the chip shows the whole name"
        );
    }

    // The check's other side: a name that fits the chip's text area —
    // its hover is the pick's hint, the draft's title, not the name.
    #[test]
    fn a_fitting_target_hovers_the_pick_hint() {
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let attack = &mut tactics.presets[0]
            .attack_instructions
            .as_mut()
            .expect("PES 21 has instructions")[0];
        attack.instruction =
            instruction::encode(version, Instruction::CounterTarget).expect("stored");
        attack.player = 30;
        tactics.starting_eleven[0] = 30;
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        // Premise: the fixture leaves roster slot 30 without a player.
        assert_eq!(name_at(&roster, 30), "(empty)");
        let mut window = Headless::sized(800.0, 1200.0);
        window
            .ctx
            .global_style_mut(|style| style.interaction.tooltip_delay = 0.0);

        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let chip = chip_under(&nodes, combo_row(&nodes, 0)[0]);
        assert_eq!(chip.label(), Some("1 (empty)"), "the chip's text");
        assert_eq!(
            window.tactics_tooltip(
                centre(chip),
                "The instruction's target",
                version,
                &mut tactics,
                &mut view,
                &roster,
            ),
            Some("The instruction's target: click, then a pitch player".to_owned()),
            "a chip that fits hovers the pick's hint"
        );
    }
}
