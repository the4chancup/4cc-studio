use super::*;

// plans/save_editor.md "Editing UI": every widget id under the strip and
// the tab is the selected player's, so a text field's undo history and a
// number control's in-progress edit never cross players.
#[test]
fn a_name_edit_does_not_reach_the_next_players_field() {
    let path = fixture(PesVersion::Pes17, "cross_player_name");
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(702);
    let mut window = Headless::sized(1100.0, 700.0);
    let name_of = |view: &SaveView, id: u32| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(id)
            .unwrap()
            .name
            .clone()
    };

    // Select 70201 (roster slot 0) and type an edit into the Name field.
    let file = EditFile::load(&path).unwrap();
    let a_id = file.team(702).unwrap().roster[0].player_id;
    let b_id = file.team(702).unwrap().roster[1].player_id;
    view.selected = Some(Selection::Player(702, 0));
    let a_before = name_of(&view, a_id);
    let b_before = name_of(&view, b_id);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some(a_before.as_str())
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    window.frame(vec![egui::Event::Text("ZZZ".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_ne!(name_of(&view, a_id), a_before, "the edit landed on 70201");

    // Select 70202 (roster slot 1), focus his Name field, Ctrl+Z: his
    // field has only his own edit history — never 70201's text.
    view.selected = Some(Selection::Player(702, 1));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some(b_before.as_str())
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    // The field's own undo answers Ctrl+Z (the `command` bit is what its
    // `matches_logically(COMMAND)` arm tests, as a real key event carries
    // it): 70202's field has none of 70201's history to restore.
    window.frame(
        vec![key(
            egui::Key::Z,
            egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
            true,
        )],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(name_of(&view, b_id), b_before, "70202's name is unchanged");
}

// The same rule across opens: egui keeps a TextEdit's undo history by
// widget id, so the pushed id carries the open's generation — a save
// opened next starts the same player id's Name field with no history,
// and Ctrl+Z past the field's own edit restores none of the last save's
// text. (egui's undoer only commits a change to an undo point once the
// text has sat for a second, so each typed stretch waits out its 60-odd
// headless frames.)
#[test]
fn a_second_saves_name_field_carries_none_of_the_first_saves_undo_history() {
    let path_a = fixture(PesVersion::Pes21, "history_a");
    let path_b = fixture(PesVersion::Pes21, "history_b");
    let mut view = SaveView::new();
    let mut window = Headless::sized(1100.0, 700.0);
    let name_of = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .name
            .clone()
    };
    /// The Name field's node: the enabled text edit right of its label's
    /// cell, holding `value` (the Shirt field carries the same text on
    /// this fixture, so position, not value alone, names it).
    fn name_field<'a>(nodes: &'a [accesskit::Node], value: &str) -> &'a accesskit::Node {
        let field = right_of_label(nodes, "Name", Role::TextInput)
            .into_iter()
            .find(|node| !node.is_disabled())
            .unwrap_or_else(|| panic!("no Name field {value:?}"));
        assert_eq!(field.value(), Some(value), "the Name field holds {value:?}");
        field
    }
    let settle = |window: &mut Headless, view: &mut SaveView| {
        for _ in 0..70 {
            window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
        }
    };

    // Save A: type into the selected player's (70101's) Name field, twice,
    // letting the field's undoer commit each stretch as an undo point.
    view.open(&path_a);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, name_field(&nodes, "PLACEHOLDER"), &mut view);
    window.frame(vec![egui::Event::Text("Z".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    settle(&mut window, &mut view);
    window.frame(vec![egui::Event::Text("Y".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    settle(&mut window, &mut view);
    let a_name = name_of(&view);
    assert_ne!(a_name, "PLACEHOLDER", "premise: save A's edit landed");

    // Save B (a fresh copy): one letter into the same player's Name field,
    // then two Ctrl+Z with the focus still on it — the first undoes the
    // letter, the second has nothing of this save's left to undo and must
    // not reach into save A's history.
    view.open(&path_b);
    let stored = name_of(&view);
    assert_eq!(stored, "PLACEHOLDER", "premise: save B opens unedited");
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(&mut window, name_field(&nodes, "PLACEHOLDER"), &mut view);
    window.frame(vec![egui::Event::Text("X".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    // A field's own undo answers Ctrl+Z: the event's `command` bit is what
    // its `matches_logically(COMMAND)` arm tests (a real platform event
    // carries it together with `ctrl`; `Modifiers::CTRL` alone is not).
    for _ in 0..2 {
        window.frame(
            vec![key(
                egui::Key::Z,
                egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
                true,
            )],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
    }
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        name_of(&view),
        stored,
        "the field's undo stops at save B's own edit: {a_name} never lands",
    );
}

// The strip's Name field draws 190 wide and the Shirt field 110 (the
// draft's): a fresh grid gives a non-last column only `min_cell_size` (40)
// on its sizing pass, and a TextEdit caps its width at what the cell
// offers, so the fields' sizes are allocated rather than left to the cell.
#[test]
fn the_name_and_shirt_fields_draw_at_their_set_widths() {
    let path = fixture(PesVersion::Pes21, "field_widths");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    for _ in 0..3 {
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    }
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));

    let name = right_of_label(&nodes, "Name", Role::TextInput)
        .first()
        .copied()
        .expect("the Name field")
        .bounds()
        .expect("bounds");
    assert!(
        name.x1 - name.x0 >= 190.0,
        "the Name field draws 190 wide, drew {name:?}"
    );
    let shirt = shirt_field(&nodes).bounds().expect("bounds");
    assert!(
        shirt.x1 - shirt.x0 >= 110.0,
        "the Shirt field draws 110 wide, drew {shirt:?}"
    );
}

// plans/save_editor.md "Editing UI", the info strip bullet's last
// sub-bullet: a name or shirt name edit sets the player's edited-player
// flag, in the same undo entry.
#[test]
fn a_name_or_shirt_edit_sets_the_edited_player_flag() {
    let path = fixture(PesVersion::Pes21, "edit_flag");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().edit_flags.player = false;
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let flag = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .edit_flags
            .player
    };
    assert!(!flag(&view), "premise: the flag is clear");

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some("PLACEHOLDER")
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    window.frame(vec![egui::Event::Text("X".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(flag(&view), "the edit set the flag, one entry");
    window.frame(
        vec![
            key(egui::Key::Escape, egui::Modifiers::NONE, true),
            key(egui::Key::Escape, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(!flag(&view), "undo clears it again");

    // The shirt field sets the same flag.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = shirt_field(&nodes);
    click(&mut window, shirt, &mut view);
    window.frame(vec![egui::Event::Text("Z".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(flag(&view), "a shirt edit sets the flag too");
}

// The same sub-bullet: the shirt name is upper-cased as typed.
#[test]
fn a_shirt_name_is_upper_cased_as_typed() {
    let path = fixture(PesVersion::Pes21, "uppercase_shirt");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = shirt_field(&nodes);
    let stored = shirt.value().expect("its text").to_owned();
    click(&mut window, shirt, &mut view);
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            key(egui::Key::End, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(vec![egui::Event::Text("miyako".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .shirt_name,
        format!("{stored}miyako").to_uppercase(),
        "miyako stores as MIYAKO"
    );
}

// spec/pes_savefile/operations.md "Team TOML" (the portable text limits)
// and plans/save_editor.md "Editing UI" (the info strip bullet): a stored
// shirt name past 18 letters is drawn tinted with the versions' hover; an
// edit may shorten it a keystroke at a time, never lengthen.
#[test]
fn a_stored_shirt_past_18_is_tinted_and_shortens_never_lengthens() {
    // 70101 with a 20-letter shirt name, over the portable 18 inside PES
    // 21's own field.
    let path = fixture(PesVersion::Pes21, "long_shirt");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().shirt_name = "S".repeat(20);
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let visuals = window.ctx.style_of(window.ctx.theme()).visuals.clone();
    let tint = visuals
        .extreme_bg_color
        .lerp_to_gamma(visuals.warn_fg_color, 0.25);

    // The field draws the tint and shows the versions' hover.
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = shirt_field(&nodes);
    assert_eq!(
        field.value(),
        Some("S".repeat(20).as_str()),
        "the Shirt field holds the 20 letters"
    );
    let bounds = field.bounds().expect("bounds");
    assert_eq!(
        fill_at(&shapes, bounds),
        Some(tint),
        "the shirt field is tinted"
    );
    let tooltip = tooltip_after_hover(
        &mut window,
        &mut view,
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        ),
    );
    assert_eq!(
        tooltip,
        Some("PES 2019 and older show 18 letters of a shirt name (PES 2016 shows 16)".to_string()),
        "the shirt hover"
    );

    // Two backspaces: 18 lands (each step shorter than stored).
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some("S".repeat(20).as_str())
        })
        .expect("the Shirt field");
    click(&mut window, shirt, &mut view);
    for _ in 0..2 {
        window.frame(
            vec![
                key(egui::Key::Backspace, egui::Modifiers::NONE, true),
                key(egui::Key::Backspace, egui::Modifiers::NONE, false),
            ],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
    }
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .shirt_name
            .len(),
        18,
        "the shortening landed"
    );

    // And lengthening past the current 18 is refused: two more letters do
    // not land.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some("S".repeat(18).as_str())
        })
        .expect("the Shirt field");
    click(&mut window, shirt, &mut view);
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            key(egui::Key::End, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    // The keystrokes land before the refusal: the 20 letters draw once.
    let (_, nodes, _) = window.frame(vec![egui::Event::Text("SS".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::TextInput && node.value() == Some("S".repeat(20).as_str())
        }),
        "the typed letters were drawn"
    );
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .shirt_name
            .len(),
        18,
        "20 chars over the current stored are refused"
    );
}

// The same bullet on PES 16: the field's own 16 letters bind, so a 17th
// is refused.
#[test]
fn a_pes16_strip_refuses_a_17th_shirt_letter() {
    let path = fixture(PesVersion::Pes16, "shirt_17");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = shirt_field(&nodes);
    let stored = view
        .session
        .as_ref()
        .unwrap()
        .file()
        .player(70101)
        .unwrap()
        .shirt_name
        .clone();
    click(&mut window, shirt, &mut view);
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            key(egui::Key::End, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    // The keystrokes land before the refusal: the field draws the typed
    // text on the Text frame, then reverts.
    let expected = format!("{stored}{}", "S".repeat(17));
    let (_, nodes, _) = window.frame(vec![egui::Event::Text("S".repeat(17).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::TextInput && node.value() == Some(expected.as_str())
        }),
        "the typed 17 letters were drawn"
    );
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "a 17th letter is refused on PES 16"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::TextInput && node.value() == Some(stored.as_str())
        }),
        "the field reverted"
    );
}

// plans/save_editor.md "Editing UI" with operations.md's portable limits:
// the Shirt field's hover says the limit the field enforces — the smaller
// of the portable 18 and the version's own field — so PES 21's 61-wide
// field still says 18.
#[test]
fn the_shirt_hover_names_the_limit_the_field_enforces() {
    let path = fixture(PesVersion::Pes21, "shirt_hover_limit");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = shirt_field(&nodes);
    let tooltip = tooltip_containing(&mut window, &mut view, centre(shirt), "Up to");
    assert_eq!(
        tooltip,
        Some(
            "Up to 18 letters. Western European accents (é, ü, ñ) are fine, but other letters \
                 and symbols are not."
                .to_string()
        ),
        "PES 21's shirt field is 61 wide but the editor holds 18"
    );
}

// The same bullet on PES 16: its own 16-letter field binds under the
// portable 18, and the hover says so.
#[test]
fn the_pes16_shirt_hover_names_its_own_field() {
    let path = fixture(PesVersion::Pes16, "shirt_hover_16");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = shirt_field(&nodes);
    let tooltip = tooltip_containing(&mut window, &mut view, centre(shirt), "Up to");
    assert_eq!(
        tooltip,
        Some(
            "Up to 16 letters. Western European accents (é, ü, ñ) are fine, but other letters \
                 and symbols are not."
                .to_string()
        ),
        "PES 16's own 16-letter field binds under the portable 18"
    );
}

// The boundary the hover names: the field's charset is one byte per
// letter, so é (U+00E9, inside) lands and ł (U+0142, past it) is refused.
#[test]
fn a_shirt_name_takes_latin1_letters_and_refuses_past_them() {
    /// The Shirt field's node (`shirt_field`; the name and shirt hold the
    /// same text on this fixture, so the label's cell, not the value,
    /// names it).
    fn shirt(nodes: &[accesskit::Node]) -> &accesskit::Node {
        shirt_field(nodes)
    }

    let path = fixture(PesVersion::Pes21, "shirt_latin1");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let shirt_name = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .shirt_name
            .clone()
    };

    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let stored = shirt(&nodes).value().expect("its text").to_owned();
    click(&mut window, shirt(&nodes), &mut view);
    let type_at_end = |window: &mut Headless, view: &mut SaveView, text: &str| {
        window.frame(
            vec![
                key(egui::Key::End, egui::Modifiers::NONE, true),
                key(egui::Key::End, egui::Modifiers::NONE, false),
            ],
            |ui| show(ui, view, &|| Vec::new(), None),
        );
        let (_, nodes, _) = window.frame(vec![egui::Event::Text(text.to_owned())], |ui| {
            show(ui, view, &|| Vec::new(), None)
        });
        nodes
    };

    // ł is past U+00FF: the letter draws once, then the field reverts and
    // nothing is stored.
    let nodes = type_at_end(&mut window, &mut view, "Ł");
    let typed = format!("{stored}Ł");
    assert_eq!(
        shirt(&nodes).value(),
        Some(typed.as_str()),
        "the typed ł was drawn before the revert"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        shirt(&nodes).value(),
        Some(stored.as_str()),
        "the field reverted to the stored shirt"
    );
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "the refused letter was not pushed"
    );

    // é is inside it: the same edit lands.
    type_at_end(&mut window, &mut view, "É");
    assert_eq!(shirt_name(&view), format!("{stored}É"), "the é landed");
}

// The same bullet's boundary: a shirt name of exactly the portable 18
// letters is no warning — the tint is for what a version would cut, and
// nothing is.
#[test]
fn a_shirt_at_the_portable_limit_draws_no_tint() {
    let path = fixture(PesVersion::Pes21, "shirt_at_limit");
    let mut file = EditFile::from_bytes(&fixture_save(PesVersion::Pes21)).unwrap();
    file.player_mut(70101).unwrap().shirt_name = "S".repeat(18);
    file.save(&path).unwrap();

    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let visuals = window.ctx.style_of(window.ctx.theme()).visuals.clone();
    let tint = visuals
        .extreme_bg_color
        .lerp_to_gamma(visuals.warn_fg_color, 0.25);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = shirt_field(&nodes);
    assert_eq!(
        field.value(),
        Some("S".repeat(18).as_str()),
        "the Shirt field holds the 18 letters"
    );
    let bounds = field.bounds().expect("bounds");
    assert_ne!(
        fill_at(&shapes, bounds),
        Some(tint),
        "an 18-letter shirt draws no tint"
    );
}

// The Captain box ticks on the captain's roster slot and not on another's.
#[test]
fn the_captain_box_ticks_on_the_captains_slot() {
    let path = fixture(PesVersion::Pes21, "captain_box");
    let file = EditFile::load(&path).unwrap();
    let captain = file.team(701).unwrap().tactics.set_pieces.captain;
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    let mut window = Headless::new();

    // The Captain radio is the strip's unlabeled radio; the tab's skill
    // boxes carry their names. egui 0.37 leaves an empty label unset in the
    // AccessKit tree rather than writing `""`, so the node carries `None`.
    let toggled = |nodes: &[accesskit::Node]| {
        nodes
            .iter()
            .find(|node| node.role() == Role::RadioButton && node.label().is_none())
            .map(|node| node.toggled())
    };
    // Select the captain's slot: the radio is on.
    view.selected = Some(Selection::Player(701, captain));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(toggled(&nodes), Some(Some(accesskit::Toggled::True)));
    // Another slot: off.
    let other = if captain == 0 { 1 } else { 0 };
    view.selected = Some(Selection::Player(701, other));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(toggled(&nodes), Some(Some(accesskit::Toggled::False)));
}

// plans/save_editor.md "Editing UI", the info strip bullet's last
// sub-bullet: Captain is a radio that makes this slot the team's captain
// and is not unticked by a second click (the captain belongs to the slot).
#[test]
fn the_captains_radio_cannot_be_unticked() {
    let path = fixture(PesVersion::Pes21, "captain_radio");
    let file = EditFile::load(&path).unwrap();
    let captain = file.team(701).unwrap().tactics.set_pieces.captain;
    let mut view = SaveView::new();
    view.open(&path);
    view.team = TeamChoice::Team(701);
    view.selected = Some(Selection::Player(701, captain));
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let radio = nodes
        .iter()
        .find(|node| node.role() == Role::RadioButton && node.label().is_none())
        .expect("the Captain radio");

    // A second click on the captain's radio: still his slot, no undo entry.
    click(&mut window, radio, &mut view);
    let session = view.session.as_ref().unwrap();
    assert_eq!(
        session.file().team(701).unwrap().tactics.set_pieces.captain,
        captain,
        "the captain stays"
    );
    assert!(!session.can_undo(), "the second click pushed nothing");
}

// A typed value past a field's stored width is clamped to it, the codec's own
// bound; a value in range lands as typed.
#[test]
fn a_typed_number_clamps_to_the_fields_stored_width() {
    let path = fixture(PesVersion::Pes21, "clamped_number");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    // Age's maximum is the schema's own, read from the save's version, not a
    // literal.
    let schema = pes_savefile::schema::schema_for(PesVersion::Pes21);
    let age_max = schema
        .player
        .fields
        .iter()
        .find(|spec| spec.field == PlayerField::Age)
        .map(|spec| (1u64 << spec.bit_width) - 1)
        .expect("the schema stores Age");

    // Type 99 into Age (stored 20): the field clamps it.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let age = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("20"))
        .expect("the Age control");
    click(&mut window, age, &mut view);
    window.frame(vec![egui::Event::Text("99".to_owned())], |ui| {
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
            .basic
            .age,
        age_max as u8,
        "99 clamps to the field's width"
    );

    // Type 23 into Number (in range): it lands as typed.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let number = right_of_label(&nodes, "Number", Role::SpinButton)
        .first()
        .copied()
        .expect("the Number control");
    click(&mut window, number, &mut view);
    window.frame(vec![egui::Event::Text("23".to_owned())], |ui| {
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
            .team(701)
            .unwrap()
            .roster[0]
            .number,
        23
    );
}

// plans/save_editor.md "Editing UI", the info strip bullet's Name
// sub-bullet: the save's own field bounds an edit (a PES 21 save takes 47
// to 61 bytes, a name past 46 tinted with the cut the older versions make).
#[test]
fn a_pes21_strip_takes_a_long_name_tinted_with_its_crop() {
    /// The Name field's `TextInput` node (the enabled text edit right of
    /// the "Name" label's cell).
    fn name_field(nodes: &[accesskit::Node], value: &str) -> accesskit::Rect {
        right_of_label(nodes, "Name", Role::TextInput)
            .into_iter()
            .find(|node| !node.is_disabled() && node.value() == Some(value))
            .unwrap_or_else(|| panic!("no Name field {value:?}"))
            .bounds()
            .expect("bounds")
    }

    let path = fixture(PesVersion::Pes21, "long_name_taken");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let visuals = window.ctx.style_of(window.ctx.theme()).visuals.clone();
    let tint = |v: &egui::style::Visuals| v.extreme_bg_color.lerp_to_gamma(v.warn_fg_color, 0.25);

    // 70101's stored name fits: no tint, no crop hover.
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let bounds = name_field(&nodes, "PLACEHOLDER");
    assert_ne!(
        fill_at(&shapes, bounds),
        Some(tint(&visuals)),
        "a fitting name draws no tint"
    );

    // Type a 52-byte name: taken (inside PES 21's 61), as one undo entry,
    // the field tinted with the 46-byte crop on hover.
    let name_bounds = name_field(&nodes, "PLACEHOLDER");
    window.frame(
        pointer_at(
            egui::pos2(
                ((name_bounds.x0 + name_bounds.x1) / 2.0) as f32,
                ((name_bounds.y0 + name_bounds.y1) / 2.0) as f32,
            ),
            true,
        ),
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(
        pointer_at(
            egui::pos2(
                ((name_bounds.x0 + name_bounds.x1) / 2.0) as f32,
                ((name_bounds.y0 + name_bounds.y1) / 2.0) as f32,
            ),
            false,
        ),
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            key(egui::Key::End, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(vec![egui::Event::Text("Z".repeat(41).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .name,
        format!("PLACEHOLDER{}", "Z".repeat(41)),
        "the 52-byte name was taken"
    );
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let bounds = name_field(&nodes, &format!("PLACEHOLDER{}", "Z".repeat(41)));
    assert_eq!(
        fill_at(&shapes, bounds),
        Some(tint(&visuals)),
        "the overlong name is tinted"
    );
    // The hover shows the cut: «46 N's». Hover the tinted field.
    let tooltip = tooltip_after_hover(
        &mut window,
        &mut view,
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        ),
    );
    assert_eq!(
        tooltip,
        Some(format!(
            "In PES 2019 and older the game cuts this name to «{}{}»",
            "PLACEHOLDER",
            "Z".repeat(35)
        )),
        "the crop hover"
    );
    // Escape defocuses the field before Ctrl+Z (a focused text field's own
    // undo answers first).
    window.frame(
        vec![
            key(egui::Key::Escape, egui::Modifiers::NONE, true),
            key(egui::Key::Escape, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(
        undo_entries(&mut window, &mut view),
        1,
        "the name edit is one entry"
    );

    // Exactly the portable 46 bytes ("PLACEHOLDER" + 35): no tint, no crop
    // hover — the tint is a warning for what would be cut, and nothing is.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let bounds = name_field(&nodes, "PLACEHOLDER");
    let pos = egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    );
    window.frame(pointer_at(pos, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(pointer_at(pos, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            key(egui::Key::End, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(vec![egui::Event::Text("Z".repeat(35).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let name46 = format!("PLACEHOLDER{}", "Z".repeat(35));
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let bounds = name_field(&nodes, &name46);
    assert_ne!(
        fill_at(&shapes, bounds),
        Some(tint(&visuals)),
        "a name at the portable limit draws no tint"
    );
    assert_eq!(
        tooltip_after_hover(
            &mut window,
            &mut view,
            egui::pos2(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        ),
        None,
        "a name at the portable limit has no crop hover"
    );
}

// The same rule on PES 17: its own field holds 46 bytes, so a 47-byte name
// is refused (the field reverts and nothing is pushed).
#[test]
fn a_pes17_strip_refuses_a_name_past_its_own_field() {
    let path = fixture(PesVersion::Pes17, "overlong_name_17");
    let mut view = SaveView::new();
    view.open(&path);
    view.selected = Some(Selection::Player(701, 0));
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let file = EditFile::load(&path).unwrap();
    // The field draws the raw name with `\x11` spelled out (the strip's
    // `escName`): match it as drawn.
    let stored = file
        .player(file.team(701).unwrap().roster[0].player_id)
        .unwrap()
        .name
        .replace('\x11', "\\x11");
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some(stored.as_str())
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    window.frame(
        vec![
            key(egui::Key::End, egui::Modifiers::NONE, true),
            key(egui::Key::End, egui::Modifiers::NONE, false),
        ],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    // 47 bytes past this version's own 46-byte field: the keystrokes land
    // and draw before the refusal reverts them.
    let expected = format!("{stored}{}", "Z".repeat(47));
    let (_, nodes, _) = window.frame(vec![egui::Event::Text("Z".repeat(47).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::TextInput && node.value() == Some(expected.as_str())
        }),
        "the typed 47 bytes were drawn"
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::TextInput && node.value() == Some(stored.as_str())
        }),
        "the field reverted to the stored name"
    );
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "the refused edit was not pushed"
    );
}

/// Whether `shapes` paints `text` at the heading size (the header's
/// title; the tabs' strong lines and the lists' rows are body-sized).
fn title_painted(shapes: &[egui::epaint::ClippedShape], text: &str, heading: f32) -> bool {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == text => Some(shape),
            _ => None,
        })
        .any(|shape| {
            shape
                .galley
                .job
                .sections
                .first()
                .is_some_and(|section| (section.format.font_id.size - heading).abs() < 0.5)
        })
}

/// The painted `TextShape` whose galley text is `text` (the name's runs
/// merged into one).
fn title_shape<'a>(
    shapes: &'a [egui::epaint::ClippedShape],
    text: &str,
) -> &'a egui::epaint::TextShape {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == text => Some(shape),
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("no painted text {text:?}"))
}

// plans "Editing UI", the player header bullet and the "No view draws the
// tool's name" bullet: `<id> <name>` as the player view's title over the
// strip's nine labels — on the player view alone, the team view editing no
// player. The name's colour codes paint as the player list draws them: a
// coded run in the code's colour, the rest the widget's text colour, the id
// the weak one.
// SE-GUI-01
#[test]
fn the_player_header_tops_the_player_view_alone() {
    let (mut view, mut window) = open_734_with("player_header_tabs", &[], 0);
    // A name with a colour code and a reset (the `\x11c` + 8 hex digits the
    // player list renders, the `append_name` test's name): the title's runs
    // read the code's colour then the default one.
    let mut entry = player_entry(&view, 73401);
    entry.name = "WHEN THE \x11cb7bec5ffWORK RESULT\x11d WAS GOOD".to_owned();
    view.session.as_mut().unwrap().edit_players(vec![entry]);
    let title = "73401 WHEN THE WORK RESULT WAS GOOD";
    let normal = egui::Color32::PLACEHOLDER;
    let weak = window
        .ctx
        .style_of(window.ctx.theme())
        .visuals
        .weak_text_color();
    let expected: Vec<(egui::Color32, &str)> = vec![
        (weak, "73401 "),
        (normal, "WHEN THE "),
        (egui::Color32::from_rgb(0xb7, 0xbe, 0xc5), "WORK RESULT"),
        (normal, " WAS GOOD"),
    ];
    let heading =
        window.ctx.style_of(window.ctx.theme()).text_styles[&egui::TextStyle::Heading].size;
    // The player view, with the Appearance section closed then open: the
    // header tops both arrangements.
    for appearance_open in [false, true] {
        if appearance_open {
            open_appearance(&mut window, &mut view);
        }
        let (_, nodes, shapes) =
            window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
        let shape = title_shape(&shapes, title);
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
        assert_eq!(
            painted,
            expected
                .iter()
                .map(|(color, text)| (*color, text.to_string()))
                .collect::<Vec<_>>(),
            "the title's painted runs (appearance open: {appearance_open})"
        );
        assert!(
            title_painted(&shapes, title, heading),
            "the name at the heading size"
        );
        for label in [
            "Name", "Shirt", "Number", "Captain", "ID", "Nation", "Age", "Height", "Weight",
        ] {
            assert!(label_value(&nodes, label), "{label} drawn");
        }
    }
    // The team view draws neither the title nor the player's fields.
    open_team_view(&mut window, &mut view, 734);
    let (_, nodes, shapes) =
        window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !title_painted(&shapes, title, heading),
        "no title on the team view"
    );
    for label in [
        "Shirt", "Captain", "ID", "Nation", "Age", "Height", "Weight",
    ] {
        assert!(!label_value(&nodes, label), "{label} not on the team view");
    }
}
