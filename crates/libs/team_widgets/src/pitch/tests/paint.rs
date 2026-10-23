//! The geometry and paint tests: the screen's spot mapping and its snap,
//! the turf and the lines, the markers' badge, position, name, portrait
//! and corners, the rows' coloured names.

use pes_version::PesVersion;

use super::*;
use crate::test_support::{Headless, fixture_rows, fixture_tactics, painted_color, roster_rows};

/// An outfield slot at (`x`, `y`), for the pure mapping tests.
fn outfield(x: u8, y: u8) -> FormationSlot {
    FormationSlot { position: 1, x, y }
}

/// `a` within `eps` of `b`, both coordinates.
fn near(a: egui::Pos2, b: egui::Pos2, eps: f32) {
    assert!(
        (a.x - b.x).abs() <= eps && (a.y - b.y).abs() <= eps,
        "{a:?} vs {b:?}"
    );
}

// The screen's own slot-to-spot mapping (the plan's pitch bullet): x 0
// to 104 across the canvas' width, y 24 the halfway line, the attack end
// up; the keeper sits on the goal area's line whatever his stored spot.
#[test]
fn the_badge_centre_is_the_screens_spot_mapping() {
    let origin = egui::pos2(40.0, 80.0);
    near(
        badge_centre(origin, outfield(52, 24)),
        origin + egui::vec2(159.8, 200.8),
        0.05,
    );
    near(
        badge_centre(origin, outfield(0, 0)),
        origin + egui::vec2(12.8, 363.8),
        0.05,
    );
    near(
        badge_centre(origin, outfield(104, 48)),
        origin + egui::vec2(306.8, 37.7),
        0.05,
    );
    for (x, y) in [(52, 3), (10, 40)] {
        near(
            badge_centre(origin, FormationSlot { position: 0, x, y }),
            origin + egui::vec2(160.0, 358.5),
            0.05,
        );
    }
}

// `snap` is the mapping's inverse: every stored (x, y) round-trips, and
// a point past each edge clamps to it.
#[test]
fn snap_inverts_the_mapping_and_clamps_to_the_grid() {
    let origin = egui::pos2(40.0, 80.0);
    for x in 0..=GRID_X {
        for y in 0..=GRID_Y {
            let slot = outfield(x, y);
            assert_eq!(
                snap(badge_centre(origin, slot), origin),
                (x, y),
                "({x}, {y})"
            );
        }
    }
    assert_eq!(
        snap(origin + egui::vec2(500.0, 200.0), origin),
        (104, 24),
        "past the right edge"
    );
    assert_eq!(
        snap(origin + egui::vec2(160.0, 500.0), origin),
        (52, 0),
        "past the near edge"
    );
    assert_eq!(
        snap(origin + egui::vec2(-50.0, -50.0), origin),
        (0, 48),
        "past the far corner"
    );
}

// The marker's whole box: the badge's width, the portrait's top to the
// name's bottom, the badge centre clamped so it stays inside the canvas.
#[test]
fn the_marker_box_centres_on_the_badge_and_stays_inside() {
    let origin = egui::pos2(40.0, 80.0);
    let canvas = egui::Rect::from_min_size(origin, PITCH);
    let rect = marker_rect(badge_centre(origin, outfield(52, 24)), canvas);
    near(rect.min, origin + egui::vec2(134.8, 143.8), 0.05);
    near(rect.max, origin + egui::vec2(184.8, 220.8), 0.05);
    assert_eq!(
        marker_rect(origin + egui::vec2(-30.0, 200.0), canvas).left(),
        canvas.left(),
        "a centre past the left edge clamps to it"
    );
    assert_eq!(
        marker_rect(origin + egui::vec2(350.0, 200.0), canvas).right(),
        canvas.right(),
        "a centre past the right edge clamps to it"
    );
    assert_eq!(
        marker_rect(origin + egui::vec2(160.0, -30.0), canvas).top(),
        canvas.top(),
        "a centre past the top clamps to it"
    );
    assert_eq!(
        marker_rect(origin + egui::vec2(160.0, 410.0), canvas).bottom(),
        canvas.bottom(),
        "a centre past the bottom clamps to it"
    );
}

