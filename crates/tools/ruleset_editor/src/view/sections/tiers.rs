//! The Tiers section (the draft's `VIEWS.tiers`): the tiers table, one
//! row per tier, highest first and reordered by dragging the handle,
//! each row's expander holding the per-stat targets, the free cards and
//! the conditions, and the medal systems under the table.

use std::collections::{BTreeMap, BTreeSet};

use aatf::{MedalSystem, Ruleset, Stat, Tier, TierId};
use pes_savefile::interchange::team_toml::labels::ABILITY_STATS;

use super::Section;
use super::widgets;
use crate::summary::human;

/// The table's columns (the draft's `th` row): the handle, label, id,
/// count, counts as, rate, form, injury resistance, weak foot, paid
/// cards, minimum cards, A positions, name colour, warning band, stats
/// within targets and the ✕.
const COLUMNS: usize = 16;

/// The Tiers form.
pub(crate) fn show(ui: &mut egui::Ui, ruleset: &mut Ruleset) {
    ui.heading(Section::Tiers.label());
    ui.weak(
        "Highest first: a player gets the first tier whose conditions he meets and whose \
         rating he reaches. The Save editor's tier buttons follow this order.",
    );
    let medal_tiers = medal_tier_ids(ruleset);
    tiers_table(ui, ruleset, &medal_tiers);
    ui.separator();
    medal_systems(ui, ruleset, &medal_tiers);
}

/// The tier ids the medal systems count, as a set (the draft's
/// `medalTiers`): a tier one names takes its count from the systems.
fn medal_tier_ids(ruleset: &Ruleset) -> BTreeSet<TierId> {
    ruleset
        .medal_systems
        .iter()
        .flat_map(|system| system.counts.keys().cloned())
        .collect()
}

/// The tiers table (an `egui::Grid`): the header row, one row per
/// tier plus its expander row, and "+ Add tier". A drop or a removal
/// lands after the loop so the rows keep their indices while drawn.
fn tiers_table(ui: &mut egui::Ui, ruleset: &mut Ruleset, medal_tiers: &BTreeSet<TierId>) {
    let mut drop = None;
    let mut remove = None;
    let ids: Vec<(TierId, String)> = ruleset
        .tiers
        .iter()
        .map(|tier| (tier.id.clone(), tier.label.clone()))
        .collect();
    egui::Grid::new("tiers-table")
        .num_columns(COLUMNS)
        .show(ui, |ui| {
            for head in [
                "",
                "Label",
                "Id",
                "Count",
                "Counts as",
                "Rate",
                "Form",
                "Injury res.",
                "Weak foot",
                "Paid cards",
                "Min. cards",
                "A positions",
                "Name colour",
                "Warn band",
                "Stats ≤ targets",
                "",
            ] {
                ui.strong(head);
            }
            ui.end_row();
            for index in 0..ruleset.tiers.len() {
                tier_row(
                    ui,
                    ruleset,
                    index,
                    medal_tiers,
                    &ids,
                    &mut drop,
                    &mut remove,
                );
                expander_row(ui, ruleset, index);
            }
        });
    // The draft's `reorderable` drop: the dragged tier takes the drop
    // row's place.
    if let Some((from, to)) = drop
        && from != to
    {
        let moved = ruleset.tiers.remove(from);
        ruleset.tiers.insert(to, moved);
    }
    if let Some(index) = remove {
        drop_tier(ruleset, index);
    }
    if ui.button("+ Add tier").clicked() {
        ruleset.tiers.push(new_tier(ruleset.tiers.len() + 1));
    }
}

