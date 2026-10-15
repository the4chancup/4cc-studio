use super::*;

// The TOML half: the template, the parser's errors, `update_toml`.

/// The key-table block of `docs/plans/aesthetics_export/settings_toml.md`,
/// verbatim: `to_toml` of the settings the block parses to must reproduce
/// it byte for byte.
const PLAN_BLOCK: &str = r#"# settings.toml, inside a player folder. Every key is optional: an absent key takes the
# value shown here (name and shirt_name: not written). A commented key shows what can be
# set and its range. The file never references the export's models: what a player wears
# is decided by the folder contents (models present, link files pointing at shared
# folders); boots_id/gloves_id only name the game's stock models.

# true = derive from the folder name ("15 - Snuffy" gives "Snuffy"; the whole folder
# name for players.txt-mapped folders); "text" = write as is; absent = not written.
# name and shirt_name are used only when the export's root holds the `write_names` marker;
# otherwise they are ignored and the names in the save stand.
name = "Snuffy"
shirt_name = "SNUFFY"           # "text" = write as is; absent = not written
boots_id = ""                   # "" = default (fpc_on: hidden, fpc_off 0, no marker 0 or in a Midcup export the installed ID); 0 to 100, a stock boots model; ignored when the folder's boots models or a boots link get an ID (pre-Fox face-XML boots do not)
gloves_id = ""                  # "" = default (fpc_on: hidden, fpc_off 0, no marker 0 or in a Midcup export the installed ID); 0 to 100, a stock gloves model (0 = normal hands); ignored when the folder's gloves models or a gloves link get an ID (pre-Fox face-XML gloves do not)

[appearance]
skin_color = 1                  # 0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)
iris_color = 0                  # 0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet

[appearance.physique]
neck_length = 0                 # -7 to 7
neck_size = 0                   # -7 to 7
shoulder_height = 0             # -7 to 7
shoulder_width = 0              # -7 to 7
chest = 0                       # -7 to 7
waist = 0                       # -7 to 7
arm_size = 0                    # -7 to 7
arm_length = 0                  # -7 to 7
thigh = 0                       # -7 to 7
calf = 0                        # -7 to 7
leg_length = 0                  # -7 to 7
head_length = 0                 # -7 to 7
head_width = 0                  # -7 to 7
head_depth = 0                  # -7 to 7

[appearance.strip]
sleeves = "short"               # "seasonal", "short", "long"
inners = "off"                  # "off", "normal", "turtleneck"
socks = "standard"              # "standard", "long", "short"
undershorts = "off"             # "off", "short", "winter_long" (none in summer, long in winter), "short_winter_long" (short in summer, long in winter)
untucked = true                 # true, false
ankle_taping = false            # true, false
wrist_taping = "off"            # "off", "right", "left", "both"
wrist_tape_color_left = 0       # 0 to 7
wrist_tape_color_right = 0      # 0 to 7
spectacles = 0                  # 0 none, 1 rectangle rimless, 2 rectangle half frame, 3 rectangle full frame, 4 oval rimless, 5 oval half frame, 6 oval full frame, 7 round full frame
spectacles_color = 0            # 0 white, 1 black, 2 red, 3 blue, 4 yellow, 5 green, 6 pink, 7 turquoise
gloves = false                  # true, false (outfield player gloves)
gloves_color = 0                # 0 to 7

[appearance.motion]             # on Fox written by the aesthetics patch: the tables do not hold motions
hunching_dribbling = 1          # 1 to 3 (PES 20 and 21: 1 to 5)
hunching_running = 1            # 1 to 3 (PES 20 and 21: 1 to 5)
arm_movement_dribbling = 1      # 1 to 8 (PES 15: 1 to 5; PES 20 and 21: 1 to 10)
arm_movement_running = 1        # 1 to 8 (PES 15: 1 to 5; PES 20 and 21: 1 to 10)
corner_kick = 1                 # 1 to 6 (PES 20 and 21: 1 to 10)
free_kick = 1                   # 1 to 16 (PES 20 and 21: 1 to 20)
penalty_kick = 1                # 1 to 4 (PES 20 and 21: 1 to 7)
dribbling = 0                   # 0 to 3, PES 20 and 21 only
goal_celebration_1 = 0          # 0 none, then PES 15: 1 to 112, 16 and 17: 1 to 122, 18: 1 to 127, 19: 1 to 140, 20: 1 to 152, 21: 1 to 162
goal_celebration_2 = 0          # 0 none, then PES 15: 1 to 112, 16 and 17: 1 to 122, 18: 1 to 127, 19: 1 to 140, 20: 1 to 152, 21: 1 to 162

