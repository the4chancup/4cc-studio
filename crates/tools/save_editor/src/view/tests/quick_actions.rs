use super::*;

use std::path::Path;

/// The quick actions' buttons under the list, in (y, x) order: the Make
/// buttons then Set stats, two columns (the draft's `qagrid`). A player
/// row in the list above must not pass.
fn quick_action_labels(nodes: &[accesskit::Node]) -> Vec<String> {
    let mut buttons: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Make ") || label == "Set stats")
        })
        .collect();
    buttons.sort_by(|a, b| {
        let pos = |node: &&accesskit::Node| {
            let bounds = node.bounds().expect("bounds");
            (bounds.y0, bounds.x0)
        };
        pos(a).partial_cmp(&pos(b)).expect("ordered bounds")
    });
    buttons
        .iter()
        .filter_map(|node| node.label().map(str::to_owned))
        .collect()
}

/// The grid's buttons are `expected` and no others, in (y, x) order, two
/// per row (the two-column grid), inside the side panel's x range and
/// below the list's drawn rows.
fn assert_quick_actions(nodes: &[accesskit::Node], expected: &[&str]) {
    assert_eq!(quick_action_labels(nodes), expected);
    // The filter box's bounds stand for the side panel's x range: the
    // buttons sit inside it and below the list's last drawn row.
    let panel = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && node.placeholder() == Some("Filter every team (name, shirt name or id)")
        })
        .and_then(accesskit::Node::bounds)
        .expect("the filter box bounds the side panel");
    let first_y0 = node_labeled(nodes, Role::Button, expected[0])
        .bounds()
        .expect("bounds")
        .y0;
    // The list's last drawn row's top: a row clipped at the bottom panel's
    // edge keeps an unclipped bounds, so the tops are what order the two.
    let last_row_y0 = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && node.bounds().is_some_and(|b| {
                    b.x0 >= panel.x0 - 1.0
                        && b.x1 <= panel.x1 + 1.0
                        && b.y0 > panel.y1
                        && b.y0 < first_y0
                })
        })
        .filter_map(|node| node.bounds().map(|b| b.y0))
        .fold(0.0, f64::max);
    assert!(
        first_y0 > last_row_y0,
        "the grid starts below the list's last row ({first_y0} vs {last_row_y0})"
    );
    let mut xs: Vec<f64> = expected
        .iter()
        .map(|button| {
            node_labeled(nodes, Role::Button, button)
                .bounds()
                .expect("bounds")
                .x0
        })
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    assert_eq!(xs.len(), 2, "the two columns take two x positions");
    for pair in expected.chunks(2) {
        let row_y = node_labeled(nodes, Role::Button, pair[0])
            .bounds()
            .expect("bounds")
            .y0;
        for button in pair {
            let bounds = node_labeled(nodes, Role::Button, button)
                .bounds()
                .expect("bounds");
            assert!(
                (bounds.y0 - row_y).abs() < 4.0,
                "{button} shares {0}'s row ({bounds:?})",
                pair[0]
            );
            assert!(
                bounds.x0 >= panel.x0 - 1.0 && bounds.x1 <= panel.x1 + 1.0,
                "{button} inside the side panel's x range ({bounds:?} vs {panel:?})"
            );
        }
    }
}

/// Label nodes inside the side panel (left of the detail pane's edge):
/// the quick actions' band's load error standing in for the Make buttons.
/// The detail view's own labels sit at its x and never pass.
fn quick_action_row_labels(nodes: &[accesskit::Node]) -> Vec<&accesskit::Node> {
    nodes
        .iter()
        .filter(|node| node.role() == Role::Label && node.bounds().is_some_and(|b| b.x0 < 200.0))
        .collect()
}

/// The Set stats field: the row's one number control, right of the button.
fn set_stats_field(nodes: &[accesskit::Node]) -> &accesskit::Node {
    right_of(
        nodes,
        node_labeled(nodes, Role::Button, "Set stats"),
        Role::SpinButton,
    )
}

