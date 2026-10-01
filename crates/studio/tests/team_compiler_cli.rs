//! The Team compiler's command line, run through the real binary: argument and settings
//! refusals with their exit codes (`team_compiler/settings.md` "CLI"), and the binary's own
//! startup errors.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A fresh folder holding a copy of the binary and a `data/settings.toml`, so the portable data
/// location is found beside the executable and no settings file elsewhere on the machine is read.
struct Sandbox {
    root: PathBuf,
    exe: PathBuf,
}

impl Sandbox {
    fn new(name: &str, settings: &str) -> Sandbox {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join("team_compiler_cli")
            .join(name);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(root.join("data")).unwrap();
        let built = Path::new(env!("CARGO_BIN_EXE_4cc-studio"));
        let exe = root.join(built.file_name().unwrap());
        fs::copy(built, &exe).unwrap();
        fs::write(root.join("data/settings.toml"), settings).unwrap();
        Sandbox { root, exe }
    }

    /// Runs the copied binary with the sandbox as the current directory.
    fn run(&self, args: &[&str]) -> Output {
        Command::new(&self.exe)
            .args(args)
            .current_dir(&self.root)
            .output()
            .unwrap()
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn assert_refused(output: &Output, exit_code: i32, names: &[&str]) {
    let text = stderr(output);
    assert_eq!(output.status.code(), Some(exit_code), "stderr: {text}");
    for name in names {
        assert!(text.contains(name), "stderr does not name {name}: {text}");
    }
}

// TC-CLI-03
#[test]
fn no_deploy_with_test_mode_is_an_invalid_invocation() {
    let sandbox = Sandbox::new("no_deploy_test_mode", "");
    let output = sandbox.run(&["team-compiler", "compile", "--no-deploy", "--mode", "test"]);
    assert_refused(&output, 2, &["--no-deploy", "--mode"]);
    assert!(!sandbox.root.join("output").exists());
}

// TC-CLI-05
#[test]
fn an_invalid_cpk_name_refuses_compile_before_any_export_is_read() {
    for (index, name) in ["", "con", "a/b"].into_iter().enumerate() {
        let settings = format!("[team-compiler]\ncpk_name = \"{name}\"\n");
        let sandbox = Sandbox::new(&format!("cpk_name_{index}"), &settings);
        sandbox.write("exports/aaa_export/notes.txt", "kept as it is");

        let output = sandbox.run(&["team-compiler", "compile"]);

        assert_refused(&output, 2, &["cpk_name"]);
        let export = sandbox.root.join("exports/aaa_export");
        let entries: Vec<_> = fs::read_dir(&export)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert_eq!(entries, ["notes.txt"]);
        assert_eq!(
            fs::read_to_string(export.join("notes.txt")).unwrap(),
            "kept as it is"
        );
        assert!(!sandbox.root.join("output").exists());
    }
}

// TC-CLI-07
#[test]
fn an_export_path_that_is_missing_or_no_archive_is_an_invalid_invocation() {
    let sandbox = Sandbox::new("export_paths", "");
    sandbox.write("notes.txt", "not an export");
    for command in ["check", "compile"] {
        let output = sandbox.run(&["team-compiler", command, "--export", "no_such_export"]);
        assert_refused(&output, 2, &["no_such_export"]);
        let output = sandbox.run(&["team-compiler", command, "--export", "notes.txt"]);
        assert_refused(&output, 2, &["notes.txt"]);
    }
}

#[test]
fn a_setting_of_the_wrong_type_is_a_configuration_error_naming_it() {
    let sandbox = Sandbox::new(
        "wrong_type",
        "[team-compiler]\nstrict_file_type_check = \"yes\"\n",
    );
    let output = sandbox.run(&["team-compiler", "check"]);
    assert_refused(&output, 2, &["strict_file_type_check"]);
}

#[test]
fn a_memory_cap_of_zero_is_a_configuration_error() {
    let sandbox = Sandbox::new("memory_cap", "[common]\nmemory_cap_percent = 0.0\n");
    let output = sandbox.run(&["team-compiler", "check"]);
    assert_refused(&output, 2, &["memory_cap_percent"]);
}

#[test]
fn an_unparsable_teams_list_aborts_naming_it() {
    let sandbox = Sandbox::new("teams_list", "");
    sandbox.write("data/teams_list.txt", "not a teams list\n");
    let output = sandbox.run(&["team-compiler", "check"]);
    assert_refused(&output, 3, &["teams_list.txt"]);
}

#[test]
fn modes_and_commands_this_version_lacks_are_refused() {
    let sandbox = Sandbox::new("not_available", "");
    for args in [
        ["team-compiler", "compile", "--mode", "test"].as_slice(),
        ["team-compiler", "compile", "--mode", "sider"].as_slice(),
        ["team-compiler", "upgrade-dpfl"].as_slice(),
    ] {
        assert_refused(&sandbox.run(args), 2, &["not available yet"]);
    }

    let multicpk = Sandbox::new("multicpk", "[team-compiler]\nmulticpk_mode = true\n");
    let output = multicpk.run(&["team-compiler", "compile"]);
    assert_refused(&output, 2, &["multicpk_mode", "not available yet"]);
    // `check` ignores multicpk_mode: it reaches the stub past the preflight.
    let output = multicpk.run(&["team-compiler", "check"]);
    assert_refused(&output, 3, &["check is not built yet"]);
}

#[test]
fn a_valid_setup_passes_the_preflight() {
    let sandbox = Sandbox::new("valid", "");
    sandbox.write("exports/aaa_export/notes.txt", "");
    sandbox.write("pack.ZIP", "");
    let output = sandbox.run(&[
        "team-compiler",
        "check",
        "--export",
        "exports/aaa_export",
        "--export",
        "pack.ZIP",
    ]);
    assert_refused(&output, 3, &["check is not built yet"]);
    let output = sandbox.run(&["team-compiler", "compile", "--mode", "normal"]);
    assert_refused(&output, 3, &["compile is not built yet"]);
    assert!(!sandbox.root.join("output").exists());
}

#[test]
fn a_settings_file_that_is_not_toml_stops_the_binary_naming_it() {
    let sandbox = Sandbox::new("bad_settings", "this is = = not toml");
    let output = sandbox.run(&["team-compiler", "check"]);
    assert_refused(&output, 2, &["settings.toml"]);
}

#[test]
fn the_help_lists_the_commands_and_exits_zero() {
    let sandbox = Sandbox::new("help", "");
    let output = sandbox.run(&["team-compiler", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    let text = String::from_utf8_lossy(&output.stdout);
    for command in ["compile", "check", "upgrade-dpfl"] {
        assert!(text.contains(command), "help lacks {command}: {text}");
    }
}
