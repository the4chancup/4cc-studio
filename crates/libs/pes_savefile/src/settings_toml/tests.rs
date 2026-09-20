//! The `PlayerSettings` tests: the completeness invariant, `from_player`,
//! `apply`, the key table itself, and the TOML text half.

use std::collections::HashSet;

use pes_version::PesVersion;
use toml_edit::DocumentMut;

use super::keys::Source;
use super::{
    AppearanceSettings, Kind, NameSetting, Ownership, PlayerSettings, SettingKey, SettingsError,
    face_ownership, ownership,
};
use crate::codec::{read_player, runs, write_player};
use crate::model::ingame_face::IngameFace;
use crate::model::player::PlayerAppearance;
use crate::schema::fields::PlayerField;
use crate::schema::ingame_face::{INGAME_FACE_FIELDS, IngameFaceField};
use crate::schema::schema_for;
use crate::test_support::{count, find_player, payload, record};

#[test]
fn every_settings_field_is_a_key_and_every_key_a_settings_field() {
    let mut fields = HashSet::new();
    for version in PesVersion::ALL {
        let schema = schema_for(version);
        fields.extend(runs(schema.player).map(|(f, _, _)| f));
        if let Some((_, appearance)) = &schema.appearance {
            fields.extend(runs(*appearance).map(|(f, _, _)| f));
        }
    }
    let mut settings = HashSet::new();
    for field in fields {
        if ownership(field) == Ownership::Settings {
            settings.insert(Source::Player(field));
        }
    }
    for field in IngameFaceField::ALL {
        if face_ownership(field) == Ownership::Settings {
            settings.insert(Source::Face(field));
        }
    }
    let keys: HashSet<Source> = SettingKey::ALL.iter().map(|k| k.source()).collect();
    let without_key: Vec<_> = settings.difference(&keys).collect();
    let without_field: Vec<_> = keys.difference(&settings).collect();
    assert!(
        without_key.is_empty() && without_field.is_empty(),
        "settings fields without a key: {without_key:?}; keys without a field: {without_field:?}"
    );
    // No key shares a (table, name) pair.
    let names: HashSet<(&str, &str)> = SettingKey::ALL
        .iter()
        .map(|k| (k.spec().table, k.spec().name))
        .collect();
    assert_eq!(
        names.len(),
        SettingKey::ALL.len(),
        "a duplicate (table, name)"
    );
}

#[test]
fn from_player_reads_every_key_and_the_gated_dribbling() {
    let p19 = find_player(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        70101,
    );
    let s19 = PlayerSettings::from_player(&p19).expect("from_player");
    let face = &p19.appearance.ingame_face;
    assert_eq!(
        s19.appearance.skin_color,
        Some(face.get(IngameFaceField::SkinColor).expect("read run"))
    );
    assert_eq!(
        s19.appearance.iris_color,
        Some(face.get(IngameFaceField::IrisColor).expect("read run"))
    );
    assert_eq!(
        s19.appearance.physique.neck_length,
        Some(p19.appearance.neck_length)
    );
    assert_eq!(s19.appearance.strip.sleeves, Some(p19.appearance.sleeves));
    assert_eq!(s19.appearance.motion.dribbling, None);
    assert_eq!(s19.name, Some(NameSetting::Explicit(p19.name.clone())));
    let p21 = find_player(
        &payload(PesVersion::Pes21),
        schema_for(PesVersion::Pes21),
        70101,
    );
    let s21 = PlayerSettings::from_player(&p21).expect("from_player");
    assert_eq!(s21.appearance.motion.dribbling, p21.motion.dribbling);
    assert!(s21.appearance.motion.dribbling.is_some());
}

/// The bits of a run the fifteen decoded face fields cover, as a per-byte
/// mask (run length <= 64 bytes).
fn face_field_mask(run_len: usize) -> Vec<u8> {
    let mut mask = vec![0u8; run_len];
    for spec in INGAME_FACE_FIELDS {
        for bit in spec.bit_offset..spec.bit_offset + spec.bit_width {
            mask[(bit / 8) as usize] |= 1 << (bit % 8);
        }
    }
    mask
}

