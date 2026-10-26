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
    // Each column takes its widest cell's natural width while the two fit
    // the list, else half the list each (the draft's `1fr`), the widths
    // read from the panel's own width — inside the Grid a cell's
    // `available_width` is the column's, not the panel's.
    let (cells, set_stats_index, set_stats) = match view.rules.as_ref().map(|active| &active.rules)
    {
        Some(Ok(rules)) => {
            let labels: Vec<&str> = rules
                .ruleset()
                .tiers
                .iter()
                .map(|tier| tier.label.as_str())
                .collect();
            let (widths, set_stats) = cell_widths(ui, &labels);
            (widths, labels.len(), set_stats)
        }
        // The load error is one cell (the wrapped label) before Set
        // stats; a width past the list's keeps the two halves.
        Some(Err(_)) => {
            let (_, set_stats) = cell_widths(ui, &[]);
            (vec![f32::MAX], 1, set_stats)
        }
        None => {
            let (widths, set_stats) = cell_widths(ui, &[]);
            (widths, 0, set_stats)
        }
    };
    let columns = column_widths(&cells, ui.available_width());
    let height = ui.spacing().interact_size.y;
    egui::Grid::new("save_editor_quick_actions")
        .num_columns(2)
        .spacing(egui::vec2(6.0, 3.0))
        .show(ui, |ui| {
            if let Some(active) = view.rules.as_ref() {
                match &active.rules {
                    Ok(rules) => {
                        for (index, tier) in rules.ruleset().tiers.iter().enumerate() {
                            let button = ui
                                .add_enabled_ui(enabled, |ui| {
                                    ui.add_sized(
                                        [columns[index % 2], height],
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
            // Set stats with its number, in one cell: the field takes what
            // the button leaves of the cell (the draft's shrinking input).
            let set_stats_col = columns[set_stats_index % 2];
            ui.allocate_ui(egui::vec2(set_stats_col, height), |ui| {
                ui.horizontal(|ui| {
                    // The field keeps its two-digit floor; the button takes
                    // the rest of the cell and elides past it (the same
                    // rule as the Make buttons).
                    let button_width = set_stats.button.min(
                        (set_stats_col - ui.spacing().item_spacing.x - set_stats.field).max(0.0),
                    );
                    let button = ui
                        .add_enabled_ui(enabled, |ui| {
                            ui.add_sized(
                                [button_width, height],
                                egui::Button::new("Set stats").truncate(),
                            )
                        })
                        .inner;
                    let left = set_stats_col - button.rect.width() - ui.spacing().item_spacing.x;
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
                    // The field's `min_size` is what the button leaves of
                    // the cell: a `DragValue`'s own minimum would be the
                    // 40-point `interact_size`, past the cell's edge.
                    let field = ui
                        .add_enabled_ui(enabled, |ui| {
                            ui.add(
                                egui::DragValue::new(&mut view.set_stats)
                                    .range(40..=99)
                                    .min_size(egui::vec2(left, height)),
                            )
                        })
                        .inner;
                    if !enabled {
                        field.on_disabled_hover_text(TEAM_TIP);
                    }
                });
            });
            ui.end_row();
        });
}

/// The Set stats cell's two sub-widths: the button's natural width and
/// the field's two-digit floor, needed by `show` to share the cell
/// between them.
struct SetStatsWidths {
    button: f32,
    field: f32,
}

/// The cells' natural widths in grid order — a Make button's its full
/// "Make <tier>" text plus the button padding, and the Set stats cell its
/// button, the item spacing and the field's two digits the same way (both
/// widgets take a padding each) — with the Set stats cell's sub-widths
/// returned separately for `show`'s split.
fn cell_widths(ui: &egui::Ui, labels: &[&str]) -> (Vec<f32>, SetStatsWidths) {
    let padding = 2.0 * ui.spacing().button_padding.x;
    let button = egui::TextStyle::Button.resolve(ui.style());
    let field = ui.style().drag_value_text_style.resolve(ui.style());
    let galley = |text: &str, font: &egui::FontId| {
        ui.fonts_mut(|fonts| {
            fonts.layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::WHITE)
        })
        .size()
        .x
    };
    let set_stats = SetStatsWidths {
        button: galley("Set stats", &button) + padding,
        field: galley("99", &field) + padding,
    };
    let mut widths: Vec<f32> = labels
        .iter()
        .map(|label| galley(&format!("Make {label}"), &button) + padding)
        .collect();
    widths.push(set_stats.button + ui.spacing().item_spacing.x + set_stats.field);
    (widths, set_stats)
}

/// The two columns' widths for `cells`' naturals in grid order (even
/// index: column 1, odd: column 2): each its widest cell's while the two
/// fit `list_width`, else half the list each — a label too long for its
/// column is the button's `.truncate()`'s job (its tooltip names the
/// tier).
fn column_widths(cells: &[f32], list_width: f32) -> [f32; 2] {
    let widest = |start: usize| {
        cells
            .iter()
            .skip(start)
            .step_by(2)
            .copied()
            .fold(0.0, f32::max)
    };
    let columns = [widest(0), widest(1)];
    if columns[0] + 6.0 + columns[1] <= list_width {
        columns
    } else {
        let half = (list_width - 6.0) / 2.0;
        [half, half]
    }
}
