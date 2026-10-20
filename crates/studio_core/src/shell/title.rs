//! The window title (core plan, "Window title"): the one surface a user sees while the window
//! is behind others — the taskbar button, alt-tab. While a tool reports an activity the title
//! carries it (`Team compiler · 12/48 · 01:12 - 4cc Studio`, the active tool's first, then
//! sidebar order); idle with a notice that arrived unfocused it holds the notice's text until
//! focus returns; otherwise it is `4cc Studio`. The shell sends `ViewportCommand::Title` only
//! when the string changes, once per frame at most — the window opens titled `4cc Studio`, so
//! an idle start sends nothing.

/// What the title reads from: the last string sent and the notice text held while unfocused.
pub(super) struct WindowTitle {
    /// The last string sent; starts at the title `run_gui` opens with.
    last_sent: String,
    /// The latest notice's text while it arrived with the window unfocused; cleared when the
    /// window gains focus.
    held_notice: Option<String>,
}

/// The idle title — also the title `run_gui` opens the window with.
const IDLE_TITLE: &str = "4cc Studio";

impl Default for WindowTitle {
    fn default() -> Self {
        WindowTitle {
            last_sent: IDLE_TITLE.to_owned(),
            held_notice: None,
        }
    }
}

impl WindowTitle {
    /// A notice arrived: while the window is unfocused its text is held in the title until
    /// focus returns; while focused the status bar shows it and the title stays idle.
    pub(super) fn notice(&mut self, text: &str, focused: bool) {
        if !focused {
            self.held_notice = Some(text.to_owned());
        }
    }

