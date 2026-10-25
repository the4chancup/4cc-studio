//! The Abilities & Skills tab (`plans/save_editor.md` "Editing UI", the
//! Abilities & Skills bullet; the draft's `abilitiesTab`): a top row of
//! three blocks — the Profile grid (registered position, playing style,
//! stronger foot and hand, the four shifted stored fields), the 13
//! playable positions' C/B/A ratings in the collaborator's grid, and the
//! quick actions column — then the 25 ability stats in their columns and
//! the skills and COM playing styles with the active ruleset's Cards left
//! badges. Star and playing attitude, which change only cutscenes, sit in
//! the Appearance tab's Cutscenes group (`appearance::CUTSCENE_STATS`).
//! Every control edits a copy of the player's entry and, when it changes,
//! pushes it with `Session::edit_player` under the control's `egui::Id`
//! value, as `info_strip.rs` does; edit flags are not touched. A field
//! the save's version does not store is drawn in its place disabled with
//! a `PES NN+` tooltip naming the first version that has it.

use std::ops::RangeInclusive;

use aatf::{CardLimits, TeamView};
use pes_savefile::interchange::team_toml::labels::{
    ABILITY_STATS, FOOT, PLAYABLE_POSITIONS, POSITIONS, title_case,
};
use pes_savefile::model::{PlayerEntry, PlayerPositions, PlayerSkills, PlayerStats};
use pes_savefile::schema::fields::{ABILITY_FIELDS, PlayerField};
use pes_savefile::schema::first_version_with;
use team_widgets::cards::{self, TierCards};

use crate::session::Session;
use crate::view::{
    ActiveRules, BadgeKey, SaveView, edit_player, quick_actions, rules_load_error, scoped,
    selected_player,
};

/// The Profile grid's registered-position combo.
pub(super) const REGISTERED_POSITION_LABEL: &str = "Registered position";
/// The Profile grid's playing-style picker.
pub(super) const PLAYING_STYLE_LABEL: &str = "Playing style";
/// The Profile grid's stronger-foot combo.
pub(super) const STRONGER_FOOT_LABEL: &str = "Stronger foot";
/// The Profile grid's stronger-hand combo.
pub(super) const STRONGER_HAND_LABEL: &str = "Stronger hand";

/// The playable positions' ratings, `(label, stored)` pairs: a stored
/// value past the table selects none of the three.
pub(super) const RATING_LABELS: [(&str, u8); 3] = [("C", 0), ("B", 1), ("A", 2)];

/// The playable positions' group heading; a diff row's own label starts
/// from it (`terms::row_term`).
pub(super) const PLAYABLE_POSITIONS_LABEL: &str = "Playable positions";

/// The tab for the selected player, under `info_strip`'s header: the top
/// row of three blocks (Profile, the playable positions' grid, the quick
/// actions column), each wrapping whole on a narrow pane, then the stats
/// in their columns and the skills.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot, player_id)) = selected_player(view) else {
        return;
    };
    let (version, positions, stats, mut skills) = {
        let Some(session) = view.session.as_ref() else {
            return;
        };
        let file = session.file();
        let player = file.player(player_id).expect("the selected player");
        (
            file.version(),
            player.positions.clone(),
            player.stats.clone(),
            player.skills.clone(),
        )
    };
    // Every widget id under the tab is the player's within this open: a
    // DragValue's in-progress edit, a combo's popup and the pickers' state
    // never cross players — nor saves, since egui keys a TextEdit's undo
    // history by widget id and the same player id of the save opened next
    // would otherwise inherit the last save's (and the undo merge keys
    // differ per player, which is right).
    let generation = view.generation;
    ui.push_id((generation, player_id), |ui| {
        top_row(ui, view, player_id, version, &positions, &stats);
        ui.separator();
        ui.strong("Ability stats");
        let Some(session) = view.session.as_mut() else {
            return;
        };
        ability_stats(ui, session, player_id, &stats);
        ui.separator();
        ui.strong("Skills");
        cards_row(
            ui,
            view.rules.as_ref(),
            &mut view.cards,
            session,
            (team_id, slot, player_id),
            version,
            &mut skills,
        );
    });
}

