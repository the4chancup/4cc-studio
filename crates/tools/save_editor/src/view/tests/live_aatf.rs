//! The Live AATF dock's tests (SE-GUI-30; `plans/save_editor.md`
//! "Configurable AATF rules", the Live AATF bullet): the docked list
//! shrinking and growing back with the open team's edits, the `No issues`
//! header, the remembered toggle, the checking mark, ALL's team and a
//! row's click.

use std::path::Path;

use super::*;

/// `checked` on the dock's check.
fn live_checked(
    window: &mut Headless,
    view: &mut SaveView,
    rules: Option<&Path>,
    ids: &[u32],
) -> Vec<accesskit::Node> {
    super::checked(window, view, rules, |view| &view.live.check, ids)
}

/// A click on `node` through `tool.view` (press on one frame, release on
/// the next): the `live_aatf` write lives in `Tool::view`, not `show`.
fn tool_click(
    window: &mut Headless,
    tool: &mut crate::Tool,
    ctx: &studio_core::ToolContext,
    node: &accesskit::Node,
) {
    use studio_core::StudioTool;
    press_release(window, centre(node), |ui| tool.view(ui, ctx));
}

/// Asserts no violation row is drawn under `header` (a row would be a
/// Button below the header line; a gated control's `-` stand-in — drawn
/// off the scroll's clip with real bounds — is never one).
fn assert_no_violation_row(nodes: &[accesskit::Node], header: &str) {
    let header_bottom = nodes
        .iter()
        .find(|node| node.value() == Some(header))
        .and_then(accesskit::Node::bounds)
        .expect("the header's bounds")
        .y1;
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node.label() != Some("-")
                && node.bounds().is_some_and(|b| b.y0 >= header_bottom)
        }),
        "no violation row"
    );
}

/// The dock lists the open team's violations under `N issues · <ruleset>`;
/// an edit fixing one drops its row and N by one, an undo brings both back.
// SE-GUI-30
#[test]
fn the_list_shrinks_as_edits_fix_violations_and_grows_back_on_undo() {
    let (mut view, mut window) = open_734_with("live_aatf_shrink", &[73401, 73404], 183);
    view.live_aatf = true;
    // Before the first result: `Checking <team>…`, no count yet. The
    // pointer is held (on the toolbar's empty end), so no check starts: a
    // started one can reply between a frame's two passes, and the frame
    // would draw its count instead.
    let at = egui::pos2(1090.0, 12.0);
    let (_, nodes, _) = window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Checking /lit/\u{2026}")),
        "the first check's header"
    );
    assert!(
        view.live.check.pending.is_none() && view.live.check.result.is_none(),
        "premise: no check started while the pointer is held"
    );
    assert!(
        !nodes.iter().any(|node| node
            .value()
            .is_some_and(|v| v.ends_with("issues · Autumn 26"))),
        "no count before the first result"
    );
    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 2, "both players' Height rows");
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("6 issues · Autumn 26")),
        "the count header"
    );

    // 73401 back to his fixture height through the session (one sealed
    // edit): his Height row is gone and the count is one less.
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 1, "73401's Height row is gone");
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("5 issues · Autumn 26")),
        "the smaller count"
    );

    // An undo brings the edit, the row and the count back.
    view.session.as_mut().expect("the save opened").undo();
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert_eq!(height_rows(&nodes), 2, "the row is back");
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("6 issues · Autumn 26")),
        "the count is back"
    );
}

/// A team that passes reads `<team> passes <ruleset> with no issues.` and
/// draws no violation row; a player's height edit then reads the check's
/// count.
// SE-GUI-30
#[test]
fn a_clean_team_reads_no_issues_and_a_broken_rule_the_count() {
    let (mut view, mut window) = open_legal_714("live_aatf_clean", Headless::sized(1100.0, 800.0));
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[714]);
    let name = view
        .session
        .as_ref()
        .expect("the save opened")
        .file()
        .team(714)
        .expect("team 714")
        .name
        .clone();
    let header = format!("{name} passes Autumn 26 with no issues.");
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some(header.as_str())),
        "the passing header: {header}"
    );
    assert_no_violation_row(&nodes, &header);

    // One player's height broken: the count header reads the check's
    // count.
    let first = view
        .session
        .as_ref()
        .expect("the save opened")
        .file()
        .team(714)
        .expect("team 714")
        .roster[0]
        .player_id;
    let mut entry = player_entry(&view, first);
    entry.basic.height = 183;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let nodes = live_checked(&mut window, &mut view, None, &[714]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("3 issues · Autumn 26")),
        "the count after the edit"
    );

    // The last one fixed: `No issues` again, and no row is drawn.
    view.session.as_mut().expect("the save opened").undo();
    let nodes = live_checked(&mut window, &mut view, None, &[714]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some(header.as_str())),
        "the passing header is back"
    );
    assert_no_violation_row(&nodes, &header);
}

