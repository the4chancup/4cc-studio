//! The quick actions column (`plans/save_editor.md` "Configurable AATF rules",
//! the Quick actions bullet; the draft's `quickActions`): at the Abilities &
//! Skills tab's top, one "Make <tier>" button per tier of the active ruleset
//! in its order, each a `Session::apply_tier` as one undoable edit, then the
//! Set stats row with its number — the Player menu's Set all stats in one
//! click, 40 to 99 — and `AATF…`. A rules file that does not load leaves Set
//! stats alone in the column, the load error in the tier buttons' place as
//! the badges show it; a failed `apply_tier` leaves the player as he was and
//! shows the error in the refusal dialog.

use crate::view::{Refusal, SaveView, aatf_panel, rules_load_error, selected_player};

/// The column at the tab's top; nothing when the selection names no rostered
/// player (the header draws nothing then either).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot, player_id)) = selected_player(view) else {
        return;
    };
    ui.strong("Quick actions");
    if let Some(active) = view.rules.as_ref() {
        match &active.rules {
            Ok(rules) => {
                for tier in &rules.ruleset().tiers {
                    if ui
                        .button(format!("Make {}", tier.label))
                        .on_hover_text(format!(
                            "The {} tier's values (the active ruleset)",
                            tier.label
                        ))
                        .clicked()
                        && let Some(session) = view.session.as_mut()
                        && let Err(error) = session.apply_tier(rules, team_id, slot, &tier.id)
                    {
                        view.refusal = Some(Refusal {
                            title: "Cannot apply the tier",
                            message: format!("{error:#}"),
                        });
                    }
                }
            }
            Err(error) => {
                ui.weak(rules_load_error(error));
            }
        }
    }
    ui.horizontal(|ui| {
        if ui
            .button("Set stats")
            .on_hover_text("Every ability stat to the value")
            .clicked()
            && let Some(session) = view.session.as_mut()
        {
            session.set_all_stats(player_id, view.set_stats);
        }
        ui.add(egui::DragValue::new(&mut view.set_stats).range(40..=99));
    });
    if ui
        .button("AATF\u{2026}")
        .on_hover_text("The active ruleset's violations for this team")
        .clicked()
    {
        view.aatf.open = true;
        view.aatf.mode = aatf_panel::AatfMode::Current;
    }
}
