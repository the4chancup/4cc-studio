//! `Collars/`: which files the export format admits there, and the collar name's checks, which
//! `check` reports as `compile` does.

use std::fs;
use std::path::Path;

use crate::common::Sandbox;
use crate::compile::{compiled_players, pes_settings, pes21_settings};
use crate::{CLEAN_PLAYER, TEAM_COLORS_MISSING, clean_model, findings_of, no_deploy_lines};

/// The export the tests here write, with the coverage tag a `Midcup` export carries.
const EXPORT: &str = "exports/co Midcup Collars";

/// Konami's pre-Fox shirt model `modD_shirt_tight_in_collar_052.model`, from `pes_model`'s
/// fixtures: a `.model` that reads.
fn pre_fox_model() -> Vec<u8> {
    fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../libs/pes_model/tests/fixtures/konami_collar_052.wesys.model"),
    )
    .unwrap()
}

// TC-CMN-02
#[test]
fn a_collar_named_for_a_reserved_or_no_stock_collar_is_refused_and_dropped() {
    let sandbox = Sandbox::new("collar_ids");
    for name in ["collar_105.fmdl", "neck.fmdl", "collar_9999.fmdl"] {
        sandbox.write(&format!("{EXPORT}/Collars/{name}"), &clean_model());
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(
        findings_of(&run.messages(), "co Midcup Collars"),
        [
            "Error collar_id_conflict [DropFile] at Collars/collar_105.fmdl (file=collar_105.fmdl, claimant=FPC)",
            "Error collar_id_invalid [DropFile] at Collars/collar_9999.fmdl (file=collar_9999.fmdl)",
            "Error collar_id_invalid [DropFile] at Collars/neck.fmdl (file=neck.fmdl)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-CMN-10
#[test]
fn a_texture_in_collars_is_dropped_and_the_collar_model_kept() {
    let sandbox = Sandbox::new("collar_texture");
    sandbox.write(&format!("{EXPORT}/Collars/collar_12.fmdl"), &clean_model());
    sandbox.write(&format!("{EXPORT}/Collars/collar_12.dds"), b"");
    let disallowed =
        "Error file_type_disallowed [DropFile] at Collars/collar_12.dds (file=collar_12.dds)";

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);

    assert_eq!(
        findings_of(&check.messages(), "co Midcup Collars"),
        [
            disallowed,
            "Info export_identified [Keep] (team=/co/, id=714)"
        ]
    );
    assert_eq!(check.exit_code(), 1);

    // The model is kept, and `compile` does not build collars yet.
    let compile = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        findings_of(&compile.messages(), "co Midcup Collars"),
        [
            disallowed,
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Collars/collar_12.fmdl)",
        ]
    );
}

// The refusal half of TC-CMN-08; its other half needs collars compiled.
#[test]
fn a_pre_fox_collar_past_the_version_s_stock_set_is_refused() {
    let sandbox = Sandbox::new("collar_pre_fox");
    sandbox.write(
        &format!("{EXPORT}/Collars/collar_117.model"),
        &pre_fox_model(),
    );

    let past = sandbox.run(&pes_settings(&sandbox, 17), &["check"]);

    assert_eq!(
        findings_of(&past.messages(), "co Midcup Collars"),
        [
            "Error collar_id_invalid [DropFile] at Collars/collar_117.model (file=collar_117.model)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(past.exit_code(), 1);

    let sandbox = Sandbox::new("collar_pre_fox_last");
    sandbox.write(
        &format!("{EXPORT}/Collars/collar_116.model"),
        &pre_fox_model(),
    );

    let last = sandbox.run(&pes_settings(&sandbox, 17), &["check"]);

    assert_eq!(
        findings_of(&last.messages(), "co Midcup Collars"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(last.exit_code(), 0);
}

#[test]
fn an_export_whose_only_collar_is_dropped_compiles_without_it() {
    let sandbox = Sandbox::new("collar_dropped");
    sandbox.write(&format!("{EXPORT}/{CLEAN_PLAYER}"), &clean_model());
    sandbox.write(
        &format!("{EXPORT}/Collars/collar_9999.fmdl"),
        &clean_model(),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(
        run.messages(),
        no_deploy_lines(
            &sandbox,
            [
                "co Midcup Collars: Error collar_id_invalid [DropFile] at Collars/collar_9999.fmdl (file=collar_9999.fmdl)",
                "co Midcup Collars: Info export_identified [Keep] (team=/co/, id=714)",
                &format!("co Midcup Collars: {TEAM_COLORS_MISSING}"),
            ]
        )
    );
    assert_eq!(compiled_players(&sandbox), [71403]);
    assert_eq!(run.exit_code(), 1);
}
