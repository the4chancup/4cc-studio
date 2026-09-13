//! CRI CPK archive read and write, with CRILAYLA decompression.
//!
//! A CPK file is a `CPK ` header @UTF table (version fields, offsets to the
//! other tables), the file contents, a `TOC ` table of file entries, and
//! optionally an `ETOC` table of modification times. All @UTF content is
//! XOR-encrypted with the CRI keystream. The writer's layout is the one the
//! 4cc compilers have always produced, so an archive written from the same
//! entries is byte-identical to theirs; `CpkWriter::new` takes the `Tvers`
//! tool-version string so that parity is testable.

pub mod crilayla;
mod read;
mod utf;
mod write;

pub use read::{CpkArchive, CpkEntry, CpkTimestamp};
pub use utf::{UtfColumn, UtfKind, UtfStorage, UtfTable, UtfValue};
pub use write::CpkWriter;

/// Why a CPK read or write failed.
#[derive(Debug, thiserror::Error)]
pub enum CpkError {
    /// The underlying reader or writer failed.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// An outer table header carried a different tag than expected
    /// (`CPK `, `TOC `, `ETOC`).
    #[error("unexpected table tag: expected {expected}, found {found}")]
    UnexpectedTag {
        /// The four-byte tag the code was looking for.
        expected: String,
        /// The tag actually found.
        found: String,
    },
    /// The decrypted content does not start with `@UTF`.
    #[error("missing @UTF magic")]
    BadMagic,
    /// The @UTF body length does not match the outer header's length.
    #[error("@UTF body length does not match the outer header")]
    LengthMismatch,
    /// A column type nibble outside the known set (0,2,4,6,8,10,11).
    #[error("unsupported utf column type nibble {0}")]
    UnsupportedUtfType(u8),
    /// A column storage nibble outside the known set (1,3,5).
    #[error("unsupported utf column storage nibble {0}")]
    UnsupportedUtfStorage(u8),
    /// A required header field or TOC column is absent or holds no usable
    /// value.
    #[error("missing required utf column {0}")]
    MissingColumn(&'static str),
    /// A CRILAYLA buffer is malformed (bad magic, too short, or the bitstream
    /// ran out before the output was full).
    #[error("crilayla: {0}")]
    Crilayla(&'static str),
    /// The same archive path was added twice.
    #[error("duplicate path in archive: {0}")]
    DuplicatePath(String),
    /// The archive ends before a structure that points past it.
    #[error("archive is truncated")]
    Truncated,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    const PES17: &[u8] = include_bytes!("../tests/fixtures/konami_pes17_dt42_win.cpk");
    const PES21: &[u8] = include_bytes!("../tests/fixtures/konami_pes21_dt42_all.cpk");
    const RED: &[u8] = include_bytes!("../tests/fixtures/red_pes16_face_73113.cpk");
    const PLACEHOLDER: &[u8] = include_bytes!("../tests/fixtures/cpkmc136_placeholder.cpk");

    fn open(bytes: &[u8]) -> CpkArchive<Cursor<&[u8]>> {
        CpkArchive::open(Cursor::new(bytes)).unwrap()
    }

    #[test]
    fn konami_pes17_header_and_entries() {
        let mut archive = open(PES17);
        assert_eq!(archive.header().columns.len(), 35);
        assert_eq!(
            archive.header().get(0, "Tvers"),
            Some(&UtfValue::String("CPKMC2.21.10, DLL3.00.00".into()))
        );
        let entries = archive.entries().to_vec();
        assert_eq!(entries.len(), 5);
        let first = &entries[0];
        assert_eq!(first.path, "common/sound/Announce/sa_config.bin");
        assert_eq!(first.size, 11279);
        assert_eq!(first.packed_size, 11279);
        assert_eq!(first.offset, 4096);
        assert_eq!(first.modified, None);
        let content = archive.read(first).unwrap();
        assert_eq!(content.len(), 11279);
        // WESYS-wrapped: this file carries the 00 10 11 prefix variant.
        assert_eq!(&content[3..8], b"WESYS");
    }

    #[test]
    fn konami_pes21_header_and_entries() {
        let archive = open(PES21);
        assert_eq!(archive.header().columns.len(), 44);
        let paths: Vec<&str> = archive.entries().iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths.len(), 5);
        assert_eq!(paths[0], "common/sound/config/Announce/sa_config.bin");
        let offsets: Vec<u64> = archive.entries().iter().map(|e| e.offset).collect();
        assert_eq!(offsets, [4096, 18432, 26624, 34816, 36864]);
    }

    #[test]
    fn red_fixture_entries_carry_etoc_timestamps() {
        let archive = open(RED);
        let expected = CpkTimestamp {
            year: 2026,
            month: 2,
            day: 8,
            hour: 23,
            minute: 52,
            second: 11,
        };
        assert_eq!(archive.entries().len(), 3);
        for entry in archive.entries() {
            assert_eq!(entry.modified, Some(expected));
            assert!(
                entry
                    .path
                    .starts_with("common/character0/model/character/face/real/73113/")
            );
        }
        let sizes: Vec<u32> = archive.entries().iter().map(|e| e.size).collect();
        assert_eq!(sizes, [29, 1448, 320]);
    }

    #[test]
    fn placeholder_entry_keeps_leading_slash() {
        let archive = open(PLACEHOLDER);
        assert_eq!(archive.header().columns.len(), 24);
        assert_eq!(archive.entries().len(), 1);
        assert_eq!(archive.entries()[0].path, "/placeholder");
        assert_eq!(archive.entries()[0].size, 11);
    }

    #[test]
    fn writer_output_is_byte_identical_to_pes_file_tools() {
        let mut archive = open(RED);
        let entries = archive.entries().to_vec();
        // Add in on-disk (offset) order: that is the order the original writer
        // appended the content, and byte parity needs the same layout.
        let mut by_offset = entries.clone();
        by_offset.sort_by_key(|e| e.offset);
        // The Tvers string the fixture's writer recorded; parity needs the same one.
        let mut writer = CpkWriter::new(Cursor::new(Vec::new()), "pes-file-tools").unwrap();
        for entry in &by_offset {
            let content = archive.read(entry).unwrap();
            writer.add(&entry.path, &content, entry.modified).unwrap();
        }
        let produced = writer.finish().unwrap().into_inner();
        assert_eq!(produced.len(), RED.len());
        assert_eq!(produced, RED);
    }

    #[test]
    fn written_archive_round_trips() {
        let stamp = CpkTimestamp {
            year: 2026,
            month: 11,
            day: 2,
            hour: 20,
            minute: 15,
            second: 30,
        };
        let big = vec![0xABu8; 5000];
        let mut writer = CpkWriter::new(Cursor::new(Vec::new()), "studio-test").unwrap();
        writer.add("top.txt", b"hello", Some(stamp)).unwrap();
        writer.add("a/b/deep.bin", &[1, 2, 3], Some(stamp)).unwrap();
        writer.add("big.bin", &big, Some(stamp)).unwrap();
        assert!(matches!(
            writer.add("top.txt", b"again", Some(stamp)),
            Err(CpkError::DuplicatePath(_))
        ));
        let bytes = writer.finish().unwrap().into_inner();

        let mut archive = CpkArchive::open(Cursor::new(&bytes[..])).unwrap();
        let entries = archive.entries().to_vec();
        assert_eq!(entries.len(), 3);
        for entry in &entries {
            assert_eq!(entry.modified, Some(stamp));
        }
        assert_eq!(archive.read(&entries[0]).unwrap(), [1, 2, 3]);
        assert_eq!(archive.read(&entries[1]).unwrap(), big);
        assert_eq!(archive.read(&entries[2]).unwrap(), b"hello");
        let paths: Vec<&str> = entries.iter().map(|e| e.path.as_str()).collect();
        // Top-level files get an empty DirName, which reads back with a
        // leading slash, same as the placeholder fixture.
        assert_eq!(paths, ["a/b/deep.bin", "/big.bin", "/top.txt"]);
    }

    #[test]
    fn writer_without_timestamps_emits_no_etoc() {
        let mut writer = CpkWriter::new(Cursor::new(Vec::new()), "studio-test").unwrap();
        writer.add("a.txt", b"x", None).unwrap();
        let bytes = writer.finish().unwrap().into_inner();
        let archive = CpkArchive::open(Cursor::new(&bytes[..])).unwrap();
        assert_eq!(archive.header().get(0, "EtocOffset"), Some(&UtfValue::Null));
        assert_eq!(archive.entries()[0].modified, None);
    }
}
