//! The transplant panel's tests (SE-GUI-14; `plans/save_editor.md`
//! "Aesthetics transplant"): the donor pick and its refusals, the
//! selection's preview rows, the lock on Apply, and the apply's one
//! undoable edit.

use std::path::PathBuf;

use super::*;

use crate::test_support::TEST_SALT;

/// A PES 17 fixture with `edit` applied, written under `dir` as `name`:
/// the donor save the dialog stub returns.
fn donor_save(dir: &Path, name: &str, edit: impl FnOnce(&mut EditFile)) -> PathBuf {
    let mut file =
        EditFile::from_bytes(&fixture_save(PesVersion::Pes17)).expect("the fixture loads");
    edit(&mut file);
    let path = dir.join(name);
    std::fs::write(&path, file.to_bytes(&TEST_SALT).expect("the save writes")).unwrap();
    path
}

/// The test's scratch folder: the one the open save was written to.
fn scratch_of(view: &SaveView) -> PathBuf {
    view.session
        .as_ref()
        .expect("an open save")
        .path()
        .parent()
        .expect("the scratch dir")
        .to_path_buf()
}

/// The transplant panel opened from the toolbar's Panels menu: the
/// "Aesthetics transplant…" item clicked, the next frame's node pairs
/// returned.
fn open_transplant(
    window: &mut Headless,
    view: &mut SaveView,
) -> Vec<(accesskit::NodeId, accesskit::Node)> {
    let nodes = draw(window, view, None);
    click(window, node_labeled(&nodes, Role::Button, "Panels"), view);
    let nodes = draw(window, view, None);
    click(
        window,
        node_labeled(&nodes, Role::Button, "Aesthetics transplant\u{2026}"),
        view,
    );
    let (_, pairs, _) = window.frame_ids(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
    pairs
}

/// `path` stubbed as the Pick dialog's choice; the "Pick…" button clicked.
fn pick_donor(window: &mut Headless, view: &mut SaveView, path: PathBuf) {
    view.open_save_dialog = Box::new(move || Some(path.clone()));
    let nodes = draw(window, view, None);
    click(
        window,
        node_labeled(&nodes, Role::Button, "Pick\u{2026}"),
        view,
    );
}

/// Frames until the preview lands: `pending` empty and the result keyed to
/// the session's current revision, the donor session's and the selection
/// text (bounded, so a worker that never replies fails rather than hangs).
/// Returns the landing frame's node pairs.
fn previewed(
    window: &mut Headless,
    view: &mut SaveView,
) -> Vec<(accesskit::NodeId, accesskit::Node)> {
    frames_until(
        window,
        view,
        None,
        |view| {
            let session = view.session.as_ref().expect("an open save");
            let donor = view
                .transplant
                .donor
                .as_ref()
                .map(|(_, donor)| donor.revision());
            view.transplant.preview.pending.is_none()
                && view
                    .transplant
                    .preview
                    .result
                    .as_ref()
                    .is_some_and(|(key, _)| {
                        key.revision == session.revision()
                            && Some(key.donor) == donor
                            && key.selection == view.transplant.selection
                    })
        },
        "the preview never returned",
    )
}

/// The transplant window's nodes out of `pairs` (`frame_ids`' output).
fn transplant_nodes(pairs: &[(accesskit::NodeId, accesskit::Node)]) -> Vec<&accesskit::Node> {
    nodes_in_window(pairs, "Aesthetics transplant")
        .into_iter()
        .map(|(_, node)| node)
        .collect()
}

/// The `(NodeId, node)` pair of the Button labeled `label` inside the
/// transplant window.
fn transplant_button<'a>(
    pairs: &'a [(accesskit::NodeId, accesskit::Node)],
    label: &str,
) -> (accesskit::NodeId, &'a accesskit::Node) {
    nodes_in_window(pairs, "Aesthetics transplant")
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no window button {label:?}"))
}

/// The transplant window's Label values and Button labels, top to bottom.
fn transplant_lines(window: &mut Headless, view: &mut SaveView) -> Vec<String> {
    window_lines(window, view, None, "Aesthetics transplant")
}

