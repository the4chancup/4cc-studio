//! Staging, promotion and deployment (`team_compiler/pipeline.md` "6. Post-processing"): a CPK,
//! or a test or sideload run's loose tree, is written in a staging folder of its own run and only
//! then moved to its final path, so a failed run never leaves a half-written output where the
//! game or the user could pick it up. A run that deploys checks, before any export is read, that
//! it can install into the PES folder's `download/`, and installs its CPK there by a copy and a
//! rename.

use std::collections::BTreeSet;
use std::fs::{self, File, TryLockError};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use pes_version::PesVersion;
use pipeline::CpkStem;
use studio_core::{Disposition, Message, Scope};

use crate::bins::installed;
use crate::messages::{Code, tool_message};

/// The output subfolder that holds every run's staging folder.
const STAGING: &str = ".staging";

/// The game folder's subfolder a sideloading runtime serves to the running game (FoxDen on PES
/// 2018 to 2021, Sider on PES 2017), and the name of a sideload run's staged tree.
pub(crate) const LIVECPK: &str = "livecpk";

/// The output folder's subfolder a test run's tree replaces, and the name of the staged tree.
pub(crate) const TEST_OUTPUT: &str = "test_output";

/// The game folder's subfolder the game loads the CPKs `DpFileList.bin` lists from.
const DOWNLOAD: &str = "download";

/// The list of the CPKs the game loads, in `download/`.
const DPFILELIST: &str = "DpFileList.bin";

/// This run's id, `<pid>-<unix ms>`: the process id keeps two runs at once apart, the time two
/// runs of a recycled process id.
fn run_id() -> String {
    // A clock set before 1970 leaves only the process id to tell runs apart, which is enough
    // for the runs alive at once.
    let millis = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_millis(),
        Err(_) => 0,
    };
    format!("{}-{millis}", std::process::id())
}

/// The name of the lock file of the run `run_id`, beside its folder in `.staging/`.
fn lock_name(run_id: &str) -> String {
    format!("{run_id}.lock")
}

/// A run's staging: its folder `<output>/.staging/<run_id>`, which the run's sink creates with
/// the first file it writes, and the exclusive lock held on `<output>/.staging/<run_id>.lock`
/// for the run's whole life, which tells every other run that the folder is in use. The OS
/// releases the lock however the process ends, so a killed run's folder is known to be dead by
/// the next run, which removes it (`Staging::create`).
///
/// Dropping the value removes the folder with whatever is still in it, then releases the lock
/// and removes its file, then `.staging/` when no other run is in it: every way a run ends, a
/// panic included, leaves nothing in the output folder but what it promoted. A failure to
/// remove is logged, not returned: a run that failed already carries the error that matters,
/// and a promoted one has its output in place.
pub(crate) struct Staging {
    /// `<output>/.staging`.
    root: PathBuf,
    /// `<root>/<run_id>`, the run's own folder.
    folder: PathBuf,
    /// `<root>/<run_id>.lock`.
    lock_path: PathBuf,
    /// The handle holding the lock; taken when the lock is released.
    lock: Option<File>,
}

impl Staging {
    /// Creates `<output>/.staging/` and this run's lock file, and locks it, before anything is
    /// written in this run's folder; then removes there every other run's staging that is dead
    /// (a run killed before it cleaned up: `sweep`). Failing to create the folder or to lock the
    /// file is the error, naming the path.
    pub(crate) fn create(output: &Path) -> anyhow::Result<Staging> {
        let root = output.join(STAGING);
        fs::create_dir_all(&root)
            .with_context(|| format!("{}: cannot create the staging folder", root.display()))?;
        let run_id = run_id();
        let lock_path = root.join(lock_name(&run_id));
        let cannot_lock = || format!("{}: cannot lock the run's staging", lock_path.display());
        let lock = File::create_new(&lock_path).with_context(cannot_lock)?;
        lock.try_lock()
            .map_err(io::Error::from)
            .with_context(cannot_lock)?;
        // Swept with the lock held: a run ending meanwhile removes `.staging/` only while it is
        // empty, and the held lock is also what makes the sweep leave this run's own staging.
        sweep(&root);
        Ok(Staging {
            folder: root.join(&run_id),
            root,
            lock_path,
            lock: Some(lock),
        })
    }

