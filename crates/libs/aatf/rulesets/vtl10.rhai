// AATF ruleset. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "VTL10",
    squad_size: 23,

    stats: #{
        rating: ["clearing"],
        at_most: [],
    },

    tiers: [
        #{ id: "gold", label: "Gold", count: 2,
           when: #{ height_min: 194, height_max: 194, ranges: #{ swerve: [99, 99] } },
           rate: 99, form: 8, injury_resistance: 1,
           stats: #{ place_kicking: 96, defensive_prowess: 58 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 6, min_cards: 3, free: #{ com: 2 }, a_positions: 2 },
        #{ id: "silver", label: "Silver", count: 3,
           when: #{ height_min: 194, height_max: 194, ranges: #{ swerve: [92, 92] } },
           rate: 92, form: 8, injury_resistance: 1,
           stats: #{ attacking_prowess: 95, place_kicking: 88, defensive_prowess: 56,
                    ball_winning: 88, stamina: 95, aggression: 88 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 2 }, a_positions: 3 },
        #{ id: "regular", label: "Regular", count: 10,
           when: #{ any: [#{ height_min: 178, height_max: 178 },
                          #{ height_min: 180, height_max: 180 }] },
           rate: 77, form: 4, injury_resistance: 1,
           stats: #{ attacking_prowess: 87, ball_control: 87, dribbling: 87,
                    low_pass: 87, lofted_pass: 92, finishing: 97,
                    place_kicking: 70, swerve: 87, defensive_prowess: 73,
                    ball_winning: 68, kicking_power: 87, explosive_power: 73,
                    physical_contact: 68, jump: 67, goalkeeping: 75,
                    catching: 70, reflexes: 73, coverage: 73, stamina: 55,
                    tight_possession: 87, aggression: 92 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 1 }, a_positions: 2 },
        #{ id: "buffed", label: "Buffed", count: 8,
           when: #{ height_min: 188, height_max: 188 },
           rate: 77, form: 4, injury_resistance: 1,
           stats: #{ attacking_prowess: 92, ball_control: 87, dribbling: 92,
                    low_pass: 87, lofted_pass: 87, finishing: 92,
                    place_kicking: 70, swerve: 87, defensive_prowess: 53,
                    ball_winning: 68, kicking_power: 92, speed: 88,
                    explosive_power: 88, body_control: 88, physical_contact: 88,
                    goalkeeping: 75, catching: 70, reflexes: 73, coverage: 73,
                    stamina: 60, tight_possession: 87, aggression: 87 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 1 }, a_positions: 1 },
    ],

    universal: #{
        registered_position_a: true, no_b_ratings: true, gk_not_second_a: true,
        age: [15, 50], weight: #{ floor: 30, below: 129, above: 81 },
        min_goalkeepers: 2, max_skill_cards: 12, max_com_styles: 5,
    },

    specials: [
        #{ label: "The free skills", then: #{ free_cards: [
            "scissors_feint", "flip_flap", "marseille_turn", "sombrero",
            "cut_behind_and_turn", "scotch_move", "acrobatic_finishing",
            "heel_trick", "first_time_shot", "malicia", "double_touch",
            "crossover_turn", "step_on_skill"] } },
        #{ label: "The pass skills are free to non-medals",
           when: #{ tiers: ["regular", "buffed"] },
           then: #{ free_cards: ["one_touch_pass", "weighted_pass", "pinpoint_crossing"] } },
        #{ label: "The captain's Captaincy card is free",
           when: #{ captain: true }, then: #{ free_cards: ["captaincy"] } },
        #{ label: "Incisive Run and Long Ranger are free COM styles",
           then: #{ free_cards: ["incisive_run", "long_ranger"] } },
        #{ label: "Malicia is required", then: #{ require_cards: ["malicia"] } },
        #{ label: "Regulars cannot be captain",
           when: #{ tiers: ["regular", "buffed"], captain: true },
           then: #{ forbid: "Regulars can't be captain" } },
        #{ label: "Medals cannot be goalkeepers",
           when: #{ tiers: ["gold", "silver"], positions: ["GK"] },
           then: #{ forbid: "Medals cannot play as GK" } },
        #{ label: "188cm players have no A at CB or GK",
           when: #{ tiers: ["buffed"], rated_a_at: ["CB", "GK"] },
           then: #{ forbid: "188cm players may not have an A-position at CB or GK" } },
        #{ label: "A goalkeeper's form is 8",
           when: #{ tiers: ["regular"], positions: ["GK"] },
           then: #{ form: 4 } },
        #{ label: "Regular goalkeepers are 180cm",
           when: #{ tiers: ["regular"], positions: ["GK"],
                    height_min: 178, height_max: 178 },
           then: #{ forbid: "Regular goalkeepers must be 180cm" } },
        #{ label: "Outfield regulars are 178cm",
           when: #{ tiers: ["regular"], height_min: 180, height_max: 180,
                    not: #{ positions: ["GK"] } },
           then: #{ forbid: "Outfield regulars must be 178cm" } },
        #{ label: "Buffed wingbacks are 2/2 weak foot",
           when: #{ tiers: ["buffed"], rated_a_at: ["LB", "RB"] },
           then: #{ weak_foot: #{ usage: 2, accuracy: 2 } } },
    ],

    team_settings: [
        #{ label: "No man marking on PES 16", versions: [16], man_marking: false },
        #{ label: "PES 16 support settings", versions: [16],
           auto_substitution: false, auto_offside_trap: false, auto_preset_change: false },
    ],
};

// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====
// What the schema cannot carry: the goalkeeper maximum (min_goalkeepers is a floor), and the
// 188cm players' registered positions — at most two, and no 178cm player may share one.
fn custom_checks(team) {
    let found = [];
    let goalkeepers = 0;
    for player in team.players {
        if player.position == "GK" { goalkeepers += 1; }
    }
    if goalkeepers > 2 {
        found.push(#{ kind: "Custom", text: "More than two goalkeepers" });
    }
    // The 188cm players' registered positions, in first-seen order.
    let positions = [];
    for player in team.players {
        if player.height == 188 && player.position != () && !(player.position in positions) {
            positions.push(player.position);
        }
    }
    if positions.len() > 2 {
        found.push(#{ kind: "Custom", text: "Too many registered positions with 188cm players. Maximum is 2" });
    }
    // The 178cm players at one of the first two of those positions.
    let theirs = [];
    for position in positions {
        if theirs.len() == 2 { break; }
        theirs.push(position);
    }
    for player in team.players {
        if player.height == 178 && player.position in theirs {
            found.push(#{ kind: "Custom",
                          text: "A 178cm player shares a 188cm player's registered position",
                          slot: player.slot });
        }
    }
    found
}
