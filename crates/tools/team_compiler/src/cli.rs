//! The Team compiler's command line (`team_compiler/settings.md` "CLI"): the clap surface and the
//! preflight that refuses an invalid invocation or configuration before any export is read.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::anyhow;
use clap::{Args, Command, FromArgMatches, Subcommand, ValueEnum};
use pipeline::CpkStem;
use studio_core::{AppPaths, CliError, CommonSettings, ToolContext};
use teams_list::TeamsList;

use crate::settings::{TeamCompilerSettings, from_table};

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
    /// Loose files in the Sider folder.
    Sider,
}

/// What the preflight resolved: everything `check` and `compile` start from.
#[derive(Debug)]
#[expect(dead_code, reason = "check and compile read these from step 3.8c on")]
pub(crate) struct RunInputs {
    pub(crate) settings: TeamCompilerSettings,
    pub(crate) common: CommonSettings,
    pub(crate) teams_list: TeamsList,
    /// The folder scanned for exports.
    pub(crate) exports_root: PathBuf,
    /// The `--export` paths, as given; empty for every export under the root.
    pub(crate) exports: Vec<PathBuf>,
}

/// The `team-compiler` subcommand and its own subcommands.
pub(crate) fn command() -> Command {
    TeamCompilerCommand::augment_subcommands(Command::new(crate::TOOL_ID).subcommand_required(true))
}

/// Runs a parsed `team-compiler` command: the preflight, then the command.
pub(crate) fn run(matches: &clap::ArgMatches, ctx: &ToolContext) -> Result<u8, CliError> {
    let command = TeamCompilerCommand::from_arg_matches(matches)
        .map_err(|error| CliError::new(INVALID, error))?;
    match command {
        TeamCompilerCommand::Check(source) => {
            check_export_paths(&source.exports)?;
            let (settings, common) =
                read_settings(&ctx.tool_settings(crate::TOOL_ID), ctx.common())?;
            let inputs = resolve_inputs(source, settings, common, ctx.paths())?;
            check(inputs)
        }
        TeamCompilerCommand::Compile(args) => {
            refuse_mode(args.mode, args.no_deploy)?;
            check_export_paths(&args.source.exports)?;
            let (settings, common) =
                read_settings(&ctx.tool_settings(crate::TOOL_ID), ctx.common())?;
            let cpk_stem = compile_settings(&settings)?;
            let inputs = resolve_inputs(args.source, settings, common, ctx.paths())?;
            compile(inputs, cpk_stem)
        }
        TeamCompilerCommand::UpgradeDpfl { .. } => Err(invalid(anyhow!(
            "upgrade-dpfl is not available yet in this version"
        ))),
    }
}

fn check(_inputs: RunInputs) -> Result<u8, CliError> {
    Err(CliError::new(ABORTED, anyhow!("check is not built yet")))
}

fn compile(_inputs: RunInputs, _cpk_stem: CpkStem) -> Result<u8, CliError> {
    Err(CliError::new(ABORTED, anyhow!("compile is not built yet")))
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

fn mode_name(mode: Mode) -> &'static str {
    match mode {
        Mode::Normal => "normal",
        Mode::Test => "test",
        Mode::Sider => "sider",
    }
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

fn is_archive(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("zip") || extension.eq_ignore_ascii_case("7z")
        })
}

