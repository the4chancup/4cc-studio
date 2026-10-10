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
