//! Reading a CPK archive: the `CPK ` header table, the `TOC ` entry table, and
//! the optional `ETOC` timestamp table.

use std::io::{Read, Seek, SeekFrom};

use crate::CpkError;
use crate::crilayla;
use crate::utf::{UtfTable, UtfValue};

/// One file stored in the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpkEntry {
    /// `DirName/FileName` with backslashes normalized to `/`. An empty
    /// `DirName` yields a leading `/` (e.g. `/placeholder`), matching the
    /// legacy reader.
    pub path: String,
    /// Uncompressed size (`ExtractSize`).
    pub size: u32,
    /// Stored size (`FileSize`); smaller than `size` when CRILAYLA-packed.
    pub packed_size: u32,
    /// Absolute offset in the archive: `FileOffset` plus the CRI base, which
    /// is `min(ContentOffset, TocOffset)` — 0x800 in every PES file on hand
    /// (Blue hardcodes 0x800 for the same reason).
    pub offset: u64,
    /// The ETOC timestamp, when the archive carries one for this entry.
    pub modified: Option<CpkTimestamp>,
}

/// A file modification time from an `ETOC` `UpdateDateTime` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpkTimestamp {
    /// Year, e.g. 2026.
    pub year: u16,
    /// Month, 1-12.
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Hour.
    pub hour: u8,
    /// Minute.
    pub minute: u8,
    /// Second.
    pub second: u8,
}

impl CpkTimestamp {
    /// Unpacks the `UpdateDateTime` bitfield
    /// (`year<<48 | month<<40 | day<<32 | hour<<24 | minute<<16 | second<<8`).
    pub fn from_packed(value: u64) -> Self {
        CpkTimestamp {
            year: (value >> 48) as u16,
            month: (value >> 40) as u8,
            day: (value >> 32) as u8,
            hour: (value >> 24) as u8,
            minute: (value >> 16) as u8,
            second: (value >> 8) as u8,
        }
    }

    /// Packs the timestamp back into `UpdateDateTime` form.
    pub fn to_packed(self) -> u64 {
        (u64::from(self.year) << 48)
            | (u64::from(self.month) << 40)
            | (u64::from(self.day) << 32)
            | (u64::from(self.hour) << 24)
            | (u64::from(self.minute) << 16)
            | (u64::from(self.second) << 8)
    }
}

/// An open CPK archive. Reads lazily through the reader it wraps.
#[derive(Debug)]
pub struct CpkArchive<R: Read + Seek> {
    reader: R,
    file_len: u64,
    header: UtfTable,
    entries: Vec<CpkEntry>,
}

impl<R: Read + Seek> CpkArchive<R> {
    /// Reads the `CPK ` header, `TOC `, and `ETOC` (when present) tables and
    /// builds the entry list. Does not load any file content.
    pub fn open(mut reader: R) -> Result<Self, CpkError> {
        let file_len = reader.seek(SeekFrom::End(0))?;
        let header = read_table(&mut reader, file_len, 0, b"CPK ")?;

        let content_offset = header_u64(&header, "ContentOffset")?;
        let toc_offset = header_u64(&header, "TocOffset")?;
        // The real content base libcpk uses is the earlier of the two header
        // offsets; Blue hardcodes the observed 0x800.
        let base = content_offset.min(toc_offset);

        let toc = read_table(&mut reader, file_len, toc_offset, b"TOC ")?;
        for required in [
            "DirName",
            "FileName",
            "FileSize",
            "FileOffset",
            "ExtractSize",
        ] {
            if !toc.columns.iter().any(|c| c.name == required) {
                return Err(CpkError::MissingColumn(required));
            }
        }

        let etoc = match header_value(&header, "EtocOffset") {
            Some(&UtfValue::U64(offset)) => {
                let table = read_table(&mut reader, file_len, offset, b"ETOC")?;
                if table.columns.iter().any(|c| c.name == "UpdateDateTime") {
                    Some(table)
                } else {
                    None
                }
            }
            _ => None,
        };

        let mut entries = Vec::with_capacity(toc.rows.len());
        for row in 0..toc.rows.len() {
            let dir = cell_string(&toc, row, "DirName")?;
            let file = cell_string(&toc, row, "FileName")?;
            let path = format!(
                "{}/{}",
                dir.replace('\\', "/").trim_end_matches('/'),
                file.replace('\\', "/").trim_start_matches('/')
            );
            let size = cell_u32(&toc, row, "ExtractSize")?;
            let packed_size = cell_u32(&toc, row, "FileSize")?;
            let offset = cell_u64(&toc, row, "FileOffset")? + base;
            let modified = match toc.get(row, "ID") {
                Some(UtfValue::U32(id)) => etoc_timestamp(etoc.as_ref(), *id as usize),
                Some(UtfValue::U64(id)) => etoc_timestamp(etoc.as_ref(), *id as usize),
                _ => None,
            };
            entries.push(CpkEntry {
                path,
                size,
                packed_size,
                offset,
                modified,
            });
        }

        Ok(CpkArchive {
            reader,
            file_len,
            header,
            entries,
        })
    }

