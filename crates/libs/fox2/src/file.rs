//! The `.fox2` binary layout: header, entities with their static and dynamic properties,
//! then the string table and the `end` trailer.

use std::collections::HashMap;

use crate::hash::{Dictionary, hash_string};
use crate::values::{Container, FoxString, Values, read_value, write_values};

/// What a `.fox2` read or write can fail with.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Fox2Error {
    /// The buffer ends before a field the layout requires.
    #[error("file is truncated")]
    Truncated,
    /// The first word is not `0x786F62F2`.
    #[error("invalid fox2 magic")]
    BadMagic,
    /// A fixed field carried a value other than the one the layout fixes.
    #[error("unexpected value {value} for {what}")]
    UnexpectedConstant {
        /// The field that carried the value.
        what: &'static str,
        /// The value found.
        value: u64,
    },
    /// Data type word 23 (PropertyInfo) or above 24.
    #[error("unsupported data type {0}")]
    UnsupportedDataType(u8),
    /// A container word other than 0-3.
    #[error("unknown container {0}")]
    UnknownContainer(u8),
    /// A string table entry that is not UTF-8.
    #[error("invalid UTF-8 string at {offset}")]
    InvalidUtf8 {
        /// Where the bad text starts.
        offset: usize,
    },
    /// Nonzero bytes after the `end` trailer.
    #[error("bytes after the end trailer are not zero")]
    TrailingBytes,
}

/// A `.fox2` file: its entities and its string table, in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct Fox2File {
    /// The entities, in file order.
    pub entities: Vec<Entity>,
    /// The string table entries, verbatim and in file order.
    pub string_table: Vec<TableEntry>,
}

/// One string table entry (the hash may disagree with `text` — opaque entries occur).
#[derive(Debug, Clone, PartialEq)]
pub struct TableEntry {
    /// The stored hash.
    pub hash: u64,
    /// The stored text.
    pub text: String,
}

/// One entity: its header fields and its static then dynamic properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    /// The class name (a `Hash` until resolved).
    pub class_name: FoxString,
    /// Carried header word (values look like class sizes).
    pub unknown1: i16,
    /// Carried header word (0 in all measured files).
    pub unknown2: i32,
    /// The entity's class version.
    pub version: i16,
    /// The in-engine address `EntityPtr` values point at.
    pub address: u32,
    /// Static properties, in file order.
    pub static_properties: Vec<Property>,
    /// Dynamic properties, in file order.
    pub dynamic_properties: Vec<Property>,
}

/// One property: name, container, the `StringMap` keys, and the values.
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    /// The property name (a `Hash` until resolved).
    pub name: FoxString,
    /// The container.
    pub container: Container,
    /// `StringMap` keys, one per value; empty for the other containers.
    pub keys: Vec<FoxString>,
    /// The values.
    pub values: Values,
}

/// `at + len`, `Truncated` on overflow.
fn offset_sum(at: usize, len: usize) -> Result<usize, Fox2Error> {
    at.checked_add(len).ok_or(Fox2Error::Truncated)
}

fn slice_at(bytes: &[u8], at: usize, len: usize) -> Result<&[u8], Fox2Error> {
    bytes
        .get(at..offset_sum(at, len)?)
        .ok_or(Fox2Error::Truncated)
}

fn unexpected(what: &'static str, value: u64) -> Fox2Error {
    Fox2Error::UnexpectedConstant { what, value }
}

fn u8_at(bytes: &[u8], at: usize) -> Result<u8, Fox2Error> {
    Ok(slice_at(bytes, at, 1)?[0])
}

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, Fox2Error> {
    let mut word = [0u8; 2];
    word.copy_from_slice(slice_at(bytes, at, 2)?);
    Ok(u16::from_le_bytes(word))
}

fn i16_at(bytes: &[u8], at: usize) -> Result<i16, Fox2Error> {
    let mut word = [0u8; 2];
    word.copy_from_slice(slice_at(bytes, at, 2)?);
    Ok(i16::from_le_bytes(word))
}

fn u32_at(bytes: &[u8], at: usize) -> Result<u32, Fox2Error> {
    let mut word = [0u8; 4];
    word.copy_from_slice(slice_at(bytes, at, 4)?);
    Ok(u32::from_le_bytes(word))
}