[appearance.face]
jawline = 0                     # -7 to 7
jaw_height = 0                  # -7 to 7
cheekbones = 0                  # -7 to 7
cheekbone_height = 0            # -7 to 7 (PES 16 to 21: no editor slider)
chin_width = 0                  # -7 to 7
chin_height = 0                 # -7 to 7
chin_depth = 0                  # -7 to 7
face_size = 0                   # -7 to 7
face_height = 0                 # -7 to 7
brow_width = 0                  # -7 to 7
inner_eyebrow_height = 0        # -7 to 7
eyebrow_depth = 0               # -7 to 7
temple_width = 0                # -7 to 7
outer_eyebrow_height = 0        # -7 to 7
pupil_size = 0                  # -7 to 7
eye_depth = 0                   # -7 to 7
inner_eye_position = 0          # -7 to 7
inner_eye_height = 0            # -7 to 7
outer_eye_position = 0          # -7 to 7
eye_corner_height = 0           # -7 to 7
upper_eyelid_width_inner = 0    # -7 to 7
upper_eyelid_height_inner = 0   # -7 to 7
upper_eyelid_width_outer = 0    # -7 to 7
upper_eyelid_height_outer = 0   # -7 to 7
bottom_eyelid_height = 0        # -7 to 7
horizontal_eye_position = 0     # -7 to 7
eye_height = 0                  # -7 to 7
nose_height = 0                 # -7 to 7
nose_tip_depth = 0              # -7 to 7
nostril_width = 0               # -7 to 7
nose_depth = 0                  # -7 to 7
nose_width = 0                  # -7 to 7
mouth_height = 0                # -7 to 7
mouth_depth = 0                 # -7 to 7
lip_size = 0                    # -7 to 7
mouth_corner_height = 0         # -7 to 7
lip_width = 0                   # -7 to 7
ear_length = 0                  # -7 to 7
ear_width = 0                   # -7 to 7
ear_angle = 0                   # -7 to 7
pes15_jaw_depth = 0             # -7 to 7, drawn by PES 15 only
pes15_cheekbone_depth = 0       # -7 to 7, drawn by PES 15 only
pes15_mouth_edge_position = 0   # -7 to 7, drawn by PES 15 only
pes15_eyebrow_depth = 0         # -7 to 7, drawn by PES 15 only
pes15_bottom_eyelid_position = 0 # -7 to 7, drawn by PES 15 only
pes15_nasal_bridge_position = 0 # -7 to 7, drawn by PES 15 only
pes15_nasal_root_height = 0     # -7 to 7, drawn by PES 15 only
cheek_type = 0                  # 0 to 3 (PES 15: 0 to 14; PES 19: 0 to 5)
forehead_type = 0               # 0 to 5 (PES 15 and 19: 0 to 6)
facial_hair_type = 0            # 0 to 12 (PES 15: 0 to 29; PES 20 and 21: 0 to 19)
laughter_lines_type = 0         # 0 to 4 (PES 15: 0 to 6)
upper_eyelid_type = 0           # 0 to 6 (PES 15: 0 to 14; PES 20 and 21: 0 to 7)
lower_eyelid_type = 0           # 0 to 2 (PES 15: 0 to 14; PES 20 and 21: 0 to 6)
eyebrow_type = 0                # 0 to 5 (PES 15: 0 to 17; PES 20 and 21: 0 to 7)
neck_line_type = 0              # 0 to 2 (PES 15: 0 to 5; PES 20 and 21: 0 to 3)
nose_type = 0                   # 0 to 6 (PES 15: 0 to 11; PES 20 and 21: 0 to 7)
pes15_nose_angle = 0            # -1 to 1; PES 15 only
upper_lip_type = 0              # 0 to 3 (PES 15: 0 to 7; PES 20 and 21: 0 to 4)
lower_lip_type = 0              # 0 to 2 (PES 15: 0 to 7; PES 20 and 21: 0 to 4)
eyebrow_color_red = 14          # 0 to 63
eyebrow_color_green = 10        # 0 to 63
eyebrow_color_blue = 7          # 0 to 63
eyebrow_style = 0               # 0 normal, 1 thick, 2 thin
eyebrow_density = 3             # 0 to 3
eyebrow_thickness = 1           # 0 to 3
facial_hair_color_red = 14      # 0 to 63
facial_hair_color_green = 10    # 0 to 63
facial_hair_color_blue = 7      # 0 to 63
facial_hair_thickness = 3       # 0 to 3
pes15_eye_white = 0             # 0 to 3, brighter eye whites, drawn by PES 15 only

[appearance.hair]
color_red = 14                  # 0 to 63
color_green = 10                # 0 to 63
color_blue = 7                  # 0 to 63
color_preset = 0                # 0 custom (the colour above), 1 to 8 the editor's presets
accessory = false               # true, false (PES 15: the game sets it with the variation)
accessory_color = 0             # 0 white, 1 dark grey, 2 red, 3 blue, 4 yellow, 5 green, 6 magenta, 7 cyan
style = 1                       # 1 normal, 2 dry, 3 mohican, 4 afro, 5 dreadlocks, 6 braided, 7 special, 0 in some saves; PES 16 to 21
cropped_style = 1               # 1 to 6, 0 in some saves; PES 16 to 21
length = 2                      # 1 shaven, 2 very short, 3 short, 4 medium, 5 long, 0 in some saves; PES 16 to 21
wave_level = 0                  # 0 to 6, -1 in some saves; PES 16 to 21
variation = 0                   # 0 to 29; PES 16 to 21
front_style = 1                 # 1 up, 2 down, 3 combed back, 0 in some saves; PES 16 to 21
parted = 1                      # 1 off, 2 left 2, 3 left 1, 4 centre, 5 right 1, 6 right 2, 0 in some saves; PES 16 to 21
side_back_style = 4             # 1 normal, 2 reduce body, 3 shorter sides, 4 cropped, 0 in some saves; PES 16 to 21
hairline = 1                    # 1 to 3, 0 in some saves; PES 16 to 21
forehead_width = 2              # 1 narrow, 2 normal, 3 wide, 0 in some saves; PES 16 to 21
pes15_length = 2                # 1 skinhead, 2 very short, 3 short, 4 medium, 5 long, 6 special; PES 15 only
pes15_type = 1                  # 0 none (skinhead, special), 1 straight, 2 wavy, 3 permed, 4 dreadlocks; PES 15 only
pes15_variation = 1             # 1 to 82, the editor's grid position (each length and type has its own grid); PES 15 only
"#;

