//! `compile` over a refs export on Fox (`team_compiler/blue_port.md` "Referee export
//! processing"): each referee folder prepared once and emitted under every slot `players.txt`
//! maps it to, as `face/real/referee0NN`, `k99NN` and `g99NN`, its textures once in team 999's
//! common subfolder of its name, and a link of his made a part of his slots' own packages; in a
//! normal compile all of it goes into the refs CPK, `refs_cpk_name`, beside the team side
//! (`team_compiler/pipeline.md` "5. Writer", step 5).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::common::{Run, Sandbox};
use crate::common_links::texture_directories;
use crate::compile::{
    compiled_players, cpk_entries, kit_texture, pes21_settings, tracer_kit, tracer_player_file,
};
use crate::compile_exports::TEAM_COLOR;
use crate::deploy::{install_pes, templates_folder};
use crate::models::package_names;
use crate::sideload::slashed;
use crate::textures::tracer_model_renaming;
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
    let run = sandbox.run(&pes21_settings(sandbox), &["compile", "--no-deploy"]);
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

/// The Fox referee template tree as the repository holds it, `resources/templates/
/// referees_fox/`: every file by its path below that folder, spelled with `/`, with its bytes.
pub(crate) fn referee_tree() -> BTreeMap<String, Vec<u8>> {
    snapshot(&templates_folder().join("referees_fox"))
        .into_iter()
        .map(|(path, bytes)| (slashed(&path), bytes))
        .collect()
}

/// Asserts that `entries` hold every file of `tree` (the referee template tree's 31), with its
/// bytes.
fn assert_tree_in(entries: &BTreeMap<String, Vec<u8>>, tree: &BTreeMap<String, Vec<u8>>) {
    assert_eq!(tree.len(), 31, "the tree's files");
    for (path, bytes) in tree {
        assert!(entries.get(path) == Some(bytes), "{path}");
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
    assert_tree_in(&entries, &referee_tree());
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

#[test]
fn a_data_directory_file_at_a_tree_path_replaces_that_file_of_the_refs_cpk() {
    let sandbox = Sandbox::new("ref_tree_override");
    write_ref_a(&sandbox, &["01"]);
    let appearance = b"the cup's RefereeAppearance.bin";
    let override_path = format!("data/templates/referees_fox/{REFEREE_APPEARANCE}");
    sandbox.write(&override_path, appearance);

    let (run, entries) = compile(&sandbox);

    assert_eq!(run.exit_code(), 0, "{:#?}", run.messages());
    assert_eq!(
        run.messages().first(),
        Some(&format!(
            "Info template_override_active [Keep] (path={})",
            sandbox.display(&override_path)
        ))
    );
    assert_eq!(entries[REFEREE_APPEARANCE], appearance);
    // The rest of the tree is the built-in one.
    let mut tree = referee_tree();
    tree.insert(REFEREE_APPEARANCE.to_owned(), appearance.to_vec());
    assert_tree_in(&entries, &tree);
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
    let tree = referee_tree();
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

#[test]
fn a_refs_export_s_kit_is_named_and_the_export_beside_it_compiled() {
    let sandbox = Sandbox::new("ref_kit_named");
    write_ref_a(&sandbox, &["01"]);
    sandbox.write(&format!("{REFS}/Kits/p1/kit.dds"), &tracer_kit());
    sandbox.copy_tracer("egg Midcup Tracer");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    let lines = run.messages();
    assert!(
        lines.contains(
            &"refs Cup: Error content_not_yet_compiled [DropExport] (what=Kits/p1)".to_owned()
        ),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(compiled_players(&sandbox), [79205]);
    // The refs export committed nothing, so no refs CPK replaces the installed referees.
    assert_eq!(cpk_files(&sandbox.root.join("output")), ["4cc_99_test.cpk"]);
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
