//! Live AATF's marks on the editor itself (SE-GUI-36; `plans/
//! save_editor.md` "Configurable AATF rules", the Live AATF bullet's
//! marks): the toolbar's framed toggle, the list rows' `✖`/`✔`, and the
//! selected player's field crosses.

use pes_savefile::interchange::team_toml::labels::{ABILITY_STATS, PLAYABLE_POSITIONS};

use crate::view::live_marks;

use super::*;

/// `view` on the PES 17 fixture save under `test`, team 734 open with
/// its first player selected — `open_734_with` without the height edit.
fn open_734(test: &str) -> (SaveView, Headless) {
    let path = fixture(PesVersion::Pes17, test);
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(734);
    view.selected = Some(Selection::Player(734, 0));
    (view, Headless::sized(1100.0, 800.0))
}

/// The painted text shapes whose galley text is exactly `text` (a mark's
/// `✖` or `✔`).
fn mark_shapes<'a>(
    shapes: &'a [egui::epaint::ClippedShape],
    text: &str,
) -> Vec<&'a egui::epaint::TextShape> {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == text => Some(shape),
            _ => None,
        })
        .collect()
}

/// A `Violation` on `slot` of `severity` and `kind`.
fn violation(
    slot: Option<u8>,
    severity: aatf::Severity,
    kind: aatf::ViolationKind,
) -> aatf::Violation {
    aatf::Violation {
        slot,
        severity,
        kind,
    }
}

/// `marked_field` maps each kind to the plan bullet's field, `None` for
/// the row- and team-only kinds (the Live AATF bullet's field
/// sub-bullet, one arm per kind).
#[test]
fn marked_field_maps_every_kind() {
    use aatf::ViolationKind as K;
    use live_marks::MarkedField as F;
    let stat = |name: &str| aatf::Stat::try_from(name.to_owned()).expect("a stat name");
    let position = || aatf::Position::try_from("CB".to_owned()).expect("a position name");
    let cases: &[(K, Option<F>)] = &[
        (K::RegisteredNotA, Some(F::RegisteredPosition)),
        (
            K::FieldedNotA {
                preset: 1,
                position: position(),
            },
            Some(F::Pitch),
        ),
        (K::BRating, Some(F::PlayablePositions)),
        (K::GkSecondA, Some(F::PlayablePositions)),
        (
            K::Age {
                age: 30,
                min: 15,
                max: 50,
            },
            Some(F::Age),
        ),
        (
            K::Weight {
                weight: 80,
                min: 30,
                max: 100,
            },
            Some(F::Weight),
        ),
        (K::PositionRange, Some(F::RegisteredPosition)),
        (K::PlayingStyleRange, Some(F::PlayingStyle)),
        (K::NoTier, None),
        (
            K::Stat {
                stat: stat("speed"),
                value: 99,
                target: 77,
                at_most: false,
            },
            Some(F::Stat(stat("speed"))),
        ),
        (
            K::StatTotal {
                total: 800,
                limit: 700,
            },
            None,
        ),
        (
            K::Form {
                value: 2,
                target: 4,
            },
            Some(F::Form),
        ),
        (
            K::InjuryResistance { value: 2, max: 1 },
            Some(F::InjuryResistance),
        ),
        (
            K::WeakFootUsage { value: 3, max: 2 },
            Some(F::WeakFootUsage),
        ),
        (
            K::WeakFootAccuracy { value: 3, max: 2 },
            Some(F::WeakFootAccuracy),
        ),
        (
            K::NameColor {
                color: Some("cc9900ff".to_owned()),
                expected: None,
            },
            Some(F::Name),
        ),
        (K::Cards { paid: 4, limit: 3 }, Some(F::CardsLeft)),
        (K::MinCards { paid: 1, min: 3 }, Some(F::CardsLeft)),
        (K::SkillCardCap { count: 11, max: 10 }, Some(F::Skills)),
        (K::ComStyleCap { count: 3, max: 2 }, Some(F::ComStyles)),
        (
            K::RequiredCard {
                card: aatf::Card::Skill(0),
            },
            Some(F::Skills),
        ),
        (
            K::RequiredCard {
                card: aatf::Card::ComStyle(1),
            },
            Some(F::ComStyles),
        ),
        (
            K::APositions { count: 2, max: 1 },
            Some(F::PlayablePositions),
        ),
        (K::Height { height: 183 }, Some(F::Height)),
        (
            K::TierCount {
                tier: "gold".to_owned(),
                count: 0,
                expected: 1,
            },
            None,
        ),
        (
            K::BracketQuota {
                system: "red".to_owned(),
                bracket: "tall".to_owned(),
                count: 8,
                quota: 10,
            },
            None,
        ),
        (
            K::MedalSystem {
                systems: vec!["sys".to_owned()],
            },
            None,
        ),
        (K::ExtraHeight { used: 5, total: 4 }, None),
        (K::Captain, None),
        (K::Goalkeepers { count: 0, min: 1 }, None),
        (
            K::TeamSetting {
                label: "x".to_owned(),
                setting: "y".to_owned(),
            },
            None,
        ),
        (
            K::FormationShape {
                preset: 1,
                formation: 2,
                shape: [4, 4, 2],
            },
            None,
        ),
        (
            K::Special {
                label: "x".to_owned(),
                text: "y".to_owned(),
            },
            None,
        ),
        (
            K::Custom {
                text: "y".to_owned(),
            },
            None,
        ),
        (
            K::Unused {
                allowance: "cards".to_owned(),
                used: 1,
                allowed: 2,
            },
            None,
        ),
        (
            K::UnusedChoice {
                special: "x".to_owned(),
            },
            None,
        ),
        (K::RosterOrder { player: 73401 }, None),
    ];
    for (kind, field) in cases {
        assert_eq!(live_marks::marked_field(kind), *field, "{kind:?}");
    }
}

