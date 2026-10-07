//! The working bins taken from the installed CPKs (`team_compiler/pipeline.md` "Bins
//! accumulation"): the bins the game would load from the CPKs `download/DpFileList.bin` lists
//! before the run's own, so a compiled CPK, which the game loads above them, keeps the cup's
//! colors and kits for every team the run does not compile. Each bin comes from the nearest of
//! those CPKs that holds it, and from its bundled base when none does or the walk cannot be
//! made (no PES folder, no list, a list not naming the run's CPK); a Fox player table, which
//! has no bundled base, is then absent. The walk passes over the installed refs CPK when the
//! run writes one, since the run replaces it (`pipeline.md` "5. Writer", step 5). The same
//! walk keeps every entry path of those CPKs, for the texture lookup (`pipeline.md` "Resolved
//! decisions", "A texture a model names must exist"), which `check` makes too, reading no bin.

use std::collections::{BTreeSet, HashSet};
use std::fs::{self, File};
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use anyhow::Context;
use cpk::CpkArchive;
use pes_version::PesVersion;
use pipeline::CpkStem;
use studio_core::{Disposition, Message, Scope};
use uniparam::UniformParameter;

use super::player_tables::{ItemList, ItemTable, read_player_appearance};
use super::{TeamColorBin, UniColorBin, WorkingBins, dpfl};
use crate::messages::{Code, deploy_message, tool_message};
use crate::output::deploy;
use crate::paths;
use crate::templates::Templates;

/// A file of the walk that cannot be read: `installed_bin_unreadable`'s context.
#[derive(Debug)]
pub(crate) struct Unreadable {
    /// The file: the installed `DpFileList.bin`, or the listed CPK.
    pub(crate) path: PathBuf,
    /// What failed, the CPK's bin named when it was one of them.
    pub(crate) error: anyhow::Error,
}

/// The entries of the CPKs the installed `DpFileList.bin` lists before the run's, for the
/// texture lookup (`pipeline.md` "Resolved decisions", "A texture a model names must exist").
#[derive(Debug, PartialEq)]
pub(crate) enum InstalledPaths {
    /// The lookup cannot be made: no PES folder, no `DpFileList.bin`, or a list that does not
    /// name the run's CPK, so nothing is known to come before it.
    Unknown,
    /// Every entry path of those CPKs, folded (`vtree::fold_name`).
    Known(HashSet<String>),
}

impl InstalledPaths {
    /// Whether a CPK holds `path`, compared folded; `None` when the lookup cannot be made.
    pub(crate) fn holds(&self, path: &str) -> Option<bool> {
        match self {
            InstalledPaths::Unknown => None,
            InstalledPaths::Known(paths) => Some(paths.contains(&vtree::fold_name(path))),
        }
    }

    /// The stems, folded, of the textures these CPKs hold in team `team_id`'s Common output
    /// (`paths::common_texture`), which a texture `.common` link may name instead of a file in
    /// the export's `Common/` (`pipeline.md` "Resolved decisions", "A texture a model names must
    /// exist"); empty when the lookup cannot be made.
    pub(crate) fn common_texture_stems(&self, team_id: u16) -> BTreeSet<String> {
        let paths = match self {
            InstalledPaths::Unknown => return BTreeSet::new(),
            InstalledPaths::Known(paths) => paths,
        };
        // The path of the empty stem, cut at the stem, gives the folded head and tail every
        // Common texture path of the team has.
        let empty = vtree::fold_name(&paths::common_texture(team_id, ""));
        let suffix = ".ftex";
        let prefix = empty
            .strip_suffix(suffix)
            .expect("a Common texture path ends with the stem and `.ftex`");
        paths
            .iter()
            .filter_map(|path| path.strip_prefix(prefix)?.strip_suffix(suffix))
            .map(str::to_owned)
            .collect()
    }
}

/// How far the walk got.
enum Walk {
    /// Nothing was walked: the PES folder is not a folder, or its list does not name the run's
    /// CPK, so nothing is known to come before it.
    NotMade,
    /// The PES folder has no list, which should be at this path.
    NoList(PathBuf),
    /// Every CPK listed before the run's was opened; every entry path they hold, folded.
    Walked(HashSet<String>),
}

/// A working bin the walk looks for.
#[derive(Debug, Clone, Copy)]
enum Bin {
    TeamColor,
    UniColor,
    UniformParameter,
    BootsList,
    GloveList,
    PlayerAppearance,
}

