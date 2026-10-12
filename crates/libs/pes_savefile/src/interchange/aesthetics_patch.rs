//! `aesthetics_patch.toml`: the file the Team compiler writes to its output
//! folder with its CPKs, holding what the CPK's tables cannot — on Fox,
//! every compiled player's motions and an autopilot export's names — for the
//! savefile builder to apply (plan: `pes_savefile/operations.md` "Aesthetics
//! patch"). This module is the Fox half's type and its text form; applying
//! it is a later step, the pre-Fox keys arrive with that half.

use std::collections::BTreeMap;

use pes_version::{Engine, PesVersion};
use toml_edit::{DocumentMut, Item, TableLike, Value};

use crate::codec::{text_max, write_player};
use crate::file::EditFile;
use crate::interchange::players_key;
use crate::model::player::PlayerEntry;
use crate::schema::fields::PlayerText;
use crate::schema::{VersionSchema, schema_for};
use crate::settings_toml::document::{emit_motion, padded, parse_motion, range_text};
use crate::settings_toml::{
    AppearanceSettings, MotionSettings, SettingKey, SettingsError, apply_appearance, check_text,
    get_appearance,
};

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

/// What parsing, writing or applying an `aesthetics_patch.toml` fails with.
/// The key-level problems reuse `SettingsError`'s vocabulary (`UnknownKey`,
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
    /// `pes_version` is not the save's: a patch applies to the version it
    /// was compiled for, there is no cross-version application.
    #[error("the patch is for {patch}, the save is {save}")]
    VersionMismatch {
        /// The version the patch was compiled for.
        patch: PesVersion,
        /// The save's version.
        save: PesVersion,
    },
    /// `allocation_scheme_version` is not the suite's.
    #[error("the patch was compiled under allocation scheme {patch}, this suite uses {suite}")]
    SchemeMismatch {
        /// The scheme the patch was compiled under.
        patch: u32,
        /// The suite's `ALLOCATION_SCHEME_VERSION`.
        suite: u32,
    },
    /// A key-level problem, in `settings.toml`'s vocabulary.
    #[error(transparent)]
    Settings(#[from] SettingsError),
}

/// What `apply` did: the teams written and the items the save lacked.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchSummary {
    /// The ids of the patch's teams the save holds, in file order.
    pub applied_teams: Vec<u32>,
    /// The patch's teams and players the save lacks, in file order.
    pub skipped: Vec<PatchSkip>,
}

