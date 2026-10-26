// AATF ruleset, a Rhai script. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "VGL25",
    squad_size: 23,

    stats: #{
        rating: ["dribbling", "goalkeeping", "finishing", "low_pass", "lofted_pass", "heading",
                 "swerve", "catching", "clearing", "reflexes", "coverage", "body_control",
                 "physical_contact", "kicking_power", "explosive_power", "ball_control",
                 "ball_winning", "jump", "place_kicking", "stamina", "speed", "aggression"],
        at_most: ["attacking_prowess", "defensive_prowess",
                  "weak_foot_usage", "weak_foot_accuracy"],
    },

    tiers: [
        #{ id: "gold", label: "Gold", count: 2, rate: 99, form: 8, injury_resistance: 2,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 5, free: #{ tricks: 12, com: 1 }, a_positions: 2 },
        #{ id: "silver", label: "Silver", count: 3, rate: 88, form: 8, injury_resistance: 2,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 4, free: #{ tricks: 12, com: 1 }, a_positions: 2 },
        #{ id: "regular", label: "Regular", rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 3, free: #{ tricks: 12 }, a_positions: 2 },
    ],

    card_groups: #{
        tricks: ["scissors_feint", "flip_flap", "marseille_turn", "sombrero", "cut_behind_and_turn",
                 "scotch_move", "rabona", "malicia", "double_touch", "crossover_turn",
                 "step_on_skill", "no_look_pass"],
    },

    heights: #{
        brackets: [
            #{ id: "manlet",   label: "Manlet",   up_to: 175 },
            #{ id: "mid",      label: "Mid",      exactly: [180] },
            #{ id: "tall",     label: "Tall",     exactly: [185] },
            #{ id: "giant",    label: "Giant",    exactly: [190] },
            #{ id: "colossal", label: "Colossal", exactly: [210] },
        ],
        systems: [
            #{ id: "purple", label: "Purple",
               quotas: #{ colossal: 0, giant: 6, tall: 5, mid: 6, manlet: 6 } },
        ],
    },

    universal: #{
        registered_position_a: true, no_b_ratings: true, gk_not_second_a: true,
        b_uses_a_allowance: true,
        age: [15, 50], weight: #{ floor: 30, below: 129, above: 81 },
        max_skill_cards: 41,
    },

    specials: [
        #{ label: "Acrobatic Finishing is a free card", then: #{ free_cards: ["acrobatic_finishing"] } },
        #{ label: "The captain's Captaincy card and one more are free", when: #{ captain: true },
           then: #{ free_cards: ["captaincy"], cards: 1 } },
        #{ label: "Medals cannot be goalkeepers",
           when: #{ tiers: ["gold", "silver"], positions: ["GK"] },
           then: #{ forbid: "Medals cannot play as GK" } },
        #{ label: "Goalkeepers at 189 count as tall",
           when: #{ positions: ["GK"], height_min: 189, height_max: 189 },
           then: #{ height_allowance: 4 } },
        #{ label: "Tall goalkeepers are 189",
           when: #{ positions: ["GK"], height_min: 181, height_max: 188 },
           then: #{ forbid: "GKs in this bracket must be 189cm" } },
        #{ label: "Medals cannot be giants",
           when: #{ tiers: ["gold", "silver"], height_min: 190, height_max: 190 },
           then: #{ forbid: "Medals cannot be 190cm" } },
        #{ label: "Goalkeepers cannot be giants",
           when: #{ positions: ["GK"], height_min: 190, height_max: 190 },
           then: #{ forbid: "GKs cannot be 190cm" } },
        #{ label: "Manlets", when: #{ brackets: ["manlet"] },
           then: #{ cards: 1, a_positions: 1, weak_foot: #{ usage: 4, accuracy: 4 } } },
        #{ label: "Manlet bonus", when: #{ brackets: ["manlet"] },
           then: #{ choice: [#{ stat_bonus: #{ all: #{ regular: 5, silver: 2, gold: 2 } } }] } },
    ],

    suggestions: ["cards", "com", "a_positions", "weak_foot", "height", "free_cards", "choices"],
};


