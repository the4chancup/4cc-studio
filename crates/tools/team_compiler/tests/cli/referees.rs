//! `compile` over a refs export on Fox (`team_compiler/blue_port.md` "Referee export
//! processing"): each referee folder prepared once and emitted under every slot `players.txt`
//! maps it to, as `face/real/referee0NN`, `k99NN` and `g99NN`, its textures once in team 999's
//! common subfolder of its name, and a link of his made a part of his slots' own packages; in a
//! normal compile all of it goes into the refs CPK, `refs_cpk_name`, beside the team side
//! (`team_compiler/pipeline.md` "5. Writer", step 5).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use kit_config::KitConfig;
use pes_model::format::mtl::{MaterialEntry, MaterialSet};
use pes_version::{Engine, PesVersion};

use crate::common::{Run, Sandbox};
use crate::common_links::texture_directories;
use crate::compile::{
    cpk_entries, kit_texture, pes_settings, pes21_settings, tracer_kit, tracer_player_file,
};
use crate::compile_exports::TEAM_COLOR;
use crate::deploy::{install_pes, templates_folder};
use crate::models::package_names;
use crate::prefox_faces::{nested_entries, ordered_entries};
use crate::sideload::slashed;
use crate::textures::{texture_fixture, tracer_model_renaming};
use crate::{clean_model, findings_of, snapshot};

/// The refs export's folder in the sandbox.
pub(crate) const REFS: &str = "exports/refs Cup";

/// Where `Ref A`'s textures go: team 999's common subfolder of his folder's name.
const REF_A_TEXTURES: &str = "Asset/model/character/common/999/Ref A/sourceimages/#windx11";

/// The CPK path of referee slot `slot`'s package `stem` (`face`, `boots`, `glove`) of `kind`
/// (`face/real/referee0`, `boots/k99`, `glove/g99`).
fn referee_package(kind: &str, slot: &str, stem: &str) -> String {
    format!("Asset/model/character/{kind}{slot}/#Win/{stem}.fpk")
}

/// Writes `Ref A` mapped to each of `slots`: a face model and a boots model, each naming
/// `skin.dds`, and `skin.dds` (TC-REF-01's referee).
pub(crate) fn write_ref_a(sandbox: &Sandbox, slots: &[&str]) {
    let roster: String = slots.iter().map(|slot| format!("{slot} Ref A\n")).collect();
    sandbox.write(&format!("{REFS}/players.txt"), roster.as_bytes());
    let folder = format!("{REFS}/Players/Ref A");
    sandbox.write(
        &format!("{folder}/face_high.fmdl"),
        &tracer_model_renaming("fcl_hair.fmdl", &[("shirt.dds", "skin.dds")]),
    );
    sandbox.write(
        &format!("{folder}/boots.fmdl"),
        &tracer_model_renaming("boots.fmdl", &[("shirt.dds", "skin.dds")]),
    );
    sandbox.write(
        &format!("{folder}/skin.dds"),
        &tracer_player_file("shirt.dds"),
    );
}

/// The refs CPK's file name at the default `refs_cpk_name`.
pub(crate) const REFS_CPK: &str = "4cc_18_referees.cpk";

/// `compile --no-deploy` in `sandbox` for PES 21: the run and its refs CPK's entries.
fn compile(sandbox: &Sandbox) -> (Run, BTreeMap<String, Vec<u8>>) {
    compile_for(sandbox, PesVersion::Pes21)
}

/// `compile` for `version`.
fn compile_for(sandbox: &Sandbox, version: PesVersion) -> (Run, BTreeMap<String, Vec<u8>>) {
    let settings = pes_settings(sandbox, version.number().try_into().unwrap());
    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);
    let entries = cpk_entries(&sandbox.root.join("output").join(REFS_CPK));
    (run, entries)
}

/// The line `deploy_skipped_by_flag` reports for `output/<name>`.
fn skipped(sandbox: &Sandbox, name: &str) -> String {
    format!(
        "Info deploy_skipped_by_flag [Keep] (path={})",
        sandbox.display(&format!("output/{name}"))
    )
}

/// The game path of the referee template tree's `RefereeAppearance.bin`.
const REFEREE_APPEARANCE: &str =
    "common/character0/model/character/appearance/RefereeAppearance.bin";

