//! `validate`: a rules file's load findings as `rules_finding` lines; a clean
//! file prints nothing.

use crate::common::Sandbox;
use crate::{duplicate_id_file, parse_error_file, schema_error_file, valid_file, write};

// RE-CLI-01
#[test]
fn a_valid_file_prints_no_finding_and_exits_0() {
    let sandbox = Sandbox::new("validate_ok");
    let file = write(&sandbox, "rules.rhai", &valid_file());
    let run = sandbox.run(&["validate", &file]);
    assert_eq!(run.exit_code(), 0, "{:?}", run.messages());
    assert!(run.messages().is_empty(), "{:?}", run.messages());
}

// RE-CLI-01
#[test]
fn a_parse_error_prints_its_line_and_column_and_exits_1() {
    let sandbox = Sandbox::new("validate_parse");
    // The RE-FILE-03 file: a parse error at line 12.
    let file = write(&sandbox, "rules.rhai", &parse_error_file());
    let run = sandbox.run(&["validate", &file]);
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert!(
        run.messages()
            .iter()
            .any(|line| line.starts_with("Error rules_finding (line=12, column=")),
        "{:?}",
        run.messages()
    );
}

// RE-CLI-01
#[test]
fn a_schema_error_prints_its_key_path_and_exits_1() {
    let sandbox = Sandbox::new("validate_schema");
    // The RE-FILE-03 file: `tiers[2].free.tricks` is the string "two".
    let file = write(&sandbox, "rules.rhai", &schema_error_file());
    let run = sandbox.run(&["validate", &file]);
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert!(
        run.messages()
            .iter()
            .any(|line| line.contains("tiers[2].free.tricks")),
        "{:?}",
        run.messages()
    );
}

// RE-CLI-01
#[test]
fn a_duplicate_tier_id_prints_its_key_path_and_exits_1() {
    let sandbox = Sandbox::new("validate_duplicate");
    // The RE-FILE-03 file: the second tier carries the first tier's id.
    let file = write(&sandbox, "rules.rhai", &duplicate_id_file());
    let run = sandbox.run(&["validate", &file]);
    assert_eq!(run.exit_code(), 1, "{:?}", run.messages());
    assert!(
        run.messages()
            .iter()
            .any(|line| line.contains("tiers[1]") && line.contains("gold")),
        "{:?}",
        run.messages()
    );
}