/// Whether the window draws preview output: a head's or row's Button
/// (each a click target — the hint line and the labels are not) or the
/// count line's text.
fn preview_shown(nodes: &[&accesskit::Node]) -> bool {
    nodes.iter().any(|node| {
        (node.role() == Role::Button
            && node
                .label()
                .is_some_and(|label| label.starts_with("Player ")))
            || node
                .value()
                .is_some_and(|value| value.contains("changed field(s)"))
    })
}

// SE-GUI-14
#[test]
fn a_donor_of_another_version_is_refused_naming_both() {
    let (mut view, mut window) = open_734_with("transplant_version", &[], 0);
    let dir = scratch_of(&view);
    let donor = dir.join("donor21");
    std::fs::write(&donor, fixture_save(PesVersion::Pes21)).unwrap();
    view.edit_aesthetics = true;
    let pairs = open_transplant(&mut window, &mut view);
    assert!(
        pairs
            .iter()
            .any(|(_, node)| node.label() == Some("Aesthetics transplant")),
        "the window's title"
    );
    pick_donor(&mut window, &mut view, donor);
    let lines = transplant_lines(&mut window, &mut view);
    assert!(
        lines.iter().any(|line| line
            == "The base save is a PES 2017 save and the donor a PES 2021 one: both must be of one version"),
        "the refusal's literal text: {lines:?}"
    );
    assert!(
        lines.iter().any(|line| line == "(none)"),
        "the pick dropped: {lines:?}"
    );
    assert!(
        !lines.iter().any(|line| line == "Selection"),
        "no selection row: {lines:?}"
    );
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !transplant_nodes(&pairs)
            .iter()
            .any(|node| node.role() == Role::TextInput),
        "no selection field"
    );
}

// SE-GUI-14
#[test]
fn the_preview_lists_the_targets_head_and_rows_and_apply_waits_for_the_lock() {
    let (mut view, mut window) = open_734_with("transplant_preview", &[], 0);
    let dir = scratch_of(&view);
    let donor = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    pick_donor(&mut window, &mut view, donor);
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    let pairs = previewed(&mut window, &mut view);
    let nodes = transplant_nodes(&pairs);
    assert!(
        nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.starts_with("Team 701"))
        }),
        "701's heading (the open save's team order): {pairs:?}"
    );
    let (head, _) = transplant_button(&pairs, "Player 70103  3 PLACEHOLDER");
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Button && node.label() == Some("Boots ID 0 \u{2192} 130")
        }),
        "the boots row in the tabs' terms"
    );
    let rows = match &view.transplant.preview.result {
        Some((_, Ok(preview))) => preview
            .lines
            .iter()
            .filter(|line| matches!(line, crate::view::comparator::Line::Row { .. }))
            .count(),
        _ => panic!("the preview landed"),
    };
    assert!(rows > 1, "premise: several rows");
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some(format!("{rows} changed field(s)").as_str())),
        "the draft's count line over the rows"
    );

    // The head's click selects the player, as the comparator's rows do.
    view.team = TeamChoice::Team(718);
    view.selected = Some((718, 0));
    action_click(&mut window, &mut view, head);
    assert_eq!(view.selected, Some((701, 2)), "70103's team and slot");
    assert_eq!(view.team, TeamChoice::Team(701), "the selector moved");

    // Locked: Apply is disabled, its hover the ownership line.
    let (_, apply) = transplant_button(&pairs, "Apply");
    assert!(apply.is_disabled(), "Apply is locked");
    assert_eq!(
        tooltip_containing(&mut window, &mut view, centre(apply), "Owned by"),
        Some(
            "Owned by the team's export: edit its settings.toml and recompile, then apply the patch."
                .to_string()
        ),
        "Apply's hover is the ownership line"
    );
}