// The turf (the plan's "Turf" sub-bullet): the game texture's own size,
// eleven horizontal bands dark at both ends over their two greens, the
// grain a fixed noise of luma deviation about 3.5, deterministic.
#[test]
fn the_turf_is_the_game_textures_bands_with_its_grain() {
    let image = turf_image();
    assert_eq!(image.size, TURF_SIZE);
    // A row's or band's mean channel.
    let mean = |rows: &[usize], channel: usize| {
        let (mut sum, mut n) = (0.0f64, 0usize);
        for &y in rows {
            for x in 0..image.width() {
                sum += f64::from(image.pixels[y * image.width() + x][channel]);
                n += 1;
            }
        }
        sum / n as f64
    };
    let rows_of = |band: usize| {
        (0..image.height())
            .filter(|&y| y * TURF_BANDS / image.height() == band)
            .collect::<Vec<usize>>()
    };
    for band in 0..TURF_BANDS {
        let base = if band % 2 == 0 { TURF_DARK } else { TURF_LIGHT };
        for (channel, &base) in base.iter().enumerate() {
            assert!(
                (mean(&rows_of(band), channel) - f64::from(base)).abs() <= 1.0,
                "band {band} channel {channel}"
            );
        }
    }
    // Bands 0 and 10 are dark (both ends), band 5 light.
    for (row, base) in [(0usize, TURF_DARK), (575, TURF_DARK), (288, TURF_LIGHT)] {
        for (channel, &base) in base.iter().enumerate() {
            assert!(
                (mean(&[row], channel) - f64::from(base)).abs() <= 1.0,
                "row {row} channel {channel}"
            );
        }
    }
    // The grain, away from band 1's edges.
    let lumas: Vec<f64> = (53..=103)
        .flat_map(|y| (0..image.width()).map(move |x| (y, x)))
        .map(|(y, x)| f64::from(image.pixels[y * image.width() + x].intensity()) * 255.0)
        .collect();
    let mean_luma = lumas.iter().sum::<f64>() / lumas.len() as f64;
    let std =
        (lumas.iter().map(|l| (l - mean_luma).powi(2)).sum::<f64>() / lumas.len() as f64).sqrt();
    assert!(
        (3.0..=4.0).contains(&std),
        "the grain's luma deviation {std}"
    );
    assert_eq!(turf_image().pixels, image.pixels, "deterministic");
}

