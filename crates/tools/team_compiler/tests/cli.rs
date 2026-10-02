//! The Team compiler's command line run in-process (`team_compiler/settings.md` "CLI"): the
//! preflight's refusals and `check`'s findings, observed as the exit code and the `Message`
//! events the tool emits, which the binary prints one line each.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use common::{Run, Sandbox};
use studio_core::{ExportId, PipelineEvent, RunId, Scope, StudioTool};
use team_compiler::Tool;

impl Sandbox {
    fn write(&self, relative: &str, contents: &[u8]) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    /// Copies `tests/fixtures/sources/<name>` to `<relative folder>/<name>`, leaving the fixture
    /// untouched.
    fn copy_fixture(&self, name: &str, folder: &str) {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sources")
            .join(name);
        self.write(&format!("{folder}/{name}"), &fs::read(fixture).unwrap());
    }

    /// The platform's spelling of a path inside the sandbox, as a source failure names it.
    fn display(&self, relative: &str) -> String {
        relative
            .split('/')
            .fold(self.root.clone(), |path, segment| path.join(segment))
            .display()
            .to_string()
    }

    /// An absolute path inside the sandbox, as a command-line argument.
    fn arg(&self, relative: &str) -> String {
        self.root.join(relative).to_str().unwrap().to_owned()
    }

    /// Copies the tracer bullet's export to `exports/<name>`, leaving the fixture untouched.
    fn copy_tracer(&self, name: &str) {
        let export = self.root.join("exports").join(name);
        for (relative, bytes) in snapshot(Path::new(&tracer_export())) {
            let path = export.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
    }
}

impl Run {
    /// The refusal, asserting its exit code and that its message names every one of `names`.
    fn assert_refused(&self, exit_code: u8, names: &[&str]) {
        let error = match &self.result {
            Ok(code) => panic!("not refused: exit code {code}"),
            Err(error) => error,
        };
        let text = error.to_string();
        assert_eq!(error.exit_code, exit_code, "{text}");
        for name in names {
            assert!(
                text.contains(name),
                "the refusal does not name {name}: {text}"
            );
        }
        assert!(self.events.is_empty(), "a refused command emits nothing");
    }

    /// Every `Message` event as one line: the export's file name (from its `ExportStarted`),
    /// severity, code, disposition, scope and context, so a test asserts all of them at once.
    fn messages(&self) -> Vec<String> {
        let mut names: BTreeMap<ExportId, String> = BTreeMap::new();
        let mut lines = Vec::new();
        for envelope in &self.events {
            match &envelope.event {
                PipelineEvent::ExportStarted {
                    export_id,
                    display_name,
                } => {
                    names.insert(*export_id, display_name.clone());
                }
                PipelineEvent::Message(message) => {
                    assert_eq!(envelope.export_id, message.scope.export_id());
                    let source = match message.scope.export_id() {
                        Some(export_id) => format!("{}: ", names[&export_id]),
                        None => String::new(),
                    };
                    let location = match &message.scope {
                        Scope::Run | Scope::Export { .. } => String::new(),
                        Scope::Folder { path, .. } | Scope::File { path, .. } => {
                            format!(" at {}", path.as_str())
                        }
                        Scope::RosterEntry {
                            file, line, slot, ..
                        } => format!(" at {} line {line} slot {slot:?}", file.as_str()),
                    };
                    let context: Vec<String> = message
                        .context
                        .iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect();
                    lines.push(format!(
                        "{source}{:?} {} [{:?}]{location} ({})",
                        message.severity,
                        message.code.code,
                        message.disposition,
                        context.join(", ")
                    ));
                }
                PipelineEvent::ExportProcessed { .. }
                | PipelineEvent::FolderStatus { .. }
                | PipelineEvent::Progress { .. }
                | PipelineEvent::Complete { .. }
                | PipelineEvent::UnmigratedContentFound => {}
            }
        }
        lines
    }
}

/// Every file under `root`, by relative path, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else {
                let bytes = fs::read(&path).unwrap();
                files.insert(path.strip_prefix(root).unwrap().to_path_buf(), bytes);
            }
        }
    }
    files
}

