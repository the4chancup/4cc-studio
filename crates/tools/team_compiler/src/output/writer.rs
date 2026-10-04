//! The canonical-order CPK writer (`team_compiler/pipeline.md` "5. Writer"): task batches go
//! into the CPK in manifest order whatever order they arrive in, a player folder's group
//! decided as one once its textures batch is in, then the bins.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::{Context, ensure};
use cpk::CpkWriter;
use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};
use uniparam::UniformParameter;

use crate::bins::{Rgb, TeamColorBin, WorkingBins};
use crate::messages::{Code, tool_message};
use crate::paths;
use crate::processing::TaskBatch;
use crate::templates;

/// The CPK header's tool-version string. One string per release, so a release compiling the
/// same exports writes the same bytes.
const TOOL_VERSION: &str = concat!("4cc Studio ", env!("CARGO_PKG_VERSION"));

/// One CPK being written. The file is created with the first committed entry, so a run that
/// commits nothing leaves no file and no folder behind.
pub(crate) struct CpkOutput {
    /// Where the CPK is written.
    path: PathBuf,
    cpk: Option<CpkWriter<File>>,
    /// The manifest position of the next batch to commit.
    next: usize,
    /// Batches that arrived before an earlier one, or a player folder's packages held for its
    /// textures batch, by manifest position.
    pending: BTreeMap<usize, TaskBatch>,
    /// The committed kits' `UniformParameter.bin` entries.
    uniform_parameters: Vec<(String, Vec<u8>)>,
}

impl CpkOutput {
    /// A CPK to be written at `path`; nothing is created yet.
    pub(crate) fn new(path: PathBuf) -> CpkOutput {
        CpkOutput {
            path,
            cpk: None,
            next: 0,
            pending: BTreeMap::new(),
            uniform_parameters: Vec::new(),
        }
    }

