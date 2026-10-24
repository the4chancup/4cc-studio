//! The "Player assignments" palette and the armed click-to-assign modes
//! (`plans/save_editor.md` "Tactics editor", the click-to-assign bullet
//! and "What the controls write"): the six set-piece takers, the captain
//! and the join-attack pick as chips that arm a pitch click, and the note
//! a host draws while one is armed.

use pes_savefile::model::tactics::{SetPieceTakers, TeamTactics};

use super::{RosterRow, TacticsView, name_at};

/// The armed click-to-assign mode (the draft's `S.assign`): which field
/// the next pitch click writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Assign {
    /// One of `SET_PIECES`' roles (the six takers, then the captain): the
    /// click writes the clicked starter's roster slot and disarms.
    Role(usize),
    /// The players joining the attack: each click toggles the clicked
    /// starter's membership; armed until Done.
    Join,
}

impl Assign {
    /// A click on starter `index` (a `starting_eleven` index) while armed
    /// (the draft's `assignClick`): a role writes the starter's roster
    /// slot — a same-player write is skipped — and disarms; join toggles
    /// the starter's membership, a member leaving packs the list (the
    /// later entries move up and 255 fills the end), a non-member fills
    /// the first 255 and a fourth on a full list is ignored; join stays
    /// armed.
    pub(crate) fn write(self, tactics: &mut TeamTactics, view: &mut TacticsView, index: usize) {
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
        }
    }
}

/// The pitch marker whose popover is open, with what it opened on:
/// `pitch` keeps the popover only while `scope` (the widget's `egui::Id`,
/// which the host's pushed id keys to the team and the open), `preset`
/// and `formation` are still what they were at open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OpenSlot {
    /// The pitch widget's `ui.scope_id()` at open.
    pub(crate) scope: egui::Id,
    /// `view.preset` at open.
    pub(crate) preset: usize,
    /// `view.formation` at open.
    pub(crate) formation: usize,
    /// The formation slot, 0 to 10.
    pub(crate) slot: usize,
}

/// One chip of the "Player assignments" group (the draft's `SET_PIECES`):
/// the role's label and the `SetPieceTakers` field it edits — the captain
/// is `set_pieces.captain` in this model.
struct RoleSpec {
    /// The chip's label.
    label: &'static str,
    /// The stored holder (a roster slot, 255 none).
    get: fn(&SetPieceTakers) -> u8,
    /// The stored write.
    set: fn(&mut SetPieceTakers, u8),
}

/// The six set-piece takers and the captain, in the draft's order.
const SET_PIECES: [RoleSpec; 7] = [
    RoleSpec {
        label: "Long free kick",
        get: |pieces| pieces.free_kick_long,
        set: |pieces, slot| pieces.free_kick_long = slot,
    },
    RoleSpec {
        label: "Short free kick",
        get: |pieces| pieces.free_kick_short,
        set: |pieces, slot| pieces.free_kick_short = slot,
    },
    RoleSpec {
        label: "Second free kick",
        get: |pieces| pieces.free_kick_second,
        set: |pieces, slot| pieces.free_kick_second = slot,
    },
    RoleSpec {
        label: "Left corner",
        get: |pieces| pieces.corner_left,
        set: |pieces, slot| pieces.corner_left = slot,
    },
    RoleSpec {
        label: "Right corner",
        get: |pieces| pieces.corner_right,
        set: |pieces, slot| pieces.corner_right = slot,
    },
    RoleSpec {
        label: "Penalty kick",
        get: |pieces| pieces.penalty,
        set: |pieces, slot| pieces.penalty = slot,
    },
    RoleSpec {
        label: "Captain",
        get: |pieces| pieces.captain,
        set: |pieces, slot| pieces.captain = slot,
    },
];

