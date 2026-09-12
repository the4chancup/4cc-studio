# 4cc Studio - Glossary

One line per term, with the plan section that owns the full definition. This file defines nothing
on its own: if a line and its plan section disagree, the plan is right and the line gets fixed.
Add a term when a session had to look it up or an agent misused it. Within a group, put related
terms next to each other rather than in alphabetical order; adjacency is what makes them stick.

## Project and process

| Term | Meaning | Owner |
|---|---|---|
| **4cc** | The 4chan Cup community: PES-based virtual football cups between board teams. Studio is its tool suite. | `plans/core.md` "Overview" |
| **Acceptance ID** | Stable identifier (`TC-KIT-03`) of one testable, user-observable behavior in a tool plan's "Acceptance" section; tests cite it. | `CONTRIBUTING.md` "Testing" |
| **Converge** | End-of-phase audit of code against the plan section and acceptance IDs; gaps become worklog steps. | `AGENTS.md` "Working documents" |
| **Finding** | A lib crate's structured report of a condition (stable code + context), free of user-facing text; tools map findings to messages. | `AGENTS.md` "Fundamental concepts" |
| **Gate** | One of the verification commands that must pass before a step is done (`fmt`, `clippy -D warnings`, `test`, `wasm32` check). | `CONTRIBUTING.md` "Testing and verification" |
| **Legacy tools** | Red, Blue, 4ccEditor, Midcupping, Rigdio/RigDJ, SEN:P-AI, the converters, the Blender addons: evidence of required behavior and formats, never source to translate. | `plans/core.md` "Relationship to legacy tools" |
| **Phase** | A numbered unit of the development plan (1–17), each with its own verification; the worklog tracks steps within it. | `plans/core.md` "Development Plan" |
| **Plan** | The documents under `docs/plans/`: the *why* and the spec. Future tense while a phase is open, rewritten present tense when it closes. | `plans/README.md` |

## Platform

| Term | Meaning | Owner |
|---|---|---|
| **Lib crate** | A crate under `crates/libs/` holding one format or one shared concern; depends only on other libs; returns findings, never messages. | `plans/core.md` "Crate structure" |
| **`PipelineEvent`** | The typed progress/result event tools emit (`ExportStarted`, `FolderStatus`, `Message`, `Progress`, `Complete`); GUI and CLI render it. | `plans/core.md` "Event system" |
| **`ScopePath`** | `vtree`'s canonical, platform-neutral, validated path inside an export; addresses events, findings and grid cells. | `plans/libs.md` "`vtree` path types" |
| **Stale-result envelope** | `PipelineEventEnvelope` carrying `run_id` and `export_revision`, so results from an outdated filesystem snapshot cannot overwrite newer state. | `plans/core.md` "Event system" |
| **`studio`** | The single binary: registers tools, launches the GUI or dispatches `studio <tool-id> <command>`. | `plans/core.md` "Crate structure" |
| **`studio_core`** | The platform crate: shell, `StudioTool` trait, settings framework, common widgets, event types. Holds only what runs with zero tools installed; its module tree is closed. | `plans/core.md` "Architecture" |
| **`StudioTool` / `ToolContext`** | The plugin trait every tool implements (id, settings, `cli_run`, `gui_run`) and the platform services handed to it. | `plans/core.md` "Tool plugin interface" |
| **Studio Web** | Post-release, deliberately lite browser edition; the reason every lib crate stays `wasm32`-checkable. | `plans/core.md` "Browser deployment" |
| **Tool crate** | A crate under `crates/tools/`, one per tool, with the fixed skeleton `lib.rs` / `settings.rs` / `cli.rs` / `messages.rs` / `view/`. Never depends on another tool crate. | `plans/core.md` "Workspace guardrails" |
| **Tool id** | The hyphenated stable identifier of a tool (`team-compiler`), used in CLI subcommands and `MessageCode`. | `plans/core.md` "Tool plugin interface" |

## Exports and compilation