impl Bin {
    /// The bin's path in a CPK.
    fn path(self) -> &'static str {
        match self {
            Bin::TeamColor => paths::TEAM_COLOR,
            Bin::UniColor => paths::UNI_COLOR,
            Bin::UniformParameter => paths::UNIFORM_PARAMETER,
            Bin::BootsList => ItemTable::Boots.path(),
            Bin::GloveList => ItemTable::Gloves.path(),
            Bin::PlayerAppearance => paths::PLAYER_APPEARANCE,
        }
    }

    /// The bin's file name, as `bin_source` names it.
    fn name(self) -> &'static str {
        match self {
            Bin::TeamColor => "TeamColor.bin",
            Bin::UniColor => "UniColor.bin",
            Bin::UniformParameter => "UniformParameter.bin",
            Bin::BootsList => ItemTable::Boots.name(),
            Bin::GloveList => ItemTable::Gloves.name(),
            Bin::PlayerAppearance => "PlayerAppearance.bin",
        }
    }

    /// Whether the run has a bundled base for the bin when no installed CPK holds it. The
    /// player tables have none: the seed rows are the cup's own.
    fn has_base(self) -> bool {
        match self {
            Bin::TeamColor | Bin::UniColor | Bin::UniformParameter => true,
            Bin::BootsList | Bin::GloveList | Bin::PlayerAppearance => false,
        }
    }

    /// Parses `bytes`, unwrapped, as the bin and puts it in `bins` in place of the one there.
    fn set(self, bins: &mut WorkingBins, bytes: Vec<u8>) -> anyhow::Result<()> {
        match self {
            Bin::TeamColor => bins.team_color = TeamColorBin::read(bytes)?,
            Bin::UniColor => bins.uni_color = UniColorBin::read(bytes)?,
            Bin::UniformParameter => bins.uniform_parameter = Some(UniformParameter::read(&bytes)?),
            Bin::BootsList => bins.boots_list = Some(ItemList::read(ItemTable::Boots, &bytes)?),
            Bin::GloveList => bins.glove_list = Some(ItemList::read(ItemTable::Gloves, &bytes)?),
            Bin::PlayerAppearance => {
                bins.player_appearance = Some(read_player_appearance(bytes)?);
            }
        }
        Ok(())
    }
}

/// One bin the walk looks for, and what it found.
struct Wanted {
    bin: Bin,
    /// The listed file name of the CPK the bin came from, once a CPK supplied it.
    found: Option<String>,
}

/// The bins a run compiling `cpk_stem` for `version` builds on, taken from the installed
/// CPKs of the PES folder `pes_folder` (`pipeline.md` "Bins accumulation"), passing over the
/// refs CPK `refs` when the run writes one, a bin none of them holds from its bundled base in
/// the run's `templates` (a Fox player table, having none, is then absent), the entry paths of
/// the CPKs walked, and the findings: a `bin_source` per bin found or bundled, and
/// `dpfilelist_missing` when the folder has no list (an Error when the run `deploys`, a
/// Warning when not). A list, a CPK or a bin that cannot be read is the error: the run stops
/// rather than build on an older copy.
pub(crate) fn working_bins(
    pes_folder: &Path,
    cpk_stem: &CpkStem,
    refs: Option<&CpkStem>,
    version: PesVersion,
    deploys: bool,
    templates: &Templates,
) -> Result<(WorkingBins, InstalledPaths, Vec<Message>), Unreadable> {
    let mut looked_for = vec![Bin::TeamColor, Bin::UniColor];
    // Only the Fox versions have the bin, and so a bundled base for it; and only they have
    // the player tables.
    if templates.uniform_parameter_base(version).is_some() {
        looked_for.extend([
            Bin::UniformParameter,
            Bin::BootsList,
            Bin::GloveList,
            Bin::PlayerAppearance,
        ]);
    }
    let mut wanted: Vec<Wanted> = looked_for
        .into_iter()
        .map(|bin| Wanted { bin, found: None })
        .collect();
    let mut messages = Vec::new();
    let mut bins = WorkingBins::bundled(version, templates);
    let walked = walk(pes_folder, cpk_stem, refs, |cpk, name| {
        take_bins(cpk, name, &mut wanted, &mut bins)
    })?;
    let installed = match walked {
        Walk::NotMade => InstalledPaths::Unknown,
        Walk::NoList(list) => {
            messages.push(deploy_message(
                Code::DpfilelistMissing,
                Scope::Run,
                Disposition::Keep,
                vec![("path", list.display().to_string())],
                deploys,
            ));
            InstalledPaths::Unknown
        }
        Walk::Walked(paths) => InstalledPaths::Known(paths),
    };
    for Wanted { bin, found } in wanted {
        let cpk = match found {
            Some(cpk) => cpk,
            None if bin.has_base() => "bundled".to_owned(),
            // No base to name: the table is not written.
            None => continue,
        };
        messages.push(tool_message(
            Code::BinSource,
            Scope::Run,
            Disposition::Keep,
            vec![("bin", bin.name().to_owned()), ("cpk", cpk)],
        ));
    }
    Ok((bins, installed, messages))
}

