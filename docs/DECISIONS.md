# Decision log

Append-only record of decisions made while implementing 4cc Studio where the plan in `docs/plans/`
was silent (see "When the plan has gaps" in `AGENTS.md`). The plan itself is edited first and stays
the source of truth; this log exists so a batch of decisions can be reviewed at once.

Entry format, newest last:

```
## YYYY-MM-DD — <crate or area> — <one-line summary>
Decision: what was chosen.
Why: the reasoning, in "X not Y because Y causes Z" form.
Plan: <file>#<section> edited (or "no plan edit needed" with a reason).
```

## 2026-09-07 — code style — "no parent references" applies workspace-wide
Decision: the export object model's "no parent references" constraint is a workspace-wide style
rule: no child→parent back-references anywhere, pass resolved context down as parameters. Shared
ownership of immutable leaf data across threads (`Arc<[u8]>` textures, `Arc<MatchSnapshot>`,
`Arc<dyn Fn()>` wake callbacks) is not a parent reference and stays allowed.
Why: back-references force `Weak`/`RefCell`/struct lifetimes, which is the type-level complexity
the style section bans; a narrow rule would just move the friction to the next object model.
Plan: no plan edit needed — `core.md` "Key Decisions Summary" already states it unqualified;
`aesthetics_export.md` "Design constraints" remains the worked example.

## 2026-09-07 — development plan — `color_tools` and `elevation` are built in Phase 3
Decision: `libs/color_tools` (extraction only; the picker widget waits for Phase 8) and
`libs/elevation` are Phase 3 deliverables.
Why: neither crate was assigned to any phase. Phase 3's kit processing already calls
`color_tools` for kits without `colors.txt`, and Phase 3's `output/deploy.rs` writability preflight
is the first place an access-denied PES directory must fail cleanly; building them later would
leave Phase 3 with stubs.
Plan: `core.md#Phase 3: Processing logic` edited ("New lib crates this phase").
Superseded the same day by the phase split below: both crates are now Phase 2.

## 2026-09-07 — development plan — phases 1–2 split into skeleton vs. standalone libs
Decision (user): Phase 1 is workspace bootstrap + non-GUI `studio_core`; Phase 2 is every lib
crate verifiable on its own, including `model_convert` (native half) and `pes_savefile`, and
`fmdl`/`pes_model` `ops/` ship with their format crates. Consumer-shaped libs (`aesthetics_export`,
`pipeline`, `aatf`, `music_export`, `audio_engine`, `match_feed`) stay with their first tool.
Placement details chosen by the agent: `vtree` in Phase 1 (`PipelineEvent` needs `ScopePath`);
`python_bindings` last in Phase 2; old Phase 4 (model conversion) absorbed into Phase 2 so later
numbering is unchanged; old Phase 5 becomes "Savefile integration" (Team compiler savefile
writing, `aatf`, save editor tool logic) so `aatf` keeps a home; glTF stays Phase 7.
Why: libs depend only on libs, tools depend on libs at varying points, so all standalone libs
first; the consumer-shaped ones would be designed blind without their tool.
Plan: `core.md#Development Plan` intro and Phases 1–5, 7, 12–15 edited; cross-references in
`team_compiler.md`, `save_editor.md`, `refs_arranger.md`, `balls_compiler.md` renumbered.

## 2026-09-10 — code style — "boring Rust" is justified by reviewability, not by Python contributors
Decision (user): the code style section keeps its rules but changes its stated rationale. The
reason for boring Rust is that agents write most of the code and a maintainer learning Rust
reviews it, and that the rules close off the paths agents take when fighting the compiler; not
that hypothetical Python-fluent contributors must read it. Four rules were loosened where they
fought idiomatic Rust rather than cleverness: generics over std traits (`impl Read + Seek`) are
allowed without a plan reference; the "about three adaptors" iterator limit became a "reads as one
sentence" guideline; std's own abbreviations (`len`, `buf`, `iter`, `ptr`) are allowed
names; docs on `pub` items must say something the signature does not. One rule was added:
`pub(crate)` by default, so the doc requirement scopes itself to real crate APIs.
Why: "pythonic Rust" points the wrong way (dynamic dispatch and stringly-typed maps are the Python
idioms, and the rules ban exactly those), and the contributors it was written for are unlikely to
exist. Reviewability is the constraint that does exist. The loosened rules had no simplicity
benefit — a `binrw` parser over `&mut R: Read + Seek` or a four-adaptor `collect::<Result<_,_>>()?`
is not clever — but would have pushed agents into awkward workarounds to satisfy the letter.
Plan: no plan edit needed — the rules live in `CONTRIBUTING.md`, which was edited directly.

## 2026-09-10 — code style — rules added from the EGG-Translator post-mortem; toolchain pinned
Decision (user): five style rules added after auditing EGG-Translator (a mature, agent-written
Tauri app) for the pitfalls the style section targets: closed string sets are enums; the third
copy is a function; a module is a noun, not a layer; shared mutable state is one lock per concept
with a fact in one place (prefer swapping an `Arc<Snapshot>`); a discarded `Result` carries its
reason. Two clarifications: `.lock().unwrap()` on a std mutex is an allowed bare `unwrap`; review
checks for `// SAFETY:` explicitly. The toolchain is pinned by a root `rust-toolchain.toml`
(Phase 1 deliverable).
Why: the audit found none of the headline bans violated (no `Rc<RefCell>`, `Box<dyn Any>`,
`macro_rules!`, `#[allow]`) but found the failures the list did not name: `config.engine: String`
matched against literals in 34 sites with `_ =>` fallbacks; a 2950-line `commands.rs`; the same
100-line function twice and the same 8-step config-write sequence five times; seven
`Arc<Mutex<field>>` copies of values already inside `Mutex<Config>`, hand-synchronized; 75
`let _ =` including a dropped settings save; 12 `unsafe` blocks with zero `SAFETY` comments; and a
clippy gate broken by a `rustup update` alone (one new 1.98 lint). Each new rule closes one of
those. `lock().unwrap()` accounted for ~120 of 180 `unwrap`s; demanding a message on each would
be noise.
Plan: `core.md#Workspace guardrails` item 5 (toolchain pin rationale) and `core.md#Phase 1`
(deliverable) edited. The diagnostic-logging question is resolved in the next entry.

## 2026-09-10 — methodology — cross-family second opinion at fixed checkpoints
Decision (user): a reviewer subagent from a different model family than the orchestrator is
invoked at four checkpoints — after an Acceptance section; once per *substantive* worklog step,
after implementation and tests are written and before the tests run; as the first half of every
converge audit; reactively after two failed attempts on one premise — plus on demand via `/duck`.
Batching: a step is substantive when it adds behavior or touches more than one crate; trivial steps
are not reviewed alone, their diffs ride along with the next review; never more than one critique
per step. The reviewer receives pointers (sections, paths, a diff file, acceptance IDs), never the
orchestrator's summary; returns at most seven ranked concerns marked verified/suspected; every
concern is answered in the turn report. Profiles: `gpt-astra-high` (GPT, `model: gpt-astra-high` —
an alias resolving to the latest Astra; read-only; new) when a Claude orchestrates **or when the
orchestrator does not know its own family**; `fable-medium` only when it knows it is a GPT. Cost is
not the constraint; wall-clock time is, hence the batching.
Why: modeled on GitHub Copilot CLI's "Rubber Duck" (2026-04), whose stated rationale — a model
reviewing its own work shares its own blind spots — holds regardless of tooling; the subagent
mechanism already existed here unused. Checkpoints are fixed rather than "when the agent feels the
need" because an agent does not feel the need at exactly the moments its blind spots are active.
Converge is the best fit of all: an independent code-vs-plan audit is what a fresh model family is
for. The published gain is modest for a frontier orchestrator (3.8–4.8% for a mid-tier one on hard
tasks); the cost is accepted for the compounding-error cases it targets. One critique per step
rather than one after implementation and another after tests, because each critique is a full
subagent run and two per step would dominate the step's wall-clock; reviewing code and assertions
together also lets the reviewer check that the tests exercise what the code claims. GPT as the
default when the orchestrator cannot name its family, because most orchestrators in use here are
Claudes and a same-family review is worth less than a late one. Model alias `gpt-astra-high`
rather than a versioned id so the profile follows Astra releases without edits (user verified both
resolve).
Plan: no plan edit needed — process, not architecture; lives in `AGENTS.md` "Working documents"
and "Environment". Unverified until Phase 1: what a critique costs in wall-clock time — step 1.6
records the first data point.