/// The folder of `engine`'s referee template tree, below `resources/templates/` and below a
/// data directory's `templates/`.
fn tree_folder(engine: Engine) -> &'static str {
    match engine {
        Engine::Fox => "referees_fox",
        Engine::PreFox => "referees_prefox",
    }
}

/// The referee template tree of `engine` as the repository holds it, `resources/templates/
/// referees_fox/` (31 files) or `referees_prefox/` (51): every file by its path below that
/// folder, spelled with `/`, with its bytes.
fn referee_tree(engine: Engine) -> BTreeMap<String, Vec<u8>> {
    let folder = tree_folder(engine);
    let count = match engine {
        Engine::Fox => 31,
        Engine::PreFox => 51,
    };
    let tree: BTreeMap<String, Vec<u8>> = snapshot(&templates_folder().join(folder))
        .into_iter()
        .map(|(path, bytes)| (slashed(&path), bytes))
        .collect();
    assert_eq!(tree.len(), count, "the {folder} tree's files");
    tree
}

/// Asserts that `entries` hold every file of `tree`, a referee template tree, with its bytes.
fn assert_tree_in(entries: &BTreeMap<String, Vec<u8>>, tree: &BTreeMap<String, Vec<u8>>) {
    for (path, bytes) in tree {
        assert!(entries.get(path) == Some(bytes), "{path}");
    }
}

/// The game path of the referees' marker model: stock collar 77's `nocloth` model.
const MARKER_COLLAR: &str = "Asset/model/character/uniform/nocloth/#Win/collar_077.fmdl";

/// The game path of the converted marker texture, in the referees' Common output.
const MARKER_TEXTURE: &str =
    "Asset/model/character/common/999/sourceimages/#windx11/ref_marker.ftex";

/// The game path of the pre-Fox referees' marker model: stock collar 77's `referee_collar`
/// model, the one a pre-Fox referee draws.
const PRE_FOX_MARKER_MODEL: &str =
    "common/character0/model/character/uniform/nocloth/referee_collar_077.model";

/// The game path of the pre-Fox marker model's material set, beside it.
const PRE_FOX_MARKER_MTL: &str =
    "common/character0/model/character/uniform/nocloth/referee_collar_077.mtl";

/// The game path of stock collar 77's pre-Fox model, which must exist for the referee to
/// draw his `referee_collar_077`.
const PRE_FOX_EMPTY_COLLAR: &str =
    "common/character0/model/character/uniform/nocloth/collar_077.model";

/// The game path of the converted pre-Fox marker texture, in the referees' Common output.
const PRE_FOX_MARKER_TEXTURE: &str =
    "common/character1/model/character/uniform/common/999/ref_marker.dds";

/// The game path of the pre-Fox template tree's prop model, the marker's model.
const REFEREE_PROP_MODEL: &str =
    "common/character1/model/character/parts/referee/referee_prop.model";

/// The game path of the pre-Fox template tree's prop `.mtl`, beside its model.
const REFEREE_PROP_MTL: &str = "common/character1/model/character/parts/referee/referee_prop.mtl";

/// The folder of the referee kit configs, in the template tree and in the refs CPK.
const REFEREE_CONFIGS: &str = "common/character0/model/character/uniform/team/referee/";

/// The referee kit configs of `engine`'s template tree as the repository holds it, by game
/// path.
fn template_configs(engine: Engine) -> BTreeMap<String, Vec<u8>> {
    let configs: BTreeMap<String, Vec<u8>> = referee_tree(engine)
        .into_iter()
        .filter(|(path, _)| path.starts_with(REFEREE_CONFIGS))
        .collect();
    assert_eq!(configs.len(), 20, "the tree's kit configs");
    configs
}

/// Writes `Ref A` in slot 01 (`write_ref_a`) and, when given, `marker` as the refs export's
/// `ref_marker.dds`.
fn write_refs_with_marker(sandbox: &Sandbox, marker: Option<&[u8]>) {
    write_ref_a(sandbox, &["01"]);
    if let Some(marker) = marker {
        sandbox.write(&format!("{REFS}/ref_marker.dds"), marker);
    }
}

/// The names of the `.cpk` files directly in `folder`, sorted.
fn cpk_files(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".cpk"))
        .collect();
    names.sort();
    names
}