/// `row_mark` counts the slot's errors and warnings — a zero count left
/// out, suggestions and the other slots' rows counted nowhere — and a
/// row with no Error passes (the draft's `rowMark`).
#[test]
fn row_mark_counts_the_slots_errors_and_warnings() {
    use aatf::Severity::*;
    let v = |slot, severity| violation(slot, severity, aatf::ViolationKind::Captain);
    let cases: &[(&[aatf::Violation], &str, bool)] = &[
        (&[v(Some(1), Error)], "1 error", true),
        (
            &[v(Some(1), Error), v(Some(1), Error), v(Some(1), Warning)],
            "2 errors, 1 warning",
            true,
        ),
        (&[v(Some(1), Warning)], "1 warning", false),
        (&[v(Some(1), Suggestion)], "No errors", false),
        (&[], "No errors", false),
    ];
    for (violations, tooltip, failing) in cases {
        let mark = live_marks::row_mark(violations, Some(1));
        assert_eq!(
            (mark.tooltip.as_str(), mark.failing),
            (*tooltip, *failing),
            "{violations:?}"
        );
    }
    // The team's own violations (`slot: None`) mark the team row alone,
    // and a player's his alone.
    let mark = live_marks::row_mark(&[v(Some(1), Error), v(None, Warning)], None);
    assert_eq!((mark.tooltip.as_str(), mark.failing), ("1 warning", false));
    let mark = live_marks::row_mark(
        &[v(None, Error), v(None, Warning), v(Some(1), Error)],
        Some(1),
    );
    assert_eq!((mark.tooltip.as_str(), mark.failing), ("1 error", true));
}

