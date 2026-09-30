//! The listing a consumer supplies: one export source as a flat path list plus
//! the small metadata files' bytes, and the settings the structure pass needs.

use std::collections::BTreeMap;

use pes_version::PesVersion;

/// One export source's canonical listing; `parse_listing` canonicalizes it.
/// `display_name` is the source's stem: the folder name, or the archive name
/// without extension.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalListing {
    /// The source's stem: folder name, or archive name without extension.
    pub display_name: String,
    /// Every path in the source, in any order.
    pub entries: Vec<ListedEntry>,
}

/// One path in the listing, spelled as the source spells it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedEntry {
    /// As the source spells it, relative to the source root.
    pub path: String,
    /// Whether the entry is a file or a folder.
    pub kind: ListedKind,
}

/// What a `ListedEntry` is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ListedKind {
    /// A file, with its stored size.
    File {
        /// The file's size in bytes.
        size: u64,
    },
    /// A directory; needed only for a folder with nothing below it (an empty kit).
    Folder,
}

/// The small files the structure pass reads, keyed by `ListedEntry::path`. The
/// consumer reads every listed file `is_small_metadata` accepts; a failed read
/// carries its reason (`source_read_failed`, with the disposition of what the
/// file is: `DropExport` for a roster, `DropFile` for `notes.txt` or
/// `icon.txt`). A listed file missing from the map was not read: Phase 3
/// treats it as a failed read; the GUI's shallow check gives it its own state
/// in Phase 8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmallMetadata {
    /// Each small metadata file's content, or its read failure's reason.
    pub files: BTreeMap<String, Result<Vec<u8>, String>>,
}

/// The settings that change a consequence, supplied by the consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationContext {
    /// The target version: Fox or pre-Fox allowed names.
    pub version: PesVersion,
    /// Disallowed file types are errors instead of info notes.
    pub strict_file_type_check: bool,
    /// Keep eligible dropped content instead of discarding it.
    pub pass_through: bool,
}