## 2026-09-10 — methodology — refinements from the first cross-family review
Decision: six changes from the reviewer's seven concerns on today's documents. (1) Every worklog
step carries its own `→ verify:` check; a step listed without one gets one written before it
starts — phase-level verification alone left steps 1.1–1.3 with no done condition. (2) Converge
treats plan items the plan explicitly defers to a later phase as not-gaps, and the present-tense
rewrite covers only what the phase delivered — otherwise Phase 1's converge on "Event system" would
have demanded the status-strip metrics that section itself defers. (3) Phase 8 (GUI) joins the
acceptance schedule; scenarios no automated test can prove are marked `manual` and proven by a
recorded check at converge. (4) The converge check script has two modes: report (CI, fails only on
orphan citations — unproven IDs are the normal state of an open phase) and strict (at converge, for
the closing phase); it scans every `.rs` under `crates/`, not just `tests/`, and skips withdrawn
scenarios. (5) The one-critique-per-step ceiling applies to scheduled critiques; the reactive one
is an escalation and exempt. (6) Not decided here — **open, for the user**: `reqwest` (approved in
the dependency table for update checks and downloads) starts a Tokio runtime internally even in
blocking mode, which contradicts "no async runtime anywhere in the workspace" as written. Either
the rule narrows to "no async in our code; a runtime internal to a dependency is tolerated in
`studio` only, never in a lib" or `reqwest` is replaced by a runtime-free blocking client (`ureq`,
rustls-capable). Recommendation: `ureq` — the use case is two HTTP GETs per release, and a clean
rule is worth more than `reqwest`'s feature set here.
Why: each is a case the documents' authors could not see from inside them; the reviewer found all
seven in 3.5 minutes, none had surfaced in the day's several passes. Data point for the second
opinion's worth, recorded before Phase 1 rather than at step 1.6.
Plan: no plan edit for (1)–(5) (process; `AGENTS.md`, `WORKLOG.md`, `CONTRIBUTING.md` edited).
(6) resolved the same day — next entry.

## 2026-09-10 — dependencies — `ureq` replaces `reqwest`; "no async runtime" includes dependencies
Decision (user): the desktop updater's HTTP client is `ureq` 3.x (rustls, platform certificate
verifier). The "no async runtime anywhere in the workspace" rule is stated in `core.md`
"Parallelism" to include runtimes started internally by dependencies. Downloads stream via
`into_reader()` into the temp file (progress + hashing on the way), never through the in-memory
convenience readers.
Why: `reqwest` spins up Tokio even for its blocking client, so approving it required either a
footnote to the rule ("no async in *our* code") that later agents would cite to argue for more, or
a different client. The updater does two HTTP calls per release and needs nothing `ureq` lacks.
Costs accepted, in order of weight: agents know `ureq` less well and confuse its 2.x and 3.x APIs
(expect one extra compile-fix round trip per HTTP step); the body-size limit is a footgun for
anyone not streaming (hence the plan note). Gains: no Tokio/hyper/tower in the tree, a rule with
no footnote. Verified against the ureq 3.4.1 docs the same day: `read_to_*`/`read_json` carry a
10 MB default limit, `into_reader()`/`as_reader()` are unbounded, `into_with_config().limit()`
adjusts; features `platform-verifier` (OS certificate store) and `win-system-proxy` (Windows
proxy settings) both exist — the "env-var-only proxies" drawback assumed during the discussion
was wrong and is not a cost. `ehttp` (egui's author; `ureq` natively,
`fetch` on wasm) was noted, not chosen: the updater is desktop-only and a callback API is worse for
a streamed download with progress; it becomes relevant only if Studio Web ever needs HTTP.
Plan: `core.md#External Dependencies` row replaced; `core.md#Parallelism` "Design" opens with the
runtime rule and its reasons; `core.md#Self-update` step 3 carries the streaming note.

## 2026-09-10 — logging and lints — `log` facade for diagnostics; `let_underscore_must_use` denied
Decision (user): developer diagnostics go through the `log` facade in every lib and tool crate;
`studio` installs the sink (`env_logger` in CLI mode, a small `log::Log` file writer to
`studio.log` in GUI mode); `python_bindings` installs `pyo3-log`. No `println!`/`eprintln!`
outside `studio`'s CLI result output and `main`. Findings (`Message`) stay the user-facing channel;
libs never `error!`. `clippy::let_underscore_must_use = "deny"` joins the workspace lints, with
escapes in order: handle (usually `if let Err(e) = … { debug!(…) }`), route through one `emit`,
or `#[expect(…, reason = "…")]`; `.ok();` as a discard is forbidden. `#[expect]` is preferred over
`#[allow]` generally. `anyhow` added to the dependency table, where it was used but never listed.
Why: `log` not `tracing`, because the structured and timed data Studio needs already flows
through `PipelineEvent`, leaving flat diagnostics, and `tracing`'s subscriber layers are the
type-level complexity the code style excludes; the choice is no-regret since `tracing` can later
consume `log` lines in the binary alone. The lint, because EGG-Translator's agents commented some
discards and forgot others (a dropped settings save among them); a compile error is where agents
respond, and with `log` available the honest handling of an ignorable failure is a `debug!` line,
so the lint pushes toward the right code rather than toward suppressions.
Plan: `core.md#Diagnostic logging` added under "Event system"; `core.md#External Dependencies`
rows for `anyhow`, `log`, `env_logger`, `pyo3-log`; `core.md#Phase 1` lint deliverable.
Unverified until Phase 1: whether `fallible().ok();` actually dodges the lint (it is forbidden by
rule either way), and the residual `#[expect]` count — estimate is under ten workspace-wide; count
it at the end of Phase 3.

## 2026-09-10 — methodology — plan-driven kept; five practices grafted from spec-driven and Lode
Decision (user): the project stays plan-driven rather than adopting a spec-driven framework
(Kiro, spec-kit, OpenSpec) or its tooling. Five practices are added: (1) tool plans gain an
"Acceptance" section of stable-ID GIVEN/WHEN/THEN scenarios, written just-in-time before each tool
phase, tests citing the ID in a comment; (2) every phase closes with a converge audit (code vs.
plan sections and acceptance IDs; gaps become steps); (3) `docs/GLOSSARY.md`, one line per term
pointing at the owning plan section; (4) a phase's plan sections are rewritten in the present
tense when it closes, replacing the planned end-of-project reshape into spec documents; (5) Lode's
ownership standard ("maintainable without the assistant present") added to `AGENTS.md` conduct.
Acceptance IDs live inside the tool plans, not in a separate `docs/specs/` tree.
Why: mapped against the five sources, the existing documents already are a Lode (`AGENTS.md`
summary + conduct, `CONTRIBUTING.md` practices, `DECISIONS.md` ADRs, plans as subsystem docs) with
a forward plan; the genuine gaps were enumerable requirements (nothing let an agent list every
behavior and whether it is tested), step-level "done" (verification was per phase), plan/code
drift over 17 future-tense phases, and scattered terminology. Frameworks were not adopted because
their tooling is lock-in for a procedure an agent can read from markdown, their per-feature change
folders solve parallel brownfield changes the serialized worklog does not have, and user stories /
EARS are theater for format crates whose spec is the fixture and the parity standard. Acceptance
sections are just-in-time because requirements written far ahead of their code go stale; they are
inside the plans because two homes for one behavior drift during the build (a `specs/` +
`changes/` split is the right shape only if the project reaches maintenance mode). Lib phases get
no acceptance section for the fixture reason above.
Plan: `plans/README.md` "How these documents evolve" added (replaces the end-of-project reshape
sentence); `AGENTS.md` "Working documents" (table, before-tool-phase and closing-a-phase
procedures), "Read order" (glossary), "Conduct" (ownership standard); `CONTRIBUTING.md` "Testing"
(acceptance format, ID scheme, citing rule); `WORKLOG.md` (phase-close steps 1.6–1.7, 2.19–2.20,
Phase 3 acceptance step 3.1, resume table).

## 2026-09-11 — methodology — three roles: Claude lead, SWE-2 sidekick, GPT reviewer
Decision: sessions run in Devin CLI's Fusion mode from Phase 1 step 1.1 — a Claude lead that
plans, briefs, reviews diffs and talks to the user; an SWE-2 sidekick that implements against plan
sections and runs the gates; the `gpt-astra-high` reviewer kept for lead-authored artifacts. The
lead hand-writes only correctness-critical infrastructure (lints, CI, toolchain pin, acceptance-ID
scanner, fixtures, measurements). Second-opinion checkpoint (b) narrows to steps that add a `pub`
interface or cross a crate; (c) converge becomes the primary use; (d) reactive also fires after two
failed sidekick attempts on one brief.
Why: three roles not two, because the sidekick and the reviewer do different jobs — one produces,
one judges — and Fusion leaves every artifact the reviewer exists to check (plans, acceptance
sections, briefs, converge audits) still written and reviewed by the same family; dropping the
reviewer would reopen exactly the blind spot the first review demonstrated (7 concerns none of
which a same-family pass had found). Checkpoint (b) narrows because sidekick code is already
reviewed cross-family by the lead; its residual risk is a wrong *brief*, which a per-step review
against that brief cannot see and converge against the plan can. Fusion from step 1.1 rather than
"after a skeleton exists" because the reason for waiting — style replication — was tested and
did not hold: a cold probe (`TeamName` derivation, from `CONTRIBUTING.md` and the plan section
alone) came back in 3.5 min, red-first, gates green, style rules followed literally, and with the
plan's two real ambiguities reported rather than decided. Starting early builds the rework-rate
evidence while steps are small and lead review is cheap. Gate infrastructure stays lead-written
because its wrong version passes silently rather than failing, which is the one place a plausible
implementation is worse than none.
Plan: `AGENTS.md` "Lead and sidekick" (new), "Second opinion" (checkpoints redrawn), "Environment"
(three roles); `WORKLOG.md` step 1.6 (sidekick rework/takeover data point), log line;
`team_compiler.md` open questions (team-name lowercasing rule, surfaced by the probe).

