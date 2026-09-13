//! Fox Engine FPK/FPKD package read and write.
//!
//! Layout (little-endian): a 48-byte `foxfpk` header, one 48-byte record per
//! entry (content offset/length, name offset/length, MD5 of the name), a
//! NUL-terminated name pool padded to 16, then the contents each padded to
//! 16. FPKD is the same container with a `d` kind byte. The writer sorts
//! entries by name, matching pes-file-tools byte for byte.

use std::collections::BTreeMap;
use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use md5::{Digest, Md5};

/// Which container variant the file is: plain FPK or FPKD (`d` kind byte).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpkKind {
    /// FPK (`kind` byte 0).
    Fpk,
    /// FPKD (`kind` byte `d`), the packed variant PES ships.
    Fpkd,
}

/// An FPK/FPKD package: file contents keyed by their stored `/Assets/...`
/// name, in name order.
#[derive(Debug)]
pub struct FpkFile {
    kind: FpkKind,
    entries: BTreeMap<String, Vec<u8>>,
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
struct Header {
    magic: [u8; 6],
    kind: u8,
    platform: [u8; 3],
    file_size: u32,
    #[brw(pad_before = 18)]
    unknown1: u32,
    file_count: u32,
    reference_count: u32,
    unknown2: u32,
}

#[derive(BinRead, BinWrite)]
#[brw(little)]
struct EntryRecord {
    content_offset: u64,
    content_length: u64,
    name_offset: u64,
    name_length: u64,
    checksum: [u8; 16],
}

/// Why a byte buffer is not a readable FPK.
#[derive(Debug, thiserror::Error)]
pub enum FpkError {
    /// The buffer ends before a structure that points past it.
    #[error("fpk is truncated")]
    Truncated,
    /// Wrong `foxfpk`/`win` magic.
    #[error("invalid fpk magic")]
    BadMagic,
    /// A header field carries a value this variant of the format never uses
    /// (names the field).
    #[error("unsupported fpk field: {0}")]
    Unsupported(&'static str),
    /// An entry's name or content range points outside the file.
    #[error("entry points outside the file: {name}")]
    OutOfBounds {
        /// The entry whose ranges are out of bounds.
        name: String,
    },
    /// Two entries share a name.
    #[error("duplicate entry: {0}")]
    DuplicateEntry(String),
    /// The stored MD5 does not match the entry's name bytes.
    #[error("name checksum mismatch: {0}")]
    Checksum(String),
    /// An entry name is not valid UTF-8.
    #[error("entry name is not utf-8: {0}")]
    Utf8(String),
}

impl FpkFile {
    /// An empty package of the given kind.
    pub fn new(kind: FpkKind) -> Self {
        FpkFile {
            kind,
            entries: BTreeMap::new(),
        }
    }

    /// Parses an FPK/FPKD buffer: validates the magic, the fixed header
    /// fields, every entry's bounds, name UTF-8 and MD5, and rejects
    /// duplicates.
    pub fn read(bytes: &[u8]) -> Result<FpkFile, FpkError> {
        if bytes.len() < 48 {
            return Err(FpkError::Truncated);
        }
        let mut cursor = Cursor::new(bytes);
        let header = Header::read(&mut cursor).map_err(|_| FpkError::Truncated)?;
        if header.magic != *b"foxfpk" || header.platform != *b"win" {
            return Err(FpkError::BadMagic);
        }
        let kind = match header.kind {
            0 => FpkKind::Fpk,
            b'd' => FpkKind::Fpkd,
            _ => return Err(FpkError::Unsupported("kind")),
        };
        if header.unknown1 != 2 {
            return Err(FpkError::Unsupported("unknown1"));
        }
        if header.reference_count != 0 {
            return Err(FpkError::Unsupported("reference_count"));
        }
        if header.unknown2 != 0 {
            return Err(FpkError::Unsupported("unknown2"));
        }

        let mut entries = BTreeMap::new();
        for _ in 0..header.file_count {
            let record = EntryRecord::read(&mut cursor).map_err(|_| FpkError::Truncated)?;
            let name_end = record
                .name_offset
                .checked_add(record.name_length)
                .ok_or(FpkError::Truncated)?;
            let name_bytes = bytes
                .get(record.name_offset as usize..name_end as usize)
                .ok_or(FpkError::Truncated)?;
            let name = String::from_utf8(name_bytes.to_vec())
                .map_err(|_| FpkError::Utf8(String::from_utf8_lossy(name_bytes).into_owned()))?;
            let digest: [u8; 16] = Md5::digest(name_bytes).into();
            if digest != record.checksum {
                return Err(FpkError::Checksum(name));
            }
            let content_end = record
                .content_offset
                .checked_add(record.content_length)
                .ok_or(FpkError::Truncated)?;
            let content = bytes
                .get(record.content_offset as usize..content_end as usize)
                .ok_or(FpkError::OutOfBounds {
                    name: name.clone(),
                })?;
            if entries.insert(name.clone(), content.to_vec()).is_some() {
                return Err(FpkError::DuplicateEntry(name));
            }
        }

        Ok(FpkFile { kind, entries })
    }

