// AATF ruleset, a Rhai script. Open it in the Ruleset editor to change it.
const RULESET = #{
    schema: 1,
    name: "Divegrass Before Christmas 7",
    squad_size: 23,

    stats: #{
        rating: ["dribbling", "goalkeeping", "finishing", "low_pass", "lofted_pass", "heading",
                 "swerve", "catching", "clearing", "reflexes", "coverage", "body_control",
                 "physical_contact", "kicking_power", "explosive_power", "ball_control",
                 "ball_winning", "jump", "place_kicking", "stamina", "speed", "aggression"],
        at_most: ["attacking_prowess", "ball_control", "dribbling", "low_pass", "lofted_pass",
                  "finishing", "place_kicking", "swerve", "heading", "defensive_prowess",
                  "ball_winning", "kicking_power", "speed", "explosive_power", "body_control",
                  "physical_contact", "jump", "goalkeeping", "catching", "clearing",
                  "reflexes", "coverage", "stamina", "tight_possession", "aggression",
                  "form", "injury_resistance", "weak_foot_usage", "weak_foot_accuracy"],
    },

    // No card or position rule on the page ("No tactical export rules for anything
    // else"): no card cap, the game not holding a player to ten skill cards in
    // play. 48 paid cards cover every card the game has (41 is the most skills
    // any version has, with the seven COM styles), 13 A positions its 13
    // positions.
    tiers: [
        #{ id: "gold", label: "Gold", rate: 99, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 48, a_positions: 13 },
        #{ id: "silver", label: "Silver", rate: 88, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 48, a_positions: 13,
           stat_warning: 5, total_within_targets: true },
        #{ id: "regular", label: "Regular", rate: 77, form: 8, injury_resistance: 3,
           weak_foot: #{ usage: 4, accuracy: 4 }, cards: 48, a_positions: 13,
           stat_warning: 7, total_within_targets: true },
    ],

    medal_systems: [
        #{ label: "GGG", counts: #{ gold: 3 } },
        #{ label: "GGSS", counts: #{ gold: 2, silver: 2 } },
        #{ label: "GSSSS", counts: #{ gold: 1, silver: 4 } },
        #{ label: "SSSSSS", counts: #{ silver: 6 } },
    ],

    heights: #{
        brackets: [
            #{ id: "manlet", label: "Manlet", exactly: [175] },
            #{ id: "mid",    label: "Mid",    exactly: [180] },
            #{ id: "tall",   label: "Tall",   exactly: [185] },
            #{ id: "giant",  label: "Giant",  exactly: [195] },
        ],
        systems: [
            #{ id: "height", label: "Height", by_quotas: true,
               quotas: #{ giant: 6, tall: 6, mid: 5, manlet: 6 } },
            #{ id: "manlet", label: "Manlet", by_quotas: true,
               quotas: #{ giant: 0, tall: 10, mid: 7, manlet: 6 } },
            #{ id: "universal", label: "Universal", by_quotas: true,
               quotas: #{ giant: 0, tall: 23, mid: 0, manlet: 0 } },
            #{ id: "giga", label: "GigaHeight",
               quotas: #{ giant: 0, tall: 7, mid: 10, manlet: 6 },
               extra_height: #{ total: 50, max: 210 } },
        ],
    },

    // 41 is the most skills any version has: no cap, like the paid cards.
    universal: #{ max_skill_cards: 41 },

    specials: [
        #{ label: "Goalkeepers at 189 count as 185",
           when: #{ positions: ["GK"], height_min: 189, height_max: 189 },
           then: #{ height_allowance: 4 } },
        // A choice of one option: the bonus may be taken, so a Manlet silver at 90
        // still reads as a silver (recognition sees the unbuffed 88).
        #{ label: "Manlet bonus",
           when: #{ systems: ["manlet"], height_max: 175 },
           then: #{ choice: [#{ stat_bonus: #{ all: #{ regular: 5, silver: 5 } } }] } },
        // The bonus's +5 would let a regular's band reach a silver's 88 (82 + 7):
        // his narrows to the silver's 5.
        #{ label: "Manlet regulars: warning band 5",
           when: #{ systems: ["manlet"], height_max: 175 },
           then: #{ stat_warning: #{ regular: 5 } } },
        #{ label: "Goalkeepers at 189 have no second A position",
           when: #{ positions: ["GK"], height_min: 189, height_max: 189 },
           then: #{ max_a_positions: 1 } },
        #{ label: "The registered position must be rated A", severity: "warning",
           when: #{ any: [
               #{ positions: ["GK"],  not: #{ rated_a_at: ["GK"] } },
               #{ positions: ["CB"],  not: #{ rated_a_at: ["CB"] } },
               #{ positions: ["LB"],  not: #{ rated_a_at: ["LB"] } },
               #{ positions: ["RB"],  not: #{ rated_a_at: ["RB"] } },
               #{ positions: ["DMF"], not: #{ rated_a_at: ["DMF"] } },
               #{ positions: ["CMF"], not: #{ rated_a_at: ["CMF"] } },
               #{ positions: ["LMF"], not: #{ rated_a_at: ["LMF"] } },
               #{ positions: ["RMF"], not: #{ rated_a_at: ["RMF"] } },
               #{ positions: ["AMF"], not: #{ rated_a_at: ["AMF"] } },
               #{ positions: ["LWF"], not: #{ rated_a_at: ["LWF"] } },
               #{ positions: ["RWF"], not: #{ rated_a_at: ["RWF"] } },
               #{ positions: ["SS"],  not: #{ rated_a_at: ["SS"] } },
               #{ positions: ["CF"],  not: #{ rated_a_at: ["CF"] } },
           ] },
           then: #{ forbid: "The registered position is not rated A" } },
    ],
};
