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

/// The painted TextShape of `label` inside `bounds`' y range (a button's
/// own galley, not another label's on the same line).
fn painted_label<'a>(
    shapes: &'a [egui::epaint::ClippedShape],
    label: &str,
    bounds: accesskit::Rect,
) -> &'a egui::epaint::TextShape {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape)
                if shape.galley.job.text == label
                    && f64::from(shape.pos.y) >= bounds.y0
                    && f64::from(shape.pos.y) <= bounds.y1 =>
            {
                Some(shape)
            }
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("no painted {label:?} inside {bounds:?}"))
}

/// The two columns' widths the cells should be drawn at for `expected`'s
/// labels in a list `list_width` wide: each column its widest cell's
/// natural width while the pair fit, else half each (plans "Editing UI",
/// the quick actions bullet). Computed here from the labels alone, never
/// the code's own function, so a wrong rule in the code fails the tests.
fn expected_columns(window: &Headless, expected: &[&str], list_width: f64) -> [f64; 2] {
    let style = egui::Context::default().global_style();
    let button = egui::TextStyle::Button.resolve(&style);
    // The DragValue's text is `drag_value_text_style` (`TextStyle::Button`).
    let field = egui::TextStyle::Button.resolve(&style);
    let padding = 2.0 * f64::from(style.spacing.button_padding.x);
    let natural = |text: &str, font: &egui::FontId| {
        window.ctx.fonts_mut(|fonts| {
            f64::from(
                fonts
                    .layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::WHITE)
                    .size()
                    .x,
            )
        })
    };
    let cells: Vec<f64> = expected
        .iter()
        .map(|&label| {
            if label == "Set stats" {
                natural("Set stats", &button)
                    + padding
                    + f64::from(style.spacing.item_spacing.x)
                    + natural("99", &field)
                    + padding
            } else {
                natural(label, &button) + padding
            }
        })
        .collect();
    let widest = |start: usize| {
        cells
            .iter()
            .skip(start)
            .step_by(2)
            .fold(0.0_f64, |a, &b| a.max(b))
    };
    let columns = [widest(0), widest(1)];
    if columns[0] + 6.0 + columns[1] <= list_width {
        columns
    } else {
        [(list_width - 6.0) / 2.0; 2]
    }
}

