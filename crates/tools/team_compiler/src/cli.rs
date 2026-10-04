//! The Team compiler's command line (`team_compiler/settings.md` "CLI"): a parsed command turned
//! into a run, refused before any export is read when its invocation or configuration is
//! invalid, and the run's outcome turned into the exit code scripts read.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use clap::{Args, Command, FromArgMatches, Subcommand, ValueEnum};
use pipeline::CpkStem;
use studio_core::{AppPaths, CliError, CommonSettings, SETTINGS_FILE_NAME, Severity, ToolContext};
use teams_list::TeamsList;

use crate::messages::TOOL_ID;
use crate::output::deploy;
use crate::reader::is_archive;
use crate::settings::{TeamCompilerSettings, from_table};
use crate::{check, compile};

// The exit codes are the command line's contract with scripts (`settings.md` "CLI").
/// Exit code of a run that finished with no Error finding (warnings and notes allowed).
const CLEAN: u8 = 0;
/// Exit code of a run that finished, but some export had an Error finding.
const ERRORS: u8 = 1;
/// Exit code of an invalid invocation or configuration: nothing ran.
const INVALID: u8 = 2;
/// Exit code of a run aborted before or during its work.
const ABORTED: u8 = 3;

#[derive(Debug, Subcommand)]
enum TeamCompilerCommand {
    /// Compile the exports into a CPK.
    Compile(CompileArgs),
    /// Check the exports without writing anything.
    Check(SourceArgs),
    /// Replace the installed DpFileList with the bundled official one.
    UpgradeDpfl {
        /// Apply the change instead of only listing it.
        #[arg(long)]
        yes: bool,
    },
}

/// Which exports a run reads.
#[derive(Debug, Args)]
struct SourceArgs {
    /// The folder holding the exports, for this run only; default: the exports_folder_path
    /// setting.
    exports_root: Option<PathBuf>,
    /// Only this export: a folder, .zip or .7z, anywhere on disk. Repeatable.
    #[arg(long = "export", value_name = "PATH")]
    exports: Vec<PathBuf>,
}

#[derive(Debug, Args)]
struct CompileArgs {
    #[command(flatten)]
    source: SourceArgs,
    /// Where the compiled content goes.
    #[arg(long, value_enum, default_value_t = Mode::Normal)]
    mode: Mode,
    /// Build the CPKs into the output folder without installing them.
    #[arg(long)]
    no_deploy: bool,
}

/// The output mode of a compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Mode {
    /// CPKs, installed into the game.
    Normal,
    /// Loose files in the output folder's test_output folder.
    Test,
    /// Loose files in the game folder's livecpk folder.
    Sideload,
}

/// What the preflight resolved: everything `check` and `compile` start from.
#[derive(Debug)]
pub(crate) struct RunInputs {
    /// The tool's settings section, already checked.
    pub(crate) settings: TeamCompilerSettings,
    /// The suite-wide settings: the target PES version among them.
    pub(crate) common: CommonSettings,
    /// The working teams list (the embedded one when none is installed), which maps each
    /// export's team name to its id.
    pub(crate) teams_list: TeamsList,
    /// The folder scanned for exports.
    pub(crate) exports_root: PathBuf,
    /// The `--export` paths, as given; empty for every export under the root.
    pub(crate) exports: Vec<PathBuf>,
}

/// The `team-compiler` subcommand and its own subcommands.
pub(crate) fn command() -> Command {
    TeamCompilerCommand::augment_subcommands(Command::new(TOOL_ID).subcommand_required(true))
}