    /// The title this frame should carry, `Some` only when it differs from the last sent.
    /// `activities` are the activity texts, the active tool's first then sidebar order.
    pub(super) fn frame(&mut self, activities: &[String], focused: bool) -> Option<String> {
        if focused {
            self.held_notice = None;
        }
        // An activity outranks a held notice (the draft's `windowTitle`).
        let title = if !activities.is_empty() {
            format!("{} - 4cc Studio", activities.join(" · "))
        } else if let Some(text) = &self.held_notice {
            format!("{text} - 4cc Studio")
        } else {
            IDLE_TITLE.to_owned()
        };
        if title == self.last_sent {
            return None;
        }
        self.last_sent = title.clone();
        Some(title)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use crossbeam_channel::unbounded;

    use super::*;
    use crate::shell::test_harness::*;

    fn texts(list: &[&str]) -> Vec<String> {
        list.iter().map(|text| text.to_string()).collect()
    }

    #[test]
    fn the_title_changes_only_when_the_string_changes() {
        let mut title = WindowTitle::default();
        // The window opens titled `4cc Studio`: an idle start sends nothing.
        assert_eq!(title.frame(&texts(&[]), true), None);

        let busy = texts(&["Team compiler · 12/48 · 01:12", "B · 00:03"]);
        assert_eq!(
            title.frame(&busy, true),
            Some("Team compiler · 12/48 · 01:12 · B · 00:03 - 4cc Studio".to_owned())
        );
        assert_eq!(title.frame(&busy, true), None, "unchanged sends nothing");
        assert_eq!(title.frame(&texts(&[]), true), Some(IDLE_TITLE.to_owned()));

        // An activity outranks a held notice.
        title.notice("Compiled 48 teams", false);
        assert_eq!(
            title.frame(&busy, false),
            Some("Team compiler · 12/48 · 01:12 · B · 00:03 - 4cc Studio".to_owned())
        );
        assert_eq!(
            title.frame(&texts(&[]), false),
            Some("Compiled 48 teams - 4cc Studio".to_owned())
        );
        assert_eq!(title.frame(&texts(&[]), true), Some(IDLE_TITLE.to_owned()));
        assert_eq!(title.frame(&texts(&[]), true), None);
    }

    #[test]
    fn a_notice_while_focused_is_never_held_in_the_title() {
        let mut title = WindowTitle::default();
        title.notice("Compiled 48 teams", true);
        assert_eq!(title.frame(&texts(&[]), true), None);
        // And the notice does not surface later, when focus leaves.
        assert_eq!(title.frame(&texts(&[]), false), None);
    }

    // ST-TTL-01
    #[test]
    fn the_title_command_carries_the_activity_once_and_idle_when_it_ends() {
        let Shell {
            mut app,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        calls[0].set_activity("Team compiler", Some((12, 48)), Duration::from_secs(72));
        let mut sent = Vec::new();
        for _ in 0..3 {
            sent.extend(titles(&frame(&egui_ctx, &mut app, Vec::new(), true)));
        }
        assert_eq!(
            sent,
            ["Team compiler · 12/48 · 01:12 - 4cc Studio"],
            "one Title command while the string is unchanged"
        );

        calls[0].clear_activity();
        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(titles(&out), ["4cc Studio"], "idle again");
    }

    // ST-TTL-02
    #[test]
    fn a_notice_arriving_unfocused_holds_the_title_until_focus_returns() {
        let Shell {
            mut app, ctx, _dir, ..
        } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        // An idle unfocused start sends nothing.
        let out = frame(&egui_ctx, &mut app, Vec::new(), false);
        assert!(titles(&out).is_empty(), "the idle start sends nothing");

        ctx.notify(crate::status::Notice {
            text: "Compiled 48 teams".to_owned(),
            action: None,
        });
        let out = frame(&egui_ctx, &mut app, Vec::new(), false);
        assert_eq!(
            titles(&out),
            ["Compiled 48 teams - 4cc Studio"],
            "the notice's text, never its action"
        );
        let out = frame(&egui_ctx, &mut app, Vec::new(), false);
        assert!(titles(&out).is_empty(), "held, sent once");

        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(titles(&out), ["4cc Studio"], "focus returns to idle");

        ctx.notify(crate::status::Notice {
            text: "Compiled 48 teams".to_owned(),
            action: None,
        });
        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert!(
            titles(&out).is_empty(),
            "a notice while focused sends nothing"
        );
    }

    // ST-TTL-03
    #[test]
    fn a_runs_last_event_wakes_the_gui_and_its_tick_ends_the_activity() {
        let Shell {
            mut app,
            ctx,
            calls,
            _dir,
            ..
        } = shell();
        let egui_ctx = egui::Context::default();
        egui_ctx.enable_accesskit();

        // The second stub's view is active; the first runs on a thread of its own, with its
        // own event sink the tool drains in `tick`.
        ctx.switch_to_tool("b");
        let (events_tx, events_rx) = unbounded();
        let run_ctx = ctx.with_events(events_tx);
        *calls[0].events.lock().unwrap() = Some(events_rx);
        calls[0].set_activity("A", Some((0, 1)), Duration::from_secs(1));
        let wakes = Arc::new(AtomicUsize::new(0));
        ctx.set_wake({
            let wakes = Arc::clone(&wakes);
            move |_| {
                wakes.fetch_add(1, Ordering::SeqCst);
            }
        });

        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert!(
            titles(&out).contains(&"A · 0/1 · 00:01 - 4cc Studio".to_owned()),
            "the title carries the first stub's run"
        );

        // The run emits its last event on its own thread.
        let run = std::thread::spawn(move || run_ctx.emit(envelope()));
        run.join().unwrap();
        assert!(
            wakes.load(Ordering::SeqCst) > 0,
            "the emit requested a repaint"
        );

        // One frame with no input: `tick` drained the event, the activity ended and the
        // title went idle — all while the first stub's view was never drawn.
        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert!(calls[0].activity.lock().unwrap().is_none());
        assert_eq!(titles(&out), ["4cc Studio"]);
        assert_eq!(calls[0].views(), 0, "the first stub's view was not drawn");
    }
}
