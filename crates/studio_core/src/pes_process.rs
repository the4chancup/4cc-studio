//! Which PES games are running now (core plan, "The running-PES probe"): the Save
//! editor's writing commands refuse while one is, and the GUI's version selector polls
//! it on a timer.

use std::path::PathBuf;

use pes_version::PesVersion;

/// A PES game running now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningPes {
    /// Which game it is, matched from its executable's name.
    pub version: PesVersion,
    /// The executable's full path, `None` where the process would not say (one of another
    /// user's session, or a protected process).
    pub exe: Option<PathBuf>,
}

/// The PES games running now (core plan, "The running-PES probe").
pub fn running_pes() -> Vec<RunningPes> {
    probe()
}

/// The version a process's executable name says it is: `PES2015.exe` to `PES2021.exe` in
/// any letter case, the names the games ship with. Anything else (`PES2014.exe`,
/// `PES2021.exe.bak`, `pes2021`, `xPES2021.exe`, `pes21.exe`) is no match.
#[cfg(any(windows, target_os = "linux", test))]
fn version_of_exe(name: &str) -> Option<PesVersion> {
    let lower = name.to_ascii_lowercase();
    let year: u16 = lower
        .strip_prefix("pes")?
        .strip_suffix(".exe")?
        .parse()
        .ok()?;
    // `from_number` also accepts the short `pes21.exe`, which is not a shipped name.
    PesVersion::from_number(year).filter(|version| version.year() == year)
}

/// The game a `/proc/<pid>/cmdline` buffer says its process is, or `None`: the first
/// NUL-separated argument's last segment after a `\` or `/` is the exe name (a Windows
/// path under Wine or Proton), and the argument itself is its `exe`. `#[cfg(test)]`
/// lets the parse run on every platform.
#[cfg(any(target_os = "linux", test))]
fn exe_of_cmdline(bytes: &[u8]) -> Option<(PesVersion, PathBuf)> {
    let argument = bytes.split(|byte| *byte == 0).next()?;
    let argument = String::from_utf8_lossy(argument);
    let name = argument.rsplit(['\\', '/']).next()?;
    let version = version_of_exe(name)?;
    Some((version, PathBuf::from(argument.as_ref())))
}

/// Windows: every process the Toolhelp snapshot lists, its id and its exe file name.
/// A process the snapshot cannot read is skipped.
#[cfg(windows)]
fn processes() -> Vec<(u32, String)> {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };

    // SAFETY: asks only for a process-list snapshot; the handle is closed before this
    // function returns.
    let snapshot = match unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) } {
        Ok(snapshot) => snapshot,
        Err(error) => {
            log::debug!("the process snapshot failed: {error}");
            return Vec::new();
        }
    };
    let mut processes = Vec::new();
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    // SAFETY: `snapshot` is a live snapshot handle and `entry` a valid out-entry with
    // its `dwSize` set.
    let mut more = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while more {
        let end = entry
            .szExeFile
            .iter()
            .position(|unit| *unit == 0)
            .unwrap_or(entry.szExeFile.len());
        processes.push((
            entry.th32ProcessID,
            String::from_utf16_lossy(&entry.szExeFile[..end]),
        ));
        // SAFETY: the same live snapshot handle and out-entry as `Process32FirstW`.
        more = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    // SAFETY: `snapshot` is an owned handle this function created and is done with.
    if let Err(error) = unsafe { CloseHandle(snapshot) } {
        log::debug!("closing the process snapshot failed: {error}");
    }
    processes
}

/// Windows: the games among `processes()`, each match's image path asked of the
/// process itself.
#[cfg(windows)]
fn probe() -> Vec<RunningPes> {
    processes()
        .into_iter()
        .filter_map(|(id, name)| {
            version_of_exe(&name).map(|version| RunningPes {
                version,
                exe: image_path(id),
            })
        })
        .collect()
}

/// The full image path of `process_id`, `None` when the process refuses the query. The
/// limited right is granted on an elevated process too; another user's or a protected one
/// can refuse it.
#[cfg(windows)]
fn image_path(process_id: u32) -> Option<PathBuf> {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::core::PWSTR;

    // SAFETY: asks for the query-limited right only; the handle is closed before this
    // function returns.
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }
        .inspect_err(|error| log::debug!("process {process_id} refused a query: {error}"))
        .ok()?;
    let mut buffer = [0u16; 1024];
    let mut len = buffer.len() as u32;
    // SAFETY: `process` is a live handle; `buffer` is writable for `len` UTF-16 units
    // and the call updates `len` to the count it wrote.
    let queried = unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut len,
        )
    };
    // SAFETY: `process` is an owned handle this function created and is done with.
    if let Err(error) = unsafe { CloseHandle(process) } {
        log::debug!("closing a process handle failed: {error}");
    }
    queried
        .inspect_err(|error| log::debug!("process {process_id} hid its image path: {error}"))
        .ok()?;
    Some(PathBuf::from(OsString::from_wide(&buffer[..len as usize])))
}

