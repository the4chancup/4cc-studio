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

## 2026-09-21 - pes_savefile - Team TOML carries stored values past the editor ranges; `settings.toml` stays strict
Decision (lead, on a 2.17h-3 finding): real saves store appearance values the reference editor's
UI cannot produce - 58 of the PES 15 fixture's players have `cheek_type` 14 on a field whose
editor cap is 3, 79 of PES 18's likewise, and celebrations above the `settings.toml` key table's
maxima (a stored 183 against "1 to 162"); a PES 16 player stores `place_kicking` 100. A dump
that refused them could not round-trip its own save, so Team TOML parses every scalar against
the field's *stored* range (the widest bit width of the key's `PlayerField` across the version
tables, computed from the schemas, no hand-typed maxima) and emits raw stored values, the block's
range comments documenting the game's editor range for the reader. `settings.toml` keeps the
editor ranges (`PlayerSettings::from_player` still refuses such a player: an authored aesthetics
file must be one the game's editor could have produced). The consequence in code:
`PlayerSettings::from_player`/`apply` were not reusable as the plan literally said; the shared
pieces became `settings_toml`'s `pub(crate)` `appearance_from`, `apply_appearance`,
`parse_appearance` and `emit_appearance`, with a `stored_ranges` mode, and both formats call
them (one key handling, two range policies). A test pins the split: a PES 15 player over the cap
round-trips through Team TOML while `from_player` on it is `OutOfRange`.
Rejected: clamping on export (silent loss, the thing the crate exists to avoid); relaxing
`settings.toml` (its strictness is the compiler's guard against typos).
Plan: `pes_savefile/operations.md` "Team TOML" (the `[players.NN.stats]` comment and the
`PlayerSettings` reuse sentence).

## 2026-09-21 - pes_savefile - 2.17h checkpoint (b) rulings: source-version skill gating, 15-17 texport import document, text-length checks in `apply`
Decision (lead, on the cross-family review of the 2.17h surface; seven concerns, all verified):
- **Up-conversion leaves alone what the source never stored.** A Team TOML written from a PES 16
  save lists 28 skills; applied to PES 19, the target's Double Touch stood to be cleared by a
  `false` the source could not have meant. `apply` skips a skill the *source* version lacks (the
  target's value stands, pair-wide, no note) and drops one the *target* lacks with a note. The
  document itself still emits the full array: the version gate at `apply` carries the meaning,
  not the file. Rejected: emitting only the source's skills as a list - a hand-written list would
  then be ambiguous between "unset" and "not in my version".
- **A 15-17 texport implies its roster.** The format carries no team or roster record we have
  measured, but its player records are consecutive by slot, so `Texport::team()` holds the roster
  they imply (shirt numbers 0, unknown) and `Texport::to_team_toml()` is the import document for
  every era: on 15-17 the `[team]` section is `id` alone and `number` is absent, so the target
  save's team identity and numbers stand, as the plan says. Rejected: making callers reduce the
  document by hand.
- **Face types past the cap reset to 0** on import, as 2.17f's conversion does (the converters'
  rule), not clamped to the cap; the `Capped` note carries what was written.
- **`apply` checks text lengths** against the target version's text fields (`TextTooLong { path,
  max }` before any write): a valid 60-byte PES 21 name would otherwise pass `apply` and fail the
  later `write_player`, breaking the all-or-nothing contract at the wrong layer.
- **Legacy tactics are gated by the exporting version**: the `.4cct` block always has instruction
  and auto-flag bytes (the reference writes zeros for versions without them); the reader leaves
  `None` what the header's version lacks, so a PES 16 file neither fails a same-version apply on
  `attack_defence_levels` nor overwrites fields it could not author.
- **Stored-range mode is bounded by the bit width** of the key's field (widest across versions,
  from the schemas; `schema::widest_bit_width`) - the 2026-09-21 stored-ranges decision as
  written, which the first implementation had left at `u8`; a label kind whose stored value has
  no label is emitted and parsed as the integer in Team TOML, so a width-valid value never panics
  the emitter.
- Two hostile-input panics closed (a non-ASCII `ingame_face` string sliced at a byte index; a
  `.4ccs` with more than 40 records), one test oracle rebuilt from a pre-apply snapshot (the old
  one passed against a deliberately clobbered `apply`).
Plan: `pes_savefile/operations.md` "Team TOML" (`apply` rules, `from_team` doc, `Capped`),
"Texport (read + write)" (`team()`, `to_team_toml`).

## 2026-09-21 - pes_savefile - 2.17h second review round: the cap was binding
Decision (lead): the first round returned seven concerns, all accepted, so under the new
`AGENTS.md` rule (rounds continue while a round returns seven with at least five accepted) a
second round ran over the same surface with the rework diff and the prior rulings. It returned
seven more, all verified, all accepted:
- **A 15-17 texport refuses a player list it could not read back.** The reader's termination
  rule (`id == team.id * 100 + 1 + slot`) is ours, unmeasured; the writer laid records out
  positionally, so a reordered list wrote fine and reopened short. `to_bytes` now returns
  `OldPlayerIds` before any write. Rejected: loosening the reader to "until id 0", which would
  accept files whose real termination rule we have not measured.
- **Team TOML text refuses an embedded NUL at parse** (`WrongType`, "a string without NUL"),
  the 2.17e rule for `settings.toml`: the codec's fields are NUL-terminated, so the value would
  reload truncated.
- **A shirt number wider than the target's roster field is refused at `apply`**, not clamped to
  the reference's 231: same wrong-layer break as `TextTooLong` (applied Ok, then `write_roster`
  failed). Width from the target schema's `RosterField::Number`, never a literal.
- **Checked arithmetic on user integers**: the stored-range shifts (`+7`, `-1`) and the own-id
  convention (`team_id * 100 + slot`) overflowed on `i64::MAX`/`i64::MIN`/`u32::MAX`; `None`
  is `OutOfRange` or "not the convention".
- **A legacy playable rating past the four labels is refused at `read_squad`**, not at emission
  (`to_toml` stays infallible for it).
- **Inline records reject unknown members** like top-level keys (`UnknownKey` with the array
  index in the path); `toml_edit::InlineTable` is `TableLike`, so the existing `reject` serves.
- **The `.4ccs` golden covers all 23 records and every mapped field** with the ctypes offsets
  as literals, so a mapping dropped from `legacy.rs` fails. Writing it surfaced a schema fact
  the sampled golden had not: PES 19 stores no `Aggression`, `PlayingAttitude`,
  `TightPossession`, `StrongerHand`, dribbling motion (PES 20 added them) and no base-copy edit
  flag (15-18 hold it); the golden lists them as literals. The 19 base-copy flag is the codec
  phase's table, not re-measured here.
The third round returned three (under the cap, so the loop stopped there), all accepted:
- **`apply` checks every stored value against the target's field width** (`schema::bit_width`,
  the per-version sibling of `widest_bit_width`), player keys, appearance keys and the kit
  slots' 24-bit `binding`, before any write: parse bounds by the widest width across versions
  (the stored-ranges decision), so a PES 21 `free_kick = 17` reached PES 19's 4-bit field and
  failed in `write_player`. One `check_width` helper, one `check_text` helper (NUL, single-byte
  encodability for `shirt_name`/`short_name`, capacity) at all four text sites, so a
  caller-built `TeamToml` meets the same rules a parsed one does.
- **The texport golden decodes every record with the schema codec at the literal offsets** and
  compares the whole tactics and every player, on the 17/18/19/21 fixtures; the 17 test's
  `starting_eleven.len() == 11` was true of the array type.
Plan: `pes_savefile/operations.md` "Legacy 4ccEditor formats" (the number sentence);
`verification.md` "Interchange formats".

## 2026-09-21 - python_bindings - 2.18 shape: a codec-only wheel, `abi3-py311`, a workspace member that `cargo test` never links
Decision (agent, within the plan's "PyO3 shim, proves guardrail 4 with a real cdylib build and a
smoke test"):
- **The Phase 2 surface is the four codecs' `read`/`write`** (`Fmdl`, `Skl`, `Model`,
  `MaterialSet`) under one module, `pes_models_native` (the `pes-models` extension's native
  module; the extension keeps its pure-Python fallback, so "native" is the wheel's role). The
  Blender-facing accessors wait for the extension's hot-path step, where the calling code decides
  their shape. Rejected: exposing `format/` + `ops/` wholesale now - forty functions with no
  consumer, each a guess at what Blender's addon code wants to hold.
- **`abi3-py311`**: Blender 5.0 bundles Python 3.11 and 5.2 bundles 3.13 (both measured on the
  maintainer's installs), so one wheel per platform serves every Blender the extension targets
  and the developer's own interpreter; the limited API's cost (no fast buffer paths) is nothing
  at this surface.
- **Workspace member, `cdylib` with `test = false`/`doctest = false`, `extension-module` on by
  default.** Membership gives the crate the lint table and `cargo clippy --workspace` at every PR;
  `test = false` keeps `cargo test --workspace` from linking a Python extension against a
  `libpython` it does not have (the interpreter that imports it provides the symbols). The
  crate's one test is the wheel. Rejected: a feature-gated `extension-module` with
  `--no-default-features` for tests (the pyo3 FAQ's other route) - it makes the default build the
  one nobody ships.
- **The smoke test runs the wheel unzipped onto `sys.path`**, not pip-installed: Blender's
  bundled Python has no pip, and the plan wants the test under Blender's Python (`just bindings
  <interpreter>`). The oracle is each crate's own round-trip invariant; the test is not pytest
  for the same reason.
- `pyo3` joins "External Dependencies" (the table had `pyo3-log` and not the crate it wraps);
  `maturin` is a developer tool like `just`, pinned in CI (`1.15.0`, 2026-08) and installed with pip.
Plan: `core/development_plan.md` "Phase 2" (`python_bindings`), `core/README.md` "External
Dependencies", `CONTRIBUTING.md` "Testing and verification" (the maturin line).

## 2026-09-21 - development plan - the GPU BC7 backend moves from Phase 2 to Phase 4
Decision (user): worklog step 2.5b (`dds_convert` GPU BC7: wgpu device, CPU fallback, cold-start
and throughput measured) leaves Phase 2 and is built in Phase 4 with the Team compiler's texture
step. Phase 2 delivers the CPU reference only.
Why: build it with its consumer, not before it, because every rule the plan gives the GPU path
(bounded batches against the memory budget, cancellation, writer progress, one encoded result
shared by all consumers, the fallback report) is pipeline integration; a Phase 2 version would be
shaped without a caller and reshaped once the texture step exists, the same "no consumer yet"
risk that made Phase 3 open with a tracer bullet. Rejected: a standalone proof now (device + one
encode + decode within the CPU tolerance), because the measurement it would produce (cold
pipeline creation, upload/readback) is only actionable once there is a pipeline to schedule
around, and the CPU path already satisfies the fixture-based quality checks the GPU output must
meet.
Plan: `core/development_plan.md` "Phase 2" (`ftex` + `dds_convert` bullet), "Phase 4"
(`processing/` bullet). `libs/README.md` "First-release desktop GPU BC7" is the backend's spec
and is phase-agnostic; unchanged.

## 2026-09-21 - archives - `DuplicateName`, the zip open cost, the codec fixtures
Decision (agent, at Phase 2 converge, cross-family review):
- **`ArchiveError::DuplicateName(String)`** joins the plan block: two entries normalizing to one
  tree path (`a/b` and `a\b`) were silently resolved by `HashMap` insertion order, against the
  plan-wide rule that collisions are rejected. Exact-duplicate raw names in a zip are collapsed
  by the `zip` crate's name index before the crate sees them; accepted, no tool writes such a zip.
- **`zip` reads each entry's local header at open**, not "the central directory only": `zip`
  8.6 exposes sizes and the encryption flag only through its raw entry view, which seeks to the
  local header. One seek per entry, no decompression; the plan sentence now says so rather than
  the code pretending otherwise.
- **Three codec fixtures** (PPMd `.7z`, entry-encrypted `.7z` with a readable header, BZip2
  `.zip`) pin the plan sentences that were untested: unsupported codecs list and fail at `read`;
  encryption is `Encrypted` at `read` when the header is readable. With `sevenz-rust2` built
  without AES and given the empty password, the crate's password errors are unreachable, so the
  arm matching them was dead code and is gone.
Plan: `libs/archives.md` (the block, the codec and encryption paragraph, the fixtures paragraph).

## 2026-09-21 - uniparam - canonical writer, entry-level round trip; Konami's layout measured
Decision (agent, at Phase 2 converge): `UniformParameter::write` keeps the reference writer's
layout (byte-wise name order, name pool directly after the table, contents padded to 16) and
the crate's round-trip standard is entry equality, with byte identity tested against the
reference writer's sample only. Konami's PES 21 container was measured: table order with `_`
before the digits, content pool 16-aligned (nine pad bytes), otherwise identical; not
reproduced, because Red's output (pes-file-tools' writer) is the parity standard and the game
reads both. Rejected: reproducing Konami's layout (a second layout the compiler never emits).
Plan: `libs/format_crates.md` "`uniparam`: a canonical writer, parity with the reference writer" (new).

## 2026-09-21 - fpc - the PES 19+ kit-value confirmation moves to Phase 4
Decision (agent): the confirmation that the FPC kit values (176/16/105/105) hold on PES 19+ was
due "when `kit_config` lands" and did not happen: no PES 21 FPC team's kit config was identified
on the machine, and the stock configs `kit_config` measured are not FPC kits. It is now the
first PES 21 FPC export compiled in Phase 4 (`matches_fpc` on its configs).
Plan: `libs/fpc.md` (`kit.rs` bullet).

## 2026-09-21 - teams_list, kit_config - Phase 2 converge rulings
Decision (agent, cross-family review at converge):
- **Team rows are the slash-wrapped Name cells only.** `TeamName::new` accepts bare tokens
  because it is the export side's fold too, so `701\tBackup` was loading as team `/backup/`;
  the file parser now checks the wrapping on the raw cell, per the Team compiler contract
  ("rows whose Name is not slash-wrapped load but can never match").
- **Reconcile validates uniqueness over every claimed id and consumes a placeholder whose id
  an existing name takes**, so the merged list always parses again (it did not: a same-name
  override onto a placeholder's id left two rows claiming it). The revert loop is gone:
  changes are computed once against the working list, colliding ones dropped, rows recomputed
  (the 2.12b simplification note).
- **kit_config's TOML writes are in-place** (`update_toml` keeps `[unknown]` and badge-table
  comments and forms, returns `Result` instead of panicking on a non-table section), short
  sleeves join the clamp table (kept, reported, clamped; not masked), `[unknown]` entries the
  codec cannot carry are refused at parse. Two lead goldens pin the codec to the plan's
  offset table by hand (the referee fixture field by field, five distinct colors at the
  table's offsets), since no fixture had two distinguishable values in every same-width field.
- **Blue's `UniformParameter18/19.bin`** are fixtures: 2214 + 2210 configs round-trip
  bit-identically at PES 18/19; 17 GK configs per season carry a shirt model outside
  144/160/176 (reported as `kit_shirt_model_unknown`, Info), so the PES 21 model-range
  assertion is not made on them.
Plan: `libs/teams_list.md` (`file.rs`, `reconcile.rs` bullets), `kit_config_editor.md` ("The
format" row 0x00, "`libs/kit_config`" first bullet).

## 2026-09-23 - cpk - Phase 2 converge rulings
Decision (agent, lead audit and cross-family review at converge):
- **The crate's public surface is what the tools need**: `CpkArchive::{open, entries, read}`,
  `CpkEntry`, `CpkTimestamp`, `CpkWriter::{new, add, finish}`, `CpkError`. The @UTF table
  types, `header()`, `into_inner` and the `crilayla` module had no consumer in the workspace or
  the plan (`core/architecture.md`: CRILAYLA is used only inside `cpk`) and are `pub(crate)` or
  gone; a later consumer re-exposes what it needs.
- **The reader follows the reference reader where it is stricter and where it is lenient**:
  pool strings that are not UTF-8 are refused (`InvalidUtf8`; pes-file-tools' `readString`
  raises), header and TOC integer cells are read by name regardless of U32/U64 width (its
  `row['ExtractSize']` is untyped), an entry whose sizes differ but carries no CRILAYLA magic
  comes back raw (its `read` does the same). Beyond the reference: every file-declared count
  and offset is bounds-checked before the allocation it would size (row area, row count,
  `FileOffset + base`, the CRILAYLA payload against `ExtractSize`), so a hostile archive is an
  `Err`, not an abort.
- **The writer refuses what the format cannot carry** rather than writing it truncated: a path
  holding a NUL (`InvalidPath`; the string pool is NUL-terminated) and a `Tvers` long enough
  to push the header table past the 0x800 bytes reserved before the first file
  (`HeaderTooLarge`). Its bytes are unchanged (parity test untouched).
Plan: no plan edit needed; the crate has no code block, and `libs/README.md` "Testing" already
states the fixture standard these tests follow.

## 2026-09-24 - ftex, dds_convert - Phase 2 converge rulings
Decision (agent, lead audit at converge):
- **One-channel sources decode grey.** BC4, DX10 R8 and legacy L8 decode to `R = G = B`, alpha
  255, because texconv (the reference decoder) does: its R-to-RGB conversion splats the channel
  (measured on texconv 2024.1.1.1; `DirectXTexConvert.cpp` "R format -> RGB format"). The GPU
  samples DX10 R8 as `(r, 0, 0, 1)`; conversion parity follows the reference decoder, not the
  sampler. BC4's interpolated values stay truncated: that build's R8 store truncates (BC5's R8G8
  and BC3's RGBA stores round). DirectXTex added the R8 rounding bias in 2026 (#671), so the
  fixtures pin the 2024.1.1.1 build, the one the legacy compilers shipped.
- **DXGI 96 is refused as signed** (`BC6H_SF16`); it had been mapped to FTEX's unsigned BC6H.
- **`dds_convert` rejects a zero dimension and a mip count past `log2(larger side) + 1`**, as
  D3D and texconv's default loader do (`CalculateMipLevels` → `E_INVALIDARG`; only
  `--permissive` clamps). `ftex`'s FTEX↔DDS conversions keep pes-file-tools' acceptance (no
  PES 2021 FTEX of 10370 exceeds the bound; the conversions stay lossless for any that would);
  `mip_size` no longer panics past level 31 (the reference's floor division, 0 → 1).
- **The `ftex` surface is `read_layout`, `header_bytes`, `DdsLayout`, `DdsPixel` (with
  `row_bytes`, the tight-row rule both crates use) and the FTEX functions**; `DdsHeader`,
  `Dx10Header` and `PixelFormat`'s id/DXGI/FourCC/block-size methods had no consumer outside the
  crate and are `pub(crate)`.
- **Retained gaps:** a hostile FTEX can make `ftex_to_dds` pad a frame to a large size (the
  reference pads without limit); `build_header` truncates a linear size past 4 GiB to its u32
  field; RGBA32F stays DXGI 2 (`R32G32B32A32_FLOAT`) although pes-file-tools' reader expects 1
  (the TYPELESS id; its own writer emits 2).
Plan: `libs/dds_convert.md` ("In-process DDS conversion" decoding bullet, the accepted-format
table's DDS row, the `decode` paragraph under "`dds_convert` API").

## 2026-09-26 - ftex, dds_convert - converge review, first round
Decision (agent, cross-family review at converge):
- **Reads are bounded by what the texture needs, not by what the file declares.** `ftex_to_dds`
  stops reading chunks once a frame holds its mip size and inflates only the bytes still
  needed (a small file could otherwise inflate gigabytes that the frame then truncated), and
  `mip_size` saturates instead of overflowing on `u32::MAX` dimensions. Output is unchanged for
  every file the reference reads; a file corrupt only past the bytes a frame needs now converts
  where the reference would raise.
- **What has no straight-alpha RGBA decode is refused:** DX10 premultiplied alpha (alpha mode 2)
  and uncompressed headers with no channel mask (paletted DDS), which decoded black.
- **`dds_to_ftex` refuses a declared row pitch wider than the tight row** (`"padded rows"`): FTEX
  has no pitch field, and the reference writes the padding into the pixels.
- **`convert` validates a caller-built `Decoded`** (`InvalidDecoded`) rather than panicking or
  passing wrongly sized blocks through; the 2.17h rule that a caller-built value meets a parsed
  one's rules.
- **Cache retention stays unbounded until the Phase 4 memory budget** exists (worklog 4.y);
  `Converter` exposes `retained_bytes` and `clear` for it.
Plan: `libs/dds_convert.md` (the `decode` paragraph under "`dds_convert` API", "In-memory
conversion cache" retention bullet).

## 2026-09-26 - ftex, dds_convert - converge review, second round
Decision (agent, cross-family review at converge):
- **TIFF associated alpha is un-multiplied** (`ExtraSamples = 1`): `tiff` 0.11.3 reports it as
  plain RGBA and `image` copies the samples, so the decode was premultiplied. `tiff` becomes a
  direct dependency of `dds_convert` at the version `image` already pulls (no crate enters the
  graph) to read the tag; channels are `min(255, (c * 255 + a / 2) / a)`, zero alpha unchanged,
  checked against Pillow's decode of the lead fixtures.
- **Paletted DDS is refused** (`DDPF_PALETTEINDEXED8`, P8 and A8P8), and **NVTT v1's L8 header
  reads as R8** (DirectXTex `DDSPF_L8_NVTT1`): it decoded red where texconv decodes grey. Through
  `dds_to_ftex` it now converts to an FTEX R8 like the luminance-flag L8 (pes-file-tools refuses
  both; FTEX has the format).
- **Raw FTEX frames read at most their mip size**, completing the first round's bounded reads.
- **`row_bytes` covers every pixel-stored format** (the float and packed ones included), so the
  padded-row refusal and the row-size rule apply to all of them.
- **Retained gap:** legacy A8L8 headers (luminance or NVTT v1 RGB flag, R 0xff, A 0xff00) decode
  by their masks to `(L, 0, 0, A)`; texconv 2024.1.1.1 maps them to R8G8 and decodes
  `(L, A, 0, 255)`. Neither is a grey with alpha, and no export uses the format.
Plan: `libs/dds_convert.md` (the accepted-format table's TIFF row, the `decode` paragraph under
"`dds_convert` API").

## 2026-09-26 - ftex, dds_convert, wezlib - converge review, third round
Decision (agent, cross-family review at converge):
- **`wezlib::decompress` inflates at most one byte past the declared length**, so a WESYS
  wrapper cannot inflate more than it declares (the texture reader's bounds were bypassed by
  the unwrap in front of them).
- **Legacy bump-map headers are refused** (`DDPF_BUMPDUDV`, `DDPF_BUMPLUMINANCE`: signed
  components), and **`BC4U` reads as BC4** (DirectXTex `DDSPF_BC4_UNORM`).
- **TIFF associated alpha is un-multiplied at 16 bits** for every source, then reduced by
  `image`; for 8-bit sources the result is byte-identical to an 8-bit un-multiply (checked over
  every pair), so there is one path.
- **The encoder borrows source mips and pads only unaligned ones**; its padded sizes are
  checked (`InvalidDecoded("dimensions")`).
- **Header size fields saturate**: `build_header` writes `u32::MAX` for an Argb8 pitch or a
  linear size the field cannot hold, instead of panicking (debug) or truncating (release); this
  replaces the 2026-09-24 truncation gap. Readers derive the size from the dimensions.
- **Method (user decision):** the review loop has no round ceiling (`AGENTS.md` "Second
  opinion"); this surface continues to a fourth round.
Plan: `libs/dds_convert.md` (TIFF row, the `decode` paragraph under "`dds_convert` API").

## 2026-09-26 - ftex, dds_convert - converge review, fourth round
Decision (agent, cross-family review at converge):
- **`read_layout` reads the DDS mip count from the count field alone** (DirectXTex
  `DecodeDDSHeader`), so a file with its levels but no `DDSCAPS_MIPMAP` bit keeps them;
  `dds_to_ftex` keeps pes-file-tools' caps-bit rule, the reference it is parity with.
- **A legacy header's `DDSD_DEPTH` with depth > 1 is a volume** (refused), as DirectXTex
  classifies it; it decoded as its first slice.
- **An uncompressed header's alpha mask needs `DDPF_ALPHAPIXELS` or `DDPF_ALPHA`**: DirectXTex
  matches RGB headers on the colour masks alone and decodes them opaque.
- **A block texture whose padded top mip exceeds 4 GiB of RGBA is refused**:
  `block_compression` 0.10.0 computes its output offsets in u32 (wrapped writes in release).
- **TGA**: a TGA 2.0 extension with attributes type 4 (premultiplied) is un-multiplied; 16-bit
  TGA is refused with a resave message (`image` drops the alpha bit, DirectXTex keeps it; a
  hand decoder for a format no export is known to use was not worth its code). A 16-bit TGA
  whose alpha bits are all clear decodes opaque in both, and is refused anyway.
- **Float TIFF associated alpha is un-multiplied in f32**; integer sources keep the 16-bit path
  (the f32 path differs from it on 98 of the 8-bit pairs, so the integer bytes stay as they were).
- **Method (user question):** round four ran twice on the same surface, a reviewer resumed from
  round three and a fresh one; the resumed one found four, all among the fresh one's seven.
  Reviewers stay fresh (`AGENTS.md` "Second opinion").
Plan: `libs/dds_convert.md` (TGA and TIFF rows, the `decode` paragraph under "`dds_convert` API").

## 2026-09-26 - ftex, dds_convert - converge review, fifth round
Decision (agent, cross-family review at converge):
- **TGA decodes under `image`'s default allocation limit** (the round-4 direct decoder had
  dropped it), **interleaved TGA is refused** (DirectXTex refuses it; `image` decodes storage
  order), and **a TGA whose alpha is 0 in every pixel reads opaque**: texconv's default
  (without `-tgazeroalpha`), and the common "unused alpha" TGA would otherwise encode invisible.
- **`convert` refuses a caller-built `Decoded` past 4 GiB of top-level RGBA**
  (`InvalidDecoded`): mip generation and the block codecs index in u32.
- **`dds_to_ftex` checks frame sizes and offsets against their u32 fields**
  (`HeaderFieldOverflow`), untested by construction (a frame area past 4 GiB).
- **The DDS alpha-mask gate runs before classification**, so an undeclared alpha mask no longer
  keeps an L8 header from reading as R8.
Plan: `libs/dds_convert.md` (TGA row, the `decode` paragraph under "`dds_convert` API").

## 2026-09-26 - ftex, dds_convert - converge review, sixth round (loop ends)
Decision (agent, cross-family review at converge):
- **Allocation lengths derived from texture dimensions are refused past `isize::MAX`**
  (`Unsupported`): on wasm32 a 2 GiB buffer passed every earlier check and panicked with a
  capacity overflow. Mip generation keeps its infallible signature: its levels are smaller than
  a top level that already exists in memory, under `validate`'s 4 GiB cap.
- **ARGB8 and luminance-R8 classification match the bit count** (DirectXTex matches bit counts
  before masks): a 24-bit header with ARGB8 masks read as 32-bit pixels.
- **BC7 presets**: on fully opaque pixels `opaque_basic` and `alpha_basic` emit identical
  blocks (measured on the fixture and on random opaque inputs); the opaque branch stays because
  it does less work. The branch is tested through its translucent arm, where they differ.
- **JPEG, BMP and WebP are decoded in tests** against Pillow's decodes (lead fixtures,
  `fixtures_raster_formats.py`); JPEG within 2 per channel.
- **Loop result:** six rounds, 7/7/7/7/5/4 accepted (round 4 the union of a resumed and a fresh
  reviewer; round 5 plus one lead finding). The surface closes under the accept-rate rule.
Plan: `libs/dds_convert.md` (the `decode` paragraph under "`dds_convert` API").

## 2026-09-27 - fox2 - converge rework: write refusals, one byte reader, double text as the reference writes it
Decision (agent, lead audit at converge):
- **`write` refuses what it cannot encode faithfully**: `TooLarge` for a count or size past its
  field width, `KeyCount` for `StringMap` keys that are not one per value and for keys on any
  other container (silently dropped before), `ZeroTableHash` for a table entry with hash 0 (an
  early terminator). Read-side `UnexpectedConstant` carries an `i64` (every checked field fits).
- **A present but empty float attribute is `BadValue`**; a missing one still reads `0.0`. The
  reference's parse raises on the empty string; defaulting it hid a generator bug.
- **Double text follows the reference's Python `repr`**, not the plan's "17 digits" (the XML the
  reference writes is the parity target); pinned by a lead golden, `double_golden.tsv`.
- **Dead guards and branches removed**: the per-property value-count pre-checks (the counts are
  `u16` and the vectors grow only by successful reads, so they guarded no allocation), the empty
  branch of the 0-16 byte hash (its input always ends with the NUL `hash_string` appends), the
  unused `binrw` dependency, `parse_float`.
- **Lead goldens**: five non-repetitive 129-257 byte rows in `hash_golden.tsv` (every row past
  100 bytes was a run of `x`, so a wrong block offset in the above-64 loop passed), and raw
  `city_hash64` values for the 1-3 byte branch, from the reference.
Why: the whole-crate mutation run left 47 survivors; each was a missing test, a re-read inside an
error branch, or dead code, and the sweep found duplicated byte readers, `_ =>` arms on `Values`
and `as` casts. After the rework: 431 mutants, 0 missed.
Plan: `libs/fox2.md` (the `Container` block, the `resolve`/`write` paragraph, double text, the
`from_xml` defaults and trailer slack); `stadium_compiler.md` "New formats to port" (the stale
`cityhash`/`binrw` sentence).

## 2026-09-27 - fox2 - converge review, first round; inputs are trusted
Decision (cross-family review at converge, and the maintainer):
- **Inputs are trusted, not hostile** (maintainer): hardening against deliberately crafted files
  is not a goal; review concerns that need one are rejected. Two were (resolve's allocation
  amplification, a NUL in table text).
- **A `<value>`'s text is all its text children** with comments dropped, as the reference's
  parser reads it; `node.text()` had stopped at the first comment.
- **Floats are read through a double and narrowed**, the reference's double rounding: a 17-digit
  text otherwise compiles one ULP away from the reference's bytes.
- **Double text rounds ties to even** (built on exact-precision formatting), as `repr` does.
- **The table deduplicates on hash and text**, so two literals colliding in 48 bits both get an
  entry (tested with a real collision, `c16803888`/`c21237791`).
- **The hash follows CityHash where the reference does not**: the reference leaves one sum in
  the 33-64 byte branch unmasked, which changes the hash of some non-ASCII strings; the game
  and the C# tool wrap. Two golden rows hold the wrapping values.
Plan: `libs/fox2.md` ("Strings are hashes", "String table", the `from_xml` paragraph);
`AGENTS.md` "Fundamental concepts".

## 2026-09-27 - fox2 - converge review, second round
Decision (cross-family review at converge):
- **Blank text where the reference parses strictly is an error**: `classVersion`, `unknown1`,
  `unknown2`, and whitespace-only float/double value text. Missing attributes, a value with no
  text, blank `addr` and blank integer values still read 0, as the reference's lenient integer
  parse does.
- **Double text takes the digit count from the shortest round-trip form and prefers the
  ties-to-even text at that count when it reads back**; below a power of two the nearest text
  may not, and `repr` prints the next one up (`2^-24`, golden row 30).
- **An empty entity list writes `<entities />`**, the reference's self-closing element.
- Tests added for dynamic properties through XML, table-over-dictionary precedence and the
  opaque-entry skip.
Plan: `libs/fox2.md` (the XML layout paragraph, double text, the `from_xml` defaults).

## 2026-09-27 - fox2 - converge review, third round (loop ends)
Decision (cross-family review at converge):
- **A finite float text past `f32::MAX` is `BadValue`** (the reference's packing raises; the
  narrowing had made it infinity silently); `inf`/`-inf` texts still read as infinities.
- **WideVector3 `a`/`b` present but blank are `BadValue`**, like the other strict integers.
- **Loop result:** three rounds, 5/5/2 accepted; the surface closes under the accept-rate rule.
Plan: `libs/fox2.md` (the `from_xml` paragraph).

## 2026-09-27 - workflow - a lead recommendation is applied and logged, not asked
Decision: when the lead has a clear recommended choice for a point `AGENTS.md` lists under
"Stop and ask", it applies the choice, logs it (plan edit, decision entry) and names it in the
turn report; it asks only when the choice is truly ambiguous or needs information only the
maintainer has. New dependencies and `unsafe` still need a yes first.
Why: the maintainer's instruction at 2.20f, after three converge questions that each carried a
recommendation he accepted: asking costs a round trip and adds nothing when the answer is
already the lead's recommendation, and the report keeps every such choice reversible.
Plan: `AGENTS.md` "When the plan has gaps".

## 2026-09-27 - fmdl - converge audit: custom boxes, one `Model::validate`, a narrower surface
Decision (lead audit at 2.20f, the first two confirmed by the maintainer):
- **`Custom-Bounding-Box-Meshes` follows the add-on both ways**: a marked mesh reads back with
  its group's box as `custom_bounding_box`; `to_file` emits the header for meshes with one and
  uses the custom box in place of the mesh's vertex extent when it computes a group box. Before,
  the field was never filled or written anywhere, the IR's copy included.
- **`Model::validate`** is the one list of model invariants (index ranges, parent cycles, face
  indices, attribute lengths). `from_file` ends with it, `to_file` and every op start with it;
  `antiblur::encode/decode` and `vertex_enc::decode`/`decode_model` become fallible (they
  panicked on a dangling index, round C's open item (b)); `to_file` also refuses a mesh in two
  groups or none, as `from_file` does.
- **Codec internals `pub(crate)`**: `FmdlContainer`, the vertex-attribute layout types and
  `FmdlFile::vertex_attributes`, `FmdlFile::string`, split's limits, `needs_splitting`,
  `effective_parents`. No crate, binding or plan section calls them.
- **Per-object extension headers other than the four known ones are dropped** on read; unknown
  `X-FMDL-Extensions` flags stay in `Extensions::other` (already the code; the plan block lacked
  the field).
Why: the add-on is the only writer of the header and its semantics are the only evidence; a
dead field in two plans is worse than either implementing or removing it, and implementing
keeps hand-set culling boxes through a rewrite. Five partial copies of the index checks had
already drifted (antiblur had none), so one function is the fix, not a sixth copy. Unknown
headers carry mesh indices every op renumbers, and nothing known writes one. The plan said
`to_file` writes a box per mesh; the format has none.
Plan: `libs/format_crates.md` "Layout of the format crates" (bindings surface), "`fmdl::model`"
(the `Extensions`/`validate` block, "One list of invariants", "Custom bounding boxes", the
`to_file` box sentence).

## 2026-09-27 - fmdl - converge review, first round
Decision (cross-family review at converge, 7 of 7 accepted):
- **`from_file` accepts repeated assignments to one group naming the same bounding-box id**
  (a different id stays an error): `to_file` writes one assignment per run of a
  non-consecutive group, and a split leaves exactly that, so the writer's output did not reload.
- **`format/` owes byte identity on Konami's layout, not the add-on's** (every section-0 block
  padded to 16): a plan correction, the behavior since 2.6a-1.
- **`encode_vertices` refuses unequal values for uv maps that share storage**, which silently
  lost an edit to one of them.
- **A caller-supplied split hierarchy gets `effective_parents`' cycle cut**; a cycle hung the
  subtree climb, and the IR exporter's render-parent fallback can build one.
- **A shared bone's bounding box in a merge is the union of the parts' boxes**, not the first
  part's.
- **An anti-blur duplicate keeps its source's custom box**, as the add-on copies the header.
- Deferred with an issue: u16 faces cap a reassembled split mesh at 65536 referenced vertices.
Plan: `libs/format_crates.md` ("Layout of the format crates" byte-identity rule, "Custom
bounding boxes", new "Mesh-group assignments"); `model_conversion/ir.md` (merge sentence).

## 2026-09-27 - fmdl - converge review, second round
Decision (cross-family review at converge, 5 of 6 accepted):
- **`antiblur::encode` is a no-op when `extensions.antiblur` is set**; idempotence no longer
  rests on a duplicate sitting right after its source, which a split breaks.
- **The split key is position plus the positive-weight bone mapping**, the identity `combine`
  matches on; with raw bone-index lanes a zero-weight lane could split coincident vertices
  apart and the decode weld them back.
- **`combine` reorders vertices only when a referenced index would pass u16**, so a loose loop
  stays next to its owner.
- **`vertex_enc::encode` refuses an owner map joining different topological keys** (it
  panicked).
- **The principal axis is found from all three basis starts, keeping the largest Rayleigh
  quotient**: a single start on the largest diagonal can sit on a non-principal eigenvector,
  which the 2026-09-13 decision already ruled out.
- Rejected: merging a loop-flagged part with an unflagged one reads the unflagged part's
  convention runs as loops; `model_conversion/ir.md` "Extension algorithms" settles that.
Plan: no plan edit needed: these are implementation rules inside `fmdl::ops` the plan leaves
open, and the axis rule restates the 2026-09-13 entry.

## 2026-09-27 - fmdl - converge review, third round
Decision (cross-family review at converge, 7 of 7 accepted):
- **merge refuses a part with repeated bone names** (`MergeError::DuplicateBoneName`; union by
  name is ambiguous there, and it panicked) **and parts mixing encoded and unencoded anti-blur
  requests** (`MergeError::MixedAntiblur`); the merged `antiblur` flag is set only when every
  part that requests anti-blur is encoded, so `antiblur::encode` never skips a part that still
  needs duplicates.
- **`split::decode` combines before it mutates**: a failed combine leaves the model intact.
- **Split's principal axis is computed over points in sorted face/loose order**, so the
  partition does not depend on `HashSet` iteration order.
- **uv map sharing and the aliased-map guard compare bit patterns**, so -0.0 and +0.0 stay
  distinct bytes.
- **A zero-vertex mesh keeps its declared attribute layout on decode**; **vertex-loop encoding
  refuses a face index past u16 after reordering** instead of wrapping it.
- Lead, found while reviewing the rework: **a computed group box with nothing to measure (only
  zero-vertex meshes, no custom box) is the zero box**, the add-on's value; it was written as
  +/- infinity.
Plan: `model_conversion/ir.md` "Extension algorithms" (merge sentence: anti-blur state).

## 2026-09-27 - fmdl - converge review, fourth round (loop ends)
Decision (cross-family review at converge, 4 of 4 accepted):
- **The vertex-loop key is position plus the positive-weight bone lanes**; a zero-weight lane,
  which a split rewrites, no longer decides whether two vertices are loops of one.
- **A computed group box also covers its child groups' boxes**, as the add-on's does; a group
  with neither meshes nor box still gets no assignment record.
- Tests: merging genuinely encoded parts, split round trips compared as face multisets.
- **Loop result:** four rounds, 7/5/7/4 accepted; the surface closes under the accept-rate
  rule.
Plan: `libs/format_crates.md` "`fmdl::model`" (the `to_file` box sentence).

## 2026-09-27 - studio - the executable is `4cc-studio`
Decision (maintainer): the shipped executable is `4cc-studio` (`4cc-studio.exe`), set by the
`studio` crate's `[[bin]] name`; CLI examples in the plans follow. The crates keep `studio` and
`studio_core`.
Why: the binary ships standalone and lands in download folders, where a bare `studio.exe` says
nothing about what it is. Renaming the crates would change nothing a user sees, and a crate name
cannot start with a digit anyway.
Plan: `core/architecture.md` "Crate structure" (naming bullets); CLI and file-name mentions across
`core/`, `team_compiler/`, `db_generator.md`, `refs_arranger.md`, `player_aesthetics_editor.md`,
`match_tracker/match_feed.md`, `GLOSSARY.md`.

## 2026-09-27 - archives - password-protected archives are refused at open
Decision (maintainer): `Archive::zip` refuses a zip with any encrypted entry and
`Archive::seven_z` a 7z whose header is encrypted or whose blocks carry the AES coder, both
with `Encrypted` at open; `read` no longer meets encryption.
Why: at open, the Team compiler's live shallow check reports a password-protected export the
moment it appears; refusing at `read` postponed it to the compile-start deep check. Passwords
are never prompted for, so no archive with an encrypted part can ever be compiled anyway.
Plan: `libs/archives.md` (the block, the encryption paragraph, the fixtures paragraph).

## 2026-09-27 - method - maintenance mode is a Phase 16 deliverable, not ADRs
Decision (maintainer): no per-file ADRs; `DECISIONS.md` already is the ADR log and the plans
the spec. What is missing is the procedure once the project is released, planned as a Phase 16
deliverable with its outline recorded now.
Why: per-file ADRs would add a second home for decisions the plans already carry through the
`Plan:` field, and two homes drift. A procedure written now, far ahead of its use, would go
stale, the same reason Acceptance sections are written just in time.
Plan: `core/development_plan.md` "Phase 16" (maintenance-mode bullet).

## 2026-09-27 - team compiler, fmdl, pes_model - geometry far from the origin is refused
Decision (maintainer): a model with a vertex more than 5000 units from the origin is an Error:
`fmdl::check` and `pes_model::check` report it per mesh (`fmdl_vertex_far_from_origin`,
`model_vertex_far_from_origin`), and the Team compiler maps both to `vertex_too_far_from_origin`
("Vertex too far away, it will cause persistent lag for the whole matchday"), dropping the folder.
Lead, applied as recommended: the check runs on every model in its target-format form before any
merge, so glTF and converted sources are covered and the finding names the source file; the
distance is Euclidean with 5000 itself passing; only vertices are tested, since edges and faces
lie within their vertices' reach; the finding is not pass-through-eligible, since the lag
persists for the whole matchday.
Why: far-away geometry makes the game lag persistently for the whole matchday; a warning or
pass-through would let one export degrade the rest of it.
Plan: `libs/format_crates.md` (the check bullet), `team_compiler/messages.md` (catalog row,
pass-through exclusions).

## 2026-09-27 - code style - no legacy tools or project history in code
Decision (maintainer): code comments, doc comments and identifiers do not name or allude to the
tools the suite replaces or mirrors ("the reference", "the add-on", "the 4cc compilers", "has
always produced"); parity tests, literals a file carries and fixture provenance are exempt.
Why: the history is evidence the plans hold; in code it goes stale and describes another tool
instead of the format. The earlier rule named only a few tools and was widely missed.
Plan: `CONTRIBUTING.md` (the rule), `AGENTS.md` (design sweep).

## 2026-09-28 - development plan - 0.1.0 ships at the end of Phase 8; glTF and GPU BC7 after it
Decision (maintainer, from the lead's feasibility evaluation): the suite's first release is
0.1.0, cut after Phase 8: Team compiler with compile-time savefile writing, Save editor (full
Phase 8 view), Export upgrader (structural migration), Windows only, CPU texture encoding, native
model formats only. Phase 7 (glTF) runs after it (order 1-6, 8, 0.1.0, 7, 9-16, numbers kept);
GPU BC7 (worklog 4.x, now 16.x) and Linux move to Phase 16; the updater ships as check + notice.
Red's Pre-Studio preview layout gets no dedicated input path (maintainer: few members will use
it). Lead, applied as recommended:
- **Pulled forward from Phase 16:** the Windows `just release` recipe, `CHANGELOG.md`, help
  chapters for the shipped tools, the update check and notice, and the maintenance-mode section,
  because from 0.1.0 on the released surfaces are compatibility contracts; `0.x` does not loosen
  that rule. Releases stay `0.x` until Phase 16 closes with 1.0.0; "first release" in the plans
  means 1.0.0, so the "First-release desktop GPU BC7" headings keep their names and pointers.
- **The teams-list merge ships in 0.1.0** with the check: it runs on a new binary's first start
  however the binary arrived, and without it a manually updated install never receives new
  upstream teams.
- **glTF sources are refused, not skipped, until Phase 7:** a folder whose selected
  representation would be glTF is dropped with an error instead of compiling from the opposite
  native format, since that output would change silently once Phase 7 selects the glTF.
- **The first-run data-location dialog** joins Phase 8's shell bullet; no phase listed it.
Why: Phase 8 is the first point with both main tools and a GUI, and its verification (community
feedback) needs a distributed build; every later tool replaces a legacy tool that keeps working.
glTF has no current export using it and depends on the Blender codec outside this workspace; the
CPU encoder already meets the texture quality checks. Rejected: keeping Phases 3-8 whole
(a later release for work no 0.1.0 user needs) and trimming the Save editor view (maintainer
kept it).
Plan: `core/development_plan.md` ("Releases", "Phase 2", "Phase 4", "Phase 7", "Phase 8",
"Release 0.1.0", "Phase 16"), `core/distribution.md` ("Self-update: in-place binary swap",
"Versioning"), `libs/README.md` "First-release desktop GPU BC7", `team_compiler/README.md`
"First-release GPU BC7", `team_compiler/pipeline.md` (format conversion step). Supersedes the
Phase 4 placement in the 2026-09-21 GPU BC7 entry.

## 2026-09-28 - settings.toml - stock boots/gloves IDs are authorable, `""` is default
Decision (maintainer): `settings.toml` gains top-level `boots_id` and `gloves_id`, for players
who wear one of the game's own models and so have no folder to express it. Accepted values: 0 to
100 (the stock band: the cup's stock kit always compacts Konami's boots and gloves into it, and
the compiler's per-team blocks start at 101) or `""`, meaning default, the same as an absent key.
Both keys appear in the template uncommented as `""`. Per category: folder models or a link win
(a numeric key is then ignored with `settings_model_id_conflict`, W); else a numeric key is
written; else the FPC marker decides (`fpc.on`: the hide preset's 55/11, `fpc.off`: 0/0), and
with no marker the savefile's IDs stay unchanged, so an FPC player needs no key at all.
Lead, applied as recommended: the keys sit at the file's top level beside `name`, not in
`[appearance]`, because Team TOML embeds the `[appearance]` tables beside its own player-level
full-range IDs; a numeric key wins over an FPC marker's preset ID (an explicit choice, as a
custom model's ID already does); generators (`from_player`, the Export upgrader, the save editor)
emit a stored ID only from 1 to 100 (0 is the game default, above 100 is custom content a folder
owns) and leave FPC-detected players at `""`; the upgrader reports a custom ID no export folder
claims instead of writing it.
Why: the earlier rule (boots/gloves IDs never authored) left stock-model players with no way to
state their boots, which only survived as long as nobody rebuilt the save.
Plan: `aesthetics_export/settings_toml.md` (the rule, the precedence, the template),
`aesthetics_export/fpc_toggle.md` (precedence table), `aesthetics_export/player_folders.md` (ID
space), `pes_savefile/model.md` (`PlayerSettings`, `from_player`), `pes_savefile/operations.md`
(Team TOML, aesthetics patch), `pes_savefile/verification.md`, `export_upgrader.md` step 8,
`save_editor.md` (generation), `team_compiler/messages.md`, `team_compiler/pipeline.md`
(aesthetics patch), `core/README.md` (decisions table), `GLOSSARY.md`. Supersedes the
compiler-owned classification of boots/gloves IDs in the 2026-09-12 aesthetics patch entry and the
2026-09-19 `settings.toml` key table entry.

## 2026-09-28 - boots/gloves - gloves ID 0 is a set of normal hands
Decision (maintainer, fact): gloves ID 0 is a pair of normal hands, not "no gloves": the cup's
gloves system lets any player wear a customized gloves model, which required 0 to render plain
hands. So `fpc.off` writing gloves 0 shows normal hands, gloves are not goalkeeper-only anywhere
in the plans, and the `settings.toml` template says "0 = normal hands".
Why: recorded so no later rule treats gloves as a goalkeeper field or 0 as an absent model.
Plan: `aesthetics_export/settings_toml.md`, `libs/fpc.md`, `save_editor.md` ("Appearance", FPC),
`GLOSSARY.md` ("Stock band"); code doc comments in worklog step 2.17i.

## 2026-09-28 - pes_model - the reader follows community files the census found
Decision: `PreFoxModel` resolves every section's offsets against the unwrapped file from the
section's start to the file's end (not to the next section); drops an annotation whose string
or non-zero record pointer lands off a section-2/3 record; lets a zero-length read point
anywhere; takes the first descriptor of each vertex-field type and ignores later repeats; and
`Model::from_file` repairs faces an old exporter shifted past loose vertices when the distinct
indices number exactly the vertex count (rank mapping), any other out-of-range index staying
`BadReference`. The container's byte identity is unchanged.
Why: a census of every `.model` on the maintainer's machine (6575 distinct files, the method
added to converge at 2.20g) found about 1900 community files, cup exports among them, that the
Konami-measured reader refused: 1640 with data past their section, 228 with template annotation
pointers, and a handful per remaining class. Each rule matches how the reference parser and the
game read those files; the repair is the reference importer's, the only order-preserving reading
that uses every vertex, and the files have no faithful reading without it (lead recommendation,
act-then-log per the maintainer's standing instruction).
Plan: `libs/format_crates.md` "`pes_model::format`" (community rules) and "`pes_model::model`"
(the repair; replaces the "known gap, to decide at converge" sentence).

## 2026-09-28 - model_convert, fmdl, pes_model - conversion follows what real files carry
Decision: the IR accepts unnormalized weights (finite, non-negative; above 1 kept, clamped to 1
only by the `u8` FMDL export with a `weight_clamped` finding past float noise); `fmdl_to_ir`
reorders bones parent-first as `model_to_ir` does; a `Split-Mesh` group whose combined mesh
would exceed 65536 distinct vertices stays split (both format crates' decode), its components
imported as separate meshes; split encode never emits an empty container; an FMDL
`Split-Mesh-Groups` entry is a container only when the group has a parent and a mesh; the
pre-Fox export refuses a material name XML cannot carry; a vertexless mesh is written to FMDL
with an empty bone group; `.mtl` reading accepts `maxfilter`, ignores content after the root
element, and every reader takes a repeated state's first value.
Why: a conversion census (every readable `.model` bundle and FMDL on the maintainer's machine
converted to the other engine and back) failed on about one file in six, each class traced by
measurement (weights 879 files, 80k-360k-vertex split groups 114, forward parents 79 with no
cycle, over-flagged groups 4, our own empty split container 12, NUL names 4) and a `.mtl`
census found the three reader classes (11 files) and no repeated state anywhere. Each rule
keeps what the file renders as, or refuses loudly where nothing faithful exists; the split rule
avoids changing the IR face type before Phase 7 (lead recommendation, act-then-log).
Plan: `model_conversion/ir.md` "What real files carry", `model_conversion/conversion.md` "mesh
splitting" rules, `model_conversion/README.md` (validate), `libs/format_crates.md` (`.mtl`
census rules).

## 2026-09-28 - pes_savefile - a text field holds its full length; a non-UTF-8 name is refused
Decision: a savefile text field holds up to `len` bytes: a text that fills it is written with no
NUL, only a longer one is refused (codec write, Team TOML's `text_max`, `convert`'s cuts). The
generated `shirt_name_from` keeps its one free byte. A player name that is not UTF-8 stays a
read refusal, with no lossy or relaxed mode.
Why: the savefile census found 6 PES 16 saves with 16-byte shirt names and 3 PES 17 saves with
46-byte names, none NUL-terminated; with `len - 1` we read them and refused to write them back,
and cutting a character (what 4ccEditor does) renames the player on the first save. Carried
text is therefore written as found; invented text stays conservative until a full field is seen
displaying in game. The one non-UTF-8 save (a CP1252 `£` in PES 17) does not load in 4ccEditor
either (maintainer, 2026-09-28), so no save in use carries one, and a lossy decode would write
back a different byte (lead recommendation for the length rule, act-then-log; maintainer for the
refusal).
Plan: `pes_savefile/codec.md` "Text fields hold `len` bytes" (new) and the `TextSpec` block,
`pes_savefile/model.md` field comments, `pes_savefile/operations.md` `shirt_name_from`,
`pes_savefile/verification.md` "Name fields".

## 2026-09-28 - pes_model - the 21845-face limit stays an Error
Decision: `check`'s `model_mesh_over_face_limit` keeps 21845 faces per mesh as an Error, although
193 community `.model` files (cup exports among them) go past it.
Why: the current modelling add-on enforces the limit; the files over it were made with a much
older exporter, and the test case (Bedford Rascal, about 700 faces over, most out of sight)
cannot show missing faces in game, so the in-game check cannot overturn the tool (maintainer,
2026-09-28). `ops::split` still takes such meshes apart.
Plan: `libs/format_crates.md` "`pes_model::model`" ("One list of invariants").

## 2026-09-28 - pes_savefile - no second whole-crate mutation run
Decision (maintainer): the whole-crate run at `cd3a1d1` is the last one over `pes_savefile`; its
converge closes on `just mutants-diff cd3a1d1` over the rework instead of the second whole-crate
run `AGENTS.md` "Closing a phase" asks for.
Why: the crate is about 27k lines, and the run at `cd3a1d1`, split with the VPS, was still going
after more than two hours (the first attempt was stopped at 87 minutes, half done). The diff run measures
every line the rework touches, which is where a closing run's new survivors would come from.
Plan: no plan edit needed (process, not spec); worklog step 2.20i records it.

## 2026-09-29 - pes_savefile - the cd3a1d1 run's unmeasured mutants run once
Decision (maintainer): the mutants the `cd3a1d1` whole-crate run never measured (486 at the
slice A tree) run once, split with the VPS, and their survivors are triaged before 2.20i
closes. It is not a second whole-crate run; the 2026-09-28 entry stands otherwise.
Why: that run measured 1130 of 1611 mutants: its VPS half died at 324 of 805 when ssh
dropped. Slice A's run over the survivors' functions found real gaps in that unrun tail (the
PES 16-21 and PES 15 header bounds), so the rest is not assumed covered.
Plan: no plan edit needed (process, not spec); worklog step 2.20i records it.

## 2026-09-29 - python_bindings - 2.20j converge: plan text, mutation method, no census
Decision: (1) the plan's "outside the default `cargo build` path" is corrected to what the
2026-09-21 shape made true: a workspace member that a bare root `cargo build` compiles, that
`cargo build -p studio` does not, and that `cargo test` never links. No `default-members`.
(2) The crate's mutants are measured by hand against `just bindings`, and `.cargo/mutants.toml`
excludes it from cargo-mutants. (3) No census through the wheel. (4) A justfile recipe
parameter goes through `quote()`.
Why: (1) `default-members` would have to list every other member by hand, and a lib crate
missing from it would drop out of a bare `cargo test` without an error; `just gates` and CI pass
`--workspace` anyway, and the build compiles fine where Python is installed, which every
developer of this repo needs for its scripts. (2) cargo-mutants runs `cargo test`, which never
compiles a `test = false` lib: all 46 mutants came back "missed" in 0 s, `--check` passed
uncompilable ones. The hand run (11 mutants, one per behavior, `.tmp/pb_mutants/run.py`)
caught 10; the survivor drops `pyo3_log::init()`, which nothing observes while `fmdl` and
`pes_model` do not log (the plan installs it ahead of that on purpose). (3) The shim hands the
bytes to the codecs unchanged, and those codecs' censuses (2.20f-g, 2.20g-fmdl) already ran
every file on the machine through the same functions. (4) An interpreter path with a space
(Blender under `Program Files`) was split into several arguments; `quote()`'s single-quoted
literal is one argument in sh and in PowerShell.
Plan: `core/architecture.md` (`python_bindings` paragraph), `core/distribution.md` "One
exception", `core/development_plan.md` "Phase 2" (`python_bindings`, the abi3 tag check);
`CONTRIBUTING.md` justfile rules.

## 2026-09-30 - pes_savefile - 2.20k: `.4ccs` records map to the target's record order
Decision: `read_squad` takes the target team and its save's players, and gives `.4ccs` record
*k* to the roster slot of the *k*-th rostered player in the target's player-record order
(shirt number *i* still to slot *i*). A file with more records than the target has rostered
players is the new `LegacyError::MoreRecordsThanRoster`.
Why: the plan said the records are in roster order; 4ccEditor's `export_squad` writes them
in the order of its player array, which is the save's record order, and `import_squad` reads
them back the same way. The two orders differ on 22 of the 27 distinct PES 16 saves on the
machine and on 438 of 508 teams of two PES 18 saves, and there record *k* to slot *k* puts
every stat and appearance on the wrong player. The record has no id, so the target is the
only thing that can place it. The file is not made source-relative instead because the
source save is not available when a cup member's `.4ccs` is imported.
Plan: `pes_savefile/operations.md` (the `legacy.rs` block and "`.4ccs` squad files"),
`pes_savefile/verification.md` ("Cross-implementation parity", "Interchange formats").

## 2026-09-30 - process - Reviewer critiques have no concern cap
Decision (maintainer): a cross-family reviewer returns every concern it finds, ranked by
reachability; the seven-concern cap per round is gone. A round still runs again only when at
least five of the last round's concerns were accepted, and a reviewer brief names no number of
concerns, as a cap or a target.
Why: the cap made reviewers curate instead of report. 3.1's first round set out, in its own
reasoning, to pick "seven issues that a parent will likely accept, ideally five or more", so
concerns it judged less likely to be accepted went unreported, with nothing guaranteeing a later
round finds them. The accept-rate gate already bounds the loop and catches padding; the
reachability ranking already keeps impossible edge cases at the bottom.
Plan: `AGENTS.md` "Second opinion" (supersedes the cap in the 2026-09-10 methodology entry).

## 2026-09-30 - workspace - 3.2: the acceptance-ID scanner is a Python script and gate 5
Decision: `scripts/acceptance.py` (with its tests in `scripts/acceptance_test.py`), run by
`just acceptance [report|strict]`; `report` is a fifth `just gates` gate, so CI and every
local gate run execute it. Beyond orphan citations, `report` fails on the other ways a citation
or a definition proves nothing: a citation of a withdrawn scenario, an ID-only comment not
directly above a `#[test]`, a manual check of a scenario not marked `manual`, a duplicate ID,
and an ID line in an Acceptance section that is not a scenario. `strict` checks every scenario
in the plans, not only the closing phase's, and collapsing a phase's worklog steps keeps its
`manual: checked` lines.
Why: Python, not a Rust bin, because the scanner reads Markdown and source text and needs no
workspace code; a bin would add a crate to build for a text scan, and the other repeatable
sequences are already Python scripts under `scripts/`. A gate, not only a CI job, because a
check outside `just gates` goes stale unseen (`just bindings` was red from 2.20g-h's fixtures
until 2.20j noticed). The
extra failures are each a way a scenario would silently count as proven or silently vanish
from strict. Strict over everything needs no phase marker in the plans: Acceptance sections are
written just in time, so every scenario at a converge belongs to the closing phase or an
earlier one, and an earlier proof that disappeared is a regression; that holds only if the
earlier phases' manual proofs survive the collapse.
Plan: `CONTRIBUTING.md` "Testing and verification" (gate list, citation form, the scanner's
rules, manual proofs kept at collapse); `AGENTS.md` "Closing a phase" step (3).

## 2026-09-30 - team_compiler - 3.3: the tracer fixture, its committed Red tree, the bundled bases
Decision: the tracer fixture (`crates/tools/team_compiler/tests/fixtures/tracer/`) is /egg/'s
"The Chad Stormworks Player" (face, boots, gloves, portrait) plus kit `g1`, cut from the
maintainer's copy of the EGG VGL26 export (maintainer: a cup export, a player with folders in all
three of Faces, Boots and Gloves, not the Test_stuff export). The old-layout source carries the
whole player; the Studio twin carries the face and the kit now, and gains the boots, gloves and
portrait in Phase 4 (maintainer's choice), since Fox boots and gloves need Phase 4's ID
assignment. Red's output is committed as its extracted tree, not as a hash manifest. The
`UniformParameter` fallback bases are Red's own, in `resources/bins/`.
Why: kit `g1`'s 312-byte main texture keeps the fixture at 1.3 MB where a player kit's 5.5 MB
texture would quadruple it. A manifest would make the parity case depend on a local Red install,
so it could not run in CI; the manifest scheme exists for multi-GB trees, and this one is under
1 MB. Red's bases, not the game's installed bins, because a from-scratch compile has nothing
installed to read and must build on the same base as Red to be comparable.
Plan: `team_compiler/testing.md` "Testing: parity against Red" (a small tree is committed);
`team_compiler/pipeline.md` "Resolved decisions" (the fallback bases' location and per-version
choice); the fixture's `README.md`; `resources/bins/README.md`.

## 2026-09-30 - team_compiler - 3.3: FTEX parity is by decoded content
Decision: the parity test compares an FTEX entry by the DDS `ftex::ftex_to_dds` returns for each
side (tier 2), not by its bytes; `ftex` keeps its `flate2` encoder.
Why: the tracer's `shirt.ftex` and `u0792g1.ftex` differ from Red's only inside the zlib-chunk
sizes and streams (first difference at offset 0x48, the chunk table), while both convert back
to byte-identical DDS. Matching Red's bytes would mean reproducing one zlib build's deflate
output, which `ftex` already declared a non-goal; the game reads the pixels, not the stream.
Plan: `team_compiler/testing.md` "Testing: parity against Red", tier 2.

## 2026-09-30 - kit_config / ftex / team_compiler - 3.4: the tracer's Phase 2 frictions
Decision: (1) `kit_config` owns `KitSlot` (`p1`–`p9`, `g1`): its parse, the lowercase name the
texture names carry, and the config entry name `{team}_DEF_{1st…9th|GK1st}_realUni.bin`;
`texture_names` takes it, and `aesthetics_export` parses kit folder names into it rather than
defining its own. The team id stays a `u16` whose range is the caller's. (2) `ftex::FtexInfo`
keeps the raw `texture_type`, with no `ColorSpace` accessor. (3) The FTEX parity comparison also
compares `ftex::info` of both sides. (4) The CPK tool version is `4cc Studio <version>`.
(5) The player ID (`team * 100 + slot`, past `u16`) gets its type in 3.5's shapes.
Why: (1) both names are the game's, so they are format knowledge, and one parser means the
folder grammar and the emitted names cannot disagree; a `teams_list::TeamId` would add a
dependency to a crate whose standalone conversion has no teams list. (2) Nothing reads a texture's
color space until Phase 4's texture-role table; an accessor now would have no consumer. (3) The
decoded-DDS comparison alone cannot see the texture type (a kit written as sRGB still passed); the
header comparison fails it. (4) `CpkWriter::new` requires one, and a per-release string keeps
builds reproducible within a release.
Plan: `kit_config_editor.md` "`libs/kit_config`"; `aesthetics_export/object_model.md` "Design
constraints"; `team_compiler/testing.md` tier 2; `team_compiler/pipeline.md` "5. Writer" and step 2
of the model-folder tasks (which named a `fmdl_id_change` that does not exist).

## 2026-09-30 - aesthetics_export - 3.5: the structure pass shapes
Decision: the shapes in `object_model.md` "Structure pass types", with these departures from the
earlier blocks: `validate` returns a `ValidationReport`, not a `Result` (no `FatalValidationError`);
the draft is `AestheticsExportDraft` of `FolderDraft`s, and the validated `PlayerFolder`,
`SharedModelFolder` and `KitsFolder` are the descriptors (no `*Desc` types); portraits, collars and
common are plain collections (no wrapper structs); `KitFolder.icon` is the parsed number; a
`FileDescriptor` carries its source path beside its virtual one; `parse_listing` does the root
normalization; `ISSUE_CODES` is what the tool's catalog test iterates; `ExportSlot` moves to the
Team compiler's Phase 4 planning.
Why: every structural failure is already an issue with a disposition, so a fatal error type would
have no variant. One set of validated types serves both the draft's consumers and the compile,
and a wrapper around one `Vec` adds a name and nothing else. `icon.txt` is read by the structure
pass anyway, so re-reading it downstream would repeat the parse. Flattening changes virtual paths,
so reading needs the source's own. Root normalization is format knowledge every consumer of a
listing needs, not a compiler step. A code list lets the tool check its catalog without emitting
every issue; the consequence is observed by the scenario tests.
Plan: `aesthetics_export/object_model.md` "Core types", "Structure pass types", "Player folders",
the crate layout and "Validation semantics".

## 2026-10-01 - aesthetics_export - 3.6 (b): allowlist, OS artifacts, scopes, eligibility
Decision: (1) the per-position file-type allowlist in `object_model.md` "Validation semantics",
the same for every target; a file below an unnamed subfolder takes its holding folder's code;
`Collars/` unchecked until Phase 4. (2) `Thumbs.db`, `desktop.ini` and `.DS_Store` are left out of
the draft, silently. (3) Player and kit folder names share one `<head>[ - <label>]` split at the
first `-`. (4) An issue's scope names the item its disposition acts on, so `file_type_disallowed`
is `Folder`-scoped, one per file. (5) The structure pass's pass-through-eligible codes are
`link_target_missing`, `common_link_missing`, `file_type_disallowed`, `common_file_disallowed`.
(6) Drops cascade own findings → `link_target_dropped` → `shared_folder_orphaned`. (7) `validate`
reports `team_name_unknown` (empty name) and `export_empty` itself. (8) `common_link_missing`'s
context is the link file and the Common path, not a separate link kind. (9) The kit, portrait
and logo details in "Kits, portraits, logo": `all` is a slot for duplicates and obeys the `kit`
prefix, `kit_all_unused` is `Keep`, `Portraits/` is a stem namespace, `portrait_conflict` moves
to the deep pass, a logo finding drops the logo as one unit.
Why: (1) Red's per-engine lists (no `.fmdl` pre-Fox, no `.model` on Fox) predate conversion,
which makes every model format a source for either engine; a version-dependent list would refuse
inputs the plan converts. One fallback rule for subfolders covers `extra/`, `Common/sub/` and kit
subfolders without a case each. (2) A census of the maintainer's machine found 225 such files in
the model folders of real team exports; as `Other` they would drop each folder under the default
strict check, for a file no author wrote. (3) Players and kits used the same shape in prose; one
split rule means `03 - Jean-Pierre` and `p1 - Lakers` cannot parse differently. (4) A scenario
reports one finding per disallowed file while the folder is what goes. (5) The catalog's
not-eligible classes (identity, conflicts, unused content, contradictory directives) cover every
other structure-pass drop; a list of four is what the code checks. (6) TC-STR-08 and TC-STR-16
need a player dropped by a link to leave its other shared folders unreferenced. (7) The validated
export needs a `TeamName`, and an empty root is visible only after normalization. (8) The
link's own name (`torso.fmdl.common`, `hair.png.common`) already says what kind it is. (9) Two
`all/` folders would make inheritance a pick; an `all/` texture becomes a kit's texture, so it
needs a kit's name; an unused `all/` affects no output. `Portraits/` maps one file per slot, and
two of one stem cannot both be it. "Differing portraits" needs the bytes the structure pass never
reads. The game needs all three logo sizes from one source pair, so no partial logo exists.
Plan: `aesthetics_export/object_model.md` "Validation semantics"; `team_compiler/messages.md`
`common_link_missing`.

## 2026-10-01 — aesthetics_export — an invalid kit folder name gets no other finding
Decision: a kit folder reported `kit_folder_invalid` is not checked further (no allowlist,
texture-name, marker, stem or `icon.txt` findings for its contents).
Why: its head names no slot, so which allowlist applies is unknown (`Kits/alll/` holds `all/`
content, which a kit's rules would misreport), and the folder is dropped whole regardless; its
contents' findings would describe content that goes nowhere.
Plan: `aesthetics_export/object_model.md` "Validation semantics", "Kits, portraits, logo".

## 2026-10-01 — aesthetics_export — texture links and `Common/` join the stem check; `vtree::fold_name`
Decision: (1) a player folder's texture `.common` links count in its texture-stem namespace under
their linked name, and `Common/` is a namespace of its own whose conflicting files drop each other
(`File`, `DropFile`). (2) `vtree` gains `pub fn fold_name(name: &str) -> String`, the fold of one
name with no path validation, used for every name-level lookup key in `aesthetics_export`.
Why: (1) `model_format.md` makes a link count as the linked file being local, so `hair.dds`
beside `hair.png.common` is the same ambiguity as `hair.dds` beside `hair.png`; Common materials
resolve their stems against Common's images, which two files of one stem make ambiguous. Common
cannot be dropped as a folder without dropping every player that links anything in it, so its
files drop each other, as in `Portraits/`. (2) Folding a stem or a link target by building a
`ScopePath` from it panicked on real names (`skin .png` has the stem `skin `, which no path
segment may end with); the fold itself needs no validation.
Plan: `aesthetics_export/object_model.md` "Validation semantics", "Texture stems";
`team_compiler/messages.md` `texture_stem_conflict`.

## 2026-10-01 — pipeline — the Phase 3 contract: permit, oversized priority, 7z charge, `CpkStem`
Decision: (1) `MemoryBudget::acquire` returns a `Permit` that releases on drop, or `Cancelled`
once `cancel` has run; the core plan's `acquire`/`release` pair stays as the capacity sketch.
(2) While an oversized request waits for the pipeline to drain, new ordinary requests wait too.
(3) The structure pass charges a solid `.7z` export's full size while it reads the small metadata
and releases it with the `Archive`; `compile` charges it again for its tasks. (4) `CpkStem` judges
the reserved device names on the part before the first `.`. (5) A task acquires once, before
loading, until Phase 4 settles the growing permit.
Why: (1) a permit travels with its batch to the writer, and a drop cannot be forgotten on an
error path as a `release` call can; the plan requires cancellation to wake waiters, and a waiter
woken by it must not take bytes. (2) Without it, ordinary tasks arriving faster than others
finish keep `in_flight` above zero, and the oversized request never runs (the plan's
wake-up stress case). (3) Holding every 7z decompressed from the structure pass to its tasks would
make all of them resident while the run is planned. (4) Windows reserves `con.x` as it does
`con`. (5) Phase 3's tasks know their source sizes up front.
Plan: `libs/pipeline.md` (new part); `core/parallelism.md` "Memory budget" points to it.

## 2026-10-01 — pipeline — the memory cap is a share of the memory available at run start
Decision: `pipeline::memory_cap(percent)` reads the physical memory available when the run
starts (Windows `GlobalMemoryStatusEx` `ullAvailPhys` through the `windows` crate's
`Win32_System_SystemInformation` feature, one `unsafe` call; Linux `/proc/meminfo` `MemAvailable`)
and returns that share, with no minimum and a 4 GiB assumption where the OS cannot say. The
dependency change was approved by the maintainer on 2026-10-01.
Why: the available memory, not the total, is what the run can take without pushing the game, a
browser or Blender into swap. `MemAvailable` is the kernel's estimate including reclaimable
cache, where `libc::sysinfo`'s free memory omits the cache and undercounts, and reading a file
needs no `unsafe`. No floor: the oversized branch already keeps the budget moving at any cap, and
a machine short of memory should run one task at a time. Windows has no way to ask without an OS
call; a feature on the `windows` crate already in the workspace is the smallest one (a
`sysinfo`-style crate would add a dependency tree for one number).
Plan: `libs/pipeline.md` "Memory cap"; `team_compiler/settings.md` `memory_cap_percent`;
`CONTRIBUTING.md` `unsafe` sites.

## 2026-10-01 — studio_core / team_compiler — 3.8: exit codes, console lines, CLI data location, Phase 3 settings
Decision: (1) `StudioTool::cli_run` returns `Result<u8, CliError>`: `Ok` is the exit code of a
command that reached a verdict, `CliError { exit_code, error }` a command refused before one,
printed by the binary as `error: …`. (2) The binary prints one self-contained `-` line per
`Message` to stdout from its own thread (`<source>: <Severity> <code> at <scope> (k=v, …)`); the
source is the export's file name, extension included, from `ExportStarted`. (3) `ToolContext`
gains `paths()` (`AppPaths { exe_dir, data_dir: Option }`); the CLI resolves the data location
presence-based but never asks or writes, running on in-memory defaults with no data directory
when no settings file exists. (4) New Info code `export_identified` (context `team`, `id`) is how
an export's identity is reported. (5) `messages.rs` holds severities only until Phase 8 needs
text. (6) Team compiler settings enter the struct with the phase that reads them; 0/1 settings
are TOML booleans; a missing teams list reads the embedded one, an unreadable or invalid one is
exit 3. (7) Discovery: archive extensions and `NO_USE` compared case-insensitively, `NO_USE` at
the source's own root, sources in `fold_name` order.
Why: (1) the codes are each tool's contract (the Team compiler's 0/1/2/3), so the tool returns
them; an `anyhow::Result<()>` let the binary report only success or 1, and a tool may not print.
(2) A header-per-export format breaks once exports run in parallel; display stems alone cannot
tell `co - Spring` from `co - Spring.zip` (TC-SRC-02). (3) Path settings resolve against the
exe and data folders (`settings.md` "Path resolution"), which the tool had no way to learn; a
CLI that wrote a settings file on first run would make `check` write (TC-CLI-01) and skip the
GUI's first-run choice. (4) TC-ID-01/03 need the identity in the output, and `PipelineEvent` is
tool-neutral by design. (5) The catalog plan gives no texts; inventing ~50 now would be
rewritten when the help window defines their style. (6) Writing keys nothing honors yet
(`dds_compression`, `cpk_part_max_size`) fixes their file shape before the code that reads them.
(7) Windows file names are case-insensitive, and a marker below a wrapper folder is content of
the nested root, not of the source.
Plan: `core/architecture.md` (trait block, `ToolContext`, CLI event line format); `core/README.md`
`anyhow` row; `core/distribution.md` "Data location"; `team_compiler/settings.md` (settings
paragraph); `team_compiler/messages.md` (`export_identified`, text principle);
`team_compiler/pipeline.md` "1. Reader" step 1.

## 2026-10-01 — archives — directory entries listed apart, by `folders()`
Decision (lead): `Archive::folders() -> &[String]` lists an archive's directory entries,
normalized like file paths; `entries()` stays files only. Not chosen: a kind field on `Entry`.
Why: an empty kit folder is a placeholder kit (2026-09-12), and in an archive an empty folder
exists only as a directory entry, which the crate dropped (3.1 review); a mixed `entries()`
would make every size sum (the 7z charge in `libs/pipeline.md`) filter out folders, and a
caller that forgot would charge nothing wrong today but read a folder as a zero-byte file.
Folders are not checked for duplicates: no bytes for two spellings to disagree on.
Plan: `libs/archives.md` (API block, normalization paragraph).

## 2026-10-01 — archives — a directory entry naming the root is skipped
Decision (lead, at 3.8d review): a directory entry made only of `/`, `\` and `.` segments
(`./`, `/`, `.`) is left out of `folders()`; any other directory entry goes through the file
names' `normalize`, so `..` is still `InvalidName`.
Why: with 3.8d's `folders()` such an entry reached `normalize`, which refuses an empty name, so
an archive that lists its own root (before 3.8d, skipped like every directory entry) would have
been refused whole as `export_extract_failed`; the entry carries no folder to report.
Plan: `libs/archives.md` (normalization paragraph).

## 2026-10-01 — team_compiler — 3.9: Phase 3 compile's edges
Decision (lead, applied per the maintainer's standing rule; reversible): (1) A face folder
missing a file Phase 4 would inject (`face_diff.bin`; with `fcl_hair.fmdl`, also
`fcl_hair_sim.fclo` and the paired `fcl_hair.skl`) is "anything else": the export is skipped
with `content_not_yet_compiled`. (2) The placeholder kit (checkerboard texture and template
config, `kit_placeholder`) is Phase 3's, its UniColor entry Phase 4's. (3) Two exports of one
team abort through the writer's duplicate-path invariant (`cpk_write_failed`) until Phase 4's
`duplicate_aesthetics_export`. (4) Staging, promotion and the output preflight are Phase 3's
(single CPK); Phase 4 adds the output sinks, deployment's `.partial` copy and multi-CPK.
(5) Run driver shapes, admission in manifest order on one coordinator thread, a `.7z` export's
tasks sharing its one permit; staging folder `<pid>-<unix ms>`. (6) No new message code for
where the CPK went: `deploy_skipped_by_flag` with the flag, nothing without it until Phase 4
deploys. (7) `rayon` enters the workspace now, as the plan's dependency table names it.
Why: (1) the scope's own rule is "rather than writing an incomplete CPK", and Red injects
these, so a CPK without them differs from Red's in game. (2) TC-SRC-01's THEN needs the
placeholder; UniColor is bins work Phase 3 excludes. (3) Run planning is excluded from Phase 3
by name; the invariant already refuses the CPK, which writes nothing wrong. (4) TC-OUT-03/05
and TC-CLI-06 are Phase 3 scenarios. (5) Admission in writer order is the simplest
progress-safe protocol `core/parallelism.md` asks for; a task request behind an oversized
export permit would wait forever. (6) Every new code is user text Phase 8 must write.
Plan: `team_compiler/README.md` "Phase 3 scope"; `team_compiler/pipeline.md` "Run driver
shapes (Phase 3)"; `core/development_plan.md` (Phase 4 `output/` bullet, Phase 17 placeholder).

## 2026-10-01 — studio — the binary's own exit codes
Decision (lead, from the code-style audit of 3.8): the binary's codes are listed in the plan, in
the tools' meanings: 2 for clap's argument errors, a subcommand no tool owns and an unloadable
settings file; 3 when the executable's folder cannot be found or the console printer stops.
3.8a had exited 2 for a missing executable folder, an environment failure.
Why: the plan named only clap's 2, so the other codes were bare literals no document owned.
Plan: `core/architecture.md` (the exit-code paragraph under `AppPaths`).

## 2026-10-02 — team_compiler — 3.9a review: task reads on the coordinator, kit paths, staging
Decision (lead, reviewing 3.9a; reversible): (1) The coordinator reads each task's files from
its export's content source and hands the bytes to `process_task(index, task, files, ctx)`; a
read failure is `source_read_failed` (`DropFolder`), a conversion or packing failure
`folder_pack_failed`. `TaskBatch` carries no `export_id` (its messages carry their scopes).
(2) `aesthetics_export::KitFolder` carries its `path` (as the export spells it) instead of
`folder_name`. (3) A finished run removes its staging folder and `.staging/` when empty.
(4) A face folder packs only what the tracer path packs: `face_diff.bin`,
`fcl_hair_sim.fclo`, and `fcl_hair.skl` (as `fcl_hair_sim.skl`) only beside `fcl_hair.fmdl`;
textures are `.dds` (converted) or `.ftex` (as they are). (5) An `--export` path that stopped
being a folder or an archive between the preflight and discovery is an error naming it (exit 3).
Why: (1) an archive is one sequential stream, so pool threads sharing a `&mut` source (3.9a's
shape) would serialize on it in 3.9c, and messages.md gives reads their own code. (2) The kit
scope was rebuilt as `Kits/<name>`, which differs from an export spelling `kits/`, so a GUI
tree would hang the finding on a node that does not exist. (3) An empty `.staging/` left in the
output folder is clutter a user sees and did not make. (4) 3.9a packed any `.bin`/`.fclo`
under its own name and dropped the pairing check: game-facing content the Phase 3 scope does
not name. (5) the race has no better owner than discovery; the preflight's exit 2 stands for
the normal case.
Plan: `team_compiler/pipeline.md` "Run driver shapes (Phase 3)" (block, Admission, Output);
`aesthetics_export/object_model.md` (`KitFolder`); `team_compiler/README.md` "Phase 3 scope"
(the face files and texture formats named).

## 2026-10-02 — team_compiler — 3.9b: what the Phase 3 subset gate counts
Decision (lead, from a design pass before briefing 3.9b; reversible): the gate counts every
item the validated export holds except what validation dropped or left unmapped, an unused
`all/`, a link whose missing target `pass_through` keeps, a `kit_mask` on a Fox target (planning
drops it from the kit's textures, silently), a player's `settings.toml` and `fpc.off`, and the
root and kit metadata. A file a lenient setting keeps despite its finding counts, and so do a
portrait, `ingame_face`, `fpc.on`, shared folders, logo, collars and Common. A mapped folder
with no face model is refused (the scope's "holding one or more Fox face models"). The context
is `what` (a path, the target or `refs`) or, for a missing injected file, `missing`. The
first item is found in a fixed order (target, refs, players with files before missing files,
shared folders, kits by slot, the rest). The gate and processing share one classification
(`face_file`, the kit stems, the texture format), so processing keeps no "cannot be compiled
yet" branch of its own. `fcl_hair_sim.fclo` without `fcl_hair.fmdl` is still packed.
Why: the plan's "files a lenient setting keeps … do not count" cannot be implemented: the
validated export does not mark them (`FileDescriptor` has no flag; only the issue list knows),
and Phase 4's own treatment of them is unsettled; refusing never writes a CPK that differs from
Phase 4's. `kit_mask` is not emitted on Fox in Phase 4 either (`pipeline.md` kit textures), and
refusing it would skip 14 of 365 surveyed kits for nothing. `fpc.on` changes every kit config;
`ingame_face` reroutes the face; both would emit something Phase 3 does not build.
Plan: `team_compiler/README.md` "Phase 3 scope" (the "what would be emitted" sentence, context
keys, order).

## 2026-10-02 — team_compiler — Phase 3 emits no FolderStatus, Progress or Complete
Decision (lead, reversible): Phase 3's pipeline emits `ExportStarted`, `Message` and
`ExportProcessed` only. `FolderStatus`, `Progress` and `Complete` are emitted from Phase 8, with
the progress grid and the run strip that consume them; `Complete`'s placeholder fields are
restructured then. TC-DSP-02 loses its "the export finishes as done with errors" clause: that
outcome is a grid cell's, specified in `gui.md` "Cell states", and belongs to Phase 8's
acceptance. Step 3.9b2 is the compile-observed scenario tests only.
Why: the console prints none of the three, and 3.z's view is a plain log with no grid, so in
Phase 3 they would have no consumer and no observable behavior to test; `Complete`'s fields are
documented placeholders, so emitting them now would build a shape the plan already says to
replace. Emitting them with their consumer lets the grid's needs settle what they carry.
Plan: `team_compiler/README.md` "Phase 3 scope" (last paragraph) and TC-DSP-02.

## 2026-10-02 — team_compiler — 3.9c: a `.7z`'s tasks share its permit; the writer reports in order
Decision (lead, reversible): `TaskBatch.permit` is `Option<Arc<pipeline::Permit>>`. The
coordinator reads every task of a `.7z` export, then frees the archive and hands each task a
share of the export's permit, released when the writer has committed the last of them; a
folder or zip task holds its own. The writer thread reports each committed batch's messages,
in manifest order, and an export's `ExportProcessed` after its last task; an export with no
task reports it right after planning. The worker count is `pipeline::thread_count_detect`
(the plan's), not `studio_core`'s `CommonSettings::worker_threads`, which duplicates it with no
caller (left for the Phase 3 `pub` audit).
Why: with tasks running in parallel, the coordinator moves to the next export while a `.7z`'s
tasks still hold its bytes; dropping the permit with the content source (as the serial 3.9a
did) would release a charge whose memory is still in use, and one plain `Permit` cannot be
held by several tasks. Reporting from the pool would make the console's line order, and the
scenario tests' expected lists, depend on thread timing.
Plan: `team_compiler/pipeline.md` "Run driver shapes (Phase 3)" (the block's `permit`, and
"Admission").

## 2026-10-02 — team_compiler — 3.9d: output failures are findings, and every one discards the staging
Decision (lead, reversible): an output folder that cannot be created or written is refused by
`compile` before any export is read (a probe file created and removed; exit 3 naming the
folder). Any failure writing the CPK, the kits' `UniformParameter.bin` included, is reported as
`cpk_write_failed`, and a failed rename onto the final path as `output_commit_failed`; both are
Fatal `AbortRun` findings on the run scope, context `path` (the final CPK path) and `error`,
and both discard the run's staging folder (and `.staging/` when empty), so the previous CPK is
all that is left. A failure removing the emptied run folder after a successful rename is logged,
not a commit failure: the new CPK is in place.
Why: as `anyhow` errors (3.9a-c) these aborted with an `error:` line and left the staged CPK
behind, which the catalog's "required guarantees" forbid. `uniparam_compile_failed` is not
used yet: in Phase 3 the only bin is built on the bundled base, and its failure modes arrive
with Phase 4's installed-bin lookup; until then the one code covers the whole write.
Plan: no plan edit needed: `pipeline.md` "Output" and `messages.md` already state the probe
and both guarantees; this entry records the Phase 3 mapping.

## 2026-10-02 — studio_core, team_compiler — 3.z: the shell slice's edges
Decision (lead, reversible): the Phase 3 shell is `StudioApp` in `studio_core::shell` (a
sidebar listing the tools, the active tool's view, every tool ticked each frame, a
`SwitchTool` request applied), launched by `4cc-studio` with no arguments on `eframe` 0.36.2, `egui` raised from 0.36.1
in lockstep (the plan's GUI stack; default features, the wgpu renderer; 0.36.2 is the latest
release, a patch whose eframe stops busy-looping a core while it waits for a redraw). A tool's run gets its own event
sink through `ToolContext::with_events`; the shell consumes no events itself yet. The console's
event lines move from the binary to `studio_core::EventLines`, used by the console printer and
the Team compiler's log. The Team compiler's view shows its effective settings read-only, a
Compile button that runs `compile` with no arguments through the CLI's own parser, preflight
and verdict on a thread of its own, and a log of that run's lines ending with its exit code.
The binary stays a console-subsystem exe and `--gui` autorun stays refused until Phase 8.
Why: an envelope names no tool, so routing a run's events through the shell would make it
guess whose log they belong to; one formatter keeps a finding's text the same in the terminal
and the window, which TC-GUI-02 now asserts; editing settings needs the shell to save the file
(the settings menu, Phase 8), and the GUI subsystem needs `AttachConsole`, an `unsafe` Windows
call nothing else needs yet; the button reuses the CLI path so the GUI cannot compile anything
the CLI would refuse. TC-GUI-02's "the run completing" becomes the exit-code line: Phase 3
emits no `Complete` (decision "Phase 3 emits no FolderStatus, Progress or Complete").
Plan: `core/architecture.md` "Event system" (the GUI paragraph) and the `events.rs` line of the
crate tree; `core/development_plan.md` "Phase 3" (the shell slice bullet); `team_compiler/
README.md` TC-GUI-02.

## 2026-10-02 — team_compiler — a missing exports folder is created or refused in plain words
Decision (lead, reversible; asked for by the maintainer after the TC-GUI-01/02 run): the
relative `exports_folder_path` (the default `exports/`) is created when missing, before a run
reads it; an exports folder named any other way (an absolute setting, the CLI's positional
root) is never created, and a missing one is refused before the run with a sentence naming the
path and the setting to change, exit code 2. When the relative default cannot be created, the
run is refused naming the path, exit code 3. A new Warning, `no_exports_found` (Run scope,
context: the folder), reports an exports folder holding no export. TC-CLI-08 and TC-CLI-09
cover both.
Why: the plan already said the folder is "created on first run" but not by whom or when, and
gave no text for a missing one, so Phase 3 printed `cannot read the exports folder: The system
cannot find the path specified. (os error 3)` with exit code 3, which tells a cup member
neither that the folder is missing nor what to do. Creating a folder the user named outside the
executable's directory would hide a typo in the setting; exit code 2 matches `--export` naming a
missing path (TC-CLI-07). Without `no_exports_found`, the freshly created folder compiles to a
run that ends at "exit code 0" with nothing in the log.
Plan: `team_compiler/settings.md` "Path resolution" (the exports row); `team_compiler/messages.md`
"Catalog" (`no_exports_found`, and the settings/environment paragraph after "Output stage and
savefile"); `team_compiler/README.md` TC-CLI-08, TC-CLI-09.

## 2026-10-02 — team_compiler — 3.y: the run driver's plan shapes follow the code
Decision (lead, reversible): the Phase 3 `PlanReport` has no `dropped` list, an export the
subset gate skips being only its `DropExport` message; `plan_run` takes
`Vec<(ExportId, ResolvedAestheticsExport)>` and the target `PesVersion`, as built since 3.9a;
`aesthetics_export/object_model.md` stops restating `plan_run`/`process_task` and points at
`team_compiler/pipeline.md` "Run driver shapes (Phase 3)"; `core/parallelism.md`'s run sketch
is marked as the full design, with the Phase 3 names beside it.
Why: the converge audit compared every plan code block with its type and found these the only
undocumented differences in Phase 3's run driver. Nothing reads a list of dropped IDs in Phase 3
(the console and the GUI log print messages), so carrying both would be two homes for one fact;
Phase 4's entry gate, which may reshape these types, can add the list with its first reader.
The `ExportId` pairs are how the gate's messages name their export, and a `CompileContext`
holding only the version would be a wrapper around one value. Two plan files giving the same
signature had already drifted apart (object_model.md kept the pre-3.9a `process_task`).
Plan: `team_compiler/pipeline.md` "Run driver shapes (Phase 3)"; `aesthetics_export/
object_model.md` "Core types" (the tool-side block); `core/parallelism.md` "Design".

## 2026-10-02 — pes_version, team_compiler — the road stays open for a third engine (4.0a)
Decision: `pes_version::Engine` is branched on only with exhaustive `match`es (no `==`/`!=`
against one variant, no `PesVersion` ordering standing in for an engine; comparing two engines
stays allowed); Phase 4's `OutputSink` takes entries by
output-relative path, with game paths the materialize step's alone. Nothing PES 12-specific
enters the plans: the study's stub stays stashed. ID allocation and bins are not made per-target
(the maintainer: modding progress may give PES 12 CPK-like files and custom team IDs).
Why: the PES 12 study found `engine()` an `if >= Pes18` and six `==`/`!=` sites, so a third
engine would be silently treated as pre-Fox or Fox, not refused by the compiler; and an output
sink that knows game paths would make another engine's archive tree a coordinator rewrite instead
of a new variant. Per-target IDs and bins would be machinery for a target that may not need it.
Plan: `docs/CONTRIBUTING.md` "Closed sets are enums"; `team_compiler/pipeline.md` "5. Writer"
(the materialize paragraph after the output modes). Step 4.0c applies the rule to the existing sites.

## 2026-10-02 — distribution — a size budget for the release binary
Decision: the release binary stays under about 30 MB; up to 50 MB is acceptable where a feature
truly needs it (maintainer). Dependencies are weighed against it on an optimized build.
Why: the 50 MB the maintainer saw was the unoptimized debug build; the release build measures
17.1 MiB, so the budget leaves room without letting dependency choices drift unmeasured.
Plan: `core/distribution.md` "Distribution: portable .7z bundle" (the size budget paragraph).

## 2026-10-02 — team_compiler — sideload mode and the overrides folder (4.0b)
Decision: `--mode sider` is `--mode sideload` (output `sideload_output/`, setting
`sideload_output_path`, GUI entry "Compile for sideloading"); Red's `sideload/` input folder is
`overrides/` (message `overrides_active`). With the sideload mode selected, the Compile button has
a **Launch PES** button beside it that starts the selected PES the ordinary way when it is not
running. No status-bar entry.
Why: the mode names its purpose, not one external tool, and Red's folder name collided with it:
"sideload" meant both "files injected into the CPK" and "loose files served to a running game".
The launch shortcut belongs where the prototyping loop happens, the Team compiler's view, not in
the shell's typed status bar.
Plan: `team_compiler/settings.md`, `pipeline.md`, `gui.md`, `messages.md`, `README.md`,
`testing.md`; `player_aesthetics_editor.md`; `core/development_plan.md`, `core/distribution.md`;
`GLOSSARY.md`. Step 4.0d renames the code.

## 2026-10-03 — team_compiler — sideloading through FoxDen (4.0e)
Decision (maintainer): Studio builds no sideloader of its own. Sideload mode writes the unpacked
PES folder structure to `{pes_folder_path}\livecpk\`, the root FoxDen's LiveCPK serves on PES
18–21 (the user's Sider 3 serves it on 17 through a `cpk.root` line); `sideload_output_path` is
dropped. On PES 15/16 the mode is refused (exit code 2) and disabled in the GUI. Studio is the
folder's only writer, so a sideload run replaces its whole contents (lead, on the maintainer's
"Studio will be its only consumer": no stale file from an earlier run can keep overriding).
FoxDen's gaps are requests to its maintainer, not Studio work (worklog "Issues").
Why: FoxDen (formerly `fox_hook`, maintained by a close contributor who takes requests) already
has a LiveCPK built from scratch, loads through a `dinput8.dll` proxy however PES is started, and
picks up files written while PES runs, which is everything the planned sideloader was for, at no
`unsafe`, 32-bit target or hook research in Studio. Sideloading only speeds up aesthetics
makers' model iteration (cup streams never use it, for stability), and no runtime has ever
served PES 15/16; Studio's first full release targets one Fox version the 4cc will play for at
least a year, so pre-Fox sideloading (a 32-bit FoxDen, or previewing pre-Fox models in PES 17,
which shares the `.model` format) is decided after that release.
Plan: `team_compiler/pipeline.md` "5. Writer" (output modes), `settings.md` (table, path
resolution, CLI), `gui.md` (Compile button), `GLOSSARY.md`; `refs_arranger.md` (FoxDen name,
PES 2020); `core/README.md` "Project context" (PES 2020); `model_conversion/conversion.md`.

## 2026-10-03 — workspace — compatible licenses join the allowlist; deps-check after a new dependency
Decision (maintainer): a license compatible with `MIT OR Apache-2.0` (no copyleft obligation on
Studio's source or binary) is added to `deny.toml` when a dependency needs it, without asking;
`BSL-1.0` is the first, for `clipboard-win` and `error-code` under `eframe`'s clipboard
(`arboard`). `just deps-check` runs right after any change that adds a package to `Cargo.lock`,
transitive ones included; it stays out of `just gates`.
Why: 3.z added `eframe` and its two `BSL-1.0` crates, and nothing local ran the license check:
Phases 1 and 2 had run `deps-check` by hand at close, Phase 3 did not, and only CI's
`deps-check` job failed, on every push since 3.z. Boost is permissive and needs no notice in a
binary, so asking would only have delayed the same answer.
Plan: `core/distribution.md` "License"; `CONTRIBUTING.md` "Testing and verification"; `deny.toml`.

## 2026-10-03 — workflow — who decided lives here, not in the plans
Decision (maintainer): plans, `AGENTS.md` and `CONTRIBUTING.md` state a rule without a
`(maintainer, <date>)` note; who decided and when is this file's. Four such notes had no entry,
so they are recorded here, each the maintainer's: Studio Web's first load targets a few MB as
transferred (2026-10-02, `core/gui.md` "Transfer size"); the local mutation half runs at
below-normal priority on half the logical CPUs per cargo process instead of holding two CPUs back
(2026-10-02, `AGENTS.md`); a dependency the plan already names needs no new yes (2026-10-02,
`AGENTS.md`); the PES 2020 account in `core/README.md` "Project context" (2026-10-03, given
because the question keeps coming up).
Why: the notes repeated this file and bloated the documents every agent reads in full.
Plan: `AGENTS.md` (decision logging); `core/distribution.md`, `core/gui.md`, `core/README.md`,
`team_compiler/gui.md`, `pes_savefile/codec.md`.

## 2026-10-03 — development plan — Fox first; game-behavior changes go through FoxDen
Decision (maintainer): the cup moves from PES 17 to a Fox version (not yet chosen) around April
2027 and plays Fox versions for at least a year after, so Studio's first-class target is that Fox
version. Changes to the game's own behavior that Studio's output relies on are made in FoxDen
(which applies the 4cc exe patches at runtime to the stock exe), not in a patched executable.
FoxDen needs the Fox games' Lua engine, so a pre-Fox equivalent is built only if a pre-Fox
version becomes the cup's game again; until then a feature that needs one is Fox-only.
Why: FoxDen was written to take the patches out of the exe so the modding system is more
flexible; the patched exe stays only because FoxDen has been tested on a couple of invitationals
so far. Building a pre-Fox runtime now would serve a version the cup leaves in six months.
Plan: `core/development_plan.md` "Target versions" (new paragraph after "Releases").

## 2026-10-03 — aesthetics — aesthetics travel in the database tables (step 4.0)
Decision (maintainer, from in-game Test 1 on PES 2021): on Fox a player's appearance, boots and
gloves travel in the Team compiler's CPK as rows of `PlayerAppearance.bin`, `BootsList.bin` and
`GloveList.bin`, over a **stripped** official save (every appearance id -1, prepared once per
version by the Save editor's strip-and-seed, whose seed CPK holds everyone's appearance until an
export replaces it). The aesthetics patch keeps only `name` and a new string-only `shirt_name`,
applied by the savefile builder for autopilot teams (and by the compiler to a configured local
save; a names write never sets an id). `height` and `weight` leave `settings.toml` (height is
tactical; weight follows for consistency). An absent key takes its default (the template's
value) on both engines, so a compiled player's appearance is his file alone; the undecoded bits
of the appearance block become `[appearance.unknown.pesNN]` raw values (bools and base64 runs),
per version, default 0 until an in-game check says otherwise. Imports and name writes keep ids
at -1; looks change only through the export, since any Edit-mode change restores a player's id.
Pre-Fox keeps the full savefile aesthetics patch (second-class support, not third-class: as
usable as possible for invitationals on pre-Fox versions).
Why: the test showed the game reads the tables per player whenever the save's appearance id is
-1, and a kept id wins; the stock exe applies `GloveList.bin` without any `Player.bin` goalkeeper
position, while the 4cc gloves patch overwrites it (a FoxDen request, worklog "Issues"). Moving
appearance out of the save makes aesthetics independent of the savefile; keeping a names-only
patch serves autopilot teams, whose tactical exports carry placeholder names. Building rows from
the file alone, not over the installed row, keeps a compile reproducible; carrying the undecoded
bits in the file keeps the about 5% of players who rely on in-game hair, where neutral defaults
would reset them. One meaning of "absent" on both engines replaces two rules per engine.
Plan: `aesthetics_export/settings_toml.md` "Player settings in exports"; `aesthetics_export/
fpc_toggle.md`; `pes_savefile/operations.md` "Aesthetics patch" and Team TOML; `pes_savefile/
model.md` "Player settings model" (planned for Phase 5); `save_editor.md` "Read-only aesthetics",
"Stripped save" (new), feature inventory; `team_compiler/pipeline.md` (ID writes, "Bins
accumulation", "Post-processing", game paths, resolved decision); `team_compiler/README.md`;
`player_aesthetics_editor.md`; `core/development_plan.md` Phases 4, 5, 6; `core/README.md`;
`GLOSSARY.md` ("Aesthetics patch", "Stripped save", "Autopilot team").

## 2026-10-03 — team_compiler — Phase 4 itemization rulings
Decision: the gaps found while itemizing Phase 4 (worklog step 4.1) are settled as follows. A
folder whose selected model representation is glTF is dropped with the new catalog row
`model_gltf_unsupported` (E, `DropFolder`, not pass-through-eligible) until Phase 7, and
`content_not_yet_compiled` keeps its row until step 4.20 withdraws it. Phase 4 writes
`BootsList.bin`/`GloveList.bin` rows only for compiled players whose custom boots/gloves output
committed, keeps every other row, and passes `PlayerAppearance.bin` through unchanged; Phase 5
adds the stock/default-ID rows and the appearance rows from `settings.toml`. Deployment adds no
exit code: the existing mapping holds (an Error finding exits 1, a Fatal one 3). Output-mode
artifact routing: `teamnotes.txt` under `output_folder_path` in every mode; sideload puts bins,
overrides and referee content at their game paths in `livecpk/`; test puts bins under
`test_output/_bins/` at game-relative paths, referee content per export like a team's, and does
not apply overrides. `run_pes` moves to Phase 8 with sideload mode's Launch PES button, as one
launcher. Phase 4 reports only `settings_toml_invalid` (parse) of the settings codes; the others
describe resolved values and are Phase 5's. Hand auto-split is Phase 4 (step 4.18). The source
snapshot is Phase 4, an export's revision being the listing's (path, size, modified time) set for
a folder and the archive's size and modified time, pinned at planning and rechecked before each
task's read. `dds_compression`'s level is chosen by measuring levels 1, 3 and 6 in step 4.16 and
recorded in a decision entry. TC-GUI-02's THEN accepts both the deployed and the promoted outcome.
CLI `check` runs both passes on every source kind. `model_source_ambiguous` keeps its row, its
concrete input being two glTF files of one stem, so its scenario is Phase 7's. The new Info row
`bin_source` names the CPK that supplied each working bin, or `bundled`. Step 4.6 emits
`kit_texture_not_used` for a `kit_mask` on a Fox target, which Phase 3 dropped silently.
Why: on each point the plan was silent or two plan sections disagreed, and the draft's reasons
held: narrowing `content_not_yet_compiled` to a folder scope would change a code's disposition
under an unchanged name; writing boots/gloves rows from the planned IDs is the only split that
keeps Phase 4's CPK game-correct without reading `settings.toml`; a degraded run is "an Error
finding in some scope" exactly as the mapping defines 1, and a fourth code would make callers
special-case deployment; test mode shows what the compiler did to an export, and an override is
not the export's; one launcher for both PES launches avoids two code paths for the same exe; the
reader already lists (path, size, modified time) and the GUI watcher keys on the same value; a
`check` that misses a model error the next `compile` reports defeats the command; a maintainer
checking stale midcup colors needs to know which CPK a bin came from.
Plan: `team_compiler/README.md` "Acceptance" (Phase 4 scope and scenarios; TC-OUT-02, TC-GUI-02,
TC-SRC-01 edited); `team_compiler/messages.md` (`model_gltf_unsupported`, `bin_source`,
`model_source_ambiguous`); `team_compiler/settings.md` "CLI"; `team_compiler/pipeline.md` ("2.
Per-export serial steps" step 2, "6. Post-processing" Run PES, "Resolved decisions" Source
snapshot and Output-mode artifact routing, "Run-result semantics"); `core/development_plan.md`
Phases 4 and 8; `GLOSSARY.md` ("Output sink", "Teams part", "Placeholder CPK").

## 2026-10-03 — team_compiler — Phase 4 pre-step measurements
Decision: C2, `colors.txt` has one grammar for the root and the kit files (one or more colors per
line, hex `#RRGGBB` or three decimals, label and `- `/`,` separators tolerated), a kit takes its
first two valid colors, the root file its first four, unfilled team slots keep the working bin's
bytes, and the maintainer confirms it at 4.8; C8, the referee face, boots, gloves and common paths
on both engines join "Game paths reference" and the two marker paths join "Referee export
processing"; C10, the blank face folder is Fox `face.fpk` with only `face_diff.bin` plus the
template `face.fpkd`, pre-Fox a `face.xml` naming `oral_dummy_win32.model` and `dummy.mtl` with the
template `face_diff.bin`; C17, `DpFileList.bin` is a 16-byte header, 48-byte records and an
all-zero tail of no fixed length that the reader ignores, the upgrade copying the bundled file byte
for byte; C19, `collar_<ID>` accepts the ID with or without zero padding, is emitted under the
game's three-digit name, and must be a stock collar of the target version (21: 1-131 and 901-913;
17: 1-116 and 901-916; unmeasured versions the intersection), 105 refused; C20, `_r_ll` is the
512 logo, `_r_l` 256, `_r` 128; C24, a DDS portrait passes through unchanged and any other image
format is encoded to BC3 at its own size with a full mip chain.
Why: each was an open point the plan left for Phase 4, and each is now a measurement rather than a
choice: PES 2021's and PES 2017's installed files (`E:/PES2021`, `E:/PES2017`: 1534 cup
portraits, the two `DpFileList.bin` files, the `nocloth` sets of the base data CPKs, team 701's
logos, `TeamColor.bin` and `UniColor.bin` in `4cc_08_bins.cpk`), Red's code (`bins_update.py`,
`referee_tools.py`, `export_move.py`) for the parts the game files do not show, and a corpus of 53
VGL26 Team Notes for the color lines authors actually write (`.tmp/measurements_4_pre.md` names
the sources). Guessing any of them would have been a plan sentence the first test contradicted.
Plan: `aesthetics_export/player_folders.md` ("Portraits", "Root files" "Colors");
`aesthetics_export/object_model.md` ("File-type allowlist"); `team_compiler/pipeline.md` (step 4,
"Kits", "Logo", "Collars", "Bins accumulation", "DpFileList upgrade", "Game paths reference",
"Resolved decisions and open questions"); `team_compiler/messages.md` (`collar_id_invalid`);
`team_compiler/README.md` (Phase 3 scope, TC-CMN-01); `team_compiler/blue_port.md` ("Referee
export processing").

## 2026-10-03 — team_compiler — a player folder's tasks commit as one group
Decision: the writer holds a player folder's face, boots and gloves batches until the folder's
textures batch (last in the manifest) arrives, then commits the packages that succeeded and the
textures; failed textures drop the whole folder (`folder_pack_failed`, `DropFolder`), and the
textures stay out when every package failed. The textures batch is where 4.5's
`shared_texture_conflict` will name the losing package to skip.
Why: the plan says the textures commit once after the player's tasks report, but not what a
texture failure does. Committing the textures last on their own (the first 4.3 shape) left
packages in the CPK pointing at a texture it lacks, and could not drop a conflict's losing
package, which is already committed by then; the sidekick found both. The group is charged to
the memory budget once, at its first task, and its tasks share that permit: a textures task
waiting for a permit of its own could wait forever for memory the held packages never release
(the sidekick's contradiction of the lead's rework brief, accepted).
Plan: `team_compiler/pipeline.md` "3. Per-model-folder parallel steps" step 6.

## 2026-10-03 — team_compiler — shared boots/gloves outputs: their texture home and ID order
Decision: a shared `Boots/`/`Gloves/` folder linked plainly keeps its textures beside its own
package, at `Asset/model/character/boots/k{id}/#windx11/{stem}.ftex` (gloves
`…/glove/g{id}/#windx11/`), its FMDL paths naming `/Assets/pes16/model/character/boots/k{id}/`
(`…/glove/g{id}/`). Shared IDs go in case-insensitive folder-name order (`vtree::fold_name`, ties
by spelling). The pool check (`boots_id_pool_exhausted`/`gloves_id_pool_exhausted`) runs in the
structure pass after identity, so `check` reports it too; `SharedModelFolder` gained its `path`.
Why: the plan said the textures stay "in that model's own texture location" without naming it;
Red's tracer output puts them at `boots/k{id}/#windx11/` with those FMDL paths, and 306 of 400
gloves FMDLs in the VGL26 corpus already name `…/glove/g{id}/`. A plain byte order would put
every capitalized name before every lowercase one, an order no member sees in Explorer. TC-MOD-06
reports the pool "when the export is checked", and `check` runs only the structure pass. The
path was guessed from the first file's folder, which a nested file makes wrong.
Plan: `team_compiler/pipeline.md` "Game paths reference" (new row);
`aesthetics_export/player_folders.md` "At compile time", item 2.

## 2026-10-03 — team_compiler — a combined shared folder's textures all go to the player
Decision: when a player combines a boots/gloves link with local models, every texture of the
shared folder is copied to the player's common subfolder, not only those the merged part names;
the parts merged under one allowed name go in case-folded file-name order, ties by export path;
two parts' skeletons agree when their paired `.skl` files are byte-identical or both absent.
Why: planning, which builds the textures task, reads no model bytes, so it cannot see which stems
the shared models name; reading them there would load every shared model twice. A shared
`Boots/`/`Gloves/` folder's textures are its models', so the difference is an unused file at
most. Byte comparison of a handful of `.skl` files says what the plan's content hash says.
Plan: `team_compiler/pipeline.md` "3. Per-model-folder parallel steps", step 6.

## 2026-10-03 — team_compiler — texture conflicts by source package; the player's face files win
Decision: a player's texture sources are its own folder and each folder it combines, each counting
for the package it feeds (own folder and combined face: the face; combined boots/gloves: that
package). One stem (case-folded) from two sources with different converted bytes is
`shared_texture_conflict` across packages (the lower package dropped, with the textures only its
sources hold) and `merged_texture_conflict` within one (the folder dropped). A shared face folder's
`face_diff.bin`/`fcl_hair_sim.fclo` give way to the player folder's own.
Why: the plan names the two codes per task and per model, but the textures are one task per
folder and planning reads no model bytes, so which model names a texture is unknown there; the
source's package is the nearest known stand-in, and dropping the lower package's whole source
keeps every remaining model pointing at its own source's texture. The two face files are not
model parts, so merging does not apply; "local parts layered over the base" says the player's
own wins.
Plan: `team_compiler/pipeline.md` step 6; `aesthetics_export/player_folders.md` "A link plus
local models combines".

## 2026-10-03 — team_compiler — TC-MOD-04 tests a combined shared folder, not two subfolders
Decision: TC-MOD-04's conflicting case is a player's own `skin.dds` against a combined
`Boots/Crocs/skin.dds`, not `face/skin.dds` against `boots/skin.dds`.
Why: a player folder's reserved subfolders share one texture namespace with its root ("stems
resolve across the player folder including its reserved subfolders", `player_folders.md`), so
two `skin.dds` there are `texture_stem_conflict` and validation drops the folder before any
compile; the scenario as written could never reach `shared_texture_conflict`. Two sources of
one player holding a stem exist only when a shared folder is combined.
Plan: `team_compiler/README.md` "Acceptance", TC-MOD-04.

## 2026-10-03 — team_compiler — Fox's Common textures are one task no player task waits on
Decision: on Fox, every texture directly in `Common/` is converted by one task per export
(after its shared folders, before its portraits) into `Asset/model/character/common/{team_id}/
sourceimages/#windx11/`, committed on its own. A player package baking in a `.common`-linked
model reads the Common model itself and points the part's stems Common holds there. A failure of
the Common task is reported and the linking players still commit.
Why: one Common texture serves every linking player, so putting the task in a player's writer
group would tie all of them together; dropping the linking players on its failure would need a
cross-group dependency in the writer, for a failure fixed in one place. A player linking a shared
output that failed is already kept the same way.
Plan: `team_compiler/pipeline.md` "Resolved decisions" (new bullet) and the "Shared/Common
dependency graph" open question narrowed to cache ownership.

## 2026-10-03 — team_compiler — a texture's role comes from its stem (`_nrm` is a normal map)
Decision: the compiler converts a texture as a normal map (`dds_convert::TextureRole::Normal`)
when its stem ends in `_nrm` in any case, and as color otherwise.
Why: the plan says role-specific handling applies but not where the role comes from. Reading
it from the FMDL sampler would make the folder's textures task read every model, and one stem
is converted once per folder whichever models and samplers name it; the `_nrm` suffix is the
Studio's own role table's and the game's (`skin_nrm`, `oral_nrm`, `dummy_nrm`). A DDS normal
map already in BC3 passes through either way, so the rule matters for raster and BC7 sources.
Plan: `team_compiler/pipeline.md` step 5.

## 2026-10-03 — dds_convert — `encode_dds` writes a DDS in a named codec on any engine
Decision: `dds_convert` gains `encode_dds(decoded, codec)`, a DDS in BC1, BC3 or BC7 at the
source's size whatever the target, for the portraits (BC3 with a full mip chain on every
version).
Why: `convert` picks the container and codec from the target version, so on a Fox target it
returns an FTEX, and on PES 15-17 an opaque source would come out BC1; the portrait rule fixes
both the container and the codec, so the caller names them rather than `convert` growing a
special case.
Plan: `libs/dds_convert.md` "`dds_convert` API".

