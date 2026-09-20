//! `team.toml`'s team and tactics halves: `from_team` dumps a `TeamEntry`,
//! `to_toml` emits the plan's block (one key per line, comments at column 33,
//! an unset key as its commented neutral line, a table only when its section
//! carries data), `parse` accepts partial documents (absent = untouched on
//! import) and `apply` writes the present keys back, gated by the target
//! schema's field tables rather than by matching on `PesVersion`.

use pes_version::PesVersion;
use toml_edit::{DocumentMut, Item, TableLike, Value};

use crate::interchange::team_toml::{
    AutoSection, ImportNote, InstructionEntry, InstructionsSection, PresetSection,
    SetPiecesSection, SlidersSection, StyleSection, TacticsSection, TeamEditFlagsSection,
    TeamSection, TeamToml, TeamTomlError,
};
use crate::model::instruction::Instruction;
use crate::model::names::display_name;
use crate::model::player::PlayerEntry;
use crate::model::tactics::{AdvancedInstruction, FormationSlot};
use crate::model::team::{KitSlot, TeamColor, TeamEntry};
use crate::schema::fields::{PlayerText, PresetField, TacticsField, TeamField};
use crate::schema::instruction;
use crate::schema::{TacticsSchema, schema_for};

use super::{labels, player};

/// `body` padded so `#` starts at character 33 (1-based); a body longer than
/// the pad still gets one space before `#`.
pub(super) fn padded(body: &str, comment: &str) -> String {
    if comment.is_empty() {
        return body.to_string();
    }
    let pad = 32_usize.saturating_sub(body.len()).max(1);
    format!("{body}{}# {comment}", " ".repeat(pad))
}

/// The TOML text of a `Some` value, or `neutral` for a `None` key.
pub(super) fn value_or(value: Option<String>, neutral: &str) -> String {
    value.unwrap_or_else(|| neutral.to_string())
}

impl TeamToml {
    /// Reads a `team.toml` text. Every key is optional; anything the format
    /// does not know is `UnknownKey`.
    pub fn parse(text: &str) -> Result<Self, TeamTomlError> {
        let document: DocumentMut = text
            .parse()
            .map_err(|e: toml_edit::TomlError| TeamTomlError::Toml(e.to_string()))?;
        let mut parsed = TeamToml::default();
        for (name, item) in document.iter() {
            match name {
                "pes_version" => parsed.pes_version = Some(version(item)?),
                "team" => parsed.team = parse_team(item)?,
                "tactics" => parsed.tactics = parse_tactics(item)?,
                "players" => parsed.players = player::parse_players(item)?,
                _ => {
                    return Err(TeamTomlError::UnknownKey {
                        key: name.to_string(),
                    });
                }
            }
        }
        Ok(parsed)
    }

    /// The document for one team of `version`'s save. Every field the model
    /// carries is `Some`; version-gated model `Option`s stay `None` and emit
    /// commented lines.
    pub fn from_team(
        version: PesVersion,
        team: &TeamEntry,
        players: &[&PlayerEntry],
    ) -> Result<Self, TeamTomlError> {
        let player_sections = player::players_from(version, team, players)?;
        let edit = &team.edit_flags;
        let tactics = &team.tactics;
        let mut presets = [PresetSection::default(); 3];
        for (section, preset) in presets.iter_mut().zip(tactics.presets.iter()) {
            let style = &preset.style;
            let sliders = &preset.sliders;
            section.style = StyleSection {
                attacking_style: Some(style.attacking_style),
                attacking_zone: Some(style.attacking_zone),
                buildup: Some(style.buildup),
                positioning: Some(style.positioning),
                defensive_style: Some(style.defensive_style),
                containment_area: Some(style.containment_area),
                pressure: Some(style.pressure),
                fluid: style.fluid,
            };
            section.sliders = SlidersSection {
                support_range: Some(sliders.support_range),
                defensive_line: Some(sliders.defensive_line),
                compactness: Some(sliders.compactness),
                numbers_in_attack: Some(sliders.numbers_in_attack),
                numbers_in_defence: Some(sliders.numbers_in_defence),
            };
            section.formations = [
                Some(preset.formations[0].players),
                Some(preset.formations[1].players),
                Some(preset.formations[2].players),
            ];
            section.instructions = match (&preset.attack_instructions, &preset.defence_instructions)
            {
                (Some(attack), Some(defence)) => Some(InstructionsSection {
                    attack: entries(version, attack)?,
                    defence: entries(version, defence)?,
                }),
                // The codec sets both sides together; a lone side is
                // unreachable from a decoded save.
                (None, None) | (Some(_), None) | (None, Some(_)) => None,
            };
        }
        Ok(TeamToml {
            pes_version: Some(version),
            team: TeamSection {
                id: Some(team.id),
                name: Some(team.name.clone()),
                short_name: Some(team.short_name.clone()),
                manager_id: team.manager_id,
                stadium_id: team.stadium_id,
                color_1: team.colors.map(|c| c[0]),
                color_2: team.colors.map(|c| c[1]),
                kit_slots: team.kit_slots,
                edit_flags: TeamEditFlagsSection {
                    name: Some(edit.name),
                    short_name: edit.short_name,
                    stadium: edit.stadium,
                    strip: edit.strip,
                },
            },
            tactics: TacticsSection {
                starting_eleven: Some(tactics.starting_eleven),
                bench_order: Some(tactics.bench_order),
                set_pieces: SetPiecesSection {
                    free_kick_long: Some(tactics.set_pieces.free_kick_long),
                    free_kick_short: Some(tactics.set_pieces.free_kick_short),
                    free_kick_second: Some(tactics.set_pieces.free_kick_second),
                    corner_left: Some(tactics.set_pieces.corner_left),
                    corner_right: Some(tactics.set_pieces.corner_right),
                    penalty: Some(tactics.set_pieces.penalty),
                    captain: Some(tactics.set_pieces.captain),
                },
                players_to_join_attack: Some(tactics.players_to_join_attack),
                auto: AutoSection {
                    substitution: tactics.auto.substitution,
                    offside_trap: tactics.auto.offside_trap,
                    preset_change: tactics.auto.preset_change,
                    attack_defence_levels: tactics.auto.attack_defence_levels,
                },
                presets,
            },
            players: player_sections,
        })
    }

    /// The document's text, in the plan block's order. A `None` key emits its
    /// commented neutral line; a section with no `Some` member emits no table.
    pub fn to_toml(&self) -> Result<String, TeamTomlError> {
        let mut out = String::new();
        let version = self
            .pes_version
            .map(|v| u16::from(v).to_string())
            .unwrap_or_else(|| "19".to_string());
        let body = format!("pes_version = {version}");
        let line = padded(
            &body,
            "the save this was written from; import into another version converts",
        );
        match self.pes_version {
            Some(_) => out.push_str(&line),
            None => out.push_str(&format!("# {line}")),
        }
        out.push('\n');
        emit_team(&mut out, &self.team);
        emit_tactics(&mut out, &self.tactics)?;
        player::emit_players(&mut out, &self.players)?;
        Ok(out)
    }

