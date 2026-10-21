//! The headless-window tests of the view's scenarios (`plans/save_editor.md`
//! "Acceptance", the Phase 8 scope paragraph): one egui context with AccessKit
//! on, run a frame at a time; what a frame draws is its AccessKit nodes and
//! painted shapes.

use std::path::PathBuf;

use egui::accesskit::{self, Role};
use pes_savefile::model::names::name_spans;
use pes_savefile::schema::fields::PlayerField;
use pes_savefile::{EditFile, fixture_save};
use pes_version::PesVersion;

use super::*;

/// A headless window: one egui context with AccessKit on and animations off,
/// run a frame at a time over an 800 by 600 screen.
struct Headless {
    ctx: egui::Context,
}

impl Headless {
    fn new() -> Headless {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.global_style_mut(|style| style.animation_time = 0.0);
        Headless { ctx }
    }

    /// Runs one frame of `contents`; returns what it returned, the frame's
    /// AccessKit nodes and its painted shapes.
    fn frame<R>(
        &mut self,
        events: Vec<egui::Event>,
        mut contents: impl FnMut(&mut egui::Ui) -> R,
    ) -> (R, Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        };
        let mut result = None;
        let mut output = self.ctx.run_ui(input, |ui| result = Some(contents(ui)));
        let update = output
            .platform_output
            .accesskit_update
            .take()
            .expect("AccessKit is enabled on the context");
        let shapes = std::mem::take(&mut output.shapes);
        // Nothing renders here, so the frame's texture deltas are dropped on
        // purpose.
        output.drop_without_applying_deltas();
        (
            result.expect("run_ui calls the closure"),
            update.nodes.into_iter().map(|(_, node)| node).collect(),
            shapes,
        )
    }
}

/// Writes `version`'s fixture save under the test's own scratch folder and
/// returns its path.
fn fixture(version: PesVersion, test: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("save_editor_view").join(test);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("EDIT00000000");
    std::fs::write(&path, fixture_save(version)).unwrap();
    path
}

/// The centre of a node's bounds, where a click lands.
fn centre(node: &accesskit::Node) -> egui::Pos2 {
    let bounds = node.bounds().expect("a widget has bounds");
    egui::pos2(
        ((bounds.x0 + bounds.x1) / 2.0) as f32,
        ((bounds.y0 + bounds.y1) / 2.0) as f32,
    )
}

/// The primary button pressed (or released) at `pos`, the pointer moved
/// there first.
fn pointer_at(pos: egui::Pos2, pressed: bool) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::default(),
        },
    ]
}

/// A click on `node`: press on one frame, release on the next.
fn click(window: &mut Headless, node: &accesskit::Node, view: &mut SaveView) {
    let at = centre(node);
    window.frame(pointer_at(at, true), |ui| show(ui, view, &|| Vec::new()));
    window.frame(pointer_at(at, false), |ui| show(ui, view, &|| Vec::new()));
}

/// A key pressed (or released) with `modifiers` held.
fn key(key: egui::Key, modifiers: egui::Modifiers, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed,
        modifiers,
        repeat: false,
    }
}

/// The AccessKit node of the row labeled `label`.
fn row<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some(label))
        .unwrap_or_else(|| panic!("no row {label:?}"))
}