## 2026-10-03 — workspace — the BC7 encoder builds optimized in dev
Decision: the root manifest sets `opt-level = 3` for the `block_compression` package in the dev
profile; every other dependency keeps the dev defaults.
Why: unoptimized, the BC7 encode of one 1024x1024 texture (TC-TEX-01) took 55.7 s in a test
build against 8.6 s optimized, which every `cargo test`, CI job and mutant would pay; optimizing
all dependencies would slow every clean build for no measured need.
Plan: no plan edit needed (build configuration; the profile comment points here).

## 2026-10-03 — team_compiler — a kit texture's finding drops the kit; a renamed file is sniffed
Decision: a texture finding (`texture_too_small`, `texture_not_pow2`, `texture_type_mismatch`,
`texture_codec_unsupported`) on a kit texture drops the kit, as in a model folder; in `Common/`
and for a portrait only the file is dropped. `texture_type_mismatch` is a file whose bytes open
with another accepted format's signature than its extension names.
Why: the catalog said "elsewhere the file is normally dropped" without naming the kits; a kit
config names the textures the kit has, so a kit emitted without one is not the kit the member
drew, and `texture_stem_conflict` already drops the whole kit. The signature rule needs no
decode, and TGA, which has none, can only be caught when it opens with another format's.
Plan: `team_compiler/messages.md` "Textures" preamble.

