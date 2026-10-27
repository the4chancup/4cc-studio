//! The "Player assignments" palette and the armed click-to-assign modes
//! (`plans/save_editor.md` "Tactics editor", the click-to-assign bullet
//! and "What the controls write"): the six set-piece takers, the captain
//! and the join-attack pick as chips that arm a pitch click. No note
//! explains an armed mode: the highlighted chip and the candidates'
//! corner brackets on the pitch say it.

use pes_savefile::model::tactics::{SetPieceTakers, TeamTactics};
use pes_savefile::schema::fields::{InstructionSide, TacticsField};
use pes_savefile::schema::instruction;
use pes_version::PesVersion;

use super::{RosterRow, TacticsView, name_at};

/// The armed click-to-assign mode (the draft's `S.assign`): which field
/// the next pitch click writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Assign {
    /// One of `SET_PIECES`' roles (the six takers, then the captain): the
    /// click writes the clicked starter's roster slot and disarms.
    Role(usize),
    /// The players joining the attack: each click toggles the clicked
    /// starter's membership; armed until the chip's click disarms.
    Join,
    /// An instruction's target-player pick (the draft's `S.assign.kind
    /// === "target"`): the click writes the clicked starter's roster
    /// slot and disarms.
    Target {
        /// Which side's instruction list (`attack_instructions` or
        /// `defence_instructions`).
        side: InstructionSide,
        /// The instruction's index in it (0 or 1).
        index: usize,
    },
}

impl Assign {
    /// A click on starter `index` (a `starting_eleven` index) while armed
    /// (the draft's `assignClick`): a role writes the starter's roster
    /// slot — a same-player write is skipped — and disarms; join toggles
    /// the starter's membership, a member leaving packs the list (the
    /// later entries move up and 255 fills the end), a non-member fills
    /// the first 255 and a fourth on a full list is ignored; join stays
    /// armed; a target writes the starter's roster slot and disarms, but
    /// only while that slot's instruction still takes a player (an
    /// instruction changed while armed writes nothing and disarms).
    /// `version` is the save's (the target's instruction decode).
    pub(crate) fn write(
        self,
        version: PesVersion,
        tactics: &mut TeamTactics,
        view: &mut TacticsView,
        index: usize,
    ) {
        let slot = tactics.starting_eleven[index];
        match self {
            Assign::Role(role) => {
                let role = &SET_PIECES[role];
                if (role.get)(&tactics.set_pieces) != slot {
                    (role.set)(&mut tactics.set_pieces, slot);
                }
                view.assign = None;
            }
            Assign::Join => {
                let list = &mut tactics.players_to_join_attack;
                if let Some(at) = list.iter().position(|&entry| entry == slot) {
                    list.copy_within(at + 1.., at);
                    list[2] = u8::MAX;
                } else if let Some(free) = list.iter().position(|&entry| entry == u8::MAX) {
                    list[free] = slot;
                }
            }
            Assign::Target { side, index } => {
                let preset = &mut tactics.presets[view.preset.min(2)];
                let entries = match side {
                    InstructionSide::Attack => preset.attack_instructions.as_mut(),
                    InstructionSide::Defence => preset.defence_instructions.as_mut(),
                };
                if let Some(entry) = entries.and_then(|entries| entries.get_mut(index))
                    && entry.player != slot
                    && instruction::decode(version, entry.instruction)
                        .ok()
                        .flatten()
                        .is_some_and(|inst| inst.takes_player())
                {
                    entry.player = slot;
                }
                view.assign = None;
            }
        }
    }
}

/// The pitch marker whose popover is open, with what it opened on:
/// `pitch` keeps the popover only while `scope` (the widget's `egui::Id`,
/// which the host's pushed id keys to the team and the open), `preset`
/// and `formation` are still what they were at open.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OpenSlot {
    /// The pitch widget's `ui.scope_id()` at open.
    pub(crate) scope: egui::Id,
    /// `view.preset` at open.
    pub(crate) preset: usize,
    /// `view.formation` at open.
    pub(crate) formation: usize,
    /// The formation slot, 0 to 10.
    pub(crate) slot: usize,
    /// The marker's bottom-left corner as it stood at the click,
    /// relative to the canvas' min corner: the popover's top-right stays
    /// on it while the slot's X and Y move the marker, so the fields'
    /// arrows stay under the pointer — and it scrolls with the pitch
    /// (canvas-relative, never a screen point).
    pub(crate) at: egui::Vec2,
}

impl OpenSlot {
    /// The popover's `egui::Area` id, its layer's too: `slot_popover`
    /// shows it under this id.
    pub(crate) fn area_id(&self) -> egui::Id {
        self.scope.with("slot popover")
    }
}

