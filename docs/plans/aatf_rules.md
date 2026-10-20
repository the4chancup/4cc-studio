# 4cc Studio — AATF rules plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

What exists of this is in `spec/aatf_rules.md` "4cc Studio — AATF rules plan".

## The ruleset schema

What exists of this is in `spec/aatf_rules.md` "The ruleset schema".

DBC7 ("DBC7" below) adds three things the schema cannot say (worklog 20.0d): a choice between
several medal counts, height systems told apart by their counts, and a pool of centimetres
shared by the team.

```rust
pub struct Ruleset {
    // ... every field in the spec, and:
    /// Alternative counts for some tiers, of which the team's must equal one (DBC7: GGG,
    /// GGSS, GSSSS, SSSSSS). Empty: each tier's own `count`.
    pub medal_systems: Vec<MedalSystem>,
}

pub struct MedalSystem {
    pub label: String,
    /// A tier another system names and this one does not has 0 here.
    pub counts: BTreeMap<TierId, u8>,
}

pub struct HeightSystem {
    // ... every field in the spec, and:
    /// The system holds only when the team, read under it, meets its quotas (the pool's
    /// checks included), as well as `any_player`.
    pub by_quotas: bool,
    pub extra_height: Option<ExtraHeight>,
}

/// Centimetres the whole team may add above the bracket heights (DBC7's giga system: 50,
/// no player above 210).
pub struct ExtraHeight {
    pub total: u8,
    /// No player is taller, before any allowance.
    pub max: u8,
}
```

**Medal systems.** A tier a medal system names has no `count` and no `counts_as`, and no tier
`counts_as` it; the tier that is the rest of the squad takes what the team's system leaves.
The team's system is the first whose counts equal the team's counts of the named tiers; when
none does, the team gets one `MedalSystem` violation and the rest of the squad is what the
team's named-tier players leave, so a wrong medal count is reported once, not again as the
regulars' `TierCount`. We do not give each tier a list of allowed counts instead: DBC7's
systems tie the counts together (two golds go with two silvers, never with four), which lists
per tier cannot say.

**Height systems by quotas.** "How a player is read" step 1 becomes: the first system whose
`any_player` holds and, with `by_quotas`, under which the team's quota and pool checks (step
4's part for heights) find nothing, every player read under that system (step 2). Reading the
team under each candidate is what lets an allowance or a tier that a special gives under one
system count toward that system's quotas. DBC7's organizer and its editor tell the systems
apart by how many players stand at each height; no condition on one player says that (a giga
team can hold a 195 cm player, so "any player at 195" would misread it as the Height bracket).

**The pool.** Under a system with `extra_height`, the quota brackets are filled tallest first:
players taken by height less allowance, highest first, each into the tallest quota bracket with
room whose height is at most his, the centimetres he stands above that height drawn from the
pool. A player above `max`, or below every quota bracket's height, is a `Height` violation and
takes no bracket, as a height no bracket takes does under any system; the pool overdrawn is one `ExtraHeight { used, total }` on the team; a bracket left short or
overfilled is `BracketQuota`, as elsewhere. Tallest first draws no more than any other order
that fills the quotas (what is drawn is the players' heights less the quotas' heights, the
same for every such order); the order decides only whether the quotas fill. A condition naming
a bracket still sees the bracket the player's own height takes (none, between two heights): the
assignment is the team's, made after every player is read, and DBC7 names no bracket in a
condition. We do not make the pool a custom check: a custom check runs after every player is
read, so a team whose giga players stand between bracket heights would already have failed the
built-in height checks, and the Manlet system's stat bonus needs the system that the pool
decides.

**Recognition before choices.** "How a player is read" step 2 compares the player's rating
with the tier's expected rating *before* its choices are resolved: an option only raises a
target, so a player who rates the tier without it is that tier. Read after them, DBC7's optional
Manlet bonus would make a silver at 92 (taking 4 of the 5) fall to regular, since the option,
used, raises his expected rating to 93; three Manlet teams of the maintainer's PES 2017 save
carry such a silver, and VGL25's optional manlet bonus had the same gap with no cup save to show
it. A mandatory effect still moves the threshold, as the Green silver giant's does.

