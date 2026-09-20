//! `team.toml`'s player half: `[players.NN]` and its `stats`, `positions`,
//! `skills`, `edit_flags` and `appearance` subtables. The scalar keys run
//! through `player_keys`' table; the non-scalar keys (names, number,
//! `ingame_face`, `playable`, `playing_style`, the skill arrays and
//! `appearance`) are handled here.

use std::collections::{BTreeMap, HashSet};

use pes_version::PesVersion;
use toml_edit::{Item, TableLike, Value};

use crate::interchange::team_toml::labels;
use crate::interchange::team_toml::player_keys::PlayerKey;
use crate::interchange::team_toml::{
    ImportNote, PlayerSection, PositionsSection, SkillsSection, TeamTomlError,
};
use crate::model::ingame_face::IngameFace;
use crate::model::player::PlayerEntry;
use crate::model::playstyle::PlayStyle;
use crate::model::team::TeamEntry;
use crate::schema::fields::PlayerField;
use crate::schema::limits::face_type_cap;
use crate::schema::{playstyle, schema_for};
use crate::settings_toml::keys::{SettingKey, Source};
use crate::settings_toml::{
    AppearanceSettings, appearance_from, apply_appearance, emit_appearance, get_appearance,
    parse_appearance, set_appearance,
};

use super::team::{as_table, emit, gated, opt, padded, reject, text, u16_val};

// ---------------------------------------------------------------------------
// From the model

/// The `[players.NN]` map of `team`'s rostered players, keyed by 1-based
/// roster slot. An empty `players` is a tactics-only dump — no sections. A
/// non-empty pool that lacks a rostered id is `PlayerMissing`: a partial pool
/// is a caller bug, and silently dropping players is the loss this crate
/// exists to avoid.
pub(super) fn players_from(
    version: PesVersion,
    team: &TeamEntry,
    players: &[&PlayerEntry],
) -> Result<BTreeMap<u8, PlayerSection>, TeamTomlError> {
    let mut out = BTreeMap::new();
    if players.is_empty() {
        return Ok(out);
    }
    // The gate is a set membership, not a schema scan per key per player.
    let fields = schema_for(version).player_field_set();
    for (i, slot) in team.roster.iter().enumerate() {
        if slot.player_id == 0 {
            continue;
        }
        let player = players
            .iter()
            .find(|player| player.id == slot.player_id)
            .ok_or(TeamTomlError::PlayerMissing { id: slot.player_id })?;
        let key = u8::try_from(i + 1).expect("a roster slot index fits u8");
        out.insert(key, player_from(version, slot.number, player, &fields)?);
    }
    Ok(out)
}

/// One roster slot's `PlayerSection`: every field `Some` where the model
/// carries it (a version-gated `Option` stays `None` and emits commented).
fn player_from(
    version: PesVersion,
    number: u16,
    player: &PlayerEntry,
    fields: &HashSet<PlayerField>,
) -> Result<PlayerSection, TeamTomlError> {
    let mut section = PlayerSection {
        name: Some(player.name.clone()),
        shirt_name: Some(player.shirt_name.clone()),
        number: Some(number),
        ingame_face: Some(player.appearance.ingame_face.bytes().to_vec()),
        positions: PositionsSection {
            playable: Some(player.positions.playable),
            playing_style: Some(playstyle::decode(version, player.positions.playing_style)?),
            ..PositionsSection::default()
        },
        skills: SkillsSection {
            skills: Some(player.skills.skills),
            com_styles: Some(player.skills.com_styles),
        },
        appearance: appearance_from(player)?,
        ..PlayerSection::default()
    };
    for key in PlayerKey::ALL {
        if !fields.contains(&key.spec().field) {
            continue;
        }
        if let Some(value) = key.get(player) {
            key.set_section(&mut section, value)?;
        }
    }
    Ok(section)
}

// ---------------------------------------------------------------------------
// Emit

/// Every `PlayerSection`, in slot order.
pub(super) fn emit_players(
    out: &mut String,
    players: &BTreeMap<u8, PlayerSection>,
) -> Result<(), TeamTomlError> {
    for (slot, section) in players {
        emit_player(out, *slot, section)?;
    }
    Ok(())
}

/// One scalar key's line through the table.
fn emit_key(
    out: &mut String,
    section: &PlayerSection,
    key: PlayerKey,
) -> Result<(), TeamTomlError> {
    let spec = key.spec();
    let value = key.get_section(section).map(|v| key.text(v)).transpose()?;
    emit(out, key.name(), value, &key.neutral(), spec.comment);
    Ok(())
}

/// `path` with a `%02d` slot substituted for `NN`.
fn player_path(slot: u8) -> String {
    format!("players.{slot:02}")
}

