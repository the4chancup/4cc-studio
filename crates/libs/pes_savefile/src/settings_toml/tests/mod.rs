//! The `PlayerSettings` tests: the completeness invariant, `from_player`,
//! `apply` and the key table itself (`model`), and the TOML text half
//! (`document`).

mod document;
mod model;

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
