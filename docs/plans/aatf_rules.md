# 4cc Studio — AATF rules plan

The AATF rules (the 4cc player rules a team's savefile data must satisfy; the community calls the
check "Auto-ATF") are checked and applied by `libs/aatf`. A ruleset is **data**: one Rhai map
naming the tiers, the heights, the card economy and a list of conditional "specials". A **generic
interpreter**, written in Rhai and carried inside every rules file, evaluates any such map. The
official ruleset and every invitational's are the same kind of file, differing only in their data.

Consumers: the [Save editor](save_editor.md) (the AATF check, the Make Gold/Silver/… quick
actions, the card pickers' remaining-count badges in `libs/team_widgets`), the [Team
creator](team_creator.md) (legal defaults, the check before writing) and the [Ruleset
editor](ruleset_editor.md) (which writes the data). Platform context is in the [core plan](core/README.md).

---

## Background

4ccEditor's AATF tool (`aatf.cpp`) hardcodes both the parameters and the logic in C++, so a ruleset
change is a recompile. Its git history (46 commits, 2019–2026) shows what changes between cups:

- **Numbers**, most editions: weak-foot limits, bracket quotas, tier rates, height thresholds.
- **One to three "specials" per cup**, every one of the form *players matching X get Y*: "non-medals
  registered at CB, LB or RB get +5 defensive prowess" (Spring 24), "Malicia is mandatory on every
  non-medal player" (Summer 24), "GKs and medals cannot be giants" (Autumn 24), "Green silver
  194s at −3", "Red non-medal CBs fielded only at CB may be 189" (Autumn 26), "trade a card for
  4/4 weak foot" (FAG13), "medals get a free A position *or* a free COM style" (Autumn 26).
- **Engine changes**, rarely: B ratings checked, playing-style ranges per version, which stats a
  version has.

Invitationals already run their own rulesets on forks of the editor. VGL26 (4ccEditor-VGL, branch
`vgl26`, `stats.h` + `aatf_single_vgl`) is a complete one of a different shape: five tiers
including a "buffed" one, per-stat targets per tier, one exact height per tier and no height
brackets, a different rating stat set, free A positions per tier, all trick cards free, and a free
extra card for the captain. It also reports *suggestions*: allowances a player does not use
(fewer cards, COM styles or A positions than allowed, weak foot or injury resistance below the
maximum, a captain without the free Captaincy card), shown only on request and never counted as
errors. 4ccEditor has the same idea behind its `eCheck` warnings, hardcoded off.

The schema below is the union of what both rulesets and the specials history need, and both
rulesets are its acceptance fixtures ("Verification").

---

## The rules file

One self-contained `.rhai` file is one ruleset, in three sections:

```rhai
// 4cc Studio AATF ruleset. Open it in the Ruleset editor to change it.
const RULESET = #{ ... };

// ==== LOGIC (ruleset-logic 1): written by 4cc Studio; the Ruleset editor replaces this section ====
fn check_team(team) { ... }
fn tier_values(player, tier, team) { ... }
fn card_limits(player, tier, team) { ... }
// ... the interpreter's own functions

// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====
fn custom_checks(team) { [] }
```

- **`RULESET`** is the data ("The ruleset schema"). The Ruleset editor owns it and rewrites it
  whole; a hand edit is allowed and validated like any other.
- **The logic** is the generic interpreter ("The interpreter"), copied into every file so a
  ruleset behaves the same on every Studio build: a file made under one logic version keeps
  running it until someone updates it. Its banner names the version; "Loading and validation"
  describes how the host recognizes it.
- **Custom checks** are the escape hatch: a rule the schema cannot express yet, written in Rhai by
  someone who can, without a Studio release. `check_team` calls `custom_checks(team)` when the file
  defines it and appends what it returns.

Rhai is a small scripting language written in pure Rust for embedding: scripts reach only what the
host registers, cannot touch the filesystem or network, and run under operation and call-depth
limits, so a broken file cannot hang the tool.

---

## The ruleset schema

