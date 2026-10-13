# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## Blocking a plan ruling

### Which game versions the Match tracker registers first
From: an outside contribution, 2026-10-11 (match_tracker README "Independent measurements").
The plan registers PES 2017 only and keeps PES 2021 as unverified reference data. PES 2019
and PES 2021 now have measured layouts and catalogs, each checked against the game's pages
in two matches plus an out-of-sample match. Should
the first release register PES 2019 and PES 2021 profiles beside PES 2017 (their extra
time and the fork's pointer chains, substitution journal, direct clock and heatmaps
staying unregistered), or keep PES 2017 only? The "Game versions" row in the README's key
decisions changes with the answer.

## In-game checks

### Face diffs are engine-specific
From: maintainer, 2026-10-03 (worklog Issues). A `face_diff.bin` (or its `face_diff.xml` and
`<dif>` text forms) authored for one engine misplaces the face on the other, as the kit layout
did; the compiler passes a diff through for whatever target it compiles. Deferred until after
Phase 4's kits work. To investigate then: what differs (format, bones moved, rest pose),
whether one engine's diff converts into the other's, and how a folder says which engine its
diff is for (the kit layout's `pre-fox`/`fox` markers are the model). Say when.

### A shared gloves folder's subfolder paths on PES 15-17
From: S19 review, 2026-10-11 (DECISIONS "a combined shared folder's files stand at their paths
below the player folder"). A shared gloves folder's subfolder is packed at its path, so its
`glove.xml` names `./grip/glove_r.model` and a subfolder `.mtl` naming the folder's root
texture writes `../skin.dds`. Whether PES 15-17 loads both is unchecked: a gloves folder
with a model and `.mtl` in a subfolder, its texture at the root, seen in Edit mode. If not,
the fallback is packing gloves flat (a subfolder's files by name). Not runnable by the lead
yet (4.y-ingame2, 2026-10-12): a shared gloves folder compiles to the team's first shared
ID, which the game draws only on a player the save points at it, and writing that ID into a
pre-Fox save is the deferred pre-Fox patch's; a save of yours whose player wears that ID
(`/out/`: 1684) would let it run now.

## Cup practice and preferences

## Small confirmations
