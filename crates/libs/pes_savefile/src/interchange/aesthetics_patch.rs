//! `aesthetics_patch.toml`: the file the Team compiler writes to its output
//! folder with its CPKs, holding what the CPK's tables cannot — on Fox,
//! every compiled player's motions and an autopilot export's names — for the
//! savefile builder to apply (plan: `pes_savefile/operations.md` "Aesthetics
//! patch"). This module is the Fox half's type and its text form; applying
//! it is a later step, the pre-Fox keys arrive with that half.

use std::collections::BTreeMap;

use pes_version::{Engine, PesVersion};
use toml_edit::{DocumentMut, Item, TableLike, Value};

use crate::interchange::players_key;
use crate::settings_toml::document::{emit_motion, padded, parse_motion};
use crate::settings_toml::{MotionSettings, SettingsError};

/// The suite's boots and gloves ID allocation scheme (`team_compiler/pipeline.md`); a patch
/// from another scheme is refused.
pub const ALLOCATION_SCHEME_VERSION: u32 = 1;

/// The first line of a generated `aesthetics_patch.toml`.
const HEADER: &str = "# aesthetics_patch.toml, written by the Team compiler to its output folder\n";

/// `aesthetics_patch.toml`: the compiler's resolved savefile writes for its compiled players.
#[derive(Debug, Clone, PartialEq)]
pub struct AestheticsPatch {
    /// The PES version the compile targeted; a patch applies only to a save
    /// of the same version.
    pub pes_version: PesVersion,
    /// The 4cc Studio version that wrote the patch.
    pub studio_version: String,
    /// The `ALLOCATION_SCHEME_VERSION` the compile ran under; a mismatch
    /// refuses the patch.
    pub allocation_scheme_version: u32,
    /// The CPKs the run published; empty for a sideload tree.
    pub cpks: Vec<String>,
    /// When the compile ran.
    pub compiled: toml_edit::Datetime,
    /// The teams whose players the patch writes, in file order.
    pub teams: Vec<PatchTeam>,
}

/// One `[[teams]]` block: the players of one team the patch writes.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchTeam {
    /// The team id; `players.NN` is player id `id × 100 + NN`.
    pub id: u32,
    /// For the reader; the id is what is applied.
    pub name: String,
    /// By `NN` (`players.NN`, player id `id × 100 + NN`).
    pub players: BTreeMap<u8, PatchPlayer>,
}

/// One `[teams.players.NN]` table: what the patch writes of one player.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchPlayer {
    /// An autopilot export's only; `None` = not written.
    pub name: Option<String>,
    /// An autopilot export's only, like `name`; `None` = not written.
    pub shirt_name: Option<String>,
    /// `[appearance.motion]`, as in `settings.toml`; `None` keys are not written.
    pub motion: MotionSettings,
}

/// What parsing or writing an `aesthetics_patch.toml` fails with. The
/// key-level problems reuse `SettingsError`'s vocabulary (`UnknownKey`,
/// `WrongType`, `OutOfRange`, `UnknownLabel`) so both interchange formats
/// report a bad key the same way.
#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    /// The text is not valid TOML.
    #[error("aesthetics_patch.toml is not valid TOML: {0}")]
    Toml(String),
    /// A key a present table requires is absent.
    #[error("{key}: required in this table")]
    MissingKey {
        /// The dotted key path (`studio_version`, `teams[0].id`).
        key: String,
    },
    /// `pes_version` names a pre-Fox version; the pre-Fox patch is not
    /// built yet.
    #[error("{version}: the pre-Fox aesthetics patch is not supported yet")]
    PreFox {
        /// The version the file names.
        version: PesVersion,
    },
    /// A key-level problem, in `settings.toml`'s vocabulary.
    #[error(transparent)]
    Settings(#[from] SettingsError),
}

