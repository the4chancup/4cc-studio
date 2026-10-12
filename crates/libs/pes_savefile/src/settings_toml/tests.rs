//! The `PlayerSettings` tests: the completeness invariant, `from_player`,
//! `apply`, the key table itself, and the TOML text half.

use std::collections::HashSet;

use pes_version::PesVersion;
use toml_edit::DocumentMut;

use super::keys::Source;
use super::{
    AppearanceSettings, KeyTable, Kind, NameSetting, Ownership, PlayerSettings, SettingKey,
    SettingsError, face_ownership, ownership,
};
use crate::codec::{read_player, runs, text_max, write_player};
use crate::model::ingame_face::IngameFace;
use crate::model::player::PlayerAppearance;
use crate::schema::fields::{PlayerField, PlayerText};
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
        .map(|k| (k.spec().table.path(), k.spec().name))
        .collect();
    assert_eq!(
        names.len(),
        SettingKey::ALL.len(),
        "a duplicate (table, name)"
    );
}

/// The completeness invariant, per bit and per version: the appearance
/// block — the whole appearance record on PES 15/16, the player record
/// from the `AppearanceId` row's byte to its end on PES 17+ — is carried
/// exactly once per bit: a `PlayerField` row of the block's record, an
/// `INGAME_FACE_FIELDS` row inside the run, or the run's undecoded
/// remainder (`[appearance.unknown]` `face`). Every bit outside the run
/// is a `PlayerField`'s, so no undecoded bit lies outside the run.
#[test]
fn every_bit_of_the_appearance_block_has_one_carrier() {
    /// What carries one bit of the block.
    #[derive(Debug, Clone, Copy)]
    enum Carrier {
        /// A `PlayerField` row of the record's schema.
        Field(PlayerField),
        /// An `INGAME_FACE_FIELDS` row inside the run.
        Face(IngameFaceField),
        /// The run's undecoded remainder: `[appearance.unknown]` `face`.
        Unknown,
    }
    /// The carrier's name in a failure message.
    fn name(carrier: &Carrier) -> String {
        match carrier {
            Carrier::Field(field) => format!("{field:?}"),
            Carrier::Face(field) => format!("ingame_face::{field:?}"),
            Carrier::Unknown => "unknown".to_string(),
        }
    }
    for version in PesVersion::ALL {
        let schema = schema_for(version);
        let (record, block_start) = match &schema.appearance {
            Some((_, appearance)) => (*appearance, 0),
            None => {
                let (_, offset, _) = runs(schema.player)
                    .find(|(field, _, _)| *field == PlayerField::AppearanceId)
                    .expect("PES 17+ stores the appearance id");
                (schema.player, offset as usize / 8)
            }
        };
        let run = record
            .ingame_face
            .as_ref()
            .expect("the block's record carries the run");
        let run_first_bit = run.byte_offset as usize * 8;
        let mut carriers: Vec<Vec<Carrier>> = vec![Vec::new(); record.size * 8];
        for (field, offset, width) in runs(record) {
            for bit in offset..offset + width {
                carriers[bit as usize].push(Carrier::Field(field));
            }
        }
        let mut face_bits = HashSet::new();
        for spec in INGAME_FACE_FIELDS {
            for bit in spec.bit_offset..spec.bit_offset + spec.bit_width {
                face_bits.insert(bit);
                carriers[run_first_bit + bit as usize].push(Carrier::Face(spec.field));
            }
        }
        for bit in 0..run.len * 8 {
            if !face_bits.contains(&bit) {
                carriers[run_first_bit + bit as usize].push(Carrier::Unknown);
            }
        }
        for (i, bit_carriers) in carriers[block_start * 8..].iter().enumerate() {
            assert_eq!(
                bit_carriers.len(),
                1,
                "{version:?} bit {}: carriers {:?}",
                block_start * 8 + i,
                bit_carriers.iter().map(name).collect::<Vec<_>>()
            );
        }
        for (i, bit_carriers) in carriers[block_start * 8..run_first_bit].iter().enumerate() {
            assert!(
                matches!(bit_carriers.as_slice(), [Carrier::Field(_)]),
                "{version:?} bit {} outside the run: carriers {:?}",
                block_start * 8 + i,
                bit_carriers.iter().map(name).collect::<Vec<_>>()
            );
        }
    }
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

/// The first fixture player whose run holds a nonzero undecoded bit: the
/// `face` tests need real bits to discriminate (an all-zero run would pass
/// vacuously).
fn player_with_undecoded_bits(
    payload: &[u8],
    schema: &crate::schema::VersionSchema,
) -> crate::model::player::PlayerEntry {
    // The record holding the run: the player record on 17+, the id-keyed
    // appearance record on 15/16.
    let (section, layout) = schema
        .appearance
        .as_ref()
        .map_or((&schema.players, schema.player), |(section, appearance)| {
            (section, *appearance)
        });
    for i in 0..count(payload, section) {
        let rec = record(payload, section, layout.size, i);
        let id = u32::from_le_bytes(rec[..4].try_into().expect("record id"));
        if id == 0 {
            continue;
        }
        let run = layout.ingame_face.as_ref().expect("the run lives here");
        let bytes = &rec[run.byte_offset as usize..(run.byte_offset + run.len) as usize];
        let mask = face_field_mask(bytes.len());
        if bytes.iter().zip(&mask).any(|(b, m)| b & !m != 0) {
            return find_player(payload, schema, id);
        }
    }
    panic!("{:?}: no player with an undecoded bit", schema.version);
}

#[test]
fn apply_copies_the_settings_and_leaves_the_rest() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let p = find_player(&payload, schema, 70101);
    let mut q = find_player(&payload, schema, 70102);

    // Flip one undecoded bit of q's run so the check below discriminates:
    // a keep-the-target's-bits apply would show the flip, `face` overwrites
    // it with p's bit.
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
        .apply(&mut q, PesVersion::Pes19)
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
    // The undecoded bits are `face`'s: q's run carries p's now, flipped
    // bit included.
    for (byte, (now, src)) in q
        .appearance
        .ingame_face
        .bytes()
        .iter()
        .zip(p_run)
        .enumerate()
    {
        assert_eq!(
            now & !mask[byte],
            src & !mask[byte],
            "undecoded bits of run byte {byte}"
        );
    }
    assert_eq!(q.motion, p.motion);
    assert_eq!(q.name, p.name);
    assert_eq!(q.shirt_name, p.shirt_name);
    assert_eq!(q.id, q_before.id);
    assert_eq!(q.stats, q_before.stats);
    assert_eq!(q.positions, q_before.positions);
    assert_eq!(q.skills, q_before.skills);
    assert_eq!(q.edit_flags, q_before.edit_flags);
    // q's basics (nationality, age, height, weight) are gameplay, untouched.
    assert_eq!(q.basic, q_before.basic);
}