## 2026-10-03 — development plan — a texture task's cancellation check moves to Phase 8
Decision: Phase 4's "texture conversion as bounded batches with cancellation" ships as its bound
only: each worker converts one texture at a time and `dds_convert` has no parallelism of its own,
so the conversions in flight are bounded by the worker count. Checking for cancellation between a
texture task's textures moves to Phase 8's "Implement cancellation", with the Cancel button and the
cancelled run's staging discard.
Why: nothing in Phase 4 cancels a run (the CLI has no interrupt handling, and the GUI's Cancel is
Phase 8), so a check built now could not be exercised end to end. Stopping a task early would also
need the run to discard its staging, which the coordinator's `Cancelled` path does not do yet, or
a cancelled run could promote a CPK missing textures; and the run-result semantics are an open
question. Built with its trigger, the check gets a test that cancels a real run.
Plan: `core/development_plan.md` "Phase 4" (`processing/`) and "Phase 8" (cancellation);
`core/gui.md` "Cancellation".

## 2026-10-03 — team_compiler — `kit_texture_uncompressed` is retired
Decision: an uncompressed kit texture is no finding; it is encoded like any raster source.
TC-CHK-04's uncompressed `p2/kit.dds` now compiles to a BC7 FTEX on PES 21 with no finding.
Why: `libs/dds_convert.md` already encodes every uncompressed source ("PES 15–17 crash on
uncompressed kit textures"), and TC-TEX-06 compiles a `kit.png`, which is uncompressed too, so
the finding could only refuse a file the compiler fixes. Keeping it would refuse an RGBA8
`kit.dds` while accepting the same pixels as a PNG.
Plan: `team_compiler/messages.md` "Textures" (row removed, preamble sentence);
`team_compiler/README.md` TC-CHK-04.

## 2026-10-03 — team_compiler — a format finding drops its folder by severity
Decision (user): a file that cannot be produced, or whose compiled result is unusable or visibly
compromised, is an Error and drops its folder; one that compiles to something usable but may
hide an issue is a Warning and compiles. The deep pass therefore maps a format crate's `check`
finding by its severity (Error: `DropFolder`; Warning, Info: `Keep`).
Why: one rule instead of a per-code table; a format Error seen on models the game renders is a
wrong severity in the format crate, fixed there (the lead's `fmdl::check` census at 4.7 looks
for them), not an exception in the compiler.
Plan: `team_compiler/messages.md` "Message structure" ("What makes a finding an Error").

## 2026-10-03 — team_compiler — a model's missing texture is an Error; installed CPKs are searched
Decision (user): `fmdl_no_texture_ids` is dropped. A texture a model's meshes use and nobody
supplies is an Error dropping the folder (`fmdl_texture_not_found`, and `mtl_texture_not_found`
pre-Fox). A path naming the team's Common output is looked for in the export's `Common/`, then
in the installed CPKs' tables of contents, so a partial (midcup) export pointing at Common
textures compiled earlier is not refused; a texture `.common` link without a target in the
export is satisfied the same way. Chosen by the lead where the ruling was silent: the CPKs
searched are those the installed `DpFileList.bin` lists, minus the run's own (Red filtered by
the names `midcup`, `uniform`, `faces`); paths naming anything else are not looked up; with no
readable install the finding is a Warning that keeps the folder.
Why: the player would not look as intended, which is the Error rule above; the old warning was
about IDs in paths, which nothing here resolves by. Without the CPK lookup every midcup export
would lose its changed players; without the Warning fallback, a compile on a machine with no
PES install would.
Plan: `team_compiler/pipeline.md` "Resolved decisions" ("A texture a model names must exist");
`team_compiler/messages.md` (both codes); `team_compiler/README.md` TC-TEX-05 rewritten;
worklog step 4.29.

