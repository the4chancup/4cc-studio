//! The AATF panel's tests (SE-GUI-15; `plans/save_editor.md` "Configurable
//! AATF rules", the AATF panel bullet): the window, the per-player list,
//! the click's selection, suggestions, Validate rules and Save a copy.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::*;
use crate::view::aatf_check::{CheckKey, PendingCheck};
use crate::view::aatf_panel::AatfMode;

/// `checked` on the panel's check.
fn checked(
    window: &mut Headless,
    view: &mut SaveView,
    rules: Option<&Path>,
    ids: &[u32],
) -> Vec<accesskit::Node> {
    super::checked(window, view, rules, |view| &view.aatf.check, ids)
}

/// `node` clicked with `rules` as the active rules file.
fn click_rules(
    window: &mut Headless,
    view: &mut SaveView,
    node: &accesskit::Node,
    rules: Option<&Path>,
) {
    let at = centre(node);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, view, &|| Vec::new(), rules)
    });
    window.frame(pointer_at(at, false), |ui| {
        show(ui, view, &|| Vec::new(), rules)
    });
}

/// The panel opened from the toolbar's Panels menu: the "AATF
/// violations…" item clicked, the next frame's nodes returned.
fn open_panel(
    window: &mut Headless,
    view: &mut SaveView,
    rules: Option<&Path>,
) -> Vec<accesskit::Node> {
    let nodes = draw(window, view, rules);
    click_rules(
        window,
        view,
        node_labeled(&nodes, Role::Button, "Panels"),
        rules,
    );
    let nodes = draw(window, view, rules);
    click_rules(
        window,
        view,
        node_labeled(&nodes, Role::Button, "AATF violations\u{2026}"),
        rules,
    );
    draw(window, view, rules)
}

/// The CheckBox labeled `label` toggled through its AccessKit action: a
/// team the picker scroll has not reached sits outside the window, where
/// a pointer click lands on nothing.
fn tick(window: &mut Headless, view: &mut SaveView, rules: Option<&Path>, label: &str) {
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), rules));
    let (id, _) = pairs
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("no checkbox {label:?}"));
    let id = *id;
    window.frame(
        vec![egui::Event::AccessKitActionRequest(
            accesskit::ActionRequest {
                action: accesskit::Action::Click,
                target_tree: accesskit::TreeId::ROOT,
                target_node: id,
                data: None,
            },
        )],
        |ui| show(ui, view, &|| Vec::new(), rules),
    );
}

/// The node labeled or valued `label` (a checkbox, radio or weak line).
fn node_named<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.label() == Some(label) || node.value() == Some(label))
        .unwrap_or_else(|| panic!("no node {label:?}"))
}

/// The nodes inside the panel's window (its subtree: the player list's
/// rows share the screen). `pairs` is `frame_ids`' output.
fn panel_nodes(pairs: &[(accesskit::NodeId, accesskit::Node)]) -> Vec<&accesskit::Node> {
    let (window, _) = pairs
        .iter()
        .find(|(_, node)| {
            node.label()
                .is_some_and(|label| label.starts_with("AATF violations"))
        })
        .expect("the panel window");
    let by_id: HashMap<accesskit::NodeId, &accesskit::Node> =
        pairs.iter().map(|(id, node)| (*id, node)).collect();
    let mut nodes = Vec::new();
    let mut stack = vec![*window];
    while let Some(id) = stack.pop() {
        let node = by_id[&id];
        stack.extend(node.children());
        nodes.push(node);
    }
    nodes
}

/// The panel window's Label values and Button labels, in top-to-bottom
/// order.
fn panel_lines(window: &mut Headless, view: &mut SaveView, rules: Option<&Path>) -> Vec<String> {
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), rules));
    let nodes = panel_nodes(&pairs);
    let mut lines: Vec<(f64, f64, String)> = Vec::new();
    for node in nodes {
        let text = match node.role() {
            Role::Label => node.value().map(str::to_owned),
            _ => node.label().map(str::to_owned),
        };
        if let (Some(text), Some(bounds)) = (text, node.bounds()) {
            lines.push((bounds.y0, bounds.x0, text));
        }
    }
    lines.sort_by(|a, b| {
        a.0.partial_cmp(&b.0)
            .expect("drawn bounds")
            .then(a.1.partial_cmp(&b.1).expect("drawn bounds"))
    });
    lines.into_iter().map(|(_, _, text)| text).collect()
}