`RULESET` deserializes into `Ruleset` (`rhai::serde::from_dynamic`; an unknown key is an error).
Tier, bracket, system and card-group ids are the ruleset's own strings. `Stat`, `Skill` and
`Position` are closed sets, spelled as `pes_savefile` spells them: stats are `PlayerStats`'
ability field names (`attacking_prowess` … `aggression`), skills and positions the Team TOML labels
(`interchange/team_toml/labels.rs`: `malicia`, `captaincy`, `GK`, `CB`, …).

```rust
pub struct Ruleset {
    pub schema: u32,
    pub name: String,
    pub squad_size: u8,
    pub stats: StatRules,
    /// Highest first: recognition takes the first tier that fits, and the quick-action
    /// buttons follow this order.
    pub tiers: Vec<Tier>,
    /// Named skill sets a tier can make free (`Tier::free`); `com` is reserved for the
    /// seven COM playing styles.
    pub card_groups: BTreeMap<GroupId, Vec<Skill>>,
    pub heights: Heights,
    pub universal: Universal,
    /// Applied in order: numeric effects add up, replacing effects take the last value.
    pub specials: Vec<Special>,
    /// Allowances whose unused part is reported as a suggestion: `cards`, `a_positions`,
    /// `weak_foot`, `injury_resistance`, `height`, `free_cards`, or a group id (`com` included).
    pub suggestions: Vec<String>,
}

pub struct StatRules {
    /// The stats whose maximum is a player's rating, compared with each tier's expected rating.
    pub rating: Vec<Stat>,
    /// Stats that may sit below their target; every other ability stat must equal it.
    pub at_most: Vec<Stat>,
}

pub struct Tier {
    pub id: TierId,
    pub label: String,
    /// Exact number per team. `None` on a tier without `counts_as`: the rest of the squad
    /// (one such tier at most).
    pub count: Option<u8>,
    /// The tier whose count this tier's players fill (the goalkeeper tier counts as regular).
    pub counts_as: Option<TierId>,
    /// What a player must also meet to be this tier; may not name tiers.
    pub when: Condition,
    /// Target of every ability stat not in `stats`.
    pub rate: u8,
    pub stats: BTreeMap<Stat, u8>,
    pub form: u8,
    /// At most.
    pub injury_resistance: u8,
    /// At most.
    pub weak_foot: WeakFoot,
    /// Cards a player pays for: skills and COM styles that no free allowance covers.
    pub cards: u8,
    /// Free cards per group.
    pub free: BTreeMap<GroupId, u8>,
    /// A positions included, the registered one counted; each extra A costs a card.
    pub a_positions: u8,
}

pub struct WeakFoot {
    pub usage: u8,
    pub accuracy: u8,
}

pub struct Heights {
    /// Ascending. A player's bracket is the first that takes their height less any
    /// `height_allowance`; no bracket taking it is an illegal height. Empty: heights are
    /// constrained by tier conditions and specials alone.
    pub brackets: Vec<Bracket>,
    /// The first whose `any_player` holds sets the quotas; the last has none.
    pub systems: Vec<HeightSystem>,
}

pub struct Bracket {
    pub id: BracketId,
    pub label: String,
    pub heights: BracketHeights,
}

/// Spelled `up_to: 175` or `exactly: [185]` directly in the bracket's map.
pub enum BracketHeights {
    /// At most this, above the previous bracket.
    UpTo(u8),
    Exactly(Vec<u8>),
}

pub struct HeightSystem {
    pub id: SystemId,
    pub label: String,
    /// Holds when some player meets it; may name heights and positions only.
    pub any_player: Option<Condition>,
    /// Exact count per bracket; a bracket absent here is unconstrained.
    pub quotas: BTreeMap<BracketId, u8>,
}

pub struct Universal {
    pub registered_position_a: bool,
    pub no_b_ratings: bool,
    /// A GK rating is never an outfield player's second A.
    pub gk_not_second_a: bool,
    /// Inclusive.
    pub age: Option<(u8, u8)>,
    pub weight: Option<WeightRule>,
    pub captain_required: bool,
    pub min_goalkeepers: u8,
}

/// Weight within `max(floor, height - below)..=height - above` (Autumn 26: 30, 129, 81).
pub struct WeightRule {
    pub floor: u8,
    pub below: u8,
    pub above: u8,
}

/// Every present field must hold; an empty list or `None` always holds.
pub struct Condition {
    pub tiers: Vec<TierId>,
    /// Registered position.
    pub positions: Vec<Position>,
    pub systems: Vec<SystemId>,
    pub brackets: Vec<BracketId>,
    /// Height in cm, inclusive, before any allowance.
    pub height_min: Option<u8>,
    pub height_max: Option<u8>,
    pub captain: Option<bool>,
    /// Fielded at this position in every formation the game can field: all three presets,
    /// the three formations of a fluid preset, the kick-off formation of the others. A player
    /// outside the starting eleven: registered there.
    pub fielded_only_at: Option<Position>,
    pub not: Option<Box<Condition>>,
}

pub struct Special {
    /// Shown in the editor and with the violations it causes.
    pub label: String,
    pub when: Condition,
    pub then: Effects,
    /// Of the `forbid` and `require_cards` violations; default `error`.
    pub severity: Severity,
}

/// Every field optional. A `PerTier` value is one number for every tier or a map by tier
/// id, a tier absent from the map being unaffected.
pub struct Effects {
    /// Added to the target of every ability stat (`all`) or of one stat.
    pub stat_bonus: BTreeMap<StatOrAll, PerTier<i8>>,
    /// Replaces one stat's target.
    pub stat_target: BTreeMap<Stat, PerTier<u8>>,
    pub cards: Option<PerTier<i8>>,
    pub free: BTreeMap<GroupId, PerTier<i8>>,
    /// These skills cost nothing.
    pub free_cards: Vec<Skill>,
    pub require_cards: Vec<Skill>,
    pub a_positions: Option<PerTier<i8>>,
    pub max_a_positions: Option<u8>,
    /// Replaces the tier's limits.
    pub weak_foot: Option<WeakFoot>,
    /// Centimetres subtracted from the height before bracketing.
    pub height_allowance: Option<u8>,
    /// A violation with this text.
    pub forbid: Option<String>,
    /// The first option the player uses, or none: an option is used when, without it, the
    /// player exceeds a limit the option raises.
    pub choice: Vec<Effects>,
}

pub enum PerTier<T> {
    All(T),
    ByTier(BTreeMap<TierId, T>),
}

pub enum StatOrAll {
    All,
    Stat(Stat),
}

pub enum Severity {
    Error,
    Warning,
    Suggestion,
}
```

