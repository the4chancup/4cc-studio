use super::*;

#[test]
fn a_duplicate_tier_id_is_refused() {
    // A second `regular` appended: every reference still resolves, so
    // the only error is the duplicate.
    let text = AUTUMN26.replacen(
        "#{ id: \"regular\", label: \"Regular\", rate: 77, form: 4, injury_resistance: 1,\n           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 3, free: #{ tricks: 2 }, a_positions: 1 },",
        "#{ id: \"regular\", label: \"Regular\", rate: 77, form: 4, injury_resistance: 1,\n           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 3, free: #{ tricks: 2 }, a_positions: 1 },\n        #{ id: \"regular\", label: \"Second\", count: 1, rate: 70, form: 4, injury_resistance: 1,\n           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 1, free: #{}, a_positions: 1 },",
        1,
    );
    case(
        "duplicate tier",
        &text,
        LoadRule::DuplicateId {
            kind: IdKind::Tier,
            id: "regular".to_string(),
        },
        "tiers[5]",
    );
}

#[test]
fn a_duplicate_bracket_id_is_refused() {
    // A second `manlet` appended; `mid`'s quota keys still resolve.
    let text = AUTUMN26.replacen(
        "#{ id: \"giga\",   label: \"Giga\",   exactly: [199] },",
        "#{ id: \"giga\",   label: \"Giga\",   exactly: [199] },\n            #{ id: \"manlet\", label: \"Second\", up_to: 210 },",
        1,
    );
    case(
        "duplicate bracket",
        &text,
        LoadRule::DuplicateId {
            kind: IdKind::Bracket,
            id: "manlet".to_string(),
        },
        "heights.brackets[5]",
    );
}

#[test]
fn a_duplicate_system_id_is_refused() {
    // A second `green` inserted before `red` and carrying `any_player`:
    // `red` stays the last, the duplicate is the only error.
    let text = AUTUMN26.replacen(
        "#{ id: \"red\", label: \"Red\",",
        "#{ id: \"green\", label: \"Second\", any_player: #{ height_min: 180 } },\n            #{ id: \"red\", label: \"Red\",",
        1,
    );
    case(
        "duplicate system",
        &text,
        LoadRule::DuplicateId {
            kind: IdKind::System,
            id: "green".to_string(),
        },
        "heights.systems[1]",
    );
}

#[test]
fn an_unknown_counts_as_is_refused() {
    case(
        "unknown counts_as",
        &edit(
            AUTUMN26,
            "\"regular\", when: #{ positions: [\"GK\"] }",
            "\"gollum\", when: #{ positions: [\"GK\"] }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Tier,
            id: "gollum".to_string(),
        },
        "tiers[3].counts_as",
    );
}

#[test]
fn an_unknown_free_group_is_refused() {
    case(
        "unknown free group",
        &edit(
            AUTUMN26,
            "free: #{ tricks: 3, com: 2 }",
            "free: #{ trickz: 3, com: 2 }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Group,
            id: "trickz".to_string(),
        },
        "tiers[0].free.trickz",
    );
}

#[test]
fn an_unknown_quota_bracket_is_refused() {
    case(
        "unknown quota bracket",
        &edit(
            AUTUMN26,
            "quotas: #{ giga: 0, giant: 6",
            "quotas: #{ giga: 0, gaint: 6",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Bracket,
            id: "gaint".to_string(),
        },
        "heights.systems[0].quotas.gaint",
    );
}

#[test]
fn an_unknown_suggestion_is_refused() {
    case(
        "unknown suggestion",
        &edit(
            AUTUMN26,
            "\"weak_foot\", \"injury_resistance\"",
            "\"weak_foot\", \"injury_resistans\"",
        ),
        LoadRule::UnknownSuggestion {
            entry: "injury_resistans".to_string(),
        },
        "suggestions[4]",
    );
}

#[test]
fn an_unknown_at_most_target_is_refused() {
    case(
        "unknown at_most target",
        &edit(
            AUTUMN26,
            "\"attacking_prowess\", \"defensive_prowess\"",
            "\"attaking_prowess\", \"defensive_prowess\"",
        ),
        LoadRule::UnknownTarget {
            name: "attaking_prowess".to_string(),
        },
        "stats.at_most[0]",
    );
}

