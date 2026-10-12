//! `Ruleset` and its types: a rules file's `RULESET` map as Rust, read by
//! `from_data_block` and checked against the rules the types cannot carry.
//! Plan: `aatf_rules.md` "The ruleset schema", "Loading and validation".

use std::collections::BTreeMap;

use pes_savefile::interchange::team_toml::labels::{
    ABILITY_STATS, COM_STYLES, PLAY_STYLES, POSITIONS, SKILLS,
};
use pes_version::PesVersion;
use serde::Deserialize;

pub(crate) mod check;
#[cfg(test)]
mod tests;

pub use check::{IdKind, LoadError, LoadRule, RuleError};

// ---------------------------------------------------------------------------
// The types (`aatf_rules.md` "The ruleset schema", the type block)

/// The id a tier, bracket, system or card group answers to: the ruleset's
/// own string.
pub type TierId = String;
/// The id a height bracket answers to.
pub type BracketId = String;
/// The id a height system answers to.
pub type SystemId = String;
/// The id a `card_groups` entry answers to; `com` is reserved for the seven
/// COM playing styles.
pub type GroupId = String;

/// A ruleset's data block: the tiers, card economy, heights, universal
/// rules, specials and suggestions of one AATF rules file.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ruleset {
    /// The schema version of this file; this build reads 1.
    pub schema: u32,
    /// The ruleset's name, shown in the tools.
    pub name: String,
    /// The squad's size in players.
    pub squad_size: u8,
    /// Which stats and targets the check reads.
    #[serde(default)]
    pub stats: StatRules,
    /// Highest first: recognition takes the first tier that fits, and the quick-action
    /// buttons follow this order.
    #[serde(default)]
    pub tiers: Vec<Tier>,
    /// Named card sets a tier can make free (`Tier::free`); `com` is reserved for the
    /// seven COM playing styles.
    #[serde(default)]
    pub card_groups: BTreeMap<GroupId, Vec<Card>>,
    /// The height brackets and the systems that quota them.
    #[serde(default)]
    pub heights: Heights,
    /// Team-wide rules independent of tiers.
    pub universal: Universal,
    /// Applied in order: numeric effects add up, replacing effects take the last value.
    #[serde(default)]
    pub specials: Vec<Special>,
    /// Team tactics settings required on the listed versions.
    #[serde(default)]
    pub team_settings: Vec<TeamSettings>,
    /// Allowances whose unused part is reported as a suggestion: `cards`, `a_positions`,
    /// `weak_foot`, `injury_resistance`, `height`, `free_cards`, `choices`, or a group id
    /// (`com` included).
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// Which stats and targets the check reads.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatRules {
    /// The stats whose maximum is a player's rating, compared with each tier's expected rating.
    #[serde(default)]
    pub rating: Vec<Stat>,
    /// Targets a player may sit below: ability stats, `form`, `injury_resistance`,
    /// `weak_foot_usage`, `weak_foot_accuracy`. Every other target must be met exactly.
    #[serde(default)]
    pub at_most: Vec<String>,
}

/// One tier a player can be recognized as (highest first).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    /// The tier id, the ruleset's own string.
    pub id: TierId,
    /// Shown in the editor and with the violations it causes.
    pub label: String,
    /// Exact number per team. `None` on a tier without `counts_as`: the rest of the squad
    /// (one such tier at most).
    #[serde(default)]
    pub count: Option<u8>,
    /// The tier whose count this tier's players fill (the goalkeeper tier counts as regular).
    #[serde(default)]
    pub counts_as: Option<TierId>,
    /// What a player must also meet to be this tier; may not name tiers.
    #[serde(default)]
    pub when: Condition,
    /// Target of every ability stat not in `stats`.
    pub rate: u8,
    /// Per-stat targets overriding `rate`.
    #[serde(default)]
    pub stats: BTreeMap<Stat, u8>,
    /// Target form.
    #[serde(default)]
    pub form: u8,
    /// Target injury resistance.
    #[serde(default)]
    pub injury_resistance: u8,
    /// Weak-foot target.
    #[serde(default)]
    pub weak_foot: WeakFoot,
    /// Cards a player pays for: skills and COM styles that no free allowance covers.
    #[serde(default)]
    pub cards: u8,
    /// At least this many paid skill cards, each extra A position counting as one.
    #[serde(default)]
    pub min_cards: u8,
    /// Free cards per group.
    #[serde(default)]
    pub free: BTreeMap<GroupId, u8>,
    /// A positions included, the registered one counted; each extra A costs a card.
    #[serde(default)]
    pub a_positions: u8,
}

