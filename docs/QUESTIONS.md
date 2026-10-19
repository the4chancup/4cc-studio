# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## In-game checks

### Face diffs are engine-specific
From: maintainer, 2026-10-03 (worklog Issues). A `face_diff.bin` (or its `face_diff.xml` and
`<dif>` text forms) authored for one engine misplaces the face on the other, as the kit layout
did; the compiler passes a diff through for whatever target it compiles. Deferred until after
Phase 4's kits work. To investigate then: what differs (format, bones moved, rest pose),
whether one engine's diff converts into the other's, and how a folder says which engine its
diff is for (the kit layout's `pre-fox`/`fox` markers are the model).
Investigated offline 2026-10-15 (`.tmp/face_diff/findings.md`; 18,622 diffs in the installed
games' CPKs). The format and values are the same on both engines: 1,006 of the 1,094 faces
present in both PES 17 and PES 18 have byte-identical diffs, and no header byte tells the
engines apart. The real split is the diff's version: 4 (32 bone entries, 944 bytes) against 5
(33, 960), which Konami ships in every game. Konami converted five faces 4 to 5 mechanically
(one entry inserted at index 31, the count and offset table updated). The "autoconverted"
community default used by 547 cup faces is that conversion with the count left at 32 and the
inserted entry 1.4 km off. So the misplacement you saw may come from a version 4 diff, from
that half-converted file, from eye-hiding values, or from the face model rather than the
engine. Example given 2026-10-17: a member's anime face, `lapwing`, whose pre-Fox diff he
edited for Fox (`.tmp/face_diff/lapwing/findings.md`). His fix is two head floats, 0x2C and
0x30, from 35 to 45 (eye parameters, by the census), and cup members' Fox diffs hold the same
hand-set pair (45, 45, -10, -10) more than any other value. In PES 21's Edit mode those floats
change nothing visible, even at 0 and 90: they act on moving eyes. What did the member see
wrong with the 35 diff on Fox, and where (Edit mode, the eyes during a match, a cutscene
close-up)? With that, a run there with 35 and 45 on PES 21 and PES 17 tells whether one diff
serves both engines or a folder needs one per engine.

## Small confirmations
