use super::*;

#[test]
fn choice_options_check_like_then() {
    // `free`'s per-tier map inside a `choice` option.
    case(
        "unknown tier in a choice option",
        &edit(
            AUTUMN26,
            "choice: [#{ free: #{ com: 1 } }, #{ a_positions: 1 }]",
            "choice: [#{ free: #{ com: #{ goldd: 1 } } }, #{ a_positions: 1 }]",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Tier,
            id: "goldd".to_string(),
        },
        "specials[11].then.choice[0].free.com.goldd",
    );
    // `height_allowance` inside a `choice` option names no bracket — and
    // an option may not carry it at all: both rules report.
    let err = Ruleset::from_data_block(&edit(
        AUTUMN26,
        "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"] },\n           then: #{ choice: [#{ free: #{ com: 1 } }",
        "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], brackets: [\"manlet\"] },\n           then: #{ choice: [#{ free: #{ com: 1 }, height_allowance: 4 }",
    ))
    .expect_err("allowance in a choice option");
    assert_eq!(
        err,
        LoadError::Rules(vec![
            RuleError {
                path: "specials[11].when.brackets[0]".to_string(),
                rule: LoadRule::AllowanceWithBracket,
            },
            RuleError {
                path: "specials[11].then.choice[0].height_allowance".to_string(),
                rule: LoadRule::ChoiceOptionEffect,
            },
        ]),
        "{err:?}"
    );
}

/// A `choice` option holds only the effects `Effects::choice`'s doc lists
/// (`stat_bonus`, `cards`, `free`, `free_cards`, `a_positions`,
/// `max_a_positions`, `weak_foot` — the ones an option can be found used
/// through): any other is refused at its field's path.
#[test]
fn a_choice_option_holding_other_effects_is_refused() {
    for (name, to, path) in [
        (
            "stat_target",
            "stat_target: #{ dribbling: 3 }",
            "specials[11].then.choice[0].stat_target",
        ),
        (
            "require_cards",
            "require_cards: [\"malicia\"]",
            "specials[11].then.choice[0].require_cards",
        ),
        ("form", "form: 1", "specials[11].then.choice[0].form"),
        (
            "injury_resistance",
            "injury_resistance: 1",
            "specials[11].then.choice[0].injury_resistance",
        ),
        (
            "height_allowance",
            "height_allowance: 4",
            "specials[11].then.choice[0].height_allowance",
        ),
        (
            "forbid",
            "forbid: \"x\"",
            "specials[11].then.choice[0].forbid",
        ),
        (
            "a nested choice",
            "choice: [#{ cards: 1 }]",
            "specials[11].then.choice[0].choice",
        ),
    ] {
        case(
            name,
            &edit(
                AUTUMN26,
                "choice: [#{ free: #{ com: 1 } }, #{ a_positions: 1 }]",
                &format!("choice: [#{{ free: #{{ com: 1 }}, {to} }}, #{{ a_positions: 1 }}]"),
            ),
            LoadRule::ChoiceOptionEffect,
            path,
        );
    }
    // Every effect the doc lists loads as an option.
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "choice: [#{ free: #{ com: 1 } }, #{ a_positions: 1 }]",
        "choice: [#{ stat_bonus: #{ all: 1 } }, #{ cards: 1 }, #{ free: #{ com: 1 } }, #{ free_cards: [\"malicia\"] }, #{ a_positions: 1 }, #{ max_a_positions: 3 }, #{ weak_foot: #{ usage: 4, accuracy: 4 } }]",
    ))
    .expect("the seven listed effects load as options");
}

#[test]
fn an_unknown_tier_in_a_specials_when_is_refused() {
    case(
        "unknown tier in a special's when",
        &edit(
            AUTUMN26,
            "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], positions: [\"GK\"] }",
            "when: #{ tiers: [\"gollum\"], positions: [\"GK\"] }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Tier,
            id: "gollum".to_string(),
        },
        "specials[2].when.tiers[0]",
    );
}

#[test]
fn a_tiers_when_not_naming_a_tier_is_refused() {
    case(
        "tier when.not names a tier",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"] }",
            "when: #{ not: #{ tiers: [\"gold\"] } }",
        ),
        LoadRule::TierConditionNamesTiers,
        "tiers[3].when.not.tiers",
    );
}

#[test]
fn a_tiers_when_naming_a_tier_is_refused() {
    case(
        "tier when names a tier",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"] }",
            "when: #{ tiers: [\"gold\"] }",
        ),
        LoadRule::TierConditionNamesTiers,
        "tiers[3].when.tiers",
    );
}

