//! Staging and promotion (`team_compiler/pipeline.md` "6. Post-processing"): a CPK, or a test
//! or sideload run's loose tree, is written in a staging folder of its own run and only then moved
//! to its final path, so a failed run never leaves a half-written output where the game or the
//! user could pick it up.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use pipeline::CpkStem;

/// The output subfolder that holds every run's staging folder.
const STAGING: &str = ".staging";

/// The game folder's subfolder a sideloading runtime serves to the running game (FoxDen on PES
/// 2018 to 2021, Sider on PES 2017), and the name of a sideload run's staged tree.
pub(crate) const LIVECPK: &str = "livecpk";

/// The output folder's subfolder a test run's tree replaces, and the name of the staged tree.
pub(crate) const TEST_OUTPUT: &str = "test_output";

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

/// Replaces the folder `target` (the PES folder's `livecpk/`, the output folder's
/// `test_output/`) with the tree staged at `<run_folder>/<tree>`, then discards the
/// `run_folder`. The previous tree goes first, whatever it holds: Studio is the folder's only
/// writer, and a file the export no longer has must not linger there. The staged tree is
/// renamed into place, or copied when the rename fails (the output folder on another drive than
/// the game). Nothing outside `target` and the `run_folder` is deleted. Failing to remove the
/// previous tree or to move the new one is the commit's failure; the cleanup after it is only
/// logged, as for `promote`.
pub(crate) fn promote_tree(
    run_folder: &Path,
    output: &Path,
    tree: &str,
    target: &Path,
) -> anyhow::Result<()> {
    let cannot_replace = || format!("{}: cannot replace it with the new tree", target.display());
    match fs::remove_dir_all(target) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(cannot_replace),
    }
    let staged = run_folder.join(tree);
    if let Err(error) = fs::rename(&staged, target) {
        log::debug!("{}: not renamed ({error}), copied", staged.display());
        copy_tree(&staged, target).with_context(cannot_replace)?;
    }
    discard(run_folder, output);
    Ok(())
}

