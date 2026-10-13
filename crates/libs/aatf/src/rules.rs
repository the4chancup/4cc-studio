//! `Rules`: a loaded rules file — its ruleset, its compiled script and
//! the engine it runs on — and the host API over a `TeamView`:
//! `check_team`, `tier_values`, `card_limits` and `apply_tier`.

use pes_savefile::model::player::PlayerEntry;
use serde::de::DeserializeOwned;

use crate::file::{LogicIdentity, assemble, identity};
use crate::ruleset::{LoadError, Ruleset, ruleset_from_scope, run_top_level};
use crate::team::{PlayerView, TeamView};
use crate::values::{CardLimits, TierValues};
use crate::violation::{CheckError, Violation, violations};

/// The official data block, embedded. The other rulesets' blocks are
/// embedded by their fixtures until the Ruleset editor's templates
/// (Phase 20) need them here.
const AUTUMN26: &str = include_str!("../rulesets/autumn26.rhai");

/// A loaded rules file: its ruleset, its compiled script and the engine
/// it runs on.
#[derive(Debug)]
pub struct Rules {
    /// The ruleset the file's `RULESET` deserialized into.
    ruleset: Ruleset,
    /// The compiled script (`check_team`, `tier_values`, `card_limits`,
    /// `custom_checks` live in it).
    ast: rhai::AST,
    /// The engine the script runs on (`engine.rs` — same one as every
    /// other path).
    engine: rhai::Engine,
    /// The file's logic identity (`file.rs`).
    logic: LogicIdentity,
}

/// The functions the file must define, and their arities.
const REQUIRED: [(&str, usize); 3] = [("check_team", 1), ("tier_values", 3), ("card_limits", 3)];

impl Rules {
    /// Loads a rules file through steps 1 to 3 of `aatf_rules.md`
    /// "Loading and validation"; the first failure is returned.
    pub fn load(text: &str) -> Result<Rules, LoadError> {
        let (engine, ast, scope) = run_top_level(text)?;
        // Step 1: the three functions with their arities, and every
        // `custom_checks` with one.
        for (name, arity) in REQUIRED {
            if !ast
                .iter_functions()
                .any(|f| f.name == name && f.params.len() == arity)
            {
                return Err(LoadError::MissingFunction { name, arity });
            }
        }
        for arity in ast
            .iter_functions()
            .filter(|f| f.name == "custom_checks")
            .map(|f| f.params.len())
        {
            if arity != 1 {
                return Err(LoadError::CustomChecksArity { arity });
            }
        }
        let ruleset = ruleset_from_scope(&scope)?;
        let rules = Rules {
            ruleset,
            ast,
            engine,
            logic: identity(text),
        };
        // Step 3: `check_team` runs on the sample team; a `CheckError`
        // there is a load failure — the file's own logic and custom
        // checks must run before any real check does.
        if let Err(error) = rules.check_team(&TeamView::sample_team()) {
            return Err(LoadError::SampleTeam { error });
        }
        Ok(rules)
    }

    /// The official ruleset, the default file: `rulesets/autumn26.rhai`
    /// assembled with the current logic. Panics only if the embedded
    /// file does not load, which a test rules out.
    pub fn official() -> Rules {
        Rules::from_data_block(AUTUMN26).expect("the embedded official file loads")
    }

    /// The rules of an embedded data block assembled with the current
    /// logic: the other rulesets' fixtures run on these.
    pub(crate) fn from_data_block(data_block: &str) -> Result<Rules, LoadError> {
        Rules::load(&assemble(data_block))
    }

    /// The file's ruleset.
    pub fn ruleset(&self) -> &Ruleset {
        &self.ruleset
    }

    /// The file's logic identity: the current logic, an older official
    /// version, or modified.
    pub fn logic(&self) -> LogicIdentity {
        self.logic
    }

