//! The typed model of a `.4ccm` export: slots, song entries, conditions,
//! instructions and event clips, plus the keyword form of each closed set.

use indexmap::IndexMap;
use std::fmt;

/// A team's music export: the parsed `.4ccm` file.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicExport {
    /// The team name, lowercased on load (team names match case-insensitively).
    pub team: String,
    /// Shared goalhorn positions across the team's buttons (default true).
    pub sync: bool,
    /// Loudness normalization opt-out flag (default true: normalize).
    pub normalize: bool,
    /// Each slot's entries in file order, slots in order of first appearance;
    /// no `event` entries.
    pub slots: IndexMap<Slot, Vec<SongEntry>>,
    /// The `event` entries, file order preserved.
    pub events: Vec<EventClip>,
}

/// What a song entry is for: one of the four reserved slots, or a player's
/// goalhorn button.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The team anthem.
    Anthem,
    /// A victory anthem.
    Victory,
    /// The default goalhorn, the fallback of every player.
    Goal,
    /// A chant.
    Chant,
    /// A player's goalhorn; the name is free text, commas rendering as line
    /// breaks on the button.
    Player(String),
}

/// One song line: its file and the conditions and instructions on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SongEntry {
    /// The song's path relative to the `.4ccm`'s folder, as written.
    pub file: String,
    /// All must hold for the entry to play (AND).
    pub conditions: Vec<Condition>,
    /// How the entry plays once chosen.
    pub instructions: Vec<Instruction>,
}

/// The match event an event clip answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// A red card (a second yellow included).
    Red,
    /// A yellow card.
    Yellow,
    /// An own goal.
    OwnGoal,
    /// A substitution on.
    Sub,
}

/// A clip played once when the Match tracker reports its event for its player;
/// never in a slot's priority list.
#[derive(Debug, Clone, PartialEq)]
pub struct EventClip {
    /// The line's slot field (a player name, matched case-insensitively).
    pub player: String,
    /// The `event` instruction's argument.
    pub kind: EventKind,
    /// The line's other fields; conditions kept for the round trip, never
    /// evaluated.
    pub entry: SongEntry,
}

/// A comparison operator of the numeric conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    /// `==` (`=` reads the same).
    Eq,
    /// `!=`
    Ne,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `<=`
    Le,
    /// `>=`
    Ge,
}

/// A cup match type, the `match` condition's vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    /// A group-stage match.
    Group,
    /// A survival match.
    Survival,
    /// A round of 16 match.
    Ro16,
    /// A quarterfinal.
    Quarterfinal,
    /// A semifinal.
    Semifinal,
    /// The final.
    Final,
    /// The third-place match.
    ThirdPlace,
    /// A boss match.
    Boss,
    /// A consolation match.
    Consolation,
}

/// A gate on whether an entry plays, one variant per `.4ccm` condition.
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// This player's goal count compares true.
    Goals(Comparison, i32),
    /// The team's total goals compare true.
    TeamGoals(Comparison, i32),
    /// The goal difference (us minus them) compares true.
    Lead(Comparison, i32),
    /// The player's goal count is divisible by n (n > 0).
    Every(u32),
    /// The opponent is one of the listed teams (case-insensitive, no slashes).
    Opponent(Vec<String>),
    /// The match type is one of the listed ones (`knockouts` expands on read).
    Match(Vec<MatchType>),
    /// The team is in the home slot.
    Home,
    /// First check only; the entry is unloaded afterwards.
    Once,
    /// This is the team's first goal.
    First,
    /// After scoring, still level or behind, and the opponent has scored.
    Comeback,
    /// This (or the named) player has the most goals on the team; ties count.
    MostGoals(Option<String>),
    /// Never auto-plays; a victory anthem selectable from the dropdown, under
    /// its label when one is given.
    Special(Option<String>),
    /// The goal minute compares true.
    Time(Comparison, u32),
    /// The wrapped condition does not hold.
    Not(Box<Condition>),
}

/// A playback offset, the `start` instruction's argument.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartOffset {
    /// The offset in milliseconds.
    pub millis: u64,
}