// The lines (the plan's "Lines" sub-bullet), read off the painted shapes
// in canvas-relative coordinates: the halfway line, the centre circle,
// the two penalty boxes and goal areas as segments, the two arcs as
// paths; every stroke white at 20%, 2 px wide.
#[test]
fn the_painted_lines_are_the_screens_markings() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);

    let (nodes, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let at = |x: f32, y: f32| canvas.min + egui::vec2(x, y);
    let mut segments: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::LineSegment { points, stroke } if stroke.color == LINES => {
                assert_eq!(stroke.width, 2.0, "every line 2 px");
                Some(egui::Rect::from_two_pos(points[0], points[1]))
            }
            _ => None,
        })
        .collect();
    segments.sort_by(|a, b| {
        (a.min.y, a.min.x, a.max.y, a.max.x)
            .partial_cmp(&(b.min.y, b.min.x, b.max.y, b.max.x))
            .expect("ordered bounds")
    });
    let rect = |a: (f32, f32), b: (f32, f32)| egui::Rect::from_two_pos(at(a.0, a.1), at(b.0, b.1));
    assert_eq!(
        segments,
        [
            rect((80.0, 0.0), (80.0, 66.0)),
            rect((123.0, 0.0), (123.0, 22.0)),
            rect((197.0, 0.0), (197.0, 22.0)),
            rect((240.0, 0.0), (240.0, 66.0)),
            rect((123.0, 22.0), (197.0, 22.0)),
            rect((80.0, 66.0), (240.0, 66.0)),
            rect((0.0, 192.0), (320.0, 192.0)),
            rect((80.0, 318.0), (240.0, 318.0)),
            rect((80.0, 318.0), (80.0, 384.0)),
            rect((240.0, 318.0), (240.0, 384.0)),
            rect((123.0, 362.0), (197.0, 362.0)),
            rect((123.0, 362.0), (123.0, 384.0)),
            rect((197.0, 362.0), (197.0, 384.0)),
        ],
        "the halfway line and the twelve box segments"
    );
    let circles: Vec<(egui::Pos2, f32)> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Circle(shape) if shape.stroke.color == LINES => {
                assert_eq!(shape.stroke.width, 2.0);
                Some((shape.center, shape.radius))
            }
            _ => None,
        })
        .collect();
    assert_eq!(circles, [(at(160.0, 192.0), 36.0)], "the centre circle");
    let mut arcs: Vec<Vec<egui::Pos2>> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Path(shape)
                if matches!(
                    shape.stroke.color,
                    egui::epaint::ColorMode::Solid(color) if color == LINES
                ) =>
            {
                assert_eq!(shape.stroke.width, 2.0);
                assert!(!shape.closed, "the arc is an open path");
                Some(shape.points.clone())
            }
            _ => None,
        })
        .collect();
    arcs.sort_by(|a, b| a[0].y.partial_cmp(&b[0].y).expect("ordered arcs"));
    assert_eq!(arcs.len(), 2, "one arc per end");
    // Each arc: 17 points on the centre circle's radius about the spot
    // 44 from its goal line, only the part beyond the box's far edge.
    for (points, spot, top) in [
        (arcs[0].as_slice(), at(160.0, 44.0), true),
        (arcs[1].as_slice(), at(160.0, 340.0), false),
    ] {
        assert_eq!(points.len(), 17, "the arc's points");
        for point in points {
            assert!(
                (point.distance(spot) - 36.0).abs() <= 0.01,
                "{point:?} on the circle"
            );
            assert!(
                if top {
                    point.y >= spot.y + 22.0 - 0.01
                } else {
                    point.y <= spot.y - 22.0 + 0.01
                },
                "{point:?} beyond the box's far edge"
            );
        }
    }
}