impl AestheticsPatch {
    /// Reads an `aesthetics_patch.toml` text. Every rule is an error, never
    /// a skip: the five header keys are `MissingKey`, any other top-level
    /// key `UnknownKey`, a pre-Fox `pes_version` `PreFox`, and each team's
    /// `[teams.players.NN]` takes the `name`, `shirt_name` and
    /// `[appearance.motion]` of the Fox patch alone.
    pub fn parse(text: &str) -> Result<Self, PatchError> {
        let document: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| PatchError::Toml(e.to_string()))?;
        let root = document.as_table();
        let pes_version = version(required(root, "", "pes_version")?)?;
        match pes_version.engine() {
            Engine::PreFox => {
                return Err(PatchError::PreFox {
                    version: pes_version,
                });
            }
            Engine::Fox => {}
        }
        let studio_version = string(required(root, "", "studio_version")?, "studio_version")?;
        let allocation_scheme_version = u32_value(
            required(root, "", "allocation_scheme_version")?,
            "allocation_scheme_version",
        )?;
        let cpks = string_array(required(root, "", "cpks")?, "cpks")?;
        let compiled = datetime(required(root, "", "compiled")?, "compiled")?;
        for (name, _) in root.iter() {
            const KNOWN: [&str; 6] = [
                "pes_version",
                "studio_version",
                "allocation_scheme_version",
                "cpks",
                "compiled",
                "teams",
            ];
            if !KNOWN.contains(&name) {
                return Err(SettingsError::UnknownKey {
                    key: name.to_string(),
                }
                .into());
            }
        }
        let mut teams = Vec::new();
        if let Some(item) = root.get("teams") {
            teams = parse_teams(item)?;
        }
        Ok(AestheticsPatch {
            pes_version,
            studio_version,
            allocation_scheme_version,
            cpks,
            compiled,
            teams,
        })
    }

    /// The patch's text form (the plan's block): the header comment and its
    /// five keys, then each team a `[[teams]]` block with a
    /// `[teams.players.NN]` per player — the header always written, `name`
    /// and `shirt_name` only when set — and its `[…appearance.motion]`
    /// table when a motion key is set. One blank line before each header.
    /// `Err` is a value `parse` would refuse back (a hand-built one).
    pub fn to_toml(&self) -> Result<String, PatchError> {
        let mut out = HEADER.to_string();
        out.push_str(&format!("pes_version = {}\n", self.pes_version.number()));
        out.push_str(&format!(
            "studio_version = {}\n",
            Value::from(self.studio_version.as_str())
        ));
        out.push_str(&format!(
            "allocation_scheme_version = {}\n",
            self.allocation_scheme_version
        ));
        let cpks = self
            .cpks
            .iter()
            .map(|cpk| Value::from(cpk.as_str()).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("cpks = [{cpks}]\n"));
        out.push_str(&format!("compiled = {}\n", self.compiled));
        for team in &self.teams {
            out.push_str(&format!("\n[[teams]]\nid = {}\n", team.id));
            out.push_str(&padded(
                &format!("name = {}", Value::from(team.name.as_str())),
                "for the reader; the id is what is applied",
            ));
            out.push('\n');
            for (slot, player) in &team.players {
                out.push_str(&format!("\n[teams.players.{slot:02}]\n"));
                if let Some(name) = &player.name {
                    out.push_str(&format!("name = {}\n", Value::from(name.as_str())));
                }
                if let Some(shirt_name) = &player.shirt_name {
                    out.push_str(&format!(
                        "shirt_name = {}\n",
                        Value::from(shirt_name.as_str())
                    ));
                }
                if player.motion != MotionSettings::default() {
                    out.push_str(&format!("\n[teams.players.{slot:02}.appearance.motion]\n"));
                    emit_motion(&mut out, &player.motion)?;
                }
            }
        }
        Ok(out)
    }
}

// ---------------------------------------------------------------------------
// Parse

/// `path.name`, or `name` at the root (a top-level key's path is its name).
fn leaf(path: &str, name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{path}.{name}")
    }
}

/// `table.name`, `MissingKey` at `path.name` when absent.
fn required<'a>(table: &'a dyn TableLike, path: &str, name: &str) -> Result<&'a Item, PatchError> {
    table.get(name).ok_or_else(|| PatchError::MissingKey {
        key: leaf(path, name),
    })
}

