use super::*;

use std::path::Path;

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

/// The two columns' widths the cells should be drawn at for `left`'s and
/// `right`'s labels in a list `list_width` wide: each column its widest
/// cell's natural width while the pair fit, else half each (plans
/// "Editing UI", the quick actions bullet). Computed here from the labels
/// alone, never the code's own function, so a wrong rule in the code
/// fails the tests. `right`'s last is the Set stats cell (its button,
/// the item spacing and the field's two digits).
fn expected_columns(window: &Headless, left: &[&str], right: &[&str], list_width: f64) -> [f64; 2] {
    let style = window.ctx.global_style();
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
    let widest = |labels: &[&str]| {
        labels
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
            .fold(0.0_f64, f64::max)
    };
    let columns = [widest(left), widest(right)];
    if columns[0] + 6.0 + columns[1] <= list_width {
        columns
    } else {
        [(list_width - 6.0) / 2.0; 2]
    }
}

/// The quick actions are `left`'s labels down the left column and
/// `right`'s down the right, in the given order — each column's cells
/// sharing one x inside the list rows' x range, every cell
/// `QUICK_ACTION_HEIGHT` (20 points) tall and below the list's drawn rows,
/// the Set stats row the right column's last at the buttons' own pitch.
/// `shapes` is the frame's painted shapes: every button is as wide as
/// its label plus the button padding, the label eliding only when the
/// cell cannot hold it.
fn assert_quick_actions(
    window: &Headless,
    nodes: &[accesskit::Node],
    shapes: &[egui::epaint::ClippedShape],
    left: &[&str],
    right: &[&str],
) {
    // A player row's bounds stand for the list's x range: the buttons
    // sit inside it and below the list's last drawn row.
    let row = nodes
        .iter()
        .find(|node| player_row(node))
        .and_then(accesskit::Node::bounds)
        .expect("a player row in the list");
    // Every button fills its column's width (its widest cell's natural
    // width while the two columns fit the grid's band, else half each),
    // its label eliding only when it cannot fit. The band runs past the
    // rows to the list's scroll bar.
    let style = window.ctx.global_style();
    let columns = expected_columns(window, left, right, grid_band_right(nodes) - row.x0);
    let ctx = &window.ctx;
    let mut column_xs = Vec::new();
    for (labels, cell) in [(left, columns[0]), (right, columns[1])] {
        assert!(!labels.is_empty(), "a column is expected");
        let col_x = node_labeled(nodes, Role::Button, labels[0])
            .bounds()
            .expect("bounds")
            .x0;
        column_xs.push(col_x);
        let mut last_y = f64::NEG_INFINITY;
        let mut last_gap = None;
        for label in labels {
            let button = node_labeled(nodes, Role::Button, label);
            let bounds = button.bounds().expect("bounds");
            assert!(
                (bounds.x0 - col_x).abs() <= 0.5,
                "{label} shares the column's x ({bounds:?} vs {col_x})"
            );
            assert!(
                bounds.y0 > last_y + 5.0,
                "{label} below the cell before it ({bounds:?} vs {last_y})"
            );
            if last_y > f64::NEG_INFINITY {
                let pitch = bounds.y0 - last_y;
                if let Some(gap) = last_gap.replace(pitch) {
                    assert!(
                        (gap - pitch).abs() < 0.5,
                        "{label} at the buttons' own pitch"
                    );
                }
            }
            last_y = bounds.y0;
            assert!(
                (bounds.y1 - bounds.y0 - 20.0).abs() < 0.5,
                "{label} is 20 points tall ({bounds:?})"
            );
            // The Set stats cell holds the button at its own width then
            // its field; its span is the cell's.
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
            // Set stats' button is drawn the cell less the item spacing
            // and the field's two-digit width (the field takes the rest),
            // not the whole cell as a Make button is.
            let room = if *label == "Set stats" {
                let field_font = egui::TextStyle::Button.resolve(&style);
                let digits = ctx.fonts_mut(|fonts| {
                    fonts
                        .layout_no_wrap("99".to_owned(), field_font, egui::Color32::WHITE)
                        .size()
                        .x
                });
                cell - f64::from(style.spacing.item_spacing.x)
                    - f64::from(digits)
                    - 2.0 * f64::from(style.spacing.button_padding.x)
            } else {
                cell
            };
            let elides =
                f64::from(natural) + 2.0 * f64::from(style.spacing.button_padding.x) > room + 0.5;
            assert_eq!(
                shape.galley.elided, elides,
                "{label}'s galley elided={} ({bounds:?}, cell {cell})",
                shape.galley.elided
            );
        }
    }
    column_xs.sort_by(f64::total_cmp);
    column_xs.dedup();
    assert_eq!(column_xs.len(), 2, "the two columns take two x positions");
    assert!(
        (column_xs[0] - row.x0).abs() <= 1.0,
        "the first column starts at the rows' left edge ({column_xs:?} vs {row:?})"
    );
    // Every cell inside the grid's band.
    let band_right = grid_band_right(nodes);
    for label in left.iter().chain(right.iter()) {
        let bounds = node_labeled(nodes, Role::Button, label)
            .bounds()
            .expect("bounds");
        assert!(
            bounds.x0 >= row.x0 - 1.0 && bounds.x1 <= band_right + 1.0,
            "{label} inside the grid's band ({bounds:?} vs {band_right})"
        );
    }
    // The Set stats number stays inside the list's width with its
    // button (it takes the cell's remaining width), and it is 20 tall.
    let field = set_stats_field(nodes).bounds().expect("bounds");
    assert!(
        field.x1 <= band_right + 1.0,
        "the Set stats field inside the grid's band ({field:?} vs {band_right})"
    );
    assert!(
        (field.y1 - field.y0 - 20.0).abs() < 0.5,
        "the Set stats field is 20 points tall ({field:?})"
    );
    let first_y0 = node_labeled(nodes, Role::Button, left[0])
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
}