/// A weak-foot target (usage and accuracy).
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeakFoot {
    /// Weak foot usage.
    #[serde(default)]
    pub usage: u8,
    /// Weak foot accuracy.
    #[serde(default)]
    pub accuracy: u8,
}

/// The height brackets and the systems that quota them.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Heights {
    /// Ascending. A player's bracket is the first that takes their height less any
    /// `height_allowance`; no bracket taking it is an illegal height. Empty: heights are
    /// constrained by tier conditions and specials alone.
    #[serde(default)]
    pub brackets: Vec<Bracket>,
    /// The first whose `any_player` holds sets the quotas; the last has
    /// none or an empty one.
    #[serde(default)]
    pub systems: Vec<HeightSystem>,
}

/// One height bracket: an `up_to` cap or a list of exact heights.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(try_from = "RawBracket")]
pub struct Bracket {
    /// The bracket id.
    pub id: BracketId,
    /// Shown in the editor.
    pub label: String,
    /// `up_to` or `exactly`, one of them required.
    pub heights: BracketHeights,
}

/// `up_to: 175` or `exactly: [185]`, spelled directly in the bracket's
/// map; `exactly` lists at least one height.
#[derive(Debug, Clone, PartialEq)]
pub enum BracketHeights {
    /// At most this, above the previous bracket.
    UpTo(u8),
    /// Exactly one of these heights.
    Exactly(Vec<u8>),
}

/// The bracket's on-disk shape (`heights` flattened: `up_to` or `exactly`
/// inside the bracket's own map).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawBracket {
    id: BracketId,
    label: String,
    up_to: Option<u8>,
    exactly: Option<Vec<u8>>,
}

impl TryFrom<RawBracket> for Bracket {
    type Error = String;

    fn try_from(raw: RawBracket) -> Result<Self, Self::Error> {
        let heights = match (raw.up_to, raw.exactly) {
            (Some(up_to), None) => BracketHeights::UpTo(up_to),
            (None, Some(exactly)) => {
                if exactly.is_empty() {
                    return Err(format!("{}: `exactly` lists no height", raw.id));
                }
                BracketHeights::Exactly(exactly)
            }
            (up_to, exactly) => {
                return Err(format!(
                    "{}: expected exactly one of `up_to` or `exactly` (got up_to: {}, exactly: {})",
                    raw.id,
                    up_to.is_some(),
                    exactly.is_some()
                ));
            }
        };
        Ok(Bracket {
            id: raw.id,
            label: raw.label,
            heights,
        })
    }
}

/// One height system: the quotas a bracket set takes when `any_player` holds.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeightSystem {
    /// The system id.
    pub id: SystemId,
    /// Shown in the editor.
    pub label: String,
    /// Holds when some player meets it; may name heights and positions only.
    #[serde(default)]
    pub any_player: Option<Condition>,
    /// Exact count per bracket; a bracket absent here is unconstrained.
    #[serde(default)]
    pub quotas: BTreeMap<BracketId, u8>,
}

/// Team-wide rules independent of tiers.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Universal {
    /// The registered position must be rated A.
    #[serde(default)]
    pub registered_position_a: bool,
    /// Presets (1 to 3) in which a starting-eleven player may be fielded only at positions he is
    /// rated A at, in every formation the game can field from them (counted as for
    /// `Condition::fielded_only_at`). Empty: no such rule.
    #[serde(default)]
    pub fielded_at_a: Vec<u8>,
    /// B ratings are not allowed.
    #[serde(default)]
    pub no_b_ratings: bool,
    /// B ratings count toward the A-position allowance, like extra A positions.
    #[serde(default)]
    pub b_uses_a_allowance: bool,
    /// A GK rating is never an outfield player's second A.
    #[serde(default)]
    pub gk_not_second_a: bool,
    /// Inclusive age bounds.
    #[serde(default)]
    pub age: Option<(u8, u8)>,
    /// Weight within `max(floor, height - below)..=height - above`.
    #[serde(default)]
    pub weight: Option<WeightRule>,
    /// A captain is required.
    #[serde(default)]
    pub captain_required: bool,
    /// At least this many goalkeepers.
    #[serde(default)]
    pub min_goalkeepers: u8,
    /// Skill cards a player may hold, free ones included (the game's limit: 10; VTL11 uses 11
    /// on PES 21).
    pub max_skill_cards: u8,
    /// COM playing styles at most, unset for no limit.
    #[serde(default)]
    pub max_com_styles: Option<u8>,
}

