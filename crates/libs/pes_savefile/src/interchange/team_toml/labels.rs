//! The label tables `team.toml` writes where the record stores a number or a
//! bit: `(key, ["false" label, "true" label])` pairs for the style switches,
//! index tables for positions, instructions, ratings, styles, skills and COM
//! styles. Pure data; the player tables are defined here for 2.17h-3.

/// `[tactics.preset_N.style]`: `(key, [false label, true label])` in emit order.
pub const STYLE_SWITCHES: [(&str, [&str; 2]); 7] = [
    ("attacking_style", ["counter_attack", "possession"]),
    ("attacking_zone", ["centre", "wide"]),
    ("buildup", ["long_pass", "short_pass"]),
    ("positioning", ["maintain", "flexible"]),
    ("defensive_style", ["frontline_pressure", "all_out_defence"]),
    ("containment_area", ["middle", "wide"]),
    ("pressure", ["aggressive", "conservative"]),
];

/// Formation/registered-position names; index = the stored position byte.
pub const POSITIONS: [&str; 13] = [
    "GK", "CB", "LB", "RB", "DMF", "CMF", "LMF", "RMF", "AMF", "LWF", "RWF", "SS", "CF",
];

/// Advanced-instruction names in canonical order (`model::instruction::ALL`'s
/// index): `wing_back`, not a stored value.
pub const INSTRUCTIONS: [&str; 17] = [
    "off",
    "hug_the_touchline",
    "false_no_9",
    "false_full_backs",
    "attacking_full_backs",
    "wing_rotation",
    "tiki_taka",
    "centering_targets",
    "swarm_the_box",
    "deep_defensive_line",
    "gegenpress",
    "tight_marking",
    "counter_target",
    "defensive",
    "false_winger",
    "wing_back",
    "anchoring",
];

/// Playable-position ratings; index = the stored rating (0 none to 3 A).
pub const RATINGS: [&str; 4] = ["none", "C", "B", "A"];

/// `PlayStyle` names in canonical order (`PlayStyle::ALL`'s index).
pub const PLAY_STYLES: [&str; 22] = [
    "none",
    "goal_poacher",
    "dummy_runner",
    "fox_in_the_box",
    "target_man",
    "creative_playmaker",
    "prolific_winger",
    "roaming_flank",
    "crossing_specialist",
    "classic_no_10",
    "hole_player",
    "box_to_box",
    "the_destroyer",
    "orchestrator",
    "anchor_man",
    "offensive_fullback",
    "fullback_finisher",
    "defensive_fullback",
    "build_up",
    "extra_frontman",
    "offensive_goalkeeper",
    "defensive_goalkeeper",
];

/// Player-skill names; index = the canonical skill slot (0 Scissors Feint to
/// 40 Through Passing).
pub const SKILLS: [&str; 41] = [
    "scissors_feint",
    "flip_flap",
    "marseille_turn",
    "sombrero",
    "cut_behind_and_turn",
    "scotch_move",
    "heading",
    "long_range_drive",
    "knuckle_shot",
    "acrobatic_finishing",
    "heel_trick",
    "first_time_shot",
    "one_touch_pass",
    "weighted_pass",
    "pinpoint_crossing",
    "outside_curler",
    "rabona",
    "low_lofted_pass",
    "low_punt_trajectory",
    "long_throw",
    "gk_long_throw",
    "malicia",
    "man_marking",
    "track_back",
    "acrobatic_clear",
    "captaincy",
    "super_sub",
    "fighting_spirit",
    "double_touch",
    "crossover_turn",
    "step_on_skill",
    "chip_shot",
    "dipping_shots",
    "rising_shots",
    "no_look_pass",
    "gk_high_punt",
    "penalty_specialist",
    "gk_penalty_specialist",
    "interception",
    "long_range_shooting",
    "through_passing",
];

/// COM playing-style names; index = the canonical slot (0 Trickster to 6 Long
/// Ranger).
pub const COM_STYLES: [&str; 7] = [
    "trickster",
    "mazing_run",
    "speeding_bullet",
    "incisive_run",
    "long_ball_expert",
    "early_cross",
    "long_ranger",
];

/// Stronger foot/hand labels; index = the stored bool.
pub const FOOT: [&str; 2] = ["right", "left"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::instruction::Instruction;
    use crate::model::playstyle::PlayStyle;

    /// Lowercase ASCII, digits and underscores only.
    fn is_snake_case(label: &str) -> bool {
        !label.is_empty()
            && label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    }

    fn unique(table: &[&str]) -> bool {
        let mut sorted = table.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        sorted.len() == table.len()
    }

    #[test]
    fn every_label_table_is_sized_and_unique() {
        assert_eq!(INSTRUCTIONS.len(), Instruction::ALL.len());
        assert_eq!(PLAY_STYLES.len(), PlayStyle::ALL.len());
        assert_eq!(POSITIONS.len(), 13);
        assert_eq!(RATINGS.len(), 4);
        assert_eq!(SKILLS.len(), 41);
        assert_eq!(COM_STYLES.len(), 7);
        assert_eq!(FOOT.len(), 2);
        for table in [
            POSITIONS.as_slice(),
            INSTRUCTIONS.as_slice(),
            RATINGS.as_slice(),
            PLAY_STYLES.as_slice(),
            SKILLS.as_slice(),
            COM_STYLES.as_slice(),
            FOOT.as_slice(),
        ] {
            assert!(unique(table), "duplicates in {table:?}");
        }
        for (_, pair) in STYLE_SWITCHES {
            assert_ne!(pair[0], pair[1]);
        }
    }

    #[test]
    fn labels_are_snake_case_or_the_uppercase_short_forms() {
        // Positions ("GK") and the ratings' letter grades are uppercase by the
        // plan's own TOML block; everything else is snake_case.
        for label in INSTRUCTIONS
            .iter()
            .chain(PLAY_STYLES.iter())
            .chain(SKILLS.iter())
            .chain(COM_STYLES.iter())
            .chain(FOOT.iter())
            .chain(RATINGS.iter().filter(|l| **l == "none"))
            .chain(STYLE_SWITCHES.iter().flat_map(|(key, pair)| {
                assert!(is_snake_case(key), "{key}");
                pair.iter()
            }))
        {
            assert!(is_snake_case(label), "{label}");
        }
    }
}
