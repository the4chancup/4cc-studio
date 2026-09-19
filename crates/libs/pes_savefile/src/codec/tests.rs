//! The codec test suite: fixture round trips, literals and censuses.

use std::io::Read;

use pes_version::PesVersion;

use super::team::tactics_runs;
use super::{
    CodecError, bits, read_player, read_player_into, read_roster_into, read_tactics_into,
    read_team, runs, write_player, write_roster, write_tactics, write_team,
};
use crate::model::ingame_face::IngameFace;
use crate::model::player::PlayerEntry;
use crate::model::team::TeamEntry;
use crate::schema::fields::{
    PlayerField, PlayerText, RosterField, TacticsField, TeamField, TeamText,
};
use crate::schema::ingame_face::IngameFaceField;
use crate::schema::{RecordSchema, TacticsSchema};
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
            p15.appearance
                .ingame_face
                .get(IngameFaceField::SkinColor)
                .expect("read run"),
            p15.appearance
                .ingame_face
                .get(IngameFaceField::IrisColor)
                .expect("read run"),
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
            p16.appearance
                .ingame_face
                .get(IngameFaceField::SkinColor)
                .expect("read run"),
            p16.appearance
                .ingame_face
                .get(IngameFaceField::IrisColor)
                .expect("read run"),
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
fn the_ingame_face_run_has_the_versions_length() {
    for version in FIXTURES {
        let payload = payload(version);
        let schema = schema_for(version);
        let mut player = read_player(
            record(&payload, &schema.players, schema.player.size, 0),
            schema.player,
        )
        .expect("decode");
        appearance_for(&payload, schema, &mut player);
        let expected = if version == PesVersion::Pes15 { 46 } else { 50 };
        assert_eq!(
            player.appearance.ingame_face.bytes().len(),
            expected,
            "{version:?} run length"
        );
    }
}

