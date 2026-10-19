//! The `upgrade-models` command line (the plan's "CLI" scenarios): the input
//! and output refusals, and a run's output folders and report.

use std::fs;

use crate::common::{Run, Sandbox, snapshot};

/// `upgrade-models <input> <extra_args>`.
fn upgrade_models(sandbox: &Sandbox, input: &str, extra_args: &[&str]) -> Run {
    let input = sandbox.arg(input);
    let mut args = vec!["upgrade-models", &input];
    args.extend(extra_args);
    sandbox.run(&args)
}

// EU-CLI-04
#[test]
fn a_missing_input_or_a_file_is_refused() {
    let sandbox = Sandbox::new("models_cli_refusals");
    fs::write(sandbox.path("notes.txt"), b"text").unwrap();
    let before = sandbox.listing("");

    upgrade_models(&sandbox, "gone", &[]).assert_refused(2, &["gone", "no such folder or file"]);
    upgrade_models(&sandbox, "notes.txt", &[]).assert_refused(2, &["notes.txt", "not a folder"]);

    assert_eq!(sandbox.listing(""), before, "nothing is written");
}

// EU-CLI-05
#[test]
fn an_output_folder_that_exists_or_cannot_be_created_is_refused() {
    let sandbox = Sandbox::new("models_cli_output_exists");
    fs::create_dir_all(sandbox.path("new models/XXX15 - A")).unwrap();
    fs::write(sandbox.path("new models/XXX15 - A/face.fmdl"), b"face").unwrap();
    fs::create_dir_all(sandbox.path("new models upgraded")).unwrap();
    fs::write(sandbox.path("new models upgraded/keep.txt"), b"my edit").unwrap();

    let before = snapshot(&sandbox);
    upgrade_models(&sandbox, "new models", &[]).assert_refused(3, &["new models upgraded"]);
    assert_eq!(snapshot(&sandbox), before, "nothing is written");

    // An output that cannot be created: -o names a path below a file.
    fs::write(sandbox.path("blocked"), b"a file").unwrap();
    let out = sandbox.arg("blocked/out");
    let before = snapshot(&sandbox);
    upgrade_models(&sandbox, "new models", &["-o", &out]).assert_refused(3, &["blocked"]);
    assert_eq!(snapshot(&sandbox), before, "nothing is written");
}

// EU-CLI-06
#[test]
fn the_output_is_a_sibling_folder_holding_one_folder_per_input_folder() {
    let sandbox = Sandbox::new("models_cli_run");
    // Face and boots folders, both Fox-spelled. No step parses the FMDL: a
    // file that does not read as one is copied as it is.
    fs::create_dir_all(sandbox.path("new models/XXX15 - Snuffy")).unwrap();
    fs::write(sandbox.path("new models/XXX15 - Snuffy/face.fmdl"), b"face").unwrap();
    fs::create_dir_all(sandbox.path("new models/k0431")).unwrap();
    fs::write(sandbox.path("new models/k0431/boots.fmdl"), b"boots").unwrap();

    let run = upgrade_models(&sandbox, "new models", &[]);
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        sandbox.listing("new models upgraded"),
        [
            "15 - Snuffy".to_owned(),
            "15 - Snuffy/face.fmdl".to_owned(),
            "k0431".to_owned(),
            "k0431/boots.fmdl".to_owned(),
        ],
    );
    let stdout = run.stdout();
    assert!(
        stdout.contains("model_folder_destination at 15 - Snuffy (into=Players)"),
        "{stdout}"
    );
    assert!(
        stdout.contains("model_folder_destination at k0431 (into=Boots)"),
        "{stdout}"
    );
}