## 2026-10-03 — workspace — the `base64` crate decodes `face_diff.xml`
Decision (user: either; the lead picked): the `base64` crate, not a hand-written decoder.
Why: nothing to own or mutation-test, and `settings.toml`'s raw runs (`settings_toml.md`) will
need an encoder too.
Plan: `core/README.md` dependency table.

## 2026-10-03 — aesthetics_export — `colors.txt` holds one color per line
Decision (user): the kit file writes its two colors on two lines, like the root file, instead
of the Team Note's `211 74 79 - 162 62 77`. Two colors on one line is `color_entry_invalid`;
the tolerance for a trailing icon number is gone with it.
Why: no separator to get wrong. The one-line form only exists in old Team Notes, which the
Export upgrader splits when it migrates them.
Plan: `aesthetics_export/player_folders.md` "Root files" (Colors); `team_compiler/messages.md`
(`color_entry_invalid`); `export_upgrader.md`.

## 2026-10-03 — aesthetics_export — marker names: `fpc_on`, `fpc_off`, `icon_<N>`
Decision (user: a marker replaces `icon.txt`, one spelling for every marker; the lead picked the
spelling): a kit's icon is the empty marker `icon_<N>`; a marker with a value is
`<name>_<value>`, so `fpc.on`/`fpc.off` become `fpc_on`/`fpc_off`; `ingame_face`, `pre-fox` and
`fox` keep their names; every marker tolerates `.txt`.
Why: an underscore, not a dot or a hyphen, because the format already writes `collar_12` and
`player_05`, and a dot makes the value a file extension, which file browsers hide and warn
about. A marker instead of `icon.txt` removes a file to open and a small-metadata read.
Plan: `aesthetics_export/player_folders.md` "Root files" ("Marker names"); the other sections
and the code are renamed by worklog step 4.30.

## 2026-10-03 — team_compiler — the Fox referee marker is a reserved collar, not a dt00 write
Decision (user): one stock collar ID no team uses is reserved for the referees; the refs CPK
carries the marker model as that collar, pointing at `ref_marker.dds` converted into the
referees' Common output, and the referee template kit configs name the collar.
`dt00_overwrite_allow`, `ref_marker_needs_consent` and `dt00_write_failed` are retired; a team
claiming the reserved ID gets `collar_id_invalid`. Open: the ID (a survey picks it) and the
marker model, both the maintainer's; whether the pre-Fox marker follows.
Why: writing a system CPK needed a consent setting, a backup and a rollback protocol nobody had
defined; a collar keeps everything inside the refs CPK.
Plan: `team_compiler/blue_port.md` "Referee export processing"; `messages.md` "Referees",
`collar_id_invalid`; `settings.md`; `pipeline.md` "Collars", open questions; `README.md`
TC-REF-06, TC-REF-07 rewritten; `core/distribution.md`; worklog step 4.27.

## 2026-10-03 — team_compiler — one official DpFileList for every PES version
Decision (user): the embedded list is the one standard layout (midcup CPKs up to 79 and
`4cc_90_test`), not a file per PES version; every other layout is obsolete. The PES 21
install's list lacking `4cc_90_test` is such an old list, so the default `cpk_name` stays.
Why: the cup maintains one layout. Not verified: that one file's bytes serve every PES version;
step 4.25's brief checks it with the reader.
Plan: `team_compiler/pipeline.md` "Post-processing" (DpFileList upgrade), "Multi-CPK mode",
"Resolved decisions" (templates); worklog steps 4.24, 4.25.

## 2026-10-03 — team_compiler — the referees' collar is 77, on both engines; a team using it is an Error
Decision (user): collar 77 is reserved for the referee marker; the pre-Fox marker moves to the
collar too; a regular team using the reserved collar is an Error. Shaped by the lead: a
`Collars/collar_77.*` file is `collar_id_invalid`, and a kit whose effective collar or winter
collar is 77 is the new `kit_collar_reserved`, which drops the kit.
Why: 77 is a stock collar of every version that none of the 4,635 kit configs on the
maintainer's machine uses (69 and 68, the first choices, appear in 2016 configs of /tv/, /m/
and /pol/). One method for both engines: pre-Fox support is second-class, so it gets no
exception. A team kit on collar 77 would dress its players in the referee marker.
Plan: `team_compiler/blue_port.md` "Referee export processing"; `messages.md` "Referees",
`collar_id_invalid`; `pipeline.md` "Collars", "Collar contract"; `README.md` TC-REF-04,
TC-REF-06, TC-REF-07; worklog steps 4.9, 4.27.

## 2026-10-03 — team_compiler — the texture lookup searches only the CPKs before the one compiled
Decision (user): the installed CPKs searched for a Common texture are those the DpFileList
names before the CPK being compiled, nearest first (compiling `4cc_67_midcup`: 66, then 65,
...), never a later one, and with no filter on the name.
Why: each midcup CPK is additive over the ones before it, and removing a later one must not
break it; it is the walk the bins already use. Added by the lead: a list that does not name the
CPK being compiled leaves the lookup undecidable, which is the Warning case.
Plan: `team_compiler/pipeline.md` "Resolved decisions" ("A texture a model names must exist");
worklog step 4.29.

