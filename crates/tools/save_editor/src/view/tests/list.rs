use super::*;

// SE-GUI-01
#[test]
fn an_open_save_lists_every_rostered_player_and_shows_the_first() {
    let path = fixture(PesVersion::Pes21, "open_lists");
    let file = EditFile::load(&path).unwrap();
    let rostered: usize = file
        .teams()
        .iter()
        .map(|team| {
            team.roster
                .iter()
                .filter(|slot| slot.player_id != 0)
                .count()
        })
        .sum();
    let mut view = SaveView::new();
    view.open(&path);
    // Tall enough for a team's row plus all of 701's 23 player rows (the
    // team row eats one row's height).
    let mut window = Headless::sized(800.0, 1100.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // The team selector is the left panel's combo (the tab holds combos of
    // its own now).
    let selector = nodes
        .iter()
        .filter(|node| node.role() == Role::ComboBox)
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().map(|r| r.x0).unwrap_or(f64::MAX);
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("the team selector");
    assert_eq!(selector.value(), Some("ALL"));

    // Open it: ALL first, then the save's teams in order.
    click(&mut window, selector, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let mut expected = vec!["ALL".to_string()];
    expected.extend(file.teams().iter().map(|team| team.name.clone()));
    let mut items: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| expected.iter().any(|e| e == label))
        })
        .collect();
    items.sort_by(|a, b| {
        let y = |node: &&accesskit::Node| node.bounds().map(|r| r.y0).unwrap_or(0.0);
        y(a).partial_cmp(&y(b)).expect("ordered bounds")
    });
    let items: Vec<String> = items
        .iter()
        .filter_map(|node| node.label().map(str::to_owned))
        .collect();
    assert_eq!(items, expected, "the selector's entries");

    // Click 701's entry: the list holds only its players, headed by its
    // own row (their rows name the team "/3/").
    let (_, nodes, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (item, _) = nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("/3/"))
        .expect("team 701's entry");
    action_click(&mut window, &mut view, *item);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let rows: Vec<&accesskit::Node> = nodes.iter().filter(|node| player_row(node)).collect();
    assert_eq!(
        rows.len(),
        file.team(701)
            .unwrap()
            .roster
            .iter()
            .filter(|slot| slot.player_id != 0)
            .count(),
        "team 701's roster"
    );
    assert_eq!(view.team, TeamChoice::Team(701), "701 is the chosen team");

    // The list lays out only the visible rows (`show_rows`): a screenful is
    // drawn, and scrolling reaches every rostered player (the tail's test).
    view.team = TeamChoice::All;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let drawn = nodes.iter().filter(|node| player_row(node)).count();
    assert!(
        drawn >= 20 && drawn < rostered,
        "a full first page, not the whole save ({drawn} of {rostered})"
    );
    // The first row is team 701's first rostered player: number, position,
    // team name and display name.
    assert!(
        nodes
            .iter()
            .any(|node| node.label() == Some("1 CB /3/ PLACEHOLDER")),
        "the first row"
    );

    // The info strip shows the first row's general fields as the file holds
    // them: id 70101 (the title's first text), height 180, weight 80.
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Label
                && node.value().is_some_and(|value| value.starts_with("70101"))
        }),
        "the title's id"
    );
    for value in ["180", "80"] {
        assert!(
            nodes
                .iter()
                .any(|node| node.role() == Role::SpinButton && node.value() == Some(value)),
            "a {value} control"
        );
    }
}

