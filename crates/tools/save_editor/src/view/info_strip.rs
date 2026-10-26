//! The player view's title and the first column's panels (`plans/
//! save_editor.md` "Editing UI", the player view bullet; the draft's
//! `playerHeader`, `generalPanel` and `otherStatsPanel`): `<id> <name>` at
//! the heading size, then General info — the Name and Shirt fields each
//! under a label carrying its `n/N` count, the label-left pairs on two
//! halves (each box 54 wide on its half's right edge), then the combos
//! whose labels share one column — and Other stats under it on the same
//! halves so every box lines up. Each control edits a copy of the entry
//! and, when it changes, pushes it under its own `egui::Id` (the undo
//! stack's merge key).

use pes_savefile::interchange::team_toml::labels::{FOOT, POSITIONS, title_case};
use pes_savefile::model::names::{crop_player_text, name_fits, shirt_name_edit_fits};
use pes_savefile::model::{PlayerEntry, PlayerPositions, PlayerStats};
use pes_savefile::schema::fields::{PlayerField, PlayerText, RosterField};
use pes_savefile::schema::{VersionSchema, portable_text_max, schema_for};
use pes_version::PesVersion;
use team_widgets::cards;
use team_widgets::names::append_name;

use crate::session::Session;
use crate::view::abilities::{
    PLAYING_STYLE_LABEL, REGISTERED_POSITION_LABEL, SMALL_STATS, STRONGER_FOOT_LABEL,
    STRONGER_HAND_LABEL, SmallStat, foot_label, gating_tooltip, position_label,
};
use crate::view::{SaveView, edit_player, edit_team, selected_player};

/// The General info panel's Name field.
pub(super) const NAME_LABEL: &str = "Name";
/// The General info panel's Shirt name field.
pub(super) const SHIRT_LABEL: &str = "Shirt";
/// The General info panel's Height field.
pub(super) const HEIGHT_LABEL: &str = "Height";
/// The General info panel's Weight field.
pub(super) const WEIGHT_LABEL: &str = "Weight";
/// The General info panel's Age field.
pub(super) const AGE_LABEL: &str = "Age";
/// The General info panel's Nation field.
pub(super) const NATION_LABEL: &str = "Nation";
/// The General info panel's Number field.
pub(super) const NUMBER_LABEL: &str = "Number";

/// The General info column's width (the draft's `.pcol.first` 350px: room
/// for the longest pair label, "Injury resistance", beside its box on
/// each half).
pub(super) const GENERAL_WIDTH: f32 = 350.0;
/// A pair half's width: two halves 14 points apart inside the column.
const HALF_WIDTH: f32 = (GENERAL_WIDTH - 14.0) / 2.0;
/// A pair's box width (the draft's `.pair` fields).
const BOX_WIDTH: f32 = 54.0;

/// A player name as the strip draws it: its `\x11` spelled as the four
/// characters `\\x11` (the draft's `escName`); the field turns it back on
/// edit.
pub(super) fn shown_name(name: &str) -> String {
    name.replace('\x11', "\\x11")
}

/// The header for the selected player; nothing when the selection names no
/// rostered player (a stale slot, an empty save's list).
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((_, _, player_id)) = selected_player(view) else {
        return;
    };
    let Some(session) = view.session.as_ref() else {
        return;
    };
    let name = session
        .file()
        .player(player_id)
        .expect("the selected player")
        .name
        .clone();
    // `<id> <name>` as the view's title (the plan's "No view draws the
    // tool's name" bullet): the id in the weak colour, the name's colour
    // codes drawn as the player list draws them (`append_name`), all at
    // the heading text style.
    let mut title = egui::text::LayoutJob::default();
    title.append(
        &format!("{player_id} "),
        0.0,
        egui::TextFormat::simple(egui::FontId::default(), ui.visuals().weak_text_color()),
    );
    append_name(&mut title, &name);
    let heading = egui::TextStyle::Heading.resolve(ui.style());
    for section in &mut title.sections {
        section.format.font_id = heading.clone();
    }
    ui.label(title);
}

