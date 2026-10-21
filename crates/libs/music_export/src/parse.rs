//! The `.4ccm` line grammar: the `name` line, the `sync`/`normalize` flags and
//! one song entry per remaining line, `;`-separated.

use std::borrow::Cow;

use indexmap::IndexMap;

use crate::model::{
    Comparison, Condition, EndAction, EventClip, EventKind, Instruction, MatchType, MusicExport,
    PauseAction, Slot, SongEntry, StartOffset,
};
use crate::tokens::{UnclosedBracket, tokenize};

/// A parsed export with what the reader had to make up for.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed {
    /// The export.
    pub export: MusicExport,
    /// What the file lacked, in file order; each a stable code for the tool's messages.
    pub warnings: Vec<Warning>,
}

/// A defect the reader works around rather than refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Warning {
    /// No `name` line before the first song: the file's stem is the team.
    TeamNameMissing,
    /// The bytes are not UTF-8: read as Windows-1252, undefined bytes replaced.
    NotUtf8,
}

/// A line the reader refuses, with its 1-based line number.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("line {line}: {kind}")]
pub struct ParseError {
    /// The 1-based line number.
    pub line: usize,
    /// What is wrong with it.
    pub kind: ParseErrorKind,
}

/// What the reader refuses on a line.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseErrorKind {
    /// `name;` with nothing after it.
    #[error("the team name is empty")]
    EmptyTeamName,
    /// `name` not as the first significant line, or `sync`/`normalize` after
    /// a song entry: a spot only a player name could take, and these are
    /// reserved.
    #[error("`{0}` is a reserved word, not a player name")]
    ReservedWord(String),
    /// A line starting with `;`.
    #[error("the slot field is empty")]
    EmptySlot,
    /// `<slot>;` with an empty file field (a slot alone gets the default file).
    #[error("the file field is empty")]
    EmptyFile,
    /// An empty condition or instruction field (`;;` or a trailing `;`).
    #[error("an empty condition field")]
    EmptyField,
    /// A `[` group with no closing `]`.
    #[error("`[` with no closing `]`")]
    UnclosedBracket,
    /// A field whose first word names no condition or instruction.
    #[error("`{0}` is not a condition or instruction")]
    UnknownKeyword(String),
    /// A condition or instruction short of its arguments.
    #[error("`{keyword}` needs {what}")]
    MissingArgument {
        /// The condition or instruction.
        keyword: String,
        /// What it lacks.
        what: &'static str,
    },
    /// A word after the arguments a condition or instruction takes.
    #[error("`{keyword}` does not take `{argument}`")]
    ExtraArgument {
        /// The condition or instruction.
        keyword: String,
        /// The first word too many.
        argument: String,
    },
    /// An argument outside its condition's or instruction's vocabulary or range.
    #[error("`{argument}` is not a valid argument of `{keyword}`")]
    InvalidArgument {
        /// The condition or instruction.
        keyword: String,
        /// The word refused.
        argument: String,
    },
    /// `not` wrapping an instruction, which gates nothing.
    #[error("`not` wraps the instruction `{0}`")]
    NotOfInstruction(String),
    /// Two `event` instructions on one line.
    #[error("a second `event` on one line")]
    SecondEvent,
    /// `event` on a reserved slot, which has no player to answer for.
    #[error("`event` on the reserved slot `{0}`")]
    EventOnReservedSlot(String),
}

