//! The AATF check the floating panel (`aatf_panel`) and the Live AATF dock
//! (`live_aatf`) share (`plans/save_editor.md` "Configurable AATF rules"):
//! the worker-thread check's key, pending state and spawn, the team both
//! Current-team callers read, the `violations_list` rows a check's results
//! draw, and a row click's effect on the view's selection. The check runs
//! on a worker thread — a whole-save pick is seconds of script — started
//! when the key moves on (the session's revision, the team ids or the
//! rules file) and the edit has settled (`edit_settled`, the same
//! condition the undo stack's seal uses); until the reply lands the last
//! result stays, marked as checking.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::Receiver;
use pes_savefile::EditFile;
use pes_savefile::model::PlayerEntry;
use team_widgets::violations::{ListedPlayer, SelectedPlayer, TeamViolations, violations_list};

use crate::session::rules::{self, TeamCheck};
use crate::session::selection::TeamChoice;

/// What a check ran for: a change of the open save, of the ids or of the
/// rules file recomputes it.
#[derive(Clone, PartialEq)]
pub(super) struct CheckKey {
    /// `Session::revision` when the views were built.
    pub(super) revision: u64,
    /// The teams checked, in id order.
    pub(super) ids: Vec<u32>,
    /// The rules file the check's ruleset came from (`None`: the embedded
    /// official ruleset).
    pub(super) rules_file: Option<PathBuf>,
}

/// A check in flight: the key it runs for and its reply channel.
pub(super) struct PendingCheck {
    /// The key the worker's `check_teams` runs for.
    pub(super) key: CheckKey,
    /// The worker's reply: its key and the checks, in `ids`' order.
    pub(super) replies: Receiver<(CheckKey, Vec<TeamCheck>)>,
}

/// A team check's state: the last result with its key and the check in
/// flight, if one is. The AATF panel and the Live AATF dock each hold one.
pub(super) struct TeamsCheck {
    /// The last check's key and its per-team results.
    pub(super) result: Option<(CheckKey, Vec<TeamCheck>)>,
    /// The check in flight, if one is.
    pub(super) pending: Option<PendingCheck>,
}

impl TeamsCheck {
    /// Nothing checked and nothing running.
    pub(super) fn new() -> TeamsCheck {
        TeamsCheck {
            result: None,
            pending: None,
        }
    }

    /// Another save opened: the result and the check in flight are of the
    /// old save's teams.
    pub(super) fn reset(&mut self) {
        self.result = None;
        self.pending = None;
    }

    /// One frame of the check cycle for `key`: the reply polled first, so
    /// the frame that starts a check still marks the previous result as
    /// checking; then a check spawned when the key moved on since the last
    /// result (a rules file that does not load, `rules` `None`, never
    /// starts one: its error is the list) and the edit has settled.
    /// Returns whether the list is checking — a check in flight or due at
    /// the next settled frame — for the callers' checking marks.
    pub(super) fn update(
        &mut self,
        ctx: &egui::Context,
        file: &EditFile,
        rules: Option<&Arc<aatf::Rules>>,
        key: CheckKey,
        settled: bool,
    ) -> bool {
        let reply = self
            .pending
            .as_ref()
            .map(|pending| pending.replies.try_recv());
        match reply {
            Some(Ok((key, checks))) => {
                self.result = Some((key, checks));
                self.pending = None;
            }
            // A worker that died without replying leaves no result to wait
            // for: drop the pending so the spawn below starts a fresh check.
            Some(Err(crossbeam_channel::TryRecvError::Disconnected)) => {
                log::warn!("the AATF check's worker ended without a reply");
                self.pending = None;
            }
            _ => {}
        }
        let due = !key.ids.is_empty()
            && rules.is_some()
            && self.result.as_ref().is_none_or(|(held, _)| *held != key);
        let pending_for_key = self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.key == key);
        if due
            && settled
            && !pending_for_key
            && let Some(rules) = rules
        {
            self.pending = Some(spawn_check(ctx, file, rules, key.clone()));
        }
        self.pending.is_some() || due
    }
}

/// The team the Current-team callers check: the selector's, or with ALL
/// the selected player's (the team the strip and tabs are editing); `None`
/// when ALL reads no selected player.
pub(super) fn current_team(team: TeamChoice, selected: Option<(u32, u8)>) -> Option<u32> {
    match team {
        TeamChoice::Team(id) => Some(id),
        TeamChoice::All => selected.map(|(team_id, _)| team_id),
    }
}

/// A violation row's click: the player selected and the selector moved to
/// his team unless it reads ALL (the list must show the row it selects).
pub(super) fn select_player(
    player: SelectedPlayer,
    selected: &mut Option<(u32, u8)>,
    team: &mut TeamChoice,
    filter: &mut String,
) {
    *selected = Some((player.team_id, player.slot));
    if *team != TeamChoice::All {
        *team = TeamChoice::Team(player.team_id);
    }
    // As `open` clears it: the row the click selects would be hidden by a
    // filter matching nothing.
    filter.clear();
}