#[test]
fn the_com_card_group_is_reserved() {
    // `com` inside `card_groups` next to `tricks`: `free: com` and the
    // `com` suggestion still resolve, so only ReservedGroup fires.
    case(
        "reserved com group",
        &edit(
            AUTUMN26,
            "card_groups: #{\n",
            "card_groups: #{\n        com: [\"captaincy\"],\n",
        ),
        LoadRule::ReservedGroup,
        "card_groups.com",
    );
}

#[test]
fn a_count_with_counts_as_is_refused() {
    case(
        "count with counts_as",
        &edit(
            AUTUMN26,
            "#{ id: \"goalkeeper\", label: \"Goalkeeper\", counts_as",
            "#{ id: \"goalkeeper\", label: \"Goalkeeper\", count: 1, counts_as",
        ),
        LoadRule::CountWithCountsAs,
        "tiers[3]",
    );
}

#[test]
fn two_rest_tiers_are_refused() {
    // Silver loses its `count`: silver and regular are both rest tiers.
    case(
        "two rest tiers",
        &edit(
            AUTUMN26,
            "#{ id: \"silver\", label: \"Silver\", count: 2, rate",
            "#{ id: \"silver\", label: \"Silver\", rate",
        ),
        LoadRule::MoreThanOneRestTier,
        "tiers",
    );
}

#[test]
fn a_counts_as_chain_is_refused() {
    // `goalkeeper` naming itself: the tier `counts_as` has a `counts_as`
    // of its own.
    case(
        "counts_as chain",
        &edit(
            AUTUMN26,
            "counts_as: \"regular\"",
            "counts_as: \"goalkeeper\"",
        ),
        LoadRule::CountsAsChain,
        "tiers[3].counts_as",
    );
}

#[test]
fn counts_short_of_the_squad_are_refused() {
    // `regular` gets a `count`: no rest tier, and the sum is under
    // `squad_size`.
    case(
        "counts short of squad",
        &edit(
            AUTUMN26,
            "#{ id: \"regular\", label: \"Regular\", rate: 77",
            "#{ id: \"regular\", label: \"Regular\", count: 3, rate: 77",
        ),
        LoadRule::CountsShortOfSquad {
            total: 8,
            squad_size: 23,
        },
        "tiers",
    );
}

#[test]
fn counts_equal_to_the_squad_with_no_rest_tier_loads() {
    // `regular` given `count: 18` brings the counts to `squad_size` with
    // no rest tier: 1 + 2 + 2 + 18 = 23.
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "#{ id: \"regular\", label: \"Regular\", rate: 77",
        "#{ id: \"regular\", label: \"Regular\", count: 18, rate: 77",
    ))
    .expect("counts equal to squad_size load");
}

#[test]
fn a_system_other_than_the_last_without_a_condition_is_refused() {
    // Green (system 0 of 2) loses its `any_player`.
    case(
        "system without condition",
        &edit(AUTUMN26, "any_player: #{ height_min: 194 },\n", ""),
        LoadRule::SystemWithoutCondition,
        "heights.systems[0]",
    );
}

#[test]
fn empty_strings_are_refused() {
    for (name, broken, path) in [
        (
            "empty name",
            edit(AUTUMN26, "name: \"Autumn 26\"", "name: \"\""),
            "name",
        ),
        (
            "empty tier label",
            edit(AUTUMN26, "label: \"Silver\"", "label: \"\""),
            "tiers[1].label",
        ),
        (
            "empty special label",
            edit(AUTUMN26, "label: \"Malicia is a free card\"", "label: \"\""),
            "specials[0].label",
        ),
    ] {
        case(name, &broken, LoadRule::Empty, path);
    }
}

