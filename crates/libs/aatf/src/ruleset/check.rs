//! The load-time rules: `check` and its helpers, and the error types they
//! produce (`LoadError`, `RuleError`, `LoadRule`, `IdKind`).

use std::collections::HashSet;
use std::fmt;

use pes_savefile::interchange::team_toml::labels::ABILITY_STATS;

use super::*;

/// The seven `suggestions` words; reserved as `card_groups` keys too,
/// so a suggestion can never name two allowances.
const SUGGESTION_WORDS: [&str; 7] = [
    "cards",
    "a_positions",
    "weak_foot",
    "injury_resistance",
    "height",
    "free_cards",
    "choices",
];

impl Ruleset {
    /// The rules the types cannot carry, every one broken, in a fixed
    /// order: `Ruleset`'s fields in declaration order, list items in
    /// order, map entries in key order.
    pub(crate) fn check(&self) -> Vec<RuleError> {
        let mut out = Vec::new();

        // name: an emptied string names nothing.
        if self.name.is_empty() {
            out.push(RuleError {
                path: "name".to_string(),
                rule: LoadRule::Empty,
            });
        }

        // stats.at_most: an ability stat, or one of the four non-stat targets.
        for (i, name) in self.stats.at_most.iter().enumerate() {
            if !ABILITY_STATS.contains(&name.as_str())
                && ![
                    "form",
                    "injury_resistance",
                    "weak_foot_usage",
                    "weak_foot_accuracy",
                ]
                .contains(&name.as_str())
            {
                out.push(RuleError {
                    path: format!("stats.at_most[{i}]"),
                    rule: LoadRule::UnknownTarget { name: name.clone() },
                });
            }
        }

        // tiers: DuplicateId (at the second), counts_as, free keys, the
        // when condition, count/counts_as.
        let tier_ids: HashSet<&str> = self.tiers.iter().map(|tier| tier.id.as_str()).collect();
        for (i, tier) in self.tiers.iter().enumerate() {
            let path = format!("tiers[{i}]");
            if tier.id.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.id"),
                    rule: LoadRule::Empty,
                });
            }
            if tier.label.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.label"),
                    rule: LoadRule::Empty,
                });
            }
            if self.tiers[..i].iter().any(|earlier| earlier.id == tier.id) {
                out.push(RuleError {
                    path: path.clone(),
                    rule: LoadRule::DuplicateId {
                        kind: IdKind::Tier,
                        id: tier.id.clone(),
                    },
                });
            }
            if let Some(counts_as) = &tier.counts_as {
                if !tier_ids.contains(counts_as.as_str()) {
                    out.push(RuleError {
                        path: format!("{path}.counts_as"),
                        rule: LoadRule::UnknownId {
                            kind: IdKind::Tier,
                            id: counts_as.clone(),
                        },
                    });
                } else if self
                    .tiers
                    .iter()
                    .any(|named| named.id == *counts_as && named.counts_as.is_some())
                {
                    // The tier `counts_as` names has a `counts_as` of its
                    // own, itself included.
                    out.push(RuleError {
                        path: format!("{path}.counts_as"),
                        rule: LoadRule::CountsAsChain,
                    });
                }
                if tier.count.is_some() {
                    out.push(RuleError {
                        path: path.clone(),
                        rule: LoadRule::CountWithCountsAs,
                    });
                }
            }
            for key in tier.free.keys() {
                if !self.group_has(key) {
                    out.push(RuleError {
                        path: format!("{path}.free.{key}"),
                        rule: LoadRule::UnknownId {
                            kind: IdKind::Group,
                            id: key.clone(),
                        },
                    });
                }
            }
            self.check_condition(&tier.when, &format!("{path}.when"), false, &mut out);
        }
        let total: u32 = self
            .tiers
            .iter()
            .filter_map(|tier| tier.count.map(u32::from))
            .sum();
        if total > u32::from(self.squad_size) {
            out.push(RuleError {
                path: "tiers".to_string(),
                rule: LoadRule::CountsExceedSquad {
                    total,
                    squad_size: self.squad_size,
                },
            });
        }
        let rest_tiers = self
            .tiers
            .iter()
            .filter(|tier| tier.count.is_none() && tier.counts_as.is_none())
            .count();
        // The counts equal `squad_size` when no tier is the rest of the
        // squad — under it, no team can be legal.
        if rest_tiers == 0 && total < u32::from(self.squad_size) {
            out.push(RuleError {
                path: "tiers".to_string(),
                rule: LoadRule::CountsShortOfSquad {
                    total,
                    squad_size: self.squad_size,
                },
            });
        }
        if rest_tiers > 1 {
            out.push(RuleError {
                path: "tiers".to_string(),
                rule: LoadRule::MoreThanOneRestTier,
            });
        }

        // card_groups: keys name a group; `com` and `suggestions`' seven
        // words are reserved (`com` for the COM styles, the words so a
        // suggestion can never name two allowances).
        for key in self.card_groups.keys() {
            if key.is_empty() {
                out.push(RuleError {
                    path: "card_groups".to_string(),
                    rule: LoadRule::Empty,
                });
            }
            if key == "com" || SUGGESTION_WORDS.contains(&key.as_str()) {
                out.push(RuleError {
                    path: format!("card_groups.{key}"),
                    rule: LoadRule::ReservedGroup,
                });
            }
        }

        // heights: brackets' and systems' ids unique and non-empty,
        // each bracket's heights above the brackets before it, quota
        // keys existing, a non-last system's `any_player` present and
        // non-empty and within heights and positions, the last system's
        // absent or empty.
        let mut previous_highest = 0u8;
        for (i, bracket) in self.heights.brackets.iter().enumerate() {
            let path = format!("heights.brackets[{i}]");
            if bracket.id.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.id"),
                    rule: LoadRule::Empty,
                });
            }
            if bracket.label.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.label"),
                    rule: LoadRule::Empty,
                });
            }
            if self.heights.brackets[..i]
                .iter()
                .any(|earlier| earlier.id == bracket.id)
            {
                out.push(RuleError {
                    path: path.clone(),
                    rule: LoadRule::DuplicateId {
                        kind: IdKind::Bracket,
                        id: bracket.id.clone(),
                    },
                });
            }
            // An `up_to: n` bracket takes the heights above the brackets
            // before it up to `n`, so it takes none unless `n` tops them;
            // an `exactly` list's minimum must top them.
            let (lowest, highest) = match &bracket.heights {
                BracketHeights::UpTo(limit) => (*limit, *limit),
                BracketHeights::Exactly(heights) => (
                    *heights
                        .iter()
                        .min()
                        .expect("`RawBracket` refuses an empty `exactly`"),
                    *heights
                        .iter()
                        .max()
                        .expect("`RawBracket` refuses an empty `exactly`"),
                ),
            };
            if lowest <= previous_highest {
                out.push(RuleError {
                    path: path.clone(),
                    rule: LoadRule::BracketsNotAscending,
                });
            }
            previous_highest = previous_highest.max(highest);
        }
        let bracket_ids: HashSet<&str> = self
            .heights
            .brackets
            .iter()
            .map(|bracket| bracket.id.as_str())
            .collect();
        for (i, system) in self.heights.systems.iter().enumerate() {
            let path = format!("heights.systems[{i}]");
            if system.id.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.id"),
                    rule: LoadRule::Empty,
                });
            }
            if system.label.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.label"),
                    rule: LoadRule::Empty,
                });
            }
            if self.heights.systems[..i]
                .iter()
                .any(|earlier| earlier.id == system.id)
            {
                out.push(RuleError {
                    path: path.clone(),
                    rule: LoadRule::DuplicateId {
                        kind: IdKind::System,
                        id: system.id.clone(),
                    },
                });
            }
            // Every system but the last must have `any_player`: absent or
            // empty it holds for every team, and the systems after it
            // could never apply.
            if i + 1 != self.heights.systems.len()
                && system.any_player.as_ref().is_none_or(Condition::is_empty)
            {
                out.push(RuleError {
                    path: path.clone(),
                    rule: LoadRule::SystemWithoutCondition,
                });
            }
            // Heights and positions only, so no id or colour code to check.
            if let Some(condition) = &system.any_player
                && condition_is_wide(condition)
            {
                out.push(RuleError {
                    path: format!("{path}.any_player"),
                    rule: LoadRule::SystemConditionTooWide,
                });
            }
            for key in system.quotas.keys() {
                if !bracket_ids.contains(key.as_str()) {
                    out.push(RuleError {
                        path: format!("{path}.quotas.{key}"),
                        rule: LoadRule::UnknownId {
                            kind: IdKind::Bracket,
                            id: key.clone(),
                        },
                    });
                }
            }
        }
        if let Some(last) = self.heights.systems.last()
            && last
                .any_player
                .as_ref()
                .is_some_and(|condition| !condition.is_empty())
        {
            out.push(RuleError {
                path: format!(
                    "heights.systems[{}].any_player",
                    self.heights.systems.len() - 1
                ),
                rule: LoadRule::LastSystemHasCondition,
            });
        }

        // universal: `fielded_at_a` names presets 1 to 3.
        for (i, preset) in self.universal.fielded_at_a.iter().enumerate() {
            if !(1..=3).contains(preset) {
                out.push(RuleError {
                    path: format!("universal.fielded_at_a[{i}]"),
                    rule: LoadRule::PresetOutOfRange { preset: *preset },
                });
            }
        }

        // specials: condition ids exist; `height_allowance` (in `then` or
        // any `choice` option) names no bracket in `when`.
        for (i, special) in self.specials.iter().enumerate() {
            let path = format!("specials[{i}]");
            if special.label.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.label"),
                    rule: LoadRule::Empty,
                });
            }
            self.check_condition(&special.when, &format!("{path}.when"), true, &mut out);
            if effects_have_allowance(&special.then) {
                for (j, _) in special.when.brackets.iter().enumerate() {
                    out.push(RuleError {
                        path: format!("{path}.when.brackets[{j}]"),
                        rule: LoadRule::AllowanceWithBracket,
                    });
                }
            }
            self.check_effects(&special.then, &format!("{path}.then"), &mut out);
        }

        // team_settings: a forbidden shape's counts add up to 10.
        for (i, settings) in self.team_settings.iter().enumerate() {
            if settings.label.is_empty() {
                out.push(RuleError {
                    path: format!("team_settings[{i}].label"),
                    rule: LoadRule::Empty,
                });
            }
            for (j, shape) in settings.forbidden_shapes.iter().enumerate() {
                if u32::from(shape[0]) + u32::from(shape[1]) + u32::from(shape[2]) != 10 {
                    out.push(RuleError {
                        path: format!("team_settings[{i}].forbidden_shapes[{j}]"),
                        rule: LoadRule::ShapeNotTen { shape: *shape },
                    });
                }
            }
        }

        // suggestions: one of the seven words or a group id (`com`
        // included).
        for (i, entry) in self.suggestions.iter().enumerate() {
            if !SUGGESTION_WORDS.contains(&entry.as_str()) && !self.group_has(entry) {
                out.push(RuleError {
                    path: format!("suggestions[{i}]"),
                    rule: LoadRule::UnknownSuggestion {
                        entry: entry.clone(),
                    },
                });
            }
        }

        out
    }

    /// Whether `id` names a tier.
    fn tier_has(&self, id: &str) -> bool {
        self.tiers.iter().any(|tier| tier.id == id)
    }

    /// Whether `id` names a group (`com` or a `card_groups` key).
    fn group_has(&self, id: &str) -> bool {
        id == "com" || self.card_groups.contains_key(id)
    }

    /// Whether `id` names a height system.
    fn system_has(&self, id: &str) -> bool {
        self.heights.systems.iter().any(|system| system.id == id)
    }

    /// Whether `id` names a height bracket.
    fn bracket_has(&self, id: &str) -> bool {
        self.heights.brackets.iter().any(|bracket| bracket.id == id)
    }

    /// A condition's tiers, systems and brackets checked for existence —
    /// every name a condition may hold is an error (never a skip):
    /// `UnknownId` where the name exists nowhere. `allow_tiers` splits the
    /// two `tiers` rules: `true` for a special's `when` (tiers are
    /// legitimate names), `false` for a tier's own `when`
    /// (`TierConditionNamesTiers`). `not` is checked like the condition it
    /// sits in.
    fn check_condition(
        &self,
        condition: &Condition,
        path: &str,
        allow_tiers: bool,
        out: &mut Vec<RuleError>,
    ) {
        if allow_tiers {
            for (i, id) in condition.tiers.iter().enumerate() {
                if !self.tier_has(id) {
                    out.push(RuleError {
                        path: format!("{path}.tiers[{i}]"),
                        rule: LoadRule::UnknownId {
                            kind: IdKind::Tier,
                            id: id.clone(),
                        },
                    });
                }
            }
        } else if !condition.tiers.is_empty() {
            out.push(RuleError {
                path: format!("{path}.tiers"),
                rule: LoadRule::TierConditionNamesTiers,
            });
        }
        for (i, id) in condition.systems.iter().enumerate() {
            if !self.system_has(id) {
                out.push(RuleError {
                    path: format!("{path}.systems[{i}]"),
                    rule: LoadRule::UnknownId {
                        kind: IdKind::System,
                        id: id.clone(),
                    },
                });
            }
        }
        for (i, id) in condition.brackets.iter().enumerate() {
            if !self.bracket_has(id) {
                out.push(RuleError {
                    path: format!("{path}.brackets[{i}]"),
                    rule: LoadRule::UnknownId {
                        kind: IdKind::Bracket,
                        id: id.clone(),
                    },
                });
            }
        }
        // `name_colors`: eight bytes each, any bytes (a partial paste
        // matches no name).
        for (i, entry) in condition.name_colors.iter().enumerate() {
            if entry.len() != 8 {
                out.push(RuleError {
                    path: format!("{path}.name_colors[{i}]"),
                    rule: LoadRule::ColorCodeLength,
                });
            }
        }
        if let Some(not) = &condition.not {
            self.check_condition(not, &format!("{path}.not"), allow_tiers, out);
        }
    }

    /// A `ByTier` map's tier keys checked for existence at
    /// `prefix.{id}`; `All` maps nothing.
    fn check_per_tier<T>(&self, per_tier: &PerTier<T>, prefix: &str, out: &mut Vec<RuleError>) {
        let PerTier::ByTier(map) = per_tier else {
            return;
        };
        for id in map.keys() {
            if !self.tier_has(id) {
                out.push(RuleError {
                    path: format!("{prefix}.{id}"),
                    rule: LoadRule::UnknownId {
                        kind: IdKind::Tier,
                        id: id.clone(),
                    },
                });
            }
        }
    }

    /// An effects block's PerTier map keys (tier ids) and `free` keys
    /// (group ids) checked for existence, recursively into `choice`.
    fn check_effects(&self, effects: &Effects, path: &str, out: &mut Vec<RuleError>) {
        for (key, per_tier) in &effects.stat_bonus {
            let label = match key {
                StatOrAll::All => "all",
                StatOrAll::Stat(stat) => stat.label(),
            };
            self.check_per_tier(per_tier, &format!("{path}.stat_bonus.{label}"), out);
        }
        for (stat, per_tier) in &effects.stat_target {
            self.check_per_tier(
                per_tier,
                &format!("{path}.stat_target.{}", stat.label()),
                out,
            );
        }
        for (name, per_tier) in [
            ("cards", &effects.cards),
            ("a_positions", &effects.a_positions),
        ] {
            if let Some(per_tier) = per_tier {
                self.check_per_tier(per_tier, &format!("{path}.{name}"), out);
            }
        }
        for (group, per_tier) in &effects.free {
            if !self.group_has(group) {
                out.push(RuleError {
                    path: format!("{path}.free.{group}"),
                    rule: LoadRule::UnknownId {
                        kind: IdKind::Group,
                        id: group.clone(),
                    },
                });
            }
            self.check_per_tier(per_tier, &format!("{path}.free.{group}"), out);
        }
        // `forbid`: an emptied text would show a blank violation.
        if let Some(forbid) = &effects.forbid
            && forbid.is_empty()
        {
            out.push(RuleError {
                path: format!("{path}.forbid"),
                rule: LoadRule::Empty,
            });
        }
        for (i, option) in effects.choice.iter().enumerate() {
            self.check_effects(option, &format!("{path}.choice[{i}]"), out);
        }
    }
}
/// `any_player` may name heights and positions only: whether any other
/// field is set, `not` included.
fn condition_is_wide(condition: &Condition) -> bool {
    !condition.tiers.is_empty()
        || !condition.systems.is_empty()
        || !condition.brackets.is_empty()
        || condition.captain.is_some()
        || !condition.playing_styles.is_empty()
        || !condition.holds.is_empty()
        || !condition.rated_a_at.is_empty()
        || !condition.rated_b_at.is_empty()
        || !condition.name_colors.is_empty()
        || condition.fielded_only_at.is_some()
        || condition.not.as_deref().is_some_and(condition_is_wide)
}

