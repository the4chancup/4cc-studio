//! Condition evaluation: pure functions of the match state, no prompts and no
//! exceptions; an entry that can never play again says so in its verdict.

use crate::model::{Comparison, Condition, SongEntry};
use crate::state::{MatchState, Side};

/// What a condition (or an entry's conditions) says about playing now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Holds now.
    True,
    /// Does not hold now; may later.
    False,
    /// Cannot hold again this match: the entry is unloaded from its slot.
    NeverAgain,
}

impl Condition {
    /// Evaluates for `player`'s entry on `side`, after the goal has been
    /// scored (the counts include it).
    ///
    /// `once` is `True` here every time: the slot's selection unloads a `once`
    /// entry after its first check, since a pure evaluation cannot remember
    /// one. `time` without a known [`MatchState::goal_minute`] is `False`; the
    /// view asks for the minute first ([`SongEntry::needs_goal_minute`]).
    /// `not` of a condition that can never hold again is `True`, not
    /// `NeverAgain`: it is the negation that holds from then on.
    pub fn eval(&self, state: &MatchState, side: Side, player: &str) -> Verdict {
        let team = state.team(side);
        let holds = |held: bool| if held { Verdict::True } else { Verdict::False };
        match self {
            Condition::Goals(comparison, n) => {
                holds(comparison.holds(i64::from(team.player_goals(player)), i64::from(*n)))
            }
            Condition::TeamGoals(comparison, n) => {
                holds(comparison.holds(i64::from(team.score), i64::from(*n)))
            }
            Condition::Lead(comparison, n) => {
                holds(comparison.holds(state.lead(side), i64::from(*n)))
            }
            Condition::Every(n) => holds(team.player_goals(player).is_multiple_of(*n)),
            Condition::Opponent(teams) => {
                let opponent = &state.opponent(side).name;
                holds(teams.iter().any(|team| team.eq_ignore_ascii_case(opponent)))
            }
            Condition::Match(types) => holds(types.contains(&state.match_type)),
            Condition::Home => holds(side == Side::Home),
            Condition::Once => Verdict::True,
            Condition::First => holds(team.score == 1),
            Condition::Comeback => {
                let opponent = state.opponent(side);
                holds(team.score <= opponent.score && opponent.score > 0)
            }
            Condition::MostGoals(named) => {
                holds(team.has_most_goals(named.as_deref().unwrap_or(player)))
            }
            Condition::Special(_) => Verdict::False,
            Condition::Time(comparison, minute) => match state.goal_minute {
                None => Verdict::False,
                Some(now)
                    if now > *minute
                        && matches!(
                            comparison,
                            Comparison::Lt | Comparison::Le | Comparison::Eq
                        ) =>
                {
                    Verdict::NeverAgain
                }
                Some(now) => holds(comparison.holds(i64::from(now), i64::from(*minute))),
            },
            Condition::Not(inner) => match inner.eval(state, side, player) {
                Verdict::True => Verdict::False,
                Verdict::False | Verdict::NeverAgain => Verdict::True,
            },
        }
    }

    /// Whether evaluating this condition reads the goal minute.
    pub fn needs_goal_minute(&self) -> bool {
        match self {
            Condition::Time(..) => true,
            Condition::Not(inner) => inner.needs_goal_minute(),
            Condition::Goals(..)
            | Condition::TeamGoals(..)
            | Condition::Lead(..)
            | Condition::Every(_)
            | Condition::Opponent(_)
            | Condition::Match(_)
            | Condition::Home
            | Condition::Once
            | Condition::First
            | Condition::Comeback
            | Condition::MostGoals(_)
            | Condition::Special(_) => false,
        }
    }
}

impl SongEntry {
    /// All conditions must hold (AND): `NeverAgain` when any condition can
    /// never hold again (the entry leaves the slot whatever the others say),
    /// else `False` when any fails, else `True`. An entry with no conditions
    /// always plays.
    pub fn eval(&self, state: &MatchState, side: Side, player: &str) -> Verdict {
        let mut verdict = Verdict::True;
        for condition in &self.conditions {
            match condition.eval(state, side, player) {
                Verdict::True => {}
                Verdict::False => verdict = Verdict::False,
                Verdict::NeverAgain => return Verdict::NeverAgain,
            }
        }
        verdict
    }