#[test]
fn to_toml_of_the_parsed_plan_block_is_the_plan_block() {
    let settings = PlayerSettings::parse(PLAN_BLOCK).expect("the plan block parses");
    assert_eq!(
        settings.name,
        Some(NameSetting::Explicit("Snuffy".to_string()))
    );
    assert_eq!(settings.shirt_name, Some("SNUFFY".to_string()));
    assert_eq!(settings.to_toml().expect("emit"), PLAN_BLOCK);
}

/// Every value the plan block shows is the key's default: the block is the
/// spec, `KeySpec::default` its stored form (a physique `0` is stored 7, a
/// motion `1` stored 0, a label its index). `""` is no stored value, so the
/// two stock IDs are `None` like an absent key.
#[test]
fn the_plan_block_shows_every_keys_default() {
    let settings = PlayerSettings::parse(PLAN_BLOCK).expect("the plan block parses");
    for key in SettingKey::ALL {
        assert_eq!(settings.get(key), key.spec().default, "{key:?}");
    }
}

#[test]
fn to_toml_of_a_default_is_a_fully_commented_template() {
    let text = PlayerSettings::default().to_toml().expect("emit");
    for key in SettingKey::ALL {
        let spec = key.spec();
        if spec.table == KeyTable::Top {
            // The stock-model IDs are never commented: `""` is the shown
            // default value.
            assert!(
                text.contains(&format!("{} = \"\"", spec.name)),
                "{key:?} is not an uncommented \"\" line"
            );
        } else {
            assert!(
                text.contains(&format!("# {} = ", spec.name)),
                "{key:?} is not a commented line"
            );
        }
        assert!(text.contains(spec.comment), "{key:?} comment absent");
    }
    assert!(text.contains("# name = true"), "name is commented");
    // Every key's line carries its default, the template's shown value.
    for (name, prefix) in [
        ("skin_color", "# skin_color = 1"),
        ("neck_length", "# neck_length = 0"),
        ("hunching_dribbling", "# hunching_dribbling = 1"),
        ("untucked", "# untucked = true"),
        ("sleeves", "# sleeves = \"short\""),
        ("name", "# name = true"),
        ("shirt_name", "# shirt_name = \"\""),
        ("boots_id", "boots_id = \"\""),
        ("gloves_id", "gloves_id = \"\""),
    ] {
        let line = text
            .lines()
            .find(|line| line.contains(&format!("{name} =")))
            .unwrap_or_else(|| panic!("a line for {name}"));
        assert!(line.starts_with(prefix), "{line:?}");
    }
    assert_eq!(
        PlayerSettings::parse(&text).expect("a commented file parses"),
        PlayerSettings::default(),
        "a commented file is an empty settings"
    );
    // No `[appearance.unknown]` table, not even commented: a template for a
    // new player folder has no player's bits to carry.
    assert!(
        !text.lines().any(|line| line.contains("unknown")),
        "an unknown line emitted: {text}"
    );
}

#[test]
fn name_true_parses_and_emits_true() {
    let settings = PlayerSettings::parse("name = true\n").expect("parses");
    assert_eq!(settings.name, Some(NameSetting::FromFolder));
    assert!(
        settings
            .to_toml()
            .expect("emit")
            .lines()
            .any(|line| line == "name = true"),
        "{}",
        settings.to_toml().expect("emit")
    );
}

