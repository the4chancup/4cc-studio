use super::*;

/// The load error the Cards left row draws: the error label at the pane's
/// "Cards left" heading's height or below it (the error wraps to the lines
/// under the heading when the pane is narrow). The quick actions column
/// draws the same text at the tab's top, and its copy must not pass for
/// the row's.
fn cards_left_error(nodes: &[accesskit::Node]) -> Option<&accesskit::Node> {
    let heading = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some("Cards left"))
        .and_then(accesskit::Node::bounds)
        .map(|bounds| bounds.y0)
        .expect("the Cards left heading");
    nodes.iter().find(|node| {
        node.role() == Role::Label
            && node
                .value()
                .is_some_and(|value| value.starts_with("The AATF rules file does not load:"))
            && node.bounds().is_some_and(|b| b.y0 >= heading - 1.0)
    })
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
    view.selected = Some((702, 0));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let speed = right_of_label(&nodes, "Speed", Role::SpinButton)
        .first()
        .copied()
        .expect("the Speed control");
    click(&mut window, speed, &mut view);
    window.frame(vec![egui::Event::Text("55".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let before_b = speed_of(&view, b_id);

    // Select 70202 (roster slot 1) in the same sequence and draw his tab:
    // his speed is untouched — and a keystroke reaches no control of his
    // (the in-progress edit stays 70201's own).
    view.selected = Some((702, 1));
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

    // 70101's Speed is 77: focus the control and type a new one.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
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
    // pair's half of the scenario lives on the Appearance tab's test):
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    for (label, tooltip) in [("Physical Contact", "PES 17+"), ("Clearing", "PES 16+")] {
        let button = disabled_dash(&nodes, label);
        assert_eq!(
            tooltip_after_hover(&mut window, &mut view, centre(button)),
            Some(tooltip.to_string()),
            "{label}'s tooltip"
        );
    }
    let speed = right_of_label(&nodes, "Speed", Role::SpinButton)
        .into_iter()
        .next()
        .expect("the Speed control");
    assert!(!speed.is_disabled(), "Speed is enabled on PES 15");
}

// plans/save_editor.md "Editing UI" with `docs/plans/core/gui.md` "Shell
// layout": the pane the shell leaves in its smallest default window
// (1037 × 648, less the expanded sidebar's 160 and the status bar's 24:
// 877 × 624); the player list takes its default width, and the
// tab's field/control pairs wrap below one another rather than clip —
// every control's rect lies inside the central panel.
#[test]
fn the_tab_fits_the_pane_the_shell_leaves() {
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

// plans/save_editor.md "Editing UI", the Abilities & Skills bullet (the
// stats in aligned columns filled column by column in the game's order,
// fewer columns when the pane cannot hold four): at 1100 the four columns
// are headed by Attacking Prowess, Swerve, Body Control and Coverage, the
// first column running the game's first seven stats top to bottom; at the
// pane the shell's smallest default window leaves the columns shrink to
// three, the first holding the first nine.
#[test]
fn the_stats_fill_aligned_columns_column_by_column() {
    /// The y of a label's label node and the x of its left edge.
    fn label_at<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(label))
            .unwrap_or_else(|| panic!("no label {label:?}"))
    }
    let first_column = |nodes: &[accesskit::Node], expected: &[&str]| {
        let x0 = label_at(nodes, expected[0]).bounds().expect("bounds").x0;
        let mut top = f64::MIN;
        for &label in expected {
            let bounds = label_at(nodes, label).bounds().expect("bounds");
            assert_eq!(bounds.x0, x0, "{label} shares the column's x0");
            assert!(bounds.y0 > top, "{label} sits below the stat above");
            top = bounds.y0;
        }
    };

    let path = fixture(PesVersion::Pes21, "stats_columns");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 800.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let head_y = label_at(&nodes, "Attacking Prowess")
        .bounds()
        .expect("bounds")
        .y0;
    let mut last_x = 0.0;
    for label in ["Attacking Prowess", "Swerve", "Body Control", "Coverage"] {
        let bounds = label_at(&nodes, label).bounds().expect("bounds");
        assert!(
            (bounds.y0 - head_y).abs() < 1.0,
            "{label} heads a column on the first row"
        );
        assert!(bounds.x0 > last_x, "{label} heads a column to the right");
        last_x = bounds.x0;
    }
    first_column(
        &nodes,
        &[
            "Attacking Prowess",
            "Ball Control",
            "Dribbling",
            "Low Pass",
            "Lofted Pass",
            "Finishing",
            "Place Kicking",
        ],
    );

    // The same tab on the 877 x 624 pane: three columns of nine.
    let path = fixture(PesVersion::Pes21, "stats_columns_narrow");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(877.0, 624.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    first_column(
        &nodes,
        &[
            "Attacking Prowess",
            "Ball Control",
            "Dribbling",
            "Low Pass",
            "Lofted Pass",
            "Finishing",
            "Place Kicking",
            "Swerve",
            "Heading",
        ],
    );
}

// plans/save_editor.md "Editing UI", the tab strip bullet: the tabs in two
// groups, each after its weak scope label — `Player` (Abilities & Skills,
// Appearance) then `Team` (Tactics, Team settings) — Abilities & Skills
// selected and the other three pickable.
#[test]
fn the_tab_strip_groups_the_tabs_under_weak_scope_labels() {
    let path = fixture(PesVersion::Pes21, "tab_strip");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    fn tab<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        // `toggled` marks the selectable tabs: the toolbar's "Team" menu
        // button shares the group label but is not selectable.
        nodes
            .iter()
            .find(|node| {
                node.role() == Role::Button
                    && node.label() == Some(label)
                    && node.toggled().is_some()
            })
            .unwrap_or_else(|| panic!("no tab {label:?}"))
    }
    // The strip's row in x order: the weak scope labels (Label nodes) and
    // the selectable tabs, on the selected tab's row.
    let strip_y = tab(&nodes, "Abilities & Skills")
        .bounds()
        .expect("bounds")
        .y0;
    let mut row: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            let wanted = match node.role() {
                Role::Button => {
                    node.toggled().is_some()
                        && matches!(
                            node.label(),
                            Some("Abilities & Skills" | "Appearance" | "Tactics" | "Team settings")
                        )
                }
                Role::Label => matches!(node.value(), Some("Player" | "Team")),
                _ => false,
            };
            wanted && node.bounds().is_some_and(|b| (b.y0 - strip_y).abs() < 4.0)
        })
        .collect();
    row.sort_by(|a, b| {
        let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
        x(a).partial_cmp(&x(b)).expect("ordered bounds")
    });
    let texts: Vec<&str> = row
        .iter()
        .map(|node| match node.role() {
            Role::Label => node.value().expect("a label's text"),
            _ => node.label().expect("a tab's label"),
        })
        .collect();
    assert_eq!(
        texts,
        [
            "Player",
            "Abilities & Skills",
            "Appearance",
            "Team",
            "Tactics",
            "Team settings"
        ],
        "the strip's labels in order"
    );

    let selected = tab(&nodes, "Abilities & Skills");
    assert_eq!(
        selected.toggled(),
        Some(accesskit::Toggled::True),
        "Abilities & Skills is the drawn tab"
    );
    assert!(!selected.is_disabled());
    for label in ["Appearance", "Tactics", "Team settings"] {
        let tab = tab(&nodes, label);
        assert!(!tab.is_disabled(), "{label} picks");
        assert_eq!(tab.toggled(), Some(accesskit::Toggled::False));
    }

    // A click switches the drawn tab.
    click(&mut window, tab(&nodes, "Appearance"), &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        tab(&nodes, "Appearance").toggled(),
        Some(accesskit::Toggled::True),
        "Appearance is the drawn tab"
    );
    assert_eq!(
        tab(&nodes, "Abilities & Skills").toggled(),
        Some(accesskit::Toggled::False)
    );
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