    /// The `CPK ` header table (Tvers, Align, the offsets, ...).
    pub fn header(&self) -> &UtfTable {
        &self.header
    }

    /// The file entries in `TOC ` order.
    pub fn entries(&self) -> &[CpkEntry] {
        &self.entries
    }

    /// Reads one entry's content: CRILAYLA-decompressed when `packed_size`
    /// differs from `size` and the data starts with the CRILAYLA magic, the
    /// raw stored bytes otherwise.
    pub fn read(&mut self, entry: &CpkEntry) -> Result<Vec<u8>, CpkError> {
        let content = self.read_at(entry.offset, entry.packed_size as usize)?;
        if entry.size != entry.packed_size && content.starts_with(b"CRILAYLA") {
            return crilayla::decompress(&content);
        }
        Ok(content)
    }

    /// Hands the wrapped reader back.
    pub fn into_inner(self) -> R {
        self.reader
    }

    fn read_at(&mut self, offset: u64, len: usize) -> Result<Vec<u8>, CpkError> {
        if offset.checked_add(len as u64).ok_or(CpkError::Truncated)? > self.file_len {
            return Err(CpkError::Truncated);
        }
        self.reader.seek(SeekFrom::Start(offset))?;
        let mut buffer = vec![0u8; len];
        self.reader.read_exact(&mut buffer).map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                CpkError::Truncated
            } else {
                CpkError::Io(e)
            }
        })?;
        Ok(buffer)
    }
}

/// Reads one tagged @UTF table starting at `offset` inside `reader`.
fn read_table<R: Read + Seek>(
    reader: &mut R,
    file_len: u64,
    offset: u64,
    tag: &[u8; 4],
) -> Result<UtfTable, CpkError> {
    if offset.checked_add(16).ok_or(CpkError::Truncated)? > file_len {
        return Err(CpkError::Truncated);
    }
    reader.seek(SeekFrom::Start(offset))?;
    let mut outer = [0u8; 16];
    reader.read_exact(&mut outer)?;
    let length = u64::from_le_bytes(outer[8..16].try_into().unwrap_or([0; 8]));
    if offset
        .checked_add(16)
        .and_then(|v| v.checked_add(length))
        .ok_or(CpkError::Truncated)?
        > file_len
    {
        return Err(CpkError::Truncated);
    }
    let mut table_bytes = Vec::with_capacity(16 + length as usize);
    table_bytes.extend_from_slice(&outer);
    let mut content = vec![0u8; length as usize];
    reader.read_exact(&mut content).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            CpkError::Truncated
        } else {
            CpkError::Io(e)
        }
    })?;
    table_bytes.extend_from_slice(&content);
    UtfTable::read(&table_bytes, tag)
}

fn header_value<'a>(header: &'a UtfTable, column: &str) -> Option<&'a UtfValue> {
    header.get(0, column)
}

fn header_u64(header: &UtfTable, column: &'static str) -> Result<u64, CpkError> {
    match header_value(header, column) {
        Some(&UtfValue::U64(v)) => Ok(v),
        Some(&UtfValue::U32(v)) => Ok(u64::from(v)),
        _ => Err(CpkError::MissingColumn(column)),
    }
}

fn etoc_timestamp(etoc: Option<&UtfTable>, id: usize) -> Option<CpkTimestamp> {
    match etoc?.get(id, "UpdateDateTime") {
        Some(&UtfValue::U64(v)) => Some(CpkTimestamp::from_packed(v)),
        _ => None,
    }
}

fn cell_string(table: &UtfTable, row: usize, column: &'static str) -> Result<String, CpkError> {
    match table.get(row, column) {
        Some(UtfValue::String(s)) => Ok(s.clone()),
        _ => Err(CpkError::MissingColumn(column)),
    }
}

fn cell_u32(table: &UtfTable, row: usize, column: &'static str) -> Result<u32, CpkError> {
    match table.get(row, column) {
        Some(&UtfValue::U32(v)) => Ok(v),
        Some(&UtfValue::U64(v)) => u32::try_from(v).map_err(|_| CpkError::MissingColumn(column)),
        _ => Err(CpkError::MissingColumn(column)),
    }
}

fn cell_u64(table: &UtfTable, row: usize, column: &'static str) -> Result<u64, CpkError> {
    match table.get(row, column) {
        Some(&UtfValue::U64(v)) => Ok(v),
        Some(&UtfValue::U32(v)) => Ok(u64::from(v)),
        _ => Err(CpkError::MissingColumn(column)),
    }
}