/// `dir`'s `throw19.rhai` written and its path returned: the official
/// file with the CLI's custom check that throws on captain 19 (team
/// 743's; `a_check_that_fails_on_one_team_does_not_stop_the_rest`).
fn throw19_file(dir: &Path) -> PathBuf {
    let path = dir.join("throw19.rhai");
    std::fs::write(
        &path,
        aatf::official_file().replacen(
            "fn custom_checks(team) { [] }",
            "fn custom_checks(team) { if team.captain == 19 { throw \"boom\" } [] }",
            1,
        ),
    )
    .unwrap();
    path
}

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

// SE-GUI-15
#[test]
fn the_panel_lists_the_current_teams_violations_per_player() {
    let (mut view, mut window) = open_734_with("aatf_panel_list", &[73401, 73404], 183);
    let nodes = open_panel(&mut window, &mut view, None);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Checking\u{2026}")),
        "the frame that starts the check marks it"
    );
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.label() == Some("AATF violations (Autumn 26)")),
        "the window's title"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "the team's header"
    );
    let heights: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && node.label() == Some("Height height 183 takes no bracket")
        })
        .collect();
    assert_eq!(heights.len(), 2, "the two players' Height rows");
    assert!(
        !nodes.iter().any(|node| node.value() == Some("Suggestions")),
        "Show suggestions is off"
    );
    assert!(
        !nodes
            .iter()
            .any(|node| node.value() == Some("No violations.")),
        "the team has violations"
    );
}

// SE-GUI-15
#[test]
fn a_rows_click_selects_the_player_and_moves_the_selector() {
    let (mut view, mut window) = open_734_with("aatf_panel_click", &[73401, 73404], 183);
    view.aatf.open = true;
    view.aatf.mode = AatfMode::Select;
    tick(&mut window, &mut view, None, "734 /lit/");
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "734's rows are up"
    );

    // The selector on another team: the click selects the player and
    // moves the selector to his.
    view.team = TeamChoice::Team(718);
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        height_row_below(&nodes, "Player 73404  45 Yuki-kun ^~^"),
        &mut view,
    );
    assert_eq!(view.selected, Some((734, 3)), "73404's team and slot");
    assert_eq!(view.team, TeamChoice::Team(734), "the selector moved");

    // At ALL it stays. Another of 734's players selected first, or a
    // click that did nothing would satisfy both assertions.
    view.team = TeamChoice::All;
    view.selected = Some((734, 0));
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        height_row_below(&nodes, "Player 73404  45 Yuki-kun ^~^"),
        &mut view,
    );
    assert_eq!(view.selected, Some((734, 3)));
    assert_eq!(view.team, TeamChoice::All, "ALL is kept");
}

// SE-GUI-15
#[test]
fn show_suggestions_reveals_the_unused_allowances() {
    // No team of the PES 17 fixture save is clean under the official
    // ruleset (every one carries the TierCount errors): the legal team is
    // the CLI's `official_legal.toml` on team 714 of the PES 15 save,
    // whose 32 findings are all suggestions.
    let (mut view, mut window) =
        open_legal_714("aatf_panel_suggestions", Headless::sized(1100.0, 800.0));
    view.aatf.open = true;
    let nodes = checked(&mut window, &mut view, None, &[714]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No violations.")),
        "the imported team is clean"
    );
    assert!(
        !nodes.iter().any(|node| node.value() == Some("Suggestions")),
        "no suggestions group while off"
    );
    click(
        &mut window,
        node_named(&nodes, "Show suggestions"),
        &mut view,
    );
    let lines = panel_lines(&mut window, &mut view, None);
    let at = lines
        .iter()
        .position(|line| line == "Suggestions")
        .expect("the Suggestions header");
    // The imported team's first group: slot 1's player header, then his
    // Unused rows (the fixture team's own names head each group).
    assert!(
        lines[at + 1].starts_with("Player 714"),
        "a player's group header: {:?}",
        &lines[at..at + 3]
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Unused only 1 of 2 tricks used"),
        "the first Unused row: {lines:?}"
    );
}

