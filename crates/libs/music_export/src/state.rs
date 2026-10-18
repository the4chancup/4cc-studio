//! The match as the soundboard sees it: two teams, their scores and scorers,
//! the match type and the minute of the goal being scored. Plain data owned by
//! the GUI thread; the conditions read it, the view and the Match tracker
//! write it.

use crate::model::MatchType;

/// The slot a team occupies in the match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The home slot.
    Home,
    /// The away slot.
    Away,
}

impl Side {
    /// The other slot.
    pub fn opponent(self) -> Side {
        match self {
            Side::Home => Side::Away,
            Side::Away => Side::Home,
        }
    }
}

/// One team's name, score and scorers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TeamState {
    /// The team name as its export carries it (lowercase, no slashes).
    pub name: String,
    /// Goals scored.
    pub score: u32,
    /// Each scorer's goals, in order of first goal; a name is matched
    /// case-insensitively, the first spelling kept.
    pub scorers: Vec<(String, u32)>,
}

impl TeamState {
    /// A team with no goals.
    pub fn new(name: &str) -> TeamState {
        TeamState {
            name: name.to_string(),
            score: 0,
            scorers: Vec::new(),
        }
    }

    /// `player`'s goals, 0 when they have not scored.
    pub fn player_goals(&self, player: &str) -> u32 {
        self.scorer(player).map_or(0, |i| self.scorers[i].1)
    }

    /// Whether `player` has at least as many goals as every scorer on the team.
    pub(crate) fn has_most_goals(&self, player: &str) -> bool {
        let mine = self.player_goals(player);
        self.scorers.iter().all(|(_, goals)| *goals <= mine)
    }

    /// `player`'s index in `scorers`, matched case-insensitively.
    fn scorer(&self, player: &str) -> Option<usize> {
        self.scorers
            .iter()
            .position(|(name, _)| same_player(name, player))
    }

    fn score_goal(&mut self, player: &str) {
        self.score += 1;
        match self.scorer(player) {
            Some(i) => self.scorers[i].1 += 1,
            None => self.scorers.push((player.to_string(), 1)),
        }
    }

    /// Takes one goal back; false when the player has none to take.
    fn unscore_goal(&mut self, player: &str) -> bool {
        let Some(i) = self.scorer(player) else {
            return false;
        };
        let goals = &mut self.scorers[i].1;
        if *goals == 0 || self.score == 0 {
            return false;
        }
        *goals -= 1;
        self.score -= 1;
        true
    }

    /// Back to no goals, the name kept.
    pub(crate) fn clear(&mut self) {
        self.score = 0;
        self.scorers.clear();
    }
}

/// Player names match case-insensitively: the Match tracker's scorer names and
/// a file's slot names may differ in case alone.
fn same_player(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The state every condition is evaluated against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchState {
    /// The home team.
    pub home: TeamState,
    /// The away team.
    pub away: TeamState,
    /// The match type the streamer selected (or the cup stage, once known).
    pub match_type: MatchType,
    /// The minute of the goal being scored: prompted from the streamer or
    /// taken from the Match tracker's clock when a loaded entry needs it
    /// ([`crate::SongEntry::needs_goal_minute`]); `None` until then and after
    /// each play.
    pub goal_minute: Option<u32>,
}

impl MatchState {
    /// A match at 0-0 between the two named teams.
    pub fn new(home: &str, away: &str, match_type: MatchType) -> MatchState {
        MatchState {
            home: TeamState::new(home),
            away: TeamState::new(away),
            match_type,
            goal_minute: None,
        }
    }

    /// The team in `side`'s slot.
    pub fn team(&self, side: Side) -> &TeamState {
        match side {
            Side::Home => &self.home,
            Side::Away => &self.away,
        }
    }

    /// The team in the other slot.
    pub fn opponent(&self, side: Side) -> &TeamState {
        self.team(side.opponent())
    }

    fn team_mut(&mut self, side: Side) -> &mut TeamState {
        match side {
            Side::Home => &mut self.home,
            Side::Away => &mut self.away,
        }
    }

    /// Scores a goal for `player` on `side`'s team (the default goalhorn's
    /// presses score for its own slot name).
    pub fn score_goal(&mut self, side: Side, player: &str) {
        self.team_mut(side).score_goal(player);
    }

    /// Takes back `player`'s last goal on `side`'s team; false when there is
    /// none to take.
    pub fn unscore_goal(&mut self, side: Side, player: &str) -> bool {
        self.team_mut(side).unscore_goal(player)
    }

    /// `side`'s goal difference, us minus them.
    pub(crate) fn lead(&self, side: Side) -> i64 {
        i64::from(self.team(side).score) - i64::from(self.opponent(side).score)
    }

    /// Resets one team's score and scorers (the per-team Reset button).
    pub fn clear_team(&mut self, side: Side) {
        self.team_mut(side).clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> MatchState {
        MatchState::new("jp", "v", MatchType::Group)
    }

    #[test]
    fn goals_count_per_team_and_per_player_case_insensitively() {
        let mut state = state();
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Home, "billy");
        state.score_goal(Side::Away, "Dante");
        assert_eq!(state.home.score, 2);
        assert_eq!(state.away.score, 1);
        assert_eq!(state.home.scorers, [("Billy".to_string(), 2)]);
        assert_eq!(state.home.player_goals("BILLY"), 2);
        assert_eq!(state.home.player_goals("Dante"), 0);
        assert_eq!(state.team(Side::Away).player_goals("Dante"), 1);
        assert_eq!(state.opponent(Side::Home).name, "v");
        assert_eq!(state.lead(Side::Home), 1);
        assert_eq!(state.lead(Side::Away), -1);
    }

    #[test]
    fn unscoring_takes_one_goal_back_and_refuses_at_zero() {
        let mut state = state();
        assert!(!state.unscore_goal(Side::Home, "Billy"));
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Home, "Billy");
        assert!(state.unscore_goal(Side::Home, "billy"));
        assert_eq!(state.home.score, 1);
        assert_eq!(state.home.player_goals("Billy"), 1);
        assert!(state.unscore_goal(Side::Home, "Billy"));
        assert!(!state.unscore_goal(Side::Home, "Billy"));
        assert_eq!(state.home.score, 0);
    }

    #[test]
    fn unscoring_one_scorer_keeps_anothers_goals() {
        let mut state = state();
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Home, "Bob");
        assert!(state.unscore_goal(Side::Home, "Billy"));
        assert!(!state.unscore_goal(Side::Home, "Billy"));
        assert_eq!(state.home.score, 1);
        assert_eq!(state.home.player_goals("Bob"), 1);
    }

    #[test]
    fn most_goals_counts_ties_and_an_unlisted_player_at_zero() {
        let mut state = state();
        assert!(state.home.has_most_goals("Billy"), "0-0: nobody is ahead");
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Home, "Get Out");
        assert!(state.home.has_most_goals("Billy"));
        assert!(state.home.has_most_goals("Get Out"));
        assert!(!state.home.has_most_goals("Dekinai-chan"));
        state.score_goal(Side::Home, "Get Out");
        assert!(!state.home.has_most_goals("Billy"));
    }

    #[test]
    fn clearing_a_team_keeps_its_name_and_the_other_team() {
        let mut state = state();
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Away, "Dante");
        state.clear_team(Side::Home);
        assert_eq!(state.home, TeamState::new("jp"));
        assert_eq!(state.away.score, 1);
        assert_eq!(Side::Home.opponent(), Side::Away);
        assert_eq!(Side::Away.opponent(), Side::Home);
    }
}