/// Weight within `max(floor, height - below)..=height - above` (Autumn 26: 30, 129, 81).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeightRule {
    /// Lowest allowed weight.
    pub floor: u8,
    /// Subtracted from height for the lower bound.
    pub below: u8,
    /// Subtracted from height for the upper bound.
    pub above: u8,
}

/// Every present field must hold; an empty list or `None` always holds.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    /// Of these tiers.
    #[serde(default)]
    pub tiers: Vec<TierId>,
    /// Registered position.
    #[serde(default)]
    pub positions: Vec<Position>,
    /// Of these height systems.
    #[serde(default)]
    pub systems: Vec<SystemId>,
    /// Of these height brackets.
    #[serde(default)]
    pub brackets: Vec<BracketId>,
    /// Height in cm, inclusive, before any allowance.
    #[serde(default)]
    pub height_min: Option<u8>,
    /// Height in cm, inclusive, before any allowance.
    #[serde(default)]
    pub height_max: Option<u8>,
    /// Is or is not the captain.
    #[serde(default)]
    pub captain: Option<bool>,
    /// Any of these playing styles.
    #[serde(default)]
    pub playing_styles: Vec<PlayStyle>,
    /// Holds any of these cards.
    #[serde(default)]
    pub holds: Vec<Card>,
    /// Rated A (or B) at any of these positions.
    #[serde(default)]
    pub rated_a_at: Vec<Position>,
    /// Rated B at any of these positions.
    #[serde(default)]
    pub rated_b_at: Vec<Position>,
    /// The name starts with one of these colour codes (the eight characters after `\x11c`,
    /// `cc9900ff`; see `pes_savefile`'s "Name colour codes").
    #[serde(default)]
    pub name_colors: Vec<String>,
    /// Fielded at this position in every formation the game can field: all three presets,
    /// the three formations of a fluid preset, the kick-off formation of the others. A player
    /// outside the starting eleven: registered there.
    #[serde(default)]
    pub fielded_only_at: Option<Position>,
    /// Holds only when this fails.
    #[serde(default)]
    pub not: Option<Box<Condition>>,
}

impl Condition {
    /// Whether no field is set: a condition that holds for every player.
    pub(crate) fn is_empty(&self) -> bool {
        *self == Condition::default()
    }
}

/// One special: players matching `when` get `then`'s effects or `choice`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Special {
    /// Shown in the editor and with the violations it causes.
    pub label: String,
    /// Whom the special applies to.
    #[serde(default)]
    pub when: Condition,
    /// The effects, applied in order.
    #[serde(default)]
    pub then: Effects,
    /// Of the `forbid` and `require_cards` violations; default `error`.
    #[serde(default)]
    pub severity: Severity,
}

/// Every field optional. A `PerTier` value is one number for every tier or a map by tier
/// id, a tier absent from the map being unaffected.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effects {
    /// Added to the target of every ability stat (`all`) or of one stat.
    #[serde(default)]
    pub stat_bonus: BTreeMap<StatOrAll, PerTier<i8>>,
    /// Replaces one stat's target.
    #[serde(default)]
    pub stat_target: BTreeMap<Stat, PerTier<u8>>,
    /// Added to the cards limit.
    #[serde(default)]
    pub cards: Option<PerTier<i8>>,
    /// Free cards per group; numeric effects add up across specials.
    #[serde(default)]
    pub free: BTreeMap<GroupId, PerTier<i8>>,
    /// These cards cost nothing.
    #[serde(default)]
    pub free_cards: Vec<Card>,
    /// These cards are required.
    #[serde(default)]
    pub require_cards: Vec<Card>,
    /// Added to the A-position allowance.
    #[serde(default)]
    pub a_positions: Option<PerTier<i8>>,
    /// Replaces the A-position cap.
    #[serde(default)]
    pub max_a_positions: Option<u8>,
    /// Replaces the tier's limits.
    #[serde(default)]
    pub weak_foot: Option<WeakFoot>,
    /// Centimetres subtracted from the height before bracketing.
    #[serde(default)]
    pub height_allowance: Option<u8>,
    /// A violation with this text.
    #[serde(default)]
    pub forbid: Option<String>,
    /// The first option the player uses, or none: an option is used when, without it, the
    /// player exceeds a limit the option raises; a `stat_bonus` raises the stat targets, so it is
    /// used when, without it, the player's rating is above the expected rating. A one-option
    /// choice is an optional effect (VGL25's manlet bonus).
    #[serde(default)]
    pub choice: Vec<Effects>,
}