/// `text` typed into `field`: a click to focus it, select all, the
/// characters, then Enter to commit — the crate's number-typing gesture.
fn type_number(window: &mut Headless, view: &mut SaveView, field: &accesskit::Node, text: &str) {
    click(window, field, view);
    window.frame(
        vec![
            key(egui::Key::A, egui::Modifiers::COMMAND, true),
            egui::Event::Text(text.to_owned()),
        ],
        |ui| show(ui, view, &|| Vec::new(), None),
    );
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, view, &|| Vec::new(), None),
    );
}

/// `click` under `rules_file` (the shared helpers draw the official ruleset).
fn click_with_rules(
    window: &mut Headless,
    node: &accesskit::Node,
    view: &mut SaveView,
    rules_file: &Path,
) {
    let at = centre(node);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, view, &|| Vec::new(), Some(rules_file))
    });
    window.frame(pointer_at(at, false), |ui| {
        show(ui, view, &|| Vec::new(), Some(rules_file))
    });
}

/// `rules`' `apply_tier` on the roster slot's player of `view`'s save, as the
/// click should leave his entry.
fn tiered(view: &SaveView, rules: &aatf::Rules, team_id: u32, slot: u8, tier: &str) -> PlayerEntry {
    let file = view.session.as_ref().unwrap().file();
    let team = aatf::TeamView::new(file, team_id).unwrap();
    let player = team.player(slot).unwrap();
    let id = file.team(team_id).unwrap().roster[usize::from(slot)].player_id;
    let mut entry = file.player(id).unwrap().clone();
    rules.apply_tier(&team, player, tier, &mut entry).unwrap();
    entry
}

/// `test`'s fixture save's path, opened with team 701's slot 0 selected, the
/// window at the tests' usual size.
fn open_701(test: &str) -> (PathBuf, SaveView, Headless) {
    let path = fixture(PesVersion::Pes21, test);
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    (path, view, Headless::sized(1100.0, 700.0))
}

/// `name`'s embedded ruleset as a rules file under `dir` (its own custom
/// checks included), the path returned.
fn rules_file_under(dir: &Path, name: &str) -> PathBuf {
    let template = aatf::templates()
        .into_iter()
        .find(|template| template.ruleset.name == name)
        .unwrap_or_else(|| panic!("no {name} template"));
    let path = dir.join(format!("{}.rhai", name.to_lowercase()));
    std::fs::write(
        &path,
        aatf::assemble_file(&template.ruleset, None, template.custom_checks.as_deref()),
    )
    .unwrap();
    path
}

// The quick actions under the list on either selection (plans "Editing
// UI", the Quick actions bullet): on a player row they act on him; on a
// team row every control draws disabled with the reason on hover, and the
// Player menu's entries the same. There is no "Quick actions" heading and
// no `AATF…` button (the Panels menu opens the panel).
// SE-GUI-37
#[test]
fn the_quick_actions_and_player_menu_disable_on_a_team_row() {
    let (_path, mut view, mut window) = open_701("quick_on_team_row");

    /// No node of the frame carries the tool's name (the "No view draws
    /// the tool's name" bullet): the window's title bar carries it.
    fn draws_tool_name(nodes: &[accesskit::Node]) -> bool {
        nodes.iter().any(|node| {
            node.label().is_some_and(|l| l.contains("Save editor"))
                || node.value().is_some_and(|v| v.contains("Save editor"))
        })
    }

    // A player row: the controls are enabled.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !nodes
            .iter()
            .any(|node| { node.role() == Role::Label && node.value() == Some("Quick actions") }),
        "no heading"
    );
    assert!(!draws_tool_name(&nodes), "no view draws \"Save editor\"");
    for label in ["Make Gold", "Set stats"] {
        assert!(
            !node_labeled(&nodes, Role::Button, label).is_disabled(),
            "{label} enabled on a player row"
        );
    }
    assert!(
        nodes
            .iter()
            .all(|node| node.label() != Some("AATF\u{2026}")),
        "no AATF button"
    );

    // The team row: every control disabled with the tooltip, and the
    // Player menu's entries the same.
    open_team_view(&mut window, &mut view, 701);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !draws_tool_name(&nodes),
        "the team view draws no \"Save editor\" either"
    );
    let tip = "The make buttons act on a player: pick a player row";
    for label in ["Make Gold", "Make Regular", "Set stats"] {
        let button = node_labeled(&nodes, Role::Button, label);
        assert!(button.is_disabled(), "{label} disabled on a team row");
        assert_eq!(
            tooltip_containing(&mut window, &mut view, centre(button), "make buttons"),
            Some(tip.to_string()),
            "{label}'s tooltip"
        );
    }
    let field = set_stats_field(&nodes);
    assert!(field.is_disabled(), "the number disabled too");
    assert_eq!(
        tooltip_containing(&mut window, &mut view, centre(field), "make buttons"),
        Some(tip.to_string()),
        "the number's tooltip"
    );

    let nodes = open_menu(&mut window, &mut view, "Player", "Set all stats");
    for label in [
        "Set all stats",
        "Bump all stats",
        "Toggle FPC settings",
        "Copy stats from",
        "Swap stats with",
        "Generate settings.toml",
        "Generate in-game face",
    ] {
        let item = menu_item(&nodes, label);
        assert!(item.is_disabled(), "{label} disabled on a team row");
        assert_eq!(
            tooltip_containing(&mut window, &mut view, centre(item), "acts on a player"),
            Some("The Player menu acts on a player: pick a player row".to_string()),
            "{label}'s tooltip"
        );
    }
}