    /// `team`'s violations: the file's `check_team(team)`, its returned
    /// list converted.
    pub fn check_team(&self, team: &TeamView) -> Result<Vec<Violation>, CheckError> {
        let list = self
            .engine
            .call_fn::<rhai::Dynamic>(
                &mut rhai::Scope::new(),
                &self.ast,
                "check_team",
                (team.clone(),),
            )
            .map_err(|e| CheckError::Script {
                message: e.to_string(),
            })?;
        violations(list)
    }

    /// The targets `player` (of `team`) must carry as tier `tier`, the
    /// specials applied: the file's `tier_values`.
    pub fn tier_values(
        &self,
        team: &TeamView,
        player: &PlayerView,
        tier: &str,
    ) -> Result<TierValues, CheckError> {
        self.values("tier_values", team, player, tier)
    }

    /// The card economy of `player` (of `team`) as tier `tier`: the
    /// file's `card_limits`, behind the card pickers' remaining counts.
    pub fn card_limits(
        &self,
        team: &TeamView,
        player: &PlayerView,
        tier: &str,
    ) -> Result<CardLimits, CheckError> {
        self.values("card_limits", team, player, tier)
    }

    /// Writes tier `tier`'s targets into `entry`, the record `player`
    /// views (of `team`): stats, form, injury resistance and weak foot,
    /// so the quick actions and the Team creator's defaults produce a
    /// player `check_team` accepts on those. The lib's one writing
    /// operation; the check itself never fixes anything.
    pub fn apply_tier(
        &self,
        team: &TeamView,
        player: &PlayerView,
        tier: &str,
        entry: &mut PlayerEntry,
    ) -> Result<(), CheckError> {
        self.tier_values(team, player, tier)?.write(entry);
        Ok(())
    }