#[test]
fn a_system_condition_too_wide_is_refused() {
    case(
        "system condition too wide",
        &edit(
            AUTUMN26,
            "any_player: #{ height_min: 194 }",
            "any_player: #{ height_min: 194, tiers: [\"gold\"] }",
        ),
        LoadRule::SystemConditionTooWide,
        "heights.systems[0].any_player",
    );
}

#[test]
fn a_last_system_with_a_condition_is_refused() {
    case(
        "last system has a condition",
        &edit(
            AUTUMN26,
            "#{ id: \"red\", label: \"Red\",",
            "#{ id: \"red\", label: \"Red\", any_player: #{ height_min: 180 },",
        ),
        LoadRule::LastSystemHasCondition,
        "heights.systems[1].any_player",
    );
}

#[test]
fn an_empty_any_player_is_no_condition() {
    // `any_player: #{}` holds for every player: a non-last system with it
    // is `SystemWithoutCondition`, and the last one with it loads.
    case(
        "non-last system with an empty condition",
        &edit(
            AUTUMN26,
            "any_player: #{ height_min: 194 }",
            "any_player: #{}",
        ),
        LoadRule::SystemWithoutCondition,
        "heights.systems[0]",
    );
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "#{ id: \"red\", label: \"Red\",",
        "#{ id: \"red\", label: \"Red\", any_player: #{},",
    ))
    .expect("an empty condition on the last system is no condition");
}

#[test]
fn an_allowance_with_a_bracket_is_refused() {
    case(
        "allowance with bracket",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ positions: [\"GK\"], brackets: [\"manlet\"] }",
        ),
        LoadRule::AllowanceWithBracket,
        "specials[3].when.brackets[0]",
    );
}

#[test]
fn an_allowance_with_a_bracket_under_not_or_any_is_refused() {
    // `brackets` under `not` or `any` counts at any depth: brackets are
    // computed from the allowance, wherever the condition names them.
    case(
        "allowance with a bracket under not",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ positions: [\"GK\"], not: #{ brackets: [\"manlet\"] } }",
        ),
        LoadRule::AllowanceWithBracket,
        "specials[3].when.not.brackets[0]",
    );
    case(
        "allowance with a bracket under not.not",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ positions: [\"GK\"], not: #{ not: #{ brackets: [\"manlet\"] } } }",
        ),
        LoadRule::AllowanceWithBracket,
        "specials[3].when.not.not.brackets[0]",
    );
    case(
        "allowance with a bracket under any",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ positions: [\"GK\"], any: [#{ height_min: 189 }, #{ brackets: [\"manlet\"] }] }",
        ),
        LoadRule::AllowanceWithBracket,
        "specials[3].when.any[1].brackets[0]",
    );
    // A special without an allowance may name a bracket under `not`.
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], positions: [\"GK\"] }",
        "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], positions: [\"GK\"], not: #{ brackets: [\"manlet\"] } }",
    ))
    .expect("a bracket under not without an allowance loads");
}

#[test]
fn an_unknown_id_inside_not_is_refused() {
    // `not` is checked like the condition it sits in.
    case(
        "unknown id in not",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ positions: [\"GK\"], not: #{ systems: [\"greeen\"] } }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::System,
            id: "greeen".to_string(),
        },
        "specials[3].when.not.systems[0]",
    );
}

/// `any` entries are checked like the condition they sit in: the ids in
/// them, and a tier's `when` naming no tier inside one.
#[test]
fn ids_inside_any_are_checked() {
    case(
        "unknown id in any",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ positions: [\"GK\"], any: [#{ systems: [\"greeen\"] }] }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::System,
            id: "greeen".to_string(),
        },
        "specials[3].when.any[0].systems[0]",
    );
    case(
        "tier when.any names a tier",
        &edit(
            AUTUMN26,
            "when: #{ positions: [\"GK\"] }",
            "when: #{ positions: [\"GK\"], any: [#{ tiers: [\"gold\"] }] }",
        ),
        LoadRule::TierConditionNamesTiers,
        "tiers[3].when.any[0].tiers",
    );
}

#[test]
fn an_unknown_tier_in_an_effect_is_refused() {
    case(
        "unknown tier in stat_bonus",
        &edit(
            AUTUMN26,
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2, silverr: 1 } }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Tier,
            id: "silverr".to_string(),
        },
        "specials[9].then.choice[0].stat_bonus.all.silverr",
    );
}

/// A `when.brackets` entry naming a nonexistent bracket is `UnknownId`;
/// `bracket_has`/`==` at it.
#[test]
fn an_unknown_bracket_in_a_specials_condition_is_refused() {
    case(
        "unknown bracket",
        &edit(
            AUTUMN26,
            "when: #{ brackets: [\"manlet\"] }",
            "when: #{ brackets: [\"nope\"] }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Bracket,
            id: "nope".to_string(),
        },
        "specials[8].when.brackets[0]",
    );
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "when: #{ brackets: [\"manlet\"] }",
        "when: #{ brackets: [\"giant\"] }",
    ))
    .expect("a real bracket name loads");
}