/// `held_violations` is the held check only while the toggle is on, the
/// rules file loaded and the result one team's `Ok` — a failed check,
/// another shape of result or nothing yet marks nothing (the Live AATF
/// bullet's last sub-bullet).
#[test]
fn held_violations_is_one_teams_ok_result_while_on() {
    use std::sync::Arc;
    let mut check = aatf_check::TeamsCheck::new();
    let key = |ids: &[u32]| aatf_check::CheckKey {
        revision: 1,
        ids: ids.to_vec(),
        rules_file: None,
    };
    let rules = ActiveRules {
        file: None,
        rules: Ok(Arc::new(aatf::Rules::official())),
    };
    let broken = ActiveRules {
        file: None,
        rules: Err("bad".to_owned()),
    };
    let held = |check: &_, rules: Option<&_>| {
        live_marks::held_violations(true, check, rules).map(|(id, v)| (id, v.len()))
    };
    check.result = Some((
        key(&[734]),
        vec![Ok(vec![violation(
            None,
            aatf::Severity::Error,
            aatf::ViolationKind::Captain,
        )])],
    ));
    assert_eq!(held(&check, Some(&rules)), Some((734, 1)));
    assert_eq!(
        live_marks::held_violations(false, &check, Some(&rules)),
        None,
        "off marks nothing"
    );
    assert_eq!(
        held(&check, Some(&broken)),
        None,
        "a rules file that does not load marks nothing"
    );
    assert_eq!(held(&check, None), None, "no ruleset yet");
    check.result = Some((key(&[734]), vec![Err("boom".to_owned())]));
    assert_eq!(held(&check, Some(&rules)), None, "a failed check");
    check.result = Some((key(&[734, 735]), vec![Ok(vec![]), Ok(vec![])]));
    assert_eq!(held(&check, Some(&rules)), None, "not one team's");
    check.result = None;
    assert_eq!(held(&check, Some(&rules)), None, "nothing held yet");
}