#[test]
fn height_and_weight_are_not_settings() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let p = find_player(&payload, schema, 70101);
    let mut q = find_player(&payload, schema, 70102);
    assert_ne!(
        (p.basic.height, p.basic.weight),
        (q.basic.height, q.basic.weight),
        "the test needs the two players to differ"
    );
    let (height, weight) = (q.basic.height, q.basic.weight);
    let settings = PlayerSettings::from_player(&p).expect("from_player");
    settings.apply(&mut q, PesVersion::Pes19).expect("apply");
    assert_eq!(
        (q.basic.height, q.basic.weight),
        (height, weight),
        "a settings file does not carry them"
    );
    // Nothing named `height` or `weight` emits, set or commented.
    let text = settings.to_toml().expect("emit");
    assert!(
        !text.lines().any(|line| {
            let line = line.trim_start_matches('#').trim_start();
            line.starts_with("height =") || line.starts_with("weight =")
        }),
        "a height/weight line emitted: {text}"
    );
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
        // `from_player` fills every key, so only the flipped one's bits can
        // differ after the apply (absent keys write their defaults now).
        let mut settings = PlayerSettings::from_player(&player).expect("from_player");
        let current = settings.get(key);
        let new_value = match current {
            Some(0) | None => 1,
            _ => 0,
        };
        settings
            .set(key, new_value)
            .expect("the flipped value is in range");
        let mut modified = player.clone();
        settings
            .apply(&mut modified, PesVersion::Pes21)
            .expect("apply");
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