    /// `function` (`tier_values` or `card_limits`) called on `player` as
    /// `tier`, its map converted; an id no tier has is refused here, not
    /// by the script.
    fn values<T: DeserializeOwned>(
        &self,
        function: &'static str,
        team: &TeamView,
        player: &PlayerView,
        tier: &str,
    ) -> Result<T, CheckError> {
        if !self.ruleset.tiers.iter().any(|t| t.id == tier) {
            return Err(CheckError::UnknownTier {
                tier: tier.to_string(),
            });
        }
        let map = self
            .engine
            .call_fn::<rhai::Dynamic>(
                &mut rhai::Scope::new(),
                &self.ast,
                function,
                (player.clone(), tier.to_string(), team.clone()),
            )
            .map_err(|e| CheckError::Script {
                message: e.to_string(),
            })?;
        rhai::serde::from_dynamic(&map).map_err(|e| CheckError::Values {
            function,
            message: e.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pes_savefile::EditFile;
    use pes_savefile::fixture_save;
    use pes_version::PesVersion;

    /// Team 701 of `version`'s fixture save.
    fn fixture_team(version: PesVersion) -> TeamView {
        let file = EditFile::from_bytes(&fixture_save(version)).expect("fixture save");
        TeamView::new(&file, 701).expect("team 701 views")
    }

    /// The smallest file that loads: the three functions and a
    /// one-tier `RULESET`.
    fn minimal_file() -> String {
        "const RULESET = #{ schema: 1, name: \"x\", squad_size: 23, universal: #{ max_skill_cards: 10 }, tiers: [#{ id: \"regular\", label: \"Regular\", rate: 77 }] };\nfn check_team(team) { [] }\nfn tier_values(player, tier, team) { #{} }\nfn card_limits(player, tier, team) { #{} }".to_string()
    }

    /// The official file loads, and its check runs without a script
    /// error on team 701 of every fixture save (the legal and violating
    /// fixtures are `fixtures/`; PES 20 and 21's team 701 are placeholder
    /// players, every field 0).
    #[test]
    fn official_loads_and_checks_every_fixture_version() {
        let rules = Rules::official();
        assert_eq!(rules.ruleset().name, "Autumn 26");
        for version in PesVersion::ALL {
            rules
                .check_team(&fixture_team(version))
                .unwrap_or_else(|e| panic!("{version:?}: {e}"));
        }
    }

    /// The custom checks' violations come last, after the interpreter's.
    #[test]
    fn a_custom_check_reaches_the_result() {
        // The assembled file's `fn custom_checks(team) { [] }` replaced
        // by one naming the first player's slot.
        let assembled = assemble(AUTUMN26);
        assert_eq!(
            assembled.matches("fn custom_checks(team) { [] }").count(),
            1
        );
        let text = assembled.replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { [#{ kind: \"Custom\", text: \"x\", slot: team.players[0].slot }] }",
            1,
        );
        let rules = Rules::load(&text).expect("the file loads");
        let team = fixture_team(PesVersion::Pes15);
        let found = rules.check_team(&team).expect("the check runs");
        let custom = Violation {
            slot: Some(0),
            severity: crate::ruleset::Severity::Error,
            kind: crate::violation::ViolationKind::Custom {
                text: "x".to_string(),
            },
        };
        assert_eq!(found.last(), Some(&custom));
        assert_eq!(found.iter().filter(|v| **v == custom).count(), 1);
        assert!(found.len() > 1, "the interpreter reported on team 701 too");
    }

    #[test]
    fn a_script_reads_global_ruleset() {
        // `call_fn` re-evaluates the top level on a fresh scope, so the
        // script sees `global::RULESET`: the mechanism the interpreter
        // reads its data through.
        let text = assemble(AUTUMN26).replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { [#{ kind: \"Custom\", text: global::RULESET.name }] }",
            1,
        );
        let rules = Rules::load(&text).expect("the file loads");
        let team = fixture_team(PesVersion::Pes15);
        let found = rules.check_team(&team).expect("the check runs");
        assert_eq!(
            found.last().map(|v| &v.kind),
            Some(&crate::violation::ViolationKind::Custom {
                text: "Autumn 26".to_string(),
            })
        );
    }

    #[test]
    fn a_file_without_custom_checks_reports_no_custom_violation() {
        let text = assemble(AUTUMN26).replacen("fn custom_checks(team) { [] }", "", 1);
        assert!(!text.contains("fn custom_checks"));
        let rules = Rules::load(&text).expect("the file loads");
        let found = rules
            .check_team(&fixture_team(PesVersion::Pes15))
            .expect("the check runs");
        assert!(!found.is_empty(), "team 701 is not legal under Autumn 26");
        assert!(
            !found
                .iter()
                .any(|v| matches!(v.kind, crate::violation::ViolationKind::Custom { .. }))
        );
    }

    /// The official ruleset on the sample team costs under a tenth of
    /// `MAX_OPERATIONS` (the plan's tenfold margin), and the limit is not
    /// stale: under thirty times the cost. The count and the time are in
    /// the assertion messages: to read them, tighten a bound for one run.
    #[test]
    fn the_operation_limit_keeps_a_tenfold_margin() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicU64, Ordering};

        let mut rules = Rules::official();
        let peak = Arc::new(AtomicU64::new(0));
        let seen = Arc::clone(&peak);
        rules.engine.on_progress(move |operations| {
            seen.fetch_max(operations, Ordering::Relaxed);
            None
        });
        let start = std::time::Instant::now();
        rules
            .check_team(&TeamView::sample_team())
            .expect("the check runs");
        let elapsed = start.elapsed();
        let operations = peak.load(Ordering::Relaxed);
        assert!(operations > 0, "on_progress saw the run");
        assert!(
            operations * 10 <= crate::engine::MAX_OPERATIONS,
            "{operations} operations in {elapsed:?}: the margin is under tenfold; raise MAX_OPERATIONS"
        );
        assert!(
            crate::engine::MAX_OPERATIONS <= operations * 30,
            "{operations} operations in {elapsed:?}: MAX_OPERATIONS is stale; lower it to ten times the cost"
        );
    }

    /// The tables the logic carries itself (the ability stats it targets,
    /// the positions it counts, the COM styles of the `com` group) are
    /// the label tables the host converts with: a stat missing from the
    /// logic's list would go unchecked without a word.
    #[test]
    fn the_logics_tables_are_the_label_tables() {
        use pes_savefile::interchange::team_toml::labels::{ABILITY_STATS, COM_STYLES, POSITIONS};

        let rules = Rules::official();
        for (function, table) in [
            ("ability_stats", ABILITY_STATS.as_slice()),
            ("positions", POSITIONS.as_slice()),
            ("com_styles", COM_STYLES.as_slice()),
        ] {
            let listed: Vec<String> = rules
                .engine
                .call_fn::<rhai::Array>(&mut rhai::Scope::new(), &rules.ast, function, ())
                .expect(function)
                .into_iter()
                .map(|name| name.into_string().expect("a name"))
                .collect();
            assert_eq!(listed, table, "{function}");
        }
    }

    /// A `tier_values` returning a map that is not the values (the
    /// minimal file's `#{}`) is `Values` naming the function.
    #[test]
    fn a_values_map_that_does_not_convert_is_values() {
        let rules = Rules::load(&minimal_file()).expect("the file loads");
        let team = fixture_team(PesVersion::Pes15);
        let player = team.player(0).expect("slot 0");
        match rules
            .tier_values(&team, player, "regular")
            .expect_err("an empty map")
        {
            CheckError::Values { function, .. } => assert_eq!(function, "tier_values"),
            other => panic!("expected Values, got {other:?}"),
        }
        match rules
            .card_limits(&team, player, "regular")
            .expect_err("an empty map")
        {
            CheckError::Values { function, .. } => assert_eq!(function, "card_limits"),
            other => panic!("expected Values, got {other:?}"),
        }
    }

    /// A script that loops forever stops at the operation limit with a
    /// `Script` error, not a hang: a `loop {}` on PES 21 only, so the
    /// sample team's PES 15 load passes.
    #[test]
    fn a_script_that_loops_forever_stops_at_the_operation_limit() {
        let text = assemble(AUTUMN26).replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { if team.version == 21 { loop {} } [] }",
            1,
        );
        let err = Rules::load(&text)
            .expect("the file loads")
            .check_team(&fixture_team(PesVersion::Pes21))
            .expect_err("the limit stops it");
        match &err {
            CheckError::Script { message } => {
                assert!(message.contains("operations"), "{message}");
            }
            other => panic!("expected Script, got {other:?}"),
        }
    }

    #[test]
    fn a_script_error_and_a_bad_violation_surface() {
        // `throw` on PES 21 only: the sample team's PES 15 load passes,
        // and the check on a PES 21 team is `CheckError::Script`.
        let text = assemble(AUTUMN26).replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { if team.version == 21 { throw \"boom\" } [] }",
            1,
        );
        let err = Rules::load(&text)
            .expect("the file loads")
            .check_team(&fixture_team(PesVersion::Pes21))
            .expect_err("a script error");
        match &err {
            CheckError::Script { message } => assert!(message.contains("boom"), "{message}"),
            other => panic!("expected Script, got {other:?}"),
        }
        // A bad entry in the list fails at its index, on PES 21 only: the
        // custom checks' entries follow the interpreter's own.
        let own = Rules::official()
            .check_team(&fixture_team(PesVersion::Pes21))
            .expect("the check runs")
            .len();
        assert!(own > 0, "team 701 is not legal under Autumn 26");
        let text = assemble(AUTUMN26).replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { if team.version == 21 { [#{ kind: \"Captain\" }, #{ kind: \"Nope\" }] } else { [] } }",
            1,
        );
        let err = Rules::load(&text)
            .expect("the file loads")
            .check_team(&fixture_team(PesVersion::Pes21))
            .expect_err("a bad violation");
        match &err {
            CheckError::Violation { index, message } => {
                assert_eq!(*index, own + 1);
                assert!(message.contains("Nope"), "{message}");
            }
            other => panic!("expected Violation, got {other:?}"),
        }
    }

    /// One failing file per step-1 shape: the parse position, the
    /// function arities, `RULESET` absent.
    #[test]
    fn load_step_one_refuses_each_missing_shape() {
        // A stray token: `Parse` at its line and column.
        let err = Rules::load("const A = 1;\nconst B = 2;\nhello\n").expect_err("a stray token");
        match err {
            LoadError::Parse { line, column, .. } => {
                assert_eq!((line, column), (3, 1));
            }
            other => panic!("expected Parse, got {other:?}"),
        }
        // Each required function missing.
        for (remove, name, arity) in [
            ("fn check_team(team) { [] }", "check_team", 1usize),
            (
                "fn tier_values(player, tier, team) { #{} }",
                "tier_values",
                3,
            ),
            (
                "fn card_limits(player, tier, team) { #{} }",
                "card_limits",
                3,
            ),
        ] {
            let text = minimal_file().replacen(remove, "", 1);
            assert!(!text.contains(remove));
            assert_eq!(
                Rules::load(&text).expect_err(name),
                LoadError::MissingFunction { name, arity },
                "{name}"
            );
        }
        // `check_team` exists, but with two parameters only.
        let err = Rules::load(&minimal_file().replacen(
            "fn check_team(team) { [] }",
            "fn check_team(team, extra) { [] }",
            1,
        ))
        .expect_err("wrong arity");
        assert_eq!(
            err,
            LoadError::MissingFunction {
                name: "check_team",
                arity: 1
            }
        );
        // `custom_checks` with two parameters.
        let err = Rules::load(&format!(
            "{minimal}\nfn custom_checks(team, extra) {{ [] }}",
            minimal = minimal_file()
        ))
        .expect_err("custom_checks arity");
        assert_eq!(err, LoadError::CustomChecksArity { arity: 2 });
        // No `RULESET`.
        let err = Rules::load(&minimal_file().replacen("const RULESET = ", "const OTHER = ", 1))
            .expect_err("no RULESET");
        assert_eq!(err, LoadError::NoRuleset);
        // A `let RULESET` is not a constant either.
        let err = Rules::load(&minimal_file().replacen("const RULESET = ", "let RULESET = ", 1))
            .expect_err("a let");
        assert_eq!(err, LoadError::NoRuleset);
    }

    /// Step 2: `RULESET` breaking a load-time rule gives `Rules`.
    #[test]
    fn load_step_two_reports_the_rules() {
        let text = minimal_file().replacen("name: \"x\"", "name: \"\"", 1);
        let err = Rules::load(&text).expect_err("empty name");
        assert!(matches!(err, LoadError::Rules(_)), "{err:?}");
    }

    /// Step 3: `check_team` failing on the sample team (PES 15) is a
    /// load failure, `CheckError` carried whole.
    #[test]
    fn load_step_three_runs_check_team_on_the_sample_team() {
        // A throw only on PES 15: `SampleTeam` holding `Script`.
        let err = Rules::load(&minimal_file().replacen(
            "fn check_team(team) { [] }",
            "fn check_team(team) { if team.version == 15 { throw \"boom\" } [] }",
            1,
        ))
        .expect_err("a script error on the sample team");
        match &err {
            LoadError::SampleTeam { error } => {
                assert!(
                    matches!(error, CheckError::Script { message } if message.contains("boom")),
                    "{err:?}"
                );
            }
            other => panic!("expected SampleTeam, got {other:?}"),
        }
        // A bad violation map only on PES 15: `SampleTeam` holding
        // `Violation` at index 0.
        let err = Rules::load(&minimal_file().replacen(
            "fn check_team(team) { [] }",
            "fn check_team(team) { if team.version == 15 { [#{ kind: \"Nope\" }] } else { [] } }",
            1,
        ))
        .expect_err("a bad map on the sample team");
        match &err {
            LoadError::SampleTeam {
                error: CheckError::Violation { index, .. },
            } => assert_eq!(*index, 0),
            other => panic!("expected SampleTeam holding Violation, got {other:?}"),
        }
    }
}
