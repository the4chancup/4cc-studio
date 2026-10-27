# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## In-game checks

- **Which automatic-substitution choices do PES 2018 to 2020 offer?** (`plans/save_editor.md`
  "Tactics editor", the Automatic settings row.) Measured: PES 2015, 2016 and 2021 store Off 0,
  Very late 1 and Flexible 2, and PES 2021 also Very early 3; PES 2017's screen lists the four
  in PES 2021's order (the maintainer, 2026-10-27), so it offers them. The editor offers 2018
  to 2020 the first three until a look at each game's Game Plan, Automatic settings,
  Substitution says whether Very early is there. A save's stored value outside the offered
  ones still shows, as its number.

## Cup practice

## Small confirmations

- **Which icon does each tool get?** (worklog 8.14.) Open `docs/prototypes/icons.html`,
  click one candidate per tool (the rail at the top shows the picks together), then copy the
  "Picks" box. The font build and the glyph constants wait on it. The Team compiler's is
  picked, (g); the other thirteen tools are open.
