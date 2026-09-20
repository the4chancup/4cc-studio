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
use crate::schema::fields::{PlayerField, PlayerText, RosterField};
use crate::schema::ingame_face::INGAME_FACE_FIELDS;
use crate::schema::limits::face_type_cap;
use crate::schema::{bit_width, playstyle, schema_for};
use crate::settings_toml::keys::{SettingKey, Source};
use crate::settings_toml::{
    AppearanceSettings, appearance_from, apply_appearance, emit_appearance, get_appearance,
    parse_appearance, set_appearance,
};

use super::team::{
    as_table, check_text, check_width, emit, gated, opt, padded, reject, text, text_max, u16_val,
};

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
    // Byte length first, then ASCII-hex: a non-ASCII character of the right
    // byte length must be an error, not a slice panic mid-character.
    if (text.len() != 92 && text.len() != 100) || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected,
        });
    }
    let bytes = text
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).expect("hex digits are UTF-8"), 16)
                .expect("hex digits parse")
        })
        .collect();
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
    /// The source document's stored `PlayerField`s: a key the source version
    /// cannot store says nothing about the target's value.
    from_fields: HashSet<PlayerField>,
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
    from: PesVersion,
    to: PesVersion,
    team: &mut TeamEntry,
    players: &mut [PlayerEntry],
    notes: &mut Vec<ImportNote>,
) -> Result<(), TeamTomlError> {
    let foreign = from != to;
    let mut target = Target {
        context: Context {
            to,
            foreign,
            fields: schema_for(to).player_field_set(),
            from_fields: schema_for(from).player_field_set(),
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
    // Checks first: a stored value the target's field cannot hold would fail
    // `write_player` after `apply` returned Ok, so it is refused up front.
    for key in PlayerKey::ALL {
        let Some(value) = key.get_section(section) else {
            continue;
        };
        let Some(width) = bit_width(target.context.to, key.spec().field) else {
            continue;
        };
        check_width(&format!("{path}.{}", key.spec().path), value, width)?;
    }
    let Some(player) = target.players.iter_mut().find(|p| p.id == player_id) else {
        return Err(TeamTomlError::PlayerMissing { id: player_id });
    };
    if let Some(number) = section.number {
        // The roster's number field is 8 bits on the ≤ 18 schemas; a wider
        // value would fail `write_roster` after `apply` said Ok, so it is
        // refused here instead of clamped.
        let max = schema_for(target.context.to)
            .roster
            .arrays
            .iter()
            .find(|a| matches!((a.make)(0), RosterField::Number(0)))
            .map_or(u32::from(u16::MAX), |a| (1 << a.bit_width) - 1);
        if u32::from(number) > max {
            return Err(TeamTomlError::OutOfRange {
                key: format!("{path}.number"),
                value: i64::from(number),
                range: format!("0 to {max}"),
            });
        }
        target.team.roster[index].number = number;
    }
    if let Some(name) = &section.name {
        check_text(
            &format!("{path}.name"),
            name,
            text_max(schema_for(target.context.to).player.texts, PlayerText::Name),
            false,
        )?;
        player.name = name.clone();
    }
    if let Some(shirt_name) = &section.shirt_name {
        check_text(
            &format!("{path}.shirt_name"),
            shirt_name,
            text_max(
                schema_for(target.context.to).player.texts,
                PlayerText::ShirtName,
            ),
            true,
        )?;
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
            && target.context.team_id.is_some_and(|id| {
                id.checked_mul(100)
                    .and_then(|base| base.checked_add(u32::from(slot)))
                    == Some(value)
            }) {
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
            // A skill the source version cannot store says nothing — the
            // target's value stands, note-free (a pair-wide rule).
            if !target.context.from_fields.contains(&field) {
                continue;
            }
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
            if !target.context.from_fields.contains(&field) {
                continue;
            }
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
/// over `face_type_cap` resets to 0 with a `Capped` note, and skin 7 on a
/// target without a custom skin becomes 1 (2.17f's conversion rules).
/// Same-version documents write the values as stored (a
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
            let leaf = appearance_leaf(path, key);
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
                        // 2.17f's rule: an unencodable face type resets to 0,
                        // it is not clamped to the target's cap.
                        set_appearance(&mut appearance, key, Some(0));
                        notes.push(ImportNote::Capped {
                            path: leaf,
                            from: value,
                            to: 0,
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
    // The same width check on the post-adjustment values: a value the
    // target's field cannot hold is refused before any write.
    for key in SettingKey::ALL {
        let Some(value) = get_appearance(&appearance, key) else {
            continue;
        };
        let width = match key.source() {
            Source::Player(field) => bit_width(context.to, field),
            Source::Face(field) => INGAME_FACE_FIELDS
                .iter()
                .find(|spec| spec.field == field)
                .map(|spec| spec.bit_width),
        };
        let Some(width) = width else {
            continue;
        };
        check_width(&appearance_leaf(path, key), u32::from(value), width)?;
    }
    apply_appearance(&appearance, player)?;
    Ok(())
}

/// The dotted path of an appearance key's leaf (`…appearance.strip.sleeves`).
fn appearance_leaf(path: &str, key: SettingKey) -> String {
    format!(
        "{path}.appearance{}.{}",
        &key.spec().table["appearance".len()..],
        key.spec().name
    )
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
        // The expectation is built from a pre-apply snapshot: deriving it
        // from `players` after `apply` would let any amount of clobbering
        // pass, since expected and actual would share the corruption.
        let mut expected = players.clone();
        let id = team.roster[slot].player_id;
        expected
            .iter_mut()
            .find(|player| player.id == id)
            .expect("the player")
            .stats
            .finishing = 88;
        let notes = doc
            .apply(PesVersion::Pes19, &mut target_team, &mut players)
            .expect("applies");
        assert!(notes.is_empty());
        assert_eq!(target_team.roster, team.roster);
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

        // Push the first player's facial hair type to the PES 21 cap so it
        // resets to 0 under the tighter PES 19 cap.
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
                        && *to == 0
            )),
            "a Capped note for facial_hair_type, reset to 0: {notes:?}"
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

    /// The first rostered team of `file` and its first rostered slot.
    fn first_slot(file: &EditFile) -> (&TeamEntry, usize) {
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
        (team, slot)
    }

    /// A `[players.NN]`-only document carrying `section` at `slot` (0-based).
    fn one_section(from: PesVersion, slot: usize, section: PlayerSection) -> TeamToml {
        TeamToml {
            pes_version: Some(from),
            players: BTreeMap::from([(u8::try_from(slot + 1).expect("a slot index"), section)]),
            ..TeamToml::default()
        }
    }

    /// A skill the document's source version cannot store is not written at
    /// all — the target's value stands, note-free (a pair-wide rule). The
    /// same `false` under the target's own version clears it.
    #[test]
    fn a_skill_the_source_version_lacks_is_left_untouched() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let id = team.roster[slot].player_id;
        // Double Touch: stored on PES 19+, absent on 15/16 (28 skills there).
        const DOUBLE_TOUCH: usize = 28;
        assert!(!schema_for(PesVersion::Pes16).player_has(PlayerField::Skill(28)));
        assert!(schema_for(PesVersion::Pes19).player_has(PlayerField::Skill(28)));

        let mut section = PlayerSection::default();
        section.skills.skills = Some([false; 41]);
        let mut players = file.players().to_vec();
        players
            .iter_mut()
            .find(|player| player.id == id)
            .expect("the player")
            .skills
            .skills[DOUBLE_TOUCH] = true;

        let notes = one_section(PesVersion::Pes16, slot, section.clone())
            .apply(PesVersion::Pes19, &mut team.clone(), &mut players)
            .expect("applies");
        assert!(notes.is_empty(), "{notes:?}");
        assert!(
            players
                .iter()
                .find(|player| player.id == id)
                .expect("the player")
                .skills
                .skills[DOUBLE_TOUCH],
            "a skill PES 16 cannot store is left alone"
        );

        let notes = one_section(PesVersion::Pes19, slot, section)
            .apply(PesVersion::Pes19, &mut team.clone(), &mut players)
            .expect("applies");
        assert!(notes.is_empty(), "{notes:?}");
        assert!(
            !players
                .iter()
                .find(|player| player.id == id)
                .expect("the player")
                .skills
                .skills[DOUBLE_TOUCH],
            "the same false under the target's own version clears"
        );
    }

    /// A face type over the target's cap resets to 0 with a `Capped` note
    /// (2.17f's rule — never a clamp); a value at the cap is not capped.
    #[test]
    fn a_face_type_over_the_cap_resets_and_at_the_cap_is_untouched() {
        // Facial hair: capped at 12 on PES 15-19, at 19 on 20/21.
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let id = team.roster[slot].player_id;
        let face_value = |players: &[PlayerEntry]| {
            players
                .iter()
                .find(|player| player.id == id)
                .expect("the player")
                .appearance
                .ingame_face
                .get(IngameFaceField::FacialHairType)
                .expect("a stored type")
        };

        let mut section = PlayerSection::default();
        section.appearance.face.facial_hair_type = Some(15);
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        let notes = one_section(PesVersion::Pes21, slot, section)
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("applies");
        assert_eq!(
            notes,
            vec![ImportNote::Capped {
                path: format!("players.{:02}.appearance.face.facial_hair_type", slot + 1),
                from: 15,
                to: 0,
            }]
        );
        assert_eq!(face_value(&players), 0);

        let mut section = PlayerSection::default();
        section.appearance.face.facial_hair_type = Some(12);
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        let notes = one_section(PesVersion::Pes21, slot, section)
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("applies");
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(face_value(&players), 12);
    }

    /// Skin 7 resets to 1 with a `Capped` note where the target has no
    /// custom skin (PES 18+), stands on PES 17, and an ordinary preset skin
    /// never notes.
    #[test]
    fn skin_seven_resets_only_where_the_target_has_no_custom_skin() {
        let apply = |from: PesVersion, to: PesVersion, skin: u8| {
            let (file, _) = open(to);
            let (team, slot) = first_slot(&file);
            let id = team.roster[slot].player_id;
            let mut section = PlayerSection::default();
            section.appearance.skin_color = Some(skin);
            let mut target = team.clone();
            let mut players = file.players().to_vec();
            let notes = one_section(from, slot, section)
                .apply(to, &mut target, &mut players)
                .expect("applies");
            let stored = players
                .iter()
                .find(|player| player.id == id)
                .expect("the player")
                .appearance
                .ingame_face
                .get(IngameFaceField::SkinColor)
                .expect("a stored skin");
            (slot, notes, stored)
        };

        let (slot, notes, stored) = apply(PesVersion::Pes21, PesVersion::Pes19, 7);
        assert_eq!(
            notes,
            vec![ImportNote::Capped {
                path: format!("players.{:02}.appearance.skin_color", slot + 1),
                from: 7,
                to: 1,
            }]
        );
        assert_eq!(stored, 1);

        let (_, notes, stored) = apply(PesVersion::Pes19, PesVersion::Pes17, 7);
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(stored, 7);

        let (_, notes, stored) = apply(PesVersion::Pes21, PesVersion::Pes18, 2);
        assert!(notes.is_empty(), "{notes:?}");
        assert_eq!(stored, 2);
    }

    /// A name longer than the target field minus its terminator is
    /// `TextTooLong`, returned before anything is written.
    #[test]
    fn a_name_longer_than_the_field_is_refused_and_writes_nothing() {
        let (file, _) = open(PesVersion::Pes16);
        let (team, slot) = first_slot(&file);
        let max = usize::try_from(
            schema_for(PesVersion::Pes16)
                .player
                .texts
                .iter()
                .find(|spec| spec.text == PlayerText::Name)
                .expect("a name field")
                .len
                - 1,
        )
        .expect("fits usize");
        assert_eq!(max, 45, "the field holds 46 bytes, terminator included");

        let section = PlayerSection {
            name: Some("N".repeat(60)),
            ..PlayerSection::default()
        };
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        let err = one_section(PesVersion::Pes19, slot, section)
            .apply(PesVersion::Pes16, &mut target, &mut players)
            .expect_err("too long");
        assert!(
            matches!(
                err,
                TeamTomlError::TextTooLong { ref path, max: 45 }
                    if *path == format!("players.{:02}.name", slot + 1)
            ),
            "{err:?}"
        );
        assert_eq!(&target, team, "nothing was written");
    }

    /// A name or shirt name of exactly the field's capacity applies; one
    /// byte over is `TextTooLong`.
    #[test]
    fn a_player_text_at_the_fields_capacity_applies_and_one_more_is_refused() {
        let (file, _) = open(PesVersion::Pes16);
        let (team, slot) = first_slot(&file);
        let id = team.roster[slot].player_id;
        let texts = schema_for(PesVersion::Pes16).player.texts;
        let name_max = text_max(texts, PlayerText::Name);
        let shirt_max = text_max(texts, PlayerText::ShirtName);

        let section = PlayerSection {
            name: Some("N".repeat(name_max)),
            shirt_name: Some("S".repeat(shirt_max)),
            ..PlayerSection::default()
        };
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        one_section(PesVersion::Pes16, slot, section)
            .apply(PesVersion::Pes16, &mut target, &mut players)
            .expect("at capacity applies");
        let applied = players
            .iter()
            .find(|player| player.id == id)
            .expect("the player");
        assert_eq!(applied.name, "N".repeat(name_max));
        assert_eq!(applied.shirt_name, "S".repeat(shirt_max));

        for (name_over, shirt_over, leaf, max) in [
            (true, false, "name", name_max),
            (false, true, "shirt_name", shirt_max),
        ] {
            let section = PlayerSection {
                name: Some("N".repeat(name_max + usize::from(name_over))),
                shirt_name: Some("S".repeat(shirt_max + usize::from(shirt_over))),
                ..PlayerSection::default()
            };
            let mut target = team.clone();
            let mut players = file.players().to_vec();
            let err = one_section(PesVersion::Pes16, slot, section)
                .apply(PesVersion::Pes16, &mut target, &mut players)
                .expect_err("one byte over");
            assert!(
                matches!(
                    err,
                    TeamTomlError::TextTooLong { ref path, max: m }
                        if *path == format!("players.{:02}.{leaf}", slot + 1) && m == max
                ),
                "{err:?}"
            );
            assert_eq!(&target, team, "nothing was written");
        }
    }

    /// A `shirt_name` of exactly the field's capacity applies; one more char
    /// is `TextTooLong` (pins `check_text`'s `len > max` at the boundary).
    #[test]
    fn a_shirt_name_at_capacity_applies_and_one_more_is_refused() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let id = team.roster[slot].player_id;
        let max = text_max(
            schema_for(PesVersion::Pes19).player.texts,
            PlayerText::ShirtName,
        );

        let section = PlayerSection {
            shirt_name: Some("a".repeat(max)),
            ..PlayerSection::default()
        };
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        one_section(PesVersion::Pes19, slot, section)
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("at capacity applies");
        let applied = players
            .iter()
            .find(|player| player.id == id)
            .expect("the player");
        assert_eq!(applied.shirt_name, "a".repeat(max));

        let section = PlayerSection {
            shirt_name: Some("a".repeat(max + 1)),
            ..PlayerSection::default()
        };
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        let err = one_section(PesVersion::Pes19, slot, section)
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect_err("one char over");
        assert!(
            matches!(
                err,
                TeamTomlError::TextTooLong { ref path, max: m }
                    if *path == format!("players.{:02}.shirt_name", slot + 1) && m == max
            ),
            "{err:?}"
        );
        assert_eq!(&target, team, "nothing was written");
        assert_eq!(players, file.players(), "nothing was written");
    }

    /// The `NotEncodable` note names the label of the style the target
    /// cannot store — not some other style's.
    #[test]
    fn an_unencodable_playing_style_notes_its_own_label() {
        // Roaming Flank is a PES 19+ style; 17/18 cannot store it.
        let (file, _) = open(PesVersion::Pes18);
        let (team, slot) = first_slot(&file);
        let id = team.roster[slot].player_id;
        let mut section = PlayerSection::default();
        section.positions.playing_style = Some(PlayStyle::RoamingFlank);
        let mut target = team.clone();
        let mut players = file.players().to_vec();
        let notes = one_section(PesVersion::Pes19, slot, section)
            .apply(PesVersion::Pes18, &mut target, &mut players)
            .expect("applies");
        assert_eq!(
            notes,
            vec![ImportNote::NotEncodable {
                path: format!("players.{:02}.positions.playing_style", slot + 1),
                label: "roaming_flank".to_string(),
            }]
        );
        let stored = players
            .iter()
            .find(|player| player.id == id)
            .expect("the player")
            .positions
            .playing_style;
        assert_eq!(
            stored,
            playstyle::encode(PesVersion::Pes18, PlayStyle::None).expect("every list holds none"),
            "the stored style falls back to none"
        );
    }

    /// `from_team` carries every rostered slot's shirt number into
    /// `players[NN].number`.
    #[test]
    fn from_team_carries_every_rostered_slots_shirt_number() {
        for version in FIXTURES {
            let (file, _) = open(version);
            let refs: Vec<&PlayerEntry> = file.players().iter().collect();
            for team in file
                .teams()
                .iter()
                .filter(|team| team.roster.iter().any(|slot| slot.player_id != 0))
            {
                let doc = TeamToml::from_team(version, team, &refs).expect("from_team");
                for (i, slot) in team.roster.iter().enumerate() {
                    if slot.player_id == 0 {
                        continue;
                    }
                    let key = u8::try_from(i + 1).expect("a slot index");
                    assert_eq!(
                        doc.players.get(&key).and_then(|section| section.number),
                        Some(slot.number),
                        "{version:?} team {} slot {key}",
                        team.id
                    );
                }
            }
        }
    }

    /// A key the document's version lacks emits the commented neutral line,
    /// comment included.
    #[test]
    fn a_key_the_version_lacks_emits_its_commented_neutral_line() {
        // star is PES 19+: on a PES 18 document it is `# star = 0`.
        let (file, _) = open(PesVersion::Pes18);
        let (team, _) = first_slot(&file);
        let refs: Vec<&PlayerEntry> = file.players().iter().collect();
        let doc = TeamToml::from_team(PesVersion::Pes18, team, &refs).expect("from_team");
        let text = doc.to_toml().expect("to_toml");
        let line = text
            .lines()
            .find(|line| line.contains("star ="))
            .expect("a star line, commented or not");
        assert!(line.starts_with("# star = 0"), "{line}");
        assert!(line.contains("# PES 19+, 0-7 stored"), "{line}");
    }

    /// A non-ASCII `ingame_face` of the right byte length is `WrongType`,
    /// not a mid-character slice panic.
    #[test]
    fn a_non_ascii_ingame_face_is_wrong_type_not_a_panic() {
        // A 3-byte '€' plus 89 ASCII bytes: 92 bytes, the length of a
        // 46-byte hex string, with a character straddling a 2-byte boundary.
        let text = format!("[players.01]\ningame_face = \"€{}\"\n", "a".repeat(89));
        let err = TeamToml::parse(&text).expect_err("not hex");
        assert!(matches!(err, TeamTomlError::WrongType { .. }), "{err:?}");
    }

    /// Stored-ranges mode is bounded by the field's widest bit width: neck
    /// length's 4 bits refuse 20 (stored 27) with the width's range.
    #[test]
    fn a_stored_value_past_the_fields_widest_width_is_out_of_range() {
        use crate::settings_toml::SettingsError;

        let err = TeamToml::parse("[players.01.appearance.physique]\nneck_length = 20\n")
            .expect_err("out of range");
        assert!(
            matches!(
                err,
                TeamTomlError::Settings(SettingsError::OutOfRange { ref range, .. })
                    if range == "0 to 15"
            ),
            "{err:?}"
        );
    }

    /// A stored value a `Kind::Labels` list does not name emits as the
    /// integer and reparses to the same document.
    #[test]
    fn a_width_valid_stored_label_without_a_label_round_trips_as_an_integer() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, _) = first_slot(&file);
        // sleeves: labels 0..=2, a 2-bit stored field — 3 fits the width.
        let mut players: Vec<PlayerEntry> = file.players().to_vec();
        let id = team
            .roster
            .iter()
            .find(|slot| slot.player_id != 0)
            .expect("a rostered slot")
            .player_id;
        players
            .iter_mut()
            .find(|player| player.id == id)
            .expect("the player")
            .appearance
            .sleeves = 3;
        let refs: Vec<&PlayerEntry> = players.iter().collect();
        let doc = TeamToml::from_team(PesVersion::Pes19, team, &refs).expect("from_team");
        let text = doc.to_toml().expect("to_toml");
        assert!(
            text.lines().any(|line| line.starts_with("sleeves = 3")),
            "the stored integer emits, not a panic:\n{}",
            text.lines()
                .find(|line| line.contains("sleeves"))
                .unwrap_or("(no sleeves line)")
        );
        let parsed = TeamToml::parse(&text).expect("reparses");
        assert_eq!(parsed, doc);
    }

    /// A shirt number wider than the target's roster field is refused at
    /// apply (`write_roster` would fail after `apply` returned Ok), never
    /// clamped. PES 18's field is 8 bits; PES 19's is 16.
    #[test]
    fn a_number_past_the_target_fields_width_is_refused() {
        let doc =
            TeamToml::parse("pes_version = 19\n\n[players.01]\nnumber = 999\n").expect("parses");

        let (file18, _) = open(PesVersion::Pes18);
        let team18 = file18
            .teams()
            .iter()
            .find(|team| team.roster.first().is_some_and(|s| s.player_id != 0))
            .expect("a team with slot 01 rostered");
        let mut players18 = file18.players().to_vec();
        let mut target = team18.clone();
        let err = doc
            .apply(PesVersion::Pes18, &mut target, &mut players18)
            .expect_err("999 does not fit an 8-bit number field");
        assert!(
            matches!(
                err,
                TeamTomlError::OutOfRange {
                    ref key,
                    value: 999,
                    ref range,
                } if key == "players.01.number" && range == "0 to 255"
            ),
            "{err:?}"
        );
        assert_eq!(&target, team18, "nothing was written");
        let edge =
            TeamToml::parse("pes_version = 19\n\n[players.01]\nnumber = 255\n").expect("parses");
        edge.apply(PesVersion::Pes18, &mut target, &mut players18)
            .expect("255 is the 8-bit field's last value");
        assert_eq!(target.roster[0].number, 255);

        let (file19, _) = open(PesVersion::Pes19);
        let team19 = file19
            .teams()
            .iter()
            .find(|team| team.roster.first().is_some_and(|s| s.player_id != 0))
            .expect("a team with slot 01 rostered");
        let mut players19 = file19.players().to_vec();
        let mut target = team19.clone();
        doc.apply(PesVersion::Pes19, &mut target, &mut players19)
            .expect("PES 19's 16-bit field holds 999");
        assert_eq!(target.roster[0].number, 999);
    }

    /// Hostile integers reach the stored-width check through checked math:
    /// the `value + 7` / `value - 1` shifts of Signed7 and OneBased keys do
    /// not overflow-panic.
    #[test]
    fn extreme_integers_are_out_of_range_not_a_panic() {
        use crate::settings_toml::SettingsError;

        for (table, key, value) in [
            ("physique", "neck_length", "9223372036854775807"),
            ("motion", "hunching_dribbling", "-9223372036854775808"),
        ] {
            let doc = format!("[players.01.appearance.{table}]\n{key} = {value}\n");
            let err = TeamToml::parse(&doc).expect_err("out of range");
            assert!(
                matches!(
                    err,
                    TeamTomlError::Settings(SettingsError::OutOfRange { .. })
                ),
                "{key} = {value}: {err:?}"
            );
        }
    }

    /// The own-id convention compares with checked math: a team id near
    /// `u32::MAX` cannot be `id * 100 + slot`, so it compares false and the
    /// document's value lands, rather than panicking in debug builds.
    #[test]
    fn a_maximal_team_id_does_not_overflow_the_own_id_check() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let mut players = file.players().to_vec();
        let mut target = team.clone();
        let doc = TeamToml::parse(&format!(
            "pes_version = 19\n\n[team]\nid = 4294967295\n\n[players.{:02}]\nbase_copy_id = 7\n",
            slot + 1
        ))
        .expect("parses");
        doc.apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("applies");
        let id = team.roster[slot].player_id;
        let player = players.iter().find(|p| p.id == id).expect("rostered");
        assert_eq!(player.appearance.base_copy_id, 7);
    }

    /// A stored value the target version's field cannot hold is refused at
    /// apply — `write_player` would reject it afterwards, breaking the
    /// all-or-nothing contract. `free_kick` is stored 4 bits on 15-19 and
    /// 5 bits on 20/21, so `free_kick = 17` (stored 16) is a real width
    /// difference between versions.
    #[test]
    fn a_value_past_the_target_fields_width_is_refused() {
        let (file19, _) = open(PesVersion::Pes19);
        let (team19, slot) = first_slot(&file19);
        let doc = TeamToml::parse(&format!(
            "pes_version = 19\n\n[players.{:02}.appearance.motion]\nfree_kick = 17\n",
            slot + 1
        ))
        .expect("parses");

        let mut players19 = file19.players().to_vec();
        let mut target19 = team19.clone();
        let err = doc
            .apply(PesVersion::Pes19, &mut target19, &mut players19)
            .expect_err("16 does not fit a 4-bit field");
        assert!(
            matches!(
                err,
                TeamTomlError::OutOfRange {
                    ref key,
                    value: 16,
                    ref range,
                } if key.ends_with(".appearance.motion.free_kick") && range == "0 to 15"
            ),
            "{err:?}"
        );
        assert_eq!(&target19, team19, "the team changed");
        assert_eq!(players19, file19.players(), "a player changed");

        let (file21, _) = open(PesVersion::Pes21);
        let (team21, slot21) = first_slot(&file21);
        let doc21 = TeamToml::parse(&format!(
            "pes_version = 21\n\n[players.{:02}.appearance.motion]\nfree_kick = 17\n",
            slot21 + 1
        ))
        .expect("parses");
        let mut players21 = file21.players().to_vec();
        let mut target21 = team21.clone();
        doc21
            .apply(PesVersion::Pes21, &mut target21, &mut players21)
            .expect("PES 21's 5-bit field holds it");
    }

    /// The boundary, not just a miss: `boots_id`'s 14-bit field takes 16383
    /// and refuses 16384.
    #[test]
    fn a_value_at_the_width_boundary_applies_and_one_over_is_refused() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let doc = |value| {
            TeamToml::parse(&format!(
                "pes_version = 19\n\n[players.{:02}]\nboots_id = {value}\n",
                slot + 1
            ))
            .expect("parses")
        };

        let mut players = file.players().to_vec();
        let mut target = team.clone();
        doc(16383)
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("16383 fits 14 bits");

        let mut players = file.players().to_vec();
        let mut target = team.clone();
        let err = doc(16384)
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect_err("16384 does not fit 14 bits");
        assert!(
            matches!(
                err,
                TeamTomlError::OutOfRange {
                    ref key,
                    value: 16384,
                    ref range,
                } if key.ends_with(".boots_id") && range == "0 to 16383"
            ),
            "{err:?}"
        );
        assert_eq!(&target, team);
        assert_eq!(players, file.players());
    }

    /// The same check on a caller-built document (no parse): an appearance
    /// value wider than its field — `sleeves` is 2 bits — is refused.
    #[test]
    fn a_caller_built_appearance_value_past_the_field_width_is_refused() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let mut section = PlayerSection::default();
        section.appearance.strip.sleeves = Some(5);
        let mut doc = TeamToml::default();
        doc.players
            .insert(u8::try_from(slot + 1).expect("a slot"), section);

        let mut players = file.players().to_vec();
        let mut target = team.clone();
        let err = doc
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect_err("5 does not fit a 2-bit field");
        assert!(
            matches!(
                err,
                TeamTomlError::OutOfRange {
                    ref key,
                    value: 5,
                    ref range,
                } if key.ends_with(".appearance.strip.sleeves") && range == "0 to 3"
            ),
            "{err:?}"
        );
        assert_eq!(&target, team);
        assert_eq!(players, file.players());
    }

    /// A char above U+00FF cannot encode into the single-byte fields:
    /// `shirt_name = "Ā"` is refused at apply with nothing written, and a
    /// caller-built document with a NUL in `name` is refused the same way.
    #[test]
    fn unencodable_text_is_refused_at_apply() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let key = u8::try_from(slot + 1).expect("a slot");

        let doc = TeamToml::parse(&format!(
            "pes_version = 19\n\n[players.{key:02}]\nshirt_name = \"Ā\"\n"
        ))
        .expect("parses");
        let mut players = file.players().to_vec();
        let mut target = team.clone();
        let err = doc
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect_err("Ā is above U+00FF");
        assert!(
            matches!(err, TeamTomlError::WrongType { ref key, .. } if key.ends_with(".shirt_name")),
            "{err:?}"
        );
        assert_eq!(&target, team);
        assert_eq!(players, file.players());

        let section = PlayerSection {
            name: Some("A\0B".to_string()),
            ..PlayerSection::default()
        };
        let mut doc = TeamToml::default();
        doc.players.insert(key, section);
        let mut players = file.players().to_vec();
        let mut target = team.clone();
        let err = doc
            .apply(PesVersion::Pes19, &mut target, &mut players)
            .expect_err("a NUL is refused");
        assert!(
            matches!(err, TeamTomlError::WrongType { ref key, .. } if key.ends_with(".name")),
            "{err:?}"
        );
        assert_eq!(&target, team);
        assert_eq!(players, file.players());
    }

    /// `é` and `ÿ` (U+00FF, the last single-byte char) are one byte each in
    /// the single-byte encoding: they apply and survive a
    /// `write_player`/`read_player` round trip on the PES 19 fixture.
    #[test]
    fn a_latin1_shirt_name_round_trips_through_the_codec() {
        let (file, _) = open(PesVersion::Pes19);
        let (team, slot) = first_slot(&file);
        let doc = TeamToml::parse(&format!(
            "pes_version = 19\n\n[players.{:02}]\nshirt_name = \"éÿ\"\n",
            slot + 1
        ))
        .expect("parses");
        let mut players = file.players().to_vec();
        let mut target = team.clone();
        doc.apply(PesVersion::Pes19, &mut target, &mut players)
            .expect("applies");

        let id = team.roster[slot].player_id;
        let player = players.iter().find(|p| p.id == id).expect("rostered");
        assert_eq!(player.shirt_name, "éÿ");
        let schema = schema_for(PesVersion::Pes19);
        let mut record = vec![0u8; schema.player.size];
        crate::codec::write_player(player, &mut record, schema.player).expect("write_player");
        let back = crate::codec::read_player(&record, schema.player).expect("read_player");
        assert_eq!(back.shirt_name, "éÿ");
    }
}
