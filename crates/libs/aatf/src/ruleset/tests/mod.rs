use super::*;

mod rules;
mod schema;

/// The official data block, embedded.
const AUTUMN26: &str = include_str!("../../../rulesets/autumn26.rhai");

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
/// `Ruleset` deserialization errors land as `Schema` with rhai's text.
fn schema(text: &str) -> LoadError {
    Ruleset::from_data_block(&format!("const RULESET = {text};")).expect_err("a schema error")
}
