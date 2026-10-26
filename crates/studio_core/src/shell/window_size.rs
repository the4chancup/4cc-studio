//! The window's opening size, its one-time fit to the monitor, the place it
//! reopens at and the zoom the views scale with under the design width (core
//! plan, "Shell layout", "The window's size", "Scaling with the width" and
//! "Window size classes"): the window is created at `DEFAULT_SIZE` or at the
//! remembered place, the first frame that reports the monitor's size fits and
//! centers it once, a member's own resize or move holding after that, and a
//! window narrower than `DEFAULT_SIZE.x` draws the whole UI scaled down with
//! the width.

use crate::settings::WindowPlace;

/// The size the window is created at and opens at on any monitor that holds it:
/// 1280 × 800 points, where "holds" means the height within 90% of the
/// monitor's logical height (room for the taskbar and the title bar) and the
/// width, decorations included, within the monitor's logical width.
pub(super) const DEFAULT_SIZE: egui::Vec2 = egui::vec2(1280.0, 800.0);

/// The smallest window a member's resize can reach: 960 × 600 points, under
/// the smallest default the fit produces (1264 × 648) so the fit never meets
/// it. The views are laid out for the defaults; below about 900 points wide,
/// rows with fixed parts (the Tactics pitch, a swatch row, the tab strip)
/// clip at the right edge instead of wrapping (measured 2026-10-25).
#[cfg(not(target_arch = "wasm32"))]
pub(super) const MIN_SIZE: egui::Vec2 = egui::vec2(960.0, 600.0);

/// The size the first frame fits the window to: the height is `DEFAULT_SIZE.y`
/// while it is within 90% of `monitor`'s height, else 90% of it rounded to
/// whole points; the width is `DEFAULT_SIZE.x` while the window with its
/// `decorations` fits `monitor`'s width, else what is left over them.
pub(super) fn fitted_size(monitor: egui::Vec2, decorations: egui::Vec2) -> egui::Vec2 {
    let height = if DEFAULT_SIZE.y <= monitor.y * 0.9 {
        DEFAULT_SIZE.y
    } else {
        (monitor.y * 0.9).round()
    };
    let width = if DEFAULT_SIZE.x + decorations.x <= monitor.x {
        DEFAULT_SIZE.x
    } else {
        monitor.x - decorations.x
    };
    egui::vec2(width, height)
}

/// The zoom factor for a window `window_width` points wide at zoom 1: the
/// design width the views are laid out for is `DEFAULT_SIZE.x`, so narrower
/// scales the whole UI down with the width and at or over it the zoom is 1
/// and the views reflow.
pub(super) fn width_zoom(window_width: f32) -> f32 {
    (window_width / DEFAULT_SIZE.x).min(1.0)
}

/// How the window opens: fitted and centered on the primary monitor (no remembered place, or
/// one on no monitor), or created at the main class's remembered place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Opening {
    Fit,
    /// The remembered place is a native concept: no place is ever remembered
    /// or checked on the browser build.
    #[cfg(not(target_arch = "wasm32"))]
    Place(WindowPlace),
}

/// The height of a title bar at 100% scale, in pixels: what "the title bar lies on a
/// monitor" measures (32 on Windows 10 and 11).
#[cfg(not(target_arch = "wasm32"))]
const TITLE_BAR_PX: f32 = 32.0;

/// The window's place a frame reports, in the record's units, or `None` on a frame that
/// reports no normal place: maximized or fullscreen (a minimized window reports no rects at
/// all), a rect or the native scale missing, or the viewport rects built at another zoom than
/// the pass's: on the frame where a queued zoom lands, egui rescales `screen_rect` alone at
/// `begin_pass`, so `inner_rect` and `screen_rect` differ in size there by a pixel or more
/// and agree on every other frame to a rounding error. Pixels are the outer rect's points
/// times the native scale times the zoom; points at zoom 1 are the screen rect's size times
/// the zoom.
pub(super) fn place_of(
    viewport: &egui::ViewportInfo,
    screen_rect: Option<egui::Rect>,
    zoom: f32,
) -> Option<WindowPlace> {
    if viewport.maximized == Some(true) || viewport.fullscreen == Some(true) {
        return None;
    }
    let outer = viewport.outer_rect?;
    let inner = viewport.inner_rect?;
    let native = viewport.native_pixels_per_point?;
    let screen = screen_rect?;
    // The two sizes come from different divisions (egui-winit divides the
    // inner rect's corners, `screen_rect` its size), so on a consistent frame
    // they agree to a rounding error, while on the landing frame they differ
    // by the pixel or more whose change moved the zoom: compared in pixels,
    // against half of one.
    if ((inner.size() - screen.size()) * native * zoom)
        .abs()
        .max_elem()
        >= 0.5
    {
        return None;
    }
    Some(WindowPlace {
        x: (outer.min.x * native * zoom).round(),
        y: (outer.min.y * native * zoom).round(),
        width: screen.width() * zoom,
        height: screen.height() * zoom,
        scale: native,
    })
}