/// One tier's row: the drag handle, the fields of the draft's columns
/// and the ✕ (while a second tier remains), every cell a drop target
/// for the reorder (the draft's `reorderable`).
fn tier_row(
    ui: &mut egui::Ui,
    ruleset: &mut Ruleset,
    index: usize,
    medal_tiers: &BTreeSet<TierId>,
    ids: &[(TierId, String)],
    drop: &mut Option<(usize, usize)>,
    remove: &mut Option<usize>,
) {
    // The handle is the drag source (the draft's `handle`).
    let response = ui
        .dnd_drag_source(ui.make_persistent_id(("tier-handle", index)), index, |ui| {
            ui.label("⠇").on_hover_text("Drag to reorder")
        })
        .response;
    drop_target(&response, index, drop);

    let response = widgets::text(ui, &mut ruleset.tiers[index].label, 100.0);
    drop_target(&response, index, drop);

    // An id edit renames the tier's key in every medal system's counts.
    let mut id = ruleset.tiers[index].id.clone();
    let response = ui
        .add(
            egui::TextEdit::singleline(&mut id)
                .desired_width(80.0)
                .font(egui::TextStyle::Monospace),
        )
        .on_hover_text("What conditions and other tiers refer to");
    drop_target(&response, index, drop);
    if response.changed() && id != ruleset.tiers[index].id {
        rename_tier_id(ruleset, index, id);
    }

    let medal_counted = medal_tiers.contains(&ruleset.tiers[index].id);
    let response = ui
        .horizontal(|ui| count_cell(ui, ruleset, index, medal_counted))
        .response;
    drop_target(&response, index, drop);

    let response = counts_as_cell(ui, ruleset, index, ids);
    drop_target(&response, index, drop);

    let response = widgets::number(ui, &mut ruleset.tiers[index].rate, 0..=99);
    drop_target(&response, index, drop);
    let response = widgets::number(ui, &mut ruleset.tiers[index].form, 0..=8);
    drop_target(&response, index, drop);
    let response = widgets::number(ui, &mut ruleset.tiers[index].injury_resistance, 0..=3);
    drop_target(&response, index, drop);
    let response = ui
        .horizontal(|ui| {
            widgets::number(ui, &mut ruleset.tiers[index].weak_foot.usage, 0..=4)
                .on_hover_text("Usage");
            widgets::number(ui, &mut ruleset.tiers[index].weak_foot.accuracy, 0..=4)
                .on_hover_text("Accuracy");
        })
        .response;
    drop_target(&response, index, drop);
    let response = widgets::number(ui, &mut ruleset.tiers[index].cards, 0..=20);
    drop_target(&response, index, drop);
    let response = widgets::number(ui, &mut ruleset.tiers[index].min_cards, 0..=20);
    drop_target(&response, index, drop);
    let response = widgets::number(ui, &mut ruleset.tiers[index].a_positions, 0..=13);
    drop_target(&response, index, drop);

    let response = ui
        .horizontal(|ui| name_color_cell(ui, ruleset, index))
        .response;
    drop_target(&response, index, drop);

    let response = widgets::number(ui, &mut ruleset.tiers[index].stat_warning, 0..=99)
        .on_hover_text(
            "Points above a stat's target that warn rather than err (0: every point errs)",
        );
    drop_target(&response, index, drop);

    let response = ui
        .checkbox(&mut ruleset.tiers[index].total_within_targets, "")
        .on_hover_text("The stats add up to at most the targets, an error past them");
    drop_target(&response, index, drop);

    if ruleset.tiers.len() > 1 {
        let response = ui
            .add(egui::Button::new("✕").frame_when_inactive(false))
            .on_hover_text("Remove");
        drop_target(&response, index, drop);
        if response.clicked() {
            *remove = Some(index);
        }
    }
    ui.end_row();
}

/// The row's count cell (the draft's): "by medal system" while a
/// system counts the tier, "—" under `counts_as`, else the number or
/// "the rest" with the rest checkbox.
fn count_cell(ui: &mut egui::Ui, ruleset: &mut Ruleset, index: usize, medal_counted: bool) {
    let tier = &mut ruleset.tiers[index];
    if medal_counted {
        ui.weak("by medal system");
        return;
    }
    if tier.counts_as.is_some() {
        widgets::dash(ui);
        return;
    }
    match &mut tier.count {
        Some(count) => {
            widgets::number(ui, count, 0..=32);
        }
        None => {
            ui.weak("the rest");
        }
    }
    let mut rest = tier.count.is_none();
    if ui
        .checkbox(&mut rest, "rest")
        .on_hover_text("The rest of the squad: one tier at most")
        .changed()
    {
        tier.count = if rest { None } else { Some(1) };
    }
}