/// One player folder with a face model: an export that validates with no finding.
const CLEAN_PLAYER: &str = "Players/03 - A/face_high.fmdl";

// ---------------------------------------------------------------- the tool itself

#[test]
fn the_tool_is_registered_as_the_team_compiler() {
    assert_eq!(Tool.id(), "team-compiler");
    assert_eq!(Tool.label(), "Team compiler");
    assert_eq!(
        Tool.default_settings()["cpk_name"].as_str(),
        Some("4cc_90_test")
    );
}

// ---------------------------------------------------------------- the preflight

// TC-CLI-03
#[test]
fn no_deploy_with_test_mode_is_an_invalid_invocation() {
    let sandbox = Sandbox::new("no_deploy_test_mode");
    let run = sandbox.run("", &["compile", "--no-deploy", "--mode", "test"]);
    run.assert_refused(2, &["--no-deploy", "--mode test", "incompatible"]);
    assert!(!sandbox.root.join("output").exists());
}

// TC-CLI-05
#[test]
fn an_invalid_cpk_name_refuses_compile_before_any_export_is_read() {
    for (index, name) in ["", "con", "a/b"].into_iter().enumerate() {
        let sandbox = Sandbox::new(&format!("cpk_name_{index}"));
        sandbox.write("exports/aaa_export/notes.txt", b"kept as it is");
        let before = snapshot(&sandbox.root);

        // Reading this exports root, which does not exist, would abort with 3.
        let run = sandbox.run(
            &format!("[team-compiler]\ncpk_name = \"{name}\"\n"),
            &["compile", &sandbox.arg("no such exports")],
        );

        run.assert_refused(2, &[&format!("cpk_name = \"{name}\"")]);
        assert_eq!(snapshot(&sandbox.root), before);
    }
}

// TC-CLI-07
#[test]
fn an_export_path_that_is_missing_or_no_archive_is_an_invalid_invocation() {
    let sandbox = Sandbox::new("export_paths");
    sandbox.write("notes.txt", b"not an export");
    for command in ["check", "compile"] {
        let missing = sandbox.arg("no_such_export");
        let run = sandbox.run("", &[command, "--export", &missing]);
        run.assert_refused(2, &[&missing, "no such folder or file"]);
        let not_archive = sandbox.arg("notes.txt");
        let run = sandbox.run("", &[command, "--export", &not_archive]);
        run.assert_refused(2, &[&not_archive, "not a folder, .zip or .7z"]);
    }
}

#[test]
fn a_setting_of_the_wrong_type_is_a_configuration_error_naming_it() {
    let sandbox = Sandbox::new("wrong_type");
    let run = sandbox.run(
        "[team-compiler]\nstrict_file_type_check = \"yes\"\n",
        &["check"],
    );
    run.assert_refused(2, &["settings [team-compiler]", "strict_file_type_check"]);
    let Err(error) = &run.result else {
        unreachable!("assert_refused checked the refusal")
    };
    let text = error.to_string();
    assert_eq!(text.trim_end(), text, "no trailing newline");
}

#[test]
fn a_memory_cap_of_zero_is_a_configuration_error() {
    let sandbox = Sandbox::new("memory_cap");
    let run = sandbox.run("[common]\nmemory_cap_percent = 0.0\n", &["check"]);
    run.assert_refused(2, &["memory_cap_percent"]);
}

#[test]
fn an_unparsable_teams_list_aborts_naming_it() {
    let sandbox = Sandbox::new("teams_list");
    sandbox.write("data/teams_list.txt", b"not a teams list\n");
    let run = sandbox.run("", &["check"]);
    run.assert_refused(3, &["teams_list.txt"]);
}

