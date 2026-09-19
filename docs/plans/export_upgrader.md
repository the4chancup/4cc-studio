# 4cc Studio — Export upgrader plan

The Export upgrader (`crates/tools/export_upgrader`) migrates exports from the old
format to the Studio player-folder format defined in the
[Team compiler plan](team_compiler/README.md) — the compiler only supports the Studio format,
so this is the one-time migration path for every existing export.
Platform context is in the [core plan](core/README.md).

---

## What it does

Migration builds a separate output draft and leaves the original source intact. References below
to moving, flattening, or removing files describe that draft, not destructive edits to the input.

1. **Reads the old export** (folders / .zip / .7z): Faces/Boots/Gloves item folders with embedded
   IDs, Kit Configs + Kit Textures folders, Common, Portraits, Logo, Other, Team Note txt. (A bare
   folder of item folders is the other accepted input — see "Model-folder mode".)
2. **Reads the team's savefile data** (via `pes_savefile`) to map old boots/gloves IDs to players —
   the old format encodes "which player wears boots k0915" only in the savefile, so the savefile is
   the input for a faithful migration; without one, loose mode (step 11) infers the wearers from the
   IDs instead.
3. **Builds player folders**: each player's face folder contents move into `Players/NN - Name/` (NN
   from the face folder's `XXXxx` suffix). Name comes, in order of preference, from the **face
   folder's own name part** (`XXX15 - Snuffy` → `Snuffy`; the author's curated name, and what the
   folder is called today), then from the **worn boots/gloves folder's name part** (a player with
   boots/gloves but no face folder), then from the **savefile name** with its colour codes stripped
   (`pes_savefile`'s `display_name()` — see that plan's "Name colour codes"; the raw codes are
   control characters, invalid in folder names), then NN alone. Red's Pre-Studio upgrader uses this
   same order. A `portrait.dds` inside the face
   folder stays with the player folder (the Studio format keeps that convention). Old `ingame_face` and
   tolerated `ingame_face.txt` markers are preserved and normalized to the new bare `ingame_face`
   marker; a player whose savefile record wears boots/gloves but who has **no face folder** also gets
   a player folder with an `ingame_face` marker, since a stray boots/gloves folder is exactly the
   ingame-face case. Model folders used by exactly **one** player are merged into that player's
   folder (a boots folder used only by player 15 becomes `boots.fmdl` + textures inside `15 - Name/`).
4. **Keeps shared folders**: a model folder used by **two or more** players stays as a named shared
   folder (`Boots/Crocs/`), and each of those players' folders gets a link file (`Crocs.boots`).
   Naming: nearly all model folders in current exports already carry a descriptive name after their
   ID (`k0915 - Crocs`), so migration uses that name part when present (`Boots/Crocs/`) and falls
   back to the old ID (`Boots/k0915/`) for folders named by ID alone; either way users can rename
   them (and their link files) afterwards.
