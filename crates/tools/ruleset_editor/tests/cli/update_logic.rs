//! `update-logic`: a current file is left as it was; a file with hand-edited
//! checking code is refused, untouched.

use std::fs;

use crate::common::Sandbox;
use crate::{duplicate_id_file, parse_error_file, schema_error_file, valid_file, write};

// RE-CLI-02
#[test]
fn a_current_file_is_left_byte_for_byte_with_no_bak() {
    let sandbox = Sandbox::new("update_logic_current");
    let file = write(&sandbox, "rules.rhai", &valid_file());
    let before = fs::read(sandbox.path("rules.rhai")).unwrap();
    let run = sandbox.run(&["update-logic", &file]);
    assert_eq!(run.exit_code(), 0, "{:?}", run.messages());
    assert_eq!(
        run.messages(),
        [format!(
            "Info logic_already_current (path={})",
            sandbox.path("rules.rhai").display()
        )]
    );
    assert_eq!(
        fs::read(sandbox.path("rules.rhai")).unwrap(),
        before,
        "the file is left byte for byte"
    );
    assert!(
        !sandbox.path("rules.rhai.bak").exists(),
        "no .bak is written"
    );
}

// RE-CLI-02
#[test]
fn a_modified_logic_is_refused_untouched() {
    let sandbox = Sandbox::new("update_logic_modified");
    // One comment's text changed in the checking code: the file still loads
    // but its code is not Studio's.
    let text = valid_file().replacen(
        "// The generic interpreter",
        "// The interpreter, its comment edited",
        1,
    );
    let file = write(&sandbox, "rules.rhai", &text);
    let run = sandbox.run(&["update-logic", &file]);
    run.assert_refused(2, &["not this build's", "discard"]);
    assert_eq!(
        fs::read(sandbox.path("rules.rhai")).unwrap(),
        text.as_bytes(),
        "the file is unchanged"
    );
    assert!(
        !sandbox.path("rules.rhai.bak").exists(),
        "no .bak is written"
    );
}

// RE-CLI-01
#[test]
fn update_logic_refuses_a_file_that_does_not_load() {
    let sandbox = Sandbox::new("update_logic_invalid");
    for (name, text) in [
        ("parse.rhai", parse_error_file()),
        ("schema.rhai", schema_error_file()),
        ("duplicate.rhai", duplicate_id_file()),
    ] {
        let file = write(&sandbox, name, &text);
        let before = fs::read(sandbox.path(name)).unwrap();
        let run = sandbox.run(&["update-logic", &file]);
        run.assert_refused(2, &[name]);
        assert_eq!(
            fs::read(sandbox.path(name)).unwrap(),
            before,
            "{name}: the file is left byte for byte"
        );
        assert!(
            !sandbox.path(&format!("{name}.bak")).exists(),
            "{name}: no .bak is written"
        );
    }
}
