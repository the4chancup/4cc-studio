//! The window title (core plan, "Window title"): the one surface a user sees while the window
//! is behind others — the taskbar button, alt-tab. While a tool reports an activity the title
//! carries it (`Team compiler · 12/48 · 01:12 - Unlimited Tool Works`, `APP_NAME` last; the
//! active tool's first, then sidebar order); idle with a notice that arrived unfocused it holds
//! the notice's text until focus returns; otherwise it is `APP_NAME`. The shell sends `ViewportCommand::Title` only
//! when the string changes, once per frame at most — the window opens titled `APP_NAME`, so
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
const IDLE_TITLE: &str = crate::APP_NAME;

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
            format!("{} - {}", activities.join(" · "), crate::APP_NAME)
        } else if let Some(text) = &self.held_notice {
            format!("{text} - {}", crate::APP_NAME)
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
        // The window opens titled `APP_NAME`: an idle start sends nothing.
        assert_eq!(title.frame(&texts(&[]), true), None);

        let busy = texts(&["Team compiler · 12/48 · 01:12", "B · 00:03"]);
        let busy_title = || {
            format!(
                "Team compiler · 12/48 · 01:12 · B · 00:03 - {}",
                crate::APP_NAME
            )
        };
        assert_eq!(title.frame(&busy, true), Some(busy_title()));
        assert_eq!(title.frame(&busy, true), None, "unchanged sends nothing");
        assert_eq!(title.frame(&texts(&[]), true), Some(IDLE_TITLE.to_owned()));

        // An activity outranks a held notice.
        title.notice("Compiled 48 teams", false);
        assert_eq!(title.frame(&busy, false), Some(busy_title()));
        assert_eq!(
            title.frame(&texts(&[]), false),
            Some(format!("Compiled 48 teams - {}", crate::APP_NAME))
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
        let egui_ctx = test_context();

        calls[0].set_activity("Team compiler", Some((12, 48)), Duration::from_secs(72));
        let mut sent = Vec::new();
        for _ in 0..3 {
            sent.extend(titles(&frame(&egui_ctx, &mut app, Vec::new(), true)));
        }
        assert_eq!(
            sent,
            [format!(
                "Team compiler · 12/48 · 01:12 - {}",
                crate::APP_NAME
            )],
            "one Title command while the string is unchanged"
        );

        calls[0].clear_activity();
        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(titles(&out), [IDLE_TITLE], "idle again");
    }

    // ST-TTL-02
    #[test]
    fn a_notice_arriving_unfocused_holds_the_title_until_focus_returns() {
        let Shell {
            mut app, ctx, _dir, ..
        } = shell();
        let egui_ctx = test_context();

        // An idle unfocused start sends nothing.
        let out = frame(&egui_ctx, &mut app, Vec::new(), false);
        assert!(titles(&out).is_empty(), "the idle start sends nothing");

        ctx.notify(crate::status::Notice {
            text: "Compiled 48 teams".to_owned(),
            kind: crate::status::NoticeKind::Info,
            action: None,
        });
        let out = frame(&egui_ctx, &mut app, Vec::new(), false);
        assert_eq!(
            titles(&out),
            [format!("Compiled 48 teams - {}", crate::APP_NAME)],
            "the notice's text, never its action"
        );
        let out = frame(&egui_ctx, &mut app, Vec::new(), false);
        assert!(titles(&out).is_empty(), "held, sent once");

        let out = frame(&egui_ctx, &mut app, Vec::new(), true);
        assert_eq!(titles(&out), [IDLE_TITLE], "focus returns to idle");

        ctx.notify(crate::status::Notice {
            text: "Compiled 48 teams".to_owned(),
            kind: crate::status::NoticeKind::Info,
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
        let egui_ctx = test_context();

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
            titles(&out).contains(&format!("A · 0/1 · 00:01 - {}", crate::APP_NAME)),
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
        assert_eq!(titles(&out), [IDLE_TITLE]);
        assert_eq!(calls[0].views(), 0, "the first stub's view was not drawn");
    }
}