#[test]
fn apply_copies_the_settings_and_leaves_the_rest() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let p = find_player(&payload, schema, 70101);
    let mut q = find_player(&payload, schema, 70102);

    // Flip one undecoded bit of q's run so the check below discriminates: a
    // wholesale run copy would overwrite it with p's bit.
    let mask = face_field_mask(q.appearance.ingame_face.bytes().len());
    let mut run = q.appearance.ingame_face.bytes().to_vec();
    let p_run = p.appearance.ingame_face.bytes();
    let mut flipped = false;
    'outer: for (byte, bits) in mask.iter().enumerate() {
        for bit in 0..8 {
            if bits & (1 << bit) == 0 && (run[byte] ^ p_run[byte]) & (1 << bit) == 0 {
                run[byte] ^= 1 << bit;
                flipped = true;
                break 'outer;
            }
        }
    }
    assert!(flipped, "no agreeing undecoded bit to flip");
    q.appearance.ingame_face = IngameFace::from_bytes(run);
    let q_before = q.clone();

    PlayerSettings::from_player(&p)
        .expect("from_player")
        .apply(&mut q)
        .expect("apply");

    // Every scalar appearance field and every decoded face field is p's.
    assert_eq!(
        q.appearance,
        PlayerAppearance {
            boots_id: q_before.appearance.boots_id,
            gloves_id: q_before.appearance.gloves_id,
            base_copy_id: q_before.appearance.base_copy_id,
            ingame_face: q.appearance.ingame_face.clone(),
            ..p.appearance.clone()
        }
    );
    for field in IngameFaceField::ALL {
        assert_eq!(
            q.appearance.ingame_face.get(field).expect("run"),
            p.appearance.ingame_face.get(field).expect("run"),
            "{field:?}"
        );
    }
    // The undecoded bits are carried, not authored: q's own, not p's.
    for (byte, (now, was)) in q
        .appearance
        .ingame_face
        .bytes()
        .iter()
        .zip(q_before.appearance.ingame_face.bytes())
        .enumerate()
    {
        assert_eq!(
            now & !mask[byte],
            was & !mask[byte],
            "undecoded bits of run byte {byte}"
        );
    }
    assert_eq!(q.motion, p.motion);
    assert_eq!(
        (q.basic.height, q.basic.weight),
        (p.basic.height, p.basic.weight)
    );
    assert_eq!(q.name, p.name);
    assert_eq!(q.id, q_before.id);
    assert_eq!(q.shirt_name, q_before.shirt_name);
    assert_eq!(q.stats, q_before.stats);
    assert_eq!(q.positions, q_before.positions);
    assert_eq!(q.skills, q_before.skills);
    assert_eq!(q.edit_flags, q_before.edit_flags);
    // q's nationality, age and the rest of basics are gameplay, untouched.
    assert_eq!(q.basic.nationality, q_before.basic.nationality);
    assert_eq!(q.basic.age, q_before.basic.age);
}

#[test]
fn every_key_writes_only_its_own_run() {
    let payload = payload(PesVersion::Pes21);
    let schema = schema_for(PesVersion::Pes21);
    let index = (0..count(&payload, &schema.players))
        .find(|&i| {
            read_player(
                record(&payload, &schema.players, schema.player.size, i),
                schema.player,
            )
            .expect("decode")
            .id == 70101
        })
        .expect("player 70101's record");
    let rec = record(&payload, &schema.players, schema.player.size, index);
    let run = schema.player.ingame_face.as_ref().expect("a run on PES 21");
    for key in SettingKey::ALL {
        let player = find_player(&payload, schema, 70101);
        let current = PlayerSettings::from_player(&player)
            .expect("from_player")
            .get(key);
        let new_value = match current {
            Some(0) | None => 1,
            _ => 0,
        };
        let mut settings = PlayerSettings::default();
        settings
            .set(key, new_value)
            .expect("the flipped value is in range");
        let mut modified = player.clone();
        settings.apply(&mut modified).expect("apply");
        let mut written = rec.to_vec();
        write_player(&modified, &mut written, schema.player).expect("encode");

        let (start, end) = match key.source() {
            Source::Player(field) => {
                let (_, offset, width) = runs(schema.player)
                    .find(|(f, _, _)| *f == field)
                    .unwrap_or_else(|| panic!("{key:?} source has no run"));
                (offset, offset + width)
            }
            Source::Face(field) => {
                let row = INGAME_FACE_FIELDS
                    .iter()
                    .find(|f| f.field == field)
                    .unwrap_or_else(|| panic!("{key:?} source has no row"));
                (
                    run.byte_offset * 8 + row.bit_offset,
                    run.byte_offset * 8 + row.bit_offset + row.bit_width,
                )
            }
        };
        let mut diffs = 0usize;
        for (byte, (was, now)) in rec.iter().zip(&written).enumerate() {
            for bit in 0..8 {
                if (was >> bit) & 1 != (now >> bit) & 1 {
                    let pos = byte as u32 * 8 + bit;
                    diffs += 1;
                    assert!(
                        start <= pos && pos < end,
                        "{key:?} changed bit {pos} outside its run {start}..{end}"
                    );
                }
            }
        }
        assert_ne!(diffs, 0, "{key:?} changed nothing");
        assert_eq!(
            PlayerSettings::from_player(&modified)
                .expect("from_player")
                .get(key),
            Some(new_value),
            "{key:?} reads back the new value"
        );
    }
}

