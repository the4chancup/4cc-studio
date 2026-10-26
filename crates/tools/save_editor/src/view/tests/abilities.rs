use super::*;

/// The bounds of the Skills column's header (`ui.strong("Skills")`):
/// what the Cards left row's tests scope to.
fn skills_header(nodes: &[accesskit::Node]) -> accesskit::Rect {
    nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some("Skills"))
        .and_then(accesskit::Node::bounds)
        .expect("the Skills heading")
}

/// Whether the Skills column draws a Label with `value` (the column's x
/// at the header's or right of it).
fn in_skills_column(nodes: &[accesskit::Node], value: &str) -> bool {
    let column_x0 = skills_header(nodes).x0;
    nodes.iter().any(|node| {
        node.role() == Role::Label
            && node.value() == Some(value)
            && node.bounds().is_some_and(|b| b.x0 >= column_x0 - 1.0)
    })
}

/// Whether the Skills column draws a Label whose value starts with
/// `prefix`.
fn in_skills_column_prefixed(nodes: &[accesskit::Node], prefix: &str) -> bool {
    let column_x0 = skills_header(nodes).x0;
    nodes.iter().any(|node| {
        node.role() == Role::Label
            && node.value().is_some_and(|v| v.starts_with(prefix))
            && node.bounds().is_some_and(|b| b.x0 >= column_x0 - 1.0)
    })
}

/// The badge texts a frame's Cards left row drew (`"Regular: 3 of 7
/// cards"` or the weak `"No tier matches"`), in left-to-right order: the
/// Labels at the heading's height or below it of the badge's shapes
/// (labels of another shape must not pass).
fn cards_left_badges(nodes: &[accesskit::Node]) -> Vec<String> {
    let heading_y = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some("Cards left"))
        .and_then(accesskit::Node::bounds)
        .map(|bounds| bounds.y0)
        .expect("the Cards left heading");
    let mut badges: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Label
                && node
                    .bounds()
                    .is_some_and(|bounds| bounds.y0 >= heading_y - 1.0)
                && node
                    .value()
                    .is_some_and(|value| value.ends_with(" cards") || value == "No tier matches")
        })
        .collect();
    badges.sort_by(|a, b| {
        let x = |node: &&accesskit::Node| node.bounds().map(|b| b.x0).unwrap_or(0.0);
        x(a).partial_cmp(&x(b)).expect("ordered bounds")
    });
    badges
        .iter()
        .filter_map(|node| node.value().map(str::to_owned))
        .collect()
}

