//! The `PlayerSettings` tests: the completeness invariant, `from_player`,
//! `apply`, the key table itself.

use std::collections::HashSet;
use std::io::Read;

use pes_version::PesVersion;

use super::keys::Source;
use super::{
    AppearanceSettings, Kind, NameSetting, Ownership, PlayerSettings, SettingKey, SettingsError,
    face_ownership, ownership,
};
use crate::codec::{read_player, read_player_into, runs, write_player};
use crate::model::player::{PlayerAppearance, PlayerEntry};
use crate::schema::fields::PlayerField;
use crate::schema::ingame_face::{INGAME_FACE_FIELDS, IngameFaceField};
use crate::schema::{SectionLayout, VersionSchema, schema_for};

/// The inflated payload of one version's fixture.
fn payload(version: PesVersion) -> Vec<u8> {
    let bytes: &[u8] = match version {
        PesVersion::Pes15 => include_bytes!("../../tests/fixtures/pes15_payload.bin.zz"),
        PesVersion::Pes16 => include_bytes!("../../tests/fixtures/pes16_payload.bin.zz"),
        PesVersion::Pes17 => include_bytes!("../../tests/fixtures/pes17_payload.bin.zz"),
        PesVersion::Pes18 => include_bytes!("../../tests/fixtures/pes18_payload.bin.zz"),
        PesVersion::Pes19 => include_bytes!("../../tests/fixtures/pes19_payload.bin.zz"),
        PesVersion::Pes20 => panic!("the PES 20 save shares PES 21's tables; no 20 fixture"),
        PesVersion::Pes21 => include_bytes!("../../tests/fixtures/pes21_payload.bin.zz"),
    };
    let mut out = Vec::new();
    flate2::read::ZlibDecoder::new(bytes)
        .read_to_end(&mut out)
        .expect("fixture payload inflates");
    out
}

fn count(payload: &[u8], section: &SectionLayout) -> usize {
    u16::from_le_bytes(
        payload[section.count_offset..section.count_offset + 2]
            .try_into()
            .expect("u16"),
    ) as usize
}

fn record<'a>(payload: &'a [u8], section: &SectionLayout, size: usize, i: usize) -> &'a [u8] {
    &payload[section.offset + i * size..section.offset + (i + 1) * size]
}

/// The PES 15/16 appearance record of a decoded player id.
fn appearance_for(payload: &[u8], schema: &VersionSchema, player: &mut PlayerEntry) {
    if let Some((section, appearance)) = &schema.appearance {
        for i in 0..count(payload, section) {
            let rec = record(payload, section, appearance.size, i);
            if u32::from_le_bytes(rec[..4].try_into().expect("id")) == player.id {
                read_player_into(player, rec, appearance).expect("appearance decodes");
                return;
            }
        }
        panic!("no appearance record for player {}", player.id);
    }
}

/// The decoded player with `id` (appearance fields included on 15/16).
fn find_player(payload: &[u8], schema: &VersionSchema, id: u32) -> PlayerEntry {
    for i in 0..count(payload, &schema.players) {
        let rec = record(payload, &schema.players, schema.player.size, i);
        let mut player = read_player(rec, schema.player).expect("decode");
        if player.id == id {
            appearance_for(payload, schema, &mut player);
            return player;
        }
    }
    panic!("player {id} not in the save");
}

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

#[test]
fn apply_copies_the_settings_and_leaves_the_rest() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let p = find_player(&payload, schema, 70101);
    let mut q = find_player(&payload, schema, 70102);
    let q_before = q.clone();
    PlayerSettings::from_player(&p)
        .expect("from_player")
        .apply(&mut q)
        .expect("apply");
    assert_eq!(
        q.appearance,
        PlayerAppearance {
            boots_id: q_before.appearance.boots_id,
            gloves_id: q_before.appearance.gloves_id,
            base_copy_id: q_before.appearance.base_copy_id,
            ..p.appearance.clone()
        }
    );
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
fn an_apply_then_write_touches_only_the_set_fields_bits() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut player = find_player(&payload, schema, 70101);
    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::SkinColor, 3);
    settings.set(SettingKey::NeckLength, 9);
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
    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::Dribbling, 1);
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
        settings.set(key, 1);
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
