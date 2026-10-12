//! `TeamView`/`PlayerView`: the host's read of one team of a save, every
//! value on the game's own scale, plus the tactics answers
//! (`fielded_positions`, `fielded_only_at`, `formation_shapes`).

use pes_savefile::EditFile;
use pes_savefile::interchange::team_toml::TeamToml;
use pes_savefile::interchange::team_toml::labels::POSITIONS;
use pes_savefile::model::names::name_color;
use pes_savefile::model::player::PlayerEntry;
use pes_savefile::model::playstyle::PlayStyle as SavePlayStyle;
use pes_savefile::model::tactics::{Formation, TeamTactics};
use pes_savefile::model::team::RosterSlot;
use pes_savefile::model::team::TeamEntry;
use pes_savefile::schema::playstyle;
use pes_savefile::settings_toml::AppearanceSettings;
use pes_version::PesVersion;

use crate::ruleset::{Card, PlayStyle, Position, Stat};

/// One roster player as a ruleset reads them, every value on the game's
/// scale (form 1 to 8, injury resistance 1 to 3, weak foot 1 to 4, where
/// the save stores each less one).
#[derive(Debug, Clone, PartialEq)]
pub struct PlayerView {
    /// The roster slot.
    pub(crate) slot: u8,
    /// Age in years.
    pub(crate) age: u8,
    /// Height in cm.
    pub(crate) height: u8,
    /// Weight in kg.
    pub(crate) weight: u8,
    /// Form, 1 to 8.
    pub(crate) form: u8,
    /// Injury resistance, 1 to 3.
    pub(crate) injury_resistance: u8,
    /// Weak foot usage, 1 to 4.
    pub(crate) weak_foot_usage: u8,
    /// Weak foot accuracy, 1 to 4.
    pub(crate) weak_foot_accuracy: u8,
    /// The registered position (`None` past the labels).
    pub(crate) position: Option<Position>,
    /// The playing style (`None` when the version's table has no such
    /// style).
    pub(crate) playing_style: Option<PlayStyle>,
    /// The name's leading colour code (`cc9900ff`).
    pub(crate) name_color: Option<String>,
    /// The held cards: the skills in table order, then the COM styles.
    pub(crate) cards: Vec<Card>,
    /// The ability stats (`labels::ABILITY_STATS`' order, indexed by
    /// `Stat`), `None` where the version lacks the stat.
    pub(crate) abilities: [Option<u8>; 25],
    /// The playable-position ratings, `Position`-indexed (GK 0 to CF 12;
    /// `PlayerPositions::playable` runs CF 0 to GK 12).
    pub(crate) ratings: [u8; 13],
}

impl PlayerView {
    /// The ability stat `name` names (`None` for a stat the version lacks).
    pub(crate) fn stat_by_name(&self, name: &str) -> Result<Option<u8>, String> {
        Stat::try_from(name.to_string()).map(|stat| self.abilities[usize::from(stat.0)])
    }

    /// The playable rating at `pos` (0 none to 3 A).
    pub(crate) fn rating_at(&self, pos: &str) -> Result<u8, String> {
        Position::try_from(pos.to_string()).map(|pos| self.ratings[usize::from(pos.0)])
    }
}

/// One team of a save as a ruleset reads it: its roster's players and its
/// tactics.
#[derive(Debug, Clone, PartialEq)]
pub struct TeamView {
    /// The save's version.
    pub(crate) version: PesVersion,
    /// The roster's players in slot order, empty slots left out.
    pub(crate) players: Vec<PlayerView>,
    /// The captain's roster slot.
    pub(crate) captain: Option<u8>,
    /// The team's tactics (starting eleven, presets, formations).
    pub(crate) tactics: TeamTactics,
}

