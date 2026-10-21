//! The general info strip (`plans/save_editor.md` "Editing UI", the draft's
//! `infoStrip`): the selected player's general fields, always visible above the
//! tabs. Each control edits a copy of the entry and, when it changes, pushes it
//! under its own `egui::Id` (the undo stack's merge key).

use pes_savefile::model::names::player_text_fits;
use pes_savefile::model::{PlayerEntry, TeamEntry};
use pes_savefile::schema::fields::{PlayerField, PlayerText, RosterField};
use pes_savefile::schema::{VersionSchema, schema_for};

use crate::session::Session;
use crate::view::SaveView;

/// The strip for the selected player; nothing when the selection names no
/// rostered player (a stale slot, an empty save's list).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot)) = view.selected else {
        return;
    };
    let Some(session) = view.session.as_mut() else {
        return;
    };
    let file = session.file();
    let Some(team) = file.team(team_id) else {
        return;
    };
    let Some(player_id) = team
        .roster
        .get(usize::from(slot))
        .map(|slot| slot.player_id)
    else {
        return;
    };
    if file.player(player_id).is_none() {
        return;
    }
    strip(ui, session, team_id, slot, player_id);
}

/// One row of name/ID/shirt and one of height, weight, age, nation, number
/// and captain.
fn strip(ui: &mut egui::Ui, session: &mut Session, team_id: u32, slot: u8, player_id: u32) {
    let (mut name, mut shirt, mut nation, mut shirt_number, basics, stored_captain, schema) = {
        let file = session.file();
        let player = file.player(player_id).expect("the selected player");
        let team = file.team(team_id).expect("the selected team");
        (
            // The name field shows the raw name, its `\x11` spelled as the
            // four characters `\\x11` (the draft's `escName`); on edit the
            // field's text is turned back before it is stored.
            player.name.replace('\x11', "\\x11"),
            player.shirt_name.clone(),
            player.basic.nationality,
            team.roster
                .get(usize::from(slot))
                .map_or(0, |slot| slot.number),
            (player.basic.height, player.basic.weight, player.basic.age),
            team.tactics.set_pieces.captain,
            schema_for(file.version()),
        )
    };
    let mut captain = stored_captain == slot;
    egui::Grid::new("save_editor_info_strip").show(ui, |ui| {
        ui.label("Name");
        let response = ui
            .add(egui::TextEdit::singleline(&mut name).desired_width(240.0))
            .on_hover_text(format!(
                "Up to {} bytes: a plain letter takes one, an accented letter two or more, a \
                 colour code ten",
                text_len(schema, PlayerText::Name)
            ));
        if response.changed() {
            let raw = name.replace("\\x11", "\x11");
            // An edit the codec would refuse is not pushed: the field shows
            // the stored text again next frame.
            if player_text_fits(schema.version, PlayerText::Name, &raw) {
                edit_player(session, player_id, response.id.value(), |player| {
                    player.name = raw;
                });
            }
        }
        ui.label("ID");
        ui.monospace(player_id.to_string());
        ui.label("Shirt");
        let response = ui
            .add(egui::TextEdit::singleline(&mut shirt).desired_width(140.0))
            .on_hover_text(format!(
                "Up to {} letters; accented Latin letters are fine, other scripts and symbols \
                 are not",
                text_len(schema, PlayerText::ShirtName)
            ));
        if response.changed() && player_text_fits(schema.version, PlayerText::ShirtName, &shirt) {
            edit_player(session, player_id, response.id.value(), |player| {
                player.shirt_name = shirt.clone();
            });
        }
        ui.end_row();

        let (height, weight, age) = basics;
        number(
            ui,
            session,
            player_id,
            "Height",
            height,
            max(schema, PlayerField::Height),
            |player: &mut PlayerEntry, value| player.basic.height = value,
        );
        number(
            ui,
            session,
            player_id,
            "Weight",
            weight,
            max(schema, PlayerField::Weight),
            |player: &mut PlayerEntry, value| player.basic.weight = value,
        );
        number(
            ui,
            session,
            player_id,
            "Age",
            age,
            max(schema, PlayerField::Age),
            |player: &mut PlayerEntry, value| player.basic.age = value,
        );
        ui.label("Nation");
        let response =
            ui.add(egui::DragValue::new(&mut nation).range(
                0..=u16::try_from(max(schema, PlayerField::Nationality)).unwrap_or(u16::MAX),
            ));
        if response.changed() {
            edit_player(session, player_id, response.id.value(), |player| {
                player.basic.nationality = nation;
            });
        }
        ui.label("Number");
        let response =
            ui.add(egui::DragValue::new(&mut shirt_number).range(0..=roster_number_max(schema)));
        if response.changed() {
            edit_team(session, team_id, response.id.value(), |team| {
                team.roster[usize::from(slot)].number = shirt_number;
            });
        }
        ui.label("Captain");
        // The team's `captain` is this roster slot, 0-based as stored;
        // unticking stores 255.
        let response = ui.add(egui::Checkbox::new(&mut captain, ""));
        if response.changed() {
            edit_team(session, team_id, response.id.value(), |team| {
                team.tactics.set_pieces.captain = if captain { slot } else { u8::MAX };
            });
        }
    });
}