fn emit_player(out: &mut String, slot: u8, section: &PlayerSection) -> Result<(), TeamTomlError> {
    let path = player_path(slot);
    out.push('\n');
    out.push_str(&padded(
        &format!("[{path}]"),
        "roster slot 01 .. 40 (32 on PES 15-18)",
    ));
    out.push('\n');
    emit(
        out,
        "name",
        section
            .name
            .as_ref()
            .map(|n| Value::from(n.as_str()).to_string()),
        "\"\"",
        "",
    );
    emit(
        out,
        "shirt_name",
        section
            .shirt_name
            .as_ref()
            .map(|n| Value::from(n.as_str()).to_string()),
        "\"\"",
        "",
    );
    emit(
        out,
        "number",
        section.number.map(|v| v.to_string()),
        "1",
        "shirt number, the roster's",
    );
    for key in PlayerKey::ALL {
        if key.table().is_empty() {
            emit_key(out, section, key)?;
        }
    }
    emit(
        out,
        "ingame_face",
        section.ingame_face.as_ref().map(|bytes| hex_text(bytes)),
        "\"\"",
        &format!(
            "the appearance block's undecoded run, hex, 50 bytes (46 on PES 15); applied before [{path}.appearance]"
        ),
    );

    out.push_str(&format!("\n[{path}.stats]\n"));
    for key in PlayerKey::ALL {
        if key.table() == "stats" {
            emit_key(out, section, key)?;
        }
    }

    out.push_str(&format!("\n[{path}.positions]\n"));
    emit_key(out, section, PlayerKey::Registered)?;
    emit(
        out,
        "playable",
        section.positions.playable.map(playable_text),
        &playable_text([0; 13]),
        "\"none\" | \"C\" | \"B\" | \"A\"",
    );
    emit(
        out,
        "playing_style",
        section
            .positions
            .playing_style
            .map(play_style_text)
            .transpose()?,
        "\"none\"",
        "canonical PlayStyle name; \"none\" for none",
    );
    emit_key(out, section, PlayerKey::StrongerFoot)?;
    emit_key(out, section, PlayerKey::StrongerHand)?;

    out.push_str(&format!("\n[{path}.skills]\n"));
    emit(
        out,
        "skills",
        section
            .skills
            .skills
            .map(|flags| flag_list(flags, &labels::SKILLS)),
        "[]",
        "the set skills, canonical names",
    );
    emit(
        out,
        "com_styles",
        section
            .skills
            .com_styles
            .map(|flags| flag_list(flags, &labels::COM_STYLES)),
        "[]",
        "\"trickster\" \"mazing_run\" \"speeding_bullet\" \"incisive_run\" \"long_ball_expert\" \"early_cross\" \"long_ranger\"",
    );

    out.push_str(&format!(
        "\n[{path}.edit_flags]         # the game's own flags, all bool\n"
    ));
    for key in PlayerKey::ALL {
        if key.table() == "edit_flags" {
            emit_key(out, section, key)?;
        }
    }

    emit_appearance(
        out,
        &format!("{path}.appearance"),
        &section.appearance,
        true,
    )?;
    Ok(())
}

/// `"0102…ff"` — lowercase hex.
fn hex_text(bytes: &[u8]) -> String {
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("\"{hex}\"")
}

/// `{ CF = "A", SS = "B", …, GK = "none" }` — model order (CF is index 0).
fn playable_text(playable: [u8; 13]) -> String {
    let entries: Vec<String> = (0..13)
        .map(|i| {
            let position = labels::POSITIONS[12 - i];
            let rating = labels::RATINGS[usize::from(playable[i])];
            format!("{position} = \"{rating}\"")
        })
        .collect();
    format!("{{ {} }}", entries.join(", "))
}

/// `"goal_poacher"` — the canonical label.
fn play_style_text(style: PlayStyle) -> Result<String, TeamTomlError> {
    let index = PlayStyle::ALL
        .iter()
        .position(|s| *s == style)
        .expect("a canonical style is in ALL");
    Ok(format!("\"{}\"", labels::PLAY_STYLES[index]))
}

