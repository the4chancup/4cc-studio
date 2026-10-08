//! Cross-format conversion (`team_compiler/pipeline.md` step 3 "Format conversion"): on PES
//! 2015 to 2017 a player folder's `.fmdl` with no `.model` of its stem beside it is converted to
//! a `.model` and its material set, which the face packs and lists in its `face.xml` as it
//! does a member's own; its paired `.skl` is the conversion's bind pose, packed nowhere.

use std::collections::BTreeMap;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes_settings, tracer_player_file};
use crate::prefox_faces::{
    CLEAN, face_cpk, nested_entries, ordered_entries, pes17, sampler_paths, write_slot_05_face,
};
use crate::{clean_model, findings_of};

/// The outer CPK path of the tracer's player's face CPK: team 792, slot 05.
const TRACER_FACE_CPK: &str = "common/character0/model/character/face/real/79205.cpk";

/// The folder every entry of the tracer's player's face CPK sits in.
const TRACER_FACE_FOLDER: &str = "common/character0/model/character/face/real/79205/";

/// The tracer team's Common texture directory on PES 15-17, as a `.mtl` names it.
const TRACER_COMMON: &str = "model/character/uniform/common/792/";

/// The tracer's player's texture home on PES 15-17, as a `.mtl` names it.
const TRACER_HOME: &str = "model/character/uniform/common/792/05 - The Chad Stormworks Player/";

/// The meshes of the `.model` `bytes`, as `pes_model` reads them.
fn model_mesh_count(bytes: &[u8]) -> usize {
    let file = pes_model::format::PreFoxModel::read(bytes).unwrap();
    pes_model::model::Model::from_file(&file)
        .unwrap()
        .meshes
        .len()
}

/// The meshes of the FMDL `bytes`, as `fmdl` reads them.
fn fmdl_mesh_count(bytes: &[u8]) -> usize {
    let file = fmdl::FmdlFile::read(bytes).unwrap();
    fmdl::Model::from_file(&file).unwrap().meshes.len()
}

/// The entries of the face CPK `face` by their names in `folder`.
fn face_names<'a>(face: &'a BTreeMap<String, Vec<u8>>, folder: &str) -> Vec<&'a str> {
    face.keys()
        .map(|path| path.strip_prefix(folder).unwrap())
        .collect()
}

