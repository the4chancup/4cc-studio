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
mod tests;