/// `["a", "b"]` of the set flags' names, canonical order.
fn flag_list<const N: usize>(flags: [bool; N], labels: &[&'static str]) -> String {
    let names: Vec<String> = labels
        .iter()
        .enumerate()
        .filter(|(i, _)| flags[*i])
        .map(|(_, name)| format!("\"{name}\""))
        .collect();
    format!("[{}]", names.join(", "))
}

// ---------------------------------------------------------------------------
// Parse

/// The `[players]` table: keys are exactly two digits `01` to `40`, anything
/// else is `UnknownKey` ("players.7", "players.41", "players.bob").
pub(super) fn parse_players(item: &Item) -> Result<BTreeMap<u8, PlayerSection>, TeamTomlError> {
    let table = as_table(item, "players")?;
    let mut out = BTreeMap::new();
    for (name, item) in table.iter() {
        let valid = name.len() == 2
            && name.bytes().all(|b| b.is_ascii_digit())
            && name
                .parse::<u8>()
                .map(|slot| (1..=40).contains(&slot))
                .unwrap_or(false);
        if !valid {
            return Err(TeamTomlError::UnknownKey {
                key: format!("players.{name}"),
            });
        }
        let slot = name.parse::<u8>().expect("checked above");
        out.insert(slot, parse_player(item, &player_path(slot))?);
    }
    Ok(out)
}

fn parse_player(item: &Item, path: &str) -> Result<PlayerSection, TeamTomlError> {
    let table = as_table(item, path)?;
    let stats = sub_table(table, path, "stats")?;
    let positions = sub_table(table, path, "positions")?;
    let skills = sub_table(table, path, "skills")?;
    let edit_flags = sub_table(table, path, "edit_flags")?;
    let mut section = PlayerSection {
        name: opt(table, path, "name", text)?,
        shirt_name: opt(table, path, "shirt_name", text)?,
        number: opt(table, path, "number", u16_val)?,
        ingame_face: opt(table, path, "ingame_face", hex)?,
        ..PlayerSection::default()
    };
    for key in PlayerKey::ALL {
        let (sub, subpath) = match key.table() {
            "" => (Some(table), path.to_string()),
            "stats" => (stats, format!("{path}.stats")),
            "positions" => (positions, format!("{path}.positions")),
            "edit_flags" => (edit_flags, format!("{path}.edit_flags")),
            other => unreachable!("the key table only names known tables: {other}"),
        };
        let Some(sub) = sub else { continue };
        if let Some(value) = opt(sub, &subpath, key.name(), |i, k| key.parse(i, k))? {
            key.set_section(&mut section, value)?;
        }
    }
    if let Some(t) = positions {
        let positions_path = format!("{path}.positions");
        section.positions.playable = opt(t, &positions_path, "playable", playable)?;
        section.positions.playing_style = opt(t, &positions_path, "playing_style", play_style)?;
        reject(
            t,
            &positions_path,
            &[
                "registered",
                "playable",
                "playing_style",
                "stronger_foot",
                "stronger_hand",
            ],
        )?;
    }
    if let Some(t) = skills {
        let skills_path = format!("{path}.skills");
        section.skills.skills = opt(t, &skills_path, "skills", |i, k| {
            flag_array(i, k, &labels::SKILLS)
        })?;
        section.skills.com_styles = opt(t, &skills_path, "com_styles", |i, k| {
            flag_array(i, k, &labels::COM_STYLES)
        })?;
        reject(t, &skills_path, &["skills", "com_styles"])?;
    }
    if let Some(t) = stats {
        reject(t, &format!("{path}.stats"), &stats_names())?;
    }
    if let Some(t) = edit_flags {
        reject(t, &format!("{path}.edit_flags"), &flag_names())?;
    }
    if let Some(item) = table.get("appearance") {
        section.appearance = parse_appearance(item, &format!("{path}.appearance"), true)?;
    }
    reject(
        table,
        path,
        &[
            "name",
            "shirt_name",
            "number",
            "nationality",
            "age",
            "boots_id",
            "gloves_id",
            "base_copy_id",
            "ingame_face",
            "stats",
            "positions",
            "skills",
            "edit_flags",
            "appearance",
        ],
    )?;
    Ok(section)
}

/// `table.name` as a table-like, `None` when absent, `WrongType` when not a
/// table.
fn sub_table<'a>(
    table: &'a dyn TableLike,
    path: &str,
    name: &str,
) -> Result<Option<&'a dyn TableLike>, TeamTomlError> {
    let Some(item) = table.get(name) else {
        return Ok(None);
    };
    item.as_table_like()
        .map(Some)
        .ok_or_else(|| TeamTomlError::WrongType {
            key: format!("{path}.{name}"),
            expected: "a table",
        })
}

/// The `[players.NN.stats]` key names.
fn stats_names() -> Vec<&'static str> {
    PlayerKey::ALL
        .iter()
        .filter(|key| key.table() == "stats")
        .map(|key| key.name())
        .collect()
}

/// The `[players.NN.edit_flags]` key names.
fn flag_names() -> Vec<&'static str> {
    PlayerKey::ALL
        .iter()
        .filter(|key| key.table() == "edit_flags")
        .map(|key| key.name())
        .collect()
}

/// `"0102…ff"` decoded: even length, 46 or 50 bytes, all hex.
fn hex(item: &Item, key: &str) -> Result<Vec<u8>, TeamTomlError> {
    let expected = "a hex string of 46 or 50 bytes";
    let text =
        item.as_value()
            .and_then(|v| v.as_str())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
    if text.len() != 92 && text.len() != 100 {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected,
        });
    }
    let mut bytes = Vec::with_capacity(50);
    for i in (0..text.len()).step_by(2) {
        let byte =
            u8::from_str_radix(&text[i..i + 2], 16).map_err(|_| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
        bytes.push(byte);
    }
    Ok(bytes)
}

/// `playable = { GK = "none", …, CF = "A" }`: every position required, each a
/// rating label.
fn playable(item: &Item, key: &str) -> Result<[u8; 13], TeamTomlError> {
    let table = item
        .as_table_like()
        .ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "an inline table of thirteen position ratings",
        })?;
    let mut out = [0u8; 13];
    for (index, position) in labels::POSITIONS.iter().enumerate() {
        let Some(item) = table.get(position) else {
            return Err(TeamTomlError::MissingKey {
                key: format!("{key}.{position}"),
            });
        };
        let rating = label_index(item, &format!("{key}.{position}"), &labels::RATINGS)?;
        out[12 - index] = rating;
    }
    reject(table, key, &labels::POSITIONS)?;
    Ok(out)
}

/// A string that is one of `labels`, as its index; a stranger is
/// `UnknownLabel`.
fn label_index(item: &Item, key: &str, labels: &[&str]) -> Result<u8, TeamTomlError> {
    let text =
        item.as_value()
            .and_then(|v| v.as_str())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "a string",
            })?;
    labels
        .iter()
        .position(|label| *label == text)
        .map(|index| u8::try_from(index).expect("a label index fits u8"))
        .ok_or_else(|| TeamTomlError::UnknownLabel {
            key: key.to_string(),
            label: text.to_string(),
            allowed: labels
                .iter()
                .map(|label| format!("\"{label}\""))
                .collect::<Vec<_>>()
                .join(", "),
        })
}