/// A `label`/`DragValue` pair capped at `max`; a change pushes the player's
/// entry with `set` applied.
fn number<N: egui::emath::Numeric + TryFrom<u64>>(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    label: &str,
    value: N,
    max: u64,
    set: impl FnOnce(&mut PlayerEntry, N),
) {
    let mut value = value;
    ui.label(label);
    let response = ui.add(
        egui::DragValue::new(&mut value)
            .range(N::from_f64(0.0)..=N::try_from(max).unwrap_or(N::MAX)),
    );
    if response.changed() {
        edit_player(session, player_id, response.id.value(), |player| {
            set(player, value);
        });
    }
}

/// Clones the player's entry, applies `edit` and pushes it under `key`.
fn edit_player(session: &mut Session, id: u32, key: u64, edit: impl FnOnce(&mut PlayerEntry)) {
    let mut edited = session
        .file()
        .player(id)
        .expect("the selected player")
        .clone();
    edit(&mut edited);
    session.edit_player(key, edited);
}

/// `edit_player` for the team's entry.
fn edit_team(session: &mut Session, id: u32, key: u64, edit: impl FnOnce(&mut TeamEntry)) {
    let mut edited = session.file().team(id).expect("the selected team").clone();
    edit(&mut edited);
    session.edit_team(key, edited);
}

/// The biggest value the save's `schema` stores in `field`'s player-record
/// bit run (`2^width - 1`, the codec's `ValueTooWide` bound): the range the
/// strip's number controls take, so a typed value is never one the codec
/// refuses. The strip's fields are stored on every version; a field the
/// schema lacks clamps to the model type's own range.
fn max(schema: &VersionSchema, field: PlayerField) -> u64 {
    schema
        .player
        .fields
        .iter()
        .find(|spec| spec.field == field)
        .map(|spec| (1u64 << spec.bit_width) - 1)
        .unwrap_or(u64::MAX)
}

/// The byte length of the player record's `which` text field, from the
/// schema (the Name field's bytes, and the Shirt field's char count).
fn text_len(schema: &VersionSchema, which: PlayerText) -> u32 {
    schema
        .player
        .texts
        .iter()
        .find(|spec| spec.text == which)
        .map(|spec| spec.len)
        .expect("every version stores the text")
}

/// The same bound for the roster's shirt-number run.
fn roster_number_max(schema: &VersionSchema) -> u16 {
    schema
        .roster
        .arrays
        .iter()
        .find(|array| (array.make)(0) == RosterField::Number(0))
        .map(|array| u16::try_from((1u64 << array.bit_width) - 1).unwrap_or(u16::MAX))
        .unwrap_or(u16::MAX)
}