/// What happens when a playing entry is paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    /// Keep the position.
    Continue,
    /// Go back to the start (offset).
    Restart,
}

/// What happens when a playing entry reaches its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndAction {
    /// Stop and reload instead of looping.
    Stop,
    /// Loop, which goalhorns do by default: a no-op kept so the line round-trips.
    Loop,
}

/// How a chosen entry plays, one variant per `.4ccm` instruction except
/// `event`, which lifts the entry into an [`EventClip`].
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Begin playback at this offset; looping returns to it.
    Start(StartOffset),
    /// Fixed playback speed (0.25 to 4.00 play; outside that the engine clamps).
    Speed(f64),
    /// When all of a player's non-warcry entries carry it: pick at random.
    Randomise,
    /// On pause, keep the position or restart, every nth pause.
    Pause {
        /// Keep or restart.
        action: PauseAction,
        /// Restart only every nth pause (1: every pause).
        every: u32,
    },
    /// At song end: stop, or loop (the default).
    End(EndAction),
    /// Play to completion first, then fall through to the next non-warcry entry.
    Warcry,
    /// Exclude this chant from random chant selection.
    Unrandom,
    /// At song end, play the next valid entry instead of looping.
    Advance,
    /// Mark for the team's volume-boost slider.
    Louder,
}

impl Slot {
    /// The slot's first field in the file: the reserved keyword or the player name.
    pub fn keyword(&self) -> &str {
        match self {
            Slot::Anthem => "anthem",
            Slot::Victory => "victory",
            Slot::Goal => "goal",
            Slot::Chant => "chant",
            Slot::Player(name) => name,
        }
    }

    /// The slot a line's first field names; reserved keywords are matched exactly.
    pub(crate) fn from_keyword(field: &str) -> Slot {
        match field {
            "anthem" => Slot::Anthem,
            "victory" => Slot::Victory,
            "goal" => Slot::Goal,
            "chant" => Slot::Chant,
            name => Slot::Player(name.to_string()),
        }
    }

    /// The song file assumed when a line names only its slot:
    /// `{team} - {Label}.mp3`, the label being the slot's button text.
    pub(crate) fn default_file(&self, team: &str) -> String {
        match self {
            Slot::Anthem => format!("{team} - Anthem.mp3"),
            Slot::Victory => format!("{team} - Victory Anthem.mp3"),
            Slot::Goal => format!("{team} - Goalhorn.mp3"),
            Slot::Chant => format!("{team} - Chant.mp3"),
            Slot::Player(name) => format!("{team} - {name} Goalhorn.mp3"),
        }
    }
}

impl Comparison {
    /// The operator as read; `=` is accepted for `==`.
    pub(crate) fn from_keyword(word: &str) -> Option<Comparison> {
        Some(match word {
            "=" | "==" => Comparison::Eq,
            "!=" => Comparison::Ne,
            "<" => Comparison::Lt,
            ">" => Comparison::Gt,
            "<=" => Comparison::Le,
            ">=" => Comparison::Ge,
            _ => return None,
        })
    }

    /// The operator as written.
    pub fn keyword(self) -> &'static str {
        match self {
            Comparison::Eq => "==",
            Comparison::Ne => "!=",
            Comparison::Lt => "<",
            Comparison::Gt => ">",
            Comparison::Le => "<=",
            Comparison::Ge => ">=",
        }
    }

    /// Whether `left <op> right` holds.
    pub fn holds(self, left: i64, right: i64) -> bool {
        match self {
            Comparison::Eq => left == right,
            Comparison::Ne => left != right,
            Comparison::Lt => left < right,
            Comparison::Gt => left > right,
            Comparison::Le => left <= right,
            Comparison::Ge => left >= right,
        }
    }
}

impl MatchType {
    /// Every match type, in the cup's order.
    pub const ALL: [MatchType; 9] = [
        MatchType::Group,
        MatchType::Survival,
        MatchType::Ro16,
        MatchType::Quarterfinal,
        MatchType::Semifinal,
        MatchType::Final,
        MatchType::ThirdPlace,
        MatchType::Boss,
        MatchType::Consolation,
    ];

