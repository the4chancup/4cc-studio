//! `compile` for PES 2017 of the two cases that merge `.model` files: a player folder holding
//! `ingame_face`, whose models other than gloves become his own boots, merged with a boots
//! folder his link combines, and a shared `Boots/` folder holding several boots models.

use pes_model::format::PreFoxModel;
use pes_model::model::Model;

use crate::common::Sandbox;
use crate::compile::cpk_entries;
use crate::findings_of;
use crate::prefox_faces::{
    BOOTS_K0644, CLEAN, card_materials, card_model, compile_pes17, entries_under, face_cpk, pes17,
    pre_fox_fixture, sampler_paths, small_dds, write_slot_05_face,
};

/// The folder slot 05's own boots are written to in team 714's export: his exclusive id.
const BOOTS_K0625: &str = "common/character0/model/character/boots/k0625/";

/// The card-head template's face model, a clean pre-Fox model with one material, `card`.
fn card() -> Model {
    Model::from_file(&PreFoxModel::read(&card_model()).unwrap()).unwrap()
}

/// `model` written as a `.model` file.
fn written(model: &Model) -> Vec<u8> {
    model.to_file().unwrap().write().unwrap()
}

/// The card-head model with its one material, `card`, renamed `material`.
fn card_naming(material: &str) -> Vec<u8> {
    let mut model = card();
    assert_eq!(model.materials, ["card"]);
    model.materials[0] = material.to_owned();
    written(&model)
}

/// The card-head template's material set, its one material `card` renamed `material` and its
/// one texture path `./texture.dds` renamed `./<stem>.dds`.
fn materials(material: &str, stem: &str) -> Vec<u8> {
    let text = String::from_utf8(pre_fox_fixture("cardhead_materials.mtl")).unwrap();
    assert!(text.contains("name=\"card\""), "{text}");
    assert!(text.contains("./texture.dds"), "{text}");
    text.replace("name=\"card\"", &format!("name=\"{material}\""))
        .replace("./texture.dds", &format!("./{stem}.dds"))
        .into_bytes()
}

/// The number of meshes of the `.model` `bytes`.
fn mesh_count(bytes: &[u8]) -> usize {
    Model::from_file(&PreFoxModel::read(bytes).unwrap())
        .unwrap()
        .meshes
        .len()
}

