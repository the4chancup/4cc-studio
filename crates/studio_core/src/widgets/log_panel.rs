//! The log panel (core plan, "Log panel"; the draft `docs/prototypes/shell.html`'s
//! `logPanel`): a `BottomDock` titled `Log` holding a run's lines — the newest
//! `LOG_CAP` of them, colored by severity, filterable, stuck to the bottom until the
//! member scrolls up — each finding's message code a link to its row in the help.
//! The other half of the draft's cross-linking, a line click highlighting its grid
//! cell, waits for the grid itself (worklog 8.10): only the code is clickable here.

use std::cell::Cell;

use super::BottomDock;
use crate::events::{EventLines, MessageCode, PipelineEvent, Severity};

/// The most lines the body draws (the draft's `LOG_CAP`): egui's text rendering is
/// not made for thousands of widgets, so only the newest show; full logs are the
/// tool's own files on disk.
const LOG_CAP: usize = 300;

/// The jump button's margin from the body's bottom-right corner, in points.
const JUMP_MARGIN: f32 = 10.0;

/// The body's scroll area salt (its `scroll_area::State` key for the tests).
const SCROLL_SALT: &str = "log-body";

/// One drawn line: its severity (color and filter), the message code its code part
/// links to the help for, and the text as `EventLines` prints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    /// Color and filter bucket.
    pub severity: Severity,
    /// The finding's code, when the line carries one: drawn as the help link.
    pub code: Option<MessageCode>,
    /// The whole line, monospace, exactly as the console prints it.
    pub text: String,
}

impl LogLine {
    /// The line `event` shows in the log, or `None` for events the log does not
    /// show. A `Message` keeps its severity and code (the help link); any other
    /// shown event is `Info` and codeless. `lines` is the run's formatter: events
    /// it does not show (an `ExportStarted`) still update it.
    pub fn from_event(lines: &mut EventLines, event: &PipelineEvent) -> Option<LogLine> {
        let text = lines.line(event)?;
        let (severity, code) = if let PipelineEvent::Message(message) = event {
            (message.severity, Some(message.code.clone()))
        } else {
            (Severity::Info, None)
        };
        Some(LogLine {
            severity,
            code,
            text,
        })
    }

    /// A line no event carries (the run's exit line).
    pub fn plain(severity: Severity, text: impl Into<String>) -> LogLine {
        LogLine {
            severity,
            code: None,
            text: text.into(),
        }
    }
}

/// The header's filter chips (the draft's `LOG_FILTERS`): which severities the body
/// shows. A filter hides lines; it never drops them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    /// Every line.
    All,
    /// Warning, error and fatal.
    Warnings,
    /// Error and fatal.
    Errors,
}

impl Filter {
    /// `(filter, chip label)` in header order.
    const CHIPS: [(Filter, &'static str); 3] = [
        (Filter::All, "All"),
        (Filter::Warnings, "Warnings"),
        (Filter::Errors, "Errors"),
    ];

    /// Whether a line of `severity` is drawn under this filter.
    fn shows(self, severity: Severity) -> bool {
        match self {
            Filter::All => true,
            Filter::Warnings => severity >= Severity::Warning,
            Filter::Errors => severity >= Severity::Error,
        }
    }
}

/// The body's auto-scroll state (the draft's `logStick`/`logNew`): whether the view
/// follows its newest line, the line count when it left the bottom (the `↓ N new`
/// button's base), the count last seen (a shorter one is a new run's cleared log),
/// and the pending snap the jump button asked for: `stick_to_bottom` only holds a
/// bottom the view never left, it cannot pull back to one.
struct Follow {
    stick: bool,
    left_at: usize,
    seen: usize,
    snap: bool,
}

/// The docked log: the `BottomDock` frame plus the view's own state — the severity
/// filter and the follow state. The lines live with the tool.
pub struct LogPanel {
    dock: BottomDock,
    filter: Filter,
    follow: Follow,
}

impl LogPanel {
    /// A panel titled `Log`, open, showing every line, stuck to the bottom.
    pub fn new() -> LogPanel {
        LogPanel {
            dock: BottomDock::new("log", "Log"),
            filter: Filter::All,
            follow: Follow {
                stick: true,
                left_at: 0,
                seen: 0,
                snap: false,
            },
        }
    }

