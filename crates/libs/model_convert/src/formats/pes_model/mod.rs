//! The pre-Fox `.model` + `.mtl` importer and exporter: `model_to_ir` decodes the mesh
//! splitting, rebuilds the parent-first bone order from the render hierarchy, and keeps the
//! `.mtl` tables verbatim on the materials; `ir_to_model` resolves each material for
//! pre-Fox, writes the inverse bind matrices back, and runs the format crate's encoders.

mod export;
mod import;
#[cfg(test)]
mod tests;

pub use export::{ExportedPreFox, ir_to_model};
pub use import::model_to_ir;
