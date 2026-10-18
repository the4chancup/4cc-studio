//! The kit folders' migration into `Kits/` (the plan's "Kits" scenarios):
//! every old binary config becomes its slot's `config.toml`, every kit
//! texture lands under the generic `kit` name, and what cannot be placed is
//! unmigrated content.

use std::fs;

use crate::common::Sandbox;

/// `upgrade <input> --savefile <save>`.
fn upgrade(sandbox: &Sandbox, input: &str, save: &str) -> crate::common::Run {
    let input = sandbox.arg(input);
    let save = sandbox.arg(save);
    sandbox.run(&["upgrade", &input, "--savefile", &save])
}

/// The `4CCG VGL26` fixture's path below `tests/fixtures/`.
fn kit_config(ordinal: &str) -> String {
    format!("kit_configs/4CCG VGL26/XXX_DEF_{ordinal}_realUni.bin")
}

/// `bytes` decoded for PES 21 and converted to the `config.toml` text the
/// upgrader writes (texture names dropped).
fn expected_toml(bytes: &[u8]) -> String {
    let mut config = kit_config::KitConfig::decode(bytes, pes_version::PesVersion::Pes21).unwrap();
    config.source_texture_names = None;
    config.to_toml()
}

// EU-KIT-01
#[test]
fn kits_become_slot_folders_with_config_toml_and_generic_textures() {
    let sandbox = Sandbox::new("kit_basic");
    for (ordinal, name) in [
        ("1st", "XXX_DEF_1st_realUni.bin"),
        ("2nd", "714_def_2nd_realuni.bin"),
        ("GK1st", "XXX_DEF_GK1st_realUni.bin"),
    ] {
        sandbox.copy_fixture(&kit_config(ordinal), &format!("co Kits/Kit Configs/{name}"));
    }
    for (name, bytes) in [
        ("u0XXXp1.dds", b"p1 main" as &[u8]),
        ("u0XXXp1_back.dds", b"p1 back"),
        ("u0714p2.dds", b"p2 main"),
        ("u0XXXg1.dds", b"g1 main"),
    ] {
        sandbox.write(&format!("co Kits/Kit Textures/{name}"), bytes);
    }
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    assert_eq!(
        sandbox.listing("co Full Kits/Kits"),
        [
            "g1",
            "g1/config.toml",
            "g1/kit.dds",
            "p1",
            "p1/config.toml",
            "p1/kit.dds",
            "p1/kit_back.dds",
            "p2",
            "p2/config.toml",
            "p2/kit.dds",
        ]
    );
    // The exact listing already rules a `.bin`, a `pre-fox`/`fox` marker or a
    // `Kit Configs`/`Kit Textures` folder inside `Kits/`; the output's whole
    // listing rules them anywhere.
    let output = sandbox.listing("co Full Kits");
    for entry in &output {
        assert!(!entry.ends_with(".bin"), "{entry}");
        assert!(!entry.starts_with("Kit Configs"), "{entry}");
        assert!(!entry.starts_with("Kit Textures"), "{entry}");
    }
    for (path, bytes) in [
        ("Kits/p1/kit.dds", b"p1 main" as &[u8]),
        ("Kits/p1/kit_back.dds", b"p1 back"),
        ("Kits/p2/kit.dds", b"p2 main"),
        ("Kits/g1/kit.dds", b"g1 main"),
    ] {
        assert_eq!(
            fs::read(sandbox.path(&format!("co Full Kits/{path}"))).unwrap(),
            bytes,
            "{path}"
        );
    }
}

// EU-KIT-02
#[test]
fn a_config_toml_carries_the_undecoded_bits_and_no_texture_names() {
    let sandbox = Sandbox::new("kit_unknown_bits");
    sandbox.copy_fixture(
        &kit_config("1st"),
        "co Kits/Kit Configs/XXX_DEF_1st_realUni.bin",
    );
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    let text = fs::read_to_string(sandbox.path("co Full Kits/Kits/p1/config.toml")).unwrap();
    let parsed: toml::Table = toml::from_str(&text).unwrap();
    assert!(!parsed.contains_key("source_texture_names"), "{text}");
    // The fixture's set undecoded bit survives: offset 0x17 = 4.
    assert_eq!(parsed["unknown"]["0x17"], toml::Value::Integer(4), "{text}");
    // And every other bit is kept too: the TOML parses back to the decoded
    // config with only the texture names dropped.
    let fixture = include_bytes!("../fixtures/kit_configs/4CCG VGL26/XXX_DEF_1st_realUni.bin");
    let mut expected =
        kit_config::KitConfig::decode(fixture, pes_version::PesVersion::Pes21).unwrap();
    expected.source_texture_names = None;
    assert_eq!(kit_config::KitConfig::from_toml(&text).unwrap(), expected);
}