// The same rule for a number control's in-progress edit: typing into one
// player's Speed and selecting another player commits nothing to him.
#[test]
fn a_speed_edit_does_not_reach_the_next_players_control() {
    let path = fixture(PesVersion::Pes17, "cross_player_speed");
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(702);
    let mut window = Headless::sized(1100.0, 700.0);
    let speed_of = |view: &SaveView, id: u32| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(id)
            .unwrap()
            .stats
            .speed
    };

    let file = EditFile::load(&path).unwrap();
    let a_id = file.team(702).unwrap().roster[0].player_id;
    let b_id = file.team(702).unwrap().roster[1].player_id;
    view.selected = Some(Selection::Player(702, 0));
    // The wrapped third column's stats sit below the fold on this window:
    // scroll the Speed row inside the clip before aiming the pointer.
    let nodes = scroll_into_view(&mut window, &mut view, |node| {
        node.role() == Role::Label && node.value() == Some("Speed")
    });
    let speed = right_of_label(&nodes, "Speed", Role::SpinButton)
        .first()
        .copied()
        .expect("the Speed control");
    click(&mut window, speed, &mut view);
    window.frame(vec![egui::Event::Text("55".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let before_b = speed_of(&view, b_id);

    // Select 70202 (roster slot 1) in the same sequence and draw his view:
    // his speed is untouched — and a keystroke reaches no control of his
    // (the in-progress edit stays 70201's own).
    view.selected = Some(Selection::Player(702, 1));
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    window.frame(vec![egui::Event::Text("9".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        speed_of(&view, b_id),
        before_b,
        "70202's Speed is unchanged"
    );
    assert_eq!(
        speed_of(&view, a_id),
        55,
        "the typed value landed on 70201 alone"
    );
}

// SE-GUI-03
#[test]
fn an_ability_edit_marks_the_row_until_ctrl_s_writes_the_save() {
    let path = fixture(PesVersion::Pes21, "ability_edit_save");
    let original = std::fs::read(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);

    // 70101's Speed is 77: focus the control and type a new one. The
    // wrapped third column's stats sit below the fold on this window:
    // scroll the Speed row inside the clip before aiming the pointer.
    let nodes = scroll_into_view(&mut window, &mut view, |node| {
        node.role() == Role::Label && node.value() == Some("Speed")
    });
    let speed = right_of_label(&nodes, "Speed", Role::SpinButton)
        .first()
        .expect("the Speed control")
        .bounds()
        .expect("bounds");
    assert_eq!(
        right_of_label(&nodes, "Speed", Role::SpinButton)[0].value(),
        Some("77"),
        "premise: Speed is 77"
    );
    let at = egui::pos2(
        ((speed.x0 + speed.x1) / 2.0) as f32,
        ((speed.y0 + speed.y1) / 2.0) as f32,
    );
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(vec![egui::Event::Text("55".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    row(&nodes, "1 CB /3/ PLACEHOLDER ●");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "the file is unchanged until Save"
    );

    // Ctrl+S writes it; the dot goes and the reopened file holds the value.
    let bak = path.with_file_name("EDIT00000000.bak");
    assert!(!bak.exists(), "no .bak before the first save");
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    row(&nodes, "1 CB /3/ PLACEHOLDER");
    assert_eq!(
        std::fs::read(&bak).unwrap(),
        original,
        "the .bak is the file as opened"
    );
    let reopened = Session::open(&path).unwrap();
    assert_eq!(
        reopened.file().player(70101).unwrap().stats.speed,
        55,
        "the reopened file holds the speed"
    );
}

// SE-GUI-06
#[test]
fn version_gated_controls_are_disabled_in_place_with_their_first_version() {
    // The stronger hand (the Profile block's gated control; the Cutscenes
    // pair's half of the scenario lives on the Appearance section's test):
    // the same player on PES 17 (no hand stored) and PES 21 (stored), the
    // combo in the same rect, disabled with the `PES NN+` tooltip on 17.
    let bounds_of = |version: PesVersion, test: &str| {
        let path = fixture(version, test);
        let mut view = SaveView::new();
        view.open(&path);
        let mut window = Headless::sized(1100.0, 700.0);
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let hand = control_right_of(&nodes, "Stronger hand")
            .bounds()
            .expect("bounds");
        (view, window, nodes, hand)
    };
    let (mut view17, mut window17, nodes17, hand17) = bounds_of(PesVersion::Pes17, "gated_17");
    let (_view21, _window21, nodes21, hand21) = bounds_of(PesVersion::Pes21, "gated_21");

    let hand_node = right_of_label(&nodes17, "Stronger hand", Role::ComboBox)
        .into_iter()
        .find(|node| node.is_disabled())
        .expect("the disabled hand combo");
    assert_eq!(
        hand_node.value(),
        Some("Right"),
        "the missing hand shows Right"
    );
    assert_eq!(
        tooltip_after_hover(&mut window17, &mut view17, centre(hand_node)),
        Some("PES 20+".to_string()),
        "the hand's tooltip"
    );

    // Enabled on PES 21, the control in the same rect as its PES 17 stand-in.
    assert!(
        !control_right_of(&nodes21, "Stronger hand").is_disabled(),
        "Stronger hand is enabled on PES 21"
    );
    assert_eq!(hand17, hand21, "the Stronger hand control's rect");
}

// plans/save_editor.md "Editing UI", the version-gated widgets bullet: a
// stat the version lacks shows `-`, disabled, its tooltip naming the first
// version that has it; the stats it stores draw enabled.
#[test]
fn a_stat_the_version_lacks_shows_a_dash_disabled() {
    let path = fixture(PesVersion::Pes15, "gated_stats_15");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);

    // The wrapped third column's stats sit below the fold on this window:
    // scroll each gated row inside the clip before aiming the pointer.
    for (label, tooltip) in [("Physical Contact", "PES 17+"), ("Clearing", "PES 16+")] {
        let nodes = scroll_into_view(&mut window, &mut view, |node| {
            node.role() == Role::Label && node.value() == Some(label)
        });
        let button = disabled_dash(&nodes, label);
        assert_eq!(
            tooltip_after_hover(&mut window, &mut view, centre(button)),
            Some(tooltip.to_string()),
            "{label}'s tooltip"
        );
    }
    let nodes = scroll_into_view(&mut window, &mut view, |node| {
        node.role() == Role::Label && node.value() == Some("Speed")
    });
    let speed = right_of_label(&nodes, "Speed", Role::SpinButton)
        .into_iter()
        .next()
        .expect("the Speed control");
    assert!(!speed.is_disabled(), "Speed is enabled on PES 15");
}

// plans/save_editor.md "Editing UI" with `docs/plans/core/gui.md` "Shell
// layout": the pane the shell leaves in its smallest default window
// (1037 × 648, less the sidebar's 56-point rail and the status bar's 24:
// 981 × 624 — the test draws the tighter 877 × 624); the player list
// takes its default width, and the view's field/control pairs wrap below
// one another rather than clip — every control's rect lies inside the
// central panel.
#[test]
fn the_view_fits_the_pane_the_shell_leaves() {
    let path = fixture(PesVersion::Pes21, "shell_pane");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(877.0, 624.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    for role in [
        Role::Button,
        Role::CheckBox,
        Role::ComboBox,
        Role::RadioButton,
        Role::SpinButton,
        Role::TextInput,
    ] {
        for node in nodes.iter().filter(|node| node.role() == role) {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("{role:?} has no bounds"));
            assert!(
                bounds.x1 <= 877.0,
                "{role:?} {} clips the pane's right edge ({bounds:?})",
                node.label().unwrap_or("?")
            );
        }
    }
}

// plans/save_editor.md "Editing UI", the player view bullet (the ability
// stats in one column in the game's order, each label beside its number):
// the 25 stats run top to bottom at one x, each label's SpinButton on its
// row.
#[test]
fn the_stats_fill_one_column_in_the_games_order() {
    let path = fixture(PesVersion::Pes21, "stats_column");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1280.0, 800.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let at = |label: &str| {
        nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(label))
            .unwrap_or_else(|| panic!("no label {label:?}"))
            .bounds()
            .expect("bounds")
    };
    let labels: Vec<String> = pes_savefile::interchange::team_toml::labels::ABILITY_STATS
        .iter()
        .map(|name| pes_savefile::interchange::team_toml::labels::title_case(name))
        .collect();
    let x0 = at(&labels[0]).x0;
    let mut top = f64::MIN;
    for label in &labels {
        let bounds = at(label);
        assert!(
            (bounds.x0 - x0).abs() < 1.0,
            "{label} shares the column's x0"
        );
        assert!(bounds.y0 > top, "{label} sits below the stat above");
        top = bounds.y0;
        assert!(
            !right_of_label(&nodes, label, Role::SpinButton).is_empty(),
            "{label}'s number beside it"
        );
    }
}

// plans/save_editor.md "Editing UI", the Abilities & Skills bullet (the
// playable positions' C/B/A ratings): a click stores its rating in the
// position's own `playable` slot, as one undoable edit.
#[test]
fn a_playable_positions_rating_writes_its_own_slot_and_undoes() {
    let path = fixture(PesVersion::Pes21, "playable_rating");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let playable = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .positions
            .playable
    };
    assert_eq!(playable(&view)[2], 0, "premise: LWF stores C");

    // LWF's three ratings in row order: C, B, A. Click A (stored 2;
    // `playable[2]` is LWF's array slot, not POSITIONS[9]'s index).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let ratings = rating_buttons(&nodes, "LWF");
    click(&mut window, ratings[2], &mut view);
    assert_eq!(
        playable(&view),
        {
            let mut expected = [0u8; 13];
            expected[2] = 2;
            expected
        },
        "A lands in LWF's slot and no other"
    );
    // One Ctrl+Z restores the entry.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(playable(&view)[2], 0, "the one undo restores it");
}

/// The narrowest label-to-field gap over `pairs` (`field.x0 − label.x1`
/// per row).
fn min_gap<S: AsRef<str>>(nodes: &[accesskit::Node], pairs: &[(S, Role)]) -> f64 {
    pairs
        .iter()
        .map(|(label, role)| {
            let label = label.as_ref();
            let label_rect = label_bounds(nodes, label);
            let field = right_of_label(nodes, label, *role)
                .first()
                .unwrap_or_else(|| panic!("a field right of {label:?}"))
                .bounds()
                .expect("bounds");
            field.x0 - label_rect.x1
        })
        .fold(f64::MAX, f64::min)
}

/// `label`'s label-node top, a shorthand for the row checks below.
fn y_of(nodes: &[accesskit::Node], label: &str) -> f64 {
    nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some(label))
        .unwrap_or_else(|| panic!("no label {label:?}"))
        .bounds()
        .expect("bounds")
        .y0
}

// plans/save_editor.md "Editing UI", the positions grid (the draft's
// `POS_ROWS` and `#pos-grid`): the 13 playable positions in five rows by
// line of the pitch, left wing to right wing — LWF CF SS RWF / LMF AMF
// RMF / CMF DMF / LB CB RB / GK, GK alone on the last — each row centred
// in the column, each cell's label centred over its ratings (over the B),
// the registered position's cell outlined with the widgets' radius.
#[test]
fn the_positions_grid_draws_five_centered_rows_with_the_registered_outlined() {
    /// The outline strokes' shapes, painted in the selection colour (the
    /// registered position's cell mark, the draft's `.poscell.reg`).
    fn outlines(shapes: &[egui::epaint::ClippedShape]) -> Vec<egui::epaint::RectShape> {
        let selection = egui::Visuals::dark().selection.stroke.color;
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape)
                    if shape.fill == egui::Color32::TRANSPARENT
                        && shape.stroke.color == selection =>
                {
                    Some(shape.clone())
                }
                _ => None,
            })
            .collect()
    }
    /// The rect holding `position`'s cell widgets (its label over its C
    /// to A ratings, the cell's own edges 3.5 out on each side).
    fn cell_rect(nodes: &[accesskit::Node], position: &str) -> egui::Rect {
        let label = nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(position))
            .unwrap_or_else(|| panic!("no label {position:?}"));
        let ratings = rating_buttons(nodes, position);
        let (x0, x1, top) = (
            label.bounds().expect("bounds").x0 as f32,
            label.bounds().expect("bounds").x1 as f32,
            label.bounds().expect("bounds").y0 as f32,
        );
        egui::Rect::from_min_max(
            egui::pos2(x0.min(ratings[0].bounds().expect("bounds").x0 as f32), top),
            egui::pos2(
                x1.max(ratings[2].bounds().expect("bounds").x1 as f32),
                ratings[2].bounds().expect("bounds").y1 as f32,
            ),
        )
    }

    let path = fixture(PesVersion::Pes21, "positions_grid");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 800.0);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The five rows in the pitch's order, each on its own y, x order as
    // listed, GK alone on the last.
    const EXPECTED: [&[&str]; 5] = [
        &["LWF", "CF", "SS", "RWF"],
        &["LMF", "AMF", "RMF"],
        &["CMF", "DMF"],
        &["LB", "CB", "RB"],
        &["GK"],
    ];
    let mut last_y = f64::MIN;
    for row in EXPECTED {
        let row_y = y_of(&nodes, row[0]);
        assert!(
            row_y > last_y + 5.0,
            "the {row:?} row runs below the one before"
        );
        last_y = row_y;
        let mut last_x = f64::MIN;
        for position in row {
            let bounds = label_bounds(&nodes, position);
            assert!(
                (bounds.y0 - row_y).abs() < 1.0,
                "{position} sits on its row"
            );
            assert!(bounds.x0 > last_x, "{position} follows the one before");
            last_x = bounds.x0;
        }
    }

    // Each row centred in the column: the cells' span's centre at the
    // column's centre (the 62-wide cells' edges sit 3.5 outside the
    // centred ratings row, so the outside ratings' bounds bound the
    // span).
    let column_left = label_bounds(&nodes, "General info").x0;
    let column_centre = column_left + f64::from(crate::view::info_strip::GENERAL_WIDTH) / 2.0;
    for row in EXPECTED {
        let first_c = rating_buttons(&nodes, row[0])[0].bounds().expect("bounds");
        let last_a = rating_buttons(&nodes, row[row.len() - 1])[2]
            .bounds()
            .expect("bounds");
        let centre = (first_c.x0 + last_a.x1) / 2.0;
        assert!(
            (centre - column_centre).abs() < 1.0,
            "the {row:?} row centred in the column ({centre} vs {column_centre})"
        );
    }

    // Each cell's label centred over its ratings, over the B.
    for position in EXPECTED.into_iter().flatten() {
        let label = label_bounds(&nodes, position);
        let ratings = rating_buttons(&nodes, position);
        let b = ratings[1].bounds().expect("bounds");
        let centre = |rect: accesskit::Rect| (rect.x0 + rect.x1) / 2.0;
        assert!(
            (centre(label) - centre(b)).abs() < 1.0,
            "{position}'s label centred over its B"
        );
        assert!(
            label.y1 <= b.y0 + 1.0,
            "{position}'s label sits above its ratings"
        );
    }

    // The registered position's cell is the one outlined cell, stroked
    // with the widgets' own corner radius (every frame and field in the
    // view is rounded); the pick moves the outline with it (70101 is
    // registered CB, picked CF).
    let strokes = outlines(&shapes);
    assert_eq!(strokes.len(), 1, "exactly one outlined cell");
    let radius = window
        .ctx
        .style_of(window.ctx.theme())
        .visuals
        .widgets
        .inactive
        .corner_radius;
    assert_eq!(
        strokes[0].corner_radius, radius,
        "the outline rounded as the widgets are"
    );
    let cell = cell_rect(&nodes, "CB");
    assert!(
        strokes[0].rect.contains_rect(cell),
        "the outline is around the CB cell, got {:?} for {cell:?}",
        strokes[0].rect
    );

    let combo = right_of_label(&nodes, "Registered position", Role::ComboBox)
        .first()
        .copied()
        .expect("the registered combo");
    click(&mut window, combo, &mut view);
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (item, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("CF"))
        .expect("the CF item");
    action_click(&mut window, &mut view, *item);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let strokes = outlines(&shapes);
    assert_eq!(strokes.len(), 1, "still exactly one outlined cell");
    let cell = cell_rect(&nodes, "CF");
    assert!(
        strokes[0].rect.contains_rect(cell),
        "the outline moved to the CF cell, got {:?} for {cell:?}",
        strokes[0].rect
    );
}