#[test]
fn empty_strings_at_the_untested_sites_are_refused() {
    for (name, broken, path) in [
        (
            "empty bracket label",
            edit(AUTUMN26, "label: \"Tall\"", "label: \"\""),
            "heights.brackets[2].label",
        ),
        (
            "empty system label",
            edit(AUTUMN26, "label: \"Green\"", "label: \"\""),
            "heights.systems[0].label",
        ),
        (
            "empty team settings label",
            edit(
                AUTUMN26,
                "    suggestions:",
                "    team_settings: [#{ label: \"\" }],\n    suggestions:",
            ),
            "team_settings[0].label",
        ),
        (
            "empty forbid in then",
            edit(
                AUTUMN26,
                "forbid: \"Medals cannot play as GK\"",
                "forbid: \"\"",
            ),
            "specials[2].then.forbid",
        ),
        (
            "empty forbid in a choice option",
            edit(
                AUTUMN26,
                "#{ free: #{ com: 1 } }, #{ a_positions: 1 }",
                "#{ free: #{ com: 1 }, forbid: \"\" }, #{ a_positions: 1 }",
            ),
            "specials[11].then.choice[0].forbid",
        ),
    ] {
        case(name, &broken, LoadRule::Empty, path);
    }
}

#[test]
fn an_empty_tier_id_makes_its_names_unknown() {
    // `gold` emptied: `Empty` at the tier, then `UnknownId` where every
    // `when` names it.
    let err = Ruleset::from_data_block(&edit(AUTUMN26, "id: \"gold\",", "id: \"\","))
        .expect_err("an emptied tier id");
    let unknown = || RuleError {
        path: String::new(),
        rule: LoadRule::UnknownId {
            kind: IdKind::Tier,
            id: "gold".to_string(),
        },
    };
    let mut e2 = unknown();
    e2.path = "specials[2].when.tiers[0]".to_string();
    let mut e6 = unknown();
    e6.path = "specials[6].when.tiers[0]".to_string();
    let mut e11 = unknown();
    e11.path = "specials[11].when.tiers[0]".to_string();
    assert_eq!(
        err,
        LoadError::Rules(vec![
            RuleError {
                path: "tiers[0].id".to_string(),
                rule: LoadRule::Empty,
            },
            e2,
            e6,
            e11,
        ]),
        "{err:?}"
    );
}

#[test]
fn an_empty_system_id_makes_its_names_unknown() {
    // `green` emptied: `Empty` at the system, then `UnknownId` where
    // `when` names it.
    let err = Ruleset::from_data_block(&edit(AUTUMN26, "id: \"green\",", "id: \"\","))
        .expect_err("an emptied system id");
    assert_eq!(
        err,
        LoadError::Rules(vec![
            RuleError {
                path: "heights.systems[0].id".to_string(),
                rule: LoadRule::Empty,
            },
            RuleError {
                path: "specials[10].when.systems[0]".to_string(),
                rule: LoadRule::UnknownId {
                    kind: IdKind::System,
                    id: "green".to_string(),
                },
            },
        ]),
        "{err:?}"
    );
}

#[test]
fn an_empty_card_groups_key_makes_its_names_unknown() {
    // `tricks` emptied: the tiers' `free` keys name it, the group is
    // gone, and `suggestions`' entry is unknown — every error in order.
    let err = Ruleset::from_data_block(&edit(
        AUTUMN26,
        "        tricks: [\"scissors_feint\"",
        "        \"\": [\"scissors_feint\"",
    ))
    .expect_err("an emptied group key");
    let unknown = |path: &str| RuleError {
        path: path.to_string(),
        rule: LoadRule::UnknownId {
            kind: IdKind::Group,
            id: "tricks".to_string(),
        },
    };
    assert_eq!(
        err,
        LoadError::Rules(vec![
            unknown("tiers[0].free.tricks"),
            unknown("tiers[1].free.tricks"),
            unknown("tiers[2].free.tricks"),
            unknown("tiers[4].free.tricks"),
            RuleError {
                path: "card_groups".to_string(),
                rule: LoadRule::Empty,
            },
            RuleError {
                path: "suggestions[1]".to_string(),
                rule: LoadRule::UnknownSuggestion {
                    entry: "tricks".to_string(),
                },
            },
        ]),
        "{err:?}"
    );
}