/// A default apply writes every appearance key's bits (set ones at their
/// value, absent ones at their default) plus `face`'s undecoded bits (to
/// 0), and no other bit of the record (stats, edit flags, the appearance
/// id).
#[test]
fn an_apply_then_write_touches_only_the_appearance_keys_bits() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut player = find_player(&payload, schema, 70101);
    PlayerSettings::default()
        .apply(&mut player, PesVersion::Pes19)
        .expect("apply");

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

    // The bit runs of every settings key the version holds (the stock IDs
    // included: unset they are left alone, but their bits are the file's to
    // write), plus the whole ingame-face run: `face` writes its undecoded
    // bits. A key the version has no field for (`dribbling` here) writes
    // nothing, so it contributes no run.
    let run = schema.player.ingame_face.as_ref().expect("a run on PES 19");
    let mut ranges = Vec::new();
    for key in SettingKey::ALL {
        match key.source() {
            Source::Player(field) => {
                if let Some((_, offset, width)) = runs(schema.player).find(|(f, _, _)| *f == field)
                {
                    ranges.push((offset, offset + width));
                }
            }
            Source::Face(field) => {
                let row = INGAME_FACE_FIELDS
                    .iter()
                    .find(|f| f.field == field)
                    .unwrap_or_else(|| panic!("{key:?} source has no row"));
                ranges.push((
                    run.byte_offset * 8 + row.bit_offset,
                    run.byte_offset * 8 + row.bit_offset + row.bit_width,
                ));
            }
        }
    }
    ranges.push((run.byte_offset * 8, (run.byte_offset + run.len) * 8));
    let mut diffs = 0usize;
    for (byte, (was, now)) in rec.iter().zip(&written).enumerate() {
        for bit in 0..8 {
            if (was >> bit) & 1 != (now >> bit) & 1 {
                let pos = byte as u32 * 8 + bit;
                diffs += 1;
                assert!(
                    ranges.iter().any(|(s, e)| pos >= *s && pos < *e),
                    "bit {pos} changed outside the settings keys' runs"
                );
            }
        }
    }
    assert_ne!(
        diffs, 0,
        "the defaults differ from the fixture's appearance"
    );
}

/// `dribbling` exists only on PES 20 and 21: on an older target the key is
/// skipped, set or not (an export written for a newer version compiles for
/// an older one as it is), and the rest of the settings still applies. On
/// PES 21 the same settings writes it.
#[test]
fn a_setting_the_version_lacks_is_skipped_not_refused() {
    let mut settings = PlayerSettings {
        name: Some(NameSetting::Explicit("RENAMED".to_string())),
        ..PlayerSettings::default()
    };
    settings.set(SettingKey::Sleeves, 2).expect("in range");
    settings.set(SettingKey::Dribbling, 2).expect("in range");

    let payload19 = payload(PesVersion::Pes19);
    let mut player = find_player(&payload19, schema_for(PesVersion::Pes19), 70101);
    settings
        .apply(&mut player, PesVersion::Pes19)
        .expect("a key the version lacks is skipped, not an error");
    assert_eq!(player.motion.dribbling, None, "no field, no write");
    assert_eq!(player.name, "RENAMED");
    assert_eq!(player.appearance.sleeves, 2);

    let payload21 = payload(PesVersion::Pes21);
    let mut player = find_player(&payload21, schema_for(PesVersion::Pes21), 70101);
    settings
        .apply(&mut player, PesVersion::Pes21)
        .expect("apply");
    assert_eq!(player.motion.dribbling, Some(2));
    assert_eq!(player.appearance.sleeves, 2);
}

/// A fully unset settings writes every appearance key's default, so no
/// save value shows through: the stock IDs and the names keep the save's
/// own (the compiler resolves the IDs; absent names are never written),
/// and nothing outside the appearance changes.
#[test]
fn a_default_settings_applies_the_template_values() {
    for version in [PesVersion::Pes19, PesVersion::Pes21] {
        let payload = payload(version);
        let schema = schema_for(version);
        let mut player = find_player(&payload, schema, 70101);
        let before = player.clone();
        // The fixture's appearance must differ from the defaults, or the
        // assertions below prove nothing.
        let current = PlayerSettings::from_player(&before).expect("from_player");
        assert!(
            SettingKey::ALL.iter().any(|&key| {
                let held = match key.source() {
                    Source::Player(field) => schema.player_has(field),
                    Source::Face(_) => true,
                };
                held && current.get(key) != key.spec().default
            }),
            "{version:?} player 70101 already sits at the defaults"
        );

        PlayerSettings::default()
            .apply(&mut player, version)
            .expect("apply");

        let read = PlayerSettings::from_player(&player).expect("from_player");
        for key in SettingKey::ALL {
            let held = match key.source() {
                Source::Player(field) => schema.player_has(field),
                Source::Face(_) => true,
            };
            let expected = if held { key.spec().default } else { None };
            match key {
                SettingKey::BootsId | SettingKey::GlovesId => {}
                _ => assert_eq!(read.get(key), expected, "{version:?} {key:?}"),
            }
        }
        // On PES 21 the default `dribbling = 0` is written too.
        assert_eq!(
            player.motion.dribbling,
            if version == PesVersion::Pes21 {
                Some(0)
            } else {
                None
            }
        );
        // The compiler-resolved IDs, the names and the rest of the record.
        assert_eq!(player.appearance.boots_id, before.appearance.boots_id);
        assert_eq!(player.appearance.gloves_id, before.appearance.gloves_id);
        assert_eq!(
            player.appearance.base_copy_id,
            before.appearance.base_copy_id
        );
        assert_eq!(player.name, before.name);
        assert_eq!(player.shirt_name, before.shirt_name);
        assert_eq!(player.id, before.id);
        assert_eq!(player.appearance_id, before.appearance_id);
        assert_eq!(player.basic, before.basic);
        assert_eq!(player.stats, before.stats);
        assert_eq!(player.positions, before.positions);
        assert_eq!(player.skills, before.skills);
        assert_eq!(player.edit_flags, before.edit_flags);
    }
}

