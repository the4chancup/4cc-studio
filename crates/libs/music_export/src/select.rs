//! Song selection for one button press: a team's lists walked top-down — the
//! slot's own entries, then the `goal` slot's as every player's fallback —
//! with unloads, the randomise pools, warcries and `advance` skips as the
//! plan's "Song selection" states. Pure: the caller supplies the random pick.

use indexmap::IndexMap;

use crate::eval::Verdict;
use crate::model::{Instruction, MusicExport, Slot, SongEntry};
use crate::state::{MatchState, Side};

/// One entry of a list with what the match has done to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    /// The entry.
    pub entry: SongEntry,
    /// A `once` the walk already reached: the next reach unloads the entry.
    once_spent: bool,
}

/// One team's button lists: every slot's entries, shrinking as entries
/// unload, with the `goal` slot's stored once — it is the `goal` button's
/// own list and every player's fallback (`rigparse` appends the same
/// entries to every player's list), so an unload or a spent `once` through
/// one button holds for every button.
#[derive(Debug, Clone, PartialEq)]
pub struct TeamLists {
    /// Every slot's entries but the `goal` slot's, in the export's order.
    lists: IndexMap<Slot, Vec<Loaded>>,
    /// The `goal` slot's entries, the one fallback list per team.
    goal: Vec<Loaded>,
}

/// One press of a button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Press {
    /// The team the button belongs to.
    pub side: Side,
    /// The entry that just ended (`advance`), skipped this time: its index
    /// into `entries(slot)` as the list stood when the press began.
    pub skip: Option<usize>,
    /// A warcry already played for this press: warcries are excluded now.
    pub after_warcry: bool,
}

impl TeamLists {
    /// The lists a team's buttons walk, built from the export's slots.
    pub fn load(export: &MusicExport) -> TeamLists {
        let mut lists = IndexMap::new();
        let mut goal = Vec::new();
        for (slot, entries) in &export.slots {
            let loaded: Vec<Loaded> = entries.iter().map(load).collect();
            match slot {
                Slot::Goal => goal = loaded,
                Slot::Anthem | Slot::Victory | Slot::Chant | Slot::Player(_) => {
                    lists.insert(slot.clone(), loaded);
                }
            }
        }
        TeamLists { lists, goal }
    }

    /// The button's list: the slot's own entries, then — for a player's
    /// button — the `goal` slot's (reserved slots and `goal` itself get no
    /// fallback).
    pub fn entries(&self, slot: &Slot) -> Vec<&Loaded> {
        let (own, fallback) = self.parts(slot);
        own.iter().chain(fallback.iter()).collect()
    }

    /// `slot`'s own entries and its fallback's (the `goal` list for a
    /// player's button, empty otherwise).
    fn parts(&self, slot: &Slot) -> (&[Loaded], &[Loaded]) {
        let own: &[Loaded] = match slot {
            Slot::Goal => &self.goal,
            Slot::Anthem | Slot::Victory | Slot::Chant | Slot::Player(_) => {
                self.lists.get(slot).map(Vec::as_slice).unwrap_or(&[])
            }
        };
        let fallback: &[Loaded] = match slot {
            Slot::Player(_) => &self.goal,
            Slot::Anthem | Slot::Victory | Slot::Chant | Slot::Goal => &[],
        };
        (own, fallback)
    }

    /// Whether the press's walk would reach a `time` condition while the
    /// goal minute is unknown — then the view prompts, sets
    /// [`MatchState::goal_minute`] and calls [`select`](Self::select).
    ///
    /// The same walk as `select`, run on copies of the lists so its unloads
    /// and spent `once`s vanish: the walk up to the first `time` is the same
    /// with or without the minute, so the two agree.
    pub fn needs_goal_minute(&self, slot: &Slot, state: &MatchState, press: Press) -> bool {
        if state.goal_minute.is_some() {
            return false;
        }
        let (own, fallback) = self.parts(slot);
        let (mut own, mut fallback) = (own.to_vec(), fallback.to_vec());
        walk(&mut own, &mut fallback, slot.keyword(), state, press).1
    }

