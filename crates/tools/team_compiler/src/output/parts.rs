//! Multi-CPK mode's teams parts (`team_compiler/pipeline.md` "5. Writer", step 6 "Multi-CPK
//! mode: teams parts"): the official list's numbered slots of one stem, filled in slot order
//! with whole teams, first-fit under `cpk_part_max_size`, and the placeholder CPK written in
//! every slot the run does not fill.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::{self, File};
use std::ops::Range;
use std::path::PathBuf;

use anyhow::{Context, ensure};
use cpk::CpkWriter;
use pipeline::CpkStem;
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, size_text, tool_message};
use crate::output::deploy::cpk_file_name;
use crate::output::sink::create_cpk;
use crate::upgrade::number_and_stem;

/// The teams slots the `official` list's file names reserve for the stem `stem`: each entry
/// `{prefix}_{NN}_{stem}.cpk` whose stem is exactly `stem`, so `teams` never takes a `teams2`
/// slot, and whose name is a valid `CpkStem`; without `.cpk`, ordered by `NN` read as a number
/// (`4cc_9_teams` before `4cc_41_teams`).
pub(crate) fn slots(official: &[String], stem: &str) -> Vec<CpkStem> {
    let mut numbered: Vec<(u128, CpkStem)> = official
        .iter()
        .filter_map(|name| {
            let (number, own_stem) = number_and_stem(name)?;
            if own_stem != stem {
                return None;
            }
            let slot = CpkStem::new(name.strip_suffix(".cpk")?).ok()?;
            let number = number
                .parse()
                .expect("a CPK name has at most 28 characters, and a u128 holds 38 digits");
            Some((number, slot))
        })
        .collect();
    numbered.sort_by_key(|(number, _)| *number);
    numbered.into_iter().map(|(_, slot)| slot).collect()
}

/// A team the parts cannot take: its finding, `cpk_slots_exhausted` or `cpk_team_exceeds_cap`,
/// Fatal on the run. It travels as the writer's error, so the run reports this finding rather
/// than a failure to write the CPK.
#[derive(Debug)]
pub(crate) struct Unplaced(pub(crate) Message);

impl fmt::Display for Unplaced {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a team was not placed: {}", self.0.code.code)
    }
}

impl std::error::Error for Unplaced {}

/// One teams part being written: its file in the run's staging folder, and its writer.
struct Part {
    path: PathBuf,
    cpk: CpkWriter<File>,
}

impl Part {
    /// Adds `bytes` as the entry `path`, with no timestamp, as every entry the run writes.
    fn add(&mut self, path: &str, bytes: &[u8]) -> anyhow::Result<()> {
        self.cpk
            .add(path, bytes, None)
            .with_context(|| format!("{}: cannot add {path}", self.path.display()))
    }

    /// Writes the part's table of contents and closes its file.
    fn finish(self) -> anyhow::Result<()> {
        self.cpk
            .finish()
            .with_context(|| format!("{}: cannot write the CPK", self.path.display()))?;
        Ok(())
    }
}

/// A multi-CPK run's teams parts, written into the run's staging folder as the writer decides
/// each team: its entries held until its export's last task is decided, then placed whole.
pub(crate) struct TeamsParts {
    /// The run's staging folder, where every part and placeholder goes.
    folder: PathBuf,
    /// The slots, in the order they are filled.
    slots: Vec<CpkStem>,
    /// The slots' stem, which `cpk_slots_exhausted` names.
    stem: String,
    /// `cpk_part_max_size`: the most bytes a part may hold, its table of contents included.
    cap: u64,
    /// The placeholder CPK, written as every slot the run does not fill.
    placeholder: Vec<u8>,
    /// The manifest position of each export's last task, with its source's file name: where a
    /// team ends, and how the findings name it.
    ends: BTreeMap<usize, String>,
    /// The part being filled, once a team is placed.
    part: Option<Part>,
    /// How many slots have been opened, the open part's included.
    opened: usize,
    /// The current team's entries (CPK path, bytes), held until the team is placed.
    held: Vec<(String, Vec<u8>)>,
}

impl TeamsParts {
    /// The parts of `slots` (`slots()`), stem `stem`, to be written into `folder` under `cap`
    /// bytes each, the slots left over as `placeholder`; `ends` gives each export's last
    /// manifest position and its source's file name. Nothing is written yet.
    pub(crate) fn new(
        folder: PathBuf,
        slots: Vec<CpkStem>,
        stem: &str,
        cap: u64,
        placeholder: Vec<u8>,
        ends: BTreeMap<usize, String>,
    ) -> TeamsParts {
        TeamsParts {
            folder,
            slots,
            stem: stem.to_owned(),
            cap,
            placeholder,
            ends,
            part: None,
            opened: 0,
            held: Vec::new(),
        }
    }

