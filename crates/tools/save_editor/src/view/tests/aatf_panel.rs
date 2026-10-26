//! The AATF panel's tests (SE-GUI-15; `plans/save_editor.md` "Configurable
//! AATF rules", the AATF panel bullet): the window, the per-player list,
//! the click's selection, suggestions, Validate rules and Save a copy.

use std::path::Path;

use super::*;
use crate::view::aatf_panel::AatfMode;

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

/// The node labeled or valued `label` (a checkbox, radio or weak line).
fn node_named<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.label() == Some(label) || node.value() == Some(label))
        .unwrap_or_else(|| panic!("no node {label:?}"))
}

// SE-GUI-15
#[test]
fn the_panel_lists_the_current_teams_violations_per_player() {
    let (mut view, mut window) = open_734_with("aatf_panel_list", &[73401, 73404], 183);
    // No Checking… assertion here: the menu click's release frame opens the
    // window and starts the check, whose reply can land before the next
    // frame's read; `an_edit_rechecks_and_the_old_result_marks_checking`
    // observes the mark on a frame no reply can reach.
    open_panel(&mut window, &mut view, None);
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
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
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
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
    assert_eq!(
        view.selected,
        Some(Selection::Player(734, 3)),
        "73404's team and slot"
    );
    assert_eq!(view.team, TeamChoice::Team(734), "the selector moved");

    // At ALL it stays. Another of 734's players selected first, or a
    // click that did nothing would satisfy both assertions.
    view.team = TeamChoice::All;
    view.selected = Some(Selection::Player(734, 0));
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        height_row_below(&nodes, "Player 73404  45 Yuki-kun ^~^"),
        &mut view,
    );
    assert_eq!(view.selected, Some(Selection::Player(734, 3)));
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
    let nodes = panel_checked(&mut window, &mut view, None, &[714]);
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
    click_rules(
        &mut window,
        &mut view,
        node_named(&nodes, "Select teams"),
        rules,
    );
    let nodes = draw(&mut window, &mut view, rules);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No teams are picked.")),
        "nothing ticked"
    );
    tick(&mut window, &mut view, rules, &format!("743 {name_743}"));
    tick(&mut window, &mut view, rules, &format!("744 {name_744}"));
    let nodes = panel_checked(&mut window, &mut view, rules, &[743, 744]);
    let failure = nodes
        .iter()
        .filter_map(|node| node.value())
        .find(|value| value.starts_with(&format!("The AATF check failed for team 743 {name_743}:")))
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
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
    let count = height_rows;
    assert_eq!(count(&nodes), 2, "premise: both Height rows");

    // 73401 back to his fixture height through the session, the pointer
    // held on the toolbar's empty end so the recheck waits for the
    // release: a started check could reply before the mark is read.
    let at = egui::pos2(1090.0, 12.0);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = draw(&mut window, &mut view, None);
    assert!(
        view.aatf.check.pending.is_none(),
        "premise: no check started while the pointer is held"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Checking\u{2026}")),
        "the edit marks the old result checking"
    );
    assert_eq!(count(&nodes), 2, "the stale result stays while checking");
    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
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
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
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
            .any(|node| node.value() == Some("There are no findings in the official ruleset.")),
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
        .filter(|node| node.role() == Role::Label)
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
                .is_some_and(|value| value.starts_with("The AATF rules file could not be loaded:"))
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
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
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
    let nodes = panel_checked(&mut window, &mut view, Some(taller.as_path()), &[734]);
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
                .is_some_and(|value| value.starts_with("The AATF rules file could not be loaded:"))
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
    let nodes = panel_checked(&mut window, &mut view, None, &[734]);
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
                value.starts_with(&format!("The AATF check failed for team 743 {name_743}:"))
            })
        })
    };
    let has_all_clear = |nodes: &[accesskit::Node]| {
        nodes
            .iter()
            .any(|node| node.value() == Some("No violations."))
    };

    tick(&mut window, &mut view, rules, &format!("743 {name_743}"));
    let nodes = panel_checked(&mut window, &mut view, rules, &[743]);
    assert!(has_failure_line(&nodes), "the failure line");
    assert!(
        !has_all_clear(&nodes),
        "no `No violations.` over failures only"
    );

    tick(&mut window, &mut view, rules, &format!("714 {name_714}"));
    let nodes = panel_checked(&mut window, &mut view, rules, &[714, 743]);
    assert!(has_failure_line(&nodes), "the failure line beside 714");
    assert!(
        !has_all_clear(&nodes),
        "no `No violations.` beside a failed team, 714 clean"
    );
}