    /// Takes `batch` and commits every batch that is now next in manifest order. Committing in
    /// manifest order, not arrival order, is what makes the CPK's layout the same on every run.
    /// A player folder's group (`TaskBatch::group`) is decided only once every batch of it is
    /// in: its packages wait, their memory still charged, for the textures batch that decides
    /// them. Returns the batches decided, in manifest order, as each one's manifest position
    /// and messages, so the caller reports findings in manifest order too; empty when `batch`
    /// waits for an earlier one.
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
            for batch in &mut batches {
                committed.push((batch.index, std::mem::take(&mut batch.messages)));
            }
            let result = match group {
                Some(_) => self.commit_folder(batches),
                None => batches.into_iter().try_for_each(|batch| self.commit(batch)),
            };
            if let Err(error) = result {
                // The CPK is lost, so the batches waiting here go now: their permits may be
                // what the coordinator is waiting for, and it must reach the end of the run.
                self.pending.clear();
                return Err(error);
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
    fn commit_folder(&mut self, mut batches: Vec<TaskBatch>) -> anyhow::Result<()> {
        let textures = batches
            .pop()
            .expect("a player folder's group ends with its textures batch");
        if textures.entries.is_empty() {
            return Ok(());
        }
        let mut committed = false;
        for package in batches {
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

    /// Writes the bins and closes the CPK: `UniformParameter.bin` when a kit committed, then
    /// `TeamColor.bin`, built on `bins`' with every record's header set from its position and
    /// each of `team_colors` (team id, colors) set in its team's record. Returns whether a CPK
    /// was written, `false` when no batch committed anything (then no file exists and no bin
    /// is built), and the findings to report: `bin_header_repaired` naming the teams whose
    /// header the working bin had wrong.
    pub(crate) fn finish(
        mut self,
        version: PesVersion,
        bins: WorkingBins,
        team_colors: &[(u16, Vec<Rgb>)],
    ) -> anyhow::Result<(bool, Vec<Message>)> {
        ensure!(
            self.pending.is_empty(),
            "the writer never received task {} of the manifest",
            self.next
        );
        // The bins hold what every committed kit contributed, so they are built only once
        // every batch is in, and go last.
        if !self.uniform_parameters.is_empty() {
            let base = templates::uniform_parameter_base(version)
                .with_context(|| format!("{version} has no UniformParameter.bin"))?;
            let mut bin = UniformParameter::read(base)
                .context("cannot read the bundled UniformParameter base")?;
            for (name, config) in std::mem::take(&mut self.uniform_parameters) {
                bin.insert(name, config)?;
            }
            self.add(paths::UNIFORM_PARAMETER, &bin.write())?;
        }
        // A run that committed nothing writes no file, so it adds no bin either.
        if self.cpk.is_none() {
            return Ok((false, Vec::new()));
        }
        let mut messages = Vec::new();
        let mut bin = TeamColorBin::read(bins.team_color)?;
        let repaired = bin.repair_headers();
        if !repaired.is_empty() {
            let teams: Vec<String> = repaired.iter().map(u16::to_string).collect();
            messages.push(tool_message(
                Code::BinHeaderRepaired,
                Scope::Run,
                Disposition::Keep,
                vec![
                    ("bin", "TeamColor.bin".to_owned()),
                    ("teams", teams.join(", ")),
                ],
            ));
        }
        for (team_id, colors) in team_colors {
            bin.set_colors(*team_id, colors)?;
        }
        self.add(paths::TEAM_COLOR, &bin.into_bytes())?;
        let cpk = self
            .cpk
            .expect("the CPK was created before the bins were added");
        cpk.finish()
            .with_context(|| format!("{}: cannot write the CPK", self.path.display()))?;
        Ok((true, messages))
    }

    /// A failed task contributes nothing: no entry, and no kit config to the bins.
    fn commit(&mut self, batch: TaskBatch) -> anyhow::Result<()> {
        if batch.entries.is_empty() {
            return Ok(());
        }
        for (path, bytes) in &batch.entries {
            self.add(path, bytes)?;
        }
        self.uniform_parameters.extend(batch.uniparam);
        // The task's bytes are in the CPK now, so the memory they were charged is free.
        drop(batch.permit);
        Ok(())
    }

    fn add(&mut self, path: &str, bytes: &[u8]) -> anyhow::Result<()> {
        let cpk = match self.cpk.take() {
            Some(cpk) => cpk,
            None => self.create()?,
        };
        self.cpk
            .insert(cpk)
            .add(path, bytes, None)
            .with_context(|| format!("{}: cannot add {path}", self.path.display()))
    }

    fn create(&self) -> anyhow::Result<CpkWriter<File>> {
        let cannot_create = || format!("{}: cannot create the CPK", self.path.display());
        if let Some(folder) = self.path.parent() {
            fs::create_dir_all(folder).with_context(cannot_create)?;
        }
        let file = File::create(&self.path).with_context(cannot_create)?;
        CpkWriter::new(file, TOOL_VERSION).with_context(cannot_create)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Range;
    use std::path::Path;
    use std::sync::{Arc, mpsc};
    use std::thread;
    use std::time::Duration;

    use cpk::CpkArchive;
    use pipeline::MemoryBudget;
    use studio_core::{Disposition, Scope};

    use super::*;
    use crate::messages::{Code, tool_message};
    use crate::testing::scratch;

    /// `output` finished for PES `version` on the bundled bins with no team colors, the bins
    /// asserted to report nothing: whether a CPK was written.
    fn finish_plain(output: CpkOutput, version: PesVersion) -> anyhow::Result<bool> {
        let (written, messages) = output.finish(version, WorkingBins::bundled(), &[])?;
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
        let mut output = CpkOutput::new(path.clone());
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

    #[test]
    fn batches_are_laid_out_in_manifest_order_and_the_bins_last() {
        let temp = scratch("writer_order");
        let folder = temp.path();
        let path = folder.join("run/cup.cpk");
        let mut output = CpkOutput::new(path.clone());

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
        let base =
            UniformParameter::read(templates::uniform_parameter_base(PesVersion::Pes21).unwrap())
                .unwrap();
        assert_eq!(bin.len(), base.len() + 1, "the base's entries are kept");
    }

    #[test]
    fn the_arrival_order_changes_no_byte_of_the_cpk() {
        let temp = scratch("writer_arrival");
        let folder = temp.path();
        let mut written = Vec::new();
        for (name, reversed) in [("forward", false), ("reversed", true)] {
            let path = folder.join(format!("{name}.cpk"));
            let mut output = CpkOutput::new(path.clone());
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
        let mut output = CpkOutput::new(folder.join("cup.cpk"));
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
        let mut output = CpkOutput::new(folder.join("cup.cpk"));
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
        let mut output = CpkOutput::new(path.clone());

        output.submit(batch(0, &[], Some("kit"))).unwrap();

        assert!(!finish_plain(output, PesVersion::Pes21).unwrap());
        assert!(!folder.join("run").exists(), "no file and no folder");
    }

    #[test]
    fn a_batch_that_never_arrives_is_an_error() {
        let temp = scratch("writer_gap");
        let mut output = CpkOutput::new(temp.path().join("cup.cpk"));
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
        let mut output = CpkOutput::new(temp.path().join("cup.cpk"));
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
        let mut output = CpkOutput::new(path.clone());
        output.submit(batch(0, &["a/b.bin"], None)).unwrap();
        let (written, messages) = output
            .finish(PesVersion::Pes21, bins, &team_714_colors())
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
        let base = WorkingBins::bundled().team_color;
        let (bin, messages) = team_color_run("writer_team_colors", WorkingBins::bundled());
        let mut expected = base;
        let record = team_record(714);
        expected[record + 4..record + 10].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41, 0x41, 0x41]);
        assert!(bin == expected, "only team 714's first two colors differ");
        assert_eq!(messages, []);
    }

    #[test]
    fn a_working_bin_s_broken_header_is_repaired_and_reported() {
        let mut broken = WorkingBins::bundled().team_color;
        let record = team_record(799);
        broken[record..record + 4].copy_from_slice(&[0xc1, 0x12, 0x00, 0x41]);
        let (bin, messages) = team_color_run(
            "writer_team_color_repaired",
            WorkingBins {
                team_color: broken.clone(),
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
            let mut output = CpkOutput::new(folder.join("cup.cpk"));
            output.submit(batch(0, &[], None)).unwrap();
            let mut broken = WorkingBins::bundled();
            broken.team_color[..4].copy_from_slice(&[1, 2, 3, 4]);
            let finished = output
                .finish(PesVersion::Pes21, broken, &team_colors)
                .unwrap();
            assert_eq!(finished, (false, Vec::new()), "{name}");
            assert!(!folder.exists(), "{name}: no file and no folder");
        }
    }
}
