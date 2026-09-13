//! Fox Engine FMDL model files and SKL skeletons: codec, format-native mesh operations, deep checks.

/// Findings-based lint of a loaded `Model`.
pub mod check;
/// The format-level codecs: byte-identical read and write.
pub mod format;
/// The semantic model layer: a `Model` with every index resolved.
pub mod model;
/// Format-native operations on `Model`.
pub mod ops;

pub use format::records::*;
pub use format::{FmdlContainer, FmdlError, FmdlFile, SklFile};
pub use model::*;
