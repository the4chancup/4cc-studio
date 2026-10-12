//! The one engine every script runs on: the bare engine plus the
//! standard package, the accessor table (`Player`/`Team`), and the
//! call-depth limit. The operation limit is 5.2c's.

use pes_savefile::interchange::team_toml::labels::RATINGS;
use rhai::packages::Package;
use rhai::{Array, Dynamic, Engine, EvalAltResult, Map};

use crate::ruleset::Position;
use crate::team::{PlayerView, TeamView};

/// A script error naming `message`.
fn script_error(message: String) -> Box<EvalAltResult> {
    EvalAltResult::ErrorRuntime(message.into(), rhai::Position::NONE).into()
}

/// The engine the host and the data-block loader share: `new_raw` keeps
/// the module resolver out (no `import`) and gives `print` and `debug` no
/// output (`Engine::new` writes them to stdout), the standard package adds
/// arrays, maps and strings, and the accessors are registered here and
/// nowhere else. Labels convert to and from the closed sets in these
/// functions only; an unknown label or a preset outside 1 to 3 is a
/// script error naming the value. Loops still run — the operation limit
/// is the caller's to set.
pub(crate) fn engine() -> Engine {
    let mut engine = Engine::new_raw();
    engine.register_global_module(rhai::packages::StandardPackage::new().as_shared_module());
    engine
        .register_type_with_name::<PlayerView>("Player")
        .register_get("slot", |p: &mut PlayerView| i64::from(p.slot))
        .register_get("age", |p: &mut PlayerView| i64::from(p.age))
        .register_get("height", |p: &mut PlayerView| i64::from(p.height))
        .register_get("weight", |p: &mut PlayerView| i64::from(p.weight))
        .register_get("form", |p: &mut PlayerView| i64::from(p.form))
        .register_get("injury_resistance", |p: &mut PlayerView| {
            i64::from(p.injury_resistance)
        })
        .register_get("weak_foot_usage", |p: &mut PlayerView| {
            i64::from(p.weak_foot_usage)
        })
        .register_get("weak_foot_accuracy", |p: &mut PlayerView| {
            i64::from(p.weak_foot_accuracy)
        })
        .register_get("position", |p: &mut PlayerView| {
            p.position.map_or(Dynamic::UNIT, |pos| pos.label().into())
        })
        .register_get("playing_style", |p: &mut PlayerView| {
            p.playing_style
                .map_or(Dynamic::UNIT, |style| style.label().into())
        })
        .register_get("name_color", |p: &mut PlayerView| {
            p.name_color
                .clone()
                .map_or(Dynamic::UNIT, |color| color.into())
        })
        .register_get("cards", |p: &mut PlayerView| {
            p.cards
                .iter()
                .map(|card| Dynamic::from(card.label()))
                .collect::<Array>()
        })
        .register_fn("stat", |p: &mut PlayerView, name: &str| {
            p.stat_by_name(name)
                .map(|value| value.map_or(Dynamic::UNIT, |v| i64::from(v).into()))
                .map_err(script_error)
        })
        .register_fn("rating_at", |p: &mut PlayerView, pos: &str| {
            p.rating_at(pos)
                .map(|rating| {
                    RATINGS
                        .get(usize::from(rating))
                        .map_or(Dynamic::UNIT, |label| (*label).into())
                })
                .map_err(script_error)
        });
    engine
        .register_type_with_name::<TeamView>("Team")
        .register_get("players", |t: &mut TeamView| {
            t.players
                .iter()
                .cloned()
                .map(Dynamic::from)
                .collect::<Array>()
        })
        .register_get("captain", |t: &mut TeamView| {
            t.captain
                .map_or(Dynamic::UNIT, |slot| i64::from(slot).into())
        })
        .register_get("version", |t: &mut TeamView| i64::from(t.version.number()))
        .register_fn(
            "fielded_positions",
            |t: &mut TeamView, p: PlayerView, preset: i64| -> Result<Array, Box<EvalAltResult>> {
                let preset = u8::try_from(preset)
                    .ok()
                    .filter(|preset| (1..=3).contains(preset))
                    .ok_or_else(|| script_error(format!("preset {preset}: outside 1 to 3")))?;
                Ok(t.fielded_positions(p.slot, preset)
                    .iter()
                    .map(|pos| Dynamic::from(pos.label()))
                    .collect::<Array>())
            },
        )
        .register_fn(
            "fielded_only_at",
            |t: &mut TeamView, p: PlayerView, pos: &str| {
                Position::try_from(pos.to_string())
                    .map(|position| t.fielded_only_at(p.slot, position))
                    .map_err(script_error)
            },
        )
        .register_fn("formation_shapes", |t: &mut TeamView| {
            t.formation_shapes()
                .iter()
                .map(|shape| {
                    let mut map = Map::new();
                    map.insert("preset".into(), i64::from(shape.preset).into());
                    map.insert("formation".into(), i64::from(shape.formation).into());
                    map.insert(
                        "shape".into(),
                        shape
                            .shape
                            .iter()
                            .map(|count| Dynamic::from(i64::from(*count)))
                            .collect::<Array>()
                            .into(),
                    );
                    Dynamic::from_map(map)
                })
                .collect::<Array>()
        });
    engine.set_max_call_levels(32);
    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use pes_savefile::EditFile;
    use pes_savefile::fixture_save;
    use pes_version::PesVersion;

    /// The engine and a scope holding `team` for `version`'s team 701.
    fn with_team(version: PesVersion) -> (Engine, rhai::Scope<'static>, EditFile) {
        let file = EditFile::from_bytes(&fixture_save(version)).expect("fixture save");
        let team = TeamView::new(&file, 701).expect("the team views");
        let mut scope = rhai::Scope::new();
        scope.push("team", team);
        (engine(), scope, file)
    }

    /// Team 701 of `file`, mutable.
    fn team_mut(file: &mut EditFile) -> &mut pes_savefile::model::TeamEntry {
        file.teams_mut()
            .iter_mut()
            .find(|team| team.id == 701)
            .expect("team 701")
    }

    fn eval_i64(engine: &Engine, scope: &mut rhai::Scope, script: &str) -> i64 {
        engine.eval_with_scope::<i64>(scope, script).expect(script)
    }

    /// Every `Player` accessor on UTAH TEAPOT (PES 19 slot 10), the
    /// scaled values' stored bytes asserted one less first.
    #[test]
    fn every_player_accessor_answers() {
        let (engine, mut scope, file) = with_team(PesVersion::Pes19);
        let entry = file.player(70111).expect("player 70111");
        assert_eq!(entry.stats.form, 7);
        assert_eq!(entry.stats.injury_resistance, 2);
        assert_eq!(entry.stats.weak_foot_usage, 3);
        assert_eq!(entry.stats.weak_foot_accuracy, 3);
        assert_eq!(entry.basic.age, 20);
        assert_eq!(entry.basic.height, 185);
        assert_eq!(entry.basic.weight, 80);
        for (script, want) in [
            ("team.players[10].slot", 10),
            ("team.players[10].form", 8),
            ("team.players[10].injury_resistance", 3),
            ("team.players[10].weak_foot_usage", 4),
            ("team.players[10].weak_foot_accuracy", 4),
            ("team.players[10].age", 20),
            ("team.players[10].height", 185),
            ("team.players[10].weight", 80),
        ] {
            assert_eq!(eval_i64(&engine, &mut scope, script), want, "{script}");
        }
        assert_eq!(
            engine
                .eval_with_scope::<String>(&mut scope, "team.players[10].position")
                .expect("position"),
            "CF"
        );
        // Style byte 1 on PES 19 is GoalPoacher (`PLAY_STYLES[1]`).
        assert_eq!(entry.positions.playing_style, 1);
        assert_eq!(
            engine
                .eval_with_scope::<String>(&mut scope, "team.players[10].playing_style")
                .expect("playing_style"),
            "goal_poacher"
        );
        // `cards`, skills in table order then COM styles.
        let cards = engine
            .eval_with_scope::<Array>(&mut scope, "team.players[10].cards")
            .expect("cards");
        let labels: Vec<String> = cards
            .iter()
            .map(|card| card.clone().into_string().expect("a label"))
            .collect();
        assert_eq!(
            labels,
            [
                "long_range_drive",
                "knuckle_shot",
                "first_time_shot",
                "one_touch_pass",
                "weighted_pass",
                "track_back",
                "captaincy",
                "mazing_run",
                "speeding_bullet"
            ]
        );
    }

    #[test]
    fn stat_returns_unit_where_the_version_lacks_it() {
        // PES 15 has no Physical Contact.
        let (engine, mut scope, _) = with_team(PesVersion::Pes15);
        assert!(
            engine
                .eval_with_scope::<Dynamic>(
                    &mut scope,
                    "team.players[10].stat(\"physical_contact\")"
                )
                .expect("stat")
                .is_unit(),
            "physical_contact is () on PES 15"
        );
        let (engine, mut scope, _) = with_team(PesVersion::Pes21);
        let value = eval_i64(
            &engine,
            &mut scope,
            "team.players[10].stat(\"physical_contact\")",
        );
        assert!(value > 0, "a number on PES 21: {value}");
    }

    #[test]
    fn rating_at_names_the_letters() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        // 70110 is a B at SS and LMF, nowhere else; give him an A at CF.
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == 70110)
            .expect("player 70110");
        assert_eq!(entry.positions.playable[0], 0);
        assert_eq!(entry.positions.playable[1], 2);
        entry.positions.playable[0] = 3;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        for (pos, want) in [("CF", "A"), ("SS", "B"), ("GK", "none")] {
            assert_eq!(
                engine
                    .eval_with_scope::<String>(
                        &mut scope,
                        &format!("team.players[9].rating_at(\"{pos}\")")
                    )
                    .expect("rating_at"),
                want,
                "{pos}"
            );
        }
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.players[9].rating_at(\"XX\")")
                .is_err(),
            "an unknown position is a script error"
        );
    }

    #[test]
    fn name_color_reads_a_leading_code() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == 70111)
            .expect("player 70111");
        assert_eq!(entry.name, "UTAH TEAPOT");
        entry.name = "\u{11}ccc9900ffUTAH TEAPOT".to_string();
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert_eq!(
            engine
                .eval_with_scope::<String>(&mut scope, "team.players[10].name_color")
                .expect("name_color"),
            "cc9900ff"
        );
        let (engine, mut scope, _) = with_team(PesVersion::Pes19);
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.players[10].name_color")
                .expect("name_color")
                .is_unit(),
            "no leading code gives ()"
        );
    }

    #[test]
    fn captain_is_the_slot_or_unit() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        assert_eq!(team_mut(&mut file).tactics.set_pieces.captain, 10);
        assert_eq!(eval_i64(&engine, &mut scope, "team.captain"), 10);
        team_mut(&mut file).tactics.set_pieces.captain = 0xFF;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.captain")
                .expect("captain")
                .is_unit(),
            "0xFF captain is ()"
        );
    }

    #[test]
    fn team_accessors_answer() {
        let (engine, mut scope, _) = with_team(PesVersion::Pes19);
        assert_eq!(eval_i64(&engine, &mut scope, "team.players.len"), 23);
        let fielded = engine
            .eval_with_scope::<Array>(&mut scope, "team.fielded_positions(team.players[8], 1)")
            .expect("fielded_positions");
        assert_eq!(
            fielded
                .into_iter()
                .map(|pos| pos.into_string().expect("a label"))
                .collect::<Vec<_>>(),
            vec!["AMF".to_string()]
        );
        assert!(
            engine
                .eval_with_scope::<bool>(
                    &mut scope,
                    "team.fielded_only_at(team.players[10], \"CF\")"
                )
                .expect("fielded_only_at")
        );
        let shapes = engine
            .eval_with_scope::<Array>(&mut scope, "team.formation_shapes()")
            .expect("formation_shapes");
        assert_eq!(shapes.len(), 3);
        assert_eq!(
            eval_i64(&engine, &mut scope, "team.formation_shapes()[0].preset"),
            1
        );
        let shape = engine
            .eval_with_scope::<Array>(&mut scope, "team.formation_shapes()[0].shape")
            .expect("shape");
        assert_eq!(
            shape
                .into_iter()
                .map(|count| count.as_int().expect("a count"))
                .collect::<Vec<_>>(),
            vec![4, 4, 2]
        );
        // A preset outside 1 to 3 is a script error.
        assert!(
            engine
                .eval_with_scope::<Array>(&mut scope, "team.fielded_positions(team.players[8], 4)")
                .is_err(),
            "preset 4 is a script error"
        );
        let (engine, mut scope, _) = with_team(PesVersion::Pes21);
        assert_eq!(eval_i64(&engine, &mut scope, "team.version"), 21);
    }

    #[test]
    fn deep_recursion_is_refused() {
        let (engine, scope, _) = with_team(PesVersion::Pes19);
        let recurse = |depth: u32| {
            engine.eval_with_scope::<Dynamic>(
                &mut scope.clone(),
                &format!("fn f(n) {{ if n > 0 {{ f(n - 1) }} else {{ 0 }} }} f({depth})"),
            )
        };
        let value = recurse(20).expect("20 deep is within the call-depth limit");
        assert_eq!(value.as_int(), Ok(0));
        let err = recurse(40).expect_err("40 deep is past the call-depth limit");
        assert!(
            matches!(*err, EvalAltResult::ErrorStackOverflow(..)),
            "{err:?}"
        );
    }
}
