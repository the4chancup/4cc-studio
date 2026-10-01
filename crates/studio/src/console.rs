//! The CLI's event printer: turns pipeline events into the console's `-` lines (core plan,
//! "Event system", "CLI translates events"). One self-contained line per finding, so lines from
//! exports processed in parallel never need a header to be read.

use std::collections::HashMap;
use std::thread::JoinHandle;

use crossbeam_channel::Receiver;
use studio_core::{ExportId, Message, PipelineEvent, PipelineEventEnvelope, Scope, Severity};

/// Turns pipeline events into console lines; remembers each export's source name from its
/// `ExportStarted` event.
pub(crate) struct ConsolePrinter {
    names: HashMap<ExportId, String>,
}

impl ConsolePrinter {
    pub(crate) fn new() -> ConsolePrinter {
        ConsolePrinter {
            names: HashMap::new(),
        }
    }

    /// The line for one event, or `None` for events the console does not show.
    pub(crate) fn line(&mut self, event: &PipelineEvent) -> Option<String> {
        match event {
            PipelineEvent::ExportStarted {
                export_id,
                display_name,
            } => {
                self.names.insert(*export_id, display_name.clone());
                None
            }
            PipelineEvent::Message(message) => Some(self.message_line(message)),
            PipelineEvent::ExportProcessed { .. }
            | PipelineEvent::FolderStatus { .. }
            | PipelineEvent::Progress { .. }
            | PipelineEvent::Complete { .. }
            | PipelineEvent::UnmigratedContentFound => None,
        }
    }

    fn message_line(&self, message: &Message) -> String {
        let source = match message.scope.export_id() {
            None => String::new(),
            Some(export_id) => match self.names.get(&export_id) {
                Some(name) => format!("{name}: "),
                None => format!("export {}: ", export_id.0),
            },
        };
        let location = match &message.scope {
            Scope::Run | Scope::Export { .. } => String::new(),
            Scope::Folder { path, .. } | Scope::File { path, .. } => {
                format!(" at {}", path.as_str())
            }
            Scope::RosterEntry { file, line, .. } => format!(" at {} line {line}", file.as_str()),
        };
        let context = if message.context.is_empty() {
            String::new()
        } else {
            let fields: Vec<String> = message
                .context
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect();
            format!(" ({})", fields.join(", "))
        };
        format!(
            "- {source}{} {}{location}{context}",
            severity_name(message.severity),
            message.code.code
        )
    }
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "Info",
        Severity::Warning => "Warning",
        Severity::Error => "Error",
        Severity::Fatal => "Fatal",
    }
}

#[expect(clippy::print_stdout, reason = "CLI result output is the binary's job")]
fn print_line(line: &str) {
    println!("{line}");
}