    /// Writes the document's present keys into `team`, all-or-nothing: a
    /// cloned team is patched and swapped in only when every key applied (or
    /// was noted and skipped). `team.id` and the roster are never written —
    /// `id` is informational and Team TOML does not carry roster data.
    ///
    /// A key the target's schema lacks is a `NotInThisVersion` note when the
    /// document's `pes_version` differs from `to`, and a
    /// `TeamTomlError::NotInThisVersion` when it matches or is absent. The
    /// instructions table is dropped silently on versions with no instruction
    /// block; an instruction the target cannot store becomes `Off` with a
    /// `NotEncodable` note.
    pub fn apply(
        &self,
        to: PesVersion,
        team: &mut TeamEntry,
        players: &mut [PlayerEntry],
    ) -> Result<Vec<ImportNote>, TeamTomlError> {
        let mut next_players = players.to_vec();
        let from = self.pes_version.unwrap_or(to);
        let foreign = from != to;
        let schema = schema_for(to);
        let mut notes = Vec::new();
        let mut next = team.clone();
        apply_team(&self.team, schema.team, foreign, &mut next, &mut notes)?;
        apply_tactics(
            &self.tactics,
            schema.tactic,
            to,
            foreign,
            &mut next,
            &mut notes,
        )?;
        player::apply_players(
            &self.players,
            self.team.id,
            to,
            foreign,
            &mut next,
            &mut next_players,
            &mut notes,
        )?;
        *team = next;
        players.clone_from_slice(&next_players);
        Ok(notes)
    }
}

/// The shirt name a display name produces: colour codes stripped, uppercased,
/// cut to the version's shirt-name field minus its terminator.
pub fn shirt_name_from(name: &str, version: PesVersion) -> String {
    let len = schema_for(version)
        .player
        .texts
        .iter()
        .find(|spec| spec.text == PlayerText::ShirtName)
        .map(|spec| spec.len)
        .expect("every player schema stores a shirt name");
    let take = usize::try_from(len - 1).expect("the field length minus its terminator fits usize");
    display_name(name)
        .to_uppercase()
        .chars()
        .take(take)
        .collect()
}