/// One chip of the "Player assignments" group (the draft's `SET_PIECES`):
/// the role's label and the `SetPieceTakers` field it edits — the captain
/// is `set_pieces.captain` in this model.
pub(super) struct RoleSpec {
    /// The chip's label.
    pub(super) label: &'static str,
    /// The tactics field the chip writes (`tactics_term`'s key into the
    /// table, so a row's label is never a second copy).
    pub(super) field: TacticsField,
    /// The stored holder (a roster slot, 255 none).
    get: fn(&SetPieceTakers) -> u8,
    /// The stored write.
    set: fn(&mut SetPieceTakers, u8),
}

/// The six set-piece takers and the captain, in the draft's order.
pub(super) const SET_PIECES: [RoleSpec; 7] = [
    RoleSpec {
        label: "Long free kick",
        field: TacticsField::FreeKickTakerLong,
        get: |pieces| pieces.free_kick_long,
        set: |pieces, slot| pieces.free_kick_long = slot,
    },
    RoleSpec {
        label: "Short free kick",
        field: TacticsField::FreeKickTakerShort,
        get: |pieces| pieces.free_kick_short,
        set: |pieces, slot| pieces.free_kick_short = slot,
    },
    RoleSpec {
        label: "Second free kick",
        field: TacticsField::FreeKickTakerSecond,
        get: |pieces| pieces.free_kick_second,
        set: |pieces, slot| pieces.free_kick_second = slot,
    },
    RoleSpec {
        label: "Left corner",
        field: TacticsField::CornerTakerLeft,
        get: |pieces| pieces.corner_left,
        set: |pieces, slot| pieces.corner_left = slot,
    },
    RoleSpec {
        label: "Right corner",
        field: TacticsField::CornerTakerRight,
        get: |pieces| pieces.corner_right,
        set: |pieces, slot| pieces.corner_right = slot,
    },
    RoleSpec {
        label: "Penalty kick",
        field: TacticsField::PenaltyTaker,
        get: |pieces| pieces.penalty,
        set: |pieces, slot| pieces.penalty = slot,
    },
    RoleSpec {
        label: "Captain",
        field: TacticsField::Captain,
        get: |pieces| pieces.captain,
        set: |pieces, slot| pieces.captain = slot,
    },
];

/// The role chip's label for `field`, `None` for a field that is no role:
/// `tactics_term` reads the chips' own table, never a second copy of it.
pub(super) fn role_label(field: TacticsField) -> Option<&'static str> {
    SET_PIECES
        .iter()
        .find(|role| role.field == field)
        .map(|role| role.label)
}

/// The "Player assignments" group's rows (the draft's `#tac-side
/// .tachalf`): one column of `.tset` rows 3 points apart — each of the
/// six takers and the captain its label over a chip the column's width,
/// then "Join the attack" over three chips, one per joining player. A
/// chip click arms its mode and closes the slot popover; a click on an
/// armed chip disarms it (the draft's `roleChip`/`joinChip`), and any
/// of the three join chips arms the one join mode, all three filled
/// while it is armed. The panel's heading is the caller's `panel`.
pub(super) fn player_assignments(
    ui: &mut egui::Ui,
    set_pieces: &SetPieceTakers,
    joined: &[u8; 3],
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) {
    // The frame fills the cell the host gives the panel (the pitch's
    // height), as `formation_settings`' does.
    ui.set_min_height(ui.available_height());
    // `.tachalf`: rows 3 points apart; inside a `.tset`, the label 2
    // above its chip.
    ui.spacing_mut().item_spacing.y = 2.0;
    for (index, role) in SET_PIECES.iter().enumerate() {
        ui.label(role.label);
        let holder = (role.get)(set_pieces);
        let name = if holder == u8::MAX {
            "Unassigned".to_owned()
        } else {
            name_at(roster, holder).to_owned()
        };
        let armed = view.assign == Some(Assign::Role(index));
        if chip(ui, &name, armed, "Click, then a pitch player").clicked() {
            view.assign = (!armed).then_some(Assign::Role(index));
            view.popover = None;
        }
        // 3 points to the next row.
        ui.add_space(1.0);
    }
    // "Join the attack" shows its three players as three rows, one per
    // joining player — `Unassigned` where fewer join — all three the
    // one picker's chips.
    ui.label("Join the attack");
    let join_armed = view.assign == Some(Assign::Join);
    let members: Vec<u8> = joined
        .iter()
        .copied()
        .filter(|&slot| slot != u8::MAX)
        .collect();
    for k in 0..3 {
        let name = members
            .get(k)
            .map(|&slot| name_at(roster, slot).to_owned())
            .unwrap_or_else(|| "Unassigned".to_owned());
        if chip(
            ui,
            &name,
            join_armed,
            "Click, then up to three pitch players",
        )
        .clicked()
        {
            view.assign = (!join_armed).then_some(Assign::Join);
            view.popover = None;
        }
    }
}

