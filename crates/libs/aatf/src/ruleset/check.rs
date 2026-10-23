//! The load-time rules: `check` and its helpers. The error types they
//! produce are in `errors.rs`.

use std::collections::HashSet;

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
    /// order, map entries in key order; the tier rules that need the
    /// medal systems' tiers come after `medal_systems`. The Ruleset
    /// editor calls it on a ruleset being edited, so the rail can mark
    /// the sections its errors sit in.
    pub fn check(&self) -> Vec<RuleError> {
        let mut out = Vec::new();

        // name: an emptied string names nothing.
        if self.name.is_empty() {
            out.push(RuleError {
                path: "name".to_string(),
                rule: LoadRule::Empty,
            });
        }

        // stats.at_most: an ability stat, or one of the non-stat targets.
        for (i, name) in self.stats.at_most.iter().enumerate() {
            if !ABILITY_STATS.contains(&name.as_str()) && !NON_STAT_TARGETS.contains(&name.as_str())
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
            // Eight bytes, like a `name_colors` entry.
            if let Some(name_color) = &tier.name_color
                && name_color.len() != 8
            {
                out.push(RuleError {
                    path: format!("{path}.name_color"),
                    rule: LoadRule::ColorCodeLength,
                });
            }
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
        // The tiers the medal systems count, taken as a set: a tier one
        // names takes its count from the systems, not from the squad's
        // rest.
        let medal_tiers: HashSet<&str> = self
            .medal_systems
            .iter()
            .flat_map(|system| system.counts.keys().map(String::as_str))
            .collect();
        let rest_tiers = self
            .tiers
            .iter()
            .filter(|tier| {
                tier.count.is_none()
                    && tier.counts_as.is_none()
                    && !medal_tiers.contains(tier.id.as_str())
            })
            .count();
        // The counts equal `squad_size` when no tier is the rest of the
        // squad — under it, no team can be legal. With medal systems each
        // system's counts close the gap instead (checked with them below).
        if rest_tiers == 0 && medal_tiers.is_empty() && total < u32::from(self.squad_size) {
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
        // suggestion can never name two allowances). The groups are
        // disjoint and hold no COM style: a card in two would be made
        // free by whichever group the allocation tries first.
        for (index, (key, cards)) in self.card_groups.iter().enumerate() {
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
            for card in cards {
                // `com` is the first group the allocation tries.
                if matches!(card, Card::ComStyle(_)) {
                    out.push(RuleError {
                        path: format!("card_groups.{key}"),
                        rule: LoadRule::GroupsOverlap {
                            card: card.label().to_string(),
                            other: "com".to_string(),
                        },
                    });
                }
                for (other, earlier) in self.card_groups.iter().take(index) {
                    if earlier.contains(card) {
                        out.push(RuleError {
                            path: format!("card_groups.{key}"),
                            rule: LoadRule::GroupsOverlap {
                                card: card.label().to_string(),
                                other: other.clone(),
                            },
                        });
                    }
                }
            }
        }

        // medal_systems: labels non-empty, `counts` naming existing
        // tiers. Each system's counts plus the fixed counts fit
        // `squad_size`, and equal it when no tier is the rest of the
        // squad — under it, no team can be legal.
        for (i, system) in self.medal_systems.iter().enumerate() {
            let path = format!("medal_systems[{i}]");
            if system.label.is_empty() {
                out.push(RuleError {
                    path: format!("{path}.label"),
                    rule: LoadRule::Empty,
                });
            }
            for id in system.counts.keys() {
                if !tier_ids.contains(id.as_str()) {
                    out.push(RuleError {
                        path: format!("{path}.counts.{id}"),
                        rule: LoadRule::UnknownId {
                            kind: IdKind::Tier,
                            id: id.clone(),
                        },
                    });
                }
            }
            let counts = total + system.counts.values().map(|n| u32::from(*n)).sum::<u32>();
            if counts > u32::from(self.squad_size) {
                out.push(RuleError {
                    path,
                    rule: LoadRule::CountsExceedSquad {
                        total: counts,
                        squad_size: self.squad_size,
                    },
                });
            } else if rest_tiers == 0 && counts < u32::from(self.squad_size) {
                out.push(RuleError {
                    path,
                    rule: LoadRule::CountsShortOfSquad {
                        total: counts,
                        squad_size: self.squad_size,
                    },
                });
            }
        }
        // A tier a medal system counts takes its count from the systems:
        // no `count`, no `counts_as` of its own, and no `counts_as`
        // naming it.
        for (i, tier) in self.tiers.iter().enumerate() {
            if tier.count.is_some() && medal_tiers.contains(tier.id.as_str()) {
                out.push(RuleError {
                    path: format!("tiers[{i}]"),
                    rule: LoadRule::MedalTierCounted,
                });
            }
            if let Some(counts_as) = &tier.counts_as {
                if medal_tiers.contains(tier.id.as_str()) {
                    out.push(RuleError {
                        path: format!("tiers[{i}].counts_as"),
                        rule: LoadRule::MedalTierCountsAs,
                    });
                }
                if medal_tiers.contains(counts_as.as_str()) {
                    out.push(RuleError {
                        path: format!("tiers[{i}].counts_as"),
                        rule: LoadRule::CountsAsMedalTier,
                    });
                }
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
            // Every system but the last must have `any_player` or
            // `by_quotas` with quotas: with neither it holds for every team
            // (no quota fails), and the systems after it could never apply.
            if i + 1 != self.heights.systems.len()
                && system.any_player.as_ref().is_none_or(Condition::is_empty)
                && !(system.by_quotas && !system.quotas.is_empty())
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
                let bracket = self
                    .heights
                    .brackets
                    .iter()
                    .find(|bracket| &bracket.id == key);
                match bracket {
                    None => out.push(RuleError {
                        path: format!("{path}.quotas.{key}"),
                        rule: LoadRule::UnknownId {
                            kind: IdKind::Bracket,
                            id: key.clone(),
                        },
                    }),
                    // A pool system's quotas name `exactly` brackets of
                    // one height: the draw needs the bracket's own height.
                    Some(bracket)
                        if system.extra_height.is_some()
                            && !matches!(&bracket.heights, BracketHeights::Exactly(h) if h.len() == 1) =>
                    {
                        out.push(RuleError {
                            path: format!("{path}.quotas.{key}"),
                            rule: LoadRule::ExtraHeightQuotaNotExact,
                        });
                    }
                    _ => {}
                }
            }
            // A pool system's quotas fill the squad exactly.
            if system.extra_height.is_some() {
                let quotas: u32 = system.quotas.values().map(|n| u32::from(*n)).sum();
                if quotas != u32::from(self.squad_size) {
                    out.push(RuleError {
                        path: format!("{path}.quotas"),
                        rule: LoadRule::ExtraHeightQuotasNotSquad {
                            total: quotas,
                            squad_size: self.squad_size,
                        },
                    });
                }
            }
        }
        if let Some(last) = self.heights.systems.last() {
            let last_path = format!("heights.systems[{}]", self.heights.systems.len() - 1);
            if last
                .any_player
                .as_ref()
                .is_some_and(|condition| !condition.is_empty())
            {
                out.push(RuleError {
                    path: format!("{last_path}.any_player"),
                    rule: LoadRule::LastSystemHasCondition,
                });
            }
            if last.by_quotas {
                out.push(RuleError {
                    path: format!("{last_path}.by_quotas"),
                    rule: LoadRule::LastSystemHasCondition,
                });
            }
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
                allowance_with_bracket(&special.when, &format!("{path}.when"), &mut out);
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
        for (i, entry) in condition.any.iter().enumerate() {
            self.check_condition(entry, &format!("{path}.any[{i}]"), allow_tiers, out);
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
            ("form", &effects.form),
            ("injury_resistance", &effects.injury_resistance),
        ] {
            if let Some(per_tier) = per_tier {
                self.check_per_tier(per_tier, &format!("{path}.{name}"), out);
            }
        }
        if let Some(per_tier) = &effects.stat_warning {
            self.check_per_tier(per_tier, &format!("{path}.stat_warning"), out);
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
            let option_path = format!("{path}.choice[{i}]");
            // An option holds only the effects `Effects::choice`'s doc
            // lists (`LoadRule::ChoiceOptionEffect`): an option is found
            // used through a limit it raises, which the others never do.
            for (field, present) in [
                ("stat_target", !option.stat_target.is_empty()),
                ("require_cards", !option.require_cards.is_empty()),
                ("form", option.form.is_some()),
                ("injury_resistance", option.injury_resistance.is_some()),
                ("height_allowance", option.height_allowance.is_some()),
                ("forbid", option.forbid.is_some()),
                ("choice", !option.choice.is_empty()),
                ("stat_warning", option.stat_warning.is_some()),
            ] {
                if present {
                    out.push(RuleError {
                        path: format!("{option_path}.{field}"),
                        rule: LoadRule::ChoiceOptionEffect,
                    });
                }
            }
            self.check_effects(option, &option_path, out);
        }
    }
}
/// Every `brackets` entry of `condition` reported at its own path,
/// `not` nesting included: a special with `height_allowance` names no
/// bracket, brackets being computed from the allowance.
fn allowance_with_bracket(condition: &Condition, path: &str, out: &mut Vec<RuleError>) {
    for (i, _) in condition.brackets.iter().enumerate() {
        out.push(RuleError {
            path: format!("{path}.brackets[{i}]"),
            rule: LoadRule::AllowanceWithBracket,
        });
    }
    for (i, entry) in condition.any.iter().enumerate() {
        allowance_with_bracket(entry, &format!("{path}.any[{i}]"), out);
    }
    if let Some(not) = &condition.not {
        allowance_with_bracket(not, &format!("{path}.not"), out);
    }
}

/// `any_player` may name heights and positions only: whether any other
/// field is set, `not` and `any` entries included.
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
        || !condition.ranges.is_empty()
        || condition.strong_foot.is_some()
        || condition.any.iter().any(condition_is_wide)
        || condition.not.as_deref().is_some_and(condition_is_wide)
}

/// Whether a special's `then` or any `choice` option holds
/// `height_allowance`.
fn effects_have_allowance(effects: &Effects) -> bool {
    effects.height_allowance.is_some() || effects.choice.iter().any(effects_have_allowance)
}
