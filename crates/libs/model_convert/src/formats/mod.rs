//! The native-format importers and exporters: one `x_to_ir` and one `ir_to_x` per format.

/// FMDL (PES 2018–2021) import and export.
pub mod fmdl;
/// `.model` + `.mtl` (PES 2015–2017) import and export.
pub mod pes_model;

use crate::ir::CanonicalModel;
use crate::loss;

/// A model imported from a native format plus what the import had to decide.
#[derive(Debug)]
pub struct Imported {
    /// The IR model.
    pub model: CanonicalModel,
    /// The findings the import produced.
    pub findings: Vec<loss::Finding>,
}

/// Why a conversion failed. Findings are for what a format cannot carry; this is for input
/// the format crate rejects or an IR that is not consistent.
#[derive(Debug, thiserror::Error)]
pub enum ConvertError {
    /// The `fmdl` crate rejected the model.
    #[error(transparent)]
    Fmdl(#[from] ::fmdl::FmdlError),
    /// The `pes_model` crate rejected the `.model`.
    #[error(transparent)]
    PesModel(#[from] ::pes_model::format::ModelError),
    /// The `pes_model` crate rejected the `.mtl`.
    #[error(transparent)]
    Mtl(#[from] ::pes_model::format::mtl::MtlError),
    /// A material name the model binds has no `.mtl` definition.
    #[error("material `{0}` bound by the model has no `.mtl` definition")]
    MaterialUndefined(String),
    /// A bone's stored inverse-bind matrix could not be inverted.
    #[error("bone {0}'s stored matrix is singular")]
    SingularBoneMatrix(usize),
    /// The IR failed validation.
    #[error(transparent)]
    Validation(#[from] crate::ir::ValidationError),
}

/// Zeroes every weighted slot that points past `group_len` and renormalizes only the
/// vertices a slot was dropped from; a vertex that loses all its slots ends at all zero.
/// Returns how many slots were dropped.
fn drop_out_of_group_slots(
    indices: &[[u8; 4]],
    weights: &mut [[f32; 4]],
    group_len: usize,
) -> usize {
    let mut dropped = 0;
    for (row, weights) in indices.iter().zip(weights.iter_mut()) {
        let mut vertex_dropped = false;
        for (slot, index) in row.iter().enumerate() {
            if weights[slot] > 0.0 && usize::from(*index) >= group_len {
                weights[slot] = 0.0;
                vertex_dropped = true;
                dropped += 1;
            }
        }
        // Renormalize only the vertices a slot dropped from; the rest keep the file's
        // exact weights.
        if vertex_dropped {
            let sum: f32 = weights.iter().sum();
            if sum > 0.0 {
                for weight in weights.iter_mut() {
                    *weight /= sum;
                }
            }
        }
    }
    dropped
}
