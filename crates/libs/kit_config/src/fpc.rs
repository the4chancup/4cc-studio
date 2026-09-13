//! Applying and detecting the FPC kit-config values; the values themselves
//! live in the `fpc` crate.

use pes_version::PesVersion;

use crate::model::KitConfig;

/// Sets the four kit-config fields to the version's FPC values. Returns
/// `false` (no change) on versions where FPC does not exist.
pub fn apply_fpc(config: &mut KitConfig, version: PesVersion) -> bool {
    let Some(values) = fpc::kit_values(version) else {
        return false;
    };
    config.shirt.model = values.shirt_model as u8;
    config.shorts.model = values.shorts_model as u8;
    config.shirt.collar = values.collar as u8;
    config.shirt.winter_collar = values.winter_collar as u8;
    true
}

/// Whether the config carries the version's FPC values. Always `false` on
/// versions where FPC does not exist.
pub fn matches_fpc(config: &KitConfig, version: PesVersion) -> bool {
    match fpc::kit_values(version) {
        Some(values) => {
            values.shirt_model == u16::from(config.shirt.model)
                && values.shorts_model == u16::from(config.shorts.model)
                && values.collar == u16::from(config.shirt.collar)
                && values.winter_collar == u16::from(config.shirt.winter_collar)
        }
        None => false,
    }
}
