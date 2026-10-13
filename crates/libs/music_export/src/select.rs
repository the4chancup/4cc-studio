//! Song selection for one button press: the slot's priority list, with the
//! default goalhorn appended as every player's fallback, walked top-down with
//! unloads, the randomise pool, warcries and `advance` skips as the plan's
//! "Song selection" states. Pure: the caller supplies the random pick.

use crate::eval::Verdict;
use crate::model::{Instruction, MusicExport, Slot, SongEntry};
use crate::state::{MatchState, Side};

/// One entry of a priority list with what the match has done to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    /// The entry.
    pub entry: SongEntry,
    /// From the default goalhorn (`goal`) slot, appended after the player's
    /// own entries: it is not the player's for the randomise check or pool,
    /// and its conditions count the `goal` button's own presses.
    pub fallback: bool,
    /// A `once` entry has had its first check; the next unloads it.
    checked_once: bool,
}

/// A button's priority list, in file order, shrinking as entries unload.
#[derive(Debug, Clone, PartialEq)]
pub struct PriorityList {
    entries: Vec<Loaded>,
}

/// One press of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Press<'a> {
    /// The team the button belongs to.
    pub side: Side,
    /// The button's slot name: the player, or the reserved keyword.
    pub player: &'a str,
    /// The entry that just ended (`advance`), skipped this time: its index
    /// into `entries()` as the list stood when the press began.
    pub skip: Option<usize>,
    /// A warcry already played for this press: warcries are excluded now.
    pub after_warcry: bool,
}

impl PriorityList {
    /// `slot`'s entries, then the `goal` slot's when `slot` is a player
    /// (reserved slots and the default goalhorn itself get no fallback).
    pub fn for_slot(export: &MusicExport, slot: &Slot) -> PriorityList {
        let own = export.slots.get(slot).map(Vec::as_slice).unwrap_or(&[]);
        let mut entries: Vec<Loaded> = own.iter().map(|entry| load(entry, false)).collect();
        if let Slot::Player(_) = slot
            && let Some(fallback) = export.slots.get(&Slot::Goal)
        {
            entries.extend(fallback.iter().map(|entry| load(entry, true)));
        }
        PriorityList { entries }
    }

    /// The entries still loaded, in priority order.
    pub fn entries(&self) -> &[Loaded] {
        &self.entries
    }

    /// Whether any loaded entry needs the goal minute before a press.
    pub fn needs_goal_minute(&self) -> bool {
        self.entries
            .iter()
            .any(|loaded| loaded.entry.needs_goal_minute())
    }

    /// Picks the entry to play, as an index into [`entries`](Self::entries)
    /// after this press's unloads, or `None` when nothing passes.
    ///
    /// Top-down: the skipped entry is passed over, an entry that can never
    /// play again (or a `once` already checked) is unloaded, and the first
    /// passing entry plays. Before any check, an entry carrying `randomise`
    /// whose slot's own non-warcry entries all carry it is drawn from that
    /// pool by `pick` (given the pool's size, returning an index into it),
    /// conditions unread. After a warcry has played, warcries are passed over.
    pub fn select(
        &mut self,
        state: &MatchState,
        press: Press<'_>,
        pick: &mut impl FnMut(usize) -> usize,
    ) -> Option<usize> {
        if !press.after_warcry {
            let warcries: Vec<usize> = self
                .positions(|loaded| loaded.entry.is_warcry() && !loaded.fallback)
                .into_iter()
                .filter(|&i| Some(i) != press.skip)
                .collect();
            if !warcries.is_empty()
                && warcries
                    .iter()
                    .all(|&i| self.entries[i].entry.is_randomise())
            {
                return Some(warcries[pick(warcries.len())]);
            }
        }
        let mut skip = press.skip;
        let mut i = 0;
        while i < self.entries.len() {
            if Some(i) == skip {
                i += 1;
                continue;
            }
            let loaded = &mut self.entries[i];
            let player = if loaded.fallback {
                Slot::Goal.keyword()
            } else {
                press.player
            };
            let verdict = if loaded.entry.is_once() && loaded.checked_once {
                Verdict::NeverAgain
            } else {
                loaded.checked_once = true;
                loaded.entry.eval(state, press.side, player)
            };
            let (randomise, warcry, fallback) = (
                loaded.entry.is_randomise(),
                loaded.entry.is_warcry(),
                loaded.fallback,
            );
            if verdict == Verdict::NeverAgain {
                self.entries.remove(i);
                // An entry unloaded before the skipped one moves its index
                // down with it.
                if skip.is_some_and(|skip| i < skip) {
                    skip = skip.map(|skip| skip - 1);
                }
                continue;
            }
            if randomise && !warcry && !fallback {
                let pool = self.positions(|loaded| !loaded.fallback && !loaded.entry.is_warcry());
                if pool.iter().all(|&i| self.entries[i].entry.is_randomise()) {
                    return Some(pool[pick(pool.len())]);
                }
            }
            if verdict == Verdict::True && !(press.after_warcry && warcry) {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn positions(&self, keep: impl Fn(&Loaded) -> bool) -> Vec<usize> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, loaded)| keep(loaded))
            .map(|(i, _)| i)
            .collect()
    }
}