Rules the types cannot carry, checked on load ("Loading and validation"): ids unique within their
kind, and every id a condition, effect, quota, `counts_as` or `suggestions` entry names exists; a
tier with `counts_as` has no `count`; tier counts fit `squad_size`; a special with
`height_allowance` names no bracket, since brackets are computed from the allowance.

### How a player is read

The order is fixed, so every condition sees settled inputs:

1. **Height system** (team): the first system whose `any_player` holds.
2. **Tier** (player): for each tier in list order, the specials are evaluated as if the player
   were that tier: allowances first, then the bracket, then every other effect. The tier is the
   first whose `when` holds and whose *expected rating* (the highest target among `stats.rating`)
   is at most the player's rating. If none fits, the last tier whose `when` holds is taken; if no
   tier's `when` holds, the player gets `NoTier` and no further per-player check.
3. **Checks** (player), against the recognized tier's targets and limits.
4. **Team checks**: tier counts (`counts_as` folded in), the system's bracket quotas, captain,
   goalkeepers, then `custom_checks`.

Recognition by expected rating is what lets a special move a tier's threshold: a Green silver
giant's target is 89, so a rating of 89 reads as silver, not bronze.

### The card economy

A player's **paid cards** are their skills plus COM styles, less the free ones held: per group,
up to that group's free count; every held skill in `free_cards`. The **limit** is the tier's
`cards` plus every `cards` effect, less one per A position beyond the allowance (`a_positions`
plus effects). Paid cards over the limit is a violation; so is a required card not held, more A
positions than `max_a_positions`, and more than ten skill cards (the game's limit, COM styles not
counted; built in, as are the version's valid registered-position and playing-style ranges).

---

## The official ruleset

The default file, embedded in `libs/aatf`. Transcribed from `aatf.cpp` at 4ccEditor's
`Autumn_2026_AATF` tip `8f01926` (the file last changed in `f5e7b3e`). The card groups take
`aatf.cpp`'s skill indices as canonical slots; Phase 5 verifies that against `pes_savefile`'s
schema.

```rhai
const RULESET = #{
    schema: 1,
    name: "Autumn 26",
    squad_size: 23,

    stats: #{
        rating: ["dribbling", "goalkeeping", "finishing", "low_pass", "lofted_pass", "heading",
                 "swerve", "catching", "clearing", "reflexes", "coverage", "body_control",
                 "physical_contact", "kicking_power", "explosive_power", "ball_control",
                 "ball_winning", "jump", "place_kicking", "stamina", "speed", "aggression"],
        at_most: ["attacking_prowess", "defensive_prowess"],
    },

    tiers: [
        #{ id: "gold", label: "Gold", count: 1, rate: 99, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 6, free: #{ tricks: 3, com: 2 }, a_positions: 1 },
        #{ id: "silver", label: "Silver", count: 2, rate: 92, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 5, free: #{ tricks: 3, com: 1 }, a_positions: 1 },
        #{ id: "bronze", label: "Bronze", count: 2, rate: 86, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 4, free: #{ tricks: 3, com: 1 }, a_positions: 1 },
        #{ id: "goalkeeper", label: "Goalkeeper", counts_as: "regular", when: #{ positions: ["GK"] },
           rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 2, free: #{}, a_positions: 1 },
        #{ id: "regular", label: "Regular", rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 3, free: #{ tricks: 2 }, a_positions: 1 },
    ],

    card_groups: #{
        tricks: ["scissors_feint", "flip_flap", "marseille_turn", "sombrero", "cut_behind_and_turn",
                 "scotch_move", "rabona", "double_touch", "crossover_turn", "step_on_skill",
                 "no_look_pass"],
    },

    heights: #{
        brackets: [
            #{ id: "manlet", label: "Manlet", up_to: 175 },
            #{ id: "mid",    label: "Mid",    up_to: 180 },
            #{ id: "tall",   label: "Tall",   exactly: [185] },
            #{ id: "giant",  label: "Giant",  exactly: [194] },
            #{ id: "giga",   label: "Giga",   exactly: [199] },
        ],
        systems: [
            #{ id: "green", label: "Green", any_player: #{ height_min: 194 },
               quotas: #{ giga: 0, giant: 6, tall: 6, mid: 5, manlet: 6 } },
            #{ id: "red", label: "Red",
               quotas: #{ giga: 0, giant: 0, tall: 10, mid: 7, manlet: 6 } },
        ],
    },

    universal: #{
        registered_position_a: true, no_b_ratings: true, gk_not_second_a: true,
        age: [15, 50], weight: #{ floor: 30, below: 129, above: 81 },
        captain_required: true, min_goalkeepers: 1,
    },

    specials: [
        #{ label: "Malicia is a free card", then: #{ free_cards: ["malicia"] } },
        #{ label: "The captain's Captaincy card is free", when: #{ captain: true },
           then: #{ free_cards: ["captaincy"] } },
        #{ label: "Medals cannot be goalkeepers",
           when: #{ tiers: ["gold", "silver", "bronze"], positions: ["GK"] },
           then: #{ forbid: "Medals cannot play as GK" } },
        #{ label: "Goalkeepers at 189 count as tall",
           when: #{ positions: ["GK"], height_min: 189, height_max: 189 },
           then: #{ height_allowance: 4 } },
        #{ label: "Tall goalkeepers are 189",
           when: #{ positions: ["GK"], height_min: 181, height_max: 188 },
           then: #{ forbid: "GKs in this bracket must be 189cm" } },
        #{ label: "Goalkeepers below giant", severity: "warning",
           when: #{ positions: ["GK"], height_min: 194 },
           then: #{ forbid: "GK heights cannot be 194cm or more" } },
        #{ label: "Gold below giant", severity: "warning",
           when: #{ tiers: ["gold"], height_min: 194 },
           then: #{ forbid: "Gold heights cannot be 194cm or more" } },
        #{ label: "Red CBs up to 189",
           when: #{ tiers: ["regular"], systems: ["red"], positions: ["CB"], fielded_only_at: "CB" },
           then: #{ height_allowance: 4 } },
        #{ label: "Manlets", when: #{ brackets: ["manlet"] },
           then: #{ cards: 1, a_positions: 1, weak_foot: #{ usage: 4, accuracy: 4 } } },
        #{ label: "Red manlet bonus", when: #{ systems: ["red"], height_max: 175 },
           then: #{ stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5, bronze: 3, silver: 2 } } } },
        #{ label: "Green silver giants", when: #{ tiers: ["silver"], systems: ["green"], height_min: 194 },
           then: #{ stat_bonus: #{ all: -3 } } },
        #{ label: "Medals: a free A position or a free COM style",
           when: #{ tiers: ["gold", "silver", "bronze"] },
           then: #{ choice: [#{ free: #{ com: 1 } }, #{ a_positions: 1 }] } },
    ],

    suggestions: ["cards", "tricks", "com", "weak_foot", "injury_resistance"],
};
```

Where the data reads differently from `aatf.cpp`, on purpose: `aatf.cpp` prints its two height
warnings for gold (above 194, and at 194) separately; here they are one. A player whose rating fits
no tier's target is placed by expected rating rather than by `aatf.cpp`'s thresholds, so an illegal
player can be reported against a different tier; both report the player.

## VGL26

The second embedded ruleset: a starting template in the Ruleset editor and the schema's second
fixture, transcribed from 4ccEditor-VGL's `vgl26` branch (tip `ad378d2`; `stats.h`, and
`aatf_single_vgl` in `aatf.cpp`). What it exercises that the official file does not:

