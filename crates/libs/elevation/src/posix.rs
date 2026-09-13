//! The POSIX half of the crate: root detection via `geteuid`. There is no sudo prompt to
//! raise here — `relaunch_elevated` is `Unsupported` and the tool prints the sudo hint
//! itself, since it knows its own command line.

use std::ffi::OsString;
use std::path::Path;

use crate::ElevationError;

pub(crate) fn is_elevated() -> bool {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() == 0 }
}

pub(crate) fn relaunch_elevated(_program: &Path, _args: &[OsString]) -> Result<(), ElevationError> {
    Err(ElevationError::Unsupported)
}
