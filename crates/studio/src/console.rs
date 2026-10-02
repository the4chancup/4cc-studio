//! The CLI's event printer: drains the event channel on its own thread while a command runs and
//! prints each event's line, as `studio_core::EventLines` formats it (core plan, "Event system",
//! "CLI translates events"), to stdout.

use std::thread::JoinHandle;

use crossbeam_channel::Receiver;
use studio_core::{EventLines, PipelineEventEnvelope};

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
