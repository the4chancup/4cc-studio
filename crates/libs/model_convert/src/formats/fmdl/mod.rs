//! The FMDL importer and exporter: `fmdl_to_ir` decodes the format extensions, reads the bind
//! pose from the companion SKL or PES21's template tables, and lifts the per-mesh flags to the
//! materials; `ir_to_fmdl` resolves each material for Fox, quantizes weights, derives the bone
//! positions an import did not supply, and runs the format crate's encoders.

mod export;
mod import;
#[cfg(test)]
mod tests;

use pes_version::PesVersion;

use crate::affine::Affine;
use crate::skeletons::{self, skeletons};

pub use super::Imported;
pub use export::{ExportedFox, ir_to_fmdl};
pub use import::fmdl_to_ir;

/// The bind pose `name` carries in PES21's template tables, body first then face and hands.
fn template_matrix(name: &str) -> Option<Affine> {
    skeletons::version_bone(skeletons(PesVersion::Pes21), name).map(|bone| bone.matrix)
}