/// The width the three blocks take on one line, measured once in a
/// headless frame at 1600 x 1400 with each Profile combo at its widest
/// option (the playing style's Defensive Goalkeeper; the registered
/// position, foot and hand combos stay at their minimum `combo_width`
/// whatever they show): the drawn `min_rect` of each
/// `ui.vertical` block summed with the two item gaps is Profile ~510,
/// Playable positions ~474, Quick actions ~116, so ~1117 points. A wrong
/// estimate wraps a block a bit early or late; it never splits one,
/// since the fallback is whole blocks on two lines.
pub(super) const TOP_ROW_ONE_LINE: f32 = 1117.0;

/// The top row (the draft's `topblocks`): the Profile grid, the positions
/// grid and the quick actions column, top-aligned, on one line where the
/// pane holds `TOP_ROW_ONE_LINE`; else Profile alone on the first line
/// and the positions grid with the quick actions on the second — a block
/// moves whole, never split. The session is re-taken per block so the
/// quick actions can borrow the whole view.
fn top_row(
    ui: &mut egui::Ui,
    view: &mut SaveView,
    player_id: u32,
    version: pes_version::PesVersion,
    positions: &PlayerPositions,
    stats: &PlayerStats,
) {
    // Each block under `scoped`'s fixed `scope_id` in both arrangements:
    // an egui id follows call order inside a scope, so crossing
    // `TOP_ROW_ONE_LINE` would re-key the widgets — a DragValue being
    // typed into while the window is resized would hand its in-progress
    // edit to another control. The ids are taken on `ui` here, before
    // the `horizontal_top` child would put them under its own scope.
    let profile_id = ui.make_persistent_id("profile");
    let positions_id = ui.make_persistent_id("positions");
    let actions_id = ui.make_persistent_id("actions");
    let profile_block = |ui: &mut egui::Ui, view: &mut SaveView| {
        scoped(ui, profile_id, |ui| {
            ui.vertical(|ui| {
                ui.strong("Profile");
                if let Some(session) = view.session.as_mut() {
                    profile(ui, session, player_id, version, positions, stats);
                }
            });
        });
    };
    let positions_block = |ui: &mut egui::Ui, view: &mut SaveView| {
        scoped(ui, positions_id, |ui| {
            ui.vertical(|ui| {
                ui.strong(PLAYABLE_POSITIONS_LABEL);
                if let Some(session) = view.session.as_mut() {
                    playable_positions(ui, session, player_id, positions);
                }
            });
        });
    };
    let actions_block = |ui: &mut egui::Ui, view: &mut SaveView| {
        scoped(ui, actions_id, |ui| {
            ui.vertical(|ui| {
                quick_actions::show(ui, view);
            });
        });
    };
    if ui.available_width() >= TOP_ROW_ONE_LINE {
        ui.horizontal_top(|ui| {
            profile_block(ui, view);
            positions_block(ui, view);
            actions_block(ui, view);
        });
    } else {
        profile_block(ui, view);
        ui.horizontal_top(|ui| {
            positions_block(ui, view);
            actions_block(ui, view);
        });
    }
}

/// The count of label/field pairs that fit `ui.available_width()` at
/// `pair_width` each: how many columns a grid takes at the pane's width
/// (the estimate is a typical pair's width, so a slightly wider pair
/// still lands inside). At least one.
pub(crate) fn pairs_per_row(ui: &egui::Ui, pair_width: f32) -> usize {
    (ui.available_width() / pair_width).floor().max(1.0) as usize
}

/// `end_row` after `index` pairs (1-based) of `per_row` are done, so a row
/// takes as many pairs as `per_row` allows.
pub(crate) fn maybe_end_row(ui: &mut egui::Ui, index: usize, per_row: usize) {
    if index.is_multiple_of(per_row) {
        ui.end_row();
    }
}

