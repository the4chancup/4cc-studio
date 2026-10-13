//! The per-ruleset fixtures (`aatf_rules.md` "Development phase and
//! verification"): a known-legal team reports nothing, and each
//! known-violating fixture, a real team changed in its test by the
//! smallest edit that breaks one rule, reports exactly that rule's
//! violation. The base is the sample team (`sample/team.toml`: team 714
//! of the 2026 Summer Cup save, a PES 15 team legal under Spring 26, the
//! ruleset it was built for); each ruleset's module makes it legal under
//! its own rules first.

use crate::ruleset::Severity;
use crate::violation::ViolationKind;
use crate::{PlayerView, Rules, TeamView, Violation};

mod official;

/// The violations of `team` under `rules` that are not suggestions: what
/// a member must fix.
fn problems(rules: &Rules, team: &TeamView) -> Vec<Violation> {
    rules
        .check_team(team)
        .expect("the check runs")
        .into_iter()
        .filter(|violation| violation.severity != Severity::Suggestion)
        .collect()
}

/// The player at roster `slot`.
fn player(team: &TeamView, slot: u8) -> &PlayerView {
    team.players
        .iter()
        .find(|player| player.slot == slot)
        .unwrap_or_else(|| panic!("roster slot {slot} is filled"))
}

/// The player at roster `slot`, to edit.
fn player_mut(team: &mut TeamView, slot: u8) -> &mut PlayerView {
    team.players
        .iter_mut()
        .find(|player| player.slot == slot)
        .unwrap_or_else(|| panic!("roster slot {slot} is filled"))
}

/// `player` at a tier's targets: every ability the version has at `rate`,
/// his form and injury resistance set; cards and weak foot untouched.
fn set_tier(player: &mut PlayerView, rate: u8, form: u8, injury_resistance: u8) {
    for ability in player.abilities.iter_mut().flatten() {
        *ability = rate;
    }
    player.form = form;
    player.injury_resistance = injury_resistance;
}

/// The suggestions alone, for roster `slot`, as kinds.
fn suggestions_for(rules: &Rules, team: &TeamView, slot: u8) -> Vec<ViolationKind> {
    rules
        .check_team(team)
        .expect("the check runs")
        .into_iter()
        .filter(|violation| violation.severity == Severity::Suggestion)
        .filter(|violation| violation.slot == Some(slot))
        .map(|violation| violation.kind)
        .collect()
}

/// An error-severity violation on roster `slot` (`None`: the team).
fn error(slot: Option<u8>, kind: ViolationKind) -> Violation {
    Violation {
        slot,
        severity: Severity::Error,
        kind,
    }
}

/// A warning on roster `slot`.
fn warning(slot: u8, kind: ViolationKind) -> Violation {
    Violation {
        slot: Some(slot),
        severity: Severity::Warning,
        kind,
    }
}

/// An `Unused` suggestion's kind.
fn unused(allowance: &str, used: u8, allowed: u8) -> ViolationKind {
    ViolationKind::Unused {
        allowance: allowance.to_string(),
        used,
        allowed,
    }
}