/// The issue count the callers draw: the Error and Warning violations of
/// the checks that succeeded (suggestions are not issues; a failed check's
/// violations are unknown and count nothing — the body says why).
pub(super) fn issues(checks: &[TeamCheck]) -> usize {
    checks
        .iter()
        .filter_map(|check| check.as_ref().ok())
        .flatten()
        .filter(|violation| {
            matches!(
                violation.severity,
                aatf::Severity::Error | aatf::Severity::Warning
            )
        })
        .count()
}

/// Team `id`'s name (every checked id comes from the open file's teams,
/// the selector or the selection).
pub(super) fn team_name(file: &EditFile, id: u32) -> &str {
    file.team(id)
        .expect("every checked id is a team of the open file")
        .name
        .as_str()
}

/// The check's results: one error-coloured line per team whose check
/// failed (as `check-aatf` reports it), `clean_line` (the panel's "No
/// violations."; `None` in the dock, whose header says it) when every
/// team's check succeeded and none has an error or a warning, then the
/// `violations_list` itself. Returns the player whose row was clicked
/// this frame.
pub(super) fn results(
    ui: &mut egui::Ui,
    file: &EditFile,
    id_salt: impl egui::AsIdSalt,
    key: &CheckKey,
    checks: &[TeamCheck],
    show_suggestions: bool,
    clean_line: Option<&'static str>,
) -> Option<SelectedPlayer> {
    // Player ids are stable inside an open file: one map serves the
    // whole roster scan, as `selection::rows` builds its own (its
    // construction lives there inline).
    let players: HashMap<u32, &PlayerEntry> = file
        .players()
        .iter()
        .map(|player| (player.id, player))
        .collect();
    // One `ListedPlayer` list per checked team, for the group headers;
    // `teams` borrows it below, so it is filled first.
    let mut listed: Vec<Vec<ListedPlayer>> = Vec::with_capacity(key.ids.len());
    for (id, check) in key.ids.iter().zip(checks) {
        listed.push(if check.is_ok() {
            listed_players(file, &players, *id)
        } else {
            Vec::new()
        });
    }
    let mut teams = Vec::new();
    for (i, (id, check)) in key.ids.iter().zip(checks).enumerate() {
        let name = team_name(file, *id);
        match check {
            Err(error) => {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!("Team {id} {name}: the AATF check failed: {error}"),
                );
            }
            Ok(violations) => teams.push(TeamViolations {
                team_id: *id,
                team_name: name,
                players: &listed[i],
                violations,
            }),
        }
    }
    // Only when every check succeeded: beside a failed team's line it
    // would read as clearing that team, whose violations are unknown.
    if issues(checks) == 0
        && checks.iter().all(Result::is_ok)
        && let Some(line) = clean_line
    {
        ui.weak(line);
    }
    violations_list(ui, id_salt, &teams, show_suggestions)
}

/// Team `id`'s rostered players as the list's `ListedPlayer`s, in slot
/// order, their names read through `players`.
fn listed_players<'a>(
    file: &'a EditFile,
    players: &HashMap<u32, &'a PlayerEntry>,
    id: u32,
) -> Vec<ListedPlayer<'a>> {
    let Some(team) = file.team(id) else {
        return Vec::new();
    };
    team.roster
        .iter()
        .enumerate()
        .filter(|(_, slot)| slot.player_id != 0)
        .map(|(slot, roster)| ListedPlayer {
            slot: u8::try_from(slot).expect("a roster slot"),
            id: roster.player_id,
            number: roster.number,
            name: players
                .get(&roster.player_id)
                .map(|player| player.name.as_str())
                .unwrap_or("?"),
        })
        .collect()
}

/// One check's worker: `key`'s teams viewed on the caller's thread (the
/// `EditFile` lives there), then `check_teams` on the shared ruleset off
/// it, the key riding the reply so the caller knows what landed.
fn spawn_check(
    ctx: &egui::Context,
    file: &EditFile,
    rules: &Arc<aatf::Rules>,
    key: CheckKey,
) -> PendingCheck {
    let views = rules::team_views(file, &key.ids);
    let (sender, replies) = crossbeam_channel::unbounded();
    let rules = Arc::clone(rules);
    let ctx = ctx.clone();
    let reply_key = key.clone();
    std::thread::spawn(move || {
        let checks = rules::check_teams(&rules, &views);
        // A newer check can replace the receiver mid-flight; that send's
        // failure is the superseded thread ending, not an error.
        if sender.send((reply_key, checks)).is_err() {
            log::debug!("the AATF check's caller dropped a superseded check");
        }
        ctx.request_repaint();
    });
    PendingCheck { key, replies }
}
