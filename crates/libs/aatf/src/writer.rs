//! `Ruleset` written as its `RULESET` data block: the Ruleset editor's
//! save and preview text. The block reads back as an equal `Ruleset`,
//! and writing twice gives identical bytes.

use std::fmt;

use crate::ruleset::{
    Bracket, BracketHeights, Condition, Effects, ExtraHeight, HeightSystem, Heights, MedalSystem,
    PerTier, PlayerValue, Ruleset, Severity, Special, StatOrAll, StatRules, TeamSettings, Tier,
    Universal, WeakFoot, WeightRule,
};

/// The first line of every rules file, as `rulesets/` carries it.
const HEADER: &str = "// AATF ruleset. Open it in the Ruleset editor to change it.";

/// The indent unit, four spaces.
const INDENT: &str = "    ";
/// The list items' and map entries' indent.
const I2: &str = "        ";
/// The height lists' indent, one deeper.
const I3: &str = "            ";
/// The wrap width, in columns.
const WIDTH: usize = 100;

/// `key: value`.
fn field(key: impl fmt::Display, value: impl fmt::Display) -> String {
    format!("{key}: {value}")
}

/// `text` as a Rhai string literal: `"` and `\` escaped, every
/// character below U+0020 escaped (`\n`, `\t`, `\r`, else `\xHH`, the
/// escapes rhai's tokenizer reads), everything else written as it is.
fn string_literal(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for ch in text.chars() {
        match ch {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\t' => literal.push_str("\\t"),
            '\r' => literal.push_str("\\r"),
            ch if ch < ' ' => literal.push_str(&format!("\\x{:02x}", u32::from(ch))),
            ch => literal.push(ch),
        }
    }
    literal.push('"');
    literal
}

/// `key` as a map-literal key: bare when Rhai's parser takes it as an
/// identifier, a string literal else (an id is any non-empty string).
/// Bare needs both (a) the identifier shape with at least one letter
/// (the parser alone would take a key like `a: 0, b`) and (b) the
/// parser accepting `#{ <key>: 0 }`, which refuses keywords and
/// reserved words (`in`, `go`, `match`): rhai's identifier check and
/// keyword table are behind its unstable `internals` feature, and a
/// copied list would drift with rhai upgrades.
fn map_key(key: &str) -> String {
    let shaped = key.starts_with(|ch: char| ch.is_ascii_alphabetic() || ch == '_')
        && key
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && key.chars().any(|ch| ch.is_ascii_alphabetic());
    if shaped
        && rhai::Engine::new_raw()
            .compile_expression(format!("#{{ {key}: 0 }}"))
            .is_ok()
    {
        key.to_string()
    } else {
        string_literal(key)
    }
}

/// `#{ a: 1, b: 2 }` of the `key: value` pairs, `#{}` when empty.
fn rhai_map(fields: impl IntoIterator<Item = String>) -> String {
    let fields: Vec<String> = fields.into_iter().collect();
    if fields.is_empty() {
        "#{}".to_string()
    } else {
        format!("#{{ {} }}", fields.join(", "))
    }
}

/// `[a, b]` of the items, `[]` when empty.
fn rhai_list(items: impl IntoIterator<Item = String>) -> String {
    format!("[{}]", items.into_iter().collect::<Vec<_>>().join(", "))
}

/// `All` as its number, `ByTier` as a `#{ id: n }` map.
fn per_tier<T: fmt::Display>(per_tier: &PerTier<T>) -> String {
    match per_tier {
        PerTier::All(value) => value.to_string(),
        PerTier::ByTier(by_tier) => {
            rhai_map(by_tier.iter().map(|(id, value)| field(map_key(id), value)))
        }
    }
}

/// `weak_foot` as `#{ usage: u, accuracy: a }` (its two fields required).
fn weak_foot_map(weak_foot: &WeakFoot) -> String {
    let WeakFoot { usage, accuracy } = weak_foot;
    rhai_map([field("usage", usage), field("accuracy", accuracy)])
}

/// `extra_height` as `#{ total: t, max: m }` (its two fields required).
fn extra_height_map(extra_height: &ExtraHeight) -> String {
    let ExtraHeight { total, max } = extra_height;
    rhai_map([field("total", total), field("max", max)])
}

/// `weight` as `#{ floor: f, below: b, above: a }` (its fields required).
fn weight_map(weight: &WeightRule) -> String {
    let WeightRule {
        floor,
        below,
        above,
    } = weight;
    rhai_map([
        field("floor", floor),
        field("below", below),
        field("above", above),
    ])
}

/// A `stat_bonus` map key: `all` or the stat's label.
fn stat_or_all_key(key: &StatOrAll) -> String {
    match key {
        StatOrAll::All => "all".to_string(),
        StatOrAll::Stat(stat) => map_key(stat.label()),
    }
}

/// A `ranges` map key: a stat's label or one of the names
/// `PlayerValue`'s `TryFrom<String>` reads.
fn player_value_key(value: &PlayerValue) -> String {
    map_key(match value {
        PlayerValue::Stat(stat) => stat.label(),
        PlayerValue::Age => "age",
        PlayerValue::Weight => "weight",
        PlayerValue::Form => "form",
        PlayerValue::InjuryResistance => "injury_resistance",
        PlayerValue::WeakFootUsage => "weak_foot_usage",
        PlayerValue::WeakFootAccuracy => "weak_foot_accuracy",
    })
}

/// `severity` as its lowercase spelling.
fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Suggestion => "suggestion",
    }
}

