//! The geometry and paint tests: the grid math, the pitch's lines,
//! the markers' fills, strokes and texts, the rows' coloured names.

use pes_version::PesVersion;

use super::*;
use crate::test_support::{Headless, fixture_rows, fixture_tactics, roster_rows};

// The draft's `snapXY`: the nearest stored unit, clamped to the grid.
#[test]
fn snap_rounds_to_the_nearest_unit_and_clamps_to_the_grid() {
    let origin = egui::pos2(40.0, 80.0);
    assert_eq!(snap(origin, origin), (0, 48), "the canvas' own corner");
    assert_eq!(snap(origin + PITCH, origin), (104, 0), "the far corner");
    assert_eq!(
        snap(
            origin + egui::vec2(MARGIN.x + 1.4 * UNIT.x, MARGIN.y),
            origin
        ),
        (1, 48),
        "1.4 units right rounds to 1"
    );
    assert_eq!(
        snap(origin + PITCH + egui::vec2(50.0, 50.0), origin),
        (104, 0),
        "a point past the canvas clamps"
    );
    assert_eq!(
        snap(origin - egui::vec2(50.0, 50.0), origin),
        (0, 48),
        "a point before it clamps the same way"
    );
    assert_eq!(
        snap(origin + MARGIN + egui::vec2(0.0, 10.0 * UNIT.y), origin),
        (0, 38),
        "ten units down the grid is y 38"
    );
}

// The draft's `pitchX`/`pitchY` and chip clamp: a stored (x, y) lands
// `UNIT` a unit inside the margin, high y on top; the chip centres on
// the point and clamps 2 px inside the canvas.
#[test]
fn markers_sit_on_the_grid_and_clamp_inside_the_canvas() {
    let origin = egui::pos2(10.0, 20.0);
    let canvas = egui::Rect::from_min_size(origin, PITCH);
    assert_eq!(
        pitch_point(origin, 0, 48),
        egui::pos2(14.0, 22.0),
        "the grid's own corner"
    );
    assert_eq!(
        pitch_point(origin, 52, 24),
        egui::pos2(170.0, 190.0),
        "the grid's centre"
    );
    assert_eq!(
        pitch_point(origin, 104, 0),
        egui::pos2(326.0, 358.0),
        "the far corner"
    );
    assert_eq!(
        marker_rect(egui::pos2(170.0, 190.0), canvas),
        egui::Rect::from_min_size(egui::pos2(149.0, 177.0), CHIP),
        "the chip centres on the point"
    );
    assert_eq!(
        marker_rect(egui::pos2(14.0, 22.0), canvas).min,
        egui::pos2(12.0, 22.0),
        "the top-left clamp"
    );
    assert_eq!(
        marker_rect(egui::pos2(326.0, 358.0), canvas).min,
        egui::pos2(286.0, 330.0),
        "the bottom-right clamp"
    );
}

// `paint_pitch`'s lines, read off the painted shapes in canvas-
// relative coordinates: the grid, the two boxes and six-yard boxes
// stroked in LINES, the halfway line and the centre circle.
#[test]
fn the_painted_lines_are_the_pitches_markings() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let at = |x: f32, y: f32| canvas.min + egui::vec2(x, y);
    let mut rects: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape) if shape.stroke.color == LINES => Some(shape.rect),
            _ => None,
        })
        .collect();
    rects.sort_by(|a, b| {
        (a.min.y, a.min.x)
            .partial_cmp(&(b.min.y, b.min.x))
            .expect("ordered bounds")
    });
    assert_eq!(
        rects,
        [
            egui::Rect::from_min_size(at(4.0, 2.0), egui::vec2(312.0, 336.0)),
            egui::Rect::from_min_size(at(80.0, 2.0), egui::vec2(160.0, 52.0)),
            egui::Rect::from_min_size(at(122.0, 2.0), egui::vec2(76.0, 18.0)),
            egui::Rect::from_min_size(at(80.0, 286.0), egui::vec2(160.0, 52.0)),
            egui::Rect::from_min_size(at(122.0, 320.0), egui::vec2(76.0, 18.0)),
        ],
        "the grid, the boxes and the six-yard boxes"
    );
    let halfway: Vec<[egui::Pos2; 2]> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { points, stroke } if stroke.color == LINES => Some(*points),
            _ => None,
        })
        .collect();
    assert_eq!(
        halfway,
        [[at(4.0, 170.0), at(316.0, 170.0)]],
        "the halfway line"
    );
    let circles: Vec<(egui::Pos2, f32)> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Circle(shape) if shape.stroke.color == LINES => {
                Some((shape.center, shape.radius))
            }
            _ => None,
        })
        .collect();
    assert_eq!(circles, [(at(160.0, 170.0), 30.0)], "the centre circle");
}