    /// The run's own folder, where its output is staged.
    pub(crate) fn folder(&self) -> &Path {
        &self.folder
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.folder) {
            log::debug!("{}: kept: {error}", self.folder.display());
        }
        // Released only now: while it is held, a run starting meanwhile leaves the folder to
        // this one.
        drop(self.lock.take());
        if let Err(error) = fs::remove_file(&self.lock_path) {
            log::debug!("{}: kept: {error}", self.lock_path.display());
        }
        // Another run alive at once may still be using `.staging/`; it removes the folder when
        // it finishes, so a refusal here is expected.
        if let Err(error) = fs::remove_dir(&self.root) {
            log::debug!("{}: kept: {error}", self.root.display());
        }
    }
}

/// Removes from `root`, the `.staging/` folder, the staging of every run that is dead: each
/// folder whose lock file is missing or can be locked, with that file, and each lock file with
/// no folder that can be locked. A run still alive holds its lock, so its folder is left, and so
/// is the lock of a run that has not created its folder yet. A failure is logged, never the
/// sweeping run's.
fn sweep(root: &Path) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) => {
            log::debug!("{}: not swept: {error}", root.display());
            return;
        }
    };
    let mut run_ids = BTreeSet::new();
    for entry in entries {
        match entry {
            Ok(entry) => {
                let name = entry.file_name().to_string_lossy().into_owned();
                let run_id = name.strip_suffix(".lock").unwrap_or(&name).to_owned();
                run_ids.insert(run_id);
            }
            Err(error) => log::debug!("{}: an entry not swept: {error}", root.display()),
        }
    }
    for run_id in run_ids {
        sweep_run(root, &run_id);
    }
}

