# 4cc Studio — Plan index

The architectural plan is split into one document per concern. Start with the
[core plan](core.md) (overview, crate structure, tool plugin system, GUI shell, parallelism,
development plan, key decisions); the tools and the major library crates have their own documents.

| Plan | Covers |
|------|--------|
| [Core](core.md) | Platform architecture and cross-cutting decisions |
| [Team compiler](team_compiler.md) | The flagship tool: pipeline, planning, processing, packing, deployment, GUI grid, referee support |
| [Aesthetics export](aesthetics_export.md) | The `aesthetics_export` lib and the Studio aesthetics export format: object model + validation progression, player folders, `settings.toml`, FPC markers — shared by the compiler, upgrader, kit config editor, refs arranger, and Team creator |
| [Save editor](save_editor.md) | 4ccEditor + Midcupping successor: editing UI, tactics editor, AATF rules, comparator, transplant; also specs the `team_widgets` lib shared with the Team creator |
| [Stadium compiler](stadium_compiler.md) | Stadium export compilation, fox2, billboards |
| [Export upgrader](export_upgrader.md) | Old-format → Studio-format export migration |
| [Music player](music_player.md) | Rigdio successor: match-day soundboard — .4ccm exports, conditional goalhorns, chants, loudness normalization (also specs the `music_export`/`audio_engine` libs) |
| [Music export editor](music_export_editor.md) | RigDJ successor: .4ccm editing UI, condition forms, validation, audition playback |
| [Match tracker](match_tracker.md) | SEN:P-AI successor: live match stats/events read from the game's memory, match records, and the `match_feed` lib driving the Music player's autopilot |
| [Kit config editor](kit_config_editor.md) | Kit Manager successor: kit config format reference (recovered by disassembly), TOML configs in exports, editing UI with preview and kit menu colors, and the `kit_config` lib |
| [Refs arranger](refs_arranger.md) | Referee allocation aid: per-match referee lists for the Fox referee hook (PES 18–21) with a weighted fallback fill, per-version slot appearance-rate tables and drag-and-drop slot assignment (PES 15–17), players.txt / ref_lists.txt writing, handoff to the Team compiler |
| [Balls compiler](balls_compiler.md) | Ball selection compiler: the balls export format, ball list editing with thumbnails, Ball.bin reference, own CPK output |
| [Team creator](team_creator.md) | New-team wizard for new managers and fake-team invitationals: roster file → legal Team TOML (AATF-checked defaults) + aesthetics export built from embedded starter heads; owns no format, hands off to the compiler, editors and Save editor |
| [Player aesthetics editor](player_aesthetics_editor.md) | Player-folder convenience editor: browse/launch into Blender via the `pes-models` extension's loader (manifest contract), settings.toml editing, in-place glTF conversion, base-model extraction |
| [Unified model format](model_format.md) | The Studio's authoring format: glTF + `PES_bone`/`PES_mesh` extensions, material files (`materials.toml`, name matching, layering, `.common` links) and the complete material schema (shader families, canonical texture roles, per-engine tables) |
| [Model conversion](model_conversion.md) | `model_convert` lib: IR, glTF (import + export), Blender integration, mesh splitting |
| [Savefile](pes_savefile.md) | `pes_savefile` lib: crypto, schema-driven codec, player/team/tactics model, interchange formats |
| [Library crates](libs.md) | The remaining lib crates: format parsers, DDS conversion, color tools (kit-color extraction + picker widget), archives |

Coding rules live in [`../CONTRIBUTING.md`](../CONTRIBUTING.md), domain terms in
[`../GLOSSARY.md`](../GLOSSARY.md), implementation status in [`../WORKLOG.md`](../WORKLOG.md), and
choices made where these plans were silent in [`../DECISIONS.md`](../DECISIONS.md).

## How these documents evolve

The plans are design documents *and* the specification; there is no separate spec tree. Two things
turn a plan section into a specification over the life of the project:

- **Acceptance sections.** Before the phase that delivers a tool, that tool's plan gains an
  "Acceptance" section: stable-ID GIVEN/WHEN/THEN scenarios for the user-observable behavior the
  phase adds (format in `CONTRIBUTING.md` "Testing"; procedure in `AGENTS.md` "Working
  documents"). They are written just-in-time, one phase at a time, not for the whole suite up
  front. Lib crate plans do not get them: for a format crate the fixtures and the parity standard
  are the specification.
- **Present-tense rewrite at phase close.** While a phase is open its sections read as intent
  ("the compiler will…"); when it closes — after the converge audit has confirmed the code meets
  the sections and the acceptance IDs — those sections are rewritten as a description of what
  exists ("the compiler does…"), in place. The rationale ("X not Y because Z") stays; the
  acceptance IDs stay as the behavior contract. Converting per phase rather than in one pass at the
  end keeps the plan true while the work is fresh and avoids a rewrite of ten thousand lines from
  memory.
