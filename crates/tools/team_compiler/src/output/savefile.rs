//! The savefile update (`spec/team_compiler/pipeline.md` "6. Post-processing"): a run whose
//! output the game now loads applies the `aesthetics_patch.toml` it
//! just wrote to the local savefile through `AestheticsPatch::apply`: the compiler has no
//! second savefile write path, so the local result and the savefile builder's later
//! `apply-patch` are the same bytes by construction. The save is never written while the
//! compile target's PES runs (the game would write its own save back over this one on
//! exit), and every outcome is a finding.

use std::path::{Path, PathBuf};

use pes_savefile::EditFile;
use pes_savefile::discovery::SavefileCandidate;
use pes_savefile::interchange::aesthetics_patch::{AestheticsPatch, PatchSkip};
use pes_savefile::model::player::STRIPPED_APPEARANCE_ID;
use pes_version::PesVersion;
use studio_core::{Disposition, RunningPes, Scope};

use crate::events::RunEvents;
use crate::messages::{Code, tool_message};
use crate::settings::SavefilePath;

/// Where the step reads and writes the save.
#[derive(Debug, PartialEq)]
pub(crate) enum Located {
    /// The configured path, which names a file.
    Configured(PathBuf),
    /// `auto`'s newest candidate, and how many candidates there were.
    Detected { path: PathBuf, candidates: usize },
    /// No file: the configured path, `None` for `auto`.
    Missing(Option<PathBuf>),
}

/// The save `setting` resolves to: `discover` (the Documents\KONAMI walk, whose list is
/// newest first) is called for `Auto` alone, a configured path only checked to be a file.
pub(crate) fn locate(
    setting: &SavefilePath,
    discover: impl FnOnce() -> Vec<SavefileCandidate>,
) -> Located {
    match setting {
        SavefilePath::Path(path) if path.is_file() => Located::Configured(path.clone()),
        SavefilePath::Path(path) => Located::Missing(Some(path.clone())),
        SavefilePath::Auto => {
            let candidates = discover();
            match candidates.first() {
                Some(first) => Located::Detected {
                    path: first.path.clone(),
                    candidates: candidates.len(),
                },
                None => Located::Missing(None),
            }
        }
    }
}

/// The save editor's `refuse_while_running` rule (`save_editor.md` "Writing a save"): only
/// the target version's game counts: another version's never writes this save back.
fn target_running(running: &[RunningPes], version: PesVersion) -> bool {
    running.iter().any(|game| game.version == version)
}

