// AATF ruleset. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "Autumn 25",
    squad_size: 23,

    stats: #{
        rating: ["dribbling", "goalkeeping", "finishing", "low_pass", "lofted_pass", "heading",
                 "swerve", "catching", "clearing", "reflexes", "coverage", "body_control",
                 "physical_contact", "kicking_power", "explosive_power", "ball_control",
                 "ball_winning", "jump", "place_kicking", "stamina", "speed", "aggression"],
        at_most: ["attacking_prowess", "defensive_prowess",
                  "injury_resistance", "weak_foot_usage", "weak_foot_accuracy"],
    },

    tiers: [
        #{ id: "gold", label: "Gold", count: 2, rate: 99, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 2, accuracy: 4 }, cards: 5, free: #{ tricks: 3, com: 2 }, a_positions: 1 },
        #{ id: "silver", label: "Silver", count: 2, rate: 88, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 2, accuracy: 4 }, cards: 4, free: #{ tricks: 3, com: 1 }, a_positions: 1 },
        #{ id: "goalkeeper", label: "Goalkeeper", counts_as: "regular", when: #{ positions: ["GK"] },
           rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 2, free: #{}, a_positions: 1 },
        #{ id: "regular", label: "Regular", rate: 77, form: 4, injury_resistance: 1,
           weak_foot: #{ usage: 2, accuracy: 2 }, cards: 3, free: #{ tricks: 2 }, a_positions: 1 },
    ],

    card_groups: #{
        tricks: ["scissors_feint", "flip_flap", "marseille_turn", "sombrero", "cut_behind_and_turn",
                 "scotch_move", "rabona", "double_touch", "crossover_turn", "step_on_skill",
                 "no_look_pass"],
    },

    heights: #{
        brackets: [
            #{ id: "manlet", label: "Manlet", up_to: 175 },
            #{ id: "mid",    label: "Mid",    up_to: 180 },
            #{ id: "tall",   label: "Tall",   exactly: [185] },
            #{ id: "giant",  label: "Giant",  exactly: [194] },
            #{ id: "giga",   label: "Giga",   exactly: [199] },
        ],
        systems: [
            #{ id: "green", label: "Green", any_player: #{ height_min: 194 },
               quotas: #{ giga: 0, giant: 5, tall: 6, mid: 6, manlet: 6 } },
            #{ id: "red", label: "Red",
               quotas: #{ giga: 0, giant: 0, tall: 10, mid: 7, manlet: 6 } },
        ],
    },

    universal: #{
        registered_position_a: true, no_b_ratings: true, gk_not_second_a: true,
        age: [15, 50], weight: #{ floor: 30, below: 129, above: 81 },
        captain_required: true, min_goalkeepers: 1, max_skill_cards: 10,
    },

    specials: [
        #{ label: "Malicia is a free card", then: #{ free_cards: ["malicia"] } },
        #{ label: "The captain's Captaincy card is free", when: #{ captain: true },
           then: #{ free_cards: ["captaincy"] } },
        #{ label: "Medals cannot be goalkeepers",
           when: #{ tiers: ["gold", "silver"], positions: ["GK"] },
           then: #{ forbid: "Medals cannot play as GK" } },
        #{ label: "Goalkeepers at 189 count as tall",
           when: #{ positions: ["GK"], height_min: 189, height_max: 189 },
           then: #{ height_allowance: 4 } },
        #{ label: "Tall goalkeepers are 189",
           when: #{ positions: ["GK"], height_min: 181, height_max: 188 },
           then: #{ forbid: "GKs in this bracket must be 189cm" } },
        #{ label: "Goalkeepers below giant", severity: "warning",
           when: #{ positions: ["GK"], height_min: 194 },
           then: #{ forbid: "GK heights cannot be 194cm or more" } },
        #{ label: "Gold below giant", severity: "warning",
           when: #{ tiers: ["gold"], height_min: 194 },
           then: #{ forbid: "Gold heights cannot be 194cm or more" } },
        #{ label: "Manlets", when: #{ brackets: ["manlet"] },
           then: #{ cards: 1, a_positions: 1, weak_foot: #{ usage: 4, accuracy: 4 } } },
        #{ label: "Red manlet bonus", when: #{ systems: ["red"], height_max: 175 },
           then: #{ stat_bonus: #{ all: #{ regular: 5, goalkeeper: 5 } } } },
    ],

    suggestions: ["cards", "tricks", "com", "weak_foot", "injury_resistance"],
};