/// The toggle is a framed `✔ Live AATF` button; the list marks the
/// broken player's row `✖` and a clean one's `✔`; the selected player's
/// field the violation is about carries a cross whose tooltip is the
/// violation's own line; fixing it clears the marks; off, none are
/// drawn.
// SE-GUI-36
#[test]
fn the_marks_follow_the_held_check() {
    let (mut view, mut window) = open_734_with("live_marks_gui36", &[73401], 183);
    let visuals = window.ctx.style_of(window.ctx.theme()).visuals.clone();

    // The toolbar's toggle is a framed button: the inactive fill off,
    // the selection fill on.
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let toggle = node_labeled(&nodes, Role::Button, "\u{2714} Live AATF");
    let bounds = toggle.bounds().expect("the toggle's bounds");
    assert_eq!(
        fill_at(&shapes, bounds),
        Some(visuals.widgets.inactive.weak_bg_fill),
        "the off toggle is a framed button"
    );
    click(&mut window, toggle, &mut view);
    assert!(view.live_aatf, "the click turned Live AATF on");
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        fill_at(&shapes, bounds),
        Some(visuals.selection.bg_fill),
        "the on toggle takes the selection fill"
    );

    // The first check returned: `✖` on the broken player's row in the
    // error colour, `✔` on the clean one's in the success colour, at
    // each row's right end, the tooltips the counts.
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    for (name, glyph, color, tip) in [
        ("Muh Spooks", "\u{2716}", visuals.error_fg_color, "1 error"),
        (
            "Yuki-kun ^~^",
            "\u{2714}",
            studio_core::theme::notice_color(studio_core::NoticeKind::Success, &visuals),
            "No errors",
        ),
    ] {
        let row = nodes
            .iter()
            .find(|node| player_row(node) && node.label().is_some_and(|label| label.contains(name)))
            .unwrap_or_else(|| panic!("{name}'s row"));
        let bounds = row.bounds().expect("the row's bounds");
        let mark = mark_shapes(&shapes, glyph)
            .into_iter()
            .find(|shape| {
                f64::from(shape.pos.y) >= bounds.y0
                    && f64::from(shape.pos.y) <= bounds.y1
                    && f64::from(shape.pos.x) >= bounds.x1 - 30.0
                    && f64::from(shape.pos.x) <= bounds.x1
            })
            .unwrap_or_else(|| panic!("no {glyph} on {name}'s row"));
        assert_eq!(galley_color(mark), color, "{name}'s mark's colour");
        let tip_at = egui::pos2(mark.pos.x + 2.0, mark.pos.y + 2.0);
        assert_eq!(
            tooltip_containing(&mut window, &mut view, tip_at, tip).as_deref(),
            Some(tip),
            "{name}'s mark's tooltip"
        );
    }

    // The broken player selected: one `✖` in the view, left of the
    // Height box and inside the General info column (the list's marks
    // sit left of it at x < 260); its tooltip the violation's own line.
    let height_box = control_right_of(&nodes, "Height")
        .bounds()
        .expect("the Height box's bounds");
    let skills = label_bounds(&nodes, "Skills");
    let crosses: Vec<_> = mark_shapes(&shapes, "\u{2716}")
        .into_iter()
        .filter(|shape| f64::from(shape.pos.x) > 260.0)
        .collect();
    let [cross] = crosses.as_slice() else {
        panic!(
            "expected one field cross in the view, got {}",
            crosses.len()
        )
    };
    assert!(
        f64::from(cross.pos.x + cross.galley.rect.width()) <= height_box.x0 + 0.5,
        "the cross stands just left of the Height box"
    );
    assert!(
        f64::from(cross.pos.x) < skills.x0,
        "the cross is inside the General info column"
    );
    let tip_at = egui::pos2(cross.pos.x + 2.0, cross.pos.y + 2.0);
    assert_eq!(
        tooltip_containing(&mut window, &mut view, tip_at, "takes no bracket").as_deref(),
        Some("Height: height 183 takes no bracket"),
        "the cross's tooltip is the violation's line"
    );

    // The Height box's x is the same with Live AATF off, and no mark is
    // drawn anywhere then.
    view.live_aatf = false;
    let (_, nodes_off, _) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let off_box = control_right_of(&nodes_off, "Height")
        .bounds()
        .expect("the Height box's bounds");
    assert_eq!(off_box.x0, height_box.x0, "the box keeps its x");
    let (_, _, shapes_off) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        mark_shapes(&shapes_off, "\u{2716}").is_empty(),
        "no cross with Live AATF off"
    );
    assert!(
        mark_shapes(&shapes_off, "\u{2714}").is_empty(),
        "no check mark with Live AATF off"
    );

    // The height fixed and the check returned: both rows `✔`, no cross
    // in the view.
    view.live_aatf = true;
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    // The players' rows are `✔`, and no field carries a cross any
    // longer (the team row's `✖` stays: the tier counts are the team's
    // own, broken from the start).
    assert!(
        mark_shapes(&shapes, "\u{2716}")
            .into_iter()
            .all(|shape| shape.pos.x < 260.0),
        "no field cross once the height is fixed"
    );
    for name in ["Muh Spooks", "Yuki-kun ^~^"] {
        let row = nodes
            .iter()
            .find(|node| player_row(node) && node.label().is_some_and(|label| label.contains(name)))
            .unwrap_or_else(|| panic!("{name}'s row"));
        let bounds = row.bounds().expect("the row's bounds");
        assert!(
            mark_shapes(&shapes, "\u{2714}").into_iter().any(|shape| {
                f64::from(shape.pos.y) >= bounds.y0
                    && f64::from(shape.pos.y) <= bounds.y1
                    && f64::from(shape.pos.x) >= bounds.x1 - 30.0
            }),
            "a `✔` on {name}'s row"
        );
    }
}

/// A `Stat` violation's cross stands just left of the stat's box in the
/// Ability stats column (the Live AATF bullet: a stat for `Stat`).
#[test]
fn a_stats_cross_stands_left_of_its_box() {
    let (mut view, mut window) = open_734("live_marks_stat");
    view.live_aatf = true;
    let mut entry = player_entry(&view, 73401);
    let speed = ABILITY_STATS
        .iter()
        .position(|name| *name == "speed")
        .expect("speed is an ability stat");
    // One over the regulars' 77 target (still regular: a medal's
    // recognition threshold is higher): a `Stat` error on speed alone.
    entry.stats.set_ability(speed, 78);
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let speed_box = control_right_of(&nodes, "Speed")
        .bounds()
        .expect("Speed's box");
    let cross = mark_shapes(&shapes, "\u{2716}")
        .into_iter()
        .find(|shape| {
            let right = f64::from(shape.pos.x + shape.galley.rect.width());
            ((right + f64::from(live_marks::MARK_GAP)) - speed_box.x0).abs() < 0.5
                && f64::from(shape.pos.y) >= speed_box.y0 - 2.0
                && f64::from(shape.pos.y) <= speed_box.y1 + 2.0
        })
        .expect("a cross MARK_GAP left of Speed's box");
    assert_eq!(
        galley_color(cross),
        window
            .ctx
            .style_of(window.ctx.theme())
            .visuals
            .error_fg_color,
        "the error colour"
    );
}