/// The Profile block's grid (the draft's `#profile-grid`): two
/// label/field pairs a row, the labels in their own cells — registered
/// position with playing style, stronger foot with stronger hand (a
/// missing hand stays a disabled combo at Right), then `SMALL_STATS`' four
/// pairs.
fn profile(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: pes_version::PesVersion,
    positions: &PlayerPositions,
    stats: &PlayerStats,
) {
    egui::Grid::new("abilities profile").show(ui, |ui| {
        ui.label(REGISTERED_POSITION_LABEL);
        let mut registered = positions.registered;
        let combo = egui::ComboBox::from_id_salt("abilities registered position")
            .selected_text(position_label(registered))
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
        ui.label(PLAYING_STYLE_LABEL);
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
        ui.end_row();
        ui.label(STRONGER_FOOT_LABEL);
        let mut foot = positions.stronger_foot;
        let foot_combo = egui::ComboBox::from_id_salt("abilities stronger foot")
            .selected_text(foot_label(foot))
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
        ui.label(STRONGER_HAND_LABEL);
        match positions.stronger_hand {
            Some(stored) => {
                let mut hand = stored;
                let hand_combo = egui::ComboBox::from_id_salt("abilities stronger hand")
                    .selected_text(foot_label(hand))
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
                        .show_ui(ui, |_ui| ());
                })
                .response
                .on_disabled_hover_text(gating_tooltip(PlayerField::StrongerHand));
            }
        }
        ui.end_row();
        for pair in SMALL_STATS.chunks(2) {
            for small in pair {
                small_stat(ui, session, player_id, stats, small);
            }
            ui.end_row();
        }
    });
}

/// One small stored field of the Profile block or the Appearance tab's
/// Cutscenes group: label, its schema field, the shown range and the
/// shift between shown and stored (the shifted fields are stored one
/// below the game's numbers, the draft's `+ 1`/`v - 1`).
#[derive(Clone, Copy)]
pub(super) struct SmallStat {
    /// The control's label.
    pub(super) label: &'static str,
    /// The schema field gating it.
    pub(super) field: PlayerField,
    /// The shown range.
    pub(super) min: u8,
    /// The shown range's top.
    pub(super) max: u8,
    /// `shown - shift` is the stored value.
    pub(super) shift: u8,
    /// The stored value (`None` on a version that does not store it).
    pub(super) get: fn(&PlayerStats) -> Option<u8>,
    /// The stored write.
    pub(super) set: fn(&mut PlayerStats, u8),
}

/// The Profile block's four stored fields, in the draft's order: form 1-8,
/// injury resistance 1-3, weak foot usage and accuracy 1-4 (each stored
/// one lower). Star and playing attitude, which change only cutscenes,
/// sit in the Appearance tab's `CUTSCENE_STATS`.
pub(super) const SMALL_STATS: [SmallStat; 4] = [
    SmallStat {
        label: "Form",
        field: PlayerField::Form,
        min: 1,
        max: 8,
        shift: 1,
        get: |stats| Some(stats.form),
        set: |stats, value| stats.form = value,
    },
    SmallStat {
        label: "Injury resistance",
        field: PlayerField::InjuryResistance,
        min: 1,
        max: 3,
        shift: 1,
        get: |stats| Some(stats.injury_resistance),
        set: |stats, value| stats.injury_resistance = value,
    },
    SmallStat {
        label: "Weak foot usage",
        field: PlayerField::WeakFootUsage,
        min: 1,
        max: 4,
        shift: 1,
        get: |stats| Some(stats.weak_foot_usage),
        set: |stats, value| stats.weak_foot_usage = value,
    },
    SmallStat {
        label: "Weak foot accuracy",
        field: PlayerField::WeakFootAccuracy,
        min: 1,
        max: 4,
        shift: 1,
        get: |stats| Some(stats.weak_foot_accuracy),
        set: |stats, value| stats.weak_foot_accuracy = value,
    },
];

