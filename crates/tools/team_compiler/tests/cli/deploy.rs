//! `compile`'s deployment into the PES folder's `download/` (`team_compiler/pipeline.md` "6.
//! Post-processing"): the checks made before any export is read, the copy and rename over the
//! old CPK, and the degraded run that leaves the CPK in the output folder instead.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::bins::install_names;
use crate::common::{Run, Sandbox};
use crate::compile::{cpk_entries, pes21_settings, tracer_export};
use crate::{bundled_bins_then, snapshot};

/// The findings of a PES 21 `compile` of the tracer with bundled bins, after the bins' own.
const TRACER_FINDINGS: [&str; 6] = [
    "egg Midcup Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)",
    "egg Midcup Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
    "egg Midcup Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)",
    "egg Midcup Tracer: Info export_identified [Keep] (team=/egg/, id=792)",
    "Warning player_table_missing [Keep] (table=BootsList.bin, rows=1)",
    "Warning player_table_missing [Keep] (table=GloveList.bin, rows=1)",
];

/// Writes the sandbox's PES 21 install: `PES/PES2021.exe` and a `DpFileList.bin` listing only
/// the run's CPK, so the bins are the bundled ones and nothing is listed before it.
fn install_pes(sandbox: &Sandbox) {
    sandbox.write("PES/PES2021.exe", b"the game");
    install_names(sandbox, &["4cc_99_test.cpk"]);
}

/// A deploying `compile` of the tracer in `sandbox`.
fn compile_tracer(sandbox: &Sandbox) -> Run {
    sandbox.run(
        &pes21_settings(sandbox),
        &["compile", "--export", &tracer_export()],
    )
}

/// The CPK a `--no-deploy` compile of the tracer leaves in the output folder of a sandbox of
/// its own, `name`.
fn twin_cpk(name: &str) -> PathBuf {
    let twin = Sandbox::new(name);
    let run = twin.run(
        &pes21_settings(&twin),
        &["compile", "--no-deploy", "--export", &tracer_export()],
    );
    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    twin.root.join("output/4cc_99_test.cpk")
}

/// The path, as the findings print it, the CPK of a degraded run is promoted to.
fn promoted(sandbox: &Sandbox) -> String {
    sandbox.display("output/4cc_99_test.cpk")
}

/// The names of the files in the folder `relative`, sorted.
fn file_names(sandbox: &Sandbox, relative: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(sandbox.root.join(relative))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

/// Asserts the run's CPK is in the output folder: the bytes of the `twin` CPK.
fn assert_promoted(sandbox: &Sandbox, twin: &Path) {
    let promoted = fs::read(sandbox.root.join("output/4cc_99_test.cpk")).unwrap();
    assert!(
        promoted == fs::read(twin).unwrap(),
        "the output folder holds the run's CPK"
    );
}

// TC-DEP-01
#[test]
fn compile_installs_the_cpk_over_the_old_one_and_leaves_nothing_in_the_output_folder() {
    let twin = twin_cpk("deploy_installs_twin");
    let sandbox = Sandbox::new("deploy_installs");
    install_pes(&sandbox);
    sandbox.write("PES/download/4cc_99_test.cpk", b"the old CPK");

    let run = compile_tracer(&sandbox);

    assert_eq!(run.messages(), bundled_bins_then(TRACER_FINDINGS));
    assert_eq!(run.exit_code(), 0);
    let installed = sandbox.root.join("PES/download/4cc_99_test.cpk");
    assert_eq!(cpk_entries(&installed), cpk_entries(&twin));
    // Byte for byte too: the writer's output depends on the entries alone.
    assert!(fs::read(&installed).unwrap() == fs::read(&twin).unwrap());
    assert_eq!(
        file_names(&sandbox, "PES/download"),
        ["4cc_99_test.cpk", "DpFileList.bin"]
    );
    assert_eq!(
        snapshot(&sandbox.root.join("output")),
        BTreeMap::<PathBuf, Vec<u8>>::new(),
        "no CPK and no .staging in the output folder"
    );
}

#[cfg(windows)]
// TC-DEP-02
#[test]
fn an_old_cpk_held_open_is_old_cpk_locked_and_the_cpk_is_promoted() {
    // Windows only: Linux renames over a file another process holds open.
    use std::os::windows::fs::OpenOptionsExt;
    /// `FILE_SHARE_READ` alone, as PES holds its CPKs: read, not deleted or replaced.
    const FILE_SHARE_READ: u32 = 1;

    let twin = twin_cpk("deploy_locked_twin");
    let sandbox = Sandbox::new("deploy_locked");
    install_pes(&sandbox);
    sandbox.write("PES/download/4cc_99_test.cpk", b"the old CPK");
    let held_open = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(sandbox.root.join("PES/download/4cc_99_test.cpk"))
        .unwrap();

    let run = compile_tracer(&sandbox);

    drop(held_open);
    let lines = run.messages();
    let (last, first) = lines.split_last().unwrap();
    assert_eq!(first, bundled_bins_then(TRACER_FINDINGS));
    // The error ends with the platform's own text, so only its shape is fixed.
    let prefix = format!(
        "Error old_cpk_locked [Keep] (path={}, error=",
        sandbox.display("PES/download/4cc_99_test.cpk")
    );
    let suffix = format!(", output={})", promoted(&sandbox));
    assert!(
        last.starts_with(&prefix) && last.ends_with(&suffix),
        "{last}"
    );
    assert_eq!(run.exit_code(), 1);
    assert_promoted(&sandbox, &twin);
    assert_eq!(
        fs::read(sandbox.root.join("PES/download/4cc_99_test.cpk")).unwrap(),
        b"the old CPK"
    );
    assert_eq!(
        file_names(&sandbox, "PES/download"),
        ["4cc_99_test.cpk", "DpFileList.bin"],
        "no .partial is left"
    );
}

// TC-DEP-03
#[test]
fn without_a_pes_folder_the_run_is_pes_folder_not_found_and_the_cpk_is_promoted() {
    let twin = twin_cpk("deploy_no_pes_twin");
    let sandbox = Sandbox::new("deploy_no_pes");

    let run = compile_tracer(&sandbox);

    let mut expected = vec![format!(
        "Error pes_folder_not_found [Keep] (path={}, output={})",
        sandbox.display("PES"),
        promoted(&sandbox)
    )];
    // Every bin is the bundled one: there is no list to walk.
    expected.extend(bundled_bins_then(TRACER_FINDINGS));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 1);
    assert_promoted(&sandbox, &twin);
    assert!(!sandbox.root.join("PES").exists());
}