/// The General info panel's content (`generalPanel`): the Name and Shirt
/// counted fields, the pair rows, then the combos.
pub(super) fn general(
    ui: &mut egui::Ui,
    session: &mut Session,
    team_id: u32,
    slot: u8,
    player_id: u32,
) {
    let (
        mut name,
        stored_name,
        mut shirt,
        stored_shirt,
        nation,
        mut shirt_number,
        basics,
        stored_captain,
        positions,
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
            player.positions.clone(),
            schema_for(file.version()),
        )
    };
    let captain = stored_captain == slot;
    let (height, weight, age) = basics;
    let width = ui.available_width().min(GENERAL_WIDTH);

    // The `label n/max` line over the field: the label's rect is taken
    // first, the label itself painted after the field — the count follows
    // the field's own text this frame (a TextEdit writes its buffer during
    // `ui.add`), not the stored name. The field and its count carry one
    // hover.
    let overlong = stored_name.len() > portable_text_max(PlayerText::Name);
    let (name_label_id, name_label) = ui.allocate_space(egui::vec2(
        width,
        ui.text_style_height(&egui::TextStyle::Body),
    ));
    let mut name_edit = egui::TextEdit::singleline(&mut name);
    if overlong {
        name_edit = name_edit.background_color(warning_tint(ui.visuals()));
    }
    let response = ui.add_sized([width, ui.spacing().interact_size.y], name_edit);
    // A name past what every version holds is drawn tinted with the cut
    // PES 15 to 19 would make: a warning, never a block on Save.
    let tip = if overlong {
        format!(
            "In PES 2019 and older the game cuts this name to «{}»",
            shown_name(&crop_player_text(
                PesVersion::Pes19,
                PlayerText::Name,
                &stored_name
            ))
        )
    } else {
        format!(
            "Up to {} bytes. A plain letter takes one byte, an accented letter takes two or \
             more, and a colour code takes ten. A colour code is typed as \\x11c and eight hex \
             digits.",
            text_len(schema, PlayerText::Name)
        )
    };
    counted_label(
        ui,
        name_label_id,
        name_label,
        NAME_LABEL,
        name.replace("\\x11", "\x11").len(),
        text_len(schema, PlayerText::Name),
        overlong,
    )
    .on_hover_text(&tip);
    let response = response.on_hover_text(&tip);
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

    let shirt_overlong = stored_shirt.chars().count() > portable_text_max(PlayerText::ShirtName);
    let (shirt_label_id, shirt_label) = ui.allocate_space(egui::vec2(
        width,
        ui.text_style_height(&egui::TextStyle::Body),
    ));
    let mut shirt_edit = egui::TextEdit::singleline(&mut shirt);
    if shirt_overlong {
        shirt_edit = shirt_edit.background_color(warning_tint(ui.visuals()));
    }
    let response = ui.add_sized([width, ui.spacing().interact_size.y], shirt_edit);
    let tip = if shirt_overlong {
        "PES 2019 and older show 18 letters of a shirt name (PES 2016 shows 16)".to_owned()
    } else {
        // The limit the field enforces, not the field's whole capacity:
        // the portable 18 binds on PES 20/21, its own 16 on PES 16, and
        // the charset is single-byte (é passes, ł does not).
        format!(
            "Up to {} letters. Western European accents (é, ü, ñ) are fine, but other letters \
             and symbols are not.",
            portable_text_max(PlayerText::ShirtName).min(text_len(schema, PlayerText::ShirtName))
        )
    };
    counted_label(
        ui,
        shirt_label_id,
        shirt_label,
        SHIRT_LABEL,
        shirt.chars().count(),
        portable_text_max(PlayerText::ShirtName).min(text_len(schema, PlayerText::ShirtName)),
        shirt_overlong,
    )
    .on_hover_text(&tip);
    let response = response.on_hover_text(&tip);
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

    // The label-left pairs on two halves: each label at its half's left,
    // its 54-wide box on the half's right edge.
    pair_row(ui, |ui| {
        pair_half(ui, HEIGHT_LABEL, |ui| {
            number_box(
                ui,
                session,
                player_id,
                height,
                max(schema, PlayerField::Height),
                |player, shown| player.basic.height = shown,
            );
        });
        pair_half(ui, NUMBER_LABEL, |ui| {
            let response = ui.add_sized(
                [BOX_WIDTH, ui.spacing().interact_size.y],
                egui::DragValue::new(&mut shirt_number).range(0..=roster_number_max(schema)),
            );
            if response.changed() {
                edit_team(session, team_id, response.id.value(), |team| {
                    team.roster[usize::from(slot)].number = shirt_number;
                });
            }
        });
    });
    pair_row(ui, |ui| {
        pair_half(ui, WEIGHT_LABEL, |ui| {
            number_box(
                ui,
                session,
                player_id,
                weight,
                max(schema, PlayerField::Weight),
                |player, shown| player.basic.weight = shown,
            );
        });
        pair_half(ui, NATION_LABEL, |ui| {
            number_box(
                ui,
                session,
                player_id,
                nation,
                max(schema, PlayerField::Nationality),
                |player, shown| player.basic.nationality = shown,
            );
        });
    });
    pair_row(ui, |ui| {
        pair_half(ui, AGE_LABEL, |ui| {
            number_box(
                ui,
                session,
                player_id,
                age,
                max(schema, PlayerField::Age),
                |player, shown| player.basic.age = shown,
            );
        });
        pair_half(ui, "Captain", |ui| {
            // The team's `captain` is this roster slot, 0-based as stored: a
            // radio the second click cannot untick (the captain belongs to
            // the slot) — at the left edge of its half's box.
            ui.allocate_ui(egui::vec2(BOX_WIDTH, ui.spacing().interact_size.y), |ui| {
                ui.horizontal(|ui| {
                    let response = ui.radio(captain, "");
                    if response.clicked() {
                        edit_team(session, team_id, response.id.value(), |team| {
                            team.tactics.set_pieces.captain = slot;
                        });
                    }
                });
            });
        });
    });

    // The combos, their labels in one column, every combo starting at one
    // x and ending on the column's right edge.
    combos(ui, session, player_id, &positions, schema.version);
}