/// Copies every file under `source` to the same path under `target`, folders created as
/// needed.
fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let into = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &into)?;
        } else {
            fs::copy(entry.path(), into)?;
        }
    }
    Ok(())
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

    /// A run folder under `output` holding a staged `livecpk/` tree of two files, one nested.
    fn staged_tree(output: &Path) -> PathBuf {
        let run_folder = staging_folder(output);
        let staged = run_folder.join(LIVECPK);
        fs::create_dir_all(staged.join("common/etc")).unwrap();
        fs::write(staged.join("common/etc/TeamColor.bin"), "colors").unwrap();
        fs::write(staged.join("top.bin"), "top").unwrap();
        run_folder
    }

    /// Asserts `livecpk` holds exactly `staged_tree`'s two files.
    fn assert_staged_tree(livecpk: &Path) {
        assert_eq!(
            fs::read(livecpk.join("common/etc/TeamColor.bin")).unwrap(),
            b"colors"
        );
        assert_eq!(fs::read(livecpk.join("top.bin")).unwrap(), b"top");
        assert_eq!(fs::read_dir(livecpk).unwrap().count(), 2);
    }

    #[test]
    fn the_staged_tree_replaces_livecpk_s_whole_contents_and_the_run_folder_goes() {
        let temp = scratch("promote_livecpk");
        let output = temp.path().join("output");
        let pes_folder = temp.path().join("PES");
        fs::create_dir_all(pes_folder.join("livecpk/old")).unwrap();
        fs::write(pes_folder.join("livecpk/old/old.txt"), "an earlier run").unwrap();
        fs::write(pes_folder.join("livecpk/top.bin"), "an earlier top").unwrap();
        fs::write(pes_folder.join("beside.txt"), "not ours").unwrap();
        let run_folder = staged_tree(&output);

        promote_tree(&run_folder, &output, LIVECPK, &pes_folder.join(LIVECPK)).unwrap();

        assert_staged_tree(&pes_folder.join("livecpk"));
        assert_eq!(
            fs::read(pes_folder.join("beside.txt")).unwrap(),
            b"not ours"
        );
        assert!(!output.join(STAGING).exists());
    }

    #[test]
    fn a_missing_livecpk_is_created_from_the_staged_tree() {
        let temp = scratch("promote_livecpk_new");
        let output = temp.path().join("output");
        let pes_folder = temp.path().join("PES");
        fs::create_dir_all(&pes_folder).unwrap();
        let run_folder = staged_tree(&output);

        promote_tree(&run_folder, &output, LIVECPK, &pes_folder.join(LIVECPK)).unwrap();

        assert_staged_tree(&pes_folder.join("livecpk"));
    }

    #[test]
    fn a_livecpk_that_cannot_be_removed_fails_the_promotion_naming_it() {
        let temp = scratch("promote_livecpk_blocked");
        let output = temp.path().join("output");
        let pes_folder = temp.path().join("PES");
        fs::create_dir_all(&pes_folder).unwrap();
        // A file named `livecpk` is no folder to remove.
        fs::write(pes_folder.join("livecpk"), "a file").unwrap();
        let run_folder = staged_tree(&output);

        let error =
            promote_tree(&run_folder, &output, LIVECPK, &pes_folder.join(LIVECPK)).unwrap_err();

        assert_eq!(
            error.to_string(),
            format!(
                "{}: cannot replace it with the new tree",
                pes_folder.join("livecpk").display()
            )
        );
        assert_eq!(fs::read(pes_folder.join("livecpk")).unwrap(), b"a file");
    }

    #[cfg(windows)]
    #[test]
    fn a_staged_tree_that_cannot_be_renamed_is_copied_into_place() {
        use std::os::windows::fs::OpenOptionsExt;
        /// `FILE_SHARE_READ` alone: the file may be read, so copied, but its folder not moved.
        const FILE_SHARE_READ: u32 = 1;

        let temp = scratch("promote_livecpk_copied");
        let output = temp.path().join("output");
        let pes_folder = temp.path().join("PES");
        fs::create_dir_all(&pes_folder).unwrap();
        let run_folder = staged_tree(&output);
        let held_open = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(run_folder.join("livecpk/top.bin"))
            .unwrap();

        promote_tree(&run_folder, &output, LIVECPK, &pes_folder.join(LIVECPK)).unwrap();

        drop(held_open);
        assert_staged_tree(&pes_folder.join("livecpk"));
    }

    #[cfg(windows)]
    #[test]
    fn a_previous_tree_that_cannot_be_removed_fails_the_promotion_and_is_not_overlaid() {
        use std::os::windows::fs::OpenOptionsExt;
        /// `FILE_SHARE_READ` alone: the file may be read but not deleted while it is open.
        const FILE_SHARE_READ: u32 = 1;

        let temp = scratch("promote_livecpk_held");
        let output = temp.path().join("output");
        let pes_folder = temp.path().join("PES");
        fs::create_dir_all(pes_folder.join("livecpk")).unwrap();
        fs::write(pes_folder.join("livecpk/stale.txt"), "an earlier run").unwrap();
        let run_folder = staged_tree(&output);
        // A runtime reading the previous tree holds one of its files open.
        let held_open = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(pes_folder.join("livecpk/stale.txt"))
            .unwrap();

        let error =
            promote_tree(&run_folder, &output, LIVECPK, &pes_folder.join(LIVECPK)).unwrap_err();

        drop(held_open);
        assert_eq!(
            error.to_string(),
            format!(
                "{}: cannot replace it with the new tree",
                pes_folder.join("livecpk").display()
            )
        );
        // The new tree was not copied over what is left of the old one.
        assert!(!pes_folder.join("livecpk/top.bin").exists());
    }

    #[test]
    fn copying_a_tree_copies_every_file_at_its_path() {
        let temp = scratch("copy_tree");
        let run_folder = staged_tree(temp.path());
        let target = temp.path().join("copy");

        copy_tree(&run_folder.join(LIVECPK), &target).unwrap();

        assert_staged_tree(&target);
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
