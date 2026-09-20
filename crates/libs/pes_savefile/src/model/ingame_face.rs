//! `IngameFace`: the appearance block's byte-22-to-end run, carried verbatim
//! as opaque bytes with typed accessors for the known bits
//! (`schema::ingame_face::INGAME_FACE_FIELDS`).

use crate::codec::{CodecError, bits};
use crate::schema::FieldSpec;
use crate::schema::ingame_face::{INGAME_FACE_FIELDS, IngameFaceField};

/// The ingame-face run (50 bytes, 46 on PES 15). `Default` is empty: a fresh
/// `PlayerEntry` has no run until one is read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct IngameFace(Vec<u8>);

/// The `FieldSpec` row of `field`; the table covers every variant.
fn spec(field: IngameFaceField) -> &'static FieldSpec<IngameFaceField> {
    INGAME_FACE_FIELDS
        .iter()
        .find(|s| s.field == field)
        .expect("INGAME_FACE_FIELDS covers every variant; checked by schema::ingame_face::tests")
}

impl IngameFace {
    /// Whether the run's bytes reach the field's last bit.
    fn covers(&self, spec: &FieldSpec<IngameFaceField>) -> bool {
        (spec.bit_offset + spec.bit_width).div_ceil(8) <= self.0.len() as u32
    }

    /// The field's value inside the run: `CodecError::NoIngameFaceRun` when the
    /// run does not reach the field (an entry never read from a record), an
    /// error, not a silent 0, as for an unset gated field.
    pub fn get(&self, field: IngameFaceField) -> Result<u8, CodecError> {
        let spec = spec(field);
        if !self.covers(spec) {
            return Err(CodecError::NoIngameFaceRun {
                field: format!("{field:?}"),
            });
        }
        Ok(bits::read_bits(&self.0, spec.bit_offset, spec.bit_width) as u8)
    }

    /// Writes `value` into the field's bits: `CodecError::NoIngameFaceRun` as
    /// for `get`, `ValueTooWide` when the value does not fit the field's bits.
    pub fn set(&mut self, field: IngameFaceField, value: u8) -> Result<(), CodecError> {
        let spec = spec(field);
        if !self.covers(spec) {
            return Err(CodecError::NoIngameFaceRun {
                field: format!("{field:?}"),
            });
        }
        if u32::from(value) >= (1u32 << spec.bit_width) {
            return Err(CodecError::ValueTooWide {
                field: format!("{field:?}"),
                value: u32::from(value),
                width: spec.bit_width,
            });
        }
        bits::write_bits(
            &mut self.0,
            spec.bit_offset,
            spec.bit_width,
            u32::from(value),
        );
        Ok(())
    }

    /// The run's bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    /// The run with `fields`' bits zeroed.
    fn masked(&self, fields: impl Iterator<Item = IngameFaceField>) -> Vec<u8> {
        let mut out = self.0.clone();
        for field in fields {
            let spec = spec(field);
            if self.covers(spec) {
                bits::write_bits(&mut out, spec.bit_offset, spec.bit_width, 0);
            }
        }
        out
    }

    /// The run with the player-gloves and skin bits zeroed: what the
    /// fingerprint hashes (the reference masks the same two, because they
    /// overlap fields it lists separately).
    pub fn normalized(&self) -> Vec<u8> {
        self.masked(
            [
                IngameFaceField::PlayerGloves,
                IngameFaceField::PlayerGlovesColor,
                IngameFaceField::SkinColor,
            ]
            .into_iter(),
        )
    }

    /// The run with every known field's bits zeroed: the part of the run no
    /// `IngameFaceField` names, which the comparator reports as one row.
    pub(crate) fn undecoded(&self) -> Vec<u8> {
        self.masked(IngameFaceField::ALL.into_iter())
    }

    /// Copies `source`'s bytes over this run's prefix (`min` of the two
    /// lengths) and returns how many source bytes did not fit: 0 unless the
    /// target run is shorter (a 50-byte source onto PES 15's 46 returns 4).
    /// Conversion uses it so a shorter target keeps its own tail and a longer
    /// source's tail is reported, never invented.
    pub(crate) fn copy_from(&mut self, source: &IngameFace) -> usize {
        let n = self.0.len().min(source.0.len());
        self.0[..n].copy_from_slice(&source.0[..n]);
        source.0.len() - n
    }

    /// A run around `bytes`, as copied out of a record.
    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bits `spec` covers as a per-byte mask.
    fn covered(spec: &FieldSpec<IngameFaceField>, byte: usize) -> u8 {
        let mut mask = 0u8;
        for i in 0..spec.bit_width {
            let pos = spec.bit_offset + i;
            if pos / 8 == byte as u32 {
                mask |= 1 << (pos % 8);
            }
        }
        mask
    }

    /// A run with no two equal bytes.
    fn run() -> Vec<u8> {
        (0..50u8)
            .map(|i| i.wrapping_mul(37).wrapping_add(11))
            .collect()
    }

    #[test]
    fn set_then_get_round_trips_every_field_and_touches_only_its_bits() {
        for spec in INGAME_FACE_FIELDS {
            let mut face = IngameFace::from_bytes(run());
            let before = face.bytes().to_vec();
            let max = (1u8 << spec.bit_width) - 1;
            face.set(spec.field, max).expect("the max fits");
            assert_eq!(
                face.get(spec.field).expect("covered"),
                max,
                "{:?}",
                spec.field
            );
            for (byte, (&after, &was)) in face.bytes().iter().zip(&before).enumerate() {
                let keep = !covered(spec, byte);
                assert_eq!(
                    after & keep,
                    was & keep,
                    "{:?} changed bits outside its run in byte {byte}",
                    spec.field
                );
            }
            assert!(matches!(
                face.set(spec.field, max + 1),
                Err(CodecError::ValueTooWide { .. })
            ));
        }
    }

    #[test]
    fn an_unread_run_is_no_ingame_face_run_not_a_panic() {
        let mut face = IngameFace::default();
        assert!(matches!(
            face.get(IngameFaceField::SkinColor),
            Err(CodecError::NoIngameFaceRun { .. })
        ));
        assert!(matches!(
            face.set(IngameFaceField::SkinColor, 1),
            Err(CodecError::NoIngameFaceRun { .. })
        ));
        assert!(face.normalized().is_empty());
    }

    #[test]
    fn normalized_zeroes_exactly_the_gloves_and_skin_bits() {
        let face = IngameFace::from_bytes(run());
        let out = face.normalized();
        let mut masked = vec![0u8; out.len()];
        for field in [
            IngameFaceField::PlayerGloves,
            IngameFaceField::PlayerGlovesColor,
            IngameFaceField::SkinColor,
        ] {
            for (byte, mask) in masked.iter_mut().enumerate() {
                *mask |= covered(spec(field), byte);
            }
        }
        for byte in 0..out.len() {
            assert_eq!(
                out[byte] & masked[byte],
                0,
                "byte {byte} keeps a masked bit"
            );
            assert_eq!(
                out[byte] & !masked[byte],
                face.bytes()[byte] & !masked[byte],
                "byte {byte} lost an unmasked bit"
            );
        }
        // Seven bits exactly: byte 0 low 4, byte 23 low 3.
        assert_eq!(masked.iter().map(|m| m.count_ones()).sum::<u32>(), 7);
        assert_eq!(masked[0], 0x0F);
        assert_eq!(masked[23], 0x07);
    }
}