fn i32_at(bytes: &[u8], at: usize) -> Result<i32, Fox2Error> {
    let mut word = [0u8; 4];
    word.copy_from_slice(slice_at(bytes, at, 4)?);
    Ok(i32::from_le_bytes(word))
}

fn u64_at(bytes: &[u8], at: usize) -> Result<u64, Fox2Error> {
    let mut word = [0u8; 8];
    word.copy_from_slice(slice_at(bytes, at, 8)?);
    Ok(u64::from_le_bytes(word))
}

/// `len` zero bytes at `at` (`UnexpectedConstant` naming `what` otherwise).
fn zeros_at(bytes: &[u8], at: usize, len: usize, what: &'static str) -> Result<(), Fox2Error> {
    let zeros = slice_at(bytes, at, len)?;
    if zeros.iter().any(|byte| *byte != 0) {
        let mut word = [0u8; 8];
        word[..zeros.len().min(8)].copy_from_slice(&zeros[..zeros.len().min(8)]);
        return Err(unexpected(what, u64::from_le_bytes(word)));
    }
    Ok(())
}

/// `at` rounded up to the next multiple of 16.
fn align16(at: usize) -> usize {
    (at + 15) & !15
}

/// A checked size field (`i32` in the file, `usize` for us).
fn checked_size(field: i32, what: &'static str) -> Result<usize, Fox2Error> {
    usize::try_from(field).map_err(|_| unexpected(what, field as i64 as u64))
}

fn read_property(bytes: &[u8], at: usize) -> Result<(Property, usize), Fox2Error> {
    let start = at;
    let name = FoxString::Hash(u64_at(bytes, at)?);
    let mut values = Values::empty(u8_at(bytes, at + 8)?)?;
    let container = Container::from_word(u8_at(bytes, at + 9)?)
        .ok_or(Fox2Error::UnknownContainer(bytes[at + 9]))?;
    let count = usize::from(u16_at(bytes, at + 10)?);
    if u16_at(bytes, at + 12)? != 32 {
        return Err(unexpected(
            "property offset",
            u64::from(u16_at(bytes, at + 12)?),
        ));
    }
    let size = usize::from(u16_at(bytes, at + 14)?);
    zeros_at(bytes, at + 16, 16, "property padding")?;
    let mut cursor = offset_sum(at, 32)?;
    let mut keys = Vec::new();
    if container == Container::StringMap {
        // Every entry carries a key and at least one value byte; a count that cannot fit
        // fails before the first push.
        slice_at(
            bytes,
            cursor,
            count.checked_mul(9).ok_or(Fox2Error::Truncated)?,
        )?;
        for _ in 0..count {
            keys.push(FoxString::Hash(u64_at(bytes, cursor)?));
            let consumed = read_value(&mut values, bytes, cursor + 8)?;
            let entry_end = offset_sum(cursor, 8)?
                .checked_add(consumed)
                .ok_or(Fox2Error::Truncated)?;
            let padded = align16(entry_end);
            zeros_at(bytes, entry_end, padded - entry_end, "entry padding")?;
            cursor = padded;
        }
    } else {
        // `count` elements of a known width must fit the buffer before any push.
        let needed = count
            .checked_mul(values.element_size())
            .ok_or(Fox2Error::Truncated)?;
        slice_at(bytes, cursor, needed)?;
        for _ in 0..count {
            cursor += read_value(&mut values, bytes, cursor)?;
        }
    }
    let end = offset_sum(start, size)?;
    if align16(cursor) != end {
        return Err(unexpected("property size", size as u64));
    }
    zeros_at(bytes, cursor, end - cursor, "property padding")?;
    Ok((
        Property {
            name,
            container,
            keys,
            values,
        },
        end,
    ))
}