// SE-GUI-09
#[test]
fn the_quick_actions_apply_a_tier_as_one_undoable_edit() {
    let (_path, mut view, mut window) = open_701("quick_make_gold");
    let rules = aatf::Rules::official();

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_quick_actions(
        &nodes,
        &[
            "Make Gold",
            "Make Silver",
            "Make Bronze",
            "Make Goalkeeper",
            "Make Regular",
            "Set stats",
        ],
    );

    // Make Gold: his entry is the lib's `apply_tier` run on a clone of it.
    let before = player_entry(&view, 70101);
    let expected = tiered(&view, &rules, 701, 0, "gold");
    assert_ne!(expected, before, "premise: Gold changes him");
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Make Gold"),
        &mut view,
    );
    assert_eq!(player_entry(&view, 70101), expected, "Gold's values");

    // check_team then reports no Stat-family violation on his slot.
    let team = aatf::TeamView::new(view.session.as_ref().unwrap().file(), 701).unwrap();
    let violations = rules.check_team(&team).unwrap();
    let own: Vec<&aatf::Violation> = violations
        .iter()
        .filter(|violation| {
            violation.slot == Some(0)
                && matches!(
                    violation.kind,
                    aatf::ViolationKind::Stat { .. }
                        | aatf::ViolationKind::StatTotal { .. }
                        | aatf::ViolationKind::Form { .. }
                        | aatf::ViolationKind::InjuryResistance { .. }
                        | aatf::ViolationKind::WeakFootUsage { .. }
                        | aatf::ViolationKind::WeakFootAccuracy { .. }
                        | aatf::ViolationKind::NameColor { .. }
                )
        })
        .collect();
    assert!(
        own.is_empty(),
        "no violation of what the tier sets on his slot: {own:?}"
    );

    // One Ctrl+Z restores the old entry.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        player_entry(&view, 70101),
        before,
        "the one undo restores it"
    );
    assert!(!view.session.as_ref().unwrap().can_undo(), "one entry only");
}

