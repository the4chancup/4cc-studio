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

### Which base-copied faces the game shows
From: step 5.4c's census, 2026-10-14 (`save_editor.md` "Stripped save", the base-copied face
bullet; `.tmp/5_4/base_copy_census.txt`). A Fox player record's base-copy id differs from the
player's own id on 0 to 26 players of each real PES 21 cup save (face edit flag set on nearly
all) and on all 5,037 players of the PES 21 fixture save (all copying player 7511, flag clear).
Strip and seed cannot carry the id (the database row has none), so it warns about such
players; which ones depends on two things only the game shows, in Edit mode on PES 21 (the
lead runs it through the harness once you allow it to take focus; saves backed up first):
(1) does a fixture-save player (base copy 7511, face flag clear) show player 7511's face or
his own; (2) after stripping a real cup save's base-copied player (flag set), which face does
he show. Unblocks: the warning's condition (id alone, or id and flag) and SE-STR-07's fixture,
step 5.4c2; strip and seed itself (5.4c1) goes ahead.

## Cup practice and preferences

### Fuzzy scorer matching for the feed-driven goalhorn
From: the fork survey, 2026-10-13 (`music_player.md` "Autopilot", score from the game). The
plan matches a `Goal` event's scorer to a player slot case-insensitively, standard horn on a
miss. The Rigdio fork uses `thefuzz.partial_ratio > 70` (best match wins), which suggests
its author met in-game names that differ from `.4ccm` slot names. How often do they differ
in practice (truncation, commas, accents, nicknames)? If often, should the player fall back
to a fuzzy match, accepting false positives on short names, or should a miss stay the
standard horn with a logged name so the manager fixes the export? Unblocks: the
scorer-matching rule in "Autopilot" before Phase 10's player crate.

### The stoppage chant's default
From: the fork survey, 2026-10-13 (`music_player.md` "Autopilot", stoppage chants, and
`autopilot_stoppage_chant_percent`). The plan ships the chance at 0 (chants stay manual).
The SEN:P-AI 1.3.0 rebuild ships 15 %, the ATF bot author's choice for his streams. Is 15 %
the cup's preference for an unattended match, or does a stream want chants only when the
streamer starts them? Unblocks: the setting's default; the rule itself is already in the
plan.

## Small confirmations
