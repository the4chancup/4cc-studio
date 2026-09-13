//! The `teams_list.txt` file: header, team rows, inert placeholders.
//!
//! Contract: tab-separated, a required header line whose fields are preserved
//! verbatim on write, `ID` and `Name` located by header position in either
//! order, every line's cells carried verbatim, the `Name` column folded
//! through [`TeamName::new`] on load, lines whose `Name` does not fold kept
//! verbatim as placeholders and never looked up, BOM tolerated on read and
//! never written, blank lines dropped, CRLF or LF read and CRLF always
//! written. A placeholder whose `ID` cell holds a number in 701..=920 still
//! claims that id: the file rejects any two rows sharing one.

use crate::id::TeamId;
use crate::name::TeamName;

/// One line of the file, in file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A team: the line's cells plus the validated id and folded name read
    /// from the id and name columns.
    Team {
        /// The line's raw tab-separated cells.
        cells: Vec<String>,
        /// The validated team id.
        id: TeamId,
        /// The folded, canonical name (the lookup key).
        name: TeamName,
    },
    /// A line whose `Name` cell does not fold (e.g. `Backup 1`,
    /// `Invitational 68`): kept verbatim and never looked up.
    Placeholder {
        /// The line's raw tab-separated cells.
        cells: Vec<String>,
    },
}

/// A parsed `teams_list.txt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamsList {
    header: Vec<String>,
    id_column: usize,
    name_column: usize,
    rows: Vec<Row>,
}

impl TeamsList {
    /// The list shipped with this build (`data/teams_list.txt`).
    pub const UPSTREAM: &'static str = include_str!("../data/teams_list.txt");

    /// Parses the file text. See the module docs for the contract.
    pub fn parse(text: &str) -> Result<TeamsList, TeamsListError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);

        let mut header: Option<Vec<String>> = None;
        let mut id_column = 0usize;
        let mut name_column = 0usize;
        let mut rows = Vec::new();
        let mut seen_ids = std::collections::BTreeMap::new();
        let mut seen_names = std::collections::BTreeMap::new();

        for (index, raw_line) in text.split('\n').enumerate() {
            let line = index + 1;
            let raw_line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            if raw_line.trim().is_empty() {
                continue;
            }
            let cells: Vec<String> = raw_line.split('\t').map(str::to_owned).collect();
            if header.is_none() {
                id_column = cells
                    .iter()
                    .position(|f| f == "ID")
                    .ok_or(TeamsListError::MissingColumn("ID"))?;
                name_column = cells
                    .iter()
                    .position(|f| f == "Name")
                    .ok_or(TeamsListError::MissingColumn("Name"))?;
                header = Some(cells);
                continue;
            }

            let raw_name = cells.get(name_column).cloned().unwrap_or_default();
            let id_text = cells.get(id_column).cloned().unwrap_or_default();
            let id = id_text
                .parse::<u16>()
                .ok()
                .and_then(|value| TeamId::new(value).ok());
            match TeamName::new(&raw_name) {
                Ok(name) => {
                    let id = id.ok_or_else(|| TeamsListError::InvalidId {
                        line,
                        text: id_text.clone(),
                    })?;
                    if seen_ids.insert(id, line).is_some() {
                        return Err(TeamsListError::DuplicateId { line, id: id.get() });
                    }
                    if seen_names.insert(name.clone(), line).is_some() {
                        return Err(TeamsListError::DuplicateName {
                            line,
                            name: name.as_str().to_owned(),
                        });
                    }
                    rows.push(Row::Team { cells, id, name });
                }
                Err(_) => {
                    // A placeholder's numeric in-range id is still part of
                    // the id space.
                    if let Some(id) = id
                        && seen_ids.insert(id, line).is_some()
                    {
                        return Err(TeamsListError::DuplicateId { line, id: id.get() });
                    }
                    rows.push(Row::Placeholder { cells });
                }
            }
        }

        let header = header.ok_or(TeamsListError::MissingHeader)?;
        Ok(TeamsList {
            header,
            id_column,
            name_column,
            rows,
        })
    }

    /// Header + rows, tab-separated, CRLF line ends, no BOM — exactly the
    /// input for an unmodified list.
    pub fn write(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.header.join("\t"));
        output.push_str("\r\n");
        for row in &self.rows {
            let cells = match row {
                Row::Team { cells, .. } | Row::Placeholder { cells } => cells,
            };
            output.push_str(&cells.join("\t"));
            output.push_str("\r\n");
        }
        output
    }

    /// The id a name maps to, if it is a team row.
    pub fn id_of(&self, name: &TeamName) -> Option<TeamId> {
        self.teams()
            .find(|(_, row_name)| *row_name == name)
            .map(|(id, _)| id)
    }

    /// The name an id maps to, if it is a team row.
    pub fn name_of(&self, id: TeamId) -> Option<&TeamName> {
        self.teams()
            .find(|(row_id, _)| *row_id == id)
            .map(|(_, name)| name)
    }

    /// The team rows in file order.
    pub fn teams(&self) -> impl Iterator<Item = (TeamId, &TeamName)> {
        self.rows.iter().filter_map(|row| match row {
            Row::Team { id, name, .. } => Some((*id, name)),
            Row::Placeholder { .. } => None,
        })
    }

    /// Every row in file order, placeholders included.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub(crate) fn from_parts(
        header: Vec<String>,
        id_column: usize,
        name_column: usize,
        rows: Vec<Row>,
    ) -> TeamsList {
        TeamsList {
            header,
            id_column,
            name_column,
            rows,
        }
    }

    pub(crate) fn header(&self) -> &[String] {
        &self.header
    }

    pub(crate) fn id_column(&self) -> usize {
        self.id_column
    }

    pub(crate) fn name_column(&self) -> usize {
        self.name_column
    }
}

/// Why a `teams_list.txt` text cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TeamsListError {
    /// The file has no non-blank line to take as the header.
    #[error("missing teams list header")]
    MissingHeader,
    /// The header lacks the named column.
    #[error("missing teams list column {0}")]
    MissingColumn(&'static str),
    /// The `ID` column is not a number in 701..=920 on a line whose `Name`
    /// folds.
    #[error("invalid team id {text:?} on line {line}")]
    InvalidId {
        /// 1-based line number in the input.
        line: usize,
        /// The raw `ID` field text.
        text: String,
    },
    /// Two rows share an id, teams and numeric placeholders alike.
    #[error("duplicate team id {id} on line {line}")]
    DuplicateId {
        /// 1-based line number in the input.
        line: usize,
        /// The repeated id.
        id: u16,
    },
    /// Two team rows share a folded name.
    #[error("duplicate team name {name} on line {line}")]
    DuplicateName {
        /// 1-based line number in the input.
        line: usize,
        /// The repeated folded name.
        name: String,
    },
}