/// One instruction side's two entries, decoded through the version's table.
fn entries(
    version: PesVersion,
    stored: &[AdvancedInstruction; 2],
) -> Result<[InstructionEntry; 2], TeamTomlError> {
    let mut out = [InstructionEntry::default(); 2];
    for (entry, stored) in out.iter_mut().zip(stored.iter()) {
        *entry = InstructionEntry {
            instruction: match instruction::decode(version, stored.instruction)? {
                Some(instruction) => instruction,
                None => Instruction::Off,
            },
            player: stored.player,
        };
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// Emit

/// One line: `name = value` with the comment at column 33, commented out with
/// the neutral value when the key is `None`.
pub(super) fn emit(
    out: &mut String,
    name: &str,
    value: Option<String>,
    neutral: &str,
    comment: &str,
) {
    let set = value.is_some();
    let body = format!("{name} = {}", value_or(value, neutral));
    let line = padded(&body, comment);
    if set {
        out.push_str(&line);
    } else {
        out.push_str(&format!("# {line}"));
    }
    out.push('\n');
}

fn emit_team(out: &mut String, team: &TeamSection) {
    out.push_str("\n[team]\n");
    emit(
        out,
        "id",
        team.id.map(|v| v.to_string()),
        "0",
        "informational: the caller chooses the target team",
    );
    emit(
        out,
        "name",
        team.name
            .as_ref()
            .map(|n| Value::from(n.as_str()).to_string()),
        "\"\"",
        "",
    );
    emit(
        out,
        "short_name",
        team.short_name
            .as_ref()
            .map(|n| Value::from(n.as_str()).to_string()),
        "\"\"",
        "",
    );
    emit(
        out,
        "manager_id",
        team.manager_id.map(|v| v.to_string()),
        "0",
        "PES 19+",
    );
    emit(
        out,
        "stadium_id",
        team.stadium_id.map(|v| v.to_string()),
        "0",
        "PES 19+",
    );
    emit(
        out,
        "color_1",
        team.color_1.map(color_text),
        "[0, 0, 0]",
        "PES 17+; stored channels, 0-63 each",
    );
    emit(
        out,
        "color_2",
        team.color_2.map(color_text),
        "[0, 0, 0]",
        "PES 17+; stored channels, 0-63 each",
    );
    emit(
        out,
        "kit_slots",
        team.kit_slots.map(kit_slots_text),
        "[{ number = 0, binding = 0 }]",
        "PES 17",
    );
    let flags = &team.edit_flags;
    if flags.name.is_none()
        && flags.short_name.is_none()
        && flags.stadium.is_none()
        && flags.strip.is_none()
    {
        return;
    }
    out.push_str("\n[team.edit_flags]\n");
    emit(out, "name", flags.name.map(|v| v.to_string()), "false", "");
    emit(
        out,
        "short_name",
        flags.short_name.map(|v| v.to_string()),
        "false",
        "PES 15 only",
    );
    emit(
        out,
        "stadium",
        flags.stadium.map(|v| v.to_string()),
        "false",
        "PES 20+",
    );
    emit(
        out,
        "strip",
        flags.strip.map(|v| v.to_string()),
        "false",
        "PES 17 only",
    );
}

fn emit_tactics(out: &mut String, tactics: &TacticsSection) -> Result<(), TeamTomlError> {
    out.push_str("\n[tactics]\n");
    emit(
        out,
        "starting_eleven",
        tactics.starting_eleven.map(array_text),
        "[]",
        "",
    );
    emit(
        out,
        "bench_order",
        tactics.bench_order.map(array_text),
        "[]",
        "",
    );
    emit(
        out,
        "players_to_join_attack",
        tactics.players_to_join_attack.map(array_text),
        "[]",
        "roster slots; 255 = none",
    );
    let sp = &tactics.set_pieces;
    out.push_str("\n[tactics.set_pieces]\n");
    emit(
        out,
        "free_kick_long",
        sp.free_kick_long.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    emit(
        out,
        "free_kick_short",
        sp.free_kick_short.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    emit(
        out,
        "free_kick_second",
        sp.free_kick_second.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    emit(
        out,
        "corner_left",
        sp.corner_left.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    emit(
        out,
        "corner_right",
        sp.corner_right.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    emit(
        out,
        "penalty",
        sp.penalty.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    emit(
        out,
        "captain",
        sp.captain.map(|v| v.to_string()),
        "255",
        "roster slots; 255 = none",
    );
    let auto = &tactics.auto;
    out.push_str("\n[tactics.auto]\n");
    emit(
        out,
        "substitution",
        auto.substitution.map(|v| v.to_string()),
        "255",
        "PES 16+; the stored setting byte",
    );
    emit(
        out,
        "offside_trap",
        auto.offside_trap.map(|v| v.to_string()),
        "false",
        "PES 16+",
    );
    emit(
        out,
        "preset_change",
        auto.preset_change.map(|v| v.to_string()),
        "false",
        "PES 16+",
    );
    emit(
        out,
        "attack_defence_levels",
        auto.attack_defence_levels.map(|v| v.to_string()),
        "false",
        "PES 17+",
    );
    for (i, preset) in tactics.presets.iter().enumerate() {
        let n = i + 1;
        out.push_str(&format!("\n[tactics.preset_{n}.style]\n"));
        let style = &preset.style;
        let switches: [Option<bool>; 7] = [
            style.attacking_style,
            style.attacking_zone,
            style.buildup,
            style.positioning,
            style.defensive_style,
            style.containment_area,
            style.pressure,
        ];
        for ((key, pair), value) in labels::STYLE_SWITCHES.iter().zip(switches) {
            let text = value.map(|on| format!("\"{}\"", pair[usize::from(on)]));
            emit(
                out,
                key,
                text,
                &format!("\"{}\"", pair[0]),
                &format!("\"{}\" | \"{}\"", pair[0], pair[1]),
            );
        }
        emit(
            out,
            "fluid",
            style.fluid.map(|v| v.to_string()),
            "false",
            "PES 16+",
        );
        out.push_str(&format!("\n[tactics.preset_{n}.sliders]\n"));
        let sliders = &preset.sliders;
        emit(
            out,
            "support_range",
            sliders.support_range.map(|v| v.to_string()),
            "1",
            "1 to 10",
        );
        emit(
            out,
            "defensive_line",
            sliders.defensive_line.map(|v| v.to_string()),
            "1",
            "1 to 10",
        );
        emit(
            out,
            "compactness",
            sliders.compactness.map(|v| v.to_string()),
            "1",
            "1 to 10",
        );
        emit(
            out,
            "numbers_in_attack",
            sliders.numbers_in_attack.map(|v| v.to_string()),
            "1",
            "1 few to 3 many",
        );
        emit(
            out,
            "numbers_in_defence",
            sliders.numbers_in_defence.map(|v| v.to_string()),
            "1",
            "1 few to 3 many",
        );
        for (f, formation) in preset.formations.iter().enumerate() {
            let Some(players) = formation else { continue };
            out.push_str(&format!("\n[tactics.preset_{n}.formation_{}]\n", f + 1));
            out.push_str("players = [\n");
            for slot in players.iter() {
                let entry = slot_text(
                    slot,
                    &format!("tactics.preset_{n}.formation_{}.players", f + 1),
                )?;
                out.push_str(&format!("    {entry},\n"));
            }
            out.push_str("]\n");
        }
        if let Some(instructions) = &preset.instructions {
            out.push_str(&format!("\n[tactics.preset_{n}.instructions]\n"));
            let attack: Vec<String> = instructions.attack.iter().map(instruction_text).collect();
            let defence: Vec<String> = instructions.defence.iter().map(instruction_text).collect();
            out.push_str(&format!("attack = [{}]\n", attack.join(", ")));
            out.push_str(&format!("defence = [{}]\n", defence.join(", ")));
        }
    }
    Ok(())
}

fn color_text(c: TeamColor) -> String {
    format!("[{}, {}, {}]", c.red, c.green, c.blue)
}

fn kit_slots_text(slots: [KitSlot; 10]) -> String {
    let entries: Vec<String> = slots
        .iter()
        .map(|s| format!("{{ number = {}, binding = {} }}", s.number, s.binding))
        .collect();
    format!("[{}]", entries.join(", "))
}

fn array_text<const N: usize>(a: [u8; N]) -> String {
    let items: Vec<String> = a.iter().map(|v| v.to_string()).collect();
    format!("[{}]", items.join(", "))
}

fn slot_text(slot: &FormationSlot, key: &str) -> Result<String, TeamTomlError> {
    let position = labels::POSITIONS
        .get(usize::from(slot.position))
        .ok_or_else(|| TeamTomlError::OutOfRange {
            key: key.to_string(),
            value: i64::from(slot.position),
            range: "0 to 12".to_string(),
        })?;
    Ok(format!(
        "{{ position = \"{position}\", x = {}, y = {} }}",
        slot.x, slot.y
    ))
}

fn instruction_text(entry: &InstructionEntry) -> String {
    let index = Instruction::ALL
        .iter()
        .position(|i| *i == entry.instruction)
        .expect("a canonical instruction is in ALL");
    format!(
        "{{ instruction = \"{}\", player = {} }}",
        labels::INSTRUCTIONS[index],
        entry.player
    )
}

// ---------------------------------------------------------------------------
// Parse

fn version(item: &Item) -> Result<PesVersion, TeamTomlError> {
    let value = integer(item, "pes_version")?;
    let number = u16::try_from(value).map_err(|_| TeamTomlError::OutOfRange {
        key: "pes_version".to_string(),
        value,
        range: "15 to 21".to_string(),
    })?;
    PesVersion::from_number(number).ok_or_else(|| TeamTomlError::OutOfRange {
        key: "pes_version".to_string(),
        value,
        range: "15 to 21".to_string(),
    })
}

fn parse_team(item: &Item) -> Result<TeamSection, TeamTomlError> {
    let table = as_table(item, "team")?;
    let mut team = TeamSection {
        id: opt(table, "team", "id", u32_val)?,
        name: opt(table, "team", "name", text)?,
        short_name: opt(table, "team", "short_name", text)?,
        manager_id: opt(table, "team", "manager_id", u32_val)?,
        stadium_id: opt(table, "team", "stadium_id", u16_val)?,
        color_1: opt(table, "team", "color_1", color)?,
        color_2: opt(table, "team", "color_2", color)?,
        kit_slots: opt(table, "team", "kit_slots", kit_slots)?,
        edit_flags: TeamEditFlagsSection::default(),
    };
    if let Some(item) = table.get("edit_flags") {
        let flags = as_table(item, "team.edit_flags")?;
        team.edit_flags = TeamEditFlagsSection {
            name: opt(flags, "team.edit_flags", "name", boolean)?,
            short_name: opt(flags, "team.edit_flags", "short_name", boolean)?,
            stadium: opt(flags, "team.edit_flags", "stadium", boolean)?,
            strip: opt(flags, "team.edit_flags", "strip", boolean)?,
        };
        reject(
            flags,
            "team.edit_flags",
            &["name", "short_name", "stadium", "strip"],
        )?;
    }
    reject(
        table,
        "team",
        &[
            "id",
            "name",
            "short_name",
            "manager_id",
            "stadium_id",
            "color_1",
            "color_2",
            "kit_slots",
            "edit_flags",
        ],
    )?;
    Ok(team)
}

fn parse_tactics(item: &Item) -> Result<TacticsSection, TeamTomlError> {
    let table = as_table(item, "tactics")?;
    let mut tactics = TacticsSection {
        starting_eleven: opt(table, "tactics", "starting_eleven", int_array)?,
        bench_order: opt(table, "tactics", "bench_order", int_array)?,
        players_to_join_attack: opt(table, "tactics", "players_to_join_attack", int_array)?,
        ..TacticsSection::default()
    };
    if let Some(item) = table.get("set_pieces") {
        let t = as_table(item, "tactics.set_pieces")?;
        tactics.set_pieces = SetPiecesSection {
            free_kick_long: opt(t, "tactics.set_pieces", "free_kick_long", slot)?,
            free_kick_short: opt(t, "tactics.set_pieces", "free_kick_short", slot)?,
            free_kick_second: opt(t, "tactics.set_pieces", "free_kick_second", slot)?,
            corner_left: opt(t, "tactics.set_pieces", "corner_left", slot)?,
            corner_right: opt(t, "tactics.set_pieces", "corner_right", slot)?,
            penalty: opt(t, "tactics.set_pieces", "penalty", slot)?,
            captain: opt(t, "tactics.set_pieces", "captain", slot)?,
        };
        reject(
            t,
            "tactics.set_pieces",
            &[
                "free_kick_long",
                "free_kick_short",
                "free_kick_second",
                "corner_left",
                "corner_right",
                "penalty",
                "captain",
            ],
        )?;
    }
    if let Some(item) = table.get("auto") {
        let t = as_table(item, "tactics.auto")?;
        tactics.auto = AutoSection {
            substitution: opt(t, "tactics.auto", "substitution", u8_val)?,
            offside_trap: opt(t, "tactics.auto", "offside_trap", boolean)?,
            preset_change: opt(t, "tactics.auto", "preset_change", boolean)?,
            attack_defence_levels: opt(t, "tactics.auto", "attack_defence_levels", boolean)?,
        };
        reject(
            t,
            "tactics.auto",
            &[
                "substitution",
                "offside_trap",
                "preset_change",
                "attack_defence_levels",
            ],
        )?;
    }
    for (i, preset) in tactics.presets.iter_mut().enumerate() {
        let key = format!("preset_{}", i + 1);
        let Some(item) = table.get(&key) else {
            continue;
        };
        *preset = parse_preset(item, &format!("tactics.preset_{}", i + 1))?;
    }
    reject(
        table,
        "tactics",
        &[
            "starting_eleven",
            "bench_order",
            "players_to_join_attack",
            "set_pieces",
            "auto",
            "preset_1",
            "preset_2",
            "preset_3",
        ],
    )?;
    Ok(tactics)
}

fn parse_preset(item: &Item, path: &str) -> Result<PresetSection, TeamTomlError> {
    let table = as_table(item, path)?;
    let mut preset = PresetSection::default();
    if let Some(item) = table.get("style") {
        let t = as_table(item, &format!("{path}.style"))?;
        let style_path = format!("{path}.style");
        let mut switches: [Option<bool>; 7] = [None; 7];
        for ((key, pair), slot) in labels::STYLE_SWITCHES.iter().zip(switches.iter_mut()) {
            *slot = opt(t, &style_path, key, |i, k| label(i, k, pair))?;
        }
        preset.style = StyleSection {
            attacking_style: switches[0],
            attacking_zone: switches[1],
            buildup: switches[2],
            positioning: switches[3],
            defensive_style: switches[4],
            containment_area: switches[5],
            pressure: switches[6],
            fluid: opt(t, &style_path, "fluid", boolean)?,
        };
        reject(
            t,
            &style_path,
            &[
                "attacking_style",
                "attacking_zone",
                "buildup",
                "positioning",
                "defensive_style",
                "containment_area",
                "pressure",
                "fluid",
            ],
        )?;
    }
    if let Some(item) = table.get("sliders") {
        let t = as_table(item, &format!("{path}.sliders"))?;
        let sliders_path = format!("{path}.sliders");
        preset.sliders = SlidersSection {
            support_range: opt(t, &sliders_path, "support_range", |i, k| {
                ranged(i, k, 1, 10)
            })?,
            defensive_line: opt(t, &sliders_path, "defensive_line", |i, k| {
                ranged(i, k, 1, 10)
            })?,
            compactness: opt(t, &sliders_path, "compactness", |i, k| ranged(i, k, 1, 10))?,
            numbers_in_attack: opt(t, &sliders_path, "numbers_in_attack", |i, k| {
                ranged(i, k, 1, 3)
            })?,
            numbers_in_defence: opt(t, &sliders_path, "numbers_in_defence", |i, k| {
                ranged(i, k, 1, 3)
            })?,
        };
        reject(
            t,
            &sliders_path,
            &[
                "support_range",
                "defensive_line",
                "compactness",
                "numbers_in_attack",
                "numbers_in_defence",
            ],
        )?;
    }
    for (f, formation) in preset.formations.iter_mut().enumerate() {
        let key = format!("formation_{}", f + 1);
        let Some(item) = table.get(&key) else {
            continue;
        };
        let t = as_table(item, &format!("{path}.{key}"))?;
        *formation = Some(required(
            t,
            &format!("{path}.{key}"),
            "players",
            formation_players,
        )?);
        reject(t, &format!("{path}.{key}"), &["players"])?;
    }
    if let Some(item) = table.get("instructions") {
        let t = as_table(item, &format!("{path}.instructions"))?;
        let instructions_path = format!("{path}.instructions");
        preset.instructions = Some(InstructionsSection {
            attack: required(t, &instructions_path, "attack", instruction_entries)?,
            defence: required(t, &instructions_path, "defence", instruction_entries)?,
        });
        reject(t, &instructions_path, &["attack", "defence"])?;
    }
    reject(
        table,
        path,
        &[
            "style",
            "sliders",
            "formation_1",
            "formation_2",
            "formation_3",
            "instructions",
        ],
    )?;
    Ok(preset)
}

// ---------------------------------------------------------------------------
// Value helpers

pub(super) fn as_table<'a>(item: &'a Item, key: &str) -> Result<&'a dyn TableLike, TeamTomlError> {
    item.as_table_like()
        .ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "a table",
        })
}

pub(super) fn integer(item: &Item, key: &str) -> Result<i64, TeamTomlError> {
    item.as_value()
        .and_then(|v| v.as_integer())
        .ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "an integer",
        })
}

pub(super) fn ranged(item: &Item, key: &str, min: i64, max: i64) -> Result<u8, TeamTomlError> {
    let value = integer(item, key)?;
    if !(min..=max).contains(&value) {
        return Err(TeamTomlError::OutOfRange {
            key: key.to_string(),
            value,
            range: format!("{min} to {max}"),
        });
    }
    u8::try_from(value).map_err(|_| TeamTomlError::OutOfRange {
        key: key.to_string(),
        value,
        range: format!("{min} to {max}"),
    })
}

pub(super) fn u8_val(item: &Item, key: &str) -> Result<u8, TeamTomlError> {
    ranged(item, key, 0, 255)
}

pub(super) fn slot(item: &Item, key: &str) -> Result<u8, TeamTomlError> {
    ranged(item, key, 0, 255)
}

pub(super) fn u16_val(item: &Item, key: &str) -> Result<u16, TeamTomlError> {
    let value = integer(item, key)?;
    u16::try_from(value).map_err(|_| TeamTomlError::OutOfRange {
        key: key.to_string(),
        value,
        range: "0 to 65535".to_string(),
    })
}

pub(super) fn u32_val(item: &Item, key: &str) -> Result<u32, TeamTomlError> {
    let value = integer(item, key)?;
    u32::try_from(value).map_err(|_| TeamTomlError::OutOfRange {
        key: key.to_string(),
        value,
        range: "0 to 4294967295".to_string(),
    })
}

pub(super) fn boolean(item: &Item, key: &str) -> Result<bool, TeamTomlError> {
    item.as_value()
        .and_then(|v| v.as_bool())
        .ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "true or false",
        })
}