// SE-GUI-09
#[test]
fn the_quick_actions_follow_the_rules_files_tiers() {
    let (path, mut view, mut window) = open_701("quick_vtl11");
    let rules_path = rules_file_under(path.parent().unwrap(), "VTL11");
    let rules = aatf::Rules::load(&std::fs::read_to_string(&rules_path).unwrap()).unwrap();

    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    assert_quick_actions(
        &nodes,
        &[
            "Make Gold",
            "Make Silver",
            "Make Regular",
            "Make Buffed",
            "Set stats",
        ],
    );

    // Make Silver gives him VTL11's Silver values.
    let before = player_entry(&view, 70101);
    let expected = tiered(&view, &rules, 701, 0, "silver");
    assert_ne!(expected, before, "premise: Silver changes him");
    click_with_rules(
        &mut window,
        node_labeled(&nodes, Role::Button, "Make Silver"),
        &mut view,
        &rules_path,
    );
    let after = player_entry(&view, 70101);
    assert_eq!(after, expected, "VTL11's Silver values");
    // `speed` is no `stats` override of Silver's: its target is the tier's rate, 89.
    assert_eq!(after.stats.speed, 89);
}

// SE-GUI-09, its Set stats clause (plans/save_editor.md "Configurable AATF
// rules", the Quick actions bullet): Set stats is the Player menu's Set all
// stats in one click, its number 40 to 99 starting at 77.
#[test]
fn set_stats_writes_every_ability_to_the_number() {
    // Speed moved off the fixture's 77, so the click is a real write and the
    // undo has something to restore.
    let path = fixture(PesVersion::Pes21, "quick_set_stats");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().stats.speed = 60;
    file.save(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::sized(1100.0, 700.0);
    let before = player_entry(&view, 70101);
    assert_eq!(before.stats.speed, 60, "premise: the moved stat");
    assert_eq!(view.set_stats, 77, "the number starts at 77");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Set stats"),
        &mut view,
    );
    {
        let stats = &player_entry(&view, 70101).stats;
        for (index, ability) in stats.abilities().iter().enumerate() {
            assert_eq!(*ability, Some(77), "ability {index} at the starting 77");
        }
        assert_eq!(stats.weak_foot_usage, before.stats.weak_foot_usage);
        assert_eq!(stats.weak_foot_accuracy, before.stats.weak_foot_accuracy);
        assert_eq!(stats.form, before.stats.form);
        assert_eq!(stats.injury_resistance, before.stats.injury_resistance);
    }

    // One Ctrl+Z restores the entry.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        player_entry(&view, 70101),
        before,
        "the one undo restores it"
    );

    // A new number typed into the row's field is the value written.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = set_stats_field(&nodes);
    type_number(&mut window, &mut view, field, "85");
    assert_eq!(view.set_stats, 85, "the field holds the typed 85");
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Set stats"),
        &mut view,
    );
    for (index, ability) in player_entry(&view, 70101)
        .stats
        .abilities()
        .iter()
        .enumerate()
    {
        assert_eq!(*ability, Some(85), "ability {index} at the typed 85");
    }

    // The field's 40 to 99 range settles a typed number past its top.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = set_stats_field(&nodes);
    type_number(&mut window, &mut view, field, "120");
    assert_eq!(view.set_stats, 99, "a typed 120 settles at 99");
}

// plans/save_editor.md "Configurable AATF rules", the Quick actions bullet: an
// `apply_tier` that fails leaves the player as he was and shows the error in
// the refusal dialog.
#[test]
fn a_failed_make_shows_the_refusal_and_changes_nothing() {
    let (path, mut view, mut window) = open_701("quick_make_fails");
    let rules_path = path.with_file_name("broken_tier.rhai");
    // The smallest file whose `tier_values` throws for one tier: the official
    // file assembled, with a `throw` added at the function's head for "silver"
    // alone.
    let text = aatf::assemble_file(aatf::Rules::official().ruleset(), None, None).replacen(
        "fn tier_values(player, tier, team) {",
        "fn tier_values(player, tier, team) {\n    if tier == \"silver\" { throw \"a staged failure\"; }",
        1,
    );
    std::fs::write(&rules_path, text).unwrap();
    let before = player_entry(&view, 70101);

    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    click_with_rules(
        &mut window,
        node_labeled(&nodes, Role::Button, "Make Silver"),
        &mut view,
        &rules_path,
    );
    assert_eq!(
        player_entry(&view, 70101),
        before,
        "the player is unchanged"
    );
    assert!(
        !view.session.as_ref().unwrap().can_undo(),
        "nothing on the undo stack"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Cannot apply the tier")),
        "the refusal dialog's heading"
    );
    assert!(
        nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.contains("a staged failure"))
        }),
        "the error's text"
    );
}

