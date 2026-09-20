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
Plan: no plan edit needed — `core/README.md` "Key Decisions Summary" already states it unqualified;
`aesthetics_export/object_model.md` "Design constraints" remains the worked example.

## 2026-09-07 — development plan — `color_tools` and `elevation` are built in Phase 3
Decision: `libs/color_tools` (extraction only; the picker widget waits for Phase 8) and
`libs/elevation` are Phase 3 deliverables.
Why: neither crate was assigned to any phase. Phase 3's kit processing already calls
`color_tools` for kits without `colors.txt`, and Phase 3's `output/deploy.rs` writability preflight
is the first place an access-denied PES directory must fail cleanly; building them later would
leave Phase 3 with stubs.
Plan: `core/development_plan.md#Phase 3: Processing logic` edited ("New lib crates this phase").
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
Plan: `core/development_plan.md#Development Plan` intro and Phases 1–5, 7, 12–15 edited; cross-references in
`team_compiler/README.md`, `save_editor.md`, `refs_arranger.md`, `balls_compiler.md` renumbered.

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
Plan: `core/architecture.md#Workspace guardrails` item 5 (toolchain pin rationale) and `core/architecture.md#Phase 1`
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
verifier). The "no async runtime anywhere in the workspace" rule is stated in `core/README.md`
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
Plan: `core/README.md#External Dependencies` row replaced; `core/parallelism.md#Parallelism` "Design" opens with the
runtime rule and its reasons; `core/distribution.md#Self-update` step 3 carries the streaming note.

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
Plan: `core/architecture.md#Diagnostic logging` added under "Event system"; `core/architecture.md#External Dependencies`
rows for `anyhow`, `log`, `env_logger`, `pyo3-log`; `core/development_plan.md#Phase 1` lint deliverable.
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
`team_compiler/README.md` open questions (team-name lowercasing rule, surfaced by the probe).

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
Plan: `team_compiler/pipeline.md` "Resolved decisions" (new "Teams list" bullet), settings table
(`teams_list_path`), "Path resolution", "Team ID cell", message catalog (`teams_list_read_only`);
`core/distribution.md` "Distribution" (bundle contents), "Data location" (data-dir cargo), "Self-update" step 5,
Phase 16 bundling; `aesthetics_export/README.md` crate tree comment; `GLOSSARY.md` "Team ID";
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
Plan: `team_compiler/README.md` Reader step 2 (fold rule, first non-empty token), "Export identity
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
Plan: `core/gui.md` "Status bar" (new, under GUI Design), "Shell layout" (diagram, panel order),
"Tool plugin interface" (`activity()`, `ctx.notify`), `studio_core` inventory and module tree
(`status.rs`, `shell/status_bar.rs`), "Settings menu" (best-effort save), sidebar auto-switch
(notice), "Crate structure" presentation sentence, Phases 1 and 8; "run strip" rename across
`core/README.md`, `team_compiler/README.md`, `balls_compiler.md`, `match_tracker/README.md`.

## 2026-09-11 — save editor — teams-list refresh from the savefile lives in the Save editor, as "Export teams list"
Decision (user): supersedes the location in the previous "savefile is a teams-list source" entry.
The action is the Save editor's **Export teams list** (feature table, "Database operations") and
`studio save-editor export-teams-list <EDIT> [--yes]`; the Team compiler's settings button and
`teams-list import-savefile` are withdrawn.
Why: the Save editor owns the open savefile; from its point of view the operation is an export of
the save's team table, and the compiler is only a consumer of the resulting file. Same
reconciliation, same review, same read-only handling — only the owner changed.
Plan: `save_editor.md` "Database operations" row, "CLI"; `team_compiler/pipeline.md` "Teams list" decision
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
Plan: `libs/teams_list.md` "`libs/teams_list`" (new) and intro list; `aesthetics_export/README.md` ownership
paragraph and crate tree (`teams_list.rs` removed, `identity.rs` re-described); `core/README.md`
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
Plan: `aesthetics_export/player_folders.md` "Root files" (diagram, "Logo" bullet), `ValidatedAestheticsExport`
(`logo: Option<LogoFiles>`); `team_compiler/README.md` validation categories, "Processing" Logo step,
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
Plan: `export_upgrader.md` step 7, "Verification" fixtures; `team_compiler/pipeline.md` "Processing" Logo
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
Plan: `aesthetics_export/player_folders.md` "Kits" (grammar, `all/` rule, diagram), object model (`KitsFolder`,
`KitFolder`, `KitTexture`), `kits.rs` comment; `team_compiler/README.md` validation, "Processing" Kits,
message catalog (`kit_folder_invalid` widened, `kit_slot_duplicate`, `kit_textures_inherited`,
`kit_all_file_ignored`, `kit_all_unused`), kit cells, test list; `kit_config_editor.md` derivation
input and tabs; `export_upgrader.md` step 5 (bare slots, hoist); `core/README.md` decisions table;
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
Plan: `aesthetics_export/player_folders.md` "Kits" (empty-folder paragraph, diagram); `team_compiler/README.md`
validation, "Processing" Kits (template fill), "Kit colors fallback", embedded templates list,
message catalog (`kit_placeholder`, `kit_colors_missing` reworded), test list; `core/README.md`
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
`scripts/provenance/kit_uv/`.
Plan: `aesthetics_export/README.md` object model (`KitFolder.layout`, `KitLayout`), "Kit layout marker"
paragraph, diagram; `team_compiler/README.md` validation, "Processing" Kits (layout conversion, mask fill
scope), message catalog (`kit_layout_conflict`, `kit_layout_converted`), test list;
`export_upgrader.md` step 5; `core/README.md` decisions table; `GLOSSARY.md` "Kit layout".

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
Plan: `team_compiler/pipeline.md` "Processing" Kits (mask/srm paragraph), message catalog
(`kit_texture_not_used`), test list; `aesthetics_export/README.md` Kits diagram.

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
Plan: `model_conversion/gltf.md` "Blender integration" ("What users call it"); `GLOSSARY.md` "Global
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
Plan: `core/README.md` `StudioTool` trait, `studio_core` inventory and module tree (`help/`,
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
Plan: `core/distribution.md` "Changelog and version display" (new, under Distribution and updates), "Status
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
Plan: `core/gui.md` "Window title" (new, after "Status bar"), `activity()` doc comment, module tree
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
Plan: `CONTRIBUTING.md` "Testing and verification" (gates block and justfile rules), `core/README.md`
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
Plan: `core/README.md` Phase 1 workspace bullet. `CONTRIBUTING.md` already forbids recipes from
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
findings; `aesthetics_export/object_model.md` "Common model links and model merging"; `team_compiler/README.md`
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
Plan: `team_compiler/pipeline.md` "User-supplied `face.xml`" (new, under the XML/MTL content checks), the
XML rows of that catalog table, step 4, test list; `aesthetics_export/README.md` "Model names: a free
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
Plan: `pes_savefile/model.md` "Player settings model" (completeness rule and test), "Aesthetics patch"
(new, under Interchange formats), Verification; `team_compiler/README.md` pipeline diagram, `savefile.rs`
comment, ID-assignment note, Post-processing (patch stage, savefile stage), catalog
(`patch_written`, `savefile_missing`), Resolved decisions; `save_editor.md` feature inventory,
Appearance tab, "Read-only aesthetics" (new), interchange list, CLI; `aesthetics_export/README.md`
"Player settings in exports" (completeness, template, motion, flow); `core/README.md` decisions table;
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
position roles to be confirmed in-game); `aesthetics_export/README.md` referee exception (`ref_lists.txt`
known root file); `team_compiler/README.md` root-file validation and referee processing note;
`core/README.md` Phase 13; `plans/README.md` row; `GLOSSARY.md` "Referee hook", "Referee list".

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
`apply_tier` under "Configurable AATF rules", Phase 8 build order); `core/README.md` (overview, crate
trees, Phase 8 note, Phase 17 new, Studio Web renumbered to 18); `pes_savefile/README.md` Team TOML
(absent = untouched; `shirt_name_from`); `aesthetics_export/README.md` Kits and Portraits (any image
format); `libs/README.md` pointers; `plans/README.md` rows; `GLOSSARY.md` "Roster file", "Starter head",
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
Plan: `core/distribution.md` "License" (new, under "Distribution and updates"), "Distribution: portable .7z
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
Plan: `core/README.md` crate tree; `libs/README.md` "`pes_version`" (new); `pes_savefile/README.md` crate tree line.