/// `playing_style = "goal_poacher"`: a canonical `PlayStyle` label.
fn play_style(item: &Item, key: &str) -> Result<PlayStyle, TeamTomlError> {
    let index = label_index(item, key, &labels::PLAY_STYLES)?;
    Ok(PlayStyle::ALL[usize::from(index)])
}

/// An array of unique label names → the flag array with those slots set. A
/// duplicate or a name the list lacks is `UnknownLabel`.
fn flag_array<const N: usize>(
    item: &Item,
    key: &str,
    labels: &[&'static str],
) -> Result<[bool; N], TeamTomlError> {
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of names",
            })?;
    let mut out = [false; N];
    for value in array.iter() {
        let index = usize::from(label_index(&Item::Value(value.clone()), key, labels)?);
        let slot = out
            .get_mut(index)
            .expect("the label table and the flag array share a length");
        if *slot {
            return Err(TeamTomlError::UnknownLabel {
                key: key.to_string(),
                label: labels[index].to_string(),
                allowed: labels
                    .iter()
                    .map(|label| format!("\"{label}\""))
                    .collect::<Vec<_>>()
                    .join(", "),
            });
        }
        *slot = true;
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Apply

/// Applies every `[players.NN]`: `NN` addresses `team`'s roster slot; an
/// empty slot is an `EmptySlot` note, a slot past the roster's width is
/// `NoSuchSlot`, a rostered id missing from `players` is `PlayerMissing`.
/// `team_id` is `[team] id` for the base-copy own-id rule.
/// One `apply` call's immutable context: the target version, whether the
/// document is foreign, its schema, and `[team] id`.
struct Context {
    to: PesVersion,
    foreign: bool,
    /// The target's stored `PlayerField`s, built once per `apply`.
    fields: HashSet<PlayerField>,
    team_id: Option<u32>,
}

/// The context plus the things being mutated.
struct Target<'a> {
    context: Context,
    team: &'a mut TeamEntry,
    players: &'a mut [PlayerEntry],
    notes: &'a mut Vec<ImportNote>,
}

pub(super) fn apply_players(
    sections: &BTreeMap<u8, PlayerSection>,
    team_id: Option<u32>,
    to: PesVersion,
    foreign: bool,
    team: &mut TeamEntry,
    players: &mut [PlayerEntry],
    notes: &mut Vec<ImportNote>,
) -> Result<(), TeamTomlError> {
    let mut target = Target {
        context: Context {
            to,
            foreign,
            fields: schema_for(to).player_field_set(),
            team_id,
        },
        team,
        players,
        notes,
    };
    for (slot, section) in sections {
        apply_player(section, *slot, &mut target)?;
    }
    Ok(())
}

fn apply_player(
    section: &PlayerSection,
    slot: u8,
    target: &mut Target,
) -> Result<(), TeamTomlError> {
    let path = player_path(slot);
    let index = usize::from(slot - 1);
    if index >= target.team.roster.len() {
        return Err(TeamTomlError::NoSuchSlot { slot });
    }
    let player_id = target.team.roster[index].player_id;
    if player_id == 0 {
        target.notes.push(ImportNote::EmptySlot { slot });
        return Ok(());
    }
    let Some(player) = target.players.iter_mut().find(|p| p.id == player_id) else {
        return Err(TeamTomlError::PlayerMissing { id: player_id });
    };
    if let Some(number) = section.number {
        target.team.roster[index].number = number;
    }
    if let Some(name) = &section.name {
        player.name = name.clone();
    }
    if let Some(shirt_name) = &section.shirt_name {
        player.shirt_name = shirt_name.clone();
    }
    for key in PlayerKey::ALL {
        let Some(value) = key.get_section(section) else {
            continue;
        };
        let spec = key.spec();
        if !gated(
            target.notes,
            target.context.foreign,
            target.context.fields.contains(&spec.field),
            &format!("{path}.{}", spec.path),
        )? {
            continue;
        }
        let value = if key == PlayerKey::BaseCopyId
            && target
                .context
                .team_id
                .is_some_and(|id| value == id * 100 + u32::from(slot))
        {
            // The file's own-id convention: equal to team id x 100 + slot
            // means "unset" — the target's own id stands in.
            player.id
        } else {
            value
        };
        key.set(player, value)?;
    }
    if let Some(playable) = section.positions.playable {
        player.positions.playable = playable;
    }
    if let Some(style) = section.positions.playing_style {
        player.positions.playing_style = match playstyle::encode(target.context.to, style) {
            Some(stored) => stored,
            None => {
                let index = PlayStyle::ALL
                    .iter()
                    .position(|s| *s == style)
                    .expect("a canonical style is in ALL");
                target.notes.push(ImportNote::NotEncodable {
                    path: format!("{path}.positions.playing_style"),
                    label: labels::PLAY_STYLES[index].to_string(),
                });
                playstyle::encode(target.context.to, PlayStyle::None)
                    .expect("every version's style list holds PlayStyle::None")
            }
        };
    }
    if let Some(skills) = section.skills.skills {
        for (i, set) in skills.iter().enumerate() {
            let field = PlayerField::Skill(u8::try_from(i).expect("41 skills fit u8"));
            if *set {
                if gated(
                    target.notes,
                    target.context.foreign,
                    target.context.fields.contains(&field),
                    &format!("{path}.skills.skills.{}", labels::SKILLS[i]),
                )? {
                    player.skills.skills[i] = true;
                }
            } else if target.context.fields.contains(&field) {
                player.skills.skills[i] = false;
            }
        }
    }
    if let Some(com_styles) = section.skills.com_styles {
        for (i, set) in com_styles.iter().enumerate() {
            let field = PlayerField::ComStyle(u8::try_from(i).expect("7 com styles fit u8"));
            if *set {
                if gated(
                    target.notes,
                    target.context.foreign,
                    target.context.fields.contains(&field),
                    &format!("{path}.skills.com_styles.{}", labels::COM_STYLES[i]),
                )? {
                    player.skills.com_styles[i] = true;
                }
            } else if target.context.fields.contains(&field) {
                player.skills.com_styles[i] = false;
            }
        }
    }
    if let Some(bytes) = &section.ingame_face {
        // The face run lands first: the appearance table's keys write into it.
        player
            .appearance
            .ingame_face
            .copy_from(&IngameFace::from_bytes(bytes.clone()));
    }
    let context = &target.context;
    apply_appearance_table(&section.appearance, player, context, target.notes, &path)?;
    Ok(())
}

