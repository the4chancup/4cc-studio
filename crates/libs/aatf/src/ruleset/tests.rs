use super::*;

/// The official data block, embedded.
const AUTUMN26: &str = include_str!("../../rulesets/autumn26.rhai");

/// `text` with `from` replaced once by `to` (asserted present first).
fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the snippet is in the file: {from}");
    text.replacen(from, to, 1)
}

#[test]
fn the_official_ruleset_loads() {
    let ruleset = Ruleset::from_data_block(AUTUMN26).expect("the official file loads");
    let tiers: Vec<&str> = ruleset.tiers.iter().map(|tier| tier.id.as_str()).collect();
    assert_eq!(tiers, ["gold", "silver", "bronze", "goalkeeper", "regular"]);
    assert_eq!(ruleset.tiers[4].count, None, "regular is the rest tier");
    assert_eq!(ruleset.tiers[3].counts_as.as_deref(), Some("regular"));
    // Green's quotas, in bracket order as written.
    let green = ruleset
        .heights
        .systems
        .iter()
        .find(|system| system.id == "green")
        .expect("the green system");
    let quotas: Vec<(&str, u8)> = green
        .quotas
        .iter()
        .map(|(id, count)| (id.as_str(), *count))
        .collect();
    // `BTreeMap` order is alphabetical, not the file's bracket order.
    assert_eq!(
        quotas,
        [
            ("giant", 6),
            ("giga", 0),
            ("manlet", 6),
            ("mid", 5),
            ("tall", 6)
        ]
    );
    assert_eq!(ruleset.specials.len(), 12);
    let last = ruleset.specials.last().expect("a last special");
    assert_eq!(last.then.choice.len(), 2);
    // "Red manlet bonus": `stat_bonus` is `all` keyed by four tiers.
    let bonus = ruleset
        .specials
        .iter()
        .find(|special| special.label == "Red manlet bonus")
        .expect("the manlet bonus");
    let Some(PerTier::ByTier(by_tier)) = bonus.then.stat_bonus.get(&StatOrAll::All) else {
        panic!("the bonus is not All-by-tier: {:?}", bonus.then.stat_bonus);
    };
    assert_eq!(by_tier.len(), 4);
    for (id, amount) in [
        ("regular", 5),
        ("goalkeeper", 5),
        ("bronze", 3),
        ("silver", 2),
    ] {
        assert_eq!(by_tier.get(id), Some(&amount), "{id}");
    }
}

// One failing file per load-time rule: `edit` of a unique snippet of
// the official text → `Rules` with exactly the expected error.
fn case(name: &str, broken: &str, rule: LoadRule, path: &str) {
    let err = Ruleset::from_data_block(broken).expect_err(name);
    match err {
        LoadError::Rules(errors) => {
            assert_eq!(
                errors,
                vec![RuleError {
                    path: path.to_string(),
                    rule,
                }],
                "{name}"
            );
        }
        other => panic!("{name}: expected Rules, got {other:?}"),
    }
}

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
        "#{ id: \"giga\",   label: \"Giga\",   exactly: [199] },\n            #{ id: \"manlet\", label: \"Second\", up_to: 170 },",
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

// -------------------------------------------------------------------
// Schema and parse errors

/// `Ruleset` deserialization errors land as `Schema` with rhai's text.
fn schema(text: &str) -> LoadError {
    Ruleset::from_data_block(&format!("const RULESET = {text};")).expect_err("a schema error")
}

#[test]
fn an_unknown_top_level_key_is_schema() {
    let err = schema(
        "#{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, frob: 1 }",
    );
    assert!(matches!(err, LoadError::Schema { .. }), "{err:?}");
}

#[test]
fn an_unknown_tier_key_is_schema() {
    let err = schema(
        "#{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"gold\", label: \"Gold\", rate: 99, frob: 1 }] }",
    );
    assert!(matches!(err, LoadError::Schema { .. }), "{err:?}");
}

#[test]
fn a_missing_rate_is_schema() {
    let err = schema(
        "#{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"gold\", label: \"Gold\" }] }",
    );
    assert!(matches!(err, LoadError::Schema { .. }), "{err:?}");
}

#[test]
fn an_unknown_position_label_is_schema() {
    let err = schema(
        "#{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"gold\", label: \"Gold\", rate: 99, when: #{ positions: [\"GKK\"] } }] }",
    );
    assert!(
        matches!(err, LoadError::Schema { ref message, .. } if message.contains("GKK")),
        "{err:?}"
    );
}

#[test]
fn a_bracket_with_both_or_neither_height_form_is_schema() {
    for heights in ["up_to: 175, exactly: [185]", ""] {
        let err = schema(&format!(
            "#{{ schema: 1, name: \"x\", squad_size: 23, universal: #{{ max_skill_cards: 10 }}, heights: #{{ brackets: [#{{ id: \"x\", label: \"x\", {heights} }}], systems: [] }} }}"
        ));
        // `RawBracket`'s error names both keys, whichever half is wrong.
        assert!(
            matches!(
                err,
                LoadError::Schema { ref message, .. }
                    if message.contains("up_to") && message.contains("exactly")
            ),
            "{heights}: {err:?}"
        );
    }
}