/// `item` as a table-like (`teams`, `players.NN`, `appearance`); `key` is
/// its dotted path.
fn table_like<'a>(item: &'a Item, key: &str) -> Result<&'a dyn TableLike, PatchError> {
    item.as_table_like().ok_or_else(|| {
        SettingsError::WrongType {
            key: key.to_string(),
            expected: "a table",
        }
        .into()
    })
}

/// `item` as a string (`studio_version`, team and player names).
fn string(item: &Item, key: &str) -> Result<String, PatchError> {
    item.as_value()
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            SettingsError::WrongType {
                key: key.to_string(),
                expected: "a string",
            }
            .into()
        })
}

/// `item` as a `u32` (team `id`, `allocation_scheme_version`).
fn u32_value(item: &Item, key: &str) -> Result<u32, PatchError> {
    let value = item
        .as_value()
        .and_then(|v| v.as_integer())
        .ok_or_else(|| SettingsError::WrongType {
            key: key.to_string(),
            expected: "an integer",
        })?;
    u32::try_from(value).map_err(|_| {
        SettingsError::OutOfRange {
            key: key.to_string(),
            value,
            range: "0 to 4294967295".to_string(),
        }
        .into()
    })
}

/// `item` as an array of strings (the `cpks` list).
fn string_array(item: &Item, key: &str) -> Result<Vec<String>, PatchError> {
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| SettingsError::WrongType {
                key: key.to_string(),
                expected: "an array of strings",
            })?;
    array
        .iter()
        .map(|value| {
            value.as_str().map(str::to_string).ok_or_else(|| {
                SettingsError::WrongType {
                    key: key.to_string(),
                    expected: "an array of strings",
                }
                .into()
            })
        })
        .collect()
}

/// `item` as a TOML datetime (the `compiled` stamp).
fn datetime(item: &Item, key: &str) -> Result<toml_edit::Datetime, PatchError> {
    item.as_value()
        .and_then(|v| v.as_datetime())
        .copied()
        .ok_or_else(|| {
            SettingsError::WrongType {
                key: key.to_string(),
                expected: "a datetime",
            }
            .into()
        })
}

/// `item` as a `PesVersion` (the `pes_version` key): an integer naming a
/// version, anything else `WrongType`.
fn version(item: &Item) -> Result<PesVersion, PatchError> {
    item.as_value()
        .and_then(|v| v.as_integer())
        .and_then(|v| u16::try_from(v).ok())
        .and_then(PesVersion::from_number)
        .ok_or_else(|| {
            SettingsError::WrongType {
                key: "pes_version".to_string(),
                expected: "a PES version, 15 to 21",
            }
            .into()
        })
}

/// The `teams` array of tables; each element's path is `teams[i]`,
/// zero-based.
fn parse_teams(item: &Item) -> Result<Vec<PatchTeam>, PatchError> {
    let teams = item
        .as_array_of_tables()
        .ok_or_else(|| SettingsError::WrongType {
            key: "teams".to_string(),
            expected: "an array of tables",
        })?;
    teams
        .iter()
        .enumerate()
        .map(|(i, table)| parse_team(table, &format!("teams[{i}]")))
        .collect()
}

/// One `[[teams]]`: `id` and `name` required, `players` absent = empty,
/// anything else `UnknownKey`.
fn parse_team(table: &dyn TableLike, path: &str) -> Result<PatchTeam, PatchError> {
    let id = u32_value(required(table, path, "id")?, &leaf(path, "id"))?;
    let name = string(required(table, path, "name")?, &leaf(path, "name"))?;
    for (name, _) in table.iter() {
        if !["id", "name", "players"].contains(&name) {
            return Err(SettingsError::UnknownKey {
                key: leaf(path, name),
            }
            .into());
        }
    }
    let mut players = BTreeMap::new();
    if let Some(item) = table.get("players") {
        players = parse_players(item, &leaf(path, "players"))?;
    }
    Ok(PatchTeam { id, name, players })
}

