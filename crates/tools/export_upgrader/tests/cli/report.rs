//! The report's own rules (the plan's step 10, the EU-REP scenarios): every
//! entry no step migrates is named as unmigrated content and the output
//! written without it, and a migration needing no manual attention ends
//! clean.

use pes_version::PesVersion;
use studio_core::{PipelineEvent, Severity};

use crate::common::Sandbox;
use crate::players::{boots, face, unmigrated_found, upgrade, wears};

// EU-REP-01
#[test]
fn each_other_file_is_named_and_dropped() {
    let sandbox = Sandbox::new("rep_other");
    face(&sandbox, "co Spring/Faces/XXX05 - Bob");
    sandbox.write("co Spring/Other/readme.txt", b"x");
    sandbox.write("co Spring/Other/docs/scan.pdf", b"pdf");
    sandbox.save("EDIT");

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 1);
    let stdout = run.stdout();
    for path in ["Other/readme.txt", "Other/docs/scan.pdf"] {
        assert!(
            stdout.contains(&format!(
                "- co Spring: Error unmigrated_content at {path}\n"
            )),
            "{path} not in:\n{stdout}"
        );
    }
    assert!(!sandbox.path("co Full Spring/Other").exists());
    for entry in sandbox.listing("co Full Spring") {
        let name = entry.rsplit('/').next().unwrap_or(&entry);
        assert!(name != "readme.txt" && name != "scan.pdf", "{entry}");
    }
    assert_eq!(unmigrated_found(&run), 1);
}

// EU-REP-03
#[test]
fn a_migration_needing_no_attention_is_clean() {
    let sandbox = Sandbox::new("rep_clean");
    face(&sandbox, "co Spring/Faces/XXX05 - Bob");
    boots(&sandbox, "co Spring/Boots/k0430 - Crocs");
    sandbox.save_edited("EDIT", PesVersion::Pes21, |file| {
        wears(file, 71405, 430, 0);
    });

    let run = upgrade(&sandbox, "co Spring", "EDIT");
    assert_eq!(run.exit_code(), 0, "{}", run.stdout());
    let offenders: Vec<String> = run
        .events
        .iter()
        .filter_map(|envelope| match &envelope.event {
            PipelineEvent::Message(message) if message.severity > Severity::Info => {
                Some(format!("{:?} {}", message.severity, message.code.code))
            }
            _ => None,
        })
        .collect();
    assert!(offenders.is_empty(), "{offenders:?}");
    let last = run
        .events
        .iter()
        .rev()
        .find_map(|envelope| match &envelope.event {
            PipelineEvent::Message(message) => Some(message),
            _ => None,
        })
        .expect("the verdict is a message");
    assert_eq!(last.code.code, "migration_clean");
    assert_eq!(unmigrated_found(&run), 0);
}
