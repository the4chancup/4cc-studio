// ==== LOGIC (ruleset-logic 1): written by 4cc Studio; the Ruleset editor replaces this section ====
// The interpreter is not written yet: check_team returns the custom checks' violations only.

fn check_team(team) {
    let found = [];
    if is_def_fn("custom_checks", 1) {
        found += custom_checks(team);
    }
    found
}

fn tier_values(player, tier, team) { #{} }

fn card_limits(player, tier, team) { #{} }
