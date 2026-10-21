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
/// run a frame at a time over an 800 by 600 screen (wider when a test asks,
/// the Abilities tab's rightmost controls sit past 800).
struct Headless {
    ctx: egui::Context,
    /// The screen size each frame reports.
    screen: egui::Vec2,
}

impl Headless {
    fn new() -> Headless {
        Headless::sized(800.0, 600.0)
    }

    /// A window `width` by `height` instead of the default 800 by 600.
    fn sized(width: f32, height: f32) -> Headless {
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        ctx.global_style_mut(|style| style.animation_time = 0.0);
        ctx.all_styles_mut(|style| style.interaction.tooltip_delay = 0.0);
        Headless {
            ctx,
            screen: egui::vec2(width, height),
        }
    }

    /// Runs one frame of `contents`; returns what it returned, the frame's
    /// AccessKit nodes and its painted shapes.
    fn frame<R>(
        &mut self,
        events: Vec<egui::Event>,
        contents: impl FnMut(&mut egui::Ui) -> R,
    ) -> (R, Vec<accesskit::Node>, Vec<egui::epaint::ClippedShape>) {
        let (result, nodes, shapes) = self.frame_ids(events, contents);
        (
            result,
            nodes.into_iter().map(|(_, node)| node).collect(),
            shapes,
        )
    }

    /// `frame` keeping the nodes' AccessKit ids (for `AccessKitActionRequest`
    /// events).
    fn frame_ids<R>(
        &mut self,
        events: Vec<egui::Event>,
        mut contents: impl FnMut(&mut egui::Ui) -> R,
    ) -> (
        R,
        Vec<(accesskit::NodeId, accesskit::Node)>,
        Vec<egui::epaint::ClippedShape>,
    ) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, self.screen)),
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
            update.nodes,
            shapes,
        )
    }
}

/// Writes `version`'s fixture save under a scratch folder fresh to this
/// test run (the process id, so a `.bak` left by an older run cannot shadow
/// the assertions; the folder is left, nothing removes it).
fn fixture(version: PesVersion, test: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join(format!("save_editor_view_{}", std::process::id()))
        .join(test);
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
    window.frame(pointer_at(at, true), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    window.frame(pointer_at(at, false), |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
}

/// An AccessKit `Click` action on `id`: a click delivered as the
/// accessibility event, for widgets a pointer click does not reach (a
/// combo's popup items).
fn action_click(window: &mut Headless, view: &mut SaveView, id: accesskit::NodeId) {
    window.frame(
        vec![egui::Event::AccessKitActionRequest(
            accesskit::ActionRequest {
                action: accesskit::Action::Click,
                target_tree: accesskit::TreeId::ROOT,
                target_node: id,
                data: None,
            },
        )],
        |ui| show(ui, view, &|| Vec::new(), None),
    );
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

/// Whether `node` is a player row of the list (`{number} {position} …`): a
/// Button whose label starts with the shirt number's digit.
fn player_row(node: &accesskit::Node) -> bool {
    node.role() == Role::Button
        && node
            .label()
            .and_then(|label| label.chars().next())
            .is_some_and(|c| c.is_ascii_digit())
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

    // Click 701's entry: the list holds only its players (their rows name
    // the team "/3/").
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
                && node.placeholder() == Some("Filter all teams (name, shirt, id)")
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

/// The undo stack's depth, counted by Ctrl+Z until it is empty (bounded, so
/// a stuck stack fails rather than hangs).
fn undo_entries(window: &mut Headless, view: &mut SaveView) -> usize {
    for count in 0..64 {
        if !view.session.as_ref().is_some_and(|s| s.can_undo()) {
            return count;
        }
        window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
            show(ui, view, &|| Vec::new(), None)
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
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

/// The widget nodes of `role` on `label`'s row to its right, in x order.
fn right_of_label<'a>(
    nodes: &'a [accesskit::Node],
    label: &str,
    role: Role,
) -> Vec<&'a accesskit::Node> {
    let label_node = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some(label))
        .unwrap_or_else(|| panic!("no label {label:?}"));
    let bounds = label_node.bounds().expect("the label's bounds");
    let mut found: Vec<&accesskit::Node> = nodes
        .iter()
        .filter(|node| {
            node.role() == role
                && node
                    .bounds()
                    .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
        })
        .collect();
    found.sort_by(|a, b| {
        let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
        x(a).partial_cmp(&x(b)).expect("ordered bounds")
    });
    found
}

/// The `C` `B` `A` buttons of `position`'s cell, in row order.
fn rating_buttons<'a>(nodes: &'a [accesskit::Node], position: &str) -> Vec<&'a accesskit::Node> {
    let ratings: Vec<&accesskit::Node> = right_of_label(nodes, position, Role::Button)
        .into_iter()
        .filter(|node| matches!(node.label(), Some("C" | "B" | "A")))
        .take(3)
        .collect();
    assert_eq!(ratings.len(), 3, "{position}'s three ratings");
    ratings
}

/// The first interactive node (any role but Label) on `label`'s row to its
/// right: the control the label names.
fn control_right_of<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    let label_node = nodes
        .iter()
        .find(|node| node.role() == Role::Label && node.value() == Some(label))
        .unwrap_or_else(|| panic!("no label {label:?}"));
    let bounds = label_node.bounds().expect("the label's bounds");
    nodes
        .iter()
        .filter(|node| {
            node.role() != Role::Label
                && node
                    .bounds()
                    .is_some_and(|b| (b.y0 - bounds.y0).abs() < 4.0 && b.x0 >= bounds.x1 - 1.0)
        })
        .min_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().expect("bounds").x0;
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .unwrap_or_else(|| panic!("no control right of {label:?}"))
}

