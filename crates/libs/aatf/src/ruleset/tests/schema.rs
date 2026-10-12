use super::*;

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
fn unknown_keys_and_labels_inside_specials_are_schema() {
    for (from, to, name) in [
        (
            "then: #{ forbid: \"Medals cannot play as GK\" }",
            "then: #{ forbid: \"Medals cannot play as GK\", free_card: 1 }",
            "free_card",
        ),
        (
            "when: #{ positions: [\"GK\"], height_min: 189, height_max: 189 }",
            "when: #{ postions: [\"GK\"], height_min: 189, height_max: 189 }",
            "postions",
        ),
        (
            "free_cards: [\"malicia\"]",
            "free_cards: [\"malicias\"]",
            "malicias",
        ),
        (
            "when: #{ captain: true }",
            "when: #{ captain: true, playing_styles: [\"nope\"] }",
            "nope",
        ),
    ] {
        let err = Ruleset::from_data_block(&edit(AUTUMN26, from, to)).expect_err(name);
        assert!(
            matches!(err, LoadError::Schema { ref message, .. } if message.contains(name)),
            "{name}: {err:?}"
        );
    }
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
fn an_empty_exactly_is_schema() {
    let err = schema(
        "#{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, heights: #{ brackets: [#{ id: \"x\", label: \"x\", exactly: [] }], systems: [] } }",
    );
    assert!(
        matches!(
            err,
            LoadError::Schema { ref message, .. } if message.contains("`exactly` lists no height")
        ),
        "{err:?}"
    );
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
        (
            "schema",
            "const RULESET = #{ name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 } };"
                .to_string(),
            "schema",
        ),
        (
            "name",
            "const RULESET = #{ schema: 1, squad_size: 23, universal: #{ max_skill_cards: 10 } };"
                .to_string(),
            "name",
        ),
        (
            "squad_size",
            "const RULESET = #{ schema: 1, name: \"x\", universal: #{ max_skill_cards: 10 } };"
                .to_string(),
            "squad_size",
        ),
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
        (
            "weight.floor",
            edit(AUTUMN26, "floor: 30, ", ""),
            "floor",
        ),
        (
            "weight.below",
            edit(AUTUMN26, "below: 129, ", ""),
            "below",
        ),
        (
            "weight.above",
            edit(AUTUMN26, ", above: 81", ""),
            "above",
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

/// A `let` is not a constant: a function could not read
/// `global::RULESET`, so the entry is `NoRuleset` too.
#[test]
fn a_let_ruleset_is_no_ruleset() {
    assert_eq!(
        Ruleset::from_data_block(
            "let RULESET = #{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77 }] };"
        )
        .expect_err("a let"),
        LoadError::NoRuleset
    );
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

/// `import` has no resolver on the bare engine; `print` exists under the
/// standard package and a data block containing it loads.
#[test]
fn import_is_refused_but_print_runs() {
    let err = Ruleset::from_data_block("import \"x\" as m;\nconst RULESET = #{ schema: 1 };")
        .expect_err("import has no resolver");
    assert!(matches!(err, LoadError::Parse { .. }), "{err:?}");
    Ruleset::from_data_block(
        "print(\"x\");\nconst RULESET = #{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77 }] };",
    )
    .expect("the standard package's print runs before the data block");
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
