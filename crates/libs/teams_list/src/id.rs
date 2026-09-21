//! The valid team-id range.

use std::fmt;

/// A team id in the valid `teams_list.txt` range. Referees' fixed 999 is
/// deliberately outside it and is not a `TeamId`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TeamId(u16);

impl TeamId {
    /// First valid id.
    pub const MIN: u16 = 701;
    /// Last valid id.
    pub const MAX: u16 = 920;

    /// Validates `id` against `MIN..=MAX`.
    pub fn new(id: u16) -> Result<TeamId, TeamIdError> {
        if (Self::MIN..=Self::MAX).contains(&id) {
            Ok(TeamId(id))
        } else {
            Err(TeamIdError(id))
        }
    }

    /// The numeric id.
    pub fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for TeamId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An id outside `701..=920`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("team id {0} is outside 701..=920")]
pub struct TeamIdError(pub u16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_is_enforced() {
        assert_eq!(TeamId::new(701).unwrap().get(), 701);
        assert_eq!(TeamId::new(701).unwrap().to_string(), "701");
        assert_eq!(TeamId::new(920).unwrap().get(), 920);
        assert_eq!(TeamId::new(700), Err(TeamIdError(700)));
        assert_eq!(TeamId::new(921), Err(TeamIdError(921)));
        assert_eq!(TeamId::new(999), Err(TeamIdError(999)));
    }
}