/// The row's `counts_as` combo (the draft's): "—" for none, else one
/// of the other tiers by its label; setting one clears the tier's
/// `count`.
fn counts_as_cell(
    ui: &mut egui::Ui,
    ruleset: &mut Ruleset,
    index: usize,
    ids: &[(TierId, String)],
) -> egui::Response {
    let own_id = ruleset.tiers[index].id.clone();
    let mut choice = ruleset.tiers[index].counts_as.clone();
    let selected = choice
        .as_ref()
        .map_or_else(|| "—".to_string(), |id| label_of(ids, id));
    let response = egui::ComboBox::from_id_salt(("counts_as", index))
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut choice, None, "—");
            for (id, label) in ids.iter().filter(|(id, _)| id != &own_id) {
                ui.selectable_value(&mut choice, Some(id.clone()), label);
            }
        })
        .response;
    let tier = &mut ruleset.tiers[index];
    if choice != tier.counts_as {
        tier.counts_as = choice;
        if tier.counts_as.is_some() {
            tier.count = None;
        }
    }
    response
}

/// The label `ids` gives `id` (the draft's combo shows labels), `id`
/// itself when no tier carries it.
fn label_of(ids: &[(TierId, String)], id: &TierId) -> String {
    ids.iter()
        .find(|(other, _)| other == id)
        .map(|(_, label)| label.clone())
        .unwrap_or_else(|| id.clone())
}

/// The row's name-colour cell (the draft's `colour`): the checkbox,
/// then the suite's colour field and the eight-byte hex field, the
/// alpha kept as the code's last two digits (`ff` for a new one);
/// unticked, weak "none".
fn name_color_cell(ui: &mut egui::Ui, ruleset: &mut Ruleset, index: usize) {
    let tier = &mut ruleset.tiers[index];
    let mut on = tier.name_color.is_some();
    if ui
        .checkbox(&mut on, "")
        .on_hover_text("This tier's names carry a colour code")
        .changed()
    {
        tier.name_color = on.then(|| "cc9900ff".to_string());
    }
    let Some(code) = tier.name_color.clone() else {
        ui.weak("none");
        return;
    };
    let mut rgb = [
        hex_channel(&code, 0..2),
        hex_channel(&code, 2..4),
        hex_channel(&code, 4..6),
    ];
    let output = color_tools::picker::color_field(ui, ("tier name colour", index), &mut rgb, &[]);
    if output.changed {
        tier.name_color = Some(code_with_alpha(rgb, tier.name_color.as_deref()));
    }
    if let Some(code) = &mut tier.name_color {
        ui.add(
            egui::TextEdit::singleline(code)
                .desired_width(76.0)
                .font(egui::TextStyle::Monospace),
        )
        .on_hover_text("The eight bytes after \\x11c in the name");
    }
}

/// `code`'s two hex digits at `range` as a channel (0 where a code
/// typed mid-edit is short or not hex).
fn hex_channel(code: &str, range: std::ops::Range<usize>) -> u8 {
    code.get(range)
        .and_then(|digits| u8::from_str_radix(digits, 16).ok())
        .unwrap_or(0)
}

/// `rgb` as an eight-byte code keeping `current`'s alpha (its last two
/// digits, `ff` for a new one — the draft's `colour`).
fn code_with_alpha(rgb: [u8; 3], current: Option<&str>) -> String {
    let alpha = current.and_then(|code| code.get(6..8)).unwrap_or("ff");
    format!("{:02x}{:02x}{:02x}{alpha}", rgb[0], rgb[1], rgb[2])
}

/// `index`'s tier renamed to `id`, its key carried into every medal
/// system's counts (the draft's id edit, skipped where the new id is
/// already a key so it never overwrites one).
fn rename_tier_id(ruleset: &mut Ruleset, index: usize, id: TierId) {
    let old = std::mem::replace(&mut ruleset.tiers[index].id, id.clone());
    for system in &mut ruleset.medal_systems {
        if system.counts.contains_key(&old) && !system.counts.contains_key(&id) {
            let count = system.counts.remove(&old).expect("the key is present");
            system.counts.insert(id.clone(), count);
        }
    }
}