## 2026-09-13 - dependencies - `unicode-normalization` approved; egui's font licenses allowed
Decision (user): `unicode-normalization` joins the dependency table, used by `vtree` for NFC
folding in collision detection. The `cargo deny` allowlist gains `OFL-1.1` and `Ubuntu-font-1.0`,
which egui's bundled default fonts (`epaint_default_fonts`) carry; `deny.toml` marks them as
font-only entries.
Why: the plan requires Unicode-normalized collision detection and std has no normalizer; the
crate is the ecosystem standard with no dependencies of its own. The first `just deps-check` run
rejected the fonts, which is the allowlist doing its job; both are permissive font licenses that
permit embedding, and the GUI phase decides fonts anyway (the entries go when the fonts do).
Plan: `core/README.md` "External Dependencies" row; `core/distribution.md` "License" allowlist sentence.

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
Plan: `core/README.md` Phase 1 will be rewritten in the present tense at converge; no other plan text
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
Plan: `core/architecture.md` "Tool plugin interface" (ToolContext paragraph), "Settings menu" (file layout).

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
Plan: `core/README.md` crate tree and naming convention; `libs/README.md` mentions; worklog.

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
Plan: `team_compiler/README.md` open question "`teams_list.txt` contract" (now resolved text);
`libs/fpc.md` "`libs/fpc`" (`kit_values` paragraph, with the PES 19+ verification owed to 2.13).

## 2026-09-13 - dds_convert - `block_compression` decodes as well as encodes; `texture2ddecoder` dropped
Decision: DDS block decoding uses `block_compression::decode` (BC1–BC5, BC7), the crate already
chosen for encoding; the separately listed `texture2ddecoder` crate is not added. Decoder parity
is checked against DirectXTex `texconv` (reference encoder and decoder), on synthetic fixtures it
encoded and decoded, rather than against the stadium compiler's Python decoder.
Why: one dependency for both directions, and the plan's own caveat about `texture2ddecoder`
(same name, different implementation) made its parity claim empty anyway; texconv is Microsoft's
reference for these formats, is on this machine, and produces the expected output for every mip.
Plan: `libs/dds_convert.md` "In-process DDS conversion" decoding bullet and the format table; `core/README.md`
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
Plan: `libs/dds_convert.md` "In-process DDS conversion" (DXT5nm, Passthrough, Mipmaps, FTEX texture type
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
Plan: no plan edit needed: `libs/README.md` already says encoded output is judged by decoded content and
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
Plan: `libs/teams_list.md` "`libs/teams_list`" (`file.rs`, `reconcile.rs` bullets); `kit_config_editor.md`
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
Plan: `libs/format_crates.md` new subsection "`fmdl::model`: the semantic layer the ops work on".

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
Plan: `model_conversion/conversion.md` "Performance-critical operation: mesh splitting" already describes the
algorithm shape; no edit needed.

## 2026-09-13 - pes_model - `roxmltree` for XML reading; XML written by hand
Decision: `roxmltree` (read-only XML tree, MIT OR Apache-2.0, pure Rust) is the XML reader for
`.mtl`, `face.xml` and `face_diff.xml`; those files are written by hand-rolled serializers in the
crates that own them.
Why: the plan's dependency table named no XML crate although three game-facing XML formats need
one. The shapes are tiny and fixed (a material set is a few elements with attributes), so a DOM
writer buys nothing while a hand writer can reproduce Konami's whitespace exactly (`.mtl` byte
parity becomes testable); `quick-xml` would add a streaming API and a serde surface nobody needs.
Plan: `core/README.md` "External Dependencies" row added.

## 2026-09-13 - pes_model - byte identity at the container, a fresh layout from the typed layer
Decision: `ModelContainer` (eleven opaque sections in file order) is the byte-identical `format/`
layer; `PreFoxModel::write` produces the add-on's layout and is tested for semantic equality.
Konami annotation types 1/10 and section-3 records are carried; sections 8/9/10 with content are
a read error (`Unsupported`), not a warning.
Why: the plan asks for a byte-identical `format/` round trip without saying at which level; the
`.model` pointer graph with Konami's 8/16-byte alignment gaps means a typed layer can only be
byte-identical by carrying every offset, which is the container again with more fields. The
reference parser warns and proceeds on cloth/locator models, but a warning that ends in a rewrite
dropping the referenced section is a silent corruption, and no player part uses those sections.
Plan: `libs/format_crates.md` "`pes_model::format`: the `.model` container and its sections" added (layout as
measured by `scripts/provenance/pes_model/model_census.py` on the six fixtures).

## 2026-09-13 - pes_model - the typed writer always emits header version 19
Decision: `PreFoxModel::write` puts 19 in the header whatever `version` the file declared; the
field stays on the struct as read-side information.
Why: the writer emits the version-19 record sizes (24-byte mesh records, three geometry extras,
16-byte locator records), and the reviewer pointed out that the round trip of the version-17
shadow fixture only proved our reader accepts a version-17 word over that layout, not that PES
does. The version-19 word over the version-19 layout is what the community converter shipped to
PES 16 for years; the hybrid has no evidence. Carrying the word would have made the file look
faithful while being untested.
Plan: `libs/format_crates.md` "`pes_model::format`: the `.model` container and its sections" edited.

## 2026-09-13 - pes_model - mesh splitting on `.model`: the fmdl port, four departures from the reference importer
Decision: `ops::split` ports `fmdl::ops::split` with the `.model` limits (64/60 bones, 65535/63000
vertices, 21845/20000 faces) and the per-mesh `Split-Mesh: N` header; the caller supplies the
bone hierarchy (`encode(model, parents)`) since the format stores none. Departures from the
reference importer: the combined mesh drops the `Split-Mesh` header (the reference left it on, so
a re-export added a second one); component and combined bounds are recomputed from their own
vertices (the reference copied the source box); a component's bone group is sorted by model bone
index (the reference iterated a set); a mesh with LOD levels refuses to split (the reference had
dropped LODs at import). The fragment sort axis is the fmdl port's principal axis oriented from
the base bone, not the reference's bone x-axis; as for fmdl, that changes which faces land where,
which the encoding makes irrelevant to the reassembled mesh.
Why: each departure removes a slip or an information loss; none changes what PES renders.
Plan: `model_conversion/conversion.md` "Performance-critical operation: mesh splitting" describes the shape;
`libs/format_crates.md` "`pes_model::model`" already carries the LOD rule; no further edit.

## 2026-09-13 - pes_model - `check` covers the `.mtl` too, with two codes the plan did not list
Decision: `pes_model::check` has `check(model)`, `check_materials(set)` and `check_bundle(model,
set)`; the `.mtl` rules are the Team compiler plan's (`mtl_material_duplicate`, `mtl_state_invalid`,
`mtl_blendmode_nonzero`, `mtl_state_missing`, `mtl_state_nonrecommended` as `alphablend 1 + zwrite
1`) plus `mtl_state_unknown` (Info, a state name outside the seven; `shadowcaster` appears in 18
Konami files) and `model_material_undefined` (Error, a mesh's material name the sibling `.mtl` does
not define).
Why: the plan lists the `.mtl` checks under the Team compiler, but the lib owns the grammar and
returns findings, not messages (the compiler maps codes in Phase 3); the two added codes fall out
of the census (an unknown state is a fact worth surfacing at Info) and of the bundle (an undefined
material is the one cross-file error the pair can have). Texture existence stays with the compiler.
Plan: `team_compiler/pipeline.md` "XML/MTL content checks" gains the two rows when Phase 3 writes its
message catalog; no edit now (the codes are lib-side until then).