/// Label nodes inside the side panel (left of the detail pane's edge):
/// the quick actions' band's load error standing in for the Make buttons.
/// The detail view's own labels sit at its x and never pass.
/// The quick actions grid's right edge: the player list's scroll bar's
/// (the leftmost bar of the frame). The grid spans the panel's width, while
/// the rows stop short of the band the list reserves for its bar.
fn grid_band_right(nodes: &[accesskit::Node]) -> f64 {
    nodes
        .iter()
        .filter(|node| node.role() == Role::ScrollBar)
        .filter_map(|node| node.bounds())
        .min_by(|a, b| a.x0.total_cmp(&b.x0))
        .map(|bounds| bounds.x1)
        .expect("the list's scroll bar marks the band's right")
}

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
    (path, view, Headless::sized(DEFAULT_PANE.x, 700.0))
}

/// `name`'s embedded ruleset as a rules file under `dir` (its own custom
/// checks included), the path returned.
fn rules_file_under(dir: &Path, name: &str) -> PathBuf {
    let template = aatf::templates()
        .into_iter()
        .find(|template| template.ruleset.name == name)
        .unwrap_or_else(|| panic!("no {name} template"));
    let path = dir.join(format!("{}.4ccrules", name.to_lowercase()));
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
            tooltip_containing(&mut window, &mut view, centre(button), "make buttons", None),
            Some(tip.to_string()),
            "{label}'s tooltip"
        );
    }
    let field = set_stats_field(&nodes);
    assert!(field.is_disabled(), "the number disabled too");
    assert_eq!(
        tooltip_containing(&mut window, &mut view, centre(field), "make buttons", None),
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
            tooltip_containing(
                &mut window,
                &mut view,
                centre(item),
                "acts on a player",
                None
            ),
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
    // The medal tiers down the left column in the ruleset's order, the
    // others then Set stats down the right.
    assert_quick_actions(
        &window,
        &nodes,
        &shapes,
        &["Make Gold", "Make Silver", "Make Bronze"],
        &["Make Goalkeeper", "Make Regular", "Set stats"],
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
    // VTL11's medal tiers are Gold and Silver (no bronze): Regular and
    // Buffed then Set stats down the right column.
    assert_quick_actions(
        &window,
        &nodes,
        &shapes,
        &["Make Gold", "Make Silver"],
        &["Make Regular", "Make Buffed", "Set stats"],
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

// SE-GUI-09, its Set stats clause: Set stats is the Player menu's Set all
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
    let mut window = Headless::sized(DEFAULT_PANE.x, 700.0);
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

// An `apply_tier` that fails leaves the player as he was and shows the
// error in the refusal dialog.
#[test]
fn a_failed_make_shows_the_refusal_and_changes_nothing() {
    let (path, mut view, mut window) = open_701("quick_make_fails");
    let rules_path = path.with_file_name("broken_tier.4ccrules");
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
    let mut window = Headless::sized(DEFAULT_PANE.x, 700.0);

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
        &["Make Gold", "Make Silver"],
        &["Make Regular", "Make Buffed", "Set stats"],
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
        &["Make Gold", "Make Silver", "Make Bronze"],
        &["Make Goalkeeper", "Make Regular", "Set stats"],
    );

    // A file that does not load: Set stats and the red line on the row,
    // no tier button.
    let bad = dir.join("broken.4ccrules");
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
                .is_some_and(|value| value.starts_with("The AATF rules file could not be loaded,"))
        }),
        "the load error's line on the quick actions row"
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

// A rules file that does not load: its error is a red line in the tier
// buttons' place, wrapped across the list, whose click opens the error
// in the refusal dialog.
#[test]
fn a_rules_load_error_is_a_red_line_whose_click_opens_it() {
    /// The line the error draws in the tier buttons' place.
    const LINE: &str = "The AATF rules file could not be loaded, click here for more information.";

    let path = fixture(PesVersion::Pes21, "quick_error_cell");
    let dir = path.parent().unwrap().to_path_buf();
    let bad = dir.join("bad.4ccrules");
    std::fs::write(&bad, "this is no ruleset").unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::new();

    // The grid's first frame is the invisible sizing pass: a settle
    // frame so the cells' measured widths land before asserting.
    window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(bad.as_path()))
    });
    let (_, nodes, shapes) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(bad.as_path()))
    });

    // The line's own text, in the error colour, inside the side panel.
    let error_node = quick_action_row_labels(&nodes)
        .into_iter()
        .find(|node| node.value() == Some(LINE))
        .expect("the load error's line");
    let bounds = error_node.bounds().expect("bounds");
    let style = window.ctx.style_of(window.ctx.theme());
    let shape = painted_label(&shapes, LINE, bounds);
    assert_eq!(
        galley_color(shape),
        style.visuals.error_fg_color,
        "the line is painted in the error colour"
    );

    // It spans the list's width, two lines at the default 255.
    let row = nodes
        .iter()
        .find(|node| player_row(node))
        .and_then(accesskit::Node::bounds)
        .expect("a player row in the list");
    // The laid-out row's height (the galley's first row), not
    // `fonts.row_height`'s unrounded one: rows are laid out on whole
    // pixels, so three of them can exceed three unrounded heights.
    let body_row = shape.galley.rows[0].rect().height();
    // The rows stop short of the scroll bar's band the list reserves; the
    // line, outside the list, may run into it.
    let band = f64::from(style.spacing.scroll.bar_width);
    assert!(
        (bounds.x0 - row.x0).abs() <= 1.0 && bounds.x1 <= row.x1 + band + 1.0,
        "the line spans the list ({bounds:?} vs {row:?} and its {band}-point band)"
    );
    // Not elided, since a label cut to one line hides most of the error,
    // and wrapped across the list: two body rows at the default 255, as
    // the draft draws it.
    assert!(!shape.galley.elided, "the line is not elided");
    assert!(
        bounds.y1 - bounds.y0 > f64::from(body_row),
        "the line spans more than one body row ({bounds:?} vs {body_row})"
    );
    assert!(
        bounds.y1 - bounds.y0 <= f64::from(body_row) * 2.0 + 1.0,
        "the line is two body rows at most ({bounds:?} vs {body_row})"
    );

    // No Make button; Set stats sits under the line.
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node.label().is_some_and(|label| label.starts_with("Make "))
        }),
        "no tier button"
    );
    let set_stats = node_labeled(&nodes, Role::Button, "Set stats")
        .bounds()
        .expect("bounds");
    assert!(
        set_stats.y0 >= bounds.y1 - 1.0,
        "Set stats under the line ({set_stats:?} vs {bounds:?})"
    );

    // The dialog is up only after the click: nothing of it on the frames
    // before.
    assert!(
        !label_value(&nodes, "The AATF rules file could not be loaded"),
        "no dialog title before the click"
    );
    assert!(view.refusal.is_none(), "no refusal before the click");

    // Its click opens the refusal dialog: the title as the heading, the
    // lib's error as the body, and the full text drawn nowhere in the
    // list's panel but the dialog's.
    let expected = view
        .rules
        .as_ref()
        .expect("a ruleset was attempted")
        .rules
        .as_ref()
        .expect_err("the file does not load")
        .clone();
    click_with_rules(&mut window, error_node, &mut view, &bad);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(bad.as_path()))
    });
    assert!(
        label_value(&nodes, "The AATF rules file could not be loaded"),
        "the dialog's title"
    );
    assert!(
        label_value(&nodes, &expected),
        "the dialog's body is the error itself"
    );
    // The error's text is drawn once, in the dialog; the list's panel
    // holds only the line's own text.
    let copies = nodes
        .iter()
        .filter(|node| node.value() == Some(expected.as_str()))
        .count();
    assert_eq!(copies, 1, "the error's text drawn once, in the dialog");
}