/// The point of "absent = default": the same partial file applied to two
/// different players gives them the same appearance, nothing of what the
/// save held shows through.
#[test]
fn the_same_partial_settings_gives_two_players_the_same_appearance() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let mut a = find_player(&payload, schema, 70101);
    let mut b = find_player(&payload, schema, 70102);
    assert_ne!(
        PlayerSettings::from_player(&a)
            .expect("from_player")
            .appearance,
        PlayerSettings::from_player(&b)
            .expect("from_player")
            .appearance,
        "the test needs two players whose appearance differs"
    );
    let mut settings = PlayerSettings::default();
    settings.set(SettingKey::SkinColor, 3).expect("in range");
    settings.set(SettingKey::FreeKick, 19).expect("in range");
    settings.set(SettingKey::CheekType, 2).expect("in range");
    settings.apply(&mut a, PesVersion::Pes19).expect("apply");
    settings.apply(&mut b, PesVersion::Pes19).expect("apply");
    assert_eq!(
        PlayerSettings::from_player(&a)
            .expect("from_player")
            .appearance,
        PlayerSettings::from_player(&b)
            .expect("from_player")
            .appearance
    );
}

/// `from_player`'s `face`: the run's own bytes with every named bit 0 —
/// a 46-byte run on PES 15, 50 otherwise.
#[test]
fn from_player_carries_the_runs_undecoded_bits_with_the_named_ones_zeroed() {
    for version in [PesVersion::Pes15, PesVersion::Pes19, PesVersion::Pes21] {
        let payload = payload(version);
        let schema = schema_for(version);
        let player = player_with_undecoded_bits(&payload, schema);
        let run = player.appearance.ingame_face.bytes();
        let settings = PlayerSettings::from_player(&player).expect("from_player");
        let face = settings
            .appearance
            .unknown
            .face
            .as_ref()
            .expect("a real player carries face");
        assert_eq!(face.len(), run.len(), "{version:?} run length");
        let mask = face_field_mask(run.len());
        for (byte, (&f, &r)) in face.iter().zip(run).enumerate() {
            assert_eq!(f & mask[byte], 0, "{version:?} byte {byte}: a named bit");
            assert_eq!(
                f & !mask[byte],
                r & !mask[byte],
                "{version:?} byte {byte}: an undecoded bit"
            );
        }
    }
}

