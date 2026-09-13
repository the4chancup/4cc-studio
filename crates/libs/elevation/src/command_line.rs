//! Building a Windows command line from argv: the quoting rules `CommandLineToArgvW` applies
//! when it parses the line back. `relaunch_elevated` passes its arguments through
//! `ShellExecuteExW`'s single `lpParameters` string, so this is where argv is reconstructed.

use std::ffi::{OsStr, OsString};

/// One argument quoted for a Windows command line, by `CommandLineToArgvW`'s rules: an
/// argument without spaces, tabs or quotes — and not ending in a backslash — is passed as
/// is; the empty argument is `""`; otherwise it is wrapped in `"` with each `"` escaped as
/// `\"` (doubling any backslashes immediately before it) and the run of backslashes at the
/// very end doubled before the closing quote. The `OsStr` conversion is lossy: the OS
/// receives a best-effort rendering of non-UTF-16 text.
pub(crate) fn quote_argument(arg: &OsStr) -> String {
    let text = arg.to_string_lossy();
    let needs_quotes = text.is_empty()
        || text.ends_with('\\')
        || text.chars().any(|c| matches!(c, ' ' | '\t' | '"'));
    if !needs_quotes {
        return text.into_owned();
    }
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    let mut backslashes = 0usize;
    for c in text.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                quoted.push_str(&"\\".repeat(backslashes));
                backslashes = 0;
                quoted.push(c);
            }
        }
    }
    quoted.push_str(&"\\".repeat(backslashes * 2));
    quoted.push('"');
    quoted
}

/// The `lpParameters` string: every argument quoted and joined with single spaces.
pub(crate) fn join_arguments(args: &[OsString]) -> String {
    args.iter()
        .map(|arg| quote_argument(arg))
        .collect::<Vec<_>>()
        .join(" ")
}