    /// Draws the panel over `lines` (oldest first) and returns the message code
    /// whose help link was clicked this frame.
    pub fn show(&mut self, ui: &mut egui::Ui, lines: &[LogLine]) -> Option<MessageCode> {
        // A shorter `lines` than last frame is a new run's cleared log: the jump
        // counter starts over.
        if lines.len() < self.follow.seen {
            self.follow.left_at = lines.len();
        }
        self.follow.seen = lines.len();
        // The header's chips and the body both read the filter this frame, but the
        // chips set it: it crosses through a cell so the count and the body see the
        // click's result in the same frame.
        let filter = Cell::new(self.filter);
        let mut clicked = None;
        self.dock.show(
            ui,
            |ui| {
                for (choice, label) in Filter::CHIPS {
                    if ui.selectable_label(filter.get() == choice, label).clicked() {
                        filter.set(choice);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(count_text(filter.get(), lines));
                });
            },
            |ui| body(&mut self.follow, ui, lines, filter.get(), &mut clicked),
        );
        self.filter = filter.get();
        clicked
    }
}

impl Default for LogPanel {
    fn default() -> LogPanel {
        LogPanel::new()
    }
}

/// The body: the capped tail of the filtered lines in a both-ways scroll area (the
/// lines do not wrap — a long one scrolls sideways), and the jump button over its
/// bottom-right while new lines wait unseen.
fn body(
    follow: &mut Follow,
    ui: &mut egui::Ui,
    lines: &[LogLine],
    filter: Filter,
    clicked: &mut Option<MessageCode>,
) {
    let shown = lines
        .iter()
        .filter(|line| filter.shows(line.severity))
        .count();
    let skip = shown.saturating_sub(LOG_CAP);
    let output = egui::ScrollArea::both()
        .id_salt(SCROLL_SALT)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        // The jump button's scroll lands at once; the member's wheel input keeps
        // egui's own smoothing, which is a different path.
        .animated(false)
        .show(ui, |ui| {
            for line in lines
                .iter()
                .filter(|line| filter.shows(line.severity))
                .skip(skip)
            {
                if let Some(code) = show_line(ui, line) {
                    *clicked = Some(code);
                }
            }
            if follow.snap {
                ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
            }
        });
    follow.snap = false;
    let max_offset = (output.content_size.y - output.inner_rect.height()).max(0.0);
    let at_bottom = output.state.offset.y >= max_offset - 1.0;
    if follow.stick && !at_bottom {
        follow.left_at = lines.len();
    }
    follow.stick = at_bottom;
    let new = lines.len().saturating_sub(follow.left_at);
    if !follow.stick && new > 0 {
        let text = format!("↓ {new} new");
        let size = ui
            .fonts_mut(|fonts| {
                fonts.layout_no_wrap(
                    text.clone(),
                    egui::TextStyle::Button.resolve(ui.style()),
                    egui::Color32::PLACEHOLDER,
                )
            })
            .size()
            + 2.0 * ui.spacing().button_padding;
        let corner = output.inner_rect.right_bottom() - egui::vec2(JUMP_MARGIN, JUMP_MARGIN);
        if ui
            .put(
                egui::Rect::from_min_size(corner - size, size),
                egui::Button::new(text),
            )
            .on_hover_text("Jump to the newest line")
            .clicked()
        {
            follow.stick = true;
            follow.left_at = lines.len();
            follow.snap = true;
        }
    }
}

/// The header's count line (the draft's `log-count`): `last 300 of N` when the
/// shown lines overflow the cap, `N of M lines` when a filter hides some, `N lines`
/// when everything shows.
fn count_text(filter: Filter, lines: &[LogLine]) -> String {
    let shown = lines
        .iter()
        .filter(|line| filter.shows(line.severity))
        .count();
    if shown > LOG_CAP {
        format!("last {LOG_CAP} of {shown}")
    } else if shown < lines.len() {
        format!("{shown} of {} lines", lines.len())
    } else {
        format!("{shown} lines")
    }
}

/// A line's color (the draft's `warn`/`err` classes): warning amber, error and
/// fatal red, info the normal text color.
fn line_color(severity: Severity, visuals: &egui::Visuals) -> Option<egui::Color32> {
    match severity {
        Severity::Info => None,
        Severity::Warning => Some(visuals.warn_fg_color),
        Severity::Error | Severity::Fatal => Some(visuals.error_fg_color),
    }
}

/// One line of the body: monospace, severity-colored, one row. A line with a code
/// draws the code as a link to its catalog row in the help (ST-HELP-03), found as
/// the first `"<severity name> <code>"` in the `EventLines` text; the rest of the
/// text is inert while the grid cross-link is unbuilt. Returns the clicked code.
fn show_line(ui: &mut egui::Ui, line: &LogLine) -> Option<MessageCode> {
    let color = line_color(line.severity, ui.visuals());
    let rich = |text: &str| {
        let text = egui::RichText::new(text).monospace();
        match color {
            Some(color) => text.color(color),
            None => text,
        }
    };
    let Some(code) = &line.code else {
        ui.label(rich(&line.text));
        return None;
    };
    let needle = format!("{} {}", line.severity.name(), code.code);
    let Some(start) = line.text.find(&needle) else {
        ui.label(rich(&line.text));
        return None;
    };
    let code_start = start + line.severity.name().len() + 1;
    let code_end = code_start + code.code.len();
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(rich(&line.text[..code_start]));
        if ui
            .link(egui::RichText::new(&line.text[code_start..code_end]).monospace())
            .on_hover_text("Open the help at this message")
            .clicked()
        {
            clicked = Some(code.clone());
        }
        if code_end < line.text.len() {
            ui.label(rich(&line.text[code_end..]));
        }
    });
    clicked
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use egui::accesskit;

    use super::*;
    use crate::events::{Disposition, Message, Scope};

    /// A headless window: one egui context with AccessKit on, run a frame at a time
    /// over an 800 by 600 screen (the shell harness's `frame`, for a bare `Ui`).
    struct Headless {
        ctx: egui::Context,
    }

    impl Headless {
        fn new() -> Headless {
            let ctx = egui::Context::default();
            ctx.enable_accesskit();
            Headless { ctx }
        }

        /// Runs one frame with `events` as its input; returns the frame's AccessKit
        /// nodes.
        fn frame(
            &mut self,
            events: Vec<egui::Event>,
            contents: impl FnMut(&mut egui::Ui),
        ) -> Vec<accesskit::Node> {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events,
                ..Default::default()
            };
            let mut output = self.ctx.run_ui(input, contents);
            let update = output
                .platform_output
                .accesskit_update
                .take()
                .expect("AccessKit is enabled on the context");
            // Nothing renders here, so the frame's texture deltas are dropped on
            // purpose.
            output.drop_without_applying_deltas();
            update.nodes.into_iter().map(|(_, node)| node).collect()
        }

        /// One frame of `panel` over `lines`.
        fn show(
            &mut self,
            panel: &mut LogPanel,
            lines: &[LogLine],
            events: Vec<egui::Event>,
        ) -> Vec<accesskit::Node> {
            self.frame(events, |ui| {
                panel.show(ui, lines);
            })
        }

        /// `show` with no input.
        fn idle(&mut self, panel: &mut LogPanel, lines: &[LogLine]) -> Vec<accesskit::Node> {
            self.show(panel, lines, Vec::new())
        }
    }

    /// The widget labeled `label` (a button's text sits in `label`).
    fn entry<'a>(nodes: &'a [accesskit::Node], label: &str) -> &'a accesskit::Node {
        nodes
            .iter()
            .find(|node| node.label() == Some(label))
            .unwrap_or_else(|| panic!("no widget labeled {label:?}"))
    }

    /// Whether some node carries exactly `text` (a label node's text is its `value`).
    fn shows(nodes: &[accesskit::Node], text: &str) -> bool {
        nodes.iter().any(|node| node.value() == Some(text))
    }

    /// The bounds of the node carrying exactly `text`.
    fn bounds(nodes: &[accesskit::Node], text: &str) -> accesskit::Rect {
        nodes
            .iter()
            .find(|node| node.value() == Some(text))
            .unwrap_or_else(|| panic!("no label with text {text:?}"))
            .bounds()
            .expect("a drawn node has bounds")
    }

    /// The centre of `node`'s bounds, where a click lands.
    fn centre(node: &accesskit::Node) -> egui::Pos2 {
        let bounds = node.bounds().expect("a widget has bounds");
        egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        )
    }

    /// The primary button pressed (or released) at `pos`, the pointer moved there
    /// first.
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

    /// A click on `node`: press frame, release frame, returning the release's nodes —
    /// the frame `clicked()` lands in.
    fn click(
        window: &mut Headless,
        panel: &mut LogPanel,
        lines: &[LogLine],
        node: &accesskit::Node,
    ) -> Vec<accesskit::Node> {
        let pos = centre(node);
        window.show(panel, lines, pointer_at(pos, true));
        window.show(panel, lines, pointer_at(pos, false))
    }

    /// The log body's scroll offset: its `scroll_area::State` under the id the
    /// `id_salt` chain builds — the root ui's top id, the open panel's, the panel
    /// frame's `child` content salt, then the body's `SCROLL_SALT`.
    fn scroll_offset(ctx: &egui::Context, panel: &LogPanel) -> f32 {
        let (open, _) = panel.dock.panel_ids();
        let id = egui::Id::new((egui::ViewportId::ROOT, "__top_ui"))
            .with(egui::IdSalt::new(open))
            .with(egui::IdSalt::new("child"))
            .with(egui::IdSalt::new(SCROLL_SALT));
        egui::scroll_area::State::load(ctx, id)
            .map(|state| state.offset.y)
            .expect("the log body's scroll area")
    }

    /// A dock panel's drawn height (its `PanelState`).
    fn panel_height(ctx: &egui::Context, id: egui::Id) -> f32 {
        egui::PanelState::load(ctx, id)
            .expect("the dock's panel state")
            .outer_rect
            .height()
    }

    fn plain_lines(count: usize) -> Vec<LogLine> {
        (1..=count)
            .map(|n| LogLine::plain(Severity::Info, format!("log line {n}")))
            .collect()
    }

    // ST-LOG-01
    #[test]
    fn only_the_capped_tail_of_the_log_is_drawn() {
        let lines = plain_lines(350);
        let mut panel = LogPanel::new();
        let mut window = Headless::new();
        let nodes = window.idle(&mut panel, &lines);
        assert!(
            !shows(&nodes, "log line 1"),
            "the first line is over the cap"
        );
        assert!(
            !shows(&nodes, "log line 50"),
            "the 301st from the end is too"
        );
        assert!(shows(&nodes, "log line 350"), "the newest is drawn");
        assert!(shows(&nodes, "last 300 of 350"), "the count line");

        // Stuck to the bottom: the two last drawn lines stack a row apart at the
        // body's bottom edge — a line scrolled under the viewport would be
        // window-clipped into a degenerate rect at the screen's edge instead.
        let last = bounds(&nodes, "log line 350");
        let before = bounds(&nodes, "log line 349");
        assert!(
            last.y0 - before.y0 > 5.0,
            "the newest lines stack one row apart: {} vs {}",
            last.y0,
            before.y0
        );
        let (open, _) = panel.dock.panel_ids();
        let bottom = egui::PanelState::load(&window.ctx, open)
            .expect("the open panel's state")
            .outer_rect
            .max
            .y;
        assert!(
            last.y1 > f64::from(bottom) - 30.0,
            "the newest line is at the bottom: {} vs {}",
            last.y1,
            bottom
        );
    }

    // ST-LOG-02
    #[test]
    fn scrolling_up_pauses_the_follow_and_the_jump_button_returns_to_the_bottom() {
        // 200 lines: over the body's ~11-row view but under the cap, so the drawn
        // window cannot slide underneath the assertion.
        let mut lines = plain_lines(200);
        let mut panel = LogPanel::new();
        let mut window = Headless::new();
        window.idle(&mut panel, &lines);

        // Wheel up over the body; egui drains the delta over a few frames.
        let body = egui::pos2(400.0, 500.0);
        window.show(
            &mut panel,
            &lines,
            vec![
                egui::Event::PointerMoved(body),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 200.0),
                    phase: egui::TouchPhase::Move,
                    modifiers: egui::Modifiers::default(),
                },
            ],
        );
        // egui drains the wheel delta over a few frames; let it settle before
        // recording where the view stopped.
        let mut nodes = window.idle(&mut panel, &lines);
        for _ in 0..15 {
            nodes = window.idle(&mut panel, &lines);
        }
        let offset = scroll_offset(&window.ctx, &panel);
        // The topmost visible line (real bounds, not a clipped-offscreen one): the
        // marker the "view does not move" assertion reads.
        let marker = (1..=200)
            .map(|n| format!("log line {n}"))
            .find(|text| {
                nodes
                    .iter()
                    .find(|node| node.value() == Some(text.as_str()))
                    .and_then(|node| node.bounds())
                    .is_some_and(|rect| rect.y0 > 405.0 && rect.y0 < 560.0)
            })
            .expect("a line in view")
            .clone();
        let marker_y = bounds(&nodes, &marker).y0;

        // Three lines arrive: the view does not move and `↓ 3 new` counts them.
        lines.extend((201..=203).map(|n| LogLine::plain(Severity::Info, format!("log line {n}"))));
        let nodes = window.idle(&mut panel, &lines);
        assert!(
            (scroll_offset(&window.ctx, &panel) - offset).abs() < 1.0,
            "arriving lines do not move the view: {} vs {offset}",
            scroll_offset(&window.ctx, &panel)
        );
        assert!(
            (bounds(&nodes, &marker).y0 - marker_y).abs() < 1.0,
            "the first visible line has not moved: {} vs {marker_y}",
            bounds(&nodes, &marker).y0
        );
        let jump = entry(&nodes, "↓ 3 new");
        // The button overlays the body's bottom-right: the body's bottom edge is
        // the button's own bottom edge plus its margin.
        let body_bottom = jump.bounds().expect("the jump button's bounds").y1 + JUMP_MARGIN as f64;

        click(&mut window, &mut panel, &lines, jump);
        // The click set the snap; it lands on the next frame.
        let nodes = window.idle(&mut panel, &lines);
        let last = bounds(&nodes, "log line 203");
        assert!(
            last.y1 >= body_bottom - 2.0,
            "the last line is back at the bottom: {} vs {}",
            last.y1,
            body_bottom
        );
        assert!(
            !nodes.iter().any(|node| node.label() == Some("↓ 3 new")),
            "the button is gone"
        );

        // One more arrives: the view follows it to the bottom again.
        lines.push(LogLine::plain(Severity::Info, "log line 204"));
        let nodes = window.idle(&mut panel, &lines);
        let last = bounds(&nodes, "log line 204");
        assert!(
            last.y1 >= body_bottom - 2.0,
            "the view follows the newest line: {} vs {}",
            last.y1,
            body_bottom
        );
    }

    // ST-LOG-03
    #[test]
    fn the_filter_chips_hide_lines_without_dropping_them() {
        let lines = vec![
            LogLine::plain(Severity::Info, "an info line"),
            LogLine::plain(Severity::Warning, "a warning line"),
            LogLine::plain(Severity::Error, "an error line"),
            LogLine::plain(Severity::Fatal, "a fatal line"),
        ];
        let all = [
            "an info line",
            "a warning line",
            "an error line",
            "a fatal line",
        ];
        let mut panel = LogPanel::new();
        let mut window = Headless::new();
        let nodes = window.idle(&mut panel, &lines);
        for text in all {
            assert!(shows(&nodes, text), "All draws {text}");
        }

        let nodes = click(&mut window, &mut panel, &lines, entry(&nodes, "Warnings"));
        assert!(
            !shows(&nodes, "an info line"),
            "Warnings hides the info line"
        );
        for text in &all[1..] {
            assert!(shows(&nodes, text), "Warnings keeps {text}");
        }
        assert!(
            shows(&nodes, "3 of 4 lines"),
            "the count tells the hidden ones"
        );

        let nodes = click(&mut window, &mut panel, &lines, entry(&nodes, "Errors"));
        for text in &all[..2] {
            assert!(!shows(&nodes, text), "Errors hides {text}");
        }
        for text in &all[2..] {
            assert!(shows(&nodes, text), "Errors keeps {text}");
        }
        assert!(shows(&nodes, "2 of 4 lines"));

        let nodes = click(&mut window, &mut panel, &lines, entry(&nodes, "All"));
        for text in all {
            assert!(shows(&nodes, text), "All draws {text} again");
        }
        assert!(shows(&nodes, "4 lines"));
    }

    // ST-LOG-04
    #[test]
    fn the_chevron_collapses_to_the_header_and_the_open_height_survives() {
        let lines = plain_lines(20);
        let mut panel = LogPanel::new();
        let mut window = Headless::new();
        let (open, collapsed) = panel.dock.panel_ids();
        let nodes = window.idle(&mut panel, &lines);
        let open_height = panel_height(&window.ctx, open);
        assert!(
            (open_height - 220.0).abs() < 1.0,
            "the dock opens at its default height: {open_height}"
        );

        // The chevron collapses to the header row alone; the body is gone on the
        // frame after the click (the panel branch is chosen before the header draws).
        click(&mut window, &mut panel, &lines, entry(&nodes, "▾"));
        let nodes = window.idle(&mut panel, &lines);
        assert!(!shows(&nodes, "log line 20"), "the body is not drawn");
        let header = panel_height(&window.ctx, collapsed);
        assert!(
            header < 60.0 && header > 10.0,
            "collapsed, the panel is its header row: {header}"
        );

        // Expanding restores the stored open height.
        click(&mut window, &mut panel, &lines, entry(&nodes, "▸"));
        let nodes = window.idle(&mut panel, &lines);
        assert!(shows(&nodes, "log line 20"), "the body is drawn again");
        let back = panel_height(&window.ctx, open);
        assert!(
            (back - open_height).abs() < 1.0,
            "the open height survived the collapse: {back} vs {open_height}"
        );

        // Dragging the divider up 80 points grows the panel by about 80, kept across
        // frames. The handle is the open panel's top edge.
        let top = egui::PanelState::load(&window.ctx, open)
            .expect("the open panel's state")
            .outer_rect
            .min
            .y;
        let start = egui::pos2(400.0, top);
        let end = start - egui::vec2(0.0, 80.0);
        window.show(&mut panel, &lines, pointer_at(start, true));
        window.show(&mut panel, &lines, vec![egui::Event::PointerMoved(end)]);
        window.show(&mut panel, &lines, pointer_at(end, false));
        window.idle(&mut panel, &lines);
        let grown = panel_height(&window.ctx, open);
        assert!(
            (grown - (open_height + 80.0)).abs() < 8.0,
            "the divider drag resized the panel: {grown} vs {}",
            open_height + 80.0
        );
        window.idle(&mut panel, &lines);
        window.idle(&mut panel, &lines);
        let kept = panel_height(&window.ctx, open);
        assert_eq!(kept, grown, "the dragged height holds across frames");
    }

    #[test]
    fn from_event_maps_a_message_to_its_severity_and_code() {
        let mut formatter = EventLines::new();
        let code = MessageCode {
            tool_id: "team-compiler",
            code: Cow::Borrowed("player_unlisted"),
        };
        let event = PipelineEvent::Message(Message {
            code: code.clone(),
            severity: Severity::Warning,
            disposition: Disposition::Keep,
            scope: Scope::Run,
            context: Vec::new(),
        });
        assert_eq!(
            LogLine::from_event(&mut formatter, &event),
            Some(LogLine {
                severity: Severity::Warning,
                code: Some(code),
                text: "- Warning player_unlisted".to_owned(),
            })
        );

        // An `Output` event is a codeless Info line of its text.
        let output = PipelineEvent::Output("the report".to_owned());
        assert_eq!(
            LogLine::from_event(&mut formatter, &output),
            Some(LogLine {
                severity: Severity::Info,
                code: None,
                text: "the report".to_owned(),
            })
        );

        // An event the console does not show gives no line.
        assert_eq!(
            LogLine::from_event(
                &mut formatter,
                &PipelineEvent::Progress {
                    processed: 1,
                    total: 2,
                }
            ),
            None
        );
    }
}