/// `face` carries the run's undecoded bits across versions, as far as the
/// target's run reaches: a longer value loses its tail, a shorter one
/// leaves the rest at 0.
#[test]
fn apply_carries_the_undecoded_bits_as_far_as_the_target_run_reaches() {
    // 50 bytes onto 50 (PES 16 onto PES 21): every undecoded bit lands.
    let p16 =
        player_with_undecoded_bits(&payload(PesVersion::Pes16), schema_for(PesVersion::Pes16));
    let s16 = PlayerSettings::from_player(&p16).expect("from_player");
    let mut p21 = find_player(
        &payload(PesVersion::Pes21),
        schema_for(PesVersion::Pes21),
        70101,
    );
    s16.apply(&mut p21, PesVersion::Pes21).expect("apply");
    let face16 = s16.appearance.unknown.face.as_ref().expect("face");
    assert_eq!(p21.appearance.ingame_face.undecoded()[..], face16[..]);

    // 50 onto 46 (PES 21 onto PES 15): the last 4 bytes are dropped, the
    // first 46 land.
    let p21s =
        player_with_undecoded_bits(&payload(PesVersion::Pes21), schema_for(PesVersion::Pes21));
    let mut s21 = PlayerSettings::from_player(&p21s).expect("from_player");
    s21.name = None;
    s21.shirt_name = None;
    let mut p15 = find_player(
        &payload(PesVersion::Pes15),
        schema_for(PesVersion::Pes15),
        70101,
    );
    s21.apply(&mut p15, PesVersion::Pes15).expect("apply");
    let face21 = s21.appearance.unknown.face.as_ref().expect("face");
    assert_eq!(p15.appearance.ingame_face.undecoded()[..], face21[..46]);

    // 46 onto 50 (PES 15 onto PES 21): past the value's end the undecoded
    // bits are 0.
    let p15s =
        player_with_undecoded_bits(&payload(PesVersion::Pes15), schema_for(PesVersion::Pes15));
    let mut s15 = PlayerSettings::from_player(&p15s).expect("from_player");
    s15.name = None;
    s15.shirt_name = None;
    let mut p21b = find_player(
        &payload(PesVersion::Pes21),
        schema_for(PesVersion::Pes21),
        70102,
    );
    s15.apply(&mut p21b, PesVersion::Pes21).expect("apply");
    let face15 = s15.appearance.unknown.face.as_ref().expect("face");
    let undecoded21 = p21b.appearance.ingame_face.undecoded();
    assert_eq!(undecoded21[..46], face15[..]);
    assert!(
        undecoded21[46..].iter().all(|b| *b == 0),
        "bytes 46-49's undecoded bits stay 0"
    );
}

/// An absent `face` writes all 0 to the undecoded bits: a compiled player
/// keeps nothing of what the save held there, and the named bits are the
/// keys' defaults.
#[test]
fn an_absent_face_zeroes_the_undecoded_bits() {
    let mut player =
        player_with_undecoded_bits(&payload(PesVersion::Pes19), schema_for(PesVersion::Pes19));
    PlayerSettings::default()
        .apply(&mut player, PesVersion::Pes19)
        .expect("apply");
    assert!(
        player
            .appearance
            .ingame_face
            .undecoded()
            .iter()
            .all(|b| *b == 0),
        "every undecoded bit is 0"
    );
    for key in SettingKey::ALL {
        if let Source::Face(field) = key.source() {
            assert_eq!(
                player.appearance.ingame_face.get(field).expect("named bit"),
                key.spec().default.expect("a face key carries a default"),
                "{key:?} at its default"
            );
        }
    }
}

