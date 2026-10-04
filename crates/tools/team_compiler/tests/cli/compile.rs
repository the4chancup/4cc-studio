//! `compile`: the CPK it writes, the exports it skips, and the findings and exit code it reports.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::common::{Run, Sandbox};
use crate::{CLEAN_PLAYER, clean_model, findings_of, snapshot};

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

    /// Copies the tracer bullet's player folder into `<folder>`: a face folder, with its
    /// `portrait.dds`, holding only what `compile` builds, so it compiles with no Warning or
    /// Error (its boots, hair and left glove models are Info `fmdl_weights_not_normalized`).
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
pub(crate) fn tracer_kit() -> Vec<u8> {
    fs::read(Path::new(&tracer_export()).join("Kits/g1/kit.dds")).unwrap()
}

/// The bytes of the file `name` of the tracer bullet's player folder, the one
/// `copy_tracer_face` copies.
pub(crate) fn tracer_player_file(name: &str) -> Vec<u8> {
    fs::read(
        Path::new(&tracer_export())
            .join("Players/05 - The Chad Stormworks Player")
            .join(name),
    )
    .unwrap()
}

/// The tracer bullet's `portrait.dds`.
pub(crate) fn tracer_portrait() -> Vec<u8> {
    tracer_player_file("portrait.dds")
}

/// Settings targeting PES 21 with the PES folder at `<sandbox>/PES`.
pub(crate) fn pes21_settings(sandbox: &Sandbox) -> String {
    pes_settings(sandbox, 21)
}

/// The settings text of a run for PES `version` whose game folder is the sandbox's `PES/`.
pub(crate) fn pes_settings(sandbox: &Sandbox, version: u8) -> String {
    format!(
        "[common]\npes_version = {version}\npes_folder_path = '{}'\n",
        sandbox.root.join("PES").display()
    )
}

/// `pes21_settings` with `pass_through` on.
pub(crate) fn pass_through_settings(sandbox: &Sandbox) -> String {
    format!(
        "{}[team-compiler]\npass_through = true\n",
        pes21_settings(sandbox)
    )
}

// TC-OUT-01
#[test]
fn compile_no_deploy_writes_the_cpk_to_the_output_folder_and_leaves_pes_alone() {
    let sandbox = Sandbox::new("no_deploy");
    sandbox.write("PES/download/4cc_99_test.cpk", b"the installed CPK");
    sandbox.write("PES/PES2021.exe", b"the game");
    let pes_before = snapshot(&sandbox.root.join("PES"));

    let run = sandbox.run(
        &pes21_settings(&sandbox),
        &["compile", "--no-deploy", "--export", &tracer_export()],
    );

    assert_eq!(run.exit_code(), 0);
    let promoted = sandbox.root.join("output").join("4cc_99_test.cpk");
    let entries = cpk::CpkArchive::open(fs::File::open(&promoted).unwrap())
        .unwrap()
        .entries()
        .len();
    assert!(entries > 0, "the CPK holds the tracer's content");
    assert_eq!(
        run.messages(),
        [
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)".to_owned(),
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)".to_owned(),
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)".to_owned(),
            "egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)".to_owned(),
            format!(
                "Info deploy_skipped_by_flag [Keep] (path={})",
                promoted.display()
            )
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
    assert!(sandbox.root.join("output/4cc_99_test.cpk").is_file());
    assert_eq!(
        run.messages(),
        [
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)",
            "egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)"
        ]
    );
}

