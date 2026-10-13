//! `TierValues` and `CardLimits`: what a file's `tier_values` and
//! `card_limits` answer for one player read as one tier, converted from
//! the script's maps, and the write of `TierValues` into a player record,
//! the lib's one writing operation (`aatf_rules.md` "The host").

use std::collections::BTreeMap;

use pes_savefile::model::player::{PlayerEntry, PlayerStats};
use serde::Deserialize;

use crate::ruleset::{Card, GroupId, Stat, WeakFoot};

/// The targets a player must carry as one tier, the specials applied:
/// per ability stat, form, injury resistance and weak foot, on the
/// game's scale.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TierValues {
    /// The target of every ability stat (the write skips those the
    /// version lacks).
    pub stats: BTreeMap<Stat, u8>,
    /// The target form (1 to 8).
    pub form: u8,
    /// The target injury resistance (1 to 3).
    pub injury_resistance: u8,
    /// The weak-foot limits (1 to 4 each).
    pub weak_foot: WeakFoot,
}

/// The card economy of a player as one tier.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardLimits {
    /// Paid cards allowed, the player's extra A positions deducted.
    pub limit: u8,
    /// Free cards per group (`com` included), the specials' added.
    pub free: BTreeMap<GroupId, u8>,
    /// Cards that cost nothing.
    pub free_cards: Vec<Card>,
    /// Cards the player must hold.
    pub required_cards: Vec<Card>,
    /// A positions allowed, the registered one counted.
    pub a_positions: u8,
    /// The cap on A positions, when a special sets one.
    pub max_a_positions: Option<u8>,
}

impl TierValues {
    /// Writes the values into `entry`: every ability the version has at
    /// its target, form, injury resistance and weak foot at theirs. The
    /// record stores each of the last four one below the game's scale.
    pub(crate) fn write(&self, entry: &mut PlayerEntry) {
        for (stat, value) in &self.stats {
            write_ability(&mut entry.stats, *stat, *value);
        }
        entry.stats.form = self.form.saturating_sub(1);
        entry.stats.injury_resistance = self.injury_resistance.saturating_sub(1);
        entry.stats.weak_foot_usage = self.weak_foot.usage.saturating_sub(1);
        entry.stats.weak_foot_accuracy = self.weak_foot.accuracy.saturating_sub(1);
    }
}

/// Sets ability `stat` of `stats` to `value`; a stat the version lacks
/// (`None`) stays absent. The arms follow `labels::ABILITY_STATS`' order,
/// which `PlayerStats::abilities` reads by, and a test pins each to its
/// field.
pub(crate) fn write_ability(stats: &mut PlayerStats, stat: Stat, value: u8) {
    let gated = |field: &mut Option<u8>| {
        if field.is_some() {
            *field = Some(value);
        }
    };
    match stat.0 {
        0 => stats.attacking_prowess = value,
        1 => stats.ball_control = value,
        2 => stats.dribbling = value,
        3 => stats.low_pass = value,
        4 => stats.lofted_pass = value,
        5 => stats.finishing = value,
        6 => stats.place_kicking = value,
        7 => stats.swerve = value,
        8 => stats.heading = value,
        9 => stats.defensive_prowess = value,
        10 => stats.ball_winning = value,
        11 => stats.kicking_power = value,
        12 => stats.speed = value,
        13 => stats.explosive_power = value,
        14 => stats.body_control = value,
        15 => gated(&mut stats.physical_contact),
        16 => stats.jump = value,
        17 => stats.goalkeeping = value,
        18 => gated(&mut stats.catching),
        19 => gated(&mut stats.clearing),
        20 => gated(&mut stats.reflexes),
        21 => gated(&mut stats.coverage),
        22 => stats.stamina = value,
        23 => gated(&mut stats.tight_possession),
        24 => gated(&mut stats.aggression),
        // `Stat` is built from `ABILITY_STATS`' 25 indices alone.
        other => unreachable!("ability stat index {other} is past the 25 ability stats"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pes_savefile::interchange::team_toml::labels::ABILITY_STATS;

    /// Writing index `i` into stat `i` reads back as `abilities()` in
    /// the same order; a gated stat stays `None` where it is `None`.
    #[test]
    fn each_ability_writes_its_own_field() {
        // PES 21 shape: every gated stat present.
        let mut stats = PlayerStats {
            physical_contact: Some(0),
            catching: Some(0),
            clearing: Some(0),
            reflexes: Some(0),
            coverage: Some(0),
            tight_possession: Some(0),
            aggression: Some(0),
            ..Default::default()
        };
        for (i, _) in ABILITY_STATS.iter().enumerate() {
            let index = u8::try_from(i).expect("25 stats");
            write_ability(&mut stats, Stat(index), 30 + index);
        }
        let want: Vec<Option<u8>> = (0..25u8).map(|i| Some(30 + i)).collect();
        assert_eq!(stats.abilities().to_vec(), want);
        // PES 15 shape: the gated stats absent stay absent.
        let mut stats = PlayerStats::default();
        for (i, _) in ABILITY_STATS.iter().enumerate() {
            write_ability(&mut stats, Stat(u8::try_from(i).expect("25 stats")), 50);
        }
        let abilities = stats.abilities();
        for index in [15usize, 18, 19, 20, 21, 23, 24] {
            assert_eq!(abilities[index], None, "{}", ABILITY_STATS[index]);
        }
        assert_eq!(abilities.iter().flatten().count(), 18);
        assert!(abilities.iter().flatten().all(|v| *v == 50));
    }

    /// The four scaled values land one below the game's scale.
    #[test]
    fn the_write_stores_the_scaled_values_one_below() {
        let mut entry = PlayerEntry::default();
        let values = TierValues {
            stats: BTreeMap::new(),
            form: 8,
            injury_resistance: 3,
            weak_foot: WeakFoot {
                usage: 4,
                accuracy: 2,
            },
        };
        values.write(&mut entry);
        assert_eq!(entry.stats.form, 7);
        assert_eq!(entry.stats.injury_resistance, 2);
        assert_eq!(entry.stats.weak_foot_usage, 3);
        assert_eq!(entry.stats.weak_foot_accuracy, 1);
    }
}