// SE-GUI-01
#[test]
fn a_players_name_paints_in_the_colors_his_codes_set() {
    let path = fixture(PesVersion::Pes17, "colored_names");
    let file = EditFile::load(&path).unwrap();
    let player = file.player(71801).expect("player 71801");
    let spans = name_spans(&player.name);
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(718);
    let mut window = Headless::new();
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let shape = text_shape(&shapes, "50 GK ROADTRIP");
    let painted: Vec<(egui::Color32, String)> = shape
        .galley
        .job
        .sections
        .iter()
        .map(|section| {
            (
                section.format.color,
                shape.galley.job.text[section.byte_range.start.0..section.byte_range.end.0]
                    .to_owned(),
            )
        })
        .collect();
    // The painted sections' plain runs carry `Color32::PLACEHOLDER`, which
    // the widget replaces with its own text color when painting (the
    // visuals' `text_color` for an unselected row): it is what
    // `name_spans`' `None` means. The layout merges adjacent equal-color
    // sections, so the expectation merges too.
    let normal = egui::Color32::PLACEHOLDER;
    let mut expected: Vec<(egui::Color32, String)> = vec![(normal, "50 GK ".to_owned())];
    for (color, text) in &spans {
        let color = match color {
            Some([red, green, blue]) => egui::Color32::from_rgb(*red, *green, *blue),
            None => normal,
        };
        match expected.last_mut() {
            Some((last_color, last_text)) if *last_color == color => last_text.push_str(text),
            _ => expected.push((color, text.to_string())),
        }
    }
    assert_eq!(painted, expected, "the row's painted runs");
}

// SE-GUI-02
#[test]
fn the_filter_finds_a_player_of_another_team_and_selecting_shows_him() {
    let path = fixture(PesVersion::Pes17, "cross_team_filter");
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    let mut window = Headless::new();

    // Click the filter box (its hint is its AccessKit placeholder), then type
    // the name.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let filter_box = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && node.placeholder() == Some("Filter every team (name, shirt name or id)")
        })
        .expect("the filter box");
    click(&mut window, filter_box, &mut view);
    let (_, nodes, _) = window.frame(vec![egui::Event::Text("quality".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });

    // 70202 of team 702 is listed, his row naming his team under a filter.
    let row = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("1 GK /a/ QUALITY"))
        .expect("the filtered row");
    click(&mut window, row, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Label
                && node.value().is_some_and(|value| value.starts_with("70202"))
        }),
        "the title's id"
    );

    // His id and his shirt name find him too: "anime" is in his shirt name
    // alone.
    let entry = player_entry(&view, 70202);
    assert_eq!(
        (entry.name.as_str(), entry.shirt_name.as_str()),
        ("QUALITY", "QUALITY ANIME"),
        "premise: 70202's names"
    );
    for typed in ["70202", "anime"] {
        view.filter = typed.to_owned();
        let (_, nodes, _) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        assert!(
            nodes.iter().any(|node| node.role() == Role::Button
                && node.label() == Some("1 GK /a/ QUALITY")),
            "{typed:?} finds 70202"
        );
    }
}

// The selected row's AccessKit node reports toggled, the others not (egui
// maps a selectable's `selected` to `toggled`).
#[test]
fn the_selected_row_reports_selected() {
    let path = fixture(PesVersion::Pes21, "selected_row");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let rows: Vec<&accesskit::Node> = nodes.iter().filter(|node| player_row(node)).collect();
    let (selected, rest): (Vec<&&accesskit::Node>, Vec<&&accesskit::Node>) = rows
        .iter()
        .partition(|node| node.toggled() == Some(accesskit::Toggled::True));
    assert_eq!(selected.len(), 1, "exactly one row selected");
    assert_eq!(selected[0].label(), Some("1 CB /3/ PLACEHOLDER"));
    assert!(
        rest.iter()
            .all(|node| node.toggled() == Some(accesskit::Toggled::False))
    );
}

