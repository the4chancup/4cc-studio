//! The headless-window tests of the view's scenarios (`plans/save_editor.md`
//! "Acceptance", the Phase 8 scope paragraph): one egui context with AccessKit
//! on, run a frame at a time; what a frame draws is its AccessKit nodes and
//! painted shapes.

use std::path::PathBuf;

use egui::accesskit::{self, Role};
use pes_savefile::model::names::name_spans;
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
    window.frame(pointer_at(at, true), |ui| show(ui, view, &[]));
    window.frame(pointer_at(at, false), |ui| show(ui, view, &[]));
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
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &[]));

    let selector = nodes
        .iter()
        .find(|node| node.role() == Role::ComboBox)
        .expect("the team selector");
    assert_eq!(selector.value(), Some("ALL"));

    let rows = nodes
        .iter()
        .filter(|node| node.role() == Role::Button && node.label() != Some("Open…"))
        .count();
    assert_eq!(rows, rostered, "a row per rostered player of every team");
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
    let (_, _, shapes) = window.frame(Vec::new(), |ui| show(ui, &mut view, &[]));

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

    // Click the filter box (the one enabled text input), then type the name.
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &[]));
    let filter_box = nodes
        .iter()
        .find(|node| node.role() == Role::TextInput && !node.is_disabled())
        .expect("the filter box");
    click(&mut window, filter_box, &mut view);
    let (_, nodes, _) = window.frame(vec![egui::Event::Text("quality".to_owned())], |ui| {
        show(ui, &mut view, &[])
    });

    // 70202 of team 702 is listed, his row naming his team under a filter.
    let row = nodes
        .iter()
        .find(|node| node.role() == Role::Button && node.label() == Some("1 GK /a/ QUALITY"))
        .expect("the filtered row");
    click(&mut window, row, &mut view);
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &[]));
    assert!(
        nodes.iter().any(|node| node.value() == Some("70202")),
        "the info strip shows his ID"
    );
}

#[test]
fn an_unreadable_file_is_refused_and_keeps_the_open_save() {
    let good = fixture(PesVersion::Pes21, "refused_open");
    let mut view = SaveView::new();
    view.open(&good);
    let bad = good.with_file_name("not_a_save.bin");
    std::fs::write(&bad, b"this is no save").unwrap();
    view.open(&bad);

    let mut window = Headless::new();
    let (_, nodes, _) = window.frame(Vec::new(), |ui| show(ui, &mut view, &[]));
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