// SE-GUI-14
#[test]
fn apply_is_one_undoable_edit_written_only_on_save() {
    let (mut view, mut window) = open_734_with("transplant_apply", &[], 0);
    let dir = scratch_of(&view);
    let donor_path = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    view.edit_aesthetics = true;
    pick_donor(&mut window, &mut view, donor_path);
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    let pairs = previewed(&mut window, &mut view);
    let before = player_entry(&view, 70103);
    let donor_appearance = view
        .transplant
        .donor
        .as_ref()
        .expect("a donor picked")
        .1
        .file()
        .player(70205)
        .expect("70205 is a donor player")
        .appearance
        .clone();
    assert_ne!(before.appearance, donor_appearance, "premise: they differ");
    let save_path = view
        .session
        .as_ref()
        .expect("an open save")
        .path()
        .to_path_buf();
    let bytes_before = std::fs::read(&save_path).unwrap();

    let (_, apply) = transplant_button(&pairs, "Apply");
    assert!(!apply.is_disabled(), "Apply is unlocked");
    click(&mut window, apply, &mut view);

    // The look is the donor's, the session dirty, the file on disk
    // untouched until Save.
    assert_eq!(player_entry(&view, 70103).appearance, donor_appearance);
    assert!(view.session.as_ref().unwrap().is_dirty());
    assert_eq!(
        std::fs::read(&save_path).unwrap(),
        bytes_before,
        "only Save writes the file"
    );
    let lines = transplant_lines(&mut window, &mut view);
    assert!(
        lines
            .iter()
            .any(|line| line == "Applied; unsaved until Save."),
        "the applied line: {lines:?}"
    );

    // Saved, the line goes too (the undo stack survives a save).
    view.session.as_mut().unwrap().save(&[]).unwrap();
    let lines = transplant_lines(&mut window, &mut view);
    assert!(
        !lines
            .iter()
            .any(|line| line == "Applied; unsaved until Save."),
        "the save ends the applied line: {lines:?}"
    );

    // One Ctrl+Z restores the target exactly, and the applied line goes
    // with the write it announced.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(player_entry(&view, 70103), before, "one undo restores him");
    assert!(!view.session.as_ref().unwrap().can_undo(), "one undo entry");
    let lines = transplant_lines(&mut window, &mut view);
    assert!(
        !lines
            .iter()
            .any(|line| line == "Applied; unsaved until Save."),
        "the undo ends the applied line: {lines:?}"
    );
}

// SE-GUI-14
#[test]
fn another_save_keeps_a_same_version_donor_and_a_mismatched_one_refuses() {
    let (mut view, mut window) = open_734_with("transplant_reopen", &[], 0);
    let dir = scratch_of(&view);
    let donor_path = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    pick_donor(&mut window, &mut view, donor_path.clone());
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    previewed(&mut window, &mut view);

    // Another save of the same version: the donor and the selection text
    // stay, the preview goes until Preview is pressed again.
    let second = donor_save(&dir, "second", |_| {});
    view.open(&second);
    let lines = transplant_lines(&mut window, &mut view);
    assert!(
        lines
            .iter()
            .any(|line| line == &donor_path.display().to_string()),
        "the donor stays: {lines:?}"
    );
    assert_eq!(
        view.transplant.selection, "70103:70205",
        "the selection text stays"
    );
    assert!(
        view.transplant.preview.result.is_none() && view.transplant.preview.pending.is_none(),
        "the preview went"
    );
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !preview_shown(&transplant_nodes(&pairs)),
        "no rows until Preview"
    );

    // A save of another version: the donor is refused naming both, the
    // pick dropped.
    let pes21 = dir.join("open21");
    std::fs::write(&pes21, fixture_save(PesVersion::Pes21)).unwrap();
    view.open(&pes21);
    let lines = transplant_lines(&mut window, &mut view);
    assert!(
        lines.iter().any(|line| line == "(none)"),
        "the pick dropped: {lines:?}"
    );
    assert!(
        lines.iter().any(|line| line
            == "The base save is a PES 2021 save and the donor a PES 2017 one: both must be of one version"),
        "the refusal's literal: {lines:?}"
    );
}

// SE-GUI-14
#[test]
fn a_repick_withdraws_the_landed_preview() {
    let (mut view, mut window) = open_734_with("transplant_repick", &[], 0);
    let dir = scratch_of(&view);
    let donor_path = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    view.edit_aesthetics = true;
    pick_donor(&mut window, &mut view, donor_path.clone());
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    previewed(&mut window, &mut view);

    // A re-pick (the donor re-opened in the game): the held preview goes
    // until Preview is pressed again.
    pick_donor(&mut window, &mut view, donor_path);
    assert!(
        view.transplant.preview.result.is_none() && view.transplant.preview.pending.is_none(),
        "the preview went"
    );
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !preview_shown(&transplant_nodes(&pairs)),
        "no rows until Preview"
    );

    // Preview brings the rows back.
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    let pairs = previewed(&mut window, &mut view);
    transplant_button(&pairs, "Boots ID 0 \u{2192} 130");
}