#[test]
fn a_stored_value_the_kind_cannot_represent_is_out_of_range() {
    let mut player = find_player(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        70101,
    );
    player.appearance.sleeves = 3;
    match PlayerSettings::from_player(&player) {
        Err(SettingsError::OutOfRange { key, value, .. }) => {
            assert_eq!(key, "appearance.strip.sleeves");
            assert_eq!(value, 3);
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
}

#[test]
fn update_toml_refuses_a_table_position_holding_a_value() {
    let settings = PlayerSettings::parse(PLAN_BLOCK).expect("the plan block parses");
    for (text, key) in [
        ("appearance = 5\n", "appearance"),
        ("[appearance]\nstrip = 1\n", "appearance.strip"),
    ] {
        let mut document: DocumentMut = text.parse().expect("document parses");
        match settings.update_toml(&mut document) {
            Err(SettingsError::WrongType { key: got, expected }) => {
                assert_eq!(got, key, "{text:?}");
                assert_eq!(expected, "a table");
            }
            other => panic!("{text:?}: expected WrongType, got {other:?}"),
        }
    }
}

#[test]
fn to_toml_round_trips_a_real_player_through_text() {
    for version in [PesVersion::Pes19, PesVersion::Pes15] {
        let player = find_player(&payload(version), schema_for(version), 70101);
        let settings = PlayerSettings::from_player(&player).expect("from_player");
        let text = settings.to_toml().expect("emit");
        let reparsed = PlayerSettings::parse(&text).expect("the emitted text parses");
        assert_eq!(reparsed, settings, "{version:?} player 70101 through text");
        assert_eq!(
            reparsed.shirt_name,
            Some(player.shirt_name.clone()),
            "{version:?} shirt name through text"
        );
        if version == PesVersion::Pes15 {
            assert!(
                text.lines().any(|line| line.starts_with("# dribbling =")),
                "dribbling stays commented on PES 15"
            );
        }
    }
}

/// `[appearance.unknown]` left the format once every bit of the run was a
/// key or measured unused: it is an unknown table like any other.
#[test]
fn appearance_unknown_is_an_unknown_table() {
    let err = PlayerSettings::parse(&format!(
        "[appearance.unknown]\nface = \"{}\"\n",
        "0".repeat(100)
    ))
    .expect_err("a retired table");
    assert!(
        matches!(err, SettingsError::UnknownKey { ref key } if key == "appearance.unknown"),
        "{err:?}"
    );
}

/// The three kinds the face run brought: a signed angle stored plus 1, the
/// wave level stored plus 1, the colour preset through its map; each
/// emitted back as written, each refused past its range.
#[test]
fn the_face_runs_kinds_convert_both_ways() {
    let text = "[appearance.face]\npes15_nose_angle = -1\n\n\
                [appearance.hair]\ncolor_preset = 1\nwave_level = 0\n";
    let settings = PlayerSettings::parse(text).expect("parses");
    assert_eq!(settings.get(SettingKey::Pes15NoseAngle), Some(0));
    assert_eq!(
        settings.get(SettingKey::HairColorPreset),
        Some(5),
        "the map's index 1"
    );
    assert_eq!(settings.get(SettingKey::HairWaveLevel), Some(1));
    // Every editor preset and the id the game stores for it (measured in
    // the games), both ways.
    for (index, id) in [0u8, 5, 6, 7, 1, 3, 2, 8, 4].into_iter().enumerate() {
        let text = format!("[appearance.hair]\ncolor_preset = {index}\n");
        let preset = PlayerSettings::parse(&text).expect("parses");
        assert_eq!(preset.get(SettingKey::HairColorPreset), Some(id), "{text}");
        let emitted = preset.to_toml().expect("emits");
        assert!(
            emitted.contains(&format!("\ncolor_preset = {index} ")),
            "{text}{emitted}"
        );
    }
    let emitted = settings.to_toml().expect("emits");
    for line in [
        "pes15_nose_angle = -1",
        "color_preset = 1",
        "wave_level = 0",
    ] {
        assert!(
            emitted.lines().any(|l| l.starts_with(line)),
            "{line} in {emitted}"
        );
    }
    for (text, key, range) in [
        (
            "[appearance.face]\npes15_nose_angle = 2\n",
            "appearance.face.pes15_nose_angle",
            "-1 to 1",
        ),
        (
            "[appearance.hair]\ncolor_preset = 9\n",
            "appearance.hair.color_preset",
            "0 to 8",
        ),
        (
            "[appearance.hair]\nwave_level = 7\n",
            "appearance.hair.wave_level",
            "-1 to 6",
        ),
        (
            "[appearance.hair]\nwave_level = -2\n",
            "appearance.hair.wave_level",
            "-1 to 6",
        ),
    ] {
        match PlayerSettings::parse(text) {
            Err(SettingsError::OutOfRange {
                key: k, range: r, ..
            }) => {
                assert_eq!((k.as_str(), r.as_str()), (key, range), "{text}");
            }
            other => panic!("{text}: expected OutOfRange, got {other:?}"),
        }
    }
}

/// The hair keys' ranges, copied from the plan's `[appearance.hair]`
/// template block: each key's maximum parses, one past it is `OutOfRange`
/// with the plan's range, and so is one below a range that starts at 1.
/// (`wave_level` and `color_preset` are pinned above.)
#[test]
fn the_hair_keys_ranges_are_the_plans() {
    let out_of_range = |key: &str, value: i64, range: &str| {
        let text = format!("[appearance.hair]\n{key} = {value}\n");
        match PlayerSettings::parse(&text) {
            Err(SettingsError::OutOfRange {
                key: k, range: r, ..
            }) => assert_eq!(
                (k.as_str(), r.as_str()),
                (format!("appearance.hair.{key}").as_str(), range),
                "{text}"
            ),
            other => panic!("{text}: expected OutOfRange, got {other:?}"),
        }
    };
    for (key, min, max) in [
        ("color_red", 0, 63),
        ("color_green", 0, 63),
        ("color_blue", 0, 63),
        ("accessory_color", 0, 7),
        ("style", 0, 7),
        ("cropped_style", 0, 6),
        ("length", 0, 5),
        ("variation", 0, 29),
        ("front_style", 0, 3),
        ("parted", 0, 6),
        ("side_back_style", 0, 4),
        ("hairline", 0, 3),
        ("forehead_width", 0, 3),
        ("pes15_length", 1, 6),
        ("pes15_type", 0, 4),
        ("pes15_variation", 1, 82),
    ] {
        let range = format!("{min} to {max}");
        PlayerSettings::parse(&format!("[appearance.hair]\n{key} = {max}\n"))
            .unwrap_or_else(|e| panic!("{key} = {max}: {e}"));
        out_of_range(key, max + 1, &range);
        if min == 1 {
            out_of_range(key, 0, &range);
        }
    }
}

#[test]
fn parse_reports_the_first_error_with_its_key() {
    let cases: [(&str, SettingsError); 13] = [
        (
            "appearance.beard = 1",
            SettingsError::UnknownKey {
                key: "appearance.beard".to_string(),
            },
        ),
        // Height and weight left the key table: the record fields they were
        // are gameplay, so a settings.toml naming either is a stranger.
        (
            "[appearance.physique]\nheight = 180\n",
            SettingsError::UnknownKey {
                key: "appearance.physique.height".to_string(),
            },
        ),
        (
            "[appearance.physique]\nweight = 75\n",
            SettingsError::UnknownKey {
                key: "appearance.physique.weight".to_string(),
            },
        ),
        (
            "[appearance.strip]\nboots_id = 5\n",
            SettingsError::UnknownKey {
                key: "appearance.strip.boots_id".to_string(),
            },
        ),
        (
            "[appearance]\nskin_color = 8\n",
            SettingsError::OutOfRange {
                key: "appearance.skin_color".to_string(),
                value: 8,
                range: "0 to 7".to_string(),
            },
        ),
        (
            "[appearance.physique]\nneck_length = 8\n",
            SettingsError::OutOfRange {
                key: "appearance.physique.neck_length".to_string(),
                value: 8,
                range: "-7 to 7".to_string(),
            },
        ),
        (
            "[appearance.motion]\nhunching_dribbling = 0\n",
            SettingsError::OutOfRange {
                key: "appearance.motion.hunching_dribbling".to_string(),
                value: 0,
                range: "1 to 5".to_string(),
            },
        ),
        (
            "[appearance.strip]\nsleeves = \"medium\"\n",
            SettingsError::UnknownLabel {
                key: "appearance.strip.sleeves".to_string(),
                label: "medium".to_string(),
                allowed: "\"seasonal\", \"short\", \"long\"".to_string(),
            },
        ),
        (
            "[appearance.strip]\nuntucked = 1\n",
            SettingsError::WrongType {
                key: "appearance.strip.untucked".to_string(),
                expected: "true or false",
            },
        ),
        (
            "name = false",
            SettingsError::WrongType {
                key: "name".to_string(),
                expected: "true or a string",
            },
        ),
        (
            "[appearance.physique]\nneck_length = 1.5\n",
            SettingsError::WrongType {
                key: "appearance.physique.neck_length".to_string(),
                expected: "an integer",
            },
        ),
        (
            "stats = {}",
            SettingsError::UnknownKey {
                key: "stats".to_string(),
            },
        ),
        ("this is not toml", SettingsError::Toml(String::new())),
    ];
    for (text, expected) in cases {
        let error = PlayerSettings::parse(text).expect_err(text);
        match (&error, &expected) {
            (SettingsError::Toml(_), SettingsError::Toml(_)) => {}
            _ => assert_eq!(error.to_string(), expected.to_string(), "parsing {text:?}"),
        }
    }
}

#[test]
fn update_toml_rewrites_values_and_keeps_the_users_comments() {
    let mut document: DocumentMut = "[appearance.strip]\n\
# the sleeves we want\n\
sleeves = \"short\"   # keep me\n\
"
    .parse()
    .expect("document parses");
    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::Sleeves, 2).expect("in range");
    settings.update_toml(&mut document).expect("update");
    assert_eq!(
        document.to_string(),
        "[appearance.strip]\n# the sleeves we want\nsleeves = \"long\"   # keep me\n"
    );

    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::CheekType, 2).expect("in range");
    settings.update_toml(&mut document).expect("update");
    assert!(
        document.to_string().contains("[appearance.face]"),
        "a missing table is created as a standard table: {}",
        document
    );
    assert_eq!(
        PlayerSettings::parse(&document.to_string())
            .expect("the updated document parses")
            .appearance
            .face
            .cheek_type,
        Some(2)
    );
}