#[test]
fn an_empty_bracket_id_makes_its_quotas_unknown() {
    // `tall` emptied: `Empty` at its bracket, then every system's quota
    // naming it is `UnknownId`.
    let err = Ruleset::from_data_block(&edit(
        AUTUMN26,
        "#{ id: \"tall\",   label: \"Tall\",   exactly: [185] },",
        "#{ id: \"\",   label: \"Tall\",   exactly: [185] },",
    ))
    .expect_err("an emptied bracket id");
    assert_eq!(
        err,
        LoadError::Rules(vec![
            RuleError {
                path: "heights.brackets[2].id".to_string(),
                rule: LoadRule::Empty,
            },
            RuleError {
                path: "heights.systems[0].quotas.tall".to_string(),
                rule: LoadRule::UnknownId {
                    kind: IdKind::Bracket,
                    id: "tall".to_string(),
                },
            },
            RuleError {
                path: "heights.systems[1].quotas.tall".to_string(),
                rule: LoadRule::UnknownId {
                    kind: IdKind::Bracket,
                    id: "tall".to_string(),
                },
            },
        ]),
        "{err:?}"
    );
}

#[test]
fn brackets_out_of_order_are_refused() {
    // `manlet` and `mid` swapped: `up_to: 175` now sits under 180. The
    // quotas still resolve (`manlet` and `mid` both exist), so the only
    // error is the order.
    case(
        "brackets swapped",
        &AUTUMN26.replacen(
            "#{ id: \"manlet\", label: \"Manlet\", up_to: 175 },\n            #{ id: \"mid\",    label: \"Mid\",    up_to: 180 },",
            "#{ id: \"mid\",    label: \"Mid\",    up_to: 180 },\n            #{ id: \"manlet\", label: \"Manlet\", up_to: 175 },",
            1,
        ),
        LoadRule::BracketsNotAscending,
        "heights.brackets[1]",
    );
    // `tall` at 180, not above `mid`'s 180.
    case(
        "exactly equal to the previous",
        &edit(AUTUMN26, "exactly: [185]", "exactly: [180]"),
        LoadRule::BracketsNotAscending,
        "heights.brackets[2]",
    );
    // Unsorted but all above 180 and below 194: loads.
    Ruleset::from_data_block(&edit(AUTUMN26, "exactly: [185]", "exactly: [186, 184]"))
        .expect("an unsorted exactly in range loads");
}

#[test]
fn color_code_lengths_are_refused() {
    // Seven bytes: not a code.
    case(
        "name_colors short",
        &edit(
            AUTUMN26,
            "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], positions: [\"GK\"] }",
            "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], positions: [\"GK\"], name_colors: [\"cc9900f\"] }",
        ),
        LoadRule::ColorCodeLength,
        "specials[2].when.name_colors[0]",
    );
    // An empty entry inside `not` is this rule, not `Empty`.
    case(
        "empty entry inside not",
        &edit(
            AUTUMN26,
            "when: #{ captain: true }",
            "when: #{ not: #{ name_colors: [\"\"] } }",
        ),
        LoadRule::ColorCodeLength,
        "specials[1].when.not.name_colors[0]",
    );
    // Nine bytes: over is refused too; eight loads.
    case(
        "name_colors long",
        &edit(
            AUTUMN26,
            "when: #{ captain: true }",
            "when: #{ captain: true, name_colors: [\"cc9900ff0\"] }",
        ),
        LoadRule::ColorCodeLength,
        "specials[1].when.name_colors[0]",
    );
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "when: #{ captain: true }",
        "when: #{ captain: true, name_colors: [\"cc9900ff\"] }",
    ))
    .expect("an eight-byte code loads");
}

#[test]
fn a_suggestion_word_as_a_group_is_reserved() {
    for word in [
        "cards",
        "a_positions",
        "weak_foot",
        "injury_resistance",
        "height",
        "free_cards",
        "choices",
    ] {
        case(
            word,
            &edit(
                AUTUMN26,
                "    card_groups: #{\n",
                &format!("    card_groups: #{{\n        {word}: [\"malicia\"],\n"),
            ),
            LoadRule::ReservedGroup,
            &format!("card_groups.{word}"),
        );
    }
}