/// Whether a special's `then` or any `choice` option holds
/// `height_allowance`.
fn effects_have_allowance(effects: &Effects) -> bool {
    effects.height_allowance.is_some() || effects.choice.iter().any(effects_have_allowance)
}

/// Why a ruleset did not load.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LoadError {
    /// The text does not compile, or its top level fails to run.
    #[error("{line}:{column}: {message}")]
    Parse {
        /// The line the error sits at.
        line: usize,
        /// The column the error sits at.
        column: usize,
        /// What the parser reported.
        message: String,
    },
    /// The text defines no constant `RULESET` map (a `let` is not one:
    /// a function cannot read it).
    #[error("the file defines no constant RULESET map")]
    NoRuleset,
    /// `RULESET` does not deserialize into `Ruleset`: an unknown or missing
    /// key, a wrong type, an unknown label.
    #[error("{path}: {message}")]
    Schema {
        /// The key path, empty in this slice (`docs/QUESTIONS.md`); `message`
        /// carries rhai's text, which names the key or label.
        path: String,
        /// What the deserializer reported.
        message: String,
    },
    /// A `schema` this build does not read (it reads 1).
    #[error("schema {schema} is not one this build reads (1)")]
    UnknownSchema {
        /// The schema the file names.
        schema: u32,
    },
    /// A required function is not defined with its arity.
    #[error("the file defines no {name} with {arity} parameter(s)")]
    MissingFunction {
        /// The function.
        name: &'static str,
        /// The arity it must take.
        arity: usize,
    },
    /// `custom_checks` takes exactly one parameter (the team); another
    /// arity would be skipped without a word.
    #[error("custom_checks takes one parameter (the team), not {arity}")]
    CustomChecksArity {
        /// The arity the file defines it with.
        arity: usize,
    },
    /// `check_team` fails on the sample team (load step 3).
    #[error("check_team fails on the sample team: {error}")]
    SampleTeam {
        /// The `CheckError` it gave.
        error: crate::violation::CheckError,
    },
    /// The rules the types cannot carry, every one broken, in a fixed order.
    #[error("{}", fmt_rules(.0))]
    Rules(Vec<RuleError>),
}

