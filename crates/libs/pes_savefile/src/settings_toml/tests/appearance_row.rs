use super::*;

/// The step's done-when: a save player's settings give back exactly his row
/// — the id, then the record's 56 bytes at the row offset (128 on PES 18/19,
/// 252 on PES 20/21, written literally so the check does not lean on
/// `row_offset`). 5.1a's completeness test gave every bit of that span one
/// carrier in `PlayerSettings`, so nothing can be lost. `from_player` is the
/// settings' construction; where it refuses — a stored value the key table
/// cannot express (a 3-bit face type holding 6, the cap being 5, as real cup
/// saves carry) — the same settings come through `appearance_from`, the
/// stored-ranges reader that carries such values verbatim.
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
        // offset, from the schema (not the brief's row byte 10, asserted).
        let run = schema.player.ingame_face.as_ref().expect("Fox has a run");
        let run_in_row = run.byte_offset as usize - offset;
        assert_eq!(run_in_row, 10, "{version:?}: the run's row byte");
        let mask = face_field_mask(run.len as usize);
        let mut undecoded_bit_seen = false;
        for i in 0..count(&payload, &schema.players) {
            let rec = record(&payload, &schema.players, schema.player.size, i);
            let player = read_player(rec, schema.player).expect("decode");
            if player.id == 0 {
                continue;
            }
            let settings = match PlayerSettings::from_player(&player) {
                Ok(settings) => settings,
                Err(SettingsError::OutOfRange { .. }) => {
                    // A stored value the key table cannot express: the
                    // stored-ranges reader carries it verbatim. `face` is
                    // set as `from_player` sets it.
                    let mut settings = PlayerSettings {
                        appearance: appearance_from(&player).expect("stored-ranges read"),
                        ..PlayerSettings::default()
                    };
                    settings.appearance.unknown.face =
                        Some(player.appearance.ingame_face.undecoded());
                    settings
                }
                Err(err) => panic!("{version:?} player {}: {err:?}", player.id),
            };
            let row = settings.appearance_row(player.id, version).expect("a row");
            let mut expected = [0u8; 60];
            expected[..4].copy_from_slice(&player.id.to_le_bytes());
            expected[4..].copy_from_slice(&rec[offset..offset + 56]);
            if let Some(at) = row.iter().zip(&expected).position(|(a, b)| a != b) {
                panic!(
                    "{version:?} player {}: row byte {at} is {:#04x}, the record's is {:#04x}",
                    player.id, row[at], expected[at]
                );
            }
            undecoded_bit_seen |= row[4 + run_in_row..]
                .iter()
                .zip(&mask)
                .any(|(&b, &m)| b & !m != 0);
        }
        assert!(
            undecoded_bit_seen,
            "{version:?}: no player's row held an undecoded ingame-face bit"
        );
    }
}

/// TC-BIN-28's "the template's defaults": a default settings' row, written
/// into a zeroed PES 21 record at the row offset and decoded, reads back
/// every appearance key the version has at its `KeySpec` default and `face`
/// all 0. 71405 is /co/'s slot 05.
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
    let read = PlayerSettings::from_player(&read_player(&record, schema.player).expect("decode"))
        .expect("from_player");
    for key in SettingKey::ALL {
        let held = match key.source() {
            Source::Player(field) => schema.player_has(field),
            Source::Face(_) => true,
        };
        let expected = if held { key.spec().default } else { None };
        match key {
            SettingKey::BootsId | SettingKey::GlovesId => {}
            _ => assert_eq!(read.get(key), expected, "{key:?}"),
        }
    }
    assert_eq!(
        read.appearance.unknown.face.as_deref(),
        Some(&[0u8; 50][..]),
        "the run's undecoded bits are all 0"
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
