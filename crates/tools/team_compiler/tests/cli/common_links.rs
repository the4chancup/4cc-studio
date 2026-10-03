//! `compile` over `.common` model links on Fox: the Common model baked into the player's
//! package as a part, its skeleton with it, its textures left in the team's Common output, where
//! every texture directly in `Common/` goes once for the team; and what the gate still refuses
//! around them.

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
    let export = "exports/co - Link";
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
        findings_of(&run.messages(), "co - Link"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=fcl_hair.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [shirt, cloth.as_str(), FACE_05, FACE_05_FPKD],
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
    let export = "exports/co - Boots";
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
        findings_of(&run.messages(), "co - Boots"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            shirt.as_str(),
        ],
        "each player's package, the Common texture once"
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
fn a_boots_link_beside_a_shared_boots_link_combines_the_shared_folder() {
    let sandbox = Sandbox::new("cmn_link_combined");
    let export = "exports/co - Combined";
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
            BOOTS_05,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
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
    let export = "exports/co - Slot";
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
        "Info export_identified [Keep] (team=/co/, id=714)",
        "Warning skl_no_slot [Keep] at Players/05 - A (file=face_high.fmdl.common)",
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(findings_of(&check.messages(), "co - Slot"), findings);
    assert_eq!(check.exit_code(), 0);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);
    assert_eq!(findings_of(&run.messages(), "co - Slot"), findings);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    assert_eq!(
        package_names(&entries[FACE_05]),
        ["face_diff.bin", "face_high.fmdl"]
    );
}

#[test]
fn a_material_a_local_and_a_common_part_define_over_textures_in_two_places_drops_the_folder() {
    let sandbox = Sandbox::new("cmn_material_conflict");
    let export = "exports/co - Conflict";
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
        findings_of(&run.messages(), "co - Conflict"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso2.fmdl.common)",
            "Error merge_material_conflict [DropFolder] at Players/05 - A (material=shirt)",
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
            &format!("{COMMON_TEXTURES}/shirt.ftex"),
        ],
        "nothing of slot 05, its own shirt included; the Common texture is the team's"
    );
}

#[test]
fn a_common_texture_that_cannot_convert_fails_the_common_task_and_the_linking_player_still_builds()
{
    let sandbox = Sandbox::new("cmn_texture_failed");
    let export = "exports/co - Broken";
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
    let findings = findings_of(&lines, "co - Broken");
    assert_eq!(findings.len(), 3, "{findings:?}");
    assert_eq!(
        findings[..2],
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=legs.fmdl.common)",
        ]
    );
    assert!(
        findings[2].starts_with(
            "Error folder_pack_failed [DropFolder] at Common (error=broken.dds: cannot convert to FTEX"
        ),
        "{}",
        findings[2]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [FACE_05, FACE_05_FPKD],
        "no Common texture, `cloth` included; the player's face is in"
    );
}

#[test]
fn a_link_to_a_texture_or_a_nested_common_file_is_refused_and_an_unlinked_common_model_is_not() {
    // A `.common` link to a texture names the export's first thing `compile` cannot build.
    let texture_link = Sandbox::new("cmn_texture_link");
    let export = "exports/co - Hair";
    texture_link.write(
        &format!("{export}/Players/05 - A/face_high.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    texture_link.write(&format!("{export}/Players/05 - A/hair.dds.common"), b"");
    texture_link.write(
        &format!("{export}/Common/hair.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let run = texture_link.run(&pes21_settings(&texture_link), &["compile"]);
    assert_eq!(
        findings_of(&run.messages(), "co - Hair"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Players/05 - A/hair.dds.common)",
        ]
    );
    assert_eq!(run.exit_code(), 1);

    // A file under a subfolder of `Common/`, kept by the non-strict file-type check, is named.
    let nested = Sandbox::new("cmn_nested");
    let export = "exports/co - Nested";
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
        findings_of(&run.messages(), "co - Nested"),
        [
            "Info common_file_disallowed [Keep] at Common/sub/x.dds ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Common/sub/x.dds)",
        ]
    );
    assert_eq!(run.exit_code(), 1);

    // A Common model no link names is accepted and builds nothing.
    let unlinked = Sandbox::new("cmn_unlinked");
    let export = "exports/co - Spare";
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
        findings_of(&run.messages(), "co - Spare"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&unlinked.root.join("output/4cc_90_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(paths, [FACE_05, FACE_05_FPKD]);
}