    /// Whether any condition reads the goal minute, so the view can ask for it
    /// before the evaluation.
    pub fn needs_goal_minute(&self) -> bool {
        self.conditions.iter().any(Condition::needs_goal_minute)
    }

    /// Whether the entry carries `once` and so leaves its slot after its first check.
    pub fn is_once(&self) -> bool {
        self.conditions
            .iter()
            .any(|condition| matches!(condition, Condition::Once))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Instruction, MatchType};
    use Verdict::*;

    /// jp 2-1 v in the final: Billy scored both for jp, Dante for v.
    fn state() -> MatchState {
        let mut state = MatchState::new("jp", "v", MatchType::Final);
        state.score_goal(Side::Home, "Billy");
        state.score_goal(Side::Away, "Dante");
        state.score_goal(Side::Home, "Billy");
        state
    }

    fn eval(condition: Condition) -> Verdict {
        condition.eval(&state(), Side::Home, "Billy")
    }

    #[test]
    fn counting_conditions_compare_the_right_count() {
        use Comparison::*;
        assert_eq!(eval(Condition::Goals(Eq, 2)), True);
        assert_eq!(eval(Condition::Goals(Ge, 3)), False);
        assert_eq!(
            Condition::Goals(Eq, 0).eval(&state(), Side::Home, "Get Out"),
            True,
            "a player who has not scored has 0"
        );
        assert_eq!(eval(Condition::TeamGoals(Eq, 2)), True);
        assert_eq!(eval(Condition::TeamGoals(Lt, 2)), False);
        assert_eq!(eval(Condition::Lead(Eq, 1)), True);
        assert_eq!(
            Condition::Lead(Eq, -1).eval(&state(), Side::Away, "Dante"),
            True
        );
        assert_eq!(eval(Condition::Every(2)), True);
        assert_eq!(eval(Condition::Every(3)), False);
        assert_eq!(
            Condition::Every(1).eval(&state(), Side::Home, "Dante"),
            True,
            "0 goals is divisible"
        );
    }

    #[test]
    fn opponent_and_match_read_the_other_team_and_the_stage() {
        assert_eq!(
            eval(Condition::Opponent(vec!["o".into(), "V".into()])),
            True,
            "case-insensitive"
        );
        assert_eq!(eval(Condition::Opponent(vec!["jp".into()])), False);
        assert_eq!(
            Condition::Opponent(vec!["JP".into()]).eval(&state(), Side::Away, "Dante"),
            True
        );
        assert_eq!(
            eval(Condition::Match(vec![MatchType::Group, MatchType::Final])),
            True
        );
        assert_eq!(eval(Condition::Match(vec![MatchType::Group])), False);
    }

    #[test]
    fn home_once_first_comeback_mostgoals_and_special() {
        assert_eq!(eval(Condition::Home), True);
        assert_eq!(Condition::Home.eval(&state(), Side::Away, "Dante"), False);
        assert_eq!(eval(Condition::Once), True);
        assert_eq!(eval(Condition::First), False, "jp has 2");
        assert_eq!(Condition::First.eval(&state(), Side::Away, "Dante"), True);
        assert_eq!(eval(Condition::Comeback), False, "jp leads");
        let mut level = state();
        level.score_goal(Side::Away, "Dante");
        assert_eq!(
            Condition::Comeback.eval(&level, Side::Away, "Dante"),
            True,
            "level after scoring, the opponent has scored"
        );
        let mut fresh = MatchState::new("jp", "v", MatchType::Group);
        assert_eq!(
            Condition::Comeback.eval(&fresh, Side::Home, "Billy"),
            False,
            "0-0 (a victory anthem or anthem evaluated without a goal): level, but nobody has scored"
        );
        fresh.score_goal(Side::Home, "Billy");
        assert_eq!(
            Condition::Comeback.eval(&fresh, Side::Home, "Billy"),
            False,
            "1-0 after the opener: the opponent has not scored"
        );
        fresh.score_goal(Side::Home, "Billy");
        fresh.score_goal(Side::Away, "Dante");
        assert_eq!(
            Condition::Comeback.eval(&fresh, Side::Away, "Dante"),
            True,
            "1-2 after pulling one back: still behind"
        );
        assert_eq!(eval(Condition::MostGoals(None)), True);
        assert_eq!(
            Condition::MostGoals(None).eval(&state(), Side::Home, "Get Out"),
            False
        );
        assert_eq!(
            Condition::MostGoals(Some("Billy".into())).eval(&state(), Side::Home, "Get Out"),
            True,
            "the named player's count"
        );
        assert_eq!(eval(Condition::Special(Some("x".into()))), False);
    }

