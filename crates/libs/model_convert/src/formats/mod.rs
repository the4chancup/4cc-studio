//! The native-format importers and exporters: one `x_to_ir` and one `ir_to_x` per format.

/// FMDL (PES 2018–2021) import and export.
pub mod fmdl;
// pub mod pes_model; comes next handoff

/// Why a conversion failed. Findings are for what a format cannot carry; this is for input
/// the format crate rejects or an IR that is not consistent.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// The `fmdl` crate rejected the model.
    #[error(transparent)]
    Fmdl(#[from] ::fmdl::FmdlError),
    /// The IR failed validation.
    #[error(transparent)]
    Validation(#[from] crate::ir::ValidationError),
}