// plans/save_editor.md "Editing UI", the Undo/redo bullet (one entry per
// gesture, the merge key the widget's): the cards' and pickers' merge keys
// are the player's own — while a gesture stays open (a widget keeps the
// keyboard), a tick on the next player is his own entry, not merged into
// the first's.
#[test]
fn a_card_tick_on_another_player_is_its_own_undo_entry() {
    let path = fixture(PesVersion::Pes21, "cards_merge_key");
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(702);
    view.selected = Some(Selection::Player(702, 0));
    let mut window = Headless::sized(1100.0, 700.0);
    let file = EditFile::load(&path).unwrap();
    let a_id = file.team(702).unwrap().roster[0].player_id;
    let b_id = file.team(702).unwrap().roster[1].player_id;
    let heading = |view: &SaveView, id: u32| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(id)
            .unwrap()
            .skills
            .skills[6]
    };
    let before_a = heading(&view, a_id);
    let before_b = heading(&view, b_id);

    // The undo gesture stays open while a widget holds the keyboard: focus
    // the filter box — it is drawn for every selection, so the entry
    // survives the player switch — then tick his card through AccessKit.
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (filter, _) = nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::TextInput
                && node.placeholder() == Some("Filter every team (name, shirt name or id)")
        })
        .expect("the filter box");
    let (card_a, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::CheckBox && node.label() == Some("Heading"))
        .expect("the Heading card");
    action_focus(&mut window, &mut view, *filter);
    action_click(&mut window, &mut view, *card_a);
    assert_ne!(heading(&view, a_id), before_a, "the tick landed on him");

    // Select the next player while the gesture stays open (the filter
    // still has the keyboard), and tick the same card there: his own
    // entry, not a merge.
    view.selected = Some(Selection::Player(702, 1));
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (card_b, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::CheckBox && node.label() == Some("Heading"))
        .expect("the Heading card");
    action_click(&mut window, &mut view, *card_b);
    assert_ne!(heading(&view, b_id), before_b, "his tick landed too");

    // One undo reverts the second tick alone: the first player's card
    // stays as ticked, his own entry still on the stack.
    assert!(view.session.as_mut().unwrap().undo());
    assert_eq!(heading(&view, b_id), before_b, "the second tick undone");
    assert_ne!(
        heading(&view, a_id),
        before_a,
        "the first player's tick stands"
    );
    assert!(
        view.session.as_ref().unwrap().can_undo(),
        "the first tick is its own entry"
    );
}