/// A `BRating` violation's cross stands after the Playable positions
/// header (the grid for `BRating`, `GkSecondA` and `APositions`).
#[test]
fn a_positions_violation_marks_the_grid_header() {
    let (mut view, mut window) = open_734("live_marks_positions");
    view.live_aatf = true;
    let mut entry = player_entry(&view, 73401);
    let unrated = entry
        .positions
        .playable
        .iter()
        .position(|rating| *rating == 0)
        .expect("a C-rated position");
    entry.positions.playable[unrated] = 1;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let header = label_bounds(&nodes, "Playable positions");
    assert!(
        mark_shapes(&shapes, "\u{2716}").into_iter().any(|shape| {
            f64::from(shape.pos.x) >= header.x1
                && f64::from(shape.pos.x) <= header.x1 + 30.0
                && f64::from(shape.pos.y) >= header.y0 - 4.0
                && f64::from(shape.pos.y) <= header.y1 + 4.0
        }),
        "a cross after the Playable positions header"
    );
}

/// A `SkillCardCap` violation's cross stands after the Skills header
/// (the skills for `SkillCardCap` and a required skill card).
#[test]
fn a_skills_violation_marks_the_header() {
    let (mut view, mut window) = open_734("live_marks_skills");
    view.live_aatf = true;
    let mut entry = player_entry(&view, 73401);
    entry
        .skills
        .skills
        .iter_mut()
        .take(11)
        .for_each(|held| *held = true);
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    // The header cross is the `✖` label node right of the "Skills" one
    // (a wrapped row's painted positions read as its origin, so the node
    // locates it).
    let header = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some("Skills"))
        .expect("the Skills header");
    assert!(
        first_right_of(&nodes, header, |node| {
            node.role() == Role::Label && node.value() == Some("\u{2716}")
        })
        .is_some(),
        "a cross after the Skills header"
    );
}

/// A `RegisteredNotA` violation's cross stands after the Registered
/// position label, before its combo (a registered position he is not
/// rated A at).
#[test]
fn a_registered_violation_marks_the_combo_label() {
    let (mut view, mut window) = open_734("live_marks_registered");
    view.live_aatf = true;
    let mut entry = player_entry(&view, 73401);
    let unrated = entry
        .positions
        .playable
        .iter()
        .position(|rating| *rating == 0)
        .expect("a C-rated position");
    entry.positions.registered =
        u8::try_from(PLAYABLE_POSITIONS[unrated]).expect("a position fits u8");
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let label = label_bounds(&nodes, "Registered position");
    let combo = control_right_of(&nodes, "Registered position")
        .bounds()
        .expect("the combo's bounds");
    assert!(
        mark_shapes(&shapes, "\u{2716}").into_iter().any(|shape| {
            f64::from(shape.pos.x) >= label.x1 - 4.0
                && f64::from(shape.pos.x + shape.galley.rect.width()) <= combo.x0 + 1.0
                && f64::from(shape.pos.y) >= label.y0 - 4.0
                && f64::from(shape.pos.y) <= label.y1 + 4.0
        }),
        "a cross after the Registered position label, before its combo"
    );
}