#[test]
fn a_compile_that_emits_nothing_writes_no_cpk_and_no_staging_folder() {
    let sandbox = Sandbox::new("emits_nothing");
    sandbox.write("exports/co - Off/NO_USE", b"");
    sandbox.write(&format!("exports/co - Off/{CLEAN_PLAYER}"), &clean_model());
    sandbox.write("output/4cc_99_test.cpk", b"the previous CPK");
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
    sandbox.write("output/4cc_99_test.cpk", b"the previous CPK");
    // A file where the run's staging folder goes: the CPK cannot be created in it.
    sandbox.write("output/.staging", b"in the way");
    sandbox.write(&format!("exports/co - A/{CLEAN_PLAYER}"), &clean_model());
    let before = snapshot(&sandbox.root.join("output"));

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    let (last, first) = lines.split_last().unwrap();
    assert_eq!(
        first,
        [
            "co - A: Info export_identified [Keep] (team=/co/, id=714)",
            "co - A: Info team_colors_missing [Keep] ()"
        ]
    );
    // The error names the staged CPK, whose folder is this run's: `<pid>-<ms>` is left free,
    // and so is the platform's own text at the end.
    let previous = sandbox.root.join("output").join("4cc_99_test.cpk");
    let prefix = format!(
        "Fatal cpk_write_failed [AbortRun] (path={}, error={}",
        previous.display(),
        sandbox.root.join("output").join(".staging").display()
    );
    let cannot_create = format!(
        "{}4cc_99_test.cpk: cannot create the CPK: ",
        std::path::MAIN_SEPARATOR
    );
    assert!(
        last.starts_with(&prefix) && last.contains(&cannot_create) && last.ends_with(')'),
        "{last}"
    );
    assert_eq!(run.exit_code(), 3);
    // The previous CPK and the file in the way are all `output/` holds: no partial CPK.
    assert_eq!(snapshot(&sandbox.root.join("output")), before);
}

/// Asserts `run` compiled the tracer's export and then failed to replace the previous CPK:
/// `output_commit_failed` naming it is the last finding, the exit code is 3, and `output/`
/// holds exactly what it held `before`.
fn assert_commit_failed(sandbox: &Sandbox, run: &Run, before: &BTreeMap<PathBuf, Vec<u8>>) {
    let previous = sandbox.root.join("output").join("4cc_99_test.cpk");
    let lines = run.messages();
    let (last, first) = lines.split_last().unwrap();
    assert_eq!(
        first,
        [
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=boots.fmdl, count=1662)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=fcl_hair.fmdl, count=1662)",
            "egg Tracer: Info fmdl_weights_not_normalized [Keep] at Players/05 - The Chad Stormworks Player (file=glove_l.fmdl, count=2)",
            "egg Tracer: Info export_identified [Keep] (team=/egg/, id=792)"
        ]
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
    sandbox.write("output/4cc_99_test.cpk", b"the previous CPK");
    sandbox.copy_tracer("egg Tracer");
    let before = snapshot(&sandbox.root.join("output"));
    let held_open = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(sandbox.root.join("output").join("4cc_99_test.cpk"))
        .unwrap();

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    drop(held_open);
    assert_commit_failed(&sandbox, &run, &before);
}