// The view fills its pane: the list and the quick actions run to the
// pane's bottom — the actions' band's painted bottom is the pane's own
// painted bottom at the 1280 x 800 window, no dead strip under it (the
// bottom panel takes its height off the list's scroll, whose floating
// bar reserves no strip of its own).
#[test]
fn the_quick_actions_reach_the_panes_bottom() {
    let (_path, mut view, _window) = open_701("quick_fills_pane");
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let panel_fill = window.ctx.style_of(window.ctx.theme()).visuals.panel_fill;
    let frame = |shapes: &[egui::epaint::ClippedShape],
                 pick: &dyn Fn(&egui::epaint::RectShape) -> bool| {
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.fill == panel_fill && pick(shape) => {
                    Some(shape.rect)
                }
                _ => None,
            })
            .next()
            .expect("a panel frame")
    };

    // The pane the quick actions sit in: the player list's own painted
    // frame, full height inside the window's left margin.
    let pane = frame(&shapes, &|shape| {
        shape.rect.min.x > 2.0 && shape.rect.max.x < 400.0 && shape.rect.height() > 600.0
    });
    // The quick actions' band: the bottom panel's painted frame under the
    // list — a shallow strip inside the pane's x range near its bottom.
    let band = frame(&shapes, &|shape| {
        shape.rect.min.x > 10.0
            && shape.rect.max.x < pane.max.x - 1.0
            && shape.rect.height() < 200.0
            && shape.rect.max.y > pane.max.y - 100.0
    });
    assert!(
        (band.max.y - pane.max.y).abs() <= 1.0,
        "the quick actions' bottom within a point of the pane's ({:?} vs {:?})",
        band,
        pane
    );
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

