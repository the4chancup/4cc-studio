use super::*;

// plans/save_editor.md "Editing UI", the Undo/redo and Unsaved changes
// bullets (the mechanism SE-GUI-03's scenario is built on): an edit marks the
// row while the file stands; Ctrl+S writes it and clears the mark.
#[test]
fn an_edit_marks_the_row_until_ctrl_s_writes_the_save() {
    let path = fixture(PesVersion::Pes21, "edit_then_save");
    let original = std::fs::read(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();

    // 70101's Height is 180: focus the control and type a new one.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let height = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(
        row(&nodes, "1 CB /3/ PLACEHOLDER ●").label(),
        Some("1 CB /3/ PLACEHOLDER ●")
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "the file is unchanged until Save"
    );

    // Ctrl+S writes it; the dot goes.
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    row(&nodes, "1 CB /3/ PLACEHOLDER");
    assert!(
        !nodes
            .iter()
            .any(|node| node.label().is_some_and(|label| label.contains('●'))),
        "no dirty player"
    );
    let reopened = Session::open(&path).unwrap();
    assert_eq!(
        reopened.file().player(70101).unwrap().basic.height,
        170,
        "the reopened file holds the height"
    );
}

// SE-GUI-04
#[test]
fn undo_and_redo_step_the_edits_back_and_forth() {
    let path = fixture(PesVersion::Pes21, "undo_redo");
    let original = std::fs::read(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let basics = |view: &SaveView| {
        let player = view.session.as_ref().unwrap().file().player(70101).unwrap();
        (player.basic.height, player.basic.weight)
    };

    // Height 180 to 170, then Weight 80 to 90: two gestures, two entries.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let height = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let weight = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("80"))
        .expect("the Weight control");
    click(&mut window, weight, &mut view);
    window.frame(vec![egui::Event::Text("90".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(basics(&view), (170, 90));

    // Ctrl+Z takes back the second edit only; Ctrl+Y redoes it.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        basics(&view),
        (170, 80),
        "the weight edit undone, height kept"
    );
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(basics(&view), (170, 90), "the weight edit redone");
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "the file changes only on Save"
    );
}

// plans/save_editor.md "Editing UI", the Undo/redo bullet: changes from one
// widget merge into one entry while the gesture lasts, so a drag is one undo
// step, not one per frame.
#[test]
fn a_drag_on_a_control_is_one_undo_entry() {
    let path = fixture(PesVersion::Pes21, "drag_one_entry");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let height = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .basic
            .height
    };

    // Drag the Height control over several frames.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let control = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    let at = centre(control);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    for step in 1..=5 {
        window.frame(
            vec![egui::Event::PointerMoved(egui::pos2(
                at.x + 3.0 * step as f32,
                at.y,
            ))],
            |ui| show(ui, &mut view, &|| Vec::new(), None),
        );
    }
    window.frame(pointer_at(egui::pos2(at.x + 15.0, at.y), false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let dragged = height(&view);
    assert_ne!(dragged, 180, "the drag changed the height");
    // One Ctrl+Z undoes the whole drag: the stack held one entry and the
    // height is back to before the drag.
    assert_eq!(undo_entries(&mut window, &mut view), 1, "one entry");
    assert_eq!(height(&view), 180, "the one undo restores it");
}

// plans/save_editor.md "Editing UI", the Undo/redo bullet: a second gesture on
// the same control, after the first ended, is its own undo entry.
#[test]
fn a_second_gesture_on_the_same_control_is_its_own_entry() {
    let path = fixture(PesVersion::Pes21, "two_gestures");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let at = centre(
        nodes
            .iter()
            .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
            .expect("the Height control"),
    );
    let drag = |window: &mut Headless, view: &mut SaveView, at: egui::Pos2| {
        window.frame(pointer_at(at, true), |ui| {
            show(ui, view, &|| Vec::new(), None)
        });
        window.frame(
            vec![egui::Event::PointerMoved(egui::pos2(at.x + 30.0, at.y))],
            |ui| show(ui, view, &|| Vec::new(), None),
        );
        window.frame(pointer_at(egui::pos2(at.x + 30.0, at.y), false), |ui| {
            show(ui, view, &|| Vec::new(), None)
        });
    };
    drag(&mut window, &mut view, at);
    drag(&mut window, &mut view, at);
    assert_eq!(undo_entries(&mut window, &mut view), 2, "two entries");
}

// SE-GUI-21 (the scenario is manual; this is its logic)
#[test]
fn save_asks_the_edit_mode_question_while_the_saves_own_pes_runs() {
    let path = fixture(PesVersion::Pes21, "save_question");
    let original = std::fs::read(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    // An edit, so the question leaves something behind.
    let mut edited = view
        .session
        .as_ref()
        .unwrap()
        .file()
        .player(70101)
        .unwrap()
        .clone();
    edited.basic.height = 170;
    view.session.as_mut().unwrap().edit_player(1, edited);
    let running_own = || {
        vec![RunningPes {
            version: PesVersion::Pes21,
            exe: None,
        }]
    };
    let mut window = Headless::new();

    // The window's Save opens the question; its Cancel writes nothing.
    assert!(!save(&mut view, &running_own), "not written yet");
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        pairs.iter().any(|(_, node)| {
            node.value()
                .is_some_and(|value| value.contains("PES 2021 is running"))
        }),
        "the question names the game"
    );
    click(
        &mut window,
        prompt::modal_button(&pairs, "PES 2021 is running", "Cancel"),
        &mut view,
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "Cancel writes nothing"
    );
    assert!(
        view.session.as_ref().unwrap().is_dirty(),
        "the cancelled save did not write"
    );
    assert!(
        view.edit_mode_question.is_none(),
        "Cancel closes the question"
    );

    // Its Save writes with the member's answer.
    assert!(!save(&mut view, &running_own));
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    click(
        &mut window,
        prompt::modal_button(&pairs, "PES 2021 is running", "Save"),
        &mut view,
    );
    assert!(view.edit_mode_question.is_none());
    assert!(!view.session.as_ref().unwrap().is_dirty(), "the save wrote");
    let reopened = Session::open(&path).unwrap();
    assert_eq!(
        reopened.file().player(70101).unwrap().basic.height,
        170,
        "the reopened file holds the edit"
    );

    // A game of another version writes with no question.
    let mut edited = player_entry(&view, 70101);
    edited.basic.age = 33;
    view.session.as_mut().unwrap().edit_player(5, edited);
    let running_other = || {
        vec![RunningPes {
            version: PesVersion::Pes19,
            exe: None,
        }]
    };
    assert!(save(&mut view, &running_other), "the write went through");
    assert!(view.edit_mode_question.is_none(), "no question asked");
}

// plans/save_editor.md "Editing UI", the Unsaved changes bullet: undoing back
// to the saved state clears the dot, since it is the difference from the file
// as opened or last saved, not the undo stack's length.
#[test]
fn undoing_back_to_the_saved_state_clears_the_dot() {
    let path = fixture(PesVersion::Pes21, "undo_clears_dot");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let height = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    row(&nodes, "1 CB /3/ PLACEHOLDER ●");
    // Undoing the only edit clears the dot.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    row(&nodes, "1 CB /3/ PLACEHOLDER");
    assert!(
        !nodes
            .iter()
            .any(|node| node.label().is_some_and(|label| label.contains('●'))),
        "the dot is gone"
    );
}

// plans/save_editor.md "Editing UI", the Undo/redo bullet: the more
// specific shortcut answers first — Ctrl+Shift+Z is Redo, not Undo.
#[test]
fn ctrl_shift_z_redoes_not_undoes() {
    let path = fixture(PesVersion::Pes21, "shift_z_redo");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let height = |view: &SaveView| {
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .basic
            .height
    };
    // One edit: Height 180 to 170.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let height_control = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height_control, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(height(&view), 170);

    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(height(&view), 180, "Ctrl+Z undoes");
    window.frame(
        vec![key(
            egui::Key::Z,
            egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
            true,
        )],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert_eq!(height(&view), 170, "Ctrl+Shift+Z redoes");
}

// plans/save_editor.md "Editing UI" (the Keyboard shortcuts bullet):
// Ctrl+Z while a text field holds the keyboard undoes nothing — the
// field's own undo is egui's.
#[test]
fn ctrl_z_while_a_text_field_holds_the_keyboard_undoes_nothing() {
    let path = fixture(PesVersion::Pes21, "keys_typing");
    let mut view = SaveView::new();
    view.open(&path);
    let mut edited = player_entry(&view, 70101);
    edited.basic.height = 170;
    view.session.as_mut().unwrap().edit_player(1, edited);
    let revision = view.session.as_ref().unwrap().revision();
    let mut window = Headless::new();

    // Focus the Name field, then Ctrl+Z.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let field = name_field(&nodes);
    click(&mut window, field, &mut view);
    let mut wants = false;
    window.frame(Vec::new(), |ui| {
        wants = ui.ctx().egui_wants_keyboard_input();
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(wants, "premise: the field holds the keyboard");
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(
        view.session.as_ref().unwrap().revision(),
        revision,
        "the last edit is not undone"
    );
    assert_eq!(
        view.session
            .as_ref()
            .unwrap()
            .file()
            .player(70101)
            .unwrap()
            .basic
            .height,
        170
    );
}

// `Tool::view` draws the window through the tool impl (a settings error
// counts as an empty `aatf_rules` setting).
#[test]
fn the_tool_view_draws_the_window() {
    use studio_core::StudioTool;

    let dir = crate::test_support::scratch("tool_view_window");
    let (ctx, _events_rx, _requests_rx) = tool_context(&dir, studio_core::Settings::default());
    let mut tool = crate::Tool::new();
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No save is open.")),
        "the window drew"
    );
}

// The refusal dialog closes on Escape as on OK (`view::refusal_dialog`).
#[test]
fn the_refusal_dialog_dismisses_on_escape() {
    let good = fixture(PesVersion::Pes21, "escape_refusal");
    let mut view = SaveView::new();
    view.open(&good);
    let bad = good.with_file_name("not_a_save.bin");
    std::fs::write(&bad, b"this is no save").unwrap();
    view.open(&bad);
    assert!(view.refusal.is_some());
    let mut window = Headless::new();
    // One frame for the modal to register as the top one, then Escape.
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    window.frame(
        vec![key(egui::Key::Escape, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new(), None),
    );
    assert!(view.refusal.is_none(), "Escape closes it");
}

// plans/save_editor.md "Editing UI", the open path's refusal: a file that is
// no save shows the `Session::open` error and the open save is untouched.
#[test]
fn an_unreadable_file_is_refused_and_keeps_the_open_save() {
    let good = fixture(PesVersion::Pes21, "refused_open");
    let mut view = SaveView::new();
    view.open(&good);
    let bad = good.with_file_name("not_a_save.bin");
    std::fs::write(&bad, b"this is no save").unwrap();
    view.open(&bad);

    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let message = format!("{}: cannot open the save", bad.display());
    assert!(
        nodes
            .iter()
            .any(|node| node.value().is_some_and(|value| value.contains(&message))),
        "the refusal names the file: {message}"
    );
    // The open save is untouched: its rows still list.
    assert!(
        nodes
            .iter()
            .any(|node| node.label() == Some("1 CB /3/ PLACEHOLDER")),
        "the session still shows its players"
    );
}