pub(super) fn text(item: &Item, key: &str) -> Result<String, TeamTomlError> {
    item.as_value()
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "a string",
        })
}

pub(super) fn label(item: &Item, key: &str, labels: &[&str]) -> Result<bool, TeamTomlError> {
    let text =
        item.as_value()
            .and_then(|v| v.as_str())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "a string",
            })?;
    labels
        .iter()
        .position(|l| *l == text)
        .map(|i| i == 1)
        .ok_or_else(|| TeamTomlError::UnknownLabel {
            key: key.to_string(),
            label: text.to_string(),
            allowed: labels
                .iter()
                .map(|l| format!("\"{l}\""))
                .collect::<Vec<_>>()
                .join(", "),
        })
}

pub(super) fn int_array<const N: usize>(item: &Item, key: &str) -> Result<[u8; N], TeamTomlError> {
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of integers",
            })?;
    let mut out = [0u8; N];
    if array.len() != N {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "an array of integers",
        });
    }
    for (slot, value) in out.iter_mut().zip(array.iter()) {
        *slot = ranged(&Item::Value(value.clone()), key, 0, 255)?;
    }
    Ok(out)
}

pub(super) fn color(item: &Item, key: &str) -> Result<TeamColor, TeamTomlError> {
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of three 0 to 63 integers",
            })?;
    if array.len() != 3 {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "an array of three 0 to 63 integers",
        });
    }
    let mut channels = [0u8; 3];
    for (channel, value) in channels.iter_mut().zip(array.iter()) {
        *channel = ranged(&Item::Value(value.clone()), key, 0, 63)?;
    }
    Ok(TeamColor {
        red: channels[0],
        green: channels[1],
        blue: channels[2],
    })
}