// The markers on the fixture's formation (the plan's "A marker"
// sub-bullet): a 50 x 14 badge of `BADGE_FILL` centred on the slot's
// clamped centre (the keeper's the screen's spot), the position's
// abbreviation in its group's colour, the captain's mark on his marker
// alone, a row's portrait drawn when the host hands one.
#[test]
fn the_markers_paint_badge_position_captain_and_portrait() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let mut roster = roster_rows(&rows);
    let texture = window.ctx.load_texture(
        "a portrait",
        egui::ColorImage::new([4, 4], vec![egui::Color32::RED; 16]),
        egui::TextureOptions::LINEAR,
    );
    let portrait_slot = usize::from(tactics.starting_eleven[4]);
    roster[portrait_slot].portrait = Some(texture.id());
    tactics.set_pieces.captain = tactics.starting_eleven[2];
    let players = tactics.presets[0].formations[0].players;
    let keeper = players
        .iter()
        .position(|slot| slot.position == 0)
        .expect("the fixture has a keeper slot");

    let (nodes, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    assert_eq!(
        centres[keeper],
        canvas.min + egui::vec2(160.0, 358.5),
        "the keeper sits at the screen's keeper spot"
    );
    for (index, slot) in players.iter().enumerate() {
        let badge = egui::Rect::from_center_size(centres[index], egui::vec2(50.0, 14.0));
        let rect = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect == badge => Some(shape),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no badge for marker {index}"));
        assert_eq!(rect.fill, BADGE_FILL, "marker {index}'s badge");
        // The position's abbreviation at the badge's left, in the
        // position's group's colour.
        let label = position_label(slot.position);
        let text = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(shape)
                    if shape.galley.job.text == label
                        && (shape.pos.x - (badge.left() + 3.0)).abs() <= 0.01
                        && (shape.visual_bounding_rect().center().y - badge.center().y).abs()
                            <= 2.0 =>
                {
                    Some(shape)
                }
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no position text for marker {index}"));
        assert_eq!(
            painted_color(text),
            position_colour(slot.position),
            "marker {index}'s {label} in its group's colour"
        );
    }
    // The captain's mark on the captain's marker alone.
    let badge2 = egui::Rect::from_center_size(centres[2], egui::vec2(50.0, 14.0));
    let marks: Vec<egui::Rect> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Rect(shape)
                if shape.fill == egui::Color32::WHITE
                    && shape.rect.size() == egui::vec2(12.0, 11.0) =>
            {
                Some(shape.rect)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        marks,
        [egui::Rect::from_min_max(
            egui::pos2(badge2.left() + 1.0, badge2.top() - 16.0),
            egui::pos2(badge2.left() + 13.0, badge2.top() - 5.0),
        )],
        "the white mark above the captain's badge alone"
    );
    // The portrait: the whole texture over the badge of the row that
    // hands one, no other image but the turf.
    let images: Vec<&egui::epaint::ClippedShape> = shapes
        .iter()
        .filter(|clipped| matches!(clipped.shape, egui::Shape::Mesh(_)))
        .collect();
    assert_eq!(images.len(), 2, "the turf and the one portrait");
    let portrait = images
        .iter()
        .find(|clipped| clipped.shape.texture_id() == texture.id())
        .expect("the portrait's image shape");
    let badge4 = egui::Rect::from_center_size(centres[4], egui::vec2(50.0, 14.0));
    assert_eq!(
        portrait.shape.visual_bounding_rect(),
        egui::Rect::from_min_size(
            egui::pos2(badge4.left(), badge4.top() - 50.0),
            egui::vec2(50.0, 50.0),
        ),
        "the 50 x 50 portrait standing on the badge"
    );
}

// The name below the badge keeps the raw name's colour codes (the plan's
// "the name in its colour codes' colours (white where it has none)").
#[test]
fn a_markers_name_keeps_its_colour_codes_colours() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let mut rows = fixture_rows(PesVersion::Pes21);
    rows[usize::from(tactics.starting_eleven[4])].1 = "\u{11}cff0000ffRED\u{11}d NAME".to_owned();
    let roster = roster_rows(&rows);

    let (_, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let galley = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == "RED NAME" => {
                Some(shape.galley.job.clone())
            }
            _ => None,
        })
        .next()
        .expect("the marker's name galley");
    let colours: Vec<egui::Color32> = galley
        .sections
        .iter()
        .map(|section| section.format.color)
        .collect();
    assert_eq!(
        colours,
        [egui::Color32::from_rgb(255, 0, 0), egui::Color32::WHITE,],
        "the coded run red, the plain run white"
    );
}

