# 4cc Studio - Agent briefing

4cc Studio is a Rust suite of every tool the 4cc community uses to run its PES (2015–2021) cups:
a Team compiler (aesthetics exports → CPK archives), a save editor, and a dozen smaller tools, all
in one `egui` binary that also works as a CLI. The full architectural plan is in `docs/plans/`
(index: `docs/plans/README.md`). This file is the short version: how the project is organized, how
the codebase works, and what to do when the plan runs out. Before writing or changing code, read
`docs/CONTRIBUTING.md` in full: architecture rules, code style, verification gates.

## Working documents

Five documents, five jobs. Do not let content leak between them; two homes means they drift.

| Document | Holds |
|---|---|
| `docs/plans/*.md` | The *why* and the spec: architecture, formats, rationale, resolved decisions, and each tool's "Acceptance" section. Future tense while a phase is open; rewritten in the present tense when it closes, so the plan is always the description of what exists plus what comes next |
| `docs/CONTRIBUTING.md` | How code is structured, written and verified |
| `docs/GLOSSARY.md` | One line per domain term, pointing at the plan section that owns it |
| `docs/WORKLOG.md` | *Where we are*: current status, phase/step checklist, current-state gotchas, open issues, dated log |
| `docs/DECISIONS.md` | Choices made where the plan was silent (append-only) |

**Before a step:** read the worklog's current status, then the plan section the step points at,
from the document, not from memory. **After a step:** run the gates (`CONTRIBUTING.md`), mark the
step done with a one-line summary and the files touched, update "Current status", add a log line,
commit (`git commit -F <file>`; PowerShell has no heredocs).

**Before a tool phase** (any phase whose deliverable is a tool crate, or savefile writing): the
tool's plan gets an "Acceptance" section: stable-ID, GIVEN/WHEN/THEN scenarios for the
user-observable behavior that phase delivers (format in `CONTRIBUTING.md` "Testing"). Written
just-in-time for that phase only, never for the whole project up front: requirements written far
ahead of the code that meets them go stale. Lib phases do not get one; for a format crate the
fixture and the parity standard *are* the spec.

**Closing a phase**, in this order: (1) **converge**: audit the code against the phase's plan
sections and acceptance IDs, the lead's own audit first (every plan code block compared line by
line with the type it specifies, every step's verify criterion re-run), its gaps fixed, and only
then the cross-family reviewer (below), so the reviewer's seven slots go to what the lead could
not see rather than to what it did not look for; every requirement that is missing, partial or
untested becomes a new worklog step, and the phase stays open until those are done. A requirement the plan *explicitly
defers* to a later phase (a "before implementing X, extend Y" note, an open question naming its
phase) is not a gap and stays future tense. The lead's audit ends with a **design-health pass**
over the phase's crates, separate from requirements coverage: the design-tell sweep (below) run
over the whole phase's code rather than one diff, `just mutants <crate>` run over each crate with
every survivor triaged (`CONTRIBUTING.md` "Mutation runs"), and every `pub` item listed with the
consumer that justifies it (a crate, the CLI, the bindings, or a plan section naming one). Coverage asks
"is everything the plan wants there?"; this pass asks "did the phase make the code harder to
change?", which no test or lint measures and which compounds silently across phases; (2) **rewrite** the parts of the phase's plan sections
that the phase delivered in the present tense, as a description of what now exists (acceptance IDs
stay, as the behavior contract), moving nothing to a new document and leaving deferred parts as
they are; (3) collapse the worklog's step list to the phase row. Converting per phase, not at the
end of the project, is what keeps the plan true while the memory of the work is fresh.