/// Linux: every `/proc/*/cmdline` that reads. No filter on the entry's name: past the
/// process ids only `self` and `thread-self` have one, and they are this process, never a
/// game.
#[cfg(target_os = "linux")]
fn cmdlines() -> Vec<Vec<u8>> {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    let mut cmdlines = Vec::new();
    for entry in entries.flatten() {
        if let Ok(cmdline) = std::fs::read(entry.path().join("cmdline")) {
            cmdlines.push(cmdline);
        }
    }
    cmdlines
}

/// Linux: the games among `cmdlines()`; each buffer's first argument under Wine or
/// Proton is the exe's Windows path.
#[cfg(target_os = "linux")]
fn probe() -> Vec<RunningPes> {
    cmdlines()
        .iter()
        .filter_map(|cmdline| exe_of_cmdline(cmdline))
        .map(|(version, exe)| RunningPes {
            version,
            exe: Some(exe),
        })
        .collect()
}

/// Every other platform: no probe exists.
#[cfg(not(any(windows, target_os = "linux")))]
fn probe() -> Vec<RunningPes> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_of_exe_matches_each_shipped_name_in_any_case() {
        let expected = [
            ("PES2015.exe", PesVersion::Pes15),
            ("PES2016.exe", PesVersion::Pes16),
            ("PES2017.exe", PesVersion::Pes17),
            ("PES2018.exe", PesVersion::Pes18),
            ("PES2019.exe", PesVersion::Pes19),
            ("PES2020.exe", PesVersion::Pes20),
            ("PES2021.exe", PesVersion::Pes21),
            ("pes2018.EXE", PesVersion::Pes18),
            ("Pes2021.Exe", PesVersion::Pes21),
        ];
        for (name, version) in expected {
            assert_eq!(version_of_exe(name), Some(version), "{name}");
        }
    }

    #[test]
    fn version_of_exe_matches_nothing_else() {
        for name in [
            "PES2014.exe",
            "PES2021.exe.bak",
            "pes2021",
            "xPES2021.exe",
            "pes21.exe",
        ] {
            assert_eq!(version_of_exe(name), None, "{name}");
        }
    }

    #[test]
    fn exe_of_cmdline_reads_the_first_argument_last_segment() {
        assert_eq!(
            exe_of_cmdline(b"Z:\\games\\PES2021\\PES2021.exe\0-arg\0"),
            Some((
                PesVersion::Pes21,
                PathBuf::from("Z:\\games\\PES2021\\PES2021.exe"),
            )),
        );
        assert_eq!(
            exe_of_cmdline(b"/home/x/PES2017.exe\0"),
            Some((PesVersion::Pes17, PathBuf::from("/home/x/PES2017.exe"))),
        );
    }

    #[test]
    fn exe_of_cmdline_skips_empty_and_non_game_arguments() {
        assert_eq!(exe_of_cmdline(b""), None);
        assert_eq!(exe_of_cmdline(b"wine\0"), None);
    }

    #[cfg(windows)]
    #[test]
    fn processes_lists_this_test_process() {
        let pid = std::process::id();
        let name = std::env::current_exe()
            .expect("the test binary has a path")
            .file_name()
            .expect("the path has a file name")
            .to_string_lossy()
            .into_owned();
        assert!(
            processes().contains(&(pid, name.clone())),
            "({pid}, {name}) not in the snapshot"
        );
    }

    #[cfg(windows)]
    #[test]
    fn image_path_is_this_test_binary() {
        let expected = std::env::current_exe().expect("the test binary has a path");
        assert_eq!(image_path(std::process::id()), Some(expected));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn cmdlines_hold_this_test_process() {
        let own = std::fs::read("/proc/self/cmdline").expect("/proc/self is readable");
        assert!(cmdlines().contains(&own), "own cmdline not in the list");
    }

    #[test]
    fn running_pes_lists_only_supported_versions() {
        for game in running_pes() {
            assert!(PesVersion::ALL.contains(&game.version), "{game:?}");
        }
    }
}