| Feature | VGL26 |
|---|---|
| Tiers | gold 1, silver 2, bronze 2, buffed 8, regular 10; goalkeeper `counts_as` regular |
| Per-stat targets | each tier's `rate` (99, 95, 91, 85, 80; 72 for goalkeepers) plus `stats` overrides (silver: finishing, heading and attacking prowess 99, defensive prowess 60, …) |
| Tier recognition | each tier's `when` fixes one height (gold and silver 195, bronze 190, buffed and goalkeeper 185, regular 180); no brackets, no systems |
| Rating stats | the official set less finishing, heading and aggression |
| Cards | `tricks` (VGL's set, Heading and Malicia included) free up to 12; free COM 2 or 1; `a_positions` 2 (goalkeeper 1); the captain's Captaincy free and one extra card (`cards: 1` for `captain: true`) |
| Suggestions | `cards`, `com`, `a_positions`, `weak_foot`, `injury_resistance`, `free_cards` |

---

## The interpreter

The logic section implements the three functions the host calls, over `global::RULESET` and the
host's registered accessors:

- `check_team(team) -> [violation]`: steps 1–4 of "How a player is read" for every player.
- `tier_values(player, tier, team) -> map`: the targets a player of `tier` must carry (per stat,
  form, injury resistance, weak foot), specials applied. Behind `apply_tier`.
- `card_limits(player, tier, team) -> map`: paid limit, free count per group, free and required
  cards, A positions allowed. Behind the card pickers' remaining-count badges.

A violation crosses back as a map the host converts into:

```rust
pub struct Violation {
    /// Roster slot; `None`: the team.
    pub slot: Option<u8>,
    pub severity: Severity,
    pub kind: ViolationKind,
}

pub enum ViolationKind {
    RegisteredNotA,
    BRating,
    GkSecondA,
    Age { age: u8, min: u8, max: u8 },
    Weight { weight: u8, min: u8, max: u8 },
    PositionRange,
    PlayingStyleRange,
    NoTier,
    Stat { stat: Stat, value: u8, target: u8, at_most: bool },
    Form { value: u8, target: u8 },
    InjuryResistance { value: u8, max: u8 },
    WeakFootUsage { value: u8, max: u8 },
    WeakFootAccuracy { value: u8, max: u8 },
    Cards { paid: u8, limit: u8 },
    SkillCardCap { count: u8 },
    RequiredCard { card: Skill },
    APositions { count: u8, max: u8 },
    Height { height: u8 },
    TierCount { tier: TierId, count: u8, expected: u8 },
    BracketQuota { system: SystemId, bracket: BracketId, count: u8, quota: u8 },
    Captain,
    Goalkeepers { count: u8, min: u8 },
    Special { label: String, text: String },
    Custom { text: String },
    /// A suggestion: an allowance listed in `suggestions` the player does not use up.
    Unused { allowance: String, used: u8, allowed: u8 },
}
```

These are findings (`AGENTS.md` "Fundamental concepts"): each tool maps a kind to its message
catalog; only `Special` and `Custom` carry text, the ruleset author's.

**Suggestions** need no authoring. Every allowance named in `suggestions` that a player does not
use up yields an `Unused` suggestion: paid cards under the limit, a group's free cards not all
held, a free card not held, fewer A positions than allowed, weak foot or injury resistance below
the maximum, a height below an `up_to` bracket's maximum. Stats in `at_most` are a manager's
choice, not an allowance, and are never suggested. Every surface that shows results (the Save
editor's AATF panel, the Team creator's) has a "Show suggestions" toggle, off by default, and lists
them apart from errors and warnings, uncounted.

## The host

`libs/aatf` compiles a file once per load, reads `RULESET` from the global scope, and calls the
three functions on demand. What a script sees is registered by the host and nothing else:

```rust
let mut engine = rhai::Engine::new();
engine.register_type::<PlayerEntry>()
      .register_get("slot", |p: &mut PlayerEntry| p.slot as i64)
      .register_get("height", |p: &mut PlayerEntry| p.height as i64)
      .register_fn("stat", |p: &mut PlayerEntry, name: &str| p.stat_by_name(name))
      .register_fn("playable_at", |p: &mut PlayerEntry, pos: &str| p.playable_at(pos));
engine.register_type::<TeamContext>()          // players, captain, version, tactics presets
      .register_get("players", |t: &mut TeamContext| t.players.clone())
      .register_fn("fielded_only_at", |t: &mut TeamContext, p: PlayerEntry, pos: &str| t.fielded_only_at(&p, pos));

engine.set_max_operations(MAX_OPERATIONS);
engine.set_max_call_levels(32);
```

`MAX_OPERATIONS` is set from the measured cost of the official ruleset on a full team, with a
tenfold margin. Closed sets cross the boundary as the labels the file uses (`"GK"`, `"A"`,
`"malicia"`); the Rust side keeps its enums and converts at the registration functions, nowhere
else.
Tactics questions (`fielded_only_at`) are answered in Rust from `pes_savefile`'s model, so the
starting-eleven lookup and the formation count, both wrong upstream until `f5e7b3e`, live in one
tested place. The registered
accessors are the script API, listed in `libs/aatf`'s crate doc; adding one is a plan edit to
this section, never a silent addition.

`apply_tier(player, tier, rules)` is the lib's one *writing* operation: it calls `tier_values`
and writes the result into the player (stats; form; injury resistance and weak foot at their
maximum). It is behind the Save editor's quick actions (one button per tier, in list order) and
the Team creator's default stats, so neither tool can produce a player the checker then rejects.
AATF reports violations and never fixes them.

## Loading and validation

The official file is embedded and is the default; the Save editor's settings hold the path of an
alternative file, and a "save a copy" action writes the embedded one out. A file is validated on
load, by the Save editor's "validate rules" button and by the Ruleset editor before saving:

1. It parses, and defines `RULESET`, `check_team`, `tier_values` and `card_limits` with their
   arities.
2. `RULESET` deserializes into `Ruleset` of a known `schema`, and passes the rules under "The
   ruleset schema".
3. `check_team` runs without a script error on the sample team embedded in `libs/aatf` (a real,
   anonymized cup team). What it reports does not matter here; the fixtures check that.

A failure is a finding with the file's line and column for parse errors and the key path
(`tiers[2].free.tricks`) for schema errors, before any real check runs.

**Logic identity.** `libs/aatf` embeds the current logic and the SHA-256 of every logic section
it has ever shipped, by version (`LOGIC_VERSIONS`), hashed with line endings normalized to LF. A
file's logic section is *current*, an *older official* version (the Ruleset editor offers to
update it), or *modified*: it still runs, and the Ruleset editor edits its data but says the
specials depend on that logic. A logic change that needs a new data shape bumps `schema`;
`libs/aatf` keeps a migration from every earlier schema, run by the update.