## 2026-09-13 - pes_model / model_convert - the pre-Fox multi-part merge is native, not an IR operation
Decision: `pes_model::ops::merge(parts: &[(&Model, &MaterialSet)]) -> Result<(Model,
MaterialSet), MergeError>` is the pre-Fox counterpart of `fmdl::ops::merge`; the `ingame_face`
boots merge and the link-combined case call it. `model_convert::merge_ir_parts` stays specified
but deferred, with no planned caller. Bone matrices are compared exactly, as `fmdl` compares
positions; a tolerance is decided with `model_convert` from measured matrices if glTF-authored
pre-Fox parts turn out to need one. User decision (the plan contradicted itself: `libs/README.md`'s
crate tree listed `pes_model/ops/merge.rs` while `model_conversion/README.md` routed the merge through
the IR).
Why: an IR round trip for a same-format operation is what the "format-native ops" rule exists to
avoid, and the IR route bought nothing: the `.mtl` merge is a material-name merge either way, and
the native op is what the Blender bindings can expose. An unmeasured tolerance would be the same
mistake as the `dds_convert` encode bounds.
Plan: `libs/format_crates.md` gains "`pes_model::ops::merge`"; `model_conversion/gltf.md` "IR part merge", the crate
tree, "Conversion routing" and the retargeting paragraph, `aesthetics_export/README.md` (four mentions),
`team_compiler/README.md` (pipeline step) and `AGENTS.md` ("Two engines, one IR") edited to match.

## 2026-09-13 - pes_model - merged bones agree within a measured 1e-4, not exactly
Decision: `pes_model::ops::merge` treats two bones of one name as the same bone when every one
of the twelve matrix components differs by less than `1e-4` (absolute), and keeps the first
part's matrix. Supersedes the "compared exactly" rule written earlier the same day.
Why: the exact rule could not merge Konami's own kit parts: `modD_cap` and a collar share four
shoulder bones whose matrices differ by float noise (found by the sidekick when the briefed LOD
test failed). Measured over all 2606 Konami files (`scripts/provenance/fmdl_bone_matrix/`,
numbers in `libs/README.md`): shared-skeleton noise tops out at `3.6e-5`, real bind-pose
differences start at `2.0e-4` and run to `0.5`, so `1e-4` sits in the gap on a log scale. An
unmeasured tolerance would have been the `dds_convert` mistake again; this one is measured.
Whether `fmdl::ops::merge`'s exact position comparison has the same problem on FMDL parts is a
converge question for 2.20 (no two Konami FMDL parts sharing a bone are in the fixtures).
Plan: `libs/format_crates.md` "`pes_model::ops::merge`" bones rule rewritten with the measurement;
`aesthetics_export/player_folders.md` "SKL pairing" sentence updated; `resources/prefox_model_format.md`
section 5 gains the finding for the plugin author.

## 2026-09-13 - fox2 - CityHash64 ported, not a crate; typed file keeps the string table; goldens from the reference
Decision: `libs/fox2` ports the C# tool's CityHash64 1.0.3 variant (about 150 lines) instead of
depending on the `cityhash` crate the plan had pencilled in; `Fox2File` keeps the string table as
read so `write(read(x)) == x` holds on Konami files (whose table order varies by file), and
`from_xml` rebuilds it in the reference's traversal order; the compiled goldens are the
reference's output with its trailing buffer slack removed.
Why: the hash must match one specific old CityHash variant byte for byte, and a crate's version
cannot be pinned to it, while a direct port is verified by a 46-string golden and by the 100-odd
hash/literal pairs the fixtures' own tables carry. The reference cannot round-trip Konami files
(it reorders the table) and its writer returns an over-allocated buffer (896 bytes of content in
a 1228-byte result), so the byte-identity standard is our own reader/writer on Konami's files
plus equality with the reference's logical output on compiled ones, as for `fmdl`.
Plan: `libs/fox2.md` gains "`libs/fox2`: Fox Engine entity files" (layout census over 87 files, types,
XML rules); `core/README.md` "External Dependencies" row for `cityhash` replaced.

## 2026-09-13 - fox2 - the XML form carries `classHash`/`nameHash` for unresolved names
Decision: `to_xml` writes an unresolved class or property name as the reference's `class=""` /
`name=""` plus a `classHash` / `nameHash` attribute, and `from_xml` prefers the hash attribute;
tab, LF and CR in attributes are written as character references; a `bool` value must read
`true`, `false` or empty. Reviewer findings (checkpoint b).
Why: the reference's XML is lossy on unresolved names (they recompile as the empty-string hash),
which the Stadium compiler's decompile-edit-compile ID rewrite would inherit for any Konami name
its dictionary lacks; an attribute the reference's reader ignores keeps its XML readable by it
while making ours lossless. Raw whitespace in attributes is normalized to spaces by any XML
reader, changing the string and its hash. The strict bool turns a typo in generated XML into an
error instead of a silent `false`.
Plan: `libs/fox2.md` "`libs/fox2`" last paragraph rewritten.

## 2026-09-13 - archives - `sevenz-rust2` and `zip`, both without default features; one `Archive<R: Read + Seek>`
Decision: `libs/archives` depends on `sevenz-rust2` 0.22.2 (`default-features = false`: LZMA,
LZMA2 and BCJ decoding) and `zip` 8.6.0 (`default-features = false`, `deflate`), and exposes one
`Archive<R: Read + Seek>` with `zip`/`seven_z` constructors, `entries()` and `read()`, plus a
disk-path `open` behind `cfg(not(target_arch = "wasm32"))`. A 7z is decompressed whole on the
first `read`; a zip entry by entry.
Why: the plan named `sevenz-rust`, which RUSTSEC-2026-0246 marks unmaintained (repository
deleted) in favor of `sevenz-rust2`; the plan named no zip crate and `zip` is the one everyone
uses. Default features would pull compression, AES, bzip2, PPMd, zstd and time crates into a
crate that only reads what 7-Zip, Explorer and PowerShell write. Generic over `Read + Seek` is the
std-trait generic the style rules allow and is what keeps the crate `wasm32`-clean (checked).
Plan: `libs/archives.md` gains "`libs/archives`"; `core/README.md` "External Dependencies" rows updated.

## 2026-09-13 - color_tools - extraction regions from the template sheet, thresholds from the exports, a trim fallback
Decision: the shirt and shorts regions are the PES 19 colored template's zones inset (shirt
x 0.36-0.64, y 0.05-0.88; shorts x 0.04-0.29 and 0.71-0.96, y 0.61-0.90); clustering is 5-bit
quantization with a greedy merge at RGB distance 24; "distinct" is RGB distance 60; a second
shirt cluster counts as a two-tone color at 25% and as a trim color at 10%; color 2 falls through
shirt-second, shorts, shirt-trim, shorts-second before settling for a color identical to color 1.
The extraction takes RGBA pixels, so `color_tools` has no image dependency.
Why: the plan left the rectangles and thresholds to calibration. The template sheet gives the
zones exactly, and a harness over 134 real kit texture/config pairs showed the declared config
colors are not a ground truth (only 55 declared shirt colors occur in the shirt at all; managers
leave template colors or pick accents), so the thresholds come from the one measurable split in
that data (two-tone second colors at 28-49% of the region, trims at 0-19%) and the regions from
visual swatch sheets over every kit. The trim fallback is a plan gap: with the plan's shorts-only
fallback a black kit with black shorts and gold trim would get two identical menu colors, where
every manager who bothered declared the trim. RGB distance rather than a Lab metric keeps one
notion of distance in the crate; navy against black is 64, the threshold's anchor.
Plan: `libs/color_tools.md` "Dominant kit-color extraction" rewritten with the values and the evidence.

## 2026-09-13 - elevation - `windows` and `libc` as target-gated dependencies; the surface pinned in the plan
Decision: `libs/elevation` exposes `is_elevated`, `relaunch_elevated(program, args)` and
`is_access_denied`, with `windows` 0.62.2 (four `Win32_*` features) under `cfg(windows)` and
`libc` 0.2.189 under `cfg(unix)`; every other target (wasm32) compiles to "not elevated,
unsupported". The `requireAdministrator` manifest stays with the `studio` binary.
Why: the plan named the behavior but no crate for the POSIX euid check and no API shape; `libc`
is the std-adjacent answer and `windows` is already the approved Win32 crate. Target-gating keeps
the crate on the `wasm32` gate like every other lib. Argument quoting for the relaunch is tested
against `CommandLineToArgvW` itself rather than against our reading of its rules.
Plan: `libs/elevation.md` "`libs/elevation`" gains the surface block and the test list; `core/README.md`
"External Dependencies" rows for `windows` and `libc`.