fn read_entity(bytes: &[u8], at: usize) -> Result<(Entity, usize), Fox2Error> {
    let start = at;
    if i16_at(bytes, at)? != 64 {
        return Err(unexpected(
            "entity header size",
            i16_at(bytes, at)? as i64 as u64,
        ));
    }
    let unknown1 = i16_at(bytes, at + 2)?;
    if i16_at(bytes, at + 4)? != 0 {
        return Err(unexpected(
            "entity padding",
            i16_at(bytes, at + 4)? as i64 as u64,
        ));
    }
    if u32_at(bytes, at + 6)? != 0x746E65 {
        return Err(unexpected(
            "entity magic",
            u64::from(u32_at(bytes, at + 6)?),
        ));
    }
    let address = u32_at(bytes, at + 10)?;
    if u32_at(bytes, at + 14)? != 0 {
        return Err(unexpected(
            "entity padding",
            u64::from(u32_at(bytes, at + 14)?),
        ));
    }
    let unknown2 = i32_at(bytes, at + 18)?;
    if i32_at(bytes, at + 22)? != 0 {
        return Err(unexpected(
            "entity zero word",
            i32_at(bytes, at + 22)? as i64 as u64,
        ));
    }
    let version = i16_at(bytes, at + 26)?;
    let class_name = FoxString::Hash(u64_at(bytes, at + 28)?);
    let static_count = usize::from(u16_at(bytes, at + 36)?);
    let dynamic_count = usize::from(u16_at(bytes, at + 38)?);
    if i32_at(bytes, at + 40)? != 64 {
        return Err(unexpected(
            "entity offset",
            i32_at(bytes, at + 40)? as i64 as u64,
        ));
    }
    let static_data_size = i32_at(bytes, at + 44)?;
    let data_size = i32_at(bytes, at + 48)?;
    zeros_at(bytes, at + 52, 12, "entity padding")?;

    // Every property is at least its 32-byte header (an empty one is exactly that), so a
    // hostile count fails against the buffer length before the loop allocates.
    let property_count = static_count
        .checked_add(dynamic_count)
        .ok_or(Fox2Error::Truncated)?;
    slice_at(
        bytes,
        offset_sum(start, 64)?,
        property_count.checked_mul(32).ok_or(Fox2Error::Truncated)?,
    )?;

    let mut cursor = offset_sum(start, 64)?;
    let mut static_properties = Vec::with_capacity(static_count);
    for _ in 0..static_count {
        let (property, end) = read_property(bytes, cursor)?;
        static_properties.push(property);
        cursor = end;
    }
    if cursor - start != checked_size(static_data_size, "entity static data size")? {
        return Err(unexpected(
            "entity static data size",
            static_data_size as i64 as u64,
        ));
    }
    let mut dynamic_properties = Vec::with_capacity(dynamic_count);
    for _ in 0..dynamic_count {
        let (property, end) = read_property(bytes, cursor)?;
        dynamic_properties.push(property);
        cursor = end;
    }
    if cursor - start != checked_size(data_size, "entity data size")? {
        return Err(unexpected("entity data size", data_size as i64 as u64));
    }
    Ok((
        Entity {
            class_name,
            unknown1,
            unknown2,
            version,
            address,
            static_properties,
            dynamic_properties,
        },
        cursor,
    ))
}

