//! The deep pass: what only a file's contents show, found by `check` and by `compile` alike,
//! on folder and archive exports, dropping and cascading as the structure pass's findings do.

use std::fs;
use std::path::Path;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pass_through_settings, tracer_player_file};
use crate::findings_of;

/// The bytes of `tests/fixtures/deep/<name>`.
fn deep_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/deep")
            .join(name),
    )
    .unwrap()
}

/// `vertex_too_far_from_origin` on slot 05's folder, as both commands report it.
const FAR_STRIKER: &str = "Error vertex_too_far_from_origin [DropFolder] at Players/05 - Striker (file=boots.fmdl, count=1)";

/// `/co/`'s identity line.
const IDENTIFIED: &str = "Info export_identified [Keep] (team=/co/, id=714)";

// TC-CHK-01
#[test]
fn a_far_vertex_drops_its_folder_at_check_and_at_compile_even_with_pass_through() {
    let sandbox = Sandbox::new("deep_far_vertex");
    sandbox.write(
        "exports/co - Far/Players/05 - Striker/boots.fmdl",
        &deep_fixture("boots_far.fmdl"),
    );
    sandbox.write(
        "exports/co - Far/Players/07 - Winger/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );

    let check = sandbox.run("", &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co - Far"),
        [FAR_STRIKER, IDENTIFIED]
    );
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);
    assert_eq!(
        findings_of(&compile.messages(), "co - Far"),
        [FAR_STRIKER, IDENTIFIED]
    );
    assert_eq!(compile.exit_code(), 1);
    // `/co/`'s block starts at 621: slot 07's boots are 627, slot 05's would be 625.
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
        ]
    );
}

// TC-CHK-02
#[test]
fn a_far_vertex_in_a_solid_7z_is_found_by_check() {
    let sandbox = Sandbox::new("deep_far_vertex_7z");
    sandbox.write("exports/co - Far.7z", &deep_fixture("co - Far.7z"));

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Far.7z"),
        [FAR_STRIKER, IDENTIFIED],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn a_far_vertex_in_a_shared_folder_drops_the_player_linking_it() {
    let sandbox = Sandbox::new("deep_far_vertex_shared");
    sandbox.write(
        "exports/co - Far/Boots/Crocs/boots.fmdl",
        &deep_fixture("boots_far.fmdl"),
    );
    sandbox.write("exports/co - Far/Players/05 - Striker/Crocs.boots", b"");
    sandbox.write(
        "exports/co - Far/Players/05 - Striker/glove_l.fmdl",
        &tracer_player_file("glove_l.fmdl"),
    );

    let run = sandbox.run("", &["check"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Far"),
        [
            "Error vertex_too_far_from_origin [DropFolder] at Boots/Crocs (file=boots.fmdl, count=1)",
            "Error link_target_dropped [DropFolder] at Players/05 - Striker (link=Crocs.boots, target=Boots/Crocs, finding=vertex_too_far_from_origin)",
            IDENTIFIED,
        ]
    );
    assert_eq!(run.exit_code(), 1);
}
