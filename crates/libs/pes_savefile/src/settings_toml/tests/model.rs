use super::*;

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
    for version in [
        PesVersion::Pes15,
        PesVersion::Pes16,
        PesVersion::Pes19,
        PesVersion::Pes21,
    ] {
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

        // Write through the codec and reread: on PES 15/16 the appearance
        // lives in its own record, so both records round-trip.
        let mut written = payload.clone();
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
        let start = schema.players.offset + index * schema.player.size;
        write_player(
            &player,
            &mut written[start..start + schema.player.size],
            schema.player,
        )
        .expect("encode");
        if let Some((section, appearance)) = &schema.appearance {
            let index = (0..count(&payload, section))
                .find(|&i| {
                    u32::from_le_bytes(
                        record(&payload, section, appearance.size, i)[..4]
                            .try_into()
                            .expect("id"),
                    ) == player.id
                })
                .expect("the player's appearance record");
            let start = section.offset + index * appearance.size;
            write_player(
                &player,
                &mut written[start..start + appearance.size],
                appearance,
            )
            .expect("encode");
        }
        let read = PlayerSettings::from_player(&find_player(&written, schema, 70101))
            .expect("from_player");
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