    /// The knockout stages, what the `knockouts` token stands for.
    pub const KNOCKOUTS: [MatchType; 5] = [
        MatchType::Ro16,
        MatchType::Quarterfinal,
        MatchType::Semifinal,
        MatchType::Final,
        MatchType::ThirdPlace,
    ];

    /// The type as written in a `match` condition (lowercase; read case-insensitively).
    pub fn keyword(self) -> &'static str {
        match self {
            MatchType::Group => "group",
            MatchType::Survival => "survival",
            MatchType::Ro16 => "ro16",
            MatchType::Quarterfinal => "quarterfinal",
            MatchType::Semifinal => "semifinal",
            MatchType::Final => "final",
            MatchType::ThirdPlace => "third-place",
            MatchType::Boss => "boss",
            MatchType::Consolation => "consolation",
        }
    }

    /// The type a `match` token names, whatever its case.
    pub(crate) fn from_keyword(word: &str) -> Option<MatchType> {
        MatchType::ALL
            .into_iter()
            .find(|kind| kind.keyword().eq_ignore_ascii_case(word))
    }
}

impl EventKind {
    /// The kind as written after `event`.
    pub fn keyword(self) -> &'static str {
        match self {
            EventKind::Red => "red",
            EventKind::Yellow => "yellow",
            EventKind::OwnGoal => "owngoal",
            EventKind::Sub => "sub",
        }
    }

    pub(crate) fn from_keyword(word: &str) -> Option<EventKind> {
        [
            EventKind::Red,
            EventKind::Yellow,
            EventKind::OwnGoal,
            EventKind::Sub,
        ]
        .into_iter()
        .find(|kind| kind.keyword() == word)
    }
}

impl PauseAction {
    /// The action as written after `pause`.
    pub fn keyword(self) -> &'static str {
        match self {
            PauseAction::Continue => "continue",
            PauseAction::Restart => "restart",
        }
    }

    pub(crate) fn from_keyword(word: &str) -> Option<PauseAction> {
        match word {
            "continue" => Some(PauseAction::Continue),
            "restart" => Some(PauseAction::Restart),
            _ => None,
        }
    }
}

impl EndAction {
    /// The action as written after `end`.
    pub fn keyword(self) -> &'static str {
        match self {
            EndAction::Stop => "stop",
            EndAction::Loop => "loop",
        }
    }

    pub(crate) fn from_keyword(word: &str) -> Option<EndAction> {
        match word {
            "stop" => Some(EndAction::Stop),
            "loop" => Some(EndAction::Loop),
            _ => None,
        }
    }
}

impl StartOffset {
    /// Reads `[[d:]h:]m:s` or plain seconds, the seconds with up to three
    /// decimals (`0:39.6`, `00:22.617`, `45`). Finer than a millisecond is
    /// refused rather than rounded, so the model never writes a different
    /// offset than it read.
    pub fn parse(text: &str) -> Option<StartOffset> {
        let parts: Vec<&str> = text.split(':').collect();
        if parts.is_empty() || parts.len() > 4 {
            return None;
        }
        let (seconds, larger) = parts.split_last()?;
        let mut millis: u64 = 0;
        // Minutes, hours, days, from the right.
        for (unit, millis_per_unit) in larger.iter().rev().zip([60_000, 3_600_000, 86_400_000]) {
            let value: u64 = unit.parse().ok()?;
            millis = millis.checked_add(value.checked_mul(millis_per_unit)?)?;
        }
        let (whole, fraction) = seconds.split_once('.').unwrap_or((seconds, ""));
        let whole: u64 = whole.parse().ok()?;
        if fraction.len() > 3 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let scale = match fraction.len() {
            1 => 100,
            2 => 10,
            _ => 1,
        };
        let fraction_millis = if fraction.is_empty() {
            0
        } else {
            fraction.parse::<u64>().ok()? * scale
        };
        millis = millis.checked_add(whole.checked_mul(1000)?)?;
        millis = millis.checked_add(fraction_millis)?;
        Some(StartOffset { millis })
    }
}