## 2026-09-13 - model_convert - IR shapes settled against the format crates
Decision: the IR's vertices are a struct of arrays (`Vertices`, one `Vec` per attribute) rather
than a `Vec<Vertex>`; `Bone.children` is dropped; `Mesh` carries no `alpha_flags`,
`shadow_flags`, anti-blur or `split_group_id` fields; `extension_headers` are `BTreeSet<String>`
of raw header lines; `Bone.matrix` is the crate's own `Affine` (3x4 row-major, `affine.rs`) and
`nalgebra` is not used; a `.model` import drops Konami's lower LODs, tags, editor items and
order word, reported through `loss.rs`; the template display positions wait for Phase 7.
Why: both format crates already store vertices as columns, so a conversion is a column copy and a
100k-vertex model allocates no per-vertex `Vec`s (speed is the primary goal); `children` is a
second copy of `parent` to keep in step through folds and prunes; the per-mesh Fox flags are
lifted to the material by the plan's own "Engine mapping" rule, and the importers call the format
crates' split and anti-blur *decode*, so an imported mesh is whole and carries no split identity;
the format crates type every known header already, so the IR only needs the leftovers; the only
matrix work in the suite is bone transforms, four functions that do not justify a generic linear
algebra API; no 4cc export carries LODs or Konami tags (the add-on writes none).
Plan: `model_conversion/ir.md` "IR struct" rewritten with the shapes; "Crate layout" gains
`affine.rs` and loses `ir/skeleton.rs`.

## 2026-09-13 - model_convert - skeletons parsed from the embedded .skl files, not generated source
Decision: `skeletons/` embeds `resources/skeletons/pes*/*.skl` with `include_bytes!` and parses
them once at first use (`LazyLock`) through `fmdl::SklFile`; `PesBone`/`Skeleton` are owned
`String`/`Vec` data with a binary search by name; no `phf`, no generated `data.rs`, no build
script. `skeletons/` is the one module outside `formats/` allowed to import `fmdl` (the SKL codec
only). The render hierarchy (`render_parents.rs`, 140 names from `PesSkeletonData`) and the fold
table (`fold.rs`) are the only hand-maintained tables, names only.
Why: the plan's generated `data.rs` is a 250 KB file of literals plus a generator plus a drift
test, for numbers the SKL codec already reads byte-exactly (tested on three Konami files); the
`.skl` bytes must be embedded anyway for Fox template injection. A `phf::Map` over 175 names buys
nothing over a sorted slice. The render hierarchy is not in any game file (the SKL parent column
makes 41 to 82 of each body's bones roots), so it stays a transcription, of names.
Plan: `model_conversion/conversion.md` "Skeleton data" rewritten; "Crate layout" placement rules edited.

## 2026-09-13 - model_convert - hand bones are `skh_`, not `skf_`; fold table follows chains
Decision: hand auto-split identifies hand-exclusive bones by the `skh_` prefix. The fold table is
one name -> name table for every version, followed as a chain when the target lacks the entry's
target too, with a unit test that every entry resolves on every version; entries beyond the two
legacy tables are name-based and marked unverified.
Why: the games' own `hand_l.skl` holds 19 `skh_` bones and `face.skl` 33 `skf_` bones (measured on
PES 18, 19, 21); the plan's `skf_` would have split faces at the jaw. A per-version fold table
would repeat the same entries six times, and the natural targets already chain
(`dsk_upperarm_long_l` -> `dsk_upperarm_l`, which PES15 also lacks).
Plan: `model_conversion/hand_split.md` "Hand auto-split" (detection, split, where it lives), "Skeleton
retargeting and bone conformance" step 2; `aesthetics_export/README.md` pipeline step 0.

## 2026-09-13 - model_convert - `convert` by value; the Fox bundle carries its companion SKL
Decision: `convert(bundle, target) -> Result<Converted, ConvertError>` takes and returns
`NativeModelBundle` by value; `NativeModelBundle::Fox { model: fmdl::Model, skl: Option<SklFile> }`
and `PreFox { model: pes_model::model::Model, mtl: MaterialSet }` sit at the format crates'
semantic layer (`Model`), not the file layer (`FmdlFile`/`PreFoxModel`).
Why: by value makes "both members or neither" a property of the type instead of a comment on a
`&mut self` method; the semantic layer is where the format crates' ops (split, anti-blur, vertex
encoding) already work, so the importers call them without a second decode; the SKL is the bind
pose source for an FMDL and belongs with it.
Plan: `model_conversion/ir.md` "Conversion routing" rewritten.

## 2026-09-14 - model_convert - the vertex-loop encoding is read from order on import, applied on export
Decision: `fmdl_to_ir` calls no vertex-loop decode (the IR keeps the vertex order the owner map is
read from); `ir_to_fmdl` runs `fmdl::ops::vertex_enc::encode_model` as the plan's table says, and
same-format round-trip tests compare the export against the decoded input with the same encoders
applied, not against the input's vertex order.
Why: the Rust decode returns an owner map and changes nothing, so a call on import would discard
its result; the encode is not a no-op on Konami files (highneck: 198 of 204 vertices reordered,
vertex multiset and face corner-tuples unchanged), and the legacy converters reorder the same way,
so a byte-order comparison would fail on correct output.
Plan: `model_conversion/ir.md` "Extension algorithms" gains the paragraph.