pub(super) fn kit_slots(item: &Item, key: &str) -> Result<[KitSlot; 10], TeamTomlError> {
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of ten { number, binding } tables",
            })?;
    if array.len() != 10 {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected: "an array of ten { number, binding } tables",
        });
    }
    let mut slots = [KitSlot::default(); 10];
    for (slot, value) in slots.iter_mut().zip(array.iter()) {
        let entry = value
            .as_inline_table()
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of ten { number, binding } tables",
            })?;
        let number = entry
            .get("number")
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of ten { number, binding } tables",
            })?;
        let binding = entry
            .get("binding")
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected: "an array of ten { number, binding } tables",
            })?;
        let binding = integer(&Item::Value(binding.clone()), key)?;
        *slot = KitSlot {
            number: ranged(&Item::Value(number.clone()), key, 0, 255)?,
            binding: u32::try_from(binding).map_err(|_| TeamTomlError::OutOfRange {
                key: key.to_string(),
                value: binding,
                range: "0 to 4294967295".to_string(),
            })?,
        };
    }
    Ok(slots)
}

pub(super) fn formation_players(
    item: &Item,
    key: &str,
) -> Result<[FormationSlot; 11], TeamTomlError> {
    let expected = "an array of eleven { position, x, y } tables";
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
    if array.len() != 11 {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected,
        });
    }
    let mut players = [FormationSlot::default(); 11];
    for (slot, value) in players.iter_mut().zip(array.iter()) {
        let entry = value
            .as_inline_table()
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
        let position = entry
            .get("position")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
        let position = labels::POSITIONS
            .iter()
            .position(|p| *p == position)
            .ok_or_else(|| TeamTomlError::UnknownLabel {
                key: key.to_string(),
                label: position.to_string(),
                allowed: labels::POSITIONS
                    .iter()
                    .map(|p| format!("\"{p}\""))
                    .collect::<Vec<_>>()
                    .join(", "),
            })?;
        let x = entry.get("x").ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected,
        })?;
        let y = entry.get("y").ok_or_else(|| TeamTomlError::WrongType {
            key: key.to_string(),
            expected,
        })?;
        *slot = FormationSlot {
            position: u8::try_from(position).expect("13 positions fit u8"),
            x: ranged(&Item::Value(x.clone()), key, 0, 255)?,
            y: ranged(&Item::Value(y.clone()), key, 0, 255)?,
        };
    }
    Ok(players)
}