// The screen's selection marks as the editor's states (the plan's last
// pitch sub-bullet): the open popover's marker the selected corners, no
// other; every marker the candidates' while an assignment mode is armed;
// a join member's badge `JOIN_FILL` under the armed join mode.
#[test]
fn the_corners_and_the_join_fill_mark_the_states() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;
    let index = 3;

    // The open popover's marker.
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    window.click_pitch(
        marker_point(&nodes, &players, index),
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert_eq!(view.popover.map(|open| open.slot), Some(index));
    let (nodes, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let badge = egui::Rect::from_center_size(centres[index], egui::vec2(50.0, 14.0));
    let portrait_box = egui::Rect::from_min_size(
        egui::pos2(badge.left(), badge.top() - 50.0),
        egui::vec2(50.0, 50.0),
    );
    let corners = |shapes: &[egui::epaint::ClippedShape], colour: egui::Color32| {
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::LineSegment { points, stroke } if stroke.color == colour => {
                    Some(*points)
                }
                _ => None,
            })
            .collect::<Vec<[egui::Pos2; 2]>>()
    };
    let selected = corners(&shapes, SELECTED_CORNERS);
    assert_eq!(selected.len(), 8, "the open marker's four corner arms");
    for points in selected {
        for point in points {
            assert!(
                portrait_box.expand(1.0).contains(point),
                "{point:?} on the marker's portrait box"
            );
        }
    }
    assert_eq!(
        corners(&shapes, CANDIDATE_CORNERS).len(),
        0,
        "no candidate corners with nothing armed"
    );

    // An armed mode: the candidates' corners on every marker.
    view.popover = None;
    view.assign = Some(Assign::Role(5));
    let (_, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    assert_eq!(
        corners(&shapes, CANDIDATE_CORNERS).len(),
        88,
        "the candidates' corners on all eleven"
    );
    assert_eq!(corners(&shapes, SELECTED_CORNERS).len(), 0);

    // Join armed: a member's badge fills JOIN_FILL, a non-member's
    // stays BADGE_FILL.
    view.assign = Some(Assign::Join);
    tactics.players_to_join_attack = [tactics.starting_eleven[3], u8::MAX, u8::MAX];
    let (nodes, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let fill_of = |index: usize| {
        let badge = egui::Rect::from_center_size(centres[index], egui::vec2(50.0, 14.0));
        shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(shape) if shape.rect == badge => Some(shape.fill),
                _ => None,
            })
            .next()
            .unwrap_or_else(|| panic!("no badge for marker {index}"))
    };
    assert_eq!(fill_of(3), JOIN_FILL, "the member's badge");
    assert_eq!(fill_of(4), BADGE_FILL, "a non-member's badge");

    // Nothing armed: no corners at all.
    view.assign = None;
    let (_, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    assert_eq!(corners(&shapes, CANDIDATE_CORNERS).len(), 0);
    assert_eq!(corners(&shapes, SELECTED_CORNERS).len(), 0);
}

// The markers' draw order (the plan's "drawn from the top of the canvas
// down, a press or a drop hits the marker drawn on top"): two slots on
// top of each other, the lower one's badge paints after the upper's and
// a press inside both boxes opens the lower one's popover.
#[test]
fn the_lower_marker_paints_last_and_takes_the_press() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let rows = fixture_rows(PesVersion::Pes21);
    let roster = roster_rows(&rows);
    let (a, b) = (5usize, 3usize);
    // A above (y 20), B below (y 17) on the same x; B's lower index makes
    // an index-order paint put it under A, so only the draw order wins.
    assert!(
        tactics.presets[0].formations[0].players[a].position != 0
            && tactics.presets[0].formations[0].players[b].position != 0,
        "premise: two outfield slots"
    );
    tactics.presets[0].formations[0].players[a].x = 52;
    tactics.presets[0].formations[0].players[a].y = 20;
    tactics.presets[0].formations[0].players[b].x = 52;
    tactics.presets[0].formations[0].players[b].y = 17;
    let players = tactics.presets[0].formations[0].players;

    let (nodes, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let badge_index = |index: usize| {
        let badge = egui::Rect::from_center_size(centres[index], egui::vec2(50.0, 14.0));
        shapes
            .iter()
            .position(
                |clipped| matches!(&clipped.shape, egui::Shape::Rect(shape) if shape.rect == badge),
            )
            .unwrap_or_else(|| panic!("no badge for marker {index}"))
    };
    assert!(
        badge_index(b) > badge_index(a),
        "the lower marker's badge paints after the upper's"
    );
    // A point inside both boxes: A's badge centre, 20 px above B's.
    assert!(
        marker_rect(centres[b], canvas).contains(centres[a]),
        "premise: the press is inside both boxes"
    );
    window.click_pitch(
        centres[a],
        PesVersion::Pes21,
        &mut tactics,
        &mut view,
        &roster,
    );
    assert_eq!(
        view.popover.map(|open| open.slot),
        Some(b),
        "the press hits the marker drawn on top"
    );
}