// TC-REF-01
#[test]
fn a_referee_folder_is_emitted_under_each_of_his_slots_with_his_textures_once() {
    let sandbox = Sandbox::new("ref_slots");
    write_ref_a(&sandbox, &["01", "20", "35"]);

    let (run, entries) = compile(&sandbox);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "refs Cup"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/Ref A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/Ref A (file=face_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=referees)",
        ],
        "no team_colors_missing: a referee has no team record"
    );
    assert_eq!(lines.last(), Some(&skipped(&sandbox, REFS_CPK)));
    assert_eq!(run.exit_code(), 0);
    // The refs export alone commits: no team CPK, and no bin, since the referees change none.
    assert_eq!(cpk_files(&sandbox.root.join("output")), [REFS_CPK]);
    assert!(!entries.contains_key(TEAM_COLOR));
    for slot in ["01", "20", "35"] {
        for (kind, stem) in [("face/real/referee0", "face"), ("boots/k99", "boots")] {
            let package = referee_package(kind, slot, stem);
            assert!(entries.contains_key(&package), "{package}");
            assert!(entries.contains_key(&format!("{package}d")), "{package}d");
        }
    }
    let skins: Vec<&String> = entries
        .keys()
        .filter(|path| path.ends_with("/skin.ftex"))
        .collect();
    assert_eq!(skins, [&format!("{REF_A_TEXTURES}/skin.ftex")]);
    let face =
        fpk::FpkFile::read(&entries[&referee_package("face/real/referee0", "01", "face")]).unwrap();
    let directories = texture_directories(face.get("face_high.fmdl").unwrap(), "skin.dds");
    assert!(!directories.is_empty());
    assert!(
        directories
            .iter()
            .all(|directory| directory
                == "/Assets/pes16/model/character/common/999/Ref A/sourceimages/"),
        "{directories:?}"
    );
    // No note is collected without one.
    assert!(!sandbox.root.join("output/teamnotes.txt").exists());
    // The referee kits and appearance the game needs come with them.
    assert_tree_in(&entries, &referee_tree(Engine::Fox));
}

/// The folder of referee slot `slot`'s pre-Fox face CPK: its CPK is `<folder>.cpk`, and every
/// entry of it sits in `<folder>/`.
fn pre_fox_referee_face(slot: &str) -> String {
    format!("common/character0/model/character/face/real/referee0{slot}")
}

// TC-REF-09
#[test]
fn on_pes_17_a_referee_folder_is_a_face_cpk_per_slot_his_boots_in_its_face_xml() {
    let sandbox = Sandbox::new("ref_slots_pes17");
    write_ref_a(&sandbox, &["01", "20", "35"]);

    let (run, entries) = compile_for(&sandbox, PesVersion::Pes17);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    for slot in ["01", "20", "35"] {
        let folder = pre_fox_referee_face(slot);
        let face = nested_entries(&entries[&format!("{folder}.cpk")]);
        let names: Vec<&str> = face
            .keys()
            .map(|path| path.strip_prefix(&format!("{folder}/")).unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "boots.mtl",
                "face.xml",
                "face_high.mtl",
                "oral_boots_win32.model",
                "oral_face_high_win32.model",
            ],
            "{slot}"
        );
        // His boots ride in the face, typed `parts` as a team player's own boots are.
        let entry = |xml_type: &str, path: &str, material: &str| {
            (xml_type.to_owned(), path.to_owned(), material.to_owned())
        };
        assert_eq!(
            ordered_entries(&face[&format!("{folder}/face.xml")]),
            [
                entry("parts", "./oral_boots_*.model", "./boots.mtl"),
                entry("face_neck", "./oral_face_high_*.model", "./face_high.mtl"),
            ],
            "{slot}"
        );
    }
    // No boots folder of his: the only boots files are the template tree's (`k0062`, the
    // referees' stock boots).
    let boots: Vec<&str> = entries
        .keys()
        .filter_map(|path| path.strip_prefix("common/character0/model/character/boots/"))
        .collect();
    assert_eq!(boots, ["k0062/boots.model", "k0062/boots.mtl"]);
    let skins: Vec<&String> = entries
        .keys()
        .filter(|path| path.ends_with("/skin.dds"))
        .collect();
    assert_eq!(
        skins,
        ["common/character1/model/character/uniform/common/999/Ref A/skin.dds"]
    );
    assert_tree_in(&entries, &referee_tree(Engine::PreFox));
}