/// The grid's buttons are `expected` and no others, in (y, x) order, two
/// per row (the two-column grid), inside the side panel's x range and
/// below the list's drawn rows. `shapes` is the frame's painted shapes:
/// every button is as wide as its label plus the button padding, the
/// label unelided (a narrower cell truncates it, as the first Grid
/// frame's `available_width` is no panel's).
fn assert_quick_actions(
    window: &Headless,
    nodes: &[accesskit::Node],
    shapes: &[egui::epaint::ClippedShape],
    expected: &[&str],
) {
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
    // Every button fills its column's width (its widest cell's natural
    // width while the two columns fit the list, else half each), its
    // label eliding only when it cannot fit.
    let style = egui::Context::default().global_style();
    let columns = expected_columns(window, expected, panel.x1 - panel.x0);
    let ctx = &window.ctx;
    for (index, label) in expected.iter().enumerate() {
        let cell = columns[index % 2];
        let bounds = node_labeled(nodes, Role::Button, label)
            .bounds()
            .expect("bounds");
        // The Set stats cell holds the button at its own width then its
        // field; its span is the cell's.
        let span = if *label == "Set stats" {
            let f = set_stats_field(nodes).bounds().expect("bounds");
            f.x1 - bounds.x0
        } else {
            bounds.x1 - bounds.x0
        };
        assert!(
            (span - cell).abs() <= 1.0,
            "{label}'s cell fills its column ({bounds:?} vs cell {cell})"
        );
        let shape = painted_label(shapes, label, bounds);
        let font = shape
            .galley
            .job
            .sections
            .first()
            .map(|section| section.format.font_id.clone())
            .expect("a painted label has a section");
        let natural = ctx.fonts_mut(|fonts| {
            fonts
                .layout_no_wrap((*label).to_owned(), font, egui::Color32::WHITE)
                .size()
                .x
        });
        let elides =
            f64::from(natural) + 2.0 * f64::from(style.spacing.button_padding.x) > cell + 0.5;
        assert_eq!(
            shape.galley.elided, elides,
            "{label}'s galley elided={} ({bounds:?}, cell {cell})",
            shape.galley.elided
        );
    }
    // The Set stats number stays inside the panel with its button (it
    // takes the cell's remaining width).
    let field = set_stats_field(nodes).bounds().expect("bounds");
    assert!(
        field.x1 <= panel.x1 + 1.0,
        "the Set stats field inside the side panel's x range ({field:?} vs {panel:?})"
    );
    let first_y0 = node_labeled(nodes, Role::Button, expected[0])
        .bounds()
        .expect("bounds")
        .y0;
    // The list's scroll clip's bottom: the leftmost ScrollBar's y1 (the
    // bottom panel takes the height off it).
    let clip_bottom = nodes
        .iter()
        .filter(|node| node.role() == Role::ScrollBar)
        .filter_map(|node| node.bounds())
        .min_by(|a, b| a.x0.partial_cmp(&b.x0).expect("ordered bounds"))
        .map(|bounds| bounds.y1)
        .expect("the list's scroll bar marks its clip's bottom");
    assert!(
        first_y0 >= clip_bottom - 1.0,
        "the grid starts below the list's scroll clip ({first_y0} vs {clip_bottom})"
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

    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_quick_actions(
        &window,
        &nodes,
        &shapes,
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

    let (_, nodes, shapes) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    assert_quick_actions(
        &window,
        &nodes,
        &shapes,
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
    let (_, nodes, shapes) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert_quick_actions(
        &window,
        &nodes,
        &shapes,
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
    // The grid's column widths are stored per frame — a rules change is
    // one frame stale (the Grid repaints on the next): draw a settle frame.
    window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    let (_, nodes, shapes) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert_quick_actions(
        &window,
        &nodes,
        &shapes,
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

// The grid is docked under the list (the bottom panel takes its height
// off the list's scroll): scrolling the list to its last row leaves the
// buttons where they were.
#[test]
fn the_buttons_stay_put_when_the_list_scrolls() {
    let (_path, mut view, mut window) = open_701("quick_scrolled_list");
    let make_gold = |window: &mut Headless, view: &mut SaveView| {
        let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
        node_labeled(&nodes, Role::Button, "Make Gold")
            .bounds()
            .expect("bounds")
            .y0
    };
    let top = make_gold(&mut window, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let rows_before = row_labels(&nodes);

    // A wheel down inside the list's scroll (the pointer rests over the
    // list, left of its bar).
    for _ in 0..20 {
        window.frame(
            vec![
                egui::Event::PointerMoved(egui::pos2(120.0, 400.0)),
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
    }
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_ne!(
        row_labels(&nodes),
        rows_before,
        "premise: the list scrolled"
    );
    assert_eq!(
        make_gold(&mut window, &mut view),
        top,
        "the buttons did not move"
    );
}

// The draft's `.qagrid` inside the 240-wide `.plist` holds its labels
// whole (plans "Editing UI", the Quick actions bullet): at the window's
// default size no Make button's galley elides, and every control — the
// Set stats field included — stays inside the list rows' x range.
#[test]
fn the_quick_actions_fit_the_lists_width() {
    let path = fixture(PesVersion::Pes21, "quick_fit_width");
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::new();

    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    // A player row's AccessKit bounds stand for the list rows' x range.
    let row = nodes
        .iter()
        .find(|node| player_row(node))
        .and_then(accesskit::Node::bounds)
        .expect("a player row in the list");
    for label in [
        "Make Gold",
        "Make Silver",
        "Make Bronze",
        "Make Goalkeeper",
        "Make Regular",
        "Set stats",
    ] {
        let bounds = node_labeled(&nodes, Role::Button, label)
            .bounds()
            .expect("bounds");
        assert!(
            bounds.x0 >= row.x0 - 1.0 && bounds.x1 <= row.x1 + 1.0,
            "{label} inside the list rows' x range ({bounds:?} vs {row:?})"
        );
        if label.starts_with("Make ") {
            let shape = painted_label(&shapes, label, bounds);
            assert!(!shape.galley.elided, "{label}'s galley whole ({bounds:?})");
        }
    }
    let field = set_stats_field(&nodes).bounds().expect("bounds");
    assert!(
        field.x0 >= row.x0 - 1.0 && field.x1 <= row.x1 + 1.0,
        "the Set stats field inside the rows' x range ({field:?} vs {row:?})"
    );
}

// The columns' fallback (plans "Editing UI", the quick actions bullet): a
// list too narrow for the cells' natural widths halves the columns, a
// label too long for its half eliding (its tooltip names the tier). The
// side panel keeps its 240 default under a narrow window, so the list is
// narrowed by dragging the panel's resize edge instead.
#[test]
fn the_columns_halve_when_the_content_widths_do_not_fit() {
    let path = fixture(PesVersion::Pes21, "quick_halved_columns");
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::sized(1100.0, 700.0);

    // The list as drawn at the panel's 240 default: the resize edge is the
    // panel's right border (~248), dragged left past the columns' ~221.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let row_before = nodes
        .iter()
        .find(|node| player_row(node))
        .and_then(accesskit::Node::bounds)
        .expect("a player row in the list");
    let edge = egui::pos2((row_before.x1 + 8.0) as f32, 400.0);
    // Past the columns' ~221-point fit but wide enough that the Set stats
    // pair (button ~30 + spacing + field ~30) still fits its half.
    let to = egui::pos2(edge.x - 65.0, 400.0);
    window.frame(pointer_at(edge, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(vec![egui::Event::PointerMoved(to)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(pointer_at(to, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    // The grid's column widths are stored per frame: two settle frames so
    // the resized panel's widths land before asserting.
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let row = nodes
        .iter()
        .find(|node| player_row(node))
        .and_then(accesskit::Node::bounds)
        .expect("a player row in the list");
    assert!(
        row.x1 - row.x0 < row_before.x1 - row_before.x0 - 60.0,
        "premise: the panel resized narrower ({row_before:?} vs {row:?})"
    );
    let half = (row.x1 - row.x0 - 6.0) / 2.0;
    for label in [
        "Make Gold",
        "Make Silver",
        "Make Bronze",
        "Make Goalkeeper",
        "Make Regular",
        "Set stats",
    ] {
        let bounds = node_labeled(&nodes, Role::Button, label)
            .bounds()
            .expect("bounds");
        // The Set stats cell's span ends at its field's right edge.
        let right = if label == "Set stats" {
            set_stats_field(&nodes).bounds().expect("bounds").x1
        } else {
            bounds.x1
        };
        assert!(
            bounds.x0 >= row.x0 - 1.0 && right <= row.x1 + 1.0,
            "{label} inside the list rows' x range ({bounds:?} vs {row:?})"
        );
        assert!(
            (right - bounds.x0 - half).abs() <= 1.0,
            "{label}'s cell a half of the list ({bounds:?} vs {half})"
        );
    }
    let bounds = node_labeled(&nodes, Role::Button, "Make Goalkeeper")
        .bounds()
        .expect("bounds");
    assert!(
        painted_label(&shapes, "Make Goalkeeper", bounds)
            .galley
            .elided,
        "Make Goalkeeper elides in its half ({bounds:?})"
    );
}
