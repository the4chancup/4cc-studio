//! The staging folder (`spec/export_upgrader.md` "CLI"): the output written
//! under `<name>.upgrading` beside where it goes, renamed to the output by a
//! clean verdict, so no reader sees a half-written export under its name.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use studio_core::CliError;

use crate::upgrade::aborted;

/// The claim on the output folder both commands make before anything is
/// written, one rule for both modes: an existing `output` is refused rather
/// than replaced, an existing staging folder is refused (an aborted run left
/// it, or another run is writing it), `output`'s missing parent is made (the
/// default output sits beside the input, whose parent exists by definition),
/// and the staging folder is created with `create_dir`, so one that appeared
/// between the check and the create is never written into. `output` has a
/// folder name: `upgrade`'s always does, `upgrade-models` checks it first.
pub(crate) fn claim_staging(output: &Path) -> Result<PathBuf, CliError> {
    if output.exists() {
        return Err(aborted(anyhow!(
            "{}: the output folder already exists",
            output.display()
        )));
    }
    let name = output
        .file_name()
        .expect("`output` has a folder name (the caller checked)")
        .to_string_lossy();
    let staging = output.with_file_name(format!("{name}.upgrading"));
    if staging.exists() {
        return Err(aborted(anyhow!(
            "{}: the staging folder already exists: an aborted run left it, or another run is writing it; remove it and run again",
            staging.display()
        )));
    }
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|error| {
            aborted(anyhow!(
                "{}: cannot create the directory: {error}",
                parent.display()
            ))
        })?;
    }
    fs::create_dir(&staging).map_err(|error| {
        aborted(anyhow!(
            "{}: cannot create the staging folder: {error}",
            staging.display()
        ))
    })?;
    Ok(staging)
}

/// Writes `bytes` to `staging`'s file at `relative`, parent folders created;
/// a failure is the aborted code naming the file and saying the staged
/// output is incomplete: it is left for the user to remove, since a second
/// run refuses an existing staging folder.
pub(crate) fn write_output_file(
    staging: &Path,
    relative: &str,
    bytes: &[u8],
) -> Result<(), CliError> {
    let path = staging.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            aborted(anyhow!(
                "{}: cannot create the directory: {error}; the output {} is incomplete",
                parent.display(),
                staging.display()
            ))
        })?;
    }
    fs::write(&path, bytes).map_err(|error| {
        aborted(anyhow!(
            "{}: cannot write the file: {error}; the output {} is incomplete",
            path.display(),
            staging.display()
        ))
    })
}

/// The verdict applied to the staging folder "CLI" stages the output in
/// (`spec/export_upgrader.md` "CLI"): a clean run's is renamed to the
/// output, so no reader ever sees a half-written export under the output's
/// name; an unclean run's is removed, nothing written. A failure aborts,
/// the incomplete staging folder left for the user to remove, as a mid-run
/// abort's is.
pub(crate) fn finish_staging(staging: &Path, output: &Path, clean: bool) -> Result<(), CliError> {
    if clean {
        retry_while_locked(|| fs::rename(staging, output)).map_err(|error| {
            aborted(anyhow!(
                "{}: cannot rename it to the output {}: {error}",
                staging.display(),
                output.display()
            ))
        })
    } else {
        retry_while_locked(|| fs::remove_dir_all(staging)).map_err(|error| {
            aborted(anyhow!(
                "{}: cannot remove the staging folder: {error}",
                staging.display()
            ))
        })
    }
}

/// `step` retried while it fails with `PermissionDenied` — 20 attempts,
/// 100 ms apart: on Windows a folder whose files were just written can stay
/// open for a moment in another process (an antivirus scan holds it
/// briefly), and a clean run should not be lost to a lock that clears by
/// itself ("CLI" in `spec/export_upgrader.md`). Any other error kind
/// returns at once.
fn retry_while_locked(mut step: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    for attempt in 1..=20 {
        match step() {
            Err(error) if error.kind() == io::ErrorKind::PermissionDenied && attempt < 20 => {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            result => return result,
        }
    }
    unreachable!("every attempt returns or retries")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ABORTED;
    use crate::test_support::scratch;

    #[test]
    fn an_unwritable_parent_dir_aborts_naming_it_and_the_incomplete_output() {
        let output = scratch("write_parent_blocked");
        // `Players` is a file: the parent of the `settings.toml` cannot be
        // created.
        fs::write(output.join("Players"), b"x").unwrap();
        let error = write_output_file(&output, "Players/03 - A/settings.toml", b"x")
            .expect_err("the parent cannot be created");
        assert_eq!(error.exit_code, ABORTED, "{error}");
        // The incomplete output is left as it is.
        assert!(output.join("Players").is_file());
        let text = error.to_string();
        assert!(text.contains("cannot create the directory"), "{text}");
        assert!(text.contains("03 - A"), "{text}");
        assert!(text.contains("incomplete"), "{text}");
    }

    #[test]
    fn an_unwritable_file_aborts_naming_it_and_the_incomplete_output() {
        let output = scratch("write_file_blocked");
        // `colors.txt` is a directory: the file cannot be written.
        fs::create_dir(output.join("colors.txt")).unwrap();
        let error =
            write_output_file(&output, "colors.txt", b"x").expect_err("the file cannot be written");
        assert_eq!(error.exit_code, ABORTED, "{error}");
        assert!(output.join("colors.txt").is_dir());
        let text = error.to_string();
        assert!(text.contains("cannot write the file"), "{text}");
        assert!(text.contains("colors.txt"), "{text}");
        assert!(text.contains("incomplete"), "{text}");
    }

    /// A file of `path` opened readable but not deletable (`FILE_SHARE_READ`
    /// alone): Windows refuses a rename or a removal of its folder while the
    /// handle lives — the lock a scanner's own open holds for a moment.
    #[cfg(windows)]
    fn held_without_delete(path: &Path) -> fs::File {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 1;
        fs::File::options()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(path)
            .unwrap()
    }

    #[cfg(windows)]
    #[test]
    fn finish_staging_retries_the_locked_folder_until_the_lock_clears() {
        let root = scratch("staging_locked_once");
        fs::create_dir_all(root.join("staging")).unwrap();
        let held = root.join("staging/held.txt");
        fs::write(&held, b"held").unwrap();
        let lock = held_without_delete(&held);
        let dropped = std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            drop(lock);
        });

        finish_staging(&root.join("staging"), &root.join("output"), true).unwrap();
        assert_eq!(fs::read(root.join("output/held.txt")).unwrap(), b"held");
        dropped.join().unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn finish_staging_aborts_when_the_lock_stays() {
        let root = scratch("staging_locked_always");
        fs::create_dir_all(root.join("staging")).unwrap();
        let held = root.join("staging/held.txt");
        fs::write(&held, b"held").unwrap();
        let lock = held_without_delete(&held);

        let error = finish_staging(&root.join("staging"), &root.join("output"), true)
            .expect_err("the lock stays: the retries run out");
        assert_eq!(error.exit_code, ABORTED, "{error}");
        let text = error.to_string();
        assert!(text.contains("staging"), "{text}");
        assert!(root.join("staging").exists());
        assert_eq!(fs::read(root.join("staging/held.txt")).unwrap(), b"held");
        drop(lock);
    }
}