// plans/save_editor.md "Editing UI", the Abilities & Skills bullet (the
// fields shown one above their stored value — star and playing attitude
// stored as shown): a stored 3 shows 4; typing 8 stores 7. The other
// fields of the same player are untouched by each edit. The Profile
// block's four sit in this view, the Cutscenes group's two on Appearance.
#[test]
fn the_shifted_fields_show_one_above_stored_and_store_one_below() {
    let path = fixture(PesVersion::Pes21, "shifted_fields");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    // The six fields the two groups hold: form, injury resistance, weak
    // foot usage and accuracy, star, playing attitude.
    let six = |view: &SaveView| {
        let stats = &view
            .session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .stats;
        (
            stats.form,
            stats.injury_resistance,
            stats.weak_foot_usage,
            stats.weak_foot_accuracy,
            stats.star,
            stats.playing_attitude,
        )
    };
    assert_eq!(
        six(&view),
        (3, 0, 1, 1, Some(0), Some(0)),
        "premise: the stored values"
    );

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let form = right_of_label(&nodes, "Form", Role::SpinButton)
        .first()
        .copied()
        .expect("the Form control");
    assert_eq!(form.value(), Some("4"), "stored 3 shows 4");
    click(&mut window, form, &mut view);
    window.frame(vec![egui::Event::Text("8".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        six(&view),
        (7, 0, 1, 1, Some(0), Some(0)),
        "typed 8 stores 7"
    );

    // The same round trip for the other five fields: each shows the
    // stored value plus its shift (star and playing attitude shift by
    // zero), a typed value stores one less, and the groups' other fields
    // keep their stored values. Star and playing attitude sit in the
    // Appearance section's Cutscenes group, editable while it is locked.
    for (appearance, label, shown, typed, expected) in [
        (
            false,
            "Injury resistance",
            "1",
            "3",
            (7, 2, 1, 1, Some(0), Some(0)),
        ),
        (
            false,
            "Weak foot: Usage",
            "2",
            "4",
            (7, 2, 3, 1, Some(0), Some(0)),
        ),
        (false, "Accuracy", "2", "4", (7, 2, 3, 3, Some(0), Some(0))),
        (true, "Star", "0", "7", (7, 2, 3, 3, Some(7), Some(0))),
        (
            true,
            "Playing attitude",
            "0",
            "3",
            (7, 2, 3, 3, Some(7), Some(3)),
        ),
    ] {
        let nodes = if appearance {
            open_appearance(&mut window, &mut view);
            // The Cutscenes group sits below the fold: the click needs
            // the control inside the scroll's clip.
            scroll_into_view(&mut window, &mut view, |node| {
                node.role() == Role::Label && node.value() == Some(label)
            })
        } else {
            let (_, nodes, _) =
                window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
            nodes
        };
        let control = right_of_label(&nodes, label, Role::SpinButton)
            .first()
            .copied()
            .unwrap_or_else(|| panic!("the {label} control"));
        assert_eq!(control.value(), Some(shown), "{label} shows {shown}");
        click(&mut window, control, &mut view);
        window.frame(vec![egui::Event::Text(typed.to_owned())], |ui| {
            show(ui, &mut view, &|| Vec::new(), None)
        });
        window.frame(
            vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
        assert_eq!(six(&view), expected, "{label} typed {typed}");
    }
}

// plans/save_editor.md "Editing UI", the Abilities & Skills bullet: a value
// the save holds outside a control's range shows as stored and changes only
// on an edit — drawing a player never edits him.
#[test]
fn an_out_of_range_value_draws_as_stored_without_marking_the_player() {
    // Speed 30 is under the control's 40-99 range.
    let path = fixture(PesVersion::Pes21, "draws_no_edit");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().stats.speed = 30;
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    for _ in 0..5 {
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    }
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let speed = right_of_label(&nodes, "Speed", Role::SpinButton)
        .first()
        .copied()
        .expect("the Speed control");
    assert_eq!(speed.value(), Some("30"), "the control draws the stored 30");
    let session = view.session.as_ref().unwrap();
    assert!(!session.is_dirty(), "drawing left no unsaved changes");
    assert!(!session.can_undo(), "drawing pushed no undo entry");
    assert_eq!(
        session.file().player(70101).unwrap().stats.speed,
        30,
        "the value stands as stored"
    );
}

// plans/save_editor.md "Editing UI", the Cards left bullet: one badge, the
// player's own tier under the active ruleset (`label: left of limit
// cards`, `limit - paid`); a ticked skill that no free group covers lowers
// it by one; the `aatf_rules` file's reading is the badge's.
#[test]
fn the_cards_left_badge_reads_the_players_own_tier() {
    /// The badge text `rules` reads for team 701's roster `slot` on the
    /// session's file: his own tier's label over his card economy under
    /// it. The checked player must read as a tier that is not the
    /// ruleset's first — the badge then proves his own tier was read,
    /// not the list's head.
    fn expected(view: &SaveView, rules: &aatf::Rules, slot: u8) -> String {
        let session = view.session.as_ref().unwrap();
        let team = aatf::TeamView::new(session.file(), 701).unwrap();
        let player = team.player(slot).unwrap();
        let tier = rules
            .player_tier(&team, player)
            .unwrap()
            .expect("the checked player matches a tier");
        assert_ne!(
            tier.id,
            rules.ruleset().tiers[0].id,
            "premise: slot {slot}'s own tier is not the ruleset's first"
        );
        let limits = rules.card_limits(&team, player, &tier.id).unwrap();
        format!(
            "{}: {} of {} cards",
            tier.label,
            i32::from(limits.limit) - i32::from(limits.paid),
            limits.limit
        )
    }
    /// The `left` an `expected` badge text carries (`"Regular: 3 of 7
    /// cards"` reads 3).
    fn left_of(view: &SaveView, rules: &aatf::Rules, slot: u8) -> i32 {
        expected(view, rules, slot)
            .split(": ")
            .nth(1)
            .expect("a badge text")
            .split(' ')
            .next()
            .unwrap()
            .parse()
            .expect("the left count")
    }

    let path = fixture(PesVersion::Pes21, "cards_left");
    // Slot 1's player holds Heading (skill 6), a paid card on every tier:
    // his badge is one lower than slot 0's unticked player's.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    let second = file.team(701).unwrap().roster[1].player_id;
    file.player_mut(second).unwrap().skills.skills[6] = true;
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let rules = aatf::Rules::official();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        cards_left_badges(&nodes),
        [expected(&view, &rules, 0)],
        "the one badge, his own tier's"
    );

    // "Heading" (skill 6) is unticked and counts paid on every tier: the
    // tick lowers the badge by one.
    let before = left_of(&view, &rules, 0);
    // The checkbox sits below the fold under the player header: the tick
    // goes through its AccessKit action (a pointer click lands only
    // inside the scroll's clip).
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (heading, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::CheckBox && node.label() == Some("Heading"))
        .expect("the Heading checkbox");
    action_click(&mut window, &mut view, *heading);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        cards_left_badges(&nodes),
        [expected(&view, &rules, 0)],
        "after the tick"
    );
    assert_eq!(left_of(&view, &rules, 0), before - 1, "one card left fewer");

    // Ctrl+Z unticks it: the badge is recomputed, back to the old count.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        cards_left_badges(&nodes),
        [expected(&view, &rules, 0)],
        "after the undo"
    );
    assert_eq!(left_of(&view, &rules, 0), before, "the badge restored");

    // The selection recomputes it: slot 1's player, his Heading held,
    // shows his own one-lower count — not slot 0's.
    view.selected = Some(Selection::Player(701, 1));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        cards_left_badges(&nodes),
        [expected(&view, &rules, 1)],
        "his badge, not slot 0's"
    );

    // A rules file's reading is the badge's: the same view now given the
    // DBC7 file reloads on the changed path (no fresh window).
    let aatf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let data = std::fs::read_to_string(aatf.join("rulesets/dbc7.rhai")).unwrap();
    let logic = std::fs::read_to_string(aatf.join("logic/logic.rhai")).unwrap();
    let rules_path = path.with_file_name("dbc7.4ccrules");
    std::fs::write(
        &rules_path,
        format!(
            "{}\n\n{}\n\n{}\nfn custom_checks(team) {{ [] }}\n",
            data.trim_end(),
            logic.trim_end(),
            "// ==== CUSTOM CHECKS: optional, hand-written; the Ruleset editor keeps this section as it is ===="
        ),
    )
    .unwrap();
    let dbc7 = aatf::Rules::load(&std::fs::read_to_string(&rules_path).unwrap()).unwrap();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    assert_eq!(
        cards_left_badges(&nodes),
        [expected(&view, &dbc7, 1)],
        "the badge under DBC7"
    );
}

