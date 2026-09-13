//! Writing a CPK archive in the layout the 4cc compilers have always produced:
//! files stored back to back at 0x800 alignment, then `TOC `, then `ETOC`
//! (only when every entry carries a timestamp), then the `CPK ` header table
//! written over the leading padding at offset 0.

use std::io::{Seek, SeekFrom, Write};

use crate::CpkError;
use crate::read::CpkTimestamp;
use crate::utf::{UtfColumn, UtfKind, UtfStorage, UtfTable, UtfValue};

const ALIGNMENT: u64 = 0x800;

/// One file queued for the archive.
#[derive(Debug)]
struct PendingEntry {
    path: String,
    size: u32,
    offset: u64,
    modified: Option<CpkTimestamp>,
}

/// Builds a CPK image in the wrapped writer. Alignment is fixed at 0x800.
#[derive(Debug)]
pub struct CpkWriter<W: Write + Seek> {
    writer: W,
    position: u64,
    files: Vec<PendingEntry>,
    tool_version: String,
}

impl<W: Write + Seek> CpkWriter<W> {
    /// Starts a new archive: 0x7FA zero bytes plus `(c)CRI`, so the first file
    /// lands at 0x800. `tool_version` becomes the header's `Tvers` string;
    /// passing the string an existing archive carries reproduces its bytes.
    pub fn new(mut writer: W, tool_version: &str) -> Result<Self, CpkError> {
        writer.write_all(&[0u8; 0x7FA])?;
        writer.write_all(b"(c)CRI")?;
        Ok(CpkWriter {
            writer,
            position: ALIGNMENT,
            files: Vec::new(),
            tool_version: tool_version.to_owned(),
        })
    }

    /// Stores `content` under `path`, padded to the 0x800 alignment. The same
    /// path twice is [`CpkError::DuplicatePath`]; the archive never replaces.
    pub fn add(
        &mut self,
        path: &str,
        content: &[u8],
        modified: Option<CpkTimestamp>,
    ) -> Result<(), CpkError> {
        if self.files.iter().any(|f| f.path == path) {
            return Err(CpkError::DuplicatePath(path.to_owned()));
        }
        self.files.push(PendingEntry {
            path: path.to_owned(),
            size: content.len() as u32,
            offset: self.position,
            modified,
        });
        self.writer.write_all(content)?;
        let padding = (ALIGNMENT - u64::from(content.len() as u32) % ALIGNMENT) % ALIGNMENT;
        self.writer.write_all(&vec![0u8; padding as usize])?;
        self.position += content.len() as u64 + padding;
        Ok(())
    }

    /// Writes the `TOC ` table, the `ETOC` when every entry has a timestamp,
    /// then the `CPK ` header table at offset 0, and returns the writer.
    pub fn finish(mut self) -> Result<W, CpkError> {
        let mut sorted: Vec<&PendingEntry> = self.files.iter().collect();
        sorted.sort_by_key(|f| f.path.to_uppercase());

        let mut toc = table(
            "CpkTocInfo",
            &[
                ("DirName", UtfKind::String),
                ("FileName", UtfKind::String),
                ("FileSize", UtfKind::U32),
                ("ExtractSize", UtfKind::U32),
                ("FileOffset", UtfKind::U64),
                ("ID", UtfKind::U32),
                ("UserString", UtfKind::String),
            ],
        );
        let mut etoc = table(
            "CpkEtocInfo",
            &[
                ("UpdateDateTime", UtfKind::U64),
                ("LocalDir", UtfKind::String),
            ],
        );

        let mut total_size = 0u64;
        for entry in &sorted {
            let (dir, file) = split_path(&entry.path);
            toc.rows.push(vec![
                UtfValue::String(dir.clone()),
                UtfValue::String(file),
                UtfValue::U32(entry.size),
                UtfValue::U32(entry.size),
                UtfValue::U64(entry.offset - ALIGNMENT),
                UtfValue::U32(toc.rows.len() as u32),
                UtfValue::String(String::new()),
            ]);
            if let Some(modified) = entry.modified {
                etoc.rows.push(vec![
                    UtfValue::U64(modified.to_packed()),
                    UtfValue::String(dir),
                ]);
            }
            total_size += u64::from(entry.size);
        }

        let toc_offset = self.position;
        let toc_bytes = toc.write(b"TOC ");
        let toc_size = toc_bytes.len() as u64;
        self.writer.write_all(&toc_bytes)?;
        self.position += toc_size;

        let (etoc_offset, etoc_size) = if etoc.rows.len() == toc.rows.len() {
            let padding = (ALIGNMENT - toc_size % ALIGNMENT) % ALIGNMENT;
            self.writer.write_all(&vec![0u8; padding as usize])?;
            self.position += padding;
            // The trailing all-empty row the layout ends with.
            etoc.rows
                .push(vec![UtfValue::U64(0), UtfValue::String(String::new())]);
            let offset = self.position;
            let bytes = etoc.write(b"ETOC");
            let size = bytes.len() as u64;
            self.writer.write_all(&bytes)?;
            self.position += size;
            (Some(offset), Some(size))
        } else {
            (None, None)
        };

        let header = self.header_table(toc_offset, toc_size, etoc_offset, etoc_size, total_size);
        let header_bytes = header.write(b"CPK ");
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(&header_bytes)?;
        Ok(self.writer)
    }

