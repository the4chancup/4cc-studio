//! The Save editor's message catalog: each code the commands report, its severity, and the
//! `Message` it is sent as. Every finding is run-wide (`Scope::Run`) and keeps what it is
//! about (`Disposition::Keep`): a command works on one save, not on exports, and a finding
//! that drops something says so with its own code. Until the help window needs them a row
//! holds no text: the console prints the code with its context fields.

use std::borrow::Cow;

use studio_core::{Disposition, Message, MessageCode, Scope, Severity};

/// The tool's id: its CLI subcommand, its settings section and the owner of its message codes.
pub(crate) const TOOL_ID: &str = "save-editor";

/// The codes the commands report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Code {
    /// `export-toml` wrote one team's file (`team`, `path`).
    TeamExported,
}

impl Code {
    /// The code as the console prints it and the help window lists it.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Code::TeamExported => "team_exported",
        }
    }

    /// How serious the finding is: an `Error` makes the command's exit code 1.
    fn severity(self) -> Severity {
        match self {
            Code::TeamExported => Severity::Info,
        }
    }
}

/// The message for `code` with its context fields, in template order.
pub(crate) fn message(code: Code, context: Vec<(&'static str, String)>) -> Message {
    Message {
        code: MessageCode {
            tool_id: TOOL_ID,
            code: Cow::Borrowed(code.as_str()),
        },
        severity: code.severity(),
        disposition: Disposition::Keep,
        scope: Scope::Run,
        context: context
            .into_iter()
            .map(|(key, value)| (key.to_owned(), value))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_carries_its_code_severity_and_context_run_wide() {
        let message = message(
            Code::TeamExported,
            vec![("team", "714".to_owned()), ("path", "714.toml".to_owned())],
        );
        assert_eq!(message.code.tool_id, "save-editor");
        assert_eq!(message.code.code, "team_exported");
        assert_eq!(message.severity, Severity::Info);
        assert_eq!(message.disposition, Disposition::Keep);
        assert_eq!(message.scope, Scope::Run);
        assert_eq!(
            message.context,
            vec![
                ("team".to_owned(), "714".to_owned()),
                ("path".to_owned(), "714.toml".to_owned())
            ]
        );
    }
}
