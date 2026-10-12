//! `aatf`: the 4cc player rules (the community's "Auto-ATF" check). A
//! ruleset is **data** — one Rhai map naming the tiers, the card economy,
//! the heights and a list of conditional "specials" — read here into
//! `Ruleset` and checked against the rules the types cannot carry. The
//! interpreter and the host that run the check are not written yet
//! (`aatf_rules.md` "The interpreter", "The host").

/// `Ruleset` and its types, deserialization and the load-time rules.
mod ruleset;

pub use ruleset::{
    Bracket, BracketHeights, BracketId, Card, Condition, Effects, GroupId, HeightSystem, Heights,
    IdKind, LoadError, LoadRule, PerTier, PlayStyle, Position, RuleError, Ruleset, Severity,
    Special, Stat, StatOrAll, StatRules, SystemId, TeamSettings, Tier, TierId, Universal, WeakFoot,
    WeightRule,
};