#[test]
fn an_apply_then_write_touches_only_the_set_fields_bits() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut player = find_player(&payload, schema, 70101);
    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::SkinColor, 3).expect("in range");
    settings.set(SettingKey::NeckLength, 9).expect("in range");
    settings.apply(&mut player).expect("apply");

    let index = (0..count(&payload, &schema.players))
        .find(|&i| {
            read_player(
                record(&payload, &schema.players, schema.player.size, i),
                schema.player,
            )
            .expect("decode")
            .id == 70101
        })
        .expect("player 70101's record");
    let rec = record(&payload, &schema.players, schema.player.size, index);
    let mut written = rec.to_vec();
    write_player(&player, &mut written, schema.player).expect("encode");

    let run = schema.player.ingame_face.as_ref().expect("a run on PES 19");
    let skin = INGAME_FACE_FIELDS
        .iter()
        .find(|f| f.field == IngameFaceField::SkinColor)
        .expect("a skin row");
    let neck = schema
        .player
        .fields
        .iter()
        .find(|f| f.field == PlayerField::NeckLength)
        .expect("a neck_length row");
    let ranges = [
        (
            run.byte_offset * 8 + skin.bit_offset,
            run.byte_offset * 8 + skin.bit_offset + skin.bit_width,
        ),
        (neck.bit_offset, neck.bit_offset + neck.bit_width),
    ];
    let mut diffs = 0usize;
    for (byte, (was, now)) in rec.iter().zip(&written).enumerate() {
        for bit in 0..8 {
            if (was >> bit) & 1 != (now >> bit) & 1 {
                let pos = byte as u32 * 8 + bit;
                diffs += 1;
                assert!(
                    ranges.iter().any(|(s, e)| pos >= *s && pos < *e),
                    "bit {pos} changed outside the skin and neck runs"
                );
            }
        }
    }
    assert_ne!(diffs, 0, "the two settings changed something");
}

#[test]
fn a_setting_the_version_lacks_is_refused_and_changes_nothing() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut player = find_player(&payload, schema, 70101);
    let before = player.clone();
    let mut settings = PlayerSettings {
        name: Some(NameSetting::Explicit("RENAMED".to_string())),
        ..PlayerSettings::default()
    };
    settings.set(SettingKey::Sleeves, 2).expect("in range");
    settings.set(SettingKey::Dribbling, 1).expect("in range");
    match settings.apply(&mut player) {
        Err(SettingsError::NotInThisVersion { key }) => assert_eq!(key, "dribbling"),
        other => panic!("expected NotInThisVersion, got {other:?}"),
    }
    assert_eq!(player, before, "the refused write changed the player");
}

#[test]
fn get_after_set_round_trips_every_key() {
    for key in SettingKey::ALL {
        let mut settings = PlayerSettings::default();
        settings.set(key, 1).expect("in range");
        assert_eq!(settings.get(key), Some(1), "{key:?}");
    }
}