/// Why a team cannot be viewed.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TeamViewError {
    /// The save holds no team with this id.
    #[error("unknown team {team_id}")]
    UnknownTeam {
        /// The id looked up.
        team_id: u32,
    },
    /// A roster slot names a player the save does not hold.
    #[error("roster slot {slot} names player {player_id}, whom the save does not hold")]
    MissingPlayer {
        /// The roster slot.
        slot: u8,
        /// The missing player id.
        player_id: u32,
    },
    /// A formation slot's position byte is past the position labels.
    #[error(
        "preset {preset} formation {formation} slot {slot}: position byte {byte} is past the labels"
    )]
    FormationPosition {
        /// The preset, numbered from 1.
        preset: u8,
        /// The formation within it, numbered from 1.
        formation: u8,
        /// The formation slot (0 to 10).
        slot: u8,
        /// The position byte.
        byte: u8,
    },
    /// The document names no `pes_version` to build the team on.
    #[error("the document names no pes_version")]
    NoVersion,
    /// The document does not apply onto bare records: `message` is the
    /// `TeamTomlError` or the first import note.
    #[error("the document does not apply: {message}")]
    TeamToml {
        /// The error's or note's text.
        message: String,
    },
}

/// The defender/midfielder/forward counts of one formation the game can
/// field, presets and formations numbered from 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FormationShape {
    /// The preset (1 to 3).
    pub(crate) preset: u8,
    /// The formation within it (1 to 3).
    pub(crate) formation: u8,
    /// `[defenders, midfielders, forwards]`; GK counts in no group.
    pub(crate) shape: [u8; 3],
}

impl TeamView {
    /// The team `team_id` of `file`.
    pub fn new(file: &EditFile, team_id: u32) -> Result<TeamView, TeamViewError> {
        let team = file
            .team(team_id)
            .ok_or(TeamViewError::UnknownTeam { team_id })?;
        Self::view(file.version(), team, |id| file.player(id))
    }

    /// The team a Team TOML document describes, on the version its
    /// `pes_version` names: every `players` key one bare slot.
    pub fn from_team_toml(document: &TeamToml) -> Result<TeamView, TeamViewError> {
        let version = document.pes_version.ok_or(TeamViewError::NoVersion)?;
        // Appearance is not applied: no ruleset reads it, and it lives
        // only over a real player's face data.
        let mut document = document.clone();
        for section in document.players.values_mut() {
            section.appearance = AppearanceSettings::default();
        }
        // Bare records: one roster slot per index up to the highest
        // `players` key, `i + 1` where the key is present, and one bare
        // `PlayerEntry` per present slot.
        let mut team = TeamEntry::default();
        let highest = document.players.keys().max().copied().unwrap_or(0);
        team.roster = (1..=highest)
            .map(|slot| RosterSlot {
                player_id: if document.players.contains_key(&slot) {
                    u32::from(slot)
                } else {
                    0
                },
                ..Default::default()
            })
            .collect();
        let mut players: Vec<PlayerEntry> = document
            .players
            .keys()
            .map(|slot| PlayerEntry {
                id: u32::from(*slot),
                ..Default::default()
            })
            .collect();
        let notes = document
            .apply(version, &mut team, &mut players)
            .map_err(|e| TeamViewError::TeamToml {
                message: e.to_string(),
            })?;
        if let Some(note) = notes.first() {
            return Err(TeamViewError::TeamToml {
                message: format!("{note:?}"),
            });
        }
        Self::view(version, &team, |id| {
            players.iter().find(|player| player.id == id)
        })
    }

