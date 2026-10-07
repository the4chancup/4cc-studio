//! The canonical-order CPK writer (`team_compiler/pipeline.md` "5. Writer"): the `overrides/`
//! files first, then task batches in manifest order whatever order they arrive in, a player
//! folder's group decided as one once its textures batch is in, then the bins.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use aesthetics_export::ExportCoverage;
use anyhow::{Context, ensure};
use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};

use crate::bins::kit_configs::kit_configs;
use crate::bins::player_tables::{ItemList, ItemTable, table_missing};
use crate::bins::{KitColorEntry, Rgb, WorkingBins, kit_number};
use crate::messages::{Code, tool_message};
use crate::output::sink::OutputSink;
use crate::paths;
use crate::plan::TeamKits;
use crate::plan::item_rows::ItemRow;
use crate::processing::TaskBatch;

/// One CPK's entries being written into `sink`, a CPK file or a folder of loose files. The
/// overrides go in with the first committed entry, or at the end when only they go in, so a
/// run that commits nothing and has no override adds nothing to the sink, which then creates
/// no file and no folder.
pub(crate) struct CpkOutput {
    /// Where the entries are written.
    sink: OutputSink,
    /// Whether the overrides are in the sink: they are its first entries.
    started: bool,
    /// The `overrides/` files, by CPK path: the first entries of the CPK, and the paths no task
    /// or bin may take.
    overrides: BTreeMap<String, PathBuf>,
    /// The manifest position of the next batch to commit.
    next: usize,
    /// Batches that arrived before an earlier one, or a player folder's packages held for its
    /// textures batch, by manifest position.
    pending: BTreeMap<usize, TaskBatch>,
    /// The committed kits' `UniformParameter.bin` entries.
    uniform_parameters: Vec<(String, Vec<u8>)>,
    /// The committed kits' `UniColor.bin` entries, each with its team ID, in commit order.
    kit_colors: Vec<(u16, KitColorEntry)>,
    /// The manifest positions of the batches committed: a player's boots or gloves row is set
    /// only when the task building them is among them.
    committed: BTreeSet<usize>,
    /// What goes before each bin's game path in the sink: empty, or test mode's `_bins/`.
    bins_prefix: String,
}

impl CpkOutput {
    /// A CPK's entries to be written into `sink`, starting with `overrides` (CPK path, file on
    /// disk), each file read when it is added, the bins at their game paths after
    /// `bins_prefix`; nothing is written yet.
    pub(crate) fn new(
        sink: OutputSink,
        overrides: BTreeMap<String, PathBuf>,
        bins_prefix: &str,
    ) -> CpkOutput {
        CpkOutput {
            sink,
            started: false,
            overrides,
            next: 0,
            pending: BTreeMap::new(),
            uniform_parameters: Vec::new(),
            kit_colors: Vec::new(),
            committed: BTreeSet::new(),
            bins_prefix: bins_prefix.to_owned(),
        }
    }

    /// Takes `batch` and commits every batch that is now next in manifest order. Committing in
    /// manifest order, not arrival order, is what makes the CPK's layout the same on every run.
    /// A player folder's group (`TaskBatch::group`) is decided only once every batch of it is
    /// in: its packages wait, their memory still charged, for the textures batch that decides
    /// them. Returns the batches decided, in manifest order, as each one's manifest position
    /// and messages (a `duplicate_path` for each entry an override replaced among them), so
    /// the caller reports findings in manifest order too; empty when `batch` waits for an
    /// earlier one.
    pub(crate) fn submit(
        &mut self,
        batch: TaskBatch,
    ) -> anyhow::Result<Vec<(usize, Vec<Message>)>> {
        self.pending.insert(batch.index, batch);
        let mut committed = Vec::new();
        while let Some(next) = self.pending.get(&self.next) {
            let group = next.group.clone();
            let range = group.clone().unwrap_or(self.next..self.next + 1);
            if !range.clone().all(|index| self.pending.contains_key(&index)) {
                break;
            }
            let mut batches: Vec<TaskBatch> = range
                .clone()
                .map(|index| {
                    self.pending
                        .remove(&index)
                        .expect("every position was checked")
                })
                .collect();
            let result = match group {
                Some(_) => self.commit_folder(&mut batches),
                None => batches.iter_mut().try_for_each(|batch| self.commit(batch)),
            };
            if let Err(error) = result {
                // The CPK is lost, so the batches waiting here go now: their permits may be
                // what the coordinator is waiting for, and it must reach the end of the run.
                self.pending.clear();
                return Err(error);
            }
            for batch in batches {
                committed.push((batch.index, batch.messages));
            }
            self.next = range.end;
        }
        Ok(committed)
    }

    /// Decides a player folder's group, `batches` in manifest order with the textures batch
    /// last. When the textures failed nothing of the folder commits: a package in the CPK
    /// would point at textures that are not. Otherwise each package that succeeded commits,
    /// then the textures, when at least one package did. A package the textures batch names
    /// as the loser of a `shared_texture_conflict` (`TaskBatch::skipped`) is left out the same
    /// way: the textures only its sources hold are not in the CPK.
    fn commit_folder(&mut self, batches: &mut [TaskBatch]) -> anyhow::Result<()> {
        let (textures, packages) = batches
            .split_last_mut()
            .expect("a player folder's group ends with its textures batch");
        if textures.entries.is_empty() {
            return Ok(());
        }
        let mut committed = false;
        for package in packages {
            if textures.skipped.contains(&package.index) {
                continue;
            }
            committed |= !package.entries.is_empty();
            self.commit(package)?;
        }
        if committed {
            self.commit(textures)?;
        }
        Ok(())
    }