// The same bullet, painted: overdrawn past the tier's limit the badge is
// the error colour (slot 0's player made to hold paid cards past his own
// tier's limit by ticking skills in the fixture copy, as the Heading tick
// above does). A colour is no AccessKit property, so it is read back from
// the frame's painted shapes.
#[test]
fn an_overdrawn_badge_paints_in_the_error_colour() {
    /// The color the text `text` was painted in this frame (`galley_color`
    /// on its shape).
    fn painted_color(shapes: &[egui::epaint::ClippedShape], text: &str) -> egui::Color32 {
        let shape = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(shape) if shape.galley.job.text == text => Some(shape),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted text {text:?}"));
        galley_color(shape)
    }

    let path = fixture(PesVersion::Pes21, "cards_left_overdrawn");
    // The first dozen skills ticked: enough paid cards that his own
    // tier's limit is passed.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    let id = file.team(701).unwrap().roster[0].player_id;
    for skill in 0..12 {
        file.player_mut(id).unwrap().skills.skills[skill] = true;
    }
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let rules = aatf::Rules::official();
    let badge = {
        let session = view.session.as_ref().unwrap();
        let team = aatf::TeamView::new(session.file(), 701).unwrap();
        let player = team.player(0).unwrap();
        let tier = rules
            .player_tier(&team, player)
            .unwrap()
            .expect("the player matches a tier");
        let limits = rules.card_limits(&team, player, &tier.id).unwrap();
        assert!(
            i32::from(limits.limit) - i32::from(limits.paid) < 0,
            "premise: overdrawn (paid {} of {})",
            limits.paid,
            limits.limit
        );
        format!(
            "{}: {} of {} cards",
            tier.label,
            i32::from(limits.limit) - i32::from(limits.paid),
            limits.limit
        )
    };
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let error_color = window
        .ctx
        .style_of(window.ctx.theme())
        .visuals
        .error_fg_color;
    assert_eq!(painted_color(&shapes, &badge), error_color);
}

// The same bullet: a player no tier's `when` holds for (`check_team`'s
// `NoTier`) gets the weak stand-in. VTL11's tiers each gate on an exact
// height (gold 195, silver 195, regular 180, buffed 188), so most real
// players match none — PES 21 fixture team 781 slot 0, player 78101.
#[test]
fn a_player_no_tier_matches_gets_the_stand_in() {
    let path = fixture(PesVersion::Pes21, "cards_left_no_tier");
    // VTL11's whole rules file: its data block, the current logic and the
    // block's own CUSTOM CHECKS section.
    let data = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../libs/aatf/rulesets/vtl11.rhai"),
    )
    .unwrap();
    let ruleset = aatf::Ruleset::from_data_block(&data).unwrap();
    let rules_path = path.with_file_name("vtl11.4ccrules");
    std::fs::write(
        &rules_path,
        aatf::assemble_file(
            &ruleset,
            None,
            aatf::file_sections(&data).custom_checks.as_deref(),
        ),
    )
    .unwrap();

    let rules = aatf::Rules::load(&std::fs::read_to_string(&rules_path).unwrap()).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(781, 0));
    {
        let session = view.session.as_ref().unwrap();
        let team = aatf::TeamView::new(session.file(), 781).unwrap();
        let player = team.player(0).unwrap();
        assert_eq!(
            rules.player_tier(&team, player).unwrap().map(|t| &t.id),
            None,
            "premise: 78101 matches no VTL11 tier"
        );
    }
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    assert_eq!(
        cards_left_badges(&nodes),
        ["No tier matches"],
        "the weak stand-in, the one badge"
    );
}

// The same bullet: a rules file that loads but whose `player_tier` fails on
// the selected player (a member's own logic edit) draws the red click line
// in the badge's place, not `No tier matches` (which would tell him the
// opposite), whose click opens the error; and the failure is kept under
// its key like a badge, so the failing script does not run again on every
// frame.
#[test]
fn a_failing_tier_check_shows_its_error_in_the_badges_place() {
    let path = fixture(PesVersion::Pes21, "cards_left_failing_check");
    let logic = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf/logic/logic.rhai"),
    )
    .unwrap();
    let head = "fn player_tier(player, team) {\n";
    assert_eq!(logic.matches(head).count(), 1, "premise: one player_tier");
    let failing = logic.replacen(
        head,
        &format!("{head}    throw \"the tier check fails\";\n"),
        1,
    );
    let rules_path = path.with_file_name("failing.4ccrules");
    std::fs::write(
        &rules_path,
        aatf::assemble_file(aatf::Rules::official().ruleset(), Some(&failing), None),
    )
    .unwrap();

    let rules = aatf::Rules::load(&std::fs::read_to_string(&rules_path).unwrap())
        .expect("premise: the file loads");
    let mut view = SaveView::new();
    view.open(&path);
    {
        let session = view.session.as_ref().unwrap();
        let team = aatf::TeamView::new(session.file(), 701).unwrap();
        assert!(
            rules.player_tier(&team, team.player(0).unwrap()).is_err(),
            "premise: the check fails on the selected player"
        );
    }
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, shapes) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    // `Cards left`, then the red click line in the badge's place.
    const LINE: &str = "Tier check failed, click for more information.";
    let error_node = {
        let column_x0 = skills_header(&nodes).x0;
        nodes
            .iter()
            .find(|node| {
                node.role() == Role::Label
                    && node.value() == Some(LINE)
                    && node.bounds().is_some_and(|b| b.x0 >= column_x0 - 1.0)
            })
            .expect("the tier check's line in the badge's place")
    };
    let style = window.ctx.style_of(window.ctx.theme());
    assert_eq!(
        galley_color(text_shape(&shapes, LINE)),
        style.visuals.error_fg_color,
        "the line in the error colour"
    );
    assert!(
        in_skills_column(&nodes, "Cards left"),
        "Cards left heads it"
    );
    assert!(
        !in_skills_column(&nodes, "No tier matches"),
        "no stand-in beside it"
    );
    let expected = match view.cards.as_ref() {
        Some((_, crate::view::OwnBadge::Failed(error))) => error.clone(),
        _ => panic!("the failure is kept under its key"),
    };
    assert!(
        expected.contains("the tier check fails"),
        "premise: the check's error: {expected}"
    );

    // The dialog is up only after the click.
    assert!(
        !label_value(&nodes, "The tier check failed"),
        "no dialog title before the click"
    );
    assert!(view.refusal.is_none(), "no refusal before the click");

    // Its click opens the refusal dialog with the check's own error.
    click_rules(
        &mut window,
        &mut view,
        error_node,
        Some(rules_path.as_path()),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(rules_path.as_path()))
    });
    assert!(
        label_value(&nodes, "The tier check failed"),
        "the dialog's title"
    );
    assert!(
        label_value(&nodes, &expected),
        "the dialog's body is the check's error"
    );
}

// The same bullet: the Skills header stays clean at the pane the shell
// leaves at its smallest default too (as `the_view_fits`'s): `Skills`
// alone, no badge and no error line.
#[test]
fn a_load_error_draws_nothing_in_the_skills_column_at_a_narrow_pane() {
    let path = fixture(
        PesVersion::Pes21,
        "rules_load_error_at_a_path_long_enough_to_need_the_wrap",
    );
    let dir = path.parent().unwrap().to_path_buf();
    let bad = dir.join("this_is_not_a_rules_file.4ccrules");
    assert!(
        bad.to_str().unwrap().len() >= 80,
        "premise: the path is long: {bad:?}"
    );
    std::fs::write(&bad, "this is no ruleset").unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(877.0, 624.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(bad.as_path()))
    });
    assert!(
        !in_skills_column(&nodes, "Cards left"),
        "no badge under the load error"
    );
    assert!(
        !in_skills_column_prefixed(&nodes, "The AATF rules file could not be loaded"),
        "no error line in the Skills column"
    );
}