#[test]
fn a_refs_export_whose_only_folder_validation_drops_writes_no_refs_cpk_and_no_tree() {
    let sandbox = Sandbox::new("ref_folder_dropped");
    write_ref_a(&sandbox, &["01"]);
    sandbox.write(&format!("{REFS}/Players/Ref A/readme.txt"), b"notes");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert!(
        lines.contains(
            &"refs Cup: Error file_type_disallowed [DropFolder] at Players/Ref A (file=readme.txt)"
                .to_owned()
        ),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
    // Nothing of the refs export went in, so neither the refs CPK nor its tree is written,
    // and the installed referees would stay.
    assert!(!sandbox.root.join("output").join(REFS_CPK).exists());
    assert_eq!(
        cpk_files(&sandbox.root.join("output")),
        Vec::<String>::new()
    );
}

/// Compiles Ref A for `version` in the sandbox `name` with a data directory file at the path of
/// `version`'s template tree's `RefereeAppearance.bin`, and asserts that the file replaces that
/// file of the refs CPK, reported first, the rest of the tree being the built-in one.
fn assert_tree_override_replaces_its_file(name: &str, version: PesVersion) {
    let sandbox = Sandbox::new(name);
    write_ref_a(&sandbox, &["01"]);
    let appearance = b"the cup's RefereeAppearance.bin";
    let folder = tree_folder(version.engine());
    let override_path = format!("data/templates/{folder}/{REFEREE_APPEARANCE}");
    sandbox.write(&override_path, appearance);

    let (run, entries) = compile_for(&sandbox, version);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        run.messages().first(),
        Some(&format!(
            "Info template_override_active [Keep] (path={})",
            sandbox.display(&override_path)
        ))
    );
    assert_eq!(entries[REFEREE_APPEARANCE], appearance);
    let mut tree = referee_tree(version.engine());
    tree.insert(REFEREE_APPEARANCE.to_owned(), appearance.to_vec());
    assert_tree_in(&entries, &tree);
}

#[test]
fn a_data_directory_file_at_a_tree_path_replaces_that_file_of_the_refs_cpk() {
    assert_tree_override_replaces_its_file("ref_tree_override", PesVersion::Pes21);
}

#[test]
fn on_pes_17_a_data_directory_file_at_a_pre_fox_tree_path_replaces_that_file() {
    assert_tree_override_replaces_its_file("ref_tree_override_pes17", PesVersion::Pes17);
}

// TC-REF-06
#[test]
fn a_ref_marker_goes_into_the_refs_cpk_as_collar_77_which_every_referee_kit_wears() {
    let sandbox = Sandbox::new("ref_marker");
    install_pes(&sandbox);
    // A data CPK of the game's: the marker is shown without writing anything outside the
    // refs CPK.
    let data_cpk = fs::read(templates_folder().join("placeholder.cpk")).unwrap();
    sandbox.write("PES/Data/dt00_x64.cpk", &data_cpk);
    write_refs_with_marker(&sandbox, Some(&tracer_player_file("shirt.dds")));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let entries = cpk_entries(&sandbox.root.join("PES/download").join(REFS_CPK));
    assert!(entries.contains_key(MARKER_TEXTURE), "the marker texture");
    let collar = &entries[MARKER_COLLAR];
    assert_eq!(
        texture_directories(collar, "ref_marker.dds"),
        ["/Assets/pes16/model/character/common/999/sourceimages/"]
    );
    assert_eq!(
        texture_directories(collar, "cup_logo.dds"),
        Vec::<String>::new()
    );
    for (path, template) in template_configs(Engine::Fox) {
        let config = KitConfig::decode(&entries[&path], PesVersion::Pes21).unwrap();
        assert_eq!(
            (config.shirt.collar, config.shirt.winter_collar),
            (77, 77),
            "{path}"
        );
        assert_ne!(entries[&path], template, "{path}");
    }
    assert!(
        fs::read(sandbox.root.join("PES/Data/dt00_x64.cpk")).unwrap() == data_cpk,
        "dt00_x64.cpk untouched"
    );
}

