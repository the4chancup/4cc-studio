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

/// The preview's icon, Archer on a transparent background in the tight crop on
/// the head at 64 pixels (`resources/icon/small.png`): the window's icon on
/// Windows, where the title bar draws it at 16 pixels.
#[cfg(not(target_arch = "wasm32"))]
const SMALL_ICON_PNG: &[u8] = include_bytes!("../../../../../resources/icon/small.png");

/// The preview's icon, the whole Archer picture at 256 pixels
/// (`resources/icon/window.png`): the window's icon everywhere but Windows,
/// where one image serves every size.
#[cfg(not(target_arch = "wasm32"))]
const WINDOW_ICON_PNG: &[u8] = include_bytes!("../../../../../resources/icon/window.png");

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

/// A monitor of the window's screen: its rectangle and its scale factor, the two
/// units the title-bar check needs one of each for.
#[cfg(not(target_arch = "wasm32"))]
pub(super) struct Monitor {
    /// The monitor's rectangle in the screen's physical pixels.
    rect: egui::Rect,
    /// The monitor's scale factor (native pixels per point), which sizes the
    /// title bar the check builds for it: the window's points stand at this
    /// scale's pixels while it sits on this monitor.
    scale: f32,
}

/// Whether `place`'s title bar (the window's width over the top `TITLE_BAR_PX` pixels at
/// 100%, sized by each monitor's own scale, the pixels the window stands at while it
/// sits there) overlaps that monitor's rectangle (physical pixels) with a positive
/// area; an empty list (a frame with no window to ask) trusts the place.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn on_a_monitor(place: &WindowPlace, monitors: &[Monitor]) -> bool {
    monitors.is_empty()
        || monitors.iter().any(|monitor| {
            let bar = egui::Rect::from_min_size(
                place.position(),
                egui::vec2(place.width, TITLE_BAR_PX) * monitor.scale,
            );
            monitor.rect.intersect(bar).is_positive()
        })
}

/// What the first frame does with the remembered `place` the window was created at:
/// resend its position when its title bar lies on one of `monitors` (the window's monitors,
/// each its rectangle and scale), else fit and center the window as a first start does. An
/// empty list, a frame with no window to ask, trusts the place.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn place_or_fit(place: WindowPlace, monitors: &[Monitor]) -> Opening {
    if on_a_monitor(&place, monitors) {
        Opening::Place(place)
    } else {
        Opening::Fit
    }
}

/// The monitors of `frame`'s window, each its rectangle in the screen's physical pixels
/// and its scale factor; empty when the frame has no window (a test's frame).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn monitors(frame: &eframe::Frame) -> Vec<Monitor> {
    let Some(window) = frame.winit_window() else {
        return Vec::new();
    };
    window
        .available_monitors()
        .map(|monitor| {
            let position = monitor.position();
            let size = monitor.size();
            // Screen coordinates are far inside f32's exact integer range, and a
            // scale factor (an f64 like 1.0, 1.25, 2.0) fits in one too.
            Monitor {
                rect: egui::Rect::from_min_size(
                    egui::pos2(position.x as f32, position.y as f32),
                    egui::vec2(size.width as f32, size.height as f32),
                ),
                scale: monitor.scale_factor() as f32,
            }
        })
        .collect()
}

/// The window's icon. On Windows eframe makes both of the window's icons from
/// this one image: the small one (`ICON_SMALL`, the title bar's, 16 to 24
/// pixels at the usual scales), where only the tight crop reads, and the big
/// one (`ICON_BIG`, the taskbar's). Elsewhere one image serves every size, so
/// the whole picture.
#[cfg(not(target_arch = "wasm32"))]
fn window_icon() -> egui::IconData {
    let png = if cfg!(windows) {
        SMALL_ICON_PNG
    } else {
        WINDOW_ICON_PNG
    };
    eframe::icon_data::from_png_bytes(png)
        .expect("the committed PNG decodes; the tests decode both at their stored sizes")
}

/// The native window's options for `opening`: `Fit` creates the window at `DEFAULT_SIZE` and
/// centered, the size the first frame's fit keeps on a monitor that holds it, so nothing
/// jumps there; `Place` creates it at the record's position (its pixels over its scale, the
/// points winit converts with the scale of the monitor it creates the window on) and inner
/// size, not centered (`NativeOptions`' default). Both carry the `MIN_SIZE` minimum and the
/// preview's icon.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn native_options(opening: &Opening) -> eframe::NativeOptions {
    let viewport = egui::ViewportBuilder::default()
        .with_icon(window_icon())
        .with_min_inner_size(MIN_SIZE);
    match opening {
        Opening::Fit => eframe::NativeOptions {
            viewport: viewport.with_inner_size(DEFAULT_SIZE),
            centered: true,
            ..Default::default()
        },
        Opening::Place(place) => eframe::NativeOptions {
            viewport: viewport
                .with_position(place.position() / place.scale)
                .with_inner_size(place.size()),
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
mod tests;