/// Removes the staging of the run `run_id` from `root`, its folder and then its lock file,
/// unless its lock is held. Anything else of that name (a stray file) is left.
fn sweep_run(root: &Path, run_id: &str) {
    let folder = root.join(run_id);
    let lock_path = root.join(lock_name(run_id));
    // Held for the removal, so two runs sweeping at once do not both remove the folder.
    let lock = match File::open(&lock_path) {
        Ok(lock) => match lock.try_lock() {
            Ok(()) => Some(lock),
            Err(TryLockError::WouldBlock) => return,
            Err(TryLockError::Error(error)) => {
                log::debug!("{}: not swept: {error}", lock_path.display());
                return;
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            log::debug!("{}: not swept: {error}", lock_path.display());
            return;
        }
    };
    if folder.is_dir()
        && let Err(error) = fs::remove_dir_all(&folder)
    {
        log::debug!("{}: a dead run's staging, kept: {error}", folder.display());
        return;
    }
    if let Some(lock) = lock {
        drop(lock);
        if let Err(error) = fs::remove_file(&lock_path) {
            log::debug!("{}: kept: {error}", lock_path.display());
        }
    }
}

/// The CPK file name for `cpk_stem`.
pub(crate) fn cpk_file_name(cpk_stem: &CpkStem) -> String {
    format!("{}.cpk", cpk_stem.as_str())
}

/// Creates the output folder `output` if it is missing and probes it for writing. Each failure
/// names the folder, not the probe.
pub(crate) fn prepare_output_folder(output: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(output)
        .with_context(|| format!("{}: cannot create the output folder", output.display()))?;
    probe_folder(output)
        .with_context(|| format!("{}: cannot write in the output folder", output.display()))
}

/// Probes `folder` for writing: a file created in it and removed.
fn probe_folder(folder: &Path) -> io::Result<()> {
    let probe = folder.join(format!(".write-probe-{}", std::process::id()));
    fs::write(&probe, b"")?;
    fs::remove_file(&probe)
}

/// The checks a run that deploys makes before any export is read (`pipeline.md` "6.
/// Post-processing", "Destination writability preflight"), in order: the PES folder
/// `pes_folder` is a folder (`pes_folder_not_found`), it holds `version`'s exe
/// (`pes_version_mismatch`, a Warning: the checks go on), its `download/DpFileList.bin` exists
/// and lists the run's CPK (`cpk_name_unlisted`), and the CPK can be written in `download/`
/// (`deploy_target_unwritable`). The first Error ends them. A missing or unreadable list adds no
/// finding: the working-bin walk, which reads it next, reports it (`dpfilelist_missing`,
/// `installed_bin_unreadable`). Returns the `download/` folder to install into, or `None` when
/// a check failed and the CPK goes to `promoted`, its path in the output folder, which each
/// Error names; and the findings.
pub(crate) fn preflight(
    pes_folder: &Path,
    version: PesVersion,
    cpk_stem: &CpkStem,
    promoted: &Path,
) -> (Option<PathBuf>, Vec<Message>) {
    let output = || ("output", promoted.display().to_string());
    let mut messages = Vec::new();
    if !pes_folder.is_dir() {
        messages.push(tool_message(
            Code::PesFolderNotFound,
            Scope::Run,
            Disposition::Keep,
            vec![("path", pes_folder.display().to_string()), output()],
        ));
        return (None, messages);
    }
    let exe = format!("PES{}.exe", version.year());
    if !pes_folder.join(&exe).is_file() {
        messages.push(tool_message(
            Code::PesVersionMismatch,
            Scope::Run,
            Disposition::Keep,
            vec![("path", pes_folder.display().to_string()), ("exe", exe)],
        ));
    }
    let download = pes_folder.join(DOWNLOAD);
    let list_path = download.join(DPFILELIST);
    let list = match installed::read_list(&list_path) {
        Ok(Some(list)) => list,
        Ok(None) => return (None, messages),
        Err(unreadable) => {
            log::debug!(
                "{}: not deployed: {:#}",
                unreadable.path.display(),
                unreadable.error
            );
            return (None, messages);
        }
    };
    let name = cpk_file_name(cpk_stem);
    if !list.contains(&name) {
        messages.push(tool_message(
            Code::CpkNameUnlisted,
            Scope::Run,
            Disposition::Keep,
            vec![
                ("cpk", name),
                ("path", list_path.display().to_string()),
                output(),
            ],
        ));
        return (None, messages);
    }
    if let Err((path, error)) = probe_download(&download, &name) {
        messages.push(tool_message(
            Code::DeployTargetUnwritable,
            Scope::Run,
            Disposition::Keep,
            vec![
                ("path", path.display().to_string()),
                ("error", error.to_string()),
                output(),
            ],
        ));
        return (None, messages);
    }
    (Some(download), messages)
}

/// Whether the CPK `name` can be written in `download`: the old CPK opened for writing, not
/// truncated, when there is one, else a probe file created and removed in the folder. The
/// failure is the path that cannot be written and the error. An old CPK that cannot be opened
/// for another reason than access denied is in use, by PES most likely, which may be closed
/// before the run ends: the deployment meets the lock if it is not, so the probe passes.
fn probe_download(download: &Path, name: &str) -> Result<(), (PathBuf, io::Error)> {
    let old = download.join(name);
    match fs::OpenOptions::new().write(true).open(&old) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            probe_folder(download).map_err(|error| (download.to_owned(), error))
        }
        Err(error) if elevation::is_access_denied(&error) => Err((old, error)),
        Err(error) => {
            log::debug!("{}: in use, not probed: {error}", old.display());
            Ok(())
        }
    }
}

/// Why installing the CPK into `download/` failed, named by the step that failed and not by
/// the OS error (`pipeline.md` "Deploy CPKs"): Windows reports a file held open without delete
/// sharing as access denied, the error of a folder that needs elevation, while only the rename
/// needs the old CPK free.
#[derive(Debug)]
pub(crate) enum DeployFailure {
    /// The copy to `{name}.cpk.partial` failed: the folder denies writes
    /// (`deploy_target_unwritable`).
    Copy(io::Error),
    /// The rename of the `.partial` over the old CPK failed: the old CPK is in use, by PES
    /// most likely (`old_cpk_locked`).
    Rename(io::Error),
}