/// Whether `place`'s title bar (the window's width over the top `TITLE_BAR_PX` pixels at
/// 100%, both scaled by the record's scale) overlaps one of `monitors` (physical pixels) with
/// a positive area; an empty list (a frame with no window to ask) trusts the place.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn on_a_monitor(place: &WindowPlace, monitors: &[egui::Rect]) -> bool {
    let bar = egui::Rect::from_min_size(
        place.position(),
        egui::vec2(place.width, TITLE_BAR_PX) * place.scale,
    );
    monitors.is_empty()
        || monitors
            .iter()
            .any(|monitor| monitor.intersect(bar).is_positive())
}

/// What the first frame does with the remembered `place` the window was created at:
/// resend its position when its title bar lies on one of `monitors` (the window's monitors,
/// in physical pixels), else fit and center the window as a first start does. An empty
/// list, a frame with no window to ask, trusts the place.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn place_or_fit(place: WindowPlace, monitors: &[egui::Rect]) -> Opening {
    if on_a_monitor(&place, monitors) {
        Opening::Place(place)
    } else {
        Opening::Fit
    }
}

/// The monitors of `frame`'s window, each its rectangle in the screen's physical pixels;
/// empty when the frame has no window (a test's frame).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn monitors(frame: &eframe::Frame) -> Vec<egui::Rect> {
    let Some(window) = frame.winit_window() else {
        return Vec::new();
    };
    window
        .available_monitors()
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            // Screen coordinates are far inside f32's exact integer range.
            egui::Rect::from_min_size(
                egui::pos2(position.x as f32, position.y as f32),
                egui::vec2(size.width as f32, size.height as f32),
            )
        })
        .collect()
}

/// The native window's options for `opening`: `Fit` creates the window at `DEFAULT_SIZE` and
/// centered, the size the first frame's fit keeps on a monitor that holds it, so nothing
/// jumps there; `Place` creates it at the record's position (its pixels over its scale, the
/// points winit converts with the scale of the monitor it creates the window on) and inner
/// size, not centered. Both carry the `MIN_SIZE` minimum.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn native_options(opening: &Opening) -> eframe::NativeOptions {
    match opening {
        Opening::Fit => eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(DEFAULT_SIZE)
                .with_min_inner_size(MIN_SIZE),
            centered: true,
            ..Default::default()
        },
        Opening::Place(place) => eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_position(place.position() / place.scale)
                .with_inner_size(place.size())
                .with_min_inner_size(MIN_SIZE),
            centered: false,
            ..Default::default()
        },
    }
}

/// The fit `viewport` calls for, once it reports the monitor's size: the inner size, and
/// the outer position that centers the *outer* rectangle (the decorations, outer minus
/// inner, counted; none when either rect is missing). `None` while the monitor is unknown.
pub(super) fn fit(viewport: &egui::ViewportInfo) -> Option<(egui::Vec2, egui::Pos2)> {
    let monitor = viewport.monitor_size?;
    let decorations = match (viewport.outer_rect, viewport.inner_rect) {
        (Some(outer), Some(inner)) => outer.size() - inner.size(),
        _ => egui::Vec2::ZERO,
    };
    let size = fitted_size(monitor, decorations);
    Some((size, ((monitor - size - decorations) * 0.5).to_pos2()))
}

#[cfg(test)]
mod tests {
    use eframe::App;
    use egui::ViewportCommand;

    use super::*;
    use crate::settings::{SETTINGS_FILE_NAME, Settings, WindowPlace};
    use crate::shell::StudioApp;
    use crate::shell::test_harness::*;

    /// The borders and title bar `monitor_frame`'s outer rect adds over the
    /// inner one: the scenario's 16 points wide and 39 tall.
    const DECORATIONS: egui::Vec2 = egui::vec2(16.0, 39.0);