// plans/save_editor.md "Editing UI", the Abilities & Skills bullet (the
// Profile block): the eight label/field pairs in the draft's order, two
// pairs per row, each column's four labels sharing an x0.
#[test]
fn the_profile_block_holds_its_pairs_in_two_aligned_columns() {
    let path = fixture(PesVersion::Pes21, "profile_block");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 800.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let at = |label: &str| {
        nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(label))
            .unwrap_or_else(|| panic!("no label {label:?}"))
            .bounds()
            .expect("bounds")
    };
    for (left, right) in [
        ("Registered position", "Playing style"),
        ("Stronger foot", "Stronger hand"),
        ("Form", "Injury resistance"),
        ("Weak foot usage", "Weak foot accuracy"),
    ] {
        assert!(
            (at(left).y0 - at(right).y0).abs() < 1.0,
            "{left} and {right} share a row"
        );
    }
    for column in [
        [
            "Registered position",
            "Stronger foot",
            "Form",
            "Weak foot usage",
        ],
        [
            "Playing style",
            "Stronger hand",
            "Injury resistance",
            "Weak foot accuracy",
        ],
    ] {
        let x0 = at(column[0]).x0;
        for label in &column[1..] {
            assert!(
                (at(label).x0 - x0).abs() < 1.0,
                "{label} shares the column's x0"
            );
        }
    }
}

