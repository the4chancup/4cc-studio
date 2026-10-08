//! Cross-format conversion (`team_compiler/pipeline.md` step 3 "Format conversion"): on PES
//! 2015 to 2017 a player folder's `.fmdl` with no `.model` of its stem beside it is converted to
//! a `.model` and its material set, which the face packs and lists in its `face.xml` as it
//! does a member's own; its paired `.skl` is the conversion's bind pose, packed nowhere. On PES
//! 2018 to 2021 a player folder's `.model` with no `.fmdl` of its stem is converted with its
//! `.mtl` to an FMDL, which the Models task packs as it packs a member's FMDL, with the skeleton
//! the conversion writes, when it writes one, as the part's. A model of the other format beside
//! one of the target's is ignored with no finding, and a converted model whose conversion fails
//! or whose converted form the game cannot load leaves its package out, the folder's other
//! packages and its textures standing (the writer's rule for every task failure).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::common::Sandbox;
use crate::compile::{cpk_entries, pes_settings, tracer_player_file};
use crate::models::{body_skl, package_names};
use pes_model::format::mtl::{
    Address, Filter, Material, MaterialEntry, MaterialSet, Sampler, Vector,
};

use crate::prefox_faces::{
    CLEAN, card_materials, card_model, face_cpk, face_folder, nested_entries, ordered_entries,
    pes17, sampler_paths, small_dds, write_round_hat, write_slot_05_face,
};
use crate::textures::texture_fixture;
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
        // What each conversion drops: the FMDL's redundant bone-matrix block, the hair's
        // `.skl` parents the FMDL's win over, and the `shirt` mesh's no-shadow flag, which a
        // `.mtl` cannot express.
        format!(
            "egg Midcup Tracer: Info native_field_dropped [Keep] {folder} (model=boots.fmdl, field=bone_matrices)"
        ),
        format!(
            "egg Midcup Tracer: Warning mesh_flags_dropped [Keep] {folder} (model=boots.fmdl, material=1, field=no_shadow_cast)"
        ),
        format!(
            "egg Midcup Tracer: Info native_field_dropped [Keep] {folder} (model=fcl_hair.fmdl, field=bone_matrices)"
        ),
        format!(
            "egg Midcup Tracer: Info native_field_dropped [Keep] {folder} (model=fcl_hair.fmdl, bone=8, field=skl_parent)"
        ),
        format!(
            "egg Midcup Tracer: Info native_field_dropped [Keep] {folder} (model=fcl_hair.fmdl, bone=25, field=skl_parent)"
        ),
        format!(
            "egg Midcup Tracer: Warning mesh_flags_dropped [Keep] {folder} (model=fcl_hair.fmdl, material=1, field=no_shadow_cast)"
        ),
        format!(
            "egg Midcup Tracer: Info native_field_dropped [Keep] {folder} (model=glove_l.fmdl, field=bone_matrices)"
        ),
        format!(
            "egg Midcup Tracer: Info native_field_dropped [Keep] {folder} (model=glove_r.fmdl, field=bone_matrices)"
        ),
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
    // No material of the tracer is metal: no environment sampler, and no template
    // environment map emitted (TC-MOD-36 has a metal one).
    for stem in ["boots", "fcl_hair", "glove_l", "glove_r"] {
        let set = MaterialSet::read(&face[&format!("{TRACER_FACE_FOLDER}{stem}.mtl")]).unwrap();
        assert_eq!(
            environment_samplers(&set),
            Vec::<&Sampler>::new(),
            "{stem}.mtl"
        );
    }
    assert!(
        entries.keys().all(|path| !path.ends_with("/env.dds")),
        "{:?}",
        entries.keys()
    );
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
            // Each model's losses in the order the conversion reports them, the re-binding
            // between the import's and the `.model` export's.
            format!(
                "Info native_field_dropped [Keep] {folder} (model=boots.fmdl, field=bone_matrices)"
            ),
            format!("Info skeleton_retargeted [Keep] {folder} (model=boots.fmdl, bones=7)"),
            format!(
                "Warning mesh_flags_dropped [Keep] {folder} (model=boots.fmdl, material=1, field=no_shadow_cast)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=fcl_hair.fmdl, field=bone_matrices)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=fcl_hair.fmdl, bone=8, field=skl_parent)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=fcl_hair.fmdl, bone=25, field=skl_parent)"
            ),
            format!("Info skeleton_retargeted [Keep] {folder} (model=fcl_hair.fmdl, bones=7)"),
            format!(
                "Warning mesh_flags_dropped [Keep] {folder} (model=fcl_hair.fmdl, material=1, field=no_shadow_cast)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=glove_l.fmdl, field=bone_matrices)"
            ),
            format!(
                "Info native_field_dropped [Keep] {folder} (model=glove_r.fmdl, field=bone_matrices)"
            ),
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