/// Reads a `.4ccm` file. `file_stem` is the file name without its extension,
/// the team name when the file carries no `name` line.
pub fn parse(bytes: &[u8], file_stem: &str) -> Result<Parsed, ParseError> {
    let mut warnings = Vec::new();
    let text = match std::str::from_utf8(bytes) {
        // A leading byte-order mark is dropped (Notepad writes one).
        Ok(text) => Cow::Borrowed(text.strip_prefix('\u{FEFF}').unwrap_or(text)),
        Err(_) => {
            warnings.push(Warning::NotUtf8);
            Cow::Owned(decode_windows_1252(bytes))
        }
    };
    let mut export = MusicExport {
        team: String::new(),
        sync: true,
        normalize: true,
        slots: IndexMap::new(),
        events: Vec::new(),
    };
    let mut team: Option<String> = None;
    let mut seen_line = false;
    let mut seen_entry = false;
    for (index, raw) in text.lines().enumerate() {
        let number = index + 1;
        let fail = |kind| ParseError { line: number, kind };
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split(';').map(str::trim).collect();
        let key = fields[0];
        let flag = key.to_lowercase();
        let first_line = !seen_line;
        seen_line = true;
        match key {
            // `name` is accepted only as the first significant line: a later
            // one is refused where a lenient reader loads a `name` player.
            "name" if !first_line => {
                return Err(fail(ParseErrorKind::ReservedWord(key.to_string())));
            }
            "name" => {
                let value = fields.get(1).copied().unwrap_or("");
                if value.is_empty() {
                    return Err(fail(ParseErrorKind::EmptyTeamName));
                }
                team = Some(value.to_lowercase());
            }
            // Flag keywords and values read case-insensitively (`Sync;No` is
            // a flag, not a player named Sync); after a song entry they are
            // reserved words.
            _ if flag == "sync" || flag == "normalize" => {
                if seen_entry {
                    return Err(fail(ParseErrorKind::ReservedWord(key.to_string())));
                }
                let value = fields.get(1).copied().unwrap_or("").to_lowercase();
                let enabled = !matches!(value.as_str(), "no" | "off" | "false" | "0");
                if flag == "sync" {
                    export.sync = enabled;
                } else {
                    export.normalize = enabled;
                }
            }
            _ => {
                seen_entry = true;
                let team = team.get_or_insert_with(|| {
                    warnings.push(Warning::TeamNameMissing);
                    file_stem.to_lowercase()
                });
                let (slot, entry, event) = parse_entry(&fields, team).map_err(fail)?;
                match event {
                    Some(kind) => export.events.push(EventClip {
                        player: key.to_string(),
                        kind,
                        entry,
                    }),
                    None => export.slots.entry(slot).or_default().push(entry),
                }
            }
        }
    }
    export.team = team.unwrap_or_else(|| {
        warnings.push(Warning::TeamNameMissing);
        file_stem.to_lowercase()
    });
    Ok(Parsed { export, warnings })
}

/// Windows-1252 as text, the five undefined bytes as U+FFFD.
fn decode_windows_1252(bytes: &[u8]) -> String {
    // The 0x80-0x9F row; everything else is the same code point as the byte.
    const HIGH: [char; 32] = [
        '€', '\u{FFFD}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{FFFD}', 'Ž',
        '\u{FFFD}', '\u{FFFD}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ',
        '\u{FFFD}', 'ž', 'Ÿ',
    ];
    bytes
        .iter()
        .map(|&byte| match byte {
            0x80..=0x9F => HIGH[usize::from(byte - 0x80)],
            _ => char::from(byte),
        })
        .collect()
}