/// Unknown tier ids inside a `then`'s per-tier maps are `UnknownId` at
/// their own key paths — every `PerTier` field walks it. The anchor is
/// "Green silver giants"' `then` (specials[10]): "Red manlet bonus"'s
/// `then` holds only a `choice`, whose options may not carry
/// `stat_target`, `form` or `injury_resistance`.
#[test]
fn an_unknown_tier_in_effects_maps_is_refused() {
    for (to, path) in [
        (
            "stat_target: #{ dribbling: #{ gollum: 3 } }",
            "specials[10].then.stat_target.dribbling.gollum",
        ),
        ("cards: #{ gollum: 3 }", "specials[10].then.cards.gollum"),
        (
            "a_positions: #{ gollum: 3 }",
            "specials[10].then.a_positions.gollum",
        ),
        (
            "free: #{ tricks: #{ gollum: 2 } }",
            "specials[10].then.free.tricks.gollum",
        ),
        ("form: #{ gollum: 1 }", "specials[10].then.form.gollum"),
        (
            "injury_resistance: #{ gollum: 1 }",
            "specials[10].then.injury_resistance.gollum",
        ),
    ] {
        case(
            to,
            &edit(AUTUMN26, "stat_bonus: #{ all: -3 }", to),
            LoadRule::UnknownId {
                kind: IdKind::Tier,
                id: "gollum".to_string(),
            },
            path,
        );
    }
    // A `free` map key is a group id, not a tier.
    case(
        "free group key",
        &edit(
            AUTUMN26,
            "stat_bonus: #{ all: -3 }",
            "free: #{ gollum: #{ regular: 2 } }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Group,
            id: "gollum".to_string(),
        },
        "specials[10].then.free.gollum",
    );
    // The same keys naming a real tier load.
    for to in [
        "stat_target: #{ dribbling: #{ regular: 3 } }",
        "cards: #{ regular: 3 }",
        "a_positions: #{ regular: 3 }",
        "free: #{ tricks: #{ regular: 2 } }",
        "form: #{ regular: 1 }",
        "injury_resistance: #{ regular: 1 }",
    ] {
        Ruleset::from_data_block(&edit(AUTUMN26, "stat_bonus: #{ all: -3 }", to))
            .expect("a real tier key loads");
    }
}

/// Every forbidden field in `any_player` is `SystemConditionTooWide`; the
/// three allowed ones (`height_min`, `height_max`, `positions`) together,
/// an `any` of them included, are not.
#[test]
fn any_player_is_wide_field_by_field() {
    let fields: [(&str, &str); 14] = [
        ("tiers", "tiers: [\"gold\"]"),
        ("systems", "systems: [\"red\"]"),
        ("brackets", "brackets: [\"manlet\"]"),
        ("captain", "captain: true"),
        ("playing_styles", "playing_styles: [\"none\"]"),
        ("holds", "holds: [\"malicia\"]"),
        ("rated_a_at", "rated_a_at: [\"CB\"]"),
        ("rated_b_at", "rated_b_at: [\"LB\"]"),
        ("name_colors", "name_colors: [\"cc9900ff\"]"),
        ("fielded_only_at", "fielded_only_at: \"CB\""),
        ("ranges", "ranges: #{ speed: [90, 99] }"),
        ("strong_foot", "strong_foot: \"right\""),
        ("any", "any: [#{ captain: true }]"),
        ("not", "not: #{ captain: true }"),
    ];
    for (name, snippet) in fields {
        let err = Ruleset::from_data_block(&edit(
            AUTUMN26,
            "any_player: #{ height_min: 194 }",
            &format!("any_player: #{{ height_min: 194, {snippet} }}"),
        ))
        .expect_err(name);
        assert!(
            matches!(
                err,
                LoadError::Rules(ref errors)
                    if errors
                        == &[RuleError {
                            path: "heights.systems[0].any_player".to_string(),
                            rule: LoadRule::SystemConditionTooWide,
                        }]
            ),
            "{name}: {err:?}"
        );
    }
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "any_player: #{ height_min: 194 }",
        "any_player: #{ height_min: 180, height_max: 199, positions: [\"GK\"] }",
    ))
    .expect("the three allowed fields together load");
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "any_player: #{ height_min: 194 }",
        "any_player: #{ any: [#{ height_min: 190 }, #{ positions: [\"GK\"] }] }",
    ))
    .expect("an `any` of heights and positions loads");
}
