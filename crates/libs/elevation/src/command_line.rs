//! Building a Windows command line from argv: the quoting rules `CommandLineToArgvW` applies
//! when it parses the line back. `relaunch_elevated` passes its arguments through
//! `ShellExecuteExW`'s single `lpParameters` string, so this is where argv is reconstructed.
//! Everything here works on UTF-16 code units, so an argument that is not valid Unicode
//! (an unpaired surrogate) survives the round trip unchanged.

/// `"` as a code unit.
const QUOTE: u16 = '"' as u16;
/// `\` as a code unit.
const BACKSLASH: u16 = '\\' as u16;
/// ` ` as a code unit.
const SPACE: u16 = ' ' as u16;
/// `\t` as a code unit.
const TAB: u16 = '\t' as u16;

/// One argument quoted for a Windows command line, by `CommandLineToArgvW`'s rules: an
/// argument without spaces, tabs or quotes — and not ending in a backslash — is passed as
/// is; the empty argument is `""`; otherwise it is wrapped in `"` with each `"` escaped as
/// `\"` (doubling any backslashes immediately before it) and the run of backslashes at the
/// very end doubled before the closing quote. The units are copied verbatim: the caller
/// converts with `OsStr::encode_wide`, so nothing is lost on non-Unicode text.
pub(crate) fn quote_argument(arg: &[u16]) -> Vec<u16> {
    let needs_quotes = arg.is_empty()
        || arg.last() == Some(&BACKSLASH)
        || arg.iter().any(|unit| matches!(*unit, SPACE | TAB | QUOTE));
    if !needs_quotes {
        return arg.to_vec();
    }
    let mut quoted = Vec::with_capacity(arg.len() + 2);
    quoted.push(QUOTE);
    let mut backslashes = 0usize;
    for &unit in arg {
        match unit {
            BACKSLASH => backslashes += 1,
            QUOTE => {
                quoted.extend(std::iter::repeat_n(BACKSLASH, backslashes * 2 + 1));
                quoted.push(QUOTE);
                backslashes = 0;
            }
            _ => {
                quoted.extend(std::iter::repeat_n(BACKSLASH, backslashes));
                backslashes = 0;
                quoted.push(unit);
            }
        }
    }
    quoted.extend(std::iter::repeat_n(BACKSLASH, backslashes * 2));
    quoted.push(QUOTE);
    quoted
}

/// The `lpParameters` units: every argument quoted and joined with single spaces.
pub(crate) fn join_arguments(args: &[Vec<u16>]) -> Vec<u16> {
    let mut joined = Vec::new();
    for (index, arg) in args.iter().enumerate() {
        if index > 0 {
            joined.push(SPACE);
        }
        joined.extend_from_slice(&quote_argument(arg));
    }
    joined
}