/// The entry paths of the installed CPKs listed before `cpk_name`'s but the refs CPK
/// `refs_name`, when given, for `check`'s texture links: the walk `compile` makes, reading each
/// CPK's table of contents and no bin, so `check` and `compile` agree on a link. `check`
/// reports nothing about the walk: a name that is no valid CPK name, a missing list, or a list
/// or CPK that cannot be read gives `Unknown`, and `compile` reports them.
pub(crate) fn installed_paths(
    pes_folder: &Path,
    cpk_name: &str,
    refs_name: Option<&str>,
) -> InstalledPaths {
    let named = CpkStem::new(cpk_name).and_then(|cpk_stem| {
        let refs = refs_name.map(CpkStem::new).transpose()?;
        Ok((cpk_stem, refs))
    });
    let (cpk_stem, refs) = match named {
        Ok(named) => named,
        Err(error) => {
            log::debug!("no installed CPK looked in: {cpk_name:?}, {refs_name:?}: {error}");
            return InstalledPaths::Unknown;
        }
    };
    match walk(pes_folder, &cpk_stem, refs.as_ref(), |_, _| Ok(())) {
        Ok(Walk::Walked(paths)) => InstalledPaths::Known(paths),
        Ok(Walk::NotMade | Walk::NoList(_)) => InstalledPaths::Unknown,
        Err(unreadable) => {
            log::debug!(
                "no installed CPK looked in: {}: {:#}",
                unreadable.path.display(),
                unreadable.error
            );
            InstalledPaths::Unknown
        }
    }
}

/// An installed CPK, opened on its file.
type InstalledCpk = CpkArchive<BufReader<File>>;

/// Walks every CPK `pes_folder`'s `download/DpFileList.bin` lists before `cpk_stem`'s, nearest
/// first, keeping every entry path of each, folded, and calling `take` on each with its listed
/// name: the texture lookup needs every path, so the walk does not stop once the bins are
/// found. A listed CPK with no file is passed over, and so is the refs CPK `refs` when given;
/// one that cannot be opened, or that `take` fails on, is the error, naming that CPK.
fn walk(
    pes_folder: &Path,
    cpk_stem: &CpkStem,
    refs: Option<&CpkStem>,
    mut take: impl FnMut(&mut InstalledCpk, &str) -> anyhow::Result<()>,
) -> Result<Walk, Unreadable> {
    if !pes_folder.is_dir() {
        return Ok(Walk::NotMade);
    }
    let download = pes_folder.join("download");
    let list_path = download.join("DpFileList.bin");
    let Some(list) = read_list(&list_path)? else {
        return Ok(Walk::NoList(list_path));
    };
    let own = deploy::cpk_file_name(cpk_stem);
    let Some(position) = list.iter().position(|name| *name == own) else {
        return Ok(Walk::NotMade);
    };
    let refs = refs.map(deploy::cpk_file_name);
    let mut paths = HashSet::new();
    for name in list[..position].iter().rev() {
        // The run replaces the installed refs CPK, so nothing in it is built on or looked up:
        // a refs export's texture link is not satisfied by the referees it replaces.
        if refs.as_ref() == Some(name) {
            continue;
        }
        let path = download.join(name);
        let Some(mut cpk) = open_cpk(&path)? else {
            continue;
        };
        paths.extend(
            cpk.entries()
                .iter()
                .map(|entry| vtree::fold_name(&entry.path)),
        );
        take(&mut cpk, name).map_err(|error| Unreadable { path, error })?;
    }
    Ok(Walk::Walked(paths))
}

/// The CPK names the list at `path` gives, in load order; `None` when there is no file.
pub(crate) fn read_list(path: &Path) -> Result<Option<Vec<String>>, Unreadable> {
    let unreadable = |error: anyhow::Error| Unreadable {
        path: path.to_owned(),
        error,
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable(error.into())),
    };
    dpfl::entries(&bytes).map(Some).map_err(unreadable)
}

/// The CPK at `path`, opened; `None` when there is no file.
fn open_cpk(path: &Path) -> Result<Option<InstalledCpk>, Unreadable> {
    let unreadable = |error: anyhow::Error| Unreadable {
        path: path.to_owned(),
        error,
    };
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(unreadable(error.into())),
    };
    CpkArchive::open(BufReader::new(file))
        .context("not a CPK the reader accepts")
        .map(Some)
        .map_err(unreadable)
}