| Term | Meaning | Owner |
|---|---|---|
| **Aesthetics export** | A team's folder or archive of player folders, kits, portraits and `settings.toml`; the Team compiler's unit of work and the project's primary motivation. | `plans/aesthetics_export.md` |
| **Referee export** | An aesthetics export under the reserved `/refs/` team name; prepared by the Refs arranger, compiled by the Team compiler. Only one may be enabled. | `plans/refs_arranger.md`; `plans/team_compiler.md` |
| **Referee hook** | `fox_hook`'s `03_refmod.lua`: a trampoline on the Fox games' referee slot-writer that forces or remaps the referee id for each of a match's five positions, so which referee appears is decided by a script rather than by the game's slot draw (PES 18–21). | `plans/refs_arranger.md` "The Fox referee hook" |
| **Referee list** | One line of `ref_lists.txt`: five distinct referee slot ids, one per match position, applied by the referee hook's script per match in file order. Written by the Refs arranger beside `players.txt`. | `plans/refs_arranger.md` "The lists file" |
| **Balls export** | The third export kind: a ball selection compiled by the Balls compiler into its own CPK. Its name's first word is `balls`. | `plans/balls_compiler.md` |
| **`BuildTask`** | One planned unit of compile work (a folder, a kit, a bin update) with its owning scope and planned model-ID assignments. | `plans/team_compiler.md` "Planning" |
| **Deep / shallow check** | Shallow: validate what is visible without extracting an archive (`PartialOk`/`PartialError`); deep: full validation (`FullOk`/`Error`). | `plans/core.md` "Event system" (`FolderStatus`) |
| **Export revision** | The pinned identity of an export's source contents for one run; a change mid-run is `source_changed_during_run` (`AbortRun`). | `plans/team_compiler.md` "Message catalog" |
| **Export upgrader** | The tool that migrates old-layout exports to the Studio player-folder format once; the compiler does not read old layouts. | `plans/export_upgrader.md` |
| **FPC** | Full Player Customization: the savefile + kit-config settings that hide a player's default body so the model supplies it. Toggled per player by `fpc.on` / `fpc.off` marker files. | `plans/aesthetics_export.md` "FPC toggle"; `plans/save_editor.md` |
| **Music export (`.4ccm`)** | A team's match-day audio package (anthem, goalhorns, chants, conditions) for the Music player. The `.4ccm` format stays canonical. | `plans/music_player.md` |
| **Help chapter** | A tool's part of the help window: the Markdown topics in its crate's `help/` folder (`StudioTool::help()`), plus a Messages topic generated from its `messages.rs` catalog. | `plans/core.md` "Help window" |
| **Kit folder** | One kit's `config.toml`, `colors.txt` and textures under `Kits/`, named `<slot>[ - <label>]` (`p1 - Lakers`); `Kits/all/` holds textures every kit inherits per stem. | `plans/aesthetics_export.md` "Kits" |
| **Kit layout** | Which engine's kit UV layout a main kit texture is drawn for (`KitLayout`: `PreFox` for PES 15–17, `Fox` for 18–21); the shirt is identical, the sock and shorts islands differ. Declared by an empty `pre-fox` / `fox` marker file in the kit folder; absent = the compile target's. | `plans/aesthetics_export.md` "Kit layout marker" |
| **Kit slot** | A kit's position (`KitSlot`): `p1`–`p9` player kits, `g1` goalkeeper; the slot part of a kit folder name and of the game's `u0{team}{slot}` texture names. | `plans/aesthetics_export.md` "Kits" |
| **Link file** | An empty file whose name is the reference: `Crocs.boots`, `Longhair.face`, `Keeper.gloves` point a player folder at a shared folder of that category (at most one per category); `<file>.common` points at a file in `Common/` (see "Common link"). Every link file accepts a stray `.txt` suffix (`Crocs.boots.txt`), because Windows hides known extensions and Notepad adds one. | `plans/aesthetics_export.md` "Player folders" (shared models); `plans/model_format.md` "Link files" |
| **`NO_USE` marker** | Root `NO_USE` / `NO_USE.txt` file that disables an export source; it is skipped and omitted from the grid. | `plans/team_compiler.md` "Message catalog" |
| **Parity golden standard** | Red's CPK output on the fixture set: the Team compiler's output must match it byte for byte where the plan says so. | `AGENTS.md` "Fundamental concepts" |
| **Player folder** | One player's model, textures and per-player `settings.toml` inside an aesthetics export. | `plans/aesthetics_export.md` "Player folders" |
| **Player slot** | A roster position (`PlayerSlot`) in `players.txt`; `DropSlot` discards exactly one normalized assignment. | `plans/team_compiler.md` "Message catalog" |
| **Roster file** | The Team creator's input: plain text, one player per line in shirt-number order, optional position / medal / `captain` tokens after two spaces or a tab, optional leading number. | `plans/team_creator.md` "2. Players" |
| **`settings.toml` / `config.toml` / `materials.toml`** | User-edited TOML inside exports (player aesthetics; kit config; model materials). Carry app-injected per-field comments; written only through `toml_edit`. | `plans/aesthetics_export.md`; `plans/kit_config_editor.md`; `plans/model_format.md` |
| **Starter head** | A ready player folder embedded in the Team creator (in-game head, cardhead, boxhead) that needs at most one image; the tool's word for the wiki's replace-a-texture model templates. Not a "preset": in the community that word means a tactics preset. | `plans/team_creator.md` "Starter heads" |
| **Team creator** | The new-team wizard: name + roster file → legal Team TOML (AATF-checked defaults) and a compilable aesthetics export built from starter heads. Owns no format, keeps no state, hands off to the compiler and editors. | `plans/team_creator.md` |
| **`team_widgets`** | The egui widget lib over `pes_savefile`'s team model (tactics pitch and instructions, card pickers, AATF violations list) shared by the Save editor and the Team creator; takes `&mut` model, returns what changed, never sees a session or a file. | `plans/save_editor.md` "`libs/team_widgets`" |
| **Team name** | A 4cc team's `/xx/` board name (`TeamName`), the canonical identity of an export: derived from the first word of the export's folder or archive name, lowercased and wrapped in slashes; `/refs/` and `/balls/` are reserved. In a 4cc savefile the in-game team name field holds the same `/xx/` name; the save editor edits that field, and "Export teams list" writes it into `teams_list.txt`. | `plans/libs.md` "`libs/teams_list`" (type, fold); `plans/team_compiler.md` "Export display name and team name" (derivation) |
| **Team ID** | The numeric PES team identifier (`TeamId`), valid 701–920 for normal teams; resolved from the team name via `teams_list.txt`. | `plans/libs.md` "`libs/teams_list`"; `plans/team_compiler.md` "Export identity resolution" |
| **Templates override** | A `templates/` directory whose files shadow embedded CPK/bin templates; reported per file per run. | `plans/team_compiler.md` "Message catalog" |

