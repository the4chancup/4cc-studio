//! `compile`: the CPK it writes, the exports it skips, and the findings and exit code it reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use studio_core::PipelineEvent;

use crate::common::{Run, Sandbox};
use crate::{CLEAN_PLAYER, findings_of, snapshot, source_fixture};

impl Sandbox {
    /// Copies every file under `source` into `<folder>`, leaving `source` untouched.
    fn copy_folder(&self, source: &Path, folder: &str) {
        for (relative, bytes) in snapshot(source) {
            let path = self.root.join(folder).join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
    }

    /// Copies the tracer bullet's export to `exports/<name>`.
    pub(crate) fn copy_tracer(&self, name: &str) {
        self.copy_folder(Path::new(&tracer_export()), &format!("exports/{name}"));
    }

    /// Copies the tracer bullet's player folder into `<folder>`: a face folder holding only
    /// what Phase 3 compiles, so it compiles with no finding.
    pub(crate) fn copy_tracer_face(&self, folder: &str) {
        let face = Path::new(&tracer_export()).join("Players/05 - The Chad Stormworks Player");
        self.copy_folder(&face, folder);
    }
}

/// The tracer bullet's export, read in place.
fn tracer_export() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tracer/studio/egg Tracer")
        .to_str()
        .unwrap()
        .to_owned()
}

/// The tracer bullet's `kit.dds`, a kit texture that compiles. Written without the tracer's
/// `config.toml`, its kit reports `kit_config_generated`.
fn tracer_kit() -> Vec<u8> {
    fs::read(Path::new(&tracer_export()).join("Kits/g1/kit.dds")).unwrap()
}

/// Settings targeting PES 21 with the PES folder at `<sandbox>/PES`.
pub(crate) fn pes21_settings(sandbox: &Sandbox) -> String {
    format!(
        "[common]\npes_version = 21\npes_folder_path = '{}'\n",
        sandbox.root.join("PES").display()
    )
}

/// `pes21_settings` with `pass_through` on.
fn pass_through_settings(sandbox: &Sandbox) -> String {
    format!(
        "{}[team-compiler]\npass_through = true\n",
        pes21_settings(sandbox)
    )
}

// TC-OUT-01
#[test]
fn compile_no_deploy_writes_the_cpk_to_the_output_folder_and_leaves_pes_alone() {
    let sandbox = Sandbox::new("no_deploy");
    sandbox.write("PES/download/4cc_90_test.cpk", b"the installed CPK");
    sandbox.write("PES/PES2021.exe", b"the game");
    let pes_before = snapshot(&sandbox.root.join("PES"));

    let run = sandbox.run(
        &pes21_settings(&sandbox),
        &["compile", "--no-deploy", "--export", &tracer_export()],
    );

    assert_eq!(run.exit_code(), 0);
    let promoted = sandbox.root.join("output").join("4cc_90_test.cpk");
    let entries = cpk::CpkArchive::open(fs::File::open(&promoted).unwrap())
        .unwrap()
        .entries()
        .len();
    assert!(entries > 0, "the CPK holds the tracer's content");
    assert_eq!(
        run.messages(),
        [
            "egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)".to_owned(),
            format!(
                "Info deploy_skipped_by_flag [Keep] (path={})",
                promoted.display()
            ),
        ]
    );
    assert_eq!(snapshot(&sandbox.root.join("PES")), pes_before);
    // The staging folder went with the rename: a finished run leaves only the CPK.
    assert!(!sandbox.root.join("output/.staging").exists());
}

#[test]
fn compile_without_no_deploy_promotes_the_cpk_silently() {
    let sandbox = Sandbox::new("promoted");
    let run = sandbox.run(
        &pes21_settings(&sandbox),
        &["compile", "--export", &tracer_export()],
    );
    assert_eq!(run.exit_code(), 0);
    assert!(sandbox.root.join("output/4cc_90_test.cpk").is_file());
    assert_eq!(
        run.messages(),
        ["egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)"]
    );
}

#[test]
fn a_compile_that_emits_nothing_writes_no_cpk_and_no_staging_folder() {
    let sandbox = Sandbox::new("emits_nothing");
    sandbox.write("exports/co - Off/NO_USE", b"");
    sandbox.write(&format!("exports/co - Off/{CLEAN_PLAYER}"), b"");
    sandbox.write("output/4cc_90_test.cpk", b"the previous CPK");
    let before = snapshot(&sandbox.root.join("output"));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile", "--no-deploy"]);

    assert_eq!(run.exit_code(), 0);
    assert_eq!(
        run.messages(),
        ["co - Off: Info export_disabled [DropExport] ()"]
    );
    assert_eq!(snapshot(&sandbox.root.join("output")), before);
    assert!(!sandbox.root.join("output/.staging").exists());
}