// plans/save_editor.md "Editing UI", the Cards left bullet: a file that
// does not load draws no badge; the quick actions under the list report
// it, and the error's text would push the header's row apart. The rest
// of the view draws.
#[test]
fn a_rules_file_that_does_not_load_draws_no_badge() {
    let path = fixture(PesVersion::Pes21, "rules_load_error");
    let dir = path.parent().unwrap().to_path_buf();
    let bad = dir.join("bad.4ccrules");
    std::fs::write(&bad, "this is no ruleset").unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(bad.as_path()))
    });
    assert!(
        in_skills_column(&nodes, "Skills"),
        "the Skills header draws"
    );
    assert!(
        !in_skills_column(&nodes, "Cards left"),
        "no badge under the load error"
    );
    assert!(
        !in_skills_column_prefixed(&nodes, "The AATF rules file could not be loaded"),
        "no error line in the Skills column"
    );
    // The rest of the view draws: a stat control and a skill checkbox.
    right_of_label(&nodes, "Speed", Role::SpinButton);
    assert!(
        nodes
            .iter()
            .any(|node| node.role() == Role::CheckBox && node.label() == Some("Heading")),
        "the skills draw"
    );
}

// plans/save_editor.md "Editing UI", the Abilities & Skills bullet: the
// registered position combo stores the pick (POSITIONS index) as one
// undoable edit.
#[test]
fn the_registered_position_combo_stores_the_pick_as_one_edit() {
    let path = fixture(PesVersion::Pes21, "registered_position");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .positions
            .registered,
        1,
        "premise: registered CB"
    );

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combo = right_of_label(&nodes, "Registered position", Role::ComboBox)
        .first()
        .copied()
        .expect("the registered combo");
    click(&mut window, combo, &mut view);
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (item, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("CF"))
        .expect("the CF item");
    action_click(&mut window, &mut view, *item);
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .positions
            .registered,
        12,
        "CF stores 12"
    );
    // One gesture, one entry: a single undo brings CB back.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .positions
            .registered,
        1
    );
    assert!(!view.session.as_ref().unwrap().can_undo(), "one entry only");
}

// The same bullet (the draft's first row): the stronger-foot combo stores
// the pick (FOOT index) as one undoable edit.
#[test]
fn the_stronger_foot_combo_stores_the_pick_as_one_edit() {
    let path = fixture(PesVersion::Pes21, "stronger_foot");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let foot = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .positions
            .stronger_foot
    };
    assert_eq!(foot(&view), 0, "premise: right-footed");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combo = right_of_label(&nodes, "Stronger foot", Role::ComboBox)
        .first()
        .copied()
        .expect("the foot combo");
    click(&mut window, combo, &mut view);
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (item, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Left"))
        .expect("the Left item");
    action_click(&mut window, &mut view, *item);
    assert_eq!(foot(&view), 1, "Left stores 1");
    // One gesture, one entry: a single undo brings Right back.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(foot(&view), 0, "undone");
    assert!(!view.session.as_ref().unwrap().can_undo(), "one entry only");
}

// The same bullet: the stronger-hand combo (PES 20+) stores the pick the
// same way.
#[test]
fn the_stronger_hand_combo_stores_the_pick_as_one_edit() {
    let path = fixture(PesVersion::Pes21, "stronger_hand");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let hand = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .positions
            .stronger_hand
    };
    assert_eq!(hand(&view), Some(0), "premise: right-handed");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let combo = right_of_label(&nodes, "Stronger hand", Role::ComboBox)
        .first()
        .copied()
        .expect("the hand combo");
    click(&mut window, combo, &mut view);
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (item, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Left"))
        .expect("the Left item");
    action_click(&mut window, &mut view, *item);
    assert_eq!(hand(&view), Some(1), "Left stores 1");
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(hand(&view), Some(0), "undone");
    assert!(!view.session.as_ref().unwrap().can_undo(), "one entry only");
}

