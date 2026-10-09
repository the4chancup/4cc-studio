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

use crate::bins::{BOOTS_LIST, GLOVE_LIST, install_tables, item_list, pairs_of};
use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes21_settings, tracer_kit, tracer_player_file};
use crate::conversion::HOME_714_05;
use crate::face_folders::assert_blank_face;
use crate::prefox_faces::{
    BOOTS_K0644, CLEAN, card_materials, card_model, compile_pes17, entries_under, materials_naming,
    small_dds,
};
use crate::{TEAM_COLORS_MISSING, clean_model, command_args, findings_of};

/// The entry names of the FPK `bytes`.
pub(crate) fn package_names(bytes: &[u8]) -> Vec<String> {
    fpk::FpkFile::read(bytes)
        .unwrap()
        .entries()
        .map(|(name, _)| name.to_owned())
        .collect()
}

/// One of the bundled game skeletons, `resources/skeletons/<version>/body.skl`.
pub(crate) fn body_skl(version: &str) -> Vec<u8> {
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

/// Compiles the sandbox for PES 21, asserting the export `name` reports exactly `findings`
/// and the run exits with 0, and returns the CPK's entries by path.
pub(crate) fn compile_clean(
    sandbox: &Sandbox,
    name: &str,
    findings: &[&str],
) -> BTreeMap<String, Vec<u8>> {
    let run = sandbox.run(&pes21_settings(sandbox), &["compile", "--no-deploy"]);
    assert_eq!(findings_of(&run.messages(), name), findings);
    assert_eq!(run.exit_code(), 0);
    cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"))
}

// TC-MOD-01
#[test]
fn a_player_s_own_boots_and_gloves_compile_under_its_exclusive_id_with_the_textures_once() {
    let sandbox = Sandbox::new("mod_boots_gloves");
    write_player(
        &sandbox,
        "exports/co Midcup Models/Players/05 - A",
        "kit_boots.fmdl",
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Models",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

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
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
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
        "exports/co Midcup Paired/Players/05 - A",
        "kit_boots.fmdl",
    );
    // A real skeleton that differs from the bundled PES 21 one.
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    paired.write(
        "exports/co Midcup Paired/Players/05 - A/kit_boots.skl",
        &custom,
    );
    let entries = compile_clean(
        &paired,
        "co Midcup Paired",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );
    let package = fpk::FpkFile::read(&entries[boots_fpk]).unwrap();
    assert_eq!(package.get("boots.skl").unwrap(), custom);

    let bare = Sandbox::new("mod_boots_skl_bundled");
    write_player(
        &bare,
        "exports/co Midcup Bare/Players/05 - A",
        "kit_boots.fmdl",
    );
    let entries = compile_clean(
        &bare,
        "co Midcup Bare",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );
    let package = fpk::FpkFile::read(&entries[boots_fpk]).unwrap();
    assert_eq!(package.get("boots.skl").unwrap(), body_skl("pes21"));
}

#[test]
fn a_skeleton_named_like_its_model_but_the_case_pairs_too() {
    // `Kit_Boots.skl` names `kit_boots.fmdl`'s skeleton as the file system folds it.
    let sandbox = Sandbox::new("mod_boots_skl_case");
    write_player(
        &sandbox,
        "exports/co Midcup Case/Players/05 - A",
        "kit_boots.fmdl",
    );
    let custom = body_skl("pes19");
    assert_ne!(custom, body_skl("pes21"));
    sandbox.write(
        "exports/co Midcup Case/Players/05 - A/Kit_Boots.skl",
        &custom,
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Case",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let package =
        fpk::FpkFile::read(&entries["Asset/model/character/boots/k0625/#Win/boots.fpk"]).unwrap();
    assert_eq!(package.get("boots.skl").unwrap(), custom);
}

#[test]
fn a_face_diff_bin_by_any_case_packs_as_face_diff_bin() {
    // `FACE_DIFF.BIN` is `face_diff.bin` as the file system folds it.
    let sandbox = Sandbox::new("mod_face_diff_case");
    let export = "exports/co Midcup Case";
    write_player(
        &sandbox,
        &format!("{export}/Players/05 - A"),
        "kit_boots.fmdl",
    );
    std::fs::remove_file(
        sandbox
            .root
            .join(format!("{export}/Players/05 - A/face_diff.bin")),
    )
    .unwrap();
    sandbox.write(
        &format!("{export}/Players/05 - A/Face_Diff.bin"),
        &tracer_player_file("face_diff.bin"),
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Case",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let package =
        fpk::FpkFile::read(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"])
            .unwrap();
    assert_eq!(
        package.get("face_diff.bin").unwrap(),
        tracer_player_file("face_diff.bin")
    );
}

#[test]
fn a_texture_stem_names_its_file_case_folded() {
    // `Shirt.dds` is the texture the model's `shirt` path names, folded alike.
    let sandbox = Sandbox::new("mod_stem_case");
    let export = "exports/co Midcup Case";
    write_player(
        &sandbox,
        &format!("{export}/Players/05 - A"),
        "kit_boots.fmdl",
    );
    std::fs::remove_file(
        sandbox
            .root
            .join(format!("{export}/Players/05 - A/shirt.dds")),
    )
    .unwrap();
    sandbox.write(
        &format!("{export}/Players/05 - A/Shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Case",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let package =
        fpk::FpkFile::read(&entries["Asset/model/character/boots/k0625/#Win/boots.fpk"]).unwrap();
    let directories: Vec<String> =
        texture_paths(&FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap())
            .unwrap()
            .into_iter()
            .filter(|path| path.file_name == "shirt.dds")
            .map(|path| path.directory)
            .collect();
    assert!(!directories.is_empty());
    assert!(
        directories.iter().all(|directory| directory
            == "/Assets/pes16/model/character/common/714/05 - A/sourceimages/"),
        "{directories:?}"
    );
}

// TC-MOD-03
#[test]
fn a_folder_mapped_to_two_slots_emits_its_boots_under_both_ids_and_its_textures_once() {
    let sandbox = Sandbox::new("mod_two_slots");
    sandbox.write("exports/co Midcup Twice/players.txt", b"03 A\n07 A\n");
    for name in ["boots.fmdl", "shirt.dds"] {
        sandbox.write(
            &format!("exports/co Midcup Twice/Players/A/{name}"),
            &tracer_player_file(name),
        );
    }

    let entries = compile_clean(
        &sandbox,
        "co Midcup Twice",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0623/#Win/boots.fpk",
            "Asset/model/character/boots/k0623/#Win/boots.fpkd",
            "Asset/model/character/boots/k0627/#Win/boots.fpk",
            "Asset/model/character/boots/k0627/#Win/boots.fpkd",
            "Asset/model/character/common/714/A/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/face/real/71403/#Win/face.fpk",
            "Asset/model/character/face/real/71403/#Win/face.fpkd",
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
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
    write_player(
        &sandbox,
        "exports/co Midcup Broken/Players/05 - A",
        "boots.fmdl",
    );
    // The DDS with its header cut off: nothing can read it as a texture.
    let cut = tracer_player_file("shirt.dds")[128..].to_vec();
    sandbox.write("exports/co Midcup Broken/Players/05 - A/shirt.dds", &cut);
    write_player(
        &sandbox,
        "exports/co Midcup Broken/Players/07 - B",
        "boots.fmdl",
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    let findings = findings_of(&lines, "co Midcup Broken");
    assert_eq!(findings.len(), 9, "{findings:?}");
    assert_eq!(
        findings[..8],
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert!(
        findings[8].starts_with(
            "Error folder_pack_failed [DropFolder] at Players/05 - A (error=shirt.dds: cannot convert"
        ),
        "{}",
        findings[8]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
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
pub(crate) fn boots_mesh_count(entries: &BTreeMap<String, Vec<u8>>, path: &str) -> usize {
    let package = fpk::FpkFile::read(&entries[path]).unwrap();
    let model = FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap();
    Model::from_file(&model).unwrap().meshes.len()
}

/// The mesh count of the tracer's boots model.
pub(crate) fn tracer_boots_mesh_count() -> usize {
    let model = FmdlFile::read(&tracer_player_file("boots.fmdl")).unwrap();
    Model::from_file(&model).unwrap().meshes.len()
}

// TC-MOD-07
#[test]
fn a_boots_link_beside_a_local_boots_model_combines_the_shared_folder_into_the_player_s_boots() {
    let sandbox = Sandbox::new("mod_link_combined");
    let export = "exports/co Midcup Combined";
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Combined"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
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
            "Asset/model/character/boots/k0625/#Win/boots.fpk",
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex",
            sole,
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "Asset/model/character/glove/g0625/#Win/glove.fpk",
            "Asset/model/character/glove/g0625/#Win/glove.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
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
    // it, while slot 05's merge is unchanged. Slot 07 holds no face model: its face is blank.
    sandbox.write(&format!("{export}/Players/07 - B/Crocs.boots"), b"");
    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Combined"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
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
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
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
pub(crate) fn template(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../resources/templates")
            .join(name),
    )
    .unwrap()
}

/// The face package of slot 05 in `entries`.
pub(crate) fn face_package(entries: &BTreeMap<String, Vec<u8>>) -> fpk::FpkFile {
    fpk::FpkFile::read(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"]).unwrap()
}

// TC-MOD-12
#[test]
fn a_hair_model_alone_gets_the_bundled_face_diff_hair_simulation_and_body_skeleton() {
    let sandbox = Sandbox::new("mod_face_templates");
    sandbox.write(
        "exports/co Midcup Hair/Players/05 - A/fcl_hair.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Hair",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

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

#[test]
fn a_templates_face_diff_replaces_the_bundled_one_in_a_face_whose_folder_holds_none() {
    let sandbox = Sandbox::new("mod_face_template_override");
    // The tracer's own face diff, a real one other than the bundled template.
    let template_face_diff = tracer_player_file("face_diff.bin");
    assert_ne!(template_face_diff, template("face_diff.bin"));
    sandbox.write("data/templates/face_diff.bin", &template_face_diff);
    sandbox.write(
        "exports/co Midcup Hair/Players/05 - A/fcl_hair.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Hair",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let package = face_package(&entries);
    assert!(
        package.get("face_diff.bin").unwrap() == template_face_diff,
        "the face diff is the templates folder's"
    );
    assert!(
        package.get("fcl_hair_sim.fclo").unwrap() == template("fcl_hair_sim.fclo"),
        "a resource with no file in the folder is the bundled one"
    );
}

// TC-MOD-13
#[test]
fn a_skeleton_paired_with_a_face_model_without_a_slot_is_reported_and_not_packed() {
    let sandbox = Sandbox::new("mod_skl_no_slot");
    sandbox.write(
        "exports/co Midcup Slot/Players/05 - A/face_high.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        "exports/co Midcup Slot/Players/05 - A/face_high.skl",
        &tracer_player_file("fcl_hair.skl"),
    );
    let findings = [
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
        "Info export_identified [Keep] (team=/co/, id=714)",
        "Warning skl_no_slot [Keep] at Players/05 - A (file=face_high.skl)",
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
    let package = face_package(&entries);
    let names: Vec<&str> = package.entries().map(|(name, _)| name).collect();
    assert_eq!(names, ["face_diff.bin", "face_high.fmdl"]);
}

#[test]
fn an_unsuffixed_model_is_reported_and_merged_into_the_hair_with_its_own_skeleton() {
    // `torso.fmdl` alone: reported by `check`, packed as `fcl_hair.fmdl`.
    let alone = Sandbox::new("mod_fallback_alone");
    alone.write(
        "exports/co Midcup Torso/Players/05 - A/torso.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );
    let check = alone.run(&pes21_settings(&alone), &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co Midcup Torso"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=torso.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)"
        ]
    );
    assert_eq!(check.exit_code(), 0);
    let run = alone.run(&pes21_settings(&alone), &["compile", "--no-deploy"]);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&alone.root.join("output/4cc_99_test.cpk"));
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
            &format!("exports/co Midcup Merged/Players/05 - A/{name}"),
            &tracer_player_file("fcl_hair.fmdl"),
        );
    }
    let run = merged.run(&pes21_settings(&merged), &["compile", "--no-deploy"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Merged"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=torso.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=torso.fmdl)",
            "Info team_colors_missing [Keep] ()",
            "Info fmdl_merged [Keep] at Players/05 - A (model=fcl_hair.fmdl)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&merged.root.join("output/4cc_99_test.cpk"));
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
        "exports/co Midcup Paired/Players/05 - A/torso.fmdl",
        &tracer_player_file("fcl_hair.fmdl"),
    );
    let custom = body_skl("pes19");
    paired.write("exports/co Midcup Paired/Players/05 - A/torso.skl", &custom);
    let run = paired.run(&pes21_settings(&paired), &["compile", "--no-deploy"]);
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&paired.root.join("output/4cc_99_test.cpk"));
    assert_eq!(
        face_package(&entries).get("fcl_hair_sim.skl").unwrap(),
        custom
    );
}

// TC-MOD-14
#[test]
fn a_boots_subfolder_s_model_is_the_boots_and_a_common_subfolder_s_texture_is_the_player_s() {
    let sandbox = Sandbox::new("mod_reserved_subfolders");
    let player = "exports/co Midcup Subfolders/Players/05 - A";
    sandbox.write(
        &format!("{player}/boots/hair_high.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{player}/common/skin.dds"),
        &tracer_player_file("shirt.dds"),
    );
    let boots_fpk = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    let skin = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/skin.ftex";

    let entries = compile_clean(
        &sandbox,
        "co Midcup Subfolders",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots/hair_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            boots_fpk,
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            skin,
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "the blank face for a folder whose only model is in boots/"
    );
    assert_eq!(
        package_names(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"]),
        ["face_diff.bin"]
    );
    assert_eq!(
        package_names(&entries[boots_fpk]),
        ["boots.fmdl", "boots.skl"]
    );
    assert_eq!(
        entries[skin],
        ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap()
    );
}

/// The face diff fixture `name`, from `tests/fixtures/face_diff/`.
pub(crate) fn face_diff_fixture(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/face_diff")
            .join(name),
    )
    .unwrap()
}

/// Copies the tracer's face into slot 05 of `export`, its `face_diff.bin` replaced by a
/// `face_diff.xml` holding `xml`.
fn write_xml_face(sandbox: &Sandbox, export: &str, xml: &[u8]) {
    let player = format!("{export}/Players/05 - A");
    sandbox.copy_tracer_face(&player);
    fs::remove_file(sandbox.root.join(&player).join("face_diff.bin")).unwrap();
    sandbox.write(&format!("{player}/face_diff.xml"), xml);
}

/// Copies the tracer's face into slot 07 of `export`, beside the slot 05 under test, so the
/// export still writes a CPK when slot 05's folder is dropped.
fn write_slot_07(sandbox: &Sandbox, export: &str) {
    sandbox.copy_tracer_face(&format!("{export}/Players/07 - B"));
}

/// Checks, then compiles, the sandbox for PES 21 (slot 05 under test, slot 07 written by
/// `write_slot_07`), asserting both commands report `finding` on slot 05 before the export
/// `name`'s identity (and `compile` its missing team colors after it), and that slot 05's
/// folder is not in the CPK while slot 07's is.
fn assert_face_dropped(sandbox: &Sandbox, name: &str, finding: &str) {
    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(sandbox), &command_args(command));
        let validated = [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            finding,
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ];
        // Planning's note, which only `compile` makes.
        let planned: &[&str] = if command == "compile" {
            &[TEAM_COLORS_MISSING]
        } else {
            &[]
        };
        assert_eq!(
            findings_of(&run.messages(), name),
            [&validated[..], planned].concat(),
            "{command}"
        );
        assert_eq!(run.exit_code(), 1, "{command}");
    }
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // Slot 05's face and portrait are player 71405's; `/co/`'s block starts at 621, so its
    // boots and gloves are k0625 and g0625, slot 07's 627.
    assert!(
        !entries
            .keys()
            .any(|path| path.contains("71405") || path.contains("0625/")),
        "{:?}",
        entries.keys()
    );
    assert!(entries.contains_key("Asset/model/character/face/real/71407/#Win/face.fpk"));
    assert!(entries.contains_key("Asset/model/character/boots/k0627/#Win/boots.fpk"));
}

// TC-MOD-15
#[test]
fn a_face_diff_xml_is_decoded_into_the_face_and_a_corrupt_one_drops_the_folder() {
    let sandbox = Sandbox::new("mod_face_diff_xml");
    write_xml_face(
        &sandbox,
        "exports/co Midcup Xml",
        &face_diff_fixture("dif.xml"),
    );
    let clean = [
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
        "Info export_identified [Keep] (team=/co/, id=714)",
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(findings_of(&check.messages(), "co Midcup Xml"), clean);
    assert_eq!(check.exit_code(), 0);
    let entries = compile_clean(
        &sandbox,
        "co Midcup Xml",
        &[&clean[..], &["Info team_colors_missing [Keep] ()"]].concat(),
    );

    assert_eq!(
        face_package(&entries).get("face_diff.bin").unwrap(),
        face_diff_fixture("dif.bin")
    );

    // One payload character replaced by a character base64 does not use.
    let mut corrupt = face_diff_fixture("dif.xml");
    let payload = corrupt
        .windows(4)
        .position(|bytes| bytes == b"RkFD")
        .unwrap();
    assert!(corrupt[payload + 100].is_ascii_alphanumeric());
    corrupt[payload + 100] = b'*';
    let corrupt_sandbox = Sandbox::new("mod_face_diff_xml_corrupt");
    write_xml_face(&corrupt_sandbox, "exports/co Midcup Corrupt", &corrupt);
    write_slot_07(&corrupt_sandbox, "exports/co Midcup Corrupt");

    assert_face_dropped(
        &corrupt_sandbox,
        "co Midcup Corrupt",
        "Error face_diff_invalid [DropFolder] at Players/05 - A \
         (file=face_diff.xml, reason=the base64 text holds '*' where base64 cannot)",
    );
}

// TC-MOD-42
#[test]
fn a_face_diff_given_twice_or_shorter_than_its_header_drops_the_folder() {
    // Both forms in one folder: neither is read, the conflict is the finding.
    let twice = Sandbox::new("mod_face_diff_twice");
    twice.copy_tracer_face("exports/co Midcup Twice/Players/05 - A");
    twice.write(
        "exports/co Midcup Twice/Players/05 - A/face_diff.xml",
        &face_diff_fixture("dif.xml"),
    );
    write_slot_07(&twice, "exports/co Midcup Twice");
    assert_face_dropped(
        &twice,
        "co Midcup Twice",
        "Error xml_dif_conflict [DropFolder] at Players/05 - A (file=face_diff.xml)",
    );

    // A `face_diff.bin` one byte shorter than its header gives.
    let cut = Sandbox::new("mod_face_diff_cut");
    cut.copy_tracer_face("exports/co Midcup Cut/Players/05 - A");
    cut.write(
        "exports/co Midcup Cut/Players/05 - A/face_diff.bin",
        &face_diff_fixture("dif.bin")[..943],
    );
    write_slot_07(&cut, "exports/co Midcup Cut");
    assert_face_dropped(
        &cut,
        "co Midcup Cut",
        "Error face_diff_invalid [DropFolder] at Players/05 - A \
         (file=face_diff.bin, reason=the face diff is 943 bytes long, but its \
         header gives 944: the game would read past its end)",
    );
}

#[test]
fn a_subfolder_s_parts_combine_with_loose_root_files_of_their_category() {
    let sandbox = Sandbox::new("mod_subfolder_parts");
    let player = "exports/co Midcup Parts/Players/05 - A";
    for (name, tracer_name) in [
        ("fcl_hair.fmdl", "fcl_hair.fmdl"),
        ("face/torso.fmdl", "fcl_hair.fmdl"),
        ("glove_l.fmdl", "glove_l.fmdl"),
        ("gloves/glove_r.fmdl", "glove_r.fmdl"),
    ] {
        sandbox.write(
            &format!("{player}/{name}"),
            &tracer_player_file(tracer_name),
        );
    }
    let glove_fpk = "Asset/model/character/glove/g0625/#Win/glove.fpk";

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Parts"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face/torso.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
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
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            glove_fpk,
            "Asset/model/character/glove/g0625/#Win/glove.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ]
    );
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
    let model = FmdlFile::read(package.get("fcl_hair.fmdl").unwrap()).unwrap();
    let part = FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap();
    assert_eq!(
        Model::from_file(&model).unwrap().meshes.len(),
        2 * Model::from_file(&part).unwrap().meshes.len()
    );
    assert_eq!(
        package_names(&entries[glove_fpk]),
        ["glove_l.fmdl", "glove_r.fmdl"]
    );
}

#[test]
fn a_pre_fox_target_reports_neither_the_fallback_nor_a_slotless_skeleton() {
    let sandbox = Sandbox::new("mod_pre_fox_names");
    for name in ["torso.fmdl", "face_high.fmdl"] {
        sandbox.write(
            &format!("exports/co Midcup Names/Players/05 - A/{name}"),
            &clean_model(),
        );
    }
    sandbox.write("exports/co Midcup Names/Players/05 - A/face_high.skl", b"");

    let run = sandbox.run("[common]\npes_version = 17\n", &["check"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Names"),
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
    let export = "exports/co Midcup Faces";
    write_face(&sandbox, &format!("{export}/Players/05 - A"), "shirt.dds");
    sandbox.write(&format!("{export}/Players/05 - A/Longhair.face"), b"");
    // The tracer's boots model stands in for the shared hair model.
    sandbox.write(
        &format!("{export}/Faces/Longhair/hair_high.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Faces"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Faces/Longhair (file=hair_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Longhair.face)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "nothing for Longhair on its own"
    );
    assert_eq!(
        package_names(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"]),
        ["face_diff.bin", "face_high.fmdl", "hair_high.fmdl"]
    );
}

// TC-MOD-55
#[test]
fn a_shared_face_s_boots_model_is_a_part_of_the_linking_player_s_own_boots() {
    let sandbox = Sandbox::new("mod_face_link_boots");
    install_tables(&sandbox, &[(BOOTS_LIST, &item_list(&[(70201, 11)]))]);
    let export = "exports/co Midcup Round";
    sandbox.write(&format!("{export}/Players/05 - A/Round.face"), b"");
    for name in ["fcl_hair.fmdl", "boots.fmdl"] {
        sandbox.write(
            &format!("{export}/Faces/Round/{name}"),
            &tracer_player_file(name),
        );
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Round"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // Slot 05's exclusive id in team 714's block: 101 + (714 - 701) * 40 + 5 - 1.
    let boots = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    assert_eq!(package_names(&entries[boots]), ["boots.fmdl", "boots.skl"]);
    assert_eq!(boots_mesh_count(&entries, boots), tracer_boots_mesh_count());
    assert_eq!(pairs_of(&entries, BOOTS_LIST), [(70201, 11), (71405, 625)]);
}

// TC-MOD-58
#[test]
fn a_shared_face_s_boots_model_converts_with_the_face_folder_s_mtl() {
    let sandbox = Sandbox::new("mod_face_link_boots_model");
    install_tables(&sandbox, &[(BOOTS_LIST, &item_list(&[(70201, 11)]))]);
    let export = "exports/co Midcup Round";
    sandbox.write(&format!("{export}/Players/05 - A/Round.face"), b"");
    sandbox.write(
        &format!("{export}/Faces/Round/fcl_hair.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(&format!("{export}/Faces/Round/boots.model"), &card_model());
    sandbox.write(
        &format!("{export}/Faces/Round/boots.mtl"),
        &card_materials(),
    );
    // The texture the card's `.mtl` names, as beside the pair in a player folder.
    sandbox.write(&format!("{export}/Faces/Round/skin.dds"), &small_dds());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Round"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=fcl_hair.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // Slot 05's exclusive id in team 714's block.
    let boots = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    assert_eq!(package_names(&entries[boots]), ["boots.fmdl", "boots.skl"]);
    let package = fpk::FpkFile::read(&entries[boots]).unwrap();
    // The card's one bone is the game's own: the conversion writes no skeleton, so the boots
    // get the bundled one.
    assert_eq!(package.get("boots.skl").unwrap(), body_skl("pes21"));
    // The `.mtl`'s `./skin.dds`, pointed at the player's texture home, which the combined
    // face folder's textures join.
    let file = FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap();
    let skin: Vec<(String, String)> = texture_paths(&file)
        .unwrap()
        .into_iter()
        .filter(|path| path.file_name.starts_with("skin."))
        .map(|path| (path.directory, path.file_name))
        .collect();
    assert!(!skin.is_empty(), "the boots converted with Round's .mtl");
    for path in &skin {
        assert_eq!(path, &(HOME_714_05.to_owned(), "skin.dds".to_owned()));
    }
    assert_eq!(pairs_of(&entries, BOOTS_LIST), [(70201, 11), (71405, 625)]);
}

// TC-MOD-59
#[test]
fn a_boots_link_combines_with_the_boots_of_the_player_s_shared_face() {
    let sandbox = Sandbox::new("mod_face_link_boots_link");
    install_tables(&sandbox, &[(BOOTS_LIST, &item_list(&[(70201, 11)]))]);
    let export = "exports/co Midcup Round";
    sandbox.write(&format!("{export}/Players/05 - A/Round.face"), b"");
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    for name in ["fcl_hair.fmdl", "boots.fmdl"] {
        sandbox.write(
            &format!("{export}/Faces/Round/{name}"),
            &tracer_player_file(name),
        );
    }
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Round"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // Slot 05's exclusive id in team 714's block.
    let boots = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    assert_eq!(package_names(&entries[boots]), ["boots.fmdl", "boots.skl"]);
    assert_eq!(
        boots_mesh_count(&entries, boots),
        2 * tracer_boots_mesh_count(),
        "Round's meshes plus Crocs's"
    );
    assert_eq!(pairs_of(&entries, BOOTS_LIST), [(70201, 11), (71405, 625)]);
    // The block's first shared id, which Crocs would take if a player linked it plainly.
    assert!(
        !entries
            .keys()
            .any(|path| path.starts_with("Asset/model/character/boots/k0644/")),
        "{:?}",
        entries.keys()
    );
}

// TC-MOD-60
#[test]
fn a_gloves_link_combines_with_the_hands_split_from_the_player_s_face_model() {
    let sandbox = Sandbox::new("mod_hand_split_gloves_link");
    install_tables(&sandbox, &[(GLOVE_LIST, &item_list(&[]))]);
    // A full-body model with both hands on (`tests/fixtures/hand_split/README.md`): 40 faces,
    // of which each hand's split takes 8 and the body keeps 24.
    let body =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hand_split/body.fmdl"))
            .unwrap();
    let export = "exports/co Midcup Hands";
    sandbox.write(&format!("{export}/Players/05 - A/body.fmdl"), &body);
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.gloves"), b"");
    // Crocs's gloves are the same strip under each glove's name, which is never split: their
    // bones are the split hands' own, so the two merge. An authored glove's hand bone has a
    // parent, which the strip's has not, and would not merge with them (the README).
    for name in ["glove_l.fmdl", "glove_r.fmdl"] {
        sandbox.write(&format!("{export}/Gloves/Crocs/{name}"), &body);
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co Midcup Hands"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=body.fmdl)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.gloves)",
            "Info model_hand_split [Keep] at Players/05 - A (model=body.fmdl, gloves=glove_l, glove_r)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=glove_l.fmdl)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=glove_r.fmdl)",
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    // Slot 05's exclusive id in team 714's block.
    let gloves = "Asset/model/character/glove/g0625/#Win/glove.fpk";
    assert_eq!(
        package_names(&entries[gloves]),
        ["glove_l.fmdl", "glove_r.fmdl"]
    );
    // Each hand's 8 split faces and Crocs's glove of that name, the whole strip.
    assert_eq!(
        [
            face_count(&entries, gloves, "glove_l.fmdl"),
            face_count(&entries, gloves, "glove_r.fmdl"),
        ],
        [8 + 40, 8 + 40]
    );
    assert_eq!(pairs_of(&entries, GLOVE_LIST), [(71405, 625)]);
    // The block's first shared id, which Crocs would take if a player linked it plainly.
    assert!(
        !entries
            .keys()
            .any(|path| path.starts_with("Asset/model/character/glove/g0644/")),
        "{:?}",
        entries.keys()
    );
}

// TC-MOD-04
#[test]
fn a_texture_the_face_and_a_combined_boots_folder_hold_is_packed_once_or_drops_the_boots() {
    let export = "exports/co Midcup Skin";
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
    let run = same.run(&pes21_settings(&same), &["compile", "--no-deploy"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Skin"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)"
        ]
    );
    assert_eq!(run.exit_code(), 0);
    let entries = cpk_entries(&same.root.join("output/4cc_99_test.cpk"));
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
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ]
    );

    // Different bytes: the face wins, the boots are left out with the texture only they
    // brought, and the player's own `skin` is the one packed.
    let differing = Sandbox::new("mod_skin_differing");
    write(&differing, &tracer_kit());
    let run = differing.run(&pes21_settings(&differing), &["compile", "--no-deploy"]);
    assert_eq!(
        findings_of(&run.messages(), "co Midcup Skin"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
            "Error shared_texture_conflict [DropFolder] at Players/05 - A (texture=skin, dropped=boots)"
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&differing.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            skin,
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no k0625 and no sole"
    );
    assert_eq!(
        entries[skin],
        ftex::dds_to_ftex(&tracer_player_file("shirt.dds"), ftex::ColorSpace::Normal).unwrap()
    );
}

#[test]
fn under_ingame_face_a_texture_the_player_and_a_combined_boots_folder_hold_differently_drops_him() {
    let sandbox = Sandbox::new("mod_ingame_face_skin_differing");
    let export = "exports/co Midcup Skin";
    let player = format!("{export}/Players/05 - A");
    // His own folder feeds the boots, there being no face: one package, two sources.
    sandbox.write(&format!("{player}/ingame_face"), b"");
    sandbox.write(
        &format!("{player}/kit_boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{player}/skin.dds"),
        &tracer_player_file("shirt.dds"),
    );
    sandbox.write(&format!("{player}/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(&format!("{export}/Boots/Crocs/skin.dds"), &tracer_kit());
    sandbox.write(
        &format!("{export}/Boots/Crocs/sole.dds"),
        &tracer_player_file("shirt.dds"),
    );
    // Slot 07: boots that compile, so the CPK is written.
    sandbox.write(
        &format!("{export}/Players/07 - B/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Skin"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info fmdl_merged [Keep] at Players/05 - A (model=boots.fmdl)",
            "Error merged_texture_conflict [DropFolder] at Players/05 - A (texture=skin)"
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
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "slot 07 alone: no k0625 and no texture of slot 05"
    );
}

// TC-MOD-33
#[test]
fn a_texture_the_player_s_folder_and_a_combined_face_folder_hold_differently_drops_the_player() {
    let sandbox = Sandbox::new("mod_face_texture_conflict");
    let export = "exports/co Midcup Conflict";
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Conflict"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=face_high.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Faces/Round (file=hair_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info link_combined [Keep] at Players/05 - A (link=Round.face)",
            "Error merged_texture_conflict [DropFolder] at Players/05 - A (texture=skin)"
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
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "nothing of slot 05; slot 07's face is blank"
    );
}

// TC-MOD-09
#[test]
fn parts_with_a_skeleton_mismatch_or_a_material_defined_twice_leave_their_boots_out() {
    let sandbox = Sandbox::new("mod_merge_conflicts");
    let export = "exports/co Midcup Conflicts";
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
    // Slot 07: the tracer's boots beside a copy whose `shirt` material another shader draws:
    // both define `shirt`, differently. (A copy differing only in its textures' directories
    // would not: every part's `shirt` is pointed at the player's own `shirt.dds` before the
    // merge.)
    sandbox.write(
        &format!("{export}/Players/07 - B/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    let mut other_shader =
        Model::from_file(&FmdlFile::read(&tracer_player_file("boots.fmdl")).unwrap()).unwrap();
    let shirt = other_shader
        .materials
        .iter_mut()
        .find(|material| material.name == "shirt")
        .unwrap();
    shirt.shader = "fox3ddf_blin".to_owned();
    shirt.technique = "fox3DDF_Blin".to_owned();
    sandbox.write(
        &format!("{export}/Players/07 - B/x_boots.fmdl"),
        &other_shader.to_file().unwrap().write(),
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

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Conflicts"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=kit_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=x_boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/09 - C (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Error skl_merge_conflict [DropFolder] at Players/05 - A (skeleton=differs)",
            "Error merge_material_conflict [DropFolder] at Players/07 - B (material=shirt)"
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0629/#Win/boots.fpk",
            "Asset/model/character/boots/k0629/#Win/boots.fpkd",
            "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/common/714/07 - B/sourceimages/#windx11/shirt.ftex",
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "Asset/model/character/face/real/71409/#Win/face.fpk",
            "Asset/model/character/face/real/71409/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no k0625 or k0627: the failed boots are left out; each folder's blank face commits \
         with the folder's textures"
    );
    // Each surviving face is the blank one: neither folder held a face model, slot 09
    // included.
    for player in [71405, 71407, 71409] {
        assert_blank_face(
            &entries,
            &format!("Asset/model/character/face/real/{player}/#Win/face.fpk"),
        );
    }
}

// TC-MOD-05
#[test]
fn shared_boots_folders_compile_once_each_under_the_shared_ids_in_name_order() {
    let sandbox = Sandbox::new("mod_shared_boots");
    // Crocs brings its own skeleton, Mud none, so each package says which folder it came from.
    let crocs_skl = body_skl("pes19");
    assert_ne!(crocs_skl, body_skl("pes21"));
    write_shared_boots(&sandbox, "co Midcup Shared", "Mud", None, &[11]);
    write_shared_boots(
        &sandbox,
        "co Midcup Shared",
        "Crocs",
        Some(&crocs_skl),
        &[3, 7],
    );

    let entries = compile_clean(
        &sandbox,
        "co Midcup Shared",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Boots/Crocs (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Mud (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

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
            "Asset/model/character/face/real/71403/#Win/face.fpk",
            "Asset/model/character/face/real/71403/#Win/face.fpkd",
            "Asset/model/character/face/real/71407/#Win/face.fpk",
            "Asset/model/character/face/real/71407/#Win/face.fpkd",
            "Asset/model/character/face/real/71411/#Win/face.fpk",
            "Asset/model/character/face/real/71411/#Win/face.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ],
        "no k0623, k0627 or k0631 for the linking slots; each one's face is blank"
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
        write_shared_boots(
            &sandbox,
            "co Midcup Pool",
            &format!("S{slot:02}"),
            None,
            &[slot],
        );
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(
        run.messages(),
        [
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S01 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S02 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S03 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S04 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S05 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S06 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S07 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S08 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S09 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S10 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S11 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S12 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S13 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S14 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S15 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S16 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S17 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info fmdl_weights_not_normalized [Keep] at Boots/S18 (file=boots.fmdl, count=1662)",
            "co Midcup Pool: Info export_identified [Keep] (team=/co/, id=714)",
            "co Midcup Pool: Error boots_id_pool_exhausted [DropExport] (count=18)"
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
    write_shared_boots(&sandbox, "co Midcup Named", "Zebra", None, &[3]);
    write_shared_boots(&sandbox, "co Midcup Named", "Apple", Some(&apple_skl), &[7]);
    let k0644 = "Asset/model/character/boots/k0644/#Win/boots.fpk";
    let k0645 = "Asset/model/character/boots/k0645/#Win/boots.fpk";
    let k0646 = "Asset/model/character/boots/k0646/#Win/boots.fpk";

    let first = compile_clean(
        &sandbox,
        "co Midcup Named",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Boots/Apple (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Zebra (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );
    assert!(!first.contains_key(k0646), "{:?}", first.keys());
    assert_eq!(boots_skl(&first, k0644), apple_skl, "Apple");
    assert_eq!(boots_skl(&first, k0645), body_skl("pes21"), "Zebra");

    write_shared_boots(
        &sandbox,
        "co Midcup Named",
        "Mango",
        Some(&mango_skl),
        &[11],
    );
    let second = compile_clean(
        &sandbox,
        "co Midcup Named",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Boots/Apple (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Mango (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Boots/Zebra (file=boots.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
    );
    assert_eq!(boots_skl(&second, k0644), apple_skl, "Apple");
    assert_eq!(boots_skl(&second, k0645), mango_skl, "Mango");
    assert_eq!(boots_skl(&second, k0646), body_skl("pes21"), "Zebra");
}

/// What `co Midcup Planned` reports at each compile.
const PLANNED_FINDINGS: &[&str] = &[
    "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)",
    "Info fmdl_weights_not_normalized [Keep] at Players/23 - B (file=glove_l.fmdl, count=2)",
    "Info export_identified [Keep] (team=/co/, id=714)",
    "Info team_colors_missing [Keep] ()",
];

// TC-PLN-01
#[test]
fn the_planned_ids_are_the_slot_s_and_two_compiles_write_the_same_bytes() {
    let sandbox = Sandbox::new("pln_planned_ids");
    sandbox.write(
        "exports/co Midcup Planned/Players/05 - A/boots.fmdl",
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        "exports/co Midcup Planned/Players/23 - B/glove_l.fmdl",
        &tracer_player_file("glove_l.fmdl"),
    );

    let first = compile_clean(&sandbox, "co Midcup Planned", PLANNED_FINDINGS);
    let first_cpk = fs::read(sandbox.root.join("output/4cc_99_test.cpk")).unwrap();
    let second = compile_clean(&sandbox, "co Midcup Planned", PLANNED_FINDINGS);
    let second_cpk = fs::read(sandbox.root.join("output/4cc_99_test.cpk")).unwrap();

    let paths: Vec<&str> = first.keys().map(String::as_str).collect();
    assert_eq!(
        paths,
        [
            "Asset/model/character/boots/k0625/#Win/boots.fpk",
            "Asset/model/character/boots/k0625/#Win/boots.fpkd",
            "Asset/model/character/face/real/71405/#Win/face.fpk",
            "Asset/model/character/face/real/71405/#Win/face.fpkd",
            "Asset/model/character/face/real/71423/#Win/face.fpk",
            "Asset/model/character/face/real/71423/#Win/face.fpkd",
            "Asset/model/character/glove/g0643/#Win/glove.fpk",
            "Asset/model/character/glove/g0643/#Win/glove.fpkd",
            "common/character0/model/character/uniform/team/UniColor.bin",
            "common/etc/TeamColor.bin",
        ]
    );
    assert_eq!(
        package_names(&first["Asset/model/character/glove/g0643/#Win/glove.fpk"]),
        ["glove_l.fmdl"]
    );
    assert_eq!(first, second);
    assert!(first_cpk == second_cpk, "the two CPKs differ");
}

// TC-CMN-05
#[test]
fn per_kit_models_on_pes_21_compile_the_lowest_variant_alone_and_say_so() {
    let sandbox = Sandbox::new("mod_kit_variant_models");
    let player = "exports/co Midcup Variants/Players/05 - A";
    // Different models: merged, the two would make a hair of more meshes than `pants_kit1`'s.
    sandbox.write(
        &format!("{player}/pants_kit1.fmdl"),
        &tracer_player_file("fcl_hair.fmdl"),
    );
    sandbox.write(
        &format!("{player}/pants_kit2.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    // The deep pass does not read the left-out variant, which nothing reads; it is not a part
    // of the hair, so only `pants_kit1` is the hair's fallback.
    let checked = [
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=pants_kit1.fmdl, count=1662)",
        "Info export_identified [Keep] (team=/co/, id=714)",
        "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=pants_kit1.fmdl)",
    ];

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co Midcup Variants"),
        checked
    );
    assert_eq!(check.exit_code(), 0);

    let both = compile_clean(
        &sandbox,
        "co Midcup Variants",
        &[
            &checked[..],
            &[
                "Info team_colors_missing [Keep] ()",
                "Warning kit_variant_model_left_out [Keep] at Players/05 - A (model=pants_kitN.fmdl, used=pants_kit1.fmdl)",
            ],
        ]
        .concat(),
    );
    fs::remove_file(sandbox.root.join(format!("{player}/pants_kit2.fmdl"))).unwrap();
    let alone = compile_clean(
        &sandbox,
        "co Midcup Variants",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=pants_kit1.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=pants_kit1.fmdl)",
            "Info team_colors_missing [Keep] ()",
        ],
    );

    let hair = |entries: &BTreeMap<String, Vec<u8>>| {
        face_package(entries).get("fcl_hair.fmdl").unwrap().to_vec()
    };
    assert!(hair(&both) == hair(&alone), "the packed hair differs");
}

#[test]
fn per_kit_model_files_on_pes_21_are_a_set_and_only_the_lowest_is_converted() {
    let sandbox = Sandbox::new("mod_kit_variant_pre_fox_models");
    let player = "exports/co Midcup Variant Cards/Players/05 - A";
    for variant in ["pants_kit1", "pants_kit2"] {
        sandbox.write(&format!("{player}/{variant}.model"), &card_model());
        sandbox.write(&format!("{player}/{variant}.mtl"), &card_materials());
    }
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());

    let entries = compile_clean(
        &sandbox,
        "co Midcup Variant Cards",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=pants_kit1.model)",
            "Info team_colors_missing [Keep] ()",
            "Warning kit_variant_model_left_out [Keep] at Players/05 - A (model=pants_kitN.model, used=pants_kit1.model)",
        ],
    );

    // The card's one mesh and the anti-blur mesh the FMDL export makes for it: one variant's,
    // `pants_kit2.model` being read by no task.
    let hair = face_package(&entries);
    let file = FmdlFile::read(hair.get("fcl_hair.fmdl").unwrap()).unwrap();
    assert_eq!(Model::from_file(&file).unwrap().meshes.len(), 2);
}

#[test]
fn per_kit_boots_on_pes_21_are_boots_read_without_their_kit_token() {
    let sandbox = Sandbox::new("mod_kit_variant_boots");
    let player = "exports/co Midcup Variant Boots/Players/05 - A";
    sandbox.write(&format!("{player}/boots_kit1.fmdl"), &clean_model());
    sandbox.write(&format!("{player}/boots_kit2.fmdl"), &clean_model());

    let entries = compile_clean(
        &sandbox,
        "co Midcup Variant Boots",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Warning kit_variant_model_left_out [Keep] at Players/05 - A (model=boots_kitN.fmdl, used=boots_kit1.fmdl)",
        ],
    );

    let boots = "Asset/model/character/boots/k0625/#Win/boots.fpk";
    assert!(entries.contains_key(boots), "no boots package");
    assert!(
        face_package(&entries).get("fcl_hair.fmdl").is_none(),
        "a variant was merged into the face's hair"
    );
}

// TC-MOD-62
#[test]
fn a_left_out_kit_variant_that_does_not_parse_is_not_read_on_pes_21() {
    let sandbox = Sandbox::new("mod_kit_variant_broken");
    let player = "exports/co Midcup Variant Boots/Players/05 - A";
    sandbox.write(&format!("{player}/face.fmdl"), &clean_model());
    let boots = tracer_player_file("boots.fmdl");
    sandbox.write(&format!("{player}/boots_kit1.fmdl"), &boots);
    // Four bytes no FMDL reader accepts.
    sandbox.write(&format!("{player}/boots_kit2.fmdl"), b"junk");

    let entries = compile_clean(
        &sandbox,
        "co Midcup Variant Boots",
        &[
            "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots_kit1.fmdl, count=1662)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=face.fmdl)",
            "Info team_colors_missing [Keep] ()",
            "Warning kit_variant_model_left_out [Keep] at Players/05 - A (model=boots_kitN.fmdl, used=boots_kit1.fmdl)",
        ],
    );

    let package =
        fpk::FpkFile::read(&entries["Asset/model/character/boots/k0625/#Win/boots.fpk"]).unwrap();
    let meshes = |bytes: &[u8]| {
        Model::from_file(&FmdlFile::read(bytes).unwrap())
            .unwrap()
            .meshes
            .len()
    };
    assert_eq!(meshes(package.get("boots.fmdl").unwrap()), meshes(&boots));
}

#[test]
fn per_kit_boots_in_a_shared_boots_folder_are_named_as_boots() {
    let sandbox = Sandbox::new("mod_kit_variant_shared_boots");
    let export = "exports/co Midcup Variant Crocs";
    sandbox.write(&format!("{export}/Players/05 - A/Crocs.boots"), b"");
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots_kit1.fmdl"),
        &clean_model(),
    );
    sandbox.write(
        &format!("{export}/Boots/Crocs/boots_kit2.fmdl"),
        &clean_model(),
    );

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    assert_eq!(
        findings_of(&check.messages(), "co Midcup Variant Crocs"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(check.exit_code(), 0);

    // Compiled, the shared folder's set collapses to its lowest variant, as a player's does.
    let entries = compile_clean(
        &sandbox,
        "co Midcup Variant Crocs",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Warning kit_variant_model_left_out [Keep] at Boots/Crocs (model=boots_kitN.fmdl, used=boots_kit1.fmdl)",
        ],
    );
    let variant_meshes = Model::from_file(&FmdlFile::read(&clean_model()).unwrap())
        .unwrap()
        .meshes
        .len();
    assert_eq!(
        boots_mesh_count(&entries, "Asset/model/character/boots/k0644/#Win/boots.fpk"),
        variant_meshes,
        "kit 1's meshes alone"
    );
}

// TC-MOD-56
#[test]
fn a_shared_boots_folder_s_per_kit_models_on_pes_17_collapse_to_the_lowest_variant() {
    let sandbox = Sandbox::new("mod_kit_variant_shared_boots_pes17");
    let export = "co Midcup Variant Studs";
    sandbox.write(&format!("exports/{export}/Players/05 - A/Studs.boots"), b"");
    let studs = format!("exports/{export}/Boots/Studs");
    for variant in ["boots_kit1", "boots_kit2"] {
        sandbox.write(&format!("{studs}/{variant}.model"), &card_model());
    }
    sandbox.write(&format!("{studs}/boots.mtl"), &materials_naming("studs"));
    sandbox.write(&format!("{studs}/studs.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            CLEAN[0],
            CLEAN[1],
            "Warning kit_variant_model_left_out [Keep] at Boots/Studs (model=boots_kitN.model, used=boots_kit1.model)",
        ],
    );

    // The one `boots.model` every linking player wears is kit 1's alone, not the two merged.
    let boots = entries_under(&entries, BOOTS_K0644);
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(names, ["boots.model", "boots.mtl", "studs.dds"]);
    let mesh_count = |bytes: &[u8]| {
        let file = pes_model::format::PreFoxModel::read(bytes).unwrap();
        pes_model::model::Model::from_file(&file)
            .unwrap()
            .meshes
            .len()
    };
    assert_eq!(
        mesh_count(boots["boots.model"]),
        mesh_count(&card_model()),
        "kit 1's meshes alone"
    );
    assert!(
        *boots["boots.model"] == card_model(),
        "kit 1's model as it is"
    );
}

/// The number of faces of the model `name` in the package at `path` in `entries`, read back
/// with `fmdl`.
fn face_count(entries: &BTreeMap<String, Vec<u8>>, path: &str, name: &str) -> usize {
    let package = fpk::FpkFile::read(&entries[path]).unwrap();
    let model = Model::from_file(&FmdlFile::read(package.get(name).unwrap()).unwrap()).unwrap();
    model.meshes.iter().map(|mesh| mesh.faces.len()).sum()
}

// TC-MOD-31
#[test]
fn a_face_model_weighted_to_the_hand_bones_gives_its_hands_to_the_player_s_gloves() {
    let sandbox = Sandbox::new("mod_hand_split");
    install_tables(
        &sandbox,
        &[(BOOTS_LIST, &item_list(&[])), (GLOVE_LIST, &item_list(&[]))],
    );
    // A full-body model with both hands on (`tests/fixtures/hand_split/README.md`): 40
    // faces, of which each hand's split takes 8 and the body keeps 24. Slot 05 holds it as
    // a face model, slot 06 the same bytes as boots, which are never split.
    let body =
        fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hand_split/body.fmdl"))
            .unwrap();
    let export = "exports/co Midcup Hands";
    sandbox.write(&format!("{export}/Players/05 - A/body.fmdl"), &body);
    sandbox.write(&format!("{export}/Players/06 - B/boots.fmdl"), &body);

    let entries = compile_clean(
        &sandbox,
        "co Midcup Hands",
        &[
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=body.fmdl)",
            "Info team_colors_missing [Keep] ()",
            "Info model_hand_split [Keep] at Players/05 - A (model=body.fmdl, gloves=glove_l, glove_r)",
        ],
    );

    let gloves = "Asset/model/character/glove/g0625/#Win/glove.fpk";
    assert_eq!(
        package_names(&entries[gloves]),
        ["glove_l.fmdl", "glove_r.fmdl"]
    );
    let face = "Asset/model/character/face/real/71405/#Win/face.fpk";
    let split = [
        face_count(&entries, gloves, "glove_l.fmdl"),
        face_count(&entries, gloves, "glove_r.fmdl"),
        face_count(&entries, face, "fcl_hair.fmdl"),
    ];
    assert_eq!(split, [8, 8, 24]);
    assert_eq!(
        split.iter().sum::<usize>(),
        40,
        "every face of the source, once"
    );
    assert_eq!(pairs_of(&entries, GLOVE_LIST), [(71405, 625)]);

    // Slot 06's boots keep the whole model, and give no gloves.
    let boots = "Asset/model/character/boots/k0626/#Win/boots.fpk";
    assert_eq!(face_count(&entries, boots, "boots.fmdl"), 40);
    assert!(
        !entries
            .keys()
            .any(|path| path.starts_with("Asset/model/character/glove/g0626/")),
        "no gloves for slot 06"
    );
    assert_eq!(
        pairs_of(&entries, BOOTS_LIST),
        [(71406, 626)],
        "slot 06's boots"
    );
}