#[test]
fn a_wrong_length_ingame_face_run_is_refused() {
    let payload = payload(PesVersion::Pes17);
    let schema = schema_for(PesVersion::Pes17);
    let mut player = find_player(&payload, schema, 70101);
    player.appearance.ingame_face = IngameFace::from_bytes(vec![0; 49]);
    let mut record = record(&payload, &schema.players, schema.player.size, 0).to_vec();
    match write_player(&player, &mut record, schema.player) {
        Err(CodecError::RunSize { expected, got }) => {
            assert_eq!((expected, got), (50, 49));
        }
        other => panic!("expected CodecError::RunSize, got {other:?}"),
    }
    // A run written into a record whose schema has none is ignored.
    let schema15 = schema_for(PesVersion::Pes15);
    let mut record = vec![0u8; schema15.player.size];
    write_player(&player, &mut record, schema15.player).expect("no run on the 15/16 player record");
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
        ingame_face: None,
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

/// The id a roster record carries, read through its schema.
fn roster_id(record: &[u8], schema: &RecordSchema<RosterField, TeamText>) -> u32 {
    let (_, offset, width) = runs(schema)
        .find(|(f, _, _)| *f == RosterField::TeamId)
        .expect("the roster schema has a TeamId run");
    bits::read_bits(record, offset, width)
}

/// The id a tactics record carries, read through its schema.
fn tactics_id(record: &[u8], schema: &TacticsSchema) -> u32 {
    let (_, offset, width) = schema
        .fields
        .iter()
        .map(|spec| (spec.field, spec.bit_offset, spec.bit_width))
        .find(|(f, _, _)| *f == TacticsField::TeamId)
        .expect("the tactics schema has a TeamId run");
    bits::read_bits(record, offset, width)
}

/// The id a team record carries.
fn team_id(record: &[u8], schema: &RecordSchema<TeamField, TeamText>) -> u32 {
    let (_, offset, width) = runs(schema)
        .find(|(f, _, _)| *f == TeamField::Id)
        .expect("the team schema has an Id run");
    bits::read_bits(record, offset, width)
}

/// The joined `TeamEntry` for `id`: team record plus its roster and
/// tactics records.
fn find_team(payload: &[u8], schema: &VersionSchema, id: u32) -> TeamEntry {
    for i in 0..count(payload, &schema.teams) {
        let rec = record(payload, &schema.teams, schema.team.size, i);
        let mut team = read_team(rec, schema.team).expect("team record decodes");
        if team.id != id {
            continue;
        }
        for j in 0..count(payload, &schema.rosters) {
            let rec = record(payload, &schema.rosters, schema.roster.size, j);
            if roster_id(rec, schema.roster) == id {
                read_roster_into(&mut team, rec, schema.roster).expect("roster record decodes");
                break;
            }
        }
        for j in 0..count(payload, &schema.tactics) {
            let rec = record(payload, &schema.tactics, schema.tactic.size, j);
            if tactics_id(rec, schema.tactic) == id {
                read_tactics_into(&mut team, rec, schema.tactic).expect("tactics record decodes");
                break;
            }
        }
        return team;
    }
    panic!("team {id} not in the save");
}

#[test]
fn every_team_roster_and_tactics_record_round_trips_byte_for_byte() {
    for version in FIXTURES {
        let payload = payload(version);
        let schema = schema_for(version);
        for i in 0..count(&payload, &schema.teams) {
            let rec = record(&payload, &schema.teams, schema.team.size, i);
            let team = read_team(rec, schema.team).expect("decode team");
            let mut written = rec.to_vec();
            write_team(&team, &mut written, schema.team).expect("encode team");
            assert_eq!(written, rec, "{version:?} team record {i}");
        }
        for i in 0..count(&payload, &schema.rosters) {
            let rec = record(&payload, &schema.rosters, schema.roster.size, i);
            let mut team = TeamEntry {
                id: roster_id(rec, schema.roster),
                ..TeamEntry::default()
            };
            read_roster_into(&mut team, rec, schema.roster).expect("decode roster");
            let mut written = rec.to_vec();
            write_roster(&team, &mut written, schema.roster).expect("encode roster");
            assert_eq!(written, rec, "{version:?} roster record {i}");
        }
        for i in 0..count(&payload, &schema.tactics) {
            let rec = record(&payload, &schema.tactics, schema.tactic.size, i);
            let mut team = TeamEntry {
                id: tactics_id(rec, schema.tactic),
                ..TeamEntry::default()
            };
            read_tactics_into(&mut team, rec, schema.tactic).expect("decode tactics");
            let mut written = rec.to_vec();
            write_tactics(&team, &mut written, schema.tactic).expect("encode tactics");
            assert_eq!(written, rec, "{version:?} tactics record {i}");
        }
    }
}

#[test]
fn team_roster_and_tactics_sections_list_the_same_teams() {
    let counts = [220usize, 220, 220, 202, 346, 220];
    for (i, version) in FIXTURES.iter().enumerate() {
        let payload = payload(*version);
        let schema = schema_for(*version);
        assert_eq!(
            count(&payload, &schema.teams),
            counts[i],
            "{version:?} team count"
        );
        assert_eq!(count(&payload, &schema.rosters), counts[i]);
        assert_eq!(count(&payload, &schema.tactics), counts[i]);
        // The three sections must list the same ids in the same order.
        for j in 0..counts[i] {
            let team_rec = record(&payload, &schema.teams, schema.team.size, j);
            let roster_rec = record(&payload, &schema.rosters, schema.roster.size, j);
            let tactics_rec = record(&payload, &schema.tactics, schema.tactic.size, j);
            let id = team_id(team_rec, schema.team);
            assert_eq!(
                roster_id(roster_rec, schema.roster),
                id,
                "{version:?} slot {j}"
            );
            assert_eq!(
                tactics_id(tactics_rec, schema.tactic),
                id,
                "{version:?} slot {j}"
            );
            // The joined entry round-trips through all three writers.
            let mut team = read_team(team_rec, schema.team).expect("decode team");
            read_roster_into(&mut team, roster_rec, schema.roster).expect("decode roster");
            read_tactics_into(&mut team, tactics_rec, schema.tactic).expect("decode tactics");
            let mut w_team = team_rec.to_vec();
            let mut w_roster = roster_rec.to_vec();
            let mut w_tactics = tactics_rec.to_vec();
            write_team(&team, &mut w_team, schema.team).expect("encode team");
            write_roster(&team, &mut w_roster, schema.roster).expect("encode roster");
            write_tactics(&team, &mut w_tactics, schema.tactic).expect("encode tactics");
            assert_eq!(w_team, team_rec, "{version:?} team record {j}");
            assert_eq!(w_roster, roster_rec, "{version:?} roster record {j}");
            assert_eq!(w_tactics, tactics_rec, "{version:?} tactics record {j}");
        }
        // A roster or tactics record against the wrong team is refused.
        let rec = record(&payload, &schema.rosters, schema.roster.size, 0);
        let mut team = TeamEntry {
            id: roster_id(rec, schema.roster) + 1,
            ..TeamEntry::default()
        };
        assert!(matches!(
            read_roster_into(&mut team, rec, schema.roster),
            Err(CodecError::TeamIdMismatch { .. })
        ));
        let rec = record(&payload, &schema.tactics, schema.tactic.size, 0);
        assert!(matches!(
            read_tactics_into(&mut team, rec, schema.tactic),
            Err(CodecError::TeamIdMismatch { .. })
        ));
    }
}

#[test]
fn team_701_and_team_100_literals() {
    // Team 701's name and short name on the five versions that carry it.
    for version in [
        PesVersion::Pes15,
        PesVersion::Pes16,
        PesVersion::Pes17,
        PesVersion::Pes18,
        PesVersion::Pes21,
    ] {
        let payload = payload(version);
        let team = find_team(&payload, schema_for(version), 701);
        assert_eq!(team.name, "/3/", "{version:?} team 701 name");
        assert_eq!(team.short_name, "3", "{version:?} team 701 short name");
    }
    // PES 17/18 team 701 colours.
    for version in [PesVersion::Pes17, PesVersion::Pes18] {
        let payload = payload(version);
        let team = find_team(&payload, schema_for(version), 701);
        let colors = team.colors.expect("PES 17/18 teams have colours");
        assert_eq!(
            [
                (colors[0].red, colors[0].green, colors[0].blue),
                (colors[1].red, colors[1].green, colors[1].blue)
            ],
            [(63, 42, 16), (0, 0, 0)],
            "{version:?} team 701 colours"
        );
    }
    // PES 19 team 100.
    let team = find_team(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        100,
    );
    assert_eq!(team.name, "MAN RED");
    assert_eq!(team.short_name, "MAN");
    assert_eq!(team.manager_id, Some(362381));
    assert_eq!(team.stadium_id, Some(46));
    let colors = team.colors.expect("PES 19 teams have colours");
    assert_eq!(
        [
            (colors[0].red, colors[0].green, colors[0].blue),
            (colors[1].red, colors[1].green, colors[1].blue)
        ],
        [(10, 10, 10), (10, 10, 10)]
    );
    assert_eq!(team.roster.len(), 40);
    assert_eq!(
        team.roster[..4]
            .iter()
            .map(|s| s.player_id)
            .collect::<Vec<_>>(),
        [0, 0, 0, 0]
    );
    // PES 21 team 701 ids.
    let team = find_team(
        &payload(PesVersion::Pes21),
        schema_for(PesVersion::Pes21),
        701,
    );
    assert_eq!(team.manager_id, Some(701));
    assert_eq!(team.stadium_id, Some(29));
    assert_eq!(team.roster.len(), 40);
    assert_eq!(
        team.roster[..4]
            .iter()
            .map(|s| (s.player_id, s.number))
            .collect::<Vec<_>>(),
        [(70101, 1), (70102, 2), (70103, 3), (70104, 4)]
    );
    // PES 15 team 701 roster.
    let team = find_team(
        &payload(PesVersion::Pes15),
        schema_for(PesVersion::Pes15),
        701,
    );
    assert_eq!(team.roster.len(), 32);
    assert_eq!(
        team.roster[..4]
            .iter()
            .map(|s| (s.player_id, s.number))
            .collect::<Vec<_>>(),
        [(70101, 1), (70102, 2), (70103, 3), (70104, 4)]
    );
    // PES 17 team 701 kit slot 0.
    let team = find_team(
        &payload(PesVersion::Pes17),
        schema_for(PesVersion::Pes17),
        701,
    );
    let kit = team.kit_slots.expect("PES 17 teams have kit slots")[0];
    assert_eq!(kit.number, 0);
    assert_eq!(kit.binding, 701 * 0x40);
}

#[test]
fn tactics_literals() {
    // PES 17 team 701.
    let team = find_team(
        &payload(PesVersion::Pes17),
        schema_for(PesVersion::Pes17),
        701,
    );
    let formation = &team.tactics.presets[0].formations[0];
    assert_eq!(
        formation
            .players
            .iter()
            .map(|s| s.position)
            .collect::<Vec<_>>(),
        [0, 1, 1, 3, 2, 4, 4, 7, 6, 8, 12]
    );
    assert_eq!(
        formation.players.iter().map(|s| s.x).collect::<Vec<_>>(),
        [52, 63, 41, 89, 15, 64, 40, 86, 18, 52, 52]
    );
    assert_eq!(team.tactics.presets[0].sliders.support_range, 5);
    assert_eq!(team.tactics.set_pieces.captain, 0);
    assert_eq!(
        team.tactics.starting_eleven,
        [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    );
    // PES 15 team 701.
    let team = find_team(
        &payload(PesVersion::Pes15),
        schema_for(PesVersion::Pes15),
        701,
    );
    assert_eq!(
        team.tactics.presets[0].formations[0]
            .players
            .iter()
            .map(|s| s.position)
            .collect::<Vec<_>>(),
        [0, 1, 1, 3, 2, 4, 5, 8, 11, 11, 12]
    );
    assert_eq!(team.tactics.presets[0].sliders.support_range, 3);
    assert_eq!(team.tactics.set_pieces.captain, 10);
    assert_eq!(
        team.tactics.starting_eleven,
        [0, 3, 2, 4, 1, 5, 6, 7, 8, 9, 10]
    );
    assert!(team.tactics.presets[0].attack_instructions.is_none());
    assert!(team.tactics.presets[0].defence_instructions.is_none());
    assert!(team.tactics.auto.substitution.is_none());
    assert!(team.tactics.auto.offside_trap.is_none());
    assert!(team.tactics.auto.preset_change.is_none());
    assert!(team.tactics.auto.attack_defence_levels.is_none());
    // PES 19 team 100: every taker and starter unset.
    let team = find_team(
        &payload(PesVersion::Pes19),
        schema_for(PesVersion::Pes19),
        100,
    );
    assert_eq!(team.tactics.set_pieces.captain, 255);
    assert_eq!(team.tactics.starting_eleven, [255; 11]);
}

#[test]
fn the_tactics_census_matches_the_measured_ranges() {
    for version in FIXTURES {
        let payload = payload(version);
        let schema = schema_for(version);
        for i in 0..count(&payload, &schema.tactics) {
            let rec = record(&payload, &schema.tactics, schema.tactic.size, i);
            let mut team = TeamEntry {
                id: tactics_id(rec, schema.tactic),
                ..TeamEntry::default()
            };
            // Every boolean byte decodes (0 or 1) or the read fails.
            read_tactics_into(&mut team, rec, schema.tactic).expect("decode tactics");
            for preset in &team.tactics.presets {
                for value in [
                    preset.sliders.support_range,
                    preset.sliders.defensive_line,
                    preset.sliders.compactness,
                ] {
                    assert!(
                        (1..=10).contains(&value),
                        "{version:?} tactics {i} slider {value}"
                    );
                }
                for value in [
                    preset.sliders.numbers_in_attack,
                    preset.sliders.numbers_in_defence,
                ] {
                    assert!(
                        (1..=3).contains(&value),
                        "{version:?} tactics {i} numbers {value}"
                    );
                }
            }
        }
    }
    // A synthetic boolean byte of 2 is exactly one NotBoolean.
    let payload = payload(PesVersion::Pes17);
    let schema = schema_for(PesVersion::Pes17);
    let mut rec = record(&payload, &schema.tactics, schema.tactic.size, 0).to_vec();
    let (field, offset, _) = tactics_runs(schema.tactic)
        .into_iter()
        .find(|(f, _, _)| {
            matches!(
                f,
                TacticsField::Preset {
                    preset: 0,
                    field: crate::schema::fields::PresetField::AttackingStyle,
                }
            )
        })
        .expect("a style run");
    assert_eq!(offset % 8, 0, "style runs are byte-aligned");
    rec[offset as usize / 8] = 2;
    let mut team = TeamEntry {
        id: tactics_id(&rec, schema.tactic),
        ..TeamEntry::default()
    };
    match read_tactics_into(&mut team, &rec, schema.tactic) {
        Err(CodecError::NotBoolean { field: name, value }) => {
            assert!(name.contains("AttackingStyle"), "{name}");
            assert_eq!(value, 2);
        }
        other => panic!("expected CodecError::NotBoolean for {field:?}, got {other:?}"),
    }
}
