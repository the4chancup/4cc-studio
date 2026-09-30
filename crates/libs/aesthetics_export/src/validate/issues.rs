//! What the structure pass reports. The crate decides each issue's
//! disposition; the consumer maps each issue to its own message (severity,
//! text, hint), testing that every code in `ISSUE_CODES` has a catalog row.

use vtree::ScopePath;

/// One finding of the structure pass, with what it is about and what becomes
/// of it. `code` is stable and always one of `ISSUE_CODES`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    /// The stable code (always one of `ISSUE_CODES`).
    pub code: &'static str,
    /// What the issue is about.
    pub scope: IssueScope,
    /// Structured fields, in the message template's order.
    pub context: Vec<(&'static str, String)>,
    /// The effective disposition: `Keep` when `pass_through` kept the item.
    pub disposition: Disposition,
    /// `pass_through` turned a `DropFile`/`DropFolder` into `Keep`.
    pub passed_through: bool,
}

/// What a `ValidationIssue` is about, within the one export this crate sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IssueScope {
    /// The export itself.
    Export,
    /// One folder (player, shared or kit).
    Folder(ScopePath),
    /// One file.
    File(ScopePath),
    /// One roster-file line: the file, the 1-based line number, and the slot
    /// when it parsed.
    RosterEntry {
        /// The roster file's path.
        file: ScopePath,
        /// The 1-based line number.
        line: usize,
        /// The parsed slot, when the line carries one.
        slot: Option<u16>,
    },
}

/// What becomes of the item an issue's scope names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Disposition {
    /// Kept unchanged.
    Keep,
    /// One file dropped.
    DropFile,
    /// One normalized roster assignment dropped.
    DropSlot,
    /// One folder dropped.
    DropFolder,
    /// The whole export dropped.
    DropExport,
}

/// Every code the crate emits, listed here so consumers can test their
/// catalog against it. This slice emits none; the parse slice adds its codes.
pub const ISSUE_CODES: &[&str] = &[];
