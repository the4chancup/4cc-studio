//! The player record codec: `read_player`/`read_player_into`/`write_player`.

use super::{CodecError, bits, runs, single_byte, text_bytes, write_text};
use crate::model::player::PlayerEntry;
use crate::schema::RecordSchema;
use crate::schema::fields::{PlayerField, PlayerText};

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
            PlayerText::ShirtName => single_byte(&player.shirt_name, "ShirtName")?,
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