/// Select teams checks the ticked teams; a team whose check fails is a
/// line naming the team and the error, the others still listed (the
/// throw19 rules file of the CLI's
/// `a_check_that_fails_on_one_team_does_not_stop_the_rest`).
// SE-GUI-15
#[test]
fn select_teams_lists_the_picked_teams_and_a_failed_ones_line() {
    let save = fixture(PesVersion::Pes15, "aatf_select_fail");
    let dir = save.parent().expect("the scratch dir").to_path_buf();
    let rules_path = throw19_file(&dir);
    let mut view = SaveView::new();
    view.open(&save);
    view.aatf.open = true;
    let rules = Some(rules_path.as_path());
    let mut window = Headless::sized(1100.0, 800.0);
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

    let nodes = draw(&mut window, &mut view, rules);
    click(&mut window, node_named(&nodes, "Select teams"), &mut view);
    let nodes = draw(&mut window, &mut view, rules);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No team picked.")),
        "nothing ticked"
    );
    tick(&mut window, &mut view, rules, &format!("743 {name_743}"));
    tick(&mut window, &mut view, rules, &format!("744 {name_744}"));
    let nodes = checked(&mut window, &mut view, rules, &[743, 744]);
    let failure = nodes
        .iter()
        .filter_map(|node| node.value())
        .find(|value| value.starts_with(&format!("Team 743 {name_743}: the AATF check failed:")))
        .unwrap_or_else(|| panic!("743's failure line: {nodes:?}"));
    assert!(failure.contains("boom"), "{failure}");
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some(&format!("Team 744 {name_744}"))),
        "744's header"
    );
}

/// An edit and an undo each restart the check, the previous result
/// marked Checking… until the new one lands.
// SE-GUI-15
#[test]
fn an_edit_rechecks_and_the_old_result_marks_checking() {
    let (mut view, mut window) = open_734_with("aatf_panel_recheck", &[73401, 73404], 183);
    view.aatf.open = true;
    let nodes = checked(&mut window, &mut view, None, &[734]);
    let count = height_rows;
    assert_eq!(count(&nodes), 2, "premise: both Height rows");

    // 73401 back to his fixture height through the session.
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = draw(&mut window, &mut view, None);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Checking\u{2026}")),
        "the frame that starts the recheck marks it"
    );
    assert_eq!(count(&nodes), 2, "the stale result stays while checking");
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert_eq!(count(&nodes), 1, "73401's row is gone, 73404's stays");
    // The row left is 73404's.
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Player 73404  45 Yuki-kun ^~^")),
        "73404's group"
    );
    assert!(
        !nodes
            .iter()
            .any(|node| node.value() == Some("Player 73401  49 Muh Spooks")),
        "73401's group is gone"
    );

    // An undo brings the edit, the check and the row back.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert_eq!(count(&nodes), 2, "the undo restores the row");
}

/// Validate rules: the official ruleset reports no findings; a broken
/// file lists each finding on its own line under the file's name.
// SE-GUI-15
#[test]
fn validate_rules_reports_the_files_findings() {
    let save = fixture(PesVersion::Pes17, "aatf_validate");
    let dir = save.parent().expect("the scratch dir").to_path_buf();
    let mut view = SaveView::new();
    view.open(&save);
    view.aatf.open = true;
    let mut window = Headless::sized(1100.0, 800.0);

    // The embedded official ruleset.
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Validate rules"),
        &mut view,
    );
    let nodes = draw(&mut window, &mut view, None);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("the official ruleset: no findings")),
        "the official file loads"
    );

    // The official file with two values broken: the tier counts over the
    // squad size and an unknown suggestion word.
    let two = dir.join("two.rhai");
    std::fs::write(
        &two,
        aatf::official_file()
            .replacen("squad_size: 23", "squad_size: 4", 1)
            .replacen("suggestions: [\"cards\"", "suggestions: [\"bogus\"", 1),
    )
    .unwrap();
    let nodes = draw(&mut window, &mut view, Some(two.as_path()));
    click_rules(
        &mut window,
        &mut view,
        node_labeled(&nodes, Role::Button, "Validate rules"),
        Some(two.as_path()),
    );
    let nodes = draw(&mut window, &mut view, Some(two.as_path()));
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some(&format!("{}:", two.display()))),
        "the file's name over the findings"
    );
    for line in [
        "tiers: counts total 5 over squad_size 4",
        "suggestions[0]: unknown suggestion \"bogus\"",
    ] {
        assert!(
            nodes.iter().any(|node| node.value() == Some(line)),
            "{line:?} drawn"
        );
    }

    // A file that does not parse: one line with its position.
    let broken = dir.join("broken.rhai");
    std::fs::write(&broken, "let x = ;").unwrap();
    let nodes = draw(&mut window, &mut view, Some(broken.as_path()));
    click_rules(
        &mut window,
        &mut view,
        node_labeled(&nodes, Role::Button, "Validate rules"),
        Some(broken.as_path()),
    );
    let nodes = draw(&mut window, &mut view, Some(broken.as_path()));
    let line = nodes
        .iter()
        .filter_map(|node| node.value())
        .find(|value| value.starts_with("1:"))
        .unwrap_or_else(|| panic!("the parse error's line: {nodes:?}"));
    assert_eq!(line, "1:9: Unexpected ';' (line 1, position 9)");
}