## 2026-09-11 — team compiler — teams list: `.txt`, embedded upstream, one working copy in `data/`
Decision (user): the file keeps Red's name `teams_list.txt`. The current cup's list is embedded in
the binary like the templates; the only on-disk copy is `data/teams_list.txt`, created from the
embedded one on first run, written by the grid's ID cell and by the updater's merge (which now
runs on the new binary's first start, embedded → working). Writes are best-effort: an unwritable
data directory makes the list read-only (`teams_list_read_only`), detected by attempting the write;
no elevation path.
Why: `.txt` not `.tsv` because the extension is for the user, not the parser — `.txt` opens in a
text editor on double-click, `.tsv` opens nothing or a spreadsheet that rewrites cells, and it is
the name organizers already pass around (Red's file is already tab-separated). In `data/` not
beside the exe because the grid writes into it, which makes it user data: beside the exe it would
not follow the settings into the config directory and would be lost with a replaced program
folder. Embedded not bundled so there is exactly one file on disk to edit and no "which copy" —
the reconciliation the updater already specified becomes the only sync path. No elevation
because Studio is a portable app; a portable install under Program Files is a corner case not
worth a privilege path for a one-line edit.
Plan: `team_compiler.md` "Resolved decisions" (new "Teams list" bullet), settings table
(`teams_list_path`), "Path resolution", "Team ID cell", message catalog (`teams_list_read_only`);
`core.md` "Distribution" (bundle contents), "Data location" (data-dir cargo), "Self-update" step 5,
Phase 16 bundling; `aesthetics_export.md` crate tree comment; `GLOSSARY.md` "Team ID";
`export_upgrader.md` (rename only).

## 2026-09-11 — team compiler — team-name fold is Unicode on both sides; savefile is a teams-list source
Decision (user): the first token of the export name and the teams list's Name column are both
lowercased with Unicode `str::to_lowercase`; first *non-empty* token, no token → rejected. The
savefile's `TeamEntry` rows (IDs 701–920; in-game names are the `/xx/` names) are a second source
for the existing teams-list reconciliation: on-demand "Import from savefile" button in the Team
compiler's settings and `studio team-compiler teams-list import-savefile`, savefile wins conflicts,
reviewed before writing, never automatic.
Why: Unicode not ASCII because that is what Blue did — Python's `str.lower()` is Unicode-aware and
Blue applied it to both sides — so ASCII-only would be a divergence with nothing bought; board
names are ASCII anyway (zero non-ASCII names in every `teams_list.txt` under `Cups/`, `Refs/` and
the compiler folders; `/umaJP/` is the one mixed-case entry, and both-sides folding handles it).
Savefile import because the save is the list the game actually uses for the current cup, which
makes it more authoritative than the last release's embedded copy, and the compiler already opens
it; through the merge, not an overwrite, so the user's local ID assignments survive; on demand,
because the list must keep working without a savefile and a compile must not rewrite user data
as a side effect.
Plan: `team_compiler.md` Reader step 2 (fold rule, first non-empty token), "Export identity
resolution" (column contract, dead columns, placeholders), "Resolved decisions" "Teams list"
(savefile source), "CLI" (`teams-list import-savefile`), open questions (lowercasing removed;
`teams_list.txt` contract narrowed).

## 2026-09-11 — studio_core — shell status bar with typed slots; "run strip" for the tool toolbar widget
Decision (user): the shell gets a one-row, full-width status bar present in every tool, with three
typed slots — `ShellCondition` (persistent environment states, shell-computed), `ToolActivity`
(background work of tools other than the active one, via `StudioTool::activity()`), `Notice`
(latest one-off event with at most one action, via `ctx.notify`). Static empty state (version +
data location). It is the home for what the plan called "toasts" in three places. Settings saves
become best-effort and raise `data_dir_read_only` with the teams list. The per-tool toolbar widget
is renamed "run strip".
Why: a shell-level bar, not per-tool strips, because the conditions it exists for ("data folder
read-only", "a compile is running behind this view", "update available") are not any one tool's
business — and the plan already relied on an unspecified toast mechanism for three of them.
Typed items, not free text, because a `set_status_text(String)` API is how status bars become
clutter; the type set is the admission test. Always-present, not auto-hiding, because a row that
appears shifts the tool view every time a notice arrives. "Run strip" not "strip" because *strip*
already means a kit in this codebase (`StripSlot`, 4ccEditor's `stripBlock`); "status strip" would
collide with "status bar" in every future session.
Plan: `core.md` "Status bar" (new, under GUI Design), "Shell layout" (diagram, panel order),
"Tool plugin interface" (`activity()`, `ctx.notify`), `studio_core` inventory and module tree
(`status.rs`, `shell/status_bar.rs`), "Settings menu" (best-effort save), sidebar auto-switch
(notice), "Crate structure" presentation sentence, Phases 1 and 8; "run strip" rename across
`core.md`, `team_compiler.md`, `balls_compiler.md`, `match_tracker.md`.

## 2026-09-11 — save editor — teams-list refresh from the savefile lives in the Save editor, as "Export teams list"
Decision (user): supersedes the location in the previous "savefile is a teams-list source" entry.
The action is the Save editor's **Export teams list** (feature table, "Database operations") and
`studio save-editor export-teams-list <EDIT> [--yes]`; the Team compiler's settings button and
`teams-list import-savefile` are withdrawn.
Why: the Save editor owns the open savefile; from its point of view the operation is an export of
the save's team table, and the compiler is only a consumer of the resulting file. Same
reconciliation, same review, same read-only handling — only the owner changed.
Plan: `save_editor.md` "Database operations" row, "CLI"; `team_compiler.md` "Teams list" decision
(points at the Save editor), "CLI" (command removed, pointer added).

## 2026-09-11 — libs — `teams_list` is its own leaf crate; `TeamName`/`TeamId` move there
Decision (user): a new dependency-free lib crate `teams_list` owns `TeamName` (with the one
fold, `TeamName::new(token)`), `TeamId` (701–920), `teams_list.txt` parse/write with the embedded
upstream list, and the reconcile/merge. `aesthetics_export` depends on it and keeps only the
export-format half of identity: splitting the display name into the first token. Built in
Phase 2 as step 2.12, right after `fpc`.
Why: three consumers — the Team compiler, the Save editor (`export-teams-list`) and the `studio`
updater — and none is a natural owner; leaving it in `aesthetics_export` made the Save editor and
the updater depend on the whole export object model for one small file format, which is the
"every piece of shared functionality is its own lib crate" rule broken at the first opportunity.
The fold lives in the crate that both sides call so the export-name side and the list side cannot
drift; the split stays in `aesthetics_export` because "first word of the folder name" is export
knowledge, not team knowledge. Same shape as `fpc`: a leaf crate holding data and pure rules,
callers own I/O.
Plan: `libs.md` "`libs/teams_list`" (new) and intro list; `aesthetics_export.md` ownership
paragraph and crate tree (`teams_list.rs` removed, `identity.rs` re-described); `core.md`
"tempting misplacements", crate diagram, updater text and step 5, Phase 2 leaves; `AGENTS.md`
read-order row; `GLOSSARY.md` "Team name"/"Team ID" owners; `WORKLOG.md` Phase 2 row and step
2.12 (2.12–2.20 renumbered to 2.13–2.21).

## 2026-09-11 — export upgrader — model-folder mode; CLI surface written down
Decision (user): the upgrader accepts, besides whole exports, a flat folder of old-format model
folders (optionally with a `Common/`), migrating each with the per-folder rules (`kitN`, `.mtl`
stems, type suffixes, `ingame_face`) and naming outputs so they drop straight into a Studio export
(`NN - Name/` for faces, name-part shared folders for boots/gloves). No savefile, kits, portraits,
logo or `settings.toml` in this mode. CLI: `upgrade` and `upgrade-models`; GUI: two drop targets.
Why: the compiler has no notion of the legacy spelling, and the `u0???pN` → `kitN` rewrite reaches
into FMDL path tables — an author adding one new face to an already-upgraded export must not be
told to re-upgrade the whole export or hand-edit binaries. One level, one folder per model, rather
than the converters' two-level players layout, so the tool never guesses which directory is "a
model folder". Both `_r_ll` and `_r` of an old `Logo/` are carried (as `logo.png` / `logo_small.png`)
because a deliberate small variant is indistinguishable from a downscale and silent loss is worse
than a file the user can delete. The plan previously had no CLI section for this tool at all.
Plan: `export_upgrader.md` "Model-folder mode" (new), "CLI" (new), step 1 pointer, step 7 (Logo
conversion).

## 2026-09-11 — aesthetics export — logo is one root image (+ optional `_small`), sizes generated at compile time
Decision (user): the `Logo/` folder and the hand-made `emblem_*_r/_r_l/_r_ll.png` triplet are
gone. The export root holds `logo[_<fit>].<ext>` (any accepted raster format, any size) and
optionally `logo_small[_<fit>].<ext>`; `<fit>` ∈ `crop`/`stretch`/`fit`, default `fit` for a
non-square source. The compiler decodes, squares, Lanczos-resamples to 512²/256²/128², PNG-encodes
and names them as Red did; `_small` replaces only the 128² image. Messages `logo_file_invalid`,
`logo_role_duplicate`, `logo_small_without_main`, `logo_fit_applied`, `logo_upscaled` replace
`logo_files_invalid`.
Why: one source cannot get the sizes wrong — one of the eleven shipped `Logo/` folders on disk is
510² in all three files — and the placeholder-team-ID filenames were pure ceremony. `_small` as a
separate file, not a crop region or a flag, because the menus' 128² logo is the one teams
deliberately draw differently (simplified, joke), and "a different picture" is what a file is.
Default `fit` because it is the only squaring that neither discards pixels nor distorts; the tags
exist to opt into loss, not to avoid it. Small never feeds the large sizes: upscaling a 128² image
to 512² is not what anyone means. `image` is already a dependency through `dds_convert`, so no new
crate.
Plan: `aesthetics_export.md` "Root files" (diagram, "Logo" bullet), `ValidatedAestheticsExport`
(`logo: Option<LogoFiles>`); `team_compiler.md` validation categories, "Processing" Logo step,
message catalog, Textures scoping note, test list; `export_upgrader.md` step 7.

## 2026-09-12 — export upgrader — old small logo carried only when clearly a different picture
Decision (user): supersedes the "both always" rule in the 2026-09-11 model-folder entry. The
upgrader downscales the old `_r_ll` to 128² as the compiler would (Lanczos3), compares it to the
old `_r` over RGBA, and writes `logo_small.png` only when more than 40 % of pixels differ; a pixel
differs when any channel is off by more than 32/255, and pixels transparent in both images are
equal. Either outcome is reported with the measured share (`logo_small_carried` /
`logo_small_dropped`, both I). Also closed: default `fit` stays; the three logo sizes are
confirmed identical in PES 21 (user checked the game files), so the PES 20+ size check is no
longer open.
Why: most old small logos are downscales the compiler now regenerates, so always carrying them
seeds every upgraded export with a redundant file the user must notice and delete; always dropping
loses the deliberate variants. The threshold was measured, not guessed: over the 52 distinct
`Logo/` pairs in the maintainer's library, downscales score 0–29 % (authors used nearest-neighbor
and sharpening kernels and cut alpha edges differently — noise the channel tolerance does not
absorb, so the user's first guesses of 15 % and then 3 % would both have carried most downscales)
and deliberate variants score 63–97 %; 40 % sits in the empty band on the side where the remaining
possible mistake is a redundant file, not a lost drawing. Reporting the share both ways keeps a
wrong call visible and recoverable — the source export still holds the file.
Plan: `export_upgrader.md` step 7, "Verification" fixtures; `team_compiler.md` "Processing" Logo
step (PES 21 size note).

## 2026-09-12 — aesthetics export — kit folders take a label; `Kits/all/` textures are inherited per stem
Decision (user): a kit folder is `<slot>[ - <label>]` (`p1 - Lakers`), the same shape as player
folders; the slot (`p1`–`p9`, `g1`, `all`, case-insensitive) is all the compiler reads, the label
is display only (kit cell tooltip, Kit config editor tab). A `Kits/all/` folder holds textures every
kit inherits for the stems it lacks; own files win per stem; the main `kit.dds` may be shared too.
Config texture-name fields derive from the effective set. `all/` is not a kit: no cell, no
config/colors/icon (reported and ignored), warned when no kit exists to serve. A shared base
`all/config.toml` was considered and rejected (user): a config-less kit folder means "template",
so it can be copied between exports and look the same; an inherited config would change it
invisibly, where an inherited texture changes it visibly and often intentionally. Inheritance is
resolved once, in `aesthetics_export` (`KitFolder.textures` with `KitTextureSource::{Own, Shared}`),
not by each consumer. Lead additions, revertible: `kit_slot_duplicate` discards both folders rather
than picking; the upgrader hoists a stem into `all/` only when it is byte-identical in *every* kit
(`kit_texture_hoisted`), never when some kit lacks it, so its output compiles unchanged.
Why: most teams ship the same `_back`/`_leg`/`_name` textures on all kits — nine copies that bloat
the export and drift when one is updated. Per-stem inheritance keeps the single-override case
(`p2` with its own name font) natural. Resolving in the format crate is what keeps the compiler's
emitted bytes, the editor's derived texture-name fields and the upgrader's view identical. A
survey of 129 multi-kit old exports: `_back` byte-identical across all kits in 16 of 65 exports
that have it, `_leg` 16/57, `_name` 14/65 — the all-identical hoist helps a quarter of exports and
is the only hoist that cannot change game-facing output; partial sharing is left to the author.
Plan: `aesthetics_export.md` "Kits" (grammar, `all/` rule, diagram), object model (`KitsFolder`,
`KitFolder`, `KitTexture`), `kits.rs` comment; `team_compiler.md` validation, "Processing" Kits,
message catalog (`kit_folder_invalid` widened, `kit_slot_duplicate`, `kit_textures_inherited`,
`kit_all_file_ignored`, `kit_all_unused`), kit cells, test list; `kit_config_editor.md` derivation
input and tabs; `export_upgrader.md` step 5 (bare slots, hoist); `core.md` decisions table;
`GLOSSARY.md` "Kit folder", "Kit slot".

## 2026-09-12 — team compiler — empty kit folders are placeholder kits
Decision (user): a completely empty kit folder is valid and declares the slot. The compiler fills
it with a bundled placeholder main texture — the magenta/black Source-engine "missing texture"
checkerboard — the template `config.toml`, and a `UniColor` entry as for any kit. A kit with no
usable `colors.txt` and no own texture to derive from gets a fixed loud magenta/black pair
(`kit_colors_missing`, W, now written rather than skipped). Generalized (lead): the fill applies
to any kit whose *effective* texture set lacks `kit.dds`, so a folder with only a number font is
a placeholder with a custom font, not an error; the same template mechanism now states Red's
silent `kit_mask.dds` fill, which the plan listed as an embedded template without saying what it
was for. Lead detail: 64² DXT1 with mips like Red's mask template (~3 KB), 8-pixel checks so DXT1
encodes it losslessly; placeholder kits never derive colors from the checkerboard.
Why: teams with full-body-model rosters never render a kit texture (except a pre-Fox `uniform`
model type), yet the game wants every used slot to have a texture and a config; the empty folder
says "pretend a kit exists" without a drawn texture. The checkerboard over a plain black or white
because a placeholder that does get rendered (a stray non-full-body player, the GK) is then read
as exactly what it is by anyone in the community. A loud color pair over the two quiet options —
skipping the entry (leaves a previous cup's colors in the base bin) or a plausible stand-in such as
the team's root colors (looks chosen while unchosen; "we'll never have all kits matching that") —
because a missing choice should be seen in the first menu. Tiny template because nine placeholder
kits per team should cost nothing in the CPK.
Plan: `aesthetics_export.md` "Kits" (empty-folder paragraph, diagram); `team_compiler.md`
validation, "Processing" Kits (template fill), "Kit colors fallback", embedded templates list,
message catalog (`kit_placeholder`, `kit_colors_missing` reworded), test list; `core.md`
decisions table.

## 2026-09-12 — aesthetics export / team compiler — kit layout marker and cross-engine re-layout

Decision (user): a kit folder may hold an empty `pre-fox` or `fox` marker file declaring which
engine's kit UV layout its main texture (and mask/srm) is drawn for; compiling for the other
engine re-lays those textures out. No marker = drawn for the compile target (no edit). The
marker is a file, not a `config.toml` key, because `config.toml` holds only data that reaches the
game's kit config. Only `kit` and its `kit_mask`/`kit_srm` are re-laid out (`_chest`, `_back`,
`_name`, `_leg` have their own UV spaces). The Export upgrader never writes the marker: kits have
gone through several converters unchanged, so their origin is unknowable. Lead additions: both
markers → `kit_layout_conflict` (kit discarded, the `fpc_conflict` shape); a marker in `all/` is
`kit_all_file_ignored`; conversion reported once per kit (`kit_layout_converted`); the remap is a
lead-authored const table of rectangle moves, `KIT_LAYOUT_REMAP`, read in either direction; the
mask template fill is now stated as pre-Fox only, which is what Red's `kit_masks_check` does.
Why: the two engines' kit layouts differ, measured from both games' uniform models (PES 17 dt32
`nocloth` + dt35; PES 21 dt35 + `common_package_fpk/.../pes16/.../common`): shirt, sleeves and
collar strip identical; sock islands 432 → 364 px wide, outer-anchored, same height; shorts
islands same outline but partitioned differently (444-px body + 212-px inner strip vs 152-px hem
strip + 456-px body). Neither `aes_converter_16to21` nor the 19→16 converter edits the layout,
so every cross-engine kit shipped so far has displaced socks and shorts. The exact band table is
deliberately *not* fixed in the plan: texel matching through the models' 3D positions is reliable
in u but drifts in v (a shirt control run, provably identical, drifted up to 80 px because the
Fox body has other proportions), so the numbers are settled in the Phase 4 kits step from that
matching plus a hand-adjusted fixture pair, and recorded there. Evidence scripts and images in
`.tmp/kit_uv_*.py`, `.tmp/kit_diff_*.png` (not part of the repository).
Plan: `aesthetics_export.md` object model (`KitFolder.layout`, `KitLayout`), "Kit layout marker"
paragraph, diagram; `team_compiler.md` validation, "Processing" Kits (layout conversion, mask fill
scope), message catalog (`kit_layout_conflict`, `kit_layout_converted`), test list;
`export_upgrader.md` step 5; `core.md` decisions table; `GLOSSARY.md` "Kit layout".

## 2026-09-12 — team compiler — kit `_mask` (pre-Fox) and `_srm` (Fox) are engine-specific, emitted for their engine only

Decision: `kit_mask` is emitted for PES 15–17 targets and `kit_srm` for 18–21; the other engine's
map is not emitted (`kit_texture_not_used`, I) and is never converted into the target's; a pre-Fox
target without a mask gets Red's flat template (unchanged rule), a Fox target without an srm gets
nothing. Dropping a `_mask` on Fox targets deviates from Red, which ships it converted and unread.
Why (user asked whether the cross-engine converters' handling should be taken over): measured
rather than copied. The converters only handle Fox→pre-Fox — drop the srm, write a flat
(150,130,0) mask, which is Red's own template value (151,130,0) — and their "rewrite `_srm` to
`_mask` in the config" is dead code: the field at 0x48 is `chest`, and none of the 788 PES 17 or
1359 PES 21 stock configs names a mask or srm anywhere; the game finds both by name. The two maps
differ in meaning (stock averages (148,133,13) vs (75,141,0); Fox packs specular/roughness/
metallic), so any formula between them would be an invention. Stakes are low either way: 0 of 365
library kits ship an `_srm`, 14 a `_mask`, and 4cc's PES 21 cups ran without srms.
Plan: `team_compiler.md` "Processing" Kits (mask/srm paragraph), message catalog
(`kit_texture_not_used`), test list; `aesthetics_export.md` Kits diagram.

## 2026-09-12 — model conversion — the authoring format is the "global model file" (`.glb`) to users

Decision (user): user-facing text calls the glTF authoring format the **global model file**,
extension `.glb`, joining the community's "fox model file" (`.fmdl`) / "pre-fox model file"
(`.model`) family. Rules (lead): it is a nickname, never written as an expansion of "glb"; one
bridge sentence beside the Blender export settings names glTF 2.0 as the exporter's real name;
`.gltf` + `.bin` stays accepted and is documented once, never in messages; plans, code,
identifiers and `PES_*` extension names keep "glTF".
Why: most members have never heard of glTF but all say "fox model file"; a third nickname in the
same pattern costs nothing to learn, and `gl-b` → "global" echoes its letters the way `f-mdl` →
"fox model" does, which is how the other two stuck. "Universal" and "dual" were considered:
neither echoes the extension, and "dual" reads as two models and undersells a file that also
opens in Blender, any glTF viewer and the future WASM tools. Not an expansion because the first
menu a member meets says "glTF 2.0", and a taught etymology would contradict it there.
Plan: `model_conversion.md` "Blender integration" ("What users call it"); `GLOSSARY.md` "Global
model file".

## 2026-09-12 — studio_core — help window assembled from per-tool chapters

Decision (user): a cross-tool help window, opened like the settings menu, whose content lives in
each tool crate and is collected by the window; it replaces Red's README and `readme_advanced.md`
and has a search box. Choices confirmed by the user: Markdown rendered with `egui_commonmark` (new
dependency) rather than a hand-rolled renderer; a "Messages" topic per tool generated from
`messages.rs`; message IDs in the log panel link into the help. Lead additions: `StudioTool::help()`
returning a `HelpSection` of `include_str!` topics from a `help/` folder in the crate (now part of
the mandatory tool-crate skeleton); core chapters and the changelog from `studio_core`; the window
opens on the active tool's chapter (F1 or the sidebar `?`); in-app egui window, not a viewport;
substring search, no index; `studio help [tool] [topic]` and `studio help --export <dir>` so the
release page and wiki are produced from the same text; the authoring rule that a user-observable
change lands with its topic.
Why: the plan had "in-app help and changelog" as a Phase 16 line and nothing else. Collecting
chapters from the tools follows the settings-menu pattern for the same reason — a chapter cannot
outlive or precede its tool. The generated Messages topic is what makes search worth having: the
one thing Red's users looked up by hand was a message ID. `egui_commonmark` over a hand-rolled
renderer because the latter is a permanent maintenance item for a solved problem; in-app window
over a second OS window because multi-viewport adds platform bugs a text reader does not need.
Plan: `core.md` `StudioTool` trait, `studio_core` inventory and module tree (`help/`,
`shell/help_window.rs`), tool-crate skeleton (`help/`), sidebar item 5, new "Help window"
section, "Log panel" cross-linking, Distribution readme line, Phase 8 and Phase 16 items,
dependency table, decisions table; `CONTRIBUTING.md` placement rule; `GLOSSARY.md` "Help chapter".

## 2026-09-12 — studio_core — one changelog with three readers; the version shows in the status bar only

Decision: `CHANGELOG.md` at the repository root (Keep a Changelog shape, bullets grouped by tool
inside a version) is the only description of releases; it is read by the help window's "What's
new" chapter (embedded at build), by the GitHub release body (the release process copies the
version's section, which the updater dialog already shows inline), and by a post-update status
bar notice `Updated to vX · [What's new]`. The version string is `v{CARGO_PKG_VERSION}` plus
` (debug)` under `cfg!(debug_assertions)`, no git hash (user agreed); shown in the status bar
empty state, in About (logo click: detailed block with Copy) and by `studio --version`; not in
the title bar and not beside the logo. The status bar empty state is the bare version (user):
"portable" named the only kind there is, and the data location it stood for is a once-made
choice whose failure mode already has the `data_dir_read_only` condition — About carries it for
the rare report that needs it.
Why: three surfaces that show release notes drift unless they are one file read three ways; a
what's-new popup is read by nobody who did not ask, a dismissable notice is; a git hash needs
build machinery for information the tag carries; the title bar is outside every cropped
screenshot and the logo area vanishes with the collapsed rail, so neither is a place to *find*
the version.
Plan: `core.md` "Changelog and version display" (new, under Distribution and updates), "Status
bar" (empty state, post-update notice), "Sidebar" item 1 (About), "Launch modes" (shell-owned
commands), "Help window" (chapter name), module tree (`shell/about.rs`), Phase 16.

## 2026-09-12 — studio_core — the window title shows ongoing work and the last result

Decision (user): the title bar shows the status of ongoing processes, as Red's console title did
(`1 - …`, `Done - …`), with more detail. Lead shape: `<activity> — 4cc Studio` from every tool's
`ToolActivity`, the active tool included (the status bar hides the active tool's, the title has
no duplicate to avoid); progress first because taskbar buttons truncate from the right; when
idle and unfocused, the latest `Notice` text is held in the title until the window regains
focus — Red's "Done" with the result in it, from the notice the tool already publishes; never
the version, never a `ShellCondition`; `ViewportCommand::Title` on change only; no
`ITaskbarList3` progress (not exposed by winit/eframe, text covers the need).
Why: the title bar is the one surface seen while the window is not in front, which is exactly
when a user wants to know whether a cup compile is still running — the opposite of the version
number, which is looked up with the window open. Reusing the status bar's typed items means no
new tool API and no free-text title.
Plan: `core.md` "Window title" (new, after "Status bar"), `activity()` doc comment, module tree
(`shell/title.rs`), "Changelog and version display" title-bar bullet, Phase 8.

## 2026-09-12 - workspace - a root justfile is the single definition of the gates

Decision (user, on the lead's recommendation): the workspace has a root `justfile`; `just gates`
is the one definition of the four verification gates and CI runs the same recipe; the other
recipes are the repeatable sequences the plan already names (`deps-check`, `acceptance`,
`parity`, `bindings`, `release`). Rules: a recipe is a command list, never logic (branches and
loops go to `scripts/`); recipes run under `sh` on both platforms with bare commands; nothing
hardcodes `target/`; `just` is a developer tool like rustup, not a workspace dependency, and the
cargo commands keep working without it. The file is lead-authored (a wrong gate list passes
instead of failing).
Why: the gate list already had two homes that had drifted (`cargo fmt --all --check` in
`CONTRIBUTING.md`, `cargo fmt --check` in the Phase 1 CI bullet), and the wasm gate was an
instruction ("every lib crate you touched") rather than a command, re-derived by every session
and encoded a second time in CI. Rejected: `cargo xtask` (a crate of Rust to review and maintain
for what is thirty lines of commands); cargo aliases (one cargo subcommand each, cannot sequence
or call a script). Side effect: bare recipe lines under `just` do not hit the PowerShell
false-failure trap, so the gates give an unambiguous red or green.
Plan: `CONTRIBUTING.md` "Testing and verification" (gates block and justfile rules), `core.md`
Phase 1 (justfile bullet, CI bullet) and Phase 16 (release recipe); worklog steps 1.1 and 1.2.

## 2026-09-12 - workspace - dependencies build without debug info; no shared target directory

Decision (user): the workspace profile sets `[profile.dev.package."*"] debug = false`; workspace
crates keep full debug info. A machine-wide shared `target/` (`build.target-dir`) was considered
and rejected.
Why: measured on this machine, a dev `target/` is dominated by dependency artifacts (EGG
Translator: 24 GB `debug/deps/`, 15 GB `incremental/`, of 48 GB), and dependency debug info is
never stepped into. A shared target directory only deduplicates artifacts whose crate version,
resolved features, profile and rustc version all match; across the four existing Rust projects
on the machine (zed, vscode-cli, EGG, one more) the measured overlap was 0.02 GB of 60 GB, and
Studio's pinned toolchain makes a match with any unpinned project unlikely - so it would buy a
shared build lock for no space. Stale-artifact growth has no profile fix; it is `cargo clean`
or `cargo-sweep`, periodically.
Plan: `core.md` Phase 1 workspace bullet. `CONTRIBUTING.md` already forbids recipes from
hardcoding `target/`, so a developer who moves it anyway is unaffected.

## 2026-09-12 - aesthetics export / Team compiler - shared textures via `.common` links, packed in place

Decision (user): a texture in the export's `Common/` folder is referenced from a player folder
with a texture link file (`hair.png.common`), the same `.common` mechanism already used for
models and material files - not with an in-toml tag (`<common>/hair` was the proposal and was
dropped by the user's own suggestion). A texture whose resolved file is in `Common/` - through a
texture link, a stem set in a Common material file, or a Common-linked model's own textures - is
packed once in the team's Common output and referenced in place; a texture resolved in the player
folder is relocated to the per-player common subfolder as before. This changes the compiler's
step 6, which previously relocated Common-model textures per player. The three "missing Common
target" findings collapse into one `common_link_missing` (E) whose context carries the link kind
and path. The Export upgrader turns legacy `common/???/` game-path references into stem + link
file, because stem normalization alone would silently re-point them at the player folder.
Why: one mechanism instead of two; a link works with texture auto-detection (no toml editing for
the common case) and with native `.mtl`/FMDL references, which a toml tag could not; provenance
is visible in Explorer. In place, not relocated: relocation would give N copies of the one
texture a user put in Common precisely to have one; Common is a location the game reads on both
engines (Red packs it and rewrites `common/XXX/` on both), so nothing is gained by moving it.
Plan: `model_format.md` "Link files" (table row, packing rule, why no tag), folder diagram,
findings; `aesthetics_export.md` "Common model links and model merging"; `team_compiler.md`
step 6, material stems note, findings table, parity differences; `export_upgrader.md` steps 7
and 13; `GLOSSARY.md` "Common link".

## 2026-09-12 - Team compiler - user-supplied pre-Fox `face.xml` is accepted, second-class, checked by evidence

Decision (user): a pre-Fox model folder may ship its own `face.xml`; the compiler accepts it
without first-class support, as the way to experiment with pre-Fox engine capabilities the format
does not map. The severity rule for its checks: **error** for what is known or structurally
certain not to work (Red's `xml_check.py` checks, plus a `<model>` without `path` and a doubled
face diff); **warning** for everything outside the vocabulary the compiler's own generator emits
(unknown element/attribute/type, non-numeric `ratio`, a path form the compiler cannot resolve,
an unlisted model file), kept verbatim; **info** for the compiler's own rewrites (the `face_neck`
dummy, `uniform` -> `uniform_sub` on PES15, ID substitution, `<dif>` insertion, ignored on Fox)
and for `level` != 0, which the user knows to be the LoD level: uncommon, never a crash. Filename typing is off in such a folder; only what the xml lists is emitted. New
in-game knowledge moves a value between the warning and the known sets.
Why: a hand-written xml can crash the game, so nothing goes through unexamined; but forbidding
what is merely unmapped would close the only door to discovering what the pre-Fox engines can do.
"What the generator emits" is the one set with in-game evidence behind it, so it is the honest
warning boundary, and it is maintained for free because the generator is the compiler's own code.
Red waved unresolvable game paths through silently; the compiler says it did not check them.
Plan: `team_compiler.md` "User-supplied `face.xml`" (new, under the XML/MTL content checks), the
XML rows of that catalog table, step 4, test list; `aesthetics_export.md` "Model names: a free
part plus a suffix".

## 2026-09-12 - Team compiler / save editor / pes_savefile - the aesthetics patch

Decision (user): every compile writes `aesthetics_patch.toml` beside the output CPK - the
resolved savefile writes (settings, names, FPC values, boots/gloves IDs, edit flags) for every
compiled player, in `pes_savefile`'s `PlayerSettings` schema plus the compiler-owned fields; the
save editor applies it (`apply-patch` in GUI and CLI, version and allocation-scheme must match,
preview through the comparator, undoable). Lead shape, accepted: the compiler updates a
configured local savefile **by applying that same patch**, so it has one savefile write path;
a missing local savefile stays a warning (user: even the DLC builder compiles against a template
save to test in PES - no "patch only" mode, no `savefile_path = none`). `PlayerSettings` must
cover every appearance field of the savefile except boots/gloves IDs and edit flags, with a test
over the per-version schema tables enforcing it, and the generated template lists every key
(unset ones commented, with ranges). Motion fields (hunching, arm movement, kick motions, gc1/gc2,
dribbling) are aesthetics (user). The save editor's aesthetics fields and every other aesthetics
write path (team ops, FPC toggle, transplant, import sections) are **read-only by default**, with
a per-session "Edit aesthetics anyway" unlock (user chose the unlock over a hard lock); CLI
commands are not gated.
Why: the DLC builder and the official-savefile builder are different roles, and the official save
carries the teams' custom tactics, which the DLC builder must not see - so the compile's savefile
half has to be a file that travels. Making the local write consume the same file removes drift by
construction. Read-only aesthetics because two sources for one field means the next patch silently
overwrites a hand edit or the hand edit silently diverges from the DLC; the unlock is a session
switch, not a setting, so the default cannot quietly change on the savefile builder's machine.
Plan: `pes_savefile.md` "Player settings model" (completeness rule and test), "Aesthetics patch"
(new, under Interchange formats), Verification; `team_compiler.md` pipeline diagram, `savefile.rs`
comment, ID-assignment note, Post-processing (patch stage, savefile stage), catalog
(`patch_written`, `savefile_missing`), Resolved decisions; `save_editor.md` feature inventory,
Appearance tab, "Read-only aesthetics" (new), interchange list, CLI; `aesthetics_export.md`
"Player settings in exports" (completeness, template, motion, flow); `core.md` decisions table;
`GLOSSARY.md` "Aesthetics patch".

## 2026-09-12 - Player aesthetics editor - the settings forms are generated from the schema

Decision (lead, user-approved): the Player aesthetics editor's Studio panel and the Blender
sidebar panel are generated from `pes_savefile`'s `PlayerSettings` schema (group, key, type,
range, app-injected comment), which the launch manifest carries as `settings_schema`, instead of
being hand-built over "appearance, physique, strip". The ingame-face parameter group is exposed
as plain fields with the stated workflow "game face editor, then generate `settings.toml` from
the save"; a face editor with a live preview is recorded as a future step the user finds
attractive, not a requirement on this panel.
Why: `settings.toml` is now the only route by which aesthetics reach the save (aesthetics patch,
read-only editor), so a field the schema has and the form lacks is a field nobody can set; a
generated form cannot fall behind. The manifest carrying the schema keeps the Python side free of
a second copy, per the plan's "no export-format reasoning in the plugin" rule.
Plan: `player_aesthetics_editor.md` "Settings panel", launch manifest (`settings_schema`),
Blender sidebar panel step, Key decisions.

## 2026-09-12 - Refs arranger - per-match referee lists for the Fox referee hook

Decision (user): on PES 18-21 the Refs arranger produces **referee lists** for the community's
`fox_hook` runtime (`03_refmod.lua`, a trampoline on the game's referee slot-writer that forces
or remaps the id for each of a match's five positions) instead of shaping the game's slot draw:
each referee holds exactly one slot (1..N, storage-box order), lists are ordered five-position
sets of distinct referees built by hand or randomized, and the leftover slots N+1..35 of
`players.txt` are filled by weighted repetition (the previous engine) so players without the
hook still get a spread. PES 15-17 keep the pattern/flat-table editors unchanged (no hook).
PES 2020 is assumed to match 2021. Per-stadium selection is not required - per match is; an
optional `team=<id>` selector per line is allowed because the runtime exposes team ids. Lead
choices, accepted: force lists rather than a remap table (remap collapses 35 drawn ids onto N
referees and puts the same model on the pitch twice); a plain-text `ref_lists.txt` beside
`players.txt` (Lua 5.1 consumer with no parser, arranger must read it back, fixable in Notepad);
the reading script is the hook author's, the grammar is the contract; the export model lists
the file as a known root file and the compiler ignores it.
Distribution (user): the lists file goes to the streamers only, one list per match of the
matchday in schedule order; everyone else receives nothing. The hook installs with forcing off
and remap at identity, so no file means the game's own draw over the fallback-filled
`players.txt`; the companion script must `force_off()` when no file is present, since force
state persists across script reloads within a session.
Why: the hook makes the draw irrelevant, so weighting-by-repetition and the matchday
recompile loop become unnecessary where it runs; the fallback fill is the ordinary viewer's
experience, not an edge case, so it stays the rate-table engine it was.
Plan: `refs_arranger.md` (intro, "The Fox referee hook", storage box, PES 18-21 lists mode,
"The lists file", saving, CLI `randomize`, matchday rotation, verification, open questions -
position roles to be confirmed in-game); `aesthetics_export.md` referee exception (`ref_lists.txt`
known root file); `team_compiler.md` root-file validation and referee processing note;
`core.md` Phase 13; `plans/README.md` row; `GLOSSARY.md` "Referee hook", "Referee list".

## 2026-09-12 — Team creator / save editor / libs — the Team creator is a stateless wizard; tactics widgets become `libs/team_widgets`
Decision: the Team creator (`tools/team_creator`, Phase 17, post-release) is a six-step wizard that
writes an ordinary Studio aesthetics export plus an ordinary Team TOML and hands off; it owns no
format and has no reopen mode. Defaults come from `libs/aatf` (`apply_tier`, new, also behind the
Save editor's quick actions) and the Team TOML is AATF-checked before writing; every non-blocking
element is skippable. Model presets are four **starter heads** embedded in the binary (in-game
head, two cardheads, boxhead — glTF player folders plus `starter.toml`), shipped only, no user
folder. Kits are dropped in or left to the compiler's placeholder, with a button to PES Master's
online kit creator whose PNG is Fox-layout (the kit's `fox` marker is set for it); the creator
generates no kit textures. The Save editor's tactics editor, card pickers and violations list move
into `libs/team_widgets` (egui widget lib, `&mut` model in, changes out; same class as
`color_tools`), consumed by both tools. Team TOML import gains the rule "absent = untouched";
`pes_savefile` gains `shirt_name_from`; the Aesthetics export plan states explicitly that kit
textures and portraits may be any accepted image format.
Why: a reopenable creator would be a second editor over three tools' files, and a per-tool copy
of the tactics UI would drift from the Save editor's each time a version adds a field — guardrail
1 forbids the tool dependency, so the shared part goes down a level, exactly as the color picker
did. Shipped-only starters because the starter format *is* the player folder, which the Player
aesthetics editor already edits; a discovery folder would save one copy. No generated kit because
an existing designer with patterns beats a flat fill. "Starter head" not "preset"/"template"
because the community's "preset" is a tactics preset and the compiler's "template" is a CPK/bin
template. Post-release phase because its users are the next cup's new managers and it depends on
the widest set of finished pieces.
Plan: `team_creator.md` (new); `save_editor.md` (crate layout, "`libs/team_widgets`",
`apply_tier` under "Configurable AATF rules", Phase 8 build order); `core.md` (overview, crate
trees, Phase 8 note, Phase 17 new, Studio Web renumbered to 18); `pes_savefile.md` Team TOML
(absent = untouched; `shirt_name_from`); `aesthetics_export.md` Kits and Portraits (any image
format); `libs.md` pointers; `plans/README.md` rows; `GLOSSARY.md` "Roster file", "Starter head",
"Team creator", "`team_widgets`".

## 2026-09-12 — repository — license is `MIT OR Apache-2.0`, game-derived data excluded
Decision: Studio is dual-licensed `MIT OR Apache-2.0` (workspace `license` field, `LICENSE-MIT` and
`LICENSE-APACHE` at the root, contributions under the same terms). Red stays GPL-3.0. The README
states that the PES-derived data kept for interoperability (skeletons, `.bin` templates, the
kit-icon sheet) is Konami's and not covered. `just deps-check` gains a `cargo deny` license
allowlist.
Why: the suite is designed as libraries for a community that writes tools in many languages; a
GPL `pes_savefile` would be usable only from GPL consumers, cancelling the point of the crate
boundaries. Copyleft's protection — against a closed fork — addresses a threat this community's
tooling has never seen and would survive; MIT's one cost, not being able to absorb GPL code, is
moot (Red is the author's own, 4ccEditor is zlib, the rest carry no license). Apache alongside MIT
for the patent grant and ecosystem uniformity. GPL remains the right answer if "descendants must
stay free" is held as a principle rather than weighed as a risk; it was weighed.
Plan: `core.md` "License" (new, under "Distribution and updates"), "Distribution: portable .7z
bundle", "Key Decisions Summary" row; `CONTRIBUTING.md` `just deps-check`; `README.md`.

## 2026-09-13 - libs - `PesVersion` is its own leaf crate, `pes_version`
Decision (user): the PES version enum lives in `crates/libs/pes_version` (`PesVersion`, `Engine`,
`Display`/`FromStr`, serde as the two-digit number), built in Phase 1 because `studio_core`'s
common settings need it. `pes_savefile` re-exports it instead of defining it.
Why: the savefile plan placed it in `pes_savefile`, but `studio_core` (settings), `model_convert`
(skeleton tables), `kit_config` and the compilers all take it, and none may depend on the savefile
crate; putting it in `studio_core` would make every version-aware lib depend on egui. A one-enum
crate with six consumers satisfies guardrail 3 (multiple consumers); the alternative was a second
copy of one closed set.
Plan: `core.md` crate tree; `libs.md` "`pes_version`" (new); `pes_savefile.md` crate tree line.

## 2026-09-13 - dependencies - `unicode-normalization` approved; egui's font licenses allowed
Decision (user): `unicode-normalization` joins the dependency table, used by `vtree` for NFC
folding in collision detection. The `cargo deny` allowlist gains `OFL-1.1` and `Ubuntu-font-1.0`,
which egui's bundled default fonts (`epaint_default_fonts`) carry; `deny.toml` marks them as
font-only entries.
Why: the plan requires Unicode-normalized collision detection and std has no normalizer; the
crate is the ecosystem standard with no dependencies of its own. The first `just deps-check` run
rejected the fonts, which is the allowlist doing its job; both are permissive font licenses that
permit embedding, and the GUI phase decides fonts anyway (the entries go when the fonts do).
Plan: `core.md` "External Dependencies" row; `core.md` "License" allowlist sentence.

## 2026-09-13 - workspace - Phase 1 bootstrap choices
Decision: (1) workspace lints are `rust::missing_docs = warn` (the `///`-on-every-`pub` rule,
enforced) and `clippy::let_underscore_must_use = deny`, `dbg_macro`, `print_stdout`,
`print_stderr`, `todo` at `warn` (red under `-D warnings`; `studio`'s CLI output carries one
`#[expect(clippy::print_stderr)]`). (2) `just` runs recipes under PowerShell on Windows
(`set windows-shell`): Git for Windows' `sh` is not on PATH from a plain PowerShell, the fallback
`CONTRIBUTING.md` named. Verified that a clippy warning turns `just gates` red (exit 1) under
it. (3) The wasm gate's crate list is derived from `cargo metadata` by `scripts/wasm_check.py`
(every crate under `crates/libs/` plus `studio_core`, minus a named exclusion list), not
maintained by hand in the justfile. (4) `crates/tools/*` is not yet a workspace member glob
(cargo rejects a glob matching nothing); it joins with the first tool crate. (5) CI installs
`just` and `cargo-deny` through `taiki-e/install-action@v2` (prebuilt binaries) and caches with
`Swatinem/rust-cache@v2`; the `python_bindings` job is deferred to worklog step 2.18, when the
crate exists. (6) The lockfile pins egui 0.36.1 (0.36.2 was five days old at bootstrap; the
one-week rule in `CONTRIBUTING.md`).
Why: (1) a lint that is policy in prose is not policy; each entry maps to a written rule. (3) a
hand-kept list passes when a new lib is forgotten, the failure mode the gate exists to catch.
Plan: `core.md` Phase 1 will be rewritten in the present tense at converge; no other plan text
changed.

## 2026-09-13 - studio_core - `ToolContext` settings access and the settings file layout
Decision: `ToolContext` holds `Arc<Mutex<Settings>>` shared with the shell, plus the event sender
and a `ShellRequest` sender (`SwitchTool`, `Notify`, `SettingsChanged`). Tools read copies
(`common()`, `tool_settings(id)`) and write only their own table (`set_tool_settings`), which
queues `SettingsChanged`; the shell owns load, save and the path. The file is `[common]` plus one
table per tool id; `common` is a reserved id. `Settings::save` writes `<path>.tmp` then renames.
Why: the trait passes `&ToolContext`, so writes need a lock or a channel; a lock keeps reads
current within the frame, and "one lock per concept" is the style rule for exactly this.
Explicit `SettingsChanged` rather than dirty-tracking inside `Settings`, so saving is a visible
request the shell handles like the others. Atomic write because a truncated settings file on a
crash would silently reset the user to defaults on the next start.
Plan: `core.md` "Tool plugin interface" (ToolContext paragraph), "Settings menu" (file layout).

## 2026-09-13 - workspace - PowerShell stays the Windows `just` shell; Conventional Commits
Decision (user): `just` keeps PowerShell as its Windows shell rather than adding Git's `sh` to
PATH, and recipe lines gain a "no quoted arguments" rule so they stay portable between the two
shells. Commit subjects follow Conventional Commits, `type(scope): summary`, from the next commit
on; the three existing commits are left as they are. `CHANGELOG.md` is not generated from them.
Why: PowerShell already does the one thing the gates need (run a bare command, stop on non-zero
exit; verified with a planted warning), while `Git\usr\bin` on PATH shadows `find`, `sort` and
coreutils' `link` over MSVC's `link.exe`, a known way to break Windows builds. Commit tags in the
standard shape rather than an ad-hoc `[fix]` because `git log --grep`, GitHub and changelog
tools understand it without configuration; the hand-written changelog stays because members read
releases by tool, not by commit type (core plan, "Changelog and version display").
Plan: `CONTRIBUTING.md` "Testing and verification" (justfile shell rule) and "Commits" (new).

## 2026-09-13 - libs - the WESYS zlib crate is `wezlib`
Decision (user): the crate planned as `weszlib` is `wezlib`: WESYS reads as Winning Eleven
System (Winning Eleven being PES's Japanese name), so the name is WE + zlib.
Why: the plan's spelling ran the two words together; a name that parses as "WE zlib" says what
the crate is and still cannot be confused with a general zlib crate.
Plan: `core.md` crate tree and naming convention; `libs.md` mentions; worklog.

## 2026-09-13 - teams_list / fpc - `teams_list.txt` contract details; FPC kit values per version
Decision: the parse contract left open in the Team compiler plan is fixed as the `teams_list`
crate implements it: header required and preserved, columns by header position with extra
columns carried verbatim, non-folding names are placeholders, a folding name with an out-of-range
ID is an error, duplicates are errors, BOM tolerated on read only, blank lines dropped, CRLF
written. `reconcile` is infallible and reports rows it could not merge as `unresolved` instead of
failing the whole merge. `fpc::kit_values` returns the wiki's four PES 17 values for 16/17 and
19–21 and `None` for 15 and 18.
Why: the file is written by Studio and read back by Studio, so a strict parse catches corruption
early, while placeholders keep the ID space Red's list reserves (`Backup N`, invitational slots)
without pretending they are teams. A merge that fails on one conflicting row would block an update
for a problem the user has to look at anyway; a summary lets the shell show it. FPC on PES 18 is
not documented anywhere the plan cites, so the honest value is `None` until evidence appears.
Plan: `team_compiler.md` open question "`teams_list.txt` contract" (now resolved text);
`libs.md` "`libs/fpc`" (`kit_values` paragraph, with the PES 19+ verification owed to 2.13).

## 2026-09-13 - dds_convert - `block_compression` decodes as well as encodes; `texture2ddecoder` dropped
Decision: DDS block decoding uses `block_compression::decode` (BC1–BC5, BC7), the crate already
chosen for encoding; the separately listed `texture2ddecoder` crate is not added. Decoder parity
is checked against DirectXTex `texconv` (reference encoder and decoder), on synthetic fixtures it
encoded and decoded, rather than against the stadium compiler's Python decoder.
Why: one dependency for both directions, and the plan's own caveat about `texture2ddecoder`
(same name, different implementation) made its parity claim empty anyway; texconv is Microsoft's
reference for these formats, is on this machine, and produces the expected output for every mip.
Plan: `libs.md` "In-process DDS conversion" decoding bullet and the format table; `core.md`
"External Dependencies" (row removed, `block_compression` row updated).

## 2026-09-13 - dds_convert - DXT5nm layout from Konami's files; passthrough set; FTEX type 0x9; crate API
Decision: an encoded normal map is BC3 with `A = X`, `G = Y`; the remaining channels copy the
target engine's own files (pre-Fox `R = G = B = Y`, Fox `R = 255`, `B = 0`). Compressed blocks a
target reads pass through with only a container rewrite (PES 15-18: BC1/BC2/BC3; 19-21: also
BC4/BC5/BC7); uncompressed sources are always encoded. DDS/FTEX sources keep their mip count,
raster sources get a full 2x2-box chain. Fox output carries texture type `0x9` for every role.
DDS header knowledge stays in `ftex`, exposed as `ftex::dds`; the `dds_convert` API is now a code
block in the plan (`decode`, `convert`, `Converter` with `SourceHash` and `CachePolicy`).
Why: the plan's swizzle "(A=X, G=Y, R=0, B=255)" was unsourced and wrong in R/B; texconv's own
`DXT5nm` writes R=255/B=0, and measuring Konami's PES 17 normal maps (`oral_nrm`, `bibs_nrm`,
`skin_nrm`: R==B exactly, R~G within 5/6-bit quantization, A the high-precision component; mixed
partials identify A as X) and the PES 21 `dummy_nrm` (R=255, G=125, B=0, A=127) showed both
engines agree only on A and G, so each gets its own known-good layout. The legacy path never ran
(texconv rejects `DX5nm`, and today's texconv drops X for BC5 sources), so the Konami files are the
only standard. Re-encoding a DX10-header BC1/BC3 to DXT5, as the legacy compilers did, loses
quality for nothing. `0x9` on every Fox texture is what years of PES 19-21 exports shipped; the
Konami values for color textures are not proven for exported models. One DDS header parser, not
two: `ftex` already had the table for both directions.
Plan: `libs.md` "In-process DDS conversion" (DXT5nm, Passthrough, Mipmaps, FTEX texture type
bullets) and the new "`dds_convert` API" section.

## 2026-09-13 - dds_convert / ftex - encode quality is judged against the reference encoder, not a fixed tolerance; exact zlib-size chunks stored raw
Decision: the encode tests compare our encoder's error (decoded output vs. its input) with
texconv's error on the same image, per mip on the worst channel (within 8) and on the
pixel-weighted chain mean (within 1.0); a fixed "max 16, mean 3" bound is not used. In `ftex`,
a 16 KiB piece whose zlib stream is exactly the piece's size is stored raw, because every reader
of the format (ours and the legacy ones) takes `compressed == uncompressed` to mean raw.
Why: the first brief set a fixed tolerance without measuring it; the test image's 8x4 and 4x2
mips pack a 2D gradient plus a checker into one block, where texconv's own BC3 loses 104-119 in
a channel, so the bound failed for the reference encoder as much as for ours, and the sidekick
correctly refused to widen it. The raw-chunk case was met by a 16-byte tail mip (a BC5 fixture's
2x1 and 1x1 levels compress to exactly 16 bytes); the legacy writer has the same latent bug. The
sidekick's first fix invented a "raw" meaning for bit 31 of the chunk offset, which the readers
mask off without interpreting; that is a format guess about game-facing output and was reverted.
Plan: no plan edit needed: `libs.md` already says encoded output is judged by decoded content and
that `ftex` output parity is verified by converting back.

## 2026-09-13 - teams_list / kit_config - review B resolutions
Decision: `teams_list` rows keep every cell and locate `ID`/`Name` by header label in any order;
placeholder rows with a numeric id take part in the duplicate check, and an incoming team whose id
a placeholder holds replaces that placeholder (counted as added); `reconcile` applies every
incoming change first and validates ids on the final mapping, reverting only the changes behind a
collision that remains. `kit_config` has one per-version table of field maxima used by both
`validate` (`kit_value_out_of_range`, with field, value and maximum as context) and `encode`
(clamp); Name Y clamps to the game's range (16 on PES <= 20, 39 on 21), every other field to its
bit width; the PES 15 pattern rule stays byte-level (4-bit 12-13 -> 10-11) with a warning from 12
up; a wrong-typed TOML table or key is an error rather than a silent template default.
Why: the reviewer showed each as a concrete loss (`ID	Note	Name` dropped a column and turned the
team into a placeholder; an id swap between two teams was reported as two conflicts; a Backup slot
could be double-booked; `name.y = 30` on PES 20 was flagged as clamped yet emitted as 30;
`shirt = 144` produced a valid-looking config with the template's shirt). One table for finding
and clamp is the only shape in which the two cannot disagree again.
Plan: `libs.md` "`libs/teams_list`" (`file.rs`, `reconcile.rs` bullets); `kit_config_editor.md`
"Version differences" and "Version-neutral" bullets.

## 2026-09-13 - fmdl - a `model` layer between `format/` and `ops/`
Decision: `fmdl` gains `model.rs`, a semantic `Model` (bones, material instances, meshes with the
codec's `MeshVertices` and faces, mesh groups, extension headers, raw bone matrices) built from an
`FmdlFile` and written back to a fresh one; the ops operate on `Model`, not on records.
Why: the plan's tree names `format/` and `ops/` only, but mesh splitting, anti-blur and merging
reason about bones, materials and groups, none of which exist at record level; without a semantic
layer each op would re-derive the table walk. The layout `to_file` produces is the add-on writer's,
the one PES has accepted for years, since a byte-identical rebuild is `format/`'s job and a fresh
layout is what an edited model needs anyway.
Plan: `libs.md` new subsection "`fmdl::model`: the semantic layer the ops work on".

## 2026-09-13 - fmdl - anti-blur helper material gets the specular dummy; decode clears the flag
Decision: the anti-blur duplicate material's `SpecularMap_Tex_LIN` sampler gets `dummy_srm.ftex`;
`ops::antiblur::decode` clears `extensions.antiblur`.
Why: the community add-on attaches the *normal-map* dummy to the specular sampler, which reads as a
copy-paste slip next to the line above it, and the plan's shader-family table names `dummy_srm` as
the specular fallback; the game rendered either for years, so this is not a behavior a member can
see, only the intent made explicit. Leaving the flag set on decode made a second encode list
`antiblur` twice in the header.
Plan: no plan edit needed; `model_format.md` already describes the fuzzblock helper materials.

## 2026-09-13 - fmdl - mesh splitting: the add-on's algorithm with three of its slips corrected
Decision: `ops::split` follows the community add-on's encoding (a `split-mesh` child group per split
source, components matched by stored encoding and Nth occurrence) and its greedy algorithm, with
these differences: the principal axis is the true largest-eigenvalue axis (the add-on indexes the
eigenvector table with a leftover loop variable), component vertex order is deterministic (the
add-on iterates Python sets), a fragment that selects nothing force-takes one face (the add-on
could loop forever), and a preferred base bone absent from the model is skipped instead of climbed
through the skeleton table (which lives in `model_convert`; `encode` takes explicit parents for
that caller).
Why: the encoding is what existing split models carry, so it is kept exactly; the algorithm's
slips change only which faces land in which component, which the encoding makes irrelevant to
the reassembled mesh, and determinism is required by the Nth-occurrence rule.
Plan: `model_conversion.md` "Performance-critical operation: mesh splitting" already describes the
algorithm shape; no edit needed.
