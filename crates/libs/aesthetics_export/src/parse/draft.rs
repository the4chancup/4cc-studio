//! The draft: everything the normalized tree holds, grouped by content
//! folder, nothing dropped.

use vtree::ScopePath;

use teams_list::TeamName;

use crate::{ExportCoverage, FileKind};

/// Everything the normalized tree holds, grouped by content folder, nothing
/// dropped. `parse_listing` first normalizes the root (see
/// `team_compiler/pipeline.md` "Per-export serial steps", step 1), reporting
/// `nested_folders_fixed`, `nested_root_ambiguous` and `nested_root_conflict`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AestheticsExportDraft {
    /// The source's stem, kept for presentation and source identity.
    pub export_display_name: String,
    /// The canonical team name from the first token; `None` when the stem has
    /// no token (`team_name_unknown` with an empty name).
    pub team_name: Option<TeamName>,
    /// The coverage tag from the second token, `Full` for a referee export
    /// whatever its words; `None` when a team export's second token is
    /// neither tag (`export_tag_missing`).
    pub coverage: Option<ExportCoverage>,
    /// Files directly at the root.
    pub root_files: Vec<FileDescriptor>,
    /// Root folders that are no content folder (`wrapper/`).
    pub root_folders: Vec<ScopePath>,
    /// Files directly inside `Players/`, `Kits/`, `Faces/`, `Boots/`,
    /// `Gloves/` — those folders hold only folders.
    pub stray_files: Vec<FileDescriptor>,
    /// The `Players/` children, one per folder.
    pub players: Vec<FolderDraft>,
    /// The `Faces/` children, one per folder.
    pub faces: Vec<FolderDraft>,
    /// The `Boots/` children, one per folder.
    pub boots: Vec<FolderDraft>,
    /// The `Gloves/` children, one per folder.
    pub gloves: Vec<FolderDraft>,
    /// The `Kits/` children, one per folder (`all/` included).
    pub kits: Vec<FolderDraft>,
    /// Every file below `Portraits/`.
    pub portraits: Vec<FileDescriptor>,
    /// Every file below `Collars/`.
    pub collars: Vec<FileDescriptor>,
    /// Every file below `Common/`.
    pub common: Vec<FileDescriptor>,
}

impl AestheticsExportDraft {
    /// `Referees` exactly when the team name is `/refs/`.
    pub fn kind(&self) -> ExportKind {
        if self.team_name.as_ref().is_some_and(TeamName::is_referees) {
            ExportKind::Referees
        } else {
            ExportKind::Team
        }
    }
}

/// One child folder of a content folder, structure only: its files are every
/// file below it, reserved subfolders included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderDraft {
    /// The folder's canonical path (`Players/03 - A`, `Kits/p1 - Lakers`).
    pub path: ScopePath,
    /// Every file below it.
    pub files: Vec<FileDescriptor>,
}

/// What kind of export the draft is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExportKind {
    /// A normal team export.
    Team,
    /// A referee export (`/refs/`).
    Referees,
}

/// The authoritative roster file: `players.txt`, or a referee export's
/// `refs.txt` alias when it has no `players.txt`. A file that is not UTF-8
/// has no entries (`players_txt_invalid`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRoster {
    /// The roster file's canonical path.
    pub file: ScopePath,
    /// The nonblank lines, in file order.
    pub entries: Vec<RawRosterEntry>,
}

/// One roster line, kept as parsed even when it is malformed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRosterEntry {
    /// The 1-based line number.
    pub line: usize,
    /// The leading decimal, range unchecked; `None` when none parsed.
    pub slot: Option<u16>,
    /// The trimmed remainder; `None` on a bare slot.
    pub folder_name: Option<String>,
}

/// One source file's descriptor: names, sizes and its classified kind; its
/// contents load later, per task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDescriptor {
    /// The canonical virtual path within the (normalized) export.
    pub path: ScopePath,
    /// The same file's path in the source, for reading it; differs from
    /// `path` only under a flattened layer.
    pub source: ScopePath,
    /// The file's size in bytes.
    pub size: u64,
    /// Classified from the name alone ("Structure pass types").
    pub kind: FileKind,
}