## Crate layout

```
crates/libs/aatf/
├── src/
│   ├── lib.rs          # Rules (a loaded file), the host API: check_team, tier_values,
│   │                   #   card_limits, apply_tier; Violation, Severity
│   ├── ruleset.rs      # Ruleset and its types, deserialization, the load-time rules
│   ├── engine.rs       # rhai::Engine setup: registered types and accessors, limits
│   ├── file.rs         # section split, logic identity (LOGIC_VERSIONS), assembling a file
│   │                   #   from a data block and the current logic
│   └── writer.rs       # (Phase 20) Ruleset → RULESET map literal, deterministic
├── logic/logic.rhai    # the interpreter, current version
├── rulesets/           # data blocks only; a shipped file is data + logic.rhai
│   ├── autumn26.rhai
│   └── vgl26.rhai
└── tests/fixtures/     # the sample team; known-legal and known-violating teams per ruleset
```

The data blocks hold only `const RULESET = …;`; the embedded official file is assembled from the
data and the current logic, so the logic exists once in the repository.

---

## Why data plus an interpreter in the file

- **Not TOML parameters plus a logic file** (the first design). The two files version-skew: an
  invitational's parameters without a `bronze` key against logic that reads one. Tiers as serde
  fields resisted the fourth tier Autumn 26 added.