5. **Restructures kits**: Kit Configs + Kit Textures are reorganized into `Kits/` subfolders, one
   per kit slot (config name `_1st_` → `p1/`, … `_9th_` → `p9/`, `_GK1st_` → `g1/`); the binary
   config is converted to `config.toml` via `libs/kit_config` (texture-name fields dropped — the
   compiler re-derives them from the textures present; undecoded bits preserved in the `[unknown]`
   table — see the [Kit config editor plan](kit_config_editor.md)); kit textures are renamed from
   the old `u0XXXp1_…` pattern to the generic `kit` prefix (`kit.dds`, `kit_mask.dds`,
   `kit_chest.dds`, …) that the compiler re-expands at compile time; kit colors are extracted from
   the Team Note txt into per-kit `colors.txt` files (a kit entry's trailing icon number, when
   present and not the default 3, goes to that kit's `icon.txt`); the Note's kit-entry order maps
   entries to slots (player entries → `p1`…, `gk:` entries → `g1`…), matching how Red numbered them
   in `UniColor.bin`. Kit folders are written as bare slots (`p1/`) — the old format has no kit
   names to carry. Then the shared textures are **hoisted into `Kits/all/`**: a stem present in
   *every* kit folder with byte-identical contents (two or more kits) moves to `all/` and the copies
   are deleted (`kit_texture_hoisted`, I, naming the stem). Only the all-kits-identical case
   qualifies: hoisting a stem that some kit lacks would hand that kit a texture it never had and
   flip its config's texture-name field — a game-facing change the upgrader must not make. So the
   upgraded export compiles byte-for-byte as before, just without nine copies of the same number
   font. A survey of the maintainer's 129 multi-kit old exports found `_back`/`_leg`/`_name`
   byte-identical across every kit in roughly a quarter of them; the rest share partially or were
   re-saved with different bytes, and stay as they are — the author can hoist by hand once the
   `all/` folder exists. No kit layout marker (`pre-fox` / `fox`, see "Kit layout marker" in the
   [Aesthetics export plan](aesthetics_export/README.md)) is ever written: old kits have passed through
   several export converters unchanged, so nothing in the files says which engine they were drawn
   for, and a wrong guess would silently displace a correct kit's socks and shorts. Authors add the
   marker when they compile across engines.
6. **Replaces the Team Note txt**: team colors are extracted into the root `colors.txt` and the
   "Other Notes" section into the root `notes.txt` (both optional — only generated when the Note has
   the corresponding content). The rest of the Note is dropped. Team identity for the migration is
   resolved the way Red's compiler does, since old exports carry the authoritative name in the Note:
   the Note txt's `Team:` line first (the first Note txt found at the flattened export root), the
   canonical `/xx/` first-word key derived from the export folder name as the fallback — an export
   named after something else than its team (e.g. "Numbers" for `/#/`) must still resolve. The
   resolved key is used only for the `teams_list.txt` lookup; the complete folder name remains the
   display/source name, and the Studio output carries no Note (the format has no `Team:` line —
   see the Team compiler plan's "Root files").
7. **Carries over** Portraits (`player_NN.dds` naming is unchanged in the Studio format), Collars,
   and Common, and **converts the Logo folder** to the Studio root files: the old `_r_ll` (512²)
   image becomes root `logo.png`; the old `_r` (128²) image becomes root `logo_small.png` **only
   when it is clearly a different picture** — most old small logos are plain downscales, which the
   compiler now regenerates, but some are deliberate menu variants (simplified, joke), and the
   upgrader must tell the two apart rather than always carry or always drop. The test: downscale
   `_r_ll` to 128² the way the compiler does (Lanczos3, alpha preserved), resample the old `_r` to
   128² too if it is not already, and compare the two pixel by pixel over RGBA. A pixel *differs*
   when any channel differs by more than 32 (of 255); two pixels that are both fully transparent
   are equal whatever their color channels hold (authors leave garbage behind alpha 0). The small
   image is a distinct variant, and is carried, when **more than 40 % of pixels differ**
   (`LOGO_SMALL_DISTINCT_SHARE`); below that it is a downscale and is dropped. The number comes
   from measuring the 52 distinct `Logo/` pairs in the maintainer's export library: the 45
   downscales score 0–29 % — far above codec noise, because authors downscaled with
   nearest-neighbor or sharpening kernels and cut alpha edges differently, which the channel
   tolerance does not absorb — and the 7 deliberate variants score 63–97 %, every one a wholly
   different drawing rather than a simplified crest. 40 % sits in the empty band, nearer the
   downscale side, so the mistake still possible is the harmless one (a redundant file the user
   deletes), not a lost drawing. Both outcomes are reported with the measured share
   (`logo_small_carried`, `logo_small_dropped`; both I), so a wrong call is one look at the report
   away from being fixed by hand: the old file is still in the source export. The `_r_l` (256²)
   file is always dropped — the compiler regenerates it. A folder without
   a usable `_r_ll` falls back to the largest member present, reported, and then has nothing to
   compare the small image against, so no `logo_small.png` is written. Across migrated model folders, including Common, normalize old `.mtl`
   texture references to stems as required by the Studio format; emitted game MTLs regain `.dds`
   extensions at compile time. A reference that pointed into the team's Common folder by game path
   (`model/character/uniform/common/???/hair.dds` in an `.mtl`, the Fox common directory in an FMDL
   path table; `???` is the `XXX` placeholder or a baked-in team ID, as with the kit magic below)
   cannot simply become the stem `hair` — that would resolve in the player folder and fail — so the
   upgrader writes the stem **and creates the texture link** `hair.dds.common` in that folder, which
   is the Studio spelling of the same reference (see "Link files" in the [Unified model format
   plan](model_format.md)). A Common-path reference whose file is not in the input's `Common/` is
   reported and left as it was. This applies to structural migration, not only optional glTF conversion.
   In the same pass, the legacy kit-dependent path magic is rewritten to the Studio spelling:
   `u0???p0` → `kitN`, `u0???p1`…`u0???p9` → `kit1`…`kit9`, in file and folder names and in
   `.mtl`/`face.xml` paths and FMDL path tables alike (see "Kit-dependent assets" in the [Unified
   model format plan](model_format.md)). The `???` matches any three characters: authors wrote the
   `XXX` placeholder literally or baked the real team ID in (`u0872p1.dds`), and both forms mean the
   same thing. The upgrader is the **only** Studio component that reads the legacy spelling — the
   compiler has no notion of it. `dummy_kit*` references are left as they are — the compiler
   keeps them as reserved game-substituted names. Pre-Fox **model-type prefixes become suffixes**:
   Red typed `face.xml` entries by a leading type (`gloveL_foo.model`, matched case-insensitively
   with underscores ignored, or a `model_type_<x>` token anywhere); the Studio format puts the type
   last (`foo_gloveL.model`, `foo_model_type_<x>.model` — see "Model names: a free part plus a
   suffix" in the [Aesthetics export plan](aesthetics_export/README.md)). The swap applies **only to folders
   without a `face.xml`** — filename typing exists to generate the xml, so a folder that ships its
   own xml carries the types there and its files are left exactly as named. In xml-less folders the
   upgrader moves a recognized leading type or `model_type_` token to the end of the stem (`.mtl`
   name matching is unaffected, since it accepts prefix or suffix matches); names that already end
   in a type, or carry none, are untouched.
8. **Generates `settings.toml`** for each player, populated from the savefile's aesthetic data
   (physique, strip style, skin/iris color, etc.) — the same TOML-generation logic the save editor
   uses. Boots/gloves IDs are omitted: old IDs help resolve the wearers during migration, but the
   migrated models/links determine Studio's automatic assignments. Since name writing is opt-in, the upgrader emits `name = true` when the folder name's name
   part equals the savefile name, or the explicit string otherwise — which is the case whenever the
   folder was named from the face folder rather than the savefile, and whenever the savefile name
   can't be expressed in a folder name (filesystem-invalid characters, **including name colour
   codes**, which the explicit string preserves byte-for-byte so a compile never strips them). Players whose savefile settings match
   `pes_savefile`'s FPC enable preset get an `fpc.on` marker file instead of the raw strip settings.
9. **Migrates referee exports**: the old referee layout (`refs.txt` + per-referee
   face/boots/gloves/common subfolders — the prototype of the player-folder format) compiles as it is
   thanks to the compiler's legacy acceptance (reserved subfolders, `refs.txt` alias), but is
   upgraded to the fully unified layout all the same: `refs.txt` becomes a root `players.txt`
   carrying the slot mapping (including repeated entries — the same model backing multiple of the 35
   referee slots for rarity control) in `aesthetics_export`'s canonical UTF-8/LF roster grammar shared with
   the Refs arranger, and the `refs.txt` is removed; each referee's subfolders are **flattened** into
   the player folder root (step 12). Either old `ingame_face` spelling is normalized exactly as for
   team player folders. No savefile needed — referee IDs are compiler-generated.
10. **Reports** anything that couldn't be migrated automatically (e.g. contents of the Other folder,
    unassigned model folders) for manual attention — the `UnmigratedContentFound` pipeline event;
    the tool defines its own message catalog on the shared `Message` structure (see the Team
    compiler plan's "Message catalog").
11. **Loose mode** (no savefile available): step 2 needs the savefile only to learn which player
    wears which old boots/gloves ID. When no savefile is given, the upgrader offers loose mode,
    which infers the wearer from the **ID's position inside the team's Red block** instead —
    exports being upgraded were made for Red's per-team ranges of **25** IDs (`101 + 25 × n`, see
    Red's `teams_list.txt`), not Studio's 40-ID blocks, so the block is inferred from the first
    boots (or gloves) folder ID found (`range_start = 101 + 25 × ⌊(min_id − 101) / 25⌋`; a first
    folder of 0128 still infers the 126 block) and `k0126` → player 01, `k0128` → player 03. IDs on
    the block's spare positions (24–25) belong to nobody and are reported (step 10); an ID on a slot
    without a face folder still yields a player folder with an `ingame_face` marker, as in step 3.
    Every ID maps to exactly one slot, so folders are always merged in. Loose mode is a documented
    assumption, never a cross-check: it is offered only when the savefile is absent, the report
    states the assumed pairings, and no `settings.toml` aesthetic data can be generated (step 8 is
    skipped, apart from `name` handling, which needs no savefile). Red's Pre-Studio upgrader
    implements this same mode (its `export_upgrade.py`).
12. **Flattens legacy reserved subfolders**: `face/`, `boots/`, `gloves/` and `common/` inside a
    player folder (see the Aesthetics export plan's "Reserved subfolders") are folded into the player
    folder root, since the flat layout is the canonical one. A file name present in two locations
    with identical bytes is deduplicated; with different bytes it is reported as unmigrated
    content (step 10) and the folder is left as it was — the legacy layout still compiles, so
    nothing is lost by declining to flatten. A model file in `common/` is reported likewise.
13. **Optional glTF conversion** (`--convert-to-gltf`, off by default) — after the structural
    migration is complete and verified (step 10's report is clean), a second pass converts all
    model files to glTF in place: every `.fmdl` (Fox exports) or `.model`+`.mtl` (pre-Fox exports)
    in player folders, shared folders, and Common is imported to the IR and exported as one `.glb`
    per model via `model_convert`'s `ir_to_gltf`. Old exports are always single-engine (Fox-only or
    pre-Fox-only — Red cannot handle mixed states), so there is no dual-set superset merge to
    automate: each model converts plainly through one path. Skeleton transforms come from the
    FMDL's companion SKL or the embedded template skeleton, or from a `.model`'s inline bone table
    (see "Skeleton reconstruction from FMDL" and "Skeleton in `.model`" in the [Model conversion
    plan](model_conversion/README.md)). Textures stay in place
    (existing DDS/FTEX referenced by stem from the generated `materials.toml`, whose entries carry
    the `family` inferred from the native shader plus the native engine table — see "Emission" in
    the [Unified model format plan](model_format.md)). A `.mtl.common` link converts to the
    matching `.materials.toml.common` link, and the Common `.mtl` it points at is converted in
    place like any other; texture links (`hair.dds.common`, created by step 7's path normalization
    or already present) are format-neutral and stay as they are. For pre-Fox
    exports, `.mtl` texture paths are stripped of extensions (normalized to stem-based references,
    matching the Studio format). Replacement is transactional per model folder: all parts convert
    first, and the structurally-migrated originals remain recoverable until publication completes.
    Multiple writes/deletes are not a single atomic filesystem operation. Conversion or publication
    failure restores the native folder, or retains a clearly identified recoverable copy if rollback
    itself fails; success is not reported for a partial migration. The two-step design keeps the
    structural migration lossless and parity-testable: the test suite validates the structural step against Red's reference output,
    and the glTF conversion has its own roundtrip tests in `model_convert`.

## Model-folder mode

The second input shape, alongside whole exports: **a folder of old-format model folders** — the
item folders that used to live under `Faces/`, `Boots/`, `Gloves/` (`XXX15 - Snuffy`,
`k0915 - Crocs`, …), placed loose in one directory, optionally with a `Common/` beside them. This
is what the community's export converters accept in their "player folders" mode, and it exists
for the same reason here: once a team's export has been upgraded, new models keep arriving in the
old spelling (a face made in an old workflow, a boots folder from a previous cup), and the
kit-dependent magic in particular (`u0???pN` → `kitN`) is not something an author should hand-edit
inside FMDL path tables. The user upgrades the folder and drops the results into their Studio
export.

Each top-level folder is migrated as **one model folder** with the per-folder rules of step 7 —
`kitN` rewriting in names and path tables, `.mtl` stem normalization, pre-Fox type prefix → suffix
(xml-less folders only), `ingame_face` normalization, `.mtl.common` → kept as is (a `Common/` in the
input is upgraded with the Common rules and carried over, so the links keep resolving). Output
naming follows steps 3–4 so the result is directly droppable: a face folder becomes a player folder
`NN - Name/` (NN from its `XXX` suffix, Name from its name part; without a suffix, the name part
alone); a boots/gloves folder becomes a shared-style folder named by its name part (`Crocs/`), or
by its old ID when it has no name — the user drops it into `Boots/`/`Gloves/` and adds the link
file, or moves its contents into a player folder (renaming the model to `boots.fmdl` /
`glove_*.fmdl` as the Studio format expects). The report lists, per output folder, where it goes.

Not in this mode: kits, portraits, logo, `settings.toml`, the savefile, loose mode, the roster. No
savefile is read — there are no wearers to resolve. A top-level folder that holds no model files
(and is not `Common/`) is reported as unmigrated content (step 10) and left out; the converters'
two-level `players/<player>/<model folders>/` layout is therefore *not* accepted — one level, one
folder per model, so that "what is a model folder" never has to be guessed.

Verification: fixtures with one face folder carrying `u0XXXp1` textures and path-table references
(asserting `kit1` everywhere, folder renamed `NN - Name`), one boots folder with a prefix-typed
pre-Fox model (asserting the suffix form and the `Crocs/` output name), and a `Common/` with an
`.mtl` whose references are stem-normalized while the player folder's `.mtl.common` link still
resolves against the migrated Common through `aesthetics_export`'s validation; plus one player
`.mtl` referencing `model/character/uniform/common/XXX/hair.dds` (asserting the stem `hair` in the
migrated `.mtl`, a `hair.dds.common` link beside it, and the stem resolving into Common through
validation) and one such reference whose file is absent from `Common/` (asserting it is reported
and left unchanged).

## CLI

```text
studio export-upgrader upgrade <export> [--savefile <EDIT>] [--loose] [--convert-to-gltf] [-o <dir>]
studio export-upgrader upgrade-models <folder> [-o <dir>]
```

`upgrade` takes one old export (folder, `.zip` or `.7z`); the output is a sibling folder named
after the export unless `-o` is given; `--loose` is rejected together with `--savefile`.
`upgrade-models` is model-folder mode; its output is one folder per migrated input folder under
the output directory. Both print step 10's report and exit non-zero when it is not clean. The GUI
offers the same two modes as two drop targets on the tool view.

## Dependencies

Uses `aesthetics_export` (the Studio format's object model, conventions, and validation — the upgrader builds
and verifies its output with the same crate the compiler consumes it with), `pes_savefile` (savefile
reading, TOML generation), `archives` (old export loading), `vtree`, `kit_config` (binary → TOML kit
config conversion), and the format lib crates for content inspection. Old-format knowledge stays in
this tool. The structural migration (steps 1–12) does no model-format conversion, but does rewrite
text metadata, kit configs, and legacy MTL texture references. The optional glTF conversion (step 13)
uses `model_convert`'s `ir_to_gltf` path. The crate remains a build dependency; the runtime
`--convert-to-gltf` flag only determines whether that path is executed. Structural migration lands
in Phase 6; the optional glTF pass waits for Phase 7's importer/exporter support.

## Verification

For each migration the upgrader writes an in-memory/output draft, enumerates it into canonical
`vtree::ScopePath` entries plus small metadata, parses that listing back through
`aesthetics_export::parse_listing` as `ParsedAestheticsExport`, runs the shared validation, and reports every
scoped issue. Shared validation may produce a sanitized `ValidatedAestheticsExport` with ineligible
files/folders omitted, but the upgrader applies a stricter publication rule: only a report with no
errors and no dropped scopes is labeled a clean migrated export. Any other draft remains available
for manual repair rather than being mislabeled as fully migrated. Fixtures cover both old
`ingame_face` spellings and assert one canonical bare marker in the migrated player folder;
savefile names with colour codes (see the pes_savefile plan) and assert a plain folder name plus an
explicit `name = "..."` preserving the codes; a Red-era referee export, asserting that its
face/boots/gloves/common subfolders are flattened, `refs.txt` becomes `players.txt` and is removed,
and the result compiles identically to the unflattened original; a flattening collision (same file
name, different bytes, in two subfolders) left in place and reported; loose mode on an export
whose Red block starts mid-range (first folder 0128 → block 126, 0128 → player 03, 0149 → spare,
reported); and team identity — an export whose Note `Team:` line differs from its folder name
resolves from the Note (e.g. "Numbers" → `/#/`), one without any Note falls back to the first-word
key; and the logo comparison — the two real pairs closest to the 40 % line from the survey behind
step 7: the worst-scoring downscale (`/aceg/` VGL26, a nearest-neighbor `_r` at 29 %) asserting no
`logo_small.png` and `logo_small_dropped`, and the closest deliberate variant (`/fgog/` VGLX
Friendlies, 63 %) asserting `logo_small.png` and `logo_small_carried`. Real files, not synthesized
ones, so the threshold is checked against what authors actually shipped; the asserted shares are
the measured ones, so a change to the resampler or the tolerance shows up as a failing number.

The upgrader is exercised end-to-end by the parity test (see the Team compiler plan): the full
library of old-format exports is upgraded, compiled, and the game-facing output compared against
Red's reference output.
