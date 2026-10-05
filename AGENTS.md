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
commit (`git commit -F <file>`; PowerShell has no heredocs). Once a release exists, the review
before that commit also asks whether a member would notice the change against the last release,
and if so checks the diff for its `CHANGELOG.md` `[Unreleased]` line (`CONTRIBUTING.md`
"Commits").

**Before a tool phase** (any phase whose deliverable is a tool crate, or savefile writing): the
tool's plan gets an "Acceptance" section: stable-ID, GIVEN/WHEN/THEN scenarios for the
user-observable behavior that phase delivers (format in `CONTRIBUTING.md` "Testing"). Written
just-in-time for that phase only, never for the whole project up front: requirements written far
ahead of the code that meets them go stale. Lib phases do not get one; for a format crate the
fixture and the parity standard *are* the spec.

**Closing a phase**, in this order: (1) **converge**: audit the code against the phase's plan
sections and acceptance IDs, the lead's own audit first (every plan code block compared line by
line with the type it specifies, every step's verify criterion re-run), its gaps fixed, and only
then the cross-family reviewer (below), so the reviewer's attention goes to what the lead could
not see rather than to what it did not look for; every requirement that is missing, partial or
untested becomes a new worklog step, and the phase stays open until those are done. A requirement the plan *explicitly
defers* to a later phase (a "before implementing X, extend Y" note, an open question naming its
phase) is not a gap and stays future tense. The lead's audit ends with a **design-health pass**
over the phase's crates, separate from requirements coverage: the design-tell sweep (below) run
over the whole phase's code rather than one diff, `just mutants <crate>` run over each crate with
every survivor triaged (`CONTRIBUTING.md` "Mutation runs"), and every `pub` item listed with the
consumer that justifies it (a crate, the CLI, the bindings, or a plan section naming one). A crate
that reads a file format also gets a **census**: every file of that format on the maintainer's
machine (found through the Everything index, archives opened) run through read, the semantic
round trip and `check`, tallied by outcome, each failing class diagnosed before it is fixed or
recorded as a refusal, and re-run after every fix with a regression check (`.tmp/model_census/`
is the template). Fixtures and mutation runs only test the files someone thought to add; the
first census, at 2.20g, found about 1900 community `.model` files the reader refused. A census of
a bit-packed format also tallies the bits real records set that no schema field covers: a round
trip keeps them, so only this tally shows a field modeled too narrow (at 2.20i it found PES 18's
dribbling-arm motion read as 2 bits of 3, misread for 2,808 players). Coverage asks
"is everything the plan wants there?"; this pass asks "did the phase make the code harder to
change?", which no test or lint measures and which compounds silently across phases. The
whole-crate run happens twice per crate: in the lead's audit, where its survivors seed the first
rework brief, and once at close, after the reviewer loop ends. Each rework round in between is
measured with `just mutants-diff <commit the round started from>` (2.20f: 45-53 minutes per
whole-crate run, 3 for a round's diff); (2) **rewrite** the parts of the phase's plan sections
that the phase delivered in the present tense, as a description of what now exists (acceptance IDs
stay, as the behavior contract), moving nothing to a new document and leaving deferred parts as
they are; (3) collapse the worklog's step list to the phase row, keeping its `manual: checked`
lines, which are the manual scenarios' proofs (`CONTRIBUTING.md` "Testing"). Converting per phase, not at the
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
criterion, and the rule that a plan gap is *reported*, never silently decided. The same rule
covers the brief itself: an item of the brief that turns out unused or wrong (a dependency the
code never calls, a helper that does not exist) is reported in the closing section, not
implemented verbatim; the sidekick sees the code, the lead wrote the brief from a reading of it
(2.18's brief listed a `log` dependency the crate never used, and it landed until review). The brief's
verification list names the tests of every crate that *consumes* the one being changed, not
only the crate's own: a change to `fmdl`'s reader that every game `body.skl` tripped passed
`cargo test -p fmdl` and was caught only by `model_convert`'s tests at the lead's gate run.
`fmdl`'s and `pes_model`'s consumers include `python_bindings`, whose test is `just bindings`,
outside `just gates`: 2.20g-h added fixtures to both crates, the smoke test's fixture counts
went stale, and only CI's `bindings` job saw it (run 24, 2026-09-29). A brief that adds tests says to insert them with the editor tool inside the file's existing
`#[cfg(test)]` module, never to append them with shell redirection, which lands them after the
module's closing brace (three times in 2.20f). The lead
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
lines in this diff, a comment naming or alluding to a legacy tool outside a parity test (`Red`,
`Blue`, `reference`, `add-on`, `Python`, `texconv`, `always`). The **mutation run** (`just mutants-diff <last reviewed commit>`) is the
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
returns every concern it found, ranked, each marked verified or suspected. Answer every one in the
turn report: accepted → what changed; rejected → one line why. A concern silently dropped is the
failure mode this exists to prevent. **There is no cap on concerns; rounds are bounded by the
accept rate, not by the lead's sense of importance.** A cap made reviewers curate instead of
report: 3.1's first round had a seven-concern cap, and its reasoning trace set out to pick "seven
issues that a parent will likely accept, ideally five or more", so whatever it judged unlikely to
be accepted went unreported, and a later round need not find it again. The ranking by
reachability (below) is what keeps impossible edge cases at the bottom of the list. So a critique of one surface runs another round only when at
least five of the last round's concerns were accepted (the signal was real); the next round gets
the rework diff and the prior rulings so it does not repeat them, and the loop stops at a round
under five accepted. There is no round ceiling: 2.20d's first three rounds each returned seven
(then the cap), all accepted, most of them parity and memory defects rather than padding, so a ceiling of three
stopped the loop where its signal was strongest. The stop therefore rests on the lead's
rulings: a concern is rejected, with its one-line reason, when its input cannot reach the code,
the plan or a decision entry already settles it, or it restates an accepted concern's class
without a new defect. **Reviewer briefs rank concerns by reachability.** First comes what a
real file, the crate's own readers or its own operations produce. A hand-built value that none
of them produce is low value when the crate refuses it with an error, and ranks high only
when it panics, hangs, or writes a wrong file without an error. From 2.20f's second round on,
the top of the list went to models no reader or operation produces. Round 4's brief carried this line, and
none of its four concerns needed a hand-built model. **Reviewer briefs also ask for
over-engineering**, and the lead's rulings watch for it, above all on plans: a reviewer asked
only for gaps reports only gaps, so each round adds a sentence per edge case it can name, and
the fix for one concern becomes the next round's concern. The brief asks it to flag what could
be cut (a special case a general rule already covers, machinery no scenario needs); the lead
prefers one general rule over a growing list of cases, and rejects a narrow addition that
distinguishes no behavior a real export reaches. At 3.1 (maintainer's warning), the rulings on
the sidekick's round S2 replaced a growing list of "not compilable yet" cases with one short
rule, what the tracer compiles, and rejected three narrow additions (a `NO_USE` inside a
wrapper, folders inside `Common/`, a player folder named `Players`); from GPT round 6 on, both
reviewers' briefs asked for over-engineering. Past the fifth round on one surface the lead reports the counts and the
classes found to the user and continues unless told otherwise: a check-in, not a stop. Each
round's reviewer is a fresh `run_subagent`, never a resumed one (`resume`), even though resuming
saves the re-exploration: 2.20d's fourth round ran both on the same surface, and the reviewer
resumed from round three returned four concerns, all among the fresh reviewer's seven, which
found three more, all accepted; a resumed reviewer keeps its earlier reading and its blind spots
with it. The
count *returned* is not a stop signal: a reviewer told to keep the list short withholds
(2.20c's first round returned five with its reasoning trace saying it aimed for "3-5 strong
concerns" because the brief said "do not pad"; the second round, run as an experiment, returned
three verified concerns, all accepted, one of them contradicting a lead ruling). So the brief
never says "do not pad" or names any number of concerns, as a cap or a target; it says to report
every concern found, ranked by reachability, and that a round with fewer than five accepted ends
the loop, so a verified concern is never withheld to keep the list short; padding is caught by
the accept rate, which is what that rule is for. At converge the surface is one crate (or one
coupled pair), never the phase. The user can request a critique at any time with `/duck`.

## Read order by task

You do not need to read `docs/plans/core/README.md` end to end. Read this file and the worklog's
"Current status", then:

| Working on | Read |
|---|---|
| Any code change | `docs/CONTRIBUTING.md`, then the row below that matches; `docs/GLOSSARY.md` whenever a term is unfamiliar; do not infer domain terms from their English meaning |
| A format or leaf lib crate (`cpk`, `fpk`, `fmdl`, `pes_model`, `ftex`, `dds_convert`, `fox2`, `uniparam`, `wezlib`, `vtree`, `archives`, `fpc`, `teams_list`, `color_tools`, `elevation`) | `docs/plans/libs/README.md`; for `fmdl`/`pes_model` also "Blender integration" in `model_conversion/gltf.md` |
| `kit_config` | `libs/README.md` for the crate, `kit_config_editor.md` for the format reference |
| `python_bindings` | "Blender integration" in `model_conversion/gltf.md`; guardrail 4 in `core/architecture.md` |
| A tool crate | That tool's plan, plus "Tool plugin interface", "Event system", and the tool-crate skeleton under "Crate structure" in `core/architecture.md` |
| Anything that reads or writes exports | `docs/plans/aesthetics_export/README.md` (object model, folder conventions, validation) |
| `studio_core` or `studio` | `core/architecture.md` and `core/gui.md` |
| Model conversion or glTF | `model_conversion/README.md`, `model_format.md` |
| Savefile | `pes_savefile/README.md`, then `save_editor.md` |

Several plans carry "Resolved decisions" / "Open questions" sections. Check them before asking:
your question may already be listed, with the phase in which it gets resolved.

## Fundamental concepts

- **Speed is the primary goal.** The suite exists because the Python tools are slow. Everything
  else is second, or tied for first.
- **Platform + plugins.** `studio_core` holds only what runs with zero tools installed (shell,
  `StudioTool` trait, settings framework, common widgets, `PipelineEvent` types). Every tool is its
  own crate under `crates/tools/`; every piece of shared functionality is its own lib crate under
  `crates/libs/`; `studio` is the crate of the single binary, `4cc-studio`, that registers tools
  and launches the GUI or dispatches `4cc-studio <tool-id> <command>`.
- **Two engines, one IR.** PES 15–17 use pre-Fox formats (`.model` + `.mtl`); PES 18–21 use Fox
  formats (FMDL, FPK, FTEX). Cross-format model conversion goes through `model_convert`'s IR at
  compile time; same-format work stays in the format crate's `ops/` and skips the IR unless the plan
  names an IR operation (hand auto-split, cross-version skeleton retargeting). Authoring format is glTF
  with optional `PES_bone`/`PES_mesh` extensions plus `materials.toml`.
- **Exports are the unit of work.** Three kinds: aesthetics (player folders with `settings.toml`,
  kits, portraits: the project's primary motivation; referee exports are aesthetics exports with
  the `/refs/` team name), music (`.4ccm`), balls. Aesthetic settings that used to be edited in the
  savefile by hand live in TOML inside the export and reach the game at compile time: on Fox as
  rows of the output CPK's database tables, on pre-Fox written to the savefile. The old export
  layout is not supported by the compiler; the Export upgrader migrates it
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
- **Inputs are trusted, not hostile.** Files come from cup members'
  exports and the game's own install, a fairly trustworthy environment. Malformed input still
  errors instead of panicking or writing a wrong file, but hardening against deliberately
  crafted files (allocation amplification, pathological sizes, characters no real file
  carries) is not a goal: a review concern that needs a crafted file to trigger is rejected on
  that ground, and reviewer briefs say so.

## Environment

The maintainer develops on Windows with PowerShell; Linux is a first-class target. Prefer Python
scripts over PowerShell one-liners for anything with quotes; never inline regex in PowerShell.
**Scripts never write a repo file in place**: `Path.write_text` truncates the file before it
validates its own arguments (one bad `newline=` emptied a 600-line source file that git had no
copy of). Write to a sibling temp path and `os.replace` it, and run every assertion before the
write. Small edits use the editor tool, not a script. Before editing a tree the sidekick left,
`git add -A` first, so every file, untracked ones included, has a blob in the index to restore
from. **No `git checkout -- <path>`, `git restore`, `git reset --hard` or `git stash` over
uncommitted work**: each discards whatever the index does not hold, and nothing in the reflog
gets it back (a checkout meant to undo a one-line perturbation also erased the unstaged golden it
sat in). A deliberate perturbation is undone with the editor tool, by hand; a git restore is
allowed only after `git add -A` has given every file a blob to restore from. Never redirect to
`nul` from bash: on Windows that creates a real file named `nul`, which git cannot index
(`git add -A` fails outright) and which only `Remove-Item -LiteralPath "\\?\<full path>"` can
delete; use `/dev/null` in bash, `$null` in PowerShell.

The Devin IDE compacts the sidekick's context by itself when it reaches 500K tokens, so a
full sidekick context needs no action from the lead; a brief still stands alone (file under
`.tmp/`, read in full), because a compaction may fall between two handoffs.

**Whole-crate mutation runs are split with the maintainer's VPS.** On the maintainer's PC the
user environment variable `STUDIO_MUTANTS_REMOTE=bonfire` (an alias in `~/.ssh/config`) makes
`just mutants <crate>` run half the mutants there (`scripts/mutants.py`: the working tree is
sent as a git bundle, the remote shard runs at low priority, its results land in
`mutants.out/remote/` and the combined survivors in `mutants.out/missed_all.txt`). Benchmarked
on 2026-09-27 on one commit, the VPS is about as fast as the PC (`fox2` whole crate 167 s
against 256 s, one eighth of `fmdl` 460 s against 433 s) with identical results, so the split
should roughly halve a whole-crate run, and it moves half its energy off the home machine.
Agent and IDE shells started before the variable was set do not see it, so the script also
reads it from the user's registry environment; the run's first line says which mode it took
("splitting the run with bonfire" or "running every mutant on this machine"). At 2.20i an
agent shell without the variable ran all 806 `pes_savefile` mutants locally, over three
hours. An empty value in the process opts out. `mutants-diff` splits the same way only when
its estimated local time is at least 4 minutes (`SPLIT_THRESHOLD_SECONDS`): the estimate is the
unmutated baseline plus each mutant's crate's measured mean seconds per mutant over the two
jobs (`target/mutants-cost.json`, refreshed by every local run; a never-run crate falls back to
its size, a never-measured baseline to 60 s), because cost follows a crate's test suite, not
the mutant count (0.7-1.4 s for most crates, 33-37 s for `pes_savefile`). The baseline is a
cold build in a fresh copy of the tree, about a minute, and the mean counts each job's cold
first build and every timeout: until 3.z the estimate left all three out, and a 38-mutant diff
estimated at 80 s took 270 s (now 266 s). The local half runs at below-normal priority with
each of its two cargo processes given half the logical CPUs (`sized_config`, the remote half's
sizing), not with two CPUs held back: the machine stays usable by priority, and the run gets
what the user leaves idle (measured level with the old cap on the 3.z diff). A starved test can
then time out, so a timeout that no infinite loop explains is rerun before it counts as caught.
A build killed for memory (the remote half's cap) is filed by
cargo-mutants as unviable, which hides an untested mutant: at 3.z eframe's dependency tree
outgrew the 6 GiB cap and 12 remote mutants went untested that way, so the scripts now list
every killed build and fail the run. Below
that the split's fixed overhead, about a minute plus up to 30 s of polling, eats the gain. At
2.20k a 32-mutant `pes_savefile` diff took 11 min 57 s locally; split, the local half took
417 s and the remote half 502 s from its launch (the VPS ran about a fifth slower; the total
wall time was not recorded). Manual ssh from
Git Bash uses `/c/Windows/System32/OpenSSH/ssh.exe bonfire`: Git's own `ssh` cannot reach the
Windows agent that holds the key.

**The VPS's production Fluxer instance comes first.** The remote half runs as the transient
system service `studio-mutants` (`sudo -n systemd-run`): `MemoryMax=9G` with no swap (6G
until 3.z, when eframe's dependency tree outgrew it; since then each remote cargo process also
builds with 2 jobs, `REMOTE_BUILD_JOBS`; 8G until 4.6b, whose run
peaked at 7.90 GiB; past 9G the next lever is fewer build jobs, not more memory), so a
runaway mutant is OOM-killed inside the unit and counts as caught; `OOMPolicy=continue`, so that
kill does not stop cargo-mutants; `CPUWeight=idle`, because `nice` cannot keep a `user.slice`
process off Fluxer's CPU under cgroup v2. On 2026-09-29 the old uncapped half filled the host's
RAM and swap (`display_name` with `i *= 1`, a loop that grows a `Vec` forever), and the global
OOM killer, which could as well have picked Postgres, took the test. The run prints its peak
(`remote memory peak`). The cap is final: the host has no more memory to give (4.7b's run
peaked at 8.43 GiB), so a peak near the cap changes nothing by itself. When a build is killed
at the cap, the run fails and lists the untested mutants (`report_killed`), and the fix is
then a lower `REMOTE_BUILD_JOBS`, never a higher cap. A running half is stopped with `sudo systemctl stop
studio-mutants` on the host.

**The remote half runs detached** from any ssh session (`~/studio-mutants/run/`: `job.sh`,
`pid`, `log`, `exit`, `memory_peak`), and the script polls it every 30 s with short ssh calls. A dropped link
costs a poll, not the run: at 2.20i the remote half still hung off one long ssh session and
died with it. If `just mutants` itself stops, `just mutants-collect` waits for the remote half
and fetches it, including a died half's partial results. A new run refuses to start while a
remote half is running or uncollected. **A failed half is reported, never restarted:**
`cargo mutants` does not resume, so a rerun starts from zero, hours of work (at 2.20i the
sidekick restarted the VPS half and waited on it although the local half had already tested
every mutant). A brief that starts a whole-crate run says so.

**An archived run keeps its summary, not its logs.** A run kept aside (in `.tmp/mutants_<step>_<what>/`, before
the next run overwrites `mutants.out/`) copies only `*.txt` and `outcomes.json`, never the
per-mutant `log/` or `remote/` trees: those are what reproduce a survivor, and a survivor is
reproduced by re-running its mutant, not by reading a week-old log. Whole-folder copies had
grown `.tmp/` to 17 GB by 3.8. Each local run builds in a `%TEMP%\cargo-mutants-4cc-studio-*.tmp`
copy of the tree, deleted when the run ends normally; a killed run leaves its copy behind
(some with a multi-GB `target/`), so after stopping one, delete its copy. `.cargo/mutants.toml`
sets `gitignore = true` so the copies leave out `.tmp/` and `target/`: before 3.8 each copy
carried all of `.tmp/`, and two 3.7 runs died of a full disk.

Three model families, three roles. Lead: Claude (Fable) in Devin CLI's Fusion mode. Sidekick:
SWE-2 (Kimi lineage), reached through the `sidekick` tool; a cold probe on a spec-in-hand crate
showed it follows `CONTRIBUTING.md` literally and reports plan gaps instead of deciding them, so it
is trusted with implementation, not with judgment. Reviewer subagent profiles (global,
`%APPDATA%\Devin\agents\`): `gpt-astra-high` (GPT, latest Astra) when the lead is a Claude, **or
when you do not know what model you are**; `fable-medium` (Claude) only when you know you are a
GPT. The reviewer reviews; it is never asked to edit. Cost is not the constraint on any of these;
time is, hence the batching rule above.

Two harness limits, both measured. **`run_subagent` profiles have no `skill` tool**, and the
reviewer profiles (whose files list `allowed-tools`) have `read`, `grep` and `find_file_by_name`
only. A brief that says "invoke the `karpathy-guidelines` skill" sends a subagent globbing the
repo for a file that is not there; give it the file to read instead,
`%APPDATA%\devin\skills\karpathy-guidelines\SKILL.md` (outside the repo, and its `read` tool
reaches it). The `sidekick` tool has the `skill` tool, so its briefs keep the invoke wording.
**The sidekick's role can be played by a `swe-2-high` subagent** (same model, profile file with
no `allowed-tools`, so it has the shell, `edit` and `write`; measured 2026-10-05): run it in the
foreground (a background subagent has unapproved tools denied), resume the same one per task
(`resume: <agent_id>` keeps its memory, as the sidekick's handoffs do), read its size with the
`context-usage` script before each resume (its line names the profile and its first task), and
start a fresh one once it is at 350K or more, before an autocompaction can land mid-task. The
`sidekick` tool's single sidekick cannot be reset. **Compaction does not fire while a sidekick handoff or a background subagent is in
flight**, so a run started "to save time" ahead of the `[[/compact]]` sentinel costs the
compaction instead (2.20b's mutation runs were started that way and the sentinel was ignored).
Compact first, then start the run in the resumed turn.

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

**A recommendation is applied, not asked.** When the lead already has
a clear recommended choice for one of the cases above, it applies that choice and logs the
decision (plan edit, then decision entry), and the turn report names it so the maintainer can
reverse it. It stops to ask only when the choice is truly ambiguous (no option is clearly
better on the evidence) or needs information only the maintainer has (in-game behavior, cup
practice, a preference). New dependencies and `unsafe` still need a yes first; a dependency the
plan already names (`rayon`, `crossbeam-channel`, ...) is accounted for, not new.

Logging a decision means, in this order: edit the relevant plan section so the plan stays the
source of truth (keep its "we do X, not Y, because Y causes Z" style), then append an entry to
`docs/DECISIONS.md` (format at the top of that file) immediately, not at the end of the turn.
Who decided and when is the entry's alone: no `(maintainer, <date>)` note in a plan, here or in
`CONTRIBUTING.md`.

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
