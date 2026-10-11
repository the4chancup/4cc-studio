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
the fallback is packing gloves flat (a subfolder's files by name).

### A referee's `hair_parts` textures
From: the Red inventory, 2026-10-11 (step 5.0c, `.tmp/5_0c/bins.md` G1). When Red moves a
referee's textures into the refs Common folder, it copies a texture whose path holds
`hair_parts` and keeps the original beside the face (`referee_tools.py:222-229`, commit
`8d20bbb`, "to preserve originals", no reason given); `pipeline.md` step 6 moves every texture
of a folder's own models. If the game reads a `hair_parts` texture from the face folder by a
path the rewrite does not reach, a referee's hair shows untextured. To check: a referee with a
`hair_parts` texture, compiled by Studio, in Edit mode. If his hair is untextured, step 6 keeps
those textures in the face folder too.

### Non-power-of-two mipped textures and kits over 2048 on Fox
From: step 5.0e (2), 2026-10-11 (census `.tmp/5_0e/census/census.md`). Studio refuses, on
PES 18-21, a texture with a mip chain and one side not a power of two (`texture_not_pow2`), and
a kit main texture over 2048 pixels (`kit_texture_too_big`); Red accepted both. The census of
the machine's exports found 15 such mipped textures in 48 Fox export roots (VGL26's AKG, BAG,
EGG, HDG and SMBG in `Lab\Gud`; the MARISA Kanmari boots' 2048x2096 BC7 `body.dds`) and 11 kit
main textures at 4096 or 8192 (AKG VGL26 `u0786p1-3`, FGOG). To check on PES 21: a model whose
texture is one of these (EGG's 512x461, say) and a kit at 4096, compiled by Red, in Edit mode.
Both render as intended: Studio takes Red's rules (only both sides non-power-of-two refused;
the kit size limit kept for pre-Fox). Either does not: the rule stands, with what the game
does as its reason in `messages.md`.

## Cup practice and preferences

## Small confirmations
