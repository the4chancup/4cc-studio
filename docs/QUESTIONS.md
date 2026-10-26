# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## In-game checks

- **Which automatic-substitution choices do PES 2017 to 2020 offer?** (`plans/save_editor.md`
  "Tactics editor", the Automatic settings row.) Measured: PES 2015, 2016 and 2021 store Off 0,
  Very late 1 and Flexible 2, and PES 2021 also Very early 3. The editor offers 2017 to 2020
  the first three until a look at each game's Game Plan, Automatic settings, Substitution says
  whether Very early is there (and under which stored value). A save's stored value outside
  the offered ones still shows, as its number.

## Cup practice

## Small confirmations

- **May the crash dump's hook use one `unsafe` block?** (worklog 0.0.1k4;
  `plans/core/architecture.md` "Diagnostic logging", the "crash dump" bullet.) The crash
  dump you accepted as P38 fits its condition: its crates add 221 KiB to a test build of the
  exe (ours should grow less), against the 10 MB allowed, and that build's stack overflow left
  a dump that reads correctly. But attaching the hook that catches the crash is an `unsafe`
  call in our own code (`crash-handler`'s `make_crash_event`: the closure runs inside the
  crashed process, so it must do almost nothing; ours only asks the monitor for the dump).
  The two ways: (a) yes, one `unsafe` block in `studio` with its `// SAFETY:` comment (about
  ten lines), and the dump is built as the plan describes; (b) no, and the crash dump waits:
  the other crate that hides the block (`minidumper-child`) does not fit the plan (an
  environment variable instead of an argument, no way to test a failed start, older crate
  versions). Nothing of the dump is built until you answer, since without the hook it writes
  nothing.

- **Which icon does each tool get?** (worklog 8.14.) Open `docs/prototypes/icons.html`,
  click one candidate per tool (the rail at the top shows the picks together), then copy the
  "Picks" box. The font build and the glyph constants wait on it. The Team compiler's is
  picked, (g); the other thirteen tools are open.