    /// The 44-column `CPK ` header row, in the fixed order the layout uses.
    fn header_table(
        &self,
        toc_offset: u64,
        toc_size: u64,
        etoc_offset: Option<u64>,
        etoc_size: Option<u64>,
        total_size: u64,
    ) -> UtfTable {
        let mut header = UtfTable {
            name: "CpkHeader".to_owned(),
            columns: Vec::new(),
            rows: vec![Vec::new()],
        };
        let mut add = |name: &'static str, kind: UtfKind, value: UtfValue| {
            header.columns.push(UtfColumn {
                name: name.to_owned(),
                kind,
                storage: UtfStorage::Variable,
            });
            header.rows[0].push(value);
        };
        let null = || UtfValue::Null;
        add("UpdateDateTime", UtfKind::U64, UtfValue::U64(1));
        add("FileSize", UtfKind::U64, null());
        add("ContentOffset", UtfKind::U64, UtfValue::U64(ALIGNMENT));
        add(
            "ContentSize",
            UtfKind::U64,
            UtfValue::U64(toc_offset - ALIGNMENT),
        );
        add("TocOffset", UtfKind::U64, UtfValue::U64(toc_offset));
        add("TocSize", UtfKind::U64, UtfValue::U64(toc_size));
        add("TocCrc", UtfKind::U32, null());
        add("HtocOffset", UtfKind::U64, null());
        add("HtocSize", UtfKind::U64, null());
        add(
            "EtocOffset",
            UtfKind::U64,
            etoc_offset.map_or_else(null, UtfValue::U64),
        );
        add(
            "EtocSize",
            UtfKind::U64,
            etoc_size.map_or_else(null, UtfValue::U64),
        );
        add("ItocOffset", UtfKind::U64, null());
        add("ItocSize", UtfKind::U64, null());
        add("ItocCrc", UtfKind::U32, null());
        add("GtocOffset", UtfKind::U64, null());
        add("GtocSize", UtfKind::U64, null());
        add("GtocCrc", UtfKind::U32, null());
        add("HgtocOffset", UtfKind::U64, null());
        add("HgtocSize", UtfKind::U64, null());
        add("EnabledPackedSize", UtfKind::U64, UtfValue::U64(total_size));
        add("EnabledDataSize", UtfKind::U64, UtfValue::U64(total_size));
        add("TotalDataSize", UtfKind::U64, null());
        add("Tocs", UtfKind::U32, null());
        add(
            "Files",
            UtfKind::U32,
            UtfValue::U32(self.files.len() as u32),
        );
        add("Groups", UtfKind::U32, UtfValue::U32(0));
        add("Attrs", UtfKind::U32, UtfValue::U32(0));
        add("TotalFiles", UtfKind::U32, null());
        add("Directories", UtfKind::U32, null());
        add("Updates", UtfKind::U32, null());
        add("Version", UtfKind::U16, UtfValue::U16(7));
        add("Revision", UtfKind::U16, UtfValue::U16(14));
        add("Align", UtfKind::U16, UtfValue::U16(ALIGNMENT as u16));
        add("Sorted", UtfKind::U16, UtfValue::U16(1));
        add("EnableFileName", UtfKind::U16, UtfValue::U16(1));
        add("EID", UtfKind::U16, null());
        add("CpkMode", UtfKind::U32, UtfValue::U32(1));
        add(
            "Tvers",
            UtfKind::String,
            UtfValue::String(self.tool_version.clone()),
        );
        add("Comment", UtfKind::String, UtfValue::String(String::new()));
        add("Codec", UtfKind::U32, UtfValue::U32(0));
        add("DpkItoc", UtfKind::U32, UtfValue::U32(0));
        add("EnableTocCrc", UtfKind::U16, UtfValue::U16(0));
        add("EnableFileCrc", UtfKind::U16, UtfValue::U16(0));
        add("CrcMode", UtfKind::U32, UtfValue::U32(0));
        add("CrcTable", UtfKind::Bytes, UtfValue::Bytes(Vec::new()));
        header
    }
}

fn table(name: &str, columns: &[(&'static str, UtfKind)]) -> UtfTable {
    UtfTable {
        name: name.to_owned(),
        columns: columns
            .iter()
            .map(|(name, kind)| UtfColumn {
                name: (*name).to_owned(),
                kind: *kind,
                // Recomputed from the data on write.
                storage: UtfStorage::Variable,
            })
            .collect(),
        rows: Vec::new(),
    }
}

/// Splits `dir/name` at the last `/`; no slash means an empty dir.
fn split_path(path: &str) -> (String, String) {
    match path.rfind('/') {
        Some(pos) => (path[..pos].to_owned(), path[pos + 1..].to_owned()),
        None => (String::new(), path.to_owned()),
    }
}
