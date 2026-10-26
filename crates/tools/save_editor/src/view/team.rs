//! The team view (`plans/save_editor.md` "Editing UI", the team view
//! bullet; the draft's `teamView`): the selected team's name with its
//! colours as the title, its name, short name and colours 1-2 fields (the
//! former team bar's pairs), a row of the manager and stadium IDs with a
//! `Kit slots…` button opening the kit slot assignments in a dialog, then
//! the Tactics editor. Every control edits a copy of the team's entry and,
//! when it changes, pushes it with `Session::edit_team` under the control's
//! `egui::Id` (the undo stack's merge key, as `info_strip.rs` does). A
//! field the version does not store is drawn disabled in its place, its
//! tooltip naming the first version that has it (the kit slots PES 17
//! alone has); a value the save holds outside a control's range shows as
//! stored and changes only when edited (the Abilities rule: drawing never
//! edits).

use std::ops::RangeInclusive;

use pes_savefile::model::names::team_text_fits;
use pes_savefile::model::{TeamColor, TeamEntry};
use pes_savefile::schema::fields::{TeamField, TeamText};
use pes_savefile::schema::{VersionSchema, first_version_with_team, schema_for};
use pes_version::PesVersion;
use team_widgets::names::append_name;

use crate::session::Session;
use crate::view::{SaveView, edit_team, scoped, selected_team, tactics};

/// The width the view's four field pairs take on one line, measured once
/// in a headless frame at 1600 x 1400 (the drawn row's `min_rect`: ~724.8
/// points — the pairs' widths are fixed, a longer team name changes
/// nothing). A wrong estimate wraps a bit early or late; it never splits
/// a pair, since the fallback is whole pairs on two rows.
pub(super) const TEAM_FIELDS_ONE_ROW: f32 = 725.0;
/// The view's Team field.
pub(super) const TEAM_LABEL: &str = "Team";
/// The view's Short field.
pub(super) const SHORT_LABEL: &str = "Short";
/// The two colours' labels.
pub(super) const COLOUR_LABELS: [&str; 2] = ["Colour 1", "Colour 2"];
/// The view's Manager ID field.
pub(super) const MANAGER_LABEL: &str = "Manager ID";
/// The view's Stadium ID field.
pub(super) const STADIUM_LABEL: &str = "Stadium ID";
/// The `Kit slots…` button.
pub(super) const KIT_SLOTS_LABEL: &str = "Kit slots\u{2026}";
/// A kit slot's Number field.
pub(super) const KIT_NUMBER_LABEL: &str = "Number";
/// A kit slot's bound team field.
pub(super) const BOUND_TEAM_LABEL: &str = "Bound team ID";

/// The team view for the selected team: the name as the title, the name,
/// short name and colour pairs — one row where the pane holds
/// `TEAM_FIELDS_ONE_ROW`, else the names on the first row and the colours
/// on the second, so a pair always stays whole — the manager and stadium
/// IDs with the `Kit slots…` button, then the Tactics editor. Nothing when
/// the selection names no team of the save.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, version, team)) = selected_team(view) else {
        return;
    };
    let generation = view.generation;
    let schema = schema_for(version);
    // Every widget id under the view is the team's within this open: a
    // text field's undo history, a DragValue's in-progress edit, a combo's
    // popup and the pickers' state never cross teams — nor saves (the
    // `generation` half), nor the player view's, the kind in the salt (a
    // save outside 4cc conventions can give a rostered player a team's
    // id). Each pair under `scoped`'s fixed `scope_id` in
    // both arrangements too: an id follows call order inside a scope, so
    // crossing `TEAM_FIELDS_ONE_ROW` would re-key the fields. The session
    // is re-taken per block so the Tactics editor can borrow the whole
    // view.
    ui.push_id(("team", generation, team_id), |ui| {
        // The view's title: the team's name, its colour codes drawn as the
        // list paints them (`append_name`), at the heading text style — as
        // `info_strip` draws the player's name.
        let mut title = egui::text::LayoutJob::default();
        append_name(&mut title, &team.name);
        let heading = egui::TextStyle::Heading.resolve(ui.style());
        for section in &mut title.sections {
            section.format.font_id = heading.clone();
        }
        ui.label(title);
        let ids = ["name", "short", "colour 1", "colour 2"].map(|salt| ui.make_persistent_id(salt));
        let rows: &[&[usize]] = if ui.available_width() >= TEAM_FIELDS_ONE_ROW {
            &[&[0, 1, 2, 3]]
        } else {
            &[&[0, 1], &[2, 3]]
        };
        for row in rows {
            ui.horizontal(|ui| {
                for &unit in *row {
                    scoped(ui, ids[unit], |ui| {
                        if let Some(session) = view.session.as_mut() {
                            field_unit(ui, session, team_id, version, &team, unit);
                        }
                    });
                }
            });
        }
        ui.horizontal(|ui| {
            egui::Grid::new("team fields").show(ui, |ui| {
                if let Some(session) = view.session.as_mut() {
                    ids_row(ui, session, team_id, schema, &team);
                }
                ui.end_row();
            });
            if ui
                .button(KIT_SLOTS_LABEL)
                .on_hover_text("The kit slot assignments, rarely edited")
                .clicked()
            {
                view.kit_slots = Some(team_id);
            }
        });
        // Half the item spacing: the default separator's whole gap would
        // push the Tactics' last control under the pane's fold.
        ui.add(egui::Separator::default().spacing(ui.spacing().item_spacing.y / 2.0));
        tactics::show(ui, view);
    });
}