/// Takes into `bins` from `cpk`, listed as `name`, each of `wanted` not found yet that it
/// holds, unwrapped when the bin is WESYS-compressed, and parsed: a bin that does not parse as
/// its format is as unreadable as one that cannot be read, found here rather than when the CPK
/// is finished after every export was processed.
fn take_bins(
    cpk: &mut InstalledCpk,
    name: &str,
    wanted: &mut [Wanted],
    bins: &mut WorkingBins,
) -> anyhow::Result<()> {
    for wanted in wanted.iter_mut().filter(|wanted| wanted.found.is_none()) {
        let path_in_cpk = wanted.bin.path();
        let Some(entry) = cpk.entries().iter().find(|entry| entry.path == path_in_cpk) else {
            continue;
        };
        let entry = entry.clone();
        let bytes = cpk
            .read(&entry)
            .with_context(|| format!("cannot read {path_in_cpk}"))?;
        let bytes = wezlib::decompress_if_wrapped(&bytes)
            .with_context(|| format!("cannot unwrap {path_in_cpk}"))?
            .into_owned();
        wanted
            .bin
            .set(bins, bytes)
            .with_context(|| format!("cannot parse {path_in_cpk}"))?;
        wanted.found = Some(name.to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use cpk::CpkWriter;
    use studio_core::Severity;

    use super::*;
    use crate::testing::{dpfilelist, scratch};

    /// The stem of the run's CPK in every test: `4cc_99_test`.
    fn stem() -> CpkStem {
        CpkStem::new("4cc_99_test").unwrap()
    }

    /// Writes `download/DpFileList.bin` listing `names` in the PES folder `pes`.
    fn install_list(pes: &Path, names: &[&str]) {
        fs::create_dir_all(pes.join("download")).unwrap();
        fs::write(pes.join("download/DpFileList.bin"), dpfilelist(names)).unwrap();
    }

    /// Writes the CPK `download/<name>` holding `entries` (CPK path, bytes) in the PES folder
    /// `pes`.
    fn install_cpk(pes: &Path, name: &str, entries: &[(&str, &[u8])]) {
        let file = File::create(pes.join("download").join(name)).unwrap();
        let mut cpk = CpkWriter::new(file, "test").unwrap();
        for (path, bytes) in entries {
            cpk.add(path, bytes, None).unwrap();
        }
        cpk.finish().unwrap();
    }

    /// The `bin_source` finding naming `bin` and `cpk`.
    fn source(bin: &str, cpk: &str) -> Message {
        tool_message(
            Code::BinSource,
            Scope::Run,
            Disposition::Keep,
            vec![("bin", bin.to_owned()), ("cpk", cpk.to_owned())],
        )
    }

    /// The three `bin_source` findings of a PES 21 run built on the bundled bases.
    fn all_bundled() -> [Message; 3] {
        [
            source("TeamColor.bin", "bundled"),
            source("UniColor.bin", "bundled"),
            source("UniformParameter.bin", "bundled"),
        ]
    }

    /// Asserts that `bins` are the bundled bases of `version`.
    fn assert_bundled(bins: &WorkingBins, version: PesVersion) {
        let bundled = WorkingBins::bundled(version, &Templates::embedded());
        assert!(
            bins.team_color == bundled.team_color,
            "TeamColor.bin bundled"
        );
        assert!(bins.uni_color == bundled.uni_color, "UniColor.bin bundled");
        assert!(
            written(bins) == written(&bundled),
            "UniformParameter.bin bundled"
        );
    }

    /// `bins`' `UniformParameter.bin` as it would be written.
    fn written(bins: &WorkingBins) -> Option<Vec<u8>> {
        bins.uniform_parameter.as_ref().map(UniformParameter::write)
    }

    /// A `TeamColor.bin` of team 100's one record, its colors all `color`.
    fn team_color_bin(color: u8) -> Vec<u8> {
        let mut bytes = vec![0x64, 0x00, 0x04, 0x00];
        bytes.extend([color; 12]);
        bytes
    }

    /// A `UniColor.bin` of team 100's one record holding the one kit 0, its colors all `color`.
    fn uni_color_bin(color: u8) -> Vec<u8> {
        let mut bytes = vec![0x64, 0x00, 0x00, 0x00, 0x01];
        bytes.extend([0x00, 0x03, color, color, color, color, color, color]);
        for _ in 1..10 {
            bytes.extend([0xff, 0, 0, 0, 0, 0, 0, 0]);
        }
        bytes
    }

    /// A `UniformParameter.bin` holding the one entry `name`.
    fn uniform_parameter_bin(name: &str) -> Vec<u8> {
        let mut bin = UniformParameter::new();
        bin.insert(name.to_owned(), vec![7; 120]).unwrap();
        bin.write()
    }

    #[test]
    fn with_no_pes_folder_every_bin_is_bundled() {
        let temp = scratch("installed_no_pes");
        let pes = temp.path().join("PES");

        let (bins, _, messages) = working_bins(
            &pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_bundled(&bins, PesVersion::Pes21);
        assert_eq!(messages, all_bundled());

        let (bins, _, messages) = working_bins(
            &pes,
            &stem(),
            None,
            PesVersion::Pes17,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_bundled(&bins, PesVersion::Pes17);
        assert!(bins.uniform_parameter.is_none());
        assert_eq!(messages, all_bundled()[..2]);
    }

    #[test]
    fn a_missing_list_is_an_error_when_the_run_deploys_and_a_warning_when_not() {
        let temp = scratch("installed_no_list");
        let pes = temp.path();
        let list = pes.join("download").join("DpFileList.bin");
        for (deploys, severity) in [(true, Severity::Error), (false, Severity::Warning)] {
            let (bins, _, messages) = working_bins(
                pes,
                &stem(),
                None,
                PesVersion::Pes21,
                deploys,
                &Templates::embedded(),
            )
            .unwrap();
            assert_bundled(&bins, PesVersion::Pes21);
            let missing = deploy_message(
                Code::DpfilelistMissing,
                Scope::Run,
                Disposition::Keep,
                vec![("path", list.display().to_string())],
                deploys,
            );
            assert_eq!(missing.severity, severity);
            let mut expected = vec![missing];
            expected.extend(all_bundled());
            assert_eq!(messages, expected, "deploys: {deploys}");
        }
    }

    #[test]
    fn a_list_not_naming_the_run_s_cpk_gives_the_bundled_bins() {
        let temp = scratch("installed_unlisted");
        let pes = temp.path();
        install_list(pes, &["4cc_08_bins.cpk", "4cc_61_midcup.cpk"]);
        install_cpk(pes, "4cc_61_midcup.cpk", &[(paths::UNI_COLOR, b"midcup")]);

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_bundled(&bins, PesVersion::Pes21);
        assert_eq!(messages, all_bundled());
    }

    #[test]
    fn the_installed_paths_are_every_entry_of_each_cpk_listed_before_the_run_s_own() {
        let temp = scratch("installed_paths");
        let pes = temp.path();
        install_list(
            pes,
            &[
                "4cc_08_bins.cpk",
                "4cc_61_midcup.cpk",
                "4cc_99_test.cpk",
                "4cc_63_midcup.cpk",
            ],
        );
        let hair = paths::common_texture(714, "Hair");
        install_cpk(pes, "4cc_08_bins.cpk", &[(&hair, b"farther")]);
        // The nearer CPK holds every bin the walk looks for: the walk still opens the farther.
        let pair = [0x39, 0x12, 0x01, 0x00, 0x0b, 0x00, 0x00, 0x00];
        let bins_61: [(&str, &[u8]); 6] = [
            (paths::TEAM_COLOR, &team_color_bin(61)),
            (paths::UNI_COLOR, &uni_color_bin(61)),
            (paths::UNIFORM_PARAMETER, &uniform_parameter_bin("61")),
            (paths::BOOTS_LIST, &pair),
            (paths::GLOVE_LIST, &pair),
            (paths::PLAYER_APPEARANCE, &[7; 60]),
        ];
        install_cpk(pes, "4cc_61_midcup.cpk", &bins_61);
        let after = paths::common_texture(714, "after");
        install_cpk(pes, "4cc_63_midcup.cpk", &[(&after, b"after")]);

        let (_, installed, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        let every_bin = [
            "TeamColor.bin",
            "UniColor.bin",
            "UniformParameter.bin",
            "BootsList.bin",
            "GloveList.bin",
            "PlayerAppearance.bin",
        ];
        assert_eq!(
            messages,
            every_bin.map(|bin| source(bin, "4cc_61_midcup.cpk")),
            "every bin from the nearer CPK"
        );
        let mut expected: HashSet<String> = bins_61
            .iter()
            .map(|(path, _)| vtree::fold_name(path))
            .collect();
        expected.insert(vtree::fold_name(&hair));
        assert_eq!(installed, InstalledPaths::Known(expected));
        assert_eq!(
            installed.holds(&paths::common_texture(714, "hair")),
            Some(true),
            "Hair.ftex installed, hair.ftex asked"
        );
        assert_eq!(
            installed.holds(&after),
            Some(false),
            "listed after the run's"
        );
    }

    #[test]
    fn the_lookup_cannot_be_made_with_no_pes_folder_no_list_or_a_list_not_naming_the_run_s_cpk() {
        let temp = scratch("installed_paths_unknown");
        let pes = temp.path();
        let walk = |pes: &Path| {
            working_bins(
                pes,
                &stem(),
                None,
                PesVersion::Pes21,
                false,
                &Templates::embedded(),
            )
            .unwrap()
            .1
        };
        let hair = paths::common_texture(714, "hair");

        let installed = walk(&pes.join("PES"));
        assert_eq!(installed, InstalledPaths::Unknown, "no PES folder");
        assert_eq!(installed.holds(&hair), None);
        assert_eq!(walk(pes), InstalledPaths::Unknown, "no list");
        install_list(pes, &["4cc_08_bins.cpk"]);
        install_cpk(pes, "4cc_08_bins.cpk", &[(&hair, b"hair")]);
        assert_eq!(
            walk(pes),
            InstalledPaths::Unknown,
            "the run's CPK not listed"
        );
        // Listed first, the run's CPK has nothing before it: the lookup is made and finds
        // nothing.
        install_list(pes, &["4cc_99_test.cpk", "4cc_08_bins.cpk"]);
        let installed = walk(pes);
        assert_eq!(installed, InstalledPaths::Known(HashSet::new()));
        assert_eq!(installed.holds(&hair), Some(false));
    }

    #[test]
    fn the_common_texture_stems_are_the_team_s_own_common_output_s_folded() {
        let installed = InstalledPaths::Known(
            [
                paths::common_texture(714, "Hair"),
                paths::common_texture(702, "skin"),
                // The team's Common folder, outside `sourceimages/#windx11/`.
                "Asset/model/character/common/714/sourceimages/cloth.ftex".to_owned(),
                // A player's own common subfolder.
                "Asset/model/character/common/714/05 - A/sourceimages/#windx11/shirt.ftex"
                    .to_owned(),
            ]
            .iter()
            .map(|path| vtree::fold_name(path))
            .collect(),
        );
        assert_eq!(
            installed.common_texture_stems(714),
            BTreeSet::from(["hair".to_owned()])
        );
        assert_eq!(
            installed.common_texture_stems(702),
            BTreeSet::from(["skin".to_owned()])
        );
        assert_eq!(
            InstalledPaths::Unknown.common_texture_stems(714),
            BTreeSet::new()
        );
    }

    #[test]
    fn the_check_walk_reads_the_tables_of_contents_and_no_bin() {
        let temp = scratch("installed_check_walk");
        let pes = temp.path();
        install_list(
            pes,
            &[
                "4cc_08_bins.cpk",
                "4cc_61_midcup.cpk",
                "4cc_99_test.cpk",
                "4cc_63_midcup.cpk",
            ],
        );
        // One byte short of a whole 85-byte record: `compile`'s walk stops on it.
        install_cpk(pes, "4cc_08_bins.cpk", &[(paths::UNI_COLOR, &[0; 84])]);
        let hair = paths::common_texture(714, "hair");
        install_cpk(pes, "4cc_61_midcup.cpk", &[(&hair, b"hair")]);
        let after = paths::common_texture(714, "after");
        install_cpk(pes, "4cc_63_midcup.cpk", &[(&after, b"after")]);

        assert_eq!(
            installed_paths(pes, "4cc_99_test", None),
            InstalledPaths::Known(HashSet::from([
                vtree::fold_name(paths::UNI_COLOR),
                vtree::fold_name(&hair),
            ]))
        );
    }

    #[test]
    fn the_walk_passes_over_the_refs_cpk_only_when_one_is_named() {
        let temp = scratch("installed_walk_refs");
        let pes = temp.path();
        install_list(
            pes,
            &["4cc_08_bins.cpk", "4cc_18_referees.cpk", "4cc_99_test.cpk"],
        );
        let hair = paths::common_texture(714, "hair");
        install_cpk(pes, "4cc_08_bins.cpk", &[(&hair, b"hair")]);
        let skin = paths::common_texture(999, "skin");
        install_cpk(
            pes,
            "4cc_18_referees.cpk",
            &[(&skin, b"skin"), (paths::TEAM_COLOR, &team_color_bin(1))],
        );

        assert_eq!(
            installed_paths(pes, "4cc_99_test", Some("4cc_18_referees")),
            InstalledPaths::Known(HashSet::from([vtree::fold_name(&hair)]))
        );
        assert_eq!(
            installed_paths(pes, "4cc_99_test", None),
            InstalledPaths::Known(HashSet::from([
                vtree::fold_name(&hair),
                vtree::fold_name(&skin),
                vtree::fold_name(paths::TEAM_COLOR),
            ]))
        );
        assert_eq!(
            installed_paths(pes, "4cc_99_test", Some("con")),
            InstalledPaths::Unknown,
            "no valid CPK name"
        );
        // `compile`'s walk takes no bin from it either.
        let refs = CpkStem::new("4cc_18_referees").unwrap();
        let (_, _, messages) = working_bins(
            pes,
            &stem(),
            Some(&refs),
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_eq!(messages, all_bundled());
    }

    #[test]
    fn the_check_walk_is_unknown_wherever_compile_s_would_report_or_stop() {
        let temp = scratch("installed_check_walk_unknown");
        let pes = temp.path();
        assert_eq!(
            installed_paths(&pes.join("PES"), "4cc_99_test", None),
            InstalledPaths::Unknown,
            "no PES folder"
        );
        assert_eq!(
            installed_paths(pes, "4cc_99_test", None),
            InstalledPaths::Unknown,
            "no list"
        );
        install_list(pes, &["4cc_61_midcup.cpk", "4cc_99_test.cpk"]);
        assert_eq!(
            installed_paths(pes, "4cc_99_test", None),
            InstalledPaths::Known(HashSet::new()),
            "a listed CPK with no file is passed over"
        );
        assert_eq!(
            installed_paths(pes, "con", None),
            InstalledPaths::Unknown,
            "no valid CPK name"
        );
        // Opening a directory fails with `PermissionDenied` on Windows.
        let cpk = pes.join("download").join("4cc_61_midcup.cpk");
        #[cfg(windows)]
        fs::create_dir(&cpk).unwrap();
        // Opening a directory succeeds on Unix, opening a link to itself fails (too many links).
        #[cfg(unix)]
        std::os::unix::fs::symlink(&cpk, &cpk).unwrap();
        assert_eq!(
            installed_paths(pes, "4cc_99_test", None),
            InstalledPaths::Unknown,
            "a CPK that cannot be opened"
        );
    }

    #[test]
    fn the_run_s_cpk_listed_first_gives_the_bundled_bins() {
        let temp = scratch("installed_first");
        let pes = temp.path();
        install_list(pes, &["4cc_99_test.cpk", "4cc_61_midcup.cpk"]);
        install_cpk(pes, "4cc_61_midcup.cpk", &[(paths::UNI_COLOR, b"midcup")]);
        install_cpk(pes, "4cc_99_test.cpk", &[(paths::UNI_COLOR, b"own")]);

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_bundled(&bins, PesVersion::Pes21);
        assert_eq!(messages, all_bundled());
    }

    #[test]
    fn a_listed_cpk_with_no_file_is_passed_over_and_the_next_one_s_bin_taken() {
        let temp = scratch("installed_passed_over");
        let pes = temp.path();
        install_list(
            pes,
            &[
                "4cc_08_bins.cpk",
                "4cc_40_teams.cpk",
                "4cc_61_midcup.cpk",
                "4cc_99_test.cpk",
            ],
        );
        install_cpk(
            pes,
            "4cc_08_bins.cpk",
            &[
                (paths::TEAM_COLOR, &team_color_bin(8)),
                (paths::UNI_COLOR, &uni_color_bin(8)),
            ],
        );
        install_cpk(
            pes,
            "4cc_40_teams.cpk",
            &[(paths::UNI_COLOR, &uni_color_bin(40))],
        );

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_eq!(bins.team_color.into_bytes(), team_color_bin(8));
        assert_eq!(bins.uni_color.into_bytes(), uni_color_bin(40));
        assert!(
            bins.uniform_parameter.map(|bin| bin.write())
                == written(&WorkingBins::bundled(
                    PesVersion::Pes21,
                    &Templates::embedded()
                ))
        );
        assert_eq!(
            messages,
            [
                source("TeamColor.bin", "4cc_08_bins.cpk"),
                source("UniColor.bin", "4cc_40_teams.cpk"),
                source("UniformParameter.bin", "bundled"),
            ]
        );
    }

    #[test]
    fn an_installed_uniform_parameter_bin_is_taken_on_pes_21_and_not_looked_for_on_pes_17() {
        let temp = scratch("installed_uniform_parameter");
        let pes = temp.path();
        install_list(pes, &["4cc_08_bins.cpk", "4cc_99_test.cpk"]);
        install_cpk(
            pes,
            "4cc_08_bins.cpk",
            &[(
                paths::UNIFORM_PARAMETER,
                &uniform_parameter_bin("kit configs of 08"),
            )],
        );

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_eq!(
            written(&bins),
            Some(uniform_parameter_bin("kit configs of 08"))
        );
        assert_eq!(
            messages[2],
            source("UniformParameter.bin", "4cc_08_bins.cpk")
        );

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes17,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert!(bins.uniform_parameter.is_none());
        assert_eq!(messages, all_bundled()[..2]);
    }

    #[test]
    fn an_installed_bin_that_does_not_parse_is_the_error_naming_its_cpk() {
        let temp = scratch("installed_bin_unparsable");
        let pes = temp.path();
        install_list(pes, &["4cc_08_bins.cpk", "4cc_99_test.cpk"]);
        // One byte short of a whole 85-byte record.
        install_cpk(pes, "4cc_08_bins.cpk", &[(paths::UNI_COLOR, &[0; 84])]);

        let Err(unreadable) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        ) else {
            panic!("a bin that does not parse must be the error");
        };
        assert_eq!(
            unreadable.path,
            pes.join("download").join("4cc_08_bins.cpk")
        );
        assert_eq!(
            format!("{:#}", unreadable.error),
            format!(
                "cannot parse {}: UniColor.bin is 84 bytes, not a whole number of 85-byte records",
                paths::UNI_COLOR
            )
        );
    }

    #[test]
    fn the_player_tables_are_taken_on_pes_21_and_not_looked_for_on_pes_17() {
        let temp = scratch("installed_player_tables");
        let pes = temp.path();
        install_list(pes, &["4cc_08_bins.cpk", "4cc_99_test.cpk"]);
        // Player 70201 with boots 11; one 60-byte appearance row.
        let boots = [0x39, 0x12, 0x01, 0x00, 0x0b, 0x00, 0x00, 0x00];
        install_cpk(
            pes,
            "4cc_08_bins.cpk",
            &[
                (paths::BOOTS_LIST, &boots),
                (paths::PLAYER_APPEARANCE, &[7; 60]),
            ],
        );

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_eq!(
            bins.boots_list.map(ItemList::into_bytes),
            Some(boots.to_vec())
        );
        assert_eq!(bins.glove_list, None, "no CPK holds it");
        assert_eq!(bins.player_appearance, Some(vec![7; 60]));
        let mut expected = all_bundled().to_vec();
        expected.extend([
            source("BootsList.bin", "4cc_08_bins.cpk"),
            source("PlayerAppearance.bin", "4cc_08_bins.cpk"),
        ]);
        assert_eq!(messages, expected, "no bin_source for GloveList.bin");

        let (bins, _, messages) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes17,
            true,
            &Templates::embedded(),
        )
        .unwrap();
        assert_eq!(bins.boots_list, None);
        assert_eq!(bins.player_appearance, None);
        assert_eq!(messages, all_bundled()[..2]);
    }

    #[test]
    fn an_installed_player_table_that_does_not_parse_is_the_error_naming_its_cpk() {
        for (name, path, bytes, error) in [
            (
                "installed_glove_list_unparsable",
                paths::GLOVE_LIST,
                vec![0; 12],
                "GloveList.bin is 12 bytes, not a whole number of 8-byte pairs",
            ),
            (
                "installed_player_appearance_unparsable",
                paths::PLAYER_APPEARANCE,
                vec![0; 61],
                "PlayerAppearance.bin is 61 bytes, not a whole number of 60-byte rows",
            ),
        ] {
            let temp = scratch(name);
            let pes = temp.path();
            install_list(pes, &["4cc_08_bins.cpk", "4cc_99_test.cpk"]);
            install_cpk(pes, "4cc_08_bins.cpk", &[(path, &bytes)]);

            let Err(unreadable) = working_bins(
                pes,
                &stem(),
                None,
                PesVersion::Pes21,
                true,
                &Templates::embedded(),
            ) else {
                panic!("a table that does not parse must be the error");
            };
            assert_eq!(
                unreadable.path,
                pes.join("download").join("4cc_08_bins.cpk")
            );
            assert_eq!(
                format!("{:#}", unreadable.error),
                format!("cannot parse {path}: {error}")
            );
        }
    }

    #[test]
    fn a_list_that_exists_but_cannot_be_read_is_the_error() {
        let temp = scratch("installed_list_unreadable");
        let pes = temp.path();
        let list = pes.join("download").join("DpFileList.bin");
        fs::create_dir_all(&list).unwrap();

        let Err(unreadable) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        ) else {
            panic!("a list that cannot be read must be the error, not a missing list");
        };
        assert_eq!(unreadable.path, list);
    }

    #[test]
    fn a_listed_cpk_that_exists_but_cannot_be_opened_is_the_error() {
        let temp = scratch("installed_cpk_unopenable");
        let pes = temp.path();
        install_list(pes, &["4cc_61_midcup.cpk", "4cc_99_test.cpk"]);
        let cpk = pes.join("download").join("4cc_61_midcup.cpk");
        // Opening a directory fails with `PermissionDenied` on Windows.
        #[cfg(windows)]
        fs::create_dir(&cpk).unwrap();
        // Opening a directory succeeds on Unix, opening a link to itself fails (too many links).
        #[cfg(unix)]
        std::os::unix::fs::symlink(&cpk, &cpk).unwrap();

        let Err(unreadable) = working_bins(
            pes,
            &stem(),
            None,
            PesVersion::Pes21,
            true,
            &Templates::embedded(),
        ) else {
            panic!("a CPK that cannot be opened must be the error, not a CPK with no file");
        };
        assert_eq!(unreadable.path, cpk);
    }
}