/// Slot 05's folder in the export `export` of team /co/ 714.
fn slot_05(export: &str) -> String {
    format!("exports/{export}/Players/05 - A")
}

/// `compile --no-deploy` of the sandbox's exports for PES `version` (`pes_settings`, with
/// `extra` settings appended): the run's exit code, the lines of the export `name`, and the
/// output CPK's entries, none when the run wrote no CPK.
fn compiled_for(
    sandbox: &Sandbox,
    version: u8,
    extra: &str,
    name: &str,
) -> (u8, Vec<String>, BTreeMap<String, Vec<u8>>) {
    let settings = format!("{}{extra}", pes_settings(sandbox, version));
    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);
    let lines = findings_of(&run.messages(), name)
        .into_iter()
        .map(str::to_owned)
        .collect();
    let cpk = sandbox.root.join("output/4cc_99_test.cpk");
    let entries = if cpk.exists() {
        cpk_entries(&cpk)
    } else {
        BTreeMap::new()
    };
    (run.exit_code(), lines, entries)
}

/// The card head's face model with `edit` applied to its parsed form, written back.
fn edited_card(edit: impl FnOnce(&mut pes_model::model::Model)) -> Vec<u8> {
    let file = pes_model::format::PreFoxModel::read(&card_model()).unwrap();
    let mut model = pes_model::model::Model::from_file(&file).unwrap();
    edit(&mut model);
    model.to_file().unwrap().write().unwrap()
}