/// Team tactics settings required on the listed versions (every version when empty); each
/// present field must have this value.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TeamSettings {
    /// Shown in the editor and with the violations it causes.
    pub label: String,
    /// The versions this rule applies to (written as numbers; every version when empty).
    #[serde(default)]
    pub versions: Vec<PesVersion>,
    /// Man marking set up on any preset (PES 16: the per-preset assignments; PES 17+: a
    /// defending advanced instruction of the man-marking kind).
    #[serde(default)]
    pub man_marking: Option<bool>,
    /// Any setting but off.
    #[serde(default)]
    pub auto_substitution: Option<bool>,
    /// Any setting but off.
    #[serde(default)]
    pub auto_offside_trap: Option<bool>,
    /// Any setting but off.
    #[serde(default)]
    pub auto_preset_change: Option<bool>,
    /// Any setting but off.
    #[serde(default)]
    pub auto_attack_defence_levels: Option<bool>,
    /// Banned formation shapes, as defender, midfielder and forward counts (`[3, 4, 3]`), in
    /// every formation the game can field from any preset. Defenders are CB, LB and RB;
    /// midfielders DMF, CMF, LMF, RMF and AMF; forwards LWF, RWF, SS and CF.
    #[serde(default)]
    pub forbidden_shapes: Vec<[u8; 3]>,
}

/// One number for every tier, or a map by tier id.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PerTier<T> {
    /// One number for every tier.
    All(T),
    /// By tier id; a tier absent from the map is unaffected.
    ByTier(BTreeMap<TierId, T>),
}

/// The `stat_bonus` map key: `all` for every ability stat or one stat's name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(try_from = "String")]
pub enum StatOrAll {
    /// Every ability stat.
    All,
    /// One stat.
    Stat(Stat),
}

impl TryFrom<String> for StatOrAll {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if name == "all" {
            Ok(StatOrAll::All)
        } else {
            Stat::try_from(name).map(StatOrAll::Stat)
        }
    }
}

/// Of the `forbid` and `require_cards` violations; default `error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Counted as an error.
    #[default]
    Error,
    /// A warning.
    Warning,
    /// A suggestion, reported only on request.
    Suggestion,
}

// ---------------------------------------------------------------------------
// The closed sets (spellings shared with `team.toml` via `labels`)

/// One ability stat, the index of its name in `labels::ABILITY_STATS` (the
/// `PlayerStats` field order Team TOML spells).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct Stat(pub(crate) u8);

/// A registered or playable position, its index in `labels::POSITIONS`
/// (index = the stored position byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct Position(pub(crate) u8);

/// A playing style, its index in `labels::PLAY_STYLES`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct PlayStyle(pub(crate) u8);

/// A card: a skill or a COM playing style, its index in the group's table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub enum Card {
    /// A skill (`labels::SKILLS`' index).
    Skill(u8),
    /// A COM playing style (`labels::COM_STYLES`' index).
    ComStyle(u8),
}

/// The index of `name` in `table` (a label table's index is its stored
/// value). `u8::try_from` so the table's width is the check's own.
fn label_index(table: &[&str], name: &str) -> Option<u8> {
    table
        .iter()
        .position(|label| *label == name)
        .map(|i| u8::try_from(i).expect("a label table holds fewer than 256 entries"))
}