/// A rules file that does not load: the load error stands where the list
/// would, and no check runs.
// SE-GUI-15
#[test]
fn a_rules_file_that_does_not_load_shows_the_error_in_the_lists_place() {
    let save = fixture(PesVersion::Pes17, "aatf_broken_rules");
    let dir = save.parent().expect("the scratch dir").to_path_buf();
    let broken = dir.join("broken.rhai");
    std::fs::write(&broken, "let x = ;").unwrap();
    let mut view = SaveView::new();
    view.open(&save);
    view.aatf.open = true;
    let mut window = Headless::sized(1100.0, 800.0);
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(broken.as_path()))
    });
    let nodes = panel_nodes(&pairs);
    assert!(
        nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.starts_with("The AATF rules file does not load:"))
        }),
        "the load error's line"
    );
    assert!(
        !nodes
            .iter()
            .any(|node| { node.value().is_some_and(|value| value.starts_with("Team ")) }),
        "no team listed"
    );
}

/// Another rules file replaces the completed list's rows: the check's
/// key carries the file, so the new file's check runs and its rows
/// stand where the old ones were; a file that does not load draws its
/// load error there instead.
// SE-GUI-15
#[test]
fn a_rules_file_change_replaces_the_result() {
    let (mut view, mut window) = open_734_with("aatf_rules_switch", &[73401, 73404], 183);
    view.aatf.open = true;
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 2, "the official file's Height rows");
    assert!(
        nodes.iter().any(|node| {
            node.value() == Some("BracketQuota 8 players in bracket tall of red, quota 10")
        }),
        "the official file's quota row"
    );

    // A file that loads with a different result: the official one's
    // Tall bracket also taking 183, so the two rows are gone and the
    // two new talls fill the red bracket's quota.
    let taller = dir.join("taller.rhai");
    std::fs::write(
        &taller,
        aatf::official_file().replacen("exactly: [185]", "exactly: [183, 185]", 1),
    )
    .unwrap();
    let nodes = checked(&mut window, &mut view, Some(taller.as_path()), &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "the new file's result drew the team"
    );
    assert_eq!(
        height_rows(&nodes),
        0,
        "183 takes the Tall bracket now: the new rows replaced the old"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.starts_with("BracketQuota"))
        }),
        "the red quota met: its row is gone too"
    );

    // A file that does not load: its error in the list's place, and no
    // row of either result.
    let broken = dir.join("broken.rhai");
    std::fs::write(&broken, "let x = ;").unwrap();
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(broken.as_path()))
    });
    let nodes = panel_nodes(&pairs);
    assert!(
        nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.starts_with("The AATF rules file does not load:"))
        }),
        "the load error's line"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.label()
                .is_some_and(|label| label.starts_with("Height ") || label.starts_with("Team "))
                || node.value().is_some_and(|value| {
                    value.starts_with("Team ")
                        || value.starts_with("Height ")
                        || value.starts_with("BracketQuota")
                        || value.starts_with("TierCount")
                })
        }),
        "no row of either result is drawn"
    );
}

/// Save a copy: the embedded official file written out under its own name
/// through the stubbed dialog.
// SE-GUI-15
#[test]
fn save_a_copy_writes_the_embedded_official_file() {
    let save = fixture(PesVersion::Pes17, "aatf_save_copy");
    let dir = save.parent().expect("the scratch dir").to_path_buf();
    let target = dir.join("copy.rhai");
    let mut view = SaveView::new();
    view.open(&save);
    view.aatf.open = true;
    let for_dialog = target.clone();
    view.aatf_copy_dialog = Box::new(move |name| {
        assert_eq!(name, aatf::OFFICIAL_FILE_NAME, "the suggested name");
        Some(for_dialog.clone())
    });
    let mut window = Headless::sized(1100.0, 800.0);
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Save a copy"),
        &mut view,
    );
    let bytes = std::fs::read(&target).expect("the copy written");
    assert_eq!(
        bytes,
        aatf::official_file().as_bytes(),
        "the embedded file, byte for byte"
    );
    let rules = aatf::Rules::load(&String::from_utf8(bytes).expect("the file is text"))
        .expect("the copy loads");
    assert_eq!(rules.ruleset().name, "Autumn 26");
    assert_eq!(rules.logic(), aatf::LogicIdentity::Current);
    let nodes = draw(&mut window, &mut view, None);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some(&format!("Wrote {}", target.display()))),
        "the Wrote line"
    );
}