/// Compiles Ref A without `ref_marker.dds` for `version` in the sandbox `name`, and asserts
/// that the refs CPK holds none of `collars`, the marker's model files, no marker texture, and
/// the kit configs of `version`'s template tree as they are.
fn assert_compiles_without_marker(name: &str, version: PesVersion, collars: &[&str]) {
    let sandbox = Sandbox::new(name);
    write_refs_with_marker(&sandbox, None);

    let (run, entries) = compile_for(&sandbox, version);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    for collar in collars {
        assert!(!entries.contains_key(*collar), "no {collar}");
    }
    for path in entries.keys() {
        assert!(!path.contains("ref_marker"), "{path}");
    }
    for (path, template) in template_configs(version.engine()) {
        assert!(entries[&path] == template, "{path}");
    }
}

// TC-REF-08
#[test]
fn without_a_ref_marker_the_refs_cpk_holds_no_collar_and_the_template_kit_configs() {
    assert_compiles_without_marker("ref_no_marker", PesVersion::Pes21, &[MARKER_COLLAR]);
}

#[test]
fn without_a_ref_marker_a_pes_17_refs_cpk_holds_no_collar_pair_and_the_template_kit_configs() {
    assert_compiles_without_marker(
        "ref_no_marker_pes17",
        PesVersion::Pes17,
        &[
            PRE_FOX_MARKER_MODEL,
            PRE_FOX_MARKER_MTL,
            PRE_FOX_EMPTY_COLLAR,
        ],
    );
}

// TC-REF-04
#[test]
fn on_pes_17_a_ref_marker_goes_in_as_referee_collar_77_beside_an_empty_collar_77() {
    let sandbox = Sandbox::new("ref_marker_pes17");
    write_refs_with_marker(&sandbox, Some(&tracer_player_file("shirt.dds")));

    let (run, entries) = compile_for(&sandbox, PesVersion::Pes17);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let tree = referee_tree(Engine::PreFox);
    assert!(
        entries.get(PRE_FOX_MARKER_MODEL) == Some(&tree[REFEREE_PROP_MODEL]),
        "the template's prop model as it is"
    );
    // The template's `judge_incom` alone, its diffuse map naming the marker texture.
    let template = MaterialSet::read(&tree[REFEREE_PROP_MTL]).unwrap();
    let mut expected = template
        .materials
        .into_iter()
        .find(|material| material.name == "judge_incom")
        .unwrap();
    for entry in &mut expected.entries {
        if let MaterialEntry::Sampler(sampler) = entry {
            assert_eq!(sampler.name, "DiffuseMap");
            assert_eq!(sampler.path, "./incom_bsm.dds");
            "model/character/uniform/common/999/ref_marker.dds".clone_into(&mut sampler.path);
        }
    }
    let written = MaterialSet::read(&entries[PRE_FOX_MARKER_MTL]).unwrap();
    assert_eq!(written.materials, [expected]);
    let empty_collar = fs::read(templates_folder().join("collar_empty.model")).unwrap();
    assert!(
        entries.get(PRE_FOX_EMPTY_COLLAR) == Some(&empty_collar),
        "the bundled empty collar"
    );
    assert!(
        entries[PRE_FOX_MARKER_TEXTURE].starts_with(b"DDS "),
        "the marker texture, a DDS"
    );
    for (path, template) in template_configs(Engine::PreFox) {
        let config = KitConfig::decode(&entries[&path], PesVersion::Pes17).unwrap();
        let mut expected = KitConfig::decode(&template, PesVersion::Pes17).unwrap();
        expected.shirt.collar = 77;
        expected.shirt.winter_collar = 77;
        assert_eq!(config, expected, "{path}");
        assert_ne!(entries[&path], template, "{path}");
    }
}

#[test]
fn a_data_directory_collar_empty_model_replaces_the_pre_fox_empty_collar_77() {
    let sandbox = Sandbox::new("ref_empty_collar_override");
    write_refs_with_marker(&sandbox, Some(&tracer_player_file("shirt.dds")));
    let empty_collar = b"the cup's empty collar";
    sandbox.write("data/templates/collar_empty.model", empty_collar);

    let (run, entries) = compile_for(&sandbox, PesVersion::Pes17);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        run.messages().first(),
        Some(&format!(
            "Info template_override_active [Keep] (path={})",
            sandbox.display("data/templates/collar_empty.model")
        ))
    );
    assert_eq!(entries[PRE_FOX_EMPTY_COLLAR], empty_collar);
}