    /// Picks the entry to play, as an index into [`entries`](Self::entries)
    /// as it stands after this press's unloads, or `None` when nothing
    /// passes ("no song found").
    ///
    /// Top-down: the skipped entry is passed over, an entry that can never
    /// play again (a passed `time`, a `once` already spent) is unloaded, and
    /// the first passing entry plays. Before any check, a `randomise`
    /// warcry among only-`randomise` warcries is drawn; reaching a
    /// non-warcry `randomise` entry whose group's non-warcry entries all
    /// carry it draws among that group, conditions unread — the group is the
    /// entry's own list, so a reached fallback entry draws among the `goal`
    /// entries. The skipped entry counts for the checks and is not drawn.
    /// With `press.skip` set and nothing passing, the first non-warcry entry
    /// other than the skipped one plays, conditions unread; with none, the
    /// skipped one again.
    pub fn select(
        &mut self,
        slot: &Slot,
        state: &MatchState,
        press: Press,
        pick: &mut impl FnMut(usize) -> usize,
    ) -> Option<usize> {
        let mut spare_own = Vec::new();
        let mut spare_fallback = Vec::new();
        let (own, fallback): (&mut Vec<Loaded>, &mut Vec<Loaded>) = match slot {
            Slot::Goal => (&mut self.goal, &mut spare_fallback),
            Slot::Player(_) => (
                self.lists.get_mut(slot).unwrap_or(&mut spare_own),
                &mut self.goal,
            ),
            Slot::Anthem | Slot::Victory | Slot::Chant => (
                self.lists.get_mut(slot).unwrap_or(&mut spare_own),
                &mut spare_fallback,
            ),
        };
        match walk(own, fallback, slot.keyword(), state, press).0 {
            Walk::Play(index) => Some(index),
            Walk::Draw(pool) => Some(pool[pick(pool.len())]),
            Walk::Nothing => press.skip.map(|skip| last_resort(own, fallback, skip)),
        }
    }
}

/// What the top-down walk over a button's list found.
enum Walk {
    /// The entry at this index plays (an index into the button's list as it
    /// stands after the press's unloads).
    Play(usize),
    /// The press ends in a random draw among these indices.
    Draw(Vec<usize>),
    /// Nothing played.
    Nothing,
}

/// The top-down walk of `getSong`: `own`, then the shared `goal` list as the
/// player's fallback; also returns whether it reached a `time` condition —
/// `needs_goal_minute`'s whole question (its copies make the walk's unloads
/// and spent `once`s vanish; `select` ignores it).
fn walk(
    own: &mut Vec<Loaded>,
    fallback: &mut Vec<Loaded>,
    player: &str,
    state: &MatchState,
    press: Press,
) -> (Walk, bool) {
    // Warcries play first; when every warcry in the button's list (the
    // fallback's included, the skipped one excepted) carries `randomise`,
    // one is drawn.
    if !press.after_warcry {
        let warcries: Vec<usize> = (0..own.len() + fallback.len())
            .filter(|&i| Some(i) != press.skip && at(own, fallback, own.len(), i).entry.is_warcry())
            .collect();
        if !warcries.is_empty()
            && warcries
                .iter()
                .all(|&i| at(own, fallback, own.len(), i).entry.is_randomise())
        {
            return (Walk::Draw(warcries), false);
        }
    }
    let mut skip = press.skip;
    let mut own_len = own.len();
    let mut reached_minute = false;
    let mut i = 0;
    while i < own_len + fallback.len() {
        if Some(i) == skip {
            i += 1;
            continue;
        }
        let in_own = i < own_len;
        let check = {
            let (list, index) = if in_own {
                (&mut *own, i)
            } else {
                (&mut *fallback, i - own_len)
            };
            let loaded = &mut list[index];
            // A fallback entry evaluates under the name `goal`, so its
            // `goals` counts the goalhorn button's own presses.
            let name = if in_own { player } else { Slot::Goal.keyword() };
            let check = loaded
                .entry
                .check(state, press.side, name, loaded.once_spent);
            loaded.once_spent |= check.once;
            if check.verdict == Verdict::NeverAgain {
                list.remove(index);
                if in_own {
                    own_len -= 1;
                }
                // An entry unloaded before the skipped one moves its index
                // down with it.
                if skip.is_some_and(|skip| i < skip) {
                    skip = skip.map(|skip| skip - 1);
                }
            }
            check
        };
        // A reached `time` prompts inside the check, whatever the verdict
        // after it (`TimeCondition.check` asks before the entry's outcome is
        // decided — even one that unloads the entry).
        reached_minute |= check.minute;
        if check.verdict == Verdict::NeverAgain {
            continue;
        }
        let loaded = at(own, fallback, own_len, i);
        let (randomise, warcry) = (loaded.entry.is_randomise(), loaded.entry.is_warcry());
        // A reached `randomise` entry draws among its own group's non-warcry
        // entries when all carry it; the skipped entry counts for the check
        // and is not drawn.
        if randomise && !warcry {
            let (list, start) = if in_own {
                (own.as_slice(), 0)
            } else {
                (fallback.as_slice(), own_len)
            };
            if list
                .iter()
                .filter(|entry| !entry.entry.is_warcry())
                .all(|entry| entry.entry.is_randomise())
            {
                let draw: Vec<usize> = (start..start + list.len())
                    .filter(|&j| {
                        Some(j) != skip && !at(own, fallback, own_len, j).entry.is_warcry()
                    })
                    .collect();
                return (Walk::Draw(draw), reached_minute);
            }
        }
        if check.verdict == Verdict::True && !(press.after_warcry && warcry) {
            return (Walk::Play(i), reached_minute);
        }
        i += 1;
    }
    (Walk::Nothing, reached_minute)
}

