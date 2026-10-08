# 4cc Studio — Team compiler plan: Blue features to port

Part of the [Team compiler plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Unimplemented Blue Features to Port

Blue is a prototype missing features that Red has. The Rust rewrite must implement these.

### Referee export processing (Red is the behavioral spec)

Blue explicitly skips referee exports (`coordinator.py:218`); Red's
`Engines/python/lib/referee_tools.py`, together with its referee-related path editing in
`fmdl_editing.py` and export routing in `export_move.py`, is the authoritative behavioral
specification.

**The referee export format changes only in name.** Red's current referee layout — `refs.txt` for
number→name mappings plus per-referee face/boots/gloves/common subfolders — was the prototype of the
player-folder format, tested only on referees. Referee exports adopt the **unified player-folder
format**: a `refs` export contains `Players/` folders exactly like a team's aesthetics export, with shared models
in named shared folders + link files. Since the per-referee subfolders are the format's reserved
subfolders (see "Reserved subfolders" in the [Aesthetics export plan](../aesthetics_export/README.md)) and `refs.txt` is accepted as a
legacy alias of `players.txt` (see "Player numbering"), a current-day referee export **compiles as it
is**. The Export upgrader still upgrades it to the fully unified layout — flat player folders and
`players.txt` — like every other legacy export, since the flat layout lists a player's contents more
explicitly; the legacy acceptance exists because it costs little, not as a second dialect to author
in.

**How a referee CPK is composed.** Every PES version has **35 referee slots**
(`referee001`–`referee035`), used randomly or pseudorandomly in every match. Filling all 35 with
distinct models is overkill, so in practice **about 8 models are repeated across the slots**, with
each model's repetition count determining how rare or frequent that referee is. Red drives this
through `refs.txt`, which may list the same folder name under multiple slot numbers. The unified
format keeps the mechanism as the required root **`players.txt`** slot mapping (see "Player
numbering" in the export format section): up to 35 slot entries, each naming a player folder, with
the same folder free to appear under any number of slots. The general multi-mapped player rule
performs shared preparation once per distinct source folder and slot instantiation once per listed
referee slot, rendering that slot's `refereeXXX`/`k99XX`/`g99XX` IDs and paths. Referee player
folders therefore carry no numbers in their names — the mapping lives in `players.txt`. In every
other respect a referee folder is a player folder, its `face.xml` included: on pre-Fox its local
boots and gloves are typed entries of that `face.xml` ("A link plus local models combines" in the
[Aesthetics export plan](../aesthetics_export/player_folders.md)), so no `k99XX`/`g99XX` folder is
written for them, and the game, modded to load `k99XX`/`g99XX` for slot XX, shows none there;
Fox writes them as the slot's `k99XX`/`g99XX` folders. A refs export may hold shared `Boots/` and
`Gloves/` folders like a team's, but a referee has no team block to give one an ID of its own: a
referee's link resolves to his slot's `k99XX`/`g99XX`, the shared folder written as that folder
for every slot that links it (on Fox the referee's local parts merged in, as for a player's own
folder).

**Preparing the slot mapping is a separate tool's job.** The slots are not drawn uniformly — each
PES version has measured slot appearance rates (flat per-slot chances on 17–21, a pattern table on
15/16 that demands matchday rotation) — and on PES 18–21 the community's Fox referee hook overrides
the draw from per-match lists. The [Refs arranger](../refs_arranger.md) edits `players.txt` inside the
refs export (against those tables, or as one slot per referee plus a weighted fallback fill when
the hook decides who appears) and, on Fox, the hook's `ref_lists.txt` beside it, then switches the
user to this tool for the compile — the arranger never compiles, and this tool's referee behavior
is unaffected by it: a saved `players.txt` arrives like any other file edit via the folder
watcher, and `ref_lists.txt` is not this tool's concern.

What carries over from Red as compiler-internal behavior (invisible in the format):
- `ExportIdentity::Referees`, rendered as fixed numeric ID 999 only inside game paths/templates (999
  is not a normal `TeamId`); per-referee IDs generated as `refereeXXX` / `k99XX` / `g99XX`
- Per-referee common structures and XML/MTL/FMDL path updates — no longer referee-specific: the
  standard texture relocation step moves **every** player's textures to a per-player common
  subfolder (see `pipeline.md` "Per-model-folder parallel steps"), which is Red's referee-only preprocessing
  generalized to all players
- Referee base template content (`refscpk` templates)

What does not carry over is how Red showed the referee marker (`ref_marker.dds`): pre-Fox it
replaced a texture of the referee template (`parts/referee/incom_bsm.dds` in the refs CPK), and
on Fox it wrote `cup_logo.ftex` into the system file `Data/dt00_x64.cpk` after an interactive
prompt. Both engines show the marker through a **reserved collar** instead, one method with no
pre-Fox exception, so nothing outside the refs CPK is ever written and no consent, backup or
rollback is needed:

- stock collar **77** is reserved for the referees, as 105 is for FPC (a stock collar of every
  target version, and one that none of the 4,635 kit configs on the maintainer's machine uses,
  Konami's and the cup's alike, surveyed 2026-10-03). A regular team may not use it: a `Collars/` file claiming it reports `collar_id_conflict`, and a team kit whose
  effective collar or winter collar (after FPC reconciliation and custom-collar rewriting) is
  the reserved one reports `kit_collar_reserved` and is dropped, since its players would wear
  the referees' marker;
- the refs CPK carries the marker model as that collar, at the collar's `nocloth` path ("Game
  paths reference" in `pipeline.md`), in the target's format, bundled with the compiler like the
  other referee templates. On Fox that is `collar_077.fmdl`, the file a Fox referee loads. On
  pre-Fox the referee draws `referee_collar_<ID>.model` and needs `collar_<ID>.model` to exist
  (a missing one drops him from the pre-match scenes; in-game on PES 17, 2026-10-08), so the
  refs CPK carries the marker as `referee_collar_077.model` with its `.mtl`, beside an empty
  `collar_077.model` (the cup's FPC collar 105 ships the same pair). The 77 reservation holds
  on both engines: a team's `collar_077` would be the one the referee finds;
- the model's texture path names the marker texture in the referees' Common output, which is
  `ref_marker.dds` converted like any Common texture (the bundled Fox model's base texture,
  `common/000/sourceimages/cup_logo.dds`, is pointed at `common/999/sourceimages/ref_marker.dds`;
  its normal and specular maps stay the game's dummies);
- the referee template kit configs name that collar, as collar and winter collar, a
  replacement from the data directory's `templates/referees_fox/` included (nothing else in
  them changes). They are loose files under
  `common/character0/model/character/uniform/team/referee/` in the refs CPK on both engines,
  as the templates ship them (`referee_DEF_1.bin` and the rest). On Fox each is also written
  as the entry of its name in the bins CPK's `UniformParameter.bin` (`referee_ACL_1` and the
  rest, the entries the bundled base and the installed bins already hold), replacing the
  entry carried forward: the game needs the loose file but reads the values from the entry,
  as it does for a team's kit configs (in-game tests: a collar changed in an entry was worn).
  Loose files alone leave the referees on the entries' collar 105.

Without `ref_marker.dds` the collar model and the texture are not emitted, and the template
configs keep the collar they had. A marker that its texture checks drop or whose conversion
fails counts as absent, reported by its texture code: the collar and the texture stand or fall
together, so the referees never wear a collar that names a texture the CPK lacks.

The marker model is the one the legacy tools ship as a referee prop: a square about 1.5 m
wide lying on the ground, slightly tilted, under the referee. On Fox it is the 4cc's
`referee_prop.fmdl` (one mesh, 4 vertices, material `judge_watch`, base texture
`cup_logo.dds`), on pre-Fox the `referee_prop.model` and `.mtl` of Red's referee template
(material `judge_incom`). Drawn as a collar it stays on the ground instead of following the
neck because of **static painting**, the cup community's trick: its vertices are weighted to
a dummy vertex group, usually named `static`, that is no bone of the body skeleton, so no
animation moves them. The referee step checks that the bundled models are painted that way.

### Features that disappear in a compiled GUI app

These Red features are consequences of being a distributed Python script and are eliminated by the
Rust + GUI architecture:

| Feature | Red location | Why it disappears |
|---------|-------------|-------------------|
| Auto-update system | `updating.py` (22KB) | GUI "check for updates" dialog; Rust HTTP via `reqwest` |
| Self-healing / dependency check | `dependency_check.py`, `file_management.py` | Single binary with embedded templates/fallback bins (see `pipeline.md` "Resolved decisions") — no missing files or packages |
| Admin privilege elevation | `admin_tools.py` | The shared `elevation` lib (manifest execution level + elevated relaunch) |
| First-run wizard | `settings_management.py:67-128` | GUI settings dialog |
| Settings transfer between versions | `settings_management.py:130-211` | Settings versioning in the GUI |
| Interactive settings editing | `settings_management.py:23-66` | GUI form fields |
| Comment-preserving INI parser | `commentedconfigparser` dependency | Not needed; GUI manages settings |
| Step-by-step run modes | `compiler_main.py:72-93` | CLI subcommands for automation |
| Log username cleaner | `log_username_clean.py` | Trivial, optional |
| `pes_uniparam_edit` standalone tool | `pes_uniparam_edit.py` | Covered by `uniparam` format parser |

### Size deltas in shared modules

Red's shared modules are larger than Blue's due to referee-related edge cases and additional
validation:

| Module | Red | Blue | Delta |
|--------|-----|------|-------|
| `export_move.py` | 31KB | 23KB | 8KB (referee path logic) |
| `bins_update.py` | 18KB | 12KB | 6KB (more bin-packing logic) |
| `fmdl_editing.py` | 19KB | 12KB | 7KB (`fmdl_texture_paths_change` for referees) |
| `xml_editing.py` | 22.2KB | 14.8KB | 7.4KB (more template manipulation) |
| `export_check.py` | 44KB | 41KB | 3KB (minor) |

These deltas represent logic that must be accounted for in the Rust port. Use Red as the reference
for completeness.

---