#[test]
fn update_toml_keeps_an_inline_table_inline() {
    let mut document: DocumentMut = "appearance = { skin_color = 1 }\n"
        .parse()
        .expect("document parses");
    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::Sleeves, 2).expect("in range");
    settings.update_toml(&mut document).expect("update");
    let text = document.to_string();
    assert!(text.starts_with("appearance = {"), "{text}");
    let parsed = PlayerSettings::parse(&text).expect("the updated document parses");
    assert_eq!(parsed.appearance.skin_color, Some(1));
    assert_eq!(parsed.appearance.strip.sleeves, Some(2));
}

#[test]
fn a_value_the_kind_cannot_represent_is_refused_everywhere() {
    // `set` refuses it.
    let mut settings = PlayerSettings::default();
    match settings.set(SettingKey::Sleeves, 3) {
        Err(SettingsError::OutOfRange { key, value, .. }) => {
            assert_eq!(key, "appearance.strip.sleeves");
            assert_eq!(value, 3);
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
    // A public field written directly: `to_toml`, `update_toml` and `apply`
    // all refuse with the same error, and `apply` leaves the player alone.
    settings.appearance.strip.sleeves = Some(3);
    match settings.to_toml() {
        Err(SettingsError::OutOfRange { key, value, .. }) => {
            assert_eq!(key, "appearance.strip.sleeves");
            assert_eq!(value, 3);
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
    let mut document: DocumentMut = "[appearance.strip]\nsleeves = \"short\"\n"
        .parse()
        .expect("document parses");
    match settings.update_toml(&mut document) {
        Err(SettingsError::OutOfRange { key, .. }) => {
            assert_eq!(key, "appearance.strip.sleeves")
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
    let mut player = find_player(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        70101,
    );
    let before = player.clone();
    match settings.apply(&mut player, PesVersion::Pes19) {
        Err(SettingsError::OutOfRange { key, .. }) => {
            assert_eq!(key, "appearance.strip.sleeves")
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
    assert_eq!(player, before, "the refused write changed the player");
}

#[test]
fn apply_writes_a_set_shirt_name_and_leaves_an_unset_one() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut player = find_player(&payload, schema, 70101);
    let before = player.shirt_name.clone();
    PlayerSettings {
        shirt_name: Some("SNUFFY".to_string()),
        ..PlayerSettings::default()
    }
    .apply(&mut player, PesVersion::Pes19)
    .expect("apply");
    assert_eq!(player.shirt_name, "SNUFFY");
    let mut player = find_player(&payload, schema, 70101);
    PlayerSettings::default()
        .apply(&mut player, PesVersion::Pes19)
        .expect("apply");
    assert_eq!(
        player.shirt_name, before,
        "an unset shirt_name is untouched"
    );
}

/// A `name`/`shirt_name` at its field's capacity applies; one over is
/// `TextTooLong` naming the key, and the player is unchanged. The two
/// versions' shirt-name fields differ (18 bytes on PES 19, 61 on PES 21).
#[test]
fn a_name_over_the_versions_field_is_refused_on_two_versions() {
    assert_ne!(
        text_max(
            schema_for(PesVersion::Pes19).player.texts,
            PlayerText::ShirtName
        ),
        text_max(
            schema_for(PesVersion::Pes21).player.texts,
            PlayerText::ShirtName
        ),
        "the test needs two different field lengths"
    );
    for version in [PesVersion::Pes19, PesVersion::Pes21] {
        let payload = payload(version);
        let schema = schema_for(version);
        let texts = schema.player.texts;
        let name_max = text_max(texts, PlayerText::Name);
        let shirt_max = text_max(texts, PlayerText::ShirtName);

        // At capacity applies. The shirt name's last char is `é` (U+00E9, two
        // UTF-8 bytes), so the string is `shirt_max` chars over `shirt_max`
        // bytes: it fits because the single-byte field counts chars.
        let full_shirt = format!("{}é", "S".repeat(shirt_max - 1));
        let mut player = find_player(&payload, schema, 70101);
        PlayerSettings {
            name: Some(NameSetting::Explicit("N".repeat(name_max))),
            shirt_name: Some(full_shirt.clone()),
            ..PlayerSettings::default()
        }
        .apply(&mut player, version)
        .expect("at capacity applies");
        assert_eq!(player.name, "N".repeat(name_max));
        assert_eq!(player.shirt_name, full_shirt);

        // `name` counts UTF-8 bytes, not chars: `é` is two bytes, so a
        // `name` of `é`s (plus one ASCII char when the capacity is odd)
        // fills the field exactly and applies; one `é` more still fits
        // in chars but its bytes do not.
        let full_name = if name_max.is_multiple_of(2) {
            "é".repeat(name_max / 2)
        } else {
            format!("N{}", "é".repeat(name_max / 2))
        };
        assert_eq!(
            full_name.len(),
            name_max,
            "the name fills the byte capacity exactly"
        );
        let mut player = find_player(&payload, schema, 70101);
        PlayerSettings {
            name: Some(NameSetting::Explicit(full_name.clone())),
            ..PlayerSettings::default()
        }
        .apply(&mut player, version)
        .expect("a name at the byte capacity applies");
        assert_eq!(player.name, full_name);
        let mut player = find_player(&payload, schema, 70101);
        let before = player.clone();
        match (PlayerSettings {
            name: Some(NameSetting::Explicit(format!("{full_name}é"))),
            ..PlayerSettings::default()
        })
        .apply(&mut player, version)
        {
            Err(SettingsError::TextTooLong {
                key: got,
                max: got_max,
            }) => assert_eq!((got.as_str(), got_max), ("name", name_max), "{version:?}"),
            other => panic!("{version:?} name: expected TextTooLong, got {other:?}"),
        }
        assert_eq!(player, before, "the refused write changed the player");

        for (settings, key, max) in [
            (
                PlayerSettings {
                    name: Some(NameSetting::Explicit("N".repeat(name_max + 1))),
                    ..PlayerSettings::default()
                },
                "name",
                name_max,
            ),
            (
                PlayerSettings {
                    shirt_name: Some("S".repeat(shirt_max + 1)),
                    ..PlayerSettings::default()
                },
                "shirt_name",
                shirt_max,
            ),
        ] {
            let mut player = find_player(&payload, schema, 70101);
            let before = player.clone();
            match settings.apply(&mut player, version) {
                Err(SettingsError::TextTooLong {
                    key: got,
                    max: got_max,
                }) => {
                    assert_eq!((got.as_str(), got_max), (key, max), "{version:?}");
                }
                other => panic!("{version:?} {key}: expected TextTooLong, got {other:?}"),
            }
            assert_eq!(player, before, "the refused write changed the player");
        }
    }
}

/// `NameField::check` is `apply`'s text check standalone: at the field's
/// capacity passes, one byte longer is `TextTooLong` naming the key.
#[test]
fn name_field_check_is_applys_check() {
    let texts = schema_for(PesVersion::Pes21).player.texts;
    for (field, max) in [
        (NameField::Name, text_max(texts, PlayerText::Name)),
        (NameField::ShirtName, text_max(texts, PlayerText::ShirtName)),
    ] {
        let key = field.key();
        field
            .check(&"N".repeat(max), PesVersion::Pes21)
            .expect("at capacity");
        match field.check(&"N".repeat(max + 1), PesVersion::Pes21) {
            Err(SettingsError::TextTooLong {
                key: got,
                max: got_max,
            }) => assert_eq!((got.as_str(), got_max), (key, max)),
            other => panic!("{key}: expected TextTooLong, got {other:?}"),
        }
    }
}

#[test]
fn a_shirt_name_that_is_not_a_plain_string_is_refused() {
    // A char above U+00FF does not fit the single-byte field: `WrongType` at
    // apply, and the player is unchanged.
    let mut player = find_player(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        70101,
    );
    let before = player.clone();
    match (PlayerSettings {
        shirt_name: Some("Ł".to_string()),
        ..PlayerSettings::default()
    })
    .apply(&mut player, PesVersion::Pes19)
    {
        Err(SettingsError::WrongType { key, expected }) => {
            assert_eq!(key, "shirt_name");
            assert_eq!(expected, "a string of chars at most U+00FF");
        }
        other => panic!("expected WrongType, got {other:?}"),
    }
    assert_eq!(player, before, "the refused write changed the player");

    // At parse: a NUL is refused as `name`'s is, a non-string as `a string`.
    match PlayerSettings::parse("shirt_name = \"a\\u0000b\"\n") {
        Err(SettingsError::WrongType { key, expected }) => {
            assert_eq!(key, "shirt_name");
            assert_eq!(expected, "a string without NUL");
        }
        other => panic!("expected WrongType, got {other:?}"),
    }
    match PlayerSettings::parse("shirt_name = 5\n") {
        Err(SettingsError::WrongType { key, expected }) => {
            assert_eq!(key, "shirt_name");
            assert_eq!(expected, "a string");
        }
        other => panic!("expected WrongType, got {other:?}"),
    }
}

#[test]
fn update_toml_writes_a_set_shirt_name_and_keeps_the_comment() {
    let mut document: DocumentMut = "shirt_name = \"OLD\"   # keep me\n"
        .parse()
        .expect("document parses");
    PlayerSettings {
        shirt_name: Some("SNUFFY".to_string()),
        ..PlayerSettings::default()
    }
    .update_toml(&mut document)
    .expect("update");
    assert_eq!(
        document.to_string(),
        "shirt_name = \"SNUFFY\"   # keep me\n"
    );
}

#[test]
fn an_explicit_name_with_nul_is_refused() {
    match PlayerSettings::parse("name = \"a\\u0000b\"\n") {
        Err(SettingsError::WrongType { key, expected }) => {
            assert_eq!(key, "name");
            assert_eq!(expected, "a string without NUL");
        }
        other => panic!("expected WrongType, got {other:?}"),
    }
}

#[test]
fn accepts_stored_bounds_each_kind() {
    for (kind, accepted, refused) in [
        (Kind::Number { min: 0, max: 7 }, 7, 8),
        (Kind::Signed { limit: 7 }, 14, 15),
        (Kind::Signed { limit: 1 }, 2, 3),
        (Kind::OneBased { max: 5 }, 4, 5),
        (Kind::PlusOne { max: 6 }, 7, 8),
        (Kind::PlusOne { max: 6 }, 0, 8),
        (Kind::Mapped(&[0, 5, 6, 7, 1, 3, 2, 8, 4]), 8, 9),
        (Kind::Bool, 1, 2),
        (Kind::Labels(&["a", "b"]), 1, 2),
    ] {
        assert!(kind.accepts_stored(accepted), "{kind:?} {accepted}");
        assert!(!kind.accepts_stored(refused), "{kind:?} {refused}");
    }
}

#[test]
fn parse_maps_the_toml_values_to_the_stored_ones() {
    let settings = PlayerSettings::parse(
        "[appearance.physique]\nneck_length = -7\n\
[appearance.motion]\nfree_kick = 20\n\
[appearance.strip]\nwrist_taping = \"both\"\n",
    )
    .expect("parses");
    assert_eq!(settings.get(SettingKey::NeckLength), Some(0));
    assert_eq!(settings.get(SettingKey::FreeKick), Some(19));
    assert_eq!(settings.get(SettingKey::WristTaping), Some(3));
    let settings =
        PlayerSettings::parse("[appearance.physique]\nneck_length = 7\n").expect("parses");
    assert_eq!(settings.get(SettingKey::NeckLength), Some(14));
}

#[test]
fn stock_model_ids_parse_the_stock_band_and_the_default() {
    // 0 and 100 are the stock band's ends; `""` is default.
    let settings = PlayerSettings::parse("boots_id = 0\ngloves_id = 100\n").expect("parses");
    assert_eq!(settings.boots_id, Some(0));
    assert_eq!(settings.gloves_id, Some(100));
    let settings = PlayerSettings::parse("boots_id = \"\"\n").expect("parses");
    assert_eq!(settings.boots_id, None);
    // An authored stock ID emits back as the integer.
    let text = PlayerSettings::parse("boots_id = 42\n")
        .expect("parses")
        .to_toml()
        .expect("emit");
    assert!(text.lines().any(|line| line.starts_with("boots_id = 42")));
    // 101 is past the stock band: the compiler's first team block.
    match PlayerSettings::parse("boots_id = 101\n") {
        Err(SettingsError::OutOfRange { key, value, range }) => {
            assert_eq!(
                (key.as_str(), value, range.as_str()),
                ("boots_id", 101, "0 to 100")
            );
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
    // A non-empty string is neither an integer nor the default marker.
    match PlayerSettings::parse("gloves_id = \"12\"\n") {
        Err(SettingsError::WrongType { key, expected }) => {
            assert_eq!(
                (key.as_str(), expected),
                ("gloves_id", "an integer or \"\"")
            );
        }
        other => panic!("expected WrongType, got {other:?}"),
    }
}

#[test]
fn a_stock_model_id_inside_a_table_is_an_unknown_key() {
    // The keys sit at the file's top level only: the same name inside
    // `[appearance]` or `[appearance.strip]` collides with Team TOML's
    // embedded appearance tables and is refused, naming the key.
    for (text, key) in [
        ("[appearance]\nboots_id = 5\n", "appearance.boots_id"),
        (
            "[appearance.strip]\ngloves_id = 5\n",
            "appearance.strip.gloves_id",
        ),
    ] {
        match PlayerSettings::parse(text) {
            Err(SettingsError::UnknownKey { key: got }) => assert_eq!(got, key),
            other => panic!("{text:?}: expected UnknownKey, got {other:?}"),
        }
    }
}

#[test]
fn from_player_emits_an_id_only_in_the_stock_band() {
    // 0 is the game's default.
    let p21 = find_player(
        &payload(PesVersion::Pes21),
        schema_for(PesVersion::Pes21),
        70101,
    );
    assert_eq!((p21.appearance.boots_id, p21.appearance.gloves_id), (0, 0));
    let s21 = PlayerSettings::from_player(&p21).expect("from_player");
    assert_eq!((s21.boots_id, s21.gloves_id), (None, None));
    // Above 100 is custom content a folder owns, not an authorable setting.
    let p19 = find_player(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        70101,
    );
    assert_eq!(
        (p19.appearance.boots_id, p19.appearance.gloves_id),
        (101, 101)
    );
    let s19 = PlayerSettings::from_player(&p19).expect("from_player");
    assert_eq!((s19.boots_id, s19.gloves_id), (None, None));
    // A stock ID in 1 to 100 emits.
    let stock = find_player(
        &payload(PesVersion::Pes21),
        schema_for(PesVersion::Pes21),
        79503,
    );
    assert_eq!(stock.appearance.gloves_id, 12);
    assert!(stock.appearance.boots_id > 100);
    let emitted = PlayerSettings::from_player(&stock).expect("from_player");
    assert_eq!((emitted.boots_id, emitted.gloves_id), (None, Some(12)));
    // The band's own ends.
    let mut player = p21.clone();
    player.appearance.boots_id = 1;
    player.appearance.gloves_id = 100;
    let emitted = PlayerSettings::from_player(&player).expect("from_player");
    assert_eq!((emitted.boots_id, emitted.gloves_id), (Some(1), Some(100)));
}

#[test]
fn apply_writes_an_authored_id_and_leaves_none_alone() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut player = find_player(&payload, schema, 70101);
    PlayerSettings {
        boots_id: Some(42),
        ..PlayerSettings::default()
    }
    .apply(&mut player, PesVersion::Pes19)
    .expect("apply");
    assert_eq!(player.appearance.boots_id, 42);
    assert_eq!(player.appearance.gloves_id, 101, "an unset ID is untouched");
    // `None` leaves even the game's default alone.
    let mut player = find_player(&payload, schema, 70101);
    PlayerSettings::default()
        .apply(&mut player, PesVersion::Pes19)
        .expect("apply");
    assert_eq!(
        (player.appearance.boots_id, player.appearance.gloves_id),
        (101, 101)
    );
    // `""` parses to `None` and likewise leaves the field alone.
    PlayerSettings::parse("boots_id = \"\"\n")
        .expect("parses")
        .apply(&mut player, PesVersion::Pes19)
        .expect("apply");
    assert_eq!(player.appearance.boots_id, 101);
}

#[test]
fn update_toml_writes_an_authored_id_and_keeps_the_users_default() {
    let mut document: DocumentMut = "boots_id = \"\"  # the boots comment\n\
gloves_id = \"\"   # the gloves comment\n"
        .parse()
        .expect("document parses");
    PlayerSettings {
        boots_id: Some(7),
        ..PlayerSettings::default()
    }
    .update_toml(&mut document)
    .expect("update");
    assert_eq!(
        document.to_string(),
        "boots_id = 7  # the boots comment\ngloves_id = \"\"   # the gloves comment\n"
    );
}

#[test]
fn a_hand_set_id_past_the_stock_band_is_refused_on_write() {
    // The key's 0-100 range, not the field's stored width, bounds a
    // `boots_id`/`gloves_id` written directly into a public field: 150
    // fits the storage's 14 bits but is not a stock ID.
    for (settings, name) in [
        (
            PlayerSettings {
                boots_id: Some(150),
                ..PlayerSettings::default()
            },
            "boots_id",
        ),
        (
            PlayerSettings {
                gloves_id: Some(150),
                ..PlayerSettings::default()
            },
            "gloves_id",
        ),
    ] {
        for emit in [
            settings.to_toml().map(|_| ()),
            settings.update_toml(&mut DocumentMut::new()),
        ] {
            match emit {
                Err(SettingsError::OutOfRange { key, value, range }) => {
                    assert_eq!(
                        (key.as_str(), value, range.as_str()),
                        (name, 150, "0 to 100")
                    );
                }
                other => panic!("expected OutOfRange, got {other:?}"),
            }
        }
    }
}