fn load(entry: &SongEntry, fallback: bool) -> Loaded {
    Loaded {
        entry: entry.clone(),
        fallback,
        checked_once: false,
    }
}

impl SongEntry {
    /// Whether the entry carries `warcry`.
    pub fn is_warcry(&self) -> bool {
        self.instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Warcry))
    }

    /// Whether the entry carries `randomise`.
    pub fn is_randomise(&self) -> bool {
        self.instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Randomise))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Comparison, Condition, MatchType};
    use crate::parse::parse;

    fn export(text: &str) -> MusicExport {
        parse(format!("name;jp\n{text}").as_bytes(), "jp")
            .expect("parses")
            .export
    }

    fn loaded(text: &str, player: &str) -> PriorityList {
        PriorityList::for_slot(&export(text), &Slot::Player(player.into()))
    }

    fn press(player: &str) -> Press<'_> {
        Press {
            side: Side::Home,
            player,
            skip: None,
            after_warcry: false,
        }
    }

    /// A pick that must not be called: the press is not a random one.
    fn no_pick(_: usize) -> usize {
        panic!("a random pick was asked for")
    }

    fn file(list: &PriorityList, index: Option<usize>) -> Option<String> {
        index.map(|i| list.entries()[i].entry.file.clone())
    }

    /// Selects for one press and names the chosen file.
    fn play(
        list: &mut PriorityList,
        state: &MatchState,
        press: Press<'_>,
        pick: &mut impl FnMut(usize) -> usize,
    ) -> Option<String> {
        let chosen = list.select(state, press, pick);
        file(list, chosen)
    }

    #[test]
    fn a_player_s_list_is_its_entries_then_the_default_goalhorn_s() {
        let text = "goal;main.mp3\nBilly;b2.mp3;goals >= 2\nBilly;b1.mp3\n";
        let list = loaded(text, "Billy");
        let files: Vec<(&str, bool)> = list
            .entries()
            .iter()
            .map(|loaded| (loaded.entry.file.as_str(), loaded.fallback))
            .collect();
        assert_eq!(
            files,
            [("b2.mp3", false), ("b1.mp3", false), ("main.mp3", true)]
        );
        let goal = PriorityList::for_slot(&export(text), &Slot::Goal);
        assert_eq!(goal.entries().len(), 1, "the goal slot is its own list");
        let anthem = PriorityList::for_slot(&export(text), &Slot::Anthem);
        assert!(
            anthem.entries().is_empty(),
            "no fallback for reserved slots"
        );
        let unknown = list_of_unknown_player(text);
        assert_eq!(
            unknown.entries().len(),
            1,
            "an unlisted player gets the fallback alone"
        );
    }

    fn list_of_unknown_player(text: &str) -> PriorityList {
        loaded(text, "Nobody")
    }

    #[test]
    fn the_first_passing_entry_plays_top_down() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded(
            "goal;main.mp3\nBilly;b2.mp3;goals >= 2\nBilly;b1.mp3\n",
            "Billy",
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("b1.mp3".to_string())
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("b2.mp3".to_string())
        );
        let mut only_conditional = loaded(
            "goal;main.mp3;goals >= 5\nBilly;b2.mp3;goals >= 2\n",
            "Billy",
        );
        let mut fresh = MatchState::new("jp", "v", MatchType::Group);
        fresh.score_goal(Side::Home, "Billy");
        assert_eq!(
            only_conditional.select(&fresh, press("Billy"), &mut no_pick),
            None,
            "nothing passes: no song found, no panic"
        );
    }

    #[test]
    fn a_fallback_entry_counts_the_goal_button_s_own_presses() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded("goal;brace.mp3;goals == 2\ngoal;main.mp3\n", "Billy");
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("main.mp3".to_string()),
            "Billy's two goals are not the goal button's"
        );
        state.score_goal(Side::Home, "goal");
        state.score_goal(Side::Home, "goal");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("brace.mp3".to_string())
        );
    }

    #[test]
    fn once_plays_on_its_first_check_and_unloads_on_the_next() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded("goal;main.mp3\nBilly;first.mp3;once\n", "Billy");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("first.mp3".to_string())
        );
        assert_eq!(
            list.entries().len(),
            2,
            "still loaded after the first check"
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("main.mp3".to_string())
        );
        assert_eq!(list.entries().len(), 1, "unloaded at the second check");
        // `once` checked but failing on another condition is still consumed.
        let mut list = loaded("goal;main.mp3\nBilly;late.mp3;once;goals >= 2\n", "Billy");
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("main.mp3".to_string())
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("main.mp3".to_string()),
            "its one check was the first press"
        );
        assert_eq!(list.entries().len(), 1);
    }

    #[test]
    fn a_passed_time_unloads_the_entry() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded("goal;main.mp3\nBilly;early.mp3;time < 45\n", "Billy");
        assert!(list.needs_goal_minute());
        state.score_goal(Side::Home, "Billy");
        state.goal_minute = Some(70);
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("main.mp3".to_string())
        );
        assert_eq!(list.entries().len(), 1);
        assert!(!list.needs_goal_minute());
    }

    #[test]
    fn randomise_draws_from_the_player_s_own_entries_when_all_carry_it() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let text = "goal;main.mp3\nTakumi;t1.mp3;randomise;goals >= 9\nTakumi;t2.mp3;randomise\n\
                    Takumi;cry.mp3;warcry\n";
        let mut list = loaded(text, "Takumi");
        let mut asked = Vec::new();
        let mut pick = |n: usize| {
            asked.push(n);
            n - 1
        };
        let chosen = list.select(
            &state,
            Press {
                after_warcry: true,
                ..press("Takumi")
            },
            &mut pick,
        );
        assert_eq!(file(&list, chosen), Some("t2.mp3".to_string()));
        assert_eq!(asked, [2], "the pool is the two non-warcry own entries");
        // One own entry without `randomise`: priority order instead.
        let mut mixed = loaded(
            "goal;main.mp3\nTakumi;t1.mp3;randomise;goals >= 9\nTakumi;t2.mp3\n",
            "Takumi",
        );
        assert_eq!(
            play(&mut mixed, &state, press("Takumi"), &mut no_pick),
            Some("t2.mp3".to_string())
        );
        // Only the fallback: `randomise` on the goal slot is no pool for Takumi.
        let mut fallback_only = loaded("goal;g1.mp3;randomise\ngoal;g2.mp3;randomise\n", "Takumi");
        assert_eq!(
            play(&mut fallback_only, &state, press("Takumi"), &mut no_pick),
            Some("g1.mp3".to_string())
        );
    }

    #[test]
    fn a_warcry_plays_first_then_the_next_non_warcry_entry() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded(
            "goal;main.mp3\nBilly;cry.mp3;warcry\nBilly;b.mp3\n",
            "Billy",
        );
        assert_eq!(
            play(&mut list, &state, press("Billy"), &mut no_pick),
            Some("cry.mp3".to_string())
        );
        let after = Press {
            after_warcry: true,
            ..press("Billy")
        };
        assert_eq!(
            play(&mut list, &state, after, &mut no_pick),
            Some("b.mp3".to_string())
        );
        // Several warcries all randomised: one is drawn.
        let mut cries = loaded(
            "goal;main.mp3\nBilly;c1.mp3;warcry;randomise\nBilly;c2.mp3;warcry;randomise\nBilly;b.mp3\n",
            "Billy",
        );
        let mut last = |n: usize| n - 1;
        assert_eq!(
            play(&mut cries, &state, press("Billy"), &mut last),
            Some("c2.mp3".to_string())
        );
        assert_eq!(
            play(&mut cries, &state, after, &mut no_pick),
            Some("b.mp3".to_string())
        );
    }

    #[test]
    fn advance_skips_the_entry_that_just_ended() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded(
            "goal;main.mp3\nBilly;b1.mp3;advance\nBilly;b2.mp3\n",
            "Billy",
        );
        let first = list.select(&state, press("Billy"), &mut no_pick);
        assert_eq!(file(&list, first), Some("b1.mp3".to_string()));
        let next = Press {
            skip: first,
            ..press("Billy")
        };
        assert_eq!(
            play(&mut list, &state, next, &mut no_pick),
            Some("b2.mp3".to_string())
        );
        let alone = Press {
            skip: Some(0),
            ..press("Billy")
        };
        let mut one = loaded("Billy;b1.mp3;advance\n", "Billy");
        assert_eq!(one.select(&state, alone, &mut no_pick), None);
    }

    #[test]
    fn an_advance_skip_follows_the_entry_when_an_earlier_one_unloads() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut list = loaded(
            "goal;main.mp3\nBilly;o.mp3;once;goals >= 5\nBilly;b1.mp3;advance\nBilly;b2.mp3\n",
            "Billy",
        );
        state.score_goal(Side::Home, "Billy");
        let first = list.select(&state, press("Billy"), &mut no_pick);
        assert_eq!(file(&list, first), Some("b1.mp3".to_string()));
        // The `once` entry checked False on the first press and unloads on
        // the way to the skipped index.
        let next = Press {
            skip: first,
            ..press("Billy")
        };
        assert_eq!(
            play(&mut list, &state, next, &mut no_pick),
            Some("b2.mp3".to_string()),
            "skip is the index the press began with, not the shifted one"
        );
    }

    #[test]
    fn the_match_state_reaches_every_condition_through_the_press() {
        let mut state = MatchState::new("jp", "v", MatchType::Final);
        let text =
            "goal;main.mp3\nBilly;away.mp3;not home\nBilly;final.mp3;match final;opponent V\n";
        let mut home = loaded(text, "Billy");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut home, &state, press("Billy"), &mut no_pick),
            Some("final.mp3".to_string())
        );
        let mut away = loaded(text, "Billy");
        let press = Press {
            side: Side::Away,
            ..press("Billy")
        };
        assert_eq!(
            play(&mut away, &state, press, &mut no_pick),
            Some("away.mp3".to_string())
        );
        assert_eq!(
            Condition::Goals(Comparison::Eq, 1).eval(&state, Side::Home, "Billy"),
            Verdict::True
        );
    }
}