    /// Serializes the package, byte-identical to pes-file-tools' `write` for
    /// the same entries: name order, name pool and contents padded to 16.
    pub fn write(&self) -> Vec<u8> {
        let mut name_pool = Vec::new();
        let mut content_pool = Vec::new();
        let mut records = Vec::with_capacity(self.entries.len());
        for (name, content) in &self.entries {
            let name_bytes = name.as_bytes();
            let name_offset = name_pool.len() as u64;
            name_pool.extend_from_slice(name_bytes);
            name_pool.push(0);

            let content_offset = content_pool.len() as u64;
            content_pool.extend_from_slice(content);
            content_pool.resize(content_pool.len() + pad16(content_pool.len()), 0);

            records.push((
                content_offset,
                content.len() as u64,
                name_offset,
                name_bytes.len() as u64,
                Md5::digest(name_bytes),
            ));
        }
        name_pool.resize(name_pool.len() + pad16(name_pool.len()), 0);

        let name_pool_offset = 48u64 + 48 * self.entries.len() as u64;
        let content_pool_offset = name_pool_offset + name_pool.len() as u64;

        let header = Header {
            magic: *b"foxfpk",
            kind: match self.kind {
                FpkKind::Fpk => 0,
                FpkKind::Fpkd => b'd',
            },
            platform: *b"win",
            file_size: (content_pool_offset + content_pool.len() as u64) as u32,
            unknown1: 2,
            file_count: self.entries.len() as u32,
            reference_count: 0,
            unknown2: 0,
        };

        let mut output = Vec::new();
        header
            .write(&mut output)
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
        for (content_offset, content_length, name_offset, name_length, checksum) in records {
            EntryRecord {
                content_offset: content_offset + content_pool_offset,
                content_length,
                name_offset: name_offset + name_pool_offset,
                name_length,
                checksum: checksum.into(),
            }
            .write(&mut output)
            .unwrap_or_else(|_| unreachable!("Vec write is infallible"));
        }
        output.extend_from_slice(&name_pool);
        output.extend_from_slice(&content_pool);
        output
    }

    /// FPK vs FPKD.
    pub fn kind(&self) -> FpkKind {
        self.kind
    }

    /// Number of stored files.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the package stores no files.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The entries in name order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &[u8])> {
        self.entries
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_slice()))
    }

    /// The content stored under `name`.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.entries.get(name).map(Vec::as_slice)
    }

    /// Inserts or replaces `name`, returning the replaced content if any.
    pub fn insert(&mut self, name: String, content: Vec<u8>) -> Option<Vec<u8>> {
        self.entries.insert(name, content)
    }

    /// Removes `name`, returning its content if present.
    pub fn remove(&mut self, name: &str) -> Option<Vec<u8>> {
        self.entries.remove(name)
    }
}

/// Bytes needed to bring `len` up to the next multiple of 16.
fn pad16(len: usize) -> usize {
    (16 - len % 16) % 16
}

#[cfg(test)]
mod tests {
    use super::*;

    const COACHING: &[u8] = include_bytes!("../tests/fixtures/konami_d2_coaching_target.fpk");
    const LOW_PARTS: &[u8] = include_bytes!("../tests/fixtures/konami_audiLowParts_model.fpk");
    const SEAT: &[u8] = include_bytes!("../tests/fixtures/konami_audi_seat_model.fpkd");
    const TEMPLATE: &[u8] = include_bytes!("../tests/fixtures/red_template_generic.fpkd");
    const SAMPLE_FPK: &[u8] = include_bytes!("../tests/fixtures/pft_sample.fpk");
    const SAMPLE_FPKD: &[u8] = include_bytes!("../tests/fixtures/pft_sample.fpkd");

    const FACE_BSM: &str =
        "/Assets/pes16/model/character/face/real/70101/sourceimages/#windx11/face_bsm.ftex";
    const FACE_HIGH: &str = "/Assets/pes16/model/character/face/real/70101/face_high.fmdl";
    const FACE: &str = "/Assets/pes16/model/character/face/real/70101/face.fmdl";