    /// Holds `bytes`, the current team's entry at the CPK path `path`, until the team is
    /// placed.
    pub(crate) fn hold(&mut self, path: String, bytes: Vec<u8>) {
        self.held.push((path, bytes));
    }

    /// Places the held team when `tasks`, the manifest positions the writer just decided,
    /// hold its export's last task (`place`).
    pub(crate) fn decided(&mut self, tasks: Range<usize>) -> anyhow::Result<()> {
        let Some(export) = self
            .ends
            .range(tasks)
            .next()
            .map(|(_, export)| export.clone())
        else {
            return Ok(());
        };
        self.place(&export)
    }

    /// Places the held team, the export `export`'s, whole: into the open part when the part
    /// stays at most the cap with it, else into the next slot's part, the open one finished.
    /// A team with no entry opens no slot. A team over the cap in an empty part is an
    /// `Unplaced` `cpk_team_exceeds_cap`, one needing a slot when none is left an `Unplaced`
    /// `cpk_slots_exhausted`; a part that cannot be created, added to or finished is the error
    /// naming its file.
    fn place(&mut self, export: &str) -> anyhow::Result<()> {
        if self.held.is_empty() {
            return Ok(());
        }
        // First-fit with teams never split: the open part is tried first and closed only when
        // this team does not fit it, so a part holds consecutive teams in canonical order.
        let mut part = match self.part.take() {
            Some(part) if self.len_with_held(&part) <= self.cap => part,
            Some(full) => {
                full.finish()?;
                self.open_for(export)?
            }
            None => self.open_for(export)?,
        };
        for (path, bytes) in self.held.drain(..) {
            part.add(&path, &bytes)?;
        }
        self.part = Some(part);
        Ok(())
    }

    /// The next slot's part, opened for the held team of `export`, which must fit it empty:
    /// `cpk_slots_exhausted` when no slot is left, `cpk_team_exceeds_cap` when the team alone
    /// is over the cap.
    fn open_for(&mut self, export: &str) -> anyhow::Result<Part> {
        let Some(slot) = self.slots.get(self.opened) else {
            return Err(Unplaced(tool_message(
                Code::CpkSlotsExhausted,
                Scope::Run,
                Disposition::AbortRun,
                vec![
                    ("export", export.to_owned()),
                    ("stem", self.stem.clone()),
                    ("slots", self.slots.len().to_string()),
                    ("cap", size_text(self.cap)),
                ],
            ))
            .into());
        };
        let path = self.folder.join(cpk_file_name(slot));
        let part = Part {
            cpk: create_cpk(&path)?,
            path,
        };
        self.opened += 1;
        let size = self.len_with_held(&part);
        if size > self.cap {
            return Err(Unplaced(tool_message(
                Code::CpkTeamExceedsCap,
                Scope::Run,
                Disposition::AbortRun,
                vec![
                    ("export", export.to_owned()),
                    ("size", size_text(size)),
                    ("cap", size_text(self.cap)),
                ],
            ))
            .into());
        }
        Ok(part)
    }

    /// The length `part`'s file would have with the held team added, its table of contents
    /// included.
    fn len_with_held(&self, part: &Part) -> u64 {
        let held = self.held.iter();
        part.cpk
            .len_with(held.map(|(path, bytes)| (path.as_str(), bytes.len() as u64)))
    }

