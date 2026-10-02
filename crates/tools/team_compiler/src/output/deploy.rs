//! Staging and promotion (`team_compiler/pipeline.md` "6. Post-processing"): a CPK is written
//! in a staging folder of its own run and only then renamed to its final path, so a failed run
//! never leaves a half-written CPK where the game or the user could pick it up.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use pipeline::CpkStem;

/// The output subfolder that holds every run's staging folder.
const STAGING: &str = ".staging";

/// This run's staging folder, `<output>/.staging/<pid>-<unix ms>`: the process id keeps two
/// runs at once apart, the time two runs of a recycled process id.
pub(crate) fn staging_folder(output: &Path) -> PathBuf {
    // A clock set before 1970 leaves only the process id to tell runs apart, which is enough
    // for the runs alive at once.
    let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_) => 0,
    };
    output
        .join(STAGING)
        .join(format!("{}-{millis}", std::process::id()))
}

/// The CPK file name for `cpk_stem`.
pub(crate) fn cpk_file_name(cpk_stem: &CpkStem) -> String {
    format!("{}.cpk", cpk_stem.as_str())
}

/// Creates the output folder `output` if it is missing and probes it for writing: a file
/// created in it and removed. Each failure names the folder, not the probe.
pub(crate) fn prepare_output_folder(output: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(output)
        .with_context(|| format!("{}: cannot create the output folder", output.display()))?;
    let cannot_write = || format!("{}: cannot write in the output folder", output.display());
    let probe = output.join(format!(".write-probe-{}", std::process::id()));
    fs::write(&probe, b"").with_context(cannot_write)?;
    fs::remove_file(&probe).with_context(cannot_write)
}

/// Moves the CPK staged in `run_folder` to `<output>/<cpk_stem>.cpk`, replacing a previous
/// one, then discards the emptied `run_folder`. The rename stays on one volume, so the final
/// path holds either the previous CPK or the whole new one, never part of it. Only the
/// rename's failure is the commit's: once it is done the new CPK is in place, so a failure of
/// the cleanup after it is logged, not returned. Returns the promoted path.
pub(crate) fn promote(
    run_folder: &Path,
    output: &Path,
    cpk_stem: &CpkStem,
) -> anyhow::Result<PathBuf> {
    let name = cpk_file_name(cpk_stem);
    let promoted = output.join(&name);
    fs::rename(run_folder.join(&name), &promoted)
        .with_context(|| format!("{}: cannot replace it with the new CPK", promoted.display()))?;
    discard(run_folder, output);
    Ok(promoted)
}

/// Removes the run's `run_folder` with whatever it holds, then `.staging/` when no other
/// run's folder is left in it. A failure to remove is logged, not returned: a run that
/// failed already carries the error that matters, and a promoted one has its CPK in place.
pub(crate) fn discard(run_folder: &Path, output: &Path) {
    if let Err(error) = fs::remove_dir_all(run_folder) {
        log::debug!("{}: kept: {error}", run_folder.display());
    }
    // Another run alive at once may still be using `.staging/`; it removes the folder when it
    // finishes, so a refusal here is expected.
    let staging = output.join(STAGING);
    if let Err(error) = fs::remove_dir(&staging) {
        log::debug!("{}: kept: {error}", staging.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::scratch;

    #[test]
    fn the_staging_folder_is_per_process_under_the_output_folder() {
        let folder = staging_folder(Path::new("out"));
        assert_eq!(folder.parent(), Some(Path::new("out/.staging")));
        let name = folder.file_name().unwrap().to_str().unwrap();
        let (pid, millis) = name.split_once('-').unwrap();
        assert_eq!(pid, std::process::id().to_string());
        assert!(millis.parse::<u128>().unwrap() > 0, "{name}");
    }

    #[test]
    fn the_output_folder_is_created_and_probed_leaving_nothing_in_it() {
        let temp = scratch("prepare_output");
        let root = temp.path();
        let output = root.join("new").join("output");

        prepare_output_folder(&output).unwrap();

        assert!(output.is_dir());
        assert_eq!(
            fs::read_dir(&output).unwrap().count(),
            0,
            "no probe file is left"
        );
    }

    #[test]
    fn an_output_folder_under_a_file_is_refused_naming_the_folder() {
        let temp = scratch("prepare_output_blocked");
        let root = temp.path();
        fs::write(root.join("blocker"), "").unwrap();
        let output = root.join("blocker").join("out");

        let error = prepare_output_folder(&output).unwrap_err();

        let text = error.to_string();
        assert!(
            text.starts_with(&format!(
                "{}: cannot create the output folder",
                output.display()
            )),
            "{text}"
        );
    }

    #[test]
    fn discarding_removes_the_run_folder_and_an_emptied_staging_folder() {
        let temp = scratch("discard");
        let output = temp.path();
        let run_folder = staging_folder(output);
        fs::create_dir_all(&run_folder).unwrap();
        fs::write(run_folder.join("cup.cpk"), "partial").unwrap();

        discard(&run_folder, output);

        assert!(!output.join(STAGING).exists());
    }

    #[test]
    fn discarding_keeps_the_staging_folder_another_run_still_uses() {
        let temp = scratch("discard_beside_another_run");
        let output = temp.path();
        let run_folder = staging_folder(output);
        let other_run = output.join(STAGING).join("1-1");
        fs::create_dir_all(&run_folder).unwrap();
        fs::create_dir_all(&other_run).unwrap();
        fs::write(run_folder.join("cup.cpk"), "partial").unwrap();

        discard(&run_folder, output);

        assert!(!run_folder.exists());
        assert!(other_run.is_dir());
    }

    #[test]
    fn promotion_replaces_the_previous_cpk_and_removes_the_run_folder() {
        let temp = scratch("promote");
        let output = temp.path();
        let stem = CpkStem::new("cup").unwrap();
        let run_folder = staging_folder(output);
        fs::create_dir_all(&run_folder).unwrap();
        fs::write(run_folder.join("cup.cpk"), "new").unwrap();
        fs::write(output.join("cup.cpk"), "previous").unwrap();

        let promoted = promote(&run_folder, output, &stem).unwrap();

        assert_eq!(promoted, output.join("cup.cpk"));
        assert_eq!(fs::read(&promoted).unwrap(), b"new");
        assert!(!run_folder.exists());
        assert!(!output.join(".staging").exists());
    }

    #[test]
    fn promotion_keeps_the_staging_folder_another_run_still_uses() {
        let temp = scratch("promote_beside_another_run");
        let output = temp.path();
        let stem = CpkStem::new("cup").unwrap();
        let run_folder = staging_folder(output);
        let other_run = output.join(".staging").join("1-1");
        fs::create_dir_all(&run_folder).unwrap();
        fs::create_dir_all(&other_run).unwrap();
        fs::write(run_folder.join("cup.cpk"), "new").unwrap();

        promote(&run_folder, output, &stem).unwrap();

        assert!(!run_folder.exists());
        assert!(other_run.is_dir());
    }
}
