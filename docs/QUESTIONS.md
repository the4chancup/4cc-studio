# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## Blocking a plan ruling

### An inbound control API for the music player
From: the fork survey, 2026-10-13 (`music_player.md` "Autopilot"). The Rigdio fork exposes a
local HTTP API (load, anthems, goal, victory, chant, event) that the ATF bot ("rigbot")
drives for the parts no memory cue covers (anthems, the victory anthem); the tracker's
`watch` CLI is output only and does not serve that consumer. Should the Studio player expose
such an inbound local API (the fork's endpoint table as the starting contract, `side` only,
no busy-409), or does the bot become a Studio tool and call the player in-process, which
dissolves the question? Unblocks: whether Phase 10 plans a control surface at all, and the
`match_tracker/README.md` "External consumers" wording.

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
engine. Which face did you see misplaced, and on which game? With that, one Edit-mode run
on PES 17 and PES 21 settles it: the version 4 default, it converted, the half-converted file
with its count fixed, and the version 5 default.

## Cup practice and preferences

## Small confirmations