/// A team's `players` table: keys are two digits `01` to `99` (`NN` of
/// `team id × 100 + NN`), anything else `UnknownKey`.
fn parse_players(item: &Item, path: &str) -> Result<BTreeMap<u8, PatchPlayer>, PatchError> {
    let table = table_like(item, path)?;
    let mut out = BTreeMap::new();
    for (name, item) in table.iter() {
        if !players_key(name, 1..=99) {
            return Err(SettingsError::UnknownKey {
                key: leaf(path, name),
            }
            .into());
        }
        let slot = name.parse::<u8>().expect("checked above");
        out.insert(slot, parse_player(item, &leaf(path, name))?);
    }
    Ok(out)
}

/// One `[teams.players.NN]`: `name` and `shirt_name` optional strings,
/// `appearance` a table holding `motion` alone; every other key or
/// subtable (the pre-Fox patch's `boots_id`, `appearance.face`, …) is
/// `UnknownKey`. No length checks: those are `apply`'s, against the
/// version's fields.
fn parse_player(item: &Item, path: &str) -> Result<PatchPlayer, PatchError> {
    let table = table_like(item, path)?;
    let mut player = PatchPlayer {
        name: None,
        shirt_name: None,
        motion: MotionSettings::default(),
    };
    if let Some(item) = table.get("name") {
        player.name = Some(string(item, &leaf(path, "name"))?);
    }
    if let Some(item) = table.get("shirt_name") {
        player.shirt_name = Some(string(item, &leaf(path, "shirt_name"))?);
    }
    if let Some(item) = table.get("appearance") {
        player.motion = parse_appearance(item, &leaf(path, "appearance"))?;
    }
    for (name, _) in table.iter() {
        if !["name", "shirt_name", "appearance"].contains(&name) {
            return Err(SettingsError::UnknownKey {
                key: leaf(path, name),
            }
            .into());
        }
    }
    Ok(player)
}