/// A warning-only violation's cross draws in the warning colour: the
/// official file gains a warning band on the regular tier, and 73401's
/// attacking prowess goes one over its 77 target (a `Stat` warning, not
/// an error).
#[test]
fn a_warnings_cross_draws_in_the_warning_colour() {
    let (mut view, mut window) = open_734("live_marks_warning");
    view.live_aatf = true;
    let dir = view
        .session
        .as_ref()
        .expect("the save opened")
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let rules = dir.join("warn.4ccrules");
    std::fs::write(
        &rules,
        aatf::official_file().replacen("cards: 3,", "stat_warning: 5, cards: 3,", 1),
    )
    .unwrap();
    let mut entry = player_entry(&view, 73401);
    let prowess = ABILITY_STATS
        .iter()
        .position(|name| *name == "attacking_prowess")
        .expect("attacking_prowess is an ability stat");
    entry.stats.set_ability(prowess, 78);
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, Some(rules.as_path()), &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules.as_path()))
    });
    assert!(
        view.live.check.result.as_ref().is_some_and(|(_, checks)| {
            checks[0].as_ref().is_ok_and(|violations| {
                violations.iter().any(|violation| {
                    matches!(violation.kind, aatf::ViolationKind::Stat { .. })
                        && violation.severity == aatf::Severity::Warning
                })
            })
        }),
        "premise: the check holds a Stat warning"
    );
    let prowess_box = control_right_of(&nodes, "Attacking Prowess")
        .bounds()
        .expect("Attacking prowess' box");
    let cross = mark_shapes(&shapes, "\u{2716}")
        .into_iter()
        .find(|shape| {
            let right = f64::from(shape.pos.x + shape.galley.rect.width());
            ((right + f64::from(live_marks::MARK_GAP)) - prowess_box.x0).abs() < 0.5
                && f64::from(shape.pos.y) >= prowess_box.y0 - 2.0
                && f64::from(shape.pos.y) <= prowess_box.y1 + 2.0
        })
        .expect("a cross MARK_GAP left of Attacking prowess' box");
    assert_eq!(
        galley_color(cross),
        window
            .ctx
            .style_of(window.ctx.theme())
            .visuals
            .warn_fg_color,
        "the warning colour"
    );
}

/// The team's own violations (`slot: None`) mark the team row — 734's
/// three tier counts are the team's own, broken from the start; a team
/// with none shows `✔` (the Live AATF bullet's last list sub-bullet).
#[test]
fn the_teams_own_violations_mark_the_team_row() {
    let (mut view, mut window) = open_734("live_marks_team_row");
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let row = team_row(&nodes, &view, 734);
    let bounds = row.bounds().expect("the team row's bounds");
    let mark = mark_shapes(&shapes, "\u{2716}")
        .into_iter()
        .find(|shape| {
            f64::from(shape.pos.y) >= bounds.y0
                && f64::from(shape.pos.y) <= bounds.y1
                && f64::from(shape.pos.x) >= bounds.x1 - 30.0
        })
        .expect("a `✖` on the team row");
    let tip_at = egui::pos2(mark.pos.x + 2.0, mark.pos.y + 2.0);
    assert_eq!(
        tooltip_containing(&mut window, &mut view, tip_at, "3 errors").as_deref(),
        Some("3 errors"),
        "the team's own count: the three tier counts"
    );

    // A clean team's row passes (the imported 714 holds only
    // suggestions, which count as neither).
    let (mut view, mut window) =
        open_legal_714("live_marks_team_row_clean", Headless::sized(1100.0, 800.0));
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[714]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let row = team_row(&nodes, &view, 714);
    let bounds = row.bounds().expect("the team row's bounds");
    assert!(
        mark_shapes(&shapes, "\u{2714}").into_iter().any(|shape| {
            f64::from(shape.pos.y) >= bounds.y0
                && f64::from(shape.pos.y) <= bounds.y1
                && f64::from(shape.pos.x) >= bounds.x1 - 30.0
        }),
        "a `✔` on the clean team's row"
    );
    assert!(
        mark_shapes(&shapes, "\u{2716}").is_empty(),
        "no cross anywhere"
    );
}

