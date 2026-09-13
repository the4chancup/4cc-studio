//! The property value model: one `Values` variant per data type word, plus the
//! `FoxString`/`Container`/`EntityLink`/`WideVector3` shapes they carry.

use crate::file::Fox2Error;
use crate::hash::hash_string;

/// A string the file stores as a hash: resolved to its text or still opaque.
#[derive(Debug, Clone, PartialEq)]
pub enum FoxString {
    /// Resolved text (from the file's table or a dictionary).
    Literal(String),
    /// The stored 48-bit hash, unresolved.
    Hash(u64),
}

impl FoxString {
    /// The hash the binary form stores: `text`'s hash for a literal, the hash itself for
    /// an unresolved one.
    pub fn hash(&self) -> u64 {
        match self {
            FoxString::Literal(text) => hash_string(text),
            FoxString::Hash(hash) => *hash,
        }
    }
}

/// The property container word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Container {
    /// 0: values back to back.
    StaticArray,
    /// 1: values back to back.
    DynamicArray,
    /// 2: `u64 key` + value per entry, padded to 16.
    StringMap,
    /// 3: values back to back.
    List,
}

impl Container {
    /// The container for `word`, or `None` for an unknown one.
    pub fn from_word(word: u8) -> Option<Container> {
        match word {
            0 => Some(Container::StaticArray),
            1 => Some(Container::DynamicArray),
            2 => Some(Container::StringMap),
            3 => Some(Container::List),
            _ => None,
        }
    }

    /// The XML name (`StaticArray`, `DynamicArray`, `StringMap`, `List`).
    pub fn name(self) -> &'static str {
        match self {
            Container::StaticArray => "StaticArray",
            Container::DynamicArray => "DynamicArray",
            Container::StringMap => "StringMap",
            Container::List => "List",
        }
    }

    /// The container an XML name gives, or `None` for an unknown one.
    pub fn from_name(name: &str) -> Option<Container> {
        match name {
            "StaticArray" => Some(Container::StaticArray),
            "DynamicArray" => Some(Container::DynamicArray),
            "StringMap" => Some(Container::StringMap),
            "List" => Some(Container::List),
            _ => None,
        }
    }

    /// The word the container serializes as.
    pub fn word(self) -> u8 {
        match self {
            Container::StaticArray => 0,
            Container::DynamicArray => 1,
            Container::StringMap => 2,
            Container::List => 3,
        }
    }
}

/// One variant per data type word; `data_type()` gives the word back.
#[derive(Debug, Clone, PartialEq)]
pub enum Values {
    /// 0.
    Int8(Vec<i8>),
    /// 1.
    Uint8(Vec<u8>),
    /// 2.
    Int16(Vec<i16>),
    /// 3.
    Uint16(Vec<u16>),
    /// 4.
    Int32(Vec<i32>),
    /// 5.
    Uint32(Vec<u32>),
    /// 6.
    Int64(Vec<i64>),
    /// 7.
    Uint64(Vec<u64>),
    /// 8.
    Float(Vec<f32>),
    /// 9.
    Double(Vec<f64>),
    /// 10.
    Bool(Vec<bool>),
    /// 11.
    String(Vec<FoxString>),
    /// 12.
    Path(Vec<FoxString>),
    /// 20.
    FilePtr(Vec<FoxString>),
    /// 13.
    EntityPtr(Vec<u64>),
    /// 21.
    EntityHandle(Vec<u64>),
    /// 14 (four `f32` like the other vectors).
    Vector3(Vec<[f32; 4]>),
    /// 15.
    Vector4(Vec<[f32; 4]>),
    /// 16.
    Quat(Vec<[f32; 4]>),
    /// 19.
    Color(Vec<[f32; 4]>),
    /// 17.
    Matrix3(Vec<[f32; 9]>),
    /// 18.
    Matrix4(Vec<[f32; 16]>),
    /// 22.
    EntityLink(Vec<EntityLink>),
    /// 24.
    WideVector3(Vec<WideVector3>),
}