New violation kinds: `MedalSystem { systems: Vec<String> }` (the team; every system's label, so
the message lists the counts the team may have) and
`ExtraHeight { used: u16, total: u16 }`.

New load-time rules: a medal system's label is not empty, it names existing tiers, and those
tiers have no `count` and no `counts_as` and no tier `counts_as` them; every system's counts plus
the fixed counts fit `squad_size`, and equal it when no tier is the rest of the squad. Every
height system but the last has a non-empty `any_player` or `by_quotas` with quotas (with none, no
quota fails); the last has neither
(else a team could fall through every system). A system with `extra_height` gives quotas only to
`exactly` brackets of one height, and its quotas add up to `squad_size`.

## DBC7

A test ruleset in `rulesets/dbc7.rhai` (`spec/aatf_rules.md` "Embedded rulesets": an
invitational without an organization of its own gets no template; the preview build's handling
is worklog 0.0.1's), for Divegrass Before Christmas 7 (PES 2017, exports due 2026-11-18). Its
source of truth is the wiki page
`https://implyingrigged.info/wiki/Divegrass_Before_Christmas_7:_Melty_Under_The_Christmas_Tree`,
"Tactical export Rules" (read through a browser, since a plain fetch gets HTTP 403; unchanged
between 2026-10-19 and 2026-10-20); how its rules are checked is in the organizer's fork of the
official editor, DBC7-Editor (`RockAnon/DBC7-Editor`, `aatf.cpp` at `ee73917`), a reference where
the two differ.

| Feature | DBC7 |
|---|---|
| Tiers | gold 99 and silver 88 by the medal systems GGG, GGSS, GSSSS, SSSSSS (`medal_systems`); regular 77, the rest; no per-stat targets; every ability stat at most its target |
| Tier recognition | rating alone (the official rating stats) |
| Heights | four brackets of one height each (175, 180, 185, 195); three systems `by_quotas` in the organizer's order, Height (6, 6, 5, 6 from 195 down), Manlet (0, 10, 7, 6) and Universal (0, 23, 0, 0), then GigaHeight (0, 7, 10, 6 with `extra_height` 50, at most 210) for every other team; a registered goalkeeper at 189 counts as 185 (`height_allowance: 4`) in every system, after the pool under GigaHeight, as the page says |
| Specials | the GK allowance; under Manlet, players at 175 may take +5 on every stat, regulars and silvers (the page's "silvers can also be buffed"; a gold is at 99 already), a one-option `choice` since the page allows the bonus rather than requiring it |
| Cards, positions, form, weak foot, injury resistance | none: "No tactical export rules for anything else … go crazy". Paid cards 17 (the game's 10 skills and 7 COM styles), 13 A positions, B ratings allowed, form, injury resistance and weak foot at most the game's maxima |
| Universal | none of the official file's checks; `max_skill_cards` 10, the game's own limit |

Where the data reads differently from the fork, on purpose, since the page is followed:

- The fork warns of a stat above its target by up to 7 (a regular) or 5 (a silver, a Manlet
  regular) and errs only past that; the page's "Non-Medals are 77 rated, Silvers are 88 rated"
  makes any stat above the target a violation.
- The fork also errs on what the page leaves free: the registered position not rated A, an age
  outside 15 to 50, a weight outside the official range, no captain, no registered goalkeeper,
  and a 189 cm goalkeeper with a second A position. None is in the data block (QUESTIONS.md
  "DBC7: the organizer's editor errs on what the page leaves free").
- The fork assigns GigaHeight's brackets in roster order and puts a player with no room left
  in the lowest bracket his height reaches, past its quota, drawing for him too; the pool places
  no one past a quota. On a team that fills the quotas the two place the same counts and draw
  the same total; on one that cannot (a stripped save's 23 placeholders at 180) the fork also
  reports the draw of the players it overfilled, the pool only the short bracket.
- The fork does not check the game's ten-skill limit, which `max_skill_cards` keeps.

## Coverage of the DBC7 rules

Each rule of the page's "Tactical export Rules", with the fork's `aatf_single` for how it is
checked:

| Rule (page; fork) | Schema |
|---|---|
| Height bracket: 6x 195, 6x 185, 5x 180, 6x 175 (`usingGreen`, counts equal) | `Heights::brackets` of one exact height each; a `HeightSystem` with `by_quotas` and `quotas` |
| Manlet bracket: 10x 185, 7x 180, 6x 175 (`usingRed`) | the same, the second system |
| Universal: 23x 185 (`usingCommie`) | the same, the third system |
| GigaHeight: 7x 185, 10x 180, 6x 175 and 50 cm to add to any player, at most 210 (`usingGiga`, every other team) | the last system, `quotas` and `extra_height` |
| The two-210 variant: 2x 210, 5x 185, 10x 180, 6x 175 | GigaHeight: two 185s raised 25 each |
| Manlet: stats for 175 cm players increased by 5, silvers too (`silverManletBonus`, gold 0) | a special: `when: #{ systems: ["manlet"], height_max: 175 }`, `choice: [#{ stat_bonus: … }]`, 5 for regular and silver |
| 185 cm goalkeepers may be 189, after GigaHeight's 50 cm (`heightTallGK`, a GK at 189 counted as 185 with no pool drawn) | a special: a registered GK at 189 gets `height_allowance: 4` |
| Non-medals 77, silvers 88, golds 99 (the rating's thresholds, each stat at most the target) | `Tier::rate`; recognition by rating; `StatRules::at_most` holding every ability stat |
| Medal systems GGG, GGSS, GSSSS, SSSSSS (the four count pairs) | `Ruleset::medal_systems` |
| No rule for anything else (cards, footing, formations) | tiers at the game's maxima, `universal` off but the game's skill limit |

No row is a custom check: the three new fields are the gaps, each one a form ("The editor" in
`plans/ruleset_editor.md`).

## The interpreter

What exists of this is in `spec/aatf_rules.md` "The interpreter".

Every surface that shows results (the Save editor's AATF panel, the Team creator's) has a "Show
suggestions" toggle, off by default, and lists them apart from errors and warnings, uncounted.

## Loading and validation

What exists of this is in `spec/aatf_rules.md` "Loading and validation".

A "save a copy" action writes the embedded file out, and a file is validated by the Save editor's
"validate rules" button and by the Ruleset editor before saving. A logic change that needs a new
data shape bumps `schema`; `libs/aatf` keeps a migration from every earlier schema, run by the
update.

## Crate layout

What exists of this is in `spec/aatf_rules.md` "Crate layout".

`src/writer.rs` (Phase 20): `Ruleset` → `RULESET` map literal, deterministic. A map key that
is not an identifier (an id is any non-empty string: `gold-medal`), or is one of Rhai's
keywords or reserved words (`in`, `for`), is written as a string literal, which Rhai's map
literal takes.

## Development phase and verification

What exists of this is in `spec/aatf_rules.md` "Development phase and verification".

The writer (`writer.rs`) and schema migrations arrive with the Ruleset editor in Phase 20.

DBC7's fields and data block (worklog 20.0d) are verified as the others are:

- **Parity, DBC7.** The data block against the fork's `aatf_single` (built headless, as for
  5.2f) on the maintainer's PES 2017 saves (`Documents\KONAMI\Pro Evolution Soccer 2017\save`
  and `C:\Data\4cc\Cups\Autumn24Q`), every team; built for other rules, most teams fail, and
  what is compared is the findings, each difference one listed under "DBC7" or a fix. Run at
  20.0d (2026-10-20, 220 teams each): every difference is one of those classes; the one fix it
  found is "Recognition before choices". The fork passes 29 and 36 teams, and the data block
  fails each of them on stats inside the fork's warning band alone (`QUESTIONS.md`).
- Fixtures: a legal team under each height system and each medal system; a Manlet regular and a
  Manlet silver at their raised targets, and the silver at 92 (still a silver); a goalkeeper at 189 under Universal and under GigaHeight;
  GigaHeight teams at exactly 50 cm, at 51, with a player at 211, with one at 174, with a 185
  cm player taking a 180 slot, and the page's two-210 variant; a medal count no system has; a
  stat one above its target.
- Validation: one failing file per new load-time rule.