/// The `[players.NN.appearance]` keys onto `player`. A foreign document's
/// keys the target cannot hold are pre-adjusted on a clone: a `PlayerField`
/// the version lacks is dropped with a `NotInThisVersion` note, a face type
/// over `face_type_cap` is capped with `Capped`, and skin 7 without a custom
/// skin becomes 1. Same-version documents write the values as stored (a
/// `SettingsError` — a `Some` the version lacks — propagates).
fn apply_appearance_table(
    section: &AppearanceSettings,
    player: &mut PlayerEntry,
    context: &Context,
    notes: &mut Vec<ImportNote>,
    path: &str,
) -> Result<(), TeamTomlError> {
    let mut appearance = section.clone();
    if context.foreign {
        for key in SettingKey::ALL {
            let Some(value) = get_appearance(&appearance, key) else {
                continue;
            };
            let leaf = format!(
                "{path}.appearance{}.{}",
                &key.spec().table["appearance".len()..],
                key.spec().name
            );
            match key.source() {
                Source::Player(field) => {
                    if !context.fields.contains(&field) {
                        set_appearance(&mut appearance, key, None);
                        notes.push(ImportNote::NotInThisVersion { path: leaf });
                    }
                }
                Source::Face(field) => {
                    if let Some(cap) = face_type_cap(context.to, field)
                        && value > cap
                    {
                        set_appearance(&mut appearance, key, Some(cap));
                        notes.push(ImportNote::Capped {
                            path: leaf,
                            from: value,
                            to: cap,
                        });
                    }
                }
            }
        }
        if appearance.skin_color == Some(7) && !fpc::custom_skin_available(context.to) {
            set_appearance(&mut appearance, SettingKey::SkinColor, Some(1));
            notes.push(ImportNote::Capped {
                path: format!("{path}.appearance.skin_color"),
                from: 7,
                to: 1,
            });
        }
    }
    apply_appearance(&appearance, player)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests

/// How many of `section`'s keys are `Some` — the document's density, for the
/// tests' "densest team" text-pass sample.
#[cfg(test)]
pub(super) fn some_keys(section: &PlayerSection) -> usize {
    let scalars = PlayerKey::ALL
        .iter()
        .filter(|key| key.get_section(section).is_some())
        .count();
    let named = [
        section.name.is_some(),
        section.shirt_name.is_some(),
        section.number.is_some(),
        section.ingame_face.is_some(),
        section.positions.playable.is_some(),
        section.positions.playing_style.is_some(),
        section.skills.skills.is_some(),
        section.skills.com_styles.is_some(),
    ];
    let appearance = SettingKey::ALL
        .iter()
        .filter(|key| get_appearance(&section.appearance, **key).is_some())
        .count();
    scalars + named.iter().filter(|set| **set).count() + appearance
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file::EditFile;
    use crate::interchange::team_toml::{TeamToml, TeamTomlError};
    use crate::schema::ingame_face::IngameFaceField;
    use crate::test_support::{FIXTURES, open};

    /// A foreign stand-in for each of `team`'s rostered slots: a real player
    /// of the same save that is not on this roster, re-id'd to the slot.
    fn foreign_players(file: &EditFile, team: &TeamEntry) -> Vec<PlayerEntry> {
        let rostered: Vec<u32> = team.roster.iter().map(|slot| slot.player_id).collect();
        let mut others = file
            .players()
            .iter()
            .filter(|player| !rostered.contains(&player.id));
        team.roster
            .iter()
            .filter(|slot| slot.player_id != 0)
            .map(|slot| {
                let mut player = others.next().expect("a spare player").clone();
                player.id = slot.player_id;
                player
            })
            .collect()
    }

    /// (a) Every fixture version, every team with players: the document
    /// reparses to itself, and applying onto foreign stand-ins reproduces
    /// every source `PlayerEntry` field for field, plus the roster numbers.
    #[test]
    fn every_fixture_player_round_trips_through_the_document() {
        for version in FIXTURES {
            let (file, _) = open(version);
            let refs: Vec<&PlayerEntry> = file.players().iter().collect();
            // The text pass (to_toml -> parse == document) costs tens of ms a
            // team, mostly toml_edit; the model pass (from_team -> apply onto
            // foreign targets -> field-for-field) is ~1 ms and stays
            // exhaustive. Text runs on a sample: the first rostered team, the
            // last, and the team whose players carry the most `Some` keys.
            let rostered: Vec<&TeamEntry> = file
                .teams()
                .iter()
                .filter(|team| team.roster.iter().any(|slot| slot.player_id != 0))
                .collect();
            let docs: Vec<TeamToml> = rostered
                .iter()
                .map(|team| TeamToml::from_team(version, team, &refs).expect("from_team"))
                .collect();
            let fullest = docs
                .iter()
                .enumerate()
                .max_by_key(|(_, doc)| doc.players.values().map(some_keys).sum::<usize>())
                .map(|(n, _)| n)
                .expect("a rostered team exists");
            for (n, (team, doc)) in rostered.iter().zip(&docs).enumerate() {
                if n == 0 || n + 1 == rostered.len() || n == fullest {
                    let text = doc.to_toml().expect("to_toml");
                    let parsed = TeamToml::parse(&text)
                        .unwrap_or_else(|e| panic!("{version:?} team {} reparse: {e}", team.id));
                    assert_eq!(parsed, *doc, "{version:?} team {}", team.id);
                }

                let mut target_team = (*team).clone();
                let mut targets = foreign_players(&file, team);
                let notes = doc
                    .apply(version, &mut target_team, &mut targets)
                    .unwrap_or_else(|e| panic!("{version:?} team {} apply: {e}", team.id));
                assert!(notes.is_empty(), "{version:?} team {}: {notes:?}", team.id);
                assert_eq!(
                    target_team.roster, team.roster,
                    "{version:?} team {} roster",
                    team.id
                );
                for slot in &team.roster {
                    if slot.player_id == 0 {
                        continue;
                    }
                    let source = *refs
                        .iter()
                        .find(|player| player.id == slot.player_id)
                        .expect("the rostered player");
                    let target = targets
                        .iter()
                        .find(|player| player.id == slot.player_id)
                        .expect("the stand-in");
                    assert_eq!(
                        target, source,
                        "{version:?} team {} player {}",
                        team.id, slot.player_id
                    );
                }
            }
        }
    }

    /// (b) A one-key player document changes exactly that stat of exactly
    /// that player.
    #[test]
    fn a_one_key_player_document_changes_exactly_that_stat() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file
            .teams()
            .iter()
            .find(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            .expect("a team with players");
        let slot = team
            .roster
            .iter()
            .position(|slot| slot.player_id != 0)
            .expect("a rostered slot");
        let doc = TeamToml::parse(&format!(
            "[players.{:02}.stats]\nfinishing = 88\n",
            slot + 1
        ))
        .expect("parses");
        let mut target_team = team.clone();
        let mut players = file.players().to_vec();
        let notes = doc
            .apply(PesVersion::Pes19, &mut target_team, &mut players)
            .expect("applies");
        assert!(notes.is_empty());
        assert_eq!(target_team.roster, team.roster);
        let id = team.roster[slot].player_id;
        let mut expected = players.clone();
        expected
            .iter_mut()
            .find(|player| player.id == id)
            .expect("the player")
            .stats
            .finishing = 88;
        assert_eq!(players, expected);
    }

    /// (c) A PES 21 document applied to PES 19: the 20/21-only keys are
    /// `NotInThisVersion` notes, not errors, everything else applies, and a
    /// face type over the 19 cap is `Capped`.
    #[test]
    fn a_pes21_document_on_a_pes19_target_notes_and_caps() {
        let (file21, _) = open(PesVersion::Pes21);
        let (file19, _) = open(PesVersion::Pes19);
        let team21 = file21
            .teams()
            .iter()
            .find(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            .expect("a team with players");
        let refs: Vec<&PlayerEntry> = file21.players().iter().collect();
        let mut doc = TeamToml::from_team(PesVersion::Pes21, team21, &refs).expect("from_team");

        // Push the first player's facial hair type to the PES 21 cap so the PES
        // 19 cap has to bite.
        let cap21 = face_type_cap(PesVersion::Pes21, IngameFaceField::FacialHairType)
            .expect("a PES 21 cap");
        let cap19 = face_type_cap(PesVersion::Pes19, IngameFaceField::FacialHairType)
            .expect("a PES 19 cap");
        assert!(cap21 > cap19, "the test needs the 21 cap above the 19 cap");
        doc.players
            .values_mut()
            .next()
            .expect("a player")
            .appearance
            .face
            .facial_hair_type = Some(cap21);

        let team19 = file19
            .teams()
            .iter()
            .find(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            .expect("a team with players");
        let mut target_team = team19.clone();
        let mut targets: Vec<PlayerEntry> = team19
            .roster
            .iter()
            .filter(|slot| slot.player_id != 0)
            .filter_map(|slot| {
                file19
                    .players()
                    .iter()
                    .find(|player| player.id == slot.player_id)
                    .cloned()
            })
            .collect();
        let notes = doc
            .apply(PesVersion::Pes19, &mut target_team, &mut targets)
            .expect("foreign apply is notes, not errors");

        for suffix in [
            ".stats.tight_possession",
            ".stats.aggression",
            ".stats.playing_attitude",
            ".positions.stronger_hand",
            ".appearance.motion.dribbling",
        ] {
            assert!(
                notes.iter().any(|note| matches!(
                    note,
                    ImportNote::NotInThisVersion { path }
                        if path.starts_with("players.") && path.ends_with(suffix)
                )),
                "a NotInThisVersion note ending in {suffix}: {notes:?}"
            );
        }
        assert!(
            notes.iter().any(|note| matches!(
                note,
                ImportNote::Capped { path, from, to }
                    if path.ends_with(".appearance.face.facial_hair_type")
                        && *from == cap21
                        && *to == cap19
            )),
            "a Capped note for facial_hair_type: {notes:?}"
        );
        // A field that exists on both versions applied.
        let first = doc.players.values().next().expect("a player");
        let target = targets.first().expect("a player");
        assert_eq!(Some(&target.name), first.name.as_ref());
    }

    /// (d) A 50-byte PES 16 face run writes its first 46 bytes onto a PES 15
    /// record, without a note for the length difference.
    #[test]
    fn a_longer_ingame_face_writes_its_prefix_without_a_note() {
        let (file, _) = open(PesVersion::Pes15);
        let team = file
            .teams()
            .iter()
            .find(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            .expect("a team with players");
        let slot = team
            .roster
            .iter()
            .position(|slot| slot.player_id != 0)
            .expect("a rostered slot");
        let source: Vec<u8> = (0u8..50).collect();
        let hex: String = source.iter().map(|b| format!("{b:02x}")).collect();
        let doc = TeamToml::parse(&format!(
            "pes_version = 16\n\n[players.{:02}]\ningame_face = \"{hex}\"\n",
            slot + 1
        ))
        .expect("parses");
        let mut target_team = team.clone();
        let mut players = file.players().to_vec();
        let notes = doc
            .apply(PesVersion::Pes15, &mut target_team, &mut players)
            .expect("applies");
        assert!(
            !notes.iter().any(|note| matches!(
                note,
                ImportNote::NotInThisVersion { path } if path.contains("ingame_face")
            )),
            "no face-length note: {notes:?}"
        );
        let id = team.roster[slot].player_id;
        let target = players
            .iter()
            .find(|player| player.id == id)
            .expect("the player");
        assert_eq!(target.appearance.ingame_face.bytes(), &source[..46]);
    }

    /// (e) A slot with no player is an `EmptySlot` note; a slot past the
    /// roster's width is `NoSuchSlot`.
    #[test]
    fn empty_and_missing_slots() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file.teams().first().expect("a team");
        assert!(team.roster.len() >= 30, "the fixture roster has 30+ slots");
        let mut target_team = team.clone();
        target_team.roster[29].player_id = 0;
        let mut players = file.players().to_vec();
        let doc = TeamToml::parse("[players.30]\nname = \"GHOST\"\n").expect("parses");
        let notes = doc
            .apply(PesVersion::Pes19, &mut target_team, &mut players)
            .expect("an empty slot is a note");
        assert_eq!(notes, vec![ImportNote::EmptySlot { slot: 30 }]);

        let (file18, _) = open(PesVersion::Pes18);
        let team18 = file18.teams().first().expect("a team");
        assert_eq!(team18.roster.len(), 32, "PES 18 rosters are 32 wide");
        let doc = TeamToml::parse("[players.40]\nname = \"GHOST\"\n").expect("parses");
        let mut target18 = team18.clone();
        let mut players18 = file18.players().to_vec();
        let err = doc
            .apply(PesVersion::Pes18, &mut target18, &mut players18)
            .expect_err("slot 40 does not exist");
        assert!(matches!(err, TeamTomlError::NoSuchSlot { slot: 40 }));
    }

    /// The `players` pool contract: an empty slice is a tactics-only dump; a
    /// non-empty pool missing a rostered id is `PlayerMissing`, never a
    /// silent drop.
    #[test]
    fn the_player_pool_is_all_or_nothing() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file
            .teams()
            .iter()
            .find(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            .expect("a rostered team");
        let tactics_only = TeamToml::from_team(PesVersion::Pes19, team, &[])
            .expect("an empty pool is a tactics-only dump");
        assert!(tactics_only.players.is_empty());

        let missing_id = team
            .roster
            .iter()
            .find(|slot| slot.player_id != 0)
            .expect("a rostered slot")
            .player_id;
        let pool: Vec<&PlayerEntry> = file
            .players()
            .iter()
            .filter(|player| player.id != missing_id)
            .collect();
        let err = TeamToml::from_team(PesVersion::Pes19, team, &pool)
            .expect_err("a partial pool is a caller bug");
        assert!(
            matches!(err, TeamTomlError::PlayerMissing { id } if id == missing_id),
            "{err:?}"
        );
    }

    /// (f) `base_copy_id` equal to `[team] id` x 100 + slot writes the
    /// target's own id; any other value — or no `[team] id` — applies as
    /// written.
    #[test]
    fn the_base_copy_own_id_convention() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file
            .teams()
            .iter()
            .find(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            .expect("a team with players");
        let slot = team
            .roster
            .iter()
            .position(|slot| slot.player_id != 0)
            .expect("a rostered slot");
        let id = team.roster[slot].player_id;
        let path = format!("players.{:02}", slot + 1);
        let convention = 5 * 100 + u32::try_from(slot + 1).expect("a small slot");

        let doc = TeamToml::parse(&format!(
            "[team]\nid = 5\n\n[{path}]\nbase_copy_id = {convention}\n"
        ))
        .expect("parses");
        let mut target_team = team.clone();
        let mut players = file.players().to_vec();
        doc.apply(PesVersion::Pes19, &mut target_team, &mut players)
            .expect("applies");
        let target = players
            .iter()
            .find(|player| player.id == id)
            .expect("the player");
        assert_eq!(target.appearance.base_copy_id, id);

        for (text, expected) in [
            (
                format!(
                    "[team]\nid = 5\n\n[{path}]\nbase_copy_id = {}\n",
                    convention + 1
                ),
                convention + 1,
            ),
            (
                format!("[{path}]\nbase_copy_id = {convention}\n"),
                convention,
            ),
        ] {
            let doc = TeamToml::parse(&text).expect("parses");
            let mut target_team = team.clone();
            let mut players = file.players().to_vec();
            doc.apply(PesVersion::Pes19, &mut target_team, &mut players)
                .expect("applies");
            let target = players
                .iter()
                .find(|player| player.id == id)
                .expect("the player");
            assert_eq!(target.appearance.base_copy_id, expected, "{text:?}");
        }
    }

    /// (g) Player parse errors.
    #[test]
    fn malformed_player_keys_are_errors() {
        let err = TeamToml::parse("[players.7]\nname = \"X\"\n").expect_err("players.7");
        assert!(matches!(
            err,
            TeamTomlError::UnknownKey { ref key } if key == "players.7"
        ));
        let err = TeamToml::parse("[players.41]\nname = \"X\"\n").expect_err("players.41");
        assert!(matches!(
            err,
            TeamTomlError::UnknownKey { ref key } if key == "players.41"
        ));
        let err = TeamToml::parse("[players.01.skills]\nskills = [\"heading\", \"heading\"]\n")
            .expect_err("a duplicate skill");
        assert!(matches!(
            err,
            TeamTomlError::UnknownLabel {
                ref key,
                ref label,
                ..
            } if key == "players.01.skills.skills" && label == "heading"
        ));
        let err = TeamToml::parse("[players.01.positions]\nplayable = { CF = \"A\" }\n")
            .expect_err("a missing position");
        assert!(matches!(
            err,
            TeamTomlError::MissingKey { ref key }
                if key == "players.01.positions.playable.GK"
        ));
        let err = TeamToml::parse("[players.01]\ningame_face = \"zz\"\n").expect_err("bad hex");
        assert!(matches!(
            err,
            TeamTomlError::WrongType { ref key, .. } if key == "players.01.ingame_face"
        ));
    }

    /// The stored-ranges deviation, kept deliberate: a real save's
    /// above-the-editor-cap face type rides the document unchanged (and
    /// `settings.toml`'s strict `from_player` still refuses it).
    #[test]
    fn a_face_type_above_the_editor_cap_round_trips() {
        use crate::settings_toml::{PlayerSettings, SettingsError};

        let (file, _) = open(PesVersion::Pes15);
        let over_cap = |player: &PlayerEntry| {
            player
                .appearance
                .ingame_face
                .get(IngameFaceField::CheekType)
                .is_ok_and(|v| v > 3)
        };
        let team = file
            .teams()
            .iter()
            .find(|team| {
                team.roster.iter().any(|slot| {
                    slot.player_id != 0
                        && file
                            .players()
                            .iter()
                            .find(|p| p.id == slot.player_id)
                            .is_some_and(over_cap)
                })
            })
            .expect("a team with an over-cap player");
        let player = team
            .roster
            .iter()
            .filter(|slot| slot.player_id != 0)
            .filter_map(|slot| file.players().iter().find(|p| p.id == slot.player_id))
            .find(|p| over_cap(p))
            .expect("the over-cap player");

        // settings.toml's contract is intact: it refuses the same player.
        assert!(matches!(
            PlayerSettings::from_player(player),
            Err(SettingsError::OutOfRange { .. })
        ));

        let refs: Vec<&PlayerEntry> = file.players().iter().collect();
        let doc = TeamToml::from_team(PesVersion::Pes15, team, &refs).expect("from_team");
        let text = doc.to_toml().expect("to_toml");
        let parsed = TeamToml::parse(&text).expect("stored ranges reparse");
        assert_eq!(parsed, doc);
        let mut target_team = team.clone();
        let mut targets = foreign_players(&file, team);
        parsed
            .apply(PesVersion::Pes15, &mut target_team, &mut targets)
            .expect("applies");
        let target = targets
            .iter()
            .find(|p| p.id == player.id)
            .expect("the stand-in");
        assert_eq!(target, player);
    }
}
