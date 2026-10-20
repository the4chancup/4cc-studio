//! The output folder's name (`spec/export_upgrader.md` "CLI"): the resolved
//! team's key, the coverage tag, then the rest of the old name — so the
//! compiler reads the output's team from its first word.

use aesthetics_export::{coverage, is_name_separator, team_name};

use crate::upgrade::Identity;

/// `name` minus its first token, the separator between kept
/// (`co - Spring` → ` - Spring`).
fn after_first_token(name: &str) -> &str {
    let Some(start) = name.find(|c: char| !is_name_separator(c)) else {
        return "";
    };
    match name[start..].find(is_name_separator) {
        Some(end) => &name[start + end..],
        None => "",
    }
}

/// The output folder's name (`spec/export_upgrader.md` "CLI"): the resolved
/// team's key without its slashes, then the coverage tag — `Midcup` when the
/// old name holds `midcup` or `additions` in any letter case, `Full`
/// otherwise — then the rest of the old name. When the name's first token
/// folds to the key, the rest is the name after that token, its separator
/// kept (`co - Spring` → `co Full - Spring`), and a second token already
/// reading as a tag is kept rather than doubled; otherwise the rest is a
/// space and the whole name (`Numbers` → `co Full Numbers`). A referee export
/// takes no tag. A result equal to `export_name` ignoring ASCII case gets
/// ` upgraded`, so the input is never written into.
pub(crate) fn output_name(export_name: &str, identity: &Identity) -> String {
    let (key, tag) = match identity {
        Identity::Referees => ("refs", None),
        Identity::Team { name, .. } => {
            let lowered = export_name.to_lowercase();
            let tag = if lowered.contains("midcup") || lowered.contains("additions") {
                "Midcup"
            } else {
                "Full"
            };
            (name.as_str().trim_matches('/'), Some(tag))
        }
        Identity::ModelFolders => {
            unreachable!("upgrade's resolve yields a team or the referees")
        }
    };
    let first_is_key = match team_name(export_name) {
        Some(found) => match identity {
            Identity::Team { name, .. } => found == *name,
            Identity::Referees => found.is_referees(),
            Identity::ModelFolders => {
                unreachable!("upgrade's resolve yields a team or the referees")
            }
        },
        None => false,
    };
    let mut output = key.to_owned();
    if first_is_key {
        if coverage(export_name).is_none()
            && let Some(tag) = tag
        {
            output.push(' ');
            output.push_str(tag);
        }
        output.push_str(after_first_token(export_name));
    } else {
        if let Some(tag) = tag {
            output.push(' ');
            output.push_str(tag);
        }
        output.push(' ');
        output.push_str(export_name);
    }
    if output.eq_ignore_ascii_case(export_name) {
        output.push_str(" upgraded");
    }
    output
}

#[cfg(test)]
mod tests {
    use teams_list::{TeamId, TeamName};

    use super::*;

    /// `/co/`'s identity (the acceptance section's test team, id 714).
    fn co() -> Identity {
        Identity::Team {
            id: TeamId::new(714).unwrap(),
            name: TeamName::new("co").unwrap(),
        }
    }

    #[test]
    fn the_tag_is_inserted_after_the_key_with_the_separator_kept() {
        // EU-CLI-01's four names.
        assert_eq!(output_name("co - Spring", &co()), "co Full - Spring");
        assert_eq!(
            output_name("co Day 8 Additions", &co()),
            "co Midcup Day 8 Additions"
        );
        assert_eq!(
            output_name("co midcup Day 5", &co()),
            "co midcup Day 5 upgraded"
        );
        assert_eq!(
            output_name("co Full Spring", &co()),
            "co Full Spring upgraded"
        );
        // A name whose first token is not the key keeps the whole name as the
        // rest.
        assert_eq!(output_name("Numbers", &co()), "co Full Numbers");
        assert_eq!(output_name("day5", &co()), "co Full day5");
        // The first separator is kept with the rest, whatever it is.
        assert_eq!(output_name("co.Spring", &co()), "co Full.Spring");
        assert_eq!(output_name("co+Spring", &co()), "co Full+Spring");
        assert_eq!(output_name("co_Spring", &co()), "co Full_Spring");
        // A leading separator is not part of the rest: the first token is
        // still `co`, and what follows it is ` Spring`.
        assert_eq!(output_name("_co Spring", &co()), "co Full Spring");
        // Case-insensitive equality still collides on Windows, so ` upgraded`
        // is appended.
        assert_eq!(
            output_name("CO Full Spring", &co()),
            "co Full Spring upgraded"
        );
        // A referee export takes no tag.
        assert_eq!(
            output_name("refs Spring", &Identity::Referees),
            "refs Spring upgraded"
        );
        assert_eq!(
            output_name("refs Cup", &Identity::Referees),
            "refs Cup upgraded"
        );
    }
}