// SE-GUI-32
#[test]
fn the_rules_file_row_commits_the_setting_and_the_view_follows() {
    use studio_core::StudioTool;

    let save_path = fixture(PesVersion::Pes21, "settings_rules_row");
    let dir = save_path.parent().unwrap().to_path_buf();
    let rules_path = rules_file_under(&dir, "VTL11");
    let rules_arg = rules_path.display().to_string();
    let mut tool = crate::Tool::new();
    tool.view.open(&save_path);
    let mut settings = studio_core::Settings::default();
    let mut table = tool.default_settings();
    table.insert("unread_key".to_owned(), toml::Value::Integer(3));
    settings.set_tool("save-editor", table);
    let (ctx, _events_rx, _requests_rx) = tool_context(&dir, settings);
    let mut window = Headless::sized(1100.0, 700.0);

    // The VTL11 path typed into the row and Enter pressed: the setting holds it,
    // the tool's other key unchanged, and the next view frame draws VTL11's
    // buttons.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.settings_view(ui, &ctx));
    let field = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput)
        .expect("the rules file row's edit");
    let at = centre(field);
    window.frame(pointer_at(at, true), |ui| tool.settings_view(ui, &ctx));
    window.frame(pointer_at(at, false), |ui| tool.settings_view(ui, &ctx));
    window.frame(vec![egui::Event::Text(rules_arg.clone())], |ui| {
        tool.settings_view(ui, &ctx)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| tool.settings_view(ui, &ctx),
    );
    let table = ctx.tool_settings("save-editor");
    assert_eq!(
        table["aatf_rules"].as_str(),
        Some(rules_arg.as_str()),
        "the setting holds the typed path"
    );
    assert_eq!(
        table["unread_key"].as_integer(),
        Some(3),
        "the tool's other key is unchanged"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert_quick_actions(
        &nodes,
        &[
            "Make Gold",
            "Make Silver",
            "Make Regular",
            "Make Buffed",
            "Set stats",
        ],
    );

    // The row emptied and committed: the official ruleset's buttons again.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.settings_view(ui, &ctx));
    let field = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput)
        .expect("the rules file row's edit");
    let at = centre(field);
    window.frame(pointer_at(at, true), |ui| tool.settings_view(ui, &ctx));
    window.frame(pointer_at(at, false), |ui| tool.settings_view(ui, &ctx));
    window.frame(
        vec![
            key(egui::Key::A, egui::Modifiers::COMMAND, true),
            key(egui::Key::Backspace, egui::Modifiers::NONE, true),
        ],
        |ui| tool.settings_view(ui, &ctx),
    );
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| tool.settings_view(ui, &ctx),
    );
    let table = ctx.tool_settings("save-editor");
    assert_eq!(
        table["aatf_rules"].as_str(),
        Some(""),
        "the emptied row committed"
    );
    assert_eq!(table["unread_key"].as_integer(), Some(3));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert_quick_actions(
        &nodes,
        &[
            "Make Gold",
            "Make Silver",
            "Make Bronze",
            "Make Goalkeeper",
            "Make Regular",
            "Set stats",
        ],
    );

    // A file that does not load: Set stats and the load error on the row, no
    // tier button.
    let bad = dir.join("broken.rhai");
    std::fs::write(&bad, "let x = ;").unwrap();
    let mut table = ctx.tool_settings("save-editor");
    table.insert(
        "aatf_rules".to_owned(),
        toml::Value::String(bad.display().to_string()),
    );
    ctx.set_tool_settings("save-editor", table);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert!(
        quick_action_row_labels(&nodes).iter().any(|node| {
            node.value()
                .is_some_and(|value| value.starts_with("The AATF rules file could not be loaded:"))
        }),
        "the load error on the quick actions row"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node.label().is_some_and(|label| label.starts_with("Make "))
        }),
        "no Make button"
    );
    node_labeled(&nodes, Role::Button, "Set stats");
}