/// One row of the pairs' two halves, 14 points apart.
fn pair_row(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        add(ui);
    });
}

/// One `label`-left/`box`-right pair on a `HALF_WIDTH` half (the draft's
/// `.pair`): the label at the half's left, `add_box`'s 54-wide box on its
/// right edge.
fn pair_half(ui: &mut egui::Ui, label: &str, add_box: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui(egui::vec2(HALF_WIDTH, ui.spacing().interact_size.y), |ui| {
        ui.horizontal(|ui| {
            ui.weak(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), add_box);
        });
    });
}

/// One pair's 54-wide `DragValue` over `shown` capped at `max`; a change
/// pushes the player's entry with `set` applied.
fn number_box<N: egui::emath::Numeric + TryFrom<u64>>(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    shown: N,
    max: u64,
    set: impl FnOnce(&mut PlayerEntry, N),
) {
    let mut shown = shown;
    let response = ui.add_sized(
        [BOX_WIDTH, ui.spacing().interact_size.y],
        egui::DragValue::new(&mut shown)
            .range(N::from_f64(0.0)..=N::try_from(max).unwrap_or(N::MAX)),
    );
    if response.changed() {
        edit_player(session, player_id, response.id.value(), |player| {
            set(player, shown);
        });
    }
}

/// The Other stats panel's content (`otherStatsPanel`): the four shifted
/// stored fields on General info's halves, so every box of both panels
/// lines up — Injury resistance with Form, then "Weak foot: Usage" and
/// "Accuracy" (the two sharing one label).
pub(super) fn other_stats(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    stats: &PlayerStats,
) {
    let pairs = [
        [&SMALL_STATS[1], &SMALL_STATS[0]],
        [&SMALL_STATS[2], &SMALL_STATS[3]],
    ];
    for [left, right] in pairs {
        pair_row(ui, |ui| {
            pair_half(ui, left.shown, |ui| {
                small_box(ui, session, player_id, stats, left);
            });
            pair_half(ui, right.shown, |ui| {
                small_box(ui, session, player_id, stats, right);
            });
        });
    }
}

/// `small`'s box for a pair half: the shifted `DragValue` for a field the
/// version stores (a stored value past the shown range drawn as stored),
/// the disabled `-` for one it lacks.
fn small_box(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    stats: &PlayerStats,
    small: &SmallStat,
) {
    match (small.get)(stats) {
        Some(stored) => {
            let mut shown = stored + small.shift;
            let response = ui.add_sized(
                [BOX_WIDTH, ui.spacing().interact_size.y],
                egui::DragValue::new(&mut shown)
                    .range(small.min..=small.max)
                    .clamp_existing_to_range(false),
            );
            if response.changed() {
                edit_player(session, player_id, response.id.value(), move |player| {
                    (small.set)(&mut player.stats, shown - small.shift);
                });
            }
        }
        None => {
            ui.add_enabled(
                false,
                egui::Button::new("-")
                    .min_size(egui::vec2(BOX_WIDTH, ui.spacing().interact_size.y)),
            )
            .on_disabled_hover_text(gating_tooltip(small.field));
        }
    }
}

