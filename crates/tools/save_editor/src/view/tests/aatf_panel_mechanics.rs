//! The floating AATF panel's mechanics tests (regressions and worker
//! behaviour, no scenario): the results' and the team picker's own
//! scrolls, a dead worker's re-check, the unsettled edit's hold, and
//! another save's reset.

use std::path::Path;

use super::*;
use crate::view::aatf_check::{CheckKey, PendingCheck};
use crate::view::aatf_panel::AatfMode;

/// The panel window's bounds and the last `Unused` row's.
fn panel_bounds(
    window: &mut Headless,
    view: &mut SaveView,
    rules: Option<&Path>,
) -> (accesskit::Rect, accesskit::Rect) {
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), rules));
    let window_bounds = pairs
        .iter()
        .find(|(_, node)| {
            node.label()
                .is_some_and(|label| label.starts_with("AATF violations"))
        })
        .and_then(|(_, node)| node.bounds())
        .expect("the panel window's bounds");
    let last = panel_nodes(&pairs)
        .iter()
        .filter(|node| {
            node.label()
                .is_some_and(|label| label.starts_with("Unused"))
        })
        .filter_map(|node| node.bounds())
        .max_by(|a, b| a.y0.partial_cmp(&b.y0).expect("ordered bounds"))
        .expect("an Unused row's bounds");
    (window_bounds, last)
}