/// Writes slot 05 of the export `export`: `boots.model` holding `model`, the card head's
/// material set naming `skin` as `boots.mtl`, and `skin.dds`.
fn write_boots_model(sandbox: &Sandbox, export: &str, model: &[u8]) {
    let player = slot_05(export);
    sandbox.write(&format!("{player}/boots.model"), model);
    sandbox.write(&format!("{player}/boots.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());
}

/// The boots package of slot 05 of team 714 on PES 21.
const BOOTS_FPK: &str = "Asset/model/character/boots/k0625/#Win/boots.fpk";

/// Slot 05's texture home on PES 21, as an FMDL names it.
const HOME_714_05: &str = "/Assets/pes16/model/character/common/714/05 - A/sourceimages/";

// TC-MOD-26
#[test]
fn a_model_beside_the_fmdl_of_its_stem_is_ignored_on_pes_21_and_used_on_pes_17() {
    let export = "co Midcup Boots";
    // Writes slot 05: `skin.dds` when `skin`, the tracer's boots texture, the tracer's boots
    // FMDL (naming `shirt`) when `fmdl`, and the card head as boots with its material set
    // (naming `skin`) when `model`.
    let write = |sandbox: &Sandbox, skin: bool, fmdl: bool, model: bool| {
        let player = slot_05(export);
        if skin {
            sandbox.write(&format!("{player}/skin.dds"), &small_dds());
        }
        sandbox.write(
            &format!("{player}/shirt.dds"),
            &tracer_player_file("shirt.dds"),
        );
        if fmdl {
            sandbox.write(
                &format!("{player}/boots.fmdl"),
                &tracer_player_file("boots.fmdl"),
            );
        }
        if model {
            sandbox.write(&format!("{player}/boots.model"), &card_model());
            sandbox.write(&format!("{player}/boots.mtl"), &card_materials());
        }
    };
    // The tracer's boots report their weights where they are the selected model, and the
    // beaten one reports nothing: on PES 17 neither the beaten FMDL's weights nor, on PES 21,
    // with no `skin.dds`, the beaten `.model`'s `.mtl` naming the missing `skin`.
    let weights =
        "Info fmdl_weights_not_normalized [Keep] at Players/05 - A (file=boots.fmdl, count=1662)";
    let identified = "Info export_identified [Keep] (team=/co/, id=714)";
    let no_colors = "Info team_colors_missing [Keep] ()";
    let neck = "Info xml_face_neck_added [Keep] at Players/05 - A ()";
    for (version, skin, alone_fmdl, alone_model, run_name, expected) in [
        (
            21,
            false,
            true,
            false,
            "pes21",
            vec![weights, identified, no_colors],
        ),
        (
            17,
            true,
            false,
            true,
            "pes17",
            vec![identified, no_colors, neck],
        ),
    ] {
        let both = Sandbox::new(&format!("conversion_twins_{run_name}_both"));
        write(&both, skin, true, true);
        let alone = Sandbox::new(&format!("conversion_twins_{run_name}_alone"));
        write(&alone, skin, alone_fmdl, alone_model);

        let (both_code, both_lines, both_entries) = compiled_for(&both, version, "", export);
        let (alone_code, alone_lines, alone_entries) = compiled_for(&alone, version, "", export);

        assert_eq!(
            (both_code, alone_code),
            (0, 0),
            "PES {version}: {both_lines:#?}"
        );
        assert_eq!(both_lines, expected, "PES {version}");
        assert_eq!(alone_lines, expected, "PES {version}");
        assert!(
            both_entries == alone_entries,
            "PES {version}: the CPKs differ: {:?} against {:?}",
            both_entries.keys(),
            alone_entries.keys()
        );
    }
}

// TC-MOD-34
#[test]
fn a_model_alone_is_converted_to_the_boots_fmdl_with_its_texture_in_the_player_s_folder() {
    let sandbox = Sandbox::new("conversion_model_for_fox");
    let export = "co Midcup Boots";
    write_boots_model(&sandbox, export, &card_model());

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    assert_eq!(
        lines,
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert_eq!(code, 0);
    let skin = "Asset/model/character/common/714/05 - A/sourceimages/#windx11/skin.ftex";
    assert!(entries.contains_key(skin), "{:?}", entries.keys());
    let package = fpk::FpkFile::read(&entries[BOOTS_FPK]).unwrap();
    assert_eq!(
        package_names(&entries[BOOTS_FPK]),
        ["boots.fmdl", "boots.skl"]
    );
    let file = fmdl::FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap();
    let boots = fmdl::Model::from_file(&file).unwrap();
    // The card's one mesh, and the anti-blur mesh the FMDL export makes for it.
    let anti_blur = boots
        .meshes
        .iter()
        .filter(|mesh| mesh.is_antiblur_mesh)
        .count();
    assert_eq!(
        (
            model_mesh_count(&card_model()),
            boots.meshes.len(),
            anti_blur
        ),
        (1, 2, 1)
    );
    // The `.mtl`'s `./skin.dds`, pointed at the player's texture home by its stem.
    let skin_paths: Vec<(String, String)> = fmdl::ops::paths::texture_paths(&file)
        .unwrap()
        .into_iter()
        .filter(|path| path.file_name.starts_with("skin."))
        .map(|path| (path.directory, path.file_name))
        .collect();
    assert!(!skin_paths.is_empty());
    for path in &skin_paths {
        assert_eq!(path, &(HOME_714_05.to_owned(), "skin.dds".to_owned()));
    }
    // The card's one bone is the game's own, at its pose: the conversion writes no skeleton,
    // so the boots get the bundled one.
    assert_eq!(package.get("boots.skl").unwrap(), body_skl("pes21"));
}

// TC-MOD-29
#[test]
fn a_model_whose_conversion_fails_leaves_its_package_out_even_with_pass_through() {
    let export = "co Midcup Boots";
    // The card head's one bone stores an all-zero matrix: `pes_model` reads it and its check
    // finds nothing, and the conversion cannot invert it.
    let singular = edited_card(|model| model.bones[0].matrix = [0.0; 12]);
    for (pass_through, extra) in [
        (false, ""),
        (true, "[team-compiler]\npass_through = true\n"),
    ] {
        let sandbox = Sandbox::new(&format!("conversion_singular_{pass_through}"));
        write_boots_model(&sandbox, export, &singular);

        let (code, lines, entries) = compiled_for(&sandbox, 21, extra, export);

        let failed = "Error model_conversion_failed [DropFolder] at Players/05 - A (model=boots.model, error=bone 0's stored matrix is singular)";
        assert!(lines.iter().any(|line| line == failed), "{lines:#?}");
        assert_eq!(code, 1, "pass_through {pass_through}");
        // No boots. The folder's other package, its blank face, and its textures commit as
        // beside any failed package (the writer's group rule).
        let paths: Vec<&str> = entries.keys().map(String::as_str).collect();
        assert_eq!(
            paths,
            [
                "Asset/model/character/common/714/05 - A/sourceimages/#windx11/skin.ftex",
                "Asset/model/character/face/real/71405/#Win/face.fpk",
                "Asset/model/character/face/real/71405/#Win/face.fpkd",
                "common/character0/model/character/uniform/team/UniColor.bin",
                "common/etc/TeamColor.bin",
            ],
            "pass_through {pass_through}"
        );
    }
}

// TC-MOD-30
#[test]
fn a_model_with_a_far_vertex_drops_its_folder_on_pes_21() {
    let sandbox = Sandbox::new("conversion_far_model");
    let export = "co Midcup Boots";
    let far = edited_card(|model| model.meshes[0].vertices.positions[0] = [6000.0, 0.0, 0.0]);
    write_boots_model(&sandbox, export, &far);

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    let far_line = "Error vertex_too_far_from_origin [DropFolder] at Players/05 - A (file=boots.model, count=1)";
    assert!(lines.iter().any(|line| line == far_line), "{lines:#?}");
    assert_eq!(code, 1);
    assert!(
        entries
            .keys()
            .all(|path| !path.contains("k0625") && !path.contains("71405")),
        "{:?}",
        entries.keys()
    );
}

// TC-MOD-26
#[test]
fn a_model_an_fmdl_beats_on_pes_21_drops_nothing_with_its_far_vertex() {
    let sandbox = Sandbox::new("conversion_far_model_beaten");
    let export = "co Midcup Boots";
    let player = slot_05(export);
    sandbox.write(&format!("{player}/boots.fmdl"), &clean_model());
    let far = edited_card(|model| model.meshes[0].vertices.positions[0] = [6000.0, 0.0, 0.0]);
    sandbox.write(&format!("{player}/boots.model"), &far);
    sandbox.write(&format!("{player}/boots.mtl"), &card_materials());

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    assert!(
        lines
            .iter()
            .all(|line| !line.contains("vertex_too_far_from_origin")),
        "{lines:#?}"
    );
    assert_eq!(code, 0, "{lines:#?}");
    let package = fpk::FpkFile::read(&entries[BOOTS_FPK]).unwrap();
    assert_eq!(
        fmdl_mesh_count(package.get("boots.fmdl").unwrap()),
        fmdl_mesh_count(&clean_model())
    );
}

// TC-MOD-28
#[test]
fn a_selected_gltf_drops_its_folder_on_pes_21_and_one_an_fmdl_beats_is_ignored() {
    let export = "co Midcup Gltf";
    let player = slot_05(export);
    // The glTF is never read, so any bytes stand for one.
    let gltf: &[u8] = b"glTF";
    // The glTF beats the `.model` beside it, which has no `.mtl` it would need if it were
    // converted.
    let selected = Sandbox::new("conversion_gltf_selected");
    selected.write(&format!("{player}/boots.glb"), gltf);
    selected.write(&format!("{player}/boots.model"), &card_model());
    selected.write(&format!("{player}/skin.png"), &texture_fixture("skin.png"));
    // Slot 07 compiles, so the run writes a CPK slot 05 is missing from.
    selected.write(
        &format!("exports/{export}/Players/07 - B/boots.fmdl"),
        &clean_model(),
    );
    // The FMDL beats the glTF beside it.
    let beaten = Sandbox::new("conversion_gltf_beaten");
    beaten.write(&format!("{player}/boots.glb"), gltf);
    beaten.write(&format!("{player}/boots.fmdl"), &clean_model());
    let identified = "Info export_identified [Keep] (team=/co/, id=714)";
    let no_colors = "Info team_colors_missing [Keep] ()";

    let (code, lines, entries) = compiled_for(&selected, 21, "", export);

    assert_eq!(
        lines,
        [
            identified,
            "Error model_gltf_unsupported [DropFolder] at Players/05 - A (file=boots.glb)",
            no_colors,
        ]
    );
    assert_eq!(code, 1);
    assert!(
        entries.contains_key("Asset/model/character/boots/k0627/#Win/boots.fpk"),
        "{:?}",
        entries.keys()
    );
    assert!(
        entries.keys().all(|path| !path.contains("k0625")
            && !path.contains("71405")
            && !path.contains("skin")),
        "{:?}",
        entries.keys()
    );

    let (code, lines, entries) = compiled_for(&beaten, 21, "", export);

    assert_eq!(lines, [identified, no_colors]);
    assert_eq!(code, 0);
    assert_eq!(
        package_names(&entries[BOOTS_FPK]),
        ["boots.fmdl", "boots.skl"]
    );
    let package = fpk::FpkFile::read(&entries[BOOTS_FPK]).unwrap();
    assert_eq!(
        fmdl_mesh_count(package.get("boots.fmdl").unwrap()),
        fmdl_mesh_count(&clean_model())
    );

    // `check` reports neither the glTF nor the `.model` it beats.
    for sandbox in [&selected, &beaten] {
        let run = sandbox.run(&pes_settings(sandbox, 21), &["check"]);
        assert_eq!(
            findings_of(&run.messages(), export),
            [identified],
            "{}",
            sandbox.root.display()
        );
        assert_eq!(run.exit_code(), 0);
    }
}

/// The boots package of team 714's first shared boots folder on PES 21.
const SHARED_BOOTS_FPK: &str = "Asset/model/character/boots/k0644/#Win/boots.fpk";

#[test]
fn a_shared_boots_folder_s_model_is_converted_to_its_boots_fmdl_on_pes_21() {
    let sandbox = Sandbox::new("conversion_shared_model_for_fox");
    let export = "co Midcup Shared";
    sandbox.write(&format!("{}/Crocs.boots", slot_05(export)), b"");
    let crocs = format!("exports/{export}/Boots/Crocs");
    sandbox.write(&format!("{crocs}/boots.model"), &card_model());
    sandbox.write(&format!("{crocs}/boots.mtl"), &card_materials());
    sandbox.write(&format!("{crocs}/skin.dds"), &small_dds());

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    assert_eq!(lines, CLEAN);
    assert_eq!(code, 0);
    let skin = "Asset/model/character/boots/k0644/#windx11/skin.ftex";
    assert!(entries.contains_key(skin), "{:?}", entries.keys());
    assert_eq!(
        package_names(&entries[SHARED_BOOTS_FPK]),
        ["boots.fmdl", "boots.skl"]
    );
    let package = fpk::FpkFile::read(&entries[SHARED_BOOTS_FPK]).unwrap();
    let file = fmdl::FmdlFile::read(package.get("boots.fmdl").unwrap()).unwrap();
    let boots = fmdl::Model::from_file(&file).unwrap();
    // The card's one mesh, and the anti-blur mesh the FMDL export makes for it.
    let anti_blur = boots
        .meshes
        .iter()
        .filter(|mesh| mesh.is_antiblur_mesh)
        .count();
    assert_eq!(
        (
            model_mesh_count(&card_model()),
            boots.meshes.len(),
            anti_blur
        ),
        (1, 2, 1)
    );
    // The `.mtl`'s `./skin.dds`, pointed at the shared output's folder by its stem.
    let skin_paths: Vec<(String, String)> = fmdl::ops::paths::texture_paths(&file)
        .unwrap()
        .into_iter()
        .filter(|path| path.file_name.starts_with("skin."))
        .map(|path| (path.directory, path.file_name))
        .collect();
    assert!(!skin_paths.is_empty());
    for path in &skin_paths {
        assert_eq!(
            path,
            &(
                "/Assets/pes16/model/character/boots/k0644/".to_owned(),
                "skin.dds".to_owned()
            )
        );
    }
}

#[test]
fn a_shared_folder_s_selected_gltf_drops_every_player_linking_it_on_pes_21_and_17() {
    let export = "co Midcup Gltf";
    let gltf_error =
        "Error model_gltf_unsupported [DropFolder] at Players/05 - A (file=Boots/Crocs/boots.glb)";
    let identified = "Info export_identified [Keep] (team=/co/, id=714)";
    let no_colors = "Info team_colors_missing [Keep] ()";
    let slot_06 = format!("exports/{export}/Players/06 - B");
    for (version, findings, kept, dropped) in [
        (
            21,
            vec![identified, gltf_error, no_colors],
            "Asset/model/character/boots/k0626/#Win/boots.fpk".to_owned(),
            ["k0625", "71405", "k0644"],
        ),
        (
            17,
            vec![
                identified,
                gltf_error,
                no_colors,
                "Info xml_face_neck_added [Keep] at Players/06 - B ()",
            ],
            face_cpk(6),
            ["71405", "boots/k0644", "05 - A"],
        ),
    ] {
        let sandbox = Sandbox::new(&format!("conversion_shared_gltf_pes{version}"));
        sandbox.write(&format!("{}/Crocs.boots", slot_05(export)), b"");
        // The glTF is never read, so any bytes stand for one.
        sandbox.write(&format!("exports/{export}/Boots/Crocs/boots.glb"), b"glTF");
        // Slot 06 compiles, so the run writes a CPK slot 05 is missing from.
        if version == 21 {
            sandbox.write(&format!("{slot_06}/boots.fmdl"), &clean_model());
        } else {
            sandbox.write(&format!("{slot_06}/boots.model"), &card_model());
            sandbox.write(&format!("{slot_06}/boots.mtl"), &card_materials());
            sandbox.write(&format!("{slot_06}/skin.dds"), &small_dds());
        }

        let (code, lines, entries) = compiled_for(&sandbox, version, "", export);

        assert_eq!(lines, findings, "PES {version}");
        assert_eq!(code, 1, "PES {version}");
        assert!(entries.contains_key(&kept), "{:?}", entries.keys());
        assert!(
            entries
                .keys()
                .all(|path| dropped.iter().all(|part| !path.contains(part))),
            "PES {version}: {:?}",
            entries.keys()
        );

        // `check` reports neither: the drop is planning's.
        let run = sandbox.run(&pes_settings(&sandbox, version), &["check"]);
        assert_eq!(
            findings_of(&run.messages(), export),
            [identified],
            "PES {version}"
        );
        assert_eq!(run.exit_code(), 0, "PES {version}");
    }
}

#[test]
fn a_selected_model_s_mtl_naming_a_texture_nobody_supplies_is_a_warning_on_pes_21() {
    let sandbox = Sandbox::new("conversion_mtl_texture_missing");
    let export = "co Midcup Boots";
    let player = slot_05(export);
    // No `skin.dds`, which the `.mtl` names.
    sandbox.write(&format!("{player}/boots.model"), &card_model());
    sandbox.write(&format!("{player}/boots.mtl"), &card_materials());

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    assert_eq!(
        lines,
        [
            "Warning mtl_texture_not_found [Keep] at Players/05 - A (file=boots.mtl, texture=./skin.dds, materials=card)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ]
    );
    assert_eq!(code, 0);
    assert!(
        package_names(&entries[BOOTS_FPK]).contains(&"boots.fmdl".to_owned()),
        "{:?}",
        entries.keys()
    );
}

/// The faces of the model `name` in the package at `path` in `entries`, read back with `fmdl`.
fn face_count(entries: &BTreeMap<String, Vec<u8>>, path: &str, name: &str) -> usize {
    let package = fpk::FpkFile::read(&entries[path]).unwrap();
    let file = fmdl::FmdlFile::read(package.get(name).unwrap()).unwrap();
    let model = fmdl::Model::from_file(&file).unwrap();
    model.meshes.iter().map(|mesh| mesh.faces.len()).sum()
}

#[test]
fn a_hand_weighted_model_is_converted_then_gives_its_hands_to_the_player_s_gloves() {
    let sandbox = Sandbox::new("conversion_hand_split");
    let export = "co Midcup Hands";
    // The hand-split strip as a pre-Fox pair (`tests/fixtures/hand_split/README.md`): 40
    // faces, of which each hand's split takes 8 and the body keeps 24. Its `.mtl` names no
    // texture.
    let fixture = |name: &str| {
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/hand_split")
                .join(name),
        )
        .unwrap()
    };
    let player = slot_05(export);
    sandbox.write(&format!("{player}/body.model"), &fixture("body.model"));
    sandbox.write(&format!("{player}/body.mtl"), &fixture("body.mtl"));

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    assert_eq!(
        lines,
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info fmdl_fcl_hair_fallback [Keep] at Players/05 - A (file=body.model)",
            "Info team_colors_missing [Keep] ()",
            // The face's conversion reports once; the gloves' conversion of the same model,
            // for its hands, reports nothing.
            "Info dummy_texture_added [Keep] at Players/05 - A (model=body.model, material=0, sampler=NormalMap_Tex_NRM)",
            "Info dummy_texture_added [Keep] at Players/05 - A (model=body.model, material=0, sampler=SpecularMap_Tex_LIN)",
            "Info model_hand_split [Keep] at Players/05 - A (model=body.model, gloves=glove_l, glove_r)",
        ]
    );
    assert_eq!(code, 0);
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
}

/// The card head's face model with its one bone renamed to one no game skeleton holds, which
/// the conversion keeps and writes a skeleton for.
fn card_with_its_own_bone() -> Vec<u8> {
    edited_card(|model| model.bones[0].name = "my_bone".to_owned())
}

#[test]
fn the_skeleton_a_conversion_writes_is_the_boots_or_beside_a_member_s_a_conflict() {
    let export = "co Midcup Boots";
    let alone = Sandbox::new("conversion_skeleton_alone");
    write_boots_model(&alone, export, &card_with_its_own_bone());

    let (code, lines, entries) = compiled_for(&alone, 21, "", export);

    assert_eq!(code, 0, "{lines:#?}");
    let package = fpk::FpkFile::read(&entries[BOOTS_FPK]).unwrap();
    let skeleton = fmdl::SklFile::read(package.get("boots.skl").unwrap()).unwrap();
    assert!(
        skeleton.bones.iter().any(|bone| bone.name == "my_bone"),
        "{:?}",
        skeleton
            .bones
            .iter()
            .map(|bone| &bone.name)
            .collect::<Vec<_>>()
    );

    let generated = package.get("boots.skl").unwrap().to_vec();

    // A member's `.skl` of the boots' stem holding the same bytes is the same skeleton.
    let same = Sandbox::new("conversion_skeleton_same");
    write_boots_model(&same, export, &card_with_its_own_bone());
    same.write(&format!("{}/boots.skl", slot_05(export)), &generated);

    let (code, lines, entries) = compiled_for(&same, 21, "", export);

    assert_eq!(code, 0, "{lines:#?}");
    let package = fpk::FpkFile::read(&entries[BOOTS_FPK]).unwrap();
    assert!(package.get("boots.skl").unwrap() == generated.as_slice());

    // A member's `.skl` of the boots' stem, the bundled PES 21 one, which lacks the bone.
    let beside = Sandbox::new("conversion_skeleton_beside");
    write_boots_model(&beside, export, &card_with_its_own_bone());
    beside.write(
        &format!("{}/boots.skl", slot_05(export)),
        &body_skl("pes21"),
    );

    let (code, lines, entries) = compiled_for(&beside, 21, "", export);

    let conflict = "Error skl_merge_conflict [DropFolder] at Players/05 - A (skeleton=differs)";
    assert!(lines.iter().any(|line| line == conflict), "{lines:#?}");
    assert_eq!(code, 1);
    assert!(!entries.contains_key(BOOTS_FPK), "{:?}", entries.keys());
}

#[test]
fn the_skeleton_a_slotless_model_s_conversion_writes_is_left_out_as_skl_no_slot() {
    let sandbox = Sandbox::new("conversion_skeleton_slotless");
    let export = "co Midcup Face";
    let player = slot_05(export);
    sandbox.write(
        &format!("{player}/face_high.model"),
        &card_with_its_own_bone(),
    );
    sandbox.write(&format!("{player}/face_high.mtl"), &card_materials());
    sandbox.write(&format!("{player}/skin.dds"), &small_dds());

    let (code, lines, entries) = compiled_for(&sandbox, 21, "", export);

    assert_eq!(
        lines,
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Warning skl_no_slot [Keep] at Players/05 - A (model=face_high.model)",
        ]
    );
    assert_eq!(code, 0);
    assert_eq!(
        package_names(&entries["Asset/model/character/face/real/71405/#Win/face.fpk"]),
        ["face_diff.bin", "face_high.fmdl"]
    );
}

/// The `EnvironmentMap` samplers of `set`, every material's.
fn environment_samplers(set: &MaterialSet) -> Vec<&Sampler> {
    set.materials
        .iter()
        .flat_map(|material| &material.entries)
        .filter_map(|entry| match entry {
            MaterialEntry::Sampler(sampler) if sampler.name == "EnvironmentMap" => Some(sampler),
            MaterialEntry::Sampler(_) | MaterialEntry::State(_) | MaterialEntry::Vector(_) => None,
        })
        .collect()
}

/// `clean_model()` (the tracer's right glove) with every material's shader set to
/// `fox3ddf_ggx`, the Fox shader of the `metal` family, naming no environment texture.
fn metal_model() -> Vec<u8> {
    let mut model = fmdl::Model::from_file(&fmdl::FmdlFile::read(&clean_model()).unwrap()).unwrap();
    for material in &mut model.materials {
        "fox3ddf_ggx".clone_into(&mut material.shader);
    }
    model.to_file().unwrap().write()
}

/// The bundled template environment map, `resources/templates/env.dds`.
fn environment_template() -> Vec<u8> {
    fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/templates/env.dds"))
        .unwrap()
}

/// Slot 05's texture home on PES 15-17, as a `.mtl` names it.
const PRE_FOX_HOME_714_05: &str = "model/character/uniform/common/714/05 - A/";

/// The CPK path of the environment map in slot 05's texture home on PES 15-17.
const ENVIRONMENT_714_05: &str =
    "common/character1/model/character/uniform/common/714/05 - A/env.dds";

/// Writes slot 05 of the export `export` holding `boots.fmdl` made of `metal_model()`, and
/// gives `compiled_metal` of its face's `boots.mtl`.
fn compiled_metal_boots(sandbox: &Sandbox, export: &str) -> (Vec<String>, MaterialSet, Vec<u8>) {
    sandbox.write(&format!("{}/boots.fmdl", slot_05(export)), &metal_model());
    compiled_metal(sandbox, "boots.mtl")
}

/// Compiles the sandbox's exports for PES 17 with `dds_compression` on, and gives the run's
/// lines, the `.mtl` named `mtl` of slot 05's face as `pes_model` reads it, and the bytes of
/// the CPK's `env.dds` in the slot's texture home, unwrapped: it is zlibbed as every DDS the
/// run emits. The run must exit 0.
fn compiled_metal(sandbox: &Sandbox, mtl: &str) -> (Vec<String>, MaterialSet, Vec<u8>) {
    let settings = format!(
        "{}[team-compiler]\ndds_compression = true\n",
        pes17(sandbox)
    );
    let run = sandbox.run(&settings, &["compile", "--no-deploy"]);
    let lines = run.messages();
    assert_eq!(run.exit_code(), 0, "{lines:#?}");
    let entries = cpk_entries(&sandbox.root.join("output/4cc_99_test.cpk"));
    let face = nested_entries(&entries[&face_cpk(5)]);
    let materials = MaterialSet::read(&face[&format!("{}{mtl}", face_folder(5))]).unwrap();
    let environment = entries
        .get(ENVIRONMENT_714_05)
        .unwrap_or_else(|| panic!("no {ENVIRONMENT_714_05}: {:?}", entries.keys()));
    let environment = wezlib::decompress(environment).unwrap();
    (lines, materials, environment)
}

/// Asserts that each of `materials` is the `metal` family's on PES 15-17: `Basic_CNSR`, its
/// samplers ending with the environment sampler naming `env.dds` in slot 05's texture home,
/// with the `environment` role's settings, and the `Reflection` and `Shininess` vectors.
fn assert_metal(materials: &[Material]) {
    let expected = Sampler {
        name: "EnvironmentMap".to_owned(),
        path: format!("{PRE_FOX_HOME_714_05}env.dds"),
        srgb: Some(false),
        minfilter: Some(Filter::Anisotropic),
        maxfilter: None,
        magfilter: Some(Filter::Linear),
        mipfilter: None,
        uaddr: Some(Address::Wrap),
        vaddr: Some(Address::Wrap),
        waddr: Some(Address::Wrap),
        maxaniso: Some(2),
    };
    assert!(!materials.is_empty());
    for material in materials {
        assert_eq!(material.shader, "Basic_CNSR", "{}", material.name);
        let samplers: Vec<&Sampler> = material
            .entries
            .iter()
            .filter_map(|entry| match entry {
                MaterialEntry::Sampler(sampler) => Some(sampler),
                MaterialEntry::State(_) | MaterialEntry::Vector(_) => None,
            })
            .collect();
        assert_eq!(samplers.last(), Some(&&expected), "{}", material.name);
        assert_eq!(
            samplers
                .iter()
                .filter(|sampler| sampler.name == "EnvironmentMap")
                .count(),
            1,
            "{}",
            material.name
        );
        // The samplers come first, as the converter writes them, the added one with them.
        let first_other = material
            .entries
            .iter()
            .position(|entry| !matches!(entry, MaterialEntry::Sampler(_)));
        assert_eq!(first_other, Some(samplers.len()), "{}", material.name);
        let vectors: Vec<&Vector> = material
            .entries
            .iter()
            .filter_map(|entry| match entry {
                MaterialEntry::Vector(vector) => Some(vector),
                MaterialEntry::Sampler(_) | MaterialEntry::State(_) => None,
            })
            .collect();
        assert_eq!(
            vectors,
            [
                &Vector {
                    name: "Reflection".to_owned(),
                    components: vec![1.0, 1.0, 1.0, 0.0],
                },
                &Vector {
                    name: "Shininess".to_owned(),
                    components: vec![0.9, 0.0, 0.0, 1.0],
                },
            ],
            "{}",
            material.name
        );
    }
}

// TC-MOD-36
#[test]
fn a_fox_metal_material_compiled_for_pes_17_reflects_the_template_environment_map() {
    let sandbox = Sandbox::new("conversion_metal_template");
    let export = "co Midcup Metal";

    let (lines, materials, environment) = compiled_metal_boots(&sandbox, export);

    assert!(
        lines
            .iter()
            .all(|line| !line.contains("template_override_active")),
        "{lines:#?}"
    );
    assert_metal(&materials.materials);
    assert!(
        environment == environment_template(),
        "env.dds is the bundled template"
    );
}

#[test]
fn a_linked_face_folder_s_metal_material_reflects_the_environment_map_in_the_player_s_home() {
    let sandbox = Sandbox::new("conversion_metal_shared_face");
    write_round_hat(&sandbox, "co Midcup Metal", &metal_model());

    let (lines, materials, environment) = compiled_metal(&sandbox, "hat.mtl");

    assert!(
        lines
            .iter()
            .all(|line| !line.contains("content_not_yet_compiled")),
        "{lines:#?}"
    );
    assert_metal(&materials.materials);
    assert!(
        environment == environment_template(),
        "env.dds is the bundled template"
    );
}

#[test]
fn a_member_s_own_env_dds_is_the_environment_map_a_metal_material_names() {
    let sandbox = Sandbox::new("conversion_metal_own");
    let export = "co Midcup Metal";
    // Already WESYS-wrapped, so it is emitted as it is, not re-encoded, and its bytes tell it
    // from any other.
    sandbox.write(
        &format!("{}/env.dds", slot_05(export)),
        &wezlib::compress(&small_dds()),
    );

    let (_, materials, environment) = compiled_metal_boots(&sandbox, export);

    assert_metal(&materials.materials);
    assert!(environment == small_dds(), "env.dds is the member's");
}

#[test]
fn a_templates_env_dds_replaces_the_bundled_environment_map() {
    let sandbox = Sandbox::new("conversion_metal_override");
    let export = "co Midcup Metal";
    sandbox.write("data/templates/env.dds", &small_dds());

    let (lines, materials, environment) = compiled_metal_boots(&sandbox, export);

    let active = format!(
        "Info template_override_active [Keep] (path={})",
        sandbox.display("data/templates/env.dds")
    );
    assert!(lines.contains(&active), "{lines:#?}");
    assert_metal(&materials.materials);
    assert!(environment == small_dds(), "env.dds is the override");
}
