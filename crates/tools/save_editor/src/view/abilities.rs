//! The Abilities & Skills tab (`plans/save_editor.md` "Editing UI", the
//! Abilities & Skills bullet; the draft's `abilitiesTab`): registered
//! position, playing style, stronger foot and hand, the small stored
//! fields, the 13 playable positions with their C/B/A ratings, the 25
//! ability stats, and the skills and COM playing styles with the active
//! ruleset's Cards left badges. Every control edits a copy of the player's
//! entry and, when it changes, pushes it with `Session::edit_player` under
//! the control's `egui::Id` value, as `info_strip.rs` does; edit flags are
//! not touched. A field the save's version does not store is drawn in its
//! place disabled with a `PES NN+` tooltip naming the first version that
//! has it.

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
use crate::view::{ActiveRules, BadgeKey, SaveView, edit_player, selected_player};

/// The tab for the selected player, below `info_strip` and the tab strip.
pub(crate) fn show(ui: &mut egui::Ui, view: &mut SaveView) {
    let Some((team_id, slot, player_id)) = selected_player(view) else {
        return;
    };
    let rules = view.rules.as_ref();
    let Some(session) = view.session.as_mut() else {
        return;
    };
    let (version, positions, stats, mut skills) = {
        let file = session.file();
        let player = file.player(player_id).expect("the selected player");
        (
            file.version(),
            player.positions.clone(),
            player.stats.clone(),
            player.skills.clone(),
        )
    };
    // Every widget id under the tab is the player's: a DragValue's
    // in-progress edit, a combo's popup and the pickers' state never cross
    // players (and the undo merge keys differ per player, which is right).
    ui.push_id(player_id, |ui| {
        first_row(ui, session, player_id, version, &positions);
        small_stats(ui, session, player_id, &stats);
        ui.separator();
        ui.strong("Playable positions");
        playable_positions(ui, session, player_id, &positions);
        ui.separator();
        ui.strong("Ability stats");
        ability_stats(ui, session, player_id, &stats);
        ui.separator();
        ui.strong("Skills");
        cards_row(
            ui,
            rules,
            &mut view.cards,
            session,
            (team_id, slot, player_id),
            version,
            &mut skills,
        );
    });
}

/// The pairs of a row as the width allows, the draft's order kept: the
/// field/control pairs wrap below one another on the half-monitor window
/// rather than clip (a column count from the width `ui.available_width()`
/// gives; the estimate is a typical pair's width, so a slightly wider pair
/// still lands inside).
fn pairs_per_row(ui: &egui::Ui, pair_width: f32) -> usize {
    (ui.available_width() / pair_width).floor().max(1.0) as usize
}

/// `end_row` after `index` pairs (1-based) of `per_row` are done, so a row
/// takes as many pairs as `per_row` allows.
fn maybe_end_row(ui: &mut egui::Ui, index: usize, per_row: usize) {
    if index.is_multiple_of(per_row) {
        ui.end_row();
    }
}

/// Registered position, playing style, stronger foot and stronger hand
/// (the draft's first row); a missing stronger hand is the combo at
/// Right, disabled.
fn first_row(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    version: pes_version::PesVersion,
    positions: &PlayerPositions,
) {
    let per_row = pairs_per_row(ui, 250.0);
    egui::Grid::new("abilities first row").show(ui, |ui| {
        ui.label("Registered position");
        let mut registered = positions.registered;
        let combo = egui::ComboBox::from_id_salt("abilities registered position")
            .selected_text(
                POSITIONS
                    .get(usize::from(registered))
                    .copied()
                    .unwrap_or("?"),
            )
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
        maybe_end_row(ui, 1, per_row);
        ui.label("Playing style");
        let mut style = positions.playing_style;
        if cards::playing_style_picker(ui, "abilities playing style", version, &mut style) {
            edit_player(
                session,
                player_id,
                egui::Id::new(("abilities playing style", player_id)).value(),
                |player| {
                    player.positions.playing_style = style;
                },
            );
        }
        maybe_end_row(ui, 2, per_row);
        ui.label("Stronger foot");
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
        maybe_end_row(ui, 3, per_row);
        ui.label("Stronger hand");
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
        maybe_end_row(ui, 4, per_row);
    });
}

/// One small stored field of row 2: label, its schema field, the shown
/// range and the shift between shown and stored (the fields are stored
/// one below the game's numbers, the draft's `+ 1`/`v - 1`).
#[derive(Clone, Copy)]
struct SmallStat {
    /// The control's label.
    label: &'static str,
    /// The schema field gating it.
    field: PlayerField,
    /// The shown range.
    min: u8,
    /// The shown range's top.
    max: u8,
    /// `shown - shift` is the stored value.
    shift: u8,
    /// The stored value (`None` on a version that does not store it).
    get: fn(&PlayerStats) -> Option<u8>,
    /// The stored write.
    set: fn(&mut PlayerStats, u8),
}