/// The disabled `-` button right of `label` (a gated number control).
fn disabled_dash<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
    right_of_label(nodes, label, Role::Button)
        .into_iter()
        .find(|node| node.is_disabled() && node.label() == Some("-"))
        .unwrap_or_else(|| panic!("no disabled '-' after {label:?}"))
}

/// The tooltip text after the pointer rests at `pos` for a while (bounded:
/// a missing tooltip fails rather than hangs). The pointer first moves away
/// for a frame: egui refuses a tooltip while the last click is more recent
/// than the last movement plus 0.1 s, and both clocks advance together, so
/// a `PointerMoved` that only repeats the clicked position never clears it.
fn tooltip_after_hover(
    window: &mut Headless,
    view: &mut SaveView,
    pos: egui::Pos2,
) -> Option<String> {
    window.frame(vec![egui::Event::PointerMoved(egui::Pos2::ZERO)], |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    window.frame(vec![egui::Event::PointerMoved(pos)], |ui| {
        show(ui, view, &|| Vec::new(), None)
    });
    for _ in 0..90 {
        let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, view, &|| Vec::new(), None));
        if let Some(node) = nodes
            .iter()
            .find(|node| node.value().is_some_and(|value| value.contains("PES ")))
        {
            return node.value().map(str::to_owned);
        }
    }
    None
}

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
    view.selected = Some((702, 0));
    let a_before = name_of(&view, a_id);
    let b_before = name_of(&view, b_id);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.bounds().is_some_and(|bounds| bounds.x0 < 300.0)
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    window.frame(vec![egui::Event::Text("ZZZ".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_ne!(name_of(&view, a_id), a_before, "the edit landed on 70201");

    // Select 70202 (roster slot 1), focus his Name field, Ctrl+Z: his
    // field has only his own edit history — never 70201's text.
    view.selected = Some((702, 1));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.bounds().is_some_and(|bounds| bounds.x0 < 300.0)
        })
        .expect("the Name field");
    click(&mut window, name_field, &mut view);
    window.frame(vec![key(egui::Key::Z, egui::Modifiers::CTRL, true)], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    assert_eq!(name_of(&view, b_id), b_before, "70202's name is unchanged");
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
    window.frame(vec![egui::Event::Text("77".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
    let before_b = speed_of(&view, b_id);

    // Select 70202 (roster slot 1) in the same sequence: his speed is
    // untouched — and the typed value went nowhere (no stray commit).
    view.selected = Some((702, 1));
    assert_eq!(
        speed_of(&view, b_id),
        before_b,
        "70202's Speed is unchanged"
    );
    assert_eq!(
        speed_of(&view, a_id),
        77,
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
}

// The same sub-bullet: the shirt name is upper-cased as typed.
#[test]
fn a_shirt_name_is_upper_cased_as_typed() {
    let path = fixture(PesVersion::Pes21, "uppercase_shirt");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(1100.0, 700.0);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let shirt = nodes
        .iter()
        .filter(|node| node.role() == Role::TextInput && !node.is_disabled())
        .max_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().map(|r| r.x0).unwrap_or(0.0);
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("the Shirt field");
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
// layout" (Studio opens at half the monitor, 960 × 540 on 1080p): the tab's
// field/control pairs wrap below one another rather than clip — every
// control's rect lies inside the window.
#[test]
fn the_tab_fits_the_half_monitor_window() {
    let path = fixture(PesVersion::Pes21, "half_monitor");
    let mut view = SaveView::new();
    view.open(&path);
    let mut window = Headless::sized(960.0, 540.0);
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
                bounds.x1 <= 960.0,
                "{role:?} {} clips the window's right edge ({bounds:?})",
                node.label().unwrap_or("?")
            );
        }
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

    // A rules file's tiers are the badges': DBC7's, no Bronze.
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
    let mut view = SaveView::new();
    view.open(&path);
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

// spec/pes_savefile/operations.md "Team TOML" (the portable text limits)
// and plans/save_editor.md "Editing UI" (the info strip bullet): a stored
// shirt name past 18 letters is drawn tinted with the versions' hover; an
// edit may shorten it a keystroke at a time, never lengthen.
#[test]
fn a_stored_shirt_past_18_is_tinted_and_shortens_never_lengthens() {
    /// The fill under `bounds`' centre: the smallest `Shape::Rect` covering
    /// it (the field's own frame).
    fn fill_at(
        shapes: &[egui::epaint::ClippedShape],
        bounds: accesskit::Rect,
    ) -> Option<egui::Color32> {
        let centre = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect.contains(centre) => {
                    Some((shape.fill, shape.rect.area()))
                }
                _ => None,
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).expect("areas"))
            .map(|(fill, _)| fill)
    }
    /// The Shirt field's `TextInput` (the rightmost enabled text edit).
    fn shirt_field(nodes: &[accesskit::Node], value: &str) -> accesskit::Rect {
        nodes
            .iter()
            .filter(|node| {
                node.role() == Role::TextInput && !node.is_disabled() && node.value() == Some(value)
            })
            .map(|node| node.bounds().expect("bounds"))
            .max_by(|a, b| a.x0.partial_cmp(&b.x0).expect("ordered bounds"))
            .unwrap_or_else(|| panic!("no Shirt field {value:?}"))
    }

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
    let bounds = shirt_field(&nodes, &"S".repeat(20));
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
        Some("PES 2019 and older show 18 letters of a shirt name (PES 2016: 16)".to_string()),
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
    window.frame(vec![egui::Event::Text("SS".to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
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
    let shirt = nodes
        .iter()
        .filter(|node| node.role() == Role::TextInput && !node.is_disabled())
        .max_by(|a, b| {
            let x = |node: &&accesskit::Node| node.bounds().map(|r| r.x0).unwrap_or(0.0);
            x(a).partial_cmp(&x(b)).expect("ordered bounds")
        })
        .expect("the Shirt field");
    let stored = shirt.value().expect("its text").to_owned();
    click(&mut window, shirt, &mut view);
    window.frame(vec![egui::Event::Text("S".repeat(17).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
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

// `Tool::view` draws the window through the tool impl (a settings error
// counts as an empty `aatf_rules` setting).
#[test]
fn the_tool_view_draws_the_window() {
    use studio_core::StudioTool;

    let (events_tx, _events_rx) = crossbeam_channel::unbounded();
    let (requests_tx, _requests_rx) = crossbeam_channel::unbounded();
    let ctx = studio_core::ToolContext::new(
        std::sync::Arc::new(std::sync::Mutex::new(studio_core::Settings::default())),
        studio_core::AppPaths {
            exe_dir: std::path::PathBuf::from("exe"),
            data_dir: None,
        },
        events_tx,
        requests_tx,
    );
    let mut tool = crate::Tool::new();
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| tool.view(ui, &ctx));
    assert!(
        nodes
            .iter()
            .any(|node| node.value() == Some("No save open.")),
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
    // boxes carry their names.
    let toggled = |nodes: &[accesskit::Node]| {
        nodes
            .iter()
            .find(|node| node.role() == Role::RadioButton && node.label() == Some(""))
            .map(|node| node.toggled())
    };
    // Select the captain's slot: the radio is on.
    view.selected = Some((701, captain));
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    assert_eq!(toggled(&nodes), Some(Some(accesskit::Toggled::True)));
    // Another slot: off.
    let other = if captain == 0 { 1 } else { 0 };
    view.selected = Some((701, other));
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
    view.selected = Some((701, captain));
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let radio = nodes
        .iter()
        .find(|node| node.role() == Role::RadioButton && node.label() == Some(""))
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
        nodes.iter().any(|node| node.value() == Some(id.as_str())),
        "the info strip shows the last player's ID"
    );
}

// plans/save_editor.md "Editing UI", the info strip bullet's Name
// sub-bullet: the save's own field bounds an edit (a PES 21 save takes 47
// to 61 bytes, a name past 46 tinted with the cut the older versions make).
#[test]
fn a_pes21_strip_takes_a_long_name_tinted_with_its_crop() {
    // The fill under `bounds`' centre: the smallest `Shape::Rect` covering
    // it (the field's own frame; the panel's background would win an
    // `intersects` match instead).
    fn fill_at(
        shapes: &[egui::epaint::ClippedShape],
        bounds: accesskit::Rect,
    ) -> Option<egui::Color32> {
        let centre = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect.contains(centre) => {
                    Some((shape.fill, shape.rect.area()))
                }
                _ => None,
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).expect("areas"))
            .map(|(fill, _)| fill)
    }
    /// The Name field's `TextInput` node (the enabled text edit left of the
    /// panel's middle).
    fn name_field(nodes: &[accesskit::Node], value: &str) -> accesskit::Rect {
        nodes
            .iter()
            .find(|node| {
                node.role() == Role::TextInput
                    && !node.is_disabled()
                    && node.value() == Some(value)
                    && node.bounds().is_some_and(|bounds| bounds.x0 < 300.0)
            })
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
            "In PES 2019 and older this name will be cut to: «{}{}»",
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
    view.selected = Some((701, 0));
    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &|| Vec::new(), None));
    let name_field = nodes
        .iter()
        .find(|node| {
            node.role() == Role::TextInput
                && !node.is_disabled()
                && node.bounds().is_some_and(|bounds| bounds.x0 < 300.0)
        })
        .expect("the Name field");
    let stored = name_field.value().expect("its text").to_owned();
    click(&mut window, name_field, &mut view);
    // 47 bytes past this version's own 46-byte field: refused.
    window.frame(vec![egui::Event::Text("Z".repeat(47).to_owned())], |ui| {
        show(ui, &mut view, &|| Vec::new(), None)
    });
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
