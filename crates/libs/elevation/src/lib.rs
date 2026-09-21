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
    use std::io::ErrorKind;

    #[cfg(windows)]
    #[test]
    fn is_elevated_matches_the_os_answer() {
        use ::windows::Win32::UI::Shell::IsUserAnAdmin;

        // The token query must agree with shell32's own elevation check.
        // SAFETY: IsUserAnAdmin has no preconditions.
        let shell_answer = unsafe { IsUserAnAdmin() }.as_bool();
        assert_eq!(is_elevated(), shell_answer);
    }

    #[cfg(unix)]
    #[test]
    fn is_elevated_matches_root() {
        // A test process has uid == euid, so the euid check must agree with getuid.
        // SAFETY: getuid has no preconditions and cannot fail.
        assert_eq!(is_elevated(), unsafe { libc::getuid() } == 0);
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
                command_line::quote_argument(&argument.encode_utf16().collect::<Vec<u16>>()),
                expected.encode_utf16().collect::<Vec<u16>>(),
                "{argument}"
            );
        }
        // An unpaired surrogate passes through unchanged.
        assert_eq!(
            command_line::quote_argument(&[0x61, 0xD800, 0x62]),
            [0x61, 0xD800, 0x62]
        );
    }

    #[test]
    fn join_arguments_quotes_each_and_separates_with_one_space() {
        let arguments: Vec<Vec<u16>> = ["abc", "a b", "", "C:\\dir\\"]
            .into_iter()
            .map(|argument| argument.encode_utf16().collect())
            .collect();
        assert_eq!(
            command_line::join_arguments(&arguments),
            "abc \"a b\" \"\" \"C:\\dir\\\\\""
                .encode_utf16()
                .collect::<Vec<u16>>()
        );
        assert_eq!(command_line::join_arguments(&[]), Vec::<u16>::new());
    }

    #[cfg(windows)]
    #[test]
    fn quoting_round_trips_through_the_os() {
        use ::windows::Win32::Foundation::{HLOCAL, LocalFree};
        use ::windows::Win32::UI::Shell::CommandLineToArgvW;
        use ::windows::core::PCWSTR;
        use std::os::windows::ffi::{OsStrExt, OsStringExt};

        let mut arguments: Vec<OsString> = [
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
        // An unpaired high surrogate: not valid Unicode, still a legal OS argument.
        arguments.push(OsString::from_wide(&[0x61, 0xD800, 0x62]));

        let units: Vec<Vec<u16>> = arguments
            .iter()
            .map(|arg| arg.as_os_str().encode_wide().collect())
            .collect();
        let mut line: Vec<u16> = "prog.exe ".encode_utf16().collect();
        line.extend_from_slice(&command_line::join_arguments(&units));
        line.push(0);
        // SAFETY: `line` is a NUL-terminated UTF-16 buffer that outlives the call;
        // `CommandLineToArgvW` returns a heap block of `count` NUL-terminated PWSTRs that is
        // freed with LocalFree.
        let parsed = unsafe {
            let mut count = 0i32;
            let argv = CommandLineToArgvW(PCWSTR::from_raw(line.as_ptr()), &mut count);
            assert!(!argv.is_null(), "CommandLineToArgvW failed");
            let mut parsed = Vec::with_capacity(count as usize);
            for i in 0..count as usize {
                parsed.push((*argv.add(i)).as_wide().to_vec());
            }
            LocalFree(Some(HLOCAL(argv as *mut _)));
            parsed
        };
        let mut expected: Vec<Vec<u16>> = vec!["prog.exe".encode_utf16().collect()];
        expected.extend(units);
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