// The columns' wrap never re-keys a widget: an egui id follows call
// order inside a scope, which the wrapped row does not change. A field's
// AccessKit id is the egui `Id`: the Ball Control stat's field, the
// Playing style combo and the team view's Short field at one line of
// columns and at wrapped all keep the same id.
#[test]
fn widget_ids_hold_across_the_width_thresholds() {
    let path = fixture(PesVersion::Pes21, "widget_ids");
    let mut view = SaveView::new();
    view.open(&path);
    let id_at = |window: &mut Headless, view: &mut SaveView, label: &str, role: Role| {
        let (_, ids, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
        node_id_right_of(&ids, label, role)
    };

    // The wide window is the layout test's `DEFAULT_PANE`: the three
    // columns on one line; the narrow one wraps the third.
    let mut wide = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    let ball_control = id_at(&mut wide, &mut view, "Ball Control", Role::SpinButton);
    let mut narrow = Headless::sized(877.0, 624.0);
    assert_eq!(
        id_at(&mut narrow, &mut view, "Ball Control", Role::SpinButton),
        ball_control,
        "the stat's field across the wrap"
    );

    let playing_style = id_at(&mut wide, &mut view, "Playing style", Role::ComboBox);
    assert_eq!(
        id_at(&mut narrow, &mut view, "Playing style", Role::ComboBox),
        playing_style,
        "the playing-style combo across the wrap"
    );

    // The team view's Short field across the fields' row break.
    open_team_view(&mut narrow, &mut view, 701);
    let short = id_at(&mut narrow, &mut view, "Short", Role::TextInput);
    open_team_view(&mut wide, &mut view, 701);
    assert_eq!(
        id_at(&mut wide, &mut view, "Short", Role::TextInput),
        short,
        "the view's field across the row break"
    );
}

// SE-GUI-37's player-view clause (plans/save_editor.md "Editing UI", the
// player view bullet): General info, Skills and Ability stats side by side
// left to right on the pane the shell leaves at the 1280 x 800 window
// (`DEFAULT_PANE`), their tops level, Appearance closed below them inside
// the pane's height, every first-half box on its half's right edge and the
// second half's on its label column, every combo on the
// column's, and nothing past the pane's right edge.
// SE-GUI-37
#[test]
fn the_player_view_lays_out_its_three_columns_at_1280() {
    let path = fixture(PesVersion::Pes21, "player_columns");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(DEFAULT_PANE.x, DEFAULT_PANE.y);
    // The combos' label column is measured on the first frames: let it settle.
    for _ in 0..3 {
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    }
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The title line's first text is the player's id in the weak colour.
    let weak = window
        .ctx
        .style_of(window.ctx.theme())
        .visuals
        .weak_text_color();
    let title = text_shape(&shapes, "70101");
    assert_eq!(
        title.galley.job.sections.first().map(|s| {
            (
                s.format.color,
                &title.galley.job.text[s.byte_range.start.0..s.byte_range.end.0],
            )
        }),
        Some((weak, "70101 ")),
        "the title's first run is the weak id"
    );

    // The three columns' first panels: left edges increasing, tops level.
    let titles = ["General info", "Skills", "Ability stats"].map(|t| label_bounds(&nodes, t));
    assert!(
        titles[0].x0 < titles[1].x0 && titles[1].x0 < titles[2].x0,
        "General info, Skills, Ability stats left to right: {titles:?}"
    );
    assert!(
        (titles[0].y0 - titles[1].y0).abs() < 4.0 && (titles[1].y0 - titles[2].y0).abs() < 4.0,
        "the columns' tops level: {titles:?}"
    );

    // Appearance sits closed below the deepest panel's bottom: the last
    // ability stat's row or the positions grid's last, whichever is lower.
    let stat_bottom = right_of_label(&nodes, "Aggression", Role::SpinButton)[0]
        .bounds()
        .expect("bounds")
        .y1
        .max(label_bounds(&nodes, "Aggression").y1);
    let grid_bottom = rating_buttons(&nodes, "GK")
        .iter()
        .map(|node| node.bounds().expect("bounds").y1)
        .fold(0.0, f64::max);
    let deepest = stat_bottom.max(grid_bottom);
    let appearance = node_labeled(&nodes, Role::DisclosureTriangle, "Appearance");
    assert_eq!(
        appearance.is_expanded(),
        Some(false),
        "Appearance is closed"
    );
    assert!(
        appearance.bounds().expect("bounds").y0 >= deepest,
        "Appearance below the deepest panel's bottom ({deepest})"
    );
    // The plan's "fits the pane with no scrolling": the header's bottom
    // inside the pane's height (its content edges less the two
    // `CentralPanel` margins, as `tactics.rs` bounds its deepest node).
    assert!(
        appearance.bounds().expect("bounds").y1 <= f64::from(DEFAULT_PANE.y - 2.0 * PANEL_MARGIN),
        "Appearance's bottom inside the pane ({:?} vs {})",
        appearance.bounds(),
        DEFAULT_PANE.y - 2.0 * PANEL_MARGIN
    );

    // The six label-left rows (Name, Shirt and the four combos) share
    // one label column: their fields start at one x and end flush with
    // the second half's boxes, not on the panel's edge.
    let name = name_field(&nodes).bounds().expect("bounds");
    let shirt = shirt_field(&nodes).bounds().expect("bounds");
    let shirt_label = label_bounds(&nodes, "Shirt number");
    let shirt_box = right_of_label(&nodes, "Shirt number", Role::SpinButton)[0]
        .bounds()
        .expect("bounds");
    let flush = shirt_box.x1;
    assert!(
        (name.x1 - flush).abs() < 0.5 && (shirt.x1 - flush).abs() < 0.5,
        "the counted fields end at the flush edge ({name:?}, {shirt:?} vs {flush})"
    );
    for label in [
        "Stronger foot",
        "Stronger hand",
        "Registered position",
        "Playing style",
    ] {
        let combo = right_of_label(&nodes, label, Role::ComboBox)[0]
            .bounds()
            .expect("bounds");
        assert!(
            (combo.x0 - name.x0).abs() < 0.5,
            "{label} starts at the rows' x ({combo:?} vs {name:?})"
        );
        assert!(
            (combo.x1 - flush).abs() < 0.5,
            "{label} ends at the flush edge ({combo:?} vs {flush})"
        );
    }
    // The counted labels sit on their field's row (the label's vertical
    // centre inside the field's bounds).
    for prefix in ["Name (", "Shirt ("] {
        let label = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Label
                    && node.value().is_some_and(|value| value.starts_with(prefix))
            })
            .and_then(|node| node.bounds())
            .unwrap_or_else(|| panic!("no counted label {prefix:?}"));
        let field = if prefix.starts_with("Shirt") {
            shirt
        } else {
            name
        };
        assert!(
            (label.y0 + label.y1) / 2.0 >= field.y0 - 1.0
                && (label.y0 + label.y1) / 2.0 <= field.y1 + 1.0,
            "the {prefix}label on its field's row ({label:?} vs {field:?})"
        );
    }

    // The pairs' halves 28 points apart: the first half's boxes start at
    // the rows' fields' left edge (the first half is the rows' label
    // column, their 12 and a box), the second half's left edge 28 after
    // its box's right edge.
    let height_box = right_of_label(&nodes, "Height", Role::SpinButton)[0]
        .bounds()
        .expect("bounds");
    for (label, role) in [
        ("Height", Role::SpinButton),
        ("Weight", Role::SpinButton),
        ("Age", Role::SpinButton),
        ("Injury resistance", Role::SpinButton),
        ("Weak foot: Usage", Role::SpinButton),
    ] {
        let bounds = right_of_label(&nodes, label, role)[0]
            .bounds()
            .expect("bounds");
        assert!(
            (bounds.x0 - name.x0).abs() < 0.5,
            "{label}'s box starts at the rows' field x ({bounds:?} vs {name:?})"
        );
    }
    assert!(
        (shirt_label.x0 - height_box.x1 - 28.0).abs() < 0.5,
        "the second half 28 after the first ({shirt_label:?} vs {height_box:?})"
    );
    // The second half's boxes sit just after a label column as wide as
    // "Shirt number" and its 9-point gap (the draft's `72px 54px` grid,
    // the column's `column-gap`): one x across General info's and Other
    // stats' rows, so a short label (Nation, Form) keeps its box close.
    let box_x = shirt_label.x1 + 9.0;
    for (label, role) in [
        ("Shirt number", Role::SpinButton),
        ("Nation", Role::SpinButton),
        ("Captain", Role::RadioButton),
        ("Form", Role::SpinButton),
        ("Accuracy", Role::SpinButton),
    ] {
        let bounds = right_of_label(&nodes, label, role)[0]
            .bounds()
            .expect("bounds");
        assert!(
            (bounds.x0 - box_x).abs() < 0.5,
            "{label}'s box just after the label column ({bounds:?} vs {box_x})"
        );
    }

    // The label-to-field minimums (the plan's "never closer than"): 9 on
    // every pair, 12 in the label-left rows and Ability stats, the
    // narrowest of each at its minimum.
    let pair_min = min_gap(
        &nodes,
        &[
            ("Height", Role::SpinButton),
            ("Weight", Role::SpinButton),
            ("Age", Role::SpinButton),
            ("Shirt number", Role::SpinButton),
            ("Nation", Role::SpinButton),
            ("Captain", Role::RadioButton),
            ("Injury resistance", Role::SpinButton),
            ("Weak foot: Usage", Role::SpinButton),
            ("Form", Role::SpinButton),
            ("Accuracy", Role::SpinButton),
        ],
    );
    assert!(
        (pair_min - 9.0).abs() < 0.5,
        "the narrowest pair gap at 9 ({pair_min})"
    );
    let mut row_min = min_gap(
        &nodes,
        &[
            ("Stronger foot", Role::ComboBox),
            ("Stronger hand", Role::ComboBox),
            ("Registered position", Role::ComboBox),
            ("Playing style", Role::ComboBox),
        ],
    );
    for prefix in ["Name (", "Shirt ("] {
        let label_rect = nodes
            .iter()
            .find(|node| {
                node.role() == Role::Label
                    && node.value().is_some_and(|value| value.starts_with(prefix))
            })
            .and_then(|node| node.bounds())
            .expect("the counted label's bounds");
        let field = field_right_of_label(&nodes, prefix, Role::TextInput)
            .bounds()
            .expect("bounds");
        let gap = field.x0 - label_rect.x1;
        assert!(gap >= 11.5, "{prefix}gap {gap} under 12");
        row_min = row_min.min(gap);
    }
    assert!(
        (row_min - 12.0).abs() < 0.5,
        "the narrowest row gap at 12 ({row_min})"
    );
    let stat_pairs: Vec<(String, Role)> =
        pes_savefile::interchange::team_toml::labels::ABILITY_STATS
            .iter()
            .map(|name| {
                (
                    pes_savefile::interchange::team_toml::labels::title_case(name),
                    Role::SpinButton,
                )
            })
            .collect();
    let stat_min = min_gap(&nodes, &stat_pairs);
    assert!(
        (stat_min - 12.0).abs() < 0.5,
        "the narrowest stat gap at 12 ({stat_min})"
    );

    // The COM styles' second column starts at the skills' second column's x.
    let skills_top = titles[1].y0;
    let com_top = label_bounds(&nodes, "COM playing styles").y0;
    let checkbox_xs = |top: f64, bottom: f64| {
        let mut xs: Vec<f64> = nodes
            .iter()
            .filter(|node| node.role() == Role::CheckBox)
            .filter_map(|node| node.bounds())
            .filter(|b| b.y0 >= top && b.y0 < bottom)
            .map(|b| b.x0)
            .collect();
        xs.sort_by(f64::total_cmp);
        xs.dedup();
        xs
    };
    let skills_xs = checkbox_xs(skills_top, com_top);
    let com_xs = checkbox_xs(com_top, f64::MAX);
    assert_eq!(skills_xs.len(), 2, "the skills' two columns");
    assert_eq!(com_xs.len(), 2, "the COM styles' two columns");
    assert!(
        (skills_xs[1] - com_xs[1]).abs() < 1.0,
        "the second columns share an x ({skills_xs:?} vs {com_xs:?})"
    );

    // The columns 36 points apart: each column's left minus the previous
    // column's right edge (the General info fields' edge, the card
    // columns' second one's own width out).
    assert!(
        (titles[1].x0 - name.x1 - 36.0).abs() <= 1.0,
        "the Skills column 36 points after General info ({:?} vs {name:?})",
        titles[1]
    );
    let cards_right = skills_xs[1] + f64::from(team_widgets::cards::CARD_COLUMN_WIDTH);
    assert!(
        (titles[2].x0 - cards_right - 36.0).abs() <= 1.0,
        "the Ability stats column 36 points after the cards ({:?} vs {cards_right})",
        titles[2]
    );

    // Nothing drawn past the pane's right edge, the pane's right less
    // the two CentralPanel margins (`PANEL_MARGIN` each side).
    let pane_right = f64::from(DEFAULT_PANE.x - 2.0 * PANEL_MARGIN);
    for role in [
        Role::Button,
        Role::CheckBox,
        Role::ComboBox,
        Role::RadioButton,
        Role::SpinButton,
        Role::TextInput,
    ] {
        for node in nodes.iter().filter(|node| node.role() == role) {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("{role:?} has no bounds"));
            assert!(
                bounds.x1 <= pane_right + 0.5,
                "{role:?} {} clips the pane's right edge ({bounds:?} vs {pane_right})",
                node.label().unwrap_or("?")
            );
        }
    }
}