impl Fox2File {
    /// Reads a `.fox2` binary. Strings stay `Hash`; the table stays verbatim, so a clean
    /// file rewrites byte-identically.
    pub fn read(bytes: &[u8]) -> Result<Fox2File, Fox2Error> {
        if u32_at(bytes, 0)? != 0x786F62F2 {
            return Err(Fox2Error::BadMagic);
        }
        if u32_at(bytes, 4)? != 0x35 {
            return Err(unexpected("version word", u64::from(u32_at(bytes, 4)?)));
        }
        let entity_count = i32_at(bytes, 8)?;
        let string_table_offset = i32_at(bytes, 12)?;
        if i32_at(bytes, 16)? != 32 {
            return Err(unexpected("header size", i32_at(bytes, 16)? as i64 as u64));
        }
        zeros_at(bytes, 20, 12, "header padding")?;

        // Each entity is at least its 64-byte header; a hostile count fails here, before
        // any allocation.
        if entity_count < 0 {
            return Err(Fox2Error::Truncated);
        }
        slice_at(
            bytes,
            32,
            (entity_count as usize)
                .checked_mul(64)
                .ok_or(Fox2Error::Truncated)?,
        )?;

        let mut at = 32usize;
        let mut entities = Vec::with_capacity(entity_count as usize);
        for _ in 0..entity_count {
            let (entity, end) = read_entity(bytes, at)?;
            entities.push(entity);
            at = end;
        }
        if at != checked_size(string_table_offset, "string table offset")? {
            return Err(unexpected(
                "string table offset",
                string_table_offset as i64 as u64,
            ));
        }

        let mut string_table = Vec::new();
        loop {
            let hash = u64_at(bytes, at)?;
            at += 8;
            if hash == 0 {
                break;
            }
            let len = usize::try_from(u32_at(bytes, at)?).map_err(|_| Fox2Error::Truncated)?;
            at += 4;
            let text_bytes = slice_at(bytes, at, len)?;
            let text = std::str::from_utf8(text_bytes)
                .map_err(|_| Fox2Error::InvalidUtf8 { offset: at })?
                .to_string();
            string_table.push(TableEntry { hash, text });
            at = offset_sum(at, len)?;
        }

        zeros_at(bytes, at, align16(at) - at, "trailer padding")?;
        at = align16(at);
        let trailer = slice_at(bytes, at, 5)?;
        if trailer != [0, 0, b'e', b'n', b'd'] {
            return Err(unexpected(
                "end trailer",
                trailer
                    .iter()
                    .enumerate()
                    .map(|(index, byte)| u64::from(*byte) << (8 * index))
                    .sum(),
            ));
        }
        let trailer_end = offset_sum(at, 5)?;
        zeros_at(
            bytes,
            trailer_end,
            align16(trailer_end) - trailer_end,
            "trailer padding",
        )?;
        at = align16(trailer_end);
        if bytes
            .get(at..)
            .ok_or(Fox2Error::Truncated)?
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(Fox2Error::TrailingBytes);
        }
        Ok(Fox2File {
            entities,
            string_table,
        })
    }

    /// The binary form: header, entities with computed sizes, the string table verbatim,
    /// the zero terminator and the `end` trailer.
    pub fn write(&self) -> Result<Vec<u8>, Fox2Error> {
        let mut out = vec![0u8; 32];
        let entity_count = i32::try_from(self.entities.len())
            .map_err(|_| unexpected("entity count", self.entities.len() as u64))?;
        for entity in &self.entities {
            write_entity(entity, &mut out)?;
        }
        let string_table_offset = i32::try_from(out.len())
            .map_err(|_| unexpected("string table offset", out.len() as u64))?;
        for entry in &self.string_table {
            out.extend_from_slice(&entry.hash.to_le_bytes());
            out.extend_from_slice(
                &u32::try_from(entry.text.len())
                    .map_err(|_| unexpected("string length", entry.text.len() as u64))?
                    .to_le_bytes(),
            );
            out.extend_from_slice(entry.text.as_bytes());
        }
        out.extend_from_slice(&0u64.to_le_bytes());
        out.resize(align16(out.len()), 0);
        out.extend_from_slice(&[0, 0, b'e', b'n', b'd']);
        out.resize(align16(out.len()), 0);
        out[12..16].copy_from_slice(&string_table_offset.to_le_bytes());
        out[8..12].copy_from_slice(&entity_count.to_le_bytes());
        out[0..4].copy_from_slice(&0x786F62F2u32.to_le_bytes());
        out[4..8].copy_from_slice(&0x35u32.to_le_bytes());
        out[16..20].copy_from_slice(&32i32.to_le_bytes());
        Ok(out)
    }

    /// Turns `Hash` strings into `Literal`s: the file's own table first (an entry whose
    /// stored hash disagrees with its text is skipped — the format allows opaque entries),
    /// then `dictionary`. Unknown hashes stay.
    pub fn resolve(&mut self, dictionary: Option<&Dictionary>) {
        let mut local: HashMap<u64, &str> = HashMap::new();
        for entry in &self.string_table {
            if entry.hash == hash_string(&entry.text) {
                local.entry(entry.hash).or_insert(entry.text.as_str());
            }
        }
        let lookup = |hash: u64| -> Option<String> {
            local
                .get(&hash)
                .map(|text| (*text).to_string())
                .or_else(|| dictionary.and_then(|map| map.get(hash)).map(str::to_string))
        };
        for entity in &mut self.entities {
            resolve_string(&mut entity.class_name, &lookup);
            for property in entity
                .static_properties
                .iter_mut()
                .chain(entity.dynamic_properties.iter_mut())
            {
                resolve_string(&mut property.name, &lookup);
                for key in &mut property.keys {
                    resolve_string(key, &lookup);
                }
                resolve_values(&mut property.values, &lookup);
            }
        }
    }
}