/// The field pairs' `unit`, in their order: the name pair, the short pair,
/// then the two colours.
fn field_unit(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    version: PesVersion,
    team: &TeamEntry,
    unit: usize,
) {
    match unit {
        0 => name_pair(ui, session, team_id, version, team),
        1 => short_pair(ui, session, team_id, version, team),
        colour => colour_unit(ui, session, team_id, team, colour - 2),
    }
}

/// The Kit slot assignments dialog while `view.kit_slots` names its team
/// (the draft's `dlgKitSlots`): the assignments' rows in a modal, or the
/// sentence a version without them gets; OK or Escape closes.
pub(crate) fn kit_slots_dialog(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some(team_id) = view.kit_slots else {
        return;
    };
    let Some(session) = view.session.as_mut() else {
        return;
    };
    let (version, team) = match session.file().team(team_id) {
        Some(team) => (session.file().version(), team.clone()),
        None => return,
    };
    let schema = schema_for(version);
    let mut close = false;
    // The rows' widget ids are the team's within this open, as the view's
    // are.
    let generation = view.generation;
    let response =
        egui::Modal::new(egui::Id::unique("save_editor_kit_slots")).show(ui.ctx(), |ui| {
            ui.heading("Kit slot assignments");
            ui.push_id(("team", generation, team_id), |ui| {
                kit_slots(ui, session, team_id, schema, &team);
            });
            if ui.button("OK").clicked() {
                close = true;
            }
        });
    if close || response.should_close() {
        view.kit_slots = None;
    }
}

/// The name field's unit (the draft's `teamFields` pair). A keystroke
/// `team_text_fits` refuses is not taken — the field shows the stored text
/// again next frame, the info strip's Name field's rule. A changed name
/// sets the name flag.
fn name_pair(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    version: PesVersion,
    team: &TeamEntry,
) {
    ui.weak(TEAM_LABEL);
    let mut name = team.name.clone();
    // `add_sized` allocates the width itself: the field keeps its own 240
    // whatever the wrapped row offers it.
    let response = ui.add_sized(
        [240.0, ui.spacing().interact_size.y],
        egui::TextEdit::singleline(&mut name).desired_width(240.0),
    );
    if response.changed() && team_text_fits(version, TeamText::Name, &name) {
        edit_team(session, team_id, response.id.value(), |team| {
            team.name = name;
            team.edit_flags.name = true;
        });
    }
}

/// The short-name field's unit: upper-cased as typed and holding at most
/// three letters (the cap the draft's short-name field enforced; the
/// codec's four-byte field is unterminated and would take a fourth). A
/// change sets the name flag and, on PES 15, the short-name flag too.
fn short_pair(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    version: PesVersion,
    team: &TeamEntry,
) {
    ui.weak(SHORT_LABEL);
    let mut short = team.short_name.clone();
    // The same `add_sized` allocation as the name field's above.
    let response = ui.add_sized(
        [60.0, ui.spacing().interact_size.y],
        egui::TextEdit::singleline(&mut short).desired_width(60.0),
    );
    if response.changed() {
        let upper = short.to_uppercase();
        if upper.chars().count() <= 3 && team_text_fits(version, TeamText::ShortName, &upper) {
            edit_team(session, team_id, response.id.value(), |team| {
                team.short_name = upper;
                team.edit_flags.name = true;
                if team.edit_flags.short_name.is_some() {
                    team.edit_flags.short_name = Some(true);
                }
            });
        }
    }
}

