//! The carried folders `Portraits/`, `Collars/` and `Logo/` (the plan's
//! "Portraits, logo, models and Common" scenarios): the portrait rename
//! and its face-folder conflict check, collars as they are, and the logo
//! members' root files and their downscale-or-drawing test.

use std::fs;

use crate::common::Sandbox;

/// `upgrade <input> --savefile <save>`.
fn upgrade(sandbox: &Sandbox, input: &str, save: &str) -> crate::common::Run {
    let input = sandbox.arg(input);
    let save = sandbox.arg(save);
    sandbox.run(&["upgrade", &input, "--savefile", &save])
}

/// The fixture file `relative` below `tests/fixtures/logos/`.
fn logo_fixture(relative: &str) -> Vec<u8> {
    fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/logos")
            .join(relative),
    )
    .unwrap()
}

// EU-MDL-01
#[test]
fn portraits_renamed_and_collars_carried() {
    let sandbox = Sandbox::new("mdl_portraits_collars");
    for (name, bytes) in [
        ("player_XXX05.dds", b"p05" as &[u8]),
        ("player_71407.dds", b"p07"),
        ("player_09.dds", b"p09"),
    ] {
        sandbox.write(&format!("co Kits/Portraits/{name}"), bytes);
    }
    sandbox.write("co Kits/Collars/collar_120.fmdl", b"collar");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    for (to, bytes) in [
        ("player_05.dds", b"p05" as &[u8]),
        ("player_07.dds", b"p07"),
        ("player_09.dds", b"p09"),
    ] {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Kits/Portraits/{to}"))).unwrap(),
            bytes,
            "{to}"
        );
    }
    assert_eq!(
        fs::read(sandbox.path("co Full Kits/Collars/collar_120.fmdl")).unwrap(),
        b"collar"
    );
    assert!(
        !run.stdout().contains("unmigrated_content"),
        "{}",
        run.stdout()
    );
}

// EU-MDL-02
#[test]
fn a_downscale_small_logo_is_dropped_with_its_share() {
    let sandbox = Sandbox::new("mdl_logo_downscale");
    sandbox.copy_fixture_dir("logos/OMG VGL26", "co Kits/Logo");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("co Full Kits/logo.png")).unwrap(),
        logo_fixture("OMG VGL26/emblem_0XXX_r_ll.png")
    );
    assert!(!sandbox.path("co Full Kits/logo_small.png").exists());
    // `_r_l` is dropped with no finding: no output file carries its bytes.
    let medium = logo_fixture("OMG VGL26/emblem_0XXX_r_l.png");
    for entry in sandbox.listing("co Full Kits") {
        let path = sandbox.path(&format!("co Full Kits/{entry}"));
        if path.is_file() {
            assert_ne!(fs::read(&path).unwrap(), medium, "{entry}");
        }
    }
    assert!(
        run.stdout()
            .contains("logo_small_dropped at Logo/emblem_0XXX_r.png (share=30%)"),
        "{}",
        run.stdout()
    );
}

// EU-MDL-03
#[test]
fn a_distinct_small_logo_is_carried() {
    let sandbox = Sandbox::new("mdl_logo_variant");
    sandbox.copy_fixture_dir("logos/VRG VGL26", "co Kits/Logo");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("co Full Kits/logo.png")).unwrap(),
        logo_fixture("VRG VGL26/emblem_0XXX_r_ll.png")
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Kits/logo_small.png")).unwrap(),
        logo_fixture("VRG VGL26/emblem_0XXX_r.png")
    );
    assert!(
        run.stdout()
            .contains("logo_small_carried at Logo/emblem_0XXX_r.png (share=53%)"),
        "{}",
        run.stdout()
    );
}

// EU-MDL-04
#[test]
fn the_fallback_medium_member_becomes_the_logo() {
    let sandbox = Sandbox::new("mdl_logo_fallback");
    for name in ["emblem_0XXX_r_l.png", "emblem_0XXX_r.png"] {
        sandbox.write(
            &format!("co Kits/Logo/{name}"),
            &logo_fixture(&format!("VRG VGL26/{name}")),
        );
    }
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("co Full Kits/logo.png")).unwrap(),
        logo_fixture("VRG VGL26/emblem_0XXX_r_l.png")
    );
    assert!(!sandbox.path("co Full Kits/logo_small.png").exists());
    assert!(
        run.stdout()
            .contains("logo_fallback at Logo/emblem_0XXX_r_l.png"),
        "{}",
        run.stdout()
    );
}

// EU-MDL-15
#[test]
fn a_portrait_of_other_bytes_in_the_face_folder_is_a_conflict() {
    let sandbox = Sandbox::new("mdl_portrait_conflict");
    sandbox.write("co Spring/Faces/XXX07 - A/face.fmdl", b"face");
    sandbox.write("co Spring/Faces/XXX07 - A/portrait.dds", b"face portrait");
    sandbox.write("co Spring/Portraits/player_XXX07.dds", b"other portrait");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Portraits/player_07.dds")).unwrap(),
        b"other portrait"
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/07 - A/portrait.dds")).unwrap(),
        b"face portrait"
    );
    assert!(
        run.stdout()
            .contains("portrait_conflict at Portraits/player_XXX07.dds (slot=07)"),
        "{}",
        run.stdout()
    );
}

