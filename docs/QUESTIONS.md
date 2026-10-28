# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## Before the release

- **The final pass over the built app's UI** (worklog 0.0.1f1c; the maintainer asked for it on
  2026-10-28), under way since 2026-10-28 ~14:00. Ruled so far: DECISIONS "The final pass's
  first rulings" and "second rulings", and "The wheel over a combo box or a drag value steps
  its value". Drafted from the second rulings and waiting for a look, each image with its
  reason: `.tmp/0_0_1ov/vet/list_aligned.png` (the controls above and below the list ending at
  its frame; the roster places 01 to 23), `vet/a_rating.png` (the chosen A in `#129648`, its
  letter white), and the Live AATF gallery `.tmp/0_0_1ov/live/` (07 the list's orange and
  blue check marks, 08 and 10 every field mark left of its label, 12 the suggestions' blue
  marks, 13 the dock's Suggestions group; the earlier set kept in `live_before/`); the wheel
  stepping a combo or a number box can be tried in the drafts themselves. Still to look at:
  the windows' titles at 14 in white in the built app; the preview's icon where the lead
  cannot capture it (worklog 0.0.1v: the taskbar button and Alt+Tab show the tight crop of the
  head; Explorer's file icon on the exe). Then the confirmation that freezes logic version 1's
  hash (0.0.1f2).
- **The rulesets' descriptions** (worklog 0.0.1ov7): the ruleset dropdown names each embedded
  file by a description alone, such as `/vg/ League 26`. Which description does each take,
  and which files ship in the dropdown? The files today, by their `name`: Autumn 26, Spring
  26, VGL27, VGL26, VTL11, VTL10, DBC7 (and the test rulesets Autumn 25, VGL25). The docs
  name VGL the /vg/ League, VTL the /vt/ League and DBC Divegrass Before Christmas; the 4cc's
  own name for its seasons ("4chan Cup Autumn 26"?) is not written anywhere.

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
