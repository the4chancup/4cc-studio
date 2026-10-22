//! The toolbar's menu buttons (`plans/save_editor.md` "Feature
//! inventory"): the Team menu's operations over the selected player's
//! team, each a sealed undo entry (`session/team_ops.rs`), and the Set
//! boots and gloves IDs for everyone dialog (`Boglo`, the draft's
//! `dlgBoglo`).

use crate::session::Session;
use crate::session::team_ops::{BootsMode, GlovesMode};
use crate::view::appearance::ownership_tip;
use crate::view::{SaveView, selected_player};

/// The Team menu button (the draft's `teamMenu`, the FPC items apart):
/// Set edited flag, the boots/gloves dialog, Set player names to
/// positions, and Clear visual flags (its Ctrl+F drawn beside). Every item
/// acts on the selected player's team (`DECISIONS.md` 2026-10-22); the
/// menu is disabled with no save open, and the two aesthetics items carry
/// the ownership line as their disabled hover (SE-GUI-22).
pub(crate) fn team_menu(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, _, player_id)) = selected_player(view) else {
        ui.add_enabled_ui(false, |ui| {
            ui.menu_button("Team", |_| ());
        });
        return;
    };
    let (unlocked, version) = {
        let Some(session) = view.session.as_ref() else {
            return;
        };
        (view.edit_aesthetics, session.version())
    };
    ui.menu_button("Team", |ui| {
        let Some(session) = view.session.as_mut() else {
            return;
        };
        if ui.button("Set edited flag").clicked() {
            session.set_edited_flag(team_id);
            ui.close();
        }
        if ui
            .add_enabled(
                unlocked,
                egui::Button::new("Set boots and gloves IDs for everyone\u{2026}"),
            )
            .on_disabled_hover_text(ownership_tip(version))
            .clicked()
        {
            view.boglo = Some(Boglo::new(session, player_id));
            ui.close();
        }
        if ui.button("Set player names to positions").clicked() {
            session.names_to_positions(team_id);
            ui.close();
        }
        ui.separator();
        if ui
            .add_enabled(
                unlocked,
                egui::Button::new("Clear visual flags").shortcut_text("Ctrl+F"),
            )
            .on_disabled_hover_text(ownership_tip(version))
            .clicked()
        {
            session.clear_visual_flags(team_id);
            ui.close();
        }
    });
}

/// The Set boots and gloves IDs for everyone dialog's state (`view.boglo`
/// while it is open): the mode radios and the three ID fields, prefilled
/// from the selected player's own IDs with Unchanged picked on both sides
/// (the draft's defaults); `error` holds the refusal OK last got, shown
/// under the buttons while the dialog stays open.
pub(crate) struct Boglo {
    /// The boots mode radio.
    boots: BootsPick,
    /// The gloves mode radio.
    gloves: GlovesPick,
    /// Incremental's start ID.
    start: u32,
    /// Same-ID-for-all's boots ID.
    boots_id: u32,
    /// Same-ID-for-all's gloves ID.
    gloves_id: u32,
    /// The refusal `set_boots_and_gloves` returned on OK.
    error: Option<String>,
}

/// The boots radio's pick; the IDs it needs live on `Boglo` (`start`,
/// `boots_id`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum BootsPick {
    /// Each player keeps his boots ID.
    Unchanged,
    /// `start + k - 1` for slot `k`.
    Incremental,
    /// `boots_id` for everyone.
    Same,
}

/// The gloves radio's pick.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GlovesPick {
    /// Each player keeps his gloves ID.
    Unchanged,
    /// Each player's (new) boots ID.
    LikeBoots,
    /// `gloves_id` for everyone.
    Same,
}

