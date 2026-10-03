//! The deep pass: what only a file's contents show, found by `check` and by `compile` alike,
//! on folder and archive exports, dropping and cascading as the structure pass's findings do.

use std::fs;
use std::path::Path;

use fmdl::{FmdlFile, Model};

use crate::common::Sandbox;
use crate::compile::{
    compiled_kits, compiled_players, cpk_entries, pass_through_settings, pes21_settings,
    tracer_kit, tracer_player_file,
};
use crate::findings_of;
use crate::models::face_diff_fixture;

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
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
        ]
    );
}

/// The tracer's face models in the player folder `player`, as both commands report them.
fn tracer_face_weights(player: &str) -> [String; 3] {
    [
        ("boots.fmdl", 1662),
        ("fcl_hair.fmdl", 1662),
        ("glove_l.fmdl", 2),
    ]
    .map(|(file, count)| {
        format!(
            "Info fmdl_weights_not_normalized [Keep] at Players/{player} (file={file}, count={count})"
        )
    })
}

/// `settings_toml_invalid` on slot `player`'s `settings.toml` holding `name = 5`.
fn settings_invalid(player: &str) -> String {
    format!(
        "Error settings_toml_invalid [DropFile] at Players/{player}/settings.toml \
         (file=settings.toml, error=name: expected true or a string)"
    )
}

// TC-CHK-03
#[test]
fn a_settings_toml_that_does_not_parse_is_ignored_and_the_folder_s_models_compile() {
    let sandbox = Sandbox::new("deep_settings_invalid");
    sandbox.copy_tracer_face("exports/co - Settings/Players/05 - A");
    // `name` takes `true` or a string.
    sandbox.write(
        "exports/co - Settings/Players/05 - A/settings.toml",
        b"name = 5\n",
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let mut expected = tracer_face_weights("05 - A").to_vec();
    expected.push(settings_invalid("05 - A"));
    expected.push(IDENTIFIED.to_owned());
    assert_eq!(findings_of(&run.messages(), "co - Settings"), expected);
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [71405]);
}

/// `kit_config_invalid` on `Kits/p1`, whose `config.toml` holds `shirt = 144`.
const KIT_CONFIG_INVALID: &str = "Error kit_config_invalid [DropFolder] at Kits/p1 \
     (file=config.toml, error=invalid value for shirt: 144)";

/// Writes a kit `p1` whose `config.toml` is the wrong-typed `shirt = 144` and a kit `p2`
/// with no config into `export`.
fn write_kits(sandbox: &Sandbox, export: &str) {
    sandbox.write(&format!("{export}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.write(&format!("{export}/Kits/p1/config.toml"), b"shirt = 144\n");
    sandbox.write(&format!("{export}/Kits/p2/kit.dds"), &tracer_kit());
}

#[test]
fn a_kit_config_that_does_not_parse_leaves_its_kit_out_and_the_kit_beside_it_compiles() {
    let sandbox = Sandbox::new("deep_kit_config_invalid");
    write_kits(&sandbox, "exports/co - Kits");

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co - Kits"),
        [KIT_CONFIG_INVALID, IDENTIFIED]
    );
    assert_eq!(check.exit_code(), 1);

    let compile = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    assert_eq!(
        findings_of(&compile.messages(), "co - Kits"),
        [
            KIT_CONFIG_INVALID,
            IDENTIFIED,
            "Info kit_config_generated [Keep] at Kits/p2 ()"
        ]
    );
    assert_eq!(compile.exit_code(), 1);
    assert_eq!(compiled_kits(&sandbox), ["u0714p2"]);
}

#[test]
fn pass_through_keeps_no_face_diff_kit_config_or_settings_toml_that_cannot_be_read() {
    let sandbox = Sandbox::new("deep_documents_pass_through");
    let export = "exports/co - Unreadable";
    // Slot 03: a `face_diff.bin` one byte shorter than its header gives.
    sandbox.copy_tracer_face(&format!("{export}/Players/03 - A"));
    sandbox.write(
        &format!("{export}/Players/03 - A/face_diff.bin"),
        &face_diff_fixture("dif.bin")[..943],
    );
    // Slot 05: a `face_diff.xml` beside the tracer's `face_diff.bin`.
    sandbox.copy_tracer_face(&format!("{export}/Players/05 - B"));
    sandbox.write(
        &format!("{export}/Players/05 - B/face_diff.xml"),
        &face_diff_fixture("dif.xml"),
    );
    // Slot 07: a `settings.toml` whose `name` is a number.
    sandbox.copy_tracer_face(&format!("{export}/Players/07 - C"));
    sandbox.write(
        &format!("{export}/Players/07 - C/settings.toml"),
        b"name = 5\n",
    );
    write_kits(&sandbox, export);

    let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

    let mut expected = tracer_face_weights("03 - A").to_vec();
    expected.push(
        "Error face_diff_invalid [DropFolder] at Players/03 - A (file=face_diff.bin, \
         reason=the face diff is 943 bytes long, but its header gives 944: the game would \
         read past its end)"
            .to_owned(),
    );
    expected.extend(tracer_face_weights("05 - B"));
    expected.push(
        "Error xml_dif_conflict [DropFolder] at Players/05 - B (file=face_diff.xml)".to_owned(),
    );
    expected.extend(tracer_face_weights("07 - C"));
    expected.push(settings_invalid("07 - C"));
    expected.extend(
        [
            KIT_CONFIG_INVALID,
            IDENTIFIED,
            "Info kit_config_generated [Keep] at Kits/p2 ()",
        ]
        .map(str::to_owned),
    );
    assert_eq!(findings_of(&run.messages(), "co - Unreadable"), expected);
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [71407]);
    assert_eq!(compiled_kits(&sandbox), ["u0714p2"]);
}
