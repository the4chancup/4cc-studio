//! The kit-config values every kit of an FPC team carries.

/// The kit-config values every kit of an FPC team carries, goalkeeper kit
/// included (`resources/FPC.wikitext` lines 14–20). Each is a `u8`, the width
/// a kit config stores it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KitFpcValues {
    /// Shirt model (FPC.wikitext line 17).
    pub shirt_model: u8,
    /// Shorts model (FPC.wikitext line 18).
    pub shorts_model: u8,
    /// Collar (FPC.wikitext line 19).
    pub collar: u8,
    /// Winter collar (FPC.wikitext line 20).
    pub winter_collar: u8,
}

/// The FPC kit values. They are the same on every supported PES version
/// (15 to 21): the cup's installs carry these four in their FPC teams' kit
/// configs on each of them, and each install ships the empty collar 105 and
/// shorts 16 models they point at.
pub fn kit_values() -> KitFpcValues {
    KitFpcValues {
        shirt_model: 176,
        shorts_model: 16,
        collar: 105,
        winter_collar: 105,
    }
}