## Messages

| Term | Meaning | Owner |
|---|---|---|
| **Disposition** | The processing consequence of a message, independent of severity: `Keep`, `DropFile`, `DropSlot`, `DropFolder`, `DropExport`, `AbortRun`. | `plans/team_compiler.md` "Message structure" |
| **`DoneWithErrors`** | Folder outcome when content was written despite Error-level findings (`pass_through`, or `DropFile` with the rest processed). | `plans/core.md` "Event system" |
| **Message** | A tool's structured report to the user: `MessageCode` + severity + disposition + scope + context fields. Text is rendered from the tool's catalog, never baked in. | `plans/team_compiler.md` "Message catalog" |
| **Message catalog** | A tool's table of message codes with template text and remediation hint; `messages.rs` in each tool crate. One condition, one ID. | `plans/team_compiler.md` "Message catalog" |
| **`pass_through`** | Setting that turns eligible content-level `DropFile`/`DropFolder` dispositions into keep-with-flag, as in Red; never affects `DropSlot`/`DropExport`/`AbortRun`. | `plans/team_compiler.md` "Message catalog" |
| **Scope** | What a message is about: `Run`, `Export`, `Folder`, `File`, `RosterEntry`; drives grid-cell mapping. | `plans/team_compiler.md` "Message structure" |
| **Severity** | Presentation/filtering level: `Info` < `Warning` < `Error` < `Fatal`. Never decides processing consequence. | `plans/team_compiler.md` "Message structure" |

## Formats and engines

| Term | Meaning | Owner |
|---|---|---|
| **CPK** | CRI archive format PES loads; the compiler's output container. May hold CRILAYLA-compressed entries. | `plans/libs.md` |
| **CRILAYLA** | CRI's bitstream LZ compression used inside CPKs. | `plans/libs.md` "Format references" |
| **DDS / FTEX** | DDS: the texture container for pre-Fox; FTEX: Fox Engine's texture container. `dds_convert` handles BCn encoding in-process (no texconv). | `plans/libs.md` |
| **FMDL / FPK** | Fox Engine model format and its package container (PES 18–21). | `plans/libs.md`; `plans/model_conversion.md` |
| **fox2** | Fox Engine entity files (stadiums), written from `fox2.xml` sources; strings are CityHash64-hashed. | `plans/stadium_compiler.md` |
| **Fox / pre-Fox** | The two PES engine generations: pre-Fox (PES 15–17, `.model` + `.mtl`, DDS) and Fox (PES 18–21, FMDL/FPK/FTEX). | `AGENTS.md` "Fundamental concepts" |
| **glTF authoring format** | Studio's single model authoring format: glTF plus optional `PES_bone` / `PES_mesh` extensions and `materials.toml`; converted to either engine at compile time. | `plans/model_format.md` |
| **Common link** | A link file named after a file in the export's `Common/` folder plus `.common` (`torso.glb.common`, `body.materials.toml.common`, `hair.png.common`), making that model, material file or texture resolve in the player folder as if it were local. A texture reached through Common is packed once per team and referenced in place. | `plans/model_format.md` "Link files" |
| **Global model file** | The user-facing name for a `.glb` authoring-format model, in the community's family of "fox model file" (`.fmdl`) and "pre-fox model file" (`.model`). A nickname, not an expansion of "glb"; plans and code say glTF. | `plans/model_conversion.md` "What users call it" |
| **IR** | `model_convert`'s intermediate representation; cross-format conversion goes through it, same-format work stays in the format crate's `ops/`. | `plans/model_conversion.md` |
| **Kit config** | The per-kit binary parameter block (collar, shorts, numbers…), reverse-engineered and editable as TOML `config.toml`. | `plans/kit_config_editor.md` |
| **`.model` / `.mtl`** | Pre-Fox mesh and material files. | `plans/libs.md`; `plans/model_conversion.md` |
| **uniparam** | The `UniformParameter` container in the team bins (kit configs, colors) the compiler accumulates into. | `plans/libs.md`; `plans/aesthetics_export.md` "FPC" |
| **WESYS** | zlib wrapper with a custom header used by PES for some game files. | `plans/libs.md` "Format references" |

