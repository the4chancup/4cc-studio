//! The player view's three columns (`plans/save_editor.md` "Editing UI",
//! the player view bullet; the draft's `playerView`/`.pcols`): General
//! info (`info_strip::general`), Other stats and Playable positions in
//! the first, 350 wide; Skills with the Cards left badge in its header
//! row and COM playing styles in the second; the 25 ability stats in the
//! third — 36 points apart, tops aligned, wrapping to the next line where
//! the pane cannot hold the next column. Star and playing attitude, which
//! change only cutscenes, sit in the Appearance section's Cutscenes group
//! (`appearance::CUTSCENE_STATS`); the quick actions sit under the player
//! list (`quick_actions`).
//! Every control edits a copy of the player's entry and, when it changes,
//! pushes it with `Session::edit_player` under the control's `egui::Id`
//! value, as `info_strip.rs` does; edit flags are not touched. A field
//! the save's version does not store is drawn in its place disabled with
//! a `PES NN+` tooltip naming the first version that has it.

use std::ops::RangeInclusive;

use aatf::TeamView;
use pes_savefile::EditFile;
use pes_savefile::interchange::team_toml::labels::{
    self, ABILITY_STATS, FOOT, PLAYABLE_POSITIONS, POSITIONS, title_case,
};
use pes_savefile::model::{PlayerEntry, PlayerPositions, PlayerSkills, PlayerStats};
use pes_savefile::schema::fields::{ABILITY_FIELDS, PlayerField};
use pes_savefile::schema::first_version_with;
use team_widgets::cards;

use crate::session::Session;
use crate::view::{
    ActiveRules, BadgeKey, OwnBadge, Refusal, SaveView, edit_player, info_strip, selected_player,
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

/// The line a tier check's failure draws in the badge's place: it opens
/// the error in the refusal dialog (the quick actions' line's shape).
const TIER_CHECK_FAILED_LINE: &str = "Tier check failed, click for more information.";

/// The gap between two columns (the draft's `.pcols` `gap`).
const COLUMN_GAP: f32 = 36.0;
/// The gap between the panels of one column (the draft's `.pcol` `gap`),
/// and between Skills' two card columns (`cards` keeps the same 14).
const PANEL_GAP: f32 = 14.0;

/// The columns for the selected player, under `info_strip`'s title: the
/// three fixed columns side by side, each a `ui.vertical` of panels 14
/// points apart, the row wrapping whole columns on a narrow pane.
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
    // Every widget id under the content is the player's within this open: a
    // DragValue's in-progress edit, a combo's popup and the pickers' state
    // never cross players — nor saves, since egui keys a TextEdit's undo
    // history by widget id and the same player id of the save opened next
    // would otherwise inherit the last save's (and the undo merge keys
    // differ per player, which is right).
    let SaveView {
        session,
        rules,
        cards: badge,
        generation,
        refusal,
        ..
    } = view;
    let Some(session) = session.as_mut() else {
        return;
    };
    let generation = *generation;
    let mut skills_changed = false;
    ui.push_id(("player", generation, player_id), |ui| {
        // A wrapped row with tops aligned: `horizontal_wrapped` centers on
        // the cross axis, the columns want their tops level.
        let spacing = ui.spacing().item_spacing;
        ui.scope_builder(
            egui::UiBuilder::new()
                .layout(egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true)),
            |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(COLUMN_GAP, COLUMN_GAP);
                // The row's width, read before the columns draw: `max_rect`
                // is a soft bound, so a column wider than the pane grows it.
                let pane_width = ui.available_width();
                // Column 1: General info, Other stats, Playable positions —
                // `GENERAL_WIDTH` wide, its panels `PANEL_GAP` apart.
                ui.allocate_ui_with_layout(
                    egui::vec2(info_strip::GENERAL_WIDTH, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(spacing.x, PANEL_GAP);
                        panel(ui, spacing, "General info", |ui| {
                            info_strip::general(ui, session, team_id, slot, player_id);
                        });
                        panel(ui, spacing, "Other stats", |ui| {
                            info_strip::other_stats(ui, session, player_id, &stats);
                        });
                        panel(ui, spacing, PLAYABLE_POSITIONS_LABEL, |ui| {
                            playable_positions(ui, session, player_id, &positions);
                        });
                    },
                );
                // Column 2: Skills (the Cards left badge in its header
                // row) over COM playing styles, their columns 158 wide:
                // the two-column width, or the pane's when that cannot
                // hold them (the card columns narrow to one).
                ui.allocate_ui_with_layout(
                    egui::vec2(
                        cards::TWO_COLUMN_WIDTH.min(pane_width),
                        ui.available_height(),
                    ),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(spacing.x, PANEL_GAP);
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = spacing;
                            skills_changed |= skills_panel(
                                ui,
                                rules.as_ref(),
                                badge,
                                session,
                                (team_id, slot),
                                &mut skills,
                                refusal,
                            );
                        });
                        panel(ui, spacing, "COM playing styles", |ui| {
                            skills_changed |= cards::card_columns(
                                ui,
                                version,
                                &mut skills.com_styles,
                                &labels::COM_STYLES,
                                PlayerField::ComStyle,
                            );
                        });
                    },
                );
                // Column 3: the ability stats, one pair a row.
                ui.allocate_ui_with_layout(
                    egui::vec2(170.0, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(spacing.x, PANEL_GAP);
                        panel(ui, spacing, "Ability stats", |ui| {
                            ability_stats(ui, session, player_id, &stats);
                        });
                    },
                );
            },
        );
    });
    if skills_changed {
        edit_player(
            session,
            player_id,
            egui::Id::unique(("abilities cards", player_id)).value(),
            move |player| {
                player.skills = skills;
            },
        );
    }
}

