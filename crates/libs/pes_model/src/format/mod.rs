//! The `.model` format: `ModelContainer` is the byte-identical layer
//! (header words and the eleven sections as opaque byte runs in file
//! order), `RecordArray` reads the table-of-contents-plus-records shape
//! every section and sub-table uses, and `PreFoxModel` is the typed layer
//! over them with every pointer resolved.

mod container;
mod datum;
mod model;
pub mod records;
mod write;

pub use container::{ModelContainer, Section, SectionKind};
pub use datum::{DatumFormat, DatumType};
pub use model::{
    Annotation, Bone, BoundingBox, EditorItem, EditorValue, FaceStream, Geometry, LodRecord, Mesh,
    PreFoxModel, VertexField,
};

/// Why a byte buffer is not a readable `.model`, or a model cannot be
/// written.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ModelError {
    /// The buffer ends before a structure that points past it.
    #[error("model is truncated")]
    Truncated,
    /// Wrong `MODEL` magic.
    #[error("invalid model magic")]
    BadMagic,
    /// A word that holds the same value in every known file holds another
    /// one; which word and the value found.
    #[error("unexpected value {value} for {what}")]
    UnexpectedConstant {
        /// The word that differs.
        what: &'static str,
        /// The value found.
        value: u32,
    },
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
    /// A name or annotation string is not UTF-8; where it starts in its
    /// section.
    #[error("invalid UTF-8 string at {offset}")]
    InvalidUtf8 {
        /// Offset of the string's first byte in its section.
        offset: usize,
    },
    /// A cross-section pointer or an index resolves to nothing.
    #[error("dangling {what} reference at {offset}")]
    BadReference {
        /// What the pointer was meant to name.
        what: &'static str,
        /// The file offset or index that resolved to nothing.
        offset: i64,
    },
    /// The bone-name table and the inverse-bind-matrix table disagree.
    #[error("{names} bone names for {matrices} matrices")]
    BoneTableMismatch {
        /// How many names section 5 holds.
        names: usize,
        /// How many matrices section 0's entry 0 holds.
        matrices: usize,
    },
    /// A vertex-field descriptor's type or format word is not one the
    /// format defines.
    #[error("unsupported vertex format (type {datum_type}, format {datum_format})")]
    UnsupportedVertexFormat {
        /// The descriptor's type word.
        datum_type: u32,
        /// The descriptor's format word.
        datum_format: u32,
    },
    /// The file uses a feature the typed layer does not cover (cloth,
    /// material combinations, locators): rewriting could not preserve it.
    #[error("unsupported feature: {0}")]
    Unsupported(&'static str),
    /// A model cannot be laid out as a `.model` (a count or a size that
    /// does not fit).
    #[error("cannot lay out model: {0}")]
    WriteLayout(&'static str),
    /// The buffer was WESYS-wrapped and could not be inflated; the
    /// decompression error's text.
    #[error("corrupt WESYS wrapper: {0}")]
    Wesys(String),
}