**Lead and sidekick.** Sessions run as a lead model that plans, briefs, reviews and talks to the
user, and a sidekick model (a different family) that implements and runs the gates. The lead
hand-writes only what is *correctness-critical*: things whose wrong version passes instead of
failing: workspace lints, CI, `rust-toolchain.toml`, the acceptance-ID scanner, any fixture or
golden file, any measurement. Everything with a plan section to implement against is briefed to
the sidekick. A brief names the plan section(s) and `CONTRIBUTING.md` as reading to do *before*
coding (the sidekick does not inherit this file's context), tells the sidekick to invoke the
`karpathy-guidelines` skill first (skills the lead has loaded are not active in the sidekick;
an uninstructed sidekick never invokes one), plus the acceptance IDs, the `→ verify:`
criterion, and the rule that a plan gap is *reported*, never silently decided. The brief's
verification list names the tests of every crate that *consumes* the one being changed, not
only the crate's own: a change to `fmdl`'s reader that every game `body.skl` tripped passed
`cargo test -p fmdl` and was caught only by `model_convert`'s tests at the lead's gate run. The lead
reads the whole diff before it lands, not the report about it; the report is a claim, the diff is
the evidence. Two failed sidekick attempts on one brief means the brief is suspect before the
sidekick is. **A brief is sized for one review.** Past about 500 lines of new code, a diff is
read but not resteered: a wrong shape in the first module is already copied into the third by
the time the lead sees it (`model_convert` took a 666-line rework commit after a single review
of its first four steps). A step expected to exceed that is briefed as ordered slices, each
landing as its own commit and reviewed before the next slice is briefed (`format/` first, then
`ops/`, then `check.rs`, for a format crate); generated tables and fixtures do not count toward
the size.

**The lead sees the report, not the run.** The harness shows the lead only the sidekick's final
report, never its compiler runs, so an error fixed by a workaround (a suppression, a loosened
assertion, an `#[ignore]`, a discarded `Result`) is invisible unless declared. Every brief
therefore requires a closing section, "Errors hit and how each was resolved": one line per
compiler, clippy or test failure met during the work and the fix applied, plus an explicit list
of any suppression, ignored test, removed or weakened assertion (or "none"), and, for every test
added for new behavior, the failing assertion text from its red run before the change ("red
first" is otherwise a claim the lead never sees; a test with no meaningful "before", such as a
new format crate's fixture round-trip, says so). Omitting the section or an item is a brief
violation, not an oversight. On the lead's side, every review before a commit includes two
sweeps of the diff and re-runs the gates in the real workspace; the sidekick's pasted gate
tails are a claim. The **honesty sweep** looks for a hidden failure: `#[allow`, `#[expect`,
`#[ignore`, `.ok();`, `let _ =`, `drop(` on a `Result`, `unwrap_or_default`, `unwrap_or(`. The **design sweep** looks
for code that passes and is worse, the thing neither the gates nor a model's training penalize:
`impl .* for` (a new trait: does it have two real implementors?), `dyn `, `_ =>` on one of our
enums, `.clone()`/`.to_vec()` on bulk data, `as ` casts, two functions differing in a name and a
branch, `pub` items with no caller outside the crate, a module that crossed roughly a thousand
lines in this diff. The **mutation run** (`just mutants-diff <last reviewed commit>`) is the
third check and the only one that is a measurement rather than a reading: each survivor is an
assertion the sweeps cannot see because it was never written (the `kit_config` probe found
twenty such gaps in a crate that had passed both sweeps, converge and the cross-family
reviewer), and every survivor is triaged per `CONTRIBUTING.md` "Mutation runs" before the
diff lands, the missing tests going into the rework brief. Each sweep item is a
`CONTRIBUTING.md` rule; the sweeps exist because a rule nobody greps for is a rule the review
applies only when it happens to notice. When a review finds slop
neither list names, the fix is a new entry here or in `CONTRIBUTING.md`, not a longer review:
the rules are the project's whole substitute for taste, and any quality they do not express is
quality nobody is checking.

**Plan-defined shapes are copied, not recalled.** Where the plan gives a code block for a type,
a trait or a signature, the implementation starts from that block, pasted, and any deviation is a
decision entry, never a silent rewrite. Phase 1's first converge lost three of its seven reviewer
slots to shapes the lead had written from memory, all three of them written down in the plan.

**Working in parallel.** Lead and sidekick share one working tree, so when both write code at
once (the lead on a correctness-critical crate, the sidekick on a briefed one) they work in
disjoint crates, and each verifies with crate-scoped commands (`cargo test -p vtree`,
`cargo clippy -p vtree --all-targets -- -D warnings`) while the other's crate may not compile
yet. The lead never leaves the workspace unbuildable longer than one edit (declare a module only
once its file exists), and runs the full `just gates` once, after both are done. The brief says
which crate the sidekick owns and that the rest of the tree is in motion.

**Second opinion.** A model reviewing its own work is bounded by its own training: same data, same
blind spots. The sidekick's code is already reviewed cross-family by the lead; what is *not* is
everything the lead writes itself: plans, acceptance sections, briefs, converge audits, decision
entries. At these checkpoints, and only these, get a critique from the reviewer subagent of a
*different model family* than the lead (profiles in "Environment"), invoked with `run_subagent`:
(a) after writing an Acceptance section; (b) after a worklog step that added a new `pub` interface
or touched more than one crate, after the implementation *and* its tests are written and before
the tests run, so one critique covers both the code and the assertions; steps inside one crate behind
an existing interface are covered by the lead's diff review and by converge; (c) as the second half
of every converge audit, after the lead's own, the primary use: the one check of code against
*plan* (not against the brief, which carries the lead's assumptions) by a model that wrote
neither; (d) reactively, when
two attempts on the same premise have failed, the sidekick's included. **Batch, don't stream:**
renames, doc edits, one-file fixes with no new behavior are never reviewed on their own; their
diffs ride along with the next (b) review, or with converge. Never more than one *scheduled*
critique per step; the reactive one (d) is an escalation and is exempt from the ceiling. Hand the
reviewer *pointers* (plan sections, file paths, a diff written to `.tmp/review.diff`, the
acceptance IDs), never your own summary, which carries the assumptions it is there to catch. It
returns at most seven ranked concerns, each marked verified or suspected. Answer every one in the
turn report: accepted → what changed; rejected → one line why. A concern silently dropped is the
failure mode this exists to prevent. The user can request a critique at any time with `/duck`.

## Read order by task

You do not need to read `docs/plans/core.md` end to end. Read this file and the worklog's
"Current status", then:

| Working on | Read |
|---|---|
| Any code change | `docs/CONTRIBUTING.md`, then the row below that matches; `docs/GLOSSARY.md` whenever a term is unfamiliar; do not infer domain terms from their English meaning |
| A format or leaf lib crate (`cpk`, `fpk`, `fmdl`, `pes_model`, `ftex`, `dds_convert`, `fox2`, `uniparam`, `wezlib`, `vtree`, `archives`, `fpc`, `teams_list`, `color_tools`, `elevation`) | `docs/plans/libs.md`; for `fmdl`/`pes_model` also "Blender integration" in `model_conversion.md` |
| `kit_config` | `libs.md` for the crate, `kit_config_editor.md` for the format reference |
| `python_bindings` | "Blender integration" in `model_conversion.md`; guardrail 4 in `core.md` |
| A tool crate | That tool's plan, plus "Tool plugin interface", "Event system", and the tool-crate skeleton under "Crate structure" in `core.md` |
| Anything that reads or writes exports | `docs/plans/aesthetics_export.md` (object model, folder conventions, validation) |
| `studio_core` or `studio` | `core.md` "Architecture" and "GUI Design" |
| Model conversion or glTF | `model_conversion.md`, `model_format.md` |
| Savefile | `pes_savefile.md`, then `save_editor.md` |

Several plans carry "Resolved decisions" / "Open questions" sections. Check them before asking:
your question may already be listed, with the phase in which it gets resolved.

## Fundamental concepts

- **Speed is the primary goal.** The suite exists because the Python tools are slow. Everything
  else is second, or tied for first.
- **Platform + plugins.** `studio_core` holds only what runs with zero tools installed (shell,
  `StudioTool` trait, settings framework, common widgets, `PipelineEvent` types). Every tool is its
  own crate under `crates/tools/`; every piece of shared functionality is its own lib crate under
  `crates/libs/`; `studio` is the single binary that registers tools and launches the GUI or
  dispatches `studio <tool-id> <command>`.
- **Two engines, one IR.** PES 15–17 use pre-Fox formats (`.model` + `.mtl`); PES 18–21 use Fox
  formats (FMDL, FPK, FTEX). Cross-format model conversion goes through `model_convert`'s IR at
  compile time; same-format work stays in the format crate's `ops/` and skips the IR unless the plan
  names an IR operation (hand auto-split, cross-version skeleton retargeting). Authoring format is glTF
  with optional `PES_bone`/`PES_mesh` extensions plus `materials.toml`.
- **Exports are the unit of work.** Three kinds: aesthetics (player folders with `settings.toml`,
  kits, portraits: the project's primary motivation; referee exports are aesthetics exports with
  the `/refs/` team name), music (`.4ccm`), balls. Aesthetic settings that used to be edited in the
  savefile by hand live in TOML inside the export and are **written to the savefile at compile
  time**. The old export layout is not supported by the compiler; the Export upgrader migrates it
  once.
- **User-facing TOML is edited, never regenerated.** `settings.toml`, `config.toml`,
  `materials.toml` carry app-injected per-field comments that *are* the user documentation. Write them through `toml_edit`, which preserves them; `toml` (serde) is for
  read-only parsing.
- **Legacy tools are evidence, not source.** Red, Blue, 4ccEditor, Midcupping, Rigdio, SEN:P-AI,
  the converters and the Blender addons define *what* must be produced (formats, observable
  behavior), never *how* the Rust code is organized: no line-by-line translation, no copied module
  structure. Red's CPK output is the parity golden standard.
- **Findings, not messages, in libs.** Lib crates return stable finding codes with context; each
  tool's `messages.rs` maps them to user-facing text, severity and disposition. Tools report
  progress through `PipelineEvent`s addressed by `vtree::ScopePath`.
- **Parallelism is `rayon` + `crossbeam-channel`.** No async runtime anywhere in the workspace.

## Environment

The maintainer develops on Windows with PowerShell; Linux is a first-class target. Prefer Python
scripts over PowerShell one-liners for anything with quotes; never inline regex in PowerShell.
**Scripts never write a repo file in place**: `Path.write_text` truncates the file before it
validates its own arguments (one bad `newline=` emptied a 600-line source file that git had no
copy of). Write to a sibling temp path and `os.replace` it, and run every assertion before the
write. Small edits use the editor tool, not a script. Before editing a tree the sidekick left,
`git add -A` first, so every file, untracked ones included, has a blob in the index to restore
from. Never redirect to `nul` from bash: on Windows that creates a real file named `nul`, which
git cannot index (`git add -A` fails outright) and which only `Remove-Item -LiteralPath
"\\?\<full path>"` can delete; use `/dev/null` in bash, `$null` in PowerShell.

Three model families, three roles. Lead: Claude (Fable) in Devin CLI's Fusion mode. Sidekick:
SWE-2 (Kimi lineage), reached through the `sidekick` tool; a cold probe on a spec-in-hand crate
showed it follows `CONTRIBUTING.md` literally and reports plan gaps instead of deciding them, so it
is trusted with implementation, not with judgment. Reviewer subagent profiles (global,
`%APPDATA%\Devin\agents\`): `gpt-astra-high` (GPT, latest Astra) when the lead is a Claude, **or
when you do not know what model you are**; `fable-medium` (Claude) only when you know you are a
GPT. The reviewer reviews; it is never asked to edit. Cost is not the constraint on any of these;
time is, hence the batching rule above.

## When the plan has gaps

The plan is thorough but not complete, and it will be wrong in places. When you find a gap, an
oddity, or think of an improvement, do not ignore it and do not work around it silently.

**Act, then log**, when *all* of these hold:

- it is implementation-level: no change to user-visible behavior, CLI/GUI surface, file formats,
  or anything the game reads;
- it stays inside one crate and adds no dependency, no crate, no `unsafe`, no `#[allow]`;
- the plan is *silent* on the point, not contradicted by your choice.

**Stop and ask** when *any* of these hold:

- the plan says X and you believe X is wrong (a contradiction is not a gap); never quietly "fix"
  the plan;
- it touches game-facing output, export text formats, the savefile, the UI, or the CLI;
- it needs a new dependency, a new crate, a crate-boundary move, `unsafe`, or a clippy opt-out;
- there are two reasonable options and no plan text picks one (user preference is involved).

Logging a decision means, in this order: edit the relevant plan section so the plan stays the
source of truth (keep its "we do X, not Y, because Y causes Z" style), then append an entry to
`docs/DECISIONS.md` (format at the top of that file) immediately, not at the end of the turn.

## Conduct

Invoke the `karpathy-guidelines` skill before non-trivial work if it is available.

**The ownership standard.** Everything you produce must be maintainable by the maintainer without
an assistant present: code, plan text and decision entries alike. Code the reviewer cannot follow
and a plan section only the writing session understood are not done, whatever the gates say. When
an outcome fails this test, say so and simplify before reporting. (The code half is
`CONTRIBUTING.md`'s "boring Rust" intent; this is the standard it is accepted on.)

End every turn that changed something with a short report: what changed, which gates ran and their
result, what was not verified, second-opinion concerns and how each was answered, worklog steps
updated, decisions logged in `docs/DECISIONS.md`, and open questions.
