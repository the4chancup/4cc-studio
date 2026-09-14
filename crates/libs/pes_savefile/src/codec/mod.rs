//! The generic record codec: the one engine that walks a `RecordSchema`'s
//! field table and moves values between record bytes and the model. It knows
//! nothing about versions or encryption; every offset lives in `schema/`.

/// LSB-first bit-run reads and writes.
mod bits;

use crate::model::player::PlayerEntry;
use crate::schema::RecordSchema;
use crate::schema::fields::{PlayerField, PlayerText};

/// A player/team record could not be decoded or re-encoded.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// A model value does not fit the schema's bit run.
    #[error("{field}: value {value} does not fit a {width}-bit run")]
    ValueTooWide {
        /// The field whose value is too wide.
        field: String,
        /// The offending value.
        value: u32,
        /// The run's width in bits.
        width: u32,
    },
    /// The version's schema stores a field the player has no value for.
    #[error("{field}: the version stores this field but the player has no value for it")]
    Missing {
        /// The gated field that is `None`.
        field: String,
    },
    /// A text field could not be decoded or encoded.
    #[error("text field {text}: {reason}")]
    Text {
        /// The field (`"Name"`, `"ShirtName"`).
        text: String,
        /// Why the text failed.
        reason: &'static str,
    },
    /// A record slice is not the schema's size.
    #[error("record is {got} bytes, the schema expects {expected}")]
    RecordSize {
        /// The schema's record size.
        expected: usize,
        /// The slice's actual length.
        got: usize,
    },
    /// An indexed field named an element past its array.
    #[error("{field}: index {index} is out of range")]
    Index {
        /// The field whose index is out of range.
        field: String,
        /// The offending element index.
        index: u8,
    },
}

/// Every `FieldSpec` and every expanded `ArraySpec` element of a record schema
/// as `(field, bit_offset, bit_width)`; the three record kinds reuse it.
pub(crate) fn runs<F: Copy, T>(
    schema: &RecordSchema<F, T>,
) -> impl Iterator<Item = (F, u32, u32)> + '_ {
    schema
        .fields
        .iter()
        .map(|spec| (spec.field, spec.bit_offset, spec.bit_width))
        .chain(schema.arrays.iter().flat_map(|array| {
            (0..array.count).map(move |i| {
                (
                    (array.make)(i),
                    array.base_bit + u32::from(i) * array.stride_bits,
                    array.bit_width,
                )
            })
        }))
}

/// The bytes a text field currently stores: everything up to the first NUL or
/// the field end.
fn text_bytes(record: &[u8], byte_offset: u32, len: u32) -> &[u8] {
    let field = &record[byte_offset as usize..(byte_offset + len) as usize];
    let end = field.iter().position(|&b| b == 0).unwrap_or(field.len());
    &field[..end]
}

/// Writes `bytes` at the field's start, one NUL after them, and leaves the
/// remaining bytes of the field untouched (real saves carry old bytes there).
/// The caller has already checked `bytes` fits `len - 1`.
fn write_text(record: &mut [u8], byte_offset: u32, len: u32, bytes: &[u8]) {
    let field = &mut record[byte_offset as usize..(byte_offset + len) as usize];
    field[..bytes.len()].copy_from_slice(bytes);
    field[bytes.len()] = 0;
}

/// The single-byte encoding of a shirt name: each char is one byte U+00XX.
fn shirt_name_bytes(name: &str) -> Result<Vec<u8>, CodecError> {
    let mut encoded = Vec::with_capacity(name.len());
    for c in name.chars() {
        if u32::from(c) > 0xFF {
            return Err(CodecError::Text {
                text: "ShirtName".to_string(),
                reason: "a shirt name char is above U+00FF",
            });
        }
        encoded.push(c as u8);
    }
    Ok(encoded)
}

/// Decodes one player record into a fresh `PlayerEntry`.
pub fn read_player(
    record: &[u8],
    schema: &RecordSchema<PlayerField, PlayerText>,
) -> Result<PlayerEntry, CodecError> {
    let mut player = PlayerEntry::default();
    read_player_into(&mut player, record, schema)?;
    Ok(player)
}