/// `condition` as a `#{ ... }` map literal, every non-empty field in
/// declaration order.
fn condition_map(condition: &Condition) -> String {
    let Condition {
        tiers,
        positions,
        systems,
        brackets,
        height_min,
        height_max,
        captain,
        playing_styles,
        holds,
        rated_a_at,
        rated_b_at,
        name_colors,
        fielded_only_at,
        ranges,
        strong_foot,
        any,
        not,
    } = condition;
    let mut fields: Vec<String> = Vec::new();
    if !tiers.is_empty() {
        fields.push(field(
            "tiers",
            rhai_list(tiers.iter().map(|id| string_literal(id))),
        ));
    }
    if !positions.is_empty() {
        fields.push(field(
            "positions",
            rhai_list(positions.iter().map(|pos| string_literal(pos.label()))),
        ));
    }
    if !systems.is_empty() {
        fields.push(field(
            "systems",
            rhai_list(systems.iter().map(|id| string_literal(id))),
        ));
    }
    if !brackets.is_empty() {
        fields.push(field(
            "brackets",
            rhai_list(brackets.iter().map(|id| string_literal(id))),
        ));
    }
    if let Some(height_min) = height_min {
        fields.push(field("height_min", height_min));
    }
    if let Some(height_max) = height_max {
        fields.push(field("height_max", height_max));
    }
    if let Some(captain) = captain {
        fields.push(field("captain", captain));
    }
    if !playing_styles.is_empty() {
        fields.push(field(
            "playing_styles",
            rhai_list(
                playing_styles
                    .iter()
                    .map(|style| string_literal(style.label())),
            ),
        ));
    }
    if !holds.is_empty() {
        fields.push(field(
            "holds",
            rhai_list(holds.iter().map(|card| string_literal(card.label()))),
        ));
    }
    if !rated_a_at.is_empty() {
        fields.push(field(
            "rated_a_at",
            rhai_list(rated_a_at.iter().map(|pos| string_literal(pos.label()))),
        ));
    }
    if !rated_b_at.is_empty() {
        fields.push(field(
            "rated_b_at",
            rhai_list(rated_b_at.iter().map(|pos| string_literal(pos.label()))),
        ));
    }
    if !name_colors.is_empty() {
        fields.push(field(
            "name_colors",
            rhai_list(name_colors.iter().map(|code| string_literal(code))),
        ));
    }
    if let Some(fielded_only_at) = fielded_only_at {
        fields.push(field(
            "fielded_only_at",
            string_literal(fielded_only_at.label()),
        ));
    }
    if !ranges.is_empty() {
        fields.push(field(
            "ranges",
            rhai_map(ranges.iter().map(|(value, (low, high))| {
                field(player_value_key(value), format!("[{low}, {high}]"))
            })),
        ));
    }
    if let Some(strong_foot) = strong_foot {
        fields.push(field("strong_foot", string_literal(strong_foot.label())));
    }
    if !any.is_empty() {
        fields.push(field("any", rhai_list(any.iter().map(condition_map))));
    }
    if let Some(not) = not {
        fields.push(field("not", condition_map(not)));
    }
    rhai_map(fields)
}

/// `effects` as a `#{ ... }` map literal, every non-empty field in
/// declaration order.
fn effects_map(effects: &Effects) -> String {
    let Effects {
        stat_bonus,
        stat_target,
        cards,
        free,
        free_cards,
        require_cards,
        a_positions,
        max_a_positions,
        form,
        injury_resistance,
        weak_foot,
        height_allowance,
        forbid,
        choice,
        stat_warning,
    } = effects;
    let mut fields: Vec<String> = Vec::new();
    if !stat_bonus.is_empty() {
        fields.push(field(
            "stat_bonus",
            rhai_map(
                stat_bonus
                    .iter()
                    .map(|(key, value)| field(stat_or_all_key(key), per_tier(value))),
            ),
        ));
    }
    if !stat_target.is_empty() {
        fields.push(field(
            "stat_target",
            rhai_map(
                stat_target
                    .iter()
                    .map(|(stat, value)| field(map_key(stat.label()), per_tier(value))),
            ),
        ));
    }
    if let Some(cards) = cards {
        fields.push(field("cards", per_tier(cards)));
    }
    if !free.is_empty() {
        fields.push(field(
            "free",
            rhai_map(
                free.iter()
                    .map(|(group, value)| field(map_key(group), per_tier(value))),
            ),
        ));
    }
    if !free_cards.is_empty() {
        fields.push(field(
            "free_cards",
            rhai_list(free_cards.iter().map(|card| string_literal(card.label()))),
        ));
    }
    if !require_cards.is_empty() {
        fields.push(field(
            "require_cards",
            rhai_list(
                require_cards
                    .iter()
                    .map(|card| string_literal(card.label())),
            ),
        ));
    }
    if let Some(a_positions) = a_positions {
        fields.push(field("a_positions", per_tier(a_positions)));
    }
    if let Some(max_a_positions) = max_a_positions {
        fields.push(field("max_a_positions", max_a_positions));
    }
    if let Some(form) = form {
        fields.push(field("form", per_tier(form)));
    }
    if let Some(injury_resistance) = injury_resistance {
        fields.push(field("injury_resistance", per_tier(injury_resistance)));
    }
    if let Some(weak_foot) = weak_foot {
        fields.push(field("weak_foot", weak_foot_map(weak_foot)));
    }
    if let Some(height_allowance) = height_allowance {
        fields.push(field("height_allowance", height_allowance));
    }
    if let Some(forbid) = forbid {
        fields.push(field("forbid", string_literal(forbid)));
    }
    if !choice.is_empty() {
        fields.push(field("choice", rhai_list(choice.iter().map(effects_map))));
    }
    if let Some(stat_warning) = stat_warning {
        fields.push(field("stat_warning", per_tier(stat_warning)));
    }
    rhai_map(fields)
}

/// `tier`'s fields as `key: value` strings, in declaration order: a
/// field without `#[serde(default)]` always written, a field with one
/// left out when it equals its default.
fn tier_fields(tier: &Tier) -> Vec<String> {
    let Tier {
        id,
        label,
        count,
        counts_as,
        when,
        rate,
        stats,
        form,
        injury_resistance,
        weak_foot,
        cards,
        min_cards,
        free,
        a_positions,
        name_color,
        stat_warning,
        total_within_targets,
    } = tier;
    let mut fields = vec![
        field("id", string_literal(id)),
        field("label", string_literal(label)),
    ];
    if let Some(count) = count {
        fields.push(field("count", count));
    }
    if let Some(counts_as) = counts_as {
        fields.push(field("counts_as", string_literal(counts_as)));
    }
    if !when.is_empty() {
        fields.push(field("when", condition_map(when)));
    }
    fields.push(field("rate", rate));
    if !stats.is_empty() {
        fields.push(field(
            "stats",
            rhai_map(
                stats
                    .iter()
                    .map(|(stat, target)| field(map_key(stat.label()), target)),
            ),
        ));
    }
    if *form != 0 {
        fields.push(field("form", form));
    }
    if *injury_resistance != 0 {
        fields.push(field("injury_resistance", injury_resistance));
    }
    if *weak_foot != WeakFoot::default() {
        fields.push(field("weak_foot", weak_foot_map(weak_foot)));
    }
    if *cards != 0 {
        fields.push(field("cards", cards));
    }
    if *min_cards != 0 {
        fields.push(field("min_cards", min_cards));
    }
    if !free.is_empty() {
        fields.push(field(
            "free",
            rhai_map(
                free.iter()
                    .map(|(group, count)| field(map_key(group), count)),
            ),
        ));
    }
    if *a_positions != 0 {
        fields.push(field("a_positions", a_positions));
    }
    if let Some(name_color) = name_color {
        fields.push(field("name_color", string_literal(name_color)));
    }
    if *stat_warning != 0 {
        fields.push(field("stat_warning", stat_warning));
    }
    if *total_within_targets {
        fields.push(field("total_within_targets", total_within_targets));
    }
    fields
}