    /// One harness frame whose root `ViewportInfo` reports `monitor` and an
    /// outer rect `DECORATIONS` bigger than the inner one, the window created
    /// at `DEFAULT_SIZE`.
    fn monitor_frame(
        ctx: &egui::Context,
        app: &mut StudioApp,
        monitor: Option<egui::Vec2>,
    ) -> FrameOutput {
        let mut input = raw_input(Vec::new(), true, None, None);
        input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, DEFAULT_SIZE));
        let info = input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("`RawInput::default` carries the root viewport");
        info.monitor_size = monitor;
        info.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, DEFAULT_SIZE));
        info.outer_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            DEFAULT_SIZE + DECORATIONS,
        ));
        run_frame(ctx, app, input)
    }

    /// The frame's fit commands: size and position, the title's `Title` not.
    fn fitting(output: &FrameOutput) -> Vec<ViewportCommand> {
        output
            .commands
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    ViewportCommand::InnerSize(_) | ViewportCommand::OuterPosition(_)
                )
            })
            .cloned()
            .collect()
    }

    /// One harness frame as egui-winit builds it for a window `size_px`
    /// native pixels big on a monitor at `native_ppp`: `inner_rect` and
    /// `screen_rect` are the pixels over the pixels-per-point of the
    /// context's current zoom (`update_viewport_info`, `take_egui_input`),
    /// `native_pixels_per_point` the monitor's raw scale factor.
    fn window_frame(
        ctx: &egui::Context,
        app: &mut StudioApp,
        size_px: egui::Vec2,
        native_ppp: f32,
    ) -> FrameOutput {
        let mut input = raw_input(Vec::new(), true, None, None);
        let size_pt = size_px / (ctx.zoom_factor() * native_ppp);
        input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size_pt));
        let info = input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("`RawInput::default` carries the root viewport");
        info.native_pixels_per_point = Some(native_ppp);
        info.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size_pt));
        run_frame(ctx, app, input)
    }

    /// One harness frame for a window whose outer top-left sits at `outer_min`
    /// native pixels, `size_px` native pixels big on a monitor at `native`,
    /// `maximized` reported or not: the rects are pixel rects divided by the
    /// pixels per point of the context's current zoom, as egui-winit builds
    /// them, and `DECORATIONS` and the (8, 31) inset are pixels here.
    fn placed_frame(
        ctx: &egui::Context,
        app: &mut StudioApp,
        outer_min: egui::Pos2,
        size_px: egui::Vec2,
        native: f32,
        maximized: bool,
    ) -> FrameOutput {
        let mut input = raw_input(Vec::new(), true, None, None);
        let ppp = ctx.zoom_factor() * native;
        input.screen_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size_px / ppp));
        let info = input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("`RawInput::default` carries the root viewport");
        info.native_pixels_per_point = Some(native);
        info.outer_rect = Some(egui::Rect::from_min_size(outer_min, size_px + DECORATIONS) / ppp);
        info.inner_rect =
            Some(egui::Rect::from_min_size(outer_min + egui::vec2(8.0, 31.0), size_px) / ppp);
        info.maximized = Some(maximized);
        run_frame(ctx, app, input)
    }

    #[test]
    fn place_of_reads_pixels_points_and_scale_from_a_normal_frame() {
        let mut viewport = egui::ViewportInfo {
            outer_rect: Some(egui::Rect::from_min_size(
                egui::pos2(100.0, 50.0),
                egui::vec2(1116.0, 739.0),
            )),
            inner_rect: Some(egui::Rect::from_min_size(
                egui::pos2(108.0, 81.0),
                egui::vec2(1100.0, 700.0),
            )),
            native_pixels_per_point: Some(1.0),
            ..Default::default()
        };
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100.0, 700.0));
        assert_eq!(
            <[f32; 5]>::from(place_of(&viewport, Some(screen), 1.0).unwrap()),
            [100.0, 50.0, 1100.0, 700.0, 1.0],
        );

        // The screen's pixels are whole: a fractional position is rounded.
        viewport.outer_rect = Some(egui::Rect::from_min_size(
            egui::pos2(10.51, 5.34),
            egui::vec2(1116.0, 739.0),
        ));
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            egui::pos2(18.51, 36.34),
            egui::vec2(1100.0, 700.0),
        ));
        assert_eq!(
            <[f32; 5]>::from(place_of(&viewport, Some(screen), 1.0).unwrap()),
            [11.0, 5.0, 1100.0, 700.0, 1.0],
        );

        // The same window at zoom 0.5: the rects are in the zoomed points and
        // the record comes out identical.
        viewport.outer_rect = Some(egui::Rect::from_min_size(
            egui::pos2(200.0, 100.0),
            egui::vec2(2232.0, 1478.0),
        ));
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            egui::pos2(216.0, 162.0),
            egui::vec2(2200.0, 1400.0),
        ));
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(2200.0, 1400.0));
        assert_eq!(
            <[f32; 5]>::from(place_of(&viewport, Some(screen), 0.5).unwrap()),
            [100.0, 50.0, 1100.0, 700.0, 1.0],
        );

        // A 200% display: the points are half the pixels, the scale is kept.
        viewport.native_pixels_per_point = Some(2.0);
        viewport.outer_rect = Some(egui::Rect::from_min_size(
            egui::pos2(50.0, 25.0),
            egui::vec2(566.0, 389.0),
        ));
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            egui::pos2(58.0, 56.0),
            egui::vec2(550.0, 350.0),
        ));
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(550.0, 350.0));
        assert_eq!(
            <[f32; 5]>::from(place_of(&viewport, Some(screen), 1.0).unwrap()),
            [100.0, 50.0, 550.0, 350.0, 2.0],
        );

        // A 150% display, its rects built as egui-winit builds them, a pixel
        // rect divided by the pixels per point: the inner size comes out an ULP
        // away from the screen rect's, and the frame still records.
        viewport.native_pixels_per_point = Some(1.5);
        viewport.outer_rect = Some(
            egui::Rect::from_min_size(egui::pos2(2992.0, 50.0), egui::vec2(1116.0, 739.0)) / 1.5,
        );
        let inner =
            egui::Rect::from_min_size(egui::pos2(3000.0, 81.0), egui::vec2(1100.0, 700.0)) / 1.5;
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100.0, 700.0) / 1.5);
        viewport.inner_rect = Some(inner);
        assert_ne!(inner.size(), screen.size(), "the premise: an ULP apart");
        assert_eq!(
            place_of(&viewport, Some(screen), 1.0).map(<[f32; 5]>::from),
            Some([2992.0, 50.0, 1100.0 / 1.5, 700.0 / 1.5, 1.5]),
        );

        // The frames that report no normal place: maximized or fullscreen, and
        // a rect, the scale or the screen rect missing.
        viewport.native_pixels_per_point = Some(1.0);
        viewport.outer_rect = Some(egui::Rect::from_min_size(
            egui::pos2(100.0, 50.0),
            egui::vec2(1116.0, 739.0),
        ));
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            egui::pos2(108.0, 81.0),
            egui::vec2(1100.0, 700.0),
        ));
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1100.0, 700.0));
        viewport.maximized = Some(true);
        assert_eq!(place_of(&viewport, Some(screen), 1.0), None);
        viewport.maximized = None;
        viewport.fullscreen = Some(true);
        assert_eq!(place_of(&viewport, Some(screen), 1.0), None);
        viewport.fullscreen = None;
        viewport.outer_rect = None;
        assert_eq!(place_of(&viewport, Some(screen), 1.0), None);
        viewport.outer_rect = Some(egui::Rect::from_min_size(
            egui::pos2(100.0, 50.0),
            egui::vec2(1116.0, 739.0),
        ));
        viewport.inner_rect = None;
        assert_eq!(place_of(&viewport, Some(screen), 1.0), None);
        viewport.inner_rect = Some(egui::Rect::from_min_size(
            egui::pos2(108.0, 81.0),
            egui::vec2(1100.0, 700.0),
        ));
        viewport.native_pixels_per_point = None;
        assert_eq!(place_of(&viewport, Some(screen), 1.0), None);
        viewport.native_pixels_per_point = Some(1.0);
        assert_eq!(place_of(&viewport, None, 1.0), None);

        // The zoom-landing frame: egui rescales `screen_rect` alone at
        // `begin_pass`, so the two rects differ in size there.
        let landing = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1375.0, 875.0));
        assert_eq!(place_of(&viewport, Some(landing), 1.0), None);
    }

    #[test]
    fn the_title_bar_check_overlaps_a_monitor_with_a_positive_area() {
        let monitor = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1920.0, 1080.0));
        let place = |x: f32, y: f32| WindowPlace {
            x,
            y,
            width: 1100.0,
            height: 700.0,
            scale: 1.0,
        };
        assert!(on_a_monitor(&place(100.0, 50.0), &[monitor]));
        assert!(!on_a_monitor(&place(2500.0, 50.0), &[monitor]));
        let monitors = [
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(3840.0, 2160.0)),
            egui::Rect::from_min_size(egui::pos2(-1920.0, 328.0), egui::vec2(1920.0, 1080.0)),
        ];
        assert!(on_a_monitor(&place(-1800.0, 400.0), &monitors));
        // The bar's left 120 pixels over the bottom 10 still count.
        assert!(on_a_monitor(&place(1800.0, 1070.0), &[monitor]));
        // A bar starting on the monitor's right edge does not.
        assert!(!on_a_monitor(&place(1920.0, 50.0), &[monitor]));
        // The bar's lower 12 pixels reach into the monitor from above.
        assert!(on_a_monitor(&place(100.0, -20.0), &[monitor]));
        // The bar ends 8 pixels above the monitor, although the window below it
        // reaches in.
        assert!(!on_a_monitor(&place(100.0, -40.0), &[monitor]));
        // At 200% the 1100-point bar is 2200 pixels: it reaches x = 100.
        let scaled = WindowPlace {
            x: -2100.0,
            y: 50.0,
            width: 1100.0,
            height: 700.0,
            scale: 2.0,
        };
        assert!(on_a_monitor(&scaled, &[monitor]));
        assert!(
            on_a_monitor(&place(2500.0, 50.0), &[]),
            "an empty list trusts the place"
        );

        assert_eq!(
            place_or_fit(place(100.0, 50.0), &[monitor]),
            Opening::Place(place(100.0, 50.0)),
        );
        assert_eq!(place_or_fit(place(2500.0, 50.0), &[monitor]), Opening::Fit);
        assert_eq!(
            place_or_fit(place(2500.0, 50.0), &[]),
            Opening::Place(place(2500.0, 50.0)),
        );
    }

    #[test]
    fn the_default_size_holds_a_monitor_that_fits_it_and_fits_each_side_on_a_smaller_one() {
        assert_eq!(
            fitted_size(egui::vec2(1920.0, 1080.0), DECORATIONS),
            DEFAULT_SIZE
        );
        assert_eq!(
            fitted_size(egui::vec2(2000.0, 2000.0), DECORATIONS),
            DEFAULT_SIZE
        );
        // A square monitor holds the default too. Height-fitted on a shorter
        // monitor: the width keeps 1280 wherever the decorated window fits.
        assert_eq!(
            fitted_size(egui::vec2(1500.0, 1500.0), DECORATIONS),
            DEFAULT_SIZE
        );
        assert_eq!(
            fitted_size(egui::vec2(1366.0, 768.0), DECORATIONS),
            egui::vec2(1280.0, 691.0)
        );
        // A 1080p monitor at 125% scaling: 864 points tall, so 800 is within
        // the monitor yet past 90% of it.
        assert_eq!(
            fitted_size(egui::vec2(1536.0, 864.0), DECORATIONS),
            egui::vec2(1280.0, 778.0)
        );
        assert_eq!(
            fitted_size(egui::vec2(1280.0, 720.0), DECORATIONS),
            egui::vec2(1264.0, 648.0)
        );
        // Width-limited: the window with its 16-point borders is wider than
        // the monitor, so the width is what is left over them.
        assert_eq!(
            fitted_size(egui::vec2(1200.0, 2000.0), DECORATIONS),
            egui::vec2(1184.0, 800.0)
        );
    }

    #[test]
    fn the_zoom_is_the_width_over_the_design_width_and_one_at_or_over_it() {
        assert_eq!(width_zoom(960.0), 0.75);
        assert_eq!(width_zoom(1024.0), 0.8);
        assert_eq!(width_zoom(1280.0), 1.0);
        assert_eq!(width_zoom(1600.0), 1.0);
    }

    #[test]
    fn the_window_is_created_at_the_default_size_and_centered() {
        let options = native_options(&Opening::Fit);
        assert_eq!(options.viewport.inner_size, Some(DEFAULT_SIZE));
        assert!(options.centered, "created centered on the monitor");
    }

    // ST-WIN-01 (its last clause): the options carry a minimum inner size,
    // under the smallest default the fit produces so the fit never meets it.
    #[test]
    fn the_options_stop_a_members_resize_at_the_minimum() {
        let options = native_options(&Opening::Fit);
        assert_eq!(options.viewport.min_inner_size, Some(MIN_SIZE));
        let place = WindowPlace {
            x: 100.0,
            y: 50.0,
            width: 1100.0,
            height: 700.0,
            scale: 1.0,
        };
        let options = native_options(&Opening::Place(place));
        assert_eq!(options.viewport.min_inner_size, Some(MIN_SIZE));
        let smallest = fitted_size(egui::vec2(1280.0, 720.0), DECORATIONS);
        assert!(
            MIN_SIZE.x <= smallest.x && MIN_SIZE.y <= smallest.y,
            "the minimum {MIN_SIZE:?} fits inside the smallest default {smallest:?}"
        );
    }

    // ST-WIN-01
    #[test]
    fn the_first_frame_with_a_monitor_size_fits_and_centers_the_window_once() {
        let egui_ctx = test_context();

        for (monitor, size, pos) in [
            (
                egui::vec2(1920.0, 1080.0),
                egui::vec2(1280.0, 800.0),
                egui::pos2(312.0, 120.5),
            ),
            (
                egui::vec2(1366.0, 768.0),
                egui::vec2(1280.0, 691.0),
                egui::pos2(35.0, 19.0),
            ),
            (
                egui::vec2(1280.0, 720.0),
                egui::vec2(1264.0, 648.0),
                egui::pos2(0.0, 16.5),
            ),
        ] {
            let Shell { mut app, _dir, .. } = shell();
            let out = monitor_frame(&egui_ctx, &mut app, Some(monitor));
            assert_eq!(
                fitting(&out),
                [
                    ViewportCommand::InnerSize(size),
                    ViewportCommand::OuterPosition(pos),
                ],
                "monitor {monitor:?}: size then outer-rect centering"
            );
            // No later frame sends either command: a member's resize holds.
            let out = monitor_frame(&egui_ctx, &mut app, Some(monitor));
            assert!(
                fitting(&out).is_empty(),
                "monitor {monitor:?}: the fit is sent once"
            );
        }

        // A frame reporting no monitor size sends nothing and leaves the fit
        // pending; the first frame that reports one sends both.
        let Shell { mut app, _dir, .. } = shell();
        let out = monitor_frame(&egui_ctx, &mut app, None);
        assert!(fitting(&out).is_empty(), "no monitor size: no commands");
        let out = monitor_frame(&egui_ctx, &mut app, Some(egui::vec2(1920.0, 1080.0)));
        assert_eq!(
            fitting(&out),
            [
                ViewportCommand::InnerSize(egui::vec2(1280.0, 800.0)),
                ViewportCommand::OuterPosition(egui::pos2(312.0, 120.5)),
            ],
            "the fit waited for a frame reporting the monitor"
        );
    }

    // ST-WIN-04
    #[test]
    fn the_place_is_written_at_exit_and_the_next_start_opens_there() {
        let egui_ctx = test_context();
        let Shell { mut app, _dir, .. } = shell();
        let dir = _dir.as_ref().unwrap().path().to_path_buf();

        // Three frames at the place: the second is the zoom-landing one (the
        // window is narrower than the design width, so the first frame queues
        // a zoom), the third records.
        for _ in 0..3 {
            placed_frame(
                &egui_ctx,
                &mut app,
                egui::pos2(100.0, 50.0),
                egui::vec2(1100.0, 700.0),
                1.0,
                false,
            );
        }
        app.on_exit();

        let loaded = Settings::load(&dir.join(SETTINGS_FILE_NAME)).unwrap();
        assert_eq!(
            loaded.common.window_main,
            Some(WindowPlace {
                x: 100.0,
                y: 50.0,
                width: 1100.0,
                height: 700.0,
                scale: 1.0,
            }),
        );

        // The restart: a new shell over the loaded settings in the same folder,
        // on a new context as a new run's.
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell { mut app, .. } = shell_with(
            loaded,
            vec![a, b],
            &[],
            Some(dir.clone()),
            vec![calls_a, calls_b],
        );
        let options = app.opening();
        assert_eq!(options.viewport.position, Some(egui::pos2(100.0, 50.0)));
        assert_eq!(options.viewport.inner_size, Some(egui::vec2(1100.0, 700.0)));
        assert!(!options.centered, "created where it closed, not centered");
        app.monitors = |_| {
            vec![egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920.0, 1080.0),
            )]
        };
        let restart_ctx = test_context();
        let out = placed_frame(
            &restart_ctx,
            &mut app,
            egui::pos2(100.0, 50.0),
            egui::vec2(1100.0, 700.0),
            1.0,
            false,
        );
        assert_eq!(
            fitting(&out),
            [ViewportCommand::OuterPosition(egui::pos2(100.0, 50.0))],
            "the position again in the frame's points, and no size"
        );
        let out = placed_frame(
            &restart_ctx,
            &mut app,
            egui::pos2(100.0, 50.0),
            egui::vec2(1100.0, 700.0),
            1.0,
            false,
        );
        assert!(
            fitting(&out).is_empty(),
            "the position is sent on the first frame only"
        );
    }

    // ST-WIN-04
    #[test]
    fn a_place_on_no_monitor_opens_fitted_and_an_empty_list_trusts_it() {
        let egui_ctx = test_context();
        let dir = TestDir::new("studio_core_window_place_offscreen");
        let mut settings = test_settings(dir.path());
        settings.common.window_main = Some(WindowPlace {
            x: 2500.0,
            y: 50.0,
            width: 1100.0,
            height: 700.0,
            scale: 1.0,
        });
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell { mut app, .. } = shell_with(
            settings.clone(),
            vec![a, b],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a, calls_b],
        );

        // The window is created at the place, unchecked: the check needs the
        // window's own monitor list, which exists only once the window does.
        let options = app.opening();
        assert_eq!(options.viewport.position, Some(egui::pos2(2500.0, 50.0)));
        assert_eq!(options.viewport.inner_size, Some(egui::vec2(1100.0, 700.0)));
        assert!(!options.centered);

        // (2500, 50) puts the title bar past the 1920 monitor's right edge:
        // the first frame drops the place for the fit of ST-WIN-01's first row.
        app.monitors = |_| {
            vec![egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1920.0, 1080.0),
            )]
        };
        let out = monitor_frame(&egui_ctx, &mut app, Some(egui::vec2(1920.0, 1080.0)));
        assert_eq!(
            fitting(&out),
            [
                ViewportCommand::InnerSize(egui::vec2(1280.0, 800.0)),
                ViewportCommand::OuterPosition(egui::pos2(312.0, 120.5)),
            ],
        );

        // No list at all, a frame with no window to ask: the place is trusted.
        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell { mut app, .. } = shell_with(
            settings,
            vec![a, b],
            &[],
            Some(dir.path().to_path_buf()),
            vec![calls_a, calls_b],
        );
        app.opening();
        let out = placed_frame(
            &egui_ctx,
            &mut app,
            egui::pos2(2500.0, 50.0),
            egui::vec2(1100.0, 700.0),
            1.0,
            false,
        );
        assert_eq!(
            fitting(&out),
            [ViewportCommand::OuterPosition(egui::pos2(2500.0, 50.0))],
        );
    }

    // ST-WIN-04
    #[test]
    fn a_window_on_a_200_percent_display_records_its_points_and_reopens_in_them() {
        let egui_ctx = test_context();
        let Shell { mut app, _dir, .. } = shell();
        let dir = _dir.as_ref().unwrap().path().to_path_buf();

        // The same 1100 × 700 window at (100, 50) on a 200% display: the
        // record holds the points and the scale.
        for _ in 0..3 {
            placed_frame(
                &egui_ctx,
                &mut app,
                egui::pos2(100.0, 50.0),
                egui::vec2(1100.0, 700.0),
                2.0,
                false,
            );
        }
        app.on_exit();

        let loaded = Settings::load(&dir.join(SETTINGS_FILE_NAME)).unwrap();
        assert_eq!(
            loaded.common.window_main,
            Some(WindowPlace {
                x: 100.0,
                y: 50.0,
                width: 550.0,
                height: 350.0,
                scale: 2.0,
            }),
        );

        let (a, calls_a) = stub("a", "A", None);
        let (b, calls_b) = stub("b", "B", None);
        let Shell { mut app, .. } = shell_with(
            loaded,
            vec![a, b],
            &[],
            Some(dir.clone()),
            vec![calls_a, calls_b],
        );
        let options = app.opening();
        assert_eq!(options.viewport.position, Some(egui::pos2(50.0, 25.0)));
        assert_eq!(options.viewport.inner_size, Some(egui::vec2(550.0, 350.0)));
        assert!(!options.centered);
        let restart_ctx = test_context();
        let out = placed_frame(
            &restart_ctx,
            &mut app,
            egui::pos2(100.0, 50.0),
            egui::vec2(1100.0, 700.0),
            2.0,
            false,
        );
        assert_eq!(
            fitting(&out),
            [ViewportCommand::OuterPosition(egui::pos2(50.0, 25.0))],
            "the position in the frame's own points"
        );
    }

    // ST-WIN-04
    #[test]
    fn a_window_maximized_at_close_keeps_the_place_before_the_maximize() {
        let egui_ctx = test_context();
        let Shell { mut app, _dir, .. } = shell();
        let dir = _dir.as_ref().unwrap().path().to_path_buf();

        for _ in 0..3 {
            placed_frame(
                &egui_ctx,
                &mut app,
                egui::pos2(100.0, 50.0),
                egui::vec2(1100.0, 700.0),
                1.0,
                false,
            );
        }
        // A maximized window reports no normal place: the last one stands.
        for _ in 0..2 {
            placed_frame(
                &egui_ctx,
                &mut app,
                egui::Pos2::ZERO,
                egui::vec2(1920.0, 1041.0),
                1.0,
                true,
            );
        }
        app.on_exit();

        let loaded = Settings::load(&dir.join(SETTINGS_FILE_NAME)).unwrap();
        assert_eq!(
            loaded.common.window_main,
            Some(WindowPlace {
                x: 100.0,
                y: 50.0,
                width: 1100.0,
                height: 700.0,
                scale: 1.0,
            }),
        );
    }

    // ST-WIN-02
    #[test]
    fn a_window_narrower_than_the_design_width_scales_the_ui_with_the_width() {
        // A 960 × 700 window at 100%: 960 of the 1280-point design width.
        let egui_ctx = test_context();
        let Shell { mut app, _dir, .. } = shell();
        window_frame(&egui_ctx, &mut app, egui::vec2(960.0, 700.0), 1.0);
        window_frame(&egui_ctx, &mut app, egui::vec2(960.0, 700.0), 1.0);
        assert_eq!(egui_ctx.zoom_factor(), 0.75);
        assert_eq!(egui_ctx.content_rect().width(), 1280.0);
        // The zoom holds over later frames, and a resize of the height alone
        // changes nothing.
        for size_px in [
            egui::vec2(960.0, 700.0),
            egui::vec2(960.0, 600.0),
            egui::vec2(960.0, 600.0),
        ] {
            window_frame(&egui_ctx, &mut app, size_px, 1.0);
            assert_eq!(egui_ctx.zoom_factor(), 0.75);
            assert_eq!(egui_ctx.content_rect().width(), 1280.0);
        }

        // At and over the design width the zoom is 1 and the view reflows.
        for width_px in [1280.0, 1600.0] {
            let egui_ctx = test_context();
            let Shell { mut app, _dir, .. } = shell();
            window_frame(&egui_ctx, &mut app, egui::vec2(width_px, 700.0), 1.0);
            window_frame(&egui_ctx, &mut app, egui::vec2(width_px, 700.0), 1.0);
            assert_eq!(egui_ctx.zoom_factor(), 1.0);
            assert_eq!(egui_ctx.content_rect().width(), width_px);
        }

        // At 150% display scaling 1440 native pixels are the same 960 points.
        let egui_ctx = test_context();
        let Shell { mut app, _dir, .. } = shell();
        window_frame(&egui_ctx, &mut app, egui::vec2(1440.0, 1050.0), 1.5);
        window_frame(&egui_ctx, &mut app, egui::vec2(1440.0, 1050.0), 1.5);
        assert_eq!(egui_ctx.zoom_factor(), 0.75);
        assert_eq!(egui_ctx.content_rect().width(), 1280.0);
    }

    // The zoom factor is the shell's alone: egui's keyboard zoom is off
    // (gui.md "Scaling with the width"), so Ctrl with plus changes nothing.
    #[test]
    fn a_ctrl_plus_leaves_the_zoom_alone() {
        let egui_ctx = test_context();
        let Shell { mut app, _dir, .. } = shell();
        let ctrl_plus = egui::Event::Key {
            key: egui::Key::Plus,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        };
        frame(&egui_ctx, &mut app, vec![ctrl_plus], true);
        // A keyboard zoom is queued for the next pass, which is where the
        // factor would move.
        frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(egui_ctx.zoom_factor(), 1.0);
    }
}
