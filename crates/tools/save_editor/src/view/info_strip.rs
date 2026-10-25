//! The player header (`plans/save_editor.md` "Editing UI", the player header
//! bullet; the draft's `playerHeader`/`infoStrip`): the selected player's name
//! as the tab's title, then his general fields, drawn at the top of the two
//! player tabs (Abilities & Skills, Appearance). Each control edits a copy of
//! the entry and, when it changes, pushes it under its own `egui::Id` (the
//! undo stack's merge key).

use pes_savefile::model::PlayerEntry;
use pes_savefile::model::names::{crop_player_text, name_fits, shirt_name_edit_fits};
use pes_savefile::schema::fields::{PlayerField, PlayerText, RosterField};
use pes_savefile::schema::{VersionSchema, portable_text_max, schema_for};
use pes_version::PesVersion;
use team_widgets::names::append_name;

use crate::session::Session;
use crate::view::{SaveView, edit_player, edit_team, selected_player};

/// The strip's Name field.
pub(super) const NAME_LABEL: &str = "Name";
/// The strip's Shirt name field.
pub(super) const SHIRT_LABEL: &str = "Shirt";
/// The strip's Height field.
pub(super) const HEIGHT_LABEL: &str = "Height";
/// The strip's Weight field.
pub(super) const WEIGHT_LABEL: &str = "Weight";
/// The strip's Age field.
pub(super) const AGE_LABEL: &str = "Age";
/// The strip's Nation field.
pub(super) const NATION_LABEL: &str = "Nation";

/// A player name as the strip draws it: its `\x11` spelled as the four
/// characters `\\x11` (the draft's `escName`); the field turns it back on
/// edit.
pub(super) fn shown_name(name: &str) -> String {
    name.replace('\x11', "\\x11")
}

/// The header for the selected player; nothing when the selection names no
/// rostered player (a stale slot, an empty save's list).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot, player_id)) = selected_player(view) else {
        return;
    };
    let generation = view.generation;
    let Some(session) = view.session.as_mut() else {
        return;
    };
    // Every widget id under the header is the player's within this open —
    // the salt's `strip` half keys it apart from the tab's own scope (the
    // tab pushes `(generation, player_id)` beside it): a text field's undo
    // history and a DragValue's in-progress edit never cross players — nor
    // saves, since egui keys a TextEdit's undo history by widget id and the
    // same player id of the save opened next would otherwise inherit the
    // last save's.
    ui.push_id(("strip", generation, player_id), |ui| {
        strip(ui, session, team_id, slot, player_id);
    });
}

/// The title (the player's name) over one grid per row: Name, Shirt,
/// Number and Captain on row 1; ID, Nation, Age, Height and Weight on
/// row 2. One grid for both would union the Name field's 240 into row 2's
/// ID column and push Height and Weight off the pane (the plan's "a grid
/// per row").
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
            shown_name(&player.name),
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
    let (height, weight, age) = basics;
    // The name as the tab's title, its colour codes drawn as the player
    // list draws them (`append_name`), at the heading text style.
    let mut title = egui::text::LayoutJob::default();
    append_name(&mut title, &stored_name);
    let heading = egui::TextStyle::Heading.resolve(ui.style());
    for section in &mut title.sections {
        section.format.font_id = heading.clone();
    }
    ui.label(title);
    egui::Grid::new("save_editor_info_strip").show(ui, |ui| {
        ui.label(NAME_LABEL);
        let overlong = stored_name.len() > portable_text_max(PlayerText::Name);
        let mut name_edit = egui::TextEdit::singleline(&mut name).desired_width(190.0);
        if overlong {
            name_edit = name_edit.background_color(warning_tint(ui.visuals()));
        }
        // `add_sized` allocates the width itself: a fresh grid gives a
        // non-last column only its `min_cell_size` (40) on the sizing
        // pass, and a TextEdit caps its width at what the cell offers.
        let response = ui.add_sized([190.0, ui.spacing().interact_size.y], name_edit);
        // A name past what every version holds is drawn tinted with the cut
        // PES 15 to 19 would make: a warning, never a block on Save.
        let response = if overlong {
            response.on_hover_text(format!(
                "In PES 2019 and older the game cuts this name to «{}»",
                shown_name(&crop_player_text(
                    PesVersion::Pes19,
                    PlayerText::Name,
                    &stored_name
                ))
            ))
        } else {
            response.on_hover_text(format!(
                "Up to {} bytes. A plain letter takes one byte, an accented letter takes two \
                 or more, and a colour code takes ten.",
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
        ui.label(SHIRT_LABEL);
        let shirt_overlong =
            stored_shirt.chars().count() > portable_text_max(PlayerText::ShirtName);
        let mut shirt_edit = egui::TextEdit::singleline(&mut shirt).desired_width(110.0);
        if shirt_overlong {
            shirt_edit = shirt_edit.background_color(warning_tint(ui.visuals()));
        }
        // The same `add_sized` allocation as the Name field's above.
        let response = ui.add_sized([110.0, ui.spacing().interact_size.y], shirt_edit);
        let response = if shirt_overlong {
            response.on_hover_text(
                "PES 2019 and older show 18 letters of a shirt name (PES 2016 shows 16)",
            )
        } else {
            // The limit the field enforces, not the field's whole capacity:
            // the portable 18 binds on PES 20/21, its own 16 on PES 16, and
            // the charset is single-byte (é passes, ł does not).
            response.on_hover_text(format!(
                "Up to {} letters. Western European accents (é, ü, ñ) are fine, but other \
                 letters and symbols are not.",
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
        ui.end_row();
    });
    egui::Grid::new("save_editor_info_strip_basics").show(ui, |ui| {
        ui.label("ID");
        ui.monospace(player_id.to_string());
        ui.label(NATION_LABEL);
        let response = ui.add(
            egui::DragValue::new(&mut nation).range(
                0..=u16::try_from(max(schema, PlayerField::Nationality))
                    .expect("the nationality field is 16-bit"),
            ),
        );
        if response.changed() {
            edit_player(session, player_id, response.id.value(), |player| {
                player.basic.nationality = nation;
            });
        }
        number(
            ui,
            session,
            player_id,
            AGE_LABEL,
            age,
            max(schema, PlayerField::Age),
            |player: &mut PlayerEntry, value| player.basic.age = value,
        );
        number(
            ui,
            session,
            player_id,
            HEIGHT_LABEL,
            height,
            max(schema, PlayerField::Height),
            |player: &mut PlayerEntry, value| player.basic.height = value,
        );
        number(
            ui,
            session,
            player_id,
            WEIGHT_LABEL,
            weight,
            max(schema, PlayerField::Weight),
            |player: &mut PlayerEntry, value| player.basic.weight = value,
        );
        ui.end_row();
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

/// The biggest value the save's `schema` stores in `field`'s player-record
/// bit run (`2^width - 1`, the codec's `ValueTooWide` bound): the range the
/// strip's number controls take, so a typed value is never one the codec
/// refuses. The strip's fields are stored on every version.
fn max(schema: &VersionSchema, field: PlayerField) -> u64 {
    schema
        .player
        .max_value(field)
        .expect("the strip's fields are stored on every version")
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
    let max = schema
        .roster
        .max_value(RosterField::Number(0))
        .expect("every version stores the shirt number");
    u16::try_from(max).expect("the shirt number's bound fits u16")
}