/// Applies a record's fields onto an existing `PlayerEntry`: what the PES 15/16
/// appearance record is applied with. Only the fields and texts the schema
/// lists are touched.
pub fn read_player_into(
    player: &mut PlayerEntry,
    record: &[u8],
    schema: &RecordSchema<PlayerField, PlayerText>,
) -> Result<(), CodecError> {
    if record.len() != schema.size {
        return Err(CodecError::RecordSize {
            expected: schema.size,
            got: record.len(),
        });
    }
    for (field, offset, width) in runs(schema) {
        player.set(field, bits::read_bits(record, offset, width))?;
    }
    for spec in schema.texts {
        let bytes = text_bytes(record, spec.byte_offset, spec.len);
        match spec.text {
            PlayerText::Name => {
                player.name = String::from_utf8(bytes.to_vec()).map_err(|_| CodecError::Text {
                    text: "Name".to_string(),
                    reason: "player name is not UTF-8",
                })?;
            }
            PlayerText::ShirtName => {
                // Single-byte text: byte b is the char U+00bB.
                player.shirt_name = bytes.iter().map(|&b| char::from(b)).collect();
            }
        }
    }
    Ok(())
}

/// Patches the schema's fields and texts of `player` into `record`; every byte
/// outside those runs is left as it was.
pub fn write_player(
    player: &PlayerEntry,
    record: &mut [u8],
    schema: &RecordSchema<PlayerField, PlayerText>,
) -> Result<(), CodecError> {
    if record.len() != schema.size {
        return Err(CodecError::RecordSize {
            expected: schema.size,
            got: record.len(),
        });
    }
    for (field, offset, width) in runs(schema) {
        let value = player.get(field)?;
        if width < 32 && value >= (1u32 << width) {
            return Err(CodecError::ValueTooWide {
                field: format!("{field:?}"),
                value,
                width,
            });
        }
        bits::write_bits(record, offset, width, value);
    }
    for spec in schema.texts {
        let bytes = match spec.text {
            PlayerText::Name => player.name.as_bytes().to_vec(),
            PlayerText::ShirtName => shirt_name_bytes(&player.shirt_name)?,
        };
        if bytes.len() >= spec.len as usize {
            return Err(CodecError::Text {
                text: format!("{:?}", spec.text),
                reason: "the text does not fit its field",
            });
        }
        write_text(record, spec.byte_offset, spec.len, &bytes);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use pes_version::PesVersion;

    use super::*;
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

    /// The versions a committed payload fixture exists for.
    const FIXTURES: [PesVersion; 6] = [
        PesVersion::Pes15,
        PesVersion::Pes16,
        PesVersion::Pes17,
        PesVersion::Pes18,
        PesVersion::Pes19,
        PesVersion::Pes21,
    ];

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

    /// Every player record of `schema`'s fixture as decoded entries.
    fn players(payload: &[u8], schema: &VersionSchema) -> Vec<PlayerEntry> {
        (0..count(payload, &schema.players))
            .map(|i| {
                read_player(
                    record(payload, &schema.players, schema.player.size, i),
                    schema.player,
                )
                .expect("player record decodes")
            })
            .collect()
    }

    #[test]
    fn every_player_record_round_trips_byte_for_byte() {
        for version in FIXTURES {
            let payload = payload(version);
            let schema = schema_for(version);
            for i in 0..count(&payload, &schema.players) {
                let record = record(&payload, &schema.players, schema.player.size, i);
                let player = read_player(record, schema.player).expect("decode");
                let mut written = record.to_vec();
                write_player(&player, &mut written, schema.player).expect("encode");
                assert_eq!(written, record, "{version:?} player record {i}");
            }
            if let Some((section, appearance)) = &schema.appearance {
                // Every appearance record round-trips through a player patched
                // by it, and every player id keys exactly one record.
                let records: Vec<&[u8]> = (0..count(&payload, section))
                    .map(|i| record(&payload, section, appearance.size, i))
                    .collect();
                let mut seen = std::collections::HashMap::new();
                for rec in &records {
                    let id = u32::from_le_bytes(rec[..4].try_into().expect("id"));
                    *seen.entry(id).or_insert(0usize) += 1;
                }
                for i in 0..count(&payload, &schema.players) {
                    let player_rec = record(&payload, &schema.players, schema.player.size, i);
                    let player = read_player(player_rec, schema.player).expect("decode");
                    assert_eq!(
                        seen.get(&player.id),
                        Some(&1),
                        "{version:?} player {} keys one appearance record",
                        player.id
                    );
                }
                for rec in records {
                    let mut player = PlayerEntry::default();
                    read_player_into(&mut player, rec, appearance).expect("decode appearance");
                    let mut written = rec.to_vec();
                    write_player(&player, &mut written, appearance).expect("encode appearance");
                    assert_eq!(written, rec, "{version:?} appearance record");
                }
            }
        }
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
    fn player_70101_per_version() {
        // name, shirt_name, nationality, height, weight, attacking_prowess, age,
        // registered, speed.
        let expected = [
            (
                PesVersion::Pes15,
                "QUADS ONLY",
                "TRIS BTFO",
                231,
                189,
                80,
                74,
                20,
                0,
                74,
            ),
            (
                PesVersion::Pes16,
                "PLACEHOLDER",
                "PLACEHOLDER",
                231,
                180,
                80,
                77,
                20,
                1,
                77,
            ),
            (
                PesVersion::Pes17,
                "PLACEHOLDER",
                "PLACEHOLDER",
                231,
                180,
                80,
                77,
                20,
                1,
                77,
            ),
            (
                PesVersion::Pes18,
                "QUADS ONLY",
                "TRIS GTFO",
                238,
                194,
                70,
                77,
                17,
                1,
                77,
            ),
            (
                PesVersion::Pes19,
                "QUADS ONLY",
                "TRIS BTFO",
                231,
                189,
                80,
                77,
                20,
                0,
                77,
            ),
            (
                PesVersion::Pes21,
                "PLACEHOLDER",
                "PLACEHOLDER",
                231,
                180,
                80,
                77,
                20,
                1,
                77,
            ),
        ];
        for (version, name, shirt, nat, height, weight, att, age, reg, speed) in expected {
            let payload = payload(version);
            let player = find_player(&payload, schema_for(version), 70101);
            assert_eq!(
                (
                    player.name.as_str(),
                    player.shirt_name.as_str(),
                    player.basic.nationality,
                    player.basic.height,
                    player.basic.weight,
                    player.stats.attacking_prowess,
                    player.basic.age,
                    player.positions.registered,
                    player.stats.speed,
                ),
                (name, shirt, nat, height, weight, att, age, reg, speed),
                "{version:?} player 70101"
            );
        }
        // Appearance literals on the two versions that keep a separate record.
        let p15 = find_player(
            &payload(PesVersion::Pes15),
            schema_for(PesVersion::Pes15),
            70101,
        );
        assert_eq!(
            (
                p15.appearance.boots_id,
                p15.appearance.gloves_id,
                p15.appearance.skin_color,
                p15.appearance.iris_color,
                p15.appearance.neck_length,
            ),
            (55, 11, 1, 1, 7),
            "PES 15 player 70101 appearance"
        );
        let p16 = find_player(
            &payload(PesVersion::Pes16),
            schema_for(PesVersion::Pes16),
            70101,
        );
        assert_eq!(
            (
                p16.appearance.boots_id,
                p16.appearance.gloves_id,
                p16.appearance.skin_color,
                p16.appearance.iris_color,
                p16.appearance.neck_length,
            ),
            (0, 0, 1, 1, 7),
            "PES 16 player 70101 appearance"
        );
    }

    /// The `PlayerField` variants that hold a 7-bit ability value in the model.
    const ABILITIES: [PlayerField; 24] = [
        PlayerField::AttackingProwess,
        PlayerField::BallControl,
        PlayerField::Dribbling,
        PlayerField::LowPass,
        PlayerField::LoftedPass,
        PlayerField::Finishing,
        PlayerField::PlaceKicking,
        PlayerField::Swerve,
        PlayerField::Heading,
        PlayerField::DefensiveProwess,
        PlayerField::BallWinning,
        PlayerField::KickingPower,
        PlayerField::Speed,
        PlayerField::ExplosivePower,
        PlayerField::BodyControl,
        PlayerField::Jump,
        PlayerField::Goalkeeping,
        PlayerField::Stamina,
        PlayerField::Catching,
        PlayerField::Clearing,
        PlayerField::Reflexes,
        PlayerField::Coverage,
        PlayerField::PhysicalContact,
        PlayerField::TightPossession,
    ];

    #[test]
    fn the_player_census_matches_the_measured_ranges() {
        let counts = [5060usize, 5060, 5060, 4646, 4830, 5060];
        for (i, version) in FIXTURES.iter().enumerate() {
            let payload = payload(*version);
            let schema = schema_for(*version);
            assert_eq!(
                count(&payload, &schema.players),
                counts[i],
                "{version:?} player count"
            );
            // PES 18's magical-girl joke team stores ages 13/14: the one
            // measured exception set the 15..=50 census has.
            let age_exceptions: &[(u32, u8)] = if *version == PesVersion::Pes18 {
                &[
                    (81401, 14),
                    (81403, 14),
                    (81408, 14),
                    (81411, 14),
                    (81413, 13),
                    (81416, 14),
                    (81417, 14),
                    (81421, 13),
                    (81423, 14),
                ]
            } else {
                &[]
            };
            let mut seen_exceptions = 0usize;
            for player in players(&payload, schema) {
                match age_exceptions.iter().find(|(id, _)| *id == player.id) {
                    Some((_, age)) => {
                        seen_exceptions += 1;
                        assert_eq!(player.basic.age, *age, "{version:?} age exception");
                    }
                    None => assert!(
                        (15..=50).contains(&player.basic.age),
                        "{version:?} player {} age {}",
                        player.id,
                        player.basic.age
                    ),
                }
                assert!(
                    player.positions.registered <= 12,
                    "{version:?} player {} registered {}",
                    player.id,
                    player.positions.registered
                );
                for spec in schema.player.fields {
                    if spec.bit_width == 7 && ABILITIES.contains(&spec.field) {
                        let value = player.get(spec.field).expect("a stats field");
                        if !(*version == PesVersion::Pes16
                            && player.id == 71211
                            && spec.field == PlayerField::PlaceKicking)
                        {
                            assert!(
                                (40..=99).contains(&value),
                                "{version:?} player {} {:?} {value}",
                                player.id,
                                spec.field
                            );
                        } else {
                            assert_eq!(value, 100, "the one measured exception");
                        }
                    }
                }
            }
            assert_eq!(
                seen_exceptions,
                age_exceptions.len(),
                "{version:?} every age exception seen exactly once"
            );
        }
        // The two measured non-ASCII shirt names.
        let p16 = find_player(
            &payload(PesVersion::Pes16),
            schema_for(PesVersion::Pes16),
            73903,
        );
        assert_eq!(p16.shirt_name, "N\u{fc}RBURG");
        let p21 = find_player(
            &payload(PesVersion::Pes21),
            schema_for(PesVersion::Pes21),
            83323,
        );
        assert_eq!(p21.shirt_name, "FUR\u{f0}USAGA");
    }

    #[test]
    fn version_gated_fields_are_none_where_the_version_lacks_them() {
        let p15 = players(&payload(PesVersion::Pes15), schema_for(PesVersion::Pes15));
        for p in &p15 {
            assert!(p.stats.clearing.is_none() && p.stats.reflexes.is_none());
            assert!(p.stats.coverage.is_none() && p.stats.physical_contact.is_none());
            assert!(p.stats.star.is_none() && p.stats.catching.is_some());
        }
        let p17 = players(&payload(PesVersion::Pes17), schema_for(PesVersion::Pes17));
        for p in &p17 {
            assert!(p.stats.physical_contact.is_some() && p.stats.star.is_none());
        }
        let p19 = players(&payload(PesVersion::Pes19), schema_for(PesVersion::Pes19));
        for p in &p19 {
            assert!(p.stats.star.is_some() && p.stats.tight_possession.is_none());
        }
        let p21 = players(&payload(PesVersion::Pes21), schema_for(PesVersion::Pes21));
        for p in &p21 {
            assert!(
                p.stats.clearing.is_some()
                    && p.stats.reflexes.is_some()
                    && p.stats.coverage.is_some()
                    && p.stats.physical_contact.is_some()
                    && p.stats.star.is_some()
                    && p.stats.tight_possession.is_some()
                    && p.stats.aggression.is_some()
                    && p.stats.playing_attitude.is_some()
                    && p.positions.stronger_hand.is_some()
                    && p.motion.dribbling.is_some()
                    && p.stats.catching.is_some()
            );
        }
    }

    #[test]
    fn text_writes_patch_the_field_and_keep_the_old_tail() {
        let payload = payload(PesVersion::Pes17);
        let schema = schema_for(PesVersion::Pes17);
        let record = record(&payload, &schema.players, schema.player.size, 0);
        let mut player = read_player(record, schema.player).expect("decode");

        // A shorter name leaves the old tail bytes after the new NUL.
        player.name = "AB".to_string();
        let mut written = record.to_vec();
        write_player(&player, &mut written, schema.player).expect("encode");
        let name = schema
            .player
            .texts
            .iter()
            .find(|t| t.text == PlayerText::Name)
            .expect("name spec");
        let at = name.byte_offset as usize;
        assert_eq!(&written[at..at + 3], b"AB\0");
        assert_eq!(
            written[at + 3..at + name.len as usize],
            record[at + 3..at + name.len as usize]
        );

        // A name longer than len - 1 bytes is refused.
        player.name = "x".repeat(name.len as usize);
        assert!(matches!(
            write_player(&player, &mut written, schema.player),
            Err(CodecError::Text { .. })
        ));

        // A single-byte shirt char writes as its byte and reads back equal.
        let shirt = schema
            .player
            .texts
            .iter()
            .find(|t| t.text == PlayerText::ShirtName)
            .expect("shirt spec");
        player.name = "AB".to_string();
        player.shirt_name = "MéXICO".to_string();
        write_player(&player, &mut written, schema.player).expect("encode");
        assert_eq!(written[shirt.byte_offset as usize + 1], 0xE9);
        let reread = read_player(&written, schema.player).expect("redecode");
        assert_eq!(reread.shirt_name, "MéXICO");

        // A char above U+00FF is refused.
        player.shirt_name = "€URO".to_string();
        assert!(matches!(
            write_player(&player, &mut written, schema.player),
            Err(CodecError::Text { .. })
        ));
    }

    #[test]
    fn a_gated_field_left_none_is_an_error_not_a_zero() {
        let payload = payload(PesVersion::Pes21);
        let schema = schema_for(PesVersion::Pes21);
        let mut player = find_player(&payload, schema, 70101);
        player.stats.star = None;
        let mut record = vec![0u8; schema.player.size];
        match write_player(&player, &mut record, schema.player) {
            Err(CodecError::Missing { field }) => assert_eq!(field, "Star"),
            other => panic!("expected CodecError::Missing, got {other:?}"),
        }
    }

    #[test]
    fn a_value_too_wide_for_its_run_is_refused() {
        let schema = crate::schema::RecordSchema {
            size: 2,
            fields: &[crate::schema::FieldSpec {
                field: PlayerField::Form,
                bit_offset: 0,
                bit_width: 3,
            }],
            arrays: &[],
            texts: &[],
        };
        let mut player = PlayerEntry::default();
        player.stats.form = 8;
        let mut record = [0u8; 2];
        assert!(matches!(
            write_player(&player, &mut record, &schema),
            Err(CodecError::ValueTooWide {
                value: 8,
                width: 3,
                ..
            })
        ));
    }
}