/// The XML name of data type `word` (`"int8"` .. `"WideVector3"`, `"unknown"` above 24).
pub(crate) fn type_name(word: u8) -> &'static str {
    const NAMES: [&str; 25] = [
        "int8",
        "uint8",
        "int16",
        "uint16",
        "int32",
        "uint32",
        "int64",
        "uint64",
        "float",
        "double",
        "bool",
        "String",
        "Path",
        "EntityPtr",
        "Vector3",
        "Vector4",
        "Quat",
        "Matrix3",
        "Matrix4",
        "Color",
        "FilePtr",
        "EntityHandle",
        "EntityLink",
        "PropertyInfo",
        "WideVector3",
    ];
    NAMES.get(usize::from(word)).copied().unwrap_or("unknown")
}

/// The data type word an XML name gives, or `None` for an unknown one.
pub(crate) fn type_word(name: &str) -> Option<u8> {
    (0u8..=24).find(|word| type_name(*word) == name)
}

/// An EntityLink value: three string hashes, then a `u64` handle (32 bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct EntityLink {
    /// The package path.
    pub package: FoxString,
    /// The archive path.
    pub archive: FoxString,
    /// The name inside the archive.
    pub name: FoxString,
    /// The in-engine handle.
    pub handle: u64,
}

/// A WideVector3 value: three `f32` then two `u16` (16 bytes).
#[derive(Debug, Clone, PartialEq)]
pub struct WideVector3 {
    /// First coordinate.
    pub x: f32,
    /// Second coordinate.
    pub y: f32,
    /// Third coordinate.
    pub z: f32,
    /// First `u16`.
    pub a: u16,
    /// Second `u16`.
    pub b: u16,
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

fn u16_at(bytes: &[u8], at: usize) -> Result<u16, Fox2Error> {
    let mut word = [0u8; 2];
    word.copy_from_slice(slice_at(bytes, at, 2)?);
    Ok(u16::from_le_bytes(word))
}

fn u64_at(bytes: &[u8], at: usize) -> Result<u64, Fox2Error> {
    let mut word = [0u8; 8];
    word.copy_from_slice(slice_at(bytes, at, 8)?);
    Ok(u64::from_le_bytes(word))
}

fn f32s_at<const N: usize>(bytes: &[u8], at: usize) -> Result<[f32; N], Fox2Error> {
    let mut floats = [0.0f32; N];
    let (words, _) = slice_at(bytes, at, 4 * N)?.as_chunks::<4>();
    for (value, word) in floats.iter_mut().zip(words) {
        *value = f32::from_le_bytes(*word);
    }
    Ok(floats)
}

impl Values {
    /// The data type word the variant serializes as.
    pub fn data_type(&self) -> u8 {
        match self {
            Values::Int8(_) => 0,
            Values::Uint8(_) => 1,
            Values::Int16(_) => 2,
            Values::Uint16(_) => 3,
            Values::Int32(_) => 4,
            Values::Uint32(_) => 5,
            Values::Int64(_) => 6,
            Values::Uint64(_) => 7,
            Values::Float(_) => 8,
            Values::Double(_) => 9,
            Values::Bool(_) => 10,
            Values::String(_) => 11,
            Values::Path(_) => 12,
            Values::EntityPtr(_) => 13,
            Values::Vector3(_) => 14,
            Values::Vector4(_) => 15,
            Values::Quat(_) => 16,
            Values::Matrix3(_) => 17,
            Values::Matrix4(_) => 18,
            Values::Color(_) => 19,
            Values::FilePtr(_) => 20,
            Values::EntityHandle(_) => 21,
            Values::EntityLink(_) => 22,
            Values::WideVector3(_) => 24,
        }
    }

    /// The number of values held.
    pub fn len(&self) -> usize {
        match self {
            Values::Int8(list) => list.len(),
            Values::Uint8(list) => list.len(),
            Values::Int16(list) => list.len(),
            Values::Uint16(list) => list.len(),
            Values::Int32(list) => list.len(),
            Values::Uint32(list) => list.len(),
            Values::Int64(list) => list.len(),
            Values::Uint64(list) => list.len(),
            Values::Float(list) => list.len(),
            Values::Double(list) => list.len(),
            Values::Bool(list) => list.len(),
            Values::String(list) => list.len(),
            Values::Path(list) => list.len(),
            Values::FilePtr(list) => list.len(),
            Values::EntityPtr(list) => list.len(),
            Values::EntityHandle(list) => list.len(),
            Values::Vector3(list) => list.len(),
            Values::Vector4(list) => list.len(),
            Values::Quat(list) => list.len(),
            Values::Color(list) => list.len(),
            Values::Matrix3(list) => list.len(),
            Values::Matrix4(list) => list.len(),
            Values::EntityLink(list) => list.len(),
            Values::WideVector3(list) => list.len(),
        }
    }

