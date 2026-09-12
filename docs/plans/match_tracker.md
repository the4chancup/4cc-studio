# 4cc Studio — Match tracker plan

The match tracker (`crates/tools/match_tracker`, tool id `match-tracker`, sidebar
label "Match tracker" — unlike Rigdio/RigDJ the old name has been out of use for years
and does not stay in the label) is the successor to **SEN:P-AI**, the live
match-stats extractor used on stream alongside PES 2017.

This plan also specifies **`libs/match_feed`**, the small lib crate through which the
tracker publishes live match events to the rest of the suite — primarily to the
[Music player](music_player.md), which uses the feed to run the soundboard
autonomously (its "Autopilot" section specifies that behavior). Platform context is
in the [core plan](core.md).

Unlike every other tool in the suite, the tracker's primary input is not files but the
**memory of the running game process**. Live tracking is Windows-desktop-only (see
"Platform notes"); opening and viewing saved match files (`.match.json`) works on
all platforms.

---

## Background

SEN:P-AI (`Tools_4cc/SEN_P-AI`, C++/Qt, v1.2.6) attaches to a running PES 2017,
locates the in-memory team and stat tables, and turns raw stat changes into a live
match view: score, reconstructed game clock, an event log (goals, cards, subs), and a
full per-player stats table. It saves matches, including in-progress manual or timed
autosaves, as binary `.sen` files and broadcasts events as length-prefixed JSON over a
named pipe — the protocol Rigdio
v1.9–v1.10 consumed for automatic event clips, until Rigdio removed its client in
v1.11 (2020). It has not been used on stream in a while, but it is the only map of
the game's in-memory match structures.

| Tool | Role | Location |
|------|------|----------|
| SEN:P-AI | Live PES17 match stat/event extraction (stream side) | `Tools_4cc/SEN_P-AI` |

### Unofficial fork (unverified findings)

A community-made unofficial update of SEN:P-AI exists at `Tools_4cc/SEN_P-AI-Unofficial`.
Its quality and stability are not trusted, so it is **not** an architectural reference —
only a source of reverse-engineered game knowledge to mine and then verify independently.
The findings below are attributed to it and marked **(unverified)** until confirmed against
a live game; they are collected in the "Verification" section under Testing. What the fork
adds over the original, relevant to this plan:

- **Partial PES 2021 profile**: a substantially mapped second game version — different
  process name (`PES2021.exe`, 64-bit), a pointer-chain stat-table locator, an 8500-byte
  stat entry, indicator-based entry validation, and player/team heatmap offsets. The fork
  declares a 269-slot stat catalog, but only 263 slots are explicitly initialized and the
  table contains duplicate legacy `visualIndex` values and overlapping definitions. It is
  an internally inconsistent reverse-engineering lead, not a directly reusable schema.
  Most segmented counters use 9 buckets on both versions, while "Game Ticks Played" uses
  13 on both. The profile declares up to 40 slots per team, but the fork's operational
  reader still uses 32-slot structures (`StatTableReader.h` defines `MAX_PLAYERS` as 32
  and `teamInfo::player[32]`), so support above 32 remains unverified. PES21 possession
  remains an unresolved TODO, and clock start/stop derivation is not wired for PES21.
  **(unverified)**
- **Pointer-chain acquisition**: an ASLR-proof locator that follows a static-RVA pointer
  chain to the stat-table base. The fork wires it into the operational PES21 reader and
  defines a PES17 profile for it, but does not wire that profile into the operational
  PES17 reader, so the PES17 path remains pending verification. The original used an
  exact-region-size scan as its normal path and a manually triggered whole-RW-region
  content scan as its fallback; it had no pointer-chain fast path. **(unverified)**
- **Substitution journal (PES2021)**: a per-match record at a fixed offset from the
  stats-table base that names *both* the incoming and outgoing player and the exact
  minute — the original could only see the incoming player via first-minute-on-pitch.
  **(unverified)**
- **Direct clock read (PES2021)**: a pointer chain yielding the current segment and a raw
  tick count, converting directly to game minute + injury time — vs the original's
  mode-of-last-minute reconstruction. **(unverified)**
- **WebSocket remote ingest**: a push client to a remote server with a spectator share URL,
  alongside the original named pipe. Not adopted (see "External consumers"), but noted as a
  prior art data point for a future web-spectator feature.

Two things the fork carries from the original are also relevant here: the QR-timestamp
widget, and the **wiki footballbox formatting primitives** in `WikiFootballbox.h`.
`footballBox::toWikiText()` is a generic parameter-map serializer for fields such as
teams, score, date/time and goals; it is not a scorer/minute/card assembler. The original
only declared `Match::toWikiFootballBox()` and never implemented it. The fork adds a
separate partial match adapter for teams, score, goals, and own goals, but omits cards,
date, and time and does not use the generic `footballBox` builder. `WikiEditor`, the
wiki-API client that would round-trip output through the wiki, is unfinished in both.
The suite therefore implements a new, tested exporter informed by these primitives,
rather than porting a supposedly complete generator (see "Match files").