/// One team colour's unit (PES 17+): `color_tools`' picker with no
/// suggestions, the stored 6-bit channels shown as 8 bits and back by the
/// `view/` bullet's rule (`channel_to_show`/`channel_to_store`), the shown
/// colour's upper-case hex beside it. On PES 15/16 the pair draws disabled
/// with the `PES NN+` tooltip, in the same place.
fn colour_unit(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    team: &TeamEntry,
    index: usize,
) {
    ui.label(COLOUR_LABELS[index]);
    match &team.colors {
        Some(colors) => {
            let mut shown = [
                channel_to_show(colors[index].red),
                channel_to_show(colors[index].green),
                channel_to_show(colors[index].blue),
            ];
            let out = color_tools::picker::color_field(ui, ("team colour", index), &mut shown, &[]);
            if out.changed {
                let stored = TeamColor {
                    red: channel_to_store(shown[0]),
                    green: channel_to_store(shown[1]),
                    blue: channel_to_store(shown[2]),
                };
                edit_team(session, team_id, out.response.id.value(), |team| {
                    team.colors
                        .as_mut()
                        .expect("the field is drawn only on a version that stores it")[index] =
                        stored;
                });
            }
            // The shown colour is the nearest the save holds, so the hex
            // reads the rounded-back stored value, not the pick itself.
            let display = [
                channel_to_show(channel_to_store(shown[0])),
                channel_to_show(channel_to_store(shown[1])),
                channel_to_show(channel_to_store(shown[2])),
            ];
            ui.monospace(format_hex(display));
        }
        None => {
            ui.add_enabled_ui(false, |ui| {
                color_tools::picker::color_field(ui, ("team colour", index), &mut [0, 0, 0], &[]);
            })
            .response
            .on_disabled_hover_text(gating_tooltip(
                [TeamField::Color1Red, TeamField::Color2Red][index],
            ));
            ui.weak(format_hex([0, 0, 0]));
        }
    }
}

/// The manager and stadium IDs (PES 19+, `Option` on the entry): a changed
/// stadium ID sets the stadium flag where the version has it (PES 20/21).
/// Elsewhere both draw disabled with the `PES NN+` tooltip, in the same
/// place.
fn ids_row(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    schema: &VersionSchema,
    team: &TeamEntry,
) {
    match team.manager_id {
        Some(stored) => {
            number(
                ui,
                session,
                team_id,
                MANAGER_LABEL,
                stored,
                0..=u32::try_from(field_max(schema, TeamField::ManagerId))
                    .expect("the manager id's bound fits u32"),
                |team, shown| {
                    team.manager_id = Some(shown);
                },
            );
        }
        None => {
            ui.label(MANAGER_LABEL);
            gated_number(ui, TeamField::ManagerId);
        }
    }
    match team.stadium_id {
        Some(stored) => {
            number(
                ui,
                session,
                team_id,
                STADIUM_LABEL,
                stored,
                0..=u16::try_from(field_max(schema, TeamField::StadiumId))
                    .expect("the stadium id's bound fits u16"),
                |team, shown| {
                    team.stadium_id = Some(shown);
                    if team.edit_flags.stadium.is_some() {
                        team.edit_flags.stadium = Some(true);
                    }
                },
            );
        }
        None => {
            ui.label(STADIUM_LABEL);
            gated_number(ui, TeamField::StadiumId);
        }
    }
}

