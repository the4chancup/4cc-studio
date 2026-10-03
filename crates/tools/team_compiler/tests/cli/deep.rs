//! The deep pass: what only a file's contents show, found by `check` and by `compile` alike,
//! on folder and archive exports, dropping and cascading as the structure pass's findings do.

use std::fs;
use std::path::Path;

use fmdl::{FmdlFile, Model};

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pass_through_settings, pes21_settings, tracer_player_file};
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
        [
            FAR_STRIKER,
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - Striker (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - Winger (file=boots.fmdl, count=1662)",
            IDENTIFIED
        ]
    );
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);
    assert_eq!(
        findings_of(&compile.messages(), "co - Far"),
        [
            FAR_STRIKER,
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - Striker (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - Winger (file=boots.fmdl, count=1662)",
            IDENTIFIED
        ]
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
        [
            FAR_STRIKER,
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - Striker (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - Striker (file=glove_l.fmdl, count=2)",
            IDENTIFIED
        ],
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
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - Striker (file=glove_l.fmdl, count=2)",
            "Error vertex_too_far_from_origin [DropFolder] at Boots/Crocs (file=boots.fmdl, count=1)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Error link_target_dropped [DropFolder] at Players/05 - Striker (link=Crocs.boots, target=Boots/Crocs, finding=vertex_too_far_from_origin)",
            IDENTIFIED
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

/// The tracer's boots model with its first mesh holding 21846 faces (its first, repeated), one
/// over the hard limit: `fmdl_mesh_over_face_limit`, an Error a member can still pass through.
fn boots_over_the_face_limit() -> Vec<u8> {
    let file = FmdlFile::read(&tracer_player_file("boots.fmdl")).unwrap();
    let mut model = Model::from_file(&file).unwrap();
    let mesh = &mut model.meshes[0];
    mesh.faces = vec![mesh.faces[0]; 21_846];
    model.to_file().unwrap().write()
}

/// Slot 07's boots, the tracer's, as both commands report them.
const WINGER_WEIGHTS: &str =
    "Info fmdl_weights_not_normalized [Keep] at Players/07 - Winger (file=boots.fmdl, count=1662)";

// TC-CHK-06
#[test]
fn a_format_error_drops_its_folder_unless_pass_through_keeps_it() {
    let sandbox = Sandbox::new("deep_format_error");
    sandbox.write(
        "exports/co - Faces/Players/05 - Striker/boots.fmdl",
        &boots_over_the_face_limit(),
    );
    sandbox.write(
        "exports/co - Faces/Players/07 - Winger/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );
    let striker_weights = "Info fmdl_weights_not_normalized [Keep] at Players/05 - Striker (file=boots.fmdl, count=1662)";
    let boots = |slot: u32| {
        [
            format!("Asset/model/character/boots/k{slot:04}/#Win/boots.fpk"),
            format!("Asset/model/character/boots/k{slot:04}/#Win/boots.fpkd"),
        ]
    };

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            findings_of(&run.messages(), "co - Faces"),
            [
                "Error fmdl_mesh_over_face_limit [DropFolder] at Players/05 - Striker (file=boots.fmdl, count=21846)",
                striker_weights,
                WINGER_WEIGHTS,
                IDENTIFIED
            ],
            "{command}"
        );
        assert_eq!(run.exit_code(), 1, "{command}");
    }
    // `/co/`'s block starts at 621: slot 07's boots are 627, slot 05's 625.
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(paths, boots(627));

    for command in ["check", "compile"] {
        let run = sandbox.run(&pass_through_settings(&sandbox), &[command]);
        assert_eq!(
            findings_of(&run.messages(), "co - Faces"),
            [
                "Error fmdl_mesh_over_face_limit [Keep] at Players/05 - Striker (file=boots.fmdl, count=21846)",
                striker_weights,
                WINGER_WEIGHTS,
                IDENTIFIED
            ],
            "{command}"
        );
        assert_eq!(run.exit_code(), 1, "{command}");
    }
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(paths, [boots(625), boots(627)].concat());
}

// TC-CHK-07
#[test]
fn a_model_that_does_not_parse_is_model_broken_and_drops_its_folder_even_with_pass_through() {
    let sandbox = Sandbox::new("deep_model_broken");
    sandbox.write(
        "exports/co - Broken/Players/05 - Striker/boots.fmdl",
        b"not a model",
    );
    sandbox.write(
        "exports/co - Broken/Players/07 - Winger/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );
    let findings = [
        "Error model_broken [DropFolder] at Players/05 - Striker (file=boots.fmdl, error=fmdl is truncated)",
        WINGER_WEIGHTS,
        IDENTIFIED,
    ];

    let check = sandbox.run("", &["check"]);
    assert_eq!(findings_of(&check.messages(), "co - Broken"), findings);
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);
    assert_eq!(findings_of(&compile.messages(), "co - Broken"), findings);
    assert_eq!(compile.exit_code(), 1);
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

// `compile` emits no logo yet, so until it does "no logo is emitted" holds for any export; the
// exact entry list below is what keeps proving it afterwards.
// TC-CHK-05
#[test]
fn a_logo_that_does_not_decode_is_logo_file_invalid_and_the_export_is_otherwise_kept() {
    let sandbox = Sandbox::new("deep_logo_invalid");
    sandbox.write("exports/co - Logo/logo.png", b"not an image");
    sandbox.write(
        "exports/co - Logo/Players/07 - Winger/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );
    let findings = [
        WINGER_WEIGHTS,
        "Error logo_file_invalid [DropFile] at logo.png (file=logo.png, error=image decode failed: Format error decoding Png: Invalid PNG signature.)",
        IDENTIFIED,
    ];

    let check = sandbox.run("", &["check"]);
    assert_eq!(findings_of(&check.messages(), "co - Logo"), findings);
    assert_eq!(check.exit_code(), 1);

    // Pass-through does not keep it: there is nothing the game's logo sizes can be made from.
    let compile = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);
    assert_eq!(findings_of(&compile.messages(), "co - Logo"), findings);
    assert_eq!(compile.exit_code(), 1);
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
