//! FPK / FPKD archive format (Fox Engine package).
//!
//! Rust translation of `fpk.py` from the Python compilers, written as a
//! style example for the 4cc Studio rewrite. It demonstrates the idioms the
//! final product will use throughout:
//!
//! - `binrw` derive macros for fixed binary layouts (replaces `struct.unpack`)
//! - `thiserror` for typed, descriptive errors (replaces `DecodeError` strings)
//! - `BTreeMap` for deterministic sorted output (replaces `sorted(dict.keys())`)
//! - Slice-based zero-copy reads with bounds checking via `.get()`
//! - A lossless roundtrip test at the bottom
//!
//! Format layout (all little-endian):
//!
//! ```text
//! offset  size  field
//! 0       6     magic "foxfpk"
//! 6       1     fpk type (b'd' for .fpkd, 0 for .fpk)
//! 7       3     magic "win"
//! 10      4     total file size
//! 14      18    padding
//! 32      4     unknown1 (always 2)
//! 36      4     file count
//! 40      4     reference count (always 0 in PES files)
//! 44      4     unknown2 (always 0)
//! 48      48*N  file entries
//! ...           filename table (null-terminated, 16-byte aligned)
//! ...           file contents (each 16-byte aligned)
//! ```

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;

use binrw::{binrw, BinRead, BinWrite};
use md5::{Digest, Md5};