## 2026-09-14 - model_convert - `.model` import reorders bones parent-first; direct render parent only
Decision: `model_to_ir` moves a bone to right after its render parent when the file lists it
earlier (44 of 2606 Konami `.model` files, all `dsk_forearm_*` before `dsk_forearm_t_*`) and
remaps the bone groups; `parent` is the render parent only when present in the model, never a
further ancestor. Tangent `w` is 1.0 on import (the 16->21 converter's value); reading the
handedness off the bitangent is an open question. Fields the IR lacks (LODs, tags, editor data,
`order`, `flags`) become one `native_field_dropped` finding each when non-default. `.mtl`
definitions the model does not bind are not carried.
Why: the IR's parent-first invariant is what the SKL and FMDL writers need and what a one-pass
consumer relies on; relaxing it for 1.7% of Konami files would push order handling into every
consumer. Climbing to a present ancestor would give `skf_brow_*` a parent in 914 files where the
legacy converters made them roots, a behavior change with no evidence of need. Silent field
drops are what the plan's "losses explicit, not accidental" rule forbids.
Plan: `model_conversion/ir.md` "Material sources per format" gains the `.model`-pair list.

## 2026-09-14 - model_convert - review (b) rulings: loop owners read unflagged, hand split over topological vertices
Decision: the exporters recover vertex-loop owners with the format crates' per-mesh unflagged
`vertex_enc::decode` on the IR order, never the flag-gated `decode_model` (the IR carries no
extension flag). The hand split treats entries sharing position, bone indices and weights as one
vertex for selection and growth. Losses the reviewer found unreported become `native_field_dropped`
findings (`bone_matrices`, a disagreeing SKL parent, non-unit normal/tangent `w` on `.model`
export); flag-split material names are made unique; weights must be finite and in 0..=1 to
validate; `remap_bone_group` never indexes with an unweighted slot. Rejected: adding the
`EnvironmentMap` fallback for `Metal` here; the format plan assigns the template cubemap to the
compiler's texture step (worklog issue 5 for Phase 4).
Why: identity owners re-encode an add-on file's loops as distinct vertices, a same-format loss the
plan's lossless round trip forbids; a run that satisfies the convention is a set of loops whether or
not the file declared the extension. Blender's Select More works on vertices, and the native
formats store loops, so a UV seam at the wrist would otherwise stop the growth. The audience pair's
SKL and FMDL parents agree on all 16 bones (measured), so the FMDL parent stays the IR's.
Plan: `model_conversion/ir.md` "Extension algorithms" (owner recovery), "Hand auto-split" step 2.

## 2026-09-14 - pes_savefile - one `SaveContainer` struct, salt as a parameter, partial real-file fixtures
Decision: the container layer is one `SaveContainer` struct with a `Scheme` enum (`Pes15` |
`Keyed(MasterKey)`), not the plan's "Container trait"; `to_bytes` takes the 320-byte salt as a
parameter and `file.rs` derives one with `sha2` from the payload and the clock (no RNG crate).
Fixtures are per-version slices of real saves (the prefix through the description, the first 4096
encrypted payload bytes) plus the whole decrypted payload zlib-compressed and inflated by tests
with `flate2` as a dev-dependency; no whole save is committed. The PES 16 myClub key joins
detection and reports `Pes16`; PES 20 (no save on the machine) is transcribed and marked untested.
Why: a trait with two implementors selected once at load is an enum with extra ceremony
(`CONTRIBUTING.md`, "enum + match"). The salt has no security role and a parameter is what makes
`decrypt(to_bytes(c, salt)) == c` and the byte-identity tests deterministic; `getrandom` would be a
new dependency for 320 bytes nobody checks. A save is 5-11 MB and incompressible encrypted, and its
serial section is the writing account's Windows SID, so committing whole saves would cost ~40 MB and
leak an identifier; the slices prove every layer of the container on real bytes and the inflated
payload is the complete real input every higher layer needs.
Plan: `pes_savefile/container.md` "Container format and crypto" (header layout as measured, "Container API",
"Container fixtures"); "Savefile discovery" table corrected from the same machine (PES 18 uses the
flat layout; 18/19 folder names verified).

## 2026-09-14 - pes_savefile - schema tables derived by a committed script, not transcribed; text field rules
Decision: `schema/fields.rs` and `schema/pes15.rs`..`pes20.rs` are generated by
`scripts/derive_savefile_schema.py`, which interprets the reference editor's read walks over a
symbolic byte array and emits `FieldSpec`/`ArraySpec`/`TextSpec` rows plus the three regular
tactics layouts (`PresetSpec`, `FormationLayout`, `InstructionLayout`) after checking their
regularity; the script is committed (it needs the reference source as an argument) and the tables
it wrote are committed as source. The plan's claim that PES 18 shares the 17/19 record layout is
withdrawn: each version keeps its own table. The reference's PES 19 `b_edit_stadium` read (masked
to zero) is dropped from the table rather than "fixed". Model text rules: `name` is UTF-8, `shirt_name`
is single-byte text decoded as U+00XX per byte, and writing a text leaves the field's bytes after
the NUL untouched. Closed-set numbers (positions, playing styles) stay `u8` in the codec model until
the step that converts them needs an enum.
Why: hand transcription of ~1500 rows is where a wrong offset passes every round-trip test (the
plan's own "the LLM transcribes" idea); the interpreter derives them once and the plausibility
census over every real save (every 7-bit ability in 40..99 under its own version's table, not the
neighbours') is what shows they name the right bits. The PES 18 walk is hand-shifted, so only an
interpreter could compare it with 17's. Two real shirt names carry 0xFC/0xF0 (not UTF-8) and 437 of
PES 16's names carry stale bytes after the NUL, so `String`-and-zero-fill would fail the lossless
round trip. A guessed `b_edit_stadium` bit would be a schema fact with no reference behind it.
Plan: `pes_savefile/codec.md` "Schema-driven save codec" rewritten (types, derivation, checks, record
sizes, reference readings flagged); "Payload layout per version" era note corrected.

## 2026-09-14 - pes_savefile - colour codes are eight bytes, not eight hex digits; `\x11d` resets
Decision: `display_name` strips `\x11c` plus the next eight bytes whatever they are, and the
two-byte `\x11d`; the plan's "8 hex digits" rule is corrected. `EditFile` and discovery get
their signatures in the plan (`from_bytes`/`to_bytes(salt)` pure, `load`/`save` native; discovery
pure over a Documents root with a native wrapper over `directories`).
Why: measured over 346 decorated names in the PES 16/19/21 saves: nine names of one PES 21 team
carry `\x11ca000c8ON<name>` and are complete names without the `ON`, so the game takes eight bytes
without checking hex, and two names carry `\x11d` mid-string as a reset; a hex-only rule would
leave control bytes in the compiler's folder names for those eleven players. Pure cores with
native wrappers are what keeps the crate wasm32-checkable (guardrail 6) and the tests free of the
real Documents folder.
Plan: `pes_savefile/codec.md` "Whole-file API" (API block, measured code rule), "Player settings model"
paragraph, "Savefile discovery" (API block).

## 2026-09-15 - development plan - Phase 3 opens with a tracer bullet
Decision (user): Phase 3's first code step compiles one minimal Studio-format export (one Fox
face, one kit) to a CPK through the real Phase 2 crates and compares it with Red's output for the
same source, before the tool skeleton (`settings.rs`, `cli.rs`, `messages.rs`, `view/`,
`aesthetics_export`, `pipeline`) is built. The fixture is an old-layout export migrated by hand,
reused later as the Export upgrader's input/expected pair.
Why: the phase plan was horizontal (all libs, then the skeleton, then processing, then output
comparison in Phase 4), so fifteen lib crates' `pub` APIs would meet their first consumer only
after the skeleton was already built on them; a shape wrong across crates would then be fixed
through the tool rather than in the crate. A thin end-to-end slice first tests the composition
while a fix is local, and seeds the parity harness instead of deferring it. Prompted by the
"vertical slices, not horizontal plans" argument in humanlayer's "Why Software Factories Fail".
Plan: `core/development_plan.md#Phase 3: Team compiler skeleton` edited (first bullet, Verification).

## 2026-09-15 - development plan - Phase 3 closes with a minimal `studio` shell
Decision (user): Phase 3's last code step is a shell slice: `studio_core`'s window, sidebar and
selected-tool view, the `studio` binary with `team_compiler` registered, and a Team compiler view
of settings, a run button and a plain `PipelineEvent` log. Phase 8 completes the shell instead of
starting it; its list is otherwise unchanged.
Why: `StudioTool`, the event channels and the render-only `view/` rule would otherwise meet
their first real tool only in Phase 8, after five tool crates had been written to them; one real
tool on the shell early tests the seam while a change to it touches one crate. It also makes the
Phase 10 parallelism note ("once the shell exists") true five phases sooner. Same argument as the
tracer bullet, applied to the GUI seam instead of the lib seam.
Plan: `core/development_plan.md#Phase 3: Team compiler skeleton` (last bullet, Verification) and
`core/development_plan.md#Phase 8: GUI` (opening note, first bullet) edited.

## 2026-09-19 - aatf - one self-contained Rhai rules file; TOML+CEL and the two-file split rejected
Decision (user): the AATF ruleset is a single `.rhai` file with a `const CFG = #{ ... }` parameter
map at the top (tiers as map keys, `CFG.tiers` ordered for the quick actions) and the check logic
as functions below it; the host requires `check_team`, `tier_values` and `card_limits`, validates
`CFG`'s required keys and the tier/table consistency on load, and registers a listed accessor set
(players, tactics presets, formations, fluid flags, starting eleven). The official file is
embedded in `libs/aatf` as the default; the editor's settings point at an alternative file. `toml`
and `toml_edit` no longer touch AATF; `cel-interpreter` leaves the dependency table.
Why: the Autumn 26 ruleset (4ccEditor `Autumn_2026_AATF`, tip `cf61542`) added a fourth medal tier
and three conditional rules that need sequencing, a preset x formation traversal and a
starting-eleven lookup; under CEL each is a new host-precomputed field, so a ruleset change would
still need a Rust release, which is what the configurable layer exists to prevent. Two files skew
(an invitational's TOML lacking `bronze` against a script reading it) and typed serde parameters
resist adding a tier; the stated workflow, bulk edits of the official ruleset and invitationals as
small deltas of it, is "copy one file, edit the top" only with one file. Rhai's `global::CFG`
gives functions the top-level constant without host plumbing. Costs accepted: denser syntax than
TOML, no enforced boundary between the sections, parameters readable only through Rhai.
Plan: `save_editor.md#Configurable AATF rules` rewritten; `core/README.md` crate tree, Phase 5 bullet,
dependency rows (`toml`, `rhai`) and decisions table; `team_creator.md`, `model_format.md`
"Comments are app-injected", `GLOSSARY.md`, `AGENTS.md` "User-facing TOML" reworded.

## 2026-09-19 - verification - mutation runs at review and converge, never as a gate
Decision (user): `cargo-mutants` joins the developer tools (`just mutants <crate>`,
`just mutants-diff [base]`; config `.cargo/mutants.toml`). The lead runs `mutants-diff` over each
landed step as the third check of its review, next to the honesty and design sweeps; converge's
design-health pass runs `mutants <crate>` per crate. Survivors are triaged as missing test,
equivalent mutant (excluded in the config with the equivalence named) or unreachable code.
Why: a probe on three closed crates (fpk 64 mutants, vtree 69, kit_config 334; 44 s, 43 s,
3 min 23 s) found 57 survivors: 25 equivalent (`|` vs `^` on disjoint bit fields, now excluded),
~30 real test gaps and two pub items no test calls. `kit_config` had passed the gates, both
review sweeps, converge and the cross-family reviewer with the PES 15-20 name encoding, the
`name.shape` and `cut-out` TOML values, two finding conditions and `matches_fpc` all untested;
none of these is a suppression the honesty sweep greps for, they are assertions never written,
which is the one test quality the methodology had no mechanical check for. Not a gate: roughly
an hour for the workspace today at 0.6 s per mutant, several hours at the planned size; the
per-diff and per-crate runs are minutes. The probe is reproduced by `just mutants <crate>`; its
numbers are the ones above.
Plan: `CONTRIBUTING.md` "Testing and verification" (recipes, "Mutation runs" requirement);
`AGENTS.md` review paragraph and converge design-health pass; `justfile`, `scripts/mutants_diff.py`,
`.cargo/mutants.toml`, `.gitignore`.

## 2026-09-19 - pes_savefile - model structs follow the version-gating rule, not the plan blocks
Decision (lead, recorded after the fact): `TeamEntry`, `TeamTactics`, `TacticsPreset` and
`PlayerEntry` as implemented in 2.17b-c stand; the plan's code blocks in `pes_savefile/README.md`
"Player/team/tactics model" are rewritten from the code in 2.21. Deviations: version-gated fields
are `Option<T>` (`manager_id`, `stadium_id`, `colors`, `kit_slots`, `star`, `tight_possession`,
`aggression`, `playing_attitude`, the two instruction pairs) per the plan's own rule; the
tactical half of a team lives under `tactics: TeamTactics` (its own record on disk) rather than
flat on `TeamEntry`; `edit_flags` and `players_to_join_attack` exist because the records hold
them; `bench_order` is `[u8; 21]`; the shirt number is on `RosterSlot`, not `PlayerBasics`; no
`serde` derives (no consumer yet; interchange formats are 2.17h).
Why: the blocks were written before the field tables were derived from the reference read walks
(2.17a-b); the tables, not the sketch, decide the shape. The gap is recorded here because
`AGENTS.md` makes every plan-shape deviation a decision entry and 2.17b-c landed without one.
Plan: `pes_savefile/README.md` blocks unchanged until 2.21.

## 2026-09-19 - model_convert - `formats/fmdl` and `formats/pes_model` are folder modules
Decision (lead): each format module is a folder, `mod.rs` (re-exports and the helpers both
halves share), `import.rs` (to IR), `export.rs` (from IR), `tests.rs`. Public paths unchanged
(`formats::fmdl::fmdl_to_ir`, `formats::pes_model::ir_to_model`).
Why: both files had passed the file-to-folder threshold (`CONTRIBUTING.md`: about a thousand
lines; 1167 and 1067) and each already had the two halves the plan names. The plan's rule
"one module per format: to_ir + from_ir, nothing else" still holds; a folder is one module.
Plan: `model_conversion/README.md` crate tree edited.

## 2026-09-19 - docs - plans over a thousand lines are folders; the spec stays in `docs/plans/`
Decision (user): a plan file that crosses roughly a thousand lines is split into
`docs/plans/<name>/` (`README.md` = preamble, part index, small cross-cutting sections; one file
per major section; heading text unchanged). Applied at once to the seven over the line:
`team_compiler` (2258), `core` (2174), `match_tracker` (1747), `model_conversion` (1208),
`aesthetics_export` (1144), `libs` (1084), `pes_savefile` (1009). The plans are not moved into
the crates when their sections are rewritten in the present tense.
Why: the same threshold the code uses (`CONTRIBUTING.md` "Architecture rules"); the large plans
are read by section and rewritten by phase, and `team_compiler` alone is consumed by four phases.
Keeping headings verbatim made the split a pointer-only change (the checker over every tracked
Markdown file reports the same 33 pre-existing loose section names before and after, and no
missing file). In-crate specs were rejected because plans and crates do not map one-to-one, the
present-tense rewrite is incremental, and `AGENTS.md`/`GLOSSARY.md`/`DECISIONS.md` pointers all
target `docs/plans/`.
Plan: `plans/README.md` "How these documents evolve" (the two rules), index rows.

## 2026-09-19 - aesthetics_export - the `settings.toml` key table is fixed, one key per line with its range comment
Decision (user): the key table in `aesthetics_export/settings_toml.md` "Player settings in exports"
is the export format: `name`, `[appearance]` (skin, iris), `[appearance.physique]` (height,
weight, fourteen -7..7 measures), `[appearance.strip]` (sleeves, inners, socks, undershorts,
untucked, ankle taping, wrist taping and two tape colours, spectacles and colour, gloves and
colour), `[appearance.motion]` (two hunching, two arm movement, three kick motions, dribbling on
20+, two celebrations), `[appearance.face]` (eleven feature types). Every key sits on its own line
followed by a comment giving its range or labels; the template emits the whole table in that
order, unset keys commented. Strip-style enums are lower-case string labels (the reference
editor's list words; undershorts as `off`, `short`, `winter_long`, `short_winter_long` for the
four summer/winter pairs), physique values are the game's signed numbers, 1-based motion values
are the game's numbers. The parser checks the widest range of a key; per-version narrowing is a
compile-time finding. Boots/gloves IDs, edit flags and the base-copy ID are compiler-owned and
are unknown keys to the parser.
Why: the file is hand-written by managers, so a key's range must be visible where the key is
typed, not in a document; grouped multi-key lines (the first draft) read as one setting. Labels
over stored indices for the enums because `sleeves = 2` says nothing; numbers for the colours
because their palettes are only lists of names the game shows as swatches. Widest-range parsing
because a settings file is version-neutral text and the same export compiles for several games.
Plan: `aesthetics_export/settings_toml.md` block and the provenance paragraph under it.

## 2026-09-19 - pes_savefile - the ingame-face run is opaque bytes with typed accessors; "every field" becomes "every decoded field"
Decision (user, option A of two): `PlayerAppearance.ingame_face: IngameFace` holds the appearance
block from byte 22 to its end verbatim (50 bytes on 16-21, 46 on 15); `IngameFaceField` names the
fifteen known bit runs inside it (player gloves and colour, skin, the eleven feature types, iris)
with one offset table in `schema/ingame_face.rs`, valid for every version because the block's
layout is; `IngameFace::get`/`set` are the only way to those bits, and `SkinColor`, `IrisColor`,
`PlayerGloves`, `PlayerGlovesColor` leave the per-version tables and `PlayerField`. `RecordSchema`
gains `ingame_face: Option<ByteRun>`; the generator emits it. `PlayerSettings` exposes the eleven
types plus the colours; the rest of the run is carried, not authorable, and the completeness test
asserts every *decoded* field is classified. Rejected: option B, decoding the run in-game first
(a manual session per version with a save diff per slider, blocking 2.17e until done).
Why: no legacy tool decodes the run beyond those bits, so "every appearance field" was a promise
no source could back; carrying the bytes whole is what makes transplant and fingerprint match the
reference scripts byte for byte, and accessors instead of parallel fields mean no byte has two
owners (a plain field plus a raw remainder would make the write order decide the result). A
player-only `Option` on the generic `RecordSchema` was preferred to a third type parameter or a
blob enum because it is one field with one meaning and the team/roster schemas simply say `None`.
Plan: `pes_savefile/model.md` "Player settings model" (run, field table, API blocks for
`PlayerSettings` and `ops::fpc`), `pes_savefile/codec.md` `RecordSchema` block,
`aesthetics_export/settings_toml.md` first paragraph, `libs/fpc.md` (`custom_skin_available`).

## 2026-09-19 - pes_savefile - `IngameFace::get` returns a `Result`, not a `u8`
Decision (lead, at review of 2.17e-1): `get`/`set` return `CodecError::NoIngameFaceRun` when the
run does not reach the field, which is the state of every entry never read from a record
(`Default` is empty); `IngameFace::from_bytes` is `pub(crate)`.
Why: the first cut indexed the run and panicked on a fresh entry; a silent 0 would repeat the
mistake `Missing` was introduced to avoid (2.17b). The plan block was changed first.
Plan: `pes_savefile/model.md` "Player settings model" `IngameFace` block.

## 2026-09-19 - pes_savefile - `is_fpc_player` classifies what the save shows; the compiler supplies its own
Decision (lead, at the 2.17e checkpoint review): `ops::fpc::is_fpc_player` stays the 4cc
convention's read-only test (nonexistent boots ID 55, or a custom skin) for the save editor;
the Team compiler, which knows the folder's `fpc.on`/`fpc.off` marker, passes its own
`is_fpc_player` to `fpc::check`. The reviewer's concern that a hide preset applied with a
substituted real boots ID reads as "not FPC" is accepted as a fact and rejected as a bug: such
a player is indistinguishable from a dressed one by construction, and no consumer that lacks
the marker can do better. `PlayerSettings::apply` and `ops::fpc::apply` are all-or-nothing;
`set`/`to_toml`/`update_toml` return `OutOfRange` for an unrepresentable stored value instead
of panicking; a NUL in an explicit `name` is refused at parse.
Why: a classifier that guessed intent from IDs would be wrong for exactly the teams that use
the ID-substitution the plan permits; partial writes on a rejected apply would leave a savefile
half-patched with no report; the `PlayerSettings` fields are plain public data (plan), so the
emitters, not the fields, must carry the check.
Plan: `pes_savefile/model.md` "Player settings model" code blocks (`set`, `apply`, `to_toml`,
`parse`, `ops::fpc`).

## 2026-09-20 - pes_savefile - `convert.rs` translates layout only; the converters' compile policy stays with the compiler
Decision (lead, opening 2.17f): `convert_player(source, from, target, to)` rewrites a *template*
target (a player read from a `to`-version save) from the source: every field that translates
one-to-one is copied through (name, stats, positions, skills, motion, edit flags, boots/gloves/
base-copy IDs, physique, strip fields, the ingame-face run), what does not is capped, dropped or
left to the target and reported as a `ConvertNote`. The reference converters' other writes (boots
55/gloves 11 or 0 by the models present, edit flags 0x0C/0x0F, wrist taping, ankle taping and
player gloves cleared) are not performed: they are compile policy the Team compiler applies
through `PlayerSettings`/`ops::fpc` after conversion. The 46/50-byte run rule falls out of the
template design: the source run is copied over the target run's prefix (`min(len)` bytes), a PES
15 target drops the source's last four bytes (`FaceRunTruncated`), a 16+ target keeps its own.
Rejected: padding a 46-byte run with zeros or a fixed pattern (invents bytes for a block nobody
has decoded); a converter-faithful module that also rewrites IDs (would give the IDs two owners,
the compiler being the other).
Why: the reference editor's PES 15 and 16 read walks are identical up to the iris byte and differ
only in the tail skip (3 vs 7 bytes), so "PES 15 = the first 46 bytes" is measured, not assumed;
the template already holds a real player's tail bytes, which is the best available value for
four bytes of unknown meaning. Compile policy in the converter was a consequence of the converter
being the only tool in the pipeline; here it would be applied twice.
Plan: `pes_savefile/operations.md` "Cross-version player conversion", "What the Rust module is".

## 2026-09-20 - pes_savefile - face caps reset to 0 (not clamp); skin 7 reset only across a no-custom-skin side; caps table per version
Decision (lead): a face type above `schema::limits::face_type_cap(to, field)` becomes 0, the
converters' `cap(max, default=0, value)`, although the plan text said "clamp"; the table's PES
16 and 21 columns are the two converters' caps, 15/17/18/19 are assumed equal to 16 and 20 to 21;
`settings_toml`'s widest face ranges are derived from the table so the numbers have one home.
Skin colour 7 is reset to 1 only when `fpc::custom_skin_available` is false for `from` or `to`.
Rejected: clamping to the cap (no reference produces that value); resetting 7 unconditionally
(the converters do, but both their directions have a no-custom-skin side, and a 16 → 17 import
would strip a partial-hide FPC player's custom skin); resetting to 0 as the reference editor's
import does (the converters are the verified path and they write 1).
Why: parity is measured against the converters' outputs, and a clamped value would be a value
neither tool ever wrote.
Plan: `pes_savefile/operations.md` "Cross-version player conversion" (caps table and skin rule).

## 2026-09-20 - pes_savefile - canonical `PlayStyle` with per-version lists; an unlisted stored value is an error
Decision (lead): `model/playstyle.rs` holds the 22-value canonical enum (PES 20/21's list);
`schema/playstyle.rs` holds one list per version group (15/16, 17/18, 19, 20/21) with
`decode(version, u8) -> Result<PlayStyle, CodecError>` and `encode(version, PlayStyle) ->
Option<u8>`; the twelve reference conversion arrays are a test's golden data, reproduced as
`encode(to, decode(from, i))`. A stored value outside the version's list, PES 15/16's index 16
included, is `CodecError::UnknownPlayingStyle`, not `None`. `PlayerPositions.playing_style`
stays the stored `u8`.
Why: a census of style × registered position over the six fixture saves confirmed the arrays'
index spaces (goalkeepers at 17/18 on 15/16, 16/17 on 17/18, 20/21 on 19–21) and found no
player at 15/16's index 16 in ~8800 records, so the gap is a hole to refuse rather than a style
to name; the reference editor's PES 15/16 combo box (the 17/18 names) contradicts its own arrays
and the census sides with the arrays. The model keeps the index because the codec has no version
in hand and only conversion and interchange need the canonical value.
Plan: `pes_savefile/operations.md` "Cross-version player conversion" (playing styles table),
`pes_savefile/model.md` "Player/team/tactics model" version quirks paragraph.

## 2026-09-20 - pes_savefile - a `ConvertNote` reports what depends on the player's data, not on the version pair
Decision (lead): `convert_player` emits a note for every non-carry that a *different player*
would not trigger (a face type over the cap, skin 7 across a no-custom-skin side, a style or
skill the target lacks *and the player has set*, a text cut, a 50-byte run onto a 46-byte
target). What is the same for every player of a version pair is not a note: a gated stat the
target version has no field for (the target's `None` stands), or the four tail bytes a 16+
template keeps against a PES 15 source.
Rejected: `GatedFieldDropped`/`FaceTailFromTemplate` variants (raised by the checkpoint
review). They would fire on every player of a 19 → 15 or 15 → 16 conversion and bury the
per-player notes; the caller knows the version pair and states the fact once.
Why: the plan sentence "each such case is reported as a note" was written for the lossy
per-player cases and read literally contradicted the plan's own `ConvertNote` block, which has
no such variants; the sentence was narrowed rather than the enum widened.
Plan: `pes_savefile/operations.md` "What the Rust module is" (first paragraph).

## 2026-09-20 - pes_savefile - transplant copies the whole appearance block; the comparator compares every stored field, aesthetics = what a transplant moves
Decision (lead, opening 2.17g): `ops::transplant::transplant_player` copies `appearance` (the
ingame-face run whole) and the four appearance edit flags, where the reference script copies
bytes 4..68 of the block and so leaves a 16+ run's last four bytes. Measured first: those bytes
are zero on all 24 656 players of the 16/17/18/19/21 fixtures, so the outputs are byte-identical
on real saves. `ops::compare::compare_players` walks the version schema's stored fields through
the model (plus the texts and the known ingame-face bits) rather than a hand-kept list, and
tags each row by `DiffScope`: aesthetics is the set `transplant_player` moves, everything else
(motion, the other edit flags, nationality) is gameplay. A change inside a known face field is
one `Face` row; `FaceRun` (the reference's fingerprint, old and new) is emitted only when bits
no `IngameFaceField` names differ. Saves are paired by player id. There is no `Fingerprint`
struct: `PlayerAppearance` is already the comparable value and `FaceHash` is the only thing the
struct would have added.
Rejected: reproducing the 46-byte slice (two rules for one block, and the plan's run is a unit);
a hand-written field list for the comparator (the reference editor's omits motion, edit flags
and nationality by accident of its author's needs, and a list drifts from the schema); reporting
`FaceRun` on any normalized-run difference (a nose change would print twice); the reference
editor's roster-slot pairing (a reshuffle is not an edit).
Why: byte parity is the standard and it holds; the schema is the one place that knows what a
version stores, so a comparator that reads it cannot miss a field the way both references do;
the transplant preview and the "did anything visual change" question both reduce to the
aesthetics rows, so a separate fingerprint struct would be a second copy of `PlayerAppearance`.
Plan: `pes_savefile/operations.md` "Save-to-save operations".

## 2026-09-20 - pes_savefile - 2.17g review rulings: schema stays `&'static`, first id wins, no second `Id` row
Decision (lead, at the 2.17g checkpoint review): `VersionSchema.appearance` keeps the codec
plan's `Option<(SectionLayout, &'static RecordSchema)>` shape (a by-value change made to satisfy
a test's match arms was reverted: the plan block and `scripts/derive_savefile_schema.py` both
say `&APPEARANCE`, and a shape the generator cannot emit is a shape the tables cannot be
regenerated into). `compare` indexes the second save first-record-wins, as `EditFile::player`
resolves a duplicate id, so the two APIs never disagree. The 15/16 appearance record's walk skips
every field the player record already stores (`Id`), so one identity difference is one row on
every version. `compare_players` propagates every codec error: a field the schema stores that
either entry lacks is `Err`, two unread entries included (a both-sides swallow was removed).
Why: the reviewer showed three tests that passed on a no-op or eager implementation because the
fixture pairs they used were identical; the fixes are in the tests, and `scope()` is now pinned
to the plan's own definition ("what `transplant_player` moves") by a test that flips every PES
21 field once.
Plan: `pes_savefile/operations.md` "Save-to-save operations" (unchanged; the rulings implement
it), `pes_savefile/codec.md` `VersionSchema` block (unchanged, reaffirmed).

## 2026-09-20 - plans - pes-db-generator joins the suite as the DB generator tool; Konami database tables get a `pesdb` lib
Decision (user + lead): `Tools_4cc/pes-db-generator`, the scripts that build a cup's game
database (the `common/etc/pesdb/*.bin` tables declaring the teams, placeholder players, managers
and competition entries, plus the hand procedure that gives a fresh 19+ save its player records),
becomes `tools/db_generator` (Phase 19, scheduled by need after Phases 2 and 8; plan
`db_generator.md`). Its record formats become `libs/pesdb`, which also takes the `Ball.bin` /
`BallCondition.bin` layouts the Balls compiler plan had kept inside that tool and the
`Stadium.bin` layout the Stadium compiler plan kept inside its tool: three writers of one table
family is the multiple-consumer bar of workspace guardrail 3, and the alternative was three
private copies of "fixed-size little-endian records with a per-version layout". The savefile
half of the scripts (count at 0x60, records at 0x7C) is `pes_savefile::ops::populate`, the one
API through which player records are added to an `EditFile`; the teams list input is the
suite's `teams_list.txt` through `libs/teams_list`, whose `Row::Placeholder` rows are exactly the
Backup/VGL/Invitational teams the scripts route to competitions 12/11/10.
Measured first: the PES 20 invitational save on the reference machine is a day-0 save of a
database these scripts produced; its record 70101 equals the record `player_edit.py` assembles
byte for byte, and the `Player.bin` record is a different layout from the EDIT record (the
gameplay block is not byte-shared), so the two are two formats in two crates.
Rejected: a Python-shaped port (byte templates with ids patched in) - the templates encode
layouts the suite should know by name; keeping the tool out of the suite - it is the first step
of every cup and the only one still needing a hex editor; folding `pesdb` into `pes_savefile` -
the savefile knows nothing of the database and the database CPK is the Team compiler's neighbour,
not the save's.
Plan: `db_generator.md` (new), `core/development_plan.md` "Phase 19", `pes_savefile/operations.md`
"Player section population", `balls_compiler.md` and `stadium_compiler.md` (placement rows),
`plans/README.md`, `libs/README.md`, `GLOSSARY.md`.

## 2026-09-20 - pes_savefile - 2.17h interchange formats: measured Texport layout, one import path, Team TOML keyed by roster slot
Decision (lead): the interchange formats are shaped by what the real files on the reference
machine measure, not by the plan's transcription of the reference editor's read walks:
- **Texport 18-21 is the save's own records concatenated** (team | 88-byte coach | roster |
  tactics | 660 bytes | players x roster width | 12 bytes) behind a 0x50 header and a 32-byte
  XOR key. Every record decodes with `schema_for(version)`'s record schemas at the offsets the
  sizes imply (measured on four 18, eleven 19 and five 21 files: `0x32C`/`0x834`, `0x33C`/`0x844`,
  `0x410`/`0x918`, the reference editor's numbers), and the 17 file opens with `SaveContainer`
  unchanged. So `interchange/texport.rs` owns crypto and carry-through only, `schema/texport.rs`
  the per-version constants, and no texport field table exists. Rejected: transcribing the
  reference's `fill_*_texport` walks as a second set of tables - they are the save tables with a
  different starting byte, and a second copy drifts.
- **Texport write synthesizes 18-21 files from measured templates** (header, coach block, tail
  per version; the 660-byte block zero, which real files carry) and is template-patching on
  15-17 (`NoTemplate`): the header holds no content checksum the census could find, so a
  constant header is the best evidence available; whether the game accepts it is the manual
  check the plan already names. PES 20 keeps the reference's untested key index `0x14`.
- **One import path**: `.4ccs`, `.4cct` and Texport reads all produce a `TeamToml`, and
  `TeamToml::apply` is the only code that writes interchange data into a save. Rejected:
  per-format importers (three copies of the slot mapping, the version conversion and the notes).
- **Team TOML players are keyed by roster slot**, never by id (`[players.01]`, as the aesthetics
  patch does), so a file applies to any team and carries no id; `base_copy_id` equal to the
  file's own player id means "unset" and becomes the target's own id. Gameplay values are the
  stored numbers with bit-width ranges (a dump, not the manager-facing `settings.toml`
  reinterpretation), labels only where the model already has a canonical enum; the
  `[players.NN.appearance]` table is `settings.toml`'s `[appearance]` verbatim through
  `PlayerSettings`' own parser/emitter. Presets and formations are named tables
  (`preset_1.formation_2`), not arrays of tables, so "absent = untouched" holds per preset.
- **Advanced instructions get a canonical enum** (`model/instruction.rs`, the reference's
  17-based order) with per-version `schema/instruction.rs` lists, the `PlayStyle` pattern; the
  model's `AdvancedInstruction.instruction` stays the stored `u8`.
- **`.4ccs` accepts tags `"20a"` and `"21a"`**: the plan says `"21a"` (the reference's current
  constant) but every real file on the machine is `"20a"`, and the 356-byte record decodes them
  exactly (23 records, sequential base-copy ids, `numbers` filling the tail), so the tag bump
  did not change the record. Reading the two identically is the whole cost.
- **The `.4cct` fixture is synthesized** (no real file exists): the reference's `save_tactical_data`
  write walk transcribed in Python over the PES 19 fixture save's team 713; the test's evidence
  is agreement with the Rust schema codec's `TeamTactics` for the same team, two independent
  paths.
Not measured: PES 15/16/20 texports (no files), the 15-17 texport's team and roster record
positions (the 17 payload holds the team id at four places before the tactics record; team and
roster of a 15-17 texport come from the target save until measured), `shirt_name_from`'s
character set beyond upper-casing.
Plan: `pes_savefile/operations.md` "Interchange formats" (rewritten), `pes_savefile/README.md`
crate tree, `pes_savefile/verification.md`.

## 2026-09-21 - pes_savefile - PES 20 Texport size is derived from its record sizes, not the reference's 0x39E4
Decision (lead, on the sidekick's finding in 2.17h-1): the reference editor assumes PES 20 and
21 texports share one size (`0x39E4`) and differ only in key index. PES 20's team record is 528
bytes and its roster 244 against 21's 588/284, so a 21-sized body cannot hold `team | coach |
roster | tactics | 660 | 40 players | tail` with 20's records: the width invariant failed on the
first test. `schema/texport.rs` therefore gives PES 20 the size its own records derive
(`0x39A8`), the reference's key index `0x14` and PES 21's header/coach/tail templates, all marked
unverified; detection is by size alone, every size now distinct. Rejected: keeping `0x39E4` and
a two-way detection - a layout that contradicts its own schema cannot be right, and the
detection rule would encode the contradiction. When a PES 20 texport is measured, the layout is
one table row to fix.
Plan: `pes_savefile/operations.md` "Texport (read + write)".
