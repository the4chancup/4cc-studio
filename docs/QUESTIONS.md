# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## Blocking a plan ruling

## In-game checks

### Face diffs are engine-specific
From: maintainer, 2026-10-03 (worklog Issues). A `face_diff.bin` (or its `face_diff.xml` and
`<dif>` text forms) authored for one engine misplaces the face on the other, as the kit layout
did; the compiler passes a diff through for whatever target it compiles. Deferred until after
Phase 4's kits work. To investigate then: what differs (format, bones moved, rest pose),
whether one engine's diff converts into the other's, and how a folder says which engine its
diff is for (the kit layout's `pre-fox`/`fox` markers are the model). Say when.

### Where the pre-Fox referee marker shows
From: step 4.19d's in-game check (DECISIONS 2026-10-09 "The pre-Fox marker goes by Red's
route"). The plan says the marker is a square about 1.5 m wide on the ground under the
referee. On PES 17 the lead's frames of the walkout, the lineup and the match's wide camera
(`.tmp/4_19/ingame/frames/`) showed neither the cup's own clover (the template's
`parts/referee/incom_bsm.dds`, installed in `4cc_35_referees.cpk`) nor a test texture. In
which scene does the pre-Fox marker show, and does it with the cup's current refs CPK? Run D
of `.tmp/4_19/ingame/test_ref04_runs.py` installs a magenta and yellow checker as that
texture through the test CPK slot (`install D`, then `revert`; PES closed for both), run G
the template's prop pair with it. The lead ran both on 2026-10-10 (worklog 4.y-ingame2):
no square in the walkout, the kickoff's wide camera or the replay's cameras at 0:00, and
no referee figure at all in an exhibition match on the maintainer's install (the cup's
`4cc_35_referees.cpk` holds the same prop pair and its clover texture). So two questions:
is the referee hidden on purpose on this install (and how), and which scene draws the
prop, if a visible referee is what it takes? The checker is a plain DDS where the
template's own file is WESYS-compressed, as the compiler's output is (step 4.19f), so a
run that shows it also confirms the game reads a plain DDS at that path.

### A 192x512 mipped texture, and `.model` mesh tags
The referee galosengen's `Common/scroll.dds` (Autumn Q 25 Day 1) is 192x512 with ten mip
levels: the plan's rule refuses a mipped texture whose side is not a power of two
(`texture_not_pow2`), and the census of 2026-10-09 drops his folder for it on both engines.
Does PES 17 draw that texture as Red shipped it? If it does, the rule is Fox's (FTEX) only.
Separately, the IR carries no `.model` mesh tags, so every conversion of a `.model` drops
them (`native_field_dropped (field=tags)`; the stock cap's tag is `Captainmark`): does the
pre-Fox game read a mesh tag, so that the IR needs the field (worklog 4.17 "Open for
converge")?

### A Fox cube map of Red's type
Red writes a member's cube-map DDS on PES 18-21 as an FTEX of texture type 0xD (the normal-map
type 0x9 with the cube bit), a type the game's own cube maps never use (theirs are 0x5 and
0x7, BC1, chains stopping at 4x4). The compiler writes the same since the decision of
2026-10-09 (worklog 4.y-fix3). Only referee exports carry one, a copy of the bundled template
`env.dds`, and only their pre-Fox `.mtl` materials name it, so nothing on PES 18-21 draws it
today. Does an FTEX cube map of type 0xD render when a Fox material names it (a Fox FMDL
pointed at a converted `env.ftex`, in Edit mode)? If it does not, the Fox form becomes a
check-time finding on the file instead of Red's output.

## Cup practice and preferences

## Small confirmations
