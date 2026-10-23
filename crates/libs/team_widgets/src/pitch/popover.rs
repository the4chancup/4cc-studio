//! The slot popover (`plans/save_editor.md` "Tactics editor", the pitch
//! bullet; the draft's `slotPopover`): a marker click's per-slot panel —
//! the starter's name and roster number, the Position combo, Make GK and
//! the X and Y fields.

use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::tactics::TeamTactics;

use super::{GRID_X, GRID_Y, badge_centre, marker_rect, position_label};
use crate::names::append_name;
use crate::tactics::{RosterRow, TacticsView, raw_name_at};

/// The slot popover (the draft's `slotPopover`): beside the marker, the
/// starter's name and roster number, the Position combo of the twelve
/// outfield positions, Make GK and the X
/// (0 to 104) and Y (0 to 48) fields and Close. The keeper's slot shows a
/// disabled "GK", the two fields disabled under "The goalkeeper's spot is
/// fixed" and Make GK disabled. Make GK exchanges the slot's
/// `starting_eleven` entry with the keeper slot's — the formation slots
/// stay — and closes the popover, as does Close; `pitch` also closes it
/// when the preset, the formation or the team changes.
pub(super) fn slot_popover(
    ui: &mut egui::Ui,
    tactics: &mut TeamTactics,
    view: &mut TacticsView,
    roster: &[RosterRow<'_>],
    canvas: egui::Rect,
) {
    let Some(open) = view.popover else {
        return;
    };
    let index = open.slot;
    let TeamTactics {
        presets,
        starting_eleven,
        ..
    } = tactics;
    let formation = &mut presets[open.preset.min(2)].formations[open.formation.min(2)];
    let slot = formation.players[index];
    let keeper = slot.position == 0;
    let rect = marker_rect(badge_centre(canvas.min, slot), canvas);
    egui::Popup::new(
        ui.id().with("slot popover"),
        ui.ctx().clone(),
        egui::PopupAnchor::ParentRect(rect),
        ui.layer_id(),
    )
    .align(egui::RectAlign::RIGHT_START)
    .gap(6.0)
    .show(|ui| {
        ui.horizontal(|ui| {
            // The name's colour codes, plain runs kept strong.
            let mut name = egui::text::LayoutJob::default();
            append_name(&mut name, raw_name_at(roster, starting_eleven[index]));
            for section in &mut name.sections {
                if section.format.color == egui::Color32::PLACEHOLDER {
                    section.format.color = ui.visuals().strong_text_color();
                }
            }
            ui.label(name);
            ui.weak(format!(
                "roster {}",
                usize::from(starting_eleven[index]) + 1
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Position");
            if keeper {
                ui.add_enabled_ui(false, |ui| {
                    egui::ComboBox::from_id_salt("slot position")
                        .selected_text("GK")
                        .show_ui(ui, |_ui| {});
                });
            } else {
                let mut position = slot.position;
                egui::ComboBox::from_id_salt("slot position")
                    .selected_text(position_label(slot.position))
                    .show_ui(ui, |ui| {
                        for (value, label) in POSITIONS.iter().enumerate().skip(1) {
                            ui.selectable_value(&mut position, value as u8, *label);
                        }
                    });
                if position != slot.position {
                    formation.players[index].position = position;
                }
            }
            if ui
                .add_enabled(!keeper, egui::Button::new("Make GK"))
                .on_hover_text("Swaps him into goal; the keeper takes his slot")
                .clicked()
            {
                if let Some(goal) = formation.players.iter().position(|slot| slot.position == 0)
                    && goal != index
                {
                    starting_eleven.swap(index, goal);
                }
                view.popover = None;
            }
        });
        ui.horizontal(|ui| {
            ui.label("X");
            ui.add_enabled(
                !keeper,
                egui::DragValue::new(&mut formation.players[index].x).range(0..=GRID_X),
            )
            .on_disabled_hover_text("The goalkeeper's spot is fixed");
            ui.label("Y");
            ui.add_enabled(
                !keeper,
                egui::DragValue::new(&mut formation.players[index].y).range(0..=GRID_Y),
            )
            .on_disabled_hover_text("The goalkeeper's spot is fixed");
            if ui.button("Close").clicked() {
                view.popover = None;
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use egui::accesskit::Role;
    use pes_version::PesVersion;

    use super::*;
    use crate::pitch::tests::marker_point;
    use crate::test_support::{
        Headless, centre, controls_right_of, entry, fixture_rows, fixture_tactics, roster_rows,
    };

    // The draft's `slotPopover`: a click opens the player's slot details —
    // name and roster number, the outfield Position combo, Make GK, the X
    // and Y fields and Close — and Make GK exchanges his `starting_eleven`
    // entry with the keeper slot's, the formation slots untouched, closing
    // the popover.
    #[test]
    fn a_click_opens_the_popover_and_make_gk_exchanges_the_starters() {
        let mut window = Headless::sized(600.0, 450.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let before = tactics.clone();
        let formation = before.presets[0].formations[0].clone();
        let keeper = formation
            .players
            .iter()
            .position(|slot| slot.position == 0)
            .expect("the fixture has a keeper slot");
        let index = 4;
        assert_ne!(index, keeper, "premise: not the keeper's slot");

        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let at = marker_point(&nodes, &formation.players, index);
        let (_, changed) =
            window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
        assert!(!changed, "opening the popover writes no record");
        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        assert_eq!(
            view.popover.map(|open| open.slot),
            Some(index),
            "the popover is the clicked slot's"
        );
        entry(&nodes, Role::Button, "Make GK");
        entry(&nodes, Role::Button, "Close");
        assert_eq!(
            controls_right_of(&nodes, "Position", Role::ComboBox)[0].value(),
            Some(position_label(formation.players[index].position)),
            "the position combo"
        );

        let make_gk = entry(&nodes, Role::Button, "Make GK");
        let (_, changed) = window.click_pitch(
            centre(make_gk),
            PesVersion::Pes21,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert!(changed, "Make GK is an edit");
        assert_eq!(
            tactics.starting_eleven[index], before.starting_eleven[keeper],
            "he takes the goal"
        );
        assert_eq!(
            tactics.starting_eleven[keeper], before.starting_eleven[index],
            "the keeper takes his slot"
        );
        assert_eq!(tactics.presets, before.presets, "the formation slots stay");
        assert_eq!(view.popover, None, "and the popover closes");
    }

    // The popover's fields write the stored slot: X typed to 55 lands on
    // the slot's x (0 to 104), leaving y alone.
    #[test]
    fn an_x_typed_in_the_popover_writes_x() {
        let mut window = Headless::sized(600.0, 450.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let index = 4;
        let formation = tactics.presets[0].formations[0].clone();
        assert_ne!(formation.players[index].x, 55, "premise: a real change");
        let y = formation.players[index].y;

        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let at = marker_point(&nodes, &formation.players, index);
        window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let field = controls_right_of(&nodes, "X", Role::SpinButton)
            .into_iter()
            .next()
            .expect("the X field");
        window.click_pitch(
            centre(field),
            PesVersion::Pes21,
            &mut tactics,
            &mut view,
            &roster,
        );
        window.pitch(
            PesVersion::Pes21,
            &mut tactics,
            &mut view,
            &roster,
            vec![egui::Event::Text("55".to_owned())],
        );
        window.pitch(
            PesVersion::Pes21,
            &mut tactics,
            &mut view,
            &roster,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
                repeat: false,
            }],
        );
        let slot = tactics.presets[0].formations[0].players[index];
        assert_eq!(slot.x, 55, "the typed x is written");
        assert_eq!(slot.y, y, "y stays");
    }

    // The keeper's popover ("The goalkeeper's slot is fixed"): Position a
    // disabled "GK", X and Y disabled under "The goalkeeper's spot is
    // fixed", Make GK disabled.
    #[test]
    fn the_keepers_popover_locks_the_spot() {
        let mut window = Headless::sized(600.0, 450.0);
        window
            .ctx
            .global_style_mut(|style| style.interaction.tooltip_delay = 0.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let keeper = tactics.presets[0].formations[0]
            .players
            .iter()
            .position(|slot| slot.position == 0)
            .expect("the fixture has a keeper slot");

        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let at = marker_point(&nodes, &tactics.presets[0].formations[0].players, keeper);
        window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let position = controls_right_of(&nodes, "Position", Role::ComboBox)
            .into_iter()
            .next()
            .expect("the position combo");
        assert!(position.is_disabled(), "the keeper's position is fixed");
        assert_eq!(position.value(), Some("GK"), "and shows GK");
        for label in ["X", "Y"] {
            let field = controls_right_of(&nodes, label, Role::SpinButton)
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("no {label} field"));
            assert!(field.is_disabled(), "the keeper's {label} is disabled");
            assert_eq!(
                window.pitch_tooltip(
                    centre(field),
                    "goalkeeper",
                    PesVersion::Pes21,
                    &mut tactics,
                    &mut view,
                    &roster,
                ),
                Some("The goalkeeper's spot is fixed".to_owned()),
                "the {label} field's hover"
            );
        }
        assert!(
            entry(&nodes, Role::Button, "Make GK").is_disabled(),
            "the keeper's Make GK is disabled"
        );
    }

    // The popover follows the slot's shown formation: a formation change
    // closes it.
    #[test]
    fn the_popover_closes_when_the_formation_changes() {
        let mut window = Headless::sized(600.0, 450.0);
        let mut tactics = fixture_tactics(PesVersion::Pes21);
        let mut view = TacticsView::default();
        let rows = fixture_rows(PesVersion::Pes21);
        let roster = roster_rows(&rows);
        let index = 4;

        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        let at = marker_point(&nodes, &tactics.presets[0].formations[0].players, index);
        window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        entry(&nodes, Role::Button, "Close");

        let other = entry(&nodes, Role::Button, "In-possession");
        window.click_pitch(
            centre(other),
            PesVersion::Pes21,
            &mut tactics,
            &mut view,
            &roster,
        );
        assert_eq!(view.formation, 1, "in-possession is on screen");
        assert_eq!(view.popover, None, "and the popover closed");
        let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
        assert!(
            nodes.iter().all(|node| node.label() != Some("Close")),
            "no popover is drawn"
        );
    }
}