/// The toggle is a setting: clicking it writes `live_aatf`, the tool's
/// other keys untouched, and a fresh `Tool` on the same settings draws the
/// dock on its first view frame; toggled back, the dock is gone.
// SE-GUI-30
#[test]
fn the_toggle_is_remembered_in_the_settings() {
    use studio_core::StudioTool;

    let save_path = fixture(PesVersion::Pes17, "live_aatf_remembered");
    let dir = save_path.parent().expect("the scratch dir").to_path_buf();
    let mut tool = crate::Tool::new();
    tool.view.open(&save_path);
    let mut settings = studio_core::Settings::default();
    let mut table = tool.default_settings();
    table.insert("unread_key".to_owned(), toml::Value::Integer(3));
    settings.set_tool("save-editor", table);
    let (ctx, _events_rx, _requests_rx) = tool_context(&dir, settings);
    let mut window = Headless::sized(1100.0, 700.0);

    // The toggle clicked: the setting holds true, the unread key stands.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    tool_click(
        &mut window,
        &mut tool,
        &ctx,
        node_labeled(&nodes, Role::Button, "Live AATF"),
    );
    let table = ctx.tool_settings("save-editor");
    assert_eq!(
        table["live_aatf"].as_bool(),
        Some(true),
        "the toggle set the setting"
    );
    assert_eq!(
        table["unread_key"].as_integer(),
        Some(3),
        "the tool's other key is unchanged"
    );

    // A fresh tool on the same context: the dock drawn on the first view
    // frame.
    let mut tool2 = crate::Tool::new();
    tool2.view.open(&save_path);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool2.view(ui, &ctx));
    assert!(
        nodes.iter().any(|node| node.value() == Some("Live AATF")),
        "the dock's header on the first frame"
    );

    // Toggled back: the setting is false and the dock is gone.
    tool_click(
        &mut window,
        &mut tool2,
        &ctx,
        node_labeled(&nodes, Role::Button, "Live AATF"),
    );
    assert_eq!(
        ctx.tool_settings("save-editor")["live_aatf"].as_bool(),
        Some(false),
        "the toggle wrote false"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool2.view(ui, &ctx));
    assert!(
        !nodes.iter().any(|node| node.value() == Some("Live AATF")),
        "the dock is gone"
    );
}

/// An edit marks the previous result `checking…`, its count still shown,
/// until the new check's result replaces it. The edit lands while the
/// pointer is held, so the check waits for the release: a started check
/// can reply between a frame's two passes, before the mark is read.
// SE-GUI-30
#[test]
fn an_edit_marks_the_previous_result_checking() {
    let (mut view, mut window) = open_734_with("live_aatf_checking", &[73401, 73404], 183);
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("6 issues · Autumn 26")),
        "premise: the count header"
    );

    // The press lands on the toolbar's empty end.
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
        view.live.check.pending.is_none(),
        "premise: no check started while the pointer is held"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("checking\u{2026}")),
        "the checking mark beside the old header"
    );
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("6 issues · Autumn 26")),
        "the previous count still shown"
    );
    assert_eq!(height_rows(&nodes), 2, "the stale rows stay while checking");

    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("5 issues · Autumn 26")),
        "the new count replaces it"
    );
    assert!(
        !nodes
            .iter()
            .any(|node| node.value() == Some("checking\u{2026}")),
        "and the mark is gone"
    );
}

/// The dock follows the open team and the rules file: another team picked
/// re-checks and re-counts it, and a rules file whose result differs
/// replaces the rows.
// SE-GUI-30
#[test]
fn the_dock_follows_the_open_team_and_the_rules_file() {
    let (mut view, mut window) = open_734_with("live_aatf_team_rules", &[73401, 73404], 183);
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("6 issues · Autumn 26")),
        "premise: 734's count"
    );

    // Another team picked, as the selector does: the dock re-checks it.
    view.team = TeamChoice::Team(714);
    view.selected = Some(Selection::Player(714, 0));
    let nodes = live_checked(&mut window, &mut view, None, &[714]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("3 issues · Autumn 26")),
        "714's count"
    );
    assert!(
        !nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "no row of 734's"
    );

    // Back to 734 under a rules file whose result differs: the Tall
    // bracket also takes 183, so the two Height rows go and the red
    // quota is met.
    view.team = TeamChoice::Team(734);
    view.selected = Some(Selection::Player(734, 0));
    let dir = view
        .session
        .as_ref()
        .unwrap()
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf();
    let taller = dir.join("taller.rhai");
    std::fs::write(
        &taller,
        aatf::official_file().replacen("exactly: [185]", "exactly: [183, 185]", 1),
    )
    .unwrap();
    let nodes = live_checked(&mut window, &mut view, Some(taller.as_path()), &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("3 issues · Autumn 26")),
        "the new file's count"
    );
    assert_eq!(height_rows(&nodes), 0, "183 takes the Tall bracket");
}