// The draft's `.qagrid` inside the 255-wide `.plist` holds its labels
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

// The columns' fallback (plans "Editing UI", the quick actions bullet):
// where the cells' natural widths do not fit the list each column is
// half of it, a label too long for its half eliding (its tooltip names
// the tier). The list is a fixed 255, so the fallback is reached by a
// long tier label, not a narrower list.
#[test]
fn the_columns_halve_when_the_content_widths_do_not_fit() {
    const WIDE_TIER: &str = "the longest tier in the ruleset";
    // A ruleset whose first tier's label is too long for the two
    // columns' naturals to fit inside the list's 255.
    let mut ruleset = aatf::Rules::official().ruleset().clone();
    ruleset.tiers[0].label = WIDE_TIER.to_owned();
    let path = fixture(PesVersion::Pes21, "quick_halved_columns");
    let dir = path.parent().unwrap().to_path_buf();
    let wide = dir.join("wide.4ccrules");
    std::fs::write(&wide, aatf::assemble_file(&ruleset, None, None)).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::new();

    // The grid's column widths are stored per frame: a settle frame so
    // they land before asserting.
    window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(wide.as_path()))
    });
    let (_, nodes, shapes) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(wide.as_path()))
    });
    // The cells as the labels alone say they should be: halves, the long
    // label elided in its own.
    let wide_label = format!("Make {WIDE_TIER}");
    let left = [wide_label.as_str(), "Make Silver", "Make Bronze"];
    let right = ["Make Goalkeeper", "Make Regular", "Set stats"];
    let row = nodes
        .iter()
        .find(|node| player_row(node))
        .and_then(accesskit::Node::bounds)
        .expect("a player row in the list");
    let band_right = grid_band_right(&nodes);
    let natural = expected_columns(&window, &left, &right, f64::INFINITY);
    assert!(
        natural[0] + 6.0 + natural[1] > band_right - row.x0,
        "premise: the naturals {natural:?} do not fit the band from {} to {band_right}",
        row.x0
    );
    assert_quick_actions(&window, &nodes, &shapes, &left, &right);

    let half = (band_right - row.x0 - 6.0) / 2.0;
    for label in [
        &*format!("Make {WIDE_TIER}"),
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
            bounds.x0 >= row.x0 - 1.0 && right <= band_right + 1.0,
            "{label} inside the grid's band ({bounds:?} vs {band_right})"
        );
        assert!(
            (right - bounds.x0 - half).abs() <= 1.0,
            "{label}'s cell a half of the band ({bounds:?} vs {half})"
        );
    }
    let label = format!("Make {WIDE_TIER}");
    let wide_button = node_labeled(&nodes, Role::Button, &label);
    let bounds = wide_button.bounds().expect("bounds");
    assert!(
        painted_label(&shapes, &label, bounds).galley.elided,
        "the long label elides in its half ({bounds:?})"
    );
    // The elided label's tooltip names the tier.
    assert_eq!(
        tooltip_containing(
            &mut window,
            &mut view,
            centre(wide_button),
            "tier's values",
            Some(wide.as_path())
        ),
        Some(format!(
            "The {WIDE_TIER} tier's values (the active ruleset)"
        )),
        "the elided button's tooltip"
    );
}
