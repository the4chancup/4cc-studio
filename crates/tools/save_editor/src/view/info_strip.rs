//! The general info strip (`plans/save_editor.md` "Editing UI", the draft's
//! `infoStrip`): the selected player's general fields, always visible above the
//! tabs. Each control edits a copy of the entry and, when it changes, pushes it
//! under its own `egui::Id` (the undo stack's merge key).

use pes_savefile::model::names::{crop_player_text, name_fits, shirt_name_edit_fits};
use pes_savefile::model::{PlayerEntry, TeamEntry};
use pes_savefile::schema::fields::{PlayerField, PlayerText, RosterField};
use pes_savefile::schema::{VersionSchema, portable_text_max, schema_for};
use pes_version::PesVersion;

use crate::session::Session;
use crate::view::{SaveView, edit_player, selected_player};

/// The strip for the selected player; nothing when the selection names no
/// rostered player (a stale slot, an empty save's list).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot, player_id)) = selected_player(view) else {
        return;
    };
    let generation = view.generation;
    let Some(session) = view.session.as_mut() else {
        return;
    };
    // Every widget id under the strip is the player's within this open:
    // a text field's undo history and a DragValue's in-progress edit
    // never cross players — nor saves, since egui keys a TextEdit's undo
    // history by widget id and the same player id of the save opened next
    // would otherwise inherit the last save's.
    ui.push_id((generation, player_id), |ui| {
        strip(ui, session, team_id, slot, player_id);
    });
}

/// One row of name/ID/shirt and one of height, weight, age, nation, number
/// and captain.
fn strip(ui: &mut egui::Ui, session: &mut Session, team_id: u32, slot: u8, player_id: u32) {
    let (
        mut name,
        stored_name,
        mut shirt,
        stored_shirt,
        mut nation,
        mut shirt_number,
        basics,
        stored_captain,
        schema,
    ) = {
        let file = session.file();
        let player = file.player(player_id).expect("the selected player");
        let team = file.team(team_id).expect("the selected team");
        (
            // The name field shows the raw name, its `\x11` spelled as the
            // four characters `\\x11` (the draft's `escName`); on edit the
            // field's text is turned back before it is stored.
            player.name.replace('\x11', "\\x11"),
            player.name.clone(),
            player.shirt_name.clone(),
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
    let captain = stored_captain == slot;
    // One grid per row: a grid unions each column's width across its rows,
    // so the Name/Shirt fields' 240/140 would stretch every cell column of
    // row 2 (past the pane at half-monitor widths), and a wider grid
    // min-rect would widen the whole panel's `max_rect`.
    egui::Grid::new("save_editor_info_strip").show(ui, |ui| {
        ui.label("Name");
        let overlong = stored_name.len() > portable_text_max(PlayerText::Name);
        let mut name_edit = egui::TextEdit::singleline(&mut name).desired_width(240.0);
        if overlong {
            name_edit = name_edit.background_color(warning_tint(ui.visuals()));
        }
        // `add_sized` allocates the width itself: a fresh grid gives a
        // non-last column only its `min_cell_size` (40) on the sizing
        // pass, and a TextEdit caps its width at what the cell offers.
        let response = ui.add_sized([240.0, ui.spacing().interact_size.y], name_edit);
        // A name past what every version holds is drawn tinted with the cut
        // PES 15 to 19 would make: a warning, never a block on Save.
        let response = if overlong {
            response.on_hover_text(format!(
                "In PES 2019 and older this name will be cut to: «{}»",
                crop_player_text(PesVersion::Pes19, PlayerText::Name, &stored_name)
                    .replace('\x11', "\\x11")
            ))
        } else {
            response.on_hover_text(format!(
                "Up to {} bytes: a plain letter takes one, an accented letter two or more, a \
                 colour code ten",
                text_len(schema, PlayerText::Name)
            ))
        };
        if response.changed() {
            let raw = name.replace("\\x11", "\x11");
            // An edit the codec would refuse is not pushed (the save's own
            // field is the bound): the field shows the stored text again
            // next frame.
            if name_fits(schema.version, &raw) {
                edit_player(session, player_id, response.id.value(), |player| {
                    player.name = raw;
                    player.edit_flags.player = true;
                });
            }
        }
        ui.label("ID");
        ui.monospace(player_id.to_string());
        ui.label("Shirt");
        let shirt_overlong =
            stored_shirt.chars().count() > portable_text_max(PlayerText::ShirtName);
        let mut shirt_edit = egui::TextEdit::singleline(&mut shirt).desired_width(140.0);
        if shirt_overlong {
            shirt_edit = shirt_edit.background_color(warning_tint(ui.visuals()));
        }
        // The same `add_sized` allocation as the Name field's above.
        let response = ui.add_sized([140.0, ui.spacing().interact_size.y], shirt_edit);
        let response = if shirt_overlong {
            response
                .on_hover_text("PES 2019 and older show 18 letters of a shirt name (PES 2016: 16)")
        } else {
            // The limit the field enforces, not the field's whole capacity:
            // the portable 18 binds on PES 20/21, its own 16 on PES 16, and
            // the charset is single-byte (é passes, ł does not).
            response.on_hover_text(format!(
                "Up to {} letters; Western European accents (é, ü, ñ) are fine, other letters \
                 and symbols are not",
                portable_text_max(PlayerText::ShirtName)
                    .min(text_len(schema, PlayerText::ShirtName))
            ))
        };
        if response.changed() {
            // Upper-cased as typed, then the fit check (the portable limit
            // or the version's own, whichever is shorter; a longer stored
            // name shortens, never lengthens).
            let upper = shirt.to_uppercase();
            if shirt_name_edit_fits(schema.version, &stored_shirt, &upper) {
                edit_player(session, player_id, response.id.value(), |player| {
                    player.shirt_name = upper;
                    player.edit_flags.player = true;
                });
            }
        }
    });
    egui::Grid::new("save_editor_info_strip_basics").show(ui, |ui| {
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
        // The team's `captain` is this roster slot, 0-based as stored: a
        // radio the second click cannot untick (the captain belongs to the
        // slot).
        let response = ui.radio(captain, "");
        if response.clicked() {
            edit_team(session, team_id, response.id.value(), |team| {
                team.tactics.set_pieces.captain = slot;
            });
        }
    });
}

/// The tint of a field whose text a version would crop: its usual
/// background drawn a quarter toward the warning colour.
fn warning_tint(visuals: &egui::Visuals) -> egui::Color32 {
    visuals
        .extreme_bg_color
        .lerp_to_gamma(visuals.warn_fg_color, 0.25)
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

/// The length of the player record's `which` text field, from the schema
/// (the Name field's bytes, and the Shirt field's char count).
fn text_len(schema: &VersionSchema, which: PlayerText) -> usize {
    schema
        .player
        .texts
        .iter()
        .find(|spec| spec.text == which)
        .map(|spec| usize::try_from(spec.len).expect("a field length fits usize"))
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