    fn sample_entries() -> [(String, Vec<u8>); 3] {
        [
            (FACE_BSM.to_owned(), vec![b'B'; 33]),
            (FACE_HIGH.to_owned(), vec![b'A'; 100]),
            (FACE.to_owned(), (0u8..=16).collect()),
        ]
    }

    #[test]
    fn konami_fpk_fixture_reads() {
        let fpk = FpkFile::read(COACHING).unwrap();
        assert_eq!(fpk.kind(), FpkKind::Fpk);
        assert_eq!(fpk.len(), 1);
        let (name, content) = fpk.entries().next().unwrap();
        assert_eq!(
            name,
            "/Assets/pes16/model/game2d/d2_coaching_target/scenes/d2_couaching_target.fmdl"
        );
        assert_eq!(content.len(), 1125);
    }

    #[test]
    fn konami_two_entry_fixture_yields_name_order() {
        let fpk = FpkFile::read(LOW_PARTS).unwrap();
        assert_eq!(fpk.kind(), FpkKind::Fpk);
        let entries: Vec<(&str, usize)> = fpk
            .entries()
            .map(|(name, content)| (name, content.len()))
            .collect();
        assert_eq!(
            entries,
            [
                (
                    "/Assets/pes16/model/bg/common/audi/scenes/au00.skl",
                    1088
                ),
                (
                    "/Assets/pes16/model/bg/common/audi/scenes/au_Low_parts.fmdl",
                    10567
                ),
            ]
        );
    }

    #[test]
    fn konami_fpkd_fixture_reads() {
        let fpk = FpkFile::read(SEAT).unwrap();
        assert_eq!(fpk.kind(), FpkKind::Fpkd);
        assert_eq!(fpk.len(), 1);
        let (name, content) = fpk.entries().next().unwrap();
        assert_eq!(
            name,
            "/Assets/pes16/model/bg/common/audi/seatOnly_model.fox2"
        );
        assert_eq!(content.len(), 896);
    }

    #[test]
    fn empty_fpkd_writes_red_template_bytes() {
        let fpk = FpkFile::read(TEMPLATE).unwrap();
        assert_eq!(fpk.kind(), FpkKind::Fpkd);
        assert!(fpk.is_empty());
        assert_eq!(FpkFile::new(FpkKind::Fpkd).write(), TEMPLATE);
    }

    #[test]
    fn pft_sample_reads_and_rewrites_identically() {
        let fpk = FpkFile::read(SAMPLE_FPK).unwrap();
        assert_eq!(fpk.len(), 3);
        for (name, expected) in &sample_entries() {
            assert_eq!(fpk.get(name).unwrap(), expected.as_slice());
        }
        // Insert in a different order: the writer sorts by name anyway.
        let mut rebuilt = FpkFile::new(FpkKind::Fpk);
        for (name, content) in [sample_entries()[2].clone(), sample_entries()[0].clone(), sample_entries()[1].clone()] {
            rebuilt.insert(name, content);
        }
        assert_eq!(rebuilt.write(), SAMPLE_FPK);
        let mut rebuilt_d = FpkFile::new(FpkKind::Fpkd);
        for (name, content) in sample_entries() {
            rebuilt_d.insert(name, content);
        }
        assert_eq!(rebuilt_d.write(), SAMPLE_FPKD);
    }

    #[test]
    fn konami_fixtures_round_trip() {
        for bytes in [COACHING, LOW_PARTS, SEAT] {
            let fpk = FpkFile::read(bytes).unwrap();
            let written = fpk.write();
            let reread = FpkFile::read(&written).unwrap();
            let before: Vec<(&str, &[u8])> = fpk.entries().collect();
            let after: Vec<(&str, &[u8])> = reread.entries().collect();
            assert_eq!(before, after);
            assert_eq!(fpk.kind(), reread.kind());
        }
    }

    #[test]
    fn tampering_and_bad_headers_error() {
        let mut corrupted = SAMPLE_FPK.to_vec();
        // First entry record starts at 48; checksum occupies its last 16 bytes.
        corrupted[48 + 32] ^= 0xff;
        assert!(matches!(
            FpkFile::read(&corrupted),
            Err(FpkError::Checksum(_))
        ));
        assert!(matches!(
            FpkFile::read(&SAMPLE_FPK[..20]),
            Err(FpkError::Truncated)
        ));
        let mut bad_kind = SAMPLE_FPK.to_vec();
        bad_kind[6] = b'x';
        assert!(matches!(
            FpkFile::read(&bad_kind),
            Err(FpkError::Unsupported("kind"))
        ));
    }
}