#[test]
fn a_ref_marker_that_fails_conversion_is_reported_and_left_out_with_its_collar() {
    let sandbox = Sandbox::new("ref_marker_failed");
    // A codec the converter cannot decode: the deep pass does not look at the marker, so its
    // conversion is what fails.
    write_refs_with_marker(&sandbox, Some(&texture_fixture("bc6h.dds")));

    let (run, entries) = compile(&sandbox);

    let lines = run.messages();
    assert!(
        lines.contains(
            &"refs Cup: Error texture_codec_unsupported [DropFile] at ref_marker.dds \
              (file=ref_marker.dds)"
                .to_owned()
        ),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
    // Ref A still commits, so the refs CPK and its tree are written, without the marker.
    assert!(entries.contains_key(&referee_package("face/real/referee0", "01", "face")));
    assert!(!entries.contains_key(MARKER_COLLAR), "no collar");
    assert!(!entries.contains_key(MARKER_TEXTURE), "no marker texture");
    for (path, template) in template_configs(Engine::Fox) {
        assert!(entries[&path] == template, "{path}");
    }
}

// TC-REF-03
#[test]
fn a_shared_face_two_referees_link_is_merged_into_each_one_s_face() {
    let sandbox = Sandbox::new("ref_shared_face");
    sandbox.write(&format!("{REFS}/players.txt"), b"01 Ref A\n02 Ref B\n");
    sandbox.write(&format!("{REFS}/Faces/Base/face_high.fmdl"), &clean_model());
    sandbox.write(&format!("{REFS}/Players/Ref A/Base.face"), b"");
    sandbox.write(
        &format!("{REFS}/Players/Ref A/hair_high.fmdl"),
        &clean_model(),
    );
    sandbox.write(&format!("{REFS}/Players/Ref B/Base.face"), b"");

    let (run, entries) = compile(&sandbox);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        package_names(&entries[&referee_package("face/real/referee0", "01", "face")]),
        ["face_diff.bin", "face_high.fmdl", "hair_high.fmdl"]
    );
    assert_eq!(
        package_names(&entries[&referee_package("face/real/referee0", "02", "face")]),
        ["face_diff.bin", "face_high.fmdl"]
    );
    for path in entries.keys() {
        assert!(!path.contains("Base"), "{path}");
    }
}

// TC-REF-05
#[test]
fn test_mode_writes_a_referee_folder_s_processed_files_once() {
    let sandbox = Sandbox::new("ref_test_mode");
    write_ref_a(&sandbox, &["01", "20"]);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--mode", "test"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let written: Vec<String> = snapshot(&sandbox.root.join("output/test_output"))
        .into_keys()
        .map(|path| slashed(&path))
        .filter(|path| !path.starts_with("_bins/"))
        .collect();
    assert_eq!(
        written,
        [
            "refs Cup/Players/Ref A/boots.fmdl",
            "refs Cup/Players/Ref A/boots.skl",
            "refs Cup/Players/Ref A/face_diff.bin",
            "refs Cup/Players/Ref A/face_high.fmdl",
            "refs Cup/Players/Ref A/skin.ftex",
        ]
    );
    let tree = referee_tree(Engine::Fox);
    for path in snapshot(&sandbox.root.join("output/test_output")).keys() {
        let path = slashed(path);
        assert!(!path.contains("referee020"), "{path}");
        // The template tree is no export's, so test mode writes none of it, anywhere.
        assert!(
            !tree
                .keys()
                .any(|tree_path| path.ends_with(tree_path.as_str())),
            "{path}"
        );
    }
}

// TC-REF-10
#[test]
fn a_referee_s_link_to_shared_boots_is_written_as_each_of_his_slots_boots() {
    let sandbox = Sandbox::new("ref_shared_boots");
    sandbox.write(&format!("{REFS}/players.txt"), b"01 Ref A\n20 Ref A\n");
    sandbox.write(&format!("{REFS}/Players/Ref A/Studs.boots"), b"");
    sandbox.write(
        &format!("{REFS}/Boots/Studs/boots.fmdl"),
        &tracer_player_file("boots.fmdl"),
    );
    sandbox.write(
        &format!("{REFS}/Boots/Studs/shirt.dds"),
        &tracer_player_file("shirt.dds"),
    );

    let (run, entries) = compile(&sandbox);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let boots: Vec<&str> = entries
        .keys()
        .filter_map(|path| path.strip_prefix("Asset/model/character/boots/"))
        .collect();
    assert_eq!(
        boots,
        [
            "k9901/#Win/boots.fpk",
            "k9901/#Win/boots.fpkd",
            "k9920/#Win/boots.fpk",
            "k9920/#Win/boots.fpkd",
        ]
    );
    for slot in ["01", "20"] {
        let names = package_names(&entries[&referee_package("boots/k99", slot, "boots")]);
        assert!(
            names.contains(&"boots.fmdl".to_owned()),
            "{slot}: {names:?}"
        );
    }
}