// TC-MOD-35
#[test]
fn under_ingame_face_a_boots_link_beside_a_boots_model_merges_into_the_player_s_own_boots() {
    let sandbox = Sandbox::new("prefox_ingame_link");
    let export = "co Midcup Linked";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/ingame_face"), b"");
    sandbox.write(&format!("{player}/kit_boots.model"), &card_model());
    sandbox.write(&format!("{player}/kit_boots.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    sandbox.write(&format!("{player}/Crocs.boots"), b"");
    let crocs = format!("exports/{export}/Boots/Crocs");
    let crocs_model = card_naming("crocs");
    sandbox.write(&format!("{crocs}/boots.model"), &crocs_model);
    sandbox.write(&format!("{crocs}/boots.mtl"), &materials("crocs", "crocs"));
    sandbox.write(&format!("{crocs}/crocs.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            CLEAN[0],
            CLEAN[1],
            "Info link_combined [Keep] at Players/05 - A (link=Crocs.boots)",
            "Info model_merged [Keep] at Players/05 - A (model=boots.model)",
        ],
    );

    assert!(!entries.contains_key(&face_cpk(5)), "{:?}", entries.keys());
    // Slot 05's exclusive folder alone: Crocs, linked by no other player, takes no id.
    let boots = entries_under(&entries, "common/character0/model/character/boots/");
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(names, ["k0625/boots.model", "k0625/boots.mtl"]);
    assert_eq!(
        mesh_count(boots["k0625/boots.model"]),
        mesh_count(&crocs_model) + mesh_count(&card_model()),
        "Crocs's meshes plus the local model's"
    );
    // Crocs's part first (`boots.model` before `kit_boots.model`), each material pointed at
    // its texture in the player's common folder, which the CPK holds.
    let home = "model/character/uniform/common/714/05 - A/";
    assert_eq!(
        sampler_paths(boots["k0625/boots.mtl"]),
        [format!("{home}crocs.dds"), format!("{home}skin.dds")]
    );
    for stem in ["crocs", "skin"] {
        let texture = format!("common/character1/{home}{stem}.dds");
        assert!(entries.contains_key(&texture), "{texture}");
    }
}

/// Writes slot 07 of the export `export`: `face_high.model`, its `.mtl` and `skin.dds`, a
/// clean player beside slot 05, so the CPK is written whatever slot 05 gives.
fn write_slot_07_face(sandbox: &Sandbox, export: &str) {
    let player = format!("exports/{export}/Players/07 - B");
    sandbox.write(&format!("{player}/face_high.model"), &card_model());
    sandbox.write(&format!("{player}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
}

/// Runs `compile --no-deploy` for PES 17 over `sandbox`, asserts the export `name`'s findings
/// are `CLEAN` then `failure`, that the run failed, and that the CPK holds slot 07's face and
/// no `boots/k0625/` entry.
fn assert_boots_left_out(sandbox: &Sandbox, name: &str, failure: &str) {
    let run = sandbox.run(&pes17(sandbox), &["compile", "--no-deploy"]);
    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, name),
        [CLEAN[0], CLEAN[1], failure],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(entries.contains_key(&face_cpk(7)), "{:?}", entries.keys());
    assert!(
        !entries.keys().any(|path| path.starts_with(BOOTS_K0625)),
        "{:?}",
        entries.keys()
    );
}

// TC-MOD-41
#[test]
fn under_ingame_face_boots_parts_defining_one_material_differently_leave_the_boots_out() {
    let sandbox = Sandbox::new("prefox_ingame_material");
    let export = "co Midcup Skin";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/ingame_face"), b"");
    for part in ["shirt", "socks"] {
        sandbox.write(&format!("{player}/{part}.model"), &card_naming("skin"));
        sandbox.write(&format!("{player}/{part}.mtl"), &materials("skin", part));
        sandbox.write(&format!("{player}/{part}.dds"), &small_dds());
    }
    write_slot_07_face(&sandbox, export);

    assert_boots_left_out(
        &sandbox,
        export,
        "Error merge_material_conflict [DropFolder] at Players/05 - A (material=skin)",
    );
}

// TC-MOD-41
#[test]
fn under_ingame_face_boots_parts_naming_one_bone_with_different_transforms_leave_the_boots_out() {
    let sandbox = Sandbox::new("prefox_ingame_bone");
    let export = "co Midcup Bones";
    let player = format!("exports/{export}/Players/05 - A");
    sandbox.write(&format!("{player}/ingame_face"), b"");
    let mut moved = card();
    moved.bones[0].matrix[3] += 1.0;
    sandbox.write(&format!("{player}/shirt.model"), &card_model());
    sandbox.write(&format!("{player}/socks.model"), &written(&moved));
    // One material set both parts use.
    sandbox.write(&format!("{player}/materials.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
    write_slot_07_face(&sandbox, export);

    assert_boots_left_out(
        &sandbox,
        export,
        &format!(
            "Error skl_merge_conflict [DropFolder] at Players/05 - A (bone={})",
            moved.bones[0].name
        ),
    );
}

#[test]
fn a_shared_boots_folder_holding_two_boots_models_is_written_as_one_merged_boots_model() {
    let sandbox = Sandbox::new("prefox_shared_boots_merge");
    let export = "co Midcup Twin";
    write_slot_05_face(&sandbox, export);
    sandbox.write(&format!("exports/{export}/Players/05 - A/Twin.boots"), b"");
    let twin = format!("exports/{export}/Boots/Twin");
    let studs_model = card_naming("studs");
    sandbox.write(&format!("{twin}/a_boots.model"), &card_model());
    sandbox.write(&format!("{twin}/a_boots.mtl"), &materials("card", "left"));
    sandbox.write(&format!("{twin}/left.dds"), &small_dds());
    sandbox.write(&format!("{twin}/b_boots.model"), &studs_model);
    sandbox.write(&format!("{twin}/b_boots.mtl"), &materials("studs", "right"));
    sandbox.write(&format!("{twin}/right.dds"), &small_dds());

    let entries = compile_pes17(
        &sandbox,
        export,
        &[
            CLEAN[0],
            CLEAN[1],
            "Info model_merged [Keep] at Boots/Twin (model=boots.model)",
        ],
    );

    let boots = entries_under(&entries, BOOTS_K0644);
    let names: Vec<&str> = boots.keys().copied().collect();
    assert_eq!(names, ["boots.model", "boots.mtl", "left.dds", "right.dds"]);
    assert_eq!(
        mesh_count(boots["boots.model"]),
        mesh_count(&card_model()) + mesh_count(&studs_model)
    );
    assert_eq!(
        sampler_paths(boots["boots.mtl"]),
        ["./left.dds", "./right.dds"]
    );
}
