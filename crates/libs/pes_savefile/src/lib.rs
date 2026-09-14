//! The PES EDIT savefile (`EDIT00000000`, `EDIT.bin` on PES 15): the per-version container
//! crypto, the schema-driven player/team/tactics codec over the decrypted payload, and the
//! operations consumers run on the model (settings merge, cross-version conversion,
//! transplant, fingerprint, interchange formats).
//!
//! Layers, bytes upward: `container` (bytes <-> decrypted sections, knows nothing about
//! players), `schema` + `codec` (sections <-> fields, knows nothing about encryption), `model`
//! (what consumers edit, no I/O and no version), then the operations. `file.rs` is the only
//! module that composes container and codec.

/// Records ↔ fields: the one generic bit codec over a `&VersionSchema`.
pub mod codec;
/// Bytes ↔ decrypted sections: the PES 16-21 keyed container and the PES 15 LCG one.
pub mod container;
/// Savefile discovery under the user's Documents folder.
pub mod discovery;
/// The whole-file API: `EditFile` composes container and codec.
pub mod file;
/// What consumers edit: version-independent player/team/tactics entries, no I/O.
pub mod model;
/// Per-version field tables and section layouts, data only, no logic.
pub mod schema;

pub use file::{EditFile, SaveError};
