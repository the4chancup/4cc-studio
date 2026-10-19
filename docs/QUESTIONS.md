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
change nothing visible, even at 0 and 90: they act on moving eyes.
Settled 2026-10-19 by the maintainer's second example, `Sopdog` (`C:\Data\4cc\Lab\sopdog`: a
Fox VGL22 version and a pre-Fox VGL24 one hacked from it, with the face-animation author's
explanation in `chat.txt`): the engines animate eyes differently, so the face model and its
diff are made for one engine. Pre-Fox blinks with the lower eyelid too and assumes the model's
eyes half closed, so its diff says how far to open and close them; Fox assumes them fully open
and only closes them, and rotates the whole eye, eyelids included, where pre-Fox rotates the
eyeball alone, which authors compensate for in the model (eyes pulled deeper into the head,
eyelids weighted to the head). Sopdog's two diffs agree: the same version-5 layout, the Fox one
with 60 at 0x2C/0x30 and almost no bone offsets, the pre-Fox one with 35 there and offsets on
about 20 eye, eyelid and brow entries; the backup beside the pre-Fox diff is the Fox values,
the author's starting point (lapwing: 35 pre-Fox, 45 Fox, the same split). So one diff cannot
serve both engines, and no rule converts one into the other: the model changes too. A folder
can already carry both engines' faces (the `.model` with a `face.xml` `<dif>` for PES 15-17,
the `.fmdl` with `face_diff.bin` for 18-21, each target taking its own, as the referee exports
do; `face_diff.xml` beside `face_diff.bin` is `xml_dif_conflict`), but a face made for one engine only is compiled for the
other by converting its models and passing its diff through, with nothing reported, and the
eyes misbehave. What should the compiler do with such a face: (a) say nothing, as today;
(b) report it (one Info or Warning per face, naming the engine it was made for), knowing
that once the cup moves to Fox in 2027 every pre-Fox face compiled there would carry it; or
(c) something else, such as compiling it but leaving the diff out?

## Cup practice

### A Common path spelled with the team's name
From: the 6.12c library run (`.tmp/6_12c/findings.md` D4). Two teams' old faces name their
Common files with the team's name where the three-character ID goes:
`model/character/uniform/common/o/env.dds`, `.../common/vg/oral_shirt_*.model` (/o/ 2017-2024
and VG 2017-2018 exports, 16 in the library). No compiler ever resolved that spelling: every
one on this machine writes `Common/` under the team's ID, the 2016 batch compiler and Red alike,
so from 2017 to 2023 those faces reached the game with that texture or model missing, and since
Red 3.x (2024) its check discards the whole face folder ("Files loaded from the Common folder
must use a path with a subfolder whose name is three characters long"). The Studio compiler
refuses it too (`xml_common_path_invalid` in a `face.xml`; 6.16b gives a `.mtl` the same Error),
so today these upgrades end in a compile Error. The VG exports ship the named files in their
`Common/` (`env.dds`, `fbm_s.dds`); the /o/ 2024 ones ship no `Common/` at all. Should the
upgrade (a) read the team's name there as the team's Common, so these faces compile and load
the Common files they name (which they never did in game), (b) drop such a face folder as a
leftover with a Warning, as the cups have seen them since 2024, or (c) refuse the export, the
member fixing the paths by hand?

## Small confirmations