// EU-KIT-08
#[test]
fn a_texture_of_a_slot_no_config_gives_is_unmigrated_content() {
    let sandbox = Sandbox::new("kit_texture_orphan");
    for ordinal in ["1st", "2nd", "3rd"] {
        sandbox.copy_fixture(
            &kit_config(ordinal),
            &format!("co Kits/Kit Configs/XXX_DEF_{ordinal}_realUni.bin"),
        );
    }
    sandbox.copy_fixture("co Spring/co Note.txt", "co Kits/co Note.txt");
    sandbox.write("co Kits/Kit Textures/u0XXXp5.dds", b"p5 main");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    assert!(
        run.stdout()
            .contains("unmigrated_content at Kit Textures/u0XXXp5.dds"),
        "{}",
        run.stdout()
    );
    assert!(!sandbox.path("co Full Kits/Kits/p5").exists());
    // The three configs still migrated; only the texture is unmigrated.
    assert!(sandbox.path("co Full Kits/Kits/p1/config.toml").exists());
}

// EU-KIT-10
#[test]
fn configs_in_a_kit_configs_subfolder_migrate_but_a_subfolder_texture_does_not() {
    let sandbox = Sandbox::new("kit_config_subfolder");
    for ordinal in ["1st", "2nd"] {
        sandbox.copy_fixture(
            &kit_config(ordinal),
            &format!("co Kits/Kit Configs/714/XXX_DEF_{ordinal}_realUni.bin"),
        );
    }
    sandbox.write("co Kits/Kit Textures/714/u0XXXp1.dds", b"p1 main");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    let first = include_bytes!("../fixtures/kit_configs/4CCG VGL26/XXX_DEF_1st_realUni.bin");
    let second = include_bytes!("../fixtures/kit_configs/4CCG VGL26/XXX_DEF_2nd_realUni.bin");
    for (slot, bytes) in [("p1", &first[..]), ("p2", &second[..])] {
        assert_eq!(
            fs::read_to_string(sandbox.path(&format!("co Full Kits/Kits/{slot}/config.toml")))
                .unwrap(),
            expected_toml(bytes),
            "{slot}"
        );
    }
    assert!(
        run.stdout()
            .contains("unmigrated_content at Kit Textures/714/u0XXXp1.dds"),
        "{}",
        run.stdout()
    );
}

// EU-KIT-12
#[test]
fn a_config_whose_suffix_reads_no_slot_is_unmigrated_content() {
    let sandbox = Sandbox::new("kit_bad_suffix");
    for ordinal in ["1st", "2nd"] {
        sandbox.copy_fixture(
            &kit_config(ordinal),
            &format!("co Kits/Kit Configs/XXX_DEF_{ordinal}_realUni.bin"),
        );
    }
    // GK1st's bytes under a name the grammar does not read.
    sandbox.copy_fixture(
        &kit_config("GK1st"),
        "co Kits/Kit Configs/XXX_DEF_GK2nd_realUni.bin",
    );
    // A Note with two player kit entries: the fixture's `- 3rd player:` line
    // removed.
    sandbox.copy_fixture("co Spring/co Note.txt", "co Kits/co Note.txt");
    let note_path = sandbox.path("co Kits/co Note.txt");
    let text = fs::read_to_string(&note_path).unwrap();
    let edited = text
        .lines()
        .filter(|line| !line.contains("3rd player"))
        .collect::<Vec<_>>()
        .join("\r\n");
    fs::write(&note_path, edited).unwrap();
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Kits", "EDIT");
    assert_eq!(run.exit_code(), 1, "{}", run.stdout());
    for slot in ["p1", "p2"] {
        assert!(
            sandbox
                .path(&format!("co Full Kits/Kits/{slot}/config.toml"))
                .exists(),
            "{slot}"
        );
    }
    assert!(
        run.stdout()
            .contains("unmigrated_content at Kit Configs/XXX_DEF_GK2nd_realUni.bin"),
        "{}",
        run.stdout()
    );
}
