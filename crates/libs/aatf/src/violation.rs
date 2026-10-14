//! `Violation`: what `check_team` reports — a finding a team's player
//! or the team itself breaks, converted from the script's list of maps.

use rhai::serde::from_dynamic;
use rhai::{Dynamic, Map};
use serde::Deserialize;

use crate::ruleset::{BracketId, Card, Position, Severity, Stat, SystemId, TierId};

/// One finding: a rule a player or the team breaks (`slot` `None` for
/// the team).
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    /// The roster slot the violation is on; `None`: the team.
    pub slot: Option<u8>,
    /// Error, warning or suggestion.
    pub severity: Severity,
    /// Which rule.
    pub kind: ViolationKind,
}

/// Which rule a violation reports.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum ViolationKind {
    /// The registered position is not rated A (under
    /// `registered_position_a`).
    RegisteredNotA,
    /// Fielded at `position` in preset `preset`, which it is not rated A
    /// at (under `fielded_at_a`).
    FieldedNotA {
        /// The preset (1 to 3).
        preset: u8,
        /// The position.
        position: Position,
    },
    /// A B rating where `no_b_ratings` refuses one.
    BRating,
    /// A second A rating that is GK's (under `gk_not_second_a`).
    GkSecondA,
    /// Age outside `universal.age`.
    Age {
        /// The player's age.
        age: u8,
        /// The minimum.
        min: u8,
        /// The maximum.
        max: u8,
    },
    /// Weight outside the version's weight rule.
    Weight {
        /// The player's weight.
        weight: u8,
        /// The bound's minimum.
        min: u8,
        /// The bound's maximum.
        max: u8,
    },
    /// The registered-position byte is past the version's position
    /// labels.
    PositionRange,
    /// A playing style outside its allowed set.
    PlayingStyleRange,
    /// No tier's `when` held (the player gets no tier and no further
    /// per-player check).
    NoTier,
    /// An ability stat below its target.
    Stat {
        /// The stat.
        stat: Stat,
        /// The player's value.
        value: u8,
        /// The tier's or the effect's target.
        target: u8,
        /// The target is an `at_most` entry (below is a violation, above
        /// is not).
        at_most: bool,
    },
    /// Form below the tier's target.
    Form {
        /// The player's form.
        value: u8,
        /// The target.
        target: u8,
    },
    /// Injury resistance below the target.
    InjuryResistance {
        /// The player's value.
        value: u8,
        /// The maximum.
        max: u8,
    },
    /// Weak foot usage below the target.
    WeakFootUsage {
        /// The player's value.
        value: u8,
        /// The maximum.
        max: u8,
    },
    /// Weak foot accuracy below the target.
    WeakFootAccuracy {
        /// The player's value.
        value: u8,
        /// The maximum.
        max: u8,
    },
    /// The name's leading colour code against the tier's; `expected: None`
    /// is a tier that sets none, met by a name with no colour code the
    /// tiers set.
    NameColor {
        /// The name's leading colour code (`None` for a name without one).
        color: Option<String>,
        /// The tier's code (`None` on a tier that sets none).
        expected: Option<String>,
    },
    /// Paid cards over the limit.
    Cards {
        /// The paid count.
        paid: u8,
        /// The limit.
        limit: u8,
    },
    /// Paid skill cards plus extra A positions under `min_cards`.
    MinCards {
        /// The paid count.
        paid: u8,
        /// The minimum.
        min: u8,
    },
    /// More skill cards than `max_skill_cards`.
    SkillCardCap {
        /// The count.
        count: u8,
        /// The cap.
        max: u8,
    },
    /// More COM styles than `max_com_styles`.
    ComStyleCap {
        /// The count.
        count: u8,
        /// The cap.
        max: u8,
    },
    /// A `require_cards` card not held.
    RequiredCard {
        /// The card.
        card: Card,
    },
    /// More A-rated positions than the allowance.
    APositions {
        /// The count.
        count: u8,
        /// The allowance.
        max: u8,
    },
    /// A height no bracket of the team's system takes.
    Height {
        /// The player's height in cm.
        height: u8,
    },
    /// A tier's count off its expected one (`counts_as` folded in).
    TierCount {
        /// The tier.
        tier: TierId,
        /// The players in it.
        count: u8,
        /// Its `count` or the rest of the squad.
        expected: u8,
    },
    /// A bracket's quota off its system's.
    BracketQuota {
        /// The system.
        system: SystemId,
        /// The bracket.
        bracket: BracketId,
        /// The players in it.
        count: u8,
        /// The quota.
        quota: u8,
    },
    /// No captain where `captain_required`.
    Captain,
    /// Fewer goalkeepers than `min_goalkeepers`.
    Goalkeepers {
        /// The count.
        count: u8,
        /// The minimum.
        min: u8,
    },
    /// A team settings entry whose value does not hold on this save.
    TeamSetting {
        /// The entry's label.
        label: String,
        /// Which setting fails (`man_marking`, an auto flag).
        setting: String,
    },
    /// A formation matching a `forbidden_shapes` shape.
    FormationShape {
        /// The preset (1 to 3).
        preset: u8,
        /// The formation within it (1 to 3).
        formation: u8,
        /// `[defenders, midfielders, forwards]`.
        shape: [u8; 3],
    },
    /// A special's `forbid` fired: `text` is the ruleset author's.
    Special {
        /// The special's label.
        label: String,
        /// Its `forbid` text.
        text: String,
    },
    /// A `custom_checks` violation: `text` is the check author's.
    Custom {
        /// The violation text.
        text: String,
    },
    /// A suggestion: an allowance listed in `suggestions` the player does
    /// not use up.
    Unused {
        /// Which allowance (a `suggestions` word or group).
        allowance: String,
        /// How much is used.
        used: u8,
        /// How much is allowed.
        allowed: u8,
    },
    /// A suggestion under `choices`: a special's choice the player takes
    /// no option of.
    UnusedChoice {
        /// The special's label.
        special: String,
    },
}