/// The ten kit slot rows (PES 17 only): each its Number (the stored `u8`)
/// and Bound team ID, drawn as `binding / 64` and written as
/// `id * 64 + binding % 64` (the low bits kept as stored). Either edit sets
/// the strip flag where the version has it (PES 17 alone, so `Some`), as
/// the stadium edit sets its own. Elsewhere the draft's no-records line,
/// its tooltip naming PES 17 only.
fn kit_slots(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    schema: &VersionSchema,
    team: &TeamEntry,
) {
    let Some(slots) = &team.kit_slots else {
        ui.weak("(this version does not store kit slots)")
            .on_hover_text("PES 17 only");
        return;
    };
    // The binding field's bit width gives the bound id's top (`binding` is
    // the id x 0x40, so the shown id runs to a sixty-fourth of the field's
    // own top).
    let bound_max = schema
        .team
        .max_value(TeamField::KitSlotTeam(0))
        .expect("the kit slots are drawn only where they are stored")
        / 64;
    egui::Grid::new("team kit slots").show(ui, |ui| {
        for (index, kit) in slots.iter().enumerate() {
            ui.weak(format!("Slot {}", index + 1));
            number(
                ui,
                session,
                team_id,
                KIT_NUMBER_LABEL,
                kit.number,
                0..=u8::MAX,
                move |team, shown| {
                    team.kit_slots
                        .as_mut()
                        .expect("the kit slots exist on PES 17")[index]
                        .number = shown;
                    if team.edit_flags.strip.is_some() {
                        team.edit_flags.strip = Some(true);
                    }
                },
            );
            let low = kit.binding % 64;
            number(
                ui,
                session,
                team_id,
                BOUND_TEAM_LABEL,
                kit.binding / 64,
                0..=u32::try_from(bound_max).expect("the binding's bound fits u32"),
                move |team, shown| {
                    team.kit_slots
                        .as_mut()
                        .expect("the kit slots exist on PES 17")[index]
                        .binding = shown * 64 + low;
                    if team.edit_flags.strip.is_some() {
                        team.edit_flags.strip = Some(true);
                    }
                },
            );
            ui.end_row();
        }
    });
}

/// The biggest value `schema`'s team record stores in `field`
/// (`2^width - 1`, the codec's `ValueTooWide` bound): the range the view's
/// number controls take; called only where the entry holds `Some`.
fn field_max(schema: &VersionSchema, field: TeamField) -> u64 {
    schema
        .team
        .max_value(field)
        .expect("the field is stored by this version")
}

/// The shown 8-bit channel for a stored 6-bit one (`v * 255 / 63` rounded,
/// `plans/save_editor.md` "Crate layout" `view/`).
pub(super) fn channel_to_show(v: u8) -> u8 {
    u8::try_from((u16::from(v) * 255 + 31) / 63).expect("63 maps to 255")
}

/// The stored 6-bit channel for a picked 8-bit one (`c * 63 / 255`
/// rounded): a pick always lands on a value the save holds.
fn channel_to_store(c: u8) -> u8 {
    u8::try_from((u16::from(c) * 63 + 127) / 255).expect("255 maps to 63")
}

/// `rgb` as `#RRGGBB` upper-case (the picker control's own hex form).
pub(super) fn format_hex(rgb: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2])
}

/// A `label`/`DragValue` pair over `shown` in `range`; on a change `store`
/// writes the shown value into the team's entry through `edit_team` (the
/// widget's id as the undo merge key). `clamp_existing_to_range` stays
/// off: a stored value past the range draws as stored, never clamped on
/// sight.
fn number<T: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    label: impl Into<egui::WidgetText>,
    shown: T,
    range: RangeInclusive<T>,
    store: impl FnOnce(&mut TeamEntry, T),
) {
    let mut shown = shown;
    ui.label(label);
    let response = ui.add(
        egui::DragValue::new(&mut shown)
            .range(range)
            .clamp_existing_to_range(false),
    );
    if response.changed() {
        edit_team(session, team_id, response.id.value(), |team| {
            store(team, shown);
        });
    }
}

/// The disabled stand-in for a number the version does not store: `-`
/// inside the control's usual bounds (a Button at the DragValue's minimum
/// size, so an enabled version's control takes the same rect) with the
/// `PES NN+` tooltip (`abilities`' `gated_number` for team fields).
fn gated_number(ui: &mut egui::Ui, field: TeamField) {
    ui.add_enabled(
        false,
        egui::Button::new("-").min_size(ui.spacing().interact_size),
    )
    .on_disabled_hover_text(gating_tooltip(field));
}

/// The tooltip naming the first version that stores `field` (`PES NN+`):
/// `abilities`' `gating_tooltip` for team-record fields.
fn gating_tooltip(field: TeamField) -> String {
    format!(
        "PES {}+",
        first_version_with_team(field)
            .expect("a gated field is stored by some version")
            .number()
    )
}