/// Reads a song line's fields into its slot, entry and `event` kind.
fn parse_entry(
    fields: &[&str],
    team: &str,
) -> Result<(Slot, SongEntry, Option<EventKind>), ParseErrorKind> {
    let key = fields[0];
    if key.is_empty() {
        return Err(ParseErrorKind::EmptySlot);
    }
    let slot = Slot::from_keyword(key);
    let file = match fields.get(1).copied() {
        None => slot.default_file(team),
        Some("") => return Err(ParseErrorKind::EmptyFile),
        Some(file) => file.to_string(),
    };
    let mut entry = SongEntry {
        file,
        conditions: Vec::new(),
        instructions: Vec::new(),
    };
    let mut event = None;
    for field in fields.iter().skip(2) {
        let tokens = tokenize(field).map_err(|UnclosedBracket| ParseErrorKind::UnclosedBracket)?;
        let Some((keyword, arguments)) = tokens.split_first() else {
            return Err(ParseErrorKind::EmptyField);
        };
        match parse_item(&keyword.to_lowercase(), arguments)? {
            Item::Condition(condition) => entry.conditions.push(condition),
            Item::Instruction(instruction) => entry.instructions.push(instruction),
            Item::Event(kind) => {
                if event.is_some() {
                    return Err(ParseErrorKind::SecondEvent);
                }
                if !matches!(slot, Slot::Player(_)) {
                    return Err(ParseErrorKind::EventOnReservedSlot(key.to_string()));
                }
                event = Some(kind);
            }
        }
    }
    Ok((slot, entry, event))
}

/// One parsed field.
enum Item {
    Condition(Condition),
    Instruction(Instruction),
    Event(EventKind),
}

/// Reads one field's tokens, the keyword already lowercased.
fn parse_item(keyword: &str, arguments: &[String]) -> Result<Item, ParseErrorKind> {
    let condition = |condition| Ok(Item::Condition(condition));
    let instruction = |instruction| Ok(Item::Instruction(instruction));
    match keyword {
        "goals" => {
            let (comparison, n) = compared(keyword, arguments)?;
            condition(Condition::Goals(comparison, n))
        }
        "teamgoals" => {
            let (comparison, n) = compared(keyword, arguments)?;
            condition(Condition::TeamGoals(comparison, n))
        }
        "lead" => {
            let (comparison, n) = compared(keyword, arguments)?;
            condition(Condition::Lead(comparison, n))
        }
        "every" => {
            let n = one_argument(keyword, arguments)?;
            condition(Condition::Every(positive(keyword, n)?))
        }
        "opponent" => {
            // A bracketed list is still a list: `[a b]` names two teams.
            let teams: Vec<String> = arguments
                .iter()
                .flat_map(|argument| argument.split_whitespace())
                .map(String::from)
                .collect();
            if teams.is_empty() {
                return Err(missing(keyword, "a team name"));
            }
            condition(Condition::Opponent(teams))
        }
        "match" => {
            if arguments.is_empty() {
                return Err(missing(keyword, "a match type"));
            }
            // `knockouts` alone stands for the five stages; beside other
            // types it names none and is refused like any unknown word.
            let types = match arguments {
                [word] if word.eq_ignore_ascii_case("knockouts") => MatchType::KNOCKOUTS.to_vec(),
                _ => arguments
                    .iter()
                    .map(|word| MatchType::from_keyword(word).ok_or_else(|| invalid(keyword, word)))
                    .collect::<Result<_, _>>()?,
            };
            condition(Condition::Match(types))
        }
        "home" => condition(no_arguments(keyword, arguments, Condition::Home)?),
        "once" => condition(no_arguments(keyword, arguments, Condition::Once)?),
        "first" => condition(no_arguments(keyword, arguments, Condition::First)?),
        "comeback" => condition(no_arguments(keyword, arguments, Condition::Comeback)?),
        "mostgoals" => condition(Condition::MostGoals(optional_label(keyword, arguments)?)),
        "special" => condition(Condition::Special(optional_label(keyword, arguments)?)),
        "time" => {
            let (comparison, minute) = compared(keyword, arguments)?;
            let minute =
                u32::try_from(minute).map_err(|_| invalid(keyword, &minute.to_string()))?;
            condition(Condition::Time(comparison, minute))
        }
        "not" => {
            let Some((inner, inner_arguments)) = arguments.split_first() else {
                return Err(missing(keyword, "a condition"));
            };
            match parse_item(&inner.to_lowercase(), inner_arguments)? {
                Item::Condition(wrapped) => condition(Condition::Not(Box::new(wrapped))),
                Item::Instruction(_) | Item::Event(_) => {
                    Err(ParseErrorKind::NotOfInstruction(inner.clone()))
                }
            }
        }
        "start" => {
            let offset = one_argument(keyword, arguments)?;
            let offset = StartOffset::parse(offset).ok_or_else(|| invalid(keyword, offset))?;
            instruction(Instruction::Start(offset))
        }
        "speed" => {
            let speed = one_argument(keyword, arguments)?;
            let rate: f64 = speed.parse().map_err(|_| invalid(keyword, speed))?;
            if !rate.is_finite() || rate <= 0.0 {
                return Err(invalid(keyword, speed));
            }
            instruction(Instruction::Speed(rate))
        }
        "randomise" => instruction(no_arguments(keyword, arguments, Instruction::Randomise)?),
        "pause" => {
            let Some((action, rest)) = arguments.split_first() else {
                return Err(missing(keyword, "`continue` or `restart`"));
            };
            let action =
                PauseAction::from_keyword(action).ok_or_else(|| invalid(keyword, action))?;
            let every = match rest {
                [] => 1,
                [every, n] if every == "every" => positive(keyword, n)?,
                [word, ..] => return Err(invalid(keyword, word)),
            };
            instruction(Instruction::Pause { action, every })
        }
        "end" => {
            let action = one_argument(keyword, arguments)?;
            let action = EndAction::from_keyword(action).ok_or_else(|| invalid(keyword, action))?;
            instruction(Instruction::End(action))
        }
        "warcry" => instruction(no_arguments(keyword, arguments, Instruction::Warcry)?),
        "unrandom" => instruction(no_arguments(keyword, arguments, Instruction::Unrandom)?),
        "advance" => instruction(no_arguments(keyword, arguments, Instruction::Advance)?),
        "louder" => instruction(no_arguments(keyword, arguments, Instruction::Louder)?),
        "event" => {
            let kind = one_argument(keyword, arguments)?;
            let kind = EventKind::from_keyword(kind).ok_or_else(|| invalid(keyword, kind))?;
            Ok(Item::Event(kind))
        }
        _ => Err(ParseErrorKind::UnknownKeyword(keyword.to_string())),
    }
}