/// Runs a parsed `team-compiler` command: the preflight, then the command.
pub(crate) fn run(matches: &clap::ArgMatches, ctx: &ToolContext) -> Result<u8, CliError> {
    let command = TeamCompilerCommand::from_arg_matches(matches)
        .map_err(|error| CliError::new(INVALID, error))?;
    match command {
        TeamCompilerCommand::Check(source) => {
            check_export_paths(&source.exports)?;
            let common = ctx.common();
            let settings = read_settings(&ctx.tool_settings(TOOL_ID), &common)?;
            let exports_root = prepare_exports_root(&source, &common, ctx.paths())?;
            let inputs =
                resolve_inputs(exports_root, source.exports, settings, common, ctx.paths())?;
            verdict(check::run(&inputs, ctx))
        }
        TeamCompilerCommand::Compile(args) => {
            refuse_mode(args.mode, args.no_deploy)?;
            check_export_paths(&args.source.exports)?;
            let common = ctx.common();
            let settings = read_settings(&ctx.tool_settings(TOOL_ID), &common)?;
            let cpk_stem = compile_settings(&settings)?;
            let exports_root = prepare_exports_root(&args.source, &common, ctx.paths())?;
            // `settings.md` "Path resolution": a relative output folder sits beside the
            // executable; an absolute one replaces the base.
            let output_folder = ctx.paths().exe_dir.join(&settings.output_folder_path);
            // Probed before any export is read, so a run never does the work and then fails
            // to write it.
            deploy::prepare_output_folder(&output_folder)
                .map_err(|error| CliError::new(ABORTED, error))?;
            create_teams_list(&settings.teams_list_path, ctx.paths().data_dir.as_deref())?;
            let inputs = resolve_inputs(
                exports_root,
                args.source.exports,
                settings,
                common,
                ctx.paths(),
            )?;
            verdict(compile::run(
                &inputs,
                &cpk_stem,
                &output_folder,
                args.no_deploy,
                ctx,
            ))
        }
        TeamCompilerCommand::UpgradeDpfl { .. } => Err(invalid(anyhow!(
            "upgrade-dpfl is not available yet in this version"
        ))),
    }
}

/// A run's outcome as the exit code: its worst finding's, or `ABORTED` with the error when the
/// run could not go on.
fn verdict(outcome: anyhow::Result<Option<Severity>>) -> Result<u8, CliError> {
    outcome
        .map(exit_code)
        .map_err(|error| CliError::new(ABORTED, error))
}

/// `settings.md` "CLI": 1 when some export had an Error, 3 when the run hit a Fatal finding,
/// else 0, warnings and notes included.
fn exit_code(worst: Option<Severity>) -> u8 {
    match worst {
        Some(Severity::Fatal) => ABORTED,
        Some(Severity::Error) => ERRORS,
        Some(Severity::Warning | Severity::Info) | None => CLEAN,
    }
}

fn invalid(error: anyhow::Error) -> CliError {
    CliError::new(INVALID, error)
}

/// `--no-deploy` has nothing to skip in the loose-file modes; those modes arrive in Phase 4.
fn refuse_mode(mode: Mode, no_deploy: bool) -> Result<(), CliError> {
    if no_deploy && mode != Mode::Normal {
        return Err(invalid(anyhow!(
            "--no-deploy and --mode {} are incompatible: only a normal compile deploys",
            mode_name(mode)
        )));
    }
    if mode != Mode::Normal {
        return Err(invalid(anyhow!(
            "--mode {} is not available yet in this version",
            mode_name(mode)
        )));
    }
    Ok(())
}

/// The mode as `--mode` spells it.
fn mode_name(mode: Mode) -> String {
    mode.to_possible_value()
        .expect("no `Mode` variant is skipped, so each has a value on the command line")
        .get_name()
        .to_owned()
}

/// Each `--export` must be a folder or a `.zip`/`.7z` file; a relative path resolves against the
/// current directory.
fn check_export_paths(paths: &[PathBuf]) -> Result<(), CliError> {
    for path in paths {
        if path.is_dir() {
            continue;
        }
        if !path.exists() {
            return Err(invalid(anyhow!(
                "--export {}: no such folder or file",
                path.display()
            )));
        }
        if !is_archive(path) {
            return Err(invalid(anyhow!(
                "--export {}: not a folder, .zip or .7z",
                path.display()
            )));
        }
    }
    Ok(())
}