// TC-REF-11
#[test]
fn a_refs_export_s_kit_is_file_not_used_and_the_referee_face_compiles() {
    let sandbox = Sandbox::new("ref_kit_not_used");
    write_ref_a(&sandbox, &["01"]);
    sandbox.write(&format!("{REFS}/Kits/p1/kit.dds"), &tracer_kit());
    // The map PES 2018 to 2021 does not read: the kit folder is reported whole, and nothing
    // about its files.
    sandbox.write(&format!("{REFS}/Kits/p1/kit_mask.dds"), &tracer_kit());

    let (run, entries) = compile(&sandbox);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "refs Cup"),
        [
            "Info fmdl_weights_not_normalized [Keep] at Players/Ref A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/Ref A (file=face_high.fmdl, count=1662)",
            "Info export_identified [Keep] (team=referees)",
            "Warning file_not_used [Keep] (file=Kits/p1)",
        ],
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    assert!(
        entries.contains_key(&referee_package("face/real/referee0", "01", "face")),
        "{:#?}",
        entries.keys()
    );
    // The referees wear the template tree's kits alone.
    let kit_textures: Vec<&str> = entries
        .keys()
        .filter_map(|path| path.strip_prefix("Asset/model/character/uniform/texture/#windx11/"))
        .collect();
    assert!(!kit_textures.is_empty());
    assert!(
        kit_textures.iter().all(|name| name.starts_with("referee_")),
        "no kit of the export's: {kit_textures:#?}"
    );
}

/// Writes TC-REF-01's referee in slot 01 beside the tracer as /co/ (714).
fn refs_beside_co(sandbox: &Sandbox) {
    write_ref_a(sandbox, &["01"]);
    sandbox.copy_tracer("co Midcup Tracer");
}

/// Asserts the team CPK at `team` holds /co/'s kit and no referee path, and the refs CPK at
/// `refs` Ref A's face and none of /co/'s paths.
fn assert_split(team: &Path, refs: &Path) {
    let team = cpk_entries(team);
    assert!(team.contains_key(&kit_texture("u0714g1")), "/co/'s kit");
    for path in team.keys() {
        assert!(
            !path.contains("/999/") && !path.contains("referee0") && !path.contains("k99"),
            "{path}"
        );
    }
    let refs = cpk_entries(refs);
    assert!(refs.contains_key(&referee_package("face/real/referee0", "01", "face")));
    for path in refs.keys() {
        assert!(!path.contains("714"), "{path}");
    }
}

// TC-REF-02
#[test]
fn a_refs_export_beside_a_team_goes_into_its_own_cpk_promoted_after_the_team_s() {
    let sandbox = Sandbox::new("ref_beside_team");
    refs_beside_co(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let output = sandbox.root.join("output");
    assert_split(&output.join("4cc_99_test.cpk"), &output.join(REFS_CPK));
    let lines = run.messages();
    let skips: Vec<&String> = lines
        .iter()
        .filter(|line| line.contains("deploy_skipped_by_flag"))
        .collect();
    assert_eq!(
        skips,
        [
            &skipped(&sandbox, "4cc_99_test.cpk"),
            &skipped(&sandbox, REFS_CPK)
        ],
        "one each, the team CPK first"
    );
}

// TC-REF-02
#[test]
fn a_deploying_compile_installs_the_refs_cpk_with_the_team_s() {
    let sandbox = Sandbox::new("ref_beside_team_deploys");
    install_pes(&sandbox);
    refs_beside_co(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    let download = sandbox.root.join("PES/download");
    assert_split(&download.join("4cc_99_test.cpk"), &download.join(REFS_CPK));
    assert_eq!(
        cpk_files(&sandbox.root.join("output")),
        Vec::<String>::new()
    );
}