/// A `.mtl` of one material holding one sampler per `paths` entry.
fn mtl(paths: &[&str]) -> Vec<u8> {
    let samplers: String = paths
        .iter()
        .map(|path| {
            format!("        <sampler name=\"DiffuseMap\" path=\"{path}\" srgb=\"1\" minfilter=\"linear\" magfilter=\"linear\" uaddr=\"clamp\" vaddr=\"clamp\"/>\r\n")
        })
        .collect();
    format!(
        "<materialset>\r\n    <material name=\"face_phong\" shader=\"Wrinkle_Face\">\r\n{samplers}    </material>\r\n</materialset>\r\n"
    )
    .into_bytes()
}

/// The one `.mtl` output's text.
fn mtl_text(sandbox: &Sandbox, output: &str) -> String {
    fs::read_to_string(sandbox.path(output)).unwrap()
}

// EU-MDL-05
#[test]
fn common_references_become_the_bare_name_and_the_link() {
    let sandbox = Sandbox::new("mdl_common_mtl");
    sandbox.write("co Spring/Faces/XXX07 - A/face_high_win32.model", b"face");
    sandbox.write("co Spring/Faces/XXX07 - A/face_srm.dds", b"srm");
    sandbox.write(
        "co Spring/Faces/XXX07 - A/face.mtl",
        &mtl(&[
            "./face_srm.dds",
            "model/character/uniform/common/XXX/hair.dds",
            "model/character/boots/k0002/ek0062_n.dds",
        ]),
    );
    sandbox.write(
        "co Spring/Faces/XXX07 - A/realkit/apc.mtl",
        &mtl(&["model/character/uniform/common/XXX/hair.dds"]),
    );
    sandbox.write("co Spring/Common/hair.dds", b"hair");
    fs::write(
        sandbox.path("EDIT16"),
        pes_savefile::fixture_save(pes_version::PesVersion::Pes16),
    )
    .unwrap();

    let run = upgrade(&sandbox, "co Spring", "EDIT16");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    // The face folder's own names stay; the Common reference becomes the
    // bare name plus the link; the other game path is kept as written.
    let face = mtl_text(&sandbox, "co Full Spring/Players/07 - A/face.mtl");
    for expected in [
        "./face_srm.dds",
        "./hair.dds",
        "model/character/boots/k0002/ek0062_n.dds",
    ] {
        assert!(face.contains(expected), "{expected}: {face}");
    }
    let apc = mtl_text(&sandbox, "co Full Spring/Players/07 - A/realkit/apc.mtl");
    assert!(apc.contains("./hair.dds"), "{apc}");
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Players/07 - A/hair.dds.common")).unwrap(),
        b""
    );
    assert!(
        !sandbox
            .path("co Full Spring/Players/07 - A/realkit/hair.dds.common")
            .exists()
    );
    assert_eq!(
        fs::read(sandbox.path("co Full Spring/Common/hair.dds")).unwrap(),
        b"hair"
    );
}

// EU-MDL-06
#[test]
fn a_reference_whose_common_file_is_missing_is_kept_and_warned() {
    let sandbox = Sandbox::new("mdl_common_missing");
    sandbox.write("co Spring/Faces/XXX07 - A/face_high_win32.model", b"face");
    sandbox.write(
        "co Spring/Faces/XXX07 - A/face.mtl",
        &mtl(&["model/character/uniform/common/XXX/hair.dds"]),
    );
    fs::write(
        sandbox.path("EDIT16"),
        pes_savefile::fixture_save(pes_version::PesVersion::Pes16),
    )
    .unwrap();

    let run = upgrade(&sandbox, "co Spring", "EDIT16");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    let face = mtl_text(&sandbox, "co Full Spring/Players/07 - A/face.mtl");
    assert!(
        face.contains("model/character/uniform/common/714/hair.dds"),
        "{face}"
    );
    let stdout = run.stdout();
    assert!(
        stdout.contains(
            "common_reference_kept at Faces/XXX07 - A/face.mtl (reference=model/character/uniform/common/XXX/hair.dds)"
        ),
        "{stdout}"
    );
}

// EU-REP-02
#[test]
fn a_portrait_named_for_a_slot_past_23_is_flagged() {
    let sandbox = Sandbox::new("rep_portrait_24");
    sandbox.write("co Kits/Portraits/player_XXX24.dds", b"portrait");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    assert_eq!(
        fs::read(sandbox.path("co Full Kits/Portraits/player_24.dds")).unwrap(),
        b"portrait"
    );
    let stdout = run.stdout();
    assert!(
        stdout.contains("portrait_name_invalid at Portraits/player_24.dds"),
        "{stdout}"
    );
    assert!(stdout.contains("migration_not_clean"), "{stdout}");
}