/// The painted text shape whose galley text starts with `starts_with`.
fn text_shape<'a>(
    shapes: &'a [egui::epaint::ClippedShape],
    starts_with: &str,
) -> &'a egui::epaint::TextShape {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text.starts_with(starts_with) => {
                Some(shape)
            }
            _ => None,
        })
        .next()
        .unwrap_or_else(|| panic!("no painted text starting {starts_with:?}"))
}

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
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));

    let selector = nodes
        .iter()
        .find(|node| node.role() == Role::ComboBox)
        .expect("the team selector");
    assert_eq!(selector.value(), Some("ALL"));

    // The list lays out only the visible rows (`show_rows`): a screenful is
    // drawn, and scrolling reaches every rostered player (the tail's test).
    let drawn = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && !matches!(node.label(), Some("Open…" | "Save" | "Undo" | "Redo"))
        })
        .count();
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
    // them: id 70101, height 180, weight 80.
    assert!(
        nodes.iter().any(|node| node.value() == Some("70101")),
        "the strip's ID"
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
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));

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
    // The painted sections resolve the row's uncolored runs to its normal
    // text color: "50 GK " is painted in it, and it is what `name_spans`'
    // `None` means. The layout merges adjacent equal-color sections, so the
    // expectation merges too.
    let normal = painted
        .first()
        .map(|(color, _)| *color)
        .expect("the row's prefix section");
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let filter_box = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && node.placeholder() == Some("Filter all teams (name, shirt, id)")
        })
        .expect("the filter box");
    click(&mut window, filter_box, &mut view);
    let (_, nodes, _) = window.frame(vec![egui::Event::Text("quality".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });

    // 70202 of team 702 is listed, his row naming his team under a filter.
    let row = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("1 GK /a/ QUALITY"))
        .expect("the filtered row");
    click(&mut window, row, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    assert!(
        nodes.iter().any(|node| node.value() == Some("70202")),
        "the info strip shows his ID"
    );
}

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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let height = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
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
        show(ui, &mut view, &|| Vec::new())
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let height = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let weight = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("80"))
        .expect("the Weight control");
    click(&mut window, weight, &mut view);
    window.frame(vec![egui::Event::Text("90".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    assert_eq!(basics(&view), (170, 90));

    // Ctrl+Z takes back the second edit only; Ctrl+Y redoes it.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    assert_eq!(
        basics(&view),
        (170, 80),
        "the weight edit undone, height kept"
    );
    window.frame(vec![key(egui::Key::Y, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new())
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let control = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    let at = centre(control);
    window.frame(pointer_at(at, true), |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    for step in 1..=5 {
        window.frame(
            vec![egui::Event::PointerMoved(egui::pos2(
                at.x + 3.0 * step as f32,
                at.y,
            ))],
            |ui| show(ui, &mut view, &|| Vec::new()),
        );
    }
    window.frame(pointer_at(egui::pos2(at.x + 15.0, at.y), false), |ui| {
        show(ui, &mut view, &|| Vec::new())
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let at = centre(
        nodes
            .iter()
            .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
            .expect("the Height control"),
    );
    let drag = |window: &mut Headless, view: &mut SaveView, at: egui::Pos2| {
        window.frame(pointer_at(at, true), |ui| show(ui, view, &|| Vec::new()));
        window.frame(
            vec![egui::Event::PointerMoved(egui::pos2(at.x + 30.0, at.y))],
            |ui| show(ui, view, &|| Vec::new()),
        );
        window.frame(pointer_at(egui::pos2(at.x + 30.0, at.y), false), |ui| {
            show(ui, view, &|| Vec::new())
        });
    };
    drag(&mut window, &mut view, at);
    drag(&mut window, &mut view, at);
    assert_eq!(undo_entries(&mut window, &mut view), 2, "two entries");
}

/// The undo stack's depth, counted by Ctrl+Z until it is empty (bounded, so
/// a stuck stack fails rather than hangs).
fn undo_entries(window: &mut Headless, view: &mut SaveView) -> usize {
    for count in 0..64 {
        if !view.session.as_ref().is_some_and(|s| s.can_undo()) {
            return count;
        }
        window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
            show(ui, view, &|| Vec::new())
        });
    }
    panic!("Ctrl+Z did not empty the undo stack in 64 presses")
}

// SE-GUI-21 (the scenario is manual; this is its logic)
#[test]
fn save_is_refused_while_the_saves_own_pes_runs() {
    let path = fixture(PesVersion::Pes21, "save_refused");
    let original = std::fs::read(&path).unwrap();
    let mut view = SaveView::new();
    view.open(&path);
    // An edit, so the refusal leaves something behind.
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
    let running_own = [RunningPes {
        version: PesVersion::Pes21,
        exe: None,
    }];
    save(&mut view, &|| running_own.to_vec());
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    assert!(
        nodes.iter().any(|node| {
            node.value()
                .is_some_and(|value| value.contains("PES 2021 is running"))
        }),
        "the modal names the game"
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "the file is unchanged"
    );
    assert!(
        view.session.as_ref().unwrap().is_dirty(),
        "the refused save did not write"
    );
    // OK dismisses it.
    click(
        &mut window,
        nodes
            .iter()
            .find(|node| node.role() == Role::Button && node.label() == Some("OK"))
            .expect("the modal's OK"),
        &mut view,
    );
    assert!(view.refusal.is_none(), "OK closes the refusal");
    // A game of another version does not block it; a write that succeeds
    // clears the refusal an earlier one left too.
    let running_other = [RunningPes {
        version: PesVersion::Pes19,
        exe: None,
    }];
    save(&mut view, &|| running_own.to_vec());
    assert!(view.refusal.is_some());
    save(&mut view, &|| running_other.to_vec());
    assert!(
        view.refusal.is_none(),
        "a successful save clears the refusal"
    );
    assert!(!view.session.as_ref().unwrap().is_dirty(), "the save wrote");
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let height = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("180"))
        .expect("the Height control");
    click(&mut window, height, &mut view);
    window.frame(vec![egui::Event::Text("170".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    row(&nodes, "1 CB /3/ PLACEHOLDER ●");
    // Undoing the only edit clears the dot.
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    row(&nodes, "1 CB /3/ PLACEHOLDER");
    assert!(
        !nodes
            .iter()
            .any(|node| node.label().is_some_and(|label| label.contains('●'))),
        "the dot is gone"
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
    window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    window.frame(
        vec![key(egui::Key::Escape, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    assert!(view.refusal.is_none(), "Escape closes it");
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

    let toggled = |nodes: &[accesskit::Node]| {
        nodes
            .iter()
            .find(|node| node.role() == Role::CheckBox)
            .map(|node| node.toggled())
    };
    // Select the captain's slot: the box is ticked.
    view.selected = Some((701, captain));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    assert_eq!(toggled(&nodes), Some(Some(accesskit::Toggled::True)));
    // Another slot: unticked.
    let other = if captain == 0 { 1 } else { 0 };
    view.selected = Some((701, other));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    assert_eq!(toggled(&nodes), Some(Some(accesskit::Toggled::False)));
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let age = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("20"))
        .expect("the Age control");
    click(&mut window, age, &mut view);
    window.frame(vec![egui::Event::Text("99".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let number = nodes
        .iter()
        .find(|node| node.role() == Role::SpinButton && node.value() == Some("1"))
        .expect("the Number control");
    click(&mut window, number, &mut view);
    window.frame(vec![egui::Event::Text("23".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    window.frame(
        vec![key(egui::Key::Enter, egui::Modifiers::NONE, true)],
        |ui| show(ui, &mut view, &|| Vec::new()),
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

// The selected row's AccessKit node reports toggled, the others not (egui
// maps a selectable's `selected` to `toggled`).
#[test]
fn the_selected_row_reports_selected() {
    let path = fixture(PesVersion::Pes21, "selected_row");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let rows: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == Role::Button
                && !matches!(node.label(), Some("Open…" | "Save" | "Undo" | "Redo"))
        })
        .collect();
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
    // bar, a move to its last pixels, release.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let bar = nodes
        .iter()
        .find(|node| node.role() == Role::ScrollBar)
        .expect("the list's scroll bar");
    let track = centre(bar);
    window.frame(
        pointer_at(egui::pos2(track.x, track.y - 200.0), true),
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    window.frame(
        vec![egui::Event::PointerMoved(egui::pos2(track.x, 595.0))],
        |ui| show(ui, &mut view, &|| Vec::new()),
    );
    window.frame(pointer_at(egui::pos2(track.x, 595.0), false), |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let last = row(&nodes, &label);
    click(&mut window, last, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let id = roster_slot.player_id.to_string();
    assert!(
        nodes.iter().any(|node| node.value() == Some(id.as_str())),
        "the info strip shows the last player's ID"
    );
}

// plans/save_editor.md "Editing UI", the strip's rule that a typed value is
// never one the codec refuses: a name past the field's byte limit is not
// pushed, the field reverts, and Save then writes the file.
#[test]
fn a_name_past_the_fields_limit_is_not_taken() {
    let path = fixture(PesVersion::Pes21, "overlong_name");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::new();
    // The Name field is the enabled text input carrying the stored name.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.value() == Some("PLACEHOLDER")
                && node.bounds().is_some_and(|bounds| bounds.x0 < 300.0)
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    // 71 bytes past the field's limit (61 on this version).
    window.frame(vec![egui::Event::Text("Z".repeat(60).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
    assert!(
        nodes
            .iter()
            .any(|node| { node.role() == Role::TextInput && node.value() == Some("PLACEHOLDER") }),
        "the field reverted to the stored name"
    );
    assert!(
        !view.session.as_ref().unwrap().is_dirty(),
        "the refused edit was not pushed"
    );
    // Ctrl+S writes the file (the `.bak` of the previous one is the proof).
    window.frame(vec![key(egui::Key::S, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new())
    });
    assert!(view.refusal.is_none());
    let bak = path.with_file_name("EDIT00000000.bak");
    assert!(bak.exists(), "the save wrote ({})", bak.display());
    let reopened = Session::open(&path).unwrap();
    assert_eq!(reopened.file().player(70101).unwrap().name, "PLACEHOLDER");
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new()));
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
