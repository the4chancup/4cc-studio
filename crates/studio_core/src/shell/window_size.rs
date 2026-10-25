//! The window's opening size and its one-time fit to the monitor (core plan,
//! "Shell layout", "The window's size"): the window is created at
//! `DEFAULT_SIZE`, and the first frame that reports the monitor's size fits and
//! centers it once, a member's own resize or move holding after that.

/// The size the window is created at and opens at on any monitor that holds it:
/// 1280 × 800 points (16:10), where "holds" means within 90% of the monitor's
/// logical width and height, leaving room for the taskbar and the title bar.
pub(super) const DEFAULT_SIZE: egui::Vec2 = egui::vec2(1280.0, 800.0);

/// The size the first frame fits the window to: `DEFAULT_SIZE` while it is
/// within 90% of `monitor`'s width and height, else the largest 16:10 size
/// within 90% of both, each side rounded to whole points.
pub(super) fn fitted_size(monitor: egui::Vec2) -> egui::Vec2 {
    let limit = monitor * 0.9;
    if DEFAULT_SIZE.x <= limit.x && DEFAULT_SIZE.y <= limit.y {
        return DEFAULT_SIZE;
    }
    // A 16:10 window is 1.6 times as wide as it is high; the tighter of the two
    // limits gives the width and the height follows the ratio.
    let width = limit.x.min(limit.y * 1.6);
    egui::vec2(width.round(), (width / 1.6).round())
}

/// The native window's options: created at `DEFAULT_SIZE` and centered, the size the first
/// frame's fit keeps on a monitor that holds it, so nothing jumps there.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(DEFAULT_SIZE),
        centered: true,
        ..Default::default()
    }
}

/// The fit `viewport` calls for, once it reports the monitor's size: the inner size, and
/// the outer position that centers the *outer* rectangle (the decorations, outer minus
/// inner, counted; none when either rect is missing). `None` while the monitor is unknown.
pub(super) fn fit(viewport: &egui::ViewportInfo) -> Option<(egui::Vec2, egui::Pos2)> {
    let monitor = viewport.monitor_size?;
    let size = fitted_size(monitor);
    let decorations = match (viewport.outer_rect, viewport.inner_rect) {
        (Some(outer), Some(inner)) => outer.size() - inner.size(),
        _ => egui::Vec2::ZERO,
    };
    Some((size, ((monitor - size - decorations) * 0.5).to_pos2()))
}

#[cfg(test)]
mod tests {
    use egui::ViewportCommand;

    use super::*;
    use crate::shell::StudioApp;
    use crate::shell::test_harness::*;

    /// One harness frame whose root `ViewportInfo` reports `monitor` and an
    /// outer rect 16 points wider and 39 taller than the inner one (the
    /// scenario's borders and title bar), the window created at `DEFAULT_SIZE`.
    fn monitor_frame(
        ctx: &egui::Context,
        app: &mut StudioApp,
        monitor: Option<egui::Vec2>,
    ) -> FrameOutput {
        let mut input = raw_input(Vec::new(), true, None, None);
        let info = input
            .viewports
            .get_mut(&egui::ViewportId::ROOT)
            .expect("`RawInput::default` carries the root viewport");
        info.monitor_size = monitor;
        info.inner_rect = Some(egui::Rect::from_min_size(egui::Pos2::ZERO, DEFAULT_SIZE));
        info.outer_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            DEFAULT_SIZE + egui::vec2(16.0, 39.0),
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

    #[test]
    fn the_default_size_holds_a_monitor_that_fits_it_and_shrinks_on_a_smaller_one() {
        assert_eq!(fitted_size(egui::vec2(1920.0, 1080.0)), DEFAULT_SIZE);
        assert_eq!(fitted_size(egui::vec2(2000.0, 2000.0)), DEFAULT_SIZE);
        assert_eq!(
            fitted_size(egui::vec2(1280.0, 720.0)),
            egui::vec2(1037.0, 648.0)
        );
        assert_eq!(
            fitted_size(egui::vec2(1366.0, 768.0)),
            egui::vec2(1106.0, 691.0)
        );
        // A square monitor holds the default too. Width-limited: a
        // tall-narrow one — 90% of 1400 is the width, the height follows
        // 16:10.
        assert_eq!(fitted_size(egui::vec2(1500.0, 1500.0)), DEFAULT_SIZE);
        assert_eq!(
            fitted_size(egui::vec2(1400.0, 2000.0)),
            egui::vec2(1260.0, 788.0)
        );
    }

    #[test]
    fn the_window_is_created_at_the_default_size_and_centered() {
        let options = native_options();
        assert_eq!(options.viewport.inner_size, Some(DEFAULT_SIZE));
        assert!(options.centered, "created centered on the monitor");
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
                egui::vec2(1280.0, 720.0),
                egui::vec2(1037.0, 648.0),
                egui::pos2(113.5, 16.5),
            ),
            (
                egui::vec2(1366.0, 768.0),
                egui::vec2(1106.0, 691.0),
                egui::pos2(122.0, 19.0),
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
}