/// A chip's padding (the draft's `.rolechip` `padding: 1px 6px`).
const CHIP_PADDING: egui::Vec2 = egui::vec2(6.0, 1.0);

/// One chip of the palette (the draft's `.rolechip`): the column's
/// width, the holder's name left-aligned and cut with an ellipsis at
/// the edge — the full name its hover then, `hint` when it fits.
pub(super) fn chip(
    ui: &mut egui::Ui,
    name: &str,
    armed: bool,
    hint: &'static str,
) -> egui::Response {
    ui.spacing_mut().button_padding = CHIP_PADDING;
    let width = ui.available_width();
    let response = ui.add(
        egui::Button::selectable(armed, name.to_owned())
            .truncate()
            .small()
            .min_size(egui::vec2(width, 0.0)),
    );
    let chrome = 2.0 * ui.spacing().button_padding.x;
    if super::text_width(ui, name) > width - chrome {
        response.on_hover_text(name.to_owned())
    } else {
        response.on_hover_text(hint)
    }
}

#[cfg(test)]
mod tests {
    use egui::accesskit;

    use pes_version::PesVersion;

    use super::*;
    use crate::tactics::tests::click;
    use crate::test_support::{Headless, chips_below, fixture_rows, fixture_tactics, roster_rows};

    // The draft's "Player assignments" column (`.tachalf`): each of the
    // six takers and the captain its label over a chip showing the
    // holder's name ("Unassigned" when none), then "Join the attack"
    // over three chips, one per joining player; a click arms, an armed
    // click disarms, and all three join chips show armed together.
    #[test]
    fn the_assignment_rows_name_their_holders_and_arm_the_modes() {
        let mut window = Headless::sized(1400.0, 1200.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        tactics.players_to_join_attack = [tactics.starting_eleven[2], u8::MAX, u8::MAX];
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        for role in &SET_PIECES {
            let holder = (role.get)(&tactics.set_pieces);
            let expected = if holder == u8::MAX {
                "Unassigned".to_owned()
            } else {
                name_at(&roster, holder).to_owned()
            };
            // The chip directly under the label (the next role's chip
            // is a label further down).
            let chip = chips_below(&nodes, role.label)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("{}: no chip under its label", role.label));
            assert_eq!(
                chip.label(),
                Some(expected.as_str()),
                "{}'s chip names its holder",
                role.label
            );
        }
        // "Join the attack": one chip per entry — a name where one
        // joins, "Unassigned" where none.
        let joined: Vec<String> = tactics
            .players_to_join_attack
            .iter()
            .filter(|&&slot| slot != u8::MAX)
            .map(|&slot| name_at(&roster, slot).to_owned())
            .collect();
        let expected: Vec<String> = (0..3)
            .map(|k| {
                joined
                    .get(k)
                    .cloned()
                    .unwrap_or_else(|| "Unassigned".to_owned())
            })
            .collect();
        let chips = chips_below(&nodes, "Join the attack");
        assert_eq!(chips.len(), 3, "three join chips");
        for (chip, expected) in chips.iter().zip(&expected) {
            assert_eq!(chip.label(), Some(expected.as_str()));
        }

        // A role chip click arms its mode and closes the slot popover;
        // the armed chip shows selected; a second click disarms.
        let penalty = chips_below(&nodes, "Penalty kick")[0];
        assert_eq!(
            penalty.toggled(),
            Some(accesskit::Toggled::False),
            "starts unarmed"
        );
        let (_, changed) = click(
            &mut window,
            penalty,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "arming writes no record");
        assert_eq!(view.assign, Some(Assign::Role(5)), "armed for the penalty");
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let penalty = chips_below(&nodes, "Penalty kick")[0];
        assert_eq!(
            penalty.toggled(),
            Some(accesskit::Toggled::True),
            "the armed chip is selected"
        );
        let (_, changed) = click(
            &mut window,
            penalty,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "disarming writes no record");
        assert_eq!(view.assign, None, "the second click disarms");

        // Any join chip arms the one join mode, all three selected
        // while armed; an armed click disarms.
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        let third = chips_below(&nodes, "Join the attack")[2];
        let (_, changed) = click(
            &mut window,
            third,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "arming writes no record");
        assert_eq!(view.assign, Some(Assign::Join), "join armed");
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        for chip in chips_below(&nodes, "Join the attack") {
            assert_eq!(
                chip.toggled(),
                Some(accesskit::Toggled::True),
                "all three chips show armed"
            );
        }
        let third = chips_below(&nodes, "Join the attack")[2];
        let (_, changed) = click(
            &mut window,
            third,
            version,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(!changed, "disarming writes no record");
        assert_eq!(view.assign, None, "the armed click disarms");
    }
}
