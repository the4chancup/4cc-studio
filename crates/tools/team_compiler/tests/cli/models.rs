//! `compile` over boots and gloves models: a player folder's own under the player's planned
//! ID, the skeleton packed with the boots, the folder's textures emitted once for every
//! package; and a shared `Boots/` or `Gloves/` folder players link, compiled once under one of
//! the team's shared IDs, with `check` refusing an export needing more of them than the team
//! has.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use fmdl::FmdlFile;
use fmdl::ops::paths::texture_paths;

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

/// Writes the shared boots folder `Boots/<name>` holding the tracer's boots model and texture,
/// with `skl` as its `boots.skl` when given, and a link to it in each of the player folders
/// `slots` (`Players/03 - P03/<name>.boots`), which hold nothing else.
fn write_shared_boots(
    sandbox: &Sandbox,
    export: &str,
    name: &str,
    skl: Option<&[u8]>,
    slots: &[u8],
) {
    let folder = format!("exports/{export}/Boots/{name}");
    sandbox.write(
        &format!("{folder}/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{folder}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    if let Some(skl) = skl {
        sandbox.write(&format!("{folder}/boots.skl"), skl);
    }
    for slot in slots {
        sandbox.write(
            &format!("exports/{export}/Players/{slot:02} - P{slot:02}/{name}.boots"),
            b"",
        );
    }
}

/// The `boots.skl` of the boots package at `path` in `entries`.
fn boots_skl(entries: &BTreeMap<String, Vec<u8>>, path: &str) -> Vec<u8> {
    fpk::FpkFile::read(&entries[path])
        .unwrap()
        .get("boots.skl")
        .unwrap()
        .to_vec()
}

// TC-MOD-05
#[test]
fn shared_boots_folders_compile_once_each_under_the_shared_ids_in_name_order() {
    let sandbox = Sandbox::new("mod_shared_boots");
    // Crocs brings its own skeleton, Mud none, so each package says which folder it came from.
    let crocs_skl = body_skl("pes19");
    assert_ne!(crocs_skl, body_skl("pes21"));
    write_shared_boots(&sandbox, "co - Shared", "Mud", None, &[11]);
    write_shared_boots(&sandbox, "co - Shared", "Crocs", Some(&crocs_skl), &[3, 7]);

    let entries = compile_clean(&sandbox, "co - Shared");

    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0644/#Win/boots.fpk",
            "Asset/model/character/boots/k0644/#Win/boots.fpkd",
            "Asset/model/character/boots/k0644/#windx11/shirt.ftex",
            "Asset/model/character/boots/k0645/#Win/boots.fpk",
            "Asset/model/character/boots/k0645/#Win/boots.fpkd",
            "Asset/model/character/boots/k0645/#windx11/shirt.ftex",
        ],
        "no k0623, k0627 or k0631 for the linking slots"
    );
    let crocs = "Asset/model/character/boots/k0644/#Win/boots.fpk";
    let mud = "Asset/model/character/boots/k0645/#Win/boots.fpk";
    assert_eq!(boots_skl(&entries, crocs), crocs_skl);
    assert_eq!(boots_skl(&entries, mud), body_skl("pes21"));
    // Each model names its own folder's textures: the tracer's boots model lists `shirt.dds`
    // in two texture slots, both pointed at the output's folder.
    for (path, directory) in [
        (crocs, "/Assets/pes16/model/character/boots/k0644/"),
        (mud, "/Assets/pes16/model/character/boots/k0645/"),
    ] {
        let package = fpk::FpkFile::read(&entries[path]).unwrap();
        let model = FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap();
        let shirt: Vec<String> = texture_paths(&model)
            .unwrap()
            .into_iter()
            .filter(|texture| texture.file_name == "shirt.dds")
            .map(|texture| texture.directory)
            .collect();
        assert_eq!(shirt, [directory, directory], "{path}");
    }
}

// TC-MOD-06
#[test]
fn eighteen_shared_boots_folders_exhaust_the_team_s_ids_and_check_skips_the_export() {
    let sandbox = Sandbox::new("mod_boots_pool");
    for slot in 1..=18u8 {
        write_shared_boots(&sandbox, "co - Pool", &format!("S{slot:02}"), None, &[slot]);
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(
        run.messages(),
        [
            "co - Pool: Info export_identified [Keep] (team=/co/, id=714)",
            "co - Pool: Error boots_id_pool_exhausted [DropExport] (count=18)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-PLN-02
#[test]
fn shared_ids_follow_the_folder_names_so_a_new_folder_shifts_the_ones_after_it() {
    let sandbox = Sandbox::new("pln_shared_ids");
    let apple_skl = body_skl("pes19");
    let mango_skl = body_skl("pes18");
    assert_ne!(apple_skl, body_skl("pes21"));
    assert_ne!(mango_skl, body_skl("pes21"));
    assert_ne!(mango_skl, apple_skl);
    write_shared_boots(&sandbox, "co - Named", "Zebra", None, &[3]);
    write_shared_boots(&sandbox, "co - Named", "Apple", Some(&apple_skl), &[7]);
    let k0644 = "Asset/model/character/boots/k0644/#Win/boots.fpk";
    let k0645 = "Asset/model/character/boots/k0645/#Win/boots.fpk";
    let k0646 = "Asset/model/character/boots/k0646/#Win/boots.fpk";

    let first = compile_clean(&sandbox, "co - Named");
    assert!(!first.contains_key(k0646), "{:?}", first.keys());
    assert_eq!(boots_skl(&first, k0644), apple_skl, "Apple");
    assert_eq!(boots_skl(&first, k0645), body_skl("pes21"), "Zebra");

    write_shared_boots(&sandbox, "co - Named", "Mango", Some(&mango_skl), &[11]);
    let second = compile_clean(&sandbox, "co - Named");
    assert_eq!(boots_skl(&second, k0644), apple_skl, "Apple");
    assert_eq!(boots_skl(&second, k0645), mango_skl, "Mango");
    assert_eq!(boots_skl(&second, k0646), body_skl("pes21"), "Zebra");
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