#[test]
fn modes_and_commands_this_version_lacks_are_refused() {
    let sandbox = Sandbox::new("not_available");
    let run = sandbox.run("", &["compile", "--mode", "test"]);
    run.assert_refused(2, &["--mode test", "not available yet"]);
    let run = sandbox.run("", &["compile", "--mode", "sider"]);
    run.assert_refused(2, &["--mode sider", "not available yet"]);
    let run = sandbox.run("", &["upgrade-dpfl"]);
    run.assert_refused(2, &["upgrade-dpfl", "not available yet"]);

    let multicpk = "[team-compiler]\nmulticpk_mode = true\n";
    let run = sandbox.run(multicpk, &["compile"]);
    run.assert_refused(2, &["multicpk_mode", "not available yet"]);
    // `check` ignores multicpk_mode.
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"");
    assert_eq!(sandbox.run(multicpk, &["check"]).exit_code(), 0);
}

// ---------------------------------------------------------------- compile

/// The tracer bullet's export, read in place.
fn tracer_export() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/tracer/studio/egg Tracer")
        .to_str()
        .unwrap()
        .to_owned()
}

/// Settings targeting PES 21 with the PES folder at `<sandbox>/PES`.
fn pes21_settings(sandbox: &Sandbox) -> String {
    format!(
        "[common]\npes_version = 21\npes_folder_path = '{}'\n",
        sandbox.root.join("PES").display()
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

/// The path of every entry of the CPK at `path`.
fn cpk_paths(path: &Path) -> Vec<String> {
    cpk::CpkArchive::open(fs::File::open(path).unwrap())
        .unwrap()
        .entries()
        .iter()
        .map(|entry| entry.path.clone())
        .collect()
}

/// The CPK path of the kit texture `name` (`u0792g1`).
fn kit_texture(name: &str) -> String {
    format!("Asset/model/character/uniform/texture/#windx11/{name}.ftex")
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
    let kit = fs::read(Path::new(&tracer_export()).join("Kits/g1/kit.dds")).unwrap();
    sandbox.write("exports/da - Kits/Kits/p1/kit.dds", &kit);
    sandbox.write("exports/da - Kits/players.txt", b"");
    // No roster slot maps this folder, so it would emit nothing.
    sandbox.write("exports/da - Kits/Players/Boots Only/boots.fmdl", b"");

    let run = sandbox.run(&pes21_settings(&sandbox), &["compile"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Boots"),
        [
            "Info export_identified [Keep] (team=/co/, id=701)",
            "Error content_not_yet_compiled [DropExport] (what=Players/Boots Only/boots.fmdl)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
    let entries = cpk_paths(&sandbox.root.join("output/4cc_90_test.cpk"));
    for texture in ["u0792g1", "u0702p1"] {
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

// ---------------------------------------------------------------- check

#[test]
fn check_brackets_each_export_with_its_start_and_end() {
    let sandbox = Sandbox::new("envelopes");
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"");
    let run = sandbox.run("", &["check"]);
    assert_eq!(run.exit_code(), 0);
    let export = Some(ExportId(0));
    let kinds: Vec<(Option<ExportId>, &str)> = run
        .events
        .iter()
        .map(|envelope| {
            assert_eq!(envelope.run_id, RunId(1));
            assert_eq!(envelope.export_revision, None);
            let kind = match &envelope.event {
                PipelineEvent::ExportStarted {
                    export_id,
                    display_name,
                } => {
                    assert_eq!(
                        (*export_id, display_name.as_str()),
                        (ExportId(0), "co - Spring")
                    );
                    "started"
                }
                PipelineEvent::Message(_) => "message",
                PipelineEvent::ExportProcessed { export_id } => {
                    assert_eq!(*export_id, ExportId(0));
                    "processed"
                }
                PipelineEvent::FolderStatus { .. }
                | PipelineEvent::Progress { .. }
                | PipelineEvent::Complete { .. }
                | PipelineEvent::UnmigratedContentFound => "other",
            };
            (envelope.export_id, kind)
        })
        .collect();
    assert_eq!(
        kinds,
        [
            (export, "started"),
            (export, "message"),
            (export, "processed")
        ]
    );
}

// TC-SRC-05
#[test]
fn two_refs_exports_are_both_skipped_and_a_disabled_one_is_left_out() {
    let sandbox = Sandbox::new("multiple_refs");
    for name in ["refs a", "refs b"] {
        sandbox.write(&format!("exports/{name}/players.txt"), b"01 Keeper\n");
        sandbox.write(
            &format!("exports/{name}/Players/Keeper/face_high.fmdl"),
            b"",
        );
        // Validated, this would be root_file_unexpected: its absence shows neither is.
        sandbox.write(&format!("exports/{name}/extra.bin"), b"");
    }
    sandbox.write("exports/refs c/NO_USE", b"");
    sandbox.write("exports/refs c/players.txt", b"01 Keeper\n");

    let run = sandbox.run("", &["check"]);

    assert_eq!(
        run.messages(),
        [
            "Error multiple_ref_exports [DropExport] (exports=refs a, refs b)",
            "refs a: Error multiple_ref_exports [DropExport] ()",
            "refs b: Error multiple_ref_exports [DropExport] ()",
            "refs c: Info export_disabled [DropExport] ()",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-STR-09
#[test]
fn a_disallowed_file_is_an_error_when_strict_and_an_info_when_lenient() {
    let sandbox = Sandbox::new("strict_file_type_check");
    sandbox.write("exports/co - Spring/Players/03 - A/hair.dds", b"");
    sandbox.write("exports/co - Spring/Players/03 - A/readme.txt", b"");

    let strict = sandbox.run(
        "[team-compiler]\nstrict_file_type_check = true\n",
        &["check"],
    );
    assert_eq!(
        strict.messages(),
        [
            "co - Spring: Error file_type_disallowed [DropFolder] at Players/03 - A (file=readme.txt)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(strict.exit_code(), 1);

    let lenient = sandbox.run(
        "[team-compiler]\nstrict_file_type_check = false\n",
        &["check"],
    );
    assert_eq!(
        lenient.messages(),
        [
            "co - Spring: Info file_type_disallowed [Keep] at Players/03 - A (file=readme.txt)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(lenient.exit_code(), 0);
}

#[test]
fn pass_through_keeps_a_disallowed_file_at_its_error_severity() {
    let sandbox = Sandbox::new("pass_through");
    sandbox.write("exports/co - Spring/Players/03 - A/readme.txt", b"");
    let run = sandbox.run("[team-compiler]\npass_through = true\n", &["check"]);
    assert_eq!(
        run.messages(),
        [
            "co - Spring: Error file_type_disallowed [Keep] at Players/03 - A (file=readme.txt)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-CLI-01
#[test]
fn check_writes_nothing() {
    let sandbox = Sandbox::new("writes_nothing");
    let settings = "[team-compiler]\ncpk_name = \"cup\"\n";
    sandbox.write("data/settings.toml", settings.as_bytes());
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"model");
    sandbox.write("exports/co - Spring/notes.txt", b"a note");
    sandbox.write("exports/zz - Autumn/Players/03 - B/readme.txt", b"text");
    let before = snapshot(&sandbox.root);

    let run = sandbox.run(settings, &["check"]);

    assert_eq!(run.exit_code(), 1);
    assert!(!sandbox.root.join("output").exists());
    // The settings file, the teams list and every export file, listing and bytes.
    assert_eq!(snapshot(&sandbox.root), before);
}

// TC-CLI-02
#[test]
fn check_exits_one_only_for_an_error_finding() {
    let sandbox = Sandbox::new("exit_codes");
    sandbox.write(&format!("exports/co - Notes/{CLEAN_PLAYER}"), b"");
    sandbox.write("exports/co - Notes/notes.txt", b"a note");
    sandbox.write("exports/co - Notes/extra.bin", b"");
    sandbox.write("exports/co - Error/Players/03 - A/readme.txt", b"");
    sandbox.write(&format!("exports/co - Clean/{CLEAN_PLAYER}"), b"");

    let run = sandbox.run(
        "",
        &["check", "--export", &sandbox.arg("exports/co - Notes")],
    );
    assert_eq!(
        run.messages(),
        [
            "co - Notes: Warning root_file_unexpected [DropFile] at extra.bin ()",
            "co - Notes: Info notes_found [Keep] at notes.txt ()",
            "co - Notes: Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(run.exit_code(), 0);

    let run = sandbox.run(
        "",
        &["check", "--export", &sandbox.arg("exports/co - Error")],
    );
    assert_eq!(
        run.messages(),
        [
            "co - Error: Error file_type_disallowed [DropFolder] at Players/03 - A (file=readme.txt)",
            "co - Error: Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(run.exit_code(), 1);

    let run = sandbox.run(
        "",
        &["check", "--export", &sandbox.arg("exports/co - Clean")],
    );
    assert_eq!(
        run.messages(),
        ["co - Clean: Info export_identified [Keep] (team=/co/, id=701)"]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn a_disabled_export_reports_only_that_whatever_the_marker_case() {
    let sandbox = Sandbox::new("disabled");
    for (name, marker) in [("co - One", "NO_USE.txt"), ("co - Two", "no_use")] {
        sandbox.write(&format!("exports/{name}/{marker}"), b"");
        sandbox.write(&format!("exports/{name}/extra.bin"), b"");
    }
    // A marker below the source's own root is the nested root's content, not a marker.
    sandbox.write("exports/co - Three/wrapper/NO_USE", b"");
    sandbox.write(&format!("exports/co - Three/wrapper/{CLEAN_PLAYER}"), b"");

    let run = sandbox.run("", &["check"]);

    assert_eq!(
        run.messages(),
        [
            "co - One: Info export_disabled [DropExport] ()",
            "co - Three: Warning nested_folders_fixed [Keep] (folder=wrapper)",
            "co - Three: Warning root_file_unexpected [DropFile] at NO_USE ()",
            "co - Three: Info export_identified [Keep] (team=/co/, id=701)",
            "co - Two: Info export_disabled [DropExport] ()",
        ]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn a_balls_export_is_skipped_unread() {
    let sandbox = Sandbox::new("balls");
    sandbox.write("exports/balls Spring/extra.bin", b"");
    let run = sandbox.run("", &["check"]);
    assert_eq!(
        run.messages(),
        ["balls Spring: Info export_balls_skipped [DropExport] ()"]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn an_export_is_identified_by_its_first_word() {
    let sandbox = Sandbox::new("identified");
    sandbox.write(&format!("exports/co - Spring 2026/{CLEAN_PLAYER}"), b"");
    // A refs export needs no teams-list row.
    sandbox.write("exports/refs Cup/players.txt", b"01 Keeper\n");
    sandbox.write("exports/refs Cup/Players/Keeper/face_high.fmdl", b"");
    let run = sandbox.run("", &["check"]);
    assert_eq!(
        run.messages(),
        [
            "co - Spring 2026: Info export_identified [Keep] (team=/co/, id=701)",
            "refs Cup: Info export_identified [Keep] (team=referees)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
}

#[test]
fn an_unknown_team_is_an_error_and_the_export_beside_it_is_still_checked() {
    let sandbox = Sandbox::new("unknown_team");
    sandbox.write(&format!("exports/zz - Spring/{CLEAN_PLAYER}"), b"");
    sandbox.write(&format!("exports/---/{CLEAN_PLAYER}"), b"");
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"");
    let run = sandbox.run("", &["check"]);
    assert_eq!(
        run.messages(),
        [
            "---: Error team_name_unknown [DropExport] (team_name=)",
            "co - Spring: Info export_identified [Keep] (team=/co/, id=701)",
            "zz - Spring: Error team_name_unknown [DropExport] (team_name=/zz/)",
        ]
    );
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn a_missing_exports_root_aborts_naming_it() {
    let sandbox = Sandbox::new("missing_root");
    let run = sandbox.run("", &["check"]);
    run.assert_refused(3, &[&sandbox.arg("exports")]);
}

#[test]
fn export_paths_restrict_the_run_to_the_named_sources() {
    let sandbox = Sandbox::new("named_sources");
    for name in ["co - A", "co - B", "co - C"] {
        sandbox.write(&format!("exports/{name}/{CLEAN_PLAYER}"), b"");
    }
    sandbox.write(&format!("elsewhere/co - D/{CLEAN_PLAYER}"), b"");
    let run = sandbox.run(
        "",
        &[
            "check",
            "--export",
            &sandbox.arg("elsewhere/co - D"),
            "--export",
            &sandbox.arg("exports/co - B"),
        ],
    );
    assert_eq!(
        run.messages(),
        [
            "co - B: Info export_identified [Keep] (team=/co/, id=701)",
            "co - D: Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(run.exit_code(), 0);
}

// ---------------------------------------------------------------- archive sources

/// The lines of `lines` about `source`, without their `<source>: ` prefix, so the findings of
/// sources holding the same export compare equal.
fn findings_of<'a>(lines: &'a [String], source: &str) -> Vec<&'a str> {
    let prefix = format!("{source}: ");
    lines
        .iter()
        .filter_map(|line| line.strip_prefix(prefix.as_str()))
        .collect()
}

/// The folder `exports/co - Spring/`, the same export as the `co - Spring` fixtures: git keeps no
/// empty folder, so it is built here.
fn spring_folder(sandbox: &Sandbox) {
    sandbox.write("exports/co - Spring/players.txt", b"01 Keeper\n");
    sandbox.write(
        "exports/co - Spring/notes.txt",
        b"Spring kit placeholder.\n",
    );
    fs::create_dir_all(sandbox.root.join("exports/co - Spring/Kits/p2")).unwrap();
}

/// What `check` reports about the `co - Spring` export, in any of its three forms. The roster
/// line names a player folder the export does not hold, so its finding shows the roster was read.
const SPRING_FINDINGS: [&str; 3] = [
    "Error players_txt_target_missing [DropSlot] at players.txt line 1 slot Some(1) (folder=Keeper)",
    "Info notes_found [Keep] at notes.txt ()",
    "Info export_identified [Keep] (team=/co/, id=701)",
];

#[test]
fn one_export_as_a_folder_a_zip_and_a_7z_reports_the_same_findings() {
    let sandbox = Sandbox::new("same_export_three_ways");
    spring_folder(&sandbox);
    sandbox.copy_fixture("co - Spring.zip", "exports");
    sandbox.copy_fixture("co - Spring.7z", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    for source in ["co - Spring", "co - Spring.zip", "co - Spring.7z"] {
        assert_eq!(findings_of(&lines, source), SPRING_FINDINGS, "{lines:#?}");
    }
    // Three exports of one team draw no finding about each other.
    assert_eq!(lines.len(), 3 * SPRING_FINDINGS.len(), "{lines:#?}");
    assert_eq!(run.exit_code(), 1);
}

// TC-SRC-02
#[test]
fn a_folder_and_an_archive_sharing_a_stem_are_two_exports() {
    let sandbox = Sandbox::new("folder_and_archive");
    spring_folder(&sandbox);
    sandbox.copy_fixture("co - Spring.zip", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(findings_of(&lines, "co - Spring"), SPRING_FINDINGS);
    assert_eq!(findings_of(&lines, "co - Spring.zip"), SPRING_FINDINGS);
    assert_eq!(lines.len(), 2 * SPRING_FINDINGS.len(), "{lines:#?}");
}

#[test]
fn a_corrupt_archive_is_skipped_and_the_export_beside_it_is_still_checked() {
    let sandbox = Sandbox::new("corrupt_archive");
    // The scan compares the extension in any case.
    sandbox.write("exports/co - Broken.ZIP", b"not a zip at all");
    sandbox.write(&format!("exports/co - Spring/{CLEAN_PLAYER}"), b"");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Broken.ZIP"),
        [format!(
            "Error export_extract_failed [DropExport] (path={}, error=zip: invalid Zip archive: Could not find EOCD)",
            sandbox.display("exports/co - Broken.ZIP")
        )]
    );
    assert_eq!(
        findings_of(&lines, "co - Spring"),
        ["Info export_identified [Keep] (team=/co/, id=701)"]
    );
    assert_eq!(run.exit_code(), 1);
}

// TC-SRC-07
#[test]
fn an_archive_whose_names_collide_or_escape_is_skipped_naming_the_path() {
    let sandbox = Sandbox::new("refused_listings");
    sandbox.copy_fixture("co - Case.zip", "exports");
    sandbox.copy_fixture("co - Escape.zip", "exports");

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Case.zip"),
        [
            "Error export_extract_failed [DropExport] (path=Players.txt, error=path collides with existing entry players.txt)"
        ]
    );
    assert_eq!(
        findings_of(&lines, "co - Escape.zip"),
        [format!(
            "Error export_extract_failed [DropExport] (path={}, error=invalid entry name \"../x\")",
            sandbox.display("exports/co - Escape.zip")
        )]
    );
    assert_eq!(lines.len(), 2, "{lines:#?}");
    assert_eq!(run.exit_code(), 1);
}

#[test]
fn export_paths_may_name_an_archive_outside_the_root() {
    let sandbox = Sandbox::new("named_archive");
    sandbox.write(&format!("exports/co - A/{CLEAN_PLAYER}"), b"");
    sandbox.write(&format!("exports/co - C/{CLEAN_PLAYER}"), b"");
    sandbox.copy_fixture("co - Spring.zip", "elsewhere");

    let run = sandbox.run(
        "",
        &[
            "check",
            "--export",
            &sandbox.arg("elsewhere/co - Spring.zip"),
            "--export",
            &sandbox.arg("exports/co - A"),
        ],
    );

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - A"),
        ["Info export_identified [Keep] (team=/co/, id=701)"]
    );
    assert_eq!(findings_of(&lines, "co - Spring.zip"), SPRING_FINDINGS);
    // co - C, in the root but not named, is not reported.
    assert_eq!(lines.len(), 1 + SPRING_FINDINGS.len(), "{lines:#?}");
}

#[test]
fn check_leaves_every_archive_and_nested_export_as_it_was() {
    let sandbox = Sandbox::new("archives_untouched");
    sandbox.copy_fixture("co - Spring.zip", "exports");
    sandbox.copy_fixture("co - Spring.7z", "exports");
    sandbox.write(
        &format!("exports/co - Nested/wrapper/{CLEAN_PLAYER}"),
        b"model",
    );
    let exports = sandbox.root.join("exports");
    let before = snapshot(&exports);

    let run = sandbox.run("", &["check"]);

    let lines = run.messages();
    assert_eq!(
        findings_of(&lines, "co - Nested"),
        [
            "Warning nested_folders_fixed [Keep] (folder=wrapper)",
            "Info export_identified [Keep] (team=/co/, id=701)",
        ]
    );
    assert_eq!(findings_of(&lines, "co - Spring.7z"), SPRING_FINDINGS);
    assert_eq!(findings_of(&lines, "co - Spring.zip"), SPRING_FINDINGS);
    assert_eq!(snapshot(&exports), before);
}