/// One `SmallStat`'s label/control pair: `number` for a stored field, the
/// `-` stand-in for a gated one (the label stays too: the gated control
/// keeps its cell). The Profile grid and the Appearance tab's Cutscenes
/// group share it.
pub(super) fn small_stat(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    stats: &PlayerStats,
    small: &SmallStat,
) {
    match (small.get)(stats) {
        Some(stored) => {
            number(
                ui,
                session,
                player_id,
                small.label,
                stored + small.shift,
                small.min..=small.max,
                move |player, shown| {
                    (small.set)(&mut player.stats, shown - small.shift);
                },
            );
        }
        None => {
            ui.label(small.label);
            gated_number(ui, small.field);
        }
    }
}

/// The positions grid's rows (the draft's `POS_ROWS`): the order a member
/// reads a formation in.
const POS_ROWS: [&[&str]; 3] = [
    &["CF", "SS", "LWF", "RWF"],
    &["AMF", "LMF", "RMF", "CMF", "DMF"],
    &["CB", "LB", "RB", "GK"],
];

/// The 13 positions in `POS_ROWS`, each cell the position's label above
/// the three `C` `B` `A` ratings (stored 0, 1, 2; a stored 3 selects
/// none), the registered position's cell outlined in the selection colour
/// (the draft's `td.reg`). `playable[i]` is
/// `POSITIONS[PLAYABLE_POSITIONS[i]]`'s rating: the label's index is not
/// the array's.
fn playable_positions(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    positions: &PlayerPositions,
) {
    let mut playable = positions.playable;
    egui::Grid::new("abilities playable").show(ui, |ui| {
        for row in POS_ROWS {
            for name in row {
                let index = POSITIONS
                    .iter()
                    .position(|position| position == name)
                    .expect("a POS_ROWS name is a POSITIONS name");
                let at = PLAYABLE_POSITIONS
                    .iter()
                    .position(|&position| position == index)
                    .expect("PLAYABLE_POSITIONS is a permutation of 0..13");
                let cell = ui.vertical(|ui| {
                    ui.label(*name);
                    ui.horizontal(|ui| {
                        for (label, rating) in RATING_LABELS {
                            let response = ui.selectable_value(&mut playable[at], rating, label);
                            if response.changed() {
                                edit_player(session, player_id, response.id.value(), |player| {
                                    player.positions.playable = playable;
                                });
                            }
                        }
                    });
                });
                if index == usize::from(positions.registered) {
                    ui.painter().rect_stroke(
                        cell.response.rect.expand(2.0),
                        egui::CornerRadius::ZERO,
                        egui::Stroke::new(1.0, ui.visuals().selection.stroke.color),
                        egui::StrokeKind::Outside,
                    );
                }
            }
            ui.end_row();
        }
    });
}

/// The 25 stats of `ABILITY_STATS` in aligned columns filled column by
/// column in the game's order (a row-by-row fill misaligns them as the
/// pane narrows): four columns where the pane holds them, fewer when it
/// cannot, an empty pair where the last column runs out of stats. Each
/// pair is a title-cased label and a `DragValue` 40-99; a stat the
/// version lacks shows `-`, disabled.
fn ability_stats(ui: &mut egui::Ui, session: &mut Session, player_id: u32, stats: &PlayerStats) {
    let abilities = stats.abilities();
    let cols = pairs_per_row(ui, 170.0).min(4);
    let rows = ABILITY_STATS.len().div_ceil(cols);
    egui::Grid::new("abilities stats").show(ui, |ui| {
        for r in 0..rows {
            for c in 0..cols {
                let index = c * rows + r;
                if index >= ABILITY_STATS.len() {
                    // The draft's empty pair: the last column runs out.
                    ui.label("");
                    ui.label("");
                    continue;
                }
                ui.label(title_case(ABILITY_STATS[index]));
                // Each stat's field under `scoped`'s scope id: an id
                // follows call order inside a scope, so a change of
                // column count would re-key it (the label keeps its own
                // cell; the scope wraps the field cell only, so the grid
                // still sees two cells).
                scoped(ui, ui.make_persistent_id(index), |ui| {
                    match abilities[index] {
                        Some(value) => {
                            number_field(
                                ui,
                                session,
                                player_id,
                                value,
                                40..=99,
                                move |player, shown| player.stats.set_ability(index, shown),
                            );
                        }
                        // The gated control keeps its cell too.
                        None => {
                            gated_number(ui, ABILITY_FIELDS[index]);
                        }
                    }
                });
            }
            ui.end_row();
        }
    });
}