/// `system`'s fields as `key: value` strings, in declaration order
/// (`label` and `counts` required).
fn medal_system_fields(system: &MedalSystem) -> Vec<String> {
    let MedalSystem { label, counts } = system;
    vec![
        field("label", string_literal(label)),
        field(
            "counts",
            rhai_map(counts.iter().map(|(id, count)| field(map_key(id), count))),
        ),
    ]
}

/// `bracket`'s fields as `key: value` strings, `up_to` or `exactly`
/// flattened into the bracket's map as it is read.
fn bracket_fields(bracket: &Bracket) -> Vec<String> {
    let Bracket { id, label, heights } = bracket;
    let mut fields = vec![
        field("id", string_literal(id)),
        field("label", string_literal(label)),
    ];
    match heights {
        BracketHeights::UpTo(up_to) => fields.push(field("up_to", up_to)),
        BracketHeights::Exactly(exactly) => fields.push(field(
            "exactly",
            rhai_list(exactly.iter().map(u8::to_string)),
        )),
    }
    fields
}

/// `system`'s fields as `key: value` strings: `by_quotas` before
/// `quotas`, as the hand-written blocks spell it.
fn system_fields(system: &HeightSystem) -> Vec<String> {
    let HeightSystem {
        id,
        label,
        any_player,
        quotas,
        by_quotas,
        extra_height,
    } = system;
    let mut fields = vec![
        field("id", string_literal(id)),
        field("label", string_literal(label)),
    ];
    if let Some(any_player) = any_player {
        fields.push(field("any_player", condition_map(any_player)));
    }
    if *by_quotas {
        fields.push(field("by_quotas", by_quotas));
    }
    if !quotas.is_empty() {
        fields.push(field(
            "quotas",
            rhai_map(
                quotas
                    .iter()
                    .map(|(bracket, count)| field(map_key(bracket), count)),
            ),
        ));
    }
    if let Some(extra_height) = extra_height {
        fields.push(field("extra_height", extra_height_map(extra_height)));
    }
    fields
}

/// `special`'s fields as `key: value` strings: `severity` right after
/// `label`, as the hand-written blocks spell it.
fn special_fields(special: &Special) -> Vec<String> {
    let Special {
        label,
        when,
        then,
        severity,
    } = special;
    let mut fields = vec![field("label", string_literal(label))];
    if *severity != Severity::Error {
        fields.push(field("severity", string_literal(severity_label(*severity))));
    }
    if !when.is_empty() {
        fields.push(field("when", condition_map(when)));
    }
    if *then != Effects::default() {
        fields.push(field("then", effects_map(then)));
    }
    fields
}

/// `settings`' fields as `key: value` strings, in declaration order.
fn team_settings_fields(settings: &TeamSettings) -> Vec<String> {
    let TeamSettings {
        label,
        versions,
        man_marking,
        auto_substitution,
        auto_offside_trap,
        auto_preset_change,
        auto_attack_defence_levels,
        forbidden_shapes,
    } = settings;
    let mut fields = vec![field("label", string_literal(label))];
    if !versions.is_empty() {
        fields.push(field(
            "versions",
            rhai_list(versions.iter().map(|version| version.number().to_string())),
        ));
    }
    for (key, setting) in [
        ("man_marking", man_marking),
        ("auto_substitution", auto_substitution),
        ("auto_offside_trap", auto_offside_trap),
        ("auto_preset_change", auto_preset_change),
        ("auto_attack_defence_levels", auto_attack_defence_levels),
    ] {
        if let Some(setting) = setting {
            fields.push(field(key, setting));
        }
    }
    if !forbidden_shapes.is_empty() {
        fields.push(field(
            "forbidden_shapes",
            rhai_list(
                forbidden_shapes
                    .iter()
                    .map(|shape| rhai_list(shape.iter().map(u8::to_string))),
            ),
        ));
    }
    fields
}

/// `universal`'s fields as `key: value` strings, in declaration order
/// (`max_skill_cards` required).
fn universal_fields(universal: &Universal) -> Vec<String> {
    let Universal {
        registered_position_a,
        fielded_at_a,
        no_b_ratings,
        b_uses_a_allowance,
        gk_not_second_a,
        age,
        weight,
        captain_required,
        min_goalkeepers,
        max_skill_cards,
        max_com_styles,
    } = universal;
    let mut fields: Vec<String> = Vec::new();
    if *registered_position_a {
        fields.push(field("registered_position_a", registered_position_a));
    }
    if !fielded_at_a.is_empty() {
        fields.push(field(
            "fielded_at_a",
            rhai_list(fielded_at_a.iter().map(u8::to_string)),
        ));
    }
    if *no_b_ratings {
        fields.push(field("no_b_ratings", no_b_ratings));
    }
    if *b_uses_a_allowance {
        fields.push(field("b_uses_a_allowance", b_uses_a_allowance));
    }
    if *gk_not_second_a {
        fields.push(field("gk_not_second_a", gk_not_second_a));
    }
    if let Some((low, high)) = age {
        fields.push(field("age", format!("[{low}, {high}]")));
    }
    if let Some(weight) = weight {
        fields.push(field("weight", weight_map(weight)));
    }
    if *captain_required {
        fields.push(field("captain_required", captain_required));
    }
    if *min_goalkeepers != 0 {
        fields.push(field("min_goalkeepers", min_goalkeepers));
    }
    fields.push(field("max_skill_cards", max_skill_cards));
    if let Some(max_com_styles) = max_com_styles {
        fields.push(field("max_com_styles", max_com_styles));
    }
    fields
}

/// One `#{ field: value, ... },` item under `indent`: the fields joined
/// `, `, wrapping onto continuation lines indented three more spaces
/// (under the first field) when the line would pass `WIDTH`. A field is
/// never split across lines: a longer one sits alone on its line,
/// overlong.
fn wrap_item(indent: &str, fields: Vec<String>) -> Vec<String> {
    if fields.is_empty() {
        return vec![format!("{indent}#{{}},")];
    }
    let mut lines = Vec::new();
    let mut line = format!("{indent}#{{ ");
    let last = fields.len() - 1;
    for (index, entry) in fields.iter().enumerate() {
        let piece = if index == last {
            format!("{entry} }},")
        } else {
            format!("{entry},")
        };
        if line.trim() != "#{" && line.len() + piece.len() + 1 > WIDTH {
            lines.push(line.trim_end().to_string());
            line = format!("{indent}   ");
        }
        line.push_str(&piece);
        line.push(' ');
    }
    lines.push(line.trim_end().to_string());
    lines
}

