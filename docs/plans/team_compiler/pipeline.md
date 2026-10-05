# 4cc Studio — Team compiler plan: Pipeline

Part of the [Team compiler plan](README.md). Section headings are unchanged from the single-file plan, so an existing pointer to a section still names it; only the file part of the pointer changed.

## Per-export pipeline walkthrough

This section specifies, step by step, what happens to a single export. The steps describe *behavior
to reproduce*, not code to translate (see `README.md` "Relationship to the older compilers"): the implementation
operates on the object model, and only the game-facing output must match Red's. Red module names
(`Engines/python/`) mark where each behavior is authoritatively specified; Blue module names
(`lib/python/pipeline/` for orchestration and `lib/python/processing/` for processing steps) mark
where the pipeline shape it maps onto was prototyped. The structure is adapted throughout to the new
export format.

### 1. Reader

1. **Discovery** — the exports folder (`exports/`) is scanned for directories and
   `.zip`/`.7z` archives (the extension compared case-insensitively; other files are ignored),
   taken in the order of their source names (`vtree::fold_name`), and each source's
   `ExportStarted` names it by its file name, extension included, so a folder and an archive
   sharing a stem stay apart in every console line. Exports containing a `NO_USE` or
   `NO_USE.txt` marker file at the source's own root (before nested-folder normalization; the
   name compared case-insensitively) are skipped with
   `export_disabled`; they do not participate in duplicate-ref detection or run planning, are
   omitted from the grid, and produce only an informational log entry. In the GUI this scan is
   continuous (folder watcher, see `gui.md` "Live validation"); in the CLI it runs once. (Blue:
   `reader.exports_find`)
2. **Export display name and team name** — `export_display_name` is the complete archive/folder stem
   and is used only for source identity and presentation. The first non-empty token of that name
   (split on `.`, whitespace, `-`, `+`, `_`; a name with no token is rejected) is lowercased and
   wrapped in slashes to form the canonical `team_name` (`"/" + lowercase(first token) + "/"`, for
   example `co - Spring 2026.zip` → `/co/`). Lowercasing is Unicode `str::to_lowercase`, applied
   identically to the teams list's Name column on load — the same fold on both sides, which is
   what Blue's `str.lower()` comparison did (Python's `lower()` is Unicode-aware too); the
   distinction is theoretical for board names, which are ASCII, but the rule is written so nobody
   re-derives it. `team_name`,
   **not** the complete export name, is the lookup key in `teams_list.txt` and the `/xx/` label
   shown by the compiler. If the key is `/refs/`, the export is routed to referee processing;
   referee exports use the game's fixed numeric ID 999 internally, but 999 is not a valid normal
   `TeamId`. At most one non-disabled refs export may participate in a run: the Reader already knows
   every source's `team_name`, so it detects duplicate refs exports at discovery time — each
   conflicting export is skipped with `multiple_ref_exports` and never enters validation or
   planning. If the key is `/balls/`, the export is not team content at all — it is skipped with an
   info message (`export_balls_skipped`) pointing at the [Balls compiler](../balls_compiler.md), which
   owns balls exports; `/refs/` and `/balls/` are both reserved keys.
3. **Memory acquire** — the memory budget works at **task granularity** (a model folder, a kit, the
   logo, …), improving on Blue's per-export all-or-nothing accounting: a task's budget share is
   acquired when its contents load and released as soon as the writer has written its packed
   entries. Content shared by a player's face/boots/gloves tasks stays charged until all three have
   been written. Peak memory is bounded by the tasks in flight, not the exports in flight.
   Granularity is also limited by the container: plain-folder exports (the main DLC workflow) and
   `.zip` archives support lazy per-folder loading (direct file reads / per-entry random access);
   solid `.7z` archives must be decompressed in one pass, so a 7z export is resident as a whole
   until its folders are written, and the budget accounts for its full size. An oversized
   acquisition (`size > cap`) must wait specifically for `in_flight == 0` and then proceed alone; it
   must not wait for ordinary capacity, which can never satisfy it. (See the core plan's Parallelism
   section; its semaphore pseudocode must preserve this oversized-request branch.)
4. **Load** — the export's **structure** is always loaded eagerly (folder tree, file names/sizes,
   roster, link files — what validation, the grid, and ID assignment need; kilobytes); file
   **contents** load lazily per folder into the `VirtualTree` as their processing tasks start
   (native `sevenz-rust2`/zip parsing; `.db` and `.ini` files ignored). No mutable extraction or
   patch-staging tree is used: export transformation happens in memory between source reads and CPK
   output. Final CPK output may use temporary files for atomic write-and-rename. **Source exports
   are read-only**: the compiler never writes into an export folder or archive — every auto-fix
   (nested-folder flattening, generated kit configs, derived kit colors, …) exists only in the
   in-memory tree and is reported as a message, so the user's submission stays exactly as they made
   it.

### 2. Per-export serial steps (coordinator)

