//! The canonical `.4ccm` layout: a `# team identifier` block, the reserved
//! slots, the regular players, then the event clips. Hand-written comments
//! and blank lines are not kept; flags are written only when opted out.

use crate::model::{Condition, EventClip, Instruction, MusicExport, Slot, SongEntry};
use crate::tokens::write_token;

/// Writes the export in the canonical layout, every line `\n`-terminated.
/// Empty slots are left out; reserved slots come first, in anthem, victory,
/// goal, chant order, then the players in the model's order.
pub fn write(export: &MusicExport) -> String {
    let mut text = String::new();
    text.push_str("# team identifier\n");
    text.push_str(&format!("name;{}\n", export.team));
    if !export.sync {
        text.push_str("sync;no\n");
    }
    if !export.normalize {
        text.push_str("normalize;no\n");
    }
    text.push_str("\n# reserved names\n");
    for slot in [Slot::Anthem, Slot::Victory, Slot::Goal, Slot::Chant] {
        if let Some(entries) = export.slots.get(&slot) {
            write_entries(&mut text, &slot, entries);
        }
    }
    let players: Vec<(&Slot, &Vec<SongEntry>)> = export
        .slots
        .iter()
        .filter(|(slot, entries)| matches!(slot, Slot::Player(_)) && !entries.is_empty())
        .collect();
    if !players.is_empty() {
        text.push_str("\n# regular players\n");
        for (slot, entries) in players {
            write_entries(&mut text, slot, entries);
        }
    }
    if !export.events.is_empty() {
        text.push_str("\n# event clips\n");
        for clip in &export.events {
            text.push_str(&event_line(clip));
            text.push('\n');
        }
    }
    text
}

fn write_entries(text: &mut String, slot: &Slot, entries: &[SongEntry]) {
    for entry in entries {
        text.push_str(&entry_line(slot.keyword(), entry));
        text.push('\n');
    }
}

/// `<slot>;<file>;<conditions...>;<instructions...>`.
fn entry_line(slot: &str, entry: &SongEntry) -> String {
    let mut line = format!("{slot};{}", entry.file);
    for condition in &entry.conditions {
        line.push(';');
        line.push_str(&condition_text(condition));
    }
    for instruction in &entry.instructions {
        line.push(';');
        line.push_str(&instruction_text(instruction));
    }
    line
}

/// The entry line with `;event <kind>` after the instructions.
fn event_line(clip: &EventClip) -> String {
    format!(
        "{};event {}",
        entry_line(&clip.player, &clip.entry),
        clip.kind.keyword()
    )
}

/// A condition as a field: its keyword and arguments.
pub(crate) fn condition_text(condition: &Condition) -> String {
    match condition {
        Condition::Goals(comparison, n) => format!("goals {} {n}", comparison.keyword()),
        Condition::TeamGoals(comparison, n) => format!("teamgoals {} {n}", comparison.keyword()),
        Condition::Lead(comparison, n) => format!("lead {} {n}", comparison.keyword()),
        Condition::Every(n) => format!("every {n}"),
        Condition::Opponent(teams) => {
            let names: Vec<String> = teams.iter().map(|team| write_token(team)).collect();
            format!("opponent {}", names.join(" "))
        }
        Condition::Match(types) => {
            let names: Vec<&str> = types.iter().map(|kind| kind.keyword()).collect();
            format!("match {}", names.join(" "))
        }
        Condition::Home => "home".to_string(),
        Condition::Once => "once".to_string(),
        Condition::First => "first".to_string(),
        Condition::Comeback => "comeback".to_string(),
        Condition::MostGoals(player) => labeled("mostgoals", player.as_deref()),
        Condition::Special(label) => labeled("special", label.as_deref()),
        Condition::Time(comparison, minute) => format!("time {} {minute}", comparison.keyword()),
        Condition::Not(inner) => format!("not {}", condition_text(inner)),
    }
}

fn labeled(keyword: &str, label: Option<&str>) -> String {
    match label {
        Some(label) => format!("{keyword} {}", write_token(label)),
        None => keyword.to_string(),
    }
}