## 2026-10-03 — team_compiler — the official DpFileList carries the size-split slot run
Decision (user): the one official list replaces the faces/uniform CPKs with the size-split run
(the 4 GiB limit of Git for Windows), which "Multi-CPK mode" already planned as `teams` slots.
Why: verified against the plan and the installs: the plan has the run, but no installed list
does (PES 17's still names `4cc_40_faces`/`4cc_45_uniform`), so step 4.25's embedded file is
authored from an entry list the maintainer fixes. Open: the entries, and whether the stem is
`teams` (the plan) or `players` (the maintainer's word).
Plan: `team_compiler/pipeline.md` "Post-processing" (DpFileList upgrade); worklog step 4.25 and
"Phase 4 open questions".

## 2026-10-03 — team_compiler — a `Collars/` file named for a reserved collar is a conflict
Decision (user): a team's `Collars/` file named for collar 105 (FPC) or 77 (the referees'
marker) reports `collar_id_conflict`, not `collar_id_invalid`: the run-wide claimed-ID list
starts with the suite's own two claims. `kit_collar_reserved` stays for a team kit config
naming collar 77, so a team doing both gets both codes.
Why: the ID is a real stock collar that someone else holds, which is what a conflict says;
"invalid" would send the user looking for a typo. Two codes, not one, because they are about
two files with two outcomes: the collar model is dropped, the kit is dropped.
Plan: `team_compiler/messages.md` `collar_id_invalid`, `collar_id_conflict`; `pipeline.md`
"Collars", "Collar contract"; `blue_port.md` "Referee export processing"; `README.md`
TC-CMN-02, TC-REF-07; worklog steps 4.9, 4.27.

## 2026-10-03 — team_compiler — the games' uniform models are the kit layout's source of truth
Decision (user): `KIT_LAYOUT_REMAP` takes its numbers from the two games' own uniform models
(the base data CPKs) alone. The pair made with PES Master's two kit creators is a cross-check,
not a source: it agrees on the socks and draws the shorts identically in both layouts, where
the models shift them, and the models win.
Why: the models are what the game renders; a third party's templates are its artist's reading
of them. Added by the lead: the step's golden is therefore computed from the models by a
provenance script, independent of the Rust table, instead of taken from a hand-drawn pair, so
no PES Master artwork enters the repository.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps" (Kits, "Layout
conversion"); `README.md` TC-KIT-18; `testing.md`; worklog step 4.10.

## 2026-10-03 — team_compiler — what a valid face diff is, and `face_diff.xml`'s two forms
Decision: `face_diff.xml` is the base64 of a `face_diff.bin`, alone or as the text of a `<dif>`
root (the two forms Red reads). A face diff, decoded or supplied as `face_diff.bin`, must have
the magic `FACE` and at least the length its header's two counts give; a longer file passes.
A folder with both `face_diff.bin` and `face_diff.xml` is `xml_dif_conflict`. A WESYS-wrapped
`face_diff.xml`, which Red unwraps, is not read.
Why: Red checks the decoded length for equality, and only for the xml; a census of the
maintainer's machine found 326 of 2,695 loose `face_diff.bin` 16 bytes longer than their
header gives, all in cups' CPKs, so equality would refuse the text form of files the game
loads, while a file shorter than its header makes the game read past its end, whatever form it
came in. Both files in one folder is the "two sources for one datum" the plan already
discards for `face.xml`; Red lets the `.bin` win silently. No `face_diff.xml` exists on that
machine, loose, so nothing argues for the wrapper.
Plan: `aesthetics_export/player_folders.md` "`face_diff.xml`"; `team_compiler/messages.md`
`face_diff_invalid`, `xml_dif_conflict`.

## 2026-10-03 — team_compiler — the size-split slots keep the stem `teams`
Decision (user): the slot run stays `4cc_40_teams` … as "Multi-CPK mode" planned it, not
`players`.
Why: the compiler compiles teams. The official list's other entries are a lead's draft for
the maintainer to adjust (worklog "Phase 4 open questions").
Plan: no plan edit needed (`team_compiler/pipeline.md` "Multi-CPK mode: teams parts" already
says `teams`).

## 2026-10-03 — team_compiler — the stock collar sets are measured for every version but PES 20
Decision: `collar_id_invalid`'s per-version sets are each install's own: PES 15 1-101 and
901-904; PES 16 1-105 and 901-904; PES 17 and 18 1-116 and 901-916; PES 19 1-124 and 901-916;
PES 21 1-131 and 901-913. PES 20 takes what 19 and 21 share (1-124, 901-913) until an install
is measured.
Why: the earlier interim rule gave PES 15, 16, 18, 19 and 20 the intersection of PES 17 and 21
(1-116, 901-913), which accepts collars PES 15 and 16 do not have (102-116, 905-913) and
refuses ones PES 18 and 19 do. Counted on the maintainer's installs on the external drive
(`F:\Games\PES2015` … `PES2021`), `collar_NNN` under `nocloth` in each `dt35` CPK
(`.tmp/collar_sets/measure.py`, `summary.txt`). The reserved collars hold everywhere they are
used: 77 is stock in all six; 105 is absent from PES 15, where there is no FPC preset.
Plan: `team_compiler/messages.md` `collar_id_invalid`.

## 2026-10-03 — fpc — FPC exists on PES 15 (measured) and very likely on PES 18; correction
Decision: the entry above ("the stock collar sets are measured…") ends with "105 is absent
from PES 15, where there is no FPC preset". The first half holds for the stock game only; the
second half is wrong. `fpc::kit_values` is to return the four FPC values (shirt model 176,
shorts model 16, collar 105, winter collar 105) for PES 15 too, at step 4.9, and for PES 18
unless the maintainer says its values differ.
Why: checked on the maintainer's installs on the external drive. Stock PES 15 has no collar
105 anywhere (`nocloth/collar_NNN` ends at 101, and so do the `d/modD_shirt_*_collar_NNN`
models; no Konami data pack is installed there, so one adding collars would not be seen). But
every install, PES 15 and 18 included, has its own `4cc_04_fpc.cpk` holding the empty
`collar_105` and `pants_016` models, and 301 of PES 15's 360 installed kit configs carry
exactly the four FPC values (26 of 50 teams on every kit); the same count gives 285 of 338 on
PES 16, 344 of 407 on PES 17, 35 of 49 on PES 19 and 474 of 637 on PES 21. PES 18's install
holds no team kit config; its FPC CPK has the same 15 files as PES 19's. The `None` for PES
15 and 18 came from the wiki page, which is silent on both. Reserving collar 105 on every
version, as `collar_id_conflict` does, was right all along. Tool and output:
`.tmp/fpc_kits/` (`src/main.rs`, `summary.txt`).
Plan: `libs/fpc.md` "`libs/fpc`" (`kit.rs`).

## 2026-10-03 — team_compiler, aesthetics_export — the deep pass: the compiler checks, the export crate drops
Decision: the deep pass is split in two. The Team compiler reads the files of the sanitized
export and runs the format crates' checks on them (`team_compiler/src/deep.rs`);
`aesthetics_export` takes what they find as `ContentFinding`s
(`ValidationReport::with_content_findings`) and derives the sanitized export again, so drops,
the link cascade and `pass_through` are the structure pass's own code. The pass runs before
identity and planning, in `check` and in `compile` alike. The worklog's step 4.7 had
`aesthetics_export` orchestrate the format crates itself (`validate/deep.rs`).
Why: the export crate's layout rules forbid it I/O and content parsing ("No I/O", "Deep format
validation is not here"), and it is the Studio Web cheap tier's export knowledge, so it should
not pull in the model and texture crates. Re-deriving from the parse, not patching the
sanitized export, because the cascade (`link_target_dropped`, `shared_folder_orphaned`) is
written over the draft: a second copy over the sanitized types would be two functions to keep
equal. Before planning, not inside the tasks, because a content Error must drop the whole
folder (TC-CHK-01) and free its IDs, and `portrait_conflict` drops the export; a task failure
can do neither. The cost, not yet measured: `compile` reads each checked file twice, and a
solid `.7z` is decompressed a third time (metadata, deep pass, tasks); worklog "Issues".
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps" (deep format pass);
`aesthetics_export/object_model.md` "Validation semantics" (content findings).

## 2026-10-03 — fpc — PES 18 uses the same four FPC values
Decision (user): `fpc::kit_values` returns shirt model 176, shorts model 16, collar 105 and
winter collar 105 for PES 18 too. With PES 15 (entry above) the values are then the same on
every version, so the function loses its `Option` at step 4.9.
Why: PES 18's install has the FPC CPK, the same 15 files as PES 19's, but no team kit config
to count, so the values needed the maintainer's word.
Plan: `libs/fpc.md` "`libs/fpc`" (`kit.rs`).

## 2026-10-03 — team_compiler, dds_convert — texture checks: from the header, in the deep pass
Decision: the deep pass reports `texture_type_mismatch`, `texture_too_small`,
`texture_not_pow2` and `kit_texture_too_big` from each texture's header, through a new
`dds_convert::probe` that decodes nothing; processing stops checking its sources for them.
Four rules the plan left open. (1) `texture_type_mismatch` is not pass-through-eligible; the
three size findings are. (2) A texture whose header cannot be read gets no finding in the
deep pass: its conversion fails its task. (3) `texture_codec_unsupported` stays conversion's
finding, reported by `compile` only. (4) On a kit's main texture `kit_texture_too_big`
covers the power-of-two rule, so `texture_not_pow2` is not reported beside it.
Why: (1) a mismatched file cannot be converted as the format its name declares, so there is
no content to keep, while an odd-sized texture converts and its effect in the game is the
member's risk, which is what `pass_through` is for. (2) and (3): a header shows neither cut
pixel data nor every refused layout without the decode's own code path, so a "broken texture"
finding at `check` would promise more than the probe can see; TC-TEX-04 already says
"compiled", TC-TEX-03 "checked". (4) one cause should print one line. The probe is in
`dds_convert`, not the compiler, because the header knowledge (which reader, the WESYS
unwrap, the chain a raster source gets) is that crate's.
Plan: `team_compiler/messages.md` "Textures"; `libs/dds_convert.md` "`dds_convert` API";
`team_compiler/pipeline.md` "Common textures are one task of their export".

## 2026-10-03 — team_compiler — color bins: every record's header is set on every run
Decision: the compiler writes `TeamColor.bin` and `UniColor.bin` whole on every run and sets
every record's header from its position: the team ID (100 plus the record's index) and, in
`TeamColor.bin`, the color count 4. Not: writing only the compiled teams' records, or only
their colors after the header as Red does. A record whose header was overwritten keeps its
other bytes; no finding is reported for the repair.
Why (maintainer): installed cup bins carry `TeamColor.bin` records whose colors were written
from the record's first byte, over the ID and the count (teams 799, 829 and 831 in the last
VGL's PES 21 file, team 761 in an older cup's; `resources/bins/README.md`), which is the
likely cause of team colors recently missing in the game. Nothing can go wrong from making
sure every team has a header: on a sound record the write changes no byte, so a bin built on
Red's base still compares byte-identical with Red's output.
Plan: `team_compiler/pipeline.md` "Bins accumulation"; `team_compiler/README.md` TC-BIN-13.

## 2026-10-03 — team_compiler — color bins: a repaired header is reported
Decision: when a working `TeamColor.bin` or `UniColor.bin` holds records whose header is not
their position's, the compiler reports `bin_header_repaired` (Warning), once per bin, naming
the bin and the teams. This replaces the earlier entry of this date's "no finding is reported
for the repair"; the rest of that entry stands.
Why (maintainer): a warning helps notice this sort of corruption and investigate what writes
it. It also tells the organizer which teams' colors may still be wrong, since the repair
keeps the record's other bytes.
Plan: `team_compiler/messages.md` `bin_header_repaired`; `team_compiler/pipeline.md` "Bins
accumulation"; `team_compiler/README.md` TC-BIN-13.

## 2026-10-03 — team_compiler — the official DpFileList's entries, and a check on every compile
Decision: the official list is `resources/templates/DpFileList.txt` as the maintainer edited
it, 53 entries: the eleven base CPKs, `4cc_20_stadiums` to `4cc_35_stadiums`, `4cc_41_teams`
to `4cc_45_teams`, `4cc_51_teams2` to `4cc_55_teams2`, `4cc_61_midcup` to `4cc_75_midcup`, and
`4cc_99_test`, which replaces `4cc_90_test` as the default `cpk_name`. A numbered run counts
from 1; the stadiums run counts from 0, its first CPK being the base one. Each kind of midcup
has fifteen CPKs. And: every compile with a PES folder compares the installed list's entries
with the official one's, in order; any difference that still lists the run's targets is
`dpfilelist_not_official` (Warning), the run compiling and deploying all the same, with the
upgrade offered (GUI) or named (CLI) and never applied without the user's yes. A list
lacking a target of the run stays `dpfilelist_outdated` (Error), as before.
Why (maintainer): stadium CPKs get the plain `stadiums` stem; midcups are used one per
matchday and the longest cups have fifteen matchdays; a list that is not the official one
should be noticed, but the user may have edited it on purpose or be working with an old DLC,
so compiling for it must stay possible and replacing it must be asked first. Keeping the
Error for a missing target is the lead's reading: the game would not load a CPK the list
does not name, so deploying it would look like success and do nothing.
Plan: `team_compiler/pipeline.md` "DpFileList upgrade", "Multi-CPK mode";
`team_compiler/messages.md` `dpfilelist_not_official`; `team_compiler/README.md` TC-DEP-12.

## 2026-10-03 — team_compiler — the DpFileList upgrade writes a placeholder for each missing CPK
Decision: `upgrade-dpfl`, after replacing the list, writes the empty placeholder CPK for
every official entry that has no file in `download/`, and never overwrites a file that is
there. The default `refs_cpk_name` in the plan becomes `4cc_18_referees`, the official
list's name.
Why: the official list names 53 CPKs, more than any DLC ships and partly under new names, so
an upgraded install would list CPKs that do not exist. Every installed list measured (PES 15
to 20) names only files that are present, the unused slots as placeholders, so the game's
behavior with a listed CPK that is missing is unknown, and the multi-CPK writer already
avoids relying on it. Writing a 6,272-byte file where none exists cannot lose anything.
Plan: `team_compiler/pipeline.md` "DpFileList upgrade"; `team_compiler/settings.md`.

## 2026-10-03 — team_compiler — DpFileList: old CPKs renamed by stem, a missing CPK reported
Decision: three points. (1) `upgrade-dpfl` renames an old DLC's CPKs to the official names
by stem: the same stem, else the stem without trailing digits, else one of four aliases
(`faces` and `uniform` to `teams`, `other_faces` and `other_uniform` to `teams2`); within one
official stem the old files take the official names in list order; a file with no name left
is not renamed and is listed as no longer loaded; a rename never overwrites a file, and
every rename is shown before the user's yes. This replaces the plan's earlier "no
retired-names list is needed". (2) Every compile reports `dpfilelist_cpk_missing` (Warning)
for a listed CPK with no file in `download/`, other than the run's own target. (3) Confirmed
by the maintainer: a list lacking the run's CPK stays `dpfilelist_outdated` (Error), and the
upgrade writes placeholders.
Why: (1) the maintainer asked for a simple stem-based renaming, so a DLC from an old cup
keeps working under the official list as a best effort. The four aliases and the digit rule
are the lead's additions: without them the largest CPKs of every old DLC (`4cc_40_faces`,
`4cc_30_stadiums0`) match no official stem and the renaming would not serve its purpose.
(2) the maintainer's knowledge of the game: when any listed CPK is absent, PES rejects the
whole download folder and runs vanilla, a state a user cannot diagnose from the game, and the
compiler already reads the list and the folder.
Plan: `team_compiler/pipeline.md` "DpFileList upgrade"; `team_compiler/messages.md`
`dpfilelist_cpk_missing`; `team_compiler/README.md` TC-DEP-13, TC-DEP-14.

## 2026-10-03 — team_compiler — the deep pass runs on the worker pool, within one export
Decision: the deep pass checks an export's model folders and `Common/` files in parallel on
the run's worker pool, the files of one folder in parallel too, and keeps its findings in
file order. Three choices the plan did not make: (1) exports are still checked one after
another; (2) the file a worker holds is not charged to the memory budget; (3) routing reads
the sources (listing and metadata) in parallel.
Why: measured at 4.7 (worklog), the serial pass is 18% to 47% of a folder compile, and the
maintainer asked for the pool, as the old compiler had one per model folder, and for more
parallelism where it costs little code. (1) Under rayon a worker waiting on its nested work
steals other work; an export's check started that way can block on a `.7z` permit held by
the export suspended below it on the same thread, which never resumes. Each export already
uses every core for its own files, so the loss is small. (2) At most one file per worker
thread is held, a bound that needs no permit; charging each file would add a second permit
beside a `.7z`'s whole-archive one, the wait compile's coordinator already had to design
around. (3) Routing's per-source read starts no nested work, so blocking on a permit there
cannot deadlock.
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps", "Deep format pass".

## 2026-10-03 — team_compiler — folder and zip exports are checked in parallel; only `.7z` ones in turn
Decision: this replaces point (1) of the entry "the deep pass runs on the worker pool,
within one export". Exports held as folders or `.zip` files are checked in parallel with
each other on the run's worker pool (structure pass, deep pass, identity); `.7z` exports are
checked one after another, once the others are done. Findings are still reported per source
in discovery order.
Why: the maintainer asked whether checking exports in turn would slow a cup's compilation,
whose exports are almost never kept compressed. It would: an export whose check is one large
model keeps one core busy while the others idle. The deadlock that ruled parallel exports
out needs a check that waits for memory, and only a `.7z` does (its whole-archive permit);
a folder's or a zip's check never waits, so nothing can be held below a waiting one.
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps", "Deep format pass".

## 2026-10-03 — team_compiler — a blank face folder always takes the bundled face diff
Decision: a player with no face model gets the blank face folder with the bundled template
`face_diff.bin`, whatever `face_diff.bin` its folder holds.
Why: the maintainer's ruling. Across the VGL26 exports 467 face folders hold a
`face_diff.bin` and no model (a full-body model is loaded through the boots folder, which
supports a custom skeleton); 122 hold the bundled template and the other 345 one of two
older files, which are outdated FPC-dedicated diffs. How a `face_diff.bin` left in such a
folder is reported is step 4.12's.
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps" step 4.