#[test]
fn a_previous_cpk_that_is_a_folder_fails_the_commit_and_is_kept() {
    let sandbox = Sandbox::new("output_commit_folder");
    sandbox.write("output/4cc_99_test.cpk/kept.txt", b"not a CPK");
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
    let mut players: Vec<u32> = cpk_paths(&sandbox.root.join("output/4cc_99_test.cpk"))
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

/// The file name of every portrait in the sandbox's compiled CPK (`71405.dds`), sorted.
pub(crate) fn compiled_portraits(sandbox: &Sandbox) -> Vec<String> {
    let mut portraits: Vec<String> = cpk_paths(&sandbox.root.join("output/4cc_99_test.cpk"))
        .iter()
        .filter_map(|path| path.strip_prefix("common/render/symbol/player/"))
        .map(str::to_owned)
        .collect();
    portraits.sort_unstable();
    portraits
}

/// The name of every kit texture in the sandbox's compiled CPK (`u0714g1`), sorted.
pub(crate) fn compiled_kits(sandbox: &Sandbox) -> Vec<String> {
    let mut kits: Vec<String> = cpk_paths(&sandbox.root.join("output/4cc_99_test.cpk"))
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

/// `exports/<name>`, an export whose roster maps a player folder holding a boots model and a
/// model in `gloves/` whose name gives no hand, which no run compiles yet.
fn not_yet_compiled_export(sandbox: &Sandbox, name: &str) {
    sandbox.write(&format!("exports/{name}/players.txt"), b"03 Keeper\n");
    for model in ["kit_boots.fmdl", "gloves/keeper.fmdl"] {
        sandbox.write(
            &format!("exports/{name}/Players/Keeper/{model}"),
            &clean_model(),
        );
    }
}

// TC-OUT-06
#[test]
fn compile_skips_an_export_holding_content_it_cannot_build_yet_and_builds_the_others() {
    let sandbox = Sandbox::new("not_yet_compiled");
    not_yet_compiled_export(&sandbox, "co - Keeper");
    sandbox.copy_tracer("egg Tracer");
    sandbox.write("exports/dbg - Kits/Kits/p1/kit.dds", &tracer_kit());
    sandbox.write("exports/dbg - Kits/players.txt", b"");
    // No roster slot maps this folder, so it would emit nothing.
    sandbox.write(
        "exports/dbg - Kits/Players/Keeper/gloves/keeper.fmdl",
        &clean_model(),
    );

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Keeper"),
        [
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Error content_not_yet_compiled [DropExport] (what=Players/Keeper/gloves/keeper.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_paths(&sandbox.root.join("output/4cc_99_test.cpk"));
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
    sandbox.write("output/4cc_99_test.cpk", b"the previous CPK");
    sandbox.write("exports/refs Cup/players.txt", b"01 Keeper\n");
    sandbox.write(
        "exports/refs Cup/Players/Keeper/face_high.fmdl",
        &clean_model(),
    );
    not_yet_compiled_export(&sandbox, "co - Keeper");
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
    sandbox.write("output/4cc_99_test.cpk", b"the previous CPK");
    sandbox.write(
        &format!("exports/co - Links/{CLEAN_PLAYER}"),
        &clean_model(),
    );
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
    sandbox.write(
        &format!("exports/zz - Spring/{CLEAN_PLAYER}"),
        &clean_model(),
    );
    sandbox.copy_tracer("egg Tracer");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "zz - Spring"),
        ["Error team_name_unknown [DropExport] (team_name=/zz/)"]
    );
    let entries = cpk_paths(&sandbox.root.join("output/4cc_99_test.cpk"));
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
    assert!(!sandbox.root.join("output/4cc_99_test.cpk").exists());
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/07 - B (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()"
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

/// Player 03, a face folder that compiles with no Warning or Error, in `co - Case`.
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
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Warning shared_folder_orphaned [DropFolder] at Boots/Solo ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
        ],
        compiled: Some((&[71403], &[])),
        exit_code: 0,
    },
    DropCase {
        name: "fpc_conflict",
        setup: |sandbox| {
            case_player_03(sandbox);
            sandbox.copy_tracer_face(&format!("{CASE}/Players/07 - B"));
            sandbox.write(&format!("{CASE}/Players/07 - B/fpc_on"), b"");
            sandbox.write(&format!("{CASE}/Players/07 - B/fpc_off"), b"");
        },
        findings: &[
            "Error fpc_conflict [DropFolder] at Players/07 - B ()",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Error kit_layout_conflict [DropFolder] at Kits/p1 ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Error kit_slot_duplicate [DropFolder] at Kits/p1 ()",
            "Error kit_slot_duplicate [DropFolder] at Kits/p1 - Lakers ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
            "Info kit_colors_derived [Keep] at Kits/g1 ()",
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
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=boots.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=fcl_hair.fmdl, count=1662)",
            "Info fmdl_weights_not_normalized [Keep] at Players/03 - A (file=glove_l.fmdl, count=2)",
            "Error kit_folder_invalid [DropFolder] at Kits/p10 ()",
            "Info export_identified [Keep] (team=/co/, id=714)",
            "Info team_colors_missing [Keep] ()",
            "Info kit_config_generated [Keep] at Kits/g1 ()",
            "Info kit_colors_derived [Keep] at Kits/g1 ()",
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
        let cpk = sandbox.root.join("output/4cc_99_test.cpk");
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
        fs::read(beside.root.join("output/4cc_99_test.cpk")).unwrap(),
        fs::read(alone.root.join("output/4cc_99_test.cpk")).unwrap()
    );
}
