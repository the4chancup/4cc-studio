//! `compile` over a player folder's own boots and gloves models: the packages under the
//! player's planned ID, the skeleton packed with the boots, and the folder's textures emitted
//! once for every package.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_player_file};
use crate::findings_of;

/// The entry names of the FPK `bytes`.
fn package_names(bytes: &[u8]) -> Vec<String> {
    fpk::FpkFile::read(bytes)
        .unwrap()
        .entries()
        .map(|(name, _)| name.to_owned())
        .collect()
}

/// One of the bundled game skeletons, `resources/skeletons/<version>/body.skl`.
fn body_skl(version: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../resources/skeletons")
            .join(version)
            .join("body.skl"),
    )
    .unwrap()
}

/// Writes the tracer's face files into `folder` under their own names, and its boots and
/// gloves models under `boots_name`, `glove_l.fmdl` and `glove_r.fmdl`.
fn write_player(sandbox: &Sandbox, folder: &str, boots_name: &str) {
    for name in [
        "face_diff.bin",
        "fcl_hair.fmdl",
        "fcl_hair.skl",
        "fcl_hair_sim.fclo",
        "shirt.dds",
        "glove_l.fmdl",
        "glove_r.fmdl",
    ] {
        sandbox.write(&format!("{folder}/{name}"), &tracer_player_file(name));
    }
    sandbox.write(
        &format!("{folder}/{boots_name}"),
        &tracer_player_file("boots.fmdl"),
    );
}

/// Compiles the sandbox for PES 21, asserting the export `name` reports only its identity,
/// and returns the CPK's entries by path.
fn compile_clean(sandbox: &Sandbox, name: &str) -> BTreeMap<String, Vec<u8>> {
    let run = sandbox.run(&pes21_settings(sandbox), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), name),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(run.exit_code(), 0);
    cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"))
}

// TC-MOD-01
#[test]
fn a_player_s_own_boots_and_gloves_compile_under_its_exclusive_id_with_the_textures_once() {
    let sandbox = Sandbox::new("mod_boots_gloves");
    write_player(
        &sandbox,
        "exports/co - Models/Players/05 - A",
        "kit_boots.fmdl",
    );

    let entries = compile_clean(&sandbox, "co - Models");

    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0625/#Win/boots.fpk",
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "Asset/model/character/glove/g0625/#Win/glove.fpk",
            "Asset/model/character/glove/g0625/#Win/glove.fpkd",
        ]
    );
    assert_eq!(
        package_names(&entries["Asset/model/character/boots/k0625/#Win/boots.fpk"]),
        ["boots.fmdl", "boots.skl"]
    );
    assert_eq!(
        package_names(&entries["Asset/model/character/glove/g0625/#Win/glove.fpk"]),
        ["glove_l.fmdl", "glove_r.fmdl"]
    );
}

// TC-MOD-02
#[test]
fn boots_skl_is_the_skeleton_named_after_the_boots_model_or_the_bundled_pes_21_body_skl() {
    let boots_fpk = "Asset/model/character/boots/k0625/#Win/boots.fpk";

    let paired = Sandbox::new("mod_boots_skl_paired");
    write_player(
        &paired,
        "exports/co - Paired/Players/05 - A",
        "kit_boots.fmdl",
    );
    // A real skeleton that differs from the bundled PES 21 one.
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    paired.write("exports/co - Paired/Players/05 - A/kit_boots.skl", &custom);
    let entries = compile_clean(&paired, "co - Paired");
    let package = fpk::FpkFile::read(&entries[boots_fpk]).unwrap();
    assert_eq!(package.get("boots.skl").unwrap(), custom);

    let bare = Sandbox::new("mod_boots_skl_bundled");
    write_player(&bare, "exports/co - Bare/Players/05 - A", "kit_boots.fmdl");
    let entries = compile_clean(&bare, "co - Bare");
    let package = fpk::FpkFile::read(&entries[boots_fpk]).unwrap();
    assert_eq!(package.get("boots.skl").unwrap(), body_skl("pes21"));
}

// TC-MOD-03
#[test]
fn a_folder_mapped_to_two_slots_emits_its_boots_under_both_ids_and_its_textures_once() {
    let sandbox = Sandbox::new("mod_two_slots");
    sandbox.write("exports/co - Twice/players.txt", b"03 A\n07 A\n");
    for name in ["boots.fmdl", "shirt.dds"] {
        sandbox.write(
            &format!("exports/co - Twice/Players/A/{name}"),
            &tracer_player_file(name),
        );
    }

    let entries = compile_clean(&sandbox, "co - Twice");

    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0623/#Win/boots.fpk",
            "Asset/model/character/boots/k0623/#Win/boots.fpkd",
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            "Asset/model/character/common/714/A/sourceimages/#windx11/shirt.ftex",
        ]
    );
    assert_eq!(
        entries["Asset/model/character/boots/k0623/#Win/boots.fpk"],
        entries["Asset/model/character/boots/k0627/#Win/boots.fpk"],
        "one package under both ids"
    );
}

#[test]
fn a_texture_that_cannot_convert_drops_the_whole_folder_and_the_folder_beside_it_compiles() {
    let sandbox = Sandbox::new("mod_texture_failed");
    write_player(&sandbox, "exports/co - Broken/Players/05 - A", "boots.fmdl");
    // The DDS with its header cut off: nothing can read it as a texture.
    let cut = tracer_player_file("shirt.dds")[128..].to_vec();
    sandbox.write("exports/co - Broken/Players/05 - A/shirt.dds", &cut);
    write_player(&sandbox, "exports/co - Broken/Players/07 - B", "boots.fmdl");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let findings = findings_of(&lines, "co - Broken");
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(
        findings[0],
        "Info export_identified [Keep] (team=/co/, id=714)"
    );
    assert!(
        findings[1].starts_with(
            "Error folder_pack_failed [DropFolder] at Players/05 - A (error=shirt.dds: cannot convert to FTEX"
        ),
        "{}",
        findings[1]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            "Asset/model/character/common/714/07 - B/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "Asset/model/character/glove/g0627/#Win/glove.fpk",
            "Asset/model/character/glove/g0627/#Win/glove.fpkd",
        ],
        "nothing of slot 05 is in the CPK"
    );
}

// TC-PLN-01
#[test]
fn the_planned_ids_are_the_slot_s_and_two_compiles_write_the_same_bytes() {
    let sandbox = Sandbox::new("pln_planned_ids");
    sandbox.write(
        "exports/co - Planned/Players/05 - A/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        "exports/co - Planned/Players/23 - B/glove_l.fmdl",
        &tracer_player_file("glove_l.fmdl"),
    );

    let first = compile_clean(&sandbox, "co - Planned");
    let first_cpk = fs::read(sandbox.root.join("output/4cc_90_test.cpk")).unwrap();
    let second = compile_clean(&sandbox, "co - Planned");
    let second_cpk = fs::read(sandbox.root.join("output/4cc_90_test.cpk")).unwrap();

    let paths: Vec<&str> = first.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0625/#Win/boots.fpk",
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/glove/g0643/#Win/glove.fpk",
            "Asset/model/character/glove/g0643/#Win/glove.fpkd",
        ]
    );
    assert_eq!(
        package_names(&first["Asset/model/character/glove/g0643/#Win/glove.fpk"]),
        ["glove_l.fmdl"]
    );
    assert_eq!(first, second);
    assert!(first_cpk == second_cpk, "the two CPKs differ");
}
