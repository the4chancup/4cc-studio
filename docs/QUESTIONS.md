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

- **The sliders, the player rows, the positions labels and Reset window (0.0.1z1).** Open
  `docs/prototypes/index.html`, the Save editor: the sliders are drawn as the app's, the
  handle and the rail up to it blue (a team's Formation settings, a player's Appearance);
  the player rows carry no team name under ALL; a click on a Playable positions label makes
  it the registered position; the list keeps its scroll through a reorder or a folded
  section; Reset window sits right of the version in the status bar. Your second look's
  changes (0.0.1z4): Reset window is drawn only while the window is maximized or off its
  default size; the unsaved-changes prompt is as wide as its widest row, up to nine tenths of
  the window, each change on one line; every draft's controls sit in one column left of the
  window, in sections (Window, Shell, the active tool). The app follows once you approve.
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
  8.31 (the changes list) is revised with your three answers of 2026-10-27 (a popup, the
  12-point swatch, no line under the table; `side/drafts2` `30842fa2`, renders
  `wt-drafts2/.tmp/8_31/8_31_{popup,prompt}_answered.png`): approve it or change it like the
  others.
- **The Tactics pitch's drop on another player (0.0.1y)**, as the game's drag now: the two
  swap places in the shown formation alone, while dragging an XI row onto another swaps the
  two players in every preset, as the game's select-one-then-the-other does (DECISIONS
  2026-10-27 "A pitch player dropped on another swaps the two slots in the shown formation, as
  the game's drag"). The Save editor's Tactics tab in `docs/prototypes/index.html`.

## Cup practice

The Music player's three below come from reading Rigdio v2.2.0's controls (worklog 10.0i,
`plans/music_player.md` "Playback semantics"):

- **Chaoshorn and the score.** Rigdio's Chaoshorn presses every goalhorn button, so it scores
  one goal for each player and one for the Standard Goalhorn: the scoreboard jumps by the
  roster size plus one, and each player's horn is chosen as for that goal. Should Studio's
  Chaoshorn score those goals, or play each button's horn as if for a goal and leave the score
  as it was? (Is Chaoshorn pressed while the score still matters?)
- **A button after its song ends.** When a horn with `end stop`, or a victory anthem, plays to
  its end, Rigdio's button stays pressed, and the next press only releases it (no goal, no
  sound); the press after scores and plays. Do streamers rely on that extra press, so that a
  button releasing itself at the song's end would turn their habitual double press into two
  goals? Or should the button release itself when the song ends, so the next press is a goal?
- **title.log and chants.** Rigdio writes title.log for anthems, victory anthems and
  goalhorns, never for a chant. Should the OBS overlay show a chant's title too, or stay on
  horns and anthems as it always has?

The Refs arranger's three below come from its readiness audit (worklog 13.0a,
`.tmp/13_0a/refs_audit.md`); none blocks the tool's first slice:

- **FoxDen without a lists file.** FoxDen's referee module, as shipped, forces five distinct
  referees drawn uniformly from all 35 slots at boot and after every match
  (`Tools_4cc/FoxDen/scripts/03_refmod.lua`, `randomize`), so a FoxDen player never sees the
  game's own draw. Is FoxDen expected on every viewer's install, or on the streamers' only?
  And with no `ref_lists.txt`, should the reading script keep that uniform draw, or turn
  forcing off so the game draws over the arranger's weighted fill (`plans/refs_arranger.md`
  "Who gets the file")? The second is a change the hook's author would make.
- **Which game mode do Fox cup matches run in?** FoxDen's reading of PES 2018 has Exhibition
  modes draw five referees uniformly from slots 1–10, 16–25 and 31–35, and competition, league
  and Master League modes from league-driven pools (`plans/refs_arranger.md` "Background").
  Also: which version(s) were the 125 matches of the PES 17–21 table logged on, and what is
  slot 24's real count (marked uncertain)?
- **Which PES version was the 15/16 pattern table measured on?** And is there a log from the
  other one, to say whether the two share it?

## Small confirmations

- **Which icon does each tool get?** (worklog 8.14.) Open `docs/prototypes/icons.html`,
  click one candidate per tool (the rail at the top shows the picks together), then copy the
  "Picks" box. The font build and the glyph constants wait on it. The Team compiler's is
  picked, (g); the other thirteen tools are open.
