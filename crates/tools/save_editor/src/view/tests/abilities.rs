use super::*;

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
    // The same player on PES 17 (nothing gated stored) and PES 21 (all
    // stored): the three controls draw in the same rects, disabled with the
    // `PES NN+` tooltip on 17 and enabled on 21.
    let bounds_of = |version: PesVersion, test: &str| {
        let path = fixture(version, test);
        let mut view = SaveView::new();
        view.open(&path);
        let mut window = Headless::sized(1100.0, 700.0);
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let star = control_right_of(&nodes, "Star").bounds().expect("bounds");
        let attitude = control_right_of(&nodes, "Playing attitude")
            .bounds()
            .expect("bounds");
        let hand = control_right_of(&nodes, "Stronger hand")
            .bounds()
            .expect("bounds");
        (view, window, nodes, (star, attitude, hand))
    };
    let (mut view17, mut window17, nodes17, (star17, attitude17, hand17)) =
        bounds_of(PesVersion::Pes17, "gated_17");
    let (_view21, _window21, nodes21, (star21, attitude21, hand21)) =
        bounds_of(PesVersion::Pes21, "gated_21");

    // Disabled on PES 17, each naming the first version that stores it.
    for (control, tooltip) in [("Star", "PES 19+"), ("Playing attitude", "PES 20+")] {
        let button = disabled_dash(&nodes17, control);
        assert_eq!(
            tooltip_after_hover(&mut window17, &mut view17, centre(button)),
            Some(tooltip.to_string()),
            "{control}'s tooltip"
        );
    }
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

    // Enabled on PES 21, each control in the same rect as its PES 17 stand-in.
    for label in ["Star", "Playing attitude", "Stronger hand"] {
        assert!(
            !control_right_of(&nodes21, label).is_disabled(),
            "{label} is enabled on PES 21"
        );
    }
    assert_eq!(star17, star21, "the Star control's rect");
    assert_eq!(
        attitude17, attitude21,
        "the Playing attitude control's rect"
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
// layout" (Studio opens at half the monitor, 960 × 540 on 1080p, the
// expanded sidebar's 160 and the status bar's 24 spent): the pane the shell
// leaves is 800 × 516, the player list takes its default width, and the
// tab's field/control pairs wrap below one another rather than clip —
// every control's rect lies inside the central panel.
#[test]
fn the_tab_fits_the_pane_the_shell_leaves() {
    let path = fixture(PesVersion::Pes21, "shell_pane");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(800.0, 516.0);
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
                bounds.x1 <= 800.0,
                "{role:?} {} clips the pane's right edge ({bounds:?})",
                node.label().unwrap_or("?")
            );
        }
    }
}

// The same layout rule, pinned at a known width: an ~880-point pane takes
// five label/control pairs of the small stats (170 each) and of the
// ability stats (160 each) a row — the first five sit on the first row's
// y, the sixth wraps.
#[test]
fn the_stats_rows_wrap_at_the_panes_width() {
    let path = fixture(PesVersion::Pes21, "stats_reflow");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let y = |label: &str| control_right_of(&nodes, label).bounds().expect("bounds").y0;
    let row_one = y("Attacking Prowess");
    for label in ["Ball Control", "Dribbling", "Low Pass", "Lofted Pass"] {
        assert!(
            (y(label) - row_one).abs() < 1.0,
            "{label} sits on the first row"
        );
    }
    assert!(y("Finishing") > row_one + 5.0, "the sixth ability wraps");
    let row_one = y("Form");
    for label in [
        "Injury resistance",
        "Weak foot usage",
        "Weak foot accuracy",
        "Star",
    ] {
        assert!(
            (y(label) - row_one).abs() < 1.0,
            "{label} sits on the first row"
        );
    }
    assert!(
        y("Playing attitude") > row_one + 5.0,
        "the sixth small stat wraps"
    );
}