// TC-CLI-06
#[test]
fn an_output_folder_that_cannot_be_created_refuses_compile_before_any_export_is_read() {
    let sandbox = Sandbox::new("output_unwritable");
    sandbox.write("blocker", b"");
    sandbox.copy_tracer("egg Tracer");
    // Absolute, as `pes21_settings` writes the PES path; a file is in the way of its parent.
    let output = format!("{}/blocker/out", sandbox.root.display());
    let settings = format!(
        "{}[team-compiler]\noutput_folder_path = '{output}'\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile"]);

    run.assert_refused(3, &[&output]);
}

// TC-OUT-03
#[test]
fn a_failed_cpk_write_discards_the_staging_and_leaves_the_previous_cpk() {
    let sandbox = Sandbox::new("cpk_write_failed");
    sandbox.write("output/4cc_90_test.cpk", b"the previous CPK");
    // Two exports of one team with the same player: the CPK refuses the second's face path.
    sandbox.copy_tracer_face("exports/co - A/Players/03 - A");
    sandbox.copy_tracer_face("exports/co - B/Players/03 - A");
    let before = snapshot(&sandbox.root.join("output"));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let (last, first) = lines.split_last().unwrap();
    assert_eq!(
        first,
        [
            "co - A: Info export_identified [Keep] (team=/co/, id=714)",
            "co - B: Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    // The error names the staged CPK, whose folder is this run's: `<pid>-<ms>` is left free.
    let previous = sandbox.root.join("output").join("4cc_90_test.cpk");
    let prefix = format!(
        "Fatal cpk_write_failed [AbortRun] (path={}, error={}",
        previous.display(),
        sandbox.root.join("output").join(".staging").display()
    );
    let texture = "Asset/model/character/common/714/03 - A/sourceimages/#windx11/shirt.ftex";
    let suffix = format!(
        "{}4cc_90_test.cpk: cannot add {texture}: duplicate path in archive: {texture})",
        std::path::MAIN_SEPARATOR
    );
    assert!(
        last.starts_with(&prefix) && last.ends_with(&suffix),
        "{last}"
    );
    assert_eq!(run.exit_code(), 3);
    assert_eq!(snapshot(&sandbox.root.join("output")), before);
    assert!(!sandbox.root.join("output/.staging").exists());
}

/// Asserts `run` compiled the tracer's export and then failed to replace the previous CPK:
/// `output_commit_failed` naming it is the last finding, the exit code is 3, and `output/`
/// holds exactly what it held `before`.
fn assert_commit_failed(sandbox: &Sandbox, run: &Run, before: &BTreeMap<PathBuf, Vec<u8>>) {
    let previous = sandbox.root.join("output").join("4cc_90_test.cpk");
    let lines = run.messages();
    let (last, first) = lines.split_last().unwrap();
    assert_eq!(
        first,
        ["egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)"]
    );
    // The error ends with the platform's own text, so only its shape is fixed.
    let prefix = format!(
        "Fatal output_commit_failed [AbortRun] (path={previous}, error={previous}: cannot replace it with the new CPK: ",
        previous = previous.display()
    );
    assert!(last.starts_with(&prefix) && last.ends_with(')'), "{last}");
    assert_eq!(run.exit_code(), 3);
    assert_eq!(&snapshot(&sandbox.root.join("output")), before);
    assert!(!sandbox.root.join("output/.staging").exists());
}

#[cfg(windows)]
// TC-OUT-05
#[test]
fn a_previous_cpk_held_open_without_delete_sharing_fails_the_commit_and_is_kept() {
    use std::os::windows::fs::OpenOptionsExt;
    /// `FILE_SHARE_READ` alone: another process may read the file, not delete or replace it.
    const FILE_SHARE_READ: u32 = 1;

    let sandbox = Sandbox::new("output_commit_held_open");
    sandbox.write("output/4cc_90_test.cpk", b"the previous CPK");
    sandbox.copy_tracer("egg Tracer");
    let before = snapshot(&sandbox.root.join("output"));
    let held_open = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(sandbox.root.join("output").join("4cc_90_test.cpk"))
        .unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    drop(held_open);
    assert_commit_failed(&sandbox, &run, &before);
}

#[test]
fn a_previous_cpk_that_is_a_folder_fails_the_commit_and_is_kept() {
    let sandbox = Sandbox::new("output_commit_folder");
    sandbox.write("output/4cc_90_test.cpk/kept.txt", b"not a CPK");
    sandbox.copy_tracer("egg Tracer");
    let before = snapshot(&sandbox.root.join("output"));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_commit_failed(&sandbox, &run, &before);
}

/// The path of every entry of the CPK at `path`.
fn cpk_paths(path: &Path) -> Vec<String> {
    cpk::CpkArchive::open(fs::File::open(path).unwrap())
        .unwrap()
        .entries()
        .iter()
        .map(|entry| entry.path.clone())
        .collect()
}

/// Every entry of the CPK at `path`, by its path, with its bytes.
pub(crate) fn cpk_entries(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut cpk = cpk::CpkArchive::open(fs::File::open(path).unwrap()).unwrap();
    let entries = cpk.entries().to_vec();
    entries
        .iter()
        .map(|entry| (entry.path.clone(), cpk.read(entry).unwrap()))
        .collect()
}

/// The CPK path of the kit texture `name` (`u0792g1`).
pub(crate) fn kit_texture(name: &str) -> String {
    format!("Asset/model/character/uniform/texture/#windx11/{name}.ftex")
}

/// The player id of every face in the sandbox's compiled CPK, sorted.
pub(crate) fn compiled_players(sandbox: &Sandbox) -> Vec<u32> {
    let mut players: Vec<u32> = cpk_paths(&sandbox.root.join("output/4cc_90_test.cpk"))
        .iter()
        .filter_map(|path| {
            path.strip_prefix("Asset/model/character/face/real/")?
                .strip_suffix("/#Win/face.fpk")
        })
        .map(|id| id.parse().unwrap())
        .collect();
    players.sort_unstable();
    players
}

/// The name of every kit texture in the sandbox's compiled CPK (`u0714g1`), sorted.
fn compiled_kits(sandbox: &Sandbox) -> Vec<String> {
    let mut kits: Vec<String> = cpk_paths(&sandbox.root.join("output/4cc_90_test.cpk"))
        .iter()
        .filter_map(|path| {
            path.strip_prefix("Asset/model/character/uniform/texture/#windx11/")?
                .strip_suffix(".ftex")
        })
        .map(str::to_owned)
        .collect();
    kits.sort_unstable();
    kits
}

/// `exports/<name>`, an export whose roster maps a player folder holding only `boots.fmdl`,
/// which no run reads.
fn boots_only_export(sandbox: &Sandbox, name: &str) {
    sandbox.write(&format!("exports/{name}/players.txt"), b"03 Boots Only\n");
    sandbox.write(
        &format!("exports/{name}/Players/Boots Only/boots.fmdl"),
        b"",
    );
}

// TC-OUT-06
#[test]
fn compile_skips_an_export_holding_content_it_cannot_build_yet_and_builds_the_others() {
    let sandbox = Sandbox::new("not_yet_compiled");
    boots_only_export(&sandbox, "co - Boots");
    sandbox.copy_tracer("egg Tracer");
    sandbox.write("exports/dbg - Kits/Kits/p1/kit.dds", &tracer_kit());
    sandbox.write("exports/dbg - Kits/players.txt", b"");
    // No roster slot maps this folder, so it would emit nothing.
    sandbox.write("exports/dbg - Kits/Players/Boots Only/boots.fmdl", b"");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Boots"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Players/Boots Only/boots.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_paths(&sandbox.root.join("output/4cc_90_test.cpk"));
    for texture in ["u0792g1", "u0790p1"] {
        assert!(entries.contains(&kit_texture(texture)), "{entries:#?}");
    }

    let check = sandbox.run(&pes21_settings(&sandbox), &["check"]);
    let lines = check.messages();
    assert!(
        lines
            .iter()
            .all(|line| !line.contains("content_not_yet_compiled")),
        "{lines:#?}"
    );
}

// TC-OUT-02
#[test]
fn a_compile_whose_every_export_is_skipped_leaves_the_previous_cpk_as_it_was() {
    let sandbox = Sandbox::new("every_export_skipped");
    sandbox.write("output/4cc_90_test.cpk", b"the previous CPK");
    sandbox.write("exports/refs Cup/players.txt", b"01 Keeper\n");
    sandbox.write("exports/refs Cup/Players/Keeper/face_high.fmdl", b"");
    boots_only_export(&sandbox, "co - Boots");
    let before = snapshot(&sandbox.root.join("output"));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    // Each skipped export reports an Error, so the run exits with 1.
    assert_eq!(run.exit_code(), 1);
    assert_eq!(snapshot(&sandbox.root.join("output")), before);
    assert!(!sandbox.root.join("output/.staging").exists());
}

// TC-OUT-04
#[test]
fn an_export_whose_only_player_folder_is_dropped_writes_no_cpk() {
    let sandbox = Sandbox::new("only_folder_dropped");
    sandbox.write("output/4cc_90_test.cpk", b"the previous CPK");
    sandbox.write(&format!("exports/co - Links/{CLEAN_PLAYER}"), b"");
    sandbox.write("exports/co - Links/Players/03 - A/Crocs.boots", b"");
    let before = snapshot(&sandbox.root.join("output"));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert!(
        findings_of(&lines, "co - Links").iter().any(
            |line| line.starts_with("Error link_target_missing [DropFolder] at Players/03 - A")
        ),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(snapshot(&sandbox.root.join("output")), before);
}

// TC-ID-02
#[test]
fn an_export_of_an_unknown_team_is_skipped_and_the_one_beside_it_compiled() {
    let sandbox = Sandbox::new("unknown_team_compiled");
    sandbox.write(&format!("exports/zz - Spring/{CLEAN_PLAYER}"), b"");
    sandbox.copy_tracer("egg Tracer");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "zz - Spring"),
        ["Error team_name_unknown [DropExport] (team_name=/zz/)"]
    );
    let entries = cpk_paths(&sandbox.root.join("output/4cc_90_test.cpk"));
    assert!(entries.contains(&kit_texture("u0792g1")), "{entries:#?}");
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn a_pre_fox_compile_skips_every_export_naming_the_target() {
    let sandbox = Sandbox::new("pre_fox_compile");
    sandbox.copy_tracer("egg Tracer");

    let run = sandbox.run("[common]\npes_version = 17\n", &["compile"]);

    let lines = run.messages();
    assert!(
        lines.contains(
            &"egg Tracer: Error content_not_yet_compiled [DropExport] (what=PES 2017)".to_owned()
        ),
        "{lines:#?}"
    );
    assert_eq!(run.exit_code(), 1);
    assert!(!sandbox.root.join("output/4cc_90_test.cpk").exists());
}

#[test]
fn compile_creates_a_missing_teams_list_and_check_does_not() {
    let sandbox = Sandbox::new("teams_list_created");
    fs::create_dir_all(sandbox.root.join("exports")).unwrap();
    let list = sandbox.root.join("data/teams_list.txt");
    fs::remove_file(&list).unwrap();

    assert_eq!(sandbox.run("", &["check"]).exit_code(), 0);
    assert!(!list.exists(), "check writes nothing");

    assert_eq!(sandbox.run("", &["compile"]).exit_code(), 0);
    assert_eq!(
        fs::read(&list).unwrap(),
        teams_list::TeamsList::UPSTREAM.as_bytes()
    );
}

/// `exports/co - Links`: player 03 compiles, player 07 links boots the export does not hold.
fn links_export(sandbox: &Sandbox) {
    sandbox.copy_tracer_face("exports/co - Links/Players/03 - A");
    sandbox.copy_tracer_face("exports/co - Links/Players/07 - B");
    sandbox.write("exports/co - Links/Players/07 - B/Crocs.boots", b"");
}

// TC-DSP-01
#[test]
fn a_dropped_player_folder_is_left_out_of_the_cpk() {
    let sandbox = Sandbox::new("dsp_link_dropped");
    links_export(&sandbox);

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Links"),
        [
            "Error link_target_missing [DropFolder] at Players/07 - B (link=Crocs.boots)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403]);
    assert_eq!(run.exit_code(), 1);
}

// TC-DSP-02
#[test]
fn pass_through_compiles_a_folder_with_a_missing_link_and_still_reports_the_error() {
    let sandbox = Sandbox::new("dsp_link_kept");
    links_export(&sandbox);

    let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Links"),
        [
            "Error link_target_missing [Keep] at Players/07 - B (link=Crocs.boots)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 71407]);
    assert_eq!(run.exit_code(), 1);
}

/// One TC-DSP-03 case: the export `co - Case` holding what `setup` writes, compiled with
/// `pass_through` on.
struct DropCase {
    name: &'static str,
    setup: fn(&Sandbox),
    findings: &'static [&'static str],
    /// The player ids and kit textures of the CPK, or `None` when no CPK is written.
    compiled: Option<(&'static [u32], &'static [&'static str])>,
    exit_code: u8,
}

/// The export every TC-DSP-03 case writes into.
const CASE: &str = "exports/co - Case";

/// Player 03, a face folder that compiles with no finding, in `co - Case`.
fn case_player_03(sandbox: &Sandbox) {
    sandbox.copy_tracer_face(&format!("{CASE}/Players/03 - A"));
}

/// Writes the tracer bullet's kit texture into `co - Case`'s kit folder `folder`.
fn case_kit(sandbox: &Sandbox, folder: &str) {
    sandbox.write(&format!("{CASE}/Kits/{folder}/kit.dds"), &tracer_kit());
}

const DROP_CASES: [DropCase; 8] = [
    DropCase {
        name: "players_txt_slot_duplicate",
        setup: |sandbox| {
            sandbox.write(&format!("{CASE}/players.txt"), b"03 A\n03 B\n");
            sandbox.copy_tracer_face(&format!("{CASE}/Players/A"));
            sandbox.copy_tracer_face(&format!("{CASE}/Players/B"));
        },
        findings: &[
            "Error players_txt_slot_duplicate [DropExport] at players.txt line 2 slot Some(3) ()",
        ],
        compiled: None,
        exit_code: 1,
    },
    DropCase {
        name: "players_txt_slot_invalid",
        setup: |sandbox| {
            sandbox.write(&format!("{CASE}/players.txt"), b"03 A\n24 B\n");
            sandbox.copy_tracer_face(&format!("{CASE}/Players/A"));
            sandbox.copy_tracer_face(&format!("{CASE}/Players/B"));
        },
        findings: &[
            "Error players_txt_slot_invalid [DropSlot] at players.txt line 2 slot Some(24) ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
        compiled: Some((&[71403], &[])),
        exit_code: 1,
    },
    DropCase {
        name: "shared_folder_orphaned",
        setup: |sandbox| {
            case_player_03(sandbox);
            sandbox.write(&format!("{CASE}/Boots/Solo/boots.fmdl"), b"");
        },
        // Kept, Boots/Solo would make the export's content one Phase 3 does not compile,
        // so the missing content_not_yet_compiled shows the folder was dropped.
        findings: &[
            "Warning shared_folder_orphaned [DropFolder] at Boots/Solo ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
        compiled: Some((&[71403], &[])),
        exit_code: 0,
    },
    DropCase {
        name: "fpc_conflict",
        setup: |sandbox| {
            case_player_03(sandbox);
            sandbox.copy_tracer_face(&format!("{CASE}/Players/07 - B"));
            sandbox.write(&format!("{CASE}/Players/07 - B/fpc.on"), b"");
            sandbox.write(&format!("{CASE}/Players/07 - B/fpc.off"), b"");
        },
        findings: &[
            "Error fpc_conflict [DropFolder] at Players/07 - B ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
        compiled: Some((&[71403], &[])),
        exit_code: 1,
    },
    DropCase {
        name: "kit_layout_conflict",
        setup: |sandbox| {
            case_player_03(sandbox);
            case_kit(sandbox, "p1");
            sandbox.write(&format!("{CASE}/Kits/p1/pre-fox"), b"");
            sandbox.write(&format!("{CASE}/Kits/p1/fox"), b"");
        },
        findings: &[
            "Error kit_layout_conflict [DropFolder] at Kits/p1 ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
        compiled: Some((&[71403], &[])),
        exit_code: 1,
    },
    DropCase {
        name: "shared_link_duplicate",
        setup: |sandbox| {
            case_player_03(sandbox);
            sandbox.copy_tracer_face(&format!("{CASE}/Players/07 - B"));
            sandbox.write(&format!("{CASE}/Players/07 - B/Crocs.boots"), b"");
            sandbox.write(&format!("{CASE}/Players/07 - B/Mud.boots"), b"");
        },
        // The export holds no Boots/ folder, so each link also reports its missing target,
        // which pass_through keeps; the folder is dropped all the same.
        findings: &[
            "Error shared_link_duplicate [DropFolder] at Players/07 - B (kind=boots)",
            "Error link_target_missing [Keep] at Players/07 - B (link=Crocs.boots)",
            "Error link_target_missing [Keep] at Players/07 - B (link=Mud.boots)",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ],
        compiled: Some((&[71403], &[])),
        exit_code: 1,
    },
    DropCase {
        name: "kit_slot_duplicate",
        setup: |sandbox| {
            case_player_03(sandbox);
            case_kit(sandbox, "p1");
            case_kit(sandbox, "p1 - Lakers");
            case_kit(sandbox, "g1");
        },
        findings: &[
            "Error kit_slot_duplicate [DropFolder] at Kits/p1 ()",
            "Error kit_slot_duplicate [DropFolder] at Kits/p1 - Lakers ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
        ],
        compiled: Some((&[71403], &["u0714g1"])),
        exit_code: 1,
    },
    DropCase {
        name: "kit_folder_invalid",
        setup: |sandbox| {
            case_player_03(sandbox);
            case_kit(sandbox, "p10");
            case_kit(sandbox, "g1");
        },
        findings: &[
            "Error kit_folder_invalid [DropFolder] at Kits/p10 ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
        ],
        compiled: Some((&[71403], &["u0714g1"])),
        exit_code: 1,
    },
];

// TC-DSP-03
#[test]
fn pass_through_keeps_no_finding_whose_drop_it_cannot_keep() {
    for case in DROP_CASES {
        let sandbox = Sandbox::new(&format!("dsp_{}", case.name));
        (case.setup)(&sandbox);

        let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

        let lines = run.messages();
        assert_eq!(
            findings_of(&lines, "co - Case"),
            case.findings,
            "{}",
            case.name
        );
        assert_eq!(run.exit_code(), case.exit_code, "{}", case.name);
        let cpk = sandbox.root.join("output/4cc_90_test.cpk");
        match case.compiled {
            Some((players, kits)) => {
                assert_eq!(compiled_players(&sandbox), players, "{}", case.name);
                assert_eq!(compiled_kits(&sandbox), kits, "{}", case.name);
            }
            None => assert!(!cpk.exists(), "{}", case.name),
        }
    }
}

// TC-DSP-04
#[test]
fn an_export_skipped_by_its_roster_leaves_the_export_beside_it_as_compiled_alone() {
    let beside = Sandbox::new("dsp_skipped_beside");
    beside.write("exports/co - Dup/players.txt", b"03 A\n03 B\n");
    beside.copy_tracer_face("exports/co - Dup/Players/A");
    beside.copy_tracer_face("exports/co - Dup/Players/B");
    beside.copy_tracer("egg Tracer");
    let alone = Sandbox::new("dsp_tracer_alone");
    alone.copy_tracer("egg Tracer");

    let run = beside.run(&pes21_settings(&beside), &["compile"]);
    assert_eq!(
        alone.run(&pes21_settings(&alone), &["compile"]).exit_code(),
        0
    );

    assert_eq!(
        findings_of(&run.messages(), "co - Dup"),
        ["Error players_txt_slot_duplicate [DropExport] at players.txt line 2 slot Some(3) ()"]
    );
    assert_eq!(run.exit_code(), 1);
    assert_eq!(
        fs::read(beside.root.join("output/4cc_90_test.cpk")).unwrap(),
        fs::read(alone.root.join("output/4cc_90_test.cpk")).unwrap()
    );
}

// TC-ROS-01
#[test]
fn without_players_txt_each_folder_is_the_player_its_number_names() {
    let sandbox = Sandbox::new("ros_numbered");
    sandbox.copy_tracer_face("exports/co - Numbers/Players/03 - A");
    sandbox.copy_tracer_face("exports/co - Numbers/Players/15 - B");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Numbers"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 71415]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROS-04
#[test]
fn players_txt_maps_a_folder_by_name_and_leaves_an_unlisted_one_out() {
    let sandbox = Sandbox::new("ros_named");
    sandbox.write("exports/co - Roster/players.txt", b"03 snuffy\n");
    sandbox.copy_tracer_face("exports/co - Roster/Players/Snuffy");
    sandbox.copy_tracer_face("exports/co - Roster/Players/15 - B");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Roster"),
        [
            "Warning player_unlisted [DropFolder] at Players/15 - B ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROS-05
#[test]
fn a_folder_listed_under_two_slots_is_compiled_for_both() {
    let sandbox = Sandbox::new("ros_two_slots");
    sandbox.write("exports/co - Twice/players.txt", b"03 A\n07 A\n");
    sandbox.copy_tracer_face("exports/co - Twice/Players/A");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Twice"),
        ["Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 71407]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROS-06
#[test]
fn each_bad_roster_line_is_reported_and_only_the_good_one_compiled() {
    let sandbox = Sandbox::new("ros_bad_lines");
    sandbox.write(
        "exports/co - Lines/players.txt",
        b"x3 A\n24 B\n05 Nobody\n07 C\n",
    );
    for folder in ["A", "B", "C"] {
        sandbox.copy_tracer_face(&format!("exports/co - Lines/Players/{folder}"));
    }

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Lines"),
        [
            "Error players_txt_line_invalid [DropSlot] at players.txt line 1 slot None ()",
            "Error players_txt_slot_invalid [DropSlot] at players.txt line 2 slot Some(24) ()",
            "Error players_txt_target_missing [DropSlot] at players.txt line 3 slot Some(5) (folder=Nobody)",
            "Warning player_unlisted [DropFolder] at Players/A ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71407]);
    assert_eq!(run.exit_code(), 1);
}

// TC-ROS-09
#[test]
fn an_empty_players_txt_compiles_the_kits_and_no_player() {
    let sandbox = Sandbox::new("ros_empty_roster");
    sandbox.write("exports/co - Empty/players.txt", b"");
    sandbox.copy_tracer_face("exports/co - Empty/Players/03 - A");
    sandbox.write("exports/co - Empty/Kits/p1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Empty"),
        [
            "Warning player_unlisted [DropFolder] at Players/03 - A ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/p1 ()",
        ]
    );
    assert!(compiled_players(&sandbox).is_empty());
    assert_eq!(compiled_kits(&sandbox), ["u0714p1"]);
    assert_eq!(run.exit_code(), 0);
}

// TC-KIT-01
#[test]
fn kit_folders_are_compiled_as_the_slot_their_name_starts_with() {
    let sandbox = Sandbox::new("kit_named");
    sandbox.write("exports/co - Kits/Kits/p1 - Lakers/kit.dds", &tracer_kit());
    sandbox.write("exports/co - Kits/Kits/g1/kit.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Kits"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/p1 - Lakers ()",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
        ]
    );
    assert_eq!(compiled_kits(&sandbox), ["u0714g1", "u0714p1"]);
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn a_kit_holding_only_a_back_texture_gets_the_placeholder_main_texture() {
    let sandbox = Sandbox::new("kit_back_only");
    sandbox.write("exports/co - Back/Kits/p4/kit_back.dds", &tracer_kit());

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "co - Back"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info kit_config_generated [Keep] at Kits/p4 ()",
            "Info kit_placeholder [Keep] at Kits/p4 ()",
        ]
    );
    assert_eq!(compiled_kits(&sandbox), ["u0714p4", "u0714p4_back"]);
    let names = kit_config::texture_names(
        714,
        kit_config::KitSlot::P4,
        kit_config::TexturePresence {
            kit: true,
            back: true,
            ..kit_config::TexturePresence::default()
        },
    );
    let config =
        kit_config::KitConfig::template().encode_with_names(pes_version::PesVersion::Pes21, &names);
    let entries = cpk_entries(&sandbox.root.join("output/4cc_90_test.cpk"));
    assert_eq!(
        entries["common/character0/model/character/uniform/team/714/714_DEF_4th_realUni.bin"],
        config
    );
    assert_eq!(run.exit_code(), 0);
}

// TC-STR-01
#[test]
fn content_nested_one_folder_down_compiles_as_if_at_the_root() {
    let sandbox = Sandbox::new("str_nested");
    sandbox.write("exports/co - Wrapped/notes.txt", b"Notes.\n");
    sandbox.copy_tracer_face("exports/co - Wrapped/wrapper/Players/03 - A");
    sandbox.copy_tracer_face("exports/dbg - Doubled/Players/Players/03 - A");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Wrapped"),
        [
            "Warning nested_folders_fixed [Keep] (folder=wrapper)",
            "Info notes_found [Keep] at notes.txt ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
        ]
    );
    assert_eq!(
        findings_of(&lines, "dbg - Doubled"),
        [
            "Warning nested_folders_fixed [Keep] at Players (folder=Players/Players)",
            "Info export_identified [Keep] (team=/dbg/, id=790)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [71403, 79003]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROOT-02
#[test]
fn pass_through_drops_notes_that_are_not_utf8_and_compiles_the_export() {
    let sandbox = Sandbox::new("root_notes_encoding");
    sandbox.copy_tracer("egg Tracer");
    sandbox.write("exports/egg Tracer/notes.txt", b"caf\xe9\n");

    let run = sandbox.run(&pass_through_settings(&sandbox), &["compile"]);

    assert_eq!(
        findings_of(&run.messages(), "egg Tracer"),
        [
            "Error notes_encoding_invalid [DropFile] at notes.txt ()",
            "Info export_identified [Keep] (team=/egg/, id=792)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [79205]);
    assert_eq!(run.exit_code(), 1);
}

// TC-SRC-03
#[test]
fn a_disabled_export_is_reported_once_by_check_and_compile_and_not_compiled() {
    let sandbox = Sandbox::new("src_disabled");
    sandbox.write("exports/co - One/NO_USE", b"");
    sandbox.copy_tracer_face("exports/co - One/Players/03 - A");
    sandbox.write("exports/dbg - Two/no_use.txt", b"");
    sandbox.copy_tracer_face("exports/dbg - Two/Players/03 - A");

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            run.messages(),
            [
                "co - One: Info export_disabled [DropExport] ()",
                "dbg - Two: Info export_disabled [DropExport] ()",
            ],
            "{command}"
        );
        assert_eq!(run.exit_code(), 0, "{command}");
    }
    assert!(!sandbox.root.join("output/4cc_90_test.cpk").exists());
    assert!(!sandbox.root.join("output/.staging").exists());
}

// TC-SRC-04
#[test]
fn a_balls_export_is_skipped_by_check_and_compile() {
    let sandbox = Sandbox::new("src_balls");
    sandbox.copy_tracer_face("exports/balls Spring/Players/03 - A");
    sandbox.copy_tracer("egg Tracer");

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            findings_of(&run.messages(), "balls Spring"),
            ["Info export_balls_skipped [DropExport] ()"],
            "{command}"
        );
        assert_eq!(run.exit_code(), 0, "{command}");
    }
    assert_eq!(compiled_players(&sandbox), [79205]);
}

// TC-SRC-06
#[test]
fn a_corrupt_zip_is_skipped_and_the_export_beside_it_compiled() {
    let sandbox = Sandbox::new("src_corrupt_zip");
    sandbox.write("exports/co - Broken.zip", b"not a zip at all");
    sandbox.copy_tracer("egg Tracer");

    for command in ["check", "compile"] {
        let run = sandbox.run(&pes21_settings(&sandbox), &[command]);
        assert_eq!(
            findings_of(&run.messages(), "co - Broken.zip"),
            [format!(
                "Error export_extract_failed [DropExport] (path={}, error=zip: invalid Zip archive: Could not find EOCD)",
                sandbox.display("exports/co - Broken.zip")
            )],
            "{command}"
        );
        assert_eq!(run.exit_code(), 1, "{command}");
    }
    assert_eq!(compiled_players(&sandbox), [79205]);
}

// TC-SRC-08
#[test]
fn export_paths_restrict_check_and_compile_to_the_named_exports() {
    let sandbox = Sandbox::new("src_named_exports");
    sandbox.copy_tracer_face("exports/co - A/Players/03 - A");
    sandbox.copy_tracer("egg Tracer");
    sandbox.copy_tracer_face("elsewhere/dbg - D/Players/03 - A");
    let named = [
        "--export",
        &sandbox.arg("elsewhere/dbg - D"),
        "--export",
        &sandbox.arg("exports/co - A"),
    ];

    for command in ["check", "compile"] {
        let args: Vec<&str> = std::iter::once(command).chain(named).collect();
        let run = sandbox.run(&pes21_settings(&sandbox), &args);
        assert_eq!(
            run.messages(),
            [
                "co - A: Info export_identified [Keep] (team=/co/, id=714)",
                "dbg - D: Info export_identified [Keep] (team=/dbg/, id=790)",
            ],
            "{command}"
        );
        assert_eq!(run.exit_code(), 0, "{command}");
    }
    assert_eq!(compiled_players(&sandbox), [71403, 79003]);
    assert!(compiled_kits(&sandbox).is_empty());
}

// TC-ID-01
#[test]
fn a_zip_export_is_compiled_as_the_team_its_name_starts_with() {
    let sandbox = Sandbox::new("id_zip");
    sandbox.write(
        "exports/co - Spring 2026.zip",
        &source_fixture("egg Tracer.zip"),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        ["co - Spring 2026.zip: Info export_identified [Keep] (team=/co/, id=714)"]
    );
    assert_eq!(compiled_players(&sandbox), [71405]);
    assert_eq!(compiled_kits(&sandbox), ["u0714g1"]);
    assert_eq!(run.exit_code(), 0);
}

// TC-ROOT-05
#[test]
fn a_root_notes_txt_that_cannot_be_read_is_dropped_and_the_rest_compiled() {
    let sandbox = Sandbox::new("root_notes_unreadable");
    sandbox.copy_fixture("egg Tracer bad notes.zip", "exports");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    assert_eq!(
        run.messages(),
        [
            "egg Tracer bad notes.zip: Error source_read_failed [DropFile] at notes.txt (reason=Invalid checksum)",
            "egg Tracer bad notes.zip: Info export_identified [Keep] (team=/egg/, id=792)",
        ]
    );
    assert_eq!(compiled_players(&sandbox), [79205]);
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn a_7z_over_the_memory_cap_compiles() {
    let sandbox = Sandbox::new("seven_z_over_cap");
    sandbox.copy_fixture("egg Tracer.7z", "exports");
    // A cap of a few hundred bytes against the archive's 151 KB decompressed: its permit is
    // oversized, so a task asking the budget for its own while it is held would wait forever.
    let settings = format!(
        "{}memory_cap_percent = 0.000001\n",
        pes21_settings(&sandbox)
    );

    let run = sandbox.run(&settings, &["compile"]);

    assert_eq!(
        run.messages(),
        ["egg Tracer.7z: Info export_identified [Keep] (team=/egg/, id=792)"]
    );
    assert_eq!(compiled_players(&sandbox), [79205]);
    assert_eq!(compiled_kits(&sandbox), ["u0792g1"]);
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn an_export_is_processed_after_its_last_task_or_after_planning_when_it_has_none() {
    let sandbox = Sandbox::new("processed_order");
    sandbox.write("exports/co - Kits/Kits/p1/kit.dds", &tracer_kit());
    // Fails on the pool, so its finding is the writer's to report.
    sandbox.write("exports/co - Kits/Kits/g1/kit.dds", b"not a texture");
    sandbox.write("exports/dbg - Off/NO_USE", b"");
    sandbox.write(&format!("exports/dbg - Off/{CLEAN_PLAYER}"), b"");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let events: Vec<String> = run
        .events
        .iter()
        .map(|envelope| match &envelope.event {
            PipelineEvent::ExportStarted { export_id, .. } => format!("started {}", export_id.0),
            PipelineEvent::Message(message) => format!("message {}", message.code.code),
            PipelineEvent::ExportProcessed { export_id } => format!("processed {}", export_id.0),
            other => format!("{other:?}"),
        })
        .collect();
    assert_eq!(
        events,
        [
            "started 0",
            "message export_identified",
            "started 1",
            "message export_disabled",
            "message kit_config_generated",
            "message kit_config_generated",
            "processed 1",
            "message folder_pack_failed",
            "processed 0",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn the_worker_count_changes_neither_the_findings_nor_the_cpk() {
    let mut outcomes = Vec::new();
    for threads in [1, 8] {
        let sandbox = Sandbox::new(&format!("worker_count_{threads}"));
        sandbox.copy_tracer("egg Tracer");
        sandbox.write("exports/dbg Seven.7z", &source_fixture("egg Tracer.7z"));
        sandbox.write("exports/co - Kits/Kits/p1/kit.dds", &tracer_kit());
        sandbox.write("exports/co - Kits/Kits/g1/kit.dds", &tracer_kit());
        let settings = format!("{}thread_count = {threads}\n", pes21_settings(&sandbox));

        let run = sandbox.run(&settings, &["compile"]);

        assert_eq!(run.exit_code(), 0, "{threads} threads");
        let cpk = fs::read(sandbox.root.join("output/4cc_90_test.cpk")).unwrap();
        outcomes.push((run.messages(), cpk));
    }
    assert_eq!(outcomes[0].0, outcomes[1].0);
    assert_eq!(
        outcomes[0].0,
        [
            "co - Kits: Info export_identified [Keep] (team=/co/, id=714)",
            "dbg Seven.7z: Info export_identified [Keep] (team=/dbg/, id=790)",
            "egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)",
            "co - Kits: Info kit_config_generated [Keep] at Kits/p1 ()",
            "co - Kits: Info kit_config_generated [Keep] at Kits/g1 ()",
        ]
    );
    assert!(outcomes[0].1 == outcomes[1].1, "the CPKs differ");
}