#[cfg(windows)]
// TC-DEP-04
#[test]
fn a_download_folder_that_denies_writes_is_reported_before_any_export_is_read() {
    // Windows only: the remote mutation runs build as root on Linux, and root ignores
    // permissions, so a read-only file would not deny the write there.
    let twin = twin_cpk("deploy_unwritable_twin");
    let sandbox = Sandbox::new("deploy_unwritable");
    install_pes(&sandbox);
    sandbox.write("PES/download/4cc_99_test.cpk", b"the old CPK");
    let old = sandbox.root.join("PES/download/4cc_99_test.cpk");
    let mut permissions = fs::metadata(&old).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&old, permissions).unwrap();

    let run = compile_tracer(&sandbox);

    let lines = run.messages();
    let prefix = format!(
        "Error deploy_target_unwritable [Keep] (path={}, error=",
        sandbox.display("PES/download/4cc_99_test.cpk")
    );
    let suffix = format!(", output={})", promoted(&sandbox));
    let unwritable = lines
        .iter()
        .position(|line| line.starts_with(&prefix) && line.ends_with(&suffix))
        .unwrap_or_else(|| panic!("no deploy_target_unwritable: {lines:#?}"));
    let first_export = lines
        .iter()
        .position(|line| line.starts_with("egg Midcup Tracer: "))
        .unwrap();
    assert!(unwritable < first_export, "{lines:#?}");
    assert_eq!(run.exit_code(), 1);
    assert_promoted(&sandbox, &twin);
    assert_eq!(fs::read(&old).unwrap(), b"the old CPK");
}

// TC-DEP-05
#[test]
fn a_list_not_naming_the_cpk_is_cpk_name_unlisted_and_the_cpk_is_promoted() {
    let twin = twin_cpk("deploy_unlisted_twin");
    let sandbox = Sandbox::new("deploy_unlisted");
    sandbox.write("PES/PES2021.exe", b"the game");
    install_names(&sandbox, &["4cc_80_mine.cpk"]);
    let download = snapshot(&sandbox.root.join("PES/download"));

    let run = compile_tracer(&sandbox);

    let mut expected = vec![format!(
        "Error cpk_name_unlisted [Keep] (cpk=4cc_99_test.cpk, path={}, output={})",
        sandbox.display("PES/download/DpFileList.bin"),
        promoted(&sandbox)
    )];
    expected.extend(bundled_bins_then(TRACER_FINDINGS));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 1);
    assert_promoted(&sandbox, &twin);
    assert_eq!(snapshot(&sandbox.root.join("PES/download")), download);
}

// TC-DEP-06
#[test]
fn another_version_s_exe_is_a_warning_and_the_cpk_is_installed() {
    let sandbox = Sandbox::new("deploy_version_mismatch");
    install_names(&sandbox, &["4cc_99_test.cpk"]);
    sandbox.write("PES/PES2019.exe", b"another game");

    let run = compile_tracer(&sandbox);

    let mut expected = vec![format!(
        "Warning pes_version_mismatch [Keep] (path={}, exe=PES2021.exe)",
        sandbox.display("PES")
    )];
    expected.extend(bundled_bins_then(TRACER_FINDINGS));
    assert_eq!(run.messages(), expected);
    assert_eq!(run.exit_code(), 0);
    assert!(sandbox.root.join("PES/download/4cc_99_test.cpk").is_file());
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());
}

// TC-DEP-07
#[test]
fn a_dead_run_s_staging_folder_is_removed_when_compile_starts() {
    let sandbox = Sandbox::new("deploy_dead_staging");
    install_pes(&sandbox);
    // A killed run's folder: no lock file says it is alive.
    sandbox.write("output/.staging/1-1/x.cpk", b"half a CPK");

    let run = compile_tracer(&sandbox);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert!(!sandbox.root.join("output/.staging").exists());
    assert!(sandbox.root.join("PES/download/4cc_99_test.cpk").is_file());
}
