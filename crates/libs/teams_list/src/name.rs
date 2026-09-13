//! The canonical `/xx/` team name.

use std::fmt;

/// The canonical `/xx/` team name: the one fold, applied identically to
/// export names and to the list's `Name` column.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TeamName(String);

impl TeamName {
    /// Trims whitespace, strips one leading and one trailing `/` if present,
    /// lowercases (Unicode `str::to_lowercase`), wraps in slashes. Rejects an
    /// empty result and a token that still contains `/` or whitespace.
    pub fn new(token: &str) -> Result<TeamName, TeamNameError> {
        let mut token = token.trim();
        token = token.strip_prefix('/').unwrap_or(token);
        token = token.strip_suffix('/').unwrap_or(token);
        let folded = token.to_lowercase();
        if folded.is_empty() {
            return Err(TeamNameError::Empty);
        }
        if folded.contains('/') || folded.chars().any(char::is_whitespace) {
            return Err(TeamNameError::InvalidCharacter {
                token: token.to_owned(),
            });
        }
        Ok(TeamName(format!("/{folded}/")))
    }

    /// The folded, slash-wrapped form, e.g. `/co/`.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// `/refs/` — the referee-export key.
    pub fn is_referees(&self) -> bool {
        self.0 == "/refs/"
    }

    /// `/balls/` — the balls-export key.
    pub fn is_balls(&self) -> bool {
        self.0 == "/balls/"
    }

    /// Either reserved key (`/refs/` or `/balls/`).
    pub fn is_reserved(&self) -> bool {
        self.is_referees() || self.is_balls()
    }
}

impl fmt::Display for TeamName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why a token cannot fold to a team name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TeamNameError {
    /// Nothing left after trimming and stripping slashes.
    #[error("empty team name")]
    Empty,
    /// The token still contains `/` or whitespace after folding.
    #[error("invalid team name character: {token:?}")]
    InvalidCharacter {
        /// The offending token after trimming and slash-stripping.
        token: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_to_canonical_form() {
        assert_eq!(TeamName::new("co").unwrap().as_str(), "/co/");
        assert_eq!(TeamName::new("/CO/").unwrap().as_str(), "/co/");
        assert_eq!(TeamName::new(" co ").unwrap().as_str(), "/co/");
        assert_eq!(TeamName::new("/umaJP/").unwrap().as_str(), "/umajp/");
    }

    #[test]
    fn rejects_empty_and_invalid() {
        assert_eq!(TeamName::new(""), Err(TeamNameError::Empty));
        assert_eq!(TeamName::new("  //  "), Err(TeamNameError::Empty));
        assert!(matches!(
            TeamName::new("a b"),
            Err(TeamNameError::InvalidCharacter { .. })
        ));
        assert!(matches!(
            TeamName::new("a/b"),
            Err(TeamNameError::InvalidCharacter { .. })
        ));
        assert!(matches!(
            TeamName::new("/a//"),
            Err(TeamNameError::InvalidCharacter { .. })
        ));
    }

    #[test]
    fn reserved_keys() {
        assert!(TeamName::new("refs").unwrap().is_reserved());
        assert!(TeamName::new("balls").unwrap().is_reserved());
        assert!(TeamName::new("REFS").unwrap().is_referees());
        assert!(!TeamName::new("co").unwrap().is_reserved());
    }
}