**Priority.** The first implementation targets PES 2017; the fork's PES21 knowledge is
inactive reference data until PES21 support is pursued (see "Verifying source-derived
findings"). The main
source-verification priorities are the original's hypothetical PES17 phase buckets 6–8,
the one PES17-relevant pointer-chain claim the fork makes, and an independent heatmap
viewer implemented from verified grid semantics at PES17's known offset (see "Game
structures"). If the PES17 grid is verified to populate with stable semantics, heatmaps
ship in the first implementation;
the new footballbox exporter does not depend on the fork's heatmap or PES21 findings.

### Relationship to the old codebase

As with the other tools, SEN:P-AI defines *what* the tracker must do — the memory
signatures, the event derivation rules, the clock reconstruction — and its structure
headers (`TeamDataTable.h`, `StatsInfo.cpp`) are reverse-engineering leads for a separately
documented, independently verified layout specification. It is not an architectural
reference:

- **Qt signal/slot coupling throughout**: the memory reader emits UI strings
  ("SEN:P-AI noticed the teams!") directly, the match/event logic lives inside a
  QWidget (`MatchReader`) and mutates table widgets as it derives events, and the
  clock state is spread across widget fields.
- **Global mutable stat catalog**: a 175-entry static array with static lookup maps,
  mutated in place by the column-selection dialog.
- **The `.sen` format**: a hand-rolled binary section format whose reader and writer
  are one templated function running in both directions via macros.
- **Half-finished features**: the footballbox primitives and unimplemented match
  adapter described above, and a pipe server whose only
  known consumer removed its client years ago.

**Licensing constraint.** Both SEN:P-AI repositories are GPLv3. 4cc Studio uses an
independent reimplementation based on independently verified factual layouts and observable
behavior; the legacy source is a reverse-engineering reference, not a code source. Do not
copy or adapt the fork's rendering or reducer code. The core plan's future
project-wide licensing section must reflect the same constraint; this plan does not change
that file.

**Status:** pre-implementation architectural plan. The Rust crates and APIs shown below
are proposed contracts; legacy-source claims are evidence to verify, not implemented
4cc Studio behavior.

---

## Crate layout

The tracker follows the standard tool-crate skeleton (core plan, "Tool crates"). Its elaboration
exists for one reason: the plan promises that live tracking is *Windows-only behind a trait* while
everything else (match model, files, view, feed publishing) is portable — and that promise is a
placement rule, not a comment.

```
crates/tools/match_tracker/src/
├── lib.rs              # Tool — wiring only; disables Start tracking when no memory source exists
├── settings.rs         # poll rate, autosave, match files folder, footballbox export options
├── cli.rs              # watch / replay / export subcommands
├── messages.rs
├── memory/             # reading the game — the only platform-specific code in the crate
│   ├── mod.rs          #   MemorySource trait (find process, read region, enumerate RW regions)
│   ├── win32.rs        #   #[cfg(windows)] backend: process discovery, ReadProcessMemory, elevation diagnosis
│   ├── replay.rs       #   recorded-snapshot backend for tests and the replay CLI
│   └── profiles/       #   per-version signatures — data only
│       ├── mod.rs      #     VersionProfile: table sizes, strides, pointer chains, readiness flag
│       ├── pes17.rs
│       └── pes21.rs    #     (unverified entries marked as such in doc comments)
├── acquire/            # the acquisition ladder, source-agnostic
│   ├── mod.rs          #   ladder state machine: find process → teams → stats → diff; demotion/re-acquire
│   ├── scan.rs         #   size-filtered region scan, deep-scan escalation, content validation
│   └── chain.rs        #   pointer-chain fast path
├── model/              # what a match is
│   ├── mod.rs          #   Match, Team, PlayerStats, lifecycle (TeamsDetected → Live → Finalized)
│   ├── diff.rs         #   stat diffing → events
│   ├── clock.rs        #   clock reconstruction (ET, penalties)
│   └── boundary.rs     #   session-boundary detection and staged finalization
├── files.rs            # match file JSON save/load, autosave, recovery
├── publish.rs          # match_feed publisher: model changes → feed events (prepare/commit pattern)
└── view/
    ├── mod.rs          #   match tabs, Start/Stop, source state indicator
    ├── stats.rs        #   stats table
    ├── events.rs       #   event log
    └── heatmap.rs      #   heatmap / footballbox export view
```

Placement rules:

- **`memory/win32.rs` is the only `#[cfg(windows)]` in the crate**, and `windows`-crate imports
  appear nowhere else. A Wine `/proc` backend would be a sibling file implementing the same trait.
- **`acquire/` and `model/` never import `memory/win32`**; they take a `&dyn MemorySource`. This is
  what makes the replay backend a complete test harness: every ladder and model test runs on
  recorded snapshots with no game present.
- **`memory/profiles/` is data.** Offsets, strides, and chains live there with their verification
  status; `acquire/` reads them, never hardcodes them.
- **`publish.rs` is the only module that knows about `match_feed`**, so the Music player's autopilot
  contract has one place to look.
- The saved-match viewing path (`files.rs` + `view/`) has no dependency on `memory/` or `acquire/`,
  which is how it remains available on non-Windows and in Studio Web.

---

## Reading the game

### Game structures

PES17 exposes two relevant memory-resident data sources. The roster/team-data table is
found by scanning committed private read-write regions of the known size and validating
its contents. The player-stat blocks use a version-profile pointer-chain fast path where
that chain has been verified, with validated region/content scanning as fallback (see
"Acquisition and polling"):

- **Team data table** (fixed offset inside a 384KB region): both teams' names, IDs,
  scoreboard abbreviations and banner texts, 32 player slots per side (validity
  marker, player ID, name, print name, height/weight, condition arrow), shirt
  numbers, manager name/ID, and the full tactics setup (presets, formations,
  advanced instructions, set-piece takers). A candidate is accepted only when each
  side has at least 11 entries with the correct home/away validity marker and
  team-consistent player IDs, and both team IDs and names are plausible.
- **Stats table** (home and away blocks at fixed offsets inside a 576KB region): one
  4784-byte entry per player carrying 175 stat fields. The identified ones include
  goals, own goals, assists, cards, the shot/pass/tackle breakdowns, saves,
  first/last minute on pitch, current position, game ticks played, player rating,
  and stamina; the rest are unidentified and are tracked anyway. Most counters are
  **segmented into 9 buckets**: buckets 0–2 are confirmed as the first half and 3–5 as
  the second half. The legacy code hypothesizes that 6–8 cover extra time and penalties,
  but their exact mapping is not registered until verified from recorded PES17 extra-time
  and shootout matches. Until then those combinations produce `Phase::Unknown`. A
  candidate is accepted only when
  each block has at least 11 consecutive entries with plausible player IDs and a threshold
  matching the corresponding detected roster/team. The original's `70100 ≤ id < 90300`
  heuristic is reference evidence, not Studio's acceptance range: the suite supports team IDs
  701–920, and roster membership is authoritative for validation. Home and
  away indicators or IDs must also agree with side assignment.

Signatures — process name, region sizes, offsets, entry layout, and the stat catalog —
are **per-game-version data, not code**. PES17's are known from the original; PES21's are
**claimed by the unofficial fork (unverified)** and add a substantially different layout:
8500-byte entries (vs 4784), a declared 269-slot catalog (vs 175) with only 263 explicit
initializers, duplicate `visualIndex` values, overlapping definitions, and an expanded set
of named stats at different offsets, a profile maximum of 40 player slots (vs 32; the
fork's operational 32-slot limit leaves slots 32–39 unverified), a 64-bit
process, and indicator-based entry validation (`[+0]=u16` 0xFFFD home / 0xFFFE away,
player ID at `+4`) instead of the ID-range check. Segmentation is per-stat, not
per-version: most counters have 9 phase buckets on both, and "Game Ticks Played" has 13
on both. Both versions define a 140×u32 field named "Play Area Grid" — PES17 at
entry+0x1034 and PES21 at entry+0x1938. These are candidate heatmap grids; spatial heatmap
semantics remain unverified as described below. The original mapped the PES17 grid and
processed it only through the generic stat path as a summed scalar column, hidden by
default; it never interpreted or rendered the 10x14 grid as a heatmap. The fork added the
reading and rendering code for PES21.
The PES21 stats-table base also exposes 8 per-team grids in the fork. The suite's global
PES version selector picks among registered, verified signature sets, and the tool
reports itself unavailable for versions without one; the first release registers PES17
only.

### Acquisition and polling

- A background thread polls at a configurable rate (default 100ms) through the
  acquisition ladder *find process → find teams → find stats → diff stats*, demoting
  one rung when a read fails (game closed, tables reallocated, match exited) and
  re-acquiring automatically. The team table is re-read every tick. A validated
  team-identity change after the current match has reached `Live` is itself a positive
  session boundary and initiates staged boundary finalization: the complete
  old-`MatchFinalized`/new-`TeamsDetected` batch goes through the staged
  prepare/reserve → durable write → infallible commit pattern defined under "The match feed".
  On write failure the prior lifecycle remains
  authoritative, the UI marks the tab as finalization-blocked, acquisition does not apply
  the new session to the old match, and the detected replacement-team state is retained for
  retry. Before first
  acquisition reaches `Live`, team-identity changes replace the provisional
  `TeamsDetected` draft without finalizing an empty match. A same-team rematch additionally
  requires a readiness not-live→live cycle **plus** complete counter/clock reset and fresh
  progression, then uses
  the same staged boundary-finalization path. When the readiness flag is unavailable, the
  equivalent positive signal is a stat-table disappearance or reallocation followed by a
  complete clock/counter reset and fresh live
  progression; a reset observed in place without that session boundary suspends for
  confirmation rather than finalizing immediately. A table address change alone triggers
  reacquisition, not a new match.
- **Stat-table location — pointer chain first, scan as fallback** *(unverified for
  PES21; defined in the fork's PES17 profile but not wired into its operational reader,
  pending verification)*: the fork follows a static-RVA pointer chain
  (`moduleBase + rootRVA → … → stats-table base`) to a match/stat context from which
  home stat entry 0 is reached at `+0x3804` on PES17 or `+0x64D8` on PES21. This is
  ASLR-proof and far cheaper than region scanning; it does **not** locate the separate
  roster/team-data table. A readiness flag along the chain reports whether a match is
  live, so the ladder can short-circuit before kickoff. Pointer chains break across game
  patches, so the content-signature scan stays the fallback and the deep-scan escalation
  below still applies when the chain goes stale; the chain is just the fast path when it
  resolves.
- **Deep-scan escalation**: when the teams are found but the stats table is not (the
  pointer chain is stale, or PES occasionally allocates the table in a region that misses
  the size filter), the ladder escalates automatically: the cheap size-filtered scan
  keeps running every tick, and a deep scan searches every RW region for several known
  home-roster player IDs, validates entry stride and team consistency, and confirms the
  away block at the profile's relative offset. It must not depend solely on the
  historical `team_id * 100 + 1` assumption. The deep scan runs on a throttle (every
  few seconds) until the table turns up. The original gated
  this behind a manual **"Search Harder"** button because a full-memory scan was too
  costly to run continuously even SIMD-accelerated; throttled on the polling thread, a
  plain chunked scan is cheap enough to need no button (before kickoff, when the table
  genuinely doesn't exist yet, the extra scans are bounded idle work). The run strip
  shows when the deep scan is active. The deep scan uses a time/byte budget per slice
  with exponential backoff and cancellation, so it never blocks the 100 ms polling
  cadence; benchmarks on the intended streaming machine confirm the budget before
  release.
- The polling thread belongs to the tool, not its view — tracking continues while the
  streamer has the music player's view active. A Start/Stop tracking toggle controls
  the thread. Each tracking run owns a cancellation token and monotonically increasing
  generation ID. Stop or tool drop cancels and joins the worker before addresses or
  profile state are reused; messages from older generations are discarded.
- **PES version change while tracking is blocked**: changing the global version
  selector while the tracker holds live addresses and `DecodedGameSnapshot` state would
  interpret memory with the wrong pointer width or layout. The tracker exposes a
  version-change guard to the shell while tracking is active, via the
  `version_change_blocker` hook on `StudioTool` (see the core plan's "Tool plugin
  interface"). Stopping tracking suspends the current match and clears addresses before
  the new profile takes effect. If the PES version then changes while a suspended draft
  exists, that draft remains open under its original `game_profile_id` but is detached
  from live acquisition and cannot be resumed with the new profile. The
  `FeedReleased { reason: ProfileChanged }` transition goes through the staged
  prepare/reserve → durable write → infallible commit pattern defined under "The match feed";
  its durable-write step atomically converts any active recovery marker for the draft to
  `offline_only` and records the release reason. Commit publishes reliable `FeedReleased`
  for the old `match_id`
  and clears the retained snapshot and watch, after which the tracker commits the draft's
  offline binding state. The draft remains available locally as an offline draft.
  Saving, finalizing, or discarding that offline draft does not publish to `match_feed`;
  only the currently feed-bound draft may publish lifecycle or game events.
  `FeedReleased { reason: ProfileChanged }` uses `UserAction` provenance because it follows
  the explicit version-change command. Future release reasons, including tool shutdown and
  explicit release, are deferred until their operational paths are defined. The next Start
  begins a fresh acquisition run; the next successful team detection creates a new feed-
  bound `TeamsDetected` draft with a new `match_id`. Explicit Finish is the ordinary
  finalization path.

All Win32 access (`OpenProcess`/`VirtualQueryEx`/`ReadProcessMemory` via the
`windows` crate) is isolated in a memory-source trait, so the match logic runs
identically against a live process or recorded `RawTableSnapshot` fixtures (see
Testing).

---

## Match model and event derivation

The tracker is a deterministic reducer over
`(prior MatchState, current DecodedGameSnapshot)`, producing a new state and zero or
more events. Raw stat deltas are pairwise `DecodedGameSnapshot` comparisons; lifecycle,
clock, correlation, and recovery logic additionally use
explicit reducer state. This is the part `MatchReader` buried in widget code,
restated as plain logic:

**Match lifecycle.** A match transitions through explicit states. The complete lifecycle
state is represented in snapshots, including the reason for suspension:

```rust
pub enum LifecycleState {
    TeamsDetected,
    Live,
    Suspended { reason: SuspensionReason },
    Finalized,
}
```

No active draft is represented by tracker-local `Idle` and the absence of a retained
snapshot. Abandonment is represented by the reliable `MatchAbandoned` event, after which
the broker retains no snapshot. Neither is a retained `MatchSnapshot.lifecycle` value.
Short source failures debounce within a reacquisition grace period before suspension is
published; a recovery autosave fires at most once per suspension episode. Footballbox
export requires `Finalized` and a classification-complete score; ordinary Save can save a
draft from any state.

The "Retained snapshot?" column describes a match while it remains feed-bound. Feed
release is orthogonal to lifecycle: it clears the retained snapshot and watch while leaving
the released match's reliable cursor terminal until a new match replaces it; later offline
save, finalization, or discard does not restore the cleared state.

| State | Entry condition | Score authority | GUI tab | Retained snapshot? |
|-------|-----------------|-----------------|---------|--------------------|
| `TeamsDetected` | Team table located, rosters read | No | Current Match (provisional) | Yes |
| `Live` | Stats table acquired and at least one valid live clock observed; motion is `Running` or `Stopped` during normal observation and may be `Unknown` during reconciliation | Yes | Current Match | Yes |
| `Suspended { SourceLoss }` | Read failure past debounce grace, or saved `Live` draft normalized during recovery before source reacquisition | Yes (if previously Live) | Current Match (paused) | Yes |
| `Suspended { UserStopped }` | User pressed Stop tracking | No | Current Match (paused) | Yes |
| `Finalized` | Finish after reaching `Live` / different-team boundary / verified rematch boundary | No | Finished tab | Yes after standalone feed-bound Finish; transient and replaced by the new `TeamsDetected` snapshot in a session-boundary batch; otherwise no |
| (no draft) | Idle / searching | No | — | No |
| (abandoned) | `MatchAbandoned` published | No | — | No (cleared) |
| (offline draft) | `FeedReleased { ProfileChanged }` | No | Offline draft tab | No retained snapshot/watch; the released match's terminal reliable cursor remains until replacement |

Table loss triggers `Suspended`, not immediate finalization. A source/read-loss
suspension retains the music player's loaded teams, score, and pending one-shot metadata
for deduplication; on resumption, a pending horn proceeds only if its deadline has not
expired and the reconciled clock state proves the same post-goal stopped window (same
`stop_epoch`), otherwise it is canceled. A stale pending horn never starts merely because
the feed resumed. The authority table below summarizes which events acquire, release, or
retain authority and which cancel pending automation.

Feed score authority is carried in `MatchSnapshot` via explicit `has_reached_live` and
`score_authority` fields; consumers must not infer it from clock presence. Autopilot mode
controls playback and highlighting only; selecting `Off` does not transfer score
authority. `MatchResumed` reconciles from the tracker snapshot without merging ad hoc
manual score changes. Manual goalhorn presses during source-loss suspension remain
non-scoring soundboard actions.

| Event | Acquires authority? | Releases authority? | Cancels pending automation? | Retains snapshot/watch? |
|-------|--------------------|--------------------|-----------------------------|-------------------------|
| `TeamsDetected` | No | No | No | Snapshot retained/replaced; prior watch cleared |
| `MatchLive` | Yes | — | No | Retains |
| `MatchResumed` | Yes | — | Conditional — preserves only when the snapshot proves the same stopped `stop_epoch`; otherwise cancels | Retains |
| `MatchSuspended { SourceLoss }` | — | No (retains if was Live) | No (pauses) | Retains |
| `MatchSuspended { UserStopped }` | — | Yes | Yes | Retains |
| `MatchFinalized` | — | Yes | Yes | Retains terminal snapshot/watch unless followed atomically by new-match `TeamsDetected`, which replaces the snapshot and clears the old watch |
| `MatchAbandoned` | — | Yes | Yes | Clears |
| `FeedReleased` | — | Yes | Yes | Clears retained snapshot/watch; retains the terminal reliable cursor until replacement |

While the feed is score authority, no manual soundboard action may alter authoritative
score, scorer totals, discipline, or participation state. Undo, per-team Reset, and team
load/reload affect playback state, caches, and loaded audio only. Once authority is
released, those controls resume their ordinary manual match-state behavior. `Snapshot`,
`MatchLive`, `MatchResumed`, and `ScoreReconciled` are reconciliation messages and never
trigger one-shot automation.
Score correction while the feed is authoritative is not supported in the first
implementation. The user may stop or abandon an active match to return the Music player to
manual score accounting; correcting the tracker's canonical event history is future work.
`MatchFinalized` releases feed score authority: consumers return to manual score
accounting for any post-finalization interaction, while the finalized match's terminal
lifecycle and saved result are unchanged. `MatchFinalized` neither stops currently playing
audio nor triggers victory automation.

**Cross-plan alignment:** the Music player plan's Autopilot section mirrors the
authority, automation, and reconciliation rules in the tables above (uncertain-goal
resolution via `resolves_unclassified`, reconciliation-only snapshot handling, monotonic
announcement deadlines, `stop_epoch`-verified horn continuity, `SecondYellow` → `event
red`, shootout isolation, per-side auto team loading, and the `Off` mode's
no-authority-transfer semantics); the consumer-side rules live in that plan and in the
core plan's key-decision summary.

`Finalized` is terminal for that `match_id`, and `MatchAbandoned` clears that
`match_id`: no further stat or event deltas are applied to either. The acquisition worker
may continue only to detect the next positive session boundary; it suppresses the
still-running source session and cannot create another draft from the same table/readiness
cycle.

- **Stat deltas**: every stat of every player is compared per update; segmented stats
  are compared per segment. Profiles classify verified fields as monotonic counters,
  gauges, state values, or grids. Unidentified scalar fields use `UnknownScalar`: their
  changes are recorded, but they never generate match events and their decreases are not
  used as invalid-read, reset, or session-boundary evidence until their semantics are
  verified. Each accepted change lands in the per-player stat history, referencing a
  shared timeline entry (optional UTC wall-clock timestamp + reconstructed clock).
  Decreases of gauges and state values, such as stamina, rating, and position, are ordinary
  changes. Only positive increments of profile-declared event counters can create counter-
  derived player events. Verified first-minute participation transitions and substitution-
  journal additions may create `SubOn`; clock and lifecycle events are produced only by
  their dedicated state machines. Decreases of profile-declared monotonic counters never
  synthesize inverse events or silently alter
  canonical score or history. A broad reset starts a new match only when corroborated by
  the session-boundary rules above; otherwise the snapshot is rejected and acquisition
  suspends or reacquires pending confirmation. Isolated implausible monotonic-counter
  decreases are treated as invalid-read evidence. Rejected snapshots are recorded only in
  diagnostics, not canonical match history.
- **Grid fields**: grid/blob fields such as "Play Area Grid" are excluded from the generic
  scalar stat-delta history. They are sampled on the profile's heatmap throttle and stored
  only in the optional latest-grid section. A scalar aggregate may be computed for
  diagnostics or compatibility display, but it is not persisted as per-cell stat history.
- **Events**: goal, own goal, card, dismissal, and sub-on. The initial goal holdback
  waits at most two polls or 250 ms, whichever comes first, for both assist and goal-kind
  correlation evidence. If a suspension, generation/session boundary, phase boundary, or
  termination closes the holdback, every confirmed goal delta is flushed immediately with
  the evidence currently available when its score domain is known; otherwise it is
  materialized as an `UnclassifiedGoal`. Closing a boundary never discards a confirmed
  positive goal delta. A goal is otherwise emitted when the holdback expires, without an
  assist if none was correlated; `Goal.clock` is captured from the
  `DecodedGameSnapshot` containing the goal delta, not from the later timeout. Later assist
  or goal-kind evidence updates the stored canonical event and publishes a presentation-
  only `GoalAmended` carrying the complete current assist and kind. It never re-triggers
  scoring or automation. A goal's amendment record closes on the first subsequent kickoff
  (`ClockStarted`), another potentially conflicting goal, a suspension or generation
  change, a phase boundary, or match termination. Recovered aggregate
  assist deltas never amend live goals. Kickoff is therefore a closing boundary, not an
  extension of the initial assist-correlation timeout. Multiple simultaneous goals/assists
  that make pairing ambiguous are emitted without pairing. On PES17 the outgoing player is
  not identifiable from the stats (first-minute-on-pitch only reveals the incoming
  player; the original defined a slot for the outgoing player that never got filled,
  and the feed's `player_out` is optional there). **On PES21 the fork reports a
  substitution journal** *(unverified)* at a fixed offset from the stats-table base — a
  count byte plus 40-byte records carrying team, period, minute-in-period, and both
  `inIdx`/`outIdx` — so `player_out` becomes exact and the sub minute is the recorded
  value, not the live clock. The journal lives in the game's memory and survives a
  tracker restart, so a re-attach replays existing records stamped with their own
  recorded minute.
- **Recovered aggregate state**: when the stats table is first acquired mid-match,
  pre-existing counters seed the recovered aggregate and any score, scorer, and card
  state whose domain is known, but do **not** create ordinary timeline events because
  their minutes and ordering are unknown. Goals whose score domain is unknown become
  `UnclassifiedGoal` records and do not enter ordinary `scorer_counts` until classified;
  shootout goals never enter ordinary `scorer_counts`. The match file stores initial
  counters in a distinct `recovered_aggregate` section and sets `history_complete: false`.
  Journal records carrying their own recorded match minute may create
  `RecoveredHistorical` timeline events. Neither form triggers live one-shot automation
  (see "The match feed" — provenance). `history_complete` is provisionally true at match
  creation and is confirmed on first stats acquisition only when observation begins before
  live counters advance; that first acquisition sets it false if it reveals pre-existing
  counters. It becomes permanently false for that match if the tracker attaches late or
  reacquisition reveals counter changes whose ordering or time was not observed. A
  suspension with no intervening counter changes does not by itself make history
  incomplete. While `has_reached_live == false`, `history_complete == true` is provisional
  and means only "not yet disproven"; readers and exporters must not treat history
  completeness as confirmed until the match has reached `Live`. First stats acquisition
  either confirms the value or sets it permanently false.
- **Penalty and shootout classification**: a same-player "Penalties scored" delta
  correlated with a "Goals Scored" delta marks a regulation or extra-time penalty goal.
  Shootout attempts are classified only after the phase detector has positively entered
  the verified penalty phase. `OpenPlay`, `Penalty`, and regulation-phase `Unknown` goals
  update regulation score and ordinary scorer counts. `Shootout` updates only
  `shootout_score` and does not affect the music player's `goals`, `teamgoals`, `first`,
  or `mostgoals` condition state. `GoalKind::Unknown` is published only when the phase is
  positively known to be regulation and the remaining ambiguity is open play vs.
  penalty. If the score domain of a **live-observed** goal is unknown, the reducer retains
  the delta as an attributed `UnclassifiedGoal` in `Pending` state, updates
  `Score.unclassified` for the benefiting/scoring side, and publishes the snapshot-bearing,
  presentation-only `ScoreReconciled { reason: UnclassifiedGoalObserved }`. It publishes no
  automation-eligible `Goal` until the phase is classified. An own-goal delta observed while
  the score domain is unknown is retained as `GoalAttribution::OwnGoal`; once classified, it
  emits `OwnGoal` only for a regulation-domain resolution, while any other domain publishes
  `ScoreReconciled { reason: ClassificationResolved }`. `OwnGoal` therefore has no
  `GoalKind` field: it is emitted only
  after regulation-domain classification, so no kind discrimination is needed. A record
  materialized from recovered aggregate counters is incorporated into the next `MatchLive`
  or `MatchResumed` snapshot and publishes no standalone `UnclassifiedGoalObserved` event.
  Each live-observed record retains its stable match-scoped ID, attribution, original clock,
  observed segment evidence, and ordering needed for later classification; recovered
  aggregate records use the optional fields described in the feed model below.
  If classification remains impossible, its state becomes `Permanent`. When a record
  becomes `Permanent` before finalization, the reducer publishes the snapshot-bearing,
  presentation-only
  `ScoreReconciled { reason: ClassificationBecamePermanent }`. At feed-bound finalization,
  any required `Pending` → `Permanent` reconciliation is published in the same atomic
  batch immediately before `MatchFinalized`; offline finalization applies the same change
  only to its durable local revision and publishes nothing. Neither transition alone
  changes `history_complete`; that flag changes only under the attach/reacquisition and missing-
  order/time rules above. The reducer does not guess or trigger one-shot automation.
  `Pending` records remain eligible for later classification across kickoff, suspension,
  and other automation-closing boundaries; those boundaries determine only whether
  eventual resolution publishes an automation-eligible `Goal` or `OwnGoal`, or a
  presentation-only `ScoreReconciled`. A record becomes `Permanent` at finalization, or
  earlier only when the
  reducer can prove that no future observation in the match can resolve its score domain.
  `Permanent` means automatically unresolvable in the first implementation; a
  future explicit correction workflow may still resolve it. `Score.unclassified` is the
  per-side projection of the canonical `unclassified_goals` collection, not the only
  stored representation; `classification_complete` is true exactly when that collection
  is empty, independently of `history_complete`. Unclassified totals are displayed as
  uncertain audit state, serialized with their underlying records in snapshots and match
  files, and never contribute to regulation music conditions or one-shot automation. For
  a live-observed record, if classification completes while the original post-goal
  automation window is still open, the reducer publishes the ordinary `Goal` with its
  original clock and atomically moves the record into the appropriate score domain. An
  automation-eligible `Goal` that resolves an existing `UnclassifiedGoal` carries that
  record's stable match-scoped ID in `resolves_unclassified`. Consumers remove that exact
  uncertain record before applying the classified score/scorer update; `None` denotes a
  newly observed goal that was classified immediately. An automation-eligible `OwnGoal`
  that resolves an existing attributed `UnclassifiedGoal` likewise carries that record's ID
  and moves it out of uncertain score before applying the benefiting-side regulation score;
  `None` denotes an own goal classified immediately. If classification completes after
  kickoff, suspension, or another closing boundary, it instead publishes the snapshot-
  bearing, presentation-only `ScoreReconciled { reason: ClassificationResolved }`; that event
  updates the appropriate score domain and updates ordinary scorer state only for a
  classified non-shootout goal; it is never eligible for one-shot automation. A record
  materialized from recovered aggregate counters always uses
  `ScoreReconciled { reason: ClassificationResolved }` when later classified. The Music
  player plan needs the corresponding rule that a resolving `Goal` or `OwnGoal` moves the
  existing uncertain record rather than adding a second goal. The Music player treats
  shootout-horn playback as a separate setting (default off) and never processes a shootout
  success as a regulation goal.
- **Announcement deadline anchoring**: live observations retain their original in-process
  monotonic observation instant. The Music player computes `Goal` and `OwnGoal`
  announcement deadlines from that instant, not from message receipt or UTC wall time. A
  delayed-classification goal uses only the remaining delay; if the deadline has passed,
  it may play immediately only while the same stopped window is still proven open. A
  `UnclassifiedGoal` restored in `Pending` state from disk has no valid in-process
  monotonic observation anchor. Even if its score domain is classified before the
  apparent post-goal window closes,
  restoration may publish only presentation/state reconciliation (`ScoreReconciled`, or a
  recovered-history notification followed by reconciliation), never an automation-
  eligible `Goal`. UTC timestamps must not be used to reconstruct the deadline. The Music
  player plan needs the corresponding deadline-anchoring rule.
- **Cards and dismissals**: a same-player red delta correlated with the player's second
  yellow produces one `SecondYellow` dismissal, not separate yellow and red events.
  Yellow/red correlation uses the same maximum of two polls or 250 ms, whichever comes
  first; either half of a possible second-yellow pair may open the holdback. `Card.clock`
  is captured from the snapshot containing the first relevant delta. If the window closes
  without a valid pair, the reducer emits the individually supported `Yellow` or
  `StraightRed` event using that original clock; boundary closure never discards a
  confirmed card delta. The holdback must not cross a suspension, phase or session
  boundary, another card for the player, or match termination. `SecondYellow` fires the
  music export's `event red` clip.
- **Phase detection**: for PES17, phase is inferred from the highest active 9-bucket
  segment, using the verified segment map for buckets 0–5. Buckets 6–8 remain hypothetical
  and produce `Phase::Unknown` until verified; ambiguous or unverified segment combinations
  likewise produce `Unknown` rather than guessing. Most counters share this layout on both
  versions.
- **Clock reconstruction**: on PES17 the displayed minute is the *mode* of "Last
  minute on pitch" across all players (the mode defeats per-player glitches caused by
  subs). The original's code comments describe a mode but its `std::max_element` on
  the count map actually returns the maximum minute (lexicographic pair comparison),
  not the most frequent one. The suite implements the actual mode and excludes invalid
  values and players not yet participating, determined from roster, first-minute, and
  current-position evidence; it does not discard minute zero from confirmed starters.
  A tie prefers the candidate nearest the previous reconstructed minute, then the lower
  minute. The tick count is the max of "Game ticks played" (summed across its 13
  segments per player). Seconds are interpolated from ticks using an adaptive ticks-per-
  minute estimate. Its initial value uses a fixed profile seed of a 10-minute match
  (`32 × 10` ticks per game minute), matching the legacy default at
  `MatchReader.h:74`; the reducer auto-calibrates against the recorded minute→tick history
  and records the detected match length. Injury time is
  detected when the minute pins at 45/90/105/120; halves, extra time, and penalties
  are tracked through phase transitions. Clock stop/start is detected from tick
  stagnation through an explicit debounce state machine: a stop is published only after
  at least two consecutive valid snapshots with unchanged ticks, and a restart only after
  two consecutive advancing snapshots. Clock debounce uses profile-owned
  `stop_debounce_ms` and `start_debounce_ms` constants verified from fixtures; both the
  two-sample condition and the corresponding elapsed-time minimum must be satisfied. The
  elapsed-time and event-clock anchor is the first unchanged or advancing sample,
  respectively. Read failures and phase transitions are not treated as clock samples. The
  verified PES17 values are recorded in the profile before release, so changing the poll
  rate does not materially change behavior. **On PES21 the fork reads the clock directly**
  *(unverified)* via a
  pointer chain to a clock object exposing the current segment (u32) and a raw tick
  count (f32, ticks/3600 = minutes); the segment maps to the period base minute and
  even segments flag injury time. The direct read supersedes the reconstruction where
  available; the reconstruction remains the PES17 path and the fallback if the PES21
  clock chain breaks.
- **Stop reasons**: inferred from which counters moved around the stop — offside
  beats foul, goal/own goal, a save implies a corner, an off-target shot implies a
  goal kick — replicating the original's heuristic table. A half-end reason (the
  original defined it but never actually inferred it) falls out of the clock model
  instead: a stop landing on a phase boundary.
- **The post-goal window**: a goal freezes the clock through the celebration and the
  replay; the next `ClockStarted` is the kickoff, a few seconds after the replay is
  dismissed. That stop/start pair brackets the goalhorn window the music player's
  autopilot uses (horn after its announcement delay, fade on the restart). Replay/menu
  state itself is **not** present in the known tables — if a game-state flag is ever
  located in memory, it joins the signature set and the feed as a dedicated event; until
  then the kickoff restart is the proxy.
- **Cup stage / competition mode** (open item, tied to Cup-mode migration): PES knows
  the current competition stage (group, knockout round, final) for its own bracket and
  extra-time/penalty logic, but neither the original nor the fork mapped where this is
  stored. Matches are currently played in Exhibition mode, which carries no stage info;
  the planned migration to Cup mode makes this data available. It likely lives near the
  team-data table or the match/stat context the pointer chain resolves to. If located,
  it joins the signature set and the feed as a dedicated event, allowing the match-type
  selector to be auto-populated instead of manual.

---

## The match feed (`libs/match_feed`)

The interface between the tracker and the rest of the suite. It is a lib crate
because tool crates never depend on tool crates; the tracker publishes, any tool can
subscribe (today: the music player; future consumers: overlays, a wiki helper).

```rust
// Shared identity, roster, and score primitives are canonically declared in their
// corresponding match_feed modules and re-exported here.
pub use crate::identity::MatchId;
pub use crate::roster::{PlayerRef, Roster, Side};
pub use crate::score::SideScore;

// MatchId is a UUID v4 in schema version 1. PlayerRef contains side,
// roster_index, stable game player_id, and display/fallback name. Roster
// contains stable team identity, display names, and ordered player entries.
// SideScore contains explicit home and away totals. These fields and their
// serialized representations are normative parts of the public contract even
// though their implementations live in separate modules.

pub enum Phase {
    FirstHalf,
    SecondHalf,
    ExtraTimeFirstHalf,
    ExtraTimeSecondHalf,
    Shootout,
    Unknown,
}

pub struct InjuryTime {
    pub minutes: u32,
    pub seconds: Option<u8>,
}

pub struct Clock {
    pub phase: Phase,
    pub display_minute: Option<u32>, // None during a shootout or when no valid clock has been reconstructed
    pub seconds: Option<u8>,
    pub injury_time: Option<InjuryTime>,
    pub observed_at_utc: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub observation_instant: Option<Instant>,
}

pub struct WireClock {
    pub phase: Phase,
    pub display_minute: Option<u32>,
    pub seconds: Option<u8>,
    pub injury_time: Option<InjuryTime>,
    pub observed_at_utc: Option<DateTime<Utc>>,
    pub observation_age_ms: Option<u64>,
}

pub struct Rosters {
    pub home: Roster,
    pub away: Roster,
}

pub struct MatchSnapshot {
    pub match_id: MatchId,
    pub lifecycle: LifecycleState,
    pub rosters: Rosters,
    pub clock: Option<Clock>,
    pub clock_motion: Option<ClockMotion>,
    pub has_reached_live: bool,
    pub score_authority: bool,
    pub score: Score,
    pub unclassified_goals: Vec<UnclassifiedGoal>,
    pub next_unclassified_goal_id: u64,
    pub scorer_counts: BTreeMap<PlayerKey, u32>,
    pub disciplinary: BTreeMap<PlayerKey, DisciplineState>,
    pub participation: BTreeMap<PlayerKey, ParticipationState>,
    pub history_complete: bool,
}

pub struct FeedMessage {
    pub match_id: MatchId,
    pub sequence: Option<u64>, // None only for subscriber-local SnapshotReplay
    pub provenance: Provenance,
    pub event: FeedEvent,
}

pub struct PublishRequest {
    pub match_id: MatchId,
    pub provenance: Provenance,
    pub event: FeedEvent,
}

pub enum Provenance {
    LiveObserved,
    LiveDerived,
    UserAction,
    RecoveredHistorical,
    SnapshotReplay,
}

pub enum FeedEvent {
    Snapshot       { state: MatchSnapshot, through_sequence: u64 },
    TeamsDetected  { home: Roster, away: Roster },
    MatchLive      { snapshot: MatchSnapshot },
    Goal           { side: Side, scorer: PlayerRef, assist: Option<PlayerRef>,
                     kind: GoalKind, clock: Clock,
                     resolves_unclassified: Option<UnclassifiedGoalId> },
    GoalAmended    { goal_sequence: u64, side: Side, scorer: PlayerRef,
                     assist: Option<PlayerRef>, kind: GoalKind, goal_clock: Clock },
    ScoreReconciled { snapshot: MatchSnapshot, reason: ScoreReconcileReason },
    OwnGoal        { conceding_side: Side, player: PlayerRef, clock: Clock,
                     resolves_unclassified: Option<UnclassifiedGoalId> }, // scoring side is the opposite
    Card           { side: Side, player: PlayerRef, card: CardKind, clock: Clock },
    SubOn          { side: Side, player_in: PlayerRef, player_out: Option<PlayerRef>, clock: Clock },
    ClockStarted   { clock: Clock },
    ClockStopped   { clock: Clock, reason: StopReason, stop_epoch: u64 },
    MatchSuspended { last_clock: Option<Clock>, reason: SuspensionReason },
    MatchResumed   { snapshot: MatchSnapshot },
    MatchFinalized { last_clock: Option<Clock>, score: Score },
    MatchAbandoned,
    FeedReleased   { reason: FeedReleaseReason },
}

pub struct ClockWatch {
    pub match_id: MatchId,
    pub revision: u64,
    pub clock: Clock,
}

pub enum ClockMotion {
    Unknown,
    Running,
    Stopped { stop_epoch: u64, reason: StopReason },
}

pub enum FeedReleaseReason {
    ProfileChanged,
}

pub enum ScoreReconcileReason {
    UnclassifiedGoalObserved,
    ClassificationBecamePermanent,
    ClassificationResolved,
}

pub enum SuspensionReason {
    SourceLoss,
    UserStopped,
}

pub enum StopReason {
    Unknown,
    Foul,
    Offside,
    Goal,
    OwnGoal,
    CornerKick,
    GoalKick,
    HalfEnd,
}

pub struct SegmentEvidence {
    pub segment_index: u8,
    pub observed_value: u32,
    pub poll_timestamp_utc: Option<DateTime<Utc>>,
}

pub enum UnclassifiedGoalState {
    Pending,
    Permanent,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct UnclassifiedGoalId(pub u64);

pub enum GoalAttribution {
    Scored { scoring_side: Side, scorer: PlayerRef },
    OwnGoal { conceding_side: Side, player: PlayerRef },
}

pub struct UnclassifiedGoal {
    pub id: UnclassifiedGoalId,
    pub attribution: GoalAttribution,
    pub original_clock: Option<Clock>,
    pub observed_segments: Vec<SegmentEvidence>,
    pub observed_order: Option<u64>,
    pub state: UnclassifiedGoalState,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct PlayerKey {
    pub side: Side,
    pub player_id: u32,
}

pub enum DismissalKind {
    SecondYellow,
    StraightRed,
}

pub struct DisciplineState {
    pub yellow_count: u8,
    pub dismissal: Option<DismissalKind>,
}

pub struct ParticipationState {
    pub roster_index: u8,
    pub has_appeared: bool,
    pub currently_on_pitch: bool,
}

pub struct Score {
    pub regulation: SideScore,
    pub shootout: Option<SideScore>,
    pub unclassified: SideScore,
    pub classification_complete: bool,
}

pub enum GoalKind { OpenPlay, Penalty, Shootout, Unknown }
pub enum CardKind { Yellow, SecondYellow, StraightRed }
```

`PlayerKey` is ordered by side and stable game player ID; display names and mutable roster
metadata are not map keys. In match-file and feed-wire JSON projections, `PlayerKey`-keyed
maps are encoded as arrays of records sorted by `(side, player_id)`, not as JSON objects.
Each record contains an explicit `player: PlayerKey` and `value`; readers reject duplicate
keys. `DisciplineState.dismissal` uses `DismissalKind`, keeping the non-dismissal
`CardKind::Yellow` out of that field. Exact substitution times remain in the event timeline
rather than being duplicated in `ParticipationState`.

`ScoreReconcileReason::UnclassifiedGoalObserved` creates a new live-observed uncertain
record. `ClassificationBecamePermanent` changes an unresolved record from `Pending` to
`Permanent` without assigning a score domain. `ClassificationResolved` is used for every
classification that must be reconciliation-only: the automation boundary has closed, no
valid monotonic anchor exists, or the resolved attribution/domain has no automation-eligible
feed event. Restoration itself publishes no reliable score-reconciliation event:
`SnapshotReplay` is sufficient to install the saved canonical state.

`next_unclassified_goal_id` starts at 0. Allocation assigns its current value, advances it
with checked arithmetic, and never reuses an ID within the `match_id`, including for records
materialized from recovery. Live-observed unclassified deltas
set `original_clock` and `observed_order`. Records materialized from recovered aggregate
counters leave either field absent when the original minute or ordering is unknowable; such
records never become eligible for one-shot automation. `Score.unclassified` always projects
each record onto its benefiting/scoring side; for `GoalAttribution::OwnGoal`, that is the
side opposite `conceding_side`.

The broker validates provenance against the semantic origin of each event; combinations
outside this table are rejected:

| Event category | Allowed provenance |
|----------------|--------------------|
| Direct live stat or table observations | `LiveObserved` |
| Reducer correlations, clock/lifecycle derivation, amendments, and reconciliation | `LiveDerived` |
| Finish, Abandon, user Stop, and profile release commands | `UserAction` |
| Recorded-minute journal notifications recovered after re-attach | `RecoveredHistorical` |
| Subscriber-local `Snapshot` initialization only | `SnapshotReplay` |

Each `FeedEvent` variant's automation eligibility and embedded snapshot payload:

| Event | Embeds MatchSnapshot in FeedEvent? | One-shot automation? | Score authority effect |
|-------|-------------------|----------------------|------------------------|
| `Snapshot` | Yes (replay) | No | Reflects current `score_authority` |
| `TeamsDetected` | No | No | No authority (setup only) |
| `MatchLive` | Yes | No | Acquires |
| `Goal` | No | Yes (if live provenance) | — |
| `GoalAmended` | No | No (presentation only) | — |
| `ScoreReconciled` | Yes | No | — |
| `OwnGoal` | No | Yes (if live provenance) | — |
| `Card` | No | Yes (if live provenance) | — |
| `SubOn` | No | Yes (if live provenance) | — |
| `ClockStarted` | No | No | — |
| `ClockStopped` | No | No | — |
| `MatchSuspended { SourceLoss }` | No | No (pauses) | Retains |
| `MatchSuspended { UserStopped }` | No | No (cancels) | Releases |
| `MatchResumed` | Yes | No | Acquires |
| `MatchFinalized` | No | No (cancels) | Releases |
| `MatchAbandoned` | No | No (cancels) | Releases |
| `FeedReleased` | No | No (cancels) | Releases |

The "Embeds MatchSnapshot" column describes the `FeedEvent` payload only. Independently,
every retained-state transition supplies the broker with its post-transition
`PublishTransition.resulting_snapshot`.

`MatchSnapshot` fields and their persistence rules:

| Field | Type | When set | Serialized in match file | Preserved in recovery |
|-------|------|----------|--------------------------|-----------------------|
| `match_id` | `MatchId` | Match creation | Yes | Yes |
| `lifecycle` | `LifecycleState` | Every transition | Yes | Usually; a saved `Live` draft is normalized to `Suspended { reason: SourceLoss }` before restore |
| `rosters` | `Rosters` | Initial `TeamsDetected` and provisional team/roster corrections | Yes | Yes |
| `clock` | `Option<Clock>` | First `Live`, updated through match | Yes (latest) | Yes (latest) |
| `clock_motion` | `Option<ClockMotion>` | First `Live`, updated through match | Yes (latest) | Usually; a saved `Live` draft is normalized to `Unknown` unless continuity of the same running/stopped window can still be proved |
| `has_reached_live` | `bool` | First `MatchLive` | Yes | Yes |
| `score_authority` | `bool` | Authority transitions | Yes | Yes |
| `score` | `Score` | Initialized at match creation; updated by Goal/OwnGoal observations, recovered aggregates, and classification/reconciliation | Yes | Yes |
| `unclassified_goals` | `Vec<UnclassifiedGoal>` | While any attributed goal lacks a score domain; stable `id` identifies later resolution and `state` distinguishes retryable from permanent | Yes | Yes |
| `next_unclassified_goal_id` | `u64` | Initialized to 0 at match creation; checked increment whenever an unclassified record is allocated | Yes | Yes |
| `scorer_counts` | `BTreeMap<PlayerKey, u32>` | Classified non-shootout Goal observations, eligible recovered aggregates, and classification/reconciliation | Yes | Yes |
| `disciplinary` | `BTreeMap<PlayerKey, DisciplineState>` | Card observations and recovered aggregate/current-state reconciliation | Yes | Yes |
| `participation` | `BTreeMap<PlayerKey, ParticipationState>` | `SubOn`, position/participation observations, and recovered current-state reconciliation | Yes | Yes |
| `history_complete` | `bool` | Initialized provisionally at match creation; confirmed or set permanently false on first stats acquisition and later reacquisition according to the late-attach/missing-order-or-time rules | Yes | Yes |

Together with the explicit authority fields, snapshot replay alone is sufficient to
distinguish paused feed authority from fully manual behavior without inferring it from
clock presence. Clock and clock motion are `None` until first `Live`; after that the
snapshot retains the most recently valid values through suspension and finalization.
`ClockMotion::Stopped.stop_epoch` remains unchanged across reacquisition only when
unchanged ticks and reducer state prove continuity of the same stoppage. Any observed or
potentially unobserved progression changes the epoch or sets motion to `Unknown`, which
cancels pending one-shot automation on reconciliation. Consumers associate pending
post-goal automation with the `stop_epoch` carried by `ClockStopped`; `MatchResumed` may
preserve that pending action only when its snapshot reports `ClockMotion::Stopped` with
the same epoch. Substitution history carrying a recorded match minute remains in the event
timeline, while the snapshot carries enough current state to reconcile without replaying
it. `SideScore` carries home and away totals.

A new subscriber receives a `Snapshot` with `SnapshotReplay` provenance only when an
active or retained match state exists; otherwise it starts in an explicit local `Idle`
state and waits for `TeamsDetected`. An existing subscriber also receives its own local
replay snapshot when `restore_retained` installs a recovery, as specified below.
`MatchAbandoned` and `FeedReleased` are published with `resulting_snapshot: None`:
subscribers registered before the broker transaction receive the event, while the same
transaction clears retained snapshot/watch state so later subscribers start `Idle`.
`FeedReleased` ends only the feed binding; it does not abandon an offline tracker draft.
For every snapshot-bearing message, `FeedMessage.match_id` must equal
`MatchSnapshot.match_id`.

The feed event table above marks which variants carry snapshots and which are
automation-eligible. Reconciliation includes idempotent setup such as loading missing team
exports from the snapshot rosters; "no one-shot automation" forbids inferred scoring,
audio, and event clips but not state initialization or missing-team lookup. A consumer
that did not receive the referenced `Goal` may insert or update its presentation-only
record from the complete `GoalAmended` payload, but must not score or trigger automation.
On attach or re-attach, the reducer first applies recovered records to canonical tracker
state, may publish `RecoveredHistorical` notifications for presentation/history, then
publishes `MatchLive` or `MatchResumed` with the authoritative resulting snapshot.
Consumers must not apply recovered notifications again to score, discipline, or
participation state.

**Provenance and sequencing.** The provenance table above defines allowed combinations.
Table-loss suspension, auto-detected finalization, and `ScoreReconciled` events are
`LiveDerived`; user stop, Finish, Abandon, and profile-change `FeedReleased` are
`UserAction`. Initial counter aggregates are carried only in snapshots and match files,
not synthesized into feed events — this prevents duplicate goalhorns after tracker
restart, table loss, or late subscription. Reliable sequences are scoped to `match_id`:
initial `TeamsDetected` for a new `match_id` receives sequence 1 and atomically replaces
the prior terminal/released match cursor; provisional team corrections retain the same
`match_id` and continue its sequence. Consumers deduplicate on `(match_id, Some(sequence))`
and always process a `None`-sequence replay snapshot, using its `through_sequence` only as
the reliable-event cursor.

- **In-process pub/sub**: the studio binary owns the feed and passes publish/subscribe
  handles to tools via constructor injection (`match_tracker::Tool::new(feed.publisher())`,
  `music_player::Tool::new(feed.subscribe())`), not through `ToolContext` — `studio_core`
  must stay minimal and cannot depend on `match_feed`. Subscribers drain a channel in
  their per-frame `tick()` — which the shell calls for **every** tool, not just the
  one whose view is on screen (see "Tool plugin interface" in the core plan) — so
  the music player reacts to goals while the streamer is looking at the tracker's
  stats table, and vice versa. `studio` constructs `MatchFeed::new(wake)` with the GUI-
  neutral callback specified in the core plan. The feed invokes it on every reliable
  publish, worker-state change, and replacement of the coalesced `ClockWatch` value;
  watch replacements remain rate-limited/coalesced, so they cannot cause an unbounded
  repaint rate. Timer-only work uses `request_repaint_after`. The core plan's registry
  installs `wake` as an `Arc<dyn Fn() + Send + Sync>` after an `egui::Context` is
  available, so `match_feed` does not depend directly on egui.
- **Snapshot replay for late subscribers**: when retained match state exists, subscription
  atomically registers both lanes and captures the latest matching `ClockWatch` together
  with `(snapshot, through_sequence)`. It places an immutable copy of the watch captured
  under the broker lock into the subscriber's initialization queue after the replay
  snapshot. Reliable events published after that capture follow it in the reliable queue,
  so tool start order does not matter, no event can fall into the subscription race, and
  replay cannot be mistaken for a newly observed live transition. On each tick, consumers
  drain initialization and reliable events before sampling the latest coalesced watch. A
  current watch must not replace the captured initialization value or be processed ahead
  of intervening reliable clock transitions. `MatchSnapshot.clock` in a replay is the
  clock reflected at its reliable cursor; the watch lane is authoritative for the current
  high-frequency clock. A watch for another `match_id` is discarded. The replay snapshot
  carries `sequence: None` and does not allocate a new reliable-event sequence;
  `through_sequence` is the last `Some(sequence)` already reflected in its state. Every
  ordinarily published reliable event carries `Some(sequence)`. The reducer produces an
  ordered batch of feed transitions plus one prospective tracker state, without committing
  either. Each transition carries its `PublishRequest` and post-transition
  `MatchSnapshot`, if any. The broker validates and publishes the entire batch under one
  synchronization boundary, assigning sequences in order and enqueuing either all messages
  or none. Subscription cannot interleave with a batch. The broker applies each transition
  to prospective retained snapshot, cursor, and watch state, validates the complete chain,
  and commits that broker state only if every transition succeeds. It performs any required
  retained-watch clearing under the same synchronization boundary used by subscription.
  No public API may advance retained state or the reliable cursor independently; the
  publisher API is conceptually:

  ```rust
  pub struct PublishTransition {
      pub request: PublishRequest,
      pub resulting_snapshot: Option<Arc<MatchSnapshot>>,
  }

  pub struct AssignedSequence {
      pub match_id: MatchId,
      pub sequence: u64,
  }

  pub struct PreparedBatch { /* broker-owned reservation; opaque to publishers */ }

  publisher.publish_batch(
      transitions: Vec<PublishTransition>,
  ) -> Result<Vec<u64>, PublishError>;

  // Durable finalization/release paths separate fallible preparation from
  // infallible commit while retaining the broker transaction reservation.
  publisher.prepare_batch(
      transitions: Vec<PublishTransition>,
  ) -> Result<PreparedBatch, PublishError>;
  prepared.assigned_sequences() -> &[AssignedSequence]; // available to durable revisions
  prepared.commit() -> Vec<u64>;              // infallible; enqueues the reserved batch
  prepared.abort();                           // consumes no sequence

  // Convenience wrapper for a one-transition batch.
  publisher.publish(
      request: PublishRequest,
      resulting_snapshot: Option<Arc<MatchSnapshot>>,
  ) -> Result<u64, PublishError>;
  ```

  `publisher.publish_batch` and its single-event `publish` wrapper reject
  `FeedEvent::Snapshot` and `Provenance::SnapshotReplay`; those values are created only by
  subscription and `restore_retained`. Every publisher-originated message receives
  `Some(sequence)`. The broker validates allowed provenance/event combinations, including
  `RecoveredHistorical` being presentation/history-only and `UserAction` being used only
  for explicit user commands.

  Finalization and feed release first prepare and reserve the complete broker batch.
  Preparation performs every fallible validation and sequence-allocation check without
  exposing messages and makes the reserved resulting sequences available for durable
  revisions. A durable revision records the reserved sequence belonging to its own
  `match_id`; a boundary-finalized old-match file never records the replacement match's
  sequence 1. The prepared reservation excludes reliable publication, subscription,
  restore, and retained/coalesced `ClockWatch` replacement until commit or abort. While the
  reservation is held, the tracker durably writes or converts the required file state. A
  file failure aborts the reservation without consuming sequences. After durability
  succeeds, broker commit is infallible and enqueues the reserved batch, after which the
  tracker commits its prospective in-memory state. No authoritative file mutation occurs
  before a potentially failing `PublishError`.

  A successful batch commits the prospective broker state — retained snapshot, reliable
  cursor, and watch — and returns the assigned sequences. Only after the broker accepts the
  complete batch does the tracker commit its own prospective reducer state. The tracker
  stores the sequence returned for a published `Goal`; any later
  `GoalAmended.goal_sequence` uses that value. On `PublishError`, the prior tracker,
  broker, and authoritative file states remain unchanged, no sequence is consumed, and no
  subscriber observes any part of the batch. Invariant or sequence-overflow errors are
  fatal internal feed errors: live reduction suspends and reports the failure rather than
  continuing from divergent state.

  For snapshot-bearing events, the embedded snapshot must equal the retained resulting
  snapshot; `MatchFinalized.score` must likewise equal the resulting snapshot's score. All
  event fields duplicated in retained state must agree with the transition snapshots:
  `TeamsDetected.home`/`away` equal `resulting_snapshot.rosters`; `ClockStarted.clock`
  equals the resulting clock and the resulting motion is `Running`; `ClockStopped.clock`,
  `reason`, and `stop_epoch` equal the resulting clock and `ClockMotion::Stopped`; and the
  lifecycle `last_clock` fields equal the resulting snapshot's latest clock. A non-`None`
  `resolves_unclassified` must identify a matching prior record with the same attribution;
  that exact record must be absent from the resulting snapshot, with the appropriate score
  and scorer projections updated exactly once. Ordinary `LiveObserved` or `LiveDerived`
  game events (`Goal`, `GoalAmended`, `OwnGoal`, `Card`, `SubOn`, `ClockStarted`, and
  `ClockStopped`) require prior lifecycle `Live`; snapshot-bearing `ScoreReconciled` follows
  its reason-specific reconciliation and finalization rules instead.
  `RecoveredHistorical` journal notifications are the exception: they may be published
  during recovery or re-attach before the resulting `MatchLive` or `MatchResumed`, remain
  presentation/history-only, and do not require prior `Live`.

  The broker also validates legal prior-state and `match_id` transitions plus lifecycle/result-
  state invariants for every reliable transition: `TeamsDetected` retains
  `TeamsDetected`; `MatchLive` and `MatchResumed` retain `Live`; `MatchResumed` is legal
  only from `Suspended` when the prior snapshot has `has_reached_live == true`;
  `MatchSuspended.reason` exactly matches `LifecycleState::Suspended.reason`;
  `MatchFinalized` retains `Finalized`; and `MatchAbandoned` or `FeedReleased` retains no
  snapshot or watch state. A suspended draft that has never reached live returns to
  provisional acquisition and uses `MatchLive` for its first valid live snapshot. No
  retained snapshot may represent no-draft or abandonment state. The broker also validates
  that `Score.unclassified` equals the benefiting/scoring-side projection of
  `unclassified_goals`, that every `UnclassifiedGoal.id` is strictly less than
  `next_unclassified_goal_id`, that IDs are unique within the match, that the cursor never
  decreases for a `match_id`, and that
  `Score.classification_complete == unclassified_goals.is_empty()`. Every key in
  `scorer_counts`, `disciplinary`, and `participation` must correspond to a player in the
  same-side snapshot roster, and each `ParticipationState.roster_index` must identify that
  same roster entry. Every `PlayerRef` must identify the same player ID at its side's
  `roster_index`. For `Goal` and `GoalAmended`, scorer and any present assist sides equal
  the event side; for `OwnGoal`, the player side equals `conceding_side`; for `Card` and
  `SubOn`, all player references equal the event side.
  `GoalAttribution::Scored.scorer.side` equals `scoring_side`, and
  `GoalAttribution::OwnGoal.player.side` equals `conceding_side`. Any mismatch returns
  `PublishError` without consuming a sequence. The broker also validates snapshot authority
  and clock invariants:
  `score_authority` implies `has_reached_live`;
  `TeamsDetected` has both fields false and
  no clock motion; `Live` has both fields true; source-loss `Suspended` has
  `score_authority == has_reached_live`; `Suspended { reason: UserStopped }` and
  `Finalized` have `score_authority == false`; and clock motion is absent exactly when no
  valid live clock has yet been observed. `MatchLive` and `MatchResumed` must embed snapshots
  satisfying the `Live` invariants. Mismatches return `PublishError` without consuming a
  sequence.

  Before calling recovery restore, an eligible saved draft whose lifecycle was `Live` is
  materialized as a recovery revision with
  `LifecycleState::Suspended { reason: SourceLoss }`. It preserves `has_reached_live`, score
  authority, score, and the latest clock; `clock_motion` becomes
  `Some(ClockMotion::Unknown)` unless continuity can still be proved. The normalized
  recovery snapshot is installed in the broker, and its next successful acquisition
  publishes `MatchResumed`. A provisional draft that has never
  reached live still uses `MatchLive` for its first valid live snapshot.

  Recovery uses
  `publisher.restore_retained(snapshot, through_sequence) -> Result<(), RestoreError>`.
  It succeeds when the broker has no retained match state or cursor. A
  same-match restore is accepted only as an exact idempotent replay with an identical
  snapshot and cursor; it may never decrease the cursor or replace newer retained state.
  The restored cursor must permit checked allocation of a subsequent sequence, so
  `u64::MAX` is rejected. A successful restore atomically installs the snapshot and cursor,
  delivers a subscriber-local `SnapshotReplay` (`sequence: None`) to every existing
  subscriber, and wakes them, but allocates no reliable sequence and triggers no
  automation. Subsequent recovered notifications and `MatchResumed` sequences start
  strictly above the restored cursor.
- **`Clock` carries time statistics**: every time-bearing match event carries phase plus
  optional consumer-ready conventional `display_minute`, seconds, injury time, and UTC
  audit timestamp fields. `display_minute` is `Some` for timed regulation and extra-time
  phases and `None` during a shootout. `Clock.seconds` and `InjuryTime.seconds` are `Some`
  only when sub-minute precision was observed or reconstructed. Recorded-minute journal
  events leave them `None`; consumers must not interpret `None` as zero. Live observations
  additionally carry an in-process `observation_instant`. The Music player computes
  deadlines from that monotonic instant;
  UTC is used only for display, files, and wire output. The process-local
  `observation_instant` is never persisted or serialized. JSON-lines output serializes a
  wire projection, not the in-process structures directly. Conversion replaces
  `observation_instant` with `observation_age_ms` at line-emission time; the field is `None`
  exactly when no process-local anchor exists. Non-time-bearing lifecycle messages such as
  initial roster detection carry neither timestamp. The reducer may retain a zero-based
  minute internally, exposed only if useful under an explicit name such as
  `raw_minute_index`. Consumers assign the optional `display_minute` directly and never
  perform the zero-to-one-based conversion.
  Injury time is represented separately from the regulation phase minute; shootouts use a
  distinct phase and have no fabricated regulation minute. Recovered aggregates have no
  wall-clock timestamp; recovered historical records carry their recorded game minute
  but use no wall-clock timestamp unless their source actually supplies one. `PlayerRef`
  combines the stable game player ID with side and roster index, using the name only as
  display/fallback data, so references survive snapshot replay. This feeds the music
  player's `time` conditions without prompting.
- **Two transport lanes**: ordinarily published reliable lifecycle and game events use
  `FeedMessage.sequence = Some(...)` and an ordered, unbounded queue per live subscriber;
  this lane never drops events, and dropping a subscriber releases its queue immediately.
  A disconnected/recreated subscriber initializes from a snapshot and never replays
  missed one-shot automation. Clock updates use the separate coalesced
  `ClockWatch { match_id, revision, clock }` channel; its revision is monotonic within one
  `match_id`, resets for a new match, and does not consume reliable-event sequence
  numbers. Consumers accept a watch value only when its `match_id` matches their active
  snapshot. The broker clears the retained watch in the same reliable transition
  transaction as `MatchAbandoned`, `FeedReleased`, or a new match's `TeamsDetected`;
  suspension retains the last value but publishes no advancing revisions. The bounded,
  lag-dropping watch lane
  prevents high-frequency ticks from accumulating in a subscriber's queue.
- **No built-in pipe/socket server**: the old named pipe's only consumer is gone.
  `studio match-tracker watch` runs the tracker headlessly in that CLI process and emits
  its own JSON-lines stream; it does not attach to or expose a concurrently running GUI
  instance. Each line is a separately versioned feed-wire envelope
  (`{"feed_schema": {"major": 1, "minor": 0}, "message": ...}`). Feed schema 1.0
  normatively defines `WireEnvelope`, a tagged `WireMessage::{Reliable, ClockWatch}`, and
  wire projections of `FeedMessage`, `FeedEvent`, `MatchSnapshot`, and `UnclassifiedGoal`.
  Every recursively embedded `Clock` becomes `WireClock`; no in-process `Instant` or
  struct-keyed JSON map appears on the wire. The serde tag/content names and field casing
  are part of the major-version contract. External consumers therefore receive both
  transport lanes without conflating their sequence spaces. Major versions are compatibility
  boundaries. Minor versions may add only optional/defaulted fields whose omission does not
  change existing semantics.
  Adding or changing a reliable message/event variant is a major-version change unless
  the wire format defines an explicit forward-compatible opaque-event mechanism;
  consumers must not silently skip unknown state-changing reliable events. This covers
  external integration without running a server inside the GUI app.

What the music player does with the feed — auto team loading, feed-driven scoring and
goalhorns, automatic goal minutes, the revival of the `.4ccm` `event` clips, and
the manual-override rules — is specified in the
[Music player plan's](music_player.md) "Autopilot" section.

---

## GUI view

```
┌─────────────────────────────────────────────────────────────┐
│ [Current Match] [aaa vs bbb 14-02]                          │  ← match tabs
│─────────────────────────────────────────────────────────────│
│ Tracking: reading stats                            [Start/Stop] │  ← run strip
│           /aaa/  2 - 1  /bbb/            67:24 ⏸               │  ← score + clock
│ ┌────────────────────────────────┬────────────────────────────┐ │
│ │ Player       Goals Assists ... │ 23' GOAL: Player One       │ │
│ │ Player One     1      0        │       assist: P. Two       │ │
│ │ Player Two     0      1  [ heatmap ] │ 41' YELLOW: ...      │ │
│ │ ...                            │ 67' IN: Sub Guy            │ │
│ └────────────────────────────────┴────────────────────────────┘ │
│ [Columns…] [□ Benched] [□ QR] [Finish match] [Discard draft] │
│                                      [Open] [Save] [Export…] │
└─────────────────────────────────────────────────────────────┘
```

- **Stats table**: rows = players (home/away tints; benched players hidden unless
  toggled), columns = the stat catalog with a column picker (show/hide + order,
  persisted). Changed cells flash and fade back — the original's red-fade effect,
  done as a time-based color lerp in the update loop. Ctrl+C copies the selection as
  TSV. A **heatmap** button on each player row opens the heatmap viewer (available
  for live in-memory data and saved match files that carry heatmap grids).
- **Event log**: side-colored lines stamped with the reconstructed minute.
- **Score display**: unclassified totals are shown separately with a warning, for example
  `2–1 (+1 unclassified)`; they are never folded into regulation or shootout score.
- **Match tabs**: at most one feed-bound **Current Match**, plus offline drafts, finalized
  matches, and opened files; a finished match stays open as a tab. On a validated session
  boundary after `Live`, the old Current Match tab becomes a finished tab and a new Current
  Match tab is inserted only after the broker batch is prepared/reserved, the staged
  finalized revision is durable, and the infallible atomic
  `MatchFinalized`/`TeamsDetected` commit completes. If the durability write fails, the old
  tab remains the authoritative Current Match with a finalization-blocked status and the
  detected replacement-team state is retained for retry; new-session deltas are not applied
  to the old match. A team change while still provisional updates or replaces the existing
  Current Match draft without creating a finished empty tab. On
  a profile-change release, the old Current Match becomes an **Offline draft** tab labeled
  with its original game profile and receives no further live updates. After tracking
  restarts, the next successful team detection creates the new Current Match tab. Save,
  Finish, and Discard act on the selected tab, but
  Finish or Discard of an offline draft never publishes to `match_feed`.
- **Lifecycle controls**: **Finish match** finalizes the selected live, suspended, or
  offline draft after confirmation, but is unavailable until that draft has reached
  `Live`. **Discard draft** abandons the selected draft and is available from
  `TeamsDetected`, `Live`, and `Suspended`, including offline drafts. For a feed-bound
  draft it publishes `MatchAbandoned`; for an offline draft it removes only the local
  draft. **Stop tracking** only suspends acquisition; it does not finalize or discard
  match data.
- **QR sync** (opt-in): a small QR code of the current wall-clock milliseconds,
  regenerated at a capped cadence of approximately 30 Hz while enabled — the original's
  stream-latency measurement aid, kept via the `qrcode` crate.
- **Export…**: a small menu offering the wiki footballbox export (copies
  `{{Football box}}` wikitext to the clipboard or saves as `.wiki.txt`). It is enabled
  only when the match is finalized and `score.classification_complete` is true. If
  `history_complete` is false or required minutes are missing, the exporter presents a
  validation summary and either omits incomplete
  scorer/card details or requires explicit user confirmation or editing; it never
  invents minutes.

---

## Match files

The binary `.sen` format is dropped entirely — its version-locked layout and
dual-direction template reader were most of the old file-handling code, and existing
archives remain readable in old SEN:P-AI. Matches save as **plain JSON**
(`.match.json`):

- A separately versioned match-file envelope:
  `"match_schema": {"major": 1, "minor": 0}`, plus `producer_version`,
  `game_version`, `game_profile_id`, `match_id`, lifecycle state, and
  `history_complete`. In a file whose match has not reached `Live`, a true
  `history_complete` value is provisional ("not yet disproven"), not confirmed complete;
  readers and exporters interpret it together with `has_reached_live`. Feed-wire and
  match-file schemas use separate version spaces.
  Major versions are compatibility boundaries. Minor versions may add only optional or
  defaulted fields whose omission does not change existing semantics; adding or changing
  a state-affecting enum variant is a major-version change unless an explicit forward-
  compatible opaque representation is defined. Readers must not silently skip unknown
  state-changing variants.
- Match info: team names and IDs, full rosters, and the current `Score`, which is frozen
  when lifecycle state is `Finalized` but is not necessarily fully classified. The
  regulation and shootout components are authoritative for goals already assigned to
  those domains. When `classification_complete` is false, the overall result is incomplete
  because additional goals remain unclassified; the score must not be used as a final
  result or canonical footballbox score until classification is complete. The file also
  carries detected match length and optional provenance-tagged timestamps:
  `created_at`, `first_observed_at`, optional `kickoff_at` when kickoff was actually
  observed, and optional `finalized_at`. Attaching mid-match must not label
  `first_observed_at` as kickoff. Match state also stores `has_reached_live`, the latest
  optional `Clock`, optional `ClockMotion` including `stop_epoch`, and the saved
  `score_authority`. Recovery files additionally store the feed-binding state needed to
  determine whether the saved `MatchSnapshot` is eligible for `restore_retained`.
  Ordinary read-only Open treats recorded authority as historical data and never installs
  feed authority.
- The event list and the clock-timeline entries they reference, plus a distinct
  `recovered_aggregate` section for initial counters whose order and minutes are unknown.
  Canonical `unclassified_goals` records preserve every unresolved goal's stable match-
  scoped ID, scored/own-goal attribution, optional original clock, observed segment
  evidence, optional observed ordering, and `Pending`/`Permanent` state across save/restore.
  The persisted `next_unclassified_goal_id` cursor prevents reuse after resolved records
  leave this collection. A classified goal is stored once as its
  canonical event with the final known assist. Feed-level
  `GoalAmended` messages are not separate scoring events in this timeline; an optional
  audit log may retain an amendment and its `goal_sequence` reference. Exporters always
  materialize one goal.
- Latest stats and the full per-player stat-change history; the latest stats are final
  only for a finalized match. Stat records use stable catalog IDs plus display names,
  value type, and segment count; names alone are not schema keys. `catalog_id` is unique
  within a verified 4cc Studio `game_profile_id`. Fork array positions and `visualIndex`
  values are retained only as optional provenance metadata and are never used directly as
  schema keys. Unidentified scalar fields retain their stable catalog IDs and are stored
  with field class `UnknownScalar` — their history is the raw material for identifying
  them, one of the reasons `.sen` files were kept at all. Stat records preserve the profile
  field class separately from their binary value type and segment count. Grid/blob fields
  follow the separate latest-grid rules below instead of entering generic stat history.
- **Candidate heatmap grids**: the grid layout claims and their unverified status are in
  "Game structures". Once a grid signature and its semantics are verified, its
  latest per-player/team snapshot is updated in memory at a throttled cadence and written
  on save. Profiles mark whether a grid is cumulative or instantaneous; the viewer labels
  and interprets it accordingly. A time series is not stored unless a later
  schema adds an explicitly downsampled optional timeline; grid changes are not duplicated
  into generic scalar stat history. Grids remain a separate optional section so match
  files from versions without a verified heatmap signature stay valid; the PES17
  verification task below determines whether that grid can be enabled.

Autosave carries over: on finalization and/or on a timer, with `%HOME`/`%AWAY` and
strftime placeholders in the filename pattern. Saves use write-temp → flush → atomic
replace. A suspension creates at most one separate recovery file per suspension episode
and never overwrites a user save. A timed interval resets only after a successful write;
filename collisions receive a deterministic suffix.

On startup, the tracker scans the autosave directory for unclosed recovery files and
offers **Restore**, **Open read-only**, or **Discard**. An `offline_only` draft is instead
offered **Open offline**, **Finalize offline** (available only if the draft reached
`Live`), or **Discard**, but never feed **Restore**;
it cannot call `restore_retained` or publish later events under the released `match_id`.
Its saved reliable cursor is audit metadata only. Recovery files additionally carry
`reliable_through_sequence`, the last reliable feed sequence reflected in the saved
canonical state; eligible feed restoration uses it to continue sequence allocation
strictly above the saved cursor. Restore preserves the saved `match_id`,
`has_reached_live`, `score_authority`, latest clock, `history_complete` flag, feed-binding
state, `unclassified_goals` records with their states, and
`next_unclassified_goal_id`. It normally preserves lifecycle and clock motion as well,
except that a saved `Live` draft is first materialized as a
recovery revision with `LifecycleState::Suspended { reason: SourceLoss }` and
`clock_motion = Some(ClockMotion::Unknown)` unless continuity can still be proved.
`restore_retained` validates
and installs that exact normalized `MatchSnapshot` with the saved cursor through the atomic
operation described above. It does not resume automation until live memory is reacquired
and a `MatchResumed` snapshot has reconciled the restored state; a provisional draft that
has never reached live uses `MatchLive` instead.

Finalization is staged for every path — explicit Finish or a verified session boundary. A
feed-bound finalization builds its prospective `Finalized` revision and takes the
complete broker batch (including replacement `TeamsDetected` for a session boundary)
through the staged prepare/reserve → durable write → infallible commit pattern defined
under "The match feed". During the durable-write step, if no current user save or recovery file
exists, the tracker atomically
creates a finalized safety/recovery file even when **Autosave on match end** is off.
An offline draft instead commits its finalized state locally without
feed publication. Finalization never removes the last durable copy: only after the
finalized write or conversion succeeds may an active recovery marker be cleared or
archived. If an explicit
Finish write fails, the prior draft remains open in memory, existing recovery markers
remain intact, and the UI reports that durability was not established so the user can
retry or use Save As. If a boundary-triggered write fails, the prior lifecycle remains
authoritative, the UI marks the tab as finalization-blocked, acquisition does not apply the
new session to the old match, and the detected replacement-team state is retained for
retry. It does not publish `MatchFinalized`, discard the prior recovery marker, or create a
new feed-bound draft until durability succeeds. Abandonment may delete the recovery copy
only after explicit confirmation.

**Wiki footballbox export.** A match can be exported as canonical `{{Football box}}`
wikitext only when it is `Finalized` and `score.classification_complete` is true. If
unclassified goals remain, canonical export stays unavailable in the first implementation;
confirmation alone never silently assigns them to regulation score. A future tracker-side
classification/correction workflow may resolve them. The new, tested exporter is informed
by `WikiFootballbox.h`'s generic parameter serializer and the fork's partial match adapter;
the wikitext field structure is a factual specification, not code to adapt. The original never
implemented
its declared match adapter; the fork's adapter handles teams, score, goals, and own goals
but omits cards, date, and time. The suite's exporter pulls scorers with minutes (incl.
injury time and penalty/own-goal markers), cards (single yellow, or `{{sent off}}` for two
yellows / a straight red), and score/teams from the match file. Date/time uses a verified
`kickoff_at` when available; otherwise the exporter requires user confirmation or omits
those fields. If `history_complete` is false or event minutes are missing, it presents a
validation summary and omits incomplete details unless the user explicitly supplies or
confirms them; it never invents minutes. No extra memory reading is involved. This
completes the feature the original declared but never wired up.

---

## Settings (tool section)

| Setting | Default | Notes |
|---------|---------|-------|
| Poll rate | 100ms | acquisition + diff cadence; bounded to a safe supported range |
| Autosave on match end | off | optional named autosave on finalization, not raw table loss; every finalization path first guarantees a durable safety/recovery file |
| Timed autosave + interval | off / 10s | interval is bounded; resets after a successful write |
| Autosave directory | platform documents directory / `4cc Studio/Matches` | user-selectable |
| Autosave filename pattern | `%HOME vs %AWAY %Y-%m-%d.match.json` | deterministic suffix on collision |
| Show benched by default | off | |
| QR sync widget | off | |

## CLI

```
studio match-tracker watch
    # independently attach and print versioned reliable-event/ClockWatch JSON lines
studio match-tracker dump-fixture <out> --duration <seconds>
    # debug/developer builds only: record RawTableSnapshot fixtures for replay tests
```

`watch` doubles as the external integration point: anything that wants live match
events (overlay scripts, third-party tools) runs this headless tracker session and pipes
its versioned JSON envelope (wire contract under "The match feed") instead of speaking a
bespoke protocol. `dump-fixture` records only the profile ID, monotonic
sample timing, and target-table bytes needed for `RawTableSnapshot` fixtures; it excludes
unrelated process memory.

---

## Platform notes

- **Live tracking is Windows-only**: process discovery and memory reading use Win32
  APIs. On other platforms (and in the WASM build, where process access can never
  exist) Start tracking is disabled with an explanatory tooltip; opening and viewing
  saved match files, heatmap viewing, and footballbox export remain available.
- PES running under Wine/Proton exposes its memory via `/proc/<pid>/mem`; a Linux
  backend behind the same memory-source trait is possible future work, not planned.
- Reading another process's memory can require elevation depending on how the game
  was launched. Access-denied errors use `libs/elevation` for diagnosis and optional
  elevated relaunch on Windows, and are reported distinctly from "process not found."
  The tracker otherwise follows the same operational constraint as the original.

---

## Improvements over SEN:P-AI (summary)

1. **Match logic decoupled from GUI and Win32** — pure model over
   `DecodedGameSnapshot` input, testable headless against recorded `RawTableSnapshot`
   fixtures.
2. **The feed replaces the pipe** — typed in-process events with snapshot replay, and
   the main consumer ships in the same binary: the Rigdio integration lost in 2020
   returns, stronger (autopilot instead of event clips only).
3. **Readable match records** (JSON) instead of a bespoke binary format.
4. **Per-version signature data** instead of hardcoded PES17 constants.
5. **Wiki footballbox export** — a new, tested match exporter informed by the old
   formatting primitives and the fork's partial adapter produces `{{Football box}}`
   wikitext directly from match files.
6. **Heatmaps** — if the PES17 grid is verified to populate with stable semantics,
   per-player position grids that the old tool processed only as a hidden summed scalar
   gain an independent GUI viewer implemented from verified 10x14 grid semantics;
   otherwise the PES17 viewer remains hidden.
7. **Suite integration**: settings, theme, logging, single binary — no separate Qt
   runtime.

## Dropped features

- **Named pipe server and the JSON event log file**: no surviving consumers; the
  `watch` CLI covers scripting and the match file covers the record.
- **`.sen` files**: dropped in both directions; old archives stay readable in old
  SEN:P-AI.

## Testing

- **Recorded-table replay**: a debug command dumps `RawTableSnapshot` fixtures from live
  games; the memory-source trait decodes them to `DecodedGameSnapshot` input through the
  same path as live reads. Event derivation, clock reconstruction, and phase
  transitions are asserted on recorded real matches (including extra time and
  penalty matches).
- **Acquisition validation fixtures**: a team-table candidate with 22 valid entries on only
  one side is rejected; 11 side-correct entries on each side are accepted; wrong-side
  validity markers, implausible team names, and player IDs inconsistent with the side's
  team ID are rejected.
- **Clock calibration unit tests**: synthetic tick sequences with known match lengths,
  stoppages, and injury time; stop/start debounce at the minimum and maximum supported
  poll rates, satisfying both the two-sample and profile elapsed-time thresholds.
- **Feed contract tests**: late-subscriber `SnapshotReplay` initialization without
  one-shot automation; an existing subscriber receiving
  `ScoreReconciled { reason: UnclassifiedGoalObserved }` and updating its uncertain score
  without automation; subscription immediately after that event receiving the updated
  unclassified score and record in its replay snapshot without triggering automation; a
  recovered aggregate unknown-domain goal appearing only in the next `MatchLive` or
  `MatchResumed` snapshot without a standalone `UnclassifiedGoalObserved` event; an
  automation-eligible resolving `Goal` or `OwnGoal` carrying the exact
  `UnclassifiedGoalId`, moving that record from uncertain to classified score without
  double-counting, while a `None` resolution ID adds a newly classified observation;
  `sequence: None` snapshots always processed
  and their cursor honored; rejection of
  publisher-originated `Snapshot`/`SnapshotReplay` and invalid
  provenance/event combinations; rejection of inconsistent `has_reached_live`,
  `score_authority`, lifecycle, and clock-motion combinations; rejection of snapshots
  whose unclassified score projection or `classification_complete` value disagrees with
  `unclassified_goals`; rejection of duplicate/out-of-cursor `UnclassifiedGoalId` values or
  a decreasing `next_unclassified_goal_id`; rejection of `PlayerKey` entries absent from the
matching same-side
  roster and participation entries whose `roster_index` identifies a different player;
  rejection of event or attribution `PlayerRef` values whose side, ID, or roster index
  disagrees with the enclosing side or snapshot roster; atomic snapshot cursor, immutable captured
`ClockWatch`, and both-
  lane registration under concurrent
  publish/watch replacement; initialization/reliable draining before the latest coalesced
  watch; atomic batch validation and sequence assignment, ordered transition snapshots,
  subscription exclusion during the batch, all-or-none enqueue, cursor advance, optional
  retained-snapshot install/clear, and required watch clearing; prepared-batch validation
  and match-ID-associated sequence reservation before file mutation, exclusion of publish,
  subscription, restore, and watch replacement while reserved, old-match cursor selection in
  a cross-match boundary batch, abort without sequence consumption on file failure, and
  infallible commit after durability; single-event wrapper parity;
  new-match sequence reset to 1, provisional same-match
  continuation, and rejection of illegal prior-state or `match_id` transitions; rejection
  of mismatched embedded and retained snapshots/scores; abandonment and `FeedReleased`
  observed by existing subscribers while later subscribers start `Idle`; successful
  recovery restore plus cursor continuation above the restored value, including a saved
  `Live` snapshot normalized to source-loss `Suspended` before a legal `MatchResumed`;
  reliable events
  never dropped; `TeamsDetected` performing setup without score authority, `MatchLive`
  acquiring it, and mid-match reacquisition seeding the score; source-loss suspension
  retaining consumer state and eligible pending metadata; user-stop canceling pending
  automation; user-stop, abandonment, finalization, and feed release returning to manual
  score accounting; `MatchResumed` reconciliation accepted from a suspended snapshot that
  has reached live and rejected from one that has not, with the latter using `MatchLive`
  for its first valid live snapshot; recovered historical events not triggering live
  automation; a recovered substitution appearing
  once in history and never being applied twice to participation state; `match_id`/
  reliable-sequence deduplication after reacquisition; the sequence returned by a successful
  `Goal` publish being used by its later `GoalAmended`; assist-only and kind-only
  `GoalAmended` payloads retaining the original goal clock and complete optional-assist/kind
  state while updating history without scoring or automation; a subscriber attaching between the
  goal and amendment using the complete amendment payload without triggering automation;
  `ScoreReconciled { reason: ClassificationResolved }` updating state without one-shot
  automation for post-boundary live records, recovered/restored records, and resolved
  attribution/domain combinations with no automation-eligible feed event; a late
  subscriber receiving scorer, disciplinary, participation, and team-loading state; a provisional
  correction or new match replacing slots loaded automatically for an earlier feed team
  while retaining a manual load and reporting its mismatch; Undo, per-team Reset, and team
  load/reload leaving authoritative score, scorer, discipline, and participation state
  unchanged; late subscription during source-loss suspension retaining paused feed authority
  and during user-stop suspension restoring manual score accounting from snapshot state
  alone; Autopilot `Off` suppressing playback/highlights without restoring manual scoring;
  `ClockStopped`
  propagating its `stop_epoch` to an existing subscriber and preserving pending automation
  through suspension/resume only when `MatchResumed` reports that same stopped epoch; a
  restart/stop cycle hidden inside a source-loss interval changing the stop epoch or
  reconciling clock motion to `Unknown` and canceling the pending horn; announcement
  deadlines using the original monotonic observation anchor; `WireClock` conversion
  including `observation_age_ms` exactly when that anchor exists and never serializing
  `observation_instant`; a disk-restored
  `UnclassifiedGoal` in `Pending` state with no monotonic anchor using reconciliation only
  and never triggering automation; batch preparation failure leaving reducer, broker, and
  authoritative file state unchanged, consuming no sequence, and exposing no part of the
  batch; lifecycle/result-state mismatches
  being rejected atomically; stale same-match restore rejection, exact idempotent same-match
  restore, `u64::MAX` cursor rejection, and concurrent restore/subscription; finalized
  matches releasing score authority without stopping audio or triggering victory
  automation; `FeedReleased` releasing authority and clearing broker state without
  abandoning the offline draft; `ClockWatch` initial replay, wakeups, coalescing, match-ID
  rejection, clearing, and independent revisions under load.
- **Match lifecycle tests**: same-team rematch detection through both the readiness chain
  and the scan-only table-loss/reallocation fallback; a validated different-team boundary
  after `Live` preparing/reserving the broker batch, durably writing the prospective final
  revision, infallibly publishing the old `MatchFinalized`, clearing the old watch in the
  new-match transition, and publishing
  `TeamsDetected` with a new `match_id`; a boundary write failure publishing neither
  `MatchFinalized` nor replacement `TeamsDetected`, retaining the prior lifecycle and
  recovery marker plus the detected replacement-team state, then a successful retry
  publishing the ordered atomic boundary batch; source progression during the blocked
  interval being excluded from the old match and causing the new match's recovered state
  and `history_complete` to reflect anything missed before retry; a team-identity change
  while provisional replacing the existing `TeamsDetected` draft without finalizing an
  empty match; a provisional `TeamsDetected` draft disabling Finish; discarding that feed-
  bound draft publishing `MatchAbandoned`, clearing retained snapshot/watch state, and
  suppressing immediate recreation from the same source session; transient read failure
  vs. finalized match; Finish and Discard while the
  source session remains live (the terminal `match_id` receives no more deltas and the same
  session cannot create another draft);
  PES-version change publishing `FeedReleased` and detaching an incompatible suspended
  draft, retaining its Offline draft tab while a new feed-bound Current Match becomes
  active, followed by offline save/finalize/discard without further feed publication;
  unknown score-domain goals remaining visible in `Score.unclassified` through snapshot
  replay without triggering automation, then moving atomically by stable record ID when
  classified; normal and own-goal attributions projecting onto the benefiting/scoring side,
  with an unknown-domain own goal preserving its conceding player and emitting `OwnGoal`
  only after regulation-domain classification; multiple `unclassified_goals` records
  retaining distinct IDs, attribution, scorer/clock evidence, and state through save/restore;
  classification before the post-goal boundary publishing an ordinary
  `Goal`, versus classification after a closing boundary while the record remains
  `Pending` publishing only `ScoreReconciled`; one pending record classified into
  regulation and another into shootout with only the regulation classification updating
  ordinary `scorer_counts`; an early `Pending` → `Permanent` transition publishing
  `ScoreReconciled { reason: ClassificationBecamePermanent }`; unresolved `Pending`
  records becoming `Permanent` through a reconciliation in the same atomic batch
  immediately before `MatchFinalized`; legitimate gauge/state decreases and
  `UnknownScalar` changes being accepted while only verified monotonic-counter decreases
  invoke invalid-read/reset handling; two goals or goal+assist/goal-kind evidence spanning
  adjacent polls; late assist or kind evidence amending before kickoff and being rejected
  after a phase or other amendment-closing boundary; closure of an initial goal holdback
  flushing every confirmed known-domain goal or materializing an unknown-domain record
  without discarding its delta; yellow-first
  and red-first second-yellow
  correlation, timeout, and phase/suspension boundary closure, all retaining the first
  delta's clock and never dropping a confirmed card; penalty and shootout score separation;
  clock stop/start debounce.
- **File format tests**: malformed, future-version, and oversized match files; canonical
  amended-goal materialization; profile field-class, `has_reached_live`, latest clock,
  optional timed `display_minute` versus shootout `None`, observed/reconstructed optional
  `Clock.seconds` and `InjuryTime.seconds` versus recorded-minute `None`, non-serialized
  `observation_instant`, and `ClockMotion`/`stop_epoch` round-trips; `score_authority`
  round-trips in ordinary and
  recovery files; ordinary read-only Open treating recorded authority as historical data
  without installing feed authority; recovery-only feed-binding-state round-trips;
  `PlayerKey`-keyed scorer, discipline, and participation maps round-tripping through
  ordinary and recovery files; multiple `unclassified_goals` records with stable IDs,
  scored/own-goal attribution, `Pending`/`Permanent` state, unclassified totals,
  `classification_complete`, and a non-reusing `next_unclassified_goal_id` cursor surviving
  save/restore; a restored
  `UnclassifiedGoal` in `Pending` state lacking a monotonic anchor and remaining ineligible
  for one-shot automation regardless of UTC or apparent stopped-window state;
  recovery-file discovery, cursor-preserving restore, and marker cleanup; prepared broker
  reservation followed by atomic conversion of an active recovery marker to `offline_only`
  and infallible profile-change `FeedReleased` commit, including preparation/conversion
  failure, crash/interruption cases, and rejection of feed Restore for the released
  `match_id`; Finish with no prior timed save, suspension,
  recovery file, or user save creating a durable finalized safety copy; write interruption
  retaining the draft and existing marker; atomic autosave interruption; optional 140-cell
  heatmap round-trip and absence, dimension/size rejection, and confirmation that heatmap
  changes do not create a hidden generic-stat grid timeline; worst-case JSON size and
  autosave latency for a full 120-minute match.
- **Cross-platform compilation**: Windows native plus non-Windows/WASM file-viewer
  build (Start tracking disabled, file viewing enabled).
- **Manual**: a full simulated stream with the music player on autopilot (shared with
  the music player's test plan).

### Verifying source-derived findings

The suite's first implementation targets PES 2017, so verification is prioritized
accordingly. Unresolved PES17 claims from either source are verified for the first release;
PES21 claims from the unofficial fork are **deferred** until PES21 support is pursued and
recorded here as a starting point for that future work. A claim that fails verification is
dropped, and the affected plan text reverts.

**PES17 verification required before first release:**

Each task must reach a recorded pass/fail decision before release. Failure disables the
affected optional capability or fast path but does not block the scan-based tracker itself.
Phase verification is a release gate only if first-release ET/shootout classification and
automation remain promised.

- **PES17 phase buckets**: the original confirms buckets 0–2 as the first half and 3–5
  as the second half, but only hypothesizes `1st ET: [6-7?], 2nd ET: [7?-8?]` and leaves
  segment 8 as a TODO (`MatchReader.cpp:171`). Capture and label all nine segments across
  regulation, both extra-time periods, and a shootout. Confirm the 6/7/8 mapping and
  transition behavior before enabling shootout classification or ET/penalty-phase
  automation; until then those buckets remain unregistered and produce `Phase::Unknown`.
- **PES17 heatmap grid**: the original maps the "Play Area Grid" at entry+0x1034
  (`StatsInfo.cpp:150`, 140×u32) and processes it only through the generic stat path as a
  summed scalar column, hidden by default; it never interprets or renders the 10x14 grid as
  a heatmap. Confirm that the underlying 140 cells — not merely their generic aggregate —
  populate with stable spatial semantics during a live PES17 match (values update as
  players move), determine 10×14 orientation and whether sides/halves require flipping,
  and whether values are cumulative or instantaneous. If it is confirmed to populate with
  stable semantics, PES17 heatmap recording and the viewer ship in the first release;
  otherwise the viewer remains hidden for PES17 and the signature is not registered.
- **PES17 pointer-chain locator (fast path)**: the fork's profile defines two chains
  from the same root: `root = *(moduleBase + 0x21A9E68)`. The **readiness chain** is
  `container = *(root + 0x124)`, `flag = *(u32 *)(container + 0x1168)`; `flag == 0` claims
  that a match is live. The **stats-base chain** is `subObj = *(root + 0x2890)`,
  `stats-table base = subObj + 0x317FC`; the last step is arithmetic, not a dereference.
  The operational fork does not use this profile in its PES17 `findStats()` path.
  Confirm that applying the profile's `+0x3804` player-array offset to the resolved base
  yields the same home stat-entry address found by the original scan, that the derived
  away address also agrees, and that the readiness flag distinguishes live/not-live. If
  it holds, PES17 gains the ASLR-proof fast path described under "Acquisition and
  polling"; if not, the scan-only path stands.

**Low priority (PES21, deferred until PES21 support is pursued):**

- **Pointer-chain stats-table locator (PES21)**: same two-chain structure. The readiness
  chain is `root = *(moduleBase + 0x36F4178)`, `container = *(root + 0)`,
  `flag = *(u32 *)(container + 0)`; `flag == 0` claims that a match is live. The stats-base
  chain is `subObj = *(root + 0xBD08)`, `stats-table base = subObj + 0x750D0`. Confirm both
  on a live PES 2021 (1.01).
- **PES21 stat layout**: confirm entry size 8500 and indicator validation (0xFFFD/0xFFFE
  at +0, ID at +4). Normalize the fork's declared 269-slot catalog into unique Studio
  `catalog_id` values: account for all 263 explicit initializers and six default slots,
  bounds-check every offset/type/count against the entry size, and explicitly resolve or
  reject duplicate IDs, overlapping aliases, and conflicting definitions — including the
  duplicate `visualIndex` ranges 110–134 and 176–185 — before enabling them. Confirm the
  physical 40-slot layout and capture a roster using slots 32–39. The
  suite must use dynamically sized vectors or the profile maximum of 40 throughout; the
  fork's 32-slot `teamInfo` structures are not reusable. Cross-check a sample of named
  stats (Goals, Cards, Saves, Minutes, Rating) against observed match events.
- **Substitution journal (unverified)**: verify the fork's claimed `u8` count at
  `table_base + 0x10B4D4` (maximum 12) and 40-byte record `i` at
  `table_base + 0x10B2F4 + 40*i`, with `type` at `+0`, `team` at `+4`, `period` at `+8`,
  `minute_in_period` at `+12`, `inIdx` at `+20`, and `outIdx` at `+24`. Confirm that the
  indices match the actual substitution, the recorded period/minute is exact, and the
  restart-replay behavior holds across a tracker restart mid-match.
- **Direct clock read (unverified)**: verify the fork's claimed chain
  `manager = *(u64 *)(moduleBase + 0x3705E10)`,
  `clock_object = *(u64 *)(manager + 0x50)`, the `u32` segment at
  `clock_object + 0x3FCD4`, and the `f32` ticks at `clock_object + 0x3FCD8`. Confirm that
  segment/ticks reproduce the scoreboard minute + injury time across all phases and into
  extra time. Also verify tick stagnation and resumption across ordinary stoppages, goal
  celebrations/replays, halftime, and menus, and confirm that a unique post-goal restart
  can be derived before enabling PES21 autopilot.
- **PES21 heatmaps**: confirm the 140×u32 player grid at entry+0x1938 and the 8 per-team
  grids populate during a match. Determine the meaning of all 8 grids, validate the
  fork's five-segment accumulation and odd-segment flipping, and establish whether values
  are cumulative before exposing or serializing them.

## Key decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Naming | `match_tracker` crate; label "Match tracker" — no "(SEN:P-AI)" suffix | Suite naming convention; unlike still-active Rigdio/RigDJ, the old name has been out of use for years and carries no recognition value |
| Music player interface | In-process `libs/match_feed`: typed pub/sub + snapshot replay | Both tools live in one binary, so the in-process boundary needs no socket or serialization; the independent `watch` CLI serializes the same public model separately |
| External consumers | No GUI server; `watch` runs an independent headless tracker and prints versioned JSON lines | The old pipe's only consumer is gone; a CLI stream covers scripts without inter-process GUI plumbing |
| Match files | New versioned JSON format only; `.sen` dropped | Readable, diffable, schema-evolvable; old archives keep old SEN:P-AI |
| Autonomy | Full, Co-pilot, Assist (highlight-only), and Off playback modes; modes do not transfer feed score authority, and manual playback override stays live | Specified in the Music player plan; tracker stop, feed release, abandonment, or finalization restores manual score accounting |
| Deep scan | Automatic throttled escalation; no "Search Harder" button | A plain chunked scan on the polling thread is cheap enough that the manual gate the original needed just becomes streamer homework |
| Game versions | Register verified signature sets only; PES17 first, PES21 deferred | PES17's original scan signatures are known, but buckets 6–8 of its phase map, the fork-defined pointer chain, and the heatmap grid must be verified before enabling the affected phase automation or additions; all PES21 findings remain inactive reference data until their deferred verification |
| Platform | Windows-only backend behind a memory-source trait | Win32 live tracking now; a Wine `/proc` backend stays possible; live memory access is impossible in WASM, while saved-match viewing remains supported |