// `name_scroll` (the plan's measured scroll): a wide name holds still
// `NAME_PERIOD` less its `width + NAME_GAP` loop at `NAME_SPEED`, then
// moves one loop a period, ending where it began.
#[test]
fn name_scroll_rolls_one_loop_a_period() {
    for (seconds, want) in [
        (0.0, 0.0),
        (10.4, 0.0),
        (11.5, 40.0),
        (13.4, 116.0),
        (13.5, 0.0),
        (25.0, 40.0),
    ] {
        assert!(
            (name_scroll(104.0, seconds) - want).abs() <= 1e-3,
            "{seconds} s"
        );
    }
}

// `name_still_for` answers how long until the name next moves: the rest
// of its still phase, 0 while it moves.
#[test]
fn name_still_for_says_when_it_next_moves() {
    for (seconds, want) in [(0.0, 10.5), (11.0, 0.0), (13.5, 10.5)] {
        assert!(
            (name_still_for(104.0, seconds) - want).abs() <= 1e-3,
            "{seconds} s"
        );
    }
}

// The frame's text shapes with the galley `text`, in `pos.x` order.
fn name_galleys<'a>(
    shapes: &'a [egui::epaint::ClippedShape],
    text: &str,
) -> Vec<&'a egui::epaint::TextShape> {
    let mut galleys: Vec<&egui::epaint::TextShape> = shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) if shape.galley.job.text == text => Some(shape),
            _ => None,
        })
        .collect();
    galleys.sort_by(|a, b| a.pos.x.total_cmp(&b.pos.x));
    galleys
}