/// The named bits of a `face` value are ignored on read: the keys own them.
#[test]
fn named_bits_set_in_face_do_not_reach_the_named_fields() {
    let payload = payload(PesVersion::Pes19);
    let schema = schema_for(PesVersion::Pes19);
    let p = find_player(&payload, schema, 70101);
    let mut q = find_player(&payload, schema, 70102);
    let mut settings = PlayerSettings::from_player(&p).expect("from_player");
    // All bits set, the named positions included: the keys must still win.
    let run_len = q.appearance.ingame_face.bytes().len();
    settings.appearance.unknown.face = Some(vec![0xFF; run_len]);
    settings.name = None;
    settings.shirt_name = None;
    settings.apply(&mut q, PesVersion::Pes19).expect("apply");
    for field in IngameFaceField::ALL {
        assert_eq!(
            q.appearance.ingame_face.get(field).expect("named bit"),
            p.appearance.ingame_face.get(field).expect("named bit"),
            "{field:?} is the key's value, not face's"
        );
    }
    let mask = face_field_mask(run_len);
    for (byte, &b) in q.appearance.ingame_face.bytes().iter().enumerate() {
        assert_eq!(
            b & !mask[byte],
            !mask[byte],
            "byte {byte}: the undecoded bits are all set"
        );
    }
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
        "\"\" = default (fpc_on: hidden, fpc_off 0, no marker 0 or in a Midcup export the installed ID); 0 to 100, a stock boots model; ignored when the folder's boots models or a boots link get an ID (pre-Fox face-XML boots do not)",
        "\"\" = default (fpc_on: hidden, fpc_off 0, no marker 0 or in a Midcup export the installed ID); 0 to 100, a stock gloves model (0 = normal hands); ignored when the folder's gloves models or a gloves link get an ID (pre-Fox face-XML gloves do not)",
        "0 white, 1 light, 2 fair, 3 medium, 4 olive, 5 brown, 6 black, 7 custom (invisible body, PES 15 to 17 only)",
        "0 black, 1 dark brown, 2 brown, 3 sable, 4 navy blue, 5 charcoal, 6 gray, 7 blue, 8 sienna, 9 green, 10 violet",
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
        assert!(!spec.comment.is_empty(), "{key:?} comment");
        assert_eq!(spec.comment, comment, "{key:?} comment");
        assert!(
            spec.table == KeyTable::Top || tables.contains(&spec.table.path()),
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
/// verbatim: `to_toml` of the settings the block parses to must reproduce
/// it byte for byte.
const PLAN_BLOCK: &str = r#"# settings.toml, inside a player folder. Every key is optional: an absent key takes the
# value shown here (name and shirt_name: not written). A commented key shows what can be
# set and its range. The file never references the export's models: what a player wears
# is decided by the folder contents (models present, link files pointing at shared
# folders); boots_id/gloves_id only name the game's stock models.

# true = derive from the folder name ("15 - Snuffy" gives "Snuffy"; the whole folder
# name for players.txt-mapped folders); "text" = write as is; absent = not written.
# name and shirt_name are used only when the export's root holds the `autopilot` marker (a
# team without a manager); otherwise they are ignored and the manager's names stand.
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

# The in-game face's bits no tool has decoded yet (hair, face sliders, ...), kept so a
# player's look survives. Leave them as generated.
[appearance.unknown]
face = "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
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
            reparsed.appearance.unknown.face, settings.appearance.unknown.face,
            "{version:?} face through text"
        );
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

/// `face` is the run's hex: 46 or 50 bytes. A wrong byte count, an odd
/// digit count or a non-hex char is `WrongType`; another key in the table
/// is `UnknownKey`.
#[test]
fn appearance_unknown_parses_only_a_46_or_50_byte_hex_face() {
    for (text, why) in [
        // 49 bytes: one short of the 50-byte form.
        (
            format!("[appearance.unknown]\nface = \"{}\"\n", "0".repeat(98)),
            "49 bytes",
        ),
        // An odd digit count cannot be bytes at all.
        (
            "[appearance.unknown]\nface = \"012\"\n".to_string(),
            "an odd digit count",
        ),
        // 50 bytes with a non-hex char.
        (
            format!("[appearance.unknown]\nface = \"z{}\"\n", "0".repeat(99)),
            "a non-hex char",
        ),
    ] {
        match PlayerSettings::parse(&text) {
            Err(SettingsError::WrongType { key, expected }) => assert_eq!(
                (key.as_str(), expected),
                ("appearance.unknown.face", "a hex string of 46 or 50 bytes"),
                "{why}"
            ),
            other => panic!("{why}: expected WrongType, got {other:?}"),
        }
    }
    match PlayerSettings::parse(&format!(
        "[appearance.unknown]\nface = \"{}\"\nother = 1\n",
        "0".repeat(100)
    )) {
        Err(SettingsError::UnknownKey { key }) => {
            assert_eq!(key, "appearance.unknown.other")
        }
        other => panic!("expected UnknownKey, got {other:?}"),
    }
}

/// `update_toml` sets `face`, creating `[appearance.unknown]` when the
/// document lacks it, as it does for the other tables.
#[test]
fn update_toml_writes_face_creating_the_table() {
    let mut document: DocumentMut = "[appearance]\nskin_color = 1\n"
        .parse()
        .expect("document parses");
    let mut settings = PlayerSettings::default();
    settings.appearance.unknown.face = Some(vec![0xAB; 50]);
    settings.update_toml(&mut document).expect("update");
    let text = document.to_string();
    assert!(
        text.contains(&format!(
            "[appearance.unknown]\nface = \"{}\"",
            "ab".repeat(50)
        )),
        "{text}"
    );
    assert_eq!(
        PlayerSettings::parse(&text)
            .expect("the updated document parses")
            .appearance
            .unknown
            .face,
        Some(vec![0xAB; 50])
    );
}

#[test]
fn parse_reports_the_first_error_with_its_key() {
    let cases: [(&str, SettingsError); 13] = [
        (
            "appearance.hair = 1",
            SettingsError::UnknownKey {
                key: "appearance.hair".to_string(),
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
