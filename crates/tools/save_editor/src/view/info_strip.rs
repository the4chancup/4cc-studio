//! The general info strip (`plans/save_editor.md` "Editing UI", the draft's
//! `infoStrip`): the selected player's general fields, always visible above the
//! tabs. The controls are drawn disabled until the undo stack's slice wires
//! their edits (`edit_player`/`edit_team`); the values shown are the file's.

use pes_savefile::EditFile;
use pes_savefile::model::{PlayerEntry, TeamEntry};
use pes_savefile::schema::fields::{PlayerField, RosterField};
use pes_savefile::schema::{VersionSchema, schema_for};

use crate::view::SaveView;

/// The strip for the selected player; nothing when the selection names no
/// rostered player (a stale slot, an empty save's list).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot)) = view.selected else {
        return;
    };
    let Some(session) = view.session.as_ref() else {
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
    let Some(player) = file.player(player_id) else {
        return;
    };
    strip(ui, file, team, slot, player);
}

/// One row of name/ID/shirt and one of height, weight, age, nation, number
/// and captain, every control disabled at the file's value.
fn strip(ui: &mut egui::Ui, file: &EditFile, team: &TeamEntry, slot: u8, player: &PlayerEntry) {
    let schema = schema_for(file.version());
    // The name field shows the raw name, its `\x11` spelled as the four
    // characters `\\x11` (the draft's `escName`); on edit the field's text is
    // turned back before it is stored.
    let mut name = player.name.replace('\x11', "\\x11");
    let mut shirt = player.shirt_name.clone();
    egui::Grid::new("save_editor_info_strip").show(ui, |ui| {
        ui.label("Name");
        ui.add_enabled(
            false,
            egui::TextEdit::singleline(&mut name).desired_width(240.0),
        )
        .on_disabled_hover_text("The raw name; \\x11c followed by 8 hex digits is a colour code");
        ui.label("ID");
        ui.monospace(player.id.to_string());
        ui.label("Shirt");
        ui.add_enabled(
            false,
            egui::TextEdit::singleline(&mut shirt).desired_width(140.0),
        );
        ui.end_row();

        number(
            ui,
            "Height",
            player.basic.height,
            max(schema, PlayerField::Height),
        );
        number(
            ui,
            "Weight",
            player.basic.weight,
            max(schema, PlayerField::Weight),
        );
        number(ui, "Age", player.basic.age, max(schema, PlayerField::Age));
        let mut nation = player.basic.nationality;
        ui.label("Nation");
        ui.add_enabled(
            false,
            egui::DragValue::new(&mut nation).range(
                0..=u16::try_from(max(schema, PlayerField::Nationality)).unwrap_or(u16::MAX),
            ),
        );
        let mut number_value = team
            .roster
            .get(usize::from(slot))
            .map_or(0, |slot| slot.number);
        ui.label("Number");
        ui.add_enabled(
            false,
            egui::DragValue::new(&mut number_value).range(0..=roster_number_max(schema)),
        );
        ui.label("Captain");
        // The team's `captain` is this roster slot, 0-based as stored;
        // unticking stores 255.
        let mut captain = team.tactics.set_pieces.captain == slot;
        ui.add_enabled(false, egui::Checkbox::new(&mut captain, ""));
    });
}

/// A `label`/`DragValue` pair capped at `max`.
fn number(ui: &mut egui::Ui, label: &str, value: u8, max: u64) {
    let mut value = value;
    ui.label(label);
    ui.add_enabled(
        false,
        egui::DragValue::new(&mut value).range(0..=u8::try_from(max).unwrap_or(u8::MAX)),
    );
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