    /// Writes the bins and closes the CPK, when a batch committed something or there is an
    /// override (else no file exists, no bin is built and nothing is reported):
    /// `UniformParameter.bin`, built on `bins`' by `kit_configs` from `team_kits` and the
    /// committed kit configs, when that changed it (a committed kit config on a version without
    /// the bin, PES 15-17, is an error); then `TeamColor.bin`, built on `bins`' with every
    /// record's header set from its position and each of `team_colors` (team id, colors) set in
    /// its team's record; then `UniColor.bin`, built on `bins`' the same way, each `Full`
    /// export's team of `team_kits` keeping only its kit tasks' kits in its record, then each
    /// committed kit's entry merged into its team's record in commit order; then the Fox player
    /// tables `bins` holds, with `item_rows` applied (`add_player_tables`). Returns whether a
    /// CPK was written and the findings to report: `kit_configs`' FPC findings, one
    /// `bin_header_repaired` per working bin that had a header wrong, naming the teams, a
    /// `player_table_missing` per list not found that committed rows were left out of, and a
    /// `duplicate_path` for each bin an override replaced.
    pub(crate) fn finish(
        mut self,
        version: PesVersion,
        bins: WorkingBins,
        team_colors: &[(u16, Vec<Rgb>)],
        team_kits: &[TeamKits],
        item_rows: &[ItemRow],
    ) -> anyhow::Result<(bool, Vec<Message>)> {
        ensure!(
            self.pending.is_empty(),
            "the writer never received task {} of the manifest",
            self.next
        );
        // A run that committed nothing writes no file, so it adds no bin either, unless the
        // overrides go into the CPK: they are written whatever the exports bring.
        if !self.started {
            if self.overrides.is_empty() {
                return Ok((false, Vec::new()));
            }
            self.start()?;
        }
        let WorkingBins {
            team_color,
            uni_color,
            uniform_parameter,
            boots_list,
            glove_list,
            player_appearance,
        } = bins;
        let mut messages = Vec::new();
        // The bins hold what every committed kit contributed, so they are built only once
        // every batch is in, and go last.
        let committed_configs = std::mem::take(&mut self.uniform_parameters);
        match uniform_parameter {
            Some(mut bin) => {
                // Before `UniColor.bin` is edited below: the kits an absent slot is patched
                // for are those the team's working record holds.
                let (changed, findings) =
                    kit_configs(&mut bin, &uni_color, team_kits, committed_configs, version)?;
                messages.extend(findings);
                if changed {
                    self.add_bin(paths::UNIFORM_PARAMETER, &bin.write(), &mut messages)?;
                }
            }
            None => ensure!(
                committed_configs.is_empty(),
                "{version} has no UniformParameter.bin"
            ),
        }
        let mut bin = team_color;
        messages.extend(header_repaired("TeamColor.bin", &bin.repair_headers()));
        for (team_id, colors) in team_colors {
            bin.set_colors(*team_id, colors)?;
        }
        self.add_bin(paths::TEAM_COLOR, &bin.into_bytes(), &mut messages)?;
        let mut bin = uni_color;
        messages.extend(header_repaired("UniColor.bin", &bin.repair_headers()));
        // Before the merge: a failed kit's number is among a `Full` team's, so its entry stays
        // as it was, and every committed entry then replaces or joins what is kept.
        for team in team_kits {
            match team.coverage {
                ExportCoverage::Full => {
                    let numbers: Vec<u8> =
                        team.slots.iter().map(|slot| kit_number(*slot)).collect();
                    bin.keep_kits(team.team_id, &numbers)?;
                }
                ExportCoverage::Midcup => {}
            }
        }
        for (team_id, entry) in std::mem::take(&mut self.kit_colors) {
            bin.set_kit(team_id, &entry)?;
        }
        self.add_bin(paths::UNI_COLOR, &bin.into_bytes(), &mut messages)?;
        let lists = [
            (ItemTable::Boots, boots_list),
            (ItemTable::Gloves, glove_list),
        ];
        self.add_player_tables(lists, player_appearance, item_rows, &mut messages)?;
        self.sink.finish()?;
        Ok((true, messages))
    }

    /// Adds the Fox player tables: each of `lists` (`BootsList.bin` then `GloveList.bin`) the
    /// walk found, with `item_rows`' rows of it applied against the committed batches
    /// (`ItemList::apply`), written whole, changed or not, then `player_appearance`, when found,
    /// unchanged. A list the walk did not find is not written, a list of this run's rows alone
    /// taking every other player's away, and `player_table_missing` goes to `messages` when
    /// committed rows of it are left out.
    fn add_player_tables(
        &mut self,
        lists: [(ItemTable, Option<ItemList>); 2],
        player_appearance: Option<Vec<u8>>,
        item_rows: &[ItemRow],
        messages: &mut Vec<Message>,
    ) -> anyhow::Result<()> {
        for (table, list) in lists {
            match list {
                Some(mut list) => {
                    list.apply(table, item_rows, &self.committed);
                    self.add_bin(table.path(), &list.into_bytes(), messages)?;
                }
                None => messages.extend(table_missing(table, item_rows, &self.committed)),
            }
        }
        if let Some(bytes) = player_appearance {
            self.add_bin(paths::PLAYER_APPEARANCE, &bytes, messages)?;
        }
        Ok(())
    }

    /// A failed task contributes nothing: no entry, no kit config or kit colors to the bins, and
    /// no player table row. A task with an entry an override replaced still contributes the
    /// rest, its kit config, kit colors and rows included, and the `duplicate_path` joins its
    /// messages.
    fn commit(&mut self, batch: &mut TaskBatch) -> anyhow::Result<()> {
        if batch.entries.is_empty() {
            return Ok(());
        }
        let entries = std::mem::take(&mut batch.entries);
        for (path, bytes) in &entries {
            self.add(path, bytes, &mut batch.messages)?;
        }
        self.uniform_parameters.extend(batch.uniparam.take());
        self.kit_colors.extend(batch.uni_color.take());
        self.committed.insert(batch.index);
        // The task's bytes are in the CPK and their copies gone, so the memory they were
        // charged is free. The release is explicit: a grouped batch can stay in `submit`'s
        // vector after its own commit, so waiting on its destruction would hold it longer.
        drop(entries);
        batch.permit = None;
        Ok(())
    }

    /// Adds `bytes` at `path`, after the overrides when this is the first entry. At an
    /// override's path nothing is added, the override having won, and a `duplicate_path`
    /// naming the path goes to `messages`. Paths compare exactly, as the CPK's own duplicate
    /// check does.
    fn add(&mut self, path: &str, bytes: &[u8], messages: &mut Vec<Message>) -> anyhow::Result<()> {
        if self.overrides.contains_key(path) {
            messages.push(tool_message(
                Code::DuplicatePath,
                Scope::Run,
                Disposition::Keep,
                vec![("path", path.to_owned())],
            ));
            return Ok(());
        }
        if !self.started {
            self.start()?;
        }
        self.sink.add(path, bytes)
    }

    /// Adds the bin `bytes` at its game path `path`, after the bins prefix, as `add` adds an
    /// entry.
    fn add_bin(
        &mut self,
        path: &str,
        bytes: &[u8],
        messages: &mut Vec<Message>,
    ) -> anyhow::Result<()> {
        let path = format!("{}{path}", self.bins_prefix);
        self.add(&path, bytes, messages)
    }