/// Errors that can occur while decoding or encoding an FPK file.
#[derive(Debug, thiserror::Error)]
pub enum FpkError {
    #[error("invalid FPK: {0}")]
    Invalid(&'static str),
    #[error("unsupported FPK: {0}")]
    Unsupported(&'static str),
    #[error("duplicate entry for filename '{0}'")]
    DuplicateEntry(String),
    #[error("incorrect checksum for filename '{0}'")]
    BadChecksum(String),
    #[error("entry filename is not valid UTF-8")]
    BadFilename(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Binary(#[from] binrw::Error),
}

/// The fixed 48-byte FPK header.
///
/// The struct-level `magic` consumes and verifies the leading b"foxfpk";
/// the field-level assert verifies the second magic. Field sizes and order
/// are checked by the compiler against the declared types — a wrong field
/// type or a missing field is a compile error, not a silent corruption.
#[binrw]
#[brw(little, magic = b"foxfpk")]
struct FpkHeader {
    /// b'd' for .fpkd files, 0 for plain .fpk files.
    fpk_type: u8,
    #[br(assert(&magic2 == b"win", "invalid FPK: bad second magic"))]
    magic2: [u8; 3],
    file_size: u32,
    #[brw(pad_before = 18)]
    unknown1: u32,
    file_count: u32,
    reference_count: u32,
    unknown2: u32,
}

/// One 48-byte file entry in the entry table.
#[binrw]
#[brw(little)]
struct FpkEntry {
    content_offset: u64,
    content_length: u64,
    filename_offset: u64,
    filename_length: u64,
    checksum: [u8; 16],
}

/// An FPK archive: a mapping from filenames to file contents.
///
/// `BTreeMap` iterates in sorted key order, which makes `write` deterministic
/// and byte-identical across runs — the Python original achieved the same
/// with `sorted(self.entries.keys())`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct FpkFile {
    pub entries: BTreeMap<String, Vec<u8>>,
}

impl FpkFile {
    /// Parse an FPK archive from a byte buffer.
    pub fn read(data: &[u8]) -> Result<FpkFile, FpkError> {
        let mut cursor = Cursor::new(data);
        let header = FpkHeader::read(&mut cursor)?;

        if header.unknown1 != 2 {
            return Err(FpkError::Unsupported("unknown1 != 2"));
        }
        if header.unknown2 != 0 {
            return Err(FpkError::Unsupported("unknown2 != 0"));
        }
        if header.reference_count != 0 {
            return Err(FpkError::Unsupported("reference count != 0"));
        }

        let mut entries = BTreeMap::new();
        for _ in 0..header.file_count {
            let entry = FpkEntry::read(&mut cursor)?;

            // `.get(range)` returns None instead of panicking when the range
            // is out of bounds, replacing the Python explicit length checks.
            let filename_bytes = data
                .get(entry.filename_offset as usize
                    ..(entry.filename_offset + entry.filename_length) as usize)
                .ok_or(FpkError::Invalid("unexpected end of file"))?;
            let content = data
                .get(entry.content_offset as usize
                    ..(entry.content_offset + entry.content_length) as usize)
                .ok_or(FpkError::Invalid("unexpected end of file"))?;

            let filename = String::from_utf8(filename_bytes.to_vec())?;

            let digest: [u8; 16] = Md5::digest(filename_bytes).into();
            if digest != entry.checksum {
                return Err(FpkError::BadChecksum(filename));
            }

            if entries.insert(filename.clone(), content.to_vec()).is_some() {
                return Err(FpkError::DuplicateEntry(filename));
            }
        }

        Ok(FpkFile { entries })
    }

    /// Read an FPK archive from a file on disk.
    pub fn read_file(path: impl AsRef<Path>) -> Result<FpkFile, FpkError> {
        let data = std::fs::read(path)?;
        FpkFile::read(&data)
    }

    /// Serialize the archive to bytes.
    ///
    /// Layout: header, entry table, filename table, content — with the
    /// filename table and each content block padded to 16-byte alignment,
    /// matching the Python writer byte-for-byte.
    pub fn write(&self, is_fpkd: bool) -> Result<Vec<u8>, FpkError> {
        let mut filename_buffer = Vec::new();
        let mut content_buffer = Vec::new();
        let mut entries = Vec::with_capacity(self.entries.len());

        // BTreeMap iterates in sorted filename order.
        for (filename, content) in &self.entries {
            let filename_offset = filename_buffer.len() as u64;
            filename_buffer.extend_from_slice(filename.as_bytes());
            filename_buffer.push(0);

            let content_offset = content_buffer.len() as u64;
            content_buffer.extend_from_slice(content);
            pad_to(&mut content_buffer, 16);

            entries.push(FpkEntry {
                content_offset,
                content_length: content.len() as u64,
                filename_offset,
                filename_length: filename.len() as u64,
                checksum: Md5::digest(filename.as_bytes()).into(),
            });
        }
        pad_to(&mut filename_buffer, 16);

        let entry_table_offset = 48u64;
        let filename_table_offset = entry_table_offset + 48 * entries.len() as u64;
        let content_offset_base = filename_table_offset + filename_buffer.len() as u64;

        let header = FpkHeader {
            fpk_type: if is_fpkd { b'd' } else { 0 },
            magic2: *b"win",
            file_size: (content_offset_base + content_buffer.len() as u64) as u32,
            unknown1: 2,
            file_count: entries.len() as u32,
            reference_count: 0,
            unknown2: 0,
        };

        let mut output = Cursor::new(Vec::new());
        header.write(&mut output)?;
        for entry in &entries {
            FpkEntry {
                content_offset: entry.content_offset + content_offset_base,
                filename_offset: entry.filename_offset + filename_table_offset,
                ..*entry
            }
            .write(&mut output)?;
        }

        let mut bytes = output.into_inner();
        bytes.extend_from_slice(&filename_buffer);
        bytes.extend_from_slice(&content_buffer);
        Ok(bytes)
    }

    /// Write the archive to a file on disk.
    ///
    /// Whether to write an FPKD is inferred from the file extension,
    /// like the Python original.
    pub fn write_file(&self, path: impl AsRef<Path>) -> Result<(), FpkError> {
        let is_fpkd = path
            .as_ref()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("fpkd"));
        std::fs::write(&path, self.write(is_fpkd)?)?;
        Ok(())
    }
}

/// Pad a buffer with zeroes up to the next multiple of `alignment`.
fn pad_to(buffer: &mut Vec<u8>, alignment: usize) {
    let remainder = buffer.len() % alignment;
    if remainder != 0 {
        buffer.resize(buffer.len() + alignment - remainder, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_is_lossless() {
        let mut fpk = FpkFile::default();
        fpk.entries.insert("boots.fmdl".into(), vec![1, 2, 3, 4, 5]);
        fpk.entries.insert("boots.skl".into(), vec![9; 40]);
        fpk.entries.insert("fcl_hair.fclo".into(), Vec::new());

        let bytes = fpk.write(false).expect("serialization failed");
        let reparsed = FpkFile::read(&bytes).expect("parse failed");

        assert_eq!(fpk, reparsed);
    }

    #[test]
    fn serialization_is_deterministic() {
        let mut fpk = FpkFile::default();
        fpk.entries.insert("b.bin".into(), vec![2]);
        fpk.entries.insert("a.bin".into(), vec![1]);

        assert_eq!(fpk.write(false).unwrap(), fpk.write(false).unwrap());
    }

    #[test]
    fn rejects_wrong_magic() {
        let result = FpkFile::read(b"notanfpk_______________________________________");
        assert!(matches!(result, Err(FpkError::Binary(_))));
    }
}