/// A patch item the save has no target for; the rest of the patch applied.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatchSkip {
    /// The save has no team with this id; its players were not looked at.
    TeamNotInSave {
        /// The team id.
        team: u32,
    },
    /// The save has no player with this id (`team id × 100 + NN`).
    PlayerNotInSave {
        /// The player id.
        player: u32,
    },
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
    /// `Err` is a value `parse` would refuse back: a pre-Fox `pes_version`
    /// (`PreFox`), a `players.NN` key outside `01` to `99` (`UnknownKey` at
    /// `teams[i].players.NN`), or a stored value a key's range cannot hold.
    pub fn to_toml(&self) -> Result<String, PatchError> {
        match self.pes_version.engine() {
            Engine::PreFox => {
                return Err(PatchError::PreFox {
                    version: self.pes_version,
                });
            }
            Engine::Fox => {}
        }
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
        for (i, team) in self.teams.iter().enumerate() {
            out.push_str(&format!("\n[[teams]]\nid = {}\n", team.id));
            out.push_str(&padded(
                &format!("name = {}", Value::from(team.name.as_str())),
                "for the reader; the id is what is applied",
            ));
            out.push('\n');
            for (slot, player) in &team.players {
                let key = format!("{slot:02}");
                if !players_key(&key, 1..=99) {
                    return Err(SettingsError::UnknownKey {
                        key: format!("teams[{i}].players.{key}"),
                    }
                    .into());
                }
                out.push_str(&format!("\n[teams.players.{key}]\n"));
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

    /// Applies the patch to `save`: each listed player's motions and, for
    /// an autopilot export, his names — nothing of his appearance block, so
    /// a stripped player (appearance id -1) stays stripped. Every refusal
    /// runs before any write, as `transplant` checks before it writes: a
    /// pre-Fox `pes_version` (a hand-built patch can hold one), a version
    /// or allocation-scheme mismatch, and in pass 1 each player's names
    /// against the version's text fields, each `Some` motion key against
    /// its range and the field's width; an `Err` leaves the save untouched.
    /// Only a team or player the save lacks is skipped, per item, and
    /// reported in the summary.
    pub fn apply(&self, save: &mut EditFile) -> Result<PatchSummary, PatchError> {
        match self.pes_version.engine() {
            Engine::PreFox => {
                return Err(PatchError::PreFox {
                    version: self.pes_version,
                });
            }
            Engine::Fox => {}
        }
        if self.pes_version != save.version() {
            return Err(PatchError::VersionMismatch {
                patch: self.pes_version,
                save: save.version(),
            });
        }
        if self.allocation_scheme_version != ALLOCATION_SCHEME_VERSION {
            return Err(PatchError::SchemeMismatch {
                patch: self.allocation_scheme_version,
                suite: ALLOCATION_SCHEME_VERSION,
            });
        }
        let schema = schema_for(save.version());
        let mut applied_teams = Vec::new();
        let mut skipped = Vec::new();
        // Pass 1: resolve each entry against the save, building the writes
        // on copies; an error returns with the save untouched.
        let mut built: Vec<(u32, PlayerEntry)> = Vec::new();
        for (i, team) in self.teams.iter().enumerate() {
            let team_path = format!("teams[{i}]");
            if save.team(team.id).is_none() {
                skipped.push(PatchSkip::TeamNotInSave { team: team.id });
                continue;
            }
            applied_teams.push(team.id);
            for (slot, patch_player) in &team.players {
                let player_path = format!("{team_path}.players.{slot:02}");
                let player_id = team.id * 100 + u32::from(*slot);
                let Some(player) = save.player(player_id) else {
                    skipped.push(PatchSkip::PlayerNotInSave { player: player_id });
                    continue;
                };
                built.push((
                    player_id,
                    build(player, patch_player, &player_path, schema)?,
                ));
            }
        }
        // Pass 2: every built entry replaces its player.
        for (player_id, entry) in built {
            *save
                .player_mut(player_id)
                .expect("present: pass 1 found it") = entry;
        }
        Ok(PatchSummary {
            applied_teams,
            skipped,
        })
    }
}

// ---------------------------------------------------------------------------
// Apply

/// The patch's writes onto a copy of `player` — the `Some` names checked
/// against the version's text fields as `PlayerSettings::apply` checks them,
/// each `Some` motion key against its kind's range — then written through
/// `apply_appearance` so a `Some` the version has no field for is
/// `NotInThisVersion`. The copy is encoded into a scratch record of the
/// version's player size so a stored value past the field's width is
/// refused here (`Codec`), not when the save is written. `path` is the
/// player's patch path (`teams[0].players.05`).
fn build(
    player: &PlayerEntry,
    patch: &PatchPlayer,
    path: &str,
    schema: &VersionSchema,
) -> Result<PlayerEntry, PatchError> {
    let mut next = player.clone();
    if let Some(name) = &patch.name {
        check_text(
            &leaf(path, "name"),
            name,
            text_max(schema.player.texts, PlayerText::Name),
            false,
        )?;
        next.name = name.clone();
    }
    if let Some(shirt_name) = &patch.shirt_name {
        check_text(
            &leaf(path, "shirt_name"),
            shirt_name,
            text_max(schema.player.texts, PlayerText::ShirtName),
            true,
        )?;
        next.shirt_name = shirt_name.clone();
    }
    let motion_path = format!("{path}.appearance.motion");
    let appearance = AppearanceSettings {
        motion: patch.motion.clone(),
        ..AppearanceSettings::default()
    };
    for key in SettingKey::ALL {
        let spec = key.spec();
        let Some(stored) = get_appearance(&appearance, key) else {
            continue;
        };
        if !spec.kind.accepts_stored(stored) {
            return Err(SettingsError::OutOfRange {
                key: format!("{motion_path}.{}", spec.name),
                value: i64::from(stored),
                range: range_text(spec.kind),
            }
            .into());
        }
    }
    // The only refusal the motion-only write can hit (`dribbling` on a
    // version without the field); its key names the patch path.
    apply_appearance(&appearance, &mut next).map_err(|err| match err {
        SettingsError::NotInThisVersion { key } => SettingsError::NotInThisVersion {
            key: format!("{motion_path}.{key}"),
        },
        other => other,
    })?;
    let mut record = vec![0u8; schema.player.size];
    write_player(&next, &mut record, schema.player).map_err(SettingsError::Codec)?;
    Ok(next)
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

    /// A hand-built patch can hold a pre-Fox `pes_version`; `to_toml`
    /// refuses it rather than writing a file `parse` would reject.
    #[test]
    fn to_toml_refuses_a_pre_fox_version() {
        let patch = patch_at(PesVersion::Pes17, vec![team(701, BTreeMap::new())]);
        let err = patch.to_toml().expect_err("a pre-Fox patch is not written");
        assert!(
            matches!(err, PatchError::PreFox { version } if version == PesVersion::Pes17),
            "{err:?}"
        );
    }

    /// A hand-built `players` map can hold a slot outside `01` to `99`;
    /// `to_toml` refuses it at the key `parse` would name.
    #[test]
    fn to_toml_refuses_a_player_key_outside_01_to_99() {
        for (slot, key) in [
            (0u8, "teams[0].players.00"),
            (100u8, "teams[0].players.100"),
        ] {
            let patch = patch_at(
                PesVersion::Pes21,
                vec![team(701, BTreeMap::from([(slot, named("X"))]))],
            );
            let err = patch
                .to_toml()
                .expect_err("a bad player key is not written");
            match &err {
                PatchError::Settings(SettingsError::UnknownKey { key: got }) => {
                    assert_eq!(got, key, "slot {slot}")
                }
                other => panic!("slot {slot}: {other:?}"),
            }
        }
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

    // -------------------------------------------------------------------
    // Apply

    use crate::model::player::PlayerMotion;
    use crate::schema::fields::PlayerField;
    use crate::test_support::{open, test_salt};

    /// A patch at `version` over `teams`.
    fn patch_at(version: PesVersion, teams: Vec<PatchTeam>) -> AestheticsPatch {
        AestheticsPatch {
            pes_version: version,
            studio_version: "0.1.0".to_string(),
            allocation_scheme_version: ALLOCATION_SCHEME_VERSION,
            cpks: vec!["4cc_69.cpk".to_string()],
            compiled: "2026-09-12T14:03:00Z".parse().expect("a datetime"),
            teams,
        }
    }

    /// A player with `name` alone.
    fn named(name: &str) -> PatchPlayer {
        PatchPlayer {
            name: Some(name.to_string()),
            shirt_name: None,
            motion: MotionSettings::default(),
        }
    }

    /// A player with `motion` alone.
    fn motion_only(motion: MotionSettings) -> PatchPlayer {
        PatchPlayer {
            name: None,
            shirt_name: None,
            motion,
        }
    }

    /// A `[[teams]]` block; `name` is a reader aid `apply` never uses.
    fn team(id: u32, players: BTreeMap<u8, PatchPlayer>) -> PatchTeam {
        PatchTeam {
            id,
            name: String::new(),
            players,
        }
    }

    /// Every motion key `Some` at a value differing from `motion`'s —
    /// `value + 1` inside the key's stored range, wrapping.
    fn shifted(motion: &PlayerMotion) -> MotionSettings {
        let next = |value: u8, range: u8| (value + 1) % range;
        MotionSettings {
            hunching_dribbling: Some(next(motion.hunching_dribbling, 5)),
            hunching_running: Some(next(motion.hunching_running, 5)),
            arm_movement_dribbling: Some(next(motion.arm_movement_dribbling, 10)),
            arm_movement_running: Some(next(motion.arm_movement_running, 10)),
            corner_kick: Some(next(motion.corner_kick, 10)),
            free_kick: Some(next(motion.free_kick, 20)),
            penalty_kick: Some(next(motion.penalty_kick, 7)),
            dribbling: Some(next(motion.dribbling.expect("PES 21 has the field"), 4)),
            goal_celebration_1: Some(next(motion.goal_celebration_1, 163)),
            goal_celebration_2: Some(next(motion.goal_celebration_2, 163)),
        }
    }

    /// `before` with the patch's names and `Some` motion keys written —
    /// what `apply` must produce of `before`.
    fn patched(before: &PlayerEntry, patch: &PatchPlayer) -> PlayerEntry {
        let mut out = before.clone();
        if let Some(name) = &patch.name {
            out.name = name.clone();
        }
        if let Some(shirt_name) = &patch.shirt_name {
            out.shirt_name = shirt_name.clone();
        }
        let m = &patch.motion;
        if let Some(v) = m.hunching_dribbling {
            out.motion.hunching_dribbling = v;
        }
        if let Some(v) = m.hunching_running {
            out.motion.hunching_running = v;
        }
        if let Some(v) = m.arm_movement_dribbling {
            out.motion.arm_movement_dribbling = v;
        }
        if let Some(v) = m.arm_movement_running {
            out.motion.arm_movement_running = v;
        }
        if let Some(v) = m.corner_kick {
            out.motion.corner_kick = v;
        }
        if let Some(v) = m.free_kick {
            out.motion.free_kick = v;
        }
        if let Some(v) = m.penalty_kick {
            out.motion.penalty_kick = v;
        }
        if let Some(v) = m.dribbling {
            out.motion.dribbling = Some(v);
        }
        if let Some(v) = m.goal_celebration_1 {
            out.motion.goal_celebration_1 = v;
        }
        if let Some(v) = m.goal_celebration_2 {
            out.motion.goal_celebration_2 = v;
        }
        out
    }

    // SE-PAT-01
    #[test]
    fn apply_writes_the_motions_and_names_and_nothing_else() {
        let (mut save, _) = open(PesVersion::Pes21);
        let slots = [5u8, 6, 7];
        let patched_ids: Vec<u32> = slots.iter().map(|s| 70100 + u32::from(*s)).collect();
        let before: Vec<PlayerEntry> = save.players().to_vec();
        let teams = save.teams().to_vec();
        let mut players = BTreeMap::new();
        for &id in &patched_ids {
            let player = save.player(id).expect("the fixture has the player");
            let slot = u8::try_from(id % 100).expect("an NN");
            let patch_player = PatchPlayer {
                name: Some(format!("PATCH{slot:02}")),
                shirt_name: Some(format!("SHIRT{slot:02}")),
                motion: shifted(&player.motion),
            };
            // The patch's values differ from the save's, or the write
            // proves nothing.
            assert_ne!(
                patched(player, &patch_player),
                *player,
                "player {id}: patch values must differ"
            );
            players.insert(slot, patch_player);
        }
        let patch = patch_at(PesVersion::Pes21, vec![team(701, players)]);
        let summary = patch.apply(&mut save).expect("applies");
        assert_eq!(summary.applied_teams, [701]);
        assert!(summary.skipped.is_empty());
        for (slot, patch_player) in &patch.teams[0].players {
            let id = 70100 + u32::from(*slot);
            let before = before.iter().find(|p| p.id == id).expect("in before");
            assert_eq!(
                save.player(id).expect("the player"),
                &patched(before, patch_player),
                "player {id}"
            );
        }
        for (after, before) in save.players().iter().zip(&before) {
            if !patched_ids.contains(&after.id) {
                assert_eq!(after, before, "player {}", after.id);
            }
        }
        assert_eq!(save.teams(), &teams[..], "no team was written");
        save.to_bytes(&test_salt()).expect("the save writes");
    }

    // SE-PAT-02
    #[test]
    fn an_absent_team_or_player_is_skipped_and_the_rest_applied() {
        let (mut save, _) = open(PesVersion::Pes21);
        let before: Vec<PlayerEntry> = save.players().to_vec();
        // Player 70124: team 701's first slot past its 23-man roster; the
        // save has no such player. Team 999 (the referee id) is a real cup
        // team id the fixture's all-701..=920 list does not hold.
        assert!(save.player(70124).is_none());
        assert!(save.team(999).is_none());
        let players = BTreeMap::from([
            (
                5,
                PatchPlayer {
                    name: Some("PATCH05".to_string()),
                    shirt_name: None,
                    motion: shifted(&save.player(70105).expect("present").motion),
                },
            ),
            (
                24,
                PatchPlayer {
                    name: Some("GHOST".to_string()),
                    shirt_name: Some("GHOST".to_string()),
                    motion: MotionSettings {
                        free_kick: Some(3),
                        ..MotionSettings::default()
                    },
                },
            ),
        ]);
        let absent = PatchTeam {
            id: 999,
            name: "refs".to_string(),
            // An absent team's players are not looked at: this one produces
            // no `PlayerNotInSave`.
            players: BTreeMap::from([(5, named("REF"))]),
        };
        let patch = patch_at(PesVersion::Pes21, vec![team(701, players), absent]);
        let summary = patch.apply(&mut save).expect("applies");
        assert_eq!(summary.applied_teams, [701]);
        assert_eq!(
            summary.skipped,
            [
                PatchSkip::PlayerNotInSave { player: 70124 },
                PatchSkip::TeamNotInSave { team: 999 },
            ]
        );
        let before = before.iter().find(|p| p.id == 70105).expect("in before");
        assert_eq!(
            save.player(70105).expect("the player"),
            &patched(before, patch.teams[0].players.get(&5).expect("player 05"))
        );
    }

    // SE-PAT-03
    #[test]
    fn a_version_or_scheme_mismatch_refuses_before_any_write() {
        for (pes_version, scheme, expected) in [
            (
                PesVersion::Pes19,
                ALLOCATION_SCHEME_VERSION,
                PatchError::VersionMismatch {
                    patch: PesVersion::Pes19,
                    save: PesVersion::Pes21,
                },
            ),
            (
                PesVersion::Pes21,
                2,
                PatchError::SchemeMismatch {
                    patch: 2,
                    suite: ALLOCATION_SCHEME_VERSION,
                },
            ),
            (
                PesVersion::Pes17,
                ALLOCATION_SCHEME_VERSION,
                PatchError::PreFox {
                    version: PesVersion::Pes17,
                },
            ),
        ] {
            let (mut save, _) = open(PesVersion::Pes21);
            let before = save.players().to_vec();
            let teams = save.teams().to_vec();
            let mut patch = patch_at(pes_version, vec![team(701, BTreeMap::new())]);
            patch.allocation_scheme_version = scheme;
            let err = patch.apply(&mut save).expect_err("refused");
            assert_eq!(
                err.to_string(),
                expected.to_string(),
                "{pes_version:?} {scheme}"
            );
            assert_eq!(save.players(), &before[..], "{pes_version:?}");
            assert_eq!(save.teams(), &teams[..], "{pes_version:?}");
        }
    }

    // SE-PAT-04
    #[test]
    fn a_stripped_player_stays_stripped() {
        let (mut save, _) = open(PesVersion::Pes21);
        let slots = [5u8, 6];
        for &slot in &slots {
            save.player_mut(70100 + u32::from(slot))
                .expect("present")
                .appearance_id = Some(u32::MAX);
        }
        let before: Vec<PlayerEntry> = slots
            .iter()
            .map(|slot| {
                save.player(70100 + u32::from(*slot))
                    .expect("present")
                    .clone()
            })
            .collect();
        let players: BTreeMap<u8, PatchPlayer> = slots
            .iter()
            .enumerate()
            .map(|(i, slot)| {
                (
                    *slot,
                    PatchPlayer {
                        name: Some(format!("PATCH{slot:02}")),
                        shirt_name: Some(format!("SHIRT{slot:02}")),
                        motion: shifted(&before[i].motion),
                    },
                )
            })
            .collect();
        let patch = patch_at(PesVersion::Pes21, vec![team(701, players)]);
        patch.apply(&mut save).expect("applies");
        for (before, (slot, patch_player)) in before.iter().zip(&patch.teams[0].players) {
            let id = 70100 + u32::from(*slot);
            let after = save.player(id).expect("the player");
            assert_eq!(after.appearance_id, Some(u32::MAX), "{id} stays stripped");
            assert_eq!(
                after.appearance, before.appearance,
                "{id}: the appearance block is untouched"
            );
            assert_eq!(after, &patched(before, patch_player), "{id}");
        }
    }

    /// A failure anywhere in pass 1 leaves the whole save untouched: a
    /// name past the version's field, a motion the version lacks, a stored
    /// value past the field's width.
    #[test]
    fn a_broken_patch_is_refused_whole() {
        // A name longer than PES 21's 61-byte field on the second team's
        // player: the first team's player is not written either.
        let (mut save, _) = open(PesVersion::Pes21);
        let before = save.players().to_vec();
        let teams = save.teams().to_vec();
        let patch = patch_at(
            PesVersion::Pes21,
            vec![
                team(701, BTreeMap::from([(5, named("FINE"))])),
                team(702, BTreeMap::from([(5, named(&"N".repeat(100)))])),
            ],
        );
        let err = patch.apply(&mut save).expect_err("refused");
        match &err {
            PatchError::Settings(SettingsError::TextTooLong { key, .. }) => {
                assert_eq!(key, "teams[1].players.05.name")
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(save.players(), &before[..]);
        assert_eq!(save.teams(), &teams[..]);

        // `dribbling` on a PES 18 save: the version has no such field.
        let (mut save, _) = open(PesVersion::Pes18);
        let before = save.players().to_vec();
        let teams = save.teams().to_vec();
        let patch = patch_at(
            PesVersion::Pes18,
            vec![team(
                701,
                BTreeMap::from([(
                    5,
                    motion_only(MotionSettings {
                        dribbling: Some(2),
                        ..MotionSettings::default()
                    }),
                )]),
            )],
        );
        let err = patch.apply(&mut save).expect_err("refused");
        match &err {
            PatchError::Settings(SettingsError::NotInThisVersion { key }) => {
                assert_eq!(key, "teams[0].players.05.appearance.motion.dribbling")
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(save.players(), &before[..]);
        assert_eq!(save.teams(), &teams[..]);

        // `corner_kick` stored 8: inside the key's 1-to-10 range, past PES
        // 18's 3-bit field.
        let (mut save, _) = open(PesVersion::Pes18);
        let before = save.players().to_vec();
        let teams = save.teams().to_vec();
        let patch = patch_at(
            PesVersion::Pes18,
            vec![team(
                701,
                BTreeMap::from([(
                    5,
                    motion_only(MotionSettings {
                        corner_kick: Some(8),
                        ..MotionSettings::default()
                    }),
                )]),
            )],
        );
        let err = patch.apply(&mut save).expect_err("refused");
        assert!(
            matches!(
                err,
                PatchError::Settings(SettingsError::Codec(
                    crate::CodecError::ValueTooWide {
                        ref field,
                        value: 8,
                        width: 3,
                    }
                )) if field == "CornerKickMotion"
            ),
            "{err:?}"
        );
        assert_eq!(save.players(), &before[..]);
        assert_eq!(save.teams(), &teams[..]);
    }

    /// The kind's range check the field's width does not see: `free_kick`
    /// stored 20 is past the key's `1 to 20` (`OneBased`, stored < 20) but
    /// inside PES 20/21's 5-bit `FreeKickMotion` field (holds 0 to 31).
    /// Both halves of that premise are checked against the key table and
    /// the schema, so the test cannot go green on a narrowed field.
    #[test]
    fn a_motion_past_its_keys_range_but_inside_the_field_is_refused() {
        let stored = 20;
        assert!(
            !SettingKey::FreeKick.spec().kind.accepts_stored(stored),
            "outside the kind's range"
        );
        let width = schema_for(PesVersion::Pes21)
            .player
            .fields
            .iter()
            .find(|spec| spec.field == PlayerField::FreeKickMotion)
            .expect("the schema has the field")
            .bit_width;
        assert!(u32::from(stored) < (1 << width), "inside the field's width");
        let (mut save, _) = open(PesVersion::Pes21);
        let before = save.players().to_vec();
        let teams = save.teams().to_vec();
        let patch = patch_at(
            PesVersion::Pes21,
            vec![team(
                701,
                BTreeMap::from([(
                    5,
                    motion_only(MotionSettings {
                        free_kick: Some(stored),
                        ..MotionSettings::default()
                    }),
                )]),
            )],
        );
        let err = patch.apply(&mut save).expect_err("refused");
        match &err {
            PatchError::Settings(SettingsError::OutOfRange { key, value, .. }) => {
                assert_eq!(key, "teams[0].players.05.appearance.motion.free_kick");
                assert_eq!(*value, i64::from(stored));
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(save.players(), &before[..]);
        assert_eq!(save.teams(), &teams[..]);
    }
}