/// At ALL the dock checks the selected player's team; with no player
/// selected it reads `No team is open.` and starts no check.
// SE-GUI-30
#[test]
fn at_all_the_dock_checks_the_selected_players_team() {
    let (mut view, mut window) = open_734_with("live_aatf_all", &[73401], 183);
    view.live_aatf = true;
    view.team = TeamChoice::All;
    view.selected = Some(Selection::Player(734, 0));
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "734's rows under ALL"
    );

    view.selected = None;
    let nodes = draw(&mut window, &mut view, None);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No team is open.")),
        "the empty selection's line"
    );
    assert!(view.live.check.pending.is_none(), "no check started");
    assert!(
        !nodes
            .iter()
            .any(|node| node.value() == Some("Team 734 /lit/")),
        "the stale team's rows are not drawn"
    );
}

/// SE-GUI-30's current-team rule: the checks follow the selected row's
/// team, another team's too when the filter lists its rows — the selector
/// at team 701 with `QUALITY` typed lists 70202 of team 702 (`list.rs`'
/// filter test), and his row clicked makes both checks his team's.
// SE-GUI-30
#[test]
fn the_checks_follow_the_selected_rows_team_under_a_filter() {
    // Team 702's players made tall enough to take no bracket, so the check
    // lists violations; 70202 named as `list.rs`' filter test's (the PES 21
    // fixture's players are all PLACEHOLDER).
    let path = fixture(PesVersion::Pes21, "live_aatf_filtered_team");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    for id in [70201u32, 70202] {
        file.player_mut(id).unwrap().basic.height = 183;
    }
    file.player_mut(70202).unwrap().name = "QUALITY".to_owned();
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    view.live_aatf = true;
    view.aatf.open = true;
    view.aatf.mode = crate::view::aatf_panel::AatfMode::Current;
    let mut window = Headless::sized(1100.0, 800.0);

    // The filter lists 70202 of team 702 across teams; clicking his row
    // selects him.
    view.filter = "quality".to_owned();
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (target, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("2 CB /a/ QUALITY"))
        .expect("the filtered row");
    action_click(&mut window, &mut view, *target);
    assert!(
        matches!(view.selected, Some(Selection::Player(702, _))),
        "70202's row selected: {:?}",
        view.selected
    );

    // The dock and the panel's current-team mode each check team 702 (the
    // keyed results land only for the team each caller picked), each read
    // inside its own widget's nodes: the other draws the same header, so
    // a frame-wide scan can be met by the wrong one's copy.
    live_checked(&mut window, &mut view, None, &[702]);
    panel_checked(&mut window, &mut view, None, &[702]);
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let panel = nodes_in_window(&pairs, "AATF violations");
    assert!(
        panel
            .iter()
            .any(|(_, node)| node.value() == Some("Team 702 /a/")),
        "the panel's rows are team 702's"
    );
    // The dock's band is the panel's own nodes left out: below its
    // header row and outside the floating window's subtree.
    let dock_top = pairs
        .iter()
        .find(|(_, node)| node.role() == Role::Label && node.value() == Some("Live AATF"))
        .and_then(|(_, node)| node.bounds())
        .expect("the dock's header row")
        .y0;
    let panel_ids: std::collections::HashSet<accesskit::NodeId> =
        panel.iter().map(|(id, _)| *id).collect();
    assert!(
        pairs.iter().any(|(id, node)| {
            !panel_ids.contains(id)
                && node.value() == Some("Team 702 /a/")
                && node.bounds().is_some_and(|b| b.y0 >= dock_top - 1.0)
        }),
        "the dock's rows are team 702's"
    );
}

/// A violation row's click selects the player: `view.selected` is his
/// (team, slot).
// SE-GUI-30
#[test]
fn a_rows_click_selects_the_player() {
    let (mut view, mut window) = open_734_with("live_aatf_click", &[73401, 73404], 183);
    view.live_aatf = true;
    let nodes = live_checked(&mut window, &mut view, None, &[734]);
    // Another of 734's players selected first: a click that did nothing
    // would satisfy the assertion anyway.
    view.selected = Some(Selection::Player(734, 0));
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
}