impl Boglo {
    /// The dialog as it opens: both sides on Unchanged, the IDs prefilled
    /// from `player`'s own.
    fn new(session: &Session, player_id: u32) -> Boglo {
        let player = session
            .file()
            .player(player_id)
            .expect("the selected player");
        Boglo {
            boots: BootsPick::Unchanged,
            gloves: GlovesPick::Unchanged,
            start: player.appearance.boots_id,
            boots_id: player.appearance.boots_id,
            gloves_id: player.appearance.gloves_id,
            error: None,
        }
    }

    /// The radios' picks as `set_boots_and_gloves`' modes.
    fn modes(&self) -> (BootsMode, GlovesMode) {
        (
            match self.boots {
                BootsPick::Unchanged => BootsMode::Unchanged,
                BootsPick::Incremental => BootsMode::Incremental { start: self.start },
                BootsPick::Same => BootsMode::Same(self.boots_id),
            },
            match self.gloves {
                GlovesPick::Unchanged => GlovesMode::Unchanged,
                GlovesPick::LikeBoots => GlovesMode::LikeBoots,
                GlovesPick::Same => GlovesMode::Same(self.gloves_id),
            },
        )
    }
}

/// The dialog while `view.boglo` holds one: the boots and gloves radios
/// with their ID fields (the last filled slot's resulting ID read-only
/// beside Incremental's start, the draft's `dlgBoglo`), the last refusal
/// under the buttons; OK applies and closes, a refusal keeps it open, and
/// Cancel or Escape closes it.
pub(crate) fn boglo_dialog(ui: &mut egui::Ui, view: &mut SaveView) {
    if view.boglo.is_none() {
        return;
    }
    let Some((team_id, _, _)) = selected_player(view) else {
        return;
    };
    let Some(session) = view.session.as_mut() else {
        return;
    };
    // The last filled slot's resulting ID: slot `k` (1-based) gets
    // `start + k - 1`, so the last one lands `start + index` out.
    let last_slot = session
        .file()
        .team(team_id)
        .and_then(|team| team.roster.iter().rposition(|slot| slot.player_id != 0));
    let boglo = view.boglo.as_mut().expect("checked above");
    let mut close = false;
    let response = egui::Modal::new(egui::Id::new("save_editor_boglo")).show(ui.ctx(), |ui| {
        ui.heading("Set boots and gloves IDs for everyone");
        ui.horizontal(|ui| {
            ui.label("Boots:");
            ui.radio_value(&mut boglo.boots, BootsPick::Unchanged, "Unchanged");
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut boglo.boots, BootsPick::Incremental, "Incremental from");
            ui.add(egui::DragValue::new(&mut boglo.start));
            if let Some(slot) = last_slot {
                ui.weak(format!(
                    "(\u{2192} {})",
                    u64::from(boglo.start) + u64::try_from(slot).expect("a slot index fits u64")
                ));
            }
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut boglo.boots, BootsPick::Same, "Same ID for all");
            ui.add(egui::DragValue::new(&mut boglo.boots_id));
        });
        ui.horizontal(|ui| {
            ui.label("Gloves:");
            ui.radio_value(&mut boglo.gloves, GlovesPick::Unchanged, "Unchanged");
            ui.radio_value(
                &mut boglo.gloves,
                GlovesPick::LikeBoots,
                "Same as the boot IDs",
            );
        });
        ui.horizontal(|ui| {
            ui.radio_value(&mut boglo.gloves, GlovesPick::Same, "Same ID for all");
            ui.add(egui::DragValue::new(&mut boglo.gloves_id));
        });
        ui.horizontal(|ui| {
            if ui.button("OK").clicked() {
                let (boots, gloves) = boglo.modes();
                match session.set_boots_and_gloves(team_id, boots, gloves) {
                    Ok(()) => close = true,
                    Err(error) => boglo.error = Some(format!("{error:#}")),
                }
            }
            if ui.button("Cancel").clicked() {
                close = true;
            }
        });
        if let Some(error) = &boglo.error {
            ui.colored_label(ui.visuals().error_fg_color, error.as_str());
        }
    });
    if close || response.should_close() {
        view.boglo = None;
    }
}
