//! `compile` over boots and gloves models: a player folder's own under the player's planned
//! ID, the skeleton packed with the boots, the folder's textures emitted once for every
//! package; a shared `Boots/` or `Gloves/` folder players link, compiled once under one of
//! the team's shared IDs, with `check` refusing an export needing more of them than the team
//! has; the merges: several parts under one name into one model, a shared folder a link
//! combines with the player's own model into the player's package, a shared `Faces/` folder
//! into the player's face; and a texture two of the player's sources hold, packed once or
//! resolved by its bytes.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use fmdl::ops::paths::texture_paths;
use fmdl::{FmdlFile, Model};

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_kit, tracer_player_file};
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

/// The mesh count of the `boots.fmdl` of the boots package at `path` in `entries`.
fn boots_mesh_count(entries: &BTreeMap<String, Vec<u8>>, path: &str) -> usize {
    let package = fpk::FpkFile::read(&entries[path]).unwrap();
    let model = FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap();
    Model::from_file(&model).unwrap().meshes.len()
}

/// The mesh count of the tracer's boots model.
fn tracer_boots_mesh_count() -> usize {
    let model = FmdlFile::read(&tracer_player_file("boots.fmdl")).unwrap();
    Model::from_file(&model).unwrap().meshes.len()
}

// TC-MOD-07
#[test]
fn a_boots_link_beside_a_local_boots_model_combines_the_shared_folder_into_the_player_s_boots() {
    let sandbox = Sandbox::new("mod_link_combined");
    let export = "exports/co - Combined";
    write_player(
        &sandbox,
        &format!("{export}/Players/05 - A"),
        "kit_boots.fmdl",
    );
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    // A texture of the shared folder's own, under a stem the player does not hold.
    sandbox.write(
        &format!("{export}/Boots/Crocs/sole.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let k0625 = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    let k0644 = "Asset/model/character/boots/k0644/#Win/boots.fpk";
    let sole = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/sole.ftex";

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Combined"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0625/#Win/boots.fpk",
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex",
            sole,
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "Asset/model/character/glove/g0625/#Win/glove.fpk",
            "Asset/model/character/glove/g0625/#Win/glove.fpkd",
        ],
        "no k0644: the shared folder is only a source of parts"
    );
    assert_eq!(
        boots_mesh_count(&entries, k0625),
        2 * tracer_boots_mesh_count(),
        "Crocs's meshes plus the local model's"
    );
    assert_eq!(boots_skl(&entries, k0625), body_skl("pes21"));

    // Slot 07 links Crocs plainly: Crocs compiles on its own too, as it is, textures beside
    // it, while slot 05's merge is unchanged.
    sandbox.write(&format!("{export}/Players/07 - B/Crocs.boots"), b"");
    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co - Combined"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let added: Vec<&str> = entries
        .keys()
        .map(String::as_str)
        .filter(|path| !paths.contains(path))
        .collect();
    assert_eq!(
        added,
        [
            k0644,
            "Asset/model/character/boots/k0644/#Win/boots.fpkd",
            "Asset/model/character/boots/k0644/#windx11/sole.ftex",
        ]
    );
    assert_eq!(boots_mesh_count(&entries, k0644), tracer_boots_mesh_count());
    assert_eq!(
        boots_mesh_count(&entries, k0625),
        2 * tracer_boots_mesh_count()
    );
    assert_eq!(
        entries[sole],
        entries["Asset/model/character/boots/k0644/#windx11/sole.ftex"]
    );
}

/// One of the bundled face templates, `resources/templates/<name>`.
fn template(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../resources/templates")
            .join(name),
    )
    .unwrap()
}