    /// Finishes the open part and writes the placeholder as every slot not opened: every slot
    /// of the list gets its file ("Every slot is always written"). Every team held must have
    /// been placed.
    pub(crate) fn finish(self) -> anyhow::Result<()> {
        ensure!(
            self.held.is_empty(),
            "the entries of a team were never placed in a part"
        );
        if let Some(part) = self.part {
            part.finish()?;
        }
        for slot in &self.slots[self.opened..] {
            let path = self.folder.join(cpk_file_name(slot));
            fs::write(&path, &self.placeholder)
                .with_context(|| format!("{}: cannot write the placeholder CPK", path.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use cpk::CpkArchive;

    use super::*;
    use crate::templates::Templates;
    use crate::testing::scratch;

    fn strings(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    fn stems(slots: &[CpkStem]) -> Vec<&str> {
        slots.iter().map(CpkStem::as_str).collect()
    }

    #[test]
    fn the_slots_are_the_exact_stem_s_entries_ordered_by_number() {
        let official = strings(&[
            "4cc_08_bins.cpk",
            "4cc_42_teams.cpk",
            "4cc_41_teams.cpk",
            "4cc_51_teams2.cpk",
            "4cc_43_teams_old.cpk",
            "4cc_9_teams.cpk",
        ]);
        assert_eq!(
            stems(&slots(&official, "teams")),
            ["4cc_9_teams", "4cc_41_teams", "4cc_42_teams"]
        );
        assert_eq!(stems(&slots(&official, "teams2")), ["4cc_51_teams2"]);
        // Too long for a CPK name: not a slot.
        let long = strings(&["4cc_41_teams.cpk", "abcdefghijklmnopqrstuvwx_43_teams.cpk"]);
        assert_eq!(stems(&slots(&long, "teams")), ["4cc_41_teams"]);
    }

    /// Team `team_id`'s entries, as a compile lays them out: its goalkeeper kit texture and
    /// one player's face package.
    fn team(team_id: u16) -> Vec<(String, Vec<u8>)> {
        vec![
            (
                format!("Asset/model/character/uniform/texture/#windx11/u0{team_id}g1.ftex"),
                vec![1; 1500],
            ),
            (
                format!("Asset/model/character/face/real/{team_id}05/#Win/face.fpk"),
                vec![2; 700],
            ),
        ]
    }

    /// The paths of `entries`.
    fn paths_of(entries: &[(String, Vec<u8>)]) -> Vec<String> {
        entries.iter().map(|(path, _)| path.clone()).collect()
    }

    /// The length a CPK of `entries` would have, written as a part is.
    fn cpk_len(folder: &Path, entries: &[(String, Vec<u8>)]) -> u64 {
        let probe = create_cpk(&folder.join("probe/probe.cpk")).unwrap();
        probe.len_with(
            entries
                .iter()
                .map(|(path, bytes)| (path.as_str(), bytes.len() as u64)),
        )
    }

    /// Three slots `4cc_41_teams` to `4cc_43_teams` written into `folder` under `cap`, four
    /// exports ending at manifest positions 0 to 3.
    fn three_slots(folder: &Path, cap: u64) -> TeamsParts {
        let slots = ["4cc_41_teams", "4cc_42_teams", "4cc_43_teams"]
            .map(|slot| CpkStem::new(slot).unwrap())
            .to_vec();
        let ends = [
            "co Full Spring",
            "a Full Spring",
            "b Full Spring",
            "dbg Full Spring",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, export)| (index, export.to_owned()))
        .collect();
        TeamsParts::new(
            folder.join("run"),
            slots,
            "teams",
            cap,
            Templates::embedded().placeholder_cpk().to_vec(),
            ends,
        )
    }

    /// Holds `entries` in `parts` and decides the export ending at `end`.
    fn place_team(
        parts: &mut TeamsParts,
        end: usize,
        entries: Vec<(String, Vec<u8>)>,
    ) -> anyhow::Result<()> {
        for (path, bytes) in entries {
            parts.hold(path, bytes);
        }
        parts.decided(end..end + 1)
    }

    /// The paths in the CPK at `path`, in the order their bytes sit in the file.
    fn layout(path: &Path) -> Vec<String> {
        let archive = CpkArchive::open(File::open(path).unwrap()).unwrap();
        let mut entries = archive.entries().to_vec();
        entries.sort_by_key(|entry| entry.offset);
        entries.into_iter().map(|entry| entry.path).collect()
    }

    /// The `Unplaced` finding `error` carries.
    fn unplaced(error: anyhow::Error) -> Message {
        error.downcast::<Unplaced>().expect("an unplaced team").0
    }

    #[test]
    fn teams_fill_a_part_while_they_fit_and_the_slots_left_get_the_placeholder() {
        let temp = scratch("parts_first_fit");
        let folder = temp.path();
        let (co, a, b) = (team(714), team(702), team(707));
        let together: Vec<(String, Vec<u8>)> = co.iter().chain(&a).cloned().collect();
        // Exactly /co/ and /a/ together: at most the cap is a fit.
        let mut parts = three_slots(folder, cpk_len(folder, &together));

        place_team(&mut parts, 0, co).unwrap();
        // A range holding no export's end places nothing.
        parts.hold(a[0].0.clone(), a[0].1.clone());
        parts.decided(4..9).unwrap();
        place_team(&mut parts, 1, vec![a[1].clone()]).unwrap();
        place_team(&mut parts, 2, b.clone()).unwrap();
        parts.finish().unwrap();

        let run = folder.join("run");
        assert_eq!(layout(&run.join("4cc_41_teams.cpk")), paths_of(&together));
        assert_eq!(
            fs::metadata(run.join("4cc_41_teams.cpk")).unwrap().len(),
            cpk_len(folder, &together)
        );
        assert_eq!(layout(&run.join("4cc_42_teams.cpk")), paths_of(&b));
        assert!(
            fs::read(run.join("4cc_43_teams.cpk")).unwrap()
                == Templates::embedded().placeholder_cpk(),
            "the slot left over is the placeholder"
        );
    }

    // TC-OUT-13
    #[test]
    fn a_team_over_the_cap_in_an_empty_part_is_cpk_team_exceeds_cap_with_its_size() {
        let temp = scratch("parts_team_over_cap");
        let folder = temp.path();
        let co = team(714);
        let size = cpk_len(folder, &co);
        let mut parts = three_slots(folder, size - 1);

        let error = place_team(&mut parts, 0, co).unwrap_err();

        assert_eq!(
            unplaced(error),
            tool_message(
                Code::CpkTeamExceedsCap,
                Scope::Run,
                Disposition::AbortRun,
                vec![
                    ("export", "co Full Spring".to_owned()),
                    ("size", size_text(size)),
                    ("cap", size_text(size - 1)),
                ],
            )
        );
    }

    #[test]
    fn a_team_that_fits_no_fresh_part_after_a_full_one_is_cpk_team_exceeds_cap() {
        let temp = scratch("parts_second_try_over_cap");
        let folder = temp.path();
        let co = team(714);
        let mut parts = three_slots(folder, cpk_len(folder, &co));
        place_team(&mut parts, 0, co).unwrap();
        let mut big = team(702);
        // Past the content alignment's padding, so the file grows.
        big[0].1.extend([0; 4096]);
        let size = cpk_len(folder, &big);

        let error = place_team(&mut parts, 1, big).unwrap_err();

        let message = unplaced(error);
        assert_eq!(message.code.code, "cpk_team_exceeds_cap");
        assert_eq!(
            message.context[..2],
            [
                ("export".to_owned(), "a Full Spring".to_owned()),
                ("size".to_owned(), size_text(size)),
            ]
        );
    }

    // TC-OUT-14
    #[test]
    fn a_team_needing_a_slot_past_the_last_is_cpk_slots_exhausted() {
        let temp = scratch("parts_slots_exhausted");
        let folder = temp.path();
        // One team per part.
        let cap = cpk_len(folder, &team(714));
        let mut parts = three_slots(folder, cap);
        for (end, team_id) in [(0, 714), (1, 702), (2, 707)] {
            place_team(&mut parts, end, team(team_id)).unwrap();
        }

        let error = place_team(&mut parts, 3, team(790)).unwrap_err();

        assert_eq!(
            unplaced(error),
            tool_message(
                Code::CpkSlotsExhausted,
                Scope::Run,
                Disposition::AbortRun,
                vec![
                    ("export", "dbg Full Spring".to_owned()),
                    ("stem", "teams".to_owned()),
                    ("slots", "3".to_owned()),
                    ("cap", size_text(cap)),
                ],
            )
        );
    }

    #[test]
    fn a_team_with_no_entry_opens_no_slot() {
        let temp = scratch("parts_empty_team");
        let folder = temp.path();
        let mut parts = three_slots(folder, 1 << 20);

        place_team(&mut parts, 0, Vec::new()).unwrap();
        place_team(&mut parts, 1, team(702)).unwrap();
        parts.finish().unwrap();

        let run = folder.join("run");
        assert_eq!(layout(&run.join("4cc_41_teams.cpk")), paths_of(&team(702)));
        for slot in ["4cc_42_teams.cpk", "4cc_43_teams.cpk"] {
            assert!(
                fs::read(run.join(slot)).unwrap() == Templates::embedded().placeholder_cpk(),
                "{slot}"
            );
        }
    }

    #[test]
    fn an_unplaced_team_s_error_text_names_its_finding() {
        let message = tool_message(
            Code::CpkSlotsExhausted,
            Scope::Run,
            Disposition::AbortRun,
            Vec::new(),
        );
        assert_eq!(
            Unplaced(message).to_string(),
            "a team was not placed: cpk_slots_exhausted"
        );
    }

    #[test]
    fn finishing_with_a_team_still_held_is_an_error() {
        let temp = scratch("parts_held_at_finish");
        let mut parts = three_slots(temp.path(), 1 << 20);
        parts.hold("a/b.bin".to_owned(), vec![1]);
        assert!(parts.finish().is_err());
    }
}