/// The "Player assignments" group (the draft's role palette): one chip
/// per role — "<label>: <holder's name or Unassigned>" — and the
/// join-attack chip listing its members' names, "none" when empty. A
/// click arms the mode and closes the slot popover; a click on the armed
/// chip disarms it (the draft's `roleChip`/`joinChip`).
pub(super) fn player_assignments(
    ui: &mut egui::Ui,
    set_pieces: &SetPieceTakers,
    joined: &[u8; 3],
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
) {
    ui.strong("Player assignments");
    ui.horizontal_wrapped(|ui| {
        for (index, role) in SET_PIECES.iter().enumerate() {
            let holder = (role.get)(set_pieces);
            let name = if holder == u8::MAX {
                "Unassigned".to_owned()
            } else {
                name_at(roster, holder).to_owned()
            };
            let armed = view.assign == Some(Assign::Role(index));
            if ui
                .selectable_label(armed, format!("{}: {}", role.label, name))
                .on_hover_text("Click, then a pitch player")
                .clicked()
            {
                view.assign = (!armed).then_some(Assign::Role(index));
                view.popover = None;
            }
        }
        // The join chip lists the members' first names, as the draft does.
        let members: Vec<&str> = joined
            .iter()
            .filter(|&&slot| slot != u8::MAX)
            .map(|&slot| {
                name_at(roster, slot)
                    .split_whitespace()
                    .next()
                    .unwrap_or("(empty)")
            })
            .collect();
        let armed = view.assign == Some(Assign::Join);
        let label = if members.is_empty() {
            "Join attack: none".to_owned()
        } else {
            format!("Join attack: {}", members.join(", "))
        };
        if ui
            .selectable_label(armed, label)
            .on_hover_text("Click, then up to three pitch players")
            .clicked()
        {
            view.assign = (!armed).then_some(Assign::Join);
            view.popover = None;
        }
    });
}

/// The armed mode's note (the draft's `assignnote`): what the next pitch
/// click does, and Cancel — join's Done — disarming it. A host draws it
/// inside the settings column, under the preset bar (the draft's `note`
/// in `tac-settings`).
pub fn assign_note(ui: &mut egui::Ui, view: &mut TacticsView) {
    let Some(assign) = view.assign else {
        return;
    };
    ui.horizontal_wrapped(|ui| match assign {
        Assign::Role(index) => {
            ui.label(format!(
                "Click the pitch player for {}.",
                SET_PIECES[index].label.to_lowercase()
            ));
            if ui.button("Cancel").clicked() {
                view.assign = None;
            }
        }
        Assign::Join => {
            ui.label("Click pitch players to join or leave the attack (up to three).");
            if ui.button("Done").clicked() {
                view.assign = None;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use egui::accesskit::{self, Role};

    use pes_version::PesVersion;

    use super::*;
    use crate::tactics::tests::click;
    use crate::test_support::{Headless, fixture_rows, fixture_tactics, roster_rows};

    // The draft's "Player assignments" palette: one chip per role naming
    // its holder ("Unassigned" when none) and the join chip's members
    // ("none" when empty); a click arms and disarms the mode.
    #[test]
    fn the_assignment_chips_name_their_holders_and_toggle_the_mode() {
        let mut window = Headless::sized(1400.0, 900.0);
        let version = PesVersion::Pes21;
        let mut tactics = fixture_tactics(version);
        let mut view = TacticsView::default();
        let rows = fixture_rows(version);
        let roster = roster_rows(&rows);
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        for role in &SET_PIECES {
            let holder = (role.get)(&tactics.set_pieces);
            let expected = format!("{}: {}", role.label, name_at(&roster, holder));
            assert!(
                nodes
                    .iter()
                    .any(|node| node.role() == Role::Button && node.label() == Some(&*expected)),
                "no chip {expected:?}"
            );
        }
        assert!(
            nodes.iter().any(
                |node| node.role() == Role::Button && node.label() == Some("Join attack: none")
            ),
            "the join chip reads none when empty"
        );

        // An unassigned role reads "Unassigned".
        tactics.set_pieces.free_kick_long = u8::MAX;
        let (nodes, _) = window.tactics(version, &mut tactics, &mut view, &roster, vec![]);
        assert!(
            nodes.iter().any(|node| node.role() == Role::Button
                && node.label() == Some("Long free kick: Unassigned")),
            "an unassigned role reads Unassigned"
        );

        // A click arms, the armed chip shows selected, a second click
        // disarms.
        let penalty = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Button
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Penalty kick:"))
            })
            .expect("the penalty chip");
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
        let penalty = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Button
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Penalty kick:"))
            })
            .expect("the penalty chip");
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
    }
}