/// One load-time rule broken at a key path.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleError {
    /// The key path of the value that breaks the rule
    /// (`tiers[2].free.tricks`, `specials[4].when.tiers[1]`; the list itself,
    /// `tiers`, for `CountsExceedSquad`, `CountsShortOfSquad` and
    /// `MoreThanOneRestTier`).
    pub path: String,
    /// Which rule broke.
    pub rule: LoadRule,
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.rule)
    }
}

/// Which of the four id kinds a `DuplicateId`/`UnknownId` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdKind {
    /// A tier.
    Tier,
    /// A height bracket.
    Bracket,
    /// A height system.
    System,
    /// A card group.
    Group,
}

impl fmt::Display for IdKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            IdKind::Tier => "tier",
            IdKind::Bracket => "bracket",
            IdKind::System => "system",
            IdKind::Group => "group",
        })
    }
}

/// The rules the types cannot carry, checked on load ("Loading and
/// validation").
#[derive(Debug, Clone, PartialEq)]
pub enum LoadRule {
    /// An id a file may not repeat within its kind.
    DuplicateId {
        /// Which kind of id repeated.
        kind: IdKind,
        /// The repeated id.
        id: String,
    },
    /// An id a condition, effect, quota, `counts_as` or suggestion names that
    /// exists nowhere.
    UnknownId {
        /// Which kind of id is missing.
        kind: IdKind,
        /// The unknown id.
        id: String,
    },
    /// A string is empty: a `name`, an id, a label, a `card_groups` key
    /// or a `forbid` text.
    Empty,
    /// A `name_colors` entry is not eight bytes long (any bytes — the
    /// game does not check they are hex).
    ColorCodeLength,
    /// A `suggestions` entry that is neither one of the seven words nor a
    /// group id.
    UnknownSuggestion {
        /// The entry.
        entry: String,
    },
    /// An `at_most` entry that is not an ability stat or one of `form`,
    /// `injury_resistance`, `weak_foot_usage`, `weak_foot_accuracy`.
    UnknownTarget {
        /// The entry.
        name: String,
    },
    /// A `card_groups` key `com` or one of `suggestions`' seven words:
    /// `com` is the COM styles' group, and a word named a group would let
    /// a suggestion mean two allowances.
    ReservedGroup,
    /// A tier with `counts_as` also has `count`.
    CountWithCountsAs,
    /// The tier a `counts_as` names has a `counts_as` of its own (itself
    /// included).
    CountsAsChain,
    /// More than one tier has neither `count` nor `counts_as`.
    MoreThanOneRestTier,
    /// The counts' sum is over `squad_size`.
    CountsExceedSquad {
        /// The counts' sum.
        total: u32,
        /// The squad's size.
        squad_size: u8,
    },
    /// No tier is the rest of the squad, and the counts' sum is under
    /// `squad_size`.
    CountsShortOfSquad {
        /// The counts' sum.
        total: u32,
        /// The squad's size.
        squad_size: u8,
    },
    /// A height system other than the last has no `any_player`, or an
    /// empty one.
    SystemWithoutCondition,
    /// A bracket's lowest height is at most the highest height of the
    /// brackets before it: a reordered bracket takes no player.
    BracketsNotAscending,
    /// A tier's `when` names tiers.
    TierConditionNamesTiers,
    /// A system's `any_player` sets more than `height_min`, `height_max` and
    /// `positions`.
    SystemConditionTooWide,
    /// The last system has `any_player` set.
    LastSystemHasCondition,
    /// A special with `height_allowance` (in `then` or any `choice` option)
    /// names a bracket in its `when`.
    AllowanceWithBracket,
    /// `fielded_at_a` holds a preset outside 1 to 3.
    PresetOutOfRange {
        /// The out-of-range preset.
        preset: u8,
    },
    /// A forbidden shape's counts do not add up to 10.
    ShapeNotTen {
        /// The shape (`[defenders, midfielders, forwards]`).
        shape: [u8; 3],
    },
}