// plans/save_editor.md "Editing UI", the player view bullet ("a columned
// panel narrows into fewer columns"): on a pane too narrow for the two
// card columns and their gap (the 560-point window leaves the view under
// 300, under the 330 the two need) the Skills column draws one column, no
// checkbox past the pane's right edge.
#[test]
fn the_skills_column_narrows_to_one_column_on_a_narrow_pane() {
    let path = fixture(PesVersion::Pes21, "skills_one_column");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(560.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let skills_top = label_bounds(&nodes, "Skills").y0;
    let com_top = label_bounds(&nodes, "COM playing styles").y0;
    let mut xs: Vec<f64> = nodes
        .iter()
        .filter(|node| node.role() == Role::CheckBox)
        .filter_map(|node| node.bounds())
        .filter(|b| b.y0 >= skills_top && b.y0 < com_top)
        .map(|b| b.x0)
        .collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    assert_eq!(xs.len(), 1, "the skills draw in one column ({xs:?})");
    let pane_right = 560.0 - 2.0 * f64::from(PANEL_MARGIN);
    for node in nodes.iter().filter(|node| node.role() == Role::CheckBox) {
        let bounds = node.bounds().expect("bounds");
        assert!(
            bounds.x1 <= pane_right + 0.5,
            "{:?} inside the pane's right edge ({bounds:?} vs {pane_right})",
            node.label()
        );
    }
}

// plans/save_editor.md "Editing UI", the player view bullet's first
// sentence: the view centred between the player list and the window's
// right edge — the gap after the list equals the margin after Ability
// stats to the window's right, the room the fixed columns leave split
// evenly, on every pane width.
// The centring pad reads the columns' width the previous frame measured
// (egui memory), so three settle frames run before the read.
#[test]
fn the_player_view_centres_its_columns_in_the_pane() {
    let path = fixture(PesVersion::Pes21, "view_centered");
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    for screen_w in [DEFAULT_PANE.x, DEFAULT_PANE.x + 300.0] {
        let mut window = Headless::sized(screen_w, DEFAULT_PANE.y);
        for _ in 0..3 {
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        }
        let (_, nodes, shapes) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

        let gen_left = label_bounds(&nodes, "General info").x0;
        let stats_left = label_bounds(&nodes, "Ability stats").x0;
        let stats_right = nodes
            .iter()
            .filter(|node| node.role() == Role::SpinButton)
            .filter_map(|node| node.bounds())
            .filter(|bounds| bounds.x0 >= stats_left - 1.0)
            .map(|bounds| bounds.x1)
            .fold(0.0, f64::max);
        let (list_right, _) = pane_frame(&shapes, window.screen);
        let left = gen_left - list_right;
        let right = f64::from(screen_w) - stats_right;
        assert!(
            (left - right).abs() < 1.0,
            "the gap after the list ({left}) equals the margin after Ability stats ({right}) at {screen_w}"
        );
    }
}

/// The color the letter's galley painted inside `bounds`: a section's own
/// color when one was set, else the shape's fallback (the widget's).
fn letter_color(
    shapes: &[egui::epaint::ClippedShape],
    letter: &str,
    bounds: accesskit::Rect,
) -> egui::Color32 {
    let shape = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape)
                if shape.galley.job.text == letter
                    && f64::from(shape.pos.y) >= bounds.y0
                    && f64::from(shape.pos.y) <= bounds.y1
                    && f64::from(shape.pos.x) >= bounds.x0
                    && f64::from(shape.pos.x) <= bounds.x1 =>
            {
                Some(shape)
            }
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("no painted {letter:?} inside {bounds:?}"));
    galley_color(shape)
}

// The chosen rating's fill and letter (the draft's `.sel.on` variants):
// A the success green with a dark letter, B the selection colour with
// its foreground, C a grey square with the normal text, so the three
// tell apart at a glance.
#[test]
fn the_chosen_ratings_paint_their_fills_and_letters() {
    let path = fixture(PesVersion::Pes21, "rating_colors");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    let mut playable = file.player(70101).unwrap().positions.playable;
    // CF at A (stored 2), SS at B (1), LWF at C (0), the first three
    // `playable` entries standing for them.
    assert_ne!(
        [playable[0], playable[1], playable[2]],
        [2, 1, 0],
        "premise: the fixture does not already hold A, B, C"
    );
    playable[0] = 2;
    playable[1] = 1;
    playable[2] = 0;
    file.player_mut(70101).unwrap().positions.playable = playable;
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::sized(1100.0, 800.0);
    let visuals = window.ctx.style_of(window.ctx.theme()).visuals.clone();
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let cases = [
        (
            "CF",
            2,
            studio_core::theme::SUCCESS_FG,
            egui::Color32::from_gray(0x1a),
        ),
        (
            "SS",
            1,
            visuals.selection.bg_fill,
            visuals.selection.stroke.color,
        ),
        (
            "LWF",
            0,
            egui::Color32::from_rgb(0x5a, 0x5a, 0x5a),
            visuals.text_color(),
        ),
    ];
    for (position, rating, fill, text) in cases {
        let letter = ["C", "B", "A"][rating];
        let button = rating_buttons(&nodes, position)[rating];
        let bounds = button.bounds().expect("bounds");
        assert_eq!(
            fill_at(&shapes, bounds),
            Some(fill),
            "{position}'s {letter} square fills {fill:?}"
        );
        assert_eq!(
            letter_color(&shapes, letter, bounds),
            text,
            "{position}'s {letter} letter in {text:?}"
        );
    }
}
