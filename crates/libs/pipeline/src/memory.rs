//! The run's memory cap: a share of the physical memory available now — the
//! game, a browser or Blender may hold much of the machine's total, so a cap
//! on the total would push the run into swap. Read once at run start.

/// The budget cap for a run: `percent` of the physical memory available now,
/// or of `FALLBACK_AVAILABLE` where the OS cannot say. `percent` is validated
/// by the caller's settings (`0 < percent <= 100`).
pub fn memory_cap(percent: f64) -> usize {
    cap_of(available_memory().unwrap_or(FALLBACK_AVAILABLE), percent)
}

/// What `memory_cap` assumes is available where the OS cannot say: 4 GiB.
const FALLBACK_AVAILABLE: u64 = 4 << 30;

/// The physical memory available now, or `None` where the OS cannot say or
/// the read fails. Windows asks `GlobalMemoryStatusEx` (`ullAvailPhys` counts
/// the standby cache as available); Linux reads the kernel's own
/// `MemAvailable` estimate; anything else is no target and says nothing.
#[cfg(windows)]
fn available_memory() -> Option<u64> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: `status` is a valid `MEMORYSTATUSEX` out-parameter with its
    // `dwLength` set; `GlobalMemoryStatusEx` writes the whole struct and
    // reports failure by return, not by touching it.
    unsafe { GlobalMemoryStatusEx(&mut status) }.ok()?;
    Some(status.ullAvailPhys)
}

/// The `MemAvailable` line of `/proc/meminfo`, in bytes.
#[cfg(target_os = "linux")]
fn available_memory() -> Option<u64> {
    mem_available(&std::fs::read_to_string("/proc/meminfo").ok()?)
}

/// The `MemAvailable:` line's kB value × 1024, or `None` when the line is
/// absent or unparsable. `#[cfg(test)]` lets the parse run on every platform;
/// the other-platform arm is a plain `None`.
#[cfg(any(target_os = "linux", test))]
fn mem_available(meminfo: &str) -> Option<u64> {
    for line in meminfo.lines() {
        if let Some(rest) = line.strip_prefix("MemAvailable:") {
            let kib: u64 = rest.split_whitespace().next()?.parse().ok()?;
            return Some(kib * 1024);
        }
    }
    None
}

/// Every other platform: no read exists.
#[cfg(not(any(windows, target_os = "linux")))]
fn available_memory() -> Option<u64> {
    None
}

/// `percent` of `available`: the product is computed in `f64` and `as`
/// saturates into `usize`, so a huge `available` gives `usize::MAX`, never a
/// wrap. There is no floor — the oversized branch keeps the budget making
/// progress at any cap.
fn cap_of(available: u64, percent: f64) -> usize {
    (available as f64 * percent / 100.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cap_of_gives_the_percent_share() {
        assert_eq!(cap_of(10 << 30, 80.0), 8 << 30);
        assert_eq!(cap_of(12 << 30, 100.0), 12 << 30);
        assert_eq!(cap_of(1000, 0.5), 5);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn cap_of_saturates_instead_of_overflowing() {
        assert_eq!(cap_of(u64::MAX, 100.0), usize::MAX);
    }

    #[cfg(any(windows, target_os = "linux"))]
    #[test]
    fn the_os_reports_available_memory() {
        let available = available_memory().expect("the OS reports it");
        // No machine that runs the tests has less memory free than this.
        assert!(available > 64 << 20, "{available}");
        assert!(memory_cap(50.0) > 32 << 20);
    }

    #[test]
    fn the_fallback_is_four_gib() {
        assert_eq!(FALLBACK_AVAILABLE, 4 * 1024 * 1024 * 1024);
    }

    #[test]
    fn mem_available_reads_the_line_or_reports_none() {
        let meminfo = "MemTotal:      16000000 kB\nMemAvailable:   8000000 kB\n";
        assert_eq!(mem_available(meminfo), Some(8_192_000_000));
        assert_eq!(mem_available("MemTotal:      16000000 kB\n"), None);
    }
}