## 2026-10-03 — team_compiler — the official DpFileList's 53 entries hold on every PES version
Decision: the official list keeps its 53 entries for every version, with no per-version
shorter list and no further in-game test of its length.
Why: the maintainer's ruling after two in-game tests, one per engine. PES 2015 and PES 2021
each loaded the last CPK of a 53-entry list (the install's own list extended with midcup
entries and placeholder CPKs, the test CPK last), where the longest installed list had 45.
PES 2016 to 2020 were not tested and are taken to behave like the two around them.
Plan: `team_compiler/pipeline.md` "DpFileList upgrade" (the entry list's paragraph).

## 2026-10-04 — workflow — `perf` is a commit type
Decision: `perf` joins the commit types, for a change made for speed or memory that changes
no behavior (4.7e's commit, the deep pass on the worker pool, is the first).
Why: the maintainer's ruling: it is one of Conventional Commits' usual types, and speed is the
suite's primary goal, so such commits should be findable (`git log --grep '^perf('`) rather
than filed under `refactor`.
Plan: no plan edit needed; `CONTRIBUTING.md` "Commits" lists the types.

## 2026-10-04 — workflow — one `fix` type; the changelog line tells a released bug's fix apart
Decision: no second commit type for a fix to a bug a release shipped. From the first release
on, a commit that changes something a member notices against the last release adds its
`[Unreleased]` line to `CHANGELOG.md` itself, and a fix for something not yet released adds
none; the lead's review before a commit checks for the line.
Why: the maintainer asked whether the two kinds of fix should be two types, since only one
kind belongs in the next changelog, and ruled against the split. A type would cover fixes
only, while features, changed behavior and speed-ups need entries too; it is not a usual
Conventional Commits type; and a wrong type on a pushed commit stays wrong, where a missing
changelog line is one more commit.
Plan: `core/distribution.md` "Changelog and version display"; `CONTRIBUTING.md` "Commits";
`AGENTS.md` "Working documents" (after a step).

## 2026-10-04 — team_compiler — a source's metadata is read by its check, not by routing
Decision: routing only lists each source. The small metadata files are read at the start of
the source's check, through the `ContentSource` the deep pass then reads from, for every
source kind. A `.7z` is so decompressed once for the metadata, the structure pass and the
deep pass, under one permit; its tasks' decompression in `compile` stays.
Why: routing read the metadata of every source before the first check, so a `.7z` holding a
metadata file (`players.txt`, `refs.txt`, `notes.txt`) was decompressed there and again by
its deep pass, about 2 s each on a 0.6 to 1 GB export (4.7f's timing: `check` on such an
export went from 4.2-4.4 s to 2.3-2.4 s). Routing never uses the metadata: the disabled marker is in the listing, and
the balls and duplicate-refs rules go by the team name. One read path for all three kinds,
not a `.7z` special case, because the folder and `.zip` reads are the same calls either way
and a second path would be code with no gain. The buffer is not kept for the tasks: planning
needs every export validated first, and holding every `.7z` until then is the residency the
budget forbids.
Plan: `libs/pipeline.md` "What a solid `.7z` is charged"; `team_compiler/pipeline.md` "2.
Per-export serial steps" (deep format pass, its last paragraph).

## 2026-10-04 — team_compiler — kit colors merge into a team's UniColor record by kit number
Decision: a committed kit's entry replaces the entry of its kit number in the team's
`UniColor.bin` record or joins the others; the record is then written with its count, its
entries in ascending kit number, and unused entries after them. A record whose counted
entries repeat a kit number is the base game's placeholder and is read as holding no kit. A
merge past ten entries leaves out the highest-numbered. The `colors.txt` reader lives in
`aesthetics_export` (`colors_txt.rs`), and the deep pass reports `color_entry_invalid`.
Why: the plan says a kit's entry applies only when its task commits and TC-BIN-05 keeps an
installed p1 entry when an export holds only p2, but not how one entry joins a record. Red
rewrites the whole record from the Note, which lists every kit of the team; a Studio export
holds its colors per kit folder, so a midcup export with one kit would wipe the others.
Reading the placeholder as empty, not as a white kit 0, is what keeps a first compile's
record byte-identical to Red's (measured on the tracer: Red's record for team 792 holds the
goalkeeper kit alone under a count of 1). Not settled, for the maintainer: a kit a team no
longer has keeps its entry, as its `UniformParameter` entry does; if the record's count is
what makes the game offer a kit, a full export should rather replace the record. The reader
is in the export crate, not the compiler, because the Kit config editor reads the same
files, and tool crates do not depend on each other.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps" (Bins accumulation);
`aesthetics_export/player_folders.md` "Root files" (Colors); `aesthetics_export/object_model.md`
(module tree).

## 2026-10-04 — team_compiler — kit colors derive from the effective main texture; the missing pair's bytes
Decision: when a kit's `colors.txt` gives fewer than two valid colors, the colors are derived
from the `kit` texture of the kit's effective set, so a main texture inherited from `all/`
is used like the kit's own. The "no colors chosen" pair is magenta (255, 0, 255) as the first
color and black (0, 0, 0) as the second.
Why: the plan said "the kit's own main texture, never the placeholder checkerboard" and named
the pair without its bytes or order. "Own" there is opposed to the placeholder: an inherited
`all/kit.dds` is the texture the kit wears in the game, so its colors are the kit's, and
writing the loud pair for a kit that has a real texture would report a gap that is not one.
The pair takes the placeholder texture's two colors in the order its name gives them
(`resources/kits/README.md`: the top-left check is magenta).
Plan: `team_compiler/pipeline.md` "Resolved decisions" (Kit colors fallback).

## 2026-10-04 — fpc, kit_config, team_compiler — 4.9a: the FPC kit values take no version; the template carries them
Decision: (1) `fpc::kit_values()` takes no version and returns the four values as `u8`;
`kit_config::apply_fpc(&mut config)` and `matches_fpc(&config)` lose their version too.
(2) A generated kit config is the template whatever the team's kit-FPC status; the template
carries the FPC values. (3) The team's kit-FPC status is read from the player folders of the
validated export. (4) `kit_config_version_clamped` is reported by the deep pass, at `check`
and at `compile`, on the config file, for `kit_config`'s `kit_value_out_of_range` and
`kit_pattern_unsupported_pes15` findings; `kit_config`'s other findings stay unreported for
now. (5) Step 4.9 lands as two slices: (a) FPC reconciliation and the clamp warning, (b)
collars, which waits on the maintainer's confirmation of the stock collar sets.
Why: (1) the worklog step asked for the values "on every version, without the `Option`", which
leaves a version parameter nothing reads; that parameter is how PES 15 and 18 got a wrong
`None`, and the `u16` fields forced casts at every use. (2) `fpc_toggle.md` said a generated
config gets "the plain defaults" when the status is Unknown, while `libs/fpc.md` said the
template already carries the FPC values; the code's template (the cup's generic kit config)
does carry them, there are no other defaults to fall back to, and most of the cup's teams
are FPC teams. (3) A folder validation dropped compiles no player. (4) The Kit config editor
plan calls the finding "a compile/check warning", and the fields the clamp covers are not the
ones FPC reconciliation changes, so checking the supplied config before reconciliation
loses nothing. The other findings (`kit_collar_zero`, the model and sleeve ones) have no
row in `messages.md`; what a kit with a zero collar should do needs a look at real configs
first. (5) The collar half is game-facing and needs the maintainer.
Plan: `libs/fpc.md`; `aesthetics_export/fpc_toggle.md` "Team kit-FPC status and kit configs";
`team_compiler/pipeline.md` "4. Per-export non-model steps" (Kits).

## 2026-10-04 — team_compiler — the kit layout table covers the socks alone, in two bands
Decision: (1) `KIT_LAYOUT_REMAP` has four entries, two bands per sock island (left sock:
pre-Fox u 8–168 ↔ Fox 8–128, pre-Fox 168–448 ↔ Fox 128–376, v 632–1160; the right sock
mirrored), and none for the shorts. (2) Socks are paired across engines by the angle around
the leg and by the arc fraction of the ring, per row, not by the nearest 3D point. (3) The
rectangles' outer edges are multiples of 8, each band is resampled along u from its own
rectangle, every mip level the source carries gets the same move, and a block-compressed
source keeps the blocks no destination rectangle touches. (4) The step's golden is a text
file of stripe centres (where the models put each stripe of a striped pre-Fox kit), tested
with a tolerance, not an image compared exactly. (5) The table lives in
`processing/kit_layout.rs`. (6) The number and name textures are left alone, although the
games' stock ones are arranged differently per engine.
Why: (1) The plan and the 2026-10-03 entry ("the games' uniform models are the kit layout's
source of truth") held that the models shift the Fox shorts by 36 to 60 px where PES Master
draws them alike, and ruled that the models win. The models still win, but that reading of
them was wrong: on the shorts u runs along the body's height, the Fox body stands 33 to 44 mm
taller at the hip, and a nearest-point match turned that into a shift. The garment's own
landmarks (crotch notch, waist), its height fractions and its angles agree between engines,
so the shorts are laid out alike and PES Master was right. (2) The same nearest-point match
gave the socks an outer band at exactly −60 px; measures that do not depend on the body's
size show a smooth map instead (slope about 0.7 rising to about 0.93), which two bands
approximate to 6 to 10 px rms, against 10 to 15 for one scale. A third band would chase a
difference smaller than the one between the two measures. (3) Edges on block boundaries let
the rest of a DXT kit through without a second compression; per-band resampling keeps a
flat band flat; treating every level alike needs no mip generator and keeps authored mips.
(4) The table is an approximation by a few px of a map that itself varies by about 10 px
with v, so an exact image golden would only compare the table with a copy of itself; stripe
centres from the models catch a wrong number, direction or mirror, which is what can go
wrong. (5) `processing/` has `kit.rs`, not a `kits/` folder; a sibling module avoids moving
it. (6) Re-arranging a digit atlas is a different operation with its own question (does
either game read the other's arrangement?), which only an in-game test answers.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps" (Kits, "Layout
conversion"); `aesthetics_export/player_folders.md` "Kit layout marker"; `team_compiler/README.md`
TC-KIT-18, TC-KIT-19; `testing.md`; worklog step 4.10 and "Phase 4 open questions".

## 2026-10-04 — dds_convert — the suite's one resampler is `dds_convert::resize`
Decision: `dds_convert` gains `resize(pixels, width, height, new_width, new_height)`,
Lanczos3 over straight-alpha RGBA8 through the `image` crate. The kit layout conversion
(4.10) calls it per sock band, and it converts the re-laid texture with
`dds_convert::convert` directly, outside the `Converter` cache.
Why: the plans say "Lanczos3, the compiler's one resampler" (kits, portraits, the logo)
without saying where it lives. `image` is already `dds_convert`'s dependency for raster
sources; a function there serves every tool without each one depending on `image`, and a
hand-written kernel in the Team compiler would be a second resampler by 4.11. The cache is
keyed by the source file's hash, and a re-laid kit is no longer its source file; such kits
are the exception, so they are converted uncached instead of teaching the cache a second key.
Plan: `libs/dds_convert.md` "`dds_convert` API"; `team_compiler/pipeline.md` "4. Per-export
non-model steps" (Kits, "Layout conversion").

## 2026-10-04 — team_compiler — the logo's geometry, its findings and its PNG encoder
Decision: (1) `fit` resamples the image with its proportions kept (long side to N, short side
to N × short / long rounded half up, at least 1) and then centres it on a transparent N²
canvas; `crop` cuts the long side around its centre, then resamples; `stretch` resamples
straight to N × N. Each of the three sizes is resampled from the source. (2) A square source
reports no `logo_fit_applied` whatever its tag. (3) `logo_upscaled` is reported per source
file when a side is stretched to reach the file's largest target: the long side under it for
`fit`, the short side for `crop` and `stretch`. (4) The PNGs are written by a new
`dds_convert::encode_png`. (5) The logo is one task per export, scoped on the main file, in
`processing/team_assets.rs`. (6) Step 4.11 is worked as four slices: the logo, the notes, the
kit variants, the texture `.common` links.
Why: (1) The plan said "made square per its fit tag, resampled", which read literally pads
first; Lanczos over a padded straight-alpha image blends the logo's edge with transparent
black and leaves a dark fringe, where resampling first keeps the edge as drawn. Resampling
each size from the source avoids filtering twice. (2) Nothing is made square. (3) The plan
said "smaller than its largest target" without saying which side of a non-square source
counts; the side that decides the scale factor is the one each mode maps onto N. (4) As for
`resize`: `image` is already `dds_convert`'s dependency, and a tool crate depending on it
for one encoder would be the second place the suite touches rasters. (5) The three files
are one atomic producer, which a task is; the worklog step already names the module. (6)
The four parts share no code and each is a reviewable diff of its own; one brief would pass
the 500-line limit.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps" (Logo);
`team_compiler/messages.md` (`logo_fit_applied`, `logo_upscaled`); `libs/dds_convert.md`
"`dds_convert` API".

## 2026-10-04 — team_compiler — `teamnotes.txt`: its layout, when it is written, and its failure
Decision: (1) One entry per export run planning keeps: a header line `--- /co/ ---`, the note
(LF line ends, leading and trailing blank lines removed), one empty line between entries.
(2) It is written once the run's CPK is in place; a run that writes no CPK leaves the
previous file. (3) A run that writes its CPK and collects no note removes a previous
`teamnotes.txt`. (4) A failure to write or remove it is a new code, `teamnotes_write_failed`
(Error, on the run); the CPK stays. (5) The note's text is read during validation, while
the source is open, as the root `colors.txt` is.
Why: (1) The plan said "under a team header" without a layout; Red's (`. `, `- `, `--
{team}'s note file:`) is console decoration, and a header that names the team as every
message does is enough. (2) "Rendered only after final export outcomes are known", and a
notes file describing a CPK that was never put in place would mislead. (3) TC-ROOT-09 says a
run with no notes writes no file; leaving an older file would show notes of exports this
run did not compile, and Red removed the file at the start of every run. It is the
compiler's own artifact, in its own output folder. (4) No code covered it:
`output_commit_failed` is Fatal and says the CPK is not in place, which would be false. (5)
A `.7z` is not decompressed again for a few lines of text.
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps" (5, Notes collection);
`team_compiler/messages.md` (`teamnotes_write_failed`, `notes_found`).

## 2026-10-04 — team_compiler — kit variant sets are found from the files, and completed by the textures task
Decision: (1) A variant is a file whose stem holds the token `kit1` to `kit9`, spelled
exactly so; the files of one folder differing only in that digit are one set. (2) The task
converting a folder's textures completes each texture set against the export's kit numbers
(`p1` to `p9` of `Kits/`), copying the lowest variant's converted bytes and reporting
`kit_variant_missing` once per set and number, whether or not a model references the set.
(3) A model's path naming `…kitN…` is pointed at the folder's texture home when any variant
of the set is in the folder. (4) On Fox a model file that is a higher variant of a set in
its folder is left out at planning, which reports `kit_variant_model_fox`. (5) TC-CMN-06
(`dummy_kit*` skipped by the texture-existence checks) moves to step 4.29, which builds
those checks; until then no check exists for it to skip.
Why: (2) `model_format.md` words the rule by reference ("a `kitN` reference whose variant
is missing"), but the textures task does not read models, and a set nobody references is a
member's leftover at worst: one copied texture and a warning that points at it. Reading
every model's path table a second time to spare that case is not worth it. (1) The modded
exes match the spelling as written, so a differently cased token would not be found in
game either. (3) Without it the path keeps the directory the model was exported with, and
the game looks for the variants in the wrong place. (4) Which model files are parts is
planning's decision already. (5) A scenario that passes because the code it constrains
does not exist proves nothing.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps" (Kit-dependent assets);
worklog steps 4.11 and 4.29.

## 2026-10-04 — team_compiler — TC-CMN-05 is split by engine; TC-TEX-09 covers texture links
Decision: TC-CMN-05 keeps its PES 21 half (per-kit model files reduced to the lowest); its
PES 17 half becomes TC-CMN-07, with native `.model` sources. TC-TEX-09 is new: a texture
`.common` link points the player's model at the team's Common output.
Why: the acceptance scanner counts a scenario proven as soon as one test cites its ID, so a
scenario spanning two engines read as proven once its Fox half was, with step 4.14 still to
build the other. The old wording also compiled `.fmdl` sources for PES 17, which needs the
cross-format conversion of 4.17 and is not what the scenario is about. Texture links were
described by the plan (`model_format.md` "Link files") with no scenario and no step (found
at 4.5c).
Plan: `team_compiler/README.md` "Acceptance" (TC-CMN-05, TC-CMN-07, TC-TEX-09); worklog
steps 4.11 and 4.14.

## 2026-10-04 — team_compiler — a face file with no face model is `face_file_not_used`; who gets a blank face
Decision: (1) A `face_diff.bin`, `face_diff.xml` or `fcl_hair_sim.fclo` in a player folder
with no face model, under `ingame_face` or not, is not read and is reported as
`face_file_not_used` (Info, on the folder, context `file`), by `check` and `compile`. (2)
Every roster-mapped player folder without the marker and without a face model gets the blank
face folder, whatever else it holds (a portrait alone, a boots link alone, an empty `face/`,
nothing). (3) Under the marker a model that would go into `fcl_hair`, one named `fcl_hair`
included, becomes a boots part with no finding of its own.
Why: (1) the maintainer ruled that a blank face always takes the bundled diff and left the
report to step 4.12. The output is the same with or without the file, so it is not a
warning; 467 face folders of the VGL26 exports hold one (345 an outdated FPC diff), and a
member who believes the file does something should be told once. (2) Red emits a face folder
for every player folder without the marker (`players_process.py` `copy_folder_contents`: "an
empty face category still gets its face folder emitted"), and the plan's "as a player with
no face models does" says the same; the step's wording named only the boots and gloves case.
(3) The plan lists `face_high`, `hair_high` and `oral` as the explicit face names; the marker
is the member's own instruction, so the reroute needs no report.
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps" step 4;
`team_compiler/messages.md` `face_file_not_used`; `aesthetics_export/player_folders.md`
"`ingame_face` marker"; `team_compiler/README.md` TC-MOD-32.

## 2026-10-04 — team_compiler — a failed package is left out alone, also beside a blank face (TC-MOD-09 reworded)
Decision: TC-MOD-09 now says the conflicting boots are left out of the CPK while the folder's
blank face and its textures are still packed; the `merge_material_conflict` and
`skl_merge_conflict` rows say "that package left out". The writer is unchanged.
Why: the writer's rule is that a player folder's packages that succeeded commit with its
textures (`pipeline.md` "3. Per-model-folder parallel steps", step 6), and the help already
says a merge conflict "leaves that folder's face, boots or gloves out". TC-MOD-09's "each
folder is dropped" was true only because its folders held nothing but boots; since step 4.12
every unmarked folder has a face package, blank without a face model, which succeeds. The
sidekick recommended the opposite, the writer dropping the whole folder when one package
fails, so a player never shows a blank head over the stock body; that changes the rule for a
real face beside failed boots too, and is left to the maintainer (worklog "Phase 4 open
questions").
Plan: `team_compiler/README.md` TC-MOD-09; `team_compiler/messages.md`
`merge_material_conflict`, `skl_merge_conflict`.

## 2026-10-04 — team_compiler — `duplicate_aesthetics_export` is the validation pass's, and 4.13 lands in three slices
Decision: exports resolving to one team ID are found by the validation pass, after every
export's identity is resolved, not by run planning: each conflicting export gets the Error
(context `id` and `exports`, the conflicting sources' file names) and is not planned. An
export validation dropped earlier does not count. The canonical export order is discovery's
existing order (the source's file name, case-folded, then as spelled). Step 4.13 is worked as
4.13a (this, with TC-PLN-03, 05 and 07), 4.13b (the `overrides/` tree and `duplicate_path`)
and 4.13c (source pinning).
Why: the plan put the check in run planning, which only `compile` runs, so `check` would
pass a root that `compile` then refuses; identity is resolved in the validation pass both
commands share, and nothing about the check needs a manifest. The file name already carries
the source kind as its extension, which is the plan's "path plus source kind" key.
Plan: `team_compiler/pipeline.md` "3. Per-model-folder parallel steps" (the run-level
paragraph); `team_compiler/messages.md` `duplicate_aesthetics_export`.

## 2026-10-04 — team_compiler — the `overrides/` tree is the writer's; no path preflight over the tasks
Decision: (1) Every file below the data directory's `overrides/` is a CPK entry at its
relative path; the writer adds them first, in path order, and they create the CPK even when
no export commits. An entry a task or the bins bring at an override's path is left out and
reported as `duplicate_path` (Warning, on the run, context `path`); the rest of the task is
kept. `overrides_active` (Info, on the run) names the folder and the file count, from
`compile` only. An override that cannot be read fails the CPK (`cpk_write_failed`). (2) The
folder and export forms of `duplicate_path`, and the planning preflight that would allocate
every task an output namespace, are not built.
Why: (1) the plan gave the precedence rule and left the mechanics open. The writer already
sees every path once, so the override check is one lookup there, with no need to predict a
task's paths at planning (a package's file names are known only to its task). (2) No two
tasks can claim one path today: paths are keyed by the team's ID, a player ID, an ID of the
team's block, a kit name or a player folder's name; two exports of one team are refused
since 4.13a; in-export collisions are refused or merged before planning; collars have their
own finding. The sidekick's reading at 4.13a found no counterexample. A preflight for a
collision that cannot happen would be code with no test that can fail; the writer's
duplicate invariant (`cpk_write_failed`) stays as the backstop.
Plan: `team_compiler/pipeline.md` "5. Writer" step 1 and "3. Per-model-folder parallel
steps" (the run-level paragraph); `team_compiler/messages.md` `duplicate_path`,
`overrides_active`.

## 2026-10-04 — team_compiler — a source's revision is the listing's, checked after each task's read
Decision: (1) The revision is taken by the reader's listing (a folder: each file's size and
modified time; an archive: the archive file's), not by a second pass at planning. (2)
`compile` checks it after each task's read, not before: the files that task read, or the
archive file. A file that is gone or differs aborts the run with
`source_changed_during_run` (Fatal, on the export, context `path`), the staging output
discarded. (3) Only what a task reads is checked; a file added after the listing is not a
change. `check` verifies nothing. (4) A file that is still there, unchanged, and cannot be
read stays `source_read_failed` on its folder.
Why: (1) the plan said "pinned at planning", and also that the revision is "what the reader
already lists"; the listing already asks the file system for each file's metadata, so
taking the modified time there costs nothing and covers the time validation takes, which a
pin at planning would leave open. (2) The plan said both "around each folder
materialization" and "before each task's read". A check before the read misses a file
saved over during the read; a check after it catches that and everything earlier, for the
same one lookup per file. (3) The plan's revision was the whole set of paths; checking the
whole tree before every task would cost a walk of the export per task, for files no task
reads. (4) An unreadable file is not a changed one, and dropping its folder is what the
member can act on.
Plan: `team_compiler/pipeline.md` "Resolved decisions", Source snapshot;
`team_compiler/messages.md` `source_changed_during_run`.

## 2026-10-04 — team_compiler — the maintainer's answers to four Phase 4 open questions
Decision (maintainer): (1) An override's path is compared exactly; no case folding. (2) A
failed package is left out alone, also beside a blank face (the entry on TC-MOD-09 stands),
on the condition that the error is reported visibly enough for the member to know the
folder is not usable as a whole: it is an Error on the player's folder, and the help now
says the player shows with that part missing. (3) Neither engine reads the other's number
atlas arrangement: a `_back`, `_chest` or `_leg` atlas in the other engine's arrangement is
re-arranged for the target (worklog step 4.32). (4) The kit config template stays one file
for every version: its values are taken from the most common FPC-compatible kit config on
the maintainer's machine whose `name.y` is 0 to 16, so no target clamps it.
Why: (1) overrides are rare, and an override is a file taken out of a CPK under the name
it had there. (2) Dropping the whole folder would also drop a real face beside failed
boots; what matters is that the member sees the folder is broken. (3) The maintainer's
knowledge of the games. (4) Per-version files are avoided where one file can serve.
Plan: `team_compiler/pipeline.md` "5. Writer" step 1 (exact paths), "4. Per-export
non-model steps" (number atlases); `team_compiler/messages.md` `merge_material_conflict`.

## 2026-10-04 — kit_config — `name.y` is PES 21's value on every version; the template stays
Decision (maintainer): the kit config template is kept as it is, superseding point (4) of
the entry above: a member who supplies no config does not care where the name sits. The
crate takes `name.y` as PES 21's value (0 to 39) and an older game holds half of it,
instead of clamping 30 to 16. Lead's reading of "halve or double": PES 21's 6-bit field is
the older 5-bit field plus one lower bit, so the same bytes decode to twice the older
value and nothing has to be converted: one decode and one encode for every version, bit 3
of 0x1C carrying the value's low bit on every version (most older configs already have it
set), and only the maximum differing (33 on PES 15 to 20, which is their 16).
`kit_config_version_clamped` stays for a value over 33 on those games.
Why: the census of 4,930 configs on the maintainer's machine showed the template's bytes
are the community's usual ones (15 read the old way, 30 the PES 21 way); the clamp came
from reading them as PES 21's and writing that number on an older game. With one reading,
a config moves between versions with the name in the same place, an old binary converts
without knowing which game it was made for (the Export upgrader), and Red's bytes come out
on every version.
Plan: `kit_config_editor.md` (binary layout rows 0x1C and 0x1D, "Version differences",
"Version-neutral"); `team_compiler/README.md` TC-KIT-17 (reworded), TC-KIT-24 (new).

## 2026-10-04 — aesthetics_export, team_compiler — a team export's name carries `Full` or `Midcup`
Decision (maintainer, details settled with the lead): the second word of a team export's
name, right after the team name, is its coverage tag, `Full` or `Midcup` in any letter
case; a name with neither is the Error `export_tag_missing` and the export is skipped. A
`Full` export rebuilds what the compiler holds for its team from the export alone (the
`UniColor.bin` record and its count, the team's kit configs, its players' boots and gloves
rows, later the savefile fields); a `Midcup` export replaces only what it holds. Referee
exports carry no tag and are always full. Two exports of one team in one run stay refused
(`duplicate_aesthetics_export`), a `Full` and a `Midcup` one included. The Export upgrader
writes `Midcup` for an old name holding `midcup` or `additions`, `Full` otherwise. The GUI
shows a **Full** and a **Midcup** button on an untagged export's row, in the place of its
player cells, which rename the export.
Why: the record's count is what makes the game offer a kit, so a team that drops a kit
keeps offering it unless a full export resets the record, while a midcup export holding
one kit must not wipe the others; nothing in an export's content tells the two apart. The
tag is explicit and in the name so a mistake is seen before it is compiled; the second
word is one rule beside "the first word is the team", and cannot match a description by
accident. Cups compile full exports into the main CPKs and midcup exports (sometimes a
full one, after an overhaul) into the midcup CPKs, never both for one team in one run.
Supersedes the open question left by "kit colors merge into a team's UniColor record by
kit number".
Plan: `aesthetics_export/object_model.md` "Validation semantics" (Coverage tag, the
`coverage` fields, `ExportCoverage`); `team_compiler/pipeline.md` "Bins accumulation";
`team_compiler/messages.md` `export_tag_missing`; `team_compiler/gui.md` (Untagged
export); `export_upgrader.md` "CLI"; `team_compiler/README.md` TC-ID-05 to 07, TC-BIN-14
to 17 (new), TC-ID-01, TC-PLN-03, TC-BIN-05 and 06 (reworded); `GLOSSARY.md`.

## 2026-10-05 — aesthetics — the names patch is opt-in by export; the seed rows live in `4cc_08_bins`
Decision (maintainer, from the S5 review of step 4.0's plan rewrite): on Fox the aesthetics
patch is applied whole, with no team picker: a managed team's players keep their manager's
names because its export leaves `name` and `shirt_name` out of `settings.toml`, while an
autopilot team's export sets them; so the patch never names a managed team's players. The
Save editor's strip-and-seed puts its seed rows of `PlayerAppearance.bin`, `BootsList.bin` and
`GloveList.bin` into `4cc_08_bins.cpk`, the official DpFileList entry that holds the cup's
team-related bins (`TeamColor.bin`, `UniColor.bin`, `UniformParameter.bin`), not into a CPK
of its own.
Why: the plans said "applied for autopilot teams, not for managed teams" with no mechanism,
and one compile writes one patch for every team; name writing is already opt-in per
`settings.toml`, so no new mechanism is needed. The seed's place was unnamed, and the bins
walk reads only the CPKs the installed DpFileList lists, so an unnamed seed CPK could be
silently left out; `4cc_08_bins` is already listed and already the bins' home.
Plan: `aesthetics_export/settings_toml.md` (name rules), `pes_savefile/operations.md`
"Aesthetics patch", `save_editor.md` "Stripped save", `team_compiler/pipeline.md` "Bins
accumulation".

## 2026-10-05 — pes_savefile — populated records carry their own appearance id
Decision (lead, reversible): from Phase 5, `populate_players` also sets each new record's
appearance-block player id to the player's own id, as real saves hold it; -1 is written only by
the Save editor's strip-and-seed.
Why: after step 4.0 that id decides whether the game reads the record or the tables, and a
template's leftover id would be neither the player's nor -1, a state no in-game run covered;
keeping "only strip writes -1" leaves one writer of the stripped state (review S5.6).
Plan: `pes_savefile/operations.md` "Player section population".

## 2026-10-05 — team_compiler — the aesthetics patch is written on every compile
Decision (lead, reversible): on Fox too, the Team compiler writes `aesthetics_patch.toml`
beside its CPK on every compile, not only when it resolved a name; a patch with no names
writes nothing when applied.
Why: skipping the file left the previous run's patch beside the new CPK, so a team that
removed its names would still be renamed by whoever applied the file found there; writing
it always keeps "the patch describes the CPK beside it" true with no deletion step, and is
the pre-Fox rule already (review S5.A1 item 9).
Plan: `pes_savefile/operations.md` "Aesthetics patch", `team_compiler/pipeline.md`
"Post-processing".

## 2026-10-05 — team_compiler — "every compile" means every CPK a compile publishes
Decision (lead, reversible), narrowing the entry above: the patch is written beside every
CPK a compile publishes; a run that publishes none (every export skipped, the only player
rejected, a failed write) leaves the previous CPK and its patch alone.
Why: those runs keep the previous CPK by design (TC-OUT-02 to 04), and a new patch beside
it would break the pairing the entry above exists for (review S5.A2 item 3).
Plan: `pes_savefile/operations.md` "Aesthetics patch", `team_compiler/pipeline.md`
"Post-processing".

## 2026-10-05 — team_compiler — a custom collar is set as the collar and the winter collar
Decision (lead, reversible): a `Collars/collar_<ID>` model sets both the collar and the winter
collar of every kit config of the team to its ID.
Why: the plan said "the collar", and the point of the folder is the custom model on every
player at once; a winter kit left on its stock collar would show the old model in winter
matches, and `kit_collar_reserved` already checks both fields after the rewrite (review
S6.1b item 10).
Plan: `team_compiler/pipeline.md` "Collars"; TC-CMN-01.

## 2026-10-05 — team_compiler — a user face.xml's fill-ins get no finding
Decision (lead, reversible): in a user-supplied `face.xml`, the team ID put in place of a
Common path's 3-character subfolder and the `<dif>` block inserted from `face_diff.xml` are
applied without a finding, as in a generated xml; the reported Info rewrites are the
`face_neck` dummy and `uniform_sub`. This narrows the 2026-09-12 entry's list of reported
rewrites.
Why: the catalog prose promised findings for both while no code existed for either; the user
wrote the placeholder and the `face_diff.xml` for the compiler to fill in, so a line saying it
did tells them nothing, whereas the other two change what they wrote (review S6.2a minor 3).
Plan: `team_compiler/messages.md` "User-supplied `face.xml`".

## 2026-10-05 — team_compiler — a stem's renames include its files already under official names
Decision (lead, reversible): when `upgrade-dpfl` renames an old DLC's CPKs by stem, the old
files of a stem are all its installed files once one of them needs a name, those already
under an official name included, dealt the official names in list order; the renames run
last first so none lands on a file not yet moved.
Why: the plan's example (`4cc_60_midcup` to `4cc_74_midcup` become 61 to 75) shifts files
that already carry official names, while its next sentence kept such a file in place, which
would leave `4cc_60` with no name and drop a DLC's first matchday (review S6.3b item 3).
Plan: `team_compiler/pipeline.md` "DpFileList upgrade"; TC-DEP-13.

## 2026-10-05 — team_compiler — collars, the DpFileList outside deployment, rows, referee folders
Decision (maintainer), four answers:
- A collar is a model like any other: one in the other engine's format is converted to the
  target's (`collar_12.fmdl` compiled for PES 17 becomes `collar_012.model` plus its `.mtl`),
  its textures converted as a player model's are.
- In sideload and test mode a missing `DpFileList.bin` is a Warning, `dpfilelist_missing`,
  and the bins are built on the bundled bases; it stays an Error in a compile that deploys.
- A team player a `Full` export does not compile at all keeps his `BootsList`/`GloveList`
  rows as they are.
- A referee folder is a player folder except for its names (`refereeXXX` for `xxxXX`), its
  `face.xml` included: on pre-Fox its local boots and gloves are typed `face.xml` entries and
  no `k99XX`/`g99XX` folder is written; the game is modded to load `k99XX`/`g99XX` for slot
  XX and shows nothing there when the folder is absent.
Why: collars are handled the same way on both engines; a run that deploys nothing still
tells the user that no bin from the download folder was used; with `settings.toml` every
player needs a face folder, so a team player the export leaves out is an edge case; the
referee rule needs no exception (review S6.7, S6.10b item 2, S5.2 item 7, S6.A3 item 1).
Plan: `team_compiler/pipeline.md` "Collars", "Game paths reference" (referee rows);
`team_compiler/messages.md` `dpfilelist_missing`; `team_compiler/blue_port.md` "Referee export
processing"; TC-CMN-09, TC-OUT-18, TC-REF-09.

## 2026-10-05 — team_compiler — the PES 16 Common patch, kitN, refs' shared boots and gloves
Decision (maintainer):
- The PES 16 exe patch that loads models from Common does not exist yet; it is made when the
  cup next plays PES 16, and the plan treats it as existing.
- `kitN` works on Fox through FoxDen; pre-Fox gets it when the cup returns to a pre-Fox
  version.
- A refs export may hold shared `Boots/` and `Gloves/` folders; a referee's link resolves to
  his slot's `k99XX`/`g99XX`, the shared folder written as that folder for each linking slot.
- The per-version stock collar sets need no confirmation: they were counted on every install.
Why: the cup's next game decides which exe changes are made; a referee has no team block, so
his slot's own ID is the only one a link can take.
Plan: `team_compiler/messages.md` (the PES16 Common note); `model_format.md` "Kit-dependent
assets"; `team_compiler/blue_port.md` "Referee export processing"; TC-REF-10.

## 2026-10-05 — team_compiler — collars use the kit texture; autopilot marker; motions; seed rows
Decision (maintainer):
- A collar is drawn with the team's kit texture on both engines (pre-Fox: the base
  `uniform_config.xml` gives `nocloth` collars the `collar` type and the shared `uniform.mtl`;
  Fox: the exe assigns the type), so `Collars/` holds model files only and a pre-Fox collar
  ships no `.mtl`. This replaces the first entry of this date's "textures converted" clause.
- An export opts its players' names into the aesthetics patch with an `autopilot` marker at
  its root; without it `name` and `shirt_name` are not used, on both engines.
- The Fox patch carries every compiled player's motions, which no database table holds.
- A compile with nothing to patch writes no patch and moves an earlier one beside the CPK to
  `aesthetics_patch.toml.bak`, one backup replaced each time.
- In multi-CPK mode the player tables' walk starts at the bins CPK itself, so the seed rows
  survive the next full-cup compile.
Lead's reading: motion keys a file lacks are written with their defaults, as pre-Fox writes
every appearance field, so a Fox patch is skipped only for a compile with no player and no
autopilot export.
Why: the game ignores a collar's own materials; a marker shows at a glance which teams have no
manager and makes generated names harmless; motions are aesthetics, and the patch is their only
route on Fox; a stale patch must not sit beside a newer CPK.
Plan: `team_compiler/pipeline.md` "Collars", "Bins accumulation", "Post-processing";
`aesthetics_export/object_model.md` (allowlist); `player_folders.md` "Marker names";
`settings_toml.md`; `pes_savefile/operations.md` "Aesthetics patch"; TC-CMN-08..10.

## 2026-10-06 — aatf — rulesets as data with a generic interpreter in the file; the Ruleset editor
Decision (user):
- A ruleset is data: the `RULESET` map (tiers, heights, card economy, conditional specials,
  suggestions) evaluated by a generic interpreter written in Rhai and copied into every rules
  file, plus an optional hand-written `custom_checks(team)` hook the interpreter calls and the
  editor preserves. This replaces the 2026-09-19 shape (a `CFG` parameter map above hand-written
  check functions); the one-self-contained-file property, Rhai and the three host functions stay.
- The official ruleset is expressed in the same schema and run by the same interpreter.
- A new tool, the **Ruleset editor** (`ruleset_editor`), lets organizers with no programming
  experience build a ruleset through forms; its rulesets need not depend on the official one. It
  is a separate tool, Phase 20 (post-release); seeing a team's results live while editing is not
  needed. The schema and interpreter land in Phase 5 with `libs/aatf`.
- Visual programming (blocks or a node graph) is deferred until the egui libraries mature; a
  graph could later sit beside `RULESET` without a format change.
- VGL's suggestions are adopted as a third severity, derived from each allowance a ruleset lists
  in `suggestions`, behind a "Show suggestions" toggle.
Lead's reading: the format moves out of `save_editor.md` into its own plan, since three tools
consume it; VGL26 (4ccEditor-VGL `vgl26`) becomes the schema's second embedded ruleset and
fixture; the upstream errata the old section listed were fixed upstream in `f5e7b3e`, so they are
gone from the plan.
Why: `aatf.cpp`'s history shows every special since 2019 is a condition and an effect, and
VGL26 shows a complete invitational ruleset of a different shape, so a schema covering both lets
non-programmers make rulesets; the interpreter in the file keeps a ruleset's meaning fixed when
the official logic changes between Studio builds.
Plan: new `aatf_rules.md` and `ruleset_editor.md`; `save_editor.md` "Configurable AATF rules"
cut to a pointer plus the checker UI, verification pointer; `team_creator.md` "Heights";
`core/README.md` overview, `rhai` row, decisions row; `core/architecture.md` crate tree;
`core/development_plan.md` Phase 5 bullet, Phase 18 cheap tier, new Phase 20;
`model_format.md` "Comments are app-injected"; `plans/README.md`; `GLOSSARY.md`.

## 2026-10-06 — ruleset_editor — Randomize; the VGL26 test save
Decision (user): the Ruleset editor gets a Randomize button that generates a whole new ruleset,
with three presets: Sensible, Weird, Crazy. VGL26 parity is tested on the maintainer's
`EDIT00000000_VGL26` save.
Lead's reading: a random ruleset always admits a legal team. The generator draws witness teams
first, reads the tier counts and quotas off them, and verifies the witnesses with `check_team`,
redrawing what fails. Results are seeded and reproducible across releases through the tool's own
SplitMix64. A result opens as a new unsaved document. The presets' bounds are a table in the plan.
Why: a ruleset no team can satisfy is worse than none; a seed that changed meaning between
releases could not be shared.
Plan: `ruleset_editor.md` "Randomize", start page, CLI `random`, crate layout, verification;
`aatf_rules.md` VGL26 parity; `core/development_plan.md` Phase 20.

## 2026-10-06 — aatf — VTL11 checked against the schema; team settings; the VTL11 test save
Decision (user): VTL11's rule that the 188 cm players use at most two registered positions, with
no 180 cm player sharing one, is a custom check in VTL11's file, not schema. Team tactics settings
(man marking, auto substitution, offside trap, preset change) per PES version join the schema as
`team_settings`. VTL11 parity runs on the maintainer's `EDIT00000000_VTL11` save.
Lead's reading: the rest of VTL11 needed small, general additions rather than VTL-specific
constructs: conditions on playing style, held cards, A and B ratings by position and name
colour; COM styles as cards (Long Ranger free); `at_most` widened to form, injury resistance and
weak foot (VTL wants weak foot exact); `min_cards` per tier; `b_uses_a_allowance`;
`max_skill_cards` (VTL uses 11 on PES 21) and `max_com_styles` as universal fields. A missing
field takes its empty value, `max_skill_cards` being required. `pes_savefile` must model PES 16's
per-preset man-marking assignments before `man_marking` is implemented.
Why: VTL11 is the second-largest invitational's live ruleset; each addition is a general rule a
form can express, and the one rule that is not stays in the escape hatch.
Plan: `aatf_rules.md` "Background", the schema, the card economy, "How a player is read", the
official file, violations, suggestions, "The host", crate layout, verification;
`ruleset_editor.md` sections.

## 2026-10-06 — review process — the Clef scan joins the lead's review
Decision (maintainer): every diff the lead reviews, a sidekick's step and each rework round's
fix, gets a Clef scan (`just clef-diff`), and each phase's crates get a whole-crate scan at
converge (`just clef <crate>`). The lead runs it and rules each flag like a reviewer concern;
rulings are kept in `scripts/clef_rulings.md`. When the free tier's daily neuron limit is hit,
the work continues without Clef and the scans are queued for the first run after the UTC date
changes.
Lead's reading: the model is Clef (27B) on Workers AI, one question per 60-line window ("does
this code contain a bug"), flagged at P >= 0.7, then one question per line to point at it.
Measured on 4cc-studio code: 19 of 21 injected one-edit defects caught, 1 of 21 unmodified
windows and 2 of 64 reviewed hunks flagged, and its top flags among those hunks were the
`pes_savefile` text-codec code a later fix (`cd3a1d1`) changed. Not Clef-flash (4 of 21 caught
on the same question), not Jev (its answers to an open question flag half the clean windows;
it locates a known bug as well as Clef but adds a second provider for a tenth of a cent), not
several questions in one request (Clef's joint head answers the bug question worse in a
battery: 11 of 21). Production code only: the question was not measured on test code. Ruled
flags are keyed by the flagged line's file and neighbours so a dismissal is not raised again.
Why: the sweeps and mutation runs see patterns and missing tests; a logic slip that the code's
own tests agree with had no check before the cross-family reviewer, which is queued. The scan
costs about a third of a cent per step. Flags go through the lead, not straight to the
sidekick, because a sidekick handed raw flags fixes the false ones too.
Plan: `CONTRIBUTING.md` "Testing and verification" (recipes; "Clef scan"); `AGENTS.md` lead
review and converge's design-health pass. The evaluation is outside the repository
(`SystemOne/jev-eval/round3/RESULTS.md` on the maintainer's machine).

## 2026-10-06 — review process — the Clef scan's windows stay positional
Decision (lead, measured at the maintainer's request): the scan keeps 60-line windows placed by
position (centred on a change, or tiled 60/40), not windows cut to the enclosing function
with its doc comment and `impl` header.
Why: on the 21 injected-defect pairs, function-sized windows caught 15 at P >= 0.7 against 19,
flagged 3 unmodified windows against 1 (AUC 0.87 against 0.98): a short function alone hides
the neighbouring code a bug is judged against (an overflow guard, a buffer-size rule) and
makes correct code look suspicious. On five reviewed commits the positional windows flagged
10 of 203, every one on the `pes_savefile` codec code `cd3a1d1` later fixed, so the report
now merges overlapping flagged windows into one flag. The same run found that ending
production code at a file's first `#[cfg(test)]` skipped 5,007 production lines (out-of-
line `mod tests;` declarations, single test-only items); it now ends at the inline test
module, the last item of all 133 files that have one.
Plan: `CONTRIBUTING.md` "Clef scan".

## 2026-10-07 — aatf — VGL27 checked against the schema; `registered_position_fielded`
Decision (user): VGL27's rule that a starting-eleven player is fielded at his registered position
in at least one formation is a universal switch (`registered_position_fielded`), not a general
"fielded at" condition. VGL27 (4ccEditor-VGL `vgl27`, `45ce31d`) becomes the schema's fourth
fixture; VGL26 stays the template.
Lead's reading: the rest of VGL27 needs no schema change (exact-height brackets under one
system, the official manlet special, medal tiers excluding GK). Its parity runs on the VGL26
save's teams until a VGL27 save exists, comparing findings, not legality. The host gains a
`fielded_at` accessor and the findings a `RegisteredNotFielded` kind.
Why: one checkbox covers the rule as organizers state it; a general condition would take thirteen
specials, one per position, for the same rule.
Plan: `aatf_rules.md` "Background", the schema, "VGL27", violations, "The host", crate layout,
phase and verification; `ruleset_editor.md` "Randomize"; `core/development_plan.md` Phase 5.

## 2026-10-07 — aatf — Autumn 25 and Spring 26 as test rulesets; official parity per ruleset
Decision (user): 2026's earlier official rulesets, Autumn 25 (4ccEditor `7e01541`) and Spring 26
(`c92d535`), become data-only test rulesets, each checked against `aatf.cpp` at its own commit on
the cup saves played under it. A ruleset usually serves two cups (maintainer): Autumn 25 served
Autumn 25 and Winter 26, Spring 26 served Spring and Summer 26.
Lead's reading: both fit the schema unchanged; they are the Autumn 26 file without bronze and
without its three new specials, with other numbers. The saves were matched to rulesets by file
date, not opened.
Why: Autumn 26 parity on saves built for older rules finds nearly every team illegal; under each
team's own ruleset most are legal, which tests the zero-finding path on real teams.
Plan: `aatf_rules.md` "Background", "Autumn 25 and Spring 26", crate layout, phase and
verification; `core/development_plan.md` Phase 5.

## 2026-10-07 — aatf — VGL24 and VGL25 checked; optional effects as one-option choices; `choices` suggestions
Decision (user): VGL25's optional manlet bonus (a manlet may carry the base rating or the base
plus the bonus) is a one-option `choice`, not a new `optional` field: a `stat_bonus` option is
used when, without it, the player's rating is above the expected rating. A choice the player
takes no option of is a suggestion under a new `choices` key (`UnusedChoice`). VGL25
(4ccEditor-VGL `vgl25`, `2c2d26a`) becomes a test ruleset with hand-built fixtures; VGL24
(`d4d7b15`) adds nothing to test. VGL25's injury resistance is exact, 2 for medals and 1 for
non-medals, as the VGL rules state, though `aatf_single_vgl` lets a gold sit below 2.
Lead's reading: the rest of VGL24 and VGL25 fits the schema (mixed `up_to`/`exactly` brackets,
forbids, free cards, captain and goalkeeper requirements off); the missing skill-card cap is
`max_skill_cards` 41.
Why: reusing `choice` keeps one mechanism for conditional effects, and an untaken option is an
unused allowance like the other suggestions.
Plan: `aatf_rules.md` "Background", the schema (`suggestions`, `choice`), "VGL25", violations,
suggestions, crate layout, phase and verification; `ruleset_editor.md` sections;
`core/development_plan.md` Phase 5.

## 2026-10-07 — aatf — VGL rulesets follow their rules pages; `fielded_at_a`; `forbidden_shapes`
Decision (user): the VGL26 and VGL27 rulesets follow the VGL wiki's rules pages (Chapter III,
supplied by the maintainer) where these are stricter than the checkers, each difference recorded
for parity. VGL27's positional rule is the page's ("the positions of each player on a team's first
preset … match one of the player's A positions"): universal `fielded_at_a` (presets), replacing
the `registered_position_fielded` switch of this day's earlier entry, which copied the checker's
looser test. VGL26's ban on 3-4-3-shaped formations is schema: `team_settings.forbidden_shapes`.
Lead's reading: from the pages, both rulesets also get exact injury resistance, no man marking,
support settings off but preset switching on (auto attack/defence levels left free, since the
next rule allows auto-mentalities at fixed values), and no skill-card cap; VGL27 loses the free
Heading its checker grants, VGL26 keeps it as a free card. `man_marking` covers PES 17+'s
man-marking instruction, whose ids are established before implementation. VGL27's differences
are added to the upstream bug report.
Why: the pages are the rules the organizers enforce; a checker's leniency is a defect, not a rule.
Plan: `aatf_rules.md` "Background", the schema (`Universal`, `TeamSettings`, load-time rules),
"VGL26", "VGL27", violations, "The host", verification; `ruleset_editor.md` sections and
"Randomize".

## 2026-10-07 — team_compiler — a `Full` export gets an empty `p1/` and `g1/` where it has none
Decision (maintainer): every team needs one player kit and one goalkeeper kit. A `Full` export
holding no player kit folder is compiled as if it held an empty `p1/`, and one holding no
`g1/` as if it held an empty `g1/`: each a placeholder kit (`kit_placeholder`;
`kit_colors_missing` without a `colors.txt`). A `Midcup` export gets neither: it adds to kits
already installed. The plan's rule, read literally, would rebuild a `Full` team's
`UniColor.bin` record from its kits alone, a count of 0 for an export with none.
Why: no real file holds a record counting 0, or a team without both kinds of kit, so their
effect in game is unknown, while keeping the old record would leave a past cup's kits on
offer, which a `Full` export exists to prevent. The placeholder kit is the compiler's existing
answer to a kit with nothing in it, and it shows in game as unfinished, so the gap is noticed.
Lead's reading: "one kit of each type" applies to any `Full` export lacking a type, not only
to one with no `Kits/` folder.
Plan: `team_compiler/pipeline.md` "Bins accumulation"; `team_compiler/README.md` TC-BIN-19
and TC-BIN-20 (new).

## 2026-10-07 — team_compiler — the working-bin walk's edges, and the measured DpFileList layout
Decision: the walk of the installed `DpFileList.bin` (step 4.21) builds every bin on the bundled
base when the list does not name the run's CPK, as the texture lookup already does; passes over
a listed CPK with no file; and stops the run before any export is read when the list, a CPK or a
bin in it cannot be read, with the new Fatal `installed_bin_unreadable`. The reader takes each
48-byte record's name up to its first NUL and ignores the rest of the record and the header's
first word, since PES 19's list carries an order number in its records and PES 20's a 100 in
its header; a list's order is its records' positions. Until 4.24 adds `pes_folder_not_found`,
a run with no PES folder reports only `bin_source` `bundled`. `dpfilelist_missing` is a
Warning with `--no-deploy` as in the loose-file modes (lead's reading: the 2026-10-05 answer's
reason, a run that deploys nothing, covers it, and "Post-processing" calls a `--no-deploy` run
clean, no error).
Why: an unlisted CPK has nothing known to come before it, so any CPK of the list could sit
above it. Falling back past an unreadable CPK would build the run's bins on an older copy, and
the run's CPK, loaded above it, would hide the cup's later kits and colors for every team the
run does not compile; aborting keeps the previous CPK. Positions, not PES 19's number, give the
order because every other list measured holds zeros there and PES 19's number agrees with them.
Plan: `team_compiler/pipeline.md` "Bins accumulation", "DpFileList upgrade" (the measured
layout); `team_compiler/messages.md` `installed_bin_unreadable`; `team_compiler/README.md`
TC-BIN-21 (new).

## 2026-10-07 — team_compiler — `templates/` overrides: flat names, read first, unreadable aborts
Decision: a `templates/` override is named as the embedded resource's own file is, in a flat
folder (`UniColor.bin`, `UniformParameter18.bin`, `UniformParameter19.bin`, `face_diff.bin`,
`fcl_hair_sim.fclo`, `body.skl`, `placeholder_kit.dds`); a file naming no resource is not read.
A compile reads the overrides once, before any export is read and before the working-bin walk.
An override that cannot be read is `template_override_unreadable`, now Fatal for every
resource: the run stops there. The plan had three dispositions (`DropFolder` for a template a
folder injects, `DropExport` for a referee template, `AbortRun` for a bin). A bin the walk
supplies keeps `bin_source` naming its CPK; an overridden bundled base still reports `bundled`,
beside `template_override_active`.
Why: TC-BIN-07 names the flat form, and flat names are unique among the resources embedded
today (one `body.skl`, PES 21's; a per-version skeleton embedded later needs a name of its own
in the folder). An override
is put there on purpose, as an `overrides/` file is, and an unreadable one of those fails the
CPK; dropping each folder that would inject a locked `face_diff.bin` would ship a CPK with no
faces, which a member notices later than a run that stops and names the file. One rule also
needs no per-task plumbing of a failed resource.
Plan: `team_compiler/pipeline.md` "Resolved decisions" ("Templates and fallback bins");
`team_compiler/messages.md` `template_override_unreadable`, `template_override_active`.

## 2026-10-07 — team_compiler — FPC patching takes the slots from `UniColor.bin`; bins parse as they are read
Decision: with a team's kit-FPC status On, the absent kit slots patched in `UniformParameter.bin`
are the kits the team's working `UniColor.bin` record holds that the export has no kit folder
for; a placeholder record holds none, a kit number no slot names is left alone, and a kit whose
task failed is not absent. A slot whose entry is missing or does not decode as a kit config
reports `kit_config_fpc_unpatched`; both FPC findings are the team export's, naming the slot.
A `Full` export's team configs are the entries named for its team ID; those its kits (failed or
not) do not name are removed. `UniformParameter.bin` is written whenever the run changes it. The
three bins are parsed as they are read, an installed one by the walk (`installed_bin_unreadable`),
a `templates/` one with the overrides (`template_override_unreadable`).
Why: the bundled base holds all ten configs for every team, so "every slot" would warn about
slots a team never had, and the configs alone could never show a missing one (TC-BIN-06's
`p3` needs a list from elsewhere); the record is what makes the game offer a kit. A
non-decoding entry is no config to patch, and the warning says the team needs a kit export,
which is the remedy. Parsing at read time moves a corrupt bin's failure from the end of the run
(`cpk_write_failed`, after every export was processed) to before any export is read, with the
file named; it settles the issue logged at 4.21a. Pre-Fox loose configs stay with 4.14 (TC-BIN-18).
Plan: `team_compiler/pipeline.md` "Bins accumulation" (the slots, the `Full` removal, the
parse at read time), "Resolved decisions" ("Templates and fallback bins");
`team_compiler/messages.md` `kit_config_fpc_adjusted`, `kit_config_fpc_unpatched`,
`installed_bin_unreadable`, `template_override_unreadable`; `team_compiler/README.md` TC-BIN-06
(its `UniColor.bin` record); `aesthetics_export/fpc_toggle.md` "Kit slots absent from the export".

## 2026-10-07 — team_compiler — a player table no installed CPK holds is not written
Decision: `BootsList.bin`, `GloveList.bin` and `PlayerAppearance.bin` (Fox) have no bundled
base. One the working-bin walk does not find is not written; when the run has rows for it, the
new Warning `player_table_missing` names the table and counts the rows left out. One it finds is
written on every run that writes a CPK and reported by `bin_source`; one it does not find gets no
`bin_source`. The tables are parsed as they are taken (whole 8-byte pairs, whole 60-byte rows).
Why: the game reads the highest-priority copy of each table whole, so a table of the run's rows
alone would take every other player's row away (Test 2, 2026-10-06: a player with no row wears
plain black boots); leaving the rows out costs only the compiled players' new boots or gloves,
and the warning says so. Bundling a seed table is not an option: the seed is the cup's own,
made by strip-and-seed. A `bin_source` per missing table on every from-scratch compile would
name nothing.
Plan: `team_compiler/pipeline.md` "Bins accumulation" (the paragraph on the walk's edges);
`team_compiler/messages.md` `player_table_missing` (new); `team_compiler/README.md` TC-BIN-22
(new).

## 2026-10-07 — team_compiler — what "a texture a model names must exist" compares, and where it runs
Decision: a path names the team's Common output when its directory, with `000` made the team's
ID, is the team's Common texture directory; an installed CPK holds it when its table of
contents lists the path the Common textures task writes for the stem. Both compare folded. Only
textures a mesh's material instance names are looked up; a `dummy_` stem never is. The check
runs in the model package's task, so `check` does not report `fmdl_texture_not_found`; its
Error leaves that package out and names the first missing texture, and its Warning is reported
once per missing texture. The working-bin walk now opens every CPK listed before the run's, not
stopping once each bin is found. A texture link satisfied by an installed CPK resolves to that
texture, and `check` reads the same tables of contents for it; when the lookup cannot be made,
`common_link_missing` stays an Error. The step lands as 4.29a (the model check) and 4.29b (the
link).
Why: stems fold everywhere else, and the CPKs this run compiles are written with the stem as
spelled in `Common/`, so an exact match would fail a model spelling it differently while the
same model passes against the export's own `Common/`. A texture no mesh uses is never loaded,
so it cannot make the player look wrong. Checking in the task, not at planning, avoids reading
each model a second time on the main thread, and the task is where the paths are pointed. A
failed package being left out alone is the rule of 2026-10-04, and a failed task reports one
finding. The walk needs every table of contents for the lookup, so it cannot stop early.
Plan: `team_compiler/pipeline.md` "Bins accumulation" (the walk opens every listed CPK),
"Resolved decisions" ("A texture a model names must exist"); `team_compiler/messages.md`
`fmdl_texture_not_found`.

## 2026-10-07 — aesthetics_export — `ValidationContext` carries the installed Common texture stems
Decision: `ValidationContext` gains `installed_common_textures`, the folded stems of the textures
the installed CPKs hold in the export's team's Common output; a texture `.common` link (the lib's
own `classify` says texture) whose target is not in `Common/` and whose stem is in the set is no
`common_link_missing` and stays in the validated player's files. The Team compiler reads the
team's ID from the parsed export's team name before validation to build the set, empty on
pre-Fox targets (4.15 adds their Common texture path) and for an export with no team ID.
`ValidationContext` is no longer `Copy`.
Why: the lib raises `common_link_missing` and takes the unresolved link off the player's files,
and it has no team ID or install to look in; clearing the finding afterwards in the tool would
need the dropped folder back, which the validated export no longer holds. A set of stems keeps
the lib free of CPK paths and of the walk.
Plan: `aesthetics_export/object_model.md` "Validation semantics" (what reaches the crate
through `ValidationContext`) and "Structure pass types" (its code block).

## 2026-10-07 — team_compiler — the sideload tree is staged, and 4.23 lands in two slices
Decision: a sideload run writes its loose tree under the run's staging folder and replaces the
contents of `{pes_folder_path}/livecpk/` with it only once the tree is written whole; a run that
fails, aborts or writes nothing leaves `livecpk/` as it was. A sideload run counts as not
deploying (`dpfilelist_missing` is a Warning), and `--mode sideload` with a `pes_folder_path`
that is not a folder is an invalid configuration (exit code 2). Step 4.23 lands as 4.23a (the
output sink and sideload mode) and 4.23b (test mode and the materialize seam); TC-OUT-09's
referee half lands with 4.19, which compiles referee exports.
Why: the plan stages every CPK so a failed run leaves nothing half-written; clearing
`livecpk/` first and writing into it would leave a modeler with a half tree the game serves
mid-match. Sideload never touches `download/`, so an Error for a list it does not need would
fail every sideload run on a machine without one. With no game folder the tree has no place
to go, and a run that silently wrote nowhere would look like a success.
Plan: `team_compiler/pipeline.md` "5. Writer" step 5 (sideload paragraph); `team_compiler/settings.md`
"CLI" (the sideload refusals).

## 2026-10-07 — team_compiler — where a test-mode entry goes
Decision: in test mode each task entry is written at `test_output/<source>/<folder>/<name>`:
the export's source as discovered (folder name, or archive file name with its extension), the
export folder the task works on (for a portrait or the logo, the folder of its file), and the
entry's file name at its game path; a model package's files are emitted once by their names in
the package instead of an `.fpk` and `.fpkd` per ID. The bins go under `test_output/_bins/` at
their game paths, overrides are not applied, and `test_output/` is replaced whole once written,
as `livecpk/` is.
Why: the plan says "export-relative paths" and Blue wrote `{export}/{itemfolder}/{model}/`,
but the texture tasks keep no source path once converted, and several outputs (merged parts,
template files, kit-variant fills, three logo sizes) have none. The task's folder and the
output name are known for every entry, need no new bookkeeping, and still show what the
compiler did to that folder: TC-OUT-07's `shirt.ftex` beside `fcl_hair.fmdl` is exactly this
rule. The source with its extension keeps a folder and an archive of one name apart. Replacing
the folder whole keeps a removed export's old output from looking current.
Plan: `team_compiler/pipeline.md` "5. Writer" step 5 (the materialize paragraph).

## 2026-10-07 — team_compiler — the parity test reads the CPK, not a loose tree
Decision: the parity test keeps comparing Red's reference tree with the entries of the normal
run's CPK; the loose-folder sink serves test and sideload modes and the tests of those modes,
not the parity test.
Why: the plan said the loose sink is the harness the parity test drives, to diff trees with no
CPK parsing in the middle. But the CPK is what members install, and reading it back is the one
test that checks the CPK writer against Red's output end to end; a loose tree would leave the
writer's half of every normal run unchecked by parity. The test-mode and sideload tests already
diff loose trees against the normal CPK's entries.
Plan: `team_compiler/pipeline.md` "5. Writer" step 5 (the paragraph on the loose-folder sink).

## 2026-10-07 — team_compiler — deployment: no marker file, failures named by step, a lock per staging folder
Decision: a deployment writes nothing in `download/` but the CPK (the plan's "marker file
lists what was deployed" is dropped). A failed copy to `{name}.cpk.partial` is
`deploy_target_unwritable`, a failed rename over the old CPK `old_cpk_locked`, whatever the OS
error. The CLI's probe before any export is read reports only `deploy_target_unwritable`, after
checking the PES folder, the exe, the list and the run's name in it, in that order. Each run
locks `.staging/{run_id}.lock` for its lifetime, and a run's start removes the staging folders
whose lock it can take or that have none. Until 4.25, `cpk_name_unlisted` is judged on the
installed list alone. The degradation findings name the path the CPK is promoted to.
Why: no plan text, scenario or tool reads a marker, Red wrote none, and an unknown file in the
game's download folder has an in-game effect nobody has tested. Windows reports a rename over a
file held open without delete sharing as access denied, the error of a folder needing
elevation, so the error code cannot tell the two apart while the step can (the copy needs only
the folder, the rename the old file). A locked old CPK at the probe is not reported because PES
may close before the run ends. A lock is released by the OS however the process dies, where a
process id can be recycled and an age threshold guesses; `File::try_lock` is std. The bundled
official list arrives with 4.25; a name it holds is then `dpfilelist_outdated`, with the same
consequence, so the interim reading degrades the same runs.
Plan: `team_compiler/pipeline.md` "6. Post-processing" (Staging, Deploy CPKs, Destination
writability preflight); `team_compiler/messages.md` (the five rows' contexts); TC-DEP-01.

## 2026-10-07 — team_compiler — the placeholder CPK is the shipped file, embedded
Decision: the empty placeholder CPK that `upgrade-dpfl` (and later multi-CPK mode) writes in an
unused slot is the official DLC's own 6,272-byte file, embedded from
`resources/templates/placeholder.cpk`, not written by the `cpk` crate. The official
`DpFileList.bin` is generated from `DpFileList.txt` in the layout of the maintainer's 53-entry
in-game test (header word 0, the PES 2021 list's 1204-byte zero tail) by
`scripts/provenance/fixtures/dpfl_template.py`.
Why: the shipped placeholder is CRI Packed File Maker 1.36's output (24 header columns, an
ETOC), not our writer's layout; making the writer reproduce it byte for byte would be a parity
project for one 6 KB file, while a copy is identical by construction. The list's tail is the
one the game was seen to load; the upgrade copies the file byte for byte, so its layout is
chosen once, here.
Plan: `team_compiler/pipeline.md` "Multi-CPK mode" ("Every slot is always written");
`resources/templates/README.md`.

## 2026-10-07 — team_compiler — the official-list check: only a compile that deploys, and its findings' context
Decision (lead, reversible): the installed `DpFileList.bin` is compared with the official one
only in a compile that deploys, in its preflight, after the run's CPK is found listed:
`--no-deploy`, sideload and test mode compare nothing. A list lacking the run's CPK is
`dpfilelist_outdated` when the official list names it, else `cpk_name_unlisted`; both end the
checks. A list naming it gets `dpfilelist_not_official` and then `dpfilelist_cpk_missing`,
Warnings. Their contexts: `path`, `missing` (official order), `unofficial` (installed order),
`order=differs`, the three only when they apply, lists joined by `, `; `files` for the missing
CPKs; and on all three a `command` naming `4cc-studio team-compiler upgrade-dpfl`, so the
CLI's line names the fix. The official list is a `templates/` resource, `DpFileList.bin`; an
override that does not read as a list stops the run like an unparsable bin override.
Why: the plan's "every compile with a PES folder, at deployment preflight" was ambiguous for a
compile that installs nothing; such a run puts no CPK under the list, so a warning about the
list would describe an install the run does not touch. A context key carries the subcommand
because the CLI renders a finding as its code and context alone; the GUI shows its button
instead.
Plan: `team_compiler/pipeline.md` "6. Post-processing" (DpFileList upgrade, Destination
writability preflight); `team_compiler/messages.md` (the three rows).

## 2026-10-07 — team_compiler — what `upgrade-dpfl` reports, and the cases the plan left open
Decision (lead, reversible): `upgrade-dpfl` reports through findings, as `check` and `compile`
do: `dpfilelist_cpk_renamed`, `dpfilelist_placeholder_written`, `dpfilelist_replaced` (Info),
`dpfilelist_cpk_dropped` (Warning, with the file's size when there is one), and
`dpfilelist_upgrade_planned` without `--yes` or `dpfilelist_up_to_date` alone. With `--yes`
it acts in that order, the list last, each line after its action; a file it cannot read or
write ends it with exit code 3, naming the file. The old list's `.bak` replaces an older one;
a list that does not read as a list is replaced and backed up with nothing renamed; no list
gets the official one; a list already official byte for byte is not rewritten. A rename whose
target name is held when its turn comes is not made, and the file is dropped if its name is
not official.
Why: the command line shows a tool's findings and nothing else, and findings are what the
Phase 8 dialog will list, so one set of codes serves both. Replacing the list last means a
failed rename leaves the install loading what it loaded before, and a second run finishes
the work. The last list a user had is the one a backup is for. Treating every held target
alike, rather than only those outside the stem, gives one rule that never overwrites a file.
Plan: `team_compiler/pipeline.md` "DpFileList upgrade"; `team_compiler/settings.md` "CLI";
`team_compiler/messages.md` (the six rows).

## 2026-10-07 — team_compiler — multi-CPK mode: the official list's slots, permits given back, all-or-nothing install
Decision (lead, reversible): (1) the teams part slots are the official list's entries (the
embedded `DpFileList.bin` or its `templates/` override), not the installed list's; a deploying
run needs the installed list to name every CPK it writes, each judged as the single CPK is
(`dpfilelist_outdated`, `cpk_name_unlisted`). (2) A batch held until its team is complete gives
its memory permit back when held. (3) `multicpk_mode` affects a normal compile only. (4) The
working-bin walk and the texture lookup start below the run's earliest CPK in list order (the
bins CPK). (5) The overrides go into the bins CPK; an entry at an override's path is left out
whichever CPK it would go to. (6) Several CPKs install all or none: copy all to `.partial`,
move each old CPK aside to `.cpk.old` (a locked one fails here and everything is undone and
promoted), rename the `.partial`s in, remove the `.old`s. (7) `dds_compression` leaves step
4.26 for the pre-Fox steps: it is pre-Fox only and nothing there compresses yet. (8) Contexts
of `cpk_slots_exhausted`, `cpk_team_exceeds_cap`, `cpk_size_over_limit` fixed in the catalog.
Why: (1) the DLC is built with `--no-deploy` on machines whose install is not the DLC's, so
the installed list cannot define its capacity, while the official list is what users receive.
(2) the plan's "charged like any pending batch" deadlocks once a team's charges exceed the
budget: its next task waits for bytes its own held batches keep. (3) loose files have no
parts. (4) every part the run writes replaces the installed one of its name, so none of them
may feed the run. (5) the bins CPK loads before the parts, so an override placed there would
lose to a part's entry unless the entry is left out. (6) TC-DEP-11 asks that none replaces an
installed one when one is locked; renaming each `.partial` over its old CPK cannot be undone
once a later rename fails. (7) keeps the step to what it can test.
Plan: `team_compiler/pipeline.md` "5. Writer" step 6 (Multi-CPK mode), "6. Post-processing"
(Deploy CPKs); `team_compiler/messages.md` (the three rows).

## 2026-10-07 — team_compiler — `cpk_part_max_size` is a byte count, 3 GiB; `--no-deploy` names each CPK
Decision (lead, reversible): (1) `cpk_part_max_size` is written in the settings file as a whole
number of bytes, default `3221225472` (3 GiB), not as text like `3 GB`. (2) In multi-CPK mode
`deploy_skipped_by_flag` is reported once per CPK promoted, each naming its path: the bins CPK
first, then the parts by slot number.
Why: (1) a count needs no unit parser and leaves no doubt between GB (10^9) and GiB (2^30);
the plan's "3 GB" was either, both under Git for Windows' 4 GiB ceiling, and 3 GiB shows as
`3 GiB` in the findings, which use binary units. (2) the finding's context is one path, and
deployment, which 4.26c brings to every generated CPK, reports per CPK too.
Plan: `team_compiler/settings.md` (the settings table); `team_compiler/messages.md`
(`deploy_skipped_by_flag`); `team_compiler/pipeline.md` "Multi-CPK mode: teams parts".

## 2026-10-07 — team_compiler — several CPKs: one preflight finding each, and a failed rename into place undone
Decision (lead, reversible): (1) when a deploying run writes several CPKs, `dpfilelist_outdated`
and `cpk_name_unlisted` are one finding each, `cpk` naming every CPK concerned, joined by `, `
in the run's order (the bins CPK, then the parts); `output` is then the output folder. (2) In
the all-or-none install, a `.partial` that cannot be renamed into place undoes everything, the
new CPKs already in place removed before their `.old` files go back, and is `old_cpk_locked`
naming that CPK.
Why: (1) an old list lacks the five teams slots together, and five findings saying one thing
bury the rest of the run's lines; `dpfilelist_not_official` already joins its names. (2) the
plan's move-aside order leaves this step only a name nothing should hold, but if it fails the
run must still leave `download/` as it found it, which is what "all or none" promises.
Plan: `team_compiler/pipeline.md` "6. Post-processing" (Deploy CPKs); `team_compiler/messages.md`
(`dpfilelist_outdated`, `cpk_name_unlisted`, `old_cpk_locked`).

## 2026-10-07 — team_compiler — a teams stem that is the bins CPK's own is refused
Decision (lead, reversible): `compile` in multi-CPK mode refuses, with exit code 2, a
`teams_cpk_name` equal to the stem of `bins_cpk_name` (`bins` with `4cc_08_bins`).
Why: the slots are every official entry of the teams stem, so the bins CPK would also be a
teams slot, and the bins writer and the first part would write the same staged file without
an error. Refusing the setting is one comparison; any other answer would need a rule the plan
does not have (which of the two the file is).
Plan: `team_compiler/settings.md` (the `teams_cpk_name` row).

## 2026-10-07 — team_compiler — the refs CPK among the run's CPKs, and the referee tree's names
Decision (lead, reversible): (1) the refs CPK is one of the run's CPKs when the run holds a
refs export (known from its name before any file is read), after the others, in single and
multi-CPK mode; the preflight judges it with them; it is written, with the referee template
tree, only when the refs export commits something. (2) It is not the walk's boundary: the walk
passes over the installed CPK of its name. (3) The referee template trees are embedded as
`resources/templates/referees_fox/` (and later `referees_prefox/`) with each file at its game
path, and an override is `templates/referees_fox/<game path>`, replacing that file; the flat
override names of the other resources cannot hold two trees whose file names coincide.
(4) Step 4.19 covers Fox referees; pre-Fox referees (TC-REF-09) wait on step 4.14 and the
pre-Fox gate, and the marker stays with step 4.27.
Why: (1) a run without a refs export must not be refused because an old list lacks
`4cc_18_referees`, and a refs export dropped by validation must not replace the installed
referees with an empty CPK. (2) `4cc_18_referees` loads before the stadiums, parts and midcups,
so as the boundary it would take them all out of the bins walk. (3) one rule for both trees, and
the provenance is a folder copy. (4) pre-Fox compiling is still gated and its face paths need
4.14's pre-Fox export.
Plan: `team_compiler/pipeline.md` "5. Writer" step 5, "Multi-CPK mode" (the boundary bullet),
"Resolved decisions" (embedded templates); `resources/templates/README.md`.

## 2026-10-07 — team_compiler — a refs export's kits, logo and portraits are named by the gate; its note goes under `/refs/`
Decision: on Fox a refs export compiles its mapped player folders, shared folders and Common
textures (team 999 in game paths), and the subset gate names, in place of `refs`, its first kit
by slot, then its logo, then its first portrait. It plans no colors record, no
`team_colors_missing`, no kits and no `BootsList.bin`/`GloveList.bin` rows. Its `notes.txt` goes
into `teamnotes.txt` under the header `/refs/`.
Why: a referee has no kit slot, team logo or player id, so compiling these would write files the
game never loads for a referee, and dropping them silently would hide a member's mistake; the
gate's refusal is the existing way of saying "not compiled". Rows are not written because the
game's referee hook loads slot NN's `k99NN`/`g99NN` by number, and Red writes none. The note is
kept because a cup admin reads `teamnotes.txt` for every compiled export.
Plan: `team_compiler/README.md` (the Phase 3 gate paragraph), `team_compiler/pipeline.md`
"Notes collection".

## 2026-10-07 — team_compiler — the refs CPK beside the team side: what each writes, the walk, the name
Decision: (1) the refs export's entries go only into the refs CPK; the team side (the single
CPK, or the bins CPK and the teams parts) is written only when a team export commits something
or there are overrides, so a run whose only committing export is the refs export writes the
refs CPK alone, with no bins. (2) An entry of the refs export at an override's path is left out,
as any other. (3) The walk passes over the installed refs CPK only when the run holds a refs
export, and a refs export's `.common` texture link is then never satisfied by an installed
texture (the 4.19a issue closes so). (4) `refs_cpk_name` equal to `cpk_name`, or in multi-CPK
mode to `bins_cpk_name` or a teams slot, letter case aside, is refused (exit code 2).
Why: (1) the referees change no bin, and a bins-only team CPK would replace the installed team
CPK with an empty one, which a refs-only compile must not do. (2) one override rule run-wide,
as multi-CPK mode already has. (3) the run replaces that CPK, so its old content must not satisfy
anything; a run without a refs export keeps it, and it holds nothing a team needs. (4) two
writers on one file, the same reason as the teams stem refusal.
Plan: `team_compiler/pipeline.md` "5. Writer" step 5; `team_compiler/settings.md`
`refs_cpk_name`.

## 2026-10-07 — team_compiler — the referee template tree: embedded per file, written last, not in test mode
Decision: the Fox referee tree (`resources/templates/referees_fox/`, 31 files) is embedded with
one `include_bytes!` per file, listed by game path, not with a directory-embedding crate. It is
written into the refs CPK after the refs export's entries, only when that export commits
something; in sideload mode it lands in `livecpk/` the same way; test mode writes none. A tree
entry at an `overrides/` path is left out like any other entry.
Why: `include_dir!` would be a new dependency for one folder, and a listed tree also fails the
build when a file goes missing. Test mode shows what the compiler did to the exports, and the tree
is no export's, as the overrides are not applied there either.
Plan: `team_compiler/pipeline.md` "Resolved decisions" (templates embedded), "Output-mode artifact
routing".

## 2026-10-07 — team_compiler — the Fox referee marker: which model, its texture, a failed marker, replaced configs
Decision: (1) the Fox marker is the 4cc refs compiler's `referee_prop.fmdl` (SHA-256
`0f438d5d…`), bundled as `resources/templates/referee_marker.fmdl` unchanged; at compile time its
base texture, `common/000/sourceimages/cup_logo.dds`, is pointed at
`common/999/sourceimages/ref_marker.dds`, and its two dummy maps are kept. (2) A marker that its
texture checks drop or whose conversion fails counts as absent: no collar model, configs
unchanged, the texture code reported. (3) A referee kit config replaced from the data
directory's `templates/referees_fox/` gets collar 77 too, and only the collar and winter collar
bytes of a config change.
Why: (1) all three copies on the maintainer's machine share the geometry and the static
painting (each vertex weighted to `static` only, checked with the `fmdl` crate); keeping the
file as shipped makes its provenance a hash, and the one path the compiler changes is the one
the plan names. (2) a collar naming a texture the CPK lacks would draw untextured under every
referee. (3) the marker is the export's choice, and a cup maintainer who does not want it
leaves `ref_marker.dds` out.
Plan: `team_compiler/blue_port.md` "Referee export processing"; `resources/templates/README.md`
`referee_marker.fmdl`.

## 2026-10-07 — team_compiler — collars: the 9xx IDs, one collar per export, a midcup's absent slots
Decision: (1) a collar file named for a stock 9xx collar (901 and up) is `collar_id_invalid`;
the suite's reserved IDs (105, 77) are checked before the stock set, so `collar_105` is
`collar_id_conflict` on every version, PES 15 included. (2) An export holds one collar: a second
valid `collar_<ID>` file of the same export (path order; `collar_12` beside `collar_012`
included) is `collar_id_conflict`, the export itself named as the claimant. (3) A `Midcup`
export's collar also goes into the team's kit slots it does not resend: their entries in the
working `UniformParameter.bin` get the collar after the FPC patch, in place.
Why: (1) a kit config stores a collar in one byte, so a 9xx replacement could never be worn by
the team's players, which is the whole point of a team collar; the plan already calls a reserved
ID a conflict, not an invalid ID. (2) the configs take one ID; one general rule (the claimed
list) instead of a new code. (3) the plan says "all of the team's kit configs", and a midcup
adding a collar without resending its kits is the common case, as for FPC.
Plan: `team_compiler/messages.md` `collar_id_invalid`; `team_compiler/pipeline.md` "Collars".

## 2026-10-07 — team_compiler — a refs export's collar file is named by the gate
Decision: a refs export holding a `Collars/` file is skipped with `content_not_yet_compiled`
naming its first collar file, after its kits, logo and portraits; a team export's FMDL collar
compiles on Fox from step 4.9b2.
Why: a collar replaces a stock collar and is put on the replacing team's kits; the referees'
kits are the template tree's, which wear collar 77 or their own, so a refs collar has no kit to
go on, and a silent drop would hide a file the member meant to ship.
Plan: `team_compiler/README.md`, the gate paragraph ("Step 4.19 lifts ...").

## 2026-10-07 — team_compiler, pipeline — memory accounting: a running task charges without waiting
Decision: a task still acquires its source bytes before loading, the coordinator's one blocking
request. What it allocates while running (each decoded image at its RGBA size, read from the
header before the decode; a model's parsed and merged parts at their source size, an estimate;
its batch's entries at their length) goes through a new `MemoryBudget::charge`, which takes the
bytes at once and never waits, so `in_flight` may pass the cap by the running tasks' workspace
and later acquires wait for it to drain; `MemoryBudget::peak` is the measurement. The held team
of a multi-CPK run stays outside the budget, as the plan already said, and the conversion cache
is charged together with its eviction (step 4.y).
Why: a permit that grows by waiting, which the plan's open question proposed to evaluate, can
deadlock: a task holding bytes and waiting for more waits on itself once the others drain, or on
another grower, and every worker can be in that state at once ("Admission must remain
progress-safe"). Charging everything up front at planning would need every texture's header
carried from the deep pass, and would charge a task's decodes as if they were simultaneous
although they run one after another. A charge that never waits keeps the one-acquirer protocol,
which cannot stall, and still bounds the run: the overshoot is the workers' workspace, not the
number of teams. Charging the cache without eviction would hold bytes no completion releases.
Plan: `libs/pipeline.md` "Memory budget"; `team_compiler/pipeline.md` "Admission" (Charges while
a task runs), "Resolved decisions and open questions" (Complete memory accounting).

## 2026-10-07 — dds_convert, team_compiler — the conversion cache's retention bound is built with the GUI's compile
Decision: the cache's retention bound, its eviction under memory pressure and the charge of its
retained bytes to the run's budget (worklog step 4.y) are built in Phase 8, with the GUI's
compile, the first caller that keeps a `Converter` across runs; Phase 4 leaves the cache as it
is, made per run by the compiler and dropped with it.
Why: the bound exists so a session of repeated edits does not keep every superseded
conversion, and no such session exists before the GUI: a CLI run is one process, and its cache
holds at most two exports' conversions (the engagement limit), bounded whatever the number of
teams. An eviction policy built now would be shaped without the caller whose access pattern it
serves (which runs share a converter, when a run starts, what a run may evict), and charging a
per-run cache without eviction would hold bytes no task's completion releases.
Plan: `libs/dds_convert.md` "In-memory conversion cache", the bullet "Retention is separately
bounded and budgeted".

## 2026-10-07 — team_compiler — number atlases: a digit moves by a uniform scale into Konami's slots
Decision: a `_back`, `_chest` or `_leg` atlas in the other engine's arrangement is re-arranged
digit by digit between a PES 15-17 column's equal tenths and the fixed slots a PES 18-21 row
keeps its digits in (in a 2048-long atlas, 160 long from 0, 190, then 200d - 60, digit 1's
ending at 340), by a uniform scale of 1.25 (0.8 the other way), top edges together. Outside the
glyphs the result holds the source's top-left texel; its top level is re-arranged and its mip
chain generated by the conversion.
Why: the measurement (`scripts/provenance/kit_uv/number_atlas/`) found every stock PES 21 row
keeps its digits in those slots, not in tenths (digits 2 to 9 straddle the tenths' boundaries),
and the same team's glyph keeps its proportions between the games (median aspect change 1.00).
Re-arranged by the rule, the 465 PES 17 atlases with a PES 21 counterpart match it with a
median ink overlap of 0.68, where stretching each cell onto a tenth gives 0.16; a stretched
digit would also come out 28% wider than the game's own. The top-left texel is what every stock
atlas holds outside its glyphs (their color at alpha 0); a fixed color would darken the glyphs'
edges under filtering. Where each engine samples a digit is decided in code and was not found,
so the rule copies Konami's layout rather than a measured window: an atlas laid out like the
stock ones is safe whatever that window is.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps", the glyph atlas sentences.

## 2026-10-07 — team_compiler — hand auto-split: face models only, detected by the deep pass, split in both tasks
Decision: the hand auto-split applies to the FMDLs categorization makes face content (a folder's
own or a Common model a `.common` link brings in); a model named as boots or gloves, and every
model of a shared `Boots/` or `Gloves/` folder, is never split, and there is no weight threshold.
The deep pass records which FMDLs carry positive `skh_*_l`/`skh_*_r` weights; planning gives a
player folder holding such a face part a gloves task, with the model among its files; the face
task and the gloves task each split it (FMDL → IR without its skeleton → split → FMDL), keeping
the body and the hands respectively. The face task reports a new Info, `model_hand_split`. The
round trip's `model_convert` findings are not shown until cross-format conversion maps them.
Why: read literally, "every model with hand weights" splits an authored glove (all hand
vertices) into a glove and a forearm the body would receive as face content, and FNG's
`Boots/k2214 - Park` (9 stray vertices per hand) into boots and two tiny gloves; the plan's own
scope note says boots need no split. No face model on the maintainer's machine carries stray
hand weights, so a threshold would distinguish nothing. Detection must precede planning because a
gloves ID, the `GloveList.bin` row and the gloves task are planned from file names; the deep
pass already parses every model. Splitting in both tasks keeps tasks independent (the writer
commits each whole) at the cost of a second split of a model that is rare (one full-body model
found). Importing without the `.skl` keeps the FMDL's own bones, measured equal through the
round trip on Red's `00007 - BLANK` `fcl_hair.fmdl`. The Info tells a member why gloves appear
that no file named, as `fmdl_merged` does for a merge.
Plan: `model_conversion/hand_split.md` "Pipeline integration"; `aesthetics_export/player_folders.md`
"At compile time, the pipeline" step 0; `team_compiler/pipeline.md` "2. Per-export serial steps"
step 6 and "3. Per-model-folder parallel steps" step 3; `team_compiler/messages.md`
`model_hand_split`; TC-MOD-31.

## 2026-10-07 — team_compiler — the `.mtl` a pre-Fox `.model` uses, restated from Red's code
Decision: a `.model` uses the first `.mtl` a search finds: folder by folder, a name-matched `.mtl`
(stem starts or ends the model's), then `materials.mtl`, then any `.mtl`, the first in case-folded
name order within a kind; a `.mtl.common` link counts as its target's name. Folders: the file's
folder, then the model folder; for a `.model.common` link, the link's folder (name-matched only),
then `Common/`, then the link's folder, then the model folder. No `.mtl` found is
`model_material_undefined`.
Why: the plan's wording ("then the main folder") came from Red's and defined neither folder. Red's
`find_mtl_file` takes a `main_folder_path` (the face folder) and a `model_folder_path` (the folder
holding the model or link), so the Studio terms are the model folder and the file's folder. Red's
default kind accepts any stem `materials` starts or ends with (`mat.mtl` counts) and takes the
listing's order: an exact `materials.mtl` and a case-folded order give the same answer on every
real export and one a member can predict. Red names a missing `materials.mtl` when it finds
nothing, which the game cannot load; the existing Error says so instead.
Plan: `model_format.md` "Material files", the paragraph "Pre-Fox: the `.mtl` a `.model` uses".

## 2026-10-07 — team_compiler — a model's type is read without its kit token
Decision: a model's category and allowed name are read from its stem with the kit token
(`kit1`-`kit9`, `kitN`) and one delimiter next to it removed, on both engines.
Why: read from the whole stem, a per-kit `boots_kit1` names no suffix and is face content: on Fox
its meshes are merged into the face's `fcl_hair` and the boots package lacks them, and on pre-Fox
`face.xml` would type it as a face part. Stripping the token is the reading the kit-variant rule
already uses to find a set's reference.
Plan: `team_compiler/pipeline.md` "4. Per-export non-model steps", "Kit-dependent assets".

## 2026-10-07 — team_compiler — the generated pre-Fox `face.xml` and face CPK, in Red's shape
Decision: a generated `face.xml` entry's `path` names the packed `oral_<stem>_win32.model` with `*`
in the place of `win32` (`./oral_<stem>_*.model`, the blank folder's `./oral_dummy_*.model`); the
file is the shape Red writes (XML declaration in single quotes, `<config>`, three-space indent,
`<model … />`, the `<dif>` base64 on one line, CRLF); the nested face CPK's entries repeat the outer
path (`common/character0/model/character/face/real/{id}/<file>`).
Why: the plan's blank-folder entry spelled the path `./oral_dummy_win32.model`, but Red's output,
the community's hand-written XMLs and the installed PES 2015 DLC's face CPKs all name models with
`*` for the platform, and Red's output is the parity standard; a literal `win32` path is one no
cup has loaded. The nested layout was unwritten; Red packs the face folder under that path and the
installed DLC's `70202.cpk` holds its entries there.
Plan: `team_compiler/pipeline.md` "2. Per-export serial steps" step 4, "3. Per-model-folder
parallel steps" steps 4 and 7; TC-MOD-39.

## 2026-10-07 — team_compiler — generated `face.xml` entries: order, packed names, ratio, the dummy's finding
Decision: entries follow the models' export paths case-folded, the dummy last; a model packs as
its stem lowercased with `oral_` and `_win32` each added only when missing, its type read without
them; `ratio` is the `ratio_<n>` token up to the next `_`; `xml_face_neck_added` is not reported
on a blank face.
Why: the plan named the affixes but not their order, case or doubling. Red lowercases and skips
affixes already there, and old exports carry `oral_x_win32.model` names, so doubling them would
type `oral` content wrongly; path order makes recompiles identical. Red's `ratio` took the whole
rest of the stem, so TC-MOD-21's `visor_ratio_2_parts` would get `ratio="2_parts"`, a value the
game cannot read as a ratio. A blank face's dummy is the compiler's own placeholder on every
model-less player; an Info on each would be noise the member cannot act on.
Plan: `team_compiler/pipeline.md` "3. Per-model-folder parallel steps" step 4.

## 2026-10-07 — team_compiler — pre-Fox boots and gloves folders are `k0625`/`g0625`
Decision: a pre-Fox boots output is `common/character0/model/character/boots/k{id:04}/` and a gloves
output `…/glove/g{id:04}/`, as on Fox; TC-MOD-22, TC-MOD-35 and TC-MOD-41 now name `k0644`/`k0625`.
Why: the "Game paths reference" wrote `boots/{id}/` and the scenarios read it as a bare `0644`, but
the installed PES 2015 DLC (`4cc_40_faces.cpk`) holds `boots/k0444/boots.model` and
`glove/g0708/glove.xml`, and Red keeps the `k`/`g` folder name on both engines; a bare number is a
folder the game never looks in.
Plan: `team_compiler/pipeline.md` "Game paths reference"; `team_compiler/README.md` TC-MOD-22, 35, 41.

## 2026-10-07 — team_compiler — pre-Fox shared boots and gloves output, and a combined shared face
Decision: a pre-Fox shared boots folder's model is written as `boots.model` and the `.mtl` it uses
as `boots.mtl`; several boots models in one shared folder merge with `pes_model::ops::merge`
(slice 4.14e; refused as not compiled yet until then). A shared gloves folder's models and `.mtl`
files keep their own names lowercased, with no `oral_`/`_win32` affixes, and a generated `glove.xml`
in the generated `face.xml`'s shape without `<dif>` lists them, `path="./<name>.model"`, typed by
the model-name table. Textures sit beside the models as DDS, the `.mtl` paths naming them
`./<stem>.dds`. A face link combines on pre-Fox (`link_combined`, as on Fox): the shared folder's
files go into the player's face CPK and a local file replaces the shared file packing under the
same name. A pre-Fox target plans no BootsList/GloveList rows.
Why: every boots folder of the installed PES 2015 DLC (`4cc_40_faces.cpk`, 13 checked) holds
exactly `boots.model` and `boots.mtl`, the names the game loads, so `kit_boots.model` written under
its own name would never load; its `.mtl` files name textures `./medic_bsm.dds`. Its glove folders
hold `glove.xml` naming `./glove_l.model` with no affixes (Red's `glove_element` writes the same),
and one lists `./historically.model` typed `shirt`, so names and types are free there, unlike
boots. Copying a shared face with local files on top is the plan's pre-Fox rule; a duplicate name
failing the task instead would make the copy order matter. Pre-Fox has no player tables: the
"Game paths reference" writes none, and planning rows there would report `player_table_missing`.
Plan: `team_compiler/pipeline.md` "3. Per-model-folder parallel steps" step 7;
`aesthetics_export/player_folders.md` "A link plus local models combines".

## 2026-10-07 — team_compiler — the pre-Fox Common output holds `Common/`'s models, `.mtl` files and textures
Decision: on a pre-Fox target the team's Common output holds every `.model`, `.mtl` and texture
directly in `Common/`: models under their packed `oral_<stem>_win32.model` names, `.mtl` files
under their own names with paths to Common textures made `model/character/uniform/common/{team}/
<stem>.dds`, textures as DDS. A `.model.common` entry names
`model/character/uniform/common/{team}/oral_<stem>_*.model`; a `.mtl` found in Common is named at
the same directory. TC-MOD-24 now expects `oral_legs_win32.model` in the Common output, not
`legs.model`.
Why: Red copies the Common folder whole on pre-Fox, renaming its models with `model_names_fix`
and making its `.mtl` paths absolute (`export_move.py`, `fix_mtl_paths`), and its generated XML
names a Common model as `model/character/uniform/common/XXX/oral_<stem>_*.model`; packing the model
as `legs.model` would leave that path naming nothing. Packing only the linked files instead would
need planning to collect every link of every player for a saving no cup needs: a Common model is
in Common to be linked.
Plan: `team_compiler/pipeline.md` "3. Per-model-folder parallel steps" step 4;
`team_compiler/README.md` TC-MOD-24.

## 2026-10-07 — team_compiler — pre-Fox name and limit checks: who reports what (4.14d)
Decision: `xml_oral_prefix_missing` is a user `face.xml` check and moves to 4.15 with the second
half of TC-MOD-23, now TC-XML-09; TC-MOD-23 keeps the generated model's `uniform_sub`.
`model_material_undefined` for a material the paired `.mtl` lacks is reported by the deep pass
(so `check` reports it), with TC-XML-08 moving to 4.14d; it is pass-through-eligible when a
`.mtl` was found, not when none was. `texture_not_div4` covers every pre-Fox texture and is not
pass-through-eligible. `edithair_unsupported` is pre-Fox only, from the structure pass.
`model_name_invalid` is `fmdl_name_invalid`'s rule on a pre-Fox target.
Why: the generator always writes `oral_`, so the prefix rule can only fail on a hand-written
`face.xml`, which stays refused until 4.15; citing TC-MOD-23 for half its scenario would pass the
acceptance scanner on a claim no test proves. A check reported only by `compile` leaves `check`
silent on a folder `compile` drops, which is what the deep pass exists to prevent; a model whose
`.mtl` lacks a name still packs (the game renders its fallback), while one with no `.mtl` cannot
be named in `face.xml`. Every pre-Fox texture is written BC1 or BC3, and Direct3D 9 refuses a
block-compressed texture with a side that is not a multiple of 4, so keeping one under
`pass_through` would write a texture the game cannot create. A `face_edithair.xml` means nothing
on Fox. One naming rule under two codes keeps each engine's catalog in its own format's terms.
Plan: `team_compiler/README.md` TC-MOD-23, TC-XML-09; `team_compiler/messages.md` "Model checks",
"Textures", the `edithair_unsupported`, `model_name_invalid`, `texture_not_div4` and
`model_material_undefined` rows; `aesthetics_export/player_folders.md` "Model names".

## 2026-10-07 — team_compiler — Fox referee configs are written as `UniformParameter.bin` entries too
Decision: on Fox each referee kit config the refs CPK holds loose also replaces the entry of its
name in the bins CPK's `UniformParameter.bin` (step 4.27b). The Clef scan runs only at a phase's
close, over each crate, not on each step's diff.
Why: the maintainer's in-game tests show the game needs the loose configs but reads the values
from the entries, for referees as for team kits; the plan's "never entries" left the referees on
the carried-forward entries' collar 105, so the collar-77 marker could not show. Per-diff Clef
scans flagged seven windows over 32 reviewed code commits, all false, while the first
whole-crate pass found the one real bug (maintainer's call, 2026-10-07).
Plan: `team_compiler/blue_port.md` "Referee export processing"; `AGENTS.md` "Lead and sidekick";
`CONTRIBUTING.md` "Clef scan".

## 2026-10-07 — team_compiler — Pre-Fox merges report `model_merged` and `model_merge_flags_conflict`
Decision: a pre-Fox boots merge (a shared boots folder holding several boots models, or an
`ingame_face` player's boots parts with a combined link's model) is noted as `model_merged`,
the pre-Fox twin of `fmdl_merged`, in the order the Fox merge takes; parts whose `.model`
headers carry different `flags` leave the package out with `model_merge_flags_conflict` (E).
Step 4.14's slice (e) is cut into e1 (`ingame_face`'s boots and the several-boots shared
folder), e2 (`ingame_face`'s gloves and the hand split of a `.model`) and e3 (kit variant sets).
Why: `pes_model::ops::merge` returns `FlagsConflict` for a field of unknown meaning, set only in
two Konami face-montage models; it has no merging rule, and naming it `skl_merge_conflict` or
`merge_material_conflict` would send the member to the wrong fix. The twin code keeps each
engine's catalog in its own format's terms, as `model_name_invalid` does. Slicing keeps each
diff to one review.
Plan: `team_compiler/messages.md` the `model_merged`, `model_merge_flags_conflict` and
`skl_merge_conflict` rows.

## 2026-10-07 — team_compiler — Pre-Fox `ingame_face` gloves combine without a merge
Decision: on pre-Fox an `ingame_face` player's gloves go to one player-exclusive gloves folder
written as a shared gloves folder is (each model under its own name, one `glove.xml` entry
each); a combined gloves link's models join as more entries, a local model replacing a linked
one of the same output name. The plan's earlier text merged them per side like the boots.
Why: `glove.xml` lists any number of entries, so the game loads every part unmerged; a merge
would add material and skeleton conflicts for the member to fix for no gain, and could not
combine a `handL` part with a `gloveL` one, whose `face.xml` types differ. Boots still merge,
because the game loads one `boots.model`.
Plan: `aesthetics_export/player_folders.md` "`ingame_face` with shared links" and "Merging is
Fox-only".

## 2026-10-07 — scripts — `team_compiler`'s mutants run on the PC only
Decision: `scripts/mutants.py` keeps a `LOCAL_ONLY_CRATES` set, `team_compiler` its one member;
`just mutants <crate>` for one of them, and `just mutants-diff` over a diff holding any of their
mutants, run every mutant locally instead of splitting with the VPS (maintainer's call).
Why: the remote half's builds of `team_compiler` reached the 9 GiB cap with `REMOTE_BUILD_JOBS`
already at 1 (4.14d lost two builds there), the cap cannot grow, and a killed build hides an
untested mutant until it is rerun by hand, so the split no longer saves time on this crate.
Plan: `AGENTS.md` "Environment" (the VPS cap paragraph).

## 2026-10-08 — team_compiler — Pre-Fox hand split: `<stem>_glove_l`/`_glove_r` entries in the face
Decision: on pre-Fox a hand-weighted face model is split in the face task: the body keeps its
name and `face.xml` entry, and the hands become `<stem>_glove_l.model` and
`<stem>_glove_r.model`, two more entries typed `gloveL`/`gloveR`, all three naming the source
model's `.mtl`. Under `ingame_face` nothing is split, as on Fox, where only face content is.
Why: the plan was silent on pre-Fox names. A `face.xml` lists gloves beside the face, so no
gloves task is needed; the stem prefix keeps the split parts from clashing with an authored
`glove_l.model`, and the suffix types them by the existing table. Each split part keeps a
subset of the source's materials under their names, so the source `.mtl` already defines them
and writing new material files would only add names to keep apart.
Plan: `model_conversion/hand_split.md` "Pipeline integration".

## 2026-10-08 — team_compiler — A pre-Fox gloves output packs only the `.mtl` files its models use
Decision: a pre-Fox gloves output, a shared `Gloves/` folder's or an `ingame_face` player's
own, packs the `.mtl` files its models use (`mtl_for`), each once, a player's own replacing a
combined folder's of the same name; until 4.14e2 a shared gloves folder packed every `.mtl` it
held. Planning routes each pre-Fox model and `.mtl` by the source it comes from, a marked
player's own `.mtl` files being files of both his boots and his gloves tasks.
Why: a marked player's `.mtl` files sit in one folder for both his boots and his gloves, so
packing all of them would put his boots' material set in his gloves folder; one rule for both
gloves outputs keeps one code path, and an unused `.mtl` is nothing the game loads.
Plan: `team_compiler/pipeline.md` step 7 "Packing".

## 2026-10-08 — team_compiler — Pre-Fox hand split skips a Common-linked model; TC-MOD-43
Decision: on pre-Fox a hand-weighted model a `.model.common` link brings in is not split: the
face lists the Common output's file by reference and packs nothing of it. The split covers the
models the face packs (the folder's own and a combined shared face's). The pre-Fox split's
scenario is TC-MOD-43; `model_hand_split` is reported on both engines.
Why: splitting a linked model would mean copying it into the face, so the link would stop being
a reference, for a case no export on the maintainer's machine has (one hand-weighted face model
exists there, a Fox one). Logged as an open worklog issue rather than built.
Plan: `model_conversion/hand_split.md` "Pipeline integration"; `team_compiler/README.md`
TC-MOD-43; `team_compiler/messages.md` the `model_hand_split` row.

## 2026-10-08 — team_compiler — Pre-Fox hand split of a model whose `.mtl` is a Common file is refused
Decision: on pre-Fox a hand-weighted face model whose `.mtl` is a Common file (a `.mtl.common`
link, or a `Common/` `.mtl` its search finds) fails its folder with `model_conversion_failed`
(`error=its .mtl, <path>, is a Common file, which the face does not read`), not split by reading
that Common file.
Why: the split needs the `.mtl` bytes and the face task reads no Common file (the Common output
is listed by reference). Reading one only for a split would be an exception to that rule for a
case no export on the maintainer's machine has; a refusal names the model and the fix (a local
`.mtl`), and the exception can be built when an export needs it (sidekick's finding, 4.14e3).
Plan: `model_conversion/hand_split.md` "Pipeline integration".

## 2026-10-08 — team_compiler — Pre-Fox referee marker is `referee_collar_077.model` beside an empty `collar_077.model`
Decision: on PES 15-17 the refs CPK carries the marker model as `referee_collar_077.model` with
its `.mtl`, and an empty `collar_077.model` beside it; the 77 reservation against teams stays on
both engines. TC-REF-04 names both files.
Why: in-game on PES 17 (2026-10-08, the lead's harness, worklog Issues "referee collars") the
referee draws `referee_collar_<ID>` and does not appear at all when `collar_<ID>` is missing,
so one file alone either shows nothing or breaks the referee; the cup's FPC collar 105 ships the
same pair. A team's `collar_077` would be the file the referee finds, so the reservation guards
a real clash on pre-Fox too.
Plan: `team_compiler/blue_port.md` "Referee export processing"; `team_compiler/README.md`
TC-REF-04.

## 2026-10-08 — team_compiler — A pre-Fox per-kit model set is listed by its lowest variant's entries, kit token spelled `kitN`, material included
Decision: on PES 15-17 every variant of a per-kit `.model` set is packed under its own name and
the set has one `face.xml` entry per entry of its lowest variant (its own, and its hands' when
it is hand-split), with the kit token in `path` and `material` spelled `kitN`; the `material` is
the lowest variant's `.mtl`, respelled when it carries that variant's token, so each other
variant's `.mtl` must go by the respelled name, and a variant whose own search finds another
file is `kit_variant_mtl_differs` (W), still packed. Not compiled yet, named by the pre-Fox
gate: a per-kit model under `ingame_face`, in a shared `Boots/` or `Gloves/` folder taking an
id, or behind a `.common` link.
Why: the plan gave the model-path rule (one entry naming `pants_kitN`) and left the entry's
material open. The pre-Fox exe change that reads `kitN` does not exist yet, so the compiler
fixes the contract, and respelling the whole entry is what a load-path hook does with no special
case (the legacy `u0XXXp0` magic rewrote any path holding its token); per-variant `.mtl` files
then work under their natural names, while a shared `pants.mtl` carries no token and is written
as it is. The three unlisted places have no `face.xml` entry to collapse (a shared boots folder
merges into one `boots.model`; the shared `glove.xml` and the Common models task are later
steps), and compiling them as Fox does would silently merge or double a variant.
Plan: `team_compiler/pipeline.md` "Kit-dependent assets"; `model_format.md` "Kit-dependent
assets (`kitN`)"; `team_compiler/messages.md` `kit_variant_mtl_differs`; `team_compiler/README.md`
TC-CMN-07.

## 2026-10-08 — team_compiler — Under `ingame_face` on PES 15-17 a Common link's files are copied in as parts
Decision: in a player folder holding `ingame_face` compiled for PES 15-17, a `.model.common` link
named as boots or gloves stands for the Common model's files as a part of his own package: the
model is one more input of his boots merge, or one more model of his gloves written under its
own name with a `glove.xml` entry, and the `.mtl` its search finds (a Common one, or his own
override) is read and written with it; a `.mtl.common` link one of his parts uses is copied the
same way. The Common textures such a `.mtl` names stay in the team's Common output, where the
written `.mtl` names them. A link named as face content is validation's
`ingame_face_explicit_face_model`, as a file is; a link to a per-kit model has no role.
Why: the marker means no `face.xml`, the one place a pre-Fox output names a Common model by
reference, and a boots merge needs the bytes in any case. Naming a Common path from a
`glove.xml` is a shape no legacy tool wrote and no game test covers, while copying the files in
is what a combined gloves folder's models already get; one rule for both packages. Fox bakes a
link's model into the package the same way (`player_folders.md` "Common model links and model
merging").
Plan: `aesthetics_export/player_folders.md` "`ingame_face` with shared links";
`team_compiler/README.md` TC-MOD-44, TC-MOD-45.

## 2026-10-08 — team_compiler — A user `face.xml`'s `./` reference packs its file under the referenced name; a Common reference is written as the Common output packs the model
Decision: on PES 15-17 a member's own `face.xml` is resolved and re-serialized by Red's rules:
a `./<name>` reference (its `*` read as `win32`) names a file of the face, case-folded, which is
packed under the referenced name as the xml spells it, the reference written as it is; a
`model/character/uniform/common/<3 chars>/<name>` reference has the segment replaced by the
team ID and names a file directly in `Common/`, a `.model` written as the Common output packs
it (`oral_<stem>_*.model`), a `.mtl` under its name; a kit token passes through, the reference
existing when a variant of its set is among the files; any other form is `xml_path_unchecked`,
verbatim. The listed model's `.mtl` for `model_material_undefined` is the entry's `material`,
not the search's. The xml's own `<dif>` is decoded and checked as a `face_diff.xml`'s is, and
conflicts with a face diff file beside it. A folder holding a `face.xml` has a face, whatever
models it holds. A `face.xml` in a shared face folder stays refused by the pre-Fox gate.
Why: the plan left the file names open ("re-serialized", "the team ID substituted"). The one
real member xml in the fixtures (the pre-Fox tracer's Fumos folder) names `./face_high_*.model`
beside `face_high_win32.model`, and Red's output keeps both as written: renaming the file to
its `oral_` packed name and respelling the reference would make the compiler rewrite the
member's statement of what the game loads, the thing the plan's "kept verbatim" rule protects.
Common models are the other way round because the Common output renames every model (Red's
`model_names_fix`), and Red respells the xml to match (`update_xml_for_renamed_common_models`):
a verbatim Common reference would name a file the output does not hold. The search's `.mtl` can
disagree with the xml's (Fumos's `oral_glove_l_win32.model` searches to `boots.mtl`), and a
check against the wrong `.mtl` would drop a working folder.
Plan: `team_compiler/messages.md` "User-supplied `face.xml`" (the resolution paragraph);
`team_compiler/README.md` TC-XML-01; `GLOSSARY.md` "User-supplied `face.xml`".

## 2026-10-08 — team_compiler — The pre-Fox texture-existence check runs in the deep pass, by the stem rule the face task points `.mtl` paths with
Decision: `mtl_texture_not_found` and `mtl_texture_unused_missing` are deep-pass findings on pre-Fox, so
`check` reports them; Fox's `fmdl_texture_not_found` stays in the face task. A `.mtl` material
is mesh-used when a model the pass pairs with that `.mtl` (the search, or the xml entry) binds
its name; every `.mtl` of a folder is checked, an unpaired one's misses as Info. A sampler path
is supplied, whatever directory it spells, when the folder holds its stem (own textures, the
linked shared face's, a referee's combined boots' and gloves', a texture link's linked stem, a
kit set's variant); past that a `./` or bare path is missing, a Common-form path is looked for
in `Common/` and the installed CPKs (a lookup that cannot be made, or an export with no team
ID, holds nothing), any other path and a `dummy_*` stem are not looked up. A shared face
folder's `.mtl` files are checked against its own textures alone. A `Common/` `.mtl`'s mesh-used
names are those every kept `Common/` model binds. `mtl_texture_not_found` is a Warning keeping
the folder, not Fox's Error. A pre-Fox texture link is still never satisfied by an installed
CPK.
Why: the plan's row said E/W, but the pre-Fox tracer's Fumos face, a cup-played export, names
`./face_edithair_specular_roughness.dds` on a mesh-used material and holds no such file: Red
warned and compiled it, and the game played it. Which samplers a pre-Fox shader reads is not
known, so an Error would refuse faces the game plays on a guess, and the parity standard is
Red's output. TC-XML-07 says "checked", and the deep pass already pairs each model with its `.mtl` and
has parsed both, so a compile-time check would carry the bound names into planning and read
the `.mtl` a second time for nothing; Fox checks in the task only because the merged model's
paths are pointed there. The stem rule is the pointing rule, so a path the check calls supplied
is one the task points, and a path it calls missing is one the task leaves as written. A
shared face is a complete face, so a texture only a combiner holds would make its check depend
on who links it. A `Common/` `.mtl` is packed once for the team, before any player's pairing is
known, so the Common models' names are the only used set the pass can state. Lifting the link
rule needs the face task to point a targetless link by its own stem, a change of the task, not
of this check.
Plan: `team_compiler/messages.md`, the paragraph after "Texture existence is checked **deep**".

## 2026-10-08 — team_compiler — `dds_compression` deflates at level 6 with `flate2`'s default backend
Decision: on PES 15-17 every emitted DDS is WESYS-wrapped at zlib level 6, through `flate2`'s
default `miniz_oxide` backend; the `zlib-rs` backend the plan suggested is not adopted.
Why: the plan asked for a measurement of levels 1-3 against 6 before choosing. On 36 real DDS
files (40.8 MB: a PES 17 kit pack, a team's 2048² kit textures, a 5.6 MB Common body, five
1.4 MB model textures) level 1 gives 8.6 % of the raw bytes in 47 ms, level 3 6.7 % in 161 ms,
level 6 6.4 % in 302 ms, level 9 6.2 % in 713 ms (`.tmp/bench_wezlib/results.md`). The whole
set takes a third of a second at level 6, so the lower levels save a tenth of a second per
export and cost 5 % (level 3) or 35 % (level 1) more bytes in every CPK the game loads; level 9
doubles the time for 3 %. Level 6 is also Red's, so a compressed CPK is as small as the cup is
used to. `zlib-rs` is a new dependency, which needs the maintainer's yes, bought for a cost
already under a second.
Plan: `team_compiler/settings.md` "DDS compression cost".

## 2026-10-08 — team_compiler — A `.model` collar compiled for PES 15-17 is written unchanged
Decision: a collar already in the target's format (`collar_<ID>.model` on pre-Fox, as an FMDL
on Fox) goes into the CPK byte for byte under the game's three-digit name; its material names
are the author's. The renaming to the stock collars' names (`uni_collar`, `uni_shirts`) applies
to a collar *converted* for pre-Fox (4.17), not to one passed through.
Why: the plan said "a collar compiled for pre-Fox has its materials named as the stock collars'
are" without saying whether a `.model` source is renamed too. The author of a `.model` collar
made it for that engine against the shared `uniform.mtl`, as Red passed it through; the
converted case is where the names come from another engine's materials and need the stock
names to be found at all. Red wrote the file as it was.
Plan: `team_compiler/pipeline.md` "Collars", the sentence "A collar already in the target's
format".

## 2026-10-08 — team_compiler — A re-emitted PES 15-17 kit config is written where the edited bins go
Decision: on PES 15-17 the installed loose kit configs are gathered by the working-bin walk
(nearest CPK first), and the ones a `Midcup` export patches (FPC values, collar) are written
through the bins path: into the bins CPK with teams parts, under the test prefix in test mode,
with the overrides applied, and never wrapped by `dds_compression`.
Why: the plan said the pre-Fox configs are "located in the same installed CPKs, patched, and
re-emitted" without saying which CPK carries them with parts or where test mode puts them. A
patched config is an edit of the installed state, as the `UniformParameter.bin` edit is on
Fox, not content the export supplied, so it goes where that edit goes; a team's part holds
what the team sent. No path can collide: an absent slot has no kit task.
Plan: `aesthetics_export/fpc_toggle.md` "Kit slots absent from the export are patched in
place", the "Pre-Fox" sentence.
