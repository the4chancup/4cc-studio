//! The Studio aesthetics export format as a source-neutral object model: the
//! consumer supplies a canonical listing of one export source (folder, ZIP or
//! 7z — this crate owns no I/O), and the crate parses it into a draft, decides
//! each structural issue's disposition, and projects the sanitized export the
//! consumers compile or repair from.
//!
//! Five consumers (Team compiler, Export upgrader, Kit config editor, Refs
//! arranger, Team Creator) read this crate, so its layout is organized by the
//! progression stage a consumer can stop at.

mod conventions;
mod listing;
mod slots;
mod validate;

pub use conventions::{
    FileKind, Marker, MetadataFile, ModelFormat, SharedKind, classify, is_small_metadata,
};
pub use listing::{CanonicalListing, ListedEntry, ListedKind, SmallMetadata, ValidationContext};
pub use slots::{PlayerSlot, RefSlot};
pub use validate::{Disposition, ISSUE_CODES, IssueScope, ValidationIssue};
