//! The Team compiler's command line, run through the real binary: what only the binary does,
//! which is loading the settings file, printing the console lines and a refusal's `error: ` line,
//! and turning the tool's verdict into the process exit code. The tool's own refusals and
//! findings are tested in-process, in the `team_compiler` crate's `tests/cli/`.

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
        self.run_from(&self.root, args)
    }

    /// Runs the copied binary with `current_dir` as the current directory.
    fn run_from(&self, current_dir: &Path, args: &[&str]) -> Output {
        Command::new(&self.exe)
            .args(args)
            .current_dir(current_dir)
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

#[test]
fn a_refusal_prints_one_error_line_and_exits_with_its_code() {
    let sandbox = Sandbox::new("refusal", "");
    let output = sandbox.run(&["team-compiler", "compile", "--no-deploy", "--mode", "test"]);
    assert_refused(&output, 2, &["--no-deploy"]);
    assert!(
        stderr(&output).starts_with("error: "),
        "{}",
        stderr(&output)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn check_prints_one_line_per_finding_and_exits_with_the_worst() {
    let sandbox = Sandbox::new("check", "");
    sandbox.write("data/teams_list.txt", "ID\tName\n714\t/co/\n");
    // `check` reads every model, so the clean export holds a real one with no finding: the
    // tracer's right glove.
    let glove = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../tools/team_compiler/tests/fixtures/tracer/studio/egg Tracer/Players/05 - The Chad Stormworks Player/glove_r.fmdl",
    );
    let clean = sandbox.root.join("exports/co - Clean/Players/03 - A");
    fs::create_dir_all(&clean).unwrap();
    fs::copy(&glove, clean.join("face_high.fmdl")).unwrap();
    sandbox.write("exports/co - Error/Players/03 - A/readme.txt", "");

    let output = sandbox.run(&["team-compiler", "check", "--export", "exports/co - Clean"]);
    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "- co - Clean: Info export_identified (team=/co/, id=714)\n"
    );

    let output = sandbox.run(&["team-compiler", "check", "--export", "exports/co - Error"]);
    assert_eq!(output.status.code(), Some(1), "stderr: {}", stderr(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "- co - Error: Error file_type_disallowed at Players/03 - A (file=readme.txt)\n\
         - co - Error: Info export_identified (team=/co/, id=714)\n"
    );
    assert!(stderr(&output).is_empty(), "{}", stderr(&output));
}

// TC-CLI-04
#[test]
fn compile_with_a_positional_root_compiles_it_and_leaves_the_settings_file_alone() {
    let settings = "[common]\npes_version = 21\n";
    let sandbox = Sandbox::new("positional_root", settings);
    sandbox.write("data/teams_list.txt", "ID\tName\n714\t/co/\n790\t/dbg/\n");
    let kit = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tools/team_compiler/tests/fixtures/tracer/studio/egg Tracer/Kits/g1/kit.dds");
    for folder in ["elsewhere/co - Kit/Kits/p1", "exports/dbg - Other/Kits/p1"] {
        let folder = sandbox.root.join(folder);
        fs::create_dir_all(&folder).unwrap();
        fs::copy(&kit, folder.join("kit.dds")).unwrap();
    }

    let output = sandbox.run(&["team-compiler", "compile", "elsewhere"]);

    assert_eq!(output.status.code(), Some(0), "stderr: {}", stderr(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "- co - Kit: Info export_identified (team=/co/, id=714)\n\
         - co - Kit: Info kit_config_generated at Kits/p1\n"
    );
    assert!(sandbox.root.join("output/4cc_99_test.cpk").is_file());
    assert_eq!(
        fs::read(sandbox.root.join("data/settings.toml")).unwrap(),
        settings.as_bytes()
    );
}

#[test]
fn a_settings_file_that_is_not_toml_stops_the_binary_naming_it() {
    let sandbox = Sandbox::new("bad_settings", "this is = = not toml");
    let output = sandbox.run(&["team-compiler", "check"]);
    assert_refused(&output, 2, &["settings.toml"]);
}

#[test]
fn the_settings_beside_the_executable_are_read_whatever_the_working_folder() {
    let sandbox = Sandbox::new("working_folder", "[team-compiler]\ncpk_name = 'con'\n");
    let elsewhere = sandbox.root.join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let output = sandbox.run_from(&elsewhere, &["team-compiler", "compile"]);
    assert_refused(&output, 2, &["cpk_name"]);
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
