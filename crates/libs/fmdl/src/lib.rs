//! Fox Engine FMDL model files and SKL skeletons: codec, format-native mesh operations, deep checks.

/// The format-level codecs: byte-identical read and write.
pub mod format;

pub use format::{FmdlContainer, FmdlError, SklFile};
