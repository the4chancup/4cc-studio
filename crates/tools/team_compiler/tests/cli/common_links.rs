//! `compile` over `.common` links on Fox: a model link's Common model baked into the player's
//! package as a part, its skeleton with it, its textures left in the team's Common output, where
//! every texture directly in `Common/` goes once for the team; a texture link pointing the
//! player's own models at that output; and what the gate still refuses around them.

use std::collections::BTreeMap;

use fmdl::ops::paths::texture_paths;
use fmdl::{FmdlFile, Model};

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_player_file};
use crate::findings_of;
use crate::models::{body_skl, face_package, package_names};

const FACE_05: &str = "Asset/model/character/face/real/71405/#Win/face.fpk";
const FACE_05_FPKD: &str = "Asset/model/character/face/real/71405/#Win/face.fpkd";
const BOOTS_05: &str = "Asset/model/character/boots/k0625/#Win/boots.fpk";
/// The team's Common output, the Common row of the game paths.
const COMMON_TEXTURES: &str = "Asset/model/character/common/714/sourceimages/#windx11";
const COMMON_DIRECTORY: &str = "/Assets/pes16/model/character/common/714/sourceimages/";

/// The tracer's hair model as a Common model of its own: its materials renamed with a
/// `_common` tail and its `shirt.dds` renamed `cloth.dds`, through `fmdl`'s model API, so it
/// merges with the hair itself with no material in common (two copies of one FMDL would define
/// every material twice, over textures in different places).
fn common_hair_model() -> Vec<u8> {
    let file = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    let mut model = Model::from_file(&file).unwrap();
    for material in &mut model.materials {
        material.name.push_str("_common");
        for (_, texture) in &mut material.textures {
            if texture.file_name == "shirt.dds" {
                texture.file_name = "cloth.dds".to_owned();
            }
        }
    }
    model.to_file().unwrap().write()
}

/// The mesh count of the FMDL `bytes`.
fn mesh_count(bytes: &[u8]) -> usize {
    Model::from_file(&FmdlFile::read(bytes).unwrap())
        .unwrap()
        .meshes
        .len()
}

/// The directory of every texture path of the FMDL `bytes` naming `file_name`.
fn texture_directories(bytes: &[u8], file_name: &str) -> Vec<String> {
    texture_paths(&FmdlFile::read(bytes).unwrap())
        .unwrap()
        .into_iter()
        .filter(|path| path.file_name == file_name)
        .map(|path| path.directory)
        .collect()
}

/// Asserts no entry of the CPK is a link file or a Common model's own path: the link is baked
/// away and the model travels inside a package.
fn assert_no_common_path(entries: &BTreeMap<String, Vec<u8>>) {
    for path in entries.keys() {
        assert!(
            !path.ends_with(".common") && !path.starts_with("Common"),
            "{path}"
        );
    }
}