/// Applies `patch` (the `aesthetics_patch.toml` at `patch_path` the run just wrote) to the
/// save `located` names, reporting every outcome as a `Keep` finding: no save is the
/// warning `savefile_missing`, a save the target's running game owns is skipped, an
/// unreadable or wrong-version save an error, a patch team or player the save lacks a
/// per-item error with the rest applied (a save holding none of the patch's teams is not
/// rewritten), a compiled player whose appearance id the save still sets a warning, and the
/// save written, its previous file kept as `.bak`, the info `savefile_written`.
pub(crate) fn update(
    events: &mut RunEvents,
    located: Located,
    running: &[RunningPes],
    patch_path: &Path,
    patch: &AestheticsPatch,
) {
    let report = |events: &mut RunEvents, code: Code, context: Vec<(&'static str, String)>| {
        events.message(tool_message(code, Scope::Run, Disposition::Keep, context));
    };
    let path = match &located {
        Located::Missing(configured) => {
            let mut context = vec![("patch", patch_path.display().to_string())];
            if let Some(path) = configured {
                context.push(("path", path.display().to_string()));
            }
            report(events, Code::SavefileMissing, context);
            return;
        }
        Located::Configured(path) => path.clone(),
        Located::Detected { path, candidates } => {
            report(
                events,
                Code::SavefileAutodetected,
                vec![
                    ("path", path.display().to_string()),
                    ("candidates", candidates.to_string()),
                ],
            );
            path.clone()
        }
    };
    if target_running(running, patch.pes_version) {
        report(
            events,
            Code::SavefileSkippedPesRunning,
            vec![
                ("path", path.display().to_string()),
                ("version", patch.pes_version.to_string()),
            ],
        );
        return;
    }
    let mut file = match EditFile::load(&path) {
        Ok(file) => file,
        Err(error) => {
            report(
                events,
                Code::SavefileInvalid,
                vec![
                    ("path", path.display().to_string()),
                    ("error", format!("{error:#}")),
                ],
            );
            return;
        }
    };
    if file.version() != patch.pes_version {
        report(
            events,
            Code::SavefileInvalid,
            vec![
                ("path", path.display().to_string()),
                ("version", file.version().to_string()),
            ],
        );
        return;
    }
    let summary = match patch.apply(&mut file) {
        Ok(summary) => summary,
        Err(error) => {
            report(
                events,
                Code::SavefileWriteFailed,
                vec![
                    ("path", path.display().to_string()),
                    ("error", format!("{error:#}")),
                ],
            );
            return;
        }
    };
    for skip in &summary.skipped {
        match skip {
            PatchSkip::TeamNotInSave { team } => report(
                events,
                Code::SavefileTeamMissing,
                vec![("team", team.to_string())],
            ),
            PatchSkip::PlayerNotInSave { player } => report(
                events,
                Code::SavefilePlayerMissing,
                vec![("player", player.to_string())],
            ),
        }
    }
    let mut not_stripped: Vec<u32> = patch
        .teams
        .iter()
        .filter(|team| summary.applied_teams.contains(&team.id))
        .flat_map(|team| {
            team.players
                .keys()
                .map(move |slot| team.id * 100 + u32::from(*slot))
        })
        .filter(|id| {
            file.player(*id)
                .is_some_and(|player| player.appearance_id != Some(STRIPPED_APPEARANCE_ID))
        })
        .collect();
    not_stripped.sort_unstable();
    if !not_stripped.is_empty() {
        report(
            events,
            Code::SavefilePlayerNotStripped,
            vec![(
                "players",
                not_stripped
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            )],
        );
    }
    // A save holding none of the patch's teams is left as it was: writing it would change
    // nothing but turn its `.bak` over, losing the user's previous backup.
    if summary.applied_teams.is_empty() {
        return;
    }
    match file.save(&path) {
        Ok(()) => report(
            events,
            Code::SavefileWritten,
            vec![
                ("path", path.display().to_string()),
                (
                    "teams",
                    summary
                        .applied_teams
                        .iter()
                        .map(u32::to_string)
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
            ],
        ),
        Err(error) => report(
            events,
            Code::SavefileWriteFailed,
            vec![
                ("path", path.display().to_string()),
                ("error", format!("{error:#}")),
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use crossbeam_channel::{Receiver, unbounded};
    use pes_savefile::interchange::aesthetics_patch::{
        ALLOCATION_SCHEME_VERSION, PatchTeam, utc_datetime,
    };
    use studio_core::{AppPaths, PipelineEvent, PipelineEventEnvelope, Settings, ToolContext};

    use super::*;
    use crate::testing::scratch;

    /// A discovery candidate at `path`.
    fn candidate(path: PathBuf) -> SavefileCandidate {
        SavefileCandidate {
            path,
            account: None,
            modified: SystemTime::UNIX_EPOCH,
        }
    }

    /// A Fox patch holding no teams, for a call that must not reach the save's contents.
    fn empty_patch() -> AestheticsPatch {
        AestheticsPatch {
            pes_version: PesVersion::Pes21,
            studio_version: "test".to_owned(),
            allocation_scheme_version: ALLOCATION_SCHEME_VERSION,
            cpks: Vec::new(),
            compiled: utc_datetime(0),
            teams: Vec::new(),
        }
    }

    /// Run events over a context whose channel `rx` reads the emitted messages.
    fn events() -> (RunEvents, Receiver<PipelineEventEnvelope>) {
        let settings = Settings::parse("").unwrap();
        let (events_tx, events_rx) = unbounded();
        let (requests_tx, _requests_rx) = unbounded();
        let ctx = ToolContext::new(
            Arc::new(Mutex::new(settings)),
            AppPaths {
                exe_dir: PathBuf::new(),
                data_dir: None,
            },
            events_tx,
            requests_tx,
        );
        (RunEvents::new(&ctx), events_rx)
    }

    /// The code and context pairs `rx` collected.
    fn messages(rx: &Receiver<PipelineEventEnvelope>) -> Vec<(String, Vec<(String, String)>)> {
        rx.try_iter()
            .filter_map(|envelope| match envelope.event {
                PipelineEvent::Message(message) => {
                    Some((message.code.code.to_string(), message.context))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_configured_path_is_configured_or_missing_and_never_discovers() {
        let temp = scratch("savefile_locate_configured");
        let save = temp.path().join("EDIT00000000");
        fs::write(&save, b"a save").unwrap();
        let never = || panic!("a configured path runs no discovery");
        assert_eq!(
            locate(&SavefilePath::Path(save.clone()), never),
            Located::Configured(save.clone())
        );
        let missing = temp.path().join("MISSING");
        assert_eq!(
            locate(&SavefilePath::Path(missing.clone()), never),
            Located::Missing(Some(missing))
        );
        // A folder is not a file either.
        assert_eq!(
            locate(&SavefilePath::Path(temp.path().to_owned()), never),
            Located::Missing(Some(temp.path().to_owned()))
        );
    }

    #[test]
    fn auto_takes_the_first_candidate_and_reports_the_count() {
        let first = PathBuf::from("newest");
        let candidates = move || {
            vec![
                candidate(PathBuf::from("newest")),
                candidate(PathBuf::from("older")),
            ]
        };
        assert_eq!(
            locate(&SavefilePath::Auto, candidates),
            Located::Detected {
                path: first,
                candidates: 2
            }
        );
        assert_eq!(
            locate(&SavefilePath::Auto, Vec::new),
            Located::Missing(None)
        );
    }

    #[test]
    fn target_running_counts_only_the_target_version() {
        let running = |version| vec![RunningPes { version, exe: None }];
        assert!(target_running(
            &running(PesVersion::Pes21),
            PesVersion::Pes21
        ));
        assert!(!target_running(
            &running(PesVersion::Pes19),
            PesVersion::Pes21
        ));
        assert!(!target_running(&[], PesVersion::Pes21));
    }

    #[test]
    fn update_skips_the_save_while_the_target_runs() {
        let temp = scratch("savefile_pes_running");
        let save = temp.path().join("EDIT00000000");
        fs::write(&save, pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let before = fs::read(&save).unwrap();
        let running = [RunningPes {
            version: PesVersion::Pes21,
            exe: None,
        }];
        let (mut events, rx) = events();

        update(
            &mut events,
            Located::Configured(save.clone()),
            &running,
            &temp.path().join("aesthetics_patch.toml"),
            &empty_patch(),
        );

        let messages = messages(&rx);
        let [(code, context)] = messages.as_slice() else {
            panic!("one message: {messages:?}")
        };
        assert_eq!(code, "savefile_skipped_pes_running");
        assert_eq!(
            context.as_slice(),
            [
                ("path".to_owned(), save.display().to_string()),
                ("version".to_owned(), "PES 2021".to_owned())
            ]
        );
        assert_eq!(fs::read(&save).unwrap(), before, "byte-identical");
    }

    #[test]
    fn update_writes_nothing_when_the_save_lacks_every_team_of_the_patch() {
        let temp = scratch("savefile_no_team_applied");
        let save = temp.path().join("EDIT00000000");
        let mut file =
            EditFile::from_bytes(&pes_savefile::fixture_save(PesVersion::Pes21)).unwrap();
        let free = (1..).find(|id| file.team(*id).is_none()).unwrap();
        file.team_mut(714).unwrap().id = free;
        fs::write(&save, file.to_bytes(&[7; 320]).unwrap()).unwrap();
        let before = fs::read(&save).unwrap();
        let mut patch = empty_patch();
        patch.teams.push(PatchTeam {
            id: 714,
            name: "/co/".to_owned(),
            players: BTreeMap::new(),
        });
        let (mut events, rx) = events();

        update(
            &mut events,
            Located::Configured(save.clone()),
            &[],
            &temp.path().join("aesthetics_patch.toml"),
            &patch,
        );

        let codes: Vec<String> = messages(&rx).into_iter().map(|(code, _)| code).collect();
        assert_eq!(codes, ["savefile_team_missing"]);
        assert_eq!(fs::read(&save).unwrap(), before, "byte-identical");
        assert!(
            !temp.path().join("EDIT00000000.bak").exists(),
            "no .bak turned over"
        );
    }
}
