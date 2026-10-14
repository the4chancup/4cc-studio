// ==== LOGIC (ruleset-logic 1): written by 4cc Studio; the Ruleset editor replaces this section ====
// The generic interpreter: it reads the RULESET map above over the Team and Player values the
// host registers (the script API is listed in 4cc Studio's aatf crate doc) and reports each
// violation as a map the host converts. The rules it applies are described in the plan
// (aatf_rules.md "How a player is read", "The card economy", "The interpreter").
//
// Rhai copies every argument of a call except a method call's receiver (`this`), so the
// functions that read the ruleset are written as methods on it (`rules.read_player(...)`),
// and the condition evaluator as a method on the team: those are the two large values.

// ---- What the schema leaves to the interpreter

// The seven COM playing styles: the reserved `com` card group.
fn com_styles() {
    ["trickster", "mazing_run", "speeding_bullet", "incisive_run", "long_ball_expert",
     "early_cross", "long_ranger"]
}

// The thirteen positions, in the position table's order.
fn positions() {
    ["GK", "CB", "LB", "RB", "DMF", "CMF", "LMF", "RMF", "AMF", "LWF", "RWF", "SS", "CF"]
}

// The ability stats every tier targets; a version lacking one answers `()` and is skipped.
fn ability_stats() {
    ["attacking_prowess", "ball_control", "dribbling", "low_pass", "lofted_pass", "finishing",
     "place_kicking", "swerve", "heading", "defensive_prowess", "ball_winning", "kicking_power",
     "speed", "explosive_power", "body_control", "physical_contact", "jump", "goalkeeping",
     "catching", "clearing", "reflexes", "coverage", "stamina", "tight_possession", "aggression"]
}

// ---- Reading a map whose field the file may leave out (its empty value then)