#[test]
fn the_key_specs_match_the_plan_table() {
    let comments: [&str; 52] = [
        "0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)",
        "0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet",
        "cm",
        "kg",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "-7 to 7",
        "\"seasonal\", \"short\", \"long\"",
        "\"off\", \"normal\", \"turtleneck\"",
        "\"standard\", \"long\", \"short\"",
        "\"off\", \"short\", \"winter_long\" (none in summer, long in winter), \"short_winter_long\" (short in summer, long in winter)",
        "true, false",
        "true, false",
        "\"off\", \"right\", \"left\", \"both\"",
        "0 to 7",
        "0 to 7",
        "0 none, 1 rectangle rimless, 2 rectangle half frame, 3 rectangle full frame, 4 oval rimless, 5 oval half frame, 6 oval full frame, 7 round full frame",
        "0 white, 1 black, 2 red, 3 blue, 4 yellow, 5 green, 6 pink, 7 turquoise",
        "true, false (outfield player gloves)",
        "0 to 7",
        "1 to 3 (PES 20 and 21: 1 to 5)",
        "1 to 3 (PES 20 and 21: 1 to 5)",
        "1 to 8 (PES 20 and 21: 1 to 10)",
        "1 to 8 (PES 20 and 21: 1 to 10)",
        "1 to 6 (PES 20 and 21: 1 to 10)",
        "1 to 16 (PES 20 and 21: 1 to 20)",
        "1 to 4 (PES 20 and 21: 1 to 7)",
        "0 to 3, PES 20 and 21 only",
        "0 none, 1 to 122 (PES 20 and 21: 1 to 162)",
        "0 none, 1 to 122 (PES 20 and 21: 1 to 162)",
        "0 to 3",
        "0 to 5",
        "0 to 12 (PES 20 and 21: 0 to 19)",
        "0 to 4",
        "0 to 6 (PES 20 and 21: 0 to 7)",
        "0 to 2 (PES 20 and 21: 0 to 6)",
        "0 to 5 (PES 20 and 21: 0 to 7)",
        "0 to 2 (PES 20 and 21: 0 to 3)",
        "0 to 6 (PES 20 and 21: 0 to 7)",
        "0 to 3 (PES 20 and 21: 0 to 4)",
        "0 to 2 (PES 20 and 21: 0 to 4)",
    ];
    let tables = [
        "appearance",
        "appearance.physique",
        "appearance.strip",
        "appearance.motion",
        "appearance.face",
    ];
    for (key, comment) in SettingKey::ALL.iter().zip(comments) {
        let spec = key.spec();
        assert_eq!(spec.comment, comment, "{key:?} comment");
        assert!(
            tables.contains(&spec.table),
            "{key:?} table {:?}",
            spec.table
        );
        match spec.kind {
            Kind::Labels(labels) => assert!(!labels.is_empty(), "{key:?} labels"),
            Kind::OneBased { max } => assert!(max >= 1, "{key:?} one-based max"),
            Kind::Number { min, max } => assert!(min <= max, "{key:?} number range"),
            Kind::Signed7 | Kind::Bool => {}
        }
    }
}

#[test]
fn an_unset_settings_is_empty_and_default() {
    let settings = PlayerSettings::default();
    for key in SettingKey::ALL {
        assert_eq!(settings.get(key), None, "{key:?}");
    }
    assert_eq!(settings.appearance, AppearanceSettings::default());
}

// The TOML half: the template, the parser's errors, `update_toml`.

/// The key-table block of `docs/plans/aesthetics_export/settings_toml.md`,
/// pasted verbatim: `to_toml` of the settings it parses to must reproduce it
/// byte for byte.
const PLAN_BLOCK: &str = r#"# settings.toml, inside a player folder. Every key is optional: an absent key leaves
# that savefile setting untouched. A commented key shows what can be set and its range.
# The file never references models: what a player wears is decided by the folder
# contents (models present, link files pointing at shared folders).

# true = derive from the folder name ("15 - Snuffy" gives "Snuffy"; the whole folder
# name for players.txt-mapped folders); "text" = write as is; absent = leave untouched.
name = "Snuffy"

[appearance]
skin_color = 1                  # 0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)
iris_color = 0                  # 0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet

[appearance.physique]
height = 180                    # cm
weight = 75                     # kg
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

