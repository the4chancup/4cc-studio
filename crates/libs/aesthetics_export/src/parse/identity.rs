//! The export's identity, from its display stem's words: the canonical team
//! name from the first, the coverage tag from the second.

use teams_list::TeamName;

/// What a team export covers, from the second word of its name ("Coverage
/// tag"). A referee export carries no tag: it is always `Full`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ExportCoverage {
    /// Everything the team has: the compiler rebuilds the team's records from
    /// it alone.
    Full,
    /// Additions: the compiler replaces only what the export holds.
    Midcup,
}

/// The export's canonical team name, from its display stem (the folder name,
/// or the archive name without extension): the first non-empty token of
/// `display_name`, split on `.`, whitespace, `-`, `+` and `_`, folded through
/// `TeamName::new` (it lowercases and wraps in slashes: `co - Spring 2026` →
/// `/co/`). It is the teams-list key, and the reserved `/refs/` and `/balls/`
/// mark a referee export and a balls export, which a consumer routes on
/// before parsing. `None` when the stem has no token or the token does not
/// fold (an export reported as `team_name_unknown` with an empty name).
pub fn team_name(display_name: &str) -> Option<TeamName> {
    let token = tokens(display_name).next()?;
    TeamName::new(token).ok()
}

/// The coverage tag of a display stem: its second token (split as for
/// `team_name`), `Full` or `Midcup` in any letter case (`co Full Spring
/// 2026`, `co midcup day 5`). `None` for any other second token, a missing
/// one included; a later `Full` (`b Spring Full`) does not count.
pub(crate) fn coverage(display_name: &str) -> Option<ExportCoverage> {
    let tag = tokens(display_name).nth(1)?;
    if tag.eq_ignore_ascii_case("full") {
        Some(ExportCoverage::Full)
    } else if tag.eq_ignore_ascii_case("midcup") {
        Some(ExportCoverage::Midcup)
    } else {
        None
    }
}

/// The non-empty tokens of a display stem, split on `.`, whitespace, `-`, `+`
/// and `_`.
fn tokens(display_name: &str) -> impl Iterator<Item = &str> {
    display_name
        .split(|c: char| c == '.' || c == '-' || c == '+' || c == '_' || c.is_whitespace())
        .filter(|token| !token.is_empty())
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

    #[test]
    fn the_second_token_is_the_coverage_tag_in_any_case() {
        assert_eq!(coverage("co Full Spring 2026"), Some(ExportCoverage::Full));
        assert_eq!(coverage("co midcup day 5"), Some(ExportCoverage::Midcup));
        assert_eq!(coverage("a FULL v2"), Some(ExportCoverage::Full));
        // `team_name`'s separators split the tag off too.
        assert_eq!(coverage("co - Midcup"), Some(ExportCoverage::Midcup));
    }

    #[test]
    fn only_a_whole_second_token_is_a_coverage_tag() {
        assert_eq!(coverage("b Spring Full"), None);
        assert_eq!(coverage("co Spring"), None);
        assert_eq!(coverage("co"), None);
        assert_eq!(coverage("co Fullback"), None);
    }
}