// TC-MOD-27
#[test]
fn the_tracer_s_fox_models_compile_for_pes_17_as_model_files_with_their_materials() {
    let sandbox = Sandbox::new("conversion_tracer_pes17");
    sandbox.copy_tracer("egg Midcup Tracer");

    let run = sandbox.run(&pes_settings(&sandbox, 17), &["compile", "--no-deploy"]);

    let lines = run.messages();
    let folder = "at Players/05 - The Chad Stormworks Player";
    let expected = [
        "Info bin_source [Keep] (bin=TeamColor.bin, cpk=bundled)".to_owned(),
        "Info bin_source [Keep] (bin=UniColor.bin, cpk=bundled)".to_owned(),
        format!(
            "egg Midcup Tracer: Info fmdl_weights_not_normalized [Keep] {folder} (file=boots.fmdl, count=1662)"
        ),
        format!(
            "egg Midcup Tracer: Info fmdl_weights_not_normalized [Keep] {folder} (file=fcl_hair.fmdl, count=1662)"
        ),
        format!(
            "egg Midcup Tracer: Info fmdl_weights_not_normalized [Keep] {folder} (file=glove_l.fmdl, count=2)"
        ),
        "egg Midcup Tracer: Info export_identified [Keep] (team=/egg/, id=792)".to_owned(),
        // No model is typed `face_neck`: the hair and the boots are `parts`.
        format!("egg Midcup Tracer: Info xml_face_neck_added [Keep] {folder} ()"),
        format!(
            "Info deploy_skipped_by_flag [Keep] (path={})",
            sandbox.display("output/4cc_99_test.cpk")
        ),
    ];
    assert_eq!(lines, expected);
    assert_eq!(run.exit_code(), 0);

    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let shirt = format!("common/character1/{TRACER_HOME}shirt.dds");
    assert!(entries.contains_key(&shirt), "{:?}", entries.keys());
    let face = nested_entries(&entries[TRACER_FACE_CPK]);
    // Every model converted, with its material set; no Fox file packed (the `.skl` is the
    // hair's bind pose, the `.fclo` has no PES 17 counterpart).
    assert_eq!(
        face_names(&face, TRACER_FACE_FOLDER),
        [
            "boots.mtl",
            "dummy.mtl",
            "face.xml",
            "fcl_hair.mtl",
            "glove_l.mtl",
            "glove_r.mtl",
            "oral_boots_win32.model",
            "oral_dummy_win32.model",
            "oral_fcl_hair_win32.model",
            "oral_glove_l_win32.model",
            "oral_glove_r_win32.model",
        ]
    );
    for path in entries.keys() {
        assert!(
            [".fmdl", ".skl", ".fclo"]
                .iter()
                .all(|extension| !path.ends_with(extension)),
            "{path}"
        );
    }
    // The hair and the boots carry an anti-blur mesh (the third), which a `.model` has no
    // counterpart for: the conversion folds it back into its source material.
    for (stem, meshes) in [
        ("fcl_hair", (3, 2)),
        ("boots", (3, 2)),
        ("glove_l", (1, 1)),
        ("glove_r", (1, 1)),
    ] {
        let model = &face[&format!("{TRACER_FACE_FOLDER}oral_{stem}_win32.model")];
        let counts = (
            fmdl_mesh_count(&tracer_player_file(&format!("{stem}.fmdl"))),
            model_mesh_count(model),
        );
        assert_eq!(counts, meshes, "{stem}");
    }
    let hair_mtl = &face[&format!("{TRACER_FACE_FOLDER}fcl_hair.mtl")];
    let hair_samplers = sampler_paths(hair_mtl);
    assert!(
        hair_samplers.contains(&format!("{TRACER_HOME}shirt.dds")),
        "{hair_samplers:?}"
    );
    // The FMDL's Fox dummy normal and specular maps are no texture on PES 17, so its `kit`
    // material takes the lowest `Basic_*` rung; its reserved `dummy_kit` is named where the
    // modded exes substitute it, the team's Common directory. No Fox path survives.
    let hair = pes_model::format::mtl::MaterialSet::read(hair_mtl).unwrap();
    let kit = hair
        .materials
        .iter()
        .find(|material| material.name == "kit")
        .unwrap();
    assert_eq!(kit.shader, "Basic_C");
    let kit_samplers: Vec<(String, String)> = pes_model::ops::paths::texture_paths(&hair)
        .into_iter()
        .filter(|path| path.material == "kit")
        .map(|path| {
            (
                path.sampler,
                format!("{}{}", path.directory, path.file_name),
            )
        })
        .collect();
    assert_eq!(
        kit_samplers,
        [(
            "DiffuseMap".to_owned(),
            format!("{TRACER_COMMON}dummy_kit.dds")
        )]
    );
    for stem in ["boots", "fcl_hair", "glove_l", "glove_r"] {
        for path in sampler_paths(&face[&format!("{TRACER_FACE_FOLDER}{stem}.mtl")]) {
            assert!(path.starts_with(TRACER_COMMON), "{stem}.mtl: {path}");
        }
    }
    let entry = |xml_type: &str, stem: &str| {
        (
            xml_type.to_owned(),
            format!("./oral_{stem}_*.model"),
            format!("./{stem}.mtl"),
        )
    };
    assert_eq!(
        ordered_entries(&face[&format!("{TRACER_FACE_FOLDER}face.xml")]),
        [
            entry("parts", "boots"),
            entry("parts", "fcl_hair"),
            entry("gloveL", "glove_l"),
            entry("gloveR", "glove_r"),
            entry("face_neck", "dummy"),
        ]
    );
}