/// An instruction as a field: its keyword and arguments.
pub(crate) fn instruction_text(instruction: &Instruction) -> String {
    match instruction {
        Instruction::Start(offset) => format!("start {offset}"),
        Instruction::Speed(rate) => format!("speed {rate}"),
        Instruction::Randomise => "randomise".to_string(),
        Instruction::Pause { action, every: 1 } => format!("pause {}", action.keyword()),
        Instruction::Pause { action, every } => {
            format!("pause {} every {every}", action.keyword())
        }
        Instruction::End(action) => format!("end {}", action.keyword()),
        Instruction::Warcry => "warcry".to_string(),
        Instruction::Unrandom => "unrandom".to_string(),
        Instruction::Advance => "advance".to_string(),
        Instruction::Louder => "louder".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Comparison, EndAction, EventKind, MatchType, PauseAction, StartOffset};
    use crate::parse::parse;
    use indexmap::IndexMap;

    fn entry(file: &str, conditions: Vec<Condition>, instructions: Vec<Instruction>) -> SongEntry {
        SongEntry {
            file: file.to_string(),
            conditions,
            instructions,
        }
    }

    #[test]
    fn the_layout_has_its_three_sections_and_opt_out_flags() {
        let mut slots = IndexMap::new();
        slots.insert(
            Slot::Player("Get Out".into()),
            vec![entry(
                "marisa-2.mp3",
                vec![Condition::Goals(Comparison::Ge, 2)],
                vec![],
            )],
        );
        slots.insert(Slot::Goal, vec![entry("Kurenai.mp3", vec![], vec![])]);
        slots.insert(Slot::Chant, vec![]);
        slots.insert(
            Slot::Anthem,
            vec![entry(
                "theme1.mp3",
                vec![Condition::Opponent(vec!["lit".into()])],
                vec![Instruction::End(EndAction::Stop)],
            )],
        );
        let export = MusicExport {
            team: "jp".into(),
            sync: false,
            normalize: true,
            slots,
            events: vec![EventClip {
                player: "Get Out".into(),
                kind: EventKind::Red,
                entry: entry(
                    "red.mp3",
                    vec![],
                    vec![Instruction::Start(StartOffset { millis: 2000 })],
                ),
            }],
        };
        assert_eq!(
            write(&export),
            "# team identifier\n\
             name;jp\n\
             sync;no\n\
             \n\
             # reserved names\n\
             anthem;theme1.mp3;opponent lit;end stop\n\
             goal;Kurenai.mp3\n\
             \n\
             # regular players\n\
             Get Out;marisa-2.mp3;goals >= 2\n\
             \n\
             # event clips\n\
             Get Out;red.mp3;start 0:02;event red\n"
        );
    }

    #[test]
    fn an_export_with_only_reserved_slots_has_no_player_section() {
        let mut slots = IndexMap::new();
        slots.insert(Slot::Anthem, vec![entry("a.mp3", vec![], vec![])]);
        let export = MusicExport {
            team: "x".into(),
            sync: true,
            normalize: false,
            slots,
            events: vec![],
        };
        assert_eq!(
            write(&export),
            "# team identifier\nname;x\nnormalize;no\n\n# reserved names\nanthem;a.mp3\n"
        );
    }

    #[test]
    fn every_condition_writes_its_keyword_form() {
        use Comparison::*;
        use Condition::*;
        let cases = [
            (Goals(Eq, 1), "goals == 1"),
            (TeamGoals(Ne, 2), "teamgoals != 2"),
            (Lead(Le, -1), "lead <= -1"),
            (Every(3), "every 3"),
            (Opponent(vec!["a".into(), "b".into()]), "opponent a b"),
            (
                Match(vec![MatchType::Ro16, MatchType::ThirdPlace]),
                "match ro16 third-place",
            ),
            (Home, "home"),
            (Once, "once"),
            (First, "first"),
            (Comeback, "comeback"),
            (MostGoals(None), "mostgoals"),
            (MostGoals(Some("Get Out".into())), "mostgoals [Get Out]"),
            (Special(None), "special"),
            (
                Special(Some("Promotion Anthem".into())),
                "special [Promotion Anthem]",
            ),
            (Time(Gt, 45), "time > 45"),
            (Not(Box::new(Goals(Ge, 2))), "not goals >= 2"),
        ];
        for (condition, text) in cases {
            assert_eq!(condition_text(&condition), text);
        }
    }

    #[test]
    fn every_instruction_writes_its_keyword_form() {
        use Instruction::*;
        let cases = [
            (Start(StartOffset { millis: 39_600 }), "start 0:39.6"),
            (Speed(1.5), "speed 1.5"),
            (Speed(2.0), "speed 2"),
            (Randomise, "randomise"),
            (
                Pause {
                    action: PauseAction::Restart,
                    every: 2,
                },
                "pause restart every 2",
            ),
            (
                Pause {
                    action: PauseAction::Continue,
                    every: 1,
                },
                "pause continue",
            ),
            (End(EndAction::Stop), "end stop"),
            (End(EndAction::Loop), "end loop"),
            (Warcry, "warcry"),
            (Unrandom, "unrandom"),
            (Advance, "advance"),
            (Louder, "louder"),
        ];
        for (instruction, text) in cases {
            assert_eq!(instruction_text(&instruction), text);
        }
    }

    #[test]
    fn a_written_export_reads_back_equal() {
        let text = "name;Nijiura JP\nsync;no\n\
                    anthem;Anthem.mp3;match knockouts;start 0:39.6\n\
                    victory;VA.mp3;special [Promotion Anthem];end stop\n\
                    goal;Goalhorn.mp3;opponent a [b c];pause restart every 3\n\
                    chant;c.mp3;unrandom\n\
                    Get Out;marisa-2.mp3;goals = 2;not first;mostgoals [Get Out];speed 1.25\n\
                    Get Out;red.mp3;event red\n\
                    Billy;b.mp3;every 2;time <= 45;warcry;louder;advance;randomise\n";
        let first = parse(text.as_bytes(), "stem").expect("parses").export;
        let written = write(&first);
        let second = parse(written.as_bytes(), "stem").expect("the written text parses");
        assert!(second.warnings.is_empty());
        assert_eq!(second.export, first);
        assert_eq!(write(&second.export), written, "the written form is stable");
    }
}
