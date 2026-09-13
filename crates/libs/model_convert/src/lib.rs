//! The canonical model IR, its native FMDL and `.model` importers and exporters, the embedded
//! per-version skeletons with retargeting, and the hand auto-split. Same-format work stays in
//! the format crates; this crate is the hub that cross-format conversion and the explicitly
//! planned IR operations go through.

/// The 3x4 row-major affine transform bone transforms are stored in.
pub mod affine;
/// The per-version player skeletons and their retargeting tables.
pub mod skeletons;