#[test]
fn a_fox_model_re_bound_to_pes_15_s_skeleton_is_reported_naming_it() {
    let sandbox = Sandbox::new("conversion_tracer_pes15");
    sandbox.copy_tracer("egg Midcup Tracer");

    let run = sandbox.run(&pes_settings(&sandbox, 15), &["compile", "--no-deploy"]);

    let lines = run.messages();
    // Seven of the bones the hair and the boots use sit elsewhere in PES 2015's skeleton;
    // the gloves use none of them.
    let folder = "at Players/05 - The Chad Stormworks Player";
    assert_eq!(
        findings_of(&lines, "egg Midcup Tracer"),
        [
            format!(
                "Info fmdl_weights_not_normalized [Keep] {folder} (file=boots.fmdl, count=1662)"
            ),
            format!(
                "Info fmdl_weights_not_normalized [Keep] {folder} (file=fcl_hair.fmdl, count=1662)"
            ),
            format!(
                "Info fmdl_weights_not_normalized [Keep] {folder} (file=glove_l.fmdl, count=2)"
            ),
            "Info export_identified [Keep] (team=/egg/, id=792)".to_owned(),
            format!("Info skeleton_retargeted [Keep] {folder} (model=boots.fmdl, bones=7)"),
            format!("Info skeleton_retargeted [Keep] {folder} (model=fcl_hair.fmdl, bones=7)"),
            format!("Info xml_face_neck_added [Keep] {folder} ()"),
        ]
    );
    assert_eq!(run.exit_code(), 0);
}

/// `compile --no-deploy` for PES 17 of the sandbox's exports: the lines of the export `name`
/// and slot 05's face CPK entries.
fn compiled_slot_05(sandbox: &Sandbox, name: &str) -> (Vec<String>, BTreeMap<String, Vec<u8>>) {
    let run = sandbox.run(&pes17(sandbox), &["compile", "--no-deploy"]);
    let lines = run.messages();
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let findings = findings_of(&lines, name)
        .into_iter()
        .map(str::to_owned)
        .collect();
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    (findings, nested_entries(&entries[&face_cpk(5)]))
}

#[test]
fn an_fmdl_a_model_of_its_stem_beats_on_pes_17_is_ignored_with_no_finding() {
    let export = "co Midcup Card";
    let without = Sandbox::new("conversion_beaten_without");
    write_slot_05_face(&without, export);
    let with = Sandbox::new("conversion_beaten_with");
    write_slot_05_face(&with, export);
    with.write(
        &format!("exports/{export}/Players/05 - A/face_high.fmdl"),
        &clean_model(),
    );

    let (findings_without, face_without) = compiled_slot_05(&without, export);
    let (findings_with, face_with) = compiled_slot_05(&with, export);

    assert_eq!(findings_without, CLEAN);
    assert_eq!(findings_with, findings_without);
    assert_eq!(face_with, face_without);
}

#[test]
fn a_fox_model_whose_conversion_fails_drops_its_folder_naming_the_model() {
    let sandbox = Sandbox::new("conversion_failed");
    sandbox.copy_tracer("egg Midcup Tracer");
    // The hair's skeleton with one of the FMDL's bones parented past the skeleton's own bone
    // count: `fmdl` reads it, nothing checks a `.skl` before the conversion, which refuses it
    // (`model_convert`'s `out_of_range_parents_error`).
    let mut skeleton = fmdl::SklFile::read(&tracer_player_file("fcl_hair.skl")).unwrap();
    let parent = skeleton.bones.len() + 5;
    let hair = fmdl::Model::from_file(
        &fmdl::FmdlFile::read(&tracer_player_file("fcl_hair.fmdl")).unwrap(),
    )
    .unwrap();
    let bone = skeleton
        .bones
        .iter_mut()
        .find(|bone| hair.bones.iter().any(|used| used.name == bone.name))
        .unwrap();
    bone.parent = Some(parent);
    sandbox.write(
        "exports/egg Midcup Tracer/Players/05 - The Chad Stormworks Player/fcl_hair.skl",
        &skeleton.write(),
    );

    let run = sandbox.run(&pes_settings(&sandbox, 17), &["compile", "--no-deploy"]);

    let lines = run.messages();
    let error = fmdl::FmdlError::BadReference {
        what: "skl parent",
        index: parent,
    };
    let failed = format!(
        "egg Midcup Tracer: Error model_conversion_failed [DropFolder] at Players/05 - The Chad Stormworks Player (model=fcl_hair.fmdl, error={error})"
    );
    assert!(lines.contains(&failed), "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    assert!(
        !entries.contains_key(TRACER_FACE_CPK),
        "{:?}",
        entries.keys()
    );
}
