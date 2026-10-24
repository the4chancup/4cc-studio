//! The team bar and the Team settings tab (`plans/save_editor.md` "Editing
//! UI", the team bar and the Tabs bullets; the draft's `teamBar` and
//! `teamTab`): the bar holds the selected player's team's name, short name
//! and colours 1-2 over the tab strip on every tab; the tab holds the
//! manager and stadium IDs and the kit slot assignments. Every control
//! edits a copy of the team's entry and, when it changes, pushes it with
//! `Session::edit_team` under the control's `egui::Id` (the undo stack's
//! merge key, as `info_strip.rs` does). A field the version does not store
//! is drawn disabled in its place, its tooltip naming the first version
//! that has it (the kit slots PES 17 alone has); a value the save holds
//! outside a control's range shows as stored and changes only when edited
//! (the Abilities rule: drawing never edits).

use std::ops::RangeInclusive;

use pes_savefile::model::names::team_text_fits;
use pes_savefile::model::{TeamColor, TeamEntry};
use pes_savefile::schema::fields::{TeamField, TeamText};
use pes_savefile::schema::{VersionSchema, first_version_with_team, schema_for};
use pes_version::PesVersion;

use crate::session::Session;
use crate::view::{SaveView, edit_team, scoped, selected_team};

/// The width the bar's four pairs take on one line, measured once in a
/// headless frame at 1600 x 1400 (the drawn row's `min_rect`: ~722.4
/// points — the pairs' widths are fixed, a longer team name changes
/// nothing). A wrong estimate wraps a bit early or late; it never splits
/// a pair, since the fallback is whole pairs on two rows.
pub(super) const TEAM_BAR_ONE_ROW: f32 = 723.0;

/// The bar over the tab strip, on every tab: the selected player's team's
/// name, short name and colours — one row where the pane holds
/// `TEAM_BAR_ONE_ROW`, else the names on the first row and the colours on
/// the second, so a pair always stays whole. Nothing when the selection
/// names no rostered player (the tab draws nothing then either).
pub(crate) fn bar(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, version, team)) = selected_team(view) else {
        return;
    };
    let generation = view.generation;
    let Some(session) = view.session.as_mut() else {
        return;
    };
    // The tab's `push_id` scoping, kept: every widget id under the bar is
    // the team's within this open, so a text field's undo history and the
    // pickers' state never cross teams — nor saves (the `generation`
    // half). Each pair under `scoped`'s fixed `scope_id` in both
    // arrangements too: an id follows call order inside a scope, so
    // crossing `TEAM_BAR_ONE_ROW` would re-key the fields.
    ui.push_id((generation, team_id), |ui| {
        let ids = ["name", "short", "colour 1", "colour 2"].map(|salt| ui.make_persistent_id(salt));
        let rows: &[&[usize]] = if ui.available_width() >= TEAM_BAR_ONE_ROW {
            &[&[0, 1, 2, 3]]
        } else {
            &[&[0, 1], &[2, 3]]
        };
        for row in rows {
            ui.horizontal(|ui| {
                for &unit in *row {
                    scoped(ui, ids[unit], |ui| {
                        bar_unit(ui, session, team_id, version, &team, unit);
                    });
                }
            });
        }
    });
}

/// The bar's `unit`, in its order: the name pair, the short pair, then
/// the two colours.
fn bar_unit(
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

/// The Team settings tab for the selected player's team: the manager and
/// stadium IDs and the kit slot assignments.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, version, team)) = selected_team(view) else {
        return;
    };
    let generation = view.generation;
    let Some(session) = view.session.as_mut() else {
        return;
    };
    let schema = schema_for(version);
    // Every widget id under the tab is the team's within this open: a
    // DragValue's in-progress edit never crosses teams — nor saves (the
    // `generation` half).
    ui.push_id((generation, team_id), |ui| {
        egui::Grid::new("team fields").show(ui, |ui| {
            ids_row(ui, session, team_id, schema, &team);
            ui.end_row();
        });
        ui.separator();
        ui.strong("Kit slot assignments");
        kit_slots(ui, session, team_id, schema, &team);
    });
}

/// The name field's unit (the draft's `teamBar` pair). A keystroke
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
    ui.weak("Team");
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
    ui.weak("Short");
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
    ui.label(["Colour 1", "Colour 2"][index]);
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
                "Manager ID",
                stored,
                0..=u32::try_from(field_max(schema, TeamField::ManagerId))
                    .expect("the manager id's bound fits u32"),
                |team, shown| {
                    team.manager_id = Some(shown);
                },
            );
        }
        None => {
            ui.label("Manager ID");
            gated_number(ui, TeamField::ManagerId);
        }
    }
    match team.stadium_id {
        Some(stored) => {
            number(
                ui,
                session,
                team_id,
                "Stadium ID",
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
            ui.label("Stadium ID");
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
        ui.weak("(no kit slot records on this version)")
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
                "Number",
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
                "Bound team ID",
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
/// (`2^width - 1`, the codec's `ValueTooWide` bound): the range the tab's
/// number controls take; called only where the entry holds `Some`.
fn field_max(schema: &VersionSchema, field: TeamField) -> u64 {
    schema
        .team
        .max_value(field)
        .expect("the field is stored by this version")
}

/// The shown 8-bit channel for a stored 6-bit one (`v * 255 / 63` rounded,
/// `plans/save_editor.md` "Crate layout" `view/`).
fn channel_to_show(v: u8) -> u8 {
    u8::try_from((u16::from(v) * 255 + 31) / 63).expect("63 maps to 255")
}

/// The stored 6-bit channel for a picked 8-bit one (`c * 63 / 255`
/// rounded): a pick always lands on a value the save holds.
fn channel_to_store(c: u8) -> u8 {
    u8::try_from((u16::from(c) * 63 + 127) / 255).expect("255 maps to 63")
}

/// `rgb` as `#RRGGBB` upper-case (the picker control's own hex form).
fn format_hex(rgb: [u8; 3]) -> String {
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