/// The tool's settings from its table, with the checks both commands share on them and on the
/// common settings.
fn read_settings(
    table: &toml::Table,
    common: &CommonSettings,
) -> Result<TeamCompilerSettings, CliError> {
    // A toml error's text ends with a newline, which would leave a blank line under `error: `.
    let settings = from_table(table).map_err(|error| {
        invalid(anyhow!(
            "settings [{TOOL_ID}]: {}",
            error.to_string().trim_end()
        ))
    })?;
    check_memory_cap(common.memory_cap_percent)?;
    Ok(settings)
}

fn check_memory_cap(percent: f32) -> Result<(), CliError> {
    if percent > 0.0 && percent <= 100.0 {
        return Ok(());
    }
    Err(invalid(anyhow!(
        "memory_cap_percent = {percent}: must be above 0 and at most 100"
    )))
}

/// The settings only `compile` reads: the CPK name, and the modes this version cannot compile.
fn compile_settings(settings: &TeamCompilerSettings) -> Result<CpkStem, CliError> {
    let cpk_stem = CpkStem::new(&settings.cpk_name).map_err(|error| {
        invalid(anyhow!(
            "cpk_name = \"{}\" is not a valid CPK name: {error}",
            settings.cpk_name
        ))
    })?;
    if settings.multicpk_mode {
        return Err(invalid(anyhow!(
            "multicpk_mode = true is not available yet in this version"
        )));
    }
    Ok(cpk_stem)
}

/// The run's inputs over the ready `exports_root` and the `--export` paths: the teams list is
/// loaded here, last, so a refused root never reads it.
fn resolve_inputs(
    exports_root: PathBuf,
    exports: Vec<PathBuf>,
    settings: TeamCompilerSettings,
    common: CommonSettings,
    paths: &AppPaths,
) -> Result<RunInputs, CliError> {
    let teams_list = load_teams_list(&settings.teams_list_path, paths.data_dir.as_deref())?;
    Ok(RunInputs {
        settings,
        common,
        teams_list,
        exports_root,
        exports,
    })
}

/// The teams list at `setting` (relative: in the data directory). With no data directory yet or
/// no file there, the embedded list, nothing written; a file that cannot be read or parsed aborts.
fn load_teams_list(setting: &Path, data_dir: Option<&Path>) -> Result<TeamsList, CliError> {
    let path = if setting.is_absolute() {
        setting.to_path_buf()
    } else if let Some(dir) = data_dir {
        dir.join(setting)
    } else {
        return Ok(embedded_teams_list());
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(embedded_teams_list()),
        Err(error) => {
            return Err(CliError::new(
                ABORTED,
                anyhow!("{}: cannot read the teams list: {error}", path.display()),
            ));
        }
    };
    TeamsList::parse(&text).map_err(|error| {
        CliError::new(
            ABORTED,
            anyhow!("{}: not a valid teams list: {error}", path.display()),
        )
    })
}

/// `compile` gives the member a teams list to edit: with a data directory and no file at the
/// `teams_list_path` setting's path, it writes the embedded list there (`check` never writes
/// it). The write is best-effort (`pipeline.md` "Teams list", detected by attempting it): a
/// data directory that cannot take it leaves the embedded list to be read with a warning
/// naming the path, never a failure.
fn create_teams_list(setting: &Path, data_dir: Option<&Path>) -> Result<(), CliError> {
    let Some(data_dir) = data_dir else {
        return Ok(());
    };
    // An absolute setting replaces the data directory in the join, as `load_teams_list` reads it.
    let path = data_dir.join(setting);
    if path.exists() {
        return Ok(());
    }
    // The bytes are written unchanged: the file's CRLF bytes are the upstream list's.
    if let Err(error) = fs::write(&path, TeamsList::UPSTREAM) {
        log::warn!(
            "{}: the teams list cannot be written ({error}); the embedded list is used",
            path.display()
        );
    }
    Ok(())
}