    #[test]
    fn time_needs_the_minute_and_unloads_once_it_has_passed() {
        use Comparison::*;
        assert_eq!(eval(Condition::Time(Lt, 45)), False, "no minute yet");
        let mut at_30 = state();
        at_30.goal_minute = Some(30);
        let at = |condition: Condition| condition.eval(&at_30, Side::Home, "Billy");
        assert_eq!(at(Condition::Time(Lt, 45)), True);
        assert_eq!(at(Condition::Time(Ge, 45)), False);
        assert_eq!(at(Condition::Time(Eq, 30)), True);
        let mut at_60 = state();
        at_60.goal_minute = Some(60);
        let late = |condition: Condition| condition.eval(&at_60, Side::Home, "Billy");
        assert_eq!(late(Condition::Time(Lt, 45)), NeverAgain);
        assert_eq!(late(Condition::Time(Le, 45)), NeverAgain);
        assert_eq!(late(Condition::Time(Eq, 45)), NeverAgain);
        assert_eq!(late(Condition::Time(Gt, 45)), True);
        assert_eq!(late(Condition::Time(Ne, 45)), True);
        assert_eq!(late(Condition::Time(Ge, 90)), False, "may still come");
    }

    #[test]
    fn not_inverts_and_a_never_again_inner_holds_from_then_on() {
        use Comparison::*;
        assert_eq!(eval(Condition::Not(Box::new(Condition::First))), True);
        assert_eq!(eval(Condition::Not(Box::new(Condition::Home))), False);
        let mut at_60 = state();
        at_60.goal_minute = Some(60);
        assert_eq!(
            Condition::Not(Box::new(Condition::Time(Le, 45))).eval(&at_60, Side::Home, "Billy"),
            True
        );
        assert!(Condition::Not(Box::new(Condition::Time(Le, 45))).needs_goal_minute());
        assert!(!Condition::Not(Box::new(Condition::Home)).needs_goal_minute());
    }

    fn entry(conditions: Vec<Condition>) -> SongEntry {
        SongEntry {
            file: "x.mp3".into(),
            conditions,
            instructions: vec![Instruction::Randomise],
        }
    }

    #[test]
    fn an_entry_is_the_and_of_its_conditions_with_never_again_first() {
        use Comparison::*;
        let at = |conditions: Vec<Condition>| {
            let mut late = state();
            late.goal_minute = Some(60);
            entry(conditions).eval(&late, Side::Home, "Billy")
        };
        assert_eq!(at(vec![]), True);
        assert_eq!(at(vec![Condition::Home, Condition::Goals(Ge, 2)]), True);
        assert_eq!(at(vec![Condition::Home, Condition::First]), False);
        assert_eq!(
            at(vec![Condition::First, Condition::Time(Lt, 45)]),
            NeverAgain,
            "a failing condition does not hide the unload"
        );
        assert!(entry(vec![Condition::Home, Condition::Time(Gt, 1)]).needs_goal_minute());
        assert!(!entry(vec![Condition::Home]).needs_goal_minute());
        assert!(entry(vec![Condition::Comeback, Condition::Once]).is_once());
        assert!(!entry(vec![Condition::Comeback]).is_once());
    }
}
