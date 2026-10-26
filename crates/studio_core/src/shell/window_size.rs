//! The window's opening size, its one-time fit to the monitor and the zoom the
//! views scale with under the design width (core plan, "Shell layout", "The
//! window's size" and "Scaling with the width"): the window is created at
//! `DEFAULT_SIZE`, the first frame that reports the monitor's size fits and
//! centers it once, a member's own resize or move holding after that, and a
//! window narrower than `DEFAULT_SIZE.x` draws the whole UI scaled down with
//! the width.

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

/// The native window's options: created at `DEFAULT_SIZE` and centered, the size the first
/// frame's fit keeps on a monitor that holds it, so nothing jumps there.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(DEFAULT_SIZE)
            .with_min_inner_size(MIN_SIZE),
        centered: true,
        ..Default::default()
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
    use egui::ViewportCommand;

    use super::*;
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
        let options = native_options();
        assert_eq!(options.viewport.inner_size, Some(DEFAULT_SIZE));
        assert!(options.centered, "created centered on the monitor");
    }

    // ST-WIN-01 (its last clause): the options carry a minimum inner size,
    // under the smallest default the fit produces so the fit never meets it.
    #[test]
    fn the_options_stop_a_members_resize_at_the_minimum() {
        let options = native_options();
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