/// The face package of slot 05 in `entries`.
fn face_package(entries: &BTreeMap<String, Vec<u8>>) -> fpk::FpkFile {
    fpk::FpkFile::read(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"]).unwrap()
}

// TC-MOD-12
#[test]
fn a_hair_model_alone_gets_the_bundled_face_diff_hair_simulation_and_body_skeleton() {
    let sandbox = Sandbox::new("mod_face_templates");
    sandbox.write(
        "exports/co - Hair/Players/05 - A/fcl_hair.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );

    let entries = compile_clean(&sandbox, "co - Hair");

    let package = face_package(&entries);
    let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
    assert_eq!(
        names,
        [
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair_sim.fclo",
            "fcl_hair_sim.skl"
        ]
    );
    assert_eq!(
        package.get("face_diff.bin").unwrap(),
        template("face_diff.bin")
    );
    assert_eq!(
        package.get("fcl_hair_sim.fclo").unwrap(),
        template("fcl_hair_sim.fclo")
    );
    assert_eq!(package.get("fcl_hair_sim.skl").unwrap(), body_skl("pes21"));
}

// TC-MOD-13
#[test]
fn a_skeleton_paired_with_a_face_model_without_a_slot_is_reported_and_not_packed() {
    let sandbox = Sandbox::new("mod_skl_no_slot");
    sandbox.write(
        "exports/co - Slot/Players/05 - A/face_high.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        "exports/co - Slot/Players/05 - A/face_high.skl",
        &tracer_player_file("fcl_hair.skl"),
    );
    let findings = [
        "Info export_identified [Keep] (team=/co/, id=714)",
        "Warning skl_no_slot [Keep] at Players/05 - A (file=face_high.skl)",
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(findings_of(&check.messages(), "co - Slot"), findings);
    assert_eq!(check.exit_code(), 0);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    assert_eq!(findings_of(&run.messages(), "co - Slot"), findings);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let package = face_package(&entries);
    let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
    assert_eq!(names, ["face_diff.bin", "face_high.fmdl"]);
}

#[test]
fn an_unsuffixed_model_is_reported_and_merged_into_the_hair_with_its_own_skeleton() {
    // `torso.fmdl` alone: reported by `check`, packed as `fcl_hair.fmdl`.
    let alone = Sandbox::new("mod_fallback_alone");
    alone.write(
        "exports/co - Torso/Players/05 - A/torso.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );
    let check = alone.run(&pes21_settings(&alone), &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co - Torso"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
        ]
    );
    assert_eq!(check.exit_code(), 0);
    let run = alone.run(&pes21_settings(&alone), &["compile"]);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&alone.root.join("output/4cc_90_test.cpk"));
    let package = face_package(&entries);
    let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
    assert_eq!(
        names,
        [
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair_sim.fclo",
            "fcl_hair_sim.skl"
        ]
    );

    // `torso.fmdl` beside `fcl_hair.fmdl`: one merged hair model.
    let merged = Sandbox::new("mod_fallback_merged");
    for name in ["torso.fmdl", "fcl_hair.fmdl"] {
        merged.write(
            &format!("exports/co - Merged/Players/05 - A/{name}"),
            &tracer_player_file("fcl_hair.fmdl"),
        );
    }
    let run = merged.run(&pes21_settings(&merged), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co - Merged"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=fcl_hair.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&merged.root.join("output/4cc_90_test.cpk"));
    let package = face_package(&entries);
    let model = FmdlFile::read(package.get("fcl_hair.fmdl").unwrap()).unwrap();
    let part = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    assert_eq!(
        Model::from_file(&model).unwrap().meshes.len(),
        2 * Model::from_file(&part).unwrap().meshes.len()
    );

    // `torso.fmdl` with `torso.skl`: that skeleton is the hair's.
    let paired = Sandbox::new("mod_fallback_skl");
    paired.write(
        "exports/co - Paired/Players/05 - A/torso.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );
    let custom = body_skl("pes19");
    paired.write("exports/co - Paired/Players/05 - A/torso.skl", &custom);
    let run = paired.run(&pes21_settings(&paired), &["compile"]);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&paired.root.join("output/4cc_90_test.cpk"));
    assert_eq!(
        face_package(&entries).get("fcl_hair_sim.skl").unwrap(),
        custom
    );
}

#[test]
fn a_pre_fox_target_reports_neither_the_fallback_nor_a_slotless_skeleton() {
    let sandbox = Sandbox::new("mod_pre_fox_names");
    for name in ["torso.fmdl", "face_high.fmdl", "face_high.skl"] {
        sandbox.write(&format!("exports/co - Names/Players/05 - A/{name}"), b"");
    }

    let run = sandbox.run("[common]\npes_version = 17\n", &["check"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Names"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(run.exit_code(), 0);
}

/// Writes a face folder at `folder`: the tracer's hair model as `face_high.fmdl`, its
/// `face_diff.bin`, and its `shirt.dds` under `texture_name`.
fn write_face(sandbox: &Sandbox, folder: &str, texture_name: &str) {
    sandbox.write(
        &format!("{folder}/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{folder}/face_diff.bin"),
        &tracer_player_file("face_diff.bin"),
    );
    sandbox.write(
        &format!("{folder}/{texture_name}"),
        &tracer_player_file("shirt.dds"),
    );
}

// TC-MOD-08
#[test]
fn a_face_link_combines_the_shared_face_folder_into_the_player_s_face() {
    let sandbox = Sandbox::new("mod_face_link");
    let export = "exports/co - Faces";
    write_face(&sandbox, &format!("{export}/Players/05 - A"), "shirt.dds");
    sandbox.write(&format!("{export}/Players/05 - A/Longhair.face"), b"");
    // The tracer's boots model stands in for the shared hair model.
    sandbox.write(
        &format!("{export}/Faces/Longhair/hair_high.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Faces"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info link_combined [Keep] at Players/05 - A (link=Longhair.face)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
        ],
        "nothing for Longhair on its own"
    );
    assert_eq!(
        package_names(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"]),
        ["face_diff.bin", "face_high.fmdl", "hair_high.fmdl"]
    );
}

#[test]
fn a_texture_the_face_and_a_combined_boots_folder_hold_is_packed_once_or_drops_the_boots() {
    let export = "exports/co - Skin";
    let skin = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/skin.ftex";
    let k0625 = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    let write = |sandbox: &Sandbox, shared_skin: &[u8]| {
        write_face(sandbox, &format!("{export}/Players/05 - A"), "skin.dds");
        sandbox.write(
            &format!("{export}/Players/05 - A/kit_boots.fmdl"),
            &tracer_player_file("boots.fmdl"),
        );
        sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
        sandbox.write(
            &format!("{export}/Boots/Crocs/boots.fmdl"),
            &tracer_player_file("boots.fmdl"),
        );
        sandbox.write(&format!("{export}/Boots/Crocs/skin.dds"), shared_skin);
        sandbox.write(
            &format!("{export}/Boots/Crocs/sole.dds"),
            &tracer_player_file("shirt.dds"),
        );
    };

    // The same bytes under one stem in both sources: packed once, no finding.
    let same = Sandbox::new("mod_skin_same");
    write(&same, &tracer_player_file("shirt.dds"));
    let run = same.run(&pes21_settings(&same), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co - Skin"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&same.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            k0625,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            skin,
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/sole.ftex",
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
        ]
    );

    // Different bytes: the face wins, the boots are left out with the texture only they
    // brought, and the player's own `skin` is the one packed.
    let differing = Sandbox::new("mod_skin_differing");
    write(&differing, &tracer_kit());
    let run = differing.run(&pes21_settings(&differing), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co - Skin"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
            "Error shared_texture_conflict [DropFolder] at Players/05 - A (texture=skin, dropped=boots)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&differing.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            skin,
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
        ],
        "no k0625 and no sole"
    );
    assert_eq!(
        entries[skin],
        ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap()
    );
}

#[test]
fn a_texture_the_player_s_folder_and_a_combined_face_folder_hold_differently_drops_the_player() {
    let sandbox = Sandbox::new("mod_face_texture_conflict");
    let export = "exports/co - Conflict";
    write_face(&sandbox, &format!("{export}/Players/05 - A"), "skin.dds");
    sandbox.write(&format!("{export}/Players/05 - A/Round.face"), b"");
    sandbox.write(
        &format!("{export}/Faces/Round/hair_high.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(&format!("{export}/Faces/Round/skin.dds"), &tracer_kit());
    // Slot 07: boots that compile, so the CPK is written.
    sandbox.write(
        &format!("{export}/Players/07 - B/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Conflict"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
            "Error merged_texture_conflict [DropFolder] at Players/05 - A (texture=skin)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
        ],
        "nothing of slot 05"
    );
}

// TC-MOD-09
#[test]
fn parts_with_a_skeleton_mismatch_or_a_material_defined_twice_drop_their_folder() {
    let sandbox = Sandbox::new("mod_merge_conflicts");
    let export = "exports/co - Conflicts";
    // Slot 05: two boots parts, one paired with a skeleton and one without.
    for name in ["boots.fmdl", "kit_boots.fmdl"] {
        sandbox.write(
            &format!("{export}/Players/05 - A/{name}"),
            &tracer_player_file("boots.fmdl"),
        );
    }
    sandbox.write(
        &format!("{export}/Players/05 - A/boots.skl"),
        &body_skl("pes21"),
    );
    sandbox.write(
        &format!("{export}/Players/05 - A/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    // Slot 07: the tracer's hair model as a boots part beside its boots: both define the
    // material `shirt`, differently.
    sandbox.write(
        &format!("{export}/Players/07 - B/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Players/07 - B/x_boots.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Players/07 - B/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    // Slot 09: boots that compile, so the CPK is written.
    sandbox.write(
        &format!("{export}/Players/09 - C/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Conflicts"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error skl_merge_conflict [DropFolder] at Players/05 - A (skeleton=differs)",
            "Error merge_material_conflict [DropFolder] at Players/07 - B (material=shirt)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0629/#Win/boots.fpk",
            "Asset/model/character/boots/k0629/#Win/boots.fpkd",
        ],
        "nothing of slot 05 or 07, their textures included"
    );
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