pub(super) fn instruction_entries(
    item: &Item,
    key: &str,
) -> Result<[InstructionEntry; 2], TeamTomlError> {
    let expected = "an array of two { instruction, player } tables";
    let array =
        item.as_value()
            .and_then(|v| v.as_array())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
    if array.len() != 2 {
        return Err(TeamTomlError::WrongType {
            key: key.to_string(),
            expected,
        });
    }
    let mut entries = [InstructionEntry::default(); 2];
    for (entry, value) in entries.iter_mut().zip(array.iter()) {
        let table = value
            .as_inline_table()
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
        let name = table
            .get("instruction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
        let index = labels::INSTRUCTIONS
            .iter()
            .position(|l| *l == name)
            .ok_or_else(|| TeamTomlError::UnknownLabel {
                key: key.to_string(),
                label: name.to_string(),
                allowed: labels::INSTRUCTIONS
                    .iter()
                    .map(|l| format!("\"{l}\""))
                    .collect::<Vec<_>>()
                    .join(", "),
            })?;
        let player = table
            .get("player")
            .ok_or_else(|| TeamTomlError::WrongType {
                key: key.to_string(),
                expected,
            })?;
        *entry = InstructionEntry {
            instruction: Instruction::ALL[index],
            player: ranged(&Item::Value(player.clone()), key, 0, 255)?,
        };
    }
    Ok(entries)
}

/// [`opt`] where the key is required once its table is present.
pub(super) fn required<T>(
    table: &dyn TableLike,
    path: &str,
    name: &str,
    parse: impl Fn(&Item, &str) -> Result<T, TeamTomlError>,
) -> Result<T, TeamTomlError> {
    opt(table, path, name, parse)?.ok_or_else(|| TeamTomlError::MissingKey {
        key: format!("{path}.{name}"),
    })
}

/// `table.name` parsed, `None` when absent; `key` is the dotted path prefix.
pub(super) fn opt<T>(
    table: &dyn TableLike,
    path: &str,
    name: &str,
    parse: impl Fn(&Item, &str) -> Result<T, TeamTomlError>,
) -> Result<Option<T>, TeamTomlError> {
    let Some(item) = table.get(name) else {
        return Ok(None);
    };
    parse(item, &format!("{path}.{name}")).map(Some)
}

/// Every leaf of `table` must be in `known`; the first stranger is
/// `UnknownKey` at its dotted path.
pub(super) fn reject(
    table: &dyn TableLike,
    path: &str,
    known: &[&str],
) -> Result<(), TeamTomlError> {
    for (name, _) in table.iter() {
        if !known.contains(&name) {
            return Err(TeamTomlError::UnknownKey {
                key: format!("{path}.{name}"),
            });
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Apply

/// `Ok(true)` when the key applies; a foreign document gets a note, a
/// same-version document an error.
pub(super) fn gated(
    notes: &mut Vec<ImportNote>,
    foreign: bool,
    supported: bool,
    path: &str,
) -> Result<bool, TeamTomlError> {
    if supported {
        return Ok(true);
    }
    if foreign {
        notes.push(ImportNote::NotInThisVersion {
            path: path.to_string(),
        });
        Ok(false)
    } else {
        Err(TeamTomlError::NotInThisVersion {
            path: path.to_string(),
        })
    }
}

fn apply_team(
    section: &TeamSection,
    schema: &crate::schema::RecordSchema<TeamField, crate::schema::fields::TeamText>,
    foreign: bool,
    next: &mut TeamEntry,
    notes: &mut Vec<ImportNote>,
) -> Result<(), TeamTomlError> {
    if let Some(name) = &section.name {
        next.name = name.clone();
    }
    if let Some(short_name) = &section.short_name {
        next.short_name = short_name.clone();
    }
    if let Some(manager_id) = section.manager_id
        && gated(
            notes,
            foreign,
            schema.has(TeamField::ManagerId),
            "team.manager_id",
        )?
    {
        next.manager_id = Some(manager_id);
    }
    if let Some(stadium_id) = section.stadium_id
        && gated(
            notes,
            foreign,
            schema.has(TeamField::StadiumId),
            "team.stadium_id",
        )?
    {
        next.stadium_id = Some(stadium_id);
    }
    for (index, color) in [(0usize, section.color_1), (1, section.color_2)] {
        let Some(color) = color else { continue };
        let field = if index == 0 {
            TeamField::Color1Red
        } else {
            TeamField::Color2Red
        };
        if gated(
            notes,
            foreign,
            schema.has(field),
            &format!("team.color_{}", index + 1),
        )? {
            next.colors.get_or_insert_default()[index] = color;
        }
    }
    if let Some(kit_slots) = section.kit_slots
        && gated(
            notes,
            foreign,
            schema.has(TeamField::KitSlotNumber(0)),
            "team.kit_slots",
        )?
    {
        next.kit_slots = Some(kit_slots);
    }
    let flags = &section.edit_flags;
    if let Some(name) = flags.name
        && gated(
            notes,
            foreign,
            schema.has(TeamField::EditedName),
            "team.edit_flags.name",
        )?
    {
        next.edit_flags.name = name;
    }
    if let Some(short_name) = flags.short_name
        && gated(
            notes,
            foreign,
            schema.has(TeamField::EditedShortName),
            "team.edit_flags.short_name",
        )?
    {
        next.edit_flags.short_name = Some(short_name);
    }
    if let Some(stadium) = flags.stadium
        && gated(
            notes,
            foreign,
            schema.has(TeamField::EditedStadium),
            "team.edit_flags.stadium",
        )?
    {
        next.edit_flags.stadium = Some(stadium);
    }
    if let Some(strip) = flags.strip
        && gated(
            notes,
            foreign,
            schema.has(TeamField::EditedStrip),
            "team.edit_flags.strip",
        )?
    {
        next.edit_flags.strip = Some(strip);
    }
    Ok(())
}

fn apply_tactics(
    section: &TacticsSection,
    schema: &TacticsSchema,
    to: PesVersion,
    foreign: bool,
    next: &mut TeamEntry,
    notes: &mut Vec<ImportNote>,
) -> Result<(), TeamTomlError> {
    let tactics = &mut next.tactics;
    if let Some(starting) = section.starting_eleven
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::Starting(0)),
            "tactics.starting_eleven",
        )?
    {
        tactics.starting_eleven = starting;
    }
    if let Some(bench) = section.bench_order
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::Bench(0)),
            "tactics.bench_order",
        )?
    {
        tactics.bench_order = bench;
    }
    if let Some(join) = section.players_to_join_attack
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::PlayerToJoinAttack(0)),
            "tactics.players_to_join_attack",
        )?
    {
        tactics.players_to_join_attack = join;
    }
    let sp = &section.set_pieces;
    let takers = &mut tactics.set_pieces;
    let pairs: [(Option<u8>, TacticsField, &str, &mut u8); 7] = [
        (
            sp.free_kick_long,
            TacticsField::FreeKickTakerLong,
            "tactics.set_pieces.free_kick_long",
            &mut takers.free_kick_long,
        ),
        (
            sp.free_kick_short,
            TacticsField::FreeKickTakerShort,
            "tactics.set_pieces.free_kick_short",
            &mut takers.free_kick_short,
        ),
        (
            sp.free_kick_second,
            TacticsField::FreeKickTakerSecond,
            "tactics.set_pieces.free_kick_second",
            &mut takers.free_kick_second,
        ),
        (
            sp.corner_left,
            TacticsField::CornerTakerLeft,
            "tactics.set_pieces.corner_left",
            &mut takers.corner_left,
        ),
        (
            sp.corner_right,
            TacticsField::CornerTakerRight,
            "tactics.set_pieces.corner_right",
            &mut takers.corner_right,
        ),
        (
            sp.penalty,
            TacticsField::PenaltyTaker,
            "tactics.set_pieces.penalty",
            &mut takers.penalty,
        ),
        (
            sp.captain,
            TacticsField::Captain,
            "tactics.set_pieces.captain",
            &mut takers.captain,
        ),
    ];
    for (value, field, path, target) in pairs {
        if let Some(value) = value
            && gated(notes, foreign, schema.has(field), path)?
        {
            *target = value;
        }
    }
    let auto = &section.auto;
    if let Some(substitution) = auto.substitution
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::AutoSubstitution),
            "tactics.auto.substitution",
        )?
    {
        tactics.auto.substitution = Some(substitution);
    }
    if let Some(offside_trap) = auto.offside_trap
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::AutoOffsideTrap),
            "tactics.auto.offside_trap",
        )?
    {
        tactics.auto.offside_trap = Some(offside_trap);
    }
    if let Some(preset_change) = auto.preset_change
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::AutoPresetChange),
            "tactics.auto.preset_change",
        )?
    {
        tactics.auto.preset_change = Some(preset_change);
    }
    if let Some(levels) = auto.attack_defence_levels
        && gated(
            notes,
            foreign,
            schema.has(TacticsField::AutoAttackDefenceLevels),
            "tactics.auto.attack_defence_levels",
        )?
    {
        tactics.auto.attack_defence_levels = Some(levels);
    }
    for (i, preset) in section.presets.iter().enumerate() {
        let n = i + 1;
        let target = &mut tactics.presets[i];
        let style = &preset.style;
        let target_style = &mut target.style;
        let switches: [(Option<bool>, PresetField, &str, &mut bool); 7] = [
            (
                style.attacking_style,
                PresetField::AttackingStyle,
                "attacking_style",
                &mut target_style.attacking_style,
            ),
            (
                style.attacking_zone,
                PresetField::AttackingZone,
                "attacking_zone",
                &mut target_style.attacking_zone,
            ),
            (
                style.buildup,
                PresetField::Buildup,
                "buildup",
                &mut target_style.buildup,
            ),
            (
                style.positioning,
                PresetField::Positioning,
                "positioning",
                &mut target_style.positioning,
            ),
            (
                style.defensive_style,
                PresetField::DefensiveStyle,
                "defensive_style",
                &mut target_style.defensive_style,
            ),
            (
                style.containment_area,
                PresetField::ContainmentArea,
                "containment_area",
                &mut target_style.containment_area,
            ),
            (
                style.pressure,
                PresetField::Pressure,
                "pressure",
                &mut target_style.pressure,
            ),
        ];
        for (value, field, name, target) in switches {
            let path = format!("tactics.preset_{n}.style.{name}");
            if let Some(value) = value
                && gated(notes, foreign, schema.has_preset(field), &path)?
            {
                *target = value;
            }
        }
        if let Some(fluid) = style.fluid {
            let path = format!("tactics.preset_{n}.style.fluid");
            if gated(
                notes,
                foreign,
                schema.has_preset(PresetField::FluidFormation),
                &path,
            )? {
                target.style.fluid = Some(fluid);
            }
        }
        let sliders = &preset.sliders;
        let target_sliders = &mut target.sliders;
        let slider_pairs: [(Option<u8>, PresetField, &str, &mut u8); 5] = [
            (
                sliders.support_range,
                PresetField::SupportRange,
                "support_range",
                &mut target_sliders.support_range,
            ),
            (
                sliders.defensive_line,
                PresetField::DefensiveLine,
                "defensive_line",
                &mut target_sliders.defensive_line,
            ),
            (
                sliders.compactness,
                PresetField::Compactness,
                "compactness",
                &mut target_sliders.compactness,
            ),
            (
                sliders.numbers_in_attack,
                PresetField::NumbersInAttack,
                "numbers_in_attack",
                &mut target_sliders.numbers_in_attack,
            ),
            (
                sliders.numbers_in_defence,
                PresetField::NumbersInDefence,
                "numbers_in_defence",
                &mut target_sliders.numbers_in_defence,
            ),
        ];
        for (value, field, name, target) in slider_pairs {
            let path = format!("tactics.preset_{n}.sliders.{name}");
            if let Some(value) = value
                && gated(notes, foreign, schema.has_preset(field), &path)?
            {
                *target = value;
            }
        }
        for (f, formation) in preset.formations.iter().enumerate() {
            if let Some(players) = formation {
                target.formations[f].players = *players;
            }
        }
        if let Some(instructions) = &preset.instructions
            && schema.instructions.is_some()
        {
            target.attack_instructions = Some(encode_side(
                to,
                instructions.attack,
                notes,
                &format!("tactics.preset_{n}.instructions.attack"),
            )?);
            target.defence_instructions = Some(encode_side(
                to,
                instructions.defence,
                notes,
                &format!("tactics.preset_{n}.instructions.defence"),
            )?);
        }
    }
    Ok(())
}