// `marker`'s armed fill: under join mode a member's chip paints
// JOIN_FILL, a non-member's CHIP_FILL, the keeper's KEEPER_FILL; with
// nothing armed the member's is CHIP_FILL again.
#[test]
fn the_join_modes_members_paint_in_join_fill() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    tactics.players_to_join_attack = [tactics.starting_eleven[3], u8::MAX, u8::MAX];
    view.assign = Some(Assign::Join);
    let players = tactics.presets[0].formations[0].players;
    let keeper = players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");
    assert!(keeper != 3 && keeper != 4, "premise: 3 and 4 are outfield");

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let fill_of = |shapes: &[egui::epaint::ClippedShape], canvas: egui::Rect, index: usize| {
        let slot = players[index];
        let rect = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas);
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect == rect => Some(shape.fill),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted rect for marker {index}"))
    };
    assert_eq!(fill_of(&shapes, canvas, 3), JOIN_FILL, "the member's chip");
    assert_eq!(
        fill_of(&shapes, canvas, 4),
        CHIP_FILL,
        "a non-member's chip"
    );
    assert_eq!(
        fill_of(&shapes, canvas, keeper),
        KEEPER_FILL,
        "the keeper's chip"
    );

    view.assign = None;
    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    assert_eq!(
        fill_of(&shapes, canvas, 3),
        CHIP_FILL,
        "unarmed, the member's chip is the ordinary fill"
    );
}

// `marker`'s open stroke: the marker whose popover is open strokes
// OPEN_STROKE, an unopened one CHIP_STROKE.
#[test]
fn the_open_popovers_marker_strokes_orange() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;
    let index = 3;

    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let at = marker_centre(&nodes, players[index]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(index),
        "the popover is slot 3's"
    );

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let stroke_of = |index: usize| {
        let slot = players[index];
        let rect = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas);
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect == rect => Some(shape.stroke.color),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted rect for marker {index}"))
    };
    assert_eq!(stroke_of(3), OPEN_STROKE, "the open slot's marker");
    assert_eq!(stroke_of(4), CHIP_STROKE, "an unopened one");
}

// `marker`'s texts: the position label centred on `rect.top() + 7`,
// the name cut to 8 characters on `rect.top() + 19`.
#[test]
fn the_markers_texts_anchor_inside_the_chip() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;
    let index = 4;
    let slot = players[index];
    let name: String = name_at(&roster, tactics.starting_eleven[index])
        .chars()
        .take(8)
        .collect();

    let (nodes, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let rect = marker_rect(pitch_point(canvas.min, slot.x, slot.y), canvas);
    let text_at = |pos: egui::Pos2| {
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(shape)
                    if shape.visual_bounding_rect().center().distance(pos) <= 1.0 =>
                {
                    Some(shape.galley.job.text.as_str())
                }
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no painted text at {pos:?}"))
    };
    assert_eq!(
        text_at(egui::pos2(rect.center().x, rect.top() + 7.0)),
        position_label(slot.position),
        "the position label"
    );
    assert_eq!(
        text_at(egui::pos2(rect.center().x, rect.top() + 19.0)),
        name,
        "the name"
    );
}

// The draft's `nameSpans`: the eleven rows, the bench rows and the
// popover header draw the name with its colour codes.
#[test]
fn the_rows_and_the_popover_header_draw_the_names_colours() {
    let mut window = Headless::sized(900.0, 1100.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let mut rows = fixture_rows(PesVersion::Pes21);
    rows[5] = (
        "PEPSI MAN".to_owned(),
        "\u{11}cC9002BFFPEP\u{11}cFFFFFFFFSI \u{11}c004B93FFMAN".to_owned(),
        Some(5),
    );
    let roster = roster_rows(&rows);
    // Slot 5 is in the fixture's eleven: swap so it shows once.
    tactics.starting_eleven.swap(3, 5);
    tactics.bench_order[0] = 5;

    // The color of each painted "PEPSI MAN" galley's section covering
    // `run`'s first byte.
    let run_colors = |shapes: &[egui::epaint::ClippedShape], run: &str| {
        let start = "PEPSI MAN".find(run).expect("the run's offset");
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(shape) if shape.galley.job.text == "PEPSI MAN" => Some(
                    shape
                        .galley
                        .job
                        .sections
                        .iter()
                        .find(|section| {
                            section.byte_range.start.0 <= start && start < section.byte_range.end.0
                        })
                        .unwrap_or_else(|| panic!("no section covering {run:?}"))
                        .format
                        .color,
                ),
                _ => None,
            })
            .collect::<Vec<egui::Color32>>()
    };

    let (_, shapes) = window.lineup_paint(&mut tactics, &mut view, &roster);
    let colors = run_colors(&shapes, "PEP");
    assert_eq!(colors.len(), 2, "the eleven row's and the bench row's name");
    for color in colors {
        assert_eq!(
            color,
            egui::Color32::from_rgb(0xc9, 0x00, 0x2b),
            "PEP's run keeps its code's colour"
        );
    }
    for color in run_colors(&shapes, "SI ") {
        assert_eq!(
            color,
            egui::Color32::from_rgb(0xff, 0xff, 0xff),
            "SI's run keeps its code's colour"
        );
    }
    for color in run_colors(&shapes, "MAN") {
        assert_eq!(
            color,
            egui::Color32::from_rgb(0x00, 0x4b, 0x93),
            "MAN's run keeps its code's colour"
        );
    }

    // The popover header's.
    let (nodes, _) = window.pitch(&mut tactics, &mut view, &roster, vec![]);
    let formation = tactics.presets[0].formations[0].clone();
    let at = marker_centre(&nodes, formation.players[3]);
    window.click_pitch(at, &mut tactics, &mut view, &roster);
    let (_, shapes) = window.pitch_paint(&mut tactics, &mut view, &roster);
    let colors = run_colors(&shapes, "PEP");
    assert_eq!(colors.len(), 1, "the popover header's name");
    assert_eq!(
        colors[0],
        egui::Color32::from_rgb(0xc9, 0x00, 0x2b),
        "PEP's run keeps its code's colour"
    );
}
