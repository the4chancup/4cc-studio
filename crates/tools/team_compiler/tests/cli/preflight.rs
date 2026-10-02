//! The preflight: the invocations and settings refused before any export is read.

use crate::common::Sandbox;
use crate::{CLEAN_PLAYER, snapshot};

// TC-CLI-03
#[test]
fn no_deploy_with_test_mode_is_an_invalid_invocation() {
    let sandbox = Sandbox::new("no_deploy_test_mode");
    let run = sandbox.run("", &["compile", "--no-deploy", "--mode", "test"]);
    run.assert_refused(2, &["--no-deploy", "--mode test", "incompatible"]);
    assert!(!sandbox.root.join("output").exists());
}

// TC-CLI-05
#[test]
fn an_invalid_cpk_name_refuses_compile_before_any_export_is_read() {
    for (index, name) in ["", "con", "a/b"].into_iter().enumerate() {
        let sandbox = Sandbox::new(&format!("cpk_name_{index}"));
        sandbox.write("exports/aaa_export/notes.txt", b"kept as it is");
        let before = snapshot(&sandbox.root);

        // Reading this exports root, which does not exist, would abort with 3.
        let run = sandbox.run(
            &format!("[team-compiler]\ncpk_name = \"{name}\"\n"),
            &["compile", &sandbox.arg("no such exports")],
        );

        run.assert_refused(2, &[&format!("cpk_name = \"{name}\"")]);
        assert_eq!(snapshot(&sandbox.root), before);
    }
}

// TC-CLI-07
#[test]
fn an_export_path_that_is_missing_or_no_archive_is_an_invalid_invocation() {
    let sandbox = Sandbox::new("export_paths");
    sandbox.write("notes.txt", b"not an export");
    for command in ["check", "compile"] {
        let missing = sandbox.arg("no_such_export");
        let run = sandbox.run("", &[command, "--export", &missing]);
        run.assert_refused(2, &[&missing, "no such folder or file"]);
        let not_archive = sandbox.arg("notes.txt");
        let run = sandbox.run("", &[command, "--export", &not_archive]);
        run.assert_refused(2, &[&not_archive, "not a folder, .zip or .7z"]);
    }
}

#[test]
fn a_setting_of_the_wrong_type_is_a_configuration_error_naming_it() {
    let sandbox = Sandbox::new("wrong_type");
    let run = sandbox.run(
        "[team-compiler]\nstrict_file_type_check = \"yes\"\n",
        &["check"],
    );
    run.assert_refused(2, &["settings [team-compiler]", "strict_file_type_check"]);
    let Err(error) = &run.result else {
        unreachable!("assert_refused checked the refusal")
    };
    let text = error.to_string();
    assert_eq!(text.trim_end(), text, "no trailing newline");
}

#[test]
fn a_memory_cap_of_zero_is_a_configuration_error() {
    let sandbox = Sandbox::new("memory_cap");
    let run = sandbox.run("[common]\nmemory_cap_percent = 0.0\n", &["check"]);
    run.assert_refused(2, &["memory_cap_percent"]);
}

#[test]
fn an_unparsable_teams_list_aborts_naming_it() {
    let sandbox = Sandbox::new("teams_list");
    sandbox.write("data/teams_list.txt", b"not a teams list\n");
    let run = sandbox.run("", &["check"]);
    run.assert_refused(3, &["teams_list.txt"]);
}

#[test]
fn modes_and_commands_this_version_lacks_are_refused() {
    let sandbox = Sandbox::new("not_available");
    let run = sandbox.run("", &["compile", "--mode", "test"]);
    run.assert_refused(2, &["--mode test", "not available yet"]);
    let run = sandbox.run("", &["compile", "--mode", "sider"]);
    run.assert_refused(2, &["--mode sider", "not available yet"]);
    let run = sandbox.run("", &["upgrade-dpfl"]);
    run.assert_refused(2, &["upgrade-dpfl", "not available yet"]);

    let multicpk = "[team-compiler]\nmulticpk_mode = true\n";
    let run = sandbox.run(multicpk, &["compile"]);
    run.assert_refused(2, &["multicpk_mode", "not available yet"]);
    // `check` ignores multicpk_mode.
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"");
    assert_eq!(sandbox.run(multicpk, &["check"]).exit_code(), 0);
}