/// The results scroll (the rework's scroll fix): the clean team's 32
/// suggestion rows run past the window's bottom; a wheel over the list
/// brings the last one in.
#[test]
fn the_results_scroll_reaches_the_last_row() {
    let (mut view, mut window) = open_legal_714("aatf_panel_scroll", Headless::sized(900.0, 600.0));
    view.aatf.open = true;
    view.aatf.show_suggestions = true;
    let _ = checked(&mut window, &mut view, None, &[714]);

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
    let mut entry = player_entry(&view, first);
    entry.basic.height += 1;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
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
    let mut window = Headless::sized(1100.0, 800.0);
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
    let results_top = named("No team picked.")
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

/// A click clears the player list's filter: the selection is only worth
/// it if the row it lands on can show.
// SE-GUI-15
#[test]
fn a_rows_click_clears_the_filter_so_the_row_shows() {
    let (mut view, mut window) = open_734_with("aatf_click_filter", &[73401, 73404], 183);
    view.aatf.open = true;
    view.aatf.mode = AatfMode::Select;
    view.filter = "no player named this".to_owned();
    tick(&mut window, &mut view, None, "734 /lit/");
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert!(
        row_labels(&nodes).is_empty(),
        "premise: the filter hides every player"
    );
    click(
        &mut window,
        height_row_below(&nodes, "Player 73404  45 Yuki-kun ^~^"),
        &mut view,
    );
    assert!(view.filter.is_empty(), "the click cleared the filter");
    let nodes = draw(&mut window, &mut view, None);
    assert!(
        row_labels(&nodes)
            .iter()
            .any(|label| label.contains("Yuki-kun")),
        "the selected player's row shows: {:?}",
        row_labels(&nodes)
    );
}

/// `No violations.` needs every check to have succeeded: beside a failed
/// team's line it would read as clearing that team, whose violations are
/// unknown. 743 fails alone, then beside the clean 714.
// SE-GUI-15
#[test]
fn no_violations_needs_every_check_to_succeed() {
    let (mut view, mut window) = open_legal_714("aatf_fail_only", Headless::sized(1100.0, 800.0));
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let rules_path = throw19_file(&dir);
    view.aatf.open = true;
    view.aatf.mode = AatfMode::Select;
    let rules = Some(rules_path.as_path());
    let file = view.session.as_ref().unwrap().file();
    let name_714 = file.team(714).unwrap().name.clone();
    let name_743 = file.team(743).unwrap().name.clone();
    let has_failure_line = |nodes: &[accesskit::Node]| {
        nodes.iter().any(|node| {
            node.value().is_some_and(|value| {
                value.starts_with(&format!("Team 743 {name_743}: the AATF check failed:"))
            })
        })
    };
    let has_all_clear = |nodes: &[accesskit::Node]| {
        nodes
            .iter()
            .any(|node| node.value() == Some("No violations."))
    };

    tick(&mut window, &mut view, rules, &format!("743 {name_743}"));
    let nodes = checked(&mut window, &mut view, rules, &[743]);
    assert!(has_failure_line(&nodes), "the failure line");
    assert!(
        !has_all_clear(&nodes),
        "no `No violations.` over failures only"
    );

    tick(&mut window, &mut view, rules, &format!("714 {name_714}"));
    let nodes = checked(&mut window, &mut view, rules, &[714, 743]);
    assert!(has_failure_line(&nodes), "the failure line beside 714");
    assert!(
        !has_all_clear(&nodes),
        "no `No violations.` beside a failed team, 714 clean"
    );
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
    let copy = dir.join("copy.rhai");
    view.aatf_copy_dialog = Box::new(move |_| Some(copy.clone()));
    let rules = Some(rules_path.as_path());
    let mut window = Headless::sized(1100.0, 800.0);
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
    let _ = checked(&mut window, &mut view, rules, &[743, 744]);
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
        lines.iter().any(|line| line == "No team picked."),
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
    let nodes = checked(&mut window, &mut view, None, &[734]);
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
    let nodes = checked(&mut window, &mut view, None, &[734]);
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
    let nodes = checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 1, "73401's row is gone, 73404's stays");
}
