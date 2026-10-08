//! `compile` for PES 2017: a per-kit model set (`pants_kit1.model`, `pants_kit2.model`) in a
//! player folder, every variant packed into his face CPK under its own name and the set listed
//! once in the generated `face.xml`, its kit token spelled `kitN` in the entry's path and
//! material, for the game to respell for the kit picked.

use std::collections::BTreeMap;

use crate::common::Sandbox;
use crate::prefox_faces::{
    compile_pes17, face_cpk, face_folder, materials_naming, nested_entries, ordered_entries,
    pre_fox_fixture, sampler_paths, small_dds,
};

/// The export every test here compiles.
const EXPORT: &str = "co Midcup Kit Models";

/// Slot 05's folder in `EXPORT`.
const PLAYER: &str = "exports/co Midcup Kit Models/Players/05 - A";

/// The findings of `EXPORT` compiled with nothing wrong in its set: the face has no
/// `face_neck` model of its own, the variants being `parts`.
const FINDINGS: [&str; 3] = [
    "Info export_identified [Keep] (team=/co/, id=714)",
    "Info team_colors_missing [Keep] ()",
    "Info xml_face_neck_added [Keep] at Players/05 - A ()",
];

/// The folder slot 05's textures are pointed at, as a `.mtl` names it.
const TEXTURE_HOME: &str = "model/character/uniform/common/714/05 - A/";

/// Writes slot 05's two variant models, two different real models, and `pants_kit1.dds`.
fn write_variants(sandbox: &Sandbox) {
    sandbox.write(
        &format!("{PLAYER}/pants_kit1.model"),
        &pre_fox_fixture("cardhead_face_high.model"),
    );
    sandbox.write(
        &format!("{PLAYER}/pants_kit2.model"),
        &pre_fox_fixture("cardhead_doublesided_face_high.model"),
    );
    sandbox.write(&format!("{PLAYER}/pants_kit1.dds"), &small_dds());
}

/// Slot 05's face CPK in the output CPK `entries`, by each entry's name in its folder.
fn face_files(entries: &BTreeMap<String, Vec<u8>>) -> BTreeMap<String, Vec<u8>> {
    let folder = face_folder(5);
    nested_entries(&entries[&face_cpk(5)])
        .into_iter()
        .map(|(path, bytes)| {
            (
                path.strip_prefix(folder.as_str()).unwrap().to_owned(),
                bytes,
            )
        })
        .collect()
}

/// A `face.xml` entry: type, path and material.
fn entry(xml_type: &str, path: &str, material: &str) -> (String, String, String) {
    (xml_type.to_owned(), path.to_owned(), material.to_owned())
}