    /// Whether no values are held.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The bytes one value of the variant occupies.
    pub(crate) fn element_size(&self) -> usize {
        match self {
            Values::Int8(_) | Values::Uint8(_) | Values::Bool(_) => 1,
            Values::Int16(_) | Values::Uint16(_) => 2,
            Values::Int32(_) | Values::Uint32(_) | Values::Float(_) => 4,
            Values::Int64(_)
            | Values::Uint64(_)
            | Values::Double(_)
            | Values::String(_)
            | Values::Path(_)
            | Values::FilePtr(_)
            | Values::EntityPtr(_)
            | Values::EntityHandle(_) => 8,
            Values::Vector3(_)
            | Values::Vector4(_)
            | Values::Quat(_)
            | Values::Color(_)
            | Values::WideVector3(_) => 16,
            Values::EntityLink(_) => 32,
            Values::Matrix3(_) => 36,
            Values::Matrix4(_) => 64,
        }
    }

    /// An empty `Values` of the type `word` names, `UnsupportedDataType` for PropertyInfo
    /// (23) or any word above 24.
    pub(crate) fn empty(word: u8) -> Result<Values, Fox2Error> {
        Ok(match word {
            0 => Values::Int8(Vec::new()),
            1 => Values::Uint8(Vec::new()),
            2 => Values::Int16(Vec::new()),
            3 => Values::Uint16(Vec::new()),
            4 => Values::Int32(Vec::new()),
            5 => Values::Uint32(Vec::new()),
            6 => Values::Int64(Vec::new()),
            7 => Values::Uint64(Vec::new()),
            8 => Values::Float(Vec::new()),
            9 => Values::Double(Vec::new()),
            10 => Values::Bool(Vec::new()),
            11 => Values::String(Vec::new()),
            12 => Values::Path(Vec::new()),
            13 => Values::EntityPtr(Vec::new()),
            14 => Values::Vector3(Vec::new()),
            15 => Values::Vector4(Vec::new()),
            16 => Values::Quat(Vec::new()),
            17 => Values::Matrix3(Vec::new()),
            18 => Values::Matrix4(Vec::new()),
            19 => Values::Color(Vec::new()),
            20 => Values::FilePtr(Vec::new()),
            21 => Values::EntityHandle(Vec::new()),
            22 => Values::EntityLink(Vec::new()),
            24 => Values::WideVector3(Vec::new()),
            _ => return Err(Fox2Error::UnsupportedDataType(word)),
        })
    }
}

/// Appends the value of `values`' type read at `at`; returns the bytes consumed.
pub(crate) fn read_value(values: &mut Values, bytes: &[u8], at: usize) -> Result<usize, Fox2Error> {
    match values {
        Values::Int8(list) => {
            list.push(slice_at(bytes, at, 1)?[0] as i8);
            Ok(1)
        }
        Values::Uint8(list) => {
            list.push(slice_at(bytes, at, 1)?[0]);
            Ok(1)
        }
        Values::Int16(list) => {
            list.push(i16::from_le_bytes(
                slice_at(bytes, at, 2)?
                    .try_into()
                    .map_err(|_| Fox2Error::Truncated)?,
            ));
            Ok(2)
        }
        Values::Uint16(list) => {
            list.push(u16_at(bytes, at)?);
            Ok(2)
        }
        Values::Int32(list) => {
            list.push(i32::from_le_bytes(
                slice_at(bytes, at, 4)?
                    .try_into()
                    .map_err(|_| Fox2Error::Truncated)?,
            ));
            Ok(4)
        }
        Values::Uint32(list) => {
            list.push(u32::from_le_bytes(
                slice_at(bytes, at, 4)?
                    .try_into()
                    .map_err(|_| Fox2Error::Truncated)?,
            ));
            Ok(4)
        }
        Values::Int64(list) => {
            list.push(i64::from_le_bytes(
                slice_at(bytes, at, 8)?
                    .try_into()
                    .map_err(|_| Fox2Error::Truncated)?,
            ));
            Ok(8)
        }
        Values::Uint64(list) | Values::EntityPtr(list) | Values::EntityHandle(list) => {
            list.push(u64_at(bytes, at)?);
            Ok(8)
        }
        Values::Float(list) => {
            list.push(f32::from_le_bytes(
                slice_at(bytes, at, 4)?
                    .try_into()
                    .map_err(|_| Fox2Error::Truncated)?,
            ));
            Ok(4)
        }
        Values::Double(list) => {
            list.push(f64::from_le_bytes(
                slice_at(bytes, at, 8)?
                    .try_into()
                    .map_err(|_| Fox2Error::Truncated)?,
            ));
            Ok(8)
        }
        Values::Bool(list) => {
            list.push(slice_at(bytes, at, 1)?[0] != 0);
            Ok(1)
        }
        Values::String(list) | Values::Path(list) | Values::FilePtr(list) => {
            list.push(FoxString::Hash(u64_at(bytes, at)?));
            Ok(8)
        }
        Values::Vector3(list)
        | Values::Vector4(list)
        | Values::Quat(list)
        | Values::Color(list) => {
            list.push(f32s_at::<4>(bytes, at)?);
            Ok(16)
        }
        Values::Matrix3(list) => {
            list.push(f32s_at::<9>(bytes, at)?);
            Ok(36)
        }
        Values::Matrix4(list) => {
            list.push(f32s_at::<16>(bytes, at)?);
            Ok(64)
        }
        Values::EntityLink(list) => {
            list.push(EntityLink {
                package: FoxString::Hash(u64_at(bytes, at)?),
                archive: FoxString::Hash(u64_at(bytes, at + 8)?),
                name: FoxString::Hash(u64_at(bytes, at + 16)?),
                handle: u64_at(bytes, at + 24)?,
            });
            Ok(32)
        }
        Values::WideVector3(list) => {
            let [x, y, z] = f32s_at::<3>(bytes, at)?;
            list.push(WideVector3 {
                x,
                y,
                z,
                a: u16_at(bytes, at + 12)?,
                b: u16_at(bytes, at + 14)?,
            });
            Ok(16)
        }
    }
}

/// Writes value `index` of `values` to `out` (index is in range by construction).
pub(crate) fn write_values(values: &Values, index: usize, out: &mut Vec<u8>) {
    match values {
        Values::Int8(list) => out.push(list[index] as u8),
        Values::Uint8(list) => out.push(list[index]),
        Values::Int16(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Uint16(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Int32(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Uint32(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Int64(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Uint64(list) | Values::EntityPtr(list) | Values::EntityHandle(list) => {
            out.extend_from_slice(&list[index].to_le_bytes());
        }
        Values::Float(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Double(list) => out.extend_from_slice(&list[index].to_le_bytes()),
        Values::Bool(list) => out.push(u8::from(list[index])),
        Values::String(list) | Values::Path(list) | Values::FilePtr(list) => {
            out.extend_from_slice(&list[index].hash().to_le_bytes());
        }
        Values::Vector3(list)
        | Values::Vector4(list)
        | Values::Quat(list)
        | Values::Color(list) => {
            for value in list[index] {
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        Values::Matrix3(list) => {
            for value in list[index] {
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        Values::Matrix4(list) => {
            for value in list[index] {
                out.extend_from_slice(&value.to_le_bytes());
            }
        }
        Values::EntityLink(list) => {
            let link = &list[index];
            out.extend_from_slice(&link.package.hash().to_le_bytes());
            out.extend_from_slice(&link.archive.hash().to_le_bytes());
            out.extend_from_slice(&link.name.hash().to_le_bytes());
            out.extend_from_slice(&link.handle.to_le_bytes());
        }
        Values::WideVector3(list) => {
            let value = &list[index];
            out.extend_from_slice(&value.x.to_le_bytes());
            out.extend_from_slice(&value.y.to_le_bytes());
            out.extend_from_slice(&value.z.to_le_bytes());
            out.extend_from_slice(&value.a.to_le_bytes());
            out.extend_from_slice(&value.b.to_le_bytes());
        }
    }
}