/// A Fox patch's `appearance` table holds `motion` alone; any other key or
/// subtable (a pre-Fox patch's `[appearance.face]`, …) is `UnknownKey`.
fn parse_appearance(item: &Item, path: &str) -> Result<MotionSettings, PatchError> {
    let table = table_like(item, path)?;
    let mut motion = MotionSettings::default();
    if let Some(item) = table.get("motion") {
        motion = parse_motion(item, &leaf(path, "motion"))?;
    }
    for (name, _) in table.iter() {
        if name != "motion" {
            return Err(SettingsError::UnknownKey {
                key: leaf(path, name),
            }
            .into());
        }
    }
    Ok(motion)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The five required header keys at valid values; a refusal case that
    /// does not mutate the header starts from them.
    const KEYS: &str = "\
pes_version = 21
studio_version = \"0.1.0\"
allocation_scheme_version = 1
cpks = [\"4cc_69.cpk\"]
compiled = 2026-09-12T14:03:00Z
";

    /// The header keys plus `tail`.
    fn doc(tail: &str) -> String {
        format!("{KEYS}{tail}")
    }

    /// The plan's sample, the motion lines as `settings.toml`'s emitter
    /// writes them (`padded` puts `#` at column 33).
    const GOLDEN: &str = "\
# aesthetics_patch.toml, written by the Team compiler to its output folder
pes_version = 21
studio_version = \"0.1.0\"
allocation_scheme_version = 1
cpks = [\"4cc_69.cpk\"]
compiled = 2026-09-12T14:03:00Z

[[teams]]
id = 714
name = \"/co/\"                   # for the reader; the id is what is applied

[teams.players.05]
name = \"Snuffy\"
shirt_name = \"SNUFFY\"

[teams.players.05.appearance.motion]
hunching_dribbling = 2          # 1 to 3 (PES 20 and 21: 1 to 5)
free_kick = 4                   # 1 to 16 (PES 20 and 21: 1 to 20)

[teams.players.06]

[teams.players.06.appearance.motion]
dribbling = 2                   # 0 to 3, PES 20 and 21 only
";

    /// The patch `GOLDEN` reads and writes.
    fn golden() -> AestheticsPatch {
        AestheticsPatch {
            pes_version: PesVersion::Pes21,
            studio_version: "0.1.0".to_string(),
            allocation_scheme_version: 1,
            cpks: vec!["4cc_69.cpk".to_string()],
            compiled: "2026-09-12T14:03:00Z".parse().expect("a datetime"),
            teams: vec![PatchTeam {
                id: 714,
                name: "/co/".to_string(),
                players: BTreeMap::from([
                    (
                        5,
                        PatchPlayer {
                            name: Some("Snuffy".to_string()),
                            shirt_name: Some("SNUFFY".to_string()),
                            motion: MotionSettings {
                                hunching_dribbling: Some(1),
                                free_kick: Some(3),
                                ..MotionSettings::default()
                            },
                        },
                    ),
                    (
                        6,
                        PatchPlayer {
                            name: None,
                            shirt_name: None,
                            motion: MotionSettings {
                                dribbling: Some(2),
                                ..MotionSettings::default()
                            },
                        },
                    ),
                ]),
            }],
        }
    }

    #[test]
    fn golden_text() {
        let patch = golden();
        assert_eq!(patch.to_toml().expect("emit"), GOLDEN);
        assert_eq!(
            AestheticsPatch::parse(GOLDEN).expect("the golden parses"),
            patch
        );
    }

    /// A hand-built patch can hold a stored value no key's range expresses;
    /// `to_toml` refuses it rather than writing a file `parse` would not
    /// read back. `free_kick` stored 20 would emit `free_kick = 21`, one
    /// past its 1 to 20.
    #[test]
    fn to_toml_refuses_a_value_parse_would_refuse_back() {
        let mut patch = golden();
        patch.teams[0]
            .players
            .get_mut(&5)
            .expect("player 05")
            .motion
            .free_kick = Some(20);
        let err = patch.to_toml().expect_err("free_kick = 21 is out of range");
        assert!(
            matches!(err, PatchError::Settings(SettingsError::OutOfRange { .. })),
            "{err:?}"
        );
    }

    /// Every motion key `Some` at a non-default value, on PES 21 (with
    /// `dribbling`) and PES 18 (without), two teams.
    #[test]
    fn every_motion_key_round_trips() {
        for (pes_version, dribbling) in [(PesVersion::Pes21, Some(2)), (PesVersion::Pes18, None)] {
            let motion = |goal_celebration_2| MotionSettings {
                hunching_dribbling: Some(2),
                hunching_running: Some(3),
                arm_movement_dribbling: Some(5),
                arm_movement_running: Some(6),
                corner_kick: Some(4),
                free_kick: Some(15),
                penalty_kick: Some(3),
                dribbling,
                goal_celebration_1: Some(40),
                goal_celebration_2: Some(goal_celebration_2),
            };
            let patch = AestheticsPatch {
                pes_version,
                studio_version: "0.1.0".to_string(),
                allocation_scheme_version: ALLOCATION_SCHEME_VERSION,
                cpks: vec!["4cc_69.cpk".to_string()],
                compiled: "2026-09-12T14:03:00Z".parse().expect("a datetime"),
                teams: vec![
                    PatchTeam {
                        id: 714,
                        name: "/co/".to_string(),
                        players: BTreeMap::from([(
                            5,
                            PatchPlayer {
                                name: Some("Snuffy".to_string()),
                                shirt_name: Some("SNUFFY".to_string()),
                                motion: motion(120),
                            },
                        )]),
                    },
                    PatchTeam {
                        id: 702,
                        name: "/a/".to_string(),
                        players: BTreeMap::from([(
                            3,
                            PatchPlayer {
                                name: None,
                                shirt_name: None,
                                motion: motion(150),
                            },
                        )]),
                    },
                ],
            };
            assert_eq!(
                AestheticsPatch::parse(&patch.to_toml().expect("emit")).expect("round trip"),
                patch
            );
        }
    }

    #[test]
    fn refusals() {
        // (patch text, the error it must produce); `doc` supplies the valid
        // header keys where a case does not mutate them.
        let cases: Vec<(String, PatchError)> = vec![
            (
                "pes_version = 17\nstudio_version = \"0.1.0\"\nallocation_scheme_version = 1\ncpks = []\ncompiled = 2026-09-12T14:03:00Z\n".to_string(),
                PatchError::PreFox {
                    version: PesVersion::Pes17,
                },
            ),
            (
                doc("").replace("pes_version = 21", "pes_version = 22"),
                SettingsError::WrongType {
                    key: "pes_version".to_string(),
                    expected: "a PES version, 15 to 21",
                }
                .into(),
            ),
            (
                doc("").replace("pes_version = 21", "pes_version = \"21\""),
                SettingsError::WrongType {
                    key: "pes_version".to_string(),
                    expected: "a PES version, 15 to 21",
                }
                .into(),
            ),
            (
                doc("").replace("studio_version = \"0.1.0\"\n", ""),
                PatchError::MissingKey {
                    key: "studio_version".to_string(),
                },
            ),
            (
                doc("foo = 1\n"),
                SettingsError::UnknownKey {
                    key: "foo".to_string(),
                }
                .into(),
            ),
            (
                doc("[[teams]]\nname = \"/a/\"\n"),
                PatchError::MissingKey {
                    key: "teams[0].id".to_string(),
                },
            ),
            (
                doc("[[teams]]\nid = 702\nname = \"/a/\"\n\n[teams.players.5]\n"),
                SettingsError::UnknownKey {
                    key: "teams[0].players.5".to_string(),
                }
                .into(),
            ),
            (
                doc("[[teams]]\nid = 702\nname = \"/a/\"\n\n[teams.players.00]\n"),
                SettingsError::UnknownKey {
                    key: "teams[0].players.00".to_string(),
                }
                .into(),
            ),
            (
                doc("[[teams]]\nid = 714\nname = \"/co/\"\n\n[teams.players.05]\nboots_id = 143\n"),
                SettingsError::UnknownKey {
                    key: "teams[0].players.05.boots_id".to_string(),
                }
                .into(),
            ),
            (
                doc("[[teams]]\nid = 714\nname = \"/co/\"\n\n[teams.players.05.appearance.face]\ncheek_type = 1\n"),
                SettingsError::UnknownKey {
                    key: "teams[0].players.05.appearance.face".to_string(),
                }
                .into(),
            ),
            (
                doc("[[teams]]\nid = 714\nname = \"/co/\"\n\n[teams.players.05.appearance.motion]\nfree_kick = 21\n"),
                SettingsError::OutOfRange {
                    key: "teams[0].players.05.appearance.motion.free_kick".to_string(),
                    value: 21,
                    range: "1 to 20".to_string(),
                }
                .into(),
            ),
            (
                doc("").replace("compiled = 2026-09-12T14:03:00Z", "compiled = \"x\""),
                SettingsError::WrongType {
                    key: "compiled".to_string(),
                    expected: "a datetime",
                }
                .into(),
            ),
            // A motion table takes the motion keys alone: a stranger, and a
            // key of another table, are both `UnknownKey`.
            (
                doc("[[teams]]\nid = 714\nname = \"/co/\"\n\n[teams.players.05.appearance.motion]\nfoo = 1\n"),
                SettingsError::UnknownKey {
                    key: "teams[0].players.05.appearance.motion.foo".to_string(),
                }
                .into(),
            ),
            (
                doc("[[teams]]\nid = 714\nname = \"/co/\"\n\n[teams.players.05.appearance.motion]\nskin_color = 1\n"),
                SettingsError::UnknownKey {
                    key: "teams[0].players.05.appearance.motion.skin_color".to_string(),
                }
                .into(),
            ),
        ];
        for (text, expected) in cases {
            let error = AestheticsPatch::parse(&text).expect_err(&text);
            assert_eq!(error.to_string(), expected.to_string(), "{text}");
        }
    }

    #[test]
    fn text_that_is_not_toml_is_toml_error() {
        let error = AestheticsPatch::parse("this is not toml").expect_err("not TOML");
        assert!(matches!(error, PatchError::Toml(_)), "{error:?}");
    }
}