// SE-GUI-14
#[test]
fn a_selection_naming_a_player_the_save_lacks_is_an_error_line() {
    let (mut view, mut window) = open_734_with("transplant_missing", &[], 0);
    let dir = scratch_of(&view);
    let donor = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    view.edit_aesthetics = true;
    pick_donor(&mut window, &mut view, donor);
    view.transplant.selection = "999999".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    let pairs = previewed(&mut window, &mut view);
    let nodes = transplant_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("the target save has no player 999999")),
        "the error line: {pairs:?}"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Player "))
        }),
        "no rows"
    );
    let (_, apply) = transplant_button(&pairs, "Apply");
    assert!(apply.is_disabled(), "Apply stays disabled");
}

/// An edit after the armed preview marks the held list `Previewing…` and
/// starts no run until the gesture ends — the mark read on a held-pointer
/// frame, where no reply can land (the comparator's sibling:
/// `an_edit_marks_the_held_list_comparing`).
#[test]
fn an_edit_marks_the_held_preview_previewing() {
    let (mut view, mut window) = open_734_with("transplant_mark", &[73401], 183);
    let dir = scratch_of(&view);
    let donor = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    pick_donor(&mut window, &mut view, donor);
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    previewed(&mut window, &mut view);

    // The pointer held on the toolbar's empty end so the re-run waits for
    // the release: a started run could reply before the mark is read.
    let at = egui::pos2(1090.0, 12.0);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    // 73401 back to his fixture height: the revision moves on.
    let mut entry = player_entry(&view, 73401);
    entry.basic.height = 185;
    view.session
        .as_mut()
        .expect("the save opened")
        .edit_players(vec![entry]);
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        view.transplant.preview.pending.is_none(),
        "premise: no run started while the pointer is held"
    );
    let nodes = transplant_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("Previewing\u{2026}")),
        "the edit marks the held preview"
    );
    assert!(
        nodes.iter().any(|node| {
            node.role() == Role::Button && node.label() == Some("Boots ID 0 \u{2192} 130")
        }),
        "the stale row stays while previewing"
    );
    window.frame(pointer_at(at, false), |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert!(
        view.transplant.preview.pending.is_some(),
        "the run starts on the release frame"
    );
    previewed(&mut window, &mut view);
}

/// The donor row stays one line however long the picked save's path is:
/// the label, the truncated path and the buttons share a top, every
/// button inside the window.
#[test]
fn the_donor_row_keeps_the_buttons_on_one_line_with_a_long_path() {
    let (mut view, mut window) = open_734_with("transplant_row", &[], 0);
    let dir = scratch_of(&view);
    let donor_path = donor_save(&dir, &"donor".repeat(40), |_| {});
    view.transplant.open = true;
    pick_donor(&mut window, &mut view, donor_path);
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (_, window_node) = pairs
        .iter()
        .find(|(_, node)| {
            node.label()
                .is_some_and(|label| label.starts_with("Aesthetics transplant"))
        })
        .expect("the window");
    let window_rect = window_node.bounds().expect("the window's bounds");
    let nodes = transplant_nodes(&pairs);
    let top = |keep: &dyn Fn(&accesskit::Node) -> bool| -> f64 {
        nodes
            .iter()
            .find(|node| keep(node))
            .and_then(|node| node.bounds())
            .expect("a drawn node's bounds")
            .y0
    };
    let tops = [
        top(&|node| node.role() == Role::Label && node.value() == Some("Donor save ")),
        top(&|node| {
            node.role() == Role::Label
                && node
                    .value()
                    .is_some_and(|value| value.contains("donordonor"))
        }),
        top(&|node| node.role() == Role::Button && node.label() == Some("Pick\u{2026}")),
        top(&|node| node.role() == Role::Button && node.label() == Some("Apply")),
    ];
    for (a, b) in tops.iter().zip(tops.iter().skip(1)) {
        assert!((a - b).abs() < 0.5, "the row's tops: {tops:?}");
    }
    for label in ["Pick\u{2026}", "Apply"] {
        let bounds = transplant_button(&pairs, label)
            .1
            .bounds()
            .expect("the button's bounds");
        assert!(
            bounds.x1 <= window_rect.x1,
            "{label} ends inside the window: {bounds:?} vs {window_rect:?}"
        );
    }
}