    /// The builder `new` and `from_team_toml` share: a version, a
    /// `&TeamEntry` and a player lookup by id.
    fn view<'a>(
        version: PesVersion,
        team: &'a TeamEntry,
        player: impl Fn(u32) -> Option<&'a PlayerEntry>,
    ) -> Result<TeamView, TeamViewError> {
        // Every formation's position bytes are checked here so the
        // tactics answers cannot fail on one.
        for (p, preset) in team.tactics.presets.iter().enumerate() {
            for (f, formation) in preset.formations.iter().enumerate() {
                for (s, slot) in formation.players.iter().enumerate() {
                    if usize::from(slot.position) >= POSITIONS.len() {
                        return Err(TeamViewError::FormationPosition {
                            preset: u8::try_from(p + 1).expect("three presets"),
                            formation: u8::try_from(f + 1).expect("three formations"),
                            slot: u8::try_from(s).expect("eleven slots"),
                            byte: slot.position,
                        });
                    }
                }
            }
        }
        let mut players = Vec::new();
        for (i, slot) in team.roster.iter().enumerate() {
            if slot.player_id == 0 {
                continue;
            }
            let entry = player(slot.player_id).ok_or(TeamViewError::MissingPlayer {
                slot: u8::try_from(i).expect("a roster holds fewer than 256 slots"),
                player_id: slot.player_id,
            })?;
            // A style byte past the version's table is no style (`()` in a
            // script), not an error: the logic reports it as out of range.
            let playing_style = playstyle::decode(version, entry.positions.playing_style)
                .ok()
                .and_then(|style| {
                    SavePlayStyle::ALL
                        .iter()
                        .position(|s| *s == style)
                        .map(|i| PlayStyle(u8::try_from(i).expect("22 styles")))
                });
            let mut ratings = [0u8; 13];
            for (p, rating) in ratings.iter_mut().enumerate() {
                *rating = entry.positions.playable[12 - p];
            }
            players.push(PlayerView {
                slot: u8::try_from(i).expect("a roster holds fewer than 256 slots"),
                age: entry.basic.age,
                height: entry.basic.height,
                weight: entry.basic.weight,
                form: entry.stats.form.saturating_add(1),
                injury_resistance: entry.stats.injury_resistance.saturating_add(1),
                weak_foot_usage: entry.stats.weak_foot_usage.saturating_add(1),
                weak_foot_accuracy: entry.stats.weak_foot_accuracy.saturating_add(1),
                position: (usize::from(entry.positions.registered) < POSITIONS.len())
                    .then_some(Position(entry.positions.registered)),
                playing_style,
                name_color: name_color(&entry.name).map(str::to_string),
                cards: entry
                    .skills
                    .skills
                    .iter()
                    .enumerate()
                    .filter(|(_, held)| **held)
                    .map(|(i, _)| Card::Skill(u8::try_from(i).expect("41 skills")))
                    .chain(
                        entry
                            .skills
                            .com_styles
                            .iter()
                            .enumerate()
                            .filter(|(_, held)| **held)
                            .map(|(i, _)| Card::ComStyle(u8::try_from(i).expect("7 styles"))),
                    )
                    .collect(),
                abilities: entry.stats.abilities(),
                ratings,
            });
        }
        let captain =
            (team.tactics.set_pieces.captain != 0xFF).then_some(team.tactics.set_pieces.captain);
        Ok(TeamView {
            version,
            players,
            captain,
            tactics: team.tactics.clone(),
        })
    }

    /// The sample team `Rules::load` step 3 runs `check_team` on: a
    /// real, anonymized cup team as a Team TOML document
    /// (`sample/team.toml`).
    pub(crate) fn sample_team() -> TeamView {
        TeamView::from_team_toml(
            &TeamToml::parse(include_str!("../sample/team.toml")).expect("the sample team parses"),
        )
        .expect("the sample team builds a view")
    }

    /// The formations `preset` (1 to 3) can field, as
    /// `(formation number, formation)` numbered from 1: all three when
    /// fluid, the kick-off one otherwise.
    fn fielded_formations(&self, preset: u8) -> Vec<(u8, &Formation)> {
        let preset_ref = &self.tactics.presets[usize::from(preset) - 1];
        let fielded: &[usize] = match preset_ref.style.fluid {
            Some(true) => &[0, 1, 2],
            _ => &[0],
        };
        fielded
            .iter()
            .map(|&f| {
                (
                    u8::try_from(f + 1).expect("three formations"),
                    &preset_ref.formations[f],
                )
            })
            .collect()
    }

    /// The positions the starting-eleven player at roster `slot` is
    /// fielded at in `preset`'s (1 to 3) fielded formations, once each,
    /// in the position table's order (GK first, CF last); empty for a
    /// player outside the starting eleven.
    pub(crate) fn fielded_positions(&self, slot: u8, preset: u8) -> Vec<Position> {
        let mut positions = Vec::new();
        for (_, formation) in self.fielded_formations(preset) {
            for (i, &starter) in self.tactics.starting_eleven.iter().enumerate() {
                if starter == slot {
                    let position = Position(formation.players[i].position);
                    if !positions.contains(&position) {
                        positions.push(position);
                    }
                }
            }
        }
        positions.sort();
        positions
    }

    /// Whether every formation the game can field fields the player at
    /// `position`; for a player outside the starting eleven, whether
    /// `position` is his registered one.
    pub(crate) fn fielded_only_at(&self, slot: u8, position: Position) -> bool {
        if !self.tactics.starting_eleven.contains(&slot) {
            return self
                .players
                .iter()
                .find(|player| player.slot == slot)
                .is_some_and(|player| player.position == Some(position));
        }
        for preset in 1..=3 {
            for (_, formation) in self.fielded_formations(preset) {
                for (i, &starter) in self.tactics.starting_eleven.iter().enumerate() {
                    if starter == slot && Position(formation.players[i].position) != position {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// One `FormationShape` per formation the game can field, in preset
    /// then formation order.
    pub(crate) fn formation_shapes(&self) -> Vec<FormationShape> {
        let mut out = Vec::new();
        for p in 1..=3u8 {
            for (f, formation) in self.fielded_formations(p) {
                let mut shape = [0u8; 3];
                for slot in &formation.players {
                    // `POSITIONS` bytes: CB, LB, RB; DMF, CMF, LMF, RMF,
                    // AMF; LWF, RWF, SS, CF. `new` refused any byte past
                    // them, so `_` is the GK.
                    match slot.position {
                        1..=3 => shape[0] += 1,
                        4..=8 => shape[1] += 1,
                        9..=12 => shape[2] += 1,
                        _ => {}
                    }
                }
                out.push(FormationShape {
                    preset: p,
                    formation: f,
                    shape,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pes_savefile::fixture_save;
    use pes_savefile::model::TeamEntry;

    /// `version`'s fixture save and its team 701's view.
    fn view(version: PesVersion) -> (EditFile, TeamView) {
        let file = EditFile::from_bytes(&fixture_save(version)).expect("fixture save");
        let team = TeamView::new(&file, 701).expect("team 701 views");
        (file, team)
    }

    /// `file`'s team 701, mutable.
    fn team_mut(file: &mut EditFile) -> &mut TeamEntry {
        file.teams_mut()
            .iter_mut()
            .find(|team| team.id == 701)
            .expect("team 701")
    }

    #[test]
    fn an_unknown_team_is_refused() {
        let file = EditFile::from_bytes(&fixture_save(PesVersion::Pes19)).expect("fixture save");
        assert_eq!(
            TeamView::new(&file, 9999).expect_err("no such team"),
            TeamViewError::UnknownTeam { team_id: 9999 }
        );
    }

    #[test]
    fn a_roster_slot_naming_no_player_is_refused() {
        let mut file =
            EditFile::from_bytes(&fixture_save(PesVersion::Pes19)).expect("fixture save");
        assert_ne!(team_mut(&mut file).roster[5].player_id, 9999);
        team_mut(&mut file).roster[5].player_id = 9999;
        assert_eq!(
            TeamView::new(&file, 701).expect_err("a missing player"),
            TeamViewError::MissingPlayer {
                slot: 5,
                player_id: 9999
            }
        );
    }

    #[test]
    fn a_formation_byte_past_the_labels_is_refused() {
        let mut file =
            EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).expect("fixture save");
        team_mut(&mut file).tactics.presets[0].formations[2].players[5].position = 13;
        assert_eq!(
            TeamView::new(&file, 701).expect_err("a position past the labels"),
            TeamViewError::FormationPosition {
                preset: 1,
                formation: 3,
                slot: 5,
                byte: 13
            }
        );
    }

    #[test]
    fn the_roster_views_in_slot_order() {
        let (_, team) = view(PesVersion::Pes19);
        assert_eq!(team.players.len(), 23);
        assert_eq!(
            team.players.iter().map(|p| p.slot).collect::<Vec<_>>(),
            (0..23).collect::<Vec<u8>>()
        );
    }

    #[test]
    fn fielded_positions_reads_only_fielded_formations() {
        let (mut file, team) = view(PesVersion::Pes19);
        // Slot 8 sits at formation slot 8, AMF (byte 8) everywhere.
        assert_eq!(team.tactics.presets[0].formations[1].players[8].position, 8);
        assert_eq!(team.tactics.presets[0].formations[2].players[8].position, 8);
        // Non-fluid: only the kick-off formation counts.
        assert_eq!(team.fielded_positions(8, 1), vec![Position(8)]);
        // Formations 2 and 3 both put him at LMF: the position reports
        // once, not twice.
        team_mut(&mut file).tactics.presets[0].formations[1].players[8].position = 6;
        team_mut(&mut file).tactics.presets[0].formations[2].players[8].position = 6;
        let team = TeamView::new(&file, 701).expect("the team views");
        assert_eq!(team.fielded_positions(8, 1), vec![Position(8)]);
        // The same preset made fluid: all three formations, each
        // position once (LMF 6 twice becomes once), ascending.
        team_mut(&mut file).tactics.presets[0].style.fluid = Some(true);
        let team = TeamView::new(&file, 701).expect("the team views");
        assert_eq!(team.fielded_positions(8, 1), vec![Position(6), Position(8)]);
        // A substitute: nothing.
        assert!(!team.tactics.starting_eleven.contains(&20));
        assert_eq!(team.fielded_positions(20, 1), Vec::<Position>::new());
    }

    #[test]
    fn fielded_only_at_needs_every_fielded_formation() {
        let (mut file, team) = view(PesVersion::Pes19);
        // UTAH TEAPOT (slot 10) is the CF of every formation.
        let cf = Position(12);
        assert!(team.fielded_only_at(10, cf));
        // A fielded formation moved: made fluid, formation 3 moves him
        // to SS (byte 11).
        team_mut(&mut file).tactics.presets[0].style.fluid = Some(true);
        team_mut(&mut file).tactics.presets[0].formations[2].players[10].position = 11;
        let team = TeamView::new(&file, 701).expect("the team views");
        assert!(!team.fielded_only_at(10, cf));
        // A non-fielded formation moved (a non-fluid preset's formation
        // 2): still true.
        let (mut file, _) = view(PesVersion::Pes19);
        team_mut(&mut file).tactics.presets[0].formations[1].players[10].position = 11;
        let team = TeamView::new(&file, 701).expect("the team views");
        assert!(team.fielded_only_at(10, cf));
        // Preset 3's kick-off formation: fielded even though preset 3 is
        // not fluid — moving him there breaks it too.
        let (mut file, _) = view(PesVersion::Pes19);
        assert_eq!(
            team_mut(&mut file).tactics.presets[2].style.fluid,
            Some(false)
        );
        team_mut(&mut file).tactics.presets[2].formations[0].players[10].position = 11;
        let team = TeamView::new(&file, 701).expect("the team views");
        assert!(!team.fielded_only_at(10, cf));
        // A substitute: his registered position, nothing else.
        assert!(!team.tactics.starting_eleven.contains(&20));
        let entry = file.player(70121).expect("player 70121");
        let registered_pos = Position(entry.positions.registered);
        let elsewhere = Position((entry.positions.registered + 1) % 13);
        assert!(team.fielded_only_at(20, registered_pos));
        assert!(!team.fielded_only_at(20, elsewhere));
    }

    #[test]
    fn formation_shapes_counts_every_fielded_formation() {
        let (mut file, team) = view(PesVersion::Pes17);
        // Non-fluid everywhere: one formation per preset.
        assert_eq!(team.formation_shapes().len(), 3);
        // Team 701's kick-off: GK, two CB, RB, LB, two DMF, RMF, LMF,
        // AMF, CF — GK counts in no group.
        assert_eq!(
            team.formation_shapes()[0],
            FormationShape {
                preset: 1,
                formation: 1,
                shape: [4, 5, 1]
            }
        );
        // A fluid preset adds its three formations.
        team_mut(&mut file).tactics.presets[0].style.fluid = Some(true);
        let team = TeamView::new(&file, 701).expect("the team views");
        assert_eq!(team.formation_shapes().len(), 5);
        assert_eq!(
            team.formation_shapes()[2],
            FormationShape {
                preset: 1,
                formation: 3,
                shape: [4, 5, 1]
            }
        );
    }

    #[test]
    fn from_team_toml_matches_new_on_every_version() {
        for version in PesVersion::ALL {
            let file = EditFile::from_bytes(&fixture_save(version)).expect("fixture save");
            let team = file.team(701).expect("team 701");
            let players: Vec<&PlayerEntry> = team
                .roster
                .iter()
                .filter(|slot| slot.player_id != 0)
                .map(|slot| file.player(slot.player_id).expect("a rostered player"))
                .collect();
            let document =
                TeamToml::from_team(version, team, &players).expect("from_team emits a document");
            let built = TeamView::from_team_toml(&document)
                .unwrap_or_else(|e| panic!("{version:?}: the document builds: {e}"));
            assert_eq!(
                built,
                TeamView::new(&file, 701).expect("the team views"),
                "{version:?}"
            );
        }
    }

    #[test]
    fn a_document_without_a_version_is_refused() {
        let document = TeamToml::default();
        assert_eq!(
            TeamView::from_team_toml(&document).expect_err("no version"),
            TeamViewError::NoVersion
        );
    }

    #[test]
    fn a_document_that_does_not_apply_is_team_toml() {
        // `stats.physical_contact` does not exist on PES 15; the note it
        // gives is a `TeamToml` error, not a skipped key.
        let file = EditFile::from_bytes(&fixture_save(PesVersion::Pes15)).expect("fixture save");
        let team = file.team(701).expect("team 701");
        let players: Vec<&PlayerEntry> = team
            .roster
            .iter()
            .filter(|slot| slot.player_id != 0)
            .map(|slot| file.player(slot.player_id).expect("a rostered player"))
            .collect();
        let mut document =
            TeamToml::from_team(PesVersion::Pes15, team, &players).expect("from_team emits");
        let section = document.players.get_mut(&1).expect("a first player");
        section.stats.physical_contact = Some(80);
        assert!(
            matches!(
                TeamView::from_team_toml(&document).expect_err("a version-lacking key"),
                TeamViewError::TeamToml { .. }
            ),
            "stats.physical_contact on PES 15 is refused"
        );
    }

    #[test]
    fn the_sample_team_is_team_714_of_the_cup_save() {
        let team = TeamView::sample_team();
        assert_eq!(team.players.len(), 23);
        assert_eq!(team.captain, Some(10));
    }

    #[test]
    fn pes15_fields_one_formation_per_preset() {
        let (file, team) = view(PesVersion::Pes15);
        // PES 15 has no fluid formation: `fluid` is `None` on all three
        // presets, so each fields its kick-off formation only.
        for preset in &file.team(701).expect("team 701").tactics.presets {
            assert_eq!(preset.style.fluid, None);
        }
        assert_eq!(team.formation_shapes().len(), 3);
    }
}
