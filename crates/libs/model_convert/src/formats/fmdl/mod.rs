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

/// The directory the game's dummy normal and specular maps live in (the same dummies the
/// anti-blur materials use).
const TEXTURE_DIRECTORY: &str = "/Assets/pes16/model/character/common/sourceimages/";

/// Whether a texture is the game's dummy normal or specular map: a file in
/// `TEXTURE_DIRECTORY` whose stem, case-folded, is `dummy_nrm` or `dummy_srm`, whatever its
/// extension (Konami's own models name `.tga`, the export writes `.dds`). These stand for
/// "no map" and mean nothing to PES 15-17.
pub(crate) fn is_game_dummy(directory: &str, file_name: &str) -> bool {
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    directory == TEXTURE_DIRECTORY
        && (stem.eq_ignore_ascii_case("dummy_nrm") || stem.eq_ignore_ascii_case("dummy_srm"))
}

/// Whether the PES 15-17 conversion of `model` leaves nothing of it: it has a mesh, and every
/// mesh carries the `invisible` shadow flag, which the `.model` export leaves out (the import
/// lifts each mesh's flags to its material, and `ir_to_model` returns
/// `ConvertError::EveryMeshHidden` when no mesh is left). The game draws nothing of such a model
/// on Fox either. A model with no mesh is not hidden.
pub fn every_mesh_hidden(model: &::fmdl::Model) -> bool {
    !model.meshes.is_empty()
        && model
            .meshes
            .iter()
            .all(|mesh| mesh.shadow_flags & crate::materials::to_fox::INVISIBLE_BIT != 0)
}

/// The bind pose `name` carries in PES21's template tables, body first then face and hands.
fn template_matrix(name: &str) -> Option<Affine> {
    skeletons::version_bone(skeletons(PesVersion::Pes21), name).map(|bone| bone.matrix)
}