fn resolve_string(string: &mut FoxString, lookup: &impl Fn(u64) -> Option<String>) {
    let FoxString::Hash(hash) = string else {
        return;
    };
    if let Some(text) = lookup(*hash) {
        *string = FoxString::Literal(text);
    }
}

fn resolve_values(values: &mut Values, lookup: &impl Fn(u64) -> Option<String>) {
    match values {
        Values::String(list) | Values::Path(list) | Values::FilePtr(list) => {
            for string in list {
                resolve_string(string, lookup);
            }
        }
        Values::EntityLink(list) => {
            for link in list {
                resolve_string(&mut link.package, lookup);
                resolve_string(&mut link.archive, lookup);
                resolve_string(&mut link.name, lookup);
            }
        }
        _ => {}
    }
}

fn write_property(property: &Property, out: &mut Vec<u8>) -> Result<(), Fox2Error> {
    let start = out.len();
    out.extend_from_slice(&property.name.hash().to_le_bytes());
    out.push(property.values.data_type());
    out.push(property.container.word());
    let count = u16::try_from(property.values.len())
        .map_err(|_| unexpected("value count", property.values.len() as u64))?;
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    let size_word = out.len();
    out.extend_from_slice(&0u16.to_le_bytes());
    out.resize(out.len() + 16, 0);
    if property.container == Container::StringMap {
        if property.keys.len() != property.values.len() {
            return Err(unexpected("key count", property.keys.len() as u64));
        }
        for (index, key) in property.keys.iter().enumerate() {
            out.extend_from_slice(&key.hash().to_le_bytes());
            write_values(&property.values, index, out);
            out.resize(align16(out.len()), 0);
        }
    } else {
        for index in 0..property.values.len() {
            write_values(&property.values, index, out);
        }
    }
    out.resize(align16(out.len()), 0);
    let size = u16::try_from(out.len() - start)
        .map_err(|_| unexpected("property size", (out.len() - start) as u64))?;
    out[size_word..size_word + 2].copy_from_slice(&size.to_le_bytes());
    Ok(())
}