// plans/save_editor.md "Editing UI", the top row's three blocks: on a wide
// pane Profile, Playable positions and Quick actions share the first
// line; on the pane the shell's smallest default window leaves the row
// wraps between the blocks — Playable positions lands below Profile
// whole, never split across two lines.
#[test]
fn the_top_row_wraps_between_its_blocks_never_inside_one() {
    let path = fixture(PesVersion::Pes21, "top_row_wide");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1440.0, 900.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let y = |label: &str| {
        nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(label))
            .unwrap_or_else(|| panic!("no heading {label:?}"))
            .bounds()
            .expect("bounds")
            .y0
    };
    let row_y = y("Profile");
    for heading in ["Playable positions", "Quick actions"] {
        assert!(
            (y(heading) - row_y).abs() < 1.0,
            "{heading} shares the top row"
        );
    }

    let path = fixture(PesVersion::Pes21, "top_row_narrow");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(877.0, 624.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let second = y_of(&nodes, "Playable positions");
    assert!(
        second > y_of(&nodes, "Profile") + 5.0,
        "the block wrapped whole below Profile"
    );
    assert!(
        (y_of(&nodes, "Quick actions") - second).abs() < 1.0,
        "Quick actions shares Playable positions' line"
    );
}

/// `label`'s label-node top, a shorthand for the narrow-pane checks above.
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
// `POS_ROWS`): the 13 playable positions in the collaborator's rows,
// CF SS LWF RWF / AMF LMF RMF CMF DMF / CB LB RB GK, each cell the
// position's label above its C B A ratings, the registered position's
// cell outlined.
#[test]
fn the_positions_grid_draws_the_collaborators_rows_with_the_registered_outlined() {
    /// The outline strokes a frame painted in the selection colour (the
    /// registered position's cell mark, the draft's `td.reg`).
    fn outlines(shapes: &[egui::epaint::ClippedShape]) -> Vec<egui::Rect> {
        let selection = egui::Visuals::dark().selection.stroke.color;
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape)
                    if shape.fill == egui::Color32::TRANSPARENT
                        && shape.stroke.color == selection =>
                {
                    Some(shape.rect)
                }
                _ => None,
            })
            .collect()
    }
    /// The rect holding `position`'s cell widgets (its label and its C
    /// rating, enough to name the cell).
    fn cell_rect(nodes: &[accesskit::Node], position: &str) -> egui::Rect {
        let label = nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(position))
            .unwrap_or_else(|| panic!("no label {position:?}"));
        let c = rating_buttons(nodes, position)[0];
        egui::Rect::from_min_max(
            egui::pos2(
                label.bounds().expect("bounds").x0 as f32,
                label.bounds().expect("bounds").y0 as f32,
            ),
            egui::pos2(
                c.bounds().expect("bounds").x1 as f32,
                c.bounds().expect("bounds").y1 as f32,
            ),
        )
    }

    let path = fixture(PesVersion::Pes21, "positions_grid");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 800.0);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The three rows in the collaborator's order, one y0 each, x order as
    // listed.
    for row in [
        ["CF", "SS", "LWF", "RWF"].as_slice(),
        ["AMF", "LMF", "RMF", "CMF", "DMF"].as_slice(),
        ["CB", "LB", "RB", "GK"].as_slice(),
    ] {
        let row_y = y_of(&nodes, row[0]);
        let mut last_x = f64::MIN;
        for position in row {
            let bounds = nodes
                .iter()
                .find(|node| node.role() == Role::Label && node.value() == Some(*position))
                .unwrap_or_else(|| panic!("no label {position:?}"))
                .bounds()
                .expect("bounds");
            assert!(
                (bounds.y0 - row_y).abs() < 1.0,
                "{position} sits on its row"
            );
            assert!(bounds.x0 > last_x, "{position} follows the one before");
            last_x = bounds.x0;
        }
    }
    // Each label sits above its own ratings.
    for position in [
        "CF", "SS", "LWF", "RWF", "AMF", "LMF", "RMF", "CMF", "DMF", "CB", "LB", "RB", "GK",
    ] {
        let label = nodes
            .iter()
            .find(|node| node.role() == Role::Label && node.value() == Some(position))
            .unwrap_or_else(|| panic!("no label {position:?}"))
            .bounds()
            .expect("bounds");
        let c = rating_buttons(&nodes, position)[0]
            .bounds()
            .expect("bounds");
        assert!(
            label.y1 <= c.y0 + 1.0,
            "{position}'s label sits above its C"
        );
        assert!(
            (label.x0 - c.x0).abs() < 4.0,
            "{position}'s label and C start at the cell's left edge"
        );
    }

    // The registered position's cell is the one outlined cell; the pick
    // moves the outline with it (70101 is registered CB, picked CF).
    let strokes = outlines(&shapes);
    assert_eq!(strokes.len(), 1, "exactly one outlined cell");
    let cell = cell_rect(&nodes, "CB");
    assert!(
        strokes[0].contains_rect(cell),
        "the outline is around the CB cell, got {:?} for {cell:?}",
        strokes[0]
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
        strokes[0].contains_rect(cell),
        "the outline moved to the CF cell, got {:?} for {cell:?}",
        strokes[0]
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
    view.selected = Some((702, 0));
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
                && node.placeholder() == Some("Filter all teams (name, shirt, id)")
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
    view.selected = Some((702, 1));
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
// block's four sit on this tab, the Cutscenes group's two on Appearance.
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
    // keep their stored values. Star and playing attitude sit in
    // Appearance's Cutscenes group, editable while the tab is locked.
    for (tab, label, shown, typed, expected) in [
        (
            Tab::Abilities,
            "Injury resistance",
            "1",
            "3",
            (7, 2, 1, 1, Some(0), Some(0)),
        ),
        (
            Tab::Abilities,
            "Weak foot usage",
            "2",
            "4",
            (7, 2, 3, 1, Some(0), Some(0)),
        ),
        (
            Tab::Abilities,
            "Weak foot accuracy",
            "2",
            "4",
            (7, 2, 3, 3, Some(0), Some(0)),
        ),
        (
            Tab::Appearance,
            "Star",
            "0",
            "7",
            (7, 2, 3, 3, Some(7), Some(0)),
        ),
        (
            Tab::Appearance,
            "Playing attitude",
            "0",
            "3",
            (7, 2, 3, 3, Some(7), Some(3)),
        ),
    ] {
        view.tab = tab;
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
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

// plans/save_editor.md "Editing UI", the Cards left bullet: one badge per
// tier of the active ruleset, `limit - paid`; a ticked skill that no free
// group covers lowers every badge by one; the `aatf_rules` file's tiers are
// the badges'; a file that does not load shows its error instead.
#[test]
fn the_cards_left_badges_read_the_active_ruleset() {
    /// The badge texts a frame drew (`"Gold 6"`), in left-to-right order:
    /// at the "Cards left" heading's height or below it (the team bar's
    /// "Colour 1"/"Colour 2" labels match the shape and must not pass).
    fn badges(nodes: &[accesskit::Node]) -> Vec<String> {
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
                    && node.value().is_some_and(|value| {
                        value
                            .split_once(' ')
                            .is_some_and(|(_, rest)| rest.parse::<i32>().is_ok())
                    })
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
    /// Each tier's `limit - paid` as `Rules::card_limits` gives it on the
    /// session's file, team 701, roster `slot`.
    fn expected(view: &SaveView, rules: &aatf::Rules, slot: u8) -> Vec<String> {
        let session = view.session.as_ref().unwrap();
        let team = aatf::TeamView::new(session.file(), 701).unwrap();
        let player = team.player(slot).unwrap();
        rules
            .ruleset()
            .tiers
            .iter()
            .map(|tier| {
                let limits = rules.card_limits(&team, player, &tier.id).unwrap();
                format!(
                    "{} {}",
                    tier.label,
                    i32::from(limits.limit) - i32::from(limits.paid)
                )
            })
            .collect()
    }

    let path = fixture(PesVersion::Pes21, "cards_left");
    // Slot 1's player holds Heading (skill 6), a paid card on every tier:
    // his badges are one lower than slot 0's unticked player's.
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
        badges(&nodes),
        expected(&view, &rules, 0),
        "the tiers' badges"
    );

    // "Heading" (skill 6) is unticked and counts paid on every tier: the
    // tick lowers each badge by one.
    let left_of = |view: &SaveView, slot: u8| -> Vec<i32> {
        expected(view, &rules, slot)
            .iter()
            .map(|badge| badge.rsplit(' ').next().unwrap().parse().unwrap())
            .collect()
    };
    let before = left_of(&view, 0);
    assert_eq!(before, [6, 5, 4, 2, 3], "the unticked player's badges");
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
    assert_eq!(badges(&nodes), expected(&view, &rules, 0), "after the tick");
    assert_eq!(
        left_of(&view, 0),
        before.iter().map(|left| left - 1).collect::<Vec<i32>>(),
        "every badge one lower"
    );

    // Ctrl+Z unticks it: the badges are recomputed, back to the old counts.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(badges(&nodes), expected(&view, &rules, 0), "after the undo");
    assert_eq!(left_of(&view, 0), before, "every badge restored");

    // The selection recomputes them: slot 1's player, his Heading held,
    // shows his own one-lower counts — not slot 0's.
    view.selected = Some((701, 1));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(left_of(&view, 1), [5, 4, 3, 1, 2], "his paid counts");
    assert_eq!(
        badges(&nodes),
        expected(&view, &rules, 1),
        "his badges, not slot 0's"
    );

    // A rules file's tiers are the badges': the same view now given the
    // DBC7 file reloads on the changed path (no fresh window).
    let aatf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let data = std::fs::read_to_string(aatf.join("rulesets/dbc7.rhai")).unwrap();
    let logic = std::fs::read_to_string(aatf.join("logic/logic.rhai")).unwrap();
    let rules_path = path.with_file_name("dbc7.rhai");
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
    assert_eq!(badges(&nodes), expected(&view, &dbc7, 1), "DBC7's tiers");
    assert!(
        badges(&nodes)
            .iter()
            .all(|badge| !badge.starts_with("Bronze")),
        "no Bronze: {:?}",
        badges(&nodes)
    );
}

// The same bullet, read: the error names the file and its parse error, so
// it wraps inside the pane rather than clip at its right edge (the pane
// the shell leaves at its smallest default, as `the_tab_fits`'s).
#[test]
fn a_rules_files_load_error_wraps_inside_the_pane() {
    let path = fixture(
        PesVersion::Pes21,
        "rules_load_error_at_a_path_long_enough_to_need_the_wrap",
    );
    let dir = path.parent().unwrap().to_path_buf();
    let bad = dir.join("this_is_not_a_rules_file.rhai");
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
    let error = cards_left_error(&nodes).expect("the load error's row");
    let bounds = error.bounds().expect("bounds");
    assert!(
        bounds.x1 <= 877.0,
        "the load error ends inside the window, drew {bounds:?}"
    );
}

// plans/save_editor.md "Editing UI", the Cards left bullet: a file that
// does not load shows its error in the badges' place, and the rest of the
// tab draws.
#[test]
fn a_rules_file_that_does_not_load_shows_its_error() {
    let path = fixture(PesVersion::Pes21, "rules_load_error");
    let dir = path.parent().unwrap().to_path_buf();
    let bad = dir.join("bad.rhai");
    std::fs::write(&bad, "this is no ruleset").unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| {
        show(ui, &mut view, &|| Vec::new(), Some(bad.as_path()))
    });
    assert!(
        cards_left_error(&nodes).is_some(),
        "the load error in the Cards left row"
    );
    // The rest of the tab draws: a stat control and a skill checkbox.
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