/// `index`'s tier removed, its id dropped from every medal system's
/// counts (the draft's row ✕).
fn drop_tier(ruleset: &mut Ruleset, index: usize) {
    let id = ruleset.tiers.remove(index).id;
    for system in &mut ruleset.medal_systems {
        system.counts.remove(&id);
    }
}

/// `index`'s cell took a drop this frame: remember (from, to).
fn drop_target(response: &egui::Response, index: usize, drop: &mut Option<(usize, usize)>) {
    if let Some(from) = response.dnd_release_payload::<usize>() {
        *drop = Some((*from, index));
    }
}

/// The tier "+ Add tier" adds (the draft's `tier()`): `tier<N>`,
/// "Tier <N>", count 1, rate 80.
fn new_tier(n: usize) -> Tier {
    crate::document::tier(&format!("tier{n}"), &format!("Tier {n}"), Some(1), 80)
}

/// The row's expander (the draft's `colspan` row under the data row):
/// `egui::Grid` has no colspan, so the expander is emitted in the
/// row's last column — the only column whose width nothing after it
/// reads — inside a child ui whose rect starts back at the table's
/// left edge.
fn expander_row(ui: &mut egui::Ui, ruleset: &mut Ruleset, index: usize) {
    for _ in 0..COLUMNS - 1 {
        ui.label("");
    }
    let rect = egui::Rect::from_min_max(
        egui::pos2(
            ui.max_rect().min.x + ui.spacing().interact_size.x,
            ui.cursor().min.y,
        ),
        ui.max_rect().right_bottom(),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        egui::CollapsingHeader::new("Stat targets, free cards and conditions")
            .id_salt(("tier-expander", index))
            .show(ui, |ui| expander(ui, ruleset, index));
    });
    ui.end_row();
}

/// The expander's body (the draft's `collapsing` content): the
/// per-stat targets, the free cards and the conditions' heading.
fn expander(ui: &mut egui::Ui, ruleset: &mut Ruleset, index: usize) {
    ui.horizontal(|ui| {
        ui.strong("Per-stat targets");
        ui.weak(" (unticked: the rate)");
    });
    stat_targets(ui, ruleset, index);
    ui.separator();
    ui.strong("Free cards");
    free_cards(ui, ruleset, index);
    ui.separator();
    ui.horizontal(|ui| {
        ui.strong("Conditions");
        ui.weak(" (what a player must also meet to be this tier)");
    });
    ui.weak("The condition editor comes in a later build.");
}

/// The per-stat targets, five columns as the draft's `perStatGrid`:
/// each ability stat's "own target" checkbox and its number while it
/// has one (unticked, the stat follows the rate and has no `stats`
/// entry — `DECISIONS.md` 2026-10-24).
fn stat_targets(ui: &mut egui::Ui, ruleset: &mut Ruleset, index: usize) {
    egui::Grid::new(("stat-targets", index))
        .num_columns(5)
        .show(ui, |ui| {
            for (cell, name) in ABILITY_STATS.iter().enumerate() {
                let stat = Stat::try_from((*name).to_string()).expect("an ability stat");
                ui.horizontal(|ui| {
                    let tier = &mut ruleset.tiers[index];
                    let mut own = tier.stats.contains_key(&stat);
                    if ui.checkbox(&mut own, human(name)).changed() {
                        if own {
                            tier.stats.insert(stat, tier.rate);
                        } else {
                            tier.stats.remove(&stat);
                        }
                    }
                    if let Some(target) = tier.stats.get_mut(&stat) {
                        widgets::number(ui, target, 0..=99);
                    }
                });
                if cell % 5 == 4 {
                    ui.end_row();
                }
            }
        });
}

/// The free-card counts, "COM styles" first then each card group by
/// its words (the draft's `row` of group fields): a count of 0 removes
/// the group's `free` entry.
fn free_cards(ui: &mut egui::Ui, ruleset: &mut Ruleset, index: usize) {
    let groups: Vec<String> = std::iter::once("com".to_string())
        .chain(ruleset.card_groups.keys().cloned())
        .collect();
    ui.horizontal_wrapped(|ui| {
        for group in &groups {
            ui.horizontal(|ui| {
                if group == "com" {
                    ui.label("COM styles");
                } else {
                    ui.label(human(group));
                }
                map_number(ui, &mut ruleset.tiers[index].free, group, 20, false);
            });
        }
    });
}

