use super::*;

/// A save player's settings give back exactly his row: the id, then the
/// record's 56 bytes at the row offset (128 on PES 18/19, 252 on PES 20/21,
/// written literally so the check does not lean on `row_offset`). The
/// completeness test gives every bit of that span one carrier in
/// `PlayerSettings`, so nothing can be lost. `from_player` is the
/// settings' construction; where it refuses (a stored value the key table
/// cannot express) the same settings come through `appearance_from`, the
/// stored-ranges reader that carries such values verbatim. No fixture
/// player needs it since PES 15's caps joined the ranges, so one built
/// player does: the first one with `goal_celebration_2` stored 200, which
/// its field's 8 bits hold and no key's range does (a face type's range is
/// its field's bits now, so a motion carries the bump). The row's run
/// differs from the record's only in its unused bits, which take the games'
/// values.
#[test]
fn a_save_players_settings_reproduce_his_appearance_row() {
    for (version, offset) in [
        (PesVersion::Pes18, 128usize),
        (PesVersion::Pes19, 128),
        (PesVersion::Pes20, 252),
        (PesVersion::Pes21, 252),
    ] {
        let payload = payload(version);
        let schema = schema_for(version);
        // The run inside the row: the record's run offset minus the row
        // offset, from the schema rather than a fixed byte 10; asserted below.
        let run = schema.player.ingame_face.as_ref().expect("Fox has a run");
        let run_in_row = run.byte_offset as usize - offset;
        assert_eq!(run_in_row, 10, "{version:?}: the run's row byte");
        let mut records: Vec<Vec<u8>> = (0..count(&payload, &schema.players))
            .map(|i| record(&payload, &schema.players, schema.player.size, i).to_vec())
            .filter(|rec| read_player(rec, schema.player).expect("decode").id != 0)
            .collect();
        let mut built = records[0].clone();
        let mut player = read_player(&built, schema.player).expect("decode");
        player.motion.goal_celebration_2 = 200;
        write_player(&player, &mut built, schema.player).expect("encode");
        records.push(built);
        let mut hair_seen = false;
        let mut out_of_range_players = 0usize;
        for rec in &records {
            let player = read_player(rec, schema.player).expect("decode");
            let settings = match PlayerSettings::from_player(&player) {
                Ok(settings) => settings,
                Err(SettingsError::OutOfRange { .. }) => {
                    // A stored value the key table cannot express: the
                    // stored-ranges reader carries it verbatim.
                    out_of_range_players += 1;
                    PlayerSettings {
                        appearance: appearance_from(&player).expect("stored-ranges read"),
                        ..PlayerSettings::default()
                    }
                }
                Err(err) => panic!("{version:?} player {}: {err:?}", player.id),
            };
            let row = settings.appearance_row(player.id, version).expect("a row");
            let mut expected = [0u8; 60];
            expected[..4].copy_from_slice(&player.id.to_le_bytes());
            expected[4..].copy_from_slice(&rec[offset..offset + 56]);
            let mut written = player.appearance.ingame_face.clone();
            written.write_unused();
            // The row ends inside the run: its last 46 bytes are the run's
            // first 46.
            let run_at = 4 + run_in_row;
            let in_row = expected.len() - run_at;
            expected[run_at..].copy_from_slice(&written.bytes()[..in_row]);
            if let Some(at) = row.iter().zip(&expected).position(|(a, b)| a != b) {
                panic!(
                    "{version:?} player {}: row byte {at} is {:#04x}, the record's is {:#04x}",
                    player.id, row[at], expected[at]
                );
            }
            hair_seen |= SettingKey::ALL.iter().any(|key| {
                key.spec().table == KeyTable::Hair
                    && held_on(*key, version)
                    && settings.get(*key) != key.spec().default
            });
        }
        assert!(
            hair_seen,
            "{version:?}: every player's hair is the default, the rows prove little"
        );
        // The built player's 200 is the reason `appearance_row` skips
        // `apply`'s checks: without it this test would stop proving it.
        assert!(
            out_of_range_players > 0,
            "{version:?}: no player's settings took the OutOfRange fallback"
        );
    }
}

