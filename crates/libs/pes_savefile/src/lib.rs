//! The PES EDIT savefile (`EDIT00000000`, `EDIT.bin` on PES 15): the per-version container
//! crypto, the schema-driven player/team/tactics codec over the decrypted payload, and the
//! operations consumers run on the model (settings merge, cross-version conversion,
//! transplant, fingerprint, interchange formats).
//!
//! Layers, bytes upward: `container` (bytes <-> decrypted sections, knows nothing about
//! players), `schema` + `codec` (sections <-> fields, knows nothing about encryption), `model`
//! (what consumers edit, no I/O and no version), then the operations. `file.rs` is the only
//! module that composes container and codec.

/// Bytes ↔ decrypted sections: the PES 16-21 keyed container and the PES 15 LCG one.
pub mod container;
