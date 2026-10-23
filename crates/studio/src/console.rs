//! The CLI's event and notice printers: each drains its channel on its own thread while a
//! command runs — events' lines go to stdout, as `studio_core::EventLines` formats them (core
//! plan, "Event system", "CLI translates events"), notices' texts to stderr (the status bar's
//! right slot, rendered as lines; `gui.md` "Status bar" "CLI equivalence"). On a terminal
//! stderr the event printer also draws the run's progress line and leaves its summary (core
//! plan, "The console's progress line and run summary"); piped, it prints stdout alone.

use std::io::{self, IsTerminal, Write};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::Receiver;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressState, ProgressStyle};
use studio_core::{EventLines, PipelineEvent, PipelineEventEnvelope, ShellRequest};

/// Drains the channel on its own thread, printing each line to stdout, until every sender is
/// gone. Whether stderr is a terminal decides whether the progress line is drawn; it is read
/// here alone, so `print_events` takes the decision and its writers as arguments.
pub(crate) fn spawn_printer(events: Receiver<PipelineEventEnvelope>) -> JoinHandle<io::Result<()>> {
    let draw = io::stderr().is_terminal();
    std::thread::spawn(move || {
        // The unlocked handles lock per write, as the notice printer's do.
        print_events(events, draw, &mut io::stdout(), &mut io::stderr())
    })
}

/// The run's events as lines on `out`, as `EventLines` formats them. With `draw`, the run's
/// progress line (`[ 3/12 exports | 1,284/4,010 parts | 0:42 ]`) is drawn by indicatif on the
/// process's stderr, redrawn in place at its rate and cleared while each line goes to `out`
/// (`suspend`); on `Complete` it is replaced on `err` by the run's summary, and a stream that
/// ends without one (a `check`, an upgrade, a run stopped before it compiled) leaves no line.
/// Without `draw` the bar is hidden and nothing touches `err`, so piped stdout is
/// byte-for-byte what it is on a terminal.
fn print_events(
    events: Receiver<PipelineEventEnvelope>,
    draw: bool,
    out: &mut impl Write,
    err: &mut impl Write,
) -> io::Result<()> {
    let mut lines = EventLines::new();
    let mut counts = RunCounts::new();
    let mut ended = false;
    // indicatif draws to stderr itself (the Windows console handled), capped at ten draws a
    // second once its target is set; the line is the counters message and the `{clock}` key,
    // no bar graphic.
    let bar = if draw {
        // Hidden until the first export: a bar redraws after every `suspend`, so it would
        // draw an empty frame while the findings before it print.
        let bar = ProgressBar::with_draw_target(None, ProgressDrawTarget::hidden());
        bar.set_style(
            ProgressStyle::with_template("[ {msg} | {clock} ]")
                .expect("a literal template parses")
                .with_key(
                    "clock",
                    |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                        // Rendered from the bar's state on every draw, so the clock moves
                        // between events too.
                        write!(w, "{}", elapsed_text(state.elapsed()))
                            .expect("writing into the draw buffer does not fail");
                    },
                ),
        );
        bar
    } else {
        ProgressBar::hidden()
    };
    for envelope in events {
        counts.observe(&envelope.event);
        // The line draws from the first export started, not the first finding; its message
        // is set before the bar has a target, so its first frame already holds the counters.
        if let Some(line) = counts.line() {
            bar.set_message(line);
        }
        let summary = match &envelope.event {
            PipelineEvent::ExportStarted { .. } => {
                if draw && counts.exports_total == 1 {
                    bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
                    // The steady tick makes the clock tick between findings, from its own
                    // thread: enabled once the message is up, so no empty brackets draw.
                    bar.enable_steady_tick(Duration::from_millis(500));
                }
                None
            }
            PipelineEvent::Complete {
                files_written,
                skipped_exports,
                skipped_folders,
                folders_with_errors,
                ..
            } => Some((
                *files_written,
                *skipped_exports,
                *skipped_folders,
                *folders_with_errors,
            )),
            PipelineEvent::ExportProcessed { .. }
            | PipelineEvent::FolderStatus { .. }
            | PipelineEvent::Message(_)
            | PipelineEvent::Output(_)
            | PipelineEvent::Progress { .. }
            | PipelineEvent::UnmigratedContentFound => None,
        };
        match summary {
            Some((files, skipped_exports, skipped_folders, folders_with_errors)) => {
                ended = true;
                bar.finish_and_clear();
                if draw {
                    writeln!(
                        err,
                        "{}",
                        summary_line(
                            files,
                            skipped_exports,
                            skipped_folders,
                            folders_with_errors,
                            bar.elapsed()
                        )
                    )?;
                }
            }
            None => {
                if let Some(line) = lines.line(&envelope.event) {
                    bar.suspend(|| writeln!(out, "{line}"))?;
                }
            }
        }
    }
    if !ended {
        bar.finish_and_clear();
    }
    Ok(())
}