/// An invoked Preview (Tab + Space, or AccessKit's Click — which leaves
/// the keyboard focus where it was) starts the run: the field's hold on
/// the keyboard is given up, so `edit_settled` reports settled.
#[test]
fn an_invoked_preview_starts_the_run() {
    let (mut view, mut window) = open_734_with("transplant_invoke", &[], 0);
    let dir = scratch_of(&view);
    let donor = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    pick_donor(&mut window, &mut view, donor);
    view.transplant.selection = "70103:70205".to_owned();
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (field, _) = nodes_in_window(&pairs, "Aesthetics transplant")
        .into_iter()
        .find(|(_, node)| node.role() == Role::TextInput)
        .expect("the selection field");
    action_focus(&mut window, &mut view, field);
    window.frame(vec![egui::Event::Text(",".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(view.transplant.selection, "70103:70205,", "the key landed");
    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let (preview_button, _) = nodes_in_window(&pairs, "Aesthetics transplant")
        .into_iter()
        .find(|(_, node)| node.role() == Role::Button && node.label() == Some("Preview"))
        .expect("the Preview button");
    action_click(&mut window, &mut view, preview_button);
    previewed(&mut window, &mut view);
}

/// An edit of the selection text withdraws the held preview at once (the
/// draft's `preview = null` on change): no row and no count line is drawn
/// for the old result, and Apply disables.
#[test]
fn an_edit_of_the_selection_text_withdraws_the_preview() {
    let (mut view, mut window) = open_734_with("transplant_typing", &[], 0);
    let dir = scratch_of(&view);
    let donor = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    view.edit_aesthetics = true;
    pick_donor(&mut window, &mut view, donor);
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    let pairs = previewed(&mut window, &mut view);
    let (field, _) = nodes_in_window(&pairs, "Aesthetics transplant")
        .into_iter()
        .find(|(_, node)| node.role() == Role::TextInput)
        .expect("the selection field");
    action_focus(&mut window, &mut view, field);
    window.frame(vec![egui::Event::Text(",".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(view.transplant.selection, "70103:70205,", "the key landed");

    let (_, pairs, _) =
        window.frame_ids(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert!(
        !preview_shown(&transplant_nodes(&pairs)),
        "no row and no count of the old preview"
    );
    let (_, apply) = transplant_button(&pairs, "Apply");
    assert!(apply.is_disabled(), "Apply disabled with the preview gone");
}

// SE-GUI-14
#[test]
fn after_apply_the_preview_reads_no_changes() {
    let (mut view, mut window) = open_734_with("transplant_after", &[], 0);
    let dir = scratch_of(&view);
    let donor = donor_save(&dir, "donor", |_| {});
    view.transplant.open = true;
    view.edit_aesthetics = true;
    pick_donor(&mut window, &mut view, donor);
    view.transplant.selection = "70103:70205".to_owned();
    let nodes = draw(&mut window, &mut view, None);
    click(
        &mut window,
        node_labeled(&nodes, Role::Button, "Preview"),
        &mut view,
    );
    let pairs = previewed(&mut window, &mut view);
    let (_, apply) = transplant_button(&pairs, "Apply");
    click(&mut window, apply, &mut view);

    // Armed, the preview recomputes on the new revision: nothing left to
    // copy.
    let pairs = previewed(&mut window, &mut view);
    let nodes = transplant_nodes(&pairs);
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No player of the selection changes.")),
        "the empty read after Apply: {pairs:?}"
    );
    assert!(
        !nodes.iter().any(|node| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| label.starts_with("Boots ID"))
        }),
        "the applied row is gone"
    );
    let (_, apply) = transplant_button(&pairs, "Apply");
    assert!(apply.is_disabled(), "Apply disabled once nothing changes");
}
