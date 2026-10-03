//! The resources bundled into the binary.

use pes_version::PesVersion;

const UNIFORM_PARAMETER_18: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/bins/UniformParameter18.bin"
));
const UNIFORM_PARAMETER_19: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/bins/UniformParameter19.bin"
));

/// The `kit` texture of a placeholder kit: the magenta/black checkerboard, as a DDS
/// (`resources/kits/README.md`).
pub(crate) const PLACEHOLDER_KIT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/kits/placeholder_kit.dds"
));

/// The skeleton packed under a slot's name (`boots.skl`, `fcl_hair_sim.skl`) beside a boots or
/// hair model that brings no skeleton of its own: PES 2021's full-body `body.skl`, since
/// exports put full-body models in both slots (`resources/skeletons/README.md`), not the
/// game's four-bone boots skeleton.
pub(crate) const BODY_SKELETON: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/skeletons/pes21/body.skl"
));

/// The `face_diff.bin` packed into a Fox face package whose sources hold none: the face
/// parameter file the game expects beside every face's models (`resources/templates/README.md`).
pub(crate) const FACE_DIFF: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/templates/face_diff.bin"
));

/// The `fcl_hair_sim.fclo` packed beside a `fcl_hair.fmdl` whose sources hold none: a cloth
/// simulation with nothing in it (`resources/templates/README.md`).
pub(crate) const FCL_HAIR_SIM_FCLO: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../resources/templates/fcl_hair_sim.fclo"
));

/// The `UniformParameter.bin` a compile adds its kit configs to when it has no installed one
/// to start from: one base for PES 18, one for 19-21. `None` for the pre-Fox versions, which
/// have no such bin.
pub(crate) fn uniform_parameter_base(version: PesVersion) -> Option<&'static [u8]> {
    match version {
        PesVersion::Pes18 => Some(UNIFORM_PARAMETER_18),
        PesVersion::Pes19 | PesVersion::Pes20 | PesVersion::Pes21 => Some(UNIFORM_PARAMETER_19),
        PesVersion::Pes15 | PesVersion::Pes16 | PesVersion::Pes17 => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pes_18_has_its_own_base_and_19_to_21_share_one() {
        assert_eq!(
            uniform_parameter_base(PesVersion::Pes18),
            Some(UNIFORM_PARAMETER_18)
        );
        for version in [PesVersion::Pes19, PesVersion::Pes20, PesVersion::Pes21] {
            assert_eq!(uniform_parameter_base(version), Some(UNIFORM_PARAMETER_19));
        }
        for version in [PesVersion::Pes15, PesVersion::Pes16, PesVersion::Pes17] {
            assert_eq!(uniform_parameter_base(version), None);
        }
        assert_ne!(UNIFORM_PARAMETER_18, UNIFORM_PARAMETER_19);
    }
}
