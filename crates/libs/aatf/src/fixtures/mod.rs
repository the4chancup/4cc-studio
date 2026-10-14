//! The per-ruleset fixtures (`aatf_rules.md` "Development phase and
//! verification"): a known-legal team reports nothing, and each
//! known-violating fixture, a real team changed in its test by the
//! smallest edit that breaks one rule, reports exactly that rule's
//! violation. The base is the sample team (`sample/team.toml`: team 714
//! of the 2026 Summer Cup save, a PES 15 team legal under Spring 26, the
//! ruleset it was built for); each ruleset's module makes it legal under
//! its own rules first.

use pes_savefile::interchange::team_toml::TeamToml;
use pes_savefile::model::player::PlayerEntry;
use pes_savefile::model::team::TeamEntry;
use pes_version::PesVersion;

use crate::ruleset::{Severity, Stat, WeakFoot};
use crate::team::records_from_team_toml;
use crate::values::TierValues;
use crate::violation::ViolationKind;
use crate::{PlayerView, Rules, TeamView, Violation};

mod autumn25;
mod interpreter;
mod official;
mod spring26;
mod vgl25;
mod vgl26;
mod vgl27;

/// A team as records, what `apply_tier` writes into, viewed on demand.
struct Records {
    version: PesVersion,
    team: TeamEntry,
    players: Vec<PlayerEntry>,
}

impl Records {
    /// The sample team's records.
    fn sample() -> Records {
        let document = TeamToml::parse(include_str!("../../sample/team.toml"))
            .expect("the sample team parses");
        let (version, team, players) =
            records_from_team_toml(&document).expect("the sample team builds records");
        Records {
            version,
            team,
            players,
        }
    }

    /// The sample team's records on `version`: a Team TOML document is
    /// version-neutral, so setting its `pes_version` before the build
    /// gives the same team on another version (the VGL cups play on
    /// PES 21, whose presets can be fluid; PES 15's cannot).
    fn sample_on(version: PesVersion) -> Records {
        let mut document = TeamToml::parse(include_str!("../../sample/team.toml"))
            .expect("the sample team parses");
        document.pes_version = Some(version);
        let (version, team, players) = records_from_team_toml(&document)
            .expect("the sample team builds records on the version");
        Records {
            version,
            team,
            players,
        }
    }

    /// The records' view.
    fn view(&self) -> TeamView {
        TeamView::view(self.version, &self.team, |id| {
            self.players.iter().find(|player| player.id == id)
        })
        .expect("the records view")
    }

    /// The record of roster `slot`.
    fn player_mut(&mut self, slot: u8) -> &mut PlayerEntry {
        let id = self.team.roster[usize::from(slot)].player_id;
        self.players
            .iter_mut()
            .find(|player| player.id == id)
            .unwrap_or_else(|| panic!("roster slot {slot} is filled"))
    }
}

/// `entry` at a tier's targets: every ability at `rate`, his form and
/// injury resistance set, weak foot kept (through `TierValues::write`,
/// which the fixtures then exercise too).
fn set_tier_record(entry: &mut PlayerEntry, rate: u8, form: u8, injury_resistance: u8) {
    TierValues {
        stats: (0..25u8).map(|index| (Stat(index), rate)).collect(),
        form,
        injury_resistance,
        weak_foot: WeakFoot {
            usage: entry.stats.weak_foot_usage + 1,
            accuracy: entry.stats.weak_foot_accuracy + 1,
        },
    }
    .write(entry);
}

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
