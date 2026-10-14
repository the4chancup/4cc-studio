// 4cc Studio AATF ruleset. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "VTL11",
    squad_size: 23,

    stats: #{
        rating: ["clearing"],
        at_most: [],
    },

    tiers: [
        #{ id: "gold", label: "Gold", count: 2,
           when: #{ height_min: 195, height_max: 195, ranges: #{ swerve: [99, 99] } },
           rate: 99, form: 8, injury_resistance: 1,
           stats: #{ place_kicking: 94, defensive_prowess: 65, ball_winning: 94,
                     speed: 94, physical_contact: 94, stamina: 94 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 3 }, a_positions: 2,
           name_color: "cc9900ff" },
        #{ id: "silver", label: "Silver", count: 3,
           when: #{ height_min: 195, height_max: 195, ranges: #{ swerve: [89, 89] } },
           rate: 89, form: 8, injury_resistance: 1,
           stats: #{ attacking_prowess: 94, heading: 94, defensive_prowess: 57,
                     tight_possession: 94, aggression: 94 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 2 }, a_positions: 3,
           name_color: "51bbc4ff" },
        #{ id: "regular", label: "Regular", count: 10,
           when: #{ height_min: 180, height_max: 180 },
           rate: 77, form: 4, injury_resistance: 1,
           stats: #{ attacking_prowess: 87, ball_control: 87, dribbling: 87,
                    low_pass: 87, lofted_pass: 92, finishing: 92,
                    place_kicking: 70, swerve: 87, defensive_prowess: 73,
                    ball_winning: 70, kicking_power: 87, physical_contact: 70,
                    jump: 70, catching: 70, reflexes: 73, coverage: 73,
                    stamina: 62, tight_possession: 87, aggression: 92 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 1 }, a_positions: 2 },
        #{ id: "buffed", label: "Buffed", count: 8,
           when: #{ height_min: 188, height_max: 188 },
           rate: 77, form: 4, injury_resistance: 1,
           stats: #{ attacking_prowess: 92, ball_control: 87, dribbling: 92,
                    low_pass: 87, lofted_pass: 87, finishing: 92,
                    place_kicking: 70, swerve: 87, defensive_prowess: 47,
                    ball_winning: 70, kicking_power: 92, speed: 87,
                    explosive_power: 87, body_control: 87, physical_contact: 82,
                    catching: 70, reflexes: 73, coverage: 73, stamina: 62,
                    tight_possession: 87, aggression: 87 },
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 7, min_cards: 3, free: #{ com: 1 }, a_positions: 1 },
    ],

    universal: #{
        registered_position_a: true, b_uses_a_allowance: true, gk_not_second_a: true,
        age: [15, 50], weight: #{ floor: 30, below: 129, above: 81 },
        min_goalkeepers: 2, max_skill_cards: 11, max_com_styles: 5,
    },

    specials: [
        #{ label: "The free skills", then: #{ free_cards: [
            "no_look_pass", "double_touch", "rabona", "acrobatic_finishing",
            "first_time_shot", "dipping_shots", "rising_shots", "heading",
            "malicia", "flip_flap", "marseille_turn", "scotch_move",
            "crossover_turn"] } },
        #{ label: "The pass skills are free to non-medals",
           when: #{ tiers: ["regular", "buffed"] },
           then: #{ free_cards: ["one_touch_pass", "weighted_pass", "pinpoint_crossing"] } },
        #{ label: "The captain's Captaincy card is free",
           when: #{ captain: true }, then: #{ free_cards: ["captaincy"] } },
        #{ label: "Long Ranger is a free COM style",
           then: #{ free_cards: ["long_ranger"] } },
        #{ label: "Malicia is required", then: #{ require_cards: ["malicia"] } },
        #{ label: "The GK Penalty Specialist card is banned",
           when: #{ holds: ["gk_penalty_specialist"] },
           then: #{ forbid: "The GK Penalty Specialist card is banned" } },
        #{ label: "Regulars cannot be captain",
           when: #{ tiers: ["regular", "buffed"], captain: true },
           then: #{ forbid: "Regulars can't be captain" } },
        #{ label: "Medals cannot be goalkeepers",
           when: #{ tiers: ["gold", "silver"], positions: ["GK"] },
           then: #{ forbid: "Medals cannot play as GK" } },
        #{ label: "B positions only for 188cm players at CB",
           when: #{ any: [
               #{ tiers: ["gold", "silver", "regular"],
                  rated_b_at: ["GK", "CB", "LB", "RB", "DMF", "CMF", "LMF", "RMF",
                               "AMF", "LWF", "RWF", "SS", "CF"] },
               #{ tiers: ["buffed"],
                  rated_b_at: ["GK", "LB", "RB", "DMF", "CMF", "LMF", "RMF",
                               "AMF", "LWF", "RWF", "SS", "CF"] },
           ] },
           then: #{ forbid: "Illegal B positions, only allowed on 188cm players at CB" } },
        #{ label: "188cm players have no A at CB or GK",
           when: #{ tiers: ["buffed"], rated_a_at: ["CB", "GK"] },
           then: #{ forbid: "188cm players may not have an A-position at CB or GK" } },
        #{ label: "A goalkeeper's form is 8",
           when: #{ tiers: ["regular"], positions: ["GK"] },
           then: #{ form: 4 } },
        #{ label: "A gold Fox in the Box",
           when: #{ tiers: ["gold"], playing_styles: ["fox_in_the_box"] },
           then: #{ stat_bonus: #{ ball_winning: 5, defensive_prowess: -5 } } },
        #{ label: "A gold Target Man",
           when: #{ tiers: ["gold"], playing_styles: ["target_man"] },
           then: #{ stat_bonus: #{ physical_contact: 5, defensive_prowess: -5 } } },
        #{ label: "A gold Creative Playmaker",
           when: #{ tiers: ["gold"], playing_styles: ["creative_playmaker"] },
           then: #{ stat_bonus: #{ stamina: 5, defensive_prowess: -5 } } },
    ],

    team_settings: [
        #{ label: "No man marking on PES 16", versions: [16], man_marking: false },
        #{ label: "PES 16 support settings", versions: [16],
           auto_substitution: false, auto_offside_trap: false, auto_preset_change: false },
    ],
};

// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ====
// What the schema cannot carry: the goalkeeper maximum (min_goalkeepers is a floor), and the
// 188cm players' registered positions — at most two, and no 180cm player may share one.
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
    // The 180cm players at one of the first two of those positions.
    let theirs = [];
    for position in positions {
        if theirs.len() == 2 { break; }
        theirs.push(position);
    }
    for player in team.players {
        if player.height == 180 && player.position in theirs {
            found.push(#{ kind: "Custom",
                          text: "A 180cm player shares a 188cm player's registered position",
                          slot: player.slot });
        }
    }
    found
}