/// TC-BIN-28's "the template's defaults": a default settings' row, written
/// into a zeroed PES 21 record at the row offset and decoded, reads back
/// every appearance key the version has at its `KeySpec` default and the
/// run's unused bits at the games' values. 71405 is /co/'s slot 05.
#[test]
fn a_default_row_carries_the_template_defaults() {
    let version = PesVersion::Pes21;
    let schema = schema_for(version);
    let row = PlayerSettings::default()
        .appearance_row(71405, version)
        .expect("a row");
    assert_eq!(row[..4], 71405u32.to_le_bytes(), "the row names the player");
    let mut record = vec![0u8; schema.player.size];
    record[252..252 + 56].copy_from_slice(&row[4..]);
    let player = read_player(&record, schema.player).expect("decode");
    let read = PlayerSettings::from_player(&player).expect("from_player");
    for key in SettingKey::ALL {
        let expected = if held_on(key, version) {
            key.spec().default
        } else {
            None
        };
        match key {
            SettingKey::BootsId | SettingKey::GlovesId => {}
            _ => assert_eq!(read.get(key), expected, "{key:?}"),
        }
    }
    let mut written = player.appearance.ingame_face.clone();
    written.write_unused();
    assert_eq!(
        player.appearance.ingame_face, written,
        "the run's unused bits are the games' values"
    );
}

/// The row carries neither name: an overlong one is not this function's
/// error, and the names' bytes never reach the row.
#[test]
fn the_names_are_not_the_rows() {
    let named = PlayerSettings {
        name: Some(NameSetting::Explicit("N".repeat(100))),
        shirt_name: Some("S".repeat(100)),
        ..PlayerSettings::default()
    };
    for version in [
        PesVersion::Pes18,
        PesVersion::Pes19,
        PesVersion::Pes20,
        PesVersion::Pes21,
    ] {
        let plain = PlayerSettings::default()
            .appearance_row(70101, version)
            .expect("a row");
        let row = named.appearance_row(70101, version).expect("a row");
        assert_eq!(row, plain, "{version:?}: the names reached the row");
    }
}

/// Every key at its `version_kind` extreme fits the version's fields: the
/// narrowing is what the writer relies on when it `expect`s the row to
/// build. Both ends of each held range are tried, on every Fox version.
#[test]
fn a_row_builds_from_every_key_at_its_version_range_extremes() {
    let stored_max = |kind: Kind| match kind {
        Kind::Number { max, .. } => max,
        Kind::Signed { limit } => 2 * limit,
        Kind::OneBased { max } => max - 1,
        Kind::PlusOne { max } => max + 1,
        Kind::Mapped(map) => *map.iter().max().expect("a nonempty map"),
        Kind::Bool => 1,
        Kind::Labels(labels) => labels.len() as u8 - 1,
    };
    let stored_min = |kind: Kind| match kind {
        Kind::Number { min, .. } => min,
        Kind::Signed { .. } | Kind::OneBased { .. } | Kind::PlusOne { .. } => 0,
        Kind::Mapped(map) => *map.iter().min().expect("a nonempty map"),
        Kind::Bool | Kind::Labels(_) => 0,
    };
    for version in [
        PesVersion::Pes18,
        PesVersion::Pes19,
        PesVersion::Pes20,
        PesVersion::Pes21,
    ] {
        for stored in [stored_min, stored_max] {
            let mut settings = PlayerSettings::default();
            for key in SettingKey::ALL {
                let Some(kind) = key.version_kind(version) else {
                    continue;
                };
                settings
                    .set(key, stored(kind))
                    .expect("inside the widest range");
            }
            settings
                .appearance_row(71405, version)
                .unwrap_or_else(|err| panic!("{version:?}: {err}"));
        }
    }
}

/// A pre-Fox game has no row: PES 17 reads one, but its compile writes the
/// savefile instead.
#[test]
fn a_pre_fox_version_has_no_row() {
    for version in [PesVersion::Pes15, PesVersion::Pes16, PesVersion::Pes17] {
        let err = PlayerSettings::default()
            .appearance_row(70101, version)
            .expect_err("no row before Fox");
        assert!(
            matches!(err, SettingsError::NoAppearanceRow { version: v } if v == version),
            "{version:?}: {err}"
        );
    }
}