/// `head`'s `[a, b, c]` list under `indent`, wrapping onto continuation
/// lines aligned under the first item when the line would pass `WIDTH`.
fn wrap_list(indent: &str, head: &str, items: impl IntoIterator<Item = String>) -> Vec<String> {
    let items: Vec<String> = items.into_iter().collect();
    let continuation = " ".repeat(indent.len() + head.len() + 1);
    let mut lines = Vec::new();
    let mut line = format!("{indent}{head}[");
    let last = items.len().saturating_sub(1);
    for (index, item) in items.iter().enumerate() {
        let piece = if index == last {
            format!("{item}],")
        } else {
            format!("{item},")
        };
        if index > 0 && line.len() + piece.len() + 1 > WIDTH {
            lines.push(line.trim_end().to_string());
            line = continuation.clone();
        }
        line.push_str(&piece);
        line.push(' ');
    }
    if items.is_empty() {
        line.push_str("],");
    }
    lines.push(line.trim_end().to_string());
    lines
}

/// The `universal: #{ ... }` section: the fields joined `, ` on lines
/// one indent in, wrapped onto same-indent lines when the line would
/// pass `WIDTH`, a trailing comma per line.
fn universal_section(universal: &Universal) -> Vec<String> {
    let mut lines = vec![format!("{INDENT}universal: #{{")];
    // `entry, ` after each field but the last (`entry,`): a wrapped
    // line ends with the field's comma.
    let mut pieces: Vec<String> = universal_fields(universal)
        .iter()
        .map(|entry| format!("{entry}, "))
        .collect();
    if let Some(last) = pieces.last_mut() {
        last.pop();
    }
    let mut line = I2.to_string();
    for piece in &pieces {
        if !line.trim().is_empty() && line.len() + piece.len() > WIDTH {
            lines.push(line.trim_end().to_string());
            line = I2.to_string();
        }
        line.push_str(piece);
    }
    lines.push(line.trim_end().to_string());
    lines.push(format!("{INDENT}}},"));
    lines
}

/// One top-level section of `#[ field ]` items under `I2`.
fn item_section(head: &str, items: Vec<Vec<String>>) -> Vec<String> {
    let mut lines = vec![format!("{INDENT}{head}: [")];
    for fields in items {
        lines.extend(wrap_item(I2, fields));
    }
    lines.push(format!("{INDENT}],"));
    lines
}

