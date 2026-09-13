//! The `format/` half of `fmdl`: pure codecs that reproduce what PES reads
//! and writes, with no interpretation of the data inside the blocks.

mod container;
pub(crate) mod f16;
mod file;
pub mod records;
mod skl;
mod vertex;

pub use container::{ByteBlock, FmdlContainer, RecordBlock};
pub use file::FmdlFile;
pub use skl::{SklBone, SklFile};
pub use vertex::{DatumFormat, DatumType, MeshVertices, VertexAttribute};

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
    /// A vertex datum type or storage format the FMDL format does not
    /// define, or a pairing it rejects.
    #[error("unsupported vertex format: datum type {datum_type}, format {datum_format}")]
    UnsupportedVertexFormat {
        /// The raw datum type byte.
        datum_type: u8,
        /// The raw datum format byte.
        datum_format: u8,
    },
    /// A vertex format that names known types but combines them wrongly.
    #[error("invalid vertex format: {0}")]
    InvalidVertexFormat(&'static str),
    /// An index into one of the file's tables points past its end.
    #[error("invalid {what} reference {index}")]
    BadReference {
        /// Which table the index names.
        what: &'static str,
        /// The offending index.
        index: usize,
    },
    /// The data handed to `encode_vertices`/`encode_faces` does not match
    /// the mesh's declared counts or attribute set.
    #[error("vertex data mismatch: {0}")]
    VertexMismatch(&'static str),
    /// A bone or mesh group parent chain loops back on itself.
    #[error("parent cycle in {0}")]
    ParentCycle(&'static str),
    /// A mesh belongs to two mesh groups or to none.
    #[error("bad mesh group assignment: {0}")]
    BadMeshGroupAssignment(&'static str),
    /// A mesh's bone group holds more than the format's 32 entries.
    #[error("too many bones in bone group: {0}")]
    TooManyBones(usize),
    /// A mesh holds more vertices than the format's u16 count allows.
    #[error("too many vertices in mesh: {0}")]
    TooManyVertices(usize),
}
