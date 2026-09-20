//! Administrator rights for the tools that write into the PES install directory: whether the
//! process has them, how to relaunch with them, and how to recognize the error their absence causes.

#[cfg(any(windows, test))]
mod command_line;
#[cfg(unix)]
mod posix;
#[cfg(windows)]
mod windows;

use std::ffi::OsString;
use std::path::Path;

/// What raising or checking elevation can fail with.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ElevationError {
    /// This platform has no elevation prompt.
    #[error("this platform has no elevation prompt")]
    Unsupported,
    /// The user dismissed the UAC prompt (`ERROR_CANCELLED`).
    #[error("the elevation prompt was declined")]
    Declined,
    /// The OS call's message.
    #[error("elevation failed: {0}")]
    Failed(String),
}

/// Whether this process already has administrator (Windows) or root (POSIX) rights.
#[cfg(windows)]
pub fn is_elevated() -> bool {
    windows::is_elevated()
}

/// Whether this process already has administrator (Windows) or root (POSIX) rights.
#[cfg(unix)]
pub fn is_elevated() -> bool {
    posix::is_elevated()
}

/// Whether this process already has administrator (Windows) or root (POSIX) rights; `false`
/// where the question has no meaning.
#[cfg(not(any(windows, unix)))]
pub fn is_elevated() -> bool {
    false
}

/// Starts `program` with `args` elevated (the UAC consent prompt) and returns once the new
/// process is launched; the caller then exits.
#[cfg(windows)]
pub fn relaunch_elevated(program: &Path, args: &[OsString]) -> Result<(), ElevationError> {
    windows::relaunch_elevated(program, args)
}

/// Starts `program` with `args` elevated and returns once the new process is launched; the
/// caller then exits. On POSIX this is `Unsupported`: the tool prints the sudo hint itself,
/// since it knows its own command line.
#[cfg(unix)]
pub fn relaunch_elevated(program: &Path, args: &[OsString]) -> Result<(), ElevationError> {
    posix::relaunch_elevated(program, args)
}

/// Starts `program` with `args` elevated; `Unsupported` where there is no elevation prompt.
#[cfg(not(any(windows, unix)))]
pub fn relaunch_elevated(_program: &Path, _args: &[OsString]) -> Result<(), ElevationError> {
    Err(ElevationError::Unsupported)
}

/// Whether an I/O error is the access-denied kind that elevation would cure.
pub fn is_access_denied(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::PermissionDenied
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::io::ErrorKind;

    #[test]
    fn is_elevated_is_stable() {
        // The value itself depends on how the test process was launched; what must hold is
        // that the query works and does not flip between calls.
        assert_eq!(is_elevated(), is_elevated());
    }

    #[test]
    fn access_denied_kind() {
        assert!(is_access_denied(&std::io::Error::from(
            ErrorKind::PermissionDenied
        )));
        assert!(!is_access_denied(&std::io::Error::from(
            ErrorKind::NotFound
        )));
    }

    #[test]
    fn quote_argument_cases() {
        let cases = [
            ("abc", "abc"),
            ("a b", "\"a b\""),
            ("", "\"\""),
            ("say \"hi\"", "\"say \\\"hi\\\"\""),
            ("C:\\dir\\", "\"C:\\dir\\\\\""),
            ("back\\\\\"slash", "\"back\\\\\\\\\\\"slash\""),
            ("x\\y", "x\\y"),
        ];
        for (argument, expected) in cases {
            assert_eq!(
                command_line::quote_argument(OsStr::new(argument)),
                expected,
                "{argument}"
            );
        }
    }

    #[test]
    fn join_arguments_quotes_each_and_separates_with_one_space() {
        let arguments: Vec<OsString> = ["abc", "a b", "", "C:\\dir\\"]
            .into_iter()
            .map(OsString::from)
            .collect();
        assert_eq!(
            command_line::join_arguments(&arguments),
            "abc \"a b\" \"\" \"C:\\dir\\\\\""
        );
        assert_eq!(command_line::join_arguments(&[]), "");
    }

    #[cfg(windows)]
    #[test]
    fn quoting_round_trips_through_the_os() {
        use ::windows::Win32::Foundation::{HLOCAL, LocalFree};
        use ::windows::Win32::UI::Shell::CommandLineToArgvW;
        use ::windows::core::PCWSTR;

        let arguments: Vec<OsString> = [
            "abc",
            "a b",
            "",
            "say \"hi\"",
            "C:\\dir\\",
            "back\\\\\"slash",
            "x\\y",
            "--path=C:\\Program Files\\PES 2017\\",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        let line = format!("prog.exe {}", command_line::join_arguments(&arguments));
        let wide: Vec<u16> = line.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 buffer that outlives the call;
        // `CommandLineToArgvW` returns a heap block of `count` NUL-terminated PWSTRs that is
        // freed with LocalFree.
        let parsed = unsafe {
            let mut count = 0i32;
            let argv = CommandLineToArgvW(PCWSTR::from_raw(wide.as_ptr()), &mut count);
            assert!(!argv.is_null(), "CommandLineToArgvW failed");
            let mut parsed = Vec::with_capacity(count as usize);
            for i in 0..count as usize {
                parsed.push((*argv.add(i)).to_string().expect("argv text"));
            }
            LocalFree(Some(HLOCAL(argv as *mut _)));
            parsed
        };
        let mut expected = vec!["prog.exe".to_string()];
        expected.extend(
            arguments
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned()),
        );
        assert_eq!(parsed, expected);
    }

    #[cfg(not(windows))]
    #[test]
    fn relaunch_is_unsupported() {
        assert_eq!(
            relaunch_elevated(Path::new("x"), &[]),
            Err(ElevationError::Unsupported)
        );
    }
}
