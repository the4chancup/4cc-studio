// AATF ruleset, a Rhai script. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "VGL27",
    squad_size: 23,

    stats: #{
        rating: ["dribbling", "goalkeeping", "low_pass", "lofted_pass", "swerve",
                 "catching", "clearing", "reflexes", "coverage", "body_control",
                 "physical_contact", "kicking_power", "explosive_power", "ball_control",
                 "ball_winning", "jump", "place_kicking", "stamina", "speed"],
        at_most: ["attacking_prowess", "defensive_prowess",
                  "weak_foot_usage", "weak_foot_accuracy"],
    },

    tiers: [
        #{ id: "gold", label: "Gold", count: 1,
           when: #{ not: #{ positions: ["GK"] } },
           rate: 99, form: 8, injury_resistance: 2,
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 6, free: #{ tricks: 12, com: 1 }, a_positions: 2 },
        #{ id: "silver", label: "Silver", count: 2,
           when: #{ not: #{ positions: ["GK"] } },
           rate: 94, form: 8, injury_resistance: 2,
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 5, free: #{ tricks: 12, com: 1 }, a_positions: 2 },
        #{ id: "bronze", label: "Bronze", count: 2,
           when: #{ not: #{ positions: ["GK"] } },
           rate: 89, form: 8, injury_resistance: 2,
           weak_foot: #{ usage: 4, accuracy: 4 },
           cards: 5, free: #{ tricks: 12, com: 1 }, a_positions: 2 },
        #{ id: "goalkeeper", label: "Goalkeeper", counts_as: "regular",
           when: #{ positions: ["GK"] },
           rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 },
           cards: 4, free: #{ tricks: 12 }, a_positions: 1 },
        #{ id: "regular", label: "Regular",
           when: #{ not: #{ positions: ["GK"] } },
           rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 },
           cards: 4, free: #{ tricks: 12 }, a_positions: 2 },
    ],

    card_groups: #{
        tricks: ["scissors_feint", "double_touch", "flip_flap", "marseille_turn",
                 "sombrero", "crossover_turn", "cut_behind_and_turn", "scotch_move",
                 "step_on_skill", "rabona", "no_look_pass", "malicia"],
    },

    heights: #{
        brackets: [
            #{ id: "manlet",   label: "Manlet",   exactly: [175] },
            #{ id: "mid",      label: "Mid",      exactly: [180] },
            #{ id: "tall",     label: "Tall",     exactly: [185] },
            #{ id: "giant",    label: "Giant",    exactly: [190] },
            #{ id: "colossal", label: "Colossal", exactly: [210] },
        ],
        systems: [
            #{ id: "purple", label: "Purple",
               quotas: #{ colossal: 0, giant: 0, tall: 10, mid: 7, manlet: 6 } },
        ],
    },

    universal: #{
        registered_position_a: true, fielded_at_a: [1],
        no_b_ratings: true, gk_not_second_a: true,
        age: [15, 50], weight: #{ floor: 30, below: 129, above: 81 },
        captain_required: true, min_goalkeepers: 1, max_skill_cards: 41,
    },

    specials: [
        #{ label: "The captain's Captaincy card is free, plus one card, COM style or A position",
           when: #{ captain: true },
           then: #{ free_cards: ["captaincy"],
                    choice: [#{ cards: 1 }, #{ free: #{ com: 1 } }, #{ a_positions: 1 }] } },
        #{ label: "Goalkeepers at 189 count as tall",
           when: #{ positions: ["GK"], height_min: 189, height_max: 189 },
           then: #{ height_allowance: 4 } },
        #{ label: "Tall goalkeepers are 189",
           when: #{ positions: ["GK"], height_min: 185, height_max: 185 },
           then: #{ forbid: "GKs in this bracket must be 189cm" } },
        #{ label: "Manlets", when: #{ brackets: ["manlet"] },
           then: #{ cards: 1, a_positions: 1, weak_foot: #{ usage: 4, accuracy: 4 } } },
    ],

    team_settings: [
        #{ label: "No man marking", man_marking: false },
        #{ label: "Support settings", auto_substitution: false,
           auto_offside_trap: false, auto_preset_change: false },
    ],

    suggestions: ["cards", "com", "a_positions", "weak_foot", "free_cards"],
};