/// `card_pickers` over the player's skills, with one `TierCards` per tier
/// of the loaded ruleset in its order; the load error stands in for the
/// badges. The tiers' `card_limits` results live in `cards` under the
/// `BadgeKey` they were computed for — a `Session::revision` bump, a
/// selection change or another rules file recomputes them, and any other
/// frame draws the kept ones.
fn cards_row(
    ui: &mut egui::Ui,
    rules: Option<&ActiveRules>,
    cards: &mut Option<(BadgeKey, Vec<(String, CardLimits)>)>,
    session: &mut Session,
    (team_id, slot, player_id): (u32, u8, u32),
    version: pes_version::PesVersion,
    skills: &mut PlayerSkills,
) {
    let mut load_error = None;
    if let Some(active) = rules {
        match &active.rules {
            Ok(rules) => {
                let key = BadgeKey {
                    revision: session.revision(),
                    team: team_id,
                    slot,
                    rules_file: active.file.clone(),
                };
                if cards.as_ref().is_none_or(|(held, _)| *held != key) {
                    // A tier whose `card_limits` fails is left out of the
                    // row (the AATF panel reports check errors).
                    let file = session.file();
                    let mut list = Vec::new();
                    if let Ok(team_view) = TeamView::new(file, team_id)
                        && let Some(player_view) = team_view.player(slot)
                    {
                        for tier in &rules.ruleset().tiers {
                            if let Ok(limits) = rules.card_limits(&team_view, player_view, &tier.id)
                            {
                                list.push((tier.label.clone(), limits));
                            }
                        }
                    }
                    *cards = Some((key, list));
                }
            }
            // The error stands in for the badges: stale ones go too.
            Err(error) => {
                load_error = Some(error.as_str());
                *cards = None;
            }
        }
    }
    let card_list = cards.as_ref().map_or(&[][..], |(_, list)| list.as_slice());
    if let Some(error) = load_error {
        ui.horizontal_wrapped(|ui| {
            ui.weak("Cards left");
            ui.weak(rules_load_error(error));
        });
    }
    let tiers: Vec<TierCards> = card_list
        .iter()
        .map(|(label, limits)| TierCards { label, limits })
        .collect();
    if cards::card_pickers(ui, "abilities cards", version, skills, &tiers) {
        let skills = skills.clone();
        edit_player(
            session,
            player_id,
            egui::Id::unique(("abilities cards", player_id)).value(),
            |player| {
                player.skills = skills;
            },
        );
    }
}

/// A `label`/`DragValue` pair over `shown` in `range`; on a change `store`
/// maps the shown value onto the player's stored one.
/// `clamp_existing_to_range` stays off: an egui number control clamps the
/// value it draws unless told not to, and a stored value past the range
/// would come back changed on the frame that drew it (drawing a player
/// never edits him).
pub(crate) fn number<T: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    label: impl Into<egui::WidgetText>,
    shown: T,
    range: RangeInclusive<T>,
    store: impl FnOnce(&mut PlayerEntry, T),
) -> egui::Response {
    ui.label(label);
    number_field(ui, session, player_id, shown, range, store)
}

/// `number`'s `DragValue` alone, for a caller drawing the label itself
/// (`ability_stats` wraps the field cell in the stat's `scoped` scope, so
/// the label keeps its own cell while the field keeps a stable id across
/// column counts).
fn number_field<T: egui::emath::Numeric>(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    shown: T,
    range: RangeInclusive<T>,
    store: impl FnOnce(&mut PlayerEntry, T),
) -> egui::Response {
    let mut shown = shown;
    let response = ui.add(
        egui::DragValue::new(&mut shown)
            .range(range)
            .clamp_existing_to_range(false),
    );
    if response.changed() {
        edit_player(session, player_id, response.id.value(), |player| {
            store(player, shown);
        });
    }
    response
}

