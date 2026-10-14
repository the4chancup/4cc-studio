//! The one engine every script runs on: the bare engine plus the
//! standard package, the accessor table (`Player`/`Team`), and the
//! operation and call-depth limits.

use pes_savefile::interchange::team_toml::labels::RATINGS;
use rhai::packages::Package;
use rhai::{Array, Dynamic, Engine, EvalAltResult, Map};

use crate::ruleset::Position;
use crate::team::{PlayerView, TeamView};

/// The most operations one script run may take: the official ruleset's
/// `check_team` on the sample team costs about 480 000 operations
/// (measured 2026-10-14, `rules.rs`), and the plan sets a tenfold margin,
/// which that file's test keeps honest. A script that loops forever
/// stops here with a script error, never a hang.
pub(crate) const MAX_OPERATIONS: u64 = 5_000_000;

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
/// script error naming the value.
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
        .register_get("strong_foot", |p: &mut PlayerView| {
            p.strong_foot
                .map_or(Dynamic::UNIT, |foot| foot.label().into())
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
        .register_get("man_marking", |t: &mut TeamView| t.man_marking())
        .register_get("auto_substitution", |t: &mut TeamView| {
            t.tactics
                .auto
                .substitution
                .map_or(Dynamic::UNIT, |setting| i64::from(setting).into())
        })
        .register_get("auto_offside_trap", |t: &mut TeamView| {
            t.tactics
                .auto
                .offside_trap
                .map_or(Dynamic::UNIT, Dynamic::from)
        })
        .register_get("auto_preset_change", |t: &mut TeamView| {
            t.tactics
                .auto
                .preset_change
                .map_or(Dynamic::UNIT, Dynamic::from)
        })
        .register_get("auto_attack_defence_levels", |t: &mut TeamView| {
            t.tactics
                .auto
                .attack_defence_levels
                .map_or(Dynamic::UNIT, Dynamic::from)
        })
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
        .register_fn(
            "formation",
            |t: &mut TeamView,
             preset: i64,
             formation: i64|
             -> Result<Dynamic, Box<EvalAltResult>> {
                let preset = u8::try_from(preset)
                    .ok()
                    .filter(|preset| (1..=3).contains(preset))
                    .ok_or_else(|| script_error(format!("preset {preset}: outside 1 to 3")))?;
                // A formation outside 1 to 3 (or u8) is one the preset
                // does not field.
                let Ok(formation) = u8::try_from(formation) else {
                    return Ok(Dynamic::UNIT);
                };
                Ok(t.formation(preset, formation)
                    .map_or(Dynamic::UNIT, |slots| {
                        slots
                            .iter()
                            .map(|slot| {
                                let mut map = Map::new();
                                map.insert("slot".into(), i64::from(slot.slot).into());
                                map.insert("position".into(), slot.position.label().into());
                                map.insert("x".into(), i64::from(slot.x).into());
                                map.insert("y".into(), i64::from(slot.y).into());
                                Dynamic::from_map(map)
                            })
                            .collect::<Array>()
                            .into()
                    }))
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
    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_call_levels(32);
    // rhai's expression-depth defaults differ between debug (32 at the top
    // level, 16 inside a function) and release builds (64 and 32), and the
    // interpreter's own functions nest past 16 (a loop, a branch, a call
    // and a map literal each count a level), so both are pinned: a file
    // parses the same under the tests and in the binary.
    engine.set_max_expr_depths(64, 64);
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
        let (engine, mut scope, file) = with_team(PesVersion::Pes21);
        assert_eq!(
            file.player(70111).expect("70111").stats.physical_contact,
            Some(77)
        );
        assert_eq!(
            eval_i64(
                &engine,
                &mut scope,
                "team.players[10].stat(\"physical_contact\")"
            ),
            77
        );
    }

    #[test]
    fn stat_returns_the_named_ability() {
        // 70108's dribbling set to 80 (89 in the fixture), so the two
        // stats differ: `stat` answers by name, not by position.
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes18);
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == 70108)
            .expect("player 70108");
        assert_eq!(entry.stats.physical_contact, Some(89));
        assert_eq!(entry.stats.dribbling, 89);
        entry.stats.dribbling = 80;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert_eq!(
            eval_i64(
                &engine,
                &mut scope,
                "team.players[7].stat(\"physical_contact\")"
            ),
            89
        );
        assert_eq!(
            eval_i64(&engine, &mut scope, "team.players[7].stat(\"dribbling\")"),
            80
        );
    }

    /// The letters on the save's scale (C 0, B 1, A 2; a stored 3, which
    /// no save holds, reads "3") and the playable order (SS at index 1,
    /// CMF at 6, GK at 12).
    #[test]
    fn rating_at_names_the_letters() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        // 70110, registered SS, stores 2 at indices 1 (SS) and 6 (CMF),
        // 0 elsewhere; give him a 1 at CF (index 0) and a 3 at GK (12).
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == 70110)
            .expect("player 70110");
        assert_eq!(entry.positions.registered, 11);
        assert_eq!(
            entry.positions.playable,
            [0, 2, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0]
        );
        entry.positions.playable[0] = 1;
        entry.positions.playable[12] = 3;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        for (pos, want) in [
            ("CF", "B"),
            ("SS", "A"),
            ("CMF", "A"),
            ("LMF", "C"),
            ("GK", "3"),
        ] {
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

    /// `team.man_marking` is whether any preset's eleven assignments hold
    /// anything but 0xFF: one byte of preset 3 flips it.
    #[test]
    fn man_marking_is_any_presets_assignment() {
        for version in [PesVersion::Pes15, PesVersion::Pes21] {
            let (engine, mut scope, mut file) = with_team(version);
            for preset in &team_mut(&mut file).tactics.presets {
                assert_eq!(preset.man_marking, [0xFF; 11], "{version:?}");
            }
            assert!(
                !engine
                    .eval_with_scope::<bool>(&mut scope, "team.man_marking")
                    .expect("man_marking"),
                "{version:?}: all 0xFF is false"
            );
            team_mut(&mut file).tactics.presets[2].man_marking[4] = 3;
            let team = TeamView::new(&file, 701).expect("the team views");
            scope.push("team", team);
            assert!(
                engine
                    .eval_with_scope::<bool>(&mut scope, "team.man_marking")
                    .expect("man_marking"),
                "{version:?}: one assignment on preset 3 is true"
            );
        }
    }

    /// Each auto value both ways: the stored setting as an integer, the
    /// flags as booleans.
    #[test]
    fn auto_values_answer_the_stored_setting() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes21);
        let auto = &team_mut(&mut file).tactics.auto;
        assert_eq!(auto.substitution, Some(0));
        assert_eq!(auto.offside_trap, Some(false));
        assert_eq!(auto.preset_change, Some(false));
        assert_eq!(auto.attack_defence_levels, Some(false));
        assert_eq!(eval_i64(&engine, &mut scope, "team.auto_substitution"), 0);
        for script in [
            "team.auto_offside_trap",
            "team.auto_preset_change",
            "team.auto_attack_defence_levels",
        ] {
            assert!(
                !engine
                    .eval_with_scope::<bool>(&mut scope, script)
                    .expect(script),
                "{script} off"
            );
        }
        let auto = &mut team_mut(&mut file).tactics.auto;
        auto.substitution = Some(2);
        auto.offside_trap = Some(true);
        auto.preset_change = Some(true);
        auto.attack_defence_levels = Some(true);
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert_eq!(eval_i64(&engine, &mut scope, "team.auto_substitution"), 2);
        for script in [
            "team.auto_offside_trap",
            "team.auto_preset_change",
            "team.auto_attack_defence_levels",
        ] {
            assert!(
                engine
                    .eval_with_scope::<bool>(&mut scope, script)
                    .expect(script),
                "{script} on"
            );
        }
    }

    /// An auto value the version lacks is `()`: attack/defence levels
    /// before PES 17.
    #[test]
    fn auto_values_are_unit_where_the_version_lacks_them() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes15);
        let auto = &team_mut(&mut file).tactics.auto;
        assert_eq!(auto.attack_defence_levels, None);
        assert_eq!(auto.substitution, Some(0));
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.auto_attack_defence_levels")
                .expect("auto_attack_defence_levels")
                .is_unit(),
            "attack/defence levels are () on PES 15"
        );
        assert_eq!(eval_i64(&engine, &mut scope, "team.auto_substitution"), 0);
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
        // A preset outside 1 to 3 is a script error, both ends of the
        // range.
        for preset in [0, 4] {
            assert!(
                engine
                    .eval_with_scope::<Array>(
                        &mut scope,
                        &format!("team.fielded_positions(team.players[8], {preset})")
                    )
                    .is_err(),
                "preset {preset} is a script error"
            );
        }
        // Preset 3 answers too.
        let fielded = engine
            .eval_with_scope::<Array>(&mut scope, "team.fielded_positions(team.players[10], 3)")
            .expect("preset 3");
        assert_eq!(
            fielded
                .into_iter()
                .map(|pos| pos.into_string().expect("a label"))
                .collect::<Vec<_>>(),
            vec!["CF".to_string()]
        );
        let (engine, mut scope, _) = with_team(PesVersion::Pes21);
        assert_eq!(eval_i64(&engine, &mut scope, "team.version"), 21);
    }

    /// Formation slot `i` holds `starting_eleven[i]` — swapping two
    /// entries moves both players' fielded answers with it.
    #[test]
    fn fielded_answers_follow_the_starting_eleven() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        // Premise: slot 3 sits at formation index 1 (CB), slot 1 at
        // index 4 (LB) — different positions.
        assert_eq!(team_mut(&mut file).tactics.starting_eleven[1], 3);
        assert_eq!(team_mut(&mut file).tactics.starting_eleven[4], 1);
        assert_eq!(
            team_mut(&mut file).tactics.presets[0].formations[0].players[1].position,
            1
        );
        assert_eq!(
            team_mut(&mut file).tactics.presets[0].formations[0].players[4].position,
            2
        );
        team_mut(&mut file).tactics.starting_eleven.swap(1, 4);
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        let fielded = engine
            .eval_with_scope::<Array>(&mut scope, "team.fielded_positions(team.players[3], 1)")
            .expect("slot 3");
        assert_eq!(
            fielded
                .into_iter()
                .map(|pos| pos.into_string().expect("a label"))
                .collect::<Vec<_>>(),
            vec!["LB".to_string()]
        );
        assert!(
            engine
                .eval_with_scope::<bool>(
                    &mut scope,
                    "team.fielded_only_at(team.players[3], \"LB\")"
                )
                .expect("fielded_only_at")
        );
        assert!(
            !engine
                .eval_with_scope::<bool>(
                    &mut scope,
                    "team.fielded_only_at(team.players[3], \"CB\")"
                )
                .expect("fielded_only_at")
        );
        let fielded = engine
            .eval_with_scope::<Array>(&mut scope, "team.fielded_positions(team.players[1], 1)")
            .expect("slot 1");
        assert_eq!(
            fielded
                .into_iter()
                .map(|pos| pos.into_string().expect("a label"))
                .collect::<Vec<_>>(),
            vec!["CB".to_string()]
        );
        assert!(
            engine
                .eval_with_scope::<bool>(
                    &mut scope,
                    "team.fielded_only_at(team.players[1], \"CB\")"
                )
                .expect("fielded_only_at")
        );
    }

    /// `position` and `playing_style` are `()` past their tables; an
    /// unknown label is a script error.
    #[test]
    fn unit_and_script_errors_past_the_tables() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == 70111)
            .expect("player 70111");
        // Premises: registered is CF (12), style byte 1 (decodes), and
        // PES 19's 22-style table ends at index 21.
        assert_eq!(entry.positions.registered, 12);
        assert_eq!(entry.positions.playing_style, 1);
        assert!(pes_savefile::schema::playstyle::decode(PesVersion::Pes19, 22).is_err());
        entry.positions.registered = 13;
        entry.positions.playing_style = 22;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.players[10].position")
                .expect("position")
                .is_unit(),
            "registered 13 is past the labels"
        );
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.players[10].playing_style")
                .expect("playing_style")
                .is_unit(),
            "style byte 22 is past PES 19's table"
        );
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.players[10].stat(\"bogus\")")
                .is_err(),
            "an unknown stat is a script error"
        );
        assert!(
            engine
                .eval_with_scope::<bool>(
                    &mut scope,
                    "team.fielded_only_at(team.players[10], \"XX\")"
                )
                .is_err(),
            "an unknown position is a script error"
        );
    }

    #[test]
    fn playing_style_decodes_through_the_versions_table() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == 70111)
            .expect("player 70111");
        // Byte 15 is BuildUp on PES 19's table; it sits at `PlayStyle`'s
        // canonical index 18 — the stored byte and the label index
        // differ.
        assert_eq!(entry.positions.playing_style, 1);
        assert!(
            pes_savefile::schema::playstyle::decode(PesVersion::Pes19, 15)
                .is_ok_and(|style| style == pes_savefile::model::PlayStyle::BuildUp)
        );
        entry.positions.playing_style = 15;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert_eq!(
            engine
                .eval_with_scope::<String>(&mut scope, "team.players[10].playing_style")
                .expect("playing_style"),
            "build_up"
        );
        // The same byte on PES 21's table is OffensiveFullback.
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes21);
        let team = TeamView::new(&file, 701).expect("the team views");
        let slot = usize::from(team.players[10].slot);
        let player_id = file.team(701).expect("team 701").roster[slot].player_id;
        let entry = file
            .players_mut()
            .iter_mut()
            .find(|p| p.id == player_id)
            .expect("the roster slot's player");
        assert_eq!(entry.positions.playing_style, 0);
        assert_eq!(
            pes_savefile::schema::playstyle::decode(PesVersion::Pes21, 15).ok(),
            Some(pes_savefile::model::PlayStyle::OffensiveFullback)
        );
        entry.positions.playing_style = 15;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert_eq!(
            engine
                .eval_with_scope::<String>(&mut scope, "team.players[10].playing_style")
                .expect("playing_style"),
            "offensive_fullback"
        );
    }

    /// A fluid preset reports one map per formation: `preset`,
    /// `formation` and `shape` each.
    #[test]
    fn formation_shapes_reports_each_fielded_formation() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        // Premises: preset 2 is not fluid, and its formation 3 fields
        // AMF (a midfielder) at slot 8.
        assert_eq!(
            team_mut(&mut file).tactics.presets[1].style.fluid,
            Some(false)
        );
        assert_eq!(
            team_mut(&mut file).tactics.presets[1].formations[2].players[8].position,
            8
        );
        team_mut(&mut file).tactics.presets[1].style.fluid = Some(true);
        // Formation 3's AMF becomes a CF: the shape goes [4, 4, 2] →
        // [4, 3, 3].
        team_mut(&mut file).tactics.presets[1].formations[2].players[8].position = 12;
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        let shapes = engine
            .eval_with_scope::<Array>(&mut scope, "team.formation_shapes()")
            .expect("formation_shapes");
        assert_eq!(shapes.len(), 5);
        // Every entry (index, preset, formation, shape): preset 1's kick-off,
        // fluid preset 2's three formations, preset 3's kick-off.
        for (i, preset, formation, want) in [
            (0usize, 1i64, 1i64, vec![4, 4, 2]),
            (1, 2, 1, vec![4, 4, 2]),
            (2, 2, 2, vec![4, 4, 2]),
            (3, 2, 3, vec![4, 3, 3]),
            (4, 3, 1, vec![4, 4, 2]),
        ] {
            let map = shapes[i].clone().cast::<Map>();
            assert_eq!(map["preset"].as_int().expect("preset"), preset, "{i}");
            assert_eq!(
                map["formation"].as_int().expect("formation"),
                formation,
                "{i}"
            );
            let shape: Vec<i64> = map["shape"]
                .clone()
                .into_array()
                .expect("shape")
                .iter()
                .map(|count| count.as_int().expect("a count"))
                .collect();
            assert_eq!(shape, want, "{i}");
        }
    }

    /// `strong_foot` is the foot label or `()` past the two stored
    /// values; UTAH TEAPOT's byte driven through all three.
    #[test]
    fn strong_foot_reads_the_stored_byte() {
        let (engine, mut scope, mut file) = with_team(PesVersion::Pes19);
        let set_foot = |file: &mut EditFile, byte: u8| {
            file.players_mut()
                .iter_mut()
                .find(|p| p.id == 70111)
                .expect("player 70111")
                .positions
                .stronger_foot = byte;
        };
        assert_eq!(
            file.player(70111).expect("70111").positions.stronger_foot,
            0
        );
        for (byte, want) in [(0u8, "right"), (1u8, "left")] {
            set_foot(&mut file, byte);
            let team = TeamView::new(&file, 701).expect("the team views");
            scope.push("team", team);
            assert_eq!(
                engine
                    .eval_with_scope::<String>(&mut scope, "team.players[10].strong_foot")
                    .expect("strong_foot"),
                want,
                "{byte}"
            );
        }
        set_foot(&mut file, 5);
        let team = TeamView::new(&file, 701).expect("the team views");
        scope.push("team", team);
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.players[10].strong_foot")
                .expect("strong_foot")
                .is_unit(),
            "a byte past 1 is ()"
        );
    }

    /// `team.formation(preset, formation)` is the fielded formation's
    /// eleven slots, `#{ slot, position, x, y }` read against the save's
    /// own bytes, `()` for a formation the preset does not field, a
    /// script error for a preset outside 1 to 3.
    #[test]
    fn formation_reads_the_fielded_formation() {
        use pes_savefile::interchange::team_toml::labels::POSITIONS;
        let (engine, mut scope, file) = with_team(PesVersion::Pes19);
        let tactics = &file.team(701).expect("team 701").tactics;
        // Premise: preset 1 is not fluid, so its formation 2 is ()
        // while formation 1 (kick-off) answers.
        assert_eq!(tactics.presets[0].style.fluid, Some(false));
        let kick_off = &tactics.presets[0].formations[0];
        let got = engine
            .eval_with_scope::<Array>(&mut scope, "team.formation(1, 1)")
            .expect("formation");
        assert_eq!(got.len(), 11);
        for (i, entry) in got.iter().enumerate() {
            let map = entry.clone().cast::<Map>();
            assert_eq!(
                map["slot"].as_int().expect("slot"),
                i64::from(tactics.starting_eleven[i]),
                "slot {i}"
            );
            assert_eq!(
                map["position"].clone().into_string().expect("position"),
                POSITIONS[usize::from(kick_off.players[i].position)],
                "slot {i}"
            );
            assert_eq!(
                map["x"].as_int().expect("x"),
                i64::from(kick_off.players[i].x),
                "slot {i}"
            );
            assert_eq!(
                map["y"].as_int().expect("y"),
                i64::from(kick_off.players[i].y),
                "slot {i}"
            );
        }
        assert!(
            engine
                .eval_with_scope::<Dynamic>(&mut scope, "team.formation(1, 2)")
                .expect("formation")
                .is_unit(),
            "a non-fluid preset fields its kick-off formation alone"
        );
        for preset in [0, 4] {
            assert!(
                engine
                    .eval_with_scope::<Dynamic>(&mut scope, &format!("team.formation({preset}, 1)"))
                    .is_err(),
                "preset {preset} is a script error"
            );
        }
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