#[test]
fn giga_first_fails_every_later_bracket() {
    // `giga` first: every bracket after it sits under its 199 — a check
    // against only the immediate predecessor would pass `[2]` to `[4]`.
    let err = Ruleset::from_data_block(&AUTUMN26.replacen(
        "#{ id: \"manlet\", label: \"Manlet\", up_to: 175 },\n            #{ id: \"mid\",    label: \"Mid\",    up_to: 180 },\n            #{ id: \"tall\",   label: \"Tall\",   exactly: [185] },\n            #{ id: \"giant\",  label: \"Giant\",  exactly: [194] },\n            #{ id: \"giga\",   label: \"Giga\",   exactly: [199] },",
        "#{ id: \"giga\",   label: \"Giga\",   exactly: [199] },\n            #{ id: \"manlet\", label: \"Manlet\", up_to: 175 },\n            #{ id: \"mid\",    label: \"Mid\",    up_to: 180 },\n            #{ id: \"tall\",   label: \"Tall\",   exactly: [185] },\n            #{ id: \"giant\",  label: \"Giant\",  exactly: [194] },",
        1,
    ))
    .expect_err("giga first");
    assert_eq!(
        err,
        LoadError::Rules(
            (1..=4)
                .map(|i| RuleError {
                    path: format!("heights.brackets[{i}]"),
                    rule: LoadRule::BracketsNotAscending,
                })
                .collect()
        ),
        "{err:?}"
    );
}

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
    // `height_allowance` inside a `choice` option names no bracket.
    case(
        "allowance in a choice option",
        &edit(
            AUTUMN26,
            "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"] },\n           then: #{ choice: [#{ free: #{ com: 1 } }",
            "when: #{ tiers: [\"gold\", \"silver\", \"bronze\"], brackets: [\"manlet\"] },\n           then: #{ choice: [#{ free: #{ com: 1 }, height_allowance: 4 }",
        ),
        LoadRule::AllowanceWithBracket,
        "specials[11].when.brackets[0]",
    );
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
fn counts_over_the_squad_are_refused() {
    case(
        "counts exceed squad",
        &edit(AUTUMN26, "squad_size: 23", "squad_size: 4"),
        LoadRule::CountsExceedSquad {
            total: 5,
            squad_size: 4,
        },
        "tiers",
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
fn a_preset_out_of_range_is_refused() {
    case(
        "preset out of range",
        &edit(
            AUTUMN26,
            "registered_position_a: true",
            "registered_position_a: true, fielded_at_a: [4]",
        ),
        LoadRule::PresetOutOfRange { preset: 4 },
        "universal.fielded_at_a[0]",
    );
}

#[test]
fn a_shape_not_ten_is_refused() {
    case(
        "shape not ten",
        &edit(
            AUTUMN26,
            "suggestions:",
            "team_settings: [#{ label: \"no 3-4-4\", forbidden_shapes: [[3, 4, 4]] }],\n    suggestions:",
        ),
        LoadRule::ShapeNotTen { shape: [3, 4, 4] },
        "team_settings[0].forbidden_shapes[0]",
    );
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
        "specials[9].then.stat_bonus.all.silverr",
    );
}

/// Counts summing to exactly `squad_size` load — `>` at the boundary.
#[test]
fn counts_at_the_squad_size_load() {
    let ruleset = Ruleset::from_data_block(&edit(AUTUMN26, "squad_size: 23", "squad_size: 5"))
        .expect("counts at the boundary load");
    assert_eq!(ruleset.squad_size, 5);
}

/// A `forbidden_shapes` entry summing to 10 loads — the `+` mutants would
/// refuse it.
#[test]
fn a_forbidden_shape_of_ten_loads() {
    Ruleset::from_data_block(&edit(
        AUTUMN26,
        "suggestions:",
        "team_settings: [#{ label: \"no 3-4-3\", forbidden_shapes: [[3, 4, 3]] }],\n    suggestions:",
    ))
    .expect("a shape adding to ten loads");
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
/// their own key paths — `stat_target`, `cards` and `free` all walk it.
#[test]
fn an_unknown_tier_in_effects_maps_is_refused() {
    for (from, to, path) in [
        (
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            "stat_target: #{ dribbling: #{ gollum: 3 } }",
            "specials[9].then.stat_target.dribbling.gollum",
        ),
        (
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            "cards: #{ gollum: 3 }",
            "specials[9].then.cards.gollum",
        ),
        (
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            "a_positions: #{ gollum: 3 }",
            "specials[9].then.a_positions.gollum",
        ),
        (
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            "free: #{ tricks: #{ gollum: 2 } }",
            "specials[9].then.free.tricks.gollum",
        ),
    ] {
        case(
            from,
            &edit(AUTUMN26, from, to),
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
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            "free: #{ gollum: #{ regular: 2 } }",
        ),
        LoadRule::UnknownId {
            kind: IdKind::Group,
            id: "gollum".to_string(),
        },
        "specials[9].then.free.gollum",
    );
    // The same keys naming a real tier load.
    for to in [
        "stat_target: #{ dribbling: #{ regular: 3 } }",
        "cards: #{ regular: 3 }",
        "a_positions: #{ regular: 3 }",
        "free: #{ tricks: #{ regular: 2 } }",
    ] {
        Ruleset::from_data_block(&edit(
            AUTUMN26,
            "stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } }",
            to,
        ))
        .expect("a real tier key loads");
    }
}

/// Every forbidden field in `any_player` is `SystemConditionTooWide`; the
/// three allowed ones (`height_min`, `height_max`, `positions`) together
/// are not.
#[test]
fn any_player_is_wide_field_by_field() {
    let fields: [(&str, &str); 11] = [
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
}

/// `LoadError::Rules` displays each `RuleError` as `path: message`,
/// joined by `"; "`.
#[test]
fn rules_error_texts_name_path_and_message() {
    let err = LoadError::Rules(vec![
        RuleError {
            path: "tiers[1]".to_string(),
            rule: LoadRule::DuplicateId {
                kind: IdKind::Tier,
                id: "gold".to_string(),
            },
        },
        RuleError {
            path: "team_settings[0].forbidden_shapes[0]".to_string(),
            rule: LoadRule::ShapeNotTen { shape: [3, 4, 4] },
        },
    ]);
    assert_eq!(
        err.to_string(),
        "tiers[1]: tier id \"gold\" is used twice; \
         team_settings[0].forbidden_shapes[0]: shape [3, 4, 4] does not add up to 10"
    );
    for (kind, expected) in [
        (IdKind::Tier, "tier id"),
        (IdKind::Bracket, "bracket id"),
        (IdKind::System, "system id"),
        (IdKind::Group, "group id"),
    ] {
        let err = LoadRule::UnknownId {
            kind,
            id: "x".to_string(),
        }
        .to_string();
        assert!(err.contains(expected), "{expected}: {err}");
    }
    for (rule, expected) in [
        (LoadRule::CountsAsChain, "counts_as"),
        (LoadRule::SystemWithoutCondition, "any_player"),
        (LoadRule::Empty, "empty"),
        (LoadRule::BracketsNotAscending, "bracket"),
        (LoadRule::ColorCodeLength, "eight"),
    ] {
        let text = rule.to_string();
        assert!(text.contains(expected), "{rule:?}: {text}");
    }
    // `ReservedGroup`'s one text names both reasons.
    let text = LoadRule::ReservedGroup.to_string();
    assert!(
        text.contains("com") && text.contains("suggestions"),
        "{text}"
    );
    let text = LoadRule::CountsShortOfSquad {
        total: 8,
        squad_size: 23,
    }
    .to_string();
    let total_at = text.find('8').expect("the total in the text");
    let size_at = text.find("23").expect("the squad size in the text");
    assert!(total_at < size_at, "{text}");
}
