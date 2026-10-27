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

## Draft looks

- **The drafts in the app's own fonts (0.0.1w3).** Their approval is withdrawn until you have
  looked at them again: they now draw Selawik and the suite's symbols font (worklog 0.0.1w,
  DECISIONS 2026-10-27 "The symbols are a font of their own"), where every earlier review saw
  Segoe UI. Open `docs/prototypes/index.html`; the app's code follows the drafts once approved.
  Alternatives to Selawik (0.0.1w4) come after, side by side.
- **Track D's drafts, 8.30, 8.32, 8.33 and 8.34**, revised with your answers of 2026-10-27
  (DECISIONS "Formation pictures, the formation's name and the nation names: the maintainer's
  answers"): branch `side/drafts2`, each step's renders and proposed plan text in
  `C:/Data/4cc/Tools_Mine/4cc-studio-temp/wt-drafts2/.tmp/<step>/look.md`. Approve or change
  each; the plan text lands on approval. 8.32 (0.0.1x) is the exception: it is on main
  with the rule the in-game captures gave (LMF and RMF count with the AMF, not the CMF), its
  plan text written (`plans/save_editor.md` "The formation's name"); look at the pitch's
  bottom-left corner in the Save editor's Tactics tab of `docs/prototypes/index.html`.
- **The Tactics pitch's drop on another player (0.0.1y)**, as the game's drag now: the two
  swap places in the shown formation alone, while dragging an XI row onto another swaps the
  two players in every preset, as the game's select-one-then-the-other does (DECISIONS
  2026-10-27 "A pitch player dropped on another swaps the two slots in the shown formation, as
  the game's drag"). The Save editor's Tactics tab in `docs/prototypes/index.html`.

## Cup practice

## Small confirmations

- **The changes list (8.31): a popup or a window?** (worklog 8.31; the draft and its notes:
  branch `side/drafts2`, `C:/Data/4cc/Tools_Mine/4cc-studio-temp/wt-drafts2/.tmp/8_31/look.md`,
  renders `8_31_list.png`, `8_31_undone.png`.) A click on the toolbar's `• unsaved changes`
  opens the close prompt's table under it. Drafted as a popup that any click outside, Esc or
  an undo closes; the alternative is a window that stays open while you undo. And should a
  colour row draw a swatch before its hex, as the team view does (the rows are strings today;
  a swatch needs a typed value per row)?
- **The rules load error on two lines?** (Save editor, the red error under the player list,
  s3h; `.tmp/report.md` 2026-10-26 16:40.) Your wording takes three lines at 240 points in the
  app (the draft shows two: the browser's text was narrower). Keep it, or shorten it to fit
  two, e.g. "The AATF rules file could not be loaded, click here for details."?
- **Which icon does each tool get?** (worklog 8.14.) Open `docs/prototypes/icons.html`,
  click one candidate per tool (the rail at the top shows the picks together), then copy the
  "Picks" box. The font build and the glyph constants wait on it. The Team compiler's is
  picked, (g); the other thirteen tools are open.