1. **Root normalization** — a folder is a **usable export root** when it directly holds one of the
   export's content folders (`Players`, `Kits`, `Faces`, `Boots`, `Gloves`, `Portraits`, `Common`,
   `Collars`, case-insensitive) or a root `logo*` image; root metadata alone (`colors.txt`,
   `notes.txt`, `players.txt`, `README.txt`, …) does not make one. If the export root is not
   usable, its child folders are scanned for a usable export root; exactly one candidate is
   flattened in memory (`nested_folders_fixed`) with loose root files preserved (an outer
   `notes.txt` beside `wrapper/Players/` stays the export's `notes.txt`). Separately, a content
   folder whose only entry is a folder of its own name that itself holds only folders
   (`Players/Players/03 - A/`, `Kits/Kits/p1/`) is flattened one level too, reported with the same
   `nested_folders_fixed`; an inner folder holding files is content, not a doubled layer
   (`Boots/Boots/boots.fmdl` is the shared boots folder `Boots`). **Intentional deviation from Red**
   (whose `team_id_get.py` takes the first matching child): several usable candidates are ambiguous
   and reject the export (`nested_root_ambiguous`), and a loose root file colliding with a flattened
   file at the same virtual path rejects it too (`nested_root_conflict`) — consistent with the
   plan-wide rule that collisions are rejected, never silently resolved.
2. **Validation, in two passes** — validation is read-only, and its expensive half, the deep
   pass, runs on the worker pool like compilation ("Deep format pass" below). Every finding is
   independent and per scope, and the report keeps the export's file order whatever the
   scheduling.
   - **Structure/roster/path pass (`aesthetics_export`)**: operates only on the eager descriptors and
     small required metadata. It validates virtual-path safety, folder and file naming, root shape,
     roster syntax and identity, link targets, source-format pairing, extension-based allowlists,
     and **texture stem conflicts** (two image files with the same stem but different extensions in
     one model folder — `texture_stem_conflict`), without loading model, texture, or archive
     payloads. This is the pass used by shallow archive checks.
   - **Deep format pass (format crates)**: runs over the sanitized export the structure pass
     leaves, before identity and planning. The Team compiler materializes each scope under the
     memory budget and delegates FMDL, `.model`/MTL/XML, glTF, texture, kit-config and
     `settings.toml` parsing to their owning crates; what they find goes back to
     `aesthetics_export` as content findings, and the sanitized export is derived again with
     them ("Content findings" in the [Aesthetics export
     plan](../aesthetics_export/object_model.md)). So a content finding follows the structure
     pass's rules: it drops the folder or file it names before any ID is planned for it, a
     dropped shared folder or `Common/` file takes the players linking it down, and
     `pass_through` keeps what is eligible. A failure only processing can meet (a conversion,
     a merge, a packing) stays a task failure: it drops the task's unit and cascades nowhere.
     Plain-folder full checks run both passes; packed archives run the structure pass during live
     checking and this deferred deep pass at compile start before eligibility is decided. The CLI
     `check` has no live check and runs both passes on every source kind, archives included, so a
     model error the next `compile` would report is never missed by the `check` before it.

     The deep pass runs on the run's worker pool. Within an export
     its model folders (player and shared) and `Common/`'s files are checked in parallel, and
     so are the files of one folder, each worker reading its own file; the few files left
     (the `Portraits/` files, the kits, the logo) are checked in order. The findings are
     collected in the export's file order. A folder source's reads are independent; an archive
     is one handle behind a lock (a `.zip` inflates one entry at a time, a `.7z`'s reads copy
     out of its one decompressed buffer), so there only the checks run in parallel. The file a
     worker holds is not charged to the memory budget: at most one per worker thread is held
     at a time, and a `.7z`'s buffer is charged as before. Exports held as folders or `.zip`
     files are also checked in parallel with each other: nothing in their check waits for
     memory, and a cup's exports are nearly always compiled from plain folders. `.7z` exports
     are checked one after another, once the others are done, because a worker waiting for
     its folder's files takes other work, and a `.7z` export's check started that way can
     wait for a permit held by the export suspended below it on the same thread, forever.
     Each source's findings are reported in discovery order either way. Routing only lists
     each source (a folder's tree, an archive's header), so the sources are routed in
     parallel and no archive is decompressed for an export routing sets aside. A source's
     small metadata is read by its check, through the handle the deep pass then reads its
     contents from: a `.7z` is decompressed once for the metadata, the structure pass and the
     deep pass, under one permit released when the deep pass ends. Measured
     at 4.7 on two VGL26 exports of 0.6 and 1 GB, the serial pass took 0.5 to 0.8 s of a 1.7 to
     3.1 s folder compile: most of it model parsing on one, whole-file reads of 8192-pixel
     textures (for their headers) on the other.

   Together the two passes cover these categories, adapted from Red's `export_check.py` to the new
format:
   - **Nested folder fix**: `Players/Players/` and similar are auto-flattened with a warning.
   - **Player folders**: normal teams are numbered via the folder name (`NN - Name`, 01–23) or
     authoritative root `players.txt` mapping (01–23); referee exports require `players.txt`
     mappings in 01–35 and may map the same folder to several slots (see "Player numbering").
     Duplicate slots are rejected; unnumbered-and-unlisted folders are rejected; model files are
     categorized (face/boots/gloves) and validated per mode — Fox: allowed FMDL names (`face_high`,
     `hair_high`, `oral`, `fcl_hair`, `boots`, `glove_l`, `glove_r`); pre-Fox: `.model`/`.mtl`/XML
     structure checks (`face_edithair.xml` and `hair.xml` rejected).
   - **Shared folders** (`Faces/`, `Boots/`, `Gloves/`): same model checks, including glTF-only
     material files (`materials.toml` catch-all and `*.materials.toml` name-matched; see the
     [Unified model format plan](../model_format.md)'s "Material files"); every folder must be referenced by at least one
     player link file (warning if orphaned; error if a link references a missing folder).
   - **Root files**: `colors.txt` and `notes.txt` are optional. `players.txt` is optional for normal
     teams but required for referee exports; when present it must satisfy the shared roster grammar
     and mapping rules. A present `colors.txt` must parse, and a non-empty `notes.txt` must be valid
     UTF-8 (an optional UTF-8 BOM is stripped). Unexpected root files are warned about; a root
     `README.txt` (case-insensitive) is known and ignored — the Team creator writes one, and human
     readme files in exports predate it. Referee exports additionally permit `ref_marker.dds` and `ref_lists.txt` (the Refs arranger's file
     for the Fox referee hook; not read, not packed). A missing referee roster reports
     `players_txt_missing`; the invalid `ParsedAestheticsExport` remains available to the Refs arranger
     for repair.
   - **Kits**: kit folder names must parse as `<slot>[ - <label>]` with the slot in `p1`–`p9`,
     `g1`, `all` (`kit_folder_invalid`), one folder per slot (`kit_slot_duplicate` discards both);
     `all/` textures are merged into every kit lacking the stem (`kit_textures_inherited`), and
     `all/` itself may hold nothing else (`kit_all_file_ignored`) and must have kits to serve
     (`kit_all_unused`) — grammar and merge in "Kits" in the [Aesthetics export
     plan](../aesthetics_export/README.md); a missing `config.toml`
     is auto-generated from the template (`kit_config_generated`); a missing `colors.txt` has its
     two menu colors derived from the main kit texture (`kit_colors_derived` — dominant-color
     extraction via `libs/color_tools`, see the [library crates plan](../libs/README.md)), and a present file
     that yields fewer than two valid colors falls back to the same derivation after its
     `color_entry_invalid` warnings; a kit with no derivable colors gets the loud magenta/black
     "no colors chosen" pair (`kit_colors_missing`, W); a kit whose effective textures lack
     `kit.dds` — an empty folder included — is a **placeholder kit** (`kit_placeholder`, I): it
     gets the bundled checkerboard texture and is otherwise a normal kit (see "Kits" in the [Aesthetics export
     plan](../aesthetics_export/README.md)); a kit holds at most one `icon_<N>` marker, numbered 0–23 (`kit_icon_invalid` warns and
     falls back to the default); at most one layout marker (`pre-fox` / `fox`, `.txt` tolerated;
     both → `kit_layout_conflict`, kit discarded); textures validated as DDS and named with the `kit` prefix
     (`kit.dds`, `kit_mask.dds`, `kit_chest.dds`, …); supplied configs' FPC fields are checked
     against the team's FPC status (reconciled at compile time — see "FPC toggle" in the [Aesthetics export plan](../aesthetics_export/fpc_toggle.md)).
   - **Portraits/Logo/Common/Collars**: portrait naming (`player_NN.dds`, NN in 01–23), root logo
     files (stem grammar `logo[_small][_<fit>]`, one file per role, main present whenever
     `_small` is, format in the raster allowlist — see "Root files" in the [Aesthetics export
     plan](../aesthetics_export/README.md); decodability is checked in the deep pass), texture integrity
     (`texture_check`), file-type allowlists per mode. Collar findings map to the team-name cell (collars have no cell
     of their own).
   - Processing consequences follow each message's effective disposition: `DropFile` removes only
     that file, `DropSlot` removes only the invalid normalized roster assignment, `DropFolder` skips
     that folder or equivalent atomic owning task scope, `DropExport` blocks the whole export, and
     `AbortRun` terminates the run. `pass_through` overrides only eligible content-level
     `DropFile`/`DropFolder` dispositions; foundational path, roster-identity, and required-metadata
     failures are never overridden. See `gui.md` "Cell states" for GUI outcomes.
3. **Export identity resolution** — after validation, a normal export's canonical `team_name` from
   the Reader is looked up in `teams_list.txt` — one row per team, tab-separated `ID` and `Name`
   columns, Name lowercased on load with the Reader's fold (see step 2 above), further columns
   ignored (Red's `MinBootsID`/`MaxBootsID` are dead: Studio derives the 40-ID block from the team
   ID, see the export plan's "ID allocation"); rows whose Name is not slash-wrapped (Red's `Backup 1`, `VGL 55`
   placeholders) load but can never match. (The GUI also performs an early lookup after structural validation so
   the live grid can show or edit the ID immediately.) The complete `export_display_name` is never
   used as the lookup key. Valid normal `TeamId` values are 701–920. In Red, an unresolvable team
   prompts interactively; in 4cc Studio the grid's **team ID cell** turns red and editable — the
   user types an ID, confirms with Enter, and it is written to the teams list (see "GUI: the
   live-validating team grid"). In the CLI an unresolvable normal team is a hard error. A successful
   lookup produces `ExportIdentity::Team { id, name }`; a referee export produces
   `ExportIdentity::Referees` without entering the normal teams list. The referee pipeline renders
   the game's fixed numeric ID 999 only at game-format boundaries. Either identity is wrapped in
   `ResolvedAestheticsExport` before run planning. (Red/Blue: `team_id_get.py`)
4. **Portrait extraction** — for `ExportIdentity::Team { id, .. }`, a player folder's `portrait.*`
   (any accepted image; a non-DDS source is encoded as "Portraits" in the export plan says) is
   renamed (`player_{id}{NN}.dds` for PES ≤18, `{id}{NN}.dds` for 19+; in Red
   `portraits_move.py` stages face-folder portraits under the `player_` name and `export_move.py`'s
   Portraits pass applies the version-specific final name) and staged with the portraits; an
   `ingame_face` marker removes face-classified `ModelPart`s and face-specific ancillary files, and
   **no face folder is emitted at all** — on either engine, the presence of any face folder (even
   one containing only boots/gloves models) overrides the ingame-customized face, and an empty face
   folder makes the face blank. Explicitly-named face models (`face_high`, `hair_high`, `oral`)
   combined with `ingame_face` drop the whole player folder (`ingame_face_explicit_face_model`);
   arbitrarily-named models (which would route to `fcl_hair`) reroute to the boots folder as `boots`
   (renamed or merged), carrying their paired SKL. The remaining non-face parts are
   relocated out of the face folder: gloves parts go to **player-specific gloves folders**
   (`glove_l`, `glove_r`), and the rerouted arbitrary-named models plus any explicit boots models
   are **merged into one boots model** (on Fox via `fmdl`'s mesh merging; on pre-Fox via
   `pes_model`'s native merge over the `.model` + `.mtl` pair) and moved
   to a **player-specific boots folder** — the one case where pre-Fox produces
   player-exclusive boots/gloves folders with IDs from the per-team block scheme (see "ingame_face
   marker"). Without `ingame_face`, a player with boots/gloves but no face models still gets a
   (blank) face folder: the absence of the default PES face is the final element of FPC, so the
   folder must exist (Red's current behavior, kept). That holds for every roster-mapped player
   folder without the marker and without a face model, whatever else it holds: a portrait
   alone, textures alone, a boots link alone, an empty `face/`, nothing. On Fox the blank folder is
   `face/real/{id}{NN}/#Win/face.fpk` holding only the template `face_diff.bin`, plus `face.fpkd`
   (the template `generic.fpkd`): no model, no `.skl`, no `.fclo`. On pre-Fox it is
   `face/real/{id}{NN}.cpk` holding a `face.xml` with one
   `<model level="0" type="face_neck" path="./oral_dummy_win32.model" material="./dummy.mtl"/>` and
   a `<dif>` from the template `face_diff.bin`, plus `oral_dummy_win32.model` (the template
   `dummy.model`) and `dummy.mtl` (an empty `<materialset>`). The blank folder's diff is always
   the bundled template, whatever `face_diff.bin` the player folder holds: without a face
   model the file has no face to shape, and the two other files found in such folders
   across the VGL26 exports are outdated FPC-dedicated diffs. A face file left in a player
   folder with no face model, with or without the marker (`face_diff.bin`, `face_diff.xml`,
   `fcl_hair_sim.fclo`), is not read and is reported as `face_file_not_used`, by `check` too:
   an Info, since the output is what it would be without the file, and 467 face folders of the
   VGL26 exports hold one. Under the marker the reroute itself reports nothing: the marker is
   the member's own instruction. Conflicting portraits for the same
   number fail the export. Referee identity uses its separate slot-derived face paths and does not
   construct a normal `TeamId`. (Red: `portraits_move.py`, `export_move.py`)
5. **Notes collection** — a root `notes.txt` is strict UTF-8; an optional UTF-8 BOM is stripped and
   CRLF/CR newlines are normalized to LF. Invalid encoding reports the file-scoped
   `notes_encoding_invalid` and drops only the note. Empty or whitespace-only notes produce no
   entry. Valid non-empty content is stored as a manifest-staged non-model artifact under a team
   header, replacing Red's "Other Notes" extraction from the Team Note txt. `teamnotes.txt` is
   rendered only after final export outcomes are known: it is a deterministic UTF-8/LF run artifact,
   atomically replaced, ordered by canonical export order, and contains only accepted exports with
   non-empty valid notes. (Blue: `note_txt_append`) An *accepted* export is one run planning
   keeps, the set whose team colors are applied. The note's text is read while the export's
   source is open for validation, as the root `colors.txt` is, so no task reads it. The file
   is one entry per export: a header line `--- /co/ ---` (the export's team name), then the
   note, its line ends made LF and its leading blank lines and trailing whitespace removed,
   then one empty line between entries; the file ends with one LF. It is written (to a temporary file
   beside it, then renamed) once the run's CPK is in place, so a run that writes no CPK leaves
   the previous file as it was. A run that writes its CPK and has no note to collect removes a
   previous `teamnotes.txt`, as Red reset the file at every run: a file left over would show
   the notes of exports this run did not compile. A failure to write or remove it is
   `teamnotes_write_failed` (E, on the run), after which the CPK stays in place.
6. **Player folder categorization** — each player folder is categorized into optional
   face/boots/gloves inputs for run planning; any category may be absent (for team players and
   referees alike — the unified player folder format has no separate face-folder concept, so a
   player folder with only boots/gloves is structurally valid). Run planning turns these inputs
   into model tasks. Normal teams receive
   boots/gloves IDs from the **hardcoded per-team block scheme**; referee tasks receive their fixed
   slot-derived `k99XX`/`g99XX` IDs instead (see "Player folders" in the [Aesthetics export plan](../aesthetics_export/object_model.md)). Assigned IDs
   are written, on Fox, into the CPK's own `BootsList.bin`/`GloveList.bin` rows, so the CPK carries
   them; on pre-Fox into the savefile, so a compiled team CPK and its savefile must be deployed
   together, and the aesthetics patch written beside the CPK is how they travel together when the
   savefile is on someone else's machine (see "Post-processing"). A missing local savefile stays a **warning** that
   skips only the savefile step: the patch is still written, and because allocation is
   deterministic (fixed per-team blocks, exclusive IDs by player number, shared IDs in alphabetical
   order), recompiling the same exports later with a savefile configured produces identical IDs, so
   a CPK-now/savefile-later split remains consistent. The deploy-together rule is distribution
   guidance, not a compile blocker.

### 3. Per-model-folder parallel steps (rayon)

After validation and export-identity resolution, a serial **run-level** planning phase resolves
`duplicate_aesthetics_export`, shared/Common dependencies, deterministic IDs, cross-export and known-path
collisions, `overrides/` precedence, canonical export order, and staged non-model artifacts into an
immutable **build manifest**. Its tasks cover model folders and the non-model work (kits, logo,
Common, collars, portraits, referee marker) alike; each task is an atomic unit — it commits whole or
not at all. Canonical export order uses normalized source-relative path plus source kind as its
stable key, never the potentially duplicated `export_display_name`: it is discovery's order, by
the source's file name (extension included, so a folder and an archive of one stem stay apart),
case-folded and then as spelled. `duplicate_aesthetics_export` is found before planning, by the
validation pass once every export's identity is resolved, so `check` reports it as `compile`
does; planning never sees the conflicting exports. The manifest allocates every
task an output namespace that cannot conflict with another task's. A cross-export collision drops
the losing task or export rather than an arbitrary entry that could leave its model/XML/package
incomplete; an override stays path-granular and intentionally supersedes the export
entry. No preflight over the tasks' paths is built: no two tasks can claim one path today
(every path is keyed by the team's ID, a player ID, a boots or gloves ID of the team's block,
a kit name or a player folder's name; two exports of one team are refused; the collisions
inside one export are refused or merged before its tasks are planned; collars have
`collar_id_conflict`), so the folder and export forms of `duplicate_path` have no trigger,
and the writer's duplicate invariant is the backstop. A step that adds an output two tasks
can claim adds the check with it. Folder-internal names that depend on deep parsing — glTF material-role image names, model
fallback/merge products — are resolved deterministically inside that task and checked for collisions
within its allocated namespace. All identity, allocation, roster, portrait-conflict, and run-global
collision checks finish before tasks are dispatched. Deep task failures may be at most `DropFolder`
unless the writer can roll back the whole export. Each planned face/boots/gloves model folder is
then processed as an independent parallel task (Blue: `coordinator._model_folder_task`, Red:
`export_move.py`):

1. **Format conversion** — when several source representations coexist (e.g. `boots.fmdl` +
   `boots.model` + `boots.glb`), selection is **target-native first, then glTF, then conversion
   from the opposite native format**; duplicate sources within the selected representation are
   ambiguous and drop the folder (`model_source_ambiguous`). If the selected model's format doesn't
   match the target PES version, convert via the IR (see the [Model conversion
   plan](../model_conversion/README.md)). Until glTF support lands (Phase 7, after Release
   0.1.0), a selected glTF representation drops the folder with an error rather than falling
   through to the opposite native format, so the same export never compiles differently once
   glTF is read.
2. **ID replacement** — dummy team IDs are replaced in file contents (FMDL texture path tables via
   the `fmdl` crate's texture-path rewriting, `ops::paths::rewrite_texture_paths`; `.mtl` texture
   IDs pre-Fox) and in file names.
3. **Fox mode fixups** — FMDL files renamed by stripping prefixes to the allowed names (arbitrary
   names route to the `fcl_hair.fmdl` merge — Red's fallback generalized), and all models resolving
   to the same allowed name — including `.common` links to Common models, which Fox cannot load at
   runtime — **merged into one FMDL** (see "Common model links and model merging" in the [Team
   export plan](../aesthetics_export/README.md)); missing template files injected (`face_diff.bin`, `fcl_hair_sim.fclo` when
   `fcl_hair.fmdl` exists, and the default `boots.skl`/`fcl_hair_sim.skl` skeletons from template
   `body.skl` when no part brings a custom SKL for that slot — see "SKL pairing" there); `face_diff.xml`
   converted to `face_diff.bin`.
4. **Pre-Fox fixups** — `face.xml` created, with every model of the player folder listed under
   its category's type attribute (boots and gloves models included — the typed XML is why pre-Fox
   needs no per-player boots/gloves folders), or a user-supplied one checked and re-serialized
   (see "User-supplied `face.xml`" under the XML/MTL checks); `glove.xml` for shared gloves folders;
   `.common` links resolve to Common paths in the generated XML (Red's common-link behavior). (Red's
   3-digit glove ID padding, `g567` → `g0567`, has no equivalent: the Studio format has no ID-named
   folders — IDs are auto-assigned. The Export upgrader normalizes old IDs when parsing them.)
5. **Texture conversion** — all image formats are interchangeable as texture sources (see the
   [library crates plan](../libs/README.md)): DDS, FTEX, PNG, JPEG, BMP, WebP, TGA, TIFF. Material
   references (FMDL path tables, `.mtl`, `.materials.toml`) name textures by **stem** (filename
   without extension); the compiler resolves each stem to the actual file in the model folder
   and converts according to the selected PES version, not just the Fox/pre-Fox split. BC7 is
   supported on PES 19–21 only: PES 15–18 transcode BC7 and encode raster sources to BC3, with
   BC1 for eligible fully opaque color textures under the library plan's alpha/role rules.
   PES 19–21 retain supported BC7 inputs and use BC7 for newly encoded ordinary raster textures.
   Role-specific normal/data handling still applies; already-compatible blocks avoid re-encoding.
   A texture's role comes from its stem: one ending in `_nrm` (any case) is a normal map, any
   other is color, the suffix of the role table in the [Unified model format
   plan](../model_format.md) and of the game's own maps (`skin_nrm`, `oral_nrm`, `dummy_nrm`).
   It is not read from the sampler of the model naming it: a stem is converted once per folder,
   whichever models and samplers name it, by a task that reads no model.
   Raster sources decode via `image`, missing mipmaps are generated, and Fox output uses FTEX with
   target-appropriate headers. Two image files with the same stem
   but different extensions is a conflict (`texture_stem_conflict`). Optional zlib DDS
   compression (PES ≤17). An in-memory conversion cache eliminates repeat conversion on the
   edit→compile→test loop; see the [library crates plan](../libs/README.md) for design and engagement
   rules (≤2-team limit, no disk persistence).
6. **Texture relocation to common** (player folders only) — **all** textures of a player folder's
   own models (and a referee's), whether used by one of its models or several, are moved to a
   **per-player common subfolder** in the packed output, and the texture paths inside the FMDL/MTL
   files are rewritten to match. (The destination is fixed at player-folder split time — the
   player's textures are set aside for the common subfolder and emitted once; each parallel model
   task only rewrites its own model's paths. A folder mapped to several team or referee slots still
   emits its textures **once**, into a single common subfolder keyed by the source folder's name —
   every mapped slot's instantiated model references that one location. This is Red's
   in-game-verified referee layout: its preprocessing runs once per distinct referee folder,
   rewriting texture paths to the name-keyed per-referee common subfolder — MTL/XML to
   `model/character/uniform/common/{team ID}/{folder name}/` (packed under `common/character1/`),
   Fox FMDL to `/Assets/pes16/model/character/common/{team ID}/{folder name}/sourceimages/` — and
   copying the folder's common content once, before the per-slot instantiation pass. Referees are
   the team with ID 999 here, nothing more: the same path shape with `999` where a team's ID goes —
   Red's source spells the placeholder `XXX`/`000` before substitution. Textures referenced in place
   from the export's `Common/` folder sit one level up, `…/common/{team ID}/` (pre-Fox) and
   `…/common/{team ID}/sourceimages/` (Fox), which is Red's Common location on each engine.) The
   shared textures commit
   once, after all of the player's face/boots/gloves tasks report: identical destination bytes
   deduplicate, while the same destination with different bytes reports `shared_texture_conflict`
   and drops the losing task, chosen by canonical order (`face` > `boots` > `gloves`), never rayon
   completion order. The player's texture sources are its own folder (its reserved subfolders included:
   they share its texture namespace, where one stem twice is `texture_stem_conflict`) and each
   shared face, boots or gloves folder it combines, and a source counts for the package it feeds: the player's own folder and a
   combined face folder for the face, a combined boots (gloves) folder for the boots (gloves).
   Stems compare case-folded. Two sources of different packages holding one stem with different
   bytes is `shared_texture_conflict`, the lower package dropped with every texture only its
   sources hold; two sources of one package is `merged_texture_conflict`, since the one model
   they build has no winner, and drops the folder. Planning cannot tell which model names which
   texture (it reads no model bytes), so a source stands for its package. The writer commits a player folder's tasks as one group: it holds the
   folder's face/boots/gloves batches until the folder's textures batch arrives, then commits the
   packages that succeeded and the textures; if the textures fail (`folder_pack_failed`), none
   of the folder's packages commit, since each would point at a texture the CPK lacks; if every
   package failed, the textures stay out. Rationale: a single, predictable location for every player's textures — no
   per-user-count special cases — and textures shared between the player's face/boots/gloves are
   packed once for free. **Shared model folders normally keep their textures with their own ID-based
   shared-model output.** When Fox combination instead merges a shared model into a player-exclusive
   FMDL, the shared folder's textures are copied to the player's common subfolder (all of them, not only
   those the merged part names: planning reads no model bytes, and a shared folder's textures are
   its models') and the merged FMDL's paths are rewritten; a shared output that is still used plainly keeps its own
   texture copy. This is compiler-internal (the export format has no common concept; the models' own
   texture references are the ground truth) and generalizes Red's referee-only common preprocessing
   (`referee_tools.py`) to every player. **Textures that resolve into the export's `Common/` folder
   are the exception: they are never relocated.** Whether reached through a texture link
   (`hair.png.common`), a stem set in a Common material file, or a Common-linked model's own
   textures (merged into the player's FMDL on Fox or not), such a texture is packed once in the
   team's Common output (the Common row of the Game paths table) and the model's path is rewritten
   to point there — Red's `common/XXX/` handling with the team ID substituted, on both engines.
   Relocating them per player would defeat the reason a texture is in Common (one file for many
   players), and Common is a location the game already reads on both engines, so nothing is gained
   by moving it. The rule in one line: *resolved in the player folder → the player's; resolved in
   Common → the team's* (format side: "Link files" in the [Unified model format
   plan](../model_format.md)). **Intentional deviation from Red's packed layout for teams** (verified
   extensively in-game for players as well as referees, across versions) — see the parity test
   section.
7. **Packing** — pre-Fox faces are packed into a per-model nested CPK; pre-Fox boots/gloves emit
   loose files; Fox mode packs allowed types (`.bin`, `.fmdl`, `.skl`, `.fclo`) into an FPK plus a
   template `.fpkd` (player-owned and merge-copied textures having been relocated to the per-player
   common subfolder; plain shared-output textures remain in that model's own texture location).
   Packed entries are emitted to the writer queue. (Blue: `contents_packing.py`)

### 4. Per-export non-model steps

These are logical per-export steps (Red: `export_move.py`, plus `bins_update.py` for its namesake
step; Blue: `export_move_non_model`). They are ordinary
manifest tasks and may run alongside model tasks once their dependencies are satisfied; this section
describes behavior, not a serial scheduling requirement:

- **Kits** — per kit (each `KitFolder` of the validated export; `all/` is not one): missing kit
  configs (`config.toml`) are generated from the
  template, retaining the template's five `[colors]` defaults and its FPC kit values (the
  template carries them whatever the team's FPC status); the kit's two-color `colors.txt` (grammar: "Root files", "Colors" in
  the [Aesthetics export plan](../aesthetics_export/player_folders.md)) feeds only `UniColor.bin`,
  not the config's distinct five RGB fields. Supplied configs' FPC fields are reconciled with the
  team's status (see "FPC toggle" in the [Aesthetics export
  plan](../aesthetics_export/fpc_toggle.md)); the kit's *effective* textures — own files plus the
  `all/` files it inherits, already merged by `aesthetics_export` — are renamed by replacing the
  `kit` prefix with the kit's ID — `u0{team_id}{slot}` (e.g. `kit_chest.dds` in team 701's `p2/`
  becomes `u0701p2_chest.dds`; an inherited `all/kit_back.dds` becomes `u0701p1_back.dds`,
  `u0701p2_back.dds`, … one per kit, since the game has no shared kit textures) — and converted.
  Each kit task reads a shared file itself: kit tasks stay independent, and the game-facing output
  is byte-for-byte what a copy in the kit folder would give. Two textures the game requires are
  **filled from bundled templates** when the effective set lacks them: `kit.dds` from the
  placeholder kit texture — a placeholder kit, reported once (`kit_placeholder`) — and, for pre-Fox
  targets only, `kit_mask.dds` from the mask template, silently, as Red's `kit_masks_check` did
  (most kits ship no mask: 70 of 641
  surveyed). **Mask and srm are engine-specific, not two names for one map**: pre-Fox reads
  `{kit}_mask`, Fox reads `{kit}_srm`, each by name convention only — the config's five texture
  fields (main, back, chest, leg, name) never name either, in any game (all 788 PES 17 and 1359
  PES 21 stock configs checked; the community converters' "rewrite `_srm` to `_mask` in the
  config" touches the `chest` slot and is a no-op). Their channels mean different things (stock
  PES 17 masks average (148,133,13), stock PES 21 srms (75,141,0): Fox packs
  specular/roughness/metallic), so **neither is ever converted into the other** — a channel
  formula would be invented, not taken from anywhere. Rule: `kit_mask` is emitted for 15–17
  targets and `kit_srm` for 18–21; the one the target's engine does not read is **not emitted**
  and reported once (`kit_texture_not_used`, I); a pre-Fox target lacking a mask gets the template
  as above; a Fox target lacking an srm gets nothing — Red never made one, the game falls back on
  its own, and 0 of 365 kits in the maintainer's library ship an `_srm` at all (14 ship a mask).
  This is what the community converters do for Fox→pre-Fox (drop the srm, write a flat
  (150,130,0) mask — Red's template is flat (151,130,0), the same default) and the mirror of it
  for the direction none of them handles. Dropping a `_mask` on Fox targets is an **intentional
  deviation from Red**, which converts it to FTEX and ships a file the game never opens. The placeholder is the
  **magenta/black checkerboard** — the Source-engine "missing texture" pattern every 4cc viewer
  recognizes, so a placeholder that does get rendered (a stray non-full-body player, the GK) is
  read as exactly what it is, not as a black kit someone chose. 64² DXT1 with a full mip chain
  like Red's mask template (~3 KB, so a full-body team's nine placeholder kits cost nothing), with
  8-pixel checks so the two-color 4×4 DXT1 blocks encode it losslessly; it passes the kit-texture
  checks by construction and is never a source for color derivation. **Layout conversion**: when
  the kit's layout marker names the other engine than the target (`pre-fox` compiled for 18–21,
  `fox` for 15–17 — see "Kit layout marker" in the [Aesthetics export plan](../aesthetics_export/README.md)),
  the kit's emitted textures that the uniform models map (the effective `kit`, and its `kit_mask` /
  `kit_srm` where the target takes one) are decoded, **re-laid out** and re-encoded before the
  rename/convert step, reported once per kit (`kit_layout_converted`, I). Only the two **sock
  islands** differ between the layouts: the shirt, sleeves, collar strip and shorts sit at the same
  texels in both. The re-layout is a fixed table of axis-aligned rectangle moves,
  `KIT_LAYOUT_REMAP`, two bands per sock island, in the units of a 2048-px texture and scaled to
  the texture's own size: on the left sock, pre-Fox u 8–168 ↔ Fox u 8–128 and pre-Fox u 168–448 ↔
  Fox u 128–376, over v 632–1160 in both; the right sock is the mirror image (u → 2048 − u). A
  band is resampled along u only, row by row, from its own source rectangle (Lanczos3, the
  compiler's one resampler; near the rectangle's edge the filter uses only the texels inside
  it), so rows never mix and a flat band stays flat. The Fox→pre-Fox table is the inverse of the pre-Fox→Fox
  one, so the two are one const read in either direction. Every texel outside the destination
  rectangles keeps its value (for pre-Fox→Fox that includes the strip the narrower Fox sock
  leaves, u 376–448, which no Fox model reads), and a block-compressed source keeps the very
  blocks no destination rectangle touches: the rectangles' outer edges are multiples of 8, so on a
  2048 or 1024 texture they fall on block edges and the rest of the kit is not compressed a
  second time. The same move is applied to every mip level the source carries, the rectangles
  scaled to the level and rounded to whole texels (a band rounded to no width is skipped).
  **The games' own uniform models (the base data CPKs) are the source of truth** for the table,
  read by `scripts/provenance/kit_uv/`. *Outlines* (`kit_uv_diff.py`): the sock islands span u
  8–440 pre-Fox and 8–372 in Fox (left; mirrored on the right), v 640–1152 in both; the shorts
  islands have one outline in both (u 16–644 / 1404–2032, v 1164–1948). *Socks*
  (`kit_uv_sock_angle.py`): the two engines' sock meshes are different meshes on bodies of
  different proportions, so texels are paired per row (v does not move: each row sits at the same
  fraction of the sock's length in both, within 0.03) by two measures that do not depend on the
  leg's size or place, the angle around the leg's axis and the fraction of the ring's arc length.
  Both engines put the seam at the same angle (about 112°, within 3°) and turn once between the
  island's ends. The Fox unwrap is uniform (each 16 px holds about 4.4 % of the ring everywhere)
  and the pre-Fox one is not (3.0–3.3 % at u under 100, rising to 4.1–4.2 % from u 256 on), which
  is why one scale does not fit: the map's slope rises smoothly from about 0.7 to about 0.93. Two
  bands approximate it: the table sits within 6 px rms of the angle pairs and 10 px rms of the arc
  pairs (one scale: 10 and 15; three sock variants, both islands). That residual is mostly real:
  the two measures differ from each other by up to about 9 px, and the map shifts by up to about
  10 px either way with v, which no rectangle table follows. *Shorts*
  (`kit_uv_pants_height.py`): not re-laid. On the shorts u runs along the body's height, and a
  nearest-3D-point match (`kit_uv_fit2.py`) showed Fox texels 36–60 px "lower"; that was the Fox
  body standing 33–44 mm taller at the hip (44–59 px of u), not the layout. In both engines the
  crotch notch and the waist sit at the same u (280 and 272, 640), each u holds the same fraction
  of the garment's height (within 0.01) and each v the same angle around the leg (within 6°). The
  two games only split the shorts between their model files differently (pre-Fox: `pants_*` up to
  u 460 plus a `_sub` strip; Fox: a `pants_*` hem strip up to u 184 plus a `_sub` body), which a
  texture does not see. No kit drawn by hand for both layouts exists in the community to check
  the table against; the nearest thing is PES Master's two kit creators (the PES 2017 one and the
  PES 2018-2021 one), which draw the same 21 brand templates for each layout from named layers,
  and it is a cross-check, never a source. It agrees: its shorts are identical in both layouts (20
  of 21 templates), and on the socks nothing moves along v while its two bands (u 8–195 onto
  8–137, then a shift of −60 px) sit within 15 px of the table. The table is lead-authored (it is
  a measurement) and lives with the kit step (`processing/kit_layout.rs`). Its test golden,
  `tests/fixtures/kit_layout/`, is where the models put each stripe of a striped kit, written by
  `kit_layout_fixture.py` from the models alone, never from the table; the table's stripe centres
  sit within 3.8 px of it. Placeholder textures are engine-neutral and are never re-laid out.
  `_chest`, `_back`, `_leg` and `_name` are glyph atlases the game reads cell by cell, not
  textures mapped through the uniform models, so the uniform table does not apply to them. Their
  own arrangement differs between the games' stock files (`kit_leg_atlas.py`: every stock PES 17
  `_back`, `_chest` and `_leg`, 797 of each, stacks its ten digits in a column, 128×2048 or
  64×1024, and every stock PES 21 one, 1,357 of each, lays them in a row, 2048×256 or 1024×128;
  `_name` is a 4:1 strip in both), and neither engine reads the other's. So a `_back`, `_chest`
  or `_leg` atlas in the other engine's arrangement is re-arranged into the target's when the
  kit is compiled: its shape alone (tall or wide) says which arrangement a file has, so no
  marker is involved, and `_name` is left as it is. The digit cells of the two arrangements are
  measured before this is built (worklog step 4.32); until then the atlases pass as they are.
  The TOML config is compiled to the game's 120-byte binary via `libs/kit_config`
  (texture-name fields derived from the effective texture set, version-specific bit packing applied —
  including the PES 15 shirt-pattern clamp; see the [Kit config editor plan](../kit_config_editor.md))
  and emitted under the game's kit-config name for its slot (old `XXX_DEF_1st_realUni.bin` pattern
  with the team ID applied). A supplied config's value that does not fit the target version is
  reported by the deep pass, so by `check` as by `compile` (`kit_config_version_clamped` on the
  file, kept, naming the field, the value and the version's maximum: `kit_config`'s
  `kit_value_out_of_range` and `kit_pattern_unsupported_pes15` findings), and clamped when the
  binary is emitted. `kit_config`'s other findings are not reported by the compiler yet
  (worklog "Issues").
- **Logo** — the game's three PNGs are *produced*, not passed through: the main `logo*` file is
  decoded (`image`, via `dds_convert`'s decoders), made square per its fit tag (`crop` /
  `stretch` / `fit`, default `fit`), resampled with Lanczos3 to 512² and 256², and encoded as
  PNG; the 128² one comes from `logo_small*` the same way when present, otherwise from the main
  image. Alpha is preserved (formats without it yield opaque logos). The three sizes are the same
  in every supported version (checked against PES 21's own files, not only ≤19-era exports). The
  three are named `emblem_0{team_id}_r_ll/_r_l/_r.png` (PES ≤19) or `e_000{team_id}…` (PES 20+) —
  Red's destination names unchanged, so the CPK layout is parity-identical. `_r_ll` is the 512²
  one, `_r_l` the 256² one and `_r` the 128² one (the one `logo_small*` feeds when present),
  measured on team 701's files in PES 21's `4cc_45_uniform.cpk`. A source smaller than its
  largest target is upscaled and reported (`logo_upscaled`); a non-square source made square is
  reported with the mode applied (`logo_fit_applied`). The three files are one atomic producer:
  if any input fails to decode, no logo is emitted for the team. The geometry, for a source of
  w × h and a target of N²: `stretch` resamples the whole image to N × N; `crop` cuts the long
  side to the short one around its centre (the cut starts at half the difference, rounded
  down) and resamples that square to N × N; `fit` resamples the image with its proportions
  kept, the long side to N and the short one to N × short / long (rounded half up, at least
  1), and places it centred (the offset rounded down) on a fully transparent N² canvas. `fit`
  resamples before it pads, not after: Lanczos over a padded image would blend the logo's
  edge with the border's transparent black and leave a dark fringe. A square source is
  resampled as it is and reports no `logo_fit_applied`, whatever its tag; a source already at
  N² is not filtered at all. Each target is resampled from the source, never from a larger
  target. A source is *upscaled* when either of its sides is stretched to reach the file's
  largest target (512 for the main file, 128 for the small one): the long side under it for
  `fit`, the short side under it for `crop` and `stretch`. Both findings are per source file
  (context `file`, with `mode` or the source's `size` and the `target`), reported on the
  logo's task, which is the export's one logo task (`processing/team_assets.rs`) and is
  scoped on the main file: a failure drops the logo alone. The PNGs are 8-bit RGBA, written
  by `dds_convert::encode_png`. A DDS or FTEX source gives its top mip level.
- **Collars** — the model files are passed through unmodified. These are custom collar models that
  replace one of PES's many stock collar models (the game's `nocloth` set); the compiler derives the
  replaced model's ID and sets it as the collar and the winter collar in all of the team's kit configs, which puts the
  custom model on every player at once — a quick alternative to per-player models. This automatic ID
  extraction and kit-config rewriting is an **intentional deviation from Red**, which passes collar
  files through without changing the configs. The replaced ID comes from the filename, which must be
  `collar_<ID>`, the ID with or without zero padding (`collar_12` and `collar_012` name the same
  collar); the file is emitted under the game's three-digit name, `collar_012.fmdl` on Fox and
  `collar_012.model` plus its `.mtl` pre-Fox, at the target's `nocloth` path ("Game paths
  reference"). A name that doesn't parse, or an ID that is not a stock collar of the target version
  (the per-version sets, counted in the installed games' base data CPKs, are in `messages.md`,
  `collar_id_invalid`), reports `collar_id_invalid`. Two teams replacing the same stock collar ID
  is an error: planning is serial,
  so a run-wide list of the IDs claimed so far catches duplicates without any advance cross-checking
  — the later claimant in canonical export order reports `collar_id_conflict` and its collar is
  discarded. The list starts with the suite's own two claims, 105 (the FPC collar — replacing it
  would break FPC teams everywhere) and 77 (the referees' marker collar, `blue_port.md` "Referee
  export processing"), so a team's file named for either is the same conflict, not an invalid
  ID: the ID is a real collar that someone else holds. Custom collars are **compatible with team FPC**: collar rewriting runs after FPC
  reconciliation, so the custom ID deliberately overrides the FPC collar value in the configs.
- **Common** — pre-Fox: `.mtl` texture IDs and relative→absolute path fixes; both modes: texture
  conversion, dummy ID replacement, `oral_`/`_win32` model-name prefixes, face XML references to
  renamed common models updated.
- **Kit-dependent assets** — `kitN`/`kit1`…`kit9` tokens in file names and written paths (FMDL
  path tables, `.mtl` sampler paths, `face.xml` model paths) pass through **verbatim**; the modded
  exes match that spelling directly, and the historical `u0XXXp0` magic is legacy that only the
  Export upgrader understands (the compiler has no finding for it — a leftover reference just fails
  the ordinary texture-existence checks). Variant sets are completed against the
  export's kit numbers (`kit_variant_missing`, lowest variant copied) and per-kit *model* sets
  collapse to one `face.xml` entry on pre-Fox or to the lowest variant on Fox
  (`kit_variant_model_fox`). Rules and rationale: "Kit-dependent assets" in the [Unified model
  format plan](../model_format.md). How the compiler finds a set: a *variant* is a file whose
  stem holds the token `kit1` to `kit9` (delimited by `_`, `-`, `.` or the stem's ends,
  spelled exactly so), and the files of one folder whose stems differ only in that token's
  digit are one set, whose *reference* is the stem with `kitN` in the token's place. A
  texture set is completed by the task that converts the folder's textures (a model folder's,
  or `Common/`'s): for each kit number the export defines (its `Kits/` folders `p1` to `p9`;
  `g1` is not a number of its own) that the set lacks, the lowest variant's converted bytes
  are emitted under the missing variant's name and `kit_variant_missing` is reported on the
  folder, once per set and number. The set is found from the files, not from the models'
  references: a set exists only to be referenced, and reading every model's path table in
  the textures task for the rare set nobody references would cost a second read of each
  model. An export with no kit folder defines no number, so nothing is completed. A model's
  texture path naming a reference (`pants_kitN`) is pointed at the folder's texture home
  when the folder holds any variant of that set, the file name kept as it is. On Fox a model
  file that is a variant with a lower variant of its set in the same folder is not compiled,
  and planning reports `kit_variant_model_fox` on the folder, once per set. The legacy
  `dummy_kit*` stems keep working as **reserved, game-substituted names**: the texture-existence
  checks (`mtl_texture_not_found`, `material_texture_not_found`, FMDL path checks) skip them and the
  path is emitted verbatim. Red's `dummy_kit_replace.py` — copying the team's kit 1 textures over
  `dummy_kit*` in Common as a fallback for the substitution — is **not ported**: that fallback dates
  from when the substitution lived in Sider scripts; it is in the exes now and stable.
- **Bins accumulation** — team colors (from the root `colors.txt`, when present: its first four
  valid colors, the `TeamColor.bin` record's capacity, slots it does not fill keeping the working
  bin's bytes) and per-kit colors
  (from each kit folder's `colors.txt`, or derived from the main kit texture when it's missing —
  shirt-region dominant color plus either the shirt's second tone or the shorts-region dominant;
  `libs/color_tools` — with the menu icon number from the kit's optional `icon_<N>` marker, default 3) are
  staged for in-memory copies of `TeamColor.bin` and `UniColor.bin` (fixed per-team byte offsets;
  Red: `bins_update.py`; the file's grammar is "Root files", "Colors" in the [Aesthetics export
  plan](../aesthetics_export/player_folders.md)). A kit's entry goes into its team's
  `UniColor.bin` record by kit number (p1 to p9 are 0 to 8, g1 is 0x10): it replaces the
  entry of that number or joins the others, and the record is written again with its count,
  its entries in ascending kit number and unused entries after them (`FF` and seven zero
  bytes, as Red writes them). That merge is a `Midcup` export's: a kit it does not hold keeps
  its entry, and a team with no committed kit keeps its record's bytes. A `Full` export
  (`aesthetics_export/object_model.md` "Coverage tag") rebuilds what the compiler holds for
  its team from the export alone, since the record's count is what makes the game offer a
  kit: the `UniColor.bin` record holds its committed kits and nothing else (the count theirs,
  every other entry unused), the team's kit configs in `UniformParameter.bin` that the export
  does not hold are removed, and no absent kit slot is FPC-patched, there being none. A
  `Full` export with no root `colors.txt` still keeps its `TeamColor.bin` record's bytes
  (`team_colors_missing`): there is nothing to rebuild it from. What a `Full` export cannot
  do is remove files: compiled into a CPK above another that holds the team's older content
  (a midcup CPK over the cup's main one), a player or kit it no longer has still loads from
  below. A record whose counted entries repeat a kit
  number is the base game's placeholder for a team with no kit colors (ten white entries
  numbered 0 under a count of 2, 750 of the bundled base's 821 records) and is read as
  holding no kit, which makes a first compile's record the one Red writes. A record holds
  ten entries; when the merge gives more (possible only with an installed entry the export
  format has no slot for, a second goalkeeper kit) the highest-numbered are left out. A kit's
  colors and their findings (`kit_colors_derived`, `kit_colors_missing`) are its task's; a
  `colors.txt` line that does not parse (`color_entry_invalid`) is the deep pass's, for the
  kit files and the root file alike. The root `colors.txt` is no task's file: its colors are
  read while the export's source is open for validation (a `.7z` is not decompressed again
  for them) and applied for every team export planning keeps, which also reports
  `team_colors_missing` for one without the file; a referee export has no record and its
  file is not read. Both color bins are written whole on every
  run that writes a CPK, with every record's header set from its position: the team ID (100 plus the record's
  index) and, in `TeamColor.bin`, the color count 4. A sound record is unchanged by this; an
  installed bin's record whose colors were written over its header (`resources/bins/README.md`
  measures them) gets its header back, so the game finds the team again, and keeps its other
  bytes until an export with colors is compiled for that team; `bin_header_repaired` names the
  bin and the teams, so the corruption is noticed. Kit configs are staged for
  `UniformParameter.bin` compilation (Fox only; Red's `UniformParameter{18,19}.bin` are only its
  bundled per-version fallback bases); when
  the team's kit-FPC status is On, kit slots absent from the export are FPC-patched from the
  installed cup content (see "FPC toggle" in the [Aesthetics export plan](../aesthetics_export/fpc_toggle.md)). A kit's UniColor/UniformParameter entry is applied only
  if that kit's task actually commits — a failed kit never mutates the global bins. On Fox the
  **player appearance tables** accumulate the same way: every compiled player gets his
  `PlayerAppearance.bin` row (the appearance bytes built from his resolved `settings.toml`, its
  defaults for absent keys, nothing from the installed row; Phase 5, Phase 4 passing the bin
  through unchanged) and his `BootsList.bin` and
  `GloveList.bin` rows (the resolved IDs, Phase 5; Phase 4 writes a row only for a committed
  output; a category whose output failed keeps its installed row),
  a compiled player of a `Full` export's team whose folder holds no boots or no gloves loses
  that installed row (a failed output still keeps it; a `Full` export rebuilds its team's rows from the export alone; until
  Phase 5 writes the default-ID rows he then has none, TC-BIN-17), and every other player
  keeps his installed rows, so the output carries complete tables (the
  game reads the highest-priority copy of each whole; the seed rows ride in `4cc_08_bins.cpk`
  ("Stripped save" in the [Save editor plan](../save_editor.md)); the base game's copies are
  wezlib-compressed and the cup's plain, so they are read either way and written plain, which the
  game accepts, as Test 1's CPKs showed). Final bins are
  built after all task outcomes are known. The working bins are fetched from the user's installed
  CPKs, so recompiles update the current cup state: the download folder's `DpFileList.bin` (the
  binary list PES loads CPKs from) is walked from the bottom up — the bottom has the highest PES
  loading priority (Red approximates the same precedence by sorting the DPFL's CPK names in reverse
  alphabetical order) — and, as in Red, the walk starts just below the compiler's own output CPK,
  skipping it and anything with higher priority: this keeps each compilation idempotent even after
  newer midcup CPKs are added further down the list. Each bin is taken from the first CPK the walk
  finds it in, with the supplying CPK reported; the bundled bases only serve from-scratch compiles.
  The same walk locates pre-Fox kit-config bins for FPC patching. Bins updating is unconditional
  (Red's `bins_updating` setting is dropped — see "Resolved decisions").

### 5. Writer (single-threaded)

1. **Override priority** — manifest preflight gives the `overrides/` tree (Red's `sideload/`)
   precedence over export entries at the same output path, and the writer emits accepted
   overrides before export content in canonical order. Overrides target the normal team CPK(s);
   there is no refs-specific overrides tree unless one is introduced later. Every file below
   `overrides/` in the data directory is one CPK entry at its path relative to that folder
   (`overrides/common/etc/TeamColor.bin` is the entry `common/etc/TeamColor.bin`). `compile`
   lists the tree once, before its tasks start, and a tree holding a file is reported as
   `overrides_active` (the folder, the file count). The writer adds the override entries
   first, in path order, so they create the CPK even when no export commits anything. An
   entry a task or the bins then bring at an override's path is left out, the override having
   won, and reported as `duplicate_path` (Warning, on the run, naming the path); the task's
   other entries and its contributions to the bins are kept. An override that cannot be read
   fails the CPK (`cpk_write_failed`): a file the operator put there on purpose is not
   silently skipped. Paths compare exactly, case included, as the CPK's own duplicate check
   compares them: an override is made by copying a file out of a CPK, under the name it had
   there, so folding case would be handling for a spelling nobody produces.
2. **Incremental CPK writing** — the writer accepts completed task batches as they arrive, but
   final payload layout follows canonical manifest order, not arrival order. Per-task atomicity
   discards every staged entry from a failed task. Ordered draining and memory admission must be
   coordinated so waiting later batches cannot prevent the next batch from completing. Written
   entries release their bytes; shared/cache-owned bytes remain charged while retained.
3. **Duplicate invariant check** — all collision winners are decided deterministically by the
   manifest (or, for deep-derived names, inside the owning task) — never by rayon completion or
   writer arrival order. The writer treats any path that still arrives twice as an internal
   invariant violation; canonical manifest order and normalized CPK timestamps make otherwise
   identical builds reproducible. The CPK header's tool-version string is `4cc Studio <version>`
   (the workspace version), so the same release reproduces the same bytes.
4. **Bins last** — after all exports complete, the accumulated bins (committed mutations only) are
   written.
5. **Output modes and targets** — normal (CPK), test (unpacked tree per export in `test_output/`),
   sideload (unpacked PES-folder structure in the game folder's `livecpk\`). Test-mode directory names derive from
   the canonical source key, not from potentially duplicated display names. Multi-CPK mode (the cup
   DLC mode) routes team content into **size-split `teams` parts** plus the bins CPK — see
   "Multi-CPK mode: teams parts" below. Referee content always routes to its own CPK named by
   `refs_cpk_name`, separate from those team targets; `refs_cpk_name` affects only normal CPK mode
   (test/sideload retain their existing per-export layouts). Every emitted CPK name (team, teams part,
   refs) is a validated `CpkStem` (shared `pipeline` type): filename stem only, 1–28 characters from
   ASCII alphanumeric, `_`, `-`, and `.`; no separators, control characters, trailing dot, Windows
   reserved-device names, or user-supplied `.cpk` suffix; uniqueness is checked case-insensitively.
   Deployment/DpFileList/locked-file/marker-summary logic runs per generated CPK, including the refs
   CPK and every teams part.

6. **Multi-CPK mode: teams parts.** Red's multi-CPK split by *content type* (`4cc_40_faces` /
   `4cc_45_uniform` / `4cc_08_bins`) is replaced by a split by *size*: all team content — faces,
   portraits, boots, gloves, common, kits, kit configs, logos — goes into a sequence of **`teams`
   CPKs**, plus the separate bins CPK. The reason is a hard external limit: the cup DLC is versioned
   in Git, and Git for Windows cannot handle objects of 4 GiB or more (`unsigned long` is 32-bit
   under MinGW; the `size_t` conversion is still ongoing upstream in 2026). A **32-team**
   `4cc_40_faces.cpk` already sits at 3.77 GB, so a 48-team cup lands near 5.7 GB — well past the
   limit — while the uniform CPK was barely used. The faces/uniform distinction stops paying for
   itself, and one uniformly filled sequence of parts replaces it.

   - **The slots come from the DpFileList, midcup style.** The official DPFL reserves a run of
     numbered names with the same stem — `4cc_41_teams`, `4cc_42_teams`, … — exactly like the
     `4cc_61_midcup` … `4cc_75_midcup` run (and `4cc_20_stadiums` … `4cc_35_stadiums`). The compiler takes as
     its part slots every DPFL entry matching `{prefix}_{NN}_{teams_cpk_stem}` (the `teams_cpk_name`
     setting names the stem, default `teams`), ordered by number. The stem match is **exact** — the
     text after `_NN_` must equal the stem, so `teams` never claims `teams2` slots — which is what
     makes a **second slot run usable for a side event**: with `teams_cpk_name = teams2` and the
     DPFL reserving `4cc_51_teams2` … in place of the old `4cc_50_other_faces`/`4cc_55_other_uniform`,
     the same compile fills that run instead, sharing only the bins CPK (there was never an
     `other_bins`). There is no parts-count setting:
     capacity is a cup-level decision made once in the artifact every user already receives, and
     the compiler can never emit a part the users' game will not load. How many slots and which
     numbers is the DPFL author's call; the official list reserves **five slots** per run (41 to
     45, and 51 to 55 for `teams2`), which at the default cap gives 15 GB — roughly 2.5× a
     48-team cup.
   - **Filling is deterministic first-fit by canonical team order, one team per decision, made by
     the writer on exact sizes.** Teams are never split across parts: the writer holds a team's task
     batches until the team is complete, then places the whole team — into the current part if it
     fits under `cpk_part_max_size` (TOC included), otherwise it closes the part, opens the next slot,
     and places it there. This costs nothing measurable: the writer already lays out entries in
     canonical order, so a team's batches were waiting for their earlier siblings anyway; holding
     them until the team's *last* batch lands adds a few seconds of retention for one team's compiled
     output (on the order of 100–250 MB, charged to the memory budget like any pending batch) while
     the workers keep processing the next teams. Writing is disk-bound and writes the same bytes
     either way. No planning-time estimate, no sidecar state, and no hard-limit check is needed
     because the decision is made on bytes actually produced. A single team larger than the cap is
     impossible in practice at 3 GB and is a fatal `cpk_team_exceeds_cap` rather than a silent split.
     Whether teams *could* straddle parts was considered (PES merges every CPK into one virtual
     filesystem, so the game would not care) and rejected for tidiness: a team living in one CPK is
     what maintainers expect when they open one. Minimizing Git churn is not a goal here: a cup DLC
     is compiled **once**; midcup changes go to the designated midcup CPKs (one per day, compiled in
     single-CPK mode with `cpk_name` set to that day's slot), so parts are never rewritten and
     cascading cannot occur.
   - **Every slot is always written.** Slots the run does not fill receive the **empty placeholder
     CPK** — the same 6,272-byte zero-entry CPK the official DLC already ships for unused
     `midcup`/`test`/`stadiums` slots (`4cc_68_midcup.cpk` is one). This satisfies the DPFL without
     relying on PES tolerating listed-but-missing files, and it makes superseded content trivial: a
     recompile that needs fewer parts overwrites the stale higher slots with placeholders. The
     placeholder's bytes are emitted by the `cpk` crate and verified byte-identical to the shipped one
     in tests. Running out of slots (content exceeds `slots × cap`) is fatal: `cpk_slots_exhausted`
     names the shortfall so the DPFL author can add slots.
   - **`cpk_part_max_size`** defaults to 3 GB — comfortably under Git for Windows' 4 GiB object
     ceiling, and configurable. It applies to teams parts only; a single-CPK run that
     exceeds it (a midcup day gone wild) is not split — single-CPK names have no slot run — but gets
     `cpk_size_over_limit` (W), since the CPK is fine for PES and only the repository will object.
   - **No legacy DPFL support in multi-CPK mode.** Multi-CPK is run only by the cup maintainers
     assembling a whole DLC from the managers' exports; they are always on the latest tooling. The
     compiler ships the current official DPFL as an embedded template (see "Resolved
     decisions") and, when the installed one lacks the required slots, refuses the run with
     `dpfilelist_outdated` and offers the **DpFileList upgrade** described under "Post-processing".
   `test_output/` and `teamnotes.txt` are resolved beneath `output_folder_path`; a
   CLI positional exports-root overrides `exports_folder_path` for that invocation only.

   **Test mode is Red's step 1.** Red's `1_exports_to_extracted.bat` stopped after pre-processing,
   leaving the checked and edited export in `extracted/` with its original item-folder layout, before
   the files were rearranged into PES paths and packed — the one staged run that stayed useful during
   development, because it shows what the compiler *did* to an export before that is obscured by
   relocation and packing. Blue kept it as `--mode test` (processed model folders emitted under
   `{export}/{itemfolder}/{model}/`, no FPK packing, no game paths). The Studio keeps it under the
   same name for the same reason; a regression suite does not replace it. The suite says *that* final
   output differs from a reference; test mode is for the moments without a reference — developing a
   new pipeline step (glTF conversion, a new version's game paths) or a cup maintainer asking why a
   texture was renamed. Sideload mode is the complementary view: the same content *after* relocation,
   unpacked — what the CPK would contain. In Red's terms it is **steps 1+2** (`extracted/` →
   `patches_contents/`, the PES folder structure that step 3 packed), and its purpose is
   prototyping for aesthetics makers (cup streams never sideload): a runtime in the game serves
   loose files from that structure without a CPK, without touching `download/` or the DpFileList,
   and without restarting the game between iterations, so a modeler can compile → alt-tab → see
   the change. Output goes to `{pes_folder_path}\livecpk\`, the root that runtime serves (written
   with the same access path as deployment to `download/`, elevation included); Studio
   is the folder's only writer, so a sideload run replaces its whole contents with the run's
   output and a file dropped from the export never lingers there. The runtime is not Studio's:
   - **PES 18–21: FoxDen** (`Tools_4cc/FoxDen`, the community's `dinput8.dll` proxy, maintained by
     a close contributor, who takes requests). Its LiveCPK serves `livecpk\` while the folder
     exists, picks up files written while PES runs on their next load, and loads however PES is
     started, so a PES the user launched is served too. Studio neither installs nor drives it.
   - **PES 17: Sider 3**, the user's own, with `livecpk\` as a `cpk.root` line in its `sider.ini`.
   - **PES 15/16: none exists**, so `--mode sideload` with those versions is refused as an
     invalid configuration (exit code 2, the "CLI" section of `settings.md`) and the GUI disables
     the mode. Pre-Fox sideloading is revisited after the first full release (decision entry
     "sideloading through FoxDen").

   The cost Blue paid — a second code path in the coordinator — is contained by deciding the
   **output target once, at manifest planning**, and consuming it at exactly one seam: each task's
   final *materialize* step (relocation to game paths, Fox FPK packing). In test mode that step is
   skipped and the task emits its processed entries with export-relative paths; in normal and sideload
   mode it runs. Everything upstream (checks, model conversion, texture work, name editing) and
   everything downstream (the writer, memory permits, events) is identical. Bins and `teamnotes.txt`
   are emitted in every mode. The loose-folder sink is shared between test and sideload modes and is also
   the harness the unit and parity tests drive the pipeline into — trees are diffed directly, with no
   CPK parsing in the middle — so the sink is test infrastructure that the GUI happens to expose,
   not a feature carried for one dropdown entry. A sink receives each entry by its output-relative
   path and knows nothing of game paths, which are the materialize step's alone, so a new output
   shape (another engine's archive tree, worklog 4.0a) is a new `OutputSink` variant, not a change
   to the coordinator or the writer.

### 6. Post-processing

- **Staging** — every generated CPK is written to a run-scoped staging folder,
  `output/.staging/{run_id}/`, beneath the `output/` folder next to the exe (created on demand). No
  CPK is ever written directly at its final path, so a failed or cancelled run leaves nothing
  half-written where PES or the user could pick it up.
- **Deploy CPKs** — deployment is **always attempted**; there is no `move_cpks` switch (see "Why no
  `move_cpks`" below). Each staged CPK is copied to `{pes_folder_path}/download/{name}.cpk.partial`
  (the download folder is normally on another volume, so this is a copy, not a rename) and then
  renamed over the old CPK; the rename is the only step that needs the old CPK unlocked, and it is
  atomic on the same volume. A marker file lists what was deployed. After the deployment transaction
  commits, the staging folder is deleted — like Red, nothing is left in `output/`.
- **Degraded run: promotion to `output/`** — if deployment cannot happen (`pes_folder_not_found`,
  `old_cpk_locked`, `deploy_target_unwritable`, `dpfilelist_missing`, ...), the staged CPKs are
  **promoted** to `output/{name}.cpk` (same volume, atomic rename, replacing any previous one)
  instead of discarded; the message says so, the savefile step is skipped (see below), and the GUI
  offers **Retry** (for the locked case — the user closes PES and retries the deployment alone, no
  recompile) and **Open output folder** (a status-bar notice action after the run, and a permanent entry in the
  Compile dropdown). The CLI reports the error and the promoted path. `output/` is thus Red's
  `patches_output/` in its new place, reached by degradation or by `--no-deploy` rather than by a
  setting. A machine without PES installed compiles this way automatically: the CPK lands in
  `output/`, the bins are built from the bundled fallback bases, and the run says why.
- **`--no-deploy` (CLI only)** — skips the deployment *and* the savefile step and promotes the
  staged CPKs to `output/` as a clean run, no error. This is the production flag for cup
  maintainers building DLC on a machine whose install must not be disturbed, and for build boxes; it
  is a flag, not a persisted setting, because the GUI's job is installing into the user's game.
- **Why no `move_cpks`.** Red's setting existed because Red's bins came from its bundled
  `Engines/bins/`, so a compile was meaningful without a PES install. The Studio's bins accumulation
  reads the installed CPKs, the savefile step reads the install's save, and the version selector
  checks the install's exe — the pipeline is built around a present install, and a "don't install"
  switch on top of it was carrying real weight: dual-severity messages (`cpk_name_unlisted` W/E),
  conditional checks (`pes_version_mismatch`, `dpfilelist_missing`), a marker message
  (`ref_marker_requires_move`), `run_pes` depending on it, two preflight target sets, and — worst —
  a coherence hole, since the savefile was still written while the CPK it referred to was not
  installed. The promotion path already had to exist for failed deployments, so the setting's only
  unique behavior was reachable at zero cost by degradation; the flag covers the one deliberate case.
- **DpFileList upgrade** — the compiler never edits `DpFileList.bin` as a side effect of a compile
  (the Balls compiler's "check, never write" rule holds for every tool). It does offer one explicit,
  consent-based action. The current official DPFL ships as an embedded template
  (`templates/` override applies, so maintainers can hot-swap it between releases). There is
  one official list, not one per PES version, and its entries are
  `resources/templates/DpFileList.txt`, one CPK per line in load order: the eleven base CPKs
  (`4cc_01_db` to `4cc_18_referees`), `4cc_20_stadiums` and its fifteen midcups
  (`4cc_21_stadiums` to `4cc_35_stadiums`), the size-split slot runs of "Multi-CPK mode"
  (`4cc_41_teams` to `4cc_45_teams`, `4cc_51_teams2` to `4cc_55_teams2`) in place of the
  obsolete faces/uniform split, fifteen midcups (`4cc_61_midcup` to `4cc_75_midcup`) and
  `4cc_99_test`. A numbered run counts from 1; the stadiums run counts from 0 because its
  first CPK is the base one, the midcups following it. Fifteen midcups of each kind are one
  per matchday of the longest cups. Every other layout an install may carry is not the
  official one, including the list of the last PES 21 cup. No installed list has this layout
  yet, so the embedded file is written for the release from that entry list. The game loads
  a list of this length whole: PES 2015 and PES 2021, one per engine, each loaded the last
  CPK of a 53-entry list in an in-game test (the longest installed list has 45), and the
  versions between them are taken to do the same. On every compile
  with a PES folder, at deployment preflight, the installed DPFL's entries are compared with
  the bundled one's, in order:
  - the same entries in the same order → nothing to do;
  - any difference, while every target of this run is listed → `dpfilelist_not_official` (W):
    the run compiles and deploys as usual, because a list edited on purpose, or an old DLC's,
    must stay usable, and the finding says how the list differs (entries missing, entries the
    official list lacks, or the order) and names the upgrade;
  - missing any target of this run (typically the pre-`teams` layout, or a list without
    `4cc_99_test` under the default `cpk_name`) → `dpfilelist_outdated` (E; the run degrades
    like any deployment failure: the game would not load the CPK);
  - separately, a listed CPK with no file in `download/`, other than the run's own target →
    `dpfilelist_cpk_missing` (W), naming the files: when any listed CPK is absent the game
    rejects the whole download folder and runs vanilla, which is why a DLC ships a
    placeholder in every unused slot. The run compiles and deploys; the upgrade below writes
    the placeholders.

  With any of these findings the GUI shows **Upgrade DpFileList**, which asks before it writes; the
  CLI prints the equivalent subcommand, `4cc-studio team-compiler upgrade-dpfl`, and never
  upgrades on its own.
  - The upgrade is an **override, not a merge**: the bundled official DPFL replaces the installed
    file byte for byte, the old one kept as `DpFileList.bin.bak`. The aesthetics community gives
    zero support for custom-edited DPFLs — they have caused a long tail of problems — so preserving
    a user's own entries would be preserving exactly the state the upgrade exists to end. The dialog
    is honest about it: it lists every installed entry that the official list does not contain
    and that is not renamed (below) as "will no longer be loaded", and for each whose `.cpk`
    still sits in `download/` shows the size and offers deletion (renaming a 3.7 GB file as a
    backup is pointless; the user confirms per file, and nothing is deleted without that
    confirmation).
  - **An old DLC's CPKs are renamed by stem**, a best effort so a DLC cut for an older list
    keeps loading under the official one. A name's stem is what follows `{prefix}_{NN}_`.
    Each installed entry the official list lacks, whose file is in `download/`, is matched to
    the official entries of its stem: the same stem, else the stem without trailing digits
    (`stadiums0` is `stadiums`), else one of four fixed aliases, since the teams runs replaced
    the faces/uniform split (`faces` and `uniform` are `teams`; `other_faces` and
    `other_uniform` are `teams2`). Within one official stem the old files, in their list
    order, take the official names in their list order, so load order is kept
    (`4cc_60_midcup` to `4cc_74_midcup` become 61 to 75; `4cc_40_faces` and `4cc_45_uniform`
    become `4cc_41_teams` and `4cc_42_teams`; `4cc_38_balls` becomes `4cc_16_balls`). A file
    already under an official name keeps it and its place in that order. An old file with no
    official name left for it (a sixteenth midcup, a stem the official list does not have)
    is not renamed and is listed as "will no longer be loaded"; a rename never overwrites a
    file. The dialog, and `upgrade-dpfl` without `--yes`, show every rename before anything
    is done.
  - **The upgrade leaves no listed CPK missing.** After the renames, for every official entry
    with no file in `download/` it writes the empty placeholder CPK ("Every slot is always
    written" in "Multi-CPK mode"); it never overwrites a file that is there. When any listed
    CPK is absent the game rejects the whole download folder and runs vanilla, and the
    official list names 53 CPKs where an old DLC ships about 40.
  - The DPFL binary format is small and version-stable enough to own here. Measured on the PES
    2021 and PES 2017 files: a 16-byte header (`u32 0`, `u32 entry_count`, 8 zero bytes), then
    `entry_count` records of 48 bytes each (the CPK file name with `.cpk`, NUL-padded), then an
    all-zero tail of no fixed length (1204 bytes on PES 21's file, 905 on PES 17's) that the reader
    accepts and ignores. The upgrade writes the bundled file byte for byte, so the writer never
    chooses a tail. PES 15, 16, 18, 19 and 20 are checked the same way when an install is
    available. The reader already exists for the bins walk; the upgrade only copies the bundled
    file.
- **Destination writability preflight** — the two failure modes users actually hit (the download
  folder needs elevation because PES sits under `Program Files`; the old CPK is locked because PES
  is still running) are detectable long before Compile, so they are checked **live**, like the
  version selector's exe check, not discovered at the end of a multi-minute run. For every planned
  destination (the single CPK, or every teams slot plus the bins CPK in multi-CPK mode, plus the refs CPK, under
  `download/`): if the file exists, open it for
  writing without truncating; if not, create and remove a probe file in its folder. The failure
  kind is read from the OS error — `ERROR_ACCESS_DENIED` → elevation needed, `ERROR_SHARING_VIOLATION`
  → in use (cross-checked against the core plan's running-PES poll, which names the culprit). The
  check runs at startup, on window focus, whenever a relevant setting changes (`pes_folder_path`,
  `pes_version`, CPK names, `multicpk_mode`), and whenever the PES process poll changes
  state. Results show on the Compile button as a warning badge with the reason in the tooltip and a
  line in the run strip: *"PES 2019 is running — close it before compiling"* / *"The download
  folder needs administrator rights"* with a **Relaunch as administrator** action (the `elevation`
  lib). The button stays enabled — the user may close PES during the compile — except for the
  elevation case, where compiling first would waste the run (relaunching loses it), so the click
  prompts for elevation up front. The deployment stage re-checks regardless; the preflight makes
  its failures rare, not impossible. A `compile` that deploys runs the `download/` probe once
  before it reads any export, so a run that cannot deploy says so first
  (`deploy_target_unwritable`) and still compiles to `output/` (TC-DEP-04).
- **Aesthetics patch** — written beside the output CPK as `aesthetics_patch.toml`: the resolved
  savefile writes for the compiled players (format and rules in "Aesthetics patch" in the
  [Savefile plan](../pes_savefile/operations.md)). On **Fox** it holds names and shirt names only
  (the rest is in the CPK's player appearance tables, "Bins accumulation") and, as on pre-Fox, is
  written beside every CPK the run publishes (none when it publishes none, the previous pair
  staying as it was), so an earlier run's patch never sits beside a newer CPK; the
  savefile builder applies it whole, a managed team's export
  leaving `name` and `shirt_name` out so its players keep their manager's names while an
  autopilot team's export sets them. On
  **pre-Fox** it holds everything: settings.toml settings with
  `name = true` and FPC markers resolved to concrete values, the authored stock boots/gloves IDs
  (else the marker's default) for categories requesting no standalone output, and the
  auto-assigned boots/gloves IDs **only for content that was actually packed**: if a boots/gloves task failed, the affected players
  keep their existing savefile IDs rather than pointing at absent CPK content, while their
  independent valid settings still apply. The patch travels with the CPK: it describes the CPK it
  sits beside, whether or not this machine deployed it. This is what separates the **DLC builder**
  from the **savefile builder**: the former compiles every export and hands over CPK + patch, the
  latter applies the patch in the save editor to the official save, which the former never touches
  (it holds the teams' custom tactics, which only the savefile builder receives). Reported as
  `patch_written` (I) with the path.
- **Savefile update** — the local savefile, when one is configured, is updated by **applying the
  patch just written** through `pes_savefile`'s `apply_patch` — the compiler has no second write
  path, so the local result and the savefile builder's later result are the same bytes by
  construction. Saving uses a `.bak` backup, the same convention as the save editor's. A missing
  savefile stays a warning (`savefile_missing`, naming the patch): even the DLC builder is expected
  to compile against a template save, to test the DLC in PES. **The savefile is never written while
  PES is running**, in any output mode: the game holds its edit data in memory and writes it back on
  exit, so a write made now would not go live without a restart and would be overwritten by whatever
  the player changed in-game before closing. The step is skipped with `savefile_skipped_pes_running`
  (W) telling the user the settings will be applied by the next compile made with PES closed; the
  rest of the run is unaffected. This matters mostly for **sideload mode**, whose whole point is
  compiling while PES stays open — model iterations land immediately through sideloading, and the savefile
  half of the export (on Fox the names; on pre-Fox also boots/gloves IDs and player settings)
  catches up on the first compile after PES is closed. On Fox a names write leaves every
  appearance id as it is, so a stripped local save stays stripped. In normal mode a running PES already fails deployment on the locked CPK, so the rule
  adds nothing there. The check uses the same running-PES detection as the core plan's
  version-selector poll; the CLI performs it once, at the savefile stage. The savefile is likewise
  **skipped whenever the CPKs were not deployed** — a degraded run promoted to `output/`, or
  `--no-deploy`. The incoherence it prevents is pre-Fox's, where a savefile can point at
  boots/gloves IDs whose content is not installed — exactly what the "only for content actually
  written" rule exists to prevent; on Fox a compiled player's IDs live in the CPK's own table
  rows, but the same skip keeps one rule for both engines.
- **Run PES** — optional launch of `PES20{version}.exe`, only after the complete deployment
  transaction succeeds. Built in Phase 8 together with sideload mode's Launch PES button, as one
  launcher: both start the same exe the same way, and the GUI phase is where its only manual
  check lives.

### Run driver shapes (Phase 3)

The Phase 3 subset of the walkthrough above, in the crate's own types (`pub(crate)`; Phase 4's
entry gate may reshape them when run planning arrives). Field types not spelled out are the
`aesthetics_export` types the validated export already carries.

```rust
// plan/: serial, over the identity-resolved exports in ExportId order.
pub(crate) fn plan_run(exports: Vec<(ExportId, ResolvedAestheticsExport)>, version: PesVersion)
    -> PlanReport;
pub(crate) struct PlanReport {
    pub(crate) manifest: BuildManifest,      // Phase 3 has no run-level fatal in planning
    pub(crate) messages: Vec<Message>,       // content_not_yet_compiled, kit_config_*, ...; an
                                             // export the subset gate skips is its DropExport
}
pub(crate) struct BuildManifest {
    pub(crate) tasks: Vec<BuildTask>,        // canonical order: export, then faces by first
                                             // roster slot, then kits by KitSlot
}
pub(crate) struct BuildTask {
    pub(crate) export_id: ExportId,
    pub(crate) team_id: u16,
    pub(crate) kind: TaskKind,
    pub(crate) charge: usize,                // source bytes it reads; see "Memory budget"
}
pub(crate) enum TaskKind {
    Face { folder: /* the mapped player folder */, player_ids: Vec<u32> }, // one task per folder:
                                             // textures once, one face.fpk per roster slot
    Kit { slot: KitSlot, kit: /* the validated kit, all/ inheritance applied */ },
}
// processing/: one task. The coordinator reads its files from the export's content source
// (opened once per export) and hands task and bytes to the pool, which only converts and packs.
pub(crate) fn process_task(index: usize, task: BuildTask, files: TaskFiles, ctx: &CompileContext)
    -> TaskBatch;                            // TaskFiles: the bytes of every file the task reads
pub(crate) struct TaskBatch {
    pub(crate) index: usize,                 // its manifest position: the writer's order
    pub(crate) entries: Vec<(String, Vec<u8>)>, // CPK path, bytes; empty when the task failed
    pub(crate) uniparam: Option<(String, Vec<u8>)>, // a kit's config, applied only if committed
    pub(crate) messages: Vec<Message>,
    pub(crate) permit: Option<Arc<pipeline::Permit>>, // released when the writer has the
                                             // entries; shared by a `.7z` export's tasks
}
```

**Admission.** One coordinator thread acquires each task's permit in manifest order and only
then hands the task to the pool; the writer, on its own thread, commits batches in manifest
order from a reorder buffer and drops each batch's permit after writing it. Every task admitted
before the next one in writer order is already running or done, so the batch the writer waits
for always holds its permit and the run cannot stall on canonical order ("Admission must
remain progress-safe", `core/parallelism.md`). A `.7z` export is opened once for its tasks and
charged once, by the sum of its entries' sizes (`libs/pipeline.md`); its tasks share that
permit and acquire none of their own, since an oversized export permit would otherwise block
its own tasks' requests forever. The coordinator reads all of a `.7z` export's tasks before
handing any to the pool, then frees the decompressed archive and gives each task a share of the
export's permit (an `Arc`), so the charge stays held until the writer has the last of them.
The writer reports each batch's messages as it commits it, so findings come out in manifest
order whatever order the pool finishes in, and reports an export's `ExportProcessed` after its
last task; an export with no task reports it once planning is done. The coordinator, not the
pool, reads each task's files: an
archive is one sequential stream, so pool threads sharing it would only wait on each other,
and the pool's work (conversion, packing) needs no source handle. A file that cannot be read
fails its task with `source_read_failed` (`DropFolder`, naming the file) before the pool sees
it; a failure to convert or pack is `folder_pack_failed`.

**Output.** The CPK is written to `output/.staging/<pid>-<unix ms>/<cpk_name>.cpk` and renamed
to `output/<cpk_name>.cpk` (the run id is per process, so two CLI runs never share a staging
folder); the run's folder is then removed, and `.staging/` too when no other run's folder is
in it, so a finished run leaves nothing but the CPK; a run that emits no entry writes nothing and leaves the previous CPK. The output
folder is created and probed for writing (a file created and removed) before any export is
read; a refusal is the CLI's exit 3 naming the path (TC-CLI-06), not a pipeline message.
`deploy_skipped_by_flag` is emitted, naming the promoted path, when `--no-deploy` is given;
without it Phase 3 promotes the same way and says nothing more (deployment is Phase 4's).

### Game paths reference

| Content | Pre-Fox (PES 15–17) | Fox (PES 18+) |
|---|---|---|
| Faces | `common/character0/model/character/face/real/{id}.cpk` | `Asset/model/character/face/real/{id}/#Win/` |
| Boots | `common/character0/model/character/boots/{id}/` | `Asset/model/character/boots/{id}/#Win/` |
| Gloves | `common/character0/model/character/glove/{id}/` | `Asset/model/character/glove/{id}/#Win/` |
| Shared boots/gloves output's own textures | the folder itself | `Asset/model/character/boots/k{id}/#windx11/` (FMDL paths `/Assets/pes16/model/character/boots/k{id}/`), `…/glove/g{id}/#windx11/` (`/Assets/pes16/model/character/glove/g{id}/`) |
| Kit configs | `common/character0/model/character/uniform/team/{team_id}/` | same |
| Kit textures | `common/character0/model/character/uniform/texture/` | `Asset/model/character/uniform/texture/#windx11/` |
| Collars | `common/character0/model/character/uniform/nocloth/` | `Asset/model/character/uniform/nocloth/#Win/` |
| Common | `common/character1/model/character/uniform/common/{team_id}/` | `Asset/model/character/common/{team_id}/` |
| Referee faces (NN = slot 01-35) | `common/character0/model/character/face/real/referee0NN.cpk` | `Asset/model/character/face/real/referee0NN/#Win/` (`face.fpk` and `face.fpkd`; textures under `referee0NN/sourceimages/#windx11/`) |
| Referee boots | `common/character0/model/character/boots/k99NN/` | `Asset/model/character/boots/k99NN/#Win/` (`boots.fpk` and `boots.fpkd`) |
| Referee gloves | `common/character0/model/character/glove/g99NN/` | `Asset/model/character/glove/g99NN/#Win/` (`glove.fpk` and `glove.fpkd`) |
| Referee common (per referee folder) | `common/character1/model/character/uniform/common/999/{folder}/` (MTL/XML paths `model/character/uniform/common/999/{folder}/`) | `Asset/model/character/common/999/{folder}/sourceimages/#windx11/` (FMDL paths `/Assets/pes16/model/character/common/999/{folder}/sourceimages/`) |
| Portraits | `common/render/symbol/player/` | same |
| Logos | `common/render/symbol/flag/` | same |
| TeamColor.bin | `common/etc/TeamColor.bin` | same |
| UniColor.bin | `common/character0/model/character/uniform/team/UniColor.bin` | same |
| UniformParameter | — | `common/character0/model/character/uniform/team/UniformParameter.bin` |
| PlayerAppearance.bin | — | `common/character0/model/character/appearance/PlayerAppearance.bin` (60-byte rows: player id, then the record's 56 appearance bytes) |
| BootsList.bin | — | `common/character0/model/character/boots/BootsList.bin` ((player id, boots ID) u32 pairs, sorted by id) |
| GloveList.bin | — | `common/character0/model/character/glove/GloveList.bin` ((player id, gloves ID) u32 pairs, sorted by id) |

### Resolved decisions and open questions

Resolved decisions:

- The team info file is dismissed in favor of root `colors.txt`/`notes.txt`; the canonical
  first-word `team_name` (not the complete `export_display_name`) is looked up in `teams_list.txt`.
- `ExportIdentity` keeps normal 701–920 `TeamId` values separate from referees; validation preserves
  all issues while producing a sanitized eligible export; normal-team numbering comes from folder
  names or root `players.txt`, while referee exports require the unified player-folder format's root
  `players.txt` slot mapping.
- Player-owned textures relocate to per-player common while plain ID-based shared outputs retain
  their own textures; a multi-mapped folder (one source folder listed under several `players.txt`
  slots — rarity-repeated referees, or one model backing several team players) is instantiated per
  slot but emits its textures **once**, into a single common subfolder keyed by the source folder's
  name that every mapped slot's model references (Red's in-game-verified referee layout — no texture
  duplication across slots).
- Cross-export duplicate precedence is resolved by run-level manifest preflight; multiple normal
  exports resolving to the same team ID are rejected with `duplicate_aesthetics_export`.
- **ID allocation**: 40-ID per-team blocks (23 player-exclusive + 17 shared), applied independently
  in the disjoint boots and gloves namespaces; `allocation_scheme_version = 1` recorded in build
  metadata. Permanent per scheme version but upgradeable: the sizing, the upgrade mechanism and a
  scheme change's cost on each engine live in "Assigns IDs automatically" of the
  [Aesthetics export plan](../aesthetics_export/player_folders.md).
- **Root normalization**: exactly one usable nested root is flattened; multiple usable roots
  (`nested_root_ambiguous`) and loose-root/nested collisions (`nested_root_conflict`) reject the
  export — an intentional deviation from Red's first-match rule.
- **Model source selection**: target-native first, then glTF, then conversion from the opposite
  native format; in-representation duplicates are rejected (`model_source_ambiguous`).
- **Kit colors fallback**, in order: the kit's `colors.txt`; when missing or yielding fewer than
  two valid colors, derivation from the kit's own main texture (`kit_colors_derived`) — never from
  the placeholder checkerboard; when that is impossible too (no decodable own texture, or a
  placeholder kit), the fixed **magenta/black "no colors chosen" pair is written**
  (`kit_colors_missing`, W): magenta (255, 0, 255) as the first color, black (0, 0, 0) as the
  second. The kit's own main texture is the `kit` texture of its effective set, so one
  inherited from `all/` counts: it is what the kit wears in the game. Loud on purpose: the menu dots and scoreboard strip then show a pair
  nobody would pick, so the omission is seen in the first menu instead of being papered over. The
  alternatives were both quiet failures — skipping the entry leaves whatever a previous cup's bin
  held, and a stand-in such as the team's root colors looks right while being unchosen. The
  pair matches the placeholder texture, so a fully placeholder kit is consistently "unfinished"
  everywhere it appears.
- **`colors.txt` grammar and TeamColor capacity**: one grammar for the root and the kit files,
  one color per line, written in "Root files", "Colors" in the [Aesthetics export
  plan](../aesthetics_export/player_folders.md); the root file fills up to four team colors, the
  `TeamColor.bin` record's capacity.
- **Per-player common path templates**: the referee rows of "Game paths reference" carry Red's
  name-keyed per-referee common layout on both engines, the shape of "Texture relocation to
  common" with 999 as the team ID.
- **FPC markers are player-level presets** (`fpc_on` = hide, `fpc_off` = un-hide, for that player's
  appearance settings only); non-FPC players are valid on FPC teams, so mixed-marker teams are
  ordinary supported usage. Team **kit**-FPC status is two-state (`On`/`Unknown`): any `fpc_on`
  writes the FPC kit values into every config; otherwise supplied configs are left untouched — the
  compiler never auto-reverts FPC values. Kit slots absent from the export are FPC-patched from the
  installed cup content (`kit_config_fpc_adjusted`), so a midcup export can add an FPC player
  without resending untouched kits (see "FPC toggle" in the [Aesthetics export plan](../aesthetics_export/fpc_toggle.md)).
- **The aesthetics patch is the compiler's only savefile write path**: a compile writes
  `aesthetics_patch.toml` beside each CPK it publishes (on Fox only names, the rest travelling in the CPK's
  player appearance tables); a configured local savefile is updated by applying that
  patch, never by a separate write. Reason: the DLC builder and the savefile builder are different
  roles — the official save carries the teams' custom tactics, which the DLC builder must not have —
  so the compile's savefile half has to be a file that can be carried to another machine and applied
  there; making the local write use the same file removes the possibility of the two drifting. A
  missing local savefile remains a warning, not a supported quiet mode, because the DLC builder
  still compiles against a template save to test in PES.
- **Bins updating is unconditional** — Red's `bins_updating` setting is dropped: it only existed
  before "fetch the latest installed bins and update those" was a thing, and skipping bins updates
  is never useful now that recompiles patch the current cup state.
- **Working-bin lookup follows `DpFileList.bin`**: the download folder's DpFileList is walked from
  the bottom up (the bottom has the highest PES loading priority; Red approximates this by sorting
  the DPFL's CPK names in reverse alphabetical order), starting just below the compiler's own output
  CPK — skipping it and anything with higher priority keeps recompiles idempotent even after newer
  midcup CPKs are added — and each bin comes from the first CPK the walk finds it in, with the
  supplying CPK reported; the bundled bases only serve from-scratch compiles (see "Bins
  accumulation").
- **Templates and fallback bins are embedded in the exe, with an override directory**: Red's loose
  `Engines/templates/` and `Engines/bins/` files (refscpk trees, the per-version player skeletons
  from `resources/skeletons/` — Red's single `body.skl` generalized, see the Model conversion plan's
  "Skeleton data" — `face_diff.bin`, the template environment cubemap that the material schema's
  `metal` family falls back to on pre-Fox (see the Unified model format plan's "Textures"),
  `kit_mask.dds` and the checkerboard `kit.dds` for placeholder kits, dummy model/MTL, `generic.fpkd`,
  `fcl_hair_sim.fclo`, the generic kit config, the
  `TeamColor`/`UniColor`/`UniformParameter` fallback bases (in `resources/bins/`, Red's own: PES 18
  builds on `UniformParameter18.bin`, 19–21 on `UniformParameter19.bin`), and the current official
  `DpFileList.bin` (for slot discovery and the DpFileList upgrade — see "Multi-CPK mode: teams
  parts" and "Post-processing") — ~6 MB total) ship inside the binary
  (`include_dir!`), version-locked to the compiler logic that consumes them; Red's
  `file_critical_check` class of missing/mismatched-template errors disappears. A `templates/`
  folder in the data directory shadows embedded resources per file, so cup maintainers can hot-swap
  fallback bins or referee template content between releases; each active override is reported
  (`template_override_active`), and an override file that exists but cannot be read reports
  `template_override_unreadable`.
- **Teams list: embedded upstream, one working copy in the data directory.** The current cup's
  `teams_list.txt` ships *inside* the binary like the templates (upstream), and the only copy on
  disk is `data/teams_list.txt` (working), created from the embedded one on first run and edited
  by the grid's ID cell and by the updater's merge (see the core plan's "Self-update"). We do this,
  not Red's loose file beside the exe, because the grid writes into the list, which makes it user
  data: it must follow the settings file into the user config directory when that mode is chosen,
  or a replaced program folder loses the user's ID assignments; and a single on-disk file leaves
  nothing for a user to edit by mistake. The file keeps Red's name and `.txt` extension: it is the
  file cup organizers already pass around, and `.txt` opens in a text editor on double-click where
  `.tsv` opens nothing, or a spreadsheet that rewrites cells. **Writes are best-effort, never
  elevated**: when the data directory is not writable (the typical case is a portable install
  unpacked under Program Files), the ID cell stays read-only with the reason in its tooltip and
  the updater's merge is reported as skipped (`teams_list_read_only`) — Studio is a portable app,
  and elevating for a one-line edit is handling a corner case nobody should be in. Detection is by
  attempting the write, not by inspecting the path: there is no list of protected directories to
  keep right. **The savefile is a second source for the same merge.** In-game team names in a 4cc
  save are the `/xx/` names, so the savefile's `TeamEntry { id, name }` rows for IDs 701–920 are
  a teams list — and the authoritative one for the cup the user is actually playing, ahead of
  whatever the last release embedded. The [Save editor](../save_editor.md)'s **Export teams list**
  action (it owns the open savefile; the compiler only consumes the list) runs the *same*
  reconciliation as the updater with the savefile as the incoming list (added / kept / overridden,
  reviewed before writing; savefile value wins a conflict), never a blind overwrite. It is
  on-demand, not automatic: the list must keep working with no savefile present
  (`savefile_missing` is a supported mode), and rewriting user data as a side effect of a compile
  would be a surprise. Placeholder rows (`Backup N`, `Invitational N`) come along and are inert.
- **Collar contract**: the replaced stock collar ID comes from the `collar_[ID]` filename
  (`collar_id_invalid` otherwise); two teams
  claiming the same ID is an error caught by a run-wide claimed-ID list during serial planning
  (`collar_id_conflict`, later claimant in canonical order loses); the list starts with ID 105,
  reserved for FPC, and 77, reserved for the referees' marker, so neither can be replaced (a
  team kit using 77 is `kit_collar_reserved`). Custom collars are compatible
  with team FPC — collar rewriting runs after FPC reconciliation and deliberately overrides the FPC
  collar value (see "Collars").
- **Missing savefile stays a warning**: deterministic ID allocation keeps a later savefile-inclusive
  recompile of the same exports consistent, so the CPK step may proceed.
- **Pre-Fox local boots/gloves get no ID of their own**: models embedded as typed face-XML entries
  are ordinary face-XML models with no ID concept and request no standalone output, so a
  local-only category's ID is the authored stock ID or the FPC marker's default (steps 2 and 3
  of "Player settings in exports" in the [Aesthetics export plan](../aesthetics_export/settings_toml.md)).
  Only shared pre-Fox boots/gloves folders (ID-named outputs) get compiler-assigned IDs; a player
  combining local parts with a shared link gets the shared ID written while the local parts ride in the face XML. **Exception — `ingame_face`**: with no face
  folder emitted, gloves and boots parts are relocated to player-specific folders with IDs from the
  per-team block scheme, and those IDs are written to the savefile on pre-Fox; a shared link
  combined with local parts of the same category is merged into that player-exclusive folder (the
  one pre-Fox `link_combined` case), so the player never has two candidate IDs for one savefile slot
  (see "ingame_face marker" in the [Aesthetics export plan](../aesthetics_export/README.md)).
- **Check-cache identity**: recursive manifest fingerprint combined with teams-list and
  validation-settings revisions; the watcher only triggers re-fingerprinting (see "Live
  validation").
- **Shallow archive checks never decompress solid data**: `.zip` per-entry access may read small
  metadata; solid-`.7z` content checks defer to the compile-start deep check, with the roster
  recorded as unread.
- **Virtual-path safety** is specified on the shared `vtree::ScopePath`/`RelativeScopePath` types
  (see the [library crates plan](../libs/README.md)): traversal/absolute paths rejected, separators
  normalized, case/Unicode collisions detected before extraction or packing; `aesthetics_export` uses the
  shared type.
- **Source snapshot** (Phase 4): a source's revision is taken when the reader lists it: for a
  folder, each listed file's size and modified time; for an archive, the archive file's size and
  modified time. `compile` checks it after each task's read: a folder's task checks the files it
  read, an archive's task the archive file (a `.7z`, read once for all its tasks, is checked once,
  after that read). A file that is gone, or whose size or modified time is no longer the
  listing's, aborts the run with `source_changed_during_run`: the staging output is discarded and
  nothing is deployed, so the previous CPK stays as it was. The check comes after the read, not
  before it, so a file saved over while it was being read is caught too. Only what a task reads
  is checked: a file added to a folder after its listing is in no task, so it changes nothing
  the run writes. `check` reads each source right after listing it and checks nothing. The check
  is against a member saving over a file while the cup is compiled, not against a hostile one: a
  replacement of the same size that keeps the modified time (or lands inside the file system's
  time granularity) goes unseen. The GUI watcher (Phase 8) keys its stale-result envelopes on
  the same value.
- **Merge-copy collisions within one child** use the dedicated `merged_texture_conflict`: when two
  parts merged into the same output FMDL (Fox baking of local, shared, and `.common`-linked models
  into one allowed name) copy the same texture destination with different bytes, no canonical winner
  exists — the parts are equal-standing inputs of one model, so the child drops rather than silently
  corrupting one part's look. Group-wide collisions across a player's face/boots/gloves tasks keep
  the `shared_texture_conflict` rule, which does have a canonical winner (face > boots > gloves).
- **Common textures are one task of their export, which no player task waits on (Fox).** Every
  texture directly in `Common/` is converted once, by one task placed after the export's shared
  folders and before its portraits, into the team's Common output. A player package that bakes in a
  Common-linked model reads the Common model itself and points the part's stems that Common holds
  at that output. The task commits on its own, not inside a player's group: its textures serve every
  linking player, so a group holding it would tie all of them together. When it fails, its failure
  is reported and the linking players still commit, as a player linking a shared output that failed
  does; dropping them instead would need the writer to track a dependency across groups, for a
  failure the member fixes in one place, `Common/`. That is the rule for a failure of the
  conversion itself. A Common texture the deep pass finds wrong (`messages.md` "Textures") never
  reaches the task: the file is dropped before planning, and a player whose texture link names
  it follows `link_target_dropped`, like any link to a dropped target.
- **Texture references are stem-based (input)**: all material references (FMDL path tables,
  `.mtl`, `.materials.toml`) name textures by stem (filename without extension) in their
  source/authoring form. Any image format (DDS, FTEX, PNG, JPEG, BMP, WebP, TGA, TIFF) may be
  used as a source for any model format — they are fully interchangeable. The compiler resolves
  each stem to the actual file in the folder and converts to the target format. Two image files
  with the same stem but different extensions is `texture_stem_conflict` (checked in the
  structure pass). FMDL path tables were already stem-based; Studio-format `.mtl` and
  `.materials.toml` write stems; the Export upgrader strips extensions from old `.mtl` texture
  paths when migrating. **Output** `.mtl` files (written at compile time) keep `.dds` extensions
  on texture paths, as the game expects; FMDL output path tables remain stem-based. Omitted or
  blank texture roles are auto-named from the material name + role suffix (fixed table; see
  "Textures" in the [Unified model format plan](../model_format.md)); this happens during deep
  parsing, not the structure pass. A stem resolves in the folder of the material file that set it,
  so material files linked from Common (`*.materials.toml.common`, `*.mtl.common`) name Common's
  textures while local overrides name the player folder's; a texture link (`hair.png.common`)
  makes the Common file resolve under that stem in the player folder. Where a texture resolved
  decides where it is packed (step 6): player folder → relocated to the player's common subfolder;
  Common → referenced in place, once per team.
- **A texture a model names must exist** (`fmdl_texture_not_found`, `mtl_texture_not_found`): a
  player whose texture nobody supplies does not look as its author meant, so the finding is an
  Error and drops the folder. A texture used by one of the model's meshes is supplied when
  (1) its stem resolves in the model's folder, links included; or (2) its path names the team's
  Common output and the export's `Common/` holds the stem; or (3) its path names the team's
  Common output and an **installed CPK** holds that path. Case 3 is the midcup export: a partial
  export ships a changed player whose model points at Common textures compiled on an earlier
  day, and its `Common/` is not in the export. For the same reason a texture `.common` link
  whose target is not in the export (`common_link_missing`) is satisfied by an installed CPK
  holding the texture at the team's Common path; a model link is not, since a Fox merge needs
  the source model. The CPKs searched are those the installed `DpFileList.bin` lists **before**
  the CPK being compiled, nearest first, and never a later one: compiling `4cc_67_midcup` looks
  in `4cc_66_midcup`, then `4cc_65_midcup`, and so on down the list. Each midcup CPK is additive
  over the ones before it, so it may rely on them, and removing a later one must not break it.
  It is the walk "Bins accumulation" makes for the working bins; Red's texture lookup also
  kept only CPKs named `midcup`, `uniform` or `faces`, a filter dropped here. A
  multi-CPK run searches before the first CPK it writes. Each CPK's table of contents is read
  once per run. A path naming anything else (the game's own textures, `dummy_*`) is kept as it
  is and not looked up: the game's data CPKs are not read. When the lookup cannot be made (no
  PES folder, no `DpFileList.bin`, or a list that does not name the CPK being compiled, so
  nothing is known to come before it), case 3 cannot be decided, and the finding is a Warning
  that keeps the folder and says the textures could not be looked for. Red never checked a Fox model's textures at all, and only warned for a
  pre-Fox one; its warning for an FMDL with no team ID in its texture paths
  (`fmdl_no_texture_ids`) has no equivalent, since nothing is found by ID here.
- **Conversion cache engages only for ≤2 teams**: the in-memory texture conversion cache is
  bypassed for compiles of 3+ teams; see the [library crates plan](../libs/README.md) for rationale.
- **Output-mode artifact routing**: `teamnotes.txt` is written under `output_folder_path` in every
  mode. In sideload mode the bins, the `overrides/` files and referee content land at their game
  paths inside `livecpk/`, since that tree is what the CPK would contain. In test mode the bins go
  under `test_output/_bins/` at their game-relative paths, referee content is written per export
  like a team's, and overrides are not applied: test mode shows what the compiler did to an
  export, and an override is not the export's. Normal single-CPK, multi-CPK and refs routing is
  the writer's (step 5 above).

Open questions — **implementation/spec work** (no user preference involved; resolved during the
phase that owns them):

- **Shared/Common dependency graph.** Define cache ownership when a player task needs shared or
  Common models for a Fox merge while those folders also have independent output work (each
  player task reads and converts its parts itself today; the ordering of Fox's Common textures is
  settled above).
- **Task-batch commit protocol.** The required guarantee is fixed: incremental CPK writing must
  never expose entries from a failed atomic task. Define the exact stage/commit boundaries between
  workers, the per-player shared-texture aggregation, and the writer.
- **Cancellation and fatal-output cleanup.** Partial CPKs must be discarded and prior
  published/installed outputs must remain untouched on failure or cancellation, as required by the
  runtime catalog. The mechanism is fixed in "Post-processing" (run-scoped `output/.staging/`,
  `.partial` copy + atomic rename in `download/`, promotion to `output/` on deployment failure);
  what remains is the backup/rollback protocol across multiple CPKs and the savefile (a dead
  run's `.staging/` folder is removed when the next compile starts, TC-DEP-07).
- **Complete memory accounting.** Source sizes do not cover decoded textures, converted models,
  merged meshes, or packed entries; evaluate an RAII budget permit that grows and shrinks with
  actual allocations. Solid 7z archives charge their full decompressed buffer until all dependent
  tasks drain.
- **`teams_list.txt` contract.** Resolved in `teams_list` (Phase 2): tab-separated (not
  whitespace — Blue's `split()` turned `Backup 1` into `Backup`); a header line is required and its
  fields are preserved on write; `ID` and `Name` are located by header position and further
  columns (`MinBootsID`, `MaxBootsID`) are carried verbatim; Name folded through `TeamName::new` on
  load; a line whose Name does not fold is a placeholder kept verbatim and never looked up; a line
  whose Name folds but whose ID is outside 701–920 is an error, as are duplicate IDs or names; a
  UTF-8 BOM is tolerated on read and never written; blank lines are dropped; CRLF or LF read, CRLF
  written (the file Red's current version writes: 220 rows, all ASCII). Still open: atomic writes
  and concurrency between the grid's ID-cell write, the updater merge and the savefile import
  (callers own I/O; Phase 3's one writer, `compile`, only creates a missing file, so the
  question opens with the grid's ID-cell write in Phase 8 and the update merge of Release 0.1.0).
- **Run-result semantics.** Define separate compile and deployment outcome types so scoped errors,
  `pass_through`, failed moves, and failed savefile writes map predictably to GUI state and CLI exit
  codes. The exit codes are fixed (`settings.md` "CLI"): deployment adds none, a degraded run
  exits 1 and an aborted one 3; what remains is the outcome types for GUI state. Deployment is
  transactional across generated CPKs and the savefile, with backups and rollback; PES launches
  only after the complete transaction succeeds. (No system CPK is part of it: the Fox referee
  marker is a reserved collar inside the refs CPK, `blue_port.md` "Referee export processing".)
- **Superseded installed outputs.** Within the teams slot run this is solved: every slot is written
  each multi-CPK run (content or the empty placeholder), so stale parts cannot survive. What remains
  is the single-CPK-versus-multi-CPK crossover (a `4cc_99_test` left from a test compile is not
  touched by a DLC compile, and vice versa — decide whether that needs a message) and the rule that
  an unrecognized user file is never deleted; retired official names are handled by the DpFileList
  upgrade's confirmed per-file deletion.

**Pre-Phase-3 gates** (the normal-team versus referee `ExportIdentity` boundary, sanitized
validated-versus-eligible projection, roster-entry scope/disposition semantics): confirmed
2026-09-30 in the [Aesthetics export plan](../aesthetics_export/object_model.md), "Validation
semantics".

**Pre-Phase-4 gates:** finalize analyzed-output enumeration and namespace allocation (including
folder-internal deep-derived names, without adding a separate `AnalyzedRun` type) and freeze every
model task's planned model-ID assignments before processing implementation begins.

---
