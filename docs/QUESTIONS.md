# Open questions for the maintainer

The one list of what only the maintainer can answer: in-game behavior, cup practice, a
preference between options the plan does not pick. The lead adds an entry when it parks a
question (`AGENTS.md` "When the plan has gaps") and removes it when answered, logging the
answer where it belongs (a plan edit and a `DECISIONS.md` entry, or a worklog note). Lead and
sidekick work in progress stays in `WORKLOG.md` "Issues"; this file holds nothing the lead can
settle itself. Entries are grouped by what an answer unblocks, newest last within a group; each
names where the question came from so the context can be read there.

## Blocking a plan ruling

### A shared face folder's `face.xml`
From: step 4.20 (DECISIONS 2026-10-09). A `Faces/` folder's own `face.xml` is ignored on PES
15-17 with `xml_ignored_shared`, each linking player's face listing the folder's models by the
normal route, as a member's own xml is ignored on Fox. Should the shared xml rule every player
combining the face, and what would his own files add to it (entries appended, or his models
replacing the entries of their names)? A plan ruling; then the face task reads it as the
member's own and the deep pass checks it as one.

### FoxDen per-kit models
From: step 4.11c, maintainer's note of 2026-10-05. The plan assumes a tag-less `modelname`
fills every kit number up to the team's kit count that has no `modelname_kit<N>`, once any
variant exists (`model_format.md` "Kit-dependent assets", `kit_variant_model_fox`). The local
FoxDen (`02_kitswappers.lua`) swaps whole folders instead (`<id>p<kit>\#Win\`, kit 1 never
swapping, legacy `u0TTTp0` and 25-id blocks).
Needed: the file names FoxDen looks for (per model inside the face package, or per-kit
folders); how that meets Fox merging (one merged FMDL per kit?); whether the rule covers
textures and pre-Fox. Then the plan's rule and the finding change.

### Per-kit model sets on pre-Fox
From: step 4.16. Once pre-Fox kit numbers exist: is a per-kit *model* set (4.14e4) completed
against them as a texture set is (the lowest variant copied under a missing number), or left
as the member made it? Also unknown: what the game does with a `face.xml` entry whose
respelled model is missing (no pre-Fox exe reads `kitN` yet).

### Converted collars
From: step 4.17 (slice e's ruling, DECISIONS 2026-10-08 "Collars across engines"). Two parts:
(a) is a converted collar drawn right with its first material as `uni_collar` and the rest as
`uni_shirts`? The stock referee collar uses `uni_shirts` alone. An in-game check. (b) Which
version's stock `uniform.mtl` should the templates ship, so a `.model` collar can convert for
PES 18-21? Until then a `.model` collar on Fox is left out at planning with
`model_conversion_failed`.

### `dummy_kit` on the modded PES 15-17 exes
From: step 4.17a. Do the modded exes substitute `dummy_kit` at
`model/character/uniform/common/<team>/dummy_kit.dds` for a converted face model? 4.17a
points converted materials there, where the legacy pre-Fox exports name it. An in-game check.

### Hidden Fox meshes converted to pre-Fox
From: step 4.17c1 (measured on `legacy19to16_oral.mtl`). A `.mtl` cannot express `invisible`,
so a hidden Fox mesh (Konami's oral mesh in `addon_oral.fmdl`) shows on PES 17 after
conversion (Warning `mesh_flags_dropped`), and its `dummy_bsm` material resolves on neither
the Fox path the converter keeps nor the `./.dds` the legacy converter wrote. What do the
modded exes draw for such a material, and should a hidden mesh be dropped from the `.model`
instead of unhidden? A plan ruling, then a converter change.

## In-game checks

### Stock collar sets, and collars beyond the stock set
From: step 4.9b. Whether the shipped collar sets look right in game, and what a collar ID
beyond the stock set draws. Test 3 (`.tmp/4_0/apptest/out_test3/GUIDE.txt`) is built for it;
the lead ran its PES 17 half through the harness on 2026-10-09 and could see only the keeper:
collar 201 (no model anywhere) draws a plain neckline, no crash; kit 1's collar 200 was never
visible because every outfielder of /a/ is a billboard or custom-body model in Edit mode. The
PES 21 half, Test 3b and a match view are yours.

### The sock table's look
From: step 4.10. Needs a kit with a vertical design on its socks, compiled for PES 17 and 21
and compared in game. Test 3 holds that kit (design A on kit 1); see the collar entry above for
why the lead's Edit-mode run on PES 17 could not show it.

### Face diffs are engine-specific
From: maintainer, 2026-10-03 (worklog Issues). A `face_diff.bin` (or its `face_diff.xml` and
`<dif>` text forms) authored for one engine misplaces the face on the other, as the kit layout
did; the compiler passes a diff through for whatever target it compiles. Deferred until after
Phase 4's kits work. To investigate then: what differs (format, bones moved, rest pose),
whether one engine's diff converts into the other's, and how a folder says which engine its
diff is for (the kit layout's `pre-fox`/`fox` markers are the model). Say when.

## Cup practice and preferences

### Solid `.7z` exports over the memory budget
From: step 4.7 (worklog Issues; `libs/pipeline.md` "What a solid `.7z` is charged"). A solid
archive whose buffers exceed the budget is refused whole; keeping the buffers that fit and
letting go of the rest is not designed. Cups are compiled from folders, so: does this case
matter at all?

### Clef's first token
From: the lead's note of 2026-10-07. After the maintainer took Workers Paid, the full pass's
first request on token 1 (the maintainer's account) still got the "daily free allocation"
error (4006), so the pass ran on token 2. Is the paid plan on the first account, and active?
No code change either way.

### Commit retags
From: the 2026-10-05 retag of `review` commits to `fix`. Should any of them rather be
`refactor` or `test`?

## Small confirmations

### `just parity` and `just release`
`CONTRIBUTING.md` names both recipes; the justfile lacks them. Add the recipes, or drop the
mentions?

### Shorts not re-laid across engines
A decision entry reversed the plan's reading (the shorts keep their layout). Fine as is, or
reverse?

### `kit_config::validate`'s unreported findings
The compiler reports two of the lib's findings; `kit_collar_zero` and four more go
unreported. Report them in the compiler too, or leave them to the Kit config editor?