/// The run's counters the progress line draws: the exports started (the total) and
/// processed, and the parts the `Progress` events reported. Counting lives here, away from
/// the bar's real stderr, so a test can read the line's text.
struct RunCounts {
    exports_done: usize,
    exports_total: usize,
    /// The last `Progress` pair (`None` until one arrives): the manifest's tasks done of all.
    parts: Option<(usize, usize)>,
}

impl RunCounts {
    fn new() -> RunCounts {
        RunCounts {
            exports_done: 0,
            exports_total: 0,
            parts: None,
        }
    }

    /// One event's effect on the counters.
    fn observe(&mut self, event: &PipelineEvent) {
        match event {
            PipelineEvent::ExportStarted { .. } => self.exports_total += 1,
            PipelineEvent::ExportProcessed { .. } => self.exports_done += 1,
            PipelineEvent::Progress { processed, total } => {
                self.parts = Some((*processed, *total));
            }
            PipelineEvent::FolderStatus { .. }
            | PipelineEvent::Message(_)
            | PipelineEvent::Output(_)
            | PipelineEvent::Complete { .. }
            | PipelineEvent::UnmigratedContentFound => {}
        }
    }

    /// The progress line's counters text, `None` before the first `ExportStarted`.
    fn line(&self) -> Option<String> {
        (self.exports_total > 0)
            .then(|| progress_line(self.exports_done, self.exports_total, self.parts))
    }
}

/// The progress line's counters, which `set_message` puts inside the template's `[ ... ]`:
/// the exports done of all started, then the parts done once a `Progress` arrived.
fn progress_line(
    exports_done: usize,
    exports_total: usize,
    parts: Option<(usize, usize)>,
) -> String {
    let parts = match parts {
        Some((done, total)) => format!(" | {}/{} parts", grouped(done), grouped(total)),
        None => String::new(),
    };
    format!(
        "{}/{} exports{parts}",
        grouped(exports_done),
        grouped(exports_total)
    )
}

/// The summary line a `Complete` leaves: the files written, the exports skipped, the
/// folders skipped and the folders with errors, each a count of distinct exports or folders
/// with its plural right, and the run's elapsed `m:ss`.
fn summary_line(
    files: usize,
    skipped_exports: usize,
    skipped_folders: usize,
    folders_with_errors: usize,
    elapsed: Duration,
) -> String {
    format!(
        "Done: {} file{} written, {} export{} skipped, {} folder{} skipped, {} folder{} with errors, {}",
        grouped(files),
        plural_s(files),
        grouped(skipped_exports),
        plural_s(skipped_exports),
        grouped(skipped_folders),
        plural_s(skipped_folders),
        grouped(folders_with_errors),
        plural_s(folders_with_errors),
        elapsed_text(elapsed)
    )
}

/// `n`'s digits grouped by threes with commas (`1,284`).
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut text = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            text.push(',');
        }
        text.push(digit);
    }
    text
}

