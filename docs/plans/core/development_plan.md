# 4cc Studio — Core plan: Development plan

> **Plan:** work not built yet (the open phase's sections may be partly built). What the suite does now is described in `docs/spec/`.

## Development Plan

What exists of this is in `spec/core/development_plan.md` "Development Plan".

**Releases.** The suite first ships as **0.1.0 at the end of Phase 8** ("Release 0.1.0" below):
the Team compiler with compile-time aesthetics (on Fox the output CPK's database tables, on
pre-Fox savefile writing), the Save editor and the Export upgrader, on
Windows, with the CPU texture encoder and the native model formats only. Phase 8 is the line
because it is the first point where the two main tools exist with a GUI, and because its own
verification (community feedback) needs a distributed build; every later tool replaces a legacy
tool that keeps working until then. The phases therefore run in the order 1–6, 8, Release 0.1.0,
7, 9–16: glTF (Phase 7) moves after the release, since no current export carries glTF and it
depends on the Blender `pes-models` codec outside this workspace. Phase numbers are kept, since
every pointer names them. Releases stay `0.x` until Phase 16 closes with **1.0.0**; "first
release" in the plans means 1.0.0, so a "first-release" feature may land after 0.1.0.

### Phase 6: Export upgrader (`tools/export_upgrader`)

- Implement old-format export loading (the only place old-format knowledge lives)
- Implement savefile-driven ID→player mapping
- Implement player folder merging (single-user model folders) and shared folder + link file
  generation (multi-user folders)
- Implement kit restructuring (binary kit configs → `config.toml`, `colors.txt` and the `icon_<N>`
  marker from Team Note colors)
- Implement `settings.toml` generation from savefile aesthetics: every key, `name = true` when the folder's name part equals the save's name and the explicit
  string otherwise, `shirt_name` the save's string (a migrated player must look the same once his
  rows replace his stripped record)
- CLI: `4cc-studio export-upgrader upgrade ./old_export.zip --savefile <EDIT>`

**Verification:** Upgrade the full library of existing old-format exports; compile the results;
compare game-facing output against Red's output for the same exports.

### Phase 7: glTF support

Runs after Release 0.1.0 (see "Releases" above). Besides the codec, it lifts the Team compiler's
interim refusal of glTF sources (Phase 4) and enables the Export upgrader's `--convert-to-gltf`
pass (`export_upgrader.md` step 13).

- Add the `gltf` crate dependency to `model_convert` (Phase 2 built its native-format half)
- Implement `gltf_to_ir` and `ir_to_gltf` with a shared PES_bone/PES_mesh and materials.toml contract
- First settle the `metal` and `glass` Fox defaults and the `shaded` `Basic_CNSR` vectors that
  glTF sources make reachable (`spec/model_format.md` "Checked against the community's reference")
- Coordinate with the Blender project's `pes-models` PES glTF codec (its implementation lives there)
- Test with real Blender-exported models

**Verification:** glTF → FMDL → import in Blender → re-export → compare.

### Phase 8: GUI

