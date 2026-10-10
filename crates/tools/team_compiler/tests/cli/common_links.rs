//! `compile` over `.common` links on Fox: a model link's Common model baked into the player's
//! package as a part, its skeleton with it, its textures left in the team's Common output, where
//! every texture directly in `Common/` goes once for the team; a texture link pointing the
//! player's own models at that output; and what a lenient file-type check keeps around them,
//! which nothing reads.

use std::collections::BTreeMap;

use fmdl::ops::paths::{rewrite_texture_paths, texture_paths};
use fmdl::{FmdlFile, Model};

use crate::bins::{install_cpk, install_names};
use crate::common::Sandbox;
use crate::compile::{
    compiled_players, cpk_entries, pes21_settings, pixels_cut_dds, tracer_kit, tracer_player_file,
};
use crate::conversion::HOME_714_05;
use crate::models::{body_skl, face_package, package_names};
use crate::prefox_faces::{card_materials, card_model, materials_naming, small_dds};
use crate::textures::texture_fixture;
use crate::{clean_model, findings_of};
use pes_model::format::mtl::MaterialSet;

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
pub(crate) fn texture_directories(bytes: &[u8], file_name: &str) -> Vec<String> {
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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
    // Named for the boots: a link is read directly in the player folder alone, and the linked
    // model's name gives its role.
    for slot in ["05 - A", "07 - B"] {
        sandbox.write(
            &format!("{export}/Players/{slot}/kit_boots.fmdl.common"),
            b"",
        );
    }
    sandbox.write(
        &format!("{export}/Common/kit_boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    sandbox.write(&format!("{export}/Common/kit_boots.skl"), &custom);
    sandbox.write(
        &format!("{export}/Common/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let shirt = format!("{COMMON_TEXTURES}/shirt.ftex");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Boots"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/kit_boots.fmdl (file=kit_boots.fmdl, count=1662)",
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);
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

/// Writes the export `export`'s `Common/legs.model`, the card head, with the card's material
/// set naming `skin` as `Common/legs.mtl`, and `Common/skin.dds`.
fn write_common_card(sandbox: &Sandbox, export: &str) {
    sandbox.write(&format!("{export}/Common/legs.model"), &card_model());
    sandbox.write(&format!("{export}/Common/legs.mtl"), &card_materials());
    sandbox.write(&format!("{export}/Common/skin.dds"), &small_dds());
}

/// The meshes of the card head converted to an FMDL: its one mesh and the anti-blur mesh the
/// FMDL export makes for it.
const CONVERTED_CARD_MESHES: usize = 2;

/// The card head as a model of the player's own: its one material renamed `torso`, in the
/// model and in its material set, which names `./face.dds`. It merges with the card itself
/// with no material in common, and its one bone is the card's, at the card's pose (a part of
/// another skeleton's pose would be `skl_merge_conflict`).
fn own_card() -> (Vec<u8>, Vec<u8>) {
    let file = pes_model::format::PreFoxModel::read(&card_model()).unwrap();
    let mut model = pes_model::model::Model::from_file(&file).unwrap();
    model.materials = vec!["torso".to_owned()];
    let mut materials = MaterialSet::read(&materials_naming("face")).unwrap();
    materials.materials[0].name = "torso".to_owned();
    (model.to_file().unwrap().write().unwrap(), materials.write())
}

#[test]
fn a_common_model_link_converts_the_model_with_its_common_mtl_into_the_player_s_face() {
    let sandbox = Sandbox::new("cmn_model_link");
    let export = "exports/co Midcup Card";
    let player = format!("{export}/Players/05 - A");
    let (torso, torso_materials) = own_card();
    sandbox.write(&format!("{player}/torso.model"), &torso);
    sandbox.write(&format!("{player}/torso.mtl"), &torso_materials);
    sandbox.write(&format!("{player}/face.dds"), &small_dds());
    sandbox.write(&format!("{player}/legs.model.common"), b"");
    write_common_card(&sandbox, export);
    let skin = format!("{COMMON_TEXTURES}/skin.ftex");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Card"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.model.common)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.model)",
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
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/face.ftex",
            skin.as_str(),
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ],
        "skin once, in the team's Common output"
    );
    assert_no_common_path(&entries);
    let package = face_package(&entries);
    let merged = package.get("fcl_hair.fmdl").unwrap();
    assert_eq!(
        mesh_count(merged),
        2 * CONVERTED_CARD_MESHES,
        "the local part's meshes plus the converted Common model's"
    );
    // The Common `.mtl`'s `./skin.dds`, pointed at the team's Common output, the player's own
    // `./face.dds` at his subfolder.
    let skin_directories = texture_directories(merged, "skin.dds");
    assert!(!skin_directories.is_empty());
    assert!(
        skin_directories
            .iter()
            .all(|directory| directory == COMMON_DIRECTORY),
        "{skin_directories:?}"
    );
    let face_directories = texture_directories(merged, "face.dds");
    assert!(!face_directories.is_empty());
    assert!(
        face_directories.iter().all(|directory| directory
            == "/Assets/pes16/model/character/common/714/05 - A/sourceimages/"),
        "{face_directories:?}"
    );
}

// TC-MOD-61
#[test]
fn a_common_model_converted_with_the_player_s_own_mtl_names_his_texture_of_its_stem() {
    let sandbox = Sandbox::new("cmn_model_local_mtl");
    let export = "exports/co Midcup Card";
    let player = format!("{export}/Players/05 - A");
    let (torso, torso_materials) = own_card();
    sandbox.write(&format!("{player}/torso.model"), &torso);
    sandbox.write(&format!("{player}/torso.mtl"), &torso_materials);
    sandbox.write(&format!("{player}/face.dds"), &small_dds());
    sandbox.write(&format!("{player}/legs.model.common"), b"");
    // His own `legs.mtl` beats `Common/`'s, and its `./skin.dds` is his own, other bytes than
    // Common's `skin.dds`.
    sandbox.write(&format!("{player}/legs.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &tracer_kit());
    write_common_card(&sandbox, export);
    let home = "Asset/model/character/common/714/05 - A/sourceimages/#windx11";
    let common_skin = format!("{COMMON_TEXTURES}/skin.ftex");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Card"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.model.common)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.model)",
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
            format!("{home}/face.ftex").as_str(),
            format!("{home}/skin.ftex").as_str(),
            common_skin.as_str(),
            FACE_05,
            FACE_05_FPKD,
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ],
        "his skin at his home, Common's packed by the Common textures task all the same"
    );
    let package = face_package(&entries);
    let merged = package.get("fcl_hair.fmdl").unwrap();
    // His `legs.mtl` set `./skin.dds`, so it resolves in his folder, where his `skin.dds` is.
    for file_name in ["skin.dds", "face.dds"] {
        let directories = texture_directories(merged, file_name);
        assert!(!directories.is_empty(), "{file_name}");
        assert!(
            directories.iter().all(|directory| directory == HOME_714_05),
            "{file_name}: {directories:?}"
        );
    }
}

#[test]
fn two_players_linking_one_common_model_each_convert_it_and_share_its_texture() {
    let sandbox = Sandbox::new("cmn_model_link_twice");
    let export = "exports/co Midcup Cards";
    for slot in ["05 - A", "07 - B"] {
        sandbox.write(&format!("{export}/Players/{slot}/legs.model.common"), b"");
    }
    write_common_card(&sandbox, export);
    let skin = format!("{COMMON_TEXTURES}/skin.ftex");
    let face_07 = "Asset/model/character/face/real/71407/#Win/face.fpk";

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Cards"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.model.common)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/07 - B (file=legs.model.common)",
            "Info team_colors_missing [Keep] ()"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            skin.as_str(),
            FACE_05,
            FACE_05_FPKD,
            face_07,
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin"
        ],
        "each player's face, the Common texture once"
    );
    for path in [FACE_05, face_07] {
        let package = fpk::FpkFile::read(&entries[path]).unwrap();
        let hair = package.get("fcl_hair.fmdl").unwrap();
        assert_eq!(mesh_count(hair), CONVERTED_CARD_MESHES, "{path}");
        let directories = texture_directories(hair, "skin.dds");
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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
    // A header the deep pass reads, over pixel data cut short: the conversion fails.
    sandbox.write(&format!("{export}/Common/broken.dds"), &pixels_cut_dds());
    sandbox.write(
        &format!("{export}/Common/cloth.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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

/// The texture link above with an FTEX for target: the link's stem is the name before
/// `.ftex.common`, as before `.dds.common`.
#[test]
fn a_link_to_an_ftex_points_the_player_s_model_at_the_one_copy_in_the_team_s_common_output() {
    let sandbox = Sandbox::new("cmn_texture_link_ftex");
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
    sandbox.write(&format!("{player}/hair.ftex.common"), b"");
    let hair_ftex =
        ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap();
    sandbox.write(&format!("{export}/Common/hair.ftex"), &hair_ftex);
    let hair = format!("{COMMON_TEXTURES}/hair.ftex");
    let skin = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/skin.ftex";

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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
}

/// The tracer's hair model with its `shirt.dds` renamed `hair.dds` and pointed at the game's
/// own Common texture folder, which names no team.
fn model_naming_game_hair() -> Vec<u8> {
    let mut file = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    rewrite_texture_paths(&mut file, |path| {
        if path.file_name == "shirt.dds" {
            path.file_name = "hair.dds".to_owned();
            path.directory = GAME_COMMON_DIRECTORY.to_owned();
        }
    })
    .unwrap();
    file.write()
}

/// The game's own Common texture folder, which names no team.
const GAME_COMMON_DIRECTORY: &str = "/Assets/pes16/model/character/common/sourceimages/";

#[test]
fn a_common_model_s_texture_a_link_of_the_player_s_stands_for_is_in_the_team_s_common_output() {
    // A midcup export: the player's `hair.dds.common` stands for a texture an earlier
    // installed CPK holds in the team's Common output, and `Common/` does not.
    let sandbox = Sandbox::new("cmn_model_linked_texture");
    let export = "exports/co Midcup Hair";
    let player = format!("{export}/Players/05 - A");
    sandbox.write(&format!("{player}/legs.fmdl.common"), b"");
    sandbox.write(&format!("{player}/hair.dds.common"), b"");
    sandbox.write(
        &format!("{export}/Common/legs.fmdl"),
        &model_naming_game_hair(),
    );
    install_names(&sandbox, &["4cc_61_midcup.cpk", "4cc_62_midcup.cpk"]);
    install_cpk(
        &sandbox,
        "4cc_61_midcup.cpk",
        &[(
            &format!("{COMMON_TEXTURES}/hair.ftex"),
            b"compiled on an earlier day",
        )],
    );
    let settings = format!(
        "{}[team-compiler]\ncpk_name = \"4cc_62_midcup\"\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Hair"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/legs.fmdl (file=legs.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_62_midcup.cpk"));
    let package = face_package(&entries);
    let hair = texture_directories(package.get("fcl_hair.fmdl").unwrap(), "hair.dds");
    assert!(!hair.is_empty());
    assert!(
        hair.iter().all(|directory| directory == COMMON_DIRECTORY),
        "{hair:?}"
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

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

// TC-MOD-54
#[test]
fn a_fox_model_whose_mtl_is_a_common_file_converts_with_it() {
    let sandbox = Sandbox::new("cmn_fox_mtl_link");
    let export = "exports/co Midcup Body";
    let player = format!("{export}/Players/05 - A");
    sandbox.write(&format!("{player}/body.model"), &card_model());
    sandbox.write(&format!("{player}/body.mtl.common"), b"");
    sandbox.write(&format!("{export}/Common/body.mtl"), &card_materials());
    sandbox.write(&format!("{export}/Common/skin.dds"), &small_dds());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Body"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=body.model)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    // The converted model's materials, each its name and shader, read back with `fmdl`, and
    // the directories its paths of the set's one texture, `./skin.dds`, name.
    let materials = |sandbox: &Sandbox| -> (Vec<(String, String)>, Vec<String>) {
        let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
        let face = face_package(&entries);
        let fmdl = face.get("fcl_hair.fmdl").unwrap();
        let directories = texture_directories(fmdl, "skin.dds");
        // The set's one sampler made it into the converted model.
        assert!(!directories.is_empty());
        let materials = Model::from_file(&FmdlFile::read(fmdl).unwrap())
            .unwrap()
            .materials
            .into_iter()
            .map(|material| (material.name, material.shader))
            .collect();
        (materials, directories)
    };
    let (linked, directories) = materials(&sandbox);
    let set = MaterialSet::read(&card_materials()).unwrap();
    assert!(!set.materials.is_empty());
    for material in &set.materials {
        assert!(
            linked.iter().any(|(name, _)| *name == material.name),
            "{linked:?}"
        );
    }
    // The set is Common's, so the texture it names is Common's: the team's Common output.
    assert!(
        directories
            .iter()
            .all(|directory| directory == COMMON_DIRECTORY),
        "{directories:?}"
    );
    // The same as the set gives the model as a local `.mtl` of its folder.
    let local = Sandbox::new("cmn_fox_mtl_local");
    local.write(&format!("{player}/body.model"), &card_model());
    local.write(&format!("{player}/body.mtl"), &card_materials());
    local.write(&format!("{player}/skin.dds"), &small_dds());
    let run = local.run(&pes21_settings(&local), &["compile", "--no-deploy"]);
    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let (local_materials, local_directories) = materials(&local);
    assert_eq!(linked, local_materials);
    // With `skin.dds` in the player's folder too, the folder's own texture wins, as the local
    // set's does: the player's texture home.
    let both = Sandbox::new("cmn_fox_mtl_link_own_texture");
    both.write(&format!("{player}/body.model"), &card_model());
    both.write(&format!("{player}/body.mtl.common"), b"");
    both.write(&format!("{player}/skin.dds"), &small_dds());
    both.write(&format!("{export}/Common/body.mtl"), &card_materials());
    both.write(&format!("{export}/Common/skin.dds"), &small_dds());
    let run = both.run(&pes21_settings(&both), &["compile", "--no-deploy"]);
    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let (_, own_directories) = materials(&both);
    assert!(
        local_directories
            .iter()
            .all(|directory| directory == HOME_714_05),
        "{local_directories:?}"
    );
    assert_eq!(own_directories, local_directories);
}

// TC-CMN-13
#[test]
fn a_texture_below_a_common_subfolder_is_not_used_on_fox() {
    let sandbox = Sandbox::new("cmn_nested");
    let export = "exports/co Midcup Nested";
    sandbox.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{export}/Common/sub/x.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Nested"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Warning file_not_used [Keep] (file=Common/sub/x.dds)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(
        entries
            .keys()
            .all(|path| !path.starts_with(&format!("{COMMON_TEXTURES}/x."))),
        "{:#?}",
        entries.keys()
    );
    assert!(entries.contains_key(FACE_05), "{:#?}", entries.keys());
}

// TC-CMN-14
#[test]
fn a_common_model_an_fmdl_of_its_stem_beats_is_not_checked_and_the_link_loads_the_fmdl() {
    let sandbox = Sandbox::new("cmn_beaten_model");
    let export = "exports/co Midcup Beaten";
    sandbox.write(&format!("{export}/Players/05 - A/boots.model.common"), b"");
    let boots = tracer_player_file("boots.fmdl");
    sandbox.write(&format!("{export}/Common/boots.fmdl"), &boots);
    // Four bytes no `.model` reader accepts.
    sandbox.write(&format!("{export}/Common/boots.model"), b"junk");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Beaten"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Common/boots.fmdl (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );
    let package = fpk::FpkFile::read(&entries[BOOTS_05]).unwrap();
    assert_eq!(
        mesh_count(package.get("boots.fmdl").unwrap()),
        mesh_count(&boots)
    );
}

#[test]
fn a_common_model_an_fmdl_the_deep_pass_drops_beats_is_dropped_with_it() {
    let sandbox = Sandbox::new("cmn_beaten_model_dropped");
    let export = "exports/co Midcup Beaten";
    sandbox.write(&format!("{export}/Players/05 - A/boots.fmdl.common"), b"");
    sandbox.write(
        &format!("{export}/Players/07 - B/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    // A link naming the beaten `.model`, which loads the FMDL of its stem.
    sandbox.write(&format!("{export}/Players/09 - C/boots.model.common"), b"");
    // Four bytes no FMDL reader accepts, beside a `.model` it beats, which nothing reads.
    sandbox.write(&format!("{export}/Common/boots.fmdl"), b"junk");
    sandbox.write(&format!("{export}/Common/boots.model"), &card_model());
    sandbox.write(&format!("{export}/Common/boots.mtl"), &card_materials());
    sandbox.write(&format!("{export}/Common/skin.dds"), &small_dds());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Beaten"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=face_high.fmdl, count=1662)",
            "Error model_broken [DropFile] at Common/boots.fmdl (file=boots.fmdl, error=fmdl is truncated)",
            "Info common_model_beaten_dropped [DropFile] at Common/boots.model (file=Common/boots.model, winner=Common/boots.fmdl)",
            "Error link_target_dropped [DropFolder] at Players/05 - A (link=boots.fmdl.common, target=Common/boots.fmdl, finding=model_broken)",
            "Error link_target_dropped [DropFolder] at Players/09 - C (link=boots.model.common, target=Common/boots.model, finding=common_model_beaten_dropped)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    // No boots converted from the `.model`, nor any other: slot 07's face alone.
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(
        entries
            .keys()
            .all(|path| !path.starts_with("Asset/model/character/boots/")),
        "{:#?}",
        entries.keys()
    );
    assert_eq!(compiled_players(&sandbox), [71407]);
}

#[test]
fn a_beaten_common_model_is_not_dropped_for_a_dropped_file_of_its_stem_that_is_no_model() {
    let sandbox = Sandbox::new("cmn_beaten_model_texture_dropped");
    let export = "exports/co Midcup Beaten";
    sandbox.write(&format!("{export}/Players/05 - A/boots.fmdl.common"), b"");
    sandbox.write(&format!("{export}/Common/boots.fmdl"), &clean_model());
    sandbox.write(&format!("{export}/Common/boots.model"), &card_model());
    // A texture of the stem the deep pass drops (a PNG under a `.dds` name): it beats no
    // model, so the `.model` the FMDL beats stays, unread, and the link loads the FMDL.
    sandbox.write(
        &format!("{export}/Common/boots.dds"),
        &texture_fixture("kit.png"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Beaten"),
        [
            "Error texture_type_mismatch [DropFile] at Common/boots.dds (file=boots.dds)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );
}

#[test]
fn a_common_texture_the_decoder_refuses_is_dropped_alone_and_the_others_are_packed() {
    let sandbox = Sandbox::new("cmn_texture_unreadable");
    let export = "exports/co Midcup Studs";
    sandbox.write(&format!("{export}/Players/05 - A/studs.dds.common"), b"");
    sandbox.write(
        &format!("{export}/Common/studs.dds"),
        &tracer_player_file("shirt.dds"),
    );
    // Four bytes that open with no accepted format's signature: no header to read.
    sandbox.write(&format!("{export}/Common/boots.dds"), b"junk");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Studs"),
        [
            "Error texture_unreadable [DropFile] at Common/boots.dds (file=boots.dds)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let common: Vec<&str> = entries
        .keys()
        .map(String::as_str)
        .filter(|path| path.starts_with(COMMON_TEXTURES))
        .collect();
    assert_eq!(common, [format!("{COMMON_TEXTURES}/studs.ftex")]);
}

/// Writes the export `export`: slot 05 holding `model_name` (`model`) and `boots.mtl.common`,
/// and `Common/boots.mtl` holding bytes no `.mtl` reader accepts.
fn write_broken_common_mtl(sandbox: &Sandbox, export: &str, model_name: &str, model: &[u8]) {
    let player = format!("{export}/Players/05 - A");
    sandbox.write(&format!("{player}/{model_name}"), model);
    sandbox.write(&format!("{player}/boots.mtl.common"), b"");
    sandbox.write(&format!("{export}/Common/boots.mtl"), b"not a material set");
}

#[test]
fn a_common_mtl_no_search_reads_is_not_checked_on_pes_21() {
    let sandbox = Sandbox::new("cmn_fox_mtl_unread");
    let export = "exports/co Midcup Boots";
    // An FMDL carries its materials: no search reads the Common `.mtl` his link names.
    write_broken_common_mtl(&sandbox, export, "boots.fmdl", &clean_model());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Boots"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );

    // A `.model` in its place is converted with the `.mtl` its search finds through the link:
    // that one is read, and its Error drops the file, after which his search finds none.
    let model = Sandbox::new("cmn_fox_mtl_searched");
    write_broken_common_mtl(&model, export, "boots.model", &card_model());
    let run = model.run(&pes21_settings(&model), &["compile", "--no-deploy"]);
    let lines = run.messages();
    let error = pes_model::format::mtl::MaterialSet::read(b"not a material set")
        .unwrap_err()
        .to_string();
    assert_eq!(
        findings_of(&lines, "co Midcup Boots"),
        [
            "Error model_material_undefined [DropFolder] at Players/05 - A (file=boots.model)",
            format!(
                "Error mtl_broken [DropFile] at Common/boots.mtl (file=boots.mtl, error={error})"
            )
            .as_str(),
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
}

#[test]
fn a_common_mtl_no_link_names_is_not_checked_when_a_model_pairs_locally_on_pes_21() {
    let sandbox = Sandbox::new("cmn_fox_mtl_local_pair");
    let export = "exports/co Midcup Boots";
    let player = format!("{export}/Players/05 - A");
    // His `.model` pairs with his own `.mtl`: his search never reaches `Common/`.
    sandbox.write(&format!("{player}/boots.model"), &card_model());
    sandbox.write(&format!("{player}/boots.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    sandbox.write(&format!("{export}/Common/stray.mtl"), b"not a material set");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Boots"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        package_names(&entries[BOOTS_05]),
        ["boots.fmdl", "boots.skl"]
    );
}

#[test]
fn a_common_mtl_a_link_s_search_falls_back_to_is_checked_on_pes_21() {
    let sandbox = Sandbox::new("cmn_fox_mtl_fallback");
    let export = "exports/co Midcup Legs";
    sandbox.write(&format!("{export}/Players/05 - A/legs.model.common"), b"");
    sandbox.write(&format!("{export}/Common/legs.model"), &card_model());
    // The search's first candidate, dropped: the folders' pass then lands on `materials.mtl`.
    sandbox.write(&format!("{export}/Common/legs.mtl"), b"not a material set");
    // The card's material set with its one material, `card`, renamed: the fallback defines
    // no material the card binds.
    let text = String::from_utf8(card_materials()).unwrap();
    assert!(text.contains("name=\"card\""), "{text}");
    sandbox.write(
        &format!("{export}/Common/materials.mtl"),
        text.replace("name=\"card\"", "name=\"sleeve\"").as_bytes(),
    );
    sandbox.write(&format!("{export}/Common/skin.dds"), &small_dds());

    let run = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    let lines = run.messages();
    let error = MaterialSet::read(b"not a material set")
        .unwrap_err()
        .to_string();
    assert_eq!(
        findings_of(&lines, "co Midcup Legs"),
        [
            "Error model_material_undefined [DropFolder] at Players/05 - A (file=legs.model.common, mtl=Common/materials.mtl, materials=card)",
            format!(
                "Error mtl_broken [DropFile] at Common/legs.mtl (file=legs.mtl, error={error})"
            )
            .as_str(),
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
}

#[test]
fn a_shared_folder_s_common_link_is_disallowed_and_read_by_nothing() {
    let sandbox = Sandbox::new("cmn_shared_link");
    let export = "exports/co Midcup Crocs";
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(&format!("{export}/Boots/Crocs/legs.fmdl.common"), b"");
    let settings = format!(
        "{}[team-compiler]\nstrict_file_type_check = false\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Crocs"),
        [
            "Info file_type_disallowed [Keep] at Boots/Crocs (file=legs.fmdl.common)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // The shared boots compile on their own, under the team's first shared id.
    let shared_boots = "Asset/model/character/boots/k0644/#Win/boots.fpk";
    assert_eq!(
        package_names(&entries[shared_boots]),
        ["boots.fmdl", "boots.skl"]
    );
    assert_no_common_path(&entries);
}

#[test]
fn a_shared_folder_s_mtl_link_gives_its_model_no_material_set() {
    // The deep pass's `.mtl` search resolves no link of a shared folder, as the boots task's
    // does not: the `.model` has every material undefined, and the folder is dropped.
    let sandbox = Sandbox::new("cmn_shared_mtl_link");
    let export = "exports/co Midcup Crocs";
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Players/07 - B/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(&format!("{export}/Boots/Crocs/boots.model"), &card_model());
    sandbox.write(&format!("{export}/Boots/Crocs/boots.mtl.common"), b"");
    sandbox.write(&format!("{export}/Common/boots.mtl"), &card_materials());
    sandbox.write(&format!("{export}/Common/skin.dds"), &small_dds());
    let settings = format!(
        "{}[team-compiler]\nstrict_file_type_check = false\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Crocs"),
        [
            "Info file_type_disallowed [Keep] at Boots/Crocs (file=boots.mtl.common)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=face_high.fmdl, count=1662)",
            "Error model_material_undefined [DropFolder] at Boots/Crocs (file=boots.model)",
            "Error link_target_dropped [DropFolder] at Players/05 - A (link=Crocs.boots, target=Boots/Crocs, finding=model_material_undefined)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    assert_eq!(compiled_players(&sandbox), [71407]);
}

#[test]
fn a_combined_shared_folder_s_mtl_link_is_not_resolved_in_common() {
    // The deep pass's search sees no `Common/` file for a shared folder, so `boots.model` there
    // takes `materials.mtl`; the player's boots task, which combines the folder, searches it
    // the same way, and never reaches the Common `.mtl` the name-matched link would stand for.
    let sandbox = Sandbox::new("cmn_combined_mtl_link");
    let export = "exports/co Midcup Crocs";
    let player = format!("{export}/Players/05 - A");
    let (own, own_materials) = own_card();
    sandbox.write(&format!("{player}/boots.model"), &own);
    sandbox.write(&format!("{player}/boots.mtl"), &own_materials);
    sandbox.write(&format!("{player}/face.dds"), &small_dds());
    sandbox.write(&format!("{player}/Crocs.boots"), b"");
    let crocs = format!("{export}/Boots/Crocs");
    sandbox.write(&format!("{crocs}/boots.model"), &card_model());
    sandbox.write(&format!("{crocs}/materials.mtl"), &card_materials());
    sandbox.write(&format!("{crocs}/skin.dds"), &small_dds());
    sandbox.write(&format!("{crocs}/boots.mtl.common"), b"");
    sandbox.write(&format!("{export}/Common/boots.mtl"), &card_materials());
    let settings = format!(
        "{}[team-compiler]\nstrict_file_type_check = false\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Crocs"),
        [
            "Info file_type_disallowed [Keep] at Boots/Crocs (file=boots.mtl.common)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)"
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let package = fpk::FpkFile::read(&entries[BOOTS_05]).unwrap();
    let boots = package.get("boots.fmdl").unwrap();
    assert_eq!(mesh_count(boots), 2 * CONVERTED_CARD_MESHES);
    // The shared `materials.mtl`'s `./skin.dds`, the shared folder's texture, at his home.
    let directories = texture_directories(boots, "skin.dds");
    assert!(!directories.is_empty());
    assert!(
        directories.iter().all(|directory| directory == HOME_714_05),
        "{directories:?}"
    );
}

#[test]
fn an_unlinked_common_model_builds_nothing() {
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
    let run = unlinked.run(&pes21_settings(&unlinked), &["compile", "--no-deploy"]);
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