impl Stat {
    /// The stat's `stats.<name>` spelling.
    pub fn label(self) -> &'static str {
        ABILITY_STATS[usize::from(self.0)]
    }
}

impl TryFrom<String> for Stat {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        label_index(&ABILITY_STATS, &name)
            .map(Stat)
            .ok_or_else(|| format!("{name:?}: unknown stat"))
    }
}

impl Position {
    /// The position's `GK`/`CB`/`CF` spelling.
    pub fn label(self) -> &'static str {
        POSITIONS[usize::from(self.0)]
    }
}

impl TryFrom<String> for Position {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        label_index(&POSITIONS, &name)
            .map(Position)
            .ok_or_else(|| format!("{name:?}: unknown position"))
    }
}

impl PlayStyle {
    /// The style's canonical label.
    pub fn label(self) -> &'static str {
        PLAY_STYLES[usize::from(self.0)]
    }
}

impl TryFrom<String> for PlayStyle {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        label_index(&PLAY_STYLES, &name)
            .map(PlayStyle)
            .ok_or_else(|| format!("{name:?}: unknown playing style"))
    }
}

impl Card {
    /// The card's name in its group.
    pub fn label(self) -> &'static str {
        match self {
            Card::Skill(index) => SKILLS[usize::from(index)],
            Card::ComStyle(index) => COM_STYLES[usize::from(index)],
        }
    }
}

impl TryFrom<String> for Card {
    type Error = String;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        label_index(&SKILLS, &name)
            .map(Card::Skill)
            .or_else(|| label_index(&COM_STYLES, &name).map(Card::ComStyle))
            .ok_or_else(|| format!("{name:?}: unknown card"))
    }
}

// ---------------------------------------------------------------------------
// Loading and the load-time rules

/// The compile-and-run step `from_data_block` and `Rules::load` share:
/// `engine()`'s AST and the scope the top level ran in.
pub(crate) fn run_top_level(
    text: &str,
) -> Result<(rhai::Engine, rhai::AST, rhai::Scope<'static>), LoadError> {
    // The one engine every script runs on; a data block evaluates its
    // literals and touches nothing else.
    let engine = crate::engine::engine();
    let ast = engine.compile(text).map_err(|e| LoadError::Parse {
        line: e.position().line().unwrap_or(0),
        column: e.position().position().unwrap_or(0),
        message: e.to_string(),
    })?;
    let mut scope = rhai::Scope::new();
    engine
        .run_ast_with_scope(&mut scope, &ast)
        .map_err(|e| LoadError::Parse {
            line: e.position().line().unwrap_or(0),
            column: e.position().position().unwrap_or(0),
            message: e.to_string(),
        })?;
    Ok((engine, ast, scope))
}

/// Step 2 of "Loading and validation": the `RULESET` map of the run's
/// scope into `Ruleset`, its schema and the load-time rules.
pub(crate) fn ruleset_from_scope(scope: &rhai::Scope) -> Result<Ruleset, LoadError> {
    let Some((_, constant, dynamic)) = scope.iter().find(|(name, _, _)| *name == "RULESET") else {
        return Err(LoadError::NoRuleset);
    };
    // A `let RULESET` leaves a non-constant entry a function cannot
    // read (`global::RULESET` would be `ErrorVariableNotFound`).
    if !constant || !dynamic.is_map() {
        return Err(LoadError::NoRuleset);
    }
    let ruleset: Ruleset = rhai::serde::from_dynamic(&dynamic).map_err(|e| LoadError::Schema {
        path: String::new(),
        message: e.to_string(),
    })?;
    if ruleset.schema != 1 {
        return Err(LoadError::UnknownSchema {
            schema: ruleset.schema,
        });
    }
    let rules = ruleset.check();
    if !rules.is_empty() {
        return Err(LoadError::Rules(rules));
    }
    Ok(ruleset)
}

impl Ruleset {
    /// Reads a data block: Rhai text defining `const RULESET = #{ ... };`.
    /// The parse, `RULESET`, schema and version checks stop at the first
    /// error; `Rules` reports every load-time rule the text breaks.
    pub fn from_data_block(text: &str) -> Result<Ruleset, LoadError> {
        run_top_level(text).and_then(|(_, _, scope)| ruleset_from_scope(&scope))
    }
}
