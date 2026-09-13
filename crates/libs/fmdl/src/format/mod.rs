//! The `format/` half of `fmdl`: pure codecs that reproduce what PES reads
//! and writes, with no interpretation of the data inside the blocks.

mod container;
mod file;
pub mod records;
mod skl;

pub use container::{ByteBlock, FmdlContainer, RecordBlock};
pub use file::FmdlFile;
pub use skl::{SklBone, SklFile};

/// Why a byte buffer is not a readable FMDL or SKL.
#[derive(Debug, thiserror::Error)]
pub enum FmdlError {
    /// The buffer ends before a structure that points past it.
    #[error("fmdl is truncated")]
    Truncated,
    /// Wrong `FMDL` magic.
    #[error("invalid fmdl magic")]
    BadMagic,
    /// A section's descriptor table names the same block id twice.
    #[error("duplicate section {section} block {id}")]
    DuplicateBlock {
        /// 0 for the record section, 1 for the raw-data section.
        section: u8,
        /// The block id that appeared twice.
        id: u32,
    },
    /// The SKL magic field is not the format's constant 12.
    #[error("invalid skl magic: {0}")]
    BadSklMagic(u32),
    /// The SKL record size field is not the format's constant 56.
    #[error("invalid skl record size: {0}")]
    BadSklRecordSize(u32),
    /// An SKL bone name is not UTF-8 or has no terminating NUL in the file.
    #[error("invalid skl bone name")]
    InvalidName,
    /// A string descriptor or its index points at data the file does not
    /// hold.
    #[error("invalid string reference {index}")]
    BadStringReference {
        /// The index into the strings block that failed.
        index: usize,
    },
    /// A string descriptor's span is not UTF-8.
    #[error("invalid utf-8 in fmdl string")]
    InvalidUtf8,
}
