# 4cc Studio — Match tracker plan

The match tracker (`crates/tools/match_tracker`, tool id `match-tracker`, sidebar
label "Match tracker" — unlike Rigdio/RigDJ the old name has been out of use for years
and does not stay in the label) is the successor to **SEN:P-AI**, the live
match-stats extractor used on stream alongside PES 2017.

This plan also specifies **`libs/match_feed`**, the small lib crate through which the
tracker publishes live match events to the rest of the suite — primarily to the
[Music player](../music_player.md), which uses the feed to run the soundboard
autonomously (its "Autopilot" section specifies that behavior). Platform context is
in the [core plan](../core/README.md).

Unlike every other tool in the suite, the tracker's primary input is not files but the
**memory of the running game process**. Live tracking is Windows-desktop-only (see
"Platform notes"); opening and viewing saved match files (`.match.json`) works on
all platforms.

---

The plan is split into parts; section headings are unchanged from the
single-file plan.

| Part | Covers |
|---|---|
| [Reading the game](reading_the_game.md) | Reading the game |
| [Match model](match_model.md) | Match model and event derivation |
| [Match feed](match_feed.md) | The match feed (`libs/match_feed`) |
| [GUI view](gui.md) | GUI view |
| [Match files](match_files.md) | Match files |
| [Testing](testing.md) | Testing |

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
rather than porting a supposedly complete generator (see `match_files.md` "Match files").

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
its versioned JSON envelope (wire contract under `match_feed.md` "The match feed") instead of speaking a
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