// TC-CMN-07
#[test]
fn a_per_kit_model_set_is_packed_whole_and_listed_once_as_its_reference() {
    let sandbox = Sandbox::new("prefox_kit_variants");
    write_variants(&sandbox);
    for mtl in ["pants_kit1.mtl", "pants_kit2.mtl"] {
        sandbox.write(&format!("{PLAYER}/{mtl}"), &materials_naming("pants_kitN"));
    }

    let entries = compile_pes17(&sandbox, EXPORT, &FINDINGS);

    assert!(
        entries.contains_key(&format!("common/character1/{TEXTURE_HOME}pants_kit1.dds")),
        "{:?}",
        entries.keys()
    );
    let face = face_files(&entries);
    assert_eq!(
        face.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "dummy.mtl",
            "face.xml",
            "oral_dummy_win32.model",
            "oral_pants_kit1_win32.model",
            "oral_pants_kit2_win32.model",
            "pants_kit1.mtl",
            "pants_kit2.mtl",
        ]
    );
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [
            entry("parts", "./oral_pants_kitN_*.model", "./pants_kitN.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
    assert_eq!(
        face["oral_pants_kit1_win32.model"],
        pre_fox_fixture("cardhead_face_high.model")
    );
    assert_eq!(
        face["oral_pants_kit2_win32.model"],
        pre_fox_fixture("cardhead_doublesided_face_high.model")
    );
    // The reference names no file: it is pointed at the folder holding its variants, its
    // name kept for the game to respell.
    for mtl in ["pants_kit1.mtl", "pants_kit2.mtl"] {
        assert_eq!(
            sampler_paths(&face[mtl]),
            [format!("{TEXTURE_HOME}pants_kitN.dds")],
            "{mtl}"
        );
    }
}

#[test]
fn a_variant_whose_mtl_is_not_the_one_its_entry_implies_is_named() {
    let sandbox = Sandbox::new("prefox_kit_variants_mtl_differs");
    write_variants(&sandbox);
    sandbox.write(
        &format!("{PLAYER}/pants_kit1.mtl"),
        &materials_naming("pants_kitN"),
    );
    // `pants_kit2`'s search finds `materials.mtl`, where the game, respelling the entry's
    // `pants_kitN.mtl`, looks for `pants_kit2.mtl`.
    sandbox.write(
        &format!("{PLAYER}/materials.mtl"),
        &materials_naming("pants_kitN"),
    );

    let entries = compile_pes17(
        &sandbox,
        EXPORT,
        &[
            FINDINGS[0],
            FINDINGS[1],
            "Warning kit_variant_mtl_differs [Keep] at Players/05 - A (model=pants_kit2.model, mtl=./materials.mtl, expected=./pants_kit2.mtl)",
            FINDINGS[2],
        ],
    );

    let face = face_files(&entries);
    assert!(face.contains_key("materials.mtl"), "{:?}", face.keys());
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [
            entry("parts", "./oral_pants_kitN_*.model", "./pants_kitN.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
}

#[test]
fn a_variant_whose_mtl_is_in_another_directory_than_its_entry_implies_is_named() {
    let sandbox = Sandbox::new("prefox_kit_variants_mtl_directory");
    write_variants(&sandbox);
    // `pants_kit1`'s search finds the Common `.mtl` its link names; `pants_kit2`'s finds the
    // face's own `pants_kit2.mtl`, while the game, respelling the entry, looks in the Common
    // output.
    sandbox.write(&format!("{PLAYER}/pants_kit1.mtl.common"), b"");
    sandbox.write(
        "exports/co Midcup Kit Models/Common/pants_kit1.mtl",
        &materials_naming("pants_kitN"),
    );
    sandbox.write(
        &format!("{PLAYER}/pants_kit2.mtl"),
        &materials_naming("pants_kitN"),
    );

    let entries = compile_pes17(
        &sandbox,
        EXPORT,
        &[
            // A `Common/` `.mtl`'s paths resolve in `Common/`, which holds no `pants_kit`
            // texture, and its mesh-used materials are the `Common/` models' (none here).
            "Info mtl_texture_unused_missing [Keep] at Common/pants_kit1.mtl (file=pants_kit1.mtl, texture=./pants_kitN.dds, materials=card)",
            FINDINGS[0],
            FINDINGS[1],
            "Warning kit_variant_mtl_differs [Keep] at Players/05 - A (model=pants_kit2.model, mtl=./pants_kit2.mtl, expected=model/character/uniform/common/714/pants_kit2.mtl)",
            FINDINGS[2],
        ],
    );

    let face = face_files(&entries);
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [
            entry(
                "parts",
                "./oral_pants_kitN_*.model",
                "model/character/uniform/common/714/pants_kitN.mtl"
            ),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
}

/// The shared face folder slot 05 links in the tests of a set split between it and his own
/// files.
const LONGHAIR: &str = "exports/co Midcup Kit Models/Faces/Longhair";

/// Writes `Faces/Longhair` holding `pants_kit1.model` and `pants_kit2.model` (both the
/// single-sided card head) with one `pants.mtl`, and slot 05's link to it.
fn write_linked_set(sandbox: &Sandbox) {
    sandbox.write(&format!("{PLAYER}/Longhair.face"), b"");
    for name in ["pants_kit1.model", "pants_kit2.model"] {
        sandbox.write(
            &format!("{LONGHAIR}/{name}"),
            &pre_fox_fixture("cardhead_face_high.model"),
        );
    }
    sandbox.write(&format!("{LONGHAIR}/pants.mtl"), &materials_naming("skin"));
    sandbox.write(&format!("{LONGHAIR}/skin.dds"), &small_dds());
}

/// The findings of `EXPORT` compiled with slot 05 linking `Faces/Longhair` and nothing wrong.
const LINKED_FINDINGS: [&str; 4] = [
    FINDINGS[0],
    FINDINGS[1],
    "Info link_combined [Keep] at Players/05 - A (link=Longhair.face)",
    FINDINGS[2],
];

/// Compiles `write_linked_set` plus slot 05's own variant `own` (`pants_kit<own>.model`, the
/// double-sided card head) with his own `pants.mtl`, and asserts the face lists one set of
/// both folders' variants: his own `own`, and the shared other.
fn assert_one_set_with_own_variant(own: u8) {
    let sandbox = Sandbox::new(&format!("prefox_kit_variants_split_{own}"));
    write_linked_set(&sandbox);
    sandbox.write(
        &format!("{PLAYER}/pants_kit{own}.model"),
        &pre_fox_fixture("cardhead_doublesided_face_high.model"),
    );
    sandbox.write(&format!("{PLAYER}/pants.mtl"), &materials_naming("skin"));
    sandbox.write(&format!("{PLAYER}/skin.dds"), &small_dds());

    let entries = compile_pes17(&sandbox, EXPORT, &LINKED_FINDINGS);

    let face = face_files(&entries);
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [
            entry("parts", "./oral_pants_kitN_*.model", "./pants.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
    let shared = 3 - own;
    assert_eq!(
        face[&format!("oral_pants_kit{own}_win32.model")],
        pre_fox_fixture("cardhead_doublesided_face_high.model")
    );
    assert_eq!(
        face[&format!("oral_pants_kit{shared}_win32.model")],
        pre_fox_fixture("cardhead_face_high.model")
    );
}

// The player's own variant replaces the shared one of its name, as a copy would, and the face
// holds one set of both, whichever variant is his.
#[test]
fn a_set_split_between_a_linked_face_and_the_player_s_files_is_one_set() {
    assert_one_set_with_own_variant(1);
}

#[test]
fn a_set_split_with_the_player_s_own_higher_variant_is_one_set() {
    assert_one_set_with_own_variant(2);
}

#[test]
fn a_player_s_own_variant_of_a_linked_set_is_checked_against_the_shared_entry() {
    let sandbox = Sandbox::new("prefox_kit_variants_split_mtl_differs");
    write_linked_set(&sandbox);
    // The shared `pants_kit1` lists the set with the shared `pants.mtl`; the player's own
    // `pants_kit2` finds his `materials.mtl`.
    sandbox.write(
        &format!("{PLAYER}/pants_kit2.model"),
        &pre_fox_fixture("cardhead_doublesided_face_high.model"),
    );
    sandbox.write(
        &format!("{PLAYER}/materials.mtl"),
        &materials_naming("skin"),
    );

    compile_pes17(
        &sandbox,
        EXPORT,
        &[
            LINKED_FINDINGS[0],
            LINKED_FINDINGS[1],
            LINKED_FINDINGS[2],
            "Warning kit_variant_mtl_differs [Keep] at Players/05 - A (model=pants_kit2.model, mtl=./materials.mtl, expected=./pants.mtl)",
            LINKED_FINDINGS[3],
        ],
    );
}

#[test]
fn the_lowest_variant_lists_a_linked_set_wherever_it_sorts() {
    let sandbox = Sandbox::new("prefox_kit_variants_split_lowest");
    write_linked_set(&sandbox);
    // The shared `pants_kit2` sorts first (`Faces/` before `Players/`), yet the player's own
    // `pants_kit1` lists the set, with his `pants_kit1.mtl` respelled.
    sandbox.write(
        &format!("{PLAYER}/pants_kit1.model"),
        &pre_fox_fixture("cardhead_doublesided_face_high.model"),
    );
    sandbox.write(
        &format!("{PLAYER}/pants_kit1.mtl"),
        &materials_naming("skin"),
    );

    let entries = compile_pes17(
        &sandbox,
        EXPORT,
        &[
            LINKED_FINDINGS[0],
            LINKED_FINDINGS[1],
            LINKED_FINDINGS[2],
            "Warning kit_variant_mtl_differs [Keep] at Players/05 - A (model=pants_kit2.model, mtl=./pants.mtl, expected=./pants_kit2.mtl)",
            LINKED_FINDINGS[3],
        ],
    );

    assert_eq!(
        ordered_entries(&face_files(&entries)["face.xml"]),
        [
            entry("parts", "./oral_pants_kitN_*.model", "./pants_kitN.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
}

#[test]
fn a_set_sharing_one_mtl_names_it_as_it_is() {
    let sandbox = Sandbox::new("prefox_kit_variants_one_mtl");
    write_variants(&sandbox);
    sandbox.write(
        &format!("{PLAYER}/pants.mtl"),
        &materials_naming("pants_kitN"),
    );

    let entries = compile_pes17(&sandbox, EXPORT, &FINDINGS);

    let face = face_files(&entries);
    assert_eq!(
        ordered_entries(&face["face.xml"]),
        [
            entry("parts", "./oral_pants_kitN_*.model", "./pants.mtl"),
            entry("face_neck", "./oral_dummy_*.model", "./dummy.mtl"),
        ]
    );
}