/// The disabled stand-in for a number the save's version does not store:
/// `-` inside the control's usual bounds (a Button at the DragValue's
/// minimum size, so an enabled version's control takes the same rect,
/// SE-GUI-06) with the `PES NN+` tooltip.
pub(crate) fn gated_number(ui: &mut egui::Ui, field: PlayerField) {
    ui.add_enabled(
        false,
        egui::Button::new("-").min_size(ui.spacing().interact_size),
    )
    .on_disabled_hover_text(gating_tooltip(field));
}

/// The tooltip naming the first version that stores `field` (`PES NN+`).
pub(crate) fn gating_tooltip(field: PlayerField) -> String {
    format!(
        "PES {}+",
        first_version_with(field)
            .expect("a gated field is stored by some version")
            .number()
    )
}

/// `stored`'s position name as the registered-position combo shows it,
/// `?` past the table.
pub(super) fn position_label(stored: u8) -> &'static str {
    POSITIONS.get(usize::from(stored)).copied().unwrap_or("?")
}

/// `stored`'s foot/hand label (`FOOT` title-cased), its number when past
/// the table.
pub(super) fn foot_label(stored: u8) -> String {
    FOOT.get(usize::from(stored))
        .map(|name| title_case(name))
        .unwrap_or_else(|| stored.to_string())
}

#[cfg(test)]
mod tests {
    use pes_savefile::schema::fields::ABILITY_FIELDS;
    use pes_savefile::schema::fields::PlayerField;
    use pes_savefile::schema::fields::PlayerText;
    use pes_savefile::schema::{RecordSchema, schema_for};
    use pes_version::PesVersion;

    use super::SMALL_STATS;

    /// The `2^bit_width - 1` bound `info_strip.rs`'s `max` reads, over the
    /// appearance record too (a field may live there on PES 15/16); `None`
    /// on a version that does not store `field`.
    fn field_max(version: PesVersion, field: PlayerField) -> Option<u64> {
        fn in_record(
            record: &RecordSchema<PlayerField, PlayerText>,
            field: PlayerField,
        ) -> Option<u64> {
            record
                .fields
                .iter()
                .find(|spec| spec.field == field)
                .map(|spec| u64::from(spec.bit_width))
                .or_else(|| {
                    record.arrays.iter().find_map(|array| {
                        (0..array.count)
                            .any(|i| (array.make)(i) == field)
                            .then_some(u64::from(array.bit_width))
                    })
                })
        }
        let schema = schema_for(version);
        let width = in_record(schema.player, field).or_else(|| {
            schema
                .appearance
                .as_ref()
                .and_then(|(_, record)| in_record(record, field))
        })?;
        Some((1u64 << width) - 1)
    }

    // plans/save_editor.md "Editing UI", the Abilities & Skills bullet
    // ("each inside the field's bit width on every version"): every range's
    // top fits the field's bit width on every version storing it (after
    // the stored-one-lower shift). The Appearance tab's Cutscenes pair is
    // a `SmallStat` too.
    #[test]
    fn every_ranges_top_fits_the_fields_bit_width() {
        for (field, top, shift) in SMALL_STATS
            .iter()
            .chain(crate::view::appearance::fields::CUTSCENE_STATS.iter())
            .map(|small| (small.field, small.max, small.shift))
            .chain(ABILITY_FIELDS.iter().map(|&field| (field, 99u8, 0u8)))
        {
            for version in PesVersion::ALL {
                let Some(max) = field_max(version, field) else {
                    continue;
                };
                assert!(
                    u64::from(top - shift) <= max,
                    "{version:?} {field:?}: shown top {top} stores {} past {max}",
                    top - shift
                );
            }
        }
    }
}
