//! The `.model` container and record-array reader: `ModelContainer` is the
//! byte-identical layer (header words and the eleven sections as opaque
//! byte runs in file order); `RecordArray` reads the
//! table-of-contents-plus-records shape every section and sub-table uses.

mod container;
pub mod records;

pub use container::{ModelContainer, Section, SectionKind};

/// Why a byte buffer is not a readable `.model`.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ModelError {
    /// The buffer ends before a structure that points past it.
    #[error("model is truncated")]
    Truncated,
    /// Wrong `MODEL` magic.
    #[error("invalid model magic")]
    BadMagic,
    /// A header word that is constant in every known file (0 at offset 12,
    /// 9 at offset 16) holds another value; the value found.
    #[error("invalid header word {0}")]
    BadHeaderWord(u32),
    /// The section table's own record array or an entry is off; the value
    /// that failed the check.
    #[error("invalid section table value {0}")]
    BadSectionTable(u32),
    /// A record array at `at` declares records before its own table of
    /// contents ends.
    #[error("invalid record array at {at}")]
    BadRecordArray {
        /// Offset of the array's table of contents in the buffer.
        at: usize,
    },
    /// The buffer was WESYS-wrapped and could not be inflated; the
    /// decompression error's text.
    #[error("corrupt WESYS wrapper: {0}")]
    Wesys(String),
}