fn num(map, key) { if key in map && map[key] != () { map[key] } else { 0 } }
fn list(map, key) { if key in map && map[key] != () { map[key] } else { [] } }
fn table(map, key) { if key in map && map[key] != () { map[key] } else { #{} } }
fn flag(map, key) { if key in map && map[key] != () { map[key] } else { false } }

// A `PerTier` value for `tier_id`: the number itself, or the map's entry (`()` when the tier
// is absent from it, which leaves that tier unaffected).
fn per_tier(value, tier_id) {
    if type_of(value) == "map" {
        if tier_id in value { value[tier_id] } else { () }
    } else {
        value
    }
}

// The ruleset with every list or map the file may leave out present and empty, so the hot
// paths read `this.specials` or `this.heights.brackets` directly: a `list(this, ...)` call
// would copy the whole ruleset each time.
fn normalized(rules) {
    rules.tiers = list(rules, "tiers");
    rules.specials = list(rules, "specials");
    rules.team_settings = list(rules, "team_settings");
    rules.suggestions = list(rules, "suggestions");
    rules.card_groups = table(rules, "card_groups");
    let heights = table(rules, "heights");
    heights.brackets = list(heights, "brackets");
    heights.systems = list(heights, "systems");
    rules.heights = heights;
    let stats = table(rules, "stats");
    stats.rating = list(stats, "rating");
    stats.at_most = list(stats, "at_most");
    rules.stats = stats;
    rules
}

// ---- The three functions the host calls

fn check_team(team) {
    let rules = normalized(global::RULESET);
    let found = [];
    let system = rules.height_system(team);
    let read = [];
    for player in team.players {
        let r = rules.read_player(team, system, player);
        read.push(r);
        if r.tier == () {
            found.push(#{ kind: "NoTier", slot: player.slot });
            continue;
        }
        found += rules.player_checks(team, system, player, r.tier, r.profile);
    }
    found += rules.team_checks(team, system, read);
    if is_def_fn("custom_checks", 1) {
        found += custom_checks(team);
    }
    found
}

// The targets `player` must carry as tier `tier` (its id): every ability stat's, form, injury
// resistance and the weak-foot limits, the specials applied as `check_team` applies them.
fn tier_values(player, tier, team) {
    let rules = normalized(global::RULESET);
    let t = rules.tier(tier);
    if t == () { throw "unknown tier " + tier; }
    let p = rules.profile(team, rules.height_system(team), player, tier);
    let weak_foot = weak_foot_of(t, p);
    #{
        stats: targets(t, p),
        form: raised_target(t, p, "form"),
        injury_resistance: raised_target(t, p, "injury_resistance"),
        weak_foot: #{ usage: num(weak_foot, "usage"), accuracy: num(weak_foot, "accuracy") },
    }
}

// The card economy of `player` as tier `tier`: the paid limit (his extra A positions
// deducted), the free count per group, the free and required cards, the A positions allowed.
fn card_limits(player, tier, team) {
    let rules = normalized(global::RULESET);
    let t = rules.tier(tier);
    if t == () { throw "unknown tier " + tier; }
    let p = rules.profile(team, rules.height_system(team), player, tier);
    let cards = rules.card_summary(t, p, player);
    let free_cards = [];
    for card in p.free_cards { if !(card in free_cards) { free_cards.push(card); } }
    let required_cards = [];
    for required in p.require_cards { required_cards.push(required.card); }
    #{
        limit: cards.limit, free: cards.free, free_cards: free_cards,
        required_cards: required_cards, a_positions: cards.a_allowance,
        max_a_positions: p.max_a_positions,
    }
}

// ---- Conditions

// Whether `cond` holds for `player` read as tier `tier_id` in bracket `bracket_id` (each `()`
// when not known yet); `this` is the team. An absent or empty condition holds. A present
// list field is a list (the host's schema refused anything else), tested by key first: this
// runs for every special of every tier of every player.
fn condition_holds(system, player, tier_id, bracket_id, cond) {
    if cond == () { return true; }
    if "tiers" in cond && cond.tiers.len() > 0 && !(tier_id in cond.tiers) { return false; }
    if "positions" in cond && cond.positions.len() > 0 && !(player.position in cond.positions) {
        return false;
    }
    if "systems" in cond && cond.systems.len() > 0
        && (system == () || !(system.id in cond.systems)) {
        return false;
    }
    if "brackets" in cond && cond.brackets.len() > 0 && !(bracket_id in cond.brackets) {
        return false;
    }
    if "height_min" in cond && player.height < cond.height_min { return false; }
    if "height_max" in cond && player.height > cond.height_max { return false; }
    if "captain" in cond && (this.captain == player.slot) != cond.captain { return false; }
    if "playing_styles" in cond && cond.playing_styles.len() > 0
        && !(player.playing_style in cond.playing_styles) {
        return false;
    }
    if "holds" in cond && cond.holds.len() > 0 {
        let held = player.cards;
        let any = false;
        for card in cond.holds { if card in held { any = true; break; } }
        if !any { return false; }
    }
    if "rated_a_at" in cond && cond.rated_a_at.len() > 0 && !player.rated_at(cond.rated_a_at, "A") {
        return false;
    }
    if "rated_b_at" in cond && cond.rated_b_at.len() > 0 && !player.rated_at(cond.rated_b_at, "B") {
        return false;
    }
    if "name_colors" in cond && cond.name_colors.len() > 0
        && !(player.name_color in cond.name_colors) {
        return false;
    }
    if "fielded_only_at" in cond && !this.fielded_only_at(player, cond.fielded_only_at) {
        return false;
    }
    if "ranges" in cond {
        for key in cond.ranges.keys() {
            let value = player_value(player, key);
            if value == () || value < cond.ranges[key][0] || value > cond.ranges[key][1] {
                return false;
            }
        }
    }
    if "strong_foot" in cond && player.strong_foot != cond.strong_foot { return false; }
    if "any" in cond && cond.any.len() > 0 {
        let holds = false;
        for entry in cond.any {
            if this.condition_holds(system, player, tier_id, bracket_id, entry) {
                holds = true;
                break;
            }
        }
        if !holds { return false; }
    }
    if "not" in cond && this.condition_holds(system, player, tier_id, bracket_id, cond["not"]) {
        return false;
    }
    true
}

// `player`'s `PlayerValue` `key`: an ability stat through `player.stat` (`()`
// where the version lacks it, failing the range), the accessor of the same
// name for the six other names.
fn player_value(player, key) {
    if key == "age" { player.age }
    else if key == "weight" { player.weight }
    else if key == "form" { player.form }
    else if key == "injury_resistance" { player.injury_resistance }
    else if key == "weak_foot_usage" { player.weak_foot_usage }
    else if key == "weak_foot_accuracy" { player.weak_foot_accuracy }
    else { player.stat(key) }
}

// Whether `this` (a player) is rated `rating` at any of `positions`.
fn rated_at(positions, rating) {
    for pos in positions { if this.rating_at(pos) == rating { return true; } }
    false
}

// The first height system whose `any_player` holds for some player (`()` without systems);
// `this` is the ruleset. The condition may name heights and positions only, so it is
// evaluated with no tier and no bracket.
fn height_system(team) {
    for system in this.heights.systems {
        let cond = table(system, "any_player");
        if cond.keys().len() == 0 { return system; }
        for player in team.players {
            if team.condition_holds((), player, (), (), cond) { return system; }
        }
    }
    ()
}

// ---- Reading a player as a tier: the specials' effects, the bracket, the choices

// The first bracket taking `height` (an `up_to` bracket takes heights above the brackets
// before it), or `()`; `this` is the ruleset.
fn bracket_of(height) {
    let floor = 0;
    for bracket in this.heights.brackets {
        if "up_to" in bracket {
            if height > floor && height <= bracket.up_to { return bracket.id; }
            floor = max(floor, bracket.up_to);
        } else {
            if height in bracket.exactly { return bracket.id; }
            for h in bracket.exactly { floor = max(floor, h); }
        }
    }
    ()
}

// How `player` reads as `tier_id`: height allowances first, then the bracket, then every
// other effect of the specials whose condition holds, then the choices; `this` is the ruleset.
fn profile(team, system, player, tier_id) {
    let p = #{
        allowance: 0, bracket: (), bonus: #{}, targets: #{}, cards: 0, free: #{},
        free_cards: [], require_cards: [], a_positions: 0, max_a_positions: (), weak_foot: (),
        form: 0, injury_resistance: 0, forbids: [], choices: [], taken: [],
    };
    for special in this.specials {
        let then = table(special, "then");
        if "height_allowance" in then
            && team.condition_holds(system, player, tier_id, (), special.when) {
            p.allowance += then.height_allowance;
        }
    }
    p.bracket = this.bracket_of(player.height - p.allowance);
    for special in this.specials {
        if team.condition_holds(system, player, tier_id, p.bracket, special.when) {
            p = apply_effects(p, tier_id, special.label, severity_of(special), table(special, "then"));
        }
    }
    this.resolve_choices(tier_id, p, player)
}