fn embedded_teams_list() -> TeamsList {
    TeamsList::parse(TeamsList::UPSTREAM)
        .expect("the embedded teams list parses; the teams_list crate's tests check it")
}

/// The exports root: the command line's (relative to the current directory), else the
/// `exports_folder_path` setting (relative: beside the executable).
fn exports_root(argument: Option<&Path>, common: &CommonSettings, exe_dir: &Path) -> PathBuf {
    match argument {
        Some(root) => root.to_path_buf(),
        None => exe_dir.join(&common.exports_folder_path),
    }
}

/// The exports root, ready for the run's scan (`settings.md` "Path resolution"). The relative
/// setting's folder is created when missing, so a fresh install has one to put exports in; a
/// folder named any other way (an absolute setting, the command line's root) is never created,
/// and a missing one is a configuration error naming the path and what to do. With `--export`
/// paths the root is not scanned, so it is neither created nor checked.
fn prepare_exports_root(
    source: &SourceArgs,
    common: &CommonSettings,
    paths: &AppPaths,
) -> Result<PathBuf, CliError> {
    let root = exports_root(source.exports_root.as_deref(), common, &paths.exe_dir);
    if !source.exports.is_empty() || root.is_dir() {
        return Ok(root);
    }
    if source.exports_root.is_some() {
        return Err(invalid(anyhow!(
            "the exports folder {} given on the command line does not exist",
            root.display()
        )));
    }
    if common.exports_folder_path.is_absolute() {
        // With no data directory there is no settings file, so the setting is the built-in
        // relative default and this arm is not reached in practice.
        let settings_file = match &paths.data_dir {
            Some(data_dir) => data_dir.join(SETTINGS_FILE_NAME).display().to_string(),
            None => "the settings file".to_owned(),
        };
        return Err(invalid(anyhow!(
            "the exports folder {} does not exist: create it and put your exports inside, or \
             set exports_folder_path in {settings_file} to the folder that holds them",
            root.display()
        )));
    }
    fs::create_dir_all(&root).map_err(|error| {
        CliError::new(
            ABORTED,
            anyhow!(
                "{}: cannot create the exports folder: {error}",
                root.display()
            ),
        )
    })?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::scratch;

    fn parse(args: &[&str]) -> TeamCompilerCommand {
        let matches = command()
            .try_get_matches_from(std::iter::once("team-compiler").chain(args.iter().copied()))
            .unwrap();
        TeamCompilerCommand::from_arg_matches(&matches).unwrap()
    }

    #[test]
    fn compile_defaults_to_normal_mode_and_collects_every_export() {
        let TeamCompilerCommand::Compile(args) =
            parse(&["compile", "root", "--export", "a", "--export", "b.zip"])
        else {
            panic!("not compile");
        };
        assert_eq!(args.mode, Mode::Normal);
        assert!(!args.no_deploy);
        assert_eq!(args.source.exports_root, Some(PathBuf::from("root")));
        assert_eq!(
            args.source.exports,
            [PathBuf::from("a"), PathBuf::from("b.zip")]
        );
    }

    #[test]
    fn a_command_is_required_and_check_takes_no_mode() {
        assert!(command().try_get_matches_from(["team-compiler"]).is_err());
        assert!(
            command()
                .try_get_matches_from(["team-compiler", "check", "--mode", "normal"])
                .is_err()
        );
    }

    #[test]
    fn the_worst_severity_decides_the_exit_code() {
        assert_eq!(exit_code(None), 0);
        assert_eq!(exit_code(Some(Severity::Info)), 0);
        assert_eq!(exit_code(Some(Severity::Warning)), 0);
        assert_eq!(exit_code(Some(Severity::Error)), 1);
        assert_eq!(exit_code(Some(Severity::Fatal)), 3);
    }

    #[test]
    fn a_run_that_cannot_go_on_is_aborted_with_its_error() {
        assert_eq!(verdict(Ok(Some(Severity::Error))).unwrap(), 1);
        let error = verdict(Err(anyhow!("the disk is full"))).unwrap_err();
        assert_eq!(error.exit_code, 3);
        assert_eq!(error.to_string(), "the disk is full");
    }

    #[test]
    fn modes_are_named_as_the_command_line_spells_them() {
        assert_eq!(mode_name(Mode::Normal), "normal");
        assert_eq!(mode_name(Mode::Test), "test");
        assert_eq!(mode_name(Mode::Sideload), "sideload");
    }

    #[test]
    fn no_deploy_is_refused_with_either_loose_file_mode_and_kept_with_normal() {
        assert!(refuse_mode(Mode::Normal, true).is_ok());
        assert!(refuse_mode(Mode::Normal, false).is_ok());
        for mode in [Mode::Test, Mode::Sideload] {
            let error = refuse_mode(mode, true).unwrap_err();
            assert_eq!(error.exit_code, INVALID);
            assert!(error.to_string().contains("incompatible"), "{error}");
            let error = refuse_mode(mode, false).unwrap_err();
            assert!(error.to_string().contains("not available yet"), "{error}");
        }
    }

    #[test]
    fn export_paths_accept_folders_and_archives_only() {
        let temp = scratch("cli_export_paths");
        let root = temp.path();
        fs::write(root.join("pack.7z"), "").unwrap();
        fs::write(root.join("notes.txt"), "").unwrap();
        assert!(check_export_paths(&[root.to_path_buf(), root.join("pack.7z")]).is_ok());
        for refused in [root.join("notes.txt"), root.join("missing.zip")] {
            let error = check_export_paths(std::slice::from_ref(&refused)).unwrap_err();
            assert_eq!(error.exit_code, INVALID);
            assert!(error.to_string().contains(&refused.display().to_string()));
        }
    }

    #[test]
    fn the_memory_cap_must_be_above_zero_and_at_most_a_hundred() {
        assert!(check_memory_cap(100.0).is_ok());
        assert!(check_memory_cap(0.5).is_ok());
        for refused in [0.0, -1.0, 100.5, f32::NAN] {
            let error = check_memory_cap(refused).unwrap_err();
            assert_eq!(error.exit_code, INVALID);
            assert!(error.to_string().contains("memory_cap_percent"));
        }
    }

    #[test]
    fn settings_of_the_wrong_type_or_a_memory_cap_out_of_range_are_refused() {
        let table: toml::Table = toml::from_str("pass_through = true").unwrap();
        let settings = read_settings(&table, &CommonSettings::default()).unwrap();
        assert!(settings.pass_through);

        let table: toml::Table = toml::from_str("pass_through = 1").unwrap();
        let error = read_settings(&table, &CommonSettings::default()).unwrap_err();
        assert_eq!(error.exit_code, INVALID);
        assert!(error.to_string().contains("pass_through"), "{error}");

        let common = CommonSettings {
            memory_cap_percent: 0.0,
            ..CommonSettings::default()
        };
        let error = read_settings(&toml::Table::new(), &common).unwrap_err();
        assert!(error.to_string().contains("memory_cap_percent"), "{error}");
    }

    #[test]
    fn compile_refuses_an_invalid_cpk_name_and_multicpk_mode() {
        let mut settings = TeamCompilerSettings::default();
        assert_eq!(compile_settings(&settings).unwrap().as_str(), "4cc_99_test");
        settings.multicpk_mode = true;
        let error = compile_settings(&settings).unwrap_err();
        assert!(error.to_string().contains("multicpk_mode"), "{error}");
        settings.cpk_name = "con".to_owned();
        let error = compile_settings(&settings).unwrap_err();
        assert_eq!(error.exit_code, INVALID);
        assert!(error.to_string().contains("cpk_name = \"con\""), "{error}");
    }

    #[test]
    fn the_teams_list_falls_back_to_the_embedded_one_without_a_file() {
        let embedded = embedded_teams_list();
        let temp = scratch("cli_teams_list");
        let root = temp.path();
        let relative = Path::new("teams_list.txt");
        assert_eq!(load_teams_list(relative, None).unwrap(), embedded);
        assert_eq!(load_teams_list(relative, Some(root)).unwrap(), embedded);
        assert!(!root.join("teams_list.txt").exists(), "nothing written");

        fs::write(root.join("teams_list.txt"), "ID\tName\n792\t/egg/\n").unwrap();
        let own = load_teams_list(relative, Some(root)).unwrap();
        assert_eq!(own.teams().count(), 1);
        // An absolute path is read even with no data directory.
        let absolute = root.join("teams_list.txt");
        assert_eq!(load_teams_list(&absolute, None).unwrap(), own);

        fs::write(root.join("teams_list.txt"), "not a teams list").unwrap();
        let error = load_teams_list(relative, Some(root)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED);
        assert!(error.to_string().contains("teams_list.txt"), "{error}");

        // A path that exists but cannot be read as a file aborts too.
        fs::remove_file(root.join("teams_list.txt")).unwrap();
        fs::create_dir(root.join("teams_list.txt")).unwrap();
        let error = load_teams_list(relative, Some(root)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED);
        assert!(error.to_string().contains("cannot read"), "{error}");
    }

    #[test]
    fn a_missing_teams_list_is_created_only_with_a_data_directory() {
        let temp = scratch("cli_create_teams_list");
        let root = temp.path();
        let file = root.join("teams_list.txt");
        create_teams_list(&file, None).unwrap();
        assert!(!file.exists(), "no data directory, nothing written");

        create_teams_list(Path::new("teams_list.txt"), Some(root)).unwrap();
        assert_eq!(fs::read(&file).unwrap(), TeamsList::UPSTREAM.as_bytes());

        let own = "ID\tName\n792\t/egg/\n";
        fs::write(&file, own).unwrap();
        create_teams_list(Path::new("teams_list.txt"), Some(root)).unwrap();
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            own,
            "an existing file is kept"
        );
    }

    /// The warning lines `log` has recorded since the recorder was installed.
    fn warnings() -> Vec<String> {
        static LINES: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
        struct Recorder;
        impl log::Log for Recorder {
            fn enabled(&self, _: &log::Metadata) -> bool {
                true
            }
            fn log(&self, record: &log::Record) {
                LINES
                    .lock()
                    .unwrap()
                    .push(format!("{} {}", record.level(), record.args()));
            }
            fn flush(&self) {}
        }
        static RECORDER: Recorder = Recorder;
        log::set_logger(&RECORDER).unwrap_or_default();
        log::set_max_level(log::LevelFilter::Warn);
        LINES.lock().unwrap().clone()
    }

    #[test]
    fn an_unwritable_teams_list_path_warns_and_the_embedded_list_stands() {
        let temp = scratch("cli_teams_list_unwritable");
        let root = temp.path();
        // The write is best-effort: it cannot be made, a warning names it, and the run reads
        // the embedded list exactly as it does with no data directory. The recorder must be
        // installed before the write, or the warning is gone before anyone hears it.
        warnings();
        create_teams_list(Path::new("no folder/teams_list.txt"), Some(root)).unwrap();
        assert_eq!(
            load_teams_list(Path::new("no folder/teams_list.txt"), Some(root)).unwrap(),
            embedded_teams_list(),
        );
        assert!(
            warnings().iter().any(|line| line.starts_with("WARN")
                && line.contains("no folder")
                && line.contains("embedded")),
            "{:?}",
            warnings()
        );
    }

    #[test]
    fn the_exports_root_is_the_argument_or_the_setting_beside_the_executable() {
        let common = CommonSettings::default();
        let exe_dir = Path::new("exe_dir");
        assert_eq!(
            exports_root(Some(Path::new("given")), &common, exe_dir),
            PathBuf::from("given")
        );
        assert_eq!(
            exports_root(None, &common, exe_dir),
            exe_dir.join("exports")
        );
    }

    fn app_paths(exe_dir: &Path, data_dir: Option<&Path>) -> AppPaths {
        AppPaths {
            exe_dir: exe_dir.to_path_buf(),
            data_dir: data_dir.map(Path::to_path_buf),
        }
    }

    #[test]
    fn a_missing_root_given_on_the_command_line_is_refused_and_not_created() {
        let temp = scratch("cli_root_command_line");
        let root = temp.path();
        let given = root.join("given");
        let source = SourceArgs {
            exports_root: Some(given.clone()),
            exports: vec![],
        };
        let error =
            prepare_exports_root(&source, &CommonSettings::default(), &app_paths(root, None))
                .unwrap_err();
        assert_eq!(error.exit_code, INVALID);
        assert_eq!(
            error.to_string(),
            format!(
                "the exports folder {} given on the command line does not exist",
                given.display()
            )
        );
        assert!(!given.exists(), "not created");
    }

    #[test]
    fn with_export_paths_a_missing_root_is_neither_created_nor_refused() {
        let temp = scratch("cli_root_named_exports");
        let root = temp.path();
        let source = SourceArgs {
            exports_root: None,
            exports: vec![root.join("co - A")],
        };
        let ready =
            prepare_exports_root(&source, &CommonSettings::default(), &app_paths(root, None))
                .unwrap();
        assert_eq!(ready, root.join("exports"));
        assert!(!ready.exists(), "not created");
    }

    #[test]
    fn an_absolute_setting_naming_a_missing_folder_is_refused_naming_the_settings_file() {
        let temp = scratch("cli_root_absolute");
        let root = temp.path();
        let missing = root.join("elsewhere");
        let common = CommonSettings {
            exports_folder_path: missing.clone(),
            ..CommonSettings::default()
        };
        let source = SourceArgs {
            exports_root: None,
            exports: vec![],
        };
        let data_dir = root.join("data");
        let error =
            prepare_exports_root(&source, &common, &app_paths(root, Some(&data_dir))).unwrap_err();
        assert_eq!(error.exit_code, INVALID);
        assert_eq!(
            error.to_string(),
            format!(
                "the exports folder {} does not exist: create it and put your exports inside, \
                 or set exports_folder_path in {} to the folder that holds them",
                missing.display(),
                data_dir.join("settings.toml").display()
            )
        );
        // Without a data directory the sentence still reads, naming no file.
        let error = prepare_exports_root(&source, &common, &app_paths(root, None)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("exports_folder_path in the settings file to"),
            "{error}"
        );
        assert!(!missing.exists(), "not created");
    }

    #[test]
    fn the_relative_default_is_created_and_a_folder_that_cannot_be_aborts() {
        let temp = scratch("cli_root_created");
        let root = temp.path();
        let source = SourceArgs {
            exports_root: None,
            exports: vec![],
        };
        let common = CommonSettings::default();
        let ready = prepare_exports_root(&source, &common, &app_paths(root, None)).unwrap();
        assert_eq!(ready, root.join("exports"));
        assert!(ready.is_dir(), "created");
        // Already there: returned as it is.
        assert_eq!(
            prepare_exports_root(&source, &common, &app_paths(root, None)).unwrap(),
            ready
        );

        // A file where the executable's folder should be: the folder cannot be created.
        let blocker = root.join("blocker");
        fs::write(&blocker, "").unwrap();
        let error = prepare_exports_root(&source, &common, &app_paths(&blocker, None)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED);
        let text = error.to_string();
        assert!(
            text.starts_with(&format!(
                "{}: cannot create the exports folder: ",
                blocker.join("exports").display()
            )),
            "{text}"
        );
    }
}
