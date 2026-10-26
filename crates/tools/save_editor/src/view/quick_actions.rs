//! The quick actions (`plans/save_editor.md` "Configurable AATF rules", the
//! Quick actions bullet; the draft's `qagrid`): a two-column grid under the
//! player list, one "Make <tier>" button per tier of the active ruleset in
//! its order, each a `Session::apply_tier` as one undoable edit, then the
//! Set stats cell — the Player menu's Set all stats in one click, 40 to 99.
//! They act on the selected player, so a team row leaves them disabled with
//! the reason on hover. A rules file that does not load leaves Set stats
//! alone in the grid, the load error in the tier buttons' place as the
//! Cards left row shows it; a failed `apply_tier` leaves the player as he
//! was and shows the error in the refusal dialog. The floating AATF panel
//! opens from the toolbar's Panels menu.

use crate::view::{Refusal, SaveView, rules_load_error, selected_player};

/// The disabled controls' tooltip on a team selection.
const TEAM_TIP: &str = "The make buttons act on a player: pick a player row";

/// The grid under the list; its controls disabled while the selection names
/// no rostered player (a team row).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let player = selected_player(view);
    let enabled = player.is_some();
    egui::Grid::new("save_editor_quick_actions")
        .num_columns(2)
        .show(ui, |ui| {
            // The cells are the panel's halves (the draft's `1fr`): a
            // wider tier label elides inside its cell rather than clip
            // past the panel's edge.
            let cell = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
            let height = ui.spacing().interact_size.y;
            if let Some(active) = view.rules.as_ref() {
                match &active.rules {
                    Ok(rules) => {
                        for (index, tier) in rules.ruleset().tiers.iter().enumerate() {
                            let button = ui
                                .add_enabled_ui(enabled, |ui| {
                                    ui.add_sized(
                                        [cell, height],
                                        egui::Button::new(format!("Make {}", tier.label))
                                            .truncate(),
                                    )
                                })
                                .inner;
                            let response = if enabled {
                                button.on_hover_text(format!(
                                    "The {} tier's values (the active ruleset)",
                                    tier.label
                                ))
                            } else {
                                button.on_disabled_hover_text(TEAM_TIP)
                            };
                            if response.clicked()
                                && let Some((team_id, slot, _)) = player
                                && let Some(session) = view.session.as_mut()
                                && let Err(error) =
                                    session.apply_tier(rules, team_id, slot, &tier.id)
                            {
                                view.refusal = Some(Refusal {
                                    title: "Cannot apply the tier",
                                    message: format!("{error:#}"),
                                });
                            }
                            if index % 2 == 1 {
                                ui.end_row();
                            }
                        }
                    }
                    Err(error) => {
                        // Wrapped: the panel is narrow and the error names a path.
                        ui.add(
                            egui::Label::new(egui::RichText::new(rules_load_error(error)).weak())
                                .wrap(),
                        );
                    }
                }
            }
            // Set stats with its number, in one cell.
            ui.allocate_ui(egui::vec2(cell, height), |ui| {
                ui.horizontal(|ui| {
                    let button = ui.add_enabled(enabled, egui::Button::new("Set stats"));
                    let response = if enabled {
                        button.on_hover_text(
                            "Set every ability stat to the value next to this button",
                        )
                    } else {
                        button.on_disabled_hover_text(TEAM_TIP)
                    };
                    if response.clicked()
                        && let Some((_, _, player_id)) = player
                        && let Some(session) = view.session.as_mut()
                    {
                        session.set_all_stats(player_id, view.set_stats);
                    }
                    let field = ui.add_enabled(
                        enabled,
                        egui::DragValue::new(&mut view.set_stats).range(40..=99),
                    );
                    if !enabled {
                        field.on_disabled_hover_text(TEAM_TIP);
                    }
                });
            });
            ui.end_row();
        });
}