impl Ruleset {
    /// The data block that reads back as this ruleset: the header
    /// comment line and `const RULESET = #{ ... };`, ending in a
    /// newline.
    pub fn to_data_block(&self) -> String {
        // Destructuring names every field on purpose: a field added to
        // the schema fails to compile here until the writer handles it.
        // The same holds of the field functions below. The fields come
        // in declaration order but two, spelled as the hand-written
        // blocks and the editor's preview read them: a special's
        // `severity` follows `label`, a height system's `by_quotas`
        // precedes `quotas`.
        let Ruleset {
            schema,
            name,
            squad_size,
            stats,
            tiers,
            card_groups,
            medal_systems,
            heights,
            universal,
            specials,
            team_settings,
            suggestions,
        } = self;
        let mut sections: Vec<Vec<String>> = Vec::new();

        let StatRules { rating, at_most } = stats;
        if !rating.is_empty() || !at_most.is_empty() {
            let mut lines = vec![format!("{INDENT}stats: #{{")];
            if !rating.is_empty() {
                lines.extend(wrap_list(
                    I2,
                    "rating: ",
                    rating.iter().map(|stat| string_literal(stat.label())),
                ));
            }
            if !at_most.is_empty() {
                lines.extend(wrap_list(
                    I2,
                    "at_most: ",
                    at_most.iter().map(|name| string_literal(name)),
                ));
            }
            lines.push(format!("{INDENT}}},"));
            sections.push(lines);
        }

        if !tiers.is_empty() {
            sections.push(item_section(
                "tiers",
                tiers.iter().map(tier_fields).collect(),
            ));
        }

        if !card_groups.is_empty() {
            let mut lines = vec![format!("{INDENT}card_groups: #{{")];
            for (group, cards) in card_groups {
                lines.extend(wrap_list(
                    I2,
                    &format!("{}: ", map_key(group)),
                    cards.iter().map(|card| string_literal(card.label())),
                ));
            }
            lines.push(format!("{INDENT}}},"));
            sections.push(lines);
        }

        if !medal_systems.is_empty() {
            sections.push(item_section(
                "medal_systems",
                medal_systems.iter().map(medal_system_fields).collect(),
            ));
        }

        let Heights { brackets, systems } = heights;
        if !brackets.is_empty() || !systems.is_empty() {
            let mut lines = vec![format!("{INDENT}heights: #{{")];
            if !brackets.is_empty() {
                lines.push(format!("{I2}brackets: ["));
                for bracket in brackets {
                    lines.extend(wrap_item(I3, bracket_fields(bracket)));
                }
                lines.push(format!("{I2}],"));
            }
            if !systems.is_empty() {
                lines.push(format!("{I2}systems: ["));
                for system in systems {
                    lines.extend(wrap_item(I3, system_fields(system)));
                }
                lines.push(format!("{I2}],"));
            }
            lines.push(format!("{INDENT}}},"));
            sections.push(lines);
        }

        sections.push(universal_section(universal));

        if !specials.is_empty() {
            sections.push(item_section(
                "specials",
                specials.iter().map(special_fields).collect(),
            ));
        }

        if !team_settings.is_empty() {
            sections.push(item_section(
                "team_settings",
                team_settings.iter().map(team_settings_fields).collect(),
            ));
        }

        if !suggestions.is_empty() {
            sections.push(wrap_list(
                INDENT,
                "suggestions: ",
                suggestions.iter().map(|entry| string_literal(entry)),
            ));
        }

        let mut block = format!(
            "{HEADER}\nconst RULESET = #{{\n{INDENT}schema: {schema},\n{INDENT}name: {},\n{INDENT}squad_size: {squad_size},\n",
            string_literal(name)
        );
        for section in &sections {
            block.push('\n');
            for line in section {
                block.push_str(line);
                block.push('\n');
            }
        }
        block.push_str("};\n");
        block
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use pes_version::PesVersion;

    use super::*;
    use crate::ruleset::{Card, Foot, PlayStyle, Position, Stat};

    /// `name` as a `Stat`.
    fn stat(name: &str) -> Stat {
        Stat::try_from(name.to_string()).expect("a stat name")
    }

    /// `name` as a `Position`.
    fn position(name: &str) -> Position {
        Position::try_from(name.to_string()).expect("a position name")
    }

    /// `name` as a `PlayStyle`.
    fn play_style(name: &str) -> PlayStyle {
        PlayStyle::try_from(name.to_string()).expect("a playing style")
    }

    /// `name` as a `Card`.
    fn card(name: &str) -> Card {
        Card::try_from(name.to_string()).expect("a card name")
    }

    /// `map_key`'s bare-or-literal decision: an identifier shape with a
    /// letter is bare when rhai's parser takes `#{ <key>: 0 }` (`in`,
    /// `go` are reserved), everything else a string literal — `a: 0, b`
    /// among them, which the parser alone would take as two keys.
    #[test]
    fn map_key_spells_identifiers_bare_and_the_rest_quoted() {
        for (key, expected) in [
            ("gold", "gold"),
            ("_x", "_x"),
            ("ball_winning", "ball_winning"),
            ("a_1", "a_1"),
            ("a: 0, b", "\"a: 0, b\""),
            ("_", "\"_\""),
            ("__", "\"__\""),
            ("1st", "\"1st\""),
            ("in", "\"in\""),
            ("go", "\"go\""),
            ("médaille", "\"médaille\""),
        ] {
            assert_eq!(map_key(key), expected, "{key}");
        }
    }

    /// `wrap_item`'s edge: a second field ending the line at the width
    /// stays, one character more wraps it under the first field (the
    /// indent plus three spaces).
    #[test]
    fn wrap_item_wraps_the_field_over_the_width() {
        // `        #{ ` (11) + `a: 1, ` (6) leaves 17; a 79-char field's
        // ` },` piece is 82, and 17 + 82 + 1 is exactly WIDTH.
        let lines = wrap_item("        ", vec!["a: 1".to_string(), "x".repeat(79)]);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert_eq!(lines[0].len(), 99);
        assert_eq!(
            lines[0],
            format!("        #{{ a: 1, {} }},", "x".repeat(79))
        );
        let lines = wrap_item("        ", vec!["a: 1".to_string(), "x".repeat(80)]);
        assert_eq!(
            lines,
            vec![
                "        #{ a: 1,".to_string(),
                format!("           {} }},", "x".repeat(80)),
            ]
        );
    }

    /// `wrap_list`'s continuation is aligned under the first item
    /// (`head`'s `[` and one more), and the first item never wraps: an
    /// overlong one stays on the head's line.
    #[test]
    fn wrap_list_wraps_under_the_first_item() {
        // `        rating: [` is 17 columns: the continuation is 17
        // spaces.
        let lines = wrap_list(
            "        ",
            "rating: ",
            vec!["\"a\"".to_string(), "\"b\"".to_string(), "x".repeat(75)],
        );
        assert_eq!(
            lines,
            vec![
                "        rating: [\"a\", \"b\",".to_string(),
                format!("{}{}],", " ".repeat(17), "x".repeat(75)),
            ]
        );
        let lines = wrap_list("    ", "suggestions: ", vec!["x".repeat(90)]);
        assert_eq!(
            lines,
            vec![format!("    suggestions: [{}],", "x".repeat(90))],
            "the first item never wraps"
        );
    }

    /// `universal`'s fields joined `, ` one indent in, each line ending
    /// a comma: `max_skill_cards`'s piece lands the line on exactly
    /// `WIDTH` and stays, `max_com_styles` wraps onto its own line.
    #[test]
    fn universal_wraps_onto_same_indent_lines() {
        let universal = Universal {
            registered_position_a: true,
            fielded_at_a: Vec::new(),
            no_b_ratings: false,
            b_uses_a_allowance: true,
            gk_not_second_a: false,
            age: Some((18, 135)),
            weight: None,
            captain_required: false,
            min_goalkeepers: 0,
            max_skill_cards: 10,
            max_com_styles: Some(2),
        };
        assert_eq!(
            universal_section(&universal),
            vec![
                "    universal: #{".to_string(),
                "        registered_position_a: true, b_uses_a_allowance: true, age: [18, 135], max_skill_cards: 10,".to_string(),
                "        max_com_styles: 2,".to_string(),
                "    },".to_string(),
            ]
        );
    }

    /// The sections' presence: `stats` writes only the list it holds
    /// (either alone), `heights` either list it holds — and neither
    /// writes an empty section. The editor's preview hands the writer
    /// rulesets the load rules would refuse, so `brackets` without
    /// `systems` still writes.
    #[test]
    fn sections_appear_only_with_content() {
        // VGL26 has no heights: no `heights:` section.
        let vgl26 =
            Ruleset::from_data_block(include_str!("../rulesets/vgl26.rhai")).expect("VGL26 loads");
        assert!(!vgl26.to_data_block().contains("heights:"));

        let mut ruleset = small_ruleset();
        // `small_ruleset` has `rating` and `at_most` empty: no `stats:`.
        assert!(!ruleset.to_data_block().contains("stats:"));

        // Brackets but no systems: `brackets:` alone.
        ruleset.heights.systems.clear();
        let written = ruleset.to_data_block();
        assert!(written.contains("brackets: ["), "{written}");
        assert!(!written.contains("systems:"), "{written}");

        // `stats` with `rating` only, then `at_most` only.
        ruleset.stats = StatRules {
            rating: vec![stat("speed")],
            at_most: Vec::new(),
        };
        let written = ruleset.to_data_block();
        assert!(written.contains("stats: #{"), "{written}");
        assert!(written.contains("rating: ["), "{written}");
        assert!(!written.contains("at_most:"), "{written}");
        ruleset.stats = StatRules {
            rating: Vec::new(),
            at_most: vec!["form".to_string()],
        };
        let written = ruleset.to_data_block();
        assert!(written.contains("at_most: ["), "{written}");
        assert!(!written.contains("rating:"), "{written}");
    }

    /// Every embedded ruleset: written, read back equal, written again
    /// byte-identical. `vtl10` and `vtl11` end in a CUSTOM CHECKS
    /// section the writer does not carry, so the comparison is of the
    /// `Ruleset`s, not of the written text to the file.
    #[test]
    fn every_embedded_ruleset_round_trips() {
        for (name, text) in [
            ("autumn25", include_str!("../rulesets/autumn25.rhai")),
            ("autumn26", include_str!("../rulesets/autumn26.rhai")),
            ("dbc7", include_str!("../rulesets/dbc7.rhai")),
            ("spring26", include_str!("../rulesets/spring26.rhai")),
            ("vgl25", include_str!("../rulesets/vgl25.rhai")),
            ("vgl26", include_str!("../rulesets/vgl26.rhai")),
            ("vgl27", include_str!("../rulesets/vgl27.rhai")),
            ("vtl10", include_str!("../rulesets/vtl10.rhai")),
            ("vtl11", include_str!("../rulesets/vtl11.rhai")),
        ] {
            let ruleset =
                Ruleset::from_data_block(text).unwrap_or_else(|error| panic!("{name}: {error}"));
            let written = ruleset.to_data_block();
            let reread = Ruleset::from_data_block(&written)
                .unwrap_or_else(|error| panic!("{name}: {error}\n{written}"));
            assert_eq!(reread, ruleset, "{name}");
            assert_eq!(reread.to_data_block(), written, "{name}");
        }
    }

    /// A `Condition` holding `height_min`/`height_max` and nothing else
    /// (every field named, the same compile guard as the writer's
    /// destructuring, throughout this module).
    fn height_condition(low: u8, high: u8) -> Condition {
        Condition {
            tiers: Vec::new(),
            positions: Vec::new(),
            systems: Vec::new(),
            brackets: Vec::new(),
            height_min: Some(low),
            height_max: Some(high),
            captain: None,
            playing_styles: Vec::new(),
            holds: Vec::new(),
            rated_a_at: Vec::new(),
            rated_b_at: Vec::new(),
            name_colors: Vec::new(),
            fielded_only_at: None,
            ranges: BTreeMap::new(),
            strong_foot: None,
            any: Vec::new(),
            not: None,
        }
    }

    /// A `Condition` holding only `strong_foot`.
    fn strong_foot_condition(strong_foot: Option<Foot>) -> Condition {
        Condition {
            tiers: Vec::new(),
            positions: Vec::new(),
            systems: Vec::new(),
            brackets: Vec::new(),
            height_min: None,
            height_max: None,
            captain: None,
            playing_styles: Vec::new(),
            holds: Vec::new(),
            rated_a_at: Vec::new(),
            rated_b_at: Vec::new(),
            name_colors: Vec::new(),
            fielded_only_at: None,
            ranges: BTreeMap::new(),
            strong_foot,
            any: Vec::new(),
            not: None,
        }
    }

    /// A `Condition` holding only `positions` (the `not` below).
    fn positions_condition(positions: Vec<Position>) -> Condition {
        Condition {
            tiers: Vec::new(),
            positions,
            systems: Vec::new(),
            brackets: Vec::new(),
            height_min: None,
            height_max: None,
            captain: None,
            playing_styles: Vec::new(),
            holds: Vec::new(),
            rated_a_at: Vec::new(),
            rated_b_at: Vec::new(),
            name_colors: Vec::new(),
            fielded_only_at: None,
            ranges: BTreeMap::new(),
            strong_foot: None,
            any: Vec::new(),
            not: None,
        }
    }

    /// A `Condition` holding only `captain`.
    fn captain_condition(captain: Option<bool>) -> Condition {
        Condition {
            tiers: Vec::new(),
            positions: Vec::new(),
            systems: Vec::new(),
            brackets: Vec::new(),
            height_min: None,
            height_max: None,
            captain,
            playing_styles: Vec::new(),
            holds: Vec::new(),
            rated_a_at: Vec::new(),
            rated_b_at: Vec::new(),
            name_colors: Vec::new(),
            fielded_only_at: None,
            ranges: BTreeMap::new(),
            strong_foot: None,
            any: Vec::new(),
            not: None,
        }
    }

    /// A `Condition` with every field set non-default: `any` and `not`
    /// nested, every `PlayerValue` in `ranges`, both `Foot`s across the
    /// condition and its `any` entry.
    fn full_condition() -> Condition {
        Condition {
            tiers: vec!["mid".to_string()],
            positions: vec![position("GK")],
            systems: vec!["green".to_string()],
            brackets: vec!["tall".to_string()],
            height_min: Some(180),
            height_max: Some(190),
            captain: Some(true),
            playing_styles: vec![play_style("goal_poacher")],
            holds: vec![card("scissors_feint"), card("long_ranger")],
            rated_a_at: vec![position("CF")],
            rated_b_at: vec![position("CB")],
            name_colors: vec!["cc9900ff".to_string()],
            fielded_only_at: Some(position("SS")),
            ranges: [
                (PlayerValue::Stat(stat("speed")), (90, 99)),
                (PlayerValue::Age, (18, 35)),
                (PlayerValue::Weight, (70, 90)),
                (PlayerValue::Form, (4, 8)),
                (PlayerValue::InjuryResistance, (1, 3)),
                (PlayerValue::WeakFootUsage, (1, 4)),
                (PlayerValue::WeakFootAccuracy, (1, 4)),
            ]
            .into_iter()
            .collect(),
            strong_foot: Some(Foot::Right),
            any: vec![
                strong_foot_condition(Some(Foot::Left)),
                captain_condition(Some(false)),
            ],
            not: Some(Box::new(positions_condition(vec![position("SS")]))),
        }
    }

    /// `(&str, n)` pairs as a `BTreeMap<String, _>` (tier ids in
    /// `PerTier::ByTier`, group ids in `free`, counts in `quotas`).
    fn by_id<T: Copy>(pairs: &[(&str, T)]) -> BTreeMap<String, T> {
        pairs
            .iter()
            .map(|&(id, value)| (id.to_string(), value))
            .collect()
    }

    /// An `Effects` with every field set non-default but
    /// `height_allowance` (`full_condition` names brackets, which an
    /// allowance forbids): both `PerTier` forms and both `StatOrAll`s.
    fn full_effects() -> Effects {
        Effects {
            stat_bonus: [
                (
                    StatOrAll::All,
                    PerTier::ByTier(by_id(&[("mid", 5), ("gold", 3)])),
                ),
                (StatOrAll::Stat(stat("speed")), PerTier::All(-3)),
            ]
            .into_iter()
            .collect(),
            stat_target: [(
                stat("finishing"),
                PerTier::ByTier(by_id(&[("regular", 90)])),
            )]
            .into_iter()
            .collect(),
            cards: Some(PerTier::All(2)),
            free: [
                ("com".to_string(), PerTier::All(1)),
                (
                    "gold-medal".to_string(),
                    PerTier::ByTier(by_id(&[("mid", 2)])),
                ),
            ]
            .into_iter()
            .collect(),
            free_cards: vec![card("malicia"), card("long_ranger")],
            require_cards: vec![card("captaincy")],
            a_positions: Some(PerTier::All(1)),
            max_a_positions: Some(4),
            form: Some(PerTier::ByTier(by_id(&[("gold", 2)]))),
            injury_resistance: Some(PerTier::All(-1)),
            weak_foot: Some(WeakFoot {
                usage: 4,
                accuracy: 4,
            }),
            height_allowance: None,
            forbid: Some("The special forbids this".to_string()),
            choice: vec![choice_effects()],
            stat_warning: Some(PerTier::ByTier(by_id(&[("regular", 5)]))),
        }
    }

    /// A `choice` option: only the effects that raise a limit.
    fn choice_effects() -> Effects {
        Effects {
            stat_bonus: [(StatOrAll::Stat(stat("speed")), PerTier::All(5))]
                .into_iter()
                .collect(),
            stat_target: BTreeMap::new(),
            cards: Some(PerTier::ByTier(by_id(&[("mid", 1)]))),
            free: [("in".to_string(), PerTier::All(1))].into_iter().collect(),
            free_cards: vec![card("malicia")],
            require_cards: Vec::new(),
            a_positions: Some(PerTier::All(1)),
            max_a_positions: Some(3),
            form: None,
            injury_resistance: None,
            weak_foot: Some(WeakFoot {
                usage: 3,
                accuracy: 3,
            }),
            height_allowance: None,
            forbid: None,
            choice: Vec::new(),
            stat_warning: None,
        }
    }

    /// An `Effects` holding only `height_allowance`.
    fn allowance_effects() -> Effects {
        Effects {
            stat_bonus: BTreeMap::new(),
            stat_target: BTreeMap::new(),
            cards: None,
            free: BTreeMap::new(),
            free_cards: Vec::new(),
            require_cards: Vec::new(),
            a_positions: None,
            max_a_positions: None,
            form: None,
            injury_resistance: None,
            weak_foot: None,
            height_allowance: Some(4),
            forbid: None,
            choice: Vec::new(),
            stat_warning: None,
        }
    }

    /// An `Effects` with nothing set.
    fn empty_effects() -> Effects {
        Effects {
            stat_bonus: BTreeMap::new(),
            stat_target: BTreeMap::new(),
            cards: None,
            free: BTreeMap::new(),
            free_cards: Vec::new(),
            require_cards: Vec::new(),
            a_positions: None,
            max_a_positions: None,
            form: None,
            injury_resistance: None,
            weak_foot: None,
            height_allowance: None,
            forbid: None,
            choice: Vec::new(),
            stat_warning: None,
        }
    }

    /// A `Tier` with every field set non-default but `count` and
    /// `counts_as` (the two cannot coexist): `when` holds heights only
    /// since a tier's condition may not name tiers.
    fn rich_tier(id: &str, label: &str, count: Option<u8>, counts_as: Option<&str>) -> Tier {
        Tier {
            id: id.to_string(),
            label: label.to_string(),
            count,
            counts_as: counts_as.map(str::to_string),
            when: height_condition(180, 200),
            rate: 90,
            stats: [(stat("speed"), 95)].into_iter().collect(),
            form: 8,
            injury_resistance: 3,
            weak_foot: WeakFoot {
                usage: 4,
                accuracy: 4,
            },
            cards: 6,
            min_cards: 2,
            free: by_id(&[
                ("1st", 1),
                ("com", 1),
                ("go", 1),
                ("gold-medal", 2),
                ("in", 2),
            ]),
            a_positions: 1,
            name_color: Some("cc9900ff".to_string()),
            stat_warning: 5,
            total_within_targets: true,
        }
    }

    /// A ruleset holding every value the schema can spell: every field
    /// of every type named and set non-default (where the load rules
    /// allow), every enum variant present, ids needing string-literal
    /// map keys (`"in"`, `"go"`, `"1st"`, `"gold-medal"`, `"médaille"`)
    /// and labels needing escapes. One ruleset suffices: `any` and
    /// `not` leave room for both `Foot`s and the second `PerTier` form.
    fn every_field_ruleset() -> Ruleset {
        Ruleset {
            schema: 1,
            name: "Every field".to_string(),
            squad_size: 23,
            stats: StatRules {
                rating: vec![stat("speed"), stat("dribbling")],
                at_most: vec!["attacking_prowess".to_string(), "form".to_string()],
            },
            tiers: vec![
                // A `"` `\`, a tab and `é` in `label` (written `\"` `\\`
                // `\t` and as it is).
                rich_tier("gold", "Gold \"médaille\" \\\té", None, None),
                rich_tier("silver", "Silver", None, None),
                rich_tier("mid", "Mid", Some(5), None),
                rich_tier("médaille", "Médaille", None, Some("regular")),
                rich_tier("regular", "Regular", None, None),
            ],
            card_groups: [
                ("1st".to_string(), vec![card("scissors_feint")]),
                ("go".to_string(), vec![card("flip_flap")]),
                ("gold-medal".to_string(), vec![card("marseille_turn")]),
                ("in".to_string(), vec![card("sombrero")]),
            ]
            .into_iter()
            .collect(),
            medal_systems: vec![
                MedalSystem {
                    label: "GGG".to_string(),
                    counts: by_id(&[("gold", 3)]),
                },
                MedalSystem {
                    label: "GGSS".to_string(),
                    counts: by_id(&[("gold", 2), ("silver", 2)]),
                },
            ],
            heights: Heights {
                brackets: vec![
                    Bracket {
                        id: "manlet".to_string(),
                        label: "Manlet".to_string(),
                        heights: BracketHeights::UpTo(175),
                    },
                    Bracket {
                        id: "tall".to_string(),
                        label: "Tall".to_string(),
                        heights: BracketHeights::Exactly(vec![185]),
                    },
                    Bracket {
                        id: "giga".to_string(),
                        label: "Giga".to_string(),
                        heights: BracketHeights::Exactly(vec![210]),
                    },
                ],
                systems: vec![
                    HeightSystem {
                        id: "green".to_string(),
                        label: "Green".to_string(),
                        any_player: Some(height_condition(190, 255)),
                        quotas: by_id(&[("manlet", 5), ("tall", 18)]),
                        by_quotas: true,
                        extra_height: None,
                    },
                    HeightSystem {
                        id: "giga".to_string(),
                        label: "Giga".to_string(),
                        any_player: None,
                        quotas: by_id(&[("tall", 17), ("giga", 6)]),
                        by_quotas: false,
                        extra_height: Some(ExtraHeight {
                            total: 50,
                            max: 210,
                        }),
                    },
                ],
            },
            universal: Universal {
                registered_position_a: true,
                fielded_at_a: vec![1, 2],
                no_b_ratings: true,
                b_uses_a_allowance: true,
                gk_not_second_a: true,
                age: Some((18, 35)),
                weight: Some(WeightRule {
                    floor: 30,
                    below: 129,
                    above: 81,
                }),
                captain_required: true,
                min_goalkeepers: 1,
                max_skill_cards: 10,
                max_com_styles: Some(2),
            },
            specials: vec![
                Special {
                    label: "Every field".to_string(),
                    when: full_condition(),
                    then: full_effects(),
                    severity: Severity::Error,
                },
                Special {
                    label: "Goalkeepers at 189 count as 185".to_string(),
                    when: height_condition(189, 189),
                    then: allowance_effects(),
                    severity: Severity::Warning,
                },
                Special {
                    // U+0001, written `\x01`.
                    label: "A\u{1}suggestion".to_string(),
                    when: strong_foot_condition(None),
                    then: empty_effects(),
                    severity: Severity::Suggestion,
                },
            ],
            team_settings: vec![TeamSettings {
                label: "Support settings".to_string(),
                versions: vec![PesVersion::Pes16, PesVersion::Pes21],
                man_marking: Some(false),
                auto_substitution: Some(true),
                auto_offside_trap: Some(false),
                auto_preset_change: Some(true),
                auto_attack_defence_levels: Some(false),
                forbidden_shapes: vec![[3, 4, 3]],
            }],
            suggestions: vec![
                "cards".to_string(),
                "in".to_string(),
                "go".to_string(),
                "choices".to_string(),
            ],
        }
    }

    /// Every field of every type round-trips, written and read back.
    #[test]
    fn every_field_round_trips() {
        let ruleset = every_field_ruleset();
        let written = ruleset.to_data_block();
        let reread =
            Ruleset::from_data_block(&written).unwrap_or_else(|error| panic!("{error}\n{written}"));
        assert_eq!(reread, ruleset);
        assert_eq!(reread.to_data_block(), written);
        // Map keys: `in`, `go`, `1st`, `gold-medal` and `médaille` need
        // string literals; `gold` and `com` are bare.
        for key in [
            "\"gold-medal\":",
            "\"in\":",
            "\"go\":",
            "\"1st\":",
            "\"médaille\"",
        ] {
            assert!(written.contains(key), "{key} missing from\n{written}");
        }
        assert!(written.contains("gold:"), "a bare key, from\n{written}");
        assert!(written.contains(" com:"), "a bare key, from\n{written}");
        // The escapes: `\"`, `\\`, `\t`, `\x01`; `é` written as it is.
        for escaped in ["\\\"", "\\\\", "\\t", "\\x01", "é"] {
            assert!(
                written.contains(escaped),
                "{escaped} missing from\n{written}"
            );
        }
        // A `severity: "error"` is never written: it is the default.
        assert!(!written.contains("error"), "Severity::Error omitted");
    }

    /// The small ruleset `the_layout_is_pinned` pins: two tiers, one
    /// bracket, one height system, one special, `universal` with two
    /// fields.
    fn small_ruleset() -> Ruleset {
        Ruleset {
            schema: 1,
            name: "Small".to_string(),
            squad_size: 11,
            stats: StatRules {
                rating: Vec::new(),
                at_most: Vec::new(),
            },
            tiers: vec![
                Tier {
                    id: "gold".to_string(),
                    label: "Gold".to_string(),
                    count: Some(3),
                    counts_as: None,
                    when: strong_foot_condition(None),
                    rate: 99,
                    stats: BTreeMap::new(),
                    form: 0,
                    injury_resistance: 3,
                    weak_foot: WeakFoot {
                        usage: 4,
                        accuracy: 4,
                    },
                    cards: 6,
                    min_cards: 0,
                    free: by_id(&[("com", 1)]),
                    a_positions: 0,
                    name_color: None,
                    stat_warning: 0,
                    total_within_targets: false,
                },
                Tier {
                    id: "regular".to_string(),
                    label: "Regular".to_string(),
                    count: None,
                    counts_as: None,
                    when: strong_foot_condition(None),
                    rate: 77,
                    stats: BTreeMap::new(),
                    form: 0,
                    injury_resistance: 0,
                    weak_foot: WeakFoot {
                        usage: 0,
                        accuracy: 0,
                    },
                    cards: 0,
                    min_cards: 0,
                    free: BTreeMap::new(),
                    a_positions: 0,
                    name_color: None,
                    stat_warning: 0,
                    total_within_targets: false,
                },
            ],
            card_groups: BTreeMap::new(),
            medal_systems: Vec::new(),
            heights: Heights {
                brackets: vec![Bracket {
                    id: "tall".to_string(),
                    label: "Tall".to_string(),
                    heights: BracketHeights::Exactly(vec![185]),
                }],
                systems: vec![
                    // A `by_quotas` system may not be the last, so
                    // `rest` applies otherwise after it.
                    HeightSystem {
                        id: "picked".to_string(),
                        label: "Picked".to_string(),
                        any_player: None,
                        quotas: by_id(&[("tall", 3)]),
                        by_quotas: true,
                        extra_height: None,
                    },
                    HeightSystem {
                        id: "rest".to_string(),
                        label: "Rest".to_string(),
                        any_player: None,
                        quotas: by_id(&[("tall", 8)]),
                        by_quotas: false,
                        extra_height: None,
                    },
                ],
            },
            universal: Universal {
                registered_position_a: false,
                fielded_at_a: Vec::new(),
                no_b_ratings: false,
                b_uses_a_allowance: false,
                gk_not_second_a: false,
                age: None,
                weight: None,
                captain_required: true,
                min_goalkeepers: 0,
                max_skill_cards: 10,
                max_com_styles: None,
            },
            specials: vec![Special {
                label: "A special".to_string(),
                when: captain_condition(Some(true)),
                then: Effects {
                    stat_bonus: BTreeMap::new(),
                    stat_target: BTreeMap::new(),
                    cards: None,
                    free: BTreeMap::new(),
                    free_cards: vec![card("captaincy")],
                    require_cards: Vec::new(),
                    a_positions: None,
                    max_a_positions: None,
                    form: None,
                    injury_resistance: None,
                    weak_foot: None,
                    height_allowance: None,
                    forbid: None,
                    choice: Vec::new(),
                    stat_warning: None,
                },
                severity: Severity::Warning,
            }],
            team_settings: Vec::new(),
            suggestions: vec!["cards".to_string()],
        }
    }

    /// The written block, pinned to the byte: the sections and their
    /// blank lines, one `#{ ... },` item wrapped under its first field,
    /// `universal` inside `#{ ... }`, the lists and the `};` ending.
    /// The two departures from declaration order show too: `severity`
    /// right after `label`, `by_quotas` before `quotas`. `Tier::form` 0
    /// is a default, so no `form:` is written; the block reads back
    /// equal.
    #[test]
    fn the_layout_is_pinned() {
        let ruleset = small_ruleset();
        const EXPECTED: &str = r##"// AATF ruleset. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "Small",
    squad_size: 11,

    tiers: [
        #{ id: "gold", label: "Gold", count: 3, rate: 99, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 6, free: #{ com: 1 } },
        #{ id: "regular", label: "Regular", rate: 77 },
    ],

    heights: #{
        brackets: [
            #{ id: "tall", label: "Tall", exactly: [185] },
        ],
        systems: [
            #{ id: "picked", label: "Picked", by_quotas: true, quotas: #{ tall: 3 } },
            #{ id: "rest", label: "Rest", quotas: #{ tall: 8 } },
        ],
    },

    universal: #{
        captain_required: true, max_skill_cards: 10,
    },

    specials: [
        #{ label: "A special", severity: "warning", when: #{ captain: true },
           then: #{ free_cards: ["captaincy"] } },
    ],

    suggestions: ["cards"],
};
"##;
        let written = ruleset.to_data_block();
        assert_eq!(written, EXPECTED);
        assert!(!written.contains("form:"), "a tier's `form` 0 is left out");
        assert!(
            !written.contains("error"),
            "`Severity::Error` is never spelled"
        );
        let reread =
            Ruleset::from_data_block(&written).unwrap_or_else(|error| panic!("{error}\n{written}"));
        assert_eq!(reread, ruleset);
        assert_eq!(reread.to_data_block(), written);
    }
}
