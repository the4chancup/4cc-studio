//! Which players the master list shows: the team the selector names, or every
//! team while the filter box is in use (`plans/save_editor.md` "Editing UI",
//! the Player list bullet).

use std::collections::HashMap;

use pes_savefile::EditFile;
use pes_savefile::model::names::display_name;
use pes_savefile::model::{PlayerEntry, TeamEntry};

/// Which team the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TeamChoice {
    /// Every team of the save, in the save's own order.
    All,
    /// One team, by id.
    Team(u32),
}

/// One listed row: the player's team, roster slot and id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Row {
    /// The team the player is rostered on.
    pub team_id: u32,
    /// His roster slot, the 0-based index into the team's roster.
    pub slot: u8,
    /// His player id.
    pub player_id: u32,
}

/// The rows the list shows: a non-empty `filter` matches across all teams
/// (whatever the choice) on the display name, the shirt name or the id,
/// case-insensitive, in team then slot order; otherwise the chosen team's
/// roster, or every team's for `All`. Empty slots (player id 0) are never
/// listed.
pub(crate) fn rows(file: &EditFile, team: &TeamChoice, filter: &str) -> Vec<Row> {
    let query = filter.trim().to_lowercase();
    let filtering = !query.is_empty();
    // Player ids are stable inside an open file: one map serves the whole
    // scan instead of a `file.player()` linear scan per slot.
    let players: HashMap<u32, &PlayerEntry> = file
        .players()
        .iter()
        .map(|player| (player.id, player))
        .collect();
    let mut rows = Vec::new();
    for team_entry in file.teams() {
        if !filtering
            && let TeamChoice::Team(id) = team
            && *id != team_entry.id
        {
            continue;
        }
        rows.extend(team_rows(team_entry, &players, &query));
    }
    rows
}

/// The rostered slots of `team`, in slot order; with `query` non-empty only
/// the players matching it.
fn team_rows(team: &TeamEntry, players: &HashMap<u32, &PlayerEntry>, query: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    for (slot, roster_slot) in team.roster.iter().enumerate() {
        if roster_slot.player_id == 0 {
            continue;
        }
        if !query.is_empty()
            && !players
                .get(&roster_slot.player_id)
                .is_some_and(|player| matches(player, query))
        {
            continue;
        }
        rows.push(Row {
            team_id: team.id,
            slot: u8::try_from(slot).expect("a roster is at most 40 slots"),
            player_id: roster_slot.player_id,
        });
    }
    rows
}

/// Whether `player` answers the trimmed, lowercased `query`: a substring of
/// his display name, shirt name or id as text.
fn matches(player: &PlayerEntry, query: &str) -> bool {
    display_name(&player.name).to_lowercase().contains(query)
        || player.shirt_name.to_lowercase().contains(query)
        || player.id.to_string().contains(query)
}

#[cfg(test)]
mod tests {
    use pes_savefile::fixture_save;
    use pes_version::PesVersion;

    use super::*;

    fn open(version: PesVersion) -> EditFile {
        EditFile::from_bytes(&fixture_save(version)).unwrap()
    }

    /// The rostered slots of `team_id` as the file lists them, for comparing
    /// `rows`' output with.
    fn rostered(file: &EditFile, team_id: u32) -> Vec<Row> {
        file.team(team_id)
            .unwrap()
            .roster
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.player_id != 0)
            .map(|(slot, roster_slot)| Row {
                team_id,
                slot: u8::try_from(slot).unwrap(),
                player_id: roster_slot.player_id,
            })
            .collect()
    }

    #[test]
    fn a_team_lists_its_rostered_slots_in_order() {
        let file = open(PesVersion::Pes17);
        assert_eq!(
            rows(&file, &TeamChoice::Team(718), ""),
            rostered(&file, 718)
        );
        // Team 718's first slot is player 71801; a team the save lacks lists nothing.
        assert_eq!(rows(&file, &TeamChoice::Team(718), "")[0].player_id, 71801);
        assert!(rows(&file, &TeamChoice::Team(99999), "").is_empty());
    }

    #[test]
    fn all_lists_every_teams_roster_in_save_order() {
        let file = open(PesVersion::Pes17);
        let expected: Vec<Row> = file
            .teams()
            .iter()
            .flat_map(|team| rostered(&file, team.id))
            .collect();
        assert_eq!(rows(&file, &TeamChoice::All, ""), expected);
    }

    #[test]
    fn a_filter_matches_the_display_name_across_all_teams() {
        let file = open(PesVersion::Pes17);
        // 71801's raw name carries colour codes; the match is on the
        // code-free display name, case-insensitive.
        assert!(file.player(71801).unwrap().name.contains('\x11'));
        assert_eq!(
            rows(&file, &TeamChoice::Team(701), "roadtrip"),
            [Row {
                team_id: 718,
                slot: 0,
                player_id: 71801,
            }]
        );
        assert!(rows(&file, &TeamChoice::All, "ROADTRIP").len() == 1);
        // A whitespace-only filter is no filter: the chosen team's roster.
        assert_eq!(
            rows(&file, &TeamChoice::Team(701), "   "),
            rostered(&file, 701)
        );
    }

    #[test]
    fn a_filter_matches_the_id_and_the_shirt_name() {
        let file = open(PesVersion::Pes17);
        assert_eq!(
            rows(&file, &TeamChoice::Team(701), "71801"),
            [Row {
                team_id: 718,
                slot: 0,
                player_id: 71801,
            }]
        );
        assert_eq!(
            rows(&file, &TeamChoice::Team(701), "exact change"),
            [Row {
                team_id: 718,
                slot: 0,
                player_id: 71801,
            }]
        );
        // No match: nothing, not the chosen team's roster.
        assert!(rows(&file, &TeamChoice::Team(701), "no such player").is_empty());
    }
}