- **Not CEL** (one non-Turing-complete expression per rule). Autumn 26's specials need sequencing
  (the A-or-COM choice), a preset × formation traversal and a starting-eleven lookup; under CEL
  each is a new host helper, so a ruleset change would still need a Rust release.
- **Not hand-written Rhai logic per ruleset** (the 2026-09-19 design). Only a programmer could
  make a new ruleset, and invitationals are organized by people who are not.
- **Not a TOML ruleset interpreted by Rust.** A file's meaning would depend on the Studio build
  that runs it: the official logic changes most cups, so an invitational's rules would change
  under it when members update. Every new kind of rule would also need a Rust release.
- **Not visual programming** (blocks or a node graph generating Rhai). Checked in 2026-10:
  node-graph widgets exist for egui 0.36 (`nodez`, `egui-snarl`) but block editors do not, and
  reading hand-edited Rhai back into a graph needs a lossless Rhai parser nobody maintains.
  Every rule in seven years of history is a condition and an effect, which forms express. A graph
  can come later as a `GRAPH` block beside `RULESET`, without changing the format.

The costs accepted: `RULESET`'s syntax is denser than TOML (the Ruleset editor hides it, and
schema errors carry a key path); each file carries a copy of the logic (a few hundred lines);
and a rule the schema cannot express needs either the custom-checks hook or a schema extension.

---

## Development phase and verification

Phase 5 (with the Save editor's logic): `Ruleset` and the load-time rules, the interpreter, the
host and its accessors, `apply_tier`, logic identity, the Autumn 26 and VGL26 data blocks, the
sample team and the fixtures. The writer (`writer.rs`) and schema migrations arrive with the
Ruleset editor in Phase 20.

Verification:

- **Parity, official.** The Autumn 26 file flags the same players with the same violation kinds
  as `aatf.cpp` on a corpus of real cup saves; each difference is one of the deliberate ones
  above or a fix.
- **Parity, VGL26.** The same against `aatf_single_vgl` on VGL teams (where the corpus comes
  from is an open question for Phase 5).
- Per ruleset, known-legal teams report nothing; each known-violating fixture reports exactly its
  violation. Every special of the official file is exercised by at least one fixture.
- Suggestions: a fixture per allowance kind, and none for an allowance not listed in
  `suggestions`.
- Validation: one failing file per load-time rule, each reporting its key path or line.
- `apply_tier` then `check_team`: no violation of the applied tier's targets, for every tier of
  both rulesets.
- A script that loops forever stops at the operation limit with a finding, not a hang.