impl fmt::Display for LoadRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadRule::DuplicateId { kind, id } => write!(f, "{kind} id {id:?} is used twice"),
            LoadRule::UnknownId { kind, id } => write!(f, "unknown {kind} id {id:?}"),
            LoadRule::UnknownSuggestion { entry } => {
                write!(f, "unknown suggestion {entry:?}")
            }
            LoadRule::UnknownTarget { name } => write!(f, "unknown at_most target {name:?}"),
            LoadRule::ReservedGroup => {
                f.write_str("`com` or a `suggestions` word is a reserved card group")
            }
            LoadRule::CountWithCountsAs => f.write_str("a tier with `counts_as` also has `count`"),
            LoadRule::CountsAsChain => {
                f.write_str("the tier a `counts_as` names has a `counts_as` of its own")
            }
            LoadRule::MoreThanOneRestTier => {
                f.write_str("more than one tier has neither `count` nor `counts_as`")
            }
            LoadRule::CountsExceedSquad { total, squad_size } => {
                write!(f, "counts total {total} over squad_size {squad_size}")
            }
            LoadRule::CountsShortOfSquad { total, squad_size } => {
                write!(f, "counts total {total} under squad_size {squad_size}")
            }
            LoadRule::SystemWithoutCondition => {
                f.write_str("a height system other than the last has no or an empty `any_player`")
            }
            LoadRule::Empty => f.write_str("empty string"),
            LoadRule::ColorCodeLength => f.write_str("a `name_colors` entry is not eight bytes"),
            LoadRule::BracketsNotAscending => {
                f.write_str("bracket heights not above the brackets before it")
            }
            LoadRule::TierConditionNamesTiers => f.write_str("a tier's `when` names tiers"),
            LoadRule::SystemConditionTooWide => {
                f.write_str("`any_player` sets more than heights and positions")
            }
            LoadRule::LastSystemHasCondition => f.write_str("the last system has `any_player`"),
            LoadRule::AllowanceWithBracket => {
                f.write_str("a special with `height_allowance` names a bracket")
            }
            LoadRule::PresetOutOfRange { preset } => {
                write!(f, "preset {preset} is outside 1 to 3")
            }
            LoadRule::ShapeNotTen { shape } => {
                write!(f, "shape {shape:?} does not add up to 10")
            }
        }
    }
}

/// The `Rules` display's join of `RuleError`s.
fn fmt_rules(rules: &[RuleError]) -> String {
    rules
        .iter()
        .map(RuleError::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}