/// A click on a row's mark still selects the row: the mark senses hover
/// alone, so the click lands on the row (the Live AATF bullet's).
#[test]
fn a_click_on_a_rows_mark_selects_the_row() {
    let (mut view, mut window) = open_734_with("live_marks_mark_click", &[73401], 183);
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    // Another player selected first: a click that did nothing would
    // satisfy the assertion anyway.
    view.selected = Some(Selection::Player(734, 3));
    let row = nodes
        .iter()
        .find(|node| {
            player_row(node)
                && node
                    .label()
                    .is_some_and(|label| label.contains("Muh Spooks"))
        })
        .expect("his row");
    let bounds = row.bounds().expect("the row's bounds");
    let mark = mark_shapes(&shapes, "\u{2716}")
        .into_iter()
        .find(|shape| {
            f64::from(shape.pos.y) >= bounds.y0
                && f64::from(shape.pos.y) <= bounds.y1
                && f64::from(shape.pos.x) >= bounds.x1 - 30.0
        })
        .expect("the `✖` on his row");
    click_at(
        &mut window,
        egui::pos2(
            mark.pos.x + mark.galley.rect.width() / 2.0,
            mark.pos.y + mark.galley.rect.height() / 2.0,
        ),
        &mut view,
    );
    assert_eq!(
        view.selected,
        Some(Selection::Player(734, 0)),
        "the click on the mark selected his row"
    );
}

/// A name too long for the list truncates before the row's mark (the
/// Live AATF bullet's: the text never runs under it).
#[test]
fn a_long_name_truncates_before_the_rows_mark() {
    let path = fixture(PesVersion::Pes21, "live_marks_long_name");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().name = "W".repeat(40);
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    view.selected = Some(Selection::Player(701, 0));
    view.live_aatf = true;
    // The held result injected for team 701, clean: every row passes
    // and carries the `✔` the name truncates before.
    let revision = view.session.as_ref().expect("the save opened").revision();
    view.live.check.result = Some((
        aatf_check::CheckKey {
            revision,
            ids: vec![701],
            rules_file: None,
        },
        vec![Ok(Vec::new())],
    ));
    let mut window = Headless::new();
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let row = nodes
        .iter()
        .find(|node| player_row(node) && node.label().is_some_and(|label| label.contains("WWWW")))
        .expect("his row");
    let bounds = row.bounds().expect("the row's bounds");
    let mark = mark_shapes(&shapes, "\u{2714}")
        .into_iter()
        .find(|shape| {
            f64::from(shape.pos.y) >= bounds.y0
                && f64::from(shape.pos.y) <= bounds.y1
                && f64::from(shape.pos.x) >= bounds.x1 - 30.0
        })
        .expect("the mark on his row");
    let name_shape = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text)
                if text.galley.job.text.contains("WWWW")
                    && f64::from(text.pos.y) >= bounds.y0
                    && f64::from(text.pos.y) <= bounds.y1 =>
            {
                Some(text)
            }
            _ => None,
        })
        .next()
        .expect("the row's name galley");
    assert!(name_shape.galley.elided, "the name is elided");
    let name_right = f64::from(name_shape.pos.x + name_shape.galley.rect.width());
    assert!(
        name_right <= f64::from(mark.pos.x) + 0.5,
        "the name ends before the mark ({name_right} vs {})",
        mark.pos.x
    );
}

