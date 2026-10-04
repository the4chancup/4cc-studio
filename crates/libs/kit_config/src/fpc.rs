//! Applying and detecting the FPC kit-config values; the values themselves
//! live in the `fpc` crate.

use crate::model::KitConfig;

/// Sets the config's shirt model, shorts model, collar and winter collar to
/// the FPC values, leaving every other field as it is.
pub fn apply_fpc(config: &mut KitConfig) {
    let values = fpc::kit_values();
    config.shirt.model = values.shirt_model;
    config.shorts.model = values.shorts_model;
    config.shirt.collar = values.collar;
    config.shirt.winter_collar = values.winter_collar;
}

/// Whether the config carries all four FPC values.
pub fn matches_fpc(config: &KitConfig) -> bool {
    let values = fpc::kit_values();
    config.shirt.model == values.shirt_model
        && config.shorts.model == values.shorts_model
        && config.shirt.collar == values.collar
        && config.shirt.winter_collar == values.winter_collar
}