/// Drains the channel on its own thread, printing each line to stdout, until every sender is gone.
pub(crate) fn spawn_printer(events: Receiver<PipelineEventEnvelope>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut printer = ConsolePrinter::new();
        for envelope in events {
            if let Some(line) = printer.line(&envelope.event) {
                print_line(&line);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use studio_core::{Disposition, FolderStatus, MessageCode};
    use vtree::ScopePath;

    use super::*;

    fn message(severity: Severity, code: &'static str, scope: Scope) -> PipelineEvent {
        PipelineEvent::Message(Message {
            code: MessageCode {
                tool_id: "team-compiler",
                code: Cow::Borrowed(code),
            },
            severity,
            disposition: Disposition::Keep,
            scope,
            context: Vec::new(),
        })
    }

    fn started(id: u64, name: &str) -> PipelineEvent {
        PipelineEvent::ExportStarted {
            export_id: ExportId(id),
            display_name: name.to_owned(),
        }
    }

    fn path(text: &str) -> ScopePath {
        ScopePath::new(text).unwrap()
    }

    /// A printer that has seen export 1 start as `co - Spring.zip`.
    fn printer() -> ConsolePrinter {
        let mut printer = ConsolePrinter::new();
        assert_eq!(printer.line(&started(1, "co - Spring.zip")), None);
        printer
    }

    #[test]
    fn a_run_scoped_message_has_no_source_and_no_location() {
        let event = message(Severity::Error, "multiple_ref_exports", Scope::Run);
        assert_eq!(
            printer().line(&event).unwrap(),
            "- Error multiple_ref_exports"
        );
    }

    #[test]
    fn an_export_scoped_message_names_its_source() {
        let event = message(
            Severity::Info,
            "export_identified",
            Scope::Export {
                export_id: ExportId(1),
            },
        );
        assert_eq!(
            printer().line(&event).unwrap(),
            "- co - Spring.zip: Info export_identified"
        );
    }

    #[test]
    fn a_folder_scoped_message_shows_the_folder_path() {
        let event = message(
            Severity::Warning,
            "player_unlisted",
            Scope::Folder {
                export_id: ExportId(1),
                path: path("Players/15 - B"),
            },
        );
        assert_eq!(
            printer().line(&event).unwrap(),
            "- co - Spring.zip: Warning player_unlisted at Players/15 - B"
        );
    }

    #[test]
    fn a_file_scoped_message_shows_the_file_path() {
        let event = message(
            Severity::Fatal,
            "file_unreadable",
            Scope::File {
                export_id: ExportId(1),
                path: path("Players/15 - B/face.dds"),
            },
        );
        assert_eq!(
            printer().line(&event).unwrap(),
            "- co - Spring.zip: Fatal file_unreadable at Players/15 - B/face.dds"
        );
    }

    #[test]
    fn a_roster_entry_message_shows_file_and_line() {
        let event = message(
            Severity::Error,
            "players_txt_line_invalid",
            Scope::RosterEntry {
                export_id: ExportId(1),
                file: path("players.txt"),
                line: 1,
                slot: None,
            },
        );
        assert_eq!(
            printer().line(&event).unwrap(),
            "- co - Spring.zip: Error players_txt_line_invalid at players.txt line 1"
        );
    }

    #[test]
    fn context_fields_follow_in_parentheses_in_the_given_order() {
        let PipelineEvent::Message(mut message) =
            message(Severity::Error, "multiple_ref_exports", Scope::Run)
        else {
            unreachable!("the helper builds a Message");
        };
        message.context = vec![
            ("exports".to_owned(), "refs a, refs b".to_owned()),
            ("count".to_owned(), "2".to_owned()),
        ];
        assert_eq!(
            printer().line(&PipelineEvent::Message(message)).unwrap(),
            "- Error multiple_ref_exports (exports=refs a, refs b, count=2)"
        );
    }

    #[test]
    fn an_export_that_never_started_is_named_by_its_number() {
        let event = message(
            Severity::Warning,
            "player_unlisted",
            Scope::Export {
                export_id: ExportId(7),
            },
        );
        assert_eq!(
            printer().line(&event).unwrap(),
            "- export 7: Warning player_unlisted"
        );
    }

    #[test]
    fn exports_with_the_same_stem_stay_distinguishable() {
        let mut printer = ConsolePrinter::new();
        printer.line(&started(1, "co - Spring"));
        printer.line(&started(2, "co - Spring.zip"));
        let line_for = |printer: &mut ConsolePrinter, id: u64| {
            printer
                .line(&message(
                    Severity::Error,
                    "players_txt_missing",
                    Scope::Export {
                        export_id: ExportId(id),
                    },
                ))
                .unwrap()
        };
        assert_eq!(
            line_for(&mut printer, 1),
            "- co - Spring: Error players_txt_missing"
        );
        assert_eq!(
            line_for(&mut printer, 2),
            "- co - Spring.zip: Error players_txt_missing"
        );
    }

    #[test]
    fn events_other_than_messages_print_nothing() {
        let mut printer = ConsolePrinter::new();
        let silent = [
            PipelineEvent::ExportProcessed {
                export_id: ExportId(1),
            },
            PipelineEvent::FolderStatus {
                export_id: ExportId(1),
                path: path("Players/15 - B"),
                status: FolderStatus::Done,
            },
            PipelineEvent::Progress {
                processed: 1,
                total: 2,
            },
            PipelineEvent::Complete {
                success: true,
                files_written: 0,
                skipped_folders: 0,
                duplicates: 0,
                folders_with_errors: 0,
            },
            PipelineEvent::UnmigratedContentFound,
        ];
        for event in &silent {
            assert_eq!(printer.line(event), None);
        }
    }
}