/// The tool's settings from its table, and the common ones, with the checks both commands share.
fn read_settings(
    table: &toml::Table,
    common: CommonSettings,
) -> Result<(TeamCompilerSettings, CommonSettings), CliError> {
    let settings = from_table(table)
        .map_err(|error| invalid(anyhow!("settings [{}]: {error}", crate::TOOL_ID)))?;
    check_memory_cap(common.memory_cap_percent)?;
    Ok((settings, common))
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

fn resolve_inputs(
    source: SourceArgs,
    settings: TeamCompilerSettings,
    common: CommonSettings,
    paths: &AppPaths,
) -> Result<RunInputs, CliError> {
    let teams_list = load_teams_list(&settings.teams_list_path, paths.data_dir.as_deref())?;
    let exports_root = exports_root(source.exports_root.as_deref(), &common, &paths.exe_dir);
    Ok(RunInputs {
        settings,
        common,
        teams_list,
        exports_root,
        exports: source.exports,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> TeamCompilerCommand {
        let matches = command()
            .try_get_matches_from(std::iter::once("team-compiler").chain(args.iter().copied()))
            .unwrap();
        TeamCompilerCommand::from_arg_matches(&matches).unwrap()
    }

    /// A fresh `<temp>/team_compiler_cli_<name>_<pid>` folder.
    fn scratch(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("team_compiler_cli_{name}_{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        root
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
    fn no_deploy_is_refused_with_either_loose_file_mode_and_kept_with_normal() {
        assert!(refuse_mode(Mode::Normal, true).is_ok());
        assert!(refuse_mode(Mode::Normal, false).is_ok());
        for mode in [Mode::Test, Mode::Sider] {
            let error = refuse_mode(mode, true).unwrap_err();
            assert_eq!(error.exit_code, INVALID);
            assert!(error.to_string().contains("incompatible"), "{error}");
            let error = refuse_mode(mode, false).unwrap_err();
            assert!(error.to_string().contains("not available yet"), "{error}");
        }
    }

    #[test]
    fn archive_extensions_are_case_insensitive() {
        assert!(is_archive(Path::new("a.zip")));
        assert!(is_archive(Path::new("a.ZIP")));
        assert!(is_archive(Path::new("a.7Z")));
        assert!(!is_archive(Path::new("a.rar")));
        assert!(!is_archive(Path::new("zip")));
    }

    #[test]
    fn export_paths_accept_folders_and_archives_only() {
        let root = scratch("export_paths");
        fs::write(root.join("pack.7z"), "").unwrap();
        fs::write(root.join("notes.txt"), "").unwrap();
        assert!(check_export_paths(&[root.clone(), root.join("pack.7z")]).is_ok());
        for refused in [root.join("notes.txt"), root.join("missing.zip")] {
            let error = check_export_paths(std::slice::from_ref(&refused)).unwrap_err();
            assert_eq!(error.exit_code, INVALID);
            assert!(error.to_string().contains(&refused.display().to_string()));
        }
        fs::remove_dir_all(&root).unwrap();
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
        let (settings, _) = read_settings(&table, CommonSettings::default()).unwrap();
        assert!(settings.pass_through);

        let table: toml::Table = toml::from_str("pass_through = 1").unwrap();
        let error = read_settings(&table, CommonSettings::default()).unwrap_err();
        assert_eq!(error.exit_code, INVALID);
        assert!(error.to_string().contains("pass_through"), "{error}");

        let common = CommonSettings {
            memory_cap_percent: 0.0,
            ..CommonSettings::default()
        };
        let error = read_settings(&toml::Table::new(), common).unwrap_err();
        assert!(error.to_string().contains("memory_cap_percent"), "{error}");
    }

    #[test]
    fn compile_refuses_an_invalid_cpk_name_and_multicpk_mode() {
        let mut settings = TeamCompilerSettings::default();
        assert_eq!(compile_settings(&settings).unwrap().as_str(), "4cc_90_test");
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
        let root = scratch("teams_list");
        let relative = Path::new("teams_list.txt");
        assert_eq!(load_teams_list(relative, None).unwrap(), embedded);
        assert_eq!(load_teams_list(relative, Some(&root)).unwrap(), embedded);
        assert!(!root.join("teams_list.txt").exists(), "nothing written");

        fs::write(root.join("teams_list.txt"), "ID\tName\n701\t/egg/\n").unwrap();
        let own = load_teams_list(relative, Some(&root)).unwrap();
        assert_eq!(own.teams().count(), 1);
        // An absolute path is read even with no data directory.
        let absolute = root.join("teams_list.txt");
        assert_eq!(load_teams_list(&absolute, None).unwrap(), own);

        fs::write(root.join("teams_list.txt"), "not a teams list").unwrap();
        let error = load_teams_list(relative, Some(&root)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED);
        assert!(error.to_string().contains("teams_list.txt"), "{error}");

        // A path that exists but cannot be read as a file aborts too.
        fs::remove_file(root.join("teams_list.txt")).unwrap();
        fs::create_dir(root.join("teams_list.txt")).unwrap();
        let error = load_teams_list(relative, Some(&root)).unwrap_err();
        assert_eq!(error.exit_code, ABORTED);
        assert!(error.to_string().contains("cannot read"), "{error}");
        fs::remove_dir_all(&root).unwrap();
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
}
