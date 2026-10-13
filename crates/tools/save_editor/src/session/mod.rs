//! The session: the save that is open, and the operations over it that the commands run (and
//! the window will, through its undo stack). egui-free, so the command line and the window
//! cannot diverge (`plans/save_editor.md` "Crate layout"). `pes_savefile` owns every byte and
//! every operation on the model; the session resolves ids against the open save and hands
//! the lib the entries it asks for.

use std::path::Path;

use anyhow::{Context, anyhow};
use pes_savefile::EditFile;
use pes_savefile::interchange::team_toml::TeamToml;
use pes_savefile::model::PlayerEntry;
use pes_version::PesVersion;

/// The open save.
pub(crate) struct Session {
    file: EditFile,
}

impl Session {
    /// Opens the save at `path`. The error names the path and what `pes_savefile` refused.
    pub(crate) fn open(path: &Path) -> anyhow::Result<Session> {
        let file = EditFile::load(path)
            .with_context(|| format!("{}: cannot open the save", path.display()))?;
        Ok(Session { file })
    }

    /// The open save's game version.
    pub(crate) fn version(&self) -> PesVersion {
        self.file.version()
    }

    /// The ids of the save's teams, in record order.
    pub(crate) fn team_ids(&self) -> impl Iterator<Item = u32> {
        self.file.teams().iter().map(|team| team.id)
    }

    /// The Team TOML text of the team with `id`: its entry, tactics and every rostered
    /// player with every field. An id the save lacks is an error naming it.
    pub(crate) fn export_team(&self, id: u32) -> anyhow::Result<String> {
        let team = self
            .file
            .team(id)
            .ok_or_else(|| anyhow!("the save has no team {id}"))?;
        // The whole pool, borrowed: `from_team` picks the roster's players out of it.
        let players: Vec<&PlayerEntry> = self.file.players().iter().collect();
        let document = TeamToml::from_team(self.version(), team, &players)
            .with_context(|| format!("team {id} cannot be exported"))?;
        document
            .to_toml()
            .with_context(|| format!("team {id} cannot be written as Team TOML"))
    }
}