/// One panel of a column: the `ui.strong` title over `add`, one item of
/// the column's vertical — the column's `PANEL_GAP` sits between panels,
/// `spacing` (the pane's own) restored inside.
fn panel(ui: &mut egui::Ui, spacing: egui::Vec2, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = spacing;
        ui.strong(title);
        add(ui);
    });
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

/// One small stored field of the Other stats panel or the Appearance
/// section's Cutscenes group: the term's name, the drawn label, its schema
/// field, the shown range and the shift between shown and stored (the
/// shifted fields are stored one below the game's numbers, the draft's
/// `+ 1`/`v - 1`).
#[derive(Clone, Copy)]
pub(super) struct SmallStat {
    /// The diff row's name for the field (`terms::row_term`); the control
    /// draws `shown`.
    pub(super) label: &'static str,
    /// The control's drawn label (the weak foot's two share one's:
    /// "Weak foot: Usage", then "Accuracy").
    pub(super) shown: &'static str,
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

/// The Other stats panel's four stored fields, in the draft's order: form
/// 1-8, injury resistance 1-3, weak foot usage and accuracy 1-4 (each
/// stored one lower). Star and playing attitude, which change only
/// cutscenes, sit in the Appearance section's `CUTSCENE_STATS`.
pub(super) const SMALL_STATS: [SmallStat; 4] = [
    SmallStat {
        label: "Form",
        shown: "Form",
        field: PlayerField::Form,
        min: 1,
        max: 8,
        shift: 1,
        get: |stats| Some(stats.form),
        set: |stats, value| stats.form = value,
    },
    SmallStat {
        label: "Injury resistance",
        shown: "Injury resistance",
        field: PlayerField::InjuryResistance,
        min: 1,
        max: 3,
        shift: 1,
        get: |stats| Some(stats.injury_resistance),
        set: |stats, value| stats.injury_resistance = value,
    },
    SmallStat {
        label: "Weak foot usage",
        shown: "Weak foot: Usage",
        field: PlayerField::WeakFootUsage,
        min: 1,
        max: 4,
        shift: 1,
        get: |stats| Some(stats.weak_foot_usage),
        set: |stats, value| stats.weak_foot_usage = value,
    },
    SmallStat {
        label: "Weak foot accuracy",
        shown: "Accuracy",
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
/// keeps its cell). The Appearance section's Cutscenes group uses it; the
/// Other stats panel's `small` pairs draw the halves apart.
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
                small.shown,
                stored + small.shift,
                small.min..=small.max,
                move |player, shown| {
                    (small.set)(&mut player.stats, shown - small.shift);
                },
            );
        }
        None => {
            ui.label(small.shown);
            gated_number(ui, small.field);
        }
    }
}

/// The positions grid's rows (the draft's `POS_ROWS`): the order a member
/// reads a formation in, left wing to right wing, GK alone below.
const POS_ROWS: [&[&str]; 4] = [
    &["LWF", "CF", "SS", "RWF"],
    &["LMF", "AMF", "CMF", "DMF", "RMF"],
    &["LB", "CB", "RB"],
    &["GK"],
];