/// Why a check could not produce its violations.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CheckError {
    /// The script failed: a runtime error or a limit reached; `message`
    /// is rhai's, with its position.
    #[error("the check's script failed: {message}")]
    Script {
        /// rhai's error text.
        message: String,
    },
    /// `check_team` returned something other than a list (`type_name`:
    /// rhai's name for it).
    #[error("check_team did not return a list (got {type_name})")]
    NotAList {
        /// rhai's name for the type.
        type_name: String,
    },
    /// Entry `index` of the returned list does not convert into a
    /// `Violation`.
    #[error("violation {index}: {message}")]
    Violation {
        /// The entry's index.
        index: usize,
        /// What did not convert.
        message: String,
    },
    /// `tier_values` or `card_limits` (`function`) returned a map that
    /// does not convert into its values.
    #[error("{function} did not return its values: {message}")]
    Values {
        /// The script function.
        function: &'static str,
        /// What did not convert.
        message: String,
    },
    /// No tier of the file has this id.
    #[error("unknown tier {tier}")]
    UnknownTier {
        /// The id asked for.
        tier: String,
    },
}

/// The violations of `check_team`'s returned list, in its order: every
/// entry converts or the first bad one fails the whole check (a bug in
/// the logic or a custom check, never skipped).
pub(crate) fn violations(list: Dynamic) -> Result<Vec<Violation>, CheckError> {
    let entries = list
        .into_array()
        .map_err(|type_name| CheckError::NotAList {
            type_name: type_name.to_string(),
        })?;
    let mut out = Vec::with_capacity(entries.len());
    for (index, entry) in entries.into_iter().enumerate() {
        let mut map = entry
            .try_cast::<Map>()
            .ok_or_else(|| CheckError::Violation {
                index,
                message: "not a map".to_string(),
            })?;
        let slot = match map.remove("slot") {
            Some(d) if d.is_unit() => None,
            Some(d) => Some(
                u8::try_from(d.as_int().map_err(|e| CheckError::Violation {
                    index,
                    message: format!("slot is not a number: {e}"),
                })?)
                .map_err(|_| CheckError::Violation {
                    index,
                    message: format!("slot {d:?}: past u8"),
                })?,
            ),
            None => None,
        };
        let severity = match map.remove("severity") {
            None => Severity::Error,
            Some(d) => from_dynamic::<Severity>(&d).map_err(|e| CheckError::Violation {
                index,
                message: format!("severity: {e}"),
            })?,
        };
        let kind_dynamic = Dynamic::from_map(map.clone());
        let kind =
            from_dynamic::<ViolationKind>(&kind_dynamic).map_err(|e| CheckError::Violation {
                index,
                message: e.to_string(),
            })?;
        // `deny_unknown_fields` does not see a unit variant's extra key
        // (serde's internally-tagged read lets it through): rule it out
        // here — a unit variant takes `kind` alone. Every variant placed
        // on purpose: a new one lands in a match arm, not a `_`.
        match kind {
            ViolationKind::RegisteredNotA
            | ViolationKind::BRating
            | ViolationKind::GkSecondA
            | ViolationKind::PositionRange
            | ViolationKind::PlayingStyleRange
            | ViolationKind::NoTier
            | ViolationKind::Captain => {
                if let Some(extra) = map.keys().find(|key| key.as_str() != "kind") {
                    return Err(CheckError::Violation {
                        index,
                        message: format!("`{extra}`: unknown key on a unit variant"),
                    });
                }
            }
            ViolationKind::FieldedNotA { .. }
            | ViolationKind::Age { .. }
            | ViolationKind::Weight { .. }
            | ViolationKind::Stat { .. }
            | ViolationKind::Form { .. }
            | ViolationKind::InjuryResistance { .. }
            | ViolationKind::WeakFootUsage { .. }
            | ViolationKind::WeakFootAccuracy { .. }
            | ViolationKind::NameColor { .. }
            | ViolationKind::Cards { .. }
            | ViolationKind::MinCards { .. }
            | ViolationKind::SkillCardCap { .. }
            | ViolationKind::ComStyleCap { .. }
            | ViolationKind::RequiredCard { .. }
            | ViolationKind::APositions { .. }
            | ViolationKind::Height { .. }
            | ViolationKind::TierCount { .. }
            | ViolationKind::BracketQuota { .. }
            | ViolationKind::Goalkeepers { .. }
            | ViolationKind::TeamSetting { .. }
            | ViolationKind::FormationShape { .. }
            | ViolationKind::Special { .. }
            | ViolationKind::Custom { .. }
            | ViolationKind::Unused { .. }
            | ViolationKind::UnusedChoice { .. } => {}
        }
        out.push(Violation {
            slot,
            severity,
            kind,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::engine;
    use crate::ruleset::Severity;

    /// `text` evaluated as a `check_team`-shaped list of one map.
    fn one(text: &str) -> Dynamic {
        engine()
            .eval::<Dynamic>(&format!("[{text}]"))
            .expect("a list of one map")
    }

    /// `text` → the one `Violation` its map converts into.
    fn one_violation(text: &str) -> Violation {
        violations(one(text))
            .expect(text)
            .into_iter()
            .next()
            .expect("one")
    }

    /// Every `ViolationKind` variant converts; `slot` and `severity`
    /// vary across rows (`warning` and `suggestion` each once).
    #[test]
    fn every_kind_converts() {
        let cases: &[(&str, Option<u8>, Severity, ViolationKind)] = &[
            (
                "#{ kind: \"RegisteredNotA\" }",
                None,
                Severity::Error,
                ViolationKind::RegisteredNotA,
            ),
            (
                "#{ kind: \"FieldedNotA\", preset: 1, position: \"CB\", slot: 5, severity: \"warning\" }",
                Some(5),
                Severity::Warning,
                ViolationKind::FieldedNotA {
                    preset: 1,
                    position: Position(1),
                },
            ),
            (
                "#{ kind: \"BRating\" }",
                None,
                Severity::Error,
                ViolationKind::BRating,
            ),
            (
                "#{ kind: \"GkSecondA\" }",
                None,
                Severity::Error,
                ViolationKind::GkSecondA,
            ),
            (
                "#{ kind: \"Age\", age: 15, min: 16, max: 40 }",
                None,
                Severity::Error,
                ViolationKind::Age {
                    age: 15,
                    min: 16,
                    max: 40,
                },
            ),
            (
                "#{ kind: \"Weight\", weight: 80, min: 30, max: 50 }",
                None,
                Severity::Error,
                ViolationKind::Weight {
                    weight: 80,
                    min: 30,
                    max: 50,
                },
            ),
            (
                "#{ kind: \"PositionRange\" }",
                None,
                Severity::Error,
                ViolationKind::PositionRange,
            ),
            (
                "#{ kind: \"PlayingStyleRange\" }",
                None,
                Severity::Error,
                ViolationKind::PlayingStyleRange,
            ),
            (
                "#{ kind: \"NoTier\" }",
                None,
                Severity::Error,
                ViolationKind::NoTier,
            ),
            (
                "#{ kind: \"Stat\", stat: \"speed\", value: 80, target: 92, at_most: true }",
                None,
                Severity::Error,
                ViolationKind::Stat {
                    stat: Stat(12),
                    value: 80,
                    target: 92,
                    at_most: true,
                },
            ),
            (
                "#{ kind: \"Form\", value: 2, target: 4 }",
                None,
                Severity::Error,
                ViolationKind::Form {
                    value: 2,
                    target: 4,
                },
            ),
            (
                "#{ kind: \"InjuryResistance\", value: 0, max: 2 }",
                None,
                Severity::Error,
                ViolationKind::InjuryResistance { value: 0, max: 2 },
            ),
            (
                "#{ kind: \"WeakFootUsage\", value: 2, max: 4 }",
                None,
                Severity::Error,
                ViolationKind::WeakFootUsage { value: 2, max: 4 },
            ),
            (
                "#{ kind: \"WeakFootAccuracy\", value: 1, max: 3 }",
                None,
                Severity::Error,
                ViolationKind::WeakFootAccuracy { value: 1, max: 3 },
            ),
            (
                "#{ kind: \"NameColor\", color: \"c0c0c0ff\", expected: () }",
                None,
                Severity::Error,
                ViolationKind::NameColor {
                    color: Some("c0c0c0ff".to_string()),
                    expected: None,
                },
            ),
            (
                "#{ kind: \"Cards\", paid: 10, limit: 8 }",
                None,
                Severity::Error,
                ViolationKind::Cards { paid: 10, limit: 8 },
            ),
            (
                "#{ kind: \"MinCards\", paid: 3, min: 5 }",
                None,
                Severity::Error,
                ViolationKind::MinCards { paid: 3, min: 5 },
            ),
            (
                "#{ kind: \"SkillCardCap\", count: 11, max: 10 }",
                None,
                Severity::Error,
                ViolationKind::SkillCardCap { count: 11, max: 10 },
            ),
            (
                "#{ kind: \"ComStyleCap\", count: 8, max: 7 }",
                None,
                Severity::Error,
                ViolationKind::ComStyleCap { count: 8, max: 7 },
            ),
            (
                "#{ kind: \"RequiredCard\", card: \"malicia\" }",
                None,
                Severity::Error,
                ViolationKind::RequiredCard {
                    card: Card::Skill(21),
                },
            ),
            (
                "#{ kind: \"APositions\", count: 3, max: 2 }",
                None,
                Severity::Error,
                ViolationKind::APositions { count: 3, max: 2 },
            ),
            (
                "#{ kind: \"Height\", height: 191 }",
                None,
                Severity::Error,
                ViolationKind::Height { height: 191 },
            ),
            (
                "#{ kind: \"TierCount\", tier: \"gold\", count: 0, expected: 1 }",
                None,
                Severity::Error,
                ViolationKind::TierCount {
                    tier: "gold".to_string(),
                    count: 0,
                    expected: 1,
                },
            ),
            (
                "#{ kind: \"BracketQuota\", system: \"red\", bracket: \"tall\", count: 3, quota: 6 }",
                None,
                Severity::Error,
                ViolationKind::BracketQuota {
                    system: "red".to_string(),
                    bracket: "tall".to_string(),
                    count: 3,
                    quota: 6,
                },
            ),
            (
                "#{ kind: \"Captain\" }",
                None,
                Severity::Error,
                ViolationKind::Captain,
            ),
            (
                "#{ kind: \"Goalkeepers\", count: 0, min: 1 }",
                None,
                Severity::Error,
                ViolationKind::Goalkeepers { count: 0, min: 1 },
            ),
            (
                "#{ kind: \"TeamSetting\", label: \"no offside trap\", setting: \"offside_trap\" }",
                None,
                Severity::Error,
                ViolationKind::TeamSetting {
                    label: "no offside trap".to_string(),
                    setting: "offside_trap".to_string(),
                },
            ),
            (
                "#{ kind: \"FormationShape\", preset: 2, formation: 3, shape: [3, 4, 3] }",
                None,
                Severity::Error,
                ViolationKind::FormationShape {
                    preset: 2,
                    formation: 3,
                    shape: [3, 4, 3],
                },
            ),
            (
                "#{ kind: \"Special\", label: \"Red CBs\", text: \"CBs up to 189\" }",
                None,
                Severity::Error,
                ViolationKind::Special {
                    label: "Red CBs".to_string(),
                    text: "CBs up to 189".to_string(),
                },
            ),
            (
                "#{ kind: \"Custom\", text: \"x\", slot: 0, severity: \"suggestion\" }",
                Some(0),
                Severity::Suggestion,
                ViolationKind::Custom {
                    text: "x".to_string(),
                },
            ),
            (
                "#{ kind: \"Unused\", allowance: \"cards\", used: 1, allowed: 3 }",
                None,
                Severity::Error,
                ViolationKind::Unused {
                    allowance: "cards".to_string(),
                    used: 1,
                    allowed: 3,
                },
            ),
            (
                "#{ kind: \"UnusedChoice\", special: \"Medals\" }",
                None,
                Severity::Error,
                ViolationKind::UnusedChoice {
                    special: "Medals".to_string(),
                },
            ),
        ];
        assert_eq!(cases.len(), 32);
        for (text, slot, severity, kind) in cases {
            let got = violations(one(text)).expect(text).remove(0);
            assert_eq!(
                got,
                Violation {
                    slot: *slot,
                    severity: *severity,
                    kind: kind.clone(),
                },
                "{text}"
            );
        }
    }

    #[test]
    fn slot_and_severity_default() {
        // No `slot`, no `severity`: `None` and `Error`.
        assert_eq!(one_violation("#{ kind: \"Captain\" }").slot, None);
        assert_eq!(
            one_violation("#{ kind: \"Captain\" }").severity,
            Severity::Error
        );
        assert_eq!(one_violation("#{ kind: \"Captain\", slot: () }").slot, None);
    }

    /// `bad` second in a good-bad-good list is `Violation { index: 1 }`
    /// naming `culprit`.
    fn bad_entry(bad: &str, culprit: &str) {
        let err = violations(
            engine()
                .eval::<Dynamic>(&format!(
                    "[#{{ kind: \"Captain\" }}, {bad}, #{{ kind: \"Custom\", text: \"x\" }}]"
                ))
                .expect("a list"),
        )
        .expect_err("the bad entry");
        match err {
            CheckError::Violation { index, message } => {
                assert_eq!(index, 1);
                assert!(message.contains(culprit), "{culprit}: {message}");
            }
            other => panic!("expected Violation, got {other:?}"),
        }
    }

    #[test]
    fn bad_entries_fail_at_their_index() {
        bad_entry("#{ kind: \"Nope\" }", "Nope");
        bad_entry("#{ kind: \"Age\", age: 15, min: 16 }", "max");
        bad_entry(
            "#{ kind: \"Age\", age: 15, min: 16, max: 40, extra: 1 }",
            "extra",
        );
        bad_entry("#{ kind: \"Captain\", extra: 1 }", "extra");
        bad_entry("#{ kind: \"Age\", age: 300, min: 16, max: 40 }", "u8");
        bad_entry("#{ kind: \"Age\", age: -1, min: 16, max: 40 }", "u8");
        bad_entry(
            "#{ kind: \"Stat\", stat: \"pace\", value: 80, target: 92, at_most: true }",
            "pace",
        );
        bad_entry("#{ kind: \"Captain\", severity: \"fatal\" }", "fatal");
        bad_entry("#{ kind: \"Captain\", slot: 300 }", "300");
        bad_entry("5", "not a map");
    }

    #[test]
    fn a_map_is_not_a_list() {
        let err =
            violations(engine().eval::<Dynamic>("#{}").expect("a map")).expect_err("not a list");
        assert!(matches!(err, CheckError::NotAList { .. }), "{err:?}");
    }
}