fn write_entity(entity: &Entity, out: &mut Vec<u8>) -> Result<(), Fox2Error> {
    let start = out.len();
    out.extend_from_slice(&64i16.to_le_bytes());
    out.extend_from_slice(&entity.unknown1.to_le_bytes());
    out.extend_from_slice(&0i16.to_le_bytes());
    out.extend_from_slice(&0x746E65u32.to_le_bytes());
    out.extend_from_slice(&entity.address.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&entity.unknown2.to_le_bytes());
    out.extend_from_slice(&0i32.to_le_bytes());
    out.extend_from_slice(&entity.version.to_le_bytes());
    out.extend_from_slice(&entity.class_name.hash().to_le_bytes());
    let static_count = u16::try_from(entity.static_properties.len())
        .map_err(|_| unexpected("property count", entity.static_properties.len() as u64))?;
    out.extend_from_slice(&static_count.to_le_bytes());
    let dynamic_count = u16::try_from(entity.dynamic_properties.len())
        .map_err(|_| unexpected("property count", entity.dynamic_properties.len() as u64))?;
    out.extend_from_slice(&dynamic_count.to_le_bytes());
    out.extend_from_slice(&64i32.to_le_bytes());
    let static_size_word = out.len();
    out.extend_from_slice(&0i32.to_le_bytes());
    let data_size_word = out.len();
    out.extend_from_slice(&0i32.to_le_bytes());
    out.resize(out.len() + 12, 0);

    for property in &entity.static_properties {
        write_property(property, out)?;
    }
    let static_end = out.len();
    for property in &entity.dynamic_properties {
        write_property(property, out)?;
    }
    let static_data_size = i32::try_from(static_end - start)
        .map_err(|_| unexpected("entity static data size", (static_end - start) as u64))?;
    let data_size = i32::try_from(out.len() - start)
        .map_err(|_| unexpected("entity data size", (out.len() - start) as u64))?;
    out[static_size_word..static_size_word + 4].copy_from_slice(&static_data_size.to_le_bytes());
    out[data_size_word..data_size_word + 4].copy_from_slice(&data_size.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::{EntityLink, WideVector3};

    const AUDI: &[u8] = include_bytes!("../tests/fixtures/audi_low_parts.fox2");
    const AUDI_COMPILED: &[u8] = include_bytes!("../tests/fixtures/audi_low_parts.compiled.fox2");
    const BOOTS: &[u8] = include_bytes!("../tests/fixtures/boots_edit_k0051.fox2");
    const BOOTS_COMPILED: &[u8] =
        include_bytes!("../tests/fixtures/boots_edit_k0051.compiled.fox2");
    const EDIT_SPIKE: &[u8] = include_bytes!("../tests/fixtures/edit_spike.fox2");
    const EDIT_SPIKE_COMPILED: &[u8] = include_bytes!("../tests/fixtures/edit_spike.compiled.fox2");
    const STEWARD: &[u8] = include_bytes!("../tests/fixtures/steward_sit_st074.fox2");
    const STEWARD_COMPILED: &[u8] =
        include_bytes!("../tests/fixtures/steward_sit_st074.compiled.fox2");

    const FIXTURES: [(&str, &[u8]); 8] = [
        ("audi", AUDI),
        ("audi_compiled", AUDI_COMPILED),
        ("boots", BOOTS),
        ("boots_compiled", BOOTS_COMPILED),
        ("edit_spike", EDIT_SPIKE),
        ("edit_spike_compiled", EDIT_SPIKE_COMPILED),
        ("steward", STEWARD),
        ("steward_compiled", STEWARD_COMPILED),
    ];

    #[test]
    fn konami_and_compiled_files_rewrite_byte_identically() {
        for (name, bytes) in FIXTURES {
            let file = Fox2File::read(bytes).expect(name);
            assert_eq!(file.write().as_deref(), Ok(bytes), "{name}");
        }
    }

    #[test]
    fn audi_low_parts_as_measured() {
        let file = Fox2File::read(AUDI).expect("audi");
        assert_eq!(file.entities.len(), 2);
        let first = &file.entities[0];
        assert_eq!(first.class_name.hash(), hash_string("DataSet"));
        assert_eq!(first.unknown1, 296);
        assert_eq!(first.unknown2, 0);
        assert_eq!(first.version, 0);
        assert_eq!(first.address, 0x100);
        assert_eq!(first.static_properties.len(), 3);
        assert!(first.dynamic_properties.is_empty());
        let name = &first.static_properties[0];
        assert_eq!(name.name.hash(), hash_string("name"));
        assert_eq!(name.container, Container::StaticArray);
        assert_eq!(
            name.values,
            Values::String(vec![FoxString::Hash(0xB8A0BF169F98)])
        );
        let map = &first.static_properties[2];
        assert_eq!(map.container, Container::StringMap);
        assert_eq!(
            map.keys,
            vec![FoxString::Hash(hash_string("AudienceModel0000"))]
        );
        assert_eq!(map.values, Values::EntityPtr(vec![0x200]));
        let second = &file.entities[1];
        assert_eq!(second.unknown1, 192);
        assert_eq!(second.version, 1);
        assert_eq!(second.address, 0x200);
        let models = &second.static_properties[2];
        assert_eq!(models.name.hash(), hash_string("models"));
        assert_eq!(models.container, Container::DynamicArray);
        assert!(matches!(models.values, Values::FilePtr(ref list) if list.len() == 1));
        assert_eq!(file.string_table.len(), 11);
        assert_eq!(
            file.string_table[0].text,
            "/Assets/pes16/model/bg/common/audi/scenes/au00.skl"
        );
    }

    #[test]
    fn resolve_uses_the_table_then_the_dictionary() {
        let mut file = Fox2File::read(AUDI).expect("audi");
        file.resolve(None);
        let first = &file.entities[0];
        assert_eq!(first.class_name, FoxString::Literal("DataSet".to_string()));
        assert_eq!(
            first.static_properties[0].values,
            Values::String(vec![FoxString::Hash(0xB8A0BF169F98)])
        );
        assert_eq!(
            first.static_properties[2].keys,
            vec![FoxString::Literal("AudienceModel0000".to_string())]
        );
        file.resolve(Some(&Dictionary::from_lines("\n")));
        assert_eq!(
            file.entities[0].static_properties[0].values,
            Values::String(vec![FoxString::Literal(String::new())])
        );
        assert_eq!(file.write().as_deref(), Ok(AUDI));
    }

    fn property(container: Container, values: Values, keys: &[&str]) -> Property {
        Property {
            name: FoxString::Hash(hash_string("p")),
            container,
            keys: keys
                .iter()
                .map(|key| FoxString::Hash(hash_string(key)))
                .collect(),
            values,
        }
    }

    #[test]
    fn every_value_type_round_trips() {
        let link = || EntityLink {
            package: FoxString::Hash(1),
            archive: FoxString::Hash(2),
            name: FoxString::Hash(3),
            handle: 4,
        };
        let wide = || WideVector3 {
            x: 1.5,
            y: -2.25,
            z: 3.0,
            a: 7,
            b: 8,
        };
        let map_keys: &[&str] = &["k1", "k2"];
        let properties = vec![
            property(Container::StaticArray, Values::Int8(vec![-3, 42]), &[]),
            property(Container::StaticArray, Values::Uint8(vec![7, 200]), &[]),
            property(Container::StaticArray, Values::Int16(vec![-300, 1234]), &[]),
            property(Container::StaticArray, Values::Uint16(vec![60000, 17]), &[]),
            property(
                Container::StaticArray,
                Values::Int32(vec![-70000, 123456]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Uint32(vec![4_000_000_000, 5]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Int64(vec![-9_000_000_000_000, 8]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Uint64(vec![u64::MAX, 1]),
                &[],
            ),
            property(Container::StaticArray, Values::Float(vec![0.5, -2.25]), &[]),
            property(
                Container::StaticArray,
                Values::Double(vec![0.1, -1e300]),
                &[],
            ),
            property(Container::StaticArray, Values::Bool(vec![true, false]), &[]),
            property(
                Container::StaticArray,
                Values::String(vec![FoxString::Hash(11), FoxString::Hash(22)]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Path(vec![FoxString::Hash(33), FoxString::Hash(44)]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::FilePtr(vec![FoxString::Hash(55), FoxString::Hash(66)]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::EntityPtr(vec![0x100, 0x200]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::EntityHandle(vec![9, 10]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Vector3(vec![[1.0, 2.0, 3.0, 4.0], [5.0, 6.0, 7.0, 8.0]]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Vector4(vec![[9.0, 8.0, 7.0, 6.0], [5.0, 4.0, 3.0, 2.0]]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Quat(vec![[0.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0, 0.0]]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Color(vec![[1.0, 0.5, 0.25, 1.0], [0.0, 0.0, 0.0, 0.0]]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Matrix3(vec![[1.0; 9], [2.0; 9]]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::Matrix4(vec![[3.0; 16], [4.0; 16]]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::EntityLink(vec![link(), link()]),
                &[],
            ),
            property(
                Container::StaticArray,
                Values::WideVector3(vec![wide(), wide()]),
                &[],
            ),
            property(Container::StringMap, Values::Int32(vec![1, 2]), map_keys),
            property(
                Container::StringMap,
                Values::Vector4(vec![[1.0; 4], [2.0; 4]]),
                map_keys,
            ),
            property(
                Container::StringMap,
                Values::String(vec![FoxString::Hash(77), FoxString::Hash(88)]),
                map_keys,
            ),
        ];
        let file = Fox2File {
            entities: vec![Entity {
                class_name: FoxString::Hash(hash_string("DataSet")),
                unknown1: 7,
                unknown2: -3,
                version: 2,
                address: 0x400,
                static_properties: properties,
                dynamic_properties: Vec::new(),
            }],
            string_table: vec![
                TableEntry {
                    hash: hash_string("alpha"),
                    text: "alpha".to_string(),
                },
                TableEntry {
                    hash: hash_string("beta"),
                    text: "beta".to_string(),
                },
            ],
        };
        let bytes = file.write().expect("write");
        assert_eq!(Fox2File::read(&bytes).as_ref(), Ok(&file));
    }

    #[test]
    fn writer_slack_is_read_and_dropped() {
        let mut slack = AUDI.to_vec();
        slack.extend_from_slice(&[0u8; 300]);
        let file = Fox2File::read(&slack).expect("slack");
        assert_eq!(file.write().as_deref(), Ok(AUDI));
        let mut garbage = AUDI.to_vec();
        garbage.extend_from_slice(&[0u8; 299]);
        garbage.push(1);
        assert_eq!(Fox2File::read(&garbage), Err(Fox2Error::TrailingBytes));
    }

    #[test]
    fn malformed_inputs_error() {
        let mut bad_magic = AUDI.to_vec();
        bad_magic[0..4].copy_from_slice(&0x5A4D656Fu32.to_le_bytes());
        assert_eq!(Fox2File::read(&bad_magic), Err(Fox2Error::BadMagic));
        assert_eq!(Fox2File::read(&AUDI[..100]), Err(Fox2Error::Truncated));
        let mut bad_version = AUDI.to_vec();
        bad_version[4..8].copy_from_slice(&0x36u32.to_le_bytes());
        assert!(matches!(
            Fox2File::read(&bad_version),
            Err(Fox2Error::UnexpectedConstant { .. })
        ));
        // Entity 0 at 32, its first property at 32 + 64; data_type at +8, container at +9.
        let property = 32 + 64;
        let mut bad_type = AUDI.to_vec();
        bad_type[property + 8] = 23;
        assert_eq!(
            Fox2File::read(&bad_type),
            Err(Fox2Error::UnsupportedDataType(23))
        );
        let mut bad_container = AUDI.to_vec();
        bad_container[property + 9] = 9;
        assert_eq!(
            Fox2File::read(&bad_container),
            Err(Fox2Error::UnknownContainer(9))
        );
        let mut bad_count = AUDI.to_vec();
        bad_count[8..12].copy_from_slice(&0x7FFFFFFFi32.to_le_bytes());
        assert!(Fox2File::read(&bad_count).is_err());
    }

    #[test]
    fn empty_properties_round_trip() {
        let empty = |container| Property {
            name: FoxString::Hash(hash_string("p")),
            container,
            keys: Vec::new(),
            values: Values::Int32(Vec::new()),
        };
        let file = Fox2File {
            entities: vec![Entity {
                class_name: FoxString::Hash(hash_string("DataSet")),
                unknown1: 0,
                unknown2: 0,
                version: 0,
                address: 0x100,
                static_properties: vec![
                    empty(Container::StaticArray),
                    empty(Container::StaticArray),
                    empty(Container::StaticArray),
                ],
                dynamic_properties: vec![
                    empty(Container::DynamicArray),
                    empty(Container::DynamicArray),
                ],
            }],
            string_table: Vec::new(),
        };
        let bytes = file.write().expect("write");
        assert_eq!(Fox2File::read(&bytes).as_ref(), Ok(&file));
    }

    #[test]
    fn padding_spans_are_validated() {
        // audi entity 0, property 0 at 96: one 8-byte String value ends at 136, the
        // property is 48 bytes, so 136..144 is the property tail.
        let mut file = AUDI.to_vec();
        file[136] = 1;
        assert!(matches!(
            Fox2File::read(&file),
            Err(Fox2Error::UnexpectedConstant {
                what: "property padding",
                ..
            })
        ));
        // The StringMap entry tails: steward entity 19's `links` property sits at 12464;
        // each entry is key(8) + EntityLink(32) = 40 bytes padded to 48, so the first
        // tail is 12496 + 40 .. 12544.
        let mut steward = include_bytes!("../tests/fixtures/steward_sit_st074.fox2").to_vec();
        steward[12496 + 40] = 1;
        assert!(matches!(
            Fox2File::read(&steward),
            Err(Fox2Error::UnexpectedConstant {
                what: "entry padding",
                ..
            })
        ));
        // audi's zero-hash terminator ends at 874; padding to 880, trailer at 880,
        // padding 885..896.
        let mut before_trailer = AUDI.to_vec();
        before_trailer[874] = 1;
        assert!(matches!(
            Fox2File::read(&before_trailer),
            Err(Fox2Error::UnexpectedConstant {
                what: "trailer padding",
                ..
            })
        ));
        let mut after_trailer = AUDI.to_vec();
        after_trailer[885] = 1;
        assert!(matches!(
            Fox2File::read(&after_trailer),
            Err(Fox2Error::UnexpectedConstant {
                what: "trailer padding",
                ..
            })
        ));
    }
}