[appearance.motion]
hunching_dribbling = 1          # 1 to 3 (PES 20 and 21: 1 to 5)
hunching_running = 1            # 1 to 3 (PES 20 and 21: 1 to 5)
arm_movement_dribbling = 1      # 1 to 8 (PES 20 and 21: 1 to 10)
arm_movement_running = 1        # 1 to 8 (PES 20 and 21: 1 to 10)
corner_kick = 1                 # 1 to 6 (PES 20 and 21: 1 to 10)
free_kick = 1                   # 1 to 16 (PES 20 and 21: 1 to 20)
penalty_kick = 1                # 1 to 4 (PES 20 and 21: 1 to 7)
dribbling = 0                   # 0 to 3, PES 20 and 21 only
goal_celebration_1 = 0          # 0 none, 1 to 122 (PES 20 and 21: 1 to 162)
goal_celebration_2 = 0          # 0 none, 1 to 122 (PES 20 and 21: 1 to 162)

[appearance.face]
cheek_type = 0                  # 0 to 3
forehead_type = 0               # 0 to 5
facial_hair_type = 0            # 0 to 12 (PES 20 and 21: 0 to 19)
laughter_lines_type = 0         # 0 to 4
upper_eyelid_type = 0           # 0 to 6 (PES 20 and 21: 0 to 7)
lower_eyelid_type = 0           # 0 to 2 (PES 20 and 21: 0 to 6)
eyebrow_type = 0                # 0 to 5 (PES 20 and 21: 0 to 7)
neck_line_type = 0              # 0 to 2 (PES 20 and 21: 0 to 3)
nose_type = 0                   # 0 to 6 (PES 20 and 21: 0 to 7)
upper_lip_type = 0              # 0 to 3 (PES 20 and 21: 0 to 4)
lower_lip_type = 0              # 0 to 2 (PES 20 and 21: 0 to 4)
"#;

#[test]
fn to_toml_of_the_parsed_plan_block_is_the_plan_block() {
    let settings = PlayerSettings::parse(PLAN_BLOCK).expect("the plan block parses");
    assert_eq!(
        settings.name,
        Some(NameSetting::Explicit("Snuffy".to_string()))
    );
    assert_eq!(settings.to_toml().expect("emit"), PLAN_BLOCK);
}

#[test]
fn to_toml_of_a_default_is_a_fully_commented_template() {
    let text = PlayerSettings::default().to_toml().expect("emit");
    for key in SettingKey::ALL {
        let spec = key.spec();
        assert!(
            text.contains(&format!("# {} = ", spec.name)),
            "{key:?} is not a commented line"
        );
        assert!(text.contains(spec.comment), "{key:?} comment absent");
    }
    assert!(text.contains("# name = true"), "name is commented");
    // Every kind's commented line carries its neutral example value.
    for (name, prefix) in [
        ("skin_color", "# skin_color = 0"),
        ("neck_length", "# neck_length = 0"),
        ("hunching_dribbling", "# hunching_dribbling = 1"),
        ("untucked", "# untucked = false"),
        ("sleeves", "# sleeves = \"seasonal\""),
        ("name", "# name = true"),
    ] {
        let line = text
            .lines()
            .find(|line| line.contains(&format!(" {name} =")))
            .unwrap_or_else(|| panic!("a line for {name}"));
        assert!(line.starts_with(prefix), "{line:?}");
    }
    assert_eq!(
        PlayerSettings::parse(&text).expect("a commented file parses"),
        PlayerSettings::default(),
        "a commented file is an empty settings"
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
        assert_eq!(
            PlayerSettings::parse(&text).expect("the emitted text parses"),
            settings,
            "{version:?} player 70101 through text"
        );
        if version == PesVersion::Pes15 {
            assert!(
                text.lines().any(|line| line.starts_with("# dribbling =")),
                "dribbling stays commented on PES 15"
            );
        }
    }
}

#[test]
fn parse_reports_the_first_error_with_its_key() {
    let cases: [(&str, SettingsError); 11] = [
        (
            "appearance.hair = 1",
            SettingsError::UnknownKey {
                key: "appearance.hair".to_string(),
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
            "[appearance.physique]\nheight = 1.5\n",
            SettingsError::WrongType {
                key: "appearance.physique.height".to_string(),
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
    match settings.apply(&mut player) {
        Err(SettingsError::OutOfRange { key, .. }) => {
            assert_eq!(key, "appearance.strip.sleeves")
        }
        other => panic!("expected OutOfRange, got {other:?}"),
    }
    assert_eq!(player, before, "the refused write changed the player");
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
        (Kind::Signed7, 14, 15),
        (Kind::OneBased { max: 5 }, 4, 5),
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
