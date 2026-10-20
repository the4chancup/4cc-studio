//! The CLI's event and notice printers: each drains its channel on its own thread while a
//! command runs — events' lines go to stdout, as `studio_core::EventLines` formats them (core
//! plan, "Event system", "CLI translates events"), notices' texts to stderr (the status bar's
//! right slot, rendered as lines; `gui.md` "Status bar" "CLI equivalence").

use std::io::{self, Write};
use std::thread::JoinHandle;

use crossbeam_channel::Receiver;
use studio_core::{EventLines, PipelineEventEnvelope, ShellRequest};

#[expect(clippy::print_stdout, reason = "CLI result output is the binary's job")]
fn print_line(line: &str) {
    println!("{line}");
}

/// Drains the channel on its own thread, printing each line to stdout, until every sender is gone.
pub(crate) fn spawn_printer(events: Receiver<PipelineEventEnvelope>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut lines = EventLines::new();
        for envelope in events {
            if let Some(line) = lines.line(&envelope.event) {
                print_line(&line);
            }
        }
    })
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
    use crossbeam_channel::unbounded;
    use studio_core::Notice;

    use super::*;

    #[test]
    fn the_notice_printer_writes_each_notices_text_and_skips_other_requests() {
        let (tx, rx) = unbounded();
        tx.send(ShellRequest::Notify(Notice {
            text: "Switched to PES 2019".to_owned(),
            action: None,
        }))
        .unwrap();
        tx.send(ShellRequest::SettingsChanged).unwrap();
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