/// A positions cell's width (the draft's `.poscell` 62px).
const POS_CELL_WIDTH: f32 = 62.0;
/// A rating square's side (the draft's `.rate .sel` 17px).
const RATING_SIDE: f32 = 17.0;
/// The gap between a cell's rating squares (the draft's `.rate` 2px).
const RATING_GAP: f32 = 2.0;
/// A cell's three rating squares' width.
const RATINGS_WIDTH: f32 = 3.0 * RATING_SIDE + 2.0 * RATING_GAP;

/// The chosen rating's fill and text colours (the draft's `.sel.on`
/// variants): A the success green with dark text, B the selection colour
/// with its foreground, C a grey square with the normal text — named
/// outright, since a selected button's own text colour is the
/// selection's foreground.
fn rating_colors(ui: &egui::Ui, rating: u8) -> (egui::Color32, egui::Color32) {
    match rating {
        2 => (
            studio_core::theme::SUCCESS_FG,
            egui::Color32::from_gray(0x1a),
        ),
        1 => (
            ui.visuals().selection.bg_fill,
            ui.visuals().selection.stroke.color,
        ),
        _ => (
            egui::Color32::from_rgb(0x5a, 0x5a, 0x5a),
            ui.visuals().text_color(),
        ),
    }
}

/// The 13 positions in `POS_ROWS`, each row centred in the column (the
/// draft's `#pos-grid .posrow` `justify-content: center`), each cell the
/// position's label above the three `C` `B` `A` ratings (stored 0, 1, 2;
/// a stored 3 selects none), the cell's contents centred so the label
/// sits over the B, the registered position's cell outlined in the
/// selection colour (the draft's `.poscell.reg`). `playable[i]` is
/// `POSITIONS[PLAYABLE_POSITIONS[i]]`'s rating: the label's index is not
/// the array's.
fn playable_positions(
    ui: &mut egui::Ui,
    session: &mut Session,
    player_id: u32,
    positions: &PlayerPositions,
) {
    let mut playable = positions.playable;
    let cell_gap = ui.spacing().item_spacing.x;
    let body = egui::TextStyle::Body.resolve(ui.style());
    let column_width = info_strip::GENERAL_WIDTH.min(ui.available_width());
    for row in POS_ROWS {
        // `add_space` advances the cursor outright, no item spacing.
        let row_width = row.len() as f32 * POS_CELL_WIDTH + (row.len() - 1) as f32 * cell_gap;
        ui.horizontal(|ui| {
            ui.add_space(((column_width - row_width) / 2.0).max(0.0));
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
                    // The draft's `.poscell` 62px and `.rate`'s squares
                    // 2 apart: `Button::selectable` is what
                    // `selectable_value` draws.
                    ui.set_width(POS_CELL_WIDTH);
                    // The cell's contents centred (the draft's `align-items:
                    // center`): the label padded to the cell's centre, so
                    // over the B; the squares 3.5 in.
                    let galley = ui.painter().layout_no_wrap(
                        (*name).to_owned(),
                        body.clone(),
                        egui::Color32::WHITE,
                    );
                    ui.horizontal(|ui| {
                        ui.add_space(((POS_CELL_WIDTH - galley.size().x) / 2.0).max(0.0));
                        ui.label(*name);
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = RATING_GAP;
                        ui.spacing_mut().button_padding = egui::vec2(0.0, 0.0);
                        ui.add_space((POS_CELL_WIDTH - RATINGS_WIDTH) / 2.0);
                        for (label, rating) in RATING_LABELS {
                            let response = if playable[at] == rating {
                                // The chosen rating filled (the draft's
                                // `.sel.on.*`).
                                let (fill, text) = rating_colors(ui, rating);
                                ui.add(
                                    egui::Button::selectable(
                                        true,
                                        egui::RichText::new(label).color(text),
                                    )
                                    .min_size(egui::vec2(RATING_SIDE, RATING_SIDE))
                                    .fill(fill),
                                )
                            } else {
                                ui.add(
                                    egui::Button::selectable(false, label)
                                        .min_size(egui::vec2(RATING_SIDE, RATING_SIDE)),
                                )
                            };
                            if response.clicked() && playable[at] != rating {
                                playable[at] = rating;
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
        });
    }
}

/// The 25 stats of `ABILITY_STATS` in one column in the game's order (the
/// view's third column), each label beside its `DragValue` 40-99; a stat
/// the version lacks shows `-`, disabled.
fn ability_stats(ui: &mut egui::Ui, session: &mut Session, player_id: u32, stats: &PlayerStats) {
    let abilities = stats.abilities();
    egui::Grid::new("abilities stats")
        .num_columns(2)
        .show(ui, |ui| {
            for (index, &label) in ABILITY_STATS.iter().enumerate() {
                ui.label(title_case(label));
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
                ui.end_row();
            }
        });
}

/// The Skills panel's content: the header row's Cards left badge — the
/// player's own tier's, the tier `Rules::player_tier` reads him as, then
/// that tier's `card_limits`; no tier matching draws the stand-in, a
/// file that does not load draws nothing (the quick actions report it),
/// and a check that fails on him draws the red click line that opens the
/// error in `refusal`, then the checkboxes in their two columns.
/// Returns whether a card was ticked this frame. The computed badge
/// lives in `cards` under the `BadgeKey`
/// it was computed for — a `Session::revision` bump, a selection change
/// or another rules file recomputes it, and any other frame draws the
/// kept one.
fn skills_panel(
    ui: &mut egui::Ui,
    rules: Option<&ActiveRules>,
    badge: &mut Option<(BadgeKey, OwnBadge)>,
    session: &mut Session,
    (team_id, slot): (u32, u8),
    skills: &mut PlayerSkills,
    refusal: &mut Option<Refusal>,
) -> bool {
    let mut load_error = false;
    if let Some(active) = rules {
        match &active.rules {
            Ok(rules) => {
                let key = BadgeKey {
                    revision: session.revision(),
                    team: team_id,
                    slot,
                    rules_file: active.file.clone(),
                };
                if badge.as_ref().is_none_or(|(held, _)| *held != key) {
                    *badge = Some((key, own_badge(rules, session.file(), team_id, slot)));
                }
            }
            // The load error draws nothing here (the quick actions under
            // the list say so, and its text pushed the header's row
            // apart): a stale badge goes too.
            Err(_) => {
                load_error = true;
                *badge = None;
            }
        }
    }
    // A tier check that fails on him draws `Cards left` then a red click
    // line in the badge's place, opening its error in the refusal dialog
    // (the AATF check never runs the two functions the badge asks, so a
    // member's edit that breaks only them would otherwise show nowhere).
    let tier_error = match badge.as_ref() {
        Some((_, OwnBadge::Failed(error))) => Some(error.clone()),
        _ => None,
    };
    ui.horizontal_wrapped(|ui| {
        ui.strong("Skills");
        if let Some(error) = &tier_error {
            ui.weak("Cards left");
            let response = ui
                .add(
                    egui::Label::new(
                        egui::RichText::new(TIER_CHECK_FAILED_LINE)
                            .color(ui.visuals().error_fg_color),
                    )
                    .sense(egui::Sense::click()),
                )
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if response.clicked() {
                *refusal = Some(Refusal {
                    title: "The tier check failed",
                    message: error.clone(),
                });
            }
        } else if !load_error {
            match badge.as_ref() {
                Some((_, OwnBadge::Tier { label, limits })) => cards::cards_left(
                    ui,
                    &cards::CardsBadge::Tier(cards::TierCards { label, limits }),
                ),
                Some((_, OwnBadge::NoTier)) => cards::cards_left(ui, &cards::CardsBadge::NoTier),
                Some((_, OwnBadge::Failed(_))) | None => {
                    ui.weak("Cards left");
                }
            }
        }
    });
    cards::card_columns(
        ui,
        session.file().version(),
        &mut skills.skills,
        &labels::SKILLS,
        PlayerField::Skill,
    )
}

/// What `rules` read team `team_id`'s roster `slot` as on `file`: his own
/// tier with its card economy, no tier, or a check that failed with its
/// error (a team the view cannot build, or an empty slot, fails too).
fn own_badge(rules: &aatf::Rules, file: &EditFile, team_id: u32, slot: u8) -> OwnBadge {
    let team = match TeamView::new(file, team_id) {
        Ok(team) => team,
        Err(error) => return OwnBadge::Failed(error.to_string()),
    };
    let Some(player) = team.player(slot) else {
        return OwnBadge::Failed(format!("roster slot {slot} holds no player"));
    };
    match rules.player_tier(&team, player) {
        Ok(Some(tier)) => match rules.card_limits(&team, player, &tier.id) {
            Ok(limits) => OwnBadge::Tier {
                label: tier.label.clone(),
                limits,
            },
            Err(error) => OwnBadge::Failed(error.to_string()),
        },
        Ok(None) => OwnBadge::NoTier,
        Err(error) => OwnBadge::Failed(error.to_string()),
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
    // the stored-one-lower shift). The Appearance section's Cutscenes
    // pair is a `SmallStat` too.
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