/// One side's two entries re-encoded for `to`; an instruction the version
/// cannot store becomes `Off` (stored 0) with a `NotEncodable` note.
fn encode_side(
    to: PesVersion,
    entries: [InstructionEntry; 2],
    notes: &mut Vec<ImportNote>,
    path: &str,
) -> Result<[AdvancedInstruction; 2], TeamTomlError> {
    let mut out = [AdvancedInstruction::default(); 2];
    for (target, entry) in out.iter_mut().zip(entries.iter()) {
        let stored = match instruction::encode(to, entry.instruction) {
            Some(stored) => stored,
            None => {
                let index = Instruction::ALL
                    .iter()
                    .position(|i| *i == entry.instruction)
                    .expect("a canonical instruction is in ALL");
                notes.push(ImportNote::NotEncodable {
                    path: path.to_string(),
                    label: labels::INSTRUCTIONS[index].to_string(),
                });
                0
            }
        };
        *target = AdvancedInstruction {
            instruction: stored,
            player: entry.player,
        };
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use pes_version::PesVersion;

    use super::*;
    use crate::interchange::team_toml::{ImportNote, TeamToml, TeamTomlError};
    use crate::test_support::{FIXTURES, open};

    /// `team` with every field defaulted except `id` and `roster`, which Team
    /// TOML never carries.
    fn blank_slate(team: &TeamEntry) -> TeamEntry {
        TeamEntry {
            id: team.id,
            roster: team.roster.clone(),
            ..TeamEntry::default()
        }
    }

    /// (a) Every fixture team: document -> text -> parse is identity, and
    /// applying onto a blank team reproduces the original field for field.
    #[test]
    fn every_fixture_team_round_trips_through_the_document() {
        for version in FIXTURES {
            let (file, _) = open(version);
            let refs: Vec<&PlayerEntry> = file.players().iter().collect();
            // Same split as the player test: the model pass (from_team ->
            // apply -> field-for-field) stays exhaustive; the text pass
            // (to_toml -> parse == document, ~60 ms a team in toml_edit) runs
            // on a sample: the first team, the last, and the team whose
            // players carry the most `Some` keys.
            let docs: Vec<TeamToml> = file
                .teams()
                .iter()
                .map(|team| TeamToml::from_team(version, team, &refs).expect("from_team"))
                .collect();
            let fullest = docs
                .iter()
                .enumerate()
                .max_by_key(|(_, doc)| doc.players.values().map(player::some_keys).sum::<usize>())
                .map(|(n, _)| n)
                .expect("a team exists");
            for (n, (team, doc)) in file.teams().iter().zip(&docs).enumerate() {
                if n == 0 || n + 1 == docs.len() || n == fullest {
                    let text = doc.to_toml().expect("to_toml");
                    let parsed = TeamToml::parse(&text).unwrap_or_else(|e| {
                        panic!(
                            "{version:?} team {} reparse: {e}
{text}",
                            team.id
                        )
                    });
                    assert_eq!(parsed, *doc, "{version:?} team {}", team.id);
                }
                let mut target = blank_slate(team);
                // `apply` only looks the rostered ids up; cloning the whole
                // save's players per team was the model pass's cost.
                let mut players: Vec<PlayerEntry> = file
                    .players()
                    .iter()
                    .filter(|player| team.roster.iter().any(|slot| slot.player_id == player.id))
                    .cloned()
                    .collect();
                let notes = doc
                    .apply(version, &mut target, &mut players)
                    .expect("same-version apply");
                assert!(notes.is_empty(), "{version:?} team {}: {notes:?}", team.id);
                assert_eq!(&target, team, "{version:?} team {}", team.id);
            }
        }
    }

    /// (b) A one-key document changes exactly that key.
    #[test]
    fn a_one_key_document_changes_exactly_that_key() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file.teams().first().expect("a team");
        let doc = TeamToml::parse("[tactics.set_pieces]\npenalty = 3\n").expect("parses");
        let mut target = team.clone();
        let notes = doc
            .apply(PesVersion::Pes19, &mut target, &mut [])
            .expect("applies");
        assert!(notes.is_empty());
        let mut expected = team.clone();
        expected.tactics.set_pieces.penalty = 3;
        assert_eq!(target, expected);
    }

    /// (c) A foreign document's unsupported key is a note; the target's own
    /// version (or no version) makes it an error. Either way nothing applied.
    #[test]
    fn a_version_gated_team_field_is_a_note_across_versions_and_an_error_within() {
        let (file, _) = open(PesVersion::Pes18);
        let team = file.teams().first().expect("a team");

        let foreign =
            TeamToml::parse("pes_version = 21\n\n[team]\nmanager_id = 5\n").expect("parses");
        let mut target = team.clone();
        let notes = foreign
            .apply(PesVersion::Pes18, &mut target, &mut [])
            .expect("note, not error");
        assert_eq!(
            notes,
            vec![ImportNote::NotInThisVersion {
                path: "team.manager_id".to_string()
            }]
        );
        assert_eq!(&target, team);

        for text in [
            "[team]\nmanager_id = 5\n",
            "pes_version = 18\n\n[team]\nmanager_id = 5\n",
        ] {
            let doc = TeamToml::parse(text).expect("parses");
            let mut target = team.clone();
            let err = doc
                .apply(PesVersion::Pes18, &mut target, &mut [])
                .expect_err("hard error");
            assert!(
                matches!(err, TeamTomlError::NotInThisVersion { .. }),
                "{err:?}"
            );
            assert_eq!(&target, team);
        }
    }

    /// (d) An instruction the target cannot store degrades to Off with a note;
    /// a version with no instruction table drops the table silently.
    #[test]
    fn an_unencodable_instruction_is_a_note_and_no_table_is_silent() {
        let text = "pes_version = 19\n\n[tactics.preset_1.instructions]\n\
            attack = [{ instruction = \"wing_back\", player = 2 }, { instruction = \"off\", player = 0 }]\n\
            defence = [{ instruction = \"off\", player = 0 }, { instruction = \"off\", player = 0 }]\n";
        let doc = TeamToml::parse(text).expect("parses");

        let (file17, _) = open(PesVersion::Pes17);
        let team17 = file17.teams().first().expect("a team");
        let mut target = team17.clone();
        let notes = doc
            .apply(PesVersion::Pes17, &mut target, &mut [])
            .expect("applies");
        assert_eq!(
            notes,
            vec![ImportNote::NotEncodable {
                path: "tactics.preset_1.instructions.attack".to_string(),
                label: "wing_back".to_string()
            }]
        );
        let stored = target.tactics.presets[0]
            .attack_instructions
            .expect("PES 17 has instructions");
        assert_eq!(stored[0].instruction, 0, "wing_back fell back to Off");
        assert_eq!(stored[0].player, 2);

        let (file16, _) = open(PesVersion::Pes16);
        let team16 = file16.teams().first().expect("a team");
        let mut target = team16.clone();
        let notes = doc
            .apply(PesVersion::Pes16, &mut target, &mut [])
            .expect("applies");
        assert!(notes.is_empty(), "{notes:?}");
        assert!(target.tactics.presets[0].attack_instructions.is_none());
    }

    /// (e) Parse errors carry the dotted path, the expected shape and the
    /// allowed range/labels.
    #[test]
    fn parse_errors_are_typed() {
        let err = TeamToml::parse("[team]\nbogus = 1\n").expect_err("unknown key");
        assert!(
            matches!(&err, TeamTomlError::UnknownKey { key } if key == "team.bogus"),
            "{err:?}"
        );
        let err = TeamToml::parse("[tactics]\nbench_order = \"x\"\n").expect_err("wrong type");
        assert!(matches!(err, TeamTomlError::WrongType { .. }), "{err:?}");
        let err = TeamToml::parse("[tactics.preset_1.sliders]\nsupport_range = 11\n")
            .expect_err("out of range");
        assert!(
            matches!(
                &err,
                TeamTomlError::OutOfRange { key, value: 11, range }
                    if key == "tactics.preset_1.sliders.support_range" && range == "1 to 10"
            ),
            "{err:?}"
        );
        let err = TeamToml::parse("[tactics.preset_1.style]\nattacking_style = \"posession\"\n")
            .expect_err("unknown label");
        assert!(
            matches!(
                &err,
                TeamTomlError::UnknownLabel { key, label, allowed }
                    if key == "tactics.preset_1.style.attacking_style"
                        && label == "posession"
                        && allowed == "\"counter_attack\", \"possession\""
            ),
            "{err:?}"
        );
        let ten = (0..10)
            .map(|_| "{ position = \"GK\", x = 0, y = 0 }")
            .collect::<Vec<_>>()
            .join(", ");
        let err = TeamToml::parse(&format!(
            "[tactics.preset_1.formation_1]\nplayers = [{ten}]\n"
        ))
        .expect_err("ten entries");
        assert!(matches!(err, TeamTomlError::WrongType { .. }), "{err:?}");
    }

    /// A present `[instructions]` or `[formation_N]` table requires its keys;
    /// a missing one is `MissingKey`, not a defaulted section that `apply`
    /// would then write over the target's real instructions.
    #[test]
    fn a_present_table_missing_a_required_key_is_an_error() {
        let err = TeamToml::parse(
            "[tactics.preset_1.instructions]\n\
             defence = [{ instruction = \"off\", player = 0 }, { instruction = \"off\", player = 0 }]\n",
        )
        .expect_err("attack is required once the table exists");
        assert!(
            matches!(&err, TeamTomlError::MissingKey { key }
                if key == "tactics.preset_1.instructions.attack"),
            "{err:?}"
        );
        let err = TeamToml::parse("[tactics.preset_1.formation_1]\n")
            .expect_err("players is required once the table exists");
        assert!(
            matches!(&err, TeamTomlError::MissingKey { key }
                if key == "tactics.preset_1.formation_1.players"),
            "{err:?}"
        );
    }

    /// Every version's player schema stores a shirt-name text field, so
    /// `shirt_name_from`'s lookup `expect` is provably unreachable.
    #[test]
    fn every_version_stores_a_shirt_name() {
        for version in PesVersion::ALL {
            assert!(
                schema_for(version)
                    .player
                    .texts
                    .iter()
                    .any(|spec| spec.text == PlayerText::ShirtName),
                "{version:?}"
            );
        }
        assert_eq!(shirt_name_from("lord fulp", PesVersion::Pes19), "LORD FULP");
    }

    /// A formation emits as a multi-line array, one entry per line.
    #[test]
    fn formations_emit_one_entry_per_line() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file.teams().first().expect("a team");
        let text = TeamToml::from_team(PesVersion::Pes19, team, &[])
            .expect("from_team")
            .to_toml()
            .expect("to_toml");
        let lines: Vec<&str> = text.lines().collect();
        let at = lines
            .iter()
            .position(|l| *l == "players = [")
            .expect("a formation's players array");
        for (i, line) in lines[at + 1..at + 12].iter().enumerate() {
            assert!(
                line.starts_with("    { position = \"") && line.ends_with(" },"),
                "line {i}: {line:?}"
            );
        }
        assert_eq!(lines[at + 12], "]");
        // The multi-line form still parses back to the same document.
        let doc = TeamToml::from_team(PesVersion::Pes19, team, &[]).expect("from_team");
        assert_eq!(TeamToml::parse(&text).expect("reparse"), doc);
    }

    /// (f) The emitted text opens with the plan block's lines.
    #[test]
    fn the_emitted_text_follows_the_plan_block() {
        let (file, _) = open(PesVersion::Pes19);
        let team = file.teams().first().expect("a team");
        let text = TeamToml::from_team(PesVersion::Pes19, team, &[])
            .expect("from_team")
            .to_toml()
            .expect("to_toml");
        let lines: Vec<&str> = text.lines().collect();
        let version_body = "pes_version = 19";
        assert_eq!(
            lines[0],
            format!(
                "{version_body}{}# the save this was written from; import into another version converts",
                " ".repeat(32 - version_body.len())
            )
        );
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "[team]");
        assert!(lines[3].starts_with("id = "), "{:?}", lines[3]);
        let manager = lines
            .iter()
            .find(|l| l.starts_with("manager_id"))
            .expect("a manager_id line");
        let body = format!("manager_id = {}", team.manager_id.expect("PES 19 has it"));
        let expected = format!("{body}{}# PES 19+", " ".repeat(32 - body.len()));
        assert_eq!(*manager, expected, "the comment sits at column 33");
    }

    /// A tactics key the target's schema lacks is a `NotInThisVersion` note
    /// across versions and an error within — `has` and `has_preset` both.
    #[test]
    fn a_version_gated_tactics_field_is_a_note_across_versions_and_an_error_within() {
        let (file, _) = open(PesVersion::Pes15);
        let team = file.teams().first().expect("a team");

        for text in [
            "pes_version = 19\n\n[tactics.auto]\noffside_trap = true\n",
            "pes_version = 19\n\n[tactics.preset_1.style]\nfluid = true\n",
        ] {
            let doc = TeamToml::parse(text).expect("parses");
            let mut target = team.clone();
            let mut players = file.players().to_vec();
            let notes = doc
                .apply(PesVersion::Pes15, &mut target, &mut players)
                .expect("a foreign gated key is a note");
            let path = if text.contains("offside_trap") {
                "tactics.auto.offside_trap"
            } else {
                "tactics.preset_1.style.fluid"
            };
            assert_eq!(
                notes,
                vec![ImportNote::NotInThisVersion {
                    path: path.to_string()
                }]
            );
            assert_eq!(&target, team, "{text:?}");
        }

        for text in [
            "[tactics.auto]\noffside_trap = true\n",
            "pes_version = 15\n\n[tactics.preset_1.style]\nfluid = true\n",
        ] {
            let doc = TeamToml::parse(text).expect("parses");
            let mut target = team.clone();
            let mut players = file.players().to_vec();
            let err = doc
                .apply(PesVersion::Pes15, &mut target, &mut players)
                .expect_err("a same-version gated key is an error");
            assert!(matches!(err, TeamTomlError::NotInThisVersion { .. }));
            assert_eq!(&target, team, "{text:?}");
        }
    }

    /// `shirt_name_from`: upper-case, colour codes stripped, cut to the
    /// field's `len - 1` (18 on every version → 17 chars) by characters.
    #[test]
    fn shirt_name_from_uppercases_strips_and_cuts_by_chars() {
        assert_eq!(shirt_name_from("Snuffy", PesVersion::Pes19), "SNUFFY");
        // A `\x11c` colour code and its eight payload bytes are stripped.
        assert_eq!(
            shirt_name_from(
                "\x11c\x00\x00\x00\x00\x00\x00\x00\x00Snuffy",
                PesVersion::Pes19
            ),
            "SNUFFY"
        );
        // `ShirtName` is `len: 18` on PES 15 and 19: 17 chars + terminator.
        let long = "a".repeat(30);
        assert_eq!(shirt_name_from(&long, PesVersion::Pes19), "A".repeat(17));
        assert_eq!(shirt_name_from(&long, PesVersion::Pes15), "A".repeat(17));
        // Accented chars upper-case and count once, not per UTF-8 byte.
        assert_eq!(shirt_name_from("aéü", PesVersion::Pes19), "AÉÜ");
        let accented = "é".repeat(30);
        assert_eq!(
            shirt_name_from(&accented, PesVersion::Pes19)
                .chars()
                .count(),
            17
        );
    }
}