/// Installs the CPK staged in `staging` as `<download>/<cpk_stem>.cpk`: copied to
/// `<cpk_stem>.cpk.partial` (a copy: the output folder is usually on another volume than the
/// game), then renamed over the old CPK, which on one volume is atomic, so the game finds the
/// old CPK or the whole new one. A failure removes the `.partial` (a failure to remove it is
/// logged) and leaves the old CPK as it was; the staged CPK stays, for the output folder.
pub(crate) fn deploy(
    staging: &Staging,
    download: &Path,
    cpk_stem: &CpkStem,
) -> Result<(), DeployFailure> {
    let name = cpk_file_name(cpk_stem);
    let partial = download.join(format!("{name}.partial"));
    if let Err(error) = fs::copy(staging.folder.join(&name), &partial) {
        remove_partial(&partial);
        return Err(DeployFailure::Copy(error));
    }
    if let Err(error) = fs::rename(&partial, download.join(&name)) {
        remove_partial(&partial);
        return Err(DeployFailure::Rename(error));
    }
    Ok(())
}

/// Removes a failed deployment's `.partial`; a failure is logged.
fn remove_partial(partial: &Path) {
    if let Err(error) = fs::remove_file(partial) {
        log::debug!("{}: kept: {error}", partial.display());
    }
}

/// Moves the CPK staged in `staging` to `<output>/<cpk_stem>.cpk`, replacing a previous one.
/// The rename stays on one volume, so the final path holds either the previous CPK or the
/// whole new one, never part of it. Returns the promoted path.
pub(crate) fn promote(
    staging: &Staging,
    output: &Path,
    cpk_stem: &CpkStem,
) -> anyhow::Result<PathBuf> {
    let name = cpk_file_name(cpk_stem);
    let promoted = output.join(&name);
    fs::rename(staging.folder.join(&name), &promoted)
        .with_context(|| format!("{}: cannot replace it with the new CPK", promoted.display()))?;
    Ok(promoted)
}