/// `advance`'s last resort: the first non-warcry entry other than the one
/// that ended, conditions unread (`getSong`'s fallback); with none, that
/// entry itself.
fn last_resort(own: &[Loaded], fallback: &[Loaded], skip: usize) -> usize {
    for i in 0..own.len() + fallback.len() {
        if i != skip && !at(own, fallback, own.len(), i).entry.is_warcry() {
            return i;
        }
    }
    skip
}

/// The combined-list entry at index `i`: `own_len` and under is `own`,
/// beyond it the fallback list.
fn at<'a>(own: &'a [Loaded], fallback: &'a [Loaded], own_len: usize, i: usize) -> &'a Loaded {
    if i < own_len {
        &own[i]
    } else {
        &fallback[i - own_len]
    }
}

fn load(entry: &SongEntry) -> Loaded {
    Loaded {
        entry: entry.clone(),
        once_spent: false,
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

    fn team_lists(text: &str) -> TeamLists {
        TeamLists::load(&export(text))
    }

    fn player(name: &str) -> Slot {
        Slot::Player(name.into())
    }

    fn press() -> Press {
        Press {
            side: Side::Home,
            skip: None,
            after_warcry: false,
        }
    }

    /// A pick that must not be called: the press is not a random one.
    fn no_pick(_: usize) -> usize {
        panic!("a random pick was asked for")
    }

    fn file(lists: &TeamLists, slot: &Slot, index: Option<usize>) -> Option<String> {
        index.map(|i| lists.entries(slot)[i].entry.file.clone())
    }

    /// Selects for one press of `slot`'s button and names the chosen file.
    fn play(
        lists: &mut TeamLists,
        slot: &Slot,
        state: &MatchState,
        press: Press,
        pick: &mut impl FnMut(usize) -> usize,
    ) -> Option<String> {
        let chosen = lists.select(slot, state, press, pick);
        file(lists, slot, chosen)
    }

    #[test]
    fn a_player_s_list_is_its_entries_then_the_default_goalhorn_s() {
        let lists = team_lists("goal;main.mp3\nBilly;b2.mp3;goals >= 2\nBilly;b1.mp3\n");
        let files: Vec<&str> = lists
            .entries(&player("Billy"))
            .iter()
            .map(|loaded| loaded.entry.file.as_str())
            .collect();
        assert_eq!(files, ["b2.mp3", "b1.mp3", "main.mp3"]);
        assert_eq!(
            lists.entries(&Slot::Goal).len(),
            1,
            "the goal slot is its own list"
        );
        assert!(
            lists.entries(&Slot::Anthem).is_empty(),
            "no fallback for reserved slots"
        );
        assert_eq!(
            lists.entries(&player("Nobody")).len(),
            1,
            "an unlisted player gets the fallback alone"
        );
    }

    #[test]
    fn the_first_passing_entry_plays_top_down() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("goal;main.mp3\nBilly;b2.mp3;goals >= 2\nBilly;b1.mp3\n");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("b1.mp3".to_string())
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("b2.mp3".to_string())
        );
        let mut only_conditional =
            team_lists("goal;main.mp3;goals >= 5\nBilly;b2.mp3;goals >= 2\n");
        let mut fresh = MatchState::new("jp", "v", MatchType::Group);
        fresh.score_goal(Side::Home, "Billy");
        assert_eq!(
            only_conditional.select(&billy, &fresh, press(), &mut no_pick),
            None,
            "nothing passes: no song found, no panic"
        );
    }

    #[test]
    fn a_fallback_entry_counts_the_goal_button_s_own_presses() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("goal;brace.mp3;goals == 2\ngoal;main.mp3\n");
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("main.mp3".to_string()),
            "Billy's two goals are not the goal button's"
        );
        state.score_goal(Side::Home, "goal");
        state.score_goal(Side::Home, "goal");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("brace.mp3".to_string())
        );
    }

    /// `once` is spent the first time the check *reaches* it — every
    /// condition before it held — and unloads the entry on the next reach.
    #[test]
    fn once_is_spent_when_the_walk_reaches_it() {
        // `goals >= 2;once`: the first goal fails before `once` is reached;
        // the second reaches it and plays; the third finds it spent.
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("goal;main.mp3\nBilly;o.mp3;goals >= 2;once\nBilly;b.mp3\n");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("b.mp3".to_string())
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("o.mp3".to_string())
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("b.mp3".to_string()),
            "the spent `once` unloads on the next reach"
        );
        // `once;goals >= 2`: the first press reaches the `once` — spent even
        // though the entry fails — so it never plays.
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut lists = team_lists("goal;main.mp3\nBilly;o.mp3;once;goals >= 2\nBilly;b.mp3\n");
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("b.mp3".to_string())
        );
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("b.mp3".to_string()),
            "its one reach was the first press"
        );
        assert_eq!(lists.entries(&billy).len(), 2, "the entry unloaded");
    }

    /// The fallback is one list for the team, not a copy per button: a
    /// `once` spent or an unload through one player's button holds for every
    /// button, the `goal` button's included.
    #[test]
    fn the_fallback_is_one_list_for_every_button() {
        let mut lists = team_lists("goal;g.mp3;once\nA;a.mp3;goals >= 9\nB;b.mp3;goals >= 9\n");
        let state = MatchState::new("jp", "v", MatchType::Group);
        assert_eq!(
            play(&mut lists, &player("A"), &state, press(), &mut no_pick),
            Some("g.mp3".to_string())
        );
        assert_eq!(
            lists.select(&player("B"), &state, press(), &mut no_pick),
            None,
            "spent through A's button: B's reach unloads it"
        );
        assert_eq!(
            lists.select(&Slot::Goal, &state, press(), &mut no_pick),
            None,
            "gone for the `goal` button too"
        );
    }

    #[test]
    fn a_fallback_unload_through_one_button_shrinks_every_list() {
        // A fallback `time < 30` at minute 40 unloads through A's press and
        // is gone from B's list and the `goal` slot's.
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let mut lists = team_lists("goal;g.mp3;time < 30\ngoal;main.mp3\nA;a.mp3;goals >= 9\n");
        state.goal_minute = Some(40);
        assert_eq!(
            play(&mut lists, &player("A"), &state, press(), &mut no_pick),
            Some("main.mp3".to_string())
        );
        let files = |slot: &Slot| -> Vec<String> {
            lists
                .entries(slot)
                .iter()
                .map(|loaded| loaded.entry.file.clone())
                .collect()
        };
        assert_eq!(files(&Slot::Goal), ["main.mp3"]);
        assert_eq!(
            files(&player("B")),
            ["main.mp3"],
            "B's button sees the same shrunken fallback"
        );
    }

    #[test]
    fn a_passed_time_unloads_the_entry() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("goal;main.mp3\nBilly;early.mp3;time < 45\n");
        assert!(lists.needs_goal_minute(&billy, &state, press()));
        state.score_goal(Side::Home, "Billy");
        state.goal_minute = Some(70);
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("main.mp3".to_string())
        );
        assert_eq!(lists.entries(&billy).len(), 1);
        state.goal_minute = None;
        assert!(!lists.needs_goal_minute(&billy, &state, press()));
    }

    /// The prompt is asked only when the walk reaches a `time`.
    #[test]
    fn needs_goal_minute_is_the_walk_up_to_the_first_time() {
        let billy = player("Billy");
        // `goals >= 2;time < 30`: on the first goal the `time` is not reached.
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let lists = team_lists("goal;main.mp3\nBilly;b.mp3;goals >= 2;time < 30\n");
        state.score_goal(Side::Home, "Billy");
        assert!(!lists.needs_goal_minute(&billy, &state, press()));
        state.score_goal(Side::Home, "Billy");
        assert!(lists.needs_goal_minute(&billy, &state, press()));
        state.goal_minute = Some(40);
        assert!(
            !lists.needs_goal_minute(&billy, &state, press()),
            "the minute is already known"
        );
        // A line that plays before a `time` line ends the walk first.
        let lists = team_lists("Billy;b.mp3\nBilly;late.mp3;time < 30\n");
        let state = MatchState::new("jp", "v", MatchType::Group);
        assert!(!lists.needs_goal_minute(&billy, &state, press()));
        // A random draw reached before any `time` ends the walk too.
        let lists =
            team_lists("Billy;r1.mp3;randomise\nBilly;r2.mp3;randomise\ngoal;g.mp3;time < 30\n");
        assert!(!lists.needs_goal_minute(&billy, &state, press()));
    }

    #[test]
    fn randomise_draws_among_the_reached_entry_s_own_group() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let takumi = player("Takumi");
        let text = "goal;main.mp3\nTakumi;t1.mp3;randomise;goals >= 9\nTakumi;t2.mp3;randomise\n\
                    Takumi;cry.mp3;warcry\n";
        let mut lists = team_lists(text);
        let mut asked = Vec::new();
        let mut pick = |n: usize| {
            asked.push(n);
            n - 1
        };
        let chosen = lists.select(
            &takumi,
            &state,
            Press {
                after_warcry: true,
                ..press()
            },
            &mut pick,
        );
        assert_eq!(file(&lists, &takumi, chosen), Some("t2.mp3".to_string()));
        assert_eq!(asked, [2], "the pool is the two non-warcry own entries");
        // One own entry without `randomise`: priority order instead.
        let mut mixed =
            team_lists("goal;main.mp3\nTakumi;t1.mp3;randomise;goals >= 9\nTakumi;t2.mp3\n");
        assert_eq!(
            play(&mut mixed, &takumi, &state, press(), &mut no_pick),
            Some("t2.mp3".to_string())
        );
        // Reached on a fallback entry, the pool is the `goal` slot's own
        // entries; the player's are not in it.
        let mut lists =
            team_lists("goal;g1.mp3;randomise\ngoal;g2.mp3;randomise\nTakumi;t.mp3;goals >= 9\n");
        let mut asked_fallback = Vec::new();
        let mut pick = |n: usize| {
            asked_fallback.push(n);
            0
        };
        let chosen = lists.select(&takumi, &state, press(), &mut pick);
        assert_eq!(file(&lists, &takumi, chosen), Some("g1.mp3".to_string()));
        assert_eq!(
            asked_fallback,
            [2],
            "the draw was among the two `goal` lines"
        );
    }

    #[test]
    fn a_warcry_plays_first_then_the_next_non_warcry_entry() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("goal;main.mp3\nBilly;cry.mp3;warcry\nBilly;b.mp3\n");
        assert_eq!(
            play(&mut lists, &billy, &state, press(), &mut no_pick),
            Some("cry.mp3".to_string())
        );
        let after = Press {
            after_warcry: true,
            ..press()
        };
        assert_eq!(
            play(&mut lists, &billy, &state, after, &mut no_pick),
            Some("b.mp3".to_string())
        );
        // Several warcries all randomised: one is drawn.
        let mut cries = team_lists(
            "goal;main.mp3\nBilly;c1.mp3;warcry;randomise\nBilly;c2.mp3;warcry;randomise\nBilly;b.mp3\n",
        );
        let mut last = |n: usize| n - 1;
        assert_eq!(
            play(&mut cries, &billy, &state, press(), &mut last),
            Some("c2.mp3".to_string())
        );
        assert_eq!(
            play(&mut cries, &billy, &state, after, &mut no_pick),
            Some("b.mp3".to_string())
        );
        // The fallback's warcries are the same pool: an own and a `goal`
        // `warcry;randomise` draw among two.
        let mut cries = team_lists(
            "goal;c.mp3;warcry;randomise\nBilly;b.mp3;warcry;randomise\nBilly;rest.mp3\n",
        );
        let mut asked = Vec::new();
        let mut pick = |n: usize| {
            asked.push(n);
            0
        };
        assert_eq!(
            play(&mut cries, &billy, &state, press(), &mut pick),
            Some("b.mp3".to_string())
        );
        assert_eq!(asked, [2], "own and fallback warcries are the one pool");
    }

    #[test]
    fn advance_skips_the_entry_that_just_ended() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("goal;main.mp3\nBilly;b1.mp3;advance\nBilly;b2.mp3\n");
        let first = lists.select(&billy, &state, press(), &mut no_pick);
        assert_eq!(file(&lists, &billy, first), Some("b1.mp3".to_string()));
        let next = Press {
            skip: first,
            ..press()
        };
        assert_eq!(
            play(&mut lists, &billy, &state, next, &mut no_pick),
            Some("b2.mp3".to_string())
        );
    }

    /// `advance` with nothing passing plays the first other non-warcry
    /// entry, conditions unread; with none, the one that ended plays again.
    #[test]
    fn advance_s_last_resort_reads_no_conditions() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists("Billy;b1.mp3;advance\nBilly;b2.mp3;goals >= 9\n");
        let first = lists.select(&billy, &state, press(), &mut no_pick);
        assert_eq!(file(&lists, &billy, first), Some("b1.mp3".to_string()));
        let next = Press {
            skip: first,
            ..press()
        };
        assert_eq!(
            play(&mut lists, &billy, &state, next, &mut no_pick),
            Some("b2.mp3".to_string()),
            "the first non-warcry entry other than the one that ended"
        );
        // Only the ended one and a warcry: it plays again.
        let mut lists = team_lists("Billy;b1.mp3;advance\nBilly;c.mp3;warcry\n");
        let next = Press {
            skip: Some(0),
            after_warcry: true,
            ..press()
        };
        assert_eq!(
            play(&mut lists, &billy, &state, next, &mut no_pick),
            Some("b1.mp3".to_string()),
            "no other non-warcry entry: the skipped one plays again"
        );
    }

    #[test]
    fn randomise_does_not_draw_the_entry_advance_skips() {
        let state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists(
            "goal;main.mp3\nBilly;b1.mp3;randomise;advance\nBilly;b2.mp3;randomise;advance\n",
        );
        let mut asked = Vec::new();
        let mut first_of = |n: usize| {
            asked.push(n);
            0
        };
        let first = lists.select(&billy, &state, press(), &mut first_of);
        assert_eq!(file(&lists, &billy, first), Some("b1.mp3".to_string()));
        let next = Press {
            skip: first,
            ..press()
        };
        assert_eq!(
            play(&mut lists, &billy, &state, next, &mut first_of),
            Some("b2.mp3".to_string()),
            "the entry that just ended is not drawn again"
        );
        assert_eq!(asked, [2, 1], "it still counts for the all-entries check");
    }

    #[test]
    fn an_advance_skip_follows_the_entry_when_an_earlier_one_unloads() {
        let mut state = MatchState::new("jp", "v", MatchType::Group);
        let billy = player("Billy");
        let mut lists = team_lists(
            "goal;main.mp3\nBilly;o.mp3;once;goals >= 5\nBilly;b1.mp3;advance\nBilly;b2.mp3\n",
        );
        state.score_goal(Side::Home, "Billy");
        let first = lists.select(&billy, &state, press(), &mut no_pick);
        assert_eq!(file(&lists, &billy, first), Some("b1.mp3".to_string()));
        // The `once` entry spent on the first press unloads on the way to
        // the skipped index.
        let next = Press {
            skip: first,
            ..press()
        };
        assert_eq!(
            play(&mut lists, &billy, &state, next, &mut no_pick),
            Some("b2.mp3".to_string()),
            "skip is the index the press began with, not the shifted one"
        );
    }

    #[test]
    fn the_match_state_reaches_every_condition_through_the_press() {
        let mut state = MatchState::new("jp", "v", MatchType::Final);
        let billy = player("Billy");
        let text =
            "goal;main.mp3\nBilly;away.mp3;not home\nBilly;final.mp3;match final;opponent V\n";
        let mut home = team_lists(text);
        state.score_goal(Side::Home, "Billy");
        assert_eq!(
            play(&mut home, &billy, &state, press(), &mut no_pick),
            Some("final.mp3".to_string())
        );
        let mut away = team_lists(text);
        let press = Press {
            side: Side::Away,
            ..press()
        };
        assert_eq!(
            play(&mut away, &billy, &state, press, &mut no_pick),
            Some("away.mp3".to_string())
        );
        assert_eq!(
            Condition::Goals(Comparison::Eq, 1).eval(&state, Side::Home, "Billy"),
            Verdict::True
        );
    }
}
