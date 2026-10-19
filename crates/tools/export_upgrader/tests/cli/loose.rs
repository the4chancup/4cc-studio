//! Loose mode (the plan's step 11, the EU-LOOSE scenarios): no savefile, so
//! each item folder's wearer is the player at its ID's position in the team's
//! old block of 25 IDs, the block its kind's lowest ID falls in, and each
//! assumed pairing is reported.

use std::fs;

use pes_savefile::settings_toml::{NameSetting, PlayerSettings};

use crate::common::Sandbox;
use crate::players::{boots, face};

/// `upgrade <input> --loose`.
fn upgrade_loose(sandbox: &Sandbox, input: &str) -> crate::common::Run {
    let input = sandbox.arg(input);
    sandbox.run(&["upgrade", &input, "--loose"])
}

// EU-LOOSE-01
#[test]
fn the_ids_position_in_the_block_pairs_the_wearer() {
    let sandbox = Sandbox::new("loose_ids");
    // /a/'s old block is 126 to 150, so 0128 is slot 03 and 0130 slot 05;
    // 0149 is a spare position and 0155 past the block.
    sandbox.write("a Spring/a Note.txt", b"Team: /a/\r\n");
    face(&sandbox, "a Spring/Faces/XXX03 - Ann");
    boots(&sandbox, "a Spring/Boots/k0128 - Rex");
    boots(&sandbox, "a Spring/Boots/k0130");
    sandbox.write("a Spring/Boots/k0149/boots.fmdl", b"spare");
    sandbox.write("a Spring/Boots/k0155/boots.fmdl", b"past");

    let run = upgrade_loose(&sandbox, "a Spring");
    assert_eq!(run.exit_code(), 1);
    let stdout = run.stdout();
    for line in [
        "- a Spring: Info wearer_assumed at Boots/k0128 - Rex (id=0128, slot=03)\n",
        "- a Spring: Info wearer_assumed at Boots/k0130 (id=0130, slot=05)\n",
        "- a Spring: Error unmigrated_content at Boots/k0149\n",
        "- a Spring: Error unmigrated_content at Boots/k0155\n",
    ] {
        assert!(stdout.contains(line), "{line:?} not in:\n{stdout}");
    }
    let out = "a Full Spring";
    assert_eq!(
        fs::read(sandbox.path(&format!("{out}/Players/03 - Ann/boots.fmdl"))).unwrap(),
        b"boots"
    );
    assert_eq!(
        fs::read(sandbox.path(&format!("{out}/Players/05/boots.fmdl"))).unwrap(),
        b"boots"
    );
    assert_eq!(
        fs::read(sandbox.path(&format!("{out}/Players/05/ingame_face"))).unwrap(),
        b""
    );
    assert!(
        !sandbox
            .path(&format!("{out}/Players/05/settings.toml"))
            .exists()
    );
    let text =
        fs::read_to_string(sandbox.path(&format!("{out}/Players/03 - Ann/settings.toml"))).unwrap();
    let parsed = PlayerSettings::parse(&text).expect("the settings.toml parses");
    assert_eq!(
        parsed,
        PlayerSettings {
            name: Some(NameSetting::FromFolder),
            ..PlayerSettings::default()
        }
    );
    let mut entries: Vec<String> = fs::read_dir(sandbox.path(&format!("{out}/Players")))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_str().unwrap().to_owned())
        .collect();
    entries.sort();
    assert_eq!(entries, ["03 - Ann", "05"]);
    assert!(!sandbox.path(&format!("{out}/Boots")).exists());
    for entry in sandbox.listing(out) {
        let path = sandbox.path(&format!("{out}/{entry}"));
        if path.is_file() {
            let bytes = fs::read(&path).unwrap();
            assert!(bytes != b"spare" && bytes != b"past", "{entry}");
        }
    }
}