#[test]
fn per_tier_deserializes_in_both_forms() {
    let ruleset = Ruleset::from_data_block(
        "const RULESET = #{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77 }], specials: [#{ label: \"x\", then: #{ stat_bonus: #{ all: -3, dribbling: #{ regular: 5 } } } }] };",
    )
    .expect("both forms parse");
    let Some(&PerTier::All(-3)) = ruleset.specials[0].then.stat_bonus.get(&StatOrAll::All) else {
        panic!("all is not All");
    };
    let Some(PerTier::ByTier(map)) = ruleset.specials[0].then.stat_bonus.get(&StatOrAll::Stat(
        Stat::try_from("dribbling".to_string()).expect("a stat"),
    )) else {
        panic!("the stat is not ByTier");
    };
    assert_eq!(map.get("regular"), Some(&5));
}

/// A tier's `stats` names ability stats only: `form` is the tier's own
/// `form` key, not a stat — `stats: #{ form: 8 }` is `Schema` naming
/// `"form"`.
#[test]
fn a_form_in_a_stats_map_is_schema() {
    let err = schema(
        "#{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"gold\", label: \"Gold\", rate: 99, stats: #{ form: 8 } }] }",
    );
    assert!(
        matches!(err, LoadError::Schema { ref message, .. } if message.contains("\"form\"")),
        "{err:?}"
    );
}

/// Every required key absent is `Schema` naming it: a violation shows
/// the special's label, and an absent `universal` would default
/// `max_skill_cards` to 0.
#[test]
fn a_required_key_absent_is_schema() {
    for (name, text, key) in [
        ("a tier's id", edit(AUTUMN26, "id: \"gold\", ", ""), "id"),
        ("a tier's label", edit(AUTUMN26, "label: \"Gold\", ", ""), "label"),
        ("a bracket's id", edit(AUTUMN26, "id: \"manlet\", ", ""), "id"),
        ("a bracket's label", edit(AUTUMN26, "label: \"Manlet\", ", ""), "label"),
        ("a system's id", edit(AUTUMN26, "id: \"green\", ", ""), "id"),
        ("a system's label", edit(AUTUMN26, "label: \"Green\", ", ""), "label"),
        (
            "universal",
            "const RULESET = #{ schema: 1, name: \"x\", squad_size: 23 };".to_string(),
            "universal",
        ),
        (
            "universal.max_skill_cards",
            "const RULESET = #{ schema: 1, name: \"x\", squad_size: 23, universal: #{ } };"
                .to_string(),
            "max_skill_cards",
        ),
        (
            "a special's label",
            edit(AUTUMN26, "label: \"Malicia is a free card\", ", ""),
            "label",
        ),
        (
            "a team settings entry's label",
            "const RULESET = #{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, team_settings: [#{ }] };"
                .to_string(),
            "label",
        ),
    ] {
        let err = Ruleset::from_data_block(&text).expect_err(name);
        assert!(
            matches!(err, LoadError::Schema { ref message, .. } if message.contains(key)),
            "{name}: {err:?}"
        );
    }
}

#[test]
fn schema_two_is_unknown_schema() {
    let err =
        schema("#{ schema: 2, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 } }");
    assert_eq!(err, LoadError::UnknownSchema { schema: 2 }, "{err:?}");
}

#[test]
fn no_ruleset_is_no_ruleset() {
    assert_eq!(
        Ruleset::from_data_block("const OTHER = 1;").expect_err("no map"),
        LoadError::NoRuleset
    );
    assert_eq!(
        Ruleset::from_data_block("const RULESET = 1;").expect_err("not a map"),
        LoadError::NoRuleset
    );
}

/// The bare engine refuses everything but the data itself: `import` has
/// no resolver and `print` does not exist.
#[test]
fn import_and_print_are_refused() {
    let err = Ruleset::from_data_block("import \"x\" as m;\nconst RULESET = #{ schema: 1 };")
        .expect_err("import has no resolver");
    assert!(matches!(err, LoadError::Parse { .. }), "{err:?}");
    let err = Ruleset::from_data_block("print(\"x\");\nconst RULESET = #{ schema: 1 };")
        .expect_err("print is not a function");
    assert!(matches!(err, LoadError::Parse { .. }), "{err:?}");
}

#[test]
fn a_stray_token_is_parse_with_its_position() {
    let err = Ruleset::from_data_block("const A = 1;\nconst B = 2;\nhello\nconst RULESET = #{};")
        .expect_err("a stray token is a parse error");
    match err {
        LoadError::Parse { line, column, .. } => {
            // rhai reports the error at the `const` following the stray
            // `hello`: line 4, column 1.
            assert_eq!((line, column), (4, 1), "line 4 column 1");
        }
        other => panic!("{other:?}"),
    }
}

/// Every entry of each closed-set table deserializes to a value whose
/// `label()` is that entry — killing the `label` and `==` mutants the
/// first-entry tests missed.
#[test]
fn every_label_round_trips() {
    for name in ABILITY_STATS {
        assert_eq!(
            Stat::try_from(name.to_string()).expect(name).label(),
            name,
            "{name}"
        );
    }
    for name in POSITIONS {
        assert_eq!(
            Position::try_from(name.to_string()).expect(name).label(),
            name,
            "{name}"
        );
    }
    for name in PLAY_STYLES {
        assert_eq!(
            PlayStyle::try_from(name.to_string()).expect(name).label(),
            name,
            "{name}"
        );
    }
    for name in SKILLS {
        assert_eq!(
            Card::try_from(name.to_string()).expect(name).label(),
            name,
            "{name}"
        );
    }
    for name in COM_STYLES {
        assert_eq!(
            Card::try_from(name.to_string()).expect(name).label(),
            name,
            "{name}"
        );
    }
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
}