fn missing(keyword: &str, what: &'static str) -> ParseErrorKind {
    ParseErrorKind::MissingArgument {
        keyword: keyword.to_string(),
        what,
    }
}

fn invalid(keyword: &str, argument: &str) -> ParseErrorKind {
    ParseErrorKind::InvalidArgument {
        keyword: keyword.to_string(),
        argument: argument.to_string(),
    }
}

fn extra(keyword: &str, argument: &str) -> ParseErrorKind {
    ParseErrorKind::ExtraArgument {
        keyword: keyword.to_string(),
        argument: argument.to_string(),
    }
}

/// A comparison and its number: `>= 2`.
fn compared(keyword: &str, arguments: &[String]) -> Result<(Comparison, i32), ParseErrorKind> {
    match arguments {
        [] => Err(missing(keyword, "a comparison and a number")),
        [_] => Err(missing(keyword, "a number after the comparison")),
        [comparison, n, rest @ ..] => {
            if let Some(word) = rest.first() {
                return Err(extra(keyword, word));
            }
            let comparison =
                Comparison::from_keyword(comparison).ok_or_else(|| invalid(keyword, comparison))?;
            let n = n.parse().map_err(|_| invalid(keyword, n))?;
            Ok((comparison, n))
        }
    }
}

/// Exactly one argument.
fn one_argument<'a>(keyword: &str, arguments: &'a [String]) -> Result<&'a str, ParseErrorKind> {
    match arguments {
        [] => Err(missing(keyword, "an argument")),
        [argument] => Ok(argument),
        [_, word, ..] => Err(extra(keyword, word)),
    }
}