The shell slice from Phase 3 (window, sidebar, one tool's view, a plain event log) exists by now;
this phase completes it. Its scenarios are written: the shell's under `gui.md` "Acceptance"
(`ST`), each tool's view under its plan's "Acceptance" (`TC-GUI`, `SE-GUI`, `EU-GUI`).

- Before any view or shell element is coded, its draft (`gui.md` "Prototypes before
  implementation"): `docs/prototypes/shell.html` first, then one per tool as its view comes up
  (`team_compiler.html`, `save_editor.html`, `export_upgrader.html`), each reviewed for
  usability by the lead first (ideas to `PROPOSALS.md`), then refined with the maintainer and
  kept as the view's specification; each tool plan says where its draft sits in that tool's
  build order
- Before the first tool's draft, one usability pass over what members already touch, built in
  Phases 1-6 (the export format and folder conventions, the message catalogs' texts and what
  each asks of the member, defaults, the handoffs from upgrade to compile to install), its
  ideas `PROPOSALS.md` entries ruled before the drafts are drawn around them
- Complete the `studio_core` app shell (sidebar, tool registry rendering, settings menu with per-tool
  collapsible sections, status bar with its three slots and static empty state, window title
  from activities and the held notice, help window with chapter tree, generated Messages topics,
  search, log-panel message links, and `4cc-studio help`), and the first-run data-location dialog
  (`spec/core/distribution.md` "Data location: asked on first run")
- Build the common widgets: progress grid, log panel (auto-scroll, filters, grid cross-linking),
  run strip; and the console printer's progress line and run summary for CLI mode, drawn on
  stderr in a terminal only (`architecture.md` "Event system")
- Build `color_tools`' Phase 8 widgets (`plans/libs/color_tools.md`): the kit menu icon
  rendering (the grid's kit-cell miniatures, pattern-only mode) and the color-picker popup (the
  Save editor's team color fields; the Kit config editor and the Team creator later)
- Implement each tool's `view()`: Team compiler grid (live validation), save editor
  (player/team/tactics editing, AATF checks, comparator, transplant — build order in the [Save
  editor plan](../save_editor.md); the `session/` wiring only the view drives, undo,
  selection, recent files, the FPC toggle and AATF quick actions, lands with it; the tactics editor, card pickers and violations list are built as
  `libs/team_widgets`, which the Team creator later reuses), export upgrader
- Implement the exports folder watcher (`notify` + debouncing) and the check cache in
  `libs/pipeline` (`spec/libs/pipeline.md`; the design is `plans/team_compiler/gui.md` "Live
  validation"), and the Team compiler's `check/` scheduling its two-tier check over them
- Each view over the receivers its tool's `tick` drains (`gui.md` "Event wiring"): the grid,
  the log panel and the run strip read the tool's state, never a shell-owned channel
- Implement cancellation (`spec/core/gui.md` "Cancellation"): the Compile button's Cancel, the pipeline
  checking it between tasks and between a texture task's textures, the cancelled run's staging
  discarded
- The Team compiler's `run_pes` and sideload mode's Launch PES button as one launcher of
  `PES20{version}.exe` (`spec/team_compiler/pipeline.md` "Post-processing", Run PES): both start the
  same exe the same way, and this phase holds its only manual check
- Test on Windows (Linux is a Phase 16 item: no Linux machine is available, and 0.1.0 is
  Windows-only)

**Verification:** the acceptance scenarios above, the automated ones through the headless
harness (a frame's AccessKit nodes, clicks as press and release frames: `studio_core/src/shell/mod.rs`'s
tests are the model), the `manual` ones checked on Windows and recorded in the worklog; user
feedback from the modding community arrives through Release 0.1.0, which follows.

### Release 0.1.0

Cut once Phase 8 closes. It is a milestone rather than a numbered phase: its deliverables are the
slice of Phase 16 an early release cannot do without, pulled forward.

**Ships:** the Team compiler (Phases 3–5: native FMDL/`.model` sources, CPU texture encoding,
compile-time aesthetics, on Fox the output CPK's database tables and on pre-Fox savefile writing),
the Save editor (Phase 5 logic and the full Phase 8 view), and the
Export upgrader (Phase 6, structural migration; its glTF pass waits for Phase 7), as the Windows
portable bundle. Old-format exports reach the compiler through the upgrader; Red's Pre-Studio
preview layout gets no dedicated input path, since few members are expected to use it; an AET
referee export's per-referee `face/`, `boots/`, `gloves/` and `common/` subfolders reach it
through the upgrader too (`player_layout_proto`).

**Does not ship:** glTF (Phase 7), desktop GPU BC7, the Linux bundle, and the updater's
download/swap/rollback (all Phase 16), and the tools of Phases 9–15.

- Release bundling as the `just release <version>` recipe, Windows bundle only (`distribution.md`
  "Distribution: portable .7z bundle", "Changelog and version display"): the `.7z` with a SHA256
  checksum asset, the release body copied from the version's `CHANGELOG.md` section
- `CHANGELOG.md` started; help chapters for the shipped tools and the core chapters (Red's two
  readmes migrated into the Team compiler and core chapters); `4cc-studio help --export` wired
  into the release process
- The updater's check and notice (`distribution.md` "Self-update: in-place binary swap", step 1,
  and step 2 reduced to a notice): the release notes and a link to the release page, with *Skip
  this version* / *Remind me later*; the CLI one-line notice. Plus step 5, the teams-list
  reconciliation on the first start of a new version: it runs on a new binary's first start
  however the binary arrived, so a manual update needs it as much as the swap does
- Maintenance mode, moved here from Phase 16: the `AGENTS.md` section written as 0.1.0 is cut,
  since from this release on the released surfaces (export layout, `settings.toml`/`config.toml`
  keys, CLI flags and exit codes, finding codes) are compatibility contracts. The outline is Phase
  16's bullet. Phases 7 and 9–15 keep the phase procedure; each may ship as a `0.x` release

**Verification:** `just release 0.1.0` produces the bundle and checksum; a recorded manual check
on a clean Windows machine: the bundle extracted to a fresh folder, the first-run dialog, a
fixture export compiled through `quick_compile.bat`, and the update notice shown against a test
release with a higher tag. The release closes with converge and rewrite over its own sections.

### Phase 9: Stadium compiler (`tools/stadium_compiler`)

- Implement the fox2 parser (+ CityHash64) as `libs/fox2`
- Implement the stadium DB bin generation
- Build the stadium pipeline on the shared reader/coordinator/writer scaffolding
- Implement components, billboards, packing, ID remapping
- Implement the tool's view (reuses the progress grid: rows = stadiums, cells = components)

**Verification:** fox2 roundtrip tests (byte-identical, matching FoxTool); output comparison against
the Python stadium compiler on real stadium exports.

### Phase 10: Music tools (`tools/music_player`, `tools/music_export_editor`)

The music tools are independent of phases 2–9 — they touch none of the game-file lib crates and need
only `studio_core` and the app shell, so this phase can run in parallel with (or before) the others
once the shell exists.

- Implement .4ccm parsing/writing + the condition/instruction model as `libs/music_export`
- Build `libs/audio_engine` (kira + symphonia playback, Opus decoder, ebur128 loudness, limiter)
- Music player tool: selection logic, soundboard view, normalization, chants, match state
- Music export editor tool: editing view, condition forms, live preview, validation, audition
  playback

**Verification:** .4ccm round-trip on the community export library; selection-logic scenario tests;
loudness within tolerance of the ffmpeg reference; manual streamer/manager testing.

### Phase 11: Match tracker (`tools/match_tracker`)

Needs only the shell for the tracker itself, plus Phase 10's music player for the autopilot half.

- Transcribe the PES17 memory signatures (team data table, stats table, stat catalog) as per-version
  data; implement the Win32 memory source and the acquisition ladder behind the memory-source trait
- Build the match model: stat diffing, event derivation, clock reconstruction
- Implement `libs/match_feed` (typed pub/sub + snapshot replay) and the tracker view (stats table,
  event log, match tabs)
- Match file save/load (JSON) + autosave; `watch` CLI
- Music player autopilot: feed subscription, auto team loading, feed-driven scoring, event clips

**Verification:** replay tests on recorded table snapshots (including ET/penalty matches);
clock-reconstruction unit tests; scripted-feed autopilot tests; a live simulated stream with
autopilot on.

### Phase 12: Kit config editor (`tools/kit_config_editor`)

Small: the format logic already exists in `libs/kit_config` from Phase 2/4 — this phase adds the GUI
form, the preview, and the container-editing mode.

- Editing form with constraint gating; export-folder tabs and loose-file mode
- Schematic + texture-composite preview (via `dds_convert`)
- UniformParameter container editing (via `uniparam`); `convert`/`check` CLI

**Verification:** bit-identical roundtrips over the bundled UniformParameter contents; byte-parity
edits against Kit Manager on the same file.

### Phase 13: Refs arranger (`tools/refs_arranger`)

Small: the export knowledge already lives in `libs/aesthetics_export` — this phase adds
the slot-allocation editor and the players.txt writer. Needs Phase 3 (`aesthetics_export`)
and Phase 8 (the shell).

- Slot appearance-rate tables as per-version constants (flat rates for PES 17–21, pattern table for
  PES 15/16)
- Slot-list editor with drag-and-drop from the referee storage box; read-only pattern preview with
  hover cross-linking (15/16 mode)
- Lists mode for PES 18–21 (the Fox referee hook decides who appears): one home slot per referee,
  ordered five-position lists built by hand or randomized, the weighted fallback fill of the
  leftover slots, `ref_lists.txt` writer
- players.txt writing (overwrite confirmation) + Team compiler handoff (`ToolContext` tool switch);
  `chances` and `randomize` CLI

**Verification:** players.txt and ref_lists.txt round-trips through `aesthetics_export`; chance-math
unit tests against the measured tables; randomizer and fallback-fill invariants;
interrupted/observed atomic replacement tests (including Windows); a refs export arranged, saved,
and compiled by the Team compiler.

### Phase 14: Balls compiler (`tools/balls_compiler`)

Small: the heavy machinery (FPK/CPK packing, DDS↔FTEX, FMDL path rewriting, IR
conversion) exists as lib crates by this point. Needs Phase 2 (format libs and `model_convert`
for native conversion), Phase 7 (glTF support), and Phase 8 (the shell).

- Balls export parsing + validation (in-tool; the format has one consumer)
- `Ball.bin` writer + embedded `BallCondition.bin`/`ball.fpkd` templates
- Per-ball pipeline: model selection/conversion, texture conversion, FPK packing, thumbnails, CPK
  output
- List editor view (storage box + ordered list, thumbnails, utility buttons); `import-legacy` CLI

**Verification:** output comparison against 2.03 on the same library (accounting for the intentional
drops: no `name.txt`/`ball.fpk.xml` in the CPK); `Ball.bin` byte-parity for identical lists;
interrupted/observed atomic `balls.txt` replacement tests (including Windows); shared `CpkStem`
boundary tests; in-game menu check.

### Phase 15: Player aesthetics editor (`tools/player_aesthetics_editor`)

Small and non-essential (a convenience tool — schedule freely once its
dependencies exist): Phase 2 (format libs, `model_convert`, `pes_savefile` settings and FPC
presets), Phase 3 (`aesthetics_export`), Phase 7 (glTF support), and Phase 8 (the shell). The dual-set superset merge lands here if not already built. The Blender-side
counterpart (the `pes-models` extension's loader module) is the Blender
project's deliverable, not this workspace's; the launch manifest is the
contract between them.

- Plain-folder player browser/launcher + manifest writing + Blender launch; archives require
  explicit extraction before launch, editing, or conversion
- settings.toml editing panel (comment-preserving via `toml_edit`) + `fpc_on`/`fpc_off`/`ingame_face` marker
  toggles; missing settings stay unset until the user edits them
- In-place glTF conversion (superset merge, app-data backup + restore); `launch`/`convert` CLI
- Base-model extraction from installed CPKs + bundled glTF stand-ins

**Verification:** golden-manifest tests; superset-merge field-provenance tests; transactional
conversion publication and backup/restore; comment preservation; compiled-output comparisons on both
engine targets against the chosen geometry and preserved compatible metadata. A dual-set merge
cannot retain both original geometries when they differ; verify that deliberate choice separately
from unintended conversion loss.

### Phase 16: Polish and distribution

Closes with the **1.0.0** release. Release 0.1.0 already delivered the Windows `just release`
recipe, `CHANGELOG.md`, the first help chapters, the update check and notice with the teams-list
merge, and the maintenance-mode section; this phase completes them.

- Release bundling: the Linux bundle joins the `just release <version>` recipe (see
  `distribution.md` "Distribution and updates", `distribution.md` "Changelog and version
  display"); Linux tested on a real machine
- Self-updater, the rest of it: the *Update now* option, download/verify/swap, `4cc-studio
  update`, rollback button (see `distribution.md` "Distribution and updates")
- Desktop GPU BC7 on Phase 4's texture step (`libs/README.md` "First-release desktop GPU BC7"):
  cold pipeline creation and upload/readback measured first, then bounded batches, the Auto/CPU
  setting and the fallback report
- Documentation: every shipped tool's `help/` chapter written for members (a tool shipping in a
  `0.x` release carries its chapter from that release); the bundled readme stays a pointer (the
  window itself is Phase 8)
- Web guide: after the pre-release testing and before 1.0.0, a guide to the whole suite in web
  format, with as many visual elements as can be made (screenshots, annotated folder trees,
  diagrams), above all on preparing a Studio aesthetics export (player folders, subfolders,
  link files, shared folders, `Common/`, kit sets). The `help/` chapters stay the in-app
  reference; the guide is where a member learns the layout before opening the app, because
  the export's flexibility is what a new manager gets wrong first
- Maintenance mode: the `AGENTS.md` section on how the project changes once released is written
  at Release 0.1.0 (not earlier: a procedure written far ahead of its use goes stale) and reviewed
  here for 1.0.0. Agreed outline: the plans stay the spec, edited in place, and `DECISIONS.md` stays the
  decision (ADR) log, split per year past about 4-5k lines, with no per-file ADRs or ADR
  tooling. A fix is a failing regression test first, a plan edit only if behavior changes, and
  `mutants-diff`. A feature runs the phase procedure at small scale (acceptance scenarios,
  converge, present-tense rewrite), as Phases 17-18 already do. Released surfaces become
  compatibility contracts (`settings.toml`/`config.toml` keys, CLI flags and exit codes, the
  export layout, finding codes, `.4ccm`): breaking one needs an Export upgrader migration or a
  clear refusal, never a silent reinterpretation. Converge runs per release over the commits
  since the last tag: the plan sections they touched, and `mutants-diff <last tag>`

### Phase 17: Team creator (`tools/team_creator`, post-release)

Not a first-release gate: its users are the *next* cup's new managers, and it depends on the
widest set of finished pieces — `libs/team_widgets` and `libs/aatf` with `apply_tier` (Phases 5
and 8), the compiler's kit placeholder (Phase 3, its UniColor entry Phase 4) and layout-marker rules (Phase 4), `aesthetics_export`
writing, and the Player aesthetics editor to hand off to (Phase 15). Plan: [Team
creator](../team_creator.md).

- The wizard (team, players, roles, cards, looks, kits, finish) and its help chapter written
  from the wiki's newcomer pages
- Roster file grammar; `legal_defaults` (positions from a stock formation, heights to bracket
  quota, weights, captain, set pieces) over `aatf::apply_tier`; stock formation constants
- The four embedded starter heads in Studio format (in-game head, two cardheads, boxhead)
- Export + Team TOML + README writer (temp sibling, renamed into place, never over an existing
  folder); `create` CLI; hand-offs to the four tools

**Verification:** `legal_defaults` output passes `libs/aatf` for every stock formation and both
height systems; a created export compiles on both engines with only the expected placeholders; the
Team TOML imports into a fixture save and passes the Save editor's AATF check; each starter head
verified in-game on PES 17 and PES 21; wizard and CLI produce identical files for identical inputs.

### Phase 18: Studio Web (post-release)

Not a desktop release gate. Ships once the desktop pipeline has settled, in two steps that match
the eligibility tiers in `spec/core/gui.md` "Browser deployment":

1. **Cheap tier**: the eframe `wasm32` build of the shell plus the file-picking tools (Save editor,
   Kit config editor, Music export editor, Export upgrader checks; the Ruleset editor once Phase 20
   has delivered it). Needs the async file-access
   adapter (`rfd` file picking + a File System Access API adapter for folders), browser-storage
   settings persistence, and hiding of the desktop-only tools. This step alone is the "open a
   bookmark and edit your team" demo.
2. **Pipeline tier**: single-team compilation. Needs the `libs/pipeline` browser variants (async
   reader/writer, async `MemoryBudget`, `wasm-bindgen-rayon` workers), the small in-flight limit, and
   COOP/COEP headers on the host.

Hosting on the community server (nginx, HTTPS, two headers) is evaluated at step 1; CI has been
`cargo check`ing the lib crates for `wasm32` since Phase 1 (guardrail 6), so this phase is UI
wiring and adapters, not porting.

**Verification:** the cheap-tier tools' file round-trips in Chromium and Firefox; a single-team
compile in Chromium producing the same CPK as the desktop CPU path on the same export; embedded
resource download size measured and, if it matters, lazy-fetched behind the template accessor.

### Phase 19: DB generator (`tools/db_generator`, `libs/pesdb`)

Small, and scheduled by need rather than by number: it depends only on Phase 2 (`teams_list`,
`cpk`, `pes_savefile`) and Phase 8 (the shell), and a cup start is when it is wanted. Until then
the Python scripts work. Plan: [DB generator](../db_generator.md).

- `libs/pesdb`: the six database tables the generator writes as typed per-version records
  (`Team`, `Player`, `PlayerAppearance`, `PlayerAssignment`, `Coach`, `CompetitionEntry`), the
  blank-table and Konami-table lists per version; `Ball`/`BallCondition` and `Stadium` join when
  Phases 14 and 9 need them (the Ball.bin writer planned inside the Balls compiler moves here)
- `pes_savefile::ops::populate`: the 19+ placeholder player section written through the codec
  (the scripts' hex-editor procedure), with the base player per version as a fixture
- The tool: teams list → tables (Backup/VGL/Invitational placeholder rows classified into their
  competitions), Konami tables copied from the configured install's data CPK, output as tree or
  CPK, the populated EDIT; `generate` CLI; the view

**Verification:** byte parity with the scripts' output on a three-team list per version (their
trees committed as fixtures); `pesdb` round trips on real Konami tables; populating the PES 20
fixture's stripped player section reproduces it byte for byte; in-game start with a generated
database and EDIT (manual, per version).

### Phase 20: Ruleset editor (`tools/ruleset_editor`, post-release)

Not a first-release gate: its users are invitational organizers, who use 4ccEditor forks until
then. Depends on `libs/aatf` with the schema and the interpreter (Phase 5) and the shell (Phase 8).
Plan: [Ruleset editor](../ruleset_editor.md).

- `libs/aatf`'s writer (`Ruleset` → `RULESET` block, deterministic) and the schema-migration hook
- The tool: start page and templates, the section forms (tiers, heights, cards, universal rules,
  specials with the condition and effects editors, suggestions), opening and saving with logic
  identity, the rules summary, Randomize (Sensible / Weird / Crazy, seeded, every result with a
  legal witness team); `validate`, `update-logic`, `summary` and `random` CLI; the help chapter

**Verification:** as in the Ruleset editor plan's "Development phase": writer round trips on both
embedded rulesets, VGL26 rebuilt from Blank through the forms (manual), logic update and refusal,
no invalid ruleset ever written, every random ruleset's witnesses legal over 1,000 seeds per
preset.

### Unscheduled tools

The suite takes in every tool the community uses, those released after it as well, so an
outside tool that would talk to a Studio tool is planned into the suite and calls it
in-process; we do not give Studio tools inbound APIs for outside programs, because each one
is a server and a wire contract to keep for a consumer the suite is going to absorb anyway.
Tools the suite will take in, with no phase or plan yet:

- **Aesthetics ATF bot** (the maintainer's `Tools_Marco/4cc-aes-atf-bot`, Python; the Aesthetics
  ATF is the semi-manual check of a cup's aesthetics exports, `GLOSSARY.md` "ATF"): drives a
  running PES's Edit mode with a virtual Xbox 360 pad and records every player of each team of
  a teams list through OBS's WebSocket (a still, a dribbling clip, a corner-kick clip), so a
  cup's aesthetics are checked without playing through the rosters by hand. Its pad and OBS
  capture already drive the in-game check harness (`scripts/ingame/game.py` `pad` and
  `obs-shot`).
- **rigbot**: lets managers control their own team's tactics remotely during a match, and
  drives the Rigdio fork's local API (`music_player.md` "Autopilot": team loads, the anthems,
  the victory anthem, the chant, goals), passing SEN:P-AI `--no-rigdio` because it drives
  Rigdio itself. Not published: its author wants to work on it further before publishing the
  source (2026-10-18), so it waits; nothing scheduled depends on it, and until then what it does
  is known only from the forks' READMEs.

---