/// Replaces the folder `target` (the PES folder's `livecpk/`, the output folder's
/// `test_output/`) with the tree staged at `<staging folder>/<tree>`. The previous tree goes
/// first, whatever it holds: Studio is the folder's only writer, and a file the export no longer
/// has must not linger there. The staged tree is renamed into place, or copied when the rename
/// fails (the output folder on another drive than the game). Nothing outside `target` is
/// deleted. Failing to remove the previous tree or to move the new one is the error.
pub(crate) fn promote_tree(staging: &Staging, tree: &str, target: &Path) -> anyhow::Result<()> {
    let cannot_replace = || format!("{}: cannot replace it with the new tree", target.display());
    match fs::remove_dir_all(target) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).with_context(cannot_replace),
    }
    let staged = staging.folder.join(tree);
    if let Err(error) = fs::rename(&staged, target) {
        log::debug!("{}: not renamed ({error}), copied", staged.display());
        copy_tree(&staged, target).with_context(cannot_replace)?;
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::scratch;

    /// A staging under `output` whose folder holds the CPK `cup.cpk` with `bytes`.
    fn staged_cpk(output: &Path, bytes: &[u8]) -> Staging {
        let staging = Staging::create(output).unwrap();
        fs::create_dir_all(staging.folder()).unwrap();
        fs::write(staging.folder().join("cup.cpk"), bytes).unwrap();
        staging
    }

    /// Whether the lock file at `path` is held: a second handle cannot take it.
    fn is_held(path: &Path) -> bool {
        match File::open(path).unwrap().try_lock() {
            Ok(()) => false,
            Err(TryLockError::WouldBlock) => true,
            Err(TryLockError::Error(error)) => panic!("{}: {error}", path.display()),
        }
    }

    #[test]
    fn the_staging_folder_is_per_process_under_the_output_folder() {
        let temp = scratch("staging_folder");
        let output = temp.path();

        let staging = Staging::create(output).unwrap();

        let folder = staging.folder();
        assert_eq!(folder.parent(), Some(output.join(STAGING).as_path()));
        let name = folder.file_name().unwrap().to_str().unwrap();
        let (pid, millis) = name.split_once('-').unwrap();
        assert_eq!(pid, std::process::id().to_string());
        assert!(millis.parse::<u128>().unwrap() > 0, "{name}");
    }

    #[test]
    fn a_run_s_own_lock_is_held_until_its_folder_is_gone() {
        let temp = scratch("staging_own_lock");
        let output = temp.path();
        let staging = staged_cpk(output, b"partial");
        let folder = staging.folder().to_owned();
        let lock = output.join(STAGING).join(format!(
            "{}.lock",
            folder.file_name().unwrap().to_str().unwrap()
        ));
        assert!(is_held(&lock), "held while the run lives");

        drop(staging);

        assert!(!folder.exists());
        assert!(!lock.exists());
        assert!(!output.join(STAGING).exists(), "an emptied .staging goes");
    }

    #[test]
    fn a_staging_that_cannot_be_created_is_an_error_naming_the_folder() {
        let temp = scratch("staging_blocked");
        let output = temp.path();
        fs::write(output.join(STAGING), "in the way").unwrap();

        let Err(error) = Staging::create(output) else {
            panic!("created over a file");
        };

        assert!(
            error.to_string().starts_with(&format!(
                "{}: cannot create the staging folder",
                output.join(STAGING).display()
            )),
            "{error}"
        );
    }

    #[test]
    fn starting_a_run_removes_dead_runs_staging_and_keeps_live_runs() {
        let temp = scratch("staging_sweep");
        let output = temp.path();
        let root = output.join(STAGING);
        for folder in ["1-1", "2-2", "3-3"] {
            fs::create_dir_all(root.join(folder)).unwrap();
            fs::write(root.join(folder).join("x.cpk"), "half a CPK").unwrap();
        }
        // 1-1 has no lock; 2-2's and the lone 4-4's are free; 3-3's and the lone 5-5's are held,
        // the latter by a run that has not created its folder yet.
        for lock in ["2-2.lock", "3-3.lock", "4-4.lock", "5-5.lock"] {
            fs::write(root.join(lock), "").unwrap();
        }
        let held: Vec<File> = ["3-3.lock", "5-5.lock"]
            .iter()
            .map(|lock| {
                let file = File::open(root.join(lock)).unwrap();
                file.try_lock().unwrap();
                file
            })
            .collect();

        let staging = Staging::create(output).unwrap();

        let mut left: Vec<String> = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        let own = format!(
            "{}.lock",
            staging.folder().file_name().unwrap().to_str().unwrap()
        );
        let mut expected = vec![
            "3-3".to_owned(),
            "3-3.lock".to_owned(),
            "5-5.lock".to_owned(),
        ];
        expected.push(own);
        expected.sort();
        assert_eq!(left, expected);
        assert!(root.join("3-3/x.cpk").is_file());
        drop(held);
    }

    #[test]
    fn probing_a_download_folder_that_does_not_exist_fails_naming_the_folder() {
        let temp = scratch("probe_missing_download");
        let download = temp.path().join("download");

        let Err((path, error)) = probe_download(&download, "cup.cpk") else {
            panic!("a folder that does not exist was probed as writable");
        };

        assert_eq!(path, download);
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
    }

    // Windows only: there a folder cannot be opened as a file, the one portable way to make the
    // lock exist and not open; Linux opens and locks a folder.
    #[cfg(windows)]
    #[test]
    fn a_staging_whose_lock_cannot_be_opened_is_left() {
        let temp = scratch("staging_lock_unopenable");
        let output = temp.path();
        let root = output.join(STAGING);
        fs::create_dir_all(root.join("9-9")).unwrap();
        fs::write(root.join("9-9/x.cpk"), "half a CPK").unwrap();
        fs::create_dir_all(root.join("9-9.lock")).unwrap();

        let _staging = Staging::create(output).unwrap();

        assert!(root.join("9-9/x.cpk").is_file());
    }

    #[test]
    fn dropping_a_staging_keeps_the_staging_folder_another_run_still_uses() {
        let temp = scratch("staging_beside_another_run");
        let output = temp.path();
        let staging = staged_cpk(output, b"partial");
        let other_run = output.join(STAGING).join("1-1");
        fs::create_dir_all(&other_run).unwrap();
        let folder = staging.folder().to_owned();

        drop(staging);

        assert!(!folder.exists());
        assert!(other_run.is_dir());
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
    fn deploying_copies_the_staged_cpk_over_the_old_one() {
        let temp = scratch("deploy");
        let output = temp.path().join("output");
        let download = temp.path().join("download");
        fs::create_dir_all(&download).unwrap();
        fs::write(download.join("cup.cpk"), "old").unwrap();
        let staging = staged_cpk(&output, b"new");

        deploy(&staging, &download, &CpkStem::new("cup").unwrap()).unwrap();

        assert_eq!(fs::read(download.join("cup.cpk")).unwrap(), b"new");
        assert_eq!(fs::read_dir(&download).unwrap().count(), 1, "no .partial");
        assert_eq!(
            fs::read(staging.folder().join("cup.cpk")).unwrap(),
            b"new",
            "a copy: the staged CPK stays until the staging goes"
        );
    }

    #[test]
    fn a_failing_copy_is_the_unwritable_kind_and_leaves_no_partial() {
        let temp = scratch("deploy_copy_failed");
        let output = temp.path().join("output");
        // No `download/` folder: the copy cannot create the `.partial`.
        let download = temp.path().join("download");
        let staging = staged_cpk(&output, b"new");

        let failure = deploy(&staging, &download, &CpkStem::new("cup").unwrap()).unwrap_err();

        assert!(matches!(failure, DeployFailure::Copy(_)), "{failure:?}");
        assert!(!download.exists(), "nothing is written: {failure:?}");
        assert!(staging.folder().join("cup.cpk").is_file());
    }

    #[test]
    fn a_failing_rename_is_the_locked_kind_and_leaves_no_partial() {
        let temp = scratch("deploy_rename_failed");
        let output = temp.path().join("output");
        let download = temp.path().join("download");
        // A non-empty folder at the old CPK's path: nothing can be renamed over it.
        fs::create_dir_all(download.join("cup.cpk")).unwrap();
        fs::write(download.join("cup.cpk/kept.txt"), "kept").unwrap();
        let staging = staged_cpk(&output, b"new");

        let failure = deploy(&staging, &download, &CpkStem::new("cup").unwrap()).unwrap_err();

        assert!(matches!(failure, DeployFailure::Rename(_)), "{failure:?}");
        assert!(!download.join("cup.cpk.partial").exists());
        assert_eq!(fs::read_dir(&download).unwrap().count(), 1);
        assert_eq!(
            fs::read(download.join("cup.cpk/kept.txt")).unwrap(),
            b"kept"
        );
    }

    #[test]
    fn promotion_replaces_the_previous_cpk_and_the_staging_goes_with_the_run() {
        let temp = scratch("promote");
        let output = temp.path();
        let stem = CpkStem::new("cup").unwrap();
        let staging = staged_cpk(output, b"new");
        fs::write(output.join("cup.cpk"), "previous").unwrap();

        let promoted = promote(&staging, output, &stem).unwrap();
        drop(staging);

        assert_eq!(promoted, output.join("cup.cpk"));
        assert_eq!(fs::read(&promoted).unwrap(), b"new");
        assert!(!output.join(STAGING).exists());
    }

    /// A staging under `output` holding a staged `livecpk/` tree of two files, one nested.
    fn staged_tree(output: &Path) -> Staging {
        let staging = Staging::create(output).unwrap();
        let staged = staging.folder().join(LIVECPK);
        fs::create_dir_all(staged.join("common/etc")).unwrap();
        fs::write(staged.join("common/etc/TeamColor.bin"), "colors").unwrap();
        fs::write(staged.join("top.bin"), "top").unwrap();
        staging
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
        let staging = staged_tree(&output);

        promote_tree(&staging, LIVECPK, &pes_folder.join(LIVECPK)).unwrap();
        drop(staging);

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
        let staging = staged_tree(&output);

        promote_tree(&staging, LIVECPK, &pes_folder.join(LIVECPK)).unwrap();

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
        let staging = staged_tree(&output);

        let error = promote_tree(&staging, LIVECPK, &pes_folder.join(LIVECPK)).unwrap_err();

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
        let staging = staged_tree(&output);
        let held_open = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(staging.folder().join("livecpk/top.bin"))
            .unwrap();

        promote_tree(&staging, LIVECPK, &pes_folder.join(LIVECPK)).unwrap();

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
        let staging = staged_tree(&output);
        // A runtime reading the previous tree holds one of its files open.
        let held_open = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(pes_folder.join("livecpk/stale.txt"))
            .unwrap();

        let error = promote_tree(&staging, LIVECPK, &pes_folder.join(LIVECPK)).unwrap_err();

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
        let staging = staged_tree(temp.path());
        let target = temp.path().join("copy");

        copy_tree(&staging.folder().join(LIVECPK), &target).unwrap();

        assert_staged_tree(&target);
    }
}