// A name wider than its badge scrolls as the game's (the plan's "A
// wider name scrolls"): two copies a loop apart, the first `name_scroll`
// left of the badge's left edge, clocked from the pitch's first frame; a
// name that fits stays centred, unmoved.
#[test]
fn a_wide_name_scrolls_two_copies_from_the_pitchs_first_frame() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let mut rows = fixture_rows(PesVersion::Pes21);
    let index = 4;
    rows[usize::from(tactics.starting_eleven[index])].1 = "3DSMAX HAS STOPPED WORKING".to_owned();
    rows[usize::from(tactics.starting_eleven[3])].1 = "SHORT".to_owned();
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;

    // The clock starts at the pitch's first frame, 100.0.
    window.pitch_paint_at(100.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let (nodes, shapes) =
        window.pitch_paint_at(111.5, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let badge_left = centres[index].x - 25.0;
    let galleys = name_galleys(&shapes, "3DSMAX HAS STOPPED WORKING");
    assert_eq!(galleys.len(), 2, "the name and its repeat a loop apart");
    let width = galleys[0].galley.size().x;
    assert!(width > 50.0, "premise: the name is wider than the badge");
    assert!(
        (galleys[0].pos.x - (badge_left - name_scroll(width, 11.5))).abs() <= 0.5,
        "the first copy scrolled left of the badge"
    );
    assert!(
        (galleys[1].pos.x - (galleys[0].pos.x + width + NAME_GAP)).abs() <= 0.5,
        "its repeat a loop to the right"
    );

    // A name that fits stays centred on its badge, unmoved.
    let short_galleys = name_galleys(&shapes, "SHORT");
    assert_eq!(short_galleys.len(), 1, "a fitting name paints once");
    let short = short_galleys[0];
    assert!(
        (short.pos.x - (centres[3].x - short.galley.size().x / 2.0)).abs() <= 0.5
            && (short.pos.y - (centres[3].y + NAME_DY - short.galley.size().y / 2.0)).abs() <= 0.5,
        "the short name still centred under its badge"
    );
}

// The clock restarts when a frame passed without the pitch: a frame at
// 105 without it and the 111.5 frame's offset is the fresh start's 0.
#[test]
fn the_scrolls_clock_restarts_when_a_frame_skipped_the_pitch() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let mut rows = fixture_rows(PesVersion::Pes21);
    let index = 4;
    rows[usize::from(tactics.starting_eleven[index])].1 = "3DSMAX HAS STOPPED WORKING".to_owned();
    let roster = roster_rows(&rows);
    let players = tactics.presets[0].formations[0].players;

    window.pitch_paint_at(100.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    window.frame_at(Some(105.0), vec![], |_ui| {});
    let (nodes, shapes) =
        window.pitch_paint_at(111.5, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let canvas = canvas_rect(&nodes);
    let centres = centres_of(canvas, &players);
    let badge_left = centres[index].x - 25.0;
    let galleys = name_galleys(&shapes, "3DSMAX HAS STOPPED WORKING");
    assert_eq!(galleys.len(), 2, "the name and its repeat a loop apart");
    assert!(
        (galleys[0].pos.x - badge_left).abs() <= 0.5,
        "the restarted clock's offset is 0"
    );
}

// While a shown name is wide the pitch asks a repaint when it next
// moves: its still time while still, at once while it moves; with no
// wide name it asks nothing. (egui forces the context's first frame and
// every frame after an immediate repaint to delay 0, so the still
// measure runs on the third pitch frame and the no-request check on the
// frame after the carry-over drains.)
#[test]
fn the_pitch_repaints_when_the_next_name_moves() {
    let mut window = Headless::sized(600.0, 450.0);
    let mut tactics = fixture_tactics(PesVersion::Pes21);
    let mut view = TacticsView::default();
    let mut rows = fixture_rows(PesVersion::Pes21);
    rows[usize::from(tactics.starting_eleven[4])].1 = "3DSMAX HAS STOPPED WORKING".to_owned();
    let roster = roster_rows(&rows);

    // The clock's first frame, at 100.0; the next drains the carry-over
    // frame-1 repaint egui always makes (the texture and font uploads).
    window.pitch_paint_at(100.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    window.pitch_paint_at(105.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let (_, shapes) =
        window.pitch_paint_at(105.1, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let width = name_galleys(&shapes, "3DSMAX HAS STOPPED WORKING")[0]
        .galley
        .size()
        .x;
    let still = name_still_for(width, 5.1);
    assert!(
        (window.repaint_delay().as_secs_f32() - still).abs() <= 0.05,
        "the still name's repaint in its still time ({} vs {still})",
        window.repaint_delay().as_secs_f32()
    );

    // 12 s in, the name moves: the repaint is at once.
    window.pitch_paint_at(112.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    assert!(
        window.repaint_delay().as_secs_f32() <= 0.05,
        "the moving name's repaint at once ({:?})",
        window.repaint_delay()
    );

    // No wide name (the fixture's own names are all wide too): the
    // pitch requests nothing (the immediate request's carry-over frame
    // drains first).
    let mut rows = fixture_rows(PesVersion::Pes21);
    for slot in tactics.starting_eleven {
        rows[usize::from(slot)].1 = "P".to_owned();
    }
    let roster = roster_rows(&rows);
    window.pitch_paint_at(113.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    window.pitch_paint_at(114.0, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    assert_eq!(
        window.repaint_delay(),
        std::time::Duration::MAX,
        "nothing requested without a wide name"
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

    let (_, shapes) = window.lineup_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
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

    // The popover header's — and the marker's, which draws the same
    // coloured name below the badge (wider than the badge: two copies).
    let (nodes, _) = window.pitch(PesVersion::Pes21, &mut tactics, &mut view, &roster, vec![]);
    let formation = tactics.presets[0].formations[0].clone();
    let at = marker_point(&nodes, &formation.players, 3);
    window.click_pitch(at, PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let (_, shapes) = window.pitch_paint(PesVersion::Pes21, &mut tactics, &mut view, &roster);
    let colors = run_colors(&shapes, "PEP");
    assert_eq!(
        colors.len(),
        3,
        "the marker's two copies (\"PEPSI MAN\" is wide) and the popover header's name"
    );
    for color in colors {
        assert_eq!(
            color,
            egui::Color32::from_rgb(0xc9, 0x00, 0x2b),
            "PEP's run keeps its code's colour"
        );
    }
}