// TC-MOD-10
// TC-MOD-11
#[test]
fn a_common_model_link_bakes_the_model_into_the_face_and_its_texture_stays_in_the_team_s_common() {
    let sandbox = Sandbox::new("cmn_face_link");
    let export = "exports/co Midcup Link";
    let player = format!("{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/torso.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{player}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    sandbox.write(&format!("{player}/legs.fmdl.common"), b"");
    sandbox.write(&format!("{export}/Common/legs.fmdl"), &common_hair_model());
    sandbox.write(
        &format!("{export}/Common/cloth.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let cloth = format!("{COMMON_TEXTURES}/cloth.ftex");
    let shirt = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex";

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Link"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=torso.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
            "Info team_colors_missing [Keep] ()",
            "Info fmdl_merged [Keep] at Players/05 - A (model=fcl_hair.fmdl)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            shirt,
            cloth.as_str(),
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ],
        "cloth once, in the team's Common output and not under the player's subfolder"
    );
    assert_no_common_path(&entries);
    assert_eq!(
        entries[&cloth],
        ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap()
    );
    let package = face_package(&entries);
    assert_eq!(
        package_names(&entries[FACE_05]),
        [
            "face_diff.bin",
            "fcl_hair.fmdl",
            "fcl_hair_sim.fclo",
            "fcl_hair_sim.skl"
        ]
    );
    let merged = package.get("fcl_hair.fmdl").unwrap();
    assert_eq!(
        mesh_count(merged),
        2 * mesh_count(&tracer_player_file("fcl_hair.fmdl")),
        "the local part's meshes plus the Common model's"
    );
    // The Common model's `cloth` points at the team's Common output, the player's own `shirt`
    // at the player's subfolder.
    let cloth_directories = texture_directories(merged, "cloth.dds");
    assert!(!cloth_directories.is_empty());
    assert!(
        cloth_directories
            .iter()
            .all(|directory| directory == COMMON_DIRECTORY),
        "{cloth_directories:?}"
    );
    let shirt_directories = texture_directories(merged, "shirt.dds");
    assert!(!shirt_directories.is_empty());
    assert!(
        shirt_directories.iter().all(|directory| directory
            == "/Assets/pes16/model/character/common/714/05 - A/sourceimages/"),
        "{shirt_directories:?}"
    );
}

#[test]
fn a_boots_link_packs_the_common_skeleton_and_two_players_linking_one_model_share_its_texture() {
    let sandbox = Sandbox::new("cmn_boots_link");
    let export = "exports/co Midcup Boots";
    for slot in ["05 - A", "07 - B"] {
        sandbox.write(
            &format!("{export}/Players/{slot}/boots/legs.fmdl.common"),
            b"",
        );
    }
    sandbox.write(
        &format!("{export}/Common/legs.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    sandbox.write(&format!("{export}/Common/legs.skl"), &custom);
    sandbox.write(
        &format!("{export}/Common/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let shirt = format!("{COMMON_TEXTURES}/shirt.ftex");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Boots"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            shirt.as_str(),
            FACE_05,
            FACE_05_FPKD,
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "each player's package and blank face, the Common texture once"
    );
    assert_no_common_path(&entries);
    for path in [BOOTS_05, "Asset/model/character/boots/k0627/#Win/boots.fpk"] {
        let package = fpk::FpkFile::read(&entries[path]).unwrap();
        let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
        assert_eq!(names, ["boots.fmdl", "boots.skl"], "{path}");
        assert_eq!(package.get("boots.skl").unwrap(), custom, "{path}");
        let directories = texture_directories(package.get("boots.fmdl").unwrap(), "shirt.dds");
        assert!(!directories.is_empty());
        assert!(
            directories
                .iter()
                .all(|directory| directory == COMMON_DIRECTORY),
            "{path}: {directories:?}"
        );
    }
}

#[test]
fn two_link_spellings_of_one_common_model_compile_it_once() {
    // `boots.fmdl.common` and the tolerated `boots.fmdl.common.txt` are one link to
    // one model: the boots task reads the model once, not twice.
    let sandbox = Sandbox::new("cmn_link_twice");
    let export = "exports/co Midcup Links";
    sandbox.write(&format!("{export}/Players/05 - A/boots.fmdl.common"), b"");
    sandbox.write(
        &format!("{export}/Players/05 - A/boots.fmdl.common.txt"),
        b"",
    );
    sandbox.write(
        &format!("{export}/Common/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let package = fpk::FpkFile::read(&entries[BOOTS_05]).unwrap();
    assert_eq!(
        package.entries().map(|(name, _)| name).collect::<Vec<_>>(),
        ["boots.fmdl", "boots.skl"]
    );
}

#[test]
fn a_common_skeleton_pairs_with_its_model_case_folded() {
    // `Common/Boots.skl` names `boots.fmdl`'s skeleton as the file system folds it.
    let sandbox = Sandbox::new("cmn_skl_case");
    let export = "exports/co Midcup Case";
    sandbox.write(&format!("{export}/Players/05 - A/boots.fmdl.common"), b"");
    sandbox.write(
        &format!("{export}/Common/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    sandbox.write(&format!("{export}/Common/Boots.skl"), &custom);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let package = fpk::FpkFile::read(&entries[BOOTS_05]).unwrap();
    assert_eq!(package.get("boots.skl").unwrap(), custom);
}

#[test]
fn a_boots_link_beside_a_shared_boots_link_combines_the_shared_folder() {
    let sandbox = Sandbox::new("cmn_link_combined");
    let export = "exports/co Midcup Combined";
    sandbox.write(
        &format!("{export}/Players/05 - A/kit_boots.fmdl.common"),
        b"",
    );
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Common/kit_boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Combined"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Common/kit_boots.fmdl (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no k0644: the shared folder is only a source of parts"
    );
    let package = fpk::FpkFile::read(&entries[BOOTS_05]).unwrap();
    assert_eq!(
        mesh_count(package.get("boots.fmdl").unwrap()),
        2 * mesh_count(&tracer_player_file("boots.fmdl"))
    );
}

#[test]
fn a_common_skeleton_of_a_slotless_face_model_is_reported_on_the_link_and_not_packed() {
    let sandbox = Sandbox::new("cmn_skl_no_slot");
    let export = "exports/co Midcup Slot";
    sandbox.write(
        &format!("{export}/Players/05 - A/face_high.fmdl.common"),
        b"",
    );
    sandbox.write(
        &format!("{export}/Common/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Common/face_high.skl"),
        &tracer_player_file("fcl_hair.skl"),
    );
    let findings = [
        "Info fmdl_weights_not_normalized [Keep] at Common/face_high.fmdl (file=face_high.fmdl, count=1662)",
        "Info export_identified [Keep] (team=/co/, id=714)",
        "Warning skl_no_slot [Keep] at Players/05 - A (file=face_high.fmdl.common)",
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(findings_of(&check.messages(), "co Midcup Slot"), findings);
    assert_eq!(check.exit_code(), 0);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Slot"),
        [&findings[..], &["Info team_colors_missing [Keep] ()"]].concat()
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        package_names(&entries[FACE_05]),
        ["face_diff.bin", "face_high.fmdl"]
    );
}

#[test]
fn a_material_a_local_and_a_common_part_define_over_textures_in_two_places_drops_the_folder() {
    let sandbox = Sandbox::new("cmn_material_conflict");
    let export = "exports/co Midcup Conflict";
    let player = format!("{export}/Players/05 - A");
    // The same hair model locally and in Common, each beside a `shirt.dds` of its own.
    sandbox.write(
        &format!("{player}/torso.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(&format!("{player}/torso2.fmdl.common"), b"");
    sandbox.write(
        &format!("{player}/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    sandbox.write(
        &format!("{export}/Common/torso2.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Common/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    // Slot 07: boots that compile, so the CPK is written.
    sandbox.write(
        &format!("{export}/Players/07 - B/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Conflict"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=torso.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Common/torso2.fmdl (file=torso2.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso2.fmdl.common)",
            "Info team_colors_missing [Keep] ()",
            "Error merge_material_conflict [DropFolder] at Players/05 - A (material=shirt)"
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            &format!("{COMMON_TEXTURES}/shirt.ftex"),
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "nothing of slot 05, its own shirt included; the Common texture is the team's"
    );
}

#[test]
fn a_common_texture_that_cannot_convert_fails_the_common_task_and_the_linking_player_still_builds()
{
    let sandbox = Sandbox::new("cmn_texture_failed");
    let export = "exports/co Midcup Broken";
    sandbox.write(&format!("{export}/Players/05 - A/legs.fmdl.common"), b"");
    sandbox.write(
        &format!("{export}/Common/legs.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    // The DDS with its header cut off: nothing can read it as a texture.
    let cut = tracer_player_file("shirt.dds")[128..].to_vec();
    sandbox.write(&format!("{export}/Common/broken.dds"), &cut);
    sandbox.write(
        &format!("{export}/Common/cloth.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let findings = findings_of(&lines, "co Midcup Broken");
    assert_eq!(findings.len(), 5, "{findings:?}");
    assert_eq!(
        findings[..4],
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert!(
        findings[4].starts_with(
            "Error folder_pack_failed [DropFolder] at Common (error=broken.dds: cannot convert"
        ),
        "{}",
        findings[4]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ],
        "no Common texture, `cloth` included; the player's face is in"
    );
}

/// The tracer's hair model naming `hair.dds` in its `shirt` material and `skin.dds` in every
/// other material that named `shirt.dds`, through `fmdl`'s model API: one model with a path
/// to each texture.
fn model_naming_hair_and_skin() -> Vec<u8> {
    let file = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    let mut model = Model::from_file(&file).unwrap();
    for material in &mut model.materials {
        let renamed = if material.name == "shirt" {
            "hair.dds"
        } else {
            "skin.dds"
        };
        for (_, texture) in &mut material.textures {
            if texture.file_name == "shirt.dds" {
                texture.file_name = renamed.to_owned();
            }
        }
    }
    model.to_file().unwrap().write()
}

// TC-TEX-09
#[test]
fn a_texture_link_points_the_player_s_model_at_the_one_copy_in_the_team_s_common_output() {
    let sandbox = Sandbox::new("cmn_texture_link");
    let export = "exports/co Midcup Hair";
    let player = format!("{export}/Players/05 - A");
    sandbox.write(
        &format!("{player}/face_high.fmdl"),
        &model_naming_hair_and_skin(),
    );
    sandbox.write(
        &format!("{player}/skin.dds"),
        &tracer_player_file("shirt.dds"),
    );
    sandbox.write(&format!("{player}/hair.dds.common"), b"");
    sandbox.write(
        &format!("{export}/Common/hair.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let hair = format!("{COMMON_TEXTURES}/hair.ftex");
    let skin = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/skin.ftex";

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Hair"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            skin,
            hair.as_str(),
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ],
        "hair once, in the team's Common output; the player's own folder holds skin alone"
    );
    assert_no_common_path(&entries);
    let package = face_package(&entries);
    let model = package.get("face_high.fmdl").unwrap();
    let hair_directories = texture_directories(model, "hair.dds");
    assert!(!hair_directories.is_empty());
    assert!(
        hair_directories
            .iter()
            .all(|directory| directory == COMMON_DIRECTORY),
        "{hair_directories:?}"
    );
    let skin_directories = texture_directories(model, "skin.dds");
    assert!(!skin_directories.is_empty());
    assert!(
        skin_directories.iter().all(|directory| directory
            == "/Assets/pes16/model/character/common/714/05 - A/sourceimages/"),
        "{skin_directories:?}"
    );
}

#[test]
fn a_texture_link_whose_target_is_not_in_common_drops_its_folder() {
    let sandbox = Sandbox::new("cmn_texture_link_missing");
    let export = "exports/co Midcup Hair";
    sandbox.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(&format!("{export}/Players/05 - A/hair.dds.common"), b"");
    sandbox.write(
        &format!("{export}/Common/skin.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Hair"),
        [
            "Error common_link_missing [DropFolder] at Players/05 - A (link=hair.dds.common, path=Common/hair.dds)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn a_link_to_a_material_file_or_a_nested_common_file_is_refused_and_an_unlinked_common_model_is_not()
 {
    // A `.common` link to a material file names the export's first thing `compile` cannot
    // build.
    let material_link = Sandbox::new("cmn_material_link");
    let export = "exports/co Midcup Hair";
    material_link.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    material_link.write(&format!("{export}/Players/05 - A/body.mtl.common"), b"");
    // A real material set, so the deep pass keeps it and the link reaches the gate.
    let mtl = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../libs/pes_model/tests/fixtures/konami_shadow.mtl"),
    )
    .unwrap();
    material_link.write(&format!("{export}/Common/body.mtl"), &mtl);
    let run = material_link.run(&pes21_settings(&material_link), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Hair"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info mtl_state_missing [Keep] at Common/body.mtl (file=body.mtl, count=7)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Players/05 - A/body.mtl.common)"
        ]
    );
    assert_eq!(run.exit_code(), 1);

    // A file under a subfolder of `Common/`, kept by the non-strict file-type check, is named.
    let nested = Sandbox::new("cmn_nested");
    let export = "exports/co Midcup Nested";
    nested.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    nested.write(
        &format!("{export}/Common/sub/x.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let settings = format!(
        "{}[team-compiler]\nstrict_file_type_check = false\n",
        pes21_settings(&nested)
    );
    let run = nested.run(&settings, &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Nested"),
        [
            "Info common_file_disallowed [Keep] at Common/sub/x.dds ()",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Common/sub/x.dds)"
        ]
    );
    assert_eq!(run.exit_code(), 1);

    // A Common model no link names is accepted and builds nothing.
    let unlinked = Sandbox::new("cmn_unlinked");
    let export = "exports/co Midcup Spare";
    unlinked.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    unlinked.write(
        &format!("{export}/Common/spare.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    unlinked.write(
        &format!("{export}/Common/spare.skl"),
        &tracer_player_file("fcl_hair.skl"),
    );
    let run = unlinked.run(&pes21_settings(&unlinked), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Spare"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Common/spare.fmdl (file=spare.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&unlinked.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ]
    );
}
