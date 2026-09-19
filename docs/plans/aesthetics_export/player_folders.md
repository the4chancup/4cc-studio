# 4cc Studio — Aesthetics export plan: Player folders

Part of the [Aesthetics export plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Player folders (the Studio export format)

**This is a primary motivation for the project.** The export format itself is new: it is called the
**Studio export format** and was designed for 4cc Studio — no existing tool produces or consumes it
yet. A "player folder" holds all of a player's models —
face, boots, and gloves — in a single folder, without the old split into separate Faces/Boots/Gloves
item folders. (Red's referee export layout was the prototype of this idea, tested only on referees;
the unified format keeps its per-referee face/boots/gloves/common subfolders as optional **reserved
subfolders** — see below — and is used by team and referee exports alike — see "Referee export
processing" in the [Team compiler plan](../team_compiler/README.md).)

```
Players/
├── 15 - Snuffy/
│   ├── face_high.fmdl        (or .model, or .glb — see the Unified model format plan)
│   ├── hair_high.fmdl
│   ├── oral.fmdl
│   ├── boots.fmdl            (player-exclusive boots)
│   ├── Crocs.boots           (link file: references Boots/Crocs/, shared with other players)
│   ├── glove_l.fmdl
│   ├── glove_r.fmdl
│   ├── *.dds / *.ftex / *.png / *.jpg / *.bmp / *.webp / *.tga / *.tiff
│   │                         (textures: any image format, referenced by stem; interchangeable)
│   ├── materials.toml        (glTF models only: PES material properties keyed by material name;
│   │                          the catch-all applies to all glTFs in the folder. Optional
│   │                          name-matched *.materials.toml files (e.g. body.materials.toml)
│   │                          override per-model, mirroring the .model/.mtl system; a
│   │                          *.materials.toml.common or *.mtl.common link pulls the file of
│   │                          that name from Common/ — see the Unified model format plan)
│   ├── ingame_face           (optional empty marker: remove custom face parts, keep other categories)
│   ├── fpc.on                (optional empty marker: apply the FPC preset at compile time; see `fpc_toggle.md` "FPC toggle")
│   ├── settings.toml         (savefile settings; see below)
│   ├── face/  boots/  gloves/  common/
│   │                         (optional reserved subfolders: their files are that category's parts
│   │                          wholesale, common/ holds textures; see "Reserved subfolders")
│   └── ...
└── 16 - Another Player/
    └── ...
```

**Player numbering** — a normal team player folder gets its number(s) in one of two ways:

1. **In the folder name**: `NN - Name` with NN in 01–23, as above.
2. **Via a root `players.txt`**: lines of `NN <folder name>`; the listed folders then carry no
   number in their names. Sparse lists are allowed, and the same folder may be listed under multiple
   team slots. When `players.txt` is present it is authoritative: every player folder must be listed
   in it, and folder-name numbers are not used.

**Savefile identity.** Normal 4cc team rosters and player IDs are aligned: slot `NN` belongs to
player ID `team_id * 100 + NN`. Resolve that record by ID, not by name or physical file offset;
model/portrait destinations and savefile edits use the same identity. For team 701, slot 03 targets
70103 even if its savefile name is generic. A folder mapped to several slots applies to each slot's
corresponding ID. Names may be written through the opt-in `name` setting, but never select the
record; a missing record is not replaced by a name-based match.

**Multi-mapped processing.** For team and referee identities alike, shared preparation (parse,
convert, and merge) runs once per distinct source player folder; the results are then rendered and
packed per mapped slot (IDs, paths, portraits, savefile entries), while the folder's relocated
textures are emitted once into their name-keyed common subfolder shared by all mapped slots (see
"Texture relocation to common" in the [Team compiler plan](../team_compiler/README.md)). Both stages commit as one atomic unit, so a partial set of slot
instances is never written. Referees use this same mechanism with their `refereeXXX`/`k99XX`/`g99XX`
rendering.

**Referee exception:** a refs export requires root `players.txt`; numbered referee folder names are
not a supported substitute. Its valid slots are 01–35, and repeating a folder under several slots is
how the export controls referee rarity. The same filename serves both identities, and the export
name's first word being `refs` is the referee discriminant. For legacy Red-era referee exports
only, a root **`refs.txt`** is accepted as a read-only alias of `players.txt` (identical grammar);
when both are present `players.txt` is authoritative and the alias is ignored with `refs_txt_ignored`.
Studio never writes the alias: the Refs arranger and the Export upgrader emit `players.txt` and
remove a `refs.txt` they read from. A refs export may also carry a root **`ref_lists.txt`** — the
referee lists the Fox referee hook applies per match (grammar and purpose in the [Refs arranger
plan](../refs_arranger.md), "The lists file"). It is the arranger's file: the export model lists it
as a known root file so validation does not warn about it, and the compiler neither reads nor
packs it — it travels to the hook's folder beside the compiled CPK by hand.

**`players.txt` grammar.** Input is strict UTF-8 with an optional UTF-8 BOM; LF and CRLF are
accepted, blank lines are ignored, and comments are not supported. Every nonblank line starts with a
decimal slot followed by whitespace; the trimmed remainder of that line is the complete folder name.
Folder lookup is case-insensitive, but case/Unicode-colliding player folder names are rejected
rather than selected arbitrarily. Writers emit two-digit slots as canonical UTF-8/LF with one
trailing newline. The Refs arranger and Export upgrader read and write this same grammar. A referee
roster must retain at least one valid normalized assignment; an empty file or a file whose entries
all drop reports `players_txt_invalid` and drops the export. An empty authoritative normal-team
roster is allowed for a kit-only export; any present player folders are then unlisted and follow
`player_unlisted`.

**`ingame_face` marker.** Bare `ingame_face` and the tolerated Notepad form `ingame_face.txt` are
recognized before extension allowlist checks, represented as `PlayerFolder.ingame_face`, and never
reported as disallowed files. Parsing normalizes either spelling to the same boolean; generated or
upgraded exports write the bare marker. At categorization it removes face-classified parts and
face-specific ancillary files, and **no face folder is emitted** (on either engine — a face folder
containing only boots/gloves would override the ingame face, and an empty one would blank it).

The face-classified parts split into two groups with different fates:

- **Explicitly-named face models** (`face_high`, `hair_high`, `oral`) have no skeleton slot and
  belong only in a face folder — there is none under `ingame_face`, and `fcl_hair` cannot absorb
  them (it is a single body-skeleton model, not a container for the typed face set). Their presence
  combined with `ingame_face` is a hard error that drops the whole player folder
  (`ingame_face_explicit_face_model`).
- **Arbitrarily-named models** (those that would route to `fcl_hair`, which — like `boots` —
  supports the full body skeleton) **reroute to the boots folder** instead of being dropped:
  renamed to `boots`, or merged into one `boots` if several candidates are present (Fox: `fmdl`
  mesh merging; pre-Fox: `pes_model`'s native merge over the `.model` + `.mtl` pair), and carrying their paired SKL into
  the `boots.skl` slot (see "SKL pairing"). This generalizes the existing ingame_face boots
  relocation to the body-skeleton face content that would otherwise be lost.

Remaining gloves parts are relocated to **player-specific gloves folders** (`glove_l`, `glove_r`),
and the rerouted arbitrary-named models plus any explicit boots models form the **player-specific
boots folder** — the one case where pre-Fox produces player-exclusive boots/gloves folders with IDs
from the per-team block scheme and savefile writes. Without `ingame_face`, a player with boots/gloves
but no face models keeps a blank face folder (the FPC requirement — see the pipeline walkthrough).

**`ingame_face` with shared links.** A boots/gloves link with no local parts of that category
resolves as usual: on either engine the player simply wears the shared folder's ID, since nothing
needs a face folder. A link **combined with local parts** of the same category is where pre-Fox
must deviate from its no-merge rule: normally the shared folder loads by ID while the local parts
ride in the face XML, but under `ingame_face` there is no face XML, and relocating the local parts
to a player-exclusive folder would leave the player with two candidate IDs (the shared folder's and
the exclusive one) for a single savefile slot. Pre-Fox therefore behaves like Fox's `link_combined`
here: the linked shared model is one more input of the player-exclusive merge (`pes_model`'s
native merge, like the multiple-boots case above), the exclusive ID wins and is written to
the savefile, and the shared folder is left untouched for players who link it plainly. Gloves follow
the same rule per side (`glove_l` with `glove_l`, `glove_r` with `glove_r`). A **face** link under
`ingame_face` is contradictory (the marker suppresses the face folder the link would fill) and drops
the player folder with `ingame_face_explicit_face_model`, exactly like explicitly-named local face
models.

**Reserved subfolders.** Inside a player folder, the subfolder names `face`, `boots`, `gloves` and
`common` are reserved (case-insensitive). This is the layout of Red's referee exports — the
prototype of the player-folder format — kept in the Studio format as **legacy support**, so that
current-day referee exports (and Red Pre-Studio aesthetics exports using the same subfolders) compile as
they are; it costs little. The flat player folder is the canonical layout — it lists a player's
contents more explicitly — so Studio-generated exports never use the subfolders and the Export
upgrader flattens them (see "Referee export processing" in the [Team compiler plan](../team_compiler/blue_port.md) and the Export upgrader plan). Any other
subfolder is `file_type_disallowed` for its files, as today. Red implements the same reservations
(its `pre_studio_plan.md`, "Reserved subfolders"); the two differ only in conflict handling, noted
below.

- A `face/`, `boots/` or `gloves/` subfolder's files are **that category's parts wholesale**: they
  skip filename categorization (a `hair_high.fmdl` inside `boots/` is a boots part), otherwise they
  are ordinary local parts — they take part in the same allowed-name resolution, SKL pairing, texture
  stem resolution (stems resolve across the player folder including its reserved subfolders) and
  merging as loose root files of the same category. Their textures follow the normal texture
  relocation to the per-player common output.
- `common/` holds textures only: they are texture sources like any other image in the player folder,
  and are emitted into the per-player common output as they are. Model files in `common/` are
  `file_type_disallowed`.
- **Combination follows the plan-wide rule, not Red's.** Red treats a labelled subfolder combined
  with a same-category link file, or with loose same-category root files, as a conflict that drops
  the player folder (it cannot merge). Studio can, so a subfolder's parts **combine** with a
  same-category link (`link_combined`, the shared folder as the base) and with loose root files of
  the same category, exactly as two loose files would — there is no subfolder-versus-file conflict.
  The one contradiction that remains is `ingame_face` plus a `face/` subfolder: the subfolder's
  files are face parts and follow the marker's rules (explicitly-named face models drop the folder
  with `ingame_face_explicit_face_model`; arbitrary-named models reroute to boots).
- An **empty `face/`** counts as face content: the player gets a blank face folder, as a player with
  no face models does (the FPC requirement — see the pipeline walkthrough); under `ingame_face` an
  empty `face/` is simply ignored (nothing to contradict). Empty `boots/`, `gloves/` and `common/`
  are ignored.
- Root normalization never treats a reserved subfolder as a nested export root, and `Players/`
  detection is unaffected: the reserved names are matched one level below a player folder only.

**Shared models** live in Faces/Boots/Gloves folders identified by **name only** (no embedded IDs —
player folders are the main reference point for each player). A player folder references a shared
folder with an empty link file named after it (`Crocs.boots`, `Longhair.face`, `Keeper
gloves.gloves`). A stray `.txt` suffix (`Crocs.boots.txt`) is accepted silently — Windows hides
known extensions by default, so users creating link files with Notepad often produce one. At most
one shared link per category (face, boots, and gloves) may appear in a player folder; multiple links
of any same kind are rejected. Multiple shared-face bases are ambiguous pre-Fox because copied trees
can collide, and unnecessary on Fox because one shared base can already merge with all local parts.
Fox merging therefore combines that one shared base with local parts, never several same-kind shared
links:

```
Boots/
└── Crocs/                    (shared boots, referenced by two+ players via link files)
    ├── boots.fmdl
    └── *.dds
```

**A link plus local models combines** — for every category, so there is no link-versus-model
conflict anywhere. The shared folder acts as a **reusable base** (a body, a head, a boot shape) and
the player's local models are parts layered over it. How that is realized differs by engine, because
pre-Fox is far more permissive:

- **Pre-Fox**: everything in a player folder that isn't a boots or gloves link file becomes part of
  that player's **face folder**. The `face.xml` has a per-entry model *type* property, so boots and
  gloves models load into their own skeletons from the face folder — meaning per-player boots/gloves
  folders are never needed, and nothing is ever merged. A shared face folder is copied per player
  and receives the local files on top; a boots/gloves link keeps loading its shared folder by ID
  while the player's local parts load from the face folder alongside it.
- **Fox**: there is no model type property, so per-player boots and gloves folders *are* required.
  Local boots/gloves models form a **new player-exclusive folder** with its own ID from the team's
  block, and a shared model referenced by a link becomes just another component of it, merged in
  (see below). The shared folder is left untouched for the players that link it plainly. A shared
  **face** link with no local face parts is the same mechanism one part deep: the shared face model
  becomes the player's face FMDL outright (a merge of one), since shared face folders take no ID and
  have no independent output to be left untouched.

Combining is reported as `link_combined`. Consequences for Fox ID assignment: a player who combines
gets their deterministic player-exclusive ID rather than the shared folder's, and a shared
boots/gloves folder that every referencing player combines is only a source of parts — it needs no
folder of its own in the output and consumes no ID from the scarce shared pool (it is still
"referenced", so not `shared_folder_orphaned`). Pre-Fox only ever assigns IDs to shared boots/gloves
folders, since player-exclusive ones don't exist there — except under `ingame_face`, where the
relocated gloves/boots parts form player-exclusive folders with their own IDs (see "ingame_face
marker").

**Common model links and model merging** — a model can be loaded from the export's `Common` folder
instead of shipping in the player folder: an empty link file named after the Common model plus a
`.common` extension (a stray `.txt` is accepted, as with shared-folder links) — Red's pre-Fox
`.model.common` link convention, extended to the Studio format's model file names. On pre-Fox targets
the link resolves to a real runtime reference: the generated XML points at the Common path (on PES16
via the patched exe — see the note under the XML/MTL checks). Fox engines cannot load models from
Common at all, so there the compiler **bakes the link away: the Common model's meshes are merged
into the player's output FMDL** at compile time via the `fmdl` crate.

The same link convention applies to **material definition files**: `body.mtl.common` and
`body.materials.toml.common` pull `Common/body.mtl` / `Common/body.materials.toml` into the player
folder's material resolution as if they were local files of that stem (Red's `find_mtl_file`
already resolves `.mtl.common` names). A Common-linked model needs no such link — its material files
resolve in Common first, with the player folder's matching files layered on top as overrides. And
to **textures**: `hair.png.common` stands in for `Common/hair.png` under the stem `hair`, for
auto-detection and explicit stems in material files and for native `.mtl`/FMDL references alike —
the way to share one texture among many players. A texture that resolves into Common is packed once
in the team's Common output and referenced there; one that resolves in the player folder is the
player's own and travels with it. Rules, same-stem layering, texture-stem resolution and the
packing rule are in the [Unified model format plan](../model_format.md)'s "Link files"; the pre-Fox
XML points at the Common MTL path when the MTL was found in or linked from Common, as Red does.

Merging is driven by name resolution. Red's prefix convention (a prefixed allowed name is renamed to
it: `kit_boots.fmdl` → `boots.fmdl`) extends from pure renaming to multi-part assembly: **all models
— local files, shared-folder parts, `.common` links, or a mix — that resolve to the same allowed
output name are merged into that one model**, in alphabetical source-name order (deterministic
recompiles). `torso_fcl_hair.fmdl` + `legs_fcl_hair.fmdl.common` → one `fcl_hair.fmdl`; a `.common`
link with no local counterpart bakes the shared model in unchanged — the Fox substitute for pre-Fox
common-model sharing. Parts may be in any supported source format (converted to FMDL first, then
merged via the `fmdl` crate's mesh merging).

**Merging is Fox-only.** Fox model sets are closed — a face is a fixed list of FMDL names, and boots
and gloves are separate ID'd folders with one model each (`boots`, `glove_l`, `glove_r`) — so every
extra part has to be merged into one of those. Pre-Fox needs none of it: the typed `face.xml`
absorbs any number of entries of any type, and `.common` links stay runtime references. **Exception
— `ingame_face`**: with no face folder emitted, non-face parts are relocated to player-specific
folders; multiple boots models are merged into one (Fox: `fmdl` mesh merging; pre-Fox:
`pes_model::ops::merge`, the native counterpart with the same rules over the `.model` + `.mtl`
pair — see "`pes_model::ops::merge`" in the Libraries plan — the one pre-Fox merge case), and
gloves go to `glove_l`/`glove_r` folders (see "ingame_face marker").

**Model names: a free part plus a suffix.** A model file name is `<anything>_<suffix>` (or just
`<suffix>`), and the **suffix is always at the end** — one rule for every source format and both
engines. The suffix says what the model is; the compiler derives the Fox destination and the
pre-Fox `face.xml` type from it through one table:

| Suffix | Fox destination | Pre-Fox `face.xml` type |
|---|---|---|
| `face_high`, `hair_high`, `oral`, `fcl_hair` | that FMDL (merge if several) | `face_neck` for `face_high` (Red's rule); `parts` otherwise |
| `boots` | the `boots` folder (`boots.fmdl`, merge if several) | `parts` (boots use the body skeleton; today's typing) |
| `glove_l` / `gloveL` | the gloves folder, `glove_l.fmdl` | `gloveL` |
| `glove_r` / `gloveR` | the gloves folder, `glove_r.fmdl` | `gloveR` |
| `handL` / `handR` | the gloves folder (`glove_l`/`glove_r` — hand-skeleton models have nowhere else to go on Fox) | `handL` / `handR` |
| `uniform`, `shirt`, `pants_nocloth`, `eye`, `mouth`, `face_neck`, `parts` | face: the `fcl_hair.fmdl` merge | as named (`uniform` → `uniform_sub` on PES15, Red's rule) |
| `model_type_<x>` | face: the `fcl_hair.fmdl` merge | `<x>` verbatim — the escape hatch for a type this table does not know |
| *(none)* | face: the `fcl_hair.fmdl` merge (`fmdl_fcl_hair_fallback` reports each routed file) | `parts` (Red's default) |

The first column's Fox allowed names and the pre-Fox type names are both native vocabularies, so
both are accepted; `glove_l`/`gloveL` and `glove_r`/`gloveR` are aliases of each other. Matching is
case-insensitive and ignores underscores inside the suffix, as Red's typing did. A `_ratio_<n>` token
anywhere in the free part still sets the `face.xml` entry's `ratio` attribute (Red's convention).
Pre-Fox entries additionally get the `oral_`/`_win32` affixes on the emitted file name (a PES 16
loading requirement). The type column applies **only when targeting pre-Fox and only when the
compiler generates the `face.xml`**: a folder that ships its own xml carries the types there, and its
file names are not interpreted for typing. A user-written `face.xml` is accepted as a second-class
path — the way to experiment with what the pre-Fox engines can do beyond this table — and is checked
by the compiler with errors for what is known to break and warnings for everything outside the
vocabulary it generates itself ("User-supplied `face.xml`" in the [Team compiler
plan](../team_compiler/README.md)). Fox has nothing resembling designable model types, so on
Fox a type maps to a destination and nothing more. glTF
models follow the same table — the type lives in the file name, not in the glTF or the material
toml, so a `.glb` authored once compiles typed on pre-Fox and merged on Fox.

**This inverts Red's typing, which matched types as prefixes** (`gloveL_foo.model`); the Studio
format matches them as suffixes (`foo_gloveL.model`), the same position the Fox allowed names already
occupied (`kit_boots.fmdl`), so there is one place to look. The Export upgrader swaps them (see its
plan). "Arbitrary model names are face content" is the last row: the generalization of Red's
fcl_hair fallback (which renamed a single arbitrary-named face FMDL to `fcl_hair.fmdl`) — a model
with no recognized suffix is a face part on both engines, which is what makes the plain-named case
work with no naming ceremony: `torso.fmdl` + `legs.fmdl.common` → one `fcl_hair.fmdl` on Fox, two
`parts` entries pre-Fox. Boots and gloves models must therefore *say so* by suffix — in a player
folder because an unsuffixed model would be taken for face content, and in a shared boots/gloves
folder, where nothing can be a face, unsuffixed names are `fmdl_name_invalid` errors.

**SKL pairing** — a model may carry a custom skeleton. Each source format keeps it differently: an
`.fmdl` in a companion `.skl` file named after the model's source basename (`commander.skl` for
`commander.fmdl`, `kit_boots.skl` for `kit_boots.fmdl`); a glTF in its native `skin` (see
"Skeleton: native glTF skin" in the [Unified model format plan](../model_format.md)); a `.model`
**inline**, in its per-bone matrix table — pre-Fox has no sidecar to pair (see "Skeleton in
`.model`" in the [Model conversion plan](../model_conversion/README.md)). The `.skl` pairing is recognized in
**Common folders, shared (Faces/Boots/Gloves) folders, and player folders** alike. When a model is
pulled in — via a `.common` link, a shared link, or used locally — its skeleton travels with it into
the destination output: on Fox targets as the destination's `.skl` (pass-through bytes for `.fmdl`
inputs; generated from the IR for glTF and `.model` inputs that carry bones outside the target's
skeleton tables), on pre-Fox targets inside the written `.model`'s bone table, for every source
format. This covers the rare case where a model has a custom pose that depends on a custom skeleton;
the common case needs no SKL and the compiler's template skeletons suffice. Independently of custom
skeletons, every model is **retargeted to the target version's body skeleton** at compile time — the
games' `body.skl` files differ per version in bone set and rest pose (PES15 markedly), and the
compiler ships all of them; see "Skeleton retargeting and bone conformance" in the [Model conversion
plan](../model_conversion/README.md). This absorbs the standalone `4cc-model-simplifier-15` step.

Only two output destinations have SKL slots: **`fcl_hair`** (the `fcl_hair_sim.skl` slot) and
**`boots`** (the `boots.skl` slot) — both are full-body-skeleton models. `face_high`, `hair_high`,
and `oral` have no skeleton slot. A custom SKL arriving at a destination replaces the template
injection for that slot: if any part merged into the destination brings a custom SKL, the template
`boots.skl`/`fcl_hair_sim.skl` is not injected and the custom one is renamed to the slot's canonical
name. If no part brings a custom SKL, today's template injection stands. A custom SKL paired with a
model that resolves to `face_high`/`hair_high`/`oral` has no slot to land in and is reported as a
warning (`skl_no_slot`) — a no-op file the user likely authored by mistake.

**What the injected files are.** Both slot templates are the **body skeleton** under the slot's
name — Red ships PES21's `body.skl` and writes it as `boots.skl` / `fcl_hair_sim.skl`. The name
`boots.skl` is misleading: the game's own `boots.skl` (`resources/skeletons/pes*/boots.skl`) has
four bones (`sk_foot_*`, `dsk_toe_*`) and is never what a 4cc export ships. The community
convention this encodes is that the boots folder is the place for a **full-body model** whenever the
`fcl_hair` slot is not used — a body model saved as `boots.fmdl` needs the whole body skeleton, so
members used to add a renamed `body.skl` themselves; the compiler's template made the manual copy
unnecessary. Real boots models use a subset of the same bones, so the body skeleton serves them too.
PES21's file is the template because it has the largest bone set of the Fox versions, so any bone a
model may reference has a bind pose. **Once the retargeting pass folds bones the target lacks, that
superset rationale disappears** and injecting the *target version's* `body.skl` becomes the
consistent choice (its rest pose is what the game's animations drive). Whether the game behaves
differently with a same-version skeleton than with PES21's is untested in-game; the plan keeps PES21
as the injected file until a compile-and-play comparison on PES18/19 settles it (open point).

**Merge constraint** — parts merged into one output FMDL must reference the same skeleton. A part
with a custom SKL and a part using the default template skeleton reference different skeletons, as
do two parts with different custom SKLs. "Same" is decided by **content hash** for two `.skl`
files (identical bytes under different filenames are one skeleton) and by **bone-transform
comparison with tolerance** whenever a part's skeleton comes from the IR (glTF skins, `.model` bone
tables). The pre-Fox native merge compares `.model` bone matrices within a measured `1e-4` per
component (Libraries plan, "`pes_model::ops::merge`"). A skeleton mismatch between merge
parts is a hard error (`skl_merge_conflict`) that drops the folder.

**Kits** live in a `Kits/` folder with one subfolder per kit (this per-kit granularity also drives
the GUI's per-kit grid cells). A kit folder is named by its **slot**, optionally followed by a
label in the same `<slot> - <label>` shape as player folders: `p1`, `p1 - Lakers`, `g1 - Goalie`.
The slot part (`p1`–`p9`, `g1`, or `all`, case-insensitive) is all the compiler reads; the label is
for the human — the GUI kit cell's tooltip and the Kit config editor's tab show it, nothing is
derived from it. Two folders resolving to the same slot (`p1/` and `p1 - Lakers/`) are an error
(`kit_slot_duplicate`), not a pick. The special folder **`all/`** holds textures shared by every
kit: for each kit, the effective texture set is the kit folder's own files plus every `all/` file
whose stem the kit does not have itself — own files win, per stem. Most teams use the same
`_back`/`_leg`/`_name` number and name textures on all their kits, and copying them into nine
folders is what made old exports heavy and what let one kit silently fall behind when the set was
updated. The rule is per stem, so a kit can inherit `kit_back` and still override `kit_name`; it
covers the main `kit.dds` too, for the rare team whose kits differ only by config. The five
texture-name fields of each kit's config are derived from the *effective* set — an inherited
`kit_back.dds` makes the kit's `back` field non-empty exactly as an own copy would. `all/` is not
a kit: it has no cell, no `config.toml`, `colors.txt` or `icon.txt` (such files there are
reported and ignored), and an `all/` beside no kit folder is reported as unused. A shared base
config was considered and rejected: a kit folder without `config.toml` means "the template", so
the folder describes its own look and can be copied between exports unchanged. With an inherited
config, the same folder would compile differently in each export, and a collar or shorts-number
change is invisible until the game shows it. An inherited *texture* changes a copied kit too, but
visibly, as itself — and often intentionally, since the kit takes on the number font of the team
it joins.

A kit folder may be **completely empty** — a bare `p1/` with nothing in it, and no `all/` to
inherit from. It still declares that the slot exists, and the compiler fills it in: a bundled
placeholder main texture (the magenta/black "missing texture" checkerboard), the template
`config.toml`, and a `UniColor` entry as for any kit (colors from the kit's `colors.txt`, else the
loud magenta/black "no colors chosen" pair — see "Kit colors fallback" in the [Team compiler
plan](../team_compiler/README.md)). Teams whose whole roster is
full-body models never render a kit texture (except a pre-Fox player given the `uniform` model
type), yet the game expects every kit slot the team uses to have a texture and a config; the empty
folder says "pretend a kit exists here" without making the author draw one. The same fill applies
to any kit whose effective texture set lacks `kit.dds`, so a folder holding only a `kit_back.dds`
is a placeholder kit with a custom number font, not an error. Kit textures follow the export-wide
texture rule: matched by **stem**, in **any accepted image format** — `kit.png` is as good as
`kit.dds` (the compiler converts; the diagrams say `.dds` only by habit). Online kit designers
hand out PNGs, and a new manager should not need a DDS tool to use one.

**Kit layout marker.** The kit UV layout changed between the two engines: the shirt, sleeves and
collar strip are identical in PES 15–17 and 18–21, but the **sock islands are 68 px narrower** in
Fox (u 8–372 instead of 8–440 on the left, mirrored on the right; height unchanged) and the
**shorts islands keep their outline but are partitioned differently** (pre-Fox: a 444-px shorts
body at the outer edge plus a 212-px inner-thigh strip; Fox: a 152-px hem strip at the outer edge
plus a 456-px body). A kit drawn for one engine and compiled for the other therefore shows its
sock and shorts designs displaced. An optional empty **marker file** in the kit folder, named
`pre-fox` or `fox` (case-insensitive; the Notepad forms `pre-fox.txt` / `fox.txt` are tolerated,
as for `ingame_face`), declares which layout the kit's `kit.dds` and its `kit_mask.dds` /
`kit_srm.dds` are drawn for. When the marker names the other engine than the compile target, the
compiler re-lays those textures out (see "Kits" under "Processing" in the [Team compiler
plan](../team_compiler/README.md)); when it names the target's engine, or is absent, nothing is edited —
**no marker means "drawn for whatever you compile for"**, which is today's behavior. Both markers
in one folder is an error (`kit_layout_conflict`, kit discarded), like `fpc_conflict`. The marker
belongs in kit folders only: in `all/` it is `kit_all_file_ignored` like any non-texture, and a
kit that inherits `all/kit.dds` states the layout of that inherited texture with its own marker.
It is a marker file, not a `config.toml` key, because `config.toml` holds only data that reaches
the game's kit config; the layout describes the texture and stays beside it. The Export upgrader
never writes one: kits have passed through several converters unchanged, so their true origin is
unknowable from the files, and a wrong guess would silently displace a correct kit. The only
authored kit config is the TOML file `config.toml` — the
binary game format is never part of an export and exists only as compile output; old exports' binary
configs are converted by the [Export upgrader](../export_upgrader.md) (schema and binary layout in the
[Kit config editor plan](../kit_config_editor.md)); each kit folder also has a `colors.txt` with the
kit's two menu colors, one per line, in the same color-entry format the old Team Note txt used, and
optionally an `icon.txt` holding the kit's menu icon number (0–23, the old Note txt kit entries'
trailing number — selects the two-color kit icon pattern shown next to the formations in the
prematch gameplan screens, which only PES 15/16 display; absent = default 3):

```
Kits/
├── all/                      (optional: textures every kit inherits unless it has its own file of that stem)
│   ├── kit_back.dds
│   ├── kit_leg.dds
│   └── kit_name.dds
├── p1 - Lakers/              (slot, optionally ` - ` and a free label)
│   ├── config.toml           (kit config — TOML; compiled to the game binary at compile time)
│   ├── colors.txt            (the two menu colors, one per line; old Team Note color-entry format)
│   ├── icon.txt              (optional: menu icon number, 0-23; absent = default 3)
│   ├── pre-fox               (optional empty marker, `pre-fox` or `fox`: which engine's kit layout the
│   │                          main texture and mask are drawn for; absent = the compile target's)
│   ├── kit.dds               (main kit texture)
│   ├── kit_mask.dds          (kit textures: generic "kit" prefix — no more u0XXXp1 naming;
│   ├── kit_srm.dds            the prefix is replaced with the kit's ID, e.g. u0701p2,
│   ├── kit_chest.dds          at compile time; `_mask` is the pre-Fox material map and `_srm`
│   └── kit_*.dds              the Fox one — each is emitted only for its engine, never converted)
├── p2/
│   ├── kit_name.dds          (overrides all/kit_name.dds for this kit; kit_back/kit_leg are inherited)
│   └── ...
├── p3/                       (empty: a placeholder kit — checkerboard texture, template config, menu
│                              colors from a colors.txt here or the loud "none" pair; for full-body teams)
└── g1/
    └── ...
```

**Portraits** — a player's portrait normally lives in their player folder (stem `portrait`, any
accepted image format; the compiler converts to the game's DDS). The
standalone `Portraits/` folder (`player_NN.dds` files) is deliberately kept alongside: some managers
make custom portrait sets — while keeping the players' models unchanged — depending on the opponent
team they are about to face, and the standalone folder supports those model-less portrait exports.
Conflicting portraits for the same player in both locations remain an error (`portrait_conflict`).

**Root files** — the old Team Note txt is dismissed entirely. In its place:

```
<export root>/
├── colors.txt                (optional: the team's colors, one per line — same format as a kit folder's colors.txt)
├── notes.txt                 (optional: strict UTF-8 free-form notes, collected into teamnotes.txt at compile time)
├── README.txt                (optional: for humans only — known to the model, ignored by the compiler;
│                              the Team creator writes one with next steps)
├── players.txt               (optional for teams, required for refs: slot → player folder mapping)
├── logo.png                  (optional: the team logo, one image in any accepted format, any size;
│                              a `_crop` / `_stretch` / `_fit` tag chooses how a non-square one
│                              becomes square — see "Logo")
├── logo_small.png            (optional: a different image for the game's smallest, 128² logo —
│                              menus use it; same tags; absent = downscaled from logo)
├── Players/ ...
├── Kits/ ...
└── ...
```

- **Team identity**: the canonical `team_name` is always derived from the first word of
  `export_display_name`; the complete stem remains presentation/source identity only. There is no
  `Team:` line anywhere.
- **Team colors**: optional root `colors.txt`, feeding `TeamColor.bin` (absent: the team's existing
  bin colors are left untouched).
- **Kit colors**: each kit folder's `colors.txt` (plus optional `icon.txt`), feeding `UniColor.bin`.
- **Notes**: optional strict-UTF-8 root `notes.txt`, replacing the old "Other Notes" section. An
  optional BOM is stripped, newlines normalize to LF, invalid encoding drops the note with
  `notes_encoding_invalid`, and empty/whitespace-only content produces no `teamnotes.txt` entry.
- **Player numbers**: optional root `players.txt` for normal teams as an alternative to numbered
  folder names; required for referee exports, whose authoritative slot mapping and repetition cannot
  be expressed through numbered folder names.
- **Logo**: optional, at most **two root image files**, any accepted raster format (the texture
  allowlist), any size. Stem grammar: `logo` `[_small]` `[_<fit>]`, tags in that order, matched
  case-insensitively. `logo*` (no `_small`) is the **main** logo: the compiler always produces the
  game's 512² and 256² PNGs from it, and the 128² one too unless `logo_small*` exists.
  `logo_small*` is an optional **different image for the 128² logo only** — the game's menus show
  that one, and teams sometimes want a simplified or joke version there; it never influences the
  two larger sizes. `<fit>` is one of `crop` (center-crop the long side), `stretch` (resample to
  square, distorting), `fit` (letterbox with a transparent border), each file carrying its own; a
  square source needs no tag; a non-square source without one is treated as `fit`, the only mode
  that loses nothing and distorts nothing — the tag exists to *opt into* loss. A `logo_small*`
  without a main `logo*` is an error: the game needs all three sizes and the small one is not a
  source for the large ones. There is no `Logo/` folder: the old format asked authors for a
  hand-made triplet with a placeholder team ID baked into three filenames, and one in eleven
  shipped exports got the sizes wrong; files the compiler resizes cannot. More than one file per
  role (two `logo*`, or two `logo_small*`) is an error, not a pick.

The Export upgrader generates all of these from the old Team Note txt and `Logo/` folder when
migrating.

At compile time, the pipeline:

0. **Hand auto-split** — in-process in Rust via `model_convert`, select each hand's vertices with
   positive `skh_*_l`/`skh_*_r` weights, grow the selection once along the mesh topology, and separate
   it into `glove_l`/`glove_r`, leaving the body. This is equivalent to Blender's select → `Ctrl +`
   → `P` workflow, not an invocation of Blender (see "Hand auto-split" in the
   [Model conversion plan](../model_conversion/README.md)). Models without such weights pass through unchanged.
   The split parts appear as virtual model files before categorization.
1. **Categorizes** each model file as face/boots/gloves — by filename convention or by
   face.xml-style metadata (the same categorization problem the 16→21 converter already solves via
   `parseFaceXml` and filename matching). Anything the conventions don't claim is **face content by
   default**, so there is no "uncategorized model" failure. Pre-Fox the category only picks the
   model's *type* attribute in the generated `face.xml` — everything ships in the one face folder;
   Fox splits the categories into separate folders and merges each category's parts (see "Common
   model links and model merging")
2. **Assigns IDs automatically** (Fox needs both pools; pre-Fox only the shared one, having no
   player-exclusive boots/gloves folders — except under `ingame_face`, see "ingame_face marker") —
   from a **hardcoded per-team block of 40 IDs** in the 4-digit boots/gloves ID space: the first 100
   IDs are reserved for the stock PES boots, then each team gets a fixed block starting from team ID
   701 (`block_start = 101 + (team_id - 701) × 40`; team 701 gets 101–140). The block splits into
   the player-exclusive part (deterministic: `block_start + player_number - 1` for slots 01–23,
   extending the 16→21 converter's `bootsId = base_id + player_number - 1` scheme) and the **17
   shared-folder IDs** (the remainder, assigned in alphabetical folder-name order — deterministic,
   so recompiling an unchanged export yields the same IDs). Link files resolve to the assigned IDs.
   Boots and gloves are **disjoint game namespaces** (separate `boots/{id}/` and `glove/{id}/`
   folders, `k`/`g` prefixes), so the identical block layout applies independently in each — no
   boots-versus-gloves split of the block is needed. Sizing rationale: at 220 teams (IDs 701–920),
   the 40-ID blocks end at 8900, leaving 8901–9899 unallocated and keeping clear of the `99XX` band
   that referee `k99XX`/`g99XX` IDs occupy (the hard ceiling would be 44 per team, ending at 9780;
   45 would cross into the referee band and overflow into 5 digits). The block size is fixed for the
   lifetime of a scheme version — IDs are baked into distributed savefiles — so builds record
   `allocation_scheme_version = 1` in their metadata. The ABI is future-upgradeable: a later exe
   patch might allow 5-digit IDs, letting each player's boots/gloves ID equal their player ID; any
   such change is a new scheme version, and since a scheme change is always shipped with a
   from-scratch savefile remake, bumping the version is a sanctioned path rather than a
   compatibility break
3. **Splits and packs** everything into the correct game structures (face folder with player ID,
   boots/gloves folders with assigned IDs), rewriting FMDL/MTL/XML texture paths to match
4. **Plans conditional savefile mutations** for assigned IDs and independent accepted settings.
   Producer commits activate mutations that reference compiled assets; after all outcomes are known,
   `pes_savefile` serializes only activated asset mutations plus eligible independent settings. This
   makes the player wear successfully emitted boots/gloves without pointing at failed content — the
   coordination currently done manually with 4ccEditor.

**The old export format is not supported by the compiler.** Old exports are migrated once with the
Export upgrader tool (see the [Export upgrader plan](../export_upgrader.md)), which also resolves old
embedded IDs to the new name-based structure.

---