/// A held check carrying an Error on the selected player for every field
/// General info and Other stats mark: no cross's painted rect intersects
/// another text's, and every box and combo keeps its x with the marks
/// drawn (the decision entry: the field keeps its x).
#[test]
fn no_cross_overlaps_text_and_no_field_moves() {
    use aatf::ViolationKind as K;
    let (mut view, mut window) = open_734("live_marks_no_overlap");
    view.live_aatf = true;
    let error = |kind| violation(Some(0), aatf::Severity::Error, kind);
    let violations = vec![
        error(K::Height { height: 183 }),
        error(K::Age {
            age: 30,
            min: 15,
            max: 50,
        }),
        error(K::Weight {
            weight: 80,
            min: 30,
            max: 100,
        }),
        error(K::Form {
            value: 2,
            target: 4,
        }),
        error(K::InjuryResistance { value: 2, max: 1 }),
        error(K::WeakFootUsage { value: 3, max: 2 }),
        error(K::WeakFootAccuracy { value: 3, max: 2 }),
        error(K::NameColor {
            color: Some("cc9900ff".to_owned()),
            expected: None,
        }),
        error(K::RegisteredNotA),
        error(K::PlayingStyleRange),
    ];
    let revision = view.session.as_ref().expect("the save opened").revision();
    view.live.check.result = Some((
        aatf_check::CheckKey {
            revision,
            ids: vec![734],
            rules_file: None,
        },
        vec![Ok(violations)],
    ));
    let (_, nodes_mark, shapes_mark) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let crosses: Vec<_> = mark_shapes(&shapes_mark, "\u{2716}")
        .into_iter()
        .filter(|shape| f64::from(shape.pos.x) > 250.0)
        .collect();
    assert_eq!(crosses.len(), 10, "one cross per marked field");

    // No cross touches another text's galley; the narrowest clearance
    // between a cross and any text sharing its row is reported. The
    // view's texts start at x 256 (the player list's end is 248).
    let texts: Vec<&egui::epaint::TextShape> = shapes_mark
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(text)
                if text.galley.job.text != "\u{2716}" && f64::from(text.pos.x) > 250.0 =>
            {
                Some(text)
            }
            _ => None,
        })
        .collect();
    let mut clearance = f32::MAX;
    for cross in &crosses {
        let cross_rect = egui::Rect::from_min_size(cross.pos, cross.galley.rect.size());
        for text in &texts {
            let text_rect = egui::Rect::from_min_size(text.pos, text.galley.rect.size());
            assert!(
                !cross_rect.intersects(text_rect),
                "the cross at {:?} overlaps {:?} at {text_rect:?}",
                cross.pos,
                text.galley.job.text
            );
            if cross_rect.min.y < text_rect.max.y && text_rect.min.y < cross_rect.max.y {
                clearance = clearance.min(
                    (text_rect.min.x - cross_rect.max.x).max(cross_rect.min.x - text_rect.max.x),
                );
            }
        }
    }
    assert!(
        clearance >= live_marks::MARK_GAP,
        "a cross {clearance} points from a text, under MARK_GAP"
    );

    // Every marked field's box or combo keeps its x with Live AATF off.
    view.live_aatf = false;
    let (_, nodes_off, _) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    for label in [
        "Height",
        "Shirt number",
        "Weight",
        "Nation",
        "Age",
        "Captain",
        "Injury resistance",
        "Form",
        "Weak foot: Usage",
        "Accuracy",
        "Stronger foot",
        "Stronger hand",
        "Registered position",
        "Playing style",
    ] {
        let marked = control_right_of(&nodes_mark, label)
            .bounds()
            .expect("the control's bounds");
        let off = control_right_of(&nodes_off, label)
            .bounds()
            .expect("the control's bounds");
        assert_eq!(marked.x0, off.x0, "{label}'s control kept its x");
    }
    for field in [name_field, shirt_field] {
        let marked = field(&nodes_mark).bounds().expect("the field's bounds");
        let off = field(&nodes_off).bounds().expect("the field's bounds");
        assert_eq!(marked.x0, off.x0, "the counted field kept its x");
    }
}

/// A check that fails marks nothing: the dock reports the failure and no
/// `✖` or `✔` is drawn (the Live AATF bullet's last sub-bullet).
#[test]
fn a_failed_check_marks_nothing() {
    let path = fixture(PesVersion::Pes17, "live_marks_failed");
    let dir = path.parent().expect("the scratch dir").to_path_buf();
    let rules = throw19_file(&dir);
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(743);
    view.selected = Some(Selection::Player(743, 0));
    view.live_aatf = true;
    let mut window = Headless::sized(1100.0, 800.0);
    let nodes = live_checked(&mut window, &mut view, Some(rules.as_path()), &[743]);
    assert!(
        nodes.iter().any(|node| {
            node.value()
                .is_some_and(|v| v.contains("The AATF check failed"))
        }),
        "premise: the dock reports the failed check"
    );
    let (_, _, shapes) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules.as_path()))
    });
    assert!(mark_shapes(&shapes, "\u{2716}").is_empty(), "no cross");
    assert!(mark_shapes(&shapes, "\u{2714}").is_empty(), "no check mark");
}
