//! `summary`: the ruleset's readable rules text on stdout, or written to
//! `-o` with a replaced file kept as `.bak`; a file that does not open is
//! refused before anything is written.

use std::fs;
use std::path::Path;

use crate::common::Sandbox;
use crate::{
    duplicate_id_file, embedded_file, parse_error_file, schema_error_file, valid_file, write,
};

/// The golden summary of the ruleset `name`, a Markdown file of
/// `tests/fixtures/summary/`: the ruleset's summary as the draft writes it.
fn golden(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/summary")
            .join(format!("{name}.md")),
    )
    .unwrap()
}

// RE-CLI-03
#[test]
fn a_valid_file_prints_the_rules_text_byte_for_byte() {
    for (name, text) in [
        ("autumn26", valid_file()),
        ("dbc7", embedded_file("dbc7")),
        ("vgl26", embedded_file("vgl26")),
    ] {
        let sandbox = Sandbox::new(&format!("summary_stdout_{name}"));
        let file = write(&sandbox, "rules.4ccrules", &text);
        let run = sandbox.run(&["summary", &file]);
        assert_eq!(run.exit_code(), 0, "{name}: {:?}", run.messages());
        assert_eq!(
            run.stdout(),
            golden(name),
            "{name}: stdout is the golden byte for byte"
        );
    }
}

// RE-CLI-03
#[test]
fn summary_o_writes_the_text_and_keeps_a_replaced_file_as_bak() {
    let sandbox = Sandbox::new("summary_out");
    let file = write(&sandbox, "rules.4ccrules", &valid_file());
    fs::write(sandbox.path("rules.md"), "old").unwrap();
    let out = sandbox.arg("rules.md");
    let run = sandbox.run(&["summary", &file, "-o", &out]);
    assert_eq!(run.exit_code(), 0, "{:?}", run.messages());
    assert_eq!(
        fs::read_to_string(sandbox.path("rules.md")).unwrap(),
        golden("autumn26"),
        "the file holds the golden byte for byte"
    );
    assert_eq!(
        fs::read_to_string(sandbox.path("rules.md.bak")).unwrap(),
        "old",
        "the replaced file is kept as .bak"
    );
    assert_eq!(
        run.stdout(),
        format!(
            "- Info summary_written (path={})\n",
            sandbox.path("rules.md").display()
        ),
        "nothing on stdout but the finding"
    );
}

// RE-CLI-03
#[test]
fn summary_o_onto_the_rules_file_itself_is_refused() {
    let sandbox = Sandbox::new("summary_out_self");
    let file = write(&sandbox, "rules.4ccrules", &valid_file());
    let before = fs::read(sandbox.path("rules.4ccrules")).unwrap();
    let out = file.clone();
    let run = sandbox.run(&["summary", &file, "-o", &out]);
    run.assert_refused(2, &["rules.4ccrules", "rules file"]);
    assert_eq!(
        fs::read(sandbox.path("rules.4ccrules")).unwrap(),
        before,
        "the file is unchanged"
    );
    assert!(
        !sandbox.path("rules.4ccrules.bak").exists(),
        "no .bak is written"
    );
}

// RE-CLI-03
#[test]
fn summary_o_to_a_new_path_writes_no_bak() {
    let sandbox = Sandbox::new("summary_out_new");
    let file = write(&sandbox, "rules.4ccrules", &valid_file());
    let out = sandbox.arg("rules.md");
    let run = sandbox.run(&["summary", &file, "-o", &out]);
    assert_eq!(run.exit_code(), 0, "{:?}", run.messages());
    assert_eq!(
        fs::read_to_string(sandbox.path("rules.md")).unwrap(),
        golden("autumn26")
    );
    assert!(!sandbox.path("rules.md.bak").exists(), "no .bak is written");
    assert!(
        !sandbox.path("rules.md.tmp").exists(),
        "no temporary sibling is left"
    );
}

// RE-CLI-03
#[test]
fn a_file_that_does_not_open_is_refused_writing_nothing() {
    let sandbox = Sandbox::new("summary_refused");
    for (name, text) in [
        ("parse.4ccrules", parse_error_file()),
        ("schema.4ccrules", schema_error_file()),
        ("duplicate.4ccrules", duplicate_id_file()),
    ] {
        let file = write(&sandbox, name, &text);
        let out = sandbox.arg(&format!("{name}.md"));
        let run = sandbox.run(&["summary", &file, "-o", &out]);
        run.assert_refused(2, &[name]);
        assert!(
            !sandbox.path(&format!("{name}.md")).exists(),
            "{name}: the -o path still does not exist"
        );
        assert!(
            !sandbox.path(&format!("{name}.md.bak")).exists(),
            "{name}: no .bak is written"
        );
    }
}

// RE-SUM-02
#[test]
fn two_rulesets_differing_in_one_rate_differ_in_one_line() {
    let sandbox = Sandbox::new("summary_diff");
    let file = write(&sandbox, "rules.4ccrules", &valid_file());
    let first = sandbox.run(&["summary", &file]);
    assert_eq!(first.exit_code(), 0, "{:?}", first.messages());

    // Autumn 26 with the Silver tier's rate set to 90.
    let aatf_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../libs/aatf");
    let block = fs::read_to_string(aatf_dir.join("rulesets/autumn26.rhai")).unwrap();
    let mut ruleset = aatf::Ruleset::from_data_block(&block).unwrap();
    assert_eq!(
        ruleset.tiers[1].id, "silver",
        "premise: the second tier is Silver"
    );
    assert_eq!(ruleset.tiers[1].rate, 92, "premise: Silver's rate is 92");
    ruleset.tiers[1].rate = 90;
    let edited = write(
        &sandbox,
        "edited.4ccrules",
        &aatf::assemble_file(&ruleset, None, None),
    );
    let second = sandbox.run(&["summary", &edited]);
    assert_eq!(second.exit_code(), 0, "{:?}", second.messages());

    let first = first.stdout();
    let second = second.stdout();
    let first_lines: Vec<&str> = first.lines().collect();
    let second_lines: Vec<&str> = second.lines().collect();
    assert_eq!(
        first_lines.len(),
        second_lines.len(),
        "only the Silver line differs"
    );
    let differing: Vec<usize> = (0..first_lines.len())
        .filter(|&index| first_lines[index] != second_lines[index])
        .collect();
    assert_eq!(differing.len(), 1, "exactly one line differs");
    let index = differing[0];
    assert!(
        first_lines[index].starts_with("- **Silver**:"),
        "the differing line is Silver's: {}",
        first_lines[index]
    );
    assert!(first_lines[index].contains("every stat 92"));
    assert!(second_lines[index].contains("every stat 90"));
}