## Savefile

| Term | Meaning | Owner |
|---|---|---|
| **AATF rules** | The 4cc rule set constraining player stats and medals; the save editor checks and applies it from a configurable parameter file (`libs/aatf`). Expansion of the acronym is not given in the plans. | `plans/save_editor.md` |
| **Aesthetics patch** | `aesthetics_patch.toml`, written by the Team compiler beside every output CPK: the resolved savefile writes (settings, names, FPC values, boots/gloves IDs, edit flags) for every compiled player. Applied to a save by the save editor; also how the compiler updates the local save. Lets the DLC builder and the savefile builder be different people. | `plans/pes_savefile.md` "Aesthetics patch" |
| **Aesthetics fingerprint** | Decoded appearance data of a player used to diff saves (Midcupping's `compare-saves`). | `plans/pes_savefile.md` |
| **Aesthetics transplant** | Copying a player's appearance block from one save to another (Midcupping's `transplant-aesthetics`). | `plans/pes_savefile.md` |
| **Comparator** | Save-to-save diff of gameplay and aesthetics (4ccEditor's comparator merged with Midcupping's). | `plans/save_editor.md` "Comparator" |
| **`EditFile`** | `pes_savefile`'s decoded, editable representation of an `EDIT00000000` save. | `plans/pes_savefile.md` |
| **Interchange formats** | Team TOML (Studio) and legacy `.4ccs` / `.4cct` files for moving team data between saves. | `plans/pes_savefile.md` |
| **`PlayerSettings`** | The per-player aesthetic settings written from `settings.toml` into the savefile at compile time. | `plans/pes_savefile.md` |
| **Savefile (`EDIT00000000`)** | PES's encrypted edit-data file holding players, teams, tactics; per-version crypto and schema in `pes_savefile`. | `plans/pes_savefile.md` |

## Stream-side tools

| Term | Meaning | Owner |
|---|---|---|
| **Autopilot** | Music player mode where `match_feed` events (goals, cards, halves) drive the soundboard without operator input. | `plans/music_player.md` |
| **Anthem / victory anthem / goalhorn / chant** | The audio slots of a music export: team music; post-win music (with selectable `special` variants); per-player or default (`goal`) scoring music; crowd loop faded out on a timer. | `plans/music_player.md` |
| **`match_feed`** | In-process typed pub/sub of live match events read from the game's memory, with snapshot replay; feeds the tracker view and the Music player. | `plans/match_tracker.md` |
| **Match record** | `.match.json`: the tracker's saved per-match stats/events (replaces binary `.sen`). | `plans/match_tracker.md` |

## Legacy tools (evidence only)

| Term | Meaning | Owner |
|---|---|---|
| **4ccEditor** | C++ PES save editor; source for savefile schema and editing behavior. | `plans/save_editor.md` |
| **Blue / Red** | The two Python AET (aesthetics export) compilers. Red is the CPK-output parity standard; Blue supplies parser variants. | `plans/team_compiler.md` "Relationship to the older compilers" |
| **Midcupping** | Python scripts for save crypto, transplant and aesthetics diff. | `plans/pes_savefile.md` |
| **pes-fmdl / pes-model** | Blender addons for FMDL and `.model`; their hot paths move into the `python_bindings` wheel. | `plans/model_conversion.md` "Blender integration" |
| **Rigdio / RigDJ** | Music player and music export editor being replaced. | `plans/music_player.md`; `plans/music_export_editor.md` |
| **SEN:P-AI** | Match tracker being replaced. | `plans/match_tracker.md` |