    /// Adds the overrides, in path order, so they are the sink's first entries. Each override
    /// is read as it is added: a handful of files, never charged to the memory budget. One that
    /// cannot be read fails the output, naming it: a file the operator put there on purpose is
    /// not skipped.
    fn start(&mut self) -> anyhow::Result<()> {
        for (path, file) in &self.overrides {
            let bytes = fs::read(file)
                .with_context(|| format!("{}: cannot read the override", file.display()))?;
            self.sink.add(path, &bytes)?;
        }
        self.started = true;
        Ok(())
    }
}

/// The `bin_header_repaired` finding for the working bin `bin` whose headers were wrong for
/// the `repaired` teams; none when every header was sound.
fn header_repaired(bin: &str, repaired: &[u16]) -> Option<Message> {
    if repaired.is_empty() {
        return None;
    }
    let teams: Vec<String> = repaired.iter().map(u16::to_string).collect();
    Some(tool_message(
        Code::BinHeaderRepaired,
        Scope::Run,
        Disposition::Keep,
        vec![("bin", bin.to_owned()), ("teams", teams.join(", "))],
    ))
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::ops::Range;
    use std::path::Path;
    use std::sync::{Arc, mpsc};
    use std::thread;
    use std::time::Duration;

    use cpk::CpkArchive;
    use kit_config::{KitConfig, matches_fpc};
    use pipeline::MemoryBudget;
    use studio_core::{Disposition, ExportId, Scope};
    use uniparam::UniformParameter;

    use super::*;
    use crate::bins::{TeamColorBin, UniColorBin};
    use crate::messages::{Code, tool_message};
    use crate::plan::EffectiveTeamKitFpc;
    use crate::plan::item_rows::RowChange;
    use crate::templates::Templates;
    use crate::testing::scratch;

    /// The bins a PES 21 run with no installed bin builds on.
    fn bundled() -> WorkingBins {
        WorkingBins::bundled(PesVersion::Pes21, &Templates::embedded())
    }

    /// The bundled `TeamColor.bin`'s bytes.
    fn bundled_team_color() -> Vec<u8> {
        Templates::embedded().team_color().to_vec()
    }

    /// The bundled `UniColor.bin`'s bytes.
    fn bundled_uni_color() -> Vec<u8> {
        Templates::embedded().uni_color().to_vec()
    }

    /// `output` finished for PES `version` on the bundled bins with no team colors, the bins
    /// asserted to report nothing: whether a CPK was written.
    fn finish_plain(output: CpkOutput, version: PesVersion) -> anyhow::Result<bool> {
        let (written, messages) = output.finish(
            version,
            WorkingBins::bundled(version, &Templates::embedded()),
            &[],
            &[],
            &[],
        )?;
        assert_eq!(messages, []);
        Ok(written)
    }

    /// A batch whose one message names its index.
    fn batch(index: usize, paths: &[&str], uniparam: Option<&str>) -> TaskBatch {
        TaskBatch {
            index,
            entries: paths
                .iter()
                .map(|path| ((*path).to_owned(), path.as_bytes().to_vec()))
                .collect(),
            group: None,
            skipped: Vec::new(),
            uniparam: uniparam.map(|name| (name.to_owned(), vec![7; 120])),
            uni_color: None,
            messages: vec![note(index)],
            permit: None,
        }
    }

    /// `batch`, as a member of the player folder group at `group`.
    fn grouped(index: usize, group: Range<usize>, paths: &[&str]) -> TaskBatch {
        TaskBatch {
            group: Some(group),
            ..batch(index, paths, None)
        }
    }

    /// A folder's group at 1..5 (face, boots, gloves, textures) between a face at 0 and a kit
    /// at 5, each package's entries given or empty for a failed one, the textures' likewise.
    fn folder_run(packages: [&[&str]; 3], textures: &[&str]) -> Vec<TaskBatch> {
        let [face, boots, gloves] = packages;
        vec![
            batch(0, &["face/real/71403/face.fpk"], None),
            grouped(1, 1..5, face),
            grouped(2, 1..5, boots),
            grouped(3, 1..5, gloves),
            grouped(4, 1..5, textures),
            batch(5, &["kit/kit.ftex"], Some("kit")),
        ]
    }

    /// Writes `batches`, submitted in the given order, to `name.cpk` and returns its layout
    /// and the positions `submit` reported, in order.
    fn write_all(folder: &Path, name: &str, batches: Vec<TaskBatch>) -> (Vec<String>, Vec<usize>) {
        let path = folder.join(format!("{name}.cpk"));
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
        let mut reported = Vec::new();
        for batch in batches {
            for (index, messages) in output.submit(batch).unwrap() {
                assert_eq!(messages, [note(index)], "task {index}'s own message");
                reported.push(index);
            }
        }
        assert!(finish_plain(output, PesVersion::Pes21).unwrap());
        (layout(&path), reported)
    }

    #[test]
    fn a_folder_whose_textures_failed_commits_nothing_and_the_rest_of_the_run_goes_on() {
        let temp = scratch("writer_group_textures_failed");
        let batches = folder_run(
            [
                &["face/face.fpk"],
                &["boots/boots.fpk"],
                &["glove/glove.fpk"],
            ],
            &[],
        );
        let (layout, reported) = write_all(temp.path(), "textures_failed", batches);
        assert_eq!(
            layout,
            [
                "face/real/71403/face.fpk",
                "kit/kit.ftex",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
        assert_eq!(reported, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn a_folder_with_one_failed_package_commits_the_others_and_its_textures() {
        let temp = scratch("writer_group_package_failed");
        let batches = folder_run(
            [&["face/face.fpk"], &[], &["glove/glove.fpk"]],
            &["common/shirt.ftex"],
        );
        let (layout, reported) = write_all(temp.path(), "package_failed", batches);
        assert_eq!(
            layout,
            [
                "face/real/71403/face.fpk",
                "face/face.fpk",
                "glove/glove.fpk",
                "common/shirt.ftex",
                "kit/kit.ftex",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
        assert_eq!(reported, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn a_folder_whose_every_package_failed_keeps_its_textures_out() {
        let temp = scratch("writer_group_packages_failed");
        let batches = folder_run([&[], &[], &[]], &["common/shirt.ftex"]);
        let (layout, _) = write_all(temp.path(), "packages_failed", batches);
        assert_eq!(
            layout,
            [
                "face/real/71403/face.fpk",
                "kit/kit.ftex",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
    }

    #[test]
    fn a_package_the_textures_batch_skips_is_left_out_and_the_rest_of_the_folder_commits() {
        let temp = scratch("writer_group_package_skipped");
        let mut batches = folder_run(
            [
                &["face/face.fpk"],
                &["boots/boots.fpk"],
                &["glove/glove.fpk"],
            ],
            &["common/shirt.ftex"],
        );
        // The boots lost a `shared_texture_conflict`: the textures batch names their task.
        batches[4].skipped = vec![2];
        let (layout, reported) = write_all(temp.path(), "package_skipped", batches);
        assert_eq!(
            layout,
            [
                "face/real/71403/face.fpk",
                "face/face.fpk",
                "glove/glove.fpk",
                "common/shirt.ftex",
                "kit/kit.ftex",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
        assert_eq!(reported, [0, 1, 2, 3, 4, 5]);

        // With every package skipped the textures stay out, as when every package failed.
        let temp = scratch("writer_group_all_skipped");
        let mut batches = folder_run(
            [
                &["face/face.fpk"],
                &["boots/boots.fpk"],
                &["glove/glove.fpk"],
            ],
            &["common/shirt.ftex"],
        );
        batches[4].skipped = vec![1, 2, 3];
        let (layout, _) = write_all(temp.path(), "all_skipped", batches);
        assert_eq!(
            layout,
            [
                "face/real/71403/face.fpk",
                "kit/kit.ftex",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
    }

    #[test]
    fn a_group_arriving_out_of_order_lays_the_cpk_out_as_in_order() {
        let temp = scratch("writer_group_out_of_order");
        let folder = temp.path();
        let run = || {
            folder_run(
                [
                    &["face/face.fpk"],
                    &["boots/boots.fpk"],
                    &["glove/glove.fpk"],
                ],
                &["common/shirt.ftex"],
            )
        };
        let (in_order, reported) = write_all(folder, "in_order", run());
        assert_eq!(reported, [0, 1, 2, 3, 4, 5]);
        // The kit first, then the textures before its packages, the packages in reverse, the
        // face at 0 last: nothing is decided before the face arrives, then everything is.
        let mut shuffled = run();
        shuffled.reverse();
        let (out_of_order, reported) = write_all(folder, "out_of_order", shuffled);
        assert_eq!(out_of_order, in_order);
        assert_eq!(reported, [0, 1, 2, 3, 4, 5]);
        assert_eq!(
            in_order,
            [
                "face/real/71403/face.fpk",
                "face/face.fpk",
                "boots/boots.fpk",
                "glove/glove.fpk",
                "common/shirt.ftex",
                "kit/kit.ftex",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
    }

    fn note(index: usize) -> Message {
        tool_message(
            Code::FolderPackFailed,
            Scope::Run,
            Disposition::Keep,
            vec![("task", index.to_string())],
        )
    }

    /// Three batches, the second a kit with a config.
    fn three_batches() -> [TaskBatch; 3] {
        [
            batch(0, &["m/third.bin", "b/fourth.bin"], None),
            batch(1, &["z/second.bin"], Some("kit")),
            batch(2, &["a/first.bin"], None),
        ]
    }

    /// The CPK's entries in the order their bytes sit in the file.
    fn layout(path: &Path) -> Vec<String> {
        let archive = CpkArchive::open(File::open(path).unwrap()).unwrap();
        let mut entries = archive.entries().to_vec();
        entries.sort_by_key(|entry| entry.offset);
        entries.into_iter().map(|entry| entry.path).collect()
    }

    /// The bytes of the CPK at `cpk`'s entry `path`.
    fn entry(cpk: &Path, path: &str) -> Vec<u8> {
        let mut archive = CpkArchive::open(File::open(cpk).unwrap()).unwrap();
        let entry = archive
            .entries()
            .iter()
            .find(|entry| entry.path == path)
            .unwrap_or_else(|| panic!("no {path} in the CPK"))
            .clone();
        archive.read(&entry).unwrap()
    }

    /// What the override file for the CPK path `path` holds: other bytes than a batch's entry
    /// at that path (`batch` writes the path itself).
    fn override_bytes(path: &str) -> Vec<u8> {
        format!("override of {path}").into_bytes()
    }

    /// Overrides at the CPK paths `paths`, each a file in `folder` holding
    /// `override_bytes(path)`.
    fn overrides(folder: &Path, paths: &[&str]) -> BTreeMap<String, PathBuf> {
        let folder = folder.join("overrides");
        fs::create_dir_all(&folder).unwrap();
        paths
            .iter()
            .enumerate()
            .map(|(index, path)| {
                let file = folder.join(format!("{index}.bin"));
                fs::write(&file, override_bytes(path)).unwrap();
                ((*path).to_owned(), file)
            })
            .collect()
    }

    /// The `duplicate_path` finding naming `path`.
    fn duplicate(path: &str) -> Message {
        tool_message(
            Code::DuplicatePath,
            Scope::Run,
            Disposition::Keep,
            vec![("path", path.to_owned())],
        )
    }

    #[test]
    fn the_overrides_go_first_and_replace_a_task_s_entry_at_their_path() {
        let temp = scratch("writer_overrides");
        let folder = temp.path();
        let path = folder.join("cup.cpk");
        let mut output = CpkOutput::new(
            OutputSink::cpk(path.clone()),
            overrides(folder, &["z/over.bin", "a/over.bin"]),
            "",
        );

        let committed = output
            .submit(batch(0, &["a/over.bin", "b/own.bin"], None))
            .unwrap();

        assert_eq!(committed, [(0, vec![note(0), duplicate("a/over.bin")])]);
        let message = &committed[0].1[1];
        assert_eq!(message.code.code, "duplicate_path");
        assert_eq!(
            (message.severity, message.disposition, &message.scope),
            (
                studio_core::Severity::Warning,
                Disposition::Keep,
                &Scope::Run
            )
        );
        assert!(finish_plain(output, PesVersion::Pes21).unwrap());
        assert_eq!(
            layout(&path),
            [
                "a/over.bin",
                "z/over.bin",
                "b/own.bin",
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
        assert_eq!(entry(&path, "a/over.bin"), override_bytes("a/over.bin"));
        assert_eq!(entry(&path, "b/own.bin"), b"b/own.bin");
    }

    #[test]
    fn an_override_at_a_bin_s_path_replaces_the_bin_and_finish_reports_it() {
        let temp = scratch("writer_override_bin");
        let folder = temp.path();
        let path = folder.join("cup.cpk");
        let mut output = CpkOutput::new(
            OutputSink::cpk(path.clone()),
            overrides(folder, &[paths::TEAM_COLOR]),
            "",
        );
        output.submit(batch(0, &["a/b.bin"], None)).unwrap();

        let (written, messages) = output
            .finish(
                PesVersion::Pes21,
                WorkingBins::bundled(PesVersion::Pes21, &Templates::embedded()),
                &team_714_colors(),
                &[],
                &[],
            )
            .unwrap();

        assert!(written);
        assert_eq!(messages, [duplicate(paths::TEAM_COLOR)]);
        assert_eq!(
            layout(&path),
            [paths::TEAM_COLOR, "a/b.bin", paths::UNI_COLOR]
        );
        assert_eq!(
            entry(&path, paths::TEAM_COLOR),
            override_bytes(paths::TEAM_COLOR)
        );
    }

    #[test]
    fn a_kit_whose_entry_is_overridden_still_gives_its_kit_config_and_colors_to_the_bins() {
        let temp = scratch("writer_override_kit");
        let folder = temp.path();
        let path = folder.join("cup.cpk");
        let mut output = CpkOutput::new(
            OutputSink::cpk(path.clone()),
            overrides(folder, &["kit/kit.ftex"]),
            "",
        );

        let committed = output.submit(kit_batch(0, &["kit/kit.ftex"])).unwrap();

        assert_eq!(committed, [(0, vec![note(0), duplicate("kit/kit.ftex")])]);
        assert!(finish_plain(output, PesVersion::Pes21).unwrap());
        assert_eq!(entry(&path, "kit/kit.ftex"), override_bytes("kit/kit.ftex"));
        let bin = UniformParameter::read(&entry(&path, paths::UNIFORM_PARAMETER)).unwrap();
        assert_eq!(bin.get("kit"), Some(&[7; 120][..]));
        let kit_colors = entry(&path, paths::UNI_COLOR);
        let start = kit_record(792);
        assert_eq!(
            kit_colors[start + 5..start + 13],
            [0x10, 0x0b, 0xc1, 0x12, 0x00, 0x41, 0x41, 0x41],
            "the kit's entry is merged into team 792's record"
        );
    }

    #[test]
    fn overrides_and_no_committed_batch_still_write_the_cpk_with_the_bins() {
        let temp = scratch("writer_overrides_alone");
        let folder = temp.path();
        let path = folder.join("run/cup.cpk");
        let mut output = CpkOutput::new(
            OutputSink::cpk(path.clone()),
            overrides(folder, &["a/over.bin"]),
            "",
        );
        // A failed task commits nothing.
        output.submit(batch(0, &[], Some("kit"))).unwrap();

        assert!(finish_plain(output, PesVersion::Pes21).unwrap());

        assert_eq!(
            layout(&path),
            ["a/over.bin", paths::TEAM_COLOR, paths::UNI_COLOR]
        );
        assert_eq!(entry(&path, "a/over.bin"), override_bytes("a/over.bin"));
        assert!(
            entry(&path, paths::UNI_COLOR) == bundled_uni_color(),
            "no kit committed"
        );
    }

    #[test]
    fn an_override_that_cannot_be_read_fails_the_cpk_naming_the_file() {
        let temp = scratch("writer_override_gone");
        let folder = temp.path();
        let gone = overrides(folder, &["a/over.bin"]);
        let file = gone["a/over.bin"].clone();
        fs::remove_file(&file).unwrap();
        let expected = format!("{}: cannot read the override", file.display());

        // At the first entry a task commits.
        let mut output =
            CpkOutput::new(OutputSink::cpk(folder.join("first.cpk")), gone.clone(), "");
        let error = output.submit(batch(0, &["b/own.bin"], None)).unwrap_err();
        assert_eq!(error.to_string(), expected);

        // At the end, when no task committed anything.
        let output = CpkOutput::new(OutputSink::cpk(folder.join("last.cpk")), gone, "");
        let error = finish_plain(output, PesVersion::Pes21).unwrap_err();
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn batches_are_laid_out_in_manifest_order_and_the_bins_last() {
        let temp = scratch("writer_order");
        let folder = temp.path();
        let path = folder.join("run/cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");

        output.submit(batch(2, &["a/first.bin"], None)).unwrap();
        output
            .submit(batch(1, &["z/second.bin"], Some("kit")))
            .unwrap();
        assert!(!path.exists(), "nothing is written before task 0 arrives");
        output
            .submit(batch(0, &["m/third.bin", "b/fourth.bin"], None))
            .unwrap();

        assert!(finish_plain(output, PesVersion::Pes21).unwrap());
        assert_eq!(
            layout(&path),
            [
                "m/third.bin",
                "b/fourth.bin",
                "z/second.bin",
                "a/first.bin",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
            ]
        );
        let mut archive = CpkArchive::open(File::open(&path).unwrap()).unwrap();
        let entry = archive
            .entries()
            .iter()
            .find(|entry| entry.path == paths::UNIFORM_PARAMETER)
            .unwrap()
            .clone();
        let bin = UniformParameter::read(&archive.read(&entry).unwrap()).unwrap();
        assert_eq!(bin.get("kit"), Some(&[7; 120][..]));
        let base = UniformParameter::read(
            Templates::embedded()
                .uniform_parameter_base(PesVersion::Pes21)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(bin.len(), base.len() + 1, "the base's entries are kept");
    }

    #[test]
    fn uniform_parameter_bin_is_built_on_the_working_one() {
        let temp = scratch("writer_working_uniform_parameter");
        let path = temp.path().join("cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
        output.submit(batch(0, &["a/b.bin"], Some("kit"))).unwrap();
        // PES 18's base stands in for an installed bin: it is not the one a PES 21 run bundles.
        let templates = Templates::embedded();
        let working = templates.uniform_parameter_base(PesVersion::Pes18).unwrap();
        let bins = WorkingBins {
            uniform_parameter: Some(UniformParameter::read(working).unwrap()),
            ..bundled()
        };

        let (written, _) = output
            .finish(PesVersion::Pes21, bins, &[], &[], &[])
            .unwrap();

        assert!(written);
        let bin = UniformParameter::read(&entry(&path, paths::UNIFORM_PARAMETER)).unwrap();
        let mut expected = UniformParameter::read(working).unwrap();
        expected.insert("kit".to_owned(), vec![7; 120]).unwrap();
        assert!(
            bin.write() == expected.write(),
            "the working bin with the kit's config added"
        );
    }

    #[test]
    fn the_arrival_order_changes_no_byte_of_the_cpk() {
        let temp = scratch("writer_arrival");
        let folder = temp.path();
        let mut written = Vec::new();
        for (name, reversed) in [("forward", false), ("reversed", true)] {
            let path = folder.join(format!("{name}.cpk"));
            let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
            let mut batches = three_batches();
            if reversed {
                batches.reverse();
            }
            for batch in batches {
                output.submit(batch).unwrap();
            }
            assert!(finish_plain(output, PesVersion::Pes21).unwrap());
            written.push(fs::read(&path).unwrap());
        }
        assert_eq!(written[0], written[1]);
    }

    #[test]
    fn submit_returns_each_committed_batch_and_its_messages_in_manifest_order() {
        let temp = scratch("writer_committed");
        let folder = temp.path();
        let mut output =
            CpkOutput::new(OutputSink::cpk(folder.join("cup.cpk")), BTreeMap::new(), "");
        let [first, second, third] = three_batches();

        assert_eq!(output.submit(third).unwrap(), [], "task 2 waits for 0");
        assert_eq!(output.submit(first).unwrap(), [(0, vec![note(0)])]);
        assert_eq!(
            output.submit(second).unwrap(),
            [(1, vec![note(1)]), (2, vec![note(2)])]
        );
    }

    #[test]
    fn a_failed_commit_releases_the_permits_of_the_batches_waiting_behind_it() {
        let temp = scratch("writer_failed_commit");
        let folder = temp.path();
        let mut output =
            CpkOutput::new(OutputSink::cpk(folder.join("cup.cpk")), BTreeMap::new(), "");
        let budget = MemoryBudget::new(1);
        let mut waiting = batch(1, &["a/waiting.bin"], None);
        waiting.permit = Some(Arc::new(budget.acquire(1).unwrap()));
        output.submit(waiting).unwrap();

        // Two entries with one path: the CPK refuses the second.
        let error = output
            .submit(batch(0, &["a/twice.bin", "a/twice.bin"], None))
            .unwrap_err();
        assert!(
            error.to_string().contains("cannot add a/twice.bin"),
            "{error}"
        );

        let (admitted_tx, admitted) = mpsc::channel();
        thread::spawn(move || admitted_tx.send(budget.acquire(1).is_ok()).unwrap());
        assert_eq!(
            admitted.recv_timeout(Duration::from_secs(5)),
            Ok(true),
            "the waiting batch's permit was released"
        );
    }

    #[test]
    fn a_failed_task_contributes_nothing_not_even_its_kit_config() {
        let temp = scratch("writer_failed");
        let folder = temp.path();
        let path = folder.join("run/cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");

        output.submit(batch(0, &[], Some("kit"))).unwrap();

        assert!(!finish_plain(output, PesVersion::Pes21).unwrap());
        assert!(!folder.join("run").exists(), "no file and no folder");
    }

    #[test]
    fn a_batch_that_never_arrives_is_an_error() {
        let temp = scratch("writer_gap");
        let mut output = CpkOutput::new(
            OutputSink::cpk(temp.path().join("cup.cpk")),
            BTreeMap::new(),
            "",
        );
        output.submit(batch(1, &["a/b.bin"], None)).unwrap();
        let error = finish_plain(output, PesVersion::Pes21).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the writer never received task 0 of the manifest"
        );
    }

    #[test]
    fn a_kit_config_for_a_version_without_the_bin_is_an_error() {
        let temp = scratch("writer_pre_fox");
        let mut output = CpkOutput::new(
            OutputSink::cpk(temp.path().join("cup.cpk")),
            BTreeMap::new(),
            "",
        );
        output.submit(batch(0, &["a/b.bin"], Some("kit"))).unwrap();
        let error = finish_plain(output, PesVersion::Pes17).unwrap_err();
        assert_eq!(error.to_string(), "PES 2017 has no UniformParameter.bin");
    }

    /// The start of team `team_id`'s `TeamColor.bin` record.
    fn team_record(team_id: usize) -> usize {
        (team_id - 100) * 16
    }

    /// Team 714's two colors, as its root `colors.txt` gives them.
    fn team_714_colors() -> Vec<(u16, Vec<Rgb>)> {
        vec![(714, vec![[0xc1, 0x12, 0x00], [0x41, 0x41, 0x41]])]
    }

    /// A run committing one entry, finished on `bins` with team 714's colors: the CPK's
    /// `TeamColor.bin` and the findings `finish` returned.
    fn team_color_run(name: &str, bins: WorkingBins) -> (Vec<u8>, Vec<Message>) {
        let temp = scratch(name);
        let path = temp.path().join("cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
        output.submit(batch(0, &["a/b.bin"], None)).unwrap();
        let (written, messages) = output
            .finish(PesVersion::Pes21, bins, &team_714_colors(), &[], &[])
            .unwrap();
        assert!(written);
        let mut archive = CpkArchive::open(File::open(&path).unwrap()).unwrap();
        let entry = archive
            .entries()
            .iter()
            .find(|entry| entry.path == paths::TEAM_COLOR)
            .unwrap()
            .clone();
        (archive.read(&entry).unwrap(), messages)
    }

    #[test]
    fn team_color_bin_is_the_working_bin_with_each_team_s_colors_set() {
        let base = bundled_team_color();
        let (bin, messages) = team_color_run("writer_team_colors", bundled());
        let mut expected = base;
        let record = team_record(714);
        expected[record + 4..record + 10].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
        assert!(bin == expected, "only team 714's first two colors differ");
        assert_eq!(messages, []);
    }

    #[test]
    fn a_working_bin_s_broken_header_is_repaired_and_reported() {
        let mut broken = bundled_team_color();
        let record = team_record(799);
        broken[record..record + 4].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41]);
        let (bin, messages) = team_color_run(
            "writer_team_color_repaired",
            WorkingBins {
                team_color: TeamColorBin::read(broken.clone()).unwrap(),
                ..bundled()
            },
        );
        assert_eq!(bin[record..record + 4], [0x1f, 0x03, 0x04, 0x00]);
        // Team 799's record keeps its colors, team 714's gets its own, and every other
        // record is the working bin's.
        let mut expected = broken;
        expected[record..record + 4].copy_from_slice(&[0x1f, 0x03, 0x04, 0x00]);
        let ours = team_record(714);
        expected[ours + 4..ours + 10].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
        assert!(bin == expected, "only 799's header and 714's colors differ");
        let [message] = messages.as_slice() else {
            panic!("{messages:?}");
        };
        assert_eq!(message.code.code, "bin_header_repaired");
        assert_eq!(
            (message.severity, message.disposition, &message.scope),
            (
                studio_core::Severity::Warning,
                Disposition::Keep,
                &Scope::Run
            )
        );
        assert_eq!(
            message.context,
            [
                ("bin".to_owned(), "TeamColor.bin".to_owned()),
                ("teams".to_owned(), "799".to_owned())
            ]
        );
    }

    #[test]
    fn a_run_that_commits_nothing_builds_no_bin_and_reports_nothing() {
        for (name, team_colors) in [
            ("writer_no_entry_no_colors", Vec::new()),
            ("writer_no_entry_colors", team_714_colors()),
        ] {
            let temp = scratch(name);
            let folder = temp.path().join("run");
            let mut output =
                CpkOutput::new(OutputSink::cpk(folder.join("cup.cpk")), BTreeMap::new(), "");
            output.submit(batch(0, &[], None)).unwrap();
            let mut team_color = bundled_team_color();
            team_color[..4].copy_from_slice(&[1, 2, 3, 4]);
            let broken = WorkingBins {
                team_color: TeamColorBin::read(team_color).unwrap(),
                ..bundled()
            };
            let finished = output
                .finish(PesVersion::Pes21, broken, &team_colors, &[], &[])
                .unwrap();
            assert_eq!(finished, (false, Vec::new()), "{name}");
            assert!(!folder.exists(), "{name}: no file and no folder");
        }
    }

    /// The start of team `team_id`'s `UniColor.bin` record.
    fn kit_record(team_id: usize) -> usize {
        (team_id - 100) * 85
    }

    /// The tracer's goalkeeper kit's entry: icon 11, `#c11200` and `#414141`.
    fn tracer_g1() -> KitColorEntry {
        KitColorEntry {
            kit: 0x10,
            icon: 11,
            colors: [[0xc1, 0x12, 0x00], [0x41, 0x41, 0x41]],
        }
    }

    /// The kit batch at `index` of team 792's goalkeeper kit, its entries `paths` (empty for a
    /// failed task).
    fn kit_batch(index: usize, paths: &[&str]) -> TaskBatch {
        TaskBatch {
            uni_color: Some((792, tracer_g1())),
            ..batch(index, paths, Some("kit"))
        }
    }

    /// `batches` submitted in order and finished for PES 21 on `bins` with no team colors: the
    /// CPK's `UniColor.bin` and the findings `finish` returned.
    fn uni_color_run(
        name: &str,
        batches: Vec<TaskBatch>,
        bins: WorkingBins,
    ) -> (Vec<u8>, Vec<Message>) {
        let temp = scratch(name);
        let path = temp.path().join("cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
        for batch in batches {
            output.submit(batch).unwrap();
        }
        let (written, messages) = output
            .finish(PesVersion::Pes21, bins, &[], &[], &[])
            .unwrap();
        assert!(written);
        let mut archive = CpkArchive::open(File::open(&path).unwrap()).unwrap();
        let entry = archive
            .entries()
            .iter()
            .find(|entry| entry.path == paths::UNI_COLOR)
            .unwrap()
            .clone();
        (archive.read(&entry).unwrap(), messages)
    }

    #[test]
    fn uni_color_bin_is_the_working_bin_with_each_committed_kit_s_entry_merged() {
        let (bin, messages) = uni_color_run(
            "writer_uni_color",
            vec![kit_batch(0, &["kit/kit.ftex"])],
            bundled(),
        );

        assert_eq!(messages, []);
        // Team 792's placeholder record now holds the one kit, then nine unused entries.
        let mut record = vec![0x18, 0x03, 0x00, 0x00, 0x01];
        record.extend([0x10, 0x0b, 0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
        for _ in 0..9 {
            record.extend([0xff, 0, 0, 0, 0, 0, 0, 0]);
        }
        let mut expected = bundled_uni_color();
        let start = kit_record(792);
        expected[start..start + 85].copy_from_slice(&record);
        assert!(bin == expected, "only team 792's record differs");
    }

    #[test]
    fn a_failed_kit_task_leaves_uni_color_bin_as_the_working_bin() {
        let (bin, messages) = uni_color_run(
            "writer_uni_color_failed",
            vec![kit_batch(0, &[]), batch(1, &["a/b.bin"], None)],
            bundled(),
        );

        assert_eq!(messages, []);
        assert!(
            bin == bundled_uni_color(),
            "the failed kit's entry is not applied"
        );
    }

    #[test]
    fn each_working_color_bin_s_broken_header_is_reported_teamcolor_first() {
        let mut team_color = bundled_team_color();
        let team = team_record(799);
        team_color[team..team + 4].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41]);
        let mut uni_color = bundled_uni_color();
        let kits = kit_record(799);
        uni_color[kits..kits + 4].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41]);
        let bins = WorkingBins {
            team_color: TeamColorBin::read(team_color).unwrap(),
            uni_color: UniColorBin::read(uni_color).unwrap(),
            ..bundled()
        };

        let (bin, messages) = uni_color_run(
            "writer_uni_color_repaired",
            vec![batch(0, &["a/b.bin"], None)],
            bins,
        );

        assert_eq!(bin[kits..kits + 4], [0x1f, 0x03, 0x00, 0x00]);
        let contexts: Vec<(&str, &[(String, String)])> = messages
            .iter()
            .map(|message| (message.code.code.as_ref(), message.context.as_slice()))
            .collect();
        assert_eq!(
            contexts,
            [
                (
                    "bin_header_repaired",
                    &[
                        ("bin".to_owned(), "TeamColor.bin".to_owned()),
                        ("teams".to_owned(), "799".to_owned())
                    ][..]
                ),
                (
                    "bin_header_repaired",
                    &[
                        ("bin".to_owned(), "UniColor.bin".to_owned()),
                        ("teams".to_owned(), "799".to_owned())
                    ][..]
                ),
            ]
        );
        assert_eq!(
            (
                messages[1].severity,
                messages[1].disposition,
                &messages[1].scope
            ),
            (
                studio_core::Severity::Warning,
                Disposition::Keep,
                &Scope::Run
            )
        );
    }

    /// One kit batch committed (or not, when `commits` is unset), finished for PES 21 on the
    /// bundled bins with team 714's `Midcup` export of no kit, its status `fpc`, a working
    /// `UniformParameter.bin` holding only 714's p1 config, lacking the FPC values, and 714's
    /// working `UniColor.bin` record holding kit 0 (p1) only: the CPK's layout, its
    /// `UniformParameter.bin` when it holds one, and the findings.
    fn finish_with(
        name: &str,
        commits: bool,
        fpc: EffectiveTeamKitFpc,
    ) -> (Vec<String>, Option<Vec<u8>>, Vec<Message>) {
        let mut config = KitConfig::template();
        config.shirt.model = 144;
        assert!(!matches_fpc(&config));
        let mut uniform_parameter = UniformParameter::new();
        uniform_parameter
            .insert(
                "714_DEF_1st_realUni.bin".to_owned(),
                config.encode(PesVersion::Pes21).to_vec(),
            )
            .unwrap();
        // Count 1, kit 0 with icon 3, then nine unused entries.
        let mut record = vec![0xca, 0x02, 0x00, 0x00, 1, 0, 3, 0, 0, 0, 0, 0, 0];
        for _ in 1..10 {
            record.extend([0xff, 0, 0, 0, 0, 0, 0, 0]);
        }
        let mut uni_color = bundled_uni_color();
        let start = kit_record(714);
        uni_color[start..start + 85].copy_from_slice(&record);
        let team = TeamKits {
            export_id: ExportId(3),
            team_id: 714,
            coverage: ExportCoverage::Midcup,
            fpc,
            slots: Vec::new(),
        };

        let temp = scratch(name);
        let path = temp.path().join("cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
        let paths: &[&str] = if commits { &["a/b.bin"] } else { &[] };
        output.submit(batch(0, paths, None)).unwrap();
        let bins = WorkingBins {
            uniform_parameter: Some(uniform_parameter),
            uni_color: UniColorBin::read(uni_color).unwrap(),
            ..bundled()
        };
        let (written, messages) = output
            .finish(PesVersion::Pes21, bins, &[], &[team], &[])
            .unwrap();
        if !written {
            return (Vec::new(), None, messages);
        }
        let layout = layout(&path);
        let bin = layout
            .contains(&paths::UNIFORM_PARAMETER.to_owned())
            .then(|| entry(&path, paths::UNIFORM_PARAMETER));
        (layout, bin, messages)
    }

    #[test]
    fn a_patched_config_alone_writes_uniform_parameter_bin_first_of_the_bins() {
        let (layout, bin, messages) =
            finish_with("writer_fpc_patched", true, EffectiveTeamKitFpc::On);

        assert_eq!(
            layout,
            [
                "a/b.bin",
                paths::UNIFORM_PARAMETER,
                paths::TEAM_COLOR,
                paths::UNI_COLOR
            ]
        );
        let bin = UniformParameter::read(&bin.unwrap()).unwrap();
        let config = KitConfig::decode(
            bin.get("714_DEF_1st_realUni.bin").unwrap(),
            PesVersion::Pes21,
        )
        .unwrap();
        assert!(matches_fpc(&config));
        assert_eq!(
            messages,
            [tool_message(
                Code::KitConfigFpcAdjusted,
                Scope::Export {
                    export_id: ExportId(3)
                },
                Disposition::Keep,
                vec![("slot", "p1".to_owned())],
            )]
        );
    }

    #[test]
    fn a_bin_nothing_changed_is_not_written_and_a_run_that_commits_nothing_reports_nothing() {
        // Status unknown: nothing to patch, and no kit committed.
        let (layout, bin, messages) =
            finish_with("writer_fpc_unchanged", true, EffectiveTeamKitFpc::Unknown);
        assert_eq!(layout, ["a/b.bin", paths::TEAM_COLOR, paths::UNI_COLOR]);
        assert_eq!((bin, messages), (None, Vec::new()));

        // A config there is to patch, but no CPK is written.
        let (layout, bin, messages) =
            finish_with("writer_fpc_no_cpk", false, EffectiveTeamKitFpc::On);
        assert_eq!((layout, bin, messages), (Vec::new(), None, Vec::new()));
    }

    #[test]
    fn the_player_tables_found_follow_uni_color_bin_with_the_committed_rows_set() {
        let temp = scratch("writer_player_tables");
        let path = temp.path().join("cup.cpk");
        let mut output = CpkOutput::new(OutputSink::cpk(path.clone()), BTreeMap::new(), "");
        // Slot 05's boots commit; slot 07's boots task failed.
        output.submit(batch(0, &["boots/k0625.fpk"], None)).unwrap();
        output.submit(batch(1, &[], None)).unwrap();
        let installed = |pairs: &[(u32, u32)]| -> Vec<u8> {
            pairs
                .iter()
                .flat_map(|(player_id, id)| [player_id.to_le_bytes(), id.to_le_bytes()])
                .flatten()
                .collect()
        };
        let appearance = vec![0x5a; 120];
        let bins = WorkingBins {
            boots_list: Some(
                ItemList::read(ItemTable::Boots, &installed(&[(71407, 7), (70201, 11)])).unwrap(),
            ),
            player_appearance: Some(appearance.clone()),
            ..bundled()
        };
        let rows = [
            ItemRow {
                table: ItemTable::Boots,
                player_id: 71405,
                change: RowChange::Set { id: 625, task: 0 },
            },
            ItemRow {
                table: ItemTable::Boots,
                player_id: 71407,
                change: RowChange::Set { id: 627, task: 1 },
            },
            ItemRow {
                table: ItemTable::Gloves,
                player_id: 71405,
                change: RowChange::Set { id: 625, task: 0 },
            },
        ];

        let (written, messages) = output
            .finish(PesVersion::Pes21, bins, &[], &[], &rows)
            .unwrap();

        assert!(written);
        assert_eq!(
            messages,
            [tool_message(
                Code::PlayerTableMissing,
                Scope::Run,
                Disposition::Keep,
                vec![
                    ("table", "GloveList.bin".to_owned()),
                    ("rows", "1".to_owned())
                ],
            )],
            "no GloveList.bin installed"
        );
        assert_eq!(
            layout(&path),
            [
                "boots/k0625.fpk",
                paths::TEAM_COLOR,
                paths::UNI_COLOR,
                paths::BOOTS_LIST,
                paths::PLAYER_APPEARANCE,
            ]
        );
        assert_eq!(
            entry(&path, paths::BOOTS_LIST),
            installed(&[(70201, 11), (71405, 625), (71407, 7)])
        );
        assert_eq!(entry(&path, paths::PLAYER_APPEARANCE), appearance);
    }
}
