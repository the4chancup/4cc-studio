//! Format-native operations on `Model`: things that change a model's
//! meaning rather than its bytes.

/// Texture path editing on the sibling `.mtl`.
pub mod paths;
/// Vertex-loop preservation through the vertex ordering convention.
pub mod vertex_enc;