/// The medal systems under the table (the draft's): the header row
/// holds one checkbox per tier, ticking puts the tier in every system
/// at 0 and clears its count, clearing removes it from every system;
/// one row per system (its label field, ✕ and a count per counted
/// tier); "+ Add medal system".
fn medal_systems(ui: &mut egui::Ui, ruleset: &mut Ruleset, medal_tiers: &BTreeSet<TierId>) {
    ui.strong("Medal systems");
    ui.weak(
        "Alternative medal counts: the team's counts of the ticked tiers must equal one \
         system's. Unticked tiers keep their own count, one of them the rest of the squad.",
    );
    let tiers: Vec<(TierId, String)> = ruleset
        .tiers
        .iter()
        .map(|tier| (tier.id.clone(), tier.label.clone()))
        .collect();
    let has_systems = !ruleset.medal_systems.is_empty();
    let mut remove = None;
    egui::Grid::new("medal-systems")
        .num_columns(1 + tiers.len())
        .show(ui, |ui| {
            ui.label("");
            for (index, (id, label)) in tiers.iter().enumerate() {
                let mut counted = medal_tiers.contains(id);
                let tip = if has_systems {
                    "The medal systems count this tier"
                } else {
                    "Add a medal system first"
                };
                let cell = ui.horizontal(|ui| {
                    ui.add_enabled(has_systems, egui::Checkbox::new(&mut counted, label))
                        .changed()
                });
                cell.response.on_hover_text(tip);
                let toggled = cell.inner;
                if toggled {
                    set_counted(ruleset, index, counted);
                }
            }
            ui.end_row();
            for row in 0..ruleset.medal_systems.len() {
                ui.horizontal(|ui| {
                    widgets::text(ui, &mut ruleset.medal_systems[row].label, 90.0);
                    if ui
                        .add(egui::Button::new("✕").frame_when_inactive(false))
                        .on_hover_text("Remove")
                        .clicked()
                    {
                        remove = Some(row);
                    }
                });
                for (id, _) in &tiers {
                    if medal_tiers.contains(id) {
                        map_number(ui, &mut ruleset.medal_systems[row].counts, id, 32, true);
                    } else {
                        widgets::dash(ui);
                    }
                }
                ui.end_row();
            }
        });
    if let Some(row) = remove {
        ruleset.medal_systems.remove(row);
    }
    if ui.button("+ Add medal system").clicked() {
        ruleset.medal_systems.push(MedalSystem {
            label: format!("System {}", ruleset.medal_systems.len() + 1),
            counts: BTreeMap::new(),
        });
    }
}

/// `index`'s tier put into every medal system at 0 (`counted`, its
/// `count` cleared) or taken out of them all (the draft's header
/// checkbox).
fn set_counted(ruleset: &mut Ruleset, index: usize, counted: bool) {
    if counted {
        for system in &mut ruleset.medal_systems {
            system
                .counts
                .entry(ruleset.tiers[index].id.clone())
                .or_insert(0);
        }
        ruleset.tiers[index].count = None;
    } else {
        for system in &mut ruleset.medal_systems {
            system.counts.remove(&ruleset.tiers[index].id);
        }
    }
}

/// `map[key]` as a `DragValue` clamped to 0..=`max`, absent read as 0;
/// a written value is kept, 0 keeping its entry (`keep_zero`) or
/// removing it (a free-cards count of 0 is no entry).
fn map_number(
    ui: &mut egui::Ui,
    map: &mut BTreeMap<String, u8>,
    key: &str,
    max: u8,
    keep_zero: bool,
) -> egui::Response {
    let mut value = map.get(key).copied().unwrap_or(0);
    let response = ui.add(egui::DragValue::new(&mut value).range(0..=max));
    if response.changed() {
        if value == 0 && !keep_zero {
            map.remove(key);
        } else {
            map.insert(key.to_string(), value);
        }
    }
    response
}