impl fmt::Display for StartOffset {
    /// `m:ss`, `h:mm:ss` from an hour up, the decimals only when nonzero
    /// and without trailing zeros (`0:39.6`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_seconds = self.millis / 1000;
        let fraction = self.millis % 1000;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds / 60) % 60;
        let seconds = total_seconds % 60;
        if hours > 0 {
            write!(f, "{hours}:{minutes:02}:{seconds:02}")?;
        } else {
            write!(f, "{minutes}:{seconds:02}")?;
        }
        if fraction > 0 {
            let digits = format!("{fraction:03}");
            write!(f, ".{}", digits.trim_end_matches('0'))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_offsets_read_every_written_form() {
        let cases = [
            ("0:02", 2_000),
            ("45", 45_000),
            ("0:39.6", 39_600),
            ("00:22.617", 22_617),
            ("1:30", 90_000),
            ("1:02:03", 3_723_000),
            ("1:00:00:00", 86_400_000),
            ("0:00.05", 50),
            ("0:3.", 3_000),
        ];
        for (text, millis) in cases {
            assert_eq!(
                StartOffset::parse(text),
                Some(StartOffset { millis }),
                "{text}"
            );
        }
        for bad in [
            "",
            "a",
            "1:",
            ":5",
            "1:2:3:4:5",
            "0:39.6175",
            "0:.5",
            "-1",
            "1.5:00",
        ] {
            assert_eq!(StartOffset::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn start_offsets_write_the_shortest_form_that_reads_back() {
        let cases = [
            (2_000, "0:02"),
            (45_000, "0:45"),
            (39_600, "0:39.6"),
            (22_617, "0:22.617"),
            (3_723_000, "1:02:03"),
            (50, "0:00.05"),
            (0, "0:00"),
        ];
        for (millis, text) in cases {
            let offset = StartOffset { millis };
            assert_eq!(offset.to_string(), text);
            assert_eq!(StartOffset::parse(text), Some(offset));
        }
    }

    #[test]
    fn comparisons_read_both_equality_spellings_and_hold() {
        assert_eq!(Comparison::from_keyword("="), Some(Comparison::Eq));
        assert_eq!(Comparison::from_keyword("=="), Some(Comparison::Eq));
        assert_eq!(Comparison::from_keyword("=>"), None);
        assert!(Comparison::Le.holds(2, 2));
        assert!(!Comparison::Lt.holds(2, 2));
        assert!(Comparison::Ne.holds(1, 2));
        assert!(Comparison::Ge.holds(3, 2));
        assert!(!Comparison::Gt.holds(2, 3));
        assert!(!Comparison::Gt.holds(2, 2));
        assert!(Comparison::Eq.holds(-1, -1));
    }

    #[test]
    fn match_types_read_case_insensitively_and_write_lowercase() {
        for kind in MatchType::ALL {
            assert_eq!(MatchType::from_keyword(kind.keyword()), Some(kind));
            assert_eq!(
                MatchType::from_keyword(&kind.keyword().to_uppercase()),
                Some(kind)
            );
        }
        assert_eq!(
            MatchType::from_keyword("Third-Place"),
            Some(MatchType::ThirdPlace)
        );
        assert_eq!(MatchType::from_keyword("knockouts"), None);
        assert_eq!(MatchType::from_keyword("playoff"), None);
    }

    #[test]
    fn default_files_follow_the_slot_label() {
        assert_eq!(Slot::Anthem.default_file("jp"), "jp - Anthem.mp3");
        assert_eq!(Slot::Victory.default_file("jp"), "jp - Victory Anthem.mp3");
        assert_eq!(Slot::Goal.default_file("jp"), "jp - Goalhorn.mp3");
        assert_eq!(Slot::Chant.default_file("jp"), "jp - Chant.mp3");
        assert_eq!(
            Slot::Player("Billy".into()).default_file("jp"),
            "jp - Billy Goalhorn.mp3"
        );
    }

    #[test]
    fn reserved_slot_keywords_match_exactly() {
        assert_eq!(Slot::from_keyword("goal"), Slot::Goal);
        assert_eq!(Slot::from_keyword("Goal"), Slot::Player("Goal".into()));
        assert_eq!(Slot::Goal.keyword(), "goal");
        assert_eq!(Slot::Player("Get Out".into()).keyword(), "Get Out");
    }
}