// plans/save_editor.md "Editing UI", the Tabs bullet: the four tabs draw,
// Abilities & Skills selected, the rest disabled until their slices land.
#[test]
fn the_tab_strip_shows_abilities_selected_and_three_disabled() {
    let path = fixture(PesVersion::Pes21, "tab_strip");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let tab = |label: &str| {
        nodes
            .iter()
            .find(|node| node.role() == Role::Button && node.label() == Some(label))
            .unwrap_or_else(|| panic!("no tab {label:?}"))
    };
    let selected = tab("Abilities & Skills");
    assert_eq!(
        selected.toggled(),
        Some(accesskit::Toggled::True),
        "Abilities & Skills is the drawn tab"
    );
    assert!(!selected.is_disabled());
    for label in ["Appearance", "Team", "Tactics"] {
        let tab = tab(label);
        assert!(tab.is_disabled(), "{label} is disabled");
        assert_eq!(tab.toggled(), Some(accesskit::Toggled::False));
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
// four fields shown one above their stored value): a stored 3 shows 4;
// typing 8 stores 7.
#[test]
fn the_shifted_fields_show_one_above_stored_and_store_one_below() {
    let path = fixture(PesVersion::Pes21, "shifted_fields");
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
            .stats
            .form,
        3,
        "premise: form is stored 3"
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
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .stats
            .form,
        7,
        "typed 8 stores 7"
    );
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
    /// The badge texts a frame drew (`"Gold 6"`), in left-to-right order.
    fn badges(nodes: &[accesskit::Node]) -> Vec<String> {
        let mut badges: Vec<&accesskit::Node> = nodes
            .iter()
            .filter(|node| {
                node.role() == Role::Label
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
    /// session's file, team 701, slot 0.
    fn expected(view: &SaveView, rules: &aatf::Rules) -> Vec<String> {
        let session = view.session.as_ref().unwrap();
        let team = aatf::TeamView::new(session.file(), 701).unwrap();
        let player = team.player(0).unwrap();
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
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let rules = aatf::Rules::official();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(badges(&nodes), expected(&view, &rules), "the tiers' badges");

    // "Heading" (skill 6) is unticked and counts paid on every tier: the
    // tick lowers each badge by one.
    let left_of = |view: &SaveView| -> Vec<i32> {
        expected(view, &rules)
            .iter()
            .map(|badge| badge.rsplit(' ').next().unwrap().parse().unwrap())
            .collect()
    };
    let before = left_of(&view);
    assert_eq!(before, [6, 5, 4, 2, 3], "the unticked player's badges");
    let heading = nodes
        .iter()
        .find(|node| node.role() == Role::CheckBox && node.label() == Some("Heading"))
        .expect("the Heading checkbox");
    click(&mut window, heading, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(badges(&nodes), expected(&view, &rules), "after the tick");
    assert_eq!(
        left_of(&view),
        before.iter().map(|left| left - 1).collect::<Vec<i32>>(),
        "every badge one lower"
    );

    // Ctrl+Z unticks it: the badges are recomputed, back to the old counts.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(badges(&nodes), expected(&view, &rules), "after the undo");
    assert_eq!(left_of(&view), before, "every badge restored");

    // A rules file's tiers are the badges': the same view now given the
    // DBC7 file reloads on the changed path (no fresh window).
    let aatf = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let data = std::fs::read_to_string(aatf.join("rulesets/dbc7.rhai")).unwrap();
    let logic = std::fs::read_to_string(aatf.join("logic/logic.rhai")).unwrap();
    let rules_path = fixture(PesVersion::Pes21, "cards_left").with_file_name("dbc7.rhai");
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
    assert_eq!(badges(&nodes), expected(&view, &dbc7), "DBC7's tiers");
    assert!(
        badges(&nodes)
            .iter()
            .all(|badge| !badge.starts_with("Bronze")),
        "no Bronze: {:?}",
        badges(&nodes)
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
        nodes.iter().any(|node| {
            node.role() == Role::Label
                && node
                    .value()
                    .is_some_and(|value| value.starts_with("The AATF rules file does not load:"))
        }),
        "the load error's row"
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