/// The results scroll (the rework's scroll fix): the clean team's 32
/// suggestion rows run past the window's bottom; a wheel over the list
/// brings the last one in.
#[test]
fn the_results_scroll_reaches_the_last_row() {
    let (mut view, mut window) =
        open_legal_714("aatf_panel_scroll", Headless::sized(DEFAULT_PANE.x, 600.0));
    view.aatf.open = true;
    view.aatf.show_suggestions = true;
    let _ = panel_checked(&mut window, &mut view, None, &[714]);

    let (window_bounds, last) = panel_bounds(&mut window, &mut view, None);
    assert!(
        last.y0 > window_bounds.y1,
        "the last Unused row starts past the window's bottom: {last:?} vs {window_bounds:?}"
    );

    // A wheel over the results brings it inside, a few frames at most.
    let mut last = last;
    for _ in 0..10 {
        let at = egui::pos2(
            ((window_bounds.x0 + window_bounds.x1) / 2.0) as f32,
            (window_bounds.y1 - 20.0) as f32,
        );
        window.frame(
            vec![
                egui::Event::PointerMoved(at),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -400.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Wheel,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
        let (_, bounds) = panel_bounds(&mut window, &mut view, None);
        last = bounds;
        if last.y1 <= window_bounds.y1 {
            break;
        }
    }
    assert!(
        last.y1 <= window_bounds.y1,
        "the scroll reached the last row: {last:?} vs {window_bounds:?}"
    );

    // An edit while scrolled to the bottom: Checking… stays in view, above
    // the scroll, so the rows below read as the previous result's.
    let first = view
        .session
        .as_ref()
        .unwrap()
        .file()
        .team(714)
        .unwrap()
        .roster[0]
        .player_id;
    // The pointer held on the toolbar's empty end, so the recheck waits
    // for its release: a started check could reply before the mark is
    // read.
    window.frame(pointer_at(egui::pos2(890.0, 12.0), true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let mut entry = player_entry(&view, first);
    entry.basic.height += 1;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        view.aatf.check.pending.is_none(),
        "premise: no check started while the pointer is held"
    );
    let checking = panel_nodes(&pairs)
        .into_iter()
        .find(|node| node.value() == Some("Checking\u{2026}"))
        .and_then(|node| node.bounds())
        .expect("Checking… drawn after the edit");
    assert!(
        checking.y0 >= window_bounds.y0 && checking.y1 <= window_bounds.y1,
        "Checking… inside the window while scrolled: {checking:?} vs {window_bounds:?}"
    );
}

/// The team picker's scroll is its own scroll area's, not shared with
/// the results': both sit in the window's `Ui`, where egui's default
/// `scroll_area` id makes them one state — the short results clamp the
/// picker's offset to 0 each frame. A wheel over the picker reaches the
/// last team of the save.
#[test]
fn the_picker_scroll_reaches_the_last_team() {
    let save = fixture(PesVersion::Pes17, "aatf_picker_scroll");
    let mut view = SaveView::new();
    view.open(&save);
    view.aatf.open = true;
    view.aatf.mode = AatfMode::Select;
    let mut window = Headless::sized(DEFAULT_PANE.x, 800.0);
    let (first_label, last_label) = {
        let teams = view
            .session
            .as_ref()
            .expect("the save opened")
            .file()
            .teams();
        (
            format!("{} {}", teams[0].id, teams[0].name),
            format!(
                "{} {}",
                teams[teams.len() - 1].id,
                teams[teams.len() - 1].name
            ),
        )
    };

    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let nodes = panel_nodes(&pairs);
    let named = |label: &str| -> &accesskit::Node {
        nodes
            .iter()
            .copied()
            .find(|node| node.label() == Some(label) || node.value() == Some(label))
            .unwrap_or_else(|| panic!("no node {label:?}"))
    };
    let at = centre(named(&first_label));
    // The picker's clip: from the first checkbox's top to the "No team
    // picked." line, the results area's first content just below it.
    let picker_top = named(&first_label)
        .bounds()
        .expect("the first checkbox's bounds")
        .y0;
    let results_top = named("No teams are picked.")
        .bounds()
        .expect("the picked line's bounds")
        .y0;
    assert!(
        named(&last_label)
            .bounds()
            .expect("the last checkbox's bounds")
            .y0
            > results_top,
        "premise: the last team starts past the picker: {last_label:?}"
    );

    let mut last = None;
    for _ in 0..40 {
        let (_, pairs, _) = window.frame_ids(
            vec![
                egui::Event::PointerMoved(at),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -400.0),
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Wheel,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
        let nodes = panel_nodes(&pairs);
        let bounds = nodes
            .iter()
            .copied()
            .find(|node| node.label() == Some(last_label.as_str()))
            .and_then(|node| node.bounds())
            .expect("the last checkbox's bounds");
        if bounds.y0 >= picker_top && bounds.y1 <= results_top {
            return;
        }
        last = Some(bounds);
    }
    panic!("the picker's scroll never reached {last_label:?}: {last:?}")
}

/// Another save starts the panel afresh (`SaveView::open`): picks,
/// results, the pending check, the validation and the `Wrote` line go
/// with the old save; open, mode and Show suggestions stay.
#[test]
fn another_save_starts_the_panel_afresh() {
    let save15 = fixture(PesVersion::Pes15, "aatf_open_reset_15");
    let dir = save15.parent().expect("the scratch dir").to_path_buf();
    let rules_path = throw19_file(&dir);
    let mut view = SaveView::new();
    view.open(&save15);
    view.aatf.open = true;
    view.aatf.mode = AatfMode::Select;
    view.aatf.show_suggestions = true;
    let copy = dir.join("copy.4ccrules");
    view.aatf_copy_dialog = Box::new(move |_| Some(copy.clone()));
    let rules = Some(rules_path.as_path());
    let mut window = Headless::sized(DEFAULT_PANE.x, 800.0);
    let names = |id: u32| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .team(id)
            .unwrap()
            .name
            .clone()
    };
    let (name_743, name_744) = (names(743), names(744));
    tick(&mut window, &mut view, rules, &format!("743 {name_743}"));
    tick(&mut window, &mut view, rules, &format!("744 {name_744}"));
    let _ = panel_checked(&mut window, &mut view, rules, &[743, 744]);
    let nodes = draw(&mut window, &mut view, rules);
    click_rules(
        &mut window,
        &mut view,
        node_labeled(&nodes, Role::Button, "Validate rules"),
        rules,
    );
    let nodes = draw(&mut window, &mut view, rules);
    click_rules(
        &mut window,
        &mut view,
        node_labeled(&nodes, Role::Button, "Save a copy"),
        rules,
    );
    let _ = draw(&mut window, &mut view, rules);
    assert!(
        view.aatf.check.result.is_some() && !view.aatf.picked.is_empty(),
        "premise: a check ran and teams were picked"
    );

    let save17 = fixture(PesVersion::Pes17, "aatf_open_reset_17");
    view.open(&save17);
    assert!(view.aatf.picked.is_empty(), "the picks went with the save");
    assert!(
        view.aatf.check.result.is_none() && view.aatf.check.pending.is_none(),
        "the result and its check went with the save"
    );
    assert!(
        view.aatf.open && view.aatf.mode == AatfMode::Select && view.aatf.show_suggestions,
        "the view choices stay"
    );
    let lines = panel_lines(&mut window, &mut view, None);
    assert!(
        lines.iter().any(|line| line == "No teams are picked."),
        "nothing picked on the new save: {lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.contains("no findings") || line.starts_with("Wrote")),
        "the validation and copy lines went: {lines:?}"
    );
}

/// A worker that ends without replying: `show` drops the dead check's
/// `pending` and the next settled frame starts a fresh one.
#[test]
fn a_worker_that_ends_without_a_reply_is_checked_again() {
    let (mut view, mut window) = open_734_with("aatf_dead_worker", &[73401, 73404], 183);
    view.aatf.open = true;
    // A pending whose sender is already gone: the worker died.
    let (sender, replies) = crossbeam_channel::unbounded();
    drop(sender);
    view.aatf.check.pending = Some(PendingCheck {
        key: CheckKey {
            revision: view.session.as_ref().expect("the save opened").revision(),
            ids: vec![734],
            rules_file: None,
        },
        replies,
    });
    // The dead pending is dropped and a fresh check for the key starts.
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "a fresh check ran"
    );
}

/// An unsettled edit marks the last result `Checking…` and starts no
/// check until the gesture ends (`due` draws the mark, `settled` gates
/// the spawn).
#[test]
fn an_unsettled_edit_marks_checking_and_holds_the_check() {
    let (mut view, mut window) = open_734_with("aatf_unsettled", &[73401, 73404], 183);
    view.aatf.open = true;
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 2, "premise: both Height rows");

    // The press lands on an empty spot of the central panel; the edit
    // lands while the pointer stays down.
    let at = egui::pos2(700.0, 780.0);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);

    for _ in 0..2 {
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        assert!(
            nodes
                .iter()
                .any(|node| node.value() == Some("Checking\u{2026}")),
            "marked as checking while held"
        );
        assert!(
            view.aatf.check.pending.is_none(),
            "no check started while held"
        );
        assert_eq!(height_rows(&nodes), 2, "the stale rows stay");
    }
    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        view.aatf.check.pending.is_some(),
        "the check starts on the release frame"
    );
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 1, "73401's row is gone, 73404's stays");
}