// A special's severity, `error` when it names none.
fn severity_of(special) {
    if "severity" in special && special.severity != () { special.severity } else { "error" }
}

// `p` with `then`'s effects applied for `tier_id`: numeric effects add up, replacing effects
// take the last value, a `height_allowance` is left to `profile`, and a `choice` waits for
// `resolve_choices`.
fn apply_effects(p, tier_id, label, severity, then) {
    let bonus = table(then, "stat_bonus");
    for key in bonus.keys() {
        let amount = per_tier(bonus[key], tier_id);
        if amount != () { p.bonus[key] = num(p.bonus, key) + amount; }
    }
    let targets = table(then, "stat_target");
    for key in targets.keys() {
        let value = per_tier(targets[key], tier_id);
        if value != () { p.targets[key] = value; }
    }
    if "cards" in then {
        let value = per_tier(then.cards, tier_id);
        if value != () { p.cards += value; }
    }
    let free = table(then, "free");
    for key in free.keys() {
        let value = per_tier(free[key], tier_id);
        if value != () { p.free[key] = num(p.free, key) + value; }
    }
    p.free_cards += list(then, "free_cards");
    for card in list(then, "require_cards") {
        p.require_cards.push(#{ card: card, severity: severity });
    }
    if "a_positions" in then {
        let value = per_tier(then.a_positions, tier_id);
        if value != () { p.a_positions += value; }
    }
    if "max_a_positions" in then { p.max_a_positions = then.max_a_positions; }
    if "form" in then {
        let value = per_tier(then.form, tier_id);
        if value != () { p.form += value; }
    }
    if "injury_resistance" in then {
        let value = per_tier(then.injury_resistance, tier_id);
        if value != () { p.injury_resistance += value; }
    }
    if "weak_foot" in then { p.weak_foot = then.weak_foot; }
    if "forbid" in then {
        p.forbids.push(#{ label: label, text: then.forbid, severity: severity });
    }
    if "choice" in then {
        p.choices.push(#{ label: label, severity: severity, options: then.choice });
    }
    p
}

// `p` with each choice's first used option applied, the used choices' labels in `taken`;
// `this` is the ruleset. An option is used when, without it, the player exceeds a limit the
// option raises.
fn resolve_choices(tier_id, p, player) {
    let tier = this.tier(tier_id);
    for choice in p.choices {
        for option in choice.options {
            if this.option_used(tier, p, player, option) {
                p = apply_effects(p, tier_id, choice.label, choice.severity, option);
                p.taken.push(choice.label);
                break;
            }
        }
    }
    p
}

// Whether `player`, read as `tier` under `p`, exceeds a limit `option` raises; `this` is the
// ruleset.
fn option_used(tier, p, player, option) {
    let cards = this.card_summary(tier, p, player);
    if "stat_bonus" in option
        && this.player_rating(player) > this.expected_rating(tier, p) {
        return true;
    }
    if "cards" in option && cards.paid > cards.limit { return true; }
    for group in table(option, "free").keys() {
        if num(cards.held_by_group, group) > num(cards.free, group) { return true; }
    }
    for card in list(option, "free_cards") {
        if card in cards.held && !(card in p.free_cards) { return true; }
    }
    if "a_positions" in option && cards.counted_a > cards.a_allowance { return true; }
    if "max_a_positions" in option && p.max_a_positions != ()
        && cards.a_count > p.max_a_positions {
        return true;
    }
    if "weak_foot" in option {
        let weak_foot = weak_foot_of(tier, p);
        if player.weak_foot_usage > num(weak_foot, "usage")
            || player.weak_foot_accuracy > num(weak_foot, "accuracy") {
            return true;
        }
    }
    false
}

// The tier map `tier_id` names; `this` is the ruleset.
fn tier(tier_id) {
    for tier in this.tiers { if tier.id == tier_id { return tier; } }
    ()
}

// ---- Targets and the rating

// The target of every ability stat for a player read as `tier` under `p`: a special's target,
// the tier's own, or its rate, plus the bonuses. One map per tier: computing each stat's
// target on its own would copy the tier and the profile for each of the 25 stats.
fn targets(tier, p) {
    let out = #{};
    let overrides = table(tier, "stats");
    let all = num(p.bonus, "all");
    for stat in ability_stats() {
        let base = if stat in p.targets {
            p.targets[stat]
        } else if stat in overrides {
            overrides[stat]
        } else {
            tier.rate
        };
        out[stat] = base + all + num(p.bonus, stat);
    }
    out
}

// The highest target among the rating stats: what a player of `tier` rates; `this` is the
// ruleset.
fn expected_rating(tier, p) {
    let targets = targets(tier, p);
    let best = 0;
    for stat in this.stats.rating {
        best = max(best, targets[stat]);
    }
    best
}

// The highest of the player's rating stats, those the version lacks skipped; `this` is the
// ruleset.
fn player_rating(player) {
    let best = 0;
    for stat in this.stats.rating {
        let value = player.stat(stat);
        if value != () { best = max(best, value); }
    }
    best
}

// The weak-foot limits: a special's replacement, else the tier's.
fn weak_foot_of(tier, p) {
    if p.weak_foot != () { p.weak_foot } else { table(tier, "weak_foot") }
}

// The tier's `key` value plus the specials' effects of the same key.
fn raised_target(tier, p, key) { num(tier, key) + num(p, key) }

// The name-colour codes the tiers set (empty when none does).
fn name_colors() {
    let codes = [];
    for tier in this.tiers {
        if "name_color" in tier && tier.name_color != () { codes.push(tier.name_color); }
    }
    codes
}

// `player`'s tier and profile: the first tier in list order whose `when` holds and whose
// expected rating is at most the player's; else the last tier whose `when` holds; else no tier
// (`tier: ()`); `this` is the ruleset.
fn read_player(team, system, player) {
    let rating = this.player_rating(player);
    let last = #{ player: player, tier: (), profile: () };
    for tier in this.tiers {
        let p = this.profile(team, system, player, tier.id);
        if !team.condition_holds(system, player, tier.id, p.bracket, table(tier, "when")) {
            continue;
        }
        last = #{ player: player, tier: tier, profile: p };
        if this.expected_rating(tier, p) <= rating { return last; }
    }
    last
}

// ---- The card economy

// The groups holding `card`: `com` for a COM style, the file's groups for the rest.
fn groups_of(groups, card) {
    let found = [];
    if card in com_styles() { found.push("com"); }
    for group in groups.keys() { if card in groups[group] { found.push(group); } }
    found
}

// A player's cards against `tier` under `p`: the held cards, how many are skills and COM
// styles, the free count and the held count per group (a card free by name is not counted
// there), the paid cards (and paid skills), the A and B ratings, the A allowance, the
// extra A positions and the paid limit; `this` is the ruleset.
fn card_summary(tier, p, player) {
    let held = player.cards;
    let com = com_styles();
    let skills = 0;
    let coms = 0;
    for card in held { if card in com { coms += 1; } else { skills += 1; } }
    let free = #{};
    let tier_free = table(tier, "free");
    for group in tier_free.keys() { free[group] = tier_free[group]; }
    for group in p.free.keys() { free[group] = num(free, group) + p.free[group]; }
    let groups = this.card_groups;
    let held_by_group = #{};
    let remaining = #{};
    for group in free.keys() { remaining[group] = free[group]; }
    let paid = 0;
    let paid_skills = 0;
    for card in held {
        if card in p.free_cards { continue; }
        let mine = groups_of(groups, card);
        for group in mine { held_by_group[group] = num(held_by_group, group) + 1; }
        let covered = false;
        for group in mine {
            if num(remaining, group) > 0 {
                remaining[group] -= 1;
                covered = true;
                break;
            }
        }
        if covered { continue; }
        paid += 1;
        if !(card in com) { paid_skills += 1; }
    }
    let a_count = 0;
    let b_count = 0;
    for pos in positions() {
        let rating = player.rating_at(pos);
        if rating == "A" { a_count += 1; } else if rating == "B" { b_count += 1; }
    }
    let counted_a = a_count + (if flag(this.universal, "b_uses_a_allowance") { b_count } else { 0 });
    let a_allowance = num(tier, "a_positions") + p.a_positions;
    let extra = max(0, counted_a - a_allowance);
    #{
        held: held, skills: skills, coms: coms, free: free, held_by_group: held_by_group,
        paid: paid, paid_skills: paid_skills, a_count: a_count, b_count: b_count,
        counted_a: counted_a, a_allowance: a_allowance, extra: extra,
        limit: num(tier, "cards") + p.cards - extra,
    }
}

// ---- The per-player checks

// Whether `value` misses `target`: above it when `name` is an `at_most` target, else off it.
fn misses(at_most, name, value, target) {
    if name in at_most { value > target } else { value != target }
}

// Every violation of `player`, read as `tier` under `p`, against the universal rules, the
// tier's targets and limits, the card economy, the height and the specials' forbids, then the
// suggestions; `this` is the ruleset.
fn player_checks(team, system, player, tier, p) {
    let found = [];
    let slot = player.slot;
    let universal = this.universal;
    let at_most = this.stats.at_most;
    // Built in: the version's registered-position and playing-style ranges.
    if player.position == () { found.push(#{ kind: "PositionRange", slot: slot }); }
    if player.playing_style == () { found.push(#{ kind: "PlayingStyleRange", slot: slot }); }
    if flag(universal, "registered_position_a") && player.position != ()
        && player.rating_at(player.position) != "A" {
        found.push(#{ kind: "RegisteredNotA", slot: slot });
    }
    for preset in list(universal, "fielded_at_a") {
        for pos in team.fielded_positions(player, preset) {
            if player.rating_at(pos) != "A" {
                found.push(#{ kind: "FieldedNotA", slot: slot, preset: preset, position: pos });
            }
        }
    }
    let cards = this.card_summary(tier, p, player);
    if flag(universal, "no_b_ratings") && cards.b_count > 0 {
        found.push(#{ kind: "BRating", slot: slot });
    }
    // A GK rating among several A positions: the GK's own second A included.
    if flag(universal, "gk_not_second_a") && cards.a_count > 1 && player.rating_at("GK") == "A" {
        found.push(#{ kind: "GkSecondA", slot: slot });
    }
    if "age" in universal && universal.age != () {
        let age = universal.age;
        if player.age < age[0] || player.age > age[1] {
            found.push(#{ kind: "Age", slot: slot, age: player.age, min: age[0], max: age[1] });
        }
    }
    if "weight" in universal && universal.weight != () {
        let rule = universal.weight;
        let low = max(rule.floor, player.height - rule.below);
        let high = max(0, player.height - rule.above);
        if player.weight < low || player.weight > high {
            found.push(#{ kind: "Weight", slot: slot, weight: player.weight, min: low, max: high });
        }
    }
    let targets = targets(tier, p);
    for stat in ability_stats() {
        let value = player.stat(stat);
        if value == () { continue; }
        let target = targets[stat];
        if misses(at_most, stat, value, target) {
            found.push(#{
                kind: "Stat", slot: slot, stat: stat, value: value, target: target,
                at_most: stat in at_most,
            });
        }
    }
    let form = raised_target(tier, p, "form");
    if misses(at_most, "form", player.form, form) {
        found.push(#{ kind: "Form", slot: slot, value: player.form, target: form });
    }
    let injury_resistance = raised_target(tier, p, "injury_resistance");
    if misses(at_most, "injury_resistance", player.injury_resistance, injury_resistance) {
        found.push(#{
            kind: "InjuryResistance", slot: slot, value: player.injury_resistance,
            max: injury_resistance,
        });
    }
    let weak_foot = weak_foot_of(tier, p);
    let usage = num(weak_foot, "usage");
    if misses(at_most, "weak_foot_usage", player.weak_foot_usage, usage) {
        found.push(#{ kind: "WeakFootUsage", slot: slot, value: player.weak_foot_usage, max: usage });
    }
    let accuracy = num(weak_foot, "accuracy");
    if misses(at_most, "weak_foot_accuracy", player.weak_foot_accuracy, accuracy) {
        found.push(#{
            kind: "WeakFootAccuracy", slot: slot, value: player.weak_foot_accuracy, max: accuracy,
        });
    }
    // The name colour: the tier's own code, or none of the codes the
    // tiers set when it has none.
    if "name_color" in tier && tier.name_color != () {
        if player.name_color != tier.name_color {
            found.push(#{
                kind: "NameColor", slot: slot, color: player.name_color,
                expected: tier.name_color,
            });
        }
    } else if player.name_color != () && player.name_color in this.name_colors() {
        found.push(#{ kind: "NameColor", slot: slot, color: player.name_color });
    }
    if cards.paid > cards.limit {
        found.push(#{ kind: "Cards", slot: slot, paid: cards.paid, limit: cards.limit });
    }
    let min_cards = num(tier, "min_cards");
    if cards.paid_skills + cards.extra < min_cards {
        found.push(#{
            kind: "MinCards", slot: slot, paid: cards.paid_skills + cards.extra, min: min_cards,
        });
    }
    for required in p.require_cards {
        if !(required.card in cards.held) {
            found.push(#{
                kind: "RequiredCard", slot: slot, card: required.card, severity: required.severity,
            });
        }
    }
    if p.max_a_positions != () && cards.a_count > p.max_a_positions {
        found.push(#{ kind: "APositions", slot: slot, count: cards.a_count, max: p.max_a_positions });
    }
    if cards.skills > universal.max_skill_cards {
        found.push(#{
            kind: "SkillCardCap", slot: slot, count: cards.skills, max: universal.max_skill_cards,
        });
    }
    if "max_com_styles" in universal && universal.max_com_styles != ()
        && cards.coms > universal.max_com_styles {
        found.push(#{
            kind: "ComStyleCap", slot: slot, count: cards.coms, max: universal.max_com_styles,
        });
    }
    if p.bracket == () && this.heights.brackets.len() > 0 {
        found.push(#{ kind: "Height", slot: slot, height: player.height });
    }
    for forbid in p.forbids {
        found.push(#{
            kind: "Special", slot: slot, label: forbid.label, text: forbid.text,
            severity: forbid.severity,
        });
    }
    found += this.suggestions(tier, p, player, cards);
    found
}

// An allowance named in `suggestions` the player does not use up, one suggestion each;
// `this` is the ruleset.
fn suggestions(tier, p, player, cards) {
    let found = [];
    let slot = player.slot;
    for allowance in this.suggestions {
        if allowance == "cards" {
            if cards.paid < cards.limit {
                found.push(unused(slot, allowance, cards.paid, cards.limit));
            }
        } else if allowance == "a_positions" {
            if cards.counted_a < cards.a_allowance {
                found.push(unused(slot, allowance, cards.counted_a, cards.a_allowance));
            }
        } else if allowance == "weak_foot" {
            let weak_foot = weak_foot_of(tier, p);
            let usage = num(weak_foot, "usage");
            let accuracy = num(weak_foot, "accuracy");
            if player.weak_foot_usage < usage {
                found.push(unused(slot, allowance, player.weak_foot_usage, usage));
            }
            if player.weak_foot_accuracy < accuracy {
                found.push(unused(slot, allowance, player.weak_foot_accuracy, accuracy));
            }
        } else if allowance == "injury_resistance" {
            let target = raised_target(tier, p, "injury_resistance");
            if player.injury_resistance < target {
                found.push(unused(slot, allowance, player.injury_resistance, target));
            }
        } else if allowance == "height" {
            // Below an `up_to` bracket's top, the allowance counted.
            for bracket in this.heights.brackets {
                if bracket.id == p.bracket && "up_to" in bracket
                    && player.height - p.allowance < bracket.up_to {
                    found.push(unused(slot, allowance, player.height, bracket.up_to + p.allowance));
                }
            }
        } else if allowance == "free_cards" {
            let unique = [];
            for card in p.free_cards { if !(card in unique) { unique.push(card); } }
            let held = 0;
            for card in unique { if card in cards.held { held += 1; } }
            if held < unique.len() { found.push(unused(slot, allowance, held, unique.len())); }
        } else if allowance == "choices" {
            for choice in p.choices {
                if !(choice.label in p.taken) {
                    found.push(#{
                        kind: "UnusedChoice", slot: slot, special: choice.label,
                        severity: "suggestion",
                    });
                }
            }
        } else {
            // A card group: its free cards not all held.
            let free = num(cards.free, allowance);
            let held = num(cards.held_by_group, allowance);
            if held < free { found.push(unused(slot, allowance, held, free)); }
        }
    }
    found
}

// An `Unused` suggestion.
fn unused(slot, allowance, used, allowed) {
    #{ kind: "Unused", slot: slot, allowance: allowance, used: used, allowed: allowed, severity: "suggestion" }
}

// ---- The team checks

// The tier counts (`counts_as` folded in, the tier without a count being the rest of the
// squad), the system's bracket quotas, the captain, the goalkeepers and the team settings for
// the save's version; `this` is the ruleset, `read` is `read_player`'s result per player.
fn team_checks(team, system, read) {
    let found = [];
    let fixed = 0;
    for tier in this.tiers { fixed += num(tier, "count"); }
    for tier in this.tiers {
        if "counts_as" in tier && tier.counts_as != () { continue; }
        let expected = if "count" in tier && tier.count != () {
            tier.count
        } else {
            this.squad_size - fixed
        };
        let count = 0;
        for r in read {
            if r.tier != () && (r.tier.id == tier.id || ("counts_as" in r.tier && r.tier.counts_as == tier.id)) {
                count += 1;
            }
        }
        if count != expected {
            found.push(#{ kind: "TierCount", tier: tier.id, count: count, expected: expected });
        }
    }
    if system != () {
        let quotas = table(system, "quotas");
        for bracket in this.heights.brackets {
            if !(bracket.id in quotas) { continue; }
            let count = 0;
            for r in read {
                if r.tier != () && r.profile.bracket == bracket.id { count += 1; }
            }
            if count != quotas[bracket.id] {
                found.push(#{
                    kind: "BracketQuota", system: system.id, bracket: bracket.id, count: count,
                    quota: quotas[bracket.id],
                });
            }
        }
    }
    let universal = this.universal;
    if flag(universal, "captain_required") && team.captain == () {
        found.push(#{ kind: "Captain" });
    }
    let goalkeepers = 0;
    for r in read { if r.player.position == "GK" { goalkeepers += 1; } }
    let min_goalkeepers = num(universal, "min_goalkeepers");
    if goalkeepers < min_goalkeepers {
        found.push(#{ kind: "Goalkeepers", count: goalkeepers, min: min_goalkeepers });
    }
    for entry in this.team_settings {
        let versions = list(entry, "versions");
        if versions.len() > 0 && !(team.version in versions) { continue; }
        // A setting the version lacks (`()`) cannot be checked.
        if "man_marking" in entry && team.man_marking != entry.man_marking {
            found.push(team_setting(entry.label, "man_marking"));
        }
        if "auto_substitution" in entry && team.auto_substitution != ()
            && (team.auto_substitution != 0) != entry.auto_substitution {
            found.push(team_setting(entry.label, "auto_substitution"));
        }
        if "auto_offside_trap" in entry && team.auto_offside_trap != ()
            && team.auto_offside_trap != entry.auto_offside_trap {
            found.push(team_setting(entry.label, "auto_offside_trap"));
        }
        if "auto_preset_change" in entry && team.auto_preset_change != ()
            && team.auto_preset_change != entry.auto_preset_change {
            found.push(team_setting(entry.label, "auto_preset_change"));
        }
        if "auto_attack_defence_levels" in entry && team.auto_attack_defence_levels != ()
            && team.auto_attack_defence_levels != entry.auto_attack_defence_levels {
            found.push(team_setting(entry.label, "auto_attack_defence_levels"));
        }
        let shapes = list(entry, "forbidden_shapes");
        if shapes.len() > 0 {
            for formation in team.formation_shapes() {
                if formation.shape in shapes {
                    found.push(#{
                        kind: "FormationShape", preset: formation.preset,
                        formation: formation.formation, shape: formation.shape,
                    });
                }
            }
        }
    }
    found
}

// A `TeamSetting` violation.
fn team_setting(label, setting) {
    #{ kind: "TeamSetting", label: label, setting: setting }
}