/// The four combos, their labels in one column, every combo starting at
/// one x and ending on the column's right edge (`generalPanel`'s
/// `.combos`): the labels' widest measured width plus the gap leaves the
/// rest of the column to the buttons.
fn combos(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    positions: &PlayerPositions,
    version: PesVersion,
) {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let label_width = [
        STRONGER_FOOT_LABEL,
        STRONGER_HAND_LABEL,
        REGISTERED_POSITION_LABEL,
        PLAYING_STYLE_LABEL,
    ]
    .iter()
    .map(|label| {
        ui.painter()
            .layout_no_wrap((*label).to_owned(), body.clone(), egui::Color32::WHITE)
            .size()
            .x
    })
    .fold(0.0, f32::max);
    let combo_width = (ui.available_width() - label_width - ui.spacing().item_spacing.x).max(0.0);
    egui::Grid::new("general combos")
        .num_columns(2)
        .show(ui, |ui| {
            ui.weak(STRONGER_FOOT_LABEL);
            let mut foot = positions.stronger_foot;
            let foot_combo = egui::ComboBox::from_id_salt("abilities stronger foot")
                .selected_text(foot_label(foot))
                .width(combo_width)
                .show_ui(ui, |ui| {
                    for (index, label) in FOOT.iter().enumerate() {
                        ui.selectable_value(&mut foot, index as u8, title_case(label));
                    }
                });
            if foot != positions.stronger_foot {
                edit_player(
                    session,
                    player_id,
                    foot_combo.response.id.value(),
                    |player| {
                        player.positions.stronger_foot = foot;
                    },
                );
            }
            ui.end_row();
            ui.weak(STRONGER_HAND_LABEL);
            match positions.stronger_hand {
                Some(stored) => {
                    let mut hand = stored;
                    let hand_combo = egui::ComboBox::from_id_salt("abilities stronger hand")
                        .selected_text(foot_label(hand))
                        .width(combo_width)
                        .show_ui(ui, |ui| {
                            for (index, label) in FOOT.iter().enumerate() {
                                ui.selectable_value(&mut hand, index as u8, title_case(label));
                            }
                        });
                    if hand != stored {
                        edit_player(
                            session,
                            player_id,
                            hand_combo.response.id.value(),
                            |player| {
                                player.positions.stronger_hand = Some(hand);
                            },
                        );
                    }
                }
                None => {
                    // The version has no stronger hand: the combo stays, at
                    // Right, disabled with the `PES NN+` tooltip (SE-GUI-06).
                    ui.add_enabled_ui(false, |ui| {
                        egui::ComboBox::from_id_salt("abilities stronger hand")
                            .selected_text(foot_label(0))
                            .width(combo_width)
                            .show_ui(ui, |_ui| ());
                    })
                    .response
                    .on_disabled_hover_text(gating_tooltip(PlayerField::StrongerHand));
                }
            }
            ui.end_row();
            ui.weak(REGISTERED_POSITION_LABEL);
            let mut registered = positions.registered;
            let combo = egui::ComboBox::from_id_salt("abilities registered position")
                .selected_text(position_label(registered))
                .width(combo_width)
                .show_ui(ui, |ui| {
                    for (index, label) in POSITIONS.iter().enumerate() {
                        ui.selectable_value(&mut registered, index as u8, *label);
                    }
                });
            if registered != positions.registered {
                edit_player(session, player_id, combo.response.id.value(), |player| {
                    player.positions.registered = registered;
                });
            }
            ui.end_row();
            ui.weak(PLAYING_STYLE_LABEL);
            ui.scope(|ui| {
                ui.spacing_mut().combo_width = combo_width;
                let mut style = positions.playing_style;
                if cards::playing_style_picker(ui, "abilities playing style", version, &mut style) {
                    edit_player(
                        session,
                        player_id,
                        egui::Id::unique(("abilities playing style", player_id)).value(),
                        |player| {
                            player.positions.playing_style = style;
                        },
                    );
                }
            });
            ui.end_row();
        });
}

/// The `label n/max` line painted at `rect` under `id` (allocated before
/// the field was drawn, so the count follows the field's text as typed
/// this frame): `label` in the weak colour, `n/max` the same — or the
/// warning colour when `warn` (the count past the portable limit). Its
/// AccessKit node returned for the hover it shares with the field.
fn counted_label(
    ui: &mut egui::Ui,
    id: egui::Id,
    rect: egui::Rect,
    label: &str,
    n: usize,
    max: usize,
    warn: bool,
) -> egui::Response {
    let weak = ui.visuals().weak_text_color();
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mut job = egui::text::LayoutJob::default();
    job.append(
        &format!("{label} "),
        0.0,
        egui::TextFormat::simple(body.clone(), weak),
    );
    job.append(
        &format!("{n}/{max}"),
        0.0,
        egui::TextFormat::simple(
            body,
            if warn {
                ui.visuals().warn_fg_color
            } else {
                weak
            },
        ),
    );
    let galley = ui.painter().layout_job(job);
    // `interact`, not `allocate_rect`: the space was taken before the
    // field drew, so this must not move the cursor again.
    let response = ui.interact(rect, id, egui::Sense::hover());
    response.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::accesskit::Role::Label,
            true,
            format!("{label} {n}/{max}"),
        )
    });
    ui.painter().galley(rect.min, galley, weak);
    response
}

/// The tint of a field whose text a version would crop: its usual
/// background drawn a quarter toward the warning colour.
fn warning_tint(visuals: &egui::Visuals) -> egui::Color32 {
    visuals
        .extreme_bg_color
        .lerp_to_gamma(visuals.warn_fg_color, 0.25)
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
