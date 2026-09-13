//! Format-native operations on `Model`: things that change a model's
//! meaning rather than its bytes.

/// The anti-blur mesh duplication PES applies to flagged meshes.
pub mod antiblur;
/// Texture path rewriting on the record layer.
pub mod paths;