/// `elapsed` as `m:ss` (`0:42`, `62:07`).
fn elapsed_text(elapsed: Duration) -> String {
    let seconds = elapsed.as_secs();
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// The plural `s`, absent for a count of one.
fn plural_s(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// Drains the shell-request channel, printing each notice's text to `out` on its own line,
/// until every sender is gone. Other requests are the shell's alone and are skipped; the
/// action a notice may carry cannot be clicked in a terminal.
fn print_notices(requests: Receiver<ShellRequest>, out: &mut impl Write) -> io::Result<()> {
    for request in requests {
        if let ShellRequest::Notify(notice) = request {
            writeln!(out, "{}", notice.text)?;
        }
    }
    Ok(())
}

/// Runs `print_notices` on stderr on its own thread; the join carries the writes' outcome.
pub(crate) fn spawn_notice_printer(requests: Receiver<ShellRequest>) -> JoinHandle<io::Result<()>> {
    std::thread::spawn(move || {
        // The unlocked handle locks per write: holding `stderr().lock()` for the command's
        // whole run would block every other thread's stderr write, the logger's included, and
        // hang the command at its first warning.
        print_notices(requests, &mut std::io::stderr())
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crossbeam_channel::unbounded;
    use studio_core::{
        Disposition, ExportId, Message, MessageCode, Notice, RunId, Scope, Severity,
    };

    use super::*;

    /// The `export_id`-less envelope every run's events share.
    fn envelope(event: PipelineEvent) -> PipelineEventEnvelope {
        PipelineEventEnvelope {
            run_id: RunId(1),
            export_id: None,
            export_revision: None,
            event,
        }
    }

    /// One run's stream: two exports started, a finding, three parts, both exports done,
    /// the run's end.
    fn run_events() -> Receiver<PipelineEventEnvelope> {
        let (tx, rx) = unbounded();
        let finding = PipelineEvent::Message(Message {
            code: MessageCode {
                tool_id: "team-compiler",
                code: Cow::Borrowed("bad_file"),
            },
            severity: Severity::Warning,
            disposition: Disposition::Keep,
            scope: Scope::Export {
                export_id: ExportId(1),
            },
            context: Vec::new(),
        });
        for event in [
            PipelineEvent::ExportStarted {
                export_id: ExportId(1),
                display_name: "co One".to_owned(),
            },
            PipelineEvent::ExportStarted {
                export_id: ExportId(2),
                display_name: "co Two".to_owned(),
            },
            finding,
            PipelineEvent::Progress {
                processed: 1,
                total: 3,
            },
            PipelineEvent::Progress {
                processed: 2,
                total: 3,
            },
            PipelineEvent::Progress {
                processed: 3,
                total: 3,
            },
            PipelineEvent::ExportProcessed {
                export_id: ExportId(1),
            },
            PipelineEvent::ExportProcessed {
                export_id: ExportId(2),
            },
            PipelineEvent::Complete {
                success: false,
                files_written: 12,
                skipped_exports: 1,
                skipped_folders: 2,
                duplicates: 0,
                folders_with_errors: 3,
            },
        ] {
            tx.send(envelope(event)).unwrap();
        }
        drop(tx);
        rx
    }

    #[test]
    fn the_lines_print_the_same_stdout_whether_the_progress_line_draws_or_not() {
        let mut out_no_draw = Vec::new();
        let mut err_no_draw = Vec::new();
        print_events(run_events(), false, &mut out_no_draw, &mut err_no_draw).unwrap();
        assert_eq!(out_no_draw, b"- co One: Warning bad_file\n");
        assert_eq!(err_no_draw, b"", "nothing is drawn on a piped stderr");

        let mut out_draw = Vec::new();
        let mut err_draw = Vec::new();
        print_events(run_events(), true, &mut out_draw, &mut err_draw).unwrap();
        assert_eq!(out_draw, out_no_draw, "the line changes no stdout byte");
        let err = String::from_utf8(err_draw).unwrap();
        assert!(
            err.starts_with(
                "Done: 12 files written, 1 export skipped, 2 folders skipped, 3 folders with errors, 0:"
            ),
            "the summary is all a drawn run leaves on stderr: {err}"
        );
    }

    #[test]
    fn the_progress_line_is_the_counters_alone_the_template_adding_brackets_and_clock() {
        assert_eq!(
            progress_line(3, 12, Some((1284, 4010))),
            "3/12 exports | 1,284/4,010 parts"
        );
        assert_eq!(progress_line(0, 1, None), "0/1 exports");
    }

    #[test]
    fn the_summary_line_counts_the_run_with_each_plural_right() {
        assert_eq!(
            summary_line(1284, 2, 2, 1, Duration::from_secs(42)),
            "Done: 1,284 files written, 2 exports skipped, 2 folders skipped, 1 folder with errors, 0:42"
        );
        assert_eq!(
            summary_line(1, 1, 1, 0, Duration::from_secs(5)),
            "Done: 1 file written, 1 export skipped, 1 folder skipped, 0 folders with errors, 0:05"
        );
        assert_eq!(
            summary_line(0, 0, 0, 3, Duration::from_secs(62 * 60 + 7)),
            "Done: 0 files written, 0 exports skipped, 0 folders skipped, 3 folders with errors, 62:07"
        );
    }

    #[test]
    fn the_progress_counters_start_empty_and_count_exports_and_parts() {
        let mut counts = RunCounts::new();
        assert_eq!(counts.line(), None, "no line before the first export");
        let finding = run_events()
            .iter()
            .map(|envelope| envelope.event)
            .find(|event| matches!(event, PipelineEvent::Message(_)))
            .unwrap();
        counts.observe(&finding);
        assert_eq!(counts.line(), None, "a finding is no export either");
        counts.observe(&PipelineEvent::ExportStarted {
            export_id: ExportId(1),
            display_name: "co One".to_owned(),
        });
        counts.observe(&PipelineEvent::ExportStarted {
            export_id: ExportId(2),
            display_name: "co Two".to_owned(),
        });
        assert_eq!(counts.line().as_deref(), Some("0/2 exports"));
        counts.observe(&PipelineEvent::ExportProcessed {
            export_id: ExportId(1),
        });
        counts.observe(&PipelineEvent::Progress {
            processed: 3,
            total: 6,
        });
        assert_eq!(counts.line().as_deref(), Some("1/2 exports | 3/6 parts"));
    }

    #[test]
    fn the_progress_counters_never_count_more_exports_done_than_started() {
        let mut counts = RunCounts::new();
        for envelope in run_events() {
            counts.observe(&envelope.event);
        }
        assert!(
            counts.exports_done <= counts.exports_total,
            "{} of {}",
            counts.exports_done,
            counts.exports_total
        );
        assert_eq!(counts.line().as_deref(), Some("2/2 exports | 3/3 parts"));
    }

    #[test]
    fn grouped_groups_digits_by_threes() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(284), "284");
        assert_eq!(grouped(284_010), "284,010");
    }

    #[test]
    fn the_notice_printer_writes_each_notices_text_and_skips_other_requests() {
        let (tx, rx) = unbounded();
        tx.send(ShellRequest::Notify(Notice {
            text: "Switched to PES 2019".to_owned(),
            action: None,
        }))
        .unwrap();
        tx.send(ShellRequest::SettingsChanged).unwrap();
        tx.send(ShellRequest::Close).unwrap();
        tx.send(ShellRequest::Notify(Notice {
            text: "Compiled 48 teams".to_owned(),
            action: None,
        }))
        .unwrap();
        drop(tx);
        let mut out = Vec::new();
        print_notices(rx, &mut out).unwrap();
        assert_eq!(out, b"Switched to PES 2019\nCompiled 48 teams\n");
    }
}
