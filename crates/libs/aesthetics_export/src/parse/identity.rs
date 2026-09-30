//! The export's identity: the canonical team name from the display stem's
//! first token.

use teams_list::TeamName;

/// The first non-empty token of `display_name`, split on `.`, whitespace,
/// `-`, `+` and `_`, folded through `TeamName::new` (it lowercases and wraps
/// in slashes: `co - Spring 2026.zip` → `/co/`). `None` when the stem has no
/// token or the token does not fold.
pub fn team_name(display_name: &str) -> Option<TeamName> {
    let token = display_name
        .split(|c: char| c == '.' || c == '-' || c == '+' || c == '_' || c.is_whitespace())
        .find(|token| !token.is_empty())?;
    TeamName::new(token).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_token_becomes_the_team_name() {
        assert_eq!(
            team_name("co - Spring 2026").map(|name| name.as_str().to_owned()),
            Some("/co/".to_owned())
        );
        assert_eq!(team_name("--- ").map(|name| name.as_str().to_owned()), None);
        assert!(team_name("refs Winter").unwrap().is_referees());
        assert_eq!(
            team_name("_x+y").map(|name| name.as_str().to_owned()),
            Some("/x/".to_owned())
        );
    }
}
