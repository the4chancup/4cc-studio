//! `aatf`: the 4cc player rules (the community's "Auto-ATF" check). A
//! ruleset is **data** — one Rhai map naming the tiers, the card economy,
//! the heights and a list of conditional "specials" — read here into
//! `Ruleset` and checked against the rules the types cannot carry. The
//! host is written: `TeamView`/`PlayerView` and the script engine. What
//! is not: the interpreter (`check_team`, `tier_values`, `card_limits`)
//! and `apply_tier` (`aatf_rules.md` "The interpreter").
//!
//! The host reads a team as a `TeamView` of `PlayerView`s on the game's
//! own scale, and every script runs on the one engine of `engine.rs`.
//! The script API a ruleset sees, on `Player`: `slot`, `age`, `height`,
//! `weight`; `form`, `injury_resistance`, `weak_foot_usage`,
//! `weak_foot_accuracy` (the game's scale); `position`, `playing_style`,
//! `name_color` (label or `()`); `cards` (the held skills and COM styles'
//! labels); `stat(name)` (the ability stat's value, `()` where the
//! version lacks it); `rating_at(pos)` (`"A"`, `"B"`, `"C"`, `"none"`).
//! On `Team`: `players` (the roster in slot order), `captain` (the slot,
//! `()` for none), `version` (the short number);
//! `fielded_positions(player, preset)` (labels, once each),
//! `fielded_only_at(player, pos)`, `formation_shapes()` (`#{ preset,
//! formation, shape: [defenders, midfielders, forwards] }`).

/// The one engine every script runs on: packages, accessors, limits.
mod engine;
/// `Ruleset` and its types, deserialization and the load-time rules.
mod ruleset;
/// `TeamView`/`PlayerView` from `pes_savefile`'s model, the tactics answers.
mod team;

pub use team::{PlayerView, TeamView, TeamViewError};

pub use ruleset::{
    Bracket, BracketHeights, BracketId, Card, Condition, Effects, GroupId, HeightSystem, Heights,
    IdKind, LoadError, LoadRule, PerTier, PlayStyle, Position, RuleError, Ruleset, Severity,
    Special, Stat, StatOrAll, StatRules, SystemId, TeamSettings, Tier, TierId, Universal, WeakFoot,
    WeightRule,
};