// plans/save_editor.md "Editing UI", the Player list bullet (a row one
// line, truncated, its full name on hover): a 52-byte name keeps every row
// at one line, and the scroll still reaches the last player.
#[test]
fn a_long_name_truncates_instead_of_wrapping_the_row() {
    let path = fixture(PesVersion::Pes21, "long_row");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().name = "N".repeat(52);
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    let mut window = Headless::new();
    let row_height = window
        .ctx
        .style_of(window.ctx.theme())
        .spacing
        .interact_size
        .y;
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let rows: Vec<&accesskit::Node> = nodes.iter().filter(|node| player_row(node)).collect();
    assert!(!rows.is_empty());
    for row in &rows {
        let bounds = row.bounds().expect("row bounds");
        assert_eq!(
            bounds.y1 - bounds.y0,
            f64::from(row_height),
            "every row is one line"
        );
    }

    // Drag the scroll bar to the bottom: the last rostered row still shows.
    let bar = nodes
        .iter()
        .filter(|node| node.role() == Role::ScrollBar)
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().map(|r| r.x0).unwrap_or(0.0);
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("the list's scroll bar");
    let track = centre(bar);
    window.frame(
        pointer_at(egui::pos2(track.x, track.y - 200.0), true),
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(
        vec![egui::Event::PointerMoved(egui::pos2(track.x, 595.0))],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(pointer_at(egui::pos2(track.x, 595.0), false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let file2 = EditFile::load(&path).unwrap();
    let (_, roster_slot) = file2
        .team(701)
        .unwrap()
        .roster
        .iter()
        .enumerate()
        .rev()
        .find(|(_, slot)| slot.player_id != 0)
        .expect("701 has a rostered slot");
    let player = file2
        .player(roster_slot.player_id)
        .expect("the last player");
    let label = format!(
        "{} {} {}",
        roster_slot.number,
        pes_savefile::interchange::team_toml::labels::POSITIONS
            [usize::from(player.positions.registered)],
        pes_savefile::model::names::display_name(&player.name)
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    row(&nodes, &label);
}

// The list's scroll reaches every rostered player (SE-GUI-01's list drawn
// through `show_rows`): the last row of ALL clicks like any other.
#[test]
fn scrolling_to_the_end_of_all_selects_the_last_player() {
    let path = fixture(PesVersion::Pes21, "scroll_to_end");
    let file = EditFile::load(&path).unwrap();
    // The last rostered slot of the last team, as the row labels it.
    let team = file.teams().last().expect("teams exist");
    let (_, roster_slot) = team
        .roster
        .iter()
        .enumerate()
        .rev()
        .find(|(_, slot)| slot.player_id != 0)
        .expect("the last team has a rostered slot");
    let player = file.player(roster_slot.player_id).expect("the last player");
    let label = format!(
        "{} {} {} {}",
        roster_slot.number,
        pes_savefile::interchange::team_toml::labels::POSITIONS
            [usize::from(player.positions.registered)],
        team.name,
        pes_savefile::model::names::display_name(&player.name)
    );

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    // Drag the scroll bar's handle to the bottom: a pointer press inside the
    // bar, a move to its last pixels, release. The list's bar is the
    // leftmost (the tab's content scrolls too).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let bar = nodes
        .iter()
        .filter(|node| node.role() == Role::ScrollBar)
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().map(|r| r.x0).unwrap_or(0.0);
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("the list's scroll bar");
    let track = centre(bar);
    window.frame(
        pointer_at(egui::pos2(track.x, track.y - 200.0), true),
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(
        vec![egui::Event::PointerMoved(egui::pos2(track.x, 595.0))],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(pointer_at(egui::pos2(track.x, 595.0), false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let last = row(&nodes, &label);
    click(&mut window, last, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let id = roster_slot.player_id.to_string();
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Label && node.value().is_some_and(|value| value.starts_with(&id))
        }),
        "the title's id"
    );
}

// SE-GUI-08
#[test]
fn a_row_dragged_above_another_trades_their_records_between_the_ids() {
    let path = fixture(PesVersion::Pes21, "drag_reorder");
    // Distinguish the two records the drag trades (the fixture's placeholders
    // are identical): a name, a stat, a boot id and a base copy each.
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().name = "FIRST".to_owned();
    file.player_mut(70101).unwrap().stats.speed = 70;
    file.player_mut(70101).unwrap().appearance.boots_id = 11;
    file.player_mut(70101).unwrap().appearance.base_copy_id = 9001;
    file.player_mut(70102).unwrap().name = "SECOND".to_owned();
    file.player_mut(70102).unwrap().stats.speed = 71;
    file.player_mut(70102).unwrap().appearance.boots_id = 12;
    file.player_mut(70102).unwrap().appearance.base_copy_id = 9002;
    file.save(&path).unwrap();
    let roster_before = file.team(701).unwrap().roster.clone();
    let first = file.player(70101).unwrap().clone();
    let second = file.player(70102).unwrap().clone();

    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // Select another row first, so the selection following the dragged
    // record is not vacuous.
    click(&mut window, row(&nodes, "3 CB PLACEHOLDER"), &mut view);
    assert_eq!(
        view.selected,
        Some(Selection::Player(701, 2)),
        "premise: row 03 selected"
    );

    // Drag player 02's row above player 01's.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let from = centre(row(&nodes, "2 CB SECOND"));
    let onto = centre(row(&nodes, "1 CB FIRST"));
    drag(&mut window, from, onto, &mut view);

    // The list shows the new order, the moved rows marked unsaved; the
    // selection followed the dragged record to its new slot.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let labels = row_labels(&nodes);
    assert_eq!(&labels[..2], ["1 CB SECOND ●", "2 CB FIRST ●"]);
    assert_eq!(
        view.selected,
        Some(Selection::Player(701, 0)),
        "the selection sits on the dropped record's slot"
    );

    // Save, reopen: slot 01 still holds player 70101, whose record is the
    // one player 02 had, and the other way round; the roster is untouched.
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let reopened = EditFile::load(&path).unwrap();
    let team = reopened.team(701).expect("team 701");
    assert_eq!(
        team.roster, roster_before,
        "the roster's ids and shirt numbers are as they were"
    );
    let held = reopened.player(70101).expect("player 70101");
    assert_eq!(record(held), record(&second), "slot 01's record is 70102's");
    assert_eq!(held.id, 70101);
    assert_eq!(held.appearance_id, Some(70101));
    assert_eq!(held.appearance.base_copy_id, 9001);
    let held = reopened.player(70102).expect("player 70102");
    assert_eq!(record(held), record(&first), "slot 02's record is 70101's");
    assert_eq!(held.id, 70102);
    assert_eq!(held.appearance_id, Some(70102));
    assert_eq!(held.appearance.base_copy_id, 9002);

    // The roster-order rule is unbroken.
    let view_team = aatf::TeamView::new(&reopened, 701).expect("the team views");
    let violations = aatf::Rules::official()
        .check_team(&view_team)
        .expect("the check runs");
    assert!(
        violations
            .iter()
            .all(|violation| !matches!(violation.kind, aatf::ViolationKind::RosterOrder { .. })),
        "no RosterOrder: {violations:?}"
    );
}

// SE-GUI-08 (the plan's Player list bullet: under ALL each team's rows run
// unbroken under its team row): a drop on a row of the dragged player's own
// team moves him as in one team's list; a drop on another team's row or on
// a team row does nothing.
#[test]
fn under_all_a_drag_moves_within_the_team_and_nowhere_else() {
    let path = fixture(PesVersion::Pes21, "drag_reorder_all");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().name = "FIRST".to_owned();
    file.player_mut(70102).unwrap().name = "SECOND".to_owned();
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    assert_eq!(view.team, TeamChoice::All, "open selects ALL");
    // Tall enough for team 702's block to be drawn under team 701's.
    let mut window = Headless::sized(800.0, 1100.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    // A drag onto another team's row: nothing moves.
    let before = row_labels(&nodes);
    let from = centre(row(&nodes, "2 CB /3/ SECOND"));
    let other = centre(row(&nodes, "1 CB /a/ PLACEHOLDER"));
    drag(&mut window, from, other, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        row_labels(&nodes),
        before,
        "a drop on another team's row moved nothing"
    );
    assert!(!view.session.as_ref().unwrap().is_dirty());

    // A drag onto the team row: nothing moves.
    let from = centre(row(&nodes, "2 CB /3/ SECOND"));
    let onto = centre(team_row(&nodes, &view, 701));
    drag(&mut window, from, onto, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        row_labels(&nodes),
        before,
        "a drop on the team row moved nothing"
    );
    assert!(!view.session.as_ref().unwrap().is_dirty());
    assert!(!view.session.as_ref().unwrap().can_undo());

    // A drag onto the player's own team's row: the records trade, as in
    // one team's list.
    let from = centre(row(&nodes, "2 CB /3/ SECOND"));
    let onto = centre(row(&nodes, "1 CB /3/ FIRST"));
    drag(&mut window, from, onto, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        &row_labels(&nodes)[..2],
        ["1 CB /3/ SECOND ●", "2 CB /3/ FIRST ●"],
        "the drop on his own team moved him"
    );
}
