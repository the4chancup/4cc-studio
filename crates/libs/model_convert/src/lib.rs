//! The canonical model IR, its native FMDL and `.model` importers and exporters, the embedded
//! per-version skeletons with retargeting, and the hand auto-split. Same-format work stays in
//! the format crates; this crate is the hub that cross-format conversion and the explicitly
//! planned IR operations go through.

/// The 3x4 row-major affine transform bone transforms are stored in.
pub mod affine;
mod convert;
/// The native-format importers and exporters over the IR.
pub mod formats;
/// The canonical model: the superset both format importers fill and both exporters read.
pub mod ir;
/// What a conversion could not carry: findings with stable codes.
pub mod loss;
/// The engine-neutral material schema.
pub mod materials;
/// IR-level operations that never know which format the data came from.
pub mod ops;
/// The per-version player skeletons and their retargeting tables.
pub mod skeletons;

pub use convert::{Converted, NativeModelBundle, convert, needs_conversion};
pub use formats::{ConvertError, Imported, fmdl::ExportedFox, pes_model::ExportedPreFox};
pub use loss::{Finding, Subject};