/// Row 2's fields, in the draft's order: form 1-8, injury resistance 1-3,
/// weak foot usage and accuracy 1-4 (each stored one lower), star 0-7 and
/// playing attitude 0-3 as stored.
const SMALL_STATS: [SmallStat; 6] = [
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
    SmallStat {
        label: "Star",
        field: PlayerField::Star,
        min: 0,
        max: 7,
        shift: 0,
        get: |stats| stats.star,
        set: |stats, value| stats.star = Some(value),
    },
    SmallStat {
        label: "Playing attitude",
        field: PlayerField::PlayingAttitude,
        min: 0,
        max: 3,
        shift: 0,
        get: |stats| stats.playing_attitude,
        set: |stats, value| stats.playing_attitude = Some(value),
    },
];

/// The small stored fields (the draft's second row); a field the version
/// does not store shows `-`, disabled.
fn small_stats(ui: &mut egui::Ui, session: &mut Session, player_id: u32, stats: &PlayerStats) {
    let per_row = pairs_per_row(ui, 170.0);
    egui::Grid::new("abilities small stats").show(ui, |ui| {
        for (index, small) in SMALL_STATS.iter().enumerate() {
            match (small.get)(stats) {
                Some(stored) => number(
                    ui,
                    session,
                    player_id,
                    small.label,
                    stored + small.shift,
                    small.min..=small.max,
                    move |player, shown| {
                        (small.set)(&mut player.stats, shown - small.shift);
                    },
                ),
                // The label stays too: the gated control keeps its cell.
                None => {
                    ui.label(small.label);
                    gated_number(ui, small.field);
                }
            }
            maybe_end_row(ui, index + 1, per_row);
        }
    });
}

/// The 13 positions in `POSITIONS` order, four to a row, each its label
/// and the three `C` `B` `A` ratings (stored 0, 1, 2; a stored 3 selects
/// none). `playable[i]` is `POSITIONS[PLAYABLE_POSITIONS[i]]`'s rating:
/// the label's index is not the array's.
fn playable_positions(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    positions: &PlayerPositions,
) {
    let mut playable = positions.playable;
    egui::Grid::new("abilities playable")
        .num_columns(4)
        .show(ui, |ui| {
            for (row, group) in POSITIONS.chunks(4).enumerate() {
                for (inner, name) in group.iter().enumerate() {
                    let index = row * 4 + inner;
                    let at = PLAYABLE_POSITIONS
                        .iter()
                        .position(|&position| position == index)
                        .expect("PLAYABLE_POSITIONS is a permutation of 0..13");
                    ui.horizontal(|ui| {
                        ui.label(*name);
                        for (label, rating) in [("C", 0), ("B", 1), ("A", 2)] {
                            let response = ui.selectable_value(&mut playable[at], rating, label);
                            if response.changed() {
                                edit_player(session, player_id, response.id.value(), |player| {
                                    player.positions.playable = playable;
                                });
                            }
                        }
                    });
                }
                ui.end_row();
            }
        });
}

/// The 25 stats of `ABILITY_STATS`, five to a row, each its title-cased
/// label and a `DragValue` 40-99; a stat the version lacks shows `-`,
/// disabled.
fn ability_stats(ui: &mut egui::Ui, session: &mut Session, player_id: u32, stats: &PlayerStats) {
    let abilities = stats.abilities();
    let per_row = pairs_per_row(ui, 160.0);
    egui::Grid::new("abilities stats").show(ui, |ui| {
        for (index, name) in ABILITY_STATS.iter().enumerate() {
            match abilities[index] {
                Some(value) => number(
                    ui,
                    session,
                    player_id,
                    title_case(name),
                    value,
                    40..=99,
                    move |player, shown| player.stats.set_ability(index, shown),
                ),
                // The label stays too: the gated control keeps its cell.
                None => {
                    ui.label(title_case(name));
                    gated_number(ui, ABILITY_FIELDS[index]);
                }
            }
            maybe_end_row(ui, index + 1, per_row);
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
        ui.horizontal(|ui| {
            ui.weak("Cards left");
            ui.weak(format!("The AATF rules file does not load: {error}"));
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
            egui::Id::new(("abilities cards", player_id)).value(),
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
fn number(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    label: impl Into<egui::WidgetText>,
    shown: u8,
    range: RangeInclusive<u8>,
    store: impl FnOnce(&mut PlayerEntry, u8),
) {
    let mut shown = shown;
    ui.label(label);
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
}

/// The disabled stand-in for a number the save's version does not store:
/// `-` inside the control's usual bounds (a Button at the DragValue's
/// minimum size, so an enabled version's control takes the same rect,
/// SE-GUI-06) with the `PES NN+` tooltip.
fn gated_number(ui: &mut egui::Ui, field: PlayerField) {
    ui.add_enabled(
        false,
        egui::Button::new("-").min_size(ui.spacing().interact_size),
    )
    .on_disabled_hover_text(gating_tooltip(field));
}

/// The tooltip naming the first version that stores `field` (`PES NN+`).
fn gating_tooltip(field: PlayerField) -> String {
    format!(
        "PES {}+",
        first_version_with(field)
            .expect("a gated field is stored by some version")
            .number()
    )
}

/// `stored`'s foot/hand label (`FOOT` title-cased), its number when past
/// the table.
fn foot_label(stored: u8) -> String {
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
    // the stored-one-lower shift).
    #[test]
    fn every_ranges_top_fits_the_fields_bit_width() {
        for (field, top, shift) in SMALL_STATS
            .iter()
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