/// No argument at all: the bare keyword is the item.
fn no_arguments<T>(keyword: &str, arguments: &[String], item: T) -> Result<T, ParseErrorKind> {
    match arguments.first() {
        Some(word) => Err(extra(keyword, word)),
        None => Ok(item),
    }
}

/// At most one argument, a label.
fn optional_label(keyword: &str, arguments: &[String]) -> Result<Option<String>, ParseErrorKind> {
    match arguments {
        [] => Ok(None),
        [label] => Ok(Some(label.clone())),
        [_, word, ..] => Err(extra(keyword, word)),
    }
}

/// A whole number above zero.
fn positive(keyword: &str, word: &str) -> Result<u32, ParseErrorKind> {
    match word.parse::<u32>() {
        Ok(n) if n > 0 => Ok(n),
        _ => Err(invalid(keyword, word)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(text: &str) -> MusicExport {
        let parsed = parse(text.as_bytes(), "stem").expect("parses");
        assert!(parsed.warnings.is_empty(), "{:?}", parsed.warnings);
        parsed.export
    }

    fn refused(text: &str) -> ParseError {
        parse(text.as_bytes(), "stem").expect_err("refused")
    }

    fn entries<'a>(export: &'a MusicExport, slot: &Slot) -> &'a [SongEntry] {
        export.slots.get(slot).map(Vec::as_slice).unwrap_or(&[])
    }

    #[test]
    fn the_name_line_is_lowercased_and_flags_default_on() {
        let export = parsed("# team identifier\nname;Nijiura JP\n\nanthem;Anthem.mp3\n");
        assert_eq!(export.team, "nijiura jp");
        assert!(export.sync);
        assert!(export.normalize);
        assert_eq!(entries(&export, &Slot::Anthem)[0].file, "Anthem.mp3");
    }

    #[test]
    fn flags_read_any_order_and_any_disabling_value() {
        let export = parsed("name;x\nnormalize;No\nsync;0\nanthem;a.mp3\n");
        assert!(!export.sync);
        assert!(!export.normalize);
        let export = parsed("name;x\nsync;yes\nnormalize;off\nanthem;a.mp3\n");
        assert!(export.sync);
        assert!(!export.normalize);
        let export = parsed("name;x\nsync\nnormalize;false\nanthem;a.mp3\n");
        assert!(export.sync);
        assert!(!export.normalize);
    }

    #[test]
    fn a_byte_order_mark_is_dropped() {
        let parsed = parse("\u{FEFF}name;jp\nanthem;a.mp3\n".as_bytes(), "stem").expect("parses");
        assert!(parsed.warnings.is_empty(), "{:?}", parsed.warnings);
        assert_eq!(parsed.export.team, "jp");
    }

    /// `name` is accepted only as the first significant line: a later one is
    /// refused as the reserved word, where a lenient reader loads a player
    /// named `name`.
    #[test]
    fn name_is_the_first_significant_line_or_a_refusal() {
        assert_eq!(
            refused("sync;no\nname;jp\n"),
            ParseError {
                line: 2,
                kind: ParseErrorKind::ReservedWord("name".into()),
            }
        );
    }

    /// Flag keywords and values are case-insensitive: `Sync;No` is the sync
    /// flag, not a player named Sync.
    #[test]
    fn flags_read_case_insensitively() {
        let export = parsed("name;x\nSync;No\nNormalize;OFF\nanthem;a.mp3\n");
        assert!(!export.sync);
        assert!(!export.normalize);
        assert_eq!(export.slots.len(), 1, "`Sync` is a flag, not a player");
    }

    /// An empty group `[]` is no token: `special []` reads as a bare
    /// `special`.
    #[test]
    fn an_empty_group_is_no_token() {
        let export = parsed("name;x\nvictory;v.mp3;special []\n");
        assert_eq!(
            entries(&export, &Slot::Victory)[0].conditions,
            [Condition::Special(None)]
        );
    }

    #[test]
    fn a_missing_name_falls_back_to_the_stem_with_a_warning() {
        let parsed = parse(b"anthem\n", "JP Worldcup").expect("parses");
        assert_eq!(parsed.warnings, [Warning::TeamNameMissing]);
        assert_eq!(parsed.export.team, "jp worldcup");
        assert_eq!(
            entries(&parsed.export, &Slot::Anthem)[0].file,
            "jp worldcup - Anthem.mp3"
        );
        let parsed = parse(b"# nothing\n", "Empty").expect("parses");
        assert_eq!(parsed.warnings, [Warning::TeamNameMissing]);
        assert_eq!(parsed.export.team, "empty");
    }

    #[test]
    fn a_slot_alone_gets_its_default_file() {
        let export = parsed("name;jp\nanthem\nvictory\ngoal\nchant\nBilly\n");
        assert_eq!(entries(&export, &Slot::Anthem)[0].file, "jp - Anthem.mp3");
        assert_eq!(
            entries(&export, &Slot::Victory)[0].file,
            "jp - Victory Anthem.mp3"
        );
        assert_eq!(entries(&export, &Slot::Goal)[0].file, "jp - Goalhorn.mp3");
        assert_eq!(entries(&export, &Slot::Chant)[0].file, "jp - Chant.mp3");
        assert_eq!(
            entries(&export, &Slot::Player("Billy".into()))[0].file,
            "jp - Billy Goalhorn.mp3"
        );
    }

    #[test]
    fn slots_keep_file_order_and_group_by_first_appearance() {
        let export = parsed(
            "name;x\nchant;c1.mp3\nBilly;b1.mp3\nchant;c2.mp3\nBilly;b2.mp3\nanthem;a.mp3\n",
        );
        let slots: Vec<&Slot> = export.slots.keys().collect();
        assert_eq!(
            slots,
            [&Slot::Chant, &Slot::Player("Billy".into()), &Slot::Anthem]
        );
        let chants: Vec<&str> = entries(&export, &Slot::Chant)
            .iter()
            .map(|entry| entry.file.as_str())
            .collect();
        assert_eq!(chants, ["c1.mp3", "c2.mp3"]);
    }

    #[test]
    fn every_condition_reads_typed() {
        let line = "P;p.mp3;goals = 1;teamgoals != 2;lead <= -1;every 3;opponent a [b c];\
                    match RO16 Third-Place;home;once;first;comeback;mostgoals;mostgoals [Get Out];\
                    special;special [Promotion Anthem];time > 45;not goals >= 2\n";
        let export = parsed(&format!("name;x\n{line}"));
        let entry = &entries(&export, &Slot::Player("P".into()))[0];
        use Comparison::*;
        use Condition::*;
        assert_eq!(
            entry.conditions,
            [
                Goals(Eq, 1),
                TeamGoals(Ne, 2),
                Lead(Le, -1),
                Every(3),
                Opponent(vec!["a".into(), "b".into(), "c".into()]),
                Match(vec![MatchType::Ro16, MatchType::ThirdPlace]),
                Home,
                Once,
                First,
                Comeback,
                MostGoals(None),
                MostGoals(Some("Get Out".into())),
                Special(None),
                Special(Some("Promotion Anthem".into())),
                Time(Gt, 45),
                Not(Box::new(Goals(Ge, 2))),
            ]
        );
        assert!(entry.instructions.is_empty());
    }

    #[test]
    fn knockouts_expands_to_the_five_stages() {
        let export = parsed("name;x\nanthem;a.mp3;match Knockouts\n");
        assert_eq!(
            entries(&export, &Slot::Anthem)[0].conditions,
            [Condition::Match(MatchType::KNOCKOUTS.to_vec())]
        );
    }

    #[test]
    fn knockouts_beside_another_type_is_refused() {
        for text in [
            "name;x\nanthem;a.mp3;match knockouts final\n",
            "name;x\nanthem;a.mp3;match final knockouts\n",
        ] {
            assert_eq!(
                refused(text),
                ParseError {
                    line: 2,
                    kind: ParseErrorKind::InvalidArgument {
                        keyword: "match".into(),
                        argument: "knockouts".into(),
                    },
                },
                "{text:?}"
            );
        }
    }

    #[test]
    fn every_instruction_reads_typed() {
        let line = "P;p.mp3;start 0:39.6;speed 1.5;randomise;pause restart every 2;pause continue;\
                    end stop;end loop;warcry;unrandom;advance;louder\n";
        let export = parsed(&format!("name;x\n{line}"));
        let entry = &entries(&export, &Slot::Player("P".into()))[0];
        use Instruction::*;
        assert_eq!(
            entry.instructions,
            [
                Start(StartOffset { millis: 39_600 }),
                Speed(1.5),
                Randomise,
                Pause {
                    action: PauseAction::Restart,
                    every: 2
                },
                Pause {
                    action: PauseAction::Continue,
                    every: 1
                },
                End(EndAction::Stop),
                End(EndAction::Loop),
                Warcry,
                Unrandom,
                Advance,
                Louder,
            ]
        );
        assert!(entry.conditions.is_empty());
    }

    #[test]
    fn keywords_read_in_any_case_and_fields_trim() {
        // The keyword is case-insensitive; a vocabulary argument (`stop`) is not.
        let export = parsed("name;x\nP ; p.mp3 ; GOALS >= 2 ;End stop\n");
        let entry = &entries(&export, &Slot::Player("P".into()))[0];
        assert_eq!(entry.file, "p.mp3");
        assert_eq!(entry.conditions, [Condition::Goals(Comparison::Ge, 2)]);
        assert_eq!(entry.instructions, [Instruction::End(EndAction::Stop)]);
    }

    #[test]
    fn whitespace_only_and_indented_lines_are_skipped() {
        let export = parsed("name;x\n   \n\t\n  # indented comment\n  anthem;a.mp3  \n");
        assert_eq!(entries(&export, &Slot::Anthem)[0].file, "a.mp3");
        assert_eq!(export.slots.len(), 1, "an indented comment is no player");
    }

    #[test]
    fn not_reads_its_inner_keyword_in_any_case() {
        let export = parsed("name;x\nP;p.mp3;NOT Goals >= 2;not HOME\n");
        let entry = &entries(&export, &Slot::Player("P".into()))[0];
        assert_eq!(
            entry.conditions,
            [
                Condition::Not(Box::new(Condition::Goals(Comparison::Ge, 2))),
                Condition::Not(Box::new(Condition::Home)),
            ]
        );
    }

    #[test]
    fn an_event_line_becomes_a_clip_apart_from_the_slots() {
        let export = parsed("name;x\nBilly;b.mp3\nBilly;red.mp3;event red;start 0:02\n");
        assert_eq!(entries(&export, &Slot::Player("Billy".into())).len(), 1);
        assert_eq!(
            export.events,
            [EventClip {
                player: "Billy".into(),
                kind: EventKind::Red,
                entry: SongEntry {
                    file: "red.mp3".into(),
                    conditions: vec![],
                    instructions: vec![Instruction::Start(StartOffset { millis: 2000 })],
                },
            }]
        );
        let export =
            parsed("name;x\nA;a.mp3;event owngoal\nB;b.mp3;event sub\nC;c.mp3;event yellow\n");
        let kinds: Vec<EventKind> = export.events.iter().map(|clip| clip.kind).collect();
        assert_eq!(
            kinds,
            [EventKind::OwnGoal, EventKind::Sub, EventKind::Yellow]
        );
        assert!(export.slots.is_empty());
    }

    #[test]
    fn crlf_and_windows_1252_bytes_read_with_a_warning() {
        let export = parsed("name;x\r\n\r\nanthem;a.mp3;end stop\r\n");
        assert_eq!(entries(&export, &Slot::Anthem)[0].file, "a.mp3");
        let parsed = parse(b"name;x\nanthem;caf\xe9 \x80.mp3\n", "stem").expect("parses");
        assert_eq!(parsed.warnings, [Warning::NotUtf8]);
        assert_eq!(entries(&parsed.export, &Slot::Anthem)[0].file, "café €.mp3");
        let parsed = parse("name;x\nanthem;café.mp3\n".as_bytes(), "stem").expect("parses");
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn refusals_name_the_line_and_the_defect() {
        use ParseErrorKind::*;
        let cases: Vec<(&str, usize, ParseErrorKind)> = vec![
            ("name;\nanthem\n", 1, EmptyTeamName),
            ("name;x\nname;y\nanthem\n", 2, ReservedWord("name".into())),
            ("name;x\nanthem\nsync;no\n", 3, ReservedWord("sync".into())),
            ("name;x\nanthem\nname;y\n", 3, ReservedWord("name".into())),
            ("name;x\n;a.mp3\n", 2, EmptySlot),
            ("name;x\nanthem;\n", 2, EmptyFile),
            ("name;x\nanthem;a.mp3;\n", 2, EmptyField),
            ("name;x\nanthem;a.mp3;special [x\n", 2, UnclosedBracket),
            (
                "name;x\nanthem;a.mp3;chaos\n",
                2,
                UnknownKeyword("chaos".into()),
            ),
            (
                "name;x\nanthem;a.mp3;goals >=\n",
                2,
                MissingArgument {
                    keyword: "goals".into(),
                    what: "a number after the comparison",
                },
            ),
            (
                "name;x\nanthem;a.mp3;home now\n",
                2,
                ExtraArgument {
                    keyword: "home".into(),
                    argument: "now".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;goals => 2\n",
                2,
                InvalidArgument {
                    keyword: "goals".into(),
                    argument: "=>".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;goals >= two\n",
                2,
                InvalidArgument {
                    keyword: "goals".into(),
                    argument: "two".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;every 0\n",
                2,
                InvalidArgument {
                    keyword: "every".into(),
                    argument: "0".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;match playoff\n",
                2,
                InvalidArgument {
                    keyword: "match".into(),
                    argument: "playoff".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;time < -1\n",
                2,
                InvalidArgument {
                    keyword: "time".into(),
                    argument: "-1".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;start 1:2:3:4:5\n",
                2,
                InvalidArgument {
                    keyword: "start".into(),
                    argument: "1:2:3:4:5".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;speed 0\n",
                2,
                InvalidArgument {
                    keyword: "speed".into(),
                    argument: "0".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;pause restart each 2\n",
                2,
                InvalidArgument {
                    keyword: "pause".into(),
                    argument: "each".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;end never\n",
                2,
                InvalidArgument {
                    keyword: "end".into(),
                    argument: "never".into(),
                },
            ),
            (
                "name;x\nP;a.mp3;event blue\n",
                2,
                InvalidArgument {
                    keyword: "event".into(),
                    argument: "blue".into(),
                },
            ),
            (
                "name;x\nanthem;a.mp3;not end stop\n",
                2,
                NotOfInstruction("end".into()),
            ),
            ("name;x\nP;a.mp3;event red;event sub\n", 2, SecondEvent),
            (
                "name;x\ngoal;a.mp3;event red\n",
                2,
                EventOnReservedSlot("goal".into()),
            ),
        ];
        for (text, line, kind) in cases {
            assert_eq!(refused(text), ParseError { line, kind }, "{text:?}");
        }
    }

    #[test]
    fn errors_display_their_line() {
        let error = refused("name;x\n\n# c\nanthem;a.mp3;goals >= two\n");
        assert_eq!(
            error.to_string(),
            "line 4: `two` is not a valid argument of `goals`"
        );
    }
}
